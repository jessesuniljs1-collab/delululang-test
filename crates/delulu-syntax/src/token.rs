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

    // Keywords activated in Stage 7 (spec §2). `actor`/`spawn` come off the reserved list;
    // `consume`/`recover` are NEW keywords (a Stage-1 omission, invariant 37) — pre-0.7 code
    // using them as identifiers gets DL1608 with an exact rename repair.
    KwActor,
    KwSpawn,
    KwConsume,
    KwRecover,

    // Keywords activated in Tier 1 (owner-directed 2026-08-08): bounded iteration. `for`/`in`/
    // `break`/`continue` come off the reserved list. `while` already existed; `break`/`continue`
    // now make it (and `for`) breakable.
    KwFor,
    KwIn,
    KwBreak,
    KwContinue,

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
                // Stage 7: a newline after a bare `consume`/`recover` terminates the statement.
                // The legitimate v0.7 forms (`consume x`, `recover { … }`) are single-line, so
                // this exists to make pre-0.7 identifier uses land on DL1608 with a clean span
                // instead of a confusing cross-line parse cascade.
                | TokenKind::KwConsume
                | TokenKind::KwRecover
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
            TokenKind::KwActor => "actor",
            TokenKind::KwSpawn => "spawn",
            TokenKind::KwConsume => "consume",
            TokenKind::KwRecover => "recover",
            TokenKind::KwFor => "for",
            TokenKind::KwIn => "in",
            TokenKind::KwBreak => "break",
            TokenKind::KwContinue => "continue",
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
            TokenKind::KwActor => "`actor`".into(),
            TokenKind::KwSpawn => "`spawn`".into(),
            TokenKind::KwConsume => "`consume`".into(),
            TokenKind::KwRecover => "`recover`".into(),
            TokenKind::KwFor => "`for`".into(),
            TokenKind::KwIn => "`in`".into(),
            TokenKind::KwBreak => "`break`".into(),
            TokenKind::KwContinue => "`continue`".into(),
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
        "actor" => TokenKind::KwActor,
        "spawn" => TokenKind::KwSpawn,
        "consume" => TokenKind::KwConsume,
        "recover" => TokenKind::KwRecover,
        "for" => TokenKind::KwFor,
        "in" => TokenKind::KwIn,
        "break" => TokenKind::KwBreak,
        "continue" => TokenKind::KwContinue,
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
/// Stage 7 removed `actor` and `spawn` (now active keywords). The six reference capabilities
/// STAY reserved: they are recognized *contextually in type position only* (spec §2, build-order
/// deviation 5) — activation is not tokenization — and DL0106 keeps rejecting them as declared
/// names, which is exactly what makes the contextual reading unambiguous.
/// Tier 1 (owner-directed 2026-08-08) removed `for`, `in`, `break` and `continue` — now active
/// keywords for bounded iteration.
///
/// **Reserved here means "not usable as an identifier". It does not mean "unbuilt", and this
/// comment used to conflate the two.** The list below splits on that line:
///
/// - **Reserved AND built:** the six Pony reference capabilities `iso`/`val`/`ref`/`box`/`trn`/
///   `tag`. All six parse as a contextual type prefix (`parse_type_prefixed`, build-order
///   deviation 5) and all six carry full deny tables in `delulu_check::rcaps`. They are reserved
///   as identifiers precisely *because* they are live in type position. Until 2026-08-23 this
///   comment called `ref`/`box`/`trn` "genuinely unbuilt", which was wrong in a way a reader
///   could act on — and singled out three of six words that are in every respect alike.
/// - **Reserved and genuinely unbuilt:** `async`/`await` (Async is an effect, not surface syntax —
///   Constitution decision 12 rejected a parallel async system), `trait`/`impl`/`where` (no
///   typeclass design exists), and `pure` (redundant with `!{}`).
/// - **Reserved as namespace guards:** `plugin`, `secret`, `cap` — the authority-bearing type
///   names, kept out of the identifier space so a user declaration cannot shadow them (C23).
pub const RESERVED: &[&str] = &[
    "async", "await", "iso", "val", "ref", "box", "tag", "trn", "plugin",
    "secret", "cap", "trait", "impl", "where",
    "pure",
];

pub fn is_reserved(word: &str) -> bool {
    RESERVED.contains(&word)
}

/// One row of the token index (Stage 9b — the generated reference's token chapter).
pub struct TokenInfo {
    /// The `TokenKind` variant name, exactly as written in the enum.
    pub variant: &'static str,
    /// The literal lexeme, where the token has exactly one spelling.
    pub lexeme: Option<&'static str>,
    /// The token's human description (the one diagnostics use).
    pub describe: String,
}

