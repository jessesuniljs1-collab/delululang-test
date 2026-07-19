//! Stage 10 phase 10b — the JIT policy gate (Track A3, spec §2.3, invariant 46).
//!
//! No native tier exists in v1.x. That is exactly why this phase exists: the grant, the manifest
//! request line, the authority report, and DL1906 all land BEFORE any engine, so no engine can
//! ever exist ungated. Two laws meet here and both are tested: invariant 46 (native emission is
//! a grant, default-off everywhere) and invariant 45 (a hint may not change what a program does
//! — so an ungranted `@jit` is IGNORED with a warning, never refused).

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
    let d = std::env::temp_dir().join(format!("delulu-jitpol-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

const HINTED: &str = "module m\n\n@jit\nfn hot(x: Int) -> Int {\n    x * 2\n}\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(str(hot(21)))\n}\n";

/// Invariants 45 and 46 in one test: without the grant the hint is IGNORED — the program still
/// runs, interpreted, output correct, exit 0 — and DL1906 says so on stderr. A gate that refused
/// the program would let a hint change behavior; a gate that stayed silent would be a policy
/// nobody can see. Both halves are the point.
#[test]
fn an_ungranted_jit_hint_warns_dl1906_and_the_program_still_runs() {
    let dir = scratch("warn");
    let f = dir.join("hinted.delulu");
    std::fs::write(&f, HINTED).unwrap();
    let o = delulu(&["run", &f.to_string_lossy(), "--grant", "console"]);
    assert!(o.status.success(), "the hint may not change whether the program runs");
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "42", "output is the interpreter's");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL1906"), "the ignored hint is announced, not silent: {err}");
    assert!(
        err.contains("exec.native"),
        "the warning names the grant that would change the answer"
    );
}

/// With the grant, the warning goes away — and nothing else changes, because there is no native
/// tier to engage. (When 10l builds one, criterion 2 demands its traces equal the interpreter's;
/// this test is the baseline half of that story.)
#[test]
fn a_granted_jit_hint_runs_without_the_warning() {
    let dir = scratch("granted");
    let f = dir.join("hinted.delulu");
    std::fs::write(&f, HINTED).unwrap();
    let o = delulu(&["run", &f.to_string_lossy(), "--grant", "console", "--grant", "exec.native"]);
    assert!(o.status.success());
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "42");
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains("DL1906"),
        "granted means no warning"
    );
}

/// House rule 4: the machine channel is law. A `--json` run of a hinted program carries no
/// DL1906 anywhere in stdout — the warning is a human-surface note (the DL1790 pattern), and
/// agents read the REQUEST off the authority report instead.
#[test]
fn the_json_run_channel_never_carries_the_warning() {
    let dir = scratch("json");
    let f = dir.join("hinted.delulu");
    std::fs::write(&f, HINTED).unwrap();
    let o = delulu(&["run", &f.to_string_lossy(), "--grant", "console", "--json"]);
    assert!(o.status.success());
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(!out.contains("DL1906"), "machine stdout stays byte-stable: {out}");
}

/// The authority report stamps `native_emission` ONLY when the program carries a `@jit` hint —
/// the skip branch is byte-stability for every hint-free program (house rule 4), and the hit
/// branch is the reviewable request an agent or human reads before deciding on the grant.
#[test]
fn authority_reports_the_request_only_when_a_jit_hint_exists() {
    let dir = scratch("auth");
    let hinted = dir.join("hinted.delulu");
    let plain = dir.join("plain.delulu");
    std::fs::write(&hinted, HINTED).unwrap();
    std::fs::write(&plain, HINTED.replace("@jit\n", "")).unwrap();

    let vh: serde_json::Value =
        serde_json::from_slice(&delulu(&["authority", &hinted.to_string_lossy(), "--json"]).stdout)
            .expect("hinted authority json");
    assert_eq!(vh["authority"]["native_emission"]["requested"], true);

    let vp: serde_json::Value =
        serde_json::from_slice(&delulu(&["authority", &plain.to_string_lossy(), "--json"]).stdout)
            .expect("plain authority json");
    assert!(
        vp["authority"].get("native_emission").is_none(),
        "a hint-free program's report has NO native_emission key — byte-identical to 1.0"
    );

    let human = delulu(&["authority", &hinted.to_string_lossy()]);
    let text = String::from_utf8_lossy(&human.stdout);
    assert!(text.contains("native-emission: requested"), "the human line exists: {text}");
    assert!(
        text.contains("no native tier exists in v1.x"),
        "the honesty clause is on the line itself, not in a footnote"
    );
}

/// The manifest is the reviewable request channel: a hinted package whose manifest does NOT
/// declare `exec.native = true` gets the longer warning naming the missing declaration; one that
/// declares it gets the shorter warning (declared, just not granted). Declaration is never
/// permission — `--grant-manifest` deliberately does not confer this grant (build-order D6).
#[test]
fn the_manifest_declares_the_request_but_never_grants_it() {
    let dir = scratch("manifest");
    let f = dir.join("hinted.delulu");
    std::fs::write(&f, HINTED).unwrap();

    std::fs::write(dir.join("delulu.toml"), "[package]\nname = \"m\"\n[authority]\neffects = [\"Write\"]\n").unwrap();
    let o = delulu(&["run", &f.to_string_lossy(), "--grant", "console"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("does not declare the request"),
        "undeclared request is named: {err}"
    );

    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"m\"\n[authority]\neffects = [\"Write\"]\nexec.native = true\n",
    )
    .unwrap();
    let o2 = delulu(&["run", &f.to_string_lossy(), "--grant", "console", "--grant-manifest"]);
    assert!(o2.status.success());
    let err2 = String::from_utf8_lossy(&o2.stderr);
    assert!(
        err2.contains("DL1906") && !err2.contains("does not declare"),
        "declared-but-ungranted still warns — accepting a manifest never confers the red-tier \
         grant: {err2}"
    );
}
