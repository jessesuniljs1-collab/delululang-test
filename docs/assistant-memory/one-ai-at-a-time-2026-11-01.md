---
name: one-ai-at-a-time-2026-11-01
description: "Jesse 2026-11-01: another AI may also work on DeluluLang, but only ONE AI at a time (never Claude and another AI together); docs/AI_ONBOARDING.md is the brief every AI reads first; hand over only through git"
metadata:
  node_type: memory
  type: user
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-01T06:27:14.069Z
---

Jesse, 2026-11-01: *"There maybe another AI also working on delululang, Update the ur memory and docs. Give me doc
containing everything another AI should know before that AI start working like imp things to know and things in ur
memory. I don't want everything to be mess. So I will be running either of u or another ai at time, Not both of u
together"*.

**Why:** a non-Claude AI (Codex, Gemini, …) has none of Claude's memory, and two agents editing at once would tangle
branches, records and counters (D-V2-nn, mutant M-numbers, RW rows).

**How to apply:**
- `docs/AI_ONBOARDING.md` is the brief for ANY AI: one-AI rule, start/end checklists, hard rules, the loop, commands,
  traps, this memory digested, the current state. Keep it true in the same commit as any change it describes.
- Hand-over is git only: every session ends pushed, with a CLOUD_SYNC_LOG entry whose *Open / next* is the next AI's
  inbox; durable lessons go to HANDOFF §11 + `docs/assistant-memory/` (refreshed from this directory 2026-11-01).
- Read work another AI left like a stranger's: re-run its witnesses, read its CI, verify before landing
  ([[autosave-snapshot-is-a-draft]], [[discovery-over-checklist]]).
- If a commit I did not make lands on origin while I work → stop editing, tell Jesse, re-sync.
- Branch prefixes: `laptop/` (Claude laptop), `claude/` (Claude cloud), `<ai-name>/` for another AI.
- The routine is an AI too: every 4 h but PAUSED; only Jesse resumes it ([[cloud-routine-on-demand-2026-10-17]]).
