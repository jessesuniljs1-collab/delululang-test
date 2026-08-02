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
    "attribute",
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

/// Where a production is written down, when the **normative grammar calls it something else**.
///
/// The names in [`GRAMMAR_PRODUCTIONS`] follow `parser.rs` — they exist to be fenced against a
/// `fn parse_<name>`. The normative EBNF in the stage specifications was written for a reader, and
/// uses the fuller `_decl` / `_expr` spellings. Six of the twenty-seven diverge, which meant the
/// reference published `ref.grammar.args` as a citable anchor while nothing in any specification
/// defined anything called `args`. **An index that does not lead anywhere is not an index**, and a
/// conformance anchor whose grammar cannot be found is worse than one that does not exist, because
/// a witness can cite it and look satisfied.
///
/// Only the divergences are listed; a production absent from this table is spelled the same in both
/// places. `delulu-conform` checks the table in **both** directions against the specification text —
/// every production must resolve, and no entry here may name a production the specs no longer use.
pub const NORMATIVE_NAME: &[(&str, &str)] = &[
    ("module", "module_decl"),
    ("import", "import_decl"),
    ("const", "const_decl"),
    ("if", "if_expr"),
    // `parse_args` reads the parenthesised argument list; the specs define that shape as `call`.
    ("args", "call"),
    // `parse_opt_row` reads the optional `! { … }` effect clause, spelled `effect_row` normatively.
    ("opt_row", "effect_row"),
];

/// The name the normative grammar uses for `production` — itself, unless [`NORMATIVE_NAME`] says
/// otherwise.
pub fn normative_name(production: &str) -> &str {
    NORMATIVE_NAME
        .iter()
        .find(|(p, _)| *p == production)
        .map(|(_, n)| *n)
        .unwrap_or(production)
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
