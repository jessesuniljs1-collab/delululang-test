# HANDOFF — DeluluLang

**For:** whoever picks this repository up next, human or AI.
**What this is:** a current briefing — the standing rules, what DeluluLang is, how to work here, and
the project's memory written down. It was rewritten on **2026-09-27** (V2 phase P6) from a 1,200-line
document that had grown a ledger, a feature tour, dated numbers, a problems list and a per-platform
record alongside the briefing. Those sections are kept **verbatim, with their original numbers**, in
[`docs/archive/v1/HANDOFF_HISTORY.md`](docs/archive/v1/HANDOFF_HISTORY.md); the last section here says
where each one went, so a citation of "`HANDOFF.md` §8" still finds its text. Nothing was deleted.
**Brought up to date on 2026-09-28** for the cloud period (§0): the state below, §0 itself, and §11 —
the assistant's memory — which now also travels file by file in
[`docs/assistant-memory/`](docs/assistant-memory/).
**Repository:** on the owner's laptop at `D:\nelan\DeluluLang` — a Rust workspace of 13 crates — and on GitHub, **publicly since
2026-09-17**: `origin` → `https://github.com/jessesuniljs1-collab/delululang-test.git` (§1.1).

## Where things stand (2026-09-28)

- **DeluluLang V2 is executing** — the active source of truth is `docs/DELULULANG_V2/` (start at
  `V2_README.md`).
- **Done**, each phase's CI run read green: V2-0 (workspace and documents), P1 (machine-contract truth),
  PS-0 (sandbox truth), PS-A (the process sandbox and its effect channel), P2 (run-time plugin loading),
  P4a (the Agent Skill), P3 (a standard library), PS-B (budgets as authority, the egress proxy, identity,
  break-glass), P4b–e (agent surfaces: `delulu mcp`, schemas, checked edits, Atlas and Survey tooling,
  the usability benchmark), **PS-C** (the microVM: `--isolation microvm` on Linux + KVM under
  Firecracker's jailer — except PS-C-05, a distributed guest image, which is the owner's under
  D-NE-27), **P6** (this file, the README and the Book) and **P5** (distribution: the release workflow,
  whose dry run packages, checks and installs four targets and publishes nothing — only the owner's
  first `v*` tag releases anything, D-NE-7).
- **Done in the cloud period** (the first routine run, 2026-09-28, each CI run read green):
  - **P7, verification depth — complete.** The NIST vectors checked byte-exact (`nist_kat.rs`), six
    `cargo-fuzz` targets under AddressSanitizer, the Miri-slow tests shrunk under `cfg!(miri)`; the
    `miri-slow` run `36381950975` on `047da1d` green on all three crates, closing REMAINING_WORK 5.6
    (its predecessor `36368020490` had found AUDIT-LOCK-TAKEOVER-1); and RW 5.2/5.3 —
    `DELULU_CORE.md` v0.3, progress-or-fault and the higher-order primitive, an entrenched edit under the
    owner's delegation (D-V2-49, flagged for his review).
  - **PS-D, external launchers and the attestation seam.** PS-D-01 is done:
    `--sandbox --sandbox-backend external:CMD`, level 3, measured by nobody and reported so (D-V2-46).
    **PS-D-02 is built** (2026-09-28, the first cloud routine run, D-V2-48): `--require-attestation HEX`
    at L3, `delulu sandbox attest` as the software attester. **PS-D is complete** — `fef8ccd`'s push
    run `36395256154` green on every job.
- **Studied on the laptop the same evening — NVIDIA OpenShell** (the open runtime of NVIDIA's Open Agent
  Safety Platform, launched 2026-09-28), on the owner's commission *"studied and incorporated not just as
  copy but as real engineering"*: `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` — the two designs side by
  side, what DeluluLang takes and why, what it does not. It added **PS-E** (the boundary, confirmed),
  **P8-04** (an out-of-band monitor, the shape of NVIDIA's Sentry) and **P9** (authority at the
  boundary); D-V2-52 to D-V2-55. Nothing of OpenShell is copied into the repository.
- **Begun in the cloud (routine run 2, 2026-09-28 night): PS-E-01's first step** (D-V2-56). Building it
  found the host's FIRST channel frame was the program itself, so a guest that never confined itself had
  already received it (witnessed red on `ff701ae`). Now `delulu-sandbox-channel/3`: the host opens with a
  per-run generation and no program, the guest confirms its boundary, and `boundary.rs`'s `Confirmed` is
  the only way the program is sent; and every run reports its boundary's five properties (D-V2-57 —
  macOS's gaps visible: reads open, no memory ceiling, no death signal). The CI workflows moved to their
  Node-24 action majors and `ubuntu-24.04`. A red-team sous-chef found seven defects around the new
  guarantee (which held); all verified and fixed the same run (D-V2-58) but F7 (RW 4.31). And, on CI's
  measured properties, `hostile-agent` now requires all five (D-V2-59 — flagged for the owner).
- **Routine run 3 (2026-09-29):** `witness.yml` (one test on one runner, by hand) and
  `scripts/check-other-os.sh` (clippy for macOS and Windows from Linux), so a CI-only defect is witnessed red on a branch
  before its fix reaches `master`; with them **PS-E-02 on macOS** — a guest computing when its host was
  killed lived on until its CPU ceiling (red on a runner); a watcher OUTSIDE the guest now ends it and the
  external launcher (D-V2-60).
