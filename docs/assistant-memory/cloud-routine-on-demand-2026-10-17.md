---
name: cloud-routine-on-demand-2026-10-17
description: "The on-demand cloud routine trig_01HFG6usfzmDAPQigiDu9DLq (D-V2-146): created PAUSED 2026-10-17, Jesse switches it on when he can't use the laptop; settings, schedule, and what a routine run must not do"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-17T16:41:51.148Z
---

Created 2026-10-17 at Jesse's request: **`trig_01HFG6usfzmDAPQigiDu9DLq`**, "Head Chef — on demand (start by hand)" (renamed at Jesse's request 2026-10-17; first named "DeluluLang head chef — …") — https://claude.ai/code/routines/trig_01HFG6usfzmDAPQigiDu9DLq. **Created `enabled: false`**; Jesse starts it.
Settings copied from the first routine (`trig_01GG41tCGxZXyt7bZUid8Vuu`, still disabled): Opus 5.5, repo
`delululang-test`, env `env_012Mxeiw6zvfyxPy9ZEEFf3p` (Default), tools Bash/Read/Write/Edit/Glob/Grep/Agent/WebFetch/
WebSearch, Claude-Docs connector, autofix on PR. **Push + e-mail notifications ON.** Cron **`5 */6 * * *` UTC since 2026-10-21** (05:35, 11:35, 17:35, 23:35 IST) — Jesse asked
"every 5hr and 5 mins"; cron can't express that interval. The first cron, `5 */5`, left a 4 h gap nightly (20:05→00:05 UTC);
on 2026-10-21 he saw it and chose every 6 hours (never under 5 h 05 min).

Its prompt: follow `docs/CLOUD_ROUTINE.md` (the *Since 2026-10-17* paragraphs), mandate = the 2026-10-17 commission, inbox
= newest `docs/CLOUD_SYNC_LOG.md` *Open / next*; keeps Jesse's own lines verbatim (check CI first; finish what a limit cut
off; check last commits; record everything to .md; Survey + doctor; commit as jessesuniljs1-collab); a **Run now** text
(`routine-fire-payload`) = Jesse's direction for that run. **A routine run never launches**: no other repo, no tag, no
release, no licence change, no kernel image — the launch, WSL microVM images and Windows VS Code e2e are the laptop's.

Official docs read 2026-10-17 (code.claude.com/docs/en/routines): minimum interval 1 h; on/off switch pauses; usage at
claude.ai/settings/usage; I can't read that page from the CLI. Related: [[session-start-github-sync]],
[[cloud-period-2026-09-28]], [[release-program-progress]].

Effort: the routine API stores no environment variables (CLAUDE_CODE_EFFORT_LEVEL tried, read back empty); xhigh comes from the committed `.claude/settings.json` (`effortLevel` + `modelSettings.claude-opus-5-5`). **2026-10-19: Jesse saw routine runs at MEDIUM** despite both keys → something passes effort explicitly (an explicit choice outranks settings). Fix: `.claude/settings.json` `env.CLAUDE_CODE_EFFORT_LEVEL = xhigh` (the env var outranks `--effort`, `/effort`, `modelSettings`, `effortLevel`; project settings may set it; it also pins sous-chefs at xhigh). The routine API now REFUSES `session_request.environment_variables` (400 "not supported on triggers"). Second layer only Jesse can add: the cloud environment's env vars in the claude.ai UI. Verify on the next run (the session header / `/effort status`). Jesse's env screenshots 2026-10-17: Default env = Limited network + package managers, no setup script, no env vars — fine (25 earlier runs used it); optional setup script to cache the Rust toolchain + cargo-deny was suggested to him (the env is shared by every routine using it).

**Interactive cloud sessions too (Jesse, 2026-10-17):** *"Not just routine or scheduled task runs, I will be using claude code cloud sessions also occasionally using test GitHub repo … I don't want delululang development and testing and addition of new features to be stopped just because of my lap not available and I am busy with other projects."* → an interactive cloud session with no narrower request follows docs/CLOUD_ROUTINE.md like a routine run (pushes its claude/ branch + one PR; the next session merges it after reading CI). Development never waits for the laptop. Written into CLAUDE.md, CLOUD_ROUTINE.md, HANDOFF §11.1. `.claude/settings.json` now has a SessionStart (startup) hook: `git fetch -q origin; git log --oneline -n 20 HEAD..origin/master; git branch -r --list origin/claude/*` (tested in bash + PowerShell). Old branches friendly-thompson-vz6m23 / wonderful-hamilton-bo8k53 are superseded witness branches (recorded since run 10) — not stranded work.
