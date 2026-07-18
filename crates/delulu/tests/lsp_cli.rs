//! Stage 8, phase 8e: the LSP smoke suite (criteria 1–3) — a minimal real client
//! driving `delulu lsp` over actual stdio JSON-RPC framing.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

struct Client {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    next_id: i64,
}

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(["lsp"])
            .env("DELULU_NO_FIRST_RUN", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn delulu lsp");
        let stdin = child.stdin.take().unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        let mut c = Client { child, stdin, reader, next_id: 1 };
        let init = c.request("initialize", json!({ "capabilities": {} }));
        assert!(init["capabilities"]["hoverProvider"].as_bool().unwrap_or(false));
        c.notify("initialized", json!({}));
        c
    }

    fn send(&mut self, payload: Value) {
        let body = payload.to_string();
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.stdin.flush().unwrap();
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn read_message(&mut self) -> Value {
        let mut len = 0usize;
        loop {
            let mut line = String::new();
            assert!(self.reader.read_line(&mut line).unwrap() > 0, "server hung up");
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.strip_prefix("Content-Length:") {
                len = v.trim().parse().unwrap();
            }
        }
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf).unwrap();
        serde_json::from_slice(&buf).unwrap()
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let msg = self.read_message();
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                return msg["result"].clone();
            }
            // Interleaved notifications (diagnostics) are read past here.
        }
    }

    fn open(&mut self, uri: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "delulu", "version": 1, "text": text } }),
        );
    }

    fn wait_diagnostics(&mut self, uri: &str) -> Value {
        loop {
            let msg = self.read_message();
            if msg["method"] == "textDocument/publishDiagnostics"
                && msg["params"]["uri"] == uri
            {
                return msg["params"]["diagnostics"].clone();
            }
        }
    }

    fn shutdown(mut self) {
        let _ = self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        let status = self.child.wait().unwrap();
        assert_eq!(status.code(), Some(0), "clean shutdown");
    }
}

/// Convert an LSP position back to a byte offset in `text` (UTF-16 aware) so spans can
/// be compared against `delulu check --json` byte offsets exactly.
fn pos_to_byte(text: &str, line: u64, character: u64) -> u64 {
    let mut cur = 0u64;
    let mut off = 0usize;
    if line > 0 {
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                cur += 1;
                if cur == line {
                    off = i + 1;
                    break;
                }
            }
        }
    }
    let mut units = 0u64;
    for c in text[off..].chars() {
        if units >= character || c == '\n' {
            break;
        }
        units += c.len_utf16() as u64;
        off += c.len_utf8();
    }
    off as u64
}

const DL0501_SRC: &str = "module m\nfn greet(out: Cap[Console], n: Str) { out.println(n) }\n";

/// Criterion 1 (core): push diagnostics equal `delulu check --json` — same code, same
/// byte span; the DL0501 code action applies its repair and the re-check goes green.
#[test]
fn criterion1_diagnostics_equal_check_json_and_the_repair_applies() {
    // The reference: the CLI's machine envelope.
    let dir = std::env::temp_dir().join(format!("delulu_lsp1_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("m.delulu");
    std::fs::write(&f, DL0501_SRC).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(["check", f.to_str().unwrap(), "--json"])
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .unwrap();
    let env: Value = serde_json::from_slice(&o.stdout).unwrap();
    let cli_d = &env["diagnostics"][0];

    let mut c = Client::start();
    c.open("file:///m.delulu", DL0501_SRC);
    let diags = c.wait_diagnostics("file:///m.delulu");
    assert_eq!(diags.as_array().unwrap().len(), 1);
    let d = &diags[0];
    assert_eq!(d["code"], cli_d["code"], "same code as check --json");
    assert_eq!(d["message"], cli_d["message"], "same message (en-US machine surface)");
    let start = pos_to_byte(
        DL0501_SRC,
        d["range"]["start"]["line"].as_u64().unwrap(),
        d["range"]["start"]["character"].as_u64().unwrap(),
    );
    assert_eq!(start, cli_d["spans"][0]["start"]["byte"].as_u64().unwrap(), "same span");

    // The code action: the add_effect_to_row repair, ⚠-flagged, applies to green.
    let actions = c.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": "file:///m.delulu" },
            "range": d["range"],
            "context": { "diagnostics": [d] }
        }),
    );
    let a = actions
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["data"]["repair_id"] == "add_effect_to_row")
        .expect("the DL0501 repair is offered");
    assert_eq!(a["data"]["authority_widening"], true, "criterion 2: flagged in data");
    assert_eq!(a["isPreferred"], false, "widening is never preferred");
    assert!(a["title"].as_str().unwrap().starts_with('⚠'), "⚠-titled");

    // Apply the edit client-side, resend, and the diagnostics must clear.
    let edit = &a["edit"]["changes"]["file:///m.delulu"][0];
    let s = pos_to_byte(
        DL0501_SRC,
        edit["range"]["start"]["line"].as_u64().unwrap(),
        edit["range"]["start"]["character"].as_u64().unwrap(),
    ) as usize;
    let e = pos_to_byte(
        DL0501_SRC,
        edit["range"]["end"]["line"].as_u64().unwrap(),
        edit["range"]["end"]["character"].as_u64().unwrap(),
    ) as usize;
    let mut fixed = DL0501_SRC.to_string();
    fixed.replace_range(s..e, edit["newText"].as_str().unwrap());
    c.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": "file:///m.delulu", "version": 2 },
            "contentChanges": [{ "text": fixed }]
        }),
    );
    let diags2 = c.wait_diagnostics("file:///m.delulu");
    assert_eq!(diags2.as_array().unwrap().len(), 0, "the repair makes it green: {fixed}");
    c.shutdown();
}

