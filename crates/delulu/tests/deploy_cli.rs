//! Stage 10 phase 10j (Track H): `delulu deploy plan` — the whole-deployment authority answer,
//! computed and refused against an environment profile's ceiling BEFORE anything runs (spec
//! §9.2, invariant 53). Driven through the real binary end to end, mirroring `signing_cli.rs`'s
//! and `compute_cli.rs`'s house style (fixture packages on disk, `CARGO_BIN_EXE_delulu`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_deploy_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn delulu(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .expect("run delulu")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}
fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// A package directory that DECLARES and ACTUALLY PERFORMS exactly `effects` — the checked
/// authority report reflects what `main` does, not merely what the manifest claims (mirrors
/// `signing_cli.rs`'s `scaffold_pkg`). Supports `[]`, `["Write"]`, and `["Write", "Net"]` — the
/// three shapes this test file needs.
fn scaffold_pkg(dir: &Path, pkg_name: &str, effects: &[&str]) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    let effects_toml = effects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
    std::fs::write(
        dir.join("delulu.toml"),
        format!("[package]\nname = \"{pkg_name}\"\nversion = \"1.0.0\"\n\n[authority]\neffects = [{effects_toml}]\n"),
    )
    .unwrap();
    let body = if effects.is_empty() {
        "module x\nfn main(root: Root) {\n}\n".to_string()
    } else if effects.len() == 1 && effects[0] == "Write" {
        "module x\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(\"hi\")\n}\n".to_string()
    } else {
        "module x\nfn main(root: Root) ! {Write, Net} {\n  let out = root.console()\n  out.println(\"hi\")\n  \
         let h = root.http([\"example.com\"])\n  let r = h.get(\"http://example.com\")\n}\n"
            .to_string()
    };
    std::fs::write(src.join("main.delulu"), body).unwrap();
}

/// A package whose SOURCE does not parse — the "does not check cleanly" fixture. Deliberately
/// NOT a manifest/effects mismatch: `package_authority_value` runs `check_program` alone (never
/// the separate DL0701 manifest-vs-row cross-check), so only a genuine syntax error is guaranteed
/// to make it return `None`.
fn scaffold_broken_pkg(dir: &Path) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"broken\"\nversion = \"1.0.0\"\n\n[authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();
    // Unclosed function body: the file ends mid-statement with no closing braces at all.
    std::fs::write(src.join("main.delulu"), "module broken\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n").unwrap();
}

/// An environment profile file — the SAME `[authority]\neffects = [...]` shape a package's own
/// `delulu.toml` uses (spec §9.2's deliberate reuse).
fn write_env(dir: &Path, file_name: &str, effects: &[&str]) -> PathBuf {
    let path = dir.join(file_name);
    let effects_toml = effects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
    std::fs::write(&path, format!("[authority]\neffects = [{effects_toml}]\n")).unwrap();
    path
}

