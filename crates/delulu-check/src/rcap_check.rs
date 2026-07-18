//! The reference-capability checking PASS (Stage 7, phases 7c–7f) — the second checking axis.
//!
//! Runs as a separate walk AFTER type inference (build-order deviation 4): it reads the type
//! checker's resolved per-node types (`node_types`) and signature types (`fn_types`) and never
//! participates in unification — rows and rcaps cannot bleed into each other by construction
//! (playbook trap 1).
//!
//! Fail-closed discipline (the kitchen rule): the pass distinguishes "I know this rcap" from
//! "I could not determine it". Rules that GUARD something (writes, mutating methods, later:
//! sends) refuse on Unknown; pure reads through Unknown flow on, because every dangerous use
//! of their result is itself guarded downstream.
//!
//! v0.7 precision notes (recorded, not hidden):
//! - Pattern-bound names have no tracked rcap (patterns carry no node types); writing through
//!   one is refused as undeterminable. The corpus performs no such writes.
//! - Generic field types resolve to Unknown (no substitution here); a write's VALUE check is
//!   skipped when the destination rcap is undeterminable — the receiver-writability check
//!   (the race-relevant half) always runs.

use std::collections::{HashMap, HashSet};

use delulu_diag::{Confidence, Diagnostic, Edit, Repair, Span};
use delulu_syntax::ast::*;

use crate::rcaps::{alias, can_write, default_rcap, sendable, subcap, viewpoint};
use crate::resolve::{DeclTable, TypeDefKind};
use crate::ty::{Type, TypeDefId};

/// What the pass knows about an expression's reference capability.
#[derive(Clone, Copy, Debug, PartialEq)]
enum K {
    /// An ordinary (possibly aliased) value seen at this rcap; storing it goes through
    /// `alias(κ)` (spec §3).
    Known(Rcap),
    /// An unaliased value carrying its FULL rcap: a `consume`d binding, a `recover` result,
    /// or a call returning a written unique rcap. Storing it uses κ directly.
    Unaliased(Rcap),
    /// A freshly created composite literal — provably without aliases. `natural` is its rcap
    /// when bound without demand (`ref`); `lift_val`/`lift_iso` say whether its CONTENTS
    /// permit lifting the whole value to deep immutability / uniqueness.
    Fresh { natural: Rcap, lift_val: bool, lift_iso: bool },
    Unknown,
}

struct Binding {
    rcap: Option<Rcap>,
    ty: Option<Type>,
    /// `Some` while the binding still holds a fresh, never-escaped literal: the (lift_val,
    /// lift_iso) of that literal. Cleared the moment the binding escapes (aliased into a
    /// call, another binding, a store, or a closure capture).
    fresh_lift: Option<(bool, bool)>,
    /// Definite-unassignment (phase 7d, spec §3): `Some((site, loop_carried))` once the
    /// binding has been `consume`d on SOME path reaching here — a possibly-consumed binding
    /// is a dead binding (the unique reference may already have been transferred). The bool
    /// marks a consume carried in from a previous loop iteration, for a clearer message.
    consumed: Option<(Span, bool)>,
}

pub fn check_rcaps(
    module: &Module,
    table: &DeclTable,
    node_types: &HashMap<NodeId, Type>,
    fn_types: &HashMap<String, Type>,
) -> (Vec<Diagnostic>, HashSet<NodeId>) {
    let mut pass = Pass {
        table,
        node_types,
        fn_types,
        diags: Vec::new(),
        scopes: Vec::new(),
        recover_boundary: None,
        capture_boundary: None,
        iso_moves: HashSet::new(),
    };
    for item in &module.items {
        match item {
            Item::Fn(f) => pass.check_fn(f),
            Item::Actor(a) => pass.check_actor(a),
            _ => {}
        }
    }
    (pass.diags, pass.iso_moves)
}

struct Pass<'a> {
    table: &'a DeclTable,
    node_types: &'a HashMap<NodeId, Type>,
    fn_types: &'a HashMap<String, Type>,
    diags: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, Binding>>,
    /// Scope depth at the innermost `recover` entry: references to bindings BELOW this index
    /// cross the recover boundary and must be `val`/`tag` (or consumed `iso`) — DL1605.
    recover_boundary: Option<usize>,
    /// Scope depth at the innermost lambda entry: a `consume` of a binding below this index
    /// would consume a CAPTURE — refused (the closure may run any number of times).
    capture_boundary: Option<usize>,
    /// Send/spawn argument node ids that are statically-proven iso MOVES (phase 7i): the
    /// runtime's `--debug-rcaps` verifies each one's graph is unaliased at the boundary.
    iso_moves: HashSet<NodeId>,
}

/// How a binding is being referenced, for the centralized use path.
#[derive(Clone, Copy, PartialEq)]
enum UseKind {
    Read,
    Consume,
}

