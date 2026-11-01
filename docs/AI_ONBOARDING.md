# Before you start — what any AI working on DeluluLang must know

**Who this is for.** Any AI the owner opens on this repository — Claude Code (a laptop terminal, a cloud session, the
scheduled routine) or another assistant (Codex, Gemini, Cursor, anything else). Written 2026-11-01 at the owner's request:
*"There maybe another AI also working on delululang, Update the ur memory and docs. Give me doc containing everything
another AI should know before that AI start working like imp things to know and things in ur memory. I don't want
everything to be mess. So I will be running either of u or another ai at time, Not both of u together"*.

Read this file whole before your first edit. Then read `AGENTS.md`, and `HANDOFF.md` §0, §1 and §11. **`HANDOFF.md` is the
authority**: where it and this file disagree, it wins, and you fix this file in the same change. Everything here is a
digest. The full texts are in the files each line names, and the head chef's whole memory is in
[`docs/assistant-memory/`](assistant-memory/).

---

## 1. One AI at a time, handed over through git

- **The owner runs ONE AI at a time** (2026-11-01). It is either Claude or another AI, never both at once. You are alone in
  the repository while you work, and the next AI starts from what you pushed and wrote down. Nothing else reaches it: not
  your chat, your tool's private memory, your scratch files, or an uncommitted tree.
- **The scheduled routine is an AI too.** It is `trig_01HFG6usfzmDAPQigiDu9DLq`: Claude Opus 5.5, cron `25 */4 * * *`
  UTC, which is 01:55, 05:55, 09:55, 13:55, 17:55 and 21:55 IST (set 2026-11-01). It is **paused**, by the owner's word of
  2026-11-01: *"Let routine be paused, I will resume it whenever I want."* Only the owner switches it on or off. It works
  on `master` directly and follows [`CLOUD_ROUTINE.md`](CLOUD_ROUTINE.md): one milestone per run, then a clean stop.
- **If someone else's commit lands while you work**, stop editing. That means a new commit on `origin/master`, or on your
  branch, that you did not make. Tell the owner, and re-sync before going on. Two AIs at once is exactly the mess he
  asked to avoid.
- **Name your branches after yourself.** Claude on the laptop uses `laptop/<topic>`, Claude in the cloud `claude/<topic>`.
  Another AI uses its own name, such as `codex/<topic>` or `gemini/<topic>`. Never commit to another AI's branch. To take
  one over, say so in your sync-log entry.

## 2. Start of a session, in order

1. **Sync with GitHub first** (the owner, 2026-10-17):
   - `git fetch origin --prune`, `git status -sb`, and `git log --oneline HEAD..origin/master`.
   - Fast-forward `master`. Never merge with a merge commit.
   - List the branches (`git branch -r`) and `git stash list`. Read each branch ahead of `master` and its CI run.
   - `origin/autosave/laptop` and `origin/autosave/dogfood` are snapshots a Windows task pushes. Read them as drafts;
     never merge them.
2. **Read the inbox.** The newest entries of [`CLOUD_SYNC_LOG.md`](CLOUD_SYNC_LOG.md) end with **Open / next, in order**:
   that is your work queue. Then read [`REMAINING_WORK.md`](REMAINING_WORK.md) (every open row, each checked against the
   binary), [`DELULULANG_V2/V2_PHASE_STATUS.md`](DELULULANG_V2/V2_PHASE_STATUS.md), and the newest entry of
   [`DELULULANG_V2/V2_LOG.md`](DELULULANG_V2/V2_LOG.md).
3. **Health:** `cargo run -p delulu-survey -- check` must say `ok: the Survey matches the tree`.
   `cargo run -p delulu -- doctor --check` must say `ok: N check(s) passed`. Anything else is the first thing to fix or
   record.
4. **Once per clone:** `git config core.hooksPath scripts/hooks`. The commit identity is `jessesuniljs1-collab` with e-mail
   `227307678+jessesuniljs1-collab@users.noreply.github.com`. Never commit with another address.

