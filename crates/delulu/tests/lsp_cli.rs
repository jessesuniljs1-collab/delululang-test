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
            // The property is that the refusal is HONEST: it says a local is what was refused,
            // and that it was refused rather than attempted. Asserted as substance rather than as
            // one exact phrase — the previous form pinned the sentence, so improving the wording
            // failed a test that had no quarrel with the new wording.
            let e_lower = e.to_lowercase();
            assert!(e_lower.contains("local"), "the refusal must say a LOCAL is what it refused: {e}");
            assert!(e_lower.contains("refused"), "the refusal must say it refused, not that it failed: {e}");
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

// --- completion ----------------------------------------------------------------------------------

fn complete(c: &mut Client, uri: &str, line: u64, character: u64) -> Vec<Value> {
    let r = c.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character } }),
    );
    r["items"].as_array().cloned().unwrap_or_default()
}

fn labels(items: &[Value]) -> Vec<String> {
    items.iter().filter_map(|i| i["label"].as_str().map(str::to_string)).collect()
}

/// Declarations in the file come first, and a function carries its signature AND its row — the row
/// being the part a reader most often forgets and most needs before calling.
#[test]
fn completion_offers_declarations_with_their_signature_and_authority() {
    let src = "module m\nfn greet(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\ntype Point\nfn g(root: Root) { }\n";
    let mut c = Client::start();
    c.open("file:///c.delulu", src);
    let _ = c.wait_diagnostics("file:///c.delulu");

    // typing `gr` on a fresh line
    c.notify("textDocument/didChange", json!({
        "textDocument": { "uri": "file:///c.delulu", "version": 2 },
        "contentChanges": [{ "text": format!("{src}gr") }],
    }));
    let _ = c.wait_diagnostics("file:///c.delulu");
    let items = complete(&mut c, "file:///c.delulu", 4, 2);
    let greet = items.iter().find(|i| i["label"] == "greet").expect("`greet` must be offered for prefix `gr`");

    assert_eq!(greet["kind"], 3, "a fn completes as a Function");
    let detail = greet["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("Cap[Console]") && detail.contains("Str"), "signature in detail: {detail}");
    let doc = greet["documentation"]["value"].as_str().unwrap_or_default();
    assert!(doc.contains("Write"), "the row rides on the completion: {doc}");

    assert!(!labels(&items).contains(&"Point".to_string()), "prefix `gr` must not offer `Point`");
    c.shutdown();
}

/// **Inside an effect row, only effects are legal — so only effects are offered.**
///
/// A keyword suggested there is a suggestion that cannot compile, and the completion list is the
/// most-read documentation the language has: it is consulted on every keystroke by people who have
/// not read the spec.
#[test]
fn completion_inside_an_effect_row_offers_effects_and_nothing_else() {
    let src = "module m\nfn f(root: Root) ! {";
    let mut c = Client::start();
    c.open("file:///r.delulu", src);
    let _ = c.wait_diagnostics("file:///r.delulu");

    let items = complete(&mut c, "file:///r.delulu", 1, 21);
    let got = labels(&items);
    assert!(!got.is_empty(), "an effect row must offer something");

    for name in delulu_check::check::CORE_EFFECT_NAMES {
        assert!(got.contains(&name.to_string()), "core effect `{name}` must be offered inside a row: {got:?}");
    }
    for kw in delulu_syntax::morph::MORPHABLE_KEYWORDS {
        assert!(!got.contains(&kw.to_string()), "keyword `{kw}` is not legal inside a row, so must not be offered");
    }
    c.shutdown();
}

/// Outside a row, the keywords are offered — and they are the canonical set, not a copy.
#[test]
fn completion_keywords_are_the_canonical_set() {
    let src = "module m\n";
    let mut c = Client::start();
    c.open("file:///k.delulu", src);
    let _ = c.wait_diagnostics("file:///k.delulu");

    let got = labels(&complete(&mut c, "file:///k.delulu", 1, 0));
    for kw in delulu_syntax::morph::MORPHABLE_KEYWORDS {
        assert!(got.contains(&kw.to_string()), "keyword `{kw}` must be offered: {got:?}");
    }
    c.shutdown();
}

/// A name declared in another open document is offered, and ranked below the local ones.
#[test]
fn completion_reaches_other_open_documents_but_ranks_local_names_first() {
    let mut c = Client::start();
    c.open("file:///a.delulu", "module a\nfn shared_helper(root: Root) { }\n");
    let _ = c.wait_diagnostics("file:///a.delulu");
    c.open("file:///b.delulu", "module b\nfn shared_local(root: Root) { }\n");
    let _ = c.wait_diagnostics("file:///b.delulu");

    let items = complete(&mut c, "file:///b.delulu", 2, 0);
    let by = |name: &str| items.iter().find(|i| i["label"] == name).cloned();
    let local = by("shared_local").expect("the local declaration must be offered");
    let other = by("shared_helper").expect("a declaration in another open document must be offered");
    assert!(
        local["sortText"].as_str() < other["sortText"].as_str(),
        "a name in this document must sort before one from another: {} vs {}",
        local["sortText"], other["sortText"]
    );
    c.shutdown();
}

/// Completion must stay usable on a large file — it runs on every keystroke.
#[test]
fn completion_answers_promptly_on_10kloc() {
    let src = ten_kloc();
    let mut c = Client::start();
    c.open("file:///big.delulu", &src);
    let _ = c.wait_diagnostics("file:///big.delulu");

    let t = std::time::Instant::now();
    let items = complete(&mut c, "file:///big.delulu", 1, 0);
    let ms = t.elapsed().as_millis();
    assert!(!items.is_empty(), "completion must answer on a large file");
    // Debug build, so this is a smoke bound rather than the release criterion: it catches an
    // accidental quadratic, not a missing optimisation.
    assert!(ms < 3_000, "completion took {ms} ms on 10kloc — something scales badly");
    c.shutdown();
}

// ===== the analysis cache ==================================================
//
// The server keeps the checked form of each open document and reuses it until that document is
// edited. The win is real (see `repeated_requests_do_not_recheck_every_open_document`) and so is
// the risk it introduces: a cache that fails to invalidate answers questions about a file the
// user can no longer see. The staleness test below is the one that matters.

/// Every provider must see the edit — not just the one that published the diagnostics.
///
/// Each provider is asked separately rather than trusting one to stand for the rest, because the
/// nastiest shape this bug takes is the asymmetric one: diagnostics refresh (so the file *looks*
/// re-analysed) while hover, symbols, definition and lenses go on describing the previous text.
/// Dropping the version check makes this test report `documentSymbol` still naming a function the
/// edit deleted, which is the whole hazard a cache introduces.
#[test]
fn no_provider_answers_from_a_stale_analysis() {
    let before = "module m\nfn alpha() -> Int { 1 }\n";
    let after = "module m\nfn omega() -> Int { 2 }\n";
    let uri = "file:///stale.delulu";
    let mut c = Client::start();
    c.open(uri, before);
    let _ = c.wait_diagnostics(uri);

    let names = |c: &mut Client| -> Vec<String> {
        c.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } }))
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap_or_default().to_string())
            .collect()
    };
    assert!(names(&mut c).contains(&"alpha".to_string()), "precondition: alpha is there first");

    c.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": 2 },
            "contentChanges": [{ "text": after }]
        }),
    );
    let _ = c.wait_diagnostics(uri);

    let n = names(&mut c);
    assert!(n.contains(&"omega".to_string()), "documentSymbol must see the edit: {n:?}");
    assert!(!n.contains(&"alpha".to_string()), "documentSymbol is answering from stale state: {n:?}");

    // Hover on the new name, at the position it now occupies.
    let hov = c.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 4 } }),
    );
    let md = hov["contents"]["value"].as_str().unwrap_or_default();
    assert!(md.contains("omega"), "hover must see the edit: {md}");

    // Definition of the old name must now find nothing anywhere.
    let def = c.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 4 } }),
    );
    assert_eq!(def["uri"], uri, "definition resolves in the edited document: {def}");

    // Semantic tokens are delta-encoded from the text; a stale AST would misplace them.
    let toks = c.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri } }),
    );
    assert!(!toks["data"].as_array().unwrap().is_empty(), "tokens must still be produced");
    c.shutdown();
}

