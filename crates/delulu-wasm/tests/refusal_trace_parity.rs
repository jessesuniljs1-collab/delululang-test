//! RW 4.64 (routine run 23, D-V2-129): the two engines write the SAME trace for an operation that is refused —
//! its dispatch record, then `<method>.refused` with the code and its words. The interpreter records an
//! operation at its dispatch and its refusal after it (RW 4.62); the WASM engine checked custody BEFORE it traced
//! console, clock and rand, refused a hostile path before it traced `read_text`, and wrote nothing after
//! `rand.int`'s `hi <= lo` fault — so a refused operation was in one engine's trace and not the other's. Each case
//! runs one program on both engines, under a custody that refuses everything where the case is the custody's, and
//! asserts the two traces are byte-identical and end in the refusal.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, TraceRecord, TraceSink, Value};
use delulu_wasm::{compile_module, run_main, HostConfig};

/// A custody that refuses every operation, as a revoked lease does — in the broker's own code and words.
struct DenyAll;
impl delulu_runtime::Custody for DenyAll {
    fn check(&mut self, _op: delulu_runtime::Op, _arg: Option<&str>) -> delulu_runtime::CustodyDecision {
        delulu_runtime::CustodyDecision::Deny(delulu_runtime::CustodyDenial::new(
            "DL1403",
            "lease `g_x` was revoked by audit seq 42",
        ))
    }
    fn expose(&mut self, _n: &str, _s: Option<&str>) -> Result<String, delulu_runtime::CustodyDenial> {
        Err(delulu_runtime::CustodyDenial::new("DL1403", "revoked"))
    }
    fn refresh_epoch(&mut self) {}
    fn mode(&self) -> &'static str {
        "daemon"
    }
}

fn code_of(msg: &str) -> String {
    for code in ["DL1403", "DL0904", "DL0908"] {
        if msg.contains(code) {
            return code.to_string();
        }
    }
    msg.to_string()
}

/// Run `src` on both engines; `deny` puts each under [`DenyAll`]; `read_root` grants one directory for reading, as
/// `--grant fs.read=` spells it. Returns (wasm, interp) as (the refusal's code, the trace).
fn run_both(src: &str, deny: bool, read_root: Option<&str>) -> ((String, Vec<TraceRecord>), (String, Vec<TraceRecord>)) {
    // The grants first: the WASM host is given the interpreter's own normalized read roots, as `run` gives them.
    let grants = Grants {
        console: true,
        clock: true,
        rand: true,
        fs_read: read_root.map(|r| vec![r.to_string()]).unwrap_or_default(),
        ..Default::default()
    };
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "the program checks clean: {:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).expect("the program compiles to WASM");
    let wsink = TraceSink::new();
    let cfg = HostConfig {
        console: true,
        clock: true,
        rand: true,
        rand_seed: Some(7),
        fixed_clock_ms: Some(1_000),
        fs_read_roots: grants.build_root().fs_read,
        trace: Some(wsink.clone()),
        custody: deny.then(|| std::rc::Rc::new(std::cell::RefCell::new(Box::new(DenyAll) as Box<dyn delulu_runtime::Custody>))),
        ..HostConfig::default()
    };
    let w = run_main(&wasm, &cfg).expect_err("the WASM run is refused");
    let wasm_side = (code_of(&w.message()), wsink.records());

    let isink = TraceSink::new();
    delulu_runtime::prim::set_rand_seed(7);
    delulu_runtime::prim::set_fixed_clock_ms(Some(1_000));
    set_capture(true);
    let mut interp = Interp::new(&checked.module).with_trace(isink.clone());
    if deny {
        interp = interp.with_custody(Box::new(DenyAll));
    }
    let f = interp.run_main(Value::Root(Rc::new(grants.build_root()))).expect_err("the interpreter's run is refused");
    let _ = take_capture();
    (wasm_side, (f.code.to_string(), isink.records()))
}

fn assert_refusal_parity(label: &str, src: &str, deny: bool, read_root: Option<&str>, op: &str, code: &str) {
    let ((wcode, wtrace), (icode, itrace)) = run_both(src, deny, read_root);
    assert_eq!(icode, code, "{label}: the interpreter's refusal");
    assert_eq!(wcode, code, "{label}: the WASM engine's refusal");
    assert_eq!(itrace.len(), 2, "{label}: the dispatch, then its refusal: {itrace:#?}");
    assert_eq!(itrace[0].op, op, "{label}: {itrace:#?}");
    assert_eq!(itrace[1].op, format!("{op}.refused"), "{label}: {itrace:#?}");
    assert!(itrace[1].detail.as_deref().is_some_and(|d| d.starts_with(&format!("{code}: "))), "{label}: {itrace:#?}");
    assert_eq!(wtrace, itrace, "{label}: the two engines' traces are byte-identical");
}

#[test]
fn a_refused_console_clock_or_rand_use_is_traced_the_same_by_both_engines() {
    assert_refusal_parity(
        "console",
        "module m\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(\"never\")\n}\n",
        true,
        None,
        "println",
        "DL1403",
    );
    assert_refusal_parity(
        "clock",
        "module m\nfn main(root: Root) ! {Write, Clock} {\n    let out = root.console()\n    let c = root.clock()\n    out.println(str(c.now_ms()))\n}\n",
        true,
        None,
        "now_ms",
        "DL1403",
    );
    assert_refusal_parity(
        "rand",
        "module m\nfn main(root: Root) ! {Write, Rand} {\n    let out = root.console()\n    let r = root.rand()\n    out.println(str(r.int(1, 10)))\n}\n",
        true,
        None,
        "int",
        "DL1403",
    );
}

#[test]
fn an_operation_its_effect_refuses_is_traced_the_same_by_both_engines() {
    assert_refusal_parity(
        "rand.int with hi <= lo",
        "module m\nfn main(root: Root) ! {Write, Rand} {\n    let out = root.console()\n    let r = root.rand()\n    out.println(str(r.int(5, 5)))\n}\n",
        false,
        None,
        "int",
        // An argument outside what the operation accepts, not a scope violation (A15, D-V2-190).
        "DL0908",
    );
    // The working directory, granted as `.`: nothing is created or read — the spelling is refused first.
    assert_refusal_parity(
        "a path holding NUL",
        "module m\nfn main(root: Root) ! {Read} {\n    let fr = root.fs_read(\".\")\n    match fr.read_text(\"a\\u{0}b.txt\") {\n        Ok(v) => {}\n        Err(e) => {}\n    }\n}\n",
        false,
        Some("."),
        "read_text",
        "DL0904",
    );
}