## 3. End of a session, every time, even a short one

1. Everything is committed and pushed. Nothing is left uncommitted, stashed or only local, and no background job is
   running.
2. **A sync-log entry** is appended after the last line of `docs/CLOUD_SYNC_LOG.md`, in the template that file gives. It
   holds:
   - who you are and the model that actually ran;
   - the branch and commits;
   - every file added, modified, deleted or renamed (`git diff --name-status <base>..HEAD`);
   - what you verified, and every CI run by id, marked read green, read red, or not read;
   - the **Open / next** list, in order, written so the next AI can start from it cold.
3. Run `survey build`, `check` and `findings` (0 errors) and `doctor --check` **after the last edit**, then commit the map
   (`docs/survey/`). The pre-commit hook refuses a stale map.
4. Durable lessons go into `HANDOFF.md` §11 and `docs/assistant-memory/` (one fact per file, plus a line in its
   `MEMORY.md`), not only into your own tool's memory.
5. Land on `master` only by **fast-forward, on a green CI run you have read**: `git push origin <sha>:master`. Then delete
   the merged branch from `origin`, after `git merge-base --is-ancestor <branch> origin/master` says it is in.

## 4. Hard rules — never break these

| Never | Why / where |
| --- | --- |
| Push anywhere but `origin` = `github.com/jessesuniljs1-collab/delululang-test` (public, a TESTING repository). | The owner, 2026-10-17 (night): *"Only push to and use test repo until I tell u to change. Do not change repo, unless told by me. Always push only to my repo"*. Nothing goes to `github.com/jessesuniljs1-collab/DeluluLang` (the future public home, empty) until he says so in words. |
| Force-push, rewrite pushed history, push a tag, or create a release. | Documents cite commit hashes everywhere. A `v*` tag publishes (D-NE-7). The launch (steps 4–10 of the release procedure) waits for the owner's word. |
| Put CI's skip token (the bracketed word *skip* followed by *ci*) in a commit message. | It silently skips the push run, so nothing is tested. |
| Write the word the owner banned: *graph* and *ify* as one word. | It may appear in no file, commit, branch or product surface. Check every diff with `grep -ci 'graph''ify'` before committing. |
| Delete a `.md` file. | Move a superseded document to `docs/archive/`. |
| Skip hooks (`--no-verify`), weaken a test, or "fix" a red test by loosening it. | A gate that cannot fail is not a gate. A red test is information. |
| Claim what did not run, or invent a result, contact, key or compliance claim. | "Prepared", "dispatched" and "green" are different words. A CI run counts only once its result is READ and recorded. Failed runs are recorded, not tidied away. |
| Change what Authority and the Guard *mean*. | Harden, never redefine: closing a hole is welcome, redefining is the owner's. |
| Edit an entrenched file casually. | Entrenched files: `docs/design/` CONSTITUTION, DELULU_CORE, STABILITY, SOUNDNESS_AUDIT; `SECURITY.md`; `rfcs/`; `docs/security/`; `crates/delulu-check/tests/laundering.rs`; `crates/delulu-conform/`; `tests/conformance/witnesses.toml`; CODEOWNERS "entrenched"; the licence. Each change needs a `D-V2-nn` decision plus an `ENTRENCHED_CHANGE_RECORD.md` entry. Ask `cargo run -p delulu-survey -- query doc:<path>` before editing under `docs/design/`. |
| Add a new `DL` diagnostic code casually. | There are 155. A new one needs a `D-V2-nn`, and several documents quote the count. |
| Commit or ship a built Linux kernel or microVM image. | D-NE-27: GPL obligations first. |
| Change the licence (Apache-2.0 + `NOTICE` + `TRADEMARK.md`, ruling D27). | The owner's intent of 2026-07-24 is the evidence any change must serve. |

