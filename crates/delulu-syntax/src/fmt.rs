//! `delulu fmt` — the canonical formatter (Stage 8, spec §4). One style, zero options.
//!
//! **The two laws (both compiler-bug class, DL1702, if violated):**
//! - *Identity:* `parse(fmt(src)) ≡ parse(src)` — AST-equal with spans/ids ignored and the
//!   two semantic SETS (import lists, row effect lists) order-normalized, plus the full
//!   comment sequence `(text, own_line)` preserved. See [`ast_fingerprint`].
//! - *Idempotence:* `fmt(fmt(src)) == fmt(src)` byte-equal.
//!
//! **Style (normative choices, spec §4):** 4-space indent; 100-column soft limit (call
//! arguments, list items, record fields, and parameter lists break with trailing commas
//! when a line would overflow; everything else may run long — the limit is soft); one
//! blank line between items; rows `! {Read, Write}` with a space after `!` and effects
//! alphabetical (`! e` bare tails, `! {Read | e}` mixed); imports sorted `std.*` first
//! then everything else alphabetically (package deps vs. local modules are
//! indistinguishable inside a single file — recorded in the build order).
//!
//! **Comments:** an own-line comment stays an own-line comment at the enclosing indent; a
//! trailing comment re-attaches to the line just printed. Comment TEXT is preserved
//! byte-for-byte, order preserved.
//!
//! **Refusal:** unparseable input is never "formatted" — the caller gets the diagnostics
//! instead (the kitchen rule: garbage in, refusal out, corruption never).

use std::collections::VecDeque;

use delulu_diag::{Diagnostic, FileId};

use crate::ast::*;
use crate::lexer::{lex_with_comments, Comment};

const WIDTH: usize = 100;
const INDENT: &str = "    ";

/// Format one source file. `Err` carries the lex/parse diagnostics of input the formatter
/// refuses to touch.
pub fn format_source(file: FileId, src: &str) -> Result<String, Vec<Diagnostic>> {
    let (tokens, mut diags, comments) = lex_with_comments(file, src);
    let tokens_of_ints: Vec<delulu_diag::Span> =
        tokens.iter().filter(|t| matches!(t.kind, crate::token::TokenKind::Int(_))).map(|t| t.span).collect();
    let (module, pdiags) = crate::parser::parse(file, tokens);
    diags.extend(pdiags);
    if diags.iter().any(|d| d.is_error()) {
        return Err(diags);
    }
    let _spellings = IntSpellings::enter(src, &tokens_of_ints);
    Ok(Printer::new(src, comments).module(&module))
}

thread_local! {
    /// FMT-INT-BASE-1: the spelling of each `Int` literal of the source being formatted that is not plain decimal
    /// digits, by the offset it starts at. The AST holds a literal's VALUE (`LitKind::Int(i64)`, and DIR's bytes with
    /// it), so the base it was written in reaches the printer beside the tree, as comments do — and by a literal's
    /// span, since the flat printers below are free functions that take a node and nothing else. Set for the length
    /// of one `format_source` call on the thread that makes it; empty at any other time, when an `Int` prints as its
    /// decimal digits.
    static INT_SPELLINGS: std::cell::RefCell<std::collections::HashMap<u32, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Holds [`INT_SPELLINGS`] for one `format_source` call, and puts back what was there when it ends.
struct IntSpellings(std::collections::HashMap<u32, String>);

impl IntSpellings {
    /// `ints` are the spans of the source's `Int` tokens. One written in plain decimal digits is left out: its
    /// value's digits are its spelling. Any other is kept as written — its digits and their `_` grouping — with the
    /// base's letter in lower case (`0XFF` is `0xFF`).
    fn enter(src: &str, ints: &[delulu_diag::Span]) -> IntSpellings {
        let mut map = std::collections::HashMap::new();
        for span in ints {
            let Some(text) = src.get(span.start as usize..span.end as usize) else { continue };
            if text.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let mut spelled = text.to_string();
            if let Some(base) = spelled.get(1..2).filter(|b| matches!(*b, "X" | "O" | "B")).map(str::to_ascii_lowercase) {
                spelled.replace_range(1..2, &base);
            }
            map.insert(span.start, spelled);
        }
        IntSpellings(INT_SPELLINGS.with(|m| m.replace(map)))
    }
}

impl Drop for IntSpellings {
    fn drop(&mut self) {
        INT_SPELLINGS.with(|m| *m.borrow_mut() = std::mem::take(&mut self.0));
    }
}

/// The spelling the source gave the `Int` literal at `span`, when it was not plain decimal digits.
fn int_spelling(span: delulu_diag::Span) -> Option<String> {
    INT_SPELLINGS.with(|m| m.borrow().get(&span.start).cloned())
}

/// The identity-law projection: the AST serialized with every `span`/`id` field stripped,
/// import order and row-effect order normalized (both are semantic sets — the formatter
/// sorts them, so the projection must too). Two sources are formatter-equivalent iff
/// their fingerprints match.
pub fn ast_fingerprint(module: &Module) -> String {
    let mut v = serde_json::to_value(module).expect("the AST serializes");
    strip_positions(&mut v);
    normalize_sets(&mut v, "");
    v.to_string()
}

/// The comment-attachment law's projection: every comment's `(text, own_line)` in order.
pub fn comment_sequence(src: &str) -> Vec<(String, bool)> {
    let (_t, _d, comments) = lex_with_comments(0, src);
    comments.into_iter().map(|c| (c.text, c.own_line)).collect()
}

fn strip_positions(v: &mut serde_json::Value) {
    // Spans ride in two shapes: named fields (`span`, `id`, …) and TUPLE-variant payloads
    // (`Pattern::Wildcard(Span)` serializes the span as a bare `{file,start,end}` object).
    // No AST struct has exactly those three fields, so the shape is unambiguous.
    let is_span_object = |v: &serde_json::Value| {
        v.as_object().is_some_and(|m| m.len() == 3 && m.contains_key("file") && m.contains_key("start") && m.contains_key("end"))
    };
    match v {
        serde_json::Value::Object(map) => {
            map.retain(|k, _| !matches!(k.as_str(), "span" | "id" | "name_span" | "abi_span"));
            for (_, child) in map.iter_mut() {
                if is_span_object(child) {
                    *child = serde_json::Value::Null;
                } else {
                    strip_positions(child);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for it in items {
                if is_span_object(it) {
                    *it = serde_json::Value::Null;
                } else {
                    strip_positions(it);
                }
            }
        }
        _ => {}
    }
}

fn normalize_sets(v: &mut serde_json::Value, key: &str) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, child) in map.iter_mut() {
                normalize_sets(child, k);
            }
        }
        serde_json::Value::Array(items) => {
            for it in items.iter_mut() {
                normalize_sets(it, "");
            }
            // `imports` on Module; `effects` on RowExpr — both semantic sets.
            if key == "imports" || key == "effects" {
                items.sort_by_key(|it| it.to_string());
            }
        }
        _ => {}
    }
}

// ===== the printer =========================================================

struct Printer<'a> {
    line_starts: Vec<u32>,
    comments: VecDeque<Comment>,
    out: String,
    indent: usize,
    src_len: u32,
    _src: &'a str,
}

