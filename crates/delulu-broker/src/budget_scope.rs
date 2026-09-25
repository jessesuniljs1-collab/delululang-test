//! PS-B-05: the resource-budget dimension of authority.
//!
//! D-V2-08 (owner's direction): a budget joins the `⊑` order **only** with a containment relation and
//! a meet that are proved in the Z3 model (`docs/design/models/authority_algebra.py`), checked by
//! exhaustive enumeration (the tests below), and documented in `docs/MATHEMATICS.md` §"The budget
//! dimension". Memory and processor time qualify: each is a chain under `≤`, so the pair is a product
//! of chains — a lattice — and the laws are the textbook ones. That is why these two join the order
//! and a wall-clock budget does not yet: a wall budget is a launcher control the operator sets per
//! run (PS-B-01), not something a delegator hands down.
//!
//! **What `None` means.** A grant that carries no budget is the TOP of this dimension — every node
//! written before PS-B-05, and any grant whose delegator named none. Under a BUDGETED parent an
//! unbudgeted child is therefore a WIDENING and is refused, the same asymmetry as an unbounded rate
//! under a bounded parent in [`crate::device_scope`]. A run is never unlimited all the same: an
//! unbudgeted grant runs under the operator's D-V2-25 defaults.

/// A delegated ceiling on what a holder's runs may consume.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BudgetScope {
    pub memory_bytes: u64,
    pub cpu_seconds: u64,
}

impl BudgetScope {
    /// The bottom of the dimension: one byte and one second. What an unreadable budget becomes on
    /// the way in (`brokerd::spec_to_authority`), because on this dimension absence is the TOP, so
    /// "drop what cannot be read" would fail OPEN.
    pub const SMALLEST: BudgetScope = BudgetScope { memory_bytes: 1, cpu_seconds: 1 };

    /// The canonical grant form, and the wire contract between crates exactly as the device grant
    /// string is: `mem=BYTES,cpu=SECONDS`, both named, both positive, nothing else, in that order.
    pub fn to_grant_string(&self) -> String {
        format!("mem={},cpu={}", self.memory_bytes, self.cpu_seconds)
    }

    /// Parse the grant form. Refused rather than guessed: a missing dimension, an extra one, a
    /// repeated one, a zero, or anything but a plain decimal. A budget an old reader half-understood
    /// would be a budget it half-enforced.
    pub fn parse(s: &str) -> Result<BudgetScope, String> {
        let (mut mem, mut cpu) = (None, None);
        for part in s.split(',') {
            let (k, v) = part
                .split_once('=')
                .ok_or_else(|| format!("budget `{s}`: `{part}` is not KEY=VALUE (use mem=BYTES,cpu=SECONDS)"))?;
            if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) || (v.len() > 1 && v.starts_with('0')) {
                return Err(format!("budget `{s}`: `{v}` is not a positive decimal"));
            }
            let n: u64 = v.parse().map_err(|_| format!("budget `{s}`: `{v}` is out of range"))?;
            if n == 0 {
                return Err(format!("budget `{s}`: a zero budget is refused — it means either \"nothing\" or \"unlimited\""));
            }
            let slot = match k {
                "mem" => &mut mem,
                "cpu" => &mut cpu,
                other => return Err(format!("budget `{s}`: `{other}` is not a budget dimension (mem, cpu)")),
            };
            if slot.replace(n).is_some() {
                return Err(format!("budget `{s}`: `{k}` is named twice"));
            }
        }
        match (mem, cpu) {
            (Some(memory_bytes), Some(cpu_seconds)) => Ok(BudgetScope { memory_bytes, cpu_seconds }),
            _ => Err(format!("budget `{s}`: both `mem` and `cpu` must be named")),
        }
    }
}

/// `child ⊑ parent` on this dimension. `None` is the top.
pub fn within(child: Option<&BudgetScope>, parent: Option<&BudgetScope>) -> bool {
    match (child, parent) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(c), Some(p)) => c.memory_bytes <= p.memory_bytes && c.cpu_seconds <= p.cpu_seconds,
    }
}

