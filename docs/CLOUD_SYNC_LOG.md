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
  (2) `PS-E-01, first step: the guest confirms its boundary before it is sent the program (D-V2-56)`
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
  failed — the thread-stack gate on `boundary.rs`'s test attribute, fixed, re-run green.
- Redo on the laptop: (2) changes the channel to `/3` — **rebuild the WSL microVM guest images**
  (`~/microvm-image-a`, `~/microvm-image-b`: a `/2` guest refuses a `/3` host), and run the suite on Windows
  (the AppContainer guest and `sandbox probe`'s contained path run only there and on CI).
- For the laptop's memory: `HANDOFF.md` §11.3 (raw.githubusercontent.com reachable) and §11.5 (two traps);
  `docs/assistant-memory/cloud-period-2026-09-28.md` (routine run 2).
- Open / next: (1) read (2)'s push run — every OS, and the `microvm` job (its guest image is built from
  this tree and speaks `/3`); (2) PS-E-01's next step, in this run if time allows.