impl<'a> Pass<'a> {
    // ----- environment ------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn bind(&mut self, name: &str, b: Binding) {
        if let Some(s) = self.scopes.last_mut() {
            s.insert(name.to_string(), b);
        }
    }
    fn lookup(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }
    fn lookup_mut(&mut self, name: &str) -> Option<&mut Binding> {
        self.scopes.iter_mut().rev().find_map(|s| s.get_mut(name))
    }
    /// The binding stops being a provably-unaliased fresh value.
    fn mark_escaped(&mut self, name: &str) {
        if let Some(b) = self.lookup_mut(name) {
            b.fresh_lift = None;
        }
    }

    /// THE centralized reference path (7d): every read, consume, and lvalue-base lookup goes
    /// through here, so the consume flow analysis and the recover/lambda boundary rules
    /// cannot be dodged by spelling. Returns the binding's rcap (even on error, for recovery).
    fn use_binding(&mut self, name: &str, span: Span, kind: UseKind) -> Option<Rcap> {
        // Find the binding and the scope depth it lives at.
        struct Found {
            depth: usize,
            rcap: Option<Rcap>,
            consumed: Option<(Span, bool)>,
        }
        let mut found: Option<Found> = None;
        for (depth, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(b) = scope.get(name) {
                found = Some(Found { depth, rcap: b.rcap, consumed: b.consumed });
                break;
            }
        }
        let Found { depth, rcap, consumed } = found?;

        // Use after consume (DL1602) — including possibly-consumed (branch joins) and
        // loop-carried consumes.
        if let Some((site, loop_carried)) = consumed {
            let mut d = Diagnostic::error(
                "DL1602",
                format!("use of `{name}` after `consume` — the binding is dead"),
            )
            .with_span(span, "used here");
            d = if loop_carried {
                d.with_secondary_span(site, "consumed here — by the next loop iteration this binding is already dead")
            } else {
                d.with_secondary_span(site, "consumed here")
            };
            self.diags.push(d);
            return rcap;
        }

        // The recover boundary (DL1605, spec §3): only val/tag outer bindings are visible;
        // a consumed iso may be transferred in.
        if let Some(rb) = self.recover_boundary {
            if depth < rb {
                let allowed = matches!(
                    (kind, rcap),
                    (_, Some(Rcap::Val) | Some(Rcap::Tag)) | (UseKind::Consume, Some(Rcap::Iso))
                );
                if !allowed {
                    let shown = rcap.map(|r| format!("`{}`", r.name())).unwrap_or_else(|| "an undetermined capability".into());
                    self.diags.push(
                        Diagnostic::error(
                            "DL1605",
                            format!(
                                "recover block references non-sendable outer binding `{name}` ({shown}) — only `val`, `tag`, or a consumed `iso` may cross into recover"
                            ),
                        )
                        .with_span(span, "crosses the recover boundary"),
                    );
                    return rcap;
                }
            }
        }

        match kind {
            UseKind::Read => {}
            UseKind::Consume => {
                // Consuming a CAPTURE is refused: the closure may run any number of times,
                // and each run would kill the same outer binding again (the closure skip
                // branch — kitchen rule).
                if let Some(cb) = self.capture_boundary {
                    if depth < cb {
                        self.diags.push(
                            Diagnostic::error(
                                "DL1602",
                                format!("cannot `consume` captured binding `{name}` — a closure may run any number of times"),
                            )
                            .with_span(span, "consume of a capture"),
                        );
                        return rcap;
                    }
                }
                if let Some(b) = self.scopes[depth].get_mut(name) {
                    b.consumed = Some((span, false));
                    b.fresh_lift = None;
                }
            }
        }
        rcap
    }

    /// Snapshot every visible binding's consumed state, name-keyed per scope depth
    /// (for branch joins).
    fn consumed_snapshot(&self) -> Vec<HashMap<String, Option<(Span, bool)>>> {
        self.scopes
            .iter()
            .map(|s| s.iter().map(|(n, b)| (n.clone(), b.consumed)).collect())
            .collect()
    }

    fn consumed_restore(&mut self, snap: &[HashMap<String, Option<(Span, bool)>>]) {
        for (scope, states) in self.scopes.iter_mut().zip(snap) {
            for (n, b) in scope.iter_mut() {
                if let Some(st) = states.get(n) {
                    b.consumed = *st;
                }
            }
        }
    }

    /// Join: consumed on ANY branch ⇒ consumed after (possibly-consumed is dead).
    fn consumed_join(&mut self, other: &[HashMap<String, Option<(Span, bool)>>]) {
        for (scope, states) in self.scopes.iter_mut().zip(other) {
            for (n, b) in scope.iter_mut() {
                if b.consumed.is_none() {
                    if let Some(st) = states.get(n) {
                        b.consumed = *st;
                    }
                }
            }
        }
    }

    // ----- entry ------------------------------------------------------------

    fn check_fn(&mut self, f: &FnDecl) {
        self.scopes.clear();
        self.push_scope();
        let sig_params: Vec<Type> = match self.fn_types.get(&f.name.name) {
            Some(Type::Fn { params, .. }) => params.clone(),
            _ => Vec::new(),
        };
        for (i, p) in f.params.iter().enumerate() {
            let ty = sig_params.get(i).cloned();
            let rcap = p
                .ty
                .written_rcap()
                .or_else(|| ty.as_ref().and_then(|t| self.default_of(t)));
            self.bind(&p.name.name, Binding { rcap, ty, fresh_lift: None, consumed: None });
        }
        let ret_dest = f
            .ret
            .as_ref()
            .and_then(|r| r.written_rcap())
            .or_else(|| match self.fn_types.get(&f.name.name) {
                Some(Type::Fn { ret, .. }) => self.default_of(ret),
                _ => None,
            });
        self.walk_block_with_tail(&f.body, ret_dest, f.ret.is_some());
        self.pop_scope();
    }

    fn default_of(&self, t: &Type) -> Option<Rcap> {
        let comps = |id: TypeDefId| -> Option<Vec<Type>> { components_of(self.table, id) };
        default_rcap(t, &comps)
    }

    // ----- actors (7e): T-Behavior/T-Ctor sendability + member bodies -------

    fn check_actor(&mut self, a: &ActorDecl) {
        // Every behavior/ctor parameter must be SENDABLE — iso, val, or tag — else DL1601
        // (T-Behavior/T-Ctor: crossing an actor boundary is the whole game). Undecidable
        // sendability refuses too: a guarantee that cannot be established is not granted.
        self.check_boundary_params(&a.name.name, "new", &a.ctor.params);
        for b in &a.behaviors {
            self.check_boundary_params(&a.name.name, &b.name.name, &b.params);
        }
        // Member bodies, with `self : ref ActorType` bound.
        let self_ty = Type::Actor(a.name.name.clone(), Vec::new());
        self.check_actor_member(&a.name.name, "new", &a.ctor.params, &a.ctor.body, false, &self_ty);
        for b in &a.behaviors {
            self.check_actor_member(&a.name.name, &b.name.name, &b.params, &b.body, false, &self_ty);
        }
        for f in &a.fns {
            self.check_actor_member(&a.name.name, &f.name.name, &f.params, &f.body, f.ret.is_some(), &self_ty);
        }
    }

    fn check_boundary_params(&mut self, actor: &str, member: &str, params: &[Param]) {
        let key = format!("{actor}.{member}");
        let sig_params: Vec<Type> = match self.fn_types.get(&key) {
            Some(Type::Fn { params, .. }) => params.clone(),
            _ => Vec::new(),
        };
        for (i, p) in params.iter().enumerate() {
            let ty = sig_params.get(i);
            let rcap = p.ty.written_rcap().or_else(|| ty.and_then(|t| self.default_of(t)));
            match rcap {
                Some(k) if sendable(k) => {}
                Some(k) => {
                    // Invariant 36 / acceptance criterion 11: PyObj is actor-PINNED — CPython
                    // has thread affinity, so a Python object lives and dies on the actor
                    // that created it. The refusal explains the pinning, not just the rcap.
                    let msg = if matches!(ty, Some(Type::PyObj)) {
                        format!(
                            "`{key}` parameter `{}` is a PyObj, which is pinned to its creating actor (CPython affinity) and can never cross an actor boundary",
                            p.name.name
                        )
                    } else {
                        format!(
                            "`{key}` parameter `{}` is `{}`, which is not sendable — a value crossing an actor boundary must be `iso` (consumed), `val` (deeply immutable), or `tag` (opaque identity)",
                            p.name.name,
                            k.name()
                        )
                    };
                    self.diags.push(
                        Diagnostic::error("DL1601", msg).with_span(p.ty.span(), "not sendable"),
                    );
                }
                None => {
                    self.diags.push(
                        Diagnostic::error(
                            "DL1601",
                            format!(
                                "`{key}` parameter `{}`'s sendability could not be determined — annotate it `iso`, `val`, or `tag` (a boundary guarantee is never guessed)",
                                p.name.name
                            ),
                        )
                        .with_span(p.ty.span(), "undecidable sendability"),
                    );
                }
            }
        }
    }

    fn check_actor_member(
        &mut self,
        actor: &str,
        member: &str,
        params: &[Param],
        body: &Block,
        has_ret: bool,
        self_ty: &Type,
    ) {
        self.scopes.clear();
        self.push_scope();
        // T-Behavior: `self : ref ActorType` — the ONLY ref to an actor anywhere; every
        // external reference is tag (spec §2 default + DL1607).
        self.bind(
            "self",
            Binding { rcap: Some(Rcap::Ref), ty: Some(self_ty.clone()), fresh_lift: None, consumed: None },
        );
        let key = format!("{actor}.{member}");
        let sig_params: Vec<Type> = match self.fn_types.get(&key) {
            Some(Type::Fn { params, .. }) => params.clone(),
            _ => Vec::new(),
        };
        for (i, p) in params.iter().enumerate() {
            let ty = sig_params.get(i).cloned();
            let rcap = p.ty.written_rcap().or_else(|| ty.as_ref().and_then(|t| self.default_of(t)));
            self.bind(&p.name.name, Binding { rcap, ty, fresh_lift: None, consumed: None });
        }
        let ret_dest = has_ret
            .then(|| match self.fn_types.get(&key) {
                Some(Type::Fn { ret, .. }) => self.default_of(ret),
                _ => None,
            })
            .flatten();
        self.walk_block_with_tail(body, ret_dest, has_ret);
        self.pop_scope();
    }

    // ----- statements -------------------------------------------------------

    /// Walk a block; `ret_dest`/`has_ret` describe the enclosing function's return position
    /// so the tail expression (and every `return`) gets the storability check + fresh lift.
    fn walk_block_with_tail(&mut self, b: &Block, ret_dest: Option<Rcap>, has_ret: bool) {
        self.push_scope();
        let n = b.stmts.len();
        for (i, stmt) in b.stmts.iter().enumerate() {
            let is_tail = has_ret && i + 1 == n;
            match stmt {
                Stmt::Expr(e) if is_tail => {
                    let k = self.walk_expr(e);
                    self.check_return_position(e, k, ret_dest);
                }
                Stmt::Return { value: Some(v), .. } => {
                    let k = self.walk_expr(v);
                    self.check_return_position(v, k, ret_dest);
                }
                other => self.walk_stmt(other, ret_dest, has_ret),
            }
        }
        self.pop_scope();
    }

    fn walk_stmt(&mut self, stmt: &Stmt, ret_dest: Option<Rcap>, has_ret: bool) {
        match stmt {
            Stmt::Let { name, ty, value, span, .. } => {
                let vk = self.walk_expr(value);
                let vty = self.node_types.get(&value.id()).cloned();
                let written = ty.as_ref().and_then(|t| t.written_rcap());
                let annotated_default =
                    ty.as_ref().and_then(|_| vty.as_ref().and_then(|t| self.default_of(t)));
                let dest = written.or(annotated_default);
                if let Some(d) = dest {
                    self.check_storable(vk, d, *span, "binding", written.is_some());
                }
                // The binding's rcap: the annotation's demand, else what the value provides.
                let rcap = dest.or(match vk {
                    K::Known(k) => Some(alias(k)),
                    K::Unaliased(k) => Some(k),
                    K::Fresh { natural, .. } => Some(natural),
                    K::Unknown => None,
                });
                let fresh_lift = match vk {
                    K::Fresh { lift_val, lift_iso, .. } => Some((lift_val, lift_iso)),
                    _ => None,
                };
                // If the value was another binding, that binding is now aliased.
                if let Expr::Var { path, .. } = value {
                    if path.segs.len() == 1 {
                        self.mark_escaped(&path.segs[0].name);
                    }
                }
                self.bind(&name.name, Binding { rcap, ty: vty, fresh_lift, consumed: None });
            }
            Stmt::Assign { target, value, span } => {
                let vk = self.walk_expr(value);
                if let Expr::Var { path, .. } = value {
                    if path.segs.len() == 1 {
                        self.mark_escaped(&path.segs[0].name);
                    }
                }
                self.check_write(target, vk, *span);
            }
            Stmt::While { cond, body, .. } => {
                self.walk_expr(cond);
                // Loop-carried consume: a consume anywhere in the body kills the binding for
                // every LATER iteration, so an outer binding consumed in the body is dead at
                // body entry — every in-body use (including the consume itself, which would
                // re-consume a dead binding) and every post-loop use gets DL1602.
                let mut carried = Vec::new();
                consumed_free_names(body, &mut HashSet::new(), &mut carried);
                for (name, site) in &carried {
                    if let Some(b) = self.lookup_mut(name) {
                        if b.consumed.is_none() {
                            b.consumed = Some((*site, true));
                        }
                    }
                }
                self.walk_block_with_tail(body, None, false);
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    let k = self.walk_expr(v);
                    self.check_return_position(v, k, ret_dest);
                }
                let _ = has_ret;
            }
            Stmt::Expr(e) => {
                self.walk_expr(e);
            }
        }
    }

    /// Return/tail position: a fresh-born, never-escaped local lifts freely (control leaves
    /// the frame, so the binding provably dies with no other alias); everything else is the
    /// ordinary storability check against the return rcap.
    fn check_return_position(&mut self, e: &Expr, k: K, ret_dest: Option<Rcap>) {
        let Some(dest) = ret_dest else { return };
        if let Expr::Var { path, .. } = e {
            if path.segs.len() == 1 {
                if let Some(b) = self.lookup(&path.segs[0].name) {
                    if let Some((lift_val, lift_iso)) = b.fresh_lift {
                        let ok = match dest {
                            Rcap::Val => lift_val,
                            Rcap::Iso | Rcap::Trn => lift_iso,
                            _ => true,
                        };
                        if !ok {
                            self.diags.push(
                                Diagnostic::error(
                                    "DL1603",
                                    format!(
                                        "this value's contents do not permit lifting it to `{}` at return",
                                        dest.name()
                                    ),
                                )
                                .with_span(e.span(), "returned here"),
                            );
                        }
                        return;
                    }
                }
            }
        }
        self.check_storable(k, dest, e.span(), "return value", false);
    }

    /// The storability rule (spec §3): an unconsumed value stores through `alias(κ)`; a
    /// consumed/recovered value through its full κ; a fresh literal lifts per its contents.
    fn check_storable(&mut self, k: K, dest: Rcap, span: Span, what: &str, dest_written: bool) {
        let ok = match k {
            K::Known(kk) => subcap(alias(kk), dest),
            K::Unaliased(kk) => subcap(kk, dest),
            K::Fresh { natural, lift_val, lift_iso } => match dest {
                Rcap::Val => lift_val,
                Rcap::Iso | Rcap::Trn => lift_iso,
                d => subcap(natural, d),
            },
            // A guard consulted on an unknown refuses only when the destination made an
            // explicit demand; a defaulted destination over an undeterminable value would
            // refuse ordinary generic code the rules have nothing to say about.
            K::Unknown => !dest_written,
        };
        if !ok {
            let hint = match k {
                K::Known(Rcap::Iso) => " — `consume` it to transfer the unique reference",
                K::Unknown => " — the value's reference capability could not be determined",
                _ => "",
            };
            let shown = match k {
                K::Known(kk) => format!("`{}` (aliases as `{}`)", kk.name(), alias(kk).name()),
                K::Unaliased(kk) => format!("`{}`", kk.name()),
                K::Fresh { .. } => "a fresh value whose contents forbid the lift".to_string(),
                K::Unknown => "an undetermined capability".to_string(),
            };
            self.diags.push(
                Diagnostic::error(
                    "DL1603",
                    format!(
                        "cannot store {shown} where `{}` is required ({what}){hint}",
                        dest.name()
                    ),
                )
                .with_span(span, format!("`{}` required here", dest.name())),
            );
        }
    }

    // ----- writes (spec §3: receiver ∈ {iso, trn, ref}; box/val/tag → DL1604) ----

    fn check_write(&mut self, target: &LValue, vk: K, span: Span) {
        match target {
            LValue::Var(name) => {
                let dest = self.lookup(&name.name).and_then(|b| b.rcap);
                if let Some(d) = dest {
                    self.check_storable(vk, d, span, "assignment", false);
                }
                if let Some(b) = self.lookup_mut(&name.name) {
                    b.fresh_lift = None; // rebinding: no longer the tracked fresh literal
                    b.consumed = None; // assignment REVIVES a consumed var (definite re-assignment)
                }
            }
            LValue::Field(base, fname) => {
                let (recv_k, recv_ty) = self.resolve_lvalue(base);
                self.require_writable_receiver(recv_k, target.span(), "field");
                // Value side: storable at the field's declared rcap (alias table unless
                // consumed). Undeterminable field rcap ⇒ the value check is skipped (the
                // receiver check above — the race-relevant half — already ran).
                if let Some((f_rcap, _)) = recv_ty.as_ref().and_then(|t| self.field_info(t, &fname.name)) {
                    self.check_storable(vk, f_rcap, span, &format!("field `{}`", fname.name), true);
                }
            }
            LValue::Index(base, _) => {
                let (recv_k, recv_ty) = self.resolve_lvalue(base);
                self.require_writable_receiver(recv_k, target.span(), "element");
                if let Some(Type::List(elem)) = recv_ty {
                    if let Some(d) = self.default_of(&elem) {
                        self.check_storable(vk, d, span, "list element", false);
                    }
                }
            }
        }
    }

    fn require_writable_receiver(&mut self, k: K, span: Span, what: &str) {
        let refusal = match k {
            K::Known(kk) | K::Unaliased(kk) => {
                if can_write(kk) {
                    None
                } else {
                    Some(format!(
                        "cannot write a {what} through a `{}` receiver — writes require `iso`, `trn`, or `ref`",
                        kk.name()
                    ))
                }
            }
            K::Fresh { natural, .. } => {
                if can_write(natural) {
                    None
                } else {
                    Some(format!("cannot write a {what} through this receiver"))
                }
            }
            K::Unknown => Some(format!(
                "cannot write a {what}: the receiver's reference capability could not be determined — annotate it"
            )),
        };
        if let Some(msg) = refusal {
            self.diags.push(Diagnostic::error("DL1604", msg).with_span(span, "write refused"));
        }
    }

    /// One value crossing an actor boundary (send or spawn argument) — invariant 33: what
    /// arrives must satisfy the parameter's (necessarily sendable) rcap. Arrival is
    /// `alias(κ)` for an ordinary value, full κ when consumed/recovered, and the lift rules
    /// for a fresh literal — so an UNCONSUMED iso aliases as `tag` and cannot satisfy an
    /// `iso` parameter: DL1601 with the exact `consume` repair (spec §10). A `tag` alias of
    /// an iso CAN satisfy a `tag` parameter (opaque identity travels freely).
    fn check_send_arg(&mut self, k: K, dest: Option<Rcap>, arg: &Expr) {
        // The declaration fence (7e) already refused undeterminable parameter rcaps; a None
        // here means that diagnostic exists — don't stack a second one on every send.
        let Some(dest) = dest else { return };
        if matches!(k, K::Unaliased(Rcap::Iso)) {
            self.iso_moves.insert(arg.id());
        }
        let span = arg.span();
        if matches!(self.node_types.get(&arg.id()), Some(Type::PyObj)) {
            self.diags.push(
                Diagnostic::error(
                    "DL1601",
                    "a PyObj can never cross an actor boundary — it is pinned to its creating actor (CPython affinity, invariant 36)",
                )
                .with_span(span, "PyObj is actor-pinned"),
            );
            return;
        }
        let ok = match k {
            K::Unaliased(kk) => subcap(kk, dest),
            K::Known(kk) => subcap(alias(kk), dest),
            K::Fresh { natural, lift_val, lift_iso } => match dest {
                Rcap::Val => lift_val,
                Rcap::Iso | Rcap::Trn => lift_iso,
                d => subcap(natural, d),
            },
            K::Unknown => false,
        };
        if ok {
            return;
        }
        match k {
            K::Known(Rcap::Iso) => {
                // Uniqueness makes it sendable — but only by transfer, never by alias.
                let mut d = Diagnostic::error(
                    "DL1601",
                    "an `iso` value must be `consume`d to cross an actor boundary — the unique reference transfers, it never copies",
                )
                .with_span(span, "add `consume`");
                if matches!(arg, Expr::Var { path, .. } if path.segs.len() == 1) {
                    d = d.with_repair(Repair {
                        id: "consume-iso-send",
                        confidence: Confidence::Exact,
                        authority_widening: false,
                        requires_human: false,
                        edits: vec![Edit {
                            file: span.file,
                            start_byte: span.start,
                            end_byte: span.start,
                            insert: "consume ".into(),
                        }],
                    });
                }
                self.diags.push(d);
            }
            K::Unknown => {
                self.diags.push(
                    Diagnostic::error(
                        "DL1601",
                        "this value's sendability could not be determined — a boundary guarantee is never guessed; annotate the value `iso`, `val`, or `tag`",
                    )
                    .with_span(span, "undecidable sendability"),
                );
            }
            K::Known(kk) | K::Unaliased(kk) => {
                self.diags.push(
                    Diagnostic::error(
                        "DL1601",
                        format!(
                            "a `{}` value cannot cross this actor boundary as `{}` — only `iso` (consumed), `val` (deeply immutable), or `tag` (opaque identity) travel",
                            kk.name(),
                            dest.name()
                        ),
                    )
                    .with_span(span, "not sendable"),
                );
            }
            K::Fresh { .. } => {
                self.diags.push(
                    Diagnostic::error(
                        "DL1601",
                        "this fresh value's contents are not sendable — something aliased and mutable rides along",
                    )
                    .with_span(span, "contents not sendable"),
                );
            }
        }
    }

    /// Parameter destination rcaps for a boundary member (`Actor.member`): written rcap or
    /// the default of the settled signature type, positionally.
    fn boundary_dests(&self, actor: &str, member: &str, params: &[Param]) -> Vec<Option<Rcap>> {
        let key = format!("{actor}.{member}");
        let sig: Vec<Type> = match self.fn_types.get(&key) {
            Some(Type::Fn { params, .. }) => params.clone(),
            _ => Vec::new(),
        };
        params
            .iter()
            .enumerate()
            .map(|(i, p)| p.ty.written_rcap().or_else(|| sig.get(i).and_then(|t| self.default_of(t))))
            .collect()
    }

    /// The rcap + type of an lvalue path (viewpoint-adapting through each field hop).
    fn resolve_lvalue(&mut self, lv: &LValue) -> (K, Option<Type>) {
        match lv {
            LValue::Var(name) => {
                let rcap = self.use_binding(&name.name, name.span, UseKind::Read);
                let ty = self.lookup(&name.name).and_then(|b| b.ty.clone());
                (rcap.map(K::Known).unwrap_or(K::Unknown), ty)
            }
            LValue::Field(base, fname) => {
                let (bk, bty) = self.resolve_lvalue(base);
                let info = bty.as_ref().and_then(|t| self.field_info(t, &fname.name));
                let k = match (bk, &info) {
                    (K::Known(o) | K::Unaliased(o), Some((f_rcap, _))) => match viewpoint(o, *f_rcap) {
                        Some(v) => K::Known(v),
                        None => {
                            self.diags.push(
                                Diagnostic::error(
                                    "DL1604",
                                    "no field access through a `tag` receiver — tag is identity/send only",
                                )
                                .with_span(lv.span(), "field access refused"),
                            );
                            K::Unknown
                        }
                    },
                    (K::Fresh { natural, .. }, Some((f_rcap, _))) => {
                        viewpoint(natural, *f_rcap).map(K::Known).unwrap_or(K::Unknown)
                    }
                    _ => K::Unknown,
                };
                (k, info.and_then(|(_, t)| t))
            }
            LValue::Index(base, _) => {
                let (bk, bty) = self.resolve_lvalue(base);
                let elem = match bty {
                    Some(Type::List(e)) => Some((*e).clone()),
                    _ => None,
                };
                let k = match (bk, &elem) {
                    (K::Known(o) | K::Unaliased(o), Some(et)) => self
                        .default_of(et)
                        .and_then(|f| viewpoint(o, f))
                        .map(K::Known)
                        .unwrap_or(K::Unknown),
                    _ => K::Unknown,
                };
                (k, elem)
            }
        }
    }

    /// Field lookup on a record or actor type: (declared-or-default rcap, field type if
    /// resolvable).
    fn field_info(&self, recv: &Type, fname: &str) -> Option<(Rcap, Option<Type>)> {
        let te = match recv {
            Type::Record(id, _) => {
                let td = self.table.type_def(*id);
                let TypeDefKind::Record(fields) = &td.kind else { return None };
                fields.iter().find(|(n, _)| n == fname).map(|(_, t)| t)?
            }
            Type::Actor(name, _) => {
                let adef = self.table.actors.get(name)?;
                adef.fields.iter().find(|(n, _, _)| n == fname).map(|(_, t, _)| t)?
            }
            _ => return None,
        };
        let fty = lower_shallow(te, self.table);
        let rcap = te
            .written_rcap()
            .or_else(|| fty.as_ref().and_then(|t| self.default_of(t)));
        Some((rcap?, fty))
    }

    // ----- expressions ------------------------------------------------------

    fn walk_expr(&mut self, e: &Expr) -> K {
        match e {
            Expr::Lit { .. } => K::Known(Rcap::Val),
            Expr::Var { path, span, .. } => {
                if path.segs.len() == 1 {
                    let name = &path.segs[0].name;
                    if self.lookup(name).is_some() {
                        return self
                            .use_binding(name, *span, UseKind::Read)
                            .map(K::Known)
                            .unwrap_or(K::Unknown);
                    }
                    // A top-level fn used as a value captures nothing — a `val` closure.
                    if self.table.fns.contains_key(name) {
                        return K::Known(Rcap::Val);
                    }
                    // Module consts are pure values (§5.5).
                    if self.table.consts.contains_key(name) {
                        return K::Known(Rcap::Val);
                    }
                }
                K::Unknown
            }
            Expr::List { items, .. } => {
                let ks: Vec<K> = items.iter().map(|i| self.walk_expr(i)).collect();
                for (i, item) in items.iter().enumerate() {
                    self.escape_arg(item);
                    let _ = (i, &ks);
                }
                fresh_composite(&ks)
            }
            Expr::Record { path, fields, span, .. } => {
                let mut ks = Vec::new();
                for (fname, fe) in fields {
                    let k = self.walk_expr(fe);
                    self.escape_arg(fe);
                    // Field-declared rcap constrains the literal, too.
                    if let Some(ty) = self.node_types.get(&e.id()) {
                        if let Some((f_rcap, _)) = self.field_info(ty, &fname.name) {
                            // Only enforce explicit written demands here; defaulted field
                            // rcaps are satisfied by the lift rules on the whole literal.
                            if self.written_field_rcap(ty, &fname.name).is_some() {
                                self.check_storable(k, f_rcap, *span, &format!("field `{}`", fname.name), true);
                            }
                        }
                    }
                    ks.push(k);
                }
                let _ = path;
                fresh_composite(&ks)
            }
            Expr::Call { callee, args, .. } => {
                self.walk_expr(callee);
                // Free BUILTINS have known retention behavior (they are the prelude, §11):
                // `push(list, v)` mutates its first argument in place — the write rule, not
                // an escape — and stores the second; the pure ones retain nothing at all.
                if let Expr::Var { path, .. } = &**callee {
                    if path.segs.len() == 1 && !self.table.fns.contains_key(&path.segs[0].name) {
                        match path.segs[0].name.as_str() {
                            "push" => {
                                if let Some(recv) = args.first() {
                                    let rk = self.walk_expr(recv);
                                    if matches!(self.node_types.get(&recv.id()), Some(Type::List(_))) {
                                        self.require_writable_receiver(rk, recv.span(), "list element (push)");
                                    }
                                }
                                for a in args.iter().skip(1) {
                                    self.walk_expr(a);
                                    self.escape_arg(a);
                                }
                                return K::Known(Rcap::Val);
                            }
                            "str" | "len" | "int" | "float" | "parse_int" | "range" => {
                                for a in args {
                                    self.walk_expr(a);
                                }
                                return self.default_k(e);
                            }
                            _ => {}
                        }
                    }
                }
                let arg_ks: Vec<K> = args
                    .iter()
                    .map(|a| {
                        let k = self.walk_expr(a);
                        self.escape_arg(a);
                        k
                    })
                    .collect();
                if let Expr::Var { path, .. } = &**callee {
                    if path.segs.len() == 1 {
                        if let Some(sig) = self.table.fns.get(&path.segs[0].name).cloned() {
                            // Argument storability against each parameter's rcap — a `val`
                            // parameter fed an aliased `ref` would let the callee treat (and
                            // later SEND) shared mutable state as immutable. Undeterminable
                            // parameter rcaps (generics) skip the check; the send sites that
                            // could exploit them are themselves fail-closed.
                            let sig_tys: Vec<Type> = match self.fn_types.get(&path.segs[0].name) {
                                Some(Type::Fn { params, .. }) => params.clone(),
                                _ => Vec::new(),
                            };
                            for (i, p) in sig.params.iter().enumerate() {
                                let dest = p
                                    .ty
                                    .written_rcap()
                                    .or_else(|| sig_tys.get(i).and_then(|t| self.default_of(t)));
                                if let (Some(d), Some(k), Some(a)) = (dest, arg_ks.get(i), args.get(i)) {
                                    self.check_storable(
                                        *k,
                                        d,
                                        a.span(),
                                        &format!("argument `{}`", p.name.name),
                                        p.ty.written_rcap().is_some(),
                                    );
                                }
                            }
                            // A call returning a WRITTEN unique rcap hands it over unaliased
                            // (the callee proved it at its own return).
                            if let Some(w) = sig.ret.as_ref().and_then(|r| r.written_rcap()) {
                                return match w {
                                    Rcap::Iso | Rcap::Trn => K::Unaliased(w),
                                    other => K::Known(other),
                                };
                            }
                        }
                    }
                }
                self.default_k(e)
            }
            Expr::Method { recv, name, args, span, .. } => {
                let rk = self.walk_expr(recv);
                let arg_ks: Vec<K> = args
                    .iter()
                    .map(|a| {
                        let k = self.walk_expr(a);
                        self.escape_arg(a);
                        k
                    })
                    .collect();
                // Mutating builtins require a writable receiver (the write rule applied to
                // the one mutating method the stdlib has).
                let recv_ty = self.node_types.get(&recv.id());
                if name.name == "push" && matches!(recv_ty, Some(Type::List(_))) {
                    self.require_writable_receiver(rk, *span, "list element (push)");
                }
                // T-Send argument sendability (7f, spec §4): everything crossing an actor
                // boundary must ARRIVE at the parameter's rcap, and an unconsumed iso gets
                // DL1601 with the EXACT `consume` repair.
                if let Some(Type::Actor(aname, _)) = recv_ty {
                    let aname = aname.clone();
                    let beh_params = self
                        .table
                        .actors
                        .get(&aname)
                        .and_then(|adef| adef.behavior(&name.name))
                        .map(|b| b.params.clone());
                    if let Some(params) = beh_params {
                        let dests = self.boundary_dests(&aname, &name.name, &params);
                        for ((a, k), dest) in args.iter().zip(&arg_ks).zip(dests) {
                            self.check_send_arg(*k, dest, a);
                        }
                    }
                }
                // T-SyncMethod (7e): an actor's `fn` method is callable only from `self` —
                // `self` is the ONLY `ref` to an actor; every outsider holds `tag`, and tag
                // denies synchronous access. Messages are the only cross-actor interface.
                if let Some(Type::Actor(aname, _)) = recv_ty {
                    if let Some(adef) = self.table.actors.get(aname) {
                        if adef.sync_fn(&name.name).is_some()
                            && !matches!(rk, K::Known(Rcap::Ref) | K::Unaliased(Rcap::Ref))
                        {
                            self.diags.push(
                                Diagnostic::error(
                                    "DL1604",
                                    format!(
                                        "`{}.{}` is a synchronous method, callable only from `self` — an actor reference is `tag`, and messages are the only cross-actor interface",
                                        aname, name.name
                                    ),
                                )
                                .with_span(*span, "send a message (`be` behavior) instead"),
                            );
                        }
                    }
                }
                self.default_k(e)
            }
            Expr::Field { recv, name, span, .. } => {
                let rk = self.walk_expr(recv);
                let recv_ty = self.node_types.get(&recv.id()).cloned();
                match (rk, recv_ty.as_ref().and_then(|t| self.field_info(t, &name.name))) {
                    (K::Known(o) | K::Unaliased(o), Some((f, _))) => match viewpoint(o, f) {
                        Some(v) => K::Known(v),
                        None => {
                            self.diags.push(
                                Diagnostic::error(
                                    "DL1604",
                                    "no field access through a `tag` receiver — tag is identity/send only",
                                )
                                .with_span(*span, "field access refused"),
                            );
                            K::Unknown
                        }
                    },
                    (K::Fresh { natural, .. }, Some((f, _))) => {
                        viewpoint(natural, f).map(K::Known).unwrap_or(K::Unknown)
                    }
                    _ => K::Unknown,
                }
            }
            Expr::Index { recv, index, .. } => {
                let rk = self.walk_expr(recv);
                self.walk_expr(index);
                let elem = match self.node_types.get(&recv.id()) {
                    Some(Type::List(e)) => Some((**e).clone()),
                    _ => None,
                };
                match (rk, elem) {
                    (K::Known(o) | K::Unaliased(o), Some(et)) => self
                        .default_of(&et)
                        .and_then(|f| viewpoint(o, f))
                        .map(K::Known)
                        .unwrap_or(K::Unknown),
                    _ => K::Unknown,
                }
            }
            Expr::Unary { operand, .. } => {
                self.walk_expr(operand);
                K::Known(Rcap::Val)
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs);
                self.walk_expr(rhs);
                K::Known(Rcap::Val)
            }
            Expr::If { cond, then_, else_, .. } => {
                self.walk_expr(cond);
                // Branch-sensitive consume states: consumed on EITHER branch ⇒ dead after.
                let pre = self.consumed_snapshot();
                let a = self.walk_block_value(then_);
                let after_then = self.consumed_snapshot();
                self.consumed_restore(&pre);
                let b = match else_ {
                    Some(e2) => self.walk_expr(e2),
                    None => K::Known(Rcap::Val),
                };
                self.consumed_join(&after_then);
                if a == b {
                    a
                } else {
                    self.default_k(e)
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.walk_expr(scrutinee);
                let pre = self.consumed_snapshot();
                let mut arm_states = Vec::new();
                let mut out: Option<K> = None;
                for arm in arms {
                    self.consumed_restore(&pre);
                    self.push_scope();
                    self.bind_pattern(&arm.pattern);
                    let k = self.walk_expr(&arm.body);
                    self.pop_scope();
                    arm_states.push(self.consumed_snapshot());
                    out = Some(match out {
                        None => k,
                        Some(prev) if prev == k => k,
                        Some(_) => K::Unknown,
                    });
                }
                self.consumed_restore(&pre);
                for st in &arm_states {
                    self.consumed_join(st);
                }
                match out {
                    Some(K::Unknown) | None => self.default_k(e),
                    Some(k) => k,
                }
            }
            Expr::Lambda { params, body, .. } => {
                // Closure rcap inference (spec §3): all captures val/tag ⇒ val (sendable);
                // otherwise ref. Captures are free variables resolved in the enclosing env.
                let mut bound: HashSet<String> = params.iter().map(|p| p.name.name.clone()).collect();
                let mut captured: Vec<String> = Vec::new();
                free_vars_block(body, &mut bound, &mut captured);
                let mut all_immutable = true;
                for c in &captured {
                    match self.lookup(c).and_then(|b| b.rcap) {
                        Some(Rcap::Val) | Some(Rcap::Tag) => {}
                        Some(_) => all_immutable = false,
                        // Unresolved in the env: a module fn/const (val) or unknown — a
                        // sendability claim must not rest on a capture we cannot see, unless
                        // it is a known top-level item.
                        None => {
                            if !(self.table.fns.contains_key(c) || self.table.consts.contains_key(c)) {
                                all_immutable = false;
                            }
                        }
                    }
                    self.mark_escaped(c);
                }
                // Walk the body for rule violations inside (captures already marked). The
                // capture boundary makes `consume` of a capture refusable (a closure may run
                // any number of times), while the enclosing recover boundary — if any —
                // still applies to reads that reach past it.
                let saved_cb = self.capture_boundary;
                self.capture_boundary = Some(self.scopes.len());
                self.push_scope();
                for p in params {
                    let rcap = p.ty.written_rcap();
                    self.bind(&p.name.name, Binding { rcap, ty: None, fresh_lift: None, consumed: None });
                }
                self.walk_block_with_tail(body, None, false);
                self.pop_scope();
                self.capture_boundary = saved_cb;
                K::Known(if all_immutable { Rcap::Val } else { Rcap::Ref })
            }
            Expr::Try { inner, .. } => {
                self.walk_expr(inner);
                self.default_k(e)
            }
            Expr::Block(b) => self.walk_block_value(b),
            // T-Consume (spec §3): yields the binding's FULL rcap, unaliased, and kills the
            // binding — flow-sensitively, through the centralized use path.
            Expr::Consume { name, span, .. } => self
                .use_binding(&name.name, *span, UseKind::Consume)
                .map(K::Unaliased)
                .unwrap_or(K::Unknown),
            // T-Recover (spec §3): the body checks under the boundary restriction — only
            // val/tag (or consumed iso) outer bindings are visible (DL1605 via use_binding) —
            // and the result lifts to the target: full κ, unaliased.
            Expr::Recover { target, body, .. } => {
                let saved = self.recover_boundary;
                self.recover_boundary = Some(self.scopes.len());
                self.walk_block_value(body);
                self.recover_boundary = saved;
                K::Unaliased(target.unwrap_or(Rcap::Iso))
            }
            // T-Spawn (7f): construction is a send to the new actor — constructor arguments
            // cross the boundary and get the same sendability rule as behavior sends.
            Expr::Spawn { actor, args, .. } => {
                let arg_ks: Vec<K> = args
                    .iter()
                    .map(|a| {
                        let k = self.walk_expr(a);
                        self.escape_arg(a);
                        k
                    })
                    .collect();
                let aname = actor.segs.last().map(|s| s.name.clone()).unwrap_or_default();
                let ctor_params = self.table.actors.get(&aname).map(|adef| adef.ctor_params.clone());
                if let Some(params) = ctor_params {
                    let dests = self.boundary_dests(&aname, "new", &params);
                    for ((a, k), dest) in args.iter().zip(&arg_ks).zip(dests) {
                        self.check_send_arg(*k, dest, a);
                    }
                }
                K::Known(Rcap::Tag)
            }
        }
    }

    fn walk_block_value(&mut self, b: &Block) -> K {
        self.push_scope();
        let n = b.stmts.len();
        let mut out = K::Known(Rcap::Val);
        for (i, stmt) in b.stmts.iter().enumerate() {
            if i + 1 == n {
                if let Stmt::Expr(e) = stmt {
                    out = self.walk_expr(e);
                    continue;
                }
            }
            self.walk_stmt(stmt, None, false);
        }
        self.pop_scope();
        out
    }

    /// Pattern-bound names: no tracked rcap in v0.7 (documented at module top).
    fn bind_pattern(&mut self, p: &Pattern) {
        match p {
            Pattern::Bind(id) => {
                self.bind(&id.name, Binding { rcap: None, ty: None, fresh_lift: None, consumed: None });
            }
            Pattern::Variant { fields, .. } => {
                for f in fields {
                    self.bind_pattern(f);
                }
            }
            Pattern::Wildcard(_) | Pattern::Lit(_, _) => {}
        }
    }

    /// Passing a named binding anywhere it may be retained ends its fresh-unescaped life.
    fn escape_arg(&mut self, e: &Expr) {
        if let Expr::Var { path, .. } = e {
            if path.segs.len() == 1 {
                self.mark_escaped(&path.segs[0].name);
            }
        }
    }

    fn default_k(&self, e: &Expr) -> K {
        self.node_types
            .get(&e.id())
            .and_then(|t| self.default_of(t))
            .map(K::Known)
            .unwrap_or(K::Unknown)
    }

    fn written_field_rcap(&self, recv: &Type, fname: &str) -> Option<Rcap> {
        let Type::Record(id, _) = recv else { return None };
        let TypeDefKind::Record(fields) = &self.table.type_def(*id).kind else { return None };
        fields.iter().find(|(n, _)| n == fname).and_then(|(_, te)| te.written_rcap())
    }
}

