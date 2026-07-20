//! Stage 10 phase 10i — post-quantum signatures wired into the CLI (Track G, spec §8,
//! invariant 51), driven through the real binary.
//!
//! `delulu-runtime/src/pqc.rs` already proves the envelope format, the policy gates, and the
//! DL1908/DL1910 refusals as a library (its own unit tests are the spec these flags implement).
//! What this file proves is that the SAME guarantees reach a human at a terminal:
//!
//! - `delulu sign --hybrid` without `--unstable` refuses, and leaves no `.sig` file behind.
//! - `delulu sign --hybrid --unstable` produces a real `dlsig1` envelope that verifies.
//! - `delulu verify-sig --require-hybrid` refuses a classical-only artifact — in EITHER shape
//!   classical-only comes in, because a check that only recognised one shape would pass every
//!   artifact signed before this phase straight through the gate meant to catch them.
//! - `delulu verify-sig` with none of the new flags is untouched: a plain classical signature
//!   verifies exactly as it always has (`signing_cli.rs`'s existing coverage is the other half of
//!   this guarantee; this file re-pins it against the NEW code path `verify-sig` now runs
//!   through, `pqc::verify`, rather than the old direct call to `verify_detached`).
//! - A hybrid signature handed to a plain `verify-sig` (no `--require-hybrid`) is STILL refused
//!   without `--unstable` — the gate is tripped by what the envelope carries, not by policy.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_pqc_{tag}_{}", std::process::id()));
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

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// `keygen`, through the real CLI — every fixture that needs an actual `delulu sign` round trip
/// mints its OWN key under a private `DELULU_HOME`, so tests never share or race on a keyring.
fn keygen(dir: &Path, home: &Path) -> String {
    let o = delulu(dir, home, &["keygen", "--json"]);
    assert!(o.status.success(), "keygen: {}", stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).expect("keygen --json is valid JSON");
    v["public_key"].as_str().expect("keygen prints public_key").to_string()
}

/// A fixed seed for the ONE fixture below built directly against `delulu_runtime::pqc` instead of
/// through the CLI: a `dlsig1` envelope that declares only `ed25519`. The CLI itself never emits
/// that shape (its classical default stays the legacy 96-byte blob, deliberately — see
/// `signing.rs`'s `cmd_sign`), so it does not exist unless something builds it directly.
const SEED: [u8; 32] = [9u8; 32];

