# Branch cleanup — 2026-10-26

An operations record, in the discipline of [`DISK-CLEANUP-2026-10-17.md`](DISK-CLEANUP-2026-10-17.md) and
[`WORKTREE-CLEANUP-2026-10-21.md`](WORKTREE-CLEANUP-2026-10-21.md): what was removed, how each removal was checked
first, and what was kept. No commit was lost, and nothing in DeluluLang changed.

## Why

The owner, 2026-10-26: *"the repository has accumulated too many branches. Audit them and safely clean up stale,
redundant, and unnecessary branches while preserving valuable work and avoiding the loss of unmerged changes."*
`origin` held 91 branches. The last cleanup (2026-10-17, `docs/CLOUD_SYNC_LOG.md`) removed 23. Since then every
laptop pull request left its `laptop/…` branch behind, and every cloud session left its `claude/…` one.

## What was checked first

For every branch on `origin`, after `git fetch --prune`: `git merge-base --is-ancestor origin/BRANCH origin/master`.
If that failed, the same against `origin/launch/2.0.0`. If both failed, `git cherry origin/master BRANCH` and the
branch's own commits were read. A branch was removed only when its tip is in `master`'s or `launch/2.0.0`'s history,
or when its commits are reachable from the archive branch below. `master`, `launch/*`, `rc/*`, `archive/*` and
`autosave/*` were never candidates. The same check ran again immediately before the deletion.

## What was done

**81 branches removed from `origin`, each one already in a kept branch's history.** To restore one:
`git push origin <tip>:refs/heads/<name>`. Every tip below is a commit of `master`, except `launch-merge-1020`'s,
which is in `launch/2.0.0`.

