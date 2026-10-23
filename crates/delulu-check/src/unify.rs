//! Inference context and unification (spec §6.2, audit R-3/R-3b).
//!
//! Soundness-critical rules implemented here:
//! - **R-3:** rows unify by *equality* (plus row-variable binding). There is NO subsumption
//!   inside unification; parameter-position rows are never implicitly widened or narrowed.
//!   The two subsumption sites (T-Fn declaration, annotated-lambda) live in `check.rs`.
//! - **R-3b:** a row variable that would receive conflicting closed bindings is a *conflict*
//!   (mapped to DL0504 by the checker), never a union-merge.

use std::cell::Cell;
use std::collections::{BTreeSet, HashSet};

use crate::ty::{Effect, Row, RowVar, Type, TypeVar};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnifyError {
    /// Structural type mismatch (mapped to DL0401).
    Type,
    /// Closed rows differ with no bound row-variable involved — e.g. the F-3 contravariance
    /// ascription. Mapped to DL0401 (structural).
    RowMismatch,
    /// A row variable already bound to one closed row is required to equal a different closed
    /// row (audit F-3b). Mapped to DL0504 — never merged to a union.
    RowConflict,
    /// Occurs check failed (an infinite type/row).
    Occurs,
}

/// The most parts one type may have written out (RW 2.3, D-V2-179). A binding is shared by every place its variable
/// stands, so a type of a few nodes through its variables can be exponentially large written out: `let a1 = Pair { l:
/// a0, r: a0 }` … `a22` is ~4 million parts, and every walk that wrote one out doubled with each line (22 lines, 5.8 s).
/// A type past this is written out only to its top, and the program is refused (DL0211).
pub const TYPE_PARTS_LIMIT: usize = 100_000;

/// A type written out in more parts than this counts toward [`TYPE_PARTS_TOTAL`]; one this small is written out
/// whatever the total, and is all a walk writes out once a type has been refused.
pub const TYPE_PARTS_FREE: usize = 1_000;

/// The most parts the module's large types (past [`TYPE_PARTS_FREE`]) may have written out in all: the bound on many
/// references to one type just under [`TYPE_PARTS_LIMIT`].
pub const TYPE_PARTS_TOTAL: u64 = 20_000_000;

/// The inference substitution: bindings for type and row variables.
pub struct InferCtx {
    type_bindings: Vec<Option<Type>>,
    row_bindings: Vec<Option<Row>>,
    /// Parts written out so far by types past [`TYPE_PARTS_FREE`].
    written: Cell<u64>,
    /// Whether a type was too large to write out (`TYPE_PARTS_LIMIT`, `TYPE_PARTS_TOTAL`); the checker refuses the
    /// program when one was.
    too_large: Cell<bool>,
}

impl InferCtx {
    pub fn new() -> Self {
        InferCtx { type_bindings: Vec::new(), row_bindings: Vec::new(), written: Cell::new(0), too_large: Cell::new(false) }
    }

    /// Whether a type has been too large to write out (RW 2.3): the checker refuses the program with DL0211.
    pub fn too_large(&self) -> bool {
        self.too_large.get()
    }

    pub fn fresh_type(&mut self) -> Type {
        let v = self.type_bindings.len() as u32;
        self.type_bindings.push(None);
        Type::Var(TypeVar(v))
    }

    pub fn fresh_row_var(&mut self) -> RowVar {
        let v = self.row_bindings.len() as u32;
        self.row_bindings.push(None);
        RowVar(v)
    }

    pub fn fresh_row(&mut self) -> Row {
        Row { effects: BTreeSet::new(), tail: Some(self.fresh_row_var()) }
    }

    // ----- resolution ------------------------------------------------------

    /// Follow type-variable bindings at the top level only.
    fn resolve_type_shallow(&self, t: &Type) -> Type {
        let mut cur = t.clone();
        while let Type::Var(TypeVar(v)) = cur {
            match &self.type_bindings[v as usize] {
                Some(bound) => cur = bound.clone(),
                None => break,
            }
        }
        cur
    }

