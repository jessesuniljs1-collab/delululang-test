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
pub mod deprecation;
pub mod prim_table;

/// The language edition this toolchain speaks (spec §2.2, invariant 43). Distinct from the
/// toolchain's own version: the edition moves only with strictly-additive minors, so a package
/// pinning an older one still means exactly what it said.
pub const LANGUAGE_EDITION: &str = "1.0";
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
pub use resolve::{ActorDef, DeclTable, FnSig, GKind, STD_ACTORS_SRC};
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

    // ----- prelude-builtin collisions (HARDENING_CAMPAIGN C11) ---------------

    #[test]
    fn a_function_named_after_a_prelude_builtin_is_refused_at_its_definition() {
        // Before: this checked CLEAN in isolation. The definition was accepted and every call
        // to it silently resolved to the builtin instead, so the only symptom was a type error
        // at a call site naming `Option` — a type the author never wrote — for a function
        // declared to return `Result`. Nothing anywhere named the collision.
        let c = check(
            "module m\ntype E = Bad\nfn parse_int(s: Str) -> Result[Int, E] { Ok(1) }\nfn use_it(s: Str) -> Int { match parse_int(s) { Ok(n) => n, Err(e) => 0 } }\n",
        );
        let d = c
            .diagnostics
            .iter()
            .find(|d| d.is_error() && d.code == "DL0302")
            .expect("expected DL0302 at the definition");
        assert!(d.message.contains("prelude builtin"), "the message must name the cause: {d:?}");
    }

    #[test]
    fn a_refused_builtin_name_does_not_panic_the_checker() {
        // `check_fn` looked its signature up with `.expect("fn in table")`. Refusing to register
        // a name therefore turned a bad program into a HOST PANIC, one skipped registration
        // away at all times. A host panic is never an acceptable answer to a bad program.
        for name in crate::check::PRELUDE_BUILTINS {
            let src = format!("module m\nfn {name}(x: Int) -> Int {{ x }}\n");
            let c = check(&src);
            assert!(c.has_errors(), "`{name}` must be refused as a declared name");
            assert!(
                c.diagnostics.iter().any(|d| d.code == "DL0302"),
                "`{name}` should be DL0302, got {:?}",
                c.diagnostics
            );
        }
    }

    #[test]
    fn the_builtin_list_matches_the_names_the_checker_actually_intercepts() {
        // The drift guard. `PRELUDE_BUILTINS` is the refusal list; `check_builtin_call` is the
        // interception list. If a builtin is added to one and not the other, a name becomes
        // silently un-callable again — which is the exact defect this pair of lists exists to
        // prevent. `None` is intercepted as a bare name rather than a call, so it is the one
        // entry that legitimately does not appear as a call arm.
        let src = include_str!("check.rs");
        let start = src.find("fn check_builtin_call").expect("check_builtin_call exists");
        let body = &src[start..];
        let end = body.find("\n            _ => None,").expect("the builtin match ends with a `_` arm");
        let mut intercepted: Vec<&str> = Vec::new();
        for line in body[..end].lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix('"') {
                if let Some(q) = rest.find('"') {
                    if rest[q..].trim_start_matches('"').trim_start().starts_with("=>") {
                        intercepted.push(&rest[..q]);
                    }
                }
            }
        }
        intercepted.sort_unstable();
        intercepted.dedup();
        let mut declared: Vec<&str> =
            crate::check::PRELUDE_BUILTINS.iter().copied().filter(|n| *n != "None").collect();
        declared.sort_unstable();
        assert_eq!(
            intercepted, declared,
            "PRELUDE_BUILTINS and the names `check_builtin_call` intercepts have diverged. \
             Any name in one and not the other is either un-refusable or un-callable."
        );
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

    // ----- C23: no declaration may shadow a builtin type or core effect ----
    //
    // Every one of these programs was ACCEPTED before the fix, and every one of them was inert:
    // the declaration had no effect anywhere, and no diagnostic was produced at any line. The
    // hazard is review, not execution — `type Cap = Int` in a source file invites a reader to
    // believe `Cap[FsRead]` is a user type, in a language whose premise is that authority can be
    // read off the source.

    #[test]
    fn every_builtin_type_name_is_refused_as_a_user_type() {
        for name in crate::check::PRELUDE_TYPES {
            let src = format!("module m\ntype {name} = Int\nfn g() -> Int {{ 1 }}\n");
            let e = errors(&src);
            assert!(
                e.contains(&"DL0302".to_string()),
                "`type {name} = Int` must be DL0302 — it would be silently inert, got {e:?}"
            );
        }
    }

    #[test]
    fn every_core_effect_name_is_refused_as_a_user_effect() {
        // The authority-bearing half: `effect Write` left every `! {Write}` row meaning the CORE
        // Write effect, so the author's "private" effect was the one that reaches the filesystem.
        for name in crate::check::CORE_EFFECT_NAMES {
            let src = format!("module m\neffect {name}\nfn g() -> Int {{ 1 }}\n");
            let e = errors(&src);
            assert!(
                e.contains(&"DL0302".to_string()),
                "`effect {name}` must be DL0302 — it would be silently inert, got {e:?}"
            );
        }
    }

    #[test]
    fn a_secret_alias_named_int_cannot_hide_inside_a_foreign_signature() {
        // The program that started C23: `type Int = Secret[Str]` then a foreign signature naming
        // `Int`. It checked CLEAN. Both engines lower foreign params by name, so no secret ever
        // actually crossed (`DL0602` still refused the value, verified) — but the file read as
        // though one could, and that is not a state this language may accept.
        let e = errors(
            "module m\ntype Int = Secret[Str]\nforeign \"c\" lib l { fn f(x: Int) -> Float }\n",
        );
        assert!(e.contains(&"DL0302".to_string()), "{e:?}");
    }

    #[test]
    fn an_alias_in_a_foreign_signature_names_its_target_and_offers_the_edit() {
        // C24. `type Meters = (Int)` is a genuine alias (the parenthesised form — see C28 for why
        // the bare `type Meters = Int` is a single-variant SUM instead). The fence still refuses it,
        // deliberately: both engines lower foreign signatures by type NAME through one shared path
        // that cannot see module aliases, so expanding here and not there is how ABI confusion
        // starts. What changed is that the refusal now names the target and hands over the edit,
        // instead of claiming `Meters` is not a marshallable type when `Meters` IS an Int.
        let c = check("module m\ntype Meters = (Int)\nforeign \"c\" lib l { fn f(x: Meters) -> Float }\n");
        let d = c
            .diagnostics
            .iter()
            .find(|d| d.code == "DL1301")
            .unwrap_or_else(|| panic!("expected DL1301, got {:?}", c.diagnostics));
        assert!(d.message.contains("alias for `Int`"), "must name the target: {}", d.message);
        let r = d.repairs.first().expect("an exact repair must be offered");
        assert_eq!(r.edits[0].insert, "Int", "the repair must write the underlying type");
        // Criterion 3 still holds: no repair may launder a secret across the FFI.
        for r in &d.repairs {
            assert!(!r.id.contains("expose"));
        }
    }

    #[test]
    fn an_alias_for_a_function_type_is_the_no_callbacks_rule_not_a_marshalling_complaint() {
        // C24. `type F = fn(Int) -> Int` used to report DL1301 ("F does not marshal") — true, and
        // useless. R-6a is about what the type MEANS, and this one means a re-entry point into
        // verified code, so it is DL1302 with the alias's expansion pointed at.
        let c = check("module m\ntype F = fn(Int) -> Int\nforeign \"c\" lib l { fn f(cb: F) -> Int }\n");
        let d = c
            .diagnostics
            .iter()
            .find(|d| d.code == "DL1302")
            .unwrap_or_else(|| panic!("expected DL1302, got {:?}", c.diagnostics));
        assert!(d.message.contains("R-6a"), "{}", d.message);
        assert!(d.repairs.is_empty(), "DL1302 is requires_human — no machine repairs");
    }

    #[test]
    fn a_cyclic_alias_cannot_hang_the_foreign_fence() {
        // The skip-branch case for C24's alias walk. `type A = A` is accepted by the checker (C16),
        // so an unbounded resolver would turn three lines of source into a hung compiler. Bounded
        // walk: this must return an error promptly, not spin.
        let e = errors("module m\ntype A = (A)\nforeign \"c\" lib l { fn f(x: A) -> Int }\n");
        assert!(e.contains(&"DL1301".to_string()), "{e:?}");
    }

    #[test]
    fn a_foreign_lib_or_actor_may_not_take_a_builtin_type_name() {
        // Both join the TYPE namespace and are matched AFTER the builtins, so the handle type
        // would be unnameable — a block whose type no signature could ever mention.
        let e = errors("module m\nforeign \"c\" lib Int { fn f(x: Int) -> Int }\n");
        assert!(e.contains(&"DL0302".to_string()), "foreign lib named Int: {e:?}");
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

    // ----- Stage 7 phase 7e: actor declarations (T-Actor/Ctor/Behavior/SyncMethod) ------

    const COUNTER: &str = "module m\n\
        actor Counter {\n\
        var count: Int\n\
        let label: Str\n\
        new(start: Int, label: Str) { self.count = start\nself.label = label }\n\
        be add(n: Int) { self.count = self.count + n }\n\
        fn doubled() -> Int { self.count * 2 }\n\
        }\n";

    #[test]
    fn a_well_formed_actor_checks_clean() {
        let c = check(COUNTER);
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        // Members register as Actor.member facts.
        assert!(c.result.facts.contains_key("Counter.add"), "behavior facts registered");
        assert!(c.result.facts.contains_key("Counter.new"), "ctor facts registered");
    }

    #[test]
    fn a_ref_behavior_param_is_dl1601() {
        let e = errors(
            "module m\nactor A { var xs: List[Int]\nnew() { self.xs = [] }\n\
             be feed(v: ref List[Int]) { } }\n",
        );
        assert!(e.contains(&"DL1601".to_string()), "{e:?}");
    }

    #[test]
    fn a_val_defaulted_behavior_param_is_clean() {
        // List[Int] defaults val — sendable; Int/Str default val — sendable.
        let c = check(
            "module m\nactor A { var total: Int\nnew() { self.total = 0 }\n\
             be feed(vs: List[Int], name: Str) { self.total = vs.len() } }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_pyobj_behavior_param_is_dl1601_with_the_pinning_explanation() {
        // Acceptance criterion 11: PyObj is actor-pinned (invariant 36 — CPython affinity).
        let c = check(
            "module m\nactor A { var n: Int\nnew() { self.n = 0 }\nbe feed(p: PyObj) { } }\n",
        );
        let d = c.diagnostics.iter().find(|d| d.code == "DL1601").expect("DL1601 expected");
        assert!(d.message.contains("pinned"), "the pinning explanation: {}", d.message);
        assert!(d.message.contains("CPython"), "names the affinity: {}", d.message);
    }

    #[test]
    fn undecidable_param_sendability_is_dl1601() {
        // A bare generic parameter: sendability cannot be established — never guessed.
        let e = errors(
            "module m\nactor Cell[T] { var n: Int\nnew() { self.n = 0 }\nbe put(v: T) { } }\n",
        );
        assert!(e.contains(&"DL1601".to_string()), "{e:?}");
    }

    #[test]
    fn a_written_val_generic_param_is_sendable() {
        // The Promise pattern (spec §8): `val T` is sendable by annotation.
        let c = check(
            "module m\nactor Cell[T] { var n: Int\nnew() { self.n = 0 }\nbe put(v: val T) { } }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_non_tag_rcap_on_an_actor_type_is_dl1607() {
        let c = check(&format!("{COUNTER}fn f(a: ref Counter) {{ }}\n"));
        let d = c.diagnostics.iter().find(|d| d.code == "DL1607").expect("DL1607 expected");
        assert!(!d.repairs.is_empty(), "carries the exact normalize repair");
    }

    #[test]
    fn a_bare_actor_type_is_implicitly_tag_and_clean() {
        let c = check(&format!("{COUNTER}fn f(a: Counter) {{ }}\n"));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn an_external_sync_call_on_an_actor_reference_is_dl1604() {
        // T-SyncMethod: outsiders hold tag; messages are the only cross-actor interface.
        let e = errors(&format!("{COUNTER}fn f(a: Counter) -> Int {{ a.doubled() }}\n"));
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn a_sync_call_from_self_is_clean_and_carries_the_row() {
        let c = check(
            "module m\nactor A {\n\
             var n: Int\n\
             new() { self.n = 0 }\n\
             be report(out: Cap[Console]) ! {Write} { out.println(str(self.describe())) }\n\
             fn describe() -> Int { self.n }\n\
             }\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn field_access_through_an_actor_reference_is_dl1604() {
        // An outsider's actor reference is tag: no field access at all.
        let e = errors(&format!("{COUNTER}fn f(a: Counter) -> Int {{ a.count }}\n"));
        assert!(e.contains(&"DL1604".to_string()), "{e:?}");
    }

    #[test]
    fn a_constructor_that_misses_a_field_is_refused() {
        let e = errors(
            "module m\nactor A { var x: Int\nlet y: Str\nnew(x: Int) { self.x = x } }\n",
        );
        assert!(e.contains(&"DL0405".to_string()), "{e:?}");
    }

    #[test]
    fn a_behavior_with_an_undeclared_effect_is_dl0501() {
        let e = errors(
            "module m\nactor A { var n: Int\nnew() { self.n = 0 }\n\
             be log(out: Cap[Console]) { out.println(\"x\") } }\n",
        );
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
    }

    #[test]
    fn stringifying_an_actor_reference_is_refused() {
        // Actor references are opaque identity (tag) — no str, no ==.
        let e = errors(&format!("{COUNTER}fn f(a: Counter) -> Str {{ str(a) }}\n"));
        assert!(e.contains(&"DL0604".to_string()), "{e:?}");
    }

    // ----- Stage 7 phase 7f: Async + T-Spawn/T-Send (causal rows, criteria 2 & 5) -------

    #[test]
    fn spawn_and_send_check_clean_under_async_and_carry_the_causal_edge() {
        let c = check(&format!(
            "{COUNTER}fn go() ! {{Async}} {{\n\
             let a = spawn Counter(0, \"c\")\n\
             a.add(5)\n}}\n"
        ));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        assert!(c.result.facts["go"].effects.contains(&Effect::Async), "Async in go's row");
        // The causal edge for `delulu why` (criterion 5): send sites are callees.
        assert!(c.result.facts["go"].callees.contains("Counter.add"), "{:?}", c.result.facts["go"].callees);
        assert!(c.result.facts["go"].callees.contains("Counter.new"));
    }

    #[test]
    fn spawn_without_async_in_the_row_is_dl0501() {
        let e = errors(&format!(
            "{COUNTER}fn go() {{\nlet a = spawn Counter(0, \"c\")\n}}\n"
        ));
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
    }

    #[test]
    fn criterion5_a_send_site_must_cover_the_behaviors_row() {
        // The behavior is !{Write}; a send from an !{Async}-only function is DL0501 — the
        // effect system and the actor system are ONE system (invariant 35).
        let src = "module m\n\
            actor Logger {\n\
            var out: Cap[Console]\n\
            new(out: Cap[Console]) { self.out = out }\n\
            be log(msg: Str) ! {Write} { self.out.println(msg) }\n\
            }\n";
        let e = errors(&format!(
            "{src}fn go(l: Logger) ! {{Async}} {{ l.log(\"x\") }}\n"
        ));
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
        let c = check(&format!(
            "{src}fn go(l: Logger) ! {{Async, Write}} {{ l.log(\"x\") }}\n"
        ));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_constructor_row_is_the_spawners_row() {
        // T-Spawn: row {Async} ∪ row(A.new) — a Read-ing constructor makes the spawner Read.
        let src = "module m\n\
            actor Loader {\n\
            var data: Str\n\
            new(fs: Cap[FsRead]) ! {Read} { self.data = \"\" }\n\
            }\n";
        let e = errors(&format!("{src}fn go(fs: Cap[FsRead]) ! {{Async}} {{ let a = spawn Loader(fs) }}\n"));
        assert!(e.contains(&"DL0501".to_string()), "{e:?}");
        let c = check(&format!(
            "{src}fn go(fs: Cap[FsRead]) ! {{Async, Read}} {{ let a = spawn Loader(fs) }}\n"
        ));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn criterion2_sending_a_ref_list_is_dl1601() {
        let src = "module m\n\
            actor Sink { var n: Int\nnew() { self.n = 0 }\nbe feed(vs: List[Int]) { } }\n";
        let e = errors(&format!(
            "{src}fn go(s: Sink) ! {{Async}} {{\n\
             let xs: ref List[Int] = [1]\n\
             s.feed(xs)\n}}\n"
        ));
        assert!(e.contains(&"DL1601".to_string()), "{e:?}");
    }

    #[test]
    fn criterion2_an_unconsumed_iso_gets_dl1601_with_the_exact_consume_repair() {
        let src = "module m\n\
            actor Sink { var n: Int\nnew() { self.n = 0 }\nbe take(vs: iso List[Int]) { } }\n";
        let c = check(&format!(
            "{src}fn go(s: Sink) ! {{Async}} {{\n\
             let xs: iso List[Int] = recover {{ [1] }}\n\
             s.take(xs)\n}}\n"
        ));
        let d = c.diagnostics.iter().find(|d| d.code == "DL1601").expect("DL1601 expected");
        let r = d.repairs.first().expect("the exact consume repair");
        assert_eq!(r.edits[0].insert, "consume ");
    }

    #[test]
    fn criterion2_a_consumed_iso_crosses_and_the_sender_loses_it() {
        let src = "module m\n\
            actor Sink { var n: Int\nnew() { self.n = 0 }\nbe take(vs: iso List[Int]) { } }\n";
        let c = check(&format!(
            "{src}fn go(s: Sink) ! {{Async}} {{\n\
             let xs: iso List[Int] = recover {{ [1] }}\n\
             s.take(consume xs)\n}}\n"
        ));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        // …and any later use is DL1602 (the binding is dead — the list is GONE).
        let e = errors(&format!(
            "{src}fn go(s: Sink) ! {{Async}} {{\n\
             let xs: iso List[Int] = recover {{ [1] }}\n\
             s.take(consume xs)\n\
             let n = xs.len()\n}}\n"
        ));
        assert!(e.contains(&"DL1602".to_string()), "{e:?}");
    }

    #[test]
    fn spawn_of_an_unknown_actor_is_dl0301() {
        let e = errors("module m\nfn go() ! {Async} { let a = spawn Ghost(1) }\n");
        assert!(e.contains(&"DL0301".to_string()), "{e:?}");
    }

    // ----- Stage 7 phase 7j: Promise[T] row plumbing (criterion 9, spec §8) -------------

    #[test]
    fn criterion9_an_effectful_callback_surfaces_in_the_callers_row() {
        // then's send site carries {Async} ∪ e — the R-4 law applied to a stdlib actor.
        let src = |row: &str| {
            format!(
                "module p\nfn go(out: Cap[Console]) ! {row} {{\n\
                 let pr = spawn Promise()\n\
                 pr.then(fn(v: val Str) ! {{Write}} {{ out.println(v) }})\n\
                 pr.fulfill(\"hi\")\n}}\n"
            )
        };
        let e = errors(&src("{Async}"));
        assert!(e.contains(&"DL0501".to_string()), "the callback's Write must surface: {e:?}");
        let c = check(&src("{Async, Write}"));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn criterion9_a_pure_callback_keeps_the_caller_at_async_only() {
        let c = check(
            "module p\nfn go() ! {Async} {\n\
             let pr = spawn Promise()\n\
             pr.then(fn(v: val Int) { })\n\
             pr.fulfill(7)\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn criterion9_fulfill_sites_charge_async_only() {
        // fulfill's `e` binds at THEN sites (no fulfill argument can constrain it); charging
        // fulfill callers for an unconstrainable tail would refuse every fulfill-only fn.
        let c = check(
            "module p\nfn feed() ! {Async} {\n\
             let pr = spawn Promise()\n\
             pr.fulfill(\"data\")\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn async_is_an_ordinary_row_citizen() {
        // Row polymorphism, manifests, `why` — Async obeys every row rule; here: it flows
        // through a row-polymorphic higher-order function like any effect.
        let c = check(&format!(
            "{COUNTER}fn apply[e](f: fn() -> Unit ! e) ! e {{ f() }}\n\
             fn go(a: Counter) ! {{Async}} {{ apply(fn() ! {{Async}} {{ a.add(1) }}) }}\n"
        ));
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    // ===== Stage 8, phase 8a: `test` blocks + assert/assert_eq (spec §2, invariant 41) =====

    #[test]
    fn a_test_block_type_checks_with_test_root_and_asserts() {
        let c = check(
            "module m\nfn double(n: Int) -> Int { n * 2 }\n\
             test \"double doubles\" {\n  assert_eq(double(21), 42)\n  assert(double(0) == 0)\n}\n",
        );
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_test_row_bounds_its_body_like_a_fn_row() {
        // Undeclared Write inside a test — DL0501 with the same widening repair as a fn,
        // and the message names the TEST honestly, not a phantom "function".
        let c = check(
            "module m\ntest \"writes\" {\n  let out = test_root.console()\n  out.println(\"hi\")\n}\n",
        );
        let d = c.diagnostics.iter().find(|d| d.code == "DL0501").expect("DL0501 expected");
        assert!(d.message.contains("test \"writes\""), "honest wording: {}", d.message);
        assert!(
            d.repairs.iter().any(|r| r.id == "add_effect_to_row" && r.authority_widening),
            "the agent-facing repair must be present"
        );
        // With the row declared, the same body checks clean.
        let ok = check(
            "module m\ntest \"writes\" ! {Write} {\n  let out = test_root.console()\n  out.println(\"hi\")\n}\n",
        );
        assert!(!ok.has_errors(), "{:?}", ok.diagnostics);
    }

    #[test]
    fn a_test_declaring_an_unperformed_effect_warns_dl0502() {
        let c = check("module m\ntest \"lazy\" ! {Write} {\n  assert(true)\n}\n");
        assert!(
            c.diagnostics.iter().any(|d| d.code == "DL0502" && d.message.contains("test \"lazy\"")),
            "{:?}",
            c.diagnostics
        );
    }

    #[test]
    fn assert_eq_on_a_secret_is_dl0605_r5_holds_in_tests() {
        // "Comparing secrets in tests is refused like everywhere else" (spec §2).
        let e = errors(
            "module m\nfn f(a: Secret[Str], b: Secret[Str]) { assert_eq(a, b) }\n",
        );
        assert!(e.contains(&"DL0605".to_string()), "{e:?}");
    }

    #[test]
    fn assert_takes_exactly_one_bool() {
        let e = errors("module m\nfn f() { assert(1) }\n");
        assert!(!e.is_empty(), "assert(Int) must be a type error");
        let e2 = errors("module m\nfn f() { assert(true, false) }\n");
        assert!(e2.contains(&"DL0401".to_string()), "{e2:?}");
    }

    #[test]
    fn asserts_are_pure_a_rowless_test_stays_pure() {
        // Assertions add no effects: a test with no row and only asserts checks clean (pure).
        let c = check("module m\ntest \"pure\" {\n  assert_eq(1 + 1, 2)\n}\n");
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
    }

    #[test]
    fn a_broken_test_body_still_surfaces_at_check() {
        // THE KITCHEN RULE'S skip branch: "compiled out of builds" must never become
        // "diagnosed never" — a type error inside a test block is a check error.
        let c = check("module m\ntest \"broken\" {\n  let x: Int = \"not an int\"\n  assert(x == 0)\n}\n");
        assert!(c.has_errors(), "a broken test body must fail `delulu check`");
    }

    #[test]
    fn a_test_body_must_produce_unit() {
        let c = check("module m\ntest \"leaks a value\" {\n  41 + 1\n}\n");
        assert!(c.has_errors(), "a non-Unit tail in a test body must be refused");
    }

    // ===== Stage 8, phase 8b: the catalog layer (spec §6.1, invariant 39) ==============

    /// Criterion 7's seed, end to end on a REAL diagnostic: the slang catalog renders the
    /// human header from the site's typed args; the en-US human path is byte-identical to
    /// the pre-catalog renderer; and the machine envelope's `message` stays the in-code
    /// en-US prose — the envelope API cannot even see a catalog (invariant 39 by
    /// construction).
    #[test]
    fn criterion7_seed_dl0501_slang_human_vs_frozen_machine_envelope() {
        use delulu_diag::{
            envelope, render_human, render_human_localized, Catalog, Palette, SourceMap,
        };
        let src = "module m\nfn greet(out: Cap[Console], n: Str) { out.println(n) }\n";
        let c = check(src);
        let d = c.diagnostics.iter().find(|d| d.code == "DL0501").expect("DL0501");
        assert!(
            d.args.iter().any(|(k, v)| k == "fn" && v.contains("greet")),
            "the site carries typed args: {:?}",
            d.args
        );

        let mut map = SourceMap::new();
        map.add_file("m.delulu", src);
        let slang = Catalog::delulu_slang();
        let human = render_human_localized(d, &map, &Palette::none(), Some(slang));
        assert!(human.contains("no cap 💀"), "slang voice renders: {human}");
        assert!(human.contains("function `greet`"), "{human}");
        assert!(human.contains("delulu explain E-DL0501"), "the explain pointer survives");

        // en-US human output: byte-identical with and without the catalog layer present.
        assert_eq!(
            render_human(d, &map),
            render_human_localized(d, &map, &Palette::none(), None),
            "the catalog layer must not move a single en-US byte"
        );

        // The machine envelope: en-US prose, catalog-blind.
        let env = envelope("check", &c.diagnostics, None, &map);
        let msg = env["diagnostics"][0]["message"].as_str().unwrap();
        assert!(msg.contains("performs effect"), "envelope message is en-US: {msg}");
        assert!(!msg.contains("no cap"), "slang can never reach the machine channel");
    }

    #[test]
    fn tests_leave_no_trace_in_authority_facts_invariant_38() {
        // The SAME module with and without a test block yields identical facts — tests are
        // compiled out, so authority reports cannot change (invariants 38 + 41).
        let with = check(
            "module m\nfn double(n: Int) -> Int { n * 2 }\n\
             test \"t\" ! {Write} {\n  let out = test_root.console()\n  out.println(str(double(2)))\n}\n",
        );
        let without = check("module m\nfn double(n: Int) -> Int { n * 2 }\n");
        assert!(!with.has_errors(), "{:?}", with.diagnostics);
        let mut a: Vec<_> = with.result.facts.keys().collect();
        let mut b: Vec<_> = without.result.facts.keys().collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "a test block must not add authority facts");
        assert_eq!(
            with.result.facts["double"].effects, without.result.facts["double"].effects,
            "existing facts must be untouched"
        );
    }
}
