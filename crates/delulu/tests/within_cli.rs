//! P9-01 — `delulu authority FILE --within BOUNDARY` (`V2_OPENSHELL_STUDY.md` §4.8, `V2_P9_DESIGN.md`),
//! witnessed against the corpus the design measured: every program in `examples/`, `examples/guide/` and
//! `docs/book/samples/` is WITHIN its own required grants, and EXCEEDS each one-grant narrowing — with
//! the dropped grant as the counterexample and, as its location, the very call that asks for it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .args(args)
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("failed to run delulu")
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn json_of(o: &Output) -> Value {
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("not one JSON value ({e}):\n{}", out(o)))
}

/// `authority FILE --json` → its `required_grants`.
fn required(file: &str) -> Vec<String> {
    let o = delulu(&["authority", file, "--json"]);
    assert_eq!(o.status.code(), Some(0), "{file}: {}", out(&o));
    json_of(&o)["authority"]["required_grants"]
        .as_array()
        .unwrap_or_else(|| panic!("{file}: no required_grants"))
        .iter()
        .map(|g| g.as_str().unwrap().to_string())
        .collect()
}

/// `authority FILE --within G… --json` → (exit code, the `within` object).
fn within(file: &str, boundary: &[String]) -> (i32, Value) {
    let mut args: Vec<&str> = vec!["authority", file];
    for g in boundary {
        args.push("--within");
        args.push(g);
    }
    // The EMPTY boundary has no `--within` flag to spell it: it is an empty `--within-file`.
    let empty = std::env::temp_dir().join(format!("delulu-within-empty-{}.grants", std::process::id()));
    let empty_s = empty.to_string_lossy().to_string();
    if boundary.is_empty() {
        std::fs::write(&empty, "# grants nothing\n").unwrap();
        args.push("--within-file");
        args.push(&empty_s);
    }
    args.push("--json");
    let o = delulu(&args);
    let code = o.status.code().unwrap_or(-1);
    let v = json_of(&o);
    (code, v["authority"]["within"].clone())
}

/// A requirement as a boundary grant: its operator's data filled in (a secret's VALUE, a library's
/// PATH, an actuator's envelope — the names stay the program's). `None` for a scope the source does not
/// show, which no boundary line can name.
fn as_boundary(req: &str) -> Option<String> {
    if let Some(name) = req.strip_prefix("secret:").and_then(|r| r.strip_suffix("=VALUE")) {
        return (name != "NAME").then(|| format!("secret:{name}=v"));
    }
    if let Some(lib) = req.strip_prefix("foreign.c=").and_then(|r| r.strip_suffix(":PATH")) {
        return (lib != "LOGICAL").then(|| format!("foreign.c={lib}:/opt/lib/lib{lib}.so"));
    }
    if let Some(dev) = req.strip_prefix("actuator=").and_then(|r| r.strip_suffix(":DIM=lo..hi")) {
        return (dev != "DEVICE")
            .then(|| format!("actuator={dev}:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold"));
    }
    let placeholder = ["=PATH", "=HOST", "=DEVICE", "=PATTERN"].iter().any(|p| req.ends_with(p));
    (!placeholder).then(|| req.to_string())
}

/// The call a requirement's counterexample must point at: the primitive that asks for it.
fn method_of(grant: &str) -> &'static str {
    let key = grant.split(['=', ':']).next().unwrap();
    match key {
        "fs.read" => "fs_read(",
        "fs.write" => "fs_write(",
        "net" | "net.special" => "http(",
        "console" => "console(",
        "clock" => "clock(",
        "rand" => "rand(",
        // `root.declassify()`, or a secret's `verify`/`expose` where nothing mints it.
        "declassify" => "",
        "secret" => "secret(",
        "actuator" => "actuator(",
        "sensor" => "sensor(",
        "compute" => "compute(",
        "foreign.python" => "python(",
        "foreign.c" => "foreign",
        other => panic!("no method known for `{other}`"),
    }
}

fn corpus() -> Vec<String> {
    let root = workspace_root();
    let mut files = Vec::new();
    for dir in ["examples", "examples/guide", "docs/book/samples"] {
        let mut here: Vec<String> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "delulu"))
            .map(|p| format!("{dir}/{}", p.file_name().unwrap().to_string_lossy()))
            .collect();
        here.sort();
        files.extend(here);
    }
    files
}

