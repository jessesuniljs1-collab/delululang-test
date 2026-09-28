---
name: delulu-continuation-protocol
description: "How the DeluluLang autonomous continuation timer must be armed — one-shot chain with a nonce, never a recurring cron"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-08-02T11:17:25.781Z
---

Jesse wants overnight autonomous phase execution AND the ability to interrupt it at any moment.
His rules, verbatim: **"DO NOT remove this feature"**, *"Latest owner instruction always overrides
older scheduled work"*, *"Never let an old timer win against a newer instruction."*

**Why the first design kept fighting him (he asked twice — 2026-08-02, "why task schedule still
like that"):** the continuation was a **recurring** `CronCreate` job. Three mechanics made that
lose against him every time:

1. A recurring job is a *standing order*. `Esc` cancels the in-flight turn; it does not touch the
   scheduler, which lives outside the turn. So interrupting guarantees the next tick still fires.
2. A cron fire injects a **user-role** message. Once enqueued it is indistinguishable from Jesse
   typing — nothing downstream can tell them apart by inspection alone.
3. Jobs are in-memory and die with the process, but a prompt the job **already enqueued** is in the
   session `.jsonl` and replays on `--resume`. So closing the terminal still resurrects the effect.

A "check for newer instructions first" step cannot fix this: it runs *inside* the fired turn, so
the wake-up has already happened and Jesse already sees work restarting.

**The protocol that replaces it — default is STOP, continuing requires a fresh token:**

- Arm with `CronCreate(recurring: false)` — one shot, then it deletes itself. The chain advances
  **only** if I reach the end of a cleanly completed phase and arm the next one. An interrupt
  breaks the chain by construction, with no state to check.
- Write a nonce to `C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\continuation.json`
  (sibling of `memory/`, deliberately not inside it) and put the same nonce in the prompt text.
  On firing: nonce mismatch → superseded, stop. Already `consumed` → replayed transcript, stop.
  Otherwise mark consumed, then work.
- Then still check for owner messages newer than `armed_at` before doing anything.

**How to apply:** never arm a recurring job for continuation work. One-shot + nonce, every time.
Re-arming is the *last* action of a finished phase, never the first action of a starting one.
See [[delulu-hardening-campaign.md]] for what the phases are.
