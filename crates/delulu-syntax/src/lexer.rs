//! The Stage-1 lexer (spec §2), including Go-style automatic statement
//! termination (§2.2): a newline inserts `Term` iff the previous significant
//! token can end a statement. A block comment containing a newline counts as
//! a newline. Explicit `;` always produces `Term`.

use delulu_diag::{Diagnostic, FileId, Span};

use crate::token::{keyword, Token, TokenKind};

pub fn lex(file: FileId, src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let (tokens, diags, _comments) = Lexer::new(file, src).run();
    (tokens, diags)
}

/// Lex with the comment side channel (Stage 8, phase 8d): identical tokens and
/// diagnostics to [`lex`], plus every comment in source order for the formatter.
pub fn lex_with_comments(file: FileId, src: &str) -> (Vec<Token>, Vec<Diagnostic>, Vec<Comment>) {
    Lexer::new(file, src).run()
}

/// A source comment captured for the formatter (Stage 8, phase 8d). `text` is the raw
/// slice INCLUDING its `//` or `/* */` markers; `own_line` is whether nothing but
/// whitespace precedes it on its line (an own-line comment stays own-line when
/// reprinted; anything else is a trailing comment).
#[derive(Clone, Debug)]
pub struct Comment {
    pub text: String,
    pub start: u32,
    pub own_line: bool,
}

struct Lexer<'a> {
    file: FileId,
    src: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    diags: Vec<Diagnostic>,
    comments: Vec<Comment>,
}

impl<'a> Lexer<'a> {
    fn new(file: FileId, src: &'a str) -> Self {
        // Skip a leading UTF-8 byte-order mark (U+FEFF): many editors (and PowerShell's `Set-Content
        // -Encoding utf8` on Windows) prepend one, and it isn't source text. Only a *leading* BOM is
        // trivia; a U+FEFF elsewhere still lexes normally (and is rejected as an unexpected char).
        let pos = if src.starts_with('\u{feff}') { '\u{feff}'.len_utf8() } else { 0 };
        Lexer { file, src, pos, tokens: Vec::new(), diags: Vec::new(), comments: Vec::new() }
    }

    fn run(mut self) -> (Vec<Token>, Vec<Diagnostic>, Vec<Comment>) {
        while self.pos < self.src.len() {
            self.skip_trivia();
            if self.pos >= self.src.len() {
                break;
            }
            self.scan_token();
        }
        // End of file acts like a final newline (§2.2), so a last line without
        // a trailing newline still terminates its statement.
        self.maybe_insert_term(self.src.len());
        let eof = self.src.len() as u32;
        self.tokens.push(Token { kind: TokenKind::Eof, span: Span::new(self.file, eof, eof) });
        (self.tokens, self.diags, self.comments)
    }

    /// Record a comment spanning `start..self.pos` for the formatter's side channel.
    fn record_comment(&mut self, start: usize) {
        let own_line = self.src[..start]
            .chars()
            .rev()
            .take_while(|&c| c != '\n')
            .all(char::is_whitespace);
        self.comments.push(Comment {
            text: self.src[start..self.pos].to_string(),
            start: start as u32,
            own_line,
        });
    }