impl<'a> Printer<'a> {
    fn new(src: &'a str, comments: Vec<Comment>) -> Printer<'a> {
        let mut line_starts = vec![0u32];
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        Printer { line_starts, comments: comments.into(), out: String::new(), indent: 0, src_len: src.len() as u32, _src: src }
    }

    fn line_of(&self, pos: u32) -> usize {
        self.line_starts.partition_point(|&s| s <= pos)
    }

    fn pad(&mut self) {
        for _ in 0..self.indent {
            self.out.push_str(INDENT);
        }
    }

    fn line(&mut self, text: &str) {
        self.pad();
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn blank(&mut self) {
        if !self.out.is_empty() && !self.out.ends_with("\n\n") {
            self.out.push('\n');
        }
    }

    /// Flush comments positioned before `pos`. Own-line comments print at the current
    /// indent; trailing comments re-attach to the previous printed line (there is always
    /// one — the module header at minimum), which keeps their `own_line = false` identity.
    fn flush_comments_before(&mut self, pos: u32) {
        // The source line just after the previously emitted own-line comment, so a blank line the
        // author put BETWEEN two comment paragraphs survives (`HARDENING_CAMPAIGN.md` C15).
        //
        // Without this the formatter emitted consecutive own-line comments back to back, merging
        // two deliberately separated paragraphs into one block. The identity law did not catch it,
        // and could not: its projection is each comment's `(text, own_line)` in order, and both are
        // unchanged by the merge. Only the *spacing between* comments was lost — which is exactly
        // the part carrying the author's meaning, and the part no law was watching.
        let mut prev_end_line: Option<usize> = None;
        while self.comments.front().is_some_and(|c| c.start < pos) {
            let c = self.comments.pop_front().expect("front checked");
            if c.own_line {
                let start_line = self.line_of(c.start);
                // One or more blank source lines between the last comment and this one → keep one.
                // Collapsing runs of blank lines to a single one is ordinary canonical formatting;
                // deleting the separation entirely is not.
                if prev_end_line.is_some_and(|p| start_line > p + 1) {
                    self.blank();
                }
                for l in c.text.split('\n') {
                    self.line(l);
                }
                // A block comment spans its own newlines, so the end line is derived from the text
                // rather than assumed to be the start line.
                prev_end_line = Some(start_line + c.text.matches('\n').count());
            } else {
                self.attach_trailing(&c.text);
                // A trailing comment does not participate in paragraph spacing: it belongs to the
                // line above it, so it must not make the next own-line comment look separated.
                prev_end_line = None;
            }
        }
    }

    /// Attach a trailing comment to the just-printed line.
    fn attach_trailing(&mut self, text: &str) {
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        if self.out.is_empty() {
            self.out.push_str(text);
        } else {
            self.out.push_str("  ");
            self.out.push_str(text);
        }
        self.out.push('\n');
    }

    /// After printing a node that ends at source `end`, attach any comment sitting on the
    /// same source line (a trailing comment travels with its line).
    fn attach_same_line(&mut self, end: u32) {
        let line = self.line_of(end);
        while self.comments.front().is_some_and(|c| !c.own_line && c.start >= end && self.line_of(c.start) == line) {
            let c = self.comments.pop_front().expect("front checked");
            self.attach_trailing(&c.text);
        }
    }

    // ----- module ----------------------------------------------------------

    fn module(mut self, m: &Module) -> String {
        // File-header comments: everything positioned before the module NAME (the
        // `module` keyword sits immediately before it; comments between the header line
        // and the first item flush with that item).
        self.flush_comments_before(m.name.span().start);
        for a in &m.attrs {
            let t = attr_text(a);
            self.line(&t);
        }
        let header = format!("module {}", m.name.dotted());
        self.line(&header);
        self.attach_same_line(m.name.span().end);

        // Imports: std.* first, then the rest, each group alphabetical (spec §4; the
        // deps-vs-local distinction needs a manifest a single file does not carry).
        if !m.imports.is_empty() {
            self.blank();
            let mut imports: Vec<&Import> = m.imports.iter().collect();
            imports.sort_by_key(|i| {
                let dotted = i.path.dotted();
                (!dotted.starts_with("std"), dotted, i.alias.as_ref().map(|a| a.name.clone()))
            });
            for imp in imports {
                let mut s = String::new();
                if imp.public {
                    s.push_str("pub ");
                }
                s.push_str("import ");
                s.push_str(&imp.path.dotted());
                if let Some(a) = &imp.alias {
                    s.push_str(" as ");
                    s.push_str(&a.name);
                }
                self.line(&s);
            }
        }

        for item in &m.items {
            self.blank();
            self.flush_comments_before(item_span(item).start);
            self.item(item);
        }
        self.flush_comments_before(self.src_len + 1);
        // Exactly one trailing newline.
        while self.out.ends_with("\n\n") {
            self.out.pop();
        }
        if !self.out.ends_with('\n') {
            self.out.push('\n');
        }
        self.out
    }

    fn item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.fn_decl(f, ""),
            Item::Const(c) => {
                let mut s = String::new();
                if c.public {
                    s.push_str("pub ");
                }
                s.push_str("let ");
                s.push_str(&c.name.name);
                if let Some(t) = &c.ty {
                    s.push_str(": ");
                    s.push_str(&type_text(t));
                }
                s.push_str(" = ");
                s.push_str(&expr_flat(&c.value, 0, false));
                self.line(&s);
                self.attach_same_line(c.span.end);
            }
            Item::Effect(e) => {
                let s = format!("{}effect {}", if e.public { "pub " } else { "" }, e.name.name);
                self.line(&s);
                self.attach_same_line(e.span.end);
            }
            Item::Type(t) => self.type_decl(t),
            Item::Foreign(fd) => self.foreign_decl(fd),
            Item::Actor(a) => self.actor_decl(a),
            Item::Test(t) => self.test_decl(t),
        }
    }

    fn fn_decl(&mut self, f: &FnDecl, prefix: &str) {
        // Attributes: one per line, directly above the declaration, after the prefix's
        // indentation (Stage 10, spec §2.2). One canonical placement — never inline with the
        // head — so the identity law has exactly one shape to round-trip.
        for a in &f.attrs {
            self.line(&format!("{prefix}{}", attr_text(a)));
        }
        let mut head = String::new();
        head.push_str(prefix);
        if f.public {
            head.push_str("pub ");
        }
        head.push_str("fn ");
        head.push_str(&f.name.name);
        head.push_str(&generics_text(&f.generics));
        let params: Vec<String> = f.params.iter().map(param_text).collect();
        let tail = {
            let mut t = String::new();
            if let Some(r) = &f.ret {
                t.push_str(" -> ");
                t.push_str(&type_text(r));
            }
            if let Some(r) = &f.row {
                t.push(' ');
                t.push_str(&row_text(r));
            }
            t
        };
        let flat = format!("{head}({}){tail} {{", params.join(", "));
        if self.indent * 4 + flat.len() <= WIDTH || params.is_empty() {
            self.line(&flat);
        } else {
            self.line(&format!("{head}("));
            self.indent += 1;
            for p in &params {
                self.line(&format!("{p},"));
            }
            self.indent -= 1;
            self.line(&format!("){tail} {{"));
        }
        self.block_body(&f.body);
        self.line("}");
        self.attach_same_line(f.span.end);
    }

    fn test_decl(&mut self, t: &TestDecl) {
        let mut head = format!("test {}", escape_str(&t.name));
        if let Some(r) = &t.row {
            head.push(' ');
            head.push_str(&row_text(r));
        }
        head.push_str(" {");
        self.line(&head);
        self.block_body(&t.body);
        self.line("}");
        self.attach_same_line(t.span.end);
    }

    fn type_decl(&mut self, t: &TypeDecl) {
        let mut head = String::new();
        if t.public {
            head.push_str("pub ");
        }
        head.push_str("type ");
        head.push_str(&t.name.name);
        head.push_str(&generics_text(&t.generics));
        match &t.kind {
            TypeDeclKind::Record(fields) => {
                if fields.is_empty() {
                    self.line(&format!("{head} {{}}"));
                } else {
                    self.line(&format!("{head} {{"));
                    self.indent += 1;
                    for fd in fields {
                        self.line(&format!("{}: {},", fd.name.name, type_text(&fd.ty)));
                    }
                    self.indent -= 1;
                    self.line("}");
                }
            }
            TypeDeclKind::Sum(variants) => {
                let vs: Vec<String> = variants
                    .iter()
                    .map(|v| {
                        if v.fields.is_empty() {
                            v.name.name.clone()
                        } else {
                            format!("{}({})", v.name.name, v.fields.iter().map(type_text).collect::<Vec<_>>().join(", "))
                        }
                    })
                    .collect();
                self.line(&format!("{head} = {}", vs.join(" | ")));
            }
            TypeDeclKind::Alias(ty) => {
                self.line(&format!("{head} = {}", type_text(ty)));
            }
        }
        self.attach_same_line(t.span.end);
    }

    fn foreign_decl(&mut self, fd: &ForeignDecl) {
        let mut head = String::new();
        if fd.public {
            head.push_str("pub ");
        }
        head.push_str(&format!("foreign {} lib {} {{", escape_str(&fd.abi), fd.name.name));
        self.line(&head);
        self.indent += 1;
        for f in &fd.fns {
            self.flush_comments_before(f.span.start);
            let params: Vec<String> = f.params.iter().map(param_text).collect();
            let mut s = format!("fn {}({})", f.name.name, params.join(", "));
            if let Some(r) = &f.ret {
                s.push_str(" -> ");
                s.push_str(&type_text(r));
            }
            self.line(&s);
            self.attach_same_line(f.span.end);
        }
        self.flush_comments_before(fd.span.end);
        self.indent -= 1;
        self.line("}");
        self.attach_same_line(fd.span.end);
    }

    fn actor_decl(&mut self, a: &ActorDecl) {
        for at in &a.attrs {
            let t = attr_text(at);
            self.line(&t);
        }
        let mut head = String::new();
        if a.public {
            head.push_str("pub ");
        }
        head.push_str("actor ");
        head.push_str(&a.name.name);
        head.push_str(&generics_text(&a.generics));
        if let Some(n) = a.mailbox {
            head.push_str(&format!("(mailbox = {n})"));
        }
        head.push_str(" {");
        self.line(&head);
        self.indent += 1;
        for f in &a.fields {
            self.flush_comments_before(f.span.start);
            let kw = if f.mutable { "var" } else { "let" };
            self.line(&format!("{kw} {}: {}", f.name.name, type_text(&f.ty)));
            self.attach_same_line(f.span.end);
        }
        // ctor
        self.blank();
        self.flush_comments_before(a.ctor.span.start);
        let params: Vec<String> = a.ctor.params.iter().map(param_text).collect();
        let mut chead = format!("new({})", params.join(", "));
        if let Some(r) = &a.ctor.row {
            chead.push(' ');
            chead.push_str(&row_text(r));
        }
        chead.push_str(" {");
        self.line(&chead);
        self.block_body(&a.ctor.body);
        self.line("}");
        self.attach_same_line(a.ctor.span.end);
        for b in &a.behaviors {
            self.blank();
            self.flush_comments_before(b.span.start);
            let params: Vec<String> = b.params.iter().map(param_text).collect();
            let mut bhead = format!("be {}({})", b.name.name, params.join(", "));
            if let Some(r) = &b.row {
                bhead.push(' ');
                bhead.push_str(&row_text(r));
            }
            bhead.push_str(" {");
            self.line(&bhead);
            self.block_body(&b.body);
            self.line("}");
            self.attach_same_line(b.span.end);
        }
        for f in &a.fns {
            self.blank();
            self.flush_comments_before(f.span.start);
            self.fn_decl(f, "");
        }
        self.flush_comments_before(a.span.end);
        self.indent -= 1;
        self.line("}");
        self.attach_same_line(a.span.end);
    }

