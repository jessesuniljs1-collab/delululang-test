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
  2026-11-01: *"Let routine be paused, I will resume it whenever I want."* Only the owner switches it on or off. He may also
  start one run by hand while it stays paused, as he did three times on 2026-11-02 (runs 17 to 19), so read the
  newest commits on `origin/master` before you work. It works on `master` directly and follows [`CLOUD_ROUTINE.md`](CLOUD_ROUTINE.md): one milestone per run, then a clean stop.
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
| Add a new `DL` diagnostic code casually. | There are 157. A new one needs a `D-V2-nn`, and several documents quote the count. |
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
   records; the last one used is **M1075**, so continue from M1076. Run one cargo test target at a time when you do this.
3. **Lint after the LAST `.rs` edit**, tests included: `cargo fmt --all`, then `bash scripts/clippy.sh -j 4`. That is
   fmt check plus `clippy --workspace --all-targets -D warnings`, and it writes a stamp the pre-commit hook demands.
4. **Test.** `cargo test --workspace --no-fail-fast -j 4`, or `scripts/suite.sh`. Freeze the tree while it runs, and read
   cargo's own exit code, never a pipeline's.
   - **The laptop has little memory**: run single targets there and let GitHub run the full suite (the owner,
     2026-10-19). Push the branch, dispatch `ci.yml` with `-f jobs=everything`, read the run to its end, and land only
     on green.
5. **Regenerate the map LAST**: `cargo run -p delulu-survey -- build`, then `-- check` and `-- findings`, which must show
   0 errors. A warning is for you to judge: on 2026-11-02 there were 6, each a path some prose names that is not in the
   tree. Dated records (the archive, the memory notes) are not held to today's counts.
   Then `cargo run -p delulu -- doctor --check`.
6. **Record:**
   - a `V2_LOG.md` entry (what, why, evidence);
   - `CHANGELOG.md` for anything a user sees;
   - a `D-V2-nn` for a decision (the last is **D-V2-208**);
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
| The owner's installed toolchain | `which delulu` and its date. On 2026-11-02 the laptop's `delulu` on `PATH` and its VS Code extension were from 2026-08-07, three months behind (RW 6.53). Reinstall only on the owner's word. |
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
  job took 3 h 11 min until on-demand routine run 19's fix (`04db8fb`); since then 2 h 01 min (`41416143488`),
  against a 4 h cap.
- **macOS:** unix-socket paths are limited to 104 bytes; a socket answers `EOPNOTSUPP`, not `ENXIO`; and the clock is
  microsecond-grained. **Windows:** 8.3 short names and junctions are other spellings of a path.
- **A security decision on an unnormalized spelling is walked past by another spelling.** That covers a relative path,
  `..`, an 8.3 name, a junction, case, or a dangling link. Ask what else spells the same thing.
- **A lock taken over after a timeout is a race, not a recovery.** Use the OS lock.
- **Append a log entry after the previous entry's LAST line**, and keep a table row on one line.

## 8. The memory, digested (the full files are in `docs/assistant-memory/`)

**Owner instructions in force** (newest first):
- **2026-11-12:** *"I will be using different models as Head Chef or Sous Chef or Agents for building delululang."* The
  model and the effort are his choice. The repository pins neither (`.claude/settings.json` sets no effort since that
  day). A routine run's effort is the `CLAUDE_CODE_EFFORT_LEVEL` line in its cloud environment
  (`docs/CLOUD_ROUTINE.md`, *Models and effort*). Read your own model and effort from the run, record them, and sign
  your commits with that model.
- **2026-11-05:** *"I allow u to suggest any changes that is against current constitution.md, license or any other .md
  file rules and regulations"*. A recommendation may cross any rule; it names the rule and its cost, and stays a
  suggestion. Entrenched files change only by the recorded route (`HANDOFF.md` §11.1).
- **2026-11-02:**
  - *"Still cannot write complete program like in java or python"*: "ready" is judged against an ordinary program too,
    not only against gates (RW 1.15, 2.33–2.40). Whether the launch waits for the language basics is his choice,
    asked and not yet answered.
  - The autosave task renewed for seven days, to 2026-11-09.
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
  - Sous-chef agents only where they add real value, one at a time preferred. Claude's are the model the owner names, else Sonnet 5.5 or
    Haiku 5.5 (2026-11-12).
    Judge an agent by its logs, never its summary, and re-run every claim against the current binary.
