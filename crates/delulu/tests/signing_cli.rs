//! Stage 8, phase 8h: signing + registry client groundwork (criterion 9) — driven
//! through the real binary. keygen → sign → verify-sig round-trips; tamper detection;
//! `publish --dry-run` catches an authority-widening minor bump (DL1003) against a local
//! index fixture; `add` renders the authority summary from the index line alone.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_sign_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn delulu(dir: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_HOME", home)
        .output()
        .expect("run delulu")
}

/// THE UNSIGNED CASE — the skip branch. An artifact with no signature at all must FAIL
/// verification, and it must fail through the **exit code**, not only in the rendered verdict.
///
/// Added by the DRILL-001 rehearsal (see `docs/security/DRILL-001.md`). A staged bypass that
/// returned 0 for an unsigned artifact passed the entire 875-test suite: every existing signing
/// test asserted on the *verdict string* and none on the exit code for the missing-signature path.
/// A caller gating on `$?` — which is every shell script and CI job on earth — would have accepted
/// an unsigned artifact.
///
/// This is the shape the house rule warns about: the rule was enforced everywhere except the
/// branch where the checker had nothing to check.
#[test]
fn an_unsigned_artifact_fails_verification_by_exit_code() {
    let dir = tmp("unsigned");
    let home = dir.join("home");
    let art = dir.join("payload.dwx");
    std::fs::write(&art, b"contents that nobody ever signed").unwrap();

    let o = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap()]);
    let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));

    assert_eq!(
        o.status.code(),
        Some(1),
        "an UNSIGNED artifact must exit nonzero — a caller gating on the exit code would otherwise \
         accept it:\n{text}"
    );
    assert!(text.contains("unsigned"), "the verdict must name the reason: {text}");

    // And the machine surface must agree with the exit code. A JSON envelope that says one thing
    // while the exit code says another is a bypass waiting to be found.
    let j = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--json"]);
    assert_eq!(j.status.code(), Some(1), "--json must fail identically");
    let v: Value = serde_json::from_slice(&j.stdout).expect("verify-sig --json is valid JSON");
    assert_ne!(v["verdict"], "valid", "an unsigned artifact is never `valid`: {v}");
}

#[test]
fn keygen_sign_verify_round_trip_and_tamper_detection() {
    let dir = tmp("roundtrip");
    let home = dir.join("home");

    // keygen mints a key + prints the public identity.
    let kg = delulu(&dir, &home, &["keygen", "--json"]);
    assert!(kg.status.success(), "{}", String::from_utf8_lossy(&kg.stderr));
    let kgv: Value = serde_json::from_slice(&kg.stdout).unwrap();
    let signer = kgv["public_key"].as_str().unwrap().to_string();
    assert_eq!(signer.len(), 64, "ed25519 pubkey is 32 bytes hex");

    // An "artifact" (any bytes — sign is format-agnostic; here a stand-in .dwx).
    let art = dir.join("thing.dwx");
    std::fs::write(&art, b"\0asm\x01\x00\x00\x00some artifact bytes").unwrap();

    let sg = delulu(&dir, &home, &["sign", art.to_str().unwrap()]);
    assert!(sg.status.success(), "{}", String::from_utf8_lossy(&sg.stderr));
    assert!(dir.join("thing.dwx.sig").exists(), "detached signature written");

    // verify-sig: valid, by the same signer; --key pins the identity.
    let vf = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--key", &signer, "--json"]);
    assert!(vf.status.success(), "{}", String::from_utf8_lossy(&vf.stderr));
    let vfv: Value = serde_json::from_slice(&vf.stdout).unwrap();
    assert_eq!(vfv["verdict"], "valid");
    assert_eq!(vfv["signer"], signer);

    // A wrong pinned key is rejected even though the signature itself is valid.
    let wrong = "0".repeat(64);
    let vf2 = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--key", &wrong]);
    assert_eq!(vf2.status.code(), Some(1), "pinned-key mismatch fails");

    // Tamper the artifact ⇒ DL1705.
    std::fs::write(&art, b"\0asm\x01\x00\x00\x00TAMPERED bytes now").unwrap();
    let vf3 = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--json"]);
    assert_eq!(vf3.status.code(), Some(1));
    let vf3v: Value = serde_json::from_slice(&vf3.stdout).unwrap();
    assert_eq!(vf3v["verdict"], "invalid");
    assert_eq!(vf3v["code"], "DL1705");
}