    // ----- blocks and statements -------------------------------------------

    fn block_body(&mut self, b: &Block) {
        self.indent += 1;
        for stmt in &b.stmts {
            self.flush_comments_before(stmt_span(stmt).start);
            self.stmt(stmt);
        }
        self.flush_comments_before(b.span.end);
        self.indent -= 1;
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let { name, ty, value, mutable, span } => {
                let kw = if *mutable { "var" } else { "let" };
                let mut head = format!("{kw} {}", name.name);
                if let Some(t) = ty {
                    head.push_str(": ");
                    head.push_str(&type_text(t));
                }
                head.push_str(" = ");
                self.value_line(head, value, span.end);
            }
            Stmt::Assign { target, value, span, compound } => match (compound, value) {
                // RW 2.34: printed as written, `target op= rhs` — the parser's rewrite is the value's left side.
                (Some(op), Expr::Binary { rhs, .. }) => {
                    let head = format!("{} {}= ", lvalue_text(target), bin_text(*op));
                    self.value_line(head, rhs, span.end);
                }
                _ => {
                    let head = format!("{} = ", lvalue_text(target));
                    self.value_line(head, value, span.end);
                }
            },
            Stmt::While { cond, body, span } => {
                let c = expr_flat(cond, 0, true);
                self.line(&format!("while {c} {{"));
                self.block_body(body);
                self.line("}");
                self.attach_same_line(span.end);
            }
            Stmt::For { var, iter, body, span } => {
                let it = expr_flat(iter, 0, true);
                self.line(&format!("for {} in {it} {{", var.name));
                self.block_body(body);
                self.line("}");
                self.attach_same_line(span.end);
            }
            Stmt::Break { span } => {
                self.line("break");
                self.attach_same_line(span.end);
            }
            Stmt::Continue { span } => {
                self.line("continue");
                self.attach_same_line(span.end);
            }
            Stmt::Return { value, span } => {
                match value {
                    Some(v) => {
                        self.value_line("return ".to_string(), v, span.end);
                        return;
                    }
                    None => self.line("return"),
                }
                self.attach_same_line(span.end);
            }
            Stmt::Expr(e) => {
                self.value_line(String::new(), e, e.span().end);
            }
        }
    }

    /// Print `head` followed by a value expression, breaking the value's outermost
    /// construct (or laying out its blocks) when the flat form overflows the soft limit.
    fn value_line(&mut self, head: String, value: &Expr, end: u32) {
        // Block-shaped values lay out as blocks whenever flat doesn't fit (or the blocks
        // hold more than one statement).
        if let Some(()) = self.try_block_value(&head, value) {
            self.attach_same_line(end);
            return;
        }
        let flat = expr_flat(value, 0, false);
        if self.indent * 4 + head.len() + flat.len() <= WIDTH {
            self.line(&format!("{head}{flat}"));
        } else if self.try_break_outermost(&head, value).is_none() {
            self.line(&format!("{head}{flat}")); // soft limit: long stays long
        }
        self.attach_same_line(end);
    }

    /// Multiline layout for block-shaped values (`if`/`match`/`block`/`recover`/lambda)
    /// that don't fit flat (a `match` never fits — it is always multiline).
    fn try_block_value(&mut self, head: &str, value: &Expr) -> Option<()> {
        let flat_fits = |p: &Printer, e: &Expr| {
            let f = expr_flat(e, 0, false);
            p.indent * 4 + head.len() + f.len() <= WIDTH
        };
        match value {
            Expr::Match { scrutinee, arms, .. } => {
                let s = expr_flat(scrutinee, 0, true);
                self.line(&format!("{head}match {s} {{"));
                self.indent += 1;
                for arm in arms {
                    self.flush_comments_before(arm.span.start);
                    let pat = pattern_text(&arm.pattern);
                    match &arm.body {
                        Expr::Block(b) => {
                            self.line(&format!("{pat} => {{"));
                            self.block_body(b);
                            self.line("}");
                        }
                        // FMT-MATCH-ARM-1: an arm whose value is block-shaped (an inner `match`, an `if` that does not
                        // fit flat) is laid out as any block-shaped value is, not squashed onto the arm's line.
                        e => {
                            let head = format!("{pat} => ");
                            if self.try_block_value(&head, e).is_none() {
                                let body = expr_flat(e, 0, false);
                                self.line(&format!("{head}{body}"));
                            }
                        }
                    }
                    self.attach_same_line(arm.span.end);
                }
                self.indent -= 1;
                self.line("}");
                Some(())
            }
            Expr::If { .. } if !flat_fits(self, value) || !if_is_flat_shaped(value) => {
                self.if_multiline(head, value);
                Some(())
            }
            Expr::Block(b) => {
                self.line(&format!("{head}{{"));
                self.block_body(b);
                self.line("}");
                Some(())
            }
            Expr::Recover { target, body, .. } if !flat_fits(self, value) || body.stmts.len() > 1 => {
                let t = match target {
                    Some(r) => format!("{} ", r.name()),
                    None => String::new(),
                };
                self.line(&format!("{head}recover {t}{{"));
                self.block_body(body);
                self.line("}");
                Some(())
            }
            Expr::Lambda { params, ret, row, body, .. } if !flat_fits(self, value) || body.stmts.len() > 1 => {
                let ps: Vec<String> = params.iter().map(param_text).collect();
                let mut h = format!("{head}fn({})", ps.join(", "));
                if let Some(r) = ret {
                    h.push_str(" -> ");
                    h.push_str(&type_text(r));
                }
                if let Some(r) = row {
                    h.push(' ');
                    h.push_str(&row_text(r));
                }
                h.push_str(" {");
                self.line(&h);
                self.block_body(body);
                self.line("}");
                Some(())
            }
            _ => None,
        }
    }

    /// `head`-prefixed multiline `if` chain: `} else if c {` on one line, Go-style.
    fn if_multiline(&mut self, head: &str, e: &Expr) {
        let Expr::If { cond, then_, else_, .. } = e else { unreachable!("if only") };
        let c = expr_flat(cond, 0, true);
        self.line(&format!("{head}if {c} {{"));
        self.block_body(then_);
        let mut cur = else_;
        loop {
            match cur {
                None => {
                    self.line("}");
                    break;
                }
                Some(next) => match next.as_ref() {
                    Expr::If { cond, then_, else_, .. } => {
                        let c = expr_flat(cond, 0, true);
                        self.line(&format!("}} else if {c} {{"));
                        self.block_body(then_);
                        cur = else_;
                    }
                    Expr::Block(b) => {
                        self.line("} else {");
                        self.block_body(b);
                        self.line("}");
                        break;
                    }
                    other => {
                        // A non-block else value (grammar permits an expression).
                        self.line(&format!("}} else {{ {} }}", expr_flat(other, 0, false)));
                        break;
                    }
                },
            }
        }
    }

    /// Break the outermost call/list/record of an overflowing value line.
    fn try_break_outermost(&mut self, head: &str, value: &Expr) -> Option<()> {
        match value {
            Expr::Call { callee, args, .. } if !args.is_empty() => {
                let c = expr_flat(callee, P_POSTFIX, false);
                self.line(&format!("{head}{c}("));
                self.break_args(args);
                self.line(")");
                Some(())
            }
            Expr::Method { recv, name, args, .. } if !args.is_empty() => {
                let r = expr_flat(recv, P_POSTFIX, false);
                self.line(&format!("{head}{r}.{}(", name.name));
                self.break_args(args);
                self.line(")");
                Some(())
            }
            Expr::Spawn { actor, args, .. } if !args.is_empty() => {
                self.line(&format!("{head}spawn {}(", actor.dotted()));
                self.break_args(args);
                self.line(")");
                Some(())
            }
            Expr::List { items, .. } if !items.is_empty() => {
                self.line(&format!("{head}["));
                self.break_args(items);
                self.line("]");
                Some(())
            }
            Expr::Record { path, fields, .. } if !fields.is_empty() => {
                self.line(&format!("{head}{} {{", path.dotted()));
                self.indent += 1;
                for (n, v) in fields {
                    self.line(&format!("{}: {},", n.name, expr_flat(v, 0, false)));
                }
                self.indent -= 1;
                self.line("}");
                Some(())
            }
            _ => None,
        }
    }

    fn break_args(&mut self, args: &[Expr]) {
        self.indent += 1;
        for a in args {
            self.line(&format!("{},", expr_flat(a, 0, false)));
        }
        self.indent -= 1;
    }
}

/// The canonical text of an attribute (Stage 10): `@name` or `@name("arg")`.
fn attr_text(a: &Attribute) -> String {
    match &a.arg {
        Some(arg) => format!("@{}(\"{arg}\")", a.name.name),
        None => format!("@{}", a.name.name),
    }
}

fn item_span(item: &Item) -> delulu_diag::Span {
    match item {
        Item::Fn(f) => f.span,
        Item::Type(t) => t.span,
        Item::Effect(e) => e.span,
        Item::Const(c) => c.span,
        Item::Foreign(fd) => fd.span,
        Item::Actor(a) => a.span,
        Item::Test(t) => t.span,
    }
}