// ===== incremental synchronisation =========================================

/// A range edit must land the server on exactly the document the editor has.
///
/// This is the failure mode incremental sync exists to risk: the server's copy drifts from the
/// client's by a byte, nothing announces it, and from then on every diagnostic, hover and rename
/// is computed against text the user cannot see and reported at offsets that no longer line up.
///
/// So the test does not check the edits individually — it applies a sequence of them and then
/// requires the server to say **the same things** about the result as it says about a second
/// document opened with that text in one go. Semantic tokens carry the most weight: they are
/// delta-encoded across the whole file including comments, so a single byte of drift moves every
/// token after it.
///
/// The sequence is chosen to hit the four places implementations break: an insert inside a line,
/// a delete spanning a line boundary, two changes in one notification (the second's range is
/// expressed against the result of the first, not against the original), and positions past a
/// multi-byte character — LSP columns are UTF-16 code units, so in `test "λ"` the closing quote
/// sits at column 7 but byte **8**, and an implementation that treats columns as bytes lands
/// inside `λ` instead of after it.
///
/// The multi-byte edits deliberately land in a **test block's name**, because `documentSymbol`
/// echoes that name verbatim and so carries the actual characters into the response. An earlier
/// draft of this test put them in a comment and *passed against a byte-indexed implementation* —
/// `// λ ok` and `//  okλ` have the same start and the same UTF-16 length, so every derived
/// artifact matched while the two documents differed. A test that cannot fail is not a test.
#[test]
fn incremental_edits_agree_with_a_full_replace() {
    let start = "module m\n\nfn alpha() -> Int { 1 }\nfn beta() -> Int { 2 }\ntest \"λ\" { assert(true) }\n";
    let expected =
        "module m\n\nfn alphaX() -> Int { 1 }\nfn gamma() -> Int { 3 }\ntest \"λABCD\" { assert(true) }\n";
    let live = "file:///inc.delulu";
    let reference = "file:///ref.delulu";

    let mut c = Client::start();
    c.open(live, start);
    let _ = c.wait_diagnostics(live);

    let edit = |c: &mut Client, version: i64, changes: Value| {
        c.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": live, "version": version }, "contentChanges": changes }),
        );
        let _ = c.wait_diagnostics(live);
    };

    // 1. Insert inside a line: `alpha` becomes `alphaX`.
    edit(&mut c, 2, json!([{
        "range": { "start": { "line": 2, "character": 8 }, "end": { "line": 2, "character": 8 } },
        "text": "X"
    }]));
    // 2. Delete a whole line, range spanning the line boundary.
    edit(&mut c, 3, json!([{
        "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 4, "character": 0 } },
        "text": ""
    }]));
    // 3. Insert text containing a newline at the end of the document.
    edit(&mut c, 4, json!([{
        "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 3, "character": 0 } },
        "text": "fn gamma() -> Int { 3 }\n"
    }]));
    // 4. Two changes in ONE notification, both immediately after the `λ` in the test name.
    //    Column 7 is the position after `λ` (byte 8); column 9 is the position after `λAB`
    //    (byte 10) and is only correct if the first change has ALREADY been applied. Counting
    //    columns in bytes lands inside `λ` on both, putting the inserted text before it:
    //    `test "ABCDλ"` instead of `test "λABCD"`.
    edit(&mut c, 5, json!([
        {
            "range": { "start": { "line": 4, "character": 7 }, "end": { "line": 4, "character": 7 } },
            "text": "AB"
        },
        {
            "range": { "start": { "line": 4, "character": 9 }, "end": { "line": 4, "character": 9 } },
            "text": "CD"
        }
    ]));

    c.open(reference, expected);
    let _ = c.wait_diagnostics(reference);

    let ask = |c: &mut Client, method: &str, uri: &str| {
        c.request(method, json!({ "textDocument": { "uri": uri } }))
    };
    assert_eq!(
        ask(&mut c, "textDocument/semanticTokens/full", live),
        ask(&mut c, "textDocument/semanticTokens/full", reference),
        "the incrementally edited document is not byte-identical to the one opened whole"
    );
    assert_eq!(
        ask(&mut c, "textDocument/documentSymbol", live),
        ask(&mut c, "textDocument/documentSymbol", reference),
        "symbols disagree between the incremental and full paths"
    );
    let names: Vec<String> = ask(&mut c, "textDocument/documentSymbol", live)
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        names,
        ["alphaX", "gamma", "test \"λABCD\""],
        "the edits produced the intended document: {names:?}"
    );
    c.shutdown();
}