/// Scaffold a package that performs `Write` at the given version.
fn scaffold_pkg(dir: &Path, version: &str) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        format!(
            "[package]\nname = \"widget\"\nversion = \"{version}\"\n\n[authority]\neffects = [\"Write\"]\n"
        ),
    )
    .unwrap();
    std::fs::write(
        src.join("main.delulu"),
        "module widget\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(\"hi\")\n}\n",
    )
    .unwrap();
}

#[test]
fn criterion9_publish_catches_widening_bump_and_add_reads_the_index_line() {
    let dir = tmp("registry");
    let home = dir.join("home");
    let index = dir.join("index");
    std::fs::create_dir_all(&index).unwrap();

    // A prior index line: widget 1.0.0 was PURE (no effects).
    std::fs::write(
        index.join("widget"),
        "{\"name\":\"widget\",\"version\":\"1.0.0\",\"effects\":[],\"capabilities\":[],\"secrets\":[],\"yanked\":false}\n",
    )
    .unwrap();

    // The package at 1.1.0 now performs Write — a minor bump that WIDENS authority.
    let pkg = dir.join("widget");
    scaffold_pkg(&pkg, "1.1.0");
    let o = delulu(
        &dir,
        &home,
        &["publish", "--dry-run", pkg.to_str().unwrap(), "--index", index.to_str().unwrap(), "--json"],
    );
    assert_eq!(o.status.code(), Some(1), "widening minor bump must be refused");
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["code"], "DL1003", "the semver-authority law fires: {v}");

    // A MAJOR bump to 2.0.0 is allowed (widening authority is semver-major).
    scaffold_pkg(&pkg, "2.0.0");
    let ok = delulu(
        &dir,
        &home,
        &["publish", "--dry-run", pkg.to_str().unwrap(), "--index", index.to_str().unwrap(), "--json"],
    );
    assert!(ok.status.success(), "a major bump publishes: {}", String::from_utf8_lossy(&ok.stdout));
    let okv: Value = serde_json::from_slice(&ok.stdout).unwrap();
    let index_line: Value = serde_json::from_str(okv["index_line"].as_str().unwrap()).unwrap();
    let effects: Vec<String> = index_line["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    assert!(effects.iter().any(|e| e == "Write"), "the new index line carries the authority summary: {okv}");

    // `add` renders the authority from the index line ALONE — no package, no download.
    let add = delulu(&dir, &home, &["add", "widget", "--index", index.to_str().unwrap(), "--json"]);
    assert!(add.status.success(), "{}", String::from_utf8_lossy(&add.stderr));
    let addv: Value = serde_json::from_slice(&add.stdout).unwrap();
    assert_eq!(addv["version"], "1.0.0", "reads the latest index line");
    assert_eq!(addv["from"], "index-line-only", "no download happened");
    // The authority summary is present without ever touching a package on disk.
    assert!(addv["authority"]["effects"].is_array());
}

