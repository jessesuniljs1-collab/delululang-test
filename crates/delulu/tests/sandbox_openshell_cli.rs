//! PS-E-05 (a), D-V2-82: `delulu sandbox policy <file> --format openshell` through the binary.
//!
//! The export writes the wall an OpenShell sandbox should put around `delulu run`, from the program's
//! authority and the operator's grants. Its one rule: nothing on the wall that the authority and the
//! grants do not allow — a grant the program cannot use is omitted and named, one OpenShell cannot
//! state refuses the export by name, and the one HTTP method is written out rather than taken from a
//! preset that also allows `HEAD` and `OPTIONS`. OpenShell's own prover reads the same documents in
//! `.github/workflows/openshell.yml` (`scripts/openshell-prove.sh`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-openshell-{}-{tag}", std::process::id()));
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

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Reads files, writes one, fetches from two hosts — one of them by wildcard.
const PROGRAM: &str = "module p\n\nfn main(root: Root) ! {Read, Write, Net} {\n    \
    let src = root.fs_read(\"./data\")\n    let out = root.fs_write(\"./out\")\n    \
    let h = root.http([\"api.example.com\", \"*.cdn.example.org\"])\n    \
    let t = src.read_text(\"./data/in.txt\")\n    let r = h.get(\"https://api.example.com/v1\")\n    \
    let _ = out.write_text(\"./out/x.txt\", \"hi\")\n}\n";

/// Prints, and nothing else.
const PRINTER: &str = "module q\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"hi\")\n}\n";

const GRANTS: [&str; 5] = ["fs.read=./data", "fs.write=./out", "net=api.example.com", "net=*.cdn.example.org", "console"];

fn export(dir: &Path, file: &str, extra: &[&str]) -> Output {
    let mut args = vec!["sandbox", "policy", file, "--format", "openshell"];
    args.extend_from_slice(extra);
    delulu(dir, &args)
}

