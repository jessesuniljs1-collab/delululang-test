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

### 2026-09-28 — routine run 1: PS-D and P7 complete; ADAPTER-SPELL-1, ATTEST-FIFO-1; RW 7.3, 7.4; P8 designed
- Session: `https://claude.ai/code/session_013AZJ6RqYq59CeMgkvLV1BM`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there, `docs/CLOUD_ROUTINE.md`)   Pull request: none   Merged: n/a
- Base: `5bb39bc`
- Commits: (1) `9ac5477` actors_pingpong: judge criterion 1 against a control on a free machine (D-V2-47);
  (2) `fef8ccd` PS-D-02: the attestation seam — --require-attestation, sandbox attest (D-V2-48);
  (3) `ed74683` Loop engineering: CI read through MCP, cargo fetch, the ping-pong verdict printed by CI;
  (4) `b7abcfb` P7: Delulu Core v0.3 — progress-or-fault and the higher-order primitive (D-V2-49);
  (5) `81c644f` ADAPTER-SPELL-1: a hardware driver is verified and started as one file (D-V2-50); PS-D complete;
  (6) `0a7b881` ATTEST-FIFO-1: the attestation is read only from a regular file; P7 complete;
  (7) `e430c2a` P8 designed (D-V2-51): the control program in a guest, then a Verified-class adapter;
  (8) `17d4623` RW 7.3: a manual container workflow — build the Dockerfile and the devcontainer on GitHub;
  (9) `f5b5985` RW 7.4: the editor's end-to-end test runs on Linux (a POSIX branch), and passed;
  (10) `8686479` RW 7.3: the Dockerfile's first build failed (embedded files not copied) — fixed; the devcontainer built;
  (11) `3fd53cd` What CI's runners are: SMT on x86, measured on arm64 (D-V2-47 §6); loop lessons;
  (12) `f4c9937` RW 7.3 closed: the fixed Dockerfile's image built, ran a granted program, refused an ungranted one;
  (13) `RW 7.4's next: an editor-e2e workflow (by hand first) — the real VS Code on a Linux runner`
- Files and folders:
  M `crates/delulu-runtime/tests/actors_pingpong.rs` — (1) the control, the quiet-machine rule, the share-of-machine bar; (3) its verdict file
  A `crates/delulu/src/attest.rs` — (2) the statement, its canonical bytes, the verifier, the reference attester
  A `crates/delulu/tests/sandbox_attest_cli.rs` — (2) five end-to-end tests
  M `crates/delulu/src/guest.rs` — (2) the pinned key on the external backend; nonce and path to the launcher; verify before the hello
  M `crates/delulu/src/cli.rs` — (2) `--require-attestation`; the help lines; `own_words` (nothing after `--` is delulu's)
  M `crates/delulu/src/main.rs`, `run_cmd.rs`, `sandbox.rs`, `schema.rs` — (2) the module; the refusal without `--sandbox`; the verb; `sandbox.attestation`
  M `CHANGELOG.md`, `HANDOFF.md`, `docs/DEPLOYMENT.md`, `docs/for-agents.md`, `docs/REMAINING_WORK.md` (4.24) — (2)
  M `docs/DELULULANG_V2/V2_DECISION_LOG.md` — D-V2-47 (1), D-V2-48 (2)
  M `docs/DELULULANG_V2/V2_LOG.md` — this run's entry (1)(2)
  M `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `V2_IMPLEMENTATION_ROADMAP.md`, `V2_SECURITY_MODEL.md` — (2) PS-D-02 built
  M `.github/workflows/ci.yml` — (3) the test and arm64 jobs print `actors_pingpong`'s verdict after the suite
  M `docs/CLOUD_ROUTINE.md`, `CLAUDE.md`, `AGENTS.md` — (3) CI through the GitHub MCP tools (no `gh` in the VM),
    `cargo fetch --locked` before the suite, `test:` Survey ids, a red already fixed by a newer commit
  M `docs/assistant-memory/cloud-period-2026-09-28.md` — (3) what routine run 1 learned
  M `docs/design/DELULU_CORE.md` — (4) ENTRENCHED: v0.3 — `fault(c)`, `E-Refuse`, `E-Fault`, progress-or-fault; `hop`, `T-HOp`, `E-HOp`
  M `docs/design/ENTRENCHED_CHANGE_RECORD.md` — (4) the record for that edit, under the owner's delegation
  M `docs/design/PROOF_CAMPAIGN.md`, `docs/MATHEMATICS.md` — (4) P17-T1/T2 noted as repaired on paper
  M `docs/REMAINING_WORK.md` — (4) 5.2 and 5.3 closed; `CHANGELOG.md`, `V2_PHASE_STATUS.md`, `V2_IMPLEMENTATION_ROADMAP.md` — (4)
  M `crates/delulu/src/cli.rs` (`resolve_driver`, the interpreter hint), `crates/delulu/src/run_cmd.rs` — (5) ADAPTER-SPELL-1
  M `crates/delulu/tests/hw_adapter_cli.rs` — (5) the witness test; one assertion updated to the resolved interpreter
  M `HANDOFF.md` §11.4, `docs/DELULULANG_V2/V2_SECURITY_MODEL.md`, `docs/QUESTIONS.md`, `docs/REMAINING_WORK.md` 4.7 — (5)
    the stale "no signature check" corrected; the residual named; `V2_PHASE_STATUS.md` row 13 — (5) PS-D complete
  M `crates/delulu/src/attest.rs`, `crates/delulu/tests/sandbox_attest_cli.rs` — (6) ATTEST-FIFO-1 and its test
  M `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 12), `docs/REMAINING_WORK.md` (5.6), `HANDOFF.md`, `README.md` — (6) P7 complete
  M `.github/workflows/ci.yml`, `docs/CLOUD_ROUTINE.md` — (6) the verdict step last in each test job, a notice annotation; the log tool's 5,000-line cap
  A `docs/DELULULANG_V2/V2_P8_DESIGN.md` — (7) P8's design; M `docs/REPOSITORY_STRUCTURE.md` §5.11 (its row),
    `V2_DECISION_LOG.md` (D-V2-51), `V2_PHASE_STATUS.md` row 14, `V2_IMPLEMENTATION_ROADMAP.md` P8
  A `.github/workflows/container.yml` — (8) by hand only: `docker build` of the Dockerfile and `devcontainer up`, as committed
  M `editors/vscode/e2e.js` — (9) `pgrep`/`pkill` beside PowerShell, `--no-sandbox` as root; `docs/REMAINING_WORK.md` 7.4 closed; `docs/DELULULANG_V2/V2_P8_DESIGN.md` — where P8-01's work is
  M `Dockerfile`, `.devcontainer/devcontainer.json`, `docs/design/CROSS_PLATFORM_VERIFICATION.md` — (10) the first builds' results; the Dockerfile fixed
  M `crates/delulu-runtime/tests/actors_pingpong.rs` (the verdict names SMT), `V2_DECISION_LOG.md` (D-V2-47 §6), `AGENTS.md`
    (brief for the evidence of an absence), `HANDOFF.md` §11.3, `docs/CLOUD_ROUTINE.md`, `docs/assistant-memory/cloud-period-2026-09-28.md` — (11)
  M `Dockerfile` (header), `docs/design/CROSS_PLATFORM_VERIFICATION.md`, `docs/REMAINING_WORK.md` 7.3 — (12) RW 7.3 closed
  A `.github/workflows/editor-e2e.yml` — (13) by hand first: VS Code from Microsoft's apt repository, `xvfb-run node e2e.js`
  Deleted: nothing.
  M `docs/CLOUD_SYNC_LOG.md` — this entry
  M `docs/survey/*` — regenerated
