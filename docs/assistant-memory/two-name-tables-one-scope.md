---
name: two-name-tables-one-scope
description: "2026-10-19: WASM-HYGIENE-2 — the WASM backend keeps locals and function values in two tables; a block opened a frame in one, so an inner `let f = fn…` replaced the outer `f` for the rest of the function; and the 50,000-program gate generated no function value at all"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-19T15:00:00.000Z
---

When names live in more than one table, every scope that opens a frame in one must open a frame in each — and a hygiene
fix for one table asks the question of the other.

**Why:** the WASM backend resolves locals through `Cx::scopes` and function values (lambdas it inlines) through
`Cx::callables`. `compile_block_inner` pushed a `scopes` frame per block and no `callables` frame, so a `let f = fn…` in an
`if` or `while` body went into the function's own frame and replaced the outer `f` until the function ended: `2 / 1` on
the interpreter, `2 / 2` on WASM (on `master` before RW 3.4 too, in the `if` form). WASM-HYGIENE-1, fixed the same day,
had asked which FRAMES an inlined body sees; nobody asked whether each kind of frame is opened by each block. Found by the
on-demand routine's fourth run while verifying the laptop's RW 3.4, by reading `compile_block_inner` against the two
`push` sites; fixed in one line pair, witnessed red first.

**And the gate could not have seen it:** the two-engine differential (50,000 programs, 0 mismatches) generated no function
value — a generator that never produces a construct says nothing about it. The loop generator now binds `h`, calls it in
the loop and the tail, and shadows it in the loop's body or an `if` there (149 of 400 programs in `loop_parity`'s sample);
its count is asserted, so a generator that stops shadowing is red (mutant M512).

**How to apply:** for a compiler or interpreter change, `grep` every table a lookup reads (`scopes`, `callables`, generics,
types) and check each scope-opening site pushes and pops every one; and before trusting a differential's zero, list the
constructs its generator can emit.