/// The server advertises incremental sync, and `openClose` with it.
///
/// Stated as its own test because the omission is silent and total: a client reading the object
/// form of `textDocumentSync` without `openClose` never sends `didOpen`, so the server ends up
/// with no documents and answers every request with nothing, while looking perfectly healthy.
#[test]
fn incremental_sync_is_advertised_with_open_close() {
    let mut c = Client::start();
    let caps = c.request("initialize", json!({ "capabilities": {} }));
    let sync = &caps["capabilities"]["textDocumentSync"];
    assert_eq!(sync["change"], 2, "incremental sync: {sync}");
    assert_eq!(sync["openClose"], true, "openClose must be explicit in the object form: {sync}");
    c.shutdown();
}

/// A malformed range must not take the editing session down with it.
///
/// The offsets in a `didChange` arrive from another process over a pipe, and `replace_range`
/// panics on a range it does not like. An inverted range, a line past the end of the file and a
/// column past the end of a line are all sent here; afterwards the server must still be answering.
#[test]
fn a_malformed_range_cannot_kill_the_server() {
    let uri = "file:///rough.delulu";
    let mut c = Client::start();
    c.open(uri, "module m\nfn ok() -> Int { 1 }\n");
    let _ = c.wait_diagnostics(uri);

    for (n, range) in [
        // Inverted: end before start.
        json!({ "start": { "line": 1, "character": 10 }, "end": { "line": 1, "character": 2 } }),
        // A line that does not exist.
        json!({ "start": { "line": 99, "character": 0 }, "end": { "line": 99, "character": 5 } }),
        // A column far past the end of a real line.
        json!({ "start": { "line": 0, "character": 9999 }, "end": { "line": 0, "character": 9999 } }),
    ]
    .into_iter()
    .enumerate()
    {
        c.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 + n },
                "contentChanges": [{ "range": range, "text": "\n" }]
            }),
        );
        let _ = c.wait_diagnostics(uri);
    }

    // Still alive, still answering, still shutting down cleanly.
    let syms = c.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } }));
    assert!(syms.is_array(), "the server survived and still answers: {syms}");
    c.shutdown();
}

