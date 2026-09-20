//! Cross-package dependency resolution and whole-workspace checking (Stage 2 §3–§4).
//!
//! [`resolve_workspace`] loads a root package plus its `path` dependencies recursively into ONE
//! source map (so cross-package spans stay coherent), deduplicating by package name and detecting
//! dependency cycles (DL1005), source conflicts (DL1008), and unpinned git deps (DL1007). Git
//! *fetching* is out of scope this round — a pinned git dep is reported as deferred, never faked.
//!
//! [`check_workspace`] assembles every package's modules under one global type registry and one
//! set of import rules (local module tree first, then declared dependency modules — ambiguity is
//! DL1006), producing a [`Program`] whose facts span the whole graph. On top of that:
//! [`check_self_authority`] (DL1009, per package) and [`check_pins`] (DL1001, per dependency edge)
//! turn "a dependency cannot silently gain an effect" into a compile error.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use delulu_diag::{Confidence, Diagnostic, Edit, Repair, SourceMap, Span};
use delulu_syntax::ast::*;

use crate::check::check_module;
use crate::manifest::{AuthoritySpec, DepSource, Manifest};
use crate::package::{load_package_into, ModuleUnit};
use crate::program::{build_kind, dup, insert_unique_type, push_prelude, Program};
use crate::resolve::{classify_generics, ConstSig, DeclTable, ForeignDef, ForeignFnDef, FnSig, TypeDef};
use crate::ty::{ResourceKind, TypeDefId};

/// One resolved package in the workspace.
pub struct ResolvedPackage {
    pub name: String,
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub is_root: bool,
    /// Package indices of this package's resolved `path` dependencies.
    pub dep_idxs: Vec<usize>,
    /// Path-relative source string for the lockfile (`path+<relpath-from-root>`).
    pub source_id: String,
}

/// One module in the workspace, tagged with its owning package.
pub struct WsModule {
    pub pkg: usize,
    pub unit: ModuleUnit,
}

/// The resolved workspace: every package and module, one shared source map, and resolution
/// diagnostics (DL1004/1005/1006/1007/1008 + any parse errors).
pub struct Workspace {
    pub root: usize,
    pub packages: Vec<ResolvedPackage>,
    pub index: HashMap<String, usize>,
    pub modules: Vec<WsModule>,
    pub source_map: SourceMap,
    pub diagnostics: Vec<Diagnostic>,
    /// Names of git dependencies whose resolution is deferred to a later stage (not faked).
    pub git_deferred: Vec<String>,
}

impl Workspace {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// The root package, if the graph resolved far enough to have one.
    ///
    /// `Option`, not a direct index, because `packages` is legitimately EMPTY whenever resolution
    /// failed before it could record the root: an unreadable `delulu.toml` (empty file, no
    /// `[package]` section, no `name` key, not TOML at all, or a dependency table this parser does
    /// not understand) yields a workspace with zero packages and zero modules. This used to be
    /// `&self.packages[self.root]`, and a caller that reached for the root's directory in order to
    /// print a *helpful note* about the empty module list panicked instead
    /// (`HARDENING_CAMPAIGN.md` C49). An empty `delulu.toml` is the most ordinary beginner mistake
    /// there is, and it crashed the build with a Rust backtrace.
    ///
    /// Returning `Option` rather than fixing that one call site is deliberate: the next caller
    /// cannot reintroduce the crash without the compiler making them think about the empty case.
    pub fn root_pkg(&self) -> Option<&ResolvedPackage> {
        self.packages.get(self.root)
    }

    /// The root package's entry module (the one declaring `fn main`), if any.
    pub fn root_entry_module(&self) -> Option<String> {
        self.modules
            .iter()
            .filter(|m| m.pkg == self.root)
            .find(|m| m.unit.module.items.iter().any(|it| matches!(it, Item::Fn(f) if f.name.name == "main")))
            .map(|m| m.unit.name.clone())
    }
}

/// Resolve a workspace rooted at `root_dir`. Never panics; problems land in `diagnostics`.
pub fn resolve_workspace(root_dir: impl AsRef<Path>) -> Workspace {
    let root_dir = root_dir.as_ref().to_path_buf();
    let mut ws = Workspace {
        root: 0,
        packages: Vec::new(),
        index: HashMap::new(),
        modules: Vec::new(),
        source_map: SourceMap::new(),
        diagnostics: Vec::new(),
        git_deferred: Vec::new(),
    };
    let mut stack: Vec<String> = Vec::new();
    let root_canon = canonical(&root_dir);
    let root_idx = resolve_dir(&mut ws, &root_dir, &root_canon, true, &mut stack);
    ws.root = root_idx.unwrap_or(0);
    ws
}

fn canonical(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

fn resolve_dir(
    ws: &mut Workspace,
    dir: &Path,
    root_dir: &Path,
    is_root: bool,
    stack: &mut Vec<String>,
) -> Option<usize> {
    let manifest_path = dir.join("delulu.toml");
    let raw = match std::fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            let f = ws.source_map.add_file(display_manifest(dir, is_root), String::new());
            ws.diagnostics.push(
                Diagnostic::error("DL1004", format!("cannot read manifest `{}`: {e}", manifest_path.display()))
                    .with_bare_span(Span::new(f, 0, 0)),
            );
            return None;
        }
    };
    let file = ws.source_map.add_file(display_manifest(dir, is_root), raw.clone());
    let (manifest, mdiags) = Manifest::parse(&raw, file);
    ws.diagnostics.extend(mdiags);
    let manifest = manifest?;
    let name = manifest.name.clone();

    // The language edition (spec §2.2): a package written for a NEWER edition than this toolchain
    // speaks is refused before anything is compiled. Checked here so it applies to every package
    // in the graph, not only the root — a dependency from the future is exactly as unreadable.
    if let Some(d) = manifest.edition_check(crate::LANGUAGE_EDITION) {
        ws.diagnostics.push(d);
    }

    // Cycle: a dependency points back at a package currently being resolved (an ancestor).
    if stack.contains(&name) {
        ws.diagnostics.push(
            Diagnostic::error("DL1005", format!("dependency cycle through package `{name}`"))
                .with_span(manifest.dep_span(&name), "this dependency closes a package cycle"),
        );
        return ws.index.get(&name).copied();
    }

    // Dedupe (diamond): same name already resolved. Two different source dirs = DL1008.
    let dir_canon = canonical(dir);
    if let Some(&existing) = ws.index.get(&name) {
        if canonical(&ws.packages[existing].dir) != dir_canon {
            ws.diagnostics.push(
                Diagnostic::error(
                    "DL1008",
                    format!("package `{name}` is required from two different sources — versions/authority could differ invisibly"),
                )
                .with_span(Span::new(file, 0, 0), "second source of this package name"),
            );
        }
        return Some(existing);
    }

    // Register this package and load its modules into the shared source map.
    let prefix = if is_root { String::new() } else { format!("dep:{name}/") };
    let loaded = load_package_into(dir, &mut ws.source_map, &prefix);
    ws.diagnostics.extend(loaded.diagnostics);
    let idx = ws.packages.len();
    let source_id = source_id_for(root_dir, &dir_canon, is_root);
    ws.packages.push(ResolvedPackage {
        name: name.clone(),
        dir: dir.to_path_buf(),
        manifest: manifest.clone(),
        is_root,
        dep_idxs: Vec::new(),
        source_id,
    });
    ws.index.insert(name.clone(), idx);
    for unit in loaded.modules {
        ws.modules.push(WsModule { pkg: idx, unit });
    }

    // Recurse into dependencies.
    stack.push(name.clone());
    let mut dep_idxs = Vec::new();
    for dep in &manifest.dependencies {
        match &dep.source {
            DepSource::Path(rel) => {
                let dep_dir = dir.join(rel);
                if !dep_dir.join("delulu.toml").exists() {
                    ws.diagnostics.push(
                        Diagnostic::error("DL1004", format!("path dependency `{}` not found at `{}`", dep.name, dep_dir.display()))
                            .with_span(manifest.dep_span(&dep.name), "no package at this path"),
                    );
                    continue;
                }
                if let Some(child) = resolve_dir(ws, &dep_dir, root_dir, false, stack) {
                    if !dep_idxs.contains(&child) {
                        dep_idxs.push(child);
                    }
                }
            }
            DepSource::Git { rev, tag, .. } => {
                if rev.is_none() && tag.is_none() {
                    ws.diagnostics.push(
                        Diagnostic::error("DL1007", format!("git dependency `{}` must pin `rev` or `tag` (floating branches are forbidden)", dep.name))
                            .with_span(manifest.dep_span(&dep.name), "add `rev = \"...\"` or `tag = \"...\"`"),
                    );
                } else {
                    ws.git_deferred.push(dep.name.clone());
                }
            }
        }
    }
    stack.pop();
    ws.packages[idx].dep_idxs = dep_idxs;
    Some(idx)
}

