---
name: delulu-stale-worktree
description: "A detached agent worktree holds a second full copy of the DeluluLang repo at a 2026-07-18 commit; exclude it from any tree walk, and never delete it without asking Jesse"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-31T21:01:13.842Z
---

`D:\nelan\DeluluLang\.claude\worktrees\agent-a4541bfe777803515` is a **registered git worktree** on
branch `worktree-agent-a4541bfe777803515`, pinned at commit `51a378a` (Stage 7 phase 7h,
2026-07-18) — 96 commits behind `master` as of 2026-08-01. It is hidden from `git status` by
`.git/info/exclude`, which is why it goes unnoticed.

**Why:** it is a complete second copy of the repository at a *different revision*. Any tool that
walks the tree without excluding it silently doubles every file and mixes two revisions of the same
document into one result — counts, greps, and maps all come out wrong in a way that looks
plausible. Found while building [[delulu-survey-map]]; a `du -sh` on it timed out at two minutes,
which is the first clue it is not scratch.

**How to apply:**

- Exclude `.claude` from any repository walk, count, or index. The Survey does this by name and
  documents the reason; keep it that way.
- Watch for it in numbers: a "184 Rust files" that suddenly reads ~368 means something walked it.
- **This one was REMOVED 2026-08-01**, authorized by Jesse after it was reported. Two independent
  checks ran first (branch held 0 commits master lacked AND its tip was an ancestor of master;
  working copy had 0 uncommitted/untracked/stashed) so the deletion took a label, not history —
  `51a378a` is still reachable from master. Full record + recovery command:
  `docs/survey/REMOVALS.md`.
- **The `.claude` exclusion stays.** It was never a fix for one worktree; the next agent run can
  create another.
- **Never delete a worktree without asking.** Jesse's pattern here — authorize, but "verify twice
  before removing" and record what was removed and why — is the standing shape for destructive
  repository operations. New ones go in `docs/survey/REMOVALS.md`.

Related: [[delulu-survey-map]], [[subagent-cost-strategy]] (the agent runs that create these).
