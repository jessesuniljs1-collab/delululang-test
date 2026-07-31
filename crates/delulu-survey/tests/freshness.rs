//! The anti-rot gate.
//!
//! A map maintained by remembering to maintain it is a map that is wrong by the third commit. This
//! suite makes the tree itself the authority: if the repository changed and `docs/survey/` did not,
//! `cargo test --workspace` fails and says exactly which file is behind and how to fix it.
//!
//! The remedy is always one command:
//!
//! ```text
//! cargo run -p delulu-survey -- build
//! ```

use delulu_survey::{render, Survey};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // `crates/delulu-survey` → the workspace root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn committed(name: &str) -> String {
    let p = repo_root().join("docs/survey").join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("docs/survey/{name} is missing ({e}) — run `cargo run -p delulu-survey -- build`"))
}

/// The gate. Every generated channel must match what is committed.
#[test]
fn the_committed_map_matches_the_tree() {
    let s = Survey::build(&repo_root());
    for (name, fresh) in [
        ("SURVEY.md", render::markdown(&s)),
        ("DISCREPANCIES.md", render::discrepancies(&s)),
        ("survey.json", render::json(&s)),
    ] {
        let on_disk = committed(name);
        if on_disk != fresh {
            let (a, b) = first_difference(&on_disk, &fresh);
            panic!(
                "docs/survey/{name} is out of date — the repository changed and its map did not.\n\
                 Run `cargo run -p delulu-survey -- build` and commit the result alongside the change.\n\n\
                 first difference at line {}:\n  committed: {a}\n  current:   {b}",
                a_line(&on_disk, &fresh)
            );
        }
    }
}

fn a_line(a: &str, b: &str) -> usize {
    a.lines().zip(b.lines()).position(|(x, y)| x != y).map(|i| i + 1).unwrap_or(0)
}

fn first_difference(a: &str, b: &str) -> (String, String) {
    match a.lines().zip(b.lines()).find(|(x, y)| x != y) {
        Some((x, y)) => (truncate(x), truncate(y)),
        None => (format!("<{} lines>", a.lines().count()), format!("<{} lines>", b.lines().count())),
    }
}

fn truncate(s: &str) -> String {
    if s.chars().count() > 120 {
        s.chars().take(120).collect::<String>() + " …"
    } else {
        s.to_string()
    }
}

/// Building twice must produce the same map.
///
/// This is not a formality. The first version of the Survey walked its own output directory, so
/// each run found new relations inside the report the previous run had written, and no number of
/// runs would ever converge — the gate above would have failed forever while being entirely
/// correct to do so. `OUTPUT_DIR` is excluded for this reason and this test is what holds it.
#[test]
fn the_map_reaches_a_fixed_point() {
    let root = repo_root();
    let first = render::json(&Survey::build(&root));
    let second = render::json(&Survey::build(&root));
    assert_eq!(first, second, "two builds of the same tree disagree — the Survey is reading something it writes");
}

/// The provenance law, enforced rather than promised.
#[test]
fn every_edge_names_the_line_it_was_read_from() {
    let s = Survey::build(&repo_root());
    let bad: Vec<&delulu_survey::Edge> = s.edges.iter().filter(|e| e.file.is_empty() || e.line == 0).collect();
    assert!(
        bad.is_empty(),
        "{} edge(s) carry no citation, e.g. {:?} — an edge nobody can check is a guess with better \
         typography, and this map does not publish guesses",
        bad.len(),
        bad.first()
    );
}

/// Every edge endpoint must be a node that exists.
#[test]
fn no_edge_points_at_a_node_that_is_not_there() {
    let s = Survey::build(&repo_root());
    let dangling: Vec<String> = s
        .edges
        .iter()
        .flat_map(|e| [&e.from, &e.to])
        .filter(|id| s.node(id).is_none())
        .cloned()
        .collect();
    assert!(dangling.is_empty(), "{} dangling endpoint(s), e.g. {:?}", dangling.len(), dangling.first());
}

/// The map's own totals must agree with the map's own contents. A map that miscounts itself has no
/// standing to report that another document miscounts.
#[test]
fn the_reported_totals_match_what_is_in_the_map() {
    let s = Survey::build(&repo_root());
    let crates = s.nodes.iter().filter(|n| n.kind == delulu_survey::NodeKind::Crate && n.id != "workspace").count();
    assert_eq!(s.facts.crates as usize, crates, "facts.crates disagrees with the crate nodes in the map");
    assert!(
        s.facts.crates_shipped <= s.facts.crates,
        "more shipped crates than crates: {} > {}",
        s.facts.crates_shipped,
        s.facts.crates
    );
}

/// The tree the Survey actually describes, checked against the tree as Cargo sees it.
#[test]
fn every_workspace_member_is_in_the_map() {
    let root = repo_root();
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace manifest");
    let s = Survey::build(&root);

    let mut in_members = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with("members") {
            in_members = true;
            continue;
        }
        if in_members {
            if t.starts_with(']') {
                break;
            }
            let Some(entry) = t.trim_end_matches(',').strip_prefix('"') else { continue };
            let name = entry.trim_end_matches('"').rsplit('/').next().unwrap_or_default();
            assert!(
                s.node(&format!("crate:{name}")).is_some(),
                "workspace member `{name}` is missing from the Survey"
            );
        }
    }
}
