//! Stage 4 embedded-CPython runtime (spec §5). **Every PyO3 line in the interpreter lives here**,
//! behind `#[cfg(feature = "python")]`. Nothing outside this module names a PyO3 type; with the
//! feature off, `root.python(...)` yields `ForeignErr::Unavailable` (DL1307) and this module reduces
//! to the small feature-agnostic shims at the bottom.
//!
//! ## Ownership / GIL model (the sound embedding story)
//!
//! * The embedded interpreter is a **process-global singleton**, prepared lazily by
//!   [`ensure_available`] (`pyo3::prepare_freethreaded_python`, i.e. `auto-initialize` OFF) on the
//!   first grant-checked `root.python(load)` call. Preparation is guarded by `catch_unwind`, so an
//!   interpreter that cannot start becomes a returned `Err`, never a crash (spec §5.1 / DL1307).
//! * A `PyObj` runtime value wraps a [`PyObjVal`] = `Py<PyAny>` — a **GIL-independent, reference-
//!   counted** owned handle. `Py<T>` may be stored in a DeluluLang `Value`, cloned, and moved
//!   between interpreter steps *without* holding the GIL; it only needs the GIL to be dereferenced.
//! * **Every** Python touch happens inside `Python::with_gil(|py| …)`. Within that closure a
//!   `Py<PyAny>` is `bind`-ed to a GIL-bound `Bound<'py, PyAny>`, operated on, and either marshalled
//!   back to a scalar `Value` or re-`unbind`-ed to a fresh `Py<PyAny>`. **No `Bound<'py>` ever
//!   escapes the closure** — only GIL-independent `Py<T>` or plain Rust scalars cross back into the
//!   interpreter, so a `PyObj` sitting in a DeluluLang variable between operations holds no borrow
//!   and no lock. Dropping a `PyObjVal` outside the GIL is safe: PyO3 defers the ref-count decrement
//!   until the GIL is next held.
//!
//! ## Honesty (spec §5.3 / §10, carried verbatim into `delulu explain E-DL1305`)
//!
//! The import allowlist gates **the interface** — what the program may reach for *by name*. It does
//! not bound what Python transitively imports or does once running: embedded Python has full process
//! authority at the OS level. The real bounds are (a) the reachability gate (no `Cap[Python]`, no
//! Python at all) and (b) the process/worker/microVM layer (Stage 5). The word "sandbox" does not
//! apply here.

use crate::value::Value;

#[cfg(not(feature = "python"))]
use delulu_diag::Span;
#[cfg(not(feature = "python"))]
use crate::value::Fault;

/// Whether an import `name` is permitted by a granted allowlist. Patterns are either an exact module
/// name (`"numpy"`) or a `prefix.*` wildcard (`"numpy.*"` matches `numpy` and any `numpy.<sub>`).
pub fn allowlist_allows(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|pat| {
        if let Some(prefix) = pat.strip_suffix(".*") {
            name == prefix || name.starts_with(&format!("{prefix}."))
        } else {
            name == pat
        }
    })
}

/// Build a `std.py.PyErr { kind, message }` record value (spec §5.2). `message` is UNTRUSTED foreign
/// data (invariant 21): it is length-bounded to `max` bytes here, at the boundary, reusing the
/// `--foreign-max-ret` ceiling the C FFI uses.
pub fn py_err_value(kind: &str, message: &str, max: usize) -> Value {
    let msg = bound_message(message, max);
    Value::Record {
        name: std::rc::Rc::from("PyErr"),
        fields: std::rc::Rc::new(std::cell::RefCell::new(vec![
            ("kind".to_string(), Value::str(kind.to_string())),
            ("message".to_string(), Value::str(msg)),
        ])),
    }
}

/// Length-bound an untrusted foreign message to `max` bytes (invariant 21), on a char boundary, with
/// an explicit truncation marker so a silent cut is never mistaken for the whole message.
fn bound_message(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… (truncated at --foreign-max-ret = {max} bytes)", &s[..end])
}