fn display_manifest(dir: &Path, is_root: bool) -> String {
    if is_root {
        "delulu.toml".to_string()
    } else {
        let name = dir.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        format!("dep:{name}/delulu.toml")
    }
}

fn source_id_for(root_dir: &Path, dir_canon: &Path, is_root: bool) -> String {
    if is_root {
        return "path+.".to_string();
    }
    let rel = pathdiff(root_dir, dir_canon);
    format!("path+{}", rel.replace('\\', "/"))
}

/// A best-effort relative path from `base` to `target` (lexical, `/`-normalized).
fn pathdiff(base: &Path, target: &Path) -> String {
    let b: Vec<_> = base.components().collect();
    let t: Vec<_> = target.components().collect();
    let mut i = 0;
    while i < b.len() && i < t.len() && b[i] == t[i] {
        i += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in i..b.len() {
        parts.push("..".to_string());
    }
    for c in &t[i..] {
        parts.push(c.as_os_str().to_string_lossy().to_string());
    }
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

// ===== whole-workspace checking =================================================================

/// Check every module in the workspace under one global type registry, resolving imports across
/// package boundaries (DL1006 on local/dep ambiguity, DL0303 when unknown). Returns a [`Program`]
/// whose facts/call_owner span the whole graph; `entry_module` is the root's `main` module.
pub fn check_workspace(ws: &Workspace) -> Program {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let n = ws.modules.len();

    // 1. Global type registry.
    let mut gtypes: Vec<TypeDef> = Vec::new();
    push_prelude(&mut gtypes);
    // Derived from `push_prelude` rather than listed. This line used to name TWO of the seven
    // prelude types, so on this path `PyErr`, `ForeignErr`, `Limits`, `Grant` and `PluginErr` were
    // absent from every module's table — and `Grant` turned that into a checker panic as soon as P2
    // gave a program a reason to write one. See `program::prelude_index`.
    let prelude_ix: HashMap<String, TypeDefId> = crate::program::prelude_index(&gtypes);

    let mut owned_types: Vec<Vec<(String, TypeDefId, bool)>> = vec![Vec::new(); n];
    for (owned, module) in owned_types.iter_mut().zip(&ws.modules) {
        let unit = &module.unit;
        let mut seen = HashSet::new();
        for item in &unit.module.items {
            if let Item::Type(td) = item {
                if !seen.insert(td.name.name.clone()) {
                    diagnostics.push(dup(&unit.name, "type", &td.name.name, td.span));
                    continue;
                }
                // C23: third and last construction path (the dependency graph). All three refuse
                // identically — a rule that holds on two paths out of three holds nowhere.
                if let Some(d) = crate::resolve::shadows_a_builtin_type("type", &td.name) {
                    diagnostics.push(d);
                    continue;
                }
                let id = TypeDefId(gtypes.len() as u32);
                gtypes.push(TypeDef {
                    name: td.name.name.clone(),
                    generics: td.generics.iter().map(|g| g.name.clone()).collect(),
                    kind: build_kind(&td.kind),
                });
                owned.push((td.name.name.clone(), id, td.public));
            }
        }
    }

    // 2. Own fns/consts/effects per module.
    let mut owned_fns: Vec<Vec<FnSig>> = vec![Vec::new(); n];
    let mut owned_consts: Vec<Vec<ConstSig>> = vec![Vec::new(); n];
    let mut owned_effects: Vec<Vec<(String, bool)>> = vec![Vec::new(); n];
    for gi in 0..n {
        let unit = &ws.modules[gi].unit;
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
                    owned_fns[gi].push(FnSig {
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
                    owned_consts[gi].push(ConstSig { name: c.name.name.clone(), ty: c.ty.clone() });
                }
                Item::Effect(e) => match crate::resolve::shadows_a_core_effect(&e.name) {
                    Some(d) => diagnostics.push(d), // C23, authority-bearing (see program.rs)
                    None => owned_effects[gi].push((e.name.name.clone(), e.public)),
                },
                _ => {}
            }
        }
    }

    // 3. Module resolution index: (pkg, dotted-name) -> global module index.
    let mut gi_by_pkg_name: HashMap<(usize, String), usize> = HashMap::new();
    for gi in 0..n {
        gi_by_pkg_name.insert((ws.modules[gi].pkg, ws.modules[gi].unit.name.clone()), gi);
    }

    // 4. Per-module scope (own + imported pub, local-then-dependency) and check.
    let mut facts = HashMap::new();
    let mut fn_types = HashMap::new();
    let mut call_owner: HashMap<String, HashMap<String, String>> = HashMap::new();
    // Modules that have already had a re-export cycle reported (DL1005), to avoid duplicates.
    let mut reported_cycles: HashSet<usize> = HashSet::new();

    // Which module owns each source file, so a diagnostic that lands outside the module being
    // checked can be attributed to the module whose text it actually is (C58).
    let file_owner: HashMap<delulu_diag::FileId, usize> =
        (0..n).map(|gi| (ws.modules[gi].unit.file, gi)).collect();

    for gi in 0..n {
        let unit = &ws.modules[gi].unit;
        let p = ws.modules[gi].pkg;
        let m = &unit.name;
        let mut type_ix = prelude_ix.clone();
        let mut fns: HashMap<String, FnSig> = HashMap::new();
        let mut consts: HashMap<String, ConstSig> = HashMap::new();
        let mut user_effects: HashSet<String> = HashSet::new();
        let mut owner: HashMap<String, String> = HashMap::new();
        let mut fn_order: Vec<String> = Vec::new();

        for (name, id, _) in &owned_types[gi] {
            type_ix.insert(name.clone(), *id);
        }
        for sig in &owned_fns[gi] {
            fns.insert(sig.name.clone(), sig.clone());
            owner.insert(sig.name.clone(), m.clone());
            fn_order.push(sig.name.clone());
        }
        for c in &owned_consts[gi] {
            consts.insert(c.name.clone(), c.clone());
        }
        for (e, _) in &owned_effects[gi] {
            user_effects.insert(e.clone());
        }

        for imp in &unit.module.imports {
            let target = imp.path.dotted();
            let local = gi_by_pkg_name.get(&(p, target.clone())).copied();
            let mut dep_hits: Vec<usize> = Vec::new();
            for &dp in &ws.packages[p].dep_idxs {
                if let Some(&g) = gi_by_pkg_name.get(&(dp, target.clone())) {
                    dep_hits.push(g);
                }
            }
            let mut candidates: Vec<usize> = Vec::new();
            if let Some(g) = local {
                candidates.push(g);
            }
            candidates.extend(dep_hits);
            if candidates.is_empty() {
                diagnostics.push(
                    Diagnostic::error("DL0303", format!("unknown module `{target}` imported by `{m}`"))
                        .with_span(imp.span, "no such module in this package or its declared dependencies"),
                );
                continue;
            }
            if candidates.len() > 1 {
                diagnostics.push(
                    Diagnostic::error("DL1006", format!("import `{target}` is ambiguous between a local module and a dependency"))
                        .with_span(imp.span, "rename the local module or alias the dependency — imports are never resolved by guessing"),
                );
            }
            let tgi = candidates[0];
            // Bring in the target's EXPORTS: its own pub items plus anything it re-exported via
            // `pub import` (§2). Re-export cycles are DL1005.
            let ex = compute_exports_gi(
                tgi,
                ws,
                &owned_types,
                &owned_fns,
                &owned_consts,
                &owned_effects,
                &gi_by_pkg_name,
                &mut HashSet::new(),
                &mut reported_cycles,
                &mut diagnostics,
            );
            for (name, id) in &ex.types {
                insert_unique_type(&mut type_ix, name, *id, m, &mut diagnostics, imp);
            }
            for (sig, owner_mod) in &ex.fns {
                if fns.contains_key(&sig.name) {
                    diagnostics.push(dup(m, "imported value", &sig.name, imp.span));
                } else {
                    fns.insert(sig.name.clone(), sig.clone());
                    owner.insert(sig.name.clone(), owner_mod.clone());
                }
            }
            for c in &ex.consts {
                consts.entry(c.name.clone()).or_insert_with(|| c.clone());
            }
            for e in &ex.effects {
                user_effects.insert(e.clone());
            }
        }

        // Foreign blocks (Stage 4): register each module's own `foreign` lib types.
        let mut foreigns: std::collections::HashMap<String, ForeignDef> = std::collections::HashMap::new();
        for item in &unit.module.items {
            if let delulu_syntax::ast::Item::Foreign(fd) = item {
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

        let table = DeclTable { types: gtypes.clone(), type_ix, user_effects, fns, consts, foreigns, actors: HashMap::new(), fn_order };
        let result = check_module(&unit.module, &table);
        diagnostics.extend(reframe_foreign_scope_errors(&result.diags, gi, ws, &owned_types, &file_owner));
        for (name, f) in result.facts {
            facts.insert(format!("{m}::{name}"), f);
        }
        for (name, t) in result.fn_types {
            fn_types.insert(format!("{m}::{name}"), t);
        }
        call_owner.insert(m.clone(), owner);
    }

    // One fact reported once. A type in a diamond's shared dependency is lowered afresh for every
    // path that reaches it, so the SAME failure arrived up to four times (C58: 25 errors at 14
    // distinct locations). Deduplication is by code + message + primary span, which is exactly
    // "this is the same sentence about the same place" and never merges two different facts.
    let mut seen: HashSet<(&'static str, String, Option<Span>)> = HashSet::new();
    diagnostics.retain(|d| seen.insert((d.code, d.message.clone(), d.primary_span())));

    let entry_module = ws.root_entry_module();
    let type_names = crate::ty::TypeNameList(gtypes.iter().map(|d| d.name.clone()).collect());
    Program { diagnostics, facts, fn_types, call_owner, entry_module, type_names }
}

/// Re-frame diagnostics that landed in **another module's file** (C58).
///
/// A `pub fn`'s signature is lowered in the scope of whoever IMPORTS it, not where it was written.
/// So when an importer cannot see a type that signature names, `lower_type` reports `unknown type`
/// at the span where the text lives — inside the dependency's own source, which checked clean on its
/// own, and in a file where that type IS in scope. The rule is right (the importer genuinely cannot
/// use that function); the report blamed the wrong line, in the wrong package, for a reason that
/// reads as false.
///
/// Those reports are collapsed into ONE diagnostic per source module, at the import that brought the
/// signature in, naming where the type is declared and both ends it can be fixed from.
///
/// **Only the case it can identify is rewritten.** A foreign-file diagnostic that is not a tagged
/// `DL0301` is passed through untouched — dropping a diagnostic nobody classified is the fail-open
/// branch this project refuses everywhere else.
fn reframe_foreign_scope_errors(
    diags: &[Diagnostic],
    gi: usize,
    ws: &Workspace,
    owned_types: &[Vec<(String, TypeDefId, bool)>],
    file_owner: &HashMap<delulu_diag::FileId, usize>,
) -> Vec<Diagnostic> {
    let unit = &ws.modules[gi].unit;
    let mut out: Vec<Diagnostic> = Vec::new();
    // The type names this module could not see, first-seen order (determinism), paired with the
    // modules whose signatures wanted them.
    let mut missing: Vec<String> = Vec::new();
    let mut sources: Vec<String> = Vec::new();

    // Where a type is actually declared. "unknown type" is false here in the way that matters —
    // the type exists — so saying where turns a dead end into one edit.
    let home_of = |name: &str| -> Option<String> {
        (0..ws.modules.len())
            .find(|&mi| owned_types[mi].iter().any(|(tn, _, _)| tn == name))
            .map(|mi| ws.modules[mi].unit.name.clone())
    };

    for d in diags {
        let elsewhere = d.primary_span().map(|s| s.file != unit.file).unwrap_or(false);
        let named = d.args.iter().find(|(k, _)| k == "type").map(|(_, v)| v.clone());
        match (d.code, named) {
            // Reported inside SOMEBODY ELSE'S file: that file checked clean on its own and the type
            // is in scope there. Collapsed into one diagnostic at the import, below.
            ("DL0301", Some(name)) if elsewhere => {
                if !missing.contains(&name) {
                    missing.push(name);
                }
                if let Some(src) = d.primary_span().and_then(|s| file_owner.get(&s.file)) {
                    let m = ws.modules[*src].unit.name.clone();
                    if !sources.contains(&m) {
                        sources.push(m);
                    }
                }
            }
            // In this module's own file the span is already right, so the error STAYS THERE and only
            // gains the answer. Rewriting it to point at an import would trade a precise location
            // for a general one — and a plain typo (`Smaple`) has no home, so it is left exactly as
            // it was rather than decorated with a guess.
            ("DL0301", Some(name)) => {
                let mut d = d.clone();
                if let Some(home) = home_of(&name) {
                    d.message = format!(
                        "{} — `{name}` is declared in `{home}`, which is not in scope here; add `import {home}`, or have the module you import re-export it with `pub import {home}`",
                        d.message
                    );
                }
                out.push(d);
            }
            _ => out.push(d.clone()),
        }
    }

    if !missing.is_empty() {
        let list = missing.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ");
        let from = sources.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ");
        let homes: Vec<String> = {
            let mut h: Vec<String> = Vec::new();
            for n in &missing {
                if let Some(x) = home_of(n) {
                    if !h.contains(&x) {
                        h.push(x);
                    }
                }
            }
            h
        };
        let advice = match homes.first() {
            Some(home) => format!(
                "an imported signature is resolved in YOUR scope, so a name the dependency can see is not automatically one you can see. Add `import {home}` here, or re-export it with `pub import {home}` in the module you import"
            ),
            None => "no module in this workspace declares them".to_string(),
        };
        let where_ = match homes.len() {
            0 => String::new(),
            1 => format!(", declared in `{}`", homes[0]),
            _ => format!(", declared in {}", homes.iter().map(|h| format!("`{h}`")).collect::<Vec<_>>().join(" and ")),
        };
        let mut d = Diagnostic::error(
            "DL0301",
            format!("`{}` cannot see {list}{where_} — named by signatures it imports from {from}: {advice}", unit.name),
        );
        for name in &missing {
            d = d.with_arg("type", name.clone());
        }
        // Point at the import that brought a failing signature in. If the module reached it some
        // other way, fall back to the module header rather than inventing a location.
        let at = sources
            .iter()
            .find_map(|src| unit.module.imports.iter().find(|i| i.path.dotted() == *src))
            .map(|i| (i.span, "this import does not bring those types with it"))
            .unwrap_or((unit.module.name.span(), "this module"));
        out.push(d.with_span(at.0, at.1));
    }

    out
}

// ===== per-package authority ====================================================================

/// The computed authority of one package (§4.1): effects, capability kinds, and secret names
/// reachable from its `pub` functions (plus `main` for a bin).
#[derive(Clone, Debug, Default)]
pub struct PackageAuthority {
    pub effects: BTreeSet<String>,
    pub cap_kinds: BTreeSet<String>,
    pub secrets: BTreeSet<String>,
}

/// Seed keys (`mod::fn`) for a package's public surface (+ `main` if present).
fn public_seeds(ws: &Workspace, pkg_idx: usize) -> Vec<String> {
    let mut seeds = Vec::new();
    for wm in ws.modules.iter().filter(|m| m.pkg == pkg_idx) {
        for item in &wm.unit.module.items {
            if let Item::Fn(f) = item {
                if f.public || f.name.name == "main" {
                    seeds.push(format!("{}::{}", wm.unit.name, f.name.name));
                }
            }
        }
    }
    seeds
}

fn reachable_from_seeds(program: &Program, seeds: &[String]) -> HashSet<String> {
    let mut seen = HashSet::new();
    let mut queue: VecDeque<String> = seeds.iter().cloned().collect();
    while let Some(key) = queue.pop_front() {
        if !seen.insert(key.clone()) {
            continue;
        }
        let Some((m, _f)) = key.split_once("::") else { continue };
        if let Some(facts) = program.facts.get(&key) {
            for callee in &facts.callees {
                if let Some(owner) = program.call_owner.get(m).and_then(|o| o.get(callee)) {
                    queue.push_back(format!("{owner}::{callee}"));
                }
            }
        }
    }
    seen
}

/// Compute a package's authority over the whole-workspace program.
pub fn package_authority(ws: &Workspace, program: &Program, pkg_idx: usize) -> PackageAuthority {
    let mut auth = PackageAuthority::default();
    for key in reachable_from_seeds(program, &public_seeds(ws, pkg_idx)) {
        if let Some(f) = program.facts.get(&key) {
            for e in &f.effects {
                auth.effects.insert(e.name().to_string());
            }
            for k in &f.cap_kinds {
                if !matches!(k, ResourceKind::PluginHost) {
                    auth.cap_kinds.insert(k.name().to_string());
                }
            }
            auth.secrets.extend(f.secret_names.iter().cloned());
        }
    }
    auth
}

// ===== DL1009: package self-authority check =====================================================

/// Each package's computed authority must be within its own manifest's `[authority]` declaration —
/// its `effects`, and (campaign C19, ruling D34) the `secrets` it reads.
///
/// The secret half was missing, and its absence was the same shape as C18 one layer down:
/// `root.secret("X")` contributes **no effect and no capability kind**, only a name, so a package
/// could read any secret it liked while its manifest declared none, and the effects-only check saw
/// nothing wrong. The manifest is supposed to be a *ceiling* — Constitution invariant 10 counts
/// scopes, not just effect kinds — so a dimension the ceiling does not mention is a dimension the
/// ceiling does not bound. `package_authority` already computed `auth.secrets`; nothing consumed it.
pub fn check_self_authority(ws: &Workspace, program: &Program) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (idx, pkg) in ws.packages.iter().enumerate() {
        let auth = package_authority(ws, program, idx);
        let declared: HashSet<&str> = pkg.manifest.authority.effects.iter().map(|s| s.as_str()).collect();
        for e in &auth.effects {
            if !declared.contains(e.as_str()) {
                diags.push(
                    Diagnostic::error(
                        "DL1009",
                        format!("package `{}` performs effect `{e}` not permitted by its authority manifest", pkg.name),
                    )
                    .with_span(pkg.manifest.effects_span(), format!("add `{e}` to `[authority] effects` in delulu.toml")),
                );
            }
        }
        let declared_secrets: HashSet<&str> =
            pkg.manifest.authority.secrets.iter().map(|s| s.as_str()).collect();
        for s in &auth.secrets {
            if !declared_secrets.contains(s.as_str()) {
                diags.push(
                    Diagnostic::error(
                        "DL1009",
                        format!(
                            "package `{}` reads secret `{s}` not permitted by its authority manifest",
                            pkg.name
                        ),
                    )
                    .with_span(
                        pkg.manifest.secrets_span(),
                        format!("add `\"{s}\"` to `[authority] secrets` in delulu.toml"),
                    ),
                );
            }
        }
    }
    diags
}

// ===== pub import re-export resolution (§2) =====================================================

/// The items a module exports to importers: its own `pub` items plus everything it re-exports via
/// `pub import` (transitively). `fns` carry their true owner module so cross-module calls resolve
/// to where the function actually lives.
struct GiExports {
    types: Vec<(String, TypeDefId)>,
    fns: Vec<(FnSig, String)>,
    consts: Vec<ConstSig>,
    effects: Vec<String>,
}

/// Resolve a module name referenced from package `from_pkg` to a global module index:
/// local module first, then declared dependencies (the same order as the main import loop, but
/// without re-reporting DL1006 ambiguity — that is the importing site's job).
fn resolve_target_gi(
    from_pkg: usize,
    target: &str,
    ws: &Workspace,
    gi_by_pkg_name: &HashMap<(usize, String), usize>,
) -> Option<usize> {
    if let Some(&g) = gi_by_pkg_name.get(&(from_pkg, target.to_string())) {
        return Some(g);
    }
    for &dp in &ws.packages[from_pkg].dep_idxs {
        if let Some(&g) = gi_by_pkg_name.get(&(dp, target.to_string())) {
            return Some(g);
        }
    }
    None
}

/// Compute a module's exports, following `pub import` edges with cycle detection (DL1005).
#[allow(clippy::too_many_arguments)]
fn compute_exports_gi(
    gi: usize,
    ws: &Workspace,
    owned_types: &[Vec<(String, TypeDefId, bool)>],
    owned_fns: &[Vec<FnSig>],
    owned_consts: &[Vec<ConstSig>],
    owned_effects: &[Vec<(String, bool)>],
    gi_by_pkg_name: &HashMap<(usize, String), usize>,
    visiting: &mut HashSet<usize>,
    reported: &mut HashSet<usize>,
    diags: &mut Vec<Diagnostic>,
) -> GiExports {
    let owner_name = ws.modules[gi].unit.name.clone();
    let mut ex = GiExports { types: Vec::new(), fns: Vec::new(), consts: Vec::new(), effects: Vec::new() };
    for (name, id, is_pub) in &owned_types[gi] {
        if *is_pub {
            ex.types.push((name.clone(), *id));
        }
    }
    for sig in &owned_fns[gi] {
        if sig.public {
            ex.fns.push((sig.clone(), owner_name.clone()));
        }
    }
    for c in &owned_consts[gi] {
        ex.consts.push(c.clone());
    }
    for (e, is_pub) in &owned_effects[gi] {
        if *is_pub {
            ex.effects.push(e.clone());
        }
    }

    visiting.insert(gi);
    let p = ws.modules[gi].pkg;
    for imp in &ws.modules[gi].unit.module.imports {
        if !imp.public {
            continue; // only `pub import` re-exports
        }
        let target = imp.path.dotted();
        if let Some(tgi) = resolve_target_gi(p, &target, ws, gi_by_pkg_name) {
            if visiting.contains(&tgi) {
                if reported.insert(gi) {
                    diags.push(
                        Diagnostic::error(
                            "DL1005",
                            format!("re-export cycle: `{owner_name}` re-exports `{target}`, which re-exports back"),
                        )
                        .with_span(imp.span, "this `pub import` closes a re-export cycle"),
                    );
                }
                continue;
            }
            let child = compute_exports_gi(
                tgi, ws, owned_types, owned_fns, owned_consts, owned_effects, gi_by_pkg_name, visiting, reported, diags,
            );
            ex.types.extend(child.types);
            ex.fns.extend(child.fns);
            ex.consts.extend(child.consts);
            ex.effects.extend(child.effects);
        }
    }
    visiting.remove(&gi);
    ex
}

// ===== DL1001: dependency authority pin check ===================================================

/// For every dependency edge, verify the dependency's computed authority is within its pin
/// (attenuation order §A.3: effects ⊆ pin effects, declared scopes narrower-or-equal). A missing
/// pin is DL1001 too (every dependency must carry a pin — §3.2).
pub fn check_pins(ws: &Workspace, program: &Program) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for pkg in &ws.packages {
        for dep in &pkg.manifest.dependencies {
            // Resolve the dependency package index (path deps only; git is deferred).
            let Some(&dep_idx) = ws.index.get(&dep.name) else { continue };
            if !pkg.dep_idxs.contains(&dep_idx) {
                continue;
            }
            let auth = package_authority(ws, program, dep_idx);
            let dep_manifest = &ws.packages[dep_idx].manifest;

            match &dep.pin {
                None => {
                    let repair = pin_insert_repair(pkg, dep, &auth);
                    diags.push(
                        Diagnostic::error(
                            "DL1001",
                            format!(
                                "dependency `{}` has no authority pin — accepting its authority [{}] must be an explicit decision",
                                dep.name,
                                comma(&auth.effects)
                            ),
                        )
                        .with_span(pkg.manifest.dep_span(&dep.name), "add a `[dependencies.<name>.authority]` pin")
                        .with_repair(repair),
                    );
                }
                Some(pin) => {
                    let pinned: HashSet<&str> = pin.effects.iter().map(|s| s.as_str()).collect();
                    let excess: Vec<String> = auth.effects.iter().filter(|e| !pinned.contains(e.as_str())).cloned().collect();
                    let scope_probs = scope_violations(pin, &dep_manifest.authority);

                    if !excess.is_empty() || !scope_probs.is_empty() {
                        let mut d = Diagnostic::error(
                            "DL1001",
                            format!(
                                "dependency `{}` exceeds its authority pin: {}",
                                dep.name,
                                describe_excess(&excess, &scope_probs)
                            ),
                        )
                        .with_span(pkg.manifest.dep_span(&dep.name), "dependency pinned here");
                        // Point at an offending public function in the dependency (dep: prefixed).
                        if let Some((span, fname)) = offending_fn(ws, program, dep_idx, &excess) {
                            d = d.with_secondary_span(span, format!("`{fname}` performs `{}` here", excess.join(", ")));
                        }
                        diags.push(d);
                    }
                }
            }
        }
    }
    diags
}

fn comma(s: &BTreeSet<String>) -> String {
    s.iter().cloned().collect::<Vec<_>>().join(", ")
}

fn describe_excess(excess: &[String], scopes: &[String]) -> String {
    let mut parts = Vec::new();
    if !excess.is_empty() {
        parts.push(format!("effects [{}] not in the pin", excess.join(", ")));
    }
    parts.extend(scopes.iter().cloned());
    parts.join("; ")
}

/// Scope subset per §A.3 — only enforced for kinds the pin explicitly constrains.
fn scope_violations(pin: &AuthoritySpec, dep: &AuthoritySpec) -> Vec<String> {
    let mut v = Vec::new();
    if !pin.fs_read.is_empty() {
        for s in &dep.fs_read {
            if !dep_scope_within_paths(s, &pin.fs_read) {
                v.push(format!("fs.read `{s}` is outside the pinned paths"));
            }
        }
    }
    if !pin.fs_write.is_empty() {
        for s in &dep.fs_write {
            if !dep_scope_within_paths(s, &pin.fs_write) {
                v.push(format!("fs.write `{s}` is outside the pinned paths"));
            }
        }
    }
    if !pin.net.is_empty() {
        let hosts: HashSet<&str> = pin.net.iter().map(|s| s.as_str()).collect();
        for s in &dep.net {
            if !hosts.contains(s.as_str()) {
                v.push(format!("net host `{s}` is not in the pinned host set"));
            }
        }
    }
    // Secrets (campaign C19, ruling D34). This dimension was absent while `AuthoritySpec` already
    // carried the field: a dependency could read any secret it named and the pin said nothing,
    // because a secret read contributes no effect and no capability kind for the coarser checks to
    // catch. It follows the same convention as its siblings above — an EMPTY pin means the consumer
    // did not constrain this dimension, exactly as an empty `net` pin does not constrain hosts.
    // Names are compared exactly; there is no prefix or glob relation between secret names.
    if !pin.secrets.is_empty() {
        let pinned: HashSet<&str> = pin.secrets.iter().map(|s| s.as_str()).collect();
        for s in &dep.secrets {
            if !pinned.contains(s.as_str()) {
                v.push(format!("secret `{s}` is not in the pinned secret set"));
            }
        }
    }
    v
}

/// `child` is descendant-or-equal of some `parent` (path prefix, `/`-normalized).
fn dep_scope_within_paths(child: &str, parents: &[String]) -> bool {
    let c = normalize_path(child);
    parents.iter().any(|p| {
        let p = normalize_path(p);
        if p.is_empty() {
            // The package root itself (a pin of `.`): anything that does not climb out of it is
            // within it. Kept explicit so the empty string cannot accidentally prefix-match.
            return !c.starts_with("..");
        }
        c == p || c.starts_with(&format!("{p}/"))
    })
}

/// Lexically normalize a declared scope for comparison: `\` → `/`, **and `.` / `..` resolved**.
///
/// # Campaign finding DEPPIN-LEX-1 — the pin was escapable by spelling
///
/// This used to be `p.replace('\\', "/").trim_end_matches('/')` — separators and trailing slashes
/// only, with `..` left in place. The comparison above is a *prefix* test, so a dependency could
/// declare a scope that lexically sits under the pin and resolves outside it:
///
/// ```text
///   pin:  fs.read = ["data"]
///   dep:  fs.read = ["data/../../outside"]   → starts_with("data/") → PASSED
///   dep:  fs.read = ["../outside"]           → DL1001 refused
/// ```
///
/// Both spellings name the same place outside the pin; only the second was caught. Witnessed with
/// `delulu check` on a real two-package workspace before the fix. This is C84's shape one layer out:
/// a containment decision made on an unresolved path string.
///
/// **Scope of the fix, stated honestly.** The pin is a *declared-authority ceiling* a consumer puts
/// on a dependency (§A.3), not the runtime boundary — runtime filesystem access is bounded by the
/// grant and by `prim::resolve_in_scope`, which normalizes properly, and `--grant-manifest` reads
/// only the ROOT package's manifest, never a dependency's. So this defeated the supply-chain
/// constraint, not filesystem containment. It is fixed because the pin is presented as a real
/// constraint (finding C19 / ruling D34 treated a missing pin dimension as a genuine defect), and a
/// constraint that a spelling walks past is decoration.
fn normalize_path(p: &str) -> String {
    let unix = p.replace('\\', "/");
    let absolute = unix.starts_with('/');
    let mut out: Vec<&str> = Vec::new();
    // `..` above a relative root cannot be cancelled — it must survive as `..`, or `../outside`
    // would normalize to `outside` and land *inside* a pin it climbs out of.
    let mut climbs = 0usize;
    for seg in unix.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                if out.pop().is_none() && !absolute {
                    climbs += 1;
                }
            }
            s => out.push(s),
        }
    }
    let mut parts: Vec<&str> = vec![".."; climbs];
    parts.extend(out);
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