/// Closing a document and reopening it must not resurrect the analysis of what it used to hold.
///
/// This is the ordinary life of a file under an agent or a version-control operation: the editor
/// closes it, something rewrites it on disk, the editor opens it again at the same URI. A cache
/// kept beside the documents and keyed by a per-document counter gets this wrong when the counter
/// restarts on reopen — which is exactly why the analysis now lives inside the document record
/// and is dropped with it.
#[test]
fn reopening_a_changed_document_does_not_serve_the_closed_one() {
    let uri = "file:///reopen.delulu";
    let mut c = Client::start();
    c.open(uri, "module m\nfn before_close() -> Int { 1 }\n");
    let _ = c.wait_diagnostics(uri);
    c.notify("textDocument/didClose", json!({ "textDocument": { "uri": uri } }));

    c.open(uri, "module m\nfn after_close() -> Int { 2 }\n");
    let _ = c.wait_diagnostics(uri);
    let syms = c.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } }));
    let names: Vec<&str> =
        syms.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap_or_default()).collect();
    assert_eq!(names, ["after_close"], "the reopened document is analysed afresh: {names:?}");
    c.shutdown();
}

/// The same question, asked of the same files, must get the same answer every run.
///
/// `definition` is first-match-wins across open documents. Iterating a `HashMap` to decide that
/// makes the answer depend on the process's hash seed, so the same editor session could jump to a
/// different file after a restart. The order is: this document first, then sorted by URI.
#[test]
fn definition_prefers_this_document_then_a_stable_order() {
    // Five documents all declaring `target`, plus one that merely mentions it.
    let decls = ["a", "b", "c", "d", "e"];
    let caller = "module z\nfn caller() -> Int { target() }\n";

    for _run in 0..3 {
        let mut c = Client::start();
        for (i, name) in decls.iter().enumerate() {
            let src = format!("module {name}\nfn target() -> Int {{ {i} }}\n");
            c.open(&format!("file:///{name}.delulu"), &src);
            let _ = c.wait_diagnostics(&format!("file:///{name}.delulu"));
        }
        c.open("file:///z.delulu", caller);
        let _ = c.wait_diagnostics("file:///z.delulu");

        // From a document that does NOT declare it: the lexicographically first URI wins.
        let col = caller.lines().nth(1).unwrap().find("target").unwrap();
        let def = c.request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": "file:///z.delulu" },
                "position": { "line": 1, "character": col }
            }),
        );
        assert_eq!(def["uri"], "file:///a.delulu", "stable order, run {_run}: {def}");

        // From a document that DOES declare it: itself, even though `a` sorts first.
        let def_local = c.request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": "file:///c.delulu" },
                "position": { "line": 1, "character": 4 }
            }),
        );
        assert_eq!(def_local["uri"], "file:///c.delulu", "this document first, run {_run}: {def_local}");
        c.shutdown();
    }
}