// ===========================================================================
// Feature ON: the real PyO3 implementation.
// ===========================================================================

#[cfg(feature = "python")]
pub use with_python::{call_pyobj, call_python_cap, ensure_available, PyObjVal};

#[cfg(feature = "python")]
mod with_python {
    use std::sync::OnceLock;

    use pyo3::prelude::*;
    use pyo3::types::{PyList, PyTuple};

    use delulu_diag::Span;

    use super::{allowlist_allows, py_err_value};
    use crate::value::{CapScope, CapVal, Fault, Value};

    /// A `PyObj` runtime value: a GIL-independent owned handle to a Python object. Safe to store and
    /// drop off the GIL (PyO3 defers the decref). Cloning bumps the reference count, which *does*
    /// require the GIL — so `Clone` is implemented by hand via `clone_ref` (the interpreter clones
    /// `Value`s freely, so a plain derive is impossible: `Py<T>` is deliberately not `Clone`).
    pub struct PyObjVal(pub Py<PyAny>);

    impl Clone for PyObjVal {
        fn clone(&self) -> Self {
            Python::with_gil(|py| PyObjVal(self.0.clone_ref(py)))
        }
    }

    /// Prepare the embedded interpreter lazily (spec §5.1). Idempotent and guarded: an interpreter
    /// that cannot start is a returned `Err(reason)` (→ `ForeignErr::Unavailable`, DL1307), never a
    /// crash. `prepare_freethreaded_python` is the `auto-initialize`-OFF entry point.
    pub fn ensure_available() -> Result<(), String> {
        static READY: OnceLock<bool> = OnceLock::new();
        if *READY.get_or_init(|| {
            std::panic::catch_unwind(|| {
                pyo3::prepare_freethreaded_python();
            })
            .is_ok()
        }) {
            Ok(())
        } else {
            Err("embedded CPython failed to initialize".to_string())
        }
    }

    /// Dispatch a `Cap[Python]` operation (spec §5.2): `import`, `of_int/of_float/of_str/of_bool`,
    /// `list`, `to_int/to_float/to_str/to_bool`. The allowlist is carried in the capability's scope
    /// (minted by `root.python`); `import` off the allowlist is a runtime `PyErr` value (DL1305), not
    /// a fault. A Python exception becomes a `PyErr { kind, message }` value; a `Fault` is reserved
    /// for checker-guaranteed-impossible shapes.
    pub fn call_python_cap(cap: &CapVal, method: &str, args: &[Value], max: usize, span: Span) -> Result<Value, Fault> {
        let allowlist = match &cap.scope {
            CapScope::Python { allowlist } => allowlist.clone(),
            _ => return Err(Fault::at("DL0907", "Cap[Python] with a non-Python scope (checker bug)", span)),
        };
        Python::with_gil(|py| match method {
            "import" => {
                let name = str_arg(args, 0, span)?;
                if !allowlist_allows(&allowlist, &name) {
                    // DL1305 as a runtime PyErr value (criterion 5). The interface gate refuses the
                    // import BY NAME before Python ever sees it — the honesty note (§5.3) explains why
                    // this bounds the interface, not transitive behaviour.
                    return Ok(Value::err(py_err_value(
                        "ImportNotAllowed",
                        &format!("import of `{name}` is not in the granted `foreign.python` allowlist (DL1305)"),
                        max,
                    )));
                }
                match py.import(name.as_str()) {
                    Ok(module) => Ok(Value::ok(pyobj(module.into_any()))),
                    Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
                }
            }
            "of_int" => Ok(pyobj(int_arg(args, 0, span)?.into_pyobject(py).unwrap().into_any())),
            "of_float" => Ok(pyobj(float_arg(args, 0, span)?.into_pyobject(py).unwrap().into_any())),
            "of_str" => Ok(pyobj(str_arg(args, 0, span)?.into_pyobject(py).unwrap().into_any())),
            "of_bool" => {
                let b = bool_arg(args, 0, span)?;
                Ok(pyobj(pyo3::types::PyBool::new(py, b).to_owned().into_any()))
            }
            "list" => {
                let elems = pyobj_list_arg(py, args, 0, span)?;
                match PyList::new(py, elems) {
                    Ok(list) => Ok(pyobj(list.into_any())),
                    Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
                }
            }
            "to_int" => extract_scalar::<i64>(py, args, span, max, Value::Int),
            "to_float" => extract_scalar::<f64>(py, args, span, max, Value::Float),
            "to_bool" => extract_scalar::<bool>(py, args, span, max, Value::Bool),
            "to_str" => extract_scalar::<String>(py, args, span, max, Value::str),
            other => Err(Fault::at("DL0907", format!("unknown Cap[Python] method `{other}` (checker bug)"), span)),
        })
    }