| Branch | Tip | Last commit | Contained in |
|---|---|---|---|
| `claude/estop-observed-baseline` | `65a4ec7` | 2026-10-18 | `master` |
| `claude/rw-4-17-normalization` | `7d04370` | 2026-10-18 | `master` |
| `claude/monitor-chain-follower` | `71cae40` | 2026-10-19 | `master` |
| `claude/rw-4-57-run-node` | `49f6c6f` | 2026-10-19 | `master` |
| `claude/study-c-remeasure` | `81da0ae` | 2026-10-19 | `master` |
| `claude/wasm-hygiene-2` | `79f83d6` | 2026-10-19 | `master` |
| `claude/wasm-oom-1` | `8fc42f8` | 2026-10-19 | `master` |
| `laptop/rw-4-69` | `98edf2f` | 2026-10-19 | `master` |
| `laptop/vm-posture` | `a53f90a` | 2026-10-19 | `master` |
| `laptop/wasm-loops` | `4d11575` | 2026-10-19 | `master` |
| `laptop/wasm-short` | `9367b13` | 2026-10-19 | `master` |
| `claude/dwx-flags` | `ec4bc87` | 2026-10-20 | `master` |
| `claude/rw-7-20-egress-window` | `7355808` | 2026-10-20 | `master` |
| `claude/wasm-read-scope` | `669664c` | 2026-10-20 | `master` |
| `laptop/eq-fn` | `d18d86f` | 2026-10-20 | `master` |
| `laptop/msg-cycles` | `d63d13d` | 2026-10-20 | `master` |
| `laptop/value-cycles` | `825657a` | 2026-10-20 | `master` |
| `laptop/wasm-emptylist` | `4d200b6` | 2026-10-20 | `master` |
| `laptop/wasm-fields` | `a7b6476` | 2026-10-20 | `master` |
| `laptop/wasm-float` | `11a50bd` | 2026-10-20 | `master` |
| `laptop/wasm-fnref` | `a9ca534` | 2026-10-20 | `master` |
| `laptop/wasm-fswrite` | `275af08` | 2026-10-20 | `master` |
| `laptop/wasm-list` | `e85fcb5` | 2026-10-20 | `master` |
| `laptop/wasm-narrow` | `ce5f6ab` | 2026-10-20 | `master` |
| `laptop/wasm-parse` | `d51d8d3` | 2026-10-20 | `master` |
| `laptop/wasm-records` | `d259cf2` | 2026-10-20 | `master` |
| `laptop/wasm-show` | `cba20d4` | 2026-10-20 | `master` |
| `laptop/wasm-streq` | `46caa70` | 2026-10-20 | `master` |
| `laptop/wasm-strmethods` | `650387a` | 2026-10-20 | `master` |
| `laptop/dwx-actors` | `a2fb0cd` | 2026-10-21 | `master` |
| `laptop/dwx-actors-run` | `12d5fa2` | 2026-10-21 | `master` |
| `laptop/dwx-custody` | `0439453` | 2026-10-21 | `master` |
| `laptop/epoch-window` | `f05c329` | 2026-10-21 | `master` |
| `laptop/launcher-witness` | `7431dbf` | 2026-10-21 | `master` |
| `laptop/lease-remedy` | `5470b53` | 2026-10-21 | `master` |
| `laptop/net-path-record` | `a01a27a` | 2026-10-21 | `master` |
| `laptop/p9-05-packages` | `fa4021c` | 2026-10-21 | `master` |
| `laptop/p9-05-requests` | `eef9583` | 2026-10-21 | `master` |
| `laptop/records-1021` | `3af2b2a` | 2026-10-21 | `master` |
| `laptop/wasm-contains-compound` | `30885b1` | 2026-10-21 | `master` |
| `laptop/wasm-http` | `4729747` | 2026-10-21 | `master` |
| `laptop/wasm-list-methods` | `d42d216` | 2026-10-21 | `master` |
| `laptop/wasm-maps` | `8cfbc01` | 2026-10-21 | `master` |
| `launch-merge-1020` | `8ff2b17` | 2026-10-21 | `launch/2.0.0` |
| `laptop/autosave-push` | `e8b3be4` | 2026-10-22 | `master` |
| `laptop/broker-slow-answer` | `86b8df5` | 2026-10-22 | `master` |
| `laptop/broker-stop-waits` | `41e9724` | 2026-10-22 | `master` |
| `laptop/core-mechanized` | `b8c4fde` | 2026-10-22 | `master` |
| `laptop/deep-messages` | `ec333c4` | 2026-10-22 | `master` |
| `laptop/deep-values` | `f791ded` | 2026-10-22 | `master` |
| `laptop/gc-map` | `8a88466` | 2026-10-22 | `master` |
| `laptop/gc-map-cycles` | `35d8208` | 2026-10-22 | `master` |
| `laptop/match-nested` | `0061ca1` | 2026-10-22 | `master` |
| `laptop/miri-exhaust-budget` | `1f44f55` | 2026-10-22 | `master` |
| `laptop/rcaps-depth` | `671677d` | 2026-10-22 | `master` |
| `laptop/records-1022` | `37b3e4c` | 2026-10-22 | `master` |
| `laptop/unc-grant-warning` | `a47516c` | 2026-10-22 | `master` |
| `laptop/wasm-nested-values` | `8fb2867` | 2026-10-22 | `master` |
| `laptop/wasm-package-build` | `0344884` | 2026-10-22 | `master` |
| `laptop/wasm-secret-handle` | `4af166e` | 2026-10-22 | `master` |
| `laptop/channel-control-busy-runner` | `2349cff` | 2026-10-23 | `master` |
| `laptop/ctutils-0385` | `269237d` | 2026-10-23 | `master` |
| `laptop/deploy-plan-scopes` | `05909ef` | 2026-10-23 | `master` |
| `laptop/http-post` | `b40de37` | 2026-10-23 | `master` |
| `laptop/launch-records` | `3334f93` | 2026-10-23 | `master` |
| `laptop/monitor-witness-says-why` | `c153433` | 2026-10-23 | `master` |
| `laptop/net-methods` | `d58811f` | 2026-10-23 | `master` |
| `laptop/record-width` | `118cf51` | 2026-10-23 | `master` |
| `laptop/records-1023` | `30e714b` | 2026-10-23 | `master` |
| `laptop/secret-audience` | `85f6b5c` | 2026-10-23 | `master` |
| `laptop/send-rebind` | `ce25bc1` | 2026-10-23 | `master` |
| `laptop/send-url-let` | `57d93da` | 2026-10-23 | `master` |
| `laptop/type-size` | `d4c6e1a` | 2026-10-23 | `master` |
| `laptop/wasm-handles` | `36740a4` | 2026-10-23 | `master` |
| `laptop/wasm-post` | `41f1dc6` | 2026-10-23 | `master` |
| `laptop/cert-secret` | `bf922f6` | 2026-10-24 | `master` |
| `laptop/const-check` | `0f79ac4` | 2026-10-24 | `master` |
| `laptop/generic-rigid` | `85f5f7a` | 2026-10-24 | `master` |
| `laptop/wasm-rem` | `0549645` | 2026-10-24 | `master` |
| `laptop/miri-methods` | `2ddd15c` | 2026-10-25 | `master` |
| `laptop/within-file-use` | `daff3ec` | 2026-10-25 | `master` |

