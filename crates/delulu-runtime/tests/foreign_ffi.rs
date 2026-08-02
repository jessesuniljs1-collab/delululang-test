//! Live-DLL end-to-end tests for the Stage-4 C FFI runtime (spec §4, criteria 1/4/8).
//!
//! Fixture strategy: a tiny C-ABI shared library is compiled ONCE per test run with `rustc
//! --crate-type cdylib` into `CARGO_TARGET_TMPDIR`. `rustc` is guaranteed present wherever
//! `cargo test` runs (unlike `cl.exe`/`gcc`), and a Rust `extern "C"` cdylib is a real C-ABI
//! library on all three OSes — so these tests are portable without a C toolchain. The
//! validation layer itself (invariant 21 / criterion 8) is additionally unit-tested against
//! crafted byte buffers in `src/foreign.rs`, independent of any library.

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Fault, Grants, Interp, TraceSink, Value};

/// The C-ABI fixture source. Everything the marshalling matrix needs: scalar round-trips, the
/// two-word `(ptr, len)` Str parameter, a valid / invalid-UTF-8 / unterminated string return, and
/// an opaque-pointer round-trip.
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

/// Compile the fixture cdylib once and return its path (as a string usable in a grant).
fn fixture_path() -> &'static str {
    static P: OnceLock<String> = OnceLock::new();
    P.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        let src = dir.join("dl_fixture.rs");
        std::fs::write(&src, FIXTURE_SRC).expect("write fixture source");
        // Derive the platform's dynamic-library filename from `std::env::consts` rather than a
        // per-OS `cfg` table: `DLL_PREFIX`/`DLL_SUFFIX` are `""`/`.dll` on Windows, `lib`/`.so` on
        // Linux, `lib`/`.dylib` on macOS (Darwin). One code path, correct on every target with no
        // OS-specific branch to keep in sync — and the fixture is loaded by full path, so the
        // prefix is immaterial to the loader.
        let dll = dir.join(format!("{}dl_fixture{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_SUFFIX));
        let out = std::process::Command::new("rustc")
            .args(["--edition", "2021", "--crate-type", "cdylib"])
            .arg(&src)
            .arg("-o")
            .arg(&dll)
            .output()
            .expect("rustc must be runnable (tests run under cargo)");
        assert!(
            out.status.success(),
            "fixture cdylib failed to compile: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        dll.to_string_lossy().to_string()
    })
}

/// Check `src`, wire the Stage-4 foreign runtime data (bind sites from the checker, the given
/// logical→path grants, `max_ret`), grant a console, run `main`, and return (result, trace,
/// captured console output).
fn run_foreign(src: &str, foreign_grants: &[(&str, &str)], max_ret: usize) -> (Result<Value, Fault>, TraceSink, String) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
    let mut grants = Grants::default();
    grants.console = true;
    for (lib, path) in foreign_grants {
        grants.foreign_c.insert(lib.to_string(), path.to_string());
    }
    let sink = TraceSink::new();
    let interp = Interp::new(&checked.module)
        .with_foreign(checked.result.foreign_binds.clone(), grants.foreign_c.clone(), max_ret)
        .with_trace(sink.clone());
    set_capture(true);
    let out = interp.run_main(Value::Root(Rc::new(grants.build_root())));
    let printed = take_capture().unwrap_or_default();
    (out, sink, printed)
}

const MAX: usize = delulu_runtime::foreign::DEFAULT_MAX_RET;

// ----- criterion 1: a real C call, right value, ForeignCall in the trace ----

#[test]
fn cos_returns_the_right_value_with_foreigncall_in_the_trace() {
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_cos(x: Float) -> Float }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(str(m.dl_cos(1.0))), Err(_) => c.println(\"bind failed\") } }\n";
    let (out, sink, printed) = run_foreign(src, &[("fixture", fixture_path())], MAX);
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), 1.0f64.cos().to_string(), "cos(1.0) must round-trip exactly");
    let records = sink.records();
    let fc: Vec<_> = records.iter().filter(|r| r.effect == "ForeignCall").collect();
    assert_eq!(fc.len(), 1, "exactly one ForeignCall record: {records:?}");
    assert_eq!(fc[0].op, "dl_cos");
    assert_eq!(fc[0].cap_kind, "fixture");
}

// ----- criterion 4 / trap 4: fail-fast binding ------------------------------

#[test]
fn missing_symbol_is_symbolmissing_at_bind_never_mid_run() {
    // The block declares a symbol the library does not export. Even though the program would only
    // ever call `dl_cos`, the bind itself must fail (all symbols resolve at bind time — DL1304's
    // runtime face), and nothing foreign must have run.
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_cos(x: Float) -> Float\n fn dl_missing_xyz(x: Int) -> Int }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(str(m.dl_cos(1.0))), Err(e) => match e {\n\
            SymbolMissing(n) => c.println(\"missing:\" + n),\n\
            NotGranted => c.println(\"not granted\"),\n\
            Unavailable(x) => c.println(\"unavailable:\" + x),\n\
            BadReturn(x) => c.println(\"bad:\" + x) } } }\n";
    let (out, sink, printed) = run_foreign(src, &[("fixture", fixture_path())], MAX);
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), "missing:dl_missing_xyz");
    assert!(
        sink.records().iter().all(|r| r.effect != "ForeignCall"),
        "fail-fast: no foreign instruction may run after a failed bind"
    );
}

