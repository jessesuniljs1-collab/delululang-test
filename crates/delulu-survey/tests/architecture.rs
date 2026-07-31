//! The architecture, held to its word.
//!
//! Three properties are stated in prose across this repository, and prose does not fail a build.
//! Each is checked here, and — because the Survey already reads every manifest and every `use` —
//! **each is checked using the map itself**. The map is the evidence for the claims made about the
//! thing that produces it.
//!
//! 1. The Survey depends on **no sibling crate**, so it opens when the compiler does not build.
//! 2. The dependency runs **one way**: the CLI knows about the Survey, never the reverse.
//! 3. The Survey's own **structural invariants live in one place**, so the command that reports
//!    them and the test that enforces them cannot drift.

use delulu_survey::{EdgeKind, Survey};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// **The Survey must depend on no sibling crate.**
///
/// This is the property that makes the map usable in the situation you most need a map: the tree
/// is mid-refactor and the checker does not compile. The moment `delulu-survey` depends on
/// `delulu-check`, reading the map requires building the thing the map was going to help you fix.
///
/// Stated in `Cargo.toml` and in the crate docs; enforced here, from the manifest as the Survey
/// itself read it.
#[test]
fn the_survey_depends_on_no_sibling_crate() {
    let s = Survey::build(&repo_root());
    let siblings: Vec<&str> = s
        .out("crate:delulu-survey")
        .into_iter()
        .filter(|e| e.kind == EdgeKind::DependsOn)
        .map(|e| e.to.as_str())
        .collect();

    assert!(
        siblings.is_empty(),
        "delulu-survey depends on {siblings:?}. The map must be readable when the compiler is not \
         buildable — that is the whole reason it derives from text rather than from compiler facts. \
         If this dependency is genuinely needed, the property is what has to change, deliberately, \
         and `docs/survey/README.md` says otherwise today."
    );
}

/// **The dependency runs one way.**
///
/// `delulu doctor` makes repository health part of the ordinary developer experience, which means
/// the CLI knows about the Survey. The reverse must never become true: a Survey that needed the
/// CLI would be a cycle, and would put the map behind the very binary it exists to help repair.
#[test]
fn the_cli_knows_about_the_survey_and_not_the_reverse() {
    let s = Survey::build(&repo_root());

    let cli_uses_survey = s
        .out("crate:delulu")
        .into_iter()
        .any(|e| e.kind == EdgeKind::DependsOn && e.to == "crate:delulu-survey");
    assert!(cli_uses_survey, "`delulu doctor` needs the Survey; the CLI should declare it");

    let survey_uses_cli = s
        .out("crate:delulu-survey")
        .into_iter()
        .any(|e| e.kind == EdgeKind::DependsOn && e.to == "crate:delulu");
    assert!(!survey_uses_cli, "delulu-survey depends on the CLI — the direction has been inverted");
}

/// **The invariants have exactly one home.**
///
/// `delulu doctor` renders whatever `delulu_survey::integrity` returns and adds nothing of its
/// own, and `freshness.rs` asserts the same list. This test pins the arrangement that makes that
/// true: the list is non-empty, every entry is named and reportable, and names are unique — so a
/// caller can key on one and two invariants cannot quietly become the same row.
#[test]
fn the_integrity_list_is_a_single_usable_source() {
    let s = Survey::build(&repo_root());
    let checks = delulu_survey::integrity(&s);

    assert!(checks.len() >= 4, "expected the four invariants at minimum, found {}", checks.len());
    for c in &checks {
        assert!(!c.name.is_empty(), "an invariant with no name cannot be reported to anyone");
        assert!(!c.detail.is_empty(), "`{}` reports nothing measured — a passing check should still inform", c.name);
    }

    let mut names: Vec<&str> = checks.iter().map(|c| c.name).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "two invariants share a name; a caller cannot tell them apart");
}

/// The health verdict must agree with the checks it is built from — a summary that can disagree
/// with its own detail is worse than no summary.
#[test]
fn the_health_verdict_agrees_with_its_own_evidence() {
    let root = repo_root();
    let h = delulu_survey::inspect(&root, delulu_survey::Repair::ReportOnly);

    assert_eq!(
        h.is_healthy(),
        h.integrity.iter().all(|i| i.ok) && h.tally.is_healthy() && h.stale.is_empty(),
        "is_healthy() disagrees with the integrity list, the tally, or the staleness it reports"
    );
    assert_eq!(h.failures().len(), h.integrity.iter().filter(|i| !i.ok).count());
}
