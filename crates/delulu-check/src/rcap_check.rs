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

use delulu_diag::{Diagnostic, Span};
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
}

pub fn check_rcaps(
    module: &Module,
    table: &DeclTable,
    node_types: &HashMap<NodeId, Type>,
    fn_types: &HashMap<String, Type>,
) -> Vec<Diagnostic> {
    let mut pass = Pass { table, node_types, fn_types, diags: Vec::new(), scopes: Vec::new() };
    for item in &module.items {
        if let Item::Fn(f) = item {
            pass.check_fn(f);
        }
    }
    pass.diags
}

struct Pass<'a> {
    table: &'a DeclTable,
    node_types: &'a HashMap<NodeId, Type>,
    fn_types: &'a HashMap<String, Type>,
    diags: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, Binding>>,
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
            self.bind(&p.name.name, Binding { rcap, ty, fresh_lift: None });
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
                self.bind(&name.name, Binding { rcap, ty: vty, fresh_lift });
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
                let (dest, _) = match self.lookup(&name.name) {
                    Some(b) => (b.rcap, b.ty.clone()),
                    None => (None, None),
                };
                if let Some(d) = dest {
                    self.check_storable(vk, d, span, "assignment", false);
                }
                if let Some(b) = self.lookup_mut(&name.name) {
                    b.fresh_lift = None; // rebinding: no longer the tracked fresh literal
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

    /// The rcap + type of an lvalue path (viewpoint-adapting through each field hop).
    fn resolve_lvalue(&mut self, lv: &LValue) -> (K, Option<Type>) {
        match lv {
            LValue::Var(name) => match self.lookup(&name.name) {
                Some(b) => (b.rcap.map(K::Known).unwrap_or(K::Unknown), b.ty.clone()),
                None => (K::Unknown, None),
            },
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

    /// Field lookup on a record type: (declared-or-default rcap, field type if resolvable).
    fn field_info(&self, recv: &Type, fname: &str) -> Option<(Rcap, Option<Type>)> {
        let Type::Record(id, _) = recv else { return None };
        let td = self.table.type_def(*id);
        let TypeDefKind::Record(fields) = &td.kind else { return None };
        let (_, te) = fields.iter().find(|(n, _)| n == fname)?;
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
            Expr::Var { path, .. } => {
                if path.segs.len() == 1 {
                    if let Some(b) = self.lookup(&path.segs[0].name) {
                        return b.rcap.map(K::Known).unwrap_or(K::Unknown);
                    }
                    // A top-level fn used as a value captures nothing — a `val` closure.
                    if self.table.fns.contains_key(&path.segs[0].name) {
                        return K::Known(Rcap::Val);
                    }
                    // Module consts are pure values (§5.5).
                    if self.table.consts.contains_key(&path.segs[0].name) {
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
                for a in args {
                    self.walk_expr(a);
                    self.escape_arg(a);
                }
                // Mutating builtins require a writable receiver (the write rule applied to
                // the one mutating method the stdlib has).
                let recv_ty = self.node_types.get(&recv.id());
                if name.name == "push" && matches!(recv_ty, Some(Type::List(_))) {
                    self.require_writable_receiver(rk, *span, "list element (push)");
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
                let a = self.walk_block_value(then_);
                let b = match else_ {
                    Some(e2) => self.walk_expr(e2),
                    None => K::Known(Rcap::Val),
                };
                if a == b {
                    a
                } else {
                    self.default_k(e)
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.walk_expr(scrutinee);
                let mut out: Option<K> = None;
                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern);
                    let k = self.walk_expr(&arm.body);
                    self.pop_scope();
                    out = Some(match out {
                        None => k,
                        Some(prev) if prev == k => k,
                        Some(_) => K::Unknown,
                    });
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
                // Walk the body for rule violations inside (captures already marked).
                self.push_scope();
                for p in params {
                    let rcap = p.ty.written_rcap();
                    self.bind(&p.name.name, Binding { rcap, ty: None, fresh_lift: None });
                }
                self.walk_block_with_tail(body, None, false);
                self.pop_scope();
                K::Known(if all_immutable { Rcap::Val } else { Rcap::Ref })
            }
            Expr::Try { inner, .. } => {
                self.walk_expr(inner);
                self.default_k(e)
            }
            Expr::Block(b) => self.walk_block_value(b),
            // Stage 7d territory (flow analysis); until then `consume x` already yields the
            // binding's FULL rcap, unaliased — that much is 7c-true.
            Expr::Consume { name, .. } => {
                let k = self
                    .lookup(&name.name)
                    .and_then(|b| b.rcap)
                    .map(K::Unaliased)
                    .unwrap_or(K::Unknown);
                self.mark_escaped(&name.name);
                k
            }
            // A recover block's result is re-proved and lifted: full κ, unaliased (7d adds
            // the environment restriction; the lift itself is 7c-true).
            Expr::Recover { target, body, .. } => {
                self.walk_block_value(body);
                K::Unaliased(target.unwrap_or(Rcap::Iso))
            }
            // 7f territory: spawn types as tag; sends check sendability there.
            Expr::Spawn { args, .. } => {
                for a in args {
                    self.walk_expr(a);
                    self.escape_arg(a);
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
                self.bind(&id.name, Binding { rcap: None, ty: None, fresh_lift: None });
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
