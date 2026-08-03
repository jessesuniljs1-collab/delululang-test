//! The Stage-1 lexer (spec §2), including Go-style automatic statement
//! termination (§2.2): a newline inserts `Term` iff the previous significant
//! token can end a statement. A block comment containing a newline counts as
//! a newline. Explicit `;` always produces `Term`.

use delulu_diag::{Diagnostic, FileId, Span};

use crate::token::{keyword, Token, TokenKind};

/// Unicode bidirectional formatting characters, which can reorder how source *renders* without
/// changing how it *lexes* — the "Trojan Source" attack (CVE-2021-42574). In a language whose
/// stated purpose includes humans reviewing AI-written code, a file that renders differently than
/// it runs is an attack on the one control the reviewer has. So a raw occurrence of any of these,
/// anywhere in source, is refused outright (DL0107).
///
/// Each is named, because a bare code point in a table is exactly the kind of thing a future
/// reader cannot verify. The set is the eleven characters Rust denies for the same reason.
/// Identifiers are already ASCII-only (DL0101 refuses the rest), so these can reach source *only*
/// inside a comment or a string literal — which are precisely the attack's two vectors. A program
/// that genuinely needs one of these code points in string *data* writes it as an escape
/// (`\u{202e}`): plain ASCII in source, so visible to a reviewer and unaffected by this scan.
const BIDI_CONTROLS: &[(char, &str)] = &[
    ('\u{200E}', "LEFT-TO-RIGHT MARK"),
    ('\u{200F}', "RIGHT-TO-LEFT MARK"),
    ('\u{202A}', "LEFT-TO-RIGHT EMBEDDING"),
    ('\u{202B}', "RIGHT-TO-LEFT EMBEDDING"),
    ('\u{202C}', "POP DIRECTIONAL FORMATTING"),
    ('\u{202D}', "LEFT-TO-RIGHT OVERRIDE"),
    ('\u{202E}', "RIGHT-TO-LEFT OVERRIDE"),
    ('\u{2066}', "LEFT-TO-RIGHT ISOLATE"),
    ('\u{2067}', "RIGHT-TO-LEFT ISOLATE"),
    ('\u{2068}', "FIRST STRONG ISOLATE"),
    ('\u{2069}', "POP DIRECTIONAL ISOLATE"),
];

/// Characters that an editor, a terminal, a diff viewer, GitHub, and Unicode's own `splitlines`
/// all render as a **line break**, but which this lexer does not treat as one (DL0108).
///
/// Same threat model as [`BIDI_CONTROLS`] — a file that renders differently than it runs — reached
/// by a different door, and campaign finding C89 observed *two* working attacks with them:
///
/// 1. **A line comment ends only at `\n`.** So one of these swallows the next *rendered* line into
///    the comment. This is the INVERSE of Trojan Source and is worse: the reviewer sees a line of
///    code that the compiler never compiles. A guard clause — `if amount > LIMIT { return }` —
///    sitting visibly above a transfer, and simply not there.
/// 2. **Automatic semicolon insertion fires only at `\n`.** So one of these silently JOINS two
///    statements the reviewer sees on separate lines, changing which expression a binding gets.
///
/// Both were observed checking clean, and the first also survived `fmt --check` — `fmt` preserved
/// the character verbatim and was idempotent, so the one accidental defense evaporated after a
/// single format pass.
///
/// Refused rather than reinterpreted, exactly as DL0107 refuses: making these *end* a comment would
/// silently promote hidden text into live code in any file that already contains one. A program
/// that genuinely needs one of these code points in string *data* writes the `\u{…}` escape — plain
/// ASCII in source, visible to a reviewer, and untouched by this scan.
///
/// **CR is not here**, because `\r\n` is an ordinary Windows line ending; a *lone* CR is refused
/// separately below, where the pair can be told from the stray.
const LINE_BREAK_LOOKALIKES: &[(char, &str)] = &[
    ('\u{000B}', "LINE TABULATION (vertical tab)"),
    ('\u{000C}', "FORM FEED"),
    ('\u{0085}', "NEXT LINE"),
    ('\u{2028}', "LINE SEPARATOR"),
    ('\u{2029}', "PARAGRAPH SEPARATOR"),
];

