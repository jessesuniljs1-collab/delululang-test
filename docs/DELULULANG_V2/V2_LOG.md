# V2 log

The running log (owner, 2026-09-18, D-V2-22): one short block per phase, newest last. Details live
in the commits and in project storage (`D:\nelan\DeluluLang-agent-transcripts\<date>-<phase>\`).
Phase state: [`V2_PHASE_STATUS.md`](V2_PHASE_STATUS.md). Decisions: [`V2_DECISION_LOG.md`](V2_DECISION_LOG.md).
The long logs, [`V2_EXECUTION_LOG.md`](V2_EXECUTION_LOG.md) and [`V2_AGENT_LOG.md`](V2_AGENT_LOG.md),
are frozen at P1.

## V2-0 — 2026-09-17 — complete
- Commit `e48f9c3` (CI `35258166713` green), record `94c6e29` (CI `35259510471` green).
- 32 historical documents moved to `docs/archive/v1/`; the V2 folder; the Survey's archive-mirror rule.

## P1 machine-contract truth — 2026-09-18 — complete
- Commit `d8dbc24` (CI `35299533343` green on 3 OSes), record `a93cfd8`. Opus 5 sous-chef; head chef verified.
- One JSON envelope on every reachable success; 145 → 1 diagnostics; `authority --grants`; editless
  repairs need a human; `test --json` diagnostics; bare `delulu test`; guide paths; accounting gate.
- Suite 1,674/0 on 126 binaries; snapshot 58 cases, all attributable. Rulings D-V2-19 to D-V2-21.
- Left open: P1-F1 to P1-F4 (roadmap, P1), 6.13, P1-11 (D-V2-17).
