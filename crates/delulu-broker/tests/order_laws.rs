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
fn auth(paths: &BTreeSet<String>) -> Authority {
    Authority { effects: BTreeSet::new(), scopes: Scopes { fs_read: paths.clone(), ..Default::default() } }
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
// Laws the project's comments CLAIM. These are the ones under test.
// ---------------------------------------------------------------------------

/// `authority.rs` documents `⊓` as symmetric: "note `⊓` is symmetric so the value is identical
/// either way". The existing unit test `meet_is_symmetric` checks one pair.
#[test]
#[ignore = "P17-F2 OPEN: `⊓` is not symmetric on representations (414 counterexamples). \
            Run with `--ignored`. Fix is format-affecting; see docs/design/PROOF_CAMPAIGN.md."]
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

/// A partial order requires antisymmetry. The module calls this structure a "lattice", which
/// presupposes one.
#[test]
#[ignore = "P17-F1 OPEN: `⊑` is a preorder, not a partial order (206 counterexamples). \
            Run with `--ignored`. See docs/design/PROOF_CAMPAIGN.md."]
fn the_order_is_antisymmetric_as_the_word_lattice_presupposes() {
    let sets = subsets(2);
    let mut counterexamples = Vec::new();
    for a in &sets {
        for b in &sets {
            if a != b && leq(a, b) && leq(b, a) {
                counterexamples.push(format!("  A={} B={}", show(a), show(b)));
            }
        }
    }
    assert!(
        counterexamples.is_empty(),
        "`⊑` is NOT antisymmetric (it is a PREORDER, not a partial order) — {} counterexample(s):\n{}",
        counterexamples.len(),
        counterexamples.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Canonical JSON feeds the audit hash chain and certificate signatures. If two `⊑`-equivalent
/// authorities serialize differently, the same logical grant has two different hashes.
#[test]
#[ignore = "P17-F3 OPEN: ⊑-equivalent authorities hash differently in the audit chain \
            (206 counterexamples). Run with `--ignored`. See docs/design/PROOF_CAMPAIGN.md."]
fn equivalent_authorities_serialize_identically() {
    let sets = subsets(2);
    let mut counterexamples = Vec::new();
    for a in &sets {
        for b in &sets {
            if a != b && leq(a, b) && leq(b, a) {
                let (ja, jb) = (auth(a).to_json().to_string(), auth(b).to_json().to_string());
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
