//! The map must be what a clean checkout would produce.
//!
//! `freshness.rs` holds the committed map to the tree it was built from. That tree is whatever sits
//! on the machine that ran the build — including files git ignores, which exist only where a build
//! ran. Map one of those and the committed map depends on that machine: every clean checkout
//! disagrees with it, and the freshness gate fails everywhere except the one place that could have
//! caught it.
//!
//! That happened. The packaged VS Code extension — the `.vsix` that `npm run package` writes into
//! `editors/vscode/`, gitignored build output — was a node in the committed map until the first CI run (2026-09-14) built the map
//! from a fresh clone, counted 1119 nodes where the development machine had written 1120, and failed
//! `the_committed_map_matches_the_tree` and three `delulu doctor` tests on every runner.
//!
//! So this test asks git rather than a list: of every file the Survey reads, which does git ignore?
//! The answer must be none. A new kind of build output fails here — on the machine that has it —
//! before it can reach a commit.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn repo_root() -> PathBuf {
    // `crates/delulu-survey` → the workspace root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn the_survey_reads_no_file_that_git_ignores() {
    let root = repo_root();

    // A source archive has no `.git` and so no ignore rules to consult. That is the only case in
    // which this test holds nothing, and it says so rather than passing quietly.
    let in_git = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_git {
        eprintln!("SKIPPED: {} is not a git work tree, so there are no ignore rules to check against", root.display());
        return;
    }

    let files = delulu_survey::scan::walk(&root);
    assert!(!files.is_empty(), "the Survey read no files at all — this test would pass while checking nothing");

    let mut child = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["check-ignore", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("git is on PATH in any work tree this test runs in");
    {
        let mut stdin = child.stdin.take().expect("stdin of git check-ignore");
        for f in &files {
            writeln!(stdin, "{}", f.rel).expect("write a path to git check-ignore");
        }
    }
    let out = child.wait_with_output().expect("git check-ignore finished");

    // Exit 0: at least one path is ignored. 1: none is. Anything else is git failing, which must not
    // read as "nothing ignored" — a gate that passes when its tool breaks is not a gate.
    assert!(
        matches!(out.status.code(), Some(0) | Some(1)),
        "git check-ignore failed ({:?}): {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let ignored: Vec<String> = String::from_utf8_lossy(&out.stdout).lines().map(str::to_string).collect();
    assert!(
        ignored.is_empty(),
        "the Survey reads {} file(s) that git ignores, so the committed map would depend on this \
         machine and disagree with every clean checkout:\n  {}\n\nThese are build outputs. Exclude \
         them in `delulu_survey::EXCLUDED_DIRS` or `EXCLUDED_FILE_EXTENSIONS`, with the reason.",
        ignored.len(),
        ignored.join("\n  ")
    );
}

/// A checkout made with `git worktree add` (or a submodule) holds `.git` as a FILE — one line,
/// `gitdir: …` — not a directory, and `git check-ignore` never names it, so the test above cannot see
/// it. Mapped, it is a node no ordinary clone has: a map built in a worktree (the routine writes
/// slices in one since run 25) disagrees with every runner, and the pre-commit hook refused the first
/// commit made from one (2026-10-18). Version-control internals are excluded in either shape.
#[test]
fn a_worktrees_git_file_is_not_mapped() {
    let dir = std::env::temp_dir().join(format!("survey-worktree-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    std::fs::write(dir.join(".git"), "gitdir: /elsewhere/.git/worktrees/x\n").unwrap();
    std::fs::write(dir.join("docs/README.md"), "# a document\n").unwrap();
    let files: Vec<String> = delulu_survey::scan::walk(&dir).into_iter().map(|f| f.rel).collect();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(files.contains(&"docs/README.md".to_string()), "the walk read the tree at all: {files:?}");
    assert!(!files.iter().any(|f| f == ".git"), "a worktree's `.git` file was mapped: {files:?}");
}
