---
name: autosave-task-2026-10-17
description: "The Task Scheduler job DeluluLang-autosave (scripts/autosave.ps1): pushes branches holding commits origin lacks (fixed 2026-10-22, PR #43; was: ahead of upstream) + snapshots uncommitted work to autosave/laptop every 10 min, outside Claude; Jesse 2026-10-20: keep it ON, push frequently — re-install -Hours 168 whenever NextRunTime is empty"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-27T15:54:48.134Z
---

Jesse, 2026-10-17: *"continue cooking till usage limit hits. Can u run something parellely so that everything will get
pushed to github repo even if usage limit hits"*. Built `scripts/autosave.ps1` (commit `424c391`), installed with
`-Install` as Windows Task Scheduler job **`DeluluLang-autosave`**: every 10 min, LogonType Interactive (runs while
Jesse is signed in to Windows; independent of Claude Code and the terminal). Runs a copy at
`%LOCALAPPDATA%\DeluluLang\autosave.ps1`; log `autosave.log` there.

**Jesse, 2026-10-20: *"push everything to test github repo frequently. enable that windows task scheduler for automatic
github push"*** — the 7-hour install of 2026-10-19 had expired; re-installed **2026-10-19 22:42 UTC for 7 days**
(`-Hours 168`, ends ~2026-10-26); a hand run pushed snapshot `c94cbba`. **Why:** he wants GitHub to hold everything,
always. **How to apply:** push every verified commit immediately (branch + PR); at the start of every laptop session run
`Get-ScheduledTaskInfo DeluluLang-autosave` and, if `NextRunTime` is empty, `powershell -File scripts\autosave.ps1 -Install -Hours 168`
without asking.

Each run: `git fetch`; plain-push every local branch ahead of its upstream (diverged → logged only; a branch with no
upstream is not pushed — push new branches yourself); snapshot uncommitted work through a private index (`add -A` →
`write-tree` → `commit-tree`, parents = previous snapshot + HEAD) and push it to **`autosave/laptop`** — never merge it;
recover from it. Refuses the banned word, files over 50 MB, or a HEAD on no branch of origin. CI doesn't run on it.

After the launch, origin changes to DeluluLang — the task follows `origin` automatically, but the branch then lives on
DeluluLang (decide whether to keep snapshot branches public there). Related: [[session-start-github-sync]],
[[release-program-progress]], [[autosave-snapshot-is-a-draft]].

**Update 2026-10-21 18:03 IST:** Jesse asked "Is that windows task scheduler enabled and working". Checked: Ready, every 10 min, LastTaskResult 0, log shows snapshots pushed to autosave/laptop and "clean" runs; every branch equal to origin. Renewed with `scriptsutosave.ps1 -Install -Hours 168` → runs to 2026-10-28 18:03 IST. Renew again at the next session start if NextRunTime is near/empty.

**Update 2026-10-27 21:25 IST (Jesse: "Check autosave for pushing everything every 10 min"):** Ready, every 10 min, LastTaskResult 0, installed copy == `scripts/autosave.ps1` (#43's), log "clean" for both `master` and `D:\nelan\dl-launch`; log gaps = laptop off, and 3 "fetch failed" lines = no network at wake. Renewed `-Install -Hours 168` → runs to **2026-11-03 21:25 IST**.

**Update 2026-10-22 ~21:20 IST (Jesse: "is autosave working properly as intended, To which branch ... pushing everything"):** snapshot step works (`b35b6b1` 15:33 UTC; `838c739` == tree of `8a88466`); the 16 h log gap = laptop shut down 04:59–20:49 IST. **Push step was wrong (witnessed):** judged by the configured upstream, so a branch with NO upstream (every `git switch -c X master`) was skipped → its unpushed commit lived on disk alone; a `master` ahead was pushed ONTO origin master (bypassing land-on-green); `launch-merge-1020` pushed under its own name each run; branches tracking origin/master re-pushed every run. **Fix PR #43** (`laptop/autosave-push`, stacked on #42): push a branch under its own name when it holds a commit no origin branch holds (excluding `origin/autosave/*`); master/main → `autosave/laptop-master`/`-main`; diverged logged; orphan refused; `-DryRun`. Harness `scratchpad/nest/autosave_harness.sh` (bare origin + clone, one branch per case). **After #43 lands: re-install from the landed script** (`powershell -File scriptsutosave.ps1 -Install -Hours 168`) — the task runs the INSTALLED copy, which is still the old one until then.
