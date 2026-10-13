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

/// P9-02 (`V2_OPENSHELL_STUDY.md` §4.9): `delulu grants diff OLD NEW` names what a change of grant set
/// ADDS, by category and with no severity grade — every grant form, read through the generator of its
/// own documented example: against an empty OLD each example is one finding of its own dimension, and
/// against itself none. Exit 1 when the change adds anything, 0 when it adds nothing.
#[test]
fn grants_diff_names_what_a_change_adds_and_nothing_it_does_not() {
    let dir = std::env::temp_dir().join(format!("delulu-grants-diff-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let write = |name: &str, lines: &[&str]| -> String {
        let f = dir.join(name);
        std::fs::write(&f, lines.iter().map(|l| format!("{l}\n")).collect::<String>()).unwrap();
        f.to_string_lossy().to_string()
    };
    let diff = |old: &str, new: &str| -> (i32, Value) {
        let o = delulu(&["grants", "diff", old, new, "--json"]);
        let code = o.status.code().unwrap_or(-1);
        // `grants` verbs print their fields beside the envelope's own.
        (code, json_of(&o))
    };
    // The grant grammar's own examples, read from the binary (`toolchain --json`), so a form added
    // later is in this witness the day it exists.
    let tc = json_of(&delulu(&["toolchain", "--json"]));
    let forms: Vec<(String, String)> = tc["toolchain"]["grants"]
        .as_array()
        .unwrap_or_else(|| panic!("toolchain lists no grant forms: {tc}"))
        .iter()
        .map(|f| (f["key"].as_str().unwrap().to_string(), f["example"].as_str().unwrap().to_string()))
        .collect();
    assert!(forms.len() >= 15, "{forms:?}");
    let empty = write("empty.grants", &["# grants nothing"]);
    for (key, example) in &forms {
        if example.contains("env:") {
            continue; // a secret read from the environment: its variable is unset here
        }
        let one = write("one.grants", &[example]);
        let (code, v) = diff(&empty, &one);
        assert_eq!(code, 1, "`{example}` added to nothing: {v:#}");
        let found = v["findings"].as_array().unwrap();
        assert_eq!(found.len(), 1, "`{example}`: {v:#}");
        let want_dim = key.trim_end_matches(':');
        assert_eq!(found[0]["dimension"], want_dim, "`{example}`: {v:#}");
        let want_cat = match want_dim {
            "net.special" => "special_use_reach",
            "actuator" | "sensor" | "compute" => "device_reach_expansion",
            _ => "effect_expansion",
        };
        assert_eq!(found[0]["category"], want_cat, "`{example}`: {v:#}");
        let (code, v) = diff(&one, &one);
        assert_eq!((code, v["findings"].as_array().unwrap().len()), (0, 0), "`{example}` against itself: {v:#}");
    }
    // A wider scope in a kind OLD already grants is a scope expansion; a narrower one is no finding.
    let old = write("old.grants", &["fs.read=./data", "net=example.com", "actuator=arm0/elbow:angle_deg=-30..30,heartbeat_ms=200,ttl_ms=60000,fail=hold"]);
    let new = write(
        "new.grants",
        &["fs.read=./data/sub", "fs.read=./other", "net=example.com", "net=api.example.com",
          "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold"],
    );
    let (code, v) = diff(&old, &new);
    assert_eq!(code, 1, "{v:#}");
    let got: Vec<(String, String)> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["category"].as_str().unwrap().to_string(), f["grant"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(
        got,
        [
            ("scope_expansion".to_string(), "fs.read=./other".to_string()),
            ("scope_expansion".to_string(), "net=api.example.com".to_string()),
            ("device_reach_expansion".to_string(), "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold".to_string()),
        ],
        "the narrower `./data/sub` and the unchanged host are no finding; a widened envelope is: {v:#}"
    );
    // The reverse is no narrowing (`./data` is wider than `./data/sub`) — and is named as one scope expansion.
    let (code, v) = diff(&new, &old);
    assert_eq!((code, v["findings"][0]["grant"].as_str()), (1, Some("fs.read=./data")), "{v:#}");
    // A change that only narrows — a subdirectory, the same host, a tighter envelope — adds nothing.
    let narrower = write(
        "narrower.grants",
        &["fs.read=./data/sub", "net=example.com", "actuator=arm0/elbow:angle_deg=-10..10,heartbeat_ms=200,ttl_ms=60000,fail=hold"],
    );
    let (code, v) = diff(&old, &narrower);
    assert_eq!((code, v["findings"].as_array().unwrap().len(), v["adds"].as_bool()), (0, 0, Some(false)), "{v:#}");
    // An accelerator's envelope: a larger memory ceiling, another kernel, a waived attestation — each is reach added.
    let gpu = "compute=gpu0:memory_bytes=1048576,kernel_ms=0..50,queue_depth=4,power_w=0..120,adapter=cpu-reference,format=refkernel-1,kernel=reduce_sum:k.refkernel";
    let base_g = write("gpu.grants", &[gpu]);
    for (why, wider) in [
        ("memory", gpu.replace("memory_bytes=1048576", "memory_bytes=2097152")),
        ("a kernel", format!("{gpu},kernel=scale:s.refkernel")),
        ("a waiver", format!("{gpu},waive-attestation")),
    ] {
        let f = write("gpu-wider.grants", &[&wider]);
        let (code, v) = diff(&base_g, &f);
        assert_eq!((code, v["findings"][0]["category"].as_str()), (1, Some("device_reach_expansion")), "{why}: {v:#}");
    }
    let tighter = write("gpu-tighter.grants", &[&gpu.replace("memory_bytes=1048576", "memory_bytes=4096")]);
    assert_eq!(diff(&base_g, &tighter).0, 0, "a smaller ceiling adds nothing");
    // The human answer names the same, and a lease node is refused by name until it is diffed (exit 2).
    let o = delulu(&["grants", "diff", &old, &new]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(out(&o).contains("device_reach_expansion") && out(&o).contains("fs.read=./other"), "{}", out(&o));
    let o = delulu(&["grants", "diff", "g_1", &new]);
    assert!(o.status.code() == Some(2) && out(&o).contains("not a file with a node"), "a node beside a file is refused: {}", out(&o));
    for args in [vec!["grants", "diff", &old], vec!["grants", "diff", &old, &new, "--bogus"]] {
        let o = delulu(&args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", out(&o));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Stops a test's broker on drop, from a stable directory, never panicking.
struct BrokerGuard(PathBuf);
impl Drop for BrokerGuard {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.0)
            .args(["broker", "stop"])
            .output();
    }
}

/// P9-02 between two lease NODES: each node's authority comes from the broker that holds it and is compared by
/// the lattice — effects, paths, hosts, a device's envelope and the budget (absent is the top of that dimension).
#[test]
fn grants_diff_between_two_nodes_names_effect_scope_device_and_budget_reach() {
    // Short: a broker's socket path is bounded (104 bytes on macOS).
    let base = std::env::temp_dir().join(format!("dgd{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    for d in ["data/sub", "other"] {
        std::fs::create_dir_all(cwd.join(d)).unwrap();
    }
    std::fs::create_dir_all(&state).unwrap();
    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(&cwd)
            .env("DELULU_STATE_DIR", &state)
            .env("DELULU_NO_COLOR", "1")
            .args(args)
            .output()
            .expect("failed to run delulu")
    };
    let o = run(&["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", out(&o));
    let guard = BrokerGuard(state.clone());
    let delegate = |extra: &[&str]| -> String {
        let mut args = vec!["grants", "delegate", "--ttl", "1h", "--json"];
        args.extend_from_slice(extra);
        let o = run(&args);
        assert!(o.status.success(), "delegate {extra:?}: {}", out(&o));
        json_of(&o)["node"].as_str().unwrap().to_string()
    };
    let arm = "arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold";
    let old = delegate(&["--effects", "Read,Net", "--fs-read", "./data", "--net", "example.com", "--budget", "mem=1048576,cpu=10"]);
    let new = delegate(&[
        "--effects", "Read,Write,Net,Actuate", "--fs-read", "./data/sub", "--fs-read", "./other", "--fs-write", "./data",
        "--net", "example.com", "--net", "api.example.com", "--device", arm, "--budget", "mem=2097152,cpu=10",
    ]);
    let narrow_arm = delegate(&["--effects", "Actuate", "--device", "arm0/elbow:angle_deg=-10..10,heartbeat_ms=200,ttl_ms=60000,fail=hold"]);
    let unbudgeted = delegate(&["--effects", "Read,Net", "--fs-read", "./data", "--net", "example.com"]);
    let diff = |a: &str, b: &str| -> (i32, Vec<(String, String)>) {
        let o = run(&["grants", "diff", a, b, "--json"]);
        let code = o.status.code().unwrap_or(-1);
        let v = json_of(&o);
        assert_eq!(v["adds"].as_bool(), Some(code == 1), "{v:#}");
        let found = v["findings"]
            .as_array()
            .unwrap_or_else(|| panic!("{a} {b}: {v:#}"))
            .iter()
            .map(|f| (f["category"].as_str().unwrap().to_string(), f["grant"].as_str().unwrap().to_string()))
            .collect();
        (code, found)
    };
    let (code, found) = diff(&old, &new);
    assert_eq!(code, 1, "{found:?}");
    let cats: Vec<&str> = found.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(
        cats,
        ["effect_expansion", "effect_expansion", "scope_expansion", "scope_expansion", "scope_expansion", "device_reach_expansion", "budget_expansion"],
        "{found:?}"
    );
    let grants: Vec<&str> = found.iter().map(|(_, g)| g.as_str()).collect();
    assert!(grants.contains(&"Write") && grants.contains(&"Actuate") && grants.contains(&"net=api.example.com"), "{found:?}");
    assert!(grants.iter().any(|g| g.starts_with("fs.read=") && g.ends_with("other")), "{found:?}");
    assert!(!grants.iter().any(|g| g.ends_with("sub")), "a subdirectory of OLD's scope adds nothing: {found:?}");
    // The reverse: OLD's `./data` is wider than NEW's `./data/sub` — the one thing it adds.
    let (code, found) = diff(&new, &old);
    assert_eq!((code, found.len()), (1, 1), "{found:?}");
    assert!(found[0].0 == "scope_expansion" && found[0].1.ends_with("data"), "{found:?}");
    // A wider envelope on a device both grant; no budget beside a budget (absent is the top).
    let o = run(&["grants", "diff", &narrow_arm, &new, "--json"]);
    let v = json_of(&o);
    let wider = v["findings"].as_array().unwrap().iter().find(|f| f["category"] == "device_reach_expansion").cloned();
    assert!(wider.is_some_and(|f| f["detail"].as_str().unwrap().contains("wider")), "{v:#}");
    let (code, found) = diff(&old, &unbudgeted);
    assert_eq!((code, found.len(), found[0].0.as_str()), (1, 1, "budget_expansion"), "{found:?}");
    assert_eq!(diff(&old, &old), (0, vec![]));
    // A file beside a node is two domains: refused (exit 2).
    std::fs::write(cwd.join("g.grants"), "console\n").unwrap();
    let o = run(&["grants", "diff", "g.grants", &old]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    // The broker is stopped BEFORE its state directory goes: `broker stop` finds the daemon through that directory,
    // and a guard dropped after the removal left the daemon running past the test (seen as an orphan `delulu` the
    // runners killed at cleanup, and three in the VM).
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-03's build step 1 (`V2_P9_DESIGN.md`): a refused run's report records the refusal and the call it was made at — the
/// span P9-01's requirement list maps to the narrowest grant (here `fs.read=./config` at 48:18, the same place
/// `authority --within` names). Before, `run --report-out` wrote `diagnostics: []` for a refused run and the refusal reached
/// only the terminal, in prose.
#[test]
fn a_refused_runs_report_records_the_refusal_at_the_call_the_verdict_names() {
    let dir = std::env::temp_dir().join(format!("delulu-refusal-report-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let report = dir.join("report.json");
    let file = "examples/guide/05_capabilities.delulu";
    let o = delulu(&["run", file, "--grant", "console", "--report-out", report.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let r: Value = serde_json::from_str(&std::fs::read_to_string(&report).expect("a report is written")).unwrap();
    let d = &r["diagnostics"][0];
    assert_eq!(d["code"], "DL0703", "the refusal is in the report: {r:#}");
    let start = &d["spans"][0]["start"];
    let (line, col) = (start["line"].as_u64(), start["col"].as_u64());
    let (code, v) = within(file, &["console".to_string()]);
    assert_eq!(code, 1);
    let at = &v["counterexample"]["at"];
    assert_eq!((line, col), (at["line"].as_u64(), at["column"].as_u64()), "the report's call is the verdict's: {r:#}\n{v:#}");
    assert_eq!(v["counterexample"]["grant"], "fs.read=./config");
    assert_eq!(r["summary"]["errors"], 1, "{r:#}");
    // A clean run's report still carries none.
    let o = delulu(&["run", "docs/book/samples/01_hello.delulu", "--grant", "console", "--report-out", report.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let r: Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(r["diagnostics"], serde_json::json!([]), "{r:#}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// P9-03's next step (`V2_P9_DESIGN.md`): a SANDBOXED run's report records the refusal too. Its report is written by the host,
/// which refused the call itself, and the call is the span the guest's request named — the same call, in the same file, as
/// the ordinary run's report (step 1) records: for a file and for a package's flattened text alike. Before, the host wrote
/// `diagnostics: []` for a refused run and the refusal reached only the guest's standard error, in prose.
#[test]
fn a_sandboxed_refused_runs_report_records_the_hosts_refusal_at_the_ordinary_runs_call() {
    let dir = std::env::temp_dir().join(format!("delulu-sandbox-refusal-report-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (plain, boxed) = (dir.join("plain.json"), dir.join("sandboxed.json"));
    let read = |p: &Path| -> Value { serde_json::from_str(&std::fs::read_to_string(p).expect("a report is written")).unwrap() };
    for (target, grants) in [("examples/guide/05_capabilities.delulu", &["--grant", "console"][..]), ("examples/greeter", &[][..])] {
        let mut args = vec!["run", target];
        args.extend_from_slice(grants);
        let o = delulu(&[&args[..], &["--report-out", plain.to_str().unwrap()]].concat());
        assert_eq!(o.status.code(), Some(1), "{target}: {}", out(&o));
        let o = delulu(&[&args[..], &["--sandbox", "--report-out", boxed.to_str().unwrap()]].concat());
        assert_eq!(o.status.code(), Some(1), "{target}: {}", out(&o));
        let (p, s) = (read(&plain), read(&boxed));
        let (pd, sd) = (&p["diagnostics"][0], &s["diagnostics"][0]);
        assert_eq!(pd["code"], "DL0703", "{target}: the ordinary run's refusal: {p:#}");
        assert_eq!(sd["code"], pd["code"], "{target}: the host's refusal is in its report: {s:#}");
        assert_eq!(sd["message"], pd["message"], "{target}: {s:#}");
        let (ps, ss) = (&pd["spans"][0], &sd["spans"][0]);
        assert_eq!((&ss["file"], &ss["start"], &ss["end"]), (&ps["file"], &ps["start"], &ps["end"]), "{target}: the same call: {s:#}\n{p:#}");
        assert_eq!(s["diagnostics"].as_array().map(Vec::len), Some(1), "{target}: {s:#}");
        assert_eq!(s["summary"]["errors"], 1, "{target}: {s:#}");
    }
    // A clean sandboxed run's report still carries none.
    let o = delulu(&["run", "docs/book/samples/01_hello.delulu", "--sandbox", "--grant", "console", "--report-out", boxed.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert_eq!(read(&boxed)["diagnostics"], serde_json::json!([]));
    let _ = std::fs::remove_dir_all(&dir);
}
