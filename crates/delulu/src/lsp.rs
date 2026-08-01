//! `delulu lsp` — the language server (Stage 8, spec §3). Stdio, LSP 3.17, one instance
//! per workspace, ZERO new dependencies (build-order deviation 3): hand-rolled JSON-RPC
//! framing + value-level protocol over `serde_json`, fenced by a smoke suite that drives
//! the real binary over the real wire.
//!
//! **Analysis only, structurally:** this module never constructs an `Interp`, a broker
//! client, or a plugin host — it cannot run code, hold leases, or effect anything. Its
//! availability is not a security property (spec §11).
//!
//! **The diagnostics ARE the compiler's** (invariant 38): every publish re-runs the
//! exact `check_source` pipeline; codes, spans, messages, and repairs are the same
//! objects `delulu check --json` reports — the smoke suite asserts equality.

use std::collections::HashMap;
use std::io::{BufRead, Write};

use serde_json::{json, Value};

use delulu_check::check_source;
use delulu_diag::Diagnostic;

pub fn run_lsp(_args: &[String]) -> i32 {
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut server = Server { docs: HashMap::new(), shutdown_seen: false };
    loop {
        let Some(raw) = read_message(&mut reader) else {
            // Client hung up. Clean only if shutdown was requested first.
            return i32::from(!server.shutdown_seen);
        };
        let Ok(msg) = serde_json::from_str::<Value>(&raw) else { continue };
        let id = msg.get("id").cloned();
        match msg["method"].as_str() {
            Some("initialize") => respond(id, server.initialize()),
            Some("initialized") => {}
            Some("shutdown") => {
                server.shutdown_seen = true;
                respond(id, Value::Null);
            }
            Some("exit") => return i32::from(!server.shutdown_seen),
            Some("textDocument/didOpen") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["textDocument"]["text"].as_str().unwrap_or("").to_string();
                server.docs.insert(uri.clone(), text);
                server.publish(&uri);
            }
            Some("textDocument/didChange") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                // Full sync (declared in the capabilities): the last change carries the text.
                if let Some(text) = msg["params"]["contentChanges"]
                    .as_array()
                    .and_then(|a| a.last())
                    .and_then(|c| c["text"].as_str())
                {
                    server.docs.insert(uri.clone(), text.to_string());
                    server.publish(&uri);
                }
            }
            Some("textDocument/didClose") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
                server.docs.remove(uri);
            }
            Some("textDocument/hover") => {
                let r = server.hover(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/codeAction") => {
                let r = server.code_actions(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/documentSymbol") => {
                let r = server.document_symbols(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/inlayHint") => {
                let r = server.inlay_hints(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/definition") => {
                let r = server.definition(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/references") => {
                let r = server.references(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/rename") => match server.rename(&msg["params"]) {
                Ok(r) => respond(id, r),
                Err(why) => {
                    if let Some(id) = id {
                        respond_err(id, -32602, why);
                    }
                }
            },
            Some("textDocument/semanticTokens/full") => {
                let r = server.semantic_tokens(&msg["params"]);
                respond(id, r);
            }
            Some("textDocument/codeLens") => {
                let r = server.code_lens(&msg["params"]);
                respond(id, r);
            }
            Some("workspace/executeCommand") => {
                let r = server.execute_command(&msg["params"]);
                respond(id, r);
            }
            // Politely refuse anything unknown that expects an answer.
            _ => {
                if let Some(id) = id {
                    respond_err(id, -32601, "method not implemented by this server");
                }
            }
        }
    }
}

struct Server {
    docs: HashMap<String, String>,
    shutdown_seen: bool,
}

impl Server {
    fn initialize(&self) -> Value {
        json!({
            "capabilities": {
                "textDocumentSync": 1, // full
                "hoverProvider": true,
                "codeActionProvider": true,
                "documentSymbolProvider": true,
                "inlayHintProvider": true,
                "definitionProvider": true,
                "referencesProvider": true,
                "renameProvider": true,
                "codeLensProvider": { "resolveProvider": false },
                "semanticTokensProvider": {
                    "legend": { "tokenTypes": SEMANTIC_TOKEN_TYPES, "tokenModifiers": [] },
                    "full": true
                },
                "executeCommandProvider": { "commands": ["delulu.authority"] },
            },
            "serverInfo": { "name": "delulu-lsp", "version": env!("CARGO_PKG_VERSION") }
        })
    }

    /// Definition of the module-level name under the cursor, searched across every open
    /// document (build-order deviation 3: declaration + reference walk, not a DefId graph).
    fn definition(&self, params: &Value) -> Value {
        let Some((word, _, _)) = self.word_at(params) else { return Value::Null };
        for (uri, text) in &self.docs {
            let checked = check_source(0, text);
            if let Some(sp) = decl_name_span(&checked.module, &word) {
                return json!({ "uri": uri, "range": byte_range(text, sp.start, sp.end) });
            }
        }
        Value::Null
    }

    fn references(&self, params: &Value) -> Value {
        let Some((word, _, _)) = self.word_at(params) else { return json!([]) };
        let mut out = Vec::new();
        for (uri, text) in &self.docs {
            let checked = check_source(0, text);
            for sp in name_occurrences(&checked.module, &word) {
                out.push(json!({ "uri": uri, "range": byte_range(text, sp.start, sp.end) }));
            }
        }
        Value::Array(out)
    }

    /// Rename a MODULE-LEVEL name across every open document. Locals refuse honestly
    /// (deviation 3): a shadow-aware local rename needs a DefId graph the server does not have.
    fn rename(&self, params: &Value) -> Result<Value, &'static str> {
        let Some((word, _, _)) = self.word_at(params) else {
            return Err("nothing renameable at this position");
        };
        let new_name = params["newName"].as_str().unwrap_or("");
        if new_name.is_empty()
            || !new_name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            || !new_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || delulu_syntax::token::is_reserved(new_name)
        {
            return Err("the new name is not a legal identifier");
        }
        let is_decl = self
            .docs
            .values()
            .any(|text| decl_name_span(&check_source(0, text).module, &word).is_some());
        if !is_decl {
            return Err(
                "only module-level names (fn/type/effect/const/actor) can be renamed — a local \
                 rename needs shadow-aware resolution the server does not have, so it is refused \
                 rather than guessed",
            );
        }
        let mut changes = serde_json::Map::new();
        for (uri, text) in &self.docs {
            let checked = check_source(0, text);
            let edits: Vec<Value> = name_occurrences(&checked.module, &word)
                .into_iter()
                .map(|sp| {
                    json!({ "range": byte_range(text, sp.start, sp.end), "newText": new_name })
                })
                .collect();
            if !edits.is_empty() {
                changes.insert(uri.clone(), Value::Array(edits));
            }
        }
        Ok(json!({ "changes": changes }))
    }

    /// Full-document semantic tokens: keywords/strings/numbers/comments from the lexer,
    /// plus the SEMANTIC classes the spec names — effects, rcaps, capability types,
    /// secrets — from the AST (distinct kinds, criterion: spec §3 table).
    fn semantic_tokens(&self, params: &Value) -> Value {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let Some(text) = self.docs.get(uri) else { return json!({ "data": [] }) };
        let mut toks: Vec<(u32, u32, u32)> = Vec::new(); // (start_byte, end_byte, type)

        let (lexed, _d, comments) = delulu_syntax::lexer::lex_with_comments(0, text);
        for t in &lexed {
            use delulu_syntax::token::TokenKind as K;
            let ty = match &t.kind {
                K::Str(_) => Some(TOK_STRING),
                K::Int(_) | K::Float(_) => Some(TOK_NUMBER),
                k if k.keyword_lexeme().is_some() => Some(TOK_KEYWORD),
                _ => None,
            };
            if let Some(ty) = ty {
                toks.push((t.span.start, t.span.end, ty));
            }
        }
        for c in &comments {
            toks.push((c.start, c.start + c.text.len() as u32, TOK_COMMENT));
        }

        let checked = check_source(0, text);
        for item in &checked.module.items {
            use delulu_syntax::ast::Item;
            match item {
                Item::Fn(f) => {
                    toks.push((f.name.span.start, f.name.span.end, TOK_FUNCTION));
                    for p in &f.params {
                        classify_type(&p.ty, &mut toks);
                    }
                    if let Some(r) = &f.ret {
                        classify_type(r, &mut toks);
                    }
                    if let Some(r) = &f.row {
                        classify_row(r, &mut toks);
                    }
                }
                Item::Type(t) => toks.push((t.name.span.start, t.name.span.end, TOK_TYPE)),
                Item::Effect(e) => toks.push((e.name.span.start, e.name.span.end, TOK_EFFECT)),
                Item::Actor(a) => {
                    toks.push((a.name.span.start, a.name.span.end, TOK_TYPE));
                    for b in &a.behaviors {
                        toks.push((b.name.span.start, b.name.span.end, TOK_FUNCTION));
                        if let Some(r) = &b.row {
                            classify_row(r, &mut toks);
                        }
                    }
                }
                _ => {}
            }
        }

        // Delta-encode, position-sorted, overlaps dropped (first wins).
        toks.sort_by_key(|t| t.0);
        toks.dedup_by_key(|t| t.0);
        let mut data: Vec<u32> = Vec::with_capacity(toks.len() * 5);
        let (mut prev_line, mut prev_char) = (0u32, 0u32);
        for (s, e, ty) in toks {
            let p = byte_to_pos(text, s);
            let (line, ch) = (p["line"].as_u64().unwrap() as u32, p["character"].as_u64().unwrap() as u32);
            let len: u32 = text
                .get(s as usize..(e as usize).min(text.len()))
                .map(|t| t.chars().map(char::len_utf16).sum::<usize>() as u32)
                .unwrap_or(0);
            let dl = line - prev_line;
            let dc = if dl == 0 { ch - prev_char } else { ch };
            data.extend([dl, dc, len, ty, 0]);
            prev_line = line;
            prev_char = ch;
        }
        json!({ "data": data })
    }

    /// Code lenses on `fn main` and each `test` (spec §3): `▶ run` + the authority line.
    fn code_lens(&self, params: &Value) -> Value {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let Some(text) = self.docs.get(uri) else { return json!([]) };
        let checked = check_source(0, text);
        let mut lenses = Vec::new();
        for item in &checked.module.items {
            use delulu_syntax::ast::Item;
            match item {
                Item::Fn(f) if f.name.name == "main" => {
                    let range = byte_range(text, f.name.span.start, f.name.span.end);
                    lenses.push(json!({
                        "range": range,
                        "command": { "title": "▶ run", "command": "delulu.run", "arguments": [uri] }
                    }));
                    let authority = checked
                        .result
                        .facts
                        .get("main")
                        .map(|fa| {
                            let mut es: Vec<&str> = fa.effects.iter().map(|e| e.name()).collect();
                            es.sort();
                            if es.is_empty() { "pure".to_string() } else { format!("{{{}}}", es.join(", ")) }
                        })
                        .unwrap_or_default();
                    lenses.push(json!({
                        "range": byte_range(text, f.name.span.start, f.name.span.end),
                        "command": {
                            "title": format!("authority: {authority}"),
                            "command": "delulu.authority",
                            "arguments": [uri]
                        }
                    }));
                }
                Item::Test(t) => {
                    let range = byte_range(text, t.name_span.start, t.name_span.end);
                    lenses.push(json!({
                        "range": range,
                        "command": {
                            "title": "▶ run test",
                            "command": "delulu.test.run",
                            "arguments": [uri, t.name]
                        }
                    }));
                }
                _ => {}
            }
        }
        Value::Array(lenses)
    }

    /// `delulu.authority` (spec §3): the §10.5 report for an open document — agent
    /// harnesses call this instead of shelling out. Analysis of the document alone: no
    /// manifest scopes attach to a bare URI (the CLI's report is the one with custody
    /// and manifest stamps).
    fn execute_command(&self, params: &Value) -> Value {
        if params["command"].as_str() != Some("delulu.authority") {
            return Value::Null;
        }
        let Some(uri) = params["arguments"][0].as_str() else { return Value::Null };
        let Some(text) = self.docs.get(uri) else { return Value::Null };
        let checked = check_source(0, text);
        if checked.has_errors() {
            return json!({ "error": "the document has check errors — fix them first" });
        }
        delulu_check::authority_report(
            &checked.module.name.dotted(),
            &checked.result,
            &delulu_check::ScopeInfo::default(),
        )
    }

    /// The identifier word at the request's position: `(word, start, end)` bytes.
    fn word_at(&self, params: &Value) -> Option<(String, u32, u32)> {
        let uri = params["textDocument"]["uri"].as_str()?;
        let text = self.docs.get(uri)?;
        let pos = &params["position"];
        let byte =
            pos_to_byte(text, pos["line"].as_u64()?, pos["character"].as_u64()?) as usize;
        let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
        let b = text.as_bytes();
        if byte >= b.len() || !is_word(b[byte]) {
            return None;
        }
        let mut s = byte;
        while s > 0 && is_word(b[s - 1]) {
            s -= 1;
        }
        let mut e = byte;
        while e < b.len() && is_word(b[e]) {
            e += 1;
        }
        Some((text[s..e].to_string(), s as u32, e as u32))
    }

    /// Re-check the document and push its diagnostics — the compiler's own, verbatim.
    fn publish(&self, uri: &str) {
        let Some(text) = self.docs.get(uri) else { return };
        let checked = check_source(0, text);
        let diags: Vec<Value> =
            checked.diagnostics.iter().map(|d| lsp_diagnostic(text, d)).collect();
        notify(
            "textDocument/publishDiagnostics",
            json!({ "uri": uri, "diagnostics": diags }),
        );
    }

    /// Hover: a fn declaration name shows its full signature + transitive authority; any
    /// typed expression shows its settled type.
    fn hover(&self, params: &Value) -> Value {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let Some(text) = self.docs.get(uri) else { return Value::Null };
        let pos = &params["position"];
        let byte = pos_to_byte(text, pos["line"].as_u64().unwrap_or(0), pos["character"].as_u64().unwrap_or(0));
        let checked = check_source(0, text);

        // Function declaration names first: signature + authority.
        for item in &checked.module.items {
            if let delulu_syntax::ast::Item::Fn(f) = item {
                if f.name.span.start <= byte && byte < f.name.span.end.max(f.name.span.start + 1) {
                    let sig = checked
                        .result
                        .fn_types
                        .get(&f.name.name)
                        .map(|t| t.show(&checked.table).to_string())
                        .unwrap_or_else(|| "fn".to_string());
                    let authority = checked
                        .result
                        .facts
                        .get(&f.name.name)
                        .map(|fa| {
                            let mut es: Vec<&str> = fa.effects.iter().map(|e| e.name()).collect();
                            es.sort();
                            if es.is_empty() { "pure".to_string() } else { format!("{{{}}}", es.join(", ")) }
                        })
                        .unwrap_or_default();
                    let md = format!("```delulu\nfn {}: {}\n```\nauthority: {}", f.name.name, sig, authority);
                    return json!({
                        "contents": { "kind": "markdown", "value": md },
                        "range": byte_range(text, f.name.span.start, f.name.span.end),
                    });
                }
            }
        }

        // Otherwise: the smallest typed expression containing the position.
        let mut best: Option<(u32, u32, String)> = None;
        walk_exprs(&checked.module, &mut |e| {
            let sp = e.span();
            if sp.start <= byte && byte < sp.end {
                if let Some(ty) = checked.result.node_types.get(&e.id()) {
                    let width = sp.end - sp.start;
                    if best.as_ref().is_none_or(|(s, e2, _)| (e2 - s) > width) {
                        best = Some((sp.start, sp.end, ty.show(&checked.table).to_string()));
                    }
                }
            }
        });
        match best {
            Some((s, e, ty)) => json!({
                "contents": { "kind": "markdown", "value": format!("```delulu\n{ty}\n```") },
                "range": byte_range(text, s, e),
            }),
            None => Value::Null,
        }
    }

    /// Every typed repair becomes a code action. Authority-widening repairs are ⚠-titled
    /// and `isPreferred: false` with the flag surfaced in `data` (criterion 2);
    /// `requires_human` repairs are documentation-only (no edit to apply).
    fn code_actions(&self, params: &Value) -> Value {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let Some(text) = self.docs.get(uri) else { return json!([]) };
        let range = &params["range"];
        let lo = pos_to_byte(text, range["start"]["line"].as_u64().unwrap_or(0), range["start"]["character"].as_u64().unwrap_or(0));
        let hi = pos_to_byte(text, range["end"]["line"].as_u64().unwrap_or(0), range["end"]["character"].as_u64().unwrap_or(0));
        let checked = check_source(0, text);
        let mut actions = Vec::new();
        for d in &checked.diagnostics {
            let overlaps = d
                .primary_span()
                .map(|sp| sp.start <= hi && lo <= sp.end)
                .unwrap_or(true);
            if !overlaps {
                continue;
            }
            for r in &d.repairs {
                let widening = r.authority_widening;
                let title = if widening {
                    format!("⚠ {} (widens authority — review)", r.id)
                } else if r.requires_human {
                    format!("{} (requires a human decision — no automatic edit)", r.id)
                } else {
                    r.id.to_string()
                };
                let mut action = json!({
                    "title": title,
                    "kind": "quickfix",
                    "isPreferred": !widening && !r.requires_human,
                    "diagnostics": [lsp_diagnostic(text, d)],
                    "data": {
                        "code": d.code,
                        "repair_id": r.id,
                        "authority_widening": widening,
                        "requires_human": r.requires_human,
                    },
                });
                if !r.requires_human && !r.edits.is_empty() {
                    let edits: Vec<Value> = r
                        .edits
                        .iter()
                        .map(|e| {
                            json!({
                                "range": byte_range(text, e.start_byte, e.end_byte),
                                "newText": e.insert,
                            })
                        })
                        .collect();
                    action["edit"] = json!({ "changes": { uri: edits } });
                }
                actions.push(action);
            }
        }
        Value::Array(actions)
    }

    /// Items (incl. actors, behaviors, tests) as document symbols.
    fn document_symbols(&self, params: &Value) -> Value {
        use delulu_syntax::ast::Item;
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let Some(text) = self.docs.get(uri) else { return json!([]) };
        let checked = check_source(0, text);
        let sym = |name: &str, kind: u32, s: u32, e: u32, text: &str| {
            json!({
                "name": name,
                "kind": kind,
                "range": byte_range(text, s, e),
                "selectionRange": byte_range(text, s, e),
            })
        };
        let mut out = Vec::new();
        for item in &checked.module.items {
            match item {
                Item::Fn(f) => out.push(sym(&f.name.name, 12, f.span.start, f.span.end, text)),
                Item::Type(t) => out.push(sym(&t.name.name, 5, t.span.start, t.span.end, text)),
                Item::Effect(e) => out.push(sym(&e.name.name, 11, e.span.start, e.span.end, text)),
                Item::Const(c) => out.push(sym(&c.name.name, 14, c.span.start, c.span.end, text)),
                Item::Foreign(fd) => out.push(sym(&fd.name.name, 2, fd.span.start, fd.span.end, text)),
                Item::Actor(a) => {
                    let mut actor = sym(&a.name.name, 5, a.span.start, a.span.end, text);
                    let mut children = Vec::new();
                    for b in &a.behaviors {
                        children.push(sym(&b.name.name, 6, b.span.start, b.span.end, text));
                    }
                    for f in &a.fns {
                        children.push(sym(&f.name.name, 6, f.span.start, f.span.end, text));
                    }
                    actor["children"] = Value::Array(children);
                    out.push(actor);
                }
                Item::Test(t) => {
                    out.push(sym(&format!("test \"{}\"", t.name), 12, t.span.start, t.span.end, text))
                }
            }
        }
        Value::Array(out)
    }

    /// The authority lens (criterion 2): inferred rows on unannotated lambdas.
    fn inlay_hints(&self, params: &Value) -> Value {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let Some(text) = self.docs.get(uri) else { return json!([]) };
        let checked = check_source(0, text);
        let mut hints = Vec::new();
        walk_exprs(&checked.module, &mut |e| {
            if let delulu_syntax::ast::Expr::Lambda { row: None, body, .. } = e {
                // The lambda EXPRESSION is pure (making a closure does nothing); the
                // inferred row the lens shows is the FUNCTION'S — inside its type.
                if let Some(delulu_check::Type::Fn { row, .. }) =
                    checked.result.node_types.get(&e.id())
                {
                    let mut es: Vec<String> =
                        row.effects.iter().map(|x| x.name().to_string()).collect();
                    es.sort();
                    let label = if es.is_empty() {
                        "! {}".to_string()
                    } else {
                        format!("! {{{}}}", es.join(", "))
                    };
                    hints.push(json!({
                        "position": byte_to_pos(text, body.span.start),
                        "label": label,
                        "kind": 1,
                        "paddingRight": true,
                        "data": { "authority_lens": true },
                    }));
                }
            }
        });
        Value::Array(hints)
    }
}

// ===== wire helpers ========================================================

fn read_message(reader: &mut impl BufRead) -> Option<String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None; // EOF
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            content_length = v.trim().parse().ok();
        }
    }
    let n = content_length?;
    let mut buf = vec![0u8; n];
    reader.read_exact(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

fn send(payload: Value) {
    let body = payload.to_string();
    let mut out = std::io::stdout().lock();
    let _ = write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = out.flush();
}

fn respond(id: Option<Value>, result: Value) {
    let Some(id) = id else { return };
    send(json!({ "jsonrpc": "2.0", "id": id, "result": result }));
}

fn respond_err(id: Value, code: i64, message: &str) {
    send(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }));
}

