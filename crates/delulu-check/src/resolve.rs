//! Name resolution (spec §5): build the declaration table for a module, detect duplicates
//! (DL0302), and classify each function's generics as type- or row-kinded (DL0410). Type
//! lowering and the judgment live in `check.rs`; this pass is pure over the AST.

use std::collections::{HashMap, HashSet};

use delulu_diag::Diagnostic;
use delulu_syntax::ast::*;

use crate::ty::TypeDefId;

/// Kind of a generic parameter: a type variable or a row variable (§6.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GKind {
    Type,
    Row,
}

#[derive(Clone, Debug)]
pub enum TypeDefKind {
    Record(Vec<(String, TypeExpr)>),
    Sum(Vec<(String, Vec<TypeExpr>)>),
    Alias(TypeExpr),
}

#[derive(Clone, Debug)]
pub struct TypeDef {
    pub name: String,
    pub generics: Vec<String>,
    pub kind: TypeDefKind,
}

/// A function's signature, kept so call sites can re-lower it with fresh inference
/// variables (per-call-site instantiation, §5).
#[derive(Clone, Debug)]
pub struct FnSig {
    pub name: String,
    pub public: bool,
    pub generics: Vec<String>,
    pub gkinds: HashMap<String, GKind>,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub row: Option<RowExpr>,
}

#[derive(Clone, Debug)]
pub struct ConstSig {
    pub name: String,
    pub ty: Option<TypeExpr>,
}

/// One function declared inside a `foreign` block (spec §2). Kept so T-ForeignCall can re-lower
/// the signature at each `m.method(..)` call site.
#[derive(Clone, Debug)]
pub struct ForeignFnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub span: delulu_diag::Span,
}

/// A resolved `foreign <abi> lib <name> { … }` block. `name` is both the nominal opaque handle
/// type (`Type::Foreign(name)`) and the logical grant name.
#[derive(Clone, Debug)]
pub struct ForeignDef {
    pub abi: String,
    pub name: String,
    pub fns: Vec<ForeignFnDef>,
}

/// One `be` behavior of an actor (Stage 7, spec §2). Behaviors return `Unit` structurally;
/// the row is what the SEND site's row must contain (T-Send, `{Async} ∪ row(beh)`).
#[derive(Clone, Debug)]
pub struct ActorBehavior {
    pub name: String,
    pub params: Vec<Param>,
    pub row: Option<RowExpr>,
    pub span: delulu_diag::Span,
}

/// A resolved `actor A { … }` declaration (Stage 7, spec §2). Shares the type namespace
/// (an actor name is a type name — `Type::Actor`).
#[derive(Clone, Debug)]
pub struct ActorDef {
    pub name: String,
    pub generics: Vec<String>,
    /// (name, declared type, mutable) — fields are actor-internal state, assigned in `new`.
    pub fields: Vec<(String, TypeExpr, bool)>,
    pub ctor_params: Vec<Param>,
    pub ctor_row: Option<RowExpr>,
    pub behaviors: Vec<ActorBehavior>,
    /// Sync methods — callable only from `self` (T-SyncMethod; the rcap pass enforces it).
    pub fns: Vec<FnSig>,
}

impl ActorDef {
    pub fn behavior(&self, name: &str) -> Option<&ActorBehavior> {
        self.behaviors.iter().find(|b| b.name == name)
    }
    pub fn sync_fn(&self, name: &str) -> Option<&FnSig> {
        self.fns.iter().find(|f| f.name == name)
    }
}

#[derive(Debug)]
pub struct DeclTable {
    pub types: Vec<TypeDef>,
    pub type_ix: HashMap<String, TypeDefId>,
    pub user_effects: HashSet<String>,
    pub fns: HashMap<String, FnSig>,
    pub consts: HashMap<String, ConstSig>,
    /// `foreign` blocks by lib name (spec §2). Each name is also an opaque `Type::Foreign`.
    pub foreigns: HashMap<String, ForeignDef>,
    /// `actor` declarations by name (Stage 7, spec §2) — also type-namespace citizens.
    pub actors: HashMap<String, ActorDef>,
    /// Insertion order of functions, for deterministic checking and reporting.
    pub fn_order: Vec<String>,
}

impl DeclTable {
    pub fn type_def(&self, id: TypeDefId) -> &TypeDef {
        &self.types[id.0 as usize]
    }

