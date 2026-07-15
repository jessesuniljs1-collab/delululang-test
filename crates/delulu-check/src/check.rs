//! The type & effect/authority judgment (spec §6.2–§6.5). THE HEART.
//!
//! Core soundness properties enforced here (audit §D, rules R-1…R-7):
//! - Effects arise ONLY from capability operations (T-CapOp) and `Secret.expose` (Declassify).
//!   A function holding no capability performs no effect; the row is a sound over-approximation.
//! - Rows are checked by *subset* at the function boundary (T-Fn): `ε_body ⊆ ε_declared`.
//!   Subsumption exists only here and at annotated lambdas — never inside unification (R-3).
//! - Capabilities are unforgeable: `Cap[R]`/`Root`/`Secret[T]` have no literal or constructor.
//! - Secrets do not launder: `Secret[T]` is not `T` (DL0602); opaque types have no
//!   `str`/`==`/serialization (DL0604/DL0605, R-5); `expose` carries `Declassify` (R-2).

use std::collections::{BTreeSet, HashMap, HashSet};

use delulu_diag::{Confidence, Diagnostic, Edit, Repair, Span};
use delulu_syntax::ast::*;

use crate::resolve::{DeclTable, FnSig, GKind, TypeDefKind};
use crate::ty::{Effect, ResourceKind, Row, RowVar, Type, TypeDefId};
use crate::unify::{InferCtx, UnifyError};

/// What the checker learned about one function, for the authority report and reachability.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FnFacts {
    pub effects: BTreeSet<Effect>,
    pub cap_kinds: BTreeSet<ResourceKind>,
    pub callees: BTreeSet<String>,
    pub uses_secret: bool,
    pub declassifies: bool,
    pub pure: bool,
    /// Secret names requested via `root.secret("NAME")` — for the authority report.
    pub secret_names: BTreeSet<String>,
}

pub struct CheckResult {
    pub diags: Vec<Diagnostic>,
    pub facts: HashMap<String, FnFacts>,
    pub main_row: Option<BTreeSet<Effect>>,
    pub main_present: bool,
    /// Type of the last top-level binding checked (for the REPL). Keyed by fn name.
    pub fn_types: HashMap<String, Type>,
    /// Which `foreign` lib each `root.foreign(load)` call site binds (Stage 4, phase 4d). The
    /// grammar has no method type-argument syntax, so `[M]` in `root.foreign[M](load)` is inferred
    /// from context; the interpreter needs that resolved name to know which library to dlopen and
    /// which symbols to resolve. Keyed by the `Expr::Method` node id of the `root.foreign(..)` call.
    pub foreign_binds: HashMap<NodeId, String>,
    /// The post-check typed AST's per-node resolved type (Stage 6 / DIR §2.3). Every expression
    /// and block node carries the fully-substituted type the checker assigned it — the side table
    /// the AST comment always promised. Populated after the substitution is settled; keyed by
    /// `NodeId`. This is what DIR serializes so a Verified plugin can be re-checked without
    /// re-inferring.
    pub node_types: HashMap<NodeId, Type>,
    /// The per-node resolved effect row, companion to `node_types` (DIR §2.3). The row is the
    /// resolved effect set the node performs; polymorphic tails that never bound to concrete
    /// effects are dropped (they contribute nothing observable — the boundary subset check in
    /// `check_fn` is the authoritative gate, and DIR re-verification replays it).
    pub node_rows: HashMap<NodeId, Row>,
}

/// Effect-row accumulator: concrete effects plus any polymorphic tails still in play.
/// Kept sound by resolving tails through the substitution before the boundary check.
#[derive(Clone, Default)]
struct RowAcc {
    effects: BTreeSet<Effect>,
    tails: BTreeSet<RowVar>,
}

impl RowAcc {
    fn add_effect(&mut self, e: Effect) {
        self.effects.insert(e);
    }
    fn add_row(&mut self, r: &Row) {
        self.effects.extend(r.effects.iter().cloned());
        if let Some(t) = r.tail {
            self.tails.insert(t);
        }
    }
    fn merge(&mut self, other: &RowAcc) {
        self.effects.extend(other.effects.iter().cloned());
        self.tails.extend(other.tails.iter().cloned());
    }
}

/// The genv maps a function's generics to fresh inference variables for this checking pass.
#[derive(Clone, Default)]
struct Genv {
    types: HashMap<String, Type>,
    rows: HashMap<String, RowVar>,
}

pub fn check_module(module: &Module, table: &DeclTable) -> CheckResult {
    let mut checker = Checker {
        table,
        cx: InferCtx::new(),
        diags: Vec::new(),
        facts: HashMap::new(),
        fn_types: HashMap::new(),
        pending_foreign_binds: Vec::new(),
        pending_gets: Vec::new(),
        node_types_raw: HashMap::new(),
        node_row_accs: HashMap::new(),
    };
    for item in &module.items {
        if let Item::Fn(f) = item {
            checker.check_fn(f);
        }
    }
    // The compile-time foreign fence (T-ForeignSig): every foreign signature must marshal.
    for item in &module.items {
        if let Item::Foreign(fd) = item {
            checker.check_foreign_decl(fd);
        }
    }
    // Resolve each `root.foreign(load)` call site's inferred lib type `M` now that the whole
    // program's substitution is settled (the annotation/use that pins `M` may occur anywhere in the
    // function body). A call whose handle never resolves to a concrete `Type::Foreign` is skipped —
    // such a program cannot call a method on the handle either, so the interpreter never needs it.
    let mut foreign_binds: HashMap<NodeId, String> = HashMap::new();
    for (id, tv) in &checker.pending_foreign_binds {
        if let Type::Foreign(name) = checker.cx.apply_type(&Type::Var(*tv)) {
            foreign_binds.insert(*id, name);
        }
    }
    // R-6a / DL0803 (Stage 6, audit F-6): a function-typed parameter ANYWHERE in `F` — including
    // nested inside a composite — at a `get` on a **Contained** plugin is a compile error. An
    // opaque module holding a funcref could invoke it at moments no caller's row accounts for, so
    // the rule is permanent, not a deferred feature. Decided here because the class and `F` are
    // only known once the whole module's substitution has settled.
    //
    // Verified plugins accept callbacks (rows composed per R-4) — their code is re-proved at load,
    // so the callback's row is real. The asymmetry IS the Verified/Contained split, in the types.
    let pending_gets = std::mem::take(&mut checker.pending_gets);
    for (span, f_var, class_ty) in pending_gets {
        if !matches!(checker.cx.apply_type(&class_ty), Type::Contained) {
            continue;
        }
        let f = checker.cx.apply_type(&Type::Var(f_var));
        let Type::Fn { params, .. } = &f else { continue };
        if params.iter().any(type_contains_fn) {
            checker.diags.push(
                Diagnostic::error(
                    "DL0803",
                    "a function-typed value cannot be passed to a Contained plugin export — no callbacks, by rule R-6a",
                )
                .with_span(span, "this `get` types an export of an opaque module")
                .with_secondary_span(span, "an opaque module holding a re-entry point into verified code could invoke it at times no caller's row accounts for"),
            );
        }
    }

    let main_present = table.fns.contains_key("main");
    let main_row = checker.facts.get("main").map(|f| f.effects.clone());

    // Settle the DIR side tables now that the whole substitution is fixed: resolve every recorded
    // node type through the final substitution, and collapse each node's raw effect accumulator to
    // its resolved effect set (DIR §2.3). This runs after all functions are checked so that later
    // unifications are reflected in earlier nodes' types.
    let mut node_types: HashMap<NodeId, Type> = HashMap::with_capacity(checker.node_types_raw.len());
    for (id, raw) in &checker.node_types_raw {
        node_types.insert(*id, checker.cx.apply_type(raw));
    }
    let mut node_rows: HashMap<NodeId, Row> = HashMap::with_capacity(checker.node_row_accs.len());
    for (id, acc) in &checker.node_row_accs {
        node_rows.insert(*id, checker.resolve_row_acc(acc));
    }

    CheckResult {
        diags: checker.diags,
        facts: checker.facts,
        main_row,
        main_present,
        fn_types: checker.fn_types,
        foreign_binds,
        node_types,
        node_rows,
    }
}

/// Lower a **concrete** type expression (no generics in scope) against a module's declaration
/// table — the Stage-6 entry point for plugin-manifest export signature strings (spec §2.1). This
/// reuses the checker's own `lower_type` rule code, so a manifest signature means *exactly* what
/// the same text means in source. Any lowering error (unknown type, bad arity, unknown effect,
/// row variable — signatures are monomorphic) is returned as `Err`; the caller maps it to DL1501.
pub fn lower_export_signature(t: &delulu_syntax::ast::TypeExpr, table: &DeclTable) -> Result<Type, String> {
    let mut checker = Checker {
        table,
        cx: InferCtx::new(),
        diags: Vec::new(),
        facts: HashMap::new(),
        fn_types: HashMap::new(),
        pending_foreign_binds: Vec::new(),
        pending_gets: Vec::new(),
        node_types_raw: HashMap::new(),
        node_row_accs: HashMap::new(),
    };
    let ty = checker.lower_type(t, &Genv::default(), &mut FnFacts::default());
    if let Some(d) = checker.diags.iter().find(|d| d.is_error()) {
        return Err(format!("{}: {}", d.code, d.message));
    }
    Ok(ty)
}

struct Checker<'a> {
    table: &'a DeclTable,
    cx: InferCtx,
    diags: Vec<Diagnostic>,
    facts: HashMap<String, FnFacts>,
    fn_types: HashMap<String, Type>,
    /// `root.foreign(load)` call sites and the fresh handle var each produced, resolved to a
    /// concrete lib name after all functions are checked (see `check_module`).
    pending_foreign_binds: Vec<(NodeId, crate::ty::TypeVar)>,
    /// `p.get(name)` call sites (Stage 6): the site's span, the fresh `F` variable the annotation
    /// will pin, and the receiver's class marker. R-6a/DL0803 is decided once inference settles
    /// (see `check_module`), because neither `F` nor the class is known when the method is checked.
    pending_gets: Vec<(Span, crate::ty::TypeVar, Type)>,
    /// DIR §2.3 side tables, recorded raw (pre-substitution) during checking and resolved once at
    /// the end of `check_module`. `node_types_raw` holds each node's assigned type (possibly with
    /// inference variables); `node_row_accs` holds each node's raw effect accumulator.
    node_types_raw: HashMap<NodeId, Type>,
    node_row_accs: HashMap<NodeId, RowAcc>,
}

/// Per-function-body checking context.
struct FnCtx {
    #[allow(dead_code)]
    name: String,
    genv: Genv,
    /// The function's declared return type (for `?` and `return`).
    ret: Type,
    /// The error type extracted from a `Result` return, if any (for `?`).
    ret_err: Option<Type>,
    /// Local variable environment (name → type).
    locals: Vec<HashMap<String, Type>>,
    facts: FnFacts,
}

