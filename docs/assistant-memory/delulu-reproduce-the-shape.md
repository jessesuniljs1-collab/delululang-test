---
name: delulu-reproduce-the-shape
description: "reproducing a recorded defect means matching the witness's SHAPE, not its description — a near-miss repro reads exactly like a fix"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-23T16:50:48.644Z
---

When re-checking whether a recorded defect is still open, **reproduce the witness's shape, not its
prose description.** A near-miss reproduction returns clean and is indistinguishable from a fix.

**Why:** on 2026-08-23 I checked whether "type inference is exponential" (HARDENING_CAMPAIGN P16)
was still open. The record says *"28 lines of nested record literals (`Pair { l: a, r: a }`, 24
deep) exhaust memory: 223 MB at depth 18, 3.5 GB at 22, killed at 24."* My first repro used
`type Pair[T] { l: T, r: T }` — **one** type parameter — and checked clean in **120 ms at depth
60**. Everything about it matched the description. It was the wrong shape: with one parameter the
inferred type is `Pair[Pair[…Int]]`, which is *linear* in depth. The blowup needs **two**
parameters, so `Pair { l: a, r: a }` infers `Pair[T0, T0]` and the type doubles per level.

With `type Pair[L, R] { l: L, r: R }`: 78 ms @ 12 · 170 ms @ 16 · 539 ms @ 18 · 2.5 s @ 20 ·
**10.2 s @ 22**, from 29 lines of source. Genuinely exponential, genuinely still open.

**How to apply:** before writing "verified fixed", ask *what mechanism made the original defect
fire, and does my input actually trigger that mechanism?* Then confirm the repro produces the
**baseline** effect (a measurable curve, a crash, a wrong answer) before trusting its refusal —
this is the same rule as *"a witness that never ran is not a negative result"*
([[delulu-p22-containment-campaign]]) and *"a gate that cannot fail is not a gate"*
([[delulu-proof-campaign]]).

Two corroborating instances from the same pass, both in the other direction (docs stale, code
ahead): the parser bound *was* genuinely closed (tested at 50,000 deep → `DL0210`, exit 1), and all
six Pony rcaps *were* genuinely built despite a code comment calling three of them "unbuilt". Test
both directions; do not assume the docs err only one way.
