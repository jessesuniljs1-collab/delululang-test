---
name: autosave-snapshot-is-a-draft
description: "2026-10-18: a session that recovers work from origin/autosave/laptop verifies it as a stranger's — re-run its witnesses, read it adversarially, rewrite its records as its own, check older snapshots landed"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-18T20:30:00.000Z
---

`origin/autosave/laptop` (`scripts/autosave.ps1`, every 10 minutes while the Task Scheduler job runs) holds whatever a
laptop session had not committed. The on-demand routine's first run (2026-10-18) found snapshot `b4a9e18` (base
`640a7d5`) holding the laptop's RW 6.1 with its records already written ("Commits: (1) this entry's commit").

**Why:** the claimed witnesses and mutant re-ran as claimed, and an adversarial read still found two defects — a
catalog's line break forging a line of the `authority` report (`terminal_safe` where a one-line slot needs
`terminal_line`) and a doc comment carried onto the wrong test.

**How to apply:** `git diff BASE SNAPSHOT | git apply`, then: re-run every witness the records claim (red on the old
code, green on the new), read the diff as a stranger's, rewrite its records as the recovering session's (name the
snapshot hash), and for each older snapshot check `git diff SNAPSHOT NEXT_COMMIT -- PATHS` is empty. Never merge the
autosave branch. `docs/CLOUD_ROUTINE.md` step 1; `HANDOFF.md` §11.5.