    /// TypeDefIds of the prelude sum types, so the checker can name them.
    pub fn io_err(&self) -> TypeDefId {
        self.type_ix["IoErr"]
    }
    pub fn net_err(&self) -> TypeDefId {
        self.type_ix["NetErr"]
    }
    /// `std.foreign` error sum (Stage 4, spec §8).
    pub fn foreign_err(&self) -> TypeDefId {
        self.type_ix["ForeignErr"]
    }
    /// `std.py` error record (Stage 4, spec §5.2).
    pub fn py_err(&self) -> TypeDefId {
        self.type_ix["PyErr"]
    }
    /// The actuation error sum (Stage 10, 10e — spec §5.1).
    pub fn actuate_err(&self) -> TypeDefId {
        self.type_ix["ActuateErr"]
    }
    /// The compute-dispatch error sum (Stage 10, 10h — spec §7.1).
    pub fn compute_err(&self) -> TypeDefId {
        self.type_ix["ComputeErr"]
    }

    /// Resolve a bare constructor name to the UNIQUE sum type that declares it, with the variant's
    /// declared field types. `None` if no sum type has the constructor, or if more than one does —
    /// e.g. `Other` is a variant of both `IoErr` and `NetErr`, so bare `Other(..)` is ambiguous and
    /// must be disambiguated by the expected type (a later refinement), not synthesised here.
    pub fn variant_ctor(&self, ctor: &str) -> Option<(TypeDefId, Vec<TypeExpr>)> {
        let mut found: Option<(TypeDefId, Vec<TypeExpr>)> = None;
        for (i, def) in self.types.iter().enumerate() {
            if let TypeDefKind::Sum(variants) = &def.kind {
                if let Some((_, fields)) = variants.iter().find(|(n, _)| n == ctor) {
                    if found.is_some() {
                        return None; // ambiguous across sum types
                    }
                    found = Some((TypeDefId(i as u32), fields.clone()));
                }
            }
        }
        found
    }
}

