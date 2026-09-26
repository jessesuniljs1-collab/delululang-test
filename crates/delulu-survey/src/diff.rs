//! `delulu-survey diff <git-ref>` (P4-06): what a change touched, and everything that breaks because
//! of it.
//!
//! The question before a review is not "what does this ONE file reach" (`impact`) but "what does this
//! CHANGE reach": several files, some added, some deleted, some the map has never heard of. So the
//! files git names are mapped to the map's nodes, and the walk starts from all of them at once
//! ([`Survey::walk_many`]) — the union of their blast radii, each node once, at its nearest distance,
//! every hop citing the file and line it was read from, exactly as `impact` does. Nothing a change
//! touched is dropped: a path the map has no node for is listed as such, not skipped, and a changed
//! file that is ENTRENCHED is named with its owner, because that is the fact a reviewer must see
//! before anything else.
//!
//! Reading git is the only thing here that leaves the process, and it is read-only: `git diff
//! --name-status` and `git ls-files --others`. A revision that begins with `-` is refused before git
//! is started — `git diff --output=F` writes a file, and an argument must never become an option.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::answers::{envelope, via_json};
use crate::{Dir, Survey};

/// One path a change touched, and what happened to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// `added`, `modified`, `deleted`, `type-changed`, `unmerged`, `untracked` — git's letter, spelled out.
    pub status: &'static str,
    /// Repository-relative, forward-slashed, as git prints it.
    pub path: String,
}

fn status_word(letter: char) -> &'static str {
    match letter {
        'A' => "added",
        'M' => "modified",
        'D' => "deleted",
        'T' => "type-changed",
        'U' => "unmerged",
        'C' => "copied",
        'R' => "renamed",
        _ => "changed",
    }
}

/// Parse `git diff --name-status -z` output: a status field, then one path (two for a rename or a
/// copy — the old path, reported as deleted, then the new one), each NUL-terminated. `-z` so that a
/// path with a space, a quote or a newline arrives as itself and not in git's quoted form.
pub fn parse_name_status(z: &str) -> Result<Vec<Change>, String> {
    let mut fields = z.split('\0').filter(|f| !f.is_empty());
    let mut out = Vec::new();
    while let Some(status) = fields.next() {
        let letter = status.chars().next().ok_or("an empty status field")?;
        let mut path = || fields.next().map(str::to_string).ok_or_else(|| format!("status `{status}` names no path"));
        match letter {
            'R' | 'C' => {
                let old = path()?;
                let new = path()?;
                if letter == 'R' {
                    out.push(Change { status: "deleted", path: old });
                }
                out.push(Change { status: if letter == 'R' { "added" } else { "copied" }, path: new });
            }
            _ => out.push(Change { status: status_word(letter), path: path()? }),
        }
    }
    Ok(out)
}

/// The paths `rev` changed in `root`'s repository, compared with the working tree (so uncommitted
/// work counts, and untracked files are included — a new file is a change), or between two commits
/// when `rev` is a range (`A..B`, which compares commits only).
pub fn git_changes(root: &Path, rev: &str) -> Result<Vec<Change>, String> {
    if rev.is_empty() || rev.starts_with('-') {
        return Err(format!("`{rev}` is not a revision: a revision may not be empty or begin with `-`"));
    }
    let git = |args: &[&str]| -> Result<String, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .map_err(|e| format!("cannot run git: {e}"))?;
        if !out.status.success() {
            let why = String::from_utf8_lossy(&out.stderr);
            return Err(format!("git could not answer for `{rev}`: {}", why.trim()));
        }
        String::from_utf8(out.stdout).map_err(|_| "git printed a path that is not UTF-8".to_string())
    };
    let mut changes = parse_name_status(&git(&["diff", "--name-status", "-z", "--no-renames", rev, "--"])?)?;
    if !rev.contains("..") {
        for path in git(&["ls-files", "--others", "--exclude-standard", "-z"])?.split('\0').filter(|p| !p.is_empty()) {
            changes.push(Change { status: "untracked", path: path.to_string() });
        }
    }
    changes.sort_by(|a, b| a.path.cmp(&b.path).then(a.status.cmp(b.status)));
    changes.dedup();
    Ok(changes)
}

