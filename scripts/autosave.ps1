<#
  autosave.ps1 - keep the laptop's work on GitHub even when the session that made it stops.

  The owner, 2026-10-17: "Can u run something parellely so that everything will get pushed to github repo even if
  usage limit hits". A session that reaches its usage limit stops mid-task, and whatever it had not pushed then waits
  on one disk. This runs OUTSIDE Claude - Windows Task Scheduler, every 10 minutes, no usage spent - and does two
  things:

  1. Pushes every local branch that is AHEAD of its upstream: a plain push, never a force; a branch that has
     diverged is logged and left alone. Commits are made only after the gates (AGENTS.md), so this pushes nothing
     a session would not have pushed itself a minute later.
  2. Snapshots UNCOMMITTED work - tracked edits and the new files git does not ignore - onto the branch
     `autosave/laptop`, without touching HEAD, the index or the working tree (a private index, write-tree,
     commit-tree). Each snapshot's parents are the previous snapshot and HEAD, so the branch only ever
     fast-forwards. A snapshot is NOT a verified commit: never merge it; the next session reads it and recovers
     what it needs. CI does not run on it (ci.yml runs on pushes to master and main only).

  It refuses a snapshot that carries the word the owner banned, or a file over 50 MB, and logs why.

  Install (a task for the current user, every 10 minutes for 7 hours): powershell -File scripts\autosave.ps1 -Install
  Remove:                                                               powershell -File scripts\autosave.ps1 -Uninstall
  One run by hand:                                                      powershell -File scripts\autosave.ps1
  Log: %LOCALAPPDATA%\DeluluLang\autosave.log
#>
param(
    [string]$Repo = (Split-Path -Parent $PSScriptRoot),
    [switch]$Install,
    [switch]$Uninstall,
    [int]$Hours = 7,
    [int]$Minutes = 10,
    [string]$Branch = 'autosave/laptop'
)
$ErrorActionPreference = 'Continue'
$TaskName = 'DeluluLang-autosave'
$State = Join-Path $env:LOCALAPPDATA 'DeluluLang'
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

# stdout only (PowerShell 5.1 turns a native program's stderr into error records; it is dropped here)
function Git-Quiet { & $git -C $Repo @args 2>$null }
# stdout and stderr into the log; the exit code stays in $LASTEXITCODE
function Git-Logged([string]$line) {
    $o = cmd.exe /d /c "`"$git`" -C `"$Repo`" $line 2>&1"
    $c = $LASTEXITCODE
    foreach ($l in $o) { Say "  $l" }
    $global:LASTEXITCODE = $c
}

Git-Logged 'fetch -q origin'
if ($LASTEXITCODE -ne 0) { Say 'fetch failed; nothing pushed'; exit 1 }

# 1. Committed work: every branch ahead of its upstream, fast-forward only.
foreach ($row in (Git-Quiet for-each-ref '--format=%(refname:short)|%(upstream:short)|%(upstream:track)' refs/heads)) {
    $b, $up, $track = $row -split '\|', 3
    if (-not $up) { continue }
    if ($track -match 'behind') {
        if ($track -match 'ahead') { Say "$b has diverged from $up $track - not pushed" }
        continue
    }
    if ($track -match 'ahead') {
        Say "push $b $track"
        Git-Logged "push -q origin $b"
        if ($LASTEXITCODE -ne 0) { Say "push $b FAILED" }
    }
}

# 2. Uncommitted work: a snapshot through a private index, so HEAD, the index and the tree are untouched.
$gitDir = Git-Quiet rev-parse --absolute-git-dir
$tmpIndex = Join-Path $State 'autosave.index'
Copy-Item -LiteralPath (Join-Path $gitDir 'index') -Destination $tmpIndex -Force
$env:GIT_INDEX_FILE = $tmpIndex
Git-Quiet add -A | Out-Null
$tree = Git-Quiet write-tree
Remove-Item Env:GIT_INDEX_FILE
if (-not $tree) { Say 'write-tree failed'; exit 1 }

$head = Git-Quiet rev-parse HEAD
$on = Git-Quiet rev-parse --abbrev-ref HEAD
if ($tree -eq (Git-Quiet rev-parse 'HEAD^{tree}')) { Say "clean: $on $($head.Substring(0, 7))"; exit 0 }
$prev = Git-Quiet rev-parse -q --verify "refs/remotes/origin/$Branch"
if ($prev -and ((Git-Quiet rev-parse "$prev^{tree}") -eq $tree)) { Say "unchanged since snapshot $($prev.Substring(0, 7))"; exit 0 }

$banned = 'graph' + 'ify'
if ((Git-Quiet diff $head $tree) -match $banned) { Say 'REFUSED: the snapshot carries the banned word; nothing pushed'; exit 1 }
$big = Git-Quiet ls-tree -r -l $tree | Where-Object { $s = ($_ -split '\s+')[3]; $s -match '^\d+$' -and [int64]$s -gt 50MB }
if ($big) { Say "REFUSED: a file over 50 MB: $($big -join '; ')"; exit 1 }

$parents = @()
if ($prev) {
    $parents += '-p', $prev
    Git-Quiet merge-base --is-ancestor $head $prev | Out-Null
    if ($LASTEXITCODE -ne 0) { $parents += '-p', $head }
} else {
    $parents += '-p', $head
}
$changed = @(Git-Quiet diff --name-only $head $tree)
$when = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm') + ' UTC'
$msg = @"
autosave: uncommitted laptop work on $on at $when

A snapshot of the working tree taken by scripts/autosave.ps1 (Windows Task Scheduler), so that work a
session had not yet committed survives the session stopping. It is NOT a verified commit: never merge
it; the next session compares it with its base and recovers what it needs.

Base: $head ($on)
Paths that differ from the base: $($changed.Count)

Co-Authored-By: Claude <noreply@anthropic.com>
"@
$msgFile = Join-Path $State 'autosave.msg'
[IO.File]::WriteAllText($msgFile, $msg.Replace("`r`n", "`n"), (New-Object Text.UTF8Encoding $false))
$commit = Git-Quiet commit-tree $tree @parents -F $msgFile
if (-not $commit) { Say 'commit-tree failed'; exit 1 }
Git-Logged "push -q origin ${commit}:refs/heads/$Branch"
if ($LASTEXITCODE -eq 0) {
    Say "snapshot $($commit.Substring(0, 7)) pushed to $Branch - $($changed.Count) path(s) over $($head.Substring(0, 7)) on $on"
} else {
    Say "snapshot $($commit.Substring(0, 7)) push FAILED"
}

$lines = @(Get-Content -LiteralPath $Log -ErrorAction SilentlyContinue)
if ($lines.Count -gt 2000) { $lines | Select-Object -Last 1000 | Set-Content -LiteralPath $Log -Encoding utf8 }
exit 0