#[test]
fn every_corpus_program_is_within_its_own_grants_and_exceeds_each_narrowing_at_the_call_that_asks() {
    let mut programs = 0;
    let mut narrowings = 0;
    for file in corpus() {
        let req = required(&file);
        let boundary: Vec<String> = req.iter().filter_map(|r| as_boundary(r)).collect();
        if boundary.len() != req.len() {
            continue; // a scope the source does not show: the next test's case
        }
        programs += 1;
        let (code, v) = within(&file, &boundary);
        assert_eq!((code, v["result"].as_str()), (0, Some("within")), "{file} within {boundary:?}: {v:#}");
        assert!(v["counterexample"].is_null() && v["findings"].as_array().unwrap().is_empty(), "{file}: {v:#}");
        for (i, dropped) in req.iter().enumerate() {
            let narrower: Vec<String> = boundary.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, g)| g.clone()).collect();
            let (code, v) = within(&file, &narrower);
            assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{file} without `{dropped}`: {v:#}");
            let c = &v["counterexample"];
            assert_eq!(c["grant"].as_str(), Some(dropped.as_str()), "{file}: the counterexample is the dropped grant: {v:#}");
            let line = c["at"]["line"].as_u64().unwrap_or_else(|| panic!("{file}: `{dropped}` names no call: {v:#}"));
            let text = std::fs::read_to_string(workspace_root().join(&file)).unwrap();
            let src_line = text.lines().nth(line as usize - 1).unwrap();
            let method = method_of(dropped);
            let named = if method.is_empty() {
                ["declassify(", "verify(", "expose("].iter().any(|m| src_line.contains(m))
            } else {
                src_line.contains(method)
            };
            assert!(named, "{file}: `{dropped}`'s counterexample points at line {line}, `{src_line}` — not its call");
            narrowings += 1;
        }
    }
    // The control (§11.5, run 19): a corpus read in which nothing was decided proves nothing.
    assert!(programs >= 15 && narrowings >= 15, "{programs} program(s), {narrowings} narrowing(s)");
}

