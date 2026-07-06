//! DeluluLang WASM backend (Stage 3, "Containment"). Compiles checked programs to WebAssembly
//! and runs them under an embedded, deny-by-default Wasmtime host. The interpreter remains the
//! reference semantics; this backend must produce identical observable results (two-engine
//! parity, §9) — that equivalence is the correctness contract, enforced by tests.
//!
//! Phase 3a scope: the pure-Int/Bool fragment (arithmetic, comparisons, `if`, `let`, recursion,
//! calls) compiled to core WASM and run under Wasmtime, verified equal to the interpreter.
//! Capabilities, strings, GC types, and the `delulu:cap` host interface build on this next.

mod artifact;
mod codegen;
pub mod gen;
mod host;

pub use artifact::{
    embed_authority, read_and_verify, Artifact, ArtifactError, AUTHORITY_SECTION, DWX_VERSION,
};
pub use codegen::{compile_module, uses_console, CompileError};
pub use host::{run_console_fn, run_int_fn, run_main_console, WasmError};

use delulu_syntax::ast::Module;

/// Compile a checked module and run one of its exported pure functions under Wasmtime.
pub fn compile_and_run_int(module: &Module, name: &str, args: &[i64]) -> Result<i64, String> {
    let wasm = compile_module(module).map_err(|e| e.message())?;
    run_int_fn(&wasm, name, args).map_err(|e| e.message())
}

#[cfg(test)]
mod tests {
    use super::*;
    use delulu_check::check_source;
    use delulu_runtime::Interp;

    /// The core Phase-3a correctness contract: for a pure program, the WASM engine and the
    /// interpreter must agree on the result of calling a function with Int arguments.
    fn assert_parity(src: &str, name: &str, args: &[i64]) {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "program should check clean: {:?}", checked.diagnostics);

        let interp = Interp::new(&checked.module);
        let interp_result = interp.call_int_fn(name, args).expect("interpreter run");

        let wasm_result = compile_and_run_int(&checked.module, name, args).expect("wasm run");

