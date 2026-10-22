<#
  autosave.ps1 - keep the laptop's work on GitHub even when the session that made it stops.

  The owner, 2026-10-17: "Can u run something parellely so that everything will get pushed to github repo even if
  usage limit hits". A session that reaches its usage limit stops mid-task, and whatever it had not pushed then waits
  on one disk. This runs OUTSIDE Claude - Windows Task Scheduler, every 10 minutes, no usage spent - and does two
  things:

  1. Pushes every local branch that holds a commit no branch of origin holds, under its own name: a plain push,
     never a force; a branch that has diverged from its namesake on origin is logged and left alone. Commits are
     made only after the gates (AGENTS.md), so this pushes nothing a session would not have pushed itself a minute
     later - except `master` and `main`, which land only by fast-forward on a green run (owner, 2026-10-19): their
     unpushed commits go to `autosave/laptop-master` (or `-main`), never merged. A branch that shares no commit with
     origin is refused (after the launch, that is the whole development history).
  2. Snapshots UNCOMMITTED work - tracked edits and the new files git does not ignore - of every worktree onto a
     branch of its own: the repository's own checkout onto `autosave/laptop`, a linked worktree onto
     `autosave/laptop-<its folder's name>` (`D:\nelan\dl-launch`, where the release branch is merged, onto
     `autosave/laptop-dl-launch`; owner, 2026-10-22: "also snapshot the dl-launch worktree") - without touching
     HEAD, the index or the working tree (a private index, write-tree, commit-tree). Each snapshot's parents are
     the previous snapshot and HEAD, so the branch only ever fast-forwards. A snapshot is NOT a verified commit:
     never merge it; the next session reads it and recovers what it needs. CI does not run on it (ci.yml runs on
     pushes to master and main only).

  It refuses a snapshot that carries the word the owner banned, a file over 50 MB, or a HEAD that is on no branch of
  origin (the snapshot would carry history origin does not have - after the 2.0.0 launch, the whole development
  history into the public repository), and logs why.

  Install (a task for the current user, every 10 minutes for 7 hours): powershell -File scripts\autosave.ps1 -Install
  Remove:                                                               powershell -File scripts\autosave.ps1 -Uninstall
  One run by hand:                                                      powershell -File scripts\autosave.ps1
  One run that pushes nothing, logging what it would push:              powershell -File scripts\autosave.ps1 -DryRun
  Log: %LOCALAPPDATA%\DeluluLang\autosave.log
#>
param(
    [string]$Repo = (Split-Path -Parent $PSScriptRoot),
    [switch]$Install,
    [switch]$Uninstall,
    [switch]$DryRun,
    [int]$Hours = 7,
    [int]$Minutes = 10,
    [string]$Branch = 'autosave/laptop',
    [string]$State = (Join-Path $env:LOCALAPPDATA 'DeluluLang')
)
$ErrorActionPreference = 'Continue'
$TaskName = 'DeluluLang-autosave'
$Log = Join-Path $State 'autosave.log'
New-Item -ItemType Directory -Force $State | Out-Null

function Say([string]$m) {
    $t = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss')
    Add-Content -LiteralPath $Log -Value "${t}Z $m" -Encoding utf8
}

if ($Uninstall) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Say 'uninstalled'
    'uninstalled'
    exit 0
}

if ($Install) {
    $repoFull = (Resolve-Path -LiteralPath $Repo).Path
    # The task runs a copy kept outside the checkout: a branch switch must not take the script away from it.
    $copy = Join-Path $State 'autosave.ps1'
    Copy-Item -LiteralPath $PSCommandPath -Destination $copy -Force
    $ps = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
    $conhost = Join-Path $env:SystemRoot 'System32\conhost.exe'
    # conhost --headless: no console window flashes up every ten minutes on the owner's screen.
    $arg = "--headless `"$ps`" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$copy`" -Repo `"$repoFull`" -Branch `"$Branch`""
    $action = New-ScheduledTaskAction -Execute $conhost -Argument $arg
    $trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) `
        -RepetitionInterval (New-TimeSpan -Minutes $Minutes) -RepetitionDuration (New-TimeSpan -Hours $Hours)
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable `
        -MultipleInstances IgnoreNew -ExecutionTimeLimit (New-TimeSpan -Minutes 5)
    Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Force `
        -Description "DeluluLang: push branches ahead of GitHub and snapshot uncommitted work to $Branch (scripts/autosave.ps1)" | Out-Null
    Say "installed: every $Minutes min for $Hours hours, repo $repoFull, branch $Branch"
    "installed: every $Minutes min for $Hours hours; log $Log"
    exit 0
}

$git = (Get-Command git -ErrorAction SilentlyContinue).Source
if (-not $git) { Say 'git not found'; exit 1 }

# The checkout git runs in: the repository's own, or the worktree being snapshotted (step 2).
$script:Here = $Repo
# stdout only (PowerShell 5.1 turns a native program's stderr into error records; it is dropped here)
function Git-Quiet { & $git -C $script:Here @args 2>$null }
# stdout and stderr into the log; the exit code stays in $LASTEXITCODE
function Git-Logged([string]$line) {
    $o = cmd.exe /d /c "`"$git`" -C `"$($script:Here)`" $line 2>&1"
    $c = $LASTEXITCODE
    foreach ($l in $o) { Say "  $l" }
    $global:LASTEXITCODE = $c
}

