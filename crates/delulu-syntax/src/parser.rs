//! The Stage-1 parser (spec §3): error-recovering recursive descent with a Pratt
//! core for expressions. On error it records a diagnostic and resynchronizes at
//! the next statement/item boundary, so one run can surface many diagnostics.
//!
//! Reserved-word policy (§2.3 precision): `expect_decl_name` rejects reserved
//! words (DL0106); member names after `.` never pass through it, so `root.secret(…)`
//! parses while `fn secret()` does not.

use delulu_diag::{Confidence, Diagnostic, Edit, FileId, Repair, Span};

use crate::ast::*;
use crate::token::{is_reserved, Token, TokenKind};

pub fn parse(file: FileId, tokens: Vec<Token>) -> (Module, Vec<Diagnostic>) {
    let mut p = Parser::new(file, tokens);
    let module = p.parse_module();
    (module, p.diags)
}

/// Parse a standalone type expression — **the ordinary type grammar**, nothing more (Stage 6:
/// plugin-manifest export signature strings are parsed with exactly this, spec §2.1). The whole
/// token stream must be one type: trailing input is an error. Errors are ordinary parse
/// diagnostics; the caller (the plugin manifest checker) maps them to DL1501.
pub fn parse_type_expr(file: FileId, tokens: Vec<Token>) -> (TypeExpr, Vec<Diagnostic>) {
    let mut p = Parser::new(file, tokens);
    let ty = p.parse_type();
    // Statement terminators the lexer may have inserted at end-of-input are not "trailing input".
    while p.eat(&TokenKind::Term) {}
    if !p.at_eof() {
        let span = p.span();
        p.error(
            "DL0201",
            format!("unexpected input after the type: found {}", p.peek().describe()),
            span,
            "a signature string must be a single type",
        );
    }
    (ty, p.diags)
}

struct Parser {
    #[allow(dead_code)]
    file: FileId,
    tokens: Vec<Token>,
    pos: usize,
    next_node: u32,
    diags: Vec<Diagnostic>,
    /// Set once per resync episode so we don't emit a cascade for one mistake.
    panicking: bool,
}

impl Parser {
    fn new(file: FileId, tokens: Vec<Token>) -> Self {
        Parser { file, tokens, pos: 0, next_node: 0, diags: Vec::new(), panicking: false }
    }

    // ----- node ids and cursor --------------------------------------------

