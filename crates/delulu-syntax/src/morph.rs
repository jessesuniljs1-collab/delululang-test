//! Surface-syntax morphs (Stage 8 §6.5; `docs/design/SYNTAX_MORPH_SPEC.md`).
//!
//! A **morph** renames DeluluLang's keywords at the surface without changing the program. A human
//! may read and write `函数` where the language says `fn`; an AI may apply a short-alias profile to
//! cut tokenizer cost. The AST is identical under every morph, so every hash, artifact, diagnostic
//! code, and DIR is computed on the **canonical** form — always. Morphs live at the edge: this
//! module and the lexer's alias hook are the only places in the toolchain that know they exist.
//!
//! Three properties make this safe enough to ship, and all three are enforced in [`Morph::new`]
//! rather than documented and hoped for:
//!
//! 1. **Bijective.** Each canonical keyword has at most one alias, and no two keywords share one.
//!    Without this, rendering back to canonical would have to guess.
//! 2. **No alias may be another keyword's canonical spelling.** Bijectivity alone permits
//!    `let = "fn"`, which is a review attack: a file where the word `fn` means `let`, rendering
//!    perfectly, round-tripping perfectly, and lying to every human who reads it. The spec's own
//!    stated law ("no two keywords to the same alias; aliases must not collide with each other")
//!    does **not** forbid it — that is a gap in the spec, closed here and recorded in
//!    `HARDENING_CAMPAIGN.md` C22.
//! 3. **Every alias is exactly one token.** An alias containing whitespace, a comment starter, a
//!    quote, or a bidi control would let a morph restructure the program rather than rename it.
//!    Bidi controls are refused for the same reason the lexer refuses them in source (DL0107): a
//!    morph that renders differently than it lexes defeats the point of having a canonical form.
//!
//! Identifiers, string literals, and comments are never morphed. Identifiers are prose the author
//! chose; strings and comments are prose the *catalog* system localizes (a different mechanism —
//! `LOCALIZATION_PLUGIN_GUIDE.md`).

use std::collections::BTreeMap;

use delulu_diag::{Diagnostic, FileId, Span};

use crate::token::{keyword, TokenKind};

/// What a morph is for. Recorded, never enforced — the mechanism is identical in all three cases,
/// and privileging one audience's surface is exactly what this feature exists not to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MorphKind {
    /// Keywords in a human language (`fn` → `函数`).
    Human,
    /// Short aliases chosen to reduce tokenizer cost. Savings are tokenizer-specific and this
    /// toolchain never claims a number for them (Constitution §5.11).
    Compact,
    /// Anything else an author or an agent prefers.
    Custom,
}

impl MorphKind {
    pub fn parse(s: &str) -> Option<MorphKind> {
        Some(match s {
            "human" => MorphKind::Human,
            "compact" => MorphKind::Compact,
            "custom" => MorphKind::Custom,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            MorphKind::Human => "human",
            MorphKind::Compact => "compact",
            MorphKind::Custom => "custom",
        }
    }
}

/// A validated surface morph. Construction is the only way to get one, and construction validates,
/// so a `Morph` value in hand is a morph that obeys the canonical-form law.
#[derive(Clone, Debug)]
pub struct Morph {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: MorphKind,
    /// canonical keyword → alias, ordered so rendering is deterministic.
    canon_to_alias: BTreeMap<String, String>,
    /// alias → the token it produces. Sorted longest-first at lex time (see [`Self::aliases`]).
    alias_to_kind: Vec<(String, TokenKind)>,
}

/// The canonical keywords a morph may rename: exactly the lexer's active keyword table.
///
/// Contextual keywords (`foreign`, `lib`) are deliberately absent. The lexer emits them as
/// identifiers and the parser recognizes them positionally, so renaming them would be renaming an
/// identifier — which rule 3 of the spec forbids. Reserved-but-inactive words are absent for the
/// same reason: there is no token to rename yet.
pub const MORPHABLE_KEYWORDS: &[&str] = &[
    "fn", "let", "var", "if", "else", "while", "return", "match", "module", "import", "pub", "type",
    "effect", "true", "false", "actor", "spawn", "consume", "recover", "for", "in", "break",
    "continue",
];

/// A reason a morph was refused, carrying the diagnostic code it maps to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MorphError {
    pub code: &'static str,
    pub message: String,
}