/// Build the `insert-pin` repair for a missing pin. Flagged `authority_widening` (§3.2).
fn pin_insert_repair(pkg: &ResolvedPackage, dep: &crate::manifest::Dependency, auth: &PackageAuthority) -> Repair {
    let effects = auth.effects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
    let insert = format!("\n[dependencies.{}.authority]\neffects = [{}]\n", dep.name, effects);
    let at = pkg.manifest.raw.len() as u32;
    Repair {
        id: "insert_dependency_pin",
        confidence: Confidence::Suggest,
        authority_widening: true,
        requires_human: false,
        edits: vec![Edit { file: pkg.manifest.file, start_byte: at, end_byte: at, insert }],
        reason: None,
    }
}

/// Find a public function in the dependency whose effects include an excess effect, for a span.
fn offending_fn(ws: &Workspace, program: &Program, dep_idx: usize, excess: &[String]) -> Option<(Span, String)> {
    if excess.is_empty() {
        return None;
    }
    for wm in ws.modules.iter().filter(|m| m.pkg == dep_idx) {
        for item in &wm.unit.module.items {
            if let Item::Fn(f) = item {
                if !f.public {
                    continue;
                }
                let key = format!("{}::{}", wm.unit.name, f.name.name);
                if let Some(facts) = program.facts.get(&key) {
                    if facts.effects.iter().any(|e| excess.iter().any(|x| x == e.name())) {
                        return Some((f.name.span, f.name.name.clone()));
                    }
                }
            }
        }
    }
    None
}