/// The K of a freshly built composite from its element Ks: `ref`-born; liftable to `val` iff
/// every element may live under deep immutability; liftable to `iso` iff every element is
/// sendable-or-unique (nothing aliased and mutable rides along).
fn fresh_composite(elems: &[K]) -> K {
    let mut lift_val = true;
    let mut lift_iso = true;
    for k in elems {
        match k {
            K::Known(kk) => {
                if !subcap(alias(*kk), Rcap::Val) {
                    lift_val = false;
                }
                if !sendable(alias(*kk)) {
                    lift_iso = false;
                }
            }
            K::Unaliased(kk) => {
                if !subcap(*kk, Rcap::Val) {
                    lift_val = false;
                }
                if !sendable(*kk) {
                    lift_iso = false;
                }
            }
            K::Fresh { lift_val: lv, lift_iso: li, .. } => {
                lift_val &= lv;
                lift_iso &= li;
            }
            K::Unknown => {
                lift_val = false;
                lift_iso = false;
            }
        }
    }
    K::Fresh { natural: Rcap::Ref, lift_val, lift_iso }
}

/// Component types of a user type for the default-rcap rule (records: field types; sums: all
/// variant field types), resolved SHALLOWLY — generics unsubstituted resolve to `None`, which
/// the default rule treats as undeterminable (fail closed).
fn components_of(table: &DeclTable, id: TypeDefId) -> Option<Vec<Type>> {
    let td = table.type_def(id);
    let tes: Vec<&TypeExpr> = match &td.kind {
        TypeDefKind::Record(fields) => fields.iter().map(|(_, t)| t).collect(),
        TypeDefKind::Sum(variants) => variants.iter().flat_map(|(_, ts)| ts.iter()).collect(),
        TypeDefKind::Alias(t) => vec![t],
    };
    tes.iter().map(|te| lower_shallow(te, table)).collect()
}