    fn node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node);
        self.next_node += 1;
        id
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_at(&self, ahead: usize) -> &TokenKind {
        let i = (self.pos + ahead).min(self.tokens.len() - 1);
        &self.tokens[i].kind
    }

    fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn prev_span(&self) -> Span {
        self.tokens[self.pos.saturating_sub(1)].span
    }

    fn at(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    /// True when the cursor is on an identifier token equal to `word` — the test for a
    /// *contextual* keyword (`foreign`, `lib`, `as`), which stays lexed as an identifier so it
    /// remains usable as a member name (`root.foreign`, `x.lib`).
    fn at_kw_ident(&self, word: &str) -> bool {
        matches!(self.peek(), TokenKind::Ident(n) if n == word)
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if !self.at_eof() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn error(&mut self, code: &'static str, msg: impl Into<String>, span: Span, label: impl Into<String>) {
        if self.panicking {
            return;
        }
        self.panicking = true;
        self.diags.push(Diagnostic::error(code, msg).with_span(span, label));
    }

    fn expect(&mut self, kind: TokenKind) -> bool {
        if self.at(&kind) {
            self.bump();
            self.panicking = false;
            true
        } else {
            let found = self.peek().describe();
            let want = kind.describe();
            self.error(
                "DL0201",
                format!("expected {want}, found {found}"),
                self.span(),
                format!("expected {want} here"),
            );
            false
        }
    }

    /// Consume one statement terminator (`;` or inserted). Tolerant: a closing
    /// brace or EOF also ends a statement without an explicit terminator.
    fn expect_term(&mut self) {
        if self.eat(&TokenKind::Term) {
            self.panicking = false;
            return;
        }
        if matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            return;
        }
        self.error("DL0209", "expected end of statement", self.span(), "expected a newline or `;`");
        self.recover_stmt();
    }

    /// Expect an identifier that introduces a NEW name; reject reserved words (DL0106).
    fn expect_decl_name(&mut self) -> Ident {
        match self.peek().clone() {
            // Stage 7: `consume`/`recover` are now keywords; a declaration named after one is
            // pre-0.7 code — DL1608 with the exact rename repair, recovering as the identifier
            // so the rest of the declaration still parses.
            k @ (TokenKind::KwConsume | TokenKind::KwRecover) => {
                let span = self.span();
                let name = k.keyword_lexeme().unwrap().to_string();
                self.bump();
                self.panicking = false;
                self.dl1608(&name, span);
                Ident { name, span }
            }
            TokenKind::Ident(name) => {
                let span = self.span();
                self.bump();
                self.panicking = false;
                if is_reserved(&name) {
                    self.diags.push(
                        Diagnostic::error(
                            "DL0106",
                            format!("`{name}` is reserved for a future stage and cannot be a declared name"),
                        )
                        .with_span(span, "reserved word"),
                    );
                }
                Ident { name, span }
            }
            other => {
                let span = self.span();
                self.error("DL0201", format!("expected a name, found {}", other.describe()), span, "name expected here");
                Ident { name: "<error>".into(), span }
            }
        }
    }

    /// A member name after `.` — reserved words are fine here: member position is not a declaration
    /// site, so a keyword lexeme (`py.import`, spec §5.2) is an unambiguous member name.
    fn expect_member_name(&mut self) -> Ident {
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                let span = self.span();
                self.bump();
                self.panicking = false;
                Ident { name, span }
            }
            ref other if other.keyword_lexeme().is_some() => {
                let name = other.keyword_lexeme().unwrap().to_string();
                let span = self.span();
                self.bump();
                self.panicking = false;
                Ident { name, span }
            }
            other => {
                let span = self.span();
                self.error("DL0201", format!("expected a member name, found {}", other.describe()), span, "member name expected");
                Ident { name: "<error>".into(), span }
            }
        }
    }

    // ----- resynchronization ----------------------------------------------

    fn recover_stmt(&mut self) {
        while !self.at_eof() {
            if self.eat(&TokenKind::Term) {
                return;
            }
            if matches!(self.peek(), TokenKind::RBrace) {
                return;
            }
            self.bump();
        }
    }

    fn recover_item(&mut self) {
        while !self.at_eof() {
            if self.at_kw_ident("foreign") {
                return;
            }
            if matches!(
                self.peek(),
                TokenKind::KwFn
                    | TokenKind::KwType
                    | TokenKind::KwEffect
                    | TokenKind::KwPub
                    | TokenKind::KwLet
                    | TokenKind::KwImport
                    | TokenKind::KwActor
            ) {
                return;
            }
            self.bump();
        }
    }

    // ----- module ----------------------------------------------------------

    fn parse_module(&mut self) -> Module {
        // `module` header (§3). Recoverable if missing.
        let name = if self.eat(&TokenKind::KwModule) {
            let path = self.parse_path();
            self.expect_term();
            path
        } else {
            self.error("DL0204", "file must begin with a `module` declaration", self.span(), "add `module <name>` here");
            self.panicking = false;
            Path { segs: vec![Ident { name: "<missing>".into(), span: self.span() }] }
        };

        let mut imports = Vec::new();
        loop {
            while self.eat(&TokenKind::Term) {}
            // An import is `import …` or `pub import …`. A bare `pub` followed by anything else
            // (e.g. `pub fn`) is an item, not an import — that ends the import section.
            let is_import = self.at(&TokenKind::KwImport)
                || (self.at(&TokenKind::KwPub) && *self.peek_at(1) == TokenKind::KwImport);
            if !is_import {
                break;
            }
            if let Some(im) = self.parse_import() {
                imports.push(im);
            }
        }

        let mut items = Vec::new();
        while !self.at_eof() {
            // Skip statement terminators inserted after item-closing braces (§2.2).
            if self.eat(&TokenKind::Term) {
                continue;
            }
            match self.parse_item() {
                Some(item) => items.push(item),
                None => {
                    if !self.at_eof() {
                        self.error("DL0208", "expected an item (`fn`, `type`, `effect`, `actor`, `let`, or `pub`)", self.span(), "not an item");
                        self.recover_item();
                    }
                }
            }
        }

        Module { name, imports, items }
    }

    fn parse_import(&mut self) -> Option<Import> {
        let start = self.span();
        let public = self.eat(&TokenKind::KwPub); // `pub import` re-exports (§2)
        self.bump(); // import
        let path = self.parse_path();
        let alias = if let TokenKind::Ident(a) = self.peek().clone() {
            if a == "as" {
                self.bump();
                Some(self.expect_decl_name())
            } else {
                None
            }
        } else {
            None
        };
        let span = start.to(self.prev_span());
        self.expect_term();
        Some(Import { public, path, alias, span })
    }

    fn parse_path(&mut self) -> Path {
        let mut segs = Vec::new();
        segs.push(self.expect_member_name());
        while self.at(&TokenKind::Dot) {
            // lookahead: `.` then ident continues a path
            if let TokenKind::Ident(_) = self.peek_at(1) {
                self.bump(); // dot
                segs.push(self.expect_member_name());
            } else {
                break;
            }
        }
        Path { segs }
    }

    // ----- items ------------------------------------------------------------

    fn parse_item(&mut self) -> Option<Item> {
        let public = self.eat(&TokenKind::KwPub);
        // `foreign` is an active *contextual* keyword (spec §2): still lexed as an identifier so
        // `root.foreign(…)` stays legal, but recognized here as `foreign STRING lib IDENT { … }`.
        if self.at_kw_ident("foreign") {
            return Some(Item::Foreign(self.parse_foreign_decl(public)));
        }
        // `test` is a keyword only in item position (Stage 8, spec §2): still lexed as an
        // identifier — `let test = 1` and `fn test()` stay legal — but a bare `test` here can
        // only start a test block (no other item begins with an identifier).
        if self.at_kw_ident("test") {
            return Some(Item::Test(self.parse_test_decl(public)));
        }
        match self.peek() {
            TokenKind::KwFn => Some(Item::Fn(self.parse_fn(public))),
            TokenKind::KwType => Some(Item::Type(self.parse_type_decl(public))),
            TokenKind::KwEffect => Some(Item::Effect(self.parse_effect_decl(public))),
            TokenKind::KwActor => Some(Item::Actor(self.parse_actor_decl(public))),
            TokenKind::KwLet => Some(Item::Const(self.parse_const(public))),
            TokenKind::KwVar => {
                // Module-level mutable state is forbidden (§5.5, audit closes the ambient
                // laundering channel). Diagnose specifically, then recover by skipping it.
                let span = self.span();
                self.diags.push(
                    Diagnostic::error("DL0305", "module-level mutable state (`var`) is forbidden")
                        .with_span(span, "modules may only hold `let` constants of pure values"),
                );
                self.bump();
                self.recover_stmt();
                None
            }
            _ => {
                if public {
                    self.error("DL0208", "expected an item after `pub`", self.span(), "expected `fn`, `type`, `effect`, or `let`");
                }
                None
            }
        }
    }

    /// `foreign_decl = "foreign" STRING "lib" IDENT "{" { foreign_fn } "}"` (spec §2).
    fn parse_foreign_decl(&mut self, public: bool) -> ForeignDecl {
        let start = self.span();
        self.bump(); // `foreign` (identifier token used as a contextual keyword)

        // The ABI string.
        let mut abi = String::new();
        let mut abi_present = false;
        let abi_span;
        match self.peek().clone() {
            TokenKind::Str(s) => {
                abi_span = self.span();
                self.bump();
                self.panicking = false;
                abi = s;
                abi_present = true;
            }
            other => {
                abi_span = self.span();
                self.error(
                    "DL0201",
                    format!("expected an ABI string after `foreign`, found {}", other.describe()),
                    abi_span,
                    "expected a string like `\"c\"`",
                );
            }
        }

        // The `lib` contextual keyword.
        if self.at_kw_ident("lib") {
            self.bump();
            self.panicking = false;
        } else {
            self.error(
                "DL0201",
                format!("expected `lib` after the ABI string, found {}", self.peek().describe()),
                self.span(),
                "expected `lib` here",
            );
        }

        let name = self.expect_decl_name();

        // DL1308: `"c"` is the only ABI supported in v0.4.
        if abi_present && abi != "c" {
            self.diags.push(
                Diagnostic::error("DL1308", format!("unsupported ABI `\"{abi}\"` — v0.4 supports only the C ABI (`\"c\"`)"))
                    .with_span(abi_span, "only `\"c\"` is supported here"),
            );
        }

        // The block body: zero or more foreign functions.
        self.expect(TokenKind::LBrace);
        let mut fns = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at_eof() {
            if self.eat(&TokenKind::Term) {
                continue;
            }
            let before = self.pos;
            if let Some(ff) = self.parse_foreign_fn() {
                fns.push(ff);
            }
            if self.pos == before {
                // No progress — force one to avoid an infinite loop.
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace);
        let span = start.to(self.prev_span());
        self.expect_term();
        ForeignDecl { public, abi, abi_span, name, fns, id: self.node_id(), span }
    }

    /// `foreign_fn = "fn" IDENT "(" [params] ")" [ "->" type ]` — deliberately no effect row:
    /// a foreign function's row is implicitly `!{ForeignCall}`, always (spec §2). A `!` row here
    /// is a parse error.
    fn parse_foreign_fn(&mut self) -> Option<ForeignFn> {
        if !self.at(&TokenKind::KwFn) {
            self.error(
                "DL0201",
                format!("expected a foreign `fn` declaration, found {}", self.peek().describe()),
                self.span(),
                "expected `fn` or `}` here",
            );
            self.recover_stmt();
            return None;
        }
        let start = self.span();
        self.bump(); // fn
        let name = self.expect_decl_name();
        let params = self.parse_params();
        let ret = if self.eat(&TokenKind::Arrow) { Some(self.parse_type()) } else { None };
        // No effect-row syntax: the row is always `!{ForeignCall}` (spec §2, invariant 19).
        if self.at(&TokenKind::Bang) {
            let bang = self.span();
            self.error(
                "DL0201",
                "a foreign function has no effect row — its row is always `! {ForeignCall}`",
                bang,
                "remove this effect row",
            );
            let _ = self.parse_opt_row(); // consume it so parsing recovers
        }
        let span = start.to(self.prev_span());
        self.expect_term();
        Some(ForeignFn { name, params, ret, span })
    }

    /// `actor_decl = "actor" IDENT [generics] "{" { actor_member } "}"` (Stage 7, spec §2).
    /// `be`, `new` are contextual keywords INSIDE actor bodies only — top-level code may still
    /// use them as identifiers (invariant 37 needs no migration for them). Exactly one `new`
    /// per actor: zero synthesizes an empty one (so the checker always has a constructor node),
    /// extras are diagnosed and dropped.
    fn parse_actor_decl(&mut self, public: bool) -> ActorDecl {
        let start = self.span();
        self.bump(); // actor
        let name = self.expect_decl_name();
        let generics = self.parse_generics();
        self.expect(TokenKind::LBrace);
        let mut fields = Vec::new();
        let mut ctors: Vec<CtorDecl> = Vec::new();
        let mut behaviors = Vec::new();
        let mut fns = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at_eof() {
            if self.eat(&TokenKind::Term) {
                continue;
            }
            let before = self.pos;
            match self.peek().clone() {
                TokenKind::KwLet | TokenKind::KwVar => {
                    let mutable = matches!(self.peek(), TokenKind::KwVar);
                    let fstart = self.span();
                    self.bump();
                    let fname = self.expect_decl_name();
                    self.expect(TokenKind::Colon);
                    let ty = self.parse_type();
                    let span = fstart.to(self.prev_span());
                    // Grammar: fields carry no initializer — they are assigned in `new`.
                    if self.at(&TokenKind::Eq) {
                        self.error(
                            "DL0201",
                            "actor fields have no initializer — assign them in `new`",
                            self.span(),
                            "remove the `= …` and initialize in the constructor",
                        );
                        self.recover_stmt();
                    } else {
                        self.expect_term();
                    }
                    fields.push(ActorField { mutable, name: fname, ty, span });
                }
                TokenKind::Ident(ref n) if n == "new" && matches!(self.peek_at(1), TokenKind::LParen) => {
                    let cstart = self.span();
                    self.bump(); // new
                    let params = self.parse_params();
                    let row = self.parse_opt_row();
                    let body = self.parse_block();
                    let span = cstart.to(self.prev_span());
                    ctors.push(CtorDecl { params, row, body, id: self.node_id(), span });
                }
                TokenKind::Ident(ref n) if n == "be" && matches!(self.peek_at(1), TokenKind::Ident(_)) => {
                    let bstart = self.span();
                    self.bump(); // be
                    let bname = self.expect_decl_name();
                    let params = self.parse_params();
                    // Behaviors have no return type (spec §2): they yield `Unit` at the send
                    // site. DL1606 with the exact delete repair.
                    if self.at(&TokenKind::Arrow) {
                        let arrow = self.span();
                        self.bump();
                        let ty = self.parse_type();
                        let bad = arrow.to(ty.span());
                        self.diags.push(
                            Diagnostic::error(
                                "DL1606",
                                format!(
                                    "behavior `{}` declares a return type — behaviors yield `Unit` at the send site",
                                    bname.name
                                ),
                            )
                            .with_span(bad, "delete the return type")
                            .with_repair(Repair {
                                id: "delete-behavior-return-type",
                                confidence: Confidence::Exact,
                                authority_widening: false,
                                requires_human: false,
                                edits: vec![Edit {
                                    file: bad.file,
                                    start_byte: bad.start,
                                    end_byte: bad.end,
                                    insert: String::new(),
                                }],
                            }),
                        );
                    }
                    let row = self.parse_opt_row();
                    let body = self.parse_block();
                    let span = bstart.to(self.prev_span());
                    behaviors.push(BehaviorDecl { name: bname, params, row, body, id: self.node_id(), span });
                }
                TokenKind::KwFn => {
                    fns.push(self.parse_fn(false));
                }
                other => {
                    self.error(
                        "DL0201",
                        format!(
                            "expected an actor member (`let`/`var` field, `new`, `be`, or `fn`), found {}",
                            other.describe()
                        ),
                        self.span(),
                        "not an actor member",
                    );
                    self.recover_stmt();
                }
            }
            if self.pos == before {
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace);
        let span = start.to(self.prev_span());
        let ctor = match ctors.len() {
            0 => {
                self.panicking = false;
                self.error(
                    "DL0201",
                    format!("actor `{}` must declare exactly one `new` constructor", name.name),
                    span,
                    "add `new(…) { … }`",
                );
                self.panicking = false;
                CtorDecl {
                    params: Vec::new(),
                    row: None,
                    body: Block { stmts: Vec::new(), id: self.node_id(), span },
                    id: self.node_id(),
                    span,
                }
            }
            1 => ctors.pop().unwrap(),
            _ => {
                let extra = ctors[1].span;
                self.panicking = false;
                self.error(
                    "DL0201",
                    format!("actor `{}` declares more than one `new` — exactly one is allowed", name.name),
                    extra,
                    "remove this constructor",
                );
                self.panicking = false;
                ctors.into_iter().next().unwrap()
            }
        };
        self.expect_term();
        ActorDecl { public, name, generics, fields, ctor, behaviors, fns, id: self.node_id(), span }
    }

    /// Invariant 37 (DL1608): pre-0.7 code using `consume`/`recover` as an identifier gets an
    /// exact rename repair (`consume` → `consume_`), applied corpus-wide by
    /// `delulu fmt --migrate 0.7`. Never breakage-by-surprise.
    fn dl1608(&mut self, word: &str, span: Span) {
        let renamed = format!("{word}_");
        self.diags.push(
            Diagnostic::error(
                "DL1608",
                format!("`{word}` is a keyword in v0.7 and can no longer be used as an identifier"),
            )
            .with_span(span, format!("rename to `{renamed}` (or run `delulu fmt --migrate 0.7`)"))
            .with_repair(Repair {
                id: "rename-v07-keyword",
                confidence: Confidence::Exact,
                authority_widening: false,
                requires_human: false,
                edits: vec![Edit {
                    file: span.file,
                    start_byte: span.start,
                    end_byte: span.end,
                    insert: renamed,
                }],
            }),
        );
    }

    fn parse_generics(&mut self) -> Vec<Ident> {
        let mut generics = Vec::new();
        if self.eat(&TokenKind::LBracket) {
            if !self.at(&TokenKind::RBracket) {
                loop {
                    generics.push(self.expect_decl_name());
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBracket);
        }
        generics
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.expect(TokenKind::LParen);
        if !self.at(&TokenKind::RParen) {
            loop {
                let name = self.expect_decl_name();
                self.expect(TokenKind::Colon);
                let ty = self.parse_type();
                params.push(Param { name, ty });
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.at(&TokenKind::RParen) {
                    break; // trailing comma
                }
            }
        }
        self.expect(TokenKind::RParen);
        params
    }

    fn parse_fn(&mut self, public: bool) -> FnDecl {
        let start = self.span();
        self.bump(); // fn
        let name = self.expect_decl_name();
        let generics = self.parse_generics();
        let params = self.parse_params();
        let ret = if self.eat(&TokenKind::Arrow) { Some(self.parse_type()) } else { None };
        let row = self.parse_opt_row();
        let body = self.parse_block();
        let span = start.to(self.prev_span());
        FnDecl { public, name, generics, params, ret, row, body, id: self.node_id(), span }
    }

    /// `test_decl = "test" STRING [effect_row] block` (Stage 8, spec §2). The grammar keeps
    /// test blocks OUTSIDE the `[pub]` group — a test is not an exported item, so `pub test`
    /// is diagnosed and the `pub` discarded (parse recovery keeps the block itself).
    fn parse_test_decl(&mut self, public: bool) -> TestDecl {
        let start = self.span();
        self.bump(); // `test` (identifier token used as a contextual keyword)
        if public {
            self.error(
                "DL0208",
                "`test` blocks cannot be `pub`",
                start,
                "tests are not exported items — remove `pub`",
            );
        }
        let (name, name_span) = match self.peek().clone() {
            TokenKind::Str(s) => {
                let sp = self.span();
                self.bump();
                self.panicking = false;
                (s, sp)
            }
            other => {
                let sp = self.span();
                self.error(
                    "DL0201",
                    format!("expected a test name string after `test`, found {}", other.describe()),
                    sp,
                    "expected a string like `\"adds correctly\"`",
                );
                (String::new(), sp)
            }
        };
        let row = self.parse_opt_row();
        let body = self.parse_block();
        let span = start.to(self.prev_span());
        TestDecl { name, name_span, row, body, id: self.node_id(), span }
    }

    fn parse_const(&mut self, public: bool) -> ConstDecl {
        let start = self.span();
        self.bump(); // let
        let name = self.expect_decl_name();
        let ty = if self.eat(&TokenKind::Colon) { Some(self.parse_type()) } else { None };
        self.expect(TokenKind::Eq);
        let value = self.parse_expr();
        let span = start.to(self.prev_span());
        self.expect_term();
        ConstDecl { public, name, ty, value, id: self.node_id(), span }
    }

    fn parse_effect_decl(&mut self, public: bool) -> EffectDecl {
        let start = self.span();
        self.bump(); // effect
        let name = self.expect_decl_name();
        let span = start.to(self.prev_span());
        self.expect_term();
        EffectDecl { public, name, id: self.node_id(), span }
    }

    fn parse_type_decl(&mut self, public: bool) -> TypeDecl {
        let start = self.span();
        self.bump(); // type
        let name = self.expect_decl_name();
        let generics = self.parse_generics();
        let kind = if self.at(&TokenKind::LBrace) {
            // record
            self.bump();
            let mut fields = Vec::new();
            if !self.at(&TokenKind::RBrace) {
                loop {
                    let fname = self.expect_decl_name();
                    self.expect(TokenKind::Colon);
                    let ty = self.parse_type();
                    fields.push(FieldDef { name: fname, ty });
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                    if self.at(&TokenKind::RBrace) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBrace);
            TypeDeclKind::Record(fields)
        } else if self.eat(&TokenKind::Eq) {
            // sum (one or more `|`-separated variants) OR alias (a single type)
            if self.looks_like_variant() {
                let mut variants = Vec::new();
                loop {
                    variants.push(self.parse_variant());
                    if !self.eat(&TokenKind::Pipe) {
                        break;
                    }
                }
                TypeDeclKind::Sum(variants)
            } else {
                TypeDeclKind::Alias(self.parse_type())
            }
        } else {
            self.error("DL0201", "expected `{` (record) or `=` (sum or alias) in type declaration", self.span(), "here");
            TypeDeclKind::Record(Vec::new())
        };
        let span = start.to(self.prev_span());
        self.expect_term();
        TypeDecl { public, name, generics, kind, id: self.node_id(), span }
    }

    /// A variant starts with a capitalized-or-any identifier optionally followed
    /// by `(`; a sum has at least one, and multiple are `|`-separated. We treat
    /// `Ident` or `Ident(` at the head, with a following `|` anywhere, as a sum;
    /// otherwise the RHS is an alias. To keep this decidable, the rule is:
    /// an identifier immediately followed by `(` or `|` (or end) is a variant list.
    fn looks_like_variant(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(_))
            && matches!(self.peek_at(1), TokenKind::LParen | TokenKind::Pipe | TokenKind::Term | TokenKind::Eof)
    }

    fn parse_variant(&mut self) -> VariantDef {
        let name = self.expect_member_name();
        let mut fields = Vec::new();
        if self.eat(&TokenKind::LParen) {
            if !self.at(&TokenKind::RParen) {
                loop {
                    fields.push(self.parse_type());
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen);
        }
        VariantDef { name, fields }
    }

    // ----- types ------------------------------------------------------------

    /// True when the token `ahead` positions away can begin a type.
    fn type_starts_at(&self, ahead: usize) -> bool {
        matches!(self.peek_at(ahead), TokenKind::Ident(_) | TokenKind::KwFn | TokenKind::LParen)
    }

    fn parse_type(&mut self) -> TypeExpr {
        // Stage 7: an optional rcap prefix (`iso T`, `val fn(val T) -> …`). Contextual: the six
        // words are RESERVED identifiers, recognized here only when a type follows (build-order
        // deviation 5) — so `x: iso List[Int]` works while no expression position changes.
        if let TokenKind::Ident(name) = self.peek() {
            if let Some(rcap) = Rcap::from_name(name) {
                if self.type_starts_at(1) {
                    let start = self.span();
                    self.bump();
                    // `iso val T` — one prefix only; diagnose and skip the extras.
                    while let TokenKind::Ident(n2) = self.peek() {
                        if Rcap::from_name(n2).is_some() && self.type_starts_at(1) {
                            self.error(
                                "DL0201",
                                "only one reference capability may prefix a type",
                                self.span(),
                                "remove this capability",
                            );
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let inner = self.parse_type_core();
                    let span = start.to(self.prev_span());
                    return TypeExpr::Rcap { rcap, inner: Box::new(inner), span };
                }
            }
        }
        self.parse_type_core()
    }

    fn parse_type_core(&mut self) -> TypeExpr {
        // A token that cannot begin a type gets the type-position diagnostic (DL0203), not the
        // generic "expected a member name" the path parser would otherwise produce. Reported here
        // because this is the only place that knows we are in type position at all.
        if !self.type_starts_at(0) {
            let span = self.span();
            let found = self.peek().describe();
            self.error("DL0203", format!("expected a type, found {found}"), span, "type expected");
            return TypeExpr::Named {
                path: Path { segs: vec![Ident { name: "<error>".into(), span }] },
                args: Vec::new(),
                span,
            };
        }
        if self.at(&TokenKind::KwFn) {
            let start = self.span();
            self.bump();
            self.expect(TokenKind::LParen);
            let mut params = Vec::new();
            if !self.at(&TokenKind::RParen) {
                loop {
                    params.push(self.parse_type());
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen);
            let ret = if self.eat(&TokenKind::Arrow) { Some(Box::new(self.parse_type())) } else { None };
            let row = self.parse_opt_row();
            let span = start.to(self.prev_span());
            TypeExpr::Fn { params, ret, row, span }
        } else if self.at(&TokenKind::LParen) {
            self.bump();
            let inner = self.parse_type();
            self.expect(TokenKind::RParen);
            inner
        } else {
            let start = self.span();
            let path = self.parse_path();
            let mut args = Vec::new();
            if self.eat(&TokenKind::LBracket) {
                if !self.at(&TokenKind::RBracket) {
                    loop {
                        args.push(self.parse_type());
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket);
            }
            let span = start.to(self.prev_span());
            TypeExpr::Named { path, args, span }
        }
    }

    fn parse_opt_row(&mut self) -> Option<RowExpr> {
        if !self.at(&TokenKind::Bang) {
            return None;
        }
        let start = self.span();
        self.bump(); // !
        if self.eat(&TokenKind::LBrace) {
            let mut effects = Vec::new();
            let mut tail = None;
            if !self.at(&TokenKind::RBrace) {
                loop {
                    effects.push(self.parse_path());
                    if self.eat(&TokenKind::Comma) {
                        continue;
                    }
                    if self.eat(&TokenKind::Pipe) {
                        tail = Some(self.expect_member_name());
                    }
                    break;
                }
            }
            self.expect(TokenKind::RBrace);
            let span = start.to(self.prev_span());
            Some(RowExpr { effects, tail, span })
        } else {
            // `! e` — tail-only row.
            let tail = self.expect_member_name();
            let span = start.to(self.prev_span());
            Some(RowExpr { effects: Vec::new(), tail: Some(tail), span })
        }
    }

    // ----- blocks and statements -------------------------------------------

    fn parse_block(&mut self) -> Block {
        let start = self.span();
        if !self.expect(TokenKind::LBrace) {
            return Block { stmts: Vec::new(), id: self.node_id(), span: start };
        }
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at_eof() {
            // Skip stray terminators between statements.
            if self.eat(&TokenKind::Term) {
                continue;
            }
            let before = self.pos;
            if let Some(s) = self.parse_stmt() {
                stmts.push(s);
            }
            if self.pos == before {
                // No progress — force one to avoid an infinite loop.
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace);
        let span = start.to(self.prev_span());
        Block { stmts, id: self.node_id(), span }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            TokenKind::KwLet | TokenKind::KwVar => {
                let mutable = matches!(self.peek(), TokenKind::KwVar);
                let start = self.span();
                self.bump();
                let name = self.expect_decl_name();
                let ty = if self.eat(&TokenKind::Colon) { Some(self.parse_type()) } else { None };
                self.expect(TokenKind::Eq);
                let value = self.parse_expr();
                let span = start.to(self.prev_span());
                self.expect_term();
                Some(Stmt::Let { name, ty, value, mutable, span })
            }
            TokenKind::KwWhile => {
                let start = self.span();
                self.bump();
                let cond = self.parse_expr_no_struct();
                let body = self.parse_block();
                let span = start.to(self.prev_span());
                Some(Stmt::While { cond, body, span })
            }
            TokenKind::KwReturn => {
                let start = self.span();
                self.bump();
                let value = if self.at(&TokenKind::Term) || self.at(&TokenKind::RBrace) {
                    None
                } else {
                    Some(self.parse_expr())
                };
                let span = start.to(self.prev_span());
                self.expect_term();
                Some(Stmt::Return { value, span })
            }
            _ => {
                // Expression statement OR assignment.
                let expr = self.parse_expr();
                if self.at(&TokenKind::Eq) {
                    let start = expr.span();
                    self.bump();
                    let value = self.parse_expr();
                    let span = start.to(self.prev_span());
                    self.expect_term();
                    match self.expr_to_lvalue(expr) {
                        Some(target) => Some(Stmt::Assign { target, value, span }),
                        None => {
                            self.error("DL0207", "invalid assignment target", start, "cannot assign to this expression");
                            None
                        }
                    }
                } else {
                    self.expect_term();
                    Some(Stmt::Expr(expr))
                }
            }
        }
    }

    fn expr_to_lvalue(&self, e: Expr) -> Option<LValue> {
        match e {
            Expr::Var { path, .. } if path.segs.len() == 1 => {
                Some(LValue::Var(path.segs.into_iter().next().unwrap()))
            }
            Expr::Field { recv, name, .. } => Some(LValue::Field(Box::new(self.expr_to_lvalue(*recv)?), name)),
            Expr::Index { recv, index, .. } => Some(LValue::Index(Box::new(self.expr_to_lvalue(*recv)?), *index)),
            _ => None,
        }
    }

    // ----- expressions (Pratt) ---------------------------------------------

    fn parse_expr(&mut self) -> Expr {
        self.parse_bin(0, true)
    }

    /// Condition/scrutinee context: record literals are disabled (§3), so
    /// `if p { … }` reads `p` as a variable and `{ … }` as the block.
    fn parse_expr_no_struct(&mut self) -> Expr {
        self.parse_bin(0, false)
    }

    /// Binary operators by precedence climbing. Comparison and equality are
    /// non-associative (DL0206).
    fn parse_bin(&mut self, min_prec: u8, allow_struct: bool) -> Expr {
        let mut lhs = self.parse_unary(allow_struct);
        loop {
            let (op, prec, non_assoc) = match self.binop() {
                Some(x) => x,
                None => break,
            };
            if prec < min_prec {
                break;
            }
            self.bump();
            let rhs = self.parse_bin(prec + 1, allow_struct);
            if non_assoc {
                // Guard against `a < b < c`.
                if let Some((_, next_prec, next_non)) = self.binop() {
                    if next_non && next_prec == prec {
                        self.error("DL0206", "comparison operators are non-associative", self.span(), "add parentheses");
                    }
                }
            }
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), id: self.node_id(), span };
        }
        lhs
    }

    fn binop(&self) -> Option<(BinOp, u8, bool)> {
        Some(match self.peek() {
            TokenKind::OrOr => (BinOp::Or, 1, false),
            TokenKind::AndAnd => (BinOp::And, 2, false),
            TokenKind::EqEq => (BinOp::Eq, 3, true),
            TokenKind::NotEq => (BinOp::Ne, 3, true),
            TokenKind::Lt => (BinOp::Lt, 4, true),
            TokenKind::Le => (BinOp::Le, 4, true),
            TokenKind::Gt => (BinOp::Gt, 4, true),
            TokenKind::Ge => (BinOp::Ge, 4, true),
            TokenKind::Plus => (BinOp::Add, 5, false),
            TokenKind::Minus => (BinOp::Sub, 5, false),
            TokenKind::Star => (BinOp::Mul, 6, false),
            TokenKind::Slash => (BinOp::Div, 6, false),
            TokenKind::Percent => (BinOp::Rem, 6, false),
            _ => return None,
        })
    }

    fn parse_unary(&mut self, allow_struct: bool) -> Expr {
        let start = self.span();
        match self.peek() {
            TokenKind::Minus => {
                self.bump();
                let operand = self.parse_unary(allow_struct);
                let span = start.to(operand.span());
                Expr::Unary { op: UnOp::Neg, operand: Box::new(operand), id: self.node_id(), span }
            }
            TokenKind::Bang => {
                self.bump();
                let operand = self.parse_unary(allow_struct);
                let span = start.to(operand.span());
                Expr::Unary { op: UnOp::Not, operand: Box::new(operand), id: self.node_id(), span }
            }
            // Stage 7: `consume x` — yields x's full rcap and kills the binding (spec §3).
            // Locals/params only in v0.7: `consume x.f` gets the field-consume variant of
            // DL1602 here, recovering on the base binding.
            TokenKind::KwConsume => {
                self.bump();
                if let TokenKind::Ident(_) = self.peek() {
                    let name = self.expect_member_name();
                    if self.at(&TokenKind::Dot) {
                        let dot_start = self.span();
                        while self.at(&TokenKind::Dot) && matches!(self.peek_at(1), TokenKind::Ident(_)) {
                            self.bump();
                            self.bump();
                        }
                        self.diags.push(
                            Diagnostic::error(
                                "DL1602",
                                "`consume` of a field is not supported in v0.7 — consume locals and parameters only",
                            )
                            .with_span(dot_start.to(self.prev_span()), "consume the whole binding instead"),
                        );
                    }
                    let span = start.to(self.prev_span());
                    Expr::Consume { name, id: self.node_id(), span }
                } else {
                    // Pre-0.7 identifier use (`consume(…)`, `consume = …`, bare `consume`).
                    self.dl1608("consume", start);
                    let seg = Ident { name: "consume".into(), span: start };
                    let e = Expr::Var { path: Path { segs: vec![seg] }, id: self.node_id(), span: start };
                    self.parse_postfix_on(e)
                }
            }
            // Stage 7: `recover [iso|val] { … }` — restricted environment, result lifted
            // (spec §3; default lift target iso).
            TokenKind::KwRecover => {
                self.bump();
                let mut target = None;
                let mut is_recover_form = self.at(&TokenKind::LBrace);
                if let TokenKind::Ident(n) = self.peek().clone() {
                    if let Some(r) = Rcap::from_name(&n) {
                        if matches!(self.peek_at(1), TokenKind::LBrace) {
                            let rspan = self.span();
                            self.bump();
                            if !matches!(r, Rcap::Iso | Rcap::Val) {
                                self.diags.push(
                                    Diagnostic::error(
                                        "DL1607",
                                        format!("`recover` lifts only to `iso` or `val`, not `{}`", r.name()),
                                    )
                                    .with_span(rspan, "use `iso` (the default) or `val`"),
                                );
                            }
                            target = Some(r);
                            is_recover_form = true;
                        }
                    }
                }
                if is_recover_form {
                    let body = self.parse_block();
                    let span = start.to(self.prev_span());
                    Expr::Recover { target, body, id: self.node_id(), span }
                } else {
                    // Pre-0.7 identifier use.
                    self.dl1608("recover", start);
                    let seg = Ident { name: "recover".into(), span: start };
                    let e = Expr::Var { path: Path { segs: vec![seg] }, id: self.node_id(), span: start };
                    self.parse_postfix_on(e)
                }
            }
            _ => self.parse_postfix(allow_struct),
        }
    }

    fn parse_postfix(&mut self, allow_struct: bool) -> Expr {
        let e = self.parse_primary(allow_struct);
        self.parse_postfix_on(e)
    }

    fn parse_postfix_on(&mut self, mut e: Expr) -> Expr {
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    let args = self.parse_args();
                    let span = e.span().to(self.prev_span());
                    e = Expr::Call { callee: Box::new(e), args, id: self.node_id(), span };
                }
                TokenKind::Dot => {
                    self.bump();
                    let name = self.expect_member_name();
                    if self.at(&TokenKind::LParen) {
                        let args = self.parse_args();
                        let span = e.span().to(self.prev_span());
                        e = Expr::Method { recv: Box::new(e), name, args, id: self.node_id(), span };
                    } else {
                        let span = e.span().to(name.span);
                        e = Expr::Field { recv: Box::new(e), name, id: self.node_id(), span };
                    }
                }
                TokenKind::LBracket => {
                    self.bump();
                    let index = self.parse_expr();
                    self.expect(TokenKind::RBracket);
                    let span = e.span().to(self.prev_span());
                    e = Expr::Index { recv: Box::new(e), index: Box::new(index), id: self.node_id(), span };
                }
                TokenKind::Question => {
                    self.bump();
                    let span = e.span().to(self.prev_span());
                    e = Expr::Try { inner: Box::new(e), id: self.node_id(), span };
                }
                _ => break,
            }
        }
        e
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        self.expect(TokenKind::LParen);
        if !self.at(&TokenKind::RParen) {
            loop {
                args.push(self.parse_expr());
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.at(&TokenKind::RParen) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen);
        args
    }

    fn parse_primary(&mut self, allow_struct: bool) -> Expr {
        let start = self.span();
        match self.peek().clone() {
            TokenKind::Int(v) => {
                self.bump();
                Expr::Lit { kind: LitKind::Int(v), id: self.node_id(), span: start }
            }
            TokenKind::Float(v) => {
                self.bump();
                Expr::Lit { kind: LitKind::Float(v), id: self.node_id(), span: start }
            }
            TokenKind::Str(s) => {
                self.bump();
                Expr::Lit { kind: LitKind::Str(s), id: self.node_id(), span: start }
            }
            TokenKind::KwTrue => {
                self.bump();
                Expr::Lit { kind: LitKind::Bool(true), id: self.node_id(), span: start }
            }
            TokenKind::KwFalse => {
                self.bump();
                Expr::Lit { kind: LitKind::Bool(false), id: self.node_id(), span: start }
            }
            TokenKind::LParen => {
                self.bump();
                let inner = self.parse_expr();
                self.expect(TokenKind::RParen);
                inner
            }
            TokenKind::LBracket => {
                self.bump();
                let mut items = Vec::new();
                if !self.at(&TokenKind::RBracket) {
                    loop {
                        items.push(self.parse_expr());
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                        if self.at(&TokenKind::RBracket) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket);
                let span = start.to(self.prev_span());
                Expr::List { items, id: self.node_id(), span }
            }
            TokenKind::KwIf => self.parse_if(),
            TokenKind::KwMatch => self.parse_match(),
            TokenKind::KwFn => self.parse_lambda(),
            // Stage 7: `spawn A(args)` — a primary, so `spawn A(1).ping(2)` chains a send
            // through the ordinary postfix loop (T-Spawn then T-Send).
            TokenKind::KwSpawn => {
                self.bump();
                let actor = self.parse_path();
                let args = self.parse_args();
                let span = start.to(self.prev_span());
                Expr::Spawn { actor, args, id: self.node_id(), span }
            }
            TokenKind::Ident(_) => {
                // Only a single identifier is a primary; any following `.name` is a field or
                // method access handled by `parse_postfix` (so `out.println(x)` is a Method,
                // not a dotted Var). Module-qualified names are not a Stage-1 expression form.
                let seg = self.expect_member_name();
                let path = Path { segs: vec![seg] };
                // Record literal: `Path { field: ... }` — disabled in cond/scrutinee context.
                if allow_struct && self.at(&TokenKind::LBrace) && self.looks_like_record_literal() {
                    self.bump(); // {
                    let mut fields = Vec::new();
                    if !self.at(&TokenKind::RBrace) {
                        loop {
                            let fname = self.expect_member_name();
                            self.expect(TokenKind::Colon);
                            let val = self.parse_expr();
                            fields.push((fname, val));
                            if !self.eat(&TokenKind::Comma) {
                                break;
                            }
                            if self.at(&TokenKind::RBrace) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RBrace);
                    let span = start.to(self.prev_span());
                    Expr::Record { path, fields, id: self.node_id(), span }
                } else {
                    let span = path.span();
                    Expr::Var { path, id: self.node_id(), span }
                }
            }
            other => {
                self.error("DL0202", format!("expected an expression, found {}", other.describe()), start, "expected an expression");
                // Produce a placeholder so callers keep making progress.
                Expr::Lit { kind: LitKind::Int(0), id: self.node_id(), span: start }
            }
        }
    }

    /// Distinguish `Point { x: 1 }` from `name {` that is actually a following
    /// block: a record literal has `}` immediately, or `ident :` after `{`.
    fn looks_like_record_literal(&self) -> bool {
        // self.peek() == `{`
        matches!(self.peek_at(1), TokenKind::RBrace)
            || (matches!(self.peek_at(1), TokenKind::Ident(_)) && matches!(self.peek_at(2), TokenKind::Colon))
    }

    fn parse_if(&mut self) -> Expr {
        let start = self.span();
        self.bump(); // if
        let cond = self.parse_expr_no_struct();
        let then_ = self.parse_block();
        let else_ = if self.eat(&TokenKind::KwElse) {
            if self.at(&TokenKind::KwIf) {
                Some(Box::new(self.parse_if()))
            } else {
                let b = self.parse_block();
                Some(Box::new(Expr::Block(b)))
            }
        } else {
            None
        };
        let span = start.to(self.prev_span());
        Expr::If { cond: Box::new(cond), then_, else_, id: self.node_id(), span }
    }

    fn parse_match(&mut self) -> Expr {
        let start = self.span();
        self.bump(); // match
        let scrutinee = self.parse_expr_no_struct();
        self.expect(TokenKind::LBrace);
        let mut arms = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at_eof() {
            if self.eat(&TokenKind::Term) {
                continue;
            }
            let arm_start = self.span();
            let pattern = self.parse_pattern();
            self.expect(TokenKind::FatArrow);
            let body = if self.at(&TokenKind::LBrace) {
                Expr::Block(self.parse_block())
            } else {
                self.parse_expr()
            };
            let span = arm_start.to(self.prev_span());
            arms.push(Arm { pattern, body, span });
            // Arms separated by `,` or a terminator; tolerate both.
            if !self.eat(&TokenKind::Comma) {
                self.eat(&TokenKind::Term);
            }
        }
        self.expect(TokenKind::RBrace);
        let span = start.to(self.prev_span());
        Expr::Match { scrutinee: Box::new(scrutinee), arms, id: self.node_id(), span }
    }

    fn parse_pattern(&mut self) -> Pattern {
        let start = self.span();
        match self.peek().clone() {
            TokenKind::Underscore => {
                self.bump();
                Pattern::Wildcard(start)
            }
            TokenKind::Int(v) => {
                self.bump();
                Pattern::Lit(LitKind::Int(v), start)
            }
            TokenKind::Str(s) => {
                self.bump();
                Pattern::Lit(LitKind::Str(s), start)
            }
            TokenKind::KwTrue => {
                self.bump();
                Pattern::Lit(LitKind::Bool(true), start)
            }
            TokenKind::KwFalse => {
                self.bump();
                Pattern::Lit(LitKind::Bool(false), start)
            }
            TokenKind::Ident(_) => {
                let path = self.parse_path();
                if self.at(&TokenKind::LParen) {
                    self.bump();
                    let mut fields = Vec::new();
                    if !self.at(&TokenKind::RParen) {
                        loop {
                            fields.push(self.parse_pattern());
                            if !self.eat(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen);
                    let span = start.to(self.prev_span());
                    Pattern::Variant { path, fields, span }
                } else if path.segs.len() == 1 {
                    // Convention (like Rust/Haskell): a capitalized name is a nullary variant
                    // pattern (`Red`, `None`), a lowercase name is a fresh binding (`n`, `v`).
                    // This is what makes `match c { Red => .., Green => .. }` see the variants and
                    // exhaustiveness checking work.
                    let seg = path.segs.into_iter().next().unwrap();
                    if seg.name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                        let span = seg.span;
                        Pattern::Variant { path: Path { segs: vec![seg] }, fields: Vec::new(), span }
                    } else {
                        Pattern::Bind(seg)
                    }
                } else {
                    let span = path.span();
                    Pattern::Variant { path, fields: Vec::new(), span }
                }
            }
            other => {
                self.error("DL0205", format!("expected a pattern, found {}", other.describe()), start, "pattern expected");
                Pattern::Wildcard(start)
            }
        }
    }

    fn parse_lambda(&mut self) -> Expr {
        let start = self.span();
        self.bump(); // fn
        let params = self.parse_params();
        let ret = if self.eat(&TokenKind::Arrow) { Some(self.parse_type()) } else { None };
        let row = self.parse_opt_row();
        let body = self.parse_block();
        let span = start.to(self.prev_span());
        Expr::Lambda { params, ret, row, body, id: self.node_id(), span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_src(src: &str) -> (Module, Vec<Diagnostic>) {
        let (tokens, ldiags) = lex(0, src);
        assert!(ldiags.is_empty(), "lex errors: {ldiags:?}");
        parse(0, tokens)
    }

    fn parse_ok(src: &str) -> Module {
        let (m, d) = parse_src(src);
        assert!(d.is_empty(), "unexpected parse diagnostics: {d:?}");
        m
    }

    #[test]
    fn parses_the_reference_program() {
        let src = include_str!("../../../examples/demo.delulu");
        let (m, d) = {
            let (tokens, ldiags) = lex(0, src);
            assert!(ldiags.is_empty(), "lex: {ldiags:?}");
            parse(0, tokens)
        };
        assert!(d.is_empty(), "parse diagnostics: {d:?}");
        assert_eq!(m.name.dotted(), "demo");
        assert_eq!(m.items.len(), 5);
    }

    #[test]
    fn pure_fn_has_no_row() {
        let m = parse_ok("module m\nfn f(n: Int) -> Int { n }\n");
        match &m.items[0] {
            Item::Fn(f) => {
                assert!(f.row.is_none());
                assert_eq!(f.name.name, "f");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn effect_row_parses() {
        let m = parse_ok("module m\nfn g(c: Cap[Console]) ! {Write} { c.println(\"x\") }\n");
        match &m.items[0] {
            Item::Fn(f) => {
                let row = f.row.as_ref().unwrap();
                assert_eq!(row.effects.len(), 1);
                assert_eq!(row.effects[0].dotted(), "Write");
                assert!(row.tail.is_none());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn row_polymorphism_parses() {
        let m = parse_ok("module m\nfn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e { f(x) }\n");
        match &m.items[0] {
            Item::Fn(f) => {
                assert_eq!(f.generics.len(), 3);
                let row = f.row.as_ref().unwrap();
                assert_eq!(row.tail.as_ref().unwrap().name, "e");
                assert!(row.effects.is_empty());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn reserved_word_at_decl_site_is_dl0106() {
        let (_, d) = parse_src("module m\nfn secret() { }\n");
        assert!(d.iter().any(|x| x.code == "DL0106"), "{d:?}");
    }

    #[test]
    fn reserved_word_as_member_is_fine() {
        let m = parse_ok("module m\nfn main(root: Root) ! {Read} { let k = root.secret(\"K\") }\n");
        assert_eq!(m.items.len(), 1);
    }

    #[test]
    fn if_else_and_precedence() {
        parse_ok("module m\nfn f(n: Int) -> Int { if n < 2 { n } else { f(n - 1) + f(n - 2) } }\n");
    }

    #[test]
    fn record_literal_disabled_in_condition() {
        // `if p { }` — `p` is a var, `{ }` is the then-block, no record literal.
        let m = parse_ok("module m\ntype P { x: Int }\nfn f(p: Bool) -> Int { if p { 1 } else { 2 } }\n");
        assert_eq!(m.items.len(), 2);
    }

    #[test]
    fn record_literal_in_normal_position() {
        parse_ok("module m\ntype P { x: Int }\nfn f() -> P { P { x: 1 } }\n");
    }

    #[test]
    fn sum_type_and_match() {
        parse_ok(
            "module m\ntype Color = Red | Green | Blue\nfn f(c: Color) -> Int { match c { Red => 0, Green => 1, Blue => 2 } }\n",
        );
    }

    #[test]
    fn try_operator_and_methods() {
        parse_ok("module m\nfn f(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} { let s = fs.read_text(\"a\")? }\n");
    }

    #[test]
    fn chained_comparison_is_dl0206() {
        let (_, d) = parse_src("module m\nfn f() -> Bool { 1 < 2 < 3 }\n");
        assert!(d.iter().any(|x| x.code == "DL0206"), "{d:?}");
    }

    #[test]
    fn error_recovery_surfaces_multiple_diagnostics() {
        // Two broken items; the parser should recover and report on both.
        let (_, d) = parse_src("module m\nfn a( { }\nfn b(( { }\n");
        assert!(d.len() >= 2, "expected multiple diagnostics, got {d:?}");
    }

    #[test]
    fn missing_module_header_is_dl0204() {
        let (_, d) = parse_src("fn f() { }\n");
        assert!(d.iter().any(|x| x.code == "DL0204"), "{d:?}");
    }

    // ----- Stage 4: foreign blocks -----------------------------------------

    #[test]
    fn foreign_block_parses_round_trip() {
        let m = parse_ok(
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float\n fn sqrt(x: Float) -> Float }\n",
        );
        assert_eq!(m.items.len(), 1);
        match &m.items[0] {
            Item::Foreign(fd) => {
                assert_eq!(fd.abi, "c");
                assert!(!fd.public);
                assert_eq!(fd.name.name, "mathlib");
                assert_eq!(fd.fns.len(), 2);
                assert_eq!(fd.fns[0].name.name, "cos");
                assert_eq!(fd.fns[0].params.len(), 1);
                assert_eq!(fd.fns[0].params[0].name.name, "x");
                assert!(fd.fns[0].ret.is_some());
                assert_eq!(fd.fns[1].name.name, "sqrt");
            }
            other => panic!("expected a foreign block, got {other:?}"),
        }
    }

    #[test]
    fn pub_foreign_block_and_empty_body_parse() {
        let m = parse_ok("module m\npub foreign \"c\" lib libc { }\n");
        match &m.items[0] {
            Item::Foreign(fd) => {
                assert!(fd.public);
                assert_eq!(fd.name.name, "libc");
                assert!(fd.fns.is_empty());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn non_c_abi_is_dl1308() {
        let (_, d) = parse_src("module m\nforeign \"rust\" lib r { fn f() }\n");
        assert!(d.iter().any(|x| x.code == "DL1308"), "{d:?}");
    }

    #[test]
    fn foreign_fn_with_effect_row_is_a_parse_error() {
        // A foreign fn's row is implicitly `!{ForeignCall}`; writing one is rejected at parse.
        let (_, d) = parse_src("module m\nforeign \"c\" lib l { fn f() -> Int ! {Net} }\n");
        assert!(d.iter().any(|x| x.code == "DL0201"), "{d:?}");
    }

    #[test]
    fn foreign_is_still_usable_as_a_member_name() {
        // `foreign` is a contextual keyword: `root.foreign(...)` must keep parsing as a method.
        let m = parse_ok("module m\nfn main(root: Root) { let m = root.foreign(load) }\n");
        assert_eq!(m.items.len(), 1);
    }

    // ----- Stage 7 (phase 7a): actors, rcaps, spawn/consume/recover, DL1608 ----

    #[test]
    fn actor_decl_round_trips() {
        let m = parse_ok(
            "module m\n\
             actor Counter {\n\
               var count: Int\n\
               let label: Str\n\
               new(start: Int, label: Str) { }\n\
               be add(n: Int) { }\n\
               be report(out: val Str) ! {Write} { }\n\
               fn double(n: Int) -> Int { n * 2 }\n\
             }\n",
        );
        match &m.items[0] {
            Item::Actor(a) => {
                assert_eq!(a.name.name, "Counter");
                assert_eq!(a.fields.len(), 2);
                assert!(a.fields[0].mutable && a.fields[0].name.name == "count");
                assert!(!a.fields[1].mutable && a.fields[1].name.name == "label");
                assert_eq!(a.ctor.params.len(), 2);
                assert_eq!(a.behaviors.len(), 2);
                assert_eq!(a.behaviors[0].name.name, "add");
                assert_eq!(a.behaviors[1].name.name, "report");
                assert!(a.behaviors[1].row.is_some());
                assert_eq!(a.fns.len(), 1);
                assert_eq!(a.fns[0].name.name, "double");
            }
            other => panic!("expected an actor, got {other:?}"),
        }
    }

    #[test]
    fn rcap_prefixes_parse_in_every_type_position() {
        // Param, let annotation, fn-type parameter (spec §8 writes `val fn(val T)`), generic arg.
        let m = parse_ok(
            "module m\n\
             fn f(xs: iso List[Int], s: val Str, cb: val fn(val Str) -> Unit ! e) ! e {\n\
               let b: box Int = 1\n\
             }\n",
        );
        let Item::Fn(f) = &m.items[0] else { panic!() };
        assert_eq!(f.params[0].ty.written_rcap(), Some(Rcap::Iso));
        assert_eq!(f.params[1].ty.written_rcap(), Some(Rcap::Val));
        assert_eq!(f.params[2].ty.written_rcap(), Some(Rcap::Val));
        // The fn-type's own parameter carries its rcap too.
        match f.params[2].ty.core() {
            TypeExpr::Fn { params, .. } => assert_eq!(params[0].written_rcap(), Some(Rcap::Val)),
            other => panic!("expected a fn type under the rcap, got {other:?}"),
        }
    }

    #[test]
    fn spawn_consume_recover_parse() {
        let m = parse_ok(
            "module m\n\
             fn main() {\n\
               let a = spawn m.Counter(1, \"x\")\n\
               let xs: iso List[Int] = recover { [1, 2] }\n\
               let f: val Str = recover val { \"s\" }\n\
               a.add(consume xs)\n\
             }\n",
        );
        let Item::Fn(f) = &m.items[0] else { panic!() };
        // spawn with a dotted path
        let Stmt::Let { value: Expr::Spawn { actor, args, .. }, .. } = &f.body.stmts[0] else {
            panic!("expected spawn, got {:?}", f.body.stmts[0])
        };
        assert_eq!(actor.dotted(), "m.Counter");
        assert_eq!(args.len(), 2);
        // recover default target (iso) and explicit val
        let Stmt::Let { value: Expr::Recover { target: None, .. }, .. } = &f.body.stmts[1] else {
            panic!()
        };
        let Stmt::Let { value: Expr::Recover { target: Some(Rcap::Val), .. }, .. } = &f.body.stmts[2] else {
            panic!()
        };
        // consume as a send argument
        let Stmt::Expr(Expr::Method { args: send_args, .. }) = &f.body.stmts[3] else { panic!() };
        assert!(matches!(&send_args[0], Expr::Consume { name, .. } if name.name == "xs"));
    }

    #[test]
    fn spawn_chains_into_a_send_through_postfix() {
        let m = parse_ok("module m\nfn main() { spawn Counter(0).add(1) }\n");
        let Item::Fn(f) = &m.items[0] else { panic!() };
        let Stmt::Expr(Expr::Method { recv, name, .. }) = &f.body.stmts[0] else { panic!() };
        assert_eq!(name.name, "add");
        assert!(matches!(&**recv, Expr::Spawn { .. }));
    }

    #[test]
    fn dl1608_on_consume_as_a_declared_name_with_exact_rename() {
        let (_, d) = parse_src("module m\nfn f() { let consume = 5 }\n");
        let diag = d.iter().find(|x| x.code == "DL1608").expect("DL1608 expected");
        let repair = diag.repairs.first().expect("exact rename repair expected");
        assert_eq!(repair.edits[0].insert, "consume_");
    }

    #[test]
    fn dl1608_on_recover_as_a_fn_name() {
        let (_, d) = parse_src("module m\nfn recover() { }\n");
        assert!(d.iter().any(|x| x.code == "DL1608"), "{d:?}");
    }

    #[test]
    fn dl1608_on_expression_position_use_still_parses_the_call() {
        // Pre-0.7 `consume(3)` — a call of a function named consume. DL1608 fires and the
        // call structure survives (recovery as an identifier + the ordinary postfix loop).
        let (m, d) = parse_src("module m\nfn f() -> Int { consume(3) }\n");
        assert!(d.iter().any(|x| x.code == "DL1608"), "{d:?}");
        let Item::Fn(f) = &m.items[0] else { panic!() };
        assert!(matches!(&f.body.stmts[0], Stmt::Expr(Expr::Call { .. })));
    }

    #[test]
    fn consume_and_recover_remain_member_names() {
        // Member position is not a declaration site (same rule as `py.import`).
        let m = parse_ok("module m\nfn f(q: Int) { let a = q.consume()\nlet b = q.recover(1) }\n");
        assert_eq!(m.items.len(), 1);
    }

    #[test]
    fn spawn_remains_a_member_name() {
        let m = parse_ok("module m\nfn f(q: Int) { let a = q.spawn() }\n");
        assert_eq!(m.items.len(), 1);
    }

    #[test]
    fn dl1606_behavior_return_type_with_delete_repair() {
        let (_, d) = parse_src(
            "module m\nactor A { new() { }\nbe f(x: val Str) -> Int { } }\n",
        );
        let diag = d.iter().find(|x| x.code == "DL1606").expect("DL1606 expected: {d:?}");
        let repair = diag.repairs.first().expect("delete repair expected");
        assert_eq!(repair.edits[0].insert, "");
    }

    #[test]
    fn actor_requires_exactly_one_new() {
        let (_, d) = parse_src("module m\nactor A { be f(n: Int) { } }\n");
        assert!(d.iter().any(|x| x.code == "DL0201" && x.message.contains("exactly one `new`")), "{d:?}");
        let (_, d2) = parse_src("module m\nactor A { new() { }\nnew(n: Int) { } }\n");
        assert!(d2.iter().any(|x| x.code == "DL0201" && x.message.contains("more than one")), "{d2:?}");
    }

    #[test]
    fn actor_field_initializer_is_rejected() {
        let (_, d) = parse_src("module m\nactor A { var n: Int = 3\nnew() { } }\n");
        assert!(d.iter().any(|x| x.code == "DL0201" && x.message.contains("no initializer")), "{d:?}");
    }

    #[test]
    fn double_rcap_prefix_is_an_error() {
        let (_, d) = parse_src("module m\nfn f(x: iso val Str) { }\n");
        assert!(d.iter().any(|x| x.code == "DL0201" && x.message.contains("one reference capability")), "{d:?}");
    }

    #[test]
    fn recover_lift_target_must_be_iso_or_val() {
        let (_, d) = parse_src("module m\nfn f() { let x = recover ref { 1 } }\n");
        assert!(d.iter().any(|x| x.code == "DL1607"), "{d:?}");
    }

    #[test]
    fn consume_of_a_field_is_the_dl1602_variant() {
        let (_, d) = parse_src("module m\nfn f() { let y = consume x.inner }\n");
        assert!(d.iter().any(|x| x.code == "DL1602" && x.message.contains("field")), "{d:?}");
    }

    #[test]
    fn be_and_new_stay_ordinary_identifiers_outside_actors() {
        let m = parse_ok("module m\nfn f() { let be = 1\nlet new = be + 1 }\n");
        assert_eq!(m.items.len(), 1);
    }

    #[test]
    fn rcaps_stay_reserved_as_declared_names() {
        let (_, d) = parse_src("module m\nfn iso() { }\n");
        assert!(d.iter().any(|x| x.code == "DL0106"), "{d:?}");
    }

    // ===== Stage 8, phase 8a: `test` blocks (spec §2) ==================================

    #[test]
    fn a_test_block_parses_with_name_row_and_body() {
        let m = parse_ok("module m\ntest \"adds correctly\" ! {Write} { assert(true) }\n");
        match &m.items[0] {
            Item::Test(t) => {
                assert_eq!(t.name, "adds correctly");
                let row = t.row.as_ref().expect("row");
                assert_eq!(row.effects[0].dotted(), "Write");
                assert_eq!(t.body.stmts.len(), 1);
            }
            _ => panic!("expected Item::Test"),
        }
    }

    #[test]
    fn a_rowless_test_block_parses_pure() {
        let m = parse_ok("module m\ntest \"pure\" { assert_eq(1, 1) }\n");
        match &m.items[0] {
            Item::Test(t) => assert!(t.row.is_none()),
            _ => panic!("expected Item::Test"),
        }
    }

    #[test]
    fn pub_test_is_refused_but_recovers() {
        // Grammar: test_decl sits OUTSIDE the `[pub]` group — tests are not exported items.
        let (m, d) = parse_src("module m\npub test \"t\" { assert(true) }\n");
        assert!(d.iter().any(|x| x.code == "DL0208" && x.message.contains("cannot be `pub`")), "{d:?}");
        assert!(matches!(&m.items[0], Item::Test(_)), "recovery keeps the block");
    }

    #[test]
    fn a_test_without_a_name_string_is_dl0201() {
        let (_, d) = parse_src("module m\ntest { assert(true) }\n");
        assert!(d.iter().any(|x| x.code == "DL0201" && x.message.contains("test name")), "{d:?}");
    }

    #[test]
    fn test_stays_an_ordinary_identifier_outside_item_position() {
        // `test` is contextual: fine as a fn name, a binding, and a call target.
        let m = parse_ok("module m\nfn test(n: Int) -> Int { n }\nfn f() -> Int { let test = test(1)\ntest }\n");
        assert_eq!(m.items.len(), 2);
        let Item::Fn(f) = &m.items[0] else { panic!() };
        assert_eq!(f.name.name, "test");
    }
}