- **PS-E-03 begun (routine run 3, D-V2-61):** an escaped-guest harness found H1, H2 and H3 all real — an
  escaped Linux guest could open sockets (and connect to the operator's own), read other processes'
  `environ`, make io_uring/memfd/userfaultfd/pidfd calls and a user namespace — plus H7, the operator's
  terminal; all closed, each with a mutant. H4 too: a serving host is non-dumpable (D-V2-62). And H8, new:
  an escaped guest typed into the operator's terminal with `TIOCSTI` — refused now (D-V2-63); H9, new: it
  could signal the operator's processes — now only itself (D-V2-64); H10, new: or change their limits,
  priority, CPUs and scheduling — refused (D-V2-65). H5: below ABI 3 the report claimed writes denied —
  exact now (D-V2-66).
- **PS-E complete (routine run 12, D-V2-93)** — `contained` requires egress, resource and host-loss confinement (D-V2-92),
  with three residuals kept open: E-03's H6, macOS's launcher window, macOS's reads. **Next: P8** (P8-01 first), then P9.
- **P8-01 complete (routine run 13, D-V2-95)** — a control program runs in a guest, at L1 and L2: the host performs its actuator commands and
  sensor reads by the interpreter's own body against the run's device broker; the broker starts when the guest is sent its
  program. P8-02 (a signed Verified driver, D-V2-96/97 — RW 4.7 closed) and P8-03 (the simulator as a device process,
  D-V2-98) complete (routine run 14).
- **P8-04 in progress (routine run 15, 2026-10-09):** step 1 MEASURED before building on it — an envelope's refusal was
  not in the audit chain (AUDIT-REFUSAL-1, fixed, D-V2-100) and the chain's seq was not unique (AUDIT-SEQ-1, fixed,
  D-V2-102); step 3 (b) built — `delulu monitor watch` quarantines a run under its node and the revocation's record says
  why (D-V2-101); HTTP-SCHEME-1 fixed on the way. **Routine run 16 (2026-10-10):** ACTOR-CUSTODY-1 closed — an actor's
  effects answer to the run's custody and brokers (D-V2-103); PLUGIN-CUSTODY-1 the same — a plugin export runs under the
  host's custody and devices, and plugins are callable under the daemon at last (D-V2-104); RW 4.48 and AUDIT-DAY-1 closed
  (D-V2-105); RW 4.51 (a plugin's node revoked when its run ends). **Next:** **P8-04 step 5** (option (c), a broker dead-man for the monitor's node), RW
  4.46 (the egress client's refusals), then P9.
- **Before PS-E closed, its open items were:** E-03's H6 (macOS and Windows under the escaped-guest harness) and a red-team
  pass on the filter; E-01's `contained` set (attesters' claims as properties were built by routine run 11, D-V2-87; a
  macOS guest's memory ceiling — the host's sampler — by routine run 12, D-V2-90, so macOS's reads are its one gap); E-04's
  macOS `fexecve` (its attestation binding — a v2 statement naming the launcher its attester measured — routine run 11,
  D-V2-89). E-02, E-05 ((a) routine run
  8, D-V2-82; (b) routine runs 9–10, D-V2-83, D-V2-85 — the guest inside a real OpenShell sandbox) and E-06 (routine run
  7, D-V2-78, D-V2-80) are complete. Then **P8** (P8-01 the
  control program in a guest, P8-02 the Verified-class adapter, P8-03 a reference transport, P8-04),
  then **P9**. A real device stays environment-blocked. ADAPTER-SPELL-1 (the subprocess driver verified
  as one file and started as another) was found and fixed while sizing P8.
- **Last pushed from the laptop:** `Cloud handoff (7)` — the laptop synced with the first routine run's
  13 commits (`5bb39bc..dd543e5`, a fast-forward; the Survey matched the tree, 0 errors, `doctor` 29/29
  on Windows), and the OpenShell study. Earlier, CI on `30a6b8d` was green on every job — 1,967 tests
  passed on Windows, 1,985 on Linux, 1,975 on macOS, 0 failed.
- **Resume from `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, then the newest entry in
  `docs/DELULULANG_V2/V2_LOG.md`, then `docs/CLOUD_SYNC_LOG.md`.** Decisions taken under the owner's
  delegation are `V2_DECISION_LOG.md` (`D-V2-nn`).
- **What is open**, everywhere: `docs/REMAINING_WORK.md`. **What only the owner can decide:**
  `V2_DECISION_LOG.md`, *Owner decisions carried from V1, still open*; the owner-reserved list in
  `AGENTS.md`; and §1.1's gate for the final public repository.

> **Status: PRODUCTION READY WITH DOCUMENTED DEPLOYMENT REQUIREMENTS** — Windows and Linux, in the
> Tier-2 deployment of `docs/DEPLOYMENT.md` (the 2026-08-10 campaign's verdict). macOS is verified by CI
> but not covered by the verdict: the Tier-2 cross-account boundary was tested with a real second UID on
> Linux only.

Read §1 and §2 before touching anything. The rest is reference.

> ### If you are Claude Code, read this first
>
> **On the owner's laptop**, assistant memory for this project lives at
> `C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\`, keyed by project path, and a session
> opened at `D:\nelan\DeluluLang` loads its `MEMORY.md` automatically. It lives **outside the
> repository**: it does not travel with a clone and is not shared with another account or machine.
>
> **In a cloud session there is no memory directory at all** — Claude Code's auto memory is
> machine-local. Two copies travel with the repository instead: **§11 of this file** (the durable
> content, organised by purpose, and the authority if anything disagrees) and
> **[`docs/assistant-memory/`](docs/assistant-memory/)** (the memory directory itself, file by file,
> snapshotted on 2026-09-28 and sanitized for a public repository — read its `MEMORY.md` index at the
> start of a session). `CLAUDE.md`, which imports `AGENTS.md`, is loaded into every session; this file
> is not — read §0, §1 and §11.

---

## 0. The cloud period — 2026-09-28 to 2026-10-16

**What.** The owner needs the laptop for other work until **2026-10-16**. DeluluLang is developed from
**Claude Code cloud sessions** (claude.ai/code, the Claude app, or `claude --cloud`) on the GitHub
repository, which is already connected to Claude; the laptop's checkout is synced afterwards (owner,
2026-09-28).

**How a cloud session differs from the laptop** (Claude Code's documentation, read 2026-09-28 —
*Use Claude Code in the cloud*, *Configure cloud environments*, *How Claude remembers your project*):

| | The laptop | A cloud session |
|---|---|---|
| Machine | Windows 11, and WSL2 Ubuntu 20.04 with KVM | a fresh Ubuntu 24.04 x86-64 VM: 4 vCPUs, 16 GB, 30 GB of disk |
| Tools | Rust 1.96.1, Python, WSL, the microVM lab (§11.3) | Rust, Python 3, GCC/Clang, Docker and `gh` pre-installed; `rust-toolchain.toml` installs 1.96.1 on the first `cargo` call (crates.io and `static.rust-lang.org` are on the default Trusted network list) |
| Pushing | to `master` directly | **only to the session's own branch** → a pull request into `master`, which the owner merges; CI runs on pull requests |
| Memory | auto memory, loaded every session | none — §11 and `docs/assistant-memory/` |
| Other operating systems | native Windows; macOS through CI | all through CI |
| KVM and the microVM | the WSL2 lab | none documented — CI's `microvm` job |
| Agent-pass folders | `D:\nelan\DeluluLang-agent-transcripts\` | nothing persists outside the repository — record verified results in `V2_LOG.md` |

**Rules for the cloud period** — §1 still holds in full; these are added to it:

1. **Record every change in [`docs/CLOUD_SYNC_LOG.md`](docs/CLOUD_SYNC_LOG.md)**, in the same pull
   request: commits, and every file and folder added, modified, deleted or renamed — the owner's
   instruction, *"keep a record of files and folders changed, to be synced with the local repo later."*
   The file has the template and the sync procedure.
2. **Pushing.** The scheduled routine works on `master` and pushes there (`docs/CLOUD_ROUTINE.md`); an
   interactive cloud session pushes only its own branch and opens a pull request into `master`. Never
   force-push, delete no branch, and say what was verified and which CI runs were read.
3. **Heavy runs go to GitHub** (owner, 2026-09-28: *"run everything on github"*): the three-OS matrix,
   Miri and `miri-slow`, `heavy-gates`, the release dry run, `channel-measure`,
   `host-capability-probe` — `gh workflow run ci.yml --ref <branch> -f jobs=…`. In the VM, run the
   suite alone with `-j 4`.
4. **Durable facts go into §11** and, in the same format, into `docs/assistant-memory/` — there is no
   memory directory to write to.
5. **The owner's delegation (2026-09-28, later that day):** *"I want the development of delululang to be continued in my absentia. No need to wait for any of my input, Claude u can take better decisions than me on delululang. Run verification loops and loop engineering."*
   Every DeluluLang decision is the head chef's until he is back. At noon he widened it while creating
   the routine: *"create it. I want opus 5.5 at xhigh effort. run verification loops. routine should be set for every 5hr till oct 16 … Can start PS-D-02. Tell to finish all the phases and verify. Set up loop engineering. any file or folder is allowed to modified or created or even deleted. If every phase is built continue improving and verifying delululang the lang of the future."* So **any file or folder may be created, modified or deleted** —
   entrenched files included, each edit recorded in `ENTRENCHED_CHANGE_RECORD.md` — **except four
   things that stay his**: the final public repository, the licence, D-NE-7 (no tag, no release) and
   D-NE-27. His morning stop before PS-D-02 is superseded. The engine is a **Claude Code routine** — a
   saved prompt that runs as a cloud session every few hours with the laptop off — and every run follows
   **[`docs/CLOUD_ROUTINE.md`](docs/CLOUD_ROUTINE.md)**: orient, check health, verify the previous run,
   choose one piece of work, build it through the inner loop, close it, watch CI, record.
6. **Run and check everything with the Survey and `doctor`** (owner, 2026-09-28): `survey check` and
   `doctor --check` at the start of every session; `survey impact`/`affected-by`/`query` before every
   change; `survey build`, `check`, `findings` (0 errors) and `doctor --check` after the last edit and
   before every commit and pull request — and their results in the pull request and the sync-log entry.
   `survey diff <base>` lists a branch's changed paths for that entry. The full table is in `AGENTS.md`;
   §4 of this file explains the Survey.

**What the laptop holds that the cloud does not** — check or redo after the sync: the WSL2 KVM lab and
its guest images (§11.3); the agent-pass folders (§11.3; their verified results are in `V2_LOG.md`);
`.claude/worktrees/` (a stale 2026-07-18 copy — never delete it without asking); the auto-memory
directory itself.

**Syncing back** is written out in `docs/CLOUD_SYNC_LOG.md`, *Syncing the laptop afterwards*: a clean
laptop tree, a fast-forward pull, the CRLF check, the Survey and `doctor`, the suite on Windows and in
WSL, each entry's "redo on the laptop" items, and the memory carried back by hand.

**PS-D-02 — built on 2026-09-28 by the first cloud routine run (D-V2-48).** The design it was built
from, kept because D-V2-48 records where the build departed from it (the reference attester became the
verb `delulu sandbox attest --key SEEDFILE --attester NAME …`, beside `sandbox ticket`, rather than a new
top-level command; and `delulu` now reads nothing after a bare `--`):

1. The seam verifies a **statement**, not a platform. An attestation document is
   `{"format":"delulu-attestation-v1","statement":{"nonce","attester","guarantees":[…]},"signature"}` —
   ed25519 over `"delulu-attestation-v1\n"` followed by the statement's canonical JSON. Whoever signs
   it — a verifier service that checked a hardware quote, a CI system that built the launcher's image,
   an operator — is the attester, and its word is exactly as good as its key's custody. DeluluLang
   checks the signature against a key the **operator pinned** and the nonce it chose for this run,
   nothing else. (`ed25519-dalek` is already in `delulu-runtime`; `delulu keygen` already mints seeds.)
2. The nonce is fresh per run (32 bytes of OS randomness), given to the launcher as
   `DELULU_ATTEST_NONCE`; the launcher writes the document, atomically (temporary file, then rename),
   to the path in `DELULU_ATTEST_OUT`, inside the host's per-run directory. A replayed document fails on
   the nonce.
3. `delulu run … --sandbox-backend external:CMD --require-attestation HEX` reads and checks it after the
   launcher starts and **before the host sends the program** — so on refusal the program never ran and
   no effect was performed under the grants or a lease. At L3 only: L1 and L2 are measured, and the flag
   there is exit 2.
4. The report gains `sandbox.attestation = {attester, guarantees, verified}` — the attester's claims,
   labelled as the attester's, beside `host_guarantees` (still empty) and never merged into them. The
   level stays 3; L4 `attested` waits for a hardware attester.
5. A reference attester, `delulu attest launch --key NAME --guarantee TEXT … -- COMMAND…`, signs the
   statement with a `keygen` seed and runs the command with the channel on its standard input and
   output. It is a **software** attester — it says what its key's holder says — and the tests' fake one.
6. A lease-level constraint ("this node may only be used attested") is **not** part of it: it changes
   the authority model, which is RFC territory and the owner's governance.

**PS-E, P8-04 and P9 — designed on 2026-09-28 (evening) from the study of NVIDIA OpenShell.** The owner
paused the routine, had the laptop synced with the first run's commits, commissioned the study, and had
the routine resumed afterwards. `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` is the record: what was read
(57 documentation pages, the seven architecture diagrams from their sources, the repository at
`36b0386`), the two designs side by side, and each slice's design, witness and falsifier. The routine's
order (`docs/CLOUD_ROUTINE.md` step 4) is now **PS-E → P8 (with P8-04) → P9**. Its §4.3 items are
hypotheses until an escaped-guest witness is red; its terms (D-V2-52) — nothing of OpenShell copied,
none of its crates a dependency — bind every run.

---

## 1. Standing rules — these are not suggestions

| Rule | Why |
| --- | --- |
| **Push only to the testing remote — nowhere else.** `origin` is `github.com/jessesuniljs1-collab/delululang-test`, **public since 2026-09-17** (private from 2026-09-14). No other remote, and never rewrite history that has been pushed. **Push every commit there as soon as it is made**, so the local repository and GitHub stay in sync — no need to ask first (owner's standing permission, 2026-09-17). **In a cloud session (§0), `git push` reaches only the session's own branch: push there, open a pull request into `master`, and the owner merges.** | Owner's instruction, 2026-09-14, replacing the *"NEVER push to GitHub"* that held from the first commit (and is why CI never ran before then). The remote exists for **testing on macOS and other operating systems** (CI) and for **editing from the cloud**, and the owner made it public on 2026-09-17. **The project's final public repository will be a different one, a step the owner takes personally** — do not create one or push to one. Pushed history stays as it is because the documents cite commit hashes throughout. See §1.1. |
| **The final public repository is gated on the owner's decisions — remind him first, then wait.** Before it is created, added as a remote, or pushed to, put the items in §1.1 *Before the final public repository* in front of the owner, with the suggestions recorded there, and get his decision on each. Nothing is pushed there until every one is decided. | Owner's instruction, 2026-09-17: *"before moving to real public repo later remind to make changes to these and remind me that time before even push happens. Do not push to new real repo (future) unless my decision on these are given."* The items are already visible in the public testing repository; the final one is the chance to leave them behind. |
| **Never auto-decide the licence.** | Owner-reserved. The decision itself is made: Apache-2.0 for the code, `NOTICE`, and `TRADEMARK.md` for the name, under ruling **D27** (§8, §11.1). Any *change* to it is the owner's — present options and wait. (This row said "recommended and staged, not decided" until 2026-09-25, contradicting §8 and §11.1 of this same file.) |
| **Harden, never redefine, Authority and Guard.** | You may close holes in them. You may not change what they *mean* without the owner. |
| **Latest owner instruction beats older scheduled work.** | If a timer, a plan, or this document conflicts with what the owner just said, the owner wins. |
| **Never claim execution that did not happen.** | "Prepared" and "green" are different words and this project keeps them different. See §8. |
| **Do not fabricate evidence, and never delete a failed experiment.** | Failed runs are recorded, not tidied away. |
| **The word "graphify" appears nowhere in the repository or product surfaces.** | Owner ruling. (The `dcg` credit in the Guard docs stands and is unrelated.) |
| **Regenerate the Survey after any change, before running the suite.** | `cargo run -p delulu-survey -- build`. A test enforces freshness; see §4. |
| **Run `delulu doctor` — it is the one command that answers "is this healthy?"** | Three sections: **environment** (install, state directory, audit chain), **security posture** (root-issuance mode, anchor-key custody, whether the filesystem can enforce owner-only permissions, and whether the RUNNING broker agrees with the policy on disk), and **repository** (the map's freshness and integrity). Exit 0 healthy, **1 when a problem remains**, 2 on a bad invocation. `--check` never writes; `--json` emits one envelope. Run it **on the deployment host, as the account your agents use** — see §11.4 and `docs/DEPLOYMENT.md`. |

### 1.1 The testing remote (since 2026-09-14; public since 2026-09-17)

| | |
| --- | --- |
| **Where** | `origin` → `https://github.com/jessesuniljs1-collab/delululang-test.git`. **Public since 2026-09-17**, when the owner changed its visibility; private from its first push on 2026-09-14 until then. It is a testing repository: the project's final public repository will be a different one. |
| **Why** | In the owner's words: for testing DeluluLang *"in mac os and other os"*, and for *"editing and working on delululang from the cloud"*. It is not a distribution channel — README's *Not distributed* row still holds. |
| **What was pushed** | `master` (the GitHub default branch; local `master` tracks `origin/master`), `rc/1.0.0-drill` (DRILL-001's one unmerged commit, kept as a record) and the annotated tag `v1.0.0` (`198bf44`). The branch stays `master`, the name this history has always used, rather than GitHub's suggested `main`; `ci.yml` triggers on both. |
| **What it switched on** | `.github/workflows/ci.yml` — the three-OS matrix **including `macos-latest`**, arm64, clippy, Miri, the editor build, `cargo deny` and the formal models — on every push to `master` and on pull requests; **by hand** from Actions → CI → *Run workflow*, whose `jobs` choice runs `everything` by default, or just `heavy` (`heavy-gates` and `miri-slow`, the two jobs no push runs), or one of those two; and **nightly at 03:00 UTC, every job, unless the repository variable `NIGHTLY` is `off`** (see *What it costs*). **Activated is not executed**, and the first run's result is read and recorded in `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **What it costs** | **Nothing, since 2026-09-17.** GitHub documents its standard hosted runners as *free and unlimited on public repositories*. While the repository was private (2026-09-14 to 2026-09-17) its minutes came out of the account's quota, with macOS and Windows billed at higher rates than Linux, so three things were rationed: the nightly schedule was **opt-in** (it ran only where the repository variable `NIGHTLY` was `on`), the manual button defaulted to the two heavy jobs, and docs-only commits said `[skip ci]`. On 2026-09-17 the owner asked for the restrictions kept for cost to be reconsidered, and all three were lifted: the nightly runs every job unless `NIGHTLY` is `off` (Settings → Secrets and variables → Actions → Variables) — kept as an off-switch for a private copy, which would pay again — the button defaults to `everything`, and every push runs CI. Two things remain true: a nightly run re-tests the newest commit on `master` whether or not anything changed, and GitHub disables a public repository's scheduled workflows after 60 days without activity. |
| **Public since 2026-09-17** | The owner made the repository public. For CI that changes two things, both from GitHub's documentation: runners are free and unlimited, and they are larger — **4 CPUs and 16 GB** for Linux x64, Linux arm64 and Windows (2 CPUs and 8 GB on a private repository); macOS stays at 3 (M1) and 7 GB. The second has a consequence nobody has measured yet: `actors_pingpong`'s speedup criterion (at least 1.5× at 4 workers) is asserted only where 4 hardware threads exist, so it has never been asserted on a CI runner, and from the next run it will be on Linux x64, Linux arm64 and Windows. On the development machine, pinned to 4 or 8 of its 16 logical CPUs, it measured 0.65–1.14× (unpinned it passes, at 2.16×). A pinned laptop is not a faithful 4-CPU runner, so the next run is the real measurement — and it may fail there. Recorded before that run in `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. **It did not fail:** run 6 passed it on all three 4-CPU runners, so the prediction was wrong, and it stays recorded as wrong. |

The CI runs that followed the first push — the first six, each read and recorded — are in
`docs/archive/v1/HANDOFF_HISTORY.md` (§1.1's run rows) and, run by run, in
`docs/design/CROSS_PLATFORM_VERIFICATION.md` §9; every V2 run is in `docs/DELULULANG_V2/V2_LOG.md`.

**Before the final public repository — a gate, not a to-do list.** This testing repository has been
public since 2026-09-17, so everything below is already visible here. **Before the final public
repository is created, added as a remote, or pushed to, put this list and the suggestions after it in
front of the owner, and wait for his decision on each item. Nothing is pushed there until all of them
are decided** (owner's instruction, 2026-09-17; §1). The list:

1. **The word §1 bans is in the history.** Commit `0a58451` (2026-07-14) put it, with the tool's
   address, into `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md` and into its own commit message.
   `f4ffd01` anonymised the document but not the history, and §1 and §11.1 spell the word in stating
   the rule. This testing repository holds that history, because the push carried all of it, and
   since 2026-09-17 it is public. For the final public repository the choices are a fresh history, a
   rewrite (which changes every hash from `0a58451` onward — and these documents cite hashes
   throughout), or accepting it.
2. **Every commit records its author's e-mail address**, which this testing repository has shown
   publicly since 2026-09-17.
3. **`.github/CODEOWNERS` names a deliberate placeholder**, `@PENDING-PUBLIC-project-lead`. GitHub
   reports it as an unknown owner until it is replaced at public launch, as the file's own header
   says.
4. **The `SECURITY.md` controls marked `PENDING-PUBLIC`** — the disclosure address and its PGP key,
   branch protection, signed commits, SLSA L3 and the Scorecard floor — are written for the final
   public repository (`docs/REMAINING_WORK.md` 7.8). For the testing repository the owner decided on
   2026-09-17: `SECURITY.md` §1 now routes reports through GitHub's private vulnerability reporting,
   switched on that day together with secret scanning and push protection (approval record:
   `docs/design/ENTRENCHED_CHANGE_RECORD.md`).

**Suggested on 2026-09-17 — not decided, and not to be acted on without the owner.** He asked for no
change to these yet, only for this reminder:

| Item | This testing repository | The final public repository |
|---|---|---|
| 1. The banned word | Reword the two rule lines in §1 and §11.1 so they do not spell it; the old commit stays in history | Start from a fresh history |
| 2. The author e-mail | New commits use GitHub's private no-reply address. Change git's `user.email` **before** switching on GitHub's *Block command line pushes that expose my email*, or pushes are refused | A fresh history, authored with the no-reply address |
| 3. The CODEOWNERS placeholder | Leave it as it is | Replace it with the owner's real handle, and update `governance.rs::pending_public_controls_are_still_marked_as_pending`, which requires the placeholder today |
| 4. The `PENDING-PUBLIC` controls | They stay pending; the testing repository has the interim reporting channel above | Decide which to switch on at launch: the `security@` address and its PGP key, branch protection, signed commits, SLSA L3, the Scorecard floor |

Rewriting *this* repository's history would remove items 1 and 2 from it, and is not recommended: it
changes every commit hash from July onward, and these documents cite hashes throughout.

---

## 2. What DeluluLang is, in one page

A statically typed language whose central claim is: **a program can do nothing except what it was
explicitly handed.** Not "nothing dangerous" — nothing.

```delulu
module hello

fn main(root: Root) ! {Write} {
    let out = root.console()
    out.println("Hello, Delulu")
}
```

Three ideas carry everything else:

1. **Effects are in the type.** `! {Write}` is the *effect row* — the compiler's record of what this
   function may do. Omit it and the function is `!{}`, **proved** pure, not promised pure. A function
   that performs an effect its row does not declare is a compile error (`DL0501`).
2. **Capabilities are values.** `Cap[Console]`, `Cap[FsRead]`, `Cap[Net]` are ordinary parameters.
   Delete `root` from `main`'s signature and nothing below it can reach the console, however much it
   asks. There is no ambient authority anywhere.
3. **Authority is computed, not declared.** `delulu authority <file|package>` answers "what can this
   program do to my machine?" from the code itself, transitively, before you run it. Running requires
   handing the right over on the command line: `delulu run app --grant console`. Leave it off and you
   get `DL0703`, by design.

Everything else in the project — the broker, the Guard, plugins, the atlas, the registry — exists to
extend those three ideas to things larger than one file: to processes, to packages, to fleets, and to
agents acting on your behalf.

**What V2 has added so far** (2026-09-17 onwards, `docs/DELULULANG_V2/`): a program can run as a
**guest that holds no authority of its own** — every effect it asks for is performed by the host, under
the same checks — in a jailed process (`--sandbox`, on all three systems) or in **its own kernel under
Firecracker** (`--isolation microvm`, Linux with KVM), each run reporting the level it actually got;
every run is **budgeted**, and a budget is an authority dimension a delegation can only narrow;
`http.get` fetches over verified HTTPS through one host-side client; a program **loads plugins at run
time**; the standard library has real collections; and agents get a **machine surface** — `delulu mcp`,
closed JSON Schemas, `toolchain --json`, checked edits, the Atlas's authority chain.

**Who it is for:** developers, and equally **AI agents, LLMs, robots and physical AI**. That is why
every command has a `--json` envelope, why the language server is a first-class surface, and why
`delulu.authority` is answerable over the wire without shelling out.

---

## 4. The Survey — use it, and the difference from the Atlas

**These are two different tools and confusing them wastes an afternoon.**

| | **Survey** (`delulu-survey`) | **Atlas** (`delulu-atlas`) |
| --- | --- | --- |
| Maps | **this repository** — crates, modules, docs, diagnostic codes, rulings | **a DeluluLang program** you wrote |
| Answers | "what breaks if I change this file?" | "what can this program do, and why?" |
| Built from | the tree, by a scan; every edge cites a `file:line` | the compiler's own facts about your code |
| Lives in | `docs/survey/` (committed, freshness-tested) | generated on demand |
| You run | `cargo run -p delulu-survey -- <verb>` | `delulu atlas <file\|package>` |

### Use the Survey before you change anything

```
cargo run -p delulu-survey -- impact <id>       # everything that breaks if this changes
cargo run -p delulu-survey -- affected-by <id>  # everything this rests on
cargo run -p delulu-survey -- rdeps <id>        # what points at it — ONE hop
cargo run -p delulu-survey -- path <a> <b>      # how one reaches the other, hop by hop
cargo run -p delulu-survey -- diff <rev>        # what a CHANGE breaks: changed files → nodes → impact union
cargo run -p delulu-survey -- query <id>        # a node, and both directions
cargo run -p delulu-survey -- findings          # the discrepancy list
```

IDs look like `crate:delulu-check`, `mod:crates/delulu-check/src/ty.rs`, `doc:README.md`,
`code:DL0501`, `ruling:S10-D64`, `finding:C69`. Add `--json` to any verb.

### Regenerate it after, and *before* you run the suite

```
cargo run -p delulu-survey -- build     # regenerate
cargo run -p delulu-survey -- check     # fail if the committed map is stale
```

A test enforces this (`the_committed_map_matches_the_tree`), and `delulu doctor` checks it too. **The
single most common self-inflicted failure in this project is editing source while the test suite runs
and then blaming the three `doctor_cli` failures on something else.** Regenerate, then measure.

**One known limit of the Survey:** it maps what comments *cite*. A coupling nobody wrote down is
invisible to it. That cost a real bug in P19 — `lsp.rs` reached nothing but `main.rs` in the map,
while the VS Code extension depended on it critically. If you create a cross-language coupling,
**name the other file in a comment** or the map will not know.

### The Atlas, for programs

```
delulu atlas app.delulu --format tree|json|dot|mermaid|html
delulu atlas node <name> | callers <fn> | calls <fn> | why <Effect> | path <A> <B>
```

`--format html` emits a self-contained document; the VS Code extension shows it in a panel
(*DeluluLang: Show authority atlas*).

---

## 7. How to work here

```
cargo build --release                              # first build fetches everything
cargo test --workspace --no-fail-fast              # 150 test binaries (--no-fail-fast MATTERS:
                                                   #  without it cargo stops at the first failing target)
cargo clippy --workspace --all-targets -- -D warnings   # currently ZERO warnings; keep it there
bash scripts/cli-sweep.sh <abs-path-to-delulu>     # every case an exact exit code; it counts its own
cargo run -p delulu-survey -- build                # ALWAYS, after any change
./target/release/delulu doctor                     # the health command: environment,
                                                   #  SECURITY POSTURE, and the repo map.
                                                   #  Exit 1 means a real problem remains.
```

In a cloud session (§0) the VM has 4 vCPUs: add `-j 4`, run the suite alone, and leave Miri,
`heavy-gates` and the other operating systems to CI (`gh workflow run ci.yml --ref <branch> -f jobs=heavy`).

Editor extension:

```
cd editors/vscode
npm install && npm test          # 15 tests, no framework dependency
npm run package                  # -> delulu-lang.vsix
npm run verify                   # refuses a .vsix that would fail to activate
node e2e.js <path-to-delulu>     # launches REAL VS Code against a REAL server
```

**House rules that have each been paid for:**

- **A gate that cannot fail is not a gate.** Every new test must be *falsified* — reintroduce the
  defect and watch it go red. Several tests in this repo were written, passed, and checked nothing.
- **Do not write examples. Generate inputs.** Hand-written corpora agree with the author's idea of the
  problem.
- **Write the skip-branch case.** Security rules die in the `else { continue }` branch.
- **Silence is not evidence.** Assert on things that *happened*, not on the absence of errors.
- **Measure the unit cost before choosing a budget.** Three attempts were wasted guessing at Miri
  budgets before one `time` invocation answered it.

---

## 10. If you are starting fresh, do this

0. **In a cloud session:** `CLAUDE.md` and `AGENTS.md` are already loaded. Read §0 of this file,
   `docs/CLOUD_SYNC_LOG.md`, and `docs/assistant-memory/MEMORY.md` first.
1. Read `README.md`, then this file's §1, then `docs/REMAINING_WORK.md` (what is open, and why).
2. `cargo build --release` and `cargo test --workspace --no-fail-fast`. Expect 0 failures (1,967 tests
   on Windows, 1,985 on Linux and 1,975 on macOS in CI run `36367985299`, 2026-09-28; the per-date
   figures are in the V2 log, and before V2 in the archived §9). **Read cargo's own exit code, not a pipeline's** — `cargo test … | tail` reports the
   *pipe's* status, which is how a red gate once survived a whole campaign described as green.
   If `doctor_cli` fails, run `cargo run -p delulu-survey -- build` and try again.
3. Ask the Survey about anything you are about to change.
4. Read `docs/release/CHECKPOINT-1.0.md` for the honest status,
   `docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md` for the most recent adversarial pass, and
   and `docs/DELULULANG_V2/V2_README.md` for what is being built now (V2) and where it stands.
5. When you add a test, **falsify it**. When you fix a bug, **witness it failing first**.
6. Regenerate the Survey before you commit.

The hardest-won lesson in this repository, learned repeatedly and at cost: **a green suite is not
evidence that the thing works. It is evidence that the tests you wrote pass.** Every serious defect
found since 1.0 was found while everything was green.

---

## 11. Everything held in assistant memory, written down here

**Why this section exists.** Claude Code keeps per-project memory at
`C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\`, keyed by the **project path**. A new
session opened at `D:\nelan\DeluluLang` **on this account, on this machine** loads the same
`MEMORY.md` automatically. But that directory is **outside the repository**: it does not travel with a
clone, does not exist on another machine, and is not shared with a different account. So everything
durable is transcribed here, and this file is the authority if the two ever disagree.

There were **26** memory topics on 2026-08-10 and **40** on 2026-09-28. Their content is below,
organised by what it is for rather than one-per-topic, because several topics say the same thing from
different angles. **Since 2026-09-28 the topics themselves are also in the repository**, file by file,
in [`docs/assistant-memory/`](docs/assistant-memory/) — sanitized for a public repository and otherwise
verbatim. In a cloud session this section and that folder are the only memory there is (§0).

**If you are a fresh session with no memory loaded, §11 is your briefing** — it is written to stand on
its own. If you *do* have memory, this section will be familiar; where the two disagree, this file
wins, and you should update the memory to match.

### 11.1 Standing owner instructions — the ones that never expire

- **Push only to the testing remote**, `origin` → `github.com/jessesuniljs1-collab/delululang-test`
  (owner instruction, 2026-09-14; public since 2026-09-17). It replaced *"NEVER push to GitHub"*,
  which held from the first commit and is why CI never ran before that date. The remote is for
  cross-OS testing and editing from the cloud. **The final public repository will be a different one,
  a step the owner takes personally**: never create one, push to one, or add any other remote, and
  never rewrite history that has been pushed. §1.1 has the details, and the owner's list of decisions
  for the final public repository. **Before that repository is created, added as a remote or pushed
  to, remind the owner of those decisions and wait for each one** (owner's instruction, 2026-09-17):
  no push to it until they are all made.
- **Commit and push every change to the testing remote, without asking** (owner's standing
  permission, 2026-09-17): *"for push everything and anything to the github test repo, no need to ask
  my permission everytime. commit locally and commit to github test repo always, both should be in
  sync."* Confirm the sync after each push, never force-push, and keep GitHub's skip token out of any
  commit message whose push should run CI. The final public repository's gate above is unaffected.
- **Never auto-decide the licence.** It is already decided: **Apache-2.0** for the code plus `NOTICE`
  and `TRADEMARK.md` for the name, shipped under ruling **D27** (commit `42702e2`) and discharging
  hardening finding C9. The rule still binds any *change* to it — owner-reserved, present options and
  wait — but the decision itself is not outstanding.
- **Harden, never redefine, Authority and Guard.** Closing a hole is welcome. Changing what they mean
  is not, without the owner. The owner has stated this as a guardrail more than once.
- **The word "graphify" appears nowhere** in the repository or any product surface. Documents were
  anonymised at commit `f4ffd01`. The `dcg` credit in the Guard documentation is unrelated and stands.
- **The scheduled-continuation feature must not be removed.** If autonomous wake-ups are used they
  must be a **one-shot chain carrying a nonce**, never a recurring cron: a recurring job outlived the
  terminal being closed and beat the owner's `Esc`, which is why the one-shot rule exists.
- **A newer owner instruction always beats older scheduled work.** Never let a timer win against a
  fresher instruction.
- **Never update memory before independent verification succeeds**, and **never fabricate evidence**.
  Only claim results that actually ran.
- **Sous-chef agents (owner's rule, 2026-09-17, revised by the owner the same day — the revision
  governs):** Opus or Sonnet only, and only where an agent adds real independent value — credits are
  not spent on agents for their own sake; high effort for the difficult briefs (the harness offers no
  reasoning switch, so the brief carries it); **if the session limit is reached, stop every agent
  gracefully and preserve all completed work** — before stopping, the agent's task progress,
  decisions, completed actions, pending tasks, relevant outputs and work state go to the phase's
  `.md` progress files and to project storage, so the work can be resumed later; **no private
  chain-of-thought or hidden reasoning is saved or reproduced** — findings and decisions, not
  deliberation. Nothing an agent claims is used before the head chef verifies it against the current
  binary or source. The owner may change this rule; its current wording governs.
- **Keep cooking, and do not ask (owner, 2026-09-20 and again 2026-09-25):** finish every phase in
  order, check that the earlier ones work and are finished, be persistent and retry until a thing is
  built, and do not ask questions — *"Even if u ask me questions I will say 'Do whatever good for
  delululang, The lang of the future'"*. Use and update the Survey and `doctor`; keep the `.md` files
  and this file current. Pause about a minute after each phase and major run for a reply, then
  continue. Three things stay the owner's regardless: **entrenched files** (CODEOWNERS), **the final
  public repository**, and **the licence**. Decisions taken under this delegation are recorded as
  `D-V2-nn … TAKEN (head chef, under the owner's delegation)`, never as rulings he made.
- **Agents (owner, 2026-09-20, then 2026-09-27):** Sonnet 5 for small, easily finished jobs; and for
  the testing passes, *"use haiku 4.5 and sonnet 5 as agents"* (2026-09-27) — agents run programs
  inside the DeluluLang sandbox on several operating systems and deliberately attempt what they must
  not be able to do, with authority configured through `delulu authority` and validated with the
  Guard. §11.7 has what running them taught.
- **V2 execution rules (owner, 2026-09-17, evening; `docs/design/DeluluLang_V2_Execution_Master_Prompt.md`):** Opus 5 is the main execution sous-chef and does most of the work — implementation, investigation, tests, refactors, security and adversarial testing, documentation migration, verification, cleanup; one strong agent at a time; the head chef writes the brief (objective, files, constraints, security and verification requirements, expected outputs), supervises, verifies every result against the binary, and commits; an agent asks rather than invents an architectural or security decision. Phases run one at a time in the approved order; after each phase's push and recorded result the work **stops, waits about sixty seconds for the owner, and continues to the next approved phase only if nothing arrives** — one controlled continuation, never a loop. The active source of truth is `docs/DELULULANG_V2/`; historical documents live in `docs/archive/v1/` and are neither maintained nor deleted.

- **Back up before a limit (owner, 2026-09-27):** *"backup everything including agents running and
  their findings"* — progress, findings, pending work and state go to files before a session limit.
- **Continue after a limit (owner, 2026-09-28):** use the Survey and `doctor`, find where the work
  stopped, and resume the stopped agents (by message, never respawned).
- **Run everything on GitHub (owner, 2026-09-28):** heavy runs — the matrix, Miri, `heavy-gates`, the
  release dry run, the measurement and probe workflows — go to CI, not the owner's machine.
- **Stop before PS-D-02 (owner, 2026-09-28, morning)** — superseded the same day by the next line.
- **Everything is allowed but four things (owner, 2026-09-28, noon):** *"create it. I want opus 5.5 at xhigh effort. run verification loops. routine should be set for every 5hr till oct 16 … Can start PS-D-02. Tell to finish all the phases and verify. Set up loop engineering. any file or folder is allowed to modified or created or even deleted. If every phase is built continue improving and verifying delululang the lang of the future."* Opus 5.5 at xhigh
  effort (`.claude/settings.json`), a routine every five hours until 2026-10-16, every phase finished and
  verified, then continuous improvement. Reserved: the final public repository, the licence, D-NE-7,
  D-NE-27.
- **Continue without him (owner, 2026-09-28):** *"I want the development of delululang to be continued in my absentia. No need to wait for any of my input, Claude u can take better decisions than me on delululang. Run verification loops and loop engineering."* Every decision is the head chef's until
  2026-10-16 except the five reserved by name (§0 rule 5); a scheduled routine runs the loop in
  `docs/CLOUD_ROUTINE.md`.
- **The cloud period (owner, 2026-09-28):** work continues from Claude Code cloud sessions until
  2026-10-16 (§0), and every change is recorded for the later sync — *"keep a record of files and
  folders changed, to be synced with the local repo later"* (`docs/CLOUD_SYNC_LOG.md`).
- **Run and check everything using the Survey and `doctor` (owner, 2026-09-28)** — at the start of
  every session, before and after every change, before every commit and pull request (§0 rule 6;
  the table in `AGENTS.md`).
- **Study NVIDIA OpenShell and incorporate it as real engineering, not a copy (owner, 2026-09-28,
  evening):** *"I want this to be studied and incorporated not just as copy but as real engineering for
  the sandbox currently we are working on"* — sync the laptop with the cloud's commits first, read every
  OpenShell document and diagram (the Markdown form of each page), update every document it changes,
  then resume the routine. Done that evening: `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md`, D-V2-52 to
  D-V2-55, phases PS-E and P9 and slice P8-04. The terms: nothing of OpenShell is copied into the
  repository, none of its crates is a dependency (Apache-2.0; `NOTICE` is the owner's).
- **New models (owner, 2026-09-28, evening):** *"does routine instructions need updation for claude code
  cloud. also models sonnet 5.5 and haiku 5.5 just now launched"* — checked against the official models
  page: Sonnet 5.5 is released, Haiku 5.5 is not yet. **Then, the same evening:** *"run Opus 5.5 at xhigh effort. if needed use sonnet 5.5 (latest) as agents and haiku 5.5 will be launched in coming weeks, use haiku 5.5 as agent after launching"* —
  so the head chef is Opus 5.5 at xhigh, agents are Sonnet 5.5 where one is needed, and Haiku 5.5 only
  after it is released (not Haiku 4.5 meanwhile).
  `docs/CLOUD_ROUTINE.md`, `AGENTS.md`, `CLAUDE.md` and §11.3 were brought up to the cloud docs the same
  evening.

### 11.2 Working rules the project has paid for

- **A gate that cannot fail is not a gate.** Every new test must be *falsified* — reintroduce the
  defect and watch it go red. Multiple tests in this repository were written, passed, and checked
  nothing at all.
- **Do not write examples. Generate inputs.** A hand-written corpus agrees with the author's idea of
  the problem. This rule was itself violated on the most safety-critical function added in P18, and
  the generated replacement immediately found the defect at iterations 67 and 4 on spellings no human
  writes.
- **Write the skip-branch case.** Security rules die in the `else { continue }` branch. Cost: a
  fail-open authority check (`R-6a` / `DL0803`) survived Stage 6.
- **Core-regression rule** *(owner's standing order)*: tooling is for future developers, the core is
  the product. After any tooling phase, diff the core's own output against a binary built from the
  pre-change commit. **A green suite is not proof.**
- **Consult the Survey before a change (`impact` = blast radius) and regenerate it after.** Enforced by
  a test, not by memory.
- **Model-attribution honesty**: check which model is actually running before writing a
  `Co-Authored-By` line. Stage 9 was mis-signed and the correction is recorded rather than rewritten.

### 11.3 Environment facts a new session will otherwise rediscover the hard way

*Most of these are the laptop's; a cloud session has none of them (§0).*

- **The cloud VM (measured by the first routine run, 2026-09-28):** no `gh` — CI is read with the GitHub
  MCP tools, and the proxy refuses the signed log-download URLs (`docs/CLOUD_ROUTINE.md` step 3); Docker,
  Node 22 and `xvfb-run` are there; a fresh clone holds only Linux's crates, so `cargo fetch --locked`
  goes before the suite or `egress_features` fails offline. `dockerd` starts, but Docker Hub refuses the
  VM's pulls (a shared address over the unauthenticated rate limit) — container builds go to
  `.github/workflows/container.yml`. VS Code installs from Microsoft's apt repository
  (`packages.microsoft.com` is reachable; the direct download host is not), which is how RW 7.4's
  end-to-end test ran.
- **The cloud VM, as the official docs describe it (read 2026-09-28, evening):** `gh` is *listed* as
  pre-installed and authenticated through the GitHub proxy — run 1 found none, so a run checks
  (`command -v gh`, `check-tools`) rather than assumes; GitHub release assets and API calls reach **only
  the repositories attached to the session** (a download from NVIDIA/OpenShell's releases gets 403 —
  OpenShell runs belong to a GitHub Actions workflow); a command waits 2 minutes by default, at most 10,
  then moves to the background; an **environment setup script** (configured by the owner in the
  environment dialog, not in the repository) is cached as a filesystem snapshot for about seven days if it
  finishes within about five minutes — a candidate for the Rust toolchain and `cargo fetch`.
- **The cloud VM reads `raw.githubusercontent.com` for any public repository** (routine run 2): an
  action's `action.yml` at a tag is readable there, which is how each action's runtime (`runs.using`)
  was checked before the Node-24 move. Release ASSETS stay reachable only for this repository.
  **So is a schema, file by file** (routine run 7): `github.com/ocsf/ocsf-schema` at `v1.8.0` — while its
  tarball (`codeload.github.com`, 403) and `schema.ocsf.io` (refused by the proxy) are not. Read a
  published schema at run time (`scripts/ocsf-validate.py`); never commit it.
- **Models (official models page, 2026-09-28):** Fable 5.1 (`claude-fable-5-1`), Opus 5.5
  (`claude-opus-5-5`), **Sonnet 5.5 (`claude-sonnet-5-5`, launched 2026-09-28; Claude Code ≥ v2.1.284 —
  the laptop has 2.1.284)**, Haiku 4.5 (`claude-haiku-4-5-20251001`, retirement not sooner than
  2026-10-15 — no longer used as an agent, §11.1). **Haiku 5.5 is announced, not released.** In Claude Code, Opus 5.5 and Sonnet 5.5 default to
  `medium` effort — the repository's `effortLevel: xhigh` is what raises them; a cybersecurity-flagged
  request on Opus 5.5 re-runs on Opus 4.8 and **the session stays there**, so a commit's
  `Co-Authored-By` must name the model actually running.

- **WSL2 has KVM (since 2026-09-26):** `nestedVirtualization` plus a boot-time `modprobe` in
  `wsl.conf`. The PS-C toolchain lives there: Firecracker at `~/bin/firecracker`, guest images at
  `~/microvm-image-a` and `~/microvm-image-b`, the gated microVM tests built into `~/delulu-kvm-target`
  with `RUSTFLAGS=--cfg delulu_kvm`; ordinary Linux runs use `~/delulu-gnu-target`. **Build a guest
  image from a clone on the Linux filesystem**, not from `/mnt/d` — libffi's configure fails on `drvfs`.
  From Git-Bash run WSL as `MSYS_NO_PATHCONV=1 wsl -e bash /mnt/d/…/script.sh`, and `wsl --shutdown`
  afterwards to give its memory back.
- **The GitHub CLI** is at `C:\Program Files\GitHub CLI\gh.exe`, not on Git-Bash's `PATH`; one job's
  log: `gh api --allow-escape-sequences repos/…/actions/jobs/<id>/logs` (only once the job finishes).
- **Agent-pass folders live outside the repository**, at `D:\nelan\DeluluLang-agent-transcripts\` —
  `2026-09-18-ps0`, `2026-09-18-psa`, `2026-09-26-ai-usability-pilot`, `2026-09-27-p5-multios`,
  `2026-09-27-p5b-haiku-guard` — each with its brief, the agents' logs and a `FINDINGS.md` carrying the
  head chef's verdicts. The verified results are in `V2_LOG.md`.
- **The laptop has about 15 GB of RAM.** On 2026-09-28 Claude Code stopped a full suite and a WSL Miri
  run because memory ran critically low while the session was idle. One heavy job at a time, `-j 4`,
  and CI for the rest.
- **Python on Windows** needs `C:/…` paths and prints in cp1252 — an arrow or emoji in `print` kills
  the script *after* it may already have written; write files, don't print them.

- **`.claude/worktrees/` holds a full second copy of the repository** at a 2026-07-18 commit. **Exclude
  it from every tree walk** — findings from it are about code that is not this checkout — and **never
  delete it without asking the owner.**
- **Two Claude accounts** are used on this laptop and both share the same memory directory and repo.
- **Usage limits are real**: on a limit-kill, resume the *same* subagent via its ID — work survives on
  disk — rather than respawning it.
- **The laptop's BSOD problem is resolved** (NVIDIA `nvlddmkm`, fixed at source 2026-07-12). Build
  caps are lifted. The commit-often habit remains sensible.
- **The `v1.0.0` tag (`198bf44`) was pushed to the private testing remote on 2026-09-14**, together
  with `master` and `rc/1.0.0-drill`; before that it was local only. A clone of that remote lands in
  `delululang-test/` unless you name the directory, which is why `README.md` and `INSTALL.md` now
  clone into `DeluluLang` explicitly.
- **Linux verification runs in WSL2 Ubuntu-20.04, and the harness has three rules that cost time to
  learn.** Drive it from **PowerShell, not Git-Bash** — Git-Bash rewrites `/mnt/...` into
  `C:/Program Files/Git/mnt/...` and the command silently fails. Pass **script files**, not inline
  `bash -lc '…'` (quoting collides with `$(…)` and nested quotes), and **strip CR first**. Use the
  **warm Linux target**: `cd /mnt/d/nelan/DeluluLang && CARGO_TARGET_DIR=/home/user/delulu-target
  cargo test --workspace` — an ext4 target is required, because a `drvfs` one breaks `libffi-sys`.
- **`/home/user/delulu-f1` and `/home/user/delulu-linux2` are preserved snapshots, not junk.** Only
  `delulu-target` is a rebuildable cache.
- **Disk cleanups have a written discipline** (`docs/archive/v1/maintenance/`): never delete a `.md`, confirm at
  a gate before permanent deletion, keep the build caches, and verify WSL content by hash against
  `D:` HEAD before removing anything there.

### 11.4 Findings that must never be quietly re-softened

- **P16 — the effect row was escapable.** `delulu authority` reported "provably pure" for a program
  that printed at run time. Fixed; it is the worst defect in the project's history and the reason the
  hardening campaign exists.
- **P17 — `⊑` residue, IF-1, audit truncation, wall-clock expiry.** Each is either fixed with the
  residue named, or open and named. None may be restated as closed.
- **P18 — F1/F2/F3.** `⊑` is a **preorder**, not a partial order, because `path::resolve` is not
  injective. That is mathematics, not a bug: "lattice" was right about the structure and wrong about
  the carrier, which belongs to the quotient. The canonical form is an **antichain** — set redundancy
  (`{./data, ./data/sub}` ≡ `{./data}`) is a second, independent collapse the Z3 model could not have
  seen, because it abstracts each dimension as a set over an opaque element type. **A proof about an
  abstraction is only as strong as the abstraction's ability to state the property.**
- **A relative canonical path must keep its `./` prefix**, or `./C:` (a directory named `C:`)
  re-resolves as the whole of drive C. That is a widening.
- **Federation: a lease token cannot cross brokers, audit chains cannot merge, and revocation dies at
  a partition.** Read `delulu-federation-scope` reasoning in
  `docs/design/CROSS_PLATFORM_VERIFICATION.md` and the RFC before touching broker credentials.
- **RFC 0001 shipped partly during its own comment period.** Open governance debt. Never restate as
  compliance.
- **The hardware adapter (D23) is an operator-supplied subprocess, not specification §5.4's signed
  plugin.** Its provenance is checked before it is spawned — a detached signature, verified, refused
  when bad, optionally required and pinned to a key (D52/D53), recorded in the audit chain on request
  (D66) — but it is not a Verified-class plugin, and **no driver for any real device ships in-tree**.
  Named as a gap, never blurred. *(This line said "no signature check" until 2026-09-28, a year of
  stages after D52 added one — found while fixing ADAPTER-SPELL-1, §11.8's lesson again.)*
- **P19 — the editor was a way in, twice**, and the second needed no click. See §8.
- **2026-08-09 — four restart-resurrection defects.** `broker rotate-key` never persisted the new key,
  so a restart resurrected every "invalidated" token (**ROTATE-1**); a revoked federation certificate
  lived only in daemon memory, so a restart let it re-adopt (**ADOPT-REPLAY-1**); an untrusted adapter
  could OOM the host with an unterminated reply line; and two more parser recursions crashed
  `delulu check`. All four recursive-descent nesting classes are now bounded
  (`DL0210`/`DL0211`/`DL0212`/`DL0213`) — **one fix does not close a class.**
- **2026-08-10 — filesystem containment escaped, and a guard seal gated nothing.**
  **SYMLINK-DANGLE-1**: `canonicalize` fails identically for "a name that is absent" and "a link whose
  target is absent", so the containment walk re-appended a *dangling symlink's* own name as a plain
  component and the write followed it out of the grant. Unlike the hardlink boundary this **is**
  workspace-deliverable — git stores a symlink as a path string. **GUARD-SPELL-1**: `fs_read`/`fs_write`
  guard rules are matched against the runtime's *resolved absolute* path, so a seal written the
  natural relative way (`guard policy set "fs_write:./out/secret.txt" sealed`) could never fire — and
  the CLI answered `ok`. Witnessed: the program wrote the sealed file. **A seal that reports success
  while gating nothing is worse than no seal.**
- **2026-08-10 — the runtime could be crashed by a valid program.** `MAX_DEPTH` bounds *call* depth
  and nothing bounded *data* depth, so a recursive value a few million deep aborted the host during
  teardown, **after the program had finished** (**INTERP-DROP-1**). That violates this project's own
  `ref.rule.runtime.faults-are-diagnostics`. Fixed by an iterative teardown on the variant payload.
- **CONTAIN-TOCTOU-1 — closed on 2026-09-27 by FS-RACE-1 (V2 P5c).** Containment was check-then-open:
  a *concurrent* writer into a granted directory could swap a checked path for a link in between, and
  the V2 testers' race did it (75 of 150 runs wrote outside the grant, plain and `--sandbox`). This
  section once called it an accepted residual whose fix, `O_NOFOLLOW`, the project refused as
  platform-dependent. It was built instead: `delulu-runtime/src/beneath.rs` opens the approved
  canonical path one component at a time from the filesystem root and follows no link (`openat` with
  `O_NOFOLLOW`, `O_PATH` on Linux; `NtCreateFile` relative to the parent's handle with
  `FILE_OPEN_REPARSE_POINT` on Windows), and every file effect goes through it. Keep the deployment
  rule anyway, as defence in depth — grant directories only the program's own user can write — and
  hardlinks remain their own documented case (`docs/DEPLOYMENT.md` §5).
- **The same-uid boundary is category 7 and cannot be closed by code.** To the kernel, a process
  running as your user *is* you. Strict anchored-root mode raises the bar (and, since 2026-08-10, is
  usable, audit-recorded and `doctor`-checkable) but its own residual is a same-user-writable policy
  file. The real boundary is a **separate OS account** — verified with a real second UID, and written
  up with the exact commands in [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md).

- **V2's findings (2026-09-17 onward), each witnessed failing on the old code and fixed;
  `V2_LOG.md` has each in full.** **GUARD-SCOPE-1**: Guard path rules matched by exact equality, so a
  seal on a directory sealed the entry and nothing inside it. **GUARD-ALIAS-1**: the broker and the
  Guard judged the *lexical* path while the filesystem followed a junction or link — both now decide on
  the pinned, resolved path. **FS-RACE-1**: above. **AUDIT-WRITERS-1**: several writers each cached the
  chain head and broke an untampered chain — one append lock, and catch-up under it.
  **AUDIT-LOCK-TAKEOVER-1**: that lock was taken over after five seconds, so a writer that was only
  slow lost it mid-append — now the operating system's lock, never taken over. **SANDBOX-STOP-1**: a
  ceiling that stopped a guest went unnamed, and exits disagreed with reports. **SANDBOX-CPU-LATE-1**:
  Windows' job time limit fired at about twice the budget — the host now polls the job's accounting.
  **GUARD-STALE-1**: an old denial answered for a new request. **VERIFY-FABRICATED-1**: `Secret.verify`
  under a lease returned a made-up `false` — now computed by the broker, Guard-gated per secret.
  **SECRETS-STALE-1**: the broker never re-read a secret stored while it ran. **JSON-EXIT-1**: `--json`'s
  fallback envelope said exit 2 and "usage" for every failure. **ADAPTER-SPELL-1** (2026-09-28, the
  first cloud routine run): a hardware driver's signature was checked on `./NAME` and the driver started
  as `NAME` from `PATH` — a pinned key vouched for a file that never ran; now resolved once, and that
  file is both verified and started. **CONFIRM-ORDER-1** (2026-09-28, routine run 2, PS-E-01): the
  host's FIRST channel frame was the program, so a guest that never confined itself had already
  received it — now `/3`, the program sendable only from `boundary.rs`'s `Confirmed`. And, from that
  run's red-team pass: **GUEST-WAIT-1** (a failed channel's guest was waited for, not ended),
  **RAN-SENT-1** (`ran` meant confirmed, not sent), **GUEST-TEXT-1** (a guest's words forged audit rows
  and filled the chain), **PIPE-WRITE-1** and **PIPE-FLOOD-1** (an external launcher's pipes unbounded).
  **Routine run 3 (2026-09-29, PS-E-03), each red under an escaped guest — a child applying the guest's own
  lock-down, then raw calls:** **GUEST-SOCKET-1** (UDP, netlink and Unix sockets, and a connect to the
  operator's own socket, while every Linux report said `network: only the channel` on a TCP-only rule),
  **GUEST-PROC-1** (another process's `environ` through `/proc`), **GUEST-DEV-1** (the operator's terminal
  through `/dev`), **GUEST-SYSCALL-1** (memfd, io_uring, userfaultfd, pidfd, the new mount API, a user
  namespace through `clone`); **HOST-DUMPABLE-1** (a serving host's `environ` read by a process of the
  same user — witnessed as a non-root user, since root reads everything); and **GUEST-TIOCSTI-1** (the
  operator's terminal is the guest's controlling terminal, and `TIOCSTI` typed into it — the escape class of
  CVE-2017-5226; the rule compares 32 bits, because the kernel truncates the command); and
  **GUEST-SIGNAL-1** (signals to any process of the same user — its host's group, or `kill(-1)`); and
  **GUEST-PROCESS-1** (the same processes' limits, priority, CPUs and scheduling, changed by pid); and
  **LANDLOCK-TRUNCATE-1** (below Landlock ABI 3 the report said writes denied: the posture matched "no file
  writes" INSIDE "no file writes but truncation" — a pin written to prove H5 held went red).
  **Routine run 5 (2026-09-29, PS-E-04): LAUNCHER-SPELL-1** — `external:NAME` left the lookup to the
  operating system, whose `PATH` search honours `.`, so a `NAME` planted in the working directory ran while
  the report named `NAME`: ADAPTER-SPELL-1's shape again, one feature later. Now `resolve_driver` resolves
  it once, the bytes are hashed and reported, and on Linux the descriptor hashed is the file started.
  **TERMINAL-TEXT-1** (the same run): a PURE program — no effect, no grant — wrote escape sequences to the
  operator's terminal through `assert_eq`'s values (title, line erase, a forged `sandbox:` line; OSC 52
  sets a clipboard), and the same bytes came through a refusal's quoted path, a test's name and a quoted
  source line. Found by asking where a guest's standard error goes. Escaped at the printers now.
  **Routine run 6 (2026-09-30): FRAME-DRIP-1** — every channel reading a peer DeluluLang does not trust
  bounded each READ and none the frame: a peer sending one byte just inside the read deadline held it open for
  ever. On the broker daemon (one connection at a time) a `Status` behind a client dribbling one byte every 2 s
  waited 23.7 s, its whole life — though IPC-1's fix (2026-08-08) had recorded the indefinite hang closed and the
  serve loop's comment promised a dribbler dropped; on a foreign call, foreign code dripping its reply held the
  host past 20 s. `delulu_runtime::channel::Within` owes each frame whole (D-V2-73). The same run closed
  PS-E-04's Windows window (D-V2-74): the launcher was hashed through a `File::open` that shares deletion, so a
  launcher renamed over while it was hashed was started — now held sharing reads only until the run ends.
  **REQUEST-HANG-1 and PROBE-DRIP-1** (the same run, D-V2-75): `brokerd::request` — every custody op of a daemon
  run and every operator command, the e-stop's revoke among them — waited without a bound on a broker that accepted
  and never answered; and the dead-man probe's 1 s bound was per read, so an answer dribbled onto the broker's socket
  kept an arm moving after an e-stop that printed "revoked" (found by the red-team pass on FRAME-DRIP-1, whose own
  record had called the probe bounded). Both owe the whole answer within a bound now. The same pass found
  **BROKER-RELDIR-1**: `broker start` with a relative state dir spawned the daemon INSIDE it with the same relative
  words — it served at `st/st`, the caller said "did not come up", and the daemon was left running (fixed: made
  absolute at the edge). Its other findings — an unbounded reply write, the registry's unbounded connections and
  crash, a FIFO in the audit directory — are RW 4.35–4.40, each re-run by the head chef or marked code reading.
  **REPLY-HOLD-1** (RW 4.35, closed by routine run 10, D-V2-86): the daemon's REPLY had no bound — a client that asked
  for a large answer and never read it held the one-connection loop, the e-stop's revoke behind it; the whole reply is
  owed within 5 s now on every OS (Windows by the pipe's own quota accounting, after a first design failed on the
  runner). **ANSWER-HOLD-1** (RW 4.37, routine run 11, D-V2-88): the same hole on the sandbox's own channel — the host's
  answer to a guest had no bound, so a guest that asked for a file's text and read nothing held its host with nothing to
  end it (a guest that is not reading spends no processor time); every guest socket has a write deadline now and each
  answer is owed whole within the frame deadline. The registry's was closed the same run (**REGISTRY-BOUNDS-1**, D-V2-76): one idle connection stalled every client and
  `Content-Length: 18446744073709551615` crashed `delulu-registry serve` — a thread per connection and `Limits` now.
  And **AUDIT-FIFO-1** (D-V2-77): a FIFO named like a day log hung every reader of the audit chain, `broker start`
  included — ATTEST-FIFO-1's shape in the audit; read only if regular now. **The secret store had it too** (routine
  run 7, RW 4.38's rest): the daemon re-reads the store before every secret operation on its one-connection loop, so a
  FIFO there held the e-stop's revoke; and a FIFO with a READER would have received the store's secrets on `secrets set`.
  One shared reader now (`audit::read_regular`), and a write that judges what it opened before it truncates.
  **Routine run 8 (2026-09-30): SCOPE-HIDDEN-1** — `authority --grants` named `fs.read=./data` alone for a program that
  also read `./secret` through a helper's `r: Root`: the scope walk matched the receiver `root` by name and printed a
  placeholder only for a kind with no literal. The runtime refused the unnamed path; the review list under-reported. A
  hidden site now leaves its kind's placeholder beside the literals (D-V2-81).
  **Routine run 12 (2026-10-05):** a macOS guest had **no memory ceiling** — macOS refuses `RLIMIT_DATA` — so one
  allocating without end ran until another ceiling ended it (red on a runner); the host samples its peak footprint every
  5 ms now and ends it (D-V2-90). And **RUNDIR-PERM-1**: every macOS sandboxed run printed a false "not owner-only …
  this filesystem does not enforce POSIX permissions" — the host made the run's directory 0755 and the guest could not
  narrow it; made 0700 by the host now, never adopted (D-V2-91). And **SILENT-QUEUE-1** (RW 4.40, D-V2-94): N clients
  that connected to the broker and said nothing delayed every client behind them — the e-stop's revoke included — by
  about 5·N s (six held a `Status` 30.6 s); each connection is read on a thread of its own now, and only whole requests
  reach the one handler.

**The search key that found four of the 2026-08-10 defects, worth applying to anything new:** a
security decision made on an **unnormalized or unresolved representation**, walked past by a different
*spelling* of the same thing — a dangling link, a `..` left in a comparison, a relative `PATH` entry,
a relative guard pattern. Ask it of every string compared to decide a security outcome: **what else
spells the same thing?**

- **2026-10-09 — AUDIT-REFUSAL-1: the chain said a refused command was allowed** (routine run 15). The broker records
  `use allow` for a device's identity; the run's envelope refused the command afterwards, on stderr only — three refused
  commands read as three allowed uses. Fixed: a `deny` citing the `allow` it overrides (D-V2-100). **AUDIT-SEQ-1** (the same
  run, RW 4.44): the daemon counted from 1 on every START and a sandboxed run's host numbered "last + 1" outside the lock —
  one verified chain read `1, 2, 3, 1, 2, 3`, and `audit export --since` dropped a revocation. Closed the same run
  (D-V2-102): the log settles each seq under its lock, and the daemon takes the chain's next seq as a floor. **Its
  residual, RW 4.48, closed 2026-10-10 by routine run 16 (D-V2-105)** — the daemon holds the lock from that floor to its
  write (2–3 wrong references in 72 parallel runs before, 0 after); and **AUDIT-DAY-1** the same run: a record stamped before
  midnight but written after another writer's post-midnight record was filed in the earlier day's file, and the whole chain
  read as broken — filed under the later of its day and the chain's now.

- **2026-10-09 — ACTOR-CUSTODY-1 (HIGH, RW 4.47) — closed 2026-10-10 by routine run 16 (D-V2-103)** — found by run 15's
  red-team pass: an actor's worker interpreter had no custody and no device broker, so under daemon custody a revocation did
  not reach an actor's uses and none was recorded — and in EVERY mode its actuator commands were answered `Ok` without
  reaching a device (no lease, no rate bound). Each worker now holds a custody client for the run's node and shares the
  run's brokers; five witnesses, red first. **Its sibling, PLUGIN-CUSTODY-1 (RW 4.50), witnessed and closed the same run
  (D-V2-104):** a plugin export's nested interpreter had a fresh allow-all custody and no device broker — and under the
  daemon no plugin could be called at all (PLUGIN-DAEMON-1: the daemon's custody never answered liveness, so every call was
  `Revoked(0)`); fixing that made the unrecorded uses reachable, 0 of 2 in the chain. It shares the host's custody now.
- **2026-10-09 — HTTP-SCHEME-1** (routine run 15): the daemon's custody gate parsed a host out of `http://host/x` as
  `http` — a plain-http fetch died DL0904 under the daemon where embedded custody returns `Refused`. Two custody modes that
  answer a program differently are a defect even when both fail closed. Fixed: no custody op for a URL the effect refuses
  first.

- **2026-09-30 — AUDIT-TEXT-1: the investigator's and the approver's screens printed an adversary's text raw.**
  TERMINAL-TEXT-1 escaped a program's strings where they are printed live; `audit tail`/`query` (a use's target is a
  path the program chose) and `guard pending` (a request's `why` is the agent's) print them from storage, and did not.
  A leased program's file name cleared the investigator's screen and could forge audit lines; an agent's `why` could
  replace its real request on the owner's screen with another. Escaped per field since D-V2-79 — and `grants tree`,
  `list` and `inspect` the same run (RW 4.42): a delegation's path with a line break forged a node line on the owner's
  screen.

### 11.5 Operational traps, recorded because each one cost time

Beyond those in §8: **WSL `nohup setsid` detached jobs do not survive** — use the harness's background
mechanism. **PowerShell 5.1 `*>` writes UTF-16LE** — `iconv` before grepping. **`cargo test` stops at
the first failing target** — always `--no-fail-fast`. **Freeze the tree during verification**; editing
while a suite runs makes the Survey stale and produces three `doctor_cli` failures that look like
product defects.

Added 2026-08-09/10, each paid for the same way:

- **A pipeline's exit code is not cargo's.** `cargo test … | tail` reports the *pipe's* status, so a
  failing suite reads as exit 0. That is how the core-invariance gate stayed red for a day while a
  campaign was described as green (**CORE-SNAPSHOT-1**). Capture `$?` from cargo directly.
- **A gate outside `cargo test` rots silently.** `delulu fmt --check docs/book/samples` is declared in
  `ci.yml` and recorded as passing; it had been failing since the formatter changed its canonical
  effect-row order. Run the *declared* gates, not just the suite.
- **A falsification that does not change the binary proves nothing.** One guard-removal patch silently
  missed (CRLF vs LF in the match text) and the test "passed" against supposedly-broken code — caught
  only because a sibling test failed and the pair disagreed. Confirm the edit landed, not that the
  runner ran.
- **A witness that never ran is not a negative result.** A guard test reported "sealed: no file" when
  in fact `run` had rejected an unknown flag and the program never executed. Check the witness
  produces the *baseline* effect before believing its refusal.
- **Read a document's structure before "fixing" what a checker reports.** The obvious fix to
  SURVEY-HEADING-1 — adding rows to the ledger table — would have made the note go away by *misfiling*
  a 2026-08-03 finding into a 2026-07-24 ledger. The defect was in the checker. A discrepancy marked
  "for a human to judge" means judge it.
- **A stale binary is not evidence.** A background `cargo test` holds `delulu.exe`, so a concurrent
  `cargo build` fails and the next run silently uses the OLD binary.
- **Never quote a `delulu doctor` check-count.** It varies by environment — a state directory or audit
  log that exists adds checks (17 on Windows here, 14 on Linux). Say "all checks pass".
- **A Python heredoc that prints an emoji dies on Windows cp1252 *before* it writes.** One doc patch
  reported two successful replacements and saved nothing. Prefer the editor for Unicode content.

Added in V2 (2026-09-17 → 2026-09-28):

- **Freeze the tree during verification — again.** On 2026-09-28 the head chef rebuilt one crate while
  the full suite ran; that run was void and had to be repeated. Edit nothing and build nothing until the
  suite reports.
- **Regenerate the Survey after the LAST edit.** A regeneration before a final edit leaves four
  stale-map failures (`doctor_cli` three times, and the freshness test).
- **Activated is not executed.** A run counts once its result is read and recorded: `gh run view --log`
  for a run, `gh api …/actions/jobs/<id>/logs` for one job.
- **CI's literal skip token anywhere in a commit message skips the push run** — it happened once, and a
  manual run recovered it.
- **Heredocs mangle backslashes and non-ASCII.** Write patch scripts and commit messages to files
  (`git commit -F`), and look for U+FFFD after any scripted edit.
- **macOS differs where it matters:** unix-socket paths are limited to 104 bytes (keep test sockets under
  `/tmp`); a socket in a directory answers `EOPNOTSUPP` (102) where Linux says `ENXIO`; the clock resolves
  microseconds, and two tests' scratch directories collided until a counter joined the timestamp.
- **Windows 8.3 short names** spell a path differently from its long form, and a CI runner's working
  directory can be an 8.3 spelling: resolve before comparing.
- **Miri is about a hundred times slower** — which is how it found AUDIT-LOCK-TAKEOVER-1. Shrink a test
  under `cfg!(miri)` only where its size is not the witness (D-V2-45); a test whose witness is the wall
  clock is ignored under Miri and runs natively on every push. `miri-slow` jobs get 240 minutes.
- **Measure an OS limit before claiming it** — Windows' job time limit fires late; macOS does not enforce
  `RLIMIT_DATA`, so no memory ceiling is claimed there. A report states only what was measured.
- **A sandboxed run writes its audit records only where the chain already exists**
  (`guest.rs::audit_sandbox`): a test that reads them creates `<state>/audit` first, or it reads an empty
  chain and fails for the wrong reason (routine run 2).
- **A witness that races two writes lets a mutant live** (routine run 2, M8): a fake guest that closed
  its input before reading the host's acceptance let a race decide which write failed, and `ran =
  confirmed` passed the test. A fake peer reads everything it must before it goes away — then the one
  write left is the one the witness is about. Run a mutant more than once when timing is involved.
- **In `sh`, a background command's standard input is `/dev/null`** unless it is redirected from a
  descriptor opened before it (`exec 3<&0; cat <&3 > file &`). A fake launcher that listens in the
  background otherwise hears nothing, and the absence it then "proves" is vacuous — so a capture
  witness also asserts that it captured SOMETHING (routine run 2, PS-E-01).
- **A macOS- or Windows-only defect is witnessed on a runner, off `master`** (routine run 3):
  `witness.yml` runs one test on one runner at a branch — read it red there, then the fix green there,
  then fast-forward `master`. Lint the macOS and Windows code in the VM first (`scripts/check-other-os.sh`). A witness
  run's test lines sit 70–85 lines before its log's end (the cache save and git's cleanup follow), so
  read it with `tail_lines` ≈ 90.
- **`git stash pop` keeps a staged new file staged** (routine run 3): the next `git add <one file>` and
  `git commit` carried it too (`37828ab`). Read `git diff --cached --stat` before every commit.
- **A green read covers every target that runs the changed code on that OS** (routine run 3): the macOS
  watcher was read green on three targets; a fourth, `sandbox_external_cli`, went red on `master` — the
  watcher's start changed a race's winner and a raw `Broken pipe (os error 32)` reached the operator.
  `witness.yml` takes several targets, or `all`.
- **A probe that parses its own output must fail on an attempt it did not hear** (routine run 3): the
  escaped-guest harness lost its first `NAME=` line to libtest's name line, and a key filter without
  digits dropped `CLONE3` — both would have passed as "nothing reached". Unreported now fails.
- **A pin can find the defect** (routine run 3): a test written to prove H5 already held went red — the
  posture matched "no file writes" INSIDE "no file writes but truncation". Write the pin even when reading
  says it holds.
- **Two checkouts sharing one `CARGO_TARGET_DIR` share `target/debug/delulu`** (routine run 3): the last
  build wins, so the other tree's integration tests can run the wrong binary. Give a worktree its own
  target, or rebuild (`touch src/main.rs`) before testing. And `a && b && (suite) &` backgrounds the WHOLE
  chain — its first commands' output is lost.
- **A records-only commit can go red** (routine run 4): the supply-chain gate reads RustSec as it is on the
  day, not the diff, so an advisory published between pushes reds the next push, whatever it changes
  (RUSTSEC-2026-0315/0316 on `9fc4d86`). Witness it in the VM: `cargo install cargo-deny --locked` (about
  four minutes), `cargo deny --all-features check advisories` before and after the fix.
- **A fake peer keeps the protocol it fakes** (routine run 4): the attestation replay test's launcher wrote
  with `cp`, which the protocol forbids — the host reads the document the moment it exists — and macOS lost
  that race once. Witness a race by holding its window open (a `sleep` between creating and filling), never
  by re-running until it shows. **Again in routine run 7:** a fixture wrote the sandbox records' generation as an
  integer where the host writes 64 hex characters, and the export read the fixture's spelling. Beside every fixture
  that fakes what the host writes, keep one witness that runs the host for real.
- **An ignored gate is not in the suite's count** (routine run 4): `delulu-wasm/tests/differential.rs`, the
  two-engine differential, is `#[ignore]`d and runs only in `heavy-gates`, so a green suite says nothing
  about it. A change to either WebAssembly engine runs it by hand (release, 50,000 programs, ~150 s).
- **A comment that names a test is a claim — grep for it** (routine run 4): the doc comment on
  `higher_order_callback_arg` named a pinning test that never existed, and one of the five entries it
  claimed pinned was pinned by nothing (a mutant removing it passed, another check refusing the same programs
  for another reason). A mutant per entry finds such a hole; a green suite does not.
- **A failure message is all the evidence a CI run keeps** (routine run 5, RW 7.17): `estop_cli`'s control test
  went red once on arm64 printing only the program's `REVOKED (operator-revoke)` — a line that says the same for
  every dead probe — while the journal holding the real reason went to a stream the assertion never printed.
  Print the reason with the verdict; a red that cannot be read cannot be root-caused.
- **A new test runs on every OS, and its own assumptions are platform rules** (routine run 5): TERMINAL-TEXT-1
  was OS-neutral code, so it went to `master` unread on the other runners — and its refusal witness asked for
  a path holding `:`, which Windows refuses first as an alternate data stream (DL0904), so the test's DL0703
  never came. The product held; the test was red on `master`. Read a slice's NEW tests on every runner before
  `master` moves, whatever its code touches.
- **A mutant that survives every witness names the witness that is missing** (routine run 5): M16 — the
  relay's last line not waited for — passed four tests, because the relay thread wins that race in practice;
  a launcher that left a writer holding the stream for a second after the guest had gone made it red, 3 of 3.
- **Ask of every string a program chose: where is it printed for a person?** (routine run 5,
  TERMINAL-TEXT-1) — the effect row governs what a program DOES; the terminal it is reported on is outside
  it, and an assertion's message was a way to it for a program with no effects. JSON escapes exactly; a
  human renderer must escape too.
- **`environ` in a `pre_exec` step is not the command's environment** (routine run 5): a step that execs by
  itself (`fexecve`) and passed `environ` started the launcher with none of the variables the host set on
  the command — caught only because a test asserted the launcher's environment. Build the environment from
  the command (`vars_os`, then `get_envs`) before the fork.
- **Hold a race's window open with work the code must do anyway** (routine run 5): the launcher-swap
  witness made the host's hash slow (a 64 MiB launcher) and swapped the path the moment `/proc/<pid>/fd`
  showed the host holding the file — no hook in the product, no re-running; the mutant failed 3 of 3.
- **A deadline on each read is not a deadline on the message** (routine run 6, FRAME-DRIP-1): a peer that sends
  one byte just inside it holds the message open for ever. For every timeout on a channel, ask what a peer
  answering JUST inside it can hold — and test that peer, not a silent one: IPC-1's witness was a client that
  stalled, so the dribbling client the same comment named was never tried.
- **Windows' `/proc/<pid>/fd` is `NtQueryInformationFile(FileProcessIdsUsingFileInformation)`** (routine run 6):
  asked through a handle opened for `FILE_READ_ATTRIBUTES` only — which takes part in no sharing check, so the
  witness neither blocks the host's open nor is blocked by it — it lists the processes holding a file open. It let
  a Windows race witness act at the moment the host held the launcher, never by re-running. And the standard
  library's `File::open` on Windows shares reading, writing AND deletion: hold a file against change with
  `share_mode(FILE_SHARE_READ)`.
- **On Windows a busy broker pipe REFUSES a second client after a second; a Unix socket queues it** (routine run 6):
  `broker_transport::connect` waits 1 s on `ERROR_PIPE_BUSY`, then fails closed. A test that puts a client behind
  another must let it ask again on Windows — FRAME-DRIP-1's broker witness went red there for that reason alone.
- **The VM's 30 GB fills** (routine run 6): cross-target lint builds (3.6 GB) and incremental caches (6.8 GB) ended a
  suite on ENOSPC — delete `target/<triple>` after `check-other-os.sh`, run suites with `CARGO_INCREMENTAL=0`.
- **Install the pinned toolchain before anything else runs `cargo` or `rustup`** (routine run 6): the first
  `cargo` call installs it, and a `rustup target add` started beside it raced the install — the toolchain was left
  with `cargo` "not applicable" and a component conflict, and had to be reinstalled.
- **A string that is STORED is printed later — ask where** (routine run 7, AUDIT-TEXT-1): TERMINAL-TEXT-1 escaped every
  live surface, and the audit chain and the Guard's queue kept a program's and an agent's strings to print a day later,
  raw, on the screens where a person investigates and approves. For every string an untrusted party can put into a
  store, find each command that prints the store for a person.
- **A mutant loop leaves the binary built from its last mutant** (routine run 8): the loop's `cargo test` rebuilt
  `target/debug/delulu` with each mutation, and restoring the source byte for byte rebuilt nothing — a by-hand check
  after it read mutant M71's unquoted host. `cargo build` after every mutant loop, before any by-hand run.
- **Choose a falsifier from the checker's own documented cases** (routine run 8): two filesystem mutations of the
  OpenShell export (an added `/tmp`, a granted read made writable) came back `unsupported` from OpenShell's prover, not
  `exceeds_boundary` — its documentation says it compares only paths both policies name — so neither could ever have
  been red. And read a tool's machine channel alone: its solver's warnings on stderr, merged with the JSON on stdout,
  made every answer "unparsed" on the first run.
- **A flag documented for a command reaches every verb of it** (routine run 8): the dispatcher refuses a shared flag a
  command's help does not document, so documenting `--grant` for `sandbox policy --format openshell` let `sandbox
  status --grant x` through to a verb that ignored it (exit 0) — found by the suite, through a sibling test's changed
  wording. A verb that does not take a flag its command documents refuses it itself.
- **A workflow is dispatchable only once it is on the default branch** (routine run 8): `run_workflow` for a file a
  branch alone has answers 404. A new by-hand workflow lands on `master` with its slice, then runs.
- **A guarantee a report claims needs a witness of its own** (routine run 3): Linux's "killed with the
  host" had been claimed since PS-A-04 and no test killed a host to see it — the macOS witness, written
  for both, was the first; it also proved the death signal with a mutant.
- **A gate that reads the whole resolved graph offline passes only while the cache holds it** (routine run 10):
  `egress_features`' `cargo metadata --offline` needs every platform's crates and a build fetches only the runner's;
  a warm rust-cache held them, until a new stable Rust (1.99.0, 2026-10-01) changed the cache key — four nightlies red on
  every operating system, and a red job saves no cache, so it never recovers alone. A test's precondition belongs in
  the workflow (`cargo fetch --locked`, D-V2-84), never in the cache's luck. A nightly is the only run that sees the
  world change while `master` does not: read every one since the last run, not only the push runs.
- **A socket's write deadline is per buffer, not per call** (routine run 10, REPLY-HOLD-1): Linux applies `SO_SNDTIMEO` to
  each buffer a `write` waits for, so one 4 MiB `write` to a peer that frees a little room every half second stays in
  the kernel for a minute — no check between writes ever runs. Bound the WHOLE write as well as each, and offer the
  transport a bounded amount per call (`channel::Within`, 64 KiB). Witness with a slow reader, not only a silent one:
  the first fix passed the silent client and failed the slow one.
- **A run can end before its records — look at every branch** (routine run 10): routine run 9 pushed `f446bfa` (PS-E-05
  (b)) to its harness branch, dispatched its runner reads and stopped — nothing on `master`, no entry anywhere, and its
  reading inside OpenShell red. Found only by listing every branch's commits `master` lacks; the clone is shallow, so
  unshallow first or every branch looks hundreds of commits ahead. Merge a recovered commit rather than cherry-pick it:
  the runs that read it name its hash.
- **A simulated wall simulates every layer of the real one** (routine run 10): run 9's test simulated OpenShell's wall as
  a seccomp filter and passed; the real wall is also Landlock with no `/proc`, and the guest's check READ `/proc`. And
  `unwrap_or_default()` on a security read turns "could not look" into "no" — here it failed closed, but it hid the cause
  behind a plausible message. Ask the kernel where the kernel can answer (`PR_GET_SECCOMP`); where a file must be read,
  say "unreadable", not "absent". Before trusting a simulation, list the real wall's layers from a real run's evidence.

- **A way out that a refusal names is a promise — run it** (routine run 11): since D-V2-59, `hostile-agent`'s DL1408 told
  the operator "an external launcher whose attester vouches for it", and no attestation could ever meet a property — the
  level-3 answer was `unknown` whatever was signed. A refusal's advice is a claim about the code, like a comment that
  names a test (run 4): for each way out a message offers, a witness that takes it.
- **A bound on one direction of a channel asks the question of the other** (routine run 11): FRAME-DRIP-1 bounded what the
  host READS from a guest (run 6), REPLY-HOLD-1 what the broker WRITES (run 10); what the host writes to a guest stayed
  unbounded until ANSWER-HOLD-1 — recorded as "code reading" by the very pass that found the broker's. When a fix bounds
  one direction, look at the other direction of every channel the same code serves.
- **A mutant must change the ANSWER, not only the source** (routine run 11): M93's first form wrote the attester's claim
  into the measured branch's map before the loop that fills it — the loop overwrote it, the binary behaved as before, and
  the mutant "survived". A survivor is first a question about the mutant: read what the mutated code returns.
- **A sampler's interval IS its ceiling's resolution — measure the overshoot before claiming the ceiling** (routine run
  12): the macOS memory sampler at the ordinary run's 25 ms let a guest reach 247 MB against a 64 MiB budget on a runner;
  at 5 ms, 69–93 MB on two reads. Print the observed value in the witness, so the first green read is also the measurement.
- **A false alarm is a defect — read the WHOLE log of a green witness** (routine run 12, RUNDIR-PERM-1): every macOS
  sandboxed run had printed a permissions warning blaming the filesystem; it sat in the stderr of a witness written for
  something else. And **lint the other OSes after the LAST code edit**: a function added after `check-other-os.sh` ran
  compiled on Windows with `unused_mut` — seen only in the Windows read's build output.
- **A deadline per connection on a loop that serves one at a time is that deadline times N** (routine run 12,
  SILENT-QUEUE-1): FRAME-DRIP-1 bounded each request at 5 s, and six silent connections still held the next client 30 s.
  Ask of every bound on a serial loop what N peers at the bound cost the one behind them.
- **A bound you add is a resource you count — count it until it is released** (routine run 12): RW 4.40's fix capped
  readers at 64 but released a reader's count when it finished READING, not when it handed its request over, so 300
  whole requests behind a busy handler grew 240 threads. Found by re-reading the fix after it was on `master`, against
  the question: what does a peer that does everything RIGHT, many times over, cost?
- **A mutant that survives on a second wall is a finding about the walls, not a gap** (routine run 13, M123): removing the
  capability's own envelope check left the broker's, which caught the command — so the witness stayed green. Name the
  second wall, then remove both (M123b, red) to show the witness can fail at all.
- **A flag a path starts to apply asks the dropped-flag question again** (routine run 13): moving `--approved` onto the
  sandbox's allowlist turned the sandbox's "refused, not dropped" test red — rightly: beside a non-hardware run the flag
  did nothing, and the ORDINARY run had dropped it in silence all along. When a path gains a flag, ask what the flag does
  when its precondition is absent, on every path that takes it.
- **The tree was edited mid-suite a third time** (routine run 13): a test file reworked while the suite ran made the
  Survey's freshness test red for that reason alone. `scripts/suite.sh` now runs the suite the routine's way and says
  `TREE-MOVED` when the tree changed under it.
- **A mutant that survives every witness at one level may be one only another level can see** (routine run 13, M128):
  starting a guest's device broker before its launch charged the launch against the first heartbeat — invisible at L1,
  where a process guest starts inside every heartbeat the witnesses use, and red at L2, where a VM's boot revoked a 150 ms
  lease first. When a design choice is about time, witness it where the time is largest.
- **`delulu` has no library target** (routine run 13): its unit tests are `cargo test -p delulu --bin delulu FILTER`;
  `--lib` answers "no library targets found" with exit 101, which reads like a red suite. In `witness.yml` it is `bin:delulu`.
- **`git revert` takes no `-q`** (routine run 12): the revert failed, the next `git commit --amend` re-labelled the
  MUTANT commit as its own revert, and the push was refused as a non-fast-forward. Nothing was lost (reset to the pushed
  branch, revert again — never a force-push); read `git log -1` after every revert before amending anything.

- **A new EXAMPLE is a new case in four gates, and no witness target reaches them** (routine run 14): adding
  `examples/line_driver` turned `cli_contract` (`delulu fmt --check examples`), `edit_cli` (the Atlas round-trip —
  `delulu edit` re-prints what it inserts, so an item's own text round-trips only in a canonically formatted file),
  `atlas_chain` (its snapshot corpus is asserted to BE the corpus) and `core_invariance` (a new example is a new
  recorded answer) red at once, while the runner reads of the slice's own tests were all green. Run `delulu fmt` on a new
  example and bless both corpora in the same commit — and read a bless: 150 lines ADDED and none removed is a new case,
  while one removed line is the core's answer moving.
- **A model switch mid-session is a fact for the commit trailer** (routine run 14): the runtime moved the session from
  Opus 5.5 to Opus 5 (1M context) partway through, with no classifier notice. `CLAUDE.md`'s rule is about what is
  ACTUALLY running, not only about a fallback: check before every commit message, and say so in the records.
- **A mutant must be able to compile** (routine run 14): M146's first form (`Unsigned if false => Some((…))`) left the
  match without the arm the compiler needed and the test run died at build time, which reads like a red witness but
  proves nothing. A mutant that does not compile is not a falsification — it is a typo. Read the mutant's own build.

- **A new SUBCOMMAND is five gates, and two of them only the local suite sees** (routine run 14): the help line, the
  dispatcher's arm and `cli::SUBCOMMANDS` (`completions_cli` proves the three name one set), `json_contract`'s failing
  sweep AND its success table, and `mcp.rs`'s door rule — every command in exactly one of `READ_ONLY` and `EFFECTORS`,
  which exists so a command added later cannot reach an agent's tool unreviewed. Add the rows with the command, in one
  commit. The same run learned the same shape for a new EXAMPLE (four gates, above): in this repository a new SURFACE is
  never one edit.
- **Classify a server by what it hands out, not by what it writes** (routine run 14): `device sim` creates no file and
  performs no effect, and it is still an EFFECTOR — it serves a conversation whose answers a control loop acts on, as
  `lsp` and `mcp` do.
- **A rule that a run must remember is a rule a run will forget — make it a hook** (routine run 15): "regenerate the map
  after the LAST edit" was written down from 2026-09-28, and run 14's closing commit still carried a map built before its
  own sync-log entry was finished — `master` and four nightlies red on every OS for four days, for that alone.
  `scripts/hooks/pre-commit` now checks the map against the INDEX (exported, then surveyed), which also catches the case a
  tree check passes: a map rebuilt from an edit that is not staged. `git config core.hooksPath scripts/hooks` in step 2.
- **"Everything it needs already exists" is a hypothesis — measure the reading surface first** (routine run 15): P8-04's
  build order, read against the code, said a refused device command was already a `deny` in the chain; one scratch daemon
  and a four-line program showed four `allow`s and no `deny`. A design step that builds nothing still gets a witness.
- **A restore by `move` keeps the old mtime** (routine run 15): a mutant loop restored each file from a backup with
  `shutil.move`, cargo saw a source older than its binary and kept the LAST mutant — the green re-run went red. Copy, or
  touch the restored file, then rebuild (§11.5's mutant-binary trap, a second way in). `scripts/mutants.py` does both,
  and ends with a control run that must be green.
- **The cross-OS lint covers only the packages it names** (routine run 15): `check-other-os.sh` linted `delulu`,
  `delulu-runtime` and `delulu-wasm`, and a Unix-only test helper in `delulu-broker` warned as dead code in every Windows
  test build, unread. `delulu-broker` is in the list now; when a package gains `cfg(unix)` or `cfg(windows)` code, add it.
- **A witness of an absence must prove it looked** (routine run 15): "a monitor on a sibling node quarantines nothing"
  passed just as well for a monitor that read nothing; the report now counts `records_read` and the witness asserts the
  run's records were read (mutant M174, which skips every record, is red only because of that assertion). For every
  "nothing happened", assert the thing that would have seen it ran.
- **Every `Interp::new` is a place the run's authority can stop** (routine run 16, ACTOR-CUSTODY-1): an interpreter is built
  with a pass-through custody and no brokers, and only `run_cmd` attached the run's — so an actor's worker, built elsewhere,
  answered an actuator command `Ok` with no device behind it and was invisible to the daemon, in a project whose every other
  path had been hardened. When a feature builds an interpreter, ask what custody and brokers it holds; `grep Interp::new`.
- **A mutant spec over several test targets needs `--no-fail-fast`** (routine run 16): M182's first read named one red
  witness — cargo stopped at the first red TARGET, and the second witness never ran under the mutant. A falsification that
  did not run is not a survivor and not a red. `scripts/mutants.py` adds the flag itself now.
- **A path that fails closed on everything hides what stands behind it** (routine run 16, PLUGIN-DAEMON-1): under the daemon
  every plugin call was refused `Revoked(0)` — safe, so nobody looked, and a decision recorded the path as working. The moment
  liveness was answered, the plugin ran under an allow-all custody (0 of 2 uses recorded). When a fail-closed path is made to
  work, witness again what it was standing in front of.
- **A key derived from a timestamp is an order claim, and a clock is not an order** (routine run 16, AUDIT-DAY-1): the audit
  chain filed each record by its own stamp's day, and a record stamped before midnight but written after another writer's
  post-midnight one broke the whole chain. Found because a test's FIXED timestamp fell on yesterday — a fixed date in a test
  is a date. Ask of every file, partition or bucket chosen by a time: what does a record stamped before the last one do?
- **A new witness is an edit — the fourth time the tree moved under a suite** (routine run 16): the next slice's witness was
  appended to a test file and its target compiled while the full suite ran. Caught by the head chef, not by a tool: the file
  was restored before the suite reached that target (which ran its own earlier build, a separate hash), so the suite's verdict
  stands — but `TREE-MOVED` cannot see an edit that is undone. While a suite runs, draft the next witness in the scratchpad.
- **A function you give a new consequence has callers you did not write the slice for** (routine run 16, RW 4.51): `end_of_run`
  revoked every node `attenuate` had minted — and device nodes are minted through `attenuate` too, so a run's arm was e-stopped
  at its end while the watchdog still ran. The slice's witness and three mutants were green; the full suite and three runners
  were red. Before giving a shared function a new effect, list its callers (`grep`, `survey affected-by`) and run their targets.
- **A sous-chef's findings must reach a file as they are found** (routine run 16): the container restarted twice in one run;
  the second killed a red-team pass mid-way, and its findings — held for its final message — were lost with it. Briefs now say
  to append each finding to `FINDINGS.md` in the scratch directory. Its leftover experiment was still worth re-running: it
  measured the daemon waiting on the audit chain beside busy sandbox hosts (RW 4.53).
- **A run's last push is the one nobody reads** (routine run 15): run 14 ended after pushing `75bb33b` to `master` and
  `6dbc54c` to its harness branch, with neither run read; the next run fired four days later. Read the push run of the
  closing commit before the closing entry says "green", or say in "Open / next" that it is unread — and step 1's branch
  listing is what found the stranded commit.

### 11.6 If you are an assistant with memory, keep it current

After verifying work, update `MEMORY.md` and the topic files. One fact per file, with frontmatter, and
a one-line pointer in `MEMORY.md`. Do not record what the repository already says — code structure,
git history, past fixes. Record what was *non-obvious*: the reasoning, the trap, the owner's ruling.

### 11.7 What running agents taught (V2's testing passes, 2026-09-18 → 2026-09-28)

- **Judge an agent by its logs, never its summary.** A Haiku 4.5 tester overclaimed; one reported
  "microVM TTL defect" was its own test mistake, refuted by the head chef's run (DL1402 at L0, L1 and
  L2); a Sonnet 5 tester's "`expose` under a lease can never succeed" was really SECRETS-STALE-1. Re-run
  every claim against the CURRENT binary before it is used or recorded.
- **Read the "non-blocking notes".** The Windows verifier's two side remarks were both real defects
  (JSON-EXIT-1, SANDBOX-CPU-LATE-1), each fixed with a test that failed first.
- **Different operating systems find different defects.** Windows found the 8.3-name and junction
  aliases, Linux the alias race and the FIFO; each missed the other's. Independent testers on two OSes
  found VERIFY-FABRICATED-1 separately — the strongest evidence a pass produced.
- **The brief forbids writing into the repository**, and still one agent wrote into its root: check
  `git status` after every agent. Agents work in a scratch folder outside the checkout.
- **Limits are real.** A Sonnet tester was stopped by the model's weekly limit mid-pass and resumed after
  the reset by message, with its work intact on disk; respawning would have lost it. Before a session
  limit, back up progress and findings (owner, 2026-09-27).
- **An agent's value is independence, not volume.** The head chef's own end-to-end test found
  GUARD-STALE-1 and SECRETS-STALE-1 while writing the fix for an agent's finding — agents and the head
  chef each find what the other misses.
- **No chain-of-thought is kept** — findings, evidence and decisions only (owner, 2026-09-17).
- **A sous-chef's brief can be stopped before the agent starts** (routine run 4): a safety classifier stopped
  the response that was briefing a red-team pass on the guest's filter; nothing ran. Do that kind of pass by
  hand, one witnessed hypothesis at a time, as PS-E-03 did. **Routine run 5:** the head chef's own response
  starting PS-E-03 H6 — an escaped-guest harness for macOS's Seatbelt profile — was stopped the same way,
  before anything of it ran. Twice now escaped-guest work in a routine run has been stopped (the filter's
  red-team pass, H6); H6 may be one to do with the owner at the laptop.
- **The defects are rarely IN the guarantee under test** (routine run 2): a Sonnet 5.5 red-team pass on
  `/3` found the guarantee held in ~85 attempts and 700 fuzzed frames — and seven real defects AROUND it
  (waiting, reporting, text, pipes). Brief a tester to attack the guarantee and to list every oddity.
  Give it a frozen COPY of the binary outside the repository, so the head chef can keep building.

### 11.8 What building and running DeluluLang V2 taught

- **The guest performs no effects.** Every effect is decided and performed on the host, so a new
  isolation backend — process, microVM, an external launcher — never needs a new authority path, and
  the same tests hold at every level.
- **Honesty lives in the report, not the prose.** Each run reports the level it actually got, what the
  host measured (`host_guarantees`, `fully_enforced`), and nothing else; an external launcher is level 3
  with no host guarantee; a guest's self-applied layers count only in words the host checks (RW 4.23).
- **Resolve at the edges; the broker stays lexical (Ruling 2).** The runtime pins file paths and opens
  exactly the pin; the CLI stores grants, rules and requests resolved; the broker compares what it is
  given.
- **Slow and different machines find real bugs.** Miri's slowness, macOS's sockets, Windows' 8.3 names,
  a 4-CPU runner — each found what the development machine never would.
- **A long-lived process holding a copy is a defect waiting** — SECRETS-STALE-1's store read once at
  start; AUDIT-WRITERS-1's cached chain head.
- **A document that names a residual must be re-read when the code moves.** CONTAIN-TOCTOU-1 stayed
  "accepted" in three documents for a day after FS-RACE-1 closed it — found while writing this handoff.
- **Study another system by its contracts and its code, then ask what YOUR design lets you do that
  it cannot** (the OpenShell study, 2026-09-28). OpenShell governs binaries it cannot read, so it must
  intercept sockets and decide per request; a DeluluLang guest performs no effects, so it can simply be
  denied a socket. Copying the other system's controls would have added interception DeluluLang does
  not need and missed the denial it can afford. Read the diagrams from their sources (an SVG is text),
  and compare the other system's list of controls line by line against your own code — that is how
  the six PS-E-03 hypotheses were found, and each is still only a hypothesis until its witness is red.

---

## Where the rest of this file went

On 2026-09-27 (V2 phase P6) these sections moved, verbatim and under the same numbers, to
[`docs/archive/v1/HANDOFF_HISTORY.md`](docs/archive/v1/HANDOFF_HISTORY.md), which is not maintained:
each describes the day it was last written. What replaces each, for what is true now:

| Section | Was | Now read |
|---|---|---|
| the old header | the "current state" paragraph, the last-updated trail, the 2026-08 campaign notes | *Where things stand* above; `docs/DELULULANG_V2/V2_PHASE_STATUS.md` |
| §1.1's run rows | the first six CI runs, each read | `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9; `V2_LOG.md` |
| §3 | the ledger of what was built, V1 and the V2 phase ledger | `docs/DELULULANG_V2/V2_PHASE_STATUS.md`; `CHANGELOG.md` |
| §5 | which document to read, and what each is for | `docs/REPOSITORY_STRUCTURE.md`; `docs/DELULULANG_V2/V2_README.md` |
| §6 | the features, explained | the Book (`docs/book/THE_DELULULANG_BOOK.md`); `docs/for-agents.md` |
| §8 | problems — what is open, and why | `docs/REMAINING_WORK.md` |
| §9 | current numbers, as measured on two dates in 2026-08 | the V2 log's per-phase numbers; `measurements/` |
| §12 | performance, measured | `measurements/METHODOLOGY.md`; the Book's performance chapter |
| §13 | what has been done on each operating system | `docs/design/CROSS_PLATFORM_VERIFICATION.md` |
| §14 | CLI vs compiler vs editor, per audience | the Book; `docs/editors.md` |

§11 stays here whole, although the plan for this rewrite named only §11.1: §11.2–§11.6 are the
project's working rules, environment facts, findings that must never be re-softened and operational
traps — rules in force, not history — and this file is where they are the authority.
