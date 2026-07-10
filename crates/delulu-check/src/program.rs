//! Whole-program (multi-module) checking (Stage 2, §5). Stage 1's `check_source` handles one
//! module in isolation; this assembles a package's module graph into a single **global type
//! registry** (so a `TypeDefId` means the same thing across modules), gives each module a scope
//! of its own declarations plus the `pub` items it imports, checks each module against that
//! scope, and computes whole-program authority across the graph.
//!
//! The Stage-1 single-file path is untouched — this is additive.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::{json, Value};

use delulu_diag::Diagnostic;
use delulu_syntax::ast::*;

use crate::authority::ScopeInfo;
use crate::check::{check_module, FnFacts};
use crate::package::Package;
use crate::resolve::{
    classify_generics, ConstSig, DeclTable, ForeignDef, ForeignFnDef, FnSig, TypeDef, TypeDefKind,
};
use crate::ty::{Effect, ResourceKind, Type, TypeDefId};

/// The checked program: diagnostics plus whole-program facts keyed by qualified name (`mod::fn`).
pub struct Program {
    pub diagnostics: Vec<Diagnostic>,
    pub facts: HashMap<String, FnFacts>,
    pub fn_types: HashMap<String, Type>,
    /// Per module: which module owns each fn name visible in it (for cross-module reachability).
    pub call_owner: HashMap<String, HashMap<String, String>>,
    pub entry_module: Option<String>,
}

impl Program {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }
}

