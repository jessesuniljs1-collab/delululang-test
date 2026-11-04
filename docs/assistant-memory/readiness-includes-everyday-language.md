---
name: readiness-includes-everyday-language
description: "Jesse 2026-11-02 — \"ready\" must include writing a complete program as in Java/Python; the release checklist + REMAINING_WORK track gates/security, not everyday language features"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-02T13:11:10.542Z
---

Jesse, 2026-11-02, after I said "2.0 is ready to ship": *"Fundamental things that are needed in a programming
language are not in delululang. Still cannot write complete program like in java or python, Most keywords and their
equivalent are missing."*

**He was right.** A probe of 39 everyday Java/Python constructs on master `9c51b4e` (harness
`scratchpad/pc/lang_probe.py`): 14 ran (one of them, string interpolation, printed `{name}` literally), 25 refused — no methods on one's own types, no class/interface/trait
(`trait`/`impl`/`where` reserved, RW 2.9 "no typeclass design exists"), no `+=`, no bitwise ops, no `0..n`, no string
interpolation/indexing/find, no abs/min/max/sqrt/pow, no tuples, no Set, no sort_by, no comprehensions, no args/env/
exit/sleep. Deliberate by design (keep): no exceptions (Constitution §5.8, Result + `?`), no module-level `var`
(DL0305), effects need a granted capability, `module` header. A single file that imports gives a misleading DL0301
(run the package folder instead).

**Why:** CHECKLIST-2.0 and REMAINING_WORK measure quality, security and proof gates; neither has a "can a newcomer
write an ordinary program" section, so "ready" was judged on the wrong axis.

**How to apply:** never call a release ready on gates alone — also judge it against an everyday-program probe
(Java/Python equivalents). Keep a language-completeness track in REMAINING_WORK. Distinguish "deliberate by design"
from "simply not built" when explaining gaps. Related: [[release-program-progress]], [[discovery-over-checklist]].
