---
name: model-attribution-honesty
description: "Before writing a Co-Authored-By line, check which model is ACTUALLY running — the head-chef persona is not the model; Stage 9 was mis-attributed"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-19T13:57:19.628Z
---

When writing a commit's `Co-Authored-By:` trailer, name the model that is **actually running this
session**, not the persona. `/model` can switch the underlying model mid-stage while the "head
chef Fable 5" framing continues unchanged — the persona and the model are different things.

**Why:** In Stage 9 (2026-07-19), the session was switched to **Opus 4.8** partway through phase
9a, but every one of the ten Stage-9 commits kept signing `Co-Authored-By: Claude Fable 5`. Jesse
caught it. Stage 9 was mostly Opus 4.8; the trailers are false for that span. Stage 8 handled the
identical switch correctly (8g–8h → Opus 4.8), so the fix is attention, not a new mechanism. A
false author line is the one kind of dishonesty this project treats as unforgivable.

**How to apply:** (1) When a `/model` command appears in the transcript, note the new model and use
it in every subsequent commit trailer until it changes again. (2) If unsure which model is running,
say so rather than guessing a flattering answer. (3) On discovering a past mis-attribution, correct
it as a visible ERRATUM in the docs/memory — do NOT rewrite git history to hide that the mistake
happened (destructive, and it erases the evidence of the correction). See [[delululang-project]]
ruling D21 and [[head-chef-handoff]].
