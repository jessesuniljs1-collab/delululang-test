---
name: cloud-period-2026-09-28
description: "Jesse's laptop is away 2026-09-28 → 2026-10-16; DeluluLang work runs in Claude Code cloud sessions via PRs, every change recorded in docs/CLOUD_SYNC_LOG.md for a later sync"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-09-28T05:57:50.018Z
---

From **2026-09-28 to 2026-10-16** Jesse uses the laptop for other work; DeluluLang is developed from
**Claude Code cloud sessions** (claude.ai/code / app / `claude --cloud`) on the GitHub testing repo,
which is already connected. The laptop checkout `D:\nelan\DeluluLang` is synced afterwards.

- **Run and check everything using the Survey and doctor** (Jesse, 2026-09-28): `survey check` + `doctor --check` at every session start; `survey impact/affected-by/query` before a change; `survey build/check/findings` + `doctor --check` after the last edit and before every commit/PR, results in the PR and the sync-log entry (table in AGENTS.md; HANDOFF §0 rule 6).
- **Baseline = the newest `Cloud handoff` commit** (`git log -1 --grep='^Cloud handoff'`): `d41e558`, then `bc9192c`, then `937aea8`, `7e67f97` (routine + delegation), then `7bd018c`, then `5bb39bc` (Cloud handoff (6), the mandate), then **`Cloud handoff (7)` = THE BASELINE** (2026-09-28 evening, pushed from the laptop after it fast-forwarded to the cloud's `dd543e5`: the OpenShell study). **ROUTINE `trig_01GG41tCGxZXyt7bZUid8Vuu`** (https://claude.ai/code/routines/trig_01GG41tCGxZXyt7bZUid8Vuu): Opus 5.5, cron `7 */5 * * *` UTC (IST 05:37/10:37/15:37/20:37/01:37), env `env_012Mxeiw6zvfyxPy9ZEEFf3p`, tools Bash/Read/Write/Edit/Glob/Grep/Agent/WebFetch/WebSearch, NO connectors (cleared — the API attached all by default), prompt = follow docs/CLOUD_ROUTINE.md + 4 never-rules + stop on 2026-10-16. First run started by hand 07:11Z: session `cse_013AZJ6RqYq59CeMgkvLV1BM`. Debug with RemoteTrigger list_runs / get_run_log. Earlier: `937aea8` ("Cloud handoff (3)": the live docs brought up to date — V2 folder, REMAINING_WORK 6.5 closed + 4.24 new, DEPLOYMENT, for-agents, GETTING_STARTED, QUESTIONS, MATHEMATICS, the Book) ("Cloud handoff (2)", pushed 2026-09-28; laptop clean, local == origin). Earlier wording: **`d41e558`** ("Cloud handoff: AGENTS.md, CLAUDE.md, the sync log, the memory snapshot", pushed 2026-09-28, laptop clean, nothing unpushed) = the commit that added `docs/CLOUD_SYNC_LOG.md`. At handoff every run was read except miri-slow `36381950975` on `047da1d` (broker + check still running) and `d41e558`'s own push run.
- **Every cloud session appends an entry to `docs/CLOUD_SYNC_LOG.md`** (Jesse: "keep a record of files
  and folders changed, to be sync with local repo later") — commits + `git diff --name-status`,
  what was verified, what to redo on the laptop, what the laptop memory must learn.
- **Cloud facts (official docs, read 2026-09-28):** Ubuntu 24.04 x86-64, 4 vCPU / 16 GB / 30 GB;
  Rust, Python, GCC/Clang, Docker, `gh` preinstalled; crates.io + static.rust-lang.org allowed
  (Trusted); `git push` ONLY to the session's own branch → PR into master, Jesse merges; auto memory
  is machine-local (NOT in cloud) → `HANDOFF.md` §11 and `docs/assistant-memory/` (a sanitized
  snapshot of this directory) are the cloud's memory; CLAUDE.md must `@AGENTS.md` for both to load.
- **DELEGATION (Jesse, 2026-09-28, later):** "I want the development of delululang to be continued in my absentia. No need to wait for any of my input, Claude u can take better decisions than me on delululang. Run verification loops and loop engineering." → every decision is the head chef's until 2026-10-16 EXCEPT the five reserved by name (final public repo, licence, entrenched files, D-NE-27, D-NE-7 no tag/release); the morning "stop before PS-D-02" is SUPERSEDED. Engine = a Claude Code ROUTINE (cloud, laptop off, ≥1 h interval, daily run cap) whose every run follows `docs/CLOUD_ROUTINE.md` and pushes to `master` (unprotected; all commits Jesse's — verified 2026-09-28). /loop (needs open session, 7-day expiry) and Desktop tasks (need the machine on) do not fit.
- **MANDATE (Jesse, 2026-09-28 noon, creating the routine):** Opus 5.5 at xhigh (`.claude/settings.json` effortLevel), every 5 h until Oct 16; start PS-D-02; FINISH ALL PHASES AND VERIFY (P7 incl. RW 5.2 entrenched edit, P8 software part); ANY FILE OR FOLDER may be created/modified/DELETED (entrenched edits → ENTRENCHED_CHANGE_RECORD.md; deletions named in the sync log); then keep improving + verifying; loop engineering = CLOUD_ROUTINE step 8. STILL his: final public repo, licence, D-NE-7 (tags/releases), D-NE-27. Routines already run with no permission prompts; I did NOT commit a bypassPermissions setting into the public repo (it would apply to anyone who clones it).
- **Same day:** "run everything on github" (heavy runs on CI, not the laptop — it ran out of RAM and
  Claude Code reaped the suite and a Miri run); "stop before PS-D-02" (its design draft is in
  `HANDOFF.md` §0).
- **ROUTINE RUN 1 (2026-09-28, 07:11 UTC) learned:** the cloud VM has NO `gh` and the proxy refuses the
  signed log URLs → read CI with the GitHub MCP tools (`actions_list`, `actions_get`, `get_job_logs`
  `failed_only`), a Haiku sous-chef to pull lines out of a long log; run `cargo fetch --locked` before
  the suite (`egress_features` runs `cargo metadata --offline`); Survey test nodes are `test:<path>`.
  `actors_pingpong` went red on Windows twice at 1.31x — starving the VM reproduced it; D-V2-47 gave it a
  control and CI now prints its verdict. PS-D-02 built (D-V2-48): `--require-attestation HEX`,
  `delulu sandbox attest`; `delulu` reads nothing after a bare `--`. Also: PS-D and P7 COMPLETE;
  ADAPTER-SPELL-1 and ATTEST-FIFO-1 found and fixed; DELULU_CORE v0.3 (D-V2-49, entrenched — owner to
  review); P8 designed (D-V2-51); RW 7.4 closed (VS Code via apt from packages.microsoft.com); Docker
  Hub refuses the VM's pulls → `container.yml` builds on GitHub; the MCP log tool returns only a job's
  last 5,000 lines.
- **Runs 2-4 (10:11, 10:12, 10:44 UTC) did nothing:** each ended in seconds on the account's five-hour
  usage limit (`rate_limit: rejected (five_hour)`) — run 1 had spent it. Jesse paused the routine at
  14:30 UTC. Routine runs share his usage; a run that starts on an exhausted window is wasted.
- **2026-09-28 evening, on the laptop:** synced (ff `5bb39bc..dd543e5`, no CRLF, Survey ok 1,462 nodes,
  findings 0 errors, doctor 29/29 on Windows; full suite NOT run locally — CI covered every OS); then
  **NVIDIA OpenShell studied** on Jesse's commission → [[openshell-study-2026-09-28]]; PS-E is next,
  then P8 (+P8-04), then P9; the routine resumed with its connectors cleared again.
- **Models, 2026-09-28 evening (official models page):** Sonnet 5.5 (`claude-sonnet-5-5`) RELEASED; Haiku 5.5 announced, NOT released (Haiku 4.5 still newest; retirement not sooner than 2026-10-15). Agents (Jesse's ruling): Sonnet 5.5; Haiku 5.5 once released; no Haiku 4.5 meanwhile. Opus 5.5 defaults to `medium` in Claude Code — repo `effortLevel: xhigh` raises it. A cyber-flagged request re-runs Opus 5.5 → Opus 4.8 and the session STAYS there: the trailer must name it. Cloud VM: `gh` listed as pre-installed in docs but run 1 found none — check; release assets only from the attached repo.
- **ROUTINE RUN 2 (2026-09-28, 20:09 UTC,** session `session_01TfVRPwf7BAv6L8SzocuB1d`**):** pushing to
  `master` worked from the routine's session (its harness names a `claude/…` branch; the routine's brief
  wins). CI read green (`045c21a`, `7e9d97d`, `cfbfbdc`). Workflows moved to the Node-24 majors (each
  checked from its `action.yml` on raw.githubusercontent.com, reachable from the VM) and pinned to
  `ubuntu-24.04`. **PS-E-01 first step (D-V2-56):** the host's first channel frame WAS the program —
  witnessed red with a capturing launcher — now `delulu-sandbox-channel/3`, `boundary.rs` typestate, a
  generation per run. Traps: sandbox audit records need `<state>/audit` to exist; `sh` background jobs
  read `/dev/null`. **PS-E-01 second step (D-V2-57):** `sandbox.properties` reported; CI measured Linux
  x86/arm64/Windows all five, macOS egress + privilege only. **Red-team pass (Sonnet 5.5, frozen binary
  copy):** guarantee held; seven defects around it, verified and fixed (D-V2-58): GUEST-WAIT-1,
  RAN-SENT-1, GUEST-TEXT-1, PIPE-WRITE-1, PIPE-FLOOD-1, F8, F10; F7 (L3 self-report as host
  guarantee) → RW 4.31. A racing witness let mutant M8 live — fake peers read before they hang up.
- **Routine run 3 (2026-09-29):** `witness.yml` (one test, one runner, any ref — CI-only witnesses
  red on a branch before `master`) and `scripts/check-other-os.sh` (clippy for macOS and Windows from Linux,
  a stand-in C compiler; libffi-sys's build script replaced through its `links` key). PS-E-02 on macOS: a computing guest outlived a killed host (red on a runner);
  a watcher OUTSIDE the guest (`__host_watch`, kqueue on the host's pipe and the guest's exit) ends it
  and the external launcher (D-V2-60) — not a thread in the guest, which an escaped guest could stop.
- **Routine run 3, later:** PS-E-02 complete (Windows launcher: kill-on-close job); PS-E-03's escaped-guest
  harness (`jail::escaped_tests`) confirmed H1–H5 and found H7–H10 (terminal read, TIOCSTI keystrokes,
  signals, other processes) — all closed with mutants (D-V2-61–D-V2-66); `master` red once on macOS (a raw
  `Broken pipe` a timing change exposed), fixed at its cause.
- **Routine run 4 (2026-09-29):** `master` red on a records-only commit — new Wasmtime advisories
  (RUSTSEC-2026-0315/0316; wasmtime 48.0.3, the LTS line, D-V2-67) and a macOS race in a TEST's fake
  attester (a plain `cp` where the protocol asks for write-then-rename); both witnessed in the VM and fixed;
  an attestation read half-written is now refused as `incomplete`, in words. Then RW 4.33 (D-V2-68): the
  plugin store had RUN a `call_ref` module — both WASM engines now refuse the WebAssembly 3.0 proposals and
  keep the 2.0 set; `scripts/check-other-os.sh` lints `delulu-runtime` and `delulu-wasm` too.
- **Routine run 5 (2026-09-29):** PS-E-04 — the external launcher resolved once (`resolve_driver`,
  shared with ADAPTER-SPELL-1's fix), hashed with BLAKE3, reported (`launcher_path`, `launcher_blake3`),
  pinnable (`--launcher-digest`), and on Linux started as the descriptor hashed (`fexecve`); LAUNCHER-SPELL-1
  (a planted `./lnch` ran through `.` on `PATH`) witnessed and closed (D-V2-69). Trap: `environ` inside a
  `pre_exec` step is not the command's environment. Then TERMINAL-TEXT-1 (D-V2-70): a pure program's
  strings (assert_eq's values, refused paths, test names, quoted lines) reached the terminal as escape
  sequences — escaped at the printers now; JSON unchanged. Then RW 4.32's stream (D-V2-71): the host relays
  a guest's standard error — `guest:`/`launcher:`, escaped, 1 MiB — and the microVM console escaped; the
  death record names the guest's words; RW 4.34 (D-V2-72): a message's continuation lines indented.
  `master` went red on Windows once, for a TEST whose path held `:` — rule: a slice's new tests are read on
  every runner first. A classifier stopped the start of H6 (a macOS escaped-guest harness) — nothing ran.
- **Routine run 6 (2026-09-30):** RW 4.32's per-frame deadline — and FRAME-DRIP-1 on two more channels: the
  broker daemon (a dribbling client held its one-connection loop, the e-stop revoke behind it; IPC-1's fix had
  bounded each read only) and a foreign call (foreign code dripping its reply). `channel::Within` owes each
  frame whole (D-V2-73). Traps: a per-read deadline is not a per-message one — test the peer answering just
  inside it; install the pinned toolchain before any concurrent `rustup`/`cargo` (a race broke it). Then PS-E-04's
  Windows window (D-V2-74): the launcher held open sharing reads only until the run ends — red on a Windows runner
  first, the swap timed by `FileProcessIdsUsingFileInformation` (Windows' view of who holds a file open).
- **Routine run 7 (2026-09-30):** PS-E-06 complete — `delulu audit export --format ocsf` / `audit verify --ocsf`,
  each event carrying its whole record (D-V2-78), and a use's record naming its effect (D-V2-80); OCSF's schema read
  at run time (`scripts/ocsf-validate.py --self-test`, `ocsf.yml`) — its raw files ARE reachable from the VM.
  AUDIT-TEXT-1 (D-V2-79): `audit tail`/`query`, `guard pending` and `grants tree`/`list`/`inspect` printed a
  program's or an agent's stored strings raw — a string that is stored is printed later; ask where. The secret store
  had AUDIT-FIFO-1's shape (RW 4.38 closed). Trap, again: a fixture faked the host's generation as an integer.
- **Routine run 8 (2026-09-30):** SCOPE-HIDDEN-1 (D-V2-81): `authority --grants` under-reported — a `Root` passed to a
  helper as `r` hid its scope (the walk matched `root` by name), and one literal of a kind silenced the placeholder for
  every other site. PS-E-05 (a) (D-V2-82): `sandbox policy --format openshell` — the OpenShell policy written from the
  program's authority, never wider than it and the grants; OpenShell 0.1.2's prover checks it (`within_boundary`, six
  widenings `exceeds`) and a real OpenShell sandbox ENFORCES it on a runner (`openshell.yml` `runtime` job: the program
  runs inside, `curl` to an unlisted host `NET:OPEN … DENIED`). E-05 (b) open: the guest cannot run nested — its own
  `seccomp` is refused inside OpenShell, so it fails closed. Traps: a mutant loop leaves the last mutant's binary; a new
  workflow is dispatchable only once on `master`; pick falsifiers from the checker's documented cases; a flag documented
  for a command reaches every verb; the REST API answers `curl` from the VM (wait on runs in the background).
- **Routine runs 9 and 10 (2026-09-30, 2026-10-04):** run 9 built PS-E-05 (b)'s outer-wall declaration (D-V2-83) and
  ended with it on its harness branch alone, unrecorded; run 10 found it by listing every branch, merged it, and fixed
  what OpenShell's runner read (the wall hides `/proc`: the in-force check is `PR_GET_SECCOMP`). Run 10 also fixed four
  red nightlies (a cache Rust 1.99.0 emptied; Wasmtime 48.0.5, D-V2-84) and measured OpenShell's relay: `sandbox exec`
  runs a command only once its input ends; `ssh` via `openshell ssh-proxy` streams. Commits since run 10 are authored by
  the owner's account (no-reply address), Claude co-author.
- **Routine run 11 (2026-10-05):** PS-E-01's "attesters' claims as properties" (D-V2-87): `hostile-agent` at L3 had no way
  out though DL1408 named one — a claim `PROPERTY: how` (exact name) of the pinned attester now meets the property, its
  state still `unknown`, the claim beside it as `attested`. Traps: a refusal's way out is a promise — run it; a mutant
  that the code after it overwrites is a no-op, not a survivor. Then ANSWER-HOLD-1 (RW 4.37, D-V2-88): a guest that read
  nothing of a large answer held its host for ever — every guest socket has a write deadline now, each answer owed whole
  within the frame deadline; a bound on one direction of a channel asks the question of the other. And PS-E-04's
  attestation binding (D-V2-89): a `delulu-attestation-v2` statement names the launcher its attester measured, refused
  for any other.
- **Routine run 12 (2026-10-05):** a macOS guest's memory ceiling is the host's sampler of its peak footprint (every
  5 ms — 25 ms let a guest reach 247 MB against 64 MiB), so `resource_ceiling` holds on every OS and macOS's reads are
  `contained`'s one gap (D-V2-90); RUNDIR-PERM-1, a false permissions alarm on every macOS run, closed (D-V2-91);
  `contained` requires egress, resource and host-loss confinement of a boundary the host measured (D-V2-92); **PS-E
  complete** with three residuals — H6, macOS's launcher window, macOS's reads (D-V2-93); SILENT-QUEUE-1 (RW 4.40,
  D-V2-94: silent connections queued the broker's clients, the e-stop's among them) and RW 4.39 closed. **Next: P8-01**
  — its build order is in `V2_P8_DESIGN.md`.
- **Routine run 13 (2026-10-05): P8-01 complete** (D-V2-95; L1 and L2, the stepped clock and sign-off, the devices' journal in the report) — a control program runs in a sandboxed guest; the host performs
  each actuator command and sensor read by the interpreter's own body (`device::actuate`/`sense`) against the run's device
  broker, which starts when the guest is sent its program; `run_cmd.rs`'s device code is shared with the sandboxed path. A
  hardware run's flags without `--broker-profile hw:` are now refused on both paths (they were silently ignored).
  `scripts/suite.sh` reports `TREE-MOVED`. **Next: P8-02** (the Verified-class adapter as a `.dpx`).
- **Back on the laptop:** follow `docs/CLOUD_SYNC_LOG.md` *Syncing the laptop afterwards* — ff-only
  pull, CRLF check, Survey + doctor, full suite Win + WSL, each entry's redo items, then merge
  `docs/assistant-memory/` and HANDOFF §11 changes back into this directory.

**Why:** Jesse's instruction, 2026-09-28. **How to apply:** in the cloud read CLAUDE.md → AGENTS.md →
HANDOFF §0/§1/§11 → docs/assistant-memory/MEMORY.md; on the laptop after the 16th, sync first.
Related: [[delulu-v2-execution]], [[testing-repo-autopush]], [[final-public-repo-gate]],
[[agent-usage-rule-2026-09-17]].

## Routine run 14 (2026-10-05) — P8-02 complete

- A hardware driver may now be a signed Verified plugin: `--adapter-dpx` read once and pinned to `--adapter-signer`,
  re-proved, and interpreted from those same bytes; `--adapter-transport` carries its frames. D-V2-96 and D-V2-97.
- **A new example is a new case in four gates** (`fmt --check examples`, the Atlas round-trip, `atlas_chain`'s snapshot
  corpus, `core_invariance`) and no witness target reaches them: the slice's local suite is what finds them.
- **The session's model changed mid-run** (Opus 5.5 → Opus 5, 1M context) with no classifier notice; commits from
  `a8040fd` on name Opus 5, as `CLAUDE.md` requires.
- **P8-03 the same run:** `delulu device sim` runs the in-tree simulator as a device process on the `CMD`/`READ` line
  protocol, so the whole stack is witnessed across a real process boundary, with parity against an in-process `sim` run
  (D-V2-98). **A new subcommand is five gates** (help, dispatcher, `SUBCOMMANDS`, both `json_contract` sweeps, the MCP
  door rule) — and a server is classified by what it hands out, not by what it writes.


## Routine run 15 (2026-10-09) — `master` green again; a commit hook

- `master` was red on every OS for four days (push run and four nightlies on `75bb33b`): its map was built before its
  last edits. Fixed by `1732d9e`, which also merged run 14's stranded `6dbc54c` (D-V2-99, re-verified).
- **`git config core.hooksPath scripts/hooks`** in every clone: `pre-commit` refuses a commit whose map does not match
  the files the commit holds (the index, exported and surveyed) — the rule made mechanical.
- **P8-04 the same run:** step 1 measured — an envelope refusal was not in the audit chain (AUDIT-REFUSAL-1, fixed,
  D-V2-100); the chain's seq is not a cursor while a sandboxed run's host writes beside the daemon (AUDIT-SEQ-1, RW 4.44,
  open). Step 3 (b) built: `delulu monitor watch` quarantines a run under its node by revoking it, and the revocation's
  record says why (D-V2-101). Next: step 5, the monitor's own death.
- **AUDIT-SEQ-1 closed the same run** (D-V2-102): the daemon counted from 1 on every start and a sandboxed run's host
  numbered "last + 1" outside the lock; the log now settles every seq under its append lock and the daemon takes the
  chain's next seq as a floor, so `revoked_by_seq` names the record that is there.
- **HTTP-SCHEME-1 the same run:** under `--broker daemon` a plain-`http://` fetch died DL0904 (the custody gate asked about
  a host named `http`); fixed to the embedded answer, `Refused`. P8-04 step 5 read against the code: build (c), a broker
  dead-man for the monitor's node.
- **The red-team pass the same run** (Sonnet 5.5, frozen binary): ACTOR-CUSTODY-1 (HIGH, open, RW 4.47 — an actor's
  effects bypass daemon custody and the device broker; revocation does not reach them); AUDIT-SEQ-1's references are wrong
  under contention (RW 4.48, D-V2-102 corrected); the monitor revoked its own node on a deny by it (fixed, `495204e`).
- **Routine run 16 (2026-10-10): ACTOR-CUSTODY-1 closed** (D-V2-103): each actor worker holds a custody client for the
  run's node and shares the run's device and compute brokers — wider than found, since in every mode an actor's command had
  been answered `Ok` by nobody. Its sibling, a plugin export's nested interpreter (RW 4.50, PLUGIN-CUSTODY-1), is a
  hypothesis to witness. Lesson: every `Interp::new` is a place the run's authority can stop.
- **PLUGIN-CUSTODY-1 closed the same run** (D-V2-104): a plugin export's nested interpreter had an allow-all custody and no
  device broker; under the daemon no plugin had ever been callable (PLUGIN-DAEMON-1 — liveness was never answered, so every
  call was `Revoked(0)`). It shares the host's custody and devices now. Lesson: a path that fails closed on everything hides
  what stands behind it.
- **RW 4.48 and AUDIT-DAY-1 closed the same run** (D-V2-105): the daemon holds the audit chain's lock from a seq's floor to
  its write (2–3 wrong references in 72 parallel runs before, 0 after); and a record stamped before midnight but written after
  another writer's post-midnight record had been filed in the earlier day's file, breaking the chain — found because a
  test's fixed timestamp fell on yesterday.
- **RW 4.51 closed the same run:** a plugin's grant node is revoked when its run ends (10g's rule for device nodes). The next
  run starts at P8-04 step 5, option (c) — sized in `V2_LOG.md` 2026-10-10 (closing).
- **Routine run 17 (2026-10-10): P8-04 step 5 built** (D-V2-106): the broker holds a dead-man for the monitor's node — a
  killed or stopped monitor's runs are revoked about one period later, the cause in the chain; the beat carries a key only the
  monitor holds, and the monitor is non-dumpable on Linux. Lessons: a witness of an ORDER must build the state where the order
  matters (M201 survived two floods that let the queue drain); a same-user-but-not-root witness is read on `ubuntu-24.04` (the VM
  is root, and the harness refuses `su -c`).
- **CLOSURE-SCOPE-1 closed the same run** (D-V2-107, RW 4.55, HIGH — the red team's F-03): a closure resolved its names in its
  CALLER's module, so a host callback inside a Verified plugin export ran the plugin's same-named function with the host's
  captured capabilities (a file written, a secret read, by a plugin with no grant). A closure carries its home module now.
- **RW 4.56 closed the same run** (D-V2-108): a plugin node's revoke now reaches its running export's next use. And `master` went
  red once on this run's own witness — its baseline asserted a throughput (200 requests in 1.6 s; CI's parallel test job answered
  45). Lessons: a baseline is the state a witness needs, never a throughput; a mutant red only while another load shared the VM is
  not yet red.