impl MorphError {
    fn new(code: &'static str, message: impl Into<String>) -> MorphError {
        MorphError { code, message: message.into() }
    }

    /// Render as a diagnostic against the morph file (spanless: the TOML is not DeluluLang source,
    /// and pointing at a byte offset inside it would imply a source map we do not build).
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::error(self.code, self.message.clone())
    }
}

impl Morph {
    /// Validate and build a morph from `(canonical, alias)` pairs.
    ///
    /// Every rule is checked and *every* violation is reported, not just the first: an author fixing
    /// a morph file wants the whole list, and a morph is refused wholesale anyway.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        kind: MorphKind,
        entries: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Morph, Vec<MorphError>> {
        let id = id.into();
        let mut errors = Vec::new();
        let mut canon_to_alias: BTreeMap<String, String> = BTreeMap::new();
        // alias → the canonical keyword that claimed it, for the collision message.
        let mut claimed: BTreeMap<String, String> = BTreeMap::new();

        for (canon, alias) in entries {
            // DL1713 — the table names something that is not a canonical keyword. Almost always a
            // typo, and silently ignoring it would leave the author believing a rename took effect.
            if !MORPHABLE_KEYWORDS.contains(&canon.as_str()) {
                errors.push(MorphError::new(
                    "DL1713",
                    format!(
                        "morph `{id}`: `{canon}` is not a keyword this language can rename — the renameable keywords are {}",
                        MORPHABLE_KEYWORDS.join(", ")
                    ),
                ));
                continue;
            }
            if let Some(prev) = canon_to_alias.get(&canon) {
                errors.push(MorphError::new(
                    "DL1710",
                    format!("morph `{id}`: keyword `{canon}` is renamed twice, to `{prev}` and to `{alias}`"),
                ));
                continue;
            }
            // DL1712 — the alias must be exactly one token.
            if let Some(why) = alias_defect(&alias) {
                errors.push(MorphError::new(
                    "DL1712",
                    format!("morph `{id}`: alias `{alias}` for `{canon}` is not a single valid token — {why}"),
                ));
                continue;
            }
            // DL1711 — the alias is some OTHER keyword's canonical spelling. Bijective, and a lie.
            if alias != canon && MORPHABLE_KEYWORDS.contains(&alias.as_str()) {
                errors.push(MorphError::new(
                    "DL1711",
                    format!(
                        "morph `{id}`: alias `{alias}` for `{canon}` is the canonical spelling of `{alias}` — \
                         under this morph the word `{alias}` would mean `{canon}`, so a reader of either \
                         surface would be misled"
                    ),
                ));
                continue;
            }
            // DL1710 — two keywords cannot share one alias, or rendering back could not be a function.
            if let Some(other) = claimed.get(&alias) {
                errors.push(MorphError::new(
                    "DL1710",
                    format!("morph `{id}`: `{other}` and `{canon}` both rename to `{alias}` — a morph must be bijective"),
                ));
                continue;
            }
            claimed.insert(alias.clone(), canon.clone());
            canon_to_alias.insert(canon, alias);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // An alias that equals its own canonical spelling is a no-op; keep it out of the maps so
        // rendering does no work and the lexer has one fewer candidate.
        canon_to_alias.retain(|canon, alias| canon != alias);

        let mut alias_to_kind: Vec<(String, TokenKind)> = canon_to_alias
            .iter()
            .filter_map(|(canon, alias)| keyword(canon).map(|k| (alias.clone(), k)))
            .collect();
        // Longest first: with `l` → `let` and `lm` → `match` both present, scanning `lm` must not
        // match `l` and leave a stray `m`. Ties broken by the alias for determinism.
        alias_to_kind.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));