/// A small structural TypeExpr → Type lowering, deep enough for rcap defaults and field
/// chains. Deliberately partial: generics, row machinery, and anything ambiguous return
/// `None` (undeterminable — the pass fails closed where a rule needs the answer).
fn lower_shallow(te: &TypeExpr, table: &DeclTable) -> Option<Type> {
    match te {
        TypeExpr::Rcap { inner, .. } => lower_shallow(inner, table),
        TypeExpr::Fn { .. } => Some(Type::Fn {
            params: Vec::new(),
            ret: Box::new(Type::Unit),
            row: crate::ty::Row::pure(),
        }),
        TypeExpr::Named { path, args, .. } => {
            if path.segs.len() != 1 {
                return None;
            }
            let name = path.segs[0].name.as_str();
            Some(match name {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Bool" => Type::Bool,
                "Str" => Type::Str,
                "Unit" => Type::Unit,
                "Root" => Type::Root,
                "ForeignPtr" => Type::ForeignPtr,
                "PyObj" => Type::PyObj,
                "List" => Type::List(Box::new(lower_shallow(args.first()?, table)?)),
                "Option" => Type::Option(Box::new(lower_shallow(args.first()?, table)?)),
                "Result" => Type::Result(
                    Box::new(lower_shallow(args.first()?, table)?),
                    Box::new(lower_shallow(args.get(1)?, table)?),
                ),
                "Secret" => Type::Secret(Box::new(lower_shallow(args.first()?, table)?)),
                "Cap" => {
                    let arg = args.first()?;
                    let TypeExpr::Named { path, .. } = arg else { return None };
                    Type::Cap(crate::ty::ResourceKind::from_name(&path.segs.last()?.name)?)
                }
                _ => {
                    if table.foreigns.contains_key(name) {
                        Type::Foreign(name.to_string())
                    } else if let Some(adef) = table.actors.get(name) {
                        if !adef.generics.is_empty() {
                            return None; // unsubstituted generics: undeterminable
                        }
                        Type::Actor(name.to_string(), Vec::new())
                    } else {
                        let id = *table.type_ix.get(name)?;
                        let td = table.type_def(id);
                        if !td.generics.is_empty() {
                            return None; // unsubstituted generics: undeterminable
                        }
                        match td.kind {
                            TypeDefKind::Record(_) => Type::Record(id, Vec::new()),
                            TypeDefKind::Sum(_) => Type::Sum(id, Vec::new()),
                            TypeDefKind::Alias(ref t) => return lower_shallow(&t.clone(), table),
                        }
                    }
                }
            })
        }
    }
}