/// The machine answer: every changed path with the node it maps to in the CURRENT map (`null` when
/// there is none — a file deleted from the working tree, or a kind the map does not read), the
/// changed nodes that are entrenched, and the
/// union walk toward what breaks, each hop with the changed node it traces back to (`origin`).
/// Uncapped, like `impact --json`.
pub fn diff_json(survey: &Survey, rev: &str, changes: &[Change], depth: u32) -> Value {
    let by_path: BTreeMap<&str, &crate::Node> =
        survey.nodes.iter().filter_map(|n| n.path.as_deref().map(|p| (p, n))).collect();
    let mut starts: Vec<&str> = Vec::new();
    let mut entrenched = Vec::new();
    let changed: Vec<Value> = changes
        .iter()
        .map(|c| {
            let node = by_path.get(c.path.as_str());
            if let Some(n) = node {
                starts.push(&n.id);
                if let Some(e) = &n.entrenched {
                    entrenched.push(json!({
                        "node": n.id,
                        "owner": e.owner,
                        "pattern": e.pattern,
                        "matched_at": { "file": e.file, "line": e.line },
                    }));
                }
            }
            json!({ "path": c.path, "status": c.status, "node": node.map(|n| n.id.as_str()) })
        })
        .collect();
    let reached = survey.walk_many(&starts, Dir::Incoming, depth);
    // Each hop's origin: follow `from` back until a start. Every chain ends at one, because a node
    // is only ever reached from a start or from a node reached before it.
    let parent: BTreeMap<&str, &str> = reached.iter().map(|r| (r.id, r.from)).collect();
    let hops: Vec<Value> = reached
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "depth": r.depth,
                "from": r.from,
                "origin": origin(&parent, r.id),
                "via": via_json(r.via.kind, &r.via.file, r.via.line),
            })
        })
        .collect();
    let mapped = changed.iter().filter(|c| !c["node"].is_null()).count();
    envelope(
        "diff",
        json!({
            "rev": rev,
            "question": "what breaks because of this change",
            "changed": changed,
            "mapped": mapped,
            "entrenched": entrenched,
            "depth_limit": depth,
            "reached": hops.len(),
            "hops": hops,
        }),
    )
}

/// The start a hop traces back to, by following `from` until it reaches a node nothing reached.
fn origin<'a>(parent: &BTreeMap<&'a str, &'a str>, mut id: &'a str) -> &'a str {
    while let Some(p) = parent.get(id) {
        id = p;
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_status_is_read_as_git_writes_it_with_z() {
        let z = "M\0crates/a/src/lib.rs\0A\0docs/a b.md\0D\0gone.rs\0R087\0old/x.rs\0new/x.rs\0T\0link\0";
        let got = parse_name_status(z).unwrap();
        let pairs: Vec<(&str, &str)> = got.iter().map(|c| (c.status, c.path.as_str())).collect();
        assert_eq!(
            pairs,
            vec![
                ("modified", "crates/a/src/lib.rs"),
                ("added", "docs/a b.md"),
                ("deleted", "gone.rs"),
                ("deleted", "old/x.rs"),
                ("added", "new/x.rs"),
                ("type-changed", "link"),
            ]
        );
        assert!(parse_name_status("M\0").is_err(), "a status with no path");
        assert_eq!(parse_name_status("").unwrap(), vec![]);
    }

    #[test]
    fn a_revision_that_looks_like_an_option_is_refused_before_git_runs() {
        let here = Path::new(".");
        for bad in ["--output=/tmp/x", "-p", ""] {
            let e = git_changes(here, bad).unwrap_err();
            assert!(e.contains("may not be empty or begin with `-`"), "{bad}: {e}");
        }
    }
}
