//! The Stage-3 §9 criterion-9 gate: a large-scale two-engine differential fuzz. Every generated
//! program is run on BOTH the interpreter (the reference engine) and the WASM backend; their results
//! must agree — identical value/output when both succeed, or both faulting (a fault is a fault). Any
//! Ok-vs-differing-Ok or Ok-vs-Err is a compiler-bug-class divergence (DL1206).
//!
//! `#[ignore]`d so it doesn't slow the normal suite; run the full 50k gate with
//!   cargo test -p delulu-wasm --test differential -- --ignored
//! The count is `DELULU_FUZZ_N` (default 50000), so a quick confidence pass is
//!   DELULU_FUZZ_N=3000 cargo test -p delulu-wasm --test differential -- --ignored

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_and_run_int, compile_module, gen, run_main, HostConfig};

struct Stats {
    checked: u64,
    agreed: u64,
    both_faulted: u64,
    mismatches: Vec<String>,
}

/// Run one random PURE program (`f(a,b,c) -> Int`) on both engines and classify.
fn run_pure(seed: u64, s: &mut Stats) {
    let (src, args) = gen::random_pure_program(seed);
    let checked = check_source(0, &src);
    if checked.has_errors() {
        return;
    }
    s.checked += 1;
    let interp = Interp::new(&checked.module);
    let iv = interp.call_int_fn("f", &args);
    let wv = compile_and_run_int(&checked.module, "f", &args);
    match (&iv, &wv) {
        (Ok(a), Ok(b)) if a == b => s.agreed += 1,
        (Err(_), Err(_)) => s.both_faulted += 1,
        _ => s.mismatches.push(format!("PURE seed {seed}: interp={iv:?} wasm={wv:?} args={args:?}\n{src}")),
    }
}

/// Run one random CONSOLE program (`main` printing `str`/concat of safe arithmetic) on both engines.
fn run_console(seed: u64, s: &mut Stats) {
    let src = gen::random_console_program(seed);
    let checked = check_source(0, &src);
    if checked.has_errors() {
        return;
    }
    let wasm = match compile_module(&checked.module) {
        Ok(w) => w,
        Err(_) => return,
    };
    s.checked += 1;
    let wr = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() });

    set_capture(true);
    let g = Grants {
        console: true,
        ..Default::default()
    };
    let interp = Interp::new(&checked.module);
    let ir = interp.run_main(Value::Root(Rc::new(g.build_root())));
    let io = take_capture().unwrap_or_default();

    match (&wr, &ir) {
        (Ok(wo), Ok(_)) if *wo == io => s.agreed += 1,
        (Err(_), Err(_)) => s.both_faulted += 1,
        _ => s.mismatches.push(format!("CONSOLE seed {seed}: wasm={wr:?} interp_out={io:?}\n{src}")),
    }
}

#[test]
#[ignore = "large-scale gate; run with --ignored (DELULU_FUZZ_N to size)"]
fn two_engine_differential_gate() {
    let n: u64 = std::env::var("DELULU_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(50_000);
    let mut s = Stats { checked: 0, agreed: 0, both_faulted: 0, mismatches: Vec::new() };

    // ~60% pure (fast, arithmetic edge cases + faults), ~40% console (str/concat/print output).
    for k in 0..n {
        let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        if k % 5 < 3 {
            run_pure(seed, &mut s);
        } else {
            run_console(seed, &mut s);
        }
    }

    println!(
        "differential: n={n} checked={} agreed={} both_faulted={} mismatches={}",
        s.checked, s.agreed, s.both_faulted, s.mismatches.len()
    );
    assert!(
        s.mismatches.is_empty(),
        "two-engine divergence ({} of {} checked):\n{}",
        s.mismatches.len(),
        s.checked,
        s.mismatches.iter().take(5).cloned().collect::<Vec<_>>().join("\n---\n")
    );
    // The generators reject some programs; require a healthy majority actually ran on both engines.
    assert!(s.checked * 5 >= n * 4, "too few programs were checkable: {} of {n}", s.checked);
    assert!(s.both_faulted > 20, "fault-parity path barely exercised: {}", s.both_faulted);
}