**Decisions.** The owner's commission of 2026-10-17 ([`design/DeluluLang_Autonomous_Development_Prompt.md`](design/DeluluLang_Autonomous_Development_Prompt.md))
delegates every outstanding DeluluLang decision to the working AI, owner-reserved ones included. Each is written in
`docs/DELULULANG_V2/V2_DECISION_LOG.md` as `D-V2-nn — … — TAKEN (head chef, <date>, under the owner's delegation)`,
never as the owner's ruling. If you are not Claude, write your own name in place of "head chef". The commission's limits
stay in force: no invented contacts, keys, results or compliance claims; keep this development repository, its history
and a backup; no microVM image until its GPL obligations are met; never weaken a test.

## 5. How every change is made

1. **Ask the Survey first.** `cargo run -p delulu-survey -- impact <id>` shows what breaks, `affected-by <id>` what it
   rests on, and `query <id>` both directions, including ENTRENCHED. IDs look like `crate:delulu-check`,
   `mod:crates/delulu/src/guest.rs`, `test:crates/delulu/tests/guest_cli.rs`, `doc:README.md` and `code:DL0501`.
2. **Witness the defect failing before you fix it.** Stash the fix and watch the new test go red. Then **falsify the test
   with mutants**: reintroduce the defect, or break the fix, and watch it go red. Mutants are numbered (`M<n>`) in the
   records; the last one used is **M1007**, so continue from M1008. Run one cargo test target at a time when you do this.
3. **Lint after the LAST `.rs` edit**, tests included: `cargo fmt --all`, then `bash scripts/clippy.sh -j 4`. That is
   fmt check plus `clippy --workspace --all-targets -D warnings`, and it writes a stamp the pre-commit hook demands.
4. **Test.** `cargo test --workspace --no-fail-fast -j 4`, or `scripts/suite.sh`. Freeze the tree while it runs, and read
   cargo's own exit code, never a pipeline's.
   - **The laptop has little memory**: run single targets there and let GitHub run the full suite (the owner,
     2026-10-19). Push the branch, dispatch `ci.yml` with `-f jobs=everything`, read the run to its end, and land only
     on green.
5. **Regenerate the map LAST**: `cargo run -p delulu-survey -- build`, then `-- check` and `-- findings`, which must show
   0 errors. A warning is for you to judge: on 2026-11-01 there were 16, nearly all of them dated memory notes quoting
   what was true when they were written. Then `cargo run -p delulu -- doctor --check`.
6. **Record:**
   - a `V2_LOG.md` entry (what, why, evidence);
   - `CHANGELOG.md` for anything a user sees;
   - a `D-V2-nn` for a decision (the last is **D-V2-203**);
   - the row in `REMAINING_WORK.md`. A table row is **one line**: never wrap it, a gate checks;
   - `V2_PHASE_STATUS.md`;
   - the sync-log entry;
   - `HANDOFF.md` §11 for a durable lesson;
   - a row in `docs/REPOSITORY_STRUCTURE.md` §5 for every new markdown file. A test enforces it.
7. **Commit** with the message written to a file and `git commit -F <file>`. Heredocs have mangled backslashes and
   non-ASCII. End the message with the model that actually ran and, if your tool has one, a session link. Claude writes
   `Co-Authored-By: Claude <model> <noreply@anthropic.com>` and `Claude-Session: …`; another AI writes its own honest
   equivalent. **Push, read the CI run, record it.**

## 6. Commands (the owner's laptop: Windows 11, Git Bash and PowerShell)

