//! End-to-end tests for `delulu plugin build | inspect` (Stage 6 "Live", phase 6c).
//!
//! The contract under test: a `kind = "plugin"` package builds into a `.dpx` whose manifest is
//! bound to its content (blake3); the manifest never overrides the code (DL1501, both
//! directions); refusals leave no partial artifact; and the machine channel is byte-identical
//! across runs (house rule 8).

use std::path::PathBuf;
use std::process::{Command, Output};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).args(args).output().expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// Scaffold a plugin package under a unique temp dir. Returns the package root.
fn scaffold(tag: &str, manifest: &str, code: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_plugin_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("delulu.toml"), manifest).unwrap();
    std::fs::write(dir.join("src").join("lib.delulu"), code).unwrap();
    dir
}

const MANIFEST: &str = r#"[package]
name = "summarize"
version = "0.1.0"
kind = "plugin"

[plugin]
api = 1
class = "verified"

[plugin.authority]
effects  = ["Read"]
requires = ["Cap[FsRead]"]

[plugin.exports]
summarize = "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}"
"#;

const CODE: &str = "module summarize\n\
    pub fn summarize(fs: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { fs.read_text(path) }\n";

#[test]
fn build_verified_dpx_and_inspect_it() {
    let dir = scaffold("build_ok", MANIFEST, CODE);
    let dpx = dir.join("summarize.dpx");
    let o = delulu(&["plugin", "build", dir.to_str().unwrap()]);
    assert!(o.status.success(), "build must succeed: {}", stderr(&o));
    assert!(dpx.exists(), "the artifact must be written");

    let o = delulu(&["plugin", "inspect", dpx.to_str().unwrap()]);
    assert!(o.status.success(), "inspect must succeed: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("`summarize` v0.1.0"), "{out}");
    assert!(out.contains("class verified"), "{out}");
    assert!(out.contains("fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}"), "{out}");
    assert!(out.contains("delulu:dir"), "the DIR section is listed: {out}");
    assert!(out.contains("signature: none"), "{out}");
}

#[test]
fn inspect_json_is_byte_identical_across_runs() {
    // House rule 8: the machine channel is deterministic and never styled.
    let dir = scaffold("json_stable", MANIFEST, CODE);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap(), "--json"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let dpx = dir.join("summarize.dpx");
    let a = delulu(&["plugin", "inspect", dpx.to_str().unwrap(), "--json"]);
    let b = delulu(&["plugin", "inspect", dpx.to_str().unwrap(), "--json"]);
    assert!(a.status.success() && b.status.success());
    assert_eq!(stdout(&a), stdout(&b), "inspect --json must be byte-identical across runs");
    assert!(!stdout(&a).contains('\u{1b}'), "machine channels are never colored");
    // The JSON parses and carries the inspection surface.
    let v: serde_json::Value = serde_json::from_str(&stdout(&a)).expect("valid JSON");
    assert_eq!(v["class"], "verified");
    assert_eq!(v["api"], 1);
    assert!(v["sections"]["dir"].is_string(), "dir hash present: {v}");
    assert!(v["sections"]["wasm"].is_null(), "no wasm for a plain verified build: {v}");
    assert_eq!(v["signed_by"], serde_json::Value::Null);
}

#[test]
fn build_is_deterministic_across_runs() {
    let dir = scaffold("build_stable", MANIFEST, CODE);
    let out1 = dir.join("a.dpx");
    let out2 = dir.join("b.dpx");
    let o1 = delulu(&["plugin", "build", dir.to_str().unwrap(), "-o", out1.to_str().unwrap()]);
    let o2 = delulu(&["plugin", "build", dir.to_str().unwrap(), "-o", out2.to_str().unwrap()]);
    assert!(o1.status.success() && o2.status.success());
    assert_eq!(
        std::fs::read(&out1).unwrap(),
        std::fs::read(&out2).unwrap(),
        "identical inputs must produce byte-identical .dpx artifacts"
    );
}

#[test]
fn manifest_export_mismatch_is_dl1501_and_no_artifact() {
    // The manifest undersells the code's row ({Read} in code, pure in manifest): still DL1501 —
    // the manifest never overrides the code, in either direction. Refusal honesty: no artifact.
    let manifest = MANIFEST.replace(
        "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}",
        "fn(Cap[FsRead], Str) -> Result[Str, IoErr]",
    );
    let dir = scaffold("mismatch", &manifest, CODE);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1), "a mismatched manifest must refuse");
    assert!(stdout(&o).contains("DL1501"), "{}", stdout(&o));
    assert!(!dir.join("summarize.dpx").exists(), "a refused build must leave no artifact");
}

