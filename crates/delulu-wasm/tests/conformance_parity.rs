//! Two-engine conformance/differential parity (Stage 3 §9 criteria 1–3), consolidating phases
//! 3a–3l. The interpreter is the reference engine; the WASM backend must produce byte-identical
//! observable output for every program inside its fragment. This file proves that three ways:
//!   1. a curated set of console programs exercising console + str + concat + clock + rand together;
//!   2. a generative fuzzer over random console programs (str/concat/print of safe arithmetic);
//!   3. fragment-external programs cleanly become `CompileError` (DL1201) so they fall back to the
//!      interpreter rather than miscompile.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, set_fixed_clock_ms, set_rand_seed, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_module, gen::random_console_program, run_main, HostConfig};

/// Run `main` on both engines under the same grants/determinism and return `(wasm_out, interp_out)`.
fn run_both(src: &str, cfg: HostConfig) -> (String, String) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean: {:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).expect("program should compile to WASM");
    let wasm_out = run_main(&wasm, &cfg).expect("wasm run");

    if let Some(ms) = cfg.fixed_clock_ms {
        set_fixed_clock_ms(Some(ms));
    }
    if let Some(s) = cfg.rand_seed {
        set_rand_seed(s);
    }
    set_capture(true);
    let g = Grants {
        console: cfg.console,
        clock: cfg.clock,
        rand: cfg.rand,
        ..Default::default()
    };
    let interp = Interp::new(&checked.module);
    interp.run_main(Value::Root(Rc::new(g.build_root()))).expect("interp run");
    let interp_out = take_capture().expect("capture was on");
    set_fixed_clock_ms(None);

    (wasm_out, interp_out)
}

fn assert_parity(src: &str, cfg: HostConfig) -> String {
    let (w, i) = run_both(src, cfg);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}\n wasm={w:?}\n interp={i:?}");
    w
}

// ----- 1. Curated programs ------------------------------------------------------------------

#[test]
fn everything_together_matches() {
    // console + a pure helper + str(Int) + concat + Cap[Clock] + Cap[Rand], all in one program.
    let src = "module m\n\
        fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }\n\
        fn describe(n: Int) -> Str { \"fib=\" + str(n) }\n\
        fn main(root: Root) ! {Write, Clock, Rand} {\n\
        \x20 let out = root.console()\n\
        \x20 let c = root.clock()\n\
        \x20 let r = root.rand()\n\
        \x20 out.println(\"start\")\n\
        \x20 out.println(describe(fib(10)))\n\
        \x20 out.println(\"t=\" + str(c.now_ms()))\n\
        \x20 out.println(\"roll=\" + str(r.int(1, 7)))\n\
        \x20 out.println(\"roll=\" + str(r.int(1, 7)))\n\
        }\n";
    let out = assert_parity(src, HostConfig { console: true, clock: true, rand: true, fixed_clock_ms: Some(1_700_000_000_000), rand_seed: Some(7), ..HostConfig::default() });
    // Spot-check the deterministic parts (clock is fixed; fib(10) = 55).
    assert!(out.contains("fib=55"), "{out}");
    assert!(out.contains("t=1700000000000"), "{out}");
}

#[test]
fn negatives_and_nested_arithmetic_match() {
    let src = "module m\nfn main(root: Root) ! {Write} {\n\
        \x20 let out = root.console()\n\
        \x20 out.println(str(0 - 9223372036854775807))\n\
        \x20 out.println(\"(\" + str(-3 * 7) + \")\")\n\
        \x20 out.println(str((100 / 3) % 7))\n\
        }\n";
    assert_parity(src, HostConfig { console: true, ..HostConfig::default() });
}

#[test]
fn deeply_chained_concatenation_matches() {
    let src = "module m\nfn main(root: Root) ! {Write} {\n\
        \x20 let out = root.console()\n\
        \x20 out.println(\"a\" + \"b\" + str(1) + \"c\" + str(2 + 3) + \"d\")\n\
        }\n";
    assert_eq!(assert_parity(src, HostConfig { console: true, ..HostConfig::default() }), "ab1c5d\n");
}

// ----- 2. Generative fuzzer -----------------------------------------------------------------

#[test]
fn random_console_programs_match_across_engines() {
    let mut compared = 0u32;
    let mut mismatches: Vec<String> = Vec::new();
    for k in 0..2000u64 {
        let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let src = random_console_program(seed);
        let checked = check_source(0, &src);
        if checked.has_errors() {
            mismatches.push(format!("seed {seed}: generated program did not check clean:\n{src}\n{:?}", checked.diagnostics));
            continue;
        }
        let wasm = compile_module(&checked.module).expect("generated console program should compile");
        let wasm_res = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() });

        set_capture(true);
        let g = Grants {
            console: true,
            ..Default::default()
        };
        let interp = Interp::new(&checked.module);
        let interp_res = interp.run_main(Value::Root(Rc::new(g.build_root())));
        let interp_out = take_capture().unwrap_or_default();

        match (&wasm_res, &interp_res) {
            (Ok(wo), Ok(_)) => {
                compared += 1;
                if *wo != interp_out {
                    mismatches.push(format!("seed {seed}: wasm={wo:?} interp={interp_out:?}\n{src}"));
                }
            }
            (Err(_), Err(_)) => {} // both faulted — consistent (the generator is fault-free, so rare)
            _ => mismatches.push(format!("seed {seed}: one engine faulted: wasm={wasm_res:?} interp_ok={}\n{src}", interp_res.is_ok())),
        }
    }
    assert!(mismatches.is_empty(), "console parity broken ({} cases):\n{}", mismatches.len(), mismatches.iter().take(5).cloned().collect::<Vec<_>>().join("\n---\n"));
    assert!(compared > 1500, "the generator produced too few comparable programs: {compared}");
}

#[test]
fn console_generator_is_deterministic() {
    assert_eq!(random_console_program(2024), random_console_program(2024));
}

// ----- 3. Fragment-external programs fall back (DL1201) --------------------------------------

#[test]
fn fragment_external_programs_are_compile_errors() {
    // Each has a COMPILABLE signature but a body outside the WASM fragment, so `compile_module`
    // must refuse (DL1201-class) — the CLI then runs it on the interpreter instead of miscompiling.
    let cases = [
        "module m\nfn f(n: Int) -> Int { match n { _ => n } }\n", // match
        "module m\nfn f() -> Int { len(range(0, 5)) }\n",         // list builtins (range/len)
        "module m\nfn f(s: Str) -> Str { s.trim() }\n",          // an unsupported Str method
    ];
    for src in cases {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "case should check clean: {src}\n{:?}", checked.diagnostics);
        assert!(compile_module(&checked.module).is_err(), "should be a DL1201 CompileError (fall back to interpreter): {src}");
    }
}