fn notify(method: &str, params: Value) {
    send(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
}

// ===== positions (UTF-16, the LSP default — computed, not approximated) =====

fn byte_to_pos(text: &str, byte: u32) -> Value {
    let byte = (byte as usize).min(text.len());
    let mut line = 0usize;
    let mut line_start = 0usize;
    for (i, b) in text.bytes().enumerate().take(byte) {
        if b == b'\n' {
            line += 1;
            line_start = i + 1;
        }
    }
    let character: usize = text[line_start..byte].chars().map(char::len_utf16).sum();
    json!({ "line": line, "character": character })
}

fn byte_range(text: &str, start: u32, end: u32) -> Value {
    json!({ "start": byte_to_pos(text, start), "end": byte_to_pos(text, end) })
}

fn pos_to_byte(text: &str, line: u64, character: u64) -> u32 {
    let mut cur_line = 0u64;
    let mut offset = 0usize;
    if line > 0 {
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                cur_line += 1;
                if cur_line == line {
                    offset = i + 1;
                    break;
                }
            }
        }
        if cur_line < line {
            return text.len() as u32;
        }
    }
    let mut units = 0u64;
    for c in text[offset..].chars() {
        if units >= character || c == '\n' {
            break;
        }
        units += c.len_utf16() as u64;
        offset += c.len_utf8();
    }
    offset as u32
}