#[test]
fn fn_main_in_a_plugin_package_is_dl1501() {
    let code = format!("{CODE}fn main(root: Root) {{ }}\n");
    let dir = scaffold("main", MANIFEST, &code);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("DL1501"), "{}", stdout(&o));
    assert!(stdout(&o).contains("fn main"), "{}", stdout(&o));
}

#[test]
fn unsupported_api_is_dl1507_at_build() {
    let manifest = MANIFEST.replace("api = 1", "api = 3");
    let dir = scaffold("api", &manifest, CODE);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("DL1507"), "{}", stdout(&o));
}

#[test]
fn non_plugin_package_is_refused() {
    let manifest = MANIFEST.replace("kind = \"plugin\"", "kind = \"lib\"");
    let dir = scaffold("kind", &manifest, CODE);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("DL1004"), "{}", stdout(&o));
}

#[test]
fn contained_build_carries_the_module_not_dir() {
    // A contained plugin ships opaque WASM: the .dpx must carry delulu:wasm and no delulu:dir.
    let manifest = r#"[package]
name = "shout"
version = "0.1.0"
kind = "plugin"

[plugin]
api = 1
class = "contained"

[plugin.authority]
effects  = []
requires = []

[plugin.exports]
shout = "fn(Int) -> Int"
"#;
    let code = "module shout\npub fn shout(x: Int) -> Int { x * 2 }\n";
    let dir = scaffold("contained", manifest, code);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap()]);
    assert!(o.status.success(), "contained build must succeed: {}", stderr(&o));
    let dpx = dir.join("shout.dpx");
    let o = delulu(&["plugin", "inspect", dpx.to_str().unwrap(), "--json"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("valid JSON");
    assert_eq!(v["class"], "contained");
    assert!(v["sections"]["wasm"].is_string(), "the module section is present: {v}");
    assert!(v["sections"]["dir"].is_null(), "a contained plugin carries no DIR: {v}");
}

#[test]
fn tampered_dir_body_refuses_at_inspect_as_dl1504() {
    // Criterion 6 seed: one flipped byte in the DIR body → the artifact's Verified content can no
    // longer be re-checked as shipped → DL1504 (and NEVER a silent fallback to contained).
    let dir = scaffold("tamper", MANIFEST, CODE);
    let o = delulu(&["plugin", "build", dir.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    let dpx_path = dir.join("summarize.dpx");
    let mut bytes = std::fs::read(&dpx_path).unwrap();
    // Flip one byte deep in the artifact (inside the DIR payload, past the manifest section).
    let at = bytes.len() - 16;
    bytes[at] ^= 0x01;
    std::fs::write(&dpx_path, &bytes).unwrap();

    let o = delulu(&["plugin", "inspect", dpx_path.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1), "a tampered artifact must refuse");
    let out = stdout(&o);
    assert!(out.contains("DL1504"), "a tampered DIR is a failed Verified re-check: {out}");
    assert!(out.contains("never falls back to Contained"), "invariant 29 stated in the refusal: {out}");
}

#[test]
fn garbage_file_refuses_cleanly_as_dl1508() {
    let path = std::env::temp_dir().join(format!("delulu_garbage_{}.dpx", std::process::id()));
    std::fs::write(&path, b"this is not a plugin artifact at all").unwrap();
    let o = delulu(&["plugin", "inspect", path.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("DL1508"), "{}", stdout(&o));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn multi_module_plugin_package_is_refused_cleanly() {
    // Deviation 3: single-module plugin packages in v0.6 — refused, never half-built.
    let dir = scaffold("multi", MANIFEST, CODE);
    std::fs::write(dir.join("src").join("extra.delulu"), "module extra\npub fn f() -> Int { 1 }\n").unwrap();
    let o = delulu(&["plugin", "build", dir.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("single-module"), "{}", stdout(&o));
    assert!(!dir.join("summarize.dpx").exists(), "no artifact on refusal");
}

/// The `.dwx` path must be entirely unaffected by the `.dpx` machinery (zero regressions on the
/// Stage-3 artifact — same crate, shared section helpers).
#[test]
fn dwx_build_still_works_beside_dpx() {
    let src = std::env::temp_dir().join(format!("delulu_dwx_beside_{}.delulu", std::process::id()));
    std::fs::write(&src, "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"still fine\") }\n").unwrap();
    let out = std::env::temp_dir().join(format!("delulu_dwx_beside_{}.dwx", std::process::id()));
    let o = delulu(&["build", src.to_str().unwrap(), "--target", "wasm", "-o", out.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(out.exists());
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}
