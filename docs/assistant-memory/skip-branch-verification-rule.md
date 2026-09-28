---
name: skip-branch-verification-rule
description: "DeluluLang kitchen rule — security rules die in the skip branch; write the 'what if the checker couldn't tell' case before claiming a rule holds"
metadata:
  type: feedback
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
---

Established 2026-07-15 during Stage 6 (Live), at real cost: R-6a/DL0803 (no callbacks into
Contained plugins) was **fail-open**. The rule read correctly, every test the sous-chef wrote
passed, and two paths still escaped with zero diagnostics — an unpinned `p.get`, and generic
laundering (a generic's type vars are instantiated fresh per call site, so a helper's `T` is never
unified with the caller's closure; `F` reads `fn('t0) -> Str` and the fn-detector's `_ => false`
arm waves it through). Fixed by DL1509 + `type_contains_var` (recurses into Fn params/ret).

**Why:** an adversarial read of the *skip* branch (`else { continue }`) is the only thing that
found it. Rules are written for the case where the checker CAN tell; the hole lives in the case
where it can't, which nobody writes a test for. Fail-closed is DeluluLang's whole posture — a
security rule must never be silently skippable because inference was underdetermined.

**How to apply:** for every rule, write the "what if the checker couldn't tell" case as a test
BEFORE claiming the rule holds; make each escape a permanent named witness plus a scope guard
proving the fence doesn't over-refuse. When a skipped check and a violated check are different
faults, give them different codes — reusing one code makes the message a lie (DL0803 says "you
passed a function"; at a laundering site no function is visible, hence DL1509).

Corollary learned the same day: when the head chef proposes a fix, the sous-chef correcting it
with a mechanism argument is worth more than clean execution of a wrong rule — Fable 5's proposed
"refuse unless F is a concrete Fn" would have closed the harmless case and left the dangerous one
open, because in the laundering case `F` *is* a concrete Fn. Extends [[delululang-project]]'s
head-chef lesson (verify against the real binary, not close-out tables).