#[test]
fn ungranted_lib_is_notgranted_as_a_result_value() {
    // `Cap[ForeignLoad]` exists (some OTHER lib is granted), but `fixture` itself has no grant:
    // the bind yields `Err(NotGranted)` — DL1303's runtime face, defense-in-depth behind the CLI
    // startup refusal.
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_cos(x: Float) -> Float }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(str(m.dl_cos(1.0))), Err(e) => match e {\n\
            NotGranted => c.println(\"not granted\"),\n\
            SymbolMissing(n) => c.println(\"missing:\" + n),\n\
            Unavailable(x) => c.println(\"unavailable:\" + x),\n\
            BadReturn(x) => c.println(\"bad:\" + x) } } }\n";
    let (out, _, printed) = run_foreign(src, &[("someotherlib", "does-not-matter.dll")], MAX);
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), "not granted");
}

#[test]
fn no_foreign_grant_at_all_means_no_cap_foreignload_dl0703() {
    // With zero foreign grants the root slice has no loader authority: `root.foreign_load()`
    // refuses with DL0703, exactly like every other ungranted root derivation.
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_cos(x: Float) -> Float }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(str(m.dl_cos(1.0))), Err(_) => c.println(\"e\") } }\n";
    let (out, _, _) = run_foreign(src, &[], MAX);
    assert_eq!(out.err().map(|f| f.code), Some("DL0703"));
}

#[test]
fn unloadable_binary_is_unavailable_not_a_crash() {
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_cos(x: Float) -> Float }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(str(m.dl_cos(1.0))), Err(e) => match e {\n\
            Unavailable(_) => c.println(\"unavailable\"),\n\
            NotGranted => c.println(\"not granted\"),\n\
            SymbolMissing(n) => c.println(\"missing:\" + n),\n\
            BadReturn(x) => c.println(\"bad:\" + x) } } }\n";
    let (out, _, printed) = run_foreign(src, &[("fixture", "no_such_library_xyz_delulu.dll")], MAX);
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), "unavailable");
}

// ----- marshalling round-trips (spec §4.2) ----------------------------------

#[test]
fn marshalling_round_trips_int_bool_str_and_foreignptr() {
    let src = "module m\n\
        foreign \"c\" lib fixture {\n\
            fn dl_add(a: Int, b: Int) -> Int\n\
            fn dl_not(b: Bool) -> Bool\n\
            fn dl_strlen(s: Str) -> Int\n\
            fn dl_first(s: Str) -> Int\n\
            fn dl_make_ptr() -> ForeignPtr\n\
            fn dl_deref(p: ForeignPtr) -> Int\n\
        }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => {\n\
            c.println(str(m.dl_add(20, 22)))\n\
            c.println(str(m.dl_not(false)))\n\
            c.println(str(m.dl_strlen(\"hello\")))\n\
            c.println(str(m.dl_first(\"hello\")))\n\
            let p = m.dl_make_ptr()\n\
            c.println(str(m.dl_deref(p)))\n\
        }, Err(_) => c.println(\"bind failed\") } }\n";
    let (out, sink, printed) = run_foreign(src, &[("fixture", fixture_path())], MAX);
    assert!(out.is_ok(), "{:?}", out.err());
    let lines: Vec<&str> = printed.lines().collect();
    // Int→int64_t and back; Bool→int32_t 0/1 and back; Str→(ptr,len) borrowed — the callee sees the
    // exact length AND readable bytes ('h' = 104); ForeignPtr→opaque void* round-trip (*ptr = 42).
    assert_eq!(lines, vec!["42", "true", "5", "104", "42"], "printed: {printed:?}");
    assert_eq!(
        sink.records().iter().filter(|r| r.effect == "ForeignCall").count(),
        6,
        "every foreign call is traced"
    );
}

#[test]
fn str_return_is_copied_then_validated_happy_path() {
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_hello() -> Str }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(m.dl_hello()), Err(_) => c.println(\"bind failed\") } }\n";
    let (out, _, printed) = run_foreign(src, &[("fixture", fixture_path())], MAX);
    assert!(out.is_ok(), "{:?}", out.err());
    assert_eq!(printed.trim(), "hi there");
}

// ----- criterion 8: hostile returns are defined failures, never a crash -----

#[test]
fn invalid_utf8_return_is_a_dl1306_fault_not_a_crash() {
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_bad_utf8() -> Str }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(m.dl_bad_utf8()), Err(_) => c.println(\"bind failed\") } }\n";
    let (out, _, _) = run_foreign(src, &[("fixture", fixture_path())], MAX);
    let fault = out.expect_err("invalid UTF-8 must be a defined fault");
    assert_eq!(fault.code, "DL1306");
    assert!(fault.message.contains("BadReturn"), "{}", fault.message);
}

#[test]
fn oversized_return_is_a_dl1306_fault_bounded_by_max_ret() {
    // The fixture returns 300 non-NUL bytes; with a 64-byte ceiling the scan must stop and refuse —
    // never read past the bound, never truncate into a silent success.
    let src = "module m\n\
        foreign \"c\" lib fixture { fn dl_unterminated() -> Str }\n\
        fn get(root: Root) -> Result[fixture, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
        fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
        match get(root) { Ok(m) => c.println(m.dl_unterminated()), Err(_) => c.println(\"bind failed\") } }\n";
    let (out, _, printed) = run_foreign(src, &[("fixture", fixture_path())], 64);
    let fault = out.expect_err("an unterminated oversized return must be a defined fault");
    assert_eq!(fault.code, "DL1306");
    assert!(fault.message.contains("--foreign-max-ret"), "{}", fault.message);
    assert!(!printed.contains("AAA"), "no truncated partial data may leak: {printed:?}");
}
