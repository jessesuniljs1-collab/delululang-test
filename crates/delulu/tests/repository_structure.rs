//! `docs/REPOSITORY_STRUCTURE.md` §5 claims **every markdown file in the repository** is accounted
//! for. This file is the mechanical check behind that claim (verification finding NE-12).
//!
//! The document said, in two places, *"Both directions are now checked mechanically … every markdown
//! file is accounted for by name or by its group."* Nothing checked it: `grep -rn
//! REPOSITORY_STRUCTURE crates/ --include=*.rs` found no hits, no test read the document, and no
//! Survey rule looked at it. A claim of mechanical checking is worse than no claim, because a reader
//! stops verifying what a machine is said to verify — and the document's own history is two
//! documented drifts in six weeks.
//!
//! The list of markdown files comes from the committed `docs/survey/survey.json` (`kind: "doc"`),
//! which is generated from the tree and whose staleness already fails a test. So this gate reads a
//! generated fact and a hand-written document, and fails when they disagree — which is the only
//! shape that can keep a hand-written map honest.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// A top-level directory is NOT a group. Naming `docs/` cannot account for 150 files inside it, and
/// allowing that would make this gate unfailable — the shape the campaign keeps finding (a gate that
/// cannot fail is not a gate). A group has to be specific enough that a reader following it lands on
/// a handful of related files.
const NOT_A_GROUP: &[&str] =
    &["", ".", "docs", "crates", "tests", "measurements", "examples", "scripts", ".github", "morphs"];

/// Is `path` accounted for by §5's text? Three ways, in the document's own words: "by name or by its
/// group" — an exact path, a bare file name, or a directory §5 lists that contains it.
fn accounted_for(path: &str, section5: &str) -> bool {
    if section5.contains(path) {
        return true;
    }
    let base = path.rsplit('/').next().unwrap_or(path);
    if !base.is_empty() && section5.contains(base) {
        return true;
    }
    let parts: Vec<&str> = path.split('/').collect();
    for k in (1..parts.len()).rev() {
        let group = parts[..k].join("/");
        if NOT_A_GROUP.contains(&group.as_str()) {
            continue;
        }
        if section5.contains(&format!("{group}/")) || section5.contains(&format!("`{group}`")) {
            return true;
        }
    }
    false
}

fn section5() -> String {
    let doc = std::fs::read_to_string(repo_root().join("docs").join("REPOSITORY_STRUCTURE.md"))
        .expect("REPOSITORY_STRUCTURE.md is committed");
    let at = doc
        .find("## 5. Every document in this repository")
        .expect("§5 must exist — it is the section this gate checks");
    doc[at..].to_string()
}

fn markdown_files() -> Vec<String> {
    let survey = std::fs::read_to_string(repo_root().join("docs").join("survey").join("survey.json"))
        .expect("docs/survey/survey.json is committed and its staleness already fails a test");
    let v: serde_json::Value = serde_json::from_str(&survey).expect("survey.json is valid JSON");
    let files: Vec<String> = v["nodes"]
        .as_array()
        .expect("survey.json carries a nodes array")
        .iter()
        .filter(|n| n["kind"] == "doc")
        .filter_map(|n| n["path"].as_str().map(str::to_string))
        .collect();
    assert!(
        files.len() > 100,
        "the Survey reported only {} markdown files — the scan broke, and a gate that reads nothing \
         passes everything",
        files.len()
    );
    files
}

#[test]
fn every_markdown_file_is_accounted_for_in_repository_structure_section_5() {
    let s5 = section5();
    let mut missing: Vec<String> = Vec::new();
    for f in markdown_files() {
        if !accounted_for(&f, &s5) {
            missing.push(f);
        }
    }
    missing.sort();
    assert!(
        missing.is_empty(),
        "`docs/REPOSITORY_STRUCTURE.md` §5 says every markdown file is accounted for by name or by \
         its group. These are not:\n  {}\n\nAdd each by name, or name the directory that groups \
         them. (Editing this list of exceptions is not an option — there is no exception list.)",
        missing.join("\n  ")
    );
}

/// **The mutant.** A gate that only ever sees a passing corpus is a gate nobody has watched fail.
/// These paths are synthetic: files in directories §5 does not list, and a file directly under a
/// top-level directory whose name §5 never writes. Each must be reported unaccounted — if any is
/// accepted, the rule above has become vacuous and the test above proves nothing.
#[test]
fn the_accounting_rule_refuses_a_file_nobody_listed() {
    let s5 = section5();
    let synthetic = [
        "docs/NOT_IN_THE_STRUCTURE_GUIDE.md",
        "docs/some-new-directory/NOTES.md",
        "crates/delulu-newcrate/README_NOT_LISTED.md",
        "measurements/study-zzz/ZZZ_NOT_A_RECORD.md",
        "NOT_LISTED_AT_ROOT.md",
    ];
    for p in synthetic {
        assert!(
            !accounted_for(p, &s5),
            "`{p}` is in no list and no group, and the accounting rule accepted it — the rule is \
             vacuous and the gate beside it is not checking anything"
        );
    }
    // And the positive control, so the rule is not simply "refuse everything": a real file named by
    // path, one named by bare name, and one covered by the group §5 declares for agents' notes.
    for p in [
        "docs/REMAINING_WORK.md",
        "HANDOFF.md",
        "docs/security/red-team-surfaces-2026-08-08/agent-notes/wasm-NOTES.md",
    ] {
        assert!(accounted_for(p, &s5), "`{p}` is accounted for and the rule must say so");
    }
}