/// Check an already-loaded package. Load diagnostics (parse, DL0303/DL0304) are carried in.
pub fn check_program(pkg: &Package) -> Program {
    let mut diagnostics: Vec<Diagnostic> = pkg.diagnostics.iter().cloned().collect();

    // ----- 1. global type registry (prelude + every module's types) -------------------------
    let mut gtypes: Vec<TypeDef> = Vec::new();
    push_prelude(&mut gtypes);
    let prelude_ix: HashMap<String, TypeDefId> = [
        ("IoErr", TypeDefId(0)),
        ("NetErr", TypeDefId(1)),
        ("ForeignErr", TypeDefId(2)),
        ("PyErr", TypeDefId(3)),
    ]
    .into_iter()
    .map(|(n, i)| (n.to_string(), i))
    .collect();

    // module name -> its own type declarations (name, global id, is_pub)
    let mut owned_types: HashMap<String, Vec<(String, TypeDefId, bool)>> = HashMap::new();
    for unit in &pkg.modules {
        let mut seen = HashSet::new();
        let mut mine = Vec::new();
        for item in &unit.module.items {
            if let Item::Type(td) = item {
                if !seen.insert(td.name.name.clone()) {
                    diagnostics.push(dup(&unit.name, "type", &td.name.name, td.span));
                    continue;
                }
                let id = TypeDefId(gtypes.len() as u32);
                gtypes.push(TypeDef {
                    name: td.name.name.clone(),
                    generics: td.generics.iter().map(|g| g.name.clone()).collect(),
                    kind: build_kind(&td.kind),
                });
                mine.push((td.name.name.clone(), id, td.public));
            }
        }
        owned_types.insert(unit.name.clone(), mine);
    }

    // ----- 2. own fns / consts / effects per module -----------------------------------------
    let mut owned_fns: HashMap<String, Vec<FnSig>> = HashMap::new();
    let mut owned_consts: HashMap<String, Vec<ConstSig>> = HashMap::new();
    let mut owned_effects: HashMap<String, Vec<(String, bool)>> = HashMap::new();
    for unit in &pkg.modules {
        let mut fns = Vec::new();
        let mut consts = Vec::new();
        let mut effects = Vec::new();
        let mut names = HashSet::new();
        for item in &unit.module.items {
            match item {
                Item::Fn(f) => {
                    if !names.insert(f.name.name.clone()) {
                        diagnostics.push(dup(&unit.name, "value", &f.name.name, f.span));
                        continue;
                    }
                    let generics: Vec<String> = f.generics.iter().map(|g| g.name.clone()).collect();
                    let (gkinds, mut gd) = classify_generics(&generics, f);
                    diagnostics.append(&mut gd);
                    fns.push(FnSig {
                        name: f.name.name.clone(),
                        public: f.public,
                        generics,
                        gkinds,
                        params: f.params.clone(),
                        ret: f.ret.clone(),
                        row: f.row.clone(),
                    });
                }
                Item::Const(c) => {
                    if !names.insert(c.name.name.clone()) {
                        diagnostics.push(dup(&unit.name, "value", &c.name.name, c.span));
                        continue;
                    }
                    consts.push(ConstSig { name: c.name.name.clone(), ty: c.ty.clone() });
                }
                Item::Effect(e) => effects.push((e.name.name.clone(), e.public)),
                _ => {}
            }
        }
        owned_fns.insert(unit.name.clone(), fns);
        owned_consts.insert(unit.name.clone(), consts);
        owned_effects.insert(unit.name.clone(), effects);
    }

    // ----- 3. per-module scope (own + imported pub) and check --------------------------------
    let mut facts: HashMap<String, FnFacts> = HashMap::new();
    let mut fn_types: HashMap<String, Type> = HashMap::new();
    let mut call_owner: HashMap<String, HashMap<String, String>> = HashMap::new();

    for unit in &pkg.modules {
        let m = &unit.name;
        let mut type_ix = prelude_ix.clone();
        let mut fns: HashMap<String, FnSig> = HashMap::new();
        let mut consts: HashMap<String, ConstSig> = HashMap::new();
        let mut user_effects: HashSet<String> = HashSet::new();
        let mut owner: HashMap<String, String> = HashMap::new();
        let mut fn_order: Vec<String> = Vec::new();

        // Own declarations (all visible to their own module).
        for (name, id, _pub) in &owned_types[m] {
            type_ix.insert(name.clone(), *id);
        }
        for sig in &owned_fns[m] {
            fns.insert(sig.name.clone(), sig.clone());
            owner.insert(sig.name.clone(), m.clone());
            fn_order.push(sig.name.clone());
        }
        for c in &owned_consts[m] {
            consts.insert(c.name.clone(), c.clone());
        }
        for (e, _) in &owned_effects[m] {
            user_effects.insert(e.clone());
        }

        // Imported pub declarations from other modules in the package.
        for imp in &unit.module.imports {
            let target = imp.path.dotted();
            if !pkg.index.contains_key(&target) {
                continue; // DL0303 already reported at load
            }
            for (name, id, is_pub) in &owned_types[&target] {
                if *is_pub {
                    insert_unique_type(&mut type_ix, name, *id, m, &mut diagnostics, imp);
                }
            }
            for sig in &owned_fns[&target] {
                if sig.public {
                    if fns.contains_key(&sig.name) {
                        diagnostics.push(dup(m, "imported value", &sig.name, imp.span));
                    } else {
                        fns.insert(sig.name.clone(), sig.clone());
                        owner.insert(sig.name.clone(), target.clone());
                    }
                }
            }
            for c in &owned_consts[&target] {
                consts.entry(c.name.clone()).or_insert_with(|| c.clone());
            }
            for (e, is_pub) in &owned_effects[&target] {
                if *is_pub {
                    user_effects.insert(e.clone());
                }
            }
        }

        // Foreign blocks (Stage 4): register each module's own `foreign` lib types.
        let mut foreigns: HashMap<String, ForeignDef> = HashMap::new();
        for item in &unit.module.items {
            if let Item::Foreign(fd) = item {
                let fname = fd.name.name.clone();
                if type_ix.contains_key(&fname) || foreigns.contains_key(&fname) {
                    diagnostics.push(dup(m, "type", &fname, fd.span));
                    continue;
                }
                let ffns = fd
                    .fns
                    .iter()
                    .map(|f| ForeignFnDef {
                        name: f.name.name.clone(),
                        params: f.params.clone(),
                        ret: f.ret.clone(),
                        span: f.span,
                    })
                    .collect();
                foreigns.insert(fname.clone(), ForeignDef { abi: fd.abi.clone(), name: fname, fns: ffns });
            }
        }

        let table = DeclTable {
            types: gtypes.clone(),
            type_ix,
            user_effects,
            fns,
            consts,
            foreigns,
            fn_order,
        };
        let result = check_module(&unit.module, &table);
        diagnostics.extend(result.diags.iter().cloned());
        for (name, f) in result.facts {
            facts.insert(format!("{m}::{name}"), f);
        }
        for (name, t) in result.fn_types {
            fn_types.insert(format!("{m}::{name}"), t);
        }
        call_owner.insert(m.clone(), owner);
    }

    let entry_module = pkg.entry_module().map(|u| u.name.clone());
    Program { diagnostics, facts, fn_types, call_owner, entry_module }
}