    /// Dispatch a `PyObj` operation (spec §5.2): `attr`, `call`, `call_method`, `index`. A Python
    /// exception becomes a `PyErr` value; the marshalled result is a fresh `PyObj`.
    pub fn call_pyobj(obj: &PyObjVal, method: &str, args: &[Value], max: usize, span: Span) -> Result<Value, Fault> {
        Python::with_gil(|py| {
            let this = obj.0.bind(py);
            match method {
                "attr" => {
                    let name = str_arg(args, 0, span)?;
                    match this.getattr(name.as_str()) {
                        Ok(v) => Ok(Value::ok(pyobj(v))),
                        Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
                    }
                }
                "call" => {
                    let elems = pyobj_list_arg(py, args, 0, span)?;
                    let tuple = match PyTuple::new(py, elems) {
                        Ok(t) => t,
                        Err(e) => return Ok(Value::err(pyerr_to_value(py, e, max))),
                    };
                    match this.call1(&tuple) {
                        Ok(v) => Ok(Value::ok(pyobj(v))),
                        Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
                    }
                }
                "call_method" => {
                    let name = str_arg(args, 0, span)?;
                    let elems = pyobj_list_arg(py, args, 1, span)?;
                    let tuple = match PyTuple::new(py, elems) {
                        Ok(t) => t,
                        Err(e) => return Ok(Value::err(pyerr_to_value(py, e, max))),
                    };
                    match this.call_method1(name.as_str(), &tuple) {
                        Ok(v) => Ok(Value::ok(pyobj(v))),
                        Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
                    }
                }
                "index" => {
                    let key = pyobj_arg(args, 0, span)?;
                    match this.get_item(key.0.bind(py)) {
                        Ok(v) => Ok(Value::ok(pyobj(v))),
                        Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
                    }
                }
                other => Err(Fault::at("DL0907", format!("unknown PyObj method `{other}` (checker bug)"), span)),
            }
        })
    }