/// Free variables of a block: single-segment `Var`/`Consume` names not bound within.
fn free_vars_block(b: &Block, bound: &mut HashSet<String>, out: &mut Vec<String>) {
    let added: Vec<String> = Vec::new();
    let mut local_added = added;
    for stmt in &b.stmts {
        match stmt {
            Stmt::Let { name, value, .. } => {
                free_vars_expr(value, bound, out);
                if bound.insert(name.name.clone()) {
                    local_added.push(name.name.clone());
                }
            }
            Stmt::Assign { target, value, .. } => {
                free_vars_lvalue(target, bound, out);
                free_vars_expr(value, bound, out);
            }
            Stmt::While { cond, body, .. } => {
                free_vars_expr(cond, bound, out);
                free_vars_block(body, bound, out);
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    free_vars_expr(v, bound, out);
                }
            }
            Stmt::Expr(e) => free_vars_expr(e, bound, out),
        }
    }
    for name in local_added {
        bound.remove(&name);
    }
}

fn free_vars_lvalue(lv: &LValue, bound: &mut HashSet<String>, out: &mut Vec<String>) {
    match lv {
        LValue::Var(id) => {
            if !bound.contains(&id.name) {
                out.push(id.name.clone());
            }
        }
        LValue::Field(base, _) => free_vars_lvalue(base, bound, out),
        LValue::Index(base, idx) => {
            free_vars_lvalue(base, bound, out);
            free_vars_expr(idx, bound, out);
        }
    }
}

