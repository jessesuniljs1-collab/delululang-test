---
name: pinned-refusal-names-its-reason
description: "POP-WALK-1 (2026-10-21, D-V2-171) — a test that pins a refusal often writes WHY beside it; read that before making the refused thing compile (list_parity's `pop` pin named the walk/slot-reuse divergence)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-21T17:19:28.933Z
---

`list_parity.rs` pinned WASM `pop` as refused, with its reason beside it: a `pop` and then a `push` reuse a slot a walk could still read. I implemented `pop` first and only the pin's red told me. Read rather than deleted, it was a real divergence: a `for` popping and pushing printed `1239` on WASM and `1234` on the interpreter, and `map`, `filter`, `fold` and `find` diverged likewise. Fix: in a module that pops a list (`Imports::pops_lists`), each walk that runs code per element walks a private copy of the slots (M751, M752 red).

**Why:** the author of a refusal usually knew the hazard; deleting the case throws that knowledge away.

**How to apply:** before widening anything a test refuses, grep for the test and read its comment; when the pin goes red, replace the case with what still stays outside — never just drop it. Related: [[gate-records-before-the-check]], [[discovery-over-checklist]].
