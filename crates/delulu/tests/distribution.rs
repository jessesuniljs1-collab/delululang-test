//! The install story, gated so it cannot rot quietly.
//!
//! `INSTALL.md` and the `INSTALL.txt` written into every archive are **two statements of the same
//! thing**, and this campaign's most repeated finding is that a claim restated by hand drifts from
//! the thing it restates — seven times in code, and twice in prose (C80's six-document contribution
//! policy, C81's workspace size). An install page is the worst place for it: a reader following a
//! stale instruction concludes the toolchain is broken, and they are not wrong to.
//!
//! So these tests do not check that the documents are *well written*. They check that the commands
//! they tell a newcomer to type are the commands the packaging script actually supports, and that
//! neither document claims a capability the project deliberately does not have.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    // tests/ -> crates/delulu -> crates -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").canonicalize().expect("repo root")
}

fn read(rel: &str) -> String {
    let p = root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} must be readable: {e}", p.display()))
}

/// The portable build is `--no-default-features`, and that is load-bearing rather than tidy: the
/// default build embeds CPython and imports a *specific* interpreter, so it cannot start on a
/// machine without that exact version. If the packaging script ever drops the flag, the archive
/// silently becomes undistributable and nothing else would notice.
#[test]
fn the_packaging_script_ships_the_portable_build() {
    let s = read("scripts/package-toolchain.sh");
    assert!(
        s.contains("--no-default-features"),
        "the shipped binary must be the Python-less build, or it will not start on a machine \
         without the exact CPython it was linked against"
    );
    assert!(
        s.contains("sha256sum") && s.contains("SHA256SUMS"),
        "an archive without checksums asks to be trusted for no reason"
    );
    for owed in ["LICENSE", "NOTICE", "TRADEMARK.md"] {
        assert!(s.contains(owed), "the archive must carry {owed} — a binary alone is not a distribution");
    }
}

/// The two install documents must agree on the commands they teach.
#[test]
fn the_install_page_and_the_archive_agree_on_the_first_commands() {
    let md = read("INSTALL.md");
    let sh = read("scripts/package-toolchain.sh");

    // The example named in the walkthrough must be the same one on both sides, and must be an
    // example the archive actually contains.
    assert!(md.contains("hello_wasm.delulu"), "INSTALL.md must name the example it walks through");
    assert!(
        sh.contains("hello_wasm.delulu"),
        "INSTALL.txt (written by the packaging script) must walk through the SAME example as \
         INSTALL.md, or the archive contradicts the page that sent the reader to it"
    );
    assert!(
        root().join("examples/hello_wasm.delulu").exists(),
        "both documents name an example that must exist in the tree"
    );

    // Both must teach the refusal, because it is the one step a newcomer would otherwise report as
    // a bug. A page that hides it produces a bug report; a page that explains it produces a user.
    for (name, text) in [("INSTALL.md", &md), ("the packaging script", &sh)] {
        assert!(
            text.contains("DL0703"),
            "{name} must show the refusal by code — it is the step that looks like a failure"
        );
        assert!(
            text.contains("--grant console"),
            "{name} must show the grant that resolves the refusal"
        );
    }
}

/// Neither document may promise the one install path the project deliberately does not offer.
///
/// `cargo install delulu` from crates.io cannot work: `delulu` depends on nine sibling crates by
/// path, every one of them `publish = false` because `STABILITY.md` §2 promises the Rust crates are
/// not a stable interface. The temptation is to quietly publish them so one command gets shorter,
/// which trades a written promise for a convenience.
#[test]
fn no_document_promises_an_install_path_that_cannot_work() {
    for f in ["INSTALL.md", "README.md", "docs/for-agents.md"] {
        for (n, line) in read(f).lines().enumerate() {
            let l = line.trim();
            // `cargo install --path …` is the real, working, from-source route.
            if l.contains("cargo install") && !l.contains("--path") {
                // Only a line that PROMISES it is a defect; a line explaining its absence is the
                // point of §3 and must stay allowed.
                let explains = ["not available", "cannot work", "does not exist", "deliberately"]
                    .iter()
                    .any(|m| l.to_lowercase().contains(m));
                assert!(
                    explains,
                    "{f}:{} promises `cargo install` from crates.io, which cannot work while every \
                     crate but the CLI is publish = false:\n  {l}",
                    n + 1
                );
            }
        }
    }
}

/// The publish posture the whole §3 argument rests on: exactly one crate may be published.
#[test]
fn exactly_one_crate_is_publishable_which_is_what_makes_section_three_true() {
    let mut publishable = Vec::new();
    let crates_dir = root().join("crates");
    for e in std::fs::read_dir(&crates_dir).expect("crates/ must be readable").flatten() {
        let manifest = e.path().join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).expect("manifest readable");
        if !text.lines().any(|l| l.trim().starts_with("publish") && l.contains("false")) {
            publishable.push(e.file_name().to_string_lossy().to_string());
        }
    }
    assert_eq!(
        publishable,
        vec!["delulu".to_string()],
        "INSTALL.md §3 argues that only the CLI is publishable; if that stops being true the \
         argument stops being true with it"
    );
}