/// Build the declaration table (with prelude types) and collect resolution diagnostics.
pub fn resolve(module: &Module) -> (DeclTable, Vec<Diagnostic>) {
    let mut diags = Vec::new();
    let mut table = DeclTable {
        types: Vec::new(),
        type_ix: HashMap::new(),
        user_effects: HashSet::new(),
        fns: HashMap::new(),
        consts: HashMap::new(),
        foreigns: HashMap::new(),
        actors: HashMap::new(),
        fn_order: Vec::new(),
    };

    // Prelude sum types (Stage-1 stdlib surface, §11): IoErr, NetErr.
    register_prelude_type(&mut table, "IoErr", &[("NotFound", &[]), ("Denied", &[]), ("Other", &["Str"])]);
    register_prelude_type(&mut table, "NetErr", &[("Refused", &[]), ("Timeout", &[]), ("Other", &["Str"])]);
    // Stage-4 foreign stdlib surface (spec §8): `std.foreign.ForeignErr` (sum) and
    // `std.py.PyErr` (record). `ForeignPtr`/`PyObj` and the per-block lib handle `M` are
    // opaque `Type` variants, registered by `check.rs`, not TypeDef entries.
    register_prelude_type(
        &mut table,
        "ForeignErr",
        &[("NotGranted", &[]), ("SymbolMissing", &["Str"]), ("BadReturn", &["Str"]), ("Unavailable", &["Str"])],
    );
    register_prelude_record(&mut table, "PyErr", &[("kind", "Str"), ("message", "Str")]);
    // Stage-6 `std.plugin` surface (spec §4). `Grant`/`Limits` are ORDINARY records and
    // `PluginErr` an ordinary sum: they *describe* authority, they do not confer it — conferral
    // happens only at `load`, under the holder check. Being ordinary is the point: a program can
    // build, inspect, and narrow a Grant with no special powers at all.
    register_plugin_prelude(&mut table);
    // Stage 10 (10e): `ActuateErr` — appended LAST, mirroring `program.rs` exactly (the
    // two-paths order law).
    register_prelude_type(
        &mut table,
        "ActuateErr",
        &[("Envelope", &["Str"]), ("LeaseRevoked", &["Str"]), ("NoDevice", &[])],
    );
    // Stage 10 (10h, Track F): `ComputeErr` — appended after `ActuateErr` in BOTH paths, because
    // the order law pins every prelude type's id by position.
    register_prelude_type(
        &mut table,
        "ComputeErr",
        &[("KernelEnvelope", &["Str"]), ("UnknownKernel", &["Str"]), ("NoAdapter", &[])],
    );

    // First pass: type names (so signatures can forward-reference them).
    for item in &module.items {
        if let Item::Type(td) = item {
            if table.type_ix.contains_key(&td.name.name) {
                diags.push(dup("type", &td.name));
                continue;
            }
            // C23: a builtin type name is intercepted by `lower_type` before user scope, so
            // registering this definition would leave it permanently unreachable.
            if let Some(d) = shadows_a_builtin_type("type", &td.name) {
                diags.push(d);
                continue;
            }
            let id = TypeDefId(table.types.len() as u32);
            let kind = match &td.kind {
                TypeDeclKind::Record(fields) => TypeDefKind::Record(
                    fields.iter().map(|f| (f.name.name.clone(), f.ty.clone())).collect(),
                ),
                TypeDeclKind::Sum(variants) => TypeDefKind::Sum(
                    variants
                        .iter()
                        .map(|v| (v.name.name.clone(), v.fields.clone()))
                        .collect(),
                ),
                TypeDeclKind::Alias(t) => TypeDefKind::Alias(t.clone()),
            };
            table.types.push(TypeDef { name: td.name.name.clone(), generics: gen_names(&td.generics), kind });
            table.type_ix.insert(td.name.name.clone(), id);
        }
    }

    // Effects.
    for item in &module.items {
        if let Item::Effect(ed) = item {
            // C23: a core effect name is resolved by `lower_row` before `user_effects`, so this
            // declaration would be inert — and effects are the authority axis.
            if let Some(d) = shadows_a_core_effect(&ed.name) {
                diags.push(d);
                continue;
            }
            if !table.user_effects.insert(ed.name.name.clone()) {
                diags.push(dup("effect", &ed.name));
            }
        }
    }

    // Foreign blocks (Stage 4, spec §2): each introduces a nominal opaque lib type `M` — sharing
    // the type namespace — plus the block's foreign functions (for T-ForeignCall).
    for item in &module.items {
        if let Item::Foreign(fd) = item {
            let name = fd.name.name.clone();
            if table.type_ix.contains_key(&name) || table.foreigns.contains_key(&name) {
                diags.push(dup("type", &fd.name));
                continue;
            }
            // C23: the lib name joins the TYPE namespace, and `lower_type` matches builtins first —
            // `foreign "c" lib Int` would produce a handle type no signature could ever name.
            if let Some(d) = shadows_a_builtin_type("foreign lib", &fd.name) {
                diags.push(d);
                continue;
            }
            let fns = fd
                .fns
                .iter()
                .map(|f| ForeignFnDef {
                    name: f.name.name.clone(),
                    params: f.params.clone(),
                    ret: f.ret.clone(),
                    span: f.span,
                })
                .collect();
            table.foreigns.insert(name.clone(), ForeignDef { abi: fd.abi.clone(), name, fns });
        }
    }

    // `std.actors` (Stage 7 phase 7j, spec §8): the stdlib Promise actor, written in
    // DeluluLang and injected as a PRELUDE actor wherever actors exist (v0.7 actors are
    // single-module; a source-level `import std.actors` lands with multi-module actors,
    // build-order §5). Registered before user actors so a user `actor Promise` is a dup.
    {
        let (std_mod, std_diags) = delulu_syntax::parse_file(u32::MAX, STD_ACTORS_SRC);
        debug_assert!(
            !std_diags.iter().any(|d| d.is_error()),
            "std.actors must parse clean: {std_diags:?}"
        );
        for item in &std_mod.items {
            if let Item::Actor(a) = item {
                table.actors.insert(a.name.name.clone(), actor_def_of(a));
            }
        }
    }

    // Actors (Stage 7, spec §2): the actor name joins the TYPE namespace (`Type::Actor`).
    for item in &module.items {
        if let Item::Actor(a) = item {
            let name = a.name.name.clone();
            if table.type_ix.contains_key(&name)
                || table.foreigns.contains_key(&name)
                || table.actors.contains_key(&name)
            {
                diags.push(dup("type", &a.name));
                continue;
            }
            // C23: the actor name joins the TYPE namespace too, and is matched after the builtins.
            if let Some(d) = shadows_a_builtin_type("actor", &a.name) {
                diags.push(d);
                continue;
            }
            table.actors.insert(name, actor_def_of(a));
        }
    }

    // Functions and consts (shared value namespace).
    for item in &module.items {
        match item {
            Item::Fn(f) => {
                if let Some(d) = shadows_a_builtin("function", &f.name) {
                    diags.push(d);
                    continue;
                }
                if table.fns.contains_key(&f.name.name) || table.consts.contains_key(&f.name.name) {
                    diags.push(dup("value", &f.name));
                    continue;
                }
                let generics = gen_names(&f.generics);
                let (gkinds, mut gdiags) = classify_generics(&generics, f);
                diags.append(&mut gdiags);
                table.fn_order.push(f.name.name.clone());
                table.fns.insert(
                    f.name.name.clone(),
                    FnSig {
                        name: f.name.name.clone(),
                        public: f.public,
                        generics,
                        gkinds,
                        params: f.params.clone(),
                        ret: f.ret.clone(),
                        row: f.row.clone(),
                    },
                );
            }
            Item::Const(c) => {
                if let Some(d) = shadows_a_builtin("constant", &c.name) {
                    diags.push(d);
                    continue;
                }
                if table.fns.contains_key(&c.name.name) || table.consts.contains_key(&c.name.name) {
                    diags.push(dup("value", &c.name));
                    continue;
                }
                table.consts.insert(c.name.name.clone(), ConstSig { name: c.name.name.clone(), ty: c.ty.clone() });
            }
            _ => {}
        }
    }

    (table, diags)
}

