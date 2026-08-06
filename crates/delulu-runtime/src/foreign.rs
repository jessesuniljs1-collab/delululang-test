//! Stage 4 C FFI runtime (spec §4). **Every native-dependency line in the interpreter lives here**
//! — `libloading` for binding a shared library and `libffi` for calling it (signatures are runtime
//! data, so the call is assembled dynamically). Nothing outside this module touches a raw pointer,
//! a `Library`, or a `Cif`.
//!
//! Honesty (spec §10, carried verbatim into the DL13xx explain texts): DeluluLang bounds foreign
//! *reachability* — a grant, a `Cap` threaded from `Root`, and `ForeignCall` in every row — not
//! foreign *behavior*. Once bound, a C library runs with **full process authority**; it can corrupt
//! or crash the process, and memory safety across the FFI is not claimed. Containment of behavior is
//! process-level at Stage 5 and microVM-level after. The word "sandbox" does not apply here.
//!
//! What this module *does* guarantee (invariant 21): every value returned from foreign code is
//! validated for *shape* at the boundary — a returned string is length-bounded and UTF-8-checked —
//! and a failure is a defined [`ForeignErr`], never UB, never a panic, never a silent truncation.

use std::collections::HashMap;
use std::os::raw::c_void;
use std::rc::Rc;

use libffi::middle::{arg, Arg, Cif, CodePtr, Type as FfiType};
use libloading::Library;

/// Default ceiling on a returned foreign string (invariant 21). Overridable with `--foreign-max-ret`.
pub const DEFAULT_MAX_RET: usize = 64 * 1024 * 1024;

/// A marshallable scalar/string kind — exactly the T-ForeignSig allowlist (spec §4.2). A well-typed
/// program can only ever present these at the boundary; the checker's DL1301/DL1302 fence guarantees
/// it, so [`FKind::from_type_name`] returning `None` means the interpreter was handed an unchecked
/// program (a bug), not a user error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FKind {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    Ptr,
}

impl FKind {
    pub fn from_type_name(name: &str) -> Option<FKind> {
        Some(match name {
            "Int" => FKind::Int,
            "Float" => FKind::Float,
            "Bool" => FKind::Bool,
            "Str" => FKind::Str,
            "Unit" => FKind::Unit,
            "ForeignPtr" => FKind::Ptr,
            _ => return None,
        })
    }
}

/// One foreign function's marshalling signature, lowered from the `foreign` block.
#[derive(Clone, Debug)]
pub struct ForeignSig {
    pub name: String,
    pub params: Vec<FKind>,
    pub ret: FKind,
}

/// A runtime value crossing the FFI boundary in either direction. Deliberately a small, owned enum
/// so the interpreter's `Value` never leaks a raw pointer or a `Library` into general evaluation.
#[derive(Clone, Debug)]
pub enum FVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<str>),
    Unit,
    Ptr(usize),
}

/// A defined foreign failure (spec §8 `ForeignErr = NotGranted | SymbolMissing(Str) | BadReturn(Str)
/// | Unavailable(Str)`). Never a panic, never UB.
///
/// `WorkerDied` is a Stage-5 phase-5h ADDITION and is **Rust-internal only** — it is NOT a new
/// variant of the *language* `std.foreign.ForeignErr` sum (language surface delta stays zero, trap 1).
/// It is produced only by the process-isolation worker path when the isolated worker subprocess dies
/// mid-call (a C segfault, a hard crash): the host reads EOF/broken-pipe on the private channel and
/// reports this instead of dying alongside the worker. The interpreter surfaces it as a clean DL1409
/// fault; the host process continues (spec §5, criterion 7 — the headline guarantee).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForeignErr {
    NotGranted,
    SymbolMissing(String),
    BadReturn(String),
    Unavailable(String),
    WorkerDied(String),
}

/// A bound foreign library: the loaded handle (kept alive for the program), the resolved code
/// address of every declared symbol, and each function's marshalling signature. The raw addresses in
/// `syms` are valid exactly as long as `_lib` — which is why they live in the same struct.
pub struct LoadedLib {
    _lib: Library,
    syms: HashMap<String, *mut c_void>,
    sigs: HashMap<String, ForeignSig>,
}

