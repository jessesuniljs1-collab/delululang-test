---
name: delulu-github-remote
description: "Since 2026-09-14 DeluluLang is pushed to Jesse's testing repo (origin), PUBLIC since 2026-09-17; push only there — the final public repo is a different one and Jesse's own step"
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-09-17T02:03:02.995Z
---

**2026-09-14: Jesse explicitly superseded the standing "NEVER push to GitHub" rule.**
`origin` = `https://github.com/jessesuniljs1-collab/delululang-test.git`. It is **PUBLIC since
2026-09-17**: it was private from 2026-09-14, until Jesse changed the visibility.
His stated purpose: testing DeluluLang *"in mac os and other os"* (CI's `macos-latest` job) and
*"editing and working on delululang from the cloud"*. **Later he will push everything to a separate
PUBLIC GitHub repo of his. That is HIS step: never create one, push to one, or add any other remote
without his explicit instruction.**

First push = commit `5471d05`, verified with `git ls-remote` (every ref MATCHES local): `master` (the
GitHub default branch; local tracks `origin/master`), `rc/1.0.0-drill` (DRILL-001's one unmerged
commit) and the annotated tag `v1.0.0` → `198bf44`. The branch was kept as `master` on purpose, not
GitHub's suggested `main`; `ci.yml` triggers on both. Pre-push checks came back clean: no secrets in
any of the 332 commits, largest blob 1.75 MB (`survey.json`), and no code or test depends on there
being no remote. Auth works non-interactively through Git Credential Manager (`gh` is NOT installed);
set `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=Never` so a push fails fast instead of hanging.

**Why:** the rule that stopped CI from ever running is gone, so macOS can finally be *executed*, by CI.

**How to apply:**
- Push only to `origin`, and never rewrite pushed history (the docs cite commit hashes everywhere).
  **Since 2026-09-17, push every commit there immediately, without asking**, and keep local and
  GitHub in sync ([[testing-repo-autopush]]).
- **Activated ≠ executed.** A CI run is not evidence until its result has been read and transcribed
  into `docs/design/CROSS_PLATFORM_VERIFICATION.md` with the run ID. Every "never executed" row
  stands until then. `REMAINING_WORK.md` 7.2 carried the §0 marker *Pending result* until run 5 closed it (2026-09-14).
- **Cost:** `ci.yml` has a nightly `schedule` (03:00 UTC) that re-runs EVERY job (the 3-OS matrix,
  arm64, Miri ×5, formal models…) against a private repo's metered minutes. GitHub has charged macOS
  at roughly 10× Linux; check the billing page for current rates. Flagged to Jesse and left unchanged:
  his call.
- **Before the PUBLIC push, these are Jesse's decisions** (HANDOFF §1.1 lists them). The banned word
  ([[no-banned-word-mentions]]) is in the HISTORY: commit `0a58451` has it in its diff and its message,
  with the tool's URL. `f4ffd01` anonymised only the tree, and HANDOFF's own rule statements spell
  the word. The options: a fresh history for the public repo, a rewrite (which changes every hash
  from `0a58451` on), or accepting it. Also: the author e-mail in every commit, the CODEOWNERS
  placeholder `@PENDING-PUBLIC-project-lead`, and the `SECURITY.md` PENDING-PUBLIC controls.
  **Present the options; never decide for him.**
- A plain clone lands in `delululang-test/`, so README and INSTALL now clone into `DeluluLang`
  explicitly.

## The first CI run went red (2026-09-14): two real causes, fixed in `a52dc39`

- **None of the six Miri jobs could ever run.** `rust-toolchain.toml` pins 1.96.1, and that pin
  OUTRANKS the toolchain a CI action installs (`dtolnay/rust-toolchain@nightly`). Miri is
  nightly-only, so a bare `cargo miri` dies in seconds. Fix: `cargo +nightly miri`, plus `rust-src`.
  **General lesson: a toolchain file in the tree silently overrides every CI toolchain step, so any
  nightly-only tool needs `+nightly`.** The docs had claimed "every command in ci.yml has been run by
  hand", but this one, as written, could not run.
- **`cargo deny` blocked on RUSTSEC-2026-0268/0269** (wasmtime 47.0.3, in WASI code DeluluLang never
  uses). Fixed with `cargo update -p wasmtime` → 47.0.4, lockfile only. Suite 1,645 passed / 0
  failed; `clippy -D warnings` clean.
- **Nightly is opt-in now:** every job carries
  `if: github.event_name != 'schedule' || vars.NIGHTLY == 'on'` (owner decision). To enable it, set
  the repo variable `NIGHTLY=on` (e.g. on the public repo).
- **GitHub CLI 2.100.0 is installed** at `C:\Program Files\GitHub CLI\gh.exe`. A session started
  before the install won't have it on PATH, so use the full path. Jesse has to `gh auth login` once;
  after that, read runs with `gh run list` / `gh run view <id> --log-failed`.
- Done 2026-09-14, as Jesse chose: run 1 was read and recorded (`CROSS_PLATFORM_VERIFICATION.md` §9),
  then `a52dc39` and `6abdfae` were pushed together, after a fresh-clone check reproduced green. That
  push triggered run 2, `34836508713`. `6abdfae` fixed the rest of run 1: the snapshot and the Survey
  (see [[delulu-clean-checkout]]) and macOS's libffi (macOS now links the system libffi). The x86
  test jobs now run `--no-fail-fast`.

## The second run (2026-09-14, `34836508713`): macOS RUNS

- **macOS built the whole workspace and passed 1,654 of 1,655 tests.** It was the first DeluluLang
  execution on a Mac, and the system-libffi switch worked. arm64, clippy, cargo deny, editor, formal,
  and Miri atlas/diag/ffi were all green.
- **It exposed two real defects.** Both are fixed in the next commit (pending run 3), and each fix was
  witnessed failing first:
  1. **A legible refusal, delivered illegibly.** `broker_transport` refuses a socket path over
     `sun_path` BY NAME, but only inside the detached daemon, whose output goes to broker.log. The
     parent saw only a 5 s timeout. Fix: `broker start` calls `check_state_dir` before spawning.
     **Lesson: a refusal in a detached process reaches nobody. Check in the process the user is
     looking at.**
  2. **A fresh `DELULU_STATE_DIR` could not start the broker at all.** The daemon was spawned with a
     missing cwd (os error 267 / ENOENT) unless `--guard-policy`/`--require-anchored-roots` created
     the directory. Found because my first witness forgot to create its dir, so the failure it hit was
     this bug and not the one under test ([[delulu-reproduce-the-shape]] again).
- **Tests that measured the RUNNER** (private-repo runners have 2 vCPUs):
  - The pingpong speedup is now asserted only with ≥4 hardware threads; otherwise it prints NOT
    MEASURED. It measures 2.16x here, with 16 threads.
  - The hw-adapter test's Windows driver is now Python. PowerShell's start-up overran the deliberate
    2000 ms `EXCHANGE_TIMEOUT`; never stretch that timeout.
  - The federation fixture's paths are shortened: macOS `$TMPDIR` is 49 chars, and the socket limit
    is 103.
- **Miri: `+nightly` works, but broker, syntax and check cannot finish in 45 minutes on a 2-vCPU
  runner.** All three were cancelled with 0 UB. broker stalled in
  `guard::tests::tier_for_subset_agrees_with_use_time_sealing_over_generated_policies`, syntax in
  `parser::tests::nesting_past_the_limit_is_refused_with_a_diagnostic_not_a_crash`, and check runs at
  ~30 s per test. None of them has `unsafe`.
  - They moved to `miri-slow` (nightly/manual, 240 min, a budget no run has confirmed).
    atlas/diag/miri-ffi stay on every push.
  - Follow-up is REMAINING_WORK 5.6: shrink those tests under `cfg!(miri)`.
  - **When diffing a test list against a log, subtract `-- --list --ignored` first.** An ignored test
    is never the one "still running"; I named `fmt_laws_100k_gate` before catching that.
- **To read a finished job's log while the RUN is still in progress:** `gh run view --log-failed`
  refuses, so use `gh api --allow-escape-sequences repos/<o>/<r>/actions/jobs/<id>/logs`.

- **Pushed:** `28e10e6` (all of run 2's fixes, plus the record) went up after a fresh-clone check
  reproduced green. That push started run 3, `34841317790`.

## Run 3 (2026-09-14, `34841317790`): **macOS GREEN**, end to end

Every step of `test (macos-latest)` passed:
- 125 binaries: 1,657 passed, 0 failed, 4 ignored.
- CLI sweep: 27/27.
- Fuzz: 50,000 generated, 30,198 executed, 0 escapes.
- Conformance: 330/330 anchors; reference in sync.
- fmt: clean.

arm64, lints, supply-chain, editor, formal and Miri atlas/diag/ffi also passed; miri-slow and
heavy-gates were skipped by design. **REMAINING_WORK 7.1 (macOS never executed) is CLOSED.** What CI
cannot close: a developer's Mac, and the VS Code extension on one. The Tier-2 cross-account boundary
was tested on Linux only.

- **Windows in run 3: 1,646/1,647.** `adapter::tests::a_garbled_reading_is_an_error_never_a_none_and_never_a_number`
  failed with `got Err(Timeout)`. It spawns five PowerShell fake drivers in a row, and each first
  exchange's deliberate 2000 ms budget includes PowerShell's cold start on a loaded 2-vCPU runner.
  This is the same class as the hw_adapter_cli driver; it passed in run 2 by luck.
  **Rule: never let a PowerShell start-up sit inside a timed test budget on CI.**
- Pushed `f1d5a79`, the run-3 record, with `[skip ci]`: docs only, and the code was identical to
  what run 3 had exercised. Verified afterwards that no CI run started for it.
- Pushed `ef9cb49`: every fake driver in `adapter.rs`'s tests (reply, echo, silent, two-line, flood)
  is now `python -c` on Windows; the sh branch is unchanged. That removes the last PowerShell test
  driver in the tree. Adapter tests 11/11 locally, and the silent driver's deadline still fires. The
  push started run 4, `34844151767`, the run that can show Windows green on CI.

## Run 4 (2026-09-14, `34844151767`, on `ef9cb49`): **Windows GREEN end to end**, one macOS flake

- **Windows passed every step for the first time on CI:** 125 binaries, 1,647 passed / 0 failed / 4
  ignored; conformance 330/330; reference in sync; sweep 27/27; fuzz 30,198 executed, SOUND. The
  Python drivers held. Linux x86-64 and arm64 green again. So every OS has now passed on CI (macOS in
  run 3, Windows in run 4, Linux in both), though not yet all in one run.
- **macOS failed `device::tests::a_beaten_lease_is_never_revoked`** (a 120 ms lease beaten every
  20 ms). The runner left the test thread unscheduled for more than 100 ms, so the watchdog was RIGHT
  to revoke. Jesse pasted an outside diagnosis ("race in Stepped mode", with two fixes: sleep 50 ms,
  or a stepped clock) and said **"Verify before doing anything."** Experiments showed both fixes
  wrong ([[delulu-timing-tests]]):
  - The test runs on the Wall clock; Stepped mode never runs in it.
  - A 50 ms sleep tolerates LESS stall (60–70 ms, against about 100 ms at 20 ms).
  - The stepped rewrite passes a mutant watchdog that fires 200 ms early. ONLY the wall-clock
    beaten-lease test catches that mutant.
- **Fix `010c36c`** (8 files). Pushed 2026-09-14 13:33 UTC on Jesse's "push it", which started run 5
  `34849980129`. A sync check right after the push found every ref identical on GitHub: `master`
  `010c36c`, `rc/1.0.0-drill` `acc6f11`, and tag `v1.0.0` `991ebab` (→ `198bf44`). There were no
  local-only refs, no stashes, no extra worktrees; only ignored build output lives only on disk. The
  fix itself:
  - The test judges each revocation against the gap its thread actually left:
    `hb + overdue_us ≤ time since the last accepted SEND + 1 µs`.
  - A gap-explained revocation restarts the drive on a fresh broker. After 10 s with no clean drive
    it fails as NOT MEASURED.
  - Verified 25/25 idle, 12/12 starved, and the mutant is still caught.
  - The two siblings (`safe_park…` at 40 ms, `dead_man_cli` at 25 ms to its first command) passed
    20/20 starved, so they are not exposed.
- **HANDOFF header:** take "lines of Rust" from the `| Rust lines |` row in `docs/survey/SURVEY.md`,
  never from arithmetic. The old 111,375 was 13 lines stale; it is now 111,437, and HEAD is 195
  commits past v1.0.0.
- REMAINING_WORK 7.2 stayed **Pending result** until a run had no failures: run 5, below.

## Run 5 (2026-09-14, `34849980129`, on `010c36c`): **GREEN on every job, all three OSes in one run**

- **The first CI run with no failures.** 11 jobs passed. `heavy-gates` and `miri-slow` were skipped by
  design: they run on a schedule or by hand only. Neither has ever run on a runner, so the 240-minute
  Miri budget is still unconfirmed.
- **Test totals:**
  - macOS, ubuntu and arm64: 125 binaries, 1,657 passed, 0 failed, 4 ignored each.
  - Windows: 125 / 1,647 / 0 / 4. Windows compiles ten fewer tests; they are all platform-gated
    (same ignored count, no failures).
- **Every later step passed on the x86 runners:** fmt, the Python-less build, conformance 330/330,
  the reference in sync, the sweep 27/27, and fuzz SOUND over 30,198 programs. The dead-man test
  fixed in `010c36c` passed everywhere.
- **REMAINING_WORK 7.2 and HANDOFF §8's CI row are CLOSED.** Both strike through their original
  wording ("CI has never executed.", checked in git history), as 7.1 was closed.
- **Not covered by CI:**
  - a developer's Mac;
  - the VS Code extension off Windows;
  - the Tier-2 cross-account boundary on macOS;
  - the nightly/manual jobs.
- **"Update all .md files", done carefully.** Present-tense docs were updated. Dated records (build
  orders, campaign logs, dated reviews, the 1.0 announcement) keep their old macOS claims as history.
  Where a record would read as current, it got a dated note instead: the Stage 2/4 specs, the PRR
  register, and CHECKPOINT-1.0 limitation 1. **Check `query <id>` for ENTRENCHED first.** None of
  these docs was entrenched.
- **Docs commit `5a8840a`** (12 files, `[skip ci]`) was pushed 2026-09-14. Polling 45 s confirmed it
  started no CI run.
  - Before committing, every test binary that reads a touched doc passed: evidence_claims,
    distribution, doctor_cli, governance, surface_guide, release, book, delulu-conform, secret_oracle
    and laundering. The Survey tests passed 56/56, and doctor reported 17/17.
  - After the push every ref matched GitHub: `master` `5a8840a`, `rc/1.0.0-drill` `acc6f11`, and
    `v1.0.0` `991ebab` (→ `198bf44`). There were no GitHub-only refs, 196 commits past v1.0.0 (the
    same as HANDOFF's header), a clean tree and no stashes.
  - The latest CI run is still `34849980129` (success), on `010c36c`. `5a8840a` changed only docs, so
    its code is byte-identical to what that run verified.
  - Jesse asked "are they in sync": identical SHAs are content equality, so a fresh clone would add
    nothing. Only gitignored build output (`target/`, `dist/`, the `.vsix`, `node_modules/`) exists
    only on the disk.

## NIGHTLY: Jesse asked my opinion (2026-09-14). It stays OFF here; a heavy-only button instead

*(Superseded 2026-09-17, when the repository went public. The nightly is now opt-OUT and the button
defaults to `everything`; see the last section.)*

- **My advice: leave `NIGHTLY` off on this PRIVATE repo, and turn it on in the PUBLIC one**, where
  standard GitHub-hosted runners are free.
  - Job times measured in run 5: Linux jobs ~16 min, arm64 ~8, macOS ~8, Windows 15–28.
  - At the usual private-repo rates (macOS ~10×, Windows ~2×), one nightly run is ~150 billed
    minutes, plus up to ~780 for heavy-gates and miri-slow.
  - A month of nightlies is ~4,500–28,000 minutes, against 2,000 (Free) or 3,000 (Pro). Once the
    quota is gone, a $0 spending limit blocks the push runs too.
  - A nightly re-tests an unchanged commit. The only things that change without a push are new
    advisories, runner images, Miri's nightly toolchain, and flakes.
- **Billing usage is not readable here.** The `gh` token's scopes are gist, read:org and repo, with
  no `user` scope. The Actions timing API returned zeros. Send Jesse to Settings → Billing and plans.
- **Built at his request: a heavy-only manual button.** The commit is "CI: the manual button runs the
  heavy jobs only, unless told otherwise [skip ci]", committed as `ce0c115`, the one after `5a8840a` (197 commits past v1.0.0). Pushed on his "push it
  fast" (`5a8840a..ce0c115`). It started no CI run, and every ref matched GitHub afterwards: `master`
  `ce0c115`, `rc/1.0.0-drill` `acc6f11`, `v1.0.0` `991ebab`.
  - `workflow_dispatch` gains an input `jobs` (a choice, default `heavy`):
    - `heavy`: heavy-gates + miri-slow;
    - `heavy-gates`: that one alone;
    - `miri-slow`: that one alone, and the expensive one;
    - `everything`: the old behaviour.
  - The 8 regular jobs add `&& (github.event_name != 'workflow_dispatch' || inputs.jobs == 'everything')`.
  - To use it: Actions → CI → Run workflow, or `gh workflow run ci.yml -f jobs=heavy`.
- **Verified with a scratchpad checker**, `ci_truth.py`: PyYAML plus a tiny evaluator of each job's
  `if:`.
  - The push, pull-request and schedule job sets are identical before and after.
  - Manual runs went from all 10 jobs to 2 / 1 / 1 / 10 for the four choices.
  - 16/16 assertions pass, and the checker FAILS on the old file and on a one-job mutant.
  - It needed one fix first: a missing scenario crashed it instead of reporting FAIL.
- **Why `[skip ci]`:** the push paths are provably unchanged, and the changed path is the one the
  button runs. **GitHub has not yet run the new file.** The first button press is its first real
  test.

## 2026-09-17: Jesse made the testing repo PUBLIC ("previous limitations won't be there")

- **Verified, not assumed.** `gh repo view` reported `visibility: PUBLIC`. A secret scan of the 8
  commits since the first push found 0 hits. The whole history has ONE author e-mail (not printed).
  **The final public repo will be a DIFFERENT one**: never create or push to it.
- **GitHub docs, fetched 2026-09-17:**
  - Public-repo runners: Linux x64, Linux arm64 and Windows get **4 CPUs / 16 GB** (private: 2 / 8).
    macOS arm64 gets 3 (M1) / 7 GB either way.
  - "Use of the standard GitHub-hosted runners is free and unlimited on public repositories."
  - A public repo's schedule is auto-disabled after 60 days without activity.
- **Now exposed publicly in this testing repo:** the banned word in HISTORY (`0a58451`), the author
  e-mail, the CODEOWNERS placeholder. SECURITY.md (ENTRENCHED, not edited) still says "not publicly
  hosted, no disclosure inbox". All of these are recorded in HANDOFF §1.1 and REMAINING_WORK 7.8 as
  Jesse's decisions. They stand for the FINAL public repo.
- **"Do whatsoever are good" on the cost restrictions: all three were LIFTED in `51aab51`:**
  - The nightly is now opt-OUT. It runs every job, heavy included, unless `NIGHTLY` is `off`. The
    variable stays as an off-switch for any private copy.
  - The button's default is `everything`. `heavy`, `heavy-gates` and `miri-slow` stay as choices.
  - Docs-only commits no longer use `[skip ci]`.
  - KEPT: miri-slow and heavy-gates stay off per-push, because they take hours and Miri runs on one
    host thread.
  - Verified with `ci_truth.py`: 23/23 assertions pass. It fails on the old file and on a
    one-job-still-`on` mutant.
- **MY MISTAKE, and the lesson.** The commit message said "this commit does not say [skip ci]".
  GitHub skips a push run on that token ANYWHERE in the message, so the push started **0 runs**.
  **Never write the literal skip token in a message whose push should run CI.** Recovered by a manual
  run on `51aab51` (the button, `jobs=everything`): run **`35147900141`**, 15 jobs, including the
  first-ever heavy-gates and miri-slow runs.
- **Ping-pong risk, recorded BEFORE the run** (CROSS_PLATFORM §9):
  - With ≥4 hardware threads it asserts a speedup of ≥1.5× at 4 workers. Every earlier CI runner had
    fewer threads, so it printed NOT MEASURED.
  - Local runs pinned to 4 or 8 of 16 logical CPUs measured 0.65–1.14×; unpinned it measured 2.16×.
  - On Windows, `available_parallelism` ignores the affinity mask. The runtime sizes nothing by CPU
    count, but a pinned laptop is still not a 4-vCPU VM.
  - Run 35147900141 is the real measurement: it may fail on ubuntu, windows and arm64.
- **Sync after the push:** `master` `51aab51`, `rc/1.0.0-drill` `acc6f11` and `v1.0.0` `991ebab` all
  matched GitHub; 198 commits past v1.0.0.
- **SECURITY.md was updated at Jesse's word** ("update security.md. Do whatever are correct and
  good"). The file is ENTRENCHED, so his instruction is the authority. The change is recorded in
  `docs/design/ENTRENCHED_CHANGE_RECORD.md`, in that file's format: what, why, the approval quote, why
  it is not an RFC, what was verified before, the residue, and how to revert.
  - **The settings came FIRST, then the file named them.** GitHub private vulnerability reporting,
    secret scanning and push protection were all switched on, and each read back as enabled.
  - **§1** now routes reports through the Security tab → Report a vulnerability.
  - **Still PENDING-PUBLIC:** `security@` and the PGP key, kept for the final repo.
    `governance.rs` requires that marker to stay.
  - **§6** lists the two GitHub protections as LIVE on the testing repo.
  - **Dependabot alerts were deliberately NOT enabled.** The two pyo3 advisories ignored on
    reachability grounds would open alerts that could never be closed.
  - **Verified:** before the change, governance 11/11 and Survey impact 0 nodes. After it,
    governance 11/11, Survey 0 errors, doctor 17/17.
- **Ops lesson:** in Windows PowerShell 5.1, piping JSON to `gh api --input -` fails with HTTP 400
  "Problems parsing JSON". Write the JSON to a UTF-8 file without BOM and pass `--input <file>`.
- **Run `35147900141` (run 6) proved my ping-pong prediction WRONG.**
  - `criterion1_pingpong… ok` on ubuntu, arm64 and Windows (4 CPUs each, where the 1.5× assertion now
    applies) and on macOS (3 CPUs, where it is skipped). A pinned laptop was not a faithful model.
  - Every push job was green: 1,657 tests on macOS, ubuntu and arm64, and 1,647 on Windows, 0 failed.
    The public runners are faster: Windows 10.4 min (15–28 before), ubuntu 6.8, arm64 5.3, macOS 7.0.
  - **heavy-gates ran for the first time and PASSED** in 7.7 min: the 100k formatter gate, the LSP
    latency gate, and the wasm differential gate.
  - **miri-slow's first run: ALL THREE CANCELLED at the 240-minute budget, 0 UB.** Read by the
    06:25 IST one-shot wake-up (nonce `ccd99f269cd8`, consumed on firing).
    - broker: 124/150 finished; in flight was
      `path::tests::set_canonicalization_preserves_the_covered_region_on_generated_input`.
    - syntax: 104/131 finished; still inside
      `parser::tests::nesting_past_the_limit_is_refused_with_a_diagnostic_not_a_crash`, for over 3 h.
      Run 2 stalled in the same test.
    - check: 30/237 finished; in flight was `dir::tests::no_byte_flip_ever_verifies_with_altered_authority`.
    - Under Miri the harness runs tests one at a time in name order, so the first unfinished name is
      the one in flight. 4 CPUs changed little, because Miri runs on one host thread.
    - **Consequence for Jesse: with the nightly on by default, EVERY nightly will end CANCELLED**
      until REMAINING_WORK 5.6 shrinks those tests under `cfg!(miri)`. Whether miri-slow stays in the
      nightly meanwhile is HIS call; nothing was changed.
    - Push run `35152405983` on `c8fb0d4` passed. Recorded in `1fa3a76`, which was pushed on his
      "push it" (push run `35172849871`).
- **Standing permission (Jesse, 2026-09-17):** commit locally AND push every commit to this testing
  repo without asking; both stay in sync. See [[testing-repo-autopush]]. The final public repo keeps
  its gate.
  - Record a prediction's outcome whichever way it falls.
- **Pushed `2162796`** (the SECURITY.md change plus the run-6 record): 199 commits past v1.0.0, every
  ref matched GitHub, tree clean. Its push DID start CI, as run `35149509824`. Before committing, a
  guard checked the message for GitHub's skip tokens and found none.
- **Gate recorded (Jesse, 2026-09-17).** The four pre-public items are NOT to be changed yet. Before
  any step toward the final public repo, remind him and wait for his decision on each; there is no
  push there before that. Details: [[final-public-repo-gate]]. Written into HANDOFF §1, §1.1 (with
  the suggestions table) and §11.1, and into REMAINING_WORK 7.5.

This session's Bash tool had no Git Unix tools on its PATH (`git`/`sort`/`head` "not found"). Use
PowerShell.

See [[delululang-project]], [[head-chef-handoff]], [[delulu-remaining-work-inventory]].