/// Every service is within the profile's ceiling: the plan is approved, exit 0, and the JSON
/// carries `approved: true` plus every service's own checked effects (spec §9.2's shape). Also
/// checks the human-readable summary line matches the spec'd wording exactly.
#[test]
fn a_plan_within_the_ceiling_is_approved() {
    let dir = tmp("ok");
    let pure = dir.join("pure");
    let writer = dir.join("writer");
    scaffold_pkg(&pure, "pure", &[]);
    scaffold_pkg(&writer, "writer", &["Write"]);
    let env = write_env(&dir, "prod.toml", &["Write", "Net"]);
    let env_str = env.to_string_lossy().to_string();

    let o = delulu(
        &dir,
        &[
            "deploy",
            "plan",
            "--service",
            &format!("pure={}", pure.to_string_lossy()),
            "--service",
            &format!("writer={}", writer.to_string_lossy()),
            "--env",
            &env_str,
            "--json",
        ],
    );
    assert!(o.status.success(), "within-ceiling plan must approve:\n{}", stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert_eq!(v["command"], "deploy");
    assert_eq!(v["subcommand"], "plan");
    assert_eq!(v["approved"], true);
    let services = v["services"].as_array().expect("services array");
    assert_eq!(services.len(), 2, "{v}");
    assert!(
        services.iter().any(|s| s["name"] == "pure" && s["effects"].as_array().unwrap().is_empty()),
        "the pure service reports zero effects: {v}"
    );
    assert!(
        services.iter().any(|s| s["name"] == "writer" && s["effects"][0] == "Write"),
        "the writer service reports Write: {v}"
    );

    // The human-readable form ends with the approved summary line, exact wording from the spec.
    let h = delulu(
        &dir,
        &[
            "deploy",
            "plan",
            "--service",
            &format!("pure={}", pure.to_string_lossy()),
            "--service",
            &format!("writer={}", writer.to_string_lossy()),
            "--env",
            &env_str,
        ],
    );
    assert!(h.status.success(), "{}", stderr(&h));
    let text = stdout(&h);
    // "EFFECT ceiling", not "authority ceiling": this command compares one authority dimension of
    // nine, and the verdict now says which (C45). The wording is pinned exactly because it is the
    // line a reader uses to decide whether a deployment is safe to launch.
    assert!(
        text.contains(&format!("deploy plan: approved — 2 service(s) within {env_str}'s EFFECT ceiling")),
        "the exact approved summary line:\n{text}"
    );
    assert!(
        text.contains("compared: effects; NOT compared:"),
        "and the approval carries its own scope:\n{text}"
    );
}

/// A service that performs `Net` when the environment profile's ceiling names only `Write` is
/// refused WHOLE — DL1909, exit 1, naming both the exceeding service and the exceeding effect,
/// and NOT naming the compliant service as if it, too, exceeded.
#[test]
fn a_service_exceeding_the_ceiling_is_refused_by_name() {
    let dir = tmp("exceed");
    let writer = dir.join("writer");
    let networker = dir.join("networker");
    scaffold_pkg(&writer, "writer", &["Write"]);
    scaffold_pkg(&networker, "networker", &["Write", "Net"]);
    let env = write_env(&dir, "prod.toml", &["Write"]); // Net is NOT in the ceiling
    let env_str = env.to_string_lossy().to_string();

    let o = delulu(
        &dir,
        &[
            "deploy",
            "plan",
            "--service",
            &format!("writer={}", writer.to_string_lossy()),
            "--service",
            &format!("networker={}", networker.to_string_lossy()),
            "--env",
            &env_str,
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(1), "exceeding the ceiling must refuse:\n{}", stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).expect("valid JSON even on refusal");
    assert_eq!(v["approved"], false, "{v}");
    assert_eq!(v["code"], "DL1909", "{v}");
    let err = v["error"].as_str().unwrap().to_string();
    assert!(err.contains("networker"), "the message names the exceeding SERVICE:\n{err}");
    assert!(err.contains("Net"), "the message names the exceeding EFFECT:\n{err}");
    assert!(!err.contains("writer"), "the compliant service must not be named as exceeding:\n{err}");

    // The human channel carries the same code and names.
    let h = delulu(
        &dir,
        &[
            "deploy",
            "plan",
            "--service",
            &format!("writer={}", writer.to_string_lossy()),
            "--service",
            &format!("networker={}", networker.to_string_lossy()),
            "--env",
            &env_str,
        ],
    );
    assert_eq!(h.status.code(), Some(1));
    let text = stderr(&h);
    assert!(text.contains("DL1909"), "{text}");
    assert!(text.contains("networker") && text.contains("Net"), "{text}");
}

/// Missing `--env` is a usage error (exit 2) — the same class as a missing positional elsewhere
/// in this CLI (e.g. `cmd_sign`'s "needs an artifact"), never a diagnostic code.
#[test]
fn missing_env_is_a_usage_error() {
    let dir = tmp("noenv");
    let pure = dir.join("pure");
    scaffold_pkg(&pure, "pure", &[]);
    let o = delulu(&dir, &["deploy", "plan", "--service", &format!("pure={}", pure.to_string_lossy())]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
}

/// Zero `--service` flags is a usage error (exit 2) — at least one service is required for a
/// deployment plan to mean anything.
#[test]
fn zero_services_is_a_usage_error() {
    let dir = tmp("noservice");
    let env = write_env(&dir, "prod.toml", &["Write"]);
    let o = delulu(&dir, &["deploy", "plan", "--env", &env.to_string_lossy()]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
}

/// A service directory that does not check cleanly is refused too — but this is a DIFFERENT
/// failure than DL1909 (the spec's own distinction: "checks fine but exceeds the profile" is not
/// the same failure as "does not check at all"), so the refusal must name the service and must
/// NOT carry the DL1909 code.
#[test]
fn a_service_that_does_not_check_cleanly_is_refused_without_dl1909() {
    let dir = tmp("badsvc");
    let broken = dir.join("broken");
    scaffold_broken_pkg(&broken);
    let env = write_env(&dir, "prod.toml", &["Write", "Net"]);
    let o = delulu(
        &dir,
        &[
            "deploy",
            "plan",
            "--service",
            &format!("broken={}", broken.to_string_lossy()),
            "--env",
            &env.to_string_lossy(),
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(1), "{}", stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert_eq!(v["approved"], false, "{v}");
    assert_ne!(v["code"], "DL1909", "a check failure is NOT DL1909: {v}");
    let err = v["error"].as_str().unwrap().to_string();
    assert!(err.contains("broken"), "names the failing service:\n{err}");
}