/// Criterion 1 (hover): a fn name shows its full signature + row + transitive authority.
#[test]
fn hover_shows_signature_row_and_authority() {
    let src = "module m\nfn greet(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\n";
    let mut c = Client::start();
    c.open("file:///h.delulu", src);
    let _ = c.wait_diagnostics("file:///h.delulu");
    let name_at = src.find("greet").unwrap();
    let h = c.request(
        "textDocument/hover",
        json!({
            "textDocument": { "uri": "file:///h.delulu" },
            "position": { "line": 1, "character": name_at - 9 } // "fn " is at line 1 col 0
        }),
    );
    let md = h["contents"]["value"].as_str().expect("hover markdown");
    assert!(md.contains("Cap[Console]") && md.contains("Str"), "signature: {md}");
    assert!(md.contains("Write"), "the row is ON the hover: {md}");
    assert!(md.contains("authority:"), "transitive authority line: {md}");
    c.shutdown();
}

/// Criterion 2: the authority lens — inferred rows appear on unannotated lambdas.
#[test]
fn criterion2_inlay_hints_on_unannotated_lambdas() {
    let src = "module m\nfn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e { f(x) }\n\
               fn go(out: Cap[Console]) ! {Write} { let r = apply(fn(n: Int) -> Int { out.println(\"x\")\n n }, 1)\n }\n";
    let checked = delulu_check::check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let mut c = Client::start();
    c.open("file:///i.delulu", src);
    let _ = c.wait_diagnostics("file:///i.delulu");
    let hints = c.request(
        "textDocument/inlayHint",
        json!({
            "textDocument": { "uri": "file:///i.delulu" },
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 10, "character": 0 } }
        }),
    );
    let arr = hints.as_array().unwrap();
    assert!(
        arr.iter().any(|h| h["label"].as_str().unwrap_or("").contains("Write")
            && h["data"]["authority_lens"] == true),
        "the lambda's inferred row surfaces: {hints}"
    );
    c.shutdown();
}

#[test]
fn document_symbols_include_actors_behaviors_and_tests() {
    let src = "module m\nactor A {\n var n: Int\n new() { self.n = 0 }\n be tick() { self.n = self.n + 1 }\n}\ntest \"t\" { assert(true) }\nfn f() -> Int { 1 }\n";
    let mut c = Client::start();
    c.open("file:///s.delulu", src);
    let _ = c.wait_diagnostics("file:///s.delulu");
    let syms = c.request(
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": "file:///s.delulu" } }),
    );
    let names: Vec<String> = syms
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"A".to_string()), "{names:?}");
    assert!(names.contains(&"test \"t\"".to_string()), "{names:?}");
    assert!(names.contains(&"f".to_string()), "{names:?}");
    let actor = &syms.as_array().unwrap().iter().find(|s| s["name"] == "A").unwrap();
    assert!(
        actor["children"].as_array().unwrap().iter().any(|c2| c2["name"] == "tick"),
        "behaviors are children"
    );
    c.shutdown();
}

