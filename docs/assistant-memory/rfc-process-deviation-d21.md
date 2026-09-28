---
name: rfc-process-deviation-d21
description: "DeluluLang D21 shipped a core authority change without the required RFC comment period — open governance debt, do not let it be reframed as compliance"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-22T08:00:43.492Z
---

On 2026-07-22 the device scope dimension shipped (ruling **D21**, commit `dee3680`, RFC 0001 phase
F1) **without the RFC comment period it required.**

**Why:** `rfcs/README.md` requires an RFC with a **≥ 14-day** comment period for any change to
"effect/authority behaviour", and says the period "does not shrink because a release is near".
Adding a dimension to the `⊑` lattice is squarely that, and `STAGE10_AUTONOMY_ADDENDUM.md` §2.2
explicitly called D12e **RFC-gated**. RFC 0001 exists but is an **unsponsored draft** — per
`CONTRIBUTING.md` §4 an AI-authored RFC needs a named accountable human, and it may not name its
own sponsor — so no period ever began. The work shipped on Jesse's direct instruction, which is
authority over the repository but is *not* the same thing as the comment period.

**How to apply:** This is recorded as a deviation in build-order D21(g), the RFC's status line, and
the addendum — in three places, deliberately. **Never let it be restated as compliance** ("the
owner approved it" is not "the period ran"). What would make it right: a named human sponsors RFC
0001, the period runs, and **if the RFC is amended or rejected, F1 changes with it** — landing first
grandfathers nothing. Byte-compatibility was preserved on purpose to keep reversal cheap
(`Authority::to_json` omits `device` when empty, so device-less authorities hash exactly as before
in the audit chain).

**Update 2026-07-22 — partially, but only partially, resolved.** Jesse **accepted the sponsor role**,
so RFC 0001 is now properly sponsored and open (comment period 2026-07-22 → 2026-08-05). That fixes
the process going forward and is recorded in the RFC, `rfcs/README.md`, and D22.

It does **not** retroactively give F1 a comment period — D21(g) stands as history. And phases
**F2–F6 were built *during* the open period** rather than after it, which `rfcs/README.md` ("accepted
RFCs are implemented") does not sanction either. That is a *smaller* deviation than F1's (the RFC is
sponsored, open, and can still be argued with) but it is one, and D22's preamble names it.

So the honest summary if asked: **two deviations of different sizes, both recorded in three places,
neither reframed as compliance.** The standing commitment for every phase is the same — if the RFC
is amended or rejected at the close, the code changes with it; nothing is grandfathered by having
landed first, and byte-compatibility was preserved deliberately to keep reversal cheap.

If Jesse ever asks whether the project is process-clean, the honest answer names both.

Related: [[delulu-federation-scope]], [[delululang-project]], [[skip-branch-verification-rule]].
