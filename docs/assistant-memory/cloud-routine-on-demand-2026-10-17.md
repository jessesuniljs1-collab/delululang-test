---
name: cloud-routine-on-demand-2026-10-17
description: "The on-demand cloud routine trig_01HFG6usfzmDAPQigiDu9DLq (D-V2-146): created PAUSED 2026-10-17, Jesse switches it on when he can't use the laptop; settings, schedule, and what a routine run must not do"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-17T16:41:51.148Z
---

Created 2026-10-17 at Jesse's request: **`trig_01HFG6usfzmDAPQigiDu9DLq`**, "DeluluLang head chef — on demand (start by
hand)" — https://claude.ai/code/routines/trig_01HFG6usfzmDAPQigiDu9DLq. **Created `enabled: false`**; Jesse starts it.
Settings copied from the first routine (`trig_01GG41tCGxZXyt7bZUid8Vuu`, still disabled): Opus 5.5, repo
`delululang-test`, env `env_012Mxeiw6zvfyxPy9ZEEFf3p` (Default), tools Bash/Read/Write/Edit/Glob/Grep/Agent/WebFetch/
WebSearch, Claude-Docs connector, autofix on PR. **Push + e-mail notifications ON.** Cron `5 */5 * * *` UTC — Jesse asked
"every 5hr and 5 mins"; cron can't express that interval, so every 5 h at :05 (20:05→00:05 is 4 h).

Its prompt: follow `docs/CLOUD_ROUTINE.md` (the *Since 2026-10-17* paragraphs), mandate = the 2026-10-17 commission, inbox
= newest `docs/CLOUD_SYNC_LOG.md` *Open / next*; keeps Jesse's own lines verbatim (check CI first; finish what a limit cut
off; check last commits; record everything to .md; Survey + doctor; commit as jessesuniljs1-collab); a **Run now** text
(`routine-fire-payload`) = Jesse's direction for that run. **A routine run never launches**: no other repo, no tag, no
release, no licence change, no kernel image — the launch, WSL microVM images and Windows VS Code e2e are the laptop's.

Official docs read 2026-10-17 (code.claude.com/docs/en/routines): minimum interval 1 h; on/off switch pauses; usage at
claude.ai/settings/usage; I can't read that page from the CLI. Related: [[session-start-github-sync]],
[[cloud-period-2026-09-28]], [[release-program-progress]].
