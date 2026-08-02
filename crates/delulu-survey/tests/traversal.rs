//! The transitive verbs, and the rule that keeps them honest.
//!
//! `rdeps` answers **one hop**. That is the right answer to "what points at this" and the wrong
//! answer to "what breaks if I change this" — `mod:crates/delulu-check/src/check.rs`, the module
//! that decides what type-checks, has exactly **one structural** edge arriving at it. The Survey's own README
//! tells a reader to reach for `rdeps` first when changing anything, so the primary documented use
//! case was returning a confident number that understated blast radius by two orders of magnitude.
//!
//! `impact` and `affected-by` close that. The interesting part is what the first version got wrong.
//!
//! **Composing every edge kind made the answer meaningless.** Measured on this repository, a walk
//! that followed all relations reported **236 nodes reachable from every starting node** — from a
//! crate, from a module, from a diagnostic code, and from `doc:README.md`. Narrative edges connect
//! everything to everything eventually: `README links-to CONTRIBUTING` followed by `CONTRIBUTING
//! references cli.rs` is two unrelated sentences laid end to end, and calling their composition
//! "what breaks" asserts a relation no file in this repository states. Every individual edge was
//! cited and true; the *path* was not.
//!
//! So a walk follows only relations that propagate, and these tests hold that line — because the
//! failure it prevents is invisible: a saturated answer looks like a thorough one.

use delulu_survey::{Dir, EdgeKind, Survey, MAX_WALK_DEPTH};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn survey() -> Survey {
    Survey::build(&repo_root())
}

const CHECKER: &str = "mod:crates/delulu-check/src/check.rs";

/// **The gap this closed, stated as a test.** One hop is not the blast radius.
///
/// The comparison is against the **composing** subset of the incoming edges, and that correction
/// was itself a lesson. The first version compared against `into_().len()` — every incoming edge —
/// and passed, until the change notes for this very feature were written: several documents now
/// mention `check.rs` by name, each of which is a real cited `links-to` edge, and the raw in-degree
/// went from 1 to 8 without one line of code moving. The map maps the documents that describe the
/// map. Compare like with like.
#[test]
fn a_transitive_walk_reaches_what_one_hop_cannot() {
    let s = survey();
    let structural = s.into_(CHECKER).iter().filter(|e| e.kind.composes()).count();
    let transitive = s.walk(CHECKER, Dir::Incoming, MAX_WALK_DEPTH).len();

    assert!(structural >= 1, "the checker's module must be declared by something");
    assert!(
        transitive > structural * 20,
        "the checker's central module has {structural} structural pointer(s) and {transitive} \
         transitive — if these are close, the walk is not walking"
    );
}

/// **The saturation rule.** A relation whose chain means nothing must not be composed.
///
/// A document is the clearest case: changing `README.md` breaks no code, so a walk that claims
/// otherwise has stopped describing this repository. Before `EdgeKind::composes`, this returned
/// 236.
#[test]
fn narrative_edges_do_not_compose_into_a_build_relation() {
    let s = survey();
    for id in ["doc:README.md", "doc:CONTRIBUTING.md"] {
        let reached = s.walk(id, Dir::Incoming, MAX_WALK_DEPTH);
        assert!(
            reached.is_empty(),
            "`{id}` claims {} node(s) break if it changes. A document does not propagate build \
             breakage; composing `links-to` with `depends-on` invents a relation no file states.",
            reached.len()
        );
        // The one-hop view must still show those relations — they are true, just not composable.
        assert!(
            !s.into_(id).is_empty(),
            "`{id}` should still have incoming narrative edges for `rdeps` to report"
        );
    }
}

/// The same rule from the other side: a diagnostic code is an idea, not a build input.
#[test]
fn a_diagnostic_code_does_not_propagate_build_breakage() {
    let s = survey();
    let direct = s.into_("code:DL0501").len();
    assert!(direct > 10, "DL0501 should be widely referenced; found {direct}");
    assert!(
        s.walk("code:DL0501", Dir::Incoming, MAX_WALK_DEPTH).is_empty(),
        "a code's references are narrative; they must not compose into a dependency chain"
    );
}

/// **The blast radius must order the way a dependency graph does.** A foundational crate reaches
/// more than the application that sits on top of it — if that inverts, the direction is backwards.
#[test]
fn deeper_crates_have_larger_blast_radius_than_the_cli() {
    let s = survey();
    let n = |id: &str| s.walk(id, Dir::Incoming, MAX_WALK_DEPTH).len();
    let diag = n("crate:delulu-diag");
    let syntax = n("crate:delulu-syntax");
    let cli = n("crate:delulu");

    assert!(diag > syntax, "delulu-diag ({diag}) is under delulu-syntax ({syntax})");
    assert!(syntax > cli, "delulu-syntax ({syntax}) should reach more than the CLI ({cli})");
    assert!(cli > 0, "the CLI crate reaches nothing, which cannot be right");
}

