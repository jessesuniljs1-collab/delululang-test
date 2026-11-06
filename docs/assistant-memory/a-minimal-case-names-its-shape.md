---
name: a-minimal-case-names-its-shape
description: "Cloud review 2026-11-05 — 'any higher-order fn fails on WASM' came from one non-generic witness; the generic fn apply[T,U] compiles; RW 3.25 names the exact shape. Try neighbouring spellings before a claim says 'any'"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-05T15:23:13.305Z
---

A minimal failing case names its shape, not its class.

**Why:** Claude's cloud review first wrote that *"any user-defined higher-order function"* fails on WASM, from one
witness `fn apply(f: fn(Int) -> Int, x: Int)` (DL1201: `codegen.rs` `is_compilable` asks a `wasm_ty` of each param;
fn types have none). Before it became a row, the generic `fn apply[T, U](f: fn(T) -> U, x: T) -> U` was tried — it
compiles and agrees (`named_fn_parity.rs` said so all along). RW 3.25 now names three exact shapes: `for i in
range(0, n)`, a NON-generic fn-typed parameter, and an `if` without `else` as a match-arm value (the `{ if … }` block
form compiles).

**How to apply:** before writing "any"/"all"/"every", try the neighbouring spellings (generic, wrapped in a block, a
lambda, a named fn) and record which fail. Sibling of [[delulu-reproduce-the-shape]] (a near-miss repro reads like a
fix); this is its mirror (one repro over-generalised reads like a class).
