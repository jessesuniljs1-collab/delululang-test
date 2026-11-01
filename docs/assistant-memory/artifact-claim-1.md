---
name: artifact-claim-1
description: "2026-10-21: a .dwx's code_blake3 excluded its delulu:authority section, so the declared effects could be edited in place and the artifact still said 'authority verified'; fixed by ENFORCING the claim at run time (DL1202 at the WASM host's custody gate), not by hashing it"
metadata:
  type: feedback
---

For every integrity check, list what the hashed bytes EXCLUDE, and ask of each excluded field whether anything downstream
trusts it.

**Why:** ARTIFACT-CLAIM-1 (RW 4.73, found by the laptop 2026-10-21 while writing an `--assert-trace` witness): the `.dwx`
hash covered the module minus its authority section, so editing `"Write"` → `"Clock"` (same length) left a valid artifact
that announced "authority verified; declared effects: Read, Clock" and wrote anyway. Grants still bounded it (no authority
escaped), but the review surface (`delulu authority x.dwx`, the run banner, the book's "cannot drift apart") was false.
A signature would not have fixed it (it says who, not whether the claim is true).

**How to apply:** prefer enforcing a self-asserted claim where it is used (here `HostConfig::declared_effects`, checked in
`custody_allows` via `Op::required_effect`) over binding it cryptographically. Related: [[handoff-11-4-2026-10-20]]
(ROOT-RESOLVE-1: which side of a check is fixed), [[discovery-over-checklist]]. Also found with it: WASM-CODE-1 — a
code-label list of nine codes called every other refusal DL0904; label by the message's first registered code.
