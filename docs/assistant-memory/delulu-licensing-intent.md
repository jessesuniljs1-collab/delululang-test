---
name: delulu-licensing-intent
description: "Jesse's licensing/creator/trademark intent for DeluluLang (given 2026-07-24) and the recommended established-practice strategy — LICENSING IS OWNER-RESERVED, never auto-decide it"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-24T17:19:51.211Z
---

**RESOLVED 2026-07-24 (owner-approved): Apache-2.0 + trademark policy, derivatives rename.** Jesse
chose Apache-2.0 for the code and reading A (a derivative language must use a DIFFERENT name).
Shipped as ruling **D27** (commit closing campaign C9): `LICENSE` (verbatim Apache-2.0, © Jesse
Sunil), `NOTICE` (sticky attribution via §4(d)), `TRADEMARK.md` (different-name rule),
`GOVERNANCE.md` (Jesse = lead), `license`/`authors` on all 12 crates, SBOM component licence added.
The record below is kept because the *reasoning* stays useful; the decision itself is now made.

Standing rule still in force for ANY FUTURE licensing change: never auto-decide it — licensing,
philosophy, governance, public spec, and backward compatibility always need explicit owner approval.

## Jesse's stated intent (2026-07-24), preserved

- **Completely open source, no restrictions.** Anyone — humans, companies, governments, AI, robots —
  may use, modify, distribute, and **sell** software written in DeluluLang, build businesses and
  derivatives on it. "Completely free to use and sell in any way by any users."
- **Jesse Sunil is the permanent original creator.** All copyright in DeluluLang is owned by Jesse
  Sunil. Nobody may claim to be the original creator, and the creator attribution may not be changed
  or removed.
- **The name "DeluluLang" is protected** from misleading misuse. Nobody may falsely claim authorship
  of the original language, and no derivative may claim to be *the original* DeluluLang version.
- **Attribution must ride with every version and build.** Jesse wants the creator notice included
  with every version and build.
- Others may add their own creator/editor names for their contributions, but not as authorship of
  the original language.

## The one ambiguity to resolve with Jesse (do NOT guess — it changes the trademark policy)

One sentence in the brief is genuinely two-way: *"if they are using any code or concept from
DeluluLang they have to change the language they are building name to DeluluLang…"*

- **Reading A (rebrand — standard, and consistent with the rest):** a derivative must use a
  **different** name, so it cannot masquerade as the official DeluluLang (the Rust→"fork it, rename
  it" model). Everything else Jesse wrote — "name protected from misleading misuse", "cannot claim
  it as original DeluluLang version" — points here.
- **Reading B (keep-the-name):** a derivative must **keep** the DeluluLang name + attribution so the
  lineage is never hidden. Makes name-protection incoherent (every fork would be "DeluluLang"), so
  almost certainly not intended — but it is Jesse's call, not mine.

## Recommended strategy (established practice only — no invented legal text)

The goals split cleanly across two legal tools, which is exactly how Rust, Python, and Mozilla do it:

1. **Code licence — Apache-2.0** (recommended over MIT). Permissive: use/modify/sell/distribute all
   allowed. Its **§4 requires retaining copyright and the NOTICE file** through redistribution, which
   is the *established mechanism* that makes "attribution rides with every build" legally sticky —
   MIT cannot force that. Its **explicit patent grant** matters for a language aimed at
   robotics/autonomous systems. (Dual **MIT OR Apache-2.0**, the Rust-ecosystem default, is the
   alternative; plain MIT is simplest but has no patent grant and weaker attribution stickiness.)
2. **NOTICE file** — "Copyright 2026 Jesse Sunil, original creator of DeluluLang." Apache §4(d) makes
   downstream keep it.
3. **TRADEMARK.md** — "DeluluLang" is Jesse Sunil's mark. Truthful reference is fine ("built with
   DeluluLang", "DeluluLang-compatible"); you may **not** name a fork "DeluluLang" or a confusingly
   similar name, claim to be the official/original version, or imply endorsement. This protects the
   name **without** restricting the code — the correct separation.
4. **GOVERNANCE.md** — names Jesse as project lead, for the "global-adoption governance" goal.

Apache-2.0 text is verbatim standard (not invented). The TRADEMARK/GOVERNANCE text is adapted from
established models (Rust) and should be shown to Jesse before committing, per his "present
recommendations before irreversible licensing decisions" rule.

This discharges campaign finding **C9** (no LICENSE → default copyright → nobody may legally use it),
the single hardest blocker to adoption. See [[delulu-hardening-campaign]].
