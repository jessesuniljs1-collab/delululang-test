//! Build an [`Atlas`] from compiler facts — and *only* from compiler facts.
//!
//! Nothing here re-parses source text: nodes and edges come from the checker's `Program`
//! (`FnFacts`: effect rows, purity, resolved callees), the resolved package/module structure, the
//! typed signatures in the AST, and the `delulu authority` report. Every collection is sorted by
//! stable id before it is stored, so the output is byte-identical across runs (criterion 1).

use std::collections::{BTreeMap, BTreeSet};

use delulu_check::program::Program;
use delulu_diag::SourceMap;
use delulu_syntax::ast::{Block, Expr, Item, LitKind, Module, Stmt, TypeExpr};
use serde_json::Value;

use crate::model::*;

/// One package, as the builder sees it.
pub struct PackageView {
    pub name: String,
    /// Names of this package's direct dependency packages.
    pub deps: Vec<String>,
    pub is_root: bool,
}

/// One module and its AST, tagged with its owning package.
pub struct ModuleView<'a> {
    pub package: String,
    pub name: String,
    pub module: &'a Module,
}

/// Everything the builder needs. Assembled by the thin CLI from its existing check pipeline.
pub struct BuildInput<'a> {
    pub root: String,
    pub packages: Vec<PackageView>,
    pub modules: Vec<ModuleView<'a>>,
    pub program: &'a Program,
    pub source_map: &'a SourceMap,
    /// The `delulu authority` report for this program — embedded verbatim and used as the parity
    /// source for `performs`/`requires` (criterion 5). Resource nodes come straight from its
    /// `capabilities` scopes + `secrets`, so the atlas's resources ARE the authority report's.
    pub authority: Value,
    /// Top-N by degree for god nodes (default 10).
    pub god_n: usize,
    /// The custody overlay (phase A3), or `None`.
    pub custody: Option<Value>,
}

/// Map a checker `ResourceKind` name / authority capability `kind` to the atlas resource class.
///
/// P4-11: the last four were missing, so a program that drove an actuator, read a sensor, dispatched
/// to a compute device or hosted plugins had none of those in its Atlas — the resources a
/// physical-safety or supply-chain audit most needs to see. Three kinds have no class, each because
/// the authority report discloses it elsewhere: `Declassify` reaches a secret (secret nodes and
/// `declassifies` edges), and `ForeignLoad`/`Python` are the foreign boundary (`foreign` nodes and
/// the report's `foreign_calls`, spec §6).
pub fn class_of_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "FsRead" => "fs_read",
        "FsWrite" => "fs_write",
        "Http" => "net",
        "Console" => "console",
        "Clock" => "clock",
        "Rand" => "rand",
        "Actuator" => "actuator",
        "Sensor" => "sensor",
        "Compute" => "compute",
        "PluginHost" => "plugin_host",
        _ => return None,
    })
}

/// Collect the user-type names named anywhere in a signature `TypeExpr` (recursing into generic
/// args and function-type params/returns). Builtins are filtered later by matching against the
/// known type-node set, so we simply gather every `Named` head here.
fn collect_type_names(t: &TypeExpr, out: &mut BTreeSet<String>) {
    match t {
        TypeExpr::Named { path, args, .. } => {
            if let Some(last) = path.segs.last() {
                out.insert(last.name.clone());
            }
            for a in args {
                collect_type_names(a, out);
            }
        }
        TypeExpr::Fn { params, ret, .. } => {
            for p in params {
                collect_type_names(p, out);
            }
            if let Some(r) = ret {
                collect_type_names(r, out);
            }
        }
        TypeExpr::Rcap { inner, .. } => collect_type_names(inner, out),
    }
}