        assert_eq!(
            interp_result, wasm_result,
            "two-engine parity broken for {name}({args:?}): interpreter={interp_result}, wasm={wasm_result}"
        );
    }

    #[test]
    fn fib_matches_the_interpreter() {
        let src = "module m\nfn fib(n: Int) -> Int { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }\n";
        for n in 0..=15 {
            assert_parity(src, "fib", &[n]);
        }
    }

    #[test]
    fn arithmetic_and_helpers_match() {
        let src = "module m\n\
            fn square(n: Int) -> Int { n * n }\n\
            fn poly(a: Int, b: Int) -> Int { square(a) + square(b) - a * b }\n\
            fn gcd(a: Int, b: Int) -> Int { if b == 0 { a } else { gcd(b, a % b) } }\n";
        assert_parity(src, "square", &[7]);
        assert_parity(src, "poly", &[3, 4]);
        assert_parity(src, "gcd", &[48, 36]);
        assert_parity(src, "gcd", &[17, 5]);
    }

    #[test]
    fn booleans_negation_and_nested_ifs_match() {
        // Only Int-parameter functions (the i64-arg parity harness can't express Bool inputs);
        // Bool RESULTS and nested control flow are exercised.
        let src = "module m\n\
            fn even(n: Int) -> Bool { n % 2 == 0 }\n\
            fn neg(n: Int) -> Int { -n }\n\
            fn max3(a: Int, b: Int, c: Int) -> Int { if a > b { if a > c { a } else { c } } else { if b > c { b } else { c } } }\n";
        assert_parity(src, "even", &[10]);
        assert_parity(src, "even", &[7]);
        assert_parity(src, "neg", &[42]);
        assert_parity(src, "neg", &[-13]);
        assert_parity(src, "max3", &[3, 9, 5]);
        assert_parity(src, "max3", &[8, 2, 4]);
    }

    #[test]
    fn unsupported_body_reports_compile_error() {
        // A compilable signature (Int -> Int) whose BODY uses an unsupported construct (`match`)
        // is a CompileError (DL1201) — the interpreter stays the reference engine for it.
        let src = "module m\nfn f(n: Int) -> Int { match n { _ => n } }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        assert!(compile_module(&checked.module).is_err(), "a match body must not compile in Phase 3a");
    }

    #[test]
    fn non_compilable_functions_are_skipped_not_errored() {
        // A string function is simply not compiled (skipped); a module of only such functions
        // compiles to an empty WASM module without error.
        let src = "module m\nfn greet() -> Str { \"hi\" }\n";
        let checked = check_source(0, src);
        assert!(compile_module(&checked.module).is_ok());
    }

    // ----- Phase 3b: the delulu:cap host interface (console println) -------------------------

    #[test]
    fn println_effect_matches_the_interpreter() {
        use delulu_check::ResourceKind;
        use delulu_runtime::{set_capture, take_capture, CapScope, CapVal, Value};

        let src = "module m\nfn greet(out: Cap[Console]) ! {Write} { out.println(\"hello from wasm\") }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        assert!(uses_console(&checked.module), "greet must be detected as using the console");

        // WASM engine: run `greet` with a granted Console handle (index 0); the effect goes
        // through the delulu:cap host import, which reads the string from guest memory.
        let wasm = compile_module(&checked.module).expect("compile");
        let wasm_out = run_console_fn(&wasm, "greet", &[0]).expect("wasm run");

        // Interpreter (reference engine): capture its console output for the same call.
        set_capture(true);
        let interp = Interp::new(&checked.module);
        let cap = Value::Cap(std::rc::Rc::new(CapVal { kind: ResourceKind::Console, scope: CapScope::Console }));
        interp.call_with("greet", vec![cap]).expect("interp run");
        let interp_out = take_capture().expect("capture was on");

        assert_eq!(wasm_out, interp_out, "console output must match across engines");
        assert_eq!(wasm_out, "hello from wasm\n");
    }

    #[test]
    fn wasm_matches_interpreter_on_random_programs_including_faults() {
        // Differential parity gate (Phase 3d): thousands of random programs — INCLUDING ones that
        // overflow or divide by zero — must AGREE on both the value (both Ok and equal) and the
        // fault (both Err). With checked-arithmetic codegen the WASM backend faults exactly where
        // the interpreter does. Ok-vs-differing-Ok or Ok-vs-Err is a backend bug.
        let mut checked_count = 0u32;
        let mut agreed_ok = 0u32;
        let mut both_faulted = 0u32;
        let mut mismatches: Vec<String> = Vec::new();
        for k in 0..5000u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            let (src, args) = gen::random_pure_program(seed);
            let checked = check_source(0, &src);
            if checked.has_errors() {
                continue;
            }
            checked_count += 1;
            let interp = Interp::new(&checked.module);
            let iv = interp.call_int_fn("f", &args);
            let wv = compile_and_run_int(&checked.module, "f", &args);
            match (&iv, &wv) {
                (Ok(a), Ok(b)) if a == b => agreed_ok += 1,
                (Err(_), Err(_)) => both_faulted += 1, // both overflow / div-by-zero — consistent
                _ => mismatches.push(format!("seed {seed}: interp={iv:?} wasm={wv:?} args={args:?}\n{src}")),
            }
        }
        assert!(checked_count > 3000, "generator produced too few valid programs: {checked_count}");
        assert!(mismatches.is_empty(), "WASM/interpreter divergence ({} cases):\n{}", mismatches.len(), mismatches.iter().take(5).cloned().collect::<Vec<_>>().join("\n---\n"));
        // Prove the fault path was actually exercised (overflow / div-by-zero really occurred).
        assert!(both_faulted > 50, "the fault-parity path was barely exercised: both_faulted={both_faulted}");
        assert!(agreed_ok > 1000, "too few agreeing runs: agreed_ok={agreed_ok}");
    }

    #[test]
    fn generator_is_deterministic() {
        assert_eq!(gen::random_pure_program(123), gen::random_pure_program(123));
    }

    #[test]
    fn main_with_console_runs_on_wasm_matching_the_interpreter() {
        // The Phase-3e milestone: a real `main(root)` that does root.console() then out.println()
        // runs under Wasmtime — the capability is minted and checked host-side (root_console) — and
        // its output equals the interpreter's.
        use delulu_runtime::{set_capture, take_capture, Grants, Value};

        let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"hi from main\") }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);

        let wasm = compile_module(&checked.module).expect("compile");
        let wasm_out = run_main_console(&wasm, true).expect("wasm run");

        set_capture(true);
        let mut g = Grants::default();
        g.console = true;
        let interp = Interp::new(&checked.module);
        interp.run_main(Value::Root(std::rc::Rc::new(g.build_root()))).expect("interp run");
        let interp_out = take_capture().expect("capture on");

        assert_eq!(wasm_out, interp_out, "main console output must match across engines");
        assert_eq!(wasm_out, "hi from main\n");
    }

    #[test]
    fn main_console_is_refused_on_wasm_when_console_not_granted() {
        let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"x\") }\n";
        let checked = check_source(0, src);
        let wasm = compile_module(&checked.module).expect("compile");
        assert!(run_main_console(&wasm, false).is_err(), "ungranted console must be refused host-side");
    }

    // ----- Phase 3i: Str concatenation (runtime bump-allocated in guest memory) --------------

    /// Run a console `main` on both engines and assert byte-identical output; return it.
    fn main_console_parity(src: &str) -> String {
        use delulu_runtime::{set_capture, take_capture, Grants, Value};
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);

        let wasm = compile_module(&checked.module).expect("compile");
        let wasm_out = run_main_console(&wasm, true).expect("wasm run");

        set_capture(true);
        let mut g = Grants::default();
        g.console = true;
        let interp = Interp::new(&checked.module);
        interp.run_main(Value::Root(std::rc::Rc::new(g.build_root()))).expect("interp run");
        let interp_out = take_capture().expect("capture on");

        assert_eq!(wasm_out, interp_out, "wasm/interpreter output must match for:\n{src}");
        wasm_out
    }

    #[test]
    fn chained_literal_concatenation_matches_the_interpreter() {
        let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"[\" + \"x\" + \"]\") }\n";
        assert_eq!(main_console_parity(src), "[x]\n");
    }

    #[test]
    fn concatenation_with_a_parameter_matches_the_interpreter() {
        // greet takes a Str parameter and concatenates it with a literal — the parameter case.
        let src = "module m\n\
            fn greet(name: Str) -> Str { \"hello, \" + name }\n\
            fn main(root: Root) ! {Write} { let out = root.console()\n out.println(greet(\"world\")) }\n";
        assert_eq!(main_console_parity(src), "hello, world\n");
    }

    #[test]
    fn repeated_concatenation_advances_the_heap() {
        // Two independent concatenations in one run: the bump allocator must give each its own
        // buffer (the second must not clobber the first).
        let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"a\" + \"b\")\n out.println(\"cc\" + \"dd\") }\n";
        assert_eq!(main_console_parity(src), "ab\nccdd\n");
    }

    // ----- Phase 3j: str(Int) formatting in guest memory ------------------------------------

    #[test]
    fn int_to_str_matches_the_interpreter() {
        // Positive, negative, zero, and i64::MAX. (i64::MIN can't be a source literal — the parser
        // reads the magnitude first and 9223372036854775808 overflows i64 — so it's tested below.)
        for lit in ["0", "7", "42", "-1", "-9", "1000000", "-1000000", "9223372036854775807"] {
            let src = format!(
                "module m\nfn main(root: Root) ! {{Write}} {{ let out = root.console()\n out.println(str({lit})) }}\n"
            );
            assert_eq!(main_console_parity(&src), format!("{lit}\n"), "str({lit}) mismatch");
        }
    }

    #[test]
    fn int_to_str_of_i64_min_matches() {
        // i64::MIN reached by computation (`i64::MIN+1 - 1`), the magnitude-overflow edge: the
        // formatter's unsigned-magnitude trick must still print it correctly on both engines.
        let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(str(-9223372036854775807 - 1)) }\n";
        assert_eq!(main_console_parity(src), "-9223372036854775808\n");
    }

    #[test]
    fn int_to_str_of_a_computation_matches() {
        // The reference program's shape: str(fib(10)) printed on both engines.
        let src = "module m\n\
            fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }\n\
            fn main(root: Root) ! {Write} { let out = root.console()\n out.println(str(fib(10))) }\n";
        assert_eq!(main_console_parity(src), "55\n");
    }

    #[test]
    fn str_composes_with_concatenation() {
        // "n=" + str(n) — the string helpers compose (both bump-allocate in the same heap).
        let src = "module m\n\
            fn label(n: Int) -> Str { \"n=\" + str(n) }\n\
            fn main(root: Root) ! {Write} { let out = root.console()\n out.println(label(-7) + \"!\") }\n";
        assert_eq!(main_console_parity(src), "n=-7!\n");
    }

    #[test]
    fn println_of_concatenation_is_compilable_now() {
        // The construct that was DL1201 in Phase 3b (`println` of a `Str + Str`) compiles in 3i.
        let src = "module m\nfn greet(out: Cap[Console], name: Str) ! {Write} { out.println(\"hi, \" + name) }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        assert!(compile_module(&checked.module).is_ok(), "Str + Str must compile in Phase 3i");
    }

    #[test]
    fn ungranted_console_handle_traps_host_side() {
        // Handle 5 is out of the granted cap table → the host refuses the effect (scope check).
        let src = "module m\nfn greet(out: Cap[Console]) ! {Write} { out.println(\"x\") }\n";
        let checked = check_source(0, src);
        let wasm = compile_module(&checked.module).expect("compile");
        let err = run_console_fn(&wasm, "greet", &[5]);
        assert!(err.is_err(), "an ungranted capability handle must be refused host-side");
    }
}
