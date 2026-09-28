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
  follow-up `bc9192c` Cloud handoff (2): run and check everything with the Survey and doctor; and
  `937aea8` Cloud handoff (3): the documents brought up to date for the cloud period; and
  `7e67f97` Cloud handoff (4): the routine's loop, and the owner's delegation; `7bd018c` Cloud handoff
  (5): a run can end at any moment — push each verified slice; and `Cloud handoff (6): the routine's
  mandate — every phase, any file, xhigh effort` — the baseline.
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
  (3) M `docs/DELULULANG_V2/V2_README.md`, `V2_PHASE_STATUS.md`, `V2_SECURITY_MODEL.md`,
      `V2_IMPLEMENTATION_ROADMAP.md`, `V2_AGENT_LOG.md` — current state, the cloud period, PS-D status
  (3) M `docs/REMAINING_WORK.md` — 6.5 closed (P4-07 had built it), 2.6/4.10/7.5/7.10a updated, 4.24 new
  (3) M `docs/DEPLOYMENT.md`, `docs/for-agents.md`, `docs/GETTING_STARTED.md`, `docs/QUESTIONS.md`,
      `docs/MATHEMATICS.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/REPOSITORY_STRUCTURE.md` —
      stale sandbox, microVM and residual statements corrected; the sandbox levels; AGENTS.md pointers
  (4) A `docs/CLOUD_ROUTINE.md` — the loop every scheduled run follows; M `AGENTS.md`, `CLAUDE.md`,
      `HANDOFF.md`, `README.md`, the V2 status files, `REMAINING_WORK.md` 4.24, the structure guide —
      the owner's delegation (PS-D-02's stop superseded; five decisions stay his), the routine on `master`
  (6) A `.claude/settings.json` (`effortLevel: xhigh`); M `docs/CLOUD_ROUTINE.md` (finish every phase incl.
      P8's software; any file may change; four decisions stay his; step 8, improve the loop), `AGENTS.md`,
      `HANDOFF.md` — the owner's noon mandate
- Survey and doctor: `survey check` ok (1,456 nodes, 12,427 edges); `survey findings` 0 errors,
  14 warnings (12 new ones are the memory snapshot's dated citations, kept verbatim by design);
  `doctor --check` ok, 29 checks passed on the laptop.
- Verified: `repository_structure`, `doctor_cli` and the Survey's freshness test pass; CI for `30a6b8d`
  and `047da1d` read (`V2_LOG.md` 2026-09-28); the handoff commits' own push runs are the first thing
  the first cloud session reads.
- Redo on the laptop: nothing.
- For the laptop's memory: the cloud period and this file (already in the laptop's memory).
- Open / next: read the `miri-slow` run `36381950975` on `047da1d` — green closes REMAINING_WORK 5.6 —
  and the push runs of the handoff commits — read at 07:10 UTC: `d41e558`, `bc9192c`, `7e67f97`, `7bd018c`
  green; **`937aea8` red on Windows only**: `actors_pingpong`'s criterion-1 speed-up measured 1.31x
  against its 1.5x bar on a docs-only commit, and the next two commits passed it — a timing criterion
  measuring a busy 4-CPU runner. Investigate it the project's way (`HANDOFF.md` §11.5: starve the
  runner, never lengthen or loosen blindly) and record the verdict. `miri-slow` `36381950975`: syntax
  green, **check green — its first complete run ever** (59 min), broker still running. Then
  **PS-D-02**, then `docs/CLOUD_ROUTINE.md` step 4's order.

### 2026-09-28 — routine run 1: CI read; the ping-pong criterion judged against a control (D-V2-47)
- Session: `https://claude.ai/code/session_013AZJ6RqYq59CeMgkvLV1BM`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there, `docs/CLOUD_ROUTINE.md`)   Pull request: none   Merged: n/a
- Base: `5bb39bc`
- Commits: (1) `9ac5477` actors_pingpong: judge criterion 1 against a control on a free machine (D-V2-47);
  (2) `PS-D-02: the attestation seam — --require-attestation, sandbox attest (D-V2-48)`
- Files and folders:
  M `crates/delulu-runtime/tests/actors_pingpong.rs` — (1) the control, the quiet-machine rule, the share-of-machine bar
  A `crates/delulu/src/attest.rs` — (2) the statement, its canonical bytes, the verifier, the reference attester
  A `crates/delulu/tests/sandbox_attest_cli.rs` — (2) five end-to-end tests
  M `crates/delulu/src/guest.rs` — (2) the pinned key on the external backend; nonce and path to the launcher; verify before the hello
  M `crates/delulu/src/cli.rs` — (2) `--require-attestation`; the help lines; `own_words` (nothing after `--` is delulu's)
  M `crates/delulu/src/main.rs`, `run_cmd.rs`, `sandbox.rs`, `schema.rs` — (2) the module; the refusal without `--sandbox`; the verb; `sandbox.attestation`
  M `CHANGELOG.md`, `HANDOFF.md`, `docs/DEPLOYMENT.md`, `docs/for-agents.md`, `docs/REMAINING_WORK.md` (4.24) — (2)
  M `docs/DELULULANG_V2/V2_DECISION_LOG.md` — D-V2-47 (1), D-V2-48 (2)
  M `docs/DELULULANG_V2/V2_LOG.md` — this run's entry (1)(2)
  M `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `V2_IMPLEMENTATION_ROADMAP.md`, `V2_SECURITY_MODEL.md` — (2) PS-D-02 built
  M `docs/CLOUD_SYNC_LOG.md` — this entry
  M `docs/survey/*` — regenerated
- Survey and doctor (after the last edit): see the commit message of each slice.
- Verified: in the VM — (1) the ping-pong test idle (measured, 2.03–2.16x), with three CPUs starved (NOT
  MEASURED, where the old test was red at 1.20x), and against a one-worker runtime mutant (red, 1.00x);
  (2) six unit and five end-to-end tests, six mutants each red on its own assertion; clippy (workspace,
  `-D warnings`) clean; the full suite alone, 1,995 passed and 1 failed — `egress_features`, which needs
  `cargo fetch` in a fresh VM and then passes (1,996 of 1,996). CI: `miri-slow` `36381950975` syntax and
  check green, broker running; `5bb39bc`'s push run `36390274072` red on Windows only, `actors_pingpong`
  1.31x/1.28x — the defect (1) fixes.
- Redo on the laptop: nothing (CI runs Windows and macOS).
- For the laptop's memory: a fresh VM needs `cargo fetch --locked` before the suite (`egress_features`
  runs `cargo metadata --offline`); a routine run has no `gh` — CI is read with the GitHub MCP tools.
- Open / next: read `9ac5477`'s and this commit's push runs (PS-D closes when the latter is green on
  every job — then `V2_PHASE_STATUS.md` row 13 says complete), and `36381950975`'s broker job (green
  closes RW 5.6); then P7's RW 5.2 (Progress restated as progress-or-fault in `DELULU_CORE.md`).