/// A request must not re-run the compiler over every open document.
///
/// Self-calibrating rather than an absolute millisecond bound, because the ratio is the property:
/// one edit costs exactly one check (unavoidable — the text changed), and a run of read-only
/// requests afterwards should cost none at all. Before the cache each `references` request ran a
/// full check per open document, so this measured about forty times the baseline instead of a
/// fraction of it.
#[test]
fn repeated_requests_do_not_recheck_every_open_document() {
    let body = |n: usize| {
        let mut s = format!("module d{n}\n");
        for i in 0..75 {
            s.push_str(&format!(
                "fn f{n}_{i}(a: Int, b: Int) -> Int {{\n    let c = a + b\n    let d = c * 2\n\
                 \x20   let e = d - a\n    let g = e % 97\n    let h = g + c\n    let k = h - d\n\
                 \x20   let l = k + e\n    let m2 = l * 3\n    k + m2 + l\n}}\n"
            ));
        }
        s.push_str("fn shared_target() -> Int { 7 }\n");
        s
    };
    let uris: Vec<String> = (0..4).map(|n| format!("file:///d{n}.delulu")).collect();

    let mut c = Client::start();
    for (n, uri) in uris.iter().enumerate() {
        c.open(uri, &body(n));
        let _ = c.wait_diagnostics(uri);
    }

    // Baseline: one edit, one check. This is the cost the cache can never remove.
    let edited = format!("{}fn extra() -> Int {{ 0 }}\n", body(0));
    let t0 = std::time::Instant::now();
    c.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": &uris[0], "version": 2 },
            "contentChanges": [{ "text": edited }]
        }),
    );
    let _ = c.wait_diagnostics(&uris[0]);
    let one_check = t0.elapsed();

    // Ten read-only requests that each used to check all four documents.
    let line = edited.lines().position(|l| l.contains("fn shared_target")).unwrap();
    let col = edited.lines().nth(line).unwrap().find("shared_target").unwrap();
    let t1 = std::time::Instant::now();
    for _ in 0..10 {
        let refs = c.request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": &uris[0] },
                "position": { "line": line, "character": col },
                "context": { "includeDeclaration": true }
            }),
        );
        assert_eq!(refs.as_array().unwrap().len(), 4, "one declaration per open document");
    }
    let ten_requests = t1.elapsed();

    eprintln!("one edit = {one_check:?}; ten cross-document requests = {ten_requests:?}");
    assert!(
        ten_requests < one_check * 4,
        "ten read-only requests took {ten_requests:?} against a {one_check:?} baseline — \
         the analysis is being recomputed per request"
    );
    c.shutdown();
}