fn register_prelude_type(table: &mut DeclTable, name: &str, variants: &[(&str, &[&str])]) {
    let id = TypeDefId(table.types.len() as u32);
    let vs = variants
        .iter()
        .map(|(vn, fields)| {
            let ftys = fields
                .iter()
                .map(|f| TypeExpr::Named {
                    path: Path { segs: vec![Ident { name: (*f).to_string(), span: dummy_span() }] },
                    args: vec![],
                    span: dummy_span(),
                })
                .collect();
            ((*vn).to_string(), ftys)
        })
        .collect();
    table.types.push(TypeDef { name: name.to_string(), generics: vec![], kind: TypeDefKind::Sum(vs) });
    table.type_ix.insert(name.to_string(), id);
}

/// Register the `std.plugin` prelude (spec §4): `Limits`, `Grant`, `PluginErr`. Shared by the
/// single-module (`resolve`) and whole-program (`program::push_prelude`) paths so a `TypeDefId`
/// means the same thing on both.
pub(crate) fn register_plugin_prelude(table: &mut DeclTable) {
    register_prelude_record(
        table,
        "Limits",
        &[("fuel", "Int"), ("mem_mb", "Int"), ("wall_ms", "Int")],
    );
    // `Grant` carries List[Str] scope dimensions plus a nested `Limits` and `require_signed`.
    let id = TypeDefId(table.types.len() as u32);
    table.types.push(TypeDef {
        name: "Grant".to_string(),
        generics: vec![],
        kind: TypeDefKind::Record(vec![
            ("effects".to_string(), list_of_str()),
            ("fs_read".to_string(), list_of_str()),
            ("fs_write".to_string(), list_of_str()),
            ("net".to_string(), list_of_str()),
            ("secrets".to_string(), list_of_str()),
            ("declassify".to_string(), list_of_str()),
            ("limits".to_string(), named("Limits")),
            ("require_signed".to_string(), named("Bool")),
        ]),
    });
    table.type_ix.insert("Grant".to_string(), id);
    register_prelude_type(
        table,
        "PluginErr",
        &[
            ("NotGranted", &["Str"]),
            ("VerifyFailed", &["Str"]),
            ("BadArtifact", &["Str"]),
            ("Revoked", &["Int"]),
            ("LimitExceeded", &["Str"]),
            ("ApiMismatch", &["Str"]),
        ],
    );
}

pub(crate) fn named(name: &str) -> TypeExpr {
    TypeExpr::Named {
        path: Path { segs: vec![Ident { name: name.to_string(), span: dummy_span() }] },
        args: vec![],
        span: dummy_span(),
    }
}

pub(crate) fn list_of_str() -> TypeExpr {
    TypeExpr::Named {
        path: Path { segs: vec![Ident { name: "List".to_string(), span: dummy_span() }] },
        args: vec![named("Str")],
        span: dummy_span(),
    }
}

fn register_prelude_record(table: &mut DeclTable, name: &str, fields: &[(&str, &str)]) {
    let id = TypeDefId(table.types.len() as u32);
    let fs = fields
        .iter()
        .map(|(fname, fty)| {
            (
                (*fname).to_string(),
                TypeExpr::Named {
                    path: Path { segs: vec![Ident { name: (*fty).to_string(), span: dummy_span() }] },
                    args: vec![],
                    span: dummy_span(),
                },
            )
        })
        .collect();
    table.types.push(TypeDef { name: name.to_string(), generics: vec![], kind: TypeDefKind::Record(fs) });
    table.type_ix.insert(name.to_string(), id);
}

