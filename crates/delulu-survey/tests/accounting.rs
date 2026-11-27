//! Every tracked file is accounted for: it is a node in the map, or a named rule says why it is not.
//!
//! `clean_checkout.rs` asks one direction — does the map read a file git ignores? This asks the other:
//! does git track a file the map does not hold? A map with a hole is still a map, and still passes its
//! freshness gate, because the gate compares the map with what the map chose to read. So the
//! denominator is git's, not the Survey's: `git ls-files`. Architecture slice 1 of the owner's
//! commission of 2026-11-14 (*"every tracked, in-scope source/config/test/CI file should be accounted
//! for"*). Measured when this was written: 1,275 tracked files, 1,274 of them nodes, and one —
//! `.claude/settings.json` — excluded by the rule that keeps agent scratch space out of the map.

use delulu_survey::{Survey, EXCLUDED_DIRS, EXCLUDED_FILE_EXTENSIONS, OUTPUT_DIR};
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// The rule that excludes `path`, if one does — a directory on the way, an extension, or the map's own output.
fn excluded_by(path: &str) -> Option<String> {
    if path.starts_with(&format!("{OUTPUT_DIR}/")) {
        return Some(format!("`{OUTPUT_DIR}/` — the Survey's own output, which it must not map"));
    }
    for (dir, why) in EXCLUDED_DIRS {
        if path.split('/').any(|part| part == *dir) {
            return Some(format!("`{dir}/` — {why}"));
        }
    }
    let ext = path.rsplit('.').next().unwrap_or_default();
    EXCLUDED_FILE_EXTENSIONS.iter().find(|(e, _)| *e == ext).map(|(e, why)| format!("`.{e}` — {why}"))
}

#[test]
fn every_tracked_file_is_a_node_or_is_excluded_by_a_named_rule() {
    let root = repo_root();
    let listed = Command::new("git").arg("-C").arg(&root).args(["ls-files", "-z"]).output();
    let Some(listed) = listed.ok().filter(|o| o.status.success()) else {
        // A source archive has no index to ask. The only case in which this holds nothing, and it says so.
        eprintln!("SKIPPED: {} is not a git work tree, so there is no list of tracked files", root.display());
        return;
    };
    let tracked: Vec<String> =
        String::from_utf8_lossy(&listed.stdout).split('\0').filter(|p| !p.is_empty()).map(str::to_string).collect();
    assert!(tracked.len() > 500, "git lists {} tracked files — this would pass while checking nothing", tracked.len());

    let s = Survey::build(&root);
    let mapped: std::collections::BTreeSet<&str> = s.nodes.iter().filter_map(|n| n.path.as_deref()).collect();
    let mut excluded = 0usize;
    let mut holes: Vec<&str> = Vec::new();
    for path in &tracked {
        // A file deleted in the working tree and not yet committed is still in the index; the map reads the tree.
        if !root.join(path).exists() {
            continue;
        }
        if mapped.contains(path.as_str()) {
            continue;
        }
        match excluded_by(path) {
            Some(_) => excluded += 1,
            None => holes.push(path),
        }
    }
    assert!(
        holes.is_empty(),
        "{} tracked file(s) are in no node of the map and no rule excludes them:\n  {}\n\
         Either the scanner skips a kind of file it should read (`crates/delulu-survey/src/scan.rs`), or the file \
         belongs under a named exclusion (`EXCLUDED_DIRS`, `EXCLUDED_FILE_EXTENSIONS`) with its reason written down.",
        holes.len(),
        holes.join("\n  ")
    );
    // The exclusions are a short, named list — not a second way for the map to have holes.
    assert!(excluded <= 5, "{excluded} tracked files are excluded by rule; an exclusion that grows is a hole with a name");
}

/// The gate above can fail: the same check, against a map that has lost a file, names the file.
#[test]
fn a_tracked_file_missing_from_the_map_would_be_named() {
    assert!(excluded_by("crates/delulu/src/cli.rs").is_none(), "an ordinary source file is excluded by no rule");
    assert!(excluded_by("README.md").is_none());
    assert!(excluded_by(".claude/settings.json").is_some_and(|w| w.contains("agent scratch")));
    assert!(excluded_by(".remember/now.md").is_some_and(|w| w.contains("plugin")));
    assert!(excluded_by("docs/survey/survey.json").is_some_and(|w| w.contains("own output")));
    assert!(excluded_by("editors/vscode/delulu.vsix").is_some());
    // A directory merely NAMED like an excluded one, deeper in the tree, is excluded too — as the scanner does.
    assert!(excluded_by("crates/x/target/debug/a.rs").is_some());
    // And a name that only contains an excluded word is not.
    assert!(excluded_by("docs/targets.md").is_none());
}
