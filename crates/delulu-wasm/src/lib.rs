//! DeluluLang WASM backend (Stage 3, "Containment"). Compiles checked programs to WebAssembly
//! and runs them under an embedded, deny-by-default Wasmtime host. The interpreter remains the
//! reference semantics; this backend must produce identical observable results (two-engine
//! parity, §9) — that equivalence is the correctness contract, enforced by tests.
//!
//! Phase 3a scope: the pure-Int/Bool fragment (arithmetic, comparisons, `if`, `let`, recursion,
//! calls) compiled to core WASM and run under Wasmtime, verified equal to the interpreter.
//! Capabilities, strings, GC types, and the `delulu:cap` host interface build on this next.

mod codegen;
mod host;

pub use codegen::{compile_module, CompileError};
pub use host::{run_int_fn, WasmError};

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
}