- **2026-07-24:** Apache-2.0 + `NOTICE` + `TRADEMARK.md` (D27).

**What the project has paid to learn** (each is a memory file; search its name):
- *"Ready" includes an ordinary program* (2026-11-02): write the programs users will write before calling anything
  ready, and separate *deliberate by design* from *simply not built* when explaining a gap.
- *What the owner runs is not what CI runs* (2026-11-02): his installed `delulu` can be months behind the tree.
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

## 10. Where the project stands (2026-11-05) — the newest sync-log entry supersedes this

- `master` holds #119 to #123 and on-demand routine runs 17 to 19's milestones, each landed on a green run (the reviews
  below are #124 to #128, also each landed on a green run):
  - INDEX-TARGET-1 (D-V2-205, RW 2.29): every walk of an assignment skipped its index, so a function the checker called
    pure printed through `xs[noisy(c)] = 9`;
  - DL0414 (D-V2-206, RW 2.30): a new warning for a `let` or `var` binding never read;
  - RENAME-TYPES-1 (RW 2.31): the editor's rename of a type or an effect left most of its uses behind;
  - DL0414's repair `prefix-underscore` (D-V2-207, RW 2.32): the laptop's draft `laptop/unread-repair`, finished. It
    renamed `x` to `_x` where `_x` already meant something in the scope and changed what the program printed
    (RENAME-CAPTURE-1); now it is offered only where no `_x` is met in the binding's scope, and the editor offers its
    edits (never preferred).
  - `http.put`, `http.patch` and `http.delete` and their `_with_secret` forms (D-V2-208, RW 2.12 closed, P9-05 complete,
    on-demand routine run 18): each judged by its method wherever a POST is, on both engines and in a guest; redirects by
    the Fetch standard's rule, and a method other than GET never sent to another origin.
  - the broker's Miri back under its cap (on-demand routine run 19): `miri-slow (delulu-broker)` ran 3 h 11 min of its
    240 on `41383380395`, 8,119 s of it in one test; path canonicalization's tests now do their work once, over the same
    inputs and laws.
  - #123, records: the everyday-program probe (RW 1.15, 2.33–2.40) and what touring every surface found (RW 6.50–6.53).
