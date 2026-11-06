//! P4-03: `delulu mcp` driven over its real wire — newline-delimited JSON-RPC on stdin and stdout —
//! the way `lsp_cli.rs` drives the language server.
//!
//! What is held here: the handshake and its version negotiation; a deterministic, read-only tool list;
//! a tool's answer being byte-for-byte the CLI's `--json` answer (there is no second implementation);
//! a hostile argument refused rather than passed on as an option; protocol errors for an unknown tool,
//! an unknown method and a line that is not JSON; no reply to a notification; and the source-tree tools
//! answering inside the tree and absent outside it.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-mcp-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Send every message, close stdin, and return every reply keyed by its id (in order).
fn converse(cwd: &Path, messages: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("mcp")
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start delulu mcp");
    {
        let mut stdin = child.stdin.take().unwrap();
        for m in messages {
            writeln!(stdin, "{m}").unwrap();
        }
        // A raw line that is not JSON, sent as a client might by mistake.
        if messages.iter().any(|m| m["id"] == json!("malformed-marker")) {
            writeln!(stdin, "{{ this is not json").unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "mcp exited {:?}: {}", out.status, String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("a reply line is not JSON ({e}): {l}")))
        .collect()
}

fn reply(replies: &[Value], id: i64) -> &Value {
    replies.iter().find(|r| r["id"] == json!(id)).unwrap_or_else(|| panic!("no reply for id {id}: {replies:#?}"))
}

fn call(id: i64, name: &str, arguments: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": arguments } })
}