/// The meet `⊓`: the tighter of the two, per dimension. `None` is the identity — meeting with the
/// top changes nothing.
pub fn meet(a: Option<&BudgetScope>, b: Option<&BudgetScope>) -> Option<BudgetScope> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) | (None, Some(x)) => Some(*x),
        (Some(x), Some(y)) => Some(BudgetScope {
            memory_bytes: x.memory_bytes.min(y.memory_bytes),
            cpu_seconds: x.cpu_seconds.min(y.cpu_seconds),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole domain the laws are checked over: the top, and every budget with each dimension in
    /// 1..=3 — nine points of a product of two chains, plus the top.
    fn domain() -> Vec<Option<BudgetScope>> {
        let mut out = vec![None];
        for m in 1..=3 {
            for c in 1..=3 {
                out.push(Some(BudgetScope { memory_bytes: m, cpu_seconds: c }));
            }
        }
        out
    }

    fn le(a: &Option<BudgetScope>, b: &Option<BudgetScope>) -> bool {
        within(a.as_ref(), b.as_ref())
    }

    fn mt(a: &Option<BudgetScope>, b: &Option<BudgetScope>) -> Option<BudgetScope> {
        meet(a.as_ref(), b.as_ref())
    }

    #[test]
    fn the_order_is_a_partial_order_over_the_whole_domain() {
        let d = domain();
        for a in &d {
            assert!(le(a, a), "reflexive: {a:?}");
            for b in &d {
                if le(a, b) && le(b, a) {
                    assert_eq!(a, b, "antisymmetric");
                }
                for c in &d {
                    if le(a, b) && le(b, c) {
                        assert!(le(a, c), "transitive: {a:?} {b:?} {c:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_meet_is_the_greatest_lower_bound_and_never_widens() {
        let d = domain();
        for a in &d {
            assert_eq!(mt(a, a), *a, "idempotent");
            for b in &d {
                let m = mt(a, b);
                assert!(le(&m, a) && le(&m, b), "a lower bound — the no-widening law: {a:?} ⊓ {b:?} = {m:?}");
                assert_eq!(m, mt(b, a), "commutative");
                for c in &d {
                    if le(c, a) && le(c, b) {
                        assert!(le(c, &m), "greatest: {c:?} under {a:?} and {b:?} but not under {m:?}");
                    }
                    assert_eq!(mt(&mt(a, b), c), mt(a, &mt(b, c)), "associative");
                }
            }
        }
    }

    /// The asymmetry that is easy to get backwards: no budget under a budget is a widening.
    #[test]
    fn an_unbudgeted_child_under_a_budgeted_parent_is_a_widening() {
        let p = BudgetScope { memory_bytes: 1 << 30, cpu_seconds: 300 };
        assert!(!within(None, Some(&p)));
        assert!(within(Some(&p), None));
        let bigger = BudgetScope { memory_bytes: 2 << 30, cpu_seconds: 300 };
        assert!(!within(Some(&bigger), Some(&p)), "more memory than delegated");
        let slower = BudgetScope { memory_bytes: 1 << 30, cpu_seconds: 301 };
        assert!(!within(Some(&slower), Some(&p)), "more processor time than delegated");
    }

    #[test]
    fn the_grant_string_round_trips_and_refuses_every_other_spelling() {
        let b = BudgetScope { memory_bytes: 268_435_456, cpu_seconds: 60 };
        assert_eq!(b.to_grant_string(), "mem=268435456,cpu=60");
        assert_eq!(BudgetScope::parse(&b.to_grant_string()), Ok(b));
        assert_eq!(BudgetScope::parse("cpu=60,mem=268435456"), Ok(b), "order does not matter on input");
        for bad in [
            "", "mem=1", "cpu=1", "mem=0,cpu=1", "mem=1,cpu=0", "mem=1,cpu=1,wall=5", "mem=1,mem=2,cpu=1",
            "mem=01,cpu=1", "mem=+1,cpu=1", "mem= 1,cpu=1", "mem=1,cpu=1,", "mem=1;cpu=1", "MEM=1,cpu=1",
            "mem=99999999999999999999,cpu=1",
        ] {
            assert!(BudgetScope::parse(bad).is_err(), "`{bad}` must be refused");
        }
    }
}