/// Every `TokenKind` variant, for the generated reference. The drift guard below fences this
/// against the enum itself, so a token added or renamed without updating this list fails the
/// build rather than silently vanishing from the reference.
pub fn token_index() -> Vec<TokenInfo> {
    let rows: Vec<(&'static str, TokenKind)> = vec![
        ("Ident", TokenKind::Ident("name".into())),
        ("Int", TokenKind::Int(0)),
        ("Float", TokenKind::Float(0.0)),
        ("Str", TokenKind::Str(String::new())),
        ("KwFn", TokenKind::KwFn),
        ("KwLet", TokenKind::KwLet),
        ("KwVar", TokenKind::KwVar),
        ("KwIf", TokenKind::KwIf),
        ("KwElse", TokenKind::KwElse),
        ("KwWhile", TokenKind::KwWhile),
        ("KwReturn", TokenKind::KwReturn),
        ("KwMatch", TokenKind::KwMatch),
        ("KwModule", TokenKind::KwModule),
        ("KwImport", TokenKind::KwImport),
        ("KwPub", TokenKind::KwPub),
        ("KwType", TokenKind::KwType),
        ("KwEffect", TokenKind::KwEffect),
        ("KwTrue", TokenKind::KwTrue),
        ("KwFalse", TokenKind::KwFalse),
        ("KwActor", TokenKind::KwActor),
        ("KwSpawn", TokenKind::KwSpawn),
        ("KwConsume", TokenKind::KwConsume),
        ("KwRecover", TokenKind::KwRecover),
        ("KwFor", TokenKind::KwFor),
        ("KwIn", TokenKind::KwIn),
        ("KwBreak", TokenKind::KwBreak),
        ("KwContinue", TokenKind::KwContinue),
        ("LParen", TokenKind::LParen),
        ("RParen", TokenKind::RParen),
        ("LBrace", TokenKind::LBrace),
        ("RBrace", TokenKind::RBrace),
        ("LBracket", TokenKind::LBracket),
        ("RBracket", TokenKind::RBracket),
        ("Comma", TokenKind::Comma),
        ("Dot", TokenKind::Dot),
        ("Colon", TokenKind::Colon),
        ("Arrow", TokenKind::Arrow),
        ("FatArrow", TokenKind::FatArrow),
        ("Bang", TokenKind::Bang),
        ("Question", TokenKind::Question),
        ("Pipe", TokenKind::Pipe),
        ("Underscore", TokenKind::Underscore),
        ("At", TokenKind::At),
        ("Eq", TokenKind::Eq),
        ("EqEq", TokenKind::EqEq),
        ("NotEq", TokenKind::NotEq),
        ("Lt", TokenKind::Lt),
        ("Le", TokenKind::Le),
        ("Gt", TokenKind::Gt),
        ("Ge", TokenKind::Ge),
        ("Plus", TokenKind::Plus),
        ("Minus", TokenKind::Minus),
        ("Star", TokenKind::Star),
        ("Slash", TokenKind::Slash),
        ("Percent", TokenKind::Percent),
        ("AndAnd", TokenKind::AndAnd),
        ("OrOr", TokenKind::OrOr),
        ("Term", TokenKind::Term),
        ("Eof", TokenKind::Eof),
    ];
    rows.into_iter()
        .map(|(variant, kind)| TokenInfo {
            variant,
            lexeme: kind.keyword_lexeme().or_else(|| punctuation_lexeme(&kind)),
            describe: kind.describe(),
        })
        .collect()
}

/// The single spelling of a punctuation token, where it has one.
fn punctuation_lexeme(kind: &TokenKind) -> Option<&'static str> {
    Some(match kind {
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::Comma => ",",
        TokenKind::Dot => ".",
        TokenKind::Colon => ":",
        TokenKind::Arrow => "->",
        TokenKind::FatArrow => "=>",
        TokenKind::Bang => "!",
        TokenKind::Question => "?",
        TokenKind::Pipe => "|",
        TokenKind::Underscore => "_",
        TokenKind::At => "@",
        TokenKind::Eq => "=",
        TokenKind::EqEq => "==",
        TokenKind::NotEq => "!=",
        TokenKind::Lt => "<",
        TokenKind::Le => "<=",
        TokenKind::Gt => ">",
        TokenKind::Ge => ">=",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        TokenKind::AndAnd => "&&",
        TokenKind::OrOr => "||",
        _ => return None,
    })
}

#[cfg(test)]
mod index_tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Parse the `TokenKind` variant names straight out of this file's own source.
    fn enum_variants() -> BTreeSet<String> {
        let src = include_str!("token.rs");
        let start = src.find("pub enum TokenKind {").expect("the enum is declared here");
        let body = &src[start..];
        let end = body.find("\n}").expect("the enum body closes");
        let mut out = BTreeSet::new();
        for line in body[..end].lines().skip(1) {
            let t = line.trim();
            if t.is_empty() || t.starts_with("//") || t.starts_with("#[") {
                continue;
            }
            let name: String =
                t.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() && name.chars().next().unwrap().is_uppercase() {
                out.insert(name);
            }
        }
        out
    }

    /// DRIFT GUARD: the index covers exactly the enum — no missing token, no stale entry.
    #[test]
    fn token_index_covers_every_variant_exactly() {
        let listed: BTreeSet<String> =
            token_index().iter().map(|t| t.variant.to_string()).collect();
        let declared = enum_variants();
        assert_eq!(
            listed, declared,
            "the token index drifted from `TokenKind`\n  missing from index: {:?}\n  stale in index: {:?}",
            declared.difference(&listed).collect::<Vec<_>>(),
            listed.difference(&declared).collect::<Vec<_>>()
        );
    }

    /// THE SKIP-BRANCH CASE (house rule 3): the guard must be able to fail — proving
    /// `enum_variants` really reads the enum rather than returning an empty set that any index
    /// would vacuously satisfy.
    #[test]
    fn the_variant_scraper_actually_finds_variants() {
        let declared = enum_variants();
        assert!(declared.len() > 40, "the scraper found only {} variants — it is not reading the enum", declared.len());
        assert!(declared.contains("KwFn") && declared.contains("Eof"));
        assert!(!declared.contains("DefinitelyNotAToken9b"));
    }

    /// Every keyword in the active keyword table appears in the index with that lexeme.
    #[test]
    fn every_active_keyword_is_indexed_with_its_lexeme() {
        let idx = token_index();
        for word in ["fn", "let", "var", "if", "else", "while", "return", "match", "module",
                     "import", "pub", "type", "effect", "true", "false", "actor", "spawn",
                     "consume", "recover"] {
            assert!(
                idx.iter().any(|t| t.lexeme == Some(word)),
                "active keyword `{word}` is missing from the token index"
            );
        }
    }
}
