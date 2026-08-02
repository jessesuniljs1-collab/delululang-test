<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_syntax::grammar::GRAMMAR_PRODUCTIONS` (fenced against `parser.rs`).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# Grammar productions

The parser is hand-written recursive descent, so this is the index of its productions: one row per user-facing production, each backed by a `parse_<name>` function that a drift guard verifies exists.

**This is an index, not the grammar.** The normative EBNF lives in the stage specifications — `STAGE1_SPECIFICATION.md` §3 carries the base language and each later stage's *lexical and grammar additions* section extends it. The **Defined as** column gives the name to look for there, because six of these productions are spelled differently in the two places: the names here follow `parser.rs`, which is what the drift guard fences them against, while the specifications use the fuller spellings a reader expects. A test (`every_grammar_production_is_defined_in_a_normative_specification`) checks in both directions that every anchor below leads to a real production — an index that does not lead anywhere is not an index.

> **Coverage (invariant 42):** 27 of 27 anchors in this chapter have both an accepting and a rejecting conformance witness (100.0%). Items marked otherwise are **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.

| Production | Defined as | Anchor | Coverage |
|---|---|---|---|
| `module` | `module_decl` | `ref.grammar.module` | covered |
| `import` | `import_decl` | `ref.grammar.import` | covered |
| `path` | — | `ref.grammar.path` | covered |
| `item` | — | `ref.grammar.item` | covered |
| `attribute` | — | `ref.grammar.attribute` | covered |
| `foreign_decl` | — | `ref.grammar.foreign_decl` | covered |
| `foreign_fn` | — | `ref.grammar.foreign_fn` | covered |
| `actor_decl` | — | `ref.grammar.actor_decl` | covered |
| `generics` | — | `ref.grammar.generics` | covered |
| `params` | — | `ref.grammar.params` | covered |
| `fn` | — | `ref.grammar.fn` | covered |
| `test_decl` | — | `ref.grammar.test_decl` | covered |
| `const` | `const_decl` | `ref.grammar.const` | covered |
| `effect_decl` | — | `ref.grammar.effect_decl` | covered |
| `type_decl` | — | `ref.grammar.type_decl` | covered |
| `variant` | — | `ref.grammar.variant` | covered |
| `type` | — | `ref.grammar.type` | covered |
| `opt_row` | `effect_row` | `ref.grammar.opt_row` | covered |
| `block` | — | `ref.grammar.block` | covered |
| `stmt` | — | `ref.grammar.stmt` | covered |
| `expr` | — | `ref.grammar.expr` | covered |
| `args` | `call` | `ref.grammar.args` | covered |
| `primary` | — | `ref.grammar.primary` | covered |
| `if` | `if_expr` | `ref.grammar.if` | covered |
| `match` | — | `ref.grammar.match` | covered |
| `pattern` | — | `ref.grammar.pattern` | covered |
| `lambda` | — | `ref.grammar.lambda` | covered |
