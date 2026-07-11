//! DeluluLang name resolution, type & effect/authority checking — the soundness core.
//!
//! Pipeline: `parse` (delulu-syntax) → `resolve` (declaration table) → `check` (the T-*
//! judgment, §6.2). The checker is where authority cannot escape the type: every rule from
//! `SOUNDNESS_AUDIT.md` (R-1…R-7) is enforced here or in `unify`.

pub mod authority;
pub mod check;
pub mod deps;
pub mod lockfile;
pub mod manifest;
pub mod package;
pub mod program;
pub mod resolve;
pub mod ty;
pub mod unify;

pub use authority::{authority_report, ScopeInfo};
pub use check::{CheckResult, FnFacts};
pub use deps::{
    check_pins, check_self_authority, check_workspace, package_authority, resolve_workspace,
    PackageAuthority, ResolvedPackage, Workspace,
};
pub use lockfile::{
    authority_widened, compute_entry, compute_lockfile, content_hash, enforce_semver_law,
    verify_locked, LockEntry, Lockfile,
};
pub use manifest::{AuthoritySpec, DepSource, Dependency, Manifest, PackageKind};
pub use package::{load_package, load_package_into, LoadedModules, ModuleUnit, Package};
pub use program::{check_program, program_authority, program_effects, Program};
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

    // ----- Stage 4: the foreign marshallability fence (phase 4b) -----------

    #[test]
    fn marshallable_foreign_block_checks_clean() {
        let c = check("module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float\n fn puts(s: Str) -> Int }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn foreignptr_marshals_but_pyobj_does_not() {
        let c = check("module m\nforeign \"c\" lib l { fn f(p: ForeignPtr) -> ForeignPtr }\n");
        assert!(!c.has_errors(), "ForeignPtr must marshal: {:?}", c.diagnostics);
        let e = errors("module m\nforeign \"c\" lib l { fn g(o: PyObj) -> Int }\n");
        assert!(e.contains(&"DL1301".to_string()), "PyObj must not marshal: {e:?}");
    }

    #[test]
    fn secret_param_in_foreign_sig_is_dl1301_and_never_suggests_expose() {
        let c = check("module m\nforeign \"c\" lib l { fn f(s: Secret[Str]) -> Int }\n");
        let d = c
            .diagnostics
            .iter()
            .find(|d| d.code == "DL1301")
            .unwrap_or_else(|| panic!("expected DL1301, got {:?}", c.diagnostics));
        // Criterion 3 (NON-NEGOTIABLE): the fence must NEVER suggest laundering a secret across
        // the FFI — `expose` may not appear anywhere in the diagnostic's repairs.
        for r in &d.repairs {
            assert!(!r.id.contains("expose"), "a repair id must not mention `expose`");
            for e in &r.edits {
                assert!(!e.insert.contains("expose"), "no repair may insert `expose`");
            }
        }
    }

    #[test]
    fn other_unmarshallable_types_in_foreign_sig_are_dl1301() {
        // Cap, the lib handle itself, List, and user types are all unmarshallable.
        for sig in [
            "fn f(c: Cap[FsRead]) -> Int",
            "fn f(m: l) -> Int",
            "fn f(xs: List[Int]) -> Int",
            "fn f(x: Int) -> List[Int]",
        ] {
            let src = format!("module m\nforeign \"c\" lib l {{ {sig} }}\n");
            let e = errors(&src);
            assert!(e.contains(&"DL1301".to_string()), "expected DL1301 for `{sig}`: {e:?}");
        }
    }

    #[test]
    fn function_typed_param_in_foreign_sig_is_dl1302_with_r6a() {
        let c = check("module m\nforeign \"c\" lib l { fn f(cb: fn(Int) -> Int) -> Int }\n");
        let d = c
            .diagnostics
            .iter()
            .find(|d| d.code == "DL1302")
            .unwrap_or_else(|| panic!("expected DL1302, got {:?}", c.diagnostics));
        assert!(d.message.contains("R-6a"), "DL1302 must reference rule R-6a: {}", d.message);
        assert!(d.repairs.is_empty(), "DL1302 is requires_human — no machine repairs");
    }

    #[test]
    fn nested_function_type_in_foreign_sig_is_dl1302() {
        // A function type nested inside a composite still trips the no-callbacks rule.
        let e = errors("module m\nforeign \"c\" lib l { fn f(xs: List[fn() -> Int]) -> Int }\n");
        assert!(e.contains(&"DL1302".to_string()), "{e:?}");
    }

    #[test]
    fn stringifying_a_foreign_handle_is_dl0604() {
        // A lib handle is R-5 opaque.
        let e = errors("module m\nforeign \"c\" lib l { }\nfn f(h: l) -> Str { str(h) }\n");
        assert!(e.contains(&"DL0604".to_string()), "{e:?}");
    }

    // ----- Stage 4: ForeignCall effect + T-ForeignBind + T-ForeignCall (4c) -----

    #[test]
    fn calling_a_foreign_method_without_foreigncall_in_row_is_dl0501() {
        let e = errors(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\nfn bad(m: mathlib) -> Float { m.cos(1.0) }\n",
        );
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
    }

    #[test]
    fn calling_a_foreign_method_with_foreigncall_declared_checks_clean() {
        let c = check(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\nfn good(m: mathlib) -> Float ! {ForeignCall} { m.cos(1.0) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        // T-ForeignCall: the method call carries exactly `{ForeignCall}`.
        assert!(c.result.facts["good"].effects.contains(&Effect::ForeignCall));
    }

    #[test]
    fn foreign_method_argument_types_are_checked() {
        // `cos` wants a Float; passing an Int is a type mismatch (DL0401/DL0402 class).
        let c = check(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\nfn f(m: mathlib) -> Float ! {ForeignCall} { m.cos(1) }\n",
        );
        assert!(c.has_errors(), "an Int arg to a Float foreign param must be rejected");
    }

    #[test]
    fn foreigncall_propagates_up_the_call_chain() {
        // This is exactly the chain `delulu why ForeignCall` walks: main -> mid -> leaf -> m.cos.
        let c = check(
            "module m\n\
             foreign \"c\" lib mathlib { fn cos(x: Float) -> Float }\n\
             fn leaf(m: mathlib) -> Float ! {ForeignCall} { m.cos(1.0) }\n\
             fn mid(m: mathlib) -> Float ! {ForeignCall} { leaf(m) }\n\
             fn main(m: mathlib) -> Float ! {ForeignCall} { mid(m) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["main"].effects.contains(&Effect::ForeignCall));
    }

    #[test]
    fn binding_a_foreign_lib_is_pure() {
        // T-ForeignBind: deriving a handle is not an effect (using it is).
        let c = check(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\nfn get(root: Root, load: Cap[ForeignLoad]) -> Result[mathlib, ForeignErr] { root.foreign(load) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["get"].pure, "binding a foreign lib must be pure");
    }

    // ----- Stage 4: root.foreign_load() + bind-site resolution (4d/4e) -----

    #[test]
    fn root_foreign_load_mints_cap_foreignload_and_is_pure() {
        // Head-chef ruling (normative in the Book ch. 14): `root.foreign_load() ->
        // Cap[ForeignLoad]` is a pure derivation from Root, like every other root.X() constructor.
        let c = check("module m\nfn get(root: Root) -> Cap[ForeignLoad] { root.foreign_load() }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["get"].pure, "minting Cap[ForeignLoad] must be pure");
        assert!(
            c.result.facts["get"].cap_kinds.contains(&ResourceKind::ForeignLoad),
            "the report must know the program wields ForeignLoad"
        );
    }

    #[test]
    fn foreign_bind_sites_resolve_their_lib_for_the_runtime() {
        // The `[M]` of `root.foreign[M](load)` is inferred from context; the interpreter needs the
        // resolved lib name per bind site to know which library to load (phase 4d).
        let c = check(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\nfn get(root: Root, load: Cap[ForeignLoad]) -> Result[mathlib, ForeignErr] { root.foreign(load) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert_eq!(c.result.foreign_binds.len(), 1, "{:?}", c.result.foreign_binds);
        assert_eq!(c.result.foreign_binds.values().next().map(String::as_str), Some("mathlib"));
    }

    #[test]
    fn full_bind_chain_with_root_foreign_load_checks_clean() {
        // The canonical Book ch. 14 shape: mint the load cap, bind with `?`, call the symbol.
        let c = check(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\n\
             fn compute(root: Root) -> Result[Float, ForeignErr] ! {ForeignCall} { let load = root.foreign_load()\n \
             let m: mathlib = root.foreign(load)?\n Ok(m.cos(1.0)) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert_eq!(c.result.foreign_binds.values().next().map(String::as_str), Some("mathlib"));
        assert!(c.result.facts["compute"].effects.contains(&Effect::ForeignCall));
    }

    #[test]
    fn foreignload_is_not_an_ordinary_capability_line_in_the_report() {
        // ForeignLoad/Python disclose under the "outside the proof" separator (`foreign_calls`),
        // never as a plain capability row (spec §6) — the foreign section is the single place the
        // proof's holes are enumerated.
        let c = check("module m\nfn main(root: Root) { let _l = root.foreign_load() }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        let report = authority_report("m", &c.result, &ScopeInfo::default());
        let caps = report["capabilities"].as_array().unwrap();
        assert!(caps.iter().all(|cap| cap["kind"] != "ForeignLoad"), "{caps:?}");
    }

    #[test]
    fn a_pure_program_authority_report_is_unchanged_by_stage4() {
        // Activating ForeignCall must not perturb a program that never touches foreign code.
        let c = check("module m\nfn f(n: Int) -> Int { n + 1 }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        let report = authority_report("m", &c.result, &ScopeInfo::default());
        let effects: Vec<String> =
            report["effects"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        assert!(effects.is_empty(), "a pure program must have no effects, got {effects:?}");
        assert_eq!(report["foreign_calls"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn row_polymorphism_pure_lambda_stays_pure() {
        let c = check(
            "module m\nfn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e { f(x) }\nfn g() -> Int { apply(fn(x: Int) -> Int { x * 2 }, 21) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["g"].pure);
    }

    // ----- Stage 4: embedded CPython (phase 4f, spec §5) -------------------

    #[test]
    fn python_surface_checks_clean_with_foreigncall() {
        // The criterion-2 shape: import, build a list, call a method, convert back. Every `std.py`
        // op has row exactly `{ForeignCall}`.
        let c = check(
            "module m\n\
             fn work(py: Cap[Python]) -> Result[Float, PyErr] ! {ForeignCall} {\n\
               let np = py.import(\"numpy\")?\n\
               let xs = py.list([py.of_float(1.0), py.of_float(2.0)])\n\
               let m = np.call_method(\"mean\", [xs])?\n\
               py.to_float(m) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["work"].effects.contains(&Effect::ForeignCall));
    }

    #[test]
    fn root_python_is_pure_and_returns_result_cap_python() {
        // T-Py binding: `root.python(load) -> Result[Cap[Python], ForeignErr]` is PURE (deriving the
        // handle is not an effect), and the report knows the program wields Python.
        let c = check(
            "module m\nfn get(root: Root, load: Cap[ForeignLoad]) -> Result[Cap[Python], ForeignErr] { root.python(load) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["get"].pure, "minting Cap[Python] must be pure");
        assert!(c.result.facts["get"].cap_kinds.contains(&ResourceKind::Python));
    }

    #[test]
    fn python_cap_is_not_an_ordinary_capability_row_in_the_report() {
        // Like ForeignLoad, Python discloses under the "outside the proof" separator, never as a
        // plain capability line (spec §6).
        let c = check(
            "module m\nfn main(root: Root) ! {ForeignCall} { let load = root.foreign_load()\n let _p = root.python(load) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        let report = authority_report("m", &c.result, &ScopeInfo::default());
        let caps = report["capabilities"].as_array().unwrap();
        assert!(caps.iter().all(|cap| cap["kind"] != "Python"), "{caps:?}");
    }

    #[test]
    fn closure_into_pyobj_call_is_dl1302_not_a_plain_mismatch() {
        // Spec §4.4 / R-6a: `obj.call([closure])` is the no-callbacks rule, DL1302 — the spec wants
        // the R-6a diagnostic, not a bare type mismatch. A pure-closure list (`List[fn]`) is caught
        // by the Python-arg fence.
        let e = errors(
            "module m\nfn f(o: PyObj, g: fn(Int) -> Int) -> Result[PyObj, PyErr] ! {ForeignCall} { o.call([g]) }\n",
        );
        assert!(e.contains(&"DL1302".to_string()), "{e:?}");
        assert!(!e.contains(&"DL0401".to_string()), "the R-6a diagnostic must replace the plain mismatch: {e:?}");
    }

    #[test]
    fn closure_mixed_into_pyobj_list_literal_is_dl1302() {
        // A closure alongside real PyObj elements: the mismatch surfaces at element unification, and
        // a function-into-PyObj is DL1302 (a closure can never become a Python value).
        let e = errors(
            "module m\nfn f(py: Cap[Python], o: PyObj) -> Result[PyObj, PyErr] ! {ForeignCall} { o.call([py.of_int(1), fn(x: Int) -> Int { x }]) }\n",
        );
        assert!(e.contains(&"DL1302".to_string()), "{e:?}");
    }

    #[test]
    fn pyobj_in_a_foreign_c_signature_is_dl1301() {
        // `PyObj` is opaque and NOT marshallable across the C boundary (spec §3 T-Py): the 4b fence
        // rejects it via M(τ).
        let e = errors("module m\nforeign \"c\" lib bad { fn takes(p: PyObj) -> Int }\n");
        assert!(e.contains(&"DL1301".to_string()), "{e:?}");
    }

    #[test]
    fn stringifying_a_pyobj_is_dl0604() {
        // `PyObj` is R-5 opaque: no str/serialize.
        let e = errors("module m\nfn f(o: PyObj) -> Str { str(o) }\n");
        assert!(e.contains(&"DL0604".to_string()), "{e:?}");
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