fn dummy_span() -> delulu_diag::Span {
    delulu_diag::Span::new(u32::MAX, 0, 0)
}

fn gen_names(gs: &[Ident]) -> Vec<String> {
    gs.iter().map(|g| g.name.clone()).collect()
}

fn dup(what: &str, name: &Ident) -> Diagnostic {
    Diagnostic::error("DL0302", format!("duplicate {what} definition `{}`", name.name))
        .with_span(name.span, "already defined")
}

/// Refuse a declaration that reuses a prelude builtin's name (DL0302).
///
/// Builtins are resolved at the CALL site before user scope, so without this the declaration
/// was accepted and then never called: every call went to the builtin. The author saw a type
/// error at some *other* line, mentioning a type they never wrote (`Option` for a function they
/// declared as returning `Result`), with nothing anywhere naming the collision. That is the
/// worst shape a diagnostic can take — correct, distant, and about the wrong thing.
///
/// Refusing is the right resolution rather than letting the user's definition win: a call would
/// otherwise mean different things depending on which module it appears in. See
/// `HARDENING_CAMPAIGN.md` C11.
fn shadows_a_builtin(what: &str, name: &Ident) -> Option<Diagnostic> {
    if !crate::check::PRELUDE_BUILTINS.contains(&name.name.as_str()) {
        return None;
    }
    Some(
        Diagnostic::error(
            "DL0302",
            format!("`{}` is a prelude builtin and cannot be redefined as a {what}", name.name),
        )
        .with_span(
            name.span,
            "calls resolve to the builtin before user scope, so this definition would never be called — rename it",
        ),
    )
}

/// Refuse a type/actor/foreign-lib declaration that reuses a builtin TYPE name (DL0302, C23).
///
/// The type resolver matches `Int`, `Root`, `Cap`, `Secret`, … before it searches user scope, so
/// such a declaration is not merely shadowed — it is *inert*. Accepting it means a source file can
/// state `type Cap = Int` and be read, reasonably, as evidence that `Cap[FsRead]` denotes something
/// the author defined. The declaration must fail where it is written.
///
/// Sibling of [`shadows_a_builtin`] (C11, the value namespace); same code, same reasoning, same
/// resolution — refuse rather than pick a winner, because either winner makes one name mean two
/// things depending on where it is read.
pub(crate) fn shadows_a_builtin_type(what: &str, name: &Ident) -> Option<Diagnostic> {
    if !crate::check::PRELUDE_TYPES.contains(&name.name.as_str()) {
        return None;
    }
    Some(
        Diagnostic::error(
            "DL0302",
            format!("`{}` is a builtin type and cannot be redefined as a {what}", name.name),
        )
        .with_span(
            name.span,
            "types resolve to the builtin before user scope, so this definition would never take effect — rename it",
        ),
    )
}

/// Refuse an `effect` declaration that reuses a CORE effect name (DL0302, C23).
///
/// `lower_row` resolves core effects before `user_effects`, so `effect Write` was inert in exactly
/// the way [`shadows_a_builtin_type`] describes — with the added hazard that effects are the
/// authority axis: the author believes they declared something private, while every `! {Write}` in
/// the module continues to mean the effect that reaches the filesystem and the console.
pub(crate) fn shadows_a_core_effect(name: &Ident) -> Option<Diagnostic> {
    if !crate::check::CORE_EFFECT_NAMES.contains(&name.name.as_str()) {
        return None;
    }
    Some(
        Diagnostic::error(
            "DL0302",
            format!("`{}` is a core effect and cannot be redeclared", name.name),
        )
        .with_span(
            name.span,
            "rows resolve core effects before user effects, so this declaration would never take effect — rename it",
        ),
    )
}

