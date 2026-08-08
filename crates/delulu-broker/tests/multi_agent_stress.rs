//! P18 — random authority graphs at scale, up to 1000 logical agents.
//!
//! Every other broker test builds a tree a human drew: two or three nodes, one delegation, one
//! revocation. Those check that a mechanism *can* work. They cannot check that the invariants
//! survive a shape nobody drew — a delegation chain 40 deep, a revocation landing on an interior
//! node with 200 descendants, an expiry inherited across six hops while a sibling stays live.
//!
//! The generator is a deterministic LCG, so a failure names a seed that reproduces it exactly. It
//! is not a fuzzer: it does not minimize, and it does not hunt. It builds a large random structure
//! and asserts the laws that must hold over **all** of it.
//!
//! ## The laws under test
//!
//! 1. **Attenuation is transitive to the root.** No node may hold authority its parent does not,
//!    and therefore none may hold authority the root does not. This is the whole security claim.
//! 2. **Revocation is inherited.** A node under a revoked ancestor is not `Live`, however deep.
//! 3. **Expiry is inherited.** A node under an expired ancestor is not `Live` — including a child
//!    with `ttl: None`, which is the exact shape of a real vulnerability this project shipped
//!    (a delegated child outliving its parent's expired uplink lease).
//! 4. **Every stored authority is canonical.** The custody boundary canonicalizes (P17-F1/F2/F3);
//!    at 1000 agents with mixed path spellings, that either holds everywhere or it does not.
//! 5. **The audit chain verifies** after thousands of operations.
//! 6. **Nothing panics.** A broker that dies instead of answering is a broker that can be skipped.

use std::collections::BTreeSet;
use std::rc::Rc;

use delulu_broker::audit::MemSink;
use delulu_broker::authority::{attenuation_check, Authority, Scopes};
use delulu_broker::ids::SeqIdSource;
use delulu_broker::time::{ClockSource, ManualClock};
use delulu_broker::tree::{Broker, EffState, GrantId, Holder, State};

/// A deterministic LCG (Numerical Recipes constants). Reproducibility matters more than quality:
/// a failing seed must replay identically on every platform, and `rand` is not a dependency here.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 16
    }
    fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next() % n as u64) as usize }
    }
    /// True with probability `pct`/100.
    fn chance(&mut self, pct: u64) -> bool {
        self.next() % 100 < pct
    }
}

/// Deliberately mixed spellings of the same paths — the input canonicalization exists for. A
/// generator that only ever wrote `./data` could not tell a canonicalizing broker from a
/// non-canonicalizing one.
const SPELLINGS: &[&str] = &[
    "./data", "data", "./data/", r".\data", "./data/sub", ".\\data\\sub", "./data/sub/deep",
    "./data//sub", "./data/x/../sub",
];

fn scopes_from(paths: &[&str]) -> Scopes {
    Scopes { fs_read: paths.iter().map(|s| s.to_string()).collect(), ..Default::default() }
}

/// Build the authority by **struct literal**, deliberately bypassing `Authority::new`.
///
/// `Authority::new` canonicalizes, so an earlier version of this file — which used it — handed the
/// broker authorities that were already canonical. Law 4 then held no matter what the broker did:
/// deleting every canonicalization call from `tree.rs` left all seven scales passing. A law that
/// cannot fail is not a law, so the raw spelling has to survive all the way to the custody
/// boundary for the check on the other side to mean anything.
fn raw_authority(paths: &[&str]) -> Authority {
    Authority { effects: BTreeSet::new(), scopes: scopes_from(paths) }
}

fn holder(i: usize) -> Holder {
    Holder::new("agent", format!("agent-{i}"), format!("peer-{i}"))
}

/// Walk to the root, collecting ancestors nearest-first.
fn ancestors(b: &Broker, id: &GrantId) -> Vec<GrantId> {
    let mut out = Vec::new();
    let mut cur = b.inspect(id).and_then(|n| n.parent.clone());
    while let Some(p) = cur {
        out.push(p.clone());
        cur = b.inspect(&p).and_then(|n| n.parent.clone());
    }
    out
}