pub fn lex(file: FileId, src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let (tokens, diags, _comments) = Lexer::new(file, src).run();
    (tokens, diags)
}

/// Lex with the comment side channel (Stage 8, phase 8d): identical tokens and
/// diagnostics to [`lex`], plus every comment in source order for the formatter.
pub fn lex_with_comments(file: FileId, src: &str) -> (Vec<Token>, Vec<Diagnostic>, Vec<Comment>) {
    Lexer::new(file, src).run()
}

/// Lex source written in a surface **morph** (Stage 8 §6.5): an alias at the start of a token
/// becomes the canonical keyword token it stands for. `None` is exactly [`lex`].
///
/// This is the ONLY place in the toolchain where a non-canonical surface becomes tokens. Everything
/// downstream — parser, checker, DIR, hashes, diagnostics, both engines — sees canonical tokens and
/// cannot tell which surface produced them, which is the property that makes morphs safe to add to a
/// language whose identity is computed from its source.
pub fn lex_with_morph(
    file: FileId,
    src: &str,
    morph: Option<&crate::morph::Morph>,
) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lx = Lexer::new(file, src);
    lx.morph = morph;
    let (tokens, diags, _comments) = lx.run();
    (tokens, diags)
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
    /// The active surface morph, if the file declares one (Stage 8 §6.5). `None` is canonical, and
    /// canonical is what every entry point uses unless it deliberately opts in.
    morph: Option<&'a crate::morph::Morph>,
}

impl<'a> Lexer<'a> {
    fn new(file: FileId, src: &'a str) -> Self {
        // Skip a leading UTF-8 byte-order mark (U+FEFF): many editors (and PowerShell's `Set-Content
        // -Encoding utf8` on Windows) prepend one, and it isn't source text. Only a *leading* BOM is
        // trivia; a U+FEFF elsewhere still lexes normally (and is rejected as an unexpected char).
        let pos = if src.starts_with('\u{feff}') { '\u{feff}'.len_utf8() } else { 0 };
        Lexer {
            file,
            src,
            pos,
            tokens: Vec::new(),
            diags: Vec::new(),
            comments: Vec::new(),
            morph: None,
        }
    }