// ===== compiler-facing helpers =============================================

/// The compiler's diagnostic, in LSP clothes: same code, same primary span, same message
/// (en-US — the machine surface is locale-invariant; catalogs are terminal-human only).
fn lsp_diagnostic(text: &str, d: &Diagnostic) -> Value {
    let (start, end) = d
        .primary_span()
        .map(|s| (s.start, s.end))
        .unwrap_or((0, 0));
    json!({
        "range": byte_range(text, start, end),
        "severity": if d.is_error() { 1 } else { 2 },
        "code": d.code,
        "source": "delulu",
        "message": d.message,
    })
}

// ===== semantic-token legend and classifiers ===============================

/// The legend (spec §3): the last four kinds are the ones the spec NAMES as distinct —
/// effects, rcaps, capability types, secrets.
const SEMANTIC_TOKEN_TYPES: [&str; 12] = [
    "keyword", "function", "type", "variable", "string", "number", "comment", "operator",
    "effect", "rcap", "capability", "secret",
];
const TOK_KEYWORD: u32 = 0;
const TOK_FUNCTION: u32 = 1;
const TOK_TYPE: u32 = 2;
const TOK_STRING: u32 = 4;
const TOK_NUMBER: u32 = 5;
const TOK_COMMENT: u32 = 6;
const TOK_EFFECT: u32 = 8;
const TOK_RCAP: u32 = 9;
const TOK_CAPABILITY: u32 = 10;
const TOK_SECRET: u32 = 11;

