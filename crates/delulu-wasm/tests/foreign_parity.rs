//! Two-engine foreign conformance parity (Stage 4 phase 4g, spec §9 criterion 6). Every foreign
//! program here produces **identical results AND identical traces** on the interpreter (the reference
//! engine) and the WASM engine — the correctness contract for the `delulu:foreign@0.4` host interface.
//!
//! Fixture strategy mirrors `delulu-runtime/tests/foreign_ffi.rs`: a tiny C-ABI cdylib is compiled
//! ONCE with `rustc --crate-type cdylib` into `CARGO_TARGET_TMPDIR` — `rustc` is guaranteed present
//! wherever `cargo test` runs (unlike a C toolchain), and an `extern "C"` cdylib is a real C-ABI
//! library on all three OSes. The same library binary drives both engines, so a value that round-trips
//! on one must round-trip identically on the other.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, TraceRecord, TraceSink, Value};
use delulu_wasm::{compile_module_with, foreign_sigs, run_main, HostConfig};

/// The C-ABI fixture — the exact surface `foreign_ffi.rs` uses, so the two engines are compared on
/// the identical library.
const FIXTURE_SRC: &str = r##"
#[no_mangle] pub extern "C" fn dl_cos(x: f64) -> f64 { x.cos() }
#[no_mangle] pub extern "C" fn dl_add(a: i64, b: i64) -> i64 { a + b }
#[no_mangle] pub extern "C" fn dl_not(b: i32) -> i32 { if b == 0 { 1 } else { 0 } }
#[no_mangle] pub extern "C" fn dl_strlen(_p: *const u8, len: usize) -> i64 { len as i64 }
#[no_mangle] pub extern "C" fn dl_first(p: *const u8, len: usize) -> i64 {
    if len == 0 { -1 } else { unsafe { *p as i64 } }
}
#[no_mangle] pub extern "C" fn dl_hello() -> *const u8 { b"hi there\0".as_ptr() }
#[no_mangle] pub extern "C" fn dl_bad_utf8() -> *const u8 { b"\xff\xfe\x00".as_ptr() }
#[no_mangle] pub extern "C" fn dl_unterminated() -> *const u8 {
    static B: [u8; 300] = [b'A'; 300];
    B.as_ptr()
}
#[no_mangle] pub extern "C" fn dl_make_ptr() -> *const u8 { static X: u8 = 42; &X as *const u8 }
#[no_mangle] pub extern "C" fn dl_deref(p: *const u8) -> i64 { unsafe { *p as i64 } }
"##;

fn fixture_path() -> &'static str {
    static P: OnceLock<String> = OnceLock::new();
    P.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        let src = dir.join("dl_fixture_wasm.rs");
        std::fs::write(&src, FIXTURE_SRC).expect("write fixture source");
        // Platform dynamic-library filename from `std::env::consts` (see foreign_ffi.rs): one path,
        // correct on Windows/.dll, Linux/.so, macOS/.dylib, loaded by full path.
        let dll = dir.join(format!("{}dl_fixture_wasm{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_SUFFIX));
        let out = std::process::Command::new("rustc")
            .args(["--edition", "2021", "--crate-type", "cdylib"])
            .arg(&src)
            .arg("-o")
            .arg(&dll)
            .output()
            .expect("rustc must be runnable (tests run under cargo)");
        assert!(out.status.success(), "fixture cdylib failed to compile: {}", String::from_utf8_lossy(&out.stderr));
        dll.to_string_lossy().to_string()
    })
}

/// The outcome of running a program on one engine: `Ok(output)` or `Err(code)` (a runtime fault code),
/// plus the recorded trace.
struct Run {
    result: Result<String, String>,
    trace: Vec<TraceRecord>,
}

/// Normalize a WASM host error message to the DL fault code it carries, so a fault compares equal to
/// the interpreter's `Fault.code` (the host surfaces the code inside a trap-string, not as a field).
fn wasm_code(msg: &str) -> String {
    for code in ["DL1306", "DL0703", "DL0903", "DL0904"] {
        if msg.contains(code) {
            return code.to_string();
        }
    }
    msg.to_string()
}

/// Run `src` on BOTH engines under identical foreign grants / `max_ret`, capturing output and trace.
/// `foreign_grants` maps logical lib name → binary path; `foreign_load` is granted iff any lib is.
fn run_both(src: &str, foreign_grants: &[(&str, &str)], max_ret: usize) -> (Run, Run) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean: {:?}", checked.diagnostics);

    let mut grant_map: HashMap<String, String> = HashMap::new();
    for (lib, path) in foreign_grants {
        grant_map.insert(lib.to_string(), path.to_string());
    }
    let foreign_load = !grant_map.is_empty();

    // ----- WASM engine -----
    let wasm = compile_module_with(&checked.module, &checked.result.foreign_binds).expect("program should compile to WASM");
    let wsink = TraceSink::new();
    let cfg = HostConfig {
        console: true,
        foreign_load,
        foreign_grants: grant_map.clone(),
        foreign_sigs: foreign_sigs(&checked.module),
        foreign_max_ret: max_ret,
        trace: Some(wsink.clone()),
        ..HostConfig::default()
    };
    let wasm_run = Run {
        result: run_main(&wasm, &cfg).map_err(|e| wasm_code(&e.message())),
        trace: wsink.records(),
    };

    // ----- interpreter (reference) -----
    let mut grants = Grants::default();
    grants.console = true;
    grants.foreign_c = grant_map;
    let isink = TraceSink::new();
    set_capture(true);
    let interp = Interp::new(&checked.module)
        .with_foreign(checked.result.foreign_binds.clone(), grants.foreign_c.clone(), max_ret)
        .with_trace(isink.clone());
    let iout = interp.run_main(Value::Root(Rc::new(grants.build_root())));
    let printed = take_capture().unwrap_or_default();
    let interp_run = Run {
        result: iout.map(|_| printed).map_err(|f| f.code.to_string()),
        trace: isink.records(),
    };

    (wasm_run, interp_run)
}