/// The whole-program authority report (§10.5), computed across the module graph from `main`.
pub fn program_authority(program: &Program, program_name: &str, scopes: &ScopeInfo) -> Value {
    let mut effects: std::collections::BTreeSet<Effect> = Default::default();
    let mut cap_kinds: std::collections::BTreeSet<ResourceKind> = Default::default();
    let mut secrets: std::collections::BTreeSet<String> = Default::default();

    let reachable = reachable_qualified(program);
    for key in &reachable {
        if let Some(f) = program.facts.get(key) {
            effects.extend(f.effects.iter().cloned());
            cap_kinds.extend(f.cap_kinds.iter().cloned());
            secrets.extend(f.secret_names.iter().cloned());
        }
    }

    let mut capabilities = Vec::new();
    for k in &cap_kinds {
        if matches!(k, ResourceKind::Declassify | ResourceKind::PluginHost) {
            continue;
        }
        capabilities.push(json!({ "kind": k.name(), "scopes": scopes.for_kind(*k) }));
    }

    let mut pure_functions: Vec<String> =
        program.facts.iter().filter(|(_, f)| f.pure).map(|(n, _)| n.clone()).collect();
    pure_functions.sort();

    let effect_names: Vec<&str> = effects.iter().map(|e| e.name()).collect();

    json!({
        "program": program_name,
        "modules": program.call_owner.keys().cloned().collect::<std::collections::BTreeSet<_>>(),
        "effects": effect_names,
        "capabilities": capabilities,
        "secrets": secrets.into_iter().collect::<Vec<_>>(),
        "foreign_calls": [],
        "contained_plugins": [],
        "pure_functions": pure_functions,
    })
}

/// The effect names the program can perform, reachable from `main` (or all functions for a
/// library). This is the package's computed authority, checked against its manifest (DL1009).
pub fn program_effects(program: &Program) -> Vec<String> {
    let mut effects: std::collections::BTreeSet<String> = Default::default();
    for key in reachable_qualified(program) {
        if let Some(f) = program.facts.get(&key) {
            for e in &f.effects {
                effects.insert(e.name().to_string());
            }
        }
    }
    effects.into_iter().collect()
}

fn reachable_qualified(program: &Program) -> HashSet<String> {
    let mut seen = HashSet::new();
    let mut queue: VecDeque<(String, String)> = VecDeque::new();
    match &program.entry_module {
        Some(m) => queue.push_back((m.clone(), "main".to_string())),
        None => {
            // No entry: treat every function as reachable (library authority).
            return program.facts.keys().cloned().collect();
        }
    }
    while let Some((m, f)) = queue.pop_front() {
        let key = format!("{m}::{f}");
        if !seen.insert(key.clone()) {
            continue;
        }
        if let Some(facts) = program.facts.get(&key) {
            for callee in &facts.callees {
                if let Some(owner) = program.call_owner.get(&m).and_then(|o| o.get(callee)) {
                    queue.push_back((owner.clone(), callee.clone()));
                }
            }
        }
    }
    seen
}

// ----- helpers -------------------------------------------------------------

pub(crate) fn build_kind(kind: &TypeDeclKind) -> TypeDefKind {
    match kind {
        TypeDeclKind::Record(fields) => {
            TypeDefKind::Record(fields.iter().map(|f| (f.name.name.clone(), f.ty.clone())).collect())
        }
        TypeDeclKind::Sum(variants) => {
            TypeDefKind::Sum(variants.iter().map(|v| (v.name.name.clone(), v.fields.clone())).collect())
        }
        TypeDeclKind::Alias(t) => TypeDefKind::Alias(t.clone()),
    }
}

