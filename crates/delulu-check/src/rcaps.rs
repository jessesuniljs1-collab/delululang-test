//! Reference-capability core (Stage 7, spec §3) — Pony's production-proven system, adopted
//! faithfully, never redesigned (playbook trap 2).
//!
//! **The deny-property definitions are normative; the tables are their consequences.** This
//! module encodes BOTH: [`denies_local`]/[`denies_global`] are the definitions, and
//! [`alias`]/[`sendable`]/[`viewpoint`] are the tables. The unit tests here check every table
//! cell against the definitions, and the phase-7j property gate (acceptance criterion 8)
//! re-validates generated alias/adaptation sequences against them — a counterexample blocks
//! the stage.
//!
//! Rcaps are a SECOND checking axis, orthogonal to effect rows (playbook trap 1): this module
//! never touches `Row`, `unify`, or `Type` internals — the rcap of a value travels beside its
//! type (build-order deviation 4), so a `val` closure can still be `!{Write}`.

use delulu_syntax::ast::Rcap;

use crate::ty::{Type, TypeDefId};

/// What an rcap **denies to aliases** — the normative definitions (spec §3). `local` is what
/// other aliases *in the same actor* may not do; `global` is what aliases *in other actors*
/// may not do. Everything else is permitted; the tables below must be consistent with exactly
/// this and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deny {
    pub read: bool,
    pub write: bool,
}

impl Deny {
    pub const NONE: Deny = Deny { read: false, write: false };
    pub const WRITE: Deny = Deny { read: false, write: true };
    pub const READ_WRITE: Deny = Deny { read: true, write: true };
}

/// What other aliases in the SAME actor may not do while this rcap exists.
pub fn denies_local(k: Rcap) -> Deny {
    match k {
        Rcap::Iso => Deny::READ_WRITE, // unique: no other local reader or writer
        Rcap::Trn => Deny::WRITE,      // sole writer; local `box` readers allowed
        Rcap::Ref => Deny::NONE,       // ordinary shared mutability within one actor
        Rcap::Val => Deny::WRITE,      // immutable: nobody writes, everybody may read
        Rcap::Box_ => Deny::NONE,      // read-only view; local writers may exist elsewhere
        Rcap::Tag => Deny::NONE,       // opaque identity: guarantees nothing to others
    }
}

/// What aliases in OTHER actors may not do while this rcap exists.
pub fn denies_global(k: Rcap) -> Deny {
    match k {
        Rcap::Iso => Deny::READ_WRITE,
        Rcap::Trn => Deny::READ_WRITE,
        Rcap::Ref => Deny::READ_WRITE,
        Rcap::Val => Deny::WRITE, // deeply immutable forever — safely readable anywhere
        Rcap::Box_ => Deny::WRITE,
        Rcap::Tag => Deny::NONE,
    }
}

/// May the HOLDER read through this rcap? (Everything but `tag` — tag is identity/send only.)
pub fn can_read(k: Rcap) -> bool {
    !matches!(k, Rcap::Tag)
}

/// May the HOLDER write through this rcap? (spec §3 write rule: receiver ∈ {iso, trn, ref}.)
pub fn can_write(k: Rcap) -> bool {
    matches!(k, Rcap::Iso | Rcap::Trn | Rcap::Ref)
}

/// The alias table (spec §3): the rcap an alias of `x: κ` gets **without** `consume`.
/// Consequence of the deny definitions: the alias is the most permissive rcap whose
/// permissions do not violate what κ denies locally.
pub fn alias(k: Rcap) -> Rcap {
    match k {
        Rcap::Iso => Rcap::Tag,  // iso denies all other local R/W → the alias may do neither
        Rcap::Trn => Rcap::Box_, // trn denies other local writes → the alias may only read
        Rcap::Ref => Rcap::Ref,
        Rcap::Val => Rcap::Val,
        Rcap::Box_ => Rcap::Box_,
        Rcap::Tag => Rcap::Tag,
    }
}

/// Sendability (spec §3; the whole game — playbook §0): a value may cross an actor boundary
/// iff its rcap guarantees the same thing to BOTH sides — the Pony characterization
/// `denies_local(κ) == denies_global(κ)`, which yields exactly {iso, val, tag}.
pub fn sendable(k: Rcap) -> bool {
    matches!(k, Rcap::Iso | Rcap::Val | Rcap::Tag)
}