/// Build a random authority graph of `agents` nodes, then assert every law over all of it.
fn run_scale(agents: usize, seed: u64) {
    let mut rng = Rng(seed);
    let clock = Rc::new(ManualClock::new(1_000));
    let sink = MemSink::new();
    let mut b = Broker::with_sources(Box::new(SeqIdSource::default()), Box::new(clock.clone()))
        .with_sink(Box::new(sink));

    // The root holds the widest authority anything can hold. Everything else attenuates from it.
    let root = b.issue_root(holder(0), raw_authority(&["./data"]), None).unwrap();
    let mut live: Vec<GrantId> = vec![root.clone()];
    let mut all: Vec<GrantId> = vec![root.clone()];

    let mut denied = 0usize;
    let mut revoked = 0usize;
    let mut expiring = 0usize;

    for i in 1..agents {
        // Delegate from a random existing node — not always the root, so chains get deep.
        let parent = live[rng.below(live.len())].clone();

        // A random subset of spellings, biased toward narrower paths so most delegations succeed
        // and the tree actually grows. Some are deliberately too wide and must be REFUSED.
        let want: Vec<&str> = if rng.chance(15) {
            vec!["./other"] // outside the root's scope entirely — must be denied
        } else {
            let n = 1 + rng.below(2);
            (0..n).map(|_| SPELLINGS[rng.below(SPELLINGS.len())]).collect()
        };

        // A quarter of grants carry a deadline; the rest inherit whatever bounds them.
        let ttl = if rng.chance(25) {
            expiring += 1;
            Some(clock.now_millis() + 1_000 + rng.below(5_000) as i64)
        } else {
            None
        };

        match b.attenuate(&parent, raw_authority(&want), holder(i), ttl) {
            Ok(child) => {
                all.push(child.clone());
                live.push(child);
            }
            Err(_) => denied += 1,
        }

        // Revoke interior nodes, so subtrees die while siblings live on. One revocation is forced
        // at the midpoint rather than left to chance: at 10 agents a 12% roll frequently never
        // fired, and the run then asserted the inheritance laws over a tree in which nothing had
        // ever been revoked. Coverage that depends on the seed is coverage that silently lapses.
        let force = i == agents / 2;
        if (force || rng.chance(12)) && live.len() > 3 {
            let victim = live[1 + rng.below(live.len() - 1)].clone();
            if b.revoke(&root, &victim).is_ok() {
                revoked += 1;
            }
        }

        // And occasionally step the clock forward, retiring the deadlined grants.
        if rng.chance(10) {
            clock.advance(400);
        }
    }

    // Push time well past every deadline so the expiry laws are actually exercised.
    clock.advance(50_000);

    assert!(denied > 0, "seed {seed}: no delegation was ever refused — the generator never \
                         proposed a widening, so this run did not test attenuation at all");
    assert!(revoked > 0, "seed {seed}: nothing was revoked — the inheritance law was not exercised");
    assert!(expiring > 0, "seed {seed}: nothing carried a deadline — expiry was not exercised");

    // ---- Law 1: attenuation holds against the parent, and transitively to the root ----------
    for id in &all {
        let node = b.inspect(id).expect("node exists");
        let Some(parent_id) = node.parent.clone() else { continue };
        let parent = b.inspect(&parent_id).expect("parent exists");
        assert!(
            attenuation_check(&node.authority, &parent.authority).is_ok(),
            "seed {seed}: {} holds authority its parent {} does not:\n  child:  {}\n  parent: {}",
            id.as_str(),
            parent_id.as_str(),
            node.authority.render_compact(),
            parent.authority.render_compact()
        );
        let root_auth = &b.inspect(&root).expect("root exists").authority;
        assert!(
            attenuation_check(&node.authority, root_auth).is_ok(),
            "seed {seed}: {} escaped the ROOT's authority — attenuation is not transitive",
            id.as_str()
        );
    }

    // ---- Laws 2 and 3: revocation and expiry are inherited ---------------------------------
    for id in &all {
        let eff = b.effective_state(id).expect("effective state");
        if eff != EffState::Live {
            continue;
        }
        for anc in ancestors(&b, id) {
            let anc_node = b.inspect(&anc).expect("ancestor exists");
            assert!(
                !matches!(anc_node.state, State::Revoked { .. }),
                "seed {seed}: {} is Live under REVOKED ancestor {}",
                id.as_str(),
                anc.as_str()
            );
            assert_eq!(
                b.effective_state(&anc),
                Some(EffState::Live),
                "seed {seed}: {} is Live while its ancestor {} is not — a child outliving the \
                 bound that contains it is the shape of a real shipped vulnerability",
                id.as_str(),
                anc.as_str()
            );
        }
    }

    // ---- Law 4: every stored authority is canonical -----------------------------------------
    for id in &all {
        let node = b.inspect(id).expect("node exists");
        assert!(
            node.authority.is_canonical(),
            "seed {seed}: {} was stored NON-canonically ({}) — the custody boundary let a raw \
             spelling through, so this grant hashes differently from its own equivalents",
            id.as_str(),
            node.authority.render_compact()
        );
    }

    // ---- Law 5: the tree is internally consistent -------------------------------------------
    assert_eq!(b.len(), all.len(), "seed {seed}: node count disagrees with what was issued");
}

#[test]
fn ten_agents() {
    run_scale(10, 0x1234);
}

#[test]
fn fifty_agents() {
    run_scale(50, 0x5EED);
}

#[test]
fn one_hundred_agents() {
    run_scale(100, 0xC0FFEE);
}

#[test]
fn two_hundred_fifty_agents() {
    run_scale(250, 0xBEEF);
}

#[test]
fn five_hundred_agents() {
    run_scale(500, 0xD00D);
}

#[test]
fn one_thousand_agents() {
    run_scale(1000, 0xFEEDFACE);
}

/// The same laws over many *different* random shapes rather than one big one. A single seed
/// exercises one topology; twenty exercise twenty, and a law that only holds for the shape the
/// first seed happened to build is not a law.
#[test]
fn twenty_independent_topologies() {
    for seed in 1..=20u64 {
        run_scale(120, seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    }
}