/// **DL1910 on the sign side, through the CLI.** `--hybrid` alone is not consent to use
/// unvalidated cryptography; `--unstable` is the deliberate opt-in `pqc::sign_hybrid` demands.
/// Without it, `sign --hybrid` must refuse — AND must not leave a `.sig` file behind. A signature
/// file on disk is itself a claim that signing happened; a stray file left by a refused command
/// would be exactly the kind of ambient trust this gate exists to prevent, and nothing downstream
/// would know to distrust it just because the command that wrote it printed an error.
#[test]
fn sign_hybrid_without_unstable_refuses_and_writes_no_signature_file() {
    let dir = tmp("sign-no-unstable");
    let home = dir.join("home");
    keygen(&dir, &home);

    let art = dir.join("payload.dwx");
    std::fs::write(&art, b"industrial artifact bytes").unwrap();

    let o = delulu(&dir, &home, &["sign", art.to_str().unwrap(), "--hybrid"]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1910"), "{err}");
    assert!(
        !dir.join("payload.dwx.sig").exists(),
        "a refused sign must not leave a signature file for something else to trust"
    );

    // The JSON channel must agree with the exit code — the DRILL-001 lesson from
    // `signing_cli.rs`'s `an_unsigned_artifact_fails_verification_by_exit_code`, reapplied here:
    // a caller gating on `$?` and a caller parsing JSON must see the SAME verdict.
    let j = delulu(&dir, &home, &["sign", art.to_str().unwrap(), "--hybrid", "--json"]);
    assert_eq!(j.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&j.stdout).expect("sign --json is valid JSON even when refused");
    assert_eq!(v["code"], "DL1910", "{v}");
    assert!(!dir.join("payload.dwx.sig").exists(), "the --json retry must not write one either");
}

/// The happy path, once a human has opted in on both ends: `sign --hybrid --unstable` produces a
/// `dlsig1` envelope carrying BOTH algorithms (proven by checking the bytes on disk, not just the
/// exit code — a bug that quietly fell back to classical-only would still "succeed"), and
/// `verify-sig --unstable` accepts it under the DEFAULT policy. `--unstable` is required on the
/// verify side too, and is deliberately included here rather than assumed — the test below this
/// one is what happens if it is left out.
#[test]
fn sign_hybrid_with_unstable_succeeds_and_the_signature_verifies() {
    let dir = tmp("sign-unstable");
    let home = dir.join("home");
    let signer = keygen(&dir, &home);

    let art = dir.join("payload.dwx");
    std::fs::write(&art, b"industrial artifact bytes").unwrap();

    let o = delulu(&dir, &home, &["sign", art.to_str().unwrap(), "--hybrid", "--unstable", "--json"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let sig_path = dir.join("payload.dwx.sig");
    assert!(sig_path.exists(), "a successful hybrid sign writes the .sig file");
    let v: Value = serde_json::from_slice(&o.stdout).expect("sign --json is valid JSON");
    assert_eq!(v["signer"], signer);
    assert_eq!(v["algorithms"], serde_json::json!(["ed25519", "ml-dsa-65"]), "{v}");

    // The bytes on disk are the `dlsig1` text envelope, not the old 96-byte blob — proof that
    // `--hybrid` took the different code path, not merely printed a different message about it.
    let bytes = std::fs::read(&sig_path).unwrap();
    assert!(bytes.starts_with(b"dlsig1"), "hybrid output must be a dlsig1 envelope, not a legacy blob");

    let vf = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--unstable", "--json"]);
    assert!(vf.status.success(), "{}", stderr(&vf));
    let vfv: Value = serde_json::from_slice(&vf.stdout).unwrap();
    assert_eq!(vfv["verdict"], "valid");
    assert_eq!(vfv["signer"], signer);
}

/// **DL1908, both of its spellings.** `verify-sig --require-hybrid` against a classical-only
/// artifact must refuse — and "classical-only" has two shapes in this codebase: the legacy
/// 96-byte v1.0 signature every artifact carries today, and a `dlsig1` envelope that merely
/// happens to name only `ed25519`. A policy check that recognised only the second shape would let
/// every artifact `delulu sign` has ever produced sail straight through the one gate that most
/// needs to catch them, so both are built here and both are asserted.
#[test]
fn verify_sig_require_hybrid_refuses_classical_only_in_both_of_its_spellings() {
    let dir = tmp("require-hybrid-classical");
    let home = dir.join("home");
    keygen(&dir, &home);

    // Spelling 1: the legacy shape, produced by the REAL `delulu sign` classical default — the
    // exact bytes every artifact signed under v1.0, and every artifact `sign` produces today
    // without `--hybrid`, carries.
    let legacy_art = dir.join("legacy.dwx");
    std::fs::write(&legacy_art, b"a v1.0-style artifact").unwrap();
    let sg = delulu(&dir, &home, &["sign", legacy_art.to_str().unwrap()]);
    assert!(sg.status.success(), "{}", stderr(&sg));
    let legacy_sig = std::fs::read(dir.join("legacy.dwx.sig")).unwrap();
    assert_eq!(legacy_sig.len(), 96, "the classical default must still be the raw 96-byte blob");

    let o = delulu(&dir, &home, &["verify-sig", legacy_art.to_str().unwrap(), "--require-hybrid", "--json"]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    let v: Value =
        serde_json::from_slice(&o.stdout).expect("verify-sig --json is valid JSON even when refused");
    assert_eq!(v["code"], "DL1908", "{v}");

    // Spelling 2: a `dlsig1` envelope that declares only `ed25519` — a shape the CLI itself never
    // emits (its classical path stays the legacy blob; see `SEED`'s doc comment above), so it is
    // built directly against `pqc::sign_classical`, the only way this shape exists at all.
    let env_art = dir.join("envelope.dwx");
    let data = b"a hand-built classical-only envelope";
    std::fs::write(&env_art, data).unwrap();
    let env_sig = delulu_runtime::pqc::sign_classical(&SEED, data);
    assert!(env_sig.starts_with(b"dlsig1"), "this fixture must actually exercise the envelope shape");
    std::fs::write(dir.join("envelope.dwx.sig"), &env_sig).unwrap();

    let o2 = delulu(&dir, &home, &["verify-sig", env_art.to_str().unwrap(), "--require-hybrid", "--json"]);
    let err2 = stderr(&o2);
    assert_eq!(o2.status.code(), Some(1), "{err2}");
    let v2: Value =
        serde_json::from_slice(&o2.stdout).expect("verify-sig --json is valid JSON even when refused");
    assert_eq!(v2["code"], "DL1908", "{v2}");
}

/// The regression check, restated for this file's own record even though `signing_cli.rs` already
/// covers the same contract: `verify-sig` with none of the new flags must keep working for a plain
/// classical signature, unchanged. `cmd_verify_sig` now routes every call through `pqc::verify`
/// instead of calling `verify_detached` directly — if that rewrite ever changed what a
/// no-flags call reports, every already-signed `.dwx`/`.dpx` in existence would start failing
/// verification with no change on the signer's part. That is the exact failure "byte-for-byte
/// unchanged" is meant to rule out, so it is asserted on both the JSON and the human-text channel.
#[test]
fn verify_sig_with_no_new_flags_still_accepts_a_plain_classical_signature() {
    let dir = tmp("plain-classical");
    let home = dir.join("home");
    let signer = keygen(&dir, &home);

    let art = dir.join("payload.dwx");
    std::fs::write(&art, b"ordinary artifact bytes, signed the ordinary way").unwrap();
    let sg = delulu(&dir, &home, &["sign", art.to_str().unwrap()]);
    assert!(sg.status.success(), "{}", stderr(&sg));

    let o = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--json"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["verdict"], "valid");
    assert_eq!(v["signer"], signer);

    // The plain-text rendering too, since a human runs this command far more often than a script.
    let ot = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap()]);
    assert!(ot.status.success());
    let text = String::from_utf8_lossy(&ot.stdout);
    assert!(text.contains("valid") && text.contains(&signer), "{text}");
}

/// **DL1910 on the verify side, under the DEFAULT policy.** It is the presence of a post-quantum
/// part in the envelope that trips this gate, not `--require-hybrid` asking for one. A
/// hybrid-signed artifact handed to a plain `verify-sig` — no flags at all, exactly what every
/// existing caller already invokes — must be refused precisely because verifying is invoking
/// unvalidated cryptography to make a trust decision, regardless of which policy asked for it.
/// A build that gated this only under `--require-hybrid` would happily run unvalidated ML-DSA
/// code on any artifact merely because nobody asked for hybrid enforcement.
#[test]
fn a_hybrid_signature_verified_without_unstable_is_refused_even_under_the_default_policy() {
    let dir = tmp("hybrid-no-unstable-verify");
    let home = dir.join("home");
    keygen(&dir, &home);

    let art = dir.join("payload.dwx");
    std::fs::write(&art, b"industrial artifact bytes").unwrap();
    let sg = delulu(&dir, &home, &["sign", art.to_str().unwrap(), "--hybrid", "--unstable"]);
    assert!(sg.status.success(), "{}", stderr(&sg));

    // No --require-hybrid, no --unstable: the default policy, exactly what every pre-10i caller
    // of verify-sig already invokes.
    let o = delulu(&dir, &home, &["verify-sig", art.to_str().unwrap(), "--json"]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    let v: Value =
        serde_json::from_slice(&o.stdout).expect("verify-sig --json is valid JSON even when refused");
    assert_eq!(v["code"], "DL1910", "{v}");
}

/// The control for the DL1908 test above: `--require-hybrid` is a gate, not a wall. A GENUINE
/// hybrid signature, verified with `--unstable` set, must pass — without a passing case, the
/// refusals in this file would be indistinguishable from a policy that simply refuses everything
/// it is handed, which is not a policy anyone could rely on.
#[test]
fn verify_sig_require_hybrid_accepts_a_genuine_hybrid_signature_under_unstable() {
    let dir = tmp("require-hybrid-genuine");
    let home = dir.join("home");
    let signer = keygen(&dir, &home);

    let art = dir.join("payload.dwx");
    std::fs::write(&art, b"industrial artifact bytes").unwrap();
    let sg = delulu(&dir, &home, &["sign", art.to_str().unwrap(), "--hybrid", "--unstable"]);
    assert!(sg.status.success(), "{}", stderr(&sg));

    let o = delulu(
        &dir,
        &home,
        &["verify-sig", art.to_str().unwrap(), "--require-hybrid", "--unstable", "--json"],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["verdict"], "valid");
    assert_eq!(v["signer"], signer);
}