fn classify_row(r: &delulu_syntax::ast::RowExpr, toks: &mut Vec<(u32, u32, u32)>) {
    for p in &r.effects {
        let sp = p.span();
        toks.push((sp.start, sp.end, TOK_EFFECT));
    }
}

fn classify_type(t: &delulu_syntax::ast::TypeExpr, toks: &mut Vec<(u32, u32, u32)>) {
    use delulu_syntax::ast::TypeExpr;
    match t {
        TypeExpr::Named { path, args, span } => {
            let head = path.segs.last().map(|s| s.name.as_str()).unwrap_or("");
            let kind = match head {
                "Cap" | "Root" | "Plugin" => TOK_CAPABILITY,
                "Secret" => TOK_SECRET,
                _ => TOK_TYPE,
            };
            let sp = path.span();
            toks.push((sp.start, sp.end, kind));
            let _ = span;
            for a in args {
                classify_type(a, toks);
            }
        }
        TypeExpr::Fn { params, ret, row, .. } => {
            for p in params {
                classify_type(p, toks);
            }
            if let Some(r) = ret {
                classify_type(r, toks);
            }
            if let Some(r) = row {
                classify_row(r, toks);
            }
        }
        TypeExpr::Rcap { rcap, inner, span } => {
            let len = rcap.name().len() as u32;
            toks.push((span.start, span.start + len, TOK_RCAP));
            classify_type(inner, toks);
        }
    }
}

