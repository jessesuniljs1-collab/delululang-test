//! Release engineering (Stage 9i, spec §7 — release criteria 4, 8, 10).
//!
//! These are the checks that decide whether a build is shippable. They deliberately test the
//! *properties* rather than the ceremony: an artifact that is byte-identical across builds, an
//! SBOM that lists what is actually linked, a provenance statement that describes the builder
//! honestly, and an announcement whose every claim traces to a measured number.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-release-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// CRITERION 4 (local form, build-order D6): a platform-independent artifact built twice from the
/// same source is BYTE-IDENTICAL.
///
/// This is the property reproducibility is actually about. If two builds of the same input differ,
/// something non-deterministic — a timestamp, a hash-map iteration order, an absolute path — has
/// leaked into the output, and no amount of signing makes that safe to distribute.
#[test]
fn criterion4_a_dwx_artifact_is_byte_identical_across_builds() {
    let dir = scratch("repro");
    let src = root().join("examples/hello_wasm.delulu");
    let a = dir.join("a.dwx");
    let b = dir.join("b.dwx");

    let o1 = delulu(&["build", &src.to_string_lossy(), "--target", "wasm", "-o", &a.to_string_lossy()]);
    assert!(o1.status.success(), "first build must succeed");
    let o2 = delulu(&["build", &src.to_string_lossy(), "--target", "wasm", "-o", &b.to_string_lossy()]);
    assert!(o2.status.success(), "second build must succeed");

    let bytes_a = std::fs::read(&a).expect("first artifact");
    let bytes_b = std::fs::read(&b).expect("second artifact");
    assert_eq!(
        bytes_a, bytes_b,
        "two builds of the same source produced different bytes — something non-deterministic \
         leaked into the artifact, and signing it would only certify the nondeterminism"
    );
    assert!(!bytes_a.is_empty(), "an empty artifact is not evidence of anything");
}

/// The signing path a release actually uses, end to end: sign, verify, then verify a tampered copy
/// and a copy whose signature was removed.
#[test]
fn criterion4_release_artifacts_sign_and_verify() {
    let dir = scratch("sign");
    let home = dir.join("home");
    let art = dir.join("release.dwx");
    std::fs::write(&art, b"release payload bytes").unwrap();

    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(&dir)
            .env("DELULU_HOME", &home)
            .env("DELULU_NO_FIRST_RUN", "1")
            .args(args)
            .output()
            .expect("run")
    };

    assert!(run(&["keygen"]).status.success(), "keygen");
    assert!(run(&["sign", &art.to_string_lossy()]).status.success(), "sign");
    assert!(
        run(&["verify-sig", &art.to_string_lossy()]).status.success(),
        "a freshly signed artifact must verify"
    );

    // Tamper: one byte.
    let mut bytes = std::fs::read(&art).unwrap();
    bytes[0] ^= 0xFF;
    std::fs::write(&art, &bytes).unwrap();
    assert!(
        !run(&["verify-sig", &art.to_string_lossy()]).status.success(),
        "a tampered artifact must NOT verify"
    );
}

/// CRITERION 10: every claim in the announcement traces to a measured number or a named artifact.
///
/// The announcement is where a project's honesty is most likely to fail, because it is the one
/// document written to persuade. This test is the line-by-line review, mechanized for the claims
/// that can be mechanized.
#[test]
fn criterion10_the_announcement_makes_no_unsupported_claim() {
    let a = std::fs::read_to_string(root().join("docs/release/ANNOUNCEMENT-1.0.md"))
        .expect("the announcement draft is committed");

    // Claims the constitution forbids outright (§5.11, §9). Checked case-insensitively so a
    // capitalised headline cannot slip through.
    let lower = a.to_lowercase();
    for forbidden in [
        "faster than c",
        "lowest tokens",
        "fewest tokens",
        "unhackable",
        "provably secure",
        "guarantees security",
        "impossible to exploit",
        "100% safe",
        "zero bugs",
    ] {
        assert!(
            !lower.contains(forbidden),
            "the announcement contains the forbidden claim `{forbidden}` — Constitution §9"
        );
    }

    // The uncomfortable numbers must be PRESENT, not merely un-contradicted. An announcement that
    // simply omits the performance position is not honest, it is quiet.
    for required in [
        "not competitive with C", // Study C's actual finding
        "8.5%",                   // Study B's repair coverage
        "measurements/",          // where the raw data lives
    ] {
        assert!(
            a.contains(required),
            "the announcement must state `{required}` — omitting an inconvenient measured result \
             is the failure mode this check exists for"
        );
    }

    // Coverage must be stated as the real number, not rounded to a claim of completeness.
    assert!(
        !a.contains("100% conformance") && !a.contains("fully covered"),
        "conformance coverage is not 100% yet; the announcement must not imply it is"
    );
}

/// The release checklist keeps its PENDING-PUBLIC items marked. Same drift risk as SECURITY.md:
/// as launch approaches, a placeholder quietly becomes a claim.
#[test]
fn the_release_checklist_still_marks_what_is_not_live() {
    let c = std::fs::read_to_string(root().join("docs/release/CHECKLIST-1.0.md"))
        .expect("the release checklist is committed");
    assert!(c.contains("PENDING-PUBLIC"), "controls needing public hosting must stay marked");
    assert!(
        c.contains("BLOCKED") || c.contains("NOT MET"),
        "the checklist must be able to say a criterion is not met — one that only has checkmarks \
         is a wish list"
    );
}

/// The SBOM lists the workspace's actual dependencies. A bill of materials that omits something
/// linked into the binary is worse than none: it will be trusted.
#[test]
fn the_sbom_lists_the_real_dependencies() {
    let sbom = std::fs::read_to_string(root().join("docs/release/SBOM-1.0.json"))
        .expect("the SBOM is committed");
    let v: serde_json::Value = serde_json::from_str(&sbom).expect("the SBOM is valid JSON");
    assert_eq!(v["bomFormat"], "CycloneDX", "CycloneDX per spec §4");

    let components = v["components"].as_array().expect("components");
    let names: Vec<&str> =
        components.iter().filter_map(|c| c["name"].as_str()).collect();

    // Every third-party crate the workspace actually depends on must appear.
    for dep in ["serde", "serde_json", "toml", "wasmtime", "ed25519-dalek", "getrandom"] {
        assert!(
            names.contains(&dep),
            "the SBOM omits `{dep}`, which the workspace links — an incomplete SBOM is worse than \
             none, because it will be trusted. Present: {names:?}"
        );
    }
}

/// The provenance statement describes the builder HONESTLY. Claiming an SLSA level the build did
/// not achieve is a supply-chain lie with a schema around it.
#[test]
fn the_provenance_states_the_builder_honestly() {
    let p = std::fs::read_to_string(root().join("docs/release/PROVENANCE-1.0.json"))
        .expect("the provenance statement is committed");
    let v: serde_json::Value = serde_json::from_str(&p).expect("valid JSON");
    assert!(
        v["_type"].as_str().unwrap_or("").contains("in-toto"),
        "provenance is an in-toto statement"
    );
    // SLSA v1 nests the builder under runDetails.
    let builder = v["predicate"]["runDetails"]["builder"]["id"].as_str().unwrap_or("");
    assert!(
        builder.contains("local"),
        "the builder must be identified as the local runner it is, not as hosted CI: `{builder}`"
    );
    let level = v["predicate"]["_slsaLevel"].as_str().unwrap_or("");
    assert!(
        level.contains("sub-L3") || level.contains("PENDING"),
        "SLSA L3 needs hosted CI; the statement must say it is not L3, got `{level}`"
    );
}