    /// Fully substitute a type (for final reporting/side tables), within [`TYPE_PARTS_LIMIT`] and
    /// [`TYPE_PARTS_TOTAL`]. A type past either is returned with only its top resolved and marks the context
    /// [`too_large`](Self::too_large), so the program is refused; from then on a type is written out only within
    /// [`TYPE_PARTS_FREE`], which keeps the rest of the check as cheap as an ordinary program's.
    pub fn apply_type(&self, t: &Type) -> Type {
        let budget = if self.too_large.get() {
            TYPE_PARTS_FREE
        } else {
            let total_left = TYPE_PARTS_TOTAL.saturating_sub(self.written.get());
            TYPE_PARTS_LIMIT.min(usize::try_from(total_left).unwrap_or(usize::MAX)).max(TYPE_PARTS_FREE)
        };
        let mut left = budget;
        match self.apply_within(t, &mut left) {
            Some(full) => {
                let parts = budget - left;
                if parts > TYPE_PARTS_FREE {
                    self.written.set(self.written.get().saturating_add(parts as u64));
                }
                full
            }
            None => {
                self.too_large.set(true);
                self.resolve_type_shallow(t)
            }
        }
    }

    /// `t` written out in at most `left` parts; `None` when it needs more.
    fn apply_within(&self, t: &Type, left: &mut usize) -> Option<Type> {
        *left = left.checked_sub(1)?;
        Some(match self.resolve_type_shallow(t) {
            Type::List(inner) => Type::List(Box::new(self.apply_within(&inner, left)?)),
            Type::Option(inner) => Type::Option(Box::new(self.apply_within(&inner, left)?)),
            Type::Map(k, v) => Type::Map(Box::new(self.apply_within(&k, left)?), Box::new(self.apply_within(&v, left)?)),
            Type::Result(o, e) => {
                Type::Result(Box::new(self.apply_within(&o, left)?), Box::new(self.apply_within(&e, left)?))
            }
            Type::Record(id, args) => {
                Type::Record(id, args.iter().map(|a| self.apply_within(a, left)).collect::<Option<_>>()?)
            }
            Type::Sum(id, args) => Type::Sum(id, args.iter().map(|a| self.apply_within(a, left)).collect::<Option<_>>()?),
            Type::Fn { params, ret, row } => Type::Fn {
                params: params.iter().map(|p| self.apply_within(p, left)).collect::<Option<_>>()?,
                ret: Box::new(self.apply_within(&ret, left)?),
                row: self.apply_row(&row),
            },
            Type::Secret(inner) => Type::Secret(Box::new(self.apply_within(&inner, left)?)),
            Type::Plugin(inner) => Type::Plugin(Box::new(self.apply_within(&inner, left)?)),
            Type::Actor(name, args) => {
                Type::Actor(name, args.iter().map(|a| self.apply_within(a, left)).collect::<Option<_>>()?)
            }
            other => other,
        })
    }

    /// Resolve a row: expand bound tail variables, accumulating their effects. Returns the
    /// normalized row (unbound or no tail) and whether any bound variable was expanded — the
    /// latter distinguishes a genuine row conflict (R-3b) from a plain closed mismatch (R-3).
    fn resolve_row_tracked(&self, r: &Row) -> (Row, bool) {
        let mut effects = r.effects.clone();
        let mut tail = r.tail;
        let mut expanded = false;
        while let Some(RowVar(v)) = tail {
            match &self.row_bindings[v as usize] {
                Some(bound) => {
                    expanded = true;
                    effects.extend(bound.effects.iter().cloned());
                    tail = bound.tail;
                }
                None => break,
            }
        }
        (Row { effects, tail }, expanded)
    }

    pub fn apply_row(&self, r: &Row) -> Row {
        self.resolve_row_tracked(r).0
    }