/// Assert byte-identical results AND traces across engines; return the (shared) result.
fn assert_parity(src: &str, foreign_grants: &[(&str, &str)], max_ret: usize) -> Result<String, String> {
    let (w, i) = run_both(src, foreign_grants, max_ret);
    assert_eq!(w.result, i.result, "two-engine RESULT parity broken for:\n{src}\n wasm={:?}\n interp={:?}", w.result, i.result);
    assert_eq!(w.trace, i.trace, "two-engine TRACE parity broken for:\n{src}\n wasm={:#?}\n interp={:#?}", w.trace, i.trace);
    w.result
}

const MAX: usize = delulu_runtime::foreign::DEFAULT_MAX_RET;

/// The standard bind helper used by every program.
const GET: &str = "fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n";

// ----- criterion 6: C cos happy path ---------------------------------------

#[test]
fn cos_happy_path_matches_across_engines() {
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{ fn dl_cos(x: Float) -> Float }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(str(m.dl_cos(1.0))), Err(_) => c.println(\"bind failed\") }} }}\n"
    );
    let out = assert_parity(&src, &[("fixture", fixture_path())], MAX).expect("cos runs on both engines");
    // The Constitution's adoption claim, running on the WASM engine: a real C call, exact value.
    assert_eq!(out.trim(), 1.0f64.cos().to_string());

    // Trace shape: exactly [ForeignCall dl_cos, Write println], seq 0 and 1 on both engines.
    let (w, _) = run_both(&src, &[("fixture", fixture_path())], MAX);
    assert_eq!(w.trace.len(), 2, "{:#?}", w.trace);
    assert_eq!((w.trace[0].effect.as_str(), w.trace[0].op.as_str(), w.trace[0].cap_kind.as_str()), ("ForeignCall", "dl_cos", "fixture"));
    assert_eq!(w.trace[0].detail.as_deref(), Some("fixture.dl_cos"));
    assert_eq!((w.trace[1].effect.as_str(), w.trace[1].op.as_str(), w.trace[1].cap_kind.as_str()), ("Write", "println", "Console"));
    assert_eq!(w.trace[0].seq, 0);
    assert_eq!(w.trace[1].seq, 1);
}

// ----- criterion 6: DL1303 no-grant (Err(NotGranted) as a Result value) ----