/// The default rcap when a type position writes none (spec §2 "Default rcaps when omitted" —
/// normative ergonomics; playbook 7b calls it load-bearing and easy to get subtly wrong).
///
/// Returns `None` when the type is UNDETERMINED (`Var` anywhere it matters): a default must
/// never be guessed for a type inference hasn't pinned — the kitchen rule's skip branch. Use
/// sites treat `None` fail-closed (like DL1509 taught us).
///
/// `components_of` resolves a user type id to its component types (record: field types; sum:
/// all variant field types), with the caller applying any generic substitution first.
/// Unknown id ⇒ `None` ⇒ undetermined ⇒ fail-closed.
pub fn default_rcap(
    t: &Type,
    components_of: &dyn Fn(TypeDefId) -> Option<Vec<Type>>,
) -> Option<Rcap> {
    let mut visiting = Vec::new();
    default_rcap_inner(t, components_of, &mut visiting)
}

fn default_rcap_inner(
    t: &Type,
    components_of: &dyn Fn(TypeDefId) -> Option<Vec<Type>>,
    visiting: &mut Vec<TypeDefId>,
) -> Option<Rcap> {
    Some(match t {
        // Plain immutable data.
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::Unit => Rcap::Val,
        // Unforgeable handles: immutable, safely shareable (invariant 36). Foreign lib
        // handles are `val` per spec §7.
        Type::Cap(_) | Type::Secret(_) | Type::Plugin(_) | Type::Root | Type::Foreign(_) => Rcap::Val,
        // The class markers only appear as Plugin's argument; harmless val.
        Type::Verified | Type::Contained => Rcap::Val,
        // Pinned to their creating actor (invariant 36: CPython affinity; a raw C pointer has
        // no cross-actor story either).
        Type::PyObj | Type::ForeignPtr => Rcap::Ref,
        // Composites of only val-defaulting components are deeply immutable → val;
        // anything else → ref (spec §2).
        Type::List(e) | Type::Option(e) => {
            match default_rcap_inner(e, components_of, visiting)? {
                Rcap::Val => Rcap::Val,
                _ => Rcap::Ref,
            }
        }
        Type::Result(o, e) => {
            let a = default_rcap_inner(o, components_of, visiting)?;
            let b = default_rcap_inner(e, components_of, visiting)?;
            if a == Rcap::Val && b == Rcap::Val { Rcap::Val } else { Rcap::Ref }
        }
        Type::Record(id, _) | Type::Sum(id, _) => {
            // Sums mirror records here (build-order deviation 8): the spec's list names
            // "records of only such types", but a sum's variant fields have NO write surface
            // at all in this runtime (`Value::Variant` holds an immutable `Rc<Vec<_>>`), so a
            // sum of only-val components is deeply immutable by mechanism, exactly like a
            // record of them.
            if visiting.contains(id) {
                // Coinductive: a cycle of otherwise-val components is deeply immutable
                // (assume val at the loop; any non-val field elsewhere still wins).
                Rcap::Val
            } else {
                visiting.push(*id);
                let comps = components_of(*id);
                let out = match comps {
                    None => None, // unknown id — cannot tell — fail closed
                    Some(cs) => {
                        let mut all_val = true;
                        let mut undetermined = false;
                        for c in &cs {
                            match default_rcap_inner(c, components_of, visiting) {
                                Some(Rcap::Val) => {}
                                Some(_) => all_val = false,
                                None => undetermined = true,
                            }
                        }
                        if undetermined {
                            None
                        } else if all_val {
                            Some(Rcap::Val)
                        } else {
                            Some(Rcap::Ref)
                        }
                    }
                };
                visiting.pop();
                return out;
            }
        }
        // Closures default `ref`; a lambda whose captures are all val/tag INFERS val at its
        // creation site (spec §3) — that is the checker's job at the site, not a default.
        Type::Fn { .. } => Rcap::Ref,
        // Undetermined: never guess (kitchen rule).
        Type::Var(_) => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::Row;

    const ALL: [Rcap; 6] = [Rcap::Iso, Rcap::Trn, Rcap::Ref, Rcap::Val, Rcap::Box_, Rcap::Tag];

    // ----- the alias table, cell for cell (spec §3) -------------------------

    #[test]
    fn alias_table_cell_for_cell() {
        assert_eq!(alias(Rcap::Iso), Rcap::Tag);
        assert_eq!(alias(Rcap::Trn), Rcap::Box_);
        assert_eq!(alias(Rcap::Ref), Rcap::Ref);
        assert_eq!(alias(Rcap::Val), Rcap::Val);
        assert_eq!(alias(Rcap::Box_), Rcap::Box_);
        assert_eq!(alias(Rcap::Tag), Rcap::Tag);
    }

    #[test]
    fn sendable_is_exactly_iso_val_tag() {
        assert!(sendable(Rcap::Iso));
        assert!(sendable(Rcap::Val));
        assert!(sendable(Rcap::Tag));
        assert!(!sendable(Rcap::Trn));
        assert!(!sendable(Rcap::Ref));
        assert!(!sendable(Rcap::Box_));
    }

    // ----- the tables against the deny DEFINITIONS (the normative side) -----

    #[test]
    fn sendable_is_the_deny_characterization() {
        // κ is sendable ⟺ it guarantees the same thing locally and globally.
        for k in ALL {
            assert_eq!(
                sendable(k),
                denies_local(k) == denies_global(k),
                "sendable({}) disagrees with the deny definitions",
                k.name()
            );
        }
    }

    #[test]
    fn an_alias_never_violates_what_the_original_denies_locally() {
        for k in ALL {
            let a = alias(k);
            if can_read(a) {
                assert!(!denies_local(k).read, "alias({0})={1} may read but {0} denies local reads", k.name(), a.name());
            }
            if can_write(a) {
                assert!(!denies_local(k).write, "alias({0})={1} may write but {0} denies local writes", k.name(), a.name());
            }
        }
    }

    /// The full legality model for "κ' may be an unconsumed alias of κ", derived from the
    /// deny definitions. Three clauses, each load-bearing (dropping any one admits a wrong
    /// cell — this test suite found that empirically):
    /// (a) the alias's permissions don't violate what the origin denies locally;
    /// (b) the origin's permissions don't violate what the alias denies locally (the alias's
    ///     rcap is a guarantee too — the origin is one of the "others" it speaks about);
    /// (c) the alias never denies MORE than the origin, locally or globally — the origin's
    ///     world may already contain aliases the origin permitted, and the new alias cannot
    ///     retroactively forbid them (this is what makes iso-of-box and ref-of-box illegal).
    fn legal_alias(k: Rcap, a: Rcap) -> bool {
        let subset = |x: Deny, y: Deny| (!x.read || y.read) && (!x.write || y.write);
        let a_ok = (!can_read(a) || !denies_local(k).read) && (!can_write(a) || !denies_local(k).write);
        let b_ok = (!can_read(k) || !denies_local(a).read) && (!can_write(k) || !denies_local(a).write);
        let c_ok = subset(denies_local(a), denies_local(k)) && subset(denies_global(a), denies_global(k));
        a_ok && b_ok && c_ok
    }

    #[test]
    fn an_alias_is_the_most_permissive_legal_rcap() {
        // The table cell must itself be legal, and NO strictly-more-permissive rcap may be —
        // this is what makes the table THE consequence of the definitions rather than one
        // safe choice among many. (Permissiveness = the (read, write) permission pair.)
        for k in ALL {
            let a = alias(k);
            assert!(legal_alias(k, a), "alias({0})={1} must itself be legal", k.name(), a.name());
            for candidate in ALL {
                let strictly_more = can_read(candidate) >= can_read(a)
                    && can_write(candidate) >= can_write(a)
                    && (can_read(candidate) > can_read(a) || can_write(candidate) > can_write(a));
                if strictly_more {
                    assert!(
                        !legal_alias(k, candidate),
                        "{} would be a more permissive legal alias of {} than {}",
                        candidate.name(),
                        k.name(),
                        a.name()
                    );
                }
            }
        }
    }

    #[test]
    fn global_denies_are_at_least_local_denies() {
        // Another actor is never allowed MORE than a local alias (sanity of the definitions).
        for k in ALL {
            let l = denies_local(k);
            let g = denies_global(k);
            assert!(g.read >= l.read && g.write >= l.write, "{}", k.name());
        }
    }

    #[test]
    fn alias_is_idempotent() {
        for k in ALL {
            assert_eq!(alias(alias(k)), alias(k), "{}", k.name());
        }
    }

    #[test]
    fn writers_are_exactly_iso_trn_ref_and_only_tag_cannot_read() {
        for k in ALL {
            assert_eq!(can_write(k), matches!(k, Rcap::Iso | Rcap::Trn | Rcap::Ref), "{}", k.name());
            assert_eq!(can_read(k), !matches!(k, Rcap::Tag), "{}", k.name());
        }
    }

    // ----- the default rule, per type family (spec §2, load-bearing) --------

    fn no_user_types(_: TypeDefId) -> Option<Vec<Type>> {
        None
    }

    fn dr(t: &Type) -> Option<Rcap> {
        default_rcap(t, &no_user_types)
    }

    #[test]
    fn plain_data_and_handles_default_val() {
        for t in [
            Type::Int,
            Type::Float,
            Type::Bool,
            Type::Str,
            Type::Unit,
            Type::Cap(crate::ty::ResourceKind::Console),
            Type::Secret(Box::new(Type::Str)),
            Type::Plugin(Box::new(Type::Verified)),
            Type::Root,
            Type::Foreign("mathlib".into()),
        ] {
            assert_eq!(dr(&t), Some(Rcap::Val), "{t}");
        }
    }

    #[test]
    fn pyobj_and_foreignptr_default_ref_pinned() {
        assert_eq!(dr(&Type::PyObj), Some(Rcap::Ref));
        assert_eq!(dr(&Type::ForeignPtr), Some(Rcap::Ref));
    }

    #[test]
    fn immutable_composites_default_val_others_ref() {
        assert_eq!(dr(&Type::list(Type::Int)), Some(Rcap::Val));
        assert_eq!(dr(&Type::list(Type::list(Type::Str))), Some(Rcap::Val));
        assert_eq!(dr(&Type::Option(Box::new(Type::Int))), Some(Rcap::Val));
        assert_eq!(dr(&Type::result(Type::Int, Type::Str)), Some(Rcap::Val));
        // A list of closures is not deeply immutable data.
        let clo = Type::Fn { params: vec![Type::Int], ret: Box::new(Type::Int), row: Row::pure() };
        assert_eq!(dr(&Type::list(clo.clone())), Some(Rcap::Ref));
        assert_eq!(dr(&Type::result(Type::Int, clo.clone())), Some(Rcap::Ref));
        assert_eq!(dr(&clo), Some(Rcap::Ref));
    }

    #[test]
    fn records_of_only_val_components_default_val() {
        let fields = |id: TypeDefId| -> Option<Vec<Type>> {
            match id.0 {
                0 => Some(vec![Type::Int, Type::Str]), // point-like: all val
                1 => Some(vec![Type::Int, Type::Fn { params: vec![], ret: Box::new(Type::Unit), row: Row::pure() }]),
                _ => None,
            }
        };
        assert_eq!(default_rcap(&Type::Record(TypeDefId(0), vec![]), &fields), Some(Rcap::Val));
        assert_eq!(default_rcap(&Type::Record(TypeDefId(1), vec![]), &fields), Some(Rcap::Ref));
        // Sums mirror records (deviation 8: variant fields have no write surface).
        assert_eq!(default_rcap(&Type::Sum(TypeDefId(0), vec![]), &fields), Some(Rcap::Val));
    }

    #[test]
    fn recursive_records_resolve_coinductively() {
        // type Node { v: Int, next: Option[Node] } — a cycle of val components is val.
        let fields = |id: TypeDefId| -> Option<Vec<Type>> {
            match id.0 {
                7 => Some(vec![Type::Int, Type::Option(Box::new(Type::Record(TypeDefId(7), vec![])))]),
                _ => None,
            }
        };
        assert_eq!(default_rcap(&Type::Record(TypeDefId(7), vec![]), &fields), Some(Rcap::Val));
    }

    // ----- the couldn't-tell cases (kitchen rule: fail closed, never guess) --

    #[test]
    fn undetermined_types_have_no_default() {
        assert_eq!(dr(&Type::Var(crate::ty::TypeVar(0))), None);
        assert_eq!(dr(&Type::list(Type::Var(crate::ty::TypeVar(0)))), None);
        assert_eq!(dr(&Type::Option(Box::new(Type::Var(crate::ty::TypeVar(1))))), None);
    }

    #[test]
    fn unknown_user_types_have_no_default() {
        // components_of can't resolve the id — the checker couldn't tell — no default.
        assert_eq!(dr(&Type::Record(TypeDefId(42), vec![])), None);
        assert_eq!(dr(&Type::Sum(TypeDefId(42), vec![])), None);
    }

    #[test]
    fn a_record_with_an_undetermined_field_has_no_default() {
        let fields = |id: TypeDefId| -> Option<Vec<Type>> {
            match id.0 {
                3 => Some(vec![Type::Int, Type::Var(crate::ty::TypeVar(9))]),
                _ => None,
            }
        };
        assert_eq!(default_rcap(&Type::Record(TypeDefId(3), vec![]), &fields), None);
    }
}