- Survey and doctor (after the last edit of each slice, and of this entry): `survey check` ok (1,461
  nodes at the end), `survey findings` 0 errors, 14 warnings (unchanged from the baseline), `doctor
  --check` ok — all checks pass; each commit message carries its own figures.
- Verified: in the VM — the full suite alone three times (1,995 + `egress_features` after `cargo fetch`;
  1,997; 1,998 passed, 0 failed), clippy `--workspace -D warnings` clean each time, `cli-sweep.sh` 45/45,
  every new test red on the old code (the ping-pong starve, ADAPTER-SPELL-1's impostor, ATTEST-FIFO-1's
  hang) and every mutant red (six on PS-D-02, the one-worker runtime), the editor's real end-to-end test
  on Linux (and falsified). CI read, each by its log where it mattered: `5bb39bc` red (Windows ping-pong,
  1.31x/1.28x); `9ac5477`, `fef8ccd`, `ed74683`, `b7abcfb`, `81c644f`, `0a7b881` green on every job;
  `miri-slow` `36381950975` green on all three crates; `container` `36399908461` (Dockerfile failed,
  devcontainer passed) and `36400717604` (both passed). **Completed: PS-D, P7.**
- Redo on the laptop: run `node editors/vscode/e2e.js` once on Windows (its Windows branch was
  restructured, not changed); nothing else — CI runs Windows and macOS on every push.
- For the laptop's memory: `docs/assistant-memory/cloud-period-2026-09-28.md` and `HANDOFF.md` §11.3
  carry this run's facts (no `gh` in the VM; the MCP log tool's 5,000-line cap; `cargo fetch` first;
  Docker Hub refused, `container.yml` instead; VS Code from Microsoft's apt repository; x86 CI runners
  are SMT, arm64 measures the ping-pong criterion).
- Open / next: (1) read the push runs of `e430c2a`, `17d4623`, `f5b5985`, `8686479`, `3fd53cd` and this
  entry's commit (each test job ends with the ping-pong verdict — record them); (2) **P8-01**, the
  control program in a guest (`V2_P8_DESIGN.md` — read its "Where the work is" first); (3) the
  adversarial pass on PS-D-02, the `--` rule and ADAPTER-SPELL-1 that a Sonnet 5 sous-chef was running
  at this entry's writing — if its findings are not in `V2_LOG.md`, re-run the pass; (4) RW 7.4's
  "next": make the editor end-to-end a CI gate on the Linux editor job. **For the owner:** D-V2-49 is an
  entrenched edit (`DELULU_CORE.md` v0.3) taken under the delegation — flagged for his review.

### 2026-09-28 (evening) — the laptop synced; NVIDIA OpenShell studied; PS-E, P8-04 and P9 designed (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q` (the laptop's head chef)   Model: Claude Opus 5.5
- Branch: `master`, from the laptop   Pull request: none   Merged: n/a
- Base: `dd543e5` (the laptop fast-forwarded to it from `5bb39bc` first — every commit in routine run 1's
  entry above; `git ls-files --eol` clean; on Windows `survey check` ok, 1,462 nodes; `findings` 0
  errors, 14 warnings; `doctor --check` 29 passed). **The newest `Cloud handoff` commit below is the
  new baseline** for the sync procedure above; everything after it on `origin` is the cloud's again.