Git-Logged 'fetch -q --prune origin'
if ($LASTEXITCODE -ne 0) { Say 'fetch failed; nothing pushed'; exit 1 }

# 1. Committed work: every branch holding a commit that no branch of origin holds, fast-forward only. Not judged by the
# branch's upstream (2026-10-22): a branch made with `git switch -c X master` has none, so a session's commit on it that
# the session had not pushed was skipped here and, its tree clean, never snapshotted - kept on this disk alone; a branch
# whose upstream has another name (`launch-merge-1020`, `launch/2.0.0`) went to origin under its own name every run, and
# one tracking `origin/master` was pushed again every run, unchanged. The snapshots are left out of "origin holds it":
# each carries HEAD as a parent.
foreach ($b in (Git-Quiet for-each-ref '--format=%(refname:short)' refs/heads)) {
    $only = Git-Quiet rev-list --count "refs/heads/$b" --not "--exclude=origin/$Branch*" '--exclude=origin/autosave/*' --remotes=origin
    if ($LASTEXITCODE -ne 0 -or $only -notmatch '^\d+$') { Say "$b - could not count its commits; not pushed"; continue }
    if ([int]$only -eq 0) { continue }
    if ([int]$only -eq [int](Git-Quiet rev-list --count "refs/heads/$b")) {
        Say "REFUSED: $b shares no commit with origin - pushing it would carry history origin does not have"
        continue
    }
    $to = $b
    if ($b -eq 'master' -or $b -eq 'main') { $to = "$Branch-$b" }
    $there = Git-Quiet rev-parse -q --verify "refs/remotes/origin/$to"
    if ($there -and $there -eq (Git-Quiet rev-parse "refs/heads/$b")) { continue }
    if ($there) {
        Git-Quiet merge-base --is-ancestor $there "refs/heads/$b" | Out-Null
        if ($LASTEXITCODE -ne 0) { Say "$b has diverged from origin/$to - not pushed ($only commit(s) only here)"; continue }
    }
    if ($DryRun) { Say "would push $b to $to - $only commit(s) not on origin"; continue }
    Say "push $b to $to - $only commit(s) not on origin"
    Git-Logged "push -q origin refs/heads/${b}:refs/heads/$to"
    if ($LASTEXITCODE -ne 0) { Say "push $b FAILED" }
}