#[test]
fn add_on_an_unknown_package_is_dl1706() {
    let dir = tmp("unknown");
    let home = dir.join("home");
    let index = dir.join("index");
    std::fs::create_dir_all(&index).unwrap();
    let o = delulu(&dir, &home, &["add", "ghost", "--index", index.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["code"], "DL1706");
}

/// An unsigned artifact and a badly-signed one must report DIFFERENT codes (campaign C38).
///
/// Both used to be DL1705, "signature verification failed" — which for an unsigned artifact is not
/// even true, because nothing was verified. The distinction is the one that matters most on this
/// surface: no signature is a POLICY question and is often benign, while a signature that fails to
/// verify is an ATTACK INDICATOR — tampered content, or the wrong key. A caller handed one code for
/// both cannot tell them apart, and the code is the contract while the message is not.
///
/// This was not a new principle. The project had already RULED it (Stage-6 deviation 8, whose own
/// test asserts "badly-signed vs unsigned are DIFFERENT faults") and the plugin path implements it
/// with DL1510 vs DL1511. Only the detached path had never followed the rule.
#[test]
fn an_unsigned_artifact_and_a_bad_signature_are_different_codes() {
    let dir = tmp("c38_codes");
    let home = tmp("c38_home");
    std::fs::write(dir.join("a.txt"), b"the artifact").unwrap();
    std::fs::write(dir.join("b.txt"), b"the artifact").unwrap();
    assert!(delulu(&dir, &home, &["keygen"]).status.success(), "keygen");
    assert!(delulu(&dir, &home, &["sign", "a.txt"]).status.success(), "sign");

    // (1) UNSIGNED: no `.sig` accompanies b.txt.
    let o = delulu(&dir, &home, &["verify-sig", "b.txt", "--json"]);
    assert_ne!(o.status.code(), Some(0), "an unsigned artifact must not verify");
    let v: Value = serde_json::from_slice(&o.stdout).expect("one JSON object");
    assert_eq!(v["code"], "DL1511", "unsigned is DL1511, not a verification failure: {v}");
    assert_eq!(v["verdict"], "unsigned", "{v}");

    // (2) BADLY SIGNED: a real signature over this artifact, one signature byte flipped.
    let mut sig = std::fs::read(dir.join("a.txt.sig")).unwrap();
    assert_eq!(sig.len(), 96, "the detached signature is pubkey(32) || sig(64)");
    sig[64] ^= 0xff;
    std::fs::write(dir.join("b.txt.sig"), &sig).unwrap();
    let o = delulu(&dir, &home, &["verify-sig", "b.txt", "--json"]);
    assert_ne!(o.status.code(), Some(0), "a bad signature must not verify");
    let v: Value = serde_json::from_slice(&o.stdout).expect("one JSON object");
    assert_eq!(v["code"], "DL1705", "a present-but-invalid signature stays DL1705: {v}");
    assert_eq!(v["verdict"], "invalid", "{v}");

    // (3) And the valid case still verifies, so neither refusal is a blanket "always refuse".
    let o = delulu(&dir, &home, &["verify-sig", "a.txt", "--json"]);
    assert_eq!(o.status.code(), Some(0), "a correctly signed artifact must verify");
    let v: Value = serde_json::from_slice(&o.stdout).expect("one JSON object");
    assert_eq!(v["verdict"], "valid", "{v}");
    assert!(v.get("code").is_none(), "a valid verification carries no failure code: {v}");
}

/// A signature over a DIFFERENT artifact must not verify — the property that makes a detached
/// signature mean anything at all. Distinct from a corrupted signature: here the signature is
/// perfectly well-formed and correctly signed, just not over this file.
#[test]
fn a_signature_from_another_artifact_does_not_verify() {
    let dir = tmp("c38_swap");
    let home = tmp("c38_swap_home");
    std::fs::write(dir.join("a.txt"), b"artifact A").unwrap();
    std::fs::write(dir.join("b.txt"), b"artifact B, different bytes").unwrap();
    assert!(delulu(&dir, &home, &["keygen"]).status.success());
    assert!(delulu(&dir, &home, &["sign", "a.txt"]).status.success());
    std::fs::copy(dir.join("a.txt.sig"), dir.join("b.txt.sig")).unwrap();
    let o = delulu(&dir, &home, &["verify-sig", "b.txt", "--json"]);
    assert_ne!(o.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["code"], "DL1705", "a valid signature over other bytes is a verification failure: {v}");
}