// ===== name resolution (deviation 3: declaration + reference walk) =========

/// The name span of a module-level declaration named `word`, if this module declares it.
fn decl_name_span(
    module: &delulu_syntax::ast::Module,
    word: &str,
) -> Option<delulu_diag::Span> {
    use delulu_syntax::ast::Item;
    module.items.iter().find_map(|item| match item {
        Item::Fn(f) if f.name.name == word => Some(f.name.span),
        Item::Type(t) if t.name.name == word => Some(t.name.span),
        Item::Effect(e) if e.name.name == word => Some(e.name.span),
        Item::Const(c) if c.name.name == word => Some(c.name.span),
        Item::Actor(a) if a.name.name == word => Some(a.name.span),
        _ => None,
    })
}

/// Every occurrence of the module-level name `word` in this module: the declaration plus
/// value references (bare or as a qualified path's final segment), type references, row
/// effect references, and spawn targets. Local binders are deliberately NOT visited.
fn name_occurrences(
    module: &delulu_syntax::ast::Module,
    word: &str,
) -> Vec<delulu_diag::Span> {
    use delulu_syntax::ast::{Item, TypeExpr};
    let mut out = Vec::new();
    if let Some(sp) = decl_name_span(module, word) {
        out.push(sp);
    }
    let path_hit = |p: &delulu_syntax::ast::Path, out: &mut Vec<delulu_diag::Span>| {
        if let Some(last) = p.segs.last() {
            if last.name == word {
                out.push(last.span);
            }
        }
    };
    fn type_paths(
        t: &TypeExpr,
        word: &str,
        out: &mut Vec<delulu_diag::Span>,
    ) {
        match t {
            TypeExpr::Named { path, args, .. } => {
                if let Some(last) = path.segs.last() {
                    if last.name == word {
                        out.push(last.span);
                    }
                }
                for a in args {
                    type_paths(a, word, out);
                }
            }
            TypeExpr::Fn { params, ret, row, .. } => {
                for p in params {
                    type_paths(p, word, out);
                }
                if let Some(r) = ret {
                    type_paths(r, word, out);
                }
                if let Some(r) = row {
                    for e in &r.effects {
                        if let Some(last) = e.segs.last() {
                            if last.name == word {
                                out.push(last.span);
                            }
                        }
                    }
                }
            }
            TypeExpr::Rcap { inner, .. } => type_paths(inner, word, out),
        }
    }
    for item in &module.items {
        match item {
            Item::Fn(f) => {
                for p in &f.params {
                    type_paths(&p.ty, word, &mut out);
                }
                if let Some(r) = &f.ret {
                    type_paths(r, word, &mut out);
                }
                if let Some(r) = &f.row {
                    for e in &r.effects {
                        path_hit(e, &mut out);
                    }
                }
            }
            Item::Const(c) => {
                if let Some(t) = &c.ty {
                    type_paths(t, word, &mut out);
                }
            }
            _ => {}
        }
    }
    walk_exprs(module, &mut |e| {
        use delulu_syntax::ast::Expr;
        match e {
            Expr::Var { path, .. } | Expr::Record { path, .. } | Expr::Spawn { actor: path, .. } => {
                path_hit(path, &mut out)
            }
            // Dotted chains are Method/Field Go-selector style (`lib.greet(x)` is a
            // Method with recv `lib`), so a qualified reference to a module-level name
            // lands HERE — the deviation-3 walk's approximation includes same-named
            // record methods, recorded openly.
            Expr::Method { name, .. } | Expr::Field { name, .. } if name.name == word => {
                out.push(name.span)
            }
            _ => {}
        }
    });
    out.sort_by_key(|s| s.start);
    out.dedup_by_key(|s| s.start);
    out
}

