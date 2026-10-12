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

/// The heading lines of `text` that repeat the heading just before them with nothing but blank lines between —
/// outside fenced code. (A section name used again further down, `### Evidence` under two dates, is not this.)
fn repeated_headings(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut prev: Option<String> = None;
    let mut fence = false;
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fence = !fence;
            prev = None;
            continue;
        }
        if fence {
            continue;
        }
        let is_heading = line.starts_with('#') && line.trim_start_matches('#').starts_with(' ');
        if is_heading {
            if prev.as_deref() == Some(line) {
                out.push((i + 1, line.to_string()));
            }
            prev = Some(line.to_string());
        } else if !line.trim().is_empty() {
            prev = None;
        }
    }
    out
}

/// Routine run 20: a records patch inserted its lessons BEFORE `### 11.6` with that heading at the end of the
/// inserted text as well, and `HANDOFF.md` went to `master` (`52b2572`) with the heading twice — every anchor
/// checked, every document gate green. No document of this repository repeats a heading on the line that
/// follows it; the check is generic, and it is a machine's, because the eye skips a repeated line.
#[test]
fn no_markdown_file_repeats_a_heading_on_the_heading_after_it() {
    let mut bad: Vec<String> = Vec::new();
    for f in markdown_files() {
        let Ok(text) = std::fs::read_to_string(repo_root().join(&f)) else { continue };
        for (line, heading) in repeated_headings(&text) {
            bad.push(format!("{f}:{line}: `{heading}` repeats the heading before it"));
        }
    }
    assert!(bad.is_empty(), "a heading written twice in a row:\n  {}", bad.join("\n  "));
}

/// The check above, watched failing: `52b2572`'s shape is found, and headings with content between them, or a
/// repeat inside fenced code, are not.
#[test]
fn the_repeated_heading_rule_finds_the_shape_it_was_written_for_and_nothing_else() {
    let shape = "- a lesson\n\n### 11.6 If you are an assistant with memory, keep it current\n### 11.6 If you are an assistant with memory, keep it current\n\nbody\n";
    assert_eq!(repeated_headings(shape).len(), 1, "the 52b2572 shape must be found");
    let spaced = "### Evidence\n\n\n### Evidence\n";
    assert_eq!(repeated_headings(spaced).len(), 1, "blank lines between do not hide a repeat");
    let control = "## 2026-10-11\n### Evidence\ntext\n## 2026-10-12\n### Evidence\ntext\n```\n# x\n# x\n```\n#hashtag\n#hashtag\n";
    assert!(repeated_headings(control).is_empty(), "{:?}", repeated_headings(control));
}
