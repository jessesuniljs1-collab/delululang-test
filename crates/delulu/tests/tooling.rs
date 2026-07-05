//! Stage-2 tooling acceptance tests: `delulu why`, `delulu authority --diff`, and the §5.5
//! `interface.json` build artifact — exercised end-to-end through the real `delulu` binary.
//!
//! These commands are function-granularity, honest approximations (see the doc comments on
//! `cmd_why` / `write_interfaces` in `crates/delulu/src/cli.rs` for exactly what is and isn't
//! claimed); these tests pin down the observable contract, not the internal algorithm.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/delulu
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// A fresh, empty scratch directory outside the repo.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_tooling_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

// ===== `delulu why` ==============================================================================

#[test]
fn why_write_on_greeter_json_returns_a_path_ending_at_the_origin() {
    let o = delulu(&["why", "Write", "examples/greeter", "--json"]);
    assert!(o.status.success(), "why should exit 0: {}\n{}", stdout(&o), stderr(&o));
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap_or_else(|e| panic!("expected JSON: {e}\n{}", stdout(&o)));
    assert_eq!(v["effect"], "Write");
    assert_eq!(v["performs"], true);
    let path = v["path"].as_array().expect("path must be an array");
    assert!(!path.is_empty(), "path must not be empty: {v}");
    // `greeter::main` calls `out.println(...)` directly (a capability operation, §7.3) — no
    // callee of `main` carries `Write` itself (salutation/shout_count are pure) — so `main` is
    // the origin: the chain's last node is the function whose row actually has `Write`.
    let last = path.last().unwrap().as_str().unwrap();
    assert!(last.ends_with("::main"), "expected the chain to end at `main`, got {path:?}");
}

#[test]
fn why_net_on_greeter_reports_program_cannot_perform() {
    // greeter's `main` is declared `! {Write}` only — it does not perform `Net` anywhere, even
    // though `Net` is a recognized core effect. `why` must say so cleanly (exit 0), not error.
    let o = delulu(&["why", "Net", "examples/greeter"]);
    assert!(o.status.success(), "why should exit 0 even when the program can't perform the effect: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("cannot perform"), "{out}");
    assert!(out.contains("Net"), "{out}");
}

#[test]
fn why_unknown_effect_name_is_a_clear_error() {
    let o = delulu(&["why", "TotallyNotAnEffect", "examples/greeter"]);
    assert_eq!(o.status.code(), Some(2), "an unrecognized effect name should be a usage error");
    assert!(stderr(&o).contains("not a known effect"), "{}", stderr(&o));
}

// ===== `delulu authority --diff` =================================================================

fn write_lock(dir: &Path, name: &str, effects: &[&str], api_row_hash: &str) -> PathBuf {
    let effects_toml = effects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
    let text = format!(
        "version = 1\n\n\
         [[package]]\n\
         name           = \"webby\"\n\
         version        = \"1.0.0\"\n\
         source         = \"path+../webby\"\n\
         content_hash   = \"blake3:aaa\"\n\
         authority_hash = \"blake3:bbb\"\n\
         api_row_hash   = \"{api_row_hash}\"\n\
         effects        = [{effects_toml}]\n\
         cap_kinds      = [\"Http\"]\n\
         scopes         = {{ net = [\"api.example.com\"], \"fs.read\" = [], \"fs.write\" = [] }}\n\
         accepted_by    = \"\"\n"
    );
    let path = dir.join(name);
    write(&path, &text);
    path
}

#[test]
fn authority_diff_reports_widening_when_new_lock_gained_an_effect() {
    let dir = scratch("diff_widen");
    let old = write_lock(&dir, "old.lock", &["Net"], "blake3:ccc");
    let new = write_lock(&dir, "new.lock", &["Net", "Write"], "blake3:ddd");

    let o = delulu(&["authority", "--diff", old.to_str().unwrap(), new.to_str().unwrap(), "--json"]);
    assert!(o.status.success(), "authority --diff should exit 0: {}\n{}", stdout(&o), stderr(&o));
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap_or_else(|e| panic!("expected JSON: {e}\n{}", stdout(&o)));

    let added: Vec<&str> = v["added_effects"]["webby"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert_eq!(added, vec!["Write"], "{v}");
    assert_eq!(v["api_row_changes"]["webby"]["old_hash"], "blake3:ccc");
    assert_eq!(v["api_row_changes"]["webby"]["new_hash"], "blake3:ddd");
    let verdict = v["verdict"].as_str().unwrap();
    assert!(verdict.contains("WIDENING"), "{verdict}");
}

#[test]
fn authority_diff_reports_ok_when_nothing_widened() {
    let dir = scratch("diff_ok");
    let old = write_lock(&dir, "old.lock", &["Net"], "blake3:ccc");
    let new = write_lock(&dir, "new.lock", &["Net"], "blake3:ccc");

    let o = delulu(&["authority", "--diff", old.to_str().unwrap(), new.to_str().unwrap(), "--json"]);
    assert!(o.status.success());
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["verdict"], "OK", "{v}");
    assert!(v["added_effects"].as_object().unwrap().is_empty(), "{v}");
}

// ===== §5.5 `interface.json` =====================================================================

#[test]
fn build_emits_interface_json_with_pub_exports() {
    let iface_path = workspace_root().join("examples/greeter/target/greeter/interface.json");
    let _ = std::fs::remove_dir_all(workspace_root().join("examples/greeter/target"));

    // Plain `check` must stay side-effect-free (no interface.json).
    let checked = delulu(&["check", "examples/greeter"]);
    assert!(checked.status.success(), "{}", stderr(&checked));
    assert!(!iface_path.exists(), "`check` must not write interface.json");

    let built = delulu(&["build", "examples/greeter"]);
    assert!(built.status.success(), "build should succeed: {}", stderr(&built));
    assert!(iface_path.exists(), "expected {} to exist after build", iface_path.display());

    let text = std::fs::read_to_string(&iface_path).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("interface.json must be valid JSON: {e}\n{text}"));
    assert_eq!(v["package"], "greeter");
    let names: Vec<&str> = v["exports"].as_array().unwrap().iter().map(|e| e["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"salutation"), "{names:?}");
    assert!(names.contains(&"shout_count"), "{names:?}");
    // Both are pure helpers — their row must show up as empty, not merely present.
    for e in v["exports"].as_array().unwrap() {
        if e["name"] == "salutation" || e["name"] == "shout_count" {
            assert_eq!(e["row"], "!{}", "{e}");
        }
    }
    assert!(v["api_row_hash"].as_str().is_some_and(|s| !s.is_empty()), "{v}");
}