/// Criterion 3's measured half (the CI-grade release-mode gate is `#[ignore]`d below):
/// edit-to-diagnostics on the 10-kLoC reference workspace.
fn ten_kloc() -> String {
    let mut src = String::from("module big\n");
    for i in 0..500 {
        src.push_str(&format!(
            "fn f{i}(a: Int, b: Int) -> Int {{\n    let c = a + b\n    let d = c * 2\n    let e = d - a\n\
             \x20   let g = if e > 0 {{ e }} else {{ 0 - e }}\n    let h = g + {i}\n\
             \x20   let k = h % 97\n    let l = k + c\n    let m2 = l * 3\n    let n = m2 - d\n\
             \x20   let o = n + g\n    let p = o % 1000\n    let q = p + k\n    let r = q - l\n\
             \x20   let s = r + m2\n    let t = s % 500\n    let u = t + n\n    let v = u - o\n\
             \x20   v + p\n}}\n"
        ));
    }
    src
}

#[test]
fn latency_smoke_diagnostics_arrive_promptly_on_10kloc() {
    let src = ten_kloc();
    assert!(src.lines().count() >= 10_000, "{} lines", src.lines().count());
    let mut c = Client::start();
    let t0 = std::time::Instant::now();
    c.open("file:///big.delulu", &src);
    let diags = c.wait_diagnostics("file:///big.delulu");
    let elapsed = t0.elapsed();
    assert_eq!(diags.as_array().unwrap().len(), 0, "the reference workspace is clean");
    eprintln!("lsp 10-kLoC edit-to-diagnostics (debug build): {elapsed:?}");
    // Debug-build smoke ceiling; the ≤150 ms criterion runs release (`--ignored` gate).
    assert!(elapsed.as_millis() < 2_000, "debug smoke ceiling: {elapsed:?}");
    c.shutdown();
}

/// Criterion 1's rename half, cross-module: the declaration lives in one open document,
/// a qualified reference in another — one rename updates BOTH (deviation 3's walk).
#[test]
fn criterion1_rename_updates_both_open_documents() {
    let lib = "module lib\npub fn greet(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\n";
    let app = "module app\nimport lib\nfn main(root: Root) ! {Write} { lib.greet(root.console(), \"hi\") }\n";
    let mut c = Client::start();
    c.open("file:///lib.delulu", lib);
    let _ = c.wait_diagnostics("file:///lib.delulu");
    c.open("file:///app.delulu", app);
    let _ = c.wait_diagnostics("file:///app.delulu");

    // definition: from the qualified use in app to the decl in lib.
    let use_at = app.find("greet").unwrap();
    let def = c.request(
        "textDocument/definition",
        json!({
            "textDocument": { "uri": "file:///app.delulu" },
            "position": { "line": 2, "character": app.lines().nth(2).unwrap().find("greet").unwrap() }
        }),
    );
    assert_eq!(def["uri"], "file:///lib.delulu", "definition jumps across documents: {def}");
    let _ = use_at;

    // references: both documents.
    let refs = c.request(
        "textDocument/references",
        json!({
            "textDocument": { "uri": "file:///lib.delulu" },
            "position": { "line": 1, "character": lib.lines().nth(1).unwrap().find("greet").unwrap() },
            "context": { "includeDeclaration": true }
        }),
    );
    let uris: Vec<&str> =
        refs.as_array().unwrap().iter().map(|r| r["uri"].as_str().unwrap()).collect();
    assert!(uris.contains(&"file:///lib.delulu") && uris.contains(&"file:///app.delulu"), "{refs}");

    // rename: edits land in BOTH files.
    let edit = c.request(
        "textDocument/rename",
        json!({
            "textDocument": { "uri": "file:///lib.delulu" },
            "position": { "line": 1, "character": lib.lines().nth(1).unwrap().find("greet").unwrap() },
            "newName": "welcome"
        }),
    );
    let changes = edit["changes"].as_object().unwrap();
    assert!(changes.contains_key("file:///lib.delulu"), "{edit}");
    assert!(changes.contains_key("file:///app.delulu"), "{edit}");
    assert_eq!(changes["file:///app.delulu"][0]["newText"], "welcome");
    c.shutdown();
}

