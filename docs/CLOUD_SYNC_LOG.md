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
8. Delete no entry here: this log stays as the record of the cloud period. **Branches:** on 2026-10-17 the
   owner chose to delete the routine's `claude/*` branches whose tips `master` already holds, and to keep the
   two whose one commit `master` lacks is cited by hash (that day's entry lists every branch and its tip).

## The closing entry — the cloud period, 2026-09-28 → 2026-10-16

**Begun 2026-10-12 by routine run 19** — four days early, as the routine's prompt asks ("start writing it on 2026-10-12,
because the weekly limit can hit before 2026-10-16"), and kept current in place by every later run until the period
ends. A run on or after 2026-10-16 that finds this section only updates its "as of" line and the open list, pushes,
and stops (`docs/CLOUD_ROUTINE.md` step 0). The dates may be extended by the owner; this section then stays the
running summary.

**As of:** `master` at the end of routine run 25 (2026-10-15) — the newest entry under *Entries* names the last commit.

**What the period built** (each item's entry below has its commits, files, evidence and CI runs):
- **Phases.** **P7** (verification depth) and **PS-D** (external launchers and the attestation seam) completed by run 1;
  **PS-E** — the boundary, confirmed, designed from the owner's commission to study NVIDIA OpenShell — **complete** (run 12,
  D-V2-93): the `/3` channel and confirmed boundary (E-01), a guest that ends with its host on every OS (E-02), the
  escaped-guest harness and its ten closed hypotheses H1–H5, H7–H10 (E-03), the external launcher resolved once, hashed and
  pinnable (E-04), the OpenShell export checked by OpenShell's own prover and run inside a real OpenShell sandbox on the
  runners (E-05), the audit chain exported as OCSF 1.8.0 (E-06). **P8** (safe autonomy): **P8-01** a control program in a
  guest, **P8-02** a signed Verified driver as a `.dpx` (RW 4.7 closed), **P8-03** the simulator as a device process (`delulu
  device sim`) — complete; **P8-04** the out-of-band monitor — complete: step 1 measured (AUDIT-REFUSAL-1, AUDIT-SEQ-1
  fixed), step 3 (b) `delulu monitor watch`, step 4's four rules (`envelope`, `denies=N/MS`, `special-use`, and `break-glass` —
  run 19), step 5 a dead monitor quarantines — **every step built by run 19 (D-V2-116): P8 complete as far as software
  reaches**, `9000c57`'s push run `38278927092` read green. **P9** — in progress: P9-01's build order read against the code by run 19
  (`V2_P9_DESIGN.md`); **P9-01 built by run 20** (D-V2-118 — `delulu authority FILE --within BOUNDARY`, each request decided by
  the runtime's own mint, the counterexample's call named); **P9-02** by the same run (D-V2-119, D-V2-120 — `delulu
  grants diff` between two grant files or two lease nodes; `secret_reach_expansion` waits for P9-04); P9-03 read against the code and its step 1 built (D-V2-121), step 1b by run 21 (D-V2-122 — a sandboxed run's report records the host's refusal at its call) and step 2 by the same run (D-V2-123 — `delulu grants propose REPORT`, the refused call's own requirement), both P9 verbs for a package directory (D-V2-124), and step 3 by run 22 (D-V2-125 — the proposal store: filed against the refused run's node, approved as a delegation under the approver's own node, rejected with a reason the agent reads), and step 4 the same run (D-V2-126 — `grants auto-approve`, the operator's standing node): **P9-03 complete**; **P9-04 step (a)** by run 23 (D-V2-128 — `http.get_with(url, headers)`, a request with headers judged before anything is resolved and sent only to the URL's origin); **P9-04 steps (b) and (c)** by run 24 (D-V2-131 — `http.get_with_secret`, a secret in one header sent only to the audience its grant names, `secret:NAME@HOST[:PORT]=…`, and `grants diff`'s `secret_reach_expansion`); **P9-04 step (d)** by run 25 (D-V2-133 — the broker's `secret_to`: a node's secret carries its audience, `grants delegate --secret-to`, `grants diff` between nodes, the Z3 model 34 obligations; D-V2-134 — the broker's `send`: a daemon or leased run sends a secret for one origin its node names, Guard-gated on the pair, recorded; its OCSF mapping) and **step (e)** the same run (D-V2-135 — `authority --json` names where a program sends each secret): **P9-04 complete**; P9-05 not yet built.
- **Defects found and closed** (`HANDOFF.md` §11.4 has each; never to be re-softened): ADAPTER-SPELL-1, ATTEST-FIFO-1,
  CONFIRM-ORDER-1, GUEST-WAIT-1, RAN-SENT-1, GUEST-TEXT-1, PIPE-WRITE-1/PIPE-FLOOD-1, GUEST-SOCKET-1, GUEST-PROC-1,
  GUEST-DEV-1, GUEST-SYSCALL-1, HOST-DUMPABLE-1, GUEST-TIOCSTI-1, GUEST-SIGNAL-1, GUEST-PROCESS-1, LANDLOCK-TRUNCATE-1,
  LAUNCHER-SPELL-1, TERMINAL-TEXT-1, FRAME-DRIP-1, REQUEST-HANG-1, PROBE-DRIP-1, BROKER-RELDIR-1, REGISTRY-BOUNDS-1,
  AUDIT-FIFO-1, AUDIT-TEXT-1, SCOPE-HIDDEN-1, REPLY-HOLD-1, ANSWER-HOLD-1, RUNDIR-PERM-1, SILENT-QUEUE-1, a macOS guest's
  missing memory ceiling, AUDIT-REFUSAL-1, AUDIT-SEQ-1, AUDIT-DAY-1, HTTP-SCHEME-1, ACTOR-CUSTODY-1 (HIGH),
  PLUGIN-CUSTODY-1/PLUGIN-DAEMON-1, CLOSURE-SCOPE-1 (HIGH), RW 4.56, RW 4.58, FIRST-BEAT-1 (RW 4.59), the red team's F1/F2
  and F3 (HIGH), RW 4.61 (D-V2-114, D-V2-115 — twelve daemon operations that answered success for a lost record), RW 4.60 (D-V2-117 — a
  report silent about devices that never started), SECRET-HIDDEN-1 (run 20 — a secret read under a computed name missing from
  `authority --grants`), DEADMAN-BUSY-1 (run 22, D-V2-127 — a healthy monitor's runs quarantined whenever the daemon was held on
  one request: a beat is now judged by when it arrived), RW 4.62 (run 23, D-V2-129 — a refused operation listed in the trace as if it
  had happened; every refusal after a dispatch is now recorded), RW 4.64 (the same run — the WASM engine's trace of a refused
  operation, now identical to the interpreter's), MAP-HANDLE-1 (run 24, D-V2-132 — under the daemon `Secret.map` ran over an
  empty string and its made-up result was exposed with no broker gate; refused in words now), NET-WILDCARD-1 (run 25, D-V2-136 —
  a pattern `net` grant refused under the daemon that embedded custody honored), the red team's F1 (run 25, D-V2-137 — a Guard
  seal on a secret's name did not cover its sends) — and whatever the later entries add.
- **The loop's own tools:** `witness.yml` (one test target on one runner at any branch), `scripts/check-other-os.sh`
  (clippy for macOS, Windows, arm64 and musl from Linux), `scripts/ci-log-summary.py`, `scripts/mutants.py`,
  `scripts/suite.sh`, `scripts/authority-corpus.py` (run 19), multi-edit mutants in `scripts/mutants.py` (run 24), the `scripts/hooks/pre-commit` map check, the repeated-heading gate in
  `repository_structure` (run 20), `ocsf.yml` and `openshell.yml`, and
  `docs/CLOUD_ROUTINE.md` itself — rewritten by almost every run (its step 8).
- **Decisions:** D-V2-56 to the newest `D-V2-nn` in `V2_DECISION_LOG.md` (run 1 took D-V2-46 to D-V2-51; the laptop's study
  took D-V2-52 to D-V2-55), every one TAKEN under the owner's delegation. **Three entrenched edits:** `DELULU_CORE.md` v0.3
  (D-V2-49, run 1) and `tests/conformance/witnesses.toml` — the anchors `ref.prim.http.get_with` (D-V2-128, run 23) and
  `ref.prim.http.get_with_secret` (D-V2-131, run 24), each in `ENTRENCHED_CHANGE_RECORD.md`. **No `DL` code was added. No `.md` file was deleted.**
- **Size, at run 25's last code** (`git diff --name-status cfbfbdc..b6128d1`): 275 commits; 171 files — 41 added, 130
  modified, **0 deleted, 0 renamed**; +44,588 / −4,893 lines. Commits by `Claude` (92, runs 1–9) and by
  `jessesuniljs1-collab` through its no-reply address with Claude as co-author (183, from run 10). (At run 19's start: 207
  commits, 139 files; at run 24's last code, 266 and 165.)

**What stays open** (the newest entry's "Open / next" is the exact list; `docs/REMAINING_WORK.md` the whole one):
**P9** (P9-05, method-and-path network scopes — read against the code by run 25, step (a) next; P9-04 complete by run 25; RW 4.65–4.67, the run-25 red team's open findings; P9-01 and P9-02 built by run 20, P9-03 by runs 20, 21 and 22, P9-04 step (a) by run 23, steps (b) and (c) by run 24), RW 4.57 (a killed run's nodes live under the daemon), RW 4.53 and 4.49 (the monitor's and the daemon's
waits on the chain), RW 4.58's O2/O4, a real device for P8 (environment-blocked). PS-E's three residuals: E-03's H6 (macOS and
Windows under the escaped-guest harness, twice stopped by a safety classifier — perhaps one for the owner at the laptop,
RW 4.27), macOS's launcher window (RW 4.28, no `fexecve`), macOS's reads.

**The owner's, untouched by the period:** the final public repository (`HANDOFF.md` §1.1's decisions first), the licence,
D-NE-7 (no tag, no release — the release workflow ran only its dry run and nightly), D-NE-27 (no kernel image), PS-C-05.
**For his review:** D-V2-49 (the entrenched edit) and D-V2-59 (`hostile-agent` requires all five properties), both flagged
when taken; the decisions each run's entry names "for the owner" (most recently D-V2-110 to D-V2-137); and P8-04's option
(c), a revoke-only principal in the grant tree, kept for him (`V2_P8_DESIGN.md`).

**Syncing the laptop** — the procedure above, from the baseline `cfbfbdc` (`Cloud handoff (10)`): a fast-forward is
expected. Then, beyond the suite on Windows and in WSL: **rebuild the WSL microVM guest images** (`~/microvm-image-a`,
`~/microvm-image-b` — the channel is `/3` since run 2 and the guest's own lock-down changed in run 3; a guest built before
refuses the current host); `git config core.hooksPath scripts/hooks` (run 15); `node editors/vscode/e2e.js` once on
Windows (run 1); and `cargo run -p delulu-survey -- check` and `cargo run -p delulu -- doctor --check` on Windows and in
WSL. **Memory:** every lesson is in `HANDOFF.md` §11 (§11.4 the findings, §11.5 the traps); `docs/assistant-memory/`
changed in two files (`cloud-period-2026-09-28.md`, `delulu-ipc-deadman-findings.md`) — copy both into the laptop's memory
directory, or have the laptop session read §11.

**The routine** is disabled (read 2026-10-17: `enabled: false`; its last run, run 25, ended 2026-10-15 and succeeded),
and **the laptop was synced on 2026-10-17** — the entry of that day. Were it enabled again, a fire on or after
2026-10-16 only checks this section and stops, unless the owner extends the dates.

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

### 2026-10-10 — routine run 16 (postscript): push runs read green; the red-team pass lost to a restart; RW 4.53 measured
- Session: `https://claude.ai/code/session_01FM5pjaFXk3YSik7zQzQXxh`   Model: Claude Opus 5.5 (no fallback notice); the lost
  red-team sous-chef ran on Sonnet 5.5 (`sonnet`), against a frozen copy of the binary under `/tmp` — `git status` clean after it
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `dd724f6`
- Commits: this one — records only
- Files and folders: M `HANDOFF.md` (§11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 5: a sous-chef appends findings to a
  file as it goes), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.53 opened), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks
  passed
- Verified: push runs `d7e2b8f` `38016042725` and `dd724f6` `38016175557` — success on every job, so `master` is green at
  the close. The latency table in `V2_LOG.md` (postscript), measured on both binaries by hand. The red-team pass: lost to the
  run's second container restart, no finding recorded.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 (a sous-chef's findings must reach a file as they are found).
- Open / next, in order: (1) read THIS commit's push run (unread at the close) and the 2026-10-10 nightly; (2) **redo the
  red-team pass** on run 16's four slices, briefed to write `FINDINGS.md` as it goes; (3) **P8-04 step 5, option (c)** — sized in
  `V2_LOG.md` 2026-10-10 (closing); (4) RW 4.53 (witness the e-stop's latency beside busy sandbox hosts first), RW 4.46, RW
  4.49. **For the owner:** D-V2-103, D-V2-104 (with D-V2-27 item 4's correction), D-V2-105. **On 2026-10-12 a run starts the
  closing entry for the cloud period.**

### 2026-10-10 — routine run 17: CI read green; P8-04 step 5 — a dead monitor quarantines (D-V2-106); the red-team pass redone — CLOSURE-SCOPE-1 found (HIGH)
- Session: `https://claude.ai/code/session_01WzxFaZVw8HRNkGs9neji5s`   Model: Claude Opus 5.5 (no fallback notice); the red-team
  sous-chef ran on Sonnet 5.5 (`sonnet`) against a frozen copy of the binary in `/tmp/redteam17` — `git status` clean after it
- Branch: `master` (through the harness branch `claude/happy-pascal-f86pm9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `ed7f7b6`
- Commits: (1) `c299e47` P8-04 step 5: a dead monitor quarantines — a dead-man for its node, held by the broker (with the
  Linux memory witness and NOT its fix, read red on a runner); (2) `3e9c496` D-V2-106 item 2: a monitor holding a dead-man's key
  is non-dumpable on Linux; (3) this entry's commit — the records
- Files and folders: (1) A `crates/delulu-broker/src/deadman.rs`; M `crates/delulu-broker/src/lib.rs`,
  `crates/delulu-broker/src/tree.rs` (the `deadmen` field), `crates/delulu/src/broker_ipc.rs` (`ArmDeadman`, `Beat`,
  `DisarmDeadman`, `DeadmanArmed`), `crates/delulu/src/brokerd.rs` (the serve loop's timer, `fire_deadmen`,
  `persist_adoption_revocations`, three unit tests), `crates/delulu/src/cli.rs` (the help), `crates/delulu/src/monitor_cmd.rs`,
  `crates/delulu/tests/json_contract.rs`, `crates/delulu/tests/monitor_cli.rs` (five witnesses), `docs/survey/*`. (2) M
  `crates/delulu/src/monitor_cmd.rs`, `docs/survey/*`. (3) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 two
  lessons), `docs/CLOUD_ROUTINE.md` (step 4's P8 line; step 5 — a non-root witness is read on `ubuntu-24.04`),
  `docs/DEPLOYMENT.md` (the monitor's death), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-106),
  `V2_PHASE_STATUS.md`, `V2_P8_DESIGN.md` (step 5 built), `docs/REMAINING_WORK.md` (4.54 closed; 4.55–4.58 opened; 4.53
  extended), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: CI first — run 16's closing commit `ed7f7b6`, push run `38017544108`, success; the 2026-10-10 nightly had not fired
  at 05:08 UTC. Every `origin/claude/*` branch checked after unshallowing: nothing stranded. The witnesses red first on the
  pre-fix monitor (a killed monitor's run still live 20 s later); the memory witness red on a non-root runner at `c299e47`
  (`38028218741`). Mutants M199–M207 red (M201 after its witness was strengthened), controls green. Clippy clean;
  `check-other-os.sh` clean for Windows and macOS. Runner reads at `3e9c496`: ubuntu-24.04 `38028453751` (213 passed),
  macOS `38028455193` (193), Windows `38028456795` (177), arm64 `38028458119` (213), `delulu-broker` on Windows `38028459469`
  (7) — 0 failed, every new witness named. The full suite alone at `3e9c496`: 2,194 passed, 0 failed, 16 ignored (161
  binaries), exit 0, the tree unmoved. The red-team pass: F-03 and F-07 re-run and reproduced by the head chef; F-01, F-06
  by code reading; F-04, F-05 not re-run (each says so in `REMAINING_WORK.md`).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (a witness of an order must build the state where the order matters; a
  same-user-but-not-root witness is read on a runner).
- Open / next, in order: (1) read `3e9c496`'s and this commit's push runs and the 2026-10-10 nightly; (2) **RW 4.55,
  CLOSURE-SCOPE-1 (HIGH)** — this run's next slice; if it does not land, it is the next run's first item; (3) **RW 4.56** (a
  plugin node's revoke reaches the next CALL only); (4) RW 4.58, 4.57, 4.53; (5) RW 4.46 and P8-04's two remaining rules.
  **For the owner:** D-V2-106 (and D-V2-103 to D-V2-105). **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-10 — routine run 17 (continued): CLOSURE-SCOPE-1 closed — a closure's names are its own module's (D-V2-107, RW 4.55)
- Session: `https://claude.ai/code/session_01WzxFaZVw8HRNkGs9neji5s`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-f86pm9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `02cc186`
- Commits: (1) `c83ebe6` CLOSURE-SCOPE-1 (RW 4.55): a closure resolves its names in its own module; (2) this entry's commit —
  the records
- Files and folders: (1) M `crates/delulu-runtime/src/interp.rs` (`Home`; `call_closure` swaps and restores it),
  `crates/delulu-runtime/src/value.rs` (`Closure::home`), `crates/delulu-runtime/src/actors.rs`, `crates/delulu-runtime/src/cycles.rs`,
  `crates/delulu/tests/plugin_load_cli.rs` (the witness), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things
  stand; §11.4 CLOSURE-SCOPE-1), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-107), `docs/REMAINING_WORK.md` (4.55
  closed), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: the red team's F-03 re-run on `3e9c496` before it was recorded (a plugin with an all-empty grant wrote its own file
  through a host callback's `FsWrite`); the witness red first ("callback answered 1000"); the red team's three repros by hand on
  the fix, the secret one under the daemon ("host redacted the secret"); mutants M208–M211 red, control green; clippy clean;
  `check-other-os.sh` clean for Windows and macOS; runner reads at `c83ebe6` — macOS `38029968775`, Windows `38029970294`, arm64
  `38029972153`, `delulu-runtime` on Windows `38029973402` — green, the witness named; the full suite alone at `c83ebe6`: 2,195
  passed, 0 failed, 16 ignored (161 binaries). Push runs: `3e9c496`'s `openshell` `38029067306` and `ocsf` `38029067286` — success;
  its CI run `38029067385` and `02cc186`'s `38029490424` were running at this commit.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (CLOSURE-SCOPE-1, with its search key).
- Open / next: (1) read the push runs of `3e9c496`, `02cc186`, `c83ebe6` and this commit, and the 2026-10-10 nightly; (2) **RW
  4.56** — this run's next slice if it lands (a gate on the plugin's node at each use the export makes); (3) RW 4.58, 4.57, 4.53;
  (4) RW 4.46 and P8-04's two remaining rules. **For the owner:** D-V2-106, D-V2-107.

### 2026-10-10 — routine run 17 (continued): RW 4.56 closed — a plugin node's revoke reaches its running export (D-V2-108); `master` red once on this run's own witness, fixed
- Session: `https://claude.ai/code/session_01WzxFaZVw8HRNkGs9neji5s`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-f86pm9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `9be433d`
- Commits: (1) `ff91be5` RW 4.56: revoking a plugin's node stops its running export at the next use; (2) `a76018f` the
  busy-daemon witness: a baseline the loop's state needs, not a throughput; (3) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-runtime/src/custody.rs` (`PluginUseCustody`), `crates/delulu-runtime/src/interp.rs`
  (`call_plugin_export`), `crates/delulu/tests/plugin_load_cli.rs` (the witness), `docs/survey/*`. (2) M
  `crates/delulu/src/brokerd.rs` (two unit tests), `docs/survey/*`. (3) M `CHANGELOG.md`, `HANDOFF.md` (where things stand;
  §11.4 RW 4.56; §11.5 the order lesson rewritten and a baseline lesson), `docs/CLOUD_ROUTINE.md` (rules: a baseline is a state,
  never a throughput), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-108), `docs/REMAINING_WORK.md` (4.56 closed),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: RW 4.56's witness red first (3 before the revoke, 117 after), green (0 after); M212–M214 red, control green; runner
  reads at `ff91be5` — macOS `38030868405`, Windows `38030870802`, arm64 `38030872242`, 31 passed and 0 failed on each. **`master`
  red once:** `3e9c496`'s push run `38029067385`, `test (ubuntu-24.04)` — the busy-daemon witness's baseline (200 requests in
  1.6 s; 45 answered); `02cc186`'s `38029490424` and `9be433d`'s `38030530082` green by chance on the same code. M201 then survived
  on a quiet VM (its first red had come under the red team's load); fixed in `a76018f` — M201 red 4 of 4, M200 and M204 red, the
  test green 3 of 3 with all four CPUs busy; read at `a76018f` on macOS `38031308292` and Windows `38031306607`. The full suite
  alone at `a76018f`: 2,196 passed, 0 failed, 16 ignored (161 binaries).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (RW 4.56) and §11.5 (an order witness deterministically; a baseline is a state).
- Open / next: (1) read the push runs of `a76018f` and this commit (`c83ebe6`'s `38030475956` and `9be433d`'s `38030530082`
  read green at this commit), and the 2026-10-10 nightly; (2) RW 4.58 (a lost audit record answered as success), RW 4.57 (a killed run's nodes), RW 4.53 (the chain's
  catch-up under contention); (3) RW 4.46 and P8-04's two remaining rules; (4) P9. **For the owner:** D-V2-106, D-V2-107,
  D-V2-108. **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-10 — routine run 17 (closing): RW 4.58 closed for revocations and mints — a lost audit record is never answered as success (D-V2-109); the run's close
- Session: `https://claude.ai/code/session_01WzxFaZVw8HRNkGs9neji5s`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/happy-pascal-f86pm9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `ada8fd8`
- Commits: (1) `75e748e` RW 4.58: a lost audit record is never answered as success (revocations and mints); (2) this entry's
  commit — the records and the loop's new line
- Files and folders: (1) M `crates/delulu-broker/src/tree.rs` (`withdraw_unrecorded`), `crates/delulu/src/broker_ipc.rs`
  (`Revoked.unrecorded`), `crates/delulu/src/brokerd.rs` (`unrecorded_mint`; the revoke's report; a witness),
  `crates/delulu/src/cli.rs` (`grants revoke` warns and exits 1), `crates/delulu/src/monitor_cmd.rs`,
  `crates/delulu/tests/monitor_cli.rs` (a witness), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand),
  `docs/CLOUD_ROUTINE.md` (step 5: a timing read during a sous-chef's load experiments is not evidence),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-109), `docs/REMAINING_WORK.md` (4.58 partly closed),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: both witnesses red first (a revoke answered `Revoked { by_seq: 3 }` for a seq never written; a mint answered
  `Issued`); M215–M218 red, controls green; clippy clean; `check-other-os.sh` clean for Windows and macOS; runner reads at
  `75e748e` — Windows `38032573759`, macOS `38032575295`, arm64 `38032576941` — green, both witnesses named; the full suite
  alone at `75e748e`: 2,198 passed, 0 failed, 16 ignored (161 binaries). Push runs read this run, all success unless said:
  `ed7f7b6` `38017544108`; `3e9c496` `38029067385` — **failure** (the busy-daemon witness's baseline, fixed in `a76018f`) with
  its `openshell` `38029067306` and `ocsf` `38029067286` green; `02cc186` `38029490424`; `c83ebe6` `38030475956`; `9be433d`
  `38030530082`; `a76018f` `38031803525`; `ada8fd8` `38031871369`. `75e748e`'s and this commit's push runs are unread at the
  close.
- **The run in one paragraph:** P8-04 step 5 built (D-V2-106 — a dead monitor quarantines; the broker holds a dead-man for its
  node); run 16's red-team pass redone, finding CLOSURE-SCOPE-1 (HIGH) and five more; RW 4.55 (D-V2-107), RW 4.56 (D-V2-108)
  and RW 4.58 for revocations and mints (D-V2-109) closed, each witnessed red first, mutated and read on the runners; `master`
  red once, on this run's own witness's throughput baseline, fixed within the run.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (CLOSURE-SCOPE-1, RW 4.56) and §11.5 (this run's four lessons).
- Open / next, in order: (1) read `75e748e`'s and this commit's push runs and the 2026-10-10 nightly; (2) RW 4.57 (a killed
  run's nodes stay live), RW 4.53 (the chain's catch-up under contention — the red team measured a false DL1405 at 60,000
  records), RW 4.58's rest; (3) RW 4.46 and P8-04's `special-use` and `break-glass` rules; (4) P9. **For the owner:** D-V2-106,
  D-V2-107, D-V2-108, D-V2-109 (and D-V2-103 to D-V2-105). **On 2026-10-12 a run starts the closing entry for the cloud period**
  (the routine's prompt).

### 2026-10-10 — routine run 17 (postscript): the last push runs read; RW 4.59 found (macOS, open)
- Session: `https://claude.ai/code/session_01WzxFaZVw8HRNkGs9neji5s`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `f0a39e7`
- Commits: this one — records only
- Files and folders: M `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.59 opened), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `f0a39e7`'s push run `38033226509` — success, so `master` is green at the close; `75e748e`'s `38033108211` — failure on
  `test (macos-latest)`, one test (`sandbox_devices_cli` `a_wedged_guest_that_never_asks_again_still_loses_its_actuator`: the
  guest's first command 28 ms past a 150 ms heartbeat); not reproduced on Linux under six busy loops (5 of 5); recorded as RW 4.59.
- Redo on the laptop: nothing.
- Open / next, in order: (1) read THIS commit's push run (unread at the close) and the 2026-10-10 and 2026-10-11 nightlies;
  (2) **RW 4.59** — measure on a macOS runner first; (3) RW 4.57, RW 4.53, RW 4.58's rest; (4) RW 4.46 and P8-04's two remaining
  rules; (5) P9. **For the owner:** D-V2-106 to D-V2-109. **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-11 — routine run 18: CI read green; RW 4.59 closed — a guest's devices start at its first request (D-V2-110)
- Session: `https://claude.ai/code/session_0129srJbQbPEieN9QdR4wHqx`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-v2rh0i`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `b77c64d`
- Commits: (1) `b0a520c` RW 4.59 witness: a guest's own check of its program is charged against its first heartbeat (the witness
  alone, read red on the runners); (2) `25a53d0` RW 4.59: a guest's devices start at its first request, not at the send; (3) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu/tests/sandbox_devices_cli.rs` (two witnesses, the measurement), `docs/survey/*`. (2) M
  `crates/delulu-runtime/src/channel.rs` (`with_devices_at_first_request`, `devices`), `crates/delulu/src/guest.rs` (`GuestDevices`;
  the close revokes the device nodes when the devices never started), `crates/delulu/src/run_cmd.rs` (the `DevicePlan` doc),
  `crates/delulu/tests/sandbox_devices_cli.rs` (a comment), `docs/survey/*`. (3) M `CHANGELOG.md`, `HANDOFF.md` (where things
  stand; §11.4 RW 4.59; §11.5 two lessons), `docs/CLOUD_ROUTINE.md` (step 4's P8 line; step 5 — a flake seen once is first a
  question about its window), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-110; D-V2-95 item 4 annotated),
  `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.59 closed), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply
  address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: CI first — `b77c64d`'s push run `38034360521` success; the 2026-10-10 nightly `38040982137` and the 2026-10-11 nightly
  `38128985173` success on all 18 jobs (this Sunday's sweep's CI read). Every `origin/claude/*` branch checked after unshallowing:
  nothing stranded (`7d6a22b`, `d3f0d52` superseded since run 10). Survey `ok` and `doctor --check` all passed at the start;
  `cargo deny … advisories` ok; `gh` present, token refused. The witness red first in the VM and on the runners at `b0a520c` —
  Windows `38152502475`, macOS `38152500629`; M219–M221 red, M222 equivalent (its unreachable guard removed), control green; clippy
  clean; `check-other-os.sh` clean for Windows and macOS; runner reads at `25a53d0` green — Windows `38153193744`, arm64
  `38153195727`, macOS `38153204486` (the whole package: 932 passed) — the witnesses named on each; the full suite alone at
  `25a53d0`: 2,200 passed, 0 failed, 16 ignored (161 binaries), exit 0, the tree unmoved. The Sunday sweep's document scan: no open
  `REMAINING_WORK.md` row that `V2_LOG.md` calls closed; four live passages that said a guest's devices start at the send, updated.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (RW 4.59) and §11.5 (a flake seen once may be a design asymmetry — grow its window;
  a mutant surviving on unreachable code is a reason to delete the code).
- Open / next, in order: (1) read this commit's push run (unread at this entry); (2) RW 4.58's rest — `DeviceRefused` answers
  `Recorded { seq }` for a record the chain never got (the client's "said once on stderr" path never fires), then `Adopt`, the
  dead-man's arm and disarm, the guard's records; (3) RW 4.57 (a killed run's nodes), RW 4.53; (4) RW 4.46 and P8-04's two
  remaining rules; (5) P9. **For the owner:** D-V2-110 (and D-V2-106 to D-V2-109). **On 2026-10-12 a run starts the closing entry
  for the cloud period.**

### 2026-10-11 — routine run 18 (continued): RW 4.58's rest for device refusals and adoptions; one wait for the chain per request (D-V2-111)
- Session: `https://claude.ai/code/session_0129srJbQbPEieN9QdR4wHqx`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-v2rh0i`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `439fdbb`
- Commits: (1) `aa20498` RW 4.58's rest: a lost device-refusal or adoption record is never answered as success; (2) this entry's
  commit — the records
- Files and folders: (1) M `crates/delulu/src/brokerd.rs` (`ChainLock`; `Adopt` and `DeviceRefused`; two witnesses),
  `crates/delulu-broker/src/tree.rs` (`withdraw_unrecorded` clears an adoption), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md`
  (where things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P8 line), `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-111), `docs/REMAINING_WORK.md` (4.58), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply
  address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `439fdbb`'s push run `38154053591` and `ocsf` `38154053594` — success. Both witnesses red first (a lost refusal answered
  `Recorded { seq: 4 }`; a held-chain adoption past the client's 15 s deadline); M223–M226 red, control green; clippy clean;
  `check-other-os.sh` clean for Windows and macOS; runner reads at `aa20498` — Windows `38154807791`, macOS `38154809795`, arm64
  `38154811843` — green, the witnesses named; the full suite alone at `aa20498`: 2,202 passed, 0 failed, 16 ignored (161 binaries).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (a wait per operation inside one request is that wait times the operations).
- Open / next, in order: (1) read this commit's push run (unread at this entry); (2) RW 4.58's last parts (the dead-man's arm and
  disarm, the guard's records), RW 4.57 (a killed run's nodes), RW 4.53; (3) RW 4.46 and P8-04's two remaining rules; (4) P9.
  **For the owner:** D-V2-110, D-V2-111 (and D-V2-106 to D-V2-109). **On 2026-10-12 a run starts the closing entry for the cloud
  period.**

### 2026-10-11 — routine run 18 (continued): RW 4.46 closed — an egress refusal is a `deny` record citing its `allow` (D-V2-112)
- Session: `https://claude.ai/code/session_0129srJbQbPEieN9QdR4wHqx`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-v2rh0i`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `9c758c3`
- Commits: (1) `ced5dc4` RW 4.46: an egress refusal is a deny record citing the use it overrides; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-broker/src/validate.rs` (`record_egress_refusal`), `crates/delulu-runtime/src/channel.rs`,
  `crates/delulu-runtime/src/custody.rs` (`note_egress_refusal`), `crates/delulu-runtime/src/egress.rs` (`refusals_since`),
  `crates/delulu-runtime/src/interp.rs`, `crates/delulu/src/broker_client.rs`, `crates/delulu/src/broker_ipc.rs` (`EgressRefused`),
  `crates/delulu/src/brokerd.rs` (the handler; a witness), `crates/delulu/tests/monitor_cli.rs` (two witnesses), `docs/survey/*`. (2) M
  `CHANGELOG.md`, `HANDOFF.md` (where things stand), `docs/CLOUD_ROUTINE.md` (step 4's P8 line), `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-112), `V2_P8_DESIGN.md` (step 4), `V2_PHASE_STATUS.md`, `docs/REMAINING_WORK.md` (4.46 closed),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the
  owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `9c758c3`'s push run `38155661919` and `ocsf` `38155661891` — success. The witnesses red on the pre-fix behaviour (M227,
  M228); M227–M231 red, control green; the special-use case by hand in the VM through a one-run `/etc/hosts` entry (restored); clippy
  clean; `check-other-os.sh` clean for Windows and macOS; runner reads at `ced5dc4` — macOS `38156513763`, Windows `38156516142`, arm64
  `38156518032` — green, the witnesses named; the full suite alone at `ced5dc4`: 2,205 passed, 0 failed, 16 ignored (161 binaries).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond the snapshot line.
- Open / next, in order: (1) read this commit's push run (unread at this entry); (2) P8-04's `special-use` rule (this run's next slice
  if it lands), then `break-glass`; (3) RW 4.58's last parts, RW 4.57, RW 4.53; (4) P9. **For the owner:** D-V2-110 to D-V2-112.
  **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-11 — routine run 18 (continued): P8-04 step 4's `special-use` rule (D-V2-113); the `envelope` rule seen firing at last
- Session: `https://claude.ai/code/session_0129srJbQbPEieN9QdR4wHqx`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-v2rh0i`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `8084e4e`
- Commits: (1) `db40109` P8-04 step 4: the monitor's special-use rule, and every single-record rule decided in one place; (2) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/monitor_cmd.rs` (`Rule::SpecialUse`, `fires_on`; a unit witness), `crates/delulu/src/cli.rs`
  (the help), `crates/delulu/tests/monitor_cli.rs` (the envelope rule firing), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md`
  (where things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P8 line), `docs/DEPLOYMENT.md` (the monitor's rules),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-113), `V2_P8_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the
  owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `8084e4e`'s `openshell` and `ocsf` runs success (its CI run running at this entry); the rule by hand in the VM through a
  one-run `/etc/hosts` entry (restored); M232–M234 red (M233 red only once the new integration witness existed), control green;
  clippy clean; `check-other-os.sh` clean for Windows and macOS; runner reads at `db40109` — Windows `38157840795`, macOS `38157843623`,
  arm64 `38157845681` — green, both witnesses named; the full suite alone at `db40109`: 2,207 passed, 0 failed, 16 ignored.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (a rule watched only for what it must not do was never seen to do anything).
- Open / next, in order: (1) read `8084e4e`'s CI run and this commit's push run; (2) the red-team pass on this run's four guarantees
  (Sonnet 5.5, running at this entry); (3) P8-04's `break-glass` rule; RW 4.58's last parts, RW 4.57, RW 4.53; (4) P9. **For the
  owner:** D-V2-110 to D-V2-113. **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-11 — routine run 18 (closing): the Sunday sweep's red-team pass — F1/F2 and F3 fixed the same run; RW 4.60 and 4.61 opened
- Session: `https://claude.ai/code/session_0129srJbQbPEieN9QdR4wHqx`   Model: Claude Opus 5.5 (no fallback notice); the red-team sous-chef
  ran on Sonnet 5.5 (`sonnet`) against a frozen copy of the binary in `/tmp/redteam18` — `git status` clean of it afterwards, `/etc/hosts`
  restored
- Branch: `master` (through the harness branch `claude/zealous-keller-v2rh0i`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `f875a6b`
- Commits: (1) `bc124e9` RW 4.46 again (red team F1/F2): a refusal is read from the call's own outcome, not the bounded log; (2)
  `db70f1f` Red team F3: an e-stop made during a guest's compute stops its first command; (3) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-runtime/src/egress.rs` (`fetch_refusal`, the thread's last refusal),
  `crates/delulu-runtime/src/interp.rs`, `crates/delulu-runtime/src/channel.rs`, `crates/delulu/tests/monitor_cli.rs` (two witnesses),
  `docs/CLOUD_ROUTINE.md` (step 5: a patch script checks every anchor first; a mutant's own command is part of it), `docs/survey/*`. (2) M
  `crates/delulu-runtime/src/device.rs` (`estop_sweep`, synchronous at construction), `crates/delulu/tests/sandbox_devices_cli.rs` (a
  witness), `CHANGELOG.md`, `HANDOFF.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-112 item 5), `docs/REMAINING_WORK.md` (4.46, 4.58
  notes; 4.60 opened), `docs/survey/*`. (3) M `HANDOFF.md` (where things stand; §11.4 the run's own fixes broken by its pass; §11.5 two
  lessons), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-110 item 5), `docs/REMAINING_WORK.md` (4.59 corrected; 4.53 F4/F6;
  4.61 opened), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub
  no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `f875a6b`'s push runs (CI `38158626570`, `openshell` `38158626534`) — success. F1 re-run on the frozen binary (128 `deny`s of
  140) and green on the fix (140 of 140); M235/M236 red; read at `bc124e9` — Windows `38159257751`, macOS `38159260568`, arm64
  `38159262871` — green. F3's witness red on M237 3 of 3 (twice on a quiet VM), green on the fix; read at `db70f1f` — Windows
  `38159975775`, macOS `38159978489`, arm64 `38159981058` — green. F5 re-run by the head chef on the current binary (reproduced). Clippy
  clean; `check-other-os.sh` clean for Windows and macOS after each code edit. The full suite alone at `db70f1f` with this entry's record edits applied: 2,206 passed, 4 failed — the
  stale-map four (`doctor_cli` ×3, `freshness`): the records were applied after the last map build and before the suite (§11.5's
  trap, met again); after `survey build` those four and the document gates re-run green.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (the run's own fixes broken by its pass) and §11.5 (moving when a mechanism starts moves
  every check it carries; a fix that reads evidence from a bounded store inherits the bound; a rule watched only for what it must not
  do; a mutant surviving on unreachable code).
- **The run in one paragraph:** CI green at the start (push and both nightlies); RW 4.59 found to be a design asymmetry and closed
  (D-V2-110); RW 4.58's rest for device refusals and adoptions, with one wait per request (D-V2-111); RW 4.46 closed with a hermetic
  witness (D-V2-112); P8-04 step 4's `special-use` rule built (D-V2-113) and the `envelope` rule seen firing at last; the Sunday sweep's
  red-team pass broke two of the run's own fixes (F1/F2, F3 — both fixed and read on the runners the same run) and found RW 4.61.
- Open / next, in order: (1) read THIS commit's push run (unread at this entry); (2) **RW 4.61** — guard operations and the dead-man's arm
  answer success with no record (HIGH for the guard; reproduced): a downgrade rolled back and refused, an upgrade said `unrecorded`, a
  witness per handler; (3) P8-04's `break-glass` rule — its record is written at the gate in `run_cmd.rs` before the lease is redeemed,
  so it has no node, and the monitor's loop reads `deny` records only: record the run's node with the use (after `redeem_lease`) and let
  a rule read that `allow`; (4) RW 4.57, RW 4.53 (with F4 and F6 — an incremental chain read for the monitor), RW 4.60, RW 4.58's last
  parts (O2/O4); (5) P9. **For the owner:** D-V2-110 to D-V2-113. **On 2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-11 — routine run 18 (postscript): RW 4.61's guard bypass closed (D-V2-114); the closing commit's push runs read
- Session: `https://claude.ai/code/session_0129srJbQbPEieN9QdR4wHqx`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-v2rh0i`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `00112e5`
- Commits: (1) `d908884` RW 4.61, the guard bypass: a bypass whose record was lost does not happen; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/brokerd.rs` (`GuardBypass`; a witness), `crates/delulu-broker/src/guard.rs`
  (`withdraw_unrecorded_bypass`), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-114), `docs/REMAINING_WORK.md` (4.61 partly closed), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: the witness red under M238 (the pre-fix answer), M239, M240; control green; clippy and `check-other-os.sh` clean.
  **Read on the runners at `d908884`** over `bin:delulu`, `guard_cli` and `guard_e2e`: Windows `38161584149` (165 passed), macOS `38161586576` (179), arm64 `38161588717` (198) — 0 failed, the witness named on each. **The full suite alone at `d908884`:** 2,211 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved. **The closing commit `00112e5`** (every code change of this run before `d908884`): push run `38161113083` — success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38161113026` success; `db70f1f`, `bc124e9` and the records commits between were pushed to `master` only inside `00112e5`'s fast-forward.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new.
- Open / next, in order: (1) read THIS commit's push run (unread at this entry); (2) RW 4.61's rest (the guard's policy edits,
  approvals, requests and permits; the dead-man's arm; `Redeem`); (3) P8-04's `break-glass` rule (the closing entry above says
  how); (4) RW 4.57, RW 4.53 (with F4 and F6), RW 4.60, RW 4.58's last parts; (5) P9. **For the owner:** D-V2-110 to D-V2-114. **On
  2026-10-12 a run starts the closing entry for the cloud period.**

### 2026-10-12 — routine run 19: CI read green; the cloud period's closing entry begun (four days early)
- Session: `https://claude.ai/code/session_01DeHytTS7xJQjv5K5ddrWP2`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-kkuv5g`, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `e95acf1`
- Commits: (1) this entry's commit — the closing entry and the routine's step 0 and step 6
- Files and folders: M `docs/CLOUD_SYNC_LOG.md` (the section *The closing entry*, before the template; this entry),
  `docs/CLOUD_ROUTINE.md` (step 0: where the closing entry lives and that every run keeps it current; step 6: the same),
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as
  co-author.
- Survey and doctor: at the start `survey check` ok (1,500 nodes, 13,892 edges), `doctor --check` all checks passed; after
  the last edit — the same (this commit's map rebuilt last; the pre-commit hook checked it against the index).
- Verified: run 18's closing push runs — CI `38162468215` on `e95acf1`, success on every job (`miri-slow` and `heavy-gates`
  skipped on a push), `ocsf` `38162468217` success; the 2026-10-12 nightly CI `38234415113` — all 18 jobs success
  (`heavy-gates`, the three `miri-slow` jobs, `microvm`, `microvm-reproducible`); the nightly release dry run `38248184651`
  success. `actors_pingpong` (annotations): push run — arm64 MEASURED 2.88x (control 4.11x, bar 1.50x); macOS NOT MEASURED
  (3 hardware threads); Windows and Ubuntu NOT MEASURED (busy or SMT); the nightly the same (arm64 2.88x, control 4.12x).
  Sandbox properties per OS as recorded since D-V2-57 (macOS: `filesystem_confinement absent` — its reads). Run 18's
  witness `a_guard_bypass_whose_record_was_lost_does_not_happen` re-run in the VM — green. Branches: every `origin/claude/*`
  read against `master` after `git fetch --unshallow`; the two with a commit `master` lacks (`7d6a22b`, `d3f0d52`) are
  superseded since run 10 — nothing stranded. `gh`: present at `/usr/local/bin/gh`, not needed (the REST API answers
  `curl`).
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: this run continues — RW 4.61's rest first (the newest entry after this one says how far it got).

### 2026-10-12 — routine run 19 (continued): RW 4.61 closed — no daemon operation is answered as recorded when its record was lost (D-V2-115)
- Session: `https://claude.ai/code/session_01DeHytTS7xJQjv5K5ddrWP2`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-kkuv5g`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `5bd440d`
- Commits: (1) `96b7b87` RW 4.61's rest: no daemon operation is answered as done when its record was lost; (2) this entry's
  commit — the records
- Files and folders: (1) M `crates/delulu-broker/src/guard.rs` (`GuardPolicy::loosened_by`; `withdraw_unrecorded_policy_edit`,
  `_approval`, `_request`), `deadman.rs` (`Disarmed`, `disarm_deadman_undoable`, `rearm_unrecorded_disarm`,
  `withdraw_unrecorded_arming`), `lease.rs` (`Redemption`, `redeem_undoable`, `withdraw_unrecorded_redemption`), `cert.rs`
  (`Renewal`, `renew_undoable`, `withdraw_unrecorded_renewal`), `tree.rs` (`unmark_redeemed`, `restore_ttl`), `secrets.rs`
  (`remove_derived`), `crates/delulu/src/brokerd.rs` (twelve handlers; `unrecorded_undone`, `unrecorded_in_force`,
  `guard_policy_edited`; seven witnesses), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.4 RW 4.61;
  §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P8-04 line), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-115),
  `docs/REMAINING_WORK.md` (4.61 closed), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md` (this
  entry; the closing entry refreshed), `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub
  no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: seven witnesses red on the code before; M241–M257 red, control green; clippy clean; `check-other-os.sh` clean for
  Windows and macOS. Read on the runners at `96b7b87` over `bin:delulu`, `guard_cli`, `guard_e2e`, `federation_cli`: Windows
  `38274676231` (181 passed), macOS `38274680188` (195), arm64 `38274684944` (214) — 0 failed, the witnesses named on each. The
  full suite alone at `96b7b87`: 2,218 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (RW 4.61) and §11.5 (enumerate a class from its mechanism, not from the finding's
  list); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next, in order: (1) read this commit's push run; (2) P8-04's `break-glass` rule (this run, next); (3) RW 4.57, RW 4.53
  (with F4 and F6), RW 4.60, RW 4.58's last parts (O2/O4); (4) P9. **For the owner:** D-V2-115.

### 2026-10-12 — routine run 19 (continued): P8-04's `break-glass` rule — every step of P8-04 built (D-V2-116)
- Session: `https://claude.ai/code/session_01DeHytTS7xJQjv5K5ddrWP2`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-kkuv5g`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `cad7c47`
- Commits: (1) `0d3f514` P8-04's break-glass rule: a monitor quarantines a run that breaks the glass; (2) this entry's commit —
  the records
- Files and folders: (1) M `crates/delulu-broker/src/validate.rs` (`record_break_glass_run`), `crates/delulu/src/broker_ipc.rs`
  (`ReqBody::BreakGlassRun`), `crates/delulu/src/brokerd.rs` (its handler; two witnesses), `crates/delulu/src/run_cmd.rs`
  (`bind_break_glass_run`), `crates/delulu/src/monitor_cmd.rs` (`Rule::BreakGlass`; `fires_on` reads the decision; the loop),
  `crates/delulu/src/cli.rs` (the help line), `crates/delulu/tests/monitor_cli.rs` (a witness), `docs/survey/*`. (2) M
  `CHANGELOG.md`, `HANDOFF.md` (where things stand), `docs/CLOUD_ROUTINE.md` (step 4: P8-04 complete), `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-116), `V2_P8_DESIGN.md` (steps 4 and 7 built), `V2_PHASE_STATUS.md` (P8's row),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`.
  Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `cad7c47`'s push runs (CI `38276195408`, `ocsf` `38276194840`) and `5bd440d`'s CI (`38270912985`) — success. The witness
  red before the fix; M258–M265 red, controls green; clippy and `check-other-os.sh` (Windows, macOS) clean. Read on the runners at
  `0d3f514` over `bin:delulu`, `monitor_cli`, `break_glass_cli`: Windows `38277449931` (190 passed), macOS `38277453631` (205), arm64
  `38277459675` (225) — 0 failed. The full suite alone at `0d3f514`: 2,222 passed, 0 failed, 16 ignored (161 binaries), cargo exit
  0, the tree unmoved.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond D-V2-116.
- Open / next, in order: (1) read this commit's push run, and record P8 complete in `V2_PHASE_STATUS.md` with it; (2) **P9** —
  P9-01 `authority --within` first (`V2_OPENSHELL_STUDY.md` §4.8); (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.60, RW 4.58's
  O2/O4, RW 4.49. **For the owner:** D-V2-115, D-V2-116 (and option (c), a revoke-only principal in the tree).

### 2026-10-12 — routine run 19 (closing): P8 complete as far as software reaches; P9-01's build order read against the code; the loop's new lines
- Session: `https://claude.ai/code/session_01DeHytTS7xJQjv5K5ddrWP2`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef
  this run)
- Branch: `master` (through the harness branch `claude/zealous-keller-kkuv5g`, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `9000c57`
- Commits: (1) this entry's commit — the records, the P9 design and a script
- Files and folders: A `docs/DELULULANG_V2/V2_P9_DESIGN.md` (P9-01's build order read against the code),
  `scripts/authority-corpus.py` (the corpus's required grants, classified, with a control); M `docs/REPOSITORY_STRUCTURE.md`
  (both listed), `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (P8 complete; P9's row), `docs/DELULULANG_V2/V2_LOG.md`, `HANDOFF.md`
  (where things stand), `docs/CLOUD_ROUTINE.md` (step 5: three lines), `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry),
  `docs/survey/*`. Deleted: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `9000c57`'s push runs — CI `38278927092` success on every job, `ocsf` `38278927078`, `openshell` `38278927163` success
  (`actors_pingpong`: arm64 MEASURED 2.95x, control 4.12x; the others NOT MEASURED, as every run). `scripts/authority-corpus.py`
  run over the corpus (23 programs: 6 pure, 12 literal, 4 grant data, 1 computed — exit 0) and against one pure program (its
  control fails, exit 1). The document gates and the Survey's tests after the records.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 (this run's lesson, in the earlier entry); nothing new here.
- **The run in one paragraph:** CI green at the start (push, both nightlies); the cloud period's closing entry begun four days
  early and kept current; RW 4.61 closed — twelve daemon operations that answered success for a lost record, four beyond the red
  team's list (D-V2-115); P8-04's `break-glass` rule and step 7's last witness (D-V2-116) — **P8 complete as far as software
  reaches**, its push run read green; P9-01's build order read against the code, the corpus measured.
- Open / next, in order: (1) read THIS commit's push run (unread at this entry); (2) **P9-01** — build step 1 (spans in the
  walk) and step 2 (the verdict as a pure function) from `V2_P9_DESIGN.md`, then the surface and the corpus witnesses; take the
  hidden-site rule as a `D-V2-nn` when built; (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.60, RW 4.58's O2/O4, RW 4.49; (4) the
  Sunday sweep (2026-10-18 if the period is extended). **For the owner:** D-V2-115, D-V2-116, option (c). **Keep the closing
  entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 19 (postscript): RW 4.60 closed (D-V2-117); the break-glass witness's race fixed
- Session: `https://claude.ai/code/session_01DeHytTS7xJQjv5K5ddrWP2`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-kkuv5g`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `3a91ab7`
- Commits: (1) `11b48cc` RW 4.60: a run report says when a guest's devices never started; (2) `0df4302` The break-glass witness's
  baseline is the glass broken, not the program's output; (3) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/guest.rs` (`GuestDevices::planned`; the report's `device.not-started`),
  `crates/delulu/src/run_cmd.rs` (`DevicePlan::device_names`), `crates/delulu/src/schema.rs` (a comment),
  `crates/delulu/tests/sandbox_devices_cli.rs` (a witness), `docs/survey/*`. (2) M `crates/delulu/tests/monitor_cli.rs`,
  `docs/survey/*`. (3) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 5: a
  witness racing the program is timing-shaped), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-117),
  `docs/REMAINING_WORK.md` (4.60 closed), `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Deleted:
  nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: the RW 4.60 witness red first; M266/M268 red, M267 survived on unreachable code and the code was removed; clippy and
  `check-other-os.sh` clean; read on the runners at `11b48cc` — Windows `38282229036`, macOS `38282233982`, arm64 `38282237136`
  (18 passed each). The full suite at `11b48cc`: 2,222 passed, **1 failed** (the break-glass witness's race, read and reproduced:
  1 of 5 with four CPUs busy); after `0df4302`, 0 of 8 under the same load, and the full suite alone: 2,223 passed, 0 failed, 16
  ignored, cargo exit 0, the tree unmoved. `3a91ab7`'s CI `38281286403` — success.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a witness whose subject races the program is timing-shaped, though it times nothing.
- Open / next, in order: (1) read THIS commit's push run (unread at this entry); (2) **P9-01** from `V2_P9_DESIGN.md` (step 1, spans
  in the walk; step 2, the verdict); (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49. **For the owner:** D-V2-115
  to D-V2-117, option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 19 (last): `e2ea6a5`'s push runs read green; the closing entry corrected
- Session: `https://claude.ai/code/session_01DeHytTS7xJQjv5K5ddrWP2`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through `claude/zealous-keller-kkuv5g`)   Pull request: none   Merged: n/a
- Base: `e2ea6a5`
- Commits: (1) this entry's commit — records only
- Files and folders: M `docs/CLOUD_SYNC_LOG.md` (the closing entry: P8-04 complete, RW 4.60 among the closed, a duplicate RW
  4.49 removed; this entry), `docs/DELULULANG_V2/V2_LOG.md` (the push runs), `docs/survey/*`. Deleted: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: `e2ea6a5`'s push runs — CI `38284697941` success on every job, `ocsf` `38284697894` success. Every code change of run 19
  (`96b7b87`, `0d3f514`, `11b48cc`, `0df4302`) is on `master` and its push run read green.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: as the entry before this one — (1) read THIS commit's push run; (2) **P9-01** from `V2_P9_DESIGN.md`; (3) RW 4.57,
  RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49. Keep the closing entry current.

### 2026-10-12 — routine run 20: CI read green; P9-01 — `delulu authority FILE --within BOUNDARY` (D-V2-118); SECRET-HIDDEN-1 closed
- Session: `https://claude.ai/code/session_01UbS6DfQbubWag83bcEk12u`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef this run)
- Branch: `master` (through the harness branch `claude/zealous-keller-an8lk9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `21421ec`
- Commits: (1) `3611ac5` P9-01: delulu authority FILE --within BOUNDARY; SECRET-HIDDEN-1 closed; (2) this entry's commit — the
  records
- Files and folders: (1) A `crates/delulu/src/within.rs` (the verdict, its renderings, seven unit tests), `crates/delulu/tests/within_cli.rs` (five witnesses); M `crates/delulu/src/cli.rs` (`ScopeWalk` keeps call sites; `requirements`/`Requirement`/`Ask`; SECRET-HIDDEN-1; `--within`/`--within-file`; `authority_within`; the help line), `crates/delulu/src/main.rs` (`mod within`), `crates/delulu/src/schema.rs` (`within`, `within_finding`), `crates/delulu/tests/cli.rs` (two SECRET-HIDDEN-1 cases), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.4 SECRET-HIDDEN-1; §11.5 four
  lessons), `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5: three lines), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md`
  (D-V2-118), `V2_P9_DESIGN.md` (P9-01 built, its departure), `V2_PHASE_STATUS.md` (P9 in progress), `docs/REMAINING_WORK.md`
  (4.29: P9-01 built), `docs/for-agents.md` (`--within` for a supervising agent), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry
  refreshed), `docs/survey/*`. Added: `crates/delulu/src/within.rs`, `crates/delulu/tests/within_cli.rs`. Deleted: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor: at the start `survey check` ok (1,502 nodes, 13,979 edges), `doctor --check` all checks passed; after the
  last edit — `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed (this
  commit's map rebuilt last; the pre-commit hook checked it against the index).
- Verified: `21421ec`'s push run CI `38286315257` — success. Run 19's two witnesses re-run in the VM — green. The SECRET-HIDDEN-1
  witness red first; `within_cli` (5) and `within.rs` (7) green; mutants M269–M280 red (M276 and M280 after their witnesses were
  strengthened; M272's code removed as unreachable), the control green; clippy clean. Read on the runners at `3611ac5` over
  `within_cli`, `cli`, `schema_cli`, `bin:delulu`: Windows `38305782405` (237 passed), macOS `38305785803` (250), arm64 `38305788572` (269) — 0 failed, `within.rs`'s seven unit tests and `within_cli` named on each, no compiler warning. The full suite alone at `3611ac5`: 2,235 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, and no `TREE-MOVED` line — `suite.sh` cannot see an edit undone before its end, which is why the slip below is recorded by hand. **The tree moved once
  under that suite** (a patch script's dry run that was not one — nine records written and restored byte for byte within about
  twenty seconds, during targets that read none of them); the document gates re-run after the records — green.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 (SECRET-HIDDEN-1) and §11.5 (measure a design's "decide with X" against the
  enforcement; a witness of an order uses an order neither the list nor its reverse gives; an empty set needs its own spelling;
  a patch script's dry-run switch is an edit with an anchor); `docs/assistant-memory/cloud-period-2026-09-28.md`.
- Open / next, in order: (1) read THIS commit's push run (unread at this entry, unless a later entry says so); (2) **P9-02**
  `grants diff` — read it against the code first (`V2_P9_DESIGN.md`), then build it on P9-01's verdict; (3) RW 4.57, RW 4.53 (with
  F4 and F6), RW 4.58's O2/O4, RW 4.49; (4) `--within` for a package directory and a `.dwx`. **For the owner:** D-V2-115 to
  D-V2-118, option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 20 (continued): P9-02's first slice — `delulu grants diff OLD NEW` between two grant files (D-V2-119)
- Session: `https://claude.ai/code/session_01UbS6DfQbubWag83bcEk12u`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-an8lk9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `52b2572`
- Commits: (1) `4f0fdd3` P9-02, first slice: delulu grants diff OLD NEW between two grant files; (2) this entry's commit — the
  records
- Files and folders: (1) M `crates/delulu/src/within.rs` (`DiffFinding`, `diff`), `crates/delulu/src/cli.rs` (`cmd_grants_diff`,
  `read_grant_lines`, the verb's arm and help line), `crates/delulu/tests/within_cli.rs` (a witness),
  `crates/delulu/tests/json_contract.rs` (the success table; the broker-verbs sweep), `docs/survey/*`. (2) M `CHANGELOG.md`,
  `HANDOFF.md` (where things stand), `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/for-agents.md`, `docs/REMAINING_WORK.md`
  (4.29), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-119), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`.
  Added, deleted, renamed: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `52b2572`'s push runs — CI `38306908871` success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38306907585` success; `actors_pingpong` — arm64 MEASURED 2.94x (control 4.12x, bar 1.50x), Windows and Ubuntu NOT MEASURED (busy or SMT), macOS NOT MEASURED (3 hardware threads). The witness red first (no verb); mutants M281–M289 red, the control green; clippy
  clean. Read on the runners at `4f0fdd3` over `within_cli` and `json_contract`: Windows `38308017895` (`json_contract` 13, `within_cli` 6), macOS `38308020954` (14, 6), arm64 `38308023699` (14, 6) — 0 failed, no compiler warning. The full suite alone at `4f0fdd3`:
  2,236 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved. The records were tried against a COPY of the files before they touched the tree (this run's lesson).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond D-V2-119 (the run's lessons are in the entry before this one).
- Open / next, in order: (1) read THIS commit's push run (unread at this entry, unless a later entry says so); (2) P9-02's lease
  nodes (`grants diff g_A g_B` — the broker's `Authority`, `budget_expansion` with it); (3) P9-03 proposals, reading this diff;
  (4) RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49; (5) `--within` for a package directory and a `.dwx`. **For the
  owner:** D-V2-115 to D-V2-119, option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 20 (closing): P9-02 between two lease nodes (D-V2-120) — P9-02 built; the run's close
- Session: `https://claude.ai/code/session_01UbS6DfQbubWag83bcEk12u`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef
  this run)
- Branch: `master` (through the harness branch `claude/zealous-keller-an8lk9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `02147ac`
- Commits: (1) `3f96546` P9-02: grants diff between two lease nodes, through their broker; (2) `94a7b21` The node-diff witness
  stops its broker before removing its state directory; (3) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/within.rs` (`diff_nodes`), `crates/delulu/src/cli.rs` (`grants_diff_nodes`; the
  node mode of `cmd_grants_diff`; the help line), `crates/delulu/tests/within_cli.rs` (a witness on a real daemon),
  `docs/survey/*`. (2) M `crates/delulu/tests/within_cli.rs` (the guard drops first), `docs/survey/*`. (3) M `CHANGELOG.md`,
  `HANDOFF.md` (where things stand; §11.5 two lessons; a `### 11.6` heading `52b2572` had put in twice), `docs/CLOUD_ROUTINE.md`
  (step 4's P9 line; step 5 a line),
  `docs/for-agents.md`, `docs/REMAINING_WORK.md` (4.29), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-120),
  `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md` (P9-02 built), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing. **Authorship:** the
  owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `02147ac`'s push runs — CI `38309152070` success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38309152114` success; `actors_pingpong` — arm64 MEASURED 2.73x (control 4.08x, bar 1.50x), macOS MEASURED 1.73x (control 3.61x, bar 1.35x — a runner that offered four threads this time), Windows and Ubuntu NOT MEASURED (busy or SMT). Mutants M290–M295 red, the control green; clippy clean. Read on the runners at
  `3f96546` over `within_cli` and `json_contract`: Windows `38309762033` (`json_contract` 13, `within_cli` 7), macOS `38309764907` (14, 7), arm64 `38309768345` (14, 7) — 0 failed, no compiler warning. The full suite alone at `3f96546`: 2,237 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved. **An orphan daemon,
  found in those green reads' last lines** (`Terminate orphan process … (delulu)` on macOS and arm64) and in the VM (three): the
  node witness removed its broker's state directory before its stop guard ran — fixed in `94a7b21`, read on the runners: `within_cli` at `94a7b21` — Windows `38310713607`, macOS `38310716813`, arm64 `38310720046`, 7 passed each and no orphan `delulu` in any cleanup.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a guard that reaches its subject through a directory drops first; a runner's
  orphan-process line is a leaked daemon.
- **The run in one paragraph:** CI green at the start (`21421ec`); **P9-01 built** — `delulu authority FILE --within BOUNDARY`,
  each request decided by the runtime's own mint, the counterexample's call named (D-V2-118), with **SECRET-HIDDEN-1** found and
  closed beside it (a computed secret name missing from `--grants`); **P9-02 built** — `delulu grants diff` between two grant
  files (D-V2-119) and two lease nodes (D-V2-120). Every code commit read on Windows, macOS and arm64 before `master` moved, three
  full suites alone (2,235, 2,236 and the last below — 0 failed), 27 mutants (M269–M295: 26 red, one on unreachable code that
  was removed), every push run read green but the last. One slip, recorded: a records patch's dry run wrote into the tree for
  about twenty seconds mid-suite (§11.5 has the lesson; the next two patches were tried on copies).
- Open / next, in order: (1) read THIS commit's push run (unread at this entry); (2) **P9-03** — proposals whose approval is
  bounded by `⊑` and reads `grants diff` (`V2_OPENSHELL_STUDY.md` §4.10) — read it against the code first, as `V2_P9_DESIGN.md`
  did for P9-01; (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49; (4) `--within` for a package directory and a
  `.dwx`; (5) the Sunday sweep (2026-10-18 if the period is extended). **For the owner:** D-V2-115 to D-V2-120, option (c).
  **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 20 (postscript): `1fc3aa0`'s push runs read; a gate for a heading written twice
- Session: `https://claude.ai/code/session_01UbS6DfQbubWag83bcEk12u`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-an8lk9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `1fc3aa0`
- Commits: (1) `b566969` A gate: no markdown file repeats a heading on the heading after it; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/tests/repository_structure.rs` (`repeated_headings`; two tests), `docs/survey/*`.
  (2) M `HANDOFF.md` (§11.5: the gate exists), `docs/CLOUD_ROUTINE.md` (step 5: a waiter fixes its SHA), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing
  entry's list of the loop's tools), `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `1fc3aa0`'s push runs — CI `38311372015` success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38311371940` success; `actors_pingpong` — arm64 MEASURED 2.90x (control 4.11x, bar 1.50x), macOS NOT MEASURED (3 hardware threads), Windows and Ubuntu NOT MEASURED (busy or SMT). The gate red with `52b2572`'s duplicate put back, green restored. Read on the runners
  at `b566969` over `repository_structure`: Windows `38311565631`, macOS `38311568364`, arm64 `38311571579` — 4 passed each, the two new tests named. The full suite alone at `b566969`: 2,239 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, and no broker daemon left running after it.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new.
- Open / next: as the closing entry before this one — (1) read THIS commit's push run; (2) **P9-03**, read against the code first;
  (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49; (4) `--within` for a package directory and a `.dwx`.
  **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 20 (last): P9-03 read against the code
- Session: `https://claude.ai/code/session_01UbS6DfQbubWag83bcEk12u`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master` (through the harness branch `claude/zealous-keller-an8lk9`)   Pull request: none   Merged: n/a
- Base: `ec026a0`
- Commits: (1) this entry's commit — records only
- Files and folders: M `docs/DELULULANG_V2/V2_P9_DESIGN.md` (P9-03's reading, measured), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_ROUTINE.md` (step 4's P9 line: P9-03's build step 1 first), `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/survey/*`.
  Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors, `doctor --check` all checks passed
- Verified: the measurements in the reading, by hand on the current binary (`run --json` and `run --report-out` on
  `examples/guide/05_capabilities.delulu` with `--grant console`; the diagnostic's span against `authority --within`'s `at`). The
  document gates and the Survey's tests after the records. `ec026a0`'s push runs: see the next entry, or unread if there is none.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next, in order: (1) read `ec026a0`'s and THIS commit's push runs; (2) **P9-03** from `V2_P9_DESIGN.md` — step 1, the
  report records a refusal with its span; (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49; (4) `--within` for a
  package directory and a `.dwx`. **For the owner:** D-V2-115 to D-V2-120, option (c). **Keep the closing entry current; from
  2026-10-16 a run only refreshes it and stops.**

### 2026-10-12 — routine run 20 (closing, last): P9-03 step 1 — a refused run's report records the refusal at its call (D-V2-121)
- Session: `https://claude.ai/code/session_01UbS6DfQbubWag83bcEk12u`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef
  this run)
- Branch: `master` (through the harness branch `claude/zealous-keller-an8lk9`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `9357ddc`
- Commits: (1) `c2dd5f0` P9-03 step 1: a refused run's report records the refusal at its call; (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/run_cmd.rs` (`RUN_FAULT`; `run_report` carries it), `crates/delulu/tests/within_cli.rs`
  (a witness), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand), `docs/CLOUD_ROUTINE.md` (step 4's P9 line),
  `docs/REMAINING_WORK.md` (4.29), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-121), `V2_P9_DESIGN.md`,
  `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing.
  **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `ec026a0`'s and `9357ddc`'s push runs — CI `38313420752` (`ec026a0`) and CI `38313748666` (`9357ddc`) — success on every job (no `openshell` run: neither push touched its paths). The witness red first; M296, M297 red, the control green; clippy
  clean. Read on the runners at `c2dd5f0` over `within_cli`, `json_contract`, `schema_cli`: Windows `38314043269`, macOS `38314046573`, arm64 `38314049378` — each job green over its three targets (`within_cli` 8 passed on each), no orphan process. The full suite alone at
  `c2dd5f0`: 2,240 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond D-V2-121.
- **The run, finally:** P9-01 built (D-V2-118) with SECRET-HIDDEN-1 closed beside it; P9-02 built (D-V2-119, D-V2-120); P9-03 read
  against the code and its step 1 built (D-V2-121); a gate for a heading written twice; an orphan daemon from this run's own witness
  found in a green read's tail and fixed. Ten commits of code and records, every code commit read on Windows, macOS and arm64 before
  `master` moved, five full suites alone (0 failed in each), 29 mutants (M269–M297: 28 red, one on unreachable code, removed).
- Open / next, in order: (1) read THIS commit's push run (unread at this entry); (2) **P9-03's next step** — a sandboxed run's
  report records its refusal too (`guest.rs` writes it), then `grants propose --from REPORT` (`V2_P9_DESIGN.md`); (3) RW 4.57,
  RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49; (4) `--within` for a package directory and a `.dwx`; (5) the Sunday sweep
  (2026-10-18 if the period is extended). **For the owner:** D-V2-115 to D-V2-121, option (c). **Keep the closing entry current;
  from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-13 — routine run 21: CI read green; P9-03 step 1b — a sandboxed run's report records the host's refusal at its call (D-V2-122)
- Session: `https://claude.ai/code/session_01DAyavJ4QKjZKUovSd9V255`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef
  this run so far)
- Branch: `master` (through the harness branch `claude/zealous-keller-uym82b`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `fc95094`
- Commits: (1) `ebf393c` P9-03: a sandboxed run's report records the host's refusal at its call; (2) this entry's commit — the
  records
- Files and folders: (1) M `crates/delulu-runtime/src/channel.rs` (`Refusal`; the host keeps its last refusal with its call; a
  unit test), `crates/delulu/src/guest.rs` (`ReportTo`; `host_refusal_diagnostics`; the evidence's refusal; a unit test),
  `crates/delulu/src/run_cmd.rs` (`flattened_name`), `crates/delulu/tests/within_cli.rs` (a witness), `docs/survey/*`. (2) M
  `CHANGELOG.md`, `HANDOFF.md` (where things stand), `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/REMAINING_WORK.md` (4.29),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-122), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md`
  (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing. **Authorship:** the owner's account through
  its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `fc95094`'s push run `38315802078` — success on every job; `actors_pingpong` arm64 MEASURED 2.97x (control 4.12x), the
  others NOT MEASURED. `ebf393c`: the witness red first; mutants M298–M306 red, the control green; clippy clean. Read on the runners
  over six targets: Windows `38327053568` (226 passed), macOS `38327056172` (241), arm64 `38327058865` (262) — 0 failed, both new tests
  named, no orphan `delulu`. The full suite alone at `ebf393c`: 2,243 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the
  tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond D-V2-122.
- Open / next, in order: (1) read THIS commit's push run; (2) **`grants propose REPORT`** — the walk keeps every minting call, a
  proposal is the refused call's static requirement (`V2_P9_DESIGN.md`), with a `schema_cli` witness that a refused run's report
  matches the `run-report` schema; (3) RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49; (4) `--within` for a package
  directory and a `.dwx`. **For the owner:** D-V2-115 to D-V2-122, option (c). **Keep the closing entry current; from 2026-10-16 a
  run only refreshes it and stops.**

### 2026-10-13 — routine run 21 (continued): P9-03 step 2 — `grants propose REPORT` (D-V2-123); `master` red on `187494a`, two witnesses fixed
- Session: `https://claude.ai/code/session_01DAyavJ4QKjZKUovSd9V255`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-uym82b`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `187494a`
- Commits: (1) `93a74b7` P9-03: grants propose REPORT - the refused call's own requirement; (2) `ea31897` The quarantine witness
  counts the third refusal in the trace; (3) `ba60f60`, `4c2ba86`, `0e92a3e` — the macOS footprint witness's three forms (green,
  green with readings, red), each read on a runner; (4) `fe621c6` The macOS peak-footprint witness reads a process of its own;
  (5) `e879816` MUTANT M317 (read red on macOS) and `995581e` its revert; (6) this entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/cli.rs` (`ScopeWalk.sites`, `MintSite`; `cmd_grants_propose`; the help line),
  `crates/delulu/src/within.rs` (`Proposal`, `propose`), `crates/delulu/tests/within_cli.rs` (two witnesses),
  `crates/delulu/tests/json_contract.rs` (the success row; the broker-verbs sweep), `crates/delulu/tests/schema_cli.rs` (a refused
  report validated), `docs/survey/*`. (2) M `crates/delulu/tests/monitor_cli.rs`, `docs/survey/*`. (3)–(5) M
  `crates/delulu/src/jail.rs` (its macOS tests only), `docs/survey/*`. (6) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5
  five lessons), `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5 a line; step 7a a gate), `docs/for-agents.md` (`grants propose`),
  `docs/REMAINING_WORK.md` (4.29; **4.62 added**), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-123),
  `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted,
  renamed: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `187494a`'s push runs — `ocsf` `38328124557` success, CI `38328124600` **failure** (Ubuntu's and macOS's test jobs; read
  and fixed — `V2_LOG.md`). `93a74b7`: the witnesses red first; mutants M307–M316 red, the control green; clippy clean; read on the
  runners over `within_cli`, `json_contract`, `schema_cli` and `delulu`'s unit tests — Windows `38328775607` (207 passed), macOS
  `38328777737` (222), arm64 `38328781069` (241); the full suite alone at `93a74b7`: 2,246 passed, 0 failed, 16 ignored (162
  binaries). The two fixes: `monitor_cli` red 3 of 3 with its window grown, green 3 of 3 after, 19 passed 3 of 3 under four busy
  CPUs, read on `ubuntu-24.04` (`38330138742`, 19 passed); the footprint witness red on macOS in its third form (`38330675493`), the
  fix read green (`38331001159`, 193 passed), M317 red (`38331248991`). The full suite alone at `995581e` with these records in the
  tree: 2,246 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a gate over the paths that succeed says nothing about the paths that fail; a list
  kept by key keeps one location per key; an offset someone else supplies is checked on a character boundary; a grown window that
  stays green refutes the guessed mechanism (print the readings); a count of a program's output is a baseline its watcher can
  pre-empt.
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-03 step 3** — the store and approval (`V2_P9_DESIGN.md` item 3;
  D-V2-123 item 6); (3) `--within` and `grants propose` for a package directory (both refuse one in words today) and a `.dwx`;
  (4) RW 4.62, RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49. **For the owner:** D-V2-115 to D-V2-123, option (c).
  **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-13 — routine run 21 (continued): `authority DIR --within` and `grants propose` for a package (D-V2-124)
- Session: `https://claude.ai/code/session_01DAyavJ4QKjZKUovSd9V255`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-uym82b`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `f3d4076`
- Commits: (1) `9473789` P9-01 and P9-03 for a package: --within and grants propose read a package directory; (2) this entry's
  commit — the records
- Files and folders: (1) M `crates/delulu/src/cli.rs` (`authority_within` takes the modules; `emit_authority` answers
  `--within`; `cmd_grants_propose` reads a package's flattened text), `crates/delulu/tests/within_cli.rs` (a witness; the corpus
  walks packages; the refusal table's `.dwx` row), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand),
  `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/for-agents.md`, `docs/REMAINING_WORK.md` (4.29),
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-124), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `f3d4076`'s push runs — CI `38332234517` success on every job (`miri-slow` and `heavy-gates` skipped on a push) — **`master` green again**, the two fixed witnesses green on Ubuntu and macOS — and `openshell` `38332234499` success; `actors_pingpong` arm64 MEASURED 2.95x (control 4.11x, bar 1.50x), the others NOT MEASURED. `9473789`: the witness red first; mutants M318–M321 red, the control green; clippy
  clean; read on the runners — Windows `38332620471` (`json_contract` 13, `within_cli` 12), macOS `38332623451` (14, 12), arm64 `38332625666` (14, 12) — 0 failed, the new test named, no compiler warning, no orphan `delulu`; the full suite alone — 2,247 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond D-V2-124.
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-03 step 3** — the store and approval (`V2_P9_DESIGN.md` item 3;
  D-V2-123 item 6); (3) a boundary over a `.dwx`; (4) RW 4.62, RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49.
  **For the owner:** D-V2-115 to D-V2-124, option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and
  stops.**

### 2026-10-13 — routine run 22: CI read green; P9-03 step 3 — the proposal store, approval under the approver's node (D-V2-125)
- Session: `https://claude.ai/code/session_01XEfaGaikkQiMYt3Z72178T`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-ekmk5a`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `ae84188`
- Commits: (1) `45877b7` P9-03 step 3: the proposal store, approval under the approver's node, rejection; (2) this entry's
  commit — the records
- Files and folders: (1) **A `crates/delulu-broker/src/proposal.rs`** (the store, `join`, approval and rejection; seven unit
  tests); M `crates/delulu-broker/src/diag.rs` (`Denial::Proposal`, DL0904), `lib.rs` (the module), `secrets.rs` (`live_authority`
  shared), `tree.rs` (the store's field), `crates/delulu/src/broker_ipc.rs` (four requests, two answers, `ProposalWire`),
  `crates/delulu/src/brokerd.rs` (the handlers and their lost-record rule; a test), `crates/delulu/src/cli.rs` (`propose --node`,
  `proposals`, `approve`, `reject`; the help), `crates/delulu/tests/within_cli.rs` (the witness), `crates/delulu/tests/json_contract.rs`
  (the sweep), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md`
  (step 4's P9 line; step 5 a line on `cli.rs`'s help text — loop engineering), `docs/for-agents.md`, `docs/REMAINING_WORK.md` (4.29), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-125), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md`
  (this entry; the closing entry), `docs/survey/*`. Added: `proposal.rs`. Deleted, renamed: nothing. **Authorship:** the owner's
  account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `ae84188`'s push runs — CI `38333635019` success on every job, `openshell` `38333634973` success; `actors_pingpong`
  arm64 MEASURED 2.88x (control 4.11x), the others NOT MEASURED. `45877b7`: the witness red first; mutants M322–M336 red (M331 after
  its witness was made to look before the dedup), the control green; clippy clean; read on the runners — Windows `38352400253` (205 passed), macOS `38352403113` (221), arm64 `38352405496` (239), and the broker's unit tests on Windows `38352408072` (`proposal`, 7) — 0 failed, every new test named, no compiler warning, no orphan `delulu`; the full
  suite alone — 2,256 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a witness of a withdrawal looks before anything that deduplicates can answer with
  the leftover (M331); `docs/assistant-memory/cloud-period-2026-09-28.md` a line.
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-03 step 4** — auto-approval within the operator's standing
  boundary, asked of what an approval would MINT (`V2_P9_DESIGN.md` item 4); (3) P9-04 (secrets bound to endpoints); (4) a boundary
  over a `.dwx`; (5) RW 4.62, RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49. **For the owner:** D-V2-115 to D-V2-125,
  option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-13 — routine run 22 (continued): P9-03 step 4 — `grants auto-approve`, the operator's standing node (D-V2-126); P9-03 complete
- Session: `https://claude.ai/code/session_01XEfaGaikkQiMYt3Z72178T`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-ekmk5a`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `11c9d94`
- Commits: (1) `a7c409c` P9-03 step 4: grants auto-approve - the operator's standing node approves what it bounds; (2) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/cli.rs` (`cmd_grants_auto_approve`; the help; the advertised verbs),
  `crates/delulu/tests/within_cli.rs` (the witness), `crates/delulu/tests/json_contract.rs` (the sweep), `docs/survey/*`. (2) M
  `CHANGELOG.md`, `HANDOFF.md` (where things stand), `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/for-agents.md`,
  `docs/REMAINING_WORK.md` (4.29), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-126), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing
  entry), `docs/survey/*`. Added, deleted, renamed: nothing. **Authorship:** the owner's account through its GitHub no-reply
  address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `11c9d94`'s push runs — CI `38353462220` **failure** — `test (ubuntu-24.04)` only: `monitor_cli`'s two monitors revoked by their own dead-men (DEADMAN-BUSY-1, found and fixed below); every other job green, `actors_pingpong` arm64 MEASURED 2.99x (control 4.12x), the others NOT MEASURED; `ocsf` `38353462196` and `openshell` `38353462310` success; step 3's audit records exported as OCSF and validated against 1.8.0 (0 problems,
  self-test 11 of 11). `a7c409c`: the witness red first; mutants M337–M341 red, the control green; clippy
  clean; read on the runners — Windows `38353918721` (27 passed: `json_contract` 13, `within_cli` 14), macOS `38353921081` (28), arm64 `38353923894` (28) — 0 failed, the new test named, no compiler warning, no orphan `delulu`; the full suite alone — 2,257 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond D-V2-126 (the memory snapshot's run-22 line).
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-04** — secrets bound to endpoints (RW 2.11; a request with
  headers, a `Secret[Str]` audience) — read against the code first, as P9-01 and P9-03 were; (3) a boundary over a `.dwx`;
  (4) RW 4.62, RW 4.57, RW 4.53 (with F4 and F6), RW 4.58's O2/O4, RW 4.49. **For the owner:** D-V2-115 to D-V2-126, option (c).
  **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-13 — routine run 22 (continued): `master` red on `11c9d94` — DEADMAN-BUSY-1 found and fixed (D-V2-127)
- Session: `https://claude.ai/code/session_01XEfaGaikkQiMYt3Z72178T`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-ekmk5a`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `a7c409c`
- Commits: (1) `e074330` DEADMAN-BUSY-1: a dead-man counts its holder's silence, never the daemon's; (2) this entry's commit —
  the records of P9-03 step 4 and of DEADMAN-BUSY-1
- Files and folders: (1) M `crates/delulu-broker/src/deadman.rs` (`beat_deadman_arrived`), `crates/delulu/src/brokerd.rs` (each
  request stamped on arrival; dead-men fired as of it; beats and disarms judged by it; a witness), `crates/delulu/src/monitor_cmd.rs`
  (the beater thread), `crates/delulu/tests/monitor_cli.rs` (a witness; `start_monitor_with`), `docs/survey/*`. (2) M `CHANGELOG.md`,
  `HANDOFF.md` (§11.4 DEADMAN-BUSY-1; §11.5 a lesson), `docs/REMAINING_WORK.md` (4.53; **4.63 added**),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-127),
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*` — beside step 4's records above. Added, deleted,
  renamed: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `11c9d94`'s push run red (`38353462220`: Ubuntu's `monitor_cli`, two monitors revoked by their own dead-men; every
  other job green) — read, its class witnessed red twice, fixed; mutants M342–M345 red, M346 survived (RW 4.63), the control
  green; read on the runners — Ubuntu `38357155428` (233 passed), Windows `38357158233` (198), macOS `38357161227` (214), arm64 `38357164145` (233) — 0 failed, both new witnesses named on each, no compiler warning, no orphan `delulu`; the full suite alone — at `e074330`, 2,258 passed and 1 failed — the runtime's thread-stack gate, red for a thread no list named, the beater (`ae9dffe` lists it; `delulu-runtime`'s library then 262 passed); at `ae9dffe`, with these records in the tree, 2,259 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 DEADMAN-BUSY-1; §11.5 — a CI red in code the run did not touch is first a witness
  of its class: read what the failing run's records say did NOT happen in the window.
- Open / next, in order: (1) read THIS commit's push run — `master` was red on `11c9d94`, and this is its fix; (2) **P9-04** —
  secrets bound to endpoints, read against the code first; (3) RW 4.63 (a seam to witness a wedged watch), RW 4.53's rest (the
  daemon's waits on the chain), RW 4.62, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a boundary over a `.dwx`. **For the owner:**
  D-V2-115 to D-V2-127, option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-13 — routine run 22 (closing): `76f83a2` red on Ubuntu — DEADMAN-BUSY-1's arming half fixed; P9-04 read against the code
- Session: `https://claude.ai/code/session_01XEfaGaikkQiMYt3Z72178T`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-ekmk5a`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `76f83a2`
- Commits: (1) `eb7b8f8` DEADMAN-BUSY-1, the arming's half: the first period starts at its answer; (2) this entry's commit — the
  records and P9-04's reading
- Files and folders: (1) M `crates/delulu-broker/src/deadman.rs` (`start_deadman_period`), `crates/delulu/src/brokerd.rs` (the
  arming's answer starts the period; the idle test measures from the records), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md`
  (§11.4 DEADMAN-BUSY-1; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-127 item 6), `V2_P9_DESIGN.md` (P9-04 read against the code),
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing. **Authorship:** the
  owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `76f83a2`'s push runs — CI `38359276890` **failure** — `test (ubuntu-24.04)` only: `brokerd`'s `an_idle_daemon_fires_an_unbeaten_dead_man_without_any_request` (DEADMAN-BUSY-1's arming half); every other job green, Windows' and macOS' test jobs among them; `openshell` `38359276774` success. `eb7b8f8`: the records' measurement red first; mutants M347 and M348 red, the control
  green; read on the runners — Ubuntu `38361582689` (233 passed), Windows `38361586297` (198), macOS `38361590312` (214), arm64 `38361593590` (233) — 0 failed, the idle test named on each, no compiler warning, no orphan `delulu`; the full suite alone — 2,259 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — time a daemon by its own records, never by when its answer reached you.
- Open / next, in order: (1) **read THIS commit's push run** — `master` was red on `11c9d94` and `76f83a2` (DEADMAN-BUSY-1's two
  halves), and this is meant to end it; if Ubuntu's test job is red again, that is the next run's only task; (2) **P9-04 step (a)**
  — a request with headers (`V2_P9_DESIGN.md`); (3) RW 4.63, RW 4.53's rest, RW 4.62, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a
  boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-127, option (c). **Keep the closing entry current; from 2026-10-16 a
  run only refreshes it and stops.**

### 2026-10-13 — routine run 22 (last): `44f5b7f`'s push run read green — `master` green again
- Session: `https://claude.ai/code/session_01XEfaGaikkQiMYt3Z72178T`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (and the harness branch `claude/zealous-keller-ekmk5a`, the same commit)   Pull request: none   Merged: n/a
- Base: `44f5b7f`
- Commits: this entry's commit — the record of the read
- Files and folders: M `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/survey/*`. Added, deleted,
  renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `44f5b7f`'s CI `38362909613` — success on every job, Ubuntu's test job included (`miri-slow` and `heavy-gates`
  skipped on a push). **`master` green again** after `11c9d94` and `76f83a2` (DEADMAN-BUSY-1, D-V2-127).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new.
- Open / next, in order: (1) read THIS commit's push run (records only); (2) **P9-04 step (a)** — a request with headers
  (`V2_P9_DESIGN.md`); (3) RW 4.63, RW 4.53's rest, RW 4.62, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a boundary over a `.dwx`.
  **For the owner:** D-V2-115 to D-V2-127, option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it
  and stops.**

### 2026-10-14 — routine run 23: CI read green; P9-04 step (a) — `http.get_with`, a request with headers (D-V2-128)
- Session: `https://claude.ai/code/session_01X1KXZB4JUGcKCDjQqrCkgQ`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-r6to2u`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `b7a5c3b`
- Commits: (1) `bee695f` P9-04 step (a): http.get_with - a request with headers, judged before anything is resolved; (2) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu-check/src/check.rs` (the `get_with` arm), `dir.rs` (`PRIM_TABLE_VERSION` 6),
  `prim_table.rs` (the row; 83), `crates/delulu-runtime/src/egress.rs` (`check_headers`, `RESERVED_HEADERS`, `request_with`, the
  origin rule, the hop's header names, the transport), `egress/tests.rs` (four witnesses), `channel.rs` (`WireValue::Map`; three
  witnesses), `interp.rs` (custody and trace), `prim.rs` (the arm), `trace.rs` (the effect; its test), `crates/delulu/src/run_cmd.rs`
  (the note's wording), `schema.rs` (the hop's `headers`), `crates/delulu/tests/egress_cli.rs` and `monitor_cli.rs` (two witnesses
  each), `docs/GETTING_STARTED.md`, `docs/design/STAGE1_SPECIFICATION.md` (the row), `docs/for-agents.md`, `docs/reference/README.md`,
  `coverage.md`, `primitives.md` (generated), **`tests/conformance/witnesses.toml` (entrenched — four rows)**, `docs/survey/*`. (2) M
  `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5 a line —
  loop engineering), `docs/REMAINING_WORK.md` (2.11), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/design/ENTRENCHED_CHANGE_RECORD.md` (the entry), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-128),
  `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added,
  deleted, renamed: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `b7a5c3b`'s push run `38365143257` and the nightlies `38380513605` and `38534326546` — success on every job;
  `actors_pingpong` arm64 MEASURED 2.93x / 2.91x, the others NOT MEASURED. `bee695f`: the absence witnessed first (DL0405; a `Map`
  refused by the channel); mutants M349–M367 red (M351 after its witness spelled the pair both ways round), the control green;
  clippy clean; macOS, Windows, arm64 and musl linted from Linux; read on the runners — `delulu-runtime`'s library on Windows `38581940557` (268 passed), macOS `38581945945` (269), arm64 `38581950457` (269), and `egress_cli`, `monitor_cli`, `schema_cli` on Windows `38581958301` (31), macOS `38581962693` (32), arm64 `38581967281` (33) — 0 failed, every new test named, no compiler warning, no orphan `delulu`; the full suite alone —
  2,270 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL (a `.dpx` built on the laptop against table version 5 is
  refused DL1503 and rebuilt — none is committed).
- For the laptop's memory: `HANDOFF.md` §11.5 — a witness of a normalizing comparison spells the colliding pair both ways round
  (M351); `docs/assistant-memory/cloud-period-2026-09-28.md` a line.
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-04 steps (b) and (c), as ONE slice** — a `Secret[Str]` in a
  header position only with its audience in the grant (alone, (b) would send a secret without `Declassify`); (3) RW 4.63, RW
  4.53's rest, RW 4.62, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-128
  (D-V2-128 with the period's second entrenched edit), option (c). **Keep the closing entry current; from 2026-10-16 a run only
  refreshes it and stops.**

### 2026-10-14 — routine run 23 (continued): `fa2d34e` red on supply-chain — rustls 0.23.46; RW 4.62 closed (D-V2-129); `5fa3261` red on Ubuntu — the verify timing witness bounded against a control (D-V2-130)
- Session: `https://claude.ai/code/session_01X1KXZB4JUGcKCDjQqrCkgQ`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-r6to2u`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `fa2d34e`
- Commits: (1) `5fa3261` rustls 0.23.45 -> 0.23.46: RUSTSEC-2026-0346 to -0349, read red on fa2d34e's push run; (2) `7c653e0`
  RW 4.62: a refused operation is followed by its refusal in the trace; (3) `76f54a0` The DIR verify timing witness: bounded
  against a control on the same machine; (4) this entry's commit — the records
- Files and folders: (1) M `Cargo.lock` (rustls; five crates' `windows-sys` 0.52.0 → 0.61.2), `deny.toml` (why), `docs/survey/*`.
  (2) M `crates/delulu-runtime/src/interp.rs` (`Dispatched`, `trace_refusal`), `crates/delulu-runtime/src/trace.rs` (the module's
  rule), `crates/delulu-wasm/src/host.rs` (`push_refusal_trace`), `crates/delulu/tests/guard_e2e.rs` and `cli.rs` (the witnesses),
  `docs/for-agents.md`, `docs/survey/*`. (3) M `crates/delulu-check/src/dir.rs` (the witness), `docs/survey/*`. (4) M `CHANGELOG.md`, `HANDOFF.md` (§11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 2 a
  line — loop engineering), `docs/REMAINING_WORK.md` (4.62 closed; **4.64 added**), `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-129, D-V2-130), `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added,
  deleted, renamed: nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `fa2d34e`'s push runs — CI `38584248231` **failure**, `supply-chain` only (RUSTSEC-2026-0346 to -0349 against
  rustls 0.23.45, published that day), every other job green; `openshell` `38584248260` success. `5fa3261`: `cargo deny --all-features
  check advisories` FAILED with the four, then ok (bans, licences, sources ok); Windows and macOS linted; its push run
  ``38586917971``: **failure** — `test (ubuntu-24.04)` only (the DIR verify timing witness — the next entry); `supply-chain` green with rustls 0.23.46, every other job green. `7c653e0`: RW 4.62 witnessed red first (a guarded write traced as dispatched, nothing after,
  nothing written); mutants M368–M373 red, the control green; clippy clean; the five other targets linted; read on the runners —
  `guard_e2e` and `cli` on Windows `38587247926` (56 passed), macOS `38587253051` (55), arm64 `38587258506` (55) — 0 failed, both new witnesses named; the full suite alone — 2,272 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left. `76f54a0`: `5fa3261`'s CI `38586917971` read red on Ubuntu (the verify
  witness, 5.7 ms against 5 ms; every other job green, `supply-chain` included); reproduced by starving the VM (red 3 of 3); fixed —
  green 5 of 5 starved, M374 red; read on the runners — `delulu-check`'s library — Ubuntu `38590951265` (248 passed; ratio 1.90, 1.70 ms a verify), Windows `38590955781` (248; 2.02, 2.19 ms), macOS `38590961151` (248; 1.91, 1.16 ms), arm64 `38590966445` (248; 1.98, 1.77 ms).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL (the lockfile moved `windows-sys` for five crates — the
  Windows suite is the first local build of them).
- For the laptop's memory: `HANDOFF.md` §11.5 — `cargo update -p ONE` can move crates you did not name: read the lockfile's diff; and a
  speed claim in absolute time is a claim about the machine — bound it against a control timed in the same window (D-V2-130).
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-04 steps (b) and (c), as ONE slice** — a `Secret[Str]` in a
  header position only with its audience in the grant; (3) RW 4.64 (the WASM engine's refusal records — next, if this run's budget does not reach it), RW 4.63,
  RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-130, option
  (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-14 — routine run 23 (continued): `6971d9c`'s push run read green — `master` green again; RW 4.64 closed — both engines trace a refusal the same
- Session: `https://claude.ai/code/session_01X1KXZB4JUGcKCDjQqrCkgQ`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-keller-r6to2u`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `6971d9c`
- Commits: (1) `b3e17fc` RW 4.64: the WASM engine traces a refused operation as the interpreter does; (2) this entry's commit —
  the records
- Files and folders: (1) M `crates/delulu-wasm/src/host.rs`; **A `crates/delulu-wasm/tests/refusal_trace_parity.rs`** (the
  two-engine witness); M `docs/survey/*`. (2) M `scripts/suite.sh` (a free-disk floor and a stale-map refusal — loop engineering), `docs/CLOUD_ROUTINE.md`
  (step 5 a line), `CHANGELOG.md`, `HANDOFF.md` (where things stand), `docs/REMAINING_WORK.md` (4.64
  closed), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-129's
  residual), `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added: the test file. Deleted, renamed:
  nothing. **Authorship:** the owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `6971d9c`'s CI `38591853430` — success on every job: **`master` green again**. `b3e17fc`: the new two-engine witness
  red first against the old host, green with the fix; mutants M375–M379 red, the control green; the WASM crate's tests, `cli`,
  `guard_e2e`, `json_contract`, `core_invariance` green; the two-engine differential — 50,000 programs, `two_engine_differential_gate` ok in 145 s. clippy clean; the five other
  targets linted; read on the runners — `delulu-wasm`'s library and `refusal_trace_parity` on Windows `38595321024` (86 + 2 passed), macOS `38595325757` (91 + 2), arm64 `38595333185` (91 + 2) — 0 failed, both witnesses named, no compiler warning, no orphan `delulu`; the full suite alone — 2,270 passed and 4 failed, 16 ignored (163 binaries) — the four were `doctor_cli` (three) and the Survey's `freshness`, a stale map: `scripts/suite.sh` was edited (this run's loop engineering) after the map was built and before the suite started, which `TREE-MOVED` cannot see; the map rebuilt, both targets green (11 and 5 passed). `suite.sh` now refuses to start on a stale map, witnessed refusing.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond the run-23 lines (the disk filled once — run 6's trap, `HANDOFF.md` §11.5).
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-04 steps (b) and (c), as ONE slice** — a `Secret[Str]` in a
  header position only with its audience in the grant; (3) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a
  boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-130, option (c). **The closing entry is current; from 2026-10-16 a
  run only refreshes it and stops.**

### 2026-10-14 — routine run 23 (last): `87c0e2f`'s push run read green; the closing entry's size brought up to date
- Session: `https://claude.ai/code/session_01X1KXZB4JUGcKCDjQqrCkgQ`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (and the harness branch `claude/zealous-keller-r6to2u`, the same commit)   Pull request: none   Merged: n/a
- Base: `87c0e2f`
- Commits: this entry's commit — the record of the read
- Files and folders: M `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry's size), `docs/survey/*`.
  Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check`
  all checks passed
- Verified: `87c0e2f`'s CI `38598988825` — success on every job (`miri-slow` and `heavy-gates` skipped on a push): `master` green at the run's end, with P9-04 step (a), RW 4.62, RW 4.64, D-V2-130 and rustls 0.23.46 on it. `actors_pingpong`: arm64 MEASURED 2.93x (control 4.12x); Ubuntu and Windows NOT MEASURED (never 3x perfectly parallel work); macOS NOT MEASURED (3 hardware threads).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new.
- Open / next, in order: (1) read THIS commit's push run (records only); (2) **P9-04 steps (b) and (c), as ONE slice** — a
  `Secret[Str]` in a header position only with its audience in the grant (`V2_P9_DESIGN.md`); (3) RW 4.63, RW 4.53's rest, RW
  4.57, RW 4.58's O2/O4, RW 4.49; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-130 (D-V2-128 with the
  period's second entrenched edit), option (c). **The closing entry is current; from 2026-10-16 a run only refreshes it and stops.**
### 2026-10-14 — routine run 24: CI read green; P9-04 steps (b) and (c) — a secret in a request, only to its grant's audience (D-V2-131)
- Session: `https://claude.ai/code/session_01Lxo11erVVd6HiYqFKAznFG`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-carson-tidpo3`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `48ef512`
- Commits: (1) `91a1eb7` P9-04 steps (b) and (c): http.get_with_secret - a secret only to the audience its grant names; (2) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu-check/src/check.rs` (the `get_with_secret` arm), `dir.rs` (`PRIM_TABLE_VERSION` 7),
  `lib.rs` (the checker's witness), `prim_table.rs` (the row; 84), `crates/delulu-runtime/src/egress.rs` (`Audience`,
  `parse_audience`, `Reason::Audience`, `SecretHeader`, `request_with_secret`, the hop's `secret`), `egress/tests.rs` (four
  witnesses), `broker.rs` (the grant line's audience; its witness), `value.rs` (`SecretAudience`, `SecretVal::bound`,
  `RootVal.secret_audiences`), `prim.rs` (the mint carries the audience; the arm; a broker handle refused), `interp.rs` (`map`
  carries the audience; no custody op for a refused handle; trace detail), `actors.rs` (the audience crosses; its witness),
  `trace.rs`, `crates/delulu/src/cli.rs` (a lease's grants carry none), `repl.rs`, `schema.rs` (the hop's `secret`), `within.rs`
  (`secret_reach_expansion`), `crates/delulu/tests/egress_cli.rs`, `broker_cli.rs`, `within_cli.rs` (one witness each),
  `scripts/mutants.py` (multi-edit mutants — loop engineering), `docs/GETTING_STARTED.md`, `docs/for-agents.md`,
  `docs/design/STAGE1_SPECIFICATION.md` (the row), `docs/reference/README.md`, `coverage.md`, `primitives.md` (generated),
  **`tests/conformance/witnesses.toml` (entrenched — four rows)**, `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where
  things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5 a line — loop engineering),
  `docs/REMAINING_WORK.md` (2.11), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/design/ENTRENCHED_CHANGE_RECORD.md`
  (the entry), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-131), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing. **Authorship:** the
  owner's account through its GitHub no-reply address, Claude as co-author.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `48ef512`'s push run `38601952201` — success on every job; `actors_pingpong` arm64 MEASURED 2.96x, the others NOT
  MEASURED. `91a1eb7`: the absence witnessed first (DL0405; the audience read as part of the name, DL0703); mutants M380–M400 (`scripts/mutants.py`) red but one: the origin judged against the audience before resolution removed (M380), the empty audience let through (M381), the per-hop origin rule ignored for the secret (M382), both walls removed at once (M383b), the hop's `secret` name dropped (M384), `audience` answered as something other than `Refused` (M385), a path let into an audience (M386), the secret's header unchecked (M387), the grant line split at its last `@` (M388), the audience not stored (M389), `Secret.map` dropping it (M390), the mint dropping it (M391), a broker handle sent (M392), a custody op asked for one (M393), the actor boundary dropping a secret's (M394) or a root's (M395), `grants diff` ignoring it (M396) or calling it `scope_expansion` (M397), the checker taking a `Str` for the secret (M398), the schema's hop `secret` mistyped (M399), `PinnedRequest`'s `Debug` printing values (M400) — and **M383, the per-hop audience check alone, survives on the first wall by design** (D-V2-131 item 3); the control green; clippy
  clean; macOS (arm64 and x86-64), Windows, arm64 Linux and musl linted from Linux after the last code edit (`scripts/check-other-os.sh`, clippy `-D warnings`, exit 0); read on the runners — `egress_cli`, `broker_cli`, `within_cli` on Windows `38626954263` (25 passed), macOS `38626958695` (26), arm64 `38626962999` (26); `delulu-runtime`'s library on Windows `38626966826` (274 passed, 1 ignored), macOS `38626971957` (275, 1), arm64 `38626976826` (275, 1); `delulu-check`'s `a_secret_rides…` on Windows `38626992133`, macOS `38626999206`, arm64 `38627003325` (1 each) — 0 failed, every new test named, no compiler warning, no orphan `delulu`; the full suite alone — 2,283 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL (a `.dpx` built on the laptop against table version 6 is
  refused DL1503 and rebuilt — none is committed).
- For the laptop's memory: `HANDOFF.md` §11.5 — a check built as two walls is falsified as two (M383/M383b; `scripts/mutants.py`
  takes `edits`); `docs/assistant-memory/cloud-period-2026-09-28.md` a line.
- Open / next, in order: (1) read THIS commit's push run; (2) **P9-04 step (d)** — the broker's audience dimension: a lease
  node's secret carries its audience (`⊑` per secret by inclusion of origins), the host asks the broker for the bytes at send
  time naming the destination, `grants diff` between nodes; then (e) `authority`'s per-secret destinations; (3) RW 4.63, RW
  4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-131
  (D-V2-128 and D-V2-131 with the period's second and third entrenched edits; D-V2-131 changes what a secret NAME may hold),
  option (c). **Keep the closing entry current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-14 — routine run 24 (continued): MAP-HANDLE-1 — `Secret.map` of a broker-held secret is refused, never run over bytes it does not hold (D-V2-132)
- Session: `https://claude.ai/code/session_01Lxo11erVVd6HiYqFKAznFG`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-carson-tidpo3`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `d61dd70`
- Commits: (1) `f4c428c` MAP-HANDLE-1: Secret.map of a broker-held secret is refused, never run over bytes it does not hold;
  (2) this entry's commit — the records
- Files and folders: (1) M `crates/delulu-runtime/src/interp.rs` (the refusal), `crates/delulu/tests/broker_cli.rs` (the
  witness), `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (§11.4 the finding), `docs/REMAINING_WORK.md` (4.49's third
  item marked closed by D-V2-111), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-132), `V2_P9_DESIGN.md` (the `map` claim corrected), `docs/CLOUD_SYNC_LOG.md` (this entry; the
  closing entry), `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: the defect witnessed on `48ef512`'s binary (`!` for `hunter2!`); M401 red, the control green; clippy clean; Windows
  and macOS linted; read on the runners — `broker_cli` on Windows `38629378991` (5 passed), macOS `38629384172` (6), arm64 `38629387987` (6) — 0 failed, both broker-held-secret witnesses named, no compiler warning, no orphan `delulu`; the full suite alone — 2,284 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left. Push runs: `91a1eb7`'s CI `38628412194` and `d61dd70`'s CI `38628780402` — success on every job (`miri-slow` and `heavy-gates` skipped on a push); `openshell` `38628412189` success. `actors_pingpong` at `d61dd70`: arm64 MEASURED 3.00x (control 4.11x); macOS NOT MEASURED (3 hardware threads); Ubuntu and Windows NOT MEASURED (never 3x perfectly parallel work).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 — MAP-HANDLE-1 and its search key (every reader of a stand-in value asks where
  the bytes are); `docs/assistant-memory/cloud-period-2026-09-28.md` a line.
- Open / next, in order: (1) read THIS commit's push run and `f4c428c`'s; (2) **P9-04 step (d)** — the broker's audience
  dimension (a lease node's secret carries its audience, `⊑` per secret by inclusion of origins; the host asks the broker for
  the bytes at send time naming the destination; `grants diff` between nodes) — and with it whether the language should reach
  the broker's whitelisted `secret_map`; then (e); (3) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49's first two
  items; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-132, option (c). **The closing entry is current;
  from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-14 — routine run 24 (last): `4f3084b` red on supply-chain — wasmtime 48.0.6 (RUSTSEC-2026-0353 to -0364); the claims P9-04 (b)/(c) overtook corrected; P9-04 (d) read against the code
- Session: `https://claude.ai/code/session_01Lxo11erVVd6HiYqFKAznFG`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (and the harness branch `claude/zealous-carson-tidpo3`, the same commits)   Pull request: none   Merged: n/a
- Base: `4f3084b`
- Commits: (1) `52193ba` The claims get_with_secret overtook: reveal() has three readers, the book a third way out; (2) `bf51fe0`
  wasmtime 48.0.5 -> 48.0.6: RUSTSEC-2026-0353 to -0359 and -0361 to -0364, read red on 4f3084b's push run; (3) this entry's
  commit — the records
- Files and folders: (1) M `crates/delulu-runtime/src/value.rs`, `prim.rs` (comments), `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/survey/*`. (2) M `Cargo.lock` (the Wasmtime release: 29 lockfile entries), `deny.toml` (why), `docs/survey/*`. (3) M
  `docs/DELULULANG_V2/V2_P9_DESIGN.md` (step (d) read against the code), `docs/CLOUD_ROUTINE.md` (step 5
  two lines — loop engineering), `HANDOFF.md` (§11.5 the `pkill` trap), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`
  (this entry; the closing entry's "as of"), `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `f4c428c`'s CI `38630746024` — success on every job (`miri-slow` and `heavy-gates` skipped on a push); `actors_pingpong` arm64 MEASURED 2.97x (control 4.12x), the others NOT MEASURED. `4f3084b`'s CI `38631973104` **failure** — `supply-chain` only (eleven Wasmtime advisories published
  that evening), every other job green. `bf51fe0`: `cargo deny` failed with the eleven, then ok (advisories, bans, licences,
  sources); the lockfile's diff read (only the Wasmtime release: 29 lockfile entries); `delulu-wasm`'s tests green; the two-engine
  differential ok in 116 s; the five other targets linted; **`bf51fe0`'s push run** (CI `38636976623`): success on every job, `supply-chain` included — **`master` green again**; `actors_pingpong` arm64 MEASURED 2.98x (control 4.13x), macOS NOT MEASURED (3 hardware threads), Ubuntu and Windows NOT MEASURED. **The full suite alone at `bf51fe0`:** 2,284 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left. `52193ba`: comments and prose only — the runtime library (275 passed, 1 ignored) and the book gate (9)
  green; clippy clean.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL (the lockfile moved the Wasmtime release — the laptop's
  first build of it).
- For the laptop's memory: `HANDOFF.md` §11.5 — `pkill -f PATTERN` matches the command that runs it; stop a job by PID.
- Open / next, in order: (1) read THIS commit's push run (records and comments only); (2) **P9-04 step (d)** — its build order
  is in `V2_P9_DESIGN.md` (a `secret_to` dimension of `NAME@host:port` pairs, a recorded `SendSecret` broker op guarded like
  `Declassify`, the witnesses to write first); then (e); (3) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49's first
  two items; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-132 (D-V2-128 and D-V2-131 with the period's
  second and third entrenched edits; D-V2-131 changes what a secret NAME may hold), option (c). **The closing entry is current;
  from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-15 — routine run 25: CI read green; P9-04 step (d)'s first half — the broker's `secret_to`, a node's secret carries its audience (D-V2-133)
- Session: `https://claude.ai/code/session_011TZbDxBVJVmBTVyDeiWhGb`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-carson-qm5vah`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `41ce7c1`
- Commits: (1) `db88e98` P9-04 (d), first half: the broker's secret_to - a node's secret carries its audience; (2) this entry's
  commit — the records
- Files and folders: (1) M `.github/workflows/ci.yml` (the model's step names; three more mutants),
  `crates/delulu-broker/src/authority.rs` (`Scopes.secret_to`, `secret_to_name`, the well-formedness rule, unit tests),
  `cert.rs` (the parser; the exhaustive round trip), `lib.rs` (the export), `proposal.rs` (the join; its test),
  `crates/delulu-runtime/src/plugin.rs` (a plugin grant names no audience), `crates/delulu/src/broker_client.rs` (one conversion),
  `broker_ipc.rs` (`AuthoritySpec`/`NodeInfo.secret_to`), `brokerd.rs` (the conversions), `cli.rs` (`--secret-to`, the help,
  `authority_spec_from_grants`, `grants_from_lease`, `secret_audiences_of`, `grants_diff_nodes`, unit test), `within.rs`
  (`diff_nodes`), `crates/delulu/tests/broker_cli.rs`, `within_cli.rs` (the witnesses), `docs/GETTING_STARTED.md`,
  `docs/MATHEMATICS.md`, `docs/QUESTIONS.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/design/AUTHORITY_GUARD_CAPSTONE.md`,
  `docs/design/models/README.md`, `docs/design/models/authority_algebra.py` (§2c, the ninth set dimension, three mutants),
  `docs/for-agents.md`, `docs/survey/*`. (2) M `CHANGELOG.md`, `HANDOFF.md` (where things stand; §11.5 a lesson),
  `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5 a line — loop engineering), `docs/REMAINING_WORK.md` (2.11),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-133),
  `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added,
  deleted, renamed: nothing. No entrenched file edited. **Authorship:** the owner's account through its GitHub no-reply
  address, Claude as co-author.
- Survey and doctor: at the start `survey check` ok (1,506 nodes, 14,421 edges), `doctor --check` 26 checks passed; after the
  last edit `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `41ce7c1`'s CI `38640213195` — success on every job; `actors_pingpong` arm64 MEASURED 2.95x (control 4.10x), the
  others NOT MEASURED. `db88e98`: the absence witnessed first (the daemon run's node `secret_to` `Null`; `--secret-to` unknown);
  mutants M402–M421 all red, the control green; the Z3 model 34 obligations, its three new mutants each caught; clippy clean;
  Windows and macOS linted from Linux; read on the runners — Windows `38720731643` (`within_cli` 15, `broker_cli` 5, the binary's
  181), macOS `38720736853` (`within_cli` 15), arm64 `38720741646` (`broker_cli` 6, `within_cli` 15), `delulu-broker`'s library
  on Windows `38720745681` (success) — 0 failed, every new test named, no orphan `delulu`; the full suite alone — 2,287 passed,
  0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL (`pip install z3-solver` for the model).
- For the laptop's memory: `HANDOFF.md` §11.5 — a literal ending `..Default::default()` drops a new field silently;
  `docs/assistant-memory/cloud-period-2026-09-28.md` a line.
- Open / next, in order: (1) read THIS commit's push run and the nightly `38702203302`; (2) **P9-04 step (d)'s second half** —
  the broker's `send` (`SendSecret`, the node's `secret_to`, the Guard on `NAME@origin`, a `send` record; the interpreter asks for
  the bytes after judging everything it can without them) — under way in this run; then (e); (3) RW 4.63, RW 4.53's rest, RW
  4.57, RW 4.58's O2/O4, RW 4.49's first two items; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-133,
  option (c). **The closing entry is current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-15 — routine run 25 (continued): P9-04 step (d)'s second half — the broker sends a secret, for one origin its node names (D-V2-134); a race witness made certain
- Session: `https://claude.ai/code/session_011TZbDxBVJVmBTVyDeiWhGb`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-carson-qm5vah`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `a97a953`
- Commits: (1) `6fe7076` P9-04 (d), second half: the broker sends a secret - for one origin its node names, recorded; (2)
  `df0fe6a` The queue_depth witness's overlap made certain: a barrier and a kernel that holds the slot; (3) this entry's commit
  — the records and `scripts/suite.sh`'s fingerprint
- Files and folders: (1) M `crates/delulu-broker/src/secrets.rs` (`send_secret_guarded`; its unit test),
  `crates/delulu-runtime/src/actors.rs` (a handle's audience crosses; its test), `custody.rs` (`Custody::send_secret`),
  `egress.rs` (`prejudge_secret`, `refuse_unsent`), `interp.rs` (`send_broker_secret`; the custody gate), `prim.rs`
  (`headers_arg`, `answer_value`, the handle refusal after the scheme and allowlist; the mint's bound handle), `value.rs`
  (`SecretVal::handle_bound`), `crates/delulu/src/broker_client.rs` (`send_secret`), `broker_ipc.rs` (`SendSecret`,
  `SecretBytes`), `brokerd.rs` (the handler; the lost-record witness), `crates/delulu/tests/broker_cli.rs` (the daemon witness
  rewritten; the leased witness), `docs/GETTING_STARTED.md`, `docs/for-agents.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/design/STAGE5_SPECIFICATION.md` (criterion 6), `docs/survey/*`. (2) M `crates/delulu-runtime/src/compute.rs` (the
  test), `docs/survey/*`. (3) M `scripts/suite.sh` (the fingerprint — loop engineering), `CHANGELOG.md`, `HANDOFF.md` (where
  things stand; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5 two lines — loop engineering),
  `docs/REMAINING_WORK.md` (2.11), `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `V2_DECISION_LOG.md` (D-V2-134), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing
  entry), `docs/survey/*`. Added, deleted, renamed: nothing. No entrenched file edited.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `a97a953`'s CI `38723018372` — success on every job (`formal` ran the 34-obligation model and its six mutants);
  `ocsf` `38723018286` and `openshell` `38723018349` success; `actors_pingpong` arm64 MEASURED 2.97x (control 4.11x), the
  others NOT MEASURED. The nightly `38702203302` at `41ce7c1` — success on every job, `heavy-gates` and `miri-slow` included.
  `6fe7076`: the absence witnessed first (both new CLI witnesses red DL0904 on `a97a953`'s source); mutants M422–M438 red but
  M428 and M432, implied walls (M428b red); M430 (the lost-record rule) red; clippy clean; Windows and macOS linted from Linux;
  runner reads in the `V2_LOG.md` entry (one red, the race below). `df0fe6a`: green 6 of 6 with four CPUs busy, M439 red, read
  green on Windows (274 passed), arm64 and macOS. The full suite alone at `df0fe6a` — 2,290 passed, 0 failed, 16 ignored (163
  binaries), cargo exit 0 (its `TREE-MOVED` a false alarm from a commit, `suite.sh` fixed).
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.5 — a slice can be written in a git worktree while the suite runs;
  `docs/assistant-memory/cloud-period-2026-09-28.md` a line.
- Open / next, in order: (1) read THIS commit's push run (and `6fe7076`'s and `df0fe6a`'s, the same push); (2) the OCSF mapping
  of a `send` (HTTP Activity with the destination) — under way; (3) **P9-04 step (e)** — `authority`'s per-secret destinations
  (a binding-aware walk: a secret through a `let`, the URL literal's origin; a computed one reported as such); then P9-05;
  (4) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49's first two items; (5) a boundary over a `.dwx`. **For the
  owner:** D-V2-115 to D-V2-134 (D-V2-134 chooses the Guard's `declassify` class for a send, with the pair as its token),
  option (c). **The closing entry is current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-15 — routine run 25 (continued): the OCSF export names a broker's `send` as HTTP Activity, its destination the endpoint
- Session: `https://claude.ai/code/session_011TZbDxBVJVmBTVyDeiWhGb`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-carson-qm5vah`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `9ed939f`
- Commits: (1) `b8da460` The OCSF export names a broker's send as HTTP Activity, its destination the endpoint; (2) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu-broker/src/ocsf.rs` (the class, the endpoint, the mapping table),
  `crates/delulu/tests/audit_ocsf_cli.rs` (the seeded sends), `docs/DEPLOYMENT.md`, `docs/survey/*`. (2) M `CHANGELOG.md`,
  `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-134's follow-up),
  `V2_P9_DESIGN.md`, `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: in the `V2_LOG.md` entry — witnessed red (class 0), OCSF 1.8.0 validation, M440/M441, Windows and macOS witness
  runs, the full suite alone at `b8da460` (2,290 passed, 0 failed, 16 ignored). `9ed939f`'s `ocsf` and `openshell` success.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new.
- Open / next, in order: (1) read `9ed939f`'s CI `38727198569` and THIS commit's push runs; (2) **P9-04 step (e)** —
  `authority`'s per-secret destinations — under way; then P9-05; (3) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW
  4.49's first two items; (4) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-134, option (c). **The closing
  entry is current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-15 — routine run 25 (continued): P9-04 step (e) — `authority --json` names where a program sends each secret (D-V2-135); P9-04 complete
- Session: `https://claude.ai/code/session_011TZbDxBVJVmBTVyDeiWhGb`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (through the harness branch `claude/zealous-carson-qm5vah`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `3f03ab0`
- Commits: (1) `b6128d1` P9-04 (e): `authority --json` names where a program sends each secret, and never guesses; (2) this
  entry's commit — the records
- Files and folders: (1) M `crates/delulu/src/within.rs` (`secret_destinations`, `SendWalk`), `cli.rs`
  (`stamp_secret_destinations`, on a file's and a package's report), `schema.rs` (the optional key),
  `crates/delulu/tests/within_cli.rs` (the witness), `docs/for-agents.md`, `CHANGELOG.md`, `docs/survey/*`. (2) M `HANDOFF.md`,
  `docs/CLOUD_ROUTINE.md` (step 4's P9 line), `docs/REMAINING_WORK.md` (2.11), `docs/assistant-memory/cloud-period-2026-09-28.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-135), `V2_P9_DESIGN.md`, `V2_PHASE_STATUS.md`,
  `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry), `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `9ed939f`'s CI `38727198569` — success on every job. `b6128d1`: absent first on `3f03ab0`'s source; M442–M448 red,
  M449 an implied reset, removed; clippy clean; `within_cli` and `json_contract` on Windows `38729783946` and macOS
  `38729787305` — success; the full suite alone — 2,291 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: nothing new beyond the memory line.
- Open / next, in order: (1) read THIS commit's push run and `3f03ab0`'s; (2) **P9-05** — method-and-path network scopes: read
  it against the code first (`net` is host-scoped in the grammar, the egress client and the broker's set inclusion alike);
  (3) proposing a secret's audience in `--grants`' lines from `secret_destinations`; (4) RW 4.63, RW 4.53's rest, RW 4.57, RW
  4.58's O2/O4, RW 4.49's first two items; (5) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-135, option (c).
  **The closing entry is current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-15 — routine run 25 (last): P9-05 read against the code; NET-WILDCARD-1 (D-V2-136); the red-team pass — F1 fixed (D-V2-137), F2–F4 recorded
- Session: `https://claude.ai/code/session_011TZbDxBVJVmBTVyDeiWhGb`   Model: Claude Opus 5.5 (no fallback notice); one sous-chef,
  Sonnet 5.5 (`claude-sonnet-5-5`), the red-team pass
- Branch: `master` (through the harness branch `claude/zealous-carson-qm5vah`, read on the runners, then fast-forwarded)
  Pull request: none   Merged: n/a
- Base: `0f73f09`
- Commits: (1) `c69506f` NET-WILDCARD-1: a pattern net grant means the same under the daemon as embedded - one host matcher;
  (2) `016b522` The one host matcher above the broker's validate tests, where clippy wants items; (3) `fa15654` The Guard's
  rule on a secret's name covers its sends (the red team's F1, D-V2-137); (4) this entry's commit — the records
- Files and folders: (1)–(2) M `crates/delulu-broker/src/validate.rs` (`host_matches`; the use check; its test),
  `crates/delulu-runtime/src/prim.rs` (`host_matches` delegates), `crates/delulu/tests/broker_cli.rs` (the witness),
  `docs/survey/*`. (3) M `crates/delulu-broker/src/guard.rs` (`pattern_matches`, `dead_pattern_reason`), `secrets.rs` (the
  witness; the revised test), `crates/delulu/tests/broker_cli.rs` (the revised lease witness), `CHANGELOG.md`,
  `docs/for-agents.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-134's correction note), `docs/survey/*`. (4) M
  `CHANGELOG.md`, `HANDOFF.md` (§11.4 two findings; §11.5 a lesson), `docs/CLOUD_ROUTINE.md` (step 4's P9 line; step 5 the
  red-team brief — loop engineering), `docs/REMAINING_WORK.md` (2.12; rows 4.65–4.67 added),
  `docs/assistant-memory/cloud-period-2026-09-28.md`, `docs/DELULULANG_V2/V2_LOG.md`, `V2_DECISION_LOG.md` (D-V2-136,
  D-V2-137), `V2_P9_DESIGN.md` (P9-05 read against the code), `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry),
  `docs/survey/*`. Added, deleted, renamed: nothing. No entrenched file edited.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: NET-WILDCARD-1 witnessed both ways on `0f73f09`'s binary, its CLI witness red on the old source, M450–M452 red;
  F1 re-witnessed (a broker unit test red), M453–M454 red; clippy clean (by cargo's own exit code); runner reads at `016b522`
  (Windows `38732093503`, macOS `38732097100`, broker library `38732100229` — success) and at `fa15654` (in `V2_LOG.md`); the
  full suite alone at `fa15654` — 2,294 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left. Push runs: `3f03ab0`'s CI `38729124643` and `ocsf` success, `0f73f09`'s
  `openshell` success (its CI below). An earlier suite at `016b522`, run while the red-team pass loaded the VM, was red once on
  `satellite_demo`'s re-contact scenario (a run exiting with nothing on stderr); green 3 of 3 alone at once, and in the suite
  above — recorded as load, per run 17's rule, not as a finding.
- Redo on the laptop: nothing beyond the suite on Windows and in WSL.
- For the laptop's memory: `HANDOFF.md` §11.4 — NET-WILDCARD-1 and F1, each with its search key; §11.5 — read clippy's exit
  code from cargo.
- Open / next, in order: (1) read THIS commit's push run and `0f73f09`'s CI; (2) **P9-05 step (a)** — path scopes in the
  grammar and the egress client (`V2_P9_DESIGN.md`'s build order); (3) RW 4.65–4.67 (the red team's F2–F4); proposing a
  secret's audience in `--grants`' lines; (4) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW 4.49's first two items;
  (5) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-137 (D-V2-137 changes what a `declassify` rule on a name
  covers), option (c). **The closing entry is current; from 2026-10-16 a run only refreshes it and stops.**

### 2026-10-15 — routine run 25 (postscript): `0f73f09`'s and `22501f6`'s push runs read
- Session: `https://claude.ai/code/session_011TZbDxBVJVmBTVyDeiWhGb`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `22501f6`
- Commits: (1) this entry's commit — the records
- Files and folders: (1) M `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md` (this entry; the closing entry's "as of"),
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): `survey check` ok, `survey findings` 0 errors (14 warnings, as before), `doctor --check` all checks passed
- Verified: `0f73f09`'s CI `38731264861` — success on every job (`miri-slow` and `heavy-gates` skipped on a push), arm64
  `actors_pingpong` MEASURED 2.99x (control 4.11x); `22501f6`'s CI `38736592686` — success on every job (16; `miri-slow` and `heavy-gates` skipped on a push), arm64 `actors_pingpong` MEASURED 2.99x (control 4.12x), and `ocsf` `38736592620` success: **`master` green at the end of the run**.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next, in order: (1) read THIS commit's push run (records only); (2) **P9-05 step (a)** — path scopes in the grammar and
  the egress client; (3) RW 4.65–4.67; proposing a secret's audience in `--grants`' lines; (4) RW 4.63, RW 4.53's rest, RW 4.57,
  RW 4.58's O2/O4, RW 4.49's first two items; (5) a boundary over a `.dwx`. **For the owner:** D-V2-115 to D-V2-137, option
  (c). **From 2026-10-16 a run only refreshes the closing entry and stops** (`docs/CLOUD_ROUTINE.md` step 0), unless the owner
  extends the dates.

### 2026-10-17 — the laptop synced to `b82a145`; the routine's 23 merged branches deleted (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `b82a145`
- Commits: (1) this entry's commit — the records
- Files and folders: M `docs/CLOUD_SYNC_LOG.md` (this entry; step 8 of *Syncing the laptop*; the closing entry's routine
  paragraph), `docs/DELULULANG_V2/V2_LOG.md`, `docs/assistant-memory/MEMORY.md` (its first three lines), `docs/survey/*`.
  Added, deleted, renamed: nothing.
- **The sync** (the procedure above): the laptop clean at the baseline `cfbfbdc` (`Cloud handoff (10)`); `git fetch` put
  `origin/master` 281 commits ahead and 0 behind — 92 by `Claude`, 189 by `jessesuniljs1-collab` through its no-reply
  address, every one co-authored by Claude (273 name Opus 5.5, 8 name Opus 5); `git merge --ff-only` to `b82a145`. Step 4:
  no `w/crlf` file. `master`, `rc/1.0.0-drill` and the `v1.0.0` tag identical on the laptop and on GitHub
  (`git ls-remote`). `git config core.hooksPath scripts/hooks` set (run 15's map check).
- Survey and doctor: **Windows** — `survey check` ok (1,506 nodes, 14,547 edges), `doctor --check` 29 checks passed. **WSL**
  (Ubuntu 20.04, the `D:` checkout, an ext4 target) — `survey check` ok, `survey findings` 0 errors (14 warnings, 38 notes),
  `doctor --check` 26 checks passed, KVM available. After this entry's last edit: in the commit message.
- Verified (CI, read): `b82a145`'s push run `38739567829` — success on every job (`miri-slow` and `heavy-gates` skipped on a
  push); the nightly `38869800819` on `b82a145` (2026-10-16) — success on every job, `miri-slow` on all three crates and
  `heavy-gates` included; the nightly `39013724751` (2026-10-17) was still running when this entry was written — read
  later the same day: success on every job, `miri-slow` and `heavy-gates` included. This entry's own commit, `ed120f1`:
  push run `39021046478` — success on every job.
- **Afterwards the same day — a disk cleanup** (the owner: *"Delete unwanted things. Log things deleted in .md file"*):
  `D:\nelan` 180 GB → 5.07 GB, almost all of it the Rust build cache `target` (178.4 GB, `cargo clean`), plus two spent
  build clones and their cache; then WSL's disk (*"clean it"*): its cargo caches, three verified clones and spent
  test scratch, `fstrim` and a compaction — `ext4.vhdx` 57.97 GB → 12.84 GB. **The WSL microVM guest images are
  still stale, and the Linux-filesystem clone they were built from is gone:** rebuild them from a fresh clone
  (`HANDOFF.md` §11.3). Recorded in `docs/maintenance/DISK-CLEANUP-2026-10-17.md`.
- **Branches** — the owner, 2026-10-17: *"claude routines made lot of branches. If not needed, should we remove them"*; asked
  to choose, the owner chose to delete the 23 merged branches and keep two. Deleted from GitHub, each checked with `git merge-base
  --is-ancestor <branch> origin/master` immediately before (nothing lost: every tip is in `master`'s history). To recreate
  one: `git push origin <tip>:refs/heads/claude/<name>`. Name and tip: `compassionate-pasteur-02il9s` `381fed8`, `friendly-thompson-c0cg57` `7940188`, `friendly-thompson-gsorc5` `dcf4fcb`, `friendly-thompson-w49pvi` `f33183c`, `happy-pascal-44t471` `ed7f7b6`, `happy-pascal-a1e0hm` `742b11b`, `happy-pascal-f86pm9` `b77c64d`, `jolly-hamilton-7fd30a` `6dbc54c`, `jolly-hamilton-dcf4b2` `ed95f0d`, `jolly-hamilton-mcawx1` `7a895f0`, `jolly-hamilton-vwdips` `f912084`, `jolly-hamilton-w805xo` `2cb3f87`, `stoic-ptolemy-1lso0u` `e936ea5`, `stoic-ptolemy-uyw9g4` `f446bfa`, `wonderful-hamilton-mjutjd` `9fc4d86`, `zealous-carson-qm5vah` `b82a145`, `zealous-carson-tidpo3` `41ce7c1`, `zealous-keller-an8lk9` `fc95094`, `zealous-keller-ekmk5a` `b7a5c3b`, `zealous-keller-kkuv5g` `21421ec`, `zealous-keller-r6to2u` `48ef512`, `zealous-keller-uym82b` `ae84188`, `zealous-keller-v2rh0i` `e95acf1`.
  **Kept:** `claude/wonderful-hamilton-bo8k53` `7d6a22b` and `claude/friendly-thompson-vz6m23` `d3f0d52` — one commit each
  that `master` lacks, superseded since run 10 and cited by hash in `V2_LOG.md` and this file. A cloud session makes a branch
  of its own on every fire, so a routine enabled again will make more.
- **The routine** (`trig_01GG41tCGxZXyt7bZUid8Vuu`, read 2026-10-17): `enabled: false`; last run 2026-10-15, session
  `cse_011TZbDxBVJVmBTVyDeiWhGb` (run 25), succeeded. The owner edited its prompt during the period; the lines added,
  verbatim: *"Check CI Runs (In GitHub Runners) if it are failure or success. If CI Run is failure make necessary changes to
  make it successful."*, *"Before starting check all previous routines or sessions works, Also check if it was completed and
  working before session limit hit. If not complete it and make it work. Also Check Last Commits Before Starting."*, *"Record
  everything to .md files. Always use and update Survey and Doctor."*, *"Always Commit as jessesuniljs1-collab OR both
  jessesuniljs1-collab and claude to my github repo."*, and after the closing-entry line *"(Doesn't means to stop
  everything, Dates may be extended)"*. The Claude-Docs connector is attached to it again (cleared on 2026-09-28; a routine
  needs none). The owner says routines are now called **scheduled tasks** in the product.
- Redo on the laptop — **not done**, each for the owner's word (CI ran the whole suite at `b82a145` on Windows, Linux, macOS
  and arm64): the full suite on Windows and in WSL; rebuilding the WSL microVM guest images (`~/microvm-image-a`, `-b`);
  `node editors/vscode/e2e.js` on Windows.
- For the laptop's memory: done — `cloud-period-2026-09-28.md` and `delulu-ipc-deadman-findings.md` copied from
  `docs/assistant-memory/` (the laptop's copies were still the `cfbfbdc` versions), and the index's first three lines brought
  up to date.
- Open / next: the closing entry's list — (1) **P9-05 step (a)**, path scopes in the grammar and the egress client; (2) RW
  4.65–4.67; proposing a secret's audience in `--grants`' lines; (3) RW 4.63, RW 4.53's rest, RW 4.57, RW 4.58's O2/O4, RW
  4.49's first two items; (4) a boundary over a `.dwx`. **For the owner:** D-V2-49 and D-V2-59, D-V2-110 to D-V2-137, P8-04's
  option (c); whether the routine (now a scheduled task) runs again, and with which dates.

### 2026-10-17 — `e1bc4df`'s push run red on macOS: a hex canary spelled by an audit hash; fixed (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `b4fb0a9`
- Commits: (1) this entry's commit
- Files and folders: M `crates/delulu/src/brokerd.rs` (one unit test's secret), `crates/delulu/tests/broker_cli.rs`,
  `crates/delulu/tests/egress_cli.rs`, `HANDOFF.md` (§11.5, two traps), `docs/CLOUD_SYNC_LOG.md` (this entry),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (7.18 closed, 7.19 observed), `docs/survey/*`. Added, deleted,
  renamed: nothing.
- **What** (`V2_LOG.md`, this date): `e1bc4df`'s push run `39025580650` was red on `test (macos-latest)` only — `broker_cli`'s
  leased-send test, *"the chain holds the bytes"*. The commit changes no code. P9-04 (d)'s two secret tests searched the
  audit chain for `8f2c`, four hex digits of the secret `hunter2-8f2c`, and a record's 64-hex-digit `hash` spelled it by
  chance. Witnessed on Windows, old code: 8 runs in 100 red; all 9 failed chains held `8f2c` only in a `hash`/`prev_hash`
  pair and `hunter2` nowhere. Now the secret is `hunter2-wq7z` and the canary `wq7z`, which is not hex; the same in
  `egress_cli` and in `brokerd`'s lost-send unit test.
- Survey and doctor (after the last edit): in the commit message.
- Verified: on the laptop (Windows) — after the fix 0 runs in 100 red; two leak mutants red at the canary checks
  (`broker_cli.rs:310`, `:404`; `egress_cli.rs:318`), restored and rebuilt; clippy exit 0; the suite 2,225 passed, 4 failed (the map's freshness, before its rebuild), 15 ignored. CI, read: `b4fb0a9`'s push run
  `39026595221` — success on every job (`miri-slow` and `heavy-gates` skipped on a push); `e1bc4df`'s `39025580650` as above.
  This entry's own push run: in the commit that follows it, or read in the next session.
- Redo on the laptop: nothing new.
- For the laptop's memory: `HANDOFF.md` §11.5 — a leak canary must be bytes no legitimate field can spell; a mutant script
  restores the source and rebuilds.
- Open / next: RW 7.19 — on Windows these two tests leave their temp directory behind (cause not confirmed). Otherwise the
  2026-10-17 entry's list stands.

### 2026-10-17 — the owner's commission recorded: every outstanding decision delegated, the release included (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `2f1992d`
- Commits: (1) this entry's commit — the first authored with the no-reply address
- Files and folders: A `docs/design/DeluluLang_Autonomous_Development_Prompt.md` (the owner's file, as he wrote it),
  `docs/assistant-memory/owner-release-delegation-2026-10-17.md`, `docs/assistant-memory/owner-pronouns.md`; M
  `AGENTS.md`, `CLAUDE.md`, `HANDOFF.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/assistant-memory/MEMORY.md`,
  `docs/assistant-memory/final-public-repo-gate.md`, `docs/assistant-memory/delulu-licensing-intent.md`,
  `docs/assistant-memory/mutant-run-per-target.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **What** (`V2_LOG.md`, this date; `HANDOFF.md`'s opening section): the owner's commission — take DeluluLang public;
  every outstanding decision, the owner-reserved ones included, is the head chef's; the release itself once objective
  criteria pass; its own limits kept. The licence was already decided (Apache-2.0, D27, 2026-07-24) — the head chef had
  told the owner otherwise earlier that day, and corrects it here. The repository now commits as
  `227307678+jessesuniljs1-collab@users.noreply.github.com`.
- Survey and doctor (after the last edit): in the commit message.
- Verified: `2f1992d`'s push run `39028791103` — success on every job (`miri-slow` and `heavy-gates` skipped on a push).
- Redo on the laptop: nothing.
- For the laptop's memory: done on the laptop — `owner-release-delegation-2026-10-17.md` and `owner-pronouns.md` added;
  `final-public-repo-gate.md` (superseded), `delulu-licensing-intent.md`, `mutant-run-per-target.md` and the index
  updated; the same here, sanitized as this folder's README asks.
- Open / next: the prompt's order — the audit, P9-05 step (a), then the decisions (each a `D-V2-nn`).

### 2026-10-17 — P9-05 step (a): network grants scoped to paths (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `f2d9407`
- Commits: (1) this entry's commit
- Files and folders: A `crates/delulu-broker/src/netscope.rs`; M `crates/delulu-broker/src/lib.rs`,
  `crates/delulu-broker/src/validate.rs`, `crates/delulu-runtime/src/broker.rs`, `crates/delulu-runtime/src/prim.rs`,
  `crates/delulu-runtime/src/interp.rs`, `crates/delulu-runtime/src/egress.rs`, `crates/delulu-runtime/src/egress/tests.rs`,
  `crates/delulu/src/openshell.rs`, `crates/delulu/tests/broker_cli.rs`, `crates/delulu/tests/egress_cli.rs`,
  `crates/delulu/tests/sandbox_openshell_cli.rs`, `CHANGELOG.md`, `skills/delulu/SKILL.md`, `docs/for-agents.md`,
  `HANDOFF.md` (§11.5), `docs/REMAINING_WORK.md` (2.12), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-138),
  `docs/DELULULANG_V2/V2_P9_DESIGN.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **What** (`V2_LOG.md`, this date; D-V2-138): `net=HOST/PATTERN`, one reading at every check under every custody; an
  ambiguous request path refused where a pattern decides; the special-use rule on the host part; the OpenShell export
  refuses a path scope.
- Survey and doctor (after the last edit): in the commit message.
- Verified: on the laptop (Windows) — the new tests witnessed red first; seven mutants red; the gates in the commit message.
  CI: this entry's push run, read in the commit that follows it.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 (two traps); the laptop's `mutant-run-per-target.md` gains the
  package-set lesson.
- Open / next: P9-05 step (b), then (c); (d) with the first non-GET primitive.

### 2026-10-17 — P9-05 step (b): the path order, by containment, proved (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `94b6c36`
- Commits: (1) this entry's commit
- Files and folders: M `crates/delulu-broker/src/netscope.rs`, `crates/delulu-broker/src/authority.rs`,
  `crates/delulu-runtime/src/prim.rs`, `crates/delulu/src/within.rs`, `crates/delulu/src/cli.rs`,
  `crates/delulu/tests/broker_cli.rs`, `crates/delulu/tests/within_cli.rs`, `docs/design/models/authority_algebra.py`,
  `docs/design/models/README.md`, `docs/MATHEMATICS.md`, `.github/workflows/ci.yml` (the Z3 steps: 41 obligations, nine
  mutants), `CHANGELOG.md`, `docs/REMAINING_WORK.md` (2.12), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-139),
  `docs/DELULULANG_V2/V2_P9_DESIGN.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- **What** (`V2_LOG.md`; D-V2-139): the `net` order by containment everywhere a set is compared, its meet, its canonical
  form; Z3 §2d.
- Survey and doctor (after the last edit): in the commit message.
- Verified: the laptop (Windows) — the agreement test red then green; eight Rust mutants and three Z3 mutants caught; Z3
  41 obligations; the gates in the commit message. CI: `94b6c36`'s (step (a)) push run `39034873839` — success on every job,
  the three operating systems' test jobs among them (`miri-slow` and `heavy-gates` skipped on a push).
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new beyond step (a)'s.
- Open / next: P9-05 (c)'s static half; the path in the chain's record (its own slice); (d) with a non-GET primitive.

### 2026-10-17 — the archive's notices and SBOM; the GPL image never distributed; the release's build reusable (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `bfb5a84`
- Commits: (1) this entry's commit
- Files and folders: A `scripts/third-party-notices.py`, `.github/workflows/release-build.yml`,
  `.github/workflows/scorecard.yml`; M `scripts/package-toolchain.sh`, `.github/workflows/release.yml`,
  `crates/delulu/tests/distribution.rs`, `CHANGELOG.md`, `HANDOFF.md` (§1.1, the destination),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-143, D-V2-144), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted, renamed: nothing.
- **What** (`V2_LOG.md`; D-V2-143, D-V2-144): notices and an SBOM in every archive, legal files required, the image never
  distributed, the release's build in a reusable workflow, Scorecard on the public repository.
- Survey and doctor (after the last edit): in the commit message.
- Verified: on the laptop — the generator for four targets, the packaging end to end on Windows, `distribution.rs`.
  CI: this commit's push run and a release dry run (`workflow_dispatch`), read in the next commit.
- Redo on the laptop: nothing.
- For the laptop's memory: the empty `DeluluLang` repository is the destination (`HANDOFF.md` §1.1).
- Open / next: the remaining decisions (SECURITY.md and the repository's controls at its creation, rustfmt, a code of
  conduct, the root-issuance default, RW 4.19, P8-04 (c)); the WSL and editor checks; the scheduled task; the release.

### 2026-10-17 — rustfmt adopted; the tree formatted once (made on the laptop)
- Session: `https://claude.ai/code/session_01XxNxT5sWUFtXDgDUuHCC6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `6e0637e`
- Commits: (1) this entry's commit
- Files and folders: A `rustfmt.toml`; M 283 Rust source files (formatting only — `git diff --stat` in the commit), the two
  test files' double-encoded characters, `.github/workflows/ci.yml` (the check), `CONTRIBUTING.md`, `CHANGELOG.md`,
  `docs/REMAINING_WORK.md` (7.9), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-145), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **What** (`V2_LOG.md`; D-V2-145): rustfmt adopted and enforced.
- Survey and doctor (after the last edit): in the commit message.
- Verified: in the commit message (clippy, the full suite after the reformat). CI read: `bfb5a84` `39036694401` and
  `6e0637e` `39037513753` success on every job; the release dry run `39037514962` success on all four targets, the
  notices and SBOM generated on each. This commit's own run: read in the next commit.
- Redo on the laptop: nothing.
- For the laptop's memory: `cargo fmt --all` before every commit now — CI's `lints` job fails an unformatted tree.
- Open / next: as the previous entry.

### 2026-10-17 — the on-demand cloud routine (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `43d8a7c`
- Commits: (1) this entry's commit
- Files and folders: M `docs/CLOUD_ROUTINE.md`, `AGENTS.md`, `HANDOFF.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-146), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/assistant-memory/*`, `docs/survey/*`. Deleted,
  renamed: nothing.
- **What** (D-V2-146): the routine `trig_01HFG6usfzmDAPQigiDu9DLq`, created paused; every session starts with `git fetch`.
- CI read: `43d8a7c` — CI `39039217277`, ocsf `39039217297`, openshell `39039217270`: success.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: the routine's ID and that it is paused; fetch `origin` first in every session.
- **Open / next, in order** (the release program; *[laptop]* = needs the owner's laptop, a routine run leaves it):
  1. **RW 4.19, the driver READY handshake — half built, NOT in `master`.** The laptop holds it uncommitted (a `git stash`
     on the laptop, "RW 4.19 READY handshake (in progress)"). Its design, for whoever finishes it: a driver prints `READY`
     before its first command; `STARTUP_DEFAULT` 10 s, `STARTUP_MAX` 120 s; `AdapterError::NotReady`; `ProcessAdapter::spawn`
     and `LineTransport::spawn` take a startup `Duration`; a `--adapter-startup-ms N` flag (1..=120000; refused where no
     driver starts); every CLI test driver (`hw_adapter_cli.rs`, `hw_dpx_cli.rs`, `sandbox_devices_cli.rs`,
     `dead_man_cli.rs`, `sandbox_modes_cli.rs`) prints `READY` first; CHANGELOG (breaking for driver authors);
     DEPLOYMENT.md; RW 4.19 closed; its decision D-V2-147. A routine run that starts before the laptop pushes it should
     take the next item instead, so the two never collide.
  2. A code of conduct: the Contributor Covenant 2.1, verbatim from its official source; reports through GitHub's private
     reporting.
  3. Record the decisions taken but not yet written: RW 4.5 (root issuance stays opt-in — ROOT_ISSUANCE_TRUST_BOUNDARY
     §10's own recommendation); version **2.0.0**; P8-04 option (c) to a post-release RFC; and update *Owner decisions
     carried from V1, still open* in `V2_DECISION_LOG.md`.
  4. The README: stale status rows; the D-V2-59 note that `contained` is best-effort.
  5. *[laptop]* Rebuild the microVM images under WSL from a fresh ext4 clone (`scripts/microvm/build-image.sh`); the WSL
     suite; `node editors/vscode/e2e.js` on Windows (a routine run may run it on Linux under `xvfb-run`).
  6. *[laptop]* The launch: configure the empty public `jessesuniljs1-collab/DeluluLang` (private vulnerability
     reporting, secret scanning, push protection, `main` protected); `SECURITY.md` §1/§6, CODEOWNERS, the governance
     test; reword the two `HANDOFF.md` rule lines that spell the banned word; the 2.0.0 bump and CHANGELOG; the
     fresh-history snapshot; tag `v2.0.0`; `RELEASES=on`; verify archives and attestations; publish; then
     `delululang-test` private as the archive; the final report.

### 2026-10-17 — RW 4.19 closed: the driver READY handshake (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `d68ff67`
- Commits: (1) this entry's commit
- Files and folders: M `crates/delulu-runtime/src/adapter.rs`, `crates/delulu-runtime/src/sim.rs`,
  `crates/delulu/src/cli.rs`, `crates/delulu/src/run_cmd.rs`, `crates/delulu/tests/hw_adapter_cli.rs`,
  `crates/delulu/tests/hw_dpx_cli.rs`, `crates/delulu/tests/sandbox_devices_cli.rs`, `CHANGELOG.md`, `docs/DEPLOYMENT.md`,
  `docs/REMAINING_WORK.md` (4.19), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-147), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/assistant-memory/delulu-hw-adapter.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **What** (D-V2-147): a driver says `READY` before `main`; `--adapter-startup-ms`.
- Survey and doctor (after the last edit): in the commit message. Suite: 2,242 passed and 4 failed before the Survey was regenerated — all four the stale-map gates (doctor_cli 3, the Survey's freshness 1), which pass after it (rerun after the last edit).
- Redo on the laptop: nothing (made there).
- For the laptop's memory: a test driver must print `READY` first.
- Open / next: item 1 of the previous entry is DONE; items 2–6 stand (the code of conduct and the decisions are next).

### 2026-10-17 — the code of conduct; RW 4.5, the version and P8-04 (c) recorded (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `f2a19db`
- Commits: (1) this entry's commit
- Files and folders: A `CODE_OF_CONDUCT.md`; M `CONTRIBUTING.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md`
  (4.5), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-148 to D-V2-151), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; and `AGENTS.md`, `docs/CLOUD_ROUTINE.md`,
  `docs/assistant-memory/agent-usage-rule-2026-09-17.md` — the owner's word that routine runs, like laptop sessions, may use
  Sonnet 5.5 and Haiku 5.5 sous-chefs when needed (Haiku 5.5 is released), mindful that parallel agents cost more usage.
  Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- Open / next, in order: (4) the README — stale status rows, the D-V2-59 note that `contained` is best-effort, the code
  of conduct linked; (5) *[laptop]* the WSL microVM images, the WSL suite, the VS Code test on Windows; (6) *[laptop]*
  the launch (as listed two entries above). Items 1–3 of that list are done.

### 2026-10-17 — the README, CLAUDE.md and the status rows brought current (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `0fc64e1`
- Commits: (1) this entry's commit
- Files and folders: M `README.md` (the status table: 2.0.0 being prepared, the latest CI run, the public repository
  named, V2's done list through P9, the next step, the on-demand routine, a new row — the sandbox's default `contained`
  does not require filesystem confinement and reports an unmeasured wall, D-V2-59/D-V2-92 — and the code of conduct
  linked), `CLAUDE.md` (GitHub first; the on-demand routine), `HANDOFF.md` (the 2026-09-28 snapshot marked as one),
  `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- Open / next, in order: (5) *[laptop]* the WSL microVM images, the WSL suite, the VS Code test on Windows (a routine run
  may run `editors/vscode/e2e.js` on Linux under `xvfb-run`); (6) *[laptop]* the launch, as listed three entries above.

### 2026-10-17 — the banned word gone from the tree; the VS Code test on Windows (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `f731e5b`
- Commits: (1) this entry's commit
- Files and folders: M `HANDOFF.md` (the two rule lines and §1.1 item 1), `AGENTS.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-152), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **Validated on the laptop:** `node editors/vscode/e2e.js target/debug/delulu.exe` on Windows 11 with a freshly packaged
  `delulu-lang.vsix` (binary built from `f731e5b`'s code): **passed** — "extension activated and the language server started
  with no errors reported", exit 0. The WSL validation (a fresh ext4 clone, the microVM image rebuilt from source, the
  KVM-gated tests, the whole suite) was running when this entry was written; its result goes in the next entry.
- Survey and doctor (after the last edit): in the commit message.
- Open / next: (5) the WSL result; (6) *[laptop]* the launch — pre-public item 1 (the banned word) is now done.

### 2026-10-17 — the machine-level validation recorded (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `ca7a238`
- Commits: (1) this entry's commit
- Files and folders: M `docs/DELULULANG_V2/V2_LOG.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **Validated** (`V2_LOG.md`, this date): VS Code e2e on Windows passed; the microVM image rebuilt from source in WSL
  (exit 0); `microvm_cli` 10/10 + its root test 1/1 as root, `microvm_criterion8` 1/1; the WSL suite 2,311 passed, 0
  failed; CI for `f2a19db`, `d68ff67`, `0fc64e1` read green. The commission's step 4 is DONE.
- Routine: renamed by the owner's request to *Head Chef — on demand (start by hand)*; still paused.
- Open / next — **the launch** (*[laptop]*, a fresh session with a full budget; the order in `HANDOFF.md` §1.1 and the
  laptop memory `release-program-progress`): (a) decide where development continues after the launch — the public
  repository with a fresh history, or this one — and rewrite `AGENTS.md`'s push rule and the routine's repository to
  match, BEFORE the public push; (b) configure the empty `jessesuniljs1-collab/DeluluLang` (private vulnerability
  reporting, secret scanning, push protection; protect `main` once it exists); (c) `SECURITY.md` §1/§6 (the private
  reporting form as the channel — no invented address or key; SLSA L2 until `gh attestation verify --signer-workflow
  …/release-build.yml` passes; the SBOM per archive), CODEOWNERS `@jessesuniljs1-collab` and
  `governance.rs::pending_public_controls_are_still_marked_as_pending`, each entrenched edit in
  `ENTRENCHED_CHANGE_RECORD.md`; (d) the 2.0.0 bump (the workspace `Cargo.toml` and `Cargo.lock`; check whether
  `editor_contract.rs` ties the extension's version to it) and the CHANGELOG's `## [2.0.0]`; (e) the fresh-history
  snapshot (no-reply author), tag `v2.0.0`, `RELEASES=on`, verify the archives and attestations, publish; (f) this
  repository private as the archive, after verifying; (g) the final report (the prompt's §7).

### 2026-10-17 — interactive cloud sessions continue the work too; a start-up hook fetches GitHub (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `5d4f6a4`
- Commits: (1) this entry's commit
- Files and folders: M `CLAUDE.md`, `docs/CLOUD_ROUTINE.md`, `HANDOFF.md` (§11.1), `.claude/settings.json` (a `SessionStart`
  hook: `git fetch -q origin`, the commits `origin/master` has that the session lacks, the `origin/claude/*` branches — so
  every session, laptop or cloud, sees what came in before it does anything), `docs/assistant-memory/*`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **What: *"Not just routine or scheduled task runs, I will be using claude code cloud sessions also occasionally using test GitHub repo … I don't want delululang development and testing and addition of new features to be stopped just because of my lap not available and I am busy with other projects."*** An interactive cloud session without a narrower request follows `docs/CLOUD_ROUTINE.md` like
  a routine run; it pushes its own branch and opens a pull request, which the next session merges after reading its CI.
- Survey and doctor (after the last edit): in the commit message.
- Open / next: as two entries above — the launch (laptop), first deciding where development continues after it.

### 2026-10-17 — the public repository's controls; where development lives after the launch (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `218f471`
- Commits: (1) this entry's commit
- Files and folders: M `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-153), `HANDOFF.md` (§1.1 item 4),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **Outside the tree:** on `jessesuniljs1-collab/DeluluLang` (still empty) private vulnerability reporting switched ON and
  Dependabot alerts ON; secret scanning and push protection were already ON — all four read back.
- Open / next — the launch, in order: (1) `SECURITY.md` §1/§6 and CODEOWNERS (+ `governance.rs`), entrenched record;
  (2) the cutover edits of D-V2-153 §3; (3) the 2.0.0 bump and CHANGELOG; (4) the snapshot push to `DeluluLang` `main`,
  then branch protection; (5) tag `v2.0.0`, `RELEASES=on`, verify archives and attestations, publish; (6) the laptop's
  `origin` and the routine switched to `DeluluLang`; `delululang-test` private; (7) the final report.

### 2026-10-17 — `SECURITY.md`'s controls decided; CODEOWNERS names the maintainer (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `30a340a`
- Commits: (1) this entry's commit
- Files and folders: M `SECURITY.md` (entrenched), `.github/CODEOWNERS`, `crates/delulu/tests/governance.rs` (the test
  rewritten, watched red first), `crates/delulu-survey/src/codeowners.rs` (a comment), `docs/for-agents.md`,
  `docs/design/ENTRENCHED_CHANGE_RECORD.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-154), `docs/REMAINING_WORK.md`
  (7.8), `HANDOFF.md` (§1.1 item 3), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Open / next — the launch: (2) the cutover edits of D-V2-153 §3; (3) the 2.0.0 bump and CHANGELOG; (4) the snapshot push
  and branch protection; (5) the tag, `RELEASES=on`, verification, publishing; (6) `origin`, the routine, the archive;
  (7) the final report.

### 2026-10-17 — `launch/2.0.0` exists: do not merge it into `master` (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `23ca64c`
- Commits: (1) this entry's commit
- Files and folders: M `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **The prepared launch commit is `f34fea4` on `launch/2.0.0`** (the 2.0.0 version, the CHANGELOG's `## [2.0.0]`, the
  cutover of D-V2-153 §3; its own sync-log entry is on that branch). Full CI dispatched on it: run `39055892355`.
- **For every session until the launch:** keep working on `master` as before; **never merge `launch/2.0.0` into
  `master`** (it would make `master` claim an unreleased 2.0.0 and point the rules at a repository that is still empty).
  The launch session first merges `master` INTO `launch/2.0.0` (so nothing done meanwhile is lost), re-runs the gates and
  CI there, then dates the CHANGELOG, snapshots that tree into `DeluluLang` `main`, tags `v2.0.0`, verifies, publishes,
  and only then moves `origin`, the routine and the archive.
- Open / next: the launch, as above. Meanwhile a routine run or cloud session takes ordinary work (`docs/CLOUD_ROUTINE.md`
  step 4, items 5–7: `REMAINING_WORK.md` rows, sweeps, features), never the launch.

### 2026-10-17 — the autosave: the laptop's work reaches GitHub even when a session stops (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `9829662`
- Commits: (1) this entry's commit
- Files and folders: A `scripts/autosave.ps1`; M `AGENTS.md` (*Read first* 0), `HANDOFF.md` (§11.1),
  `docs/CLOUD_ROUTINE.md` (step 1), `.claude/settings.json` (the start-up hook also lists `origin/autosave/*`),
  `docs/REPOSITORY_STRUCTURE.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted,
  renamed: nothing.
- **Why:** the owner, 2026-10-17 — *"Head Chef continue cooking till usage limit hits. Can u run something parellely so
  that everything will get pushed to github repo even if usage limit hits"*. A sous-chef or a routine draws on the same
  usage and stops with the session, so the safety net runs outside Claude: Windows Task Scheduler, task
  `DeluluLang-autosave`, every 10 minutes (installed 2026-10-17 18:21 UTC for 7 days; re-installed 18:28 UTC for **7 hours** at the owner's word, *"Maybe next 7 hrs is enough … We can start task scheduler later"* — `-Install` starts it again, `-Hours N` sets the span).
- **What it does:** pushes every local branch ahead of its upstream (plain push; a diverged branch is only logged), and
  snapshots uncommitted work to **`autosave/laptop`** through a private index — HEAD, the index and the working tree are
  untouched; parents are the previous snapshot and HEAD, so the branch only fast-forwards; CI does not run on it.
- **Verified:** a run by hand pushed snapshot `5fb039a` (base `9829662`, 1 path, `git status` unchanged after); a second
  run read `unchanged since snapshot 5fb039a`; the scheduled task's first run returned 0 with the same line in its log;
  the banned-word guard, run with a probe token and a probe file, refused and pushed nothing (`ls-remote` showed no
  probe branch).
- Open / next: unchanged — the launch (the entry before this one).

### 2026-10-17 — the autosave refuses to carry history `origin` does not have (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `4681008`
- Commits: (1) this entry's commit
- Files and folders: M `scripts/autosave.ps1`, `HANDOFF.md` (§11.1), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **Found while writing the launch procedure:** after the launch `origin` is the public `DeluluLang` with a fresh history
  (D-V2-153). A snapshot's parents are HEAD and the previous snapshot, so a laptop still on the development history would
  have pushed all of it — the commits the fresh history exists to leave out — into the public repository.
- **Fixed:** `fetch --prune` (the old repository's refs go once `origin` points elsewhere), and a snapshot only when HEAD
  is on some branch of `origin`; otherwise `REFUSED … nothing pushed`. A `-State` directory parameter for tests.
- **Falsified on throwaway repositories** (a bare origin with history A; a clone on an unrelated history B, edited): the
  guarded script refused, exit 1, no remote branch; the previous script, same state, pushed (the witness); control —
  HEAD on `master`, which origin has — a snapshot pushed. The task was re-installed with the guarded copy.
- Open / next: unchanged — the release procedure in the 2.0 checklist on `launch/2.0.0`.

### 2026-10-17 (night) — the owner: only the test repository until he says otherwise (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `52071e5`
- Commits: (1) this entry's commit
- Files and folders: M `AGENTS.md` (the push rule), `HANDOFF.md` (§11.1), `docs/DELULULANG_V2/V2_DECISION_LOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **The owner:** *"Only push to and use test repo until I tell u to change. Do not change repo, unless told by me. Always
  push only to my repo"*. Every push goes to `delululang-test`. Nothing was ever pushed to `DeluluLang` (its `ls-remote`
  is empty); no tag was pushed; `origin`, the routine and both repositories' visibility are unchanged.
- **The launch waits for his word:** `launch/2.0.0` (on `delululang-test`) holds the prepared release — the 2.0.0 version,
  the CHANGELOG section, the release gate (the 2.0 checklist under `docs/release/` on that branch) with steps 1–3 doable and 4–10 held. The CI
  everything run and the release dry run on it are read and recorded as they finish.
- Open / next: ordinary work on `master` (`docs/CLOUD_ROUTINE.md` step 4); keep `launch/2.0.0` current by merging
  `master` into it; the launch only when the owner says.

### 2026-10-17 (night) — RW 4.65: a Guard refusal asks for the narrowest permit; the launch CI read (made on the laptop)
- Session: `https://claude.ai/code/session_01TeFcPssxRcTvPXJpY7qdyn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `9ad8e9c`
- Commits: (1) this entry's commit
- Files and folders: M `crates/delulu-broker/src/guard.rs`, `crates/delulu-broker/src/diag.rs`,
  `crates/delulu/src/brokerd.rs` (a test), `crates/delulu/tests/guard_e2e.rs`, `CHANGELOG.md`, `docs/REMAINING_WORK.md`
  (4.65 closed), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **RW 4.65 closed:** DL1410's command asks for the narrowest permit that covers the use (`declassify:NAME@host:port`,
  the file, `effect:<E>`), the rule beside it. An item a shell would split, expand or run never enters the command — the
  item is the program's and the hint is pasted into shells. Witness red on the old code; two mutants red; `guard_e2e`
  runs the printed command. clippy 0; Windows suite 2,244 passed (the 4 map-freshness tests green once the Survey was
  regenerated).
- **CI read (`delululang-test`):** the launch branch's `jobs=everything` runs — `39055892355` on `f34fea4` and
  `39058514393` on `130193a` — **success on all 18 jobs each**, `miri-slow (delulu-broker)` included (2 h 54 min and
  2 h 13 min); the release dry run `39058516429` on `130193a` **success** (four targets packaged and installed at 2.0.0,
  the release job skipped — no tag). Master push runs `39057886362` (`424c391`), `39058210143` (`4681008`),
  `39058694156` (`52071e5`), `39059809187` (`9ad8e9c`): success. CI does not run on pushes to `launch/2.0.0` or
  `autosave/*` (it triggers on `master`/`main`); the launch branch is run by hand.
- Open / next: RW 6.1 (CLI strings in the catalog); the launch waits for the owner's word.

### 2026-10-18 — on-demand routine run 1: RW 6.1 recovered from the laptop's autosave, finished and verified; RW 7.10's stale row
- Session: `https://claude.ai/code/session_01FzcPzLxjK3s2xT743XXr51`   Model: Claude Opus 5.5 (`claude-opus-5-5`; the effort is
  `.claude/settings.json`'s xhigh — no other model or effort shown; no fallback notice; no sous-chef)
- Routine: `trig_01HFG6usfzmDAPQigiDu9DLq`, its first run (started 2026-10-18 20:06 UTC, no `routine-fire-payload`)
- Branch: `master`   Pull request: none   Merged: n/a
- Base: `640a7d5`
- Commits: (1) this entry's commit
- Files and folders: A `docs/assistant-memory/autosave-snapshot-is-a-draft.md`; M `CHANGELOG.md`, `HANDOFF.md`,
  `crates/delulu-diag/catalogs/delulu-slang.toml`, `crates/delulu-diag/src/catalog.rs`, `crates/delulu-diag/src/lib.rs`,
  `crates/delulu/src/cli.rs`, `crates/delulu/tests/locale_cli.rs`, `crates/delulu/tests/locale_plugin_cli.rs`,
  `docs/CLOUD_ROUTINE.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md`,
  `docs/assistant-memory/MEMORY.md`, `docs/design/LOCALIZATION_PLUGIN_GUIDE.md`, `docs/lang/README.md`,
  `docs/lang/delulu-slang.md`, `docs/lang/en-US.md`, `docs/survey/*` (3). Deleted, renamed: nothing.
- **Recovered:** `origin/autosave/laptop`'s newest snapshot `b4a9e18` (2026-10-17 21:15 UTC, base `640a7d5`, 14 paths) — the
  laptop's RW 6.1, begun in `session_01TeFcPssxRcTvPXJpY7qdyn`, which stopped before committing; its draft entry here is
  replaced by this one, and its `V2_LOG.md` entry rewritten as this run's. The four older snapshots are byte-identical to
  commits that followed them (`5fb039a` → `424c391`; `6bf8f8e` → `3f652cf` on `launch/2.0.0`; `36ff555` and `4d407f5` →
  `640a7d5`). The autosave branch is not merged (never).
- **RW 6.1:** three named CLI strings (`cli.authority.header`, `cli.authority.foreign-separator`, `cli.locale.add.confirm`),
  rendered through the active catalog or en-US whole; slang speaks all three; a third catalog's key installs and renders.
  Re-witnessed here: both end-to-end witnesses red on the old product code, the mutant red. **Fixed in the draft:** a
  catalog's line break started a forged line of the `authority` report (`terminal_safe` → `terminal_line`, witnessed red
  first); a doc comment carried onto the wrong test.
- **The Sunday sweep:** CI = the nightly (every job, green); an adversarial pass by hand on RW 4.65's DL1410 hint — `,`
  under PowerShell (pwsh 7.6.7 installed in the VM) and `*` widening the permit, both refuted (`V2_LOG.md`); `cmd.exe` and
  Windows PowerShell 5.1 unread. **RW 7.10 closed in the record:** `CODE_OF_CONDUCT.md` has existed since 2026-10-17
  (D-V2-151); the row still said absent.
- Survey and doctor: at the start `survey check` ok (1519 nodes, 14775 edges), `doctor --check` ok (all checks pass); after
  the last edit: in the commit message. `gh` is installed with an invalid `GH_TOKEN` (as run 10 found); CI read with `curl`.
- Verified: the full suite 2,318 passed, 0 failed, 16 ignored (exit 0); `delulu-diag` 50/50, `locale_cli` 7/7,
  `locale_plugin_cli` 3/3, clippy 0, `cargo deny … advisories` ok. CI read before the push: `39069505317` (CI) and
  `39069505339` (`ocsf`) on `640a7d5` — success; the nightly `39108708245` (all 18 jobs) — success; `actors_pingpong` and
  the sandbox properties per OS in `V2_LOG.md`.
- Redo on the laptop: optional — paste a DL1410 hint holding `,` and a quoted path into `cmd.exe` and Windows
  PowerShell 5.1 (unread in the VM). The RW 6.1 change is OS-neutral; CI runs it on every OS.
- For the laptop's memory: `HANDOFF.md` §11.5 (a slot's line count is part of its escape; an autosave snapshot is a draft),
  `docs/assistant-memory/autosave-snapshot-is-a-draft.md`; `docs/CLOUD_ROUTINE.md` step 1 says how to recover one.
- Open / next: this commit's push run (read by this run if its budget allows, else first by the next); keep `launch/2.0.0`
  current by merging `master` into it (a laptop or cloud session that can run its gates); more CLI strings as surfaces earn
  them (RW 6.1); the launch only when the owner says.

### 2026-10-18 — on-demand routine run 1 (continued): the Survey mapped a worktree's `.git` file; `launch/2.0.0` kept current
- Session: `https://claude.ai/code/session_01FzcPzLxjK3s2xT743XXr51`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master` (and `launch/2.0.0`, release procedure step 1)   Pull request: none   Merged: n/a
- Base: `a3d7678`
- Commits: (1) this entry's commit on `master`; then on `launch/2.0.0` a merge of it (its hash in the next entry)
- Files and folders: M `crates/delulu-survey/src/scan.rs`, `crates/delulu-survey/tests/clean_checkout.rs`, `HANDOFF.md`
  (§11.5), `docs/CLOUD_ROUTINE.md` (step 5), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Deleted, renamed: nothing.
- **Found:** the first commit made from a `git worktree` (the launch merge) was refused by the pre-commit hook — its map
  held `file:.git`, a worktree's `.git` being a file the walk did not skip. Witnessed red, fixed, green (`V2_LOG.md`).
- Survey and doctor (after the last edit): in the commit message.
- Verified: the full suite 2,319 passed, 0 failed, 16 ignored (cargo's exit 0, the tree unmoved).
- CI read: `a3d7678`'s push runs — CI `39163449226` success (14 jobs, `miri-slow` and `heavy-gates` skipped as on every push; `actors_pingpong`: arm64 MEASURED 2.88x where the machine gave 4.10x, passed; Ubuntu and Windows NOT MEASURED, busy; macOS NOT MEASURED, 3 threads) and `openshell` `39163449144` success.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 (a worktree's `.git` is a file).
- Open / next: the launch merge's CI (`jobs=everything`, dispatched by this run — read it); RW 6.1 grows as surfaces
  earn strings; the launch only when the owner says.

### 2026-10-18 — on-demand routine run 1 (continued): RW 4.17 — a Guard seal and the other Unicode normalization on macOS (D-V2-156)
- Session: `https://claude.ai/code/session_01FzcPzLxjK3s2xT743XXr51`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `claude/rw-4-17-normalization` (the runner witnesses), fast-forwarded into `master`   Pull request: none
- Base: `c4be914`
- Commits: `ccb97f5` Witness (RW 4.17): two Unicode normalizations of one file, one spelling?; `80a6243` Witness (RW
  4.17): a Guard path rule and the other Unicode normalization (RED on macOS); `7d04370` A Guard path rule covers the
  other Unicode normalization on macOS (RW 4.17); and this entry's commit
- Files and folders: M `crates/delulu-broker/Cargo.toml` (`icu_normalizer` on macOS only), `Cargo.lock` (one line),
  `crates/delulu-broker/src/guard.rs`, `crates/delulu-runtime/src/prim.rs` (a test), `CHANGELOG.md`, `HANDOFF.md`
  (§11.4, GUARD-NORM-1), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-156), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/REMAINING_WORK.md` (4.17 partial), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **Found and fixed (GUARD-NORM-1):** on macOS a Guard seal on a name not yet created, written in one Unicode
  normalization, was walked past by a write in the other — APFS treats them as one name, and only an EXISTING file's
  spelling is unified by `realpath` (measured). The Guard compares path rules in NFC on macOS now, nowhere else.
- Verified: the macOS reads in `V2_LOG.md` (red first, then green on four targets); the full suite 2,321 passed, 0 failed;
  `delulu-broker` 204/204; clippy 0; `check-other-os.sh aarch64-apple-darwin` clean.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing (macOS read on runners; Windows and Linux unchanged).
- For the laptop's memory: `HANDOFF.md` §11.4 GUARD-NORM-1 (a resolution that fixes the spelling only of what EXISTS).
- Open / next: this commit's push run; `launch/2.0.0`'s `jobs=everything` run `39165405002` on `9763787` (dispatched by
  this run; read it — `miri-slow` takes hours) and its release dry run `39165442908` (success); merge `master` into
  `launch/2.0.0` again for RW 4.17 (its CHANGELOG lines go into the 2.0.0 section); RW 4.17's UNC half.

### 2026-10-18 — on-demand routine run 1 (last): the e-stop witness observes its baseline; the launch branch's CI read
- Session: `https://claude.ai/code/session_01FzcPzLxjK3s2xT743XXr51`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `claude/estop-observed-baseline` (its runner reads), fast-forwarded into `master`   Pull request: none
- Base: `c7d3187`
- Commits: `65a4ec7` The e-stop witness waits for the guest's first command, not 300 ms; and this entry's commit
- Files and folders: M `crates/delulu/tests/sandbox_devices_cli.rs`, `docs/CLOUD_ROUTINE.md` (step 4: how a VM keeps
  `launch/2.0.0` current), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-156's
  note: D-V2-155 is the support matrix, not "the release gate"), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted, renamed: nothing.
- **On `launch/2.0.0` this run:** `9763787` and `b8f9f16`, two merges of `master` (release procedure step 1), each with
  the gates green on the launch tree; CI `jobs=everything` and the release dry run dispatched on each (`V2_LOG.md`).
- Survey and doctor (after the last edit): in the commit message.
- Verified: the e-stop witness red 3 of 3 with its window grown, green 3 of 3 after; its target 4 of 4 under load; the
  runner reads in `V2_LOG.md`. `c7d3187`'s push runs — CI `39167159924` and `ocsf` `39167159926` — success.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new (routine runs 17–19's baseline lessons, applied).
- Open / next, in order: (1) this commit's push run; (2) `launch/2.0.0`: read `39167478125` (`jobs=everything` on
  `b8f9f16`) and `39167480432` (its release dry run) — `39165405002` on `9763787` is red only in this witness, its Miri
  jobs as read in `V2_LOG.md`; then merge `master` into it again (this fix, these records) and dispatch `everything`
  on the result — criterion 1 of `CHECKLIST-2.0.md` needs a green run on the final launch commit; (3) RW 4.17's UNC
  half; RW 6.1's further strings; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-18 — on-demand routine run 1 (postscript): the last push run read; `launch/2.0.0` at `e098438`
- Session: `https://claude.ai/code/session_01FzcPzLxjK3s2xT743XXr51`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Base: `9ba06d2`   Commits: this entry's commit
- Files and folders: M `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Deleted, renamed: nothing.
- CI read: `9ba06d2`'s push run `39169109890` — success (14 jobs; `miri-slow` and `heavy-gates` skipped, as on every push).
- `launch/2.0.0`: the third merge, `e098438` (this fix and the records), gates green on the launch tree; CI `39169269103` (`jobs=everything`) dispatched — at 22:01 UTC 14 jobs success, the Windows tests and three `miri-slow` running. `39165405002` (`9763787`): 16 success, Windows red in the fixed witness, `miri-slow (delulu-broker)` running. `39167478125` (`b8f9f16`): 16 success, Windows green this time, two `miri-slow` running. All unread past this line.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, in order: (1) this commit's push run (records only); (2) on `launch/2.0.0` read `39169269103`
  (`jobs=everything` on `e098438`, the branch's newest commit), and the Miri jobs still running in `39165405002` and
  `39167478125` — each superseded by it, kept as evidence; (3) RW 4.17's UNC half; RW 6.1's further strings; RW 6.2;
  (4) *[laptop]* the launch, only on the owner's word — the launch session merges `master` into `launch/2.0.0` first,
  then needs a green `everything` run on its final commit (`CHECKLIST-2.0.md` criterion 1).

### 2026-10-18 — on-demand routine run 1 (postscript 2): the localization guide's key-list claim corrected
- Session: `https://claude.ai/code/session_01FzcPzLxjK3s2xT743XXr51`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Base: `3f2ac0d`   Commits: this entry's commit
- Files and folders: M `docs/design/LOCALIZATION_PLUGIN_GUIDE.md` (§2's rules: the key space named — `delulu_diag::REGISTRY`
  and `CLI_STRINGS` — where it said `docs/for-agents.md` and a "Stage-9 reference" listed it; neither does),
  `docs/REMAINING_WORK.md` (correction row 1.14), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Verified: `grep` finds no catalog key in `docs/for-agents.md` and no Stage-9 reference; `docs/lang/en-US.md` §3.1 holds
  7 of 154 codes (so the guide says "a starter set"); `delulu explain E-DL0501` answers. Document gates green.
- Survey and doctor (after the last edit): in the commit message.
- Open / next: as the entry before — this commit's push run first.

### 2026-10-19 — on-demand routine run 2: RW 4.57 — a run's own node ends with the run, however it ends (D-V2-157)
- Session: `https://claude.ai/code/session_01VboA3N3sFFxH2ptACsW8r3`   Model: Claude Opus 5.5 (`claude-opus-5-5`; the effort is
  `.claude/settings.json`'s xhigh — no other model or effort shown; no fallback notice; no sous-chef)
- Routine: `trig_01HFG6usfzmDAPQigiDu9DLq`, its second run (started 2026-10-19 00:06 UTC, no `routine-fire-payload`)
- Branch: `claude/rw-4-57-run-node` (the runner witnesses), fast-forwarded into `master`   Pull request: none   Merged: n/a
- Base: `0c84447`
- Commits: `abd71c2` A run's own node ends with the run, however it ends (RW 4.57); `4b0adcb` The run's beater is a thread
  that never runs a program (RW 4.57); and this entry's commit
- Files and folders: A `docs/assistant-memory/a-fix-for-one-node-asks-every-node.md`; M `crates/delulu/src/broker_client.rs`
  (`RunNode`, `run_deadman_period`, a unit test), `crates/delulu/src/run_cmd.rs`, `crates/delulu/src/guest.rs`,
  `crates/delulu/tests/plugin_load_cli.rs` (four witnesses), `crates/delulu-runtime/src/actors.rs` (the beater classified),
  `CHANGELOG.md`, `HANDOFF.md` (§11.4 RW 4.57, §11.5 the CPU-load trap), `docs/CLOUD_ROUTINE.md` (how to load the CPUs),
  `docs/DEPLOYMENT.md` (§3, a run's own node), `docs/design/STAGE5_SPECIFICATION.md` (§3.2's sugar ends with its run),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-157), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.57
  partial; 4.64's stale status cell corrected — closed by `b3e17fc` since routine run 23), `docs/assistant-memory/MEMORY.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- **Start of run:** `git fetch` — `master` at `0c84447` (run 1's last commit), nothing newer; `origin/autosave/laptop`'s
  newest snapshot is still `b4a9e18`, recovered by run 1; `claude/wonderful-hamilton-bo8k53` and
  `claude/friendly-thompson-vz6m23` carry one commit each, kept and superseded as recorded on 2026-10-17; every other
  `claude/*` branch is in `master`. Survey `ok` (1521 nodes, 14843 edges), `doctor --check` all checks passed. `gh` is
  installed; CI read with `curl` and the GitHub MCP tools. Run 1's claims re-run: `locale_cli` 7/7, `locale_plugin_cli` 3/3,
  `sandbox_devices_cli` 14/14, `clean_checkout` 2/2, `delulu-broker` 204/204, `delulu-diag` 50/50 — all hold. `cargo deny
  … advisories` ok.
- **CI read:** `3f2ac0d` `39170518002` and `0c84447` `39170730116` — success. `launch/2.0.0`: `39169269103` (`jobs=everything`
  on `e098438`, its newest commit) — **success on all 18 jobs**; `39167478125` (`b8f9f16`) — success on all 18;
  `39165405002` (`9763787`) — 17 success, Windows red in the e-stop witness since fixed. Per-OS verdicts in `V2_LOG.md`.
- **RW 4.57, witnessed and closed for issue-then-run (D-V2-157):** re-run first on a scratch daemon — a budget-stopped or
  killed run left its root and its plugin's node `[live]`, and a cleanly ended run left its own root `[live]` (RW 4.51 had
  ended only the plugin's). `RunNode` revokes the run's own root at its end and arms a 10 s broker dead-man on it, beaten
  by the run, so a killed run's nodes are reaped. Witnesses red on the old code; mutants M455–M460 red; macOS and Windows
  read green before `master` moved. The lease half stays open (RW 4.57).
- Survey and doctor (after the last edit): in the commit message.
- Verified: the full suite twice — at `abd71c2` 2,325 passed and 1 failed (the thread-classification gate naming the new
  beater; fixed in `4b0adcb`), then over the fix and these records 2,326 passed, 0 failed, 16 ignored (exit 0, the tree
  unmoved); clippy 0; the four witnesses on `macos-latest` `39181220241` and `windows-latest` `39181218534` (8 passed each);
  the RW 4.57 tests 12 of 12 green with the machine loaded by four concurrent copies of their binary.
- Redo on the laptop: nothing (macOS and Windows read on runners).
- For the laptop's memory: `HANDOFF.md` §11.4 (RW 4.57 — a fix for one node asks every node the holder holds) and §11.5
  (the harness refuses shell busy loops); `docs/assistant-memory/a-fix-for-one-node-asks-every-node.md`.
- Open / next, in order: (1) this commit's push run; (2) RW 4.57's lease half — a lease run's minted nodes dead-manned
  with its root's beats; (3) merge `master` into `launch/2.0.0` again for D-V2-157 (its CHANGELOG lines into the 2.0.0
  section) and read `jobs=everything` on the result; (4) RW 4.17's UNC half, RW 6.1's further strings, RW 6.2; (5)
  *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 2 (continued): RW 4.57's lease half; LEASE-DROP-1 (RW 4.68) closed; `launch/2.0.0` kept current
- Session: `https://claude.ai/code/session_01VboA3N3sFFxH2ptACsW8r3`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `claude/rw-4-57-run-node` (the runner witnesses), fast-forwarded into `master`; and `launch/2.0.0`   Pull request: none
- Base: `d144882`
- Commits: `6f98ae5` A lease run's minted nodes are dead-manned as they are minted (RW 4.57); `86ad969` A lease run refuses
  a local plugin, compute or native grant by name (RW 4.68); and this entry's commit. On `launch/2.0.0`: `b4748fa`, the merge of
  `d144882` (release procedure step 1).
- Files and folders: M `crates/delulu/src/broker_client.rs` (`Reaper`; `RunNode::for_lease`; `attenuate` arms under a lease),
  `crates/delulu/src/run_cmd.rs` (the lease hold; three more lease refusals; DL1911 steps aside beside `--lease`),
  `crates/delulu/src/guest.rs` (the lease hold), `crates/delulu/src/cli.rs` (a false comment corrected),
  `crates/delulu/tests/plugin_load_cli.rs` (three lease witnesses), `crates/delulu/tests/satellite_demo.rs` (the generated
  lease-grant witness), `crates/delulu/tests/monitor_cli.rs` (the opt-out witness asks about the monitor's node), `CHANGELOG.md`, `HANDOFF.md` (§11.4 LEASE-DROP-1; §11.5 the dry-run trap), `docs/CLOUD_ROUTINE.md`
  (step 5: join a script's input with `&&`), `docs/DEPLOYMENT.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-157 (5): the
  lease half built), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.57 closed; 4.68 added and closed; 4.35's
  stale status cell corrected — closed since routine run 10), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed:
  nothing. On `launch/2.0.0` (`b4748fa`): the merge, its CHANGELOG section moved into `## [2.0.0]`, `docs/survey/*` rebuilt there.
- **RW 4.57 closed:** a `--lease` run's own node is the delegator's and is never ended by a run; what the run mints under
  it — its devices' nodes — is dead-manned as it is minted, so a killed lease run leaves none live and the lease lives on.
  Plain and sandboxed, witnessed; mutants M461–M467 red.
- **LEASE-DROP-1 (RW 4.68), found and closed:** `run --lease` dropped a local `--grant plugin=`, `compute=` or `exec.native`
  in silence and ran on (witnessed: exit 0); a plugin program then met DL0703 asking for the flag the operator had passed.
  Each is refused by name now; the witness is generated from `delulu toolchain --json`'s grant grammar; mutants M468–M471 red.
- **The container restarted at 01:20 UTC** and killed the third full suite mid-run (`egress_cli`); the tree, the commits and
  the scratchpad survived; the suite was run again over everything.
- Survey and doctor (after the last edit): in the commit message.
- Verified: suite 4 (all of this run's code and records) 2,329 passed, 1 failed — `monitor_cli`'s opt-out witness asked
  whether ANY dead-man was armed, and a lease run's device node now is; narrowed to the monitor's node, M472 red,
  `monitor_cli` 22/22; suite 5 over the final tree: 2,330 passed, 0 failed, 16 ignored (cargo's exit 0, the tree unmoved); clippy 0.
- CI read: `d144882`'s push runs CI `39183143824` and `ocsf` `39183143907` — success; `launch/2.0.0`'s release dry run
  `39183562777` on `b4748fa` — success; the lease-half runner reads `39184608216` (macOS) and `39184606157` (Windows) — 11
  passed each; RW 4.68's at `86ad969` — `39187175838` (macOS) and `39187173387` (Windows): `compute_cli` 18 and
  `satellite_demo` 5 passed each (Windows terminated only `vctip`).
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (LEASE-DROP-1 — a refusal list needs a generated witness over everything it
  could name) and §11.5 (a dry run's dummy input is a trap for the real run).
- Open / next, in order: (1) this commit's push run; (2) `launch/2.0.0`: read `39183559939` (`jobs=everything` on `b4748fa`;
  `miri-slow` takes hours), then merge `master` into it again for the lease half and RW 4.68 and dispatch `everything` on
  the result; (3) RW 4.17's UNC half, RW 6.1's further strings, RW 6.2; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 2 (postscript): `e200edb`'s push run read; `launch/2.0.0` at `3aafb77`
- Session: `https://claude.ai/code/session_01VboA3N3sFFxH2ptACsW8r3`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Base: `e200edb`   Commits: this entry's commit
- Files and folders: M `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17), `docs/CLOUD_ROUTINE.md` (step 5: a container restart
  kills a background suite without a word), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- CI read: `e200edb`'s push runs — CI `39189114719`, `ocsf` `39189114737` and `openshell` `39189114674` — **success** (CI's 14
  jobs; `miri-slow` and `heavy-gates` skipped, as on every push; `actors_pingpong`: arm64 MEASURED 2.82x, passed; Ubuntu and
  Windows NOT MEASURED, busy; macOS NOT MEASURED, 3 threads). `master` is green on this run's last code.
- `launch/2.0.0`: `3aafb77`, the merge of `e200edb` (release procedure step 1) — the CHANGELOG's changes merged into the
  2.0.0 section's dated entry by themselves; the map rebuilt there (1523 nodes, 14919 edges, 0 errors); `doctor --check` ok;
  `doctor_cli`, `repository_structure`, `evidence_claims`, `governance`, `distribution`, `book`, `core_invariance`,
  `release`, `plugin_load_cli` (24), `satellite_demo` (5) and `monitor_cli` (22) green on the launch tree. Dispatched on it:
  `jobs=everything` `39189498514` and the release dry run `39189501390` — unread at this entry. `39183559939` (`everything`
  on `b4748fa`): 16 of 18 success at 02:05 UTC, its two `miri-slow` jobs running; superseded by `39189498514`, kept as evidence.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, in order: (1) this commit's push run (records only); (2) `launch/2.0.0`: read `39189498514` and
  `39189501390` on `3aafb77` — the branch's newest commit; `CHECKLIST-2.0.md` criterion 1 needs `everything` green on the
  final launch commit, so a launch session merges `master` once more and reads it again; (3) RW 4.17's UNC half, RW 6.1's
  further strings, RW 6.2, the open containment rows (4.63, 4.66, 4.67); (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 3: MONITOR-CHAIN-1 — a long chain killed a healthy monitor's fleet; the monitor reads only what was appended, verified (D-V2-158)
- Session: `https://claude.ai/code/session_01XjDbEL9VYTEzeybn7ALWwn`   Model: Claude Opus 5.5 (`claude-opus-5-5`; the effort is
  `.claude/settings.json`'s xhigh — no other model or effort shown; no fallback notice; no sous-chef)
- Routine: `trig_01HFG6usfzmDAPQigiDu9DLq`, its third run (started 2026-10-19 05:05 UTC, no `routine-fire-payload`)
- Branch: `claude/monitor-chain-follower` (the runner witnesses), fast-forwarded into `master`   Pull request: none   Merged: n/a
- Base: `49f6c6f`
- Commits: `90d6ddb` A monitor reads only what was appended, verified (MONITOR-CHAIN-1); and this entry's commit
- Files and folders: A `docs/assistant-memory/a-latency-may-be-a-deadline.md`; M `crates/delulu-broker/src/audit.rs`
  (`Follower`, `Followed`, `read_day_from`, `complete_lines`, five tests), `crates/delulu/src/monitor_cmd.rs` (the watch
  reads through a follower), `crates/delulu/tests/monitor_cli.rs` (the witness and its chain grower), `CHANGELOG.md`,
  `HANDOFF.md` (§11.4 MONITOR-CHAIN-1, §11.5 a latency may be a deadline), `docs/DEPLOYMENT.md` (the monitor's poll),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-158), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.49 closed;
  4.53's monitor half noted), `docs/assistant-memory/MEMORY.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed:
  nothing.
- **Start of run:** `master` at `49f6c6f`, nothing newer; `origin/autosave/laptop`'s newest snapshot still `b4a9e18`
  (recovered by run 1); every `claude/*` branch in `master` but the two kept as superseded on 2026-10-17. Survey `ok` (1522
  nodes, 14894 edges), `doctor --check` all checks passed. `gh` is installed; CI read with `curl` and the GitHub MCP tools.
  Run 2's claims re-run: `monitor_cli` 22/22, `plugin_load_cli` 24/24, `satellite_demo` 5/5, `compute_cli` 18/18 — all hold.
- **CI read:** `49f6c6f` `39190850972` — success; `launch/2.0.0` at `3aafb77`: `jobs=everything` `39189498514` — success on
  all 18 jobs — and the release dry run `39189501390` — success; `39183559939` (`b4748fa`) — success on all 18.
- **Found and fixed (MONITOR-CHAIN-1, RW 4.49 closed):** `monitor watch` read the whole chain each poll and beats its
  dead-man between polls; at 250,000 records and the default 2 s, a healthy monitor was revoked by its own broker with every
  run under it (by hand, nothing misbehaving). `audit::Follower` reads only what was appended, whole lines only, each record
  linked to the one before and its hash its own; an unverified line is named and never judged.
- Survey and doctor (after the last edit): in the commit message.
- Verified: the witness red on the old code (a 101,251-record chain; the run's lease revoked by the dead-man), green after,
  4 of 4 under four concurrent copies; mutants M473–M479 and M473b red, control green; clippy 0; `check-other-os.sh`
  (Windows, macOS) clean; runner reads at `90d6ddb` — `monitor_cli` macOS `39205018972` (22 passed) and Windows
  `39205021062` (21 passed), `audit::tests` macOS `39205023031` (23) and Windows `39205025243` (22); the full suite 2,335
  passed, 0 failed, 16 ignored (exit 0, the tree unmoved).
- Redo on the laptop: nothing (macOS and Windows read on runners).
- For the laptop's memory: `HANDOFF.md` §11.4 (MONITOR-CHAIN-1) and §11.5 (a cost recorded as a latency may be a deadline);
  `docs/assistant-memory/a-latency-may-be-a-deadline.md`.
- Open / next, in order: (1) this commit's push run; (2) RW 4.53's writer half — `AuditLog::catch_up` re-reads and
  re-counts the whole chain under the append lock whenever another writer moved it (the daemon's e-stop revoke behind it),
  and `open` on every opening: this run's next slice; (3) merge `master` into `launch/2.0.0` (release procedure step 1) and
  read `jobs=everything` on the result; (4) RW 4.17's UNC half, RW 6.1's further strings, RW 6.2, RW 4.63, 4.66, 4.67;
  (5) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 3 (continued): RW 4.53's writer half — a writer catches up by reading only what was appended (D-V2-159)
- Session: `https://claude.ai/code/session_01XjDbEL9VYTEzeybn7ALWwn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `claude/monitor-chain-follower` (the runner witnesses), fast-forwarded into `master`   Pull request: none   Merged: n/a
- Base: `200df1c`
- Commits: `7ae1524` A writer catches up by reading only what was appended (RW 4.53); and this entry's commit
- Files and folders: M `crates/delulu-broker/src/audit.rs` (`AuditLog::read_appended`, `verified_link` shared with the
  follower, three tests), `CHANGELOG.md`, `HANDOFF.md` (§11.4 MONITOR-CHAIN-1: the writing side), `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-159), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.53 partial), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted, renamed: nothing.
- **Fixed (RW 4.53, partial):** every writer's catch-up re-read and re-counted the whole chain under the append lock once
  another writer had written; a recorded daemon request beside another writer on 100,000 records took median 3,817 ms (max
  4,479), now 14 ms (max 21). `AuditLog::open` still reads the whole chain on every opening.
- Survey and doctor (after the last edit): in the commit message.
- Verified: M480 (the old behaviour) red in the witness; M481–M484 red; M485 survived the first round and is red in a third
  witness written for it; control green; clippy 0; `check-other-os.sh` (Windows, macOS) clean; runner reads at `7ae1524` —
  `audit::tests` macOS `39206620758` (26 passed), Windows `39206622817` (25); the full suite 2,338 passed, 0 failed, 16
  ignored (exit 0, the tree unmoved). CI read: `200df1c`'s push runs — CI `39206009719` and `ocsf` `39206009590` — success.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (MONITOR-CHAIN-1, the writing side).
- Open / next, in order: (1) this commit's push run; (2) merge `master` into `launch/2.0.0` (release procedure step 1: its
  CHANGELOG lines into the 2.0.0 section) and read `jobs=everything` on the result; (3) RW 4.53's last part — `AuditLog::open`
  reads the whole chain on every opening (each sandboxed run's host, each CLI command that records); its fast path needs the
  anchor to name the newest file and its length; (4) RW 4.17's UNC half, RW 6.1's further strings, RW 6.2, RW 4.63, 4.66,
  4.67; (5) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 3 (last): RW 4.53's last part — an opening reads one line when the files measure as the anchor says (D-V2-160); `launch/2.0.0` at `d2814e8`
- Session: `https://claude.ai/code/session_01XjDbEL9VYTEzeybn7ALWwn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `claude/monitor-chain-follower` (the runner witnesses), fast-forwarded into `master`; and `launch/2.0.0`   Pull request: none
- Base: `7f1a4a1`
- Commits: `5dea22e` An opening reads one line when the files measure as the anchor says (RW 4.53); and this entry's commit.
  On `launch/2.0.0`: `d2814e8`, the merge of `7f1a4a1` (release procedure step 1).
- Files and folders: M `crates/delulu-broker/src/audit.rs` (`AnchorAt`, `recover_fast`, `last_line_hash`, the writer keeps
  the note; four tests), `docs/MATHEMATICS.md` (the anchor's note), `CHANGELOG.md`, `HANDOFF.md` (§11.4 MONITOR-CHAIN-1: the
  opening), `docs/CLOUD_ROUTINE.md` (the cleanup recipe spares proc-macro libraries; remove the launch worktree's target
  after its push), `scripts/suite.sh` (the same recipe in its refusal), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-160),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17), `docs/REMAINING_WORK.md` (4.53),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing. On `launch/2.0.0` (`d2814e8`): the merge, this run's
  CHANGELOG section moved into `## [2.0.0]`, `docs/survey/*` rebuilt there.
- **Fixed (RW 4.53's last part):** every `AuditLog::open` read every day file twice (180–303 ms at 100,000 records, release);
  the anchor now notes where the chain stood, and an opening whose files still measure so reads one line (0.03–0.1 ms; the
  first opening of an older log re-affirms the anchor with its note, 247 ms). An older record edited in place to the same
  length is `verify`'s to find, as it always was — named in D-V2-160.
- **`launch/2.0.0`:** `d2814e8` merges `7f1a4a1` (MONITOR-CHAIN-1, RW 4.53's first two parts); the gates green on the launch
  tree (Survey 1524 nodes, 0 errors; `doctor --check`; book, core_invariance, distribution, doctor_cli, evidence_claims,
  governance, repository_structure, release, monitor_cli 23; `audit::tests` 26). The release dry run `39208470155` —
  success; `jobs=everything` `39208467616` — 16 of 18 success at 06:33 UTC, `miri-slow (delulu-check)` and `(delulu-broker)`
  running. This run's third slice (`5dea22e`) is not on it yet.
- Survey and doctor (after the last edit): in the commit message.
- Verified: mutants M486–M493 red (M489 after the middle-line case was added for its survivor; M489b also in
  `audit_truncation`), control green; clippy 0; `check-other-os.sh` (Windows, macOS) clean; runner reads at `5dea22e` — the
  whole `delulu-broker` lib with `audit_truncation` and `audit_wiring`: macOS `39209490393` (224 passed), Windows
  `39209493185` (222); the full suite 2,342 passed, 0 failed, 16 ignored (exit 0, the tree unmoved). CI read: `7f1a4a1`'s push
  runs — CI `39208000005` and `ocsf` `39207999996` — success.
- **Loop engineering:** the cleanup recipe `scripts/suite.sh` prints (and `CLOUD_ROUTINE.md` repeats) deleted the proc-macro
  libraries with the old test executables — they are executable too — and the next suite rebuilt most of the workspace;
  both now spare `*.so`/`*.dylib`/`*.dll`. And the launch worktree's own target (about 3 GB) made the next suite refuse to
  start under 10 GB free: step 4 now removes it after the launch push. **This run's own slip, recorded:** a broker started
  by hand for the 250,000-record measurement outlived it — its state directory was removed before it was stopped, so `broker
  stop` could not find it; found by `ps` and stopped by PID about fifty minutes after it started (§11.5's directory-guard
  trap, by hand).
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (MONITOR-CHAIN-1, its writing side and the opening), §11.5 (a cost recorded
  as a latency may be a deadline); `docs/assistant-memory/a-latency-may-be-a-deadline.md`.
- Open / next, in order: (1) this commit's push run; (2) `launch/2.0.0`: read `39208467616`'s two `miri-slow` jobs, then merge
  `master` into it again for D-V2-160 and read `jobs=everything` on the result; (3) the same class elsewhere — the monitor's
  `List` each poll returns the whole grant tree, which grows with every run's node (revoked nodes stay listed): measure it
  against a tree of tens of thousands of nodes before grading it (§11.5, a latency may be a deadline); (4) RW 4.53's
  remainder (the serial daemon behind other writers' appends on a short chain — routine run 16's measurement, not re-run);
  RW 4.17's UNC half, RW 6.1's further strings, RW 6.2, RW 4.63, 4.66, 4.67; (5) *[laptop]* the launch, only on the owner's
  word.

### 2026-10-19 — on-demand routine run 3 (postscript): `4c8e44a`'s push runs read; `launch/2.0.0` at `6c01264`
- Session: `https://claude.ai/code/session_01XjDbEL9VYTEzeybn7ALWwn`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`; and `launch/2.0.0`   Base: `4c8e44a`   Commits: this entry's commit; on `launch/2.0.0` `6c01264`, the merge
  of `4c8e44a`.
- Files and folders: M `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- CI read: `4c8e44a`'s push runs — CI `39210944095` and `ocsf` `39210944303` — **success**; `master` is green on every slice
  of this run. `launch/2.0.0` at `6c01264`: the release dry run `39212180218` — success; `jobs=everything` `39212177717` —
  running at this entry (its `miri-slow` jobs take hours).
- Measured: the monitor's per-poll `List` — at 3,002 nodes no material cost to other daemon requests; at 30,003 nodes a
  recorded request's max went from 17 ms to 371 ms beside one monitor polling every 25 ms (`V2_LOG.md`) — **RW 4.69 added,
  open**. (Added in a second commit: `docs/REMAINING_WORK.md` too.)
- Survey and doctor (after the last edit): in the commit message.
- Open / next, in order: (1) this commit's push run (records only); (2) `launch/2.0.0`: read `39212177717` (`jobs=everything`
  on `6c01264`, the branch's newest commit); `CHECKLIST-2.0.md` criterion 1 needs `everything` green on the final launch
  commit, so a launch session merges `master` once more and reads it again; (3) RW 4.69 — the monitor asks only for the live
  subtree under its node (measured first, red before); RW 4.53's remainder (routine run 16's short-chain queue measurement); RW 4.17's UNC half, RW 6.1's further strings,
  RW 6.2, RW 4.63, 4.66, 4.67; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — the laptop syncs the on-demand routine's three runs; the owner: the tests go to GitHub when the laptop cannot run them
- Session: `https://claude.ai/code/session_01EL39cBh8onpAXvyY8r6d6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Pull request: none   Merged: n/a   Base: `71cae40`
- Commits: this entry's commit.
- Synced: the laptop fast-forwarded `master` `640a7d5` → `71cae40` (26 commits — the on-demand routine's runs 1–3) and
  `launch/2.0.0` `0a3ff6e` → `6c01264`. The laptop's uncommitted RW 6.1 draft was byte-identical to the autosave snapshot
  `b4a9e18`, which run 1 recovered, corrected and committed as `a3d7678`; the draft is kept in `git stash` (never applied).
  CI read at the sync: `master`'s push runs on `7aa0744` (`39213390618`) and `71cae40` (`39213772400`) — **success**;
  `launch/2.0.0`'s release dry run `39212180218` — success; its `jobs=everything` runs `39208467616` (`d2814e8`) and
  `39212177717` (`6c01264`) — still running (`miri-slow`).
- **The owner, 2026-10-19:** *"Whenever tests cannot be run locally on lap or run cannot be completed from lap for any
  reason, automatically run them using GitHub Actions runners or any other alternatives instead. Ensure the tests complete
  successfully and review the results."* Recorded in `HANDOFF.md` §11.1, `AGENTS.md` step 3 and the memory
  (`docs/assistant-memory/tests-on-github-when-laptop-cannot.md`, and the laptop's).
- The autosave task (`DeluluLang-autosave`) had run out its 7 hours on 2026-10-18; re-installed for 7 hours.
- The laptop's memory took the three files the routine added (`a-fix-for-one-node-asks-every-node.md`,
  `a-latency-may-be-a-deadline.md`, `autosave-snapshot-is-a-draft.md`).
- Files and folders: M `AGENTS.md`, `HANDOFF.md`, `docs/assistant-memory/MEMORY.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`; A `docs/assistant-memory/tests-on-github-when-laptop-cannot.md`. Deleted, renamed: nothing.
- Survey and doctor (session start, on `71cae40`): `survey check` ok (1523 nodes, 14930 edges); `doctor --check` ok (29
  checks). After the last edit: in the commit message.
- Open / next, in order: (1) this commit's push run; (2) RW 4.69 — the monitor asks only for the live subtree under its
  node; (3) `launch/2.0.0`'s two `everything` runs, once `miri-slow` ends; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — RW 4.69: a tree too large for one listing killed a healthy monitor's fleet (MONITOR-FRAME-1, D-V2-161) — made on the laptop
- Session: `https://claude.ai/code/session_01EL39cBh8onpAXvyY8r6d6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `laptop/rw-4-69`, then `master` (fast-forward) once its CI is read green   Base: `0a9afbf`
- Commits: this entry's commit.
- Files and folders: M `crates/delulu-broker/src/tree.rs` (`Broker::run_under`), `crates/delulu/src/broker_ipc.rs`
  (`RunsUnder`/`Runs`, `MAX_RUNS_ASKED`, `FrameTooLarge`, `MAX_FRAME` public), `crates/delulu/src/brokerd.rs` (the handler, the
  oversize answer in words, a unit test), `crates/delulu/src/monitor_cmd.rs` (a poll asks `RunsUnder`; `runs_under` and its
  unit test moved to `tree.rs`), `crates/delulu/tests/monitor_cli.rs` (the witness), `CHANGELOG.md`, `HANDOFF.md` (§11.4),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-161), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.69 closed,
  4.70 added), `docs/assistant-memory/a-latency-may-be-a-deadline.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted,
  renamed: nothing.
- **Found and fixed:** past one 16 MiB frame the daemon could not send the monitor's whole-tree listing; the monitor stopped
  (`broker unreachable`) and its dead-man revoked its node and every run under it — 550 nodes of 30,000-byte descriptions
  outside its subtree, by hand on Windows. Witness red on the old code; four mutants red (`V2_LOG.md`).
- **The full suite did not run on the laptop** (the owner, 2026-10-19: the tests go to GitHub when the laptop cannot run
  them — its 2026-10-17 suite had been stopped for low memory): it runs as this branch's pull-request CI, read to its end
  before `master` moves. Locally green: the ten test targets that read the tree, the `delulu` and `delulu-broker` unit tests,
  clippy 0, fmt clean.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, in order: (1) this branch's CI, then `master`; (2) RW 4.70 — `grants list` past one frame; (3) `launch/2.0.0`:
  merge `master`, dispatch `everything` and the release dry run; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — RW 4.63: a watch held outside the daemon lets its dead-man fire; M346 red (made on the laptop)
- Session: `https://claude.ai/code/session_01EL39cBh8onpAXvyY8r6d6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `laptop/rw-4-69` (pull request #1), then `master` once its CI is read green   Base: `ea7d42b`
- Commits: this entry's commit.
- Files and folders: M `crates/delulu/tests/monitor_cli.rs`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.63
  closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- A witness only — no product code changed: one 32 MiB record written through the chain's writer holds the monitor's watch
  in its own read for three dead-man periods; its dead-man fires. M346 (the beater beats whatever the watch does), which
  survived since routine run 22, is red.
- The full suite runs on GitHub as the pull request's CI (the owner's rule of 2026-10-19). Locally: `monitor_cli` 23 passed;
  clippy 0; fmt clean.
- Survey and doctor (after the last edit): in the commit message.
- Open / next: as the entry before.

### 2026-10-19 — RW 4.70: the whole-tree reads come in pages; TREE-STATE-1; the pull request's suite read red and fixed (D-V2-162) — made on the laptop
- Session: `https://claude.ai/code/session_01EL39cBh8onpAXvyY8r6d6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `laptop/rw-4-69` (pull request #1), then `master` once its CI is read green   Base: `3203d3f`
- Commits: this entry's commit.
- CI read: pull request #1's `39224116908` (`ea7d42b`) and `39224995083` (`3203d3f`) — **failure**, each suite red on one
  test, `message_spacing` (a continuation flattened by my Python edit of `brokerd.rs`; fixed here) — Windows 2,279 passed,
  Ubuntu 2,344, macOS 2,318, arm64 2,344, 1 failed each; every other job success; `monitor_cli` green on all four.
- Files and folders: M `crates/delulu-broker/src/tree.rs` (the tree render: one index, a stack, the inherited state; a unit
  test), `crates/delulu/src/broker_ipc.rs` (`ListPage`/`ListedPage`, `LIST_PAGE_BYTES`, `encoded_len`),
  `crates/delulu/src/brokerd.rs` (the handler, `list_all`, the DL1401 text on one line, a unit test),
  `crates/delulu/src/cli.rs` (`grants list`/`tree` and the atlas overlay page; `render_tree`; `grants_answer`),
  `crates/delulu/tests/grants_cli.rs` (the witness), `crates/delulu/tests/monitor_cli.rs` (RW 4.69's state measured from the
  paged listing), `CHANGELOG.md`, `HANDOFF.md` (§11.4, §11.5), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-162),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.70 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, in order: (1) the pull request's CI on this commit, then `master` fast-forwards; (2) `launch/2.0.0`: merge
  `master`, dispatch `everything` and the release dry run; (3) RW 4.66, 4.67, 6.1's further strings, 6.2; (4) *[laptop]* the
  launch, only on the owner's word.

### 2026-10-19 — RW 3.4: loops on the WASM engine; its memory grows; WASM-HYGIENE-1 (D-V2-163) — made on the laptop
- Session: `https://claude.ai/code/session_01EL39cBh8onpAXvyY8r6d6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `laptop/wasm-loops` (a pull request into `master`), then `master` once its CI is read green   Base: `98edf2f`
- Commits: this entry's commit.
- CI read: pull request #1's `39229088627` on `98edf2f` — **success on every job** (Windows 2,283, Ubuntu 2,348, macOS
  2,322, arm64 2,348 passed, 0 failed); `master` fast-forwarded `0a9afbf` → `98edf2f` and pushed; the pull request merged.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (loops, assignment, `__alloc`, `__str_len`, the lexical
  model), `crates/delulu-wasm/src/host.rs` (the host's allocator grows the memory), `crates/delulu-wasm/src/gen.rs` (the
  loop generator), `crates/delulu-wasm/tests/differential.rs` (loop programs in the gate), `crates/delulu/tests/budget_cli.rs`
  (two WASM budget witnesses), `crates/delulu/tests/wasm_cli.rs` (the DL1201 example is a `for`), `docs/book/
  THE_DELULULANG_BOOK.md`, `measurements/study-c/HOT_PATH_TABLE.md` (a dated note; the table not re-measured),
  `CHANGELOG.md`, `HANDOFF.md` (§11.4), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-163), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/REMAINING_WORK.md` (3.4 widened, 3.7 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A
  `crates/delulu-wasm/tests/loop_parity.rs`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, in order: (1) this branch's CI, then `master`; (2) re-run Study-C in release (all three lanes) — WSL has
  gcc; (3) `launch/2.0.0`: merge `master`, dispatch `everything` and the release dry run; (4) the WASM fragment's next
  pieces — `str(Bool)` without the foreign interface, `List`; (5) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — RW 3.4 (postscript): `str(Bool)` in any WASM program (made on the laptop)
- Session: `https://claude.ai/code/session_01EL39cBh8onpAXvyY8r6d6q`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `laptop/wasm-loops` (pull request #2)   Base: `c3445be`   Commits: this entry's commit.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs`, `crates/delulu-wasm/tests/loop_parity.rs`, `CHANGELOG.md`,
  `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Open / next: as the entry before.

### 2026-10-19 — on-demand routine run 4: the laptop's RW 3.4 verified and merged (pull request #2); WASM-HYGIENE-2 (RW 3.8)
- Session: `https://claude.ai/code/session_016kec1tYrJcvMuuS75iygGL`   Model: Claude Opus 5.5 (the system prompt names
  `claude-opus-5-5`; no fallback notice; `.claude/settings.json` sets xhigh — the session is not shown its own effort; no
  sous-chef)
- Branch: `master`; the code's runner reads on `claude/wasm-hygiene-2`   Pull request: #2 merged by fast-forward   Base: `98edf2f`
- Commits: `c3445be` and `4d11575` (the laptop's, pull request #2 — `master` fast-forwarded to them); this entry's commit.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`compile_block_inner`: a frame of function values per block),
  `crates/delulu-wasm/src/gen.rs` (the loop generator binds, calls and shadows a function value),
  `crates/delulu-wasm/tests/loop_parity.rs` (the witness; the random test counts shadowing programs and prints its readings),
  `CHANGELOG.md`, `HANDOFF.md` (§11.4 WASM-HYGIENE-2), `docs/assistant-memory/MEMORY.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/REMAINING_WORK.md` (3.8 added, closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A
  `docs/assistant-memory/two-name-tables-one-scope.md`. Deleted, renamed: nothing.
- Start of session (on `98edf2f`): `survey check` ok (1524 nodes, 14985 edges); `doctor --check` ok (26 checks in this VM).
  `gh` is at `/usr/local/bin/gh` (not used; CI read through the REST API and the GitHub MCP tools).
- Verified (pull request #2): its CI `39233385903` success on every job; in the VM `delulu-wasm`'s suite, `wasm_cli` 10,
  `budget_cli` 10 and the release differential (50,000 checked, 46,855 agreed, 3,145 both faulted, 0 mismatches — the
  laptop's figures). The autosave snapshot `bbb8ea3` is the draft of `c3445be`, superseded by the two commits (checked file
  by file). `launch/2.0.0`'s `everything` runs `39208467616` and `39212177717` — success.
- **Fixed (WASM-HYGIENE-2, RW 3.8):** a `let f = fn…` inside an `if`, `else` or `while` body replaced the outer `f` for the
  rest of the function on `--engine wasm` (`2 / 1` on the interpreter, `2 / 2` on WASM — the `if` form on `98edf2f` too).
  Witness red on the old backend; the generator now writes the shape (149 of 400 sampled programs shadow `h`), red on the
  old backend too; differential with it 50,000 checked, 0 mismatches; M510–M512 red, control green (`V2_LOG.md`).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (WASM-HYGIENE-2); `docs/assistant-memory/two-name-tables-one-scope.md`.
- Open / next, in order: (1) this commit's push run and its runner reads on `claude/wasm-hygiene-2`; (2) `launch/2.0.0`: merge
  `master` (RW 4.69, 4.63, 4.70, RW 3.4, 3.8), dispatch `everything` and the release dry run; (3) re-run Study-C in release
  (all three lanes) before any row of `HOT_PATH_TABLE.md` changes; (4) the WASM fragment's next pieces — `for` and `List`;
  RW 4.66, 4.67, 6.1's further strings, 6.2; (5) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 4 (continued): Study C re-measured — the harness checks each lane's answer and times in microseconds; `launch/2.0.0` at `9e2eed8`
- Session: `https://claude.ai/code/session_016kec1tYrJcvMuuS75iygGL`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`; and `launch/2.0.0`   Base: `79f83d6`   Commits: this entry's commit; on `launch/2.0.0` `9e2eed8`, the
  merge of `79f83d6` (release procedure step 1).
- Files and folders: M `crates/delulu-measure/src/study_c.rs` (`checked`, `same_answer`, microseconds, `utc_minute`,
  `machine`, the report's machine table, `wasm ÷ C`, five unit tests), `measurements/study-c/REPORT.md` and `results.json`
  (regenerated, 2026-10-19), `measurements/study-c/HOT_PATH_TABLE.md`, `measurements/METHODOLOGY.md` (§3.2, §3.3),
  `README.md`, `docs/QUESTIONS.md`, `docs/for-agents.md`, `docs/REPOSITORY_STRUCTURE.md` (§5.9, `runs/`), `CHANGELOG.md`,
  `HANDOFF.md` (§11.5), `docs/assistant-memory/MEMORY.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (3.4),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `measurements/study-c/runs/2026-07-20/REPORT.md` and `results.json` (the v1.0
  run, copied whole before the harness overwrote it), `docs/assistant-memory/a-benchmark-checks-its-answer.md`. Deleted,
  renamed: nothing. On `launch/2.0.0` (`9e2eed8`): the merge; `master`'s three Unreleased CHANGELOG sections moved into
  `## [2.0.0]`; `HANDOFF.md` §11.1 both bullets kept; `docs/survey/*` rebuilt there.
- **Fixed (Study C):** a lane was timed whatever it printed — now only with the interpreter's answer; times were whole
  milliseconds, which read a Linux C process as 0 or 1 and dropped a benchmark from every ratio in silence — now
  microseconds, the left-out named; the record names its date and machine (`study-c/2`). Mutants M513–M518 red, control green.
- **Measured (2026-10-19 14:14 UTC, release `79f83d6`, this VM, nothing else running):** WASM runs 4 of 6 at **6.7×–11.7×
  C** whole-process (geo-mean 9.3×); the interpreter **7.5×–311.4× C** (2.0×–60.5× in the v1.0 run, where C read 5–6 ms;
  the interpreter's own times are unchanged within spread). Criterion 1 still not met. Every quoting document updated.
- **`launch/2.0.0`:** `9e2eed8` merges `master` `6c01264..79f83d6` (RW 4.69, 4.63, 4.70, 3.4, 3.8); on its tree survey 1527
  nodes, 0 errors; `doctor --check` ok; book, core_invariance, distribution, doctor_cli, evidence_claims, governance,
  release, repository_structure and `loop_parity` green. Dispatched at it: `ci.yml` `jobs=everything` and the release dry
  run — results in the next entry or the next run's.
- CI read: `4d11575`'s push run `39261866776` — **success**; `79f83d6`'s runner reads `39263152681` (macOS) and
  `39263157384` (Windows) — success, `loop_parity` 7 passed on each; its push run `39264737843` — running at this entry.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing (a laptop re-run of Study C would be a second machine's figures, welcome but not owed).
- For the laptop's memory: `HANDOFF.md` §11.5 (a benchmark checks its answer); `docs/assistant-memory/a-benchmark-checks-its-answer.md`.
- Open / next, in order: (1) the push runs of `79f83d6` and this commit; `launch/2.0.0`'s `everything` and release runs on
  `9e2eed8`; (2) Study C's start-up control (an empty `main` per lane) so a ratio can be read as compute; (3) the WASM
  fragment's next pieces — `for`, `List`, `split`, `map` (the last two kernels); RW 4.66, 4.67, 6.1's further strings, 6.2;
  (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 4 (continued): `master` red on `lints` once, fixed (`45ca06d`); Study C measures start-up
- Session: `https://claude.ai/code/session_016kec1tYrJcvMuuS75iygGL`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Base: `81da0ae`   Commits: `45ca06d` Study C's time_it witness says its range with contains (clippy,
  master red on lints); and this entry's commit.
- **`master` was red:** `81da0ae`'s push run `39268713452` — `lints` failure (clippy `manual_range_contains` in a test written
  after this run's last clippy; its message said "clippy 0"). Fixed in `45ca06d`, a one-line commit made in a worktree at
  `81da0ae` and linted there; its push run `39269260235` is read in the next entry (or by the next run).
- Files and folders (this entry's commit): M `crates/delulu-measure/src/study_c.rs` (the start-up control:
  `startup_program`, `take_control`, the report's start-up table and the table above start-up, `startup` in the record; two
  unit tests), `measurements/study-c/REPORT.md` and `results.json` (regenerated 14:26 UTC, with the control),
  `measurements/study-c/HOT_PATH_TABLE.md`, `measurements/METHODOLOGY.md`, `README.md`, `docs/QUESTIONS.md`,
  `docs/for-agents.md`, `CHANGELOG.md`, `HANDOFF.md` (§11.5), `docs/assistant-memory/a-benchmark-checks-its-answer.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (3.4), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted,
  renamed: nothing.
- **Measured (the published record, 14:26 UTC):** start-up C 0.92 ms, interpreter 4.70, WASM 7.01; C's work above its floor
  0.05–0.58 ms (no compute ratio offered), WASM's 1.30–4.13 ms; `wasm ÷ C` 6.5×–11.5× (geo-mean 8.8×), `interp ÷ C`
  8.9×–359.0×. The 14:14 run's ratios differ by up to a quarter; the table says so. Every quoting document updated to this run.
- Verified: `study_c::tests` 7; M519–M521 red, control green; clippy (`delulu-measure`) 0 after the last edit.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 (the slip: a commit's lint claim is about the commit).
- Open / next, in order: (1) `45ca06d`'s and this commit's push runs; `launch/2.0.0`'s `everything` run `39266597313` on
  `9e2eed8` (the release dry run `39266602442` there — success); then merge `master` into it again (this run's Study C and
  `45ca06d`); (2) the WASM fragment's next pieces — `for`, `List`, `split`, `map` (the last two kernels), and kernels long
  enough for C's work to be measured above its start-up; RW 4.66, 4.67, 6.1's further strings, 6.2; (3) *[laptop]* the
  launch, only on the owner's word.

### 2026-10-19 — on-demand routine run 4 (continued): loop engineering — a Rust commit needs the lint it claims (`scripts/clippy.sh`, the pre-commit hook)
- Session: `https://claude.ai/code/session_016kec1tYrJcvMuuS75iygGL`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`   Base: `0e4e9a2`   Commits: this entry's commit.
- **Why:** this run's `81da0ae` said "clippy 0" and went red on `lints` — clippy had run, then a test was written (fixed in
  `45ca06d`). "Lint after the LAST edit" was already written down twice (routine runs 12 and 25); routine run 15's lesson is
  that a rule a run must remember is one it forgets.
- **What:** `scripts/clippy.sh` runs the workspace clippy CI's `lints` job runs (`--workspace --all-targets -- -D warnings`,
  extra cargo arguments such as `-j 4` passed through) and, only when it exits 0 with the Rust inputs unchanged meanwhile,
  stamps one hash of them in the git directory (`git rev-parse --git-path clippy-stamp`, so each worktree has its own).
  `scripts/hooks/rust-inputs-hash worktree|index` computes that hash from git alone — blob ids of every `*.rs`, manifest,
  `Cargo.lock`, `clippy.toml` and `rust-toolchain.toml`; untracked-but-not-ignored files count; `.claude/` is excluded by name
  (the laptop's second copy). `scripts/hooks/pre-commit` now refuses a commit that STAGES any such input unless the index
  hashes to the stamp — no cargo in the hook, so a commit made while a suite holds the build lock still goes through when
  the lint ran before it. A records-only commit is not checked.
- **Falsified both ways** (a throwaway worktree, the hook called directly): no stamp → refused; after `clippy.sh` → its check
  passes; an edit staged after the lint → refused; an edit left unstaged after the lint → passes (the commit holds what was
  linted); a Markdown-only change → not checked. A nested git worktree's files are not listed by `git ls-files -o`
  (measured in a scratch repository); a plain copy's are, hence the `.claude/` exclusion.
- **For every session, laptop included:** run `scripts/clippy.sh -j 4` after the last `.rs` edit and before a commit that
  holds one — merges too (the launch branch's merge of `master` brings Rust in). AGENTS.md step 3, `CLOUD_ROUTINE.md` steps 4
  and 5, `HANDOFF.md` (§7's commands, §11.5), `CONTRIBUTING.md` §6 and `REPOSITORY_STRUCTURE.md` say so.
- Files and folders: M `scripts/hooks/pre-commit`, `AGENTS.md`, `CONTRIBUTING.md`, `HANDOFF.md`, `docs/CLOUD_ROUTINE.md`,
  `docs/REPOSITORY_STRUCTURE.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `scripts/clippy.sh`,
  `scripts/hooks/rust-inputs-hash`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: the first Rust commit there meets the new check — `scripts/clippy.sh -j 4` (Git-Bash) before it; if
  the hook refuses with everything staged and clippy clean, an untracked `.rs` file in the tree is the likely cause.
- Open / next: as the entry before.

### 2026-10-19 — on-demand routine run 4 (continued): `master` red on `formal` (Lean's download host), fixed (`205f0f4`); WASM-OOM-1 (RW 3.9); `launch/2.0.0` at `ece2454`
- Session: `https://claude.ai/code/session_016kec1tYrJcvMuuS75iygGL`   Model: Claude Opus 5.5 (no fallback notice; one Sonnet
  5.5 sous-chef running — the WASM red-team pass, below)
- Branch: `master`; and `launch/2.0.0`   Base: `6c26aed`   Commits: `205f0f4` formal installs Lean from its GitHub release,
  pinned by SHA-256; this entry's commit. On `launch/2.0.0`: `ece2454`, the merge of `6c26aed`.
- **`master` was red:** `0e4e9a2` (`39270059515`) and `6c26aed` (`39271301242`) — `formal` only, elan's download from
  `releases.lean-lang.org` refused by TLS before any proof ran. `205f0f4` fetches the same tarball from Lean's GitHub
  release, SHA-256-pinned, verified by running the step's script in the VM; its push run is read in the next entry or by
  the next run. `81da0ae`'s `39268713452` (`lints`, fixed by `45ca06d`) and `45ca06d`'s `39269260235` — success.
- **Fixed (WASM-OOM-1, RW 3.9):** an allocation the WASM memory could not give (past WebAssembly's 4 GiB) was reported as
  `DL0901: integer overflow`; now DL0903, naming the exhaustion. Witness red on the old `__alloc`; M522–M524 red.
- Files and folders (this entry's commit): M `crates/delulu-wasm/src/codegen.rs` (`__alloc`'s failure branches; the
  `alloc_tests` module), `crates/delulu-wasm/src/host.rs` (DL0903's text), `CHANGELOG.md`, `HANDOFF.md` (§11.4),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17), `docs/REMAINING_WORK.md` (3.9),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. In `205f0f4`: M `.github/workflows/ci.yml`, `docs/survey/survey.json`. Deleted,
  renamed: nothing.
- **`launch/2.0.0`:** `ece2454` merges `master` `79f83d6..6c26aed` (Study C re-measured, `45ca06d`, the lint stamp); on its
  tree the map 1532 nodes, 0 errors; `doctor --check` ok; `scripts/clippy.sh` clean and stamped (the first commit the hook's
  lint check passed for real); the document gates and `delulu-measure` green. `9e2eed8`'s release dry run `39266602442` —
  success; its `everything` run `39266597313` and `ece2454`'s (`39272359850`, with the release dry run `39272366257`) —
  running at this entry. `205f0f4` (the `formal` fix) reaches the launch branch at its next merge.
- Verified: as in `V2_LOG.md` (the step's script in the VM; `delulu-wasm` 129, `wasm_cli` 10, `budget_cli` 10, the
  differential 0 mismatches, clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (WASM-OOM-1: a new user of a shared signal inherits its meaning).
- Open / next, in order: (1) `205f0f4`'s and this commit's push runs — `formal` green is the witness of the fix; the
  launch runs on `ece2454`; then merge `master` into `launch/2.0.0` once more (for `205f0f4` above all); (2) the red-team
  pass's findings on the WASM fragment (`V2_LOG.md`, next entry); (3) the WASM fragment's next pieces — `for`, `List`,
  `split`, `map`; RW 4.66, 4.67, 6.1's further strings, 6.2; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — the laptop syncs on-demand routine run 4; the routine's effort pinned by an environment variable; `launch/2.0.0` at `c3b33f3`
- Session: `https://claude.ai/code/session_01WMGSP75D6H9Xw9SgL3ZD5j`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `master` (the effort commit); `launch/2.0.0` (`c3b33f3`)   Base: `8fc42f8`
- **Synced:** `master` fast-forwarded `98edf2f..8fc42f8` (run 4's nine commits, pull request #2's two among them). Every
  `origin/claude/*` branch's tip is in `master`. `origin/autosave/laptop`'s newest snapshot, `bbb8ea3` (09:39 UTC), predates
  `4d11575` (09:52 UTC), so there was nothing to recover. On `8fc42f8`: `survey check` ok (1531 nodes, 15097 edges);
  `doctor --check` ok (29 checks).
- **CI read (run 4's open items):** `205f0f4`'s push run `39274259721` and `8fc42f8`'s `39275805346` succeeded on every job,
  `formal` included, which witnesses the Lean fix. `launch/2.0.0`'s `everything` run `39272359850` on `ece2454` succeeded on
  every job (`heavy-gates` and the three `miri-slow` jobs included), and so did its release dry run `39272366257` (four
  packages).
- **What run 4 left unfinished** (its run log, read with the routine API; the usage limit stopped it at 15:12 UTC). Two
  WASM fixes were built and their mutants read red in the VM, but never committed: **negating `Int::MIN`** (the backend's
  `x * -1` wraps where the interpreter faults DL0901) and **`&&`/`||` evaluating both sides** (the interpreter short-circuits).
  Its Sonnet 5.5 red-team pass on the WASM fragment was cut off too, and its notes stayed in the VM. The log names
  some of what it found: `rand.int(lo, hi)` with `hi - lo` past `Int`'s range **panics the interpreter** (`prim.rs`, "attempt
  to subtract with overflow"); a refused `fs.read_text` reported DL0903 on WASM against DL0904 or DL0703 on the interpreter
  for some programs; and output printed before a fault was missing on WASM for one program. Redone on the laptop from this
  entry on, each witnessed first.
- **Effort (the owner, 2026-10-19: *"I noticed that routine runs or scheduled task runs is using claude opus 5.5 at medium
  effort but I want it to be always at xhigh effort"*):** `.claude/settings.json` now sets `env.CLAUDE_CODE_EFFORT_LEVEL` to
  `xhigh`. Its `effortLevel` and `modelSettings` keys already said `xhigh`, so the cloud session must have been setting effort
  explicitly, and only this variable outranks `--effort`, `/effort` and both keys (Claude Code's environment-variable reference,
  read today). A project file may set it, and single-repository cloud sessions read that file. The routine API was asked to
  carry it as well, sending its full unchanged configuration plus the variable. It refused (HTTP 400: `session_request.environment_variables is not supported on triggers`),
  and the routine is unchanged. The owner can add the same variable to the cloud environment in claude.ai as a second layer.
  The variable holds sous-chefs at `xhigh` too. `docs/CLOUD_ROUTINE.md` (*Effort*) and `HANDOFF.md` §11.3 say so. **To
  verify:** the next routine run's session header shows its effort.
- **`launch/2.0.0`:** `c3b33f3` merges `master` `6c26aed..8fc42f8` (`205f0f4`, WASM-OOM-1). The merge moved the Unreleased
  section into `## [2.0.0]` and rebuilt the map: 1532 nodes, 0 errors. `doctor --check` ok; `scripts/clippy.sh -j 4` clean
  and stamped; doctor_cli, repository_structure, evidence_claims, governance, message_spacing, release, distribution and
  book green. Dispatched at it: `ci.yml` `everything` `39296491537`, release dry run `39296497024`.
- Files and folders (the effort commit): M `.claude/settings.json`, `docs/CLOUD_ROUTINE.md`, `HANDOFF.md`,
  `docs/assistant-memory/cloud-routine-on-demand-2026-10-17.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/survey/*`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `cloud-routine-on-demand-2026-10-17.md` (the effort variable), copied both ways.
- Open / next, in order: (1) the launch runs above and this commit's push run; (2) redo run 4's two WASM fixes and the red
  team's findings, each witnessed first; (3) the WASM fragment's next pieces: `for`, `List`, `split`, `map`; RW 4.66, 4.67,
  6.1's further strings, 6.2; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — RW 3.10–3.14: the WASM engine's faults, refusals and output held to the interpreter's; `rand.int` over any span (made on the laptop)
- Session: `https://claude.ai/code/session_01WMGSP75D6H9Xw9SgL3ZD5j`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-short`   Pull request: #3   Merged: not yet (lands on `master` by fast-forward after its CI reads green)
- Base: `6a26c1f`
- **Redone from on-demand routine run 4** (its fixes and its red team's notes were lost with the VM at the usage limit; its
  run log named them). Each defect was witnessed on the laptop's binary before any fix: WASM-SHORT-1, WASM-NEG-1,
  WASM-REFUSAL-1, WASM-OUTPUT-1, RAND-SPAN-1. Details are in `V2_LOG.md` and REMAINING_WORK 3.10–3.14.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (short-circuit `&&`/`||`; negation through the checked
  multiply), `crates/delulu-wasm/src/host.rs` (`trap_or_refusal`; `HostConfig::stream_console`),
  `crates/delulu-wasm/src/gen.rs` (guarded short-circuits, negation), `crates/delulu-runtime/src/prim.rs` (`rand.int`'s
  span), `crates/delulu/src/run_cmd.rs` (the CLI streams), `crates/delulu-wasm/tests/fault_parity.rs` (four tests added
  after C20's two), `crates/delulu/tests/wasm_cli.rs`, `crates/delulu/tests/budget_cli.rs`, `CHANGELOG.md`, `HANDOFF.md`
  (§11.4), `docs/REMAINING_WORK.md` (3.10–3.14), `docs/DELULULANG_V2/V2_LOG.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md`
  (row 17, the launch runs), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Verified: mutants M525–M529 red, control green; `scripts/clippy.sh -j 4` clean and stamped; `delulu-wasm` whole and
  the ten CLI targets that run the WASM engine green on Windows. The full suite on four runners and the 50,000-program
  differential are this pull request's CI (the owner's rule of 2026-10-19).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (a differential is as wide as its generator and its comparison).
- Open / next, in order: (1) this pull request's CI, then fast-forward `master`; the launch `everything` run
  `39296491537` and `6a26c1f`'s push run; (2) the host-virtualization posture after the owner's question of 2026-10-19
  (PwnEscape, Januscape, Zapscape): `doctor` reports nested virtualization and a world-writable `/dev/kvm`,
  `DEPLOYMENT.md` names the host kernel the microVM tier needs, and a test pins the guest's absence of a network device;
  (3) the WASM fragment's next pieces: `List`, `for`, `split`, `map`; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-19 — RW 4.71: the microVM's host against 2026's KVM escapes; `doctor` reports KVM's posture (made on the laptop)
- Session: `https://claude.ai/code/session_01WMGSP75D6H9Xw9SgL3ZD5j`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/vm-posture` (on `laptop/wasm-short`)   Pull request: #4   Merged: not yet (after #3, on green CI)
- Base: `9367b13`
- **The owner's question** (2026-10-19): *"Linux recently patched vm issues, Search and learn about it, Is DeluluLang
  affected by it, What should DeluluLang do differently"*. Answered in the session: PwnEscape (CVE-2026-99999), Januscape
  (CVE-2026-53359), Zapscape (CVE-2026-64561), and the KVM CVEs of 2026-10-18. A DeluluLang guest reaches none of their
  paths; the host's kernel is the microVM's wall. `V2_LOG.md` has the reading and the sources.
- Files and folders: M `crates/delulu/src/doctor.rs` (`kvm_posture`, KVM host posture), `crates/delulu/src/microvm.rs`
  (`vm_api_calls`, pinned), `scripts/microvm/kernel.config` (six absences named), `docs/DEPLOYMENT.md` (*The microVM's
  host*), `CHANGELOG.md`, `HANDOFF.md` (§11.3, the WSL lab's `nested`), `docs/REMAINING_WORK.md` (4.71),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu/tests/microvm_guest_kernel.rs`.
  Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (Windows and the laptop's WSL; mutants M530–M532 red). The image built under the new
  `kernel.config` is CI's `microvm` job.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: the WSL lab's boot command, `modprobe kvm_amd nested=0`, the next time the KVM lab is used. A guest
  image rebuild is not needed if the built `.config` is unchanged, which is reasoned, not measured. CI's `microvm` job
  builds one.
- For the laptop's memory: nothing beyond `HANDOFF.md` §11.3.
- Open / next: as the entry before. Then the WASM `List` (literal, `len`, `get`, `push`, indexing, `for`, `map`) and
  `Str.split`, for the last two Study-C kernels.

### 2026-10-20 — RW 3.4 widened: `List` on the WASM engine, all six Study-C kernels compile; WASM-IFTY-1, WASM-INTERN-1 (made on the laptop)
- Session: `https://claude.ai/code/session_01WMGSP75D6H9Xw9SgL3ZD5j`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-list` (on `laptop/vm-posture`)   Pull request: #5   Merged: not yet (after #4, on green CI)
- Base: `a53f90a`. Pull request #3 landed on `master` by fast-forward (`9367b13`) after its CI `39299693759` read
  success on every job.
- What: D-V2-164 (`V2_DECISION_LOG.md`), REMAINING_WORK 3.4 (widened), 3.15 and 3.16 (closed), `V2_LOG.md`.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`Ty::List`, the list operations and helpers, `for`, the
  `delulu:list` imports, block types from what compiled, string collection), `crates/delulu-wasm/src/host.rs`
  (`str_split`, `index_fault`, `write_str_list`, `MEMORY_EXHAUSTED`, `run_int_fn` under a host),
  `crates/delulu-wasm/src/gen.rs` (lists in the loop generator), `crates/delulu-wasm/tests/loop_parity.rs`,
  `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`, `HANDOFF.md` (§11.4), `README.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `measurements/study-c/HOT_PATH_TABLE.md`, `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu-wasm/tests/list_parity.rs`.
  Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M533–M540; the release differential 50,000, 0 mismatches; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (a type known before compiling is a guess; a walk over an enum names every variant).
- Open / next, in order: (1) the CI of #4 and #5, landing each on green; the launch `everything` run `39296491537`; then merge
  `master` into `launch/2.0.0` again; (2) Study C over all six kernels on one machine (release, all lanes); (3) `Str == Str`,
  `pop` and the other list methods on WASM; RW 4.66, 4.67, 6.1's further strings, 6.2; (4) *[laptop]* the launch, only on the
  owner's word.

### 2026-10-20 — `Str == Str` on the WASM engine; Study C re-measured over all six kernels (made on the laptop)
- Session: `https://claude.ai/code/session_01WMGSP75D6H9Xw9SgL3ZD5j`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-streq` (on `laptop/wasm-list`)   Pull request: #6   Merged: not yet (after #5, on green CI)
- Base: `e85fcb5`. Pull request #4 landed on `master` by fast-forward (`a53f90a`) after its CI `39301188012` read
  success on every job, its `microvm` job building the guest image under the new `kernel.config`.
- What: `V2_LOG.md` (this date, the second entry); the published Study C run (2026-10-19 18:46 UTC, the laptop's WSL).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`__str_eq`, `module_any`, `module_compares`),
  `crates/delulu-wasm/src/host.rs` (`write_str_list` in one write), `crates/delulu-wasm/src/gen.rs` (comparisons in the
  console generator), `crates/delulu-wasm/tests/list_parity.rs`, `crates/delulu-wasm/tests/conformance_parity.rs`,
  `crates/delulu-measure/src/study_c.rs` (`python_command`), `measurements/study-c/REPORT.md`, `results.json`,
  `HOT_PATH_TABLE.md`, `measurements/METHODOLOGY.md`, `README.md`, `docs/QUESTIONS.md`, `docs/for-agents.md`,
  `docs/book/THE_DELULULANG_BOOK.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md`, `CHANGELOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `measurements/study-c/runs/2026-10-19-vm/`
  (`REPORT.md`, `results.json` — the run this one replaces). Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing (a cloud run of Study C would be a second machine's figures, welcome, not owed).
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #5 and #6, landing each on green; the launch `everything` run `39296491537`; then merge `master` into
  `launch/2.0.0`; (2) `pop` and the other list methods on WASM; `split` without the host crossing; RW 4.66, 4.67, 6.1, 6.2;
  (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — every `Str` method on the WASM engine; #5 and #6 landed (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-strmethods`   Pull request: #7   Merged: not yet (on green CI)
- CI read before this branch: #5's run `39304227543` and #6's run `39305971992` read success on every job (`miri-slow` and
  `heavy-gates` skipped, as on every pull request), so `master` was fast-forwarded to `46caa70` (#6, which carries #5) and
  pushed; GitHub marked both merged. Also read success: `launch/2.0.0` at `c3b33f3`, `ci.yml` `everything`
  `39296491537` (both `miri-slow` jobs included) and the release dry run `39296497024`; `master`'s push runs
  `39296967930` (`6a26c1f`), `39303198616` (`9367b13`) and `39304283373` (`a53f90a`).
- Base: `46caa70`. The branch's first commit `3ae3b3f` is the previous session's unfinished codegen half (its message
  says so); the second finishes it.
- What: `V2_LOG.md` (this date, the third entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`STR_METHODS`, `str_method_sig`, `str_method_arity`,
  `compile_str_method`, the `delulu:str` import), `crates/delulu-wasm/src/host.rs` (`delulu:str.str_method`),
  `crates/delulu-wasm/src/gen.rs` (`gen_str_method`), `crates/delulu-wasm/tests/conformance_parity.rs`, `CHANGELOG.md`,
  `docs/book/THE_DELULULANG_BOOK.md`, `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`; A `crates/delulu-wasm/tests/str_parity.rs`. Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M543–M549; the release differential 50,000, 0 mismatches; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) this branch's CI, landing on green; then merge `master` into `launch/2.0.0` and dispatch its
  `everything` and release dry runs; (2) on WASM: a named function passed as a value, `parse_int`/`parse_float`, an empty
  list typed from the checker, `pop` and the other list methods; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only
  on the owner's word.

### 2026-10-20 — a function's name as a value on the WASM engine; a generic body brings what it needs (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-fnref` (on `laptop/wasm-strmethods`, #7)   Pull request: #8   Merged: not yet (after #7, on green CI)
- Base: `650387a`.
- What: `V2_LOG.md` (this date, the fourth entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`named`, `named_value`, `resolve_callable`, `let h = double`,
  `body_is_compiled`, a generic body's literals), `crates/delulu-wasm/src/gen.rs` (`step` in the loop generator),
  `crates/delulu-wasm/tests/loop_parity.rs`, `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/REMAINING_WORK.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu-wasm/tests/named_fn_parity.rs`.
  Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M550–M557; the release differential 50,000, 0 mismatches; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) #7's and this branch's CI, landing each on green; then `launch/2.0.0`; (2) on WASM: `parse_int`
  (`03_errors`), `Float` arithmetic and `parse_float` (`01_types`, `telemetry_report`), an empty list typed from the checker
  (`02`), `pop` and the other list methods; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — `parse_int` on the WASM engine (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-parse` (on `laptop/wasm-fnref`, #8)   Pull request: #9   Merged: not yet (after #7 and #8, on green CI)
- Base: `a9ca534`. Also read: `master`'s push run `39326858607` (`46caa70`, #5 and #6 landed) — success.
- What: `V2_LOG.md` (this date, the fifth entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`module_calls_builtin`, `compile_parse_int`, the
  `delulu:str.parse_int` import), `crates/delulu-wasm/src/host.rs` (`delulu:str.parse_int`), `crates/delulu-wasm/src/gen.rs`
  (`parse_int` in the console generator), `crates/delulu-wasm/tests/str_parity.rs`,
  `crates/delulu-wasm/tests/conformance_parity.rs`, `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted,
  renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M558–M562; the release differential 50,000, 0 mismatches; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #7, #8 and this branch, landing each on green in order; then `launch/2.0.0`; (2) on WASM:
  `Float` arithmetic (IEEE as the interpreter's; `%` by the host's `fmod`), `str(Float)` without the foreign interface,
  `parse_float` (`01_types`, `telemetry_report`); an empty list typed from the checker (`02`); `pop` and the other list
  methods; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — `Float` on the WASM engine (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-float` (on `laptop/wasm-parse`, #9)   Pull request: #10   Merged: not yet (after #8 and #9, on green CI)
- Base: `d51d8d3`. Also: #7's CI `39328647809` read success on every job; `master` fast-forwarded to `650387a` and pushed.
- What: `V2_LOG.md` (this date, the sixth entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`module_uses_floats`, `mentions_float`, the `delulu:float`
  imports, `Float` operators, negation, `float`/`int`, `compile_parse`), `crates/delulu-wasm/src/host.rs`
  (`delulu:float.to_str`, `delulu:float.rem`, `delulu:str.parse_float`, `parse_by_builtin`, `float_to_guest_str`;
  `format_float` removed — `str` of a `Float` is now `Value::display` itself), `crates/delulu-wasm/src/gen.rs`
  (`gen_float`), `crates/delulu-wasm/tests/conformance_parity.rs`, `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A
  `crates/delulu-wasm/tests/float_parity.rs`. Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M563–M573; the release differential 50,000, 0 mismatches; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #8, #9 and this branch, landing each on green in order; then `launch/2.0.0`; (2) on WASM:
  records (`01_types`), an empty list typed from the checker (`02`), `pop` and the other list methods; RW 4.66, 4.67, 6.1,
  6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — an empty list typed by the checker on the WASM engine (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-emptylist` (on `laptop/wasm-float`, #10)   Pull request: #11   Merged: not yet (after #9 and #10, on green CI)
- Base: `11a50bd`. Also: #8's CI `39330325778` read success on every job; `master` fast-forwarded to `a9ca534` and pushed.
- What: `V2_LOG.md` (this date, the seventh entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`compile_checked`, `compile_module_typed`, `Cx::node_types`,
  `list_scalar`), `crates/delulu-wasm/src/lib.rs` (exports `compile_checked`), `crates/delulu/src/cli.rs` and
  `crates/delulu/src/run_cmd.rs` (call `compile_checked`), `crates/delulu-wasm/tests/list_parity.rs`,
  `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/REMAINING_WORK.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M574–M576; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #9, #10 and this branch, landing each on green in order; then `launch/2.0.0`; (2) on WASM:
  records (`01_types`), `pop` and the other list methods; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the
  owner's word.

### 2026-10-20 — `==` on records and maps; values that hold themselves; the autosave task back on (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/value-cycles` (on `laptop/wasm-emptylist`, #11)   Pull request: #12   Merged: not yet (after #10 and #11, on green CI)
- Base: `4d200b6`.
- What: `V2_LOG.md` (this date, the eighth entry); RW 2.13 and 2.14 closed, 2.15 and 2.16 opened.
- Files and folders: M `crates/delulu-runtime/src/value.rs` (`display_in`, `eq_in`, `cell_addr`, `within`; record and map
  equality; `cycle_tests`), `CHANGELOG.md`, `HANDOFF.md` (§11.1 the autosave task, §11.4 the lesson),
  `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A
  `crates/delulu-runtime/tests/value_semantics.rs`. Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M577–M582; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: done on the laptop (the autosave task note, its index line, and an index compacted under its
  size limit: twelve long lines moved whole into their topic files).
- Open / next: (1) the CI of #10, #11 and this branch, landing each on green in order; then `launch/2.0.0`; (2) MSG-CYCLE-1
  (RW 2.16), EQ-FN-1 (RW 2.15); records on WASM; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — an `iso` move carries its graph whole; the debug race checker asks for the guarantee (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/msg-cycles` (on `laptop/value-cycles`, #12)   Pull request: #13   Merged: not yet (after #10–#12, on green CI)
- Base: `825657a`.
- What: `V2_LOG.md` (this date, the ninth entry); D-V2-165; RW 2.16 closed, RW 7.20 opened.
- Files and folders: M `crates/delulu-runtime/src/actors.rs` (`MsgValue::Back`, `value_to_msg_in`, `msg_to_value_in`,
  `assert_unique_graph`, `move_graph_tests`), `crates/delulu-runtime/src/channel.rs` (`from_value_in`, a test),
  `crates/delulu-runtime/tests/actors_trace.rs`, `crates/delulu/tests/actors_cli.rs`, `docs/design/STAGE7_SPECIFICATION.md`
  (§6.2, not entrenched — `survey query`), `docs/DELULULANG_V2/V2_DECISION_LOG.md`, `CHANGELOG.md`, `docs/REMAINING_WORK.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M583–M590; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #10–#12 and this branch, landing each on green in order; then `launch/2.0.0`; (2) EQ-FN-1
  (RW 2.15); RW 7.20 under load; records on WASM; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's
  word.

### 2026-10-20 — no `==` on functions; an opaque type inside a record field is seen (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/eq-fn` (on `laptop/msg-cycles`, #13)   Pull request: #14   Merged: not yet (after #11–#13, on green CI)
- Base: `d63d13d`. Also: #10's CI `39335074061` read success on every job; `master` fast-forwarded to `11a50bd`; master's
  push run `39335645443` (`d51d8d3`) read success.
- What: `V2_LOG.md` (this date, the tenth entry); D-V2-166; RW 2.15 closed, 2.17 added closed.
- Files and folders: M `crates/delulu-check/src/check.rs` (`opaque_for`, `lacks_equality`, `no_equality`,
  `type_expr_opaque`), `crates/delulu-check/src/lib.rs` (two tests), `crates/delulu-diag/src/codes.rs` (DL0605's explain),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md`, `CHANGELOG.md`, `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M591–M598; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #11–#13 and this branch, landing each on green in order; then merge `master` into
  `launch/2.0.0` and dispatch its `everything` and release dry runs; (2) records on WASM; RW 7.20 under load; RW 4.66,
  4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — records on the WASM engine (made on the laptop)
- Session: `https://claude.ai/code/session_01Thnb8qMTfftSNkHSGE8WGZ`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-records` (on `laptop/eq-fn`, #14)   Pull request: #15   Merged: not yet (after #13 and #14, on green CI)
- Base: `d18d86f`. Also: #11's CI `39336126623` and #12's `39337254137` read success on every job; `master`
  fast-forwarded to `825657a` (both) and pushed.
- What: `V2_LOG.md` (this date, the eleventh entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`Ty::Record`, `Scalar::Record`, `RecordDesc`, the record
  registry in `build_enum_env`, `compile_record_literal`, `compile_field`, `compile_field_assign`, `lvalue_ty`,
  `lvalue_expr`), `crates/delulu/tests/wasm_cli.rs` (chapter `01`), `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A
  `crates/delulu-wasm/tests/record_parity.rs`. Deleted, renamed: nothing.
- Verified: in `V2_LOG.md` (mutants M599–M603; clippy stamped).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) the CI of #13, #14 and this branch, landing each on green in order; then merge `master` into
  `launch/2.0.0`; (2) on WASM: `Option`/`List` fields, `str` of a record, `fs_write` (`05`); RW 7.20; RW 4.66, 4.67, 6.1,
  6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — #13–#15 landed; `launch/2.0.0` merged again; linked lists and trees on WASM; RCAP-CTOR-1, WASM-SCAN-1 (made on the laptop)
- Session: `https://claude.ai/code/session_01Bp5eKZCQAuuUyAcFDuff8G`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-fields`   Pull request: see the branch   Merged: not yet (on green CI)
- Base: `d259cf2`. Also: #13's CI `39338603052`, #14's `39339338740` and #15's `39340295795` read success on every job
  (`miri-slow` and `heavy-gates` skipped, as on every pull request); `master` fast-forwarded `825657a..d259cf2` (all
  three) and pushed; `master`'s push run on `825657a` (`39339368583`) read success on every job. #15's release
  differential, unread at its commit, read afterwards: 50,000 programs, 45,027 agreed, 4,973 both faulted, 0
  mismatches; its whole `delulu-wasm` suite finished with no failure.
- `launch/2.0.0`: `master` merged in (`e32b3c7`, in the worktree `D:\nelan\dl-launch` with its own target directory):
  13 `Unreleased` sections placed under `## [2.0.0]`, `docs/survey/*` rebuilt; Survey 1541 nodes, 15408 edges, 0
  errors, 14 warnings; doctor 29 checks; clippy clean and stamped; the eight doc tests green. Its message says *"PRs
  #1-#15"*: the range `8fc42f8..d259cf2` holds #3–#15 (#1 and #2 were in the launch branch already) — recorded here, the
  pushed commit left as it is. Dispatched at it: CI `everything` `39364184892`, the release dry run `39364187774`.
- What: `V2_LOG.md` (this date, the twelfth entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`RecordDesc` fields as `Ty`, `field_ty`, `store_ty`,
  `load_ty`, `module_calls_method` on `module_any`, `enum_of_ctor`, `synthesized_ctor_ty`, `checked_ty`,
  `compile_some`, a written `let` type as the expected one, `ctor_tests`), `crates/delulu-wasm/tests/record_parity.rs`,
  `crates/delulu-check/src/rcap_check.rs` (a nullary constructor's capability), `crates/delulu-check/src/lib.rs` (its
  test), `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/REMAINING_WORK.md` (3.4; 2.18 and 3.17 added),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) this branch's CI, landing on green; the launch runs `39364184892` and `39364187774` read; (2) on
  WASM: `str` and `==` of a whole record (the interpreter's own `display`/`eq` over the guest's value, reified host-side),
  `fs_write` (`05`); RW 7.20; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — `str` and `==` of compound values on WASM; RECORD-ORDER-1 (made on the laptop)
- Session: `https://claude.ai/code/session_01Bp5eKZCQAuuUyAcFDuff8G`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-show` (on `laptop/wasm-fields`, #16)   Pull request: see the branch   Merged: not yet (after #16, on green CI)
- Base: `a7b6476`. Also: `master`'s push run on `d259cf2` (`39363663016`) read success on every job; `launch/2.0.0`'s
  release dry run `39364187774` read success (four packages; the publishing job skipped, as without a tag).
- What: `V2_LOG.md` (this date, the thirteenth entry).
- Files and folders: A `crates/delulu-wasm/src/reify.rs`, `crates/delulu-wasm/tests/value_parity.rs`; M
  `crates/delulu-wasm/src/codegen.rs` (`Descs`, `desc_ptr`, `type_desc`, `RecordDesc.name`, `module_shows_values`, the
  `delulu:value` imports, `str`/`==` of compound values, memory and heap start set after the bodies),
  `crates/delulu-wasm/src/host.rs` (`delulu:value.display`, `delulu:value.eq`, `reify_values`),
  `crates/delulu-wasm/src/lib.rs`, `crates/delulu-wasm/src/gen.rs` (a console line of `str`/`==` of `Float` lists),
  `crates/delulu-wasm/tests/{record_parity,conformance_parity,differential}.rs` (the last two compile with the checker's
  types, as the command line does),
  `crates/delulu-runtime/src/interp.rs` (`Home.records`, a literal held in declared order),
  `crates/delulu-runtime/tests/value_semantics.rs`, `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`,
  `docs/REMAINING_WORK.md` (3.4; 2.19 added), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) #16's CI, then this branch's, landing each on green in order; the launch run `39364184892` read;
  (2) on WASM: `fs_write` (`05`), a list of options, `str` of a map; RW 7.20; RW 4.66, 4.67, 6.1, 6.2; (3) *[laptop]* the
  launch, only on the owner's word.

### 2026-10-20 — `narrow` on the WASM engine; the attenuation program compiles (made on the laptop)
- Session: `https://claude.ai/code/session_01Bp5eKZCQAuuUyAcFDuff8G`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-narrow` (on `laptop/wasm-show`, #17)   Pull request: see the branch   Merged: not yet (after #16 and #17, on green CI)
- Base: `cba20d4`.
- What: `V2_LOG.md` (this date, the fourteenth entry).
- Files and folders: A `crates/delulu-wasm/tests/narrow_parity.rs`; M `crates/delulu-wasm/src/codegen.rs` (the
  `fs_narrow` import and arm; the scan names `narrow`), `crates/delulu-wasm/src/host.rs` (`delulu:cap.fs_narrow`),
  `crates/delulu-runtime/src/prim.rs` (`resolve_in_scope` public), `crates/delulu/tests/corpus_cli.rs`, `CHANGELOG.md`,
  `docs/book/THE_DELULULANG_BOOK.md`, `docs/REMAINING_WORK.md` (3.4), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) #16, #17 and this branch's CI, landing each on green in order; the launch run `39364184892` read;
  (2) on WASM: `fs_write` and `http` (`05`), a list of options, `str` of a map; RW 4.66 (a send bound to the run that
  redeemed the node: a wire change, its own decision); RW 7.20; RW 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the
  owner's word.

### 2026-10-20 — writing files on the WASM engine (made on the laptop)
- Session: `https://claude.ai/code/session_01Bp5eKZCQAuuUyAcFDuff8G`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-fswrite` (on `laptop/wasm-narrow`, #18)   Pull request: see the branch   Merged: not yet (after #17 and #18, on green CI)
- Base: `ce5f6ab`. Also: #16's CI `39365652892` read success on every job; `master` fast-forwarded `d259cf2..a7b6476`.
- What: `V2_LOG.md` (this date, the fifteenth entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`Ty::FsWrite`, the `root_fs_write`/`fs_write_text`
  imports and arms), `crates/delulu-wasm/src/host.rs` (`CapKind::FsWrite`, `fs_write_roots`, the two imports,
  `push_fs_trace`, the read path's escape record, the fs-read refusal's grant hint), `crates/delulu-wasm/src/lib.rs`
  (the custody test), `crates/delulu/src/run_cmd.rs` (`fs_write_roots` on both WASM paths),
  `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/REMAINING_WORK.md` (3.4),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new.
- Open / next: (1) #17, #18 and this branch's CI, landing each on green in order; the launch run `39364184892` read;
  (2) on WASM: `http` (`05`'s last blocker), a list of options, `str` of a map; RW 4.66; RW 7.20; RW 4.67, 6.1, 6.2;
  (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — on-demand routine run 5: the laptop's #17–#19 landed; RW 7.20 reproduced and fixed — a host's egress window is its own thread's
- Session: `https://claude.ai/code/session_018yMwiAJaxMeZ13R2HMTPTh`   Model: Claude Opus 5.5 (the system prompt names
  `claude-opus-5-5`; no fallback notice; `CLAUDE_CODE_EFFORT_LEVEL=xhigh` and `CLAUDE_EFFORT=xhigh` in the session's
  environment — the effort pin of 2026-10-19 holds); a Sonnet 5.5 red-team sous-chef on the WASM engine's file capabilities
  (its result in the next entry)
- Branch: `master`; the code's runner reads on `claude/rw-7-20-egress-window`   Pull request: none (#17–#19 landed by
  fast-forward)   Base: `a7b6476`
- Commits: `cba20d4`, `ce5f6ab`, `275af08` (the laptop's, pull requests #17–#19 — `master` fast-forwarded to them); `18b0adf`
  (RW 7.20); this entry's commit.
- Start of session (on `a7b6476`, 07:33 UTC): `survey check` ok (1540 nodes, 15400 edges); `doctor --check` ok (26 checks in
  this VM). `gh` is at `/usr/local/bin/gh` (not used; CI read through the REST API and the GitHub MCP tools).
- **Landed (the laptop's session stopped at about 04:30 UTC with three green pull requests open):** #17's CI `39367623331`,
  #18's `39368481885`, #19's `39369712525` and `a7b6476`'s push run `39368506567` — success on every job (`miri-slow` and
  `heavy-gates` skipped, as on every pull request); `master` fast-forwarded `a7b6476..275af08`; its push run `39387600500`
  success on every job. On `275af08`: `survey check` ok (1543 nodes, 15500 edges), `doctor --check` ok (26), the release
  differential (50,000 programs) passed in 151 s. The autosave branch's eleven snapshots since `bbb8ea3`: six landed exactly in
  the next commit, five were mid-edit drafts the next commit carries further (`V2_LOG.md` names each) — nothing to recover.
- **Fixed (RW 7.20):** the laptop's once-seen red of two `channel.rs` confinement tests is another thread's refused request
  counted in a host's `denied[]` — `HostChannel::answer` read a window on the process-wide egress log. Reproduced in the VM with
  a frozen pre-fix test binary: 13 of 400 runs red at the laptop's two lines; the fixed binary beside it 0 of 400. The window
  is now the answering thread's own tally; the run report's process-wide log is unchanged. Witness red on the old code; M638
  and M639 red, control green; runners at `18b0adf`: macOS `39388976507` (284 passed), Windows `39388979829` (283 passed),
  `delulu-runtime`'s whole library, 0 failed. The full suite alone at `18b0adf`: 2,431 passed, 0 failed, 16 ignored (173
  binaries), cargo exit 0, the tree unmoved.
- Files and folders: M `crates/delulu-runtime/src/egress.rs` (`MINE`, `log`, `mark`, `records_since`),
  `crates/delulu-runtime/src/channel.rs` (the witness), `crates/delulu-runtime/src/egress/tests.rs` (tightened), `CHANGELOG.md`,
  `HANDOFF.md` (§11.5), `docs/CLOUD_ROUTINE.md` (step 5: a once-seen unit-test red is reproduced with its neighbours),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (7.20 closed), `docs/assistant-memory/MEMORY.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `docs/assistant-memory/a-field-fed-by-a-global.md`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.5 (a count held in a field can still be fed by a global);
  `docs/assistant-memory/a-field-fed-by-a-global.md`, copied both ways.
- Loop engineering: `docs/CLOUD_ROUTINE.md` step 5 says how to reproduce a once-seen unit-test red (a frozen test binary, its
  filter a few hundred times at `--test-threads 16`, the fixed binary beside it as the control).
- Open / next, in order: (1) this commit's push run; (2) the red-team pass on the WASM engine's `fs_read`, `narrow` and
  `fs_write` — its findings, each re-run before use; (3) `launch/2.0.0`: merge `master` (`a7b6476`…this commit), dispatch
  `everything` and the release dry run; (4) on WASM: `http` (`05`'s last blocker), a list of options, `str` of a map; RW 4.66,
  4.67, 6.1, 6.2; (5) *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — on-demand routine run 5 (continued): a red-team pass on the WASM engine's file capabilities; RW 4.72 — a file capability is anchored at its mint (D-V2-167); `launch/2.0.0` at `3ee1370`
- Session: `https://claude.ai/code/session_018yMwiAJaxMeZ13R2HMTPTh`   Model: Claude Opus 5.5 (no fallback notice); one Sonnet
  5.5 sous-chef, the red-team pass (a usage limit stopped it at 08:10 UTC; resumed by message; its findings were on disk)
- Branch: `master`, by fast-forward from `claude/wasm-read-scope` (the code's runner reads)   Pull request: none   Base: `7355808`
- Commits: `92d9119` (F1), `3cb335e` (F2 and F3, RW 4.72), this entry's commit. `launch/2.0.0`: `3ee1370`.
- **The pass** (about 250 cases, a frozen binary in `/tmp/rtw`, nothing written in the repository): F1 — a WASM read through a
  dangling link refused without SYMLINK-DANGLE-1's words, and before asking the custody — fixed (RW 3.18); F2 and F3 — a
  file capability followed its directory when the directory (or the grant itself) was swapped for a link after the mint or
  narrowing, on BOTH engines, reads and writes outside the grant — fixed (RW 4.72, D-V2-167: the capability carries its
  `anchor`, and no use resolves the root again); F4 — `delulu run FILE.dwx` ignores the trace flags — open (RW 3.19), and
  the head chef's probe found the same path ignores `--broker daemon` and `--lease` (next entry). Every finding re-run first.
- **Verified:** witnesses red on the old code; mutants M640–M647 (M643 and M644 survive alone by design — two walls; their
  double M645/M647 red); runners at `92d9119` (macOS `39393020777`, Windows `39393024452`, macOS `wasm_cli` `39393029397`) and
  at `3cb335e` (macOS `39394517196`, `39394523755`; Windows `39394520536`, `39394527710`, `39394530798`) — all success, the new
  tests named on each; `scripts/check-other-os.sh x86_64-pc-windows-msvc` clean; the full suite at `3cb335e` 2,435 passed, 0
  failed, 16 ignored (173 binaries), exit 0.
- **`launch/2.0.0` at `3ee1370`** (`master` `d259cf2..7355808` merged; five Unreleased CHANGELOG sections moved under
  `## [2.0.0]`; Survey rebuilt 1545 nodes, 0 errors; `doctor --check` 26 ok; clippy stamped; eight doc gates green). Its release
  dry run `39392801546` success; its `everything` run `39392798450` dispatched (read at the run's close or by the next run).
- Files and folders: M `crates/delulu-runtime/src/value.rs` (`CapScope::Fs.anchor`), `crates/delulu-runtime/src/prim.rs`
  (`pin_fs_at`, `mint_anchor`, `resolve_in_anchor`, `pinned_in`; the mints, `narrow`, `read_text`, `list_dir`, the writes;
  the witness), `crates/delulu-runtime/src/interp.rs` (`fs_pin_for`), `crates/delulu-runtime/src/sink.rs`,
  `crates/delulu-runtime/src/beneath.rs` (the residual's note), `crates/delulu-wasm/src/host.rs` (`CapKind::FsRead`/`FsWrite`
  carry the anchor; the read in the interpreter's order), `crates/delulu-wasm/src/lib.rs` (two witnesses),
  `crates/delulu/tests/wasm_cli.rs` (F1's witness), `CHANGELOG.md`, `HANDOFF.md` (§11.4 ROOT-RESOLVE-1),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-167), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (3.18, 3.19,
  4.72), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing (the junction case of the witnesses ran on the Windows runners).
- For the laptop's memory: `HANDOFF.md` §11.4 (ROOT-RESOLVE-1: ask which side of a containment check is resolved when).
- Open / next, in order: (1) this commit's push run; `launch/2.0.0`'s `everything` run `39392798450`; (2) RW 3.19 and the
  `.dwx` path's unapplied `--broker`/`--lease` (the next slice of this run); (3) the pass's untried families — a user method
  named `narrow`/`read_text`, capabilities in records or closures, `--broker daemon` and `--lease` on the WASM file path;
  (4) on WASM: `http`, a list of options, `str` of a map; RW 4.66, 4.67, 6.1, 6.2; (5) *[laptop]* the launch, only on the
  owner's word.

### 2026-10-20 — on-demand routine run 5 (continued): a `.dwx` run refuses every flag it does not apply — DWX-FLAGS-1 (RW 3.19)
- Session: `https://claude.ai/code/session_018yMwiAJaxMeZ13R2HMTPTh`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef)
- Branch: `master`, by fast-forward from `claude/dwx-flags`   Pull request: none   Base: `669664c`
- Commits: `a7e50d6` (the fix); this entry's commit.
- **Fixed (RW 3.19):** `delulu run FILE.dwx` read only the grants, `--json`, the seed and the clock; `--broker daemon` (no broker
  running) and `--lease` (a token no broker issued) ran the artifact with no custody, and the trace flags wrote nothing — in
  silence. Now every flag outside `APPLIED_TO_A_DWX` is refused before anything runs (exit 2), as the sandbox refuses by
  `APPLIED_UNDER_SANDBOX`. Witness red on the old path; M648–M650 red; runners Windows `39396317161` (5 passed, filter `dwx`)
  and macOS `39396321250` (success); the full suite at `a7e50d6` 2,436 passed, 0 failed, 16 ignored (173 binaries), exit 0.
- **RW 4.72 end to end:** the red-team pass's own race scripts re-run against the fixed binary — every run refused, nothing
  read or written outside (`V2_LOG.md`).
- Files and folders: M `crates/delulu/src/run_cmd.rs` (`APPLIED_TO_A_DWX`), `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`,
  `HANDOFF.md` (§11.4 DWX-FLAGS-1), `docs/CLOUD_ROUTINE.md` (a whole-library witness log: `tail_lines` about 130),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (3.19 closed), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: `HANDOFF.md` §11.4 (DWX-FLAGS-1: an allowlist per route a command can take).
- Loop engineering: `docs/CLOUD_ROUTINE.md` — ask a whole-library witness log for about 130 lines; 2,000 brought two whole
  500-line logs into the context this run.
- Open / next, in order: (1) this commit's push run and `669664c`'s (`39396073953`); `launch/2.0.0`'s `everything`
  `39392798450` (all finished jobs success; two `miri-slow` jobs running at this entry); then merge `master` into
  `launch/2.0.0` again; (2) the pass's untried families — a user method named `narrow`/`read_text`, capabilities in records
  or closures, `--broker daemon` and `--lease` on the WASM file path end to end; (3) a `.dwx` run with a trace and custody, if
  wanted, instead of the refusal; (4) on WASM: `http`, a list of options, `str` of a map; RW 4.66, 4.67, 6.1, 6.2; (5)
  *[laptop]* the launch, only on the owner's word.

### 2026-10-20 — on-demand routine run 5 (close): `launch/2.0.0` at `20666c5`
- Session: `https://claude.ai/code/session_018yMwiAJaxMeZ13R2HMTPTh`   Model: Claude Opus 5.5 (no fallback notice)
- Branch: `launch/2.0.0` (`20666c5`, merging `master` `7355808..ec4bc87`); `master` (this entry's commit)
- Gates on the launch tree: survey build 1545 nodes, 15597 edges, 0 errors, 14 warnings, check ok; `doctor --check` ok (26 checks); `scripts/clippy.sh -j 4` clean and stamped; doctor_cli, repository_structure, evidence_claims, governance, message_spacing, release, distribution and book green. Dispatched at `20666c5`: `ci.yml` `everything` and the release dry run.
- Files and folders (`master`): M `docs/DELULULANG_V2/V2_PHASE_STATUS.md` (row 17), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_ROUTINE.md` (step 4.3: free `master`'s `target/debug` before a launch build under ~15 GB), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. (`launch/2.0.0`: `CHANGELOG.md`, `docs/survey/*` and everything the merge brought.)
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing beyond the entries above.
- **Postscript:** RW 4.72's race under a live broker daemon refused on both engines; a capability in a closure keeps its scope on both, one in a record field is DL1201 on WASM (`V2_LOG.md`). CI: push runs `39387600500` (`275af08`), `39390507228` (`7355808`), `39396073953` (`669664c` — F1 and RW 4.72 on all three operating systems) and `39397818042` (`ec4bc87` — RW 3.19) success on every job; the nightly `39391181701` (`7355808`) and `launch/2.0.0`'s `everything` `39392798450` (`3ee1370`) success on every job but their two `miri-slow` jobs, running at this entry; `20666c5`'s release dry run `39398473807` success and its `everything` `39398467836` running; `318b210`'s push run `39398823386` running.
- **`--lease` on the WASM file path:** parity with the interpreter, the lease's custody refusing an escape in the same words on both (`V2_LOG.md`). Oddity for the next run: under `--lease`, DL0703's hint tells the holder to pass `--grant`, which a lease run refuses.
- **This run, in one line:** the laptop's #17–#19 landed; RW 7.20's flake reproduced and fixed; a red-team pass on the WASM
  file capabilities found F1–F4, of which F2/F3 (RW 4.72) was a HIGH escape in BOTH engines — fixed, D-V2-167 — and F4 led to
  DWX-FLAGS-1 (RW 3.19) — fixed; `launch/2.0.0` merged twice.
- Open / next, in order: (1) the push run of this run's last commit (`669664c`, `ec4bc87`, `318b210` and `8ac7ee5` read green);
  `launch/2.0.0`'s `everything` runs `39392798450` (its two `miri-slow` jobs) and `20666c5`'s, and `20666c5`'s release dry run; (2) DL0703's
  hint under `--lease` (it names a `--grant` a lease run refuses); a user method named `narrow`/`read_text` on WASM; (3) a `.dwx` run with a trace and custody, if wanted,
  instead of the refusal; (4) on WASM: `http`, a list of options, `str` of a map; RW 4.66, 4.67, 6.1, 6.2; (5) *[laptop]* the
  launch, only on the owner's word.

### 2026-10-20 — on-demand routine run 5: stopped on the owner's word (09:50 UTC)
- Session: `https://claude.ai/code/session_018yMwiAJaxMeZ13R2HMTPTh`   Model: Claude Opus 5.5 (no fallback notice; no sous-chef running)
- **The owner, 09:50 UTC:** *"Head Chef Claude Opus 5.5, Stop safely and record and push everything to GitHub test repo"*.
  Stopped with nothing half-done: every slice of this run is on `master` (`6a03165` before this entry), the tree was clean
  and in step with `origin`, no worktree, no suite, no broker daemon and no sous-chef left running. The three working
  branches (`claude/rw-7-20-egress-window`, `claude/wasm-read-scope`, `claude/dwx-flags`) are 0 commits ahead of `master`.
- **CI at the stop** (no job failed anywhere): `6a03165`'s push run `39403469304` running (records only); `launch/2.0.0`'s
  `everything` `39398467836` (`20666c5`) 16 jobs success, its two `miri-slow` jobs running, its release dry run `39398473807`
  success; `3ee1370`'s `everything` `39392798450` and the nightly `39391181701` (`7355808`) 17 jobs success each, one
  `miri-slow` job running in each. `8ac7ee5`, `318b210`, `ec4bc87`, `669664c`, `7355808` and `275af08` push runs success on every job.
- Files and folders: M `docs/CLOUD_SYNC_LOG.md` (this entry), `docs/DELULULANG_V2/V2_LOG.md`, `docs/survey/*`.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, for whoever comes next (laptop or routine): (1) read the four runs above to their end; (2) DL0703's hint under
  `--lease`; a user method named `narrow`/`read_text` on WASM; (3) a `.dwx` run with a trace and custody, if wanted; (4) on
  WASM: `http`, a list of options, `str` of a map; RW 4.66, 4.67, 6.1, 6.2; (5) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — the laptop synced after on-demand routine run 5; a lease run's DL0703 names its delegation (RW 6.15) (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/lease-remedy`   Pull request: see the branch   Merged: not yet (on green CI, by fast-forward)
- Base: `9c1d30b`. **Synced first:** `master` fast-forwarded `a7b6476..9c1d30b` (run 5); the four runs run 5 left going read
  to their end — `39403469304`, `39398467836` (`launch/2.0.0` `20666c5`, `miri-slow` included), `39392798450`, `39391181701`:
  success on every job; `9c1d30b`'s push run `39403663796` success. `survey check` ok (1544 nodes), `doctor --check` ok (29
  on this machine). The worktree `D:\nelan\dl-launch` fast-forwarded to `launch/2.0.0` `20666c5`. Run 5's
  `docs/assistant-memory/` files and `HANDOFF.md` §11.4–§11.5 copied into the laptop's memory.
- What: `V2_LOG.md` (2026-10-21, the first entry).
- Files and folders: M `crates/delulu-runtime/src/prim.rs` (`grant_remedy`; every mint's DL0703; the unit test),
  `crates/delulu-runtime/src/broker.rs` (`Grants::lease`, `build_root`), `crates/delulu-runtime/src/value.rs`
  (`RootVal::lease`), `crates/delulu-runtime/src/actors.rs` (`RootMsg::lease`, both conversions, the round-trip test),
  `crates/delulu-wasm/src/host.rs` (`HostConfig::lease`, `HostState::lease`, five refusals in the interpreter's words),
  `crates/delulu-wasm/tests/fault_parity.rs`, `crates/delulu/src/cli.rs` (`grants_from_lease` sets it),
  `crates/delulu/src/run_cmd.rs` (the WASM `HostConfig`), `crates/delulu/src/repl.rs`,
  `crates/delulu/tests/grants_cli.rs`, `CHANGELOG.md`, `docs/REMAINING_WORK.md` (6.15), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing.
- For the laptop's memory: nothing new beyond run 5's, copied.
- Open / next: (1) this branch's CI, landing on green; (2) the next commit on this branch — a behaviour named like a
  capability method on WASM; (3) on WASM: `http` (`05`'s last blocker), a list of options, `str` of a map; RW 4.66, 4.67,
  6.1, 6.2; a `.dwx` run with a trace and custody, if wanted; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — a behaviour named like a capability method is a send on WASM (RW 3.20) (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/lease-remedy` (its second commit, on `89aeefc`)   Pull request: see the branch   Merged: not yet
- What: `V2_LOG.md` (2026-10-21, the second entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`recv_actor`; the method arm sends first),
  `crates/delulu-wasm/tests/actors_parity.rs`, `CHANGELOG.md`, `docs/REMAINING_WORK.md` (3.20),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- Open / next: (1) this branch's CI, landing on green; then `launch/2.0.0`: merge `master`, dispatch `everything` and the
  release dry run; (2) on WASM: `http` (`05`'s last blocker), a list of options, `str` of a map; RW 4.66, 4.67, 6.1, 6.2;
  a `.dwx` run with a trace and custody, if wanted; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — three new advisories fixed by patch updates; rustfmt; `scripts/clippy.sh` checks the format (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/lease-remedy` (its third commit, on `aff6dc9`)   Pull request: #20   Merged: not yet
- Why: #20's CI `39555991114` — `supply-chain` red on RUSTSEC-2026-0378, 0379 (`wasmtime` 48.0.6) and 0382 (`rustls-webpki`
  0.103.15), published after `9c1d30b`'s green push run; `lints` red on rustfmt in two of the branch's files.
- What: `V2_LOG.md` (2026-10-21, the third entry): which advisory reaches what (0379, aarch64, through a `.dwx` or `.dpx`
  someone else built, is the one that matters), the update, the evidence.
- Files and folders: M `Cargo.lock` (`rustls-webpki` 0.103.16; `wasmtime` 48.0.7 and the twenty-seven packages released with it — nothing else),
  `crates/delulu-runtime/src/prim.rs` and `crates/delulu/tests/grants_cli.rs` (rustfmt only), `scripts/clippy.sh` (refuses
  to stamp a tree `cargo fmt --check` would change), `AGENTS.md` (step 3 says so), `CHANGELOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`; the new refusal witnessed by hand (a misformatted line in a test file → "no stamp written";
  restored byte for byte → stamped with the same hash as before).
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. Loop engineering: `scripts/clippy.sh` now runs rustfmt's check first, so the pre-commit
  hook refuses an unformatted commit as it refuses an unlinted one.
- Open / next: (1) #20's CI again, landing on green; then `launch/2.0.0`: merge `master` (these advisories gate its
  `supply-chain` job too), dispatch `everything` and the release dry run; (2) `http` on the WASM engine (the next branch,
  in progress); (3) RW 4.66, 4.67, 6.1, 6.2; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — the network on the WASM engine: `root.http` and `Cap[Http].get`; chapter five runs (RW 3.4) (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-http` (on `laptop/lease-remedy`, #20)   Pull request: see the branch   Merged: not yet (after #20, on green CI)
- What: `V2_LOG.md` (2026-10-21, the fourth entry).
- Files and folders: M `crates/delulu-wasm/src/codegen.rs` (`Ty::Http`, the `root_http`/`http_get` imports gated on the
  mint, the two arms), `crates/delulu-wasm/src/host.rs` (`CapKind::Http`, `HostConfig::net`/`net_special`, the two
  imports — the interpreter's mint and `get`, in its order; `build_net_answer`), `crates/delulu-wasm/src/lib.rs` (the
  custody test), `crates/delulu-wasm/tests/fault_parity.rs` (the `http` mint's remedy), `crates/delulu/src/run_cmd.rs`
  (the grants' `net` on both WASM paths), `crates/delulu/tests/wasm_cli.rs` (`listen`; the parity test),
  `crates/delulu/tests/examples_run.rs` (chapter five on both engines), `CHANGELOG.md`, `docs/REMAINING_WORK.md` (3.4: 12
  of 17), `docs/book/THE_DELULULANG_BOOK.md` (the WASM paragraph), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- Open / next: (1) #20's CI, then this branch's, landing in order; then `launch/2.0.0`: merge `master`, dispatch
  `everything` and the release dry run; (2) on WASM: `get_with`, a list of options, `str` of a map; RW 4.66 (a bearer key
  bound at redeem — a wire change, its own D-V2), 4.67, 6.1, 6.2; (3) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — a sealed artifact runs under a lease, the broker daemon and a trace (RW 3.19's rest) (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/dwx-custody` (on `laptop/wasm-http`, #21)   Pull request: see the branch   Merged: not yet
- What: `V2_LOG.md` (2026-10-21, the fifth entry).
- Files and folders: M `crates/delulu/src/run_cmd.rs` (`lease_for_run`, `RunCustody`, `custody_for_run`, `wasm_trace_sink`,
  `emit_wasm_trace`, `assert_wasm_trace`; the source route calls them; the `.dwx` route too; `APPLIED_TO_A_DWX`),
  `crates/delulu/tests/grants_cli.rs`, `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`, `docs/REMAINING_WORK.md` (3.19),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- Open / next: (1) ARTIFACT-CLAIM-1, this branch's next commit; (2) #20 and #21 landing on green; then `launch/2.0.0`.

### 2026-10-21 — ARTIFACT-CLAIM-1: an artifact runs only what it declares (RW 4.73); WASM-CODE-1; #20 and #21 landed (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/dwx-custody` (its second commit, on `eb2372c`)   Pull request: see the branch   Merged: not yet
- **Landed:** #20 (`89aeefc`, `aff6dc9`, `5470b53`) on its CI `39558166090`, success on every job; #21 (`4729747`) on
  `39558913163`, success on every job — `master` fast-forwarded `9c1d30b..5470b53..4729747` (the push runs `39561045205`
  and `39561096457` running at this entry). The entries above that said "Merged: not yet" are these.
- What: `V2_LOG.md` (2026-10-21, the sixth entry).
- Files and folders: M `crates/delulu-wasm/src/host.rs` (`HostConfig`/`HostState::declared_effects`; `custody_allows`
  refuses an undeclared effect first), `crates/delulu-wasm/src/lib.rs` (the unit witness), `crates/delulu/src/run_cmd.rs`
  (the `.dwx` route passes its manifest's effects), `crates/delulu/src/cli.rs` (`wasm_fault_code` by the first registered
  code, its unit test; the artifact report's `enforced:` line), `crates/delulu-diag/src/codes.rs` (DL1202's explanation),
  `crates/delulu/tests/wasm_cli.rs` (the doctored artifact), `CHANGELOG.md`, `HANDOFF.md` (§11.4: ARTIFACT-CLAIM-1;
  DWX-FLAGS-1's sequel), `docs/book/THE_DELULULANG_BOOK.md` (the false sentence corrected in place),
  `docs/REMAINING_WORK.md` (4.73), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: `HANDOFF.md` §11.4 ARTIFACT-CLAIM-1 (copied as
  `artifact-claim-1.md`).
- Open / next: (1) this branch's CI, landing on green; (2) `launch/2.0.0`: the merge of `4729747` is staged in
  `D:\nelan\dl-launch`; after this branch lands, merge it too, rebuild the Survey there, run its gates, dispatch
  `everything` and the release dry run, and re-cite checklist criterion 4 (supply chain) from that run — the advisory
  database moved on 2026-10-21; (3) RW 4.66 (left open: a same-uid process reads the secret store anyway), 4.67, 6.1, 6.2;
  on WASM `get_with` (needs maps), a list of options; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — DWX-ACTORS-1: an actor program has no `.dwx` yet, and says so (RW 3.21) (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/dwx-actors` (on `laptop/dwx-custody`, #22)   Pull request: see the branch   Merged: not yet (after #22)
- What: `V2_LOG.md` (2026-10-21, the seventh entry).
- Files and folders: M `crates/delulu/src/cli.rs` (`dwx_actors_refusal`; `build_wasm_artifact` refuses actors),
  `crates/delulu/src/run_cmd.rs` (the `.dwx` route refuses an actor artifact), `crates/delulu-wasm/src/artifact.rs`
  (`exports_actor_members`), `crates/delulu-wasm/src/lib.rs` (its re-export), `crates/delulu/tests/wasm_cli.rs`,
  `CHANGELOG.md`, `docs/REMAINING_WORK.md` (3.21), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- **Also this day, on `launch/2.0.0`:** `6863bf7` merges `master` `9c1d30b..4729747` (#20, #21; the CHANGELOG's four
  sections moved under `## [2.0.0]`; the Survey rebuilt — 1545 nodes, 0 errors; `doctor --check` 29 ok; clippy stamped;
  eight doc gates, 57 passed). Dispatched there: `everything` `39562457688`, release dry run `39562463148`.
- Open / next: (1) #22, then this branch, landing on green; (2) `launch/2.0.0`: read `39562457688` and `39562463148`,
  re-cite checklist criterion 4 from the former; merge #22 and this branch, the 2.0.0 Breaking paragraph gaining
  ARTIFACT-CLAIM-1's refusal and its `.dwx` clause saying the custody and trace flags are applied; (3) RW 3.21's rest (the
  actor table in the code), P9-05's open steps (the path in the chain's record, `authority`'s path walk, methods), RW 4.67,
  6.1, 6.2; (4) *[laptop]* the launch, only on the owner's word.

### 2026-10-21 — the laptop's close: #22 and #23 landed; `launch/2.0.0` at `6863bf7`, then `5d5ba5a` (made on the laptop)
- Session: `https://claude.ai/code/session_018FZ43XgcChWVwbhr89qngA`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/records-1021` (records only)   Base: `a2fb0cd`
- **Landed, each on its own green CI read to the end, by fast-forward:** #22 (`eb2372c`, `0439453`) on `39562082756`; #23
  (`a2fb0cd`) on `39563303281` — success on every job (`miri-slow` and `heavy-gates` skipped, as on every pull request).
  `master`: `9c1d30b..5470b53..4729747..0439453..a2fb0cd` today. Push runs read: `39561045205` (`5470b53`) and
  `39561096457` (`4729747`) success on every job; `39565269313` (`0439453`) and `39566586112` (`a2fb0cd`) running at this
  entry, their `openshell` runs success.
- **`launch/2.0.0`:**
  - `6863bf7` merged `9c1d30b..4729747` (#20, #21). Its release dry run `39562463148`: **success**. Its `everything`
    `39562457688`: every job success, `supply-chain` included, except the two `miri-slow` jobs (check, broker), running.
  - `5d5ba5a` merged `4729747..a2fb0cd` (#22, #23):
    - the CHANGELOG's three sections under `## [2.0.0]`;
    - the Breaking paragraph's `.dwx` clause brought up to date, with ARTIFACT-CLAIM-1's refusal and DWX-ACTORS-1's;
    - `CHECKLIST-2.0.md` criterion 4 re-cited: the advisory database moved, the gate red before `5470b53` and green on
      `39562457688`, the tagged tree's own run read before the tag.
  - Gates on `5d5ba5a`: survey 1545 nodes, 0 errors; `doctor --check` 29 ok; clippy stamped; eight doc gates, 57 passed.
    Dispatched: `everything` `39566831246`, release dry run `39566838279`.
- Files and folders: M `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/survey/*`. Added, deleted, renamed:
  nothing.
- Survey and doctor (after the last edit): in the commit message.
- Open / next, for whoever comes next (laptop or routine):
  1. Read `39562457688`'s two `miri-slow` jobs, `39566831246` and `39566838279` (`launch/2.0.0` `5d5ba5a`), and the push
     runs `39565269313`, `39566586112`.
  2. RW 3.21's rest — the actor table carried by the code.
  3. P9-05's open steps (the path in the chain's record, `authority`'s path walk, methods).
  4. On WASM: maps (then `get_with`, `str` of a map), a list of options.
  5. RW 4.66 (open by reason: a same-uid process reads the secret store anyway), 4.67, 6.1, 6.2.
  6. *[laptop]* the launch, only on the owner's word.

### 2026-10-21 (evening) — EPOCH-WINDOW-1: a broker test timed the daemon's shutdown inside its window (RW 7.21) (made on the laptop)
- Session: `https://claude.ai/code/session_01Jomeb6seaffGYMnXvPmvGw`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/epoch-window`   Pull request: see the branch   Base: `3af2b2a`
- **Synced first:** `origin/master` = `3af2b2a`, nothing new from a routine. CI read: `launch/2.0.0` `5d5ba5a` — `everything`
  `39566831246` **success on every job** (`miri-slow` included) and the release dry run `39566838279` **success**; `6863bf7`'s
  `everything` `39562457688` success on every job (its `miri-slow` finished); the push runs `39565269313` (`0439453`) and
  `39566586112` (`a2fb0cd`) and the nightly `39567648331` (`a2fb0cd`) success; **`39569868995` (`3af2b2a`, push) failure** —
  this entry.
- What: `V2_LOG.md` (2026-10-21, the evening entry).
- Files and folders: M `crates/delulu/src/brokerd.rs` (the test), `docs/REMAINING_WORK.md` (7.21),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new (the timing rule held: the runner, not the custody).
- Open / next: this branch's CI; then RW 3.21's rest (the actor table in the code), P9-05's open steps, maps on WASM.

### 2026-10-21 (evening) — an actor program's `.dwx` runs: the module carries its own actor table (RW 3.21 closed) (made on the laptop)
- Session: `https://claude.ai/code/session_01Jomeb6seaffGYMnXvPmvGw`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/dwx-actors-run` (on `laptop/epoch-window`, #25)   Pull request: see the branch   Merged: not yet (after #25)
- What: `V2_LOG.md` (2026-10-21, the second evening entry).
- Files and folders: M `crates/delulu-wasm/src/actors.rs` (`fields_export_name`, `table_from_module`, `field_counts`,
  `set_field_counts`), `crates/delulu-wasm/src/codegen.rs` (the count globals and their exports),
  `crates/delulu-wasm/src/host.rs` (`run_artifact`, `run_actors_in`), `crates/delulu-wasm/src/lib.rs` (the re-export),
  `crates/delulu-wasm/tests/actors_parity.rs`, `crates/delulu/src/run_cmd.rs` (the `.dwx` route runs actors; `--on-quiesce`),
  `crates/delulu/src/cli.rs` (the build's refusal removed), `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`,
  `docs/REMAINING_WORK.md` (3.21), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted,
  renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- Open / next: (1) #25 and this branch landing on green; (2) `launch/2.0.0`: merge, the Breaking paragraph's actor clause
  removed; (3) P9-05's open steps, maps on WASM, RW 4.67, 6.1, 6.2.

### 2026-10-21 (evening) — P9-05 (c)'s static half: `authority --json` names each request (D-V2-168) (made on the laptop)
- Session: `https://claude.ai/code/session_01Jomeb6seaffGYMnXvPmvGw`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/p9-05-requests` (on `laptop/dwx-actors-run`, #26)   Pull request: see the branch   Merged: not yet
- **Landed:** #25 (`f05c329`, EPOCH-WINDOW-1) on its CI `39592905415`, success on every job — `master` `3af2b2a..f05c329`.
- What: `V2_LOG.md` (2026-10-21, the third evening entry); `V2_DECISION_LOG.md` D-V2-168.
- Files and folders: M `crates/delulu/src/within.rs` (`request_destinations`; `SendWalk` carries the types),
  `crates/delulu/src/cli.rs` (the report's `requests`), `crates/delulu/src/schema.rs` (the key in the closed schema),
  `crates/delulu/tests/within_cli.rs`, `tests/core-invariance/SNAPSHOT.txt` (one case, re-recorded — its diff in
  `V2_LOG.md`), `CHANGELOG.md`, `docs/REMAINING_WORK.md` (2.12), `docs/DELULULANG_V2/V2_DECISION_LOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- Open / next: (1) #26 and this branch landing on green; (2) `launch/2.0.0`: merge `f05c329` onward (the Breaking
  paragraph's actor clause to go); (3) a package's `requests`, the path in the chain's record, methods; maps on WASM.

### 2026-10-21 (evening) — a package's `requests`: each module by its own node types (D-V2-168 item 2) (made on the laptop)
- Session: `https://claude.ai/code/session_01Jomeb6seaffGYMnXvPmvGw`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/p9-05-packages` (on `laptop/p9-05-requests`, #27)   Pull request: see the branch   Merged: not yet
- **Landed:** #26 (`12d5fa2`) on its CI `39594329482`, success on every job — `master` `f05c329..12d5fa2`.
- What: `V2_LOG.md` (2026-10-21, the fourth evening entry); D-V2-168 item 2 amended.
- Files and folders: M `crates/delulu-check/src/program.rs` (`Program::node_types`; `check_program` keeps them),
  `crates/delulu-check/src/deps.rs` (`check_workspace` keeps them), `crates/delulu/src/cli.rs` (the units beside the
  modules; a package's `requests`; the single-file `Program`), `crates/delulu-atlas/src/lib.rs` (the test helper's
  `Program`), `crates/delulu/tests/within_cli.rs`, `CHANGELOG.md`, `docs/REMAINING_WORK.md` (2.12),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Added, deleted, renamed: nothing.
- Verified: in `V2_LOG.md`.
- Survey and doctor (after the last edit): in the commit message.
- Redo on the laptop: nothing. For the laptop's memory: nothing new.
- Open / next: (1) #27 and this branch landing on green; (2) `launch/2.0.0`: merge `master` (the Breaking paragraph's
  actor clause to go); (3) the path in the chain's record; methods with the first non-GET primitive; maps on WASM.

### 2026-10-21 (evening) — a once-seen Windows red made to say why (RW 7.22); `launch/2.0.0` at `678dee6` (made on the laptop)
- Session: `https://claude.ai/code/session_01Jomeb6seaffGYMnXvPmvGw`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/launcher-witness`   Pull request: see the branch   Base: `fa4021c`
- **Landed earlier this evening:** #27 (`eef9583`) on `39596126140` and #28 (`fa4021c`) on `39598422553`, success on every
  job. **Push runs:** `39595752855` (`f05c329`) and `39597236450` (`12d5fa2`) success; **`39599000467` (`eef9583`) failure** —
  this entry; `39600787572` (`fa4021c`) running.
- **`launch/2.0.0` at `678dee6`** (merging `a2fb0cd..fa4021c`, #24–#28; the CHANGELOG's two sections under `## [2.0.0]`; the
  Breaking paragraph's clause that `build` refuses an actor program removed, no longer true; the Survey rebuilt, 1545 nodes,
  0 errors; `doctor --check` 29 ok; clippy stamped; ten gates incl. `core_invariance`, 75 passed). Its release dry run
  `39601108596`: **success**. Its `everything` `39601103948`: running.
- **The autosave task** (the owner asked, 18:00 IST): Ready, every 10 minutes, last result 0, none missed. Its log shows
  each snapshot pushed to `autosave/laptop`; every branch equals `origin`. Renewed with `scripts\autosave.ps1 -Install
  -Hours 168`, to 2026-10-28 18:03 IST.
- What: `V2_LOG.md` (2026-10-21, the fifth evening entry).
- Files and folders: M `crates/delulu/tests/sandbox_confirm_cli.rs`, `docs/REMAINING_WORK.md` (7.22),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Open / next: (1) the three witness runs and `39601103948`; (2) RW 7.22's root cause from the next red's words; (3) the path
  in the chain's record; methods; maps on WASM.

### 2026-10-21 (evening) — the routine every six hours; the 2026-09-17 agent worktrees removed; RW 7.22's witnesses green (made on the laptop)
- Session: `https://claude.ai/code/session_01Jomeb6seaffGYMnXvPmvGw`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/launcher-witness` (its second commit)   Pull request: #29
- **The routine's schedule (the owner):** *"Routine / Scheduled Task u created is not following 5 hr and 5 min gap as I
  told"*. Read back from the API: `5 */5 * * *` UTC, `enabled: false` — every five hours, with a four-hour gap each night
  (20:05 → 00:05 UTC, 01:35 → 05:35 IST); cron has no 5 h 5 min interval. Offered every six hours or the same; he chose
  **every six hours**: set through the API to `5 */6 * * *` (05:35, 11:35, 17:35, 23:35 IST), read back `enabled: false` —
  still paused, the owner's switch. `docs/CLOUD_ROUTINE.md`, `HANDOFF.md` §0, D-V2-146 (amended) and the memory copy say so.
- **The two agent worktrees of 2026-09-17** removed on the owner's word, both checks first, their notes kept outside the
  repository — `docs/maintenance/WORKTREE-CLEANUP-2026-10-21.md`.
- **RW 7.22's reproduction:** `witness.yml` on `windows-latest` at `fa4021c`, the test alone, three times — `39602082207`,
  `39602087235`, `39602091804`: **success** each. Not reproduced; the witness now says why when it next fails.
- **CI watched with a Monitor** for the rest of the session (the owner asked how CI is watched): each run's end reported
  as it lands, a failed job named.
- Files and folders: M `docs/CLOUD_ROUTINE.md`, `HANDOFF.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`,
  `docs/assistant-memory/cloud-routine-on-demand-2026-10-17.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md`
  (7.22), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `docs/maintenance/WORKTREE-CLEANUP-2026-10-21.md`. Deleted, renamed:
  nothing tracked.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-21 (night) — the path in the chain's record (P9-05, D-V2-169) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/net-path-record`   Pull request: the one this entry rides in
- **Session start:** `git fetch`; `master` at `7431dbf` (#29 landed by fast-forward on its green run `39603294484`; its push
  run `39606506663` success). `launch/2.0.0`'s `everything` `39601103948` and release dry run `39601108596`: **success**,
  both read. `origin/claude/*`: only the two superseded witness branches (`7d6a22b`, `d3f0d52`). `autosave/laptop` older
  than `master`. The Survey `ok` (1545 nodes); `doctor --check` 29 checks passed. Autosave task Ready, result 0.
- **What:** a daemon or leased run's `net` use carries its request path. The broker judges it against the node's grant and
  names it in the use's payload as `url_path`, while `target` stays the host. `audit query` prints `path=`, and OCSF exports
  `http_request.url`. This fixes the chain's `use allow` for a request the call then refused (witnessed on `678dee6`'s
  binary). The tests, M703–M717 and the local results are in `V2_LOG.md`.
- Files and folders: M `crates/delulu-broker/src/{diag,ocsf,validate}.rs`,
  `crates/delulu-runtime/src/{channel,custody,interp,lib,prim}.rs`, `crates/delulu-wasm/src/host.rs`,
  `crates/delulu/src/{broker_client,broker_ipc,brokerd,cli}.rs`, `crates/delulu/tests/{audit_ocsf_cli,broker_cli}.rs`,
  `CHANGELOG.md`, `docs/DEPLOYMENT.md`, `docs/DELULULANG_V2/{V2_DECISION_LOG,V2_LOG,V2_P9_DESIGN,V2_PHASE_STATUS}.md`,
  `docs/REMAINING_WORK.md` (2.12), `HANDOFF.md` (§11.4 NET-PATH-RECORD), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-21 (night) — `Map` on the WASM engine (RW 3.4, D-V2-170) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-maps` (stacked on `laptop/net-path-record`, which landed on `master` as `a01a27a` on its green run
  `39635716984`)   Pull request: the one this entry rides in
- **What:** a `Map` compiles to WebAssembly. It lives in the guest's memory, with its keys placed by the host in the
  interpreter's own `MapKey` order. It has all eight methods, and `str` and `==` of a map. The tests, M718–M733 (M725
  survives, recorded) and the local results are in `V2_LOG.md`.
- Files and folders: M `crates/delulu-wasm/src/{codegen,host,reify}.rs`, `crates/delulu-wasm/tests/value_parity.rs`,
  `CHANGELOG.md`, `docs/for-agents.md` (its stale "none of them compile to WebAssembly"),
  `docs/DELULULANG_V2/{V2_DECISION_LOG,V2_LOG}.md`, `docs/REMAINING_WORK.md` (3.4), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`; A `crates/delulu-wasm/tests/map_parity.rs`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-21 (night) — every `List` method on the WASM engine (RW 3.4, D-V2-171) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-list-methods` (on `master` `8cfbc01`, where #31 landed on its green run `39639331172`; `master`'s
  push run for `a01a27a`, `39638797705`, success)   Pull request: the one this entry rides in
- **What:** the ten `List` methods the WASM engine lacked now compile. A walk that runs code reads a private copy of the
  slots in a module that pops, the divergence an existing pin named. The tests, M734–M752 and the local results are in
  `V2_LOG.md`.
- Files and folders: M `crates/delulu-wasm/src/{codegen,host}.rs`,
  `crates/delulu-wasm/tests/{list_parity,conformance_parity}.rs`, `CHANGELOG.md`, `docs/for-agents.md`, `HANDOFF.md`
  (§11.4 POP-WALK-1), `docs/DELULULANG_V2/{V2_DECISION_LOG,V2_LOG}.md`, `docs/REMAINING_WORK.md` (3.4),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu-wasm/tests/list_methods_parity.rs`. Deleted, renamed:
  nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — `contains` on records and sums on the WASM engine (RW 3.4) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-contains-compound` (on `master` `d42d216`, where #32 landed on its green run `39642920850`)   Pull request: the one this entry rides in
- **`launch/2.0.0` = `81f34d1`** (two merges of `master`: `8ff2b17` takes #29–#31, `81f34d1` takes #32; three
  `Unreleased` sections moved under `## [2.0.0]`; the Survey rebuilt; doctor ok; clippy stamped; the ten gates, 75
  passed). Dispatched: the release dry run `39646725256` (**success**, read) and CI `jobs=everything` `39646720253` (running at
  this commit; its result is recorded by the next master commit).
- **What:** `contains` on a list of records or sums compiles, through the host's `eq` (V2_LOG). Files: M
  `crates/delulu-wasm/src/codegen.rs`, `crates/delulu-wasm/tests/list_methods_parity.rs`, `CHANGELOG.md`,
  `docs/for-agents.md`, `docs/DELULULANG_V2/{V2_DECISION_LOG,V2_LOG}.md`, `docs/REMAINING_WORK.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — a `match` tested inside each constructor, and proved to cover every case (D-V2-172) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/match-nested` (on `master` `30885b1`, where #33 landed on its green run `39647664706`; `master`'s push
  run for `30885b1`, `39651010312`, **success**)   Pull request: #34
- **The CI `jobs=everything` run on `launch/2.0.0` `81f34d1`, `39646720253`: success** (read 2026-10-22; its last job,
  miri-slow on `delulu-broker`, finished green).
- **What:** MATCH-NEST-1 (the WASM engine ran the wrong arm of a match testing inside a constructor), MATCH-EXHAUST-1
  (the checker proved only the top level exhaustive) and WASM-IMPORTS-1 (a map-only module could not be loaded), each
  witnessed on `30885b1` and fixed. Tests, mutants M756–M776, the bound on the search and local results are in
  `V2_LOG.md`.
- Files and folders: M `crates/delulu-check/src/{check,lib}.rs`, `crates/delulu-wasm/src/{codegen,host}.rs`,
  `crates/delulu-wasm/tests/map_parity.rs`, `CHANGELOG.md`, `HANDOFF.md` (§11.4), `crates/delulu-diag/src/codes.rs`
  (DL0407's explanation), `docs/DELULULANG_V2/{V2_DECISION_LOG,V2_LOG}.md`, `docs/REMAINING_WORK.md` (2.20, 3.22,
  3.23), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu-wasm/tests/match_parity.rs`,
  `tests/conformance/reject/DL0407_{nested,literal}_non_exhaustive.delulu`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — deep values print and compare on both engines (RW 4.74) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/deep-values` (on `master` `0061ca1`, where #34 landed on its green run `39662388275`)   Pull request: the one this entry rides in
- **What:** DATA-DEPTH-1 — `str` or `==` of a value nested past the native stack killed the process on both engines,
  and a hand-written module's deep descriptor killed the WASM host; `display`, `eq` and `reify.rs` now walk worklists,
  and the descriptor parser is bounded. DATA-DEPTH-2 (RW 4.75) opened: the actor-message copy and the channel's
  serializing, not yet witnessed. Tests, mutants M777–M785 and the witnesses are in `V2_LOG.md`.
- Files and folders: M `crates/delulu-runtime/src/value.rs`, `crates/delulu-wasm/src/reify.rs`, `CHANGELOG.md`,
  `HANDOFF.md` (§11.4), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.74, 4.75), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`; A `crates/delulu-wasm/tests/deep_value_parity.rs`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — deep values and closures cross to actors (RW 4.75, 4.76) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/deep-messages` (on `laptop/deep-values` `f791ded`, PR #35)   Pull request: the one this entry rides in
- **What:** DATA-DEPTH-2's actor path (a 300,000-link chain sent to an actor killed the process) and CLOSURE-SEND-1 (a
  `let`-bound closure sent to an actor walked into itself) — each witnessed on the old binary and fixed; the root slice
  in a message boxed. Tests and mutants M786–M793 in `V2_LOG.md`.
- Files and folders: M `crates/delulu-runtime/src/{actors,value}.rs`, `crates/delulu-check/src/rcap_check.rs`,
  `crates/delulu/tests/actors_cli.rs`, `CHANGELOG.md`, `HANDOFF.md` (§11.4), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/REMAINING_WORK.md` (4.75, 4.76), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu/tests/deep_values_cli.rs`.
  Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — lists, maps and options nest on the WASM engine (RW 3.4, D-V2-173) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-nested-values` (on `laptop/deep-messages` `ec333c4`, PR #36)   Pull request: the one this entry rides in
- **What:** a slot holds a pointer to any list, map, option or result, so they nest on the WASM engine; five pins now
  list what stays outside. Also recorded here: RW 4.75's channel half is established unreachable — every capability
  method's argument is a fixed shallow type, and the host decodes each frame with `ciborium`'s recursion limit of 256.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs`,
  `crates/delulu-wasm/tests/{conformance_parity,list_parity,loop_parity,record_parity,value_parity}.rs`, `CHANGELOG.md`,
  `docs/for-agents.md`, `docs/DELULULANG_V2/{V2_DECISION_LOG,V2_LOG}.md`, `docs/REMAINING_WORK.md` (3.4, 4.75),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; A `crates/delulu-wasm/tests/nested_parity.rs`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — `--debug-rcaps` walks a deep graph (RW 4.75 closed) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/rcaps-depth` (on `laptop/wasm-nested-values` `8fb2867`, PR #37)   Pull request: the one this entry rides in
- **What:** the debug lane's uniqueness walk made iterative with paths kept as parent and step; witnessed (a 100,000-deep
  `iso` move spent the budget under the flag) and tested (the old walk overflows the stack). M798, M800 red.
- Files and folders: M `crates/delulu-runtime/src/actors.rs`, `crates/delulu-runtime/tests/actors_trace.rs`,
  `CHANGELOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.75), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — the cycle collector keeps what a map holds (GC-MAP-1, RW 4.77) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/gc-map` (on `laptop/rcaps-depth` `671677d`, PR #38; `master` `ec333c4` since #36 landed on its green run `39669053973`)   Pull request: the one this entry rides in
- **What:** GC-MAP-1 — the cycle collector's marking walk passed over maps, so a sweep emptied a live list held in an
  actor's map; witnessed on the old binary, fixed, its `match` made exhaustive over `Value`. Test in `V2_LOG.md`.
- Files and folders: M `crates/delulu-runtime/src/cycles.rs`, `crates/delulu/tests/actors_cycles_cli.rs`, `CHANGELOG.md`,
  `HANDOFF.md` (§11.4), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.77), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — a cycle through a map is collected (GC-MAP-1's residual) (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/gc-map-cycles` (on `laptop/gc-map` `8a88466`, PR #39)   Pull request: the one this entry rides in
- **What:** maps registered with the cycle collector and broken like lists (200 map cycles in a turn: 0 sweeps before,
  1 sweep and 200 collected after); `note_value` exhaustive over `Value`. Also: `launch/2.0.0` = `3d4c2c2` (a merge of
  master ..`8fb2867`, the ten gates 75 passed; pushed, not yet dispatched — the merge after #38–#40 will be).
- Files and folders: M `crates/delulu-runtime/src/cycles.rs`, `crates/delulu/tests/actors_cycles_cli.rs`, `CHANGELOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (4.77), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.
- Also in this branch: RW 7.17 records the next red's own words — `estop_cli`'s actor control on `windows-latest` (PR #38's
  run `39672085470`) lost its arm because the daemon accepted a liveness probe and did not answer within DEADMAN-1's 1 s;
  RW 4.53's measured 1.077 s maximum beside busy writers is past that deadline. The fix direction is recorded, not taken.

### 2026-10-22 (night) — `launch/2.0.0` brought up to `master` twice; RW 7.17's fix direction corrected (made on the laptop)
- Session: `https://claude.ai/code/session_01GMTbobfi6fosrVt6Qfpkfb`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/records-1022` (on `master` `35d8208`, where #40 landed on its green run `39675143583`; #38 and #39 landed
  together at `8a88466` on #39's green run `39672866524`, #38's own rerun `39672085470` attempt 2 also green; `master`'s push
  runs `39669143464` (`f791ded`), `39672300209` (`ec333c4`) and `39673797633` (`8fb2867`) **success**)   Pull request: the one this entry rides in
- **`launch/2.0.0`:** `3d4c2c2` merged `master` ..`8fb2867` (#33–#37; the four new sections under `## [2.0.0]`; the stricter
  DL0407 added to 2.0.0's **Breaking** list — a program that checked under 1.x can be refused) and `7b5aec3` merged
  ..`35d8208` (#38–#40). Each: Survey rebuilt, doctor ok, clippy stamped, the ten gates 75 passed. Dispatched on `7b5aec3`:
  CI `jobs=everything` `39678647200` and the release dry run `39678652131` — running at this commit; the next master commit
  records them.
- **What (master):** RW 7.17's two fix directions marked unsafe as stated — an off-queue liveness answer would hide a stuck
  handler from the e-stop; the next step is a measurement.
- Files and folders: M `docs/REMAINING_WORK.md` (7.17), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed:
  nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (evening) — the exhaustiveness limit's test fits Miri (`miri-slow` timed out twice) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/miri-exhaust-budget` (on `master` `37b3e4c`, where PR #41 landed on its green run `39678800260`)   Pull request: #42
- **CI read since the last entry:** `master`'s push run `39678411414` (`35d8208`) **success**; on `launch/2.0.0` `7b5aec3`
  the release dry run `39678652131` **success** and CI `jobs=everything` `39678647200` **cancelled** — every job passed
  except `miri-slow (delulu-check)`, stopped at its 240 minutes; `master`'s nightly `39746314492` (`35d8208`) the same.
- **What:** that job's time went to one test spending the checker's 10,000,000-cell search limit (11,873 s); the limit
  is ten thousand under Miri and the test is sized to it. Evidence in `V2_LOG.md`.
- **`autosave/laptop`** (read at the owner's word): its newest snapshot `838c739` (2026-10-21 20:43 UTC, on
  `laptop/gc-map`) is tree-identical to `8a88466`, the commit that landed it — nothing to recover. The task is `Ready`,
  last result 0, renewed to 2026-10-28. All 46 local branches' commits are on `origin`.
- Files and folders: M `crates/delulu-check/src/check.rs`, `crates/delulu-check/src/lib.rs`, `HANDOFF.md` (§11.5),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (2.20), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (evening) — autosave pushes what origin lacks, not what an upstream lacks (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/autosave-push` (on `laptop/miri-exhaust-budget` `1f44f55`, PR #42)   Pull request: the one this entry rides in
- **What:** `scripts/autosave.ps1`'s push step asked each branch's upstream: a branch with none (every
  `git switch -c X master`) was skipped, so its unpushed commit stayed on the disk alone; a `master` ahead was pushed onto
  `origin`'s `master`. Now a branch with a commit `origin` lacks is pushed under its own name; `master`/`main` go to
  `autosave/laptop-master`/`-main`; an orphan is refused; `-DryRun` added. Witnessed and checked on throwaway origins
  (`V2_LOG.md`). Then, at the owner's word, every linked worktree is snapshotted too — `D:\nelan\dl-launch` to
  `autosave/laptop-dl-launch`. The task was re-installed from this branch's script (the owner asked for it live), and
  `master` push run `39804641086` (`37b3e4c`, #41) read **success**.
- Files and folders: M `scripts/autosave.ps1`, `HANDOFF.md` (§11.1, autosave), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (evening) — the broker log names a late answer (RW 7.17's instrument) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/broker-slow-answer` (on `laptop/autosave-push` `e8b3be4`, PR #43)   Pull request: the one this entry rides in
- **What:** the daemon names in `broker.log` each answer given 500 ms or more after its request arrived (kind, queue
  wait, own time); `ReqBody::kind`; `estop_cli`'s no-revoke controls attach `broker.log` to a failure and print late
  answers on a green run (the measurement: `witness.yml` on `windows-latest` at `estop_cli`).
- Files and folders: M `crates/delulu/src/broker_ipc.rs`, `crates/delulu/src/brokerd.rs`,
  `crates/delulu/tests/broker_cli.rs`, `crates/delulu/tests/estop_cli.rs`, `CHANGELOG.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/REMAINING_WORK.md` (7.17), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (evening) — a grant naming a network share is warned of on Windows (RW 4.17 closed) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/unc-grant-warning` (on `laptop/broker-slow-answer` `86b8df5`, PR #44)   Pull request: the one this entry rides in
- **CI read:** PR #42's runs `39805379666` and `39806203436` **success**; PR #43's `39807030815` (`a1ab1d1`) and
  `39808528500` (`e8b3be4`) **success**; `witness.yml` on `windows-latest` at `86b8df5`, `39810107555` and `39810113455`,
  **success** — RW 7.17's measurement: no answer late in four no-revoke control runs.
- **What:** on Windows an `fs.read`/`fs.write`/`plugin` grant naming `\\server\share` is warned of once, naming the server
  and the NTLM exposure; the run goes on (`prim::unc_server`, `cli::warn_on_unc_grant`). RW 1.11's stale "not edited"
  corrected (the constitution was corrected 2026-10-17, D-V2-141). Then seven `REMAINING_WORK.md` rows wrapped across
  lines (two by this night's edits, five older) joined, words unchanged, and gated (`repository_structure.rs`
  `every_remaining_work_row_is_one_line`, red before) — M `crates/delulu/tests/repository_structure.rs` too.
- Files and folders: M `crates/delulu-runtime/src/prim.rs`, `crates/delulu/src/cli.rs`, `crates/delulu/tests/cli.rs`,
  `CHANGELOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (1.11, 4.17, 7.17), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (evening) — `broker stop` waits for the daemon to exit (RW 7.19 closed) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/broker-stop-waits` (on `laptop/unc-grant-warning` `a47516c`, PR #45)   Pull request: the one this entry rides in
- **What:** `broker stop` returns once the daemon's process is gone (5 s bound), not when it answers — the cause of
  RW 7.19, witnessed (OS error 32 removing the state directory at once on the old code). `ok: broker stopped`.
- Files and folders: M `crates/delulu/src/brokerd.rs`, `crates/delulu/tests/broker_cli.rs`, `CHANGELOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (7.19), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (evening) — a package builds to one `.dwx` (RW 3.4) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-package-build` (on `master` `41e9724`)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** #42–#44 by fast-forward to `86b8df5` on their green runs (the `miri-slow`
  dispatch `39805331281`: `delulu-check` **success in 54 minutes**, the pigeonhole test 78 s where it had been 11,873 s);
  #45–#46 to `41e9724` on #46's green `39813045667` (#45's own `39811689314` and `39812124486` green too); `master`'s push
  run `39814780780` (`86b8df5`) **success**. `launch/2.0.0` = `7e7b2ed` (merge of master ..`41e9724`; three CHANGELOG
  sections under 2.0.0; Survey 1554/16104; doctor ok; clippy stamped; the ten gates 77 passed); its release dry run
  `39816924661` **success**, its `jobs=everything` `39816919025` running.
- **What:** `build <dir> --target wasm` writes a package's `.dwx` (the program `run <dir>` runs; the manifest's scopes;
  `target/<name>/<name>.dwx`) — the corpus counter 14 of 17 as programs. Also: three entries of this log had been
  scrambled by tonight's inserts (each new one placed before the previous one's file list); the file lists moved back,
  no line changed.
- Files and folders: M `crates/delulu/src/cli.rs`, `crates/delulu/src/run_cmd.rs`, `crates/delulu/tests/package_run.rs`,
  `CHANGELOG.md`, `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (3.4), `docs/CLOUD_SYNC_LOG.md`,
  `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — a WASM program holds a secret and sends it (RW 3.4, D-V2-174) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-secret-handle` (on `laptop/wasm-package-build` `0344884`, PR #47)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** `master`'s push run `39816526461` (`41e9724`) **success**; PR #47's run
  `39818530211` **success** (14 jobs, 2 skipped), so #47 landed by fast-forward to `0344884`, and `master`'s push run
  `39821304180` for it **success**.
- **What:** in a WASM program a secret is a handle in the host's table (D-V2-174, the Stage 3 spec's §4.4 built).
  `root.secret`, `get_with_secret` (sent from the host by the interpreter's own code: audience, custody, broker `send`)
  and `verify` compile. `expose`, `root.declassify()` and `Secret.map` stay DL1205. `broker_secret_send` and
  `secret_verify` are now shared by both engines. `demo` and chapter five build: the corpus counter is 15 of 17.
- Files and folders: M `crates/delulu-runtime/src/interp.rs`, `crates/delulu-wasm/src/codegen.rs`,
  `crates/delulu-wasm/src/host.rs`, `crates/delulu-wasm/src/lib.rs`, `crates/delulu/src/run_cmd.rs`,
  `crates/delulu-diag/src/codes.rs` (DL1205's explain), `crates/delulu/tests/broker_cli.rs`,
  `crates/delulu/tests/egress_cli.rs`, `crates/delulu/tests/wasm_cli.rs`, `CHANGELOG.md`, `HANDOFF.md` (§11.5),
  `docs/book/THE_DELULULANG_BOOK.md`, `docs/design/STAGE3_SPECIFICATION.md` (a dated note under Phase 3n),
  `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-174), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (3.4),
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`; and in the PR's second commit, after its first run (`39825874508`) was red on
  `test (ubuntu-24.04)`: `tests/conformance/witnesses.toml` (entrenched: the DL1205 witness renamed) and
  `docs/design/ENTRENCHED_CHANGE_RECORD.md`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-22 (night) — Delulu Core's Theorems 1–3 machine-checked for the monomorphic fragment (RW 5.1, D-V2-175) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/core-mechanized` (on `laptop/wasm-secret-handle` `4af166e`, PR #48)   Pull request: the one this entry rides in
- **Read since the last entry:** the launch `jobs=everything` run `39816919025` on `launch/2.0.0` `7e7b2ed`: **red on
  `test (windows-latest)`**, three wall-clock tests on one runner. `monitor_cli` `a_long_chain_does_not_kill_a_healthy_monitor`
  ("broker daemon did not come up within 5s"); `sandbox_devices_cli` `a_guest_that_stops_beating_loses_its_actuator_on_the_hosts_clock`
  ("the guest said nothing for 60s"); `channel::tests::from_now_counts_the_wait_before_the_first_byte` (an in-memory frame
  not read within 100 ms). The same code was green on Windows in `master`'s push runs. To be re-run when the run ends.
- **What:** `docs/design/models/lean/DeluluCoreSafety.lean` proves progress-or-fault, preservation, effect soundness,
  safety, attenuation's monotonicity and no-forgery for the monomorphic, closed-row fragment (Lean 4.32.2). CI checks
  it, fails on any `sorry`, and requires three mutants to fail. `DELULU_CORE.md` v0.4 (entrenched, recorded): the
  honesty clause, `T-Verify` with `Declassify` as implemented, `E-Expose` and `E-Verify` written, `narrow` stated, Fact B and Theorem 3 naming `verify`'s label as the one
  no capability witnesses (RW 2.6).
- Files and folders: A `docs/design/models/lean/DeluluCoreSafety.lean`; M `.github/workflows/ci.yml` (the `formal` job),
  `docs/design/DELULU_CORE.md`, `docs/design/ENTRENCHED_CHANGE_RECORD.md`, `docs/design/models/README.md`,
  `docs/MATHEMATICS.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md` (5.1, 2.6), `CHANGELOG.md`,
  `HANDOFF.md` (§11.5), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-175), `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-23 (small hours) — `deploy plan` checks every request against the profile (RW 6.6, D-V2-176); DEPLOY-PKG-1 (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/deploy-plan-scopes` (on `laptop/core-mechanized` `b8c4fde`, PR #49)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** PR #48's second run `39827827803` **success** (14 jobs, 2 skipped), so #48 landed
  by fast-forward, master `4af166e` (push run `39831374181`). The `miri-slow` dispatch `39805331281` finished **success**,
  every job, the broker's included.
- **What:** `deploy plan` decides each service's requests against the profile's boundary by `authority --within`'s verdict,
  refusing the whole plan on any request outside it or any computed scope, named at the call (DL1909). `deploy plan`,
  `publish` and `add` check a package with dependencies whole (DEPLOY-PKG-1: they had kept the single-package loader).
  Breaking for a profile that named effects only.
- Files and folders: M `crates/delulu/src/deploy.rs`, `crates/delulu/src/cli.rs` (`checked_package`; the help line),
  `crates/delulu-diag/src/codes.rs` (DL1909's explain), `crates/delulu/tests/deploy_cli.rs`,
  `crates/delulu/tests/deploy_plan_adversarial.rs`, `crates/delulu/tests/add_path_cli.rs`, `CHANGELOG.md`, `HANDOFF.md`
  (§11.5), `docs/design/STAGE10_BUILD_ORDER.md` (a dated note under C45), `docs/DELULULANG_V2/V2_DECISION_LOG.md`
  (D-V2-176), `docs/DELULULANG_V2/V2_LOG.md`, `docs/REMAINING_WORK.md` (6.6), `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`.
  Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-23 (early morning) — CI read to its end; `a_monitor_sees_nothing_outside_its_own_subtree` says why (RW 7.23) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/monitor-witness-says-why` (on `master` `05909ef`)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** PR #50's run `39833210704` **success**, so #50 landed by fast-forward, master
  `05909ef` (push runs `39836740749`, `39836740781`: **success**). `39831374181` (`4af166e`): **success**. The launch run
  `39816919025`: **success** on attempt 2 of its Windows job. `39830724209`'s `editor` job: read `completed success` at
  03:55 IST (it had lagged the run). `39833849066` (`b8c4fde`): **failure**, one Windows test, RW 7.23.
- **What:** the failing test's messages carry the run's stderr and the monitor's report; RW 7.23 records the four Windows
  wall-clock failures of 2026-10-22.
- Files and folders: M `crates/delulu/tests/monitor_cli.rs`, `docs/REMAINING_WORK.md` (7.23),
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.

### 2026-10-23 (morning) — a WASM slot holds a capability or a secret (RW 3.4, D-V2-177) (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/wasm-handles` (on `master` `05909ef`; the parked `laptop/wasm-handles-in-slots` `5373158` folded in, not merged)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** `launch/2.0.0` = `e51b4f3` (master merged in, #47-#50); its release dry run `39865371700` **success**; its everything run `39865367900` was running. The Windows witness of RW 7.23 (a), `39864919206`: **success**. PR #51's run `39864920827`: **success**, every job, so #51 landed by fast-forward, master `c153433`; merged into this branch, its log entries kept first.
- **What:** a list, an option, a record field and a map's value hold a capability or a secret as its handle on WASM; `checked_ty` maps the handle types.
- Files and folders: M `crates/delulu-wasm/src/codegen.rs`, `crates/delulu-wasm/tests/list_parity.rs`, `crates/delulu-wasm/tests/record_parity.rs`, `CHANGELOG.md`, `docs/for-agents.md`, `docs/book/THE_DELULULANG_BOOK.md` (chapter 9), `docs/REMAINING_WORK.md` (3.4), `docs/DELULULANG_V2/V2_DECISION_LOG.md` (D-V2-177), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added: `crates/delulu-wasm/tests/handle_parity.rs`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-23 (morning) — RW 7.23 (b) root-caused: a test reader's zero sleep is a time slice on Windows (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/channel-control-busy-runner` (on `master` `36740a4`)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** PR #52's run `39867706347` **success**, so #52 landed by fast-forward, master `36740a4`. `launch/2.0.0`'s release dry run `39865371700` **success**; its everything run `39865367900` red on `test (windows-latest)`, one test (RW 7.23 (c)).
- **What:** `channel.rs`'s test reader no longer sleeps a zero gap; RW 7.23 (b)'s `channel` test root-caused; (c) recorded; a trap in `HANDOFF.md` §11.5.
- Files and folders: M `crates/delulu-runtime/src/channel.rs`, `crates/delulu/tests/plugin_load_cli.rs` (RW 7.23 (c) says why), `HANDOFF.md` (§11.5), `docs/REMAINING_WORK.md` (7.23), `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-23 (morning) — the launch branch takes #51-#53; the phase status catches up (made on the laptop)
- Session: `https://claude.ai/code/session_01PPQFYhLFcNmJrr9QuWpyzx`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/launch-records` (on `master` `2349cff`)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** PR #53's run `39872356478` **success**, so #53 landed by fast-forward, master
  `2349cff`. `launch/2.0.0` = **`0ef515f`**: master `05909ef..2349cff` merged in (#51-#53; one CHANGELOG section moved
  under 2.0.0; the Survey rebuilt there, 1556 nodes, 16327 edges; clippy stamped; doctor ok; the gates green). Its release
  dry run `39874335998` **success**; its `jobs=everything` `39874332895` **success** on 16 of 18 jobs, `test
  (windows-latest)` and `heavy-gates` included, with `miri-slow` (delulu-check) and (delulu-broker) running. `e51b4f3`'s
  `everything` `39865367900`: red on `test (windows-latest)`, RW 7.23 (c); its `miri-slow` (delulu-check) and (delulu-syntax) **success**, (delulu-broker) running.
- **What:** `V2_PHASE_STATUS.md`'s REL row names the ten launch merges of 2026-10-21 to 2026-10-23.
- Files and folders: M `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added,
  deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-23 (morning) — a record's width no longer costs per field (RW 2.10, D-V2-178) (made on the laptop)
- Session: `https://claude.ai/code/session_01E8XZYvoqTfgpQSgPDMfdcn`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/record-width` (on `master` `3334f93`)   Pull request: the one this entry rides in
- **Read since the last entry:** `launch/2.0.0` `0ef515f`'s `jobs=everything` `39874332895` **success, 18 of 18 jobs**
  (both `miri-slow` jobs included); `e51b4f3`'s `39865367900`: `miri-slow` (delulu-broker) **success**, the run red only on
  RW 7.23 (c). Autosave: ran 09:31 IST, result 0; snapshot `c7eecb2` = `2349cff`'s tree, nothing to recover.
- **What:** the interpreter finds a field of a record wider than 16 fields at its declared place, checked against the
  name held there, and puts a literal in declared order by one lookup per field. A 1,600-field literal written out of
  order had cost 39 ms (quadratic, since RECORD-ORDER-1); reads of the last of 1,600 fields cost 4x a 4-field record's.
  Both now flat (`measurements/scale/RECORD.md`).
- Files and folders: A `crates/delulu-runtime/tests/record_width.rs`. M `crates/delulu-runtime/src/interp.rs`,
  `CHANGELOG.md`, `HANDOFF.md`, `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`,
  `docs/DELULULANG_V2/V2_LOG.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/design/HARDENING_CAMPAIGN.md`,
  `docs/design/AUTHORITY_GUARD_CAPSTONE.md`, `measurements/scale/RECORD.md`, `docs/survey/*`. Deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

### 2026-10-23 (morning) — a shared type is checked once; one too large to write out is refused (RW 2.3, D-V2-179; OPAQUE-FIELD-2) (made on the laptop)
- Session: `https://claude.ai/code/session_01E8XZYvoqTfgpQSgPDMfdcn`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/type-size` (on `master` `118cf51`)   Pull request: the one this entry rides in
- **Landed and read since the last entry:** PR #55's run `39900420977` **success**; #55 landed by fast-forward, master `118cf51`.
- **What:** the occurs check, unification and `==`'s equality walk follow each binding once; a type is written out within
  100,000 parts (2,000,000 in all for large ones), past which `check` refuses with DL0211 (language-visible). 22 lines of
  self-paired values took 5.8 s and doubled per line; any depth is now refused in about 0.1 s. A record holding a user
  generic of an opaque type is refused under `==`/`str` (OPAQUE-FIELD-2, RW 2.21).
- Files and folders: A `crates/delulu-check/tests/type_size.rs`. M `crates/delulu-check/src/check.rs`,
  `crates/delulu-check/src/unify.rs`, `crates/delulu-check/src/lib.rs`, `crates/delulu-diag/src/codes.rs`, `CHANGELOG.md`,
  `HANDOFF.md`, `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/book/THE_DELULULANG_BOOK.md`, `docs/for-agents.md`, `docs/survey/*`. Deleted, renamed:
  nothing.

### 2026-10-23 (afternoon) — a sent secret is asked for and decided with its audience (RW 2.11, D-V2-180) (made on the laptop)
- Session: `https://claude.ai/code/session_01E8XZYvoqTfgpQSgPDMfdcn`   Model: Claude Opus 5.5 (the laptop; no fallback notice; no sous-chef)
- Branch: `laptop/secret-audience` (on `master` `118cf51`)   Pull request: the one this entry rides in
- **What:** `authority --grants` names a sent secret's audience (`secret:NAME@HOST:PORT=VALUE`); `--within` and
  `deploy plan` decide each send against the boundary's audience (stricter: they used to approve a boundary under which
  the run refuses the send); a `deploy plan` profile's `secrets` may name an audience. D-V2-181: `--within` and
  `deploy plan` decide each request by its path against the boundary's `net` (stricter the same way).
- Files and folders: M `crates/delulu/src/cli.rs`, `crates/delulu/src/within.rs`, `crates/delulu/src/deploy.rs`,
  `crates/delulu/tests/within_cli.rs`, `crates/delulu/tests/deploy_cli.rs`, `crates/delulu/tests/deploy_plan_adversarial.rs`
  (its fixture's request `http://` → `https://`), `CHANGELOG.md`, `docs/for-agents.md`,
  `docs/REMAINING_WORK.md`, `docs/DELULULANG_V2/V2_DECISION_LOG.md`, `docs/DELULULANG_V2/V2_LOG.md`,
  `docs/CLOUD_SYNC_LOG.md`, `docs/survey/*`. Added, deleted, renamed: nothing.
- Survey and doctor (after the last edit): in the commit message.