**Three branches with a commit `master` lacks were archived, then removed.** Each was recorded as superseded long
before. Their tips are parents of one commit with no files, `7d92a8a`, on the new branch
**`archive/superseded-2026-10-26`**. Its message names each one. Before the removal, `git merge-base --is-ancestor TIP
7d92a8a` held for all three. Restore one the same way. Never merge the archive branch.

| Branch | Tip | Last commit | What it was |
|---|---|---|---|
| `claude/wonderful-hamilton-bo8k53` | `7d6a22b` | 2026-09-29 | a PS-E-02 witness of routine run 3, landed in revised form (superseded since run 10) |
| `claude/friendly-thompson-vz6m23` | `d3f0d52` | 2026-09-29 | RW 4.32's first form, from routine run 6, landed in revised form (superseded since run 10) |
| `laptop/wasm-handles-in-slots` | `5373158` | 2026-10-23 | parked WIP, finished and folded into `laptop/wasm-handles` (V2_LOG, 2026-10-23), not merged itself |

## What was kept (8 branches)

| Branch | Tip | Why |
|---|---|---|
| `master` | `8c9ccc9` | the development line |
| `launch/2.0.0` | `f0cb42b` | the prepared 2.0.0 release (launch steps 4–10 wait for the owner's word) |
| `rc/1.0.0-drill` | `acc6f11` | DRILL-001's one unmerged commit, the 1.0.0 drill's record (`HANDOFF.md`, *What was pushed*) |
| `archive/superseded-2026-10-26` | `7d92a8a` | the three archived branches above |
| `autosave/laptop`, `autosave/laptop-dl-launch` | `cfebc39`, `ccb3241` | the laptop's autosave snapshots (`scripts/autosave.ps1`) — never merged, read at a session's start |
| `autosave/dogfood` | `ba3f0f2` | the 2026-10-24 agent campaign's workdirs and ledger (`docs/CLOUD_ROUTINE.md` cites its `VERIFIED.md`) |
| `claude/peaceful-carson-crfvfa` | `16470ed` | on-demand run 6's A10 fix (an empty list filled later with a sum or a record on WASM): committed, unrecorded, not landed when the weekly limit ended the run. The next item, not stale |

**Later the same day.** The A10 fix landed on `master` as `500de3b`, a cherry-pick with the same code (#72). The branch
was then removed. `16470ed` is kept reachable by a second archive commit, `bfb5b99`, a fast-forward of
`archive/superseded-2026-10-26` whose parents are `7d92a8a` and `16470ed`. All four tips are in its history, and run 6's
witness runs (`40191526037`, `40191528017`) still name a commit anyone can fetch. #71's and #72's own branches
(`laptop/routine-milestones`, `laptop/a10-wasm`) were removed once each had landed, under the rule above.

## On the laptop

Every local branch whose tip is in `master`'s history was deleted with `git branch -d`, which refuses a branch that is
not merged. `laptop/wasm-handles-in-slots` was deleted with `-D` after the archive held it. The worktree
`D:\nelan\dl-launch` was switched from `launch-merge-1020` to `launch/2.0.0`. It was clean, and 0 commits ahead of
its upstream. `launch-merge-1020` was then deleted locally. Local `launch/2.0.0` was fast-forwarded to `origin`'s. The
two stashes were left as they were.

## So it does not pile up again

`docs/CLOUD_ROUTINE.md`, *Branches*, and `HANDOFF.md` §11.1. A session deletes from `origin` each `claude/*` and
`laptop/*` branch whose tip is in `master`'s history once it has landed, its own included, with the same check
immediately before.