impl FnCtx {
    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.locals.pop();
    }
    fn bind(&mut self, name: &str, ty: Type) {
        self.locals.last_mut().unwrap().insert(name.to_string(), ty);
    }
    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.locals.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }
}

impl<'a> Checker<'a> {
    // ===== function checking ==============================================

    fn check_fn(&mut self, f: &FnDecl) {
        let sig = self.table.fns.get(&f.name.name).expect("fn in table").clone();
        let genv = self.make_genv(&sig);

        // Lower the signature.
        let param_types: Vec<Type> =
            f.params.iter().map(|p| self.lower_type(&p.ty, &genv, &mut FnFacts::default())).collect();
        let ret = match &f.ret {
            Some(t) => self.lower_type(t, &genv, &mut FnFacts::default()),
            None => Type::Unit,
        };
        let declared_row = match &f.row {
            Some(r) => self.lower_row(r, &genv),
            None => Row::pure(),
        };
        let ret_err = match &ret {
            Type::Result(_, e) => Some((**e).clone()),
            _ => None,
        };

        // Record the function's static type (rows never erase — invariant 3).
        let fn_ty = Type::Fn {
            params: param_types.clone(),
            ret: Box::new(ret.clone()),
            row: declared_row.clone(),
        };
        self.fn_types.insert(f.name.name.clone(), fn_ty);

        let mut ctx = FnCtx {
            name: f.name.name.clone(),
            genv,
            ret: ret.clone(),
            ret_err,
            locals: vec![HashMap::new()],
            facts: FnFacts::default(),
        };
        // Bind parameters, noting capability kinds they carry.
        for (p, ty) in f.params.iter().zip(&param_types) {
            self.note_caps_in(ty, &mut ctx.facts);
            ctx.bind(&p.name.name, ty.clone());
        }

        // Check the body; its value type must match the return type.
        let (body_ty, body_row) = self.check_block(&f.body, &mut ctx);
        self.expect_type(&ret, &body_ty, f.body.span, "function body type must match the return type");

        // Boundary check: ε_body ⊆ ε_declared (T-Fn), and unused declared effects (DL0502).
        self.check_row_subset(&body_row, &declared_row, f, &sig);

        // Finalize facts: the function's authority is its declared row (sound upper bound).
        let mut facts = std::mem::take(&mut ctx.facts);
        facts.effects = self.resolved_effects(&declared_row);
        facts.pure = facts.effects.is_empty();
        facts.declassifies = facts.effects.contains(&Effect::Declassify);
        self.facts.insert(f.name.name.clone(), facts);
    }

    // ===== the foreign marshallability fence (T-ForeignSig, spec §3) ======

    /// Check every parameter and return type of every function in a `foreign` block. This is the
    /// actual compile-time security surface of Stage 4 (playbook 4b): it must be bulletproof.
    fn check_foreign_decl(&mut self, fd: &ForeignDecl) {
        for f in &fd.fns {
            for p in &f.params {
                self.check_marshallable(&p.ty);
            }
            if let Some(ret) = &f.ret {
                self.check_marshallable(ret);
            }
        }
    }

    /// `M(τ)`: only `Int Float Bool Str Unit ForeignPtr` marshal (DL1301 otherwise, span on the
    /// offending type). A function type *anywhere* (including nested) is the no-callbacks rule
    /// (DL1302, R-6a) and takes priority over DL1301.
    ///
    /// Both diagnostics are emitted with **no repairs** — they are `requires_human` situations
    /// (spec §7), and DL1301 must NEVER suggest `expose` to launder a secret across the FFI
    /// (criterion 3). An empty repair list satisfies both, and matches how every other
    /// `requires_human` code in the compiler is emitted.
    fn check_marshallable(&mut self, t: &TypeExpr) {
        // R-6a / invariant 22: no function pointer crosses the boundary, at any depth → DL1302.
        if let Some(fspan) = first_fn_type(t) {
            self.diags.push(
                Diagnostic::error(
                    "DL1302",
                    "a function-typed value cannot cross the foreign boundary — no callbacks, by rule R-6a",
                )
                .with_span(fspan, "unverifiable code must never hold a re-entry point into verified code")
                .with_secondary_span(t.span(), "in this foreign signature type"),
            );
            return;
        }
        if !is_marshallable_type_expr(t) {
            self.diags.push(
                Diagnostic::error(
                    "DL1301",
                    format!(
                        "type `{}` cannot be marshalled across the foreign boundary",
                        render_type_expr(t)
                    ),
                )
                .with_span(t.span(), "only Int, Float, Bool, Str, Unit, and ForeignPtr marshal"),
            );
        }
    }

    fn make_genv(&mut self, sig: &FnSig) -> Genv {
        let mut genv = Genv::default();
        for g in &sig.generics {
            match sig.gkinds.get(g).copied().unwrap_or(GKind::Type) {
                GKind::Type => {
                    let v = self.cx.fresh_type();
                    genv.types.insert(g.clone(), v);
                }
                GKind::Row => {
                    let v = self.cx.fresh_row_var();
                    genv.rows.insert(g.clone(), v);
                }
            }
        }
        genv
    }

    /// The subset check at the function boundary (T-Fn). Emits DL0501 (undeclared effect,
    /// with an authority-widening repair) and DL0502 (declared-but-unused, narrowing repair).
    fn check_row_subset(&mut self, body: &RowAcc, declared: &Row, f: &FnDecl, _sig: &FnSig) {
        // Resolve body tails through the substitution; a tail equal to the declared row's tail
        // is covered, anything else contributes possibly-unknown effects.
        let declared_resolved = self.cx.apply_row(declared);
        let mut body_effects = body.effects.clone();
        let mut uncovered_tail = false;
        for &t in &body.tails {
            let r = self.cx.apply_row(&Row { effects: BTreeSet::new(), tail: Some(t) });
            body_effects.extend(r.effects.iter().cloned());
            if let Some(rt) = r.tail {
                if declared_resolved.tail != Some(rt) {
                    uncovered_tail = true;
                }
            }
        }

        let missing: Vec<Effect> =
            body_effects.difference(&declared_resolved.effects).cloned().collect();
        if !missing.is_empty() || uncovered_tail {
            let list = missing.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
            let msg = if missing.is_empty() {
                format!("function `{}` may perform an effect not declared in its row", f.name.name)
            } else {
                format!(
                    "function `{}` performs effect{} `{}` not declared in its row",
                    f.name.name,
                    if missing.len() == 1 { "" } else { "s" },
                    list
                )
            };
            let mut diag = Diagnostic::error("DL0501", msg)
                .with_span(f.name.span, "declared row is here");
            if !missing.is_empty() {
                diag = diag.with_repair(self.add_effect_repair(f, &declared_resolved, &missing));
            }
            self.diags.push(diag);
        }

        // DL0502: a declared concrete effect the body never performs (narrowing → safe).
        let unused: Vec<Effect> =
            declared_resolved.effects.difference(&body_effects).cloned().collect();
        if !unused.is_empty() && f.row.is_some() {
            let list = unused.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
            self.diags.push(
                Diagnostic::warning(
                    "DL0502",
                    format!("function `{}` declares effect{} `{}` it never performs", f.name.name,
                        if unused.len() == 1 { "" } else { "s" }, list),
                )
                .with_span(f.row.as_ref().unwrap().span, "declared here")
                .with_repair(Repair {
                    id: "remove_effect_from_row",
                    confidence: Confidence::Safe,
                    authority_widening: false,
                    requires_human: false,
                    edits: vec![],
                }),
            );
        }
    }

    fn add_effect_repair(&self, f: &FnDecl, declared: &Row, missing: &[Effect]) -> Repair {
        let add = missing.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
        // Insert into an existing `!{...}` or add a fresh row after the signature.
        let (start, end, text) = match &f.row {
            Some(r) => {
                // Insert before the closing brace of the row span.
                let insert_at = r.span.end.saturating_sub(1);
                let sep = if declared.effects.is_empty() { "" } else { ", " };
                (insert_at, insert_at, format!("{sep}{add}"))
            }
            None => {
                let at = f.body.span.start;
                (at, at, format!("! {{{add}}} "))
            }
        };
        Repair {
            id: "add_effect_to_row",
            confidence: Confidence::Exact,
            authority_widening: true,
            requires_human: false,
            edits: vec![Edit { file: f.name.span.file, start_byte: start, end_byte: end, insert: text }],
        }
    }

    // ===== blocks and statements ==========================================