/// Classify a function's generics (DL0410 if a name is used as both a type and a row).
pub fn classify_generics(generics: &[String], f: &FnDecl) -> (HashMap<String, GKind>, Vec<Diagnostic>) {
    let gset: HashSet<String> = generics.iter().cloned().collect();
    let mut type_used = HashSet::new();
    let mut row_used = HashSet::new();

    for p in &f.params {
        walk_type(&p.ty, &gset, &mut type_used, &mut row_used);
    }
    if let Some(ret) = &f.ret {
        walk_type(ret, &gset, &mut type_used, &mut row_used);
    }
    if let Some(row) = &f.row {
        walk_row(row, &gset, &mut row_used);
    }

    let mut kinds = HashMap::new();
    let mut diags = Vec::new();
    for g in generics {
        let as_type = type_used.contains(g);
        let as_row = row_used.contains(g);
        if as_type && as_row {
            diags.push(
                Diagnostic::error("DL0410", format!("generic `{g}` is used as both a type and an effect row"))
                    .with_span(f.name.span, "declared here"),
            );
            kinds.insert(g.clone(), GKind::Type);
        } else if as_row {
            kinds.insert(g.clone(), GKind::Row);
        } else {
            // Default to Type (unused generics are harmless).
            kinds.insert(g.clone(), GKind::Type);
        }
    }
    (kinds, diags)
}

fn walk_type(t: &TypeExpr, gset: &HashSet<String>, type_used: &mut HashSet<String>, row_used: &mut HashSet<String>) {
    match t {
        TypeExpr::Named { path, args, .. } => {
            if path.segs.len() == 1 && args.is_empty() && gset.contains(&path.segs[0].name) {
                type_used.insert(path.segs[0].name.clone());
            }
            for a in args {
                walk_type(a, gset, type_used, row_used);
            }
        }
        TypeExpr::Fn { params, ret, row, .. } => {
            for p in params {
                walk_type(p, gset, type_used, row_used);
            }
            if let Some(r) = ret {
                walk_type(r, gset, type_used, row_used);
            }
            if let Some(row) = row {
                walk_row(row, gset, row_used);
            }
        }
        TypeExpr::Rcap { inner, .. } => walk_type(inner, gset, type_used, row_used),
    }
}

fn walk_row(r: &RowExpr, gset: &HashSet<String>, row_used: &mut HashSet<String>) {
    if let Some(tail) = &r.tail {
        if gset.contains(&tail.name) {
            row_used.insert(tail.name.clone());
        }
    }
}

/// The `std.actors` stdlib source (Stage 7, spec §8): `Promise[T, e]` — a library ACTOR,
/// not a language feature. `T` must be sendable (`val`-carried, written at every use);
/// `e` is the row the stored callbacks may perform: it binds at `then` sites through the
/// callback argument (the R-4 law applied to a stdlib actor — the send site of `then`
/// carries `{Async} ∪ e`), while `fulfill` — which cannot constrain `e` from any argument
/// — contributes only `{Async}` at its send sites (the spec's own accounting: "row e joins
/// THEN's send row"). First fulfill wins; later ones are dropped and counted.
pub const STD_ACTORS_SRC: &str = "module std.actors
actor Promise[T, e] {
  var value: Option[T]
  var callbacks: List[fn(val T) -> Unit ! e]
  var dropped_fulfills: Int
  new() {
    self.value = None
    self.callbacks = []
    self.dropped_fulfills = 0
  }
  be fulfill(v: val T) ! e {
    match self.value {
      Some(old) => { self.dropped_fulfills = self.dropped_fulfills + 1 },
      None => {
        self.value = Some(v)
        var i = 0
        while i < self.callbacks.len() {
          match self.callbacks.get(i) {
            Some(f) => f(v),
            None => { }
          }
          i = i + 1
        }
        self.callbacks = []
      }
    }
  }
  be then(f: val fn(val T) -> Unit ! e) ! e {
    match self.value {
      Some(v2) => f(v2),
      None => { push(self.callbacks, f) }
    }
  }
}
";

/// Build the resolved [`ActorDef`] for one `actor` declaration (shared by user actors and
/// the injected `std.actors` prelude).
fn actor_def_of(a: &ActorDecl) -> ActorDef {
    ActorDef {
        name: a.name.name.clone(),
        generics: gen_names(&a.generics),
        fields: a.fields.iter().map(|fd| (fd.name.name.clone(), fd.ty.clone(), fd.mutable)).collect(),
        ctor_params: a.ctor.params.clone(),
        ctor_row: a.ctor.row.clone(),
        behaviors: a
            .behaviors
            .iter()
            .map(|b| ActorBehavior {
                name: b.name.name.clone(),
                params: b.params.clone(),
                row: b.row.clone(),
                span: b.span,
            })
            .collect(),
        fns: a
            .fns
            .iter()
            .map(|f| FnSig {
                name: f.name.name.clone(),
                public: false,
                generics: gen_names(&f.generics),
                gkinds: HashMap::new(),
                params: f.params.clone(),
                ret: f.ret.clone(),
                row: f.row.clone(),
            })
            .collect(),
    }
}