    // ----- low-level cursor -----------------------------------------------

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek2(&self) -> Option<char> {
        let mut it = self.src[self.pos..].chars();
        it.next();
        it.next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn span_from(&self, start: usize) -> Span {
        Span::new(self.file, start as u32, self.pos as u32)
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        let span = self.span_from(start);
        self.tokens.push(Token { kind, span });
    }

    fn last_kind(&self) -> Option<&TokenKind> {
        self.tokens.last().map(|t| &t.kind)
    }

    /// Insert a `Term` at `at` if the previous significant token allows it.
    fn maybe_insert_term(&mut self, at: usize) {
        if let Some(kind) = self.last_kind() {
            if kind.can_end_statement() {
                let pos = at as u32;
                self.tokens.push(Token { kind: TokenKind::Term, span: Span::new(self.file, pos, pos) });
            }
        }
    }

    // ----- trivia: whitespace and comments --------------------------------

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some('\n') => {
                    self.maybe_insert_term(self.pos);
                    self.bump();
                }
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek2() == Some('/') => {
                    // Line comment: runs to (not through) the newline.
                    let start = self.pos;
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                    self.record_comment(start);
                }
                Some('/') if self.peek2() == Some('*') => {
                    self.block_comment();
                }
                _ => break,
            }
        }
    }

    fn block_comment(&mut self) {
        let start = self.pos;
        self.bump(); // '/'
        self.bump(); // '*'
        let mut depth = 1u32;
        let mut saw_newline = false;
        while depth > 0 {
            match self.peek() {
                None => {
                    self.diags.push(
                        Diagnostic::error("DL0105", "unterminated block comment")
                            .with_span(self.span_from(start), "comment opened here"),
                    );
                    return;
                }
                Some('\n') => {
                    saw_newline = true;
                    self.bump();
                }
                Some('/') if self.peek2() == Some('*') => {
                    self.bump();
                    self.bump();
                    depth += 1;
                }
                Some('*') if self.peek2() == Some('/') => {
                    self.bump();
                    self.bump();
                    depth -= 1;
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
        self.record_comment(start);
        // §2.2: a comment containing a newline counts as a newline.
        if saw_newline {
            self.maybe_insert_term(start);
        }
    }

    // ----- tokens ----------------------------------------------------------

    fn scan_token(&mut self) {
        let start = self.pos;
        let c = match self.bump() {
            Some(c) => c,
            None => return,
        };
        match c {
            '(' => self.push(TokenKind::LParen, start),
            ')' => self.push(TokenKind::RParen, start),
            '{' => self.push(TokenKind::LBrace, start),
            '}' => self.push(TokenKind::RBrace, start),
            '[' => self.push(TokenKind::LBracket, start),
            ']' => self.push(TokenKind::RBracket, start),
            ',' => self.push(TokenKind::Comma, start),
            ':' => self.push(TokenKind::Colon, start),
            ';' => self.push(TokenKind::Term, start),
            '@' => self.push(TokenKind::At, start),
            '?' => self.push(TokenKind::Question, start),
            '.' => self.push(TokenKind::Dot, start),
            '+' => self.push(TokenKind::Plus, start),
            '*' => self.push(TokenKind::Star, start),
            '%' => self.push(TokenKind::Percent, start),
            '/' => self.push(TokenKind::Slash, start), // comments were consumed as trivia
            '-' => {
                if self.peek() == Some('>') {
                    self.bump();
                    self.push(TokenKind::Arrow, start);
                } else {
                    self.push(TokenKind::Minus, start);
                }
            }
            '=' => match self.peek() {
                Some('=') => {
                    self.bump();
                    self.push(TokenKind::EqEq, start);
                }
                Some('>') => {
                    self.bump();
                    self.push(TokenKind::FatArrow, start);
                }
                _ => self.push(TokenKind::Eq, start),
            },
            '!' => {
                if self.peek() == Some('=') {
                    self.bump();
                    self.push(TokenKind::NotEq, start);
                } else {
                    self.push(TokenKind::Bang, start);
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.bump();
                    self.push(TokenKind::Le, start);
                } else {
                    self.push(TokenKind::Lt, start);
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.bump();
                    self.push(TokenKind::Ge, start);
                } else {
                    self.push(TokenKind::Gt, start);
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.bump();
                    self.push(TokenKind::AndAnd, start);
                } else {
                    self.diags.push(
                        Diagnostic::error("DL0101", "unexpected character `&` (did you mean `&&`?)")
                            .with_bare_span(self.span_from(start)),
                    );
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.bump();
                    self.push(TokenKind::OrOr, start);
                } else {
                    self.push(TokenKind::Pipe, start);
                }
            }
            '"' => self.string(start),
            c if c.is_ascii_digit() => self.number(start),
            c if c.is_ascii_alphabetic() || c == '_' => self.ident(start),
            other => {
                self.diags.push(
                    Diagnostic::error("DL0101", format!("unexpected character `{other}`"))
                        .with_bare_span(self.span_from(start)),
                );
            }
        }
    }

    fn ident(&mut self, start: usize) {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.bump();
            } else {
                break;
            }
        }
        let text = &self.src[start..self.pos];
        if text == "_" {
            self.push(TokenKind::Underscore, start);
        } else if let Some(kw) = keyword(text) {
            self.push(kw, start);
        } else {
            // Reserved words are emitted as identifiers; the parser enforces
            // DL0106 at declaration sites (§2.3 precision).
            self.push(TokenKind::Ident(text.to_string()), start);
        }
    }

    fn number(&mut self, start: usize) {
        // Hex: 0x[0-9a-fA-F_]+
        if self.src[start..].starts_with("0x") || self.src[start..].starts_with("0X") {
            self.bump(); // 'x'
            let digits_start = self.pos;
            while let Some(c) = self.peek() {
                if c.is_ascii_hexdigit() || c == '_' {
                    self.bump();
                } else {
                    break;
                }
            }
            let digits: String =
                self.src[digits_start..self.pos].chars().filter(|&c| c != '_').collect();
            if digits.is_empty() {
                self.diags.push(
                    Diagnostic::error("DL0104", "hex literal needs at least one digit")
                        .with_bare_span(self.span_from(start)),
                );
                self.push(TokenKind::Int(0), start);
                return;
            }
            match i64::from_str_radix(&digits, 16) {
                Ok(v) => self.push(TokenKind::Int(v), start),
                Err(_) => {
                    self.diags.push(
                        Diagnostic::error("DL0104", "integer literal overflows Int (i64)")
                            .with_bare_span(self.span_from(start)),
                    );
                    self.push(TokenKind::Int(0), start);
                }
            }
            return;
        }

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                self.bump();
            } else {
                break;
            }
        }

        // Float only when `.` is followed by a digit — so `1.method()` lexes as
        // Int, Dot, Ident (§2.1 note).
        let mut is_float = false;
        if self.peek() == Some('.') && self.peek2().is_some_and(|c| c.is_ascii_digit()) {
            is_float = true;
            self.bump(); // '.'
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    self.bump();
                } else {
                    break;
                }
            }
            if matches!(self.peek(), Some('e') | Some('E')) {
                let exp_mark = self.pos;
                self.bump();
                if matches!(self.peek(), Some('+') | Some('-')) {
                    self.bump();
                }
                if !self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.diags.push(
                        Diagnostic::error("DL0104", "exponent needs at least one digit")
                            .with_bare_span(self.span_from(exp_mark)),
                    );
                } else {
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        let text: String = self.src[start..self.pos].chars().filter(|&c| c != '_').collect();
        if is_float {
            match text.parse::<f64>() {
                Ok(v) => self.push(TokenKind::Float(v), start),
                Err(_) => {
                    self.diags.push(
                        Diagnostic::error("DL0104", "invalid float literal")
                            .with_bare_span(self.span_from(start)),
                    );
                    self.push(TokenKind::Float(0.0), start);
                }
            }
        } else {
            match text.parse::<i64>() {
                Ok(v) => self.push(TokenKind::Int(v), start),
                Err(_) => {
                    self.diags.push(
                        Diagnostic::error("DL0104", "integer literal overflows Int (i64)")
                            .with_bare_span(self.span_from(start)),
                    );
                    self.push(TokenKind::Int(0), start);
                }
            }
        }
    }

    fn string(&mut self, start: usize) {
        let mut value = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    self.diags.push(
                        Diagnostic::error("DL0102", "unterminated string literal")
                            .with_span(self.span_from(start), "string opened here"),
                    );
                    self.push(TokenKind::Str(value), start);
                    return;
                }
                Some('"') => {
                    self.bump();
                    self.push(TokenKind::Str(value), start);
                    return;
                }
                Some('\\') => {
                    let esc_start = self.pos;
                    self.bump();
                    match self.bump() {
                        Some('n') => value.push('\n'),
                        Some('t') => value.push('\t'),
                        Some('r') => value.push('\r'),
                        Some('\\') => value.push('\\'),
                        Some('"') => value.push('"'),
                        Some('0') => value.push('\0'),
                        Some('u') => {
                            if self.peek() == Some('{') {
                                self.bump();
                                let hex_start = self.pos;
                                while self.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                                    self.bump();
                                }
                                let hex = &self.src[hex_start..self.pos];
                                let closed = self.peek() == Some('}');
                                if closed {
                                    self.bump();
                                }
                                let ok = closed
                                    && !hex.is_empty()
                                    && hex.len() <= 6
                                    && u32::from_str_radix(hex, 16)
                                        .ok()
                                        .and_then(char::from_u32)
                                        .map(|c| {
                                            value.push(c);
                                        })
                                        .is_some();
                                if !ok {
                                    self.diags.push(
                                        Diagnostic::error(
                                            "DL0103",
                                            "invalid unicode escape (expected \\u{1-6 hex digits})",
                                        )
                                        .with_bare_span(self.span_from(esc_start)),
                                    );
                                }
                            } else {
                                self.diags.push(
                                    Diagnostic::error("DL0103", "invalid escape: `\\u` needs `{…}`")
                                        .with_bare_span(self.span_from(esc_start)),
                                );
                            }
                        }
                        other => {
                            let shown = other.map(|c| c.to_string()).unwrap_or_default();
                            self.diags.push(
                                Diagnostic::error(
                                    "DL0103",
                                    format!("invalid escape sequence `\\{shown}`"),
                                )
                                .with_bare_span(self.span_from(esc_start)),
                            );
                        }
                    }
                }
                Some(_) => {
                    let c = self.bump().unwrap();
                    value.push(c);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        let (tokens, diags) = lex(0, src);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        tokens.into_iter().map(|t| t.kind).collect()
    }

    fn ident(s: &str) -> TokenKind {
        TokenKind::Ident(s.into())
    }

    #[test]
    fn terminator_inserted_after_statement_enders() {
        // §2.2 core rule: newline after an ident inserts Term.
        let k = kinds("let x = 1\nlet y = 2\n");
        let terms = k.iter().filter(|k| **k == TokenKind::Term).count();
        assert_eq!(terms, 2);
        // ...and the sequence is exactly as expected.
        assert_eq!(
            k,
            vec![
                TokenKind::KwLet, ident("x"), TokenKind::Eq, TokenKind::Int(1), TokenKind::Term,
                TokenKind::KwLet, ident("y"), TokenKind::Eq, TokenKind::Int(2), TokenKind::Term,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn no_terminator_after_operators_enables_continuation() {
        // `a +\n b` continues the expression; `(`-ended lines continue too.
        let k = kinds("a +\n b\n");
        assert_eq!(k, vec![ident("a"), TokenKind::Plus, ident("b"), TokenKind::Term, TokenKind::Eof]);
    }

    #[test]
    fn terminator_after_rbrace_and_question() {
        let k = kinds("f()?\nx");
        assert!(k.contains(&TokenKind::Term));
        let k2 = kinds("{ }\nx");
        // Term inserted after `}` — this is the `} else` consequence documented in §2.2.
        assert_eq!(
            k2,
            vec![TokenKind::LBrace, TokenKind::RBrace, TokenKind::Term, ident("x"), TokenKind::Term, TokenKind::Eof]
        );
    }

    #[test]
    fn eof_terminates_last_statement_without_newline() {
        let k = kinds("return x");
        assert_eq!(k, vec![TokenKind::KwReturn, ident("x"), TokenKind::Term, TokenKind::Eof]);
    }

    #[test]
    fn numbers_hex_underscores_floats_and_method_on_int() {
        assert_eq!(kinds("1_000"), vec![TokenKind::Int(1000), TokenKind::Term, TokenKind::Eof]);
        assert_eq!(kinds("0xFF"), vec![TokenKind::Int(255), TokenKind::Term, TokenKind::Eof]);
        assert_eq!(kinds("1.5"), vec![TokenKind::Float(1.5), TokenKind::Term, TokenKind::Eof]);
        assert_eq!(
            kinds("2.5e2"),
            vec![TokenKind::Float(250.0), TokenKind::Term, TokenKind::Eof]
        );
        // `1.method` is Int Dot Ident, not a float.
        assert_eq!(
            kinds("1.abs()"),
            vec![
                TokenKind::Int(1), TokenKind::Dot, ident("abs"),
                TokenKind::LParen, TokenKind::RParen, TokenKind::Term, TokenKind::Eof
            ]
        );
    }

    #[test]
    fn int_overflow_is_dl0104() {
        let (_, diags) = lex(0, "99999999999999999999");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "DL0104");
    }

    #[test]
    fn string_escapes_and_unicode() {
        let (tokens, diags) = lex(0, r#""a\nb\u{1F426}""#);
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(tokens[0].kind, TokenKind::Str("a\nb\u{1F426}".into()));
    }

    #[test]
    fn unterminated_string_is_dl0102() {
        let (_, diags) = lex(0, "\"abc\n");
        assert_eq!(diags[0].code, "DL0102");
    }

    #[test]
    fn invalid_escape_is_dl0103() {
        let (_, diags) = lex(0, r#""a\q""#);
        assert_eq!(diags[0].code, "DL0103");
    }

    #[test]
    fn nested_block_comments_and_newline_rule() {
        // Comment with newline acts as a newline: Term inserted after `x`.
        let k = kinds("x /* outer /* inner */ still\ncomment */ y");
        assert_eq!(k, vec![ident("x"), TokenKind::Term, ident("y"), TokenKind::Term, TokenKind::Eof]);
        // Single-line block comment does NOT terminate.
        let k2 = kinds("x /* c */ + y");
        assert_eq!(k2, vec![ident("x"), TokenKind::Plus, ident("y"), TokenKind::Term, TokenKind::Eof]);
    }

    #[test]
    fn unterminated_block_comment_is_dl0105() {
        let (_, diags) = lex(0, "/* never closed");
        assert_eq!(diags[0].code, "DL0105");
    }

    #[test]
    fn unexpected_char_is_dl0101() {
        let (_, diags) = lex(0, "let x = #");
        assert_eq!(diags[0].code, "DL0101");
    }

    #[test]
    fn leading_utf8_bom_is_skipped() {
        // A BOM-prefixed source lexes exactly like the un-prefixed one — no DL0101.
        let (_, diags) = lex(0, "\u{feff}module m");
        assert!(diags.is_empty(), "a leading BOM must be trivia, got {diags:?}");
        assert_eq!(kinds("\u{feff}module m"), kinds("module m"));
    }

    #[test]
    fn reserved_words_lex_as_identifiers() {
        // The parser rejects them at declaration sites; member use stays legal.
        let k = kinds("root.secret(\"K\")");
        assert_eq!(k[2], ident("secret"));
    }

    #[test]
    fn effect_row_and_operator_tokens() {
        let k = kinds("! {Read} != a <= b -> c => d");
        assert_eq!(k[0], TokenKind::Bang);
        assert!(k.contains(&TokenKind::NotEq));
        assert!(k.contains(&TokenKind::Le));
        assert!(k.contains(&TokenKind::Arrow));
        assert!(k.contains(&TokenKind::FatArrow));
    }
}
