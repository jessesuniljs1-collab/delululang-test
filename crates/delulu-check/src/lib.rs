//! DeluluLang name resolution, type & effect/authority checking — the soundness core.
//!
//! Pipeline: `parse` (delulu-syntax) → `resolve` (declaration table) → `check` (the T-*
//! judgment, §6.2). The checker is where authority cannot escape the type: every rule from
//! `SOUNDNESS_AUDIT.md` (R-1…R-7) is enforced here or in `unify`.

pub mod authority;
pub mod check;
pub mod deps;
pub mod dir;
pub mod lockfile;
pub mod manifest;
pub mod package;
pub mod plugin;
pub mod program;
pub mod rcap_check;
pub mod rcaps;
pub mod resolve;
pub mod ty;
pub mod unify;

pub use authority::{authority_report, ScopeInfo};
pub use check::{CheckResult, FnFacts};
pub use deps::{
    check_pins, check_self_authority, check_workspace, package_authority, resolve_workspace,
    PackageAuthority, ResolvedPackage, Workspace,
};
pub use dir::{
    deserialize as dir_deserialize, serialize as dir_serialize, verify as dir_verify, Dir, DirError,
};
pub use lockfile::{
    authority_widened, compute_entry, compute_lockfile, content_hash, enforce_semver_law,
    verify_locked, LockEntry, Lockfile,
};
pub use manifest::{AuthoritySpec, DepSource, Dependency, Manifest, PackageKind};
pub use package::{load_package, load_package_into, LoadedModules, ModuleUnit, Package};
pub use plugin::{
    check_plugin_module, render_type, PluginAuthority, PluginManifest, PLUGIN_API_SUPPORTED,
};
pub use program::{check_program, program_authority, program_effects, Program};
pub use resolve::{DeclTable, FnSig, GKind};
pub use ty::{Effect, PluginClass, ResourceKind, Row, RowVar, Type, TypeDefId};

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

    // ----- Stage 6: the plugin type surface (Plugin[C], load, get, unload) ----------------

    /// The Stage-1 §8.2 host API shape. `[C]`/`[F]` are inference-from-context (build-order
    /// deviation 4 — Stage-1 §6: "no turbofish"), so the class comes from the annotation.
    #[test]
    fn load_types_as_result_plugin_with_load_and_read_in_the_row() {
        let c = check(
            "module m\nfn go(root: Root, g: Grant) -> Result[Plugin[Verified], PluginErr] ! {Load, Read} {\n\
             let host = root.plugin_host()\n Ok(load(host, \"p.dpx\", g)?) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        // `load` carries {Load, Read}: bringing in code after compile time is itself in the row.
        let f = &c.result.facts["go"];
        assert!(f.effects.contains(&Effect::Load) && f.effects.contains(&Effect::Read), "{f:?}");
        assert!(f.cap_kinds.contains(&ResourceKind::PluginHost));
    }

    #[test]
    fn load_without_load_in_the_row_is_dl0501() {
        // A program that can bring in runtime code MUST say so in its row.
        let e = errors(
            "module m\nfn go(root: Root, g: Grant) -> Result[Plugin[Verified], PluginErr] ! {Read} {\n\
             let host = root.plugin_host()\n Ok(load(host, \"p.dpx\", g)?) }\n",
        );
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
    }

    #[test]
    fn the_class_annotation_pins_c_and_the_two_classes_never_unify() {
        // Invariant 29 in the types: Verified and Contained are nominal and never coerce.
        let c = check(
            "module m\nfn go(root: Root, g: Grant) -> Result[Plugin[Contained], PluginErr] ! {Load, Read} {\n\
             let host = root.plugin_host()\n Ok(load(host, \"p.dpx\", g)?) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        // Annotating the same load as the OTHER class in one function is a type error.
        let e = errors(
            "module m\nfn go(root: Root, g: Grant) ! {Load, Read} {\n\
             let host = root.plugin_host()\n\
             let p: Plugin[Verified] = load(host, \"p.dpx\", g)?\n\
             let q: Plugin[Contained] = p\n }\n",
        );
        assert!(!e.is_empty(), "Verified must never unify with Contained");
    }

    #[test]
    fn get_on_a_verified_plugin_yields_the_annotated_type_and_its_row_flows_to_the_caller() {
        // The flagship's typing shape (criterion 1): a zero-authority export types pure, and the
        // host's row is unchanged by calling it.
        let c = check(
            "module m\nfn use_it(p: Plugin[Verified]) -> Result[Str, PluginErr] {\n\
             let shout: fn(Str) -> Str ! {} = p.get(\"shout\")?\n Ok(shout(\"hi\")) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["use_it"].pure, "a pure export leaves the host's row pure");
    }

    #[test]
    fn an_effectful_export_forces_the_caller_to_declare_the_effect() {
        // Audit F-1's compile-time half (criterion 2): F's row flows into the caller via T-Call, so
        // the honest annotation only compiles when the caller's row admits it. No new rule needed.
        let e = errors(
            "module m\nfn use_it(p: Plugin[Verified]) -> Result[Str, PluginErr] {\n\
             let f: fn() -> Str ! {Net} = p.get(\"scan\")?\n Ok(f()) }\n",
        );
        assert!(e.contains(&"DL0501".to_string()), "calling a !{{Net}} export needs Net in the row: {e:?}");

        let c = check(
            "module m\nfn use_it(p: Plugin[Verified]) -> Result[Str, PluginErr] ! {Net} {\n\
             let f: fn() -> Str ! {Net} = p.get(\"scan\")?\n Ok(f()) }\n",
        );
        assert!(!c.has_errors(), "with Net declared it checks: {:?}", c.diagnostics);
    }

    #[test]
    fn a_function_typed_param_to_a_contained_export_is_dl0803_at_compile_time() {
        // R-6a / audit F-6a (criterion 7): DL0803, at the `get` call site, at COMPILE time.
        let e = errors(
            "module m\nfn use_it(p: Plugin[Contained]) -> Result[Int, PluginErr] {\n\
             let f: fn(fn(Int) -> Int) -> Int = p.get(\"apply\")?\n Ok(f(fn(x: Int) -> Int { x })) }\n",
        );
        assert!(e.contains(&"DL0803".to_string()), "{e:?}");
    }

    #[test]
    fn a_nested_function_typed_param_to_a_contained_export_is_also_dl0803() {
        // "anywhere in F, including nested" — a funcref inside a composite is still a funcref.
        let e = errors(
            "module m\nfn use_it(p: Plugin[Contained]) -> Result[Int, PluginErr] {\n\
             let f: fn(List[fn() -> Int]) -> Int = p.get(\"apply_all\")?\n Ok(f([])) }\n",
        );
        assert!(e.contains(&"DL0803".to_string()), "{e:?}");
    }

    #[test]
    fn a_function_typed_param_to_a_verified_export_is_allowed() {
        // The asymmetry IS the Verified/Contained split: Verified code is re-proved at load, so the
        // callback's row is real and R-4 composes it. Only Contained refuses (R-6a).
        let c = check(
            "module m\nfn use_it(p: Plugin[Verified]) -> Result[Int, PluginErr] {\n\
             let f: fn(fn(Int) -> Int) -> Int = p.get(\"apply\")?\n Ok(f(fn(x: Int) -> Int { x })) }\n",
        );
        assert!(!c.has_errors(), "Verified plugins accept callbacks: {:?}", c.diagnostics);
    }

    // ----- R-6a is FAIL-CLOSED at a Contained `get` (DL1509) ------------------------------------
    //
    // These two were found FAILING OPEN in review and are permanent witnesses. R-6a is a security
    // rule: "inference could not tell" must refuse, never skip.

    /// (a) `F` never pinned — no annotation, no unifying use. R-6a is undecidable, so it refuses.
    #[test]
    fn an_unpinned_get_on_a_contained_plugin_is_dl1509_not_a_silent_skip() {
        let e = errors(
            "module m\nfn use_it(p: Plugin[Contained]) -> Result[Int, PluginErr] {\n\
             let f = p.get(\"x\")?\n Ok(1) }\n",
        );
        assert!(e.contains(&"DL1509".to_string()), "an unresolved F must refuse, never skip: {e:?}");
    }

    /// (b) GENERIC LAUNDERING — the regression that matters. A generic's variables are instantiated
    /// FRESH per call site, so the body's `T` is never unified with the closure the caller passes:
    /// `F` reads as `fn('t0) -> Str`, `type_contains_fn` says false, and R-6a would never fire while
    /// a closure reaches an opaque module at runtime. DL1509 closes it at the `get` site.
    #[test]
    fn generic_laundering_of_a_closure_into_a_contained_export_is_dl1509() {
        let e = errors(
            "module m\n\
             fn helper[T](p: Plugin[Contained], x: T) -> Result[Str, PluginErr] {\n\
               let f: fn(T) -> Str ! {} = p.get(\"g\")?\n Ok(f(x)) }\n\
             fn caller(p: Plugin[Contained]) -> Result[Str, PluginErr] {\n\
               helper(p, fn(n: Int) -> Int { n }) }\n",
        );
        assert!(
            e.contains(&"DL1509".to_string()),
            "a generic parameter could be instantiated with a function type — R-6a must refuse, not skip: {e:?}"
        );
    }

    /// The fail-closed rule is scoped: a concrete, function-free Contained signature still checks.
    #[test]
    fn a_concrete_function_free_contained_get_still_checks_clean() {
        let c = check(
            "module m\nfn use_it(p: Plugin[Contained]) -> Result[Str, PluginErr] ! {Read} {\n\
             let f: fn(Str) -> Str ! {Read} = p.get(\"scan\")?\n Ok(f(\"x\")) }\n",
        );
        assert!(!c.has_errors(), "a concrete Contained signature is fine: {:?}", c.diagnostics);
    }

    /// And it does not touch Verified: generics there are safe, because Verified code is re-proved
    /// at load and a callback's row is real (R-4).
    #[test]
    fn a_generic_get_on_a_verified_plugin_is_not_refused() {
        let c = check(
            "module m\n\
             fn helper[T](p: Plugin[Verified], x: T) -> Result[Str, PluginErr] {\n\
               let f: fn(T) -> Str ! {} = p.get(\"g\")?\n Ok(f(x)) }\n",
        );
        assert!(!c.has_errors(), "Verified plugins are unaffected by the R-6a fence: {:?}", c.diagnostics);
    }

    #[test]
    fn a_plugin_handle_is_r5_opaque() {
        // A plugin handle binds a live broker node: stringifying or comparing one would leak or
        // forge authority identity.
        assert!(errors("module m\nfn f(p: Plugin[Verified]) -> Str { str(p) }\n").contains(&"DL0604".to_string()));
        assert!(errors("module m\nfn f(a: Plugin[Verified], b: Plugin[Verified]) -> Bool { a == b }\n")
            .contains(&"DL0605".to_string()));
    }

    #[test]
    fn unload_types_as_unit() {
        let c = check("module m\nfn drop_it(p: Plugin[Verified]) { p.unload() }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn grant_and_limits_are_ordinary_records_that_confer_nothing() {
        // spec §4: they DESCRIBE authority; building one is pure and needs no capability at all.
        let c = check(
            "module m\nfn mk() -> Grant {\n\
             Grant { effects: [\"Read\"], fs_read: [\"./docs\"], fs_write: [], net: [], secrets: [],\n\
             declassify: [], limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 }, require_signed: false } }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["mk"].pure, "constructing a Grant confers nothing and is pure");
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

    // ----- Stage 7 phase 7c: viewpoint adaptation + read/write rules (criterion 4) ------

    #[test]
    fn write_through_a_box_receiver_is_dl1604() {
        let e = errors(
            "module m\ntype P { x: Int }\nfn f(p: box P) { p.x = 1 }\n",
        );
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn write_through_a_val_defaulted_param_is_dl1604() {
        // No written rcap: `P` is a record of only-val fields, so the param defaults `val` —
        // deeply immutable, not writable.
        let e = errors("module m\ntype P { x: Int }\nfn f(p: P) { p.x = 1 }\n");
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn write_through_a_ref_receiver_is_clean() {
        let c = check("module m\ntype P { x: Int }\nfn f(p: ref P) { p.x = 1 }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn field_access_through_tag_is_dl1604() {
        let e = errors("module m\ntype P { x: Int }\nfn f(p: tag P) -> Int { p.x }\n");
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn field_read_through_box_is_clean() {
        let c = check("module m\ntype P { x: Int }\nfn f(p: box P) -> Int { p.x }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn push_on_a_val_defaulted_list_param_is_dl1604() {
        // `List[Int]` defaults `val` (spec §2): a parameter so typed is immutable data.
        let e = errors("module m\nfn f(xs: List[Int]) { xs.push(1) }\n");
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn push_on_a_ref_list_param_is_clean() {
        let c = check("module m\nfn f(xs: ref List[Int]) { xs.push(1) }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn free_form_push_gets_the_same_write_rule_as_method_push() {
        // `push(xs, v)` and `xs.push(v)` are the same mutation; the write rule must not be
        // dodgeable by spelling (kitchen rule: hunt the skip branch).
        let e = errors("module m\nfn f(xs: List[Int]) { push(xs, 1) }\n");
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
        let c = check("module m\nfn g(xs: ref List[Int]) { push(xs, 1) }\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn build_and_return_lifts_a_fresh_unescaped_local() {
        // The bread-and-butter pattern: build a list mutably, return it at the (val-defaulted)
        // return type. The local is fresh-born and never escaped, so the lift is sound —
        // mutating it via `push` is receiver use, not an escape.
        let c = check(
            "module m\nfn build(n: Int) -> List[Int] {\n\
             let xs = [0]\n\
             var i = 0\n\
             while i < n { xs.push(i)\ni = i + 1 }\n\
             xs\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn an_escaped_local_cannot_lift_at_return() {
        // Passing xs to a call could retain an alias; returning it as (defaulted) val after
        // that would let the caller and the retained alias disagree about immutability.
        let e = errors(
            "module m\nfn sink(xs: ref List[Int]) { }\n\
             fn f() -> List[Int] {\n\
             let xs = [1]\n\
             sink(xs)\n\
             xs\n}\n",
        );
        assert!(e.contains(&"DL1603".to_string()), "{e:?}");
    }

    #[test]
    fn storing_an_aliased_ref_where_iso_is_demanded_is_dl1603_with_consume_hint() {
        let c = check(
            "module m\nfn f() {\n\
             let xs: ref List[Int] = [1]\n\
             let ys: iso List[Int] = xs\n}\n",
        );
        let d = c.diagnostics.iter().find(|d| d.code == "DL1603");
        assert!(d.is_some(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_val_lambda_capturing_a_ref_is_dl1603() {
        // Criterion: a `val` (sendable) claim cannot rest on a mutable capture (spec §3).
        let e = errors(
            "module m\nfn g() {\n\
             let xs: ref List[Int] = [1]\n\
             let f: val fn() -> Int = fn() -> Int { xs.len() }\n}\n",
        );
        assert!(e.contains(&"DL1603".to_string()), "{e:?}");
    }

    #[test]
    fn a_pure_lambda_is_val_and_flows_into_fn_params() {
        // Deviation 9: fn positions default `box`, so both val and ref closures pass; and an
        // all-val-capture lambda satisfies an explicit `val` demand.
        let c = check(
            "module m\nfn apply(f: fn(Int) -> Int, x: Int) -> Int { f(x) }\n\
             fn g() -> Int { apply(fn(n: Int) -> Int { n * 2 }, 3) }\n\
             fn h() {\n\
             let f: val fn(Int) -> Int = fn(n: Int) -> Int { n + 1 }\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn passing_an_aliased_ref_list_where_the_param_defaults_val_is_dl1603() {
        // The laundering channel: a val parameter fed shared mutable state could later be
        // sent across an actor boundary as "immutable".
        let e = errors(
            "module m\nfn reader(xs: List[Int]) -> Int { xs.len() }\n\
             fn f() -> Int {\n\
             let xs: ref List[Int] = [1]\n\
             reader(xs)\n}\n",
        );
        assert!(e.contains(&"DL1603".to_string()), "{e:?}");
    }

    #[test]
    fn a_fresh_literal_argument_satisfies_a_val_param() {
        let c = check(
            "module m\nfn reader(xs: List[Int]) -> Int { xs.len() }\n\
             fn f() -> Int { reader([1, 2, 3]) }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn viewpoint_write_requires_the_adapted_receiver_to_be_writable() {
        // p: ref Outer, Outer.inner declared `box Inner` — writing inner.x goes through
        // ref ▷ box = box, which is not writable.
        let e = errors(
            "module m\ntype Inner { x: Int }\ntype Outer { inner: box Inner }\n\
             fn f(p: ref Outer) { p.inner.x = 1 }\n",
        );
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn recover_lifts_a_mutable_build_to_iso_here_already() {
        // The 7c half of criterion 3: the lift itself (env restriction lands in 7d).
        let c = check(
            "module m\nfn f() {\n\
             let xs: iso List[Int] = recover { let ys = [1]\nys.push(2)\nys }\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    // ----- Stage 7 phase 7d: consume flow analysis + the recover boundary ---------------

    #[test]
    fn use_after_consume_is_dl1602_with_the_consume_site() {
        let c = check(
            "module m\nfn f() {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             let ys: iso List[Int] = consume xs\n\
             let n = xs.len()\n}\n",
        );
        let d = c.diagnostics.iter().find(|d| d.code == "DL1602").expect("DL1602 expected");
        assert!(d.spans.iter().any(|s| s.secondary), "carries the consume site: {d:?}");
    }

    #[test]
    fn double_consume_is_dl1602() {
        let e = errors(
            "module m\nfn sink(v: iso List[Int]) { }\n\
             fn f() {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             sink(consume xs)\n\
             sink(consume xs)\n}\n",
        );
        assert!(e.contains(&"DL1602".to_string()), "{e:?}");
    }

    #[test]
    fn consume_in_one_branch_kills_the_binding_after_the_join() {
        // Possibly-consumed is dead: the unique reference may already have been transferred.
        let e = errors(
            "module m\nfn sink(v: iso List[Int]) { }\n\
             fn f(c: Bool) {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             if c { sink(consume xs) } else { }\n\
             let n = xs.len()\n}\n",
        );
        assert!(e.contains(&"DL1602".to_string()), "{e:?}");
    }

    #[test]
    fn consume_inside_a_loop_is_loop_carried_dead() {
        // By the second iteration the binding is already gone; the consume itself is refused.
        let e = errors(
            "module m\nfn sink(v: iso List[Int]) { }\n\
             fn f(c: Bool) {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             while c { sink(consume xs) }\n}\n",
        );
        assert!(e.contains(&"DL1602".to_string()), "{e:?}");
    }

    #[test]
    fn assignment_revives_a_consumed_var() {
        let c = check(
            "module m\nfn sink(v: iso List[Int]) { }\n\
             fn f() {\n\
             var xs: iso List[Int] = recover { [1] }\n\
             sink(consume xs)\n\
             xs = recover { [2] }\n\
             let n = xs.len()\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn use_before_consume_in_straight_line_is_clean() {
        let c = check(
            "module m\nfn sink(v: iso List[Int]) { }\n\
             fn f() {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             let n = xs.len()\n\
             sink(consume xs)\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn recover_referencing_an_outer_ref_is_dl1605() {
        // Criterion 3's reject half: mutable state must not leak into the re-proved region.
        let e = errors(
            "module m\nfn f() {\n\
             let outer: ref List[Int] = [1]\n\
             let xs: iso List[Int] = recover { outer }\n}\n",
        );
        assert!(e.contains(&"DL1605".to_string()), "{e:?}");
    }

    #[test]
    fn recover_referencing_an_outer_val_is_clean() {
        let c = check(
            "module m\nfn f(seed: Int) {\n\
             let xs: iso List[Int] = recover { [seed, seed + 1] }\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn recover_may_consume_an_outer_iso_in() {
        let c = check(
            "module m\nfn f() {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             let ys: iso List[Int] = recover { let zs = consume xs\nzs }\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_lambda_inside_recover_cannot_capture_an_outer_ref() {
        // The closure skip branch: the boundary applies through a capture, not just a
        // direct reference.
        let e = errors(
            "module m\nfn f() {\n\
             let outer: ref List[Int] = [1]\n\
             let g: val fn() -> Int = recover val { fn() -> Int { outer.len() } }\n}\n",
        );
        assert!(e.contains(&"DL1605".to_string()), "{e:?}");
    }

    #[test]
    fn consuming_a_capture_inside_a_closure_is_refused() {
        // A closure may run any number of times; each run would kill the same binding.
        let e = errors(
            "module m\nfn sink(v: iso List[Int]) { }\n\
             fn f() {\n\
             let xs: iso List[Int] = recover { [1] }\n\
             let g = fn() { sink(consume xs) }\n}\n",
        );
        assert!(e.contains(&"DL1602".to_string()), "{e:?}");
    }
}
