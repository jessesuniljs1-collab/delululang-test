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
    std::fs::write(src.join("main.delulu"), "module broken\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n")
        .unwrap();
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
    // "authority ceiling" since RW 6.6 (2026-10-22): the plan compares effects AND every request's scope against the
    // profile. Until then it compared one dimension and said "EFFECT ceiling" (C45). The wording is pinned exactly because
    // it is the line a reader uses to decide whether a deployment is safe to launch; the line above it names the scope.
    assert!(
        text.contains(&format!("deploy plan: approved — 2 service(s) within {env_str}'s authority ceiling")),
        "the exact approved summary line:\n{text}"
    );
    assert!(
        text.contains(
            "compared: effects, capability scopes (fs.read, fs.write, net, secrets), foreign holes; a profile cannot grant:"
        ),
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

/// The repository's four-package program (`station`: four packages, seven modules).
fn station() -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/tier4-multimodule/station")
        .canonicalize()
        .expect("the station corpus package")
        .to_string_lossy()
        .to_string()
}

fn profile(dir: &Path, name: &str, body: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, format!("[authority]\n{body}")).unwrap();
    path.to_string_lossy().to_string()
}

/// DEPLOY-PKG-1 (2026-10-22): a package with dependencies plans as `delulu authority <dir>` checks it — whole. `deploy
/// plan` had kept the single-package loader after C51 moved `authority` off it, and refused the four-package `station` as
/// a service that "does not check cleanly" while `authority` reported it. Under a profile naming exactly what it asks for
/// — its effects, `./data` to read and `./out` to write — it is approved, and its scopes are part of the approval.
#[test]
fn a_package_with_dependencies_plans_as_authority_checks_it() {
    let dir = tmp("station-ok");
    let env = profile(&dir, "env.toml", "effects = [\"Read\", \"Write\"]\nfs.read = [\"./data\"]\nfs.write = [\"./out\"]\n");
    let svc = format!("station={}", station());
    let o = delulu(&dir, &["deploy", "plan", "--service", &svc, "--env", &env, "--json"]);
    assert!(o.status.success(), "a package with dependencies plans:\n{}{}", stdout(&o), stderr(&o));
    let v: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert_eq!(v["approved"], true, "{v}");
    assert_eq!(v["services"][0]["within"]["result"], "within", "{v}");
    assert_eq!(v["services"][0]["effects"], serde_json::json!(["Read", "Write"]), "{v}");
}

/// RW 6.6 (2026-10-22): a request outside the profile's scopes refuses the plan, named at the call that makes it and in
/// the runtime's own words — `station` reads `./data`, which a profile granting `./other` does not cover, though the
/// profile grants the `Read` effect. Before, only effects were compared, and this plan was approved.
#[test]
fn a_request_outside_the_profiles_scopes_is_refused_at_its_call() {
    let dir = tmp("station-scope");
    let env = profile(&dir, "env.toml", "effects = [\"Read\", \"Write\"]\nfs.read = [\"./other\"]\nfs.write = [\"./out\"]\n");
    let svc = format!("station={}", station());
    let o = delulu(&dir, &["deploy", "plan", "--service", &svc, "--env", &env]);
    assert_eq!(o.status.code(), Some(1), "{}", stderr(&o));
    let err = stderr(&o);
    assert!(err.contains("DL1909") && err.contains("service `station` asks for `fs.read=./data`"), "{err}");
    assert!(err.contains("src/station.delulu:16") && err.contains("exceeds"), "named at its call: {err}");
    let o = delulu(&dir, &["deploy", "plan", "--service", &svc, "--env", &env, "--json"]);
    let v: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert_eq!(v["approved"], false, "{v}");
    let within = &v["services"][0]["within"];
    assert_eq!(
        (within["result"].as_str(), within["counterexample"]["grant"].as_str()),
        (Some("exceeds"), Some("fs.read=./data"))
    );
    assert_eq!(within["counterexample"]["at"]["line"], 16, "{v}");
}

/// A scope the source computes cannot be bounded by reading it: inconclusive, and an inconclusive plan is refused like an
/// exceeding one — no plan, no launch.
#[test]
fn a_scope_the_source_computes_is_refused_as_inconclusive() {
    let dir = tmp("computed");
    let svc = dir.join("computed");
    std::fs::create_dir_all(svc.join("src")).unwrap();
    std::fs::write(
        svc.join("delulu.toml"),
        "[package]\nname = \"computed\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Read\", \"Write\"]\n",
    )
    .unwrap();
    std::fs::write(
        svc.join("src/main.delulu"),
        "module computed\nfn main(root: Root) ! {Read, Write} {\n  let p = \"./da\" + \"ta\"\n  let r = root.fs_read(p)\n  \
         match r.read_text(\"x\") { Ok(t) => root.console().println(t)  Err(e) => root.console().println(\"no\") }\n}\n",
    )
    .unwrap();
    let env = profile(&dir, "env.toml", "effects = [\"Read\", \"Write\"]\nfs.read = [\"./data\"]\n");
    let o = delulu(&dir, &["deploy", "plan", "--service", &format!("computed={}", svc.to_string_lossy()), "--env", &env]);
    assert_eq!(o.status.code(), Some(1), "{}", stderr(&o));
    assert!(stderr(&o).contains("DL1909") && stderr(&o).contains("inconclusive"), "{}", stderr(&o));
    assert!(stderr(&o).contains("src/main.delulu:4"), "named at the computed call: {}", stderr(&o));
}

/// Secrets are bounded by name: a profile that names `API_KEY` approves a service minting it, one that names another
/// secret refuses it. And a profile that names a special-use host under `net` is not a boundary at all (D-NE-28: a
/// loopback or private host is granted only by its explicit `net.special` spelling, which a profile does not have).
#[test]
fn a_secret_is_bounded_by_name_and_a_special_use_host_is_no_boundary() {
    let dir = tmp("secret");
    let svc = dir.join("keyed");
    std::fs::create_dir_all(svc.join("src")).unwrap();
    std::fs::write(svc.join("delulu.toml"), "[package]\nname = \"keyed\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n")
        .unwrap();
    std::fs::write(svc.join("src/main.delulu"), "module keyed\nfn main(root: Root) {\n  let k = root.secret(\"API_KEY\")\n}\n")
        .unwrap();
    let svc_arg = format!("keyed={}", svc.to_string_lossy());
    let named = profile(&dir, "named.toml", "effects = []\nsecrets = [\"API_KEY\"]\n");
    let o = delulu(&dir, &["deploy", "plan", "--service", &svc_arg, "--env", &named]);
    assert!(o.status.success(), "{}{}", stdout(&o), stderr(&o));
    let other = profile(&dir, "other.toml", "effects = []\nsecrets = [\"OTHER\"]\n");
    let o = delulu(&dir, &["deploy", "plan", "--service", &svc_arg, "--env", &other]);
    assert_eq!(o.status.code(), Some(1), "{}", stderr(&o));
    assert!(stderr(&o).contains("asks for `secret:API_KEY") && stderr(&o).contains("src/main.delulu:3"), "{}", stderr(&o));
    let loopback = profile(&dir, "loopback.toml", "effects = [\"Net\"]\nnet = [\"127.0.0.1\"]\n");
    let o = delulu(&dir, &["deploy", "plan", "--service", &svc_arg, "--env", &loopback]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(stderr(&o).contains("`net=127.0.0.1` is not a boundary this profile can give"), "{}", stderr(&o));
}
