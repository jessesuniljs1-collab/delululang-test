//! The declarative grammar-production index (Stage 9, invariant 42 — the coverage law).
//!
//! The parser is hand-written recursive descent (`parser.rs`), so there is no grammar table to
//! extract. This module is the machine-readable index of the language's grammar productions: one
//! stable name per user-facing production, each backed by a `fn parse_<name>` in `parser.rs`. It
//! is metadata only. The conformance coverage tool enumerates the anchors `ref.grammar.<name>`
//! from here; the drift guard below fences the list against the parser itself, so a renamed or
//! deleted production can never leave a stale anchor behind.
//!
//! Precedence-layer internals (`parse_bin`/`parse_unary`/`parse_postfix`/`parse_expr_no_struct`/
//! `parse_type_core`) and test helpers are deliberately excluded — they are not distinct grammar
//! productions a conformance program cites, and folding them in would be noise. `expr` covers the
//! expression grammar as one production, as the reference presents it.

/// Every user-facing grammar production, each with a `fn parse_<name>` in `parser.rs`.
pub const GRAMMAR_PRODUCTIONS: &[&str] = &[
    "module",
    "import",
    "path",
    "item",
    "foreign_decl",
    "foreign_fn",
    "actor_decl",
    "generics",
    "params",
    "fn",
    "test_decl",
    "const",
    "effect_decl",
    "type_decl",
    "variant",
    "type",
    "opt_row",
    "block",
    "stmt",
    "expr",
    "args",
    "primary",
    "if",
    "match",
    "pattern",
    "lambda",
];

/// The stable conformance anchor id for a grammar production, e.g. `ref.grammar.match`.
pub fn anchor(production: &str) -> String {
    format!("ref.grammar.{production}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The index is non-empty, and names are unique.
    #[test]
    fn productions_are_unique_and_nonempty() {
        assert!(!GRAMMAR_PRODUCTIONS.is_empty());
        let mut seen = std::collections::HashSet::new();
        for p in GRAMMAR_PRODUCTIONS {
            assert!(!p.is_empty(), "empty production name");
            assert!(seen.insert(*p), "duplicate production {p}");
        }
    }

    /// DRIFT GUARD: every listed production is backed by a `fn parse_<name>` in the parser source.
    /// Renaming or deleting a production without updating this list fails here.
    #[test]
    fn every_production_has_a_parse_fn() {
        let src = include_str!("parser.rs");
        for p in GRAMMAR_PRODUCTIONS {
            let needle = format!("fn parse_{p}(");
            assert!(
                src.contains(&needle),
                "grammar production `{p}` has no `{needle}` in parser.rs — the index drifted"
            );
        }
    }

    /// THE SKIP-BRANCH CASE (house rule 3): the drift guard must actually be able to FAIL. A name
    /// with no `fn parse_*` must not be found — proving `every_production_has_a_parse_fn` is not
    /// vacuously true (e.g. matching against an empty source or a too-loose needle).
    #[test]
    fn a_bogus_production_is_not_found() {
        let src = include_str!("parser.rs");
        assert!(
            !src.contains("fn parse_definitely_not_a_production_9a("),
            "the drift guard's needle is too loose to ever fail"
        );
    }
}
