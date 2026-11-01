---
name: call-depth-is-not-data-depth
description: "DATA-DEPTH-1 (2026-10-22, RW 4.74) — MAX_DEPTH bounds program recursion, not value depth; a loop builds a value as deep as memory allows, so every walk over a Value (display, eq, WASM reify, descriptor parse, actor copy, channel wire) must be iterative; INTERP-DROP-1 fixed only Drop"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-21T19:53:21.987Z
---

`str` of a linked list built in a `while` loop killed the interpreter at 300,000 links (stack overflow on `delulu-main`, inside the default 1 GiB budget) and the WASM host at 100,000 (`reify.rs` recursion) — a guest's data crashing its host (Contained plugins included); a hand-written module did the same with a million-deep type descriptor. Drop had been made iterative years before (INTERP-DROP-1) and nobody listed the other walks. Fixed with worklists in `Value::display`/`Value::eq`/`reify.rs` and a 256 bound on descriptor nesting; `actors.rs` message copy was then witnessed (300,000 links sent to an actor, exit 127) and made worklists too (RW 4.75, closed 2026-10-22); `channel.rs` WireValue still recurses, but only fixed shallow types cross it and a hostile frame hits ciborium's 256 recursion limit. Found while checking reify's depth bound before widening WASM nested values — the planned feature again surfaced the defect.

**Why:** a fix scoped to the walk that crashed leaves its siblings; and a test that locks cycles does not lock sharing (M777/M785 survived until a cell held twice side by side was tested).

**How to apply:** when one walk over a recursive structure is fixed for depth, grep every other walk over it; test depth on a 2 MB test thread (the binary dying is the witness); test both "held inside itself" and "held twice side by side". Related: [[walk-catch-all-swallows]], [[a-latency-may-be-a-deadline]], [[discovery-over-checklist]].