        Ok(Morph {
            id,
            name: name.into(),
            version: version.into(),
            kind,
            canon_to_alias,
            alias_to_kind,
        })
    }

    /// The canonical form of an alias-bearing keyword table, for `morph info`.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.canon_to_alias.iter().map(|(c, a)| (c.as_str(), a.as_str()))
    }

    /// Aliases with the tokens they produce, longest alias first — the lexer's scan order.
    pub fn aliases(&self) -> &[(String, TokenKind)] {
        &self.alias_to_kind
    }

    /// The alias this morph uses for a canonical keyword, if it renames it.
    pub fn alias_of(&self, canonical: &str) -> Option<&str> {
        self.canon_to_alias.get(canonical).map(|s| s.as_str())
    }

    /// Render canonical source **into** this morph's surface.
    ///
    /// Implemented by lexing and then splicing each keyword token's byte range. Everything the
    /// lexer did not classify as a keyword — identifiers, literals, comments, whitespace, layout —
    /// is copied through byte-for-byte, so this cannot reflow, reorder, or reformat a program. Any
    /// lexical error in the input is returned rather than rendered around.
    ///
    /// Refuses (DL1715) when the source uses one of this morph's aliases as an identifier — see
    /// [`Self::alias_collisions`] for why that is not a renderable program.
    pub fn render(&self, file: FileId, src: &str) -> Result<String, Vec<Diagnostic>> {
        let (tokens, diags) = crate::lexer::lex(file, src);
        if diags.iter().any(|d| d.is_error()) {
            return Err(diags);
        }
        let collisions = self.alias_collisions(file, src, &tokens);
        if !collisions.is_empty() {
            return Err(collisions);
        }
        let mut edits: Vec<(usize, usize, &str)> = Vec::new();
        for t in &tokens {
            if let Some(canon) = t.kind.keyword_lexeme() {
                if let Some(alias) = self.canon_to_alias.get(canon) {
                    edits.push((t.span.start as usize, t.span.end as usize, alias.as_str()));
                }
            }
        }
        Ok(splice(src, &edits))
    }

    /// Every place canonical `src` uses one of this morph's aliases as a name (DL1715).
    ///
    /// **Under a morph, the aliases ARE the keywords.** So they are reserved in the morphed surface
    /// for exactly the reason `if` is reserved in the canonical one, and a program that names a
    /// variable `T` cannot be written in a morph where `T` means `type` — any more than it could
    /// name one `if`. Rendering such a program is not a lossy conversion to warn about; it is not a
    /// conversion at all.
    ///
    /// Without this check `render` emitted the name unchanged (identifiers are copied byte-for-byte)
    /// and [`Self::to_canonical`] then lexed it as the keyword and spliced the canonical spelling
    /// over it — so `canonical → morph → canonical` silently turned `let T = 41` into
    /// `let type = 41`, destroying the program while both commands reported success. The identity
    /// law in `SYNTAX_MORPH_SPEC.md` §1 held only for programs that happened not to do this, and the
    /// round-trip property test enumerated the *prefix* hazard (`fnord` under alias `f`) while
    /// missing the *exact-match* one. Campaign finding C82.
    ///
    /// Keyed on every non-keyword token rather than on identifiers specifically: an alias is refused
    /// wherever it would come back as a keyword, so this stays correct if the token set ever grows.
    /// One diagnostic per distinct name, at its first occurrence, because a name used twenty times
    /// is one decision for the author, not twenty.
    fn alias_collisions(&self, file: FileId, src: &str, tokens: &[crate::token::Token]) -> Vec<Diagnostic> {
        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
        let mut out = Vec::new();
        for t in tokens {
            if t.kind.keyword_lexeme().is_some() {
                continue;
            }
            let text = &src[t.span.start as usize..t.span.end as usize];
            let Some((alias, _)) = self.alias_to_kind.iter().find(|(a, _)| a == text) else {
                continue;
            };
            if seen.insert(text, ()).is_some() {
                continue;
            }
            let canon = self
                .canon_to_alias
                .iter()
                .find(|(_, a)| *a == alias)
                .map(|(c, _)| c.as_str())
                .unwrap_or("a keyword");
            out.push(
                Diagnostic::error(
                    "DL1715",
                    format!(
                        "`{text}` is used as a name here, but morph `{}` renames the keyword `{canon}` to `{text}`",
                        self.id
                    ),
                )
                .with_span(
                    Span::new(file, t.span.start, t.span.end),
                    format!(
                        "under this morph `{text}` is the keyword `{canon}`, so this name cannot be \
                         written in it — rename it, or render to a morph that does not use `{text}`"
                    ),
                ),
            );
        }
        out
    }

    /// Render source written in this morph **back** to canonical.
    ///
    /// The mirror of [`Self::render`]: lex with the morph active (so aliases produce keyword tokens)
    /// and splice each keyword token's span with its canonical spelling. Because both directions
    /// splice spans rather than re-print, `canonical → morph → canonical` is byte-identical — for
    /// any input that lexes **and that `render` accepted**. That second condition is load-bearing
    /// and was missing until C82: a program using an alias as a name lexes perfectly well in
    /// canonical, and round-tripping it used to rewrite the name into a keyword. `render` now
    /// refuses it (DL1715), so the identity law holds over exactly the inputs it is stated for.
    pub fn to_canonical(&self, file: FileId, src: &str) -> Result<String, Vec<Diagnostic>> {
        let (tokens, diags) = crate::lexer::lex_with_morph(file, src, Some(self));
        if diags.iter().any(|d| d.is_error()) {
            return Err(diags);
        }
        let mut edits: Vec<(usize, usize, &str)> = Vec::new();
        for t in &tokens {
            if let Some(canon) = t.kind.keyword_lexeme() {
                let written = &src[t.span.start as usize..t.span.end as usize];
                if written != canon {
                    edits.push((t.span.start as usize, t.span.end as usize, canon));
                }
            }
        }
        Ok(splice(src, &edits))
    }
}

