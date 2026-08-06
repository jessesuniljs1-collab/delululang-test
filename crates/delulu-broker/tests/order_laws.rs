//! P17 — the order-theoretic laws of the `⊑` attenuation relation, checked EXHAUSTIVELY over a
//! small path universe rather than on hand-picked pairs.
//!
//! `authority.rs` calls itself "the mathematical heart of custody" and its unit tests check
//! commutativity of `⊓` on exactly ONE pair under a comment reading "Property spot-check". A
//! spot-check cannot distinguish "this law holds" from "this law holds for the pair I thought of".
//! This file enumerates every subset of a universe that deliberately contains DISTINCT SPELLINGS
//! OF THE SAME RESOLVED PATH — the case a hand-written pair is least likely to contain, because a
//! human writing a test writes `./data` twice, not `./data` and `data`.
//!
//! Run: cargo test -p delulu-broker --test order_laws -- --nocapture

use std::collections::BTreeSet;

use delulu_broker::authority::{attenuation_check, Authority, Scopes};

/// Spellings chosen so that several denote the SAME resolved path:
///   `./data`, `data`, `./data/`, `.\data`  all resolve to [Name("data")]
const UNIVERSE: &[&str] =
    &["./data", "data", "./data/", r".\data", "./data/sub", "./other", "./data/../secret"];

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// An authority carrying only an fs_read scope — the dimension whose order is non-trivial.
///
/// **Struct-literal construction, deliberately.** `Authority::new` canonicalizes; building the
/// struct directly bypasses that, which is what lets the tests below still *observe* the raw-
/// spelling preorder instead of merely asserting it was once true.
fn auth(paths: &BTreeSet<String>) -> Authority {
    Authority { effects: BTreeSet::new(), scopes: Scopes { fs_read: paths.clone(), ..Default::default() } }
}

/// The canonical representative of a path set — what the broker actually stores (P17-F1/F2/F3).
fn canon(paths: &BTreeSet<String>) -> BTreeSet<String> {
    auth(paths).canonicalized().scopes.fs_read
}

fn leq(a: &BTreeSet<String>, b: &BTreeSet<String>) -> bool {
    attenuation_check(&auth(a), &auth(b)).is_ok()
}

fn meet(a: &BTreeSet<String>, b: &BTreeSet<String>) -> BTreeSet<String> {
    auth(a).intersect(&auth(b)).scopes.fs_read
}

fn show(s: &BTreeSet<String>) -> String {
    format!("{{{}}}", s.iter().cloned().collect::<Vec<_>>().join(", "))
}

/// Every subset of UNIVERSE with at most `max` elements.
fn subsets(max: usize) -> Vec<BTreeSet<String>> {
    let n = UNIVERSE.len();
    let mut out = Vec::new();
    for mask in 0u32..(1 << n) {
        if (mask.count_ones() as usize) > max {
            continue;
        }
        let mut s = BTreeSet::new();
        for (i, p) in UNIVERSE.iter().enumerate() {
            if mask & (1 << i) != 0 {
                s.insert(p.to_string());
            }
        }
        out.push(s);
    }
    out
}

// ---------------------------------------------------------------------------
// Laws that MUST hold. These are the security-relevant ones.
// ---------------------------------------------------------------------------

/// `path::is_descendant_or_equal` is private, so the element relation is exercised through the
/// public gate the security decision actually goes through: `attenuation_check` on singletons.
fn elem_leq(x: &str, y: &str) -> bool {
    leq(&set(&[x]), &set(&[y]))
}