impl Atlas {
    /// Build the atlas from compiler facts.
    pub fn build(input: BuildInput) -> Atlas {
        let mut nodes: BTreeMap<String, Node> = BTreeMap::new();
        let mut edges: BTreeSet<Edge> = BTreeSet::new();

        // module name -> package name (for resolving qualified fns and imports to ids).
        let module_pkg: BTreeMap<String, String> =
            input.modules.iter().map(|m| (m.name.clone(), m.package.clone())).collect();
        // type name -> its type-node ids (for uses_type resolution; ambiguous names are skipped).
        let mut type_by_name: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new(); // name -> [(module, id)]

        // ----- 1. package nodes + depends_on --------------------------------------------------
        for p in &input.packages {
            nodes.insert(pkg_id(&p.name), Node::new(pkg_id(&p.name), NodeKind::Package, &p.name));
            for dep in &p.deps {
                edges.insert(Edge { from: pkg_id(&p.name), to: pkg_id(dep), kind: EdgeKind::DependsOn });
            }
        }

        // ----- 2. module + type nodes, contains + imports -------------------------------------
        for mv in &input.modules {
            let mid = mod_id(&mv.package, &mv.name);
            let mut mnode = Node::new(&mid, NodeKind::Module, &mv.name);
            mnode.package = Some(mv.package.clone());
            nodes.insert(mid.clone(), mnode);
            edges.insert(Edge { from: pkg_id(&mv.package), to: mid.clone(), kind: EdgeKind::Contains });

            for item in &mv.module.items {
                if let Item::Type(td) = item {
                    let tid = type_id(&mv.package, &mv.name, &td.name.name);
                    let mut tnode = Node::new(&tid, NodeKind::Type, &td.name.name);
                    tnode.module = Some(mv.name.clone());
                    tnode.span = Some(span_loc(input.source_map, td.name.span));
                    nodes.insert(tid.clone(), tnode);
                    edges.insert(Edge { from: mid.clone(), to: tid.clone(), kind: EdgeKind::Contains });
                    type_by_name.entry(td.name.name.clone()).or_default().push((mv.name.clone(), tid));
                }
            }

            for imp in &mv.module.imports {
                let target = imp.path.dotted();
                if let Some(tpkg) = module_pkg.get(&target) {
                    edges.insert(Edge {
                        from: mid.clone(),
                        to: mod_id(tpkg, &target),
                        kind: EdgeKind::Imports,
                    });
                }
            }
        }

        // fn name-span lookup by qualified name, gathered from the AST.
        let mut fn_span: BTreeMap<String, SpanLoc> = BTreeMap::new();
        let mut fn_ast: BTreeMap<String, (&str, &delulu_syntax::ast::FnDecl)> = BTreeMap::new();
        for mv in &input.modules {
            for item in &mv.module.items {
                if let Item::Fn(f) = item {
                    let qual = format!("{}::{}", mv.name, f.name.name);
                    fn_span.insert(qual.clone(), span_loc(input.source_map, f.name.span));
                    fn_ast.insert(qual, (mv.name.as_str(), f));
                }
            }
        }

        // ----- 3. function nodes, contains, calls, performs, requires, declassifies -----------
        let mut effect_names: BTreeSet<String> = BTreeSet::new();
        for (qual, facts) in &input.program.facts {
            let Some((modname, fname)) = qual.split_once("::") else { continue };
            let Some(pkg) = module_pkg.get(modname) else { continue };
            let fid = fn_id(pkg, modname, fname);
            let mut fnode = Node::new(&fid, NodeKind::Function, fname);
            fnode.module = Some(modname.to_string());
            fnode.pure = Some(facts.pure);
            fnode.effects = facts.effects.iter().map(|e| e.name().to_string()).collect();
            fnode.span = fn_span.get(qual).cloned();
            nodes.insert(fid.clone(), fnode);
            edges.insert(Edge {
                from: mod_id(pkg, modname),
                to: fid.clone(),
                kind: EdgeKind::Contains,
            });

            // calls: resolve each callee name to its owning module (checked fact — no re-lexing).
            for callee in &facts.callees {
                if let Some(owner) = input.program.call_owner.get(modname).and_then(|o| o.get(callee)) {
                    if let Some(opkg) = module_pkg.get(owner) {
                        edges.insert(Edge {
                            from: fid.clone(),
                            to: fn_id(opkg, owner, callee),
                            kind: EdgeKind::Calls,
                        });
                    }
                }
            }

            // performs: one edge per effect in the checked row.
            for e in &facts.effects {
                let name = e.name().to_string();
                effect_names.insert(name.clone());
                edges.insert(Edge { from: fid.clone(), to: effect_id(&name), kind: EdgeKind::Performs });
            }

            // uses_type: from the checked signature (params + return). Ambiguous names are skipped.
            if let Some((modname2, f)) = fn_ast.get(qual) {
                let mut names = BTreeSet::new();
                for p in &f.params {
                    collect_type_names(&p.ty, &mut names);
                }
                if let Some(r) = &f.ret {
                    collect_type_names(r, &mut names);
                }
                for n in &names {
                    if let Some(cands) = type_by_name.get(n) {
                        let target = cands
                            .iter()
                            .find(|(m, _)| m == modname2)
                            .or_else(|| if cands.len() == 1 { cands.first() } else { None });
                        if let Some((_, tid)) = target {
                            edges.insert(Edge { from: fid.clone(), to: tid.clone(), kind: EdgeKind::UsesType });
                        }
                    }
                }
            }
        }

        // ----- 4. effect nodes ----------------------------------------------------------------
        for name in &effect_names {
            nodes.insert(effect_id(name), Node::new(effect_id(name), NodeKind::Effect, name));
        }

        // ----- 5. resource nodes + requires/declassifies --------------------------------------
        // Resource nodes come from the authority report's capability scopes (patterns) + secrets,
        // so the atlas's resources are exactly the authority report's resources (criterion 5).
        let mut class_resources: BTreeMap<String, Vec<String>> = BTreeMap::new(); // class -> [res id]
        if let Some(caps) = input.authority.get("capabilities").and_then(|c| c.as_array()) {
            for cap in caps {
                let Some(kind) = cap.get("kind").and_then(|k| k.as_str()) else { continue };
                let Some(class) = class_of_kind(kind) else { continue };
                let scopes: Vec<String> = cap
                    .get("scopes")
                    .and_then(|s| s.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                // The scopes the code NAMES (`root.fs_read("./config")`) — kept on the node rather
                // than folded into its id, so every existing id stays what it was (NE-14, P4-11).
                let requested: Vec<String> = cap
                    .get("requested_scopes")
                    .and_then(|s| s.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                let patterns = if scopes.is_empty() { vec!["*".to_string()] } else { scopes };
                for pat in patterns {
                    let rid = resource_id(class, &pat);
                    let mut rnode = Node::new(&rid, NodeKind::Resource, format!("{class}:{pat}"));
                    rnode.resource_class = Some(class.to_string());
                    rnode.pattern = Some(pat.clone());
                    rnode.requested_scopes = requested.clone();
                    nodes.insert(rid.clone(), rnode);
                    class_resources.entry(class.to_string()).or_default().push(rid);
                }
            }
        }
        // Secret resources (also a `declassifies` target).
        let secret_names: Vec<String> = input
            .authority
            .get("secrets")
            .and_then(|s| s.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        for s in &secret_names {
            let rid = resource_id("secret", s);
            let mut rnode = Node::new(&rid, NodeKind::Resource, format!("secret:{s}"));
            rnode.resource_class = Some("secret".to_string());
            rnode.pattern = Some(s.clone());
            nodes.insert(rid.clone(), rnode);
        }

        // requires edges: each function wielding a capability class requires that class's resources;
        // the root package requires the union (the program's resource surface).
        let root_pkg = input.packages.iter().find(|p| p.is_root).map(|p| p.name.clone());
        for (qual, facts) in &input.program.facts {
            let Some((modname, fname)) = qual.split_once("::") else { continue };
            let Some(pkg) = module_pkg.get(modname) else { continue };
            let fid = fn_id(pkg, modname, fname);
            for k in &facts.cap_kinds {
                if let Some(class) = class_of_kind(k.name()) {
                    if let Some(res_ids) = class_resources.get(class) {
                        for rid in res_ids {
                            edges.insert(Edge { from: fid.clone(), to: rid.clone(), kind: EdgeKind::Requires });
                        }
                    }
                }
            }
            for s in &facts.secret_names {
                let rid = resource_id("secret", s);
                let kind = if facts.declassifies { EdgeKind::Declassifies } else { EdgeKind::Requires };
                edges.insert(Edge { from: fid.clone(), to: rid, kind });
            }
        }
        if let Some(rp) = &root_pkg {
            for rid in nodes.keys().filter(|id| id.starts_with("res:")).cloned().collect::<Vec<_>>() {
                edges.insert(Edge { from: pkg_id(rp), to: rid, kind: EdgeKind::Requires });
            }
        }

        // ----- 5b. foreign boundary: nodes + function→foreign edges ---------------------------
        // C symbols come from the module's own checked `foreign` blocks (a declared symbol is a
        // checked fact and always gets a node); Python modules come from `py.import("literal")`
        // call sites. A `foreign` EDGE is added only where BOTH hold: the function's CHECKED row
        // carries `ForeignCall` (the checker's fact) and its body syntactically reaches the
        // boundary (a call of a declared foreign symbol / a literal `import`). Non-literal import
        // names are statically unknowable and are skipped — the §2.6 under-approximation caveat
        // covers exactly this.
        let mut module_c_syms: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
        for mv in &input.modules {
            let mut syms = BTreeSet::new();
            for item in &mv.module.items {
                if let Item::Foreign(fd) = item {
                    for f in &fd.fns {
                        let fid = foreign_c_id(&f.name.name);
                        let mut node = Node::new(&fid, NodeKind::Foreign, &f.name.name);
                        node.module = Some(mv.name.clone());
                        // The declaring lib, for orientation (`pattern` doubles as detail text).
                        node.pattern = Some(format!("c lib {}", fd.name.name));
                        node.span = Some(span_loc(input.source_map, f.name.span));
                        nodes.insert(fid, node);
                        syms.insert(f.name.name.clone());
                    }
                }
            }
            module_c_syms.insert(mv.name.as_str(), syms);
        }
        for (qual, facts) in &input.program.facts {
            if !facts.effects.iter().any(|e| e.name() == "ForeignCall") {
                continue; // no checked ForeignCall row ⇒ no foreign edge, ever
            }
            let Some((modname, fname)) = qual.split_once("::") else { continue };
            let Some(pkg) = module_pkg.get(modname) else { continue };
            let Some((_, f)) = fn_ast.get(qual) else { continue };
            let empty = BTreeSet::new();
            let c_syms = module_c_syms.get(modname).unwrap_or(&empty);
            let mut reach = ForeignReach::default();
            walk_block_for_foreign(&f.body, c_syms, &mut reach);
            let fid = fn_id(pkg, modname, fname);
            for sym in reach.c_symbols {
                edges.insert(Edge { from: fid.clone(), to: foreign_c_id(&sym), kind: EdgeKind::Foreign });
            }
            for module in reach.py_imports {
                let pid = foreign_py_id(&module);
                let mut node = Node::new(&pid, NodeKind::Foreign, &module);
                node.pattern = Some("python module".to_string());
                nodes.entry(pid.clone()).or_insert(node);
                edges.insert(Edge { from: fid.clone(), to: pid, kind: EdgeKind::Foreign });
            }
        }

        // ----- 6. finalize: sort, god nodes ---------------------------------------------------
        let nodes: Vec<Node> = nodes.into_values().collect(); // BTreeMap → already id-sorted
        let mut edges: Vec<Edge> = edges.into_iter().collect(); // BTreeSet → sorted
        edges.sort();

        let mut atlas = Atlas {
            version: SCHEMA.to_string(),
            root: input.root,
            nodes,
            edges,
            gods: Vec::new(),
            authority: input.authority,
            custody: None,
            caveats: vec![CAVEAT_STATIC.to_string()],
        };
        atlas.gods = compute_gods(&atlas, input.god_n.max(1));
        // The custody overlay (phase A3): read-only awareness, attached last so the graft path and
        // the build path share one implementation.
        if let Some(overlay) = input.custody {
            atlas.attach_custody(overlay, input.god_n.max(1));
        }
        atlas
    }

    /// Attach a custody overlay to a finished atlas: one `grant:` node per broker grant, a
    /// `delegates` edge per parent→child link, the verbatim custody caveat, and re-ranked god
    /// nodes. Sorted-id determinism is preserved (grant ids sort into place). Read-only awareness —
    /// the overlay never changes any compiler-derived node or edge.
    pub fn attach_custody(&mut self, overlay: Value, god_n: usize) {
        if let Some(grants) = overlay.get("grants").and_then(|g| g.as_array()) {
            for g in grants {
                let Some(gid) = g.get("id").and_then(|i| i.as_str()) else { continue };
                let nid = grant_id(gid);
                let mut node = Node::new(&nid, NodeKind::Grant, gid);
                node.pattern = g.get("state").and_then(|s| s.as_str()).map(str::to_string);
                self.nodes.push(node);
                if let Some(parent) = g.get("parent").and_then(|p| p.as_str()).filter(|p| !p.is_empty()) {
                    self.edges.push(Edge { from: grant_id(parent), to: nid, kind: EdgeKind::Delegates });
                }
            }
        }
        self.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        self.nodes.dedup_by(|a, b| a.id == b.id);
        self.edges.sort();
        self.edges.dedup();
        self.custody = Some(overlay);
        if !self.caveats.iter().any(|c| c == CAVEAT_CUSTODY) {
            self.caveats.push(CAVEAT_CUSTODY.to_string());
        }
        self.gods = compute_gods(self, god_n.max(1));
    }
}

/// What a function body reaches at the foreign boundary: declared-C-symbol calls and literal
/// Python imports. Collected by a read-only walk of the ALREADY-CHECKED AST (never re-lexed).
#[derive(Default)]
struct ForeignReach {
    c_symbols: BTreeSet<String>,
    py_imports: BTreeSet<String>,
}

fn walk_block_for_foreign(b: &Block, c_syms: &BTreeSet<String>, out: &mut ForeignReach) {
    for s in &b.stmts {
        match s {
            Stmt::Let { value, .. } => walk_expr_for_foreign(value, c_syms, out),
            Stmt::Assign { value, .. } => walk_expr_for_foreign(value, c_syms, out),
            Stmt::While { cond, body, .. } => {
                walk_expr_for_foreign(cond, c_syms, out);
                walk_block_for_foreign(body, c_syms, out);
            }
            Stmt::For { iter, body, .. } => {
                walk_expr_for_foreign(iter, c_syms, out);
                walk_block_for_foreign(body, c_syms, out);
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Return { value: Some(v), .. } => walk_expr_for_foreign(v, c_syms, out),
            Stmt::Return { value: None, .. } => {}
            Stmt::Expr(e) => walk_expr_for_foreign(e, c_syms, out),
        }
    }
}

fn walk_expr_for_foreign(e: &Expr, c_syms: &BTreeSet<String>, out: &mut ForeignReach) {
    match e {
        Expr::Method { recv, name, args, .. } => {
            // A call of a symbol this module's `foreign` block declares (the checker verified the
            // receiver is the lib handle — method resolution on foreign types is by symbol name).
            if c_syms.contains(&name.name) {
                out.c_symbols.insert(name.name.clone());
            }
            // `py.import("literal")` — the statically-known Python boundary (same rule as the
            // authority report's `imports_seen`).
            if name.name == "import" {
                if let Some(Expr::Lit { kind: LitKind::Str(s), .. }) = args.first() {
                    out.py_imports.insert(s.clone());
                }
            }
            walk_expr_for_foreign(recv, c_syms, out);
            for a in args {
                walk_expr_for_foreign(a, c_syms, out);
            }
        }
        Expr::Call { callee, args, .. } => {
            walk_expr_for_foreign(callee, c_syms, out);
            for a in args {
                walk_expr_for_foreign(a, c_syms, out);
            }
        }
        Expr::List { items, .. } => {
            for i in items {
                walk_expr_for_foreign(i, c_syms, out);
            }
        }
        Expr::Record { fields, .. } => {
            for (_, v) in fields {
                walk_expr_for_foreign(v, c_syms, out);
            }
        }
        Expr::Field { recv, .. } => walk_expr_for_foreign(recv, c_syms, out),
        Expr::Index { recv, index, .. } => {
            walk_expr_for_foreign(recv, c_syms, out);
            walk_expr_for_foreign(index, c_syms, out);
        }
        Expr::Unary { operand, .. } => walk_expr_for_foreign(operand, c_syms, out),
        Expr::Binary { lhs, rhs, .. } => {
            walk_expr_for_foreign(lhs, c_syms, out);
            walk_expr_for_foreign(rhs, c_syms, out);
        }
        Expr::If { cond, then_, else_, .. } => {
            walk_expr_for_foreign(cond, c_syms, out);
            walk_block_for_foreign(then_, c_syms, out);
            if let Some(e2) = else_ {
                walk_expr_for_foreign(e2, c_syms, out);
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            walk_expr_for_foreign(scrutinee, c_syms, out);
            for arm in arms {
                walk_expr_for_foreign(&arm.body, c_syms, out);
            }
        }
        Expr::Lambda { body, .. } => walk_block_for_foreign(body, c_syms, out),
        Expr::Try { inner, .. } => walk_expr_for_foreign(inner, c_syms, out),
        Expr::Block(b) => walk_block_for_foreign(b, c_syms, out),
        // Stage 7: spawn/send arguments can reach foreign symbols like any call arguments.
        Expr::Spawn { args, .. } => {
            for a in args {
                walk_expr_for_foreign(a, c_syms, out);
            }
        }
        Expr::Recover { body, .. } => walk_block_for_foreign(body, c_syms, out),
        Expr::Lit { .. } | Expr::Var { .. } | Expr::Consume { .. } => {}
    }
}

fn span_loc(map: &SourceMap, span: delulu_diag::Span) -> SpanLoc {
    let (line, _col) = map.position(span.file, span.start);
    SpanLoc { file: map.name(span.file).to_string(), line }
}

/// Recompute god nodes after a caller mutates the graph (e.g. attaching the custody overlay).
/// Identical ranking to the builder's: top-N by degree, ties by id.
pub fn recompute_gods(atlas: &Atlas, n: usize) -> Vec<God> {
    compute_gods(atlas, n)
}

/// Top-N nodes by degree (in + out), ties broken by id for determinism.
fn compute_gods(atlas: &Atlas, n: usize) -> Vec<God> {
    let mut degree: BTreeMap<&str, usize> = BTreeMap::new();
    for e in &atlas.edges {
        *degree.entry(e.from.as_str()).or_insert(0) += 1;
        *degree.entry(e.to.as_str()).or_insert(0) += 1;
    }
    let mut ranked: Vec<God> = atlas
        .nodes
        .iter()
        .map(|node| God {
            id: node.id.clone(),
            name: node.name.clone(),
            kind: node.kind,
            degree: degree.get(node.id.as_str()).copied().unwrap_or(0),
        })
        .collect();
    // Highest degree first; ties by id ascending (ranked is already id-sorted, so a stable sort by
    // descending degree keeps id order within a tie).
    ranked.sort_by(|a, b| b.degree.cmp(&a.degree).then(a.id.cmp(&b.id)));
    ranked.retain(|g| g.degree > 0);
    ranked.truncate(n);
    ranked
}
