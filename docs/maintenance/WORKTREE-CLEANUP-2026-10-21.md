# Worktree cleanup — 2026-10-21

An operations record, in the discipline of [`DISK-CLEANUP-2026-10-17.md`](DISK-CLEANUP-2026-10-17.md): what was removed, how
each removal was checked first, and what was kept. Nothing in DeluluLang's behaviour changed, and nothing on GitHub.

## Why

The owner asked why two local branches did not match GitHub (`worktree-agent-ab8e3c649dff8d3e7`,
`worktree-agent-af964add0968902db`). They were the git worktrees Claude Code made on 2026-09-17 for two sous-chefs —
an Opus 5 red-team pass on the sandbox surfaces and a Sonnet 5 host-capability fact sheet — under `.claude/worktrees/`.
They had never been pushed, because they held no commit of their own. A standing rule kept every agent worktree until the
owner's word (the one removed on 2026-08-01 was removed on it). Asked, he answered *"Do that is right decision"*.

## What was checked first

| Check | `agent-ab8e3c649dff8d3e7` | `agent-af964add0968902db` |
|---|---|---|
| Branch tip | `9584011` (2026-09-17) | `9584011` (2026-09-17) |
| Tip an ancestor of `origin/master` (`git merge-base --is-ancestor`) | yes | yes |
| Uncommitted or untracked (`git status --porcelain`) | one untracked note | one untracked note |
| Stashes | the repository's one, shared by every worktree (`RW 6.1 laptop draft`, on `master`) | the same |
| Size on disk | 13 MB | 13 MB |

**The notes.** `RED-TEAM-SANDBOX-SURFACES-opus5.md` (42,510 bytes) is byte-identical to
`docs/archive/v1/NEXT_EVOLUTION_2026/agent-notes/RED-TEAM-SANDBOX-SURFACES-opus5.md`.
`HOST-CAPABILITY-FACTS-sonnet5.md` (37,327 bytes) is the sous-chef's original. The archived copy (37,803 bytes) adds the
head chef's annotation of 2026-09-17, which prefixed five upstream `docs/…` link texts with their project's name. Both
originals were copied before removal to `D:\nelan\worktree-notes-2026-09-17\`, outside the repository, so no `.md` was lost.

## What was done

`git worktree remove --force` for each, then `git branch -d` for each branch. Both were deleted as merged (`was 9584011`).
`git worktree list` now names only the checkout and `D:\nelan\dl-launch`, the launch branch's worktree.
