# Removals — destructive repository operations

A log of things deleted from this repository, what was verified before each deletion, and how to
get them back. Destructive operations get their own record because the one thing you cannot
reconstruct afterwards is *what the person doing it checked first*.

Format: what, why, what was verified, how to recover.

---

## 2026-08-01 — the detached agent worktree

**Removed**

| | |
|---|---|
| Worktree | `.claude/worktrees/agent-a4541bfe777803515` |
| Branch | `worktree-agent-a4541bfe777803515` |
| Commit at removal | `51a378ab267effb9043c90f91856d8c2fe6607b4` (`51a378a`) |
| That commit | *Stage 7 (Concurrent) phase 7h: WASM cooperative actors (subset-scoped)*, 2026-07-18 21:50:59 +0530 |
| Authorized by | Jesse, 2026-08-01, "u can remove the stale worktree but verify twice before removing it" |

```
git worktree remove .claude/worktrees/agent-a4541bfe777803515
git branch -D worktree-agent-a4541bfe777803515
```

**Why**

It was a registered git worktree holding a **complete second copy of the repository** at a revision
96 commits behind `master`, left over from an agent run. `.git/info/exclude` hid it from
`git status`, which is why it survived unnoticed for two weeks.

The cost was not disk space. Any tool that walks the tree without excluding it doubles every file
and silently mixes two revisions of the same document into one result — counts, searches and maps
all come out wrong in a way that looks entirely plausible. It was found while building the Survey
(`AUDIT.md` §2.4), whose walker excludes `.claude` by name for exactly this reason. That exclusion
stays regardless: the next agent run can create another one.

**Verified before removal — twice, independently**

1. **The branch held nothing `master` does not.**

   ```
   git rev-list --count master..worktree-agent-a4541bfe777803515   →  0
   git merge-base --is-ancestor worktree-agent-… master            →  true
   ```

   Zero commits unique to the branch, and its tip is an **ancestor of `master`**. Deleting the
   branch therefore deletes a label, not history. These are two separate questions and both were
   asked: a branch can have zero unique commits and still not be an ancestor, and only the second
   check proves the commit stays reachable.

2. **The working tree held nothing uncommitted.**

   ```
   git -C <worktree> status --porcelain   →  0 entries
   git stash list                          →  empty
   ```

   No modified files, no untracked files, no stashes. Nothing existed there that existed nowhere
   else.

**Verified after removal**

```
git worktree list      →  only D:/nelan/DeluluLang [master]
git branch             →  master, rc/1.0.0-drill
git merge-base --is-ancestor 51a378ab26… master   →  true   (history intact)
git status --porcelain →  clean
```

The commit is still reachable from `master`. Nothing in the tracked tree changed, so the Survey's
node and edge counts are unaffected — the removed copy was never in the map.

**How to recover**

The commit was never orphaned, so recovery is restoring a label:

```
git branch worktree-agent-a4541bfe777803515 51a378ab267effb9043c90f91856d8c2fe6607b4
```

And, if a working copy at that revision is wanted again:

```
git worktree add <path> worktree-agent-a4541bfe777803515
```

`rc/1.0.0-drill` was **not** touched. It was not in scope and was not examined.
