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

/// Every structural invariant the map claims for itself must actually hold.
///
/// The list is **not restated here**. It is `delulu_survey::integrity`, which `delulu doctor` also
/// reports — so an invariant added there is enforced by this test and shown by that command on the
/// same commit, and the two can never disagree about what "healthy" means. These properties used
/// to be written out twice, once here and once in the CLI, which is a divergence waiting for
/// someone to add a fourth.
#[test]
fn the_map_satisfies_every_invariant_it_claims() {
    let s = Survey::build(&repo_root());
    let checks = delulu_survey::integrity(&s);
    assert!(!checks.is_empty(), "the integrity list is empty — nothing is being checked at all");

    let failed: Vec<String> =
        checks.iter().filter(|c| !c.ok).map(|c| format!("{}: {}", c.name, c.detail)).collect();
    assert!(
        failed.is_empty(),
        "{} invariant(s) broken:\n  {}\n\nAn edge nobody can check is a guess with better \
         typography, and this map does not publish guesses.",
        failed.len(),
        failed.join("\n  ")
    );
}

/// `inspect` must never write when asked not to, and must agree with a plain build.
///
/// This is the property `delulu doctor --check` rests on, and the one a CI step or a git hook
/// depends on to be safe. It is tested against the real repository because that is where it is
/// relied upon.
#[test]
fn report_only_inspection_never_writes() {
    let root = repo_root();
    let before = mtimes(&root.join("docs/survey"));
    let h = delulu_survey::inspect(&root, delulu_survey::Repair::ReportOnly);
    assert!(h.regenerated.is_empty(), "ReportOnly regenerated {:?}", h.regenerated);
    assert!(h.write_error.is_none(), "ReportOnly attempted a write: {:?}", h.write_error);
    assert_eq!(mtimes(&root.join("docs/survey")), before, "ReportOnly touched the generated files");
}

fn mtimes(dir: &std::path::Path) -> Vec<(String, std::time::SystemTime)> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<(String, std::time::SystemTime)> = rd
        .filter_map(|e| e.ok())
        .filter_map(|e| Some((e.file_name().to_string_lossy().to_string(), e.metadata().ok()?.modified().ok()?)))
        .collect();
    out.sort();
    out
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
