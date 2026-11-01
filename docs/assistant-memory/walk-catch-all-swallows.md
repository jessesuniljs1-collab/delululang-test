---
name: walk-catch-all-swallows
description: "MATCH-NEST-1 / MATCH-EXHAUST-1 (2026-10-22, D-V2-172) — a recursive walk's `_ => {}` silently drops the forms it never names; a guarantee stated in a message must be backed at every depth; a bounded search that runs out must refuse, never pass"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-21T19:23:43.995Z
---

The WASM `compile_arm` bound a constructor's field only when its pattern was a name; `_ => {}` passed over a nested constructor or literal, so `Some(Blue)` took `Some(Red) => …` (interpreter 2/7, WASM 1/10) — silently, in `run --engine wasm`, `.dwx` builds and Contained plugins. The checker had the same blind spot: it proved only the top-level constructors, while the interpreter's DL0907 text says "checker guarantees exhaustiveness". Fixed by an ordered arm chain on WASM and a Maranget search in the checker. The first search version, on running out of budget, PASSED a 400-arm match that was in fact non-exhaustive — now a spent search refuses (DL0407 in words). Found by reading code while planning a different feature (RW 3.4 nested lists).

**Why:** a catch-all in a structural walk hides every case nobody wrote down; tests only cover the cases someone thought of.

**How to apply:** when reading any walk over patterns/types/expressions, list what its `_ => {}` / `_ => return` drops and ask whether dropping is the meaning or an accident; when a message promises a guarantee, find the code that makes it and check every depth; a bounded search defaults to the sound answer when it runs out. Also: a hand-kept list beside the thing it summarizes drifts (WASM-IMPORTS-1: the import-section flag list missed `needs_maps`). Related: [[skip-branch-verification-rule]], [[discovery-over-checklist]], [[pinned-refusal-names-its-reason]].

**Again, worse (GC-MAP-1, 2026-10-22, RW 4.77):** the cycle collector's marking `match` on `Value` ended `_ => {}`; written before `Map` existed, so `Map` fell into it and a sweep EMPTIED a live list held in an actor's map (`{k: []}` for `{k: [1, 2, 3]}`) — silent data corruption. Fix + rule: make walks over a growing enum EXHAUSTIVE (name every variant), so adding a variant is a compile error at each walk.
