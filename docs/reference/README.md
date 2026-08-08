<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from the chapter list and a live coverage run.
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# The DeluluLang language reference

This reference is **generated in part**: the grammar, tokens, primitive table, diagnostics registry, audit rules, CLI contracts and normative rule statements are all extracted from the compiler source by `delulu-conform --reference`. A change to the implementation that would stale a chapter fails `--check-reference` in CI instead of quietly making the docs wrong.

Every normative statement carries a stable anchor id (`ref.rule.*`, `ref.diag.*`, `ref.prim.*`, `ref.grammar.*`, `ref.audit.*`, `ref.cli.*`), and each anchor's conformance status is reported next to it — invariant 42: *a behavior not covered by a test is not stable, and the reference says so per item.*

**Coverage today: 327 of 327 anchors (100.0%).** See [coverage.md](coverage.md).

## Semantics (Constitution §5)

- [§5.1 Authority model — pure object-capability](semantics-5-1.md) — 4 rule(s)
- [§5.2 Effect system — effect rows in every function type](semantics-5-2.md) — 6 rule(s)
- [§5.3 Kind vs. scope — the honest granularity split](semantics-5-3.md) — 2 rule(s)
- [§5.4 Secrets and information flow](semantics-5-4.md) — 5 rule(s)
- [§5.5 Modules — units of authority](semantics-5-5.md) — 3 rule(s)
- [§5.6 Packages — declared, versioned authority](semantics-5-6.md) — 5 rule(s)
- [§5.7 Imports — access is not authority](semantics-5-7.md) — 2 rule(s)
- [§5.8 Errors — results, not exceptions](semantics-5-8.md) — 3 rule(s)
- [§5.9 Concurrency — actors with reference capabilities](semantics-5-9.md) — 4 rule(s)
- [§5.10 Async — an effect, not a second system](semantics-5-10.md) — 1 rule(s)
- [§5.11 Compilation — high-level surface, tiered backend](semantics-5-11.md) — 2 rule(s)
- [§5.12 Interop — first-class Python and C, honestly fenced](semantics-5-12.md) — 4 rule(s)
- [§5.13 Portability — one portable target](semantics-5-13.md) — 2 rule(s)
- [§5.14 Security — defense in depth](semantics-5-14.md) — 5 rule(s)
- [§5.15 Runtime guarantees](semantics-5-15.md) — 3 rule(s)
- [§5.16 The authority holder model](semantics-5-16.md) — 3 rule(s)

## Extracted from the implementation

- [Tokens](tokens.md)
- [Grammar productions](grammar.md)
- [The primitive table](primitives.md)
- [Diagnostics](diagnostics.md)
- [The audit rules](audit-rules.md)
- [CLI contracts](cli.md)
- [Conformance coverage](coverage.md)

## Regenerating

```
cargo run -p delulu-conform -- --reference        # rewrite these chapters
cargo run -p delulu-conform -- --check-reference  # fail if they are stale
```
