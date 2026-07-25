//! Fault parity between the two engines (HARDENING_CAMPAIGN C20; Stage 3 invariant 15).
//!
//! Invariant 15 promises the interpreter and the WASM backend agree — same output, same exit,
//! same fault. For SUCCESS that is covered by the differential fuzz. For FAULTS it was not: the
//! differential harness classifies any `(Err, Err)` as agreement without checking the faults
//! match, and they did not. A program that divides by zero was `DL0902` on the interpreter and a
//! generic `DL0904` on the WASM engine, and a deep recursion printed one clean `DL0905` on the
//! interpreter but ~16,000 lines of guest backtrace (`<wasm function N>`, one per frame) on WASM.
//!
//! These tests pin the fixed behaviour: the deterministic arithmetic/recursion faults report the
//! SAME code on both engines, and a WASM trap never carries a backtrace. Both fail against the
//! pre-fix code.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{Grants, Interp, Value};
use delulu_wasm::{compile_module, run_main, HostConfig};

/// The DL code the interpreter faults with (or "OK" if it does not fault).
fn interp_fault_code(src: &str) -> String {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
    let g = Grants { console: true, ..Grants::default() };
    let interp = Interp::new(&checked.module);
    match interp.run_main(Value::Root(Rc::new(g.build_root()))) {
        Ok(_) => "OK".to_string(),
        Err(f) => f.code.to_string(),
    }
}

/// The WASM engine's fault detail (the string the CLI turns into a diagnostic). It embeds the DL
/// code and, after the fix, carries no backtrace.
fn wasm_fault_detail(src: &str) -> String {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).expect("compile to wasm");
    match run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }) {
        Ok(_) => "OK".to_string(),
        Err(e) => e.message(),
    }
}

const DIVZERO: &str =
    "module m\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(7 / 0)) }\n";
const OVERFLOW: &str =
    "module m\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(9223372036854775807 + 1)) }\n";
const RECUR: &str =
    "module m\nfn cd(n: Int) -> Int { if n <= 0 { 0 } else { 1 + cd(n - 1) } }\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(cd(1000000))) }\n";

/// Run `body` on a thread with a main-thread-sized (16 MiB) stack.
///
/// The interpreter's recursion guard (`MAX_DEPTH = 10_000`) is calibrated for the CLI's ~8 MiB
/// main-thread stack, where a deep recursion reaches the guard and reports DL0905 cleanly. The Rust
/// test harness runs each test on a ~2 MiB thread, on which 10 000 interpreter frames overflow the
/// host stack *before* the guard fires — a real, separate finding (HARDENING_CAMPAIGN C21). To test
/// engine *parity* rather than that stack-size fragility, these run where the CLI runs: a big stack.
fn on_a_big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(body)
        .expect("spawn")
        .join()
        .expect("the parity body must not panic");
}

#[test]
fn the_two_engines_agree_on_arithmetic_fault_codes() {
    // Divide-by-zero and overflow fault without deep recursion, so both engines run directly here.
    // (Recursion is exercised WASM-side below; the interpreter's DL0905 has its own Stage-9 test
    // and would need a large stack to reach — see C21 and `on_a_big_stack`.)
    for (src, code) in [(DIVZERO, "DL0902"), (OVERFLOW, "DL0901")] {
        assert_eq!(interp_fault_code(src), code, "the interpreter must fault with {code}");
        let wasm = wasm_fault_detail(src);
        assert!(
            wasm.contains(code),
            "the WASM engine must report {code} for the same fault (parity, invariant 15), got: {wasm}"
        );
    }
}

#[test]
fn a_wasm_stack_overflow_is_dl0905_with_no_backtrace() {
    on_a_big_stack(|| {
        // This is the headline C20 fix. A guest stack overflow used to surface as a generic DL0904
        // followed by ~16,000 lines of `<wasm function N>` — one per frame. It must now be the same
        // DL0905 the interpreter reports, on a single line, with no backtrace.
        let detail = wasm_fault_detail(RECUR);
        assert!(detail.contains("DL0905"), "a guest stack overflow must be DL0905, got: {detail}");
        assert_eq!(detail.lines().count(), 1, "the fault detail must be a single line, got:\n{detail}");
        assert!(!detail.contains("backtrace"), "no guest backtrace may appear in the message: {detail}");
    });
}
