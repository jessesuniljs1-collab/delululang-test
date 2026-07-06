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

pub struct DeclTable {
    pub types: Vec<TypeDef>,
    pub type_ix: HashMap<String, TypeDefId>,
    pub user_effects: HashSet<String>,
    pub fns: HashMap<String, FnSig>,
    pub consts: HashMap<String, ConstSig>,
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
        fn_order: Vec::new(),
    };

    // Prelude sum types (Stage-1 stdlib surface, §11): IoErr, NetErr.
    register_prelude_type(&mut table, "IoErr", &[("NotFound", &[]), ("Denied", &[]), ("Other", &["Str"])]);
    register_prelude_type(&mut table, "NetErr", &[("Refused", &[]), ("Timeout", &[]), ("Other", &["Str"])]);

    // First pass: type names (so signatures can forward-reference them).
    for item in &module.items {
        if let Item::Type(td) = item {
            if table.type_ix.contains_key(&td.name.name) {
                diags.push(dup("type", &td.name));
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
            if !table.user_effects.insert(ed.name.name.clone()) {
                diags.push(dup("effect", &ed.name));
            }
        }
    }

    // Functions and consts (shared value namespace).
    for item in &module.items {
        match item {
            Item::Fn(f) => {
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
    }
}

fn walk_row(r: &RowExpr, gset: &HashSet<String>, row_used: &mut HashSet<String>) {
    if let Some(tail) = &r.tail {
        if gset.contains(&tail.name) {
            row_used.insert(tail.name.clone());
        }
    }
}
