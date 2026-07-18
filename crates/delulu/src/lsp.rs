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
            // Politely refuse anything unknown that expects an answer.
            _ => {
                if let Some(id) = id {
                    respond_err(id, -32601, "method not implemented in v0.8");
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
            },
            "serverInfo": { "name": "delulu-lsp", "version": env!("CARGO_PKG_VERSION") }
        })
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
                        .map(|t| format!("{t}"))
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
                        best = Some((sp.start, sp.end, format!("{ty}")));
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