    /// Extract a scalar from the `PyObj` argument, wrapping success as `Ok(value)` and a failed
    /// extraction (e.g. a non-numeric object) as a `PyErr` value — never a fault.
    fn extract_scalar<'py, T>(
        py: Python<'py>,
        args: &[Value],
        span: Span,
        max: usize,
        wrap: impl Fn(T) -> Value,
    ) -> Result<Value, Fault>
    where
        T: pyo3::FromPyObject<'py>,
    {
        let obj = pyobj_arg(args, 0, span)?;
        match obj.0.bind(py).extract::<T>() {
            Ok(v) => Ok(Value::ok(wrap(v))),
            Err(e) => Ok(Value::err(pyerr_to_value(py, e, max))),
        }
    }

    /// Wrap a GIL-bound object as a `Value::PyObj`, dropping the GIL borrow (`unbind` → `Py<PyAny>`).
    fn pyobj(b: Bound<'_, PyAny>) -> Value {
        Value::PyObj(PyObjVal(b.unbind()))
    }

    /// Convert a live `PyErr` into a `std.py.PyErr { kind, message }` value. `kind` is the Python
    /// exception's type name; `message` is its `str()` — both untrusted (invariant 21), so the
    /// message is length-bounded by `py_err_value`.
    fn pyerr_to_value(py: Python<'_>, e: PyErr, max: usize) -> Value {
        let kind = e
            .get_type(py)
            .name()
            .ok()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "PyError".to_string());
        let message = e
            .value(py)
            .str()
            .ok()
            .and_then(|s| s.extract::<String>().ok())
            .unwrap_or_default();
        py_err_value(&kind, &message, max)
    }

    // ----- argument helpers (the checker guarantees the shapes) -------------

    fn str_arg(args: &[Value], i: usize, span: Span) -> Result<String, Fault> {
        match args.get(i) {
            Some(Value::Str(s)) => Ok(s.to_string()),
            _ => Err(Fault::at("DL0907", format!("expected a Str argument at position {i} (checker bug)"), span)),
        }
    }
    fn int_arg(args: &[Value], i: usize, span: Span) -> Result<i64, Fault> {
        match args.get(i) {
            Some(Value::Int(n)) => Ok(*n),
            _ => Err(Fault::at("DL0907", format!("expected an Int argument at position {i} (checker bug)"), span)),
        }
    }
    fn float_arg(args: &[Value], i: usize, span: Span) -> Result<f64, Fault> {
        match args.get(i) {
            Some(Value::Float(f)) => Ok(*f),
            _ => Err(Fault::at("DL0907", format!("expected a Float argument at position {i} (checker bug)"), span)),
        }
    }
    fn bool_arg(args: &[Value], i: usize, span: Span) -> Result<bool, Fault> {
        match args.get(i) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => Err(Fault::at("DL0907", format!("expected a Bool argument at position {i} (checker bug)"), span)),
        }
    }
    fn pyobj_arg(args: &[Value], i: usize, span: Span) -> Result<PyObjVal, Fault> {
        match args.get(i) {
            Some(Value::PyObj(o)) => Ok(o.clone()),
            _ => Err(Fault::at("DL0907", format!("expected a PyObj argument at position {i} (checker bug)"), span)),
        }
    }

    /// Bind every `PyObj` element of a `List[PyObj]` argument to the current GIL for the duration of a
    /// call. The elements are cloned `Py<PyAny>` handles; binding them is a borrow that lives only as
    /// long as the returned `Vec`, which the caller consumes into a tuple/list before the call.
    fn pyobj_list_arg<'py>(py: Python<'py>, args: &[Value], i: usize, span: Span) -> Result<Vec<Bound<'py, PyAny>>, Fault> {
        match args.get(i) {
            Some(Value::List(l)) => {
                let mut out = Vec::new();
                for v in l.borrow().iter() {
                    match v {
                        Value::PyObj(o) => out.push(o.0.bind(py).clone()),
                        _ => return Err(Fault::at("DL0907", "List[PyObj] element is not a PyObj (checker bug)", span)),
                    }
                }
                Ok(out)
            }
            _ => Err(Fault::at("DL0907", format!("expected a List[PyObj] argument at position {i} (checker bug)"), span)),
        }
    }
}

// ===========================================================================
// Feature OFF: Python-less shims. `root.python(load)` returns `Unavailable` (DL1307) before any
// `Cap[Python]` exists, so these dispatch entry points are unreachable in practice; they stay total.
// ===========================================================================

#[cfg(not(feature = "python"))]
pub fn ensure_available() -> Result<(), String> {
    Err("this build of delulu has no embedded Python (built without the `python` feature)".to_string())
}

#[cfg(not(feature = "python"))]
pub fn call_python_cap(_cap: &crate::value::CapVal, _method: &str, _args: &[Value], _max: usize, span: Span) -> Result<Value, Fault> {
    Err(Fault::at("DL1307", "embedded Python is unavailable in this build", span))
}