- Commits: (1) `fed54cb` Cloud handoff (7): NVIDIA OpenShell studied — PS-E, P8-04 and P9 designed;
  (2) `Cloud handoff (8): the OpenShell export allows GET, not a preset` — a soundness correction to the
  study's §4.5 found on re-reading it (the `read-only` preset also allows `HEAD` and `OPTIONS`, wider
  than a program's authority; grant roots are exported as absolute resolved paths), and the memory
  index's stale V2 line; (3) `Cloud handoff (9): the routine's brief checked against the cloud docs;
  Sonnet 5.5` — the owner's question the same evening: M `docs/CLOUD_ROUTINE.md` (models, effort, classifier
  fallback, `gh`, the VM's reach and limits, usage), `AGENTS.md` (Sonnet 5.5 and Haiku 4.5 for testing
  passes), `CLAUDE.md` (`gh`, release assets, the fallback's trailer), `HANDOFF.md` §11.1/§11.3,
  `docs/DELULULANG_V2/V2_README.md`, `V2_LOG.md`, `docs/assistant-memory/` (two files); (4) `Cloud handoff (10): agents are Sonnet 5.5, and Haiku 5.5 once released` — the owner's
  ruling minutes later ("run Opus 5.5 at xhigh effort. if needed use sonnet 5.5 (latest) as agents and
  haiku 5.5 will be launched in coming weeks, use haiku 5.5 as agent after launching"): M `AGENTS.md`,
  `docs/CLOUD_ROUTINE.md`, `HANDOFF.md`, `V2_README.md`, `V2_LOG.md` (and `fed54cb`'s run read green),
  `docs/assistant-memory/` (two files)
- Files and folders:
  A `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` — the study: what was read, the two designs side by side,
    what DeluluLang takes (each slice's design, witness, falsifier) and does not, the phase changes
  A `docs/assistant-memory/openshell-study-2026-09-28.md` — the study, as a memory file
  M `docs/DELULULANG_V2/V2_DECISION_LOG.md` — D-V2-52 (terms, order), D-V2-53 (PS-E), D-V2-54 (P8-04), D-V2-55 (P9)
  M `docs/DELULULANG_V2/V2_MASTER_PLAN.md` (§4 rows 14–16, §5 gaps, §8), `V2_IMPLEMENTATION_ROADMAP.md`
    (PS-E, P8-04, P9, the dependency line), `V2_PHASE_STATUS.md` (rows 14–16), `V2_P8_DESIGN.md` (P8-01 on
    PS-E; P8-04), `V2_SECURITY_MODEL.md` (§6 L3 note, §9b new, §10 — its stale "PS-D-02 is not built"
    corrected), `V2_README.md` (two file rows; where V2 is), `V2_LOG.md` (this evening's entry)
  M `docs/REMAINING_WORK.md` — rows 2.11, 2.12, 4.25–4.30, 6.14 (4.27 is hypotheses, not findings)
  M `docs/CLOUD_ROUTINE.md` — the owner's commission and its terms; step 4's order PS-E → P8 → P9
  M `HANDOFF.md` (where things stand; §0's PS-E paragraph; §11.1 the commission; §11.8 the lesson),
    `AGENTS.md` (PS-E is next), `README.md` (V2's next), `docs/DEPLOYMENT.md` (OpenShell: designed, not a
    recipe yet), `docs/REPOSITORY_STRUCTURE.md` (§5.11 rows)
  M `docs/assistant-memory/MEMORY.md`, `cloud-period-2026-09-28.md` (merged with the laptop's copy: the
    baseline, the routine, runs 2–4 and the evening), `README.md` (the added file)
  M `docs/survey/*` — regenerated
  Deleted: nothing.
- Survey and doctor (after the last edit): `survey check` ok; `survey findings` 0 errors, 14 warnings;
  `doctor --check` 29 checks passed (Windows).
- Verified: documents only — no code or test changed. The routine's runs read by their logs: run 1
  (`cse_013AZJ6RqYq59CeMgkvLV1BM`) as its entry says; runs at 10:11, 10:12 and 10:44 UTC each ended in
  seconds on the five-hour usage limit, having done nothing. This commit's push run: read by the next
  routine run (step 3).
- Redo on the laptop: from routine run 1's entry, `node editors/vscode/e2e.js` on Windows, and the sync
  procedure's full suite on Windows and in WSL — not run this evening (CI ran every OS on each of run
  1's commits, each read green by run 1).
- For the laptop's memory: done on the laptop itself — `openshell-study-2026-09-28.md` added and
  `cloud-period-2026-09-28.md` merged in both the memory directory and `docs/assistant-memory/`;
  `HANDOFF.md` §11.1 and §11.8.
- Open / next: (1) read the push runs of `045c21a`, `7e9d97d` and (10) — `fed54cb`'s, `36463058082`, was
  read green on every job; (1b) CI hygiene before it bites: `actions/checkout@v4` and `actions/cache@v4`
  run on Node 20, which GitHub now forces onto Node 24 (move to their Node-24 majors), and
  `ubuntu-latest` becomes Ubuntu 26 from 2026-10-19 (pin `ubuntu-24.04` or verify on 26 first);
  (2) **PS-E-01** — the boundary confirmed by construction
  (`V2_OPENSHELL_STUDY.md` §4.1), then E-02 … E-06 in order; (3) routine run 1's open items (2) and (4)
  stand, re-ordered: P8-01 now follows PS-E; the adversarial pass on PS-D-02 (its item 3) still stands.
  **For the owner:** D-V2-49 (an entrenched edit, `DELULU_CORE.md` v0.3) and D-V2-53's consequence for
  `hostile-agent` on hosts without user namespaces are flagged for his review.

### 2026-09-28 (night) — routine run 2: CI read; workflows off Node 20, pinned to Ubuntu 24.04; PS-E-01 begun
- Session: `https://claude.ai/code/session_01TfVRPwf7BAv6L8SzocuB1d`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there)   Pull request: none   Merged: n/a
- Base: `cfbfbdc` (the newest `Cloud handoff` commit — the laptop's baseline)
- Commits: (1) `ff701ae` CI hygiene: every action on its Node-24 major, every Ubuntu runner pinned to 24.04;
  (2) `062a78c` PS-E-01, first step: the guest confirms its boundary before it is sent the program (D-V2-56);
  (3) `6ceaf2d` PS-E-01, second step: every sandboxed run reports its boundary's five properties (D-V2-57);
  (4) `8b2994a` The red-team pass on channel /3: seven defects around a guarantee that held, fixed (D-V2-58);
  (5) `42f5ea0` PS-E-01, third step: hostile-agent refuses a boundary that lacks a property (D-V2-59);
  (6) `71221d3` RW 4.31 closed: an external guest's words about itself are its own;
  (7) `aeea324` PS-E-02, first part: an external launcher ends with its host (Linux);
  (8) `5d63119` A channel frame means one value; CI's downloads retry (the microvm job's red run 36490575764);
  (9) `Routine run 2 closed: every push run read green` — this entry's last lines
- Files and folders:
  M `.github/workflows/ci.yml`, `release.yml`, `container.yml`, `editor-e2e.yml`, `channel-measure.yml`,
    `host-capability-probe.yml` — (1) `checkout@v5`, `cache@v5`, `setup-node@v5`, `setup-python@v6`,
    `setup-java@v5`, `upload-artifact@v6`, `download-artifact@v7`, `attest-build-provenance@v3`;
    `ubuntu-latest` → `ubuntu-24.04` (the test matrix's job is now `test (ubuntu-24.04)`)
  M `docs/design/CROSS_PLATFORM_VERIFICATION.md` — (1) the pin, noted where the matrix is described
  M `crates/delulu-runtime/src/channel.rs` — (2) `delulu-sandbox-channel/3`: `Open` (generation, no program),
    `Program`, `Confined { applied, generation }`, `HostChannel::with_generation`, `ChannelSink::receive`;
    unit test for the generation; the fuzz corpus and `fuzz_one_frame` for the new frames
  A `crates/delulu/src/boundary.rs` — (2) the typestate `open` → `Opened::confirm` → `Confirmed::send_program`;
    its unit tests (an honest guest; four that do not confirm)
  M `crates/delulu/src/guest.rs` — (2) the guest confirms before it reads the program; the host mints a
    generation for every run, converses through `boundary.rs`, reports `sandbox.generation`, `outcome.ran`
    false when unconfirmed, `confirmed`/`generation` in the death record; the probe through the typestate
  M `crates/delulu/src/main.rs` (the module), `schema.rs` (`sandbox_run.generation`), `microvm.rs` (a
    comment), `crates/delulu-diag/src/codes.rs` (the SANDBOX topic names `/3`) — (2)
  A `crates/delulu/tests/sandbox_confirm_cli.rs` — (2) the two witnesses, red on `ff701ae`
  M `CHANGELOG.md`, `docs/DEPLOYMENT.md`, `docs/for-agents.md`, `docs/REMAINING_WORK.md` (4.25),
    `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-56), `V2_PHASE_STATUS.md` (row 14: in progress),
    `V2_OPENSHELL_STUDY.md` (§4.1 "built so far"), `V2_SECURITY_MODEL.md` (the diagram's `/3`; §9b) — (2)
  M `HANDOFF.md` (where things stand; §11.3 raw.githubusercontent.com; §11.5 two traps),
    `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 2) — (2)
  M `docs/CLOUD_ROUTINE.md` — (2) loop engineering: start the health build in the background at once; draft
    the records in the scratchpad while the suite runs
  M `crates/delulu/src/boundary.rs` — (3) `PROPERTIES`, `properties()` from the posture; the per-platform
    unit test; `crates/delulu/src/guest.rs` (`sandbox.properties` in the report), `schema.rs`
    (`properties`, `property`), `crates/delulu/tests/sandbox_confirm_cli.rs` (two more tests) — (3)
  M `.github/workflows/ci.yml` — (3) each test job and arm64 print one L1 run's `sandbox.properties`
    (a step and a notice) before the ping-pong verdict; `docs/CLOUD_ROUTINE.md` step 3 says to read both
  M `CHANGELOG.md`, `HANDOFF.md`, `docs/for-agents.md`, `docs/REMAINING_WORK.md` 4.25,
    `V2_DECISION_LOG.md` (D-V2-57), `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.1 — (3)
  M `crates/delulu/src/guest.rs` — (4) `end_guest` (a failed conversation ends the guest; 10 s grace after
    a goodbye), `Evidence.sent` (`ran` = sent), the oversized-program refusal, the conversation's error
    shown bounded, the death records' `sent`/`ended_by_host`; `crates/delulu/src/boundary.rs` — (4) the
    unanswered first request recorded, `unconfirmed` bounded; `crates/delulu/src/pipe_channel.rs` — (4)
    `HostPipes`: a writer thread with the deadline, a bounded read queue, two unit tests;
    `crates/delulu-runtime/src/channel.rs` — (4) `shown`, `MAX_DENIED_CHARS`, `record_unanswered`, a unit
    test; `crates/delulu-runtime/src/actors.rs` — (4) the thread exemption's reason names the writer;
    `crates/delulu/tests/sandbox_confirm_cli.rs` — (4) four witnesses and the Python fake guest
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4 the named findings; §11.5, §11.7 lessons), `docs/REMAINING_WORK.md`
    (4.31 F7, 4.32 channel hygiene), `V2_DECISION_LOG.md` (D-V2-58), `V2_PHASE_STATUS.md`,
    `docs/CLOUD_ROUTINE.md` (a red-team pass after a security slice; the cost of CI reads),
    `docs/assistant-memory/cloud-period-2026-09-28.md` — (4)
  M `crates/delulu/src/policy.rs` (`Profile::required`), `crates/delulu/src/boundary.rs` (`Requirement`,
    `Refused`, the check in `Opened::confirm`), `crates/delulu/src/guest.rs` (the requirement passed in; the
    refusal printed as DL1408, exit 2, reported `ran: false`), `crates/delulu/tests/sandbox_confirm_cli.rs`
    (the witness), `crates/delulu/tests/sandbox_run_cli.rs` (the two `hostile-agent` tests expect exit 2 on
    macOS), `skills/delulu/SKILL.md`, `docs/for-agents.md`, `CHANGELOG.md`, `HANDOFF.md`,
    `docs/REMAINING_WORK.md` 4.25, `V2_DECISION_LOG.md` (D-V2-59, flagged), `V2_PHASE_STATUS.md`,
    `V2_OPENSHELL_STUDY.md` §4.1 — (5)
  M `crates/delulu/src/guest.rs` (`guest_reported` at L3), `schema.rs` (`guest_reported`),
    `crates/delulu/tests/sandbox_confirm_cli.rs` (the witness), `crates/delulu/tests/sandbox_external_cli.rs`
    (the strict assertion), `CHANGELOG.md`, `HANDOFF.md`, `docs/for-agents.md`, `docs/REMAINING_WORK.md`
    (4.31 closed), `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.1 — (6)
  M `crates/delulu/src/guest.rs` (`launch_external`: `PR_SET_PDEATHSIG`, the parent re-check),
    `crates/delulu/tests/sandbox_confirm_cli.rs` (the witness), `CHANGELOG.md`, `HANDOFF.md`,
    `docs/REMAINING_WORK.md` 4.26, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.2 — (7)
  M `crates/delulu-runtime/src/channel.rs` (`read_frame` refuses bytes after a frame's value; the witness),
    `scripts/microvm/fetch-firecracker.sh`, `scripts/microvm/build-image.sh`, `.github/workflows/ci.yml`
    (`curl --retry 4 --retry-delay 5` on the four downloads), `docs/REMAINING_WORK.md` 4.32 — (8)
  M `docs/DELULULANG_V2/V2_LOG.md` — this run's entries
  M `docs/CLOUD_SYNC_LOG.md` — this entry
  M `docs/survey/*` — regenerated
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,464 nodes, 12,653 edges); `doctor --check` ok,
  all checks pass (26 in this VM). After each slice: in its commit message.
- Verified: CI read by id — `36463375580` (`045c21a`), `36464748581` (`7e9d97d`), `36465304879`
  (`cfbfbdc`, every job; ping-pong arm64 MEASURED 2.91x/4.13x passed, the x86 runners and macOS NOT
  MEASURED), the nightly `36402530469`, `release` `36413736270`, `editor-e2e` `36401854421` — all
  success. Each new action major's runtime read from its `action.yml` at the tag (`V2_LOG.md`).
  (1)'s push run `36477748775` — success on every job, the Node-20 warning gone from the logs read,
  arm64 ping-pong MEASURED 2.88x/4.12x passed. (2) in the VM: both witnesses red on `ff701ae` and green
  after; three mutants landed and killed (M1 program before `confirm`, M2 any generation, M3 an unconfirmed
  guest's console request answered); clippy `-D warnings` clean; the full suite alone 2,002 passed and 1
  failed — the thread-stack gate on `boundary.rs`'s test attribute, fixed, re-run green. (3): both new
  tests red on `062a78c`, green after; M4–M6 landed and killed; clippy clean; the full suite alone
  2,006 passed, 0 failed, 15 ignored (152 binaries). CI read: `062a78c` `36480421762` and `6ceaf2d`
  `36481810253` — success on every job, the `microvm` job's `/3` guest image and hostile-guest red team
  included; each OS's properties read (Linux x86-64, arm64, Windows: all five; macOS: egress and the
  privilege floor only); ping-pong arm64 MEASURED 2.88x/4.13x passed, the others NOT MEASURED.
  (4): a Sonnet 5.5 red-team sous-chef against a frozen copy of `6ceaf2d`'s binary, outside the repo
  (~85 attempts, 700 fuzzed frames; the guarantee held); every finding re-run by the head chef — six
  witnesses red on `6ceaf2d`, green after; mutants M7–M9 landed and killed (M8 survived the first witness,
  whose fake guest raced two writes — fixed); clippy clean; the full suite alone 2,013 passed, 0 failed,
  15 ignored (152 binaries). (5): the witness red on `8b2994a` (an unattested launcher ran the program
  under `hostile-agent`), green after; M10 killed; clippy clean; the full suite alone 2,014 passed,
  0 failed, 15 ignored. CI read: `8b2994a` `36486821458` and `42f5ea0` `36487993092` — success on every
  job (macOS's `hostile-agent` refusal real; the microVM's `hostile-agent` runs still run). (6): the
  witness red on `42f5ea0`, green after; M11 killed; clippy clean; the full suite alone 2,015 passed,
  0 failed, 15 ignored. (7): the witness red on `71221d3` (the launcher outlived its killed host), green
  after, twice; clippy clean; the full suite alone 2,016 passed, 0 failed, 15 ignored. **CI red:**
  `71221d3`'s run `36490575764` failed in one job — `microvm`, GitHub's release download answered HTTP 500
  to `fetch-firecracker.sh`, which had no retry; reproduced locally (a 500 once: exit 22; with `--retry`:
  exit 0), fixed in (8), the failed job re-run once. (8): the strict-frame witness red on `aeea324`, green
  after; clippy clean; the full suite alone 2,017 passed, 0 failed, 15 ignored. **Every push run of this
  run read:** `ff701ae` `36477748775`, `062a78c` `36480421762`, `6ceaf2d` `36481810253`, `8b2994a`
  `36486821458`, `42f5ea0` `36487993092`, `71221d3` `36490575764` (attempt 1 red — the Firecracker
  download's 500; attempt 2, the re-run, success), `aeea324` `36491419602`, `5d63119` `36492440031` — all
  success. `master` is green at `5d63119`.
- Redo on the laptop: (2) changes the channel to `/3` — **rebuild the WSL microVM guest images**
  (`~/microvm-image-a`, `~/microvm-image-b`: a `/2` guest refuses a `/3` host), and run the suite on Windows
  (the AppContainer guest and `sandbox probe`'s contained path run only there and on CI).
- For the laptop's memory: `HANDOFF.md` §11.3 (raw.githubusercontent.com reachable) and §11.5 (two traps);
  `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 2).
- Open / next: (1) read (9)'s push run — the only one this run did not read (a records-only commit); (2) **E-02's rest** —
  the macOS guest outlives its host (the red one; CI only: a watcher thread before the program starts,
  `kqueue` `NOTE_EXIT` on the host's pid, `getppid() == 1` fallback, `_exit` at once; witnessed on the
  macOS runner by killing the host and timing the guest from outside), and the Windows launcher's Job
  Object with kill-on-close; (3) PS-E-01's remainder — an attester's claims mapped to properties (so an
  attested L3 can satisfy `hostile-agent`), then `contained`'s set once macOS's gaps close; (4) E-03 …
  E-06, P8, P9; RW 4.32's channel hygiene. **For the
  owner:** D-V2-59 narrows D-V2-53 §2 (flagged); D-V2-49 and D-V2-53's consequence stay flagged.

### 2026-09-29 — routine run 3: a one-runner witness workflow; other OSes linted from Linux; PS-E-02 complete
- Session: `https://claude.ai/code/session_011rtmJDeAZtHQ5iCE18Pofo`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there); the macOS witnesses were read red, then green, on
  `claude/wonderful-hamilton-mjutjd` first, and `master` fast-forwarded to it   Pull request: none   Merged: n/a
- Base: `7b9aac8` (routine run 2's last commit)
- Commits: (1) `a39b423` Loop engineering: a one-runner witness workflow; macOS linted from Linux;
  (2) `2d08622` PS-E-02 witness: a computing guest ends when its host is killed (red on macOS);
  (3) `9c38027` PS-E-02 witness: the external launcher's, on macOS too;
  (4) `2d9d2a0` PS-E-02, macOS: a watcher outside the guest ends it with its host (D-V2-60);
  (5) `3ec690b` PS-E-02 on macOS: the records (D-V2-60);
  (6) `37828ab` PS-E-02 witness: an external launcher ends with its host on Windows (red expected) — it also
  carried `scripts/check-other-os.sh`, already staged when it was committed (its message does not say so);
  (7) `1a63829` PS-E-02, Windows: an external launcher joins a kill-on-close job;
  (8) `c9739db` PS-E-02 on Windows: the records - PS-E-02 complete (D-V2-60);
  (9) `dd2a542` PS-E-03, first step: the escaped guest; H1, H2, H3 confirmed and closed, H7 found (D-V2-61);
  (10) `ff251bb` A guest gone before the host opens the channel is told in words (macOS CI red, 36527491801);
  (11) `30e3262` PS-E-03 and the macOS CI fix: read green on every runner; the records;
  (12) `6f88202` PS-E-03 H4: a serving host is closed to its own user (D-V2-62);
  (13) `e584f8d` PS-E-03 H8: an escaped guest cannot type into the operator's terminal (D-V2-63); H4's records;
  (14) `407e423` PS-E-03 H9: an escaped guest signals only itself (D-V2-64);
  (15) `be749d1` PS-E-03 H10: an escaped guest changes no other process (D-V2-65);
  (16) `PS-E-03 H5: writes are denied only where truncation is too (D-V2-66); the run closed`
- Files and folders:
  A `.github/workflows/witness.yml` — (1) one test target on one runner at any ref, by hand; (4) `bin:NAME`
  A `scripts/check-macos.sh` — (1) clippy `-D warnings` for `aarch64`/`x86_64-apple-darwin` from Linux;
  R `scripts/check-macos.sh` -> `scripts/check-other-os.sh` — (6)/(7) Windows msvc linted from Linux too
  M `docs/CLOUD_ROUTINE.md` (step 5: a CI-only witness, red on a branch first; reading it), `CLAUDE.md` (the
    manual runs), `docs/REPOSITORY_STRUCTURE.md` (both rows) — (1), (5)
  M `crates/delulu/tests/sandbox_confirm_cli.rs` — (2) `a_computing_guest_ends_when_its_host_is_killed`,
    `gone()` by `ps`, `guest_of()`; (3) the launcher's witness compiled for macOS; (4) their records
  M `crates/delulu/src/jail.rs` — (4) `HOST_WATCH_SUBCOMMAND`, `HostWatch` (start, armed, drop),
    `run_host_watch` (kqueue), the non-Windows `Jail` holds the watcher; `crates/delulu/src/guest.rs` — (4) the
    watcher started in `launch` (claimed once armed) and `launch_external` (claimed for nothing);
    `crates/delulu/src/cli.rs` — (4) the dispatch; `crates/delulu/src/boundary.rs` — (4) the macOS
    property test; `crates/delulu/tests/guest_cli.rs` — (4) macOS's "killed with the host"
  M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 two lessons), `docs/DEPLOYMENT.md` (macOS row),
    `docs/book/THE_DELULULANG_BOOK.md` (macOS row), `docs/REMAINING_WORK.md` (4.26),
    `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-60), `V2_LOG.md`, `V2_PHASE_STATUS.md` (row 14),
    `V2_OPENSHELL_STUDY.md` (§4.2), `V2_SECURITY_MODEL.md`, `V2_IMPLEMENTATION_ROADMAP.md`,
    `docs/assistant-memory/cloud-period-2026-09-28.md` — (5)
  M `crates/delulu/tests/sandbox_confirm_cli.rs` — (6) `an_external_launcher_ends_when_its_host_is_killed_on_windows`
  M `crates/delulu/src/jail.rs` — (7) `end_with_host`; the Windows `Jail` holds two handles; the test accessor;
    `crates/delulu/src/guest.rs` — (7) the launcher created suspended, joined, resumed
  M `CLAUDE.md`, `HANDOFF.md`, `docs/CLOUD_ROUTINE.md`, `docs/REPOSITORY_STRUCTURE.md`,
    `docs/assistant-memory/cloud-period-2026-09-28.md` — (7) the script's new name
  M `CHANGELOG.md`, `HANDOFF.md` (§11.5: `git stash pop`), `docs/REMAINING_WORK.md` (4.26 closed),
    `V2_DECISION_LOG.md` (D-V2-60 §4), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.2,
    `V2_SECURITY_MODEL.md`, `V2_IMPLEMENTATION_ROADMAP.md` — (8)
  M `crates/delulu/src/jail.rs` — (9) `escaped_tests` (the escaped guest; H1, H2, H3, H7), `SYSTEM_READ`
    (`/proc/self`, five devices), `lock_down_self` (sockets, the H1 calls, `clone`'s namespace flags, `clone3`
    ENOSYS, the new word); `crates/delulu-runtime/src/channel.rs` (`SELF_APPLIED`: "no sockets but the
    channel"); `crates/delulu/src/policy.rs` (the network row needs it); `crates/delulu/src/boundary.rs` (the
    property test); `crates/delulu-diag/src/codes.rs` (the SANDBOX topic's Linux line) — (9)
  M `scripts/check-other-os.sh` (arm64 Linux in the defaults), `docs/CLOUD_ROUTINE.md` — (9)
  M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.4 the four findings), `docs/DEPLOYMENT.md` and
    `docs/book/THE_DELULULANG_BOOK.md` (the Linux rows), `docs/REMAINING_WORK.md` (4.27), `V2_DECISION_LOG.md`
    (D-V2-61), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.3, `V2_SECURITY_MODEL.md` §10 — (9)
  M `crates/delulu/src/boundary.rs` (`open`, `send_program` in words; the witness), `crates/delulu/src/jail.rs`
    (the watcher's `WATCH_NOTHING`) — (10)
  M `.github/workflows/witness.yml` (several targets, or `all`; `--no-fail-fast`), `docs/CLOUD_ROUTINE.md` (a
    green read covers every target), `HANDOFF.md` §11.5, `docs/DELULULANG_V2/V2_LOG.md` — (11)
  M `crates/delulu/src/guest.rs` (`PR_SET_DUMPABLE` after the launch), `crates/delulu/tests/sandbox_confirm_cli.rs`
    (the H4 witness) — (12)
  M `crates/delulu/src/jail.rs` (`escaped_tests`: H8, `TIOCSTI`, `TIOCSTI_HIGH`; `lock_down_self`: the `ioctl`
    rule), `scripts/check-other-os.sh` (musl), `docs/CLOUD_ROUTINE.md` — (13)
  M `crates/delulu/src/jail.rs` (`escaped_tests`: H9's signal attempts and the self-signal control;
    `lock_down_self`: the signal rules), `CHANGELOG.md`, `HANDOFF.md`, `docs/REMAINING_WORK.md` (4.27),
    `V2_DECISION_LOG.md` (D-V2-64), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.3,
    `V2_SECURITY_MODEL.md` §10 — (14)
  M `crates/delulu/src/jail.rs` (`escaped_tests`: H10's attempts and the `PRLIMIT_SELF` control;
    `lock_down_self`: the setters and the `prlimit64` rule), `CHANGELOG.md`, `HANDOFF.md`,
    `docs/REMAINING_WORK.md` (4.27), `V2_DECISION_LOG.md` (D-V2-65), `V2_LOG.md`, `V2_PHASE_STATUS.md`,
    `V2_OPENSHELL_STUDY.md` §4.3, `V2_SECURITY_MODEL.md` §10 — (15)
  M `crates/delulu/src/policy.rs` (`exact` for "no file writes"), `crates/delulu/src/boundary.rs` (the H5 pin),
    `CHANGELOG.md`, `HANDOFF.md`, `docs/REMAINING_WORK.md` (4.27), `V2_DECISION_LOG.md` (D-V2-66), `V2_LOG.md`
    (H5; the run closed), `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.3, `V2_SECURITY_MODEL.md` §10,
    `docs/CLOUD_ROUTINE.md` (cheaper CI polling) — (16)
  M `CHANGELOG.md`, `HANDOFF.md`, `docs/REMAINING_WORK.md` (4.27), `V2_DECISION_LOG.md` (D-V2-62, D-V2-63),
    `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` §4.3, `V2_SECURITY_MODEL.md` §10 — (13)
  M `docs/CLOUD_SYNC_LOG.md` — this entry
  M `docs/survey/*` — regenerated
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,466 nodes, 12,743 edges); `doctor --check` ok,
  all checks pass (26 in this VM). After each slice: in its commit message.
- Verified: routine run 2's last push run, `7b9aac8` `36494315402` — success (read by id). No nightly
  since `36402530469`. (1): `witness.yml` parses; `scripts/check-macos.sh` clean on both macOS targets,
  red (E0308) with a macOS-only type error planted, restored. (2): the witness green on Linux (9 ms), red
  with the jail's `PR_SET_PDEATHSIG` removed (a mutant, restored) — no test had witnessed it before; on
  macOS **red**, `witness.yml` run `36526271163` ("the guest outlived its host by more than 3.15 s").
  (3): on macOS **red**, run `36526351005` ("the external launcher outlived its host"). (4): on macOS
  **green**, runs `36526707055` (`sandbox_confirm_cli`, 12 passed; the guest gone 3.6 ms after the
  kill), `36526709263` (`guest_cli`, 5 passed), `36526711462` (the boundary unit tests, 3 passed); in
  the VM clippy `-D warnings` clean on Linux and both macOS targets; the full suite alone 2,018 passed,
  0 failed, 15 ignored (152 binaries), cargo exit 0. Push runs: `a39b423` `36526041627` — success.
  (6): on Windows **red**, run `36527876892` ("the external launcher outlived its host by more than 3.0 s").
  (7): `scripts/check-other-os.sh` clean for all three targets, red with a planted Windows-only type error;
  on Windows **green**, runs `36528140459` (`sandbox_confirm_cli`, 7 passed; the launcher gone 20 ms after
  the kill), `36528149629` (`sandbox_external_cli`), `36528152057` (the jail unit tests), `36528154367`
  (`sandbox_run_cli`). The Linux suite was not re-run for (7): every line it changed is compiled only for
  Windows (on Linux the cfg selects the same code as before), Linux clippy is clean, and
  `sandbox_confirm_cli`, `doctor_cli`, `repository_structure` and `evidence_claims` pass.
  (9): the escaped-guest witnesses red on `1a63829` for H1, H2, H3 (the whole list in `V2_LOG.md`), H7 red by
  mutant; green after; six mutants red; clippy clean on Linux, arm64 Linux, macOS and Windows; the generated
  reference in sync; the full suite alone 2,023 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.
  **CI red, read:** `3ec690b`'s push run `36527491801` failed one job, `test (macos-latest)`, one target,
  `sandbox_external_cli` — `a_backend_that_is_not_one_or_a_launcher_that_dies_fails_legibly`: a launcher
  that exits at once (`delulu --version`) got "Broken pipe (os error 32)" raw, because the macOS watcher's
  start delays the host's first write past that launcher's death (read again on its own: `witness.yml`
  `36529502237`, the same failure). Fixed in (10): the witness red on `dd2a542` ("Broken pipe (os error
  32)"), green after; read green on the runners at `ff251bb` — `36530276934` (macOS `sandbox_external_cli`,
  3 passed), `36530279678` (macOS `sandbox_confirm_cli`, 12 passed) — with PS-E-03's first CI reads:
  `36530282206` (arm64 escaped-guest tests, 5 passed, measured on a non-root runner), `36530285142`
  (x86-64 subordinate-uid `sandbox_run_cli`, 17 passed), `36530287510` (arm64 `sandbox_run_cli`).
  `c9739db`'s push run `36529103180` carries the same macOS failure (the code of `3ec690b`).
  (11): the full suite alone on the landed tree 2,024 passed, 0 failed, 15 ignored (152 binaries), cargo
  exit 0. `c9739db`'s push run `36529103180`: failure, one job, `test (macos-latest)` — the known one.
  (12): H4 measured by hand as a non-root user (`runuser -u delulutester`: a same-user process read the
  serving host's `environ`), then the witness red on `30e3262` as that user, green after; as that user
  `sandbox_confirm_cli` 13, `sandbox_run_cli` 17, `guest_cli` 5, `sandbox_external_cli` 3 passed; the full
  suite alone as root 2,025 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0. Read on the runners at
  `6f88202`: `36531906893` (x86-64, non-root, the subordinate-uid guest: `sandbox_confirm_cli` 13 — the H4
  witness measured —, `sandbox_run_cli` 17, `guest_cli` 5, `sandbox_external_cli` 3) and `36531909687`
  (arm64, the same four) — success.
  (13): H8's witness red on this run's tree (`TIOCSTI` into the controlling terminal after lock-down), green
  after; mutants M7 (the rule dropped) and M8 (compared on 64 bits: `TIOCSTI_HIGH` got through) red.
  `scripts/check-other-os.sh` now lints musl (the microVM guest's) — clean on all five targets, and red
  without the musl cast (E0308) where glibc compiles. The full suite alone 2,026 passed, 0 failed, 15
  ignored (152 binaries), cargo exit 0.
  **`30e3262`'s push run `36531207166` — success on every job**: `master` green again, the macOS test job
  among them, and the `microvm` job whose guest runs PS-E-03's filter.
  (14): H9's witness red on `e584f8d` (SIGNAL_OTHER, SIGNAL_GROUP, SIGNAL_QUEUE), green after, the
  self-signal control true; mutants M9 (rules dropped) and M10 (comparison reversed) red; clippy clean on
  Linux, arm64 Linux and musl. The full suite alone 2,027 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.
  (15): H10's witness red on `407e423` (five calls on the operator's process), green after with the control
  true; mutants M11–M13 red; clippy clean on Linux, arm64 Linux and musl (musl caught the witness's
  `sched_param`). The full suite alone 2,028 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.
  (16): the H5 pin red on `be749d1` (`filesystem_confinement` established on a pre-ABI-3 kernel's words),
  green after; the full suite alone 2,028 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.
- Redo on the laptop: (9) changes the Linux guest's own lock-down — **rebuild the WSL microVM guest images**
  (the VM guest applies the same filter) and run the suite in WSL; Windows: nothing new beyond the suite.
- For the laptop's memory: `HANDOFF.md` §11.5 (two lessons: CI-only witnesses off `master`; a claimed
  guarantee needs its own witness); `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 3).
- Push runs read: `a39b423` `36526041627` success; `3ec690b` `36527491801` **failure** (macOS `sandbox_external_cli`, a raw `Broken pipe` — fixed in `ff251bb`); `c9739db` `36529103180` **failure** (the same); `30e3262` `36531207166` success (every job — `master` green again); `e584f8d` `36533139190` success; `407e423`, `be749d1` and this entry's commit — running as it was written (Open / next (1))
- Open / next: (1) read the push runs this entry names as unread, first; (2) **PS-E-03's rest** — H5 (the
  Landlock ABI as a `hostile-agent` requirement; the report states the effective ABI), H6 (the escaped-guest
  harness on macOS and Windows through `witness.yml`: Seatbelt's and the AppContainer's reach), and a
  **red-team pass on the new filter** (a Sonnet 5.5 sous-chef adding attempts to a copy of
  `jail::escaped_tests` — keyrings, `personality`, `mknod`, the channel descriptor itself, `/sys`, `/etc`);
  (3) the guest's standard error still reaches the operator's terminal raw (escape sequences) and the guest
  keeps a controlling terminal — RW 4.32's relay and a `setsid`; (4) PS-E-01's remainder (attesters' claims as
  properties, `contained`'s set), E-04 (the launcher resolved, hashed, pinnable), E-05, E-06, then P8, P9.
  **For the owner:** D-V2-60 departs from the study's design (the macOS watcher outside the guest); D-V2-61
  to D-V2-65 close five findings the study predicted and three it did not (H7, H8, H9/H10 — the terminal,
  keystroke injection, signals and other processes); D-V2-59, D-V2-49 and D-V2-53's consequence stay
  flagged.