/// A scope the source does not show is never answered `within` by a boundary that does not bound its
/// whole kind: `exceeds` against a boundary that grants none of it, `inconclusive` against one that
/// grants some — each named, with the site when there is one.
#[test]
fn a_scope_the_source_computes_is_exceeds_against_none_and_inconclusive_against_some() {
    let file = "docs/book/samples/10_actuate_errors.delulu";
    assert_eq!(required(file), ["actuator=DEVICE:DIM=lo..hi"], "the design's class-(b) program");
    let (code, v) = within(file, &["console".to_string()]);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    assert_eq!(v["counterexample"]["grant"], "actuator=DEVICE:DIM=lo..hi", "{v:#}");
    let arm = "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold".to_string();
    let (code, v) = within(file, &[arm]);
    assert_eq!((code, v["result"].as_str()), (1, Some("inconclusive")), "a partial device boundary: {v:#}");
    assert_eq!(v["coverage"]["actuator"], "inconclusive", "{v:#}");

    // A secret read under a computed name (SECRET-HIDDEN-1): the site is the call.
    let dir = std::env::temp_dir().join(format!("delulu-within-hidden-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("hidden_secret.delulu");
    std::fs::write(
        &f,
        "module hidden_secret\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    \
         let name = \"API_\" + \"KEY\"\n    let s = root.secret(name)\n    out.println(\"got\")\n}\n",
    )
    .unwrap();
    let f = f.to_str().unwrap();
    let (code, v) = within(f, &["console".to_string()]);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "no secret in the boundary: {v:#}");
    assert_eq!((v["counterexample"]["grant"].as_str(), v["counterexample"]["at"]["line"].as_u64()), (Some("secret:NAME=VALUE"), Some(6)), "{v:#}");
    let (code, v) = within(f, &["console".to_string(), "secret:API_KEY=v".to_string()]);
    assert_eq!((code, v["result"].as_str()), (1, Some("inconclusive")), "the right name, but not one the checker read: {v:#}");

    // A literal that exceeds beside scopes no reading decides: the verdict is the definite one (M276).
    let g = dir.join("mixed.delulu");
    std::fs::write(
        &g,
        "module mixed\n\nfn pick(n: Int) -> Str {\n    if n > 1 { \"./a\" } else { \"b.example.com\" }\n}\n\n\
         fn main(root: Root) ! {Read, Net} {\n    let one = root.fs_read(\"./data\")\n    \
         let two = root.fs_read(pick(2))\n    let h = root.http([\"example.com\", pick(1)])\n}\n",
    )
    .unwrap();
    let (code, v) = within(g.to_str().unwrap(), &["fs.read=./other".to_string(), "net=example.com".to_string()]);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    let results: Vec<&str> = v["findings"].as_array().unwrap().iter().map(|f| f["result"].as_str().unwrap()).collect();
    assert!(results.contains(&"exceeds") && results.contains(&"inconclusive"), "both kinds of finding are named: {v:#}");
    assert_eq!(v["counterexample"]["grant"], "fs.read=./data", "{v:#}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Against the empty boundary every request of `05_capabilities` exceeds, and they are named in SOURCE
/// order — `root.console()` first, on line 44 — which is neither the order `--grants` lists them in
/// (`fs.read`, `fs.write`, `console`, `net`) nor its reverse.
#[test]
fn every_request_is_named_in_source_order_and_the_first_call_is_the_counterexample() {
    let (code, v) = within("examples/guide/05_capabilities.delulu", &[]);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    let order: Vec<(&str, u64)> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["grant"].as_str().unwrap(), f["at"]["line"].as_u64().unwrap()))
        .collect();
    assert_eq!(
        order,
        [("console", 44), ("fs.read=./config", 48), ("fs.write=./out", 49), ("net=example.com", 50)],
        "{v:#}"
    );
    assert_eq!(v["counterexample"]["grant"], "console", "{v:#}");
}

#[test]
fn a_boundary_file_says_what_the_flags_say_and_every_form_that_cannot_answer_refuses() {
    let file = "examples/guide/05_capabilities.delulu";
    let dir = std::env::temp_dir().join(format!("delulu-within-file-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let b = dir.join("boundary.grants");
    std::fs::write(&b, "# the agent's boundary\nfs.read=./config\n\nconsole\nnet=example.com\n").unwrap();
    let o = delulu(&["authority", file, "--within-file", b.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let text = out(&o);
    assert!(text.starts_with("exceeds:") && text.contains("fs.write=./out") && text.contains("05_capabilities.delulu:49:"), "{text}");
    std::fs::write(&b, "fs.read=./config\nfs.write=./out\nconsole\nnet=example.com\n").unwrap();
    let o = delulu(&["authority", file, "--within-file", b.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(out(&o).starts_with("within:"), "{}", out(&o));

    // Each form that does not answer the question refuses it by name — never ignores it (exit 2).
    let lock = dir.join("old.lock");
    std::fs::write(&lock, "").unwrap();
    for (why, args) in [
        ("--diff", vec!["authority", "--diff", lock.to_str().unwrap(), "examples/line_driver", "--within", "console"]),
        ("--grants", vec!["authority", file, "--grants", "--within", "console"]),
        ("a package", vec!["authority", "examples/line_driver", "--within", "console"]),
        ("a bad grant", vec!["authority", file, "--within", "fs.read="]),
        ("no value", vec!["authority", file, "--within"]),
        ("another command", vec!["check", file, "--within", "console"]),
    ] {
        let o = delulu(&args);
        assert_eq!(o.status.code(), Some(2), "{why}: {}", out(&o));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The `within` object is part of the authority report's schema, closed like every object there: both
/// answers validate through the binary's own validator.
#[test]
fn both_answers_validate_against_the_authority_schema() {
    let dir = std::env::temp_dir().join(format!("delulu-within-schema-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (label, boundary) in [("within", vec!["fs.read=./config", "console"]), ("exceeds", vec!["console"])] {
        let mut args = vec!["authority", "docs/book/samples/06_authority_report.delulu"];
        for g in &boundary {
            args.push("--within");
            args.push(g);
        }
        args.push("--json");
        let v = json_of(&delulu(&args));
        assert_eq!(v["authority"]["within"]["result"], label, "{v:#}");
        let f = dir.join(format!("{label}.json"));
        std::fs::write(&f, serde_json::to_string_pretty(&v["authority"]).unwrap()).unwrap();
        let o = delulu(&["schema", "validate", "authority", f.to_str().unwrap(), "--json"]);
        let r = json_of(&o);
        assert_eq!(r["validate"]["valid"], true, "{label}: {}", r["validate"]["errors"]);
    }
    let _ = std::fs::remove_dir_all(&dir);
}
