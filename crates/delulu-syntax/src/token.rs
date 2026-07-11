//! The Stage-1 token model (spec §2).

use delulu_diag::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // Literals and identifiers
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),

    // Keywords (active in Stage 1)
    KwFn,
    KwLet,
    KwVar,
    KwIf,
    KwElse,
    KwWhile,
    KwReturn,
    KwMatch,
    KwModule,
    KwImport,
    KwPub,
    KwType,
    KwEffect,
    KwTrue,
    KwFalse,

    // Punctuation and operators
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Arrow,     // ->
    FatArrow,  // =>
    Bang,      // !   (effect-row marker in types; logical not in expressions)
    Question,  // ?   (Result propagation)
    Pipe,      // |   (sum variants; row-variable tail)
    Underscore,
    At,        // @   (attributes; reserved token, no attribute grammar in Stage 1)
    Eq,        // =
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    AndAnd,
    OrOr,

    /// Statement terminator: an explicit `;` or one inserted at a newline (§2.2).
    Term,

    Eof,
}

impl TokenKind {
    /// §2.2: tokens after which a newline inserts a terminator.
    pub fn can_end_statement(&self) -> bool {
        matches!(
            self,
            TokenKind::Ident(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::Str(_)
                | TokenKind::KwTrue
                | TokenKind::KwFalse
                | TokenKind::KwReturn
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Question
        )
    }

    /// The source lexeme of a keyword token, if this is one. Used so a reserved word may still be a
    /// **member name** after `.` (e.g. `py.import(…)` — spec §5.2 — or a future `x.match`): member
    /// position is not a declaration site, so keywords are unambiguous there.
    pub fn keyword_lexeme(&self) -> Option<&'static str> {
        Some(match self {
            TokenKind::KwFn => "fn",
            TokenKind::KwLet => "let",
            TokenKind::KwVar => "var",
            TokenKind::KwIf => "if",
            TokenKind::KwElse => "else",
            TokenKind::KwWhile => "while",
            TokenKind::KwReturn => "return",
            TokenKind::KwMatch => "match",
            TokenKind::KwModule => "module",
            TokenKind::KwImport => "import",
            TokenKind::KwPub => "pub",
            TokenKind::KwType => "type",
            TokenKind::KwEffect => "effect",
            TokenKind::KwTrue => "true",
            TokenKind::KwFalse => "false",
            _ => return None,
        })
    }

    /// Human-facing description for "expected X, found Y" diagnostics.
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Ident(name) => format!("identifier `{name}`"),
            TokenKind::Int(v) => format!("integer `{v}`"),
            TokenKind::Float(v) => format!("float `{v}`"),
            TokenKind::Str(_) => "string literal".to_string(),
            TokenKind::KwFn => "`fn`".into(),
            TokenKind::KwLet => "`let`".into(),
            TokenKind::KwVar => "`var`".into(),
            TokenKind::KwIf => "`if`".into(),
            TokenKind::KwElse => "`else`".into(),
            TokenKind::KwWhile => "`while`".into(),
            TokenKind::KwReturn => "`return`".into(),
            TokenKind::KwMatch => "`match`".into(),
            TokenKind::KwModule => "`module`".into(),
            TokenKind::KwImport => "`import`".into(),
            TokenKind::KwPub => "`pub`".into(),
            TokenKind::KwType => "`type`".into(),
            TokenKind::KwEffect => "`effect`".into(),
            TokenKind::KwTrue => "`true`".into(),
            TokenKind::KwFalse => "`false`".into(),
            TokenKind::LParen => "`(`".into(),
            TokenKind::RParen => "`)`".into(),
            TokenKind::LBrace => "`{`".into(),
            TokenKind::RBrace => "`}`".into(),
            TokenKind::LBracket => "`[`".into(),
            TokenKind::RBracket => "`]`".into(),
            TokenKind::Comma => "`,`".into(),
            TokenKind::Dot => "`.`".into(),
            TokenKind::Colon => "`:`".into(),
            TokenKind::Arrow => "`->`".into(),
            TokenKind::FatArrow => "`=>`".into(),
            TokenKind::Bang => "`!`".into(),
            TokenKind::Question => "`?`".into(),
            TokenKind::Pipe => "`|`".into(),
            TokenKind::Underscore => "`_`".into(),
            TokenKind::At => "`@`".into(),
            TokenKind::Eq => "`=`".into(),
            TokenKind::EqEq => "`==`".into(),
            TokenKind::NotEq => "`!=`".into(),
            TokenKind::Lt => "`<`".into(),
            TokenKind::Le => "`<=`".into(),
            TokenKind::Gt => "`>`".into(),
            TokenKind::Ge => "`>=`".into(),
            TokenKind::Plus => "`+`".into(),
            TokenKind::Minus => "`-`".into(),
            TokenKind::Star => "`*`".into(),
            TokenKind::Slash => "`/`".into(),
            TokenKind::Percent => "`%`".into(),
            TokenKind::AndAnd => "`&&`".into(),
            TokenKind::OrOr => "`||`".into(),
            TokenKind::Term => "end of statement".into(),
            TokenKind::Eof => "end of file".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Active keyword table (§2.3).
pub fn keyword(word: &str) -> Option<TokenKind> {
    Some(match word {
        "fn" => TokenKind::KwFn,
        "let" => TokenKind::KwLet,
        "var" => TokenKind::KwVar,
        "if" => TokenKind::KwIf,
        "else" => TokenKind::KwElse,
        "while" => TokenKind::KwWhile,
        "return" => TokenKind::KwReturn,
        "match" => TokenKind::KwMatch,
        "module" => TokenKind::KwModule,
        "import" => TokenKind::KwImport,
        "pub" => TokenKind::KwPub,
        "type" => TokenKind::KwType,
        "effect" => TokenKind::KwEffect,
        "true" => TokenKind::KwTrue,
        "false" => TokenKind::KwFalse,
        _ => return None,
    })
}

/// Words reserved for future stages (§2.3). The lexer emits them as identifiers;
/// the parser rejects them at declaration sites (DL0106) — member names after `.`
/// remain legal, which is how `root.secret(…)` coexists with `secret` reserved.
///
/// `foreign` (Stage 4) is NOT in this list: it is an active **contextual** keyword. It is still
/// lexed as an identifier (so `root.foreign(…)` stays a legal member access, and `lib`/the ABI
/// string need no new tokens), and the parser recognizes `foreign STRING lib IDENT { … }` as a
/// declaration at item position (spec §2).
pub const RESERVED: &[&str] = &[
    "actor", "async", "await", "spawn", "iso", "val", "ref", "box", "tag", "trn", "plugin",
    "secret", "cap", "for", "in", "break", "continue", "trait", "impl", "where",
    "pure",
];

pub fn is_reserved(word: &str) -> bool {
    RESERVED.contains(&word)
}