    fn check_block(&mut self, block: &Block, ctx: &mut FnCtx) -> (Type, RowAcc) {
        ctx.push_scope();
        let mut acc = RowAcc::default();
        let mut value_ty = Type::Unit;
        let n = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i + 1 == n;
            match stmt {
                Stmt::Let { name, ty, value, .. } => {
                    let (vt, vr) = self.check_expr(value, ctx);
                    acc.add_row_acc(&vr);
                    if let Some(ann) = ty {
                        let at = self.lower_type(ann, &ctx.genv, &mut ctx.facts);
                        self.expect_type(&at, &vt, value.span(), "let binding type mismatch");
                        ctx.bind(&name.name, at);
                    } else {
                        ctx.bind(&name.name, vt);
                    }
                    value_ty = Type::Unit;
                }
                Stmt::Assign { target, value, span } => {
                    let tt = self.check_lvalue(target, ctx);
                    let (vt, vr) = self.check_expr(value, ctx);
                    acc.add_row_acc(&vr);
                    self.expect_type(&tt, &vt, *span, "assignment type mismatch");
                    value_ty = Type::Unit;
                }
                Stmt::While { cond, body, .. } => {
                    let (ct, cr) = self.check_expr(cond, ctx);
                    acc.add_row_acc(&cr);
                    self.expect_type(&Type::Bool, &ct, cond.span(), "`while` condition must be Bool");
                    let (_, br) = self.check_block(body, ctx);
                    acc.add_row_acc(&br);
                    value_ty = Type::Unit;
                }
                Stmt::Return { value, span } => {
                    match value {
                        Some(e) => {
                            let (vt, vr) = self.check_expr(e, ctx);
                            acc.add_row_acc(&vr);
                            let ret = ctx.ret.clone();
                            self.expect_type(&ret, &vt, e.span(), "return type mismatch");
                        }
                        None => {
                            let ret = ctx.ret.clone();
                            self.expect_type(&ret, &Type::Unit, *span, "return type mismatch");
                        }
                    }
                    value_ty = Type::Unit;
                }
                Stmt::Expr(e) => {
                    let (t, r) = self.check_expr(e, ctx);
                    acc.add_row_acc(&r);
                    value_ty = if is_last { t } else { Type::Unit };
                }
            }
        }
        ctx.pop_scope();
        // Record the block node in the DIR side tables (§2.3): its value type and accumulated row.
        self.node_types_raw.insert(block.id, value_ty.clone());
        self.node_row_accs.insert(block.id, acc.clone());
        (value_ty, acc)
    }

    fn check_lvalue(&mut self, lv: &LValue, ctx: &mut FnCtx) -> Type {
        match lv {
            LValue::Var(name) => match ctx.lookup(&name.name) {
                Some(t) => t,
                None => {
                    self.diags.push(
                        Diagnostic::error("DL0301", format!("unknown name `{}`", name.name))
                            .with_span(name.span, "not found in this scope"),
                    );
                    self.cx.fresh_type()
                }
            },
            LValue::Field(base, field) => {
                let bt = self.check_lvalue(base, ctx);
                self.field_type(&bt, field, ctx)
            }
            LValue::Index(base, _idx) => {
                let bt = self.check_lvalue(base, ctx);
                match self.cx.apply_type(&bt) {
                    Type::List(inner) => *inner,
                    _ => self.cx.fresh_type(),
                }
            }
        }
    }

    // ===== expressions ====================================================

    /// Record the DIR side tables for one node (the raw type and raw effect accumulator), then
    /// return them unchanged. The single choke point every `check_expr` call flows through, so the
    /// typed AST is captured without perturbing any typing rule (invariant: recording is a pure
    /// side effect — it never changes what the checker computes or reports).
    fn check_expr(&mut self, e: &Expr, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (ty, acc) = self.check_expr_inner(e, ctx);
        self.node_types_raw.insert(e.id(), ty.clone());
        self.node_row_accs.insert(e.id(), acc.clone());
        (ty, acc)
    }

    /// Resolve a raw effect accumulator to the node's concrete effect row: expand every bound tail
    /// through the substitution and union its effects; drop tails that never bound (they add no
    /// observable effect). Used only to settle the DIR side tables (§2.3).
    fn resolve_row_acc(&self, acc: &RowAcc) -> Row {
        let mut effects = acc.effects.clone();
        for t in &acc.tails {
            let r = self.cx.apply_row(&Row { effects: BTreeSet::new(), tail: Some(*t) });
            effects.extend(r.effects.into_iter());
        }
        Row { effects, tail: None }
    }

    fn check_expr_inner(&mut self, e: &Expr, ctx: &mut FnCtx) -> (Type, RowAcc) {
        match e {
            Expr::Lit { kind, .. } => (self.lit_type(kind), RowAcc::default()),
            Expr::Var { path, span, .. } => (self.check_var(path, *span, ctx), RowAcc::default()),
            Expr::List { items, span, .. } => self.check_list(items, *span, ctx),
            Expr::Record { path, fields, span, .. } => self.check_record(path, fields, *span, ctx),
            Expr::Call { callee, args, span, .. } => self.check_call(callee, args, *span, ctx),
            Expr::Method { recv, name, args, span, id } => self.check_method(recv, name, args, *span, *id, ctx),
            Expr::Field { recv, name, .. } => {
                let (rt, rr) = self.check_expr(recv, ctx);
                (self.field_type(&rt, name, ctx), rr)
            }
            Expr::Index { recv, index, .. } => {
                let (rt, mut rr) = self.check_expr(recv, ctx);
                let (it, ir) = self.check_expr(index, ctx);
                rr.add_row_acc(&ir);
                self.expect_type(&Type::Int, &it, index.span(), "index must be Int");
                let out = match self.cx.apply_type(&rt) {
                    Type::List(inner) => *inner,
                    other => {
                        self.diags.push(
                            Diagnostic::error("DL0405", format!("cannot index a value of type `{other}`"))
                                .with_span(recv.span(), "not indexable"),
                        );
                        self.cx.fresh_type()
                    }
                };
                (out, rr)
            }
            Expr::Unary { op, operand, span, .. } => self.check_unary(*op, operand, *span, ctx),
            Expr::Binary { op, lhs, rhs, span, .. } => self.check_binary(*op, lhs, rhs, *span, ctx),
            Expr::If { cond, then_, else_, span, .. } => self.check_if(cond, then_, else_.as_deref(), *span, ctx),
            Expr::Match { scrutinee, arms, span, .. } => self.check_match(scrutinee, arms, *span, ctx),
            Expr::Lambda { params, ret, row, body, .. } => self.check_lambda(params, ret.as_ref(), row.as_ref(), body, ctx),
            Expr::Try { inner, span, .. } => self.check_try(inner, *span, ctx),
            Expr::Block(b) => {
                let (t, r) = self.check_block(b, ctx);
                (t, r)
            }
        }
    }

    fn lit_type(&self, kind: &LitKind) -> Type {
        match kind {
            LitKind::Int(_) => Type::Int,
            LitKind::Float(_) => Type::Float,
            LitKind::Str(_) => Type::Str,
            LitKind::Bool(_) => Type::Bool,
        }
    }

    fn check_var(&mut self, path: &Path, span: Span, ctx: &mut FnCtx) -> Type {
        if path.segs.len() == 1 {
            let name = &path.segs[0].name;
            if let Some(t) = ctx.lookup(name) {
                return t;
            }
            // Nullary constructors.
            match name.as_str() {
                "None" => return Type::Option(Box::new(self.cx.fresh_type())),
                _ => {}
            }
            // A nullary user/prelude variant constructor used as a value (`Red`, `NotFound`, …).
            if let Some((id, fields)) = self.table.variant_ctor(name) {
                if fields.is_empty() {
                    let def = self.table.type_def(id).clone();
                    let args: Vec<Type> = def.generics.iter().map(|_| self.cx.fresh_type()).collect();
                    return Type::Sum(id, args);
                }
                self.diags.push(
                    Diagnostic::error("DL0403", format!("constructor `{name}` needs {} field(s)", fields.len()))
                        .with_span(span, "missing constructor arguments"),
                );
                return self.cx.fresh_type();
            }
            // A reference to a top-level function value (row carried — invariant 3).
            if let Some(sig) = self.table.fns.get(name).cloned() {
                ctx.facts.callees.insert(name.clone());
                return self.instantiate_fn(&sig);
            }
            if let Some(c) = self.table.consts.get(name) {
                if let Some(ty) = &c.ty {
                    return self.lower_type(&ty.clone(), &ctx.genv, &mut ctx.facts);
                }
                return self.cx.fresh_type();
            }
        }
        self.diags.push(
            Diagnostic::error("DL0301", format!("unknown name `{}`", path.dotted()))
                .with_span(span, "not found in this scope"),
        );
        self.cx.fresh_type()
    }

    fn check_list(&mut self, items: &[Expr], _span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let elem = self.cx.fresh_type();
        let mut acc = RowAcc::default();
        for it in items {
            let (t, r) = self.check_expr(it, ctx);
            acc.add_row_acc(&r);
            self.expect_type(&elem, &t, it.span(), "list elements must share a type");
        }
        (Type::List(Box::new(elem)), acc)
    }

    fn check_record(&mut self, path: &Path, fields: &[(Ident, Expr)], span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let name = path.segs.last().unwrap().name.clone();
        let mut acc = RowAcc::default();
        let Some(&id) = self.table.type_ix.get(&name) else {
            self.diags.push(Diagnostic::error("DL0301", format!("unknown type `{name}`")).with_span(span, "not a type"));
            return (self.cx.fresh_type(), acc);
        };
        let def = self.table.type_def(id).clone();
        let TypeDefKind::Record(deffields) = &def.kind else {
            self.diags.push(Diagnostic::error("DL0401", format!("`{name}` is not a record type")).with_span(span, "not a record"));
            return (self.cx.fresh_type(), acc);
        };
        // Fresh type args for the record's generics.
        let mut genv = Genv::default();
        let args: Vec<Type> = def.generics.iter().map(|g| { let v = self.cx.fresh_type(); genv.types.insert(g.clone(), v.clone()); v }).collect();
        for (fname, fexpr) in fields {
            let (ft, fr) = self.check_expr(fexpr, ctx);
            acc.add_row_acc(&fr);
            if let Some((_, decl_ty)) = deffields.iter().find(|(n, _)| *n == fname.name) {
                let expected = self.lower_type(&decl_ty.clone(), &genv, &mut ctx.facts);
                self.expect_type(&expected, &ft, fexpr.span(), "record field type mismatch");
            } else {
                self.diags.push(
                    Diagnostic::error("DL0405", format!("record `{name}` has no field `{}`", fname.name))
                        .with_span(fname.span, "unknown field"),
                );
            }
        }
        (Type::Record(id, args), acc)
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr], span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        // Builtins and value constructors when the callee is a bare, unshadowed name.
        if let Expr::Var { path, .. } = callee {
            if path.segs.len() == 1 && ctx.lookup(&path.segs[0].name).is_none() {
                let name = path.segs[0].name.as_str();
                if let Some(res) = self.check_builtin_call(name, args, span, ctx) {
                    return res;
                }
                // A user/prelude variant constructor with fields (`Say(x)`, `Other(m)`, …).
                if let Some((id, fields)) = self.table.variant_ctor(name) {
                    let def = self.table.type_def(id).clone();
                    let mut genv = Genv::default();
                    let targs: Vec<Type> = def
                        .generics
                        .iter()
                        .map(|g| {
                            let v = self.cx.fresh_type();
                            genv.types.insert(g.clone(), v.clone());
                            v
                        })
                        .collect();
                    let mut acc = RowAcc::default();
                    if args.len() != fields.len() {
                        self.diags.push(
                            Diagnostic::error("DL0403", format!("constructor `{name}` expects {} field(s), found {}", fields.len(), args.len()))
                                .with_span(span, "wrong number of arguments"),
                        );
                    }
                    for (arg, fty) in args.iter().zip(&fields) {
                        let (at, ar) = self.check_expr(arg, ctx);
                        acc.add_row_acc(&ar);
                        let expected = self.lower_type(&fty.clone(), &genv, &mut ctx.facts);
                        self.expect_type(&expected, &at, arg.span(), "constructor field type mismatch");
                    }
                    return (Type::Sum(id, targs), acc);
                }
            }
        }
        let (callee_ty, mut acc) = self.check_expr(callee, ctx);
        let callee_ty = self.cx.apply_type(&callee_ty);
        match callee_ty {
            Type::Fn { params, ret, row } => {
                if params.len() != args.len() {
                    self.diags.push(
                        Diagnostic::error("DL0403", format!("expected {} argument(s), found {}", params.len(), args.len()))
                            .with_span(span, "wrong number of arguments"),
                    );
                }
                for (arg, pty) in args.iter().zip(&params) {
                    let (at, ar) = self.check_expr(arg, ctx);
                    acc.add_row_acc(&ar);
                    self.expect_type(pty, &at, arg.span(), "argument type mismatch");
                }
                acc.add_row(&row); // T-Call: the callee's row joins ours.
                (*ret, acc)
            }
            other => {
                self.diags.push(
                    Diagnostic::error("DL0404", format!("value of type `{other}` is not callable"))
                        .with_span(callee.span(), "not a function"),
                );
                for arg in args {
                    let (_, ar) = self.check_expr(arg, ctx);
                    acc.add_row_acc(&ar);
                }
                (self.cx.fresh_type(), acc)
            }
        }
    }

    /// Prelude constructors and free builtins (§11). Returns None if `name` is not a builtin.
    fn check_builtin_call(&mut self, name: &str, args: &[Expr], span: Span, ctx: &mut FnCtx) -> Option<(Type, RowAcc)> {
        let mut acc = RowAcc::default();
        let check_args = |slf: &mut Self, ctx: &mut FnCtx, acc: &mut RowAcc| -> Vec<Type> {
            args.iter().map(|a| { let (t, r) = slf.check_expr(a, ctx); acc.add_row_acc(&r); slf.cx.apply_type(&t) }).collect()
        };
        match name {
            "Ok" => {
                let ts = check_args(self, ctx, &mut acc);
                let ok = ts.into_iter().next().unwrap_or(Type::Unit);
                Some((Type::Result(Box::new(ok), Box::new(self.cx.fresh_type())), acc))
            }
            "Err" => {
                let ts = check_args(self, ctx, &mut acc);
                let err = ts.into_iter().next().unwrap_or(Type::Unit);
                Some((Type::Result(Box::new(self.cx.fresh_type()), Box::new(err)), acc))
            }
            "Some" => {
                let ts = check_args(self, ctx, &mut acc);
                let inner = ts.into_iter().next().unwrap_or(Type::Unit);
                Some((Type::Option(Box::new(inner)), acc))
            }
            // T-Load (Stage 6, spec §3.1 / Stage-1 §8.2):
            //   load(host: Cap[PluginHost], path: Str, grant: Grant)
            //       -> Result[Plugin[C], PluginErr] ! {Load, Read}
            // `C` is a fresh variable pinned by the binding's annotation (deviation 4). The row is
            // `{Load, Read}` — bringing in code after compile time is itself an effect the host's
            // row must declare, and reading the artifact is a Read.
            "load" => {
                let ts = check_args(self, ctx, &mut acc);
                self.expect_named_arg(&ts, 0, &Type::Cap(ResourceKind::PluginHost), span, "load");
                self.expect_named_arg(&ts, 1, &Type::Str, span, "load");
                let grant_ty = Type::Record(self.table.type_ix["Grant"], vec![]);
                self.expect_named_arg(&ts, 2, &grant_ty, span, "load");
                if ts.len() != 3 {
                    self.diags.push(
                        Diagnostic::error("DL0403", format!("`load` expects 3 argument(s), found {}", ts.len()))
                            .with_span(span, "wrong number of arguments"),
                    );
                }
                acc.add_effect(Effect::Load);
                acc.add_effect(Effect::Read);
                ctx.facts.cap_kinds.insert(ResourceKind::PluginHost);
                let class = self.cx.fresh_type();
                let err = Type::Sum(self.table.type_ix["PluginErr"], vec![]);
                Some((Type::result(Type::Plugin(Box::new(class)), err), acc))
            }
            "str" => {
                let ts = check_args(self, ctx, &mut acc);
                if let Some(t) = ts.first() {
                    if self.is_opaque(t, &mut HashSet::new()) {
                        self.diags.push(
                            Diagnostic::error("DL0604", format!("value of type `{t}` cannot be stringified"))
                                .with_span(span, "opaque type (Secret/Cap/Root or a value containing one)"),
                        );
                    }
                }
                Some((Type::Str, acc))
            }
            "len" => {
                check_args(self, ctx, &mut acc);
                Some((Type::Int, acc))
            }
            "int" => {
                let ts = check_args(self, ctx, &mut acc);
                if let Some(t) = ts.first() { self.expect_type(&Type::Float, t, span, "int() expects a Float"); }
                Some((Type::Int, acc))
            }
            "float" => {
                let ts = check_args(self, ctx, &mut acc);
                if let Some(t) = ts.first() { self.expect_type(&Type::Int, t, span, "float() expects an Int"); }
                Some((Type::Float, acc))
            }
            "parse_int" => {
                check_args(self, ctx, &mut acc);
                Some((Type::Option(Box::new(Type::Int)), acc))
            }
            "range" => {
                check_args(self, ctx, &mut acc);
                Some((Type::List(Box::new(Type::Int)), acc))
            }
            "push" => {
                let ts = check_args(self, ctx, &mut acc);
                // Unify the element into the list, then yield Unit (push mutates in place).
                if let (Some(Type::List(elem)), Some(v)) = (ts.first(), ts.get(1)) {
                    let elem = (**elem).clone();
                    self.expect_type(&elem, v, span, "pushed element type must match the list");
                }
                Some((Type::Unit, acc))
            }
            _ => None,
        }
    }

    fn check_method(&mut self, recv: &Expr, name: &Ident, args: &[Expr], span: Span, node_id: NodeId, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (rt, mut acc) = self.check_expr(recv, ctx);
        let rt = self.cx.apply_type(&rt);
        // Capture literal secret names for the authority report (`root.secret("NAME")`).
        if matches!(rt, Type::Root) && name.name == "secret" {
            if let Some(Expr::Lit { kind: LitKind::Str(s), .. }) = args.first() {
                ctx.facts.secret_names.insert(s.clone());
            }
        }
        let arg_tys: Vec<(Type, Span)> = args
            .iter()
            .map(|a| {
                let (t, r) = self.check_expr(a, ctx);
                acc.add_row_acc(&r);
                (self.cx.apply_type(&t), a.span())
            })
            .collect();

        // R-4 (builtin-callback law, audit F-4): a method that invokes a function argument must
        // surface that argument's effect row into the caller. Without this, an effectful lambda
        // passed to `map` inside a "pure" function would escape the row entirely.
        if is_higher_order_method(&rt, &name.name) {
            if let Some((Type::Fn { row, .. }, _)) = arg_tys.first() {
                let r = self.cx.apply_row(row);
                acc.add_row(&r);
            }
        }

        if let Some((ret, effect, produced_cap)) = self.method_sig(&rt, &name.name, &arg_tys, span, ctx) {
            if let Some(e) = effect {
                acc.add_effect(e);
            }
            if let Some(k) = produced_cap {
                ctx.facts.cap_kinds.insert(k);
            }
            // T-ForeignBind runtime hook (phase 4d): remember this `root.foreign(load)` call site
            // and the fresh handle var it produced, so `check_module` can record the resolved lib
            // name for the interpreter once inference settles. `ret` is `Result[M, ForeignErr]`.
            if matches!(rt, Type::Root) && name.name == "foreign" {
                if let Type::Result(inner, _) = &ret {
                    if let Type::Var(tv) = **inner {
                        self.pending_foreign_binds.push((node_id, tv));
                    }
                }
            }
            return (ret, acc);
        }

        self.diags.push(
            Diagnostic::error("DL0405", format!("type `{rt}` has no method `{}`", name.name))
                .with_span(name.span, "unknown method"),
        );
        (self.cx.fresh_type(), acc)
    }

    /// The compile-time view of the primitive table (§7.3). Returns (return type, effect, a
    /// capability kind produced). Capability operations are the ONLY source of primitive
    /// effects (T-CapOp) — this table is that single source of truth for the checker.
    fn method_sig(&mut self, recv: &Type, method: &str, args: &[(Type, Span)], span: Span, ctx: &mut FnCtx) -> Option<(Type, Option<Effect>, Option<ResourceKind>)> {
        let io = Type::Sum(self.table.io_err(), vec![]);
        let net = Type::Sum(self.table.net_err(), vec![]);
        let ok_str_io = || Type::result(Type::Str, io.clone());
        match recv {
            Type::Root => {
                // Root methods mint capabilities (attenuation — pure, no effect).
                let (ret, cap) = match method {
                    "console" => (Type::Cap(ResourceKind::Console), ResourceKind::Console),
                    "fs_read" => (Type::Cap(ResourceKind::FsRead), ResourceKind::FsRead),
                    "fs_write" => (Type::Cap(ResourceKind::FsWrite), ResourceKind::FsWrite),
                    "http" => (Type::Cap(ResourceKind::Http), ResourceKind::Http),
                    "clock" => (Type::Cap(ResourceKind::Clock), ResourceKind::Clock),
                    "rand" => (Type::Cap(ResourceKind::Rand), ResourceKind::Rand),
                    "declassify" => (Type::Cap(ResourceKind::Declassify), ResourceKind::Declassify),
                    // Mints `Cap[ForeignLoad]`, which gates `root.foreign(load)` (spec §3, head-chef
                    // ruling): a pure derivation from `Root`, exactly like the other `root.X()`
                    // constructors — deriving the loader authority is not itself an effect.
                    "foreign_load" => (Type::Cap(ResourceKind::ForeignLoad), ResourceKind::ForeignLoad),
                    "plugin_host" => (Type::Cap(ResourceKind::PluginHost), ResourceKind::PluginHost),
                    "secret" => {
                        ctx.facts.uses_secret = true;
                        self.expect_arg(args, 0, &Type::Str, span);
                        return Some((Type::Secret(Box::new(Type::Str)), None, None));
                    }
                    // T-ForeignBind (spec §3): deriving a lib handle is PURE — the effect is in
                    // *using* the handle, not in binding it. `root.foreign(load: Cap[ForeignLoad])
                    // -> Result[M, ForeignErr]`. The lib type `M` is a fresh variable resolved from
                    // the binding's context (its annotation/use), i.e. the `[M]` of the normative
                    // signature is inferred rather than written — the grammar has no method
                    // type-argument syntax.
                    "foreign" => {
                        self.expect_arg(args, 0, &Type::Cap(ResourceKind::ForeignLoad), span);
                        let handle = self.cx.fresh_type();
                        let ferr = Type::Sum(self.table.foreign_err(), vec![]);
                        return Some((Type::result(handle, ferr), None, None));
                    }
                    // T-Py binding (spec §5.1): `root.python(load: Cap[ForeignLoad]) ->
                    // Result[Cap[Python], ForeignErr]` — PURE like `root.foreign` (deriving the handle
                    // is not an effect; the effect is in *using* it). Marks the program as wielding
                    // Python for the authority report (disclosed under the "outside the proof"
                    // separator, never as an ordinary capability row).
                    "python" => {
                        self.expect_arg(args, 0, &Type::Cap(ResourceKind::ForeignLoad), span);
                        let ferr = Type::Sum(self.table.foreign_err(), vec![]);
                        return Some((
                            Type::result(Type::Cap(ResourceKind::Python), ferr),
                            None,
                            Some(ResourceKind::Python),
                        ));
                    }
                    _ => return None,
                };
                // Root constructors that take a path/hosts argument.
                if matches!(method, "fs_read" | "fs_write") {
                    self.expect_arg(args, 0, &Type::Str, span);
                } else if method == "http" {
                    self.expect_arg(args, 0, &Type::List(Box::new(Type::Str)), span);
                }
                Some((ret, None, Some(cap)))
            }
            Type::Cap(ResourceKind::Console) => match method {
                "println" | "print" => {
                    self.expect_arg(args, 0, &Type::Str, span);
                    Some((Type::Unit, Some(Effect::Write), None))
                }
                "readline" => Some((ok_str_io(), Some(Effect::Read), None)),
                _ => None,
            },
            Type::Cap(ResourceKind::FsRead) => match method {
                "read_text" => { self.expect_arg(args, 0, &Type::Str, span); Some((ok_str_io(), Some(Effect::Read), None)) }
                "list_dir" => { self.expect_arg(args, 0, &Type::Str, span); Some((Type::result(Type::List(Box::new(Type::Str)), io), Some(Effect::Read), None)) }
                "narrow" => { self.expect_arg(args, 0, &Type::Str, span); Some((Type::Cap(ResourceKind::FsRead), None, None)) }
                _ => None,
            },
            Type::Cap(ResourceKind::FsWrite) => match method {
                "write_text" | "append_text" => {
                    self.expect_arg(args, 0, &Type::Str, span);
                    self.expect_arg(args, 1, &Type::Str, span);
                    Some((Type::result(Type::Unit, io), Some(Effect::Write), None))
                }
                _ => None,
            },
            Type::Cap(ResourceKind::Http) => match method {
                "get" => { self.expect_arg(args, 0, &Type::Str, span); Some((Type::result(Type::Str, net), Some(Effect::Net), None)) }
                _ => None,
            },
            Type::Cap(ResourceKind::Clock) => match method {
                "now_ms" => Some((Type::Int, Some(Effect::Clock), None)),
                _ => None,
            },
            // T-Get / T-Unload (Stage 6, spec §3.3). `p.get[F](name) -> Result[F, PluginErr]`:
            // `F` is a fresh variable pinned by the binding's annotation (deviation 4), so the
            // R-Get row check is recorded here and settled once inference finishes (see
            // `check_module`) — exactly the `root.foreign` pattern.
            //
            // What is compile-time vs. runtime, precisely:
            // - **DL0803** (a function-typed parameter anywhere in `F` on a **Contained** plugin) is
            //   COMPILE-TIME, at this call site — R-6a: an opaque module must never hold a re-entry
            //   point into verified code.
            // - The row check itself (`effects(grant) ⊆ row(F)` for Contained, R-1;
            //   `row(export) ⊆ row(F)` for Verified) is RUNTIME, because the grant and the loaded
            //   export type are runtime values. The compile-time half is automatic and needs no
            //   rule: `F`'s row flows into the caller's row through T-Call, so a host calling a
            //   `!{Read, Net}` export must already declare `Net` — the design paying off.
            Type::Plugin(class) => match method {
                "get" => {
                    self.expect_arg(args, 0, &Type::Str, span);
                    let f = self.cx.fresh_type();
                    if let Type::Var(tv) = f {
                        self.pending_gets.push((span, tv, (**class).clone()));
                    }
                    let err = Type::Sum(self.table.type_ix["PluginErr"], vec![]);
                    Some((Type::result(f, err), None, None))
                }
                // `p.unload() -> Unit` (spec §3.3). Revocation is a custody act, not an effect the
                // row tracks: the spec's signature carries no row.
                "unload" => Some((Type::Unit, None, None)),
                _ => None,
            },
            Type::Cap(ResourceKind::Rand) => match method {
                "int" => { self.expect_arg(args, 0, &Type::Int, span); self.expect_arg(args, 1, &Type::Int, span); Some((Type::Int, Some(Effect::Rand), None)) }
                "float" => Some((Type::Float, Some(Effect::Rand), None)),
                _ => None,
            },
            // T-Py (spec §5.2): every `Cap[Python]` operation has row exactly `{ForeignCall}`. The
            // constructors (`of_*`, `list`) return a bare `PyObj`; `import`/`to_*` return a `Result`
            // over `PyErr` (a record). `PyObj` is opaque throughout (R-5).
            Type::Cap(ResourceKind::Python) => {
                let py_err_id = self.table.py_err();
                let pyerr = || Type::Record(py_err_id, vec![]);
                match method {
                    "import" => {
                        self.expect_arg(args, 0, &Type::Str, span);
                        Some((Type::result(Type::PyObj, pyerr()), Some(Effect::ForeignCall), None))
                    }
                    "of_int" => { self.expect_arg(args, 0, &Type::Int, span); Some((Type::PyObj, Some(Effect::ForeignCall), None)) }
                    "of_float" => { self.expect_arg(args, 0, &Type::Float, span); Some((Type::PyObj, Some(Effect::ForeignCall), None)) }
                    "of_str" => { self.expect_arg(args, 0, &Type::Str, span); Some((Type::PyObj, Some(Effect::ForeignCall), None)) }
                    "of_bool" => { self.expect_arg(args, 0, &Type::Bool, span); Some((Type::PyObj, Some(Effect::ForeignCall), None)) }
                    "list" => {
                        // A DeluluLang closure can never become a Python value (spec §4.4 / R-6a):
                        // a function-typed element is DL1302, not a plain type mismatch.
                        if self.reject_py_callback(args, 0, span) {
                            return Some((Type::PyObj, Some(Effect::ForeignCall), None));
                        }
                        self.expect_arg(args, 0, &Type::List(Box::new(Type::PyObj)), span);
                        Some((Type::PyObj, Some(Effect::ForeignCall), None))
                    }
                    "to_int" => { self.expect_arg(args, 0, &Type::PyObj, span); Some((Type::result(Type::Int, pyerr()), Some(Effect::ForeignCall), None)) }
                    "to_float" => { self.expect_arg(args, 0, &Type::PyObj, span); Some((Type::result(Type::Float, pyerr()), Some(Effect::ForeignCall), None)) }
                    "to_str" => { self.expect_arg(args, 0, &Type::PyObj, span); Some((Type::result(Type::Str, pyerr()), Some(Effect::ForeignCall), None)) }
                    "to_bool" => { self.expect_arg(args, 0, &Type::PyObj, span); Some((Type::result(Type::Bool, pyerr()), Some(Effect::ForeignCall), None)) }
                    _ => None,
                }
            }
            // T-Py (spec §5.2): `PyObj` operations (`attr`/`call`/`call_method`/`index`), each
            // `{ForeignCall}`, each `Result[PyObj, PyErr]`. Passing a DeluluLang closure where a
            // `PyObj` is expected is DL1302 (no callbacks, R-6a) — the R-6a diagnostic, not a plain
            // type mismatch (spec §4.4).
            Type::PyObj => {
                let py_err_id = self.table.py_err();
                let result_pyobj = || Type::result(Type::PyObj, Type::Record(py_err_id, vec![]));
                match method {
                    "attr" => {
                        self.expect_arg(args, 0, &Type::Str, span);
                        Some((result_pyobj(), Some(Effect::ForeignCall), None))
                    }
                    "call" => {
                        if self.reject_py_callback(args, 0, span) {
                            return Some((result_pyobj(), Some(Effect::ForeignCall), None));
                        }
                        self.expect_arg(args, 0, &Type::List(Box::new(Type::PyObj)), span);
                        Some((result_pyobj(), Some(Effect::ForeignCall), None))
                    }
                    "call_method" => {
                        self.expect_arg(args, 0, &Type::Str, span);
                        if self.reject_py_callback(args, 1, span) {
                            return Some((result_pyobj(), Some(Effect::ForeignCall), None));
                        }
                        self.expect_arg(args, 1, &Type::List(Box::new(Type::PyObj)), span);
                        Some((result_pyobj(), Some(Effect::ForeignCall), None))
                    }
                    "index" => {
                        if self.reject_py_callback(args, 0, span) {
                            return Some((result_pyobj(), Some(Effect::ForeignCall), None));
                        }
                        self.expect_arg(args, 0, &Type::PyObj, span);
                        Some((result_pyobj(), Some(Effect::ForeignCall), None))
                    }
                    _ => None,
                }
            }
            Type::Secret(inner) => match method {
                // Secret.map requires a PURE function (DL0603); result stays tainted (R-5).
                "map" => {
                    if let Some((Type::Fn { row, ret, .. }, aspan)) = args.first() {
                        let r = self.cx.apply_row(row);
                        if !r.is_pure() {
                            self.diags.push(
                                Diagnostic::error("DL0603", "Secret.map requires a pure function")
                                    .with_span(*aspan, "this function is not pure"),
                            );
                        }
                        Some((Type::Secret(Box::new((**ret).clone())), None, None))
                    } else {
                        Some((Type::Secret(inner.clone()), None, None))
                    }
                }
                // verify is a constant-time comparison of two secrets — pure, no reveal.
                "verify" => {
                    self.expect_arg(args, 0, &Type::Secret(inner.clone()), span);
                    Some((Type::Bool, None, None))
                }
                // expose is the ONLY unwrap; it requires Cap[Declassify] and carries Declassify (R-2).
                "expose" => {
                    self.expect_arg(args, 0, &Type::Cap(ResourceKind::Declassify), span);
                    Some(((**inner).clone(), Some(Effect::Declassify), None))
                }
                _ => None,
            },
            Type::Str => match method {
                "len" => Some((Type::Int, None, None)),
                "trim" => Some((Type::Str, None, None)),
                "contains" | "starts_with" => { self.expect_arg(args, 0, &Type::Str, span); Some((Type::Bool, None, None)) }
                "split" => { self.expect_arg(args, 0, &Type::Str, span); Some((Type::List(Box::new(Type::Str)), None, None)) }
                "slice" => { self.expect_arg(args, 0, &Type::Int, span); self.expect_arg(args, 1, &Type::Int, span); Some((Type::Str, None, None)) }
                _ => None,
            },
            Type::List(elem) => match method {
                "len" => Some((Type::Int, None, None)),
                "get" => { self.expect_arg(args, 0, &Type::Int, span); Some((Type::Option(elem.clone()), None, None)) }
                "push" => { Some((Type::Unit, None, None)) }
                "map" => {
                    // Row-polymorphic builtin (R-4): the callback's row joins the caller's.
                    if let Some((Type::Fn { row, ret, .. }, _)) = args.first() {
                        // NOTE: caller already accounts for the callback row via the arg's own
                        // check; here we surface it into the method's effect via add_row upstream.
                        let _ = row;
                        Some((Type::List(Box::new((**ret).clone())), None, None))
                    } else {
                        Some((Type::List(elem.clone()), None, None))
                    }
                }
                _ => None,
            },
            // T-ForeignCall (spec §3): a method call on a lib handle types per the block's
            // signature, with row exactly `{ForeignCall}`.
            Type::Foreign(libname) => {
                let fdef = self.table.foreigns.get(libname)?.clone();
                let ffn = fdef.fns.iter().find(|f| f.name == method)?;
                if ffn.params.len() != args.len() {
                    self.diags.push(
                        Diagnostic::error(
                            "DL0403",
                            format!(
                                "foreign function `{}.{}` expects {} argument(s), found {}",
                                libname,
                                method,
                                ffn.params.len(),
                                args.len()
                            ),
                        )
                        .with_span(span, "wrong number of arguments"),
                    );
                }
                for (i, p) in ffn.params.iter().enumerate() {
                    let expected = self.lower_type(&p.ty, &Genv::default(), &mut FnFacts::default());
                    if let Some((at, aspan)) = args.get(i) {
                        self.expect_type(&expected, at, *aspan, "foreign argument type mismatch");
                    }
                }
                let ret_ty = match &ffn.ret {
                    Some(t) => self.lower_type(t, &Genv::default(), &mut FnFacts::default()),
                    None => Type::Unit,
                };
                Some((ret_ty, Some(Effect::ForeignCall), None))
            }
            _ => None,
        }
    }

    /// `expect_arg` for a builtin free function whose arguments were already checked into a plain
    /// `Vec<Type>` (the `check_builtin_call` shape). A missing argument is reported by the caller's
    /// arity check, so this only unifies what is present.
    fn expect_named_arg(&mut self, args: &[Type], i: usize, expected: &Type, span: Span, what: &str) {
        if let Some(t) = args.get(i) {
            self.expect_type(expected, t, span, &format!("`{what}` argument {} type mismatch", i + 1));
        }
    }

    fn expect_arg(&mut self, args: &[(Type, Span)], i: usize, expected: &Type, call_span: Span) {
        match args.get(i) {
            Some((t, s)) => self.expect_type(expected, t, *s, "argument type mismatch"),
            None => self.diags.push(
                Diagnostic::error("DL0403", format!("missing argument {} (expected `{expected}`)", i + 1))
                    .with_span(call_span, "too few arguments"),
            ),
        }
    }

    /// Reject a function-typed argument crossing into `PyObj`-land (spec §4.4 / R-6a): a DeluluLang
    /// closure can never become a Python callable, and no `fn → PyObj` conversion exists anywhere in
    /// the `std.py` surface. The natural failure of `py.list([closure])` / `obj.call([closure])`
    /// would be a plain type mismatch (DL0401); the spec wants the R-6a diagnostic (DL1302). Returns
    /// `true` (and emits DL1302, suppressing the follow-on DL0401) when argument `i` contains a
    /// function type at any depth.
    fn reject_py_callback(&mut self, args: &[(Type, Span)], i: usize, span: Span) -> bool {
        let Some((t, aspan)) = args.get(i) else { return false };
        let (resolved, aspan) = (self.cx.apply_type(t), *aspan);
        if type_contains_fn(&resolved) {
            self.diags.push(
                Diagnostic::error(
                    "DL1302",
                    "a function-typed value cannot cross the foreign boundary — no callbacks, by rule R-6a",
                )
                .with_span(aspan, "unverifiable code must never hold a re-entry point into verified code")
                .with_secondary_span(span, "in this Python call"),
            );
            return true;
        }
        false
    }

    fn field_type(&mut self, recv: &Type, field: &Ident, _ctx: &mut FnCtx) -> Type {
        match self.cx.apply_type(recv) {
            Type::Record(id, args) => {
                let def = self.table.type_def(id).clone();
                if let TypeDefKind::Record(fields) = &def.kind {
                    if let Some((_, ty)) = fields.iter().find(|(n, _)| *n == field.name) {
                        let mut genv = Genv::default();
                        for (g, a) in def.generics.iter().zip(&args) {
                            genv.types.insert(g.clone(), a.clone());
                        }
                        return self.lower_type(&ty.clone(), &genv, &mut FnFacts::default());
                    }
                }
                self.diags.push(
                    Diagnostic::error("DL0405", format!("no field `{}` on `{}`", field.name, def.name))
                        .with_span(field.span, "unknown field"),
                );
                self.cx.fresh_type()
            }
            other => {
                self.diags.push(
                    Diagnostic::error("DL0405", format!("type `{other}` has no field `{}`", field.name))
                        .with_span(field.span, "not a record"),
                );
                self.cx.fresh_type()
            }
        }
    }

    fn check_unary(&mut self, op: UnOp, operand: &Expr, _span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (t, r) = self.check_expr(operand, ctx);
        let t = self.cx.apply_type(&t);
        match op {
            UnOp::Neg => {
                if !t.is_numeric() && !matches!(t, Type::Var(_)) {
                    self.diags.push(Diagnostic::error("DL0401", format!("cannot negate `{t}`")).with_span(operand.span(), "expected Int or Float"));
                }
                (t, r)
            }
            UnOp::Not => {
                self.expect_type(&Type::Bool, &t, operand.span(), "`!` expects Bool");
                (Type::Bool, r)
            }
        }
    }

    fn check_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (lt, mut acc) = self.check_expr(lhs, ctx);
        let (rt, rr) = self.check_expr(rhs, ctx);
        acc.add_row_acc(&rr);
        let lt = self.cx.apply_type(&lt);
        let rt = self.cx.apply_type(&rt);
        use BinOp::*;
        let out = match op {
            Add => {
                // Int+Int, Float+Float, or Str+Str (concatenation).
                if matches!(lt, Type::Str) || matches!(rt, Type::Str) {
                    self.expect_type(&Type::Str, &lt, lhs.span(), "`+` on strings requires both sides Str");
                    self.expect_type(&Type::Str, &rt, rhs.span(), "`+` on strings requires both sides Str");
                    Type::Str
                } else {
                    self.numeric_binop(&lt, &rt, lhs.span(), rhs.span(), span)
                }
            }
            Sub | Mul | Div | Rem => self.numeric_binop(&lt, &rt, lhs.span(), rhs.span(), span),
            Lt | Le | Gt | Ge => {
                self.numeric_binop(&lt, &rt, lhs.span(), rhs.span(), span);
                Type::Bool
            }
            Eq | Ne => {
                if self.is_opaque(&lt, &mut HashSet::new()) || self.is_opaque(&rt, &mut HashSet::new()) {
                    self.diags.push(
                        Diagnostic::error("DL0605", "opaque types (Secret/Cap/Root) have no structural equality")
                            .with_span(span, "use `Secret.verify` for secrets"),
                    );
                } else {
                    self.expect_type(&lt, &rt, span, "equality requires matching types");
                }
                Type::Bool
            }
            And | Or => {
                self.expect_type(&Type::Bool, &lt, lhs.span(), "logical operator expects Bool");
                self.expect_type(&Type::Bool, &rt, rhs.span(), "logical operator expects Bool");
                Type::Bool
            }
        };
        (out, acc)
    }

    fn numeric_binop(&mut self, lt: &Type, rt: &Type, ls: Span, rs: Span, span: Span) -> Type {
        let lt = self.cx.apply_type(lt);
        let rt = self.cx.apply_type(rt);
        match (&lt, &rt) {
            (Type::Int, Type::Int) => Type::Int,
            (Type::Float, Type::Float) => Type::Float,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => {
                self.diags.push(
                    Diagnostic::error("DL0402", "mixed Int and Float (no implicit coercion — use `int()`/`float()`)")
                        .with_span(span, "operands have different numeric types"),
                );
                Type::Int
            }
            (Type::Var(_), other) | (other, Type::Var(_)) if other.is_numeric() => other.clone(),
            (Type::Var(_), Type::Var(_)) => {
                let _ = self.cx.unify_type(&lt, &Type::Int);
                let _ = self.cx.unify_type(&rt, &Type::Int);
                Type::Int
            }
            _ => {
                self.expect_type(&Type::Int, &lt, ls, "arithmetic expects Int or Float");
                self.expect_type(&Type::Int, &rt, rs, "arithmetic expects Int or Float");
                Type::Int
            }
        }
    }

    fn check_if(&mut self, cond: &Expr, then_: &Block, else_: Option<&Expr>, _span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (ct, mut acc) = self.check_expr(cond, ctx);
        self.expect_type(&Type::Bool, &ct, cond.span(), "`if` condition must be Bool");
        let (tt, tr) = self.check_block(then_, ctx);
        acc.add_row_acc(&tr);
        match else_ {
            Some(e) => {
                let (et, er) = self.check_expr(e, ctx);
                acc.add_row_acc(&er);
                self.expect_type(&tt, &et, e.span(), "`if` branches must have the same type");
                (tt, acc)
            }
            None => {
                // No else: the `if` is a statement, value Unit.
                (Type::Unit, acc)
            }
        }
    }

    fn check_match(&mut self, scrutinee: &Expr, arms: &[Arm], span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (st, mut acc) = self.check_expr(scrutinee, ctx);
        let st = self.cx.apply_type(&st);
        let result_ty = self.cx.fresh_type();
        let mut covered: HashSet<String> = HashSet::new();
        let mut has_wildcard = false;

        for arm in arms {
            ctx.push_scope();
            self.check_pattern(&arm.pattern, &st, ctx, &mut covered, &mut has_wildcard);
            let (bt, br) = self.check_expr(&arm.body, ctx);
            acc.add_row_acc(&br);
            self.expect_type(&result_ty, &bt, arm.body.span(), "match arms must have the same type");
            ctx.pop_scope();
        }

        self.check_exhaustive(&st, &covered, has_wildcard, span);
        (result_ty, acc)
    }

    fn check_pattern(&mut self, pat: &Pattern, scrut: &Type, ctx: &mut FnCtx, covered: &mut HashSet<String>, has_wildcard: &mut bool) {
        match pat {
            Pattern::Wildcard(_) => *has_wildcard = true,
            Pattern::Bind(name) => {
                *has_wildcard = true;
                ctx.bind(&name.name, self.cx.apply_type(scrut));
            }
            Pattern::Lit(kind, s) => {
                let lt = self.lit_type(kind);
                self.expect_type(scrut, &lt, *s, "pattern type mismatch");
            }
            Pattern::Variant { path, fields, span } => {
                let vname = path.segs.last().unwrap().name.clone();
                covered.insert(vname.clone());
                let field_tys = self.variant_field_types(scrut, &vname, *span);
                if field_tys.len() != fields.len() {
                    self.diags.push(
                        Diagnostic::error("DL0403", format!("variant `{vname}` binds {} field(s), pattern has {}", field_tys.len(), fields.len()))
                            .with_span(*span, "wrong number of fields"),
                    );
                }
                for (fp, ft) in fields.iter().zip(field_tys) {
                    self.check_pattern(fp, &ft, ctx, &mut HashSet::new(), &mut false);
                }
            }
        }
    }

    fn variant_field_types(&mut self, scrut: &Type, vname: &str, span: Span) -> Vec<Type> {
        match self.cx.apply_type(scrut) {
            Type::Result(ok, err) => match vname {
                "Ok" => vec![*ok],
                "Err" => vec![*err],
                _ => { self.bad_variant(vname, "Result", span); vec![] }
            },
            Type::Option(inner) => match vname {
                "Some" => vec![*inner],
                "None" => vec![],
                _ => { self.bad_variant(vname, "Option", span); vec![] }
            },
            Type::Sum(id, args) => {
                let def = self.table.type_def(id).clone();
                if let TypeDefKind::Sum(variants) = &def.kind {
                    if let Some((_, tys)) = variants.iter().find(|(n, _)| n == vname) {
                        let mut genv = Genv::default();
                        for (g, a) in def.generics.iter().zip(&args) {
                            genv.types.insert(g.clone(), a.clone());
                        }
                        return tys.iter().map(|t| self.lower_type(&t.clone(), &genv, &mut FnFacts::default())).collect();
                    }
                }
                self.bad_variant(vname, &def.name, span);
                vec![]
            }
            other => {
                self.diags.push(
                    Diagnostic::error("DL0401", format!("cannot match variant `{vname}` against `{other}`"))
                        .with_span(span, "not a sum type"),
                );
                vec![]
            }
        }
    }

    fn bad_variant(&mut self, vname: &str, ty: &str, span: Span) {
        self.diags.push(
            Diagnostic::error("DL0405", format!("`{ty}` has no variant `{vname}`")).with_span(span, "unknown variant"),
        );
    }

    fn check_exhaustive(&mut self, scrut: &Type, covered: &HashSet<String>, has_wildcard: bool, span: Span) {
        if has_wildcard {
            return;
        }
        let required: Vec<&str> = match self.cx.apply_type(scrut) {
            Type::Result(_, _) => vec!["Ok", "Err"],
            Type::Option(_) => vec!["Some", "None"],
            Type::Sum(id, _) => {
                let def = self.table.type_def(id).clone();
                if let TypeDefKind::Sum(variants) = &def.kind {
                    let missing: Vec<String> = variants.iter().map(|(n, _)| n.clone()).filter(|n| !covered.contains(n)).collect();
                    if !missing.is_empty() {
                        self.diags.push(
                            Diagnostic::error("DL0407", format!("non-exhaustive match: missing {}", missing.join(", ")))
                                .with_span(span, "add the missing arms or a `_` wildcard"),
                        );
                    }
                    return;
                }
                return;
            }
            _ => return,
        };
        let missing: Vec<&str> = required.into_iter().filter(|v| !covered.contains(*v)).collect();
        if !missing.is_empty() {
            self.diags.push(
                Diagnostic::error("DL0407", format!("non-exhaustive match: missing {}", missing.join(", ")))
                    .with_span(span, "add the missing arms or a `_` wildcard"),
            );
        }
    }

    fn check_lambda(&mut self, params: &[Param], ret: Option<&TypeExpr>, row: Option<&RowExpr>, body: &Block, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let param_types: Vec<Type> = params.iter().map(|p| self.lower_type(&p.ty, &ctx.genv, &mut ctx.facts)).collect();
        // A lambda body is checked in the enclosing function's local scope plus its params.
        ctx.push_scope();
        for (p, ty) in params.iter().zip(&param_types) {
            ctx.bind(&p.name.name, ty.clone());
        }
        let (body_ty, body_row) = self.check_block(body, ctx);
        ctx.pop_scope();

        // T-Lambda: the row is inferred as exactly the body's row, unless annotated (then checked).
        let inferred = self.acc_to_row(&body_row);
        let final_row = match row {
            Some(r) => {
                let declared = self.lower_row(r, &ctx.genv);
                // Annotated lambda: body ⊆ declared (a subsumption site, like T-Fn).
                if !self.row_covered(&inferred, &declared) {
                    self.diags.push(
                        Diagnostic::error("DL0501", "lambda performs an effect not in its declared row")
                            .with_span(r.span, "declared here"),
                    );
                }
                declared
            }
            None => inferred,
        };
        let ret_ty = match ret {
            Some(t) => {
                let rt = self.lower_type(t, &ctx.genv, &mut ctx.facts);
                self.expect_type(&rt, &body_ty, body.span, "lambda body type must match its return type");
                rt
            }
            None => body_ty,
        };
        let mut acc = RowAcc::default();
        // The lambda VALUE is pure to construct; its row lives in its type, surfacing when called.
        let _ = &mut acc;
        (Type::Fn { params: param_types, ret: Box::new(ret_ty), row: final_row }, acc)
    }

    fn check_try(&mut self, inner: &Expr, span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let (it, acc) = self.check_expr(inner, ctx);
        let it = self.cx.apply_type(&it);
        match it {
            Type::Result(ok, err) => {
                match &ctx.ret_err {
                    Some(re) => {
                        self.expect_type(re, &err, span, "`?` error type must match the function's Result error type");
                    }
                    None => {
                        self.diags.push(
                            Diagnostic::error("DL0409", "`?` is only allowed in a function that returns Result")
                                .with_span(span, "this function does not return Result"),
                        );
                    }
                }
                (*ok, acc) // pure control flow — no effect
            }
            other => {
                self.diags.push(
                    Diagnostic::error("DL0409", format!("`?` requires a Result value, found `{other}`"))
                        .with_span(inner.span(), "not a Result"),
                );
                (self.cx.fresh_type(), acc)
            }
        }
    }

    // ===== instantiation, lowering, helpers ===============================

    /// Instantiate a function signature at a use site with fresh inference variables per generic.
    fn instantiate_fn(&mut self, sig: &FnSig) -> Type {
        let genv = self.make_genv(sig);
        let params = sig.params.iter().map(|p| self.lower_type(&p.ty, &genv, &mut FnFacts::default())).collect();
        let ret = match &sig.ret {
            Some(t) => self.lower_type(t, &genv, &mut FnFacts::default()),
            None => Type::Unit,
        };
        let row = match &sig.row {
            Some(r) => self.lower_row(r, &genv),
            None => Row::pure(),
        };
        Type::Fn { params, ret: Box::new(ret), row }
    }

    fn lower_type(&mut self, t: &TypeExpr, genv: &Genv, facts: &mut FnFacts) -> Type {
        match t {
            TypeExpr::Named { path, args, span } => {
                if path.segs.len() == 1 {
                    let name = &path.segs[0].name;
                    if let Some(v) = genv.types.get(name) {
                        if !args.is_empty() {
                            self.diags.push(Diagnostic::error("DL0406", format!("type parameter `{name}` takes no arguments")).with_span(*span, "unexpected type arguments"));
                        }
                        return v.clone();
                    }
                    match name.as_str() {
                        "Int" => return Type::Int,
                        "Float" => return Type::Float,
                        "Bool" => return Type::Bool,
                        "Str" => return Type::Str,
                        "Unit" => return Type::Unit,
                        "Root" => return Type::Root,
                        "List" => return Type::List(Box::new(self.lower_one_arg(args, genv, facts, *span))),
                        "Option" => return Type::Option(Box::new(self.lower_one_arg(args, genv, facts, *span))),
                        "Result" => {
                            let (a, b) = self.lower_two_args(args, genv, facts, *span);
                            return Type::Result(Box::new(a), Box::new(b));
                        }
                        "Secret" => return Type::Secret(Box::new(self.lower_one_arg(args, genv, facts, *span))),
                        "Cap" => return self.lower_cap(args, *span, facts),
                        "ForeignPtr" => return Type::ForeignPtr,
                        "PyObj" => return Type::PyObj,
                        // `Plugin[C]` (Stage 6 §3). `C` is an ordinary inference position holding a
                        // class marker, so `let p: Plugin[Contained] = load(…)` pins the `C` that
                        // `load` returned fresh (build-order deviation 4 — the grammar has no
                        // turbofish, per Stage-1 §6).
                        "Plugin" => return Type::Plugin(Box::new(self.lower_one_arg(args, genv, facts, *span))),
                        "Verified" => return Type::Verified,
                        "Contained" => return Type::Contained,
                        _ => {}
                    }
                    // A `foreign … lib M` block name is the nominal opaque handle type `M` (§2).
                    if self.table.foreigns.contains_key(name) {
                        if !args.is_empty() {
                            self.diags.push(
                                Diagnostic::error("DL0406", format!("foreign lib type `{name}` takes no type arguments"))
                                    .with_span(*span, "unexpected type arguments"),
                            );
                        }
                        return Type::Foreign(name.clone());
                    }
                    if let Some(&id) = self.table.type_ix.get(name) {
                        let def = self.table.type_def(id).clone();
                        if args.len() != def.generics.len() {
                            self.diags.push(Diagnostic::error("DL0406", format!("type `{name}` expects {} argument(s), found {}", def.generics.len(), args.len())).with_span(*span, "wrong number of type arguments"));
                        }
                        let targs: Vec<Type> = args.iter().map(|a| self.lower_type(a, genv, facts)).collect();
                        return match def.kind {
                            TypeDefKind::Record(_) => Type::Record(id, targs),
                            TypeDefKind::Sum(_) => Type::Sum(id, targs),
                            TypeDefKind::Alias(inner) => {
                                let mut agenv = Genv::default();
                                for (g, a) in def.generics.iter().zip(&targs) {
                                    agenv.types.insert(g.clone(), a.clone());
                                }
                                self.lower_type(&inner, &agenv, facts)
                            }
                        };
                    }
                    self.diags.push(Diagnostic::error("DL0301", format!("unknown type `{name}`")).with_span(*span, "not a type"));
                    return self.cx.fresh_type();
                }
                self.diags.push(Diagnostic::error("DL0301", format!("unknown type `{}`", path.dotted())).with_span(*span, "not a type"));
                self.cx.fresh_type()
            }
            TypeExpr::Fn { params, ret, row, .. } => {
                let params = params.iter().map(|p| self.lower_type(p, genv, facts)).collect();
                let ret = match ret {
                    Some(r) => self.lower_type(r, genv, facts),
                    None => Type::Unit,
                };
                let row = match row {
                    Some(r) => self.lower_row(r, genv),
                    None => Row::pure(),
                };
                Type::Fn { params, ret: Box::new(ret), row }
            }
        }
    }

    fn lower_one_arg(&mut self, args: &[TypeExpr], genv: &Genv, facts: &mut FnFacts, span: Span) -> Type {
        if args.len() != 1 {
            self.diags.push(Diagnostic::error("DL0406", "expected exactly one type argument").with_span(span, "here"));
            return self.cx.fresh_type();
        }
        self.lower_type(&args[0], genv, facts)
    }

    fn lower_two_args(&mut self, args: &[TypeExpr], genv: &Genv, facts: &mut FnFacts, span: Span) -> (Type, Type) {
        if args.len() != 2 {
            self.diags.push(Diagnostic::error("DL0406", "expected exactly two type arguments").with_span(span, "here"));
            return (self.cx.fresh_type(), self.cx.fresh_type());
        }
        (self.lower_type(&args[0], genv, facts), self.lower_type(&args[1], genv, facts))
    }

    fn lower_cap(&mut self, args: &[TypeExpr], span: Span, facts: &mut FnFacts) -> Type {
        if args.len() != 1 {
            self.diags.push(Diagnostic::error("DL0406", "`Cap` expects one resource kind").with_span(span, "here"));
            return self.cx.fresh_type();
        }
        if let TypeExpr::Named { path, args: inner, span: ispan } = &args[0] {
            if path.segs.len() == 1 && inner.is_empty() {
                if let Some(k) = ResourceKind::from_name(&path.segs[0].name) {
                    facts.cap_kinds.insert(k);
                    return Type::Cap(k);
                }
            }
            self.diags.push(Diagnostic::error("DL0307", format!("`{}` is not a capability resource kind", path.dotted())).with_span(*ispan, "unknown resource kind"));
        } else {
            self.diags.push(Diagnostic::error("DL0307", "`Cap` expects a resource kind name").with_span(span, "here"));
        }
        self.cx.fresh_type()
    }

    fn lower_row(&mut self, r: &RowExpr, genv: &Genv) -> Row {
        let mut effects = BTreeSet::new();
        for path in &r.effects {
            let name = &path.segs.last().unwrap().name;
            if let Some(e) = Effect::core_from_name(name) {
                effects.insert(e);
            } else if self.table.user_effects.contains(name) {
                effects.insert(Effect::User(name.clone()));
            } else {
                self.diags.push(Diagnostic::error("DL0306", format!("unknown effect `{name}`")).with_span(path.span(), "not a declared effect"));
            }
        }
        let tail = match &r.tail {
            Some(t) => match genv.rows.get(&t.name) {
                Some(v) => Some(*v),
                None => {
                    self.diags.push(Diagnostic::error("DL0306", format!("unknown effect-row variable `{}`", t.name)).with_span(t.span, "not a row generic"));
                    None
                }
            },
            None => None,
        };
        Row { effects, tail }
    }

    fn note_caps_in(&self, ty: &Type, facts: &mut FnFacts) {
        match ty {
            Type::Cap(k) => { facts.cap_kinds.insert(*k); }
            Type::List(t) | Type::Option(t) | Type::Secret(t) => self.note_caps_in(t, facts),
            Type::Result(a, b) => { self.note_caps_in(a, facts); self.note_caps_in(b, facts); }
            Type::Fn { params, ret, .. } => { for p in params { self.note_caps_in(p, facts); } self.note_caps_in(ret, facts); }
            _ => {}
        }
    }

    fn acc_to_row(&self, acc: &RowAcc) -> Row {
        // Fold the tails into a single representative row by resolution.
        let mut effects = acc.effects.clone();
        let mut tail = None;
        for &t in &acc.tails {
            let r = self.cx.apply_row(&Row { effects: BTreeSet::new(), tail: Some(t) });
            effects.extend(r.effects.iter().cloned());
            if r.tail.is_some() {
                tail = r.tail;
            }
        }
        Row { effects, tail }
    }

    /// Is `inner` covered by `outer` (both resolved)? Concrete ⊆ concrete and any tail matches.
    fn row_covered(&self, inner: &Row, outer: &Row) -> bool {
        let inner = self.cx.apply_row(inner);
        let outer = self.cx.apply_row(outer);
        inner.effects.is_subset(&outer.effects) && (inner.tail.is_none() || inner.tail == outer.tail)
    }

    fn resolved_effects(&self, r: &Row) -> BTreeSet<Effect> {
        self.cx.apply_row(r).effects
    }

    /// Structural opacity (R-5): Secret/Cap/Root, or a composite containing one.
    fn is_opaque(&self, t: &Type, visiting: &mut HashSet<TypeDefId>) -> bool {
        match self.cx.apply_type(t) {
            Type::Secret(_) | Type::Cap(_) | Type::Root => true,
            // R-5: ForeignPtr, PyObj, and a foreign lib handle are opaque (no str/==/serialize).
            Type::ForeignPtr | Type::PyObj | Type::Foreign(_) => true,
            // R-5 (Stage 6): a plugin handle is opaque. It binds a live broker node; stringifying
            // or comparing one would leak/forge authority identity, and it must never serialize.
            Type::Plugin(_) => true,
            Type::List(inner) | Type::Option(inner) => self.is_opaque(&inner, visiting),
            Type::Result(a, b) => self.is_opaque(&a, visiting) || self.is_opaque(&b, visiting),
            Type::Record(id, args) | Type::Sum(id, args) => {
                if args.iter().any(|a| self.is_opaque(a, visiting)) {
                    return true;
                }
                if !visiting.insert(id) {
                    return false;
                }
                let def = self.table.type_def(id);
                let mut genv = Genv::default();
                for (g, a) in def.generics.iter().zip(&args) {
                    genv.types.insert(g.clone(), a.clone());
                }
                let opaque = match &def.kind {
                    TypeDefKind::Record(fields) => fields.iter().any(|(_, ft)| self.type_expr_opaque(ft, &genv, visiting)),
                    TypeDefKind::Sum(variants) => variants.iter().any(|(_, fts)| fts.iter().any(|ft| self.type_expr_opaque(ft, &genv, visiting))),
                    TypeDefKind::Alias(inner) => self.type_expr_opaque(inner, &genv, visiting),
                };
                visiting.remove(&id);
                opaque
            }
            _ => false,
        }
    }

    /// Opacity check that works on an AST type (used inside record/sum definitions).
    fn type_expr_opaque(&self, t: &TypeExpr, genv: &Genv, visiting: &mut HashSet<TypeDefId>) -> bool {
        match t {
            TypeExpr::Named { path, args, .. } => {
                if path.segs.len() == 1 {
                    let name = &path.segs[0].name;
                    if let Some(v) = genv.types.get(name) {
                        return self.is_opaque(v, visiting);
                    }
                    match name.as_str() {
                        "Secret" | "Cap" | "Root" | "ForeignPtr" | "PyObj" => return true,
                        "List" | "Option" => return args.first().map(|a| self.type_expr_opaque(a, genv, visiting)).unwrap_or(false),
                        "Result" => return args.iter().any(|a| self.type_expr_opaque(a, genv, visiting)),
                        _ => {}
                    }
                    if self.table.foreigns.contains_key(name) {
                        return true;
                    }
                    if let Some(&id) = self.table.type_ix.get(name) {
                        let targs: Vec<Type> = args.iter().map(|_| Type::Unit).collect();
                        return self.is_opaque(&Type::Sum(id, targs), &mut visiting.clone());
                    }
                }
                false
            }
            TypeExpr::Fn { .. } => false,
        }
    }

    fn expect_type(&mut self, expected: &Type, actual: &Type, span: Span, msg: &str) {
        match self.cx.unify_type(expected, actual) {
            Ok(()) => {}
            Err(e) => {
                let ea = self.cx.apply_type(expected);
                let aa = self.cx.apply_type(actual);
                // A secret used where its plain type is expected is the "secret cannot flow"
                // case (§6.4 rule 1) — reported as DL0602, not a generic type mismatch.
                if secret_mismatch(&ea, &aa) {
                    self.diags.push(
                        Diagnostic::error(
                            "DL0602",
                            format!("secret value cannot flow here: `Secret[..]` is not `{}` (secrets never coerce)",
                                if matches!(ea, Type::Secret(_)) { format!("{aa}") } else { format!("{ea}") }),
                        )
                        .with_span(span, "a secret cannot be used where a plain value is expected"),
                    );
                    return;
                }
                // A DeluluLang closure can never become a `PyObj` (spec §4.4 / R-6a): a function type
                // unified against `PyObj` is the no-callbacks rule, not a plain mismatch (DL1302, not
                // DL0401). This catches a closure element inside a `List[PyObj]` literal, where the
                // failure surfaces at element-unification time.
                if fn_pyobj_mismatch(&ea, &aa) {
                    self.diags.push(
                        Diagnostic::error(
                            "DL1302",
                            "a function-typed value cannot cross the foreign boundary — no callbacks, by rule R-6a",
                        )
                        .with_span(span, "unverifiable code must never hold a re-entry point into verified code"),
                    );
                    return;
                }
                let code = match e {
                    UnifyError::RowConflict => "DL0504",
                    _ => "DL0401",
                };
                self.diags.push(
                    Diagnostic::error(code, format!("{msg}: expected `{ea}`, found `{aa}`"))
                        .with_span(span, "type mismatch here"),
                );
            }
        }
    }
}

