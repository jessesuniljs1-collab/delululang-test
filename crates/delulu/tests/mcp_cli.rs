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
    std::fs::write(dir.join("bad.delulu"), "module b\n\nfn main(root: Root) {\n    let c = root.console()\n    c.println(\"hi\")\n}\n").unwrap();
    std::fs::write(dir.join("ok.delulu"), "module w\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"hi\")\n}\n").unwrap();
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
    for want in ["check", "authority", "why", "explain", "atlas", "atlas_query", "toolchain", "schema", "sandbox_probe", "sandbox_policy", "examples"] {
        assert!(names.contains(&want), "`{want}` is offered: {names:?}");
    }
    for never in ["run", "test", "grants", "broker", "fix", "fmt", "build", "survey_query", "doctor_check"] {
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
        ],
    );
    let names: Vec<String> =
        reply(&replies, 1)["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
    for want in ["survey_query", "survey_impact", "doctor_check"] {
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
}