impl LoadedLib {
    pub fn sig(&self, method: &str) -> Option<&ForeignSig> {
        self.sigs.get(method)
    }
}

/// Bind the library at `path`, resolving **every** declared symbol up front (spec §4.2, trap 4:
/// fail-fast at bind, never mid-run). A load failure is [`ForeignErr::Unavailable`]; a symbol that
/// the block declares but the library does not export is [`ForeignErr::SymbolMissing`] (DL1304) — a
/// program that binds successfully will never surprise the caller with a missing symbol during a
/// later call.
pub fn load_and_resolve(path: &str, sigs: Vec<ForeignSig>) -> Result<LoadedLib, ForeignErr> {
    // SAFETY: loading an arbitrary shared library runs its initializers with full process authority
    // — Stage 4's acknowledged threat model (spec §10; behavioral containment is Stage 5). libloading
    // upholds the dlopen/LoadLibrary contract and reports a load failure as an `Err`, never UB.
    let lib = unsafe { Library::new(path) }
        .map_err(|e| ForeignErr::Unavailable(format!("cannot load `{path}`: {e}")))?;

    let mut syms = HashMap::new();
    let mut sigmap = HashMap::new();
    for sig in sigs {
        let mut cname = sig.name.clone().into_bytes();
        cname.push(0); // NUL-terminate for the loader lookup
        // SAFETY: `cname` is NUL-terminated. `Library::get` only reads the export table; a missing
        // symbol is a returned `Err` (→ DL1304), never UB. We capture the code address immediately
        // and drop the borrow — the address stays valid for as long as `lib` (moved into the struct
        // below) is alive.
        let addr = unsafe {
            match lib.get::<unsafe extern "C" fn()>(&cname) {
                Ok(sym) => (*sym) as usize as *mut c_void,
                Err(_) => return Err(ForeignErr::SymbolMissing(sig.name.clone())),
            }
        };
        syms.insert(sig.name.clone(), addr);
        sigmap.insert(sig.name.clone(), sig);
    }
    Ok(LoadedLib { _lib: lib, syms, sigs: sigmap })
}

/// Call `method` on a bound library. Marshals arguments per spec §4.2 (`Int`→`int64_t`,
/// `Float`→`double`, `Bool`→`int32_t`, `Str`→borrowed `(const uint8_t*, size_t)`, `ForeignPtr`→
/// `void*`) and validates the return per invariant 21. `method` and the argument shapes are trusted
/// to match the block signature — the checker guarantees this — so a mismatch is a defect, not a
/// user-facing error.
pub fn call(loaded: &LoadedLib, method: &str, args: &[FVal], max_ret: usize) -> Result<FVal, ForeignErr> {
    let sig = loaded.sigs.get(method).expect("checked foreign method exists");
    let code = *loaded.syms.get(method).expect("symbol resolved at bind time");

    // Build the ffi parameter-type list and a stable backing store for the native argument bytes. A
    // `Str` marshals to TWO native arguments — (pointer, length). The store is built in full before
    // any reference into it is taken, so it never reallocates mid-borrow.
    let mut ffi_params: Vec<FfiType> = Vec::new();
    let mut cells: Vec<Cell> = Vec::new();
    let mut str_backing: Vec<Rc<str>> = Vec::new();
    for (kind, val) in sig.params.iter().zip(args) {
        match kind {
            FKind::Int => {
                ffi_params.push(FfiType::i64());
                cells.push(Cell::I64(as_i64(val)));
            }
            FKind::Float => {
                ffi_params.push(FfiType::f64());
                cells.push(Cell::F64(as_f64(val)));
            }
            FKind::Bool => {
                ffi_params.push(FfiType::i32());
                cells.push(Cell::I32(if as_bool(val) { 1 } else { 0 }));
            }
            FKind::Ptr => {
                ffi_params.push(FfiType::pointer());
                cells.push(Cell::Ptr(as_ptr(val)));
            }
            FKind::Str => {
                let s = as_str(val);
                str_backing.push(s.clone());
                let bytes = s.as_bytes();
                ffi_params.push(FfiType::pointer());
                ffi_params.push(FfiType::usize());
                cells.push(Cell::Ptr(bytes.as_ptr() as *const c_void));
                cells.push(Cell::USize(bytes.len()));
            }
            FKind::Unit => { /* `Unit` marshals to no native argument */ }
        }
    }
    let ffi_args: Vec<Arg> = cells.iter().map(Cell::as_arg).collect();

    let ret_ty = match sig.ret {
        FKind::Int => FfiType::i64(),
        FKind::Float => FfiType::f64(),
        FKind::Bool => FfiType::i32(),
        FKind::Str | FKind::Ptr => FfiType::pointer(),
        FKind::Unit => FfiType::void(),
    };
    let cif = Cif::new(ffi_params, ret_ty);
    let code_ptr = CodePtr(code);

    // SAFETY: the CIF exactly describes the marshalling agreed in the block signature, and `code` is
    // a symbol address resolved at bind time. `cells` and `str_backing` keep every borrowed argument
    // pointer valid for the whole call. What the callee does with full process authority (including a
    // crash) is out of scope — spec §4.2/§10: invariant 21 bounds returned *data*, not the callee's
    // memory safety.
    let result = unsafe {
        match sig.ret {
            FKind::Unit => {
                cif.call::<()>(code_ptr, &ffi_args);
                FVal::Unit
            }
            FKind::Int => FVal::Int(cif.call::<i64>(code_ptr, &ffi_args)),
            FKind::Float => FVal::Float(cif.call::<f64>(code_ptr, &ffi_args)),
            FKind::Bool => FVal::Bool(cif.call::<i32>(code_ptr, &ffi_args) != 0),
            FKind::Ptr => FVal::Ptr(cif.call::<*mut c_void>(code_ptr, &ffi_args) as usize),
            FKind::Str => {
                let p: *const u8 = cif.call::<*const u8>(code_ptr, &ffi_args);
                FVal::Str(Rc::from(validate_c_string(p, max_ret)?.as_str()))
            }
        }
    };
    drop(str_backing); // make the argument-lifetime contract explicit
    Ok(result)
}

