//! Live embedded-CPython end-to-end tests for the Stage-4 `std.py` runtime (spec §5, criteria 2/5).
//!
//! These need a real interpreter (and, for the NumPy demo, NumPy). They are gated on the `python`
//! feature and additionally **detect availability at runtime**, skipping-with-a-printed-notice when
//! the interpreter or NumPy is absent (CI portability). On a host that has both — as the head chef's
//! machine does — they RUN FOR REAL: criterion 2 is "the demo runs", not "the demo would run".
#![cfg(feature = "python")]

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Fault, Grants, Interp, TraceSink, Value};

/// Check `src`, wire the console + the caller's grants, run `main` under a fresh trace sink, and
/// return (result, trace, captured console output).
fn run_py(src: &str, setup: impl FnOnce(&mut Grants)) -> (Result<Value, Fault>, TraceSink, String) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
    let mut grants = Grants { console: true, ..Default::default() };
    setup(&mut grants);
    let sink = TraceSink::new();
    let interp = Interp::new(&checked.module)
        .with_foreign(
            checked.result.foreign_binds.clone(),
            grants.foreign_c.clone(),
            delulu_runtime::foreign::DEFAULT_MAX_RET,
        )
        .with_trace(sink.clone());
    set_capture(true);
    let out = interp.run_main(Value::Root(Rc::new(grants.build_root())));
    let printed = take_capture().unwrap_or_default();
    (out, sink, printed)
}

/// Whether the embedded interpreter starts on this host (feature on AND `prepare` succeeds).
fn interpreter_available() -> bool {
    delulu_runtime::python::ensure_available().is_ok()
}

fn skip(reason: &str) {
    eprintln!("SKIP (portability): {reason}");
}

// ----- criterion 2: the NumPy demo, running -------------------------------

const NUMPY_DEMO: &str = "module m\n\
    fn mean_of(py: Cap[Python]) -> Result[Float, PyErr] ! {ForeignCall} {\n\
      let np = py.import(\"numpy\")?\n\
      let xs = py.list([py.of_float(1.0), py.of_float(2.0), py.of_float(3.0), py.of_float(4.0)])\n\
      let m = np.call_method(\"mean\", [xs])?\n\
      py.to_float(m) }\n\
    fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
      let load = root.foreign_load()\n\
      match root.python(load) { Ok(py) => match mean_of(py) {\n\
        Ok(v) => c.println(str(v)),\n\
        Err(e) => c.println(\"err:\" + e.message) }, Err(_) => c.println(\"unavailable\") } }\n";

#[test]
fn numpy_mean_demo_runs_end_to_end_with_foreigncall_traced() {
    if !interpreter_available() {
        skip("no embedded CPython on this host");
        return;
    }
    let (out, sink, printed) = run_py(NUMPY_DEMO, |g| g.foreign_python = vec!["numpy".into(), "numpy.*".into()]);
    assert!(out.is_ok(), "{:?}", out.err());
    let printed = printed.trim();
    if printed.starts_with("err:") && (printed.contains("numpy") || printed.contains("No module")) {
        skip("NumPy is not installed on this host");
        return;
    }
    // The Constitution's "inherit the AI ecosystem" claim, running: mean([1,2,3,4]) = 2.5.
    assert_eq!(printed, "2.5", "the NumPy demo must compute the mean; got {printed:?}");
    // Every std.py op is `{ForeignCall}`: import, list, of_float×4, call_method, to_float are all
    // traced under the ForeignCall effect.
    let fc = sink.records().into_iter().filter(|r| r.effect == "ForeignCall").count();
    assert!(fc >= 4, "the std.py ops must be traced as ForeignCall (saw {fc})");
    assert!(
        sink.records().iter().any(|r| r.op == "py.import" && r.detail.as_deref() == Some("numpy")),
        "the numpy import must be a py.import ForeignCall record"
    );
}

// ----- criterion 5: an off-allowlist import is DL1305, logged in the trace -

const IMPORT_OS: &str = "module m\n\
    fn attempt(py: Cap[Python]) -> Str ! {ForeignCall} {\n\
      match py.import(\"os\") { Ok(_) => \"imported\", Err(e) => \"denied:\" + e.kind } }\n\
    fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
      let load = root.foreign_load()\n\
      match root.python(load) { Ok(py) => c.println(attempt(py)), Err(_) => c.println(\"unavailable\") } }\n";

#[test]
fn import_off_allowlist_is_dl1305_pyerr_and_the_denied_attempt_is_traced() {
    if !interpreter_available() {
        skip("no embedded CPython on this host");
        return;
    }
    // Allowlist is ["numpy"]; the program tries `import os` → refused BY NAME, as a runtime PyErr.
    let (out, sink, printed) = run_py(IMPORT_OS, |g| g.foreign_python = vec!["numpy".into()]);
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), "denied:ImportNotAllowed", "off-allowlist import must be a PyErr");
    // Criterion 5: the DENIED attempt is visible in the trace as a ForeignCall on `py.import` naming
    // the refused module — the interface gate is observable, not silent.
    let denied = sink
        .records()
        .into_iter()
        .find(|r| r.op == "py.import")
        .expect("the denied import attempt must be traced");
    assert_eq!(denied.effect, "ForeignCall");
    assert_eq!(denied.detail.as_deref(), Some("os"));
}

// ----- ungranted Python is NotGranted (DL1303's runtime face) --------------

const NEEDS_PY: &str = "module m\n\
    fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
      let load = root.foreign_load()\n\
      match root.python(load) { Ok(_) => c.println(\"granted\"), Err(e) => match e {\n\
        NotGranted => c.println(\"not granted\"),\n\
        Unavailable(_) => c.println(\"unavailable\"),\n\
        SymbolMissing(_) => c.println(\"sym\"),\n\
        BadReturn(_) => c.println(\"bad\") } } }\n";

#[test]
fn ungranted_python_is_notgranted_as_a_result_value() {
    // `Cap[ForeignLoad]` exists (a C lib is granted so `root.foreign_load()` succeeds), but no
    // `foreign.python` pattern is granted: `root.python` yields `Err(NotGranted)` — DL1303's runtime
    // face, defense-in-depth behind the CLI startup refusal. No interpreter is needed for this path.
    let (out, _, printed) = run_py(NEEDS_PY, |g| {
        g.foreign_c.insert("somelib".into(), "does-not-matter.dll".into());
    });
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), "not granted");
}
