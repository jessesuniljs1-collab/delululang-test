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

/// The deepest an expression may nest before the parser refuses it with DL0210 (finding P17-F5).
///
/// # Why this number
///
/// Measured, not guessed. On the `delulu-main` thread — which is given an **explicit 512 MiB**
/// stack (`delulu/src/main.rs`) — parenthesised nesting parsed cleanly at 70,000 and overflowed the
/// stack somewhere before 100,000, i.e. roughly **5 KB of stack per level** across the
/// `parse_bin → parse_unary → parse_primary → parse_expr` cycle.
///
/// The limit must hold on the **smallest** stack that might parse, not the largest. A first attempt
/// at 1,024 was sized against that 512 MiB main thread and promptly crashed this crate's own test
/// binary with `STATUS_STACK_OVERFLOW (0xc00000fd)` — libtest threads get the ordinary **2 MiB**
/// default, and so does any LSP or tooling thread where nobody called `.stack_size`. A guard that
/// only holds on the one thread that was already generously provisioned is not a guard.
///
/// The number was then **measured against that 2 MiB floor rather than estimated**, because the
/// estimate was wrong twice. In a debug build each level of the
/// `parse_bin → parse_unary → parse_primary → parse_expr` cycle costs roughly **8 KB** of stack —
/// not the ~5 KB first guessed from the 512 MiB figure. Observed in this crate's own test binary:
/// 64 levels passed, and a limit of 256 still overflowed, because 256 × 8 KB ≈ 2 MiB is the whole
/// stack. 128 leaves about half the default stack unused, and debug frames are the worst case
/// (release frames are smaller), so sizing to debug is the conservative direction.
///
/// **This does narrow the accepted language** — a 100,000-deep chain of unary `-` used to compile
/// and now will not — which is why it is documented here and in `docs/reference/diagnostics.md`
/// rather than treated as a pure bug fix. Generated code that legitimately needs more depth should
/// emit a `let` per level instead of one nested expression.
const MAX_EXPR_DEPTH: u32 = 128;

struct Parser {
    #[allow(dead_code)]
    file: FileId,
    tokens: Vec<Token>,
    pos: usize,
    next_node: u32,
    diags: Vec<Diagnostic>,
    /// Set once per resync episode so we don't emit a cascade for one mistake.
    panicking: bool,
    /// Current expression nesting depth, bounded by [`MAX_EXPR_DEPTH`] (finding P17-F5).
    depth: u32,
}

impl Parser {
    fn new(file: FileId, tokens: Vec<Token>) -> Self {
        Parser { file, tokens, pos: 0, next_node: 0, diags: Vec::new(), panicking: false, depth: 0 }
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

    // ----- the guarded list loop -------------------------------------------

    /// The parser's one loop construct for a brace-delimited list. Every such list goes
    /// through here so the forward-progress guarantee cannot be forgotten at a new call site.
    ///
    /// Runs `step` until `close` or EOF, skipping stray terminators between elements. **If a
    /// call to `step` consumes no token, this consumes one on its behalf.** That case is not
    /// hypothetical: a failed `expect` reports its diagnostic and deliberately does *not*
    /// advance, so any `step` that cannot handle the current token consumes nothing, and an
    /// unguarded loop then spins on that token forever — pushing an element per iteration if
    /// the caller collects one.
    ///
    /// The guard was written four separate times in this file, and the one loop that lacked it
    /// (`match` arms) hung the checker on a six-line program at roughly 380 MB/s until the
    /// machine ran out of memory. Centralizing it is the fix for the class, not the instance.
    /// See `HARDENING_CAMPAIGN.md` C1.
    fn parse_until(&mut self, close: &TokenKind, mut step: impl FnMut(&mut Self)) {
        while !self.at(close) && !self.at_eof() {
            if self.eat(&TokenKind::Term) {
                continue;
            }
            let before = self.pos;
            step(self);
            if self.pos == before {
                // No progress. `step` has already reported why; take the token so the loop
                // is bounded by the token count and can never spin.
                self.bump();
            }
        }
    }

    // ----- module ----------------------------------------------------------

    fn parse_module(&mut self) -> Module {
        // Attributes on the module header (Stage 10, spec §2.2): `{ attribute } module m`.
        let attrs = self.parse_attributes();
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
        // Guarded: `parse_item` returns `None` without consuming for any token that is not an
        // item keyword, and `recover_item` deliberately stops *at* `import`/`pub` — so an
        // `import` after the first item used to leave the position unchanged on every
        // iteration and spin forever, burning a core with no allocation and no output.
        self.parse_until(&TokenKind::Eof, |p| match p.parse_item() {
            Some(item) => items.push(item),
            None => {
                if p.at(&TokenKind::KwImport) {
                    // Say the actual rule. "expected an item" is true but sends the reader
                    // looking for a typo in a line that is perfectly well-formed.
                    p.error(
                        "DL0208",
                        "`import` must appear before the first item, directly under the `module` header",
                        p.span(),
                        "move this import up to the import section",
                    );
                } else if !p.at_eof() {
                    p.error("DL0208", "expected an item (`fn`, `type`, `effect`, `actor`, `let`, or `pub`)", p.span(), "not an item");
                }
                p.recover_item();
            }
        });

        Module { name, imports, items, attrs }
    }

    // ----- attributes (Stage 10, spec §2.2 — a reserved 1.0 activation) ---------------------

    /// `attribute = "@" , IDENT , [ "(" , STRING , ")" ] ;` — zero or more, validated here
    /// against the v1.x-defined set. All attributes are HINTS (invariant 45): the parser
    /// records them and nothing downstream may change semantics because of one. Unknown names,
    /// and known names with the wrong argument shape, are DL1901 with an exact removal repair —
    /// there is no silent vendor attribute space (extensions go through RFCs).
    fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        while matches!(self.peek(), TokenKind::At) {
            attrs.push(self.parse_attribute());
            // An attribute sits on its own line in the canonical style; swallow the line
            // terminator(s) so the decl it annotates is the next thing the parser sees.
            while self.eat(&TokenKind::Term) {}
        }
        attrs
    }