#[test]
fn the_server_speaks_the_protocol_and_answers_with_the_clis_own_json() {
    let dir = scratch("proto");
    std::fs::write(
        dir.join("bad.delulu"),
        "module b\n\nfn main(root: Root) {\n    let c = root.console()\n    c.println(\"hi\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("ok.delulu"),
        "module w\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"hi\")\n}\n",
    )
    .unwrap();
    let replies = converse(
        &dir,
        &[
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } }),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
            call(3, "check", json!({ "paths": ["bad.delulu"] })),
            call(4, "authority", json!({ "path": "ok.delulu" })),
            call(5, "authority", json!({ "path": "--grant=net=evil.example" })),
            call(6, "run", json!({ "path": "ok.delulu" })),
            json!({ "jsonrpc": "2.0", "id": 7, "method": "resources/list" }),
            json!({ "jsonrpc": "2.0", "id": 8, "method": "ping" }),
            json!({ "jsonrpc": "2.0", "id": 9, "method": "initialize", "params": { "protocolVersion": "1999-01-01" } }),
            call(10, "explain", json!({ "code": "DL0501" })),
            json!({ "jsonrpc": "2.0", "id": "malformed-marker", "method": "ping" }),
        ],
    );

    // The handshake: the client's revision is answered when it is one this server speaks, the newest
    // otherwise; the server says what it is and that it is read-only.
    let init = &reply(&replies, 1)["result"];
    assert_eq!(init["protocolVersion"], "2025-03-26");
    assert_eq!(init["serverInfo"]["name"], "delulu");
    assert!(init["capabilities"]["tools"].is_object());
    assert!(init["instructions"].as_str().unwrap().contains("Nothing here runs a program"));
    assert_eq!(reply(&replies, 9)["result"]["protocolVersion"], "2025-06-18", "an unknown revision gets the newest");

    // The list: read-only tools, none of them an effector, outside the source tree no Survey.
    let tools = reply(&replies, 2)["result"]["tools"].as_array().unwrap().clone();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for want in [
        "check",
        "authority",
        "why",
        "explain",
        "atlas",
        "atlas_query",
        "atlas_chain",
        "toolchain",
        "schema",
        "sandbox_probe",
        "sandbox_policy",
        "examples",
    ] {
        assert!(names.contains(&want), "`{want}` is offered: {names:?}");
    }
    for never in ["run", "test", "grants", "broker", "fix", "fmt", "build", "edit", "survey_query", "survey_diff", "doctor_check"]
    {
        assert!(!names.contains(&never), "`{never}` must not be offered here: {names:?}");
    }
    for t in &tools {
        assert_eq!(t["annotations"]["readOnlyHint"], true, "{}", t["name"]);
        assert!(t["inputSchema"]["type"] == "object", "{}", t["name"]);
    }

    // A tool's answer IS the CLI's answer.
    let checked = &reply(&replies, 3)["result"];
    let direct: Value = serde_json::from_slice(
        &Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(["check", "bad.delulu", "--json"])
            .current_dir(&dir)
            .env("DELULU_NO_FIRST_RUN", "1")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(checked["structuredContent"], direct, "the check tool is `delulu check --json`, not a copy of it");
    assert_eq!(checked["structuredContent"]["diagnostics"][0]["code"], "DL0501");
    assert_eq!(checked["isError"], false, "a program with errors is a successful CHECK");
    let auth = &reply(&replies, 4)["result"];
    assert_eq!(auth["isError"], false);
    assert!(auth["structuredContent"]["authority"]["required_grants"].as_array().unwrap().iter().any(|g| g == "console"));
    assert!(auth["content"][0]["text"].as_str().unwrap().contains("\"authority\""), "text content carries the same JSON");

    // A hostile argument is refused, and never reaches a command line as an option.
    let hostile = &reply(&replies, 5)["result"];
    assert_eq!(hostile["isError"], true);
    assert!(hostile["content"][0]["text"].as_str().unwrap().contains("may not begin with `-`"), "{hostile}");

    // Protocol errors: no such tool, no such method; `ping` answers; a non-JSON line is a parse error.
    assert_eq!(reply(&replies, 6)["error"]["code"], -32602, "there is no `run` tool");
    assert_eq!(reply(&replies, 7)["error"]["code"], -32601);
    assert_eq!(reply(&replies, 8)["result"], json!({}));
    assert!(reply(&replies, 10)["result"]["structuredContent"]["explain"]["body"].is_string());
    assert!(replies.iter().any(|r| r["error"]["code"] == -32700 && r["id"].is_null()), "a line that is not JSON: {replies:#?}");
    // The notification got no reply: every reply has an id or is the parse error.
    assert!(replies.iter().all(|r| !r["id"].is_null() || r["error"]["code"] == -32700));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Inside the source tree the Survey and `doctor --check` are tools; their answers are the Survey's
/// own (`delulu-survey --json`'s shape) and `doctor`'s.
#[test]
fn inside_the_source_tree_the_survey_and_doctor_are_tools() {
    let replies = converse(
        &repo(),
        &[
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
            call(2, "survey_query", json!({ "id": "crate:delulu-check" })),
            call(3, "survey_impact", json!({ "id": "crate:delulu-diag", "depth": 1 })),
            call(4, "survey_query", json!({ "id": "crate:no-such-crate" })),
            call(5, "survey_diff", json!({ "rev": "HEAD" })),
            call(6, "survey_diff", json!({ "rev": "--output=x" })),
        ],
    );
    let names: Vec<String> = reply(&replies, 1)["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    for want in ["survey_query", "survey_impact", "survey_diff", "doctor_check"] {
        assert!(names.contains(&want.to_string()), "{want}: {names:?}");
    }
    let q = &reply(&replies, 2)["result"]["structuredContent"];
    assert_eq!(q["tool"], "delulu-survey");
    assert_eq!(q["verb"], "query");
    assert_eq!(q["node"]["id"], "crate:delulu-check");
    assert!(q.get("entrenched").is_some(), "entrenchment is always answered");
    let imp = &reply(&replies, 3)["result"]["structuredContent"];
    assert_eq!(imp["verb"], "impact");
    assert!(imp["reached"].as_u64().unwrap() > 0, "something depends on delulu-diag");
    let miss = &reply(&replies, 4)["result"];
    assert_eq!(miss["isError"], true);
    assert!(miss["content"][0]["text"].as_str().unwrap().contains("no node"));
    // P4-06: a change's blast radius, in the Survey's own `diff` shape; an option as a revision refused.
    let d = &reply(&replies, 5)["result"];
    assert_eq!(d["isError"], false, "{d}");
    assert_eq!(d["structuredContent"]["verb"], "diff");
    assert!(d["structuredContent"]["hops"].is_array());
    let opt = &reply(&replies, 6)["result"];
    assert_eq!(opt["isError"], true);
    assert!(opt["content"][0]["text"].as_str().unwrap().contains("may not begin with `-`"), "{opt}");
}

/// RW 4.39's rest (routine run 7): a request line was read whole whatever its length — the red team's
/// 300 MiB line took the server to 333 MB. A line is bounded now: one past the bound is answered with an
/// error naming the bound, what follows it is still served, and the line is never held whole.
#[test]
fn a_request_line_past_the_bound_is_refused_and_the_session_goes_on() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("delulu mcp starts");
    {
        let mut stdin = child.stdin.take().unwrap();
        let chunk = vec![b'x'; 1 << 20];
        for _ in 0..17 {
            stdin.write_all(&chunk).unwrap();
        }
        stdin.write_all(b"\n").unwrap();
        writeln!(stdin, "{}", json!({ "jsonrpc": "2.0", "id": 7, "method": "ping" })).unwrap();
    }
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let replies: Vec<Value> = text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
    assert_eq!(replies.len(), 2, "one error, then the ping answered: {text}");
    let msg = replies[0]["error"]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("exceeds") && msg.contains("MiB"), "the error names the bound: {}", replies[0]);
    assert_eq!(replies[1]["id"], 7, "the session goes on: {}", replies[1]);
}

