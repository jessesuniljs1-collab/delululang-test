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

/// PS-B-02: the download HAS the network client. `--no-default-features` (needed to leave Python out)
/// drops `net` too, so the build must name it back — and the page that teaches the from-source
/// portable install must teach the same spelling, or a reader following it gets a delulu that
/// refuses every `http.get` and has no way to guess why.
///
/// This gate exists because the first version of PS-B shipped exactly that defect for a commit: the
/// CLI crate did not forward the runtime's `net` feature at all, so only a whole-workspace build had
/// a client, by accident of feature unification.
#[test]
fn the_shipped_binary_and_the_portable_install_have_the_network_client() {
    let sh = read("scripts/package-toolchain.sh");
    assert!(
        sh.contains("cargo build --release -p delulu --no-default-features --features net"),
        "the portable build must name `--features net`, or the download answers every http.get with Refused"
    );
    let md = read("INSTALL.md");
    assert!(
        md.contains("cargo install --path crates/delulu --no-default-features --features net"),
        "INSTALL.md must teach the same portable spelling the archive is built with"
    );
    let cli = read("crates/delulu/Cargo.toml");
    assert!(
        cli.contains("net = [\"delulu-runtime/net\"]") && cli.contains("default = [\"python\", \"net\"]"),
        "the CLI crate must forward the runtime's `net` feature and enable it by default"
    );
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

/// P5 (D-V2-42): the archive carries what the binary looks for beside itself — the surface morphs,
/// where `morph_file.rs` searches last, and the Agent Skill — so a person or an agent with nothing
/// installed but the unpacked archive has both.
#[test]
fn the_archive_ships_the_morphs_and_the_skill() {
    let sh = read("scripts/package-toolchain.sh");
    assert!(sh.contains(r#"cp morphs/*.toml "$STAGE/morphs/""#), "the archive must ship the morphs beside bin/");
    assert!(sh.contains(r#"skills/delulu/SKILL.md "$STAGE/skills/delulu/""#), "the archive must ship the Agent Skill");
    assert!(sh.contains(r#"cp -r examples/guide "$STAGE/examples/""#), "the archive must ship the guided tour GETTING_STARTED walks");
    let morph = read("crates/delulu/src/morph_file.rs");
    assert!(morph.contains(r#"root.join("morphs")"#), "the binary must look for the morphs where the archive puts them");
    assert!(
        std::fs::read_dir(root().join("morphs")).expect("morphs/").flatten().any(|e| e.path().extension().is_some_and(|x| x == "toml")),
        "there must be morphs to ship"
    );
}

/// P5-05: INSTALL.md §1 carries exactly one `install-gate` block, and it is the whole first-run
/// path — verify, unpack, PATH, a new program, its test, its authority, a run and a sandboxed run.
/// `scripts/check-install.sh` executes that block against a real archive (the release workflow does,
/// on every target); this pins that the block still walks the path, so the gate cannot be hollowed
/// out by trimming the page.
#[test]
fn the_install_page_carries_the_gate_the_release_workflow_runs() {
    let md = read("INSTALL.md");
    assert_eq!(md.matches("```sh install-gate\n").count(), 1, "exactly one install-gate block");
    let block: String = md
        .split("```sh install-gate\n")
        .nth(1)
        .and_then(|rest| rest.split("\n```").next())
        .expect("the block closes")
        .to_string();
    for step in [
        "sha256sum -c delulu-<version>-<target>.tar.gz.sha256",
        "tar -xzf delulu-<version>-<target>.tar.gz",
        "sha256sum -c SHA256SUMS",
        "export PATH=",
        "delulu --version",
        "delulu new hello",
        "delulu test .",
        "delulu authority . --grants",
        "delulu run . --grant console",
        "delulu run . --grant console --sandbox",
    ] {
        assert!(block.contains(step), "the install gate must still do `{step}`:\n{block}");
    }
    let gate = read("scripts/check-install.sh");
    assert!(gate.contains("install-gate") && gate.contains("eval \"$BLOCK\""), "the gate runs the page's block");
    assert!(gate.contains("set -euo pipefail"), "a step that fails must fail the gate");
    let wf = read(".github/workflows/release.yml");
    assert!(wf.contains("bash scripts/check-install.sh"), "the release workflow runs the gate");
    assert!(wf.contains("bash scripts/package-toolchain.sh"), "the release workflow builds with the same script");
}

/// D-NE-7 stays the owner's: the release workflow builds, attests and tests on every run, and
/// PUBLISHES nothing unless the run came from a `v*` tag AND the owner switched `RELEASES` on — and
/// even then only a draft, which a person publishes.
#[test]
fn the_release_workflow_publishes_only_a_draft_and_only_when_the_owner_says() {
    let wf = read(".github/workflows/release.yml");
    assert!(
        wf.contains("if: startsWith(github.ref, 'refs/tags/v') && vars.RELEASES == 'on'"),
        "the release job must be gated on a tag AND the owner's switch"
    );
    assert_eq!(wf.matches("gh release create").count(), 1, "one place creates a release");
    let create = wf.lines().find(|l| l.contains("gh release create")).unwrap();
    assert!(create.contains("--draft"), "a release is a draft a person publishes: {create}");
    assert!(wf.contains("actions/attest-build-provenance"), "a release's archives are attested");
    // A dry run distributes nothing: the upload and the (public, permanent) attestation are a tag's.
    for step in ["uses: actions/attest-build-provenance", "uses: actions/upload-artifact"] {
        let at = wf.find(step).unwrap_or_else(|| panic!("the workflow has `{step}`"));
        let before = &wf[..at];
        let step_start = before.rfind("- name:").expect("the step is named");
        assert!(
            wf[step_start..at].contains("if: startsWith(github.ref, 'refs/tags/v')"),
            "`{step}` must run only for a `v*` tag: {}",
            &wf[step_start..at]
        );
    }
    for target in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-pc-windows-msvc"] {
        assert!(wf.contains(target), "the workflow builds {target}, which INSTALL.md names");
        assert!(read("INSTALL.md").contains(target), "INSTALL.md names {target}, which the workflow builds");
    }
    // Nothing in the workflow pushes anywhere: no crates.io, no second repository.
    for never in ["cargo publish", "git push", "CARGO_REGISTRY_TOKEN"] {
        assert!(!wf.contains(never), "the release workflow must not `{never}`");
    }
}

/// P5-04: `delulu --version --json` names the target the binary was compiled for and the commit it was
/// built from — `null` when the build was not told, never a guess.
#[test]
fn the_binary_names_its_target_and_its_commit_or_says_it_was_not_told() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(["--version", "--json"])
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .expect("the binary runs");
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    let target = v["target"].as_str().expect("a target");
    let arch = std::env::consts::ARCH;
    assert!(target.starts_with(arch), "`{target}` must be this machine's {arch} build");
    match &v["commit"] {
        serde_json::Value::Null => {}
        serde_json::Value::String(c) => {
            let hex = c.strip_suffix("-dirty").unwrap_or(c);
            assert!(hex.len() == 40 && hex.chars().all(|ch| ch.is_ascii_hexdigit()), "a commit, not a guess: {c}");
        }
        other => panic!("commit must be a string or null: {other}"),
    }
}

/// P5-04: a release says what changed. The version this tree builds has its own `## [X.Y.Z]` section
/// in CHANGELOG.md, and the release workflow refuses a `vX.Y.Z` tag without one before building
/// anything for it — so bumping the version is the moment the section gets written.
#[test]
fn the_version_has_its_changelog_section_and_a_tag_without_one_is_refused() {
    let changelog = read("CHANGELOG.md");
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        changelog.lines().any(|l| l.starts_with(&format!("## [{version}]"))),
        "CHANGELOG.md has no `## [{version}]` section for the version this tree builds"
    );
    let wf = read(".github/workflows/release.yml");
    assert!(wf.contains("The tag has its CHANGELOG section"), "the release workflow checks the tag's section");
}