    fn parse_attribute(&mut self) -> Attribute {
        let start = self.span();
        self.bump(); // @
        let name = self.expect_decl_name();
        let mut arg = None;
        if self.eat(&TokenKind::LParen) {
            match self.peek().clone() {
                TokenKind::Str(s) => {
                    self.bump();
                    arg = Some(s);
                }
                _ => {
                    self.error(
                        "DL0202",
                        "expected a string argument in the attribute",
                        self.span(),
                        "attribute arguments are string literals, like `@inline(\"never\")`",
                    );
                }
            }
            self.expect(TokenKind::RParen);
        }
        let span = start.to(self.prev_span());
        let attr = Attribute { name, arg, span };
        self.validate_attribute(&attr);
        attr
    }

    /// DL1901 when attributes precede an item kind that does not take them. The v1.x surface is
    /// exactly `fn`, `actor`, and the module header (spec §2.2); tolerating them elsewhere would
    /// quietly mint a vendor attribute space one item kind at a time.
    fn refuse_attrs_here(&mut self, attrs: &[Attribute], what: &str) {
        for a in attrs {
            self.diags.push(
                Diagnostic::error(
                    "DL1901",
                    format!("attribute `@{}` is not permitted on {what}", a.name.name),
                )
                .with_span(a.span, "attributes apply to `fn`, `actor`, and the module header in v1.x")
                .with_repair(Repair {
                    id: "remove-unknown-attribute",
                    confidence: Confidence::Exact,
                    authority_widening: false,
                    requires_human: false,
                    edits: vec![Edit {
                        file: a.span.file,
                        start_byte: a.span.start,
                        end_byte: a.span.end,
                        insert: String::new(),
                    }],
                }),
            );
        }
    }

