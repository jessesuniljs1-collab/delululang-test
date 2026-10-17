---
name: session-start-github-sync
description: "Jesse 2026-10-17: at the START of every session, check the GitHub repo first — git fetch origin, read HEAD..origin/master and origin/claude/* branches, fast-forward, read CI of what came in — because the on-demand cloud routine may have pushed"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-17T16:41:42.112Z
---

At the start of every DeluluLang session, before the Survey/doctor and before any edit: `git fetch origin`,
`git log --oneline HEAD..origin/master`, list `origin/claude/*` branches ahead of master, fast-forward (`git pull --ff-only`),
and read the CI runs (`gh run list -R jessesuniljs1-collab/delululang-test`) of whatever came in. Also `git stash list` —
work set aside mid-task may be parked there.

**Why:** Jesse, 2026-10-17: *"save this to memory, So that u will check GitHub repo whenever u start session."* He runs the
on-demand cloud routine ([[cloud-routine-on-demand-2026-10-17]]) while away from the laptop; its runs push to `master`, so
the laptop's tree can be many commits behind without any local sign of it.

**How to apply:** first action of every session; written into the repo too (`AGENTS.md` *Read first* step 0,
`HANDOFF.md` §11.1) so cloud sessions do the same. Never force anything: fast-forward only; a divergence is investigated.