// ===== api-row canonicalization (for api_row_hash, §4.4) ========================================

/// A canonical, sorted, stable dump of every `pub` function signature in one package.
pub fn api_row_dump(ws: &Workspace, pkg_idx: usize) -> String {
    let mut rows: Vec<String> = Vec::new();
    for wm in ws.modules.iter().filter(|m| m.pkg == pkg_idx) {
        for item in &wm.unit.module.items {
            if let Item::Fn(f) = item {
                if !f.public {
                    continue;
                }
                let params = f.params.iter().map(|p| render_type(&p.ty)).collect::<Vec<_>>().join(",");
                let ret = f.ret.as_ref().map(render_type).unwrap_or_else(|| "Unit".to_string());
                let row = f.row.as_ref().map(render_row).unwrap_or_else(|| "{}".to_string());
                rows.push(format!("{}::{}({params})->{ret}!{row}", wm.unit.name, f.name.name));
            }
        }
    }
    rows.sort();
    rows.join("\n")
}

fn render_type(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Named { path, args, .. } => {
            let base = path.dotted();
            if args.is_empty() {
                base
            } else {
                let inner = args.iter().map(render_type).collect::<Vec<_>>().join(",");
                format!("{base}[{inner}]")
            }
        }
        TypeExpr::Fn { params, ret, row, .. } => {
            let ps = params.iter().map(render_type).collect::<Vec<_>>().join(",");
            let r = ret.as_ref().map(|b| render_type(b)).unwrap_or_else(|| "Unit".to_string());
            let row = row.as_ref().map(render_row).unwrap_or_else(|| "{}".to_string());
            format!("fn({ps})->{r}!{row}")
        }
        // The rcap is part of the written signature, so it is part of the fingerprint.
        TypeExpr::Rcap { rcap, inner, .. } => format!("{} {}", rcap.name(), render_type(inner)),
    }
}