| What | Command |
| --- | --- |
| GitHub CLI | `"/c/Program Files/GitHub CLI/gh.exe"`, repository `jessesuniljs1-collab/delululang-test`. The network is flaky, so wrap calls in a retry loop. |
| Runs | `gh run list -R jessesuniljs1-collab/delululang-test`; `gh run view <id> --json jobs`; one job's log with `gh api --allow-escape-sequences repos/jessesuniljs1-collab/delululang-test/actions/jobs/<job-id>/logs` |
| Dispatch | `gh workflow run ci.yml --ref <branch> -f jobs=everything` (or `heavy`, `heavy-gates`, `miri-slow`); `release.yml` (a dry run without a tag); `witness.yml -f os=macos-latest -f target=<test file> -f filter=<name>` for one test on one runner |
| A run's verdicts | `python3 scripts/ci-annotations.py <run-id>…` and `python3 scripts/ci-log-summary.py` |
| Interpreter profile | `scripts/interp-profile.sh [--base REF] [--profile NAME]` (RW 3.24's harness) |
| The launch branch | `launch/2.0.0` lives in the worktree `D:\nelan\dl-launch`. Bring it current with `git merge --no-ff --no-commit origin/master`, then `git checkout --theirs docs/survey/`, `python3 -I scripts/launch-changelog.py`, `python3 -I scripts/launch-criterion-2.py '<who>'` and `bash scripts/launch-gates.sh` (it must print `LAUNCH-GATES=ok`) |
| Other OSes | `scripts/check-other-os.sh` lints macOS and Windows code from Linux. `witness.yml` runs it for real. |
| WSL | Ubuntu with `/dev/kvm`. The microVM toolchain is at `~/microvm-image-a` and `~/bin/firecracker`. Builds there can be killed for lack of memory, so don't restart a killed job on your own. |

## 7. Traps that cost real time

- **A Bash heredoc that carries Python or Rust mangles backslashes.** It halves a doubled backslash and turns a
  `\n` into a real newline. Write `.py` and `.rs` edits with a file-writing tool, never `cat <<EOF`.
- **PowerShell `Set-Content` writes the ANSI code page, and older redirections write UTF-16.** Pass `-Encoding utf8`.
- **CRLF.** Git stores LF. A map or snapshot built from a CRLF working tree passes locally and fails on every runner
  (`git ls-files --eol`; check from a clean clone).
- **Stale binaries fake regressions.** Rebuild after restoring a mutant or merging. "All mutants survived" usually means
  the harness filter hit the wrong target.
- **A wall-clock test that fails only on CI may be measuring the runner.** Reproduce it by starving the machine, never by
  lengthening sleeps, and give the test a control measured on the same machine. On Windows, `sleep(ZERO)` yields a time
  slice.
- **Miri is about 100× slower.** Shrink a test under `cfg!(miri)` only where its size is not the witness. The broker's Miri
  job takes about 3 h 05 min, against a 4 h cap.
- **macOS:** unix-socket paths are limited to 104 bytes; a socket answers `EOPNOTSUPP`, not `ENXIO`; and the clock is
  microsecond-grained. **Windows:** 8.3 short names and junctions are other spellings of a path.
- **A security decision on an unnormalized spelling is walked past by another spelling.** That covers a relative path,
  `..`, an 8.3 name, a junction, case, or a dangling link. Ask what else spells the same thing.
- **A lock taken over after a timeout is a race, not a recovery.** Use the OS lock.
- **Append a log entry after the previous entry's LAST line**, and keep a table row on one line.

## 8. The memory, digested (the full files are in `docs/assistant-memory/`)

**Owner instructions in force** (newest first):
- **2026-11-01:**
  - One AI at a time, handed over through git (§1).
  - The routine is set to every 4 hours but paused until he resumes it.
  - This file is the brief for any AI.
- **2026-10-26:**
  - A routine run is ONE milestone, then a clean stop.
  - *"Continue cooking … till the limit hits"* is for his terminal and interactive cloud sessions: they go milestone after
    milestone, each landed and recorded before the next.
  - Branches are cleaned as work lands.
- **2026-10-24:**
  - When agents test DeluluLang, they hold no authority: the head chef does (a strict broker, leases per condition, the
    `dl` wrapper, a pre-tool gate).
  - Everything agents do is saved to storage as they go.
- **2026-10-20: keep the autosave task on.** A Windows task, `DeluluLang-autosave`, pushes branches that are ahead and
  snapshots uncommitted work to `autosave/laptop`. Expect it to push your branch too.
- **2026-10-19:** when the laptop cannot run or finish the tests, GitHub runs them, automatically, without stopping to ask.
- **2026-10-17:**
  - The commission: take DeluluLang to a public release; every decision is delegated, within its limits (§4).
  - Only the test repository.
  - Start every session by syncing with GitHub.
  - Development never waits for the laptop: cloud sessions and the routine continue it.
  - The owner is **Jesse; he/his**; he lives in IST.
- **2026-09-28:**
  - Run and check everything with the Survey and `doctor`.
  - Run heavy jobs (the OS matrix, Miri, `heavy-gates`, the release dry run) on GitHub.
- **2026-09-17:**
  - Push every verified commit to the test repository at once, without asking.
  - The banned word.
  - Sous-chef agents only where they add real value, one at a time preferred. Claude's are Sonnet 5.5 or Haiku 5.5.
    Judge an agent by its logs, never its summary, and re-run every claim against the current binary.
- **2026-07-24:** Apache-2.0 + `NOTICE` + `TRADEMARK.md` (D27).

**What the project has paid to learn** (each is a memory file; search its name):
- *Discovery over checklist:* a green checklist is not proof. Keep asking which assumption has not been attacked yet.
- *Two engines that agree prove only that they agree:* LATE-LET-1, LOCAL-CALLEE-1 and CTOR-SHADOW-1 were runtime name
  lookups that differed from the checker, and they made effect rows escapable. Mirror the checker's lookup order (local,
  then constructor, then function, then prelude), and assert the checker's value, not the other engine's.
