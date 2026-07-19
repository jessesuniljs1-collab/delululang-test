<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_syntax::grammar::GRAMMAR_PRODUCTIONS` (fenced against `parser.rs`).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# Grammar productions

The parser is hand-written recursive descent, so this is the index of its productions: one row per user-facing production, each backed by a `parse_<name>` function that a drift guard verifies exists.

> **Coverage (invariant 42):** 27 of 27 anchors in this chapter have both an accepting and a rejecting conformance witness (100.0%). Items marked otherwise are **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.

| Production | Anchor | Coverage |
|---|---|---|
| `module` | `ref.grammar.module` | covered |
| `import` | `ref.grammar.import` | covered |
| `path` | `ref.grammar.path` | covered |
| `item` | `ref.grammar.item` | covered |
| `attribute` | `ref.grammar.attribute` | covered |
| `foreign_decl` | `ref.grammar.foreign_decl` | covered |
| `foreign_fn` | `ref.grammar.foreign_fn` | covered |
| `actor_decl` | `ref.grammar.actor_decl` | covered |
| `generics` | `ref.grammar.generics` | covered |
| `params` | `ref.grammar.params` | covered |
| `fn` | `ref.grammar.fn` | covered |
| `test_decl` | `ref.grammar.test_decl` | covered |
| `const` | `ref.grammar.const` | covered |
| `effect_decl` | `ref.grammar.effect_decl` | covered |
| `type_decl` | `ref.grammar.type_decl` | covered |
| `variant` | `ref.grammar.variant` | covered |
| `type` | `ref.grammar.type` | covered |
| `opt_row` | `ref.grammar.opt_row` | covered |
| `block` | `ref.grammar.block` | covered |
| `stmt` | `ref.grammar.stmt` | covered |
| `expr` | `ref.grammar.expr` | covered |
| `args` | `ref.grammar.args` | covered |
| `primary` | `ref.grammar.primary` | covered |
| `if` | `ref.grammar.if` | covered |
| `match` | `ref.grammar.match` | covered |
| `pattern` | `ref.grammar.pattern` | covered |
| `lambda` | `ref.grammar.lambda` | covered |
