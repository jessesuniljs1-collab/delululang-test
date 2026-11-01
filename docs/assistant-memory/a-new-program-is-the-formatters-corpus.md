---
name: a-new-program-is-the-formatters-corpus
description: "FMT-CTOR-1 (2026-11-01): a new .delulu file in tests/conformance is also delulu fmt's identity-law corpus; adding one means running delulu-syntax's tests — #112 skipped it and went red on CI, and the law had found a real defect (fmt printed foo() as foo, a binding)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-01T07:10:33.936Z
---

A conformance program added for RW 2.26 (`accept/lowercase_constructor_call_form.delulu`) made
`delulu-syntax`'s `laws_hold_over_the_repository_corpus` red on Ubuntu + macOS (#112's first run `41282442664`): the
formatter printed a nullary Variant pattern bare, so `foo()` → `foo`, which the parser reads as a BINDING — `delulu fmt`
changed the program's meaning (defect since 1.x, invisible until a lowercase constructor's call form was in the corpus).

**Why:** the laptop ran the checker's and CLI's targets for the change, not `delulu-syntax`'s, because the change
"didn't touch the syntax crate". The corpus law reads every `.delulu` file in the repo.

**How to apply:** when a change adds or edits any `.delulu` file (conformance, corpus, examples, book), also run
`cargo test -p delulu-syntax --lib` (formatter laws) and `core_invariance`. And when a language rule depends on a
spelling (case, parens), ask every printer of that syntax — fmt, repairs, docs generators — whether it preserves it.
Related: [[two-engines-agreeing]] (the same session's name-resolution class), [[a-gate-runs-on-every-caller]].
HANDOFF §11.5 holds it.