fn free_vars_expr(e: &Expr, bound: &mut HashSet<String>, out: &mut Vec<String>) {
    match e {
        Expr::Var { path, .. } => {
            if path.segs.len() == 1 && !bound.contains(&path.segs[0].name) {
                out.push(path.segs[0].name.clone());
            }
        }
        Expr::Consume { name, .. } => {
            if !bound.contains(&name.name) {
                out.push(name.name.clone());
            }
        }
        Expr::Lit { .. } => {}
        Expr::List { items, .. } => items.iter().for_each(|i| free_vars_expr(i, bound, out)),
        Expr::Record { fields, .. } => fields.iter().for_each(|(_, v)| free_vars_expr(v, bound, out)),
        Expr::Call { callee, args, .. } => {
            free_vars_expr(callee, bound, out);
            args.iter().for_each(|a| free_vars_expr(a, bound, out));
        }
        Expr::Method { recv, args, .. } => {
            free_vars_expr(recv, bound, out);
            args.iter().for_each(|a| free_vars_expr(a, bound, out));
        }
        Expr::Field { recv, .. } => free_vars_expr(recv, bound, out),
        Expr::Index { recv, index, .. } => {
            free_vars_expr(recv, bound, out);
            free_vars_expr(index, bound, out);
        }
        Expr::Unary { operand, .. } => free_vars_expr(operand, bound, out),
        Expr::Binary { lhs, rhs, .. } => {
            free_vars_expr(lhs, bound, out);
            free_vars_expr(rhs, bound, out);
        }
        Expr::If { cond, then_, else_, .. } => {
            free_vars_expr(cond, bound, out);
            free_vars_block(then_, bound, out);
            if let Some(e2) = else_ {
                free_vars_expr(e2, bound, out);
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            free_vars_expr(scrutinee, bound, out);
            for arm in arms {
                let mut names = Vec::new();
                pattern_names(&arm.pattern, &mut names);
                let newly: Vec<String> =
                    names.into_iter().filter(|n| bound.insert(n.clone())).collect();
                free_vars_expr(&arm.body, bound, out);
                for n in newly {
                    bound.remove(&n);
                }
            }
        }
        Expr::Lambda { params, body, .. } => {
            let newly: Vec<String> = params
                .iter()
                .map(|p| p.name.name.clone())
                .filter(|n| bound.insert(n.clone()))
                .collect();
            free_vars_block(body, bound, out);
            for n in newly {
                bound.remove(&n);
            }
        }
        Expr::Try { inner, .. } => free_vars_expr(inner, bound, out),
        Expr::Block(b) => free_vars_block(b, bound, out),
        Expr::Spawn { args, .. } => args.iter().for_each(|a| free_vars_expr(a, bound, out)),
        Expr::Recover { body, .. } => free_vars_block(body, bound, out),
    }
}

fn pattern_names(p: &Pattern, out: &mut Vec<String>) {
    match p {
        Pattern::Bind(id) => out.push(id.name.clone()),
        Pattern::Variant { fields, .. } => fields.iter().for_each(|f| pattern_names(f, out)),
        Pattern::Wildcard(_) | Pattern::Lit(_, _) => {}
    }
}

/// `consume` targets free in a block (respecting let/param/pattern shadowing) — the loop
/// pre-scan that makes consume-in-a-loop refuse loop-carried dead bindings.
fn consumed_free_names(b: &Block, bound: &mut HashSet<String>, out: &mut Vec<(String, Span)>) {
    let mut local_added: Vec<String> = Vec::new();
    for stmt in &b.stmts {
        match stmt {
            Stmt::Let { name, value, .. } => {
                consumed_free_in_expr(value, bound, out);
                if bound.insert(name.name.clone()) {
                    local_added.push(name.name.clone());
                }
            }
            Stmt::Assign { value, .. } => consumed_free_in_expr(value, bound, out),
            Stmt::While { cond, body, .. } => {
                consumed_free_in_expr(cond, bound, out);
                consumed_free_names(body, bound, out);
            }
            Stmt::Return { value: Some(v), .. } => consumed_free_in_expr(v, bound, out),
            Stmt::Return { value: None, .. } => {}
            Stmt::Expr(e) => consumed_free_in_expr(e, bound, out),
        }
    }
    for n in local_added {
        bound.remove(&n);
    }
}

fn consumed_free_in_expr(e: &Expr, bound: &mut HashSet<String>, out: &mut Vec<(String, Span)>) {
    match e {
        Expr::Consume { name, span, .. } => {
            if !bound.contains(&name.name) {
                out.push((name.name.clone(), *span));
            }
        }
        Expr::Lit { .. } | Expr::Var { .. } => {}
        Expr::List { items, .. } => items.iter().for_each(|i| consumed_free_in_expr(i, bound, out)),
        Expr::Record { fields, .. } => {
            fields.iter().for_each(|(_, v)| consumed_free_in_expr(v, bound, out))
        }
        Expr::Call { callee, args, .. } => {
            consumed_free_in_expr(callee, bound, out);
            args.iter().for_each(|a| consumed_free_in_expr(a, bound, out));
        }
        Expr::Method { recv, args, .. } => {
            consumed_free_in_expr(recv, bound, out);
            args.iter().for_each(|a| consumed_free_in_expr(a, bound, out));
        }
        Expr::Field { recv, .. } => consumed_free_in_expr(recv, bound, out),
        Expr::Index { recv, index, .. } => {
            consumed_free_in_expr(recv, bound, out);
            consumed_free_in_expr(index, bound, out);
        }
        Expr::Unary { operand, .. } => consumed_free_in_expr(operand, bound, out),
        Expr::Binary { lhs, rhs, .. } => {
            consumed_free_in_expr(lhs, bound, out);
            consumed_free_in_expr(rhs, bound, out);
        }
        Expr::If { cond, then_, else_, .. } => {
            consumed_free_in_expr(cond, bound, out);
            consumed_free_names(then_, bound, out);
            if let Some(e2) = else_ {
                consumed_free_in_expr(e2, bound, out);
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            consumed_free_in_expr(scrutinee, bound, out);
            for arm in arms {
                let mut names = Vec::new();
                pattern_names(&arm.pattern, &mut names);
                let newly: Vec<String> = names.into_iter().filter(|n| bound.insert(n.clone())).collect();
                consumed_free_in_expr(&arm.body, bound, out);
                for n in newly {
                    bound.remove(&n);
                }
            }
        }
        // A consume inside a nested lambda is refused by the capture-boundary rule at walk
        // time, not treated as this loop iteration's consume.
        Expr::Lambda { .. } => {}
        Expr::Try { inner, .. } => consumed_free_in_expr(inner, bound, out),
        Expr::Block(b) => consumed_free_names(b, bound, out),
        Expr::Spawn { args, .. } => args.iter().for_each(|a| consumed_free_in_expr(a, bound, out)),
        Expr::Recover { body, .. } => consumed_free_names(body, bound, out),
    }
}