/// Apply non-overlapping `(start, end, replacement)` edits to `src`, in order.
fn splice(src: &str, edits: &[(usize, usize, &str)]) -> String {
    let mut out = String::with_capacity(src.len());
    let mut cursor = 0usize;
    for &(start, end, text) in edits {
        if start < cursor {
            continue; // defensive: token spans from one lex pass never overlap
        }
        out.push_str(&src[cursor..start]);
        out.push_str(text);
        cursor = end;
    }
    out.push_str(&src[cursor..]);
    out
}

/// Why an alias is not usable as a single token, or `None` if it is fine.
///
/// Deliberately a denylist of *structural* hazards rather than an allowlist of scripts: the point of
/// this feature is that an author — or an AI — may choose a surface we did not think of, including
/// scripts and emoji, so the rule is "it must be one token and it must not be able to restructure
/// the program", not "it must look like something we recognize".
fn alias_defect(alias: &str) -> Option<&'static str> {
    if alias.is_empty() {
        return Some("it is empty");
    }
    if alias.chars().any(char::is_whitespace) {
        return Some("it contains whitespace, so it would lex as more than one token");
    }
    // A bidi control inside an alias is the D26 attack wearing a different hat: the morph would
    // render one way and lex another, which is precisely what the canonical form exists to prevent.
    if alias.chars().any(is_bidi_control) {
        return Some("it contains a bidirectional control character (see DL0107)");
    }
    if alias.chars().any(|c| c.is_control()) {
        return Some("it contains a control character");
    }
    // Anything that could start a comment, a string, or a statement break would let the alias end
    // the token it is standing in for and begin something else.
    for bad in ["//", "/*", "*/"] {
        if alias.contains(bad) {
            return Some("it contains a comment delimiter");
        }
    }
    if alias.contains('"') || alias.contains('\'') {
        return Some("it contains a quote character");
    }
    if alias.contains(';') || alias.contains('\\') {
        return Some("it contains a statement separator or an escape character");
    }
    // A leading ASCII digit would lex as a number before any alias match could apply.
    if alias.starts_with(|c: char| c.is_ascii_digit()) {
        return Some("it starts with a digit, which lexes as a number");
    }
    None
}

