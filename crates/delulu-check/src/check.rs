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

/// The prelude's free builtins (§11) — every name `check_builtin_call` intercepts, plus `None`,
/// which is intercepted as a bare name.
///
/// These are resolved at the call site BEFORE user scope. A module that declares
/// `fn parse_int(...)` therefore had its definition accepted and every call to it silently
/// routed to the builtin instead, and the only symptom was a type error at the call site
/// naming a type the author never wrote. `resolve.rs` refuses the redefinition (DL0302) so the
/// collision is reported where it is caused. See `HARDENING_CAMPAIGN.md` C11.
pub const PRELUDE_BUILTINS: &[&str] =
    &["Ok", "Err", "Some", "None", "load", "assert", "assert_eq", "str", "len", "int", "float", "parse_int", "parse_float", "range", "push"];

/// The builtin TYPE names [`Checker::lower_type`] intercepts before it ever consults user scope.
///
/// This list must stay in lockstep with the `match name.as_str()` arm in `lower_type`; the test
/// `every_builtin_type_name_is_refused_as_a_user_type` walks this constant, and
/// `no_builtin_type_name_resolves_to_a_user_definition` is the behavioural half.
///
/// Why refusing is the only honest answer (`HARDENING_CAMPAIGN.md` C23): a `type Int = Secret[Str]`
/// declaration was *accepted* and then had no effect whatsoever, because `Int` is matched before
/// `table.type_ix` is searched. The author got no diagnostic anywhere, and a later reader of that
/// file — human or agent — would reasonably conclude `Int` meant a secret throughout. For a
/// language whose entire premise is that a program's authority can be read off its source, a
/// declaration that silently means nothing is the worst possible outcome: it misleads review
/// without ever failing. `Root`, `Cap`, `Secret`, and `Plugin` are on this list, so the mislead
/// lands squarely on the authority-bearing types.
pub const PRELUDE_TYPES: &[&str] = &[
    "Int", "Float", "Bool", "Str", "Unit", "Root", "List", "Option", "Result", "Secret", "Cap",
    "ForeignPtr", "PyObj", "Plugin", "Verified", "Contained",
];

/// The core effect names [`Checker::lower_row`] intercepts before it consults `user_effects` —
/// mirroring [`PRELUDE_TYPES`] for the effect namespace, and kept in lockstep with
/// [`crate::ty::Effect::core_from_name`].
///
/// This is the C23 case that actually touches Authority. `effect Write` was accepted and did
/// nothing: every `! {Write}` row still meant the CORE `Write` effect, so an author who believed
/// they had declared a private effect had in fact written the one that grants filesystem and
/// console reach — and the authority report could not tell the reader otherwise, because by the
/// time it ran there was only ever one `Write`.
pub const CORE_EFFECT_NAMES: &[&str] = &[
    "Read", "Write", "Net", "Clock", "Rand", "Declassify", "ForeignCall", "Load", "Async",
    "Actuate",
];


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
    /// Send/spawn argument nodes that are statically-proven iso MOVES (Stage 7 phase 7i):
    /// what the runtime's `--debug-rcaps` verifies unaliased at each actor boundary.
    pub iso_moves: HashSet<NodeId>,
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