# 2. Uncommitted work, in every worktree: a snapshot through a private index, so HEAD, the index and the tree are
# untouched. Until 2026-10-22 only the repository's own checkout was read, so what a session left uncommitted in a
# linked worktree (`D:\nelan\dl-launch`, where `launch/2.0.0` is merged) waited on this disk alone.
function Snapshot([string]$Dir, [string]$To, [string]$IndexName) {
    $script:Here = $Dir
    $where = if ($Dir -eq $Repo) { '' } else { " in $Dir" }
    $gitDir = Git-Quiet rev-parse --absolute-git-dir
    if (-not $gitDir) { Say "no git directory$where; nothing snapshotted"; return $false }
    $tmpIndex = Join-Path $State $IndexName
    Copy-Item -LiteralPath (Join-Path $gitDir 'index') -Destination $tmpIndex -Force
    $env:GIT_INDEX_FILE = $tmpIndex
    Git-Quiet add -A | Out-Null
    $tree = Git-Quiet write-tree
    Remove-Item Env:GIT_INDEX_FILE
    if (-not $tree) { Say "write-tree failed$where"; return $false }

    $head = Git-Quiet rev-parse HEAD
    $on = Git-Quiet rev-parse --abbrev-ref HEAD
    if ($tree -eq (Git-Quiet rev-parse 'HEAD^{tree}')) { Say "clean: $on $($head.Substring(0, 7))$where"; return $true }
    $prev = Git-Quiet rev-parse -q --verify "refs/remotes/origin/$To"
    if ($prev -and ((Git-Quiet rev-parse "$prev^{tree}") -eq $tree)) {
        Say "unchanged since snapshot $($prev.Substring(0, 7))$where"
        return $true
    }

    # A snapshot carries HEAD's history, so it may go only where that history already is. Since the 2.0.0 launch
    # `origin` is the public repository with a fresh history (D-V2-153): a snapshot of a checkout still on the
    # development history would publish all of it there. (`--prune` above drops the old repository's refs once
    # `origin` points elsewhere.)
    if (-not (Git-Quiet for-each-ref --contains $head '--format=%(refname)' refs/remotes/origin)) {
        Say "REFUSED: HEAD $($head.Substring(0, 7))$where is on no branch of origin - a snapshot would carry history origin does not have; nothing pushed"
        return $false
    }

    $banned = 'graph' + 'ify'
    if ((Git-Quiet diff $head $tree) -match $banned) { Say "REFUSED: the snapshot$where carries the banned word; nothing pushed"; return $false }
    $big = Git-Quiet ls-tree -r -l $tree | Where-Object { $s = ($_ -split '\s+')[3]; $s -match '^\d+$' -and [int64]$s -gt 50MB }
    if ($big) { Say "REFUSED: a file over 50 MB$where`: $($big -join '; ')"; return $false }

    $parents = @()
    if ($prev) {
        $parents += '-p', $prev
        Git-Quiet merge-base --is-ancestor $head $prev | Out-Null
        if ($LASTEXITCODE -ne 0) { $parents += '-p', $head }
    } else {
        $parents += '-p', $head
    }
    $changed = @(Git-Quiet diff --name-only $head $tree)
    if ($DryRun) {
        Say "would snapshot $($changed.Count) path(s) over $($head.Substring(0, 7)) on $on$where to $To"
        return $true
    }
    $when = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm') + ' UTC'
    $msg = @"
autosave: uncommitted laptop work on $on at $when

A snapshot of the working tree taken by scripts/autosave.ps1 (Windows Task Scheduler), so that work a
session had not yet committed survives the session stopping. It is NOT a verified commit: never merge
it; the next session compares it with its base and recovers what it needs.

Worktree: $Dir
Base: $head ($on)
Paths that differ from the base: $($changed.Count)

Co-Authored-By: Claude <noreply@anthropic.com>
"@
    $msgFile = Join-Path $State 'autosave.msg'
    [IO.File]::WriteAllText($msgFile, $msg.Replace("`r`n", "`n"), (New-Object Text.UTF8Encoding $false))
    $commit = Git-Quiet commit-tree $tree @parents -F $msgFile
    if (-not $commit) { Say "commit-tree failed$where"; return $false }
    Git-Logged "push -q origin ${commit}:refs/heads/$To"
    if ($LASTEXITCODE -ne 0) { Say "snapshot $($commit.Substring(0, 7))$where push FAILED"; return $false }
    Say "snapshot $($commit.Substring(0, 7)) pushed to $To - $($changed.Count) path(s) over $($head.Substring(0, 7)) on $on$where"
    return $true
}

# The repository's own checkout first, under the name it always had; then each linked worktree that is still on disk,
# under its folder's name. `git worktree list --porcelain` names the repository's own checkout first.
$trees = @(Git-Quiet worktree list --porcelain | Where-Object { $_ -like 'worktree *' } | ForEach-Object { $_.Substring(9) })
$ok = Snapshot $Repo $Branch 'autosave.index'
foreach ($wt in ($trees | Select-Object -Skip 1)) {
    $dir = $wt -replace '/', '\'
    if (-not (Test-Path -LiteralPath $dir)) { Say "worktree $dir is gone; nothing snapshotted (git worktree prune)"; continue }
    $leaf = (Split-Path -Leaf $dir) -replace '[^A-Za-z0-9._-]', '-'
    if (-not (Snapshot $dir "$Branch-$leaf" "autosave-$leaf.index")) { $ok = $false }
}
$script:Here = $Repo

$lines = @(Get-Content -LiteralPath $Log -ErrorAction SilentlyContinue)
if ($lines.Count -gt 2000) { $lines | Select-Object -Last 1000 | Set-Content -LiteralPath $Log -Encoding utf8 }
if ($ok) { exit 0 } else { exit 1 }
