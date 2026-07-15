//! Inference context and unification (spec §6.2, audit R-3/R-3b).
//!
//! Soundness-critical rules implemented here:
//! - **R-3:** rows unify by *equality* (plus row-variable binding). There is NO subsumption
//!   inside unification; parameter-position rows are never implicitly widened or narrowed.
//!   The two subsumption sites (T-Fn declaration, annotated-lambda) live in `check.rs`.
//! - **R-3b:** a row variable that would receive conflicting closed bindings is a *conflict*
//!   (mapped to DL0504 by the checker), never a union-merge.

use std::collections::BTreeSet;

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

/// The inference substitution: bindings for type and row variables.
pub struct InferCtx {
    type_bindings: Vec<Option<Type>>,
    row_bindings: Vec<Option<Row>>,
}

impl InferCtx {
    pub fn new() -> Self {
        InferCtx { type_bindings: Vec::new(), row_bindings: Vec::new() }
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

    /// Fully substitute a type (for final reporting/side tables).
    pub fn apply_type(&self, t: &Type) -> Type {
        match self.resolve_type_shallow(t) {
            Type::List(inner) => Type::List(Box::new(self.apply_type(&inner))),
            Type::Option(inner) => Type::Option(Box::new(self.apply_type(&inner))),
            Type::Result(o, e) => {
                Type::Result(Box::new(self.apply_type(&o)), Box::new(self.apply_type(&e)))
            }
            Type::Record(id, args) => {
                Type::Record(id, args.iter().map(|a| self.apply_type(a)).collect())
            }
            Type::Sum(id, args) => {
                Type::Sum(id, args.iter().map(|a| self.apply_type(a)).collect())
            }
            Type::Fn { params, ret, row } => Type::Fn {
                params: params.iter().map(|p| self.apply_type(p)).collect(),
                ret: Box::new(self.apply_type(&ret)),
                row: self.apply_row(&row),
            },
            Type::Secret(inner) => Type::Secret(Box::new(self.apply_type(&inner))),
            Type::Plugin(inner) => Type::Plugin(Box::new(self.apply_type(&inner))),
            other => other,
        }
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
            (Type::List(x), Type::List(y)) => self.unify_type(x, y),
            (Type::Option(x), Type::Option(y)) => self.unify_type(x, y),
            (Type::Result(x1, x2), Type::Result(y1, y2)) => {
                self.unify_type(x1, y1)?;
                self.unify_type(x2, y2)
            }
            (Type::Secret(x), Type::Secret(y)) => self.unify_type(x, y),
            (Type::ForeignPtr, Type::ForeignPtr) | (Type::PyObj, Type::PyObj) => Ok(()),
            (Type::Foreign(x), Type::Foreign(y)) if x == y => Ok(()),
            // `Plugin[C]` unifies structurally, so the class marker inside is an ordinary
            // inference position: `let p: Plugin[Contained] = load(…)` pins the `C` that `load`
            // returned as a fresh variable (deviation 4). `Verified`/`Contained` are nominal and
            // NEVER unify with each other — the class is exact, never coerced (invariant 29).
            (Type::Plugin(x), Type::Plugin(y)) => self.unify_type(x, y),
            (Type::Verified, Type::Verified) | (Type::Contained, Type::Contained) => Ok(()),
            (Type::Cap(x), Type::Cap(y)) if x == y => Ok(()),
            (Type::Record(id1, a1), Type::Record(id2, a2))
            | (Type::Sum(id1, a1), Type::Sum(id2, a2))
                if id1 == id2 && a1.len() == a2.len() =>
            {
                for (x, y) in a1.iter().zip(a2) {
                    self.unify_type(x, y)?;
                }
                Ok(())
            }
            (
                Type::Fn { params: p1, ret: r1, row: row1 },
                Type::Fn { params: p2, ret: r2, row: row2 },
            ) if p1.len() == p2.len() => {
                for (x, y) in p1.iter().zip(p2) {
                    self.unify_type(x, y)?;
                }
                self.unify_type(r1, r2)?;
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
        match self.resolve_type_shallow(t) {
            Type::Var(TypeVar(w)) => w == v,
            Type::List(inner) | Type::Option(inner) | Type::Secret(inner) | Type::Plugin(inner) => {
                self.occurs_type(v, &inner)
            }
            Type::Result(a, b) => self.occurs_type(v, &a) || self.occurs_type(v, &b),
            Type::Record(_, args) | Type::Sum(_, args) => args.iter().any(|a| self.occurs_type(v, a)),
            Type::Fn { params, ret, .. } => {
                params.iter().any(|p| self.occurs_type(v, p)) || self.occurs_type(v, &ret)
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
        assert_eq!(
            cx.unify_type(&Type::Cap(ResourceKind::FsRead), &Type::Cap(ResourceKind::FsWrite)),
            Err(UnifyError::Type)
        );
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