    fn run(mut self) -> (Vec<Token>, Vec<Diagnostic>, Vec<Comment>) {
        // Security scan BEFORE tokenizing, over the whole raw source, so no per-token path can
        // forget it — the rule lives in exactly one place and cannot die in a branch (the
        // project's skip-branch discipline). See `BIDI_CONTROLS` and HARDENING_CAMPAIGN C3.
        self.check_bidi_controls();
        self.check_line_break_lookalikes();
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

    /// Refuse raw bidirectional control characters anywhere in source (DL0107).
    ///
    /// One pass over the whole raw source. Because it scans raw bytes, an escaped control
    /// (`\u{202e}`) is left alone — it is the ASCII sequence `\`,`u`,`{`,… in source, not the code
    /// point — so the legitimate "I really do need this byte in a string" case stays open and
    /// stays visible in review, while the raw-byte attack is refused. Runs once, ahead of the
    /// tokenizer, and is therefore reached for every entry point (check, run, fmt, authority)
    /// because all of them lex.
    fn check_bidi_controls(&mut self) {
        for (off, ch) in self.src.char_indices() {
            if let Some((_, name)) = BIDI_CONTROLS.iter().find(|(c, _)| *c == ch) {
                let span = Span::new(self.file, off as u32, (off + ch.len_utf8()) as u32);
                self.diags.push(
                    Diagnostic::error(
                        "DL0107",
                        format!("bidirectional control character U+{:04X} ({name}) in source", ch as u32),
                    )
                    .with_span(span, "this can make the code render differently than it runs — use the `\\u{…}` escape if a string truly needs it"),
                );
            }
        }
    }

    /// Refuse characters that render as a line break but do not act as one (DL0108).
    ///
    /// One pass over the raw source, beside [`Self::check_bidi_controls`] and for the same reason:
    /// the rule lives in exactly one place, ahead of the tokenizer, so it is reached by every entry
    /// point and cannot die in a per-token branch. See [`LINE_BREAK_LOOKALIKES`] for the two attacks.
    fn check_line_break_lookalikes(&mut self) {
        let bytes = self.src.as_bytes();
        for (off, ch) in self.src.char_indices() {
            // A lone CR: renders as a line break everywhere, ends nothing here. `\r\n` is an
            // ordinary Windows line ending and is left alone — the pair is told from the stray by
            // looking at the next byte, which is why CR is handled here and not in the table.
            if ch == '\r' && bytes.get(off + 1) != Some(&b'\n') {
                self.diags.push(
                    Diagnostic::error(
                        "DL0108",
                        "lone CARRIAGE RETURN (U+000D) in source, not part of a `\\r\\n` line ending",
                    )
                    .with_span(
                        Span::new(self.file, off as u32, (off + 1) as u32),
                        "editors show a line break here and the compiler does not — anything after it \
                         on this line stays inside a comment, and two statements can silently join",
                    ),
                );
                continue;
            }
            if let Some((_, name)) = LINE_BREAK_LOOKALIKES.iter().find(|(c, _)| *c == ch) {
                self.diags.push(
                    Diagnostic::error(
                        "DL0108",
                        format!("line-break-like character U+{:04X} ({name}) in source", ch as u32),
                    )
                    .with_span(
                        Span::new(self.file, off as u32, (off + ch.len_utf8()) as u32),
                        "editors render this as a new line and the compiler does not — code after it \
                         can sit invisibly inside a comment; use the `\\u{…}` escape if a string truly needs it",
                    ),
                );
            }
        }
    }

    /// If the active morph has an alias starting at `start`, consume it and push its keyword token.
    ///
    /// Aliases are tried longest-first (`Morph::aliases` guarantees the order), so with both `l` and
    /// `lm` defined, `lm` cannot be mis-lexed as `l` followed by a stray `m`.
    ///
    /// The maximal-munch guard matters as much as the match: an alias that *ends* where an
    /// identifier character continues is not an alias occurrence. Without it, the compact alias `f`
    /// for `fn` would turn the identifier `foo` into `fn` followed by `oo`. The guard only applies
    /// when the alias itself ends in an identifier character — a symbol or CJK alias cannot be the
    /// prefix of an ASCII identifier, and identifiers here are ASCII-only.
    fn try_morph_alias(&mut self, start: usize) -> bool {
        let Some(m) = self.morph else { return false };
        let rest = &self.src[start..];
        for (alias, kind) in m.aliases() {
            let Some(tail) = rest.strip_prefix(alias.as_str()) else { continue };
            if alias.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
                if let Some(next) = tail.chars().next() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        continue;
                    }
                }
            }
            self.pos = start + alias.len();
            self.push(kind.clone(), start);
            return true;
        }
        false
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
        // A surface morph's alias becomes the keyword token it stands for. Checked here, at the one
        // point the lexer decides what token begins — and therefore AFTER trivia, so an alias
        // appearing inside a comment or a string literal is never seen by this at all. That is what
        // keeps "identifiers, strings, and comments are never morphed" a structural property rather
        // than a rule someone has to remember.
        if self.try_morph_alias(start) {
            return;
        }
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
            // The rule is shared with the runtime's `parse_float` (see `crate::num`): a literal in
            // source and a number read from input must mean the same thing, or the language has
            // two float rules wearing one name. `f64::from_str` saturates to infinity and flushes
            // to zero rather than failing, and both are the same defect the integer arm below
            // already refuses — the value written is not the value the program will use.
            //
            // Infinity remains reachable by computing it (`1.0 / 0.0`). What cannot be done is
            // spelling it as a finite number.
            let (value, message) = match crate::num::float_from_text(&text) {
                crate::num::FloatText::Value(v) => (v, None),
                crate::num::FloatText::Overflow => (
                    0.0,
                    Some(
                        "float literal overflows Float (f64) — it is larger than any representable \
                         float, so it would silently become `inf`",
                    ),
                ),
                crate::num::FloatText::Underflow => (
                    0.0,
                    Some(
                        "float literal underflows Float (f64) to zero — it is smaller than any \
                         representable non-zero float, so it would silently become `0.0`",
                    ),
                ),
                crate::num::FloatText::Malformed => (0.0, Some("invalid float literal")),
            };
            if let Some(m) = message {
                self.diags
                    .push(Diagnostic::error("DL0104", m).with_bare_span(self.span_from(start)));
            }
            self.push(TokenKind::Float(value), start);
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

    /// C17. The integer column has always refused a literal it cannot hold; the float column
    /// saturated to `inf` and flushed to `0.0` in silence. Same defect, so the same answer — and
    /// the underflow half is the worse one, because `inf` at least LOOKS wrong downstream while a
    /// silently-zeroed gain just makes a control law quietly do nothing.
    #[test]
    fn a_float_literal_that_is_not_representable_is_refused_like_an_integer_one() {
        for src in ["1.0e400", "-2.5e308000", "1.7976931348623159e308"] {
            let (_, diags) = lex(0, src);
            assert!(
                diags.iter().any(|d| d.code == "DL0104" && d.message.contains("overflows Float")),
                "{src} should be refused as an overflow, got {diags:?}"
            );
        }
        // The second is the same defect written the long way, with no exponent to notice: 400
        // zeros then a 1, well past the smallest subnormal (~4.9e-324).
        let written_out = format!("0.{}1", "0".repeat(400));
        for src in ["1.0e-400", &written_out] {
            let (_, diags) = lex(0, src);
            assert!(
                diags.iter().any(|d| d.code == "DL0104" && d.message.contains("underflows Float")),
                "{src} should be refused as an underflow, got {diags:?}"
            );
        }
    }

    /// The other side of the same rule, which is what stops it being a blunt "reject small floats":
    /// a literal the author DID write as zero is zero, and a subnormal is a real number that loses
    /// precision without losing its magnitude — neither is the defect above.
    #[test]
    fn zero_and_subnormal_float_literals_are_still_accepted() {
        assert_eq!(kinds("0.0"), vec![TokenKind::Float(0.0), TokenKind::Term, TokenKind::Eof]);
        assert_eq!(kinds("0.0e-400"), vec![TokenKind::Float(0.0), TokenKind::Term, TokenKind::Eof]);
        assert_eq!(kinds("-0.0"), vec![TokenKind::Minus, TokenKind::Float(0.0), TokenKind::Term, TokenKind::Eof]);
        // A subnormal: below f64::MIN_POSITIVE, above zero, and representable.
        let (tokens, diags) = lex(0, "1.0e-320");
        assert!(diags.is_empty(), "a subnormal is representable: {diags:?}");
        match tokens[0].kind {
            TokenKind::Float(v) => assert!(v > 0.0 && v < f64::MIN_POSITIVE, "expected a subnormal, got {v}"),
            ref k => panic!("expected a float, got {k:?}"),
        }
        // And the largest float there is, one ulp below the overflow case above.
        let (tokens, diags) = lex(0, "1.7976931348623157e308");
        assert!(diags.is_empty(), "f64::MAX is representable: {diags:?}");
        assert_eq!(tokens[0].kind, TokenKind::Float(f64::MAX));
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

    // ----- Trojan Source / bidi controls (HARDENING_CAMPAIGN C3, DL0107) ------

    #[test]
    fn a_raw_bidi_override_is_dl0107() {
        // Before this rule the file below checked clean: the override lives in a comment, which
        // the tokenizer skips wholesale, so nothing examined it. That is the Trojan Source attack.
        let (_, diags) = lex(0, "module m\n// deny if \u{202e} not admin\nfn f() {}\n");
        assert!(diags.iter().any(|d| d.code == "DL0107"), "expected DL0107, got {diags:?}");
    }

    #[test]
    fn every_bidi_control_is_refused_by_name() {
        // The whole set fires, and each carries its own code point in the message — so a reviewer
        // reading the diagnostic learns exactly which character was hiding. A missing entry here
        // is a character the scan would wave through.
        for (ch, name) in BIDI_CONTROLS {
            let src = format!("module m\n// x{ch}y\nfn f() {{}}\n");
            let (_, diags) = lex(0, &src);
            let hit = diags.iter().find(|d| d.code == "DL0107");
            let hit = hit.unwrap_or_else(|| panic!("{name} (U+{:04X}) was not refused", *ch as u32));
            assert!(
                hit.message.contains(&format!("U+{:04X}", *ch as u32)),
                "the message must name the code point: {}",
                hit.message
            );
        }
    }

    // ----- Line-break lookalikes (HARDENING_CAMPAIGN C89, DL0108) ------------

    /// The attack, kept as the exact program that used to check clean and run.
    ///
    /// Rendered in any editor this is four lines and the `println` is one of them. To the lexer the
    /// comment runs to the next `\n`, so the `println` is *inside it* — the reviewer is looking
    /// straight at a line of code the compiler never compiles.
    #[test]
    fn a_line_break_lookalike_hiding_code_in_a_comment_is_dl0108() {
        let src = "module m\nfn f() {\n  // note\u{2028}  danger()\n}\n";
        let (_, diags) = lex(0, src);
        assert!(diags.iter().any(|d| d.code == "DL0108"), "expected DL0108, got {diags:?}");
    }

    #[test]
    fn every_line_break_lookalike_is_refused_by_name() {
        // Same discipline as the bidi table: a missing entry is a character the scan waves through,
        // so assert over the table rather than over a list someone has to remember to extend.
        for (ch, name) in LINE_BREAK_LOOKALIKES {
            let src = format!("module m\n// x{ch}y\nfn f() {{}}\n");
            let (_, diags) = lex(0, &src);
            let hit = diags.iter().find(|d| d.code == "DL0108");
            let hit = hit.unwrap_or_else(|| panic!("{name} (U+{:04X}) was not refused", *ch as u32));
            assert!(
                hit.message.contains(&format!("U+{:04X}", *ch as u32)),
                "the message must name the code point: {}",
                hit.message
            );
        }
    }

    #[test]
    fn a_lone_cr_is_refused_but_crlf_is_a_normal_line_ending() {
        // The pair must survive: `\r\n` is how most of the world's Windows editors write every
        // file, and refusing it would refuse the language on its own primary platform.
        let (_, ok) = lex(0, "module m\r\nfn f() {\r\n  // note\r\n}\r\n");
        assert!(!ok.iter().any(|d| d.code == "DL0108"), "CRLF must be fine: {ok:?}");

        let (_, bad) = lex(0, "module m\nfn f() {\n  // note\r  danger()\n}\n");
        assert!(bad.iter().any(|d| d.code == "DL0108"), "a lone CR must be refused: {bad:?}");
    }

    #[test]
    fn an_escaped_line_break_lookalike_is_allowed() {
        // The same escape hatch DL0107 has, for the same reason: the scan reads RAW source, so a
        // string that genuinely needs the code point writes it visibly.
        let (_, diags) = lex(0, "module m\nfn f() -> Str { \"a\\u{2028}b\" }\n");
        assert!(!diags.iter().any(|d| d.code == "DL0108"), "an escaped code point must not fire: {diags:?}");
    }

    #[test]
    fn an_escaped_bidi_code_point_is_allowed() {
        // The scan reads RAW source, so `\u{202e}` — which is ASCII in source and visible to a
        // reviewer — is untouched. This is the escape hatch for the rare legitimate need, and it
        // is exactly what keeps the rule from breaking string DATA that wants the code point.
        let (_, diags) = lex(0, "module m\nfn f() -> Str { \"a\\u{202e}b\" }\n");
        assert!(!diags.iter().any(|d| d.code == "DL0107"), "an escaped control must not fire DL0107: {diags:?}");
    }

    #[test]
    fn right_to_left_letters_are_not_refused() {
        // The refusal is about reordering CONTROL characters, not about right-to-left scripts.
        // Arabic letters render correctly on their own and must stay legal, or the language cannot
        // hold internationalized string data — which would be its own kind of discrimination.
        let (_, diags) = lex(0, "module m\nfn f() -> Str { \"\u{0645}\u{0631}\u{062d}\u{0628}\u{0627}\" }\n");
        assert!(!diags.iter().any(|d| d.code == "DL0107"), "RTL letters must not fire DL0107: {diags:?}");
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