fn stmt_span(s: &Stmt) -> delulu_diag::Span {
    match s {
        Stmt::Let { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::While { span, .. }
        | Stmt::For { span, .. }
        | Stmt::Break { span }
        | Stmt::Continue { span }
        | Stmt::Return { span, .. } => *span,
        Stmt::Expr(e) => e.span(),
    }
}

// ===== flat text builders ==================================================

fn generics_text(generics: &[Ident]) -> String {
    if generics.is_empty() {
        String::new()
    } else {
        format!("[{}]", generics.iter().map(|g| g.name.as_str()).collect::<Vec<_>>().join(", "))
    }
}

fn param_text(p: &Param) -> String {
    format!("{}: {}", p.name.name, type_text(&p.ty))
}

fn type_text(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Named { path, args, .. } => {
            if args.is_empty() {
                path.dotted()
            } else {
                format!("{}[{}]", path.dotted(), args.iter().map(type_text).collect::<Vec<_>>().join(", "))
            }
        }
        TypeExpr::Fn { params, ret, row, .. } => {
            let mut s = format!("fn({})", params.iter().map(type_text).collect::<Vec<_>>().join(", "));
            if let Some(r) = ret {
                s.push_str(" -> ");
                s.push_str(&type_text(r));
            }
            if let Some(r) = row {
                s.push(' ');
                s.push_str(&row_text(r));
            }
            s
        }
        TypeExpr::Rcap { rcap, inner, .. } => format!("{} {}", rcap.name(), type_text(inner)),
    }
}

/// `! {Read, Write}` / `! e` / `! {Read | e}` / `! {}` — space after `!`, effects sorted
/// (a row is a semantic set; [`ast_fingerprint`] normalizes the same way).
fn row_text(r: &RowExpr) -> String {
    let mut effects: Vec<String> = r.effects.iter().map(|p| p.dotted()).collect();
    effects.sort();
    match (&effects[..], &r.tail) {
        ([], None) => "! {}".to_string(),
        ([], Some(t)) => format!("! {}", t.name),
        (es, None) => format!("! {{{}}}", es.join(", ")),
        (es, Some(t)) => format!("! {{{} | {}}}", es.join(", "), t.name),
    }
}

fn lvalue_text(lv: &LValue) -> String {
    match lv {
        LValue::Var(i) => i.name.clone(),
        LValue::Field(base, name) => format!("{}.{}", lvalue_text(base), name.name),
        LValue::Index(base, idx) => format!("{}[{}]", lvalue_text(base), expr_flat(idx, 0, false)),
    }
}

fn pattern_text(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard(_) => "_".to_string(),
        Pattern::Lit(k, span) => lit_text(k, *span),
        Pattern::Bind(i) => i.name.clone(),
        Pattern::Variant { path, fields, .. } => {
            if fields.is_empty() {
                // FMT-CTOR-1 (2026-11-01): the parser reads a lone lowercase name in a pattern as a binding
                // (`parse_pattern_inner`), so a nullary constructor spelled in lowercase keeps its call form —
                // `foo()` printed as `foo` became a binding that matches every value, and the next check refused it
                // (DL0302, D-V2-203) or, before that, the arm took every value. A capitalized or qualified name is a
                // constructor either way, and stays bare.
                let lone_lowercase =
                    path.segs.len() == 1 && !path.segs[0].name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                if lone_lowercase {
                    format!("{}()", path.dotted())
                } else {
                    path.dotted()
                }
            } else {
                format!("{}({})", path.dotted(), fields.iter().map(pattern_text).collect::<Vec<_>>().join(", "))
            }
        }
    }
}

fn lit_text(k: &LitKind, span: delulu_diag::Span) -> String {
    match k {
        // FMT-INT-BASE-1 (RW 2.35): an `Int` written in another base, or with its digits grouped, is printed as it was
        // written (`int_spelling`), where `{}` printed its value in decimal — `0xFF00` became `65280`, a mask nobody
        // can read. Any other `Int` is its decimal digits, as before.
        LitKind::Int(v) => int_spelling(span).unwrap_or_else(|| v.to_string()),
        // FMT-FLOAT-EXP-1: the shortest digits that read back as `v`, with an exponent below 1e-4 and from 1e16 up, where
        // `{}` would print every digit (300 of them for `1.0e-300`) — and a fraction before it, since the lexer reads an
        // exponent only after one. RW 2.37: the runtime's `str` of a `Float` is the same function.
        LitKind::Float(v) => crate::num::float_text(*v),
        LitKind::Str(s) => escape_str(s),
        LitKind::Bool(b) => b.to_string(),
    }
}

/// RW 2.37: an `Interp` chain printed as the f-string the parser built it from — its head and each literal operand as
/// text (braces doubled), every other operand as a hole printed flat, so the f-string stays on its one line.
fn fstring_text(e: &Expr) -> String {
    let mut operands = Vec::new();
    let mut at = e;
    while let Expr::Binary { op: BinOp::Interp, lhs, rhs, .. } = at {
        operands.push(&**rhs);
        at = lhs;
    }
    operands.reverse();
    let mut out = String::from("f\"");
    let text = |s: &str, out: &mut String| {
        let quoted = escape_str(s);
        for c in quoted[1..quoted.len() - 1].chars() {
            match c {
                '{' => out.push_str("{{"),
                '}' => out.push_str("}}"),
                c => out.push(c),
            }
        }
    };
    match at {
        Expr::Lit { kind: LitKind::Str(s), .. } => text(s, &mut out),
        // Only the parser builds an `Interp`, always on a `Str` literal head; any other head is printed as a hole.
        other => {
            out.push('{');
            out.push_str(&expr_flat(other, 0, false));
            out.push('}');
        }
    }
    for operand in operands {
        match operand {
            Expr::Lit { kind: LitKind::Str(s), .. } => text(s, &mut out),
            other => {
                out.push('{');
                out.push_str(&expr_flat(other, 0, false));
                out.push('}');
            }
        }
    }
    out.push('"');
    out
}

/// Re-escape a string literal — the exact inverse of the lexer's escape set.
fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Operator precedence, mirroring the parser: `||` < `&&` < comparisons < ranges (RW 2.36) < `|` < `^` < `&` <
/// `<< >>` (RW 2.35) < `+ -` < `* / %` < unary < postfix < atoms.
fn bin_prec(op: BinOp) -> u8 {
    use BinOp::*;
    match op {
        Or => 1,
        And => 2,
        Eq | Ne | Lt | Le | Gt | Ge => 3,
        Range | RangeInclusive => 4,
        BitOr => 5,
        BitXor => 6,
        BitAnd => 7,
        Shl | Shr => 8,
        Add | Sub => 9,
        Mul | Div | Rem => 10,
        // RW 2.37: an f-string is one primary expression.
        Interp => P_ATOM,
    }
}

/// The levels above every binary operator: a unary operator's operand, a postfix form's receiver, and an atom.
const P_UNARY: u8 = 11;
const P_POSTFIX: u8 = 12;
const P_ATOM: u8 = 13;

fn bin_text(op: BinOp) -> &'static str {
    use BinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
        Eq => "==",
        Ne => "!=",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        And => "&&",
        Or => "||",
        Range => "..",
        RangeInclusive => "..=",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        // RW 2.37: never printed as an operator — `fstring_text` prints the chain as the f-string it was.
        Interp => "+",
    }
}

