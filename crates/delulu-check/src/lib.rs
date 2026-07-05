//! DeluluLang name resolution, type & effect/authority checking — the soundness core.
//!
//! Pipeline: `parse` (delulu-syntax) → `resolve` (declaration table) → `check` (the T-*
//! judgment, §6.2). The checker is where authority cannot escape the type: every rule from
//! `SOUNDNESS_AUDIT.md` (R-1…R-7) is enforced here or in `unify`.

pub mod authority;
pub mod check;
pub mod package;
pub mod resolve;
pub mod ty;
pub mod unify;

pub use authority::{authority_report, ScopeInfo};
pub use check::{CheckResult, FnFacts};
pub use package::{load_package, ModuleUnit, Package};
pub use resolve::{DeclTable, FnSig, GKind};
pub use ty::{Effect, ResourceKind, Row, RowVar, Type, TypeDefId};

use delulu_diag::{Diagnostic, FileId};
use delulu_syntax::ast::Module;

/// The full result of checking one source file.
pub struct Checked {
    pub module: Module,
    pub table: DeclTable,
    pub result: CheckResult,
    /// Parse + resolve + check diagnostics, in phase order.
    pub diagnostics: Vec<Diagnostic>,
}

impl Checked {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }
}

/// Lex, parse, resolve, and check one source file. Always returns a `Checked` (possibly with
/// diagnostics) so downstream stages — the CLI, the REPL, tests — can decide how to proceed.
pub fn check_source(file: FileId, src: &str) -> Checked {
    let (module, mut diagnostics) = delulu_syntax::parse_file(file, src);
    let (table, rdiags) = resolve::resolve(&module);
    diagnostics.extend(rdiags);
    let result = check::check_module(&module, &table);
    diagnostics.extend(result.diags.iter().cloned());
    Checked { module, table, result, diagnostics }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Checked {
        check_source(0, src)
    }

    fn errors(src: &str) -> Vec<String> {
        check(src)
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| d.code.to_string())
            .collect()
    }

    #[test]
    fn pure_function_checks_clean_and_is_pure() {
        let c = check("module m\nfn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["fib"].pure);
    }

    #[test]
    fn undeclared_effect_is_dl0501() {
        // greet calls println (Write) but declares no row.
        let e = errors("module m\nfn greet(out: Cap[Console], n: Str) { out.println(n) }\n");
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
    }

    #[test]
    fn declared_effect_makes_it_check() {
        let c = check("module m\nfn greet(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert_eq!(c.result.facts["greet"].effects.iter().next().unwrap().name(), "Write");
    }

    #[test]
    fn secret_cannot_flow_into_println_is_dl0602() {
        // println expects Str; a Secret[Str] cannot flow there — the dedicated secret-non-flow
        // diagnostic (Secret[T] is not T).
        let e = errors("module m\nfn f(out: Cap[Console], s: Secret[Str]) ! {Write} { out.println(s) }\n");
        assert!(e.contains(&"DL0602".to_string()), "{e:?}");
    }

    #[test]
    fn stringifying_a_secret_is_dl0604() {
        let e = errors("module m\nfn f(s: Secret[Str]) -> Str { str(s) }\n");
        assert!(e.contains(&"DL0604".to_string()), "{e:?}");
    }

    #[test]
    fn equality_on_secret_is_dl0605() {
        let e = errors("module m\nfn f(a: Secret[Str], b: Secret[Str]) -> Bool { a == b }\n");
        assert!(e.contains(&"DL0605".to_string()), "{e:?}");
    }

    #[test]
    fn row_polymorphism_pure_lambda_stays_pure() {
        let c = check(
            "module m\nfn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e { f(x) }\nfn g() -> Int { apply(fn(x: Int) -> Int { x * 2 }, 21) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["g"].pure);
    }

    #[test]
    fn the_reference_program_checks_clean() {
        let src = include_str!("../../../examples/demo.delulu");
        let c = check(src);
        assert!(!c.has_errors(), "reference program should check clean: {:?}", c.diagnostics);
        // Authority report: main can Read and Write, nothing else.
        let report = authority_report("demo", &c.result, &ScopeInfo::default());
        let effects: Vec<String> = report["effects"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        assert_eq!(effects, vec!["Read", "Write"]);
        let pure: Vec<String> = report["pure_functions"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        assert!(pure.contains(&"fib".to_string()) && pure.contains(&"apply".to_string()));
        assert_eq!(report["secrets"][0], "API_KEY");
    }
}