/// Read and validate a returned C string (invariant 21): scan up to `max_ret` bytes for a NUL, copy
/// those bytes, and require valid UTF-8. Every failure is [`ForeignErr::BadReturn`] — never UB, never
/// a panic, never a silent truncation.
///
/// # Safety
/// `ptr` must be null, or point to readable bytes up to the first NUL or `max_ret`, whichever is
/// first. A hostile library that returns a pointer into unmapped memory can still fault the process;
/// that is acknowledged out of scope (spec §4.2 — invariant 21 bounds *data*, not the callee's memory
/// safety).
pub unsafe fn validate_c_string(ptr: *const u8, max_ret: usize) -> Result<String, ForeignErr> {
    if ptr.is_null() {
        return Err(ForeignErr::BadReturn("foreign returned a null string pointer".into()));
    }
    let mut len = 0usize;
    while len < max_ret {
        // SAFETY: within the [0, max_ret) window the caller guarantees readability up to the NUL.
        if unsafe { *ptr.add(len) } == 0 {
            break;
        }
        len += 1;
    }
    if len >= max_ret {
        return Err(ForeignErr::BadReturn(format!(
            "foreign string return exceeds --foreign-max-ret ({max_ret} bytes) with no terminator"
        )));
    }
    // SAFETY: the `len` bytes at `ptr` were just scanned as readable and outlive this copy.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    String::from_utf8(bytes)
        .map_err(|_| ForeignErr::BadReturn("foreign string return is not valid UTF-8".into()))
}

/// A stable owned home for one native scalar argument, so libffi's `arg(&_)` borrows something that
/// outlives the call. `Ptr` also serves the two-word `Str` marshalling (pointer half).
enum Cell {
    I64(i64),
    F64(f64),
    I32(i32),
    Ptr(*const c_void),
    USize(usize),
}

impl Cell {
    fn as_arg(&self) -> Arg {
        match self {
            Cell::I64(v) => arg(v),
            Cell::F64(v) => arg(v),
            Cell::I32(v) => arg(v),
            Cell::Ptr(v) => arg(v),
            Cell::USize(v) => arg(v),
        }
    }
}