/// Flat (single-line) expression text. `min_prec` drives re-parenthesization: exactly the
/// parens precedence requires, none the source may have carried redundantly (the AST holds
/// no paren nodes, so this IS identity-preserving). `no_struct` mirrors the grammar's
/// condition/scrutinee restriction: a record literal there must print parenthesized.
fn expr_flat(e: &Expr, min_prec: u8, no_struct: bool) -> String {
    let (text, prec) = match e {
        Expr::Lit { kind, span, .. } => (lit_text(kind, *span), P_ATOM),
        Expr::Var { path, .. } => (path.dotted(), P_ATOM),
        Expr::List { items, .. } => {
            (format!("[{}]", items.iter().map(|i| expr_flat(i, 0, false)).collect::<Vec<_>>().join(", ")), P_ATOM)
        }
        Expr::Record { path, fields, .. } => {
            let body =
                fields.iter().map(|(n, v)| format!("{}: {}", n.name, expr_flat(v, 0, false))).collect::<Vec<_>>().join(", ");
            let s =
                if fields.is_empty() { format!("{} {{}}", path.dotted()) } else { format!("{} {{ {} }}", path.dotted(), body) };
            if no_struct {
                return format!("({s})");
            }
            (s, P_ATOM)
        }
        Expr::Call { callee, args, .. } => {
            let c = expr_flat(callee, P_POSTFIX, no_struct);
            (format!("{c}({})", args.iter().map(|a| expr_flat(a, 0, false)).collect::<Vec<_>>().join(", ")), P_POSTFIX)
        }
        Expr::Method { recv, name, args, .. } => {
            let r = expr_flat(recv, P_POSTFIX, no_struct);
            (
                format!("{r}.{}({})", name.name, args.iter().map(|a| expr_flat(a, 0, false)).collect::<Vec<_>>().join(", ")),
                P_POSTFIX,
            )
        }
        Expr::Field { recv, name, .. } => (format!("{}.{}", expr_flat(recv, P_POSTFIX, no_struct), name.name), P_POSTFIX),
        Expr::Index { recv, index, .. } => {
            (format!("{}[{}]", expr_flat(recv, P_POSTFIX, no_struct), expr_flat(index, 0, false)), P_POSTFIX)
        }
        Expr::Try { inner, .. } => (format!("{}?", expr_flat(inner, P_POSTFIX, no_struct)), P_POSTFIX),
        Expr::Unary { op, operand, .. } => {
            let o = match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
                UnOp::BitNot => "~",
            };
            (format!("{o}{}", expr_flat(operand, P_UNARY, no_struct)), P_UNARY)
        }
        Expr::Binary { op: BinOp::Interp, .. } => (fstring_text(e), P_ATOM),
        // RW 2.36: a range is printed close, `0..n`, as Rust's and Kotlin's formatters print it; it does not chain,
        // so a range on its left keeps its parentheses too.
        Expr::Binary { op: op @ (BinOp::Range | BinOp::RangeInclusive), lhs, rhs, .. } => {
            let p = bin_prec(*op);
            let l = expr_flat(lhs, p + 1, no_struct);
            let r = expr_flat(rhs, p + 1, no_struct);
            (format!("{l}{}{r}", bin_text(*op)), p)
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            let p = bin_prec(*op);
            let l = expr_flat(lhs, p, no_struct);
            let r = expr_flat(rhs, p + 1, no_struct);
            (format!("{l} {} {r}", bin_text(*op)), p)
        }
        Expr::If { cond, then_, else_, .. } => {
            let c = expr_flat(cond, 0, true);
            let t = block_flat(then_);
            let s = match else_ {
                None => format!("if {c} {t}"),
                Some(e) => match e.as_ref() {
                    Expr::Block(b) => format!("if {c} {t} else {}", block_flat(b)),
                    other => format!("if {c} {t} else {}", expr_flat(other, 0, false)),
                },
            };
            // An `if` in operand position always parenthesizes (prec 0).
            (s, 0)
        }
        Expr::Match { scrutinee, arms, .. } => {
            // Flat match is a last resort (value_line always lays match out multiline);
            // it appears only nested inside another flat expression.
            let s = expr_flat(scrutinee, 0, true);
            let a = arms
                .iter()
                .map(|arm| format!("{} => {}", pattern_text(&arm.pattern), expr_flat(&arm.body, 0, false)))
                .collect::<Vec<_>>()
                .join(", ");
            (format!("match {s} {{ {a} }}"), 0)
        }
        Expr::Lambda { params, ret, row, body, .. } => {
            let ps: Vec<String> = params.iter().map(param_text).collect();
            let mut s = format!("fn({})", ps.join(", "));
            if let Some(r) = ret {
                s.push_str(" -> ");
                s.push_str(&type_text(r));
            }
            if let Some(r) = row {
                s.push(' ');
                s.push_str(&row_text(r));
            }
            s.push(' ');
            s.push_str(&block_flat(body));
            (s, 0)
        }
        Expr::Block(b) => (block_flat(b), 0),
        Expr::Spawn { actor, args, .. } => (
            format!("spawn {}({})", actor.dotted(), args.iter().map(|a| expr_flat(a, 0, false)).collect::<Vec<_>>().join(", ")),
            P_UNARY,
        ),
        Expr::Consume { name, .. } => (format!("consume {}", name.name), P_UNARY),
        Expr::Recover { target, body, .. } => {
            let t = match target {
                Some(r) => format!("{} ", r.name()),
                None => String::new(),
            };
            (format!("recover {t}{}", block_flat(body)), P_UNARY)
        }
    };
    if prec < min_prec {
        format!("({text})")
    } else {
        text
    }
}

/// A block rendered flat: `{ s1; s2 }` — explicit `;` is a legal statement terminator
/// (§2.2), which is what makes ANY block expressible on one line. The printer prefers
/// multiline layouts wherever style calls for them; flat blocks appear only nested
/// inside flat expressions.
fn block_flat(b: &Block) -> String {
    if b.stmts.is_empty() {
        return "{}".to_string();
    }
    let parts: Vec<String> = b.stmts.iter().map(stmt_flat).collect();
    format!("{{ {} }}", parts.join("; "))
}

fn stmt_flat(s: &Stmt) -> String {
    match s {
        Stmt::Let { name, ty, value, mutable, .. } => {
            let kw = if *mutable { "var" } else { "let" };
            let mut t = format!("{kw} {}", name.name);
            if let Some(ann) = ty {
                t.push_str(": ");
                t.push_str(&type_text(ann));
            }
            t.push_str(" = ");
            t.push_str(&expr_flat(value, 0, false));
            t
        }
        Stmt::Assign { target, value, compound, .. } => match (compound, value) {
            (Some(op), Expr::Binary { rhs, .. }) => {
                format!("{} {}= {}", lvalue_text(target), bin_text(*op), expr_flat(rhs, 0, false))
            }
            _ => format!("{} = {}", lvalue_text(target), expr_flat(value, 0, false)),
        },
        Stmt::Return { value, .. } => match value {
            Some(v) => format!("return {}", expr_flat(v, 0, false)),
            None => "return".to_string(),
        },
        Stmt::Expr(e) => expr_flat(e, 0, false),
        Stmt::While { cond, body, .. } => {
            format!("while {} {}", expr_flat(cond, 0, true), block_flat(body))
        }
        Stmt::For { var, iter, body, .. } => {
            format!("for {} in {} {}", var.name, expr_flat(iter, 0, true), block_flat(body))
        }
        Stmt::Break { .. } => "break".to_string(),
        Stmt::Continue { .. } => "continue".to_string(),
    }
}