    fn bind_row(&mut self, v: RowVar, row: Row) -> Result<(), UnifyError> {
        // Occurs check: the row's own tail chain must not lead back to v.
        let (resolved, _) = self.resolve_row_tracked(&row);
        if resolved.tail == Some(v) {
            return Err(UnifyError::Occurs);
        }
        self.row_bindings[v.0 as usize] = Some(row);
        Ok(())
    }

    // ----- unification -----------------------------------------------------

    pub fn unify_type(&mut self, a: &Type, b: &Type) -> Result<(), UnifyError> {
        self.unify_in(a, b, &mut HashSet::new())
    }

    /// `unify_type`, each pair of variables unified once per call (RW 2.3). A variable stands for its binding at every
    /// place it occurs, so two types of a few nodes through their variables (`a22` and a twin built the same way) were
    /// unified as the exponential trees they are written out. A pair met again either was unified (bindings only grow,
    /// so it still is) or failed, and a failure ends the call; a pair cannot meet itself inside its own binding, which
    /// the occurs check rules out.
    fn unify_in(&mut self, a: &Type, b: &Type, met: &mut HashSet<(u32, u32)>) -> Result<(), UnifyError> {
        if let (Type::Var(TypeVar(x)), Type::Var(TypeVar(y))) = (a, b) {
            if x == y || !met.insert((*x, *y)) {
                return Ok(());
            }
        }
        let a = self.resolve_type_shallow(a);
        let b = self.resolve_type_shallow(b);
        match (&a, &b) {
            (Type::Var(x), Type::Var(y)) if x == y => Ok(()),
            (Type::Var(TypeVar(v)), _) => {
                if self.occurs_type(*v, &b) {
                    return Err(UnifyError::Occurs);
                }
                self.type_bindings[*v as usize] = Some(b);
                Ok(())
            }
            (_, Type::Var(TypeVar(v))) => {
                if self.occurs_type(*v, &a) {
                    return Err(UnifyError::Occurs);
                }
                self.type_bindings[*v as usize] = Some(a);
                Ok(())
            }
            (Type::Int, Type::Int)
            | (Type::Float, Type::Float)
            | (Type::Bool, Type::Bool)
            | (Type::Str, Type::Str)
            | (Type::Unit, Type::Unit)
            | (Type::Root, Type::Root) => Ok(()),
            (Type::List(x), Type::List(y)) => self.unify_in(x, y, met),
            (Type::Option(x), Type::Option(y)) => self.unify_in(x, y, met),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.unify_in(k1, k2, met)?;
                self.unify_in(v1, v2, met)
            }
            (Type::Result(x1, x2), Type::Result(y1, y2)) => {
                self.unify_in(x1, y1, met)?;
                self.unify_in(x2, y2, met)
            }
            (Type::Secret(x), Type::Secret(y)) => self.unify_in(x, y, met),
            (Type::ForeignPtr, Type::ForeignPtr) | (Type::PyObj, Type::PyObj) => Ok(()),
            (Type::Foreign(x), Type::Foreign(y)) if x == y => Ok(()),
            // Actor references (Stage 7) are nominal by name; their type arguments unify
            // structurally (a `Cell[Int]` reference is not a `Cell[Str]` reference).
            (Type::Actor(x, a1), Type::Actor(y, a2)) if x == y && a1.len() == a2.len() => {
                for (p, q) in a1.iter().zip(a2.iter()) {
                    self.unify_in(p, q, met)?;
                }
                Ok(())
            }
            // `Plugin[C]` unifies structurally, so the class marker inside is an ordinary
            // inference position: `let p: Plugin[Contained] = load(…)` pins the `C` that `load`
            // returned as a fresh variable (deviation 4). `Verified`/`Contained` are nominal and
            // NEVER unify with each other — the class is exact, never coerced (invariant 29).
            (Type::Plugin(x), Type::Plugin(y)) => self.unify_in(x, y, met),
            (Type::Verified, Type::Verified) | (Type::Contained, Type::Contained) => Ok(()),
            (Type::Cap(x), Type::Cap(y)) if x == y => Ok(()),
            (Type::Record(id1, a1), Type::Record(id2, a2)) | (Type::Sum(id1, a1), Type::Sum(id2, a2))
                if id1 == id2 && a1.len() == a2.len() =>
            {
                for (x, y) in a1.iter().zip(a2) {
                    self.unify_in(x, y, met)?;
                }
                Ok(())
            }
            (Type::Fn { params: p1, ret: r1, row: row1 }, Type::Fn { params: p2, ret: r2, row: row2 })
                if p1.len() == p2.len() =>
            {
                for (x, y) in p1.iter().zip(p2) {
                    self.unify_in(x, y, met)?;
                }
                self.unify_in(r1, r2, met)?;
                // Row positions unify by equality (R-3): no subsumption, either variance.
                self.unify_row(row1, row2)
            }
            _ => Err(UnifyError::Type),
        }
    }

    /// Unify two rows by equality (R-3), with row-variable binding.
    pub fn unify_row(&mut self, a: &Row, b: &Row) -> Result<(), UnifyError> {
        let (ra, exp_a) = self.resolve_row_tracked(a);
        let (rb, exp_b) = self.resolve_row_tracked(b);
        let expanded = exp_a || exp_b;

        match (ra.tail, rb.tail) {
            (None, None) => {
                if ra.effects == rb.effects {
                    Ok(())
                } else if expanded {
                    // A bound row variable forced conflicting closed rows (R-3b).
                    Err(UnifyError::RowConflict)
                } else {
                    Err(UnifyError::RowMismatch)
                }
            }
            (Some(rv), None) => {
                // {A | rv} = B (closed). Equality requires A ⊆ B; bind rv := B \ A.
                if !ra.effects.is_subset(&rb.effects) {
                    return Err(if expanded { UnifyError::RowConflict } else { UnifyError::RowMismatch });
                }
                let remainder: BTreeSet<Effect> = rb.effects.difference(&ra.effects).cloned().collect();
                self.bind_row(rv, Row::closed(remainder))
            }
            (None, Some(rv)) => {
                if !rb.effects.is_subset(&ra.effects) {
                    return Err(if expanded { UnifyError::RowConflict } else { UnifyError::RowMismatch });
                }
                let remainder: BTreeSet<Effect> = ra.effects.difference(&rb.effects).cloned().collect();
                self.bind_row(rv, Row::closed(remainder))
            }
            (Some(rv), Some(sv)) => {
                if rv == sv {
                    if ra.effects == rb.effects {
                        Ok(())
                    } else {
                        Err(UnifyError::RowConflict)
                    }
                } else {
                    // {A | rv} = {B | sv}: introduce a fresh shared tail.
                    let fresh = self.fresh_row_var();
                    let b_minus_a: BTreeSet<Effect> = rb.effects.difference(&ra.effects).cloned().collect();
                    let a_minus_b: BTreeSet<Effect> = ra.effects.difference(&rb.effects).cloned().collect();
                    self.bind_row(rv, Row { effects: b_minus_a, tail: Some(fresh) })?;
                    self.bind_row(sv, Row { effects: a_minus_b, tail: Some(fresh) })
                }
            }
        }
    }

    fn occurs_type(&self, v: u32, t: &Type) -> bool {
        self.occurs_in(v, t, &mut std::collections::HashSet::new())
    }

    /// Whether the unbound variable `v` occurs in `t`, each bound variable followed once (RW 2.3): `explored` holds each
    /// variable whose binding has been searched without finding `v`. A binding is shared by every place its variable
    /// stands, so `let a1 = Pair { l: a0, r: a0 }` … `a22` is a type of 22 nodes through its variables and ~4 million
    /// written out; followed at every place, each `let`'s check walked the whole tree, and 22 lines took 5.8 s.
    fn occurs_in(&self, v: u32, t: &Type, explored: &mut std::collections::HashSet<u32>) -> bool {
        match t {
            Type::Var(TypeVar(w)) => {
                if *w == v {
                    return true;
                }
                match &self.type_bindings[*w as usize] {
                    Some(bound) if explored.insert(*w) => self.occurs_in(v, bound, explored),
                    _ => false,
                }
            }
            Type::List(inner) | Type::Option(inner) | Type::Secret(inner) | Type::Plugin(inner) => {
                self.occurs_in(v, inner, explored)
            }
            Type::Result(a, b) | Type::Map(a, b) => self.occurs_in(v, a, explored) || self.occurs_in(v, b, explored),
            Type::Record(_, args) | Type::Sum(_, args) | Type::Actor(_, args) => {
                args.iter().any(|a| self.occurs_in(v, a, explored))
            }
            Type::Fn { params, ret, .. } => {
                params.iter().any(|p| self.occurs_in(v, p, explored)) || self.occurs_in(v, ret, explored)
            }
            _ => false,
        }
    }
}