- *The effect row was escapable once* (P16), the worst defect in the project's history: `authority` said "provably pure"
  and the program printed at run time. Any change that could let a run do what its check did not see is the top
  priority.
- *Reproduce the shape, not the description:* a near-miss reproduction of a recorded defect reads exactly like a fix.
  Confirm the reproduction produces the baseline effect before you trust its refusal.
- *Skip branches:* security rules die in the `else { continue }` branch. Write the "what if the checker couldn't tell"
  case before claiming a rule holds.
- *A static verdict that skips a dimension:* list every check made at use time before saying a static answer is
  complete.
- *A hash that excludes a section binds nothing in it.* List what every integrity check leaves out.
- *A gate that records before the check it trusts* records allows for calls that are later refused.
- *A gate counts what it reads,* and *a gate runs on every caller:* make a gate fail when it did not run, and run it on
  every path that uses it.
- *A guarantee witnessed through one command* may be skipped by six others. Drive the witness through every command that
  reaches what it guards.
- *A thread-local setting reaches one thread:* test a `thread_local!` knob from every thread that runs program code.
- *Call depth is not data depth:* use worklists for display, equality and reify, and test them on loop-built
  structures.
- *A shared type walked as a tree is exponential:* follow each binding once.
- *A latency may be a deadline:* grow the input past the nearest deadline before you grade a cost.
- *A field fed by a global:* look for a `static` behind every "per-instance" number.
- *A benchmark checks its answer:* time only a lane that produced the reference output.
- *An interpreter's cut is judged by its instruction count* (callgrind), beside an interleaved timing.
- *A check that passes with a warning:* Lean's `sorry` exits 0. A crate's tests alone unify features differently from
  the workspace build.
- *A pinned refusal names its reason:* read a refusing test's comment before making the refused thing compile.
- *Core-regression rule:* after a tooling change, diff the core's own output against a pre-change binary.
- *The clean-checkout lesson:* verify gates from a fresh clone. CRLF files and git-ignored files fooled every local run.
- *Model attribution honesty:* name the model that actually ran, and never rewrite history to hide a wrong signature.

## 9. What DeluluLang is, and where its parts are