/// Is this `if` chain flat-shaped: every branch a single-statement block (flat legal)?
fn if_is_flat_shaped(e: &Expr) -> bool {
    let Expr::If { then_, else_, .. } = e else { return false };
    if then_.stmts.len() != 1 || !matches!(then_.stmts[0], Stmt::Expr(_)) {
        return false;
    }
    match else_ {
        None => true,
        Some(next) => match next.as_ref() {
            Expr::Block(b) => b.stmts.len() == 1 && matches!(b.stmts[0], Stmt::Expr(_)),
            other @ Expr::If { .. } => if_is_flat_shaped(other),
            _ => true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The full law check on one source: the output reparses clean; the identity
    /// fingerprints match; the comment sequence is preserved; and formatting is
    /// idempotent byte-for-byte. Every corpus program and every unit case goes
    /// through this single gate.
    fn laws(src: &str) -> String {
        let out = format_source(0, src).expect("input parses — fmt must not refuse");
        let (m1, d1) = crate::parse_file(0, src);
        assert!(d1.iter().all(|d| !d.is_error()), "law harness needs clean input: {d1:?}");
        let (m2, d2) = crate::parse_file(0, &out);
        assert!(d2.iter().all(|d| !d.is_error()), "formatted output must reparse clean:\n{out}\n{d2:?}");
        let (f1, f2) = (ast_fingerprint(&m1), ast_fingerprint(&m2));
        if f1 != f2 {
            let at = f1.bytes().zip(f2.bytes()).position(|(a, b)| a != b).unwrap_or(f1.len().min(f2.len()));
            let lo = at.saturating_sub(120);
            panic!(
                "IDENTITY violated at fingerprint byte {at}:\n--- input fp ---\n…{}…\n--- output fp ---\n…{}…\n--- output ---\n{out}",
                &f1[lo..(at + 120).min(f1.len())],
                &f2[lo..(at + 120).min(f2.len())],
            );
        }
        assert_eq!(
            comment_sequence(src),
            comment_sequence(&out),
            "comment attachment moved:\n--- input ---\n{src}\n--- output ---\n{out}"
        );
        let out2 = format_source(0, &out).expect("formatted output reparses");
        assert_eq!(out, out2, "IDEMPOTENCE violated:\n--- first ---\n{out}\n--- second ---\n{out2}");
        out
    }

    #[test]
    fn a_blank_line_between_comment_paragraphs_survives() {
        // C15. The formatter emitted consecutive own-line comments back to back, so two paragraphs
        // the author had deliberately separated were merged into one block — irrecoverably, since
        // nothing downstream knows where the break had been.
        //
        // The identity law did not catch it and COULD NOT: its projection is each comment's
        // `(text, own_line)` in order, and the merge changes neither. Only the spacing *between*
        // comments was lost, which is precisely the part carrying the author's meaning. A law that
        // watches the pieces and not the gaps between them has a blind spot exactly this wide.
        let src = "module m

            // First paragraph.
            // Still the first.

            // Second paragraph, separated on purpose.
            fn f() -> Int { 1 }
";
        let out = laws(src);
        let body = out.split_once("// First paragraph.").expect("the first paragraph survives").1;
        let between = body.split_once("// Second paragraph").expect("the second paragraph survives").0;
        assert!(
            between.contains(
                "

"
            ),
            "the blank line between the two comment paragraphs must survive:
{out}"
        );
    }

    #[test]
    fn comment_paragraph_spacing_is_canonical_not_merely_copied() {
        // Two blank lines collapse to one — that is ordinary canonical formatting, and it is the
        // line between "preserve the author's structure" and "preserve the author's whitespace".
        // Deleting the separation entirely was the bug; reproducing an arbitrary run of blank lines
        // would be a different one.
        let src = "module m

// A.



// B.
fn f() -> Int { 1 }
";
        let out = laws(src);
        let mid = out.split_once("// A.").unwrap().1.split_once("// B.").unwrap().0;
        assert_eq!(
            mid,
            "

",
            "runs of blank lines collapse to exactly one: {mid:?}"
        );
    }

    #[test]
    fn a_trailing_comment_does_not_create_a_false_paragraph_break() {
        // The skip branch of the C15 fix. A trailing comment belongs to the line above it, so it must
        // not be treated as the end of a paragraph — otherwise the next own-line comment gets a blank
        // line inserted *in addition* to the one item separation already provides, and the formatter
        // starts inventing whitespace instead of preserving it.
        //
        // The single blank line below is the ordinary "one blank line between items" rule, not this
        // fix; what is asserted is that nothing added a second one.
        let src = "module m
let a = 1 // trailing
// own-line, immediately after
fn f() -> Int { 1 }
";
        let out = laws(src);
        let after = out.split_once("// trailing").expect("the trailing comment survives").1;
        assert!(
            !after.starts_with(
                "


"
            ),
            "a trailing comment must not add a paragraph break on top of item separation:
{out}"
        );
        assert!(
            after.contains("// own-line"),
            "the own-line comment survives:
{out}"
        );
    }

    /// **FMT-FLOAT-EXP-1** (2026-11-08, found by RW 2.38's math programs on the laptop). A `Float` literal was printed with
    /// Rust's `{}`, which never uses an exponent, so `1.0e-300` came back as a 300-digit decimal and `6.02e23` as 24
    /// digits. A magnitude below 1e-4 or from 1e16 up now prints with an exponent (`1.0e-300`), the shortest digits that
    /// read back as the same value; everything between prints as before, so no ordinary literal moves.
    #[test]
    fn a_float_literal_of_extreme_magnitude_keeps_an_exponent() {
        let out =
            laws("module m\nfn f() -> Float {\n  1.0e-300 + 6.02e23 + 0.00001 + 1.0e16 + 2.5e10 + 0.0001 + 0.5 + 123.0\n}\n");
        assert!(out.contains("1.0e-300 + 6.02e23 + 1.0e-5 + 1.0e16 + 25000000000.0 + 0.0001 + 0.5 + 123.0"), "{out}");
    }

    /// **FMT-MATCH-ARM-1** (2026-11-09, found by RW 2.38's `loan.delulu`). A multiline `match` printed every arm body that
    /// was not a block with `expr_flat`, so an arm whose value is itself a `match` — which this printer always lays out on
    /// several lines — came out as one line holding the inner match, its block arms and their statements joined by `;`
    /// (about 700 characters in `loan.delulu`). Such an arm body is now laid out as any block-shaped value is: an inner
    /// `match` on its own lines, and an `if` that does not fit flat as a multiline `if`.
    #[test]
    fn a_match_as_an_arm_body_is_laid_out_on_its_own_lines() {
        let out = laws(
            "module m\nfn f(a: Option[Int], b: Option[Int]) -> Int {\n  match a {\n    None => 0\n    Some(x) => match b {\n      \
             None => x\n      Some(y) => {\n        let s = x + y\n        s * 2\n      }\n    }\n  }\n}\n",
        );
        assert!(out.contains("        Some(x) => match b {\n            None => x\n            Some(y) => {\n"), "{out}");
        assert!(out.lines().all(|l| l.len() <= 100), "no line squashed flat:\n{out}");
        assert!(!out.contains(';'), "no statements joined by `;`:\n{out}");
    }

    /// **FMT-CTOR-1** (2026-11-01, found by `laws_hold_over_the_repository_corpus` on RW 2.26's accepting program). A
    /// lone lowercase name in a pattern is a binding, so `foo()`, a nullary constructor in its call form, printed as
    /// `foo` became a binding that matched every value: `fmt` changed what the program meant. A capitalized name and a
    /// qualified one are constructors bare or not, and print bare; a constructor with fields keeps them.
    #[test]
    fn a_lowercase_nullary_constructor_pattern_keeps_its_call_form() {
        let out = laws(
            "module m\ntype T = foo | Bar | cell(Int)\nfn f(t: T) -> Int { match t { foo() => 1, Bar() => 2, cell(n) => n } }\n",
        );
        assert!(out.contains("foo() => 1"), "the call form stays: {out}");
        assert!(out.contains("Bar => 2") && out.contains("cell(n) => n"), "{out}");
    }

    #[test]
    fn canonical_style_choices() {
        let out = laws("module m\nfn add(a:Int,b:Int)->Int !{} {a+b}\n");
        assert!(out.contains("fn add(a: Int, b: Int) -> Int ! {} {"), "{out}");
        assert!(out.contains("\n    a + b\n"), "4-space indent: {out}");
    }

    #[test]
    fn rows_alphabetize_and_keep_tails() {
        let out = laws("module m\nfn f(c: Cap[Console], fs: Cap[FsRead]) ! {Write, Read} { let x = fs.read_text(\"a\")\n c.println(\"y\") }\n");
        assert!(out.contains("! {Read, Write}"), "{out}");
        let out2 = laws("module m\nfn apply[T, e](f: fn(T) -> T ! e, x: T) -> T ! e { f(x) }\n");
        assert!(out2.contains("! e {") && out2.contains("-> T ! e"), "{out2}");
    }

    #[test]
    fn comments_survive_with_attachment() {
        let src = "module m\n// above the fn\nfn f(n: Int) -> Int {\n    n // trailing\n}\n";
        let out = laws(src);
        assert!(out.contains("// above the fn\nfn f"), "own-line stays own-line: {out}");
        assert!(out.contains("n  // trailing"), "trailing stays trailing: {out}");
    }

    #[test]
    fn else_stays_on_the_closing_brace_line() {
        let out = laws(
            "module m\nfn f(n: Int) -> Int { if n < 100000000 { some_extremely_long_name(n) + some_extremely_long_name(n) + n } else { some_extremely_long_name(n) } }\nfn some_extremely_long_name(n: Int) -> Int { n }\n",
        );
        assert!(out.contains("} else {"), "{out}");
    }

    #[test]
    fn long_call_breaks_with_trailing_commas() {
        let src = "module m\nfn f() -> Int { some_extremely_long_function_name(11111111, 22222222, 33333333, 44444444, 55555555, 66666666, 77777777) }\nfn some_extremely_long_function_name(a: Int, b: Int, c: Int, d: Int, e: Int, f2: Int, g: Int) -> Int { a }\n";
        let out = laws(src);
        assert!(out.contains("(\n        11111111,\n"), "broken args: {out}");
        assert!(out.contains("77777777,\n    )"), "trailing comma before closer: {out}");
    }

    #[test]
    fn records_in_conditions_parenthesize() {
        let src = "module m\ntype P { x: Int }\nfn f() -> Int { if (P { x: 1 }).x == 1 { 1 } else { 0 } }\n";
        laws(src);
    }

    #[test]
    fn floats_stay_floats() {
        let out = laws("module m\nlet HALF: Float = 0.5\nlet ONE: Float = 1.0\n");
        assert!(out.contains("1.0"), "a whole float keeps its point: {out}");
    }

    #[test]
    fn string_escapes_round_trip() {
        laws("module m\nlet S: Str = \"a\\\"b\\\\c\\nd\\te\\u{1f426}\"\n");
    }

    #[test]
    fn actors_and_tests_format() {
        let src = "module m\nactor Counter {\n var count: Int\n new(start: Int) { self.count = start }\n be add(n: Int) { self.count = self.count + n }\n fn doubled() -> Int { self.count * 2 }\n}\ntest \"t\" { assert_eq(1, 1) }\n";
        let out = laws(src);
        assert!(out.contains("actor Counter {"), "{out}");
        assert!(out.contains("    be add(n: Int) {"), "{out}");
        assert!(out.contains("test \"t\" {"), "{out}");
    }

    #[test]
    fn imports_sort_std_first() {
        let src = "module m\nimport zeta\nimport std.io\nimport alpha\nfn f() -> Int { 1 }\n";
        let out = laws(src);
        let z = out.find("import zeta").unwrap();
        let s = out.find("import std.io").unwrap();
        let a = out.find("import alpha").unwrap();
        assert!(s < a && a < z, "std first, then alphabetical: {out}");
    }

    /// Criterion 4 over the real corpus: every parse-clean `.delulu` in the repository
    /// obeys identity + idempotence + comment preservation. (The reject corpus is
    /// parse-dirty by design — fmt REFUSES those, witnessed separately.)
    ///
    /// **Under Miri this stops after a handful of files, and that is deliberate.** Miri is roughly
    /// two orders of magnitude slower than native, and formatting the whole corpus under it did not
    /// finish inside a 30-minute budget — a run that does not finish is not a pass, so the honest
    /// choices were to skip the test entirely or to shrink it. Shrinking is strictly better:
    /// skipping means the formatter's code paths are never interpreted at all, while a small cap
    /// still puts real programs through `laws()` under Miri and would still catch undefined
    /// behaviour there. What Miri gives up is the *breadth* of the corpus, which the native run on
    /// every commit already covers.
    ///
    /// **The budget counts files EXAMINED, not files checked**, and the difference is the whole
    /// point. A cap on parse-clean files bounds nothing: the walk still parses every reject-corpus
    /// file it meets on the way, and those are deliberately parse-dirty and deliberately numerous.
    /// The first version of this cap was written that way and did not finish under Miri in forty
    /// minutes — it was still walking, having "checked" almost nothing. Bounding the work actually
    /// done makes the run's cost independent of how the corpus happens to be laid out.
    #[test]
    fn laws_hold_over_the_repository_corpus() {
        // Natively unbounded: the floor below (25) is the real contract. Under Miri this is what
        // makes the run terminate at all.
        //
        // FOUR, and the number is measured rather than guessed. Timed alone under Miri, this test
        // costs ~115 s per corpus file (12 files = 1377 s). The generated-programs gate beside it
        // costs ~54 s per program. At the previous budgets the two together needed ~41 minutes
        // against a 40-minute wall, so the module timed out with BOTH halves individually passing —
        // which reads exactly like a failure and is not one. Four files plus eight programs comes to
        // roughly 15 minutes, which fits with room to spare.
        //
        // Fewer files costs REPETITION, not reach: `laws()` interprets the same code either way.
        let budget = if cfg!(miri) { 4 } else { usize::MAX };
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let mut checked = 0;
        let mut examined = 0;
        let mut stack = vec![root.join("tests"), root.join("examples")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "delulu") {
                    examined += 1;
                    if examined > budget {
                        return; // Miri only — see the budget above
                    }
                    let src = std::fs::read_to_string(&p).unwrap();
                    let (_m, d) = crate::parse_file(0, &src);
                    if d.iter().any(|x| x.is_error()) {
                        continue; // reject-corpus / parse-dirty: fmt refuses these
                    }
                    let _ = laws(&src);
                    checked += 1;
                }
            }
        }
        assert!(checked >= 25, "expected a substantial corpus, formatted {checked}");
    }

    #[test]
    fn unparseable_input_is_refused_never_formatted() {
        let err = format_source(0, "module m\nfn f( {\n").expect_err("garbage must refuse");
        assert!(err.iter().any(|d| d.is_error()));
    }

    // ===== the generative gate (criterion 4: ≥100k programs) ===============
    //
    // fmt is parse-level, so the generator needs only SYNTACTIC validity — no typing
    // discipline — which frees it to hit the whole grammar: every operator, nested
    // conditionals, match patterns, lambdas, strings full of escapes, floats, actors,
    // test blocks, and comments sprinkled between statements (the attachment law under
    // fire). Deterministic xorshift, reproducible from the printed seed.

    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Rng {
            Rng(seed | 1)
        }
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n.max(1)
        }
        fn chance(&mut self, num: u64, den: u64) -> bool {
            self.below(den) < num
        }
    }

    fn gen_expr(r: &mut Rng, depth: u32) -> String {
        if depth == 0 {
            return match r.below(7) {
                // RW 2.35 (FMT-INT-BASE-1): an `Int` in another base, or with its digits grouped, so the laws hold over
                // a literal printed as it was written.
                6 => ["0x1F", "0XAB", "0xdead_BEEF", "0b1010", "0B11", "0o17", "0O7", "1_000", "0x7FFF_FFFF_FFFF_FFFF"]
                    [r.below(9) as usize]
                    .to_string(),
                0 => format!("{}", r.below(1000)),
                1 => format!("{}.{}", r.below(50), r.below(10) + 1),
                2 => ["true", "false"][r.below(2) as usize].to_string(),
                3 => format!("x{}", r.below(4)),
                4 => format!("\"s{}\\n\\\"q\\\\{}\"", r.below(100), "🐦"),
                _ => format!("h{}()", r.below(3)),
            };
        }
        let d = depth - 1;
        match r.below(13) {
            // RW 2.37: an f-string — text runs (braces doubled, escapes, other scripts) and holes of any generated
            // expression, so the laws hold over every shape a hole can hold.
            12 => {
                let mut f = String::from("f\"");
                for _ in 0..=r.below(4) {
                    if r.chance(2, 5) {
                        f.push_str(["a", " ", "é🐦", "{{", "}}", "\\n", "\\\"", "\\\\", "\\u{41}"][r.below(9) as usize]);
                    } else {
                        f.push_str(&format!("{{{}}}", gen_expr(r, d)));
                    }
                }
                f.push('"');
                f
            }
            0 => {
                // RW 2.36: the range operators too, so the laws hold over `a..b` and `a..=b`. RW 2.35: and the five
                // bitwise ones, at their four levels between a range and arithmetic.
                let op = [
                    "+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">=", "&&", "||", "..", "..=", "&", "|", "^", "<<", ">>",
                ][r.below(20) as usize];
                format!("{} {op} {}", gen_expr(r, d), gen_expr(r, d))
            }
            1 => format!("-{}", gen_expr(r, d)),
            // RW 2.35: `~` beside `!`, both prefix.
            2 => format!("{}{}", ["!", "~"][r.below(2) as usize], gen_expr(r, d)),
            3 => format!("h{}({})", r.below(3), gen_args(r, d)),
            4 => format!("x{}.m{}({})", r.below(4), r.below(3), gen_args(r, d)),
            5 => format!("x{}.f{}", r.below(4), r.below(3)),
            6 => format!("x{}[{}]", r.below(4), gen_expr(r, d)),
            7 => format!("if {} {{ {} }} else {{ {} }}", gen_expr(r, d), gen_expr(r, d), gen_expr(r, d)),
            8 => format!(
                "match {} {{ Ok(v) => {}, Err(_) => {}, _ => {} }}",
                gen_expr(r, d),
                gen_expr(r, d),
                gen_expr(r, d),
                gen_expr(r, d)
            ),
            9 => format!("fn(a: Int) -> Int {{ {} }}", gen_expr(r, d)),
            10 => {
                let n = r.below(4);
                let items: Vec<String> = (0..n).map(|_| gen_expr(r, d)).collect();
                format!("[{}]", items.join(", "))
            }
            _ => format!("h{}({})?", r.below(3), gen_args(r, d)),
        }
    }

    fn gen_args(r: &mut Rng, depth: u32) -> String {
        let n = r.below(3);
        (0..n).map(|_| gen_expr(r, depth)).collect::<Vec<_>>().join(", ")
    }

    fn gen_stmts(r: &mut Rng, depth: u32, out: &mut String, indent: &str) {
        let n = 1 + r.below(4);
        for i in 0..n {
            if r.chance(1, 4) {
                out.push_str(&format!("{indent}// c{}\n", r.below(1000)));
            }
            if r.chance(1, 8) {
                out.push_str(&format!("{indent}/* b{} */\n", r.below(1000)));
            }
            match r.below(6) {
                0 => out.push_str(&format!("{indent}let x{} = {}\n", i, gen_expr(r, depth))),
                1 => out.push_str(&format!("{indent}var x{} = {}\n", i, gen_expr(r, depth))),
                // RW 2.34: an assignment is plain or compound, so the laws hold over `x op= e` too.
                2 => {
                    let op = ["=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="][r.below(11) as usize];
                    out.push_str(&format!("{indent}x{} {op} {}\n", r.below(4), gen_expr(r, depth)))
                }
                3 if depth > 0 => {
                    out.push_str(&format!("{indent}while {} {{\n", gen_expr(r, 1)));
                    gen_stmts(r, depth - 1, out, &format!("{indent}    "));
                    out.push_str(&format!("{indent}}}\n"));
                }
                4 => out.push_str(&format!("{indent}{}\n", gen_expr(r, depth))),
                _ => out.push_str(&format!("{indent}h{}({})\n", r.below(3), gen_args(r, depth))),
            }
            if r.chance(1, 6) {
                out.push_str(&format!("{indent}h0() // trail{}\n", r.below(100)));
            }
        }
    }

    fn gen_module(seed: u64) -> String {
        let mut r = Rng::new(seed);
        let mut src = String::from("module fuzzfmt\n");
        if r.chance(1, 3) {
            src.push_str("// header comment\n");
        }
        for h in 0..3 {
            src.push_str(&format!("fn h{h}(a: Int, b: Str) -> Int ! {{Write, Read}} {{\n"));
            let mut body = String::new();
            gen_stmts(&mut r, 2, &mut body, "    ");
            src.push_str(&body);
            src.push_str("    0\n}\n");
        }
        if r.chance(1, 2) {
            src.push_str("type P { x: Int, y: Float }\n");
        }
        if r.chance(1, 3) {
            src.push_str("type C = Red | Green(Int) | Blue(Str, Bool)\n");
        }
        if r.chance(1, 3) {
            let mut body = String::new();
            gen_stmts(&mut r, 1, &mut body, "    ");
            src.push_str(&format!("test \"gen {}\" {{\n{body}    assert(true)\n}}\n", seed % 97));
        }
        src
    }

    fn run_fuzz_gate(n: u64) {
        let mut checked = 0u64;
        for seed in 1..=n {
            let src = gen_module(seed);
            let (_m, d) = crate::parse_file(0, &src);
            if d.iter().any(|x| x.is_error()) {
                // A generator artifact (e.g. `!x` where a Term made it a statement head):
                // fmt refuses parse-dirty input by design, nothing to test. Keep rare.
                continue;
            }
            let _ = laws(&src);
            checked += 1;
            if checked.is_multiple_of(10_000) {
                eprintln!("fmt fuzz: {checked}/{n} clean programs law-checked");
            }
        }
        assert!(checked >= n * 9 / 10, "the generator must produce overwhelmingly parseable programs ({checked}/{n})");
    }

    /// The always-on slice of the criterion-4 gate.
    #[test]
    fn fmt_laws_hold_over_generated_programs() {
        // 2,000 generated programs natively; 8 under Miri. Same generator, same laws, same code
        // paths interpreted — only the repetition shrinks. At 2,000 this did not finish inside a
        // 30-minute Miri budget, and an unfinished run is not a pass; the alternative was to skip
        // it under Miri, which would interpret none of the formatter at all.
        //
        // EIGHT, measured: ~54 s per program under Miri, so 20 cost ~18 minutes and left the corpus
        // test beside it too little of the 40-minute wall to finish. See the note on `budget` in
        // `laws_hold_over_the_repository_corpus` for the arithmetic of the pair.
        run_fuzz_gate(if cfg!(miri) { 8 } else { 2_000 });
    }

    /// RW 2.37: an f-string prints as the f-string it was — its text with braces doubled and its escapes, every hole
    /// flat on the one line, however long the line — and the laws hold over it.
    #[test]
    fn an_fstring_prints_as_written_and_keeps_the_laws() {
        let src = "module m\nfn f(n: Int, s: Str, xs: List[Int]) -> Str {\n    f\"a {{b}} {n + 1}\\n{s.trim()}{xs.map(fn(i: Int) -> Int { i * 2 })} \\\"q\\\" {if n > 2 { \"big\" } else { \"small\" }}\"\n}\n";
        let out = laws(src);
        assert!(out.contains(r#"f"a {{b}} {n + 1}\n{s.trim()}{xs.map(fn(i: Int) -> Int { i * 2 })} \"q\" {if n > 2 { "big" } else { "small" }}""#), "{out}");
        // A literal hole becomes text, and an f-string with no hole left is the string literal it spells.
        let out = laws("module m\nfn f(n: Int) -> Str {\n    f\"{\"x\"}{n}\" + f\"{{}}\"\n}\n");
        assert!(out.contains(r#"f"x{n}" + "{}""#), "{out}");
        // A line far past the width: the call breaks, the f-string stays whole on its line.
        let long = format!(
            "module m\nfn f(c: Cap[Console], n: Int) ! {{Write}} {{\n    c.println(f\"{} {{n}} {}\")\n}}\n",
            "w".repeat(150),
            "{n * 2}"
        );
        let out = laws(&long);
        assert!(out.lines().any(|l| l.trim_start().starts_with("f\"") && l.contains("{n * 2}")), "{out}");
    }

    /// RW 2.34: a compound assignment prints as written — `op=` with one space each side, its right side alone (never the
    /// parser's `target op rhs`) — on a local, a field and an index, and the laws hold over it.
    #[test]
    fn a_compound_assignment_prints_as_written() {
        let src = "module m\nfn f(p: ref P, xs: ref List[Int]) -> Int {\n    var x = 1\n    x+=2\n    x -= 3 * 4\n    x  *=  x + 1\n    x /= 2\n    x %= 7\n    p.n += x\n    xs[x] -= 1\n    x\n}\n";
        let out = laws(src);
        for line in [
            "    x += 2\n",
            "    x -= 3 * 4\n",
            "    x *= x + 1\n",
            "    x /= 2\n",
            "    x %= 7\n",
            "    p.n += x\n",
            "    xs[x] -= 1\n",
        ] {
            assert!(out.contains(line), "expected {line:?} in:\n{out}");
        }
        assert!(!out.contains("x = x"), "the parser's rewrite is never printed:\n{out}");
        // Flat, inside an expression (`block_flat`): a lambda's body on one line keeps its statements as written.
        let flat = laws(
            "module m\nfn f(xs: List[Int]) -> List[Int] {\n    xs.map(fn(n: Int) -> Int { var m = n; m += 1; m *= 2; m })\n}\n",
        );
        assert!(flat.contains("{ var m = n; m += 1; m *= 2; m }"), "{flat}");
        // A plain assignment of the same value stays plain: only what was written short prints short.
        let plain = laws("module m\nfn f() -> Int {\n    var x = 1\n    x = x + 2\n    x\n}\n");
        assert!(plain.contains("    x = x + 2\n") && !plain.contains("+="), "{plain}");
    }

    /// RW 2.36: a range prints close, `lo..hi` and `lo..=hi`, with exactly the parentheses its level needs: none around
    /// arithmetic operands, none in a comparison, one pair as a method's receiver or around a comparison operand.
    #[test]
    fn a_range_prints_close_with_the_parentheses_it_needs() {
        let src = "module m\nfn f(n: Int, xs: List[Int], a: Bool) -> Bool {\n    for i in 0 .. n {\n        println(str(i))\n    }\n    let r = 1 ..= n + 1\n    let s = (0..3).reverse()\n    let t = -3..n * 2\n    let u = (0..n).concat(n..=(n + 2))\n    let w = (if a { 1 } else { 2 })..3\n    xs == (0..3)\n}\n";
        let out = laws(src);
        for line in [
            "    for i in 0..n {\n",
            "    let r = 1..=n + 1\n",
            "    let s = (0..3).reverse()\n",
            "    let t = -3..n * 2\n",
            "    let u = (0..n).concat(n..=n + 2)\n",
            "    let w = (if a { 1 } else { 2 })..3\n",
            "    xs == 0..3\n",
        ] {
            assert!(out.contains(line), "expected {line:?} in:\n{out}");
        }
    }

    /// Criterion 4's full ≥100k gate. `cargo test -p delulu-syntax --release -- --ignored
    /// fmt_laws_100k` (env `DELULU_FMT_FUZZ_N` overrides the count).
    #[test]
    #[ignore]
    fn fmt_laws_100k_gate() {
        let n = std::env::var("DELULU_FMT_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(100_000);
        run_fuzz_gate(n);
    }

    /// RW 2.35: a bitwise expression prints with one space each side of its operator, `~` close to its operand, and
    /// exactly the parentheses its levels need — the ones the source carried without needing them dropped, the ones
    /// that change the tree kept — and each compound form as written.
    #[test]
    fn a_bitwise_expression_prints_with_the_parentheses_it_needs() {
        let src = "module m\nfn f(x: Int, a: Int, b: Int, c: Int, n: Int) -> Bool {\n    var m = 0\n    m|=1<<n\n    m &= ~ (a+b)\n    m ^= (a | b)\n    m<<=2\n    m >>= (n)\n    let p = (a|b)&c\n    let q = a|(b&c)\n    let r = (1<<n)-1\n    let s = 1<<(n-1)\n    let t = a<<(b<<c)\n    let u = (a<<b)<<c\n    let v = - ~a\n    let w = (a ^ b) | (a & b) | (a >> 1)\n    for i in 0..(1<<n) {\n        m ^= i\n    }\n    (x&1)==0\n}\n";
        let out = laws(src);
        for line in [
            "    m |= 1 << n\n",
            "    m &= ~(a + b)\n",
            "    m ^= a | b\n",
            "    m <<= 2\n",
            "    m >>= n\n",
            "    let p = (a | b) & c\n",
            "    let q = a | b & c\n",
            "    let r = (1 << n) - 1\n",
            "    let s = 1 << n - 1\n",
            "    let t = a << (b << c)\n",
            "    let u = a << b << c\n",
            "    let v = -~a\n",
            "    let w = a ^ b | a & b | a >> 1\n",
            "    for i in 0..1 << n {\n",
            "    x & 1 == 0\n",
        ] {
            assert!(out.contains(line), "expected {line:?} in:\n{out}");
        }
        assert!(!out.contains("m = m"), "the parser's rewrite is never printed:\n{out}");
    }

    /// FMT-INT-BASE-1 (RW 2.35): an `Int` literal written in hexadecimal, octal or binary, or with its digits grouped by
    /// `_`, prints as it was written — its base's letter in lower case, its digits and their grouping untouched —
    /// wherever a literal can stand: a value, a pattern, an index, a compound target, a flat block, an f-string's hole.
    /// It printed as its value in decimal (`0xFF00` as `65280`). A literal in plain decimal digits is unchanged, and so
    /// is every literal's value (the identity law).
    #[test]
    fn an_int_literal_keeps_the_base_it_was_written_in() {
        let src = "module m\nlet MASK: Int = 0XFF00\nfn f(xs: ref List[Int], n: Int) -> Str {\n    let a = 0xdead_BEEF + 0b1010_0101 + 0o755 + 1_000_000 + 255 + 0B11 + 0O17\n    xs[0x1F] |= 0b1\n    let ys = xs.map(fn(i: Int) -> Int { let k = i & 0xF; k | 0b1000 })\n    let t = match n {\n        0x10 => \"sixteen\"\n        0b11 => \"three\"\n        7 => \"seven\"\n        _ => f\"{n & 0xFF} of {0o17}\"\n    }\n    t\n}\n";
        let out = laws(src);
        for part in [
            "let MASK: Int = 0xFF00\n",
            "    let a = 0xdead_BEEF + 0b1010_0101 + 0o755 + 1_000_000 + 255 + 0b11 + 0o17\n",
            "    xs[0x1F] |= 0b1\n",
            "{ let k = i & 0xF; k | 0b1000 }",
            "        0x10 => \"sixteen\"\n",
            "        0b11 => \"three\"\n",
            "        7 => \"seven\"\n",
            "f\"{n & 0xFF} of {0o17}\"",
        ] {
            assert!(out.contains(part), "expected {part:?} in:\n{out}");
        }
        for decimal in ["65280", "3735928559", "165", "493", "1000000", "31"] {
            assert!(!out.contains(decimal), "a literal's value in decimal ({decimal}) must not replace its spelling:\n{out}");
        }
        // Outside a `format_source` call nothing is remembered: the spellings of one source never reach another's.
        assert!(int_spelling(delulu_diag::Span::new(0, 0, 4)).is_none());
        let plain = laws("module m\nfn f() -> Int {\n    65280 + 0\n}\n");
        assert!(plain.contains("    65280 + 0\n"), "{plain}");
    }
}