/// The marshallable allowlist for a foreign signature (T-ForeignSig): exactly
/// `Int Float Bool Str Unit ForeignPtr`, with no type arguments. Everything else — `Secret[T]`,
/// `Cap[R]`, `Root`, `Plugin[_]`, `PyObj`, a lib handle, `List[..]`, user types — is DL1301.
fn is_marshallable_type_expr(t: &TypeExpr) -> bool {
    match t {
        TypeExpr::Named { path, args, .. } => {
            args.is_empty()
                && path.segs.len() == 1
                && matches!(
                    path.segs[0].name.as_str(),
                    "Int" | "Float" | "Bool" | "Str" | "Unit" | "ForeignPtr"
                )
        }
        TypeExpr::Fn { .. } => false,
    }
}

/// The span of the first function type appearing anywhere in `t` (including nested inside type
/// arguments, e.g. `List[fn(Int) -> Int]`) — the trigger for DL1302 (invariant 22).
fn first_fn_type(t: &TypeExpr) -> Option<Span> {
    match t {
        TypeExpr::Fn { span, .. } => Some(*span),
        TypeExpr::Named { args, .. } => args.iter().find_map(first_fn_type),
    }
}

/// Whether a resolved [`Type`] contains a function type at any depth — the DL1302 trigger for an
/// argument crossing into `PyObj`-land (spec §4.4 / R-6a). Mirrors [`first_fn_type`] on the lowered
/// side, used where the offending value is an inferred argument type rather than a written signature.
fn type_contains_fn(t: &Type) -> bool {
    match t {
        Type::Fn { .. } => true,
        Type::List(e) | Type::Option(e) | Type::Secret(e) => type_contains_fn(e),
        Type::Result(o, e) => type_contains_fn(o) || type_contains_fn(e),
        Type::Record(_, args) | Type::Sum(_, args) => args.iter().any(type_contains_fn),
        _ => false,
    }
}