fn as_i64(v: &FVal) -> i64 {
    match v {
        FVal::Int(i) => *i,
        _ => 0,
    }
}
fn as_f64(v: &FVal) -> f64 {
    match v {
        FVal::Float(f) => *f,
        _ => 0.0,
    }
}
fn as_bool(v: &FVal) -> bool {
    matches!(v, FVal::Bool(true))
}
fn as_ptr(v: &FVal) -> *const c_void {
    match v {
        FVal::Ptr(p) => *p as *const c_void,
        _ => std::ptr::null(),
    }
}
fn as_str(v: &FVal) -> Rc<str> {
    match v {
        FVal::Str(s) => s.clone(),
        _ => Rc::from(""),
    }
}

/// A bound foreign library plus the logical name that named it (for the trace and diagnostics). This
/// is what the interpreter stores in a `Value::Foreign` handle.
///
/// The library is held behind the [`BoundForeign`] seam (Stage 5 phase 5h) so the interpreter and
/// WASM host do not care whether it executes **in-process** (Stage 4, [`LoadedLib`]) or in an
/// **isolated worker subprocess** (the `delulu` CLI's worker, over a private pipe). Both speak the
/// exact same marshalling ([`FVal`]).
pub struct ForeignHandle {
    pub name: String,
    pub exec: Box<dyn BoundForeign>,
}

/// A bound foreign library that can execute marshalled calls (Stage 5 phase 5h seam). Two impls:
/// [`LoadedLib`] runs the call **in this process** (Stage 4 behaviour); the `delulu` CLI's
/// `WorkerConn` forwards it to an **isolated worker subprocess** over a private pipe. Object-safe and
/// single-threaded (the interpreter is `!Send`), so `&self` methods with interior mutability suffice.
pub trait BoundForeign {
    /// The marshalling signature of `method`, if the library resolved it at bind time. Owned (a
    /// cheap clone) so the worker impl need not lend a reference into another process's state.
    fn sig(&self, method: &str) -> Option<ForeignSig>;

    /// Execute `method` with the marshalled `args`, validating the return per invariant 21. A dead
    /// isolated worker is [`ForeignErr::WorkerDied`] (the host survives — spec §5); an in-process lib
    /// never returns that variant.
    fn call(&self, method: &str, args: &[FVal], max_ret: usize) -> Result<FVal, ForeignErr>;
}

impl BoundForeign for LoadedLib {
    fn sig(&self, method: &str) -> Option<ForeignSig> {
        self.sigs.get(method).cloned()
    }

    fn call(&self, method: &str, args: &[FVal], max_ret: usize) -> Result<FVal, ForeignErr> {
        call(self, method, args, max_ret)
    }
}

/// Binds a granted foreign library to a [`BoundForeign`] executor (Stage 5 phase 5h seam). The
/// default [`InProcBinder`] loads it in-process (Stage 4, `--foreign-isolation inproc`); the `delulu`
/// CLI injects a worker-spawning binder for `--foreign-isolation process`. The interpreter holds a
/// binder and calls it from `bind_foreign`, so the isolation choice is a single injected object and
/// nothing in the evaluator branches on it.
pub trait ForeignBinder {
    /// Bind the granted binary at `path` (for logical `lib_name`) with the given signatures. A
    /// load/spawn failure is a [`ForeignErr`] the caller turns into a catchable bind result
    /// (`NotGranted` is handled by the caller before this is reached).
    fn bind(
        &self,
        lib_name: &str,
        path: &str,
        sigs: Vec<ForeignSig>,
        max_ret: usize,
    ) -> Result<Box<dyn BoundForeign>, ForeignErr>;
}

/// The default, dev binder: load + resolve the library **in this process** exactly as Stage 4 did
/// (`--foreign-isolation inproc`). Zero behaviour change for a program that does not opt into worker
/// isolation (criterion 11).
pub struct InProcBinder;

