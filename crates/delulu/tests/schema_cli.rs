//! P4-09: every emitter's REAL output validates against the schema `delulu schema` publishes for it.
//!
//! The schemas are closed (`additionalProperties: false`), so this is the test that fails when an
//! emitter gains a field its schema does not name, loses one the schema requires, or changes a type.
//! It runs the emitters through the binary over a small corpus chosen so that the optional shapes
//! actually occur — a foreign C block, embedded Python, a compute device, a plugin load, a `@jit`
//! hint, a diagnostic with a typed repair, an ordinary run, a sandboxed run, an audit-mode run — and
//! validates with `delulu schema validate`, the same code an agent would use.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-schema-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn delulu(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .current_dir(dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_STATE_DIR", dir.join("state"))
        .output()
        .expect("run delulu")
}

fn json_of(o: &Output) -> serde_json::Value {
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| {
        panic!("not JSON ({e}):\n{}\n{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
    })
}

/// Validate `value` as `schema` through the binary; panic with every problem it names.
fn assert_valid(dir: &Path, schema: &str, what: &str, value: &serde_json::Value) {
    let file = dir.join(format!("{what}.json"));
    std::fs::write(&file, serde_json::to_string_pretty(value).unwrap()).unwrap();
    let o = delulu(dir, &["schema", "validate", schema, file.to_str().unwrap(), "--json"]);
    let v = json_of(&o);
    assert_eq!(
        v["validate"]["valid"], true,
        "{what} is not a valid `{schema}`: {}\n{value:#}",
        v["validate"]["errors"]
    );
    assert_eq!(o.status.code(), Some(0));
}

const WRITER: &str = "module w\n\nfn main(root: Root) ! {Write} {\n    let w = root.fs_write(\"./out\")\n    let _ = w.write_text(\"made.txt\", \"x\")\n}\n";

fn corpus(dir: &Path) -> Vec<(&'static str, String)> {
    let programs: Vec<(&str, String)> = vec![
        ("writer", WRITER.to_string()),
        ("foreign_c", "module fc\n\nforeign \"c\" lib mathlib {\n  fn cos(x: Float) -> Float\n}\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"x\")\n}\n".to_string()),
        // The shipped example, not a copy: it is the program the book and the examples gate already run.
        ("python", include_str!("../../../examples/numpy_mean.delulu").to_string()),
        ("compute", "module cg\n\nfn main(root: Root) ! {Write, ForeignCall} {\n  let c = root.console()\n  let g = root.compute(\"gpu0\")\n  match g.dispatch(\"reduce_sum\", [1.0, 2.0]) {\n    Ok(v) => c.println(str(v)),\n    Err(e) => c.println(\"no\")\n  }\n}\n".to_string()),
        ("plugin", "module host\n\nfn shouted(h: Cap[PluginHost]) -> Result[Str, PluginErr] ! {Load, Read} {\n    let g = Grant {\n        effects: [], fs_read: [], fs_write: [], net: [], secrets: [], declassify: [],\n        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },\n        require_signed: false,\n    }\n    let p = load(h, \"shout.dpx\", g)?\n    let f: fn(Str) -> Str ! {} = p.get(\"shout\")?\n    Ok(f(\"hello\"))\n}\n\nfn main(root: Root) ! {Write, Load, Read} {\n    let out = root.console()\n    match shouted(root.plugin_host()) {\n        Ok(s) => out.println(s),\n        Err(e) => out.println(\"no\")\n    }\n}\n".to_string()),
        ("jit", "module jituse\n\n@jit\nfn hot(x: Int) -> Int {\n    x * 2\n}\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(str(hot(21)))\n}\n".to_string()),
    ];
    for (name, src) in &programs {
        std::fs::write(dir.join(format!("{name}.delulu")), src).unwrap();
    }
    std::fs::create_dir_all(dir.join("out")).unwrap();
    programs
}