pub(crate) fn push_prelude(gtypes: &mut Vec<TypeDef>) {
    let mk = |name: &str, variants: &[(&str, &[&str])]| TypeDef {
        name: name.to_string(),
        generics: vec![],
        kind: TypeDefKind::Sum(
            variants
                .iter()
                .map(|(vn, fs)| {
                    let ftys = fs
                        .iter()
                        .map(|f| TypeExpr::Named {
                            path: Path { segs: vec![Ident { name: (*f).to_string(), span: dummy() }] },
                            args: vec![],
                            span: dummy(),
                        })
                        .collect();
                    ((*vn).to_string(), ftys)
                })
                .collect(),
        ),
    };
    gtypes.push(mk("IoErr", &[("NotFound", &[]), ("Denied", &[]), ("Other", &["Str"])]));
    gtypes.push(mk("NetErr", &[("Refused", &[]), ("Timeout", &[]), ("Other", &["Str"])]));
    // Stage-4 foreign stdlib surface (spec §8): ForeignErr (sum) + PyErr (record).
    gtypes.push(mk(
        "ForeignErr",
        &[("NotGranted", &[]), ("SymbolMissing", &["Str"]), ("BadReturn", &["Str"]), ("Unavailable", &["Str"])],
    ));
    let str_ty = || TypeExpr::Named {
        path: Path { segs: vec![Ident { name: "Str".to_string(), span: dummy() }] },
        args: vec![],
        span: dummy(),
    };
    gtypes.push(TypeDef {
        name: "PyErr".to_string(),
        generics: vec![],
        kind: TypeDefKind::Record(vec![("kind".to_string(), str_ty()), ("message".to_string(), str_ty())]),
    });
}

fn dummy() -> delulu_diag::Span {
    delulu_diag::Span::new(u32::MAX, 0, 0)
}

pub(crate) fn dup(module: &str, what: &str, name: &str, span: delulu_diag::Span) -> Diagnostic {
    Diagnostic::error("DL0302", format!("duplicate {what} `{name}` in module `{module}`"))
        .with_span(span, "already defined")
}

pub(crate) fn insert_unique_type(
    type_ix: &mut HashMap<String, TypeDefId>,
    name: &str,
    id: TypeDefId,
    module: &str,
    diags: &mut Vec<Diagnostic>,
    imp: &Import,
) {
    if type_ix.contains_key(name) {
        diags.push(dup(module, "imported type", name, imp.span));
    } else {
        type_ix.insert(name.to_string(), id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::load_package;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("delulu_prog_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        dir
    }
    fn write(dir: &Path, rel: &str, contents: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, contents).unwrap();
    }

    #[test]
    fn cross_module_call_type_checks_and_authority_crosses_the_graph() {
        let dir = scratch("xmod");
        write(&dir, "src/util.delulu", "module app.util\npub fn announce(out: Cap[Console]) ! {Write} { out.println(\"hi from util\") }\n");
        write(
            &dir,
            "src/main.delulu",
            "module app\nimport app.util\nfn main(root: Root) ! {Write} { let out = root.console()\n announce(out) }\n",
        );
        let pkg = load_package(&dir);
        assert!(!pkg.has_errors(), "load: {:?}", pkg.diagnostics);
        let program = check_program(&pkg);
        assert!(!program.has_errors(), "check: {:?}", program.diagnostics);

        let report = program_authority(&program, "app", &ScopeInfo::default());
        let effects: Vec<String> = report["effects"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        assert_eq!(effects, vec!["Write"], "Write must cross the module boundary from main → util.announce");
        // announce (in app.util) is reachable and effectful; it is NOT in pure_functions.
        let pure: Vec<String> = report["pure_functions"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        assert!(!pure.contains(&"app.util::announce".to_string()));
    }

    #[test]
    fn private_item_is_not_importable() {
        let dir = scratch("private");
        write(&dir, "src/util.delulu", "module app.util\nfn private_helper() -> Int { 0 }\n");
        write(&dir, "src/main.delulu", "module app\nimport app.util\nfn main(root: Root) -> Int { private_helper() }\n");
        let pkg = load_package(&dir);
        let program = check_program(&pkg);
        // private_helper is not `pub`, so it is not visible in `app` → unknown name (DL0301).
        assert!(program.diagnostics.iter().any(|d| d.code == "DL0301"), "{:?}", program.diagnostics);
    }

    #[test]
    fn pub_type_crosses_modules() {
        let dir = scratch("pubtype");
        write(&dir, "src/geom.delulu", "module geom\npub type Point { x: Int, y: Int }\npub fn origin() -> Point { Point { x: 0, y: 0 } }\n");
        write(&dir, "src/main.delulu", "module app\nimport geom\nfn get_x() -> Int { let p = origin()\n p.x }\n");
        let pkg = load_package(&dir);
        let program = check_program(&pkg);
        assert!(!program.has_errors(), "{:?}", program.diagnostics);
    }
}
