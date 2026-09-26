//! P4-06: `delulu-survey diff <git-ref>` — what a change touched, and everything that breaks because
//! of it.
//!
//! The roadmap's verification is *a synthetic diff*: a change set this test writes down, so the answer
//! can be held against something known rather than against whatever the working tree happens to hold.
//! The rule held: the diff's reach is exactly the UNION of the changed files' own `impact` walks, each
//! node at its NEAREST distance, every hop citing a line and tracing back to a file the change touched;
//! nothing the change named is dropped (a deleted file and a path the map has no node for are listed
//! with no node, not skipped); and an entrenched file is named with its owner.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use delulu_survey::diff::{diff_json, Change};
use delulu_survey::{Dir, Survey, MAX_WALK_DEPTH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn change(status: &'static str, path: &str) -> Change {
    Change { status, path: path.to_string() }
}

#[test]
fn a_synthetic_diff_reaches_the_union_of_its_files_impacts_each_node_at_its_nearest() {
    let survey = Survey::build(&repo_root());
    let changes = vec![
        change("modified", "crates/delulu-diag/src/codes.rs"),
        change("modified", "crates/delulu-check/src/check.rs"),
        change("modified", "docs/design/CONSTITUTION.md"),
        change("deleted", "crates/delulu/src/no_longer_here.rs"),
        change("added", "assets/not-a-file-the-map-reads.bin"),
    ];
    let v = diff_json(&survey, "SYNTHETIC", &changes, MAX_WALK_DEPTH);
    assert_eq!(v["tool"], "delulu-survey");
    assert_eq!(v["verb"], "diff");
    assert_eq!(v["rev"], "SYNTHETIC");

    // Every path the change named is in the answer; the two the map cannot place have no node.
    let changed = v["changed"].as_array().unwrap();
    assert_eq!(changed.len(), changes.len(), "nothing the change named is dropped");
    let node_of: BTreeMap<&str, Option<&str>> =
        changed.iter().map(|c| (c["path"].as_str().unwrap(), c["node"].as_str())).collect();
    assert_eq!(node_of["crates/delulu-diag/src/codes.rs"], Some("mod:crates/delulu-diag/src/codes.rs"));
    assert_eq!(node_of["crates/delulu-check/src/check.rs"], Some("mod:crates/delulu-check/src/check.rs"));
    assert_eq!(node_of["docs/design/CONSTITUTION.md"], Some("doc:docs/design/CONSTITUTION.md"));
    assert_eq!(node_of["crates/delulu/src/no_longer_here.rs"], None, "a deleted file is not in the map now");
    assert_eq!(node_of["assets/not-a-file-the-map-reads.bin"], None);
    assert_eq!(v["mapped"], 3);

    // The entrenched file is named, with its owner and the CODEOWNERS line that says so.
    let entrenched = v["entrenched"].as_array().unwrap();
    assert!(
        entrenched.iter().any(|e| e["node"] == "doc:docs/design/CONSTITUTION.md" && e["owner"].is_string() && e["matched_at"]["line"].is_u64()),
        "{entrenched:#?}"
    );
    assert!(entrenched.iter().all(|e| e["node"] != "mod:crates/delulu-diag/src/codes.rs"), "an ordinary file is not called entrenched");

    // The reach is the union of the three walks, each node at the minimum of its distances.
    let starts = ["mod:crates/delulu-diag/src/codes.rs", "mod:crates/delulu-check/src/check.rs", "doc:docs/design/CONSTITUTION.md"];
    let mut want: BTreeMap<String, u32> = BTreeMap::new();
    for s in starts {
        for r in survey.walk(s, Dir::Incoming, MAX_WALK_DEPTH) {
            if starts.contains(&r.id) {
                continue; // a changed file is where the walk starts, not something it reaches
            }
            let d = want.entry(r.id.to_string()).or_insert(r.depth);
            *d = (*d).min(r.depth);
        }
    }
    let hops = v["hops"].as_array().unwrap();
    let got: BTreeMap<String, u32> =
        hops.iter().map(|h| (h["id"].as_str().unwrap().to_string(), h["depth"].as_u64().unwrap() as u32)).collect();
    assert_eq!(got.len(), hops.len(), "each node once");
    assert_eq!(got, want, "the diff's reach is the union of its files' impacts, nearest first");
    assert!(!want.is_empty(), "the synthetic change reaches something: codes.rs is depended on");
    assert_eq!(v["reached"], hops.len());

    // Every hop is cited and traces back to a file the change touched.
    let ids: BTreeMap<&str, &serde_json::Value> = hops.iter().map(|h| (h["id"].as_str().unwrap(), h)).collect();
    for h in hops {
        assert!(h["via"]["file"].is_string() && h["via"]["line"].as_u64().unwrap() >= 1, "{h}");
        assert!(starts.contains(&h["origin"].as_str().unwrap()), "{h}");
        let from = h["from"].as_str().unwrap();
        assert!(starts.contains(&from) || ids[from]["depth"].as_u64().unwrap() + 1 == h["depth"].as_u64().unwrap(), "{h}");
    }
}

#[test]
fn a_diff_that_touches_nothing_the_map_holds_reaches_nothing() {
    let survey = Survey::build(&repo_root());
    let v = diff_json(&survey, "SYNTHETIC", &[change("deleted", "gone.rs")], MAX_WALK_DEPTH);
    assert_eq!(v["mapped"], 0);
    assert_eq!(v["reached"], 0);
    assert_eq!(v["hops"], serde_json::json!([]));
    assert_eq!(v["entrenched"], serde_json::json!([]));
}

/// The git layer, on a repository this test makes: a modified file, a deleted one, an added one and
/// an untracked one against `HEAD` (the working tree counts, a new file is a change); and a range,
/// which compares commits only, so the untracked file is not part of it.
#[test]
fn git_names_every_kind_of_change_and_a_range_compares_commits_only() {
    let dir = std::env::temp_dir().join(format!("delulu-survey-diff-git-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid", "-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "-q"]);
    std::fs::write(dir.join("kept.rs"), "a").unwrap();
    std::fs::write(dir.join("gone.rs"), "b").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "one"]);
    std::fs::write(dir.join("kept.rs"), "a2").unwrap();
    std::fs::remove_file(dir.join("gone.rs")).unwrap();
    std::fs::write(dir.join("staged new.md"), "c").unwrap();
    git(&["add", "staged new.md"]);
    std::fs::write(dir.join("loose.rs"), "d").unwrap();

    let got = delulu_survey::diff::git_changes(&dir, "HEAD").unwrap();
    let pairs: Vec<(&str, &str)> = got.iter().map(|c| (c.status, c.path.as_str())).collect();
    assert_eq!(
        pairs,
        vec![("deleted", "gone.rs"), ("modified", "kept.rs"), ("untracked", "loose.rs"), ("added", "staged new.md")],
        "a path with a space arrives as itself"
    );

    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "two"]);
    std::fs::write(dir.join("after.rs"), "e").unwrap();
    let range = delulu_survey::diff::git_changes(&dir, "HEAD~1..HEAD").unwrap();
    let pairs: Vec<(&str, &str)> = range.iter().map(|c| (c.status, c.path.as_str())).collect();
    assert_eq!(
        pairs,
        vec![("deleted", "gone.rs"), ("modified", "kept.rs"), ("added", "loose.rs"), ("added", "staged new.md")],
        "a range is between commits: the untracked `after.rs` is not in it"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn survey_bin(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_delulu-survey")).args(args).current_dir(repo_root()).output().unwrap()
}

/// Through the binary, against the real repository: `HEAD` answers (whatever the working tree
/// holds, the shape is the contract), a revision git does not know is exit 2 with git's reason, and
/// an option offered as a revision is refused before git runs — `git diff --output=F` would WRITE F.
#[test]
fn the_binary_answers_a_real_revision_and_refuses_an_option_as_one() {
    let out = survey_bin(&["diff", "HEAD", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!((v["verb"].as_str(), v["rev"].as_str()), (Some("diff"), Some("HEAD")));
    assert!(v["changed"].is_array() && v["hops"].is_array() && v["entrenched"].is_array());

    let out = survey_bin(&["diff", "no-such-revision-zzz", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no-such-revision-zzz"));

    let marker = std::env::temp_dir().join(format!("delulu-survey-diff-output-{}", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let opt = format!("--output={}", marker.display());
    let out = survey_bin(&["diff", &opt]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!marker.exists(), "an option offered as a revision reached git");

    let out = survey_bin(&["diff"]);
    assert_eq!(out.status.code(), Some(2), "a revision is required");
}