/// The authority report and the Atlas, over every program in the corpus — each of which puts a
/// different optional shape into the output.
#[test]
fn the_authority_report_and_the_atlas_match_their_schemas() {
    let dir = scratch("authority");
    for (name, _) in corpus(&dir) {
        let file = format!("{name}.delulu");
        let o = delulu(&dir, &["authority", &file, "--json"]);
        let v = json_of(&o);
        assert_eq!(v["summary"]["errors"], 0, "{name} checks: {v:#}");
        assert_valid(&dir, "envelope", &format!("{name}-authority-envelope"), &v);
        assert_valid(&dir, "authority", &format!("{name}-authority"), &v["authority"]);

        let o = delulu(&dir, &["atlas", &file, "--json"]);
        let v = json_of(&o);
        assert_valid(&dir, "atlas", &format!("{name}-atlas"), &v["atlas"]);
    }
    // The optional shapes really occurred, or this test would pass on a corpus that exercised none.
    let seen = |name: &str| -> serde_json::Value {
        json_of(&delulu(&dir, &["authority", &format!("{name}.delulu"), "--json"]))["authority"].clone()
    };
    let abis: Vec<String> = ["foreign_c", "python", "compute"]
        .iter()
        .flat_map(|n| seen(n)["foreign_calls"].as_array().cloned().unwrap_or_default())
        .filter_map(|f| f["abi"].as_str().map(str::to_string))
        .collect();
    assert!(abis.iter().any(|a| a == "python") && abis.iter().any(|a| a == "compute") && abis.len() >= 3, "{abis:?}");
    assert!(seen("plugin")["plugins"].as_array().is_some_and(|p| !p.is_empty()), "a plugin load was reported");
    assert!(seen("jit")["native_emission"].is_object(), "the @jit request was reported");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A diagnostic with a typed repair, through `check --json`.
#[test]
fn a_diagnostic_and_its_repair_match_their_schemas() {
    let dir = scratch("diag");
    // `println` needs `Write`, which `main` does not declare: DL0501 with a typed repair.
    std::fs::write(dir.join("bad.delulu"), "module b\n\nfn main(root: Root) {\n    let c = root.console()\n    c.println(\"hi\")\n}\n").unwrap();
    let v = json_of(&delulu(&dir, &["check", "bad.delulu", "--json"]));
    let diags = v["diagnostics"].as_array().expect("diagnostics");
    assert!(diags.iter().any(|d| d["repairs"].as_array().is_some_and(|r| !r.is_empty())), "{v:#}");
    assert_valid(&dir, "envelope", "check-envelope", &v);
    for (i, d) in diags.iter().enumerate() {
        assert_valid(&dir, "diagnostic", &format!("diagnostic-{i}"), d);
        for (j, r) in d["repairs"].as_array().unwrap().iter().enumerate() {
            assert_valid(&dir, "repair", &format!("repair-{i}-{j}"), r);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The three kinds of run report: an ordinary run, a sandboxed run, and an audit-mode run; and the
/// policy preview.
#[test]
fn every_kind_of_run_report_and_the_policy_preview_match_their_schemas() {
    let dir = scratch("run");
    corpus(&dir);
    let grant = "fs.write=./out";
    for (what, extra) in [
        ("l0", vec![]),
        ("sandboxed", vec!["--sandbox"]),
        ("audit", vec!["--sandbox", "--mode", "audit"]),
    ] {
        let report = dir.join(format!("{what}.report.json"));
        let mut args = vec!["run", "writer.delulu", "--grant", grant, "--report-out", report.to_str().unwrap()];
        args.extend(extra);
        let o = delulu(&dir, &args);
        assert_eq!(o.status.code(), Some(0), "{what}: {}", String::from_utf8_lossy(&o.stderr));
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
        assert_valid(&dir, "run-report", &format!("{what}-report"), &v);
        assert_valid(&dir, "sandbox", &format!("{what}-sandbox"), &v["sandbox"]);
    }
    let v = json_of(&delulu(&dir, &["sandbox", "policy", "writer.delulu", "--json"]));
    assert_valid(&dir, "policy", "policy", &v["policy"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `toolchain --json` through the binary, and the schema listing itself.
#[test]
fn the_toolchain_description_and_the_schema_listing_are_valid() {
    let dir = scratch("toolchain");
    let v = json_of(&delulu(&dir, &["toolchain", "--json"]));
    assert_valid(&dir, "envelope", "toolchain-envelope", &v);
    assert_valid(&dir, "toolchain", "toolchain", &v["toolchain"]);
    let list = json_of(&delulu(&dir, &["schema", "--json"]));
    let names: Vec<&str> = list["schemas"].as_array().unwrap().iter().filter_map(|s| s["name"].as_str()).collect();
    for n in ["envelope", "diagnostic", "repair", "authority", "atlas", "sandbox", "policy", "run-report", "toolchain", "edit"] {
        assert!(names.contains(&n), "`{n}` is listed");
        let doc = json_of(&delulu(&dir, &["schema", n, "--json"]));
        assert_eq!(doc["document"]["$id"], format!("delulu:schema/{n}"));
    }
    // A file that is not what it claims fails validation, exit 1, with the reason.
    let bogus = dir.join("bogus.json");
    std::fs::write(&bogus, r#"{"code":"DL0101","severity":"catastrophic"}"#).unwrap();
    let o = delulu(&dir, &["schema", "validate", "diagnostic", bogus.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1));
    let v = json_of(&o);
    assert_eq!(v["validate"]["valid"], false);
    assert!(v["validate"]["errors"].to_string().contains("severity"), "{v}");
    let _ = std::fs::remove_dir_all(&dir);
}