A statically typed, authority-and-effect-typed language: **a program can do nothing except what it was explicitly
handed.**
- Effects are in the type, capabilities are values, and authority is computed from the code.
- A sandboxed program runs as a guest that holds no authority. The host performs every effect under grants, a lease and
  the Guard.
- There are two engines, a tree-walking interpreter (`delulu-runtime`) and WASM (`delulu-wasm`). They must agree with each
  other *and with the checker*.

It is a Rust workspace, with Rust pinned at 1.96.1 by `rust-toolchain.toml`. Its members:

| Crate | Holds |
| --- | --- |
| `delulu` | The CLI and its tests, plus the broker daemon (`brokerd.rs`), the sandbox, guest, microVM, `doctor`, the LSP and MCP |
| `delulu-syntax`, `delulu-check` | Parsing and the checker |
| `delulu-runtime`, `delulu-wasm` | The two engines |
| `delulu-broker` | The custody core: the ⊑ attenuation lattice, the grant tree, revocation, the hash-chained audit log, lease tokens. It has no transport. |
| `delulu-diag` | Diagnostics, `DLnnnn` |
| `delulu-atlas` | Maps a *program* |
| `delulu-survey` | Maps the *repository* |
| `delulu-conform` | Conformance witnesses (entrenched) |
| `delulu-fuzz` | The differential fuzzer: a run's trace must be a subset of the statically computed effect row |
| `delulu-measure`, `delulu-registry` | The published measurements; the package registry |

**Documents:**
- `HANDOFF.md` is the full briefing.
- `docs/DELULULANG_V2/` is the one active source: the phase ledger, `V2_LOG.md` and `V2_DECISION_LOG.md`.
- `docs/REMAINING_WORK.md` is every open gap.
- `docs/CLOUD_ROUTINE.md` is the routine's loop and work order (its step 4).
- `docs/CLOUD_SYNC_LOG.md` is the session ledger and the inbox.
- `docs/design/` holds the design corpus, partly entrenched.
- `docs/REPOSITORY_STRUCTURE.md` lists every file and what it is for.
- `docs/for-agents.md` is different: it is for agents that *use* DeluluLang, not for developers of it.

## 10. Where the project stands (2026-11-01, 06:30 UTC) — the newest sync-log entry supersedes this

- `master` is at `d8dcc3b` (on-demand routine run 16's close). Its push run on that commit had not been read when this
  was written.
- `launch/2.0.0` is at `1548766`, **prepared, not launched**, with `LAUNCH-GATES=ok`. CHECKLIST-2.0 is the release gate.
  The launch itself (a fresh history in the public repository, the tag, the release) waits for the owner's word.
- **Open / next, in order** (from run 16's entry):
  1. Read `master`'s push run.
  2. Bring `launch/2.0.0` current. RW 6.49 (#110: an unreadable flag value now exits 2) is a *Breaking* clause beside RW
     6.48's. RW 3.24's cuts move no surface. Then the launch gates, and the `everything` run and the release dry run,
     dispatched and read.
  3. RW 3.24, structurally: variables resolved to slots before the run, mirroring every scope the interpreter makes.
  4. RW 2.26 is decided (D-V2-203, `laptop/ctor-binding`): a pattern binding spelled like a constructor of the
     matched type is refused with DL0302. It is a *Breaking* clause for the next launch fold.
  5. Watch the broker's Miri time.
  6. The launch, only on the owner's word.
- **Counters:**
  - the last decision is D-V2-203;
  - the last mutant is M1007;
  - there are 155 DL codes;
  - pull requests are numbered on GitHub, and #110 was the last landed.

## 11. Keep this file true

Change this file whenever something in it stops being true: a rule, the routine's schedule or state, a counter, a
command. Do it in the same commit as the change. It is listed in `docs/REPOSITORY_STRUCTURE.md` §5, and `AGENTS.md` points
every AI here first.