- `launch/2.0.0` is at `728a582`, **prepared, not launched**, with `LAUNCH-GATES=ok`. It holds `master` through `04db8fb`
  (D-V2-208 is additive). CHECKLIST-2.0 cites `1487163`'s runs, read green. `728a582`'s `everything` `41416143488` and
  release dry run `41416147296` were read green by on-demand routine run 19 (18 of 18; the broker's Miri 2 h 01 min). The
  launch itself (a fresh history in the public repository, the tag, the release) waits for the owner's word.
- **2.0's readiness is in question (the owner, 2026-11-02):** *"Still cannot write complete program like in java or
  python."* A probe of 39 everyday constructs: 14 ran, 25 were refused (RW 1.15; the gaps are RW 2.33–2.40). The head
  chef recommended holding the launch until the language basics land; **the owner's answer is pending.**
- **Two independent reviews (2026-11-04/05)**, both in `docs/review/`. Both are research only, and neither took a
  decision:
  - **Codex's** (#124), committed as received.
  - **Claude's**, from a Claude Code cloud session on `master` `0670e0b`. It audited Codex's review and re-ran all 102
    of its commands on Linux; every one reproduces. It found no containment or authority escape, and recommends:
    - CLI-first;
    - the compiler, LSP and VS Code client kept as they are;
    - OpenShell as an **optional backend**, with the microVM (PS-C) frozen;
    - the 2.0 launch held behind an *ordinary agent utility* bar.
  - Its findings are `REMAINING_WORK.md` rows 2.41–2.44, 3.25, 4.88–4.92, 6.54–6.66, 7.27 and 7.28, and RW §8 item 8
    gives its order. The OpenShell part is `V2_OPENSHELL_STUDY.md` §8.
  - Its owner decisions (the review's §13) are open: hold 2.0 for that bar; adopt the OpenShell verdict; whether a
    scope refusal is catchable (RW 2.42); the exit-code mapping; and RFCs or a recorded unfreeze for the core changes.
- **The AI-first compiler and execution-pipeline audit (2026-11-05)**, the same cloud session, in `docs/review/`:
  - **Measured each pipeline stage.** Parsing is a sixth of the front end. The existing machine form (DIR) is 28×
    larger than text. The real costs are Cranelift on every WASM run and `libpython` on every command.
  - **Recommends:** text stays canonical, with package-aware node edits; `--engine auto`; an authenticated precompiled
    cache, off by default; Python out of the main binary. It rejects AI-emitted machine code, a graph store and a new
    backend.
  - **Found seven defects,** each re-run before it was recorded: RW 4.93, 4.94, 5.7, 6.67–6.70, the first three in the
    DIR path. The cause of RW 6.58 was found too.
  - Its §14 holds suggestions that cross current rules, under the owner's permission above. None was acted on.
- **A third verification pass (2026-11-05)**, the same cloud session, with no sub-agents:
  - it replayed the audit's 71 probes (all identical) and re-witnessed every defect;
  - it read the PR's and `master`'s CI (both green, nothing to fix);
  - it found **RW 4.95 (HIGH)**: `p.get` never checks a plugin export's type against the one the host asks for, so a
    `! {Load, Read}` program wrote a file through a `! {Write}` export. `check`, `authority` and a package's DL1009
    ceiling all passed. The grant still bounded the write;
  - it also found RW 4.96 (programs load only Verified plugins) and RW 6.71 (`plugin verify` miscounts exports).
- **The hidden-complexity audit (2026-11-05)**, the same cloud session, no sub-agents, in `docs/review/`:
  - it followed an AI's whole task path, from discovery to outcome, on the same binary;
  - it disproved several suspected costs: pure tasks open no keys, plugins or audit files; nothing waits on a prompt
    without a terminal; an incremental query engine is not needed; MCP's process per call is cheap;
  - it found five new rows, each witnessed:
    - RW 6.72: the LSP is package-blind (false DL0301 on the shipped greeter);
    - RW 6.73: grants are refused after earlier effects;
    - RW 3.28: an unrunnable `.dwx` for `fn main()`;
    - RW 3.29: DL0905 becomes SIGABRT under an address-space cap;
    - RW 6.74: MCP ignores cancellation;
  - it amended RW 3.26, 6.60, 6.62 and 6.69. RW 4.95 still holds.
  - **its second pass the same day**, with three Sonnet 5.5 sub-agents whose every finding the head chef re-ran:
    - corrected its own sandbox claim: the `hostile-agent` profile triggers RW 3.29;
    - corrected its peak-memory figures;
    - found RW 6.75: a sandboxed fault reaches neither the envelope nor the report;
    - found fourteen code-review defects, grouped as RW 4.97 (`verify-sig --kye` fails open) and 6.76–6.81;
    - read `master`'s red Windows run: RW 7.25's flaky monitor test. Its reading there (narrowed to a 1.9 s broker
      queue) was corrected on the laptop the same evening: the evidence excludes no cause, and neither red's cause is
      established (RW 7.25);
    - updated README and the editor, getting-started and questions pages for RW 4.95 and 6.72.
    - landed as #128, `master` at `be40a92`, on a green run (`42020534572`, 14 of 14). `master`'s Windows job was
      re-run once and passed, and the scheduled run `42009076387` was 18 of 18 green, its three `miri-slow` jobs included.
- **Open / next, in order:** as the newest `docs/CLOUD_SYNC_LOG.md` entry says. In short:
  - the owner's answers (the language basics, and the reviews' decisions above);
  - cite `41416143488` and `41416147296` in CHECKLIST-2.0 at the next merge;
  - ~~RW 4.95 with 4.96 and 4.97~~ **closed 2026-11-06 by on-demand routine run 20** (D-V2-210 to D-V2-212): `p.get` checks
    the export against the type the program asks for (R-Get), in the host and inside a plugin; a `Plugin[Contained]`
    request is refused at `load` as one; the six commands that read their own options refuse one they do not know;
  - **RW 7.29 first** (found the same run, reading RW 7.25's repeat runs): `a_watch_held_outside_the_daemon_lets_its_dead_man_fire`
    red in two shapes; shape 1's fail-open (a chain it could not read taken for *someone else revoked it*) closed the same
    day (D-V2-213), and the test now prints `broker.log` at each failure — read the next red's before naming a cause;
  - RW 4.98 is closed (D-V2-209, amended after an independent review): a beat of this version read whole and stamped
    before a judgment counts, by its key, for the period it bought; counter-tests for incomplete connections, wrong keys,
    another protocol version and other traffic; the broker's Miri green on it (`42093191962`). RW 7.25's cause is
    still not known. The six green runs of 2026-11-05 ran the test alone; since 2026-11-06 it is re-run as the push
    run runs it, the whole `monitor_cli` target many times (`witness.yml` `repeat` and `stress`), and both sides' beat
    timing is on one wall clock, with the monitor's turns step by step, so one red names its path;
  - ~~the package-blind fast paths, RW 6.72 with 6.69 (one cause)~~ — closed 2026-11-06 by the laptop (D-V2-214): the
    language server and `edit` check a module of a package as the package (`check_package_overlaid`); the other LSP
    providers' half is RW 6.82. ~~RW 3.28~~ closed the same day (D-V2-215: a wrapper calls any `main` the checker
    accepts in the WASM host's shape), and ~~RW 3.29~~ too (D-V2-216: the interpreter's stack walks a ladder and its
    depth bound follows the stack it got, or half a sandbox guest's memory ceiling), and ~~RW 6.73~~ (D-V2-217: a
    refusal names every grant the run lacks, `missing_grants`; `run --preflight` refuses before anything runs), and
    ~~RW 6.74~~ (D-V2-218: MCP calls on a pool of four, a cancel kills the call and sends nothing, `outputSchema`
    from `delulu schema`), and ~~RW 6.75~~ (D-V2-219: a sandboxed guest's fault reaches the envelope and the report,
    carried in its goodbye as the guest's word; its abnormal-end half is RW 6.84). The audit's §7 P1 items are done
    except P1-6's answer bound (RW 6.62) and the rest of P1-7 (RW 6.60, 6.68);
  - **NOW, at the owner's word (2026-11-06, D-V2-220): the language basics first.** *"Build basics first for DeluluLang
    … After each build, write and run programs and softwares of varying complexities and test and verify and fix it."*
    - The order: ~~compound assignment (2.34)~~ (done 2026-11-08 by on-demand routine run 21, D-V2-221; its programs
      found RW 2.45, 2.46 and 3.32, and RW 3.31, fixed) → ~~RW 2.46~~ (done 2026-11-08 by the laptop, D-V2-222, with
      RW 2.47) → ~~math and the checked forms (2.38)~~ (done 2026-11-09 by on-demand routine run 22, D-V2-223; its
      programs found two formatter defects, fixed, and RW 2.48 and 2.49) → ~~ranges (2.36)~~ (done 2026-11-10 by on-demand
      routine run 23, D-V2-224, with every free builtin's arguments read; its programs found WASM's missing `print`,
      `len(x)` and `push`, fixed, and RW 2.50, 2.51 and 6.85) → ~~strings (2.37)~~ (done 2026-11-10 by on-demand routine
      run 24, D-V2-225: f-strings, `s[i]`, the everyday `Str` table and `to_fixed`, with RW 2.48; its programs found
      WASM's missing `match` on a `Str`, fixed, and RW 6.86) →
      bitwise (2.35) → methods on a program's own types (2.33) → tuples, sets, keyed sorting, comprehensions (2.39) →
      argv, environment, exit code, sleep (2.40) → JSON (2.41) → interfaces (2.33, 2.9) → RW 3.24.
    - Each is built as surface syntax and prelude over the core, so no RFC period applies; one that would change
      `DELULU_CORE.md` gets its own decision.
    - After each: programs of varying size, run on both engines, and what they find fixed;
  - then the outcome contracts: RW 6.83, 6.77, 6.84, 6.76 and 6.78–6.81 (the audit's §11.7);
  - the DIR and review-surface defects (RW 4.93, 4.94, 5.7, 6.67), then RW 6.68;
  - the answer contracts: RW 6.50–6.52, 6.54–6.60 and 4.88, each witnessed red first;
  - the OpenShell pin bump (RW 7.28);
  - the launch, only on the owner's word.
- **Counters:**
  - the last decision is D-V2-223;
  - the last mutant is M1219 (with M1206b, a re-run of M1206 on clippy's form of its line);
  - there are 157 DL codes and 399 reference anchors;
  - pull requests are numbered on GitHub, and #144 was the last merged (by fast-forward, the laptop, 2026-11-08).

## 11. Keep this file true

Change this file whenever something in it stops being true: a rule, the routine's schedule or state, a counter, a
command. Do it in the same commit as the change. It is listed in `docs/REPOSITORY_STRUCTURE.md` §5, and `AGENTS.md` points
every AI here first.