impl ForeignBinder for InProcBinder {
    fn bind(
        &self,
        _lib_name: &str,
        path: &str,
        sigs: Vec<ForeignSig>,
        _max_ret: usize,
    ) -> Result<Box<dyn BoundForeign>, ForeignErr> {
        load_and_resolve(path, sigs).map(|lib| Box::new(lib) as Box<dyn BoundForeign>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The validation layer (invariant 21 / criterion 8) is unit-tested directly against crafted byte
    // buffers, independent of any library — the buffers are the adversary. A live-DLL end-to-end test
    // lives in `tests/foreign_ffi.rs`.

    // These run under Miri as well as natively, and that is the point of them.
    //
    // `validate_c_string` is the highest-risk function in the tree — a hand-rolled NUL scan over
    // caller-supplied memory, with `ptr.add` and `from_raw_parts` — and for a long time nothing
    // interpreted it, because `delulu-runtime` was not in the Miri matrix while four crates
    // containing no `unsafe` at all were. A checker aimed at code that cannot exhibit the defect
    // reports clean forever.
    //
    // The detector was confirmed live rather than assumed: a temporary probe passing a 4-byte
    // buffer with no terminator and a bound of 64 was rejected with
    // *"attempting to access 1 byte, but got alloc+0x4 which is at or beyond the end of the
    // allocation of size 4 bytes"*. Under this project's flags — `-Zmiri-disable-isolation`, and
    // Stacked Borrows deliberately left ON — Miri still sees an out-of-bounds read here.
    #[test]
    fn valid_utf8_c_string_is_read_up_to_the_nul() {
        let buf = b"hi there\0trailing garbage".as_ptr();
        // SAFETY: `buf` is a readable, NUL-terminated byte literal.
        let s = unsafe { validate_c_string(buf, DEFAULT_MAX_RET) };
        assert_eq!(s.unwrap(), "hi there");
    }

    #[test]
    fn invalid_utf8_return_is_bad_return_not_a_crash() {
        // 0xFF is never valid UTF-8; the buffer is NUL-terminated so the scan is bounded.
        let buf = [0xffu8, 0xfe, 0x00];
        // SAFETY: `buf` is a readable, NUL-terminated stack buffer.
        let r = unsafe { validate_c_string(buf.as_ptr(), DEFAULT_MAX_RET) };
        assert!(matches!(r, Err(ForeignErr::BadReturn(_))), "{r:?}");
    }

    #[test]
    fn oversized_return_is_bad_return_bounded_by_max_ret() {
        // 300 non-NUL bytes; with a small bound the scan stops without a terminator → BadReturn.
        let buf = [b'A'; 300];
        // SAFETY: the scan reads at most `max_ret` (64) of the 300 readable bytes — never out of bounds.
        let r = unsafe { validate_c_string(buf.as_ptr(), 64) };
        match r {
            Err(ForeignErr::BadReturn(msg)) => assert!(msg.contains("--foreign-max-ret"), "{msg}"),
            other => panic!("expected BadReturn, got {other:?}"),
        }
    }

    #[test]
    fn null_return_is_bad_return_not_ub() {
        // SAFETY: a null pointer is handled before any dereference.
        let r = unsafe { validate_c_string(std::ptr::null(), DEFAULT_MAX_RET) };
        assert!(matches!(r, Err(ForeignErr::BadReturn(_))), "{r:?}");
    }

    #[test]
    fn empty_c_string_is_ok() {
        let buf = b"\0".as_ptr();
        // SAFETY: a one-byte NUL-terminated buffer.
        let s = unsafe { validate_c_string(buf, DEFAULT_MAX_RET) };
        assert_eq!(s.unwrap(), "");
    }

    #[test]
    fn fkind_maps_exactly_the_marshallable_allowlist() {
        assert_eq!(FKind::from_type_name("Int"), Some(FKind::Int));
        assert_eq!(FKind::from_type_name("Float"), Some(FKind::Float));
        assert_eq!(FKind::from_type_name("Bool"), Some(FKind::Bool));
        assert_eq!(FKind::from_type_name("Str"), Some(FKind::Str));
        assert_eq!(FKind::from_type_name("Unit"), Some(FKind::Unit));
        assert_eq!(FKind::from_type_name("ForeignPtr"), Some(FKind::Ptr));
        // Non-marshallable types are fenced by the checker (DL1301) and never reach here.
        assert_eq!(FKind::from_type_name("PyObj"), None);
        assert_eq!(FKind::from_type_name("List"), None);
    }
}
