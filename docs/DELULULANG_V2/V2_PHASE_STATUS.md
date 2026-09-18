# V2 phase status

One row per phase, in the approved order. States: **not started** · **in progress** · **blocked
(awaiting owner)** · **complete** (its commit is pushed and its CI run was read). A phase is complete
only after the verification gate in `V2_IMPLEMENTATION_ROADMAP.md` §0. Tasks a phase could not finish
are named in its `V2_EXECUTION_LOG.md` entry, never dropped.

| # | Phase | State | Started | Completed | Commit | CI run | Notes |
|---|---|---|---|---|---|---|---|
| 0 | V2-0 workspace + documentation migration | complete | 2026-09-17 | 2026-09-17 | `e48f9c3` | `35258166713` — success | 32 files archived; the Survey's archive-mirror rule; the V2 folder created |
| 1 | P1 machine-contract truth | complete | 2026-09-18 | 2026-09-18 | `d8dbc24` | `35299533343` — success | 12 of 13 tasks, P1-11 deferred (D-V2-17); rulings D-V2-19 to D-V2-21; follow-ups P1-F1 to P1-F4, RW 6.13 and D-NE-17 closed by P1-F (V2_LOG); suite 1,674/0 on 126 binaries; snapshot regenerated (58 cases, all attributable) |
| 2 | PS-0 sandbox truth, probes, cheap hardenings | not started | — | — | — | — | starts only on the owner's word (2026-09-18); PS-0-02 re-specified by D-V2-21 |
| 3 | PS-A L1 process sandbox + effect channel + STRICT/AUDIT modes | not started | — | — | — | — | asks D-NE-24/26/31/33 at start |
| 4 | P2 real plugin loading | not started | — | — | — | — | asks D-NE-10 at start |
| 5 | P4a Agent Skill | not started | — | — | — | — | asks D-NE-5 at start |
| 6 | P3 standard library | not started | — | — | — | — | |
| 7 | PS-B limits as authority, egress proxy, identity, BREAK-GLASS | not started | — | — | — | — | asks D-NE-28 at start |
| 8 | P4b–e agent surfaces, MCP, checked edits, Atlas/Survey tooling, usability benchmark | not started | — | — | — | — | asks D-NE-6 at start |
| 9 | PS-C Linux/KVM microVM | not started | — | — | — | — | D-NE-23 ruled; asks D-NE-27 |
| 10 | P6 documentation consolidation (HANDOFF, Book, README) | not started | — | — | — | — | the archive half was done in V2-0 |
| 11 | P5 distribution | not started | — | — | — | — | asks D-NE-7/8 at start |
| 12 | P7 verification depth | not started | — | — | — | — | the entrenched restatement is the owner's |
| 13 | PS-D external launchers + attestation seam | not started | — | — | — | — | |
| 14 | P8 safe autonomy (signed adapter) | not started | — | — | — | — | owner-gated |

**Pending owner decisions with no phase yet blocked:** none. Decisions are asked at the start of the
phase that needs them (`V2_MASTER_PLAN.md` §7).