#[test]
fn the_element_relation_is_a_preorder() {
    for x in UNIVERSE {
        assert!(elem_leq(x, x), "reflexivity failed at {x:?}");
    }
    for x in UNIVERSE {
        for y in UNIVERSE {
            for z in UNIVERSE {
                if elem_leq(x, y) && elem_leq(y, z) {
                    assert!(
                        elem_leq(x, z),
                        "transitivity failed: {x:?} <= {y:?} <= {z:?} but not {x:?} <= {z:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_set_order_is_reflexive_and_transitive() {
    let sets = subsets(3);
    for a in &sets {
        assert!(leq(a, a), "reflexivity failed at {}", show(a));
    }
    for a in &sets {
        for b in &sets {
            if !leq(a, b) {
                continue;
            }
            for c in &sets {
                if leq(b, c) {
                    assert!(
                        leq(a, c),
                        "transitivity failed: {} <= {} <= {} but not <= the last",
                        show(a),
                        show(b),
                        show(c)
                    );
                }
            }
        }
    }
}

/// THE load-bearing law: a computed repair may never be wider than either input.
/// If this ever fails, DL0802 hands back an ESCALATION. Everything else in this file is
/// about naming and determinism; this one is about authority.
#[test]
fn the_meet_never_widens_and_is_the_greatest_lower_bound() {
    let sets = subsets(3);
    for a in &sets {
        for b in &sets {
            let m = meet(a, b);
            assert!(leq(&m, a) && leq(&m, b), "meet {} of {} and {} WIDENS", show(&m), show(a), show(b));
        }
    }
    // Greatest: nothing below both a and b escapes the meet.
    for a in &sets {
        for b in &sets {
            let m = meet(a, b);
            for c in &sets {
                if leq(c, a) && leq(c, b) {
                    assert!(
                        leq(c, &m),
                        "meet is not the GLB: {} <= {} and <= {}, but not <= meet {}",
                        show(c),
                        show(a),
                        show(b),
                        show(&m)
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Laws the project's comments CLAIM. F1–F3 were found FAILING here; each is now closed by
// canonicalization, and each keeps a companion test that still *observes* the original
// counterexamples on raw spellings, so the finding cannot quietly become folklore.
//
// The mathematics that closed them: `⊑` is defined through `path::resolve`, which is not
// injective — `./data`, `data`, `./data/` and `.\data` are one path under four names. A relation
// defined through a non-injective function is a PREORDER on its domain, never a partial order, no
// matter how the comparison is written. So F1 was never a bug in the comparison; it was the word
// "lattice" applied to the representation type when it belongs to the quotient. Choosing one
// representative per class (`path::canonicalize`) makes the quotient and the representation
// coincide, and all three findings dissolve together.
// ---------------------------------------------------------------------------

/// **P17-F2, CLOSED.** `authority.rs` documents `⊓` as symmetric: "note `⊓` is symmetric so the
/// value is identical either way". It was not: when `x` and `y` were two spellings of one path,
/// both descendant tests held, so `intersect_path_sets`' `if`/`else if` kept whichever side the
/// outer loop happened to hold — 414 counterexamples. The meet now emits canonical representatives,
/// so the two argument orders agree by construction.
#[test]
fn the_meet_is_symmetric_as_the_comment_claims() {
    let sets = subsets(2);
    let mut counterexamples = Vec::new();
    for a in &sets {
        for b in &sets {
            let (ab, ba) = (meet(a, b), meet(b, a));
            if ab != ba {
                counterexamples.push(format!(
                    "  A={} B={}  ->  A^B={}  B^A={}",
                    show(a),
                    show(b),
                    show(&ab),
                    show(&ba)
                ));
            }
        }
    }
    assert!(
        counterexamples.is_empty(),
        "`⊓` is NOT symmetric — {} counterexample(s):\n{}",
        counterexamples.len(),
        counterexamples.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}

/// **P17-F1, CLOSED.** A partial order requires antisymmetry; the module calls this structure a
/// "lattice", which presupposes one.
///
/// The theorem that actually holds, and the one the word "lattice" needs: **`A ⊑ B` and `B ⊑ A`
/// imply `canon(A) = canon(B)`** — antisymmetry on canonical representatives, i.e. on the quotient
/// by `⊑`-equivalence. Proof: take `x ∈ canon(A)`. From `A ⊑ B` there is `y ∈ canon(B)` with
/// `x ⊑ y`; from `B ⊑ A` there is `z ∈ canon(A)` with `y ⊑ z`. Then `x ⊑ z` with both in the
/// antichain `canon(A)`, so `x = z`, hence `x ⊑ y ⊑ x`, so `y` and `x` have equal resolved segments
/// and — both being canonical — equal spellings. So `x ∈ canon(B)`, and symmetrically. ∎
#[test]
fn the_order_is_antisymmetric_on_canonical_representatives() {
    let sets = subsets(2);
    let mut counterexamples = Vec::new();
    for a in &sets {
        for b in &sets {
            if leq(a, b) && leq(b, a) && canon(a) != canon(b) {
                counterexamples.push(format!(
                    "  A={} B={}  ->  canon(A)={} canon(B)={}",
                    show(a),
                    show(b),
                    show(&canon(a)),
                    show(&canon(b))
                ));
            }
        }
    }
    assert!(
        counterexamples.is_empty(),
        "`⊑` is NOT antisymmetric even on canonical representatives — {} counterexample(s):\n{}",
        counterexamples.len(),
        counterexamples.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}

/// **The finding itself, kept alive.** F1 was not a bug in the comparison and no amount of care
/// inside `attenuation_check` could have fixed it: `⊑` is defined through `path::resolve`, which is
/// not injective, so on raw spellings it is a preorder as a matter of mathematics. This test
/// asserts the counterexamples are STILL THERE on raw input, so nobody can later conclude that raw
/// path sets are safe to compare by equality — the reason canonicalization is load-bearing rather
/// than cosmetic.
#[test]
fn raw_spellings_remain_a_preorder_which_is_why_canonicalization_is_required() {
    let sets = subsets(2);
    let mut equivalent_but_unequal = 0usize;
    for a in &sets {
        for b in &sets {
            if a != b && leq(a, b) && leq(b, a) {
                equivalent_but_unequal += 1;
            }
        }
    }
    assert!(
        equivalent_but_unequal > 0,
        "no ⊑-equivalent-but-unequal raw pairs remain — if `resolve` became injective, or the \
         universe lost its distinct spellings, this test has stopped guarding anything and the \
         canonicalization rationale must be re-derived rather than assumed"
    );
    // Both sources of non-injectivity are represented: spelling, and set redundancy.
    assert!(leq(&set(&["./data"]), &set(&["data"])) && leq(&set(&["data"]), &set(&["./data"])));
    let redundant = set(&["./data", "./data/sub"]);
    let reduced = set(&["./data"]);
    assert!(
        leq(&redundant, &reduced) && leq(&reduced, &redundant) && redundant != reduced,
        "set redundancy must still be an equivalence the antichain reduction has to collapse"
    );
}

/// **P17-F3, CLOSED.** Canonical JSON feeds the audit hash chain and certificate signatures. If two
/// `⊑`-equivalent authorities serialize differently, the same logical grant carries two different
/// hashes — so a revocation keyed to one hash misses the other.
#[test]
fn equivalent_authorities_serialize_identically() {
    let sets = subsets(2);
    let mut counterexamples = Vec::new();
    for a in &sets {
        for b in &sets {
            if leq(a, b) && leq(b, a) {
                let (ja, jb) = (
                    auth(a).canonicalized().to_json().to_string(),
                    auth(b).canonicalized().to_json().to_string(),
                );
                if ja != jb {
                    counterexamples.push(format!("  {} -> {}\n  {} -> {}", show(a), ja, show(b), jb));
                }
            }
        }
    }
    assert!(
        counterexamples.is_empty(),
        "⊑-equivalent authorities produce DIFFERENT canonical JSON — {} counterexample(s):\n{}",
        counterexamples.len(),
        counterexamples.iter().take(4).cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Canonicalization is only allowed to remove the *choice of spelling*. If it ever changed which
/// paths an authority covers, every grant in the tree would silently change meaning the moment it
/// was stored — so this checks the containment decision is identical before and after, in both
/// directions, across every ordered pair the universe can build.
#[test]
fn canonicalizing_an_authority_changes_no_containment_decision() {
    let sets = subsets(3);
    for a in &sets {
        for b in &sets {
            assert_eq!(
                leq(a, b),
                attenuation_check(&auth(a).canonicalized(), &auth(b).canonicalized()).is_ok(),
                "canonicalization changed whether {} ⊑ {}",
                show(a),
                show(b)
            );
            // And mixed: canonical child against raw parent, the shape the broker actually sees
            // while a delegation is being checked against a grant issued earlier.
            assert_eq!(
                leq(a, b),
                attenuation_check(&auth(a).canonicalized(), &auth(b)).is_ok(),
                "canonicalizing only the child changed whether {} ⊑ {}",
                show(a),
                show(b)
            );
        }
    }
}

/// The invariant the broker's ingest gate relies on: whatever spelling arrives, what gets stored is
/// a fixed point. Without this, "the tree holds canonical authorities" would be a hope.
#[test]
fn canonicalization_is_idempotent_and_recognized_by_is_canonical() {
    for s in subsets(3) {
        let once = auth(&s).canonicalized();
        assert!(once.is_canonical(), "canonicalized authority not recognized as canonical: {s:?}");
        assert_eq!(once, once.canonicalized(), "canonicalization is not idempotent at {s:?}");
    }
}
