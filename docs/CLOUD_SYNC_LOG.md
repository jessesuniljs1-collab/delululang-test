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
  (16) `33c20e5` PS-E-03 H5: writes are denied only where truncation is too (D-V2-66); routine run 3's records;
  (17) `Routine run 3 closed: every push run read; master green at 33c20e5` — its last lines
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
  M `HANDOFF.md` §11.5 (three more lessons), `docs/assistant-memory/cloud-period-2026-09-28.md`,
    `docs/CLOUD_ROUTINE.md` (a run's completion from `get_workflow_run_usage`), `V2_LOG.md` — (17)
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
- For the laptop's memory: `HANDOFF.md` §11.4 (the run's findings) and §11.5 (the lessons: CI-only
  witnesses off `master`; a green read covers every target; `git stash pop`; a probe fails on what it did not
  hear; a pin can find the defect; a shared `CARGO_TARGET_DIR`; a claimed guarantee needs its own witness);
  `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 3).
- Push runs read: `a39b423` `36526041627` success; `3ec690b` `36527491801` **failure** (macOS `sandbox_external_cli`, a raw `Broken pipe` — fixed in `ff251bb`); `c9739db` `36529103180` **failure** (the same); `30e3262` `36531207166` success (every job — `master` green again); `e584f8d` `36533139190` success; `407e423` `36533983121` success; `be749d1` `36534873999` success; `33c20e5` `36535808138` success —
  every push run of this run read; `master` green at `33c20e5`. Only (17)'s own run is left to the next run
- Open / next: (1) read (17)'s push run first — the only one this run did not read (records only); (2) **PS-E-03's rest** — H5 (the
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

### 2026-09-29 — routine run 4: `master` red on a records-only commit, read and fixed (Wasmtime 48; an attestation read half-written)
- Session: `https://claude.ai/code/session_01R7P5oTaoPP7KizcvUSiJpp`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there)   Pull request: none   Merged: n/a
- Base: `9fc4d86` (routine run 3's last commit)
- Commits: (1) `63a375e` CI red on 9fc4d86, read and fixed: wasmtime 48.0.3 (D-V2-67); an attestation read half-written;
  (2) `f33183c` RW 4.33: both WebAssembly engines refuse the 3.0 proposals DeluluLang never emits (D-V2-68);
  (3) `2ea4208` RW 2.2 closed: overtaken by P3; the pin its comment named never existed, and Secret.map's entry is pinned now;
  (4) `eb3af0b` Routine run 4 closed: every push run read and green; RW 2.4 measured and bounded; the loop's lessons;
  (5) `Routine run 4: eb3af0b's push run read green; the entry's own subject corrected` — records only
- Files and folders:
  M `crates/delulu-wasm/Cargo.toml` (`wasmtime = "48"`), `Cargo.lock` (wasmtime 48.0.3 and its tree) — (1)
  M `crates/delulu/src/attest.rs` (`Refusal::Incomplete`, `incomplete`: end-of-input and a cut character;
    the unit test's cuts), `crates/delulu/tests/sandbox_attest_cli.rs` (the replay launcher writes, then
    renames; `a_document_written_in_place_is_refused_in_words_that_name_the_rename`) — (1)
  M `.github/workflows/ci.yml` and `deny.toml` (the supply-chain history: 2026-09-29) — (1)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.5: two lessons), `docs/REMAINING_WORK.md` (4.33, new),
    `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-67), `V2_LOG.md`,
    `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_ROUTINE.md` (step 2: cargo-deny in the
    background; step 3: a failing job's whole log saved to a file and grepped; the VM's limits: waiting
    without a bare `sleep`) — (1)
  M `crates/delulu-wasm/src/host.rs` (`harden_wasm_features`: the WebAssembly 3.0 proposals; three tests) — (2)
  M `scripts/check-other-os.sh` (`delulu-runtime` and `delulu-wasm` linted too) — (2)
  M `docs/CLOUD_ROUTINE.md` (step 3: a green run's verdict lines sit behind the cache save) — (2)
  M `CHANGELOG.md`, `deny.toml`, `.github/workflows/ci.yml` (the histories), `docs/REMAINING_WORK.md` (4.33
    closed), `V2_DECISION_LOG.md` (D-V2-68), `V2_LOG.md`, `docs/assistant-memory/cloud-period-2026-09-28.md` — (2)
  M `crates/delulu-check/src/check.rs` (the comment on `higher_order_callback_arg` names its real pins),
    `crates/delulu-check/tests/secret_oracle.rs` (`an_impure_mappers_row_surfaces_in_its_caller`),
    `docs/REMAINING_WORK.md` (2.2 closed), `V2_LOG.md`, `HANDOFF.md` (§11.5, §11.7) — (3)
  M `docs/REMAINING_WORK.md` (2.4: Partial, bounded; the refuted idea), `V2_LOG.md`, `HANDOFF.md` (§11.5: an ignored
    gate is not in the suite's count), `docs/CLOUD_ROUTINE.md` (step 5: `witness.yml`'s `package`; the differential
    by hand for a change to either WebAssembly engine), `docs/REMAINING_WORK.md` §1 (corrections 1.12, 1.13),
    `CHANGELOG.md` and `crates/delulu-wasm/src/host.rs` (a comment: "reached" -> "could reach" — the run showed
    the door open, not the advisory exercised) — (4)
  M `docs/CLOUD_SYNC_LOG.md` — this entry; M `docs/survey/*` — regenerated
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,468 nodes, 12,795 edges); `doctor --check` ok, all
  checks pass (26 in this VM). After the last edit: in the commit message.
- Verified: **routine run 3's closing push run `36537537718` (`9fc4d86`) — failure, read:** `supply-chain`
  (RUSTSEC-2026-0315/0316 against wasmtime 47.0.4, published after the last clean run) and
  `test (macos-latest)` (`sandbox_attest_cli`, `a_document_replayed_from_another_run_is_refused_on_its_nonce`:
  "EOF while parsing a value at line 1 column 0" — the test's launcher wrote the document in place with `cp`);
  every other job green, macOS's properties line as before, its ping-pong verdict "NOT MEASURED" (3 threads).
  `gh`: absent. (1): `cargo deny --all-features check advisories` exit 1 on 47.0.4, exit 0 on 48.0.3 (cargo-deny
  0.20.2, installed in the VM); bans, licences, sources ok. The race witnessed with the launcher's `cp` held
  open (red, the runner's words); fixed launcher green, and green slowed the same way (M3); the new words'
  witness red on `9fc4d86`, green after; M1 and M2 red. Clippy `-D warnings` clean; the full suite alone
  2,029 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.
  (2): the witnesses red on `63a375e` (the program engine accepted five proposals; the plugin store ran a
  `call_ref` module, `Some(7)`), green after; M5, M6, M7 red; `scripts/check-other-os.sh` clean for Windows,
  macOS arm64 and Linux arm64, and red on Windows with the test helper's `cfg` removed; clippy clean; the full
  suite alone 2,032 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.
  (2) on the other runners before `master` moved (`witness.yml`, `claude/friendly-thompson-w49pvi` at `f33183c`):
  macOS `36560326859` 5 passed, Windows `36560329484` 4 passed, Linux arm64 `36560332509` 5 passed.
  (3): M8 (`fold` read at 0) and M9 (`filter` dropped) red on `stdlib_p3`; M10 (`Secret.map` dropped) passed every
  test before the new pin and is red on it. Clippy clean; the full suite alone 2,033 passed, 0 failed, 15 ignored
  (152 binaries), cargo exit 0.
  **Not run:** the red-team pass on the Linux guest's filter (run 3's inbox) — a safety classifier stopped the
  response briefing the Sonnet 5.5 sous-chef before the agent started; no agent ran, and no fallback notice was
  shown (the commits keep naming Opus 5.5, the configured model).
  (4): RW 2.4 measured on the debug binary — 16 KB at depth 120 in 0.39 s; linear in size at a fixed depth; per
  nest about depth^1.6; the first-element idea in `check_list` 10x slower (1.63 s -> 17.0 s), reverted byte for
  byte (`cmp`) and the binary rebuilt and re-measured (1.66 s). Records only otherwise.
- Redo on the laptop: the suite on Windows and in WSL (wasmtime 48 is a new build on every platform).
- For the laptop's memory: `HANDOFF.md` §11.5 (a records-only commit can go red; a fake peer keeps the
  protocol it fakes); `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 4).
- Push runs read: (1) `63a375e` `36556961783` — **success on every job; `master` green again** (macOS: egress, host loss and privilege established, filesystem and memory absent as before; ping-pong MEASURED 2.66x against a 1.41x bar, control 3.75x — passed. Windows: all five established; ping-pong NOT MEASURED, the runner busy — controls 1.93x–2.50x); (2) `f33183c` `36560942558` — success on every job; (3) `2ea4208` `36562163647` — success on every job. (4) `eb3af0b` `36564217831` — success on every job. Every push run of this run read but (5)'s own. The nightly `36548984501` (`9fc4d86`) was still running at the close — its Miri jobs run for hours.
- Open / next: (1) read (5)'s push run (records only) and the nightly `36548984501` on
  `9fc4d86` — expected red on `supply-chain` (the advisories (1) fixed) and possibly macOS's `sandbox_attest_cli`;
  both fixed in `63a375e`, so read it as the old commit's record, and read the NEXT nightly as the real check;
  (2) RW 4.33 — done in (2), D-V2-68; (3) run 3's inbox
  still stands: PS-E-03 H6 (macOS and Windows under the escaped-guest harness), the red-team pass on the filter
  (not run here — do it by hand, one witnessed hypothesis at a time, e.g. the terminal settings reachable through
  the inherited standard error),
  RW 4.32's stderr relay and `setsid`, PS-E-01's remainder, E-04, E-05, E-06, then P8, P9. **E-04 was scoped and
  not started** (the time left could not finish it verified): its honest form executes the file it hashed
  (`fexecve` on Linux, which a shell-script launcher complicates — the tests' launchers are scripts), so plan it
  as its own run. (4) RW 2.4 stays Partial: a real fix is a profile of `check_list`'s unification, not the
  refuted first-element idea.
  **For the owner:** D-V2-67 (wasmtime 48, the LTS line) and D-V2-68 (the WebAssembly 3.0 proposals refused)
  are this run's decisions; D-V2-59, D-V2-49 and D-V2-53's consequence stay flagged from earlier runs.

### 2026-09-29 — routine run 5: CI read green; PS-E-04 — the external launcher resolved once, hashed, pinnable (LAUNCHER-SPELL-1)
- Session: `https://claude.ai/code/session_01PqeUqqZua7e9zJEiEs3kf4`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there; the slice was read on the other runners from
  `claude/friendly-thompson-gsorc5` first, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `cfc5b01` (routine run 4's last commit)
- Commits: (1) `370d641` PS-E-04: the external launcher resolved once, hashed, pinnable, and started as the file hashed;
  (2) `3489b57` PS-E-04's records: D-V2-69, LAUNCHER-SPELL-1, the runners read; routine run 5's entry — records only;
  (3) `6cd68c8` TERMINAL-TEXT-1: a pure program wrote escape sequences to the operator's terminal; escaped at the printers (D-V2-70);
  (4) `ac65c0e` RW 4.32: a guest's standard error relayed by the host, escaped, marked and bounded;
  (5) `d1dbf2e` TERMINAL-TEXT-1's refusal witness takes the same path on Windows: no `:` in its path;
  (6) `41ac869` RW 4.32's records: D-V2-71; master red on Windows for a test, fixed; the loop's lessons — records only;
  (7) `b7393c4` RW 4.34: a program's line break begins no line of its own in a diagnostic (D-V2-72);
  (8) `d566295` RW 4.34's records: D-V2-72, read on macOS and Windows first; H6 stopped by a classifier — records only;
  (9) `30d1563` RW 4.32: the death record names the words the guest confirmed its boundary with;
  (10) `ec4d39d` RW 4.32's death-record words recorded; master green again at 41ac869 — records only;
  (11) `1721539` RW 4.32: the microVM's console reaches the operator escaped, a line at a time;
  (12) `fa69efb` RW 4.32's microVM console recorded, read on the KVM job first; an arm64 red on ec4d39d, recorded — records
  and a test's failure message;
  (13) `Routine run 5 closed: master green at fa69efb` — records only (HANDOFF §11.5: a failure message is all the
  evidence a CI run keeps)
- Files and folders:
  A `crates/delulu/src/launcher.rs` (resolve, hash, pin, `fexecve` on Linux) — (1)
  M `crates/delulu/src/guest.rs` (`Isolation::External` carries the pin; `serve_under` resolves and refuses
    before the launch; `launch_external` starts the resolved file; `launcher_path`/`launcher_blake3` in the
    report and the launch record), `cli.rs` (`--launcher-digest`, the help), `main.rs` (`mod launcher`),
    `run_cmd.rs` (the flag describes a sandboxed run), `schema.rs` (the two fields) — (1)
  M `crates/delulu/tests/sandbox_external_cli.rs` (five witnesses) — (1)
  M `docs/DEPLOYMENT.md` (*Which file runs*), `docs/for-agents.md` — (1)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4 LAUNCHER-SPELL-1; §11.5 two lessons), `docs/CLOUD_ROUTINE.md` (step 2:
    the cross-OS targets in the background), `docs/REMAINING_WORK.md` (4.28), `docs/DELULULANG_V2/V2_DECISION_LOG.md`
    (D-V2-69), `V2_LOG.md`, `V2_OPENSHELL_STUDY.md` (§4.4 built), `V2_PHASE_STATUS.md` (PS-E), `V2_SECURITY_MODEL.md`
    (§10), `docs/assistant-memory/cloud-period-2026-09-28.md` — (2)
  A `crates/delulu/tests/terminal_text_cli.rs` (five witnesses) — (3)
  M `crates/delulu-diag/src/render.rs` (`terminal_safe`, `terminal_line`, `quotable`; the renderer uses them;
    a unit test), `crates/delulu-diag/src/lib.rs` (the two exported), `crates/delulu/src/guest.rs` (the guest's
    fault line), `cli.rs` (`delulu test`'s lines), `repl.rs` (the fault line) — (3)
  M `tests/core-invariance/SNAPSHOT.txt` — ONE line re-recorded (DL0107's quoted line: the raw U+202E is now
    `?`), reviewed — (3)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4 TERMINAL-TEXT-1; §11.5 where a program's strings are printed),
    `docs/REMAINING_WORK.md` (4.34 new and closed; 4.32 widened), `V2_DECISION_LOG.md` (D-V2-70), `V2_LOG.md`,
    `V2_SECURITY_MODEL.md` (§10), `docs/assistant-memory/cloud-period-2026-09-28.md` — (3)
  M `crates/delulu/src/guest.rs` (`relay_stderr`; `launch` separates capture from quiet; the launcher's stream
    piped), `crates/delulu/src/microvm.rs` (`take_console` is `Send`), `crates/delulu/tests/terminal_text_cli.rs`
    (four relay witnesses) — (4); `crates/delulu/tests/terminal_text_cli.rs` (the refusal witness's path) — (5)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.5: a new test's platform rules; a surviving mutant names a missing witness),
    `docs/CLOUD_ROUTINE.md` (step 5: the suite after `survey build`, the gates to re-run after records; a slice's
    new tests on every runner; reading a witness job), `docs/REMAINING_WORK.md` (4.32), `V2_DECISION_LOG.md`
    (D-V2-71), `V2_LOG.md`, `V2_SECURITY_MODEL.md` (§10) — (6)
  M `crates/delulu-diag/src/render.rs` (`indented_continuations`), `crates/delulu/tests/terminal_text_cli.rs`
    (one witness), `CHANGELOG.md`, `HANDOFF.md` (§11.7: H6 stopped by a classifier), `docs/REMAINING_WORK.md`
    (4.34 closed), `V2_DECISION_LOG.md` (D-V2-72), `V2_LOG.md`, `V2_SECURITY_MODEL.md` (§10),
    `V2_PHASE_STATUS.md` (PS-E) — (7), (8)
  M `crates/delulu/src/guest.rs` (`guest_words` in every death record), `crates/delulu/tests/sandbox_confirm_cli.rs`
    (one witness) — (9); `CHANGELOG.md`, `docs/REMAINING_WORK.md` (4.32), `V2_LOG.md` — (10)
  M `crates/delulu/tests/estop_cli.rs` (the control test prints the journal with a failure), `docs/REMAINING_WORK.md`
    (7.17, new) — (12)
  M `crates/delulu/src/microvm.rs` (`relay_console` escapes each line; a unit test) — (11); `CHANGELOG.md`,
    `docs/REMAINING_WORK.md` (4.32: the per-frame deadline left), `V2_SECURITY_MODEL.md` (§10), `V2_LOG.md`,
    `docs/CLOUD_ROUTINE.md` (reading the KVM job on a branch), `docs/assistant-memory/cloud-period-2026-09-28.md` — (12)
  M `docs/CLOUD_SYNC_LOG.md` — this entry; M `docs/survey/*` — regenerated
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,468 nodes, 12,832 edges); `doctor --check` ok, all
  checks pass. After the last edit: in (2)'s commit message.
- Verified: **CI on arrival:** `cfc5b01` `36566189830` — success on every job; the nightly `36548984501` (`9fc4d86`)
  — failure on `supply-chain` only (RUSTSEC-2026-0315/0316 against wasmtime 47.0.4, fixed in `63a375e`), every
  other job green, `miri-slow` all three (syntax 20 min, check 67 min, broker 2 h 45 min). `gh`: absent.
  `cargo deny --all-features check advisories`: ok. **(1):** the five witnesses red on `cfc5b01` (the planted
  `./lnch` ran — LAUNCHER-SPELL-1); green after; M1–M6 red (M1, the path started by name: B ran 3 of 3), each
  restored byte for byte; clippy clean; `check-other-os.sh` clean on five targets; the full suite alone
  2,034 passed, 4 failed (the stale map only — `doctor_cli` three, the Survey's freshness; green once
  regenerated), 15 ignored (152 binaries). `witness.yml` at `370d641`: macOS `36595372725`, Windows `36595377128`,
  Linux arm64 `36595380398` — every test of the three external-launcher targets passed.
  **(3):** TERMINAL-TEXT-1 witnessed red on `3489b57` on five paths (a pure program's `assert_eq` set the
  terminal's title and forged a host line, plain and under `--sandbox`; a refused path; a test's name; a quoted
  line); green after; M7–M12 red, restored byte for byte; clippy clean; the full suite alone 2,043 passed,
  1 failed (the core-invariance snapshot — one reviewed line, re-recorded, green), 15 ignored (153 binaries).
  **(4):** the relay's four witnesses red on `6cd68c8`; green after; M13–M17 red (M16 only once the last-line
  witness held its window open); clippy clean; `check-other-os.sh` clean; the full suite alone 2,048 passed,
  0 failed, 15 ignored (153 binaries), cargo exit 0. `witness.yml` at `ac65c0e`, six targets: Linux x86-64
  `36599642591` and macOS `36599633669` green; Windows `36599638102` red on one test — TERMINAL-TEXT-1's
  refusal witness (a `:` in its path, DL0904 on Windows first) — fixed by (5), read green `36600749661`.
  **(7):** the line-break witness red on `41ac869`, green after; M18 red; clippy clean; the full suite alone 2,049 passed, 0 failed, 15 ignored (153 binaries), cargo exit 0; read on Windows `36602360567` and macOS `36602364313` first, success.
  **(9):** the death-record witness red on `d566295`, green after; M19 red; clippy clean; the full suite alone 2,050
  passed, 0 failed, 15 ignored (153 binaries), cargo exit 0; read on Windows `36602983681` and macOS `36602987750`
  first, success.
  **(11):** the console witness red on `ec4d39d`, green after; M20 red; clippy clean; the full suite alone 2,051
  passed, 0 failed, 15 ignored (153 binaries), cargo exit 0; `ci.yml` dispatched on the branch at `1721539`
  (`36604242542`): `microvm` success (lifecycle, criterion 8, the probe, the hostile guests); `test (macos-latest)`
  red on the stale map only (the branch commit lacked the regenerated map); cancelled after the read.
  **Not run:** PS-E-03 H6 — the head chef's response starting a macOS escaped-guest harness was stopped by a
  safety classifier before anything of it ran; no fallback notice was shown (commits keep naming Opus 5.5).
- Redo on the laptop: the suite on Windows and in WSL (the launcher path changed on every OS; WSL runs the
  `fexecve` path and the swap witness).
- For the laptop's memory: `HANDOFF.md` §11.4 (LAUNCHER-SPELL-1) and §11.5 (`environ` in a `pre_exec` step;
  holding a race's window open with work the code does anyway); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Push runs read: (2) `3489b57` `36596687623` — success on every job. (3) `6cd68c8` `36599039239` — **failure, one job of 16:** `test (windows-latest)`, one test — TERMINAL-TEXT-1's refusal witness (a `:` in its path; DL0904 on Windows first, its message escaped); every other test on Windows passed, every other job green; fixed by (5), read green on Windows first (`36600749661`).
  (6) `41ac869` `36601247472` — **success on every job; `master` green again.** (8) `d566295` `36602779695` — success on
  every job. (10) `ec4d39d` `36603987907` — **red on
  one job, `arm64`: `estop_cli`'s control test printed `REVOKED (operator-revoke)` for a run nobody revoked** — code this run
  did not touch, passed on arm64 at `41ac869` and `d566295`; not reproduced by starving the VM; not root-caused (RW 7.17;
  the test now prints the journal's reason). (12) `fa69efb` `36606231595` — **success on every job
  (16); `master` green at `fa69efb`**, arm64's `estop_cli` included. (13), this entry's last line: the next run reads it.
- Open / next: (1) read (12)'s push run first — and RW 7.17: if `estop_cli`'s control test fails again, its message now
  carries the journal's reason; root-cause it before anything else; (2) RW 4.32's rest — the microVM's console relay
  (still raw, capped; its witness needs the KVM job), the per-frame deadline, the accepted words in the death
  record; (3) E-04's remainder
  — Windows deny-write (needs a witness first), the attestation binding a digest the attester measured; (4) run
  3's inbox: PS-E-03 H6 (macOS and Windows under the escaped-guest harness — stopped by a classifier in run 5,
  `HANDOFF.md` §11.7; perhaps one for the laptop), the red-team pass on the filter, `setsid` for the guest, PS-E-01's remainder (`contained`'s set, attesters' claims as properties), E-05, E-06, then P8,
  P9. **For the owner:** D-V2-69, D-V2-70, D-V2-71 and D-V2-72 are this run's decisions; H6 was stopped
  by a safety classifier (§11.7).

### 2026-09-30 — routine run 6: CI read green; RW 4.32 closed — FRAME-DRIP-1 on the broker, a foreign call and the sandbox host (D-V2-73)
- Session: `https://claude.ai/code/session_01TQhMrCGGMnFjTrcKdQoS9Q`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the routine pushes there; each slice read on the other runners from
  `claude/friendly-thompson-c0cg57` first, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `dcf4fcb` (routine run 5's last commit)
- Commits: (1) `626c3c5` FRAME-DRIP-1 witnesses: a dribbled frame holds the broker's serve loop and a foreign call;
  (2) `12eeb51` FRAME-DRIP-1: a peer's frame is owed whole within a bound - the broker, a foreign call, the sandbox host (D-V2-73);
  (3) `11a152b` FRAME-DRIP-1's broker witness: on Windows a busy pipe refuses, so the operator asks again;
  (4) `1997997` FRAME-DRIP-1's records: D-V2-73, read red then green on macOS and Windows; routine run 6's entry — records only;
  (5) `34cfc38` PS-E-04 on Windows, witnessed: a launcher renamed over between the hash and the start is what runs;
  (6) `8aa6249` PS-E-04 on Windows: the launcher held open, sharing reads only, from the hash until the run ends (D-V2-74);
  (7) `9bf09b9` PS-E-04 on Windows, recorded: D-V2-74, red then green on a Windows runner — records only;
  (8) `68b8888` REQUEST-HANG-1, PROBE-DRIP-1: every round trip to the broker owes its whole answer within a bound (D-V2-75);
  (9) `3ca49e4` BROKER-RELDIR-1: broker start with a relative state dir served one level down and was left running;
  (10) `76aa676` The red-team pass on FRAME-DRIP-1, recorded: D-V2-75, BROKER-RELDIR-1, RW 4.35-4.40 — records only;
  (11) `5db4df1` REGISTRY-BOUNDS-1: delulu-registry serve bounds every connection; one slow client delays no other (D-V2-76);
  (12) `2734ba9` REGISTRY-BOUNDS-1, recorded: D-V2-76, read on Windows and macOS first — records only;
  (13) `cc3857b` AUDIT-FIFO-1: an audit day log is read only if it is a regular file (D-V2-77);
  (14) `cc157bc` Routine run 6 closed: AUDIT-FIFO-1 recorded; the loop's lessons — records only;
  (15) `efe76a0` RW 4.39: the broker wire refuses bytes after a frame's value;
  (16) `RW 4.39's frame reader recorded; the disk's stale test executables` — records only
- Files and folders:
  M `crates/delulu/src/brokerd.rs` (a witness) — (1); `crates/delulu/tests/foreign_worker.rs` (the `dl_drip` fixture, a witness) — (1)
  M `crates/delulu-runtime/src/channel.rs` (`Within`, `FrameTooSlow`, `FRAME_DEADLINE`; `HostChannel::with_frame_deadline`,
    `frame_deadline`; `serve` reads through `Within`; three unit witnesses), `crates/delulu/src/boundary.rs` (the
    confirmation report read through `Within`; a witness), `crates/delulu/src/brokerd.rs` (the serve loop; its comment
    made true), `crates/delulu/src/foreign_worker.rs` (the call and the bind), `crates/delulu/src/guest.rs` (`in_words`
    tells a slow frame from silence; a witness) — (2); `crates/delulu/src/brokerd.rs` (the witness asks again) — (3)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4 FRAME-DRIP-1; §11.5 a per-read deadline is not a per-message one; install the
    pinned toolchain first), `docs/CLOUD_ROUTINE.md` (step 2: the toolchain and its targets installed first, in the
    foreground), `docs/REMAINING_WORK.md` (4.32 closed; 4.14's note), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-73),
    `V2_LOG.md`, `V2_PHASE_STATUS.md` (PS-E), `V2_SECURITY_MODEL.md` (§10), `docs/assistant-memory/cloud-period-2026-09-28.md`,
    `docs/assistant-memory/delulu-ipc-deadman-findings.md` (IPC-1's dribble half) — (4)
  M `crates/delulu/tests/sandbox_external_cli.rs` (the Windows swap witness; `holders`) — (5);
    `crates/delulu/src/launcher.rs` (Windows: `share_mode(FILE_SHARE_READ)`; the module's note) — (6)
  M `docs/DEPLOYMENT.md` (*Which file runs*: Windows), `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` (§4.4), `CHANGELOG.md`,
    `HANDOFF.md` (§11.4; §11.5 Windows' `/proc/<pid>/fd`), `docs/REMAINING_WORK.md` (4.28), `V2_DECISION_LOG.md`
    (D-V2-74), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_SECURITY_MODEL.md` (§10), `docs/assistant-memory/cloud-period-2026-09-28.md` — (7)
  M `crates/delulu/src/brokerd.rs` (`REQUEST_DEADLINE`; `request` through `request_timed`; `request_timed` reads through
    `Within`; two witnesses) — (8); `crates/delulu/src/brokerd.rs` (`start_detached` makes the state dir absolute),
    `crates/delulu/tests/broker_cli.rs` (a witness) — (9)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4), `docs/REMAINING_WORK.md` (4.35–4.40, new), `V2_DECISION_LOG.md` (D-V2-75;
    D-V2-73's last line corrected), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_SECURITY_MODEL.md` (§10),
    `docs/assistant-memory/delulu-ipc-deadman-findings.md` — (10)
  M `crates/delulu-registry/src/lib.rs` (`Limits`, `serve_with`, a thread per connection, `bounded_line`, `refuse`);
    A `crates/delulu-registry/tests/serve_bounds.rs` (four witnesses) — (11)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4), `docs/REMAINING_WORK.md` (4.36 closed), `V2_DECISION_LOG.md` (D-V2-76),
    `V2_LOG.md`, `V2_PHASE_STATUS.md` — (12)
  M `Cargo.lock`, `crates/delulu-broker/Cargo.toml` (`libc`, Unix), `crates/delulu-broker/src/audit.rs` (`read_day`;
    a witness) — (13)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4; §11.5 the VM's disk, a Windows busy pipe), `docs/CLOUD_ROUTINE.md` (step 5:
    the disk; reading a witness job), `docs/REMAINING_WORK.md` (4.38), `V2_DECISION_LOG.md` (D-V2-77), `V2_LOG.md`,
    `V2_PHASE_STATUS.md` — (14)
  M `crates/delulu/src/broker_ipc.rs` (`read_frame` refuses trailing bytes; a witness) — (15); `CHANGELOG.md`,
    `docs/CLOUD_ROUTINE.md` (the disk: stale test executables), `docs/REMAINING_WORK.md` (4.39), `V2_LOG.md` — (16)
  M `docs/CLOUD_SYNC_LOG.md` — this entry; M `docs/survey/*` — regenerated — (2), (3), (4), (6)–(16)
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,471 nodes, 12,914 edges); `doctor --check` ok, 26 checks
  passed. After the last edit: in (4)'s commit message.
- Verified: **CI on arrival:** `dcf4fcb` `36608843586` — success on every job (16); no nightly since `36548984501`.
  `gh`: absent. `cargo deny` installed (0.20.2). **(1)–(3):** both end-to-end witnesses red on `dcf4fcb` in the VM (the
  broker's `Status` waited 23.7 s; the foreign call still read at 20 s) and on the runners at `626c3c5` (macOS
  `36651091801`: 24.25 s and 20.04 s; Windows `36651089599`: the `Status` refused, `win32 error 231`); M21–M27 red, each
  restored byte for byte; clippy clean; `check-other-os.sh` clean on five targets; the full suite alone 2,058 passed,
  0 failed, 15 ignored (153 binaries), cargo exit 0; green on the runners — macOS `36651763687` (bin 138, `foreign_worker`
  5) and `36651767914` (`delulu-runtime` `channel::` 22) at `12eeb51`, Windows `36651765721` (`channel::` 22) at
  `12eeb51` and `36652035525` (bin 132, `foreign_worker` 4) at `11a152b`; `36651761315` cancelled (its broker witness
  could not pass on Windows as first written). **(5)–(6):** the Windows swap witness red at `34cfc38` (`36652740569`: B
  started under A's name, exit 1), green at `8aa6249` (`36653274720`: `sandbox_external_cli` 5, `sandbox_confirm_cli` 8,
  `sandbox_attest_cli` 4, `sandbox_run_cli` 17); clippy clean; `check-other-os.sh` clean for Windows and macOS.
  **The red-team pass** (one Sonnet 5.5 sous-chef, a frozen binary at `11a152b`, scratch only — `git status` empty):
  the guarantee held on all three channels; its findings re-run by the head chef — see `V2_LOG.md`. **(8):** both
  witnesses red on `9bf09b9` (40.2 s; 6.0 s), M28–M30 red, the full suite alone 2,060 passed, 0 failed, 15 ignored
  (153 binaries); macOS `36655171914`, Windows `36655174168` green. **(9):** the witness red on `68b8888`, green after;
  the full suite alone 2,061 passed, 0 failed, 15 ignored, cargo exit 0; `broker_cli` green on Windows `36655636658`
  and macOS `36655638846`. **(11):** three witnesses red on `76aa676` (3.2 s; the crash; the endless line), M31–M34 red;
  the full suite alone 2,065 passed, 0 failed, 15 ignored (154 binaries), cargo exit 0; every `delulu-registry` target
  green on Windows `36657264839` and macOS `36657267414`. **(13):** the witness red on `2734ba9` (5 s), M35–M37 red; the
  full suite alone 2,066 passed, 0 failed, 15 ignored (154 binaries), cargo exit 0; `delulu-broker` lib green on macOS
  `36658766644 (170 passed, the witness among them)` and Windows `36658768992 (169 passed; the witness is Unix-only)`.
  **(15):** the witness red on `cc157bc`; the full suite alone 2,067 passed, 0 failed, 15 ignored (154 binaries), cargo exit 0; macOS `36659893368 (bin 141, the witness among them; broker_cli 4, foreign_worker 5)`, Windows `36659895529 (bin 133; broker_cli 3, foreign_worker 4)`.
- Redo on the laptop: the suite on Windows and in WSL (the broker's serve loop and the foreign worker's call changed on
  every OS; on Windows the external launcher is now held open sharing reads only for the run).
- For the laptop's memory: `HANDOFF.md` §11.4 (FRAME-DRIP-1) and §11.5 (a per-read deadline is not a per-message one;
  install the pinned toolchain before any concurrent `rustup`/`cargo`); `docs/assistant-memory/cloud-period-2026-09-28.md`
  and `delulu-ipc-deadman-findings.md`.
- Push runs read: (4) `1997997` `36652712551` — success on every job (16). (7) `9bf09b9` `36653912917` — success.
  (10) `76aa676` `36656353251` — success on every job (16). (12) `2734ba9` `36657621780` — complete, no failed job of 16.
  (14) `cc157bc` `36659209716` — still running at 02:37 UTC, no job failed by then — the next run reads it. (16), this entry's last commit: the next run reads it.
- Open / next: (1) read (14)'s push run, and the nightly after it — its `miri-slow` interprets `read_day` under Miri, the
  `O_NONBLOCK` flag left out there; (2) the red-team pass's open rows: **RW 4.35** (the broker's reply write — a whole-write
  bound; on Windows `WriteFile` blocks, so an overlapped write with a deadline), **4.37** (the host's writes to a guest —
  needs the escaped-guest harness to stop reading), **4.39** (the broker wire's lax frame reader; the LSP's and MCP's
  sizes), **4.40** (the broker's queue), and `secrets.rs`'s store read the FIFO way (4.38's rest); (3) E-04's remainder —
  the attestation binding; E-01's remainder; E-05 and E-06 (both need GitHub's runners: OpenShell's pinned release, the
  OCSF schema — neither is reachable from the VM); H6 (a classifier stopped it twice — perhaps with the owner); then P8
  (P8-01 first moves the device logic out of the interpreter, `V2_P8_DESIGN.md`), P9; (4) RW 7.17: `estop_cli`'s control
  test passed on arm64 in every push run this run read — keep watching. **For the owner:** D-V2-73 to D-V2-77 are this
  run's decisions; the red-team pass is in `V2_LOG.md`.

### 2026-09-30 — routine run 7: CI read green; PS-E-06 — the audit chain exported as OCSF 1.8.0, still verifiable (D-V2-78)
- Session: `https://claude.ai/code/session_01FNJXFsa6jKeh9Q9XiZsjFR`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (each slice read on the other runners from `claude/compassionate-pasteur-02il9s` first, then
  `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `7940188` (routine run 6's last commit)
- Commits: (1) `151dd8c` PS-E-06: the audit chain exported as OCSF 1.8.0, each event carrying its record (D-V2-78);
  (2) `b50bb36` PS-E-06's witness names the machine: the export with no --device-name, on every OS;
  (3) `9c58cbb` AUDIT-TEXT-1: what a program or an agent wrote is shown escaped in audit tail and guard pending (D-V2-79);
  (4) `5a2bbfe` PS-E-06 complete: a use's audit record names its effect, and the export maps it (D-V2-80);
  (5) `3d84304` ocsf.yml: the export checked against OCSF's published schema on a runner, by hand first;
  (6) `6c7aca3` RW 4.38's remainder: the secret store is read and written only if it is a regular file;
  (7) `634e9f0` RW 4.38 closed, recorded; ocsf.yml joins the push triggers, read green by hand;
  (8) `297f976` RW 4.42: grants tree, list and inspect show a delegation's strings escaped;
  (9) `eff5367` routine run 7 closed: RW 4.42 recorded; the loop's lessons;
  (10) `dca734a` RW 4.39's rest: delulu lsp and delulu mcp bound what one message may hold;
  (11) `e6913a8` RW 4.39's rest, recorded: the LSP's and MCP's bounds; master green through 634e9f0;
  (12) `5d3e37f` the live documents E-06 overtook: HANDOFF, the routine's step 4, the Book;
  (13) `53d96e3` E-05's install facts, for its workflow; master green through eff5367;
  (14) the run's last reading: master green through e6913a8; the listing quirk, again
- Files and folders:
  A `crates/delulu-broker/src/ocsf.rs` (the mapping, `export`, `verify`), `crates/delulu/tests/audit_ocsf_cli.rs` (seven
    witnesses), `scripts/ocsf-validate.py` (checks an export against the published OCSF schema, read at run time)
  M `crates/delulu-broker/src/audit.rs` (`AuditRecord::to_value`; `from_value` and `AuditError::corrupt` crate-visible),
    `crates/delulu-broker/src/lib.rs` (`pub mod ocsf`), `crates/delulu/src/cli.rs` (`audit export`, `audit verify --ocsf`,
    `host_name`, the help's two lines)
  M `CHANGELOG.md`, `docs/DEPLOYMENT.md` (§3: sending the chain to a SIEM; §7), `docs/REMAINING_WORK.md` (6.14 closed,
    its next step), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-78), `V2_IMPLEMENTATION_ROADMAP.md`,
    `V2_OPENSHELL_STUDY.md` (§4.6 built), `V2_LOG.md`, `V2_PHASE_STATUS.md` (PS-E)
  M `crates/delulu/tests/audit_ocsf_cli.rs` (the export with no `--device-name`) — (2)
  M `crates/delulu/src/cli.rs` (`render_audit_record`, `guard pending`: every field `terminal_line`),
    `crates/delulu/tests/audit_cli.rs`, `crates/delulu/tests/guard_e2e.rs` (a witness each), `CHANGELOG.md`, `HANDOFF.md`
    (§11.4 AUDIT-TEXT-1; §11.5 a stored string is printed later), `docs/REMAINING_WORK.md` (4.41 closed, 4.42 new),
    `V2_DECISION_LOG.md` (D-V2-79), `V2_LOG.md` — (3)
  M `crates/delulu-broker/src/validate.rs` (`check_use`'s records carry the op), `crates/delulu-broker/src/ocsf.rs`
    (File System and HTTP Activity from a named effect), `crates/delulu-broker/tests/audit_wiring.rs` (a witness),
    `crates/delulu/tests/audit_ocsf_cli.rs` (a network use and a record from before in the corpus), `CHANGELOG.md`,
    `docs/DEPLOYMENT.md`, `docs/REMAINING_WORK.md` (6.14's next step done), `docs/design/STAGE5_SPECIFICATION.md` (§11,
    chunk-5 deviation 3: the use's op in the same slot), `V2_DECISION_LOG.md` (D-V2-80), `V2_OPENSHELL_STUDY.md` (§4.6
    complete), `V2_LOG.md`, `V2_PHASE_STATUS.md` — (4)
  A `.github/workflows/ocsf.yml` (dispatch only: the OCSF witnesses leave their exports, `ocsf-validate.py --self-test`
    checks each); M `scripts/ocsf-validate.py` (`--self-test`: sixteen mutations the schema forbids, each must be
    caught), `crates/delulu/tests/audit_ocsf_cli.rs` (`DELULU_OCSF_CORPUS_OUT`), `docs/REPOSITORY_STRUCTURE.md` — (5)
  M `crates/delulu-broker/src/audit.rs` (`read_day` → `read_regular(path, what)`; `NotRegular` names what),
    `crates/delulu-broker/src/secrets.rs` (`read_store`; the Unix write; a witness) — (6); `CHANGELOG.md`, `HANDOFF.md`
    (§11.4), `docs/REMAINING_WORK.md` (4.38 closed), `V2_LOG.md`, `.github/workflows/ocsf.yml` (push, by paths) — (7)
  M `crates/delulu-broker/src/tree.rs` (`render_node`), `crates/delulu/src/cli.rs` (`render_node_line`, `grants
    inspect`), `crates/delulu/tests/grants_cli.rs` (a witness) — (8); `CHANGELOG.md`, `HANDOFF.md` (§11.4 AUDIT-TEXT-1;
    §11.5 a fake peer, again), `docs/CLOUD_ROUTINE.md` (step 5: the OCSF check; a real-run witness beside a fixture;
    overlap a slice's runner read with its suite), `docs/REMAINING_WORK.md` (4.42 closed), `V2_DECISION_LOG.md`
    (D-V2-79's note), `V2_LOG.md`, `V2_PHASE_STATUS.md` — (9)
  M `crates/delulu/src/lsp.rs` (`read_message`: `MAX_MESSAGE`, `MAX_HEADER_LINE`), `crates/delulu/src/mcp.rs` (`MAX_LINE`,
    `skip_line`), `crates/delulu/tests/lsp_cli.rs`, `crates/delulu/tests/mcp_cli.rs` (a witness each) — (10);
    `CHANGELOG.md`, `docs/REMAINING_WORK.md` (4.39: the log's words left), `V2_LOG.md` — (11)
  M `HANDOFF.md` (*Where things stand*: E-06 complete), `docs/CLOUD_ROUTINE.md` (step 4: the same),
    `docs/book/THE_DELULULANG_BOOK.md` (the audit paragraph: the OCSF export, still verifiable) — (12);
    `V2_OPENSHELL_STUDY.md` (§4.5: *For the workflow*) — (13); `docs/CLOUD_ROUTINE.md` (the listing quirk: a
    workflow- or branch-filtered listing returned hours-old runs twice more) — (14)
  M `docs/CLOUD_SYNC_LOG.md` — this entry; M `docs/survey/*` — regenerated — (1)–(14)
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,472 nodes, 12,976 edges); `doctor --check` ok, 26 checks passed.
  After the last edit: in each commit's message.
- Verified: **CI on arrival:** routine run 6's last two push runs, `cc157bc` `36659209716` and `7940188` `36660801170` —
  success on every job; no nightly since `36548984501`. `gh`: absent. `cargo deny` installed (0.20.2); advisories ok.
  **(1):** seven witnesses; M38–M43 red, each restored byte for byte; clippy clean; `check-other-os.sh` clean for macOS
  (arm64) and Windows; the full suite alone 2,074 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; a real
  sandboxed chain's export and the witnesses' corpus validated against OCSF 1.8.0 (0 problems), thirteen mutations of an
  export caught by the validator. **(2):** M44 (the machine unnamed) red; read before `master` moved (witness.yml,
  `audit_ocsf_cli audit_cli cli_contract json_contract` at `b50bb36`): macOS `36675486769` (`audit_ocsf_cli` 7,
  `cli_contract` 19, `json_contract` 14) and Windows `36675489012` (7, 19, 13) — success; `master` fast-forwarded to
  `b50bb36`. **(3):** witnessed end to end in the VM on `b50bb36` (a leased program's file name reached `audit tail` raw:
  `^[]0;PWNED^G^[[2J^[[31m`); both witnesses red on the unfixed binary, M45 and M46 red, restored; clippy clean; the full
  suite alone 2,076 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved (witness.yml,
  `audit_cli guard_e2e` at `9c58cbb`): macOS `36676784576` (12, 3) and Windows `36676787378` (12, 3) — success; `master`
  fast-forwarded to `9c58cbb`. **(4):** M47–M49 red, restored; the corpus validated (0 problems), five more export
  mutations caught; clippy clean; the full suite alone 2,077 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0. **(6)–(7):** the witness red on `3d84304`; M50–M52 red, restored; clippy clean; the full suite alone 2,078 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved at `6c7aca3`: macOS `36679046053` (`delulu-broker` lib 171, the witness among them) and `36679062748` (`secret_verify_cli`, `broker_cli`), Windows `36679048557` (lib; the witness is Unix-only) — success. **(8)–(9):** the witness red on `634e9f0`; M53–M55 red, restored; clippy clean; the full suite alone 2,079 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved at `297f976` (`grants_cli guard_e2e broker_cli`): macOS `36680482439` and Windows `36680485122` — success. **(10)–(11):** both witnesses red on `eff5367`; M56, M57 red, restored; clippy clean; the full suite alone 2,081 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved at `dca734a` (`lsp_cli mcp_cli`): macOS `36681858200` and Windows `36681860992` — success.
- Redo on the laptop: nothing beyond the suite (the host-name lookup is per OS: `gethostname` on Linux and macOS,
  `COMPUTERNAME` on Windows).
- For the laptop's memory: `HANDOFF.md` §11.3 (the OCSF schema's raw files are reachable from the VM; its tarball and
  `schema.ocsf.io` are not), §11.4 (AUDIT-TEXT-1 and RW 4.42; the secret store's FIFO), §11.5 (a string that is stored
  is printed later — ask where; a fake peer kept the wrong protocol again); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Push runs read: `b50bb36` `36675985008` — 16 jobs, none failed; `9c58cbb` `36677275938` — 16 jobs, none failed (macOS:
  egress, host loss and privilege floor established, filesystem reads and the memory ceiling absent, ping-pong NOT
  MEASURED — 3 hardware threads, 2.15x observed; Windows: all five established, ping-pong NOT MEASURED — the runner busy;
  the Linux jobs' notices not read this run). `ocsf.yml` `36678699774` (by hand, `3d84304`) — success, read from its log.
  `3d84304` `36678692933` and `634e9f0` `36680110785` — success; `ocsf.yml` on push: `634e9f0` `36680110806` and `eff5367`
  `36681499372` — success. `eff5367` `36681499405` and `e6913a8` `36682783273` (the LSP's and MCP's bounds' first push run) — complete, 16 jobs
  each, none failed. Unread when this entry was written: `5d3e37f`'s, `53d96e3`'s and this commit's push runs (records
  only) — the next run reads them first. No nightly since `36548984501` (none had fired by 07:00 UTC).
- Open / next: (1) read the push runs above, and the next nightly (its `miri-slow` interprets `read_regular` and the
  store's write — both leave out `O_NONBLOCK` under Miri); (2) **PS-E's rest:** E-05 — OpenShell as a tested L3 and
  `sandbox policy --format openshell` (by hand: an `openshell.yml` that downloads the pinned release on a runner, as the
  study's §4.5 says — its new *For the workflow* note has the install facts; nothing of OpenShell is ever committed); E-04's attestation binding (a digest the attester measured
  itself); E-01's remainder (attesters' claims as properties; `contained`'s set); E-03 H6 (a classifier stopped it twice —
  perhaps with the owner); then P8 (P8-01 first), P9; (3) the red-team rows still open: RW 4.35 (the broker's reply write
  — a whole-write bound; Windows `WriteFile` blocks), 4.37 (the host's writes to a guest — needs the escaped-guest
  harness to stop reading), 4.39's last item (the broker log's three wordings of one drop), 4.40 (the broker's queue); (4) RW 7.17 — keep
  watching arm64's `estop_cli`. **For the owner:** D-V2-78 to D-V2-80 are this run's decisions.

### 2026-09-30 — routine run 8: CI read green; SCOPE-HIDDEN-1 (D-V2-81); PS-E-05 (a) — the OpenShell export (D-V2-82)
- Session: `https://claude.ai/code/session_014qVST2xeSNjC4MRacqLH8g`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the VM's checkout was the harness branch `claude/stoic-ptolemy-1lso0u`, at `master`'s head; each
  slice was pushed there first, read on the other runners, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `381fed8` (routine run 7's last commit)
- Commits: (1) `290a741` SCOPE-HIDDEN-1: authority names a scope it cannot see, beside the literals it can (D-V2-81);
  (2) `15f68bf` PS-E-05 (a): delulu sandbox policy --format openshell writes the wall from the program's authority
  (D-V2-82); (3) `8c7894e` PS-E-05 (a): a Windows spelling is refused by name; the witnesses' inputs are platform rules;
  (4) `b50aa21` PS-E-05 (a): the export's flags belong to sandbox policy alone, refused on every other verb;
  (5) `7dfb566` EXPORT-CASE-1: a host grant DeluluLang would never match is omitted from the OpenShell export, not
  lowercased; (6) `804ddc8` PS-E-05 (a) recorded: the OpenShell export, D-V2-81 and D-V2-82; the loop's lessons;
  (7) `d447b29` openshell-prove.sh: the prover's JSON read from stdout alone; a granted read made writable is the
  filesystem falsifier; (8) `a3b2c90` openshell.yml: the prover's documented falsifiers; E-05 (b)'s first attempt as a
  runtime job, by hand; (9) `154310c` openshell-runtime.sh: the gateway pinned to Docker and registered before it is
  asked anything; (10) `9594e87` … a schema-version-2 gateway config, preflighted; the gateway registered once it has
  its TLS; (11) `625b6aa` … delulu built without embedded Python, as the Dockerfile builds it; OpenShell's deny lines
  read; (12) `cc3990f` … the image on the build host's glibc; the guest itself through an OpenShell sandbox (E-05 (b));
  (13) `cb96c40` … run 7's reading — the witnesses' own mistake fixed, the nested guest's refusal witnessed;
  (14) `74fb2b0` PS-E-05 (a) enforced by a real OpenShell sandbox, read green; OpenShell's deny line asserted; the nested
  guest recorded; (15) `af09d42` openshell-runtime.sh: OpenShell's deny line waited for with a bound; two more lessons
  for HANDOFF; (16) `adff7a8` routine run 8 closed: openshell.yml's prover job on push; the nightly green; the last
  readings; (17) the run's last reading — this commit
- Files and folders (`git diff --name-status 381fed8..HEAD`):
  A `crates/delulu/src/openshell.rs` (the export: `export`, `run_as`, the canonical-host and path rules, the YAML written
    from the JSON policy; five unit tests), `crates/delulu/tests/sandbox_openshell_cli.rs` (seven witnesses),
    `scripts/openshell-prove.sh` (the export checked by OpenShell's prover against boundaries written by hand, with
    each widening required to be caught), `.github/workflows/openshell.yml` (dispatch only: a pinned OpenShell release,
    checksum-verified on the runner and kept nowhere, runs the script) — (2)–(5)
  M `crates/delulu/src/cli.rs` (the scope walk's hidden kinds — (1); the `sandbox` help's export line — (2)),
    `crates/delulu/tests/cli.rs` (SCOPE-HIDDEN-1's witness) — (1)
  M `crates/delulu/src/main.rs` (`mod openshell`), `crates/delulu/src/sandbox.rs` (`--format`, `--grant`, `--workdir`,
    `--binary`, `--run-as` on `policy` alone; `Shaping`; `export_openshell`), `crates/delulu/src/schema.rs` (the
    `openshell` schema), `docs/REPOSITORY_STRUCTURE.md` (the script's and the workflow's rows) — (2)–(4)
  M `crates/delulu/tests/sandbox_attest_cli.rs` (attest's own refusal of `--grant`, exit 2 as before) — (4)
  M `CHANGELOG.md`, `HANDOFF.md` (§11.4 SCOPE-HIDDEN-1; *Where things stand*; §11.5 two lessons),
    `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-81, D-V2-82), `docs/DELULULANG_V2/V2_LOG.md` — (1), (6)
  M `docs/CLOUD_ROUTINE.md` (step 4: E-05 (a) built; step 5: rebuild after a mutant loop; a new workflow is dispatchable
    only on `master`), `docs/DEPLOYMENT.md` (the OpenShell paragraph: the export, how to use it, what is not built),
    `docs/REMAINING_WORK.md` (4.30 half built), `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` (the status line — it said
    nothing in §4–§6 was built; §4.5 (a) built), `V2_PHASE_STATUS.md` (PS-E), `V2_IMPLEMENTATION_ROADMAP.md` (PS-E-05),
    `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 8) — (6)
  A `scripts/openshell-install.sh` (the pinned release, checksums verified — shared by both jobs), `scripts/openshell-runtime.sh`
    (the export ENFORCED by a real OpenShell sandbox, and the nested guest's fail-closed refusal) — (8)–(14); M
    `scripts/openshell-prove.sh` (stdout alone; the prover's documented falsifiers) — (7), (8);
    M `.github/workflows/openshell.yml` (the shared install; the `runtime` input and job; the tag through `env`; the
    no-Python build) — (8), (11); M `docs/REPOSITORY_STRUCTURE.md` (the two scripts) — (8)
  M `docs/DELULULANG_V2/V2_LOG.md` (the workflow's eight runs), `V2_OPENSHELL_STUDY.md` (§4.5: read by the prover;
    *For (b)* facts), `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.30: the nested guest), `CHANGELOG.md`,
    `HANDOFF.md` (§11.5: choose a falsifier from the checker's documented cases; a flag documented for a command reaches
    every verb), `docs/CLOUD_ROUTINE.md` (step 5: the OpenShell check; step 7: wait on a run from the VM with `curl`),
    `docs/assistant-memory/cloud-period-2026-09-28.md` — (14)–(16); M `.github/workflows/openshell.yml` (on push, the
    prover job) — (16)
  M `docs/CLOUD_SYNC_LOG.md` — this entry; M `docs/survey/*` — regenerated — (1)–(14)
  Deleted: nothing.
- Survey and doctor (start of run): `survey check` ok (1,476 nodes, 13,071 edges); `doctor --check` ok, 26 checks passed.
  After the last edit: in each commit's message.
- Verified: **CI on arrival:** `5d3e37f` `36683068019`, `53d96e3` `36683327300`, `381fed8` `36684530677` — success; the
  nightly `36694905248` (`381fed8`) running, no job failed by 11:00 UTC (18 jobs; `miri-slow` still going). `gh`: absent.
  Run 7's witnesses re-run in the VM — all passed. **(1):** the witness red on `381fed8`; M58–M61 red, restored; clippy
  clean; the full suite alone 2,082 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master`
  moved: macOS `36703478342` (`cli` 50) and Windows `36703480939` (`cli` 51) — success; push run `290a741`
  `36704402505` — success, 16 jobs (arm64: 2,082 passed, all five properties, ping-pong MEASURED 2.93x against a
  control of 4.13x, passed; macOS: egress, host loss and privilege floor established, reads and memory not confined,
  ping-pong NOT MEASURED — 3 hardware threads, 2.18x; Windows: all five, ping-pong NOT MEASURED — the runner busy).
  **(2)–(5):** absent on `290a741` (unknown option, exit 2); M62–M74 red, each restored (the binary rebuilt after each
  loop); PyYAML reads the document as the JSON policy; runner reads: `15f68bf` macOS `36704827508` success, Windows
  `36704830856` RED on two tests (a `:` in a path; this machine's `C:\…` path) — fixed in (3), then Windows
  `36705354867`, macOS `36705357824`, arm64 `36705360430` success; the full suite alone at `8c7894e`: 2,091 passed,
  **1 failed**, 15 ignored (156 binaries), cargo exit 101 — which led to (4)'s regression (`sandbox status --grant x`
  ignored the flag, on the branch only); `b50aa21` Windows `36706294573`, macOS `36706297664` success, the suite alone
  2,093 passed, 0 failed, cargo exit 0; EXPORT-CASE-1 found by the head chef's adversarial pass and fixed in (5);
  `7dfb566` Windows `36707268456`, macOS `36707271273` success, the full suite alone 2,094 passed, 0 failed, 15 ignored
  (156 binaries), cargo exit 0; `master` fast-forwarded to `7dfb566`; `openshell.yml` dispatched at `master`.
  **`openshell.yml` (by hand, all at `master`):** run 1 `36708161562` (`7dfb566`) — the prover answered as designed, the
  SCRIPT red (stderr merged; an `unsupported` falsifier); run 2 `36708633558` (`d447b29`) — one falsifier `unsupported`;
  **run 3 `36709035019` (`a3b2c90`) — the prover job GREEN, every expectation held** (both exports `within_boundary`, six
  widenings `exceeds_boundary`, an unknown field an `error`), and the runtime job's first attempt red (no gateway
  registered; Podman preferred); runs 4 `36709806472`, 5 `36710715856`, 6 `36711475163`, 7 `36712236680` — the prover
  green each time, the runtime job red, each for the reason the next commit fixed (a schema-v1 config; the TLS order;
  libpython; glibc 2.39; the witness's own path mistake); **run 8 `36712891798` (`cb96c40`) — both jobs GREEN**: the
  granted program ran inside an OpenShell sandbox under the exported policy, a narrower grant DL0703, a file the export
  never names and `curl` to `example.org` refused by OpenShell (`NET:OPEN [MED] DENIED /usr/bin/curl(0) ->
  example.org:443 [reason:transparent_tcp_policy_denied]` read), the effective policy equal to the export both ways,
  and the nested guest failing closed (its `seccomp` refused, the program never sent). **Push runs:** `7dfb566`
  `36708157193`, `804ddc8` `36708404843`, `d447b29` `36708628013`, `a3b2c90` `36709029216`, `154310c` `36709802148` —
  success, and `9594e87` `36710708824`, `625b6aa` `36711469433`, `cc3990f` `36712231055` — success. **Run 9**
  `36713676712` (`74fb2b0`): the runtime job red on the new deny-line assertion alone (the log arrives asynchronously);
  **run 10 `36714033297` (`af09d42`): both jobs green**, the deny line asserted and present. **The nightly `36694905248`
  (`381fed8`): complete, every job success, `miri-slow` included.** **The last readings:** `cb96c40` `36712886514`,
  `74fb2b0` `36713671193`, `af09d42` `36714026123` — success; **`adff7a8` `36714603166` — 16 jobs, none failed**; and
  `openshell.yml`'s first PUSH run, `36714603164` at `adff7a8` — success (the prover job; the runtime job skipped, as on
  every push).
- Redo on the laptop: nothing beyond the suite (the export's Windows-spelling refusal is witnessed on the Windows runner).
- For the laptop's memory: `HANDOFF.md` §11.4 (SCOPE-HIDDEN-1), §11.5 (a mutant loop leaves the last mutant's binary;
  a workflow is dispatchable only once on the default branch); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next: (1) read this commit's push run (records only) — every earlier one of this run was read green;
  (2) **E-05 (b) — the guest inside OpenShell:** run 8 witnessed that the guest cannot run nested — its own `seccomp`
  filter is refused inside OpenShell's sandbox (EPERM), so it fails closed and the host never sends the program. A
  DECISION first (D-V2-nn): how the guest locks itself down under an outer wall that forbids adding a filter (e.g. accept
  an already-installed filter it can read back, or a launcher-declared outer wall that L3 already reports as `unknown`
  and the guest's words as `guest_reported`) — never a silent skip; then `scripts/openshell-runtime.sh`'s step (5)
  flips; `docs/DEPLOYMENT.md`'s recipe only after that is green; and whether the export's default `process` identity
  should come from the image (OpenShell's docs: a non-root `USER`, or numeric ids in the policy); (3) E-01's remainder —
  `contained`'s required set TOGETHER with attesters' claims as properties (the study's design; a change to the default
  profile); E-04's attestation binding; E-03 H6; then P8 (P8-01 first), P9 (P9-01 `authority --within` can build on
  D-V2-81's hidden kinds); (4) the red-team rows RW 4.35, 4.37, 4.39's last item, 4.40; (5) RW 7.17 — arm64's `estop_cli`.
  **For the owner:** D-V2-81 and D-V2-82 are this run's decisions.

### 2026-10-04 — routine run 10: the nightly red four times, fixed (D-V2-84); routine run 9's stranded PS-E-05 (b) recovered
- Session: `https://claude.ai/code/session_01H8s9DvSTxuHXSefjLqTeY2`   Model: Claude Opus 5.5 (the scheduled routine)
- Branch: `master` (the VM's checkout was the harness branch `claude/jolly-hamilton-w805xo`, at `master`'s head; each
  slice is pushed there first, then `master` moved)   Pull request: none   Merged: n/a
- Base: `e936ea5` (routine run 8's last commit — routine run 9's `f446bfa` never reached `master`)
- Commits: (1) `9bc405e` the nightly red four times, fixed (D-V2-84); (2) `83a62ba` merge of routine run 9's `f446bfa`
  (PS-E-05 (b), D-V2-83 — stranded on `claude/stoic-ptolemy-uyw9g4`); (3) `9e75017` PS-E-05 (b): the guest asks the
  kernel whether a filter is in force, the witness's wall hides `/proc`; (4) `0ed045f`, (5) `4cae1f9` `openshell-runtime.sh`:
  the exec relay measured; (6) `95a9789` the records of (2)–(5); (7) `fc676d1` `openshell-runtime.sh`: the guest's launcher
  over `ssh`; (8) `9b77312` PS-E-05 (b) complete: the recipe, D-V2-85; (9) `9b6b4a1` RW 4.35: the broker daemon's reply
  owed within a bound; (10) `2bb26f8` RW 4.35 on Windows withdrawn — its first design failed on the runner; (11) the
  records of (9)–(10), the sweep's corrections; (12) `4d4d033` RW 4.35 on Windows: written only where the pipe has room,
  flushed only once drained; (13) `b2bd3f2` its records; (14) `db73893` the loop's last lessons; (15) the last reading —
  this commit
- Files and folders: M `.github/workflows/ci.yml` (`cargo fetch --locked` before the `test` and `arm64` suites),
  `Cargo.lock` (wasmtime 48.0.5 and its family; yoke-derive 0.8.4), `CHANGELOG.md`, `HANDOFF.md` (§11.5: a gate that
  reads the whole graph offline), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-84), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/survey/*` (regenerated) — (1).
  From run 9's `f446bfa`, merged by (2): A `crates/delulu/tests/sandbox_outer_filter_cli.rs`; M `crates/delulu-runtime/src/channel.rs`
  (`OUTER_FILTER`), `crates/delulu/src/boundary.rs`, `guest.rs`, `jail.rs`, `microvm.rs`, `scripts/openshell-runtime.sh`,
  `docs/CLOUD_ROUTINE.md` (CI verdicts from the annotations API), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-83).
  M `crates/delulu/src/jail.rs` (`PR_GET_SECCOMP`), `crates/delulu/tests/sandbox_outer_filter_cli.rs` (two walls),
  `scripts/openshell-runtime.sh`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-83 amended) — (3); M
  `scripts/openshell-runtime.sh` (the relay measured) — (4), (5); M `CHANGELOG.md`, `HANDOFF.md` (§11.5: look at every
  branch; a simulated wall simulates every layer), `docs/CLOUD_ROUTINE.md` (step 1: every branch; step 6: the author),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` (§4.5: facts for (b)),
  `docs/REMAINING_WORK.md` (4.30), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*` — (6). M `scripts/openshell-runtime.sh` (the launcher over `ssh`) — (7). M `docs/DEPLOYMENT.md` (the guest
  inside OpenShell: the recipe), `CHANGELOG.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-85), `V2_LOG.md`,
  `V2_PHASE_STATUS.md` (E-05 complete), `V2_OPENSHELL_STUDY.md`, `V2_IMPLEMENTATION_ROADMAP.md`, `docs/REMAINING_WORK.md`
  (4.30 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (8). M `crates/delulu-runtime/src/channel.rs` (`Within` bounds a
  whole write: `FrameNotTaken`, `WRITE_CHUNK`), `crates/delulu/src/brokerd.rs` (the reply's bounds; two witnesses),
  `crates/delulu/src/broker_transport.rs` (Unix `set_write_timeout`; Windows as before after (10)) — (9), (10). M
  `CHANGELOG.md`, `HANDOFF.md` (§11.4 REPLY-HOLD-1; §11.5 a socket's write deadline is per buffer; *Where things stand*:
  E-05 complete), `docs/CLOUD_ROUTINE.md` (step 4: E-05 complete), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-86),
  `V2_LOG.md`, `docs/REMAINING_WORK.md` (4.35: closed on Linux and macOS, open on Windows), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*` — (11). M `crates/delulu/src/broker_transport.rs` (Windows: `set_write_timeout`, the quota-based write,
  flush and drop), `crates/delulu/src/brokerd.rs` (the witnesses on every OS) — (12); M `CHANGELOG.md`, `HANDOFF.md`,
  `docs/DELULULANG_V2/V2_DECISION_LOG.md`, `V2_LOG.md`, `docs/REMAINING_WORK.md` (4.35 closed), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*` — (13). M `docs/CLOUD_ROUTINE.md` (step 3: `gh` present, its token refused; step 5: time a stalled
  channel before designing a fix; `check-other-os.sh TARGET…`), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (14).
  M `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/survey/*` — (15). Deleted: nothing. **Authorship:** from (1), commits are authored by the owner's account through its
  GitHub no-reply address, Claude as co-author (his routine prompt); runs 1–9 committed as `Claude`.
- Survey and doctor (start of run): `survey check` ok (1,482 nodes, 13,114 edges); `doctor --check` ok, all checks
  passed. After the last edit: in each commit's message.
- Verified: **CI on arrival** — `e936ea5`'s push run `36716667440` success; the nightlies `36844480022` (10-01),
  `36988889261` (10-02), `37110949841` (10-03), `37191610614` (10-04) **red** — `supply-chain` (RUSTSEC-2026-0325/0326/0327,
  `yoke-derive` yanked) and from 10-02 every suite (`egress_features` on a cache Rust 1.99.0's release emptied). Both
  witnessed red in the VM, then green: cargo-deny exit 1 → 0; a cold `CARGO_HOME` red → green after `cargo fetch`. Clippy
  clean; the full suite alone 2,094 passed, 0 failed, 15 ignored (156 binaries), cargo exit 0; the two-engine
  differential by hand, passed. `gh`: present, its token refused — CI read with `curl` and the MCP tools. **(1)'s push run
  `37232688376` — success**, every job, from a cold cache (arm64 ping-pong MEASURED 2.93x against a control of 4.12x,
  passed; Linux x64 and Windows NOT MEASURED — the runner busy; macOS NOT MEASURED — 3 hardware threads; properties as
  before). **(2)–(3):** run 9's witnesses 4/4 green in the VM; the new witness red on `f446bfa`'s code (OpenShell's exact
  words), green after; M84–M88 red; clippy clean; `check-other-os.sh` clean (five targets); the full suite alone 2,102
  passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; `witness.yml` at `9e75017` — arm64 `37233188734`, macOS
  `37233190495`, Windows `37233191959`, Linux x64 `37233193360` success. **`openshell.yml` (by hand, on the branch):**
  `37233187076` (`9e75017`) — prover green; runtime: (1)–(4) and (5a) held, (5b) past the filter, then no channel;
  `37233707240` (`0ed045f`), `37234148291` (`4cae1f9`) — the relay measured: `sandbox exec` starts a command only when
  its input ends (with `--tty` too); `ssh` through `openshell ssh-proxy` streams. **(6)'s push runs:** `openshell`
  `37234832923` and `ocsf` `37234832899` success; CI `37234832894` — success, 16 jobs, none failed (arm64 ping-pong MEASURED 2.94x against a control of 3.61x; the other three NOT MEASURED). **(7) read inside a real OpenShell
  sandbox: `37234907676` (`fc676d1`) — both jobs GREEN**: (5a) the undeclared guest failed closed, the program never sent;
  (5b) the declared guest ran the program (exit 0), the report level 3 with "an outer syscall filter, not its own", every
  property `unknown` — "every expectation held". **(8)'s push runs:** `openshell` `37236037513` success; CI `37236037454` —
  success, 16 jobs, none failed. **The Sunday sweep:** `heavy-gates` dispatched at `9b77312` — `37236067731` success (9 min: the 100k fmt laws,
  the LSP latency gate and the two-engine differential, on Wasmtime 48.0.5); two documents' E-05 lines corrected.
  **(9)–(10):** both witnesses red on Linux before, green after; M89–M91 red; clippy clean; `check-other-os.sh` clean; the
  full suite alone 2,104 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0 at `9b6b4a1`; `witness.yml` macOS
  `37236820414` green, Windows `37236818679` red (its first design — withdrawn in (10)). **(11)'s push run** `37238153709` —
  success, 16 jobs, none failed. **The full suite alone at `b2bd3f2`: 2,104 passed, 0 failed, 15 ignored (157
  binaries), cargo exit 0.** **(12):** Windows lint clean; clippy clean; `witness.yml` at `4d4d033` — Windows `37238378527` and macOS `37238381140` green (Windows: 140 unit tests, the two witnesses among them, its log naming both drops — "the reader took nothing" and "did not take one whole frame within 5s" — and `broker_cli` 3, `estop_cli` 5, `guard_cli` 2, `guard_e2e` 3, `secret_verify_cli` 1). **The last push run, `db73893` `37239615098` (carrying (12)–(14)) — success, 16 jobs,
  none failed** (arm64 ping-pong MEASURED 2.85x against a control of 4.03x, passed; Linux x64 and Windows NOT MEASURED —
  the runner busy; macOS NOT MEASURED — 3 hardware threads; the five properties as on every run since run 8).
- Redo on the laptop: the suite on Windows and in WSL — the broker's reply bound on Windows (REPLY-HOLD-1) is read on the
  runner only; the laptop's Windows host is its second witness. OpenShell runs only in `openshell.yml`.
- For the laptop's memory: `HANDOFF.md` §11.4 (REPLY-HOLD-1), §11.5 (a gate that reads the whole resolved graph offline;
  look at every branch; a simulated wall simulates every layer of the real one; a socket's write deadline is per buffer);
  `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next: (1) read this commit's push run (records only) — every earlier one of this run was read green; (2) RW 4.35 is closed on every OS — RW 4.40 (silent connections queue
  behind one another on the broker) is the same loop's next row; (3) **PS-E-05 is complete**;
  left beside it: `protocol: rest` TLS behind OpenShell's CA when `delulu run` itself runs inside with a network grant;
  (4) run 8's list: E-01's remainder, E-04's attestation binding, E-03 H6, then P8
  (P8-01 first), P9; the red-team rows RW 4.37, 4.39's last item, 4.40; RW 7.17.
  **For the owner:** D-V2-84, D-V2-85 and D-V2-86 are this run's decisions, and D-V2-83's amendment; D-V2-83 itself is run 9's.

### 2026-10-05 — routine run 11: CI read green; PS-E-01 — an attester's claim that names a property meets it at level 3 (D-V2-87)
- Session: `https://claude.ai/code/session_01XsQvK7btrw8VtZ3z5bZRVJ`   Model: Claude Opus 5.5 (the scheduled routine; no
  fallback notice)
- Branch: `master` (the VM's checkout was the harness branch `claude/jolly-hamilton-dcf4b2`, at `master`'s head; each
  slice is pushed there first, read on the runners, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `2cb3f87` (routine run 10's last reading)
- Commits: (1) `3ec6abe` PS-E-01: an attester's claim that names a boundary property meets it at level 3 (D-V2-87);
  (2) `5bd76aa` its records; (3) `3ee18ac` RW 4.37: a guest that stops reading an answer cannot hold its host;
  (4) `608c5c4` the Survey regenerated for it (so the branch's CI run checks a fresh map); (5) `396653b` their records;
  (6) `3c74e97` PS-E-04: an attestation can bind the launcher its attester measured (D-V2-89); (7) `52f1d8e` its records
  and the run's close; (8) the last reading — this commit
- Files and folders: M `crates/delulu/src/attest.rs` (`named_property`, `Attested::vouches_for`),
  `crates/delulu/src/boundary.rs` (`Requirement.attested`, `met`, `properties(…, attested)`, the refusal's words; two
  tests), `crates/delulu/src/guest.rs` (the attestation passed to the requirement and the report),
  `crates/delulu/src/schema.rs` (`property.attested`, `attested_property`), `crates/delulu/src/cli.rs` (`sandbox attest`'s
  help), `crates/delulu-diag/src/codes.rs` (DL1408's explanation), `crates/delulu/tests/sandbox_attest_cli.rs` (the
  witness), `docs/DEPLOYMENT.md` (an attester vouching by name; the `hostile-agent` launcher), `docs/for-agents.md` — (1).
  M `CHANGELOG.md`, `HANDOFF.md` (*Where things stand*: Next rewritten; §11.5: a refusal's way out is a promise; a mutant
  must change the answer), `docs/CLOUD_ROUTINE.md` (step 4: E-01's state; step 5: a surviving mutant is first a question
  about the mutant), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-87), `V2_LOG.md`, `V2_PHASE_STATUS.md`,
  `V2_OPENSHELL_STUDY.md` (§4.1), `docs/REMAINING_WORK.md` (4.25), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/survey/*` (regenerated) — (2). M `crates/delulu-runtime/src/channel.rs`
  (`serve` writes each answer through `Within`; `FrameNotTaken::is`; a unit witness), `crates/delulu/src/guest.rs`
  (`bounded`, `bounded_by_name` — a write deadline on every guest socket; `in_words` for an answer not taken; the
  witness) — (3); M `docs/survey/*` — (4); M `CHANGELOG.md`, `HANDOFF.md` (§11.4 ANSWER-HOLD-1; §11.5 a bound on one
  direction asks the question of the other), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-88), `V2_LOG.md`,
  `docs/REMAINING_WORK.md` (4.37 closed), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*` — (5). M `crates/delulu/src/attest.rs` (`launcher_blake3`, `FORMAT_BOUND`, `WrongLauncher`,
  `--measure-launcher`; unit tests), `crates/delulu/src/boundary.rs` (test fixtures), `crates/delulu/src/guest.rs` (the
  started launcher's digest to the check; the stderr line), `crates/delulu/src/schema.rs`, `crates/delulu/src/cli.rs`
  (help), `crates/delulu/tests/sandbox_attest_cli.rs` (the witness), `docs/DEPLOYMENT.md`, `docs/for-agents.md`,
  `docs/survey/*` — (6); M `CHANGELOG.md`, `HANDOFF.md`, `docs/CLOUD_ROUTINE.md` (step 4: E-04's state; step 5: a branch CI run never names a unit test — `witness.yml` with a filter does),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-89), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md` (§4.4),
  `docs/REMAINING_WORK.md` (4.28), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*` — (7). M `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/survey/*` — (8). Deleted:
  nothing. **Authorship:** the owner's
  account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (start of run): `survey check` ok (1,483 nodes, 13,168 edges); `doctor --check` ok, all checks
  passed. After the last edit: in (2)'s message.
- Verified: **CI on arrival** — `2cb3f87`'s push run `37240818377` success, 14 jobs (arm64 ping-pong MEASURED 2.89x
  against a control of 4.06x; Linux x64 and Windows NOT MEASURED — busy; macOS NOT MEASURED — 3 threads; the properties as
  on every run since run 8); no nightly since run 10's reading. Every `claude/*` branch listed after unshallowing — none
  carries a commit `master` lacks beyond run 10's two recorded witness branches. **(1):** the witness red on `2cb3f87`
  (DL1408 with all five vouched for, the refusal naming the way out it refused), green after; M92–M97 red; clippy clean;
  `check-other-os.sh` clean for Windows and macOS; the full suite alone 2,108 passed, 0 failed, 15 ignored (157 binaries),
  cargo exit 0; `witness.yml` at `3ec6abe` — macOS `37247548752`, Windows `37247550461`, Linux arm64 `37247552077`, all
  success, the new tests named in each log. cargo-deny installed (not needed: supply-chain green on arrival).
  **(2)'s push runs:** CI `37248188675` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.84x against a control of 4.13x; the other three NOT MEASURED); `openshell` `37248188760`, `ocsf` `37248188651` success. **(3):** the
  witness red on `5bd76aa` (the host held 20 s by a guest that read nothing), green after; M98–M102 red (M101 by its
  unit witness); clippy clean; `check-other-os.sh` clean on all five targets; the full suite alone 2,110 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; CI
  dispatched at `608c5c4`, `37249221745` — every job read green but the two long `miri-slow` jobs (check, broker), cancelled once the rest were read: `test` on Linux, macOS and Windows, arm64, `microvm` (the KVM runner — a microVM guest's channel goes through `bounded` now), `microvm-reproducible`, `heavy-gates`, `miri-slow` (syntax), lints and the rest; and `witness.yml` naming the witness green on macOS `37250364770` (all four cases, 5.8 s) and Windows `37250366783` (the two by-name cases, over a named pipe, 1.8 s). **(5)'s push runs:** CI `37250662612` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.90x against a control of 4.10x; the other three NOT MEASURED); `ocsf` `37250662608`
  success. **(6):** absent on `396653b` (the reference attester refused `--measure-launcher`), the witness green after;
  M103–M107 red; clippy clean; `check-other-os.sh` clean for Windows and macOS; the full suite alone 2,112 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; read on the
  runners before `master` moved: `witness.yml` at `3c74e97` — macOS `37251102472`, Windows `37251104259`, Linux arm64 `37251106129`, all green, `sandbox_attest_cli`'s five attestation tests and the binary's attestation unit tests named in each log.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (a refusal's way out is a promise — run it; a mutant must change the
  answer); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next: (1) read this commit's push run (records only) — every earlier push run of this run was read green,
  `52f1d8e`'s last: CI `37251922235` success, 14 jobs (arm64 ping-pong MEASURED 2.93x against a control of 4.13x),
  `ocsf` `37251922186` and `openshell` `37251922219` success; (2) PS-E's rest — E-03 H6 (macOS and Windows under the escaped-guest harness;
  twice stopped by a classifier — perhaps with the owner), `contained`'s required set (macOS's gaps), E-04's
  macOS `fexecve`; then P8 (P8-01 first),
  P9 — **worth weighing first:** whether PS-E closes with those three named as residuals (H6 needs a session the classifier
  lets run, perhaps the owner's; `contained`'s set waits on macOS's reads and memory ceiling; macOS has no `fexecve`), so
  P8-01 can start — a decision for a `D-V2-nn`, taken with P8-01's readiness read; (3) the red-team rows RW 4.39's log
  words, 4.40 (silent connections queue on the broker); RW 7.17.
  **For the owner:** D-V2-87, D-V2-88 and D-V2-89 are this run's decisions.

### 2026-10-05 — routine run 12: CI read green; PS-E-01 — a macOS guest's memory ceiling is the host's sampler (D-V2-90); RUNDIR-PERM-1 (D-V2-91)
- Session: `https://claude.ai/code/session_017EaoWq1fAD7PwrTrfFTaL3`   Model: Claude Opus 5.5 (the scheduled routine; no
  fallback notice)
- Branch: `master` (the VM's checkout was the harness branch `claude/jolly-hamilton-vwdips`, at `master`'s head; each
  slice pushed there first, read on the runners, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `ed95f0d` (routine run 11's last reading)
- Commits: (1) `d9d4803` the witness alone (a macOS guest past its memory budget, red on a runner); (2) `8e85b94` PS-E-01:
  a macOS guest's memory ceiling is the host's sampler (D-V2-90); (3) `2b37c23` mutant M108, pushed only to be read red on
  macOS; (4) `09a113f` its revert; (5) `550ba38` RUNDIR-PERM-1's witness alone; (6) `3c6e67b` RUNDIR-PERM-1's fix, the
  sampler at 5 ms, the map; (7) `07045d5` `make_run_dir` split by OS (Windows `unused_mut`); (8) `4471d6f` the records;
  (9) `0560898` PS-E-01: `contained` requires egress, resource and host-loss confinement of a boundary this host measured
  (D-V2-92); (10) `8cbd1e0` its records, PS-E closed (D-V2-93); (11) `7c55af9` RW 4.40: silent connections no longer
  queue the broker's clients (D-V2-94); (12) `101cb1e` the daemon's new threads classified in the stack gate, and
  `scripts/ci-log-summary.py`; (13) `633437e` their records; (14) `77d436e` RW 4.40's own bound — a reader counts until
  its request is handed over; (15) `89f69c8` its records and the run's close; (16) `f76eafd` RW 4.39: a dropped request
  is logged in words that say what the client did; (17) `11ef757` its records; (18) the last reading, and P8-01's build
  order — this commit
- Files and folders: M `crates/delulu/src/jail.rs` (`SAMPLED_MEMORY_CEILING`, `peak_footprint`, a macOS unit test, the
  RLIMIT_DATA comment), `crates/delulu/src/guest.rs` (`Fired::Memory`, `MemReader`, the watchdog's memory reading at
  `MEMORY_TICK`, the claim, the named stop; `make_run_dir` and its unit test), `crates/delulu/src/boundary.rs` (the macOS
  properties fixture; the `PROPERTIES` comment), `crates/delulu/src/policy.rs` (a comment),
  `crates/delulu/tests/sandbox_run_cli.rs` (two witnesses; macOS's memory now enforced), `crates/delulu/tests/guest_cli.rs`
  (the macOS claim) — (1)–(7). M `CHANGELOG.md`, `HANDOFF.md` (*Where things stand*: Next; §11.4 run 12; §11.5 three
  lessons), `docs/CLOUD_ROUTINE.md` (step 5: mutants on a runner, lint after the last edit, read a green witness whole, a
  short Windows log), `docs/DEPLOYMENT.md` (the macOS row), `docs/MATHEMATICS.md` (the claim table),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-90, D-V2-91), `V2_LOG.md`, `V2_PHASE_STATUS.md`, `V2_OPENSHELL_STUDY.md`
  (§4.1), `docs/REMAINING_WORK.md` (4.25), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`
  (this entry), `docs/survey/*` (regenerated) — (8). M `crates/delulu/src/policy.rs` (`contained`'s three,
  `requires_proof`), `crates/delulu/src/boundary.rs` (the rule and its refusal; the witness; the channel tests on `dev`),
  `crates/delulu-diag/src/codes.rs` (DL1408's explanation), `docs/for-agents.md`, `skills/delulu/SKILL.md`, `docs/survey/*`
  — (9). M `CHANGELOG.md`, `HANDOFF.md` (*Where things stand*), `docs/CLOUD_ROUTINE.md` (step 4: PS-E complete, P8 next),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-92, D-V2-93), `V2_LOG.md`, `V2_PHASE_STATUS.md` (PS-E complete),
  `V2_OPENSHELL_STUDY.md` (§4.1), `docs/REMAINING_WORK.md` (4.25 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (10).
  M `crates/delulu/src/brokerd.rs` (the accept thread, the readers, the one handler; the witness),
  `crates/delulu/src/broker_transport.rs` (the Windows `Listener` is `Send`), `docs/survey/*` — (11). M
  `crates/delulu-runtime/src/actors.rs` (the stack gate's list: `brokerd.rs`; `guest.rs`'s reason), A
  `scripts/ci-log-summary.py`, M `docs/REPOSITORY_STRUCTURE.md` (its row), `docs/survey/*` — (12). M `CHANGELOG.md`,
  `HANDOFF.md` (§11.4 SILENT-QUEUE-1; §11.5 a deadline per connection on a serial loop), `docs/CLOUD_ROUTINE.md` (step 3:
  the log summary; step 5: freeze the tree — a script added mid-suite), `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-94), `V2_LOG.md`, `docs/REMAINING_WORK.md` (4.40 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (13).
  M `crates/delulu/src/brokerd.rs` (the count released after the handover; the flood witness), `docs/survey/*` — (14).
  M `CHANGELOG.md`, `HANDOFF.md` (§11.5 a bound you add is a resource you count), `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-94 amended), `V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (15). M `crates/delulu/src/brokerd.rs`
  (`dropped_because`; the witness), `docs/survey/*` — (16). M `CHANGELOG.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/REMAINING_WORK.md` (4.39 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (17). M
  `docs/DELULULANG_V2/V2_P8_DESIGN.md` (P8-01's build order, read against the code),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `V2_LOG.md`, `docs/survey/*` — (18).
  Added: `scripts/ci-log-summary.py`. Deleted: nothing. **Authorship:** the owner's account
  through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (start of run): `survey check` ok (1,483 nodes, 13,229 edges); `doctor --check` ok, all checks passed.
  After the last edit: `survey check` ok (1,483 nodes, 13,229 edges), `findings` 0 errors, 14 warnings; `doctor --check`
  ok, all checks passed.
- Verified: **CI on arrival** — `ed95f0d`'s push run `37253089773` success, 14 jobs (arm64 ping-pong MEASURED 2.93x against
  a control of 4.10x; Linux x64 and Windows NOT MEASURED — busy; macOS NOT MEASURED — 3 threads; the properties as on every
  run since run 8); no nightly since run 10's reading. Every `claude/*` branch listed after unshallowing — none carries a
  commit `master` lacks beyond run 10's two recorded witness branches. Run 11's tests re-run in the VM: `sandbox_attest_cli`
  9, the attest/boundary/guest unit tests 20, `delulu-runtime`'s channel tests 23 — all passed. **The slice:** red on macOS
  first (`witness.yml` `37267847333` — stopped by `wall=`, `memory: not confined`); M108 red on macOS (`37268731314`, three
  tests); RUNDIR-PERM-1 red on macOS (`37268778120`); M109, M110 red in the VM; clippy clean; `check-other-os.sh` clean for
  Windows and macOS (twice — after the last code edit too); the full suite alone 2,115 passed, 0 failed, 15 ignored (157
  binaries), cargo exit 0; read green before `master` moved: at `8e85b94` Windows `37268385858`, arm64 `37268387716`, macOS
  `37268383827` (the whole package green but `doctor_cli`'s stale map); at `3c6e67b` macOS `37269100267` (the whole `delulu`
  package, 877 passed, 0 failed), Windows `37269102715`, arm64 `37269105326`; at `07045d5` Windows `37269926751` (43 passed,
  no warning), macOS `37269929073` (193 passed). The memory stop observed 247 MB against 64 MiB at 25 ms, then 68.8 MB and
  92.8 MB at 5 ms. **(8)'s and (7)'s push runs:** CI `37270189858` (`07045d5`) success, 14 jobs (arm64 ping-pong
  MEASURED 2.80x against a control of 4.12x; the other three NOT MEASURED; `resource_ceiling` established on all four
  runners, macOS's included — the first push run to say so), `ocsf` `37270189893` success; CI `37270454522` (`4471d6f`)
  success. **(9):** red first (`boundary`'s unit witness on `4471d6f`: "no memory ceiling: confirmed"); M112–M116 red;
  clippy clean; the full suite alone 2,116 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; CI dispatched at
  `0560898`, `37271099512` — every job green (`test` on all three OSes, arm64, `microvm` under `contained`,
  `microvm-reproducible`, `heavy-gates`, lints and the rest) but the three long `miri-slow` jobs, cancelled once the rest
  were read; macOS's properties: four established, `filesystem_confinement` absent. **(10)'s push run:** CI
  `37272692095` (`8cbd1e0`, carrying (9)) success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.86x against a
  control of 4.07x; the other three NOT MEASURED). **(11):** red first in the VM on `8cbd1e0` (six silent connections
  held a `Status` 30.6 s); M118 red (4.8 s); clippy clean; `check-other-os.sh` clean for Windows and macOS; read on the
  runners at `7c55af9` — Windows `37273275501` (160 passed), macOS `37273278082` (175 passed), Linux arm64 `37273280678`
  (193 passed), the daemon's unit tests and six daemon-facing targets, the witness named in each. **The full suite
  caught what the runners did not read:** the runtime's thread-stack gate refused the daemon's two new threads,
  unclassified (`actors.rs`, red in the VM's suite — the reads ran `delulu`'s targets, not `delulu-runtime`'s); classified
  in (12), the gate falsified by removing the entry. That suite also lost `doctor_cli`'s three tests to a script written
  into the tree while it ran (the map went stale) — the tree was frozen and the suite re-run: 2,117 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0.
  **(12)'s and (13)'s push runs:** CI `37274536930` (`101cb1e`) and `37274733834` (`633437e`) success. **(14):** found by
  re-reading (11) on `master`: red first on `101cb1e` (300 whole requests behind a busy handler: 4 → 240 threads); M119
  red; the 27 daemon unit tests; clippy and `check-other-os.sh` clean; the full suite alone 2,118 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; read on the
  runners: `witness.yml` at `77d436e` — Windows `37275973144` (158 passed), macOS `37275975885` (173 passed), Linux arm64 `37275979053` (187 passed, the flood witness among them), the daemon's unit tests, `broker_cli` and `estop_cli` (and `guard_cli`, `guard_e2e` on Windows and macOS). **(16):** red first on `89f69c8`; M120, M121 red; clippy and `check-other-os.sh` clean; the full
  suite alone 2,119 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; read on the runners: `witness.yml` at `f76eafd` — Windows `37277293552` (149 passed), macOS `37277296503` (164 passed), the daemon's unit tests and `broker_cli`, the witness named in each. **Push runs read:** `101cb1e` CI `37274536930` and `633437e` CI `37274733834` success; `77d436e` CI `37276656840` and `89f69c8` CI `37276819686` in progress at the close of (17). **The last push runs** (`f76eafd`'s
  and this commit's): read by the run's last reading, or by the next run first. **The last reading:** every push run of the run read green — CI `37276656840` (`77d436e`), `37276819686` (`89f69c8`), `37277912297` (`f76eafd`), `37278057068` (`11ef757`): success, 14 jobs each, none failed (arm64 ping-pong MEASURED 2.87x/4.07x, 2.90x/4.12x, 2.96x/4.06x, 2.92x/4.04x; the other three NOT MEASURED); `master` green through `11ef757`. The 2026-10-05 nightly (cron 03:00, fired about 09:15 on recent days) had not fired by 07:46.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (a sampler's interval is its ceiling's resolution; a false alarm is a defect
  — read a green witness whole, lint after the last edit; `git revert` takes no `-q`); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next: (1) read this commit's push run (records only) and the 2026-10-05 nightly; (2) **P8-01** — PS-E is complete (D-V2-93): first move the device
  operations (`Interp::call_actuator`/`call_sensor`, the envelope check, C39's refused-attempt sweep, the DL1904 records)
  behind one function the interpreter and `HostChannel` both call, the `DeviceBroker` handed to the host channel as custody
  is — `V2_P8_DESIGN.md` "Where the work is"; (3) PS-E's residuals, kept open: H6 (RW 4.27, perhaps with the owner), macOS's
  launcher window (RW 4.28), macOS's reads; (4) RW 7.17.
  **For the owner:** D-V2-90, D-V2-91, D-V2-92 (`contained`'s set — a Windows host that cannot make the AppContainer, or a
  macOS host whose watcher or sampler cannot start, now refuses the default profile) and D-V2-93 (PS-E closed with three
  residuals) and D-V2-94 (SILENT-QUEUE-1) are this run's decisions.

### 2026-10-05 — routine run 13: CI read green; P8-01 — a control program runs in a guest (D-V2-95)
- Session: `https://claude.ai/code/session_018jd8mvrdzcUzN8vj2bcFgY`   Model: Claude Opus 5.5 (the scheduled routine; no
  fallback notice)
- Branch: `master` (the VM's checkout was the harness branch `claude/jolly-hamilton-mcawx1`, at `master`'s head; each slice
  pushed there first, read on the runners, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `f912084` (routine run 12's last reading)
- Commits: (1) `76ba9ca` P8-01: a control program runs in a guest — the host performs its device operations; (2) `45e65eb`
  a hardware run's flags refused without one; the witnesses hold on a slow runner and on macOS; (3) `6796c31` the records,
  `scripts/suite.sh` and this entry; (4) `07f6cd9` P8-01 at level 2 (a KVM-gated witness); (5) `666cb53` mutant M128,
  pushed only to be read red on the KVM runner; (6) `fee5353` its revert; (7) `5b89f9f` a sandboxed simulation keeps the
  stepped clock and signs off; (8) `521d688` their records; (9) `bc8bd91` a sandboxed run reports its devices' journal;
  (10) `ac31cf2` its records, P8-01 complete; (11) `5c98e65` P8-02's build order; (12) `418dcca` P8-02 steps 2–3: the
  driver slot is an `Adapter` trait, the DIR asserted `Send`; (13) `e561983` its records and the closing reading; (14) the
  last reading and RW 4.7's pointer to P8-02 — this commit
- Files and folders: M `crates/delulu-runtime/src/device.rs` (`actuate`, `sense`, `refusal_detail`, `command_check`,
  `command_fields`), `crates/delulu-runtime/src/interp.rs` (`call_actuator`/`call_sensor` call them; a handle goes to the
  sink; the old `envelope_check`/`numeric_fields` removed), `crates/delulu-runtime/src/channel.rs`
  (`HostChannel::with_devices`; the device arm after the custody gate), `crates/delulu/src/run_cmd.rs` (`device_terms`,
  `watch_devices`/`DeviceWatch`, `DevicePlan`/`plan_devices`, `close_devices`; the hardware-flag refusal),
  `crates/delulu/src/guest.rs` (`CARRIED`, the allowlist, `GuestDevices`, the broker started at the send),
  `crates/delulu/src/doctor.rs`, `crates/delulu-diag/src/codes.rs` (DL1408's explanation), `crates/delulu/tests/atlas_chain.rs`,
  `crates/delulu/tests/sandbox_modes_cli.rs`, `docs/MATHEMATICS.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/for-agents.md`, `docs/survey/*`; A `crates/delulu/tests/sandbox_devices_cli.rs` — (1)–(2). M `CHANGELOG.md`,
  `HANDOFF.md` (*Where things stand*: P8-01; §11.5 four lessons), `docs/CLOUD_ROUTINE.md` (step 3: `--bin delulu`; step 4:
  P8-01 built; step 5: `scripts/suite.sh`, a witness log's size), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-95),
  `V2_LOG.md`, `V2_PHASE_STATUS.md` (P8 in progress), `V2_P8_DESIGN.md` (P8-01 built, three departures),
  `docs/REPOSITORY_STRUCTURE.md` (`suite.sh`'s row), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/survey/*`; A `scripts/suite.sh` — (3). M `crates/delulu/tests/microvm_cli.rs`,
  `docs/survey/*` — (4); M `crates/delulu/src/guest.rs` — (5), (6), (7); M `crates/delulu/tests/sandbox_devices_cli.rs`,
  `crates/delulu/tests/sandbox_modes_cli.rs`, `docs/survey/*` — (7); M `CHANGELOG.md`, `docs/CLOUD_ROUTINE.md`,
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-95 item 10), `V2_LOG.md`, `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (8); M `crates/delulu/src/guest.rs` (the report's `devices`),
  `crates/delulu/src/schema.rs` (`device_event`), `crates/delulu/tests/sandbox_devices_cli.rs`, `docs/for-agents.md`,
  `docs/survey/*` — (9); M `CHANGELOG.md`, `HANDOFF.md`, `docs/CLOUD_ROUTINE.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-95 item 11), `V2_LOG.md`, `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (10); M `docs/DELULULANG_V2/V2_P8_DESIGN.md` (P8-02's build order),
  `V2_LOG.md`, `docs/CLOUD_ROUTINE.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (11); M
  `crates/delulu-runtime/src/adapter.rs` (`Adapter`, the `Send` assertion), `crates/delulu-runtime/src/device.rs`,
  `crates/delulu/src/run_cmd.rs`, `docs/survey/*` — (12); M `docs/DELULULANG_V2/V2_LOG.md`, `V2_P8_DESIGN.md`,
  `V2_PHASE_STATUS.md`, `docs/CLOUD_ROUTINE.md`, `docs/CLOUD_SYNC_LOG.md`, `HANDOFF.md` (§11.5), `docs/survey/*` — (13); M
  `docs/REMAINING_WORK.md` (4.7), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (14). Deleted:
  nothing. **Authorship:** the
  owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (start of run): `survey check` ok (1,484 nodes, 13,281 edges); `doctor --check` ok, all checks passed.
  After the last edit: see the closing reading below.
- Verified: **CI on arrival** — `f912084`'s push run `37279799389` success, 14 jobs (arm64 ping-pong MEASURED 2.91x against a
  control of 4.12x; Linux x64 and Windows NOT MEASURED — busy; macOS NOT MEASURED — 3 threads; macOS
  `filesystem_confinement` absent, the rest established, as on every run since run 8); the 2026-10-05 nightly
  `37293332206` on `f912084` — every job green but the three `miri-slow` jobs, still running when last read. No `claude/*`
  branch carries a commit `master` lacks beyond run 10's two recorded witness branches. Run 12's daemon tests re-run: 28
  passed. **The slice:** red first — mutant M122 restores `master`'s "it uses Actuator. Nothing ran." and turns all seven new
  witnesses red; M123b, M124, M125, M126, M127 red; M123 survives on the broker's own envelope check (recorded, D-V2-95);
  the 46 device tests of the ordinary run and the runtime's 243 unit tests green after the move; clippy clean;
  `check-other-os.sh` clean for Windows and macOS twice; read on the runners before `master` moved — at `76ba9ca` Windows
  `37298032325` (73 passed) and arm64 `37298039715` (74) green, macOS `37298035733` red on the e-stop witness's own
  socket path (113 bytes; fixed in `45e65eb`); at `45e65eb` macOS `37299710143` (47), Windows `37299713468` (46), arm64
  `37299716481` (47), all eight device witnesses named on each. The full suite alone: first 2,124 passed, 2 failed (the
  Survey's freshness test, a test file edited mid-suite; and `sandbox_modes_cli`, which found the unapplied `--approved` —
  hardened in `45e65eb`); then 2,127 passed, 0 failed, 15 ignored (158 binaries), cargo exit 0. **Push runs:** `45e65eb`
  (carrying `76ba9ca`) CI `37300708685` success, 14 jobs (arm64 MEASURED 2.93x/4.12x), `ocsf` `37300708671` success;
  `6796c31` CI `37301082817` success, 14 jobs (arm64 MEASURED 2.90x/4.11x). **(4)–(6), level 2:** CI dispatched at
  `07f6cd9` (`37301435891`) — its `microvm` job green, 12 passed, the new witness named — then cancelled; at `666cb53`
  (`37301599674`) M128 red on the KVM runner (the VM's boot revoked the 150 ms lease first; it survives every L1 witness),
  cancelled, reverted. **(7):** M129–M132 red; clippy and `check-other-os.sh` clean; read at `5b89f9f` on macOS
  `37303092995`, Windows `37303096425`, arm64 `37303099029` (10 + 11 passed each); the full suite alone through
  `scripts/suite.sh` 2,129 passed, 0 failed, 16 ignored (158 binaries), cargo exit 0, the tree unmoved. **(9):** M133 (no
  journal) and M134 (the schema not told) red; clippy and `check-other-os.sh` clean; read at `bc8bd91` on macOS `37305292797`
  (28 passed), Windows `37305296588`, arm64 `37305299628`; the full suite alone through `scripts/suite.sh` 2,129 passed, 0
  failed, 16 ignored, cargo exit 0, the tree unmoved. **Push runs read:** `fee5353` CI `37302336639` success (its `microvm`
  job ran the L2 witness on `master`); `5b89f9f` CI `37304139892` success (arm64 2.97x), `ocsf` success; `521d688` CI
  `37304345347` success (arm64 2.92x) — 14 jobs each, none failed. **The last push runs** (`bc8bd91`'s and this commit's):
  read by the run's closing reading, or by the next run first. **(12):** M135 red; the `Send` assertion falsified (E0277);
  clippy and `check-other-os.sh` clean; read on the runners at `418dcca` before `master` moved — Windows `37307415352` and macOS `37307419324`, `hw_adapter_cli`, `dead_man_cli` and `sandbox_devices_cli`, green; the full suite alone through `scripts/suite.sh`, 2,129 passed, 0 failed, 16 ignored (158 binaries), cargo exit 0, the tree unmoved. **The closing reading:** `bc8bd91` — CI `37306161197` success, 14 jobs (arm64 MEASURED 2.88x), `ocsf` `37306161210` success; `ac31cf2` — CI `37306380748` success, 14 jobs (arm64 MEASURED 2.92x); `5c98e65`'s, `418dcca`'s and this commit's push runs were in progress at the close — the next run reads them first, with the 2026-10-05 nightly's last `miri-slow` job (`delulu-broker`; `delulu-syntax` and `delulu-check` read green).
  **The last reading:** every push run of the run read green — CI `37306707736` (`5c98e65`), `37308525902` (`418dcca`),
  `37308742427` (`e561983`): success, 14 jobs each, none failed (arm64 ping-pong MEASURED 2.93x/4.11x, 2.86x/4.12x,
  2.97x/4.12x); `master` green through `e561983`. The 2026-10-05 nightly `37293332206`: every job green but
  `miri-slow (delulu-broker)`, still running at 12:36 (`delulu-syntax` and `delulu-check` green).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (a mutant that survives on a second wall; a mutant only another level can see; a flag a path starts to apply asks
  the dropped-flag question again; the tree edited mid-suite a third time — `scripts/suite.sh`; `delulu` has no `--lib`);
  `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next: (1) read this commit's push run (records only) and the 2026-10-05 nightly's `miri-slow (delulu-broker)`;
  (2) **P8-02** — the Verified-class adapter as a `.dpx`: its build order is in `V2_P8_DESIGN.md` (written by this run
  from the code); steps 2–3's groundwork is done (`418dcca`) — start at step 3 proper, the `VerifiedAdapter`'s thread
  (P8-01 is complete, D-V2-95); (3) PS-E's residuals, kept open: H6 (RW 4.27), macOS's launcher window (RW
  4.28), macOS's reads; (4) RW 7.17. **For the owner:** D-V2-95 (P8-01, and a hardware run's flags refused without one —
  a behaviour change for the ordinary run) is this run's decision.

### 2026-10-05 — routine run 14: CI read green; P8-02 steps 3–4 — a Verified driver's logic (D-V2-96)
- Session: `https://claude.ai/code/session_015GbcmhgsYDQj97EaKh65ea`   Model: Claude Opus 5.5 (the scheduled routine; no
  fallback notice)
- Branch: `master` (the VM's checkout was the harness branch `claude/jolly-hamilton-7fd30a`, at `master`'s head; each slice
  pushed there first, read on the runners, then `master` fast-forwarded)   Pull request: none   Merged: n/a
- Base: `7a895f0` (routine run 13's last reading)
- Commits: (1) `45d1181` P8-02 steps 3–4: a Verified driver's logic on a thread of its own, the host's transport; (2) this
  entry and the records
- Files and folders: A `crates/delulu-runtime/src/verified_adapter.rs`; M `crates/delulu-runtime/src/lib.rs`,
  `crates/delulu-runtime/src/plugin.rs` (`effective_interp_limits` crate-visible), `docs/survey/*` — (1). M
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-96), `V2_P8_DESIGN.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*` — (2). Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address,
  Claude as co-author.
- Survey and doctor (start of run): `survey check` ok (1,486 nodes, 13,391 edges); `doctor --check` ok, all checks passed.
- Verified: **CI on arrival** — `7a895f0`'s push run `37310919853` success, 14 jobs (arm64 MEASURED 2.87x/4.11x); the
  2026-10-05 nightly `37293332206` success on every job, `miri-slow (delulu-broker)` included. **The slice:** eight
  witnesses; M136–M144 red but M143 (survives on a second wall; M143b red); clippy clean; read on the runners at `45d1181`
  — arm64 `37333709607`, Windows `37333705883`, macOS `37333701001`, 8 passed each; the full suite alone 2,137 passed, 0 failed, 16 ignored (158 binaries), cargo
  exit 0, the tree unmoved.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing yet.
- Open / next: **P8-03** (the simulator as a device behind the new transport, across a real process boundary) and
  **P8-04** (the out-of-band monitor); then P9. PS-E's three residuals stay open (H6/RW 4.27, macOS's launcher window/RW
  4.28, macOS's reads).

### 2026-10-05 — routine run 14 (continued): P8-02 complete — a signed Verified driver (D-V2-97); RW 4.7 closed
- Session: `https://claude.ai/code/session_015GbcmhgsYDQj97EaKh65ea`   Model: **Opus 5.5 until `fe6eb18`, then Opus 5
  (1M context)** — the runtime switched the session mid-run, with no classifier notice; commits from `a8040fd` on name
  Opus 5, as `CLAUDE.md` requires
- Branch: `master` (pushed through the harness branch `claude/jolly-hamilton-7fd30a` and fast-forwarded)   Pull request:
  none   Merged: n/a
- Base: `fe6eb18`
- Commits: (1) `a8040fd` P8-02 steps 5–7: `--adapter-dpx` — a driver read once, pinned, re-proved and interpreted;
  (2) `681b6f6` the new example through every gate it is part of (canonical form, both snapshot corpora); (3) the records
  and this entry
- Files and folders: M `crates/delulu-runtime/src/adapter.rs` (`line_reader` factored out, `LineTransport`),
  `crates/delulu-runtime/src/verified_adapter.rs` (`check_interface`; the unit tests read the shipped example),
  `crates/delulu/src/cli.rs` (`--adapter-dpx`, `--adapter-transport`, `load_dpx_driver`, the help),
  `crates/delulu/src/run_cmd.rs` (`verified_driver`, `verified_driver_flags`, the plan's boxed adapter),
  `crates/delulu/src/guest.rs` (the sandbox allowlist), `docs/survey/*`; A `crates/delulu/tests/hw_dpx_cli.rs`,
  `examples/line_driver/delulu.toml`, `examples/line_driver/src/lib.delulu` — (1). M `examples/line_driver/src/lib.delulu`
  (canonical form), `tests/core-invariance/SNAPSHOT.txt` (150 lines added, none removed), `docs/survey/*`; A
  `crates/delulu/tests/snapshots/atlas_chain/line_driver.json`, `…/line_driver__src__lib.json` — (2). M `CHANGELOG.md`,
  `HANDOFF.md` (§11.5, three lessons), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-97),
  `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/DEPLOYMENT.md`, `docs/REMAINING_WORK.md` (4.7 closed),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (3). Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors / 14 warnings, `doctor --check`
  all checks passed
- Verified: eight unit witnesses for the driver and three for the transport, five end to end; mutants M136–M153 each red
  (M143 survives on a second wall, M143b red); clippy clean; `check-other-os.sh` clean for Windows and macOS after the
  last code edit; runner reads at `a8040fd` (macOS `37336594722`, Windows `37336599645`, arm64 `37336603458`) and at
  `681b6f6` (macOS `37337864832`, Windows `37337869032`, 35 passed each — the four gates the new example changed); the
  full suite alone 2,145 passed, 0 failed, 16 ignored (159 binaries), cargo exit 0, the tree unmoved. The first suite of
  the slice found four gates red that no witness target reaches (`fmt --check examples`, the Atlas round-trip, the
  `atlas_chain` corpus, `core_invariance`) — fixed in `681b6f6`.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a new example is a new case in four gates; a model switch mid-session is
  a fact for the commit trailer; a mutant must be able to compile. `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next: (1) read this run's push runs; (2) **P8-03** — the simulator as a device behind `LineTransport`, across a
  real process boundary (the shape is in `V2_P8_DESIGN.md`); (3) **P8-04**, the out-of-band monitor; then **P9**. PS-E's
  three residuals stay open (H6/RW 4.27, macOS's launcher window/RW 4.28, macOS's reads). **For the owner:** D-V2-96 and
  D-V2-97 (P8-02 — a hardware driver may be a signed Verified plugin; RW 4.7, open since D23, is closed) are this run's
  decisions.

### 2026-10-05 — routine run 14 (continued): P8-03 complete — the simulator as a device process (D-V2-98)
- Session: `https://claude.ai/code/session_015GbcmhgsYDQj97EaKh65ea`   Model: Claude Opus 5 (1M context) — the runtime
  switched the session from Opus 5.5 mid-run, with no classifier notice
- Branch: `master` (through `claude/jolly-hamilton-7fd30a`, fast-forwarded)   Pull request: none   Merged: n/a
- Base: `963328d`
- Commits: (1) `e92870a` P8-03: the in-tree simulator as a device process, and the whole stack across a real boundary;
  (2) `cb79333` `device` declared acting (the MCP door rule); (3) the records and this entry
- Files and folders: A `crates/delulu-runtime/src/sim.rs`, `crates/delulu/src/device_cmd.rs`,
  `crates/delulu/tests/sim_device_cli.rs`; M `crates/delulu-runtime/src/device.rs` (the simulator moved out),
  `crates/delulu-runtime/src/lib.rs`, `crates/delulu/src/cli.rs` (the dispatcher, `SUBCOMMANDS`, the help),
  `crates/delulu/src/main.rs`, `crates/delulu/tests/json_contract.rs` (both sweeps), `docs/survey/*` — (1). M
  `crates/delulu/src/mcp.rs` (`EFFECTORS`), `docs/survey/*` — (2). M `CHANGELOG.md`, `HANDOFF.md` (§11.5, two lessons),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-98), `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*` — (3). Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: three end-to-end witnesses and six unit ones; M154–M158 each red; clippy clean; `check-other-os.sh` clean
  for Windows and macOS after the last code edit; read on the runners at `e92870a` — macOS `37341164996` (56 passed),
  Windows `37341170246` (55), arm64 `37341173453` (6 `sim::` tests), every new witness named; the full suite alone
  2,154 passed, 0 failed, 16 ignored (160 binaries), cargo exit 0, the tree unmoved. The first suite of the slice found the MCP door rule red (`cb79333`) — the fifth gate a new subcommand
  must pass, and one no witness target reaches.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a new subcommand is five gates; a server is classified by what it hands
  out, not by what it writes.
- Open / next: (1) read `1a022c9`'s CI run and this closing commit's (the run's `openshell` job read green already);
  (2) **P8-04** — its build order is now in `V2_P8_DESIGN.md`, read against the code by this run, and the next run starts
  at **step 3, option (b)**: a monitor node minted between the operator and the run, which can revoke its own subtree and
  nothing else. **Option (c) — a revoke-only principal in the tree — is for the OWNER**, because it changes who may
  revoke; the argument that it hardens rather than redefines (revocation is monotone, so a principal that can only
  revoke cannot widen anything) is written out in the design. (3) Then **P9**. PS-E's three residuals stay open.
  **For the owner:** D-V2-96, D-V2-97 (P8-02, RW 4.7 closed after D23) and D-V2-98 (P8-03) are this run's decisions, and
  P8-04's option (c) is the one question it leaves him.

### 2026-10-05 — routine run 14 (closing): P8-04's build order and its design question; the loop's new rule
- Session: `https://claude.ai/code/session_015GbcmhgsYDQj97EaKh65ea`   Model: Claude Opus 5 (1M context)
- Branch: `master`   Base: `1a022c9`
- Commits: this one — `docs/DELULULANG_V2/V2_P8_DESIGN.md` (P8-04's build order, seven steps, with the
  ancestor-must-bound-its-child tension and three ways out), `V2_LOG.md` (the reading, and option (c) left to the owner),
  `docs/CLOUD_ROUTINE.md` (step 4's P8 line brought up to date; **new step 7a — a new surface is never one edit**: the
  four gates an example passes and the five a subcommand does, none reachable from a witness target),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`
- Survey and doctor: `survey check` ok (1,495 nodes, 13,511 edges), `survey findings` 0 errors, `doctor --check` all
  checks passed; the document gates re-run green after the records
- Verified: nothing built in this commit — a reading and the records. `master` green through `963328d` (CI
  `37339300025`, 16 jobs, arm64 ping-pong MEASURED 2.96x/4.11x; `openshell` and `ocsf` success); `1a022c9`'s CI was in
  progress at the close, its `openshell` job already green.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 and `docs/CLOUD_ROUTINE.md` step 7a (a new surface is never one edit).
- Open / next: **P8-04 from step 3, option (b)** (`V2_P8_DESIGN.md`); read `1a022c9`'s and this commit's CI runs first.
  **For the owner:** P8-04's option (c), a revoke-only principal in the grant tree.

### 2026-10-09 — routine run 15: `master` red four days on a stale map, fixed (`1732d9e`); run 14's stranded `6dbc54c` merged; a pre-commit hook
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (the scheduled routine; no
  fallback notice)
- Branch: `master` (the VM's checkout was the harness branch `claude/happy-pascal-a1e0hm`, at `master`'s head; pushed to
  both)   Pull request: none   Merged: n/a
- Base: `75bb33b` (run 14's closing commit). Nothing was committed between 2026-10-05 16:58 and this run (2026-10-09 15:21).
- Commits: (1) `1732d9e` the merge of `6dbc54c` with the map regenerated after it; (2) this entry's commit — the hook, its
  routine step and the records
- Files and folders: (1) M `crates/delulu-runtime/src/device.rs`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-99),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (7.17), `docs/survey/*` — all from `6dbc54c`, plus the
  regenerated map. (2) A `scripts/hooks/pre-commit`; M `docs/CLOUD_ROUTINE.md` (step 2: `git config core.hooksPath
  scripts/hooks`), `docs/REPOSITORY_STRUCTURE.md` (the hook's row), `HANDOFF.md` (§11.5, two lessons),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as
  co-author.
- Survey and doctor: at the START, `survey check` **stale** (SURVEY.md, survey.json) and `doctor --check` one problem
  (the map's freshness) — `75bb33b`'s map was built before its last 21 lines of documents. After `1732d9e`: `survey
  check` ok (1,495 nodes, 13,516 edges), `doctor --check` all checks passed.
- Verified: **CI on arrival — red.** `75bb33b`'s push run `37344075684` and the nightlies `37444870874` (10-06),
  `37602353806` (10-07), `37759591993` (10-08), `37914430616` (10-09): the four test jobs red, every other job green.
  Logs read (the push run's Linux, macOS, Windows and arm64; the 10-09 nightly's Linux): the same four failures and nothing
  else — `doctor_cli` ×3 and the Survey's `the_committed_map_matches_the_tree`. Ping-pong on arm64 MEASURED 2.86x
  (push run), 2.98x, 2.95x, 2.89x, 2.97x (nightlies); macOS MEASURED 1.59x on the push run, NOT MEASURED (3 hardware
  threads) on the nightlies; Linux and Windows NOT MEASURED (busy or SMT). `6dbc54c`'s three witness runs read
  (`37344931813` arm64 — the new test named, 21 passed; `37344936032` Windows; `37344939689` macOS — all success); its
  test and mutants M159/M160 re-run red here. `1732d9e`: `doctor_cli` 11 passed, the Survey's tests and six document gates
  green locally; push run `37952078892` — see the next entry for its reading. The hook falsified three ways (a staged edit
  with no rebuild refused; rebuilt and staged accepted; an unstaged edit the map was built from — `survey check` on the
  tree ok, the hook refusing).
- Redo on the laptop: `git config core.hooksPath scripts/hooks` in the laptop's checkout (Git-Bash runs it on Windows; it
  finds `delulu-survey.exe`).
- For the laptop's memory: `HANDOFF.md` §11.5 — a rule a run must remember is a rule a run will forget (make it a hook);
  a run's last push is the one nobody reads.
- Open / next: P8-04 — this run measured step 1 against the binary before building on it (next entry).

### 2026-10-09 — routine run 15 (continued): P8-04 step 1 measured — an envelope's refusal is a `deny` in the chain (AUDIT-REFUSAL-1, D-V2-100); AUDIT-SEQ-1 found
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (pushed to the harness branch `claude/happy-pascal-a1e0hm` first, read on the runners, then `master`
  fast-forwarded)   Pull request: none   Merged: n/a
- Base: `dcd6f9a`
- Commits: (1) `5cd4474` P8-04 step 1, measured: an envelope's refusal is a `deny` in the audit chain, citing the use it
  overrides; (2) this entry's commit — the records
- Files and folders: (1) A `crates/delulu/tests/monitor_cli.rs`; M `crates/delulu-broker/src/validate.rs`
  (`record_device_refusal` and its unit test), `crates/delulu-runtime/src/custody.rs` (`note_device_refusal`),
  `crates/delulu-runtime/src/interp.rs`, `crates/delulu-runtime/src/channel.rs` (both callers of `device::actuate`),
  `crates/delulu/src/broker_ipc.rs` (`DeviceRefused`, `Recorded`), `crates/delulu/src/brokerd.rs`,
  `crates/delulu/src/broker_client.rs`, `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (§11.4 the finding, §11.5 two
  lessons), `docs/CLOUD_ROUTINE.md` (step 4's P8 line), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-100),
  `V2_P8_DESIGN.md` (P8-04 step 1, measured), `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.43 closed, 4.44 open),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub
  no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed (the commit hook checked the map against each commit's index)
- Verified: the witness red before the fix (four `use allow`, no `deny`); two end-to-end witnesses (ordinary and
  `--sandbox`) and one unit witness; mutants M161–M167 each red; clippy clean; `check-other-os.sh` clean for Windows and
  macOS; runner reads at `5cd4474` over the nine actuator targets — macOS `37954533949` (56 passed), Windows `37954537458`
  (55), arm64 `37954540761` (56), both new witnesses named on each; the full suite alone 2,158 passed, 0 failed, 16 ignored
  (161 binaries), cargo exit 0, the tree unmoved. `1732d9e`'s push run `37952078892`: all four test jobs green (Windows
  and Linux ping-pong NOT MEASURED, arm64 MEASURED 2.88x/4.12x, macOS NOT MEASURED — 3 threads).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (AUDIT-REFUSAL-1; AUDIT-SEQ-1 open) and §11.5 ("everything it needs already
  exists" is a hypothesis; a restore by `move` keeps the old mtime).
- Open / next: (1) read `dcd6f9a`'s and this commit's push runs; (2) **P8-04 step 3 (b)** — `delulu monitor watch --node
  g_M --rule envelope|denies=N/MS`: the subtree from the daemon's `List`, the chain read after the last record HASH
  (never a seq — AUDIT-SEQ-1), quarantine = a revoke with caller `g_M` of the run's node (the child of `g_M` on the path),
  the rule and its evidence in the revoke's own record; a new subcommand's five gates; witnesses in `monitor_cli.rs`;
  (3) **RW 4.44, AUDIT-SEQ-1** — one numbering for the chain. **For the owner:** D-V2-100 is this run's decision.

### 2026-10-09 — routine run 15 (continued): P8-04 step 3 (b) — `delulu monitor watch` quarantines a run under its node (D-V2-101)
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-a1e0hm`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `0ffaac2`
- Commits: (1) `b0a5a80` P8-04 step 3 (b): `delulu monitor watch` quarantines a run under its node, and the chain says
  why; (2) this entry's commit — the records
- Files and folders: (1) A `crates/delulu/src/monitor_cmd.rs`; M `crates/delulu-broker/src/lib.rs` (`bounded_text`),
  `crates/delulu-broker/src/tree.rs` (`revoke_saying`), `crates/delulu-broker/src/validate.rs` (uses `bounded_text`),
  `crates/delulu/src/broker_ipc.rs` (`Quarantine`), `crates/delulu/src/brokerd.rs`, `crates/delulu/src/cli.rs` (the
  dispatcher's arm, `SUBCOMMANDS`, the help), `crates/delulu/src/main.rs`, `crates/delulu/src/mcp.rs` (`EFFECTORS`),
  `crates/delulu/tests/json_contract.rs` (both sweeps), `crates/delulu/tests/monitor_cli.rs` (two witnesses),
  `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (§11.5, one lesson), `docs/CLOUD_ROUTINE.md` (step 4),
  `docs/DEPLOYMENT.md` (the monitor's section; a caution on `--since SEQ` while RW 4.44 is open),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-101), `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: two end-to-end witnesses and two unit ones; the quarantine measured 13 ms after the third refusal (the VM),
  11 ms (Windows), 24 ms (arm64), 107 ms (macOS), at a 25 ms poll; mutants M168–M174 each red; clippy clean;
  `check-other-os.sh` clean for Windows and macOS after the last code edit; runner reads at `b0a5a80` — macOS
  `37957000733` (198 passed), Windows `37957004727` (182), arm64 `37957008755` (217), `delulu-broker` lib on Windows
  `37957012378` (170) — every new test named; the full suite alone 2,162 passed, 0 failed, 16 ignored (161 binaries),
  cargo exit 0, the tree unmoved. Push runs read: `dcd6f9a` `37954416113` success on every job (arm64 MEASURED
  2.92x/4.11x); `1732d9e` `37952078892` cancelled by the newer push with its four test jobs green (only `fuzz` cancelled).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a witness of an absence must prove it looked.
- Open / next: (1) read `0ffaac2`'s and this commit's push runs; (2) **P8-04 step 5** — the monitor's own death
  (`quarantine` by default for a run holding a device, `continue` as the operator's opt-out), then the study's other two
  rules once their records are measured; (3) **RW 4.44, AUDIT-SEQ-1**. **For the owner:** D-V2-100 and D-V2-101 are this
  run's decisions; P8-04's option (c), a revoke-only principal, is still his.

### 2026-10-09 — routine run 15 (continued): loop engineering — `scripts/mutants.py`; the cross-OS lint reaches `delulu-broker`
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `7a192c9`
- Commits: this one
- Files and folders: A `scripts/mutants.py`; M `scripts/check-other-os.sh` (`-p delulu-broker`),
  `crates/delulu-broker/src/secrets.rs` (`in_memory_at` is `cfg(all(test, unix))`), `docs/CLOUD_ROUTINE.md` (step 5),
  `docs/REPOSITORY_STRUCTURE.md` (the script's row), `HANDOFF.md` (§11.5), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub
  no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: the extended lint RED on the unfixed source for Windows, green after for Windows and macOS; the broker's 172
  unit tests; `scripts/mutants.py` exercised (two RED mutants, a non-unique anchor refused, a BUILD-ERROR, the control
  green).
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 — the cross-OS lint covers only the packages it names.
- Open / next: as the previous entry.

### 2026-10-09 — routine run 15 (continued): AUDIT-SEQ-1 closed — the audit chain has one numbering (D-V2-102, RW 4.44)
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-a1e0hm`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `2d3741d`
- Commits: (1) `2546425` AUDIT-SEQ-1 closed: the audit chain has one numbering, whoever writes and across a daemon
  restart; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-broker/src/audit.rs` (`max_seq`, the seq settled in `append`,
  `AuditSink::next_seq`, the unit witness), `crates/delulu-broker/src/tree.rs` (`take_seq` takes the floor),
  `crates/delulu/src/brokerd.rs` (`TrackingSink` forwards it), `crates/delulu/src/guest.rs` (the host asks for no seq),
  `crates/delulu/tests/monitor_cli.rs` (the end-to-end witness), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md`
  (§11.4), `crates/delulu/src/monitor_cmd.rs` (a doc comment), `docs/CLOUD_ROUTINE.md`, `docs/DEPLOYMENT.md` (the
  caution on `--since` rewritten for old chains), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-102),
  `V2_P8_DESIGN.md`, `docs/REMAINING_WORK.md` (4.44 closed), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub
  no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: red first on the binary (`1, 2, 3, 1, 2, 3, … 11, 12, 11, …`); one unit and one end-to-end witness; mutants
  M175–M178 red through `scripts/mutants.py`, control green; clippy clean; `check-other-os.sh` (with `delulu-broker`)
  clean for Windows and macOS; runner reads at `2546425` — macOS `37959503886` (90 passed), Windows `37959508618` (83),
  arm64 `37959514499` (93), `delulu-broker` lib on macOS `37959518422` (173), the new witnesses named; the full suite
  alone 2,164 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved. Push runs read: `0ffaac2`
  `37955746780` success on every job (arm64 MEASURED 2.88x/4.11x).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL. The laptop's own `~/.delulu/audit` may hold repeated
  seqs from before the fix — it still verifies; nothing to repair.
- For the laptop's memory: `HANDOFF.md` §11.4 — AUDIT-SEQ-1, closed.
- Open / next: (1) read `7a192c9`'s, `2d3741d`'s and this commit's push runs; (2) **P8-04 step 5** — the monitor's own
  death; (3) the study's other two monitor rules once their records are measured. **For the owner:** D-V2-100, D-V2-101,
  D-V2-102.

### 2026-10-09 — routine run 15 (continued): HTTP-SCHEME-1 — a plain-`http://` fetch died under the broker daemon; P8-04 step 5 read
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-a1e0hm`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `eebf6a1`
- Commits: (1) `e7921cb` HTTP-SCHEME-1: a plain-http fetch is the same refusal under the broker daemon as without it;
  (2) this entry's commit — the records and P8-04 step 5's reading
- Files and folders: (1) M `crates/delulu-runtime/src/interp.rs` (`custody_op_for`), `crates/delulu/tests/monitor_cli.rs`,
  `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (§11.4), `docs/DELULULANG_V2/V2_LOG.md`, `V2_P8_DESIGN.md` (step 5
  read against the code; option (c) recommended), `docs/REMAINING_WORK.md` (4.45 closed),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: red first (DL0904 under the daemon, the embedded control green); M179 red through `scripts/mutants.py`,
  control green; clippy clean; `check-other-os.sh` clean for Windows and macOS; runner reads at `e7921cb` — macOS
  `37961192830` (28 passed), Windows `37961196373` (29), arm64 `37961200403` (29); the full suite alone 2,165 passed, 0
  failed, 16 ignored (161 binaries). Push runs read: `7a192c9` `37958130505` and `2d3741d` `37958791794` success. The
  VM's `/etc/hosts` was edited for one measurement and restored (checked).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 — HTTP-SCHEME-1.
- Open / next: (1) read `eebf6a1`'s and this commit's push runs; (2) **P8-04 step 5, option (c)** — a broker dead-man for
  the monitor's node (`V2_P8_DESIGN.md`), witnessed by killing a monitor mid-run; (3) finish measuring the special-use
  rule's records (an `https://` name pointed at loopback: is the egress client's special-use refusal in the chain?), then
  the break-glass rule (its record carries no node today). **For the owner:** D-V2-100, D-V2-101, D-V2-102.

### 2026-10-09 — routine run 15 (continued): the special-use rule measured (RW 4.46, open); the routine's commit order
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `73203fc`
- Commits: this one — records only
- Files and folders: M `docs/DELULULANG_V2/V2_LOG.md`, `V2_P8_DESIGN.md` (step 4 measured), `docs/REMAINING_WORK.md`
  (4.46 opened), `docs/CLOUD_ROUTINE.md` (step 4's P8 line; step 6 — commit a slice's code before its records),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub
  no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: the measurement (one `https://` run to a name pointed at loopback: "not delivered", the special-use reason on
  stderr, and only `use allow` in the chain); the VM's `/etc/hosts` restored and checked. Push runs read: `eebf6a1`
  `37960632115` success on every job (it carries AUDIT-SEQ-1's fix).
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new beyond the previous entries.
- Open / next: as the previous entry, with RW 4.46 before the `special-use` rule.

### 2026-10-09 — routine run 15 (closing): the red-team pass — ACTOR-CUSTODY-1 found (HIGH, open); the monitor's self-quarantine fixed; D-V2-102 corrected
- Session: `https://claude.ai/code/session_01S4tfGaLdv3KbErQ6JExQ4t`   Model: Claude Opus 5.5 (head chef, no fallback
  notice); the red-team sous-chef ran on **Sonnet 5.5** (`sonnet`), about 27 minutes, against a frozen copy of the binary
  at `73203fc` under `/tmp` — `git status` clean after it
- Branch: `master` (the fix through the harness branch `claude/happy-pascal-a1e0hm`, read on the runners, then
  fast-forwarded)   Pull request: none   Merged: n/a
- Base: `b0597d3`
- Commits: (1) `495204e` a monitor never quarantines its own node (red-team F3); (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/monitor_cmd.rs`, `crates/delulu/tests/monitor_cli.rs`, `docs/survey/*`.
  (2) M `CHANGELOG.md`, `HANDOFF.md` (§11.4, ACTOR-CUSTODY-1), `docs/CLOUD_ROUTINE.md` (step 4: the next run's order),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-102 item 2 corrected), `docs/REMAINING_WORK.md` (4.47, 4.48,
  4.49 opened), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted:
  nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: every red-team finding recorded here was re-run by the head chef on the current binary first — F1 (a revoke at
  3 s, the device journal "lease revoked", the actor told `COMMANDED` 60/60, exit 0), F2 (72 parallel runs: 0 duplicate
  seqs, 2 wrong references), F3 (the monitor revoked its own node) — except F4–F6, recorded as code reading. F3's fix:
  red first, M180 red, control green; runner reads macOS `37965843494`, Windows `37965846952`, arm64 `37965849965`; the
  full suite alone 2,166 passed, 0 failed, 16 ignored (161 binaries). Push runs read this run, all green on every job:
  `dcd6f9a` `37954416113`, `0ffaac2` `37955746780`, `7a192c9` `37958130505`, `2d3741d` `37958791794`, `eebf6a1`
  `37960632115`, `73203fc` `37962421933`; `1732d9e` `37952078892` cancelled by the next push with its four test jobs green.
  `da6518c`, `b0597d3` and this commit's are unread at the close.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL; `git config core.hooksPath scripts/hooks`.
- For the laptop's memory: `HANDOFF.md` §11.4 (ACTOR-CUSTODY-1; HTTP-SCHEME-1; AUDIT-REFUSAL-1; AUDIT-SEQ-1) and §11.5
  (this run's seven lessons); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next, in order: (1) read `da6518c`'s, `b0597d3`'s and this commit's push runs and the 2026-10-10 nightly; (2) **RW
  4.47, ACTOR-CUSTODY-1 (HIGH)** — an actor's effects under daemon custody: a custody client and the run's device broker
  per worker, or refuse actors that hold an actuator until then; the witness is the re-run above; (3) **RW 4.48** — hold
  the append lock from the daemon's floor to its write; (4) **P8-04 step 5, option (c)**; (5) RW 4.46 and 4.49. **For the
  owner:** D-V2-100, D-V2-101, D-V2-102 (corrected) are this run's decisions; P8-04's option (c), a revoke-only principal,
  is still his. **On 2026-10-12 a run starts the closing entry for the cloud period** (the routine's prompt).

### 2026-10-10 — routine run 16: CI read green; ACTOR-CUSTODY-1 closed — an actor's effects answer to the run's custody and brokers (D-V2-103)
- Session: `https://claude.ai/code/session_01FM5pjaFXk3YSik7zQzQXxh`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-44t471`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `742b11b`
- Commits: (1) `33f0518` ACTOR-CUSTODY-1: an actor's effects answer to the run's custody and brokers; (2) this entry's
  commit — the records, and `scripts/mutants.py` adds `--no-fail-fast`
- Files and folders: (1) M `crates/delulu-runtime/src/actors.rs` (`WorkerEffects`, `ActorSystem::start_governed`),
  `crates/delulu/src/broker_client.rs` (`same_node_clients`), `crates/delulu/src/run_cmd.rs`,
  `crates/delulu/tests/estop_cli.rs` (three actor witnesses), `crates/delulu/tests/actuate_cli.rs`,
  `crates/delulu/tests/compute_cli.rs`, `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.4
  ACTOR-CUSTODY-1 closed; §11.5 two lessons), `docs/CLOUD_ROUTINE.md` (step 4's P8 line; step 5's mutants line),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-103), `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.47
  closed, 4.50 opened), `docs/assistant-memory/cloud-period-2026-09-28.md`, `scripts/mutants.py`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as
  co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: red first on `742b11b`'s code (the chain held 0 of an actor's 14 commands; a device-node revoke never reached
  the actor; a run-node revoke never killed it; by hand, no daemon, `rate_hz=1`: the actor's two commands both landed);
  mutants M181–M186 red, control green; clippy clean; `check-other-os.sh` clean for Windows and macOS; runner reads at
  `33f0518` over six targets — macOS `38009025737`, Windows `38009027757`, arm64 `38009029739`, 49 passed and 0 failed on
  each, the five new witnesses named; the full suite alone 2,171 passed, 0 failed, 16 ignored (161 binaries). Push runs
  read: `da6518c` `37962831019`, `b0597d3` `37964495079`, `742b11b` `37967062925` — success on every job. The nightlies of
  10-06 to 10-09 are red on `75bb33b` (run 15's stale map, already recorded). The container restarted once mid-run and
  stopped the first suite; it was run again from the start.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (ACTOR-CUSTODY-1 closed; RW 4.50 a hypothesis) and §11.5 (every
  `Interp::new` is a place the run's authority can stop; a mutant over several targets needs `--no-fail-fast`).
- Open / next: (1) read `33f0518`'s and this commit's push runs and the 2026-10-10 nightly; (2) **RW 4.50, PLUGIN-CUSTODY-1**
  — witness or refute; (3) RW 4.48; (4) **P8-04 step 5, option (c)**; (5) RW 4.46 and 4.49. **For the owner:** D-V2-103.
  **On 2026-10-12 a run starts the closing entry for the cloud period** (the routine's prompt).

### 2026-10-10 — routine run 16 (continued): PLUGIN-CUSTODY-1 witnessed and closed; plugins callable under the daemon (D-V2-104)
- Session: `https://claude.ai/code/session_01FM5pjaFXk3YSik7zQzQXxh`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-44t471`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `2eb7fca`
- Commits: (1) `8f2b848` PLUGIN-CUSTODY-1: a plugin export runs under the host's custody and devices; plugins callable
  under the daemon; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-runtime/src/interp.rs` (`custody` an `Rc`; `call_plugin_export` shares it and the
  device broker), `crates/delulu/src/broker_client.rs` (`liveness`), `crates/delulu/tests/plugin_load_cli.rs` (three
  witnesses), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.4; §11.5 a lesson),
  `docs/CLOUD_ROUTINE.md` (step 4's P8 line), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-104, correcting
  D-V2-27 item 4 without editing the owner's text), `V2_PHASE_STATUS.md` (P2's daemon note; P8), `docs/REMAINING_WORK.md`
  (4.50 closed, 4.51 opened), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: red first — embedded, `rate_hz=1`, the plugin's two commands both `COMMANDED`; under the daemon every plugin
  call `Revoked 0`; with liveness alone, 0 of 2 of the plugin's uses in the chain. Mutants M187–M190 red, control green;
  clippy clean; `check-other-os.sh` clean for Windows and macOS; `delulu-runtime`'s unit tests green; runner reads at
  `8f2b848` over six targets — macOS `38011420070`, Windows `38011422634`, arm64 `38011424714` (47 passed, 0 failed), green;
  the full suite alone 2,174 passed, 0 failed, 16 ignored (161 binaries). Push runs read: `33f0518` `38010274353` and
  `2eb7fca` `38010619232` — success. RW 4.51 measured by hand (`grants list` after a run: the plugin's node `[live]`).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (PLUGIN-CUSTODY-1, PLUGIN-DAEMON-1) and §11.5 (a path that fails closed on
  everything hides what stands behind it).
- Open / next: (1) read `8f2b848`'s and this commit's push runs and the 2026-10-10 nightly; (2) RW 4.48; (3) **P8-04 step
  5, option (c)**; (4) RW 4.51, 4.46, 4.49. **For the owner:** D-V2-103, D-V2-104 (and D-V2-27 item 4's correction).
  **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-10 — routine run 16 (continued): RW 4.48 closed — the daemon holds the audit chain from a seq's floor to its write; AUDIT-DAY-1 found and closed (D-V2-105)
- Session: `https://claude.ai/code/session_01FM5pjaFXk3YSik7zQzQXxh`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-44t471`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `80c566c`
- Commits: (1) `7038885` RW 4.48: the daemon holds the audit chain from a seq's floor to its write; AUDIT-DAY-1: a record's
  day file follows the chain; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-broker/src/audit.rs` (`take_lock`, `next_seq_under`, `append_under`; the day
  file; a witness), `crates/delulu/src/brokerd.rs` (`TrackingSink` holds the lock per request; the serve loop releases it;
  two witnesses), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.4; §11.5 two lessons),
  `docs/CLOUD_ROUTINE.md` (step 4's P8 line; step 5 — draft the next witness in the scratchpad), `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-105; D-V2-102 item 2 marked closed), `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.48
  closed, 4.52 added closed), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: the red team's F2 as a probe (60 sandboxed + 12 daemon runs in parallel): 2 and 3 wrong references on
  `80c566c`'s binary, 0 in three probes on `7038885`'s; witnesses red first (reserved 2, written 3; a `prev_hash` break across
  midnight); M191–M194 red, control green; clippy clean; `check-other-os.sh` clean for Windows and macOS; runner reads at
  `7038885` — macOS `38013294529`/`38013300816`, Windows `38013296565`/`38013302335`, arm64 `38013298889`/`38013303853`,
  green, the three witnesses named; the full suite alone 2,177 passed, 0 failed, 16 ignored (161 binaries). **Not clean:** a
  test file was edited and one target compiled WHILE that suite ran (the next slice's witness); the file was restored to HEAD
  before the suite reached the target, which ran its own earlier build (`plugin_load_cli-6fe4557e`) — recorded in §11.5.
  Push runs read: `80c566c` `38012262994` — success.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (RW 4.48, AUDIT-DAY-1) and §11.5 (a timestamp-derived key is an order claim;
  a new witness is an edit).
- Open / next: (1) read `7038885`'s and this commit's push runs and the 2026-10-10 nightly; (2) RW 4.51 (this run's next
  slice, if it lands); (3) **P8-04 step 5, option (c)** — IPC `ArmDeadman`/`Beat` for a node, a timer in the daemon's loop
  (`recv_timeout`), `monitor watch --on-monitor-death quarantine|continue`, witnessed by killing a monitor mid-run;
  (4) RW 4.46, 4.49. **For the owner:** D-V2-103, D-V2-104, D-V2-105.

### 2026-10-10 — routine run 16 (closing): RW 4.51 closed — a plugin's node ends with its run (its first form's regression caught before `master`); the next run's step 5, sized
- Session: `https://claude.ai/code/session_01FM5pjaFXk3YSik7zQzQXxh`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-44t471`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `9eb6f90`
- Commits: (1) `d164a68` RW 4.51: a plugin's grant node does not outlive the run that minted it (its first form — red on the
  runners, never on `master` alone); (2) `d7e2b8f` RW 4.51 fixed again: end_of_run revokes only a run's PLUGIN nodes;
  (3) this entry's commit — the records
- Files and folders: (1)+(2) M `crates/delulu-runtime/src/custody.rs` (`end_of_run`), `crates/delulu-runtime/src/interp.rs`
  (`end_run`), `crates/delulu/src/broker_client.rs` (`minted`, plugin holders only; `end_of_run`), `crates/delulu/src/run_cmd.rs`,
  `crates/delulu/src/brokerd.rs` (a witness), `crates/delulu/tests/plugin_load_cli.rs` (a witness), `docs/survey/*`. (3) M
  `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P8 line),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.51 closed),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: RW 4.51 red first (`grants list`: the plugin's node `live`); M195–M198 red, controls green; clippy clean;
  `check-other-os.sh` clean for Windows and macOS. **The first form's regression:** suite 5 at `d164a68` — 2,177 passed,
  **1 failed** (`estop_cli` `with_nobody_revoking_anything_the_same_supervisor_keeps_its_arm`, cargo exit 101); runner reads
  `38014455713`, `38014457962`, `38014459712` red on the same test. Fixed in `d7e2b8f`: macOS `38015462027`, Windows `38015464061`, arm64 `38015466138` — green, both witnesses and `estop_cli`'s control named. Suite 6 at `d7e2b8f`:
  2,179 passed, 0 failed, 16 ignored (161 binaries), exit 0. Push runs read this run, all success: `33f0518` `38010274353`, `2eb7fca` `38010619232`, `80c566c` `38012262994`,
  `7038885` `38013916987` (and its OCSF run `38013916957`), `9eb6f90` `38014095655`.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 and §11.5 — this run's six lessons (every `Interp::new`; `--no-fail-fast` for
  mutants; a fail-closed path hides what stands behind it; a timestamp-derived key is an order claim; a new witness is an
  edit; a function given a new consequence has callers the slice did not look at).
- Open / next, in order: (1) read this commit's push run and the 2026-10-10 nightly; (2) **P8-04 step 5, option (c)** — the
  plan is in `V2_LOG.md` 2026-10-10 (closing): `ArmDeadman`/`Beat` for a node, the daemon's loop on `recv_timeout`, `monitor
  watch --on-monitor-death quarantine|continue`, witnessed by killing a monitor mid-run; (3) RW 4.46 and the `special-use` rule;
  (4) RW 4.49; (5) a red-team pass (Sonnet 5.5) on this run's four slices — custody now crosses into actors' and plugins'
  interpreters and the daemon holds the chain per request. **For the owner:** D-V2-103, D-V2-104 (with D-V2-27 item 4's
  correction), D-V2-105. **On 2026-10-12 a run starts the closing entry for the cloud period** (the routine's prompt).
