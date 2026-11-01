---
name: save-agent-work-to-storage-2026-10-24
description: Jesse 2026-10-24 — save everything agents do to durable storage as they go (usage limits hit faster with agents); the dogfood saver task and where things land
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-23T19:36:59.731Z
---

Jesse, 2026-10-24: "Head chef, save everything agents are doing to storage. My usage limits will hit faster when running agents. I don't want anything to be lost".

**Why:** a usage limit can stop the session and its agents mid-task; work held only in a temp dir or an agent's context is lost.

**How to apply:** before (or right after) launching agents: their workdirs, briefs and a STATUS file go to durable storage on a timer that runs OUTSIDE Claude. 2026-10-24 setup: `D:\nelan\dogfood-campaign\save.sh`, Task Scheduler `DeluluLang-dogfood-save` (conhost --headless, every 5 min, 24 h): robocopy mirror to `D:\nelan\dogfood-campaign\mirror\<day>\` (/XJ — agents make junctions); agent transcripts (task .output files listed in `agents.txt`) to `D:\nelan\DeluluLang-agent-transcripts\<day>-dogfood\` LOCAL ONLY (reasoning never to the repo); a filtered snapshot (no binaries, <5 MB, no keys/tokens/owner code/test secret, not the banned word) committed in its own repo `D:\nelan\dogfood-campaign\repo` and pushed to `autosave/dogfood` on the test repo (no CI: ci.yml runs on master/main only; never merge it). Tell each agent to write findings into REPORT.md the moment they are confirmed. Write per-agent briefs to files — a brief only in context is lost with the session. Related: [[agents-hold-no-authority-2026-10-24]], [[autosave-task-2026-10-17]].
