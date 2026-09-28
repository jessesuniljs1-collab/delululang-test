---
name: discovery-over-checklist
description: "Jesse's standing red-team directive — a green checklist is NOT proof of security; use the campaign as a discovery process to find what the checklist doesn't know to ask"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-08T10:28:56.617Z
---

Jesse's ruling (2026-08-08, mid-campaign): **"Do not declare the security campaign complete merely
because the known checklist is green. The goal is not to make the checklist green — the goal is to
discover what the checklist does not know to ask."**

**Why:** the P20 custody campaign proved checklist-driven verification is insufficient — of four agent
reports: one "CRITICAL" was a false positive (Agent 4, sealed bypass), one "BROKE" was already fixed
(Agent 1 tested a stale binary), one was a real operational footgun (F-CUSTODY-2), and the one genuine
security hole (P20-R4, revocation by chain extension) was **not on any checklist** and was found by
the operator asking "could this mechanism be defeated?", not by running the list. A green suite told
us nothing about the hole it couldn't state. Echoes [[skip-branch-verification-rule]] and the P17
lesson "a gate that cannot fail is not a gate."

**How to apply:** after verifying and CLOSING the known findings, do NOT stop. Continuously ask *"what
security assumption have we not attacked yet?"* and proactively hunt: untested invariants, trust
boundaries, authority transitions (mint/adopt/revoke/expire/unseal), failure modes (poisoned store,
partition, restart, clock skew, concurrent writers), and cross-feature interactions. Re-verify every
agent claim independently against the CURRENT binary (never trust a report — false positives and stale
binaries both happened). Treat the checklist as a floor to extend, never a finish line. Part of the
[[delulu-hardening-campaign]] / [[delulu-proof-campaign]] methodology.