#[test]
fn rename_of_a_local_is_refused_honestly() {
    let src = "module m\nfn f() -> Int {\n    let local_x = 1\n    local_x\n}\n";
    let mut c = Client::start();
    c.open("file:///l.delulu", src);
    let _ = c.wait_diagnostics("file:///l.delulu");
    let id = c.next_id;
    c.next_id += 1;
    c.send(json!({
        "jsonrpc": "2.0", "id": id, "method": "textDocument/rename",
        "params": {
            "textDocument": { "uri": "file:///l.delulu" },
            "position": { "line": 2, "character": 8 },
            "newName": "y"
        }
    }));
    loop {
        let msg = c.read_message();
        if msg.get("id").and_then(Value::as_i64) == Some(id) {
            let e = msg["error"]["message"].as_str().expect("locals refuse rename");
            assert!(e.contains("locals are refused"), "{e}");
            break;
        }
    }
    c.shutdown();
}

#[test]
fn semantic_tokens_carry_the_spec_named_kinds() {
    let src = "module m\nfn f(out: Cap[Console], s: Secret[Str], xs: iso List[Int]) ! {Write} { out.println(\"x\") }\n";
    let mut c = Client::start();
    c.open("file:///t.delulu", src);
    let _ = c.wait_diagnostics("file:///t.delulu");
    let toks = c.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": "file:///t.delulu" } }),
    );
    let data = toks["data"].as_array().unwrap();
    assert!(data.len() >= 5 * 5, "a real token stream: {} entries", data.len());
    let kinds: Vec<u64> = data.chunks(5).map(|c| c[3].as_u64().unwrap()).collect();
    assert!(kinds.contains(&8), "effect kind present: {kinds:?}");
    assert!(kinds.contains(&9), "rcap kind present: {kinds:?}");
    assert!(kinds.contains(&10), "capability kind present: {kinds:?}");
    assert!(kinds.contains(&11), "secret kind present: {kinds:?}");
    c.shutdown();
}

#[test]
fn code_lenses_on_main_and_tests_and_the_authority_command() {
    let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"x\") }\n\
               test \"t\" { assert(true) }\n";
    let mut c = Client::start();
    c.open("file:///cl.delulu", src);
    let _ = c.wait_diagnostics("file:///cl.delulu");
    let lenses = c.request(
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": "file:///cl.delulu" } }),
    );
    let titles: Vec<String> = lenses
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["command"]["title"].as_str().unwrap().to_string())
        .collect();
    assert!(titles.iter().any(|t| t == "▶ run"), "{titles:?}");
    assert!(titles.iter().any(|t| t.starts_with("authority: ") && t.contains("Write")), "{titles:?}");
    assert!(titles.iter().any(|t| t == "▶ run test"), "{titles:?}");

    // The agent-facing command: the §10.5 report over the wire.
    let report = c.request(
        "workspace/executeCommand",
        json!({ "command": "delulu.authority", "arguments": ["file:///cl.delulu"] }),
    );
    let effects: Vec<&str> =
        report["effects"].as_array().unwrap().iter().map(|e| e.as_str().unwrap()).collect();
    assert!(effects.contains(&"Write"), "the compiler-computed report: {report}");
    c.shutdown();
}

/// Criterion 3 proper: ≤150 ms edit-to-diagnostics, release build.
/// `cargo test -p delulu --release --test lsp_cli -- --ignored criterion3`
#[test]
#[ignore]
fn criterion3_latency_150ms_on_10kloc_release() {
    let src = ten_kloc();
    let mut c = Client::start();
    c.open("file:///big.delulu", &src);
    let _ = c.wait_diagnostics("file:///big.delulu");
    // Measure a CHANGE (the criterion is edit-to-diagnostics), best of three.
    let mut best = u128::MAX;
    for v in 2..5 {
        let edited = format!("{src}fn extra{v}() -> Int {{ {v} }}\n");
        let t0 = std::time::Instant::now();
        c.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": "file:///big.delulu", "version": v },
                "contentChanges": [{ "text": edited }]
            }),
        );
        let _ = c.wait_diagnostics("file:///big.delulu");
        best = best.min(t0.elapsed().as_millis());
    }
    eprintln!("lsp 10-kLoC edit-to-diagnostics (release, best of 3): {best} ms");
    assert!(best <= 150, "criterion 3: ≤150 ms, got {best} ms");
    c.shutdown();
}