fn render_row(r: &RowExpr) -> String {
    let mut effects: Vec<String> = r.effects.iter().map(|p| p.dotted()).collect();
    effects.sort();
    let tail = r.tail.as_ref().map(|t| format!("|{}", t.name)).unwrap_or_default();
    format!("{{{}{}}}", effects.join(","), tail)
}

/// Effect/cap-kind canonical JSON for `authority_hash` (§4.4).
pub fn authority_canonical_json(auth: &PackageAuthority) -> String {
    let effects: Vec<String> = auth.effects.iter().cloned().collect();
    let kinds: Vec<String> = auth.cap_kinds.iter().cloned().collect();
    // Deliberately effects+kinds only, matching this hash's documented meaning. Secrets are caught
    // as a *widening* by the lock entry's `secrets` field (see `authority_widened`); they are NOT
    // folded into this hash, because doing so would change every existing `authority_hash` and
    // invalidate any lockfile on the next build (DL1002) — a format break, and backward
    // compatibility is owner-reserved. A same-version secret change is still caught: the source
    // changed, so `content_hash` moves and DL1010 fires. See HARDENING_CAMPAIGN C18.
    serde_json::json!({ "effects": effects, "cap_kinds": kinds }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("delulu_deps_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        dir
    }
    fn write(dir: &Path, rel: &str, contents: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, contents).unwrap();
    }
    fn err_codes(prog: &Program) -> Vec<String> {
        prog.diagnostics.iter().filter(|d| d.is_error()).map(|d| d.code.to_string()).collect()
    }
    const LIB: &str = "[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n";

    #[test]
    fn a_new_secret_read_is_seen_as_an_authority_widening() {
        // HARDENING_CAMPAIGN C18, the end-to-end witness. Two versions of the same package that
        // differ ONLY by which secrets they read. Reading a secret is pure — no effect, no
        // capability kind — so the coarse authority (effects, cap_kinds) is byte-identical between
        // them. Before secrets entered the lock entry, `authority_widened` therefore returned
        // false here and `delulu lock` accepted the change on any bump: a dependency could begin
        // reading a new secret silently. The two assertions below are the proof: the authority
        // genuinely widened (the secret sets differ), and the lock now sees it.
        let mk = |name: &str, body: &str| -> crate::lockfile::LockEntry {
            let dir = scratch(name);
            write(&dir, "delulu.toml", LIB);
            write(&dir, "src/root.delulu", body);
            let ws = resolve_workspace(&dir);
            let prog = check_workspace(&ws);
            assert!(err_codes(&prog).is_empty(), "{name} should check clean: {:?}", prog.diagnostics);
            crate::lockfile::compute_entry(&ws, &prog, 0)
        };
        let v1 = mk(
            "secret_v1",
            "module p\npub fn key(root: Root) -> Secret[Str] { root.secret(\"TELEMETRY_TOKEN\") }\n",
        );
        let v2 = mk(
            "secret_v2",
            "module p\npub fn key(root: Root) -> Secret[Str] { root.secret(\"TELEMETRY_TOKEN\") }\n\
             pub fn key2(root: Root) -> Secret[Str] { root.secret(\"DB_PASSWORD\") }\n",
        );
        // The coarse authority is identical — this is why the widening used to be invisible.
        assert_eq!(v1.effects, v2.effects, "effects must be equal; the secret is the only change");
        assert_eq!(v1.cap_kinds, v2.cap_kinds, "cap_kinds must be equal; the secret is the only change");
        // But the authority genuinely widened, and the lock now records and detects it.
        assert!(v2.secrets.contains(&"DB_PASSWORD".to_string()), "v2 reads a new secret");
        assert!(!v1.secrets.contains(&"DB_PASSWORD".to_string()), "v1 does not read it");
        assert!(
            crate::lockfile::authority_widened(&v1, &v2),
            "a package that begins reading a new secret has widened its authority"
        );
        // The authority_hash intentionally does NOT move on a secret change (it is effects+kinds by
        // documented definition, and folding secrets in would invalidate existing lockfiles). The
        // backstop for a same-version secret change is the content hash: the source changed, so it
        // moves, and DL1010 fires. That is what makes the surgical widening fix safe.
        assert_eq!(v1.authority_hash, v2.authority_hash, "the hash stays effects+kinds by design");
        assert_ne!(v1.content_hash, v2.content_hash, "a secret change is a source change → content hash moves");
    }

    // ----- C19 (ruling D34): the manifest and the pin now bound SECRETS, not just effects -------

    #[test]
    fn a_package_must_declare_the_secrets_it_reads() {
        // C19, the self-authority half. `root.secret("X")` contributes no effect and no capability
        // kind — only a name — so an effects-only ceiling check saw nothing wrong and a package
        // could read any secret it liked while declaring none. The manifest is a CEILING, and a
        // dimension the ceiling does not mention is a dimension it does not bound.
        let dir = scratch("undeclared_secret");
        write(&dir, "delulu.toml", LIB); // `[authority] effects=[]`, no `secrets` key at all
        write(
            &dir,
            "src/root.delulu",
            "module p\npub fn key(root: Root) -> Secret[Str] { root.secret(\"DB_PASSWORD\") }\n",
        );
        let ws = resolve_workspace(&dir);
        let prog = check_workspace(&ws);
        // `check_self_authority` is the ceiling gate the CLI composes (it is not part of
        // `check_workspace`, which computes facts); call it the same way `delulu build` does.
        let diags = check_self_authority(&ws, &prog);
        let d = diags
            .iter()
            .find(|d| d.code == "DL1009")
            .unwrap_or_else(|| panic!("expected DL1009, got {diags:?}"));
        assert!(d.message.contains("DB_PASSWORD"), "the message must name the secret: {}", d.message);
    }

    #[test]
    fn declaring_the_secret_is_enough_to_check_clean() {
        // The accepting half — the ceiling bounds, it does not forbid.
        let dir = scratch("declared_secret");
        write(
            &dir,
            "delulu.toml",
            "[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\nsecrets=[\"DB_PASSWORD\"]\n",
        );
        write(
            &dir,
            "src/root.delulu",
            "module p\npub fn key(root: Root) -> Secret[Str] { root.secret(\"DB_PASSWORD\") }\n",
        );
        let ws = resolve_workspace(&dir);
        let prog = check_workspace(&ws);
        assert!(err_codes(&prog).is_empty(), "{:?}", prog.diagnostics);
        assert!(check_self_authority(&ws, &prog).is_empty(), "a declared secret is within the ceiling");
    }

    #[test]
    fn a_dependency_may_not_read_a_secret_outside_its_pin() {
        // C19, the pin half. `AuthoritySpec` already carried a `secrets` field and
        // `scope_violations` never looked at it, so a consumer who deliberately pinned WHICH secrets
        // a dependency may read was not actually constrained — the pin was decoration.
        let root = std::env::temp_dir().join("delulu_deps_test_secret_pin");
        let _ = fs::remove_dir_all(&root);
        let app = root.join("app");
        let lib = root.join("lib");
        fs::create_dir_all(app.join("src")).unwrap();
        fs::create_dir_all(lib.join("src")).unwrap();
        // The dependency reads TWO secrets and declares both (so DL1009 is satisfied).
        write(
            &lib,
            "delulu.toml",
            "[package]\nname=\"lib\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\nsecrets=[\"TELEMETRY_TOKEN\",\"DB_PASSWORD\"]\n",
        );
        write(
            &lib,
            "src/root.delulu",
            "module lib\npub fn a(root: Root) -> Secret[Str] { root.secret(\"TELEMETRY_TOKEN\") }\n\
             pub fn b(root: Root) -> Secret[Str] { root.secret(\"DB_PASSWORD\") }\n",
        );
        // The consumer pins it to ONE of them.
        write(
            &app,
            "delulu.toml",
            "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n\
             [dependencies]\nlib = { path = \"../lib\", authority = { effects = [], secrets = [\"TELEMETRY_TOKEN\"] } }\n",
        );
        write(&app, "src/root.delulu", "module app\nimport lib\npub fn go() -> Int { 1 }\n");
        let ws = resolve_workspace(&app);
        let prog = check_workspace(&ws);
        let diags = check_pins(&ws, &prog);
        let d = diags
            .iter()
            .find(|d| d.code == "DL1001")
            .unwrap_or_else(|| panic!("expected DL1001, got {diags:?}"));
        assert!(
            d.message.contains("DB_PASSWORD"),
            "the message must name the secret outside the pin: {}",
            d.message
        );
        assert!(
            !d.message.contains("TELEMETRY_TOKEN"),
            "the pinned secret is permitted and must not be reported: {}",
            d.message
        );
    }

    /// **DEPPIN-LEX-1 regression lock.** Two spellings of the same out-of-pin location must both be
    /// refused. Before the fix the first one PASSED: the comparison was a prefix test on an
    /// unresolved string, and `data/../../outside` starts with `data/`. Witnessed end-to-end with
    /// `delulu check` on a two-package workspace — `../outside` was refused DL1001 while the
    /// climbing spelling checked clean, same destination, opposite verdicts.
    #[test]
    fn a_dependency_scope_cannot_escape_its_pin_by_spelling() {
        let pin = vec!["data".to_string()];
        assert!(
            !dep_scope_within_paths("data/../../outside", &pin),
            "DEPPIN-LEX-1: a scope that climbs out of the pin is outside it however it is spelled"
        );
        assert!(!dep_scope_within_paths("../outside", &pin), "the plain spelling stays refused");
        assert!(!dep_scope_within_paths("data/../other", &pin), "a sibling reached through `..` is outside");

        // The legitimate cases must keep working — a fix that refuses everything is not a fix.
        assert!(dep_scope_within_paths("data", &pin), "the pin itself is within the pin");
        assert!(dep_scope_within_paths("data/sub/x", &pin), "a real descendant is within");
        assert!(dep_scope_within_paths("./data/sub", &pin), "a `.`-spelled descendant is within");
        assert!(dep_scope_within_paths("data\\sub", &pin), "a Windows-spelled descendant is within");
        assert!(dep_scope_within_paths("data/sub/../other", &pin), "climbing but staying inside is within");
        assert!(!dep_scope_within_paths("database", &pin), "a name that merely shares a prefix is not a child");

        // A pin of the package root contains anything that does not climb out of it.
        let root_pin = vec![".".to_string()];
        assert!(dep_scope_within_paths("data", &root_pin));
        assert!(dep_scope_within_paths("./data/sub", &root_pin));
        assert!(!dep_scope_within_paths("../outside", &root_pin), "climbing out of the root is still out");
    }

    #[test]
    fn pub_import_re_exports_transitively() {
        // root plain-imports facade; facade `pub import`s inner; so `helper` is visible in root.
        let dir = scratch("reexport");
        write(&dir, "delulu.toml", LIB);
        write(&dir, "src/inner.delulu", "module p.inner\npub fn helper(n: Str) -> Str { \"hi \" + n }\n");
        write(&dir, "src/facade.delulu", "module p.facade\npub import p.inner\n");
        write(&dir, "src/root.delulu", "module p\nimport p.facade\npub fn use_it() -> Str { helper(\"x\") }\n");
        let prog = check_workspace(&resolve_workspace(&dir));
        assert!(err_codes(&prog).is_empty(), "re-export should make helper visible: {:?}", prog.diagnostics);
    }

    #[test]
    fn re_export_cycle_is_dl1005() {
        let dir = scratch("recycle");
        write(&dir, "delulu.toml", LIB);
        write(&dir, "src/a.delulu", "module a\npub import b\npub fn fa() -> Int { 1 }\n");
        write(&dir, "src/b.delulu", "module b\npub import a\npub fn fb() -> Int { 2 }\n");
        let prog = check_workspace(&resolve_workspace(&dir));
        assert!(err_codes(&prog).iter().any(|c| c == "DL1005"), "{:?}", prog.diagnostics);
    }

    #[test]
    fn plain_import_does_not_re_export() {
        // facade plain-imports inner (no re-export); root imports facade and cannot see helper.
        let dir = scratch("noreexport");
        write(&dir, "delulu.toml", LIB);
        write(&dir, "src/inner.delulu", "module p.inner\npub fn helper() -> Int { 1 }\n");
        write(&dir, "src/facade.delulu", "module p.facade\nimport p.inner\npub fn via() -> Int { helper() }\n");
        write(&dir, "src/root.delulu", "module p\nimport p.facade\npub fn bad() -> Int { helper() }\n");
        let prog = check_workspace(&resolve_workspace(&dir));
        assert!(err_codes(&prog).iter().any(|c| c == "DL0301"), "helper must NOT leak through a plain import: {:?}", prog.diagnostics);
    }

    // ===== C58 · the report lands where the reader can act on it ============================
    //
    // A `pub fn`'s signature is lowered in the scope of whoever IMPORTS it, so an importer that
    // cannot see a type the signature names produced `unknown type` at a span inside the
    // DEPENDENCY'S own file — which had checked clean, and where the type is in scope.

    /// Sets up the exact shape: `inner` declares the type, `facade` uses it in a `pub fn` but only
    /// plain-imports it, `root` imports `facade` **and calls it**.
    ///
    /// The call matters. A signature is lowered per call site, so merely importing a function whose
    /// parameter you cannot name costs nothing — the failure appears when you use it, which is the
    /// right laziness and the reason the first version of this fixture checked clean.
    fn unexported_type_workspace(name: &str) -> std::path::PathBuf {
        let dir = scratch(name);
        write(&dir, "delulu.toml", LIB);
        write(&dir, "src/inner.delulu", "module p.inner\npub type Sample { v: Int }\n");
        write(
            &dir,
            "src/facade.delulu",
            "module p.facade\nimport p.inner\npub fn widen(s: Sample) -> Int { s.v }\n",
        );
        write(&dir, "src/root.delulu", "module p\nimport p.facade\npub fn go() -> Int { widen(0) }\n");
        dir
    }

    #[test]
    fn an_unexported_type_in_a_public_signature_is_not_blamed_on_the_file_it_is_in_scope_in() {
        let dir = unexported_type_workspace("c58blame");
        let ws = resolve_workspace(&dir);
        let prog = check_workspace(&ws);
        let root_file = ws.modules.iter().find(|m| m.unit.name == "p").expect("root module").unit.file;
        let facade_file = ws.modules.iter().find(|m| m.unit.name == "p.facade").expect("facade").unit.file;

        // Still refused — the rule is right and unchanged.
        assert!(err_codes(&prog).iter().any(|c| c == "DL0301"), "{:?}", prog.diagnostics);

        // But nothing is reported against `p.facade`, which checks clean on its own and where
        // `Sample` is genuinely in scope. That was the whole defect.
        let blamed_facade: Vec<&Diagnostic> = prog
            .diagnostics
            .iter()
            .filter(|d| d.is_error() && d.primary_span().map(|s| s.file == facade_file).unwrap_or(false))
            .collect();
        assert!(blamed_facade.is_empty(), "the dependency's own file must not be blamed: {blamed_facade:?}");

        // It is reported against the module that actually cannot compile.
        assert!(
            prog.diagnostics
                .iter()
                .any(|d| d.is_error() && d.primary_span().map(|s| s.file == root_file).unwrap_or(false)),
            "the importer must be told: {:?}",
            prog.diagnostics
        );
    }

    #[test]
    fn the_report_says_where_the_type_lives_and_how_to_reach_it() {
        let dir = unexported_type_workspace("c58advice");
        let prog = check_workspace(&resolve_workspace(&dir));
        let text = prog.diagnostics.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join("\n");
        assert!(text.contains("`Sample`"), "must name the type: {text}");
        assert!(text.contains("p.inner"), "must say where it is declared: {text}");
        assert!(text.contains("pub import p.inner"), "must state the fix: {text}");
    }

    /// The advice has to be true. Applying exactly what the message says must build clean —
    /// otherwise the diagnostic is confident and wrong, which is worse than terse.
    #[test]
    fn applying_the_suggested_re_export_builds_clean() {
        let dir = unexported_type_workspace("c58fix");
        write(
            &dir,
            "src/facade.delulu",
            "module p.facade\npub import p.inner\npub fn widen(s: Sample) -> Int { s.v }\n",
        );
        // With the re-export in place `Sample` reaches the root, so the call can be written the way
        // it always should have been — which is the point: the advice restores the whole type.
        write(
            &dir,
            "src/root.delulu",
            "module p\nimport p.facade\npub fn go() -> Int { widen(Sample { v: 1 }) }\n",
        );
        let prog = check_workspace(&resolve_workspace(&dir));
        assert!(err_codes(&prog).is_empty(), "the suggested fix must work: {:?}", prog.diagnostics);
    }

    /// The skip branch. A name that is simply misspelled has no home anywhere, so there is nothing
    /// true to add — it keeps its precise span and gains no invented advice.
    #[test]
    fn a_misspelled_type_keeps_its_plain_message() {
        let dir = scratch("c58typo");
        write(&dir, "delulu.toml", LIB);
        write(&dir, "src/root.delulu", "module p\ntype Sample { v: Int }\npub fn f(s: Smaple) -> Int { 1 }\n");
        let prog = check_workspace(&resolve_workspace(&dir));
        let d = prog
            .diagnostics
            .iter()
            .find(|d| d.code == "DL0301")
            .expect("the typo must still be refused");
        assert_eq!(d.message, "unknown type `Smaple`", "a typo must not be decorated with a guess");
    }

    /// One fact, reported once. A diamond lowers a shared dependency's signature afresh per path,
    /// which used to multiply the SAME sentence about the SAME place up to four times.
    #[test]
    fn the_same_failure_is_not_reported_once_per_path_through_a_diamond() {
        let dir = scratch("c58dedupe");
        write(&dir, "delulu.toml", LIB);
        write(&dir, "src/inner.delulu", "module p.inner\npub type Sample { v: Int }\n");
        write(&dir, "src/facade.delulu", "module p.facade\nimport p.inner\npub fn a(s: Sample) -> Int { s.v }\n");
        write(&dir, "src/other.delulu", "module p.other\nimport p.inner\npub fn b(s: Sample) -> Int { s.v }\n");
        write(&dir, "src/root.delulu", "module p\nimport p.facade\nimport p.other\npub fn go() -> Int { 1 }\n");
        let prog = check_workspace(&resolve_workspace(&dir));
        let mut seen: Vec<(&str, &str, Option<Span>)> = Vec::new();
        for d in prog.diagnostics.iter().filter(|d| d.is_error()) {
            let key = (d.code, d.message.as_str(), d.primary_span());
            assert!(!seen.contains(&key), "duplicate diagnostic: {} @ {:?}", d.message, d.primary_span());
            seen.push(key);
        }
    }
}