    /// DL1901 on anything outside the v1.x-defined attribute set, with an exact removal repair.
    /// `authority_widening: false` — removing a hint never changes what a program may do,
    /// which is the invariant-45 point made mechanical.
    fn validate_attribute(&mut self, attr: &Attribute) {
        let ok = match attr.name.name.as_str() {
            "aot" | "interpret" | "jit" => attr.arg.is_none(),
            "inline" => matches!(attr.arg.as_deref(), Some("never") | Some("always")),
            _ => false,
        };
        if ok {
            return;
        }
        let shown = match &attr.arg {
            Some(a) => format!("@{}(\"{a}\")", attr.name.name),
            None => format!("@{}", attr.name.name),
        };
        self.diags.push(
            Diagnostic::error("DL1901", format!("unknown attribute `{shown}`"))
                .with_span(
                    attr.span,
                    "v1.x defines `@aot`, `@interpret`, `@jit`, and `@inline(\"never\"|\"always\")` — all hints",
                )
                .with_repair(Repair {
                    id: "remove-unknown-attribute",
                    confidence: Confidence::Exact,
                    authority_widening: false,
                    requires_human: false,
                    edits: vec![Edit {
                        file: attr.span.file,
                        start_byte: attr.span.start,
                        end_byte: attr.span.end,
                        insert: String::new(),
                    }],
                }),
        );
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
        // `{ attribute } [pub] <item>` (Stage 10, spec §2.2). Attributes attach to `fn` and
        // `actor` declarations (and the module header, handled in `parse_module`); on any other
        // item they are DL1901 — defined nowhere means permitted nowhere, no silent tolerance.
        let attrs = self.parse_attributes();
        let public = self.eat(&TokenKind::KwPub);
        // `foreign` is an active *contextual* keyword (spec §2): still lexed as an identifier so
        // `root.foreign(…)` stays legal, but recognized here as `foreign STRING lib IDENT { … }`.
        if self.at_kw_ident("foreign") {
            self.refuse_attrs_here(&attrs, "a `foreign` block");
            return Some(Item::Foreign(self.parse_foreign_decl(public)));
        }
        // `test` is a keyword only in item position (Stage 8, spec §2): still lexed as an
        // identifier — `let test = 1` and `fn test()` stay legal — but a bare `test` here can
        // only start a test block (no other item begins with an identifier).
        if self.at_kw_ident("test") {
            self.refuse_attrs_here(&attrs, "a `test` block");
            return Some(Item::Test(self.parse_test_decl(public)));
        }
        match self.peek() {
            TokenKind::KwFn => Some(Item::Fn(self.parse_fn(public, attrs))),
            TokenKind::KwType => {
                self.refuse_attrs_here(&attrs, "a `type` declaration");
                Some(Item::Type(self.parse_type_decl(public)))
            }
            TokenKind::KwEffect => {
                self.refuse_attrs_here(&attrs, "an `effect` declaration");
                Some(Item::Effect(self.parse_effect_decl(public)))
            }
            TokenKind::KwActor => Some(Item::Actor(self.parse_actor_decl(public, attrs))),
            TokenKind::KwLet => {
                self.refuse_attrs_here(&attrs, "a module constant");
                Some(Item::Const(self.parse_const(public)))
            }
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
        self.parse_until(&TokenKind::RBrace, |p| {
            if let Some(ff) = p.parse_foreign_fn() {
                fns.push(ff);
            }
        });
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
    fn parse_actor_decl(&mut self, public: bool, attrs: Vec<Attribute>) -> ActorDecl {
        let start = self.span();
        self.bump(); // actor
        let name = self.expect_decl_name();
        let generics = self.parse_generics();
        // Stage 10 (10c, spec §3): optional `( "mailbox" "=" INT )` — the actor's mailbox
        // bound. Additive grammar; `mailbox` stays an ordinary identifier everywhere else.
        let mailbox = self.parse_mailbox_clause();
        self.expect(TokenKind::LBrace);
        let mut fields = Vec::new();
        let mut ctors: Vec<CtorDecl> = Vec::new();
        let mut behaviors = Vec::new();
        let mut fns = Vec::new();
        self.parse_until(&TokenKind::RBrace, |p| {
            match p.peek().clone() {
                TokenKind::KwLet | TokenKind::KwVar => {
                    let mutable = matches!(p.peek(), TokenKind::KwVar);
                    let fstart = p.span();
                    p.bump();
                    let fname = p.expect_decl_name();
                    p.expect(TokenKind::Colon);
                    let ty = p.parse_type();
                    let span = fstart.to(p.prev_span());
                    // Grammar: fields carry no initializer — they are assigned in `new`.
                    if p.at(&TokenKind::Eq) {
                        p.error(
                            "DL0201",
                            "actor fields have no initializer — assign them in `new`",
                            p.span(),
                            "remove the `= …` and initialize in the constructor",
                        );
                        p.recover_stmt();
                    } else {
                        p.expect_term();
                    }
                    fields.push(ActorField { mutable, name: fname, ty, span });
                }
                TokenKind::Ident(ref n) if n == "new" && matches!(p.peek_at(1), TokenKind::LParen) => {
                    let cstart = p.span();
                    p.bump(); // new
                    let params = p.parse_params();
                    let row = p.parse_opt_row();
                    let body = p.parse_block();
                    let span = cstart.to(p.prev_span());
                    ctors.push(CtorDecl { params, row, body, id: p.node_id(), span });
                }
                TokenKind::Ident(ref n) if n == "be" && matches!(p.peek_at(1), TokenKind::Ident(_)) => {
                    let bstart = p.span();
                    p.bump(); // be
                    let bname = p.expect_decl_name();
                    let params = p.parse_params();
                    // Behaviors have no return type (spec §2): they yield `Unit` at the send
                    // site. DL1606 with the exact delete repair.
                    if p.at(&TokenKind::Arrow) {
                        let arrow = p.span();
                        p.bump();
                        let ty = p.parse_type();
                        let bad = arrow.to(ty.span());
                        p.diags.push(
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
                    let row = p.parse_opt_row();
                    let body = p.parse_block();
                    let span = bstart.to(p.prev_span());
                    behaviors.push(BehaviorDecl { name: bname, params, row, body, id: p.node_id(), span });
                }
                TokenKind::KwFn => {
                    // Actor-member fns take no attributes in v1.x (the documented surface is
                    // `fn`/`actor` items and the module header); an `@` here falls to the
                    // unexpected-token arm below, which is the honest refusal.
                    fns.push(p.parse_fn(false, Vec::new()));
                }
                other => {
                    p.error(
                        "DL0201",
                        format!(
                            "expected an actor member (`let`/`var` field, `new`, `be`, or `fn`), found {}",
                            other.describe()
                        ),
                        p.span(),
                        "not an actor member",
                    );
                    p.recover_stmt();
                }
            }
        });
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
        ActorDecl { public, name, generics, fields, ctor, behaviors, fns, id: self.node_id(), span, attrs, mailbox }
    }

    /// `( "mailbox" "=" INT )` after an actor's name/generics (Stage 10, 10c). A malformed
    /// clause is DL0201 with recovery past the `)`; a bound of zero is clamped by the runtime
    /// to 1 (a mailbox that can hold nothing is a mailbox nobody meant).
    fn parse_mailbox_clause(&mut self) -> Option<u64> {
        if !self.at(&TokenKind::LParen) {
            return None;
        }
        self.bump(); // (
        let mut bound = None;
        if self.at_kw_ident("mailbox") {
            self.bump();
            self.expect(TokenKind::Eq);
            match self.peek().clone() {
                TokenKind::Int(n) => {
                    self.bump();
                    bound = Some(n.max(0) as u64);
                }
                _ => {
                    self.error(
                        "DL0201",
                        "expected an integer mailbox bound",
                        self.span(),
                        "the clause is `(mailbox = N)`, like `actor A(mailbox = 10000)`",
                    );
                }
            }
        } else {
            self.error(
                "DL0201",
                "expected `mailbox` in the actor configuration clause",
                self.span(),
                "the only v1.x actor configuration is `(mailbox = N)`",
            );
        }
        self.expect(TokenKind::RParen);
        bound
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

    /// Inside a bracketed list, a newline before the closing bracket is whitespace, not a statement
    /// terminator — so skip it (`HARDENING_CAMPAIGN.md` C47b).
    ///
    /// Only ONE position needs this, and the reason is worth recording because it made the fix nine
    /// lines instead of thirty. §2.2 inserts a `Term` at a newline only when the previous token *can
    /// end a statement*, and a comma cannot — which is exactly why a multi-line list WITH a trailing
    /// comma already parsed. The stray `Term` appears in one place only: after the final element,
    /// before the closer. So `a,\n)` was always fine and `a\n)` was not, and requiring that comma was
    /// never a design decision — it was this token, unskipped.
    fn skip_terms_before_closer(&mut self) {
        while self.eat(&TokenKind::Term) {}
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
            self.skip_terms_before_closer();
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
        self.skip_terms_before_closer();
        self.expect(TokenKind::RParen);
        params
    }

    fn parse_fn(&mut self, public: bool, attrs: Vec<Attribute>) -> FnDecl {
        let start = self.span();
        self.bump(); // fn
        let name = self.expect_decl_name();
        let generics = self.parse_generics();
        let params = self.parse_params();
        let ret = if self.eat(&TokenKind::Arrow) { Some(self.parse_type()) } else { None };
        let row = self.parse_opt_row();
        let body = self.parse_block();
        let span = start.to(self.prev_span());
        FnDecl { public, name, generics, params, ret, row, body, id: self.node_id(), span, attrs }
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
            self.skip_terms_before_closer();
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

    /// Is the RHS of `type X = …` a variant list rather than an alias?
    ///
    /// **A bare `type A = B` is an ALIAS** (`HARDENING_CAMPAIGN.md` C28). It used to be read as a
    /// single-variant sum, because a lone identifier followed by a terminator matched here — and the
    /// consequences were all silent. `type Meters = Int` made `Int` a *constructor*, so
    /// `fn g() -> Meters { Int }` checked clean; a mistyped value reported `expected 'T9'`; and **no
    /// alias to a bare type name could be written at all**, since `type Meters = (Int)` — parenthesised
    /// — was the only spelling that reached the alias production. Every language with this syntax
    /// (Rust, TypeScript, Haskell) means "alias", and that is what a reader means by it.
    ///
    /// So a variant list is now signalled syntactically and only by `(` or `|`:
    ///
    /// - `type E = A | B`   → sum (a `|` follows)
    /// - `type P = Data(Int)` → sum, one variant carrying a field
    /// - `type Meters = Int` → **alias**
    ///
    /// A single field-less variant is still expressible as `type E = A()`, which is unambiguous. The
    /// decision no longer depends on name resolution, so the grammar stays context-free: what makes
    /// this a sum is a token, not whether some identifier happens to name an existing type.
    fn looks_like_variant(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(_))
            && matches!(self.peek_at(1), TokenKind::LParen | TokenKind::Pipe)
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
            self.skip_terms_before_closer();
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
            self.skip_terms_before_closer();
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
                self.skip_terms_before_closer();
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
        self.parse_until(&TokenKind::RBrace, |p| {
            if let Some(s) = p.parse_stmt() {
                stmts.push(s);
            }
        });
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
                // `let _ = expr` — evaluate and discard (campaign finding C61, ruling D63).
                //
                // `_` was refused here (DL0201 "expected a name") while being a perfectly good
                // MATCH pattern, so the language had a discard in one position and not the other.
                // The restriction prevented nothing: `let ignored = expr` already discards, so all
                // it bought was a worse name. It is bound as the ordinary name `_`, which is safe
                // because a bare `_` LEXES as `TokenKind::Underscore` and never as an `Ident` — so
                // no expression can name it. Write-only by construction rather than by rule.
                let name = if self.at(&TokenKind::Underscore) {
                    let span = self.span();
                    self.bump();
                    self.panicking = false;
                    Ident { name: "_".into(), span }
                } else {
                    self.expect_decl_name()
                };
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
        while let Some((op, prec, non_assoc)) = self.binop() {
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

    /// Expression parsing, **depth-limited** (campaign finding P17-F5).
    ///
    /// Every nested expression passes through here — `parse_bin` calls it at each precedence level,
    /// it recurses into itself for unary chains, and `parse_primary` re-enters the whole cycle for
    /// a parenthesised sub-expression. That makes it the one choke point where depth can be counted
    /// without scattering counters through the grammar.
    ///
    /// Without this guard the parser recursed without bound and the process **died**:
    ///
    /// ```text
    /// $ delulu check deep_100000.delulu     # ((((…1…)))) nested 100,000 deep, a VALID module
    /// thread 'delulu-main' (26972) has overflowed its stack
    /// exit 127
    /// ```
    ///
    /// That is worse than a rejection. The abort carries no `DL####`, no span, and nothing a caller
    /// can catch — it bypasses the diagnostic system completely. `delulu check` is the gate every
    /// other guarantee is verified through, and a gate that can be made to die instead of answering
    /// is one that can be skipped. Now the deep input gets a diagnostic like any other refusal.
    fn parse_unary(&mut self, allow_struct: bool) -> Expr {
        if self.depth >= MAX_EXPR_DEPTH {
            // Refuse to go deeper. No token is consumed: the placeholder returns through the
            // callers already on the stack, each of which unwinds normally, and `panicking`
            // suppresses the cascade of "expected `)`" that the unclosed parens would otherwise
            // produce. Progress is guaranteed because every caller either consumes a token or
            // returns — see `error_recovery_surfaces_multiple_diagnostics`.
            let start = self.span();
            self.error(
                "DL0210",
                format!("expression nests deeper than {MAX_EXPR_DEPTH} levels"),
                start,
                "simplify or split this expression",
            );
            return Expr::Lit { kind: LitKind::Int(0), id: self.node_id(), span: start };
        }
        self.depth += 1;
        let e = self.parse_unary_inner(allow_struct);
        self.depth -= 1;
        e
    }

    fn parse_unary_inner(&mut self, allow_struct: bool) -> Expr {
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

    /// Postfix chains — `f(x)`, `.m()`, `[i]` — are built **iteratively**, so they never touch the
    /// [`MAX_EXPR_DEPTH`] descent counter. They still nest the AST: each turn wraps the previous
    /// expression in a fresh `Box`. That is the second half of P17-F5, and it is why bounding
    /// recursion alone did not stop the crash.
    ///
    /// With only the descent guard in place, `((((…1…))))` at 100,000 deep parsed without
    /// recursing — the guard returned a placeholder, and this loop then read each following `(` as
    /// a **call** on it, assembling a 100,000-deep `Box` chain. Nothing overflowed while building
    /// it. The process died later, in `Drop`, which walks that chain recursively:
    ///
    /// ```text
    /// $ delulu check deep_100000.delulu     # WITH the descent guard, before this one existed
    /// thread 'delulu-main' (29028) has overflowed its stack
    /// exit 127
    /// ```
    ///
    /// Confirmed by experiment rather than inspection: `std::mem::forget`ting the parsed module
    /// made a 50,000-deep input pass, and forgetting the diagnostics alone did not. So the depth
    /// of the tree is bounded here too — an AST this tool builds must be one it can also free.
    fn parse_postfix_on(&mut self, mut e: Expr) -> Expr {
        let mut chain = 0u32;
        loop {
            chain += 1;
            if chain > MAX_EXPR_DEPTH {
                let span = e.span();
                self.error(
                    "DL0210",
                    format!("expression nests deeper than {MAX_EXPR_DEPTH} levels"),
                    span,
                    "simplify or split this expression",
                );
                return e;
            }
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
        self.skip_terms_before_closer();
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
                self.skip_terms_before_closer();
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
                    self.skip_terms_before_closer();
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
        self.parse_until(&TokenKind::RBrace, |p| {
            let arm_start = p.span();
            let pattern = p.parse_pattern();
            p.expect(TokenKind::FatArrow);
            let body = if p.at(&TokenKind::LBrace) {
                Expr::Block(p.parse_block())
            } else {
                p.parse_expr()
            };
            // An arm body is an expression; assignment is a statement. `x => n = 1` therefore
            // stops the expression parser dead at `=`, which nothing else consumes. Name the
            // real rule and hand back the exact edit rather than leaving the reader with
            // "expected `=>`, found `=`" three tokens away from the actual mistake.
            if p.at(&TokenKind::Eq) {
                let body_span = body.span();
                p.bump(); // =
                let rhs = p.parse_expr();
                let bad = body_span.to(rhs.span());
                p.diags.push(
                    Diagnostic::error("DL0201", "a match arm's body is an expression — assignment is a statement")
                        .with_span(bad, "wrap it in a block to assign here")
                        .with_repair(Repair {
                            id: "brace-match-arm-assignment",
                            confidence: Confidence::Exact,
                            authority_widening: false,
                            requires_human: false,
                            edits: vec![
                                Edit { file: bad.file, start_byte: bad.start, end_byte: bad.start, insert: "{ ".into() },
                                Edit { file: bad.file, start_byte: bad.end, end_byte: bad.end, insert: " }".into() },
                            ],
                        }),
                );
            }
            let span = arm_start.to(p.prev_span());
            arms.push(Arm { pattern, body, span });
            // Arms separated by `,` or a terminator; tolerate both.
            if !p.eat(&TokenKind::Comma) {
                p.eat(&TokenKind::Term);
            }
        });
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

    // ----- forward progress (HARDENING_CAMPAIGN C1) --------------------------
    //
    // Two parser loops could spin forever on well-formed-looking input. Both were reached
    // from programs a person would plausibly type, and both survived v1.0.0 and the Stage-10
    // close-out. The behavioural tests below pin the fixed outcome; the structural test is
    // the one that matters most, because a reintroduced unguarded loop makes it FAIL rather
    // than HANG — a hanging test tells CI nothing.

    /// P17-F5. The witness: before `MAX_EXPR_DEPTH` existed, a **valid** module nested deeply
    /// enough did not produce a diagnostic — it overflowed the stack and killed the process
    /// (`thread 'delulu-main' has overflowed its stack`, exit 127, no code, no span, nothing
    /// catchable). The input here is the **exact size that crashed** — 100,000 — rather than a
    /// token amount safely over the limit, because two separate recursions had to be closed to
    /// survive it and only the real size exercises both: the descent guard bounds how deep the
    /// parser recurses, and the postfix-chain guard bounds how deep an AST it builds for `Drop` to
    /// walk. A smaller input passes with only the first fix in place, and would have let the
    /// second defect through.
    #[test]
    fn nesting_past_the_limit_is_refused_with_a_diagnostic_not_a_crash() {
        let src = format!("module m\nfn f() -> Int {{\n  {}1{}\n}}\n", "(".repeat(100_000), ")".repeat(100_000));
        let (_, d) = parse_src(&src);
        assert!(d.iter().any(|x| x.code == "DL0210"), "expected DL0210, got {d:?}");
    }

    /// The control, without which the limit would be indistinguishable from "reject everything".
    /// Ordinary nesting — far more than any human writes — must still parse clean.
    #[test]
    fn nesting_within_the_limit_still_parses_clean() {
        let src = format!("module m\nfn f() -> Int {{\n  {}1{}\n}}\n", "(".repeat(64), ")".repeat(64));
        let (_, d) = parse_src(&src);
        assert!(d.is_empty(), "64 levels of nesting must still parse, got {d:?}");
    }

    #[test]
    fn an_assignment_in_a_match_arm_terminates_and_names_the_rule() {
        // Before the fix: `parse_expr` stopped at `=`, nothing consumed it, and the arm loop
        // pushed a fresh `Arm` per iteration — measured at ~380 MB/s until the machine died.
        let (m, d) = parse_src("module m\nfn f(flag: Bool) {\n    var n = 0\n    match flag {\n        true => n = 1\n        false => n = 2\n    }\n}\n");
        assert!(d.iter().any(|x| x.code == "DL0201"), "expected DL0201, got {d:?}");
        assert!(
            d.iter().any(|x| x.repairs.iter().any(|r| r.id == "brace-match-arm-assignment")),
            "the repair that wraps the assignment in a block must be offered: {d:?}"
        );
        // Bounded output is the real assertion: the old loop produced arms without limit.
        assert_eq!(m.items.len(), 1);
    }

    #[test]
    fn an_import_after_an_item_terminates_and_names_the_rule() {
        // Before the fix: `parse_item` returned None without consuming and `recover_item`
        // stopped *at* `import`, so the position never moved. A pure spin — no allocation,
        // no output, nothing to notice but a pegged core.
        let (_m, d) = parse_src("module m\nfn f() {}\nimport b\n");
        let hit = d.iter().find(|x| x.code == "DL0208").expect("expected DL0208");
        assert!(
            hit.message.contains("must appear before the first item"),
            "the diagnostic should state the placement rule, not just 'expected an item': {hit:?}"
        );
    }

    #[test]
    fn every_delimited_list_loop_goes_through_the_progress_guard() {
        // The guard was written by hand four times and forgotten once. It now lives in exactly
        // one place, and this test is what keeps it there: a new hand-rolled loop over a
        // closing delimiter is a loop whose progress nobody has argued for.
        let src = include_str!("parser.rs");
        // Split so the needle does not match itself — and keep the un-split form out of every
        // comment in this file for the same reason.
        let needle = concat!("while !self.", "at(");
        let n = src.matches(needle).count();
        assert_eq!(
            n, 1,
            "expected exactly one such loop (the one inside `parse_until`), found {n}. \
             Route the new list through `parse_until` instead — an unguarded loop spins \
             forever whenever `expect` fails, because a failed `expect` does not advance."
        );
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

// ----- `type A = B` is an alias (C28, ruling D46a) -----------------------------------------

/// The disambiguation is SYNTACTIC: a variant list is signalled by `(` or `|` and by nothing else.
/// This used to resolve toward a single-variant sum, which made `type Meters = Int` declare a
/// constructor named `Int` — so `fn g() -> Meters { Int }` type-checked — and made an alias to a
/// bare type name unwritable except as `type Meters = (Int)`.
#[cfg(test)]
mod discard_binding_tests {
    use super::*;

    fn parse_ok(src: &str) -> Module {
        let (tokens, ldiags) = crate::lexer::lex(0, src);
        assert!(ldiags.is_empty(), "lex: {ldiags:?}");
        let (m, diags) = parse(0, tokens);
        assert!(!diags.iter().any(|d| d.is_error()), "parse: {diags:?}");
        m
    }

    /// C61. `let _ = expr` was DL0201 "expected a name" while `_` was a perfectly good MATCH
    /// pattern — a discard in one position and not the other. The restriction prevented nothing:
    /// `let ignored = expr` already discards, so all it bought was a worse name.
    #[test]
    fn let_underscore_binds_the_discard_name() {
        let m = parse_ok("module m
fn f() -> Int {
 let _ = 1
 2
}
");
        let Item::Fn(f) = &m.items[0] else { panic!("expected a fn") };
        let Stmt::Let { name, .. } = &f.body.stmts[0] else {
            panic!("expected a let, got {:?}", f.body.stmts[0])
        };
        assert_eq!(name.name, "_");
    }

    /// Two discards in one scope must not collide — which is the point of allowing `_` at all.
    #[test]
    fn more_than_one_discard_in_a_scope_is_fine() {
        let m = parse_ok("module m
fn f() -> Int {
 let _ = 1
 let _ = 2
 3
}
");
        let Item::Fn(f) = &m.items[0] else { panic!() };
        assert_eq!(f.body.stmts.len(), 3);
    }

    /// `var _` parses too. Pointless to write, but refusing it would need a special case, and a
    /// special case is a rule someone has to remember.
    #[test]
    fn var_underscore_parses_as_well() {
        let m = parse_ok("module m
fn f() -> Int {
 var _ = 1
 2
}
");
        let Item::Fn(f) = &m.items[0] else { panic!() };
        let Stmt::Let { name, mutable, .. } = &f.body.stmts[0] else { panic!() };
        assert_eq!(name.name, "_");
        assert!(*mutable);
    }

    /// And the property that makes binding it as a plain name SAFE rather than merely convenient:
    /// a bare `_` lexes as `TokenKind::Underscore`, never as an `Ident`, so no expression can name
    /// it. The discard is write-only by construction, not by a rule anyone could forget.
    #[test]
    fn a_bare_underscore_is_not_an_identifier_token() {
        let (tokens, diags) = crate::lexer::lex(0, "_");
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(tokens[0].kind, TokenKind::Underscore);
        // `_foo` is an ordinary identifier — only the bare one is the discard.
        let (tokens, _) = crate::lexer::lex(0, "_foo");
        assert!(
            matches!(tokens[0].kind, TokenKind::Ident(ref n) if n == "_foo"),
            "got {:?}",
            tokens[0].kind
        );
    }
}

#[cfg(test)]
mod alias_vs_sum_tests {
    use super::tests_support::*;

    #[test]
    fn a_bare_right_hand_side_is_an_alias_not_a_one_variant_sum() {
        let m = parse_ok_src("module m\ntype Meters = Int\n");
        assert!(is_alias(&m, "Meters"), "`type Meters = Int` must be an alias, not a sum");
    }

    #[test]
    fn a_pipe_or_a_paren_still_makes_a_sum() {
        let m = parse_ok_src("module m\ntype E = A | B\n");
        assert!(is_sum(&m, "E"), "`|` signals a variant list");
        let m = parse_ok_src("module m\ntype P = Data(Int)\n");
        assert!(is_sum(&m, "P"), "`(` signals a variant list even with one variant");
        // The escape hatch for a single field-less variant, which is unambiguous.
        let m = parse_ok_src("module m\ntype U = Nothing()\n");
        assert!(is_sum(&m, "U"), "`Nothing()` is a one-variant sum");
    }

    #[test]
    fn a_generic_or_qualified_right_hand_side_is_still_an_alias() {
        let m = parse_ok_src("module m\ntype Handle = List[Int]\n");
        assert!(is_alias(&m, "Handle"), "`[` after the identifier was always an alias");
        let m = parse_ok_src("module m\ntype Paren = (Int)\n");
        assert!(is_alias(&m, "Paren"), "the parenthesised spelling keeps working");
    }
}

// ----- a multi-line list needs no trailing comma (C47b, ruling D46d) -----------------------

/// §2.2 inserts a `Term` at a newline only when the previous token can end a statement, and a comma
/// cannot — which is why `a,\n)` always parsed while `a\n)` did not. One unskipped terminator, in
/// nine bracketed lists. All four spellings must now parse in every one of them.
#[cfg(test)]
mod trailing_comma_tests {
    use super::tests_support::*;

    #[test]
    fn every_bracketed_list_accepts_a_multi_line_form_without_a_trailing_comma() {
        for (what, src) in [
            ("record type body", "module m\ntype T {\n    a: Int,\n    b: Int\n}\n"),
            ("fn params", "module m\nfn g(\n    a: Int,\n    b: Int\n) -> Int {\n    a + b\n}\n"),
            ("generics", "module m\nfn g[\n    A,\n    B\n](x: Int) -> Int {\n    x\n}\n"),
            ("variant fields", "module m\ntype P = Data(\n    Int,\n    Str\n)\n"),
            ("generic type args", "module m\ntype H = Result[\n    Int,\n    Str\n]\n"),
            (
                "call args",
                "module m\nfn h(a: Int, b: Int) -> Int {\n    a + b\n}\nfn g() -> Int {\n    h(\n        1,\n        2\n    )\n}\n",
            ),
            ("list literal", "module m\nfn g() -> List[Int] {\n    [\n        1,\n        2\n    ]\n}\n"),
            (
                "record literal",
                "module m\ntype T { a: Int, b: Int }\nfn g() -> T {\n    T {\n        a: 1,\n        b: 2\n    }\n}\n",
            ),
        ] {
            let (_, d) = parse_src_raw(src);
            assert!(d.is_empty(), "{what}: a multi-line list without a trailing comma must parse: {d:?}");
        }
    }

    #[test]
    fn the_trailing_comma_forms_still_parse() {
        for src in [
            "module m\ntype T {\n    a: Int,\n    b: Int,\n}\n",
            "module m\ntype T { a: Int, b: Int }\n",
            "module m\ntype T { a: Int, b: Int, }\n",
        ] {
            let (_, d) = parse_src_raw(src);
            assert!(d.is_empty(), "the other spellings must keep parsing: {d:?}");
        }
    }
}

#[cfg(test)]
mod tests_support {
    use crate::ast::{Module, TypeDeclKind};
    use crate::lexer::lex;
    use delulu_diag::Diagnostic;

    pub fn parse_src_raw(src: &str) -> (Module, Vec<Diagnostic>) {
        let (tokens, ldiags) = lex(0, src);
        assert!(ldiags.is_empty(), "lex errors: {ldiags:?}");
        super::parse(0, tokens)
    }

    pub fn parse_ok_src(src: &str) -> Module {
        let (m, d) = parse_src_raw(src);
        assert!(d.is_empty(), "unexpected parse diagnostics: {d:?}");
        m
    }

    fn kind_of<'a>(m: &'a Module, name: &str) -> &'a TypeDeclKind {
        m.items
            .iter()
            .find_map(|i| match i {
                crate::ast::Item::Type(t) if t.name.name == name => Some(&t.kind),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no type named `{name}`"))
    }

    pub fn is_alias(m: &Module, name: &str) -> bool {
        matches!(kind_of(m, name), TypeDeclKind::Alias(_))
    }

    pub fn is_sum(m: &Module, name: &str) -> bool {
        matches!(kind_of(m, name), TypeDeclKind::Sum(_))
    }
}