/// Render a `TypeExpr` compactly for a diagnostic message, without lowering it (lowering an
/// unmarshallable type could emit unrelated diagnostics).
fn render_type_expr(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Named { path, args, .. } => {
            let base = path.dotted();
            if args.is_empty() {
                base
            } else {
                let inner = args.iter().map(render_type_expr).collect::<Vec<_>>().join(", ");
                format!("{base}[{inner}]")
            }
        }
        TypeExpr::Fn { .. } => "a function type".to_string(),
    }
}

/// Exactly one side is a function type and the other is `PyObj` — a closure being smuggled into (or
/// out of) `PyObj`-land, the no-callbacks rule (spec §4.4 / R-6a), reported as DL1302 not DL0401.
fn fn_pyobj_mismatch(a: &Type, b: &Type) -> bool {
    let is_fn = |t: &Type| matches!(t, Type::Fn { .. });
    (is_fn(a) && matches!(b, Type::PyObj)) || (matches!(a, Type::PyObj) && is_fn(b))
}

/// Exactly one side is a `Secret` and the other is a concrete non-secret type (not a variable) —
/// i.e. a secret is flowing into, or out of, a plain-typed position.
fn secret_mismatch(a: &Type, b: &Type) -> bool {
    let one_secret = matches!(a, Type::Secret(_)) ^ matches!(b, Type::Secret(_));
    let concrete = |t: &Type| !matches!(t, Type::Var(_));
    one_secret && concrete(a) && concrete(b)
}

// Small helper so RowAcc can absorb another accumulator ergonomically.
impl RowAcc {
    fn add_row_acc(&mut self, other: &RowAcc) {
        self.merge(other);
    }
}

/// Methods that invoke a function argument (so its row must surface — R-4).
fn is_higher_order_method(recv: &Type, method: &str) -> bool {
    matches!(
        (recv, method),
        (Type::List(_), "map") | (Type::List(_), "filter") | (Type::Secret(_), "map")
    )
}