/// Every hop carries the citation, at every distance. The provenance law does not weaken over
/// distance — that is the whole reason a transitive verb is allowed to exist here at all.
#[test]
fn every_hop_of_a_walk_names_the_file_and_line_it_was_read_from() {
    let s = survey();
    let reached = s.walk(CHECKER, Dir::Incoming, MAX_WALK_DEPTH);
    assert!(!reached.is_empty());
    for r in &reached {
        assert!(!r.via.file.is_empty(), "{} arrived on an uncited edge", r.id);
        assert!(r.via.line > 0, "{} arrived on an edge with no line", r.id);
        assert!(r.via.kind.composes(), "{} arrived on a non-composing edge", r.id);
        // The hop must actually connect the two nodes it claims to.
        let ends = [r.via.from.as_str(), r.via.to.as_str()];
        assert!(ends.contains(&r.id) && ends.contains(&r.from), "{} arrived on an unrelated edge", r.id);
    }
}

/// A node is reported once, at its shortest distance — so a cycle terminates the walk rather than
/// circling it, and a count means "how many things", not "how many ways".
#[test]
fn each_node_appears_once_at_its_shortest_depth() {
    let s = survey();
    let reached = s.walk("crate:delulu-diag", Dir::Incoming, MAX_WALK_DEPTH);
    let mut ids: Vec<&str> = reached.iter().map(|r| r.id).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total, "a node was reported more than once");

    // Depths are non-decreasing in emission order, which is what makes "shortest" true.
    let mut last = 0;
    for r in &reached {
        assert!(r.depth >= last, "depths are not ordered: {} after {last}", r.depth);
        last = r.depth;
    }
}

/// `--depth N` bounds the answer rather than changing it: a shallower walk is a prefix.
#[test]
fn a_bounded_walk_is_a_prefix_of_the_full_one() {
    let s = survey();
    let full = s.walk(CHECKER, Dir::Incoming, MAX_WALK_DEPTH);
    let two = s.walk(CHECKER, Dir::Incoming, 2);
    assert!(two.len() < full.len(), "depth 2 should be smaller than the full walk");
    for (a, b) in two.iter().zip(full.iter()) {
        assert_eq!(a.id, b.id, "a bounded walk must be a prefix, not a different answer");
    }
}

/// A path is a real chain: each hop starts where the last one ended, and the whole thing joins the
/// two nodes that were asked about.
#[test]
fn a_path_is_a_connected_chain_of_cited_edges() {
    let s = survey();
    let chain = s.shortest_path("crate:delulu", "crate:delulu-diag").expect("the CLI reaches diag");
    assert!(!chain.is_empty());
    assert_eq!(chain[0].from, "crate:delulu");
    assert_eq!(chain[chain.len() - 1].to, "crate:delulu-diag");
    for w in chain.windows(2) {
        assert_eq!(w[0].to, w[1].from, "the chain is broken between two hops");
    }
    for e in &chain {
        assert!(!e.file.is_empty() && e.line > 0, "an uncited hop");
    }
}

/// Direction is meaningful: the CLI reaches diagnostics, diagnostics do not reach the CLI. A
/// path verb that quietly answered the reverse question would misdescribe the architecture.
#[test]
fn a_path_is_directed() {
    let s = survey();
    assert!(s.shortest_path("crate:delulu", "crate:delulu-diag").is_some());
    assert!(
        s.shortest_path("crate:delulu-diag", "crate:delulu").is_none(),
        "delulu-diag must not depend on the CLI — and if it did, `architecture.rs` would be failing"
    );
}

/// Asking about something that is not on the map returns nothing, rather than an empty answer that
/// looks like a real one.
#[test]
fn an_unknown_node_walks_nowhere() {
    let s = survey();
    assert!(s.walk("crate:does-not-exist", Dir::Incoming, MAX_WALK_DEPTH).is_empty());
    assert!(s.shortest_path("crate:does-not-exist", "crate:delulu").is_none());
    assert!(s.shortest_path("crate:delulu", "crate:does-not-exist").is_none());
}

/// The composing set is a decision, not an accident: every edge kind is classified deliberately,
/// so adding a kind forces someone to decide which side it is on.
#[test]
fn every_edge_kind_in_the_map_has_a_composition_verdict() {
    let s = survey();
    let mut kinds: Vec<EdgeKind> = s.edges.iter().map(|e| e.kind).collect();
    kinds.sort();
    kinds.dedup();
    assert!(kinds.len() >= 8, "the map should exercise most edge kinds; saw {}", kinds.len());
    // `composes()` is exhaustive over the enum — a new variant will not compile until it is
    // classified. This asserts the classification is not vacuously all-true or all-false.
    assert!(kinds.iter().any(|k| k.composes()), "no kind composes; walks would return nothing");
    assert!(kinds.iter().any(|k| !k.composes()), "every kind composes; the saturation is back");
}
