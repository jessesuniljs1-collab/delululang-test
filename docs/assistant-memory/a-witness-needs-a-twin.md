---
name: a-witness-needs-a-twin
description: "RW 4.98, 2026-11-05 — my first witness (beat 0.7 ms into a 2 s period, checked at 2.8 s) could never go green, since a correct daemon fires there too; give every red witness a twin, the same shape without the defect, green today"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-05T16:54:39.339Z
---

A red witness must be able to pass. Give it a **twin**: the same shape with the defect's trigger removed, which must be
green today. Then the defect is the only difference between them.

**Why:** RW 4.98's first witness (`f3a2b2a`) held a beat stamped 0.7 ms into a 2 s dead-man period and checked at 2.8 s.
A beat that early extends the dead-man only to about 2 s, so a CORRECT daemon fires there too. The test was red for its
timing, not only for the defect, and no fix could ever turn it green. I caught it only while designing the fix, before
#131 landed (`126fc4e`: beat at ~1.5 s, check at 2.4 s, twin with the beat handed over at once = green 6/6). The mirror
of "a gate that cannot fail is not a gate" ([[delulu-proof-campaign]]): **a witness that cannot pass is not a witness.**

**How to apply:**
- Before landing a red/`#[ignore]`d witness, write the twin (shared helper, one flag for the trigger) and run it green.
- Work through the correct semantics on paper: what does a right implementation do at the instant the test checks?
- With a fix, the witness goes green, and mutants of the fix turn it red again.
Related: [[rw-7-25-deadman-reader-stall]], [[delulu-reproduce-the-shape]].