/// The bidi controls the lexer refuses (DL0107), duplicated as a predicate so this module does not
/// depend on the lexer's private table.
fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// The morph a file declares, from a first-line `//! morph: <id>` pragma.
///
/// Only the first line is consulted, and only a `//!` comment: a pragma that could appear anywhere
/// would be a pragma a reviewer could miss. A file with no pragma is canonical, which is what makes
/// canonical the default for every tool that does not opt in.
pub fn pragma_of(src: &str) -> Option<&str> {
    let first = src.strip_prefix('\u{feff}').unwrap_or(src).lines().next()?;
    let rest = first.trim().strip_prefix("//!")?.trim();
    let id = rest.strip_prefix("morph:")?.trim();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// A `Span` covering the pragma line, for diagnostics about it.
pub fn pragma_span(file: FileId, src: &str) -> Span {
    let end = src.find('\n').unwrap_or(src.len()) as u32;
    Span::new(file, 0, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    fn morph(pairs: &[(&str, &str)]) -> Result<Morph, Vec<MorphError>> {
        Morph::new("t", "test", "1.0.0", MorphKind::Compact, entries(pairs))
    }

    fn codes(e: &[MorphError]) -> Vec<&str> {
        e.iter().map(|x| x.code).collect()
    }

    #[test]
    fn a_valid_compact_morph_loads() {
        let m = morph(&[("fn", "f"), ("let", "l"), ("return", "r")]).expect("valid");
        assert_eq!(m.alias_of("fn"), Some("f"));
        assert_eq!(m.alias_of("match"), None, "an unlisted keyword keeps its canonical spelling");
    }

    #[test]
    fn two_keywords_may_not_share_an_alias() {
        // Without bijectivity, rendering back to canonical would have to guess which one was meant.
        let e = morph(&[("fn", "x"), ("let", "x")]).expect_err("must be refused");
        assert_eq!(codes(&e), vec!["DL1710"]);
        assert!(e[0].message.contains("bijective"), "{}", e[0].message);
    }

    #[test]
    fn an_alias_may_not_be_another_keywords_canonical_spelling() {
        // THE review attack, and the one the spec's own stated law does not forbid: this morph is
        // bijective and every alias is a single token, yet a file written in it uses the word `fn`
        // to mean `let`. It renders and round-trips perfectly while lying to every reader.
        let e = morph(&[("let", "fn")]).expect_err("must be refused");
        assert_eq!(codes(&e), vec!["DL1711"]);
        assert!(e[0].message.contains("misled"), "{}", e[0].message);
    }

    #[test]
    fn an_alias_must_be_a_single_token() {
        for (alias, needle) in [
            ("", "empty"),
            ("a b", "whitespace"),
            ("a//b", "comment"),
            ("a\"b", "quote"),
            ("a;b", "separator"),
            ("1f", "digit"),
            ("a\u{202e}b", "bidirectional"),
        ] {
            let e = morph(&[("fn", alias)]).expect_err("must be refused: {alias:?}");
            assert_eq!(codes(&e), vec!["DL1712"], "alias {alias:?}");
            assert!(e[0].message.contains(needle), "alias {alias:?}: {}", e[0].message);
        }
    }

    #[test]
    fn a_morph_may_not_rename_something_that_is_not_a_keyword() {
        // Includes the contextual keywords: `foreign` is lexed as an identifier, so renaming it
        // would be renaming an identifier, which the canonical-form law forbids.
        for canon in ["fnn", "foreign", "lib", "Int"] {
            let e = morph(&[(canon, "z")]).expect_err("must be refused");
            assert_eq!(codes(&e), vec!["DL1713"], "canonical {canon:?}");
        }
    }

    #[test]
    fn every_violation_is_reported_not_just_the_first() {
        let e = morph(&[("nope", "a"), ("let", "fn"), ("fn", "b c")]).expect_err("must be refused");
        assert_eq!(codes(&e), vec!["DL1713", "DL1711", "DL1712"]);
    }

    /// The canonical program every round-trip test uses. It deliberately contains the hazards:
    /// an identifier that *starts with* a compact alias (`fnord`, `letter`), a string literal
    /// containing keywords, a comment containing keywords, and nested blocks.
    const PROGRAM: &str = "module m\n\
        // a comment mentioning fn and let and match\n\
        pub fn fnord(letter: Int) -> Int {\n\
        \x20 let msg = \"fn let match return\"\n\
        \x20 if letter > 0 { return letter } else { return 0 }\n\
        }\n";

    fn round_trip(pairs: &[(&str, &str)]) {
        let m = morph(pairs).expect("valid morph");
        let morphed = m.render(0, PROGRAM).expect("render");
        let back = m.to_canonical(0, &morphed).expect("to_canonical");
        assert_eq!(
            back, PROGRAM,
            "identity law: canonical -> morph -> canonical must be byte-identical\nmorphed was:\n{morphed}"
        );
        // And the morphed form must be a DIFFERENT surface, or the test proves nothing.
        assert_ne!(morphed, PROGRAM, "the morph must actually change the surface");
    }

    #[test]
    fn a_compact_ascii_morph_round_trips() {
        // The token-cost case: short ASCII aliases. `f` for `fn` is the dangerous one — `fnord` and
        // `letter` in PROGRAM both begin with an alias and must survive untouched.
        round_trip(&[("fn", "f"), ("let", "l"), ("return", "r"), ("if", "i"), ("else", "e")]);
    }

    #[test]
    fn a_chinese_keyword_morph_round_trips() {
        // Identifiers are ASCII-only in this language, so a CJK alias can never collide with one.
        round_trip(&[
            ("fn", "函数"),
            ("let", "令"),
            ("return", "返回"),
            ("if", "如果"),
            ("else", "否则"),
            ("module", "模块"),
            ("pub", "公开"),
        ]);
    }

    #[test]
    fn an_emoji_morph_round_trips() {
        round_trip(&[("fn", "🔧"), ("let", "📌"), ("return", "↩"), ("if", "❓"), ("else", "🔀")]);
    }

    #[test]
    fn a_mixed_surface_round_trips() {
        // Jesse's actual ask: "Chinese or other characters or symbols or emoji's or mix of
        // everything." Nothing in the mechanism cares which script an alias comes from.
        round_trip(&[
            ("fn", "λ"),
            ("let", "令"),
            ("return", "⏎"),
            ("if", "🤔"),
            ("else", "sinon"),
            ("module", "модуль"),
        ]);
    }

    #[test]
    fn identifiers_strings_and_comments_are_never_morphed() {
        // The property the whole design rests on. Under a compact morph, PROGRAM's identifier
        // `fnord`, its string "fn let match return", and its comment must all appear verbatim in
        // the morphed output — only the keyword TOKENS move.
        // `match` is aliased to `mt`, not `m`: PROGRAM's module is named `m`, and under an `m`
        // alias this program cannot be written at all (DL1715 — see the C82 witness below). That
        // collision sat in this very test until C82 found it, invisible because this test renders
        // one direction and asserts `contains`, while the test that round-trips used alias sets
        // without `m`. Two tests straddled the defect.
        let m = morph(&[("fn", "f"), ("let", "l"), ("return", "r"), ("match", "mt")]).unwrap();
        let out = m.render(0, PROGRAM).expect("render");
        assert!(out.contains("fnord"), "identifier must survive:\n{out}");
        assert!(out.contains("letter"), "identifier must survive:\n{out}");
        assert!(out.contains("\"fn let match return\""), "string literal must survive:\n{out}");
        assert!(
            out.contains("// a comment mentioning fn and let and match"),
            "comment must survive:\n{out}"
        );
        // And the declaration keyword really did change.
        assert!(out.contains("pub f fnord"), "the `fn` keyword must be morphed:\n{out}");
    }

    #[test]
    fn a_morphed_program_parses_to_the_same_ast_as_its_canonical_form() {
        // The claim that matters: a morph is a surface, not a dialect. Both forms must produce the
        // same AST — compared through the formatter's identity fingerprint, which strips spans and
        // node ids precisely so two parses of the same program compare equal.
        let m = morph(&[("fn", "函数"), ("let", "令"), ("return", "返回"), ("if", "如果"), ("else", "否则")])
            .unwrap();
        let morphed = m.render(0, PROGRAM).expect("render");
        let canonical_again = m.to_canonical(0, &morphed).expect("to_canonical");
        let a = crate::parse_file(0, PROGRAM).0;
        let b = crate::parse_file(0, &canonical_again).0;
        assert_eq!(
            crate::fmt::ast_fingerprint(&a),
            crate::fmt::ast_fingerprint(&b),
            "a morphed program must parse to the same AST as its canonical form"
        );
    }

    #[test]
    fn an_alias_that_prefixes_an_identifier_does_not_eat_it() {
        // The maximal-munch guard, isolated. With `f` = `fn`, the identifier `foo` must stay `foo`
        // and must NOT lex as `fn` + `oo`.
        let m = morph(&[("fn", "f")]).unwrap();
        let src = "module m\nf foo() -> Int { 1 }\n";
        let canonical = m.to_canonical(0, src).expect("to_canonical");
        assert_eq!(canonical, "module m\nfn foo() -> Int { 1 }\n");
    }

    /// Campaign finding C82 — the witness, kept as the exact case that was silently corrupted.
    #[test]
    fn an_alias_used_as_a_name_is_refused() {
        // PROGRAM's module is named `m`. Under a morph where `m` means `match`, rendering used to
        // emit `module m` unchanged (names are copied byte-for-byte) and reading it back lexed that
        // `m` as the KEYWORD — so canonical → morph → canonical rewrote the program while both
        // directions reported success.
        let m = morph(&[("match", "m")]).unwrap();
        let e = m.render(0, PROGRAM).expect_err("a name that is an alias must be refused");
        assert_eq!(e.iter().map(|d| d.code).collect::<Vec<_>>(), vec!["DL1715"]);
        assert!(e[0].message.contains("`m`"), "must name the identifier: {}", e[0].message);
        assert!(e[0].message.contains("match"), "must name the keyword: {}", e[0].message);
    }

    /// A name is reported once, however often it is used — one decision for the author.
    #[test]
    fn a_repeated_colliding_name_is_reported_once() {
        let m = morph(&[("type", "T")]).unwrap();
        let src = "module a\nfn g(T: Int) -> Int { T + T }\n";
        let e = m.render(0, src).expect_err("must be refused");
        assert_eq!(e.len(), 1, "one diagnostic per distinct name, got {e:#?}");
    }

    /// Whether an alias could collide is decided by the alias's SHAPE, so assert it over the whole
    /// alias set rather than over a list of hazards someone has to remember to extend.
    #[test]
    fn every_identifier_shaped_alias_is_reserved_in_its_morph() {
        // Identifiers here are ASCII alphanumeric/underscore, not digit-initial.
        fn identifier_shaped(s: &str) -> bool {
            !s.is_empty()
                && !s.starts_with(|c: char| c.is_ascii_digit())
                && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        // Deliberately mixed: ASCII compact aliases CAN collide; CJK/emoji ones CANNOT, because
        // identifiers are ASCII-only. Both halves must come out right.
        let pairs: &[(&str, &str)] =
            &[("fn", "F"), ("let", "L"), ("type", "T"), ("match", "函数"), ("if", "🤔")];
        let m = morph(pairs).unwrap();
        for (canon, alias) in pairs {
            let src = format!("module a\nfn g({alias}: Int) -> Int {{ {alias} }}\n");
            if identifier_shaped(alias) {
                let e = m.render(0, &src).unwrap_err();
                assert_eq!(
                    e.iter().map(|d| d.code).collect::<Vec<_>>(),
                    vec!["DL1715"],
                    "alias {alias:?} for {canon:?} is identifier-shaped and must be reserved"
                );
            } else {
                // Not identifier-shaped, so this source does not even lex as canonical — which is
                // the reason such an alias is safe. Assert that, rather than asserting nothing.
                assert!(
                    crate::lexer::lex(0, &src).1.iter().any(|d| d.is_error()),
                    "alias {alias:?} is not identifier-shaped, so it cannot appear as a name"
                );
            }
        }
    }

    /// The identity law itself, not an enumeration of the hazards someone thought of.
    ///
    /// `SYNTAX_MORPH_SPEC.md` §1 promises `canonical → morph → canonical` is the identity. The old
    /// gate proved that for ONE program against alias sets that happened not to collide with its
    /// names (C82). This asserts the law over a matrix: for every program and every morph, render
    /// must either REFUSE or round-trip byte-for-byte. A future hazard of a shape nobody predicted
    /// still has to land in one of those two buckets, so this cannot go blind the same way.
    #[test]
    fn the_identity_law_holds_for_every_input_render_accepts() {
        let morphs = [
            vec![("fn", "F"), ("let", "L"), ("type", "T"), ("else", "E"), ("match", "M")],
            vec![("fn", "f"), ("let", "l"), ("return", "r"), ("if", "i"), ("else", "e")],
            vec![("fn", "函数"), ("let", "令"), ("return", "返回")],
            vec![("fn", "🔧"), ("let", "📌")],
        ];
        let programs = [
            PROGRAM,
            // names that ARE aliases above — the C82 shape, in several positions
            "module M\nfn g(T: Int) -> Int { let E = T\n E }\n",
            "module a\nfn F() -> Int { 1 }\n",
            // names that merely CONTAIN or PREFIX an alias — must round-trip, not be refused
            "module a\nfn Fold(Length: Int) -> Int { let Ts = Length\n Ts }\n",
            // keywords inside prose, which must never move
            "module a\n// fn let type else match\nfn g() -> Str { \"fn let type\" }\n",
        ];
        let mut refused = 0usize;
        let mut round_tripped = 0usize;
        for pairs in &morphs {
            let m = morph(pairs).unwrap();
            for src in &programs {
                match m.render(0, src) {
                    Err(diags) => {
                        assert!(
                            diags.iter().all(|d| d.code == "DL1715"),
                            "the only reason to refuse a lexing program is a name collision: {diags:#?}"
                        );
                        refused += 1;
                    }
                    Ok(morphed) => {
                        let back = m.to_canonical(0, &morphed).expect("accepted render must read back");
                        assert_eq!(
                            &back, src,
                            "identity law broken for morph {:?}\nmorphed was:\n{morphed}",
                            m.id
                        );
                        round_tripped += 1;
                    }
                }
            }
        }
        // Non-vacuity in BOTH directions: if nothing were refused the check would be dead, and if
        // nothing round-tripped the law would be untested.
        assert!(refused > 0, "no input was refused — the DL1715 check is not being exercised");
        assert!(round_tripped > 0, "no input round-tripped — the identity law is not being tested");
    }

    #[test]
    fn a_pragma_is_read_only_from_the_first_line() {
        assert_eq!(pragma_of("//! morph: zh-CN\nmodule m\n"), Some("zh-CN"));
        assert_eq!(pragma_of("//!morph:zh-CN\n"), Some("zh-CN"));
        assert_eq!(pragma_of("module m\n//! morph: zh-CN\n"), None, "not the first line");
        assert_eq!(pragma_of("// morph: zh-CN\n"), None, "`//` is an ordinary comment");
        assert_eq!(pragma_of("//! morph:\n"), None, "an empty id is not a pragma");
        assert_eq!(pragma_of("module m\n"), None, "no pragma means canonical");
    }
}