impl Default for InferCtx {
    fn default() -> Self {
        InferCtx::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::{ResourceKind, Type};

    fn row(effs: &[Effect]) -> Row {
        Row::closed(effs.iter().cloned().collect())
    }

    #[test]
    fn basic_type_unification() {
        let mut cx = InferCtx::new();
        let v = cx.fresh_type();
        assert!(cx.unify_type(&v, &Type::Int).is_ok());
        assert_eq!(cx.apply_type(&v), Type::Int);
    }

    #[test]
    fn type_mismatch_is_reported() {
        let mut cx = InferCtx::new();
        assert_eq!(cx.unify_type(&Type::Int, &Type::Bool), Err(UnifyError::Type));
        assert_eq!(cx.unify_type(&Type::Cap(ResourceKind::FsRead), &Type::Cap(ResourceKind::FsWrite)), Err(UnifyError::Type));
    }

    #[test]
    fn closed_rows_unify_by_equality_not_subsumption() {
        // R-3: {} and {Write} do NOT unify (the F-3 contravariance rejection in miniature).
        let mut cx = InferCtx::new();
        assert_eq!(cx.unify_row(&Row::pure(), &row(&[Effect::Write])), Err(UnifyError::RowMismatch));
    }

    #[test]
    fn row_variable_binds_to_remainder() {
        // {Read | e} = {Read, Net}  ⇒  e := {Net}
        let mut cx = InferCtx::new();
        let e = cx.fresh_row_var();
        let open = Row { effects: [Effect::Read].into_iter().collect(), tail: Some(e) };
        assert!(cx.unify_row(&open, &row(&[Effect::Read, Effect::Net])).is_ok());
        let bound = cx.apply_row(&Row { effects: BTreeSet::new(), tail: Some(e) });
        assert_eq!(bound, row(&[Effect::Net]));
    }

    #[test]
    fn conflicting_row_variable_bindings_are_a_conflict_not_a_union() {
        // R-3b / audit F-3b: e ~ {Read} then e ~ {Write}  ⇒  RowConflict (DL0504), never {Read,Write}.
        let mut cx = InferCtx::new();
        let e = cx.fresh_row_var();
        let use1 = Row { effects: BTreeSet::new(), tail: Some(e) };
        let use2 = Row { effects: BTreeSet::new(), tail: Some(e) };
        assert!(cx.unify_row(&use1, &row(&[Effect::Read])).is_ok());
        assert_eq!(cx.unify_row(&use2, &row(&[Effect::Write])), Err(UnifyError::RowConflict));
    }

    #[test]
    fn fn_type_rows_unify_invariantly() {
        // fn() -> Unit !{}  vs  fn() -> Unit !{Write}  ⇒  mismatch (the F-3 exploit, closed).
        let mut cx = InferCtx::new();
        let a = Type::Fn { params: vec![], ret: Box::new(Type::Unit), row: Row::pure() };
        let b = Type::Fn { params: vec![], ret: Box::new(Type::Unit), row: row(&[Effect::Write]) };
        assert_eq!(cx.unify_type(&a, &b), Err(UnifyError::RowMismatch));
    }
}