/// What a row boundary belongs to — `function `f`` or `test "name"` (Stage 8, phase 8a).
/// Carries exactly what DL0501/DL0502 and the `add_effect_to_row` repair need, so fns and
/// tests share one boundary check without synthesizing a fake `FnDecl`.
struct RowSubject<'a> {
    /// Leads the message: `function `f`` keeps every pre-Stage-8 diagnostic byte-identical.
    desc: String,
    /// Where "declared row is here" points (fn name / test name string).
    head_span: Span,
    row: Option<&'a RowExpr>,
    /// Insertion point for a fresh `! {…}` row when none was written.
    body_start: u32,
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
        cyclic_aliases: std::collections::HashSet::new(),
        node_types_raw: HashMap::new(),
        node_row_accs: HashMap::new(),
    };
    // BEFORE anything lowers a type: validate the alias declarations themselves. Cycles must be
    // known first (`lower_type` expands an alias by recursing, so a cycle is a stack overflow, not a
    // diagnostic), and an alias target must resolve where it is WRITTEN rather than only where it is
    // used. See `check_type_aliases`.
    checker.check_type_aliases(module);
    for item in &module.items {
        match item {
            Item::Fn(f) => checker.check_fn(f),
            // Stage 7 (phase 7e): T-Actor — fields, exactly one `new`, behaviors, sync fns.
            Item::Actor(a) => checker.check_actor(a),
            // Stage 8 (phase 8a): T-Test — typed like a Unit-returning fn, compiled out of
            // builds. Checked HERE so a broken test surfaces at `delulu check` — "compiled
            // out" must never mean "diagnosed never" (the kitchen rule's skip branch).
            Item::Test(t) => checker.check_test(t),
            _ => {}
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
    // The rule is FAIL-CLOSED. R-6a is a security rule, so "we could not tell" must refuse, never
    // skip: an unresolved variable anywhere in `F` can be instantiated with a function type at some
    // call site — and because a generic's variables are instantiated FRESH per call site, the
    // body's own variable is never unified with the caller's argument. Without this, a closure
    // launders into an opaque module through a generic helper and R-6a never fires (DL1509).
    let pending_gets = std::mem::take(&mut checker.pending_gets);
    for (span, f_var, class_ty) in pending_gets {
        if !matches!(checker.cx.apply_type(&class_ty), Type::Contained) {
            continue;
        }
        let f = checker.cx.apply_type(&Type::Var(f_var));
        match &f {
            // A function-typed parameter that is CONCRETELY present: the R-6a refusal proper.
            Type::Fn { params, .. } if params.iter().any(type_contains_fn) => {
                checker.diags.push(
                    Diagnostic::error(
                        "DL0803",
                        "a function-typed value cannot be passed to a Contained plugin export — no callbacks, by rule R-6a",
                    )
                    .with_span(span, "this `get` types an export of an opaque module")
                    .with_secondary_span(
                        span,
                        "an opaque module holding a re-entry point into verified code could invoke it at times no caller's row accounts for",
                    ),
                );
            }
            // Concrete and function-free: the only accepting case.
            Type::Fn { .. } if !type_contains_var(&f) => {}
            // Everything else is underdetermined — refuse and demand a concrete signature.
            _ => {
                checker.diags.push(
                    Diagnostic::error(
                        "DL1509",
                        format!(
                            "a Contained plugin export's signature must be concrete at the `get` site — `{}` still contains an unresolved type, so rule R-6a cannot be decided here",
                            checker.ty(&f)
                        ),
                    )
                    .with_span(span, "annotate this `get` with a concrete function signature")
                    .with_secondary_span(
                        span,
                        "an unresolved type could be instantiated with a function type at some call site, which R-6a forbids for an opaque module — this refusal is fail-closed, never a guess",
                    ),
                );
            }
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

    // Stage 7 (phases 7c–7f): the reference-capability pass — the SECOND checking axis, run
    // over the settled types so it never participates in unification (build-order deviation 4).
    let fn_types_settled: HashMap<String, Type> =
        checker.fn_types.iter().map(|(k, v)| (k.clone(), checker.cx.apply_type(v))).collect();
    let (rcap_diags, iso_moves) =
        crate::rcap_check::check_rcaps(module, table, &node_types, &fn_types_settled);
    checker.diags.extend(rcap_diags);

    CheckResult {
        diags: checker.diags,
        iso_moves,
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
        // A plugin-manifest signature is lowered against the HOST's already-validated decl table, so
        // no alias in scope here can be cyclic — `check_module` refused any that were. Empty is the
        // correct value, not a shortcut.
        cyclic_aliases: std::collections::HashSet::new(),
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
    /// Type aliases that participate in a cycle, by name. Populated once, before anything lowers a
    /// type, by [`Checker::check_type_aliases`]. `lower_type`'s alias arm EXPANDS the target by
    /// recursing, so a cycle there is unbounded recursion — `type A = A` plus one use of `A` aborted
    /// the compiler with a stack overflow (`HARDENING_CAMPAIGN.md` C54). Consulting this set is what
    /// makes that structurally impossible rather than merely diagnosed.
    cyclic_aliases: std::collections::HashSet<String>,
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
    /// How many `while`/`for` bodies enclose the statement being checked. `break`/`continue` are
    /// legal only when this is > 0 (Tier 1); a bare `break` at function level is DL0412.
    loop_depth: u32,
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
        // A function is absent from the table only when `resolve` refused to register it and
        // already said why — a name that collides with a prelude builtin, today. Returning is
        // correct: there is no signature to check against, and the reason is already reported.
        // This was an `expect("fn in table")`, i.e. a host panic one skipped registration away,
        // and a host panic is never an acceptable answer to a bad program (Stage 9, D15).
        let Some(sig) = self.table.fns.get(&f.name.name).cloned() else {
            return;
        };
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
            loop_depth: 0,
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
        let subj = RowSubject {
            desc: format!("function `{}`", f.name.name),
            head_span: f.name.span,
            row: f.row.as_ref(),
            body_start: f.body.span.start,
        };
        self.check_row_subset(&body_row, &declared_row, &subj);

        // Finalize facts: the function's authority is its declared row (sound upper bound).
        let mut facts = std::mem::take(&mut ctx.facts);
        facts.effects = self.resolved_effects(&declared_row);
        facts.pure = facts.effects.is_empty();
        facts.declassifies = facts.effects.contains(&Effect::Declassify);
        self.facts.insert(f.name.name.clone(), facts);
    }

    // ===== actors (Stage 7 phase 7e: T-Actor / T-Ctor / T-Behavior / T-SyncMethod) ======

    /// Check an `actor` declaration (spec §4). `var` fields are actor-internal state — owned,
    /// isolated, reached only via the actor's own turn — NOT the banned module-level ambient
    /// `var`. Members check like functions with `self : ref ActorType` bound; parameter
    /// SENDABILITY (DL1601) is the rcap pass's rule, not this one (two axes, two owners).
    fn check_actor(&mut self, a: &ActorDecl) {
        let Some(adef) = self.table.actors.get(&a.name.name).cloned() else { return };

        // Actor generics: row-kinded when they appear as a row tail anywhere in a member
        // signature (the Promise[T] + `! e` pattern), type-kinded otherwise.
        let row_kinded = actor_row_kinded_generics(&adef);
        let mut genv = Genv::default();
        let mut targs: Vec<Type> = Vec::new();
        for g in &adef.generics {
            if row_kinded.contains(g) {
                let v = self.cx.fresh_row_var();
                genv.rows.insert(g.clone(), v);
            } else {
                let v = self.cx.fresh_type();
                genv.types.insert(g.clone(), v.clone());
                targs.push(v);
            }
        }
        let self_ty = Type::Actor(a.name.name.clone(), targs);

        // Field types must lower cleanly (T-Actor).
        for (_, te, _) in &adef.fields {
            let _ = self.lower_type(te, &genv, &mut FnFacts::default());
        }

        // Construction is a send to the new actor (T-Ctor): checked like a behavior.
        self.check_actor_member(&a.name.name, "new", &a.ctor.params, &a.ctor.row, &a.ctor.body, None, &genv, &self_ty);
        for b in &a.behaviors {
            self.check_actor_member(&a.name.name, &b.name.name, &b.params, &b.row, &b.body, None, &genv, &self_ty);
        }
        for f in &a.fns {
            self.check_actor_member(&a.name.name, &f.name.name, &f.params, &f.row, &f.body, f.ret.as_ref(), &genv, &self_ty);
        }

        // Definite initialization (v0.7, flow-insensitive): `new` must assign every field —
        // a field read before any assignment would be a value from nowhere. Same fault
        // family as a missing record field (DL0405).
        let mut assigned = HashSet::new();
        collect_self_field_assigns(&a.ctor.body, &mut assigned);
        for (fname, _, _) in &adef.fields {
            if !assigned.contains(fname) {
                self.diags.push(
                    Diagnostic::error(
                        "DL0405",
                        format!("actor `{}`'s constructor never assigns field `{fname}`", a.name.name),
                    )
                    .with_span(a.ctor.span, "every field must be assigned in `new`"),
                );
            }
        }
    }

    /// One actor member (ctor / behavior / sync fn), checked with `self : ref ActorType`.
    /// Facts and signature types register under `Actor.member` so authority reporting and
    /// the rcap pass see them like any function.
    #[allow(clippy::too_many_arguments)]
    fn check_actor_member(
        &mut self,
        actor: &str,
        member: &str,
        params: &[Param],
        row: &Option<RowExpr>,
        body: &Block,
        ret: Option<&TypeExpr>,
        genv: &Genv,
        self_ty: &Type,
    ) {
        let param_types: Vec<Type> =
            params.iter().map(|p| self.lower_type(&p.ty, genv, &mut FnFacts::default())).collect();
        let ret_ty = match ret {
            Some(t) => self.lower_type(t, genv, &mut FnFacts::default()),
            None => Type::Unit,
        };
        let declared_row = match row {
            Some(r) => self.lower_row(r, genv),
            None => Row::pure(),
        };
        let key = format!("{actor}.{member}");
        self.fn_types.insert(
            key.clone(),
            Type::Fn { params: param_types.clone(), ret: Box::new(ret_ty.clone()), row: declared_row.clone() },
        );
        let ret_err = match &ret_ty {
            Type::Result(_, e) => Some((**e).clone()),
            _ => None,
        };
        let mut ctx = FnCtx {
            name: key.clone(),
            genv: genv.clone(),
            ret: ret_ty.clone(),
            ret_err,
            locals: vec![HashMap::new()],
            facts: FnFacts::default(),
            loop_depth: 0,
        };
        ctx.bind("self", self_ty.clone());
        for (p, ty) in params.iter().zip(&param_types) {
            self.note_caps_in(ty, &mut ctx.facts);
            ctx.bind(&p.name.name, ty.clone());
        }
        let (body_ty, body_row) = self.check_block(body, &mut ctx);
        if ret.is_some() {
            self.expect_type(&ret_ty, &body_ty, body.span, "method body type must match the return type");
        }
        self.check_member_row_subset(&body_row, &declared_row, body.span, &key);
        let mut facts = std::mem::take(&mut ctx.facts);
        facts.effects = self.resolved_effects(&declared_row);
        facts.pure = facts.effects.is_empty();
        facts.declassifies = facts.effects.contains(&Effect::Declassify);
        self.facts.insert(key, facts);
    }

    /// The T-Fn boundary check applied to an actor member: ε_body ⊆ ε_declared (DL0501).
    /// Member declarations carry no `FnDecl` shape, so this reports without the auto-repair
    /// edit machinery — the message still names the exact missing effects.
    fn check_member_row_subset(&mut self, body: &RowAcc, declared: &Row, span: Span, what: &str) {
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
            body_effects.iter().filter(|e| !declared_resolved.effects.contains(*e)).cloned().collect();
        if !missing.is_empty() || uncovered_tail {
            let list = missing.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
            let msg = if missing.is_empty() {
                format!("`{what}` performs effects its declared row does not cover")
            } else {
                format!("`{what}` performs effect(s) not in its declared row: {list}")
            };
            self.diags.push(Diagnostic::error("DL0501", msg).with_span(span, "declare these effects in the member's row"));
        }
    }

    /// T-Spawn (spec §4): `spawn A(ā)` — `ā` match `A.new`'s params; type `tag A`; row
    /// `{Async} ∪ row(A.new)`. Generics instantiate fresh per spawn site (type-kinded flow
    /// into the actor reference's arguments; row-kinded get fresh row variables, R-4-style).
    fn check_spawn(&mut self, actor: &Path, args: &[Expr], span: Span, ctx: &mut FnCtx) -> (Type, RowAcc) {
        let mut acc = RowAcc::default();
        let arg_tys: Vec<(Type, Span)> = args
            .iter()
            .map(|a| {
                let (t, r) = self.check_expr(a, ctx);
                acc.add_row_acc(&r);
                (self.cx.apply_type(&t), a.span())
            })
            .collect();
        let name = actor.segs.last().expect("path has segments").name.clone();
        let Some(adef) = self.table.actors.get(&name).cloned() else {
            self.diags.push(
                Diagnostic::error("DL0301", format!("unknown actor `{}`", actor.dotted()))
                    .with_span(span, "`spawn` needs an actor declared in this module"),
            );
            return (self.cx.fresh_type(), acc);
        };
        let row_kinded = actor_row_kinded_generics(&adef);
        let mut agenv = Genv::default();
        let mut targs: Vec<Type> = Vec::new();
        for g in &adef.generics {
            if row_kinded.contains(g) {
                let v = self.cx.fresh_row_var();
                agenv.rows.insert(g.clone(), v);
            } else {
                let v = self.cx.fresh_type();
                agenv.types.insert(g.clone(), v.clone());
                targs.push(v);
            }
        }
        if adef.ctor_params.len() != arg_tys.len() {
            self.diags.push(
                Diagnostic::error(
                    "DL0403",
                    format!("`{name}.new` expects {} argument(s), found {}", adef.ctor_params.len(), arg_tys.len()),
                )
                .with_span(span, "wrong number of constructor arguments"),
            );
        }
        for (p, (at, aspan)) in adef.ctor_params.iter().zip(&arg_tys) {
            let expected = self.lower_type(&p.ty, &agenv, &mut ctx.facts);
            self.expect_type(&expected, at, *aspan, "spawn argument type mismatch");
        }
        acc.add_effect(Effect::Async);
        if let Some(r) = &adef.ctor_row {
            let row = self.lower_row(r, &agenv);
            acc.add_row(&row);
        }
        ctx.facts.callees.insert(format!("{name}.new"));
        (Type::Actor(name, targs), acc)
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
        // An alias standing in for a function type is still the no-callbacks rule: R-6a is about
        // what the type MEANS, and `type F = fn(Int) -> Int` means a re-entry point. Reporting it as
        // "F does not marshal" would be true and useless.
        if let Some(fspan) = self.alias_chain_reaches_fn(t) {
            self.diags.push(
                Diagnostic::error(
                    "DL1302",
                    "a function-typed value cannot cross the foreign boundary — no callbacks, by rule R-6a",
                )
                .with_span(t.span(), "this alias expands to a function type")
                .with_secondary_span(fspan, "the function type is declared here"),
            );
            return;
        }
        if !is_marshallable_type_expr(t) {
            // An alias whose expansion WOULD marshal is the confusing case (`HARDENING_CAMPAIGN.md`
            // C24): `type Meters = Int` reads as an Int everywhere else in the language, so the bare
            // "only Int, Float, … marshal" message looked like the compiler had lost track of what
            // `Meters` was. Foreign signatures are matched by NAME, deliberately — the interpreter's
            // `lower_foreign_sig` and the WASM host share exactly one lowering and neither can see
            // this module's aliases, so expanding here and not there is how ABI confusion starts.
            // Name the rule, and hand over the edit.
            if let Some(target) = self.marshallable_alias_target(t) {
                let span = t.span();
                self.diags.push(
                    Diagnostic::error(
                        "DL1301",
                        format!(
                            "`{}` is an alias for `{target}`, and a foreign signature must name the marshallable type directly",
                            render_type_expr(t)
                        ),
                    )
                    .with_span(
                        span,
                        format!("write `{target}` here — both engines marshal by type name through one shared path that cannot see module aliases"),
                    )
                    .with_repair(Repair {
                        id: "name-the-marshallable-type",
                        confidence: Confidence::Exact,
                        authority_widening: false,
                        requires_human: false,
                        edits: vec![Edit {
                            file: span.file,
                            start_byte: span.start,
                            end_byte: span.end,
                            insert: target,
                        }],
                        reason: None,
                    }),
                );
                return;
            }
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

    /// Walk a single-segment alias chain and return the marshallable builtin it ultimately names.
    ///
    /// The walk is BOUNDED, and it is kept bounded even though `check_type_aliases` now refuses a
    /// cyclic alias outright (C54): this runs on the foreign-signature path, the bound costs nothing,
    /// and a resolver that cannot loop is worth more than one that relies on an earlier pass having
    /// run. 32 hops is far past any real alias chain.
    fn marshallable_alias_target(&self, t: &TypeExpr) -> Option<String> {
        let mut cur = t.clone();
        for _ in 0..32 {
            let TypeExpr::Named { path, args, .. } = &cur else { return None };
            if path.segs.len() != 1 || !args.is_empty() {
                return None;
            }
            let name = path.segs[0].name.clone();
            if MARSHALLABLE_TYPE_NAMES.contains(&name.as_str()) {
                return Some(name);
            }
            let &id = self.table.type_ix.get(&name)?;
            match &self.table.type_def(id).kind {
                TypeDefKind::Alias(inner) => cur = inner.clone(),
                _ => return None,
            }
        }
        None
    }

    /// The span of the function type an alias chain expands to, if any (same bounded walk as
    /// [`Self::marshallable_alias_target`], same reason).
    fn alias_chain_reaches_fn(&self, t: &TypeExpr) -> Option<Span> {
        let mut cur = t.clone();
        for _ in 0..32 {
            match &cur {
                TypeExpr::Fn { span, .. } => return Some(*span),
                TypeExpr::Named { path, args, .. } if path.segs.len() == 1 && args.is_empty() => {
                    let name = &path.segs[0].name;
                    if MARSHALLABLE_TYPE_NAMES.contains(&name.as_str()) {
                        return None;
                    }
                    let &id = self.table.type_ix.get(name)?;
                    match &self.table.type_def(id).kind {
                        TypeDefKind::Alias(inner) => cur = inner.clone(),
                        _ => return None,
                    }
                }
                _ => return None,
            }
        }
        None
    }

    /// Validate every type ALIAS declaration, before any type is lowered. Two rules, in this order
    /// because the second is unsafe without the first.
    ///
    /// **1. An alias chain may not cycle** (`HARDENING_CAMPAIGN.md` C54, reshaping C16). `lower_type`
    /// expands an alias by recursing into its target, so a cycle is unbounded recursion. `type A = A`
    /// with a single use of `A` aborted the compiler — `has overflowed its stack`, exit
    /// `0xC00000FD` — and so did `type A = B; type B = A` and `type A = List[A]`. This was a hard
    /// crash on ordinary input, which for anything that compiles code it did not write (an editor, a
    /// CI runner, a registry) is a denial of service.
    ///
    /// C16 recorded cyclic aliases as *hygiene* on the evidence that 5000-deep terminating chains
    /// resolve and secrets cannot launder through a cycle. Both of those hold. What that pass never
    /// tested was a cycle that is actually USED, and the declaration alone is harmless precisely
    /// because nothing lowers it. The verdict was right about what it measured and wrong about the
    /// class.
    ///
    /// Note what the crash was invisible to: a stack overflow aborts the process without producing
    /// `panicked at`, so the no-panic sweeps — which match that message (D44c) — could not see it.
    /// That is the third time a gate has been blind to the failure it exists to catch.
    ///
    /// **Only alias→alias edges can cycle**, which is what makes the graph small: a reference to a
    /// record or a sum terminates, because `lower_type` returns `Type::Record`/`Type::Sum` for those
    /// without expanding anything.
    ///
    /// **2. An alias target must resolve where it is WRITTEN** (C53). `type Meters = Metres` — a
    /// typo — used to check clean, with DL0301 arriving only at a use site; in a library whose own
    /// code never uses the alias, the diagnostic landed on a consumer who did not make the mistake.
    /// This is the C11/C23 family: a declaration accepted and then silently inert. Lowering the
    /// target here reports it at the declaration, and reuses the real resolver rather than
    /// duplicating its notion of what names exist.
    fn check_type_aliases(&mut self, module: &Module) {
        use delulu_syntax::ast::TypeDeclKind;

        // The alias graph: alias name -> the alias names its target mentions, anywhere.
        // "Anywhere" and not just at the head, because `type A = List[A]` recurses through an
        // argument just as surely as `type A = A` recurses through the head.
        let mut targets: Vec<(&Ident, &TypeExpr)> = Vec::new();
        for item in &module.items {
            if let Item::Type(td) = item {
                if let TypeDeclKind::Alias(t) = &td.kind {
                    targets.push((&td.name, t));
                }
            }
        }
        if targets.is_empty() {
            return;
        }
        let is_alias = |name: &str| {
            self.table
                .type_ix
                .get(name)
                .is_some_and(|&id| matches!(self.table.type_def(id).kind, TypeDefKind::Alias(_)))
        };
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        for (name, t) in &targets {
            let mut mentioned = Vec::new();
            collect_named_types(t, &mut mentioned);
            mentioned.retain(|n| is_alias(n));
            edges.insert(name.name.clone(), mentioned);
        }

        // Depth-first cycle detection over that graph. `on_stack` is the current path, so any edge
        // back into it closes a cycle — including a self-edge.
        let mut cyclic: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (name, _) in &targets {
            let mut on_stack: Vec<String> = Vec::new();
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
            find_alias_cycle(&name.name, &edges, &mut on_stack, &mut seen, &mut cyclic);
        }

        // One diagnostic per declaration that participates, pointing at the declaration a reader can
        // actually edit, and naming the chain so a multi-step cycle is followable.
        for (name, _) in &targets {
            if !cyclic.contains(&name.name) {
                continue;
            }
            let mut chain = vec![name.name.clone()];
            let mut cur = name.name.clone();
            for _ in 0..32 {
                let Some(next) = edges.get(&cur).and_then(|v| v.iter().find(|n| cyclic.contains(*n))) else {
                    break;
                };
                if chain.len() > 1 && *next == chain[0] {
                    chain.push(next.clone());
                    break;
                }
                chain.push(next.clone());
                cur = next.clone();
            }
            self.diags.push(
                Diagnostic::error(
                    "DL0304",
                    format!(
                        "type alias `{}` is part of a cycle ({}) — an alias must eventually name a \
                         real type, and expanding this one would never terminate",
                        name.name,
                        chain.join(" = ")
                    ),
                )
                .with_span(name.span, "this alias expands to itself"),
            );
        }
        self.cyclic_aliases = cyclic;

        // Now the targets can be lowered safely: `lower_type`'s alias arm consults
        // `cyclic_aliases` and refuses to recurse, so the pass below cannot overflow.
        for (name, t) in targets {
            if self.cyclic_aliases.contains(&name.name) {
                continue; // already reported; lowering adds nothing but noise
            }
            let mut genv = Genv::default();
            if let Some(&id) = self.table.type_ix.get(&name.name) {
                for g in self.table.type_def(id).generics.clone() {
                    genv.types.insert(g, self.cx.fresh_type());
                }
            }
            self.lower_type(t, &genv, &mut FnFacts::default());
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

    /// T-Test (Stage 8, phase 8a): a `test` body types like a `Unit`-returning fn with
    /// `test_root: Root` bound; the declared row (omitted = pure) bounds the body — the same
    /// DL0501 boundary and `add_effect_to_row` repair as everywhere else. Facts are
    /// deliberately NOT recorded: tests are compiled out of builds (invariant 41), so they
    /// must never appear in authority reports (invariant 38 keeps prior output byte-stable).
    fn check_test(&mut self, t: &TestDecl) {
        let genv = Genv::default();
        let declared_row = match &t.row {
            Some(r) => self.lower_row(r, &genv),
            None => Row::pure(),
        };
        let mut ctx = FnCtx {
            name: format!("test \"{}\"", t.name),
            genv,
            ret: Type::Unit,
            ret_err: None,
            locals: vec![HashMap::new()],
            facts: FnFacts::default(),
            loop_depth: 0,
        };
        // The runner (8g) scopes this Root by the test manifest; the TYPE is just Root.
        self.note_caps_in(&Type::Root, &mut ctx.facts);
        ctx.bind("test_root", Type::Root);

        let (body_ty, body_row) = self.check_block(&t.body, &mut ctx);
        self.expect_type(&Type::Unit, &body_ty, t.body.span, "a test body must produce `Unit`");

        let subj = RowSubject {
            desc: format!("test \"{}\"", t.name),
            head_span: t.name_span,
            row: t.row.as_ref(),
            body_start: t.body.span.start,
        };
        self.check_row_subset(&body_row, &declared_row, &subj);
    }

    /// The subset check at the function boundary (T-Fn). Emits DL0501 (undeclared effect,
    /// with an authority-widening repair) and DL0502 (declared-but-unused, narrowing repair).
    fn check_row_subset(&mut self, body: &RowAcc, declared: &Row, subj: &RowSubject) {
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
                format!("{} may perform an effect not declared in its row", subj.desc)
            } else {
                format!(
                    "{} performs effect{} `{}` not declared in its row",
                    subj.desc,
                    if missing.len() == 1 { "" } else { "s" },
                    list
                )
            };
            let mut diag = Diagnostic::error("DL0501", msg)
                .with_span(subj.head_span, "declared row is here");
            if !missing.is_empty() {
                // Catalog args (Stage 8, spec §6.1): `{fn}` carries the full subject desc so
                // localized prose stays honest for `test` blocks too.
                diag = diag
                    .with_arg("fn", subj.desc.clone())
                    .with_arg("effect", list.clone())
                    .with_repair(self.add_effect_repair(subj, &declared_resolved, &missing));
            }
            self.diags.push(diag);
        }

        // DL0502: a declared concrete effect the body never performs (narrowing → safe).
        let unused: Vec<Effect> =
            declared_resolved.effects.difference(&body_effects).cloned().collect();
        if let Some(row) = subj.row.filter(|_| !unused.is_empty()) {
            let list = unused.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
            self.diags.push(
                Diagnostic::warning(
                    "DL0502",
                    format!("{} declares effect{} `{}` it never performs", subj.desc,
                        if unused.len() == 1 { "" } else { "s" }, list),
                )
                .with_arg("fn", subj.desc.clone())
                .with_arg("effect", list.clone())
                .with_span(row.span, "declared here")
                // NE-07: this repair has never carried edits, and the flags said otherwise.
                // Narrowing a row is not a mechanical deletion — the effect may be declared
                // because a caller is about to need it, or because the row is a published
                // interface — so the decision is a person's and the flags now say so. Removing it
                // NARROWS authority, which is why it is `safe` rather than `authority_widening`;
                // the two are different questions and the answers stay separate.
                .with_repair(Repair {
                    id: "remove_effect_from_row",
                    confidence: Confidence::Safe,
                    authority_widening: false,
                    requires_human: true,
                    edits: vec![],
                    reason: Some(
                        "narrowing a declared row is a review decision, not a mechanical edit:                          the effect may be declared for a caller that does not perform it yet, or                          because the row is a published interface. Delete it yourself, or leave it.",
                    ),
                }),
            );
        }
    }

    fn add_effect_repair(&self, subj: &RowSubject, declared: &Row, missing: &[Effect]) -> Repair {
        let add = missing.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
        // Insert into an existing `!{...}` or add a fresh row after the signature.
        let (start, end, text) = match subj.row {
            Some(r) => {
                // Insert before the closing brace of the row span.
                let insert_at = r.span.end.saturating_sub(1);
                let sep = if declared.effects.is_empty() { "" } else { ", " };
                (insert_at, insert_at, format!("{sep}{add}"))
            }
            None => {
                let at = subj.body_start;
                (at, at, format!("! {{{add}}} "))
            }
        };
        Repair {
            id: "add_effect_to_row",
            confidence: Confidence::Exact,
            authority_widening: true,
            requires_human: false,
            edits: vec![Edit { file: subj.head_span.file, start_byte: start, end_byte: end, insert: text }],
            reason: None,
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
                    self.expect_type_coded(&Type::Bool, &ct, cond.span(), "`while` condition must be Bool", Some("DL0408"));
                    ctx.loop_depth += 1;
                    let (_, br) = self.check_block(body, ctx);
                    ctx.loop_depth -= 1;
                    acc.add_row_acc(&br);
                    value_ty = Type::Unit;
                }
                Stmt::For { var, iter, body, .. } => {
                    // The iterable is checked in the enclosing scope; its effects join the block's.
                    let (it, ir) = self.check_expr(iter, ctx);
                    acc.add_row_acc(&ir);
                    // It must be a `List[T]`; the loop variable is then `T`. On anything else, emit
                    // DL0411 and bind a fresh type var so the body still checks against *something*
                    // rather than cascading — the same fail-soft the index lvalue uses.
                    let elem = match self.cx.apply_type(&it) {
                        Type::List(inner) => *inner,
                        other => {
                            self.diags.push(
                                Diagnostic::error(
                                    "DL0411",
                                    format!("`for` iterates a `List`, but this is `{}`", self.ty(&other)),
                                )
                                .with_span(iter.span(), "expected a `List` here"),
                            );
                            self.cx.fresh_type()
                        }
                    };
                    // Bind the loop variable in a scope of its own so it does not leak past the loop,
                    // then check the body one loop level deeper so `break`/`continue` are legal in it.
                    ctx.push_scope();
                    ctx.bind(&var.name, elem);
                    ctx.loop_depth += 1;
                    let (_, br) = self.check_block(body, ctx);
                    ctx.loop_depth -= 1;
                    ctx.pop_scope();
                    acc.add_row_acc(&br);
                    value_ty = Type::Unit;
                }
                Stmt::Break { span } | Stmt::Continue { span } => {
                    // A gate that dies in the else branch: `break`/`continue` outside any loop is
                    // DL0412, refused rather than silently accepted and then mishandled at run time.
                    if ctx.loop_depth == 0 {
                        let word = if matches!(stmt, Stmt::Break { .. }) { "break" } else { "continue" };
                        self.diags.push(
                            Diagnostic::error("DL0412", format!("`{word}` is only valid inside a `while` or `for` loop"))
                                .with_span(*span, "not inside a loop"),
                        );
                    }
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
            effects.extend(r.effects);
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
                            Diagnostic::error("DL0405", format!("cannot index a value of type `{}`", self.ty(&other)))
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
            // T-Spawn (Stage 7 phase 7f, spec §4): `spawn A(ā)` types as `tag A` with row
            // `{Async} ∪ row(A.new)` — creating an actor is itself the Async effect, and the
            // constructor's effects are causally the spawner's (invariant 35).
            Expr::Spawn { actor, args, span, .. } => self.check_spawn(actor, args, *span, ctx),
            Expr::Consume { name, span, .. } => (self.check_var(&Path { segs: vec![name.clone()] }, *span, ctx), RowAcc::default()),
            Expr::Recover { body, .. } => self.check_block(body, ctx),
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
            if name.as_str() == "None" { return Type::Option(Box::new(self.cx.fresh_type())) }
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
                    Diagnostic::error("DL0404", format!("value of type `{}` is not callable", self.ty(&other)))
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
    ///
    /// Every name here is intercepted at the CALL site before user scope is consulted, which is
    /// why `PRELUDE_BUILTINS` exists and why redefining one is refused at the declaration.
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
            // Stage 8 (phase 8a, spec §2/§8): the two test assertions — prelude builtins,
            // PURE (they add no effects; a pure test stays pure through its asserts).
            "assert" => {
                let ts = check_args(self, ctx, &mut acc);
                if ts.len() != 1 {
                    self.diags.push(
                        Diagnostic::error("DL0401", "assert takes exactly one Bool condition")
                            .with_span(span, "assert(condition)"),
                    );
                } else if let Some(t) = ts.first() {
                    self.expect_type(&Type::Bool, t, span, "assert takes a Bool condition");
                }
                Some((Type::Unit, acc))
            }
            "assert_eq" => {
                let ts = check_args(self, ctx, &mut acc);
                if ts.len() != 2 {
                    self.diags.push(
                        Diagnostic::error("DL0401", "assert_eq takes exactly two values of one type")
                            .with_span(span, "assert_eq(left, right)"),
                    );
                } else if let (Some(a), Some(b)) = (ts.first(), ts.get(1)) {
                    self.expect_type(a, b, span, "assert_eq compares two values of one type");
                    // R-5: opaque types have no structural equality — comparing secrets in
                    // tests is refused exactly like `==` refuses it everywhere else.
                    if self.is_opaque(a, &mut HashSet::new())
                        || self.is_opaque(b, &mut HashSet::new())
                    {
                        self.diags.push(
                            Diagnostic::error(
                                "DL0605",
                                "opaque types (Secret/Cap/Root) have no structural equality",
                            )
                            .with_span(span, "use `Secret.verify` for secrets"),
                        );
                    }
                }
                Some((Type::Unit, acc))
            }
            "str" => {
                let ts = check_args(self, ctx, &mut acc);
                if let Some(t) = ts.first() {
                    if self.is_opaque(t, &mut HashSet::new()) {
                        self.diags.push(
                            Diagnostic::error("DL0604", format!("value of type `{}` cannot be stringified", self.ty(t)))
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
            "parse_float" => {
                check_args(self, ctx, &mut acc);
                Some((Type::Option(Box::new(Type::Float)), acc))
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
            match arg_tys.first() {
                Some((Type::Fn { row, .. }, _)) => {
                    let r = self.cx.apply_row(row);
                    acc.add_row(&r);
                }
                // FAIL CLOSED — campaign finding C88, and the single worst defect this campaign
                // found. This used to be an `if let` with no `else`: when the argument's type was
                // not *syntactically* a function the row was silently dropped, and "silently
                // dropped" means treated as pure.
                //
                // A bare type parameter is enough to reach it, because a rigid generic can never be
                // known to be a function:
                //
                //     fn go[T](xs: List[Int], f: T) -> Int { let ys = xs.map(f)  1 }
                //     go([1], fn(x: Int) -> Int ! {Write} { out.println("escaped"); x })
                //
                // That program checked clean, `delulu authority` reported "(none — provably pure)"
                // and listed `go` and `main` under "pure fns", `delulu why Write` answered "program
                // cannot perform `Write`" — and it printed at run time. Verbatim the F-3/F-4 exploit
                // `SOUNDNESS_AUDIT.md` calls a total soundness failure, reopened through the one
                // branch the law was never written for.
                //
                // R-3's discipline is that rows unify by EQUALITY and there is no subsumption: a row
                // the checker cannot determine is therefore not `{}`, it is unknown, and a builtin
                // that will *invoke* this value cannot be allowed to assume the pure case. This is
                // the project's own skip-branch rule — write the "what if the checker could not
                // tell" case before claiming the law holds.
                //
                // `Secret.map` is deliberately excluded: it has its own arm below that refuses the
                // same shape with a message tuned to secrets, and reporting both would give two
                // diagnostics for one mistake. `exactly_one_diagnostic_for_an_unknown_row_on_secret_map`
                // pins that, so the exclusion cannot rot into a silent gap if that arm ever moves.
                Some((other, aspan)) if !matches!(rt, Type::Secret(_)) => {
                    self.diags.push(
                        Diagnostic::error(
                            "DL0401",
                            format!(
                                "argument type mismatch: `{}` invokes its function argument, so that argument's \
                                 effect row must be known here — found `{}`",
                                name.name,
                                self.ty(other)
                            ),
                        )
                        .with_span(
                            *aspan,
                            "give this a function type with an explicit row; a bare type parameter \
                             hides the row, and an unknown row cannot be assumed empty",
                        ),
                    );
                }
                // Arity is checked by the ordinary call rules; nothing to add here. The `Secret`
                // receiver excluded above lands here too, and is refused by its own arm.
                _ => {}
            }
        }

        // T-Send (Stage 7 phase 7f, spec §4): `a.beh(ā)` on an actor reference types as
        // `Unit` with row `{Async} ∪ row(beh)` — every send site's row contains the target
        // behavior's row, which is exactly what makes whole-program authority `row(main)`
        // across the actor boundary (invariant 35; the DL0501 of criterion 5 falls out of
        // the ordinary boundary check). Row-kinded actor generics instantiate FRESH per
        // send site (the R-4 law — Promise.then's `e` plumbing, criterion 9).
        // Argument SENDABILITY (unconsumed iso → DL1601 + consume repair) is the rcap
        // pass's rule.
        if let Type::Actor(aname, atargs) = &rt {
            if let Some(adef) = self.table.actors.get(aname).cloned() {
                if let Some(beh) = adef.behavior(&name.name).cloned() {
                    let row_kinded = actor_row_kinded_generics(&adef);
                    let mut agenv = Genv::default();
                    let mut ti = atargs.iter();
                    for g in &adef.generics {
                        if row_kinded.contains(g) {
                            let v = self.cx.fresh_row_var();
                            agenv.rows.insert(g.clone(), v);
                        } else if let Some(t) = ti.next() {
                            agenv.types.insert(g.clone(), t.clone());
                        }
                    }
                    if beh.params.len() != arg_tys.len() {
                        self.diags.push(
                            Diagnostic::error(
                                "DL0403",
                                format!("behavior `{aname}.{}` expects {} argument(s), found {}", name.name, beh.params.len(), arg_tys.len()),
                            )
                            .with_span(span, "wrong number of arguments"),
                        );
                    }
                    for (p, (at, aspan)) in beh.params.iter().zip(&arg_tys) {
                        let expected = self.lower_type(&p.ty, &agenv, &mut ctx.facts);
                        self.expect_type(&expected, at, *aspan, "send argument type mismatch");
                    }
                    acc.add_effect(Effect::Async);
                    if let Some(r) = &beh.row {
                        let mut row = self.lower_row(r, &agenv);
                        // A row tail this SITE cannot constrain (it appears in no parameter
                        // of this behavior — `Promise.fulfill`'s `e`) contributes nothing
                        // here: its effects are accounted at the sites that bind it (the
                        // spec's own assignment — "row e joins THEN's send row"). Keeping
                        // an unconstrainable fresh tail would DL0501 every caller for
                        // effects no argument of theirs can introduce.
                        if let Some(tail_name) = r.tail.as_ref().map(|t| t.name.clone()) {
                            if !behavior_params_mention_tail(&beh.params, &tail_name) {
                                row.tail = None;
                            }
                        }
                        acc.add_row(&row);
                    }
                    ctx.facts.callees.insert(format!("{aname}.{}", name.name));
                    return (Type::Unit, acc);
                }
                if let Some(sig) = adef.sync_fn(&name.name).cloned() {
                    let mut agenv = Genv::default();
                    for (g, t) in adef.generics.iter().zip(atargs) {
                        agenv.types.insert(g.clone(), t.clone());
                    }
                    if sig.params.len() != arg_tys.len() {
                        self.diags.push(
                            Diagnostic::error(
                                "DL0403",
                                format!("`{aname}.{}` expects {} argument(s), found {}", name.name, sig.params.len(), arg_tys.len()),
                            )
                            .with_span(span, "wrong number of arguments"),
                        );
                    }
                    for (p, (at, aspan)) in sig.params.iter().zip(&arg_tys) {
                        let expected = self.lower_type(&p.ty, &agenv, &mut ctx.facts);
                        self.expect_type(&expected, at, *aspan, "method argument type mismatch");
                    }
                    let ret = match &sig.ret {
                        Some(t) => self.lower_type(t, &agenv, &mut ctx.facts),
                        None => Type::Unit,
                    };
                    if let Some(r) = &sig.row {
                        let row = self.lower_row(r, &agenv);
                        acc.add_row(&row);
                    }
                    ctx.facts.callees.insert(format!("{aname}.{}", name.name));
                    return (ret, acc);
                }
            }
        }

        if let Some((ret, effect, produced_cap)) = self.method_sig(&rt, &name.name, &arg_tys, span, ctx) {
            // The primitive-table arity gate (§7.3): `expect_arg` catches too-few and mistyped
            // arguments per position, but can never see surplus ones — without this,
            // `root.console(1,2,3,4,5)` minted a cap and ignored the noise (fail-open skip branch).
            if let Some(label) = prim_receiver_label(&rt) {
                if let Some(entry) = crate::prim_table::PRIM_TABLE
                    .iter()
                    .find(|e| e.receiver == label && e.method == name.name)
                {
                    if arg_tys.len() > entry.arity as usize {
                        self.diags.push(
                            Diagnostic::error(
                                "DL0403",
                                format!(
                                    "`{label}.{}` expects {} argument(s), found {}",
                                    name.name,
                                    entry.arity,
                                    arg_tys.len()
                                ),
                            )
                            .with_span(span, "wrong number of arguments"),
                        );
                    }
                }
            }
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
            Diagnostic::error("DL0405", format!("type `{}` has no method `{}`", self.ty(&rt), name.name))
                .with_span(name.span, "unknown method"),
        );
        (self.cx.fresh_type(), acc)
    }

    /// The compile-time view of the primitive table (§7.3). Returns (return type, effect, a
    /// capability kind produced). Capability operations are the ONLY source of primitive
    /// effects (T-CapOp) — this table is that single source of truth for the checker.
    /// The arity column of `prim_table::PRIM_TABLE` is enforced by the caller's gate; per-position
    /// types are enforced here via `expect_arg`.
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
                    // Stage 10 (10e, Track D): deriving a device capability is PURE attenuation
                    // like every `root.X()`; the effect is in USING it. The device name selects
                    // among the granted envelopes at runtime (kind static, scope runtime — §5.3).
                    "actuator" => {
                        self.expect_arg(args, 0, &Type::Str, span);
                        (Type::Cap(ResourceKind::Actuator), ResourceKind::Actuator)
                    }
                    "sensor" => {
                        self.expect_arg(args, 0, &Type::Str, span);
                        (Type::Cap(ResourceKind::Sensor), ResourceKind::Sensor)
                    }
                    // Stage 10 (10h, Track F): the same pure attenuation for an accelerator. The
                    // device name selects among the granted compute envelopes at runtime.
                    "compute" => {
                        self.expect_arg(args, 0, &Type::Str, span);
                        (Type::Cap(ResourceKind::Compute), ResourceKind::Compute)
                    }
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
            // Stage 10 (10e, Track D — spec §5.1). `command` takes ANY value (the command
            // record's SHAPE is scope, validated at runtime against the envelope — §5.3's
            // kind/scope split at physical stakes) and carries `Actuate`, the language's most
            // physically consequential effect. The refusal channel is a Result VALUE: the
            // command dies, not the process.
            Type::Cap(ResourceKind::Actuator) => match method {
                "command" => {
                    let _ = self.cx.fresh_type(); // the command value: any shape, scope-checked at runtime
                    let aerr = Type::Sum(self.table.actuate_err(), vec![]);
                    Some((Type::result(Type::Unit, aerr), Some(Effect::Actuate), None))
                }
                _ => None,
            },
            // Sensor reads are `Read` with sensor scopes — deliberately NOT a new effect.
            Type::Cap(ResourceKind::Sensor) => match method {
                "read" => {
                    let aerr = Type::Sum(self.table.actuate_err(), vec![]);
                    Some((Type::result(Type::Float, aerr), Some(Effect::Read), None))
                }
                _ => None,
            },
            // Stage 10 (10h, Track F — spec §7.1, invariant 50). `dispatch(kernel, buffer)` is
            // typed as WHAT IT IS: foreign code. It carries the existing core `ForeignCall`
            // effect and NOT a new one, because inventing a `Dispatch` effect would suggest
            // DeluluLang says something about what the kernel computes. It does not. It bounds
            // the kernel's reachability, its resources, and its provenance — nothing else, and
            // `delulu authority` prints it under the outside-the-proof separator to say so.
            //
            // The kernel is named by a `Str`, never passed as a function: a DeluluLang closure
            // can never become a kernel (the kernels-are-data law, §7.1). The buffer in and the
            // scalar out are the honest shape of "the host drives; devices get buffers".
            Type::Cap(ResourceKind::Compute) => match method {
                "dispatch" => {
                    self.expect_arg(args, 0, &Type::Str, span);
                    self.expect_arg(args, 1, &Type::list(Type::Float), span);
                    let cerr = Type::Sum(self.table.compute_err(), vec![]);
                    Some((Type::result(Type::Float, cerr), Some(Effect::ForeignCall), None))
                }
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
                "map" => match args.first().map(|(t, s)| (self.cx.apply_type(t), *s)) {
                    Some((Type::Fn { row, ret, .. }, aspan)) => {
                        let r = self.cx.apply_row(&row);
                        if !r.is_pure() {
                            self.diags.push(
                                Diagnostic::error("DL0603", "Secret.map requires a pure function")
                                    .with_span(aspan, "this function is not pure"),
                            );
                        }
                        Some((Type::Secret(ret), None, None))
                    }
                    // A concrete non-function argument was the fail-open skip branch:
                    // `s.map(42)` checked clean before Stage 9a caught it.
                    //
                    // `Type::Var` now falls into this same arm (campaign finding C88). It used to
                    // have its own arm, admitted on the reasoning that "the taint discipline (result
                    // stays `Secret`) holds regardless" — which is true and does not help, because
                    // the mapper is handed the PLAINTEXT and the leak happens inside it, before any
                    // result exists to be tainted:
                    //
                    //     fn go[T](k: Secret[Str], f: T) -> Int { let m = k.map(f)  1 }
                    //     go(k, fn(x: Str) -> Str ! {Write} { out.println("LEAK " + x); x })
                    //
                    // checked clean, reported "(none — provably pure)", and printed the live secret
                    // with no `Declassify` effect and no `Cap[Declassify]` anywhere — so R-2 was
                    // reopened as well as R-4. DL0603 is the gate for exactly this and it was
                    // fail-open on the branch where the checker could not tell.
                    Some((other, aspan)) => {
                        self.diags.push(
                            Diagnostic::error(
                                "DL0401",
                                format!("argument type mismatch: Secret.map expects a function, found `{}`", self.ty(&other)),
                            )
                            .with_span(aspan, "type mismatch here"),
                        );
                        Some((Type::Secret(inner.clone()), None, None))
                    }
                    None => {
                        self.diags.push(
                            Diagnostic::error("DL0403", "missing argument 1 (expected a function)")
                                .with_span(span, "too few arguments"),
                        );
                        Some((Type::Secret(inner.clone()), None, None))
                    }
                },
                // verify is a constant-time comparison of two secrets. It is NOT pure, and calling
                // it "no reveal" (as this comment did until P17) was the premise behind campaign
                // finding IF-1: it returns an ORDINARY UNTAINTED `Bool` derived from secret data,
                // which is a declassification — so by R-2 it must carry `Declassify`.
                //
                // The bit is not the "single designed bit" the docs described, because the second
                // operand need not be a secret the caller already holds. `Secret.map` hands its
                // closure the PLAINTEXT and gates only on purity (DL0603) — and purity is not
                // confidentiality — so `k.verify(k.map(fn(x) { g }))` is an equality oracle against
                // an ATTACKER-CHOSEN `g`, and iterating it recovers the whole plaintext:
                //
                //     $ delulu why Declassify extract.delulu
                //       program cannot perform `Declassify`
                //     $ delulu run extract.delulu --grant console --grant secret:API_KEY=hunter2
                //       RECOVERED SECRET = hunter2
                //
                // Emitting the effect does not make the leak impossible — a program may still do
                // this — but it can no longer do it while the toolchain reports that it cannot.
                // That is exactly what R-2 buys: declassification is visible in the row.
                // `trace::effect_for("Secret", "verify")` carries the matching runtime half.
                "verify" => {
                    self.expect_arg(args, 0, &Type::Secret(inner.clone()), span);
                    Some((Type::Bool, Some(Effect::Declassify), None))
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
                "push" => { self.expect_arg(args, 0, elem, span); Some((Type::Unit, None, None)) }
                "map" => {
                    // Row-polymorphic builtin (R-4): the callback's row joins the caller's
                    // (already accounted for via the argument's own check upstream).
                    match args.first().map(|(t, s)| (self.cx.apply_type(t), *s)) {
                        Some((Type::Fn { ret, .. }, _)) => Some((Type::List(ret), None, None)),
                        // Unresolved inference variable: cannot rule at this site (see Secret.map).
                        Some((Type::Var(_), _)) => Some((Type::List(elem.clone()), None, None)),
                        // A concrete non-function was the fail-open skip branch (`xs.map(42)`).
                        Some((other, aspan)) => {
                            self.diags.push(
                                Diagnostic::error(
                                    "DL0401",
                                    format!("argument type mismatch: List.map expects a function, found `{}`", self.ty(&other)),
                                )
                                .with_span(aspan, "type mismatch here"),
                            );
                            Some((Type::List(elem.clone()), None, None))
                        }
                        None => {
                            self.diags.push(
                                Diagnostic::error("DL0403", "missing argument 1 (expected a function)")
                                    .with_span(span, "too few arguments"),
                            );
                            Some((Type::List(elem.clone()), None, None))
                        }
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
                Diagnostic::error("DL0403", format!("missing argument {} (expected `{}`)", i + 1, self.ty(expected)))
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
            // Actor fields type here; WHO may reach them is the rcap pass's rule (only
            // `self` is `ref`; a `tag` receiver's field access is DL1604 there).
            Type::Actor(aname, atargs) => {
                if let Some(adef) = self.table.actors.get(&aname).cloned() {
                    if let Some((_, te, _)) = adef.fields.iter().find(|(n, _, _)| *n == field.name) {
                        let mut genv = Genv::default();
                        for (g, t) in adef.generics.iter().zip(&atargs) {
                            genv.types.insert(g.clone(), t.clone());
                        }
                        return self.lower_type(&te.clone(), &genv, &mut FnFacts::default());
                    }
                }
                self.diags.push(
                    Diagnostic::error("DL0405", format!("no field `{}` on actor `{aname}`", field.name))
                        .with_span(field.span, "unknown field"),
                );
                self.cx.fresh_type()
            }
            Type::Record(id, args) => {
                // Clone ONLY the one field's type expression and the generics, never the whole
                // definition. This is on the per-field-access path, so cloning the definition cost
                // one deep copy of every field per access: a function reading N fields of an
                // N-field record did N² field-entry clones, which at N=2000 was 632 ms of a 632 ms
                // check (`HARDENING_CAMPAIGN.md` C48). The clone was here to release the borrow on
                // `self.table` before `lower_type` takes `&mut self`; a scoped block does that
                // without copying anything the caller does not need.
                let found = {
                    let def = self.table.type_def(id);
                    match &def.kind {
                        TypeDefKind::Record(fields) => fields
                            .iter()
                            .find(|(n, _)| *n == field.name)
                            .map(|(_, ty)| (ty.clone(), def.generics.clone())),
                        _ => None,
                    }
                };
                if let Some((ty, generics)) = found {
                    let mut genv = Genv::default();
                    for (g, a) in generics.iter().zip(&args) {
                        genv.types.insert(g.clone(), a.clone());
                    }
                    return self.lower_type(&ty, &genv, &mut FnFacts::default());
                }
                // Only the failing path pays for the name, and only once.
                let name = self.table.type_def(id).name.clone();
                self.diags.push(
                    Diagnostic::error("DL0405", format!("no field `{}` on `{name}`", field.name))
                        .with_span(field.span, "unknown field"),
                );
                self.cx.fresh_type()
            }
            other => {
                self.diags.push(
                    Diagnostic::error("DL0405", format!("type `{}` has no field `{}`", self.ty(&other), field.name))
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
                    self.diags.push(Diagnostic::error("DL0401", format!("cannot negate `{}`", self.ty(&t))).with_span(operand.span(), "expected Int or Float"));
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
        self.expect_type_coded(&Type::Bool, &ct, cond.span(), "`if` condition must be Bool", Some("DL0408"));
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
                    Diagnostic::error("DL0401", format!("cannot match variant `{vname}` against `{}`", self.ty(&other)))
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
                    Diagnostic::error("DL0409", format!("`?` requires a Result value, found `{}`", self.ty(&other)))
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
            // Stage 7 (build-order deviation 4): the rcap axis lives BESIDE `Type` — lowering
            // ignores the prefix, permanently; the rcap checker reads it from the AST/side
            // tables. This is what keeps rows and rcaps from bleeding into unification.
            // One rcap rule DOES live here because it is about the TYPE: an actor type is
            // always `tag` (spec §2) — any other written rcap is DL1607 with the exact
            // normalize repair (delete the prefix; the default is already tag).
            TypeExpr::Rcap { rcap, inner, span } => {
                let lowered = self.lower_type(inner, genv, facts);
                if matches!(lowered, Type::Actor(_, _)) && *rcap != delulu_syntax::ast::Rcap::Tag {
                    let prefix = Span::new(span.file, span.start, inner.span().start);
                    self.diags.push(
                        Diagnostic::error(
                            "DL1607",
                            format!("an actor type is always `tag` — `{}` cannot apply to it", rcap.name()),
                        )
                        .with_span(*span, "actor references are opaque identity (tag)")
                        .with_repair(Repair {
                            id: "normalize-actor-rcap",
                            confidence: Confidence::Exact,
                            authority_widening: false,
                            requires_human: false,
                            edits: vec![Edit {
                                file: prefix.file,
                                start_byte: prefix.start,
                                end_byte: prefix.end,
                                insert: String::new(),
                            }],
                            reason: None,
                        }),
                    );
                }
                lowered
            }
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
                    // An actor name is an actor-reference type (Stage 7, spec §2) — always
                    // `tag` (DL1607 for any other written rcap; enforced in the Rcap arm).
                    if let Some(adef) = self.table.actors.get(name) {
                        // Row-kinded actor generics (`Promise[T, e]`'s `e`) are invisible
                        // in TYPE position — the written arity counts type-kinded only.
                        let row_kinded = actor_row_kinded_generics(adef);
                        let type_arity = adef.generics.iter().filter(|g| !row_kinded.contains(*g)).count();
                        if args.len() != type_arity {
                            self.diags.push(
                                Diagnostic::error("DL0406", format!("actor `{name}` expects {type_arity} type argument(s), found {}", args.len()))
                                    .with_span(*span, "wrong number of type arguments"),
                            );
                        }
                        let targs: Vec<Type> = args.iter().map(|a| self.lower_type(a, genv, facts)).collect();
                        return Type::Actor(name.clone(), targs);
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
                            // A cyclic alias is never expanded. Expanding one is unbounded recursion
                            // and used to abort the compiler with a stack overflow (C54); the cycle
                            // has already been reported at its declaration by `check_type_aliases`,
                            // so this yields a fresh variable and lets checking continue rather than
                            // reporting the same cycle once per use site.
                            TypeDefKind::Alias(_) if self.cyclic_aliases.contains(name) => {
                                self.cx.fresh_type()
                            }
                            TypeDefKind::Alias(inner) => {
                                let mut agenv = Genv::default();
                                for (g, a) in def.generics.iter().zip(&targs) {
                                    agenv.types.insert(g.clone(), a.clone());
                                }
                                self.lower_type(&inner, &agenv, facts)
                            }
                        };
                    }
                    // `with_arg("type", ..)` is not decoration: `check_workspace` reads it to tell
                    // this apart from an ordinary unknown name. An imported signature is lowered in
                    // the IMPORTER's scope, so this fires with a span inside the dependency's own
                    // source — see `reframe_foreign_scope_errors` (C58).
                    self.diags.push(
                        Diagnostic::error("DL0301", format!("unknown type `{name}`"))
                            .with_arg("type", name.clone())
                            .with_span(*span, "not a type"),
                    );
                    return self.cx.fresh_type();
                }
                self.diags.push(
                    Diagnostic::error("DL0301", format!("unknown type `{}`", path.dotted()))
                        .with_arg("type", path.dotted())
                        .with_span(*span, "not a type"),
                );
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
            // Stage 7: an actor reference is opaque identity (tag) — no str, no ==, never
            // serialized. Identity comparison is a post-1.0 question, not an accident.
            Type::Actor(_, _) => true,
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
            // Opacity is a property of the core type; the rcap prefix does not change it.
            TypeExpr::Rcap { inner, .. } => self.type_expr_opaque(inner, genv, visiting),
        }
    }

    /// Render a type for a diagnostic, naming its records and sums.
    ///
    /// Every message that shows a type goes through here. A `Record`/`Sum` is stored as a table
    /// index, so without the table the printer can only say `T11` — which is what C12 was.
    fn ty(&self, t: &Type) -> String {
        t.show(self.table).to_string()
    }

    fn expect_type(&mut self, expected: &Type, actual: &Type, span: Span, msg: &str) {
        self.expect_type_coded(expected, actual, span, msg, None)
    }

    /// `expect_type` with a caller-supplied code for the plain-mismatch case. The specific codes
    /// the registry allocates (DL0408 for a non-Bool condition) must actually be reachable, or
    /// they are frozen at 1.0 as codes nothing can ever produce.
    fn expect_type_coded(
        &mut self,
        expected: &Type,
        actual: &Type,
        span: Span,
        msg: &str,
        mismatch_code: Option<&'static str>,
    ) {
        match self.cx.unify_type(expected, actual) {
            Ok(()) => {}
            Err(e) => {
                let ea = self.cx.apply_type(expected);
                let aa = self.cx.apply_type(actual);
                // A secret used where its plain type is expected is the "secret cannot flow"
                // case (§6.4 rule 1) — reported as DL0602, not a generic type mismatch.
                if secret_mismatch(&ea, &aa) {
                    let inner =
                        if matches!(ea, Type::Secret(_)) { self.ty(&aa) } else { self.ty(&ea) };
                    self.diags.push(
                        Diagnostic::error(
                            "DL0602",
                            format!("secret value cannot flow here: `Secret[..]` is not `{inner}` (secrets never coerce)"),
                        )
                        .with_arg("inner", inner)
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
                // A capability where a plain value stands (or the reverse) is the forgery case
                // (§5.1): capabilities are derived from `Root`, never constructed, cast, or
                // coerced into being. Reported as DL0601 so the authority failure reads as what
                // it is rather than as an ordinary type mismatch.
                if capability_mismatch(&ea, &aa) {
                    self.diags.push(
                        Diagnostic::error(
                            "DL0601",
                            format!(
                                "a capability cannot be constructed or forged: `{}` is not `{}` \
                                 (capabilities are derived from `Root`)",
                                self.ty(&ea),
                                self.ty(&aa)
                            ),
                        )
                        .with_span(span, "a capability value has no constructor"),
                    );
                    return;
                }
                let code = match e {
                    UnifyError::RowConflict => "DL0504",
                    _ => mismatch_code.unwrap_or("DL0401"),
                };
                self.diags.push(
                    Diagnostic::error(code, format!("{msg}: expected `{}`, found `{}`", self.ty(&ea), self.ty(&aa)))
                        .with_span(span, "type mismatch here"),
                );
            }
        }
    }
}

/// The marshallable allowlist for a foreign signature (T-ForeignSig): exactly
/// `Int Float Bool Str Unit ForeignPtr`, with no type arguments. Everything else — `Secret[T]`,
/// `Cap[R]`, `Root`, `Plugin[_]`, `PyObj`, a lib handle, `List[..]`, user types — is DL1301.
/// `M(τ)`'s allowlist, by name — the ONE list. Both engines lower foreign signatures by type name
/// (`delulu_runtime::interp::lower_foreign_sig`, shared with the WASM host), so this list and
/// `FKind::from_type_name` are two halves of one rule and must never drift apart.
pub const MARSHALLABLE_TYPE_NAMES: &[&str] =
    &["Int", "Float", "Bool", "Str", "Unit", "ForeignPtr"];

fn is_marshallable_type_expr(t: &TypeExpr) -> bool {
    match t {
        TypeExpr::Named { path, args, .. } => {
            args.is_empty()
                && path.segs.len() == 1
                && MARSHALLABLE_TYPE_NAMES.contains(&path.segs[0].name.as_str())
        }
        TypeExpr::Fn { .. } => false,
        // An rcap prefix never appears in a foreign signature (the FFI predates rcaps and
        // marshals by copy); refuse rather than silently strip it.
        TypeExpr::Rcap { .. } => false,
    }
}

/// The span of the first function type appearing anywhere in `t` (including nested inside type
/// arguments, e.g. `List[fn(Int) -> Int]`) — the trigger for DL1302 (invariant 22).
fn first_fn_type(t: &TypeExpr) -> Option<Span> {
    match t {
        TypeExpr::Fn { span, .. } => Some(*span),
        TypeExpr::Named { args, .. } => args.iter().find_map(first_fn_type),
        TypeExpr::Rcap { inner, .. } => first_fn_type(inner),
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

/// Whether a substituted [`Type`] still contains an inference variable at any depth — including
/// inside a function type's parameters and return, which [`type_contains_fn`] deliberately does not
/// recurse into.
///
/// This is the **fail-closed** test behind DL1509 (build-order deviation 5). At a Contained `get`
/// site an unresolved variable is not "unknown but probably fine": a generic's variables are
/// instantiated *fresh per call site* (see `instantiate_fn`), so the variable standing in a generic
/// helper's body is never unified with what the caller actually passes. A closure can therefore
/// reach an opaque module while the body's `F` still reads as `fn('t0) -> Str` — exactly the
/// re-entry point R-6a exists to forbid. Refusing here keeps R-6a decidable at the one site the
/// spec requires it (the `get` call site, compile-time).
fn type_contains_var(t: &Type) -> bool {
    match t {
        Type::Var(_) => true,
        Type::List(e) | Type::Option(e) | Type::Secret(e) | Type::Plugin(e) => type_contains_var(e),
        Type::Result(o, e) => type_contains_var(o) || type_contains_var(e),
        Type::Record(_, args) | Type::Sum(_, args) => args.iter().any(type_contains_var),
        Type::Fn { params, ret, .. } => {
            params.iter().any(type_contains_var) || type_contains_var(ret)
        }
        _ => false,
    }
}

/// Actor generics that appear as a ROW TAIL anywhere in a member signature are row-kinded
/// (the `actor Promise[T, e] { be then(f: … ! e) ! e }` pattern); the rest are type-kinded.
fn actor_row_kinded_generics(adef: &crate::resolve::ActorDef) -> HashSet<String> {
    fn scan_type(te: &TypeExpr, out: &mut HashSet<String>) {
        match te {
            TypeExpr::Fn { params, ret, row, .. } => {
                params.iter().for_each(|p| scan_type(p, out));
                if let Some(r) = ret {
                    scan_type(r, out);
                }
                if let Some(r) = row {
                    if let Some(t) = &r.tail {
                        out.insert(t.name.clone());
                    }
                }
            }
            TypeExpr::Named { args, .. } => args.iter().for_each(|a| scan_type(a, out)),
            TypeExpr::Rcap { inner, .. } => scan_type(inner, out),
        }
    }
    fn scan_member(params: &[Param], row: &Option<RowExpr>, tails: &mut HashSet<String>) {
        for p in params {
            scan_type(&p.ty, tails);
        }
        if let Some(r) = row {
            if let Some(t) = &r.tail {
                tails.insert(t.name.clone());
            }
        }
    }
    let mut tails = HashSet::new();
    scan_member(&adef.ctor_params, &adef.ctor_row, &mut tails);
    for b in &adef.behaviors {
        scan_member(&b.params, &b.row, &mut tails);
    }
    for f in &adef.fns {
        scan_member(&f.params, &f.row, &mut tails);
        if let Some(r) = &f.ret {
            scan_type(r, &mut tails);
        }
    }
    adef.generics.iter().filter(|g| tails.contains(*g)).cloned().collect()
}

/// Does any parameter's written type mention `tail` as a row tail (at any fn-type depth)?
/// The T-Send tail rule: a site can only be charged for a row variable one of its own
/// arguments can bind.
fn behavior_params_mention_tail(params: &[Param], tail: &str) -> bool {
    fn scan(te: &TypeExpr, tail: &str) -> bool {
        match te {
            TypeExpr::Fn { params, ret, row, .. } => {
                row.as_ref().and_then(|r| r.tail.as_ref()).is_some_and(|t| t.name == tail)
                    || params.iter().any(|p| scan(p, tail))
                    || ret.as_ref().is_some_and(|r| scan(r, tail))
            }
            TypeExpr::Named { args, .. } => args.iter().any(|a| scan(a, tail)),
            TypeExpr::Rcap { inner, .. } => scan(inner, tail),
        }
    }
    params.iter().any(|p| scan(&p.ty, tail))
}

/// Field names assigned as `self.f = …` anywhere in the constructor body (flow-insensitive,
/// v0.7) — the definite-initialization scan.
fn collect_self_field_assigns(b: &Block, out: &mut HashSet<String>) {
    fn scan_expr(e: &Expr, out: &mut HashSet<String>) {
        match e {
            Expr::If { then_, else_, .. } => {
                collect_self_field_assigns(then_, out);
                if let Some(e2) = else_ {
                    scan_expr(e2, out);
                }
            }
            Expr::Match { arms, .. } => arms.iter().for_each(|a| scan_expr(&a.body, out)),
            Expr::Block(inner) => collect_self_field_assigns(inner, out),
            _ => {}
        }
    }
    for stmt in &b.stmts {
        match stmt {
            Stmt::Assign { target: LValue::Field(base, fname), .. } => {
                if let LValue::Var(v) = &**base {
                    if v.name == "self" {
                        out.insert(fname.name.clone());
                    }
                }
            }
            Stmt::While { body, .. } => collect_self_field_assigns(body, out),
            // A `for` body assigns `self.field` exactly as a `while` body can. Without this arm the
            // catch-all below swallowed it, so an actor constructor that assigned a field ONLY
            // inside a `for` loop was falsely reported as never assigning it (DL0405). `break`/
            // `continue` assign nothing and correctly fall through.
            Stmt::For { body, .. } => collect_self_field_assigns(body, out),
            Stmt::Expr(e) => scan_expr(e, out),
            _ => {}
        }
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
        TypeExpr::Rcap { rcap, inner, .. } => format!("{} {}", rcap.name(), render_type_expr(inner)),
    }
}

/// Exactly one side is a function type and the other is `PyObj` — a closure being smuggled into (or
/// out of) `PyObj`-land, the no-callbacks rule (spec §4.4 / R-6a), reported as DL1302 not DL0401.
fn fn_pyobj_mismatch(a: &Type, b: &Type) -> bool {
    let is_fn = |t: &Type| matches!(t, Type::Fn { .. });
    (is_fn(a) && matches!(b, Type::PyObj)) || (matches!(a, Type::PyObj) && is_fn(b))
}

/// Exactly one side is a capability (`Cap[_]` or `Root`) and the other is a concrete non-capability
/// type — i.e. a program is trying to produce a capability from something that is not one, or to
/// use one where a plain value belongs. Variables are excluded: an unresolved side is not yet a
/// forgery attempt, it is simply unknown.
fn capability_mismatch(a: &Type, b: &Type) -> bool {
    let is_cap = |t: &Type| matches!(t, Type::Cap(_) | Type::Root);
    let concrete = |t: &Type| !matches!(t, Type::Var(_));
    (is_cap(a) ^ is_cap(b)) && concrete(a) && concrete(b)
}

/// The `prim_table` receiver label for a checked receiver type (the §7.3 arity gate). `None` for
/// receivers whose methods are not primitive-table entries: foreign lib handles carry their own
/// DL0403 fence in `method_sig`, and non-primitive surfaces have their own typing rules.
fn prim_receiver_label(recv: &Type) -> Option<&'static str> {
    Some(match recv {
        Type::Root => "root",
        Type::Cap(ResourceKind::Console) => "console",
        Type::Cap(ResourceKind::FsRead) => "fs_read",
        Type::Cap(ResourceKind::FsWrite) => "fs_write",
        Type::Cap(ResourceKind::Http) => "http",
        Type::Cap(ResourceKind::Clock) => "clock",
        Type::Cap(ResourceKind::Rand) => "rand",
        Type::Cap(ResourceKind::Python) => "python",
        Type::PyObj => "pyobj",
        Type::Secret(_) => "secret",
        Type::Str => "str",
        Type::List(_) => "list",
        Type::Plugin(_) => "plugin",
        _ => return None,
    })
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

/// Every type name mentioned anywhere in a type expression, head or argument, in source order.
/// Used by the alias-cycle check, which must see `A` inside `List[A]` as well as at the head.
fn collect_named_types(t: &TypeExpr, out: &mut Vec<String>) {
    match t {
        TypeExpr::Named { path, args, .. } => {
            if path.segs.len() == 1 {
                out.push(path.segs[0].name.clone());
            }
            for a in args {
                collect_named_types(a, out);
            }
        }
        TypeExpr::Fn { params, ret, .. } => {
            for p in params {
                collect_named_types(p, out);
            }
            if let Some(r) = ret {
                collect_named_types(r, out);
            }
        }
        // A reference-capability wrapper (`iso T`, `val T`, …) still mentions `T`, and an alias can
        // cycle through one.
        TypeExpr::Rcap { inner, .. } => collect_named_types(inner, out),
    }
}

/// Mark every alias reachable from `name` that lies on a cycle. `on_stack` is the current DFS path,
/// so an edge back into it closes a cycle; `seen` keeps the walk linear in the graph's size.
fn find_alias_cycle(
    name: &str,
    edges: &HashMap<String, Vec<String>>,
    on_stack: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
    cyclic: &mut std::collections::HashSet<String>,
) {
    if let Some(at) = on_stack.iter().position(|n| n == name) {
        // Everything from the first occurrence onward is on the cycle itself.
        for n in &on_stack[at..] {
            cyclic.insert(n.clone());
        }
        cyclic.insert(name.to_string());
        return;
    }
    if !seen.insert(name.to_string()) {
        return;
    }
    on_stack.push(name.to_string());
    if let Some(next) = edges.get(name) {
        for n in next {
            find_alias_cycle(n, edges, on_stack, seen, cyclic);
        }
    }
    on_stack.pop();
}
