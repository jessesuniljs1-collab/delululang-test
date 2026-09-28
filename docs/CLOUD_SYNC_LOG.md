# Cloud sync log — every change made away from the owner's laptop

**Why this file exists.** From **2026-09-28 to 2026-10-16** the owner's laptop is in use for other
work, and DeluluLang is developed from Claude Code **cloud sessions** against the GitHub repository
(`HANDOFF.md` §0). The laptop's checkout, `D:\nelan\DeluluLang`, will be brought up to date afterwards.
Git carries the commits; this file carries what git does not make obvious at a glance — which files
and folders each session touched, what it verified, what must be redone on the laptop, and what the
laptop's own assistant memory must learn.

**The rule (owner, 2026-09-28): every cloud session appends one entry here, in the same pull request as
its changes.** A session that changes nothing still records that it ran, if it read or recorded
anything (a CI run, for example).

## The baseline

The laptop's checkout is at **the newest commit whose subject begins `Cloud handoff`** — find it with
`git log -1 --format='%h %s' --grep='^Cloud handoff'` — on `master`, clean, with nothing unpushed
(checked on 2026-09-28 with `git status` and `git ls-remote origin refs/heads/master`). Everything after
it on `origin` was made in the cloud and is listed below.

## Syncing the laptop afterwards (when the owner is back)

1. On the laptop, in `D:\nelan\DeluluLang`: `git status` must be clean (it was at the baseline).
2. `git fetch origin`, then `git log --oneline <baseline>..origin/master` — compare with the entries
   below; every commit should belong to one of them.
3. `git pull --ff-only origin master`. Never force anything; if it is not a fast-forward, stop and
   look — the laptop was not supposed to move.
4. `git ls-files --eol | grep w/crlf` must print nothing (a CRLF working tree broke the Survey once —
   `HANDOFF.md` §11.5).
5. **The Survey and `doctor`:** `cargo run -p delulu-survey -- check` (the map the cloud committed
   must match the laptop's tree — CRLF or a stray file shows here first), `-- findings` (0 errors),
   `cargo run -p delulu -- doctor --check` (all checks pass); then the full suite on Windows, and on
   Linux in WSL (`HANDOFF.md` §11.3), and `doctor --check` in WSL too.
6. Do each entry's **"Redo on the laptop"** items (for example: rebuild the microVM guest image in WSL
   if `scripts/microvm/` changed; re-run a Windows-only check).
7. Carry each entry's **"For the laptop's memory"** items into
   `C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\` (or tell the laptop session to read
   `HANDOFF.md` §11, which holds them too).
8. Delete no branch and no entry here: this log stays as the record of the cloud period.

## Entry template

```
### <YYYY-MM-DD> — <one-line summary>
- Session: <https://claude.ai/code/session_…>   Model: <model actually running>
- Branch: <claude/…>   Pull request: <#n or URL>   Merged: <yes, by the owner, <date> / not yet>
- Base: <hash the branch started from>
- Commits: <hash subject> (one per line; `git log --oneline <base>..HEAD`)
- Files and folders (`git diff --name-status <base>..HEAD`, or `cargo run -p delulu-survey -- diff <base>`):
  A <added path>
  M <modified path>
  D <deleted path — never a .md>
  R <old path> -> <new path>
- Survey and doctor (after the last edit): `survey check` <ok / stale>, `survey findings` <N errors,
  N warnings>, `doctor --check` <ok: all checks passed / the failing check>
- Verified: <what ran, where: local VM tests, CI run ids READ with their results>
- Redo on the laptop: <Windows-only or WSL/KVM checks, rebuilt artifacts — or "nothing">
- For the laptop's memory: <durable facts added to HANDOFF §11 — or "nothing">
- Open / next: <what the next session should pick up>
```

## Entries

### 2026-09-28 — the handoff to the cloud (made on the laptop)
- Session: the laptop's head-chef session (Claude Opus 5.5), `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`
- Branch: `master` (the laptop pushes to `master` directly; cloud sessions cannot)
- Base: `047da1d`
- Commits: `d41e558` Cloud handoff: AGENTS.md, CLAUDE.md, the sync log, the memory snapshot; and the
  follow-up `Cloud handoff (2): run and check everything with the Survey and doctor` — the baseline.
- Files and folders:
  A `AGENTS.md` — the rules every agent reads
  A `CLAUDE.md` — imports `AGENTS.md`; Claude-specific notes for cloud sessions
  A `docs/CLOUD_SYNC_LOG.md` — this file
  A `docs/assistant-memory/` — the laptop's memory directory (40 files + its README), sanitized
  M `HANDOFF.md` — §0 the cloud period; state as of 2026-09-28; §11 memory brought up to date
  M `README.md` — status: the latest CI run, V2 progress, the microVM layer now built
  M `docs/REPOSITORY_STRUCTURE.md` — rows for the three new files
  M `docs/REMAINING_WORK.md`, `docs/DEPLOYMENT.md` — CONTAIN-TOCTOU-1 is closed (FS-RACE-1, P5c)
  M `docs/DELULULANG_V2/V2_LOG.md` — the handoff entry
  M `docs/survey/*` — regenerated
- Survey and doctor: `survey check` ok (1,456 nodes, 12,427 edges); `survey findings` 0 errors,
  14 warnings (12 new ones are the memory snapshot's dated citations, kept verbatim by design);
  `doctor --check` ok, 29 checks passed on the laptop.
- Verified: `repository_structure`, `doctor_cli` and the Survey's freshness test pass; CI for `30a6b8d`
  and `047da1d` read (`V2_LOG.md` 2026-09-28); the handoff commits' own push runs are the first thing
  the first cloud session reads.
- Redo on the laptop: nothing.
- For the laptop's memory: the cloud period and this file (already in the laptop's memory).
- Open / next: read the `miri-slow` run `36381950975` on `047da1d` — green closes REMAINING_WORK 5.6;
  PS-D-02 waits for the owner's word; the rest is `V2_PHASE_STATUS.md` and `HANDOFF.md` *Where things stand*.
