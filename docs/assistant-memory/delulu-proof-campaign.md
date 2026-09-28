---
name: delulu-proof-campaign
description: "P17 — the campaign that moved DeluluLang from tested to (partly) proven: seven proof-boundary categories, what each now contains, and the findings that must not be re-softened"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-09-14T15:15:58.528Z
---

**Jesse's commission, 2026-08-03→04: "PROVE DELULULANG." Every subsystem must land in exactly ONE
of seven categories — proven / machine-checked / model-checked / property-tested / differentially
verified / fuzz-verified / explicitly outside the boundary. No grey areas.** Ledger:
`docs/design/PROOF_CAMPAIGN.md` (15 §§). Capstone: **`docs/MATHEMATICS.md`** — every structure, where,
why, and how strong. Complete; chain stopped, no cron armed.

**Toolchain now real and smoke-tested before use:** Z3 5.0.0 (pip), TLA+/TLC v1.7.4 + Java 21,
**Lean 4.32.2 via elan**, cargo-fuzz/cargo-deny/Miri. `cargo-audit` will NOT build — use
`cargo deny check advisories`.

**What each category contains now** (was: only tests):
- **Machine-checked (was EMPTY):** `docs/design/models/lean/DeluluCore.lean` — Effect Soundness for
  the **higher-order fragment**, AND `bad_unsound` proving the calculus *as written* admits a trace
  escaping its row (**C88 mechanized**). `#print axioms` → *"does not depend on any axioms"*. Full
  type system still unmechanized.
- **Model-checked:** `Broker.tla` 585,771 states + `Custody.tla` 2,421 states, with **three teeth
  tests that reconstruct three real historical bugs**. CI inverts them: a TLC *success* fails the build.
- **Proven:** `authority_algebra.py` — 17 Z3 obligations over all nine dimensions.
- **Property-tested:** `delulu-fuzz/src/danger.rs`, 250k programs.

**Findings that must never be re-softened:**
- **IF-1 (fixed):** `Secret.map`(plaintext, purity-gated) + `verify`(untainted Bool) + no pc-label =
  total declassification oracle. Fix = `verify` carries `Declassify` (R-2b). **Buys visibility, NOT
  impossibility.** Residue open: `verify` needs no `Cap[Declassify]`.
- **F1/F2/F3:** `⊑` is a **preorder** not a partial order; `⊓` not symmetric; ⊑-equivalent
  authorities hash differently. **Z3 localised it: the algebra is fine — it's the path ENCODING**
  (`./data` vs `data`), so the fix is canonicalization.
- **F4:** no principal types — swapping two parameters decides compilation.
- **P17-T1 (changes any future Lean work):** DELULU_CORE has ZERO occurrences of
  higher-order/callback/invoke/map — mechanizing §1–§7 as written would prove the WRONG theorem.
- **P17-T2:** Progress is FALSE as stated (scope violations are stuck; impl faults DL0904).
- **P17-C1:** audit chain detects modification/reordering, **NOT truncation**; nothing anchors the head.
- **P17-B2:** expiry uses a **wall clock** — a backwards step resurrects expired authority. Routine
  on satellites/aircraft (GNSS/NTP/RTC).
- **Not defects, verified:** capabilities crossing actor boundaries is SOUND (you can only send what
  you hold; unforgeable); zero `unsafe` in actors.rs; no deserialize vector (the grant tree never
  persists).

**Standing rules this campaign added:**
- **A gate that cannot fail is not a gate.** Two sweep cases passed `true`; a Z3 obligation was
  `Implies(False, True)` and printed PROVED. Both deleted, reasons recorded.
- **The grammar IS the coverage** — the fuzzer couldn't *write* either hole it hunted.
- `scripts/cli-sweep.sh` (22 cases) replaced a by-hand procedure. Run on Win+Linux every phase.
- **Regenerate the Survey LAST, after all doc edits** — the staleness race bit four times.
- **Miri: RAN TWICE, NEVER FINISHED — not a pass.** Do not upgrade without a `test result:` line.
- CI has every gate but **had never executed** (repo never pushed) until the 2026-09-14 private
  push activated it ([[delulu-github-remote]]); five runs are now recorded (CROSS_PLATFORM_VERIFICATION §9), and run 5
  (2026-09-14) was green on every job — all three OSes in one run. "Prepared" ≠ "triggered" ≠ "green".

Twice the project's own tests caught *me* being wrong (the `publish=false` posture; `delulu test .`).
Both recorded rather than deleted. See [[delulu-hardening-campaign]], [[delulu-survey-map]],
[[delulu-core-regression-rule]], [[delulu-continuation-protocol]].