/// A program `check` takes seconds over: thousands of small recursive functions.
fn slow_program(dir: &Path) -> String {
    let mut src = String::from("module big\n\n");
    for i in 0..16_000 {
        src.push_str(&format!("fn f{i}(x: Int) -> Int {{\n  if x < 1 {{ {i} }} else {{ f{i}(x - 1) + x * {} }}\n}}\n\n", i % 7));
    }
    src.push_str("fn main(root: Root) {}\n");
    let path = dir.join("big.delulu");
    std::fs::write(&path, src).unwrap();
    path.to_string_lossy().into_owned()
}

/// [`converse`], timed: every reply in the order the server wrote it, and how long the server took to exit after its
/// input closed.
fn converse_timed(cwd: &Path, messages: &[Value]) -> (Vec<Value>, std::time::Duration) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("mcp")
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start delulu mcp");
    let started = std::time::Instant::now();
    {
        let mut stdin = child.stdin.take().unwrap();
        for m in messages {
            writeln!(stdin, "{m}").unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    let took = started.elapsed();
    assert!(out.status.success(), "mcp exited {:?}: {}", out.status, String::from_utf8_lossy(&out.stderr));
    let replies = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("a reply line is not JSON ({e}): {l}")))
        .collect();
    (replies, took)
}

/// RW 6.74: a call runs beside the reader, so a `ping` behind a slow call is answered before it; a call the client
/// cancels (`notifications/cancelled`) is not answered, and its child is killed rather than run to its end — the
/// specification's SHOULD. The kill is measured against a control on the same machine: the same slow call, not
/// cancelled, holds the session open for its whole check; cancelled, the session ends in a fraction of that.
#[test]
fn a_cancelled_call_is_not_answered_and_a_ping_behind_a_call_is_answered_first() {
    let dir = scratch("cancel");
    let big = slow_program(&dir);
    let check = |id: i64| call(id, "check", json!({ "paths": [big] }));
    let ping = |id: i64| json!({ "jsonrpc": "2.0", "id": id, "method": "ping" });
    let cancel = |id: i64| json!({ "jsonrpc": "2.0", "method": "notifications/cancelled", "params": { "requestId": id, "reason": "test" } });

    // The control: the slow call answered, and a ping behind it answered first.
    let (replies, whole) = converse_timed(&dir, &[check(1), ping(2)]);
    let ids: Vec<&Value> = replies.iter().map(|r| &r["id"]).collect();
    assert_eq!(ids, [&json!(2), &json!(1)], "the ping is not held behind the call: {replies:#?}");
    assert_eq!(reply(&replies, 1)["result"]["isError"], false, "the control's check ran clean: {replies:#?}");

    // Cancelled: no answer for it, the ping answered, and the session over long before the check would have been.
    let (replies, cut) = converse_timed(&dir, &[check(3), cancel(3), ping(4)]);
    assert!(replies.iter().all(|r| r["id"] != json!(3)), "a cancelled call is not answered: {replies:#?}");
    assert_eq!(replies.len(), 1, "{replies:#?}");
    assert_eq!(replies[0]["id"], 4, "{replies:#?}");
    assert!(cut * 2 < whole, "the cancelled call's child was killed: the session took {cut:?}, the control {whole:?}");

    // A cancellation for an id no call has, or for an answered one, changes nothing.
    let (replies, _) = converse_timed(&dir, &[cancel(99), ping(5)]);
    assert_eq!(replies.len(), 1, "a notification is never answered: {replies:#?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// RW 6.74: every tool that answers with the CLI's envelope declares it as `outputSchema` — the `delulu schema`
/// document, with the command's own payload named where a closed schema describes it — and what it returns conforms:
/// the envelope, and the payload, are each valid by `delulu schema validate`.
#[test]
fn each_tools_output_schema_is_the_clis_own_and_its_answer_conforms() {
    let dir = scratch("output-schema");
    let demo = repo().join("examples/demo.delulu").to_string_lossy().into_owned();
    let replies = converse(
        &dir,
        &[
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
            call(2, "authority", json!({ "path": demo })),
            call(3, "toolchain", json!({})),
            call(4, "check", json!({ "paths": [demo] })),
            call(5, "atlas_chain", json!({ "path": demo })),
        ],
    );
    let tools = reply(&replies, 1)["result"]["tools"].as_array().unwrap().clone();
    let schema_of = |name: &str| tools.iter().find(|t| t["name"] == name).unwrap()["outputSchema"].clone();
    for t in &tools {
        let s = &t["outputSchema"];
        if t["name"].as_str().unwrap().starts_with("survey_") {
            assert!(s.is_null(), "a Survey answer is not an envelope: {t}");
        } else {
            assert_eq!(s["$id"], "delulu:schema/envelope", "{}", t["name"]);
            assert_eq!(s["type"], "object", "MCP asks for an object schema: {}", t["name"]);
            assert!(s["$defs"]["diagnostic"].is_object(), "self-contained: {}", t["name"]);
        }
    }
    assert!(schema_of("authority")["properties"]["authority"].is_object(), "the payload, named");
    assert!(schema_of("toolchain")["properties"]["toolchain"].is_object(), "the payload, named");
    assert!(schema_of("atlas_chain")["properties"]["chain"].is_object(), "the payload, named");
    let valid = |schema: &str, v: &Value| {
        let f = dir.join(format!("{schema}.json"));
        std::fs::write(&f, v.to_string()).unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(["schema", "validate", schema, f.to_str().unwrap()])
            .env("DELULU_NO_FIRST_RUN", "1")
            .output()
            .unwrap();
        assert!(o.status.success(), "{schema}: {}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    };
    for (id, payload) in [(2, Some("authority")), (3, Some("toolchain")), (4, None), (5, Some("chain"))] {
        let content = &reply(&replies, id)["result"]["structuredContent"];
        valid("envelope", content);
        if let Some(p) = payload {
            valid(p, &content[p]);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