fn with_grants<'a>(grants: &[&'a str], rest: &[&'a str]) -> Vec<&'a str> {
    let mut v = Vec::new();
    for g in grants {
        v.push("--grant");
        v.push(*g);
    }
    v.extend_from_slice(rest);
    v
}

/// The whole wall, as the document states it and as `--json` accounts for it — and valid against the
/// published schema, through the binary.
#[test]
fn the_wall_is_the_programs_authority_within_its_grants_and_nothing_more() {
    let dir = scratch("wall");
    std::fs::write(dir.join("p.delulu"), PROGRAM).unwrap();
    let args = with_grants(&GRANTS, &["--workdir", "/sandbox", "--json"]);
    let o = export(&dir, "p.delulu", &args);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let v: Value = serde_json::from_slice(&o.stdout).expect("one envelope");
    let e = &v["openshell"];
    let p = &e["policy"];
    assert_eq!(p["version"], 1);
    assert_eq!(
        p["filesystem_policy"],
        json!({
            "include_workdir": false,
            "read_only": ["/etc", "/lib", "/sandbox/data", "/sandbox/p.delulu", "/usr"],
            "read_write": ["/sandbox/out"],
        }),
        "the runtime's paths, the program's file, and the two grants — nothing else: {e:#}"
    );
    assert_eq!(p["landlock"]["compatibility"], "hard_requirement");
    assert_eq!(p["process"], json!({ "run_as_user": "sandbox", "run_as_group": "sandbox" }));
    let rules = p["network_policies"].as_object().unwrap();
    let mut hosts: Vec<&str> = Vec::new();
    for (_, rule) in rules {
        assert_eq!(rule["binaries"], json!([{ "path": "/usr/local/bin/delulu" }]), "one binary, delulu: {rule}");
        for ep in rule["endpoints"].as_array().unwrap() {
            hosts.push(ep["host"].as_str().unwrap());
            assert_eq!(ep["port"], 443);
            assert_eq!(ep["protocol"], "rest");
            assert_eq!(ep["enforcement"], "enforce", "an audit-mode endpoint blocks nothing: {ep}");
            assert!(ep.get("access").is_none(), "no preset — `read-only` also allows HEAD and OPTIONS: {ep}");
            assert_eq!(
                ep["rules"],
                json!([{ "allow": { "method": "GET", "path": "/" } }, { "allow": { "method": "GET", "path": "/**" } }]),
                "GET on every path, and no other method: {ep}"
            );
        }
    }
    hosts.sort();
    assert_eq!(hosts, ["**.cdn.example.org", "api.example.com"], "DeluluLang's `*.` is OpenShell's `**.`");
    // Every grant accounted for; the console and the budget are what OpenShell does not model.
    let grants_in = |k: &str| -> Vec<String> {
        e[k].as_array().unwrap().iter().map(|m| m["grant"].as_str().or(m["what"].as_str()).unwrap().to_string()).collect()
    };
    assert_eq!(grants_in("emitted").len(), 4, "{e:#}");
    assert!(grants_in("unrepresented").contains(&"console".to_string()), "{e:#}");
    assert!(grants_in("unrepresented").contains(&"budget".to_string()), "{e:#}");
    assert!(grants_in("omitted").is_empty(), "{e:#}");
    assert_eq!(grants_in("narrowed").len(), 2, "port 443 is narrower than `net=`, and says so: {e:#}");
    // The YAML says what the JSON says.
    let doc = e["document"].as_str().unwrap();
    for path in p["filesystem_policy"]["read_only"].as_array().unwrap().iter().chain(p["filesystem_policy"]["read_write"].as_array().unwrap()) {
        assert!(doc.contains(&format!("\n    - \"{}\"\n", path.as_str().unwrap())), "`{path}` missing from:\n{doc}");
    }
    for h in &hosts {
        assert!(doc.contains(&format!("      - host: \"{h}\"\n")), "`{h}` missing from:\n{doc}");
    }
    for word in ["HEAD", "OPTIONS", "access:", "audit", "include_workdir: true"] {
        assert!(!doc.contains(word), "`{word}` in the document:\n{doc}");
    }
    assert_eq!(doc.matches("method: \"GET\"").count(), 4, "{doc}");
    // Without --json the document alone, byte for byte.
    let human = export(&dir, "p.delulu", &with_grants(&GRANTS, &["--workdir", "/sandbox"]));
    assert_eq!(human.status.code(), Some(0));
    assert_eq!(text(&human.stdout), doc);
    // The published schema, through the binary.
    let f = dir.join("openshell.json");
    std::fs::write(&f, serde_json::to_string_pretty(e).unwrap()).unwrap();
    let val = delulu(&dir, &["schema", "validate", "openshell", f.to_str().unwrap(), "--json"]);
    let vv: Value = serde_json::from_slice(&val.stdout).unwrap();
    assert_eq!(vv["validate"]["valid"], true, "{}", vv["validate"]["errors"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A grant the program's authority cannot use is not written — it is named as omitted.
#[test]
fn a_grant_the_program_cannot_use_is_omitted_and_named() {
    let dir = scratch("omitted");
    std::fs::write(dir.join("q.delulu"), PRINTER).unwrap();
    let o = export(&dir, "q.delulu", &with_grants(&["console", "fs.read=./data", "fs.write=./out", "net=example.com"], &["--workdir", "/w", "--json"]));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let e = &v["openshell"];
    assert_eq!(e["policy"]["filesystem_policy"]["read_only"], json!(["/etc", "/lib", "/usr", "/w/q.delulu"]), "{e:#}");
    assert_eq!(e["policy"]["filesystem_policy"]["read_write"], json!([]));
    assert_eq!(e["policy"]["network_policies"], json!({}));
    let omitted: Vec<&str> = e["omitted"].as_array().unwrap().iter().map(|m| m["grant"].as_str().unwrap()).collect();
    assert_eq!(omitted, ["fs.read=./data", "fs.write=./out", "net=example.com"], "{e:#}");
    let doc = e["document"].as_str().unwrap();
    assert!(doc.contains("# omitted: \"net=example.com\""), "the document says so too:\n{doc}");
    assert!(doc.contains("network_policies: {}\n"), "{doc}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A grant OpenShell's format cannot state refuses the export by name — exit 1, nothing on stdout.
#[test]
fn a_grant_openshell_cannot_state_refuses_the_export_by_name() {
    let dir = scratch("refused");
    std::fs::write(dir.join("p.delulu"), PROGRAM).unwrap();
    for (grant, word) in [
        ("net.special=10.0.0.7", "special-use"),
        ("sensor=arm0/angle", "device"),
        ("plugin=./plugins", "loads code"),
        ("fs.write=/", "whole filesystem"),
        ("net=*.org", "top-level domain"),
    ] {
        let o = export(&dir, "p.delulu", &with_grants(&["fs.read=./data", grant], &["--workdir", "/sandbox"]));
        assert_eq!(o.status.code(), Some(1), "{grant}: {}", text(&o.stderr));
        assert!(o.stdout.is_empty(), "{grant}: no document may be printed: {}", text(&o.stdout));
        let err = text(&o.stderr);
        assert!(err.contains(grant) && err.contains(word), "{grant}: the refusal names the grant and why:\n{err}");
        assert!(err.contains("never wider"), "{err}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// OpenShell takes absolute paths; this machine's working directory is the wrong machine's. A relative
/// path needs `--workdir`; `..` is refused rather than collapsed; the shaping flags need the format.
#[test]
fn paths_are_the_sandboxs_and_every_malformed_request_is_refused() {
    let dir = scratch("usage");
    std::fs::write(dir.join("p.delulu"), PROGRAM).unwrap();
    let abs = dir.join("p.delulu");
    let abs = abs.to_str().unwrap();
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["--grant", "console"], "--workdir"),
        (vec!["--workdir", "/sandbox", "--grant", "fs.read=./data/../../etc"], ".."),
        (vec!["--workdir", "sandbox"], "absolute"),
        (vec!["--workdir", "/sandbox", "--run-as", "0"], "root"),
        (vec!["--workdir", "/sandbox", "--binary", "delulu"], "--binary"),
        (vec!["--workdir", "/sandbox", "--grant", "fs.read="], "empty"),
        (vec!["--workdir", "/sandbox", "--grant"], "needs a value"),
    ];
    for (extra, word) in cases {
        let o = export(&dir, "p.delulu", &extra);
        assert_eq!(o.status.code(), Some(2), "{extra:?}: {}", text(&o.stderr));
        assert!(o.stdout.is_empty(), "{extra:?}");
        assert!(text(&o.stderr).contains(word), "{extra:?}: `{word}` not in: {}", text(&o.stderr));
    }
    // The shaping flags without the format, and an unknown format.
    for extra in [vec!["--grant", "console"], vec!["--workdir", "/w"], vec!["--format", "yaml"]] {
        let mut args = vec!["sandbox", "policy", "p.delulu"];
        args.extend(extra.iter());
        let o = delulu(&dir, &args);
        assert_eq!(o.status.code(), Some(2), "{extra:?}: {}", text(&o.stderr));
    }
    // An absolute program path needs no workdir; the program's own file is on the wall.
    let o = export(&dir, abs, &["--grant", "fs.read=/srv/data", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let ro = &v["openshell"]["policy"]["filesystem_policy"]["read_only"];
    assert!(ro.as_array().unwrap().iter().any(|p| p == abs) && ro.as_array().unwrap().iter().any(|p| p == "/srv/data"), "{ro}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A secret's value never reaches the document or the envelope, and a path's quote cannot be closed.
#[test]
fn a_secret_is_named_never_shown_and_every_string_stays_quoted() {
    let dir = scratch("quoted");
    std::fs::write(dir.join("p.delulu"), PROGRAM).unwrap();
    let weird = "fs.read=./we\"ird: [x] #y";
    let o = export(
        &dir,
        "p.delulu",
        &with_grants(&["secret:TOKEN=hunter2-value", weird], &["--workdir", "/sandbox", "--json"]),
    );
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    assert!(!out.contains("hunter2"), "a secret's value reached the output:\n{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    let e = &v["openshell"];
    assert!(e["unrepresented"].as_array().unwrap().iter().any(|m| m["what"] == "secret:TOKEN"), "{e:#}");
    let doc = e["document"].as_str().unwrap();
    assert!(doc.contains("\n    - \"/sandbox/we\\\"ird: [x] #y\"\n"), "the path, escaped and whole:\n{doc}");
    let _ = std::fs::remove_dir_all(&dir);
}
