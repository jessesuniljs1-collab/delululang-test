---
name: a-scan-point-must-not-allocate
description: "2026-10-31: RW 4.67 — a sent secret's copy sat in hyper's freed buffer; the allocator zeroes every free while a send's bytes are in the process. A memory scan must wait where the program allocates nothing, look for a fragment, and read only resident pages"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-31T13:00:00.000Z
---

A secret's copy that code outside the runtime makes cannot be cleared by any call site — so the rule belongs to the
allocator — and a scan that looks for such a copy must wait where the program allocates nothing.

**Why:** routine run 25's red team found one copy of a sent secret in the run's heap after the send (RW 4.67). On-demand
routine run 12 read it again on `9553ab7` by scanning the run through `/proc`: hyper's serialized request (`…on:
hunter2-…`) in the arena of reqwest's runtime thread, on both engines. The fix (D-V2-196): while a
`delulu_runtime::scrub::Window` is open, the `delulu` binary's allocator zeroes every block it frees and moves a growing
block through that free. Its witness's first scan point was a second request, which allocates on the run's own thread
and so overwrote a copy freed there: mutant M968 (the held secret's own zeroing removed) survived it, and was red 3 of 3
once the program waited on standard input instead. A freed block's first 16 bytes are the allocator's (a partial copy),
and a whole-mapping scan of a debug build took 34 s for the interpreter's stack reservation.

**How to apply:** scan while the program blocks in a call that allocates nothing (`console.readline`); count the whole
secret and its last bytes; read only resident pages (`/proc/PID/pagemap`); keep two controls (the bytes really passed
through; the scan finds a string the program holds). Undo a mutant by an anchored edit, never `git stash drop`, and
`cargo build` before any by-hand run after a stashed reading.
