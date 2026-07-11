//! The Stage-1 parser (spec §3): error-recovering recursive descent with a Pratt
//! core for expressions. On error it records a diagnostic and resynchronizes at
//! the next statement/item boundary, so one run can surface many diagnostics.
//!
//! Reserved-word policy (§2.3 precision): `expect_decl_name` rejects reserved
//! words (DL0106); member names after `.` never pass through it, so `root.secret(…)`
//! parses while `fn secret()` does not.

use delulu_diag::{Diagnostic, FileId, Span};

use crate::ast::*;
use crate::token::{is_reserved, Token, TokenKind};

pub fn parse(file: FileId, tokens: Vec<Token>) -> (Module, Vec<Diagnostic>) {
    let mut p = Parser::new(file, tokens);
    let module = p.parse_module();
    (module, p.diags)
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
                        self.error("DL0208", "expected an item (`fn`, `type`, `effect`, `let`, or `pub`)", self.span(), "not an item");
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
        match self.peek() {
            TokenKind::KwFn => Some(Item::Fn(self.parse_fn(public))),
            TokenKind::KwType => Some(Item::Type(self.parse_type_decl(public))),
            TokenKind::KwEffect => Some(Item::Effect(self.parse_effect_decl(public))),
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

    fn parse_type(&mut self) -> TypeExpr {
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
            _ => self.parse_postfix(allow_struct),
        }
    }

    fn parse_postfix(&mut self, allow_struct: bool) -> Expr {
        let mut e = self.parse_primary(allow_struct);
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
}
