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
        return (dev != "DEVICE").then(|| format!("actuator={dev}:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold"));
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
    // And every package under `examples/` (routine run 21): `--within` answers for a package directory too.
    let mut packages: Vec<String> = std::fs::read_dir(root.join("examples"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("delulu.toml").is_file())
        .map(|p| format!("examples/{}", p.file_name().unwrap().to_string_lossy()))
        .collect();
    packages.sort();
    assert!(packages.len() >= 3, "the packages under examples/ were read: {packages:?}");
    files.extend(packages);
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
            // The call's own file: the program itself, or a package's module (named relative to the package).
            let named_file = c["at"]["file"].as_str().unwrap_or_else(|| panic!("{file}: no file named: {v:#}"));
            let program = workspace_root().join(&file);
            let src = if program.is_dir() { program.join(named_file) } else { workspace_root().join(named_file) };
            let text = std::fs::read_to_string(&src).unwrap_or_else(|e| panic!("{file}: `{}`: {e}", src.display()));
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
    assert_eq!(
        (v["counterexample"]["grant"].as_str(), v["counterexample"]["at"]["line"].as_u64()),
        (Some("secret:NAME=VALUE"), Some(6)),
        "{v:#}"
    );
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
    assert_eq!(order, [("console", 44), ("fs.read=./config", 48), ("fs.write=./out", 49), ("net=example.com", 50)], "{v:#}");
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
    assert!(
        text.starts_with("exceeds:") && text.contains("fs.write=./out") && text.contains("05_capabilities.delulu:49:"),
        "{text}"
    );
    std::fs::write(&b, "fs.read=./config\nfs.write=./out\nconsole\nnet=example.com\n").unwrap();
    let o = delulu(&["authority", file, "--within-file", b.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(out(&o).starts_with("within:"), "{}", out(&o));

    // Each form that does not answer the question refuses it by name — never ignores it (exit 2).
    let lock = dir.join("old.lock");
    std::fs::write(&lock, "").unwrap();
    // A `.dwx` carries a manifest, not the calls a counterexample names. (A package directory is answered since routine run 21.)
    let dwx = dir.join("shipped.dwx");
    std::fs::write(&dwx, b"\0asm\x01\0\0\0").unwrap();
    for (why, args) in [
        ("--diff", vec!["authority", "--diff", lock.to_str().unwrap(), "examples/line_driver", "--within", "console"]),
        ("--grants", vec!["authority", file, "--grants", "--within", "console"]),
        ("a .dwx", vec!["authority", dwx.to_str().unwrap(), "--within", "console"]),
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
    let old = write(
        "old.grants",
        &["fs.read=./data", "net=example.com", "actuator=arm0/elbow:angle_deg=-30..30,heartbeat_ms=200,ttl_ms=60000,fail=hold"],
    );
    let new = write(
        "new.grants",
        &[
            "fs.read=./data/sub",
            "fs.read=./other",
            "net=example.com",
            "net=api.example.com",
            "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold",
        ],
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
            (
                "device_reach_expansion".to_string(),
                "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold".to_string()
            ),
        ],
        "the narrower `./data/sub` and the unchanged host are no finding; a widened envelope is: {v:#}"
    );
    // The reverse is no narrowing (`./data` is wider than `./data/sub`) — and is named as one scope expansion.
    let (code, v) = diff(&new, &old);
    assert_eq!((code, v["findings"][0]["grant"].as_str()), (1, Some("fs.read=./data")), "{v:#}");
    // A change that only narrows — a subdirectory, the same host, a tighter envelope — adds nothing.
    let narrower = write(
        "narrower.grants",
        &[
            "fs.read=./data/sub",
            "net=example.com",
            "actuator=arm0/elbow:angle_deg=-10..10,heartbeat_ms=200,ttl_ms=60000,fail=hold",
        ],
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
    // P9-04 (c), D-V2-131: a secret's reach is its audience. Another host, another PORT of the same host, or any
    // audience where OLD granted the secret with none (sent nowhere) is `secret_reach_expansion`, naming the origin;
    // the same audience, a smaller one, or none at all adds nothing. Each pair both ways round where it differs.
    let s_a = write("s-a.grants", &["secret:K@a.example.com=v"]);
    for (why, new_line, origin) in [
        ("another host", "secret:K@a.example.com,b.example.com=v", "b.example.com:443"),
        ("another port", "secret:K@a.example.com,a.example.com:8443=v", "a.example.com:8443"),
    ] {
        let f = write("s-wider.grants", &[new_line]);
        let (code, v) = diff(&s_a, &f);
        assert_eq!((code, v["findings"].as_array().unwrap().len()), (1, 1), "{why}: {v:#}");
        assert_eq!(v["findings"][0]["category"], "secret_reach_expansion", "{why}: {v:#}");
        assert_eq!(v["findings"][0]["dimension"], "secret", "{why}: {v:#}");
        assert!(v["findings"][0]["detail"].as_str().unwrap().contains(origin), "{why}: names `{origin}`: {v:#}");
        assert_eq!(diff(&f, &s_a).0, 0, "{why}, the other way round, narrows");
    }
    let s_none = write("s-none.grants", &["secret:K=v"]);
    let (code, v) = diff(&s_none, &s_a);
    assert_eq!(
        (code, v["findings"][0]["category"].as_str()),
        (1, Some("secret_reach_expansion")),
        "an audience where none was: {v:#}"
    );
    assert_eq!(diff(&s_a, &s_none).0, 0, "no audience is the bottom: sent nowhere");
    assert_eq!(diff(&s_a, &s_a).0, 0, "the same audience adds nothing");
    let (code, v) = diff(&s_none, &write("s-other.grants", &["secret:J@a.example.com=v"]));
    assert_eq!(
        (code, v["findings"][0]["category"].as_str()),
        (1, Some("scope_expansion")),
        "another secret is scope, as before: {v:#}"
    );
    // P9-05 (b), D-V2-139: a path is scope. A path under a granted host, or under a granted path, adds nothing; the whole
    // host where OLD granted a path, or a sibling path, is `scope_expansion` naming the entry.
    let p_v1 = write("p-v1.grants", &["net=api.example.com/v1/**"]);
    let p_host = write("p-host.grants", &["net=api.example.com"]);
    assert_eq!(diff(&p_host, &p_v1).0, 0, "a path under a granted host adds nothing");
    assert_eq!(diff(&p_v1, &write("p-items.grants", &["net=api.example.com/v1/items/*"])).0, 0, "a narrower path adds nothing");
    for (why, wider, entry) in [
        ("the whole host", p_host.clone(), "net=api.example.com"),
        ("a sibling path", write("p-v2.grants", &["net=api.example.com/v2/**"]), "net=api.example.com/v2/**"),
    ] {
        let (code, v) = diff(&p_v1, &wider);
        let f = &v["findings"][0];
        assert_eq!(
            (code, f["category"].as_str(), f["grant"].as_str()),
            (1, Some("scope_expansion"), Some(entry)),
            "{why}: {v:#}"
        );
    }
    // The same order for special-use reach: a path of a special-use host is within it; the host itself is not within one
    // of its paths.
    let sp_host = write("sp-host.grants", &["net.special=127.0.0.1"]);
    let sp_path = write("sp-path.grants", &["net.special=127.0.0.1/health"]);
    assert_eq!(diff(&sp_host, &sp_path).0, 0, "a path of a special-use host adds nothing");
    let (code, v) = diff(&sp_path, &sp_host);
    assert_eq!((code, v["findings"][0]["category"].as_str()), (1, Some("special_use_reach")), "the whole host is more: {v:#}");
    // The human answer names the same, and a lease node is refused by name until it is diffed (exit 2).
    let o = delulu(&["grants", "diff", &old, &new]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(out(&o).contains("device_reach_expansion") && out(&o).contains("fs.read=./other"), "{}", out(&o));
    let o = delulu(&["grants", "diff", "g_1", &new]);
    assert!(
        o.status.code() == Some(2) && out(&o).contains("not a file with a node"),
        "a node beside a file is refused: {}",
        out(&o)
    );
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

/// P9-04 (e), D-V2-135: `authority --json` names every place a program SENDS a secret, with the secret and the
/// destination as far as its source shows — a `let`-bound `root.secret("NAME")` and a `Secret.map` of it keep the name, a
/// URL literal gives its origin (`host:port`, the egress client's spelling); a computed URL, and a secret that arrives as a
/// parameter or under a name a `for` rebinds, are `null`, never guessed. A program that sends none has no such key.
#[test]
fn authority_names_where_a_program_sends_each_secret_and_never_guesses() {
    let dir = std::env::temp_dir().join(format!("delulu-sends-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("sends.delulu");
    let program = r#"module sends

fn relay(h: Cap[Http], t: Secret[Str]) -> Bool ! {Net} {
    let hs = Map()
    match h.get_with_secret("https://c.example.com/x", hs, "Authorization", t) {
        Ok(b) => true
        Err(e) => false
    }
}

fn main(root: Root) ! {Net} {
    let h = root.http(["api.example.com", "b.example.com", "c.example.com"])
    let tok = root.secret("API_TOKEN")
    let hs = Map()
    let bearer = tok.map(fn(s: Str) -> Str { "Bearer " + s })
    match h.get_with_secret("https://api.example.com/v1", hs, "Authorization", bearer) {
        Ok(b) => {}
        Err(e) => {}
    }
    let url = "https://b.example.com:8443/" + "x"
    match h.get_with_secret(url, hs, "X-Key", root.secret("OTHER")) {
        Ok(b) => {}
        Err(e) => {}
    }
    for tok in [root.secret("LOOPED")] {
        match h.get_with_secret("https://api.example.com/v3", hs, "X-Loop", tok) {
            Ok(b) => {}
            Err(e) => {}
        }
    }
    if true {
        let tok = root.secret("INNER")
    }
    var k = root.secret("FIRST")
    k = root.secret("SECOND")
    match h.get_with_secret("https://api.example.com/v4", hs, "X-After", tok) {
        Ok(b) => {}
        Err(e) => {}
    }
    match h.get_with_secret("https://api.example.com/v5", hs, "X-K", k) {
        Ok(b) => {}
        Err(e) => {}
    }
    let sent = relay(h, tok)
}
"#;
    std::fs::write(&file, program).unwrap();
    let f = file.to_string_lossy().to_string();
    let o = delulu(&["authority", &f, "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let v = json_of(&o);
    let sends: Vec<(Value, Value, u64)> = v["authority"]["secret_destinations"]
        .as_array()
        .unwrap_or_else(|| panic!("no secret_destinations: {v:#}"))
        .iter()
        .map(|s| (s["secret"].clone(), s["to"].clone(), s["at"]["line"].as_u64().unwrap_or(0)))
        .collect();
    assert_eq!(
        sends,
        vec![
            (Value::Null, serde_json::json!("c.example.com:443"), 5),
            (serde_json::json!("API_TOKEN"), serde_json::json!("api.example.com:443"), 16),
            (serde_json::json!("OTHER"), Value::Null, 21),
            // `for tok in …` rebinds `tok`: the outer `API_TOKEN` is not claimed for the loop's element.
            (Value::Null, serde_json::json!("api.example.com:443"), 26),
            // A block's `let` ends with the block; an assignment rebinds.
            (serde_json::json!("API_TOKEN"), serde_json::json!("api.example.com:443"), 36),
            (serde_json::json!("SECOND"), serde_json::json!("api.example.com:443"), 40),
        ],
        "a parameter's secret, a rebound name and a computed URL are null; a let-bound mint and its map keep the name: {v:#}"
    );
    // The closed schema admits the new key, and only in its shape.
    let report = dir.join("report.json");
    std::fs::write(&report, serde_json::to_string_pretty(&v["authority"]).unwrap()).unwrap();
    let r = json_of(&delulu(&["schema", "validate", "authority", report.to_str().unwrap(), "--json"]));
    assert_eq!(r["validate"]["valid"], true, "{}", r["validate"]["errors"]);
    // A program that sends no secret has no such key — its report is what it was.
    let plain = dir.join("plain.delulu");
    std::fs::write(&plain, "module plain\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let o = delulu(&["authority", &plain.to_string_lossy(), "--json"]);
    let v = json_of(&o);
    assert!(v["authority"].is_object() && v["authority"].get("secret_destinations").is_none(), "{}", out(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

/// P9-04 (d), D-V2-133: a node's secret carries its AUDIENCE — `grants delegate --secret NAME --secret-to NAME@HOST[:PORT]`,
/// the broker's `secret_to` dimension of `NAME@host:port` pairs. `⊑` is inclusion: a child may name fewer origins for a
/// secret, never another one (DL0802, the intersection its repair); `grants diff` between two nodes reports a pair OLD lacks as
/// `secret_reach_expansion`; a secret with no pair is the bottom (sent nowhere). A pair for a secret the delegation does not
/// name grants reach to nothing and is refused in words, and so is a spelling the egress client would refuse as a URL's
/// (a path, a wildcard, an upper-case host — refused, never normalized).
#[test]
fn a_nodes_secret_carries_its_audience_and_a_wider_one_is_secret_reach() {
    let base = std::env::temp_dir().join(format!("dga{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
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
    let delegate = |extra: &[&str]| -> Output {
        let mut args = vec!["grants", "delegate", "--ttl", "1h", "--json", "--effects", "Net", "--net", "api.example.com"];
        args.extend_from_slice(extra);
        run(&args)
    };
    let node = |o: &Output| -> String {
        assert!(o.status.success(), "delegate: {}", out(o));
        json_of(o)["node"].as_str().unwrap().to_string()
    };
    // Canonical on the way in: the default port named (the egress client's spelling, compared at send time).
    let old = node(&delegate(&["--secret", "K", "--secret-to", "K@api.example.com"]));
    let o = run(&["grants", "inspect", &old, "--json"]);
    assert_eq!(json_of(&o)["node"]["secret_to"], serde_json::json!(["K@api.example.com:443"]), "{}", out(&o));
    let new = node(&delegate(&["--secret", "K", "--secret-to", "K@api.example.com", "--secret-to", "K@b.example.com:8443"]));
    let bottom = node(&delegate(&["--secret", "K"]));
    let diff = |a: &str, b: &str| -> (i32, Value) {
        let o = run(&["grants", "diff", a, b, "--json"]);
        (o.status.code().unwrap_or(-1), json_of(&o))
    };
    let (code, v) = diff(&old, &new);
    assert_eq!(code, 1, "{v:#}");
    let f = &v["findings"].as_array().unwrap_or_else(|| panic!("{v:#}"))[..];
    assert_eq!(f.len(), 1, "{v:#}");
    assert_eq!(
        (f[0]["category"].as_str(), f[0]["grant"].as_str()),
        (Some("secret_reach_expansion"), Some("K@b.example.com:8443")),
        "{v:#}"
    );
    assert_eq!(diff(&new, &old).0, 0, "fewer origins add nothing");
    assert_eq!(diff(&old, &bottom).0, 0, "no audience is the bottom: sent nowhere adds nothing");
    let (code, v) = diff(&bottom, &old);
    assert_eq!((code, v["findings"][0]["category"].as_str()), (1, Some("secret_reach_expansion")), "{v:#}");
    // A secret OLD does not hold at all is that secret's `scope_expansion`, counted once — its pairs are not a second finding.
    let other = node(&delegate(&["--secret", "J", "--secret-to", "J@api.example.com"]));
    let (code, v) = diff(&old, &other);
    let cats: Vec<&str> = v["findings"].as_array().unwrap().iter().map(|f| f["category"].as_str().unwrap()).collect();
    assert_eq!((code, cats), (1, vec!["scope_expansion"]), "{v:#}");
    // Under OLD: a pair OLD names is delegated; another origin is refused, the repair the intersection.
    let child = node(&delegate(&["--parent", &old, "--secret", "K", "--secret-to", "K@api.example.com:443"]));
    let o = run(&["grants", "inspect", &child, "--json"]);
    assert_eq!(json_of(&o)["node"]["secret_to"], serde_json::json!(["K@api.example.com:443"]), "{}", out(&o));
    let o = delegate(&["--parent", &old, "--secret", "K", "--secret-to", "K@b.example.com:8443"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(out(&o).contains("DL0802"), "{}", out(&o));
    // Refused in words, before the broker: a pair for a secret not delegated, and spellings no URL could carry.
    for (args, says) in [
        (vec!["--secret", "K", "--secret-to", "J@api.example.com"], "`--secret J`"),
        (vec!["--secret", "K", "--secret-to", "K"], "NAME@HOST"),
        (vec!["--secret", "K", "--secret-to", "K@api.example.com/v1"], "path"),
        (vec!["--secret", "K", "--secret-to", "K@*.example.com"], "wildcard"),
        (vec!["--secret", "K", "--secret-to", "K@API.example.com"], "lower case"),
    ] {
        let o = delegate(&args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", out(&o));
        assert!(out(&o).contains(says), "{args:?} must say {says:?}: {}", out(&o));
    }
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
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
    let old =
        delegate(&["--effects", "Read,Net", "--fs-read", "./data", "--net", "example.com", "--budget", "mem=1048576,cpu=10"]);
    let new = delegate(&[
        "--effects",
        "Read,Write,Net,Actuate",
        "--fs-read",
        "./data/sub",
        "--fs-read",
        "./other",
        "--fs-write",
        "./data",
        "--net",
        "example.com",
        "--net",
        "api.example.com",
        "--device",
        arm,
        "--budget",
        "mem=2097152,cpu=10",
    ]);
    let narrow_arm =
        delegate(&["--effects", "Actuate", "--device", "arm0/elbow:angle_deg=-10..10,heartbeat_ms=200,ttl_ms=60000,fail=hold"]);
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
        [
            "effect_expansion",
            "effect_expansion",
            "scope_expansion",
            "scope_expansion",
            "scope_expansion",
            "device_reach_expansion",
            "budget_expansion"
        ],
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
    for (target, grants) in
        [("examples/guide/05_capabilities.delulu", &["--grant", "console"][..]), ("examples/greeter", &[][..])]
    {
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
        assert_eq!(
            (&ss["file"], &ss["start"], &ss["end"]),
            (&ps["file"], &ps["start"], &ps["end"]),
            "{target}: the same call: {s:#}\n{p:#}"
        );
        assert_eq!(s["diagnostics"].as_array().map(Vec::len), Some(1), "{target}: {s:#}");
        assert_eq!(s["summary"]["errors"], 1, "{target}: {s:#}");
    }
    // A clean sandboxed run's report still carries none.
    let o = delulu(&[
        "run",
        "docs/book/samples/01_hello.delulu",
        "--sandbox",
        "--grant",
        "console",
        "--report-out",
        boxed.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert_eq!(read(&boxed)["diagnostics"], serde_json::json!([]));
    let _ = std::fs::remove_dir_all(&dir);
}

/// P9-03 (`V2_P9_DESIGN.md`): `grants propose REPORT` — the narrowest grant for the call a refused run was refused at, read from
/// the report's refusal (its code and the call's span, D-V2-121/D-V2-122) and the program's own source: that call's static
/// requirement, never a line the runtime or the program worded. The same proposal from an ordinary and a sandboxed run's
/// report; the right one at a SECOND call of a grant (the requirement list keeps each grant's first call only); nothing — said,
/// exit 1 — for a computed scope, a clean run, another code, or a program that changed since the run.
#[test]
fn grants_propose_reads_the_refused_calls_own_requirement_from_the_report() {
    let dir = std::env::temp_dir().join(format!("delulu-grants-propose-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let report = dir.join("report.json");
    let r = report.to_str().unwrap().to_string();
    let propose = |r: &str| -> (i32, Value) {
        let o = delulu(&["grants", "propose", r, "--json"]);
        (o.status.code().unwrap_or(-1), json_of(&o))
    };
    // The guide's refusal, from both kinds of report: the counterexample `authority --within` names.
    let file = "examples/guide/05_capabilities.delulu";
    let (_, v) = within(file, &["console".to_string()]);
    assert_eq!(v["counterexample"]["grant"], "fs.read=./config");
    for sandbox in [false, true] {
        let mut args = vec!["run", file, "--grant", "console", "--report-out", &r];
        if sandbox {
            args.push("--sandbox");
        }
        assert_eq!(delulu(&args).status.code(), Some(1));
        let (code, p) = propose(&r);
        assert_eq!(code, 0, "sandbox={sandbox}: {p:#}");
        assert_eq!(p["proposes"], true, "{p:#}");
        assert_eq!(p["proposal"], serde_json::json!(["fs.read=./config"]), "the call's own requirement, nothing wider: {p:#}");
        assert_eq!(p["refusal"]["code"], "DL0703");
        assert_eq!(
            (p["refusal"]["at"]["line"].as_u64(), p["refusal"]["at"]["column"].as_u64()),
            (v["counterexample"]["at"]["line"].as_u64(), v["counterexample"]["at"]["column"].as_u64()),
            "{p:#}"
        );
        assert_eq!(p["call"], "root.fs_read(\"./config\")", "{p:#}");
    }
    // A refusal at the SECOND call of a grant, beside one at another call of the same line: the first call of `./b` is never
    // run, so the requirement list's `at` for `fs.read=./b` is not the refused call — and the proposal is still that line.
    let twice = dir.join("twice.delulu");
    std::fs::write(
        &twice,
        "module twice\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    let a = root.fs_read(\"./a\")\n    if false {\n        \
         let early = root.fs_read(\"./b\")\n    }\n    let late = root.fs_read(\"./b\")\n    c.println(\"unreached\")\n}\n",
    )
    .unwrap();
    let t = twice.to_str().unwrap();
    let o = delulu(&["run", t, "--grant", "console", "--grant", "fs.read=./a", "--report-out", &r]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let (code, p) = propose(&r);
    assert_eq!((code, &p["proposal"]), (0, &serde_json::json!(["fs.read=./b"])), "{p:#}");
    assert_eq!(p["refusal"]["at"]["line"], 9, "the refused call is the second `./b` one: {p:#}");
    // A computed scope proposes nothing: there is no static narrowest grant for it.
    let computed = dir.join("computed.delulu");
    std::fs::write(
        &computed,
        "module computed\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    let p = \"./c\"\n    let r = root.fs_read(p)\n    c.println(\"unreached\")\n}\n",
    )
    .unwrap();
    let o = delulu(&["run", computed.to_str().unwrap(), "--grant", "console", "--report-out", &r]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let (code, p) = propose(&r);
    assert_eq!((code, &p["proposes"]), (1, &Value::Bool(false)), "{p:#}");
    assert!(p["proposal"].as_array().is_some_and(Vec::is_empty), "{p:#}");
    assert!(p["reason"].as_str().unwrap_or("").contains("computes its scope"), "{p:#}");
    // A program changed since the run: its call is no longer at the report's span.
    let o = delulu(&["run", t, "--grant", "console", "--grant", "fs.read=./a", "--report-out", &r]);
    assert_eq!(o.status.code(), Some(1));
    let src = std::fs::read_to_string(&twice).unwrap();
    std::fs::write(&twice, format!("// edited after the run\n{src}")).unwrap();
    let (code, p) = propose(&r);
    assert_eq!((code, &p["proposes"]), (1, &Value::Bool(false)), "{p:#}");
    assert!(p["reason"].as_str().unwrap_or("").contains("changed since the run"), "{p:#}");
    // The report is input: offsets past the program's end are refused in words, never sliced.
    let mut edited: Value = serde_json::from_str(&std::fs::read_to_string(&r).unwrap()).unwrap();
    edited["diagnostics"][0]["spans"][0]["start"]["byte"] = serde_json::json!(1_000_000);
    edited["diagnostics"][0]["spans"][0]["end"]["byte"] = serde_json::json!(1_000_007);
    std::fs::write(&report, edited.to_string()).unwrap();
    let (code, p) = propose(&r);
    assert_eq!((code, &p["proposes"]), (1, &Value::Bool(false)), "{p:#}");
    assert!(p["reason"].as_str().unwrap_or("").contains("changed since the run"), "{p:#}");
    // A clean run's report records no refusal; a refusal a grant does not answer is not proposed for.
    let o = delulu(&["run", "docs/book/samples/01_hello.delulu", "--grant", "console", "--report-out", &r]);
    assert_eq!(o.status.code(), Some(0));
    let (code, p) = propose(&r);
    assert_eq!((code, &p["proposes"]), (1, &Value::Bool(false)), "{p:#}");
    let mut other: Value = serde_json::from_str(&std::fs::read_to_string(&r).unwrap()).unwrap();
    other["diagnostics"] = serde_json::json!([{ "code": "DL1403", "message": "revoked", "spans": [] }]);
    std::fs::write(&report, other.to_string()).unwrap();
    let (code, p) = propose(&r);
    assert_eq!((code, &p["proposes"]), (1, &Value::Bool(false)), "{p:#}");
    assert!(p["reason"].as_str().unwrap_or("").contains("DL1403"), "{p:#}");
    // Not a run report at all is the caller's error.
    std::fs::write(&report, "{\"command\": \"check\"}").unwrap();
    assert_eq!(delulu(&["grants", "propose", &r, "--json"]).status.code(), Some(2));
    let _ = std::fs::remove_dir_all(&dir);
}

/// P9-03, generated rather than written (§11.2): one program that mints a capability of every kind a run can be refused at
/// without a device (minting is pure — nothing is performed), run with ALL of its own required grants but one. The run is
/// refused at the dropped grant's call, and `grants propose` names exactly the requirement of that call: the dropped line —
/// and, for a call that asks for two hosts at once, both of its lines, which is that call's requirement and nothing wider.
#[test]
fn grants_propose_names_the_dropped_grants_call_for_every_kind() {
    let dir = std::env::temp_dir().join(format!("delulu-grants-propose-kinds-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for d in ["r", "w"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    std::fs::write(
        dir.join("mints.delulu"),
        "module mints\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    let k = root.clock()\n    let n = root.rand()\n    \
         let r = root.fs_read(\"./r\")\n    let w = root.fs_write(\"./w\")\n    let h = root.http([\"a.example\", \"b.example\"])\n    \
         let s = root.secret(\"TOKEN\")\n    c.println(\"every mint granted\")\n}\n",
    )
    .unwrap();
    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu")).current_dir(&dir).args(args).env("DELULU_NO_COLOR", "1").output().unwrap()
    };
    let o = run(&["authority", "mints.delulu", "--json"]);
    let required: Vec<String> =
        json_of(&o)["authority"]["required_grants"].as_array().unwrap().iter().map(|g| g.as_str().unwrap().to_string()).collect();
    // A secret's line carries its value's placeholder; a run is given one.
    let given = |g: &str| g.replace("=VALUE", "=hunter2");
    assert_eq!(required.len(), 8, "{required:?}");
    let all: Vec<String> = required.iter().map(|g| given(g)).collect();
    let mut flags: Vec<&str> = Vec::new();
    for g in &all {
        flags.extend(["--grant", g]);
    }
    let o = run(&[&["run", "mints.delulu"], &flags[..]].concat());
    assert_eq!(o.status.code(), Some(0), "every grant given, the program runs: {}", out(&o));
    let mut proposed = 0;
    for (i, dropped) in required.iter().enumerate() {
        let mut flags: Vec<&str> = Vec::new();
        for (j, g) in all.iter().enumerate() {
            if j != i {
                flags.extend(["--grant", g]);
            }
        }
        let o = run(&[&["run", "mints.delulu", "--report-out", "r.json"], &flags[..]].concat());
        assert_eq!(o.status.code(), Some(1), "{dropped} dropped: {}", out(&o));
        let o = run(&["grants", "propose", "r.json", "--json"]);
        let p = json_of(&o);
        assert_eq!(o.status.code(), Some(0), "{dropped}: {p:#}");
        let lines: Vec<&str> = p["proposal"].as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect();
        let expected: Vec<&str> =
            if dropped.starts_with("net=") { vec!["net=a.example", "net=b.example"] } else { vec![dropped.as_str()] };
        assert_eq!(lines, expected, "{dropped} dropped — the refused call's own requirement: {p:#}");
        proposed += 1;
    }
    assert_eq!(proposed, 8, "every requirement was dropped once");
    let _ = std::fs::remove_dir_all(&dir);
}

/// P9-01 and P9-03 for a PACKAGE (routine run 21): `authority DIR --within` answers for a package directory as for a file —
/// the counterexample's call named in the module's own file — and `grants propose` reads a package run's report, whose call is
/// in the flattened text the run executed (named so by the run's own diagnostic), and proposes the requirement `authority DIR`
/// names for it. Before, both refused a package in words.
#[test]
fn a_package_is_checked_against_a_boundary_and_its_refusal_is_proposed_for() {
    let pkg = "examples/greeter";
    assert_eq!(required(pkg), ["console"]);
    let (code, v) = within(pkg, &["console".to_string()]);
    assert_eq!((code, v["result"].as_str()), (0, Some("within")), "{v:#}");
    let (code, v) = within(pkg, &[]);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    let at = &v["counterexample"]["at"];
    // Named as `authority DIR` names a module — relative to the package — and never the flattened text.
    let file = at["file"].as_str().unwrap_or_else(|| panic!("the call is named: {v:#}"));
    assert!(!file.contains("flattened") && file.ends_with(".delulu"), "the module's own file: {v:#}");
    let text = std::fs::read_to_string(workspace_root().join(pkg).join(file)).unwrap();
    let line = text.lines().nth(at["line"].as_u64().unwrap() as usize - 1).unwrap();
    assert!(line.contains("console("), "the counterexample points at its call: `{line}`");

    let dir = std::env::temp_dir().join(format!("delulu-package-propose-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let r = dir.join("report.json").to_string_lossy().to_string();
    for sandbox in [false, true] {
        let mut args = vec!["run", pkg, "--report-out", &r];
        if sandbox {
            args.push("--sandbox");
        }
        let o = delulu(&args);
        assert_eq!(o.status.code(), Some(1), "sandbox={sandbox}: {}", out(&o));
        let report: Value = serde_json::from_str(&std::fs::read_to_string(&r).unwrap()).unwrap();
        let o = delulu(&["grants", "propose", &r, "--json"]);
        let p = json_of(&o);
        assert_eq!(o.status.code(), Some(0), "sandbox={sandbox}: {p:#}");
        assert_eq!(p["proposal"], serde_json::json!(["console"]), "the call's own requirement: {p:#}");
        assert_eq!(p["call"], "root.console()", "{p:#}");
        let span = &report["diagnostics"][0]["spans"][0];
        assert_eq!(p["program"], span["file"], "the flattened text the run named: {p:#}");
        assert_eq!(
            (p["refusal"]["at"]["line"].as_u64(), p["refusal"]["at"]["column"].as_u64()),
            (span["start"]["line"].as_u64(), span["start"]["col"].as_u64()),
            "{p:#}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// P9-03 step 3 (`V2_P9_DESIGN.md` item 3, D-V2-125): the store and its approval. A lease run refused at a call files its
/// proposal with the broker against the node it ran under; the proposal is a REQUEST — nothing is granted by filing it. An
/// approval is a delegation under the APPROVER's own node of the proposer's authority joined with what it asked: the
/// proposer's own node cannot approve its widening (DL0802 — a program cannot relax its own authority), an approver cannot
/// grant past its own node (DL0802 again), the proposer's node is unchanged, and the program is RE-RUN under the new node,
/// where it passes the call it was refused at. A rejection's reason reaches the proposer in `--json`; a proposal from a
/// revoked node is neither filed nor, once filed, approved — a quarantined run's ask cannot bring its authority back.
#[test]
fn a_proposal_is_filed_approved_under_the_approvers_node_and_rejected_with_a_reason_the_agent_reads() {
    // Short: a broker's socket path is bounded (104 bytes on macOS).
    let base = std::env::temp_dir().join(format!("dpp{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::copy(workspace_root().join("examples/guide/05_capabilities.delulu"), cwd.join("caps.delulu")).unwrap();
    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(&cwd)
            .env("DELULU_STATE_DIR", &state)
            .env("DELULU_NO_COLOR", "1")
            .env_remove("DELULU_GUARD_OWNER")
            .args(args)
            .output()
            .expect("failed to run delulu")
    };
    let o = run(&["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", out(&o));
    let guard = BrokerGuard(state.clone());
    // The operator's node R (no `./out` write), and the agent's lease under it: the console only.
    let o = run(&[
        "grants",
        "delegate",
        "--effects",
        "Read,Write,Net",
        "--fs-read",
        "./config",
        "--net",
        "example.com",
        "--ttl",
        "1h",
        "--json",
    ]);
    assert!(o.status.success(), "{}", out(&o));
    let root = json_of(&o)["parent"].as_str().unwrap().to_string();
    let o = run(&["grants", "delegate", "--parent", &root, "--effects", "Write", "--ttl", "1h", "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let (agent, token) = {
        let v = json_of(&o);
        (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string())
    };
    let inspect = |node: &str| -> Value {
        let o = run(&["grants", "inspect", node, "--json"]);
        assert!(o.status.success(), "{}", out(&o));
        json_of(&o)["node"].clone()
    };
    let proposals = || -> Vec<Value> {
        let o = run(&["grants", "proposals", "--json"]);
        assert!(o.status.success(), "{}", out(&o));
        json_of(&o)["proposals"].as_array().cloned().unwrap_or_default()
    };
    let refused_code = |o: &Output| -> String { json_of(o)["diagnostics"][0]["code"].as_str().unwrap_or("").to_string() };
    // The agent's run is refused at the config read; its proposal is that call's own requirement, filed against its node.
    let o = run(&["run", "caps.delulu", "--lease", &token, "--report-out", "r1.json"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    // Read after the run: its redemption bound the node to the run's process (`holder_peer`) — the run's doing, not a proposal's.
    let before = inspect(&agent);
    let o = run(&["grants", "propose", "r1.json", "--node", &agent, "--json"]);
    let p = json_of(&o);
    assert_eq!(o.status.code(), Some(0), "{p:#}");
    assert_eq!(p["proposal"], serde_json::json!(["fs.read=./config"]), "{p:#}");
    let id = p["filed"]["id"].as_str().unwrap_or_else(|| panic!("filed with the broker: {p:#}")).to_string();
    assert_eq!(p["filed"]["node"], agent.as_str(), "{p:#}");
    // Filing it again while it is pending answers the same proposal.
    let o = run(&["grants", "propose", "r1.json", "--node", &agent, "--json"]);
    assert_eq!((json_of(&o)["filed"]["id"].as_str(), json_of(&o)["filed"]["deduped"].as_bool()), (Some(id.as_str()), Some(true)));
    let listed = proposals();
    assert_eq!(listed.len(), 1, "{listed:?}");
    let entry = &listed[0];
    assert_eq!(
        (entry["id"].as_str(), entry["node"].as_str(), entry["status"].as_str()),
        (Some(id.as_str()), Some(agent.as_str()), Some("pending"))
    );
    assert_eq!(entry["asks"]["effects"], serde_json::json!(["Read"]), "it asks for the read and nothing else: {entry:#}");
    assert!(
        entry["asks"]["fs_read"].as_array().is_some_and(|a| a.len() == 1 && a[0].as_str().unwrap().ends_with("config")),
        "{entry:#}"
    );
    assert_eq!(entry["call"], "root.fs_read(\"./config\")", "the proposer's account, kept as data: {entry:#}");
    assert_eq!(
        entry["mints"]["effects"],
        serde_json::json!(["Read", "Write"]),
        "the approver reads the whole new node: {entry:#}"
    );
    // Filing granted nothing: the agent's node is as it was, and the same lease is refused at the same call.
    assert_eq!(inspect(&agent), before, "filing a proposal changes no node");
    // The proposer cannot approve its own widening: under its own node the approval is not ⊑ it.
    let o = run(&["grants", "approve", &id, "--parent", &agent, "--json"]);
    assert_eq!((o.status.code(), refused_code(&o).as_str()), (Some(1), "DL0802"), "{}", out(&o));
    assert_eq!(proposals()[0]["status"], "pending", "a refused approval leaves it pending");
    // The operator approves under R: a NEW node holding the agent's authority and the read, ⊑ R.
    let o = run(&["grants", "approve", &id, "--parent", &root, "--json"]);
    let a = json_of(&o);
    assert_eq!(o.status.code(), Some(0), "{a:#}");
    let (approved, token2) = (a["node"].as_str().unwrap().to_string(), a["token"].as_str().unwrap().to_string());
    assert_ne!(approved, agent);
    let n = inspect(&approved);
    assert_eq!((n["parent"].as_str(), &n["effects"]), (Some(root.as_str()), &serde_json::json!(["Read", "Write"])), "{n:#}");
    assert_eq!(inspect(&agent), before, "the proposer's node is never widened: the program is re-run");
    let listed = proposals();
    assert_eq!(
        (listed[0]["status"].as_str(), listed[0]["approved"]["node"].as_str()),
        (Some("approved"), Some(approved.as_str())),
        "{listed:?}"
    );
    assert_eq!(listed[0]["mints"], Value::Null, "a decided proposal mints nothing more");
    // A decided proposal is not decided again.
    let o = run(&["grants", "approve", &id, "--parent", &root, "--json"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    // Re-run under the new node: past the read it was refused at, to the next call it lacks — the `./out` write.
    let o = run(&["run", "caps.delulu", "--lease", &token2, "--report-out", "r2.json"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let o = run(&["grants", "propose", "r2.json", "--node", &approved, "--json"]);
    let p = json_of(&o);
    assert_eq!(p["proposal"], serde_json::json!(["fs.write=./out"]), "the read passed; the write is next: {p:#}");
    let id2 = p["filed"]["id"].as_str().unwrap().to_string();
    // R holds no write: an approver cannot grant past its own node.
    let o = run(&["grants", "approve", &id2, "--parent", &root, "--json"]);
    assert_eq!((o.status.code(), refused_code(&o).as_str()), (Some(1), "DL0802"), "{}", out(&o));
    // Rejected, with a reason the agent reads back.
    let o = run(&["grants", "reject", &id2, "--reason", "writes stay with the operator", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let rejected = proposals().into_iter().find(|p| p["id"] == id2.as_str()).unwrap();
    assert_eq!(
        (rejected["status"].as_str(), rejected["reason"].as_str()),
        (Some("rejected"), Some("writes stay with the operator")),
        "{rejected:#}"
    );
    let o = run(&["grants", "reject", &id2, "--reason", "again", "--json"]);
    assert_eq!(o.status.code(), Some(1), "a decided proposal is not rejected again: {}", out(&o));
    // A proposal filed from a node that is revoked before its approval mints nothing; one from a revoked node is not filed.
    let o = run(&["grants", "propose", "r2.json", "--node", &approved, "--json"]);
    let id3 = json_of(&o)["filed"]["id"].as_str().unwrap().to_string();
    assert_ne!(id3, id2, "a rejected proposal is not the pending one");
    let o = run(&["grants", "revoke", &approved, "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let count = || json_of(&run(&["grants", "list", "--json"]))["count"].as_u64();
    let nodes = count();
    let o = run(&["grants", "approve", &id3, "--parent", &root, "--json"]);
    assert_eq!((o.status.code(), refused_code(&o).as_str()), (Some(1), "DL1403"), "{}", out(&o));
    assert_eq!(count(), nodes, "nothing minted for a revoked node's proposal");
    assert_eq!(proposals().into_iter().find(|p| p["id"] == id3.as_str()).unwrap()["status"], "pending");
    let o = run(&["grants", "propose", "r2.json", "--node", &approved, "--json"]);
    assert_eq!((o.status.code(), refused_code(&o).as_str()), (Some(1), "DL1403"), "{}", out(&o));
    // A grant a lease node cannot carry — a sensor — is proposed but NOT filed: an approval could not hold it.
    std::fs::write(
        cwd.join("sensor.delulu"),
        "module sensor\n\nfn main(root: Root) ! {Write, Read} {\n    let c = root.console()\n    let t = root.sensor(\"bay0/temp\")\n    \
         c.println(\"unreached\")\n}\n",
    )
    .unwrap();
    let o = run(&["run", "sensor.delulu", "--grant", "console", "--report-out", "r3.json"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let filed = proposals().len();
    let o = run(&["grants", "propose", "r3.json", "--node", &agent, "--json"]);
    let p = json_of(&o);
    assert_eq!(
        (o.status.code(), &p["proposal"], &p["filed"]),
        (Some(1), &serde_json::json!(["sensor=bay0/temp"]), &Value::Null),
        "{p:#}"
    );
    assert!(p["not_filed"].as_str().unwrap_or("").contains("no form in a lease node"), "{p:#}");
    assert_eq!(proposals().len(), filed, "nothing filed");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-03 step 4 (`V2_P9_DESIGN.md` item 4, D-V2-126): auto-approval is the operator's own STANDING DELEGATION — a node in the
/// tree. `grants auto-approve --parent B` approves, under B, every pending proposal whose mint (the proposer's authority joined
/// with its ask) is within B — the tree's own `⊑` decides — and leaves every other one pending with what `grants diff` finds
/// it adds beyond B. It never carries the Guard's owner code — not as a flag, not from the environment — so guarded authority
/// is approved by a person, one proposal at a time. And because the boundary is a node, revoking it revokes everything it
/// approved.
#[test]
fn auto_approval_approves_under_the_standing_node_what_it_bounds_and_never_guarded_authority() {
    let base = std::env::temp_dir().join(format!("dpa{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::copy(workspace_root().join("examples/guide/05_capabilities.delulu"), cwd.join("caps.delulu")).unwrap();
    let go = |args: &[&str], owner_env: Option<&str>| -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
        c.current_dir(&cwd).env("DELULU_STATE_DIR", &state).env("DELULU_NO_COLOR", "1").env_remove("DELULU_GUARD_OWNER");
        if let Some(code) = owner_env {
            c.env("DELULU_GUARD_OWNER", code);
        }
        c.args(args).output().expect("failed to run delulu")
    };
    let run = |args: &[&str]| go(args, None);
    let o = run(&["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", out(&o));
    let guard = BrokerGuard(state.clone());
    let owner = {
        let text = out(&o);
        let tail = &text[text.find("gow1_").expect("broker start prints the owner code")..];
        tail[..tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len())].to_string()
    };
    let delegate = |extra: &[&str]| -> Value {
        let mut args = vec!["grants", "delegate", "--ttl", "1h", "--json", "--owner", &owner];
        args.extend_from_slice(extra);
        let o = run(&args);
        assert!(o.status.success(), "delegate {extra:?}: {}", out(&o));
        json_of(&o)
    };
    // The operator's root R, and the standing boundary B under it: the read of `./config` with the console, and `Declassify S`.
    let root = delegate(&[
        "--effects",
        "Read,Write,Net,Declassify",
        "--fs-read",
        "./config",
        "--net",
        "example.com",
        "--declassify",
        "S",
    ])["parent"]
        .as_str()
        .unwrap()
        .to_string();
    let b = delegate(&["--parent", &root, "--effects", "Read,Write,Declassify", "--fs-read", "./config", "--declassify", "S"])
        ["node"]
        .as_str()
        .unwrap()
        .to_string();
    // Three agents under R, each refused at the `./config` read; one report, filed for each node.
    let a1 = delegate(&["--parent", &root, "--effects", "Write"]);
    let a2 =
        delegate(&["--parent", &root, "--effects", "Write,Net", "--net", "example.com"])["node"].as_str().unwrap().to_string();
    let a3 = delegate(&["--parent", &root, "--effects", "Write,Declassify", "--declassify", "S"])["node"]
        .as_str()
        .unwrap()
        .to_string();
    let o = run(&["run", "caps.delulu", "--lease", a1["token"].as_str().unwrap(), "--report-out", "r.json"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let file = |node: &str| -> String {
        let o = run(&["grants", "propose", "r.json", "--node", node, "--json"]);
        assert_eq!(o.status.code(), Some(0), "{}", out(&o));
        json_of(&o)["filed"]["id"].as_str().unwrap().to_string()
    };
    let (p1, p2, p3) = (file(a1["node"].as_str().unwrap()), file(&a2), file(&a3));
    // The owner code is refused as a flag: guarded authority is approved by a person, one proposal at a time.
    let o = run(&["grants", "auto-approve", "--parent", &b, "--owner", &owner, "--json"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("approved by a person, one proposal at a time"), "the refusal says why: {}", out(&o));
    // ...and not taken from the environment either.
    let o = go(&["grants", "auto-approve", "--parent", &b, "--json"], Some(&owner));
    let v = json_of(&o);
    assert_eq!(o.status.code(), Some(1), "a proposal left pending is exit 1: {v:#}");
    let approved = v["approved"].as_array().unwrap_or_else(|| panic!("{v:#}"));
    assert_eq!(approved.len(), 1, "{v:#}");
    assert_eq!(approved[0]["proposal"], p1.as_str(), "{v:#}");
    assert!(approved[0]["token"].is_string(), "{v:#}");
    let n1 = approved[0]["node"].as_str().unwrap().to_string();
    let pending = v["pending"].as_array().unwrap();
    let left =
        |id: &str| pending.iter().find(|p| p["proposal"] == id).cloned().unwrap_or_else(|| panic!("{id} left pending: {v:#}"));
    let (l2, l3) = (left(&p2), left(&p3));
    assert_eq!(l2["code"], "DL0802", "beyond B: {l2:#}");
    let found: Vec<(String, String)> = l2["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["category"].as_str().unwrap().to_string(), f["grant"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(
        found,
        [("effect_expansion".to_string(), "Net".to_string()), ("scope_expansion".to_string(), "net=example.com".to_string())],
        "what it adds beyond B, by category: {l2:#}"
    );
    assert_eq!(l3["code"], "DL1410", "within B, but guarded — the Guard's, not the boundary's: {l3:#}");
    assert_eq!(l3["findings"], serde_json::json!([]), "{l3:#}");
    let n = json_of(&run(&["grants", "inspect", &n1, "--json"]))["node"].clone();
    assert_eq!(
        (n["parent"].as_str(), &n["effects"]),
        (Some(b.as_str()), &serde_json::json!(["Read", "Write"])),
        "under B: {n:#}"
    );
    // A second pass approves nothing more, and says the same of what is left.
    let o = run(&["grants", "auto-approve", "--parent", &b, "--json"]);
    let v = json_of(&o);
    assert_eq!(
        (o.status.code(), v["approved"].as_array().map(Vec::len), v["pending"].as_array().map(Vec::len)),
        (Some(1), Some(0), Some(2)),
        "{v:#}"
    );
    // With nothing pending, there is nothing to leave: exit 0.
    for id in [&p2, &p3] {
        assert!(run(&["grants", "reject", id, "--reason", "not automatically"]).status.success());
    }
    let o = run(&["grants", "auto-approve", "--parent", &b, "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    // The standing delegation is one node: revoking it revokes what it approved.
    assert!(run(&["grants", "revoke", &b]).status.success());
    let n = json_of(&run(&["grants", "inspect", &n1, "--json"]))["node"].clone();
    assert_eq!(n["state"], "revoked", "{n:#}");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-05 (c), D-V2-140's static half: `authority --json` names every request a program makes — each `get`, `get_with` and
/// `get_with_secret` whose receiver the checker typed `Cap[Http]` — with its URL as far as the source shows: a literal's
/// host and path by the egress client's own parse; a computed URL, or one that client refuses (`http://`), `null` beside
/// the literal it may have. Decided by the checker's type at each call, never by a method's name: a `List`'s `get` is no
/// request — what D-V2-140 kept this open for. A program that makes none has no such key.
#[test]
fn authority_names_each_request_by_the_checkers_type_and_never_by_a_methods_name() {
    let dir = std::env::temp_dir().join(format!("delulu-requests-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let program = "module r\nfn fetch(c: Cap[Http], u: Str) -> Result[Str, NetErr] ! {Net} {\n  c.get(u)\n}\n\
                   fn main(root: Root) ! {Net, Write} {\n  let h = root.http([\"example.com\"])\n  let xs = [1, 2, 3]\n  \
                   let a = xs.get(0)\n  let r1 = h.get(\"https://example.com/v1/health\")\n  let r2 = fetch(h, \"https://example.com/x\")\n  \
                   let r3 = h.get(\"http://example.com/plain\")\n  root.console().println(\"done\")\n}\n";
    let file = dir.join("r.delulu");
    std::fs::write(&file, program).unwrap();
    let o = delulu(&["authority", &file.to_string_lossy(), "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let v = json_of(&o);
    let requests: Vec<(Value, Value, Value, u64)> = v["authority"]["requests"]
        .as_array()
        .unwrap_or_else(|| panic!("no requests: {v:#}"))
        .iter()
        .map(|r| (r["url"].clone(), r["host"].clone(), r["path"].clone(), r["at"]["line"].as_u64().unwrap_or(0)))
        .collect();
    let j = |s: &str| serde_json::json!(s);
    assert_eq!(
        requests,
        vec![
            // The parameter's URL: computed, as far as this call shows.
            (Value::Null, Value::Null, Value::Null, 3),
            (j("https://example.com/v1/health"), j("example.com"), j("/v1/health"), 9),
            // Not `https://`: the effect refuses it, and the egress client's parse names no host.
            (j("http://example.com/plain"), Value::Null, Value::Null, 11),
        ],
        "the list's `get` on line 8 is no request: {v:#}"
    );
    let report = dir.join("report.json");
    std::fs::write(&report, serde_json::to_string_pretty(&v["authority"]).unwrap()).unwrap();
    let r = json_of(&delulu(&["schema", "validate", "authority", report.to_str().unwrap(), "--json"]));
    assert_eq!(r["validate"]["valid"], true, "{}", r["validate"]["errors"]);
    let plain = dir.join("plain.delulu");
    std::fs::write(&plain, "module plain\n\nfn main(root: Root) ! {Write} {\n    let xs = [1]\n    let a = xs.get(0)\n    root.console().println(\"hi\")\n}\n").unwrap();
    let v = json_of(&delulu(&["authority", &plain.to_string_lossy(), "--json"]));
    assert!(v["authority"].is_object() && v["authority"].get("requests").is_none(), "{v:#}");

    // A package: each module's requests by that module's own node types (node ids are per module), in its own file.
    let pkg = dir.join("pkg");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n[authority]\neffects = [\"Net\", \"Write\"]\nnet = [\"example.com\"]\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src/main.delulu"),
        "module pkg\n\nimport pkg.net\n\nfn main(root: Root) ! {Net, Write} {\n    let xs = [1]\n    let a = xs.get(0)\n    \
         let r = ping(root.http([\"example.com\"]))\n    root.console().println(\"done\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src/net.delulu"),
        "module pkg.net\n\npub fn ping(c: Cap[Http]) -> Result[Str, NetErr] ! {Net} {\n    c.get(\"https://example.com/ping\")\n}\n",
    )
    .unwrap();
    let o = delulu(&["authority", &pkg.to_string_lossy(), "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let v = json_of(&o);
    let requests = v["authority"]["requests"].as_array().unwrap_or_else(|| panic!("no requests: {v:#}"));
    assert_eq!(requests.len(), 1, "the package's one request — the list's `get` in main is none: {v:#}");
    assert_eq!((&requests[0]["host"], &requests[0]["path"]), (&serde_json::json!("example.com"), &serde_json::json!("/ping")));
    assert!(requests[0]["at"]["file"].as_str().is_some_and(|f| f.ends_with("net.delulu")), "{v:#}");

    // And through a dependency: the program checked with its dependencies (`deps.rs`) keeps each module's types too.
    let (app, lib) = (dir.join("app"), dir.join("netlib"));
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(lib.join("src")).unwrap();
    std::fs::write(
        lib.join("delulu.toml"),
        "[package]\nname = \"netlib\"\nversion = \"0.1.0\"\nkind = \"lib\"\n\n[authority]\neffects = [\"Net\"]\nnet = [\"example.com\"]\n",
    )
    .unwrap();
    std::fs::write(
        lib.join("src/netlib.delulu"),
        "module netlib\n\npub fn ping(c: Cap[Http]) -> Result[Str, NetErr] ! {Net} {\n    c.get(\"https://example.com/dep\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        app.join("delulu.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n[authority]\neffects = [\"Net\", \"Write\"]\nnet = [\"example.com\"]\n\n\
         [dependencies]\nnetlib = { path = \"../netlib\", authority = { effects = [\"Net\"], net = [\"example.com\"] } }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("src/app.delulu"),
        "module app\n\nimport netlib\n\nfn main(root: Root) ! {Net, Write} {\n    let r = ping(root.http([\"example.com\"]))\n    \
         root.console().println(\"done\")\n}\n",
    )
    .unwrap();
    let o = delulu(&["authority", &app.to_string_lossy(), "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let v = json_of(&o);
    let requests = v["authority"]["requests"].as_array().unwrap_or_else(|| panic!("no requests through the dependency: {v:#}"));
    assert_eq!(requests.len(), 1, "{v:#}");
    assert_eq!(requests[0]["path"], serde_json::json!("/dep"), "{v:#}");

    // And a loose directory — modules with no manifest, checked by `check_program`.
    let loose = dir.join("loose");
    std::fs::create_dir_all(loose.join("src")).unwrap();
    std::fs::copy(pkg.join("src/main.delulu"), loose.join("src/main.delulu")).unwrap();
    std::fs::copy(pkg.join("src/net.delulu"), loose.join("src/net.delulu")).unwrap();
    let o = delulu(&["authority", &loose.to_string_lossy(), "--json"]);
    assert!(o.status.success(), "{}", out(&o));
    let v = json_of(&o);
    let requests = v["authority"]["requests"].as_array().unwrap_or_else(|| panic!("no requests in the loose directory: {v:#}"));
    assert_eq!((requests.len(), &requests[0]["path"]), (1, &serde_json::json!("/ping")), "{v:#}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// RW 2.11 (D-V2-180): a secret the program sends is a request to send it to that origin, and a run refuses the send unless
/// the grant's audience names it (P9-04 (c)). `--grants` asked for `secret:API_TOKEN=VALUE`, which lets the program hold
/// the secret and refuses every send; `--within` answered *within* for a boundary granting the secret with no audience, or
/// with one naming another origin. Now the line names the origin, a computed URL's as `HOST` and a secret the source does
/// not name as `NAME`, and each send is decided against the boundary's audience for its secret.
#[test]
fn a_sent_secret_is_asked_for_and_decided_with_its_audience() {
    let dir = std::env::temp_dir().join(format!("delulu-within-audience-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let program = |name: &str, url: &str| {
        let f = dir.join(format!("{name}.delulu"));
        std::fs::write(
            &f,
            format!(
                "module {name}\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    \
                 let h = root.http([\"api.example.com\"])\n    let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
                 match h.get_with_secret({url}, hs, \"Authorization\", tok) {{\n        \
                 Ok(body) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
            ),
        )
        .unwrap();
        f.to_string_lossy().to_string()
    };
    let grants = |extra: &str| vec!["console".to_string(), "net=api.example.com".to_string(), extra.to_string()];

    let sent = program("sent", "\"https://api.example.com:8443/v1/items\"");
    assert_eq!(required(&sent), ["console", "net=api.example.com", "secret:API_TOKEN@api.example.com:8443=VALUE"]);
    for (secret, result, reason) in [
        ("secret:API_TOKEN@api.example.com:8443=v", "within", ""),
        ("secret:API_TOKEN@other.example.com,api.example.com:8443=v", "within", ""),
        ("secret:API_TOKEN=v", "exceeds", "with no audience, so its send to `api.example.com:8443` is refused"),
        ("secret:API_TOKEN@other.example.com=v", "exceeds", "does not name `api.example.com:8443`"),
        ("secret:API_TOKEN@api.example.com=v", "exceeds", "(`api.example.com:443`) does not name `api.example.com:8443`"),
    ] {
        let (code, v) = within(&sent, &grants(secret));
        assert_eq!((code, v["result"].as_str()), (i32::from(result != "within"), Some(result)), "{secret}: {v:#}");
        if result != "within" {
            assert_eq!(v["counterexample"]["grant"], "secret:API_TOKEN@api.example.com:8443=VALUE", "{v:#}");
            assert!(v["counterexample"]["reason"].as_str().unwrap().contains(reason), "{secret}: {v:#}");
        }
    }

    // A computed URL: the audience is the operator's to choose, and no audience decides the send until it runs.
    let computed = program("computed", "\"https://api.example.com\" + \"/v1/items\"");
    assert!(required(&computed).contains(&"secret:API_TOKEN@HOST=VALUE".to_string()), "{:?}", required(&computed));
    let (code, v) = within(&computed, &grants("secret:API_TOKEN@api.example.com=v"));
    assert_eq!((code, v["result"].as_str()), (1, Some("inconclusive")), "{v:#}");
    let (code, v) = within(&computed, &grants("secret:API_TOKEN=v"));
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "no audience refuses any send: {v:#}");

    // A secret the source does not name — a parameter of the function that sends it: every secret the boundary grants
    // could be the one sent.
    let helper = dir.join("helper.delulu");
    std::fs::write(
        &helper,
        "module helper\n\nfn send(h: Cap[Http], s: Secret[Str]) -> Bool ! {Net} {\n    \
         match h.get_with_secret(\"https://api.example.com/x\", Map(), \"Authorization\", s) {\n        \
         Ok(body) => true\n        Err(e) => false\n    }\n}\n\nfn main(root: Root) ! {Net} {\n    \
         let ok = send(root.http([\"api.example.com\"]), root.secret(\"A\"))\n}\n",
    )
    .unwrap();
    let helper = helper.to_string_lossy().to_string();
    assert_eq!(required(&helper), ["net=api.example.com", "secret:A=VALUE", "secret:NAME@api.example.com:443=VALUE"]);
    let net = "net=api.example.com".to_string();
    for (boundary, result) in [
        (vec![net.clone(), "secret:A@api.example.com=v".to_string()], "within"),
        (vec![net.clone(), "secret:A@api.example.com=v".to_string(), "secret:B=v".to_string()], "inconclusive"),
        (vec![net.clone(), "secret:A=v".to_string()], "exceeds"),
    ] {
        let (code, v) = within(&helper, &boundary);
        assert_eq!((code, v["result"].as_str()), (i32::from(result != "within"), Some(result)), "{boundary:?}: {v:#}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// D-V2-181: a request is judged at its call by its path where its capability's entry names a pattern (P9-05 (a)), and
/// `--within` decided only the mint: a program minting `example.com/api/**` and requesting `https://example.com/admin`
/// was *within* a boundary of `net=example.com/api/**`, though the run refuses the request whatever it minted. Each request
/// is now decided against the boundary's `net` entries as the call decides it (`prim::host_of`, `path_of`,
/// `netscope::judge_all`), named by the narrowest line that takes it, at its call; a computed URL is *inconclusive* only
/// where the boundary scopes a host by path. `--grants` is unchanged: the line to type is the mint's. A file, a package
/// whose request is in another module, and `deploy plan` (`deploy_cli`) decide alike.
#[test]
fn a_request_is_decided_by_its_path_against_the_boundary() {
    let dir = std::env::temp_dir().join(format!("delulu-within-request-path-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let program = |name: &str, url: &str| {
        let f = dir.join(format!("{name}.delulu"));
        std::fs::write(
            &f,
            format!(
                "module {name}\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    \
                 let h = root.http([\"example.com/api/**\"])\n    match h.get({url}) {{\n        \
                 Ok(body) => out.println(\"got\")\n        Err(e) => out.println(\"no\")\n    }}\n}}\n"
            ),
        )
        .unwrap();
        f.to_string_lossy().to_string()
    };
    let scoped = vec!["console".to_string(), "net=example.com/api/**".to_string()];
    let whole = vec!["console".to_string(), "net=example.com".to_string()];

    let outside = program("outside", "\"https://example.com/admin?x=1\"");
    assert_eq!(required(&outside), ["console", "net=example.com/api/**"], "a request is decided, not typed");
    let (code, v) = within(&outside, &scoped);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    assert_eq!(
        (v["counterexample"]["grant"].as_str(), v["counterexample"]["at"]["line"].as_u64()),
        (Some("net=example.com/admin"), Some(6)),
        "named by the narrowest line, at the request's call: {v:#}"
    );
    assert!(v["counterexample"]["reason"].as_str().unwrap().contains("the path `/admin`"), "{v:#}");
    assert_eq!(within(&outside, &whole).1["result"], "within", "a whole host takes every path");

    let inside = program("inside", "\"https://example.com/api/v1/items\"");
    assert_eq!(within(&inside, &scoped).1["result"], "within");

    let computed = program("computed", "\"https://example.com/\" + \"api\"");
    let (code, v) = within(&computed, &scoped);
    assert_eq!((code, v["result"].as_str()), (1, Some("inconclusive")), "{v:#}");
    assert_eq!(within(&computed, &whole).1["result"], "within", "a computed URL under whole hosts: the mint decides");

    // A URL the egress client will not read is refused at its call whatever is granted, and said so in its words.
    let plain = program("plain", "\"http://example.com/api/x\"");
    let (code, v) = within(&plain, &whole);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    assert!(v["counterexample"]["reason"].as_str().unwrap().contains("only `https://` is fetched"), "{v:#}");

    // A package: the request is made in another module than the mint, by the checker's types there.
    let pkg = dir.join("pkg");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n[authority]\neffects = [\"Net\", \"Write\"]\nnet = [\"example.com\"]\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src/main.delulu"),
        "module pkg\n\nimport pkg.net\n\nfn main(root: Root) ! {Net, Write} {\n    let r = ping(root.http([\"example.com/api/**\"]))\n    \
         root.console().println(\"done\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src/net.delulu"),
        "module pkg.net\n\npub fn ping(c: Cap[Http]) -> Result[Str, NetErr] ! {Net} {\n    c.get(\"https://example.com/admin\")\n}\n",
    )
    .unwrap();
    let (code, v) = within(&pkg.to_string_lossy(), &scoped);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "{v:#}");
    assert_eq!(v["counterexample"]["grant"], "net=example.com/admin", "{v:#}");
    assert!(v["counterexample"]["at"]["file"].as_str().is_some_and(|f| f.ends_with("net.delulu")), "{v:#}");
    assert_eq!(within(&pkg.to_string_lossy(), &whole).1["result"], "within");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A URL written once and named (`let u = "https://…"` in a body, or a module's constant, `let U: Str = "https://…"` among
/// its items) is the literal it was bound to, as the run reads it. The walk read a URL only where the call wrote it out, so
/// such a request under a path-scoped boundary was *inconclusive*, even when it named a path outside the boundary, and a
/// sent secret's audience was proposed as `HOST`. A name that may hold another value at the call is still computed: one
/// assigned in a branch, a parameter or a local `let` shadowing the constant.
#[test]
fn a_url_named_by_a_let_or_a_constant_is_decided_by_its_literal() {
    let dir = std::env::temp_dir().join(format!("delulu-within-url-let-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let program = |name: &str, top: &str, prelude: &str, url: &str| {
        let f = dir.join(format!("{name}.delulu"));
        std::fs::write(
            &f,
            format!(
                "module {name}\n{top}\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    \
                 let h = root.http([\"example.com/api/**\"])\n{prelude}    match h.get({url}) {{\n        \
                 Ok(body) => out.println(\"got\")\n        Err(e) => out.println(\"no\")\n    }}\n}}\n"
            ),
        )
        .unwrap();
        f.to_string_lossy().to_string()
    };
    let scoped = vec!["console".to_string(), "net=example.com/api/**".to_string()];
    let decided = |f: &str, want: (i32, &str)| {
        let (code, v) = within(f, &scoped);
        assert_eq!((code, v["result"].as_str()), (want.0, Some(want.1)), "{f}: {v:#}");
        v
    };

    let v = decided(&program("let_out", "", "    let u = \"https://example.com/admin\"\n", "u"), (1, "exceeds"));
    assert_eq!(
        (v["counterexample"]["grant"].as_str(), v["counterexample"]["at"]["line"].as_u64()),
        (Some("net=example.com/admin"), Some(7)),
        "named at the request's call: {v:#}"
    );
    decided(&program("let_in", "", "    let u = \"https://example.com/api/v1\"\n", "u"), (0, "within"));
    let top = "\nlet ADMIN: Str = \"https://example.com/admin\"\n";
    decided(&program("const_out", top, "", "ADMIN"), (1, "exceeds"));
    let top_in = "\nlet ITEMS: Str = \"https://example.com/api/items\"\n";
    decided(&program("const_in", top_in, "", "ITEMS"), (0, "within"));

    // A name that may hold another value at the call.
    let branch = "    var u = \"https://example.com/api/v1\"\n    if 1 > 0 {\n        u = \"https://example.com/admin\"\n    }\n";
    decided(&program("let_branch", "", branch, "u"), (1, "inconclusive"));
    decided(
        &program("const_shadowed", top_in, "    let ITEMS = \"https://example.com/\" + \"admin\"\n", "ITEMS"),
        (1, "inconclusive"),
    );
    let param = dir.join("const_param.delulu");
    std::fs::write(
        &param,
        format!(
            "module const_param\n{top_in}\nfn fetch(h: Cap[Http], ITEMS: Str) -> Int ! {{Net}} {{\n    match h.get(ITEMS) {{\n        \
             Ok(body) => 1\n        Err(e) => 0\n    }}\n}}\n\nfn main(root: Root) ! {{Net}} {{\n    \
             let h = root.http([\"example.com/api/**\"])\n    let n = fetch(h, \"https://example.com/\" + \"admin\")\n}}\n"
        ),
    )
    .unwrap();
    let param = param.to_string_lossy().to_string();
    let (code, v) = within(&param, &["net=example.com/api/**".to_string()]);
    assert_eq!((code, v["result"].as_str()), (1, Some("inconclusive")), "a parameter shadows the constant: {v:#}");

    // A sent secret's audience is proposed from the literal its URL names.
    let send = dir.join("send.delulu");
    std::fs::write(
        &send,
        "module send\n\nfn main(root: Root) ! {Net} {\n    let h = root.http([\"api.example.com\"])\n    \
         let tok = root.secret(\"A\")\n    let u = \"https://api.example.com:8443/v1\"\n    \
         let ok = match h.get_with_secret(u, Map(), \"Authorization\", tok) {\n        Ok(b) => true\n        Err(e) => false\n    }\n}\n",
    )
    .unwrap();
    let send = send.to_string_lossy().to_string();
    assert_eq!(required(&send), ["net=api.example.com", "secret:A@api.example.com:8443=VALUE"]);
    let (code, v) = within(&send, &["net=api.example.com".to_string(), "secret:A@api.example.com:8443=v".to_string()]);
    assert_eq!((code, v["result"].as_str()), (0, Some("within")), "{v:#}");
    let (code, v) = within(&send, &["net=api.example.com".to_string(), "secret:A@api.example.com:443=v".to_string()]);
    assert_eq!((code, v["result"].as_str()), (1, Some("exceeds")), "the port is the literal's: {v:#}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// SEND-REBIND-1 (D-V2-180's walk): a secret the program sends is attributed to the name it was bound to only where nothing
/// may have bound it again. The walk restored each block's bindings at its end, which also undid an assignment made inside it,
/// so after `if c { tok = root.secret("B") }`, in a loop that assigns after it sends, and in a closure called after
/// `tok = …` (closures capture by reference), the send was attributed to the secret `tok` was first bound to, and `--within`
/// answered *within* for a boundary giving `B` no audience: a run that sends `B` is refused. Such a send is now a secret the
/// source does not name; straight-line code keeps its attribution.
#[test]
fn a_secret_reassigned_in_a_branch_a_loop_or_a_closure_is_not_attributed_to_its_first_value() {
    let dir = std::env::temp_dir().join(format!("delulu-within-rebind-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let send = |url: &str| {
        format!(
            "match h.get_with_secret(\"https://api.example.com/{url}\", Map(), \"Authorization\", tok) {{\n            \
             Ok(b) => true\n            Err(e) => false\n        }}"
        )
    };
    let program = |name: &str, body: &str| {
        let f = dir.join(format!("{name}.delulu"));
        std::fs::write(
            &f,
            format!(
                "module {name}\n\nfn main(root: Root) ! {{Net}} {{\n    let h = root.http([\"api.example.com\"])\n    \
                 var tok = root.secret(\"A\")\n{body}\n}}\n"
            ),
        )
        .unwrap();
        f.to_string_lossy().to_string()
    };
    let branch =
        program("branch", &format!("    if 1 > 0 {{\n        tok = root.secret(\"B\")\n    }}\n    let ok = {}", send("v1")));
    let looped = program(
        "looped",
        &format!(
            "    var i = 0\n    while i < 2 {{\n        let ok = {}\n        tok = root.secret(\"B\")\n        i = i + 1\n    }}",
            send("v1")
        ),
    );
    let closure = program(
        "closure",
        &format!(
            "    let f = fn() -> Bool ! {{Net}} {{\n        {}\n    }}\n    tok = root.secret(\"B\")\n    let ok = f()",
            send("v1")
        ),
    );
    let unnamed = "secret:NAME@api.example.com:443=VALUE".to_string();
    let grants = |a: &str, b: &str| vec!["net=api.example.com".to_string(), format!("secret:A{a}=v"), format!("secret:B{b}=v")];
    for f in [&branch, &looped, &closure] {
        let req = required(f);
        assert!(req.contains(&unnamed) && req.contains(&"secret:A=VALUE".to_string()), "{f}: {req:?}");
        let (code, v) = within(f, &grants("@api.example.com", ""));
        assert_eq!((code, v["result"].as_str()), (1, Some("inconclusive")), "{f}: B may be the one sent: {v:#}");
        let (code, v) = within(f, &grants("@api.example.com", "@api.example.com"));
        assert_eq!((code, v["result"].as_str()), (0, Some("within")), "{f}: either secret may go there: {v:#}");
    }

    // Straight-line code: each send is the secret its name holds at that point.
    let straight = program(
        "straight",
        &format!(
            "    tok = root.secret(\"B\")\n    let one = {}\n    if 1 > 0 {{\n        tok = root.secret(\"C\")\n        let two = {}\n    }}",
            send("v1"),
            send("v2")
        ),
    );
    assert_eq!(
        required(&straight),
        ["net=api.example.com", "secret:A=VALUE", "secret:B@api.example.com:443=VALUE", "secret:C@api.example.com:443=VALUE"]
    );
    let _ = std::fs::remove_dir_all(&dir);
}