#[test]
fn ungranted_lib_is_notgranted_on_both_engines() {
    // `Cap[ForeignLoad]` exists (some OTHER lib granted) but `fixture` has no grant → Err(NotGranted).
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{ fn dl_cos(x: Float) -> Float }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(str(m.dl_cos(1.0))), Err(e) => match e {{\n\
           NotGranted => c.println(\"not granted\"),\n\
           SymbolMissing(n) => c.println(\"missing:\" + n),\n\
           Unavailable(x) => c.println(\"unavailable:\" + x),\n\
           BadReturn(x) => c.println(\"bad:\" + x) }} }} }}\n"
    );
    let out = assert_parity(&src, &[("someotherlib", "does-not-matter.dll")], MAX).expect("no-grant runs");
    assert_eq!(out.trim(), "not granted");
    // Binding is pure — no ForeignCall trace on either engine; only the Write.
    let (w, _) = run_both(&src, &[("someotherlib", "does-not-matter.dll")], MAX);
    assert!(w.trace.iter().all(|r| r.effect != "ForeignCall"), "a failed bind runs no foreign instruction: {:#?}", w.trace);
    assert_eq!(w.trace.iter().filter(|r| r.effect == "Write").count(), 1);
}

// ----- criterion 6 / 8: bad-return validation (DL1306 on both engines) ------

#[test]
fn invalid_utf8_return_is_dl1306_on_both_engines() {
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{ fn dl_bad_utf8() -> Str }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(m.dl_bad_utf8()), Err(_) => c.println(\"bind failed\") }} }}\n"
    );
    let (w, i) = run_both(&src, &[("fixture", fixture_path())], MAX);
    // Both engines FAULT with DL1306 — never a crash, never a truncated silent success (invariant 21).
    assert_eq!(w.result, Err("DL1306".into()), "wasm should fault DL1306: {:?}", w.result);
    assert_eq!(i.result, Err("DL1306".into()), "interp should fault DL1306: {:?}", i.result);
    // The ForeignCall IS traced (recorded before the call) on both — identical traces.
    assert_eq!(w.trace, i.trace);
    assert_eq!(w.trace.len(), 1);
    assert_eq!(w.trace[0].op, "dl_bad_utf8");
}

#[test]
fn oversized_return_is_dl1306_bounded_by_max_ret_on_both_engines() {
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{ fn dl_unterminated() -> Str }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(m.dl_unterminated()), Err(_) => c.println(\"bind failed\") }} }}\n"
    );
    let (w, i) = run_both(&src, &[("fixture", fixture_path())], 64);
    assert_eq!(w.result, Err("DL1306".into()));
    assert_eq!(i.result, Err("DL1306".into()));
    assert_eq!(w.trace, i.trace);
}

// ----- criterion 6: fail-fast bind (missing symbol) -------------------------

#[test]
fn missing_symbol_is_symbolmissing_at_bind_on_both_engines() {
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{ fn dl_cos(x: Float) -> Float\n fn dl_missing_xyz(x: Int) -> Int }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(str(m.dl_cos(1.0))), Err(e) => match e {{\n\
           SymbolMissing(n) => c.println(\"missing:\" + n),\n\
           NotGranted => c.println(\"not granted\"),\n\
           Unavailable(x) => c.println(\"unavailable:\" + x),\n\
           BadReturn(x) => c.println(\"bad:\" + x) }} }} }}\n"
    );
    let out = assert_parity(&src, &[("fixture", fixture_path())], MAX).expect("bind-failure path runs");
    assert_eq!(out.trim(), "missing:dl_missing_xyz");
    // Fail-fast: nothing foreign ran on either engine.
    let (w, _) = run_both(&src, &[("fixture", fixture_path())], MAX);
    assert!(w.trace.iter().all(|r| r.effect != "ForeignCall"));
}

// ----- criterion 6: the full marshalling matrix -----------------------------

#[test]
fn marshalling_matrix_matches_across_engines() {
    // Int→int64_t, Bool→int32_t 0/1, Str→(ptr,len) borrowed, ForeignPtr→opaque void* — all round-trip
    // byte-identically on both engines, and every call is traced identically.
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{\n\
           fn dl_add(a: Int, b: Int) -> Int\n\
           fn dl_not(b: Bool) -> Bool\n\
           fn dl_strlen(s: Str) -> Int\n\
           fn dl_first(s: Str) -> Int\n\
           fn dl_make_ptr() -> ForeignPtr\n\
           fn dl_deref(p: ForeignPtr) -> Int\n\
         }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => {{\n\
           c.println(str(m.dl_add(20, 22)))\n\
           c.println(str(m.dl_not(false)))\n\
           c.println(str(m.dl_strlen(\"hello\")))\n\
           c.println(str(m.dl_first(\"hello\")))\n\
           let p = m.dl_make_ptr()\n\
           c.println(str(m.dl_deref(p)))\n\
         }}, Err(_) => c.println(\"bind failed\") }} }}\n"
    );
    let out = assert_parity(&src, &[("fixture", fixture_path())], MAX).expect("marshalling runs");
    assert_eq!(out.lines().collect::<Vec<_>>(), vec!["42", "true", "5", "104", "42"]);
    // Six foreign calls, each traced identically on both engines (asserted by assert_parity's trace eq).
    let (w, _) = run_both(&src, &[("fixture", fixture_path())], MAX);
    assert_eq!(w.trace.iter().filter(|r| r.effect == "ForeignCall").count(), 6);
}

#[test]
fn str_return_happy_path_matches_across_engines() {
    let src = format!(
        "module m\n\
         foreign \"c\" lib fixture {{ fn dl_hello() -> Str }}\n\
         {GET}\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(m.dl_hello()), Err(_) => c.println(\"bind failed\") }} }}\n"
    );
    let out = assert_parity(&src, &[("fixture", fixture_path())], MAX).expect("str-return runs");
    assert_eq!(out.trim(), "hi there");
}

// ----- Python on WASM: the sanctioned DL1201 interpreter fallback ------------

#[test]
fn python_programs_are_dl1201_on_the_wasm_backend() {
    // HEAD-CHEF RULING (phase 4g): Python-on-WASM ships as the honest DL1201 fallback, not a
    // half-working port — `py.list(xs: List[PyObj])` (required by the criterion-2 NumPy demo) needs
    // guest-side List values, which the WASM fragment does not have. A program reaching `root.python`
    // is a clean `CompileError::Unsupported` (DL1201) and runs on the interpreter, where phase 4f
    // already verified it live (CPython 3.13.5).
    let src = "module m\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        let load = root.foreign_load()\n\
        match root.python(load) { Ok(py) => c.println(\"have python\"), Err(_) => c.println(\"none\") } }\n";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    match compile_module_with(&checked.module, &checked.result.foreign_binds) {
        Err(e) => assert_eq!(e.code(), "DL1201", "python must be the DL1201 fallback, got {}", e.message()),
        Ok(_) => panic!("a python-using program must not compile to WASM in phase 4g"),
    }
}