/// Walk every expression in the module (fn/const/actor/test bodies), depth-first.
fn walk_exprs(module: &delulu_syntax::ast::Module, f: &mut impl FnMut(&delulu_syntax::ast::Expr)) {
    use delulu_syntax::ast::{Item, Stmt};
    fn block(b: &delulu_syntax::ast::Block, f: &mut impl FnMut(&delulu_syntax::ast::Expr)) {
        for s in &b.stmts {
            match s {
                Stmt::Let { value, .. } | Stmt::Assign { value, .. } => expr(value, f),
                Stmt::While { cond, body, .. } => {
                    expr(cond, f);
                    block(body, f);
                }
                Stmt::Return { value: Some(v), .. } => expr(v, f),
                Stmt::Return { value: None, .. } => {}
                Stmt::Expr(e) => expr(e, f),
            }
        }
    }
    fn expr(e: &delulu_syntax::ast::Expr, f: &mut impl FnMut(&delulu_syntax::ast::Expr)) {
        use delulu_syntax::ast::Expr::*;
        f(e);
        match e {
            List { items, .. } => items.iter().for_each(|x| expr(x, f)),
            Record { fields, .. } => fields.iter().for_each(|(_, v)| expr(v, f)),
            Call { callee, args, .. } => {
                expr(callee, f);
                args.iter().for_each(|x| expr(x, f));
            }
            Method { recv, args, .. } => {
                expr(recv, f);
                args.iter().for_each(|x| expr(x, f));
            }
            Field { recv, .. } => expr(recv, f),
            Index { recv, index, .. } => {
                expr(recv, f);
                expr(index, f);
            }
            Unary { operand, .. } => expr(operand, f),
            Binary { lhs, rhs, .. } => {
                expr(lhs, f);
                expr(rhs, f);
            }
            If { cond, then_, else_, .. } => {
                expr(cond, f);
                block(then_, f);
                if let Some(e2) = else_ {
                    expr(e2, f);
                }
            }
            Match { scrutinee, arms, .. } => {
                expr(scrutinee, f);
                arms.iter().for_each(|a| expr(&a.body, f));
            }
            Lambda { body, .. } | Recover { body, .. } => block(body, f),
            Try { inner, .. } => expr(inner, f),
            Spawn { args, .. } => args.iter().for_each(|x| expr(x, f)),
            Block(b) => block(b, f),
            Lit { .. } | Var { .. } | Consume { .. } => {}
        }
    }
    for item in &module.items {
        match item {
            Item::Fn(fd) => block(&fd.body, f),
            Item::Const(c) => expr(&c.value, f),
            Item::Actor(a) => {
                block(&a.ctor.body, f);
                a.behaviors.iter().for_each(|b| block(&b.body, f));
                a.fns.iter().for_each(|fd| block(&fd.body, f));
            }
            Item::Test(t) => block(&t.body, f),
            _ => {}
        }
    }
}
