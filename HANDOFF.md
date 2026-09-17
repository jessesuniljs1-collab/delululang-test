# HANDOFF — DeluluLang

**For:** whoever picks this repository up next, human or AI.
**Written:** 2026-08-07, at the end of the P19 ecosystem campaign.
**Last updated:** 2026-09-17 — two planning passes on the owner's commission wrote
`docs/NEXT_EVOLUTION_2026/` (the Next Evolution reassessment; the sandbox and VM isolation pass);
**both await the owner's approval and nothing of them is built** (§8). The same day the owner made
the testing remote public (§1.1). Before that, 2026-09-14: its first push, and CI green on all
three operating systems in one run.
Earlier: 2026-08-23 (`docs/REMAINING_WORK.md`), 2026-08-10 (containment + deployment hardening).
**Repository:** `D:\nelan\DeluluLang` — a Rust workspace, 13 crates, **111,437 lines of Rust**
(measured by the Survey on 2026-09-14, not remembered) — and on GitHub, **publicly since 2026-09-17**
(privately from 2026-09-14): `origin` → `https://github.com/jessesuniljs1-collab/delululang-test.git` (§1.1).
**State:** clean tree; `master`, `rc/1.0.0-drill` and the `v1.0.0` tag pushed to `origin`;
**208 commits past `v1.0.0`** at this update.

> **If you are starting today, read this first.** Two campaigns have run since this document was
> written, and the second changed what you should assume:
>
> - **2026-08-09** — production-readiness sweep; six defects fixed (see `PRODUCTION_READINESS_2026-08-09.md`).
> - **2026-08-10** — containment + deployment hardening (`PRODUCTION_READINESS_2026-08-10.md`).
>   **Eleven defects fixed**, two of them high: a **dangling symlink escaped filesystem containment**
>   (workspace-deliverable via git), and a **guard seal written the natural way gated nothing while
>   the CLI said `ok`**. Also: strict anchored-root mode was **unusable** until this campaign and now
>   works end to end; `delulu doctor` grew a `security posture` section; and
>   [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) now says what a deployment actually protects.
>
> **Current status: PRODUCTION READY WITH DOCUMENTED DEPLOYMENT REQUIREMENTS** — Windows and Linux,
> in the Tier-2 deployment of `DEPLOYMENT.md`. **macOS is now verified by CI (green on its Apple Silicon
> runner, 2026-09-14), but is not covered by this verdict: the Tier-2 cross-account boundary was tested
> with a real second UID on Linux only.**

Read §1 and §2 before touching anything. The rest is reference.

> ### If you are Claude Code, read this first
>
> Assistant memory for this project lives at:
>
> ```
> C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\
> ```
>
> It is **keyed by project path**, so a new session opened at `D:\nelan\DeluluLang` **on this account,
> on this machine** loads the same `MEMORY.md` automatically — you already have it.
>
> But that directory lives **outside the repository**. It does **not** travel with a clone, does not
> exist on another machine, and is not shared with a different account. So everything durable in it is
> transcribed into **§11 of this file**, and this file is the authority if the two ever disagree.
>
> If you *do* have the memory loaded, §11 will be familiar and you can skip it. If §11 tells you
> something `MEMORY.md` does not, trust §11 and update the memory.

---

## 1. Standing rules — these are not suggestions

| Rule | Why |
| --- | --- |
| **Push only to the testing remote — nowhere else.** `origin` is `github.com/jessesuniljs1-collab/delululang-test`, **public since 2026-09-17** (private from 2026-09-14). No other remote, and never rewrite history that has been pushed. **Push every commit there as soon as it is made**, so the local repository and GitHub stay in sync — no need to ask first (owner's standing permission, 2026-09-17). | Owner's instruction, 2026-09-14, replacing the *"NEVER push to GitHub"* that held from the first commit (and is why CI never ran before then). The remote exists for **testing on macOS and other operating systems** (CI) and for **editing from the cloud**, and the owner made it public on 2026-09-17. **The project's final public repository will be a different one, a step the owner takes personally** — do not create one or push to one. Pushed history stays as it is because the documents cite commit hashes throughout. See §1.1. |
| **The final public repository is gated on the owner's decisions — remind him first, then wait.** Before it is created, added as a remote, or pushed to, put the items in §1.1 *Before the final public repository* in front of the owner, with the suggestions recorded there, and get his decision on each. Nothing is pushed there until every one is decided. | Owner's instruction, 2026-09-17: *"before moving to real public repo later remind to make changes to these and remind me that time before even push happens. Do not push to new real repo (future) unless my decision on these are given."* The items are already visible in the public testing repository; the final one is the chance to leave them behind. |
| **Never auto-decide the licence.** | Owner-reserved. Apache-2.0 + NOTICE + TRADEMARK is *recommended* and staged, not decided. Present options and wait. |
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
| **The first run** | Run `34830053479` (2026-09-14), read with `gh`: **3 passed** — clippy, the editor build and the formal models, each on a runner for the first time — **1 skipped, 11 failed**, and every failure accounted for. Two gates had never been able to pass. **All six Miri jobs** ran a bare `cargo miri` under the stable toolchain `rust-toolchain.toml` pins (Miri is nightly-only; the job now says `+nightly`). **`cargo deny`** found two advisories published after its last clean run (2026-08-07) against wasmtime 47.0.3 — RUSTSEC-2026-0268 and RUSTSEC-2026-0269, both in WASI functionality DeluluLang never uses (no `wasmtime-wasi`, no WASI calls), both closed by the patch release **47.0.4**. Two artifacts had been recorded from this machine's disk rather than from git: the **core-invariance snapshot** counted carriage returns that 58 CRLF working-tree files had and the committed LF files do not, and the **Survey** mapped a gitignored `.vsix`. **arm64** ran all 124 test binaries and failed only on those two. **macOS** stopped building `libffi-sys`'s bundled libffi, whose aarch64 assembly current Apple clang rejects, before any DeluluLang code ran; it now links macOS's own libffi, untested until the next run. Full record: `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **The second run** | Run `34836508713`: **macOS built the whole workspace and passed 1,654 of 1,655 tests — the first DeluluLang code ever to run on a Mac.** arm64, clippy, `cargo deny`, the editor, the formal models, and Miri on `delulu-atlas`, `delulu-diag` and the FFI decoder passed. The rest failed where a test met the runner (a speedup criterion on 2-vCPU machines; a PowerShell driver too slow to start inside the adapter's deliberate 2000 ms budget), where it found a real defect (a socket-path refusal that never left the detached daemon; a fresh state directory the broker could not start in), or where it ran out of time (Miri on three crates with no `unsafe`, now nightly/manual as `miri-slow`). All fixed in the next commit; the record is in `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **The third run** | Run `34841317790`, on `28e10e6`: **green everywhere but one Windows test.** macOS end to end — 1,657 tests, the CLI sweep 27/27, the fuzz campaign's 50,000 programs with no trace escaping its row — and the same on Linux x86-64 and arm64, plus clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder. Windows passed 1,646 of 1,647: `adapter::tests::a_garbled_reading_is_an_error_never_a_none_and_never_a_number` failed on the runner (it passes locally, and passed there in run 2): its PowerShell fake driver started too slowly for the adapter's 2000 ms budget. The drivers are Python now — run 4 confirmed it. |
| **The fourth run** | Run `34844151767`, on `ef9cb49`: **Windows green end to end, its first complete pass on CI** — 125 test binaries, 1,647 tests passed and 0 failed; conformance 330 of 330; the reference in sync; the CLI sweep; the fuzz campaign with no trace escaping its row — and Linux x86-64 and arm64 green again, with clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder. **macOS failed one test.** `device::tests::a_beaten_lease_is_never_revoked` beats a 120 ms lease every 20 ms, and a command found it revoked for a missed heartbeat: the runner had left the test's thread unscheduled for more than 100 ms, so the lease really had gone unbeaten past its heartbeat and the watchdog was right. Reproduced here by starving the thread — the unmodified test failed 5 of 8 starved runs with CI's message word for word, and passed 25 of 25 idle. The test now judges each revocation against the gap its thread actually left and restarts the drive when the gap explains it; a watchdog made to fire 200 ms early still fails it, and passes every other device test. Run 5 passed with it on macOS, Linux x86-64 and arm64. Record: `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **The fifth run** | Run `34849980129`, on `010c36c` (2026-09-14): **the first run with no failures — every job green, all three operating systems in one run.** macOS, Linux x86-64 and Linux arm64 each passed 125 test binaries, 1,657 tests, 0 failed; Windows 125 binaries, 1,647 tests, 0 failed (it compiles ten platform-gated tests fewer). On the three x86 test runners every later step passed too — `fmt --check`, the Python-less build, conformance 330 of 330, the reference in sync, the CLI sweep 27 of 27, and the fuzz campaign with no trace escaping its row. clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder passed; `heavy-gates` and `miri-slow` were skipped, by design. The dead-man test run 4 failed passed on every platform, and `docs/REMAINING_WORK.md` 7.2 is closed. |
| **The sixth run** | Run `35147900141` (2026-09-17), started by hand on `51aab51` with `everything`, because that push's commit message quoted GitHub's skip token and so started no run. **The first run on the public runners:** every push job green — 1,657 tests on macOS, Linux x64 and arm64, 1,647 on Windows, 0 failed; the sweep, the fuzz campaign, conformance and the reference as before — and faster (Windows 10.4 min, against 15–28). **`heavy-gates` passed its first run ever.** The ping-pong speedup criterion, asserted on CI for the first time, passed. `miri-slow` then ran out its 240-minute budget on all three crates, with no undefined behaviour in what it reached (broker 124 of 150 tests, syntax 104 of 131, check 30 of 237), so the run as a whole ended *cancelled*; `docs/REMAINING_WORK.md` 5.6 names the tests in flight. |

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

**Who it is for:** developers, and equally **AI agents, LLMs, robots and physical AI**. That is why
every command has a `--json` envelope, why the language server is a first-class surface, and why
`delulu.authority` is answerable over the wire without shelling out.

---

## 3. What has been built — the ledger

**Stages 1–10 are BUILT.** v1.0.0 was tagged locally 2026-07-20 (`198bf44`). Since then the project
has been in continuous adversarial review rather than feature work.

| Stage | What it added |
| --- | --- |
| 1 | Lexer, AST, error-recovering parser, the core type + effect checker, the tree-walking interpreter |
| 2 | Packages, `delulu.toml`, path dependencies, the lockfile, the **semver-authority law** (authority may never widen silently across versions — `DL1003`) |
| 3 | The WASM backend (a *fragment*, not the whole language) under an embedded deny-by-default Wasmtime host |
| 4 | Foreign function interface, embedded Python, cross-engine parity |
| 5 | **Custody**: the broker, the `⊑` attenuation lattice, the grant tree, revocation epochs, the hash-chained audit log, lease tokens — and **the Guard** |
| 6 | **Plugins** (`.dpx`), two classes, signing, verification |
| 7 | **Actors** — message passing, per-sender-pair FIFO, bounded mailboxes, quiescence |
| 8 | **The language server** (`delulu lsp`, LSP 3.17) and the editor surface |
| 9 | The registry, publishing, deployment planning, the measurement program |
| 10 | "Industrial": fleets, devices, federation, hardware adapters, autonomy |

**Post-1.0 campaigns** (each has its own document — see §5):

- **P16 — hardening.** Found the worst defect in the project's history: *the effect row was escapable*
  — `delulu authority` reported "provably pure" for a program that printed at run time.
- **P17 — proof.** Every subsystem placed in exactly one of seven evidence categories. Lean 4 entered
  the picture (`C88` mechanized, no axioms); the broker was model-checked in TLA+; the nine-dimension
  authority order was proved in Z3.
- **P18 — eliminating uncertainty.** `⊑` was shown to be a *preorder*, not a partial order, because
  `path::resolve` is not injective — mathematics, not a bug. Fixed by canonicalizing at the custody
  boundary. Miri completed for the first time.
- **P19 — ecosystem** (2026-08-07). See §8 for what it found.
- **P20 — zero-trust red team & evidence honesty** (2026-08-08). Fixed five documents that *denied* a
  machine-checked proof that exists (Lean re-runs in 35 s); added the **evidence gate**
  (`crates/delulu/tests/evidence_claims.rs`) so prose cannot outlive fact. Red team: filesystem
  containment escapes through a **hardlink** (P20-R1 — documented boundary, not workspace-deliverable,
  pinned by a test); a deeply nested **type** was a checker DoS and, deeper, a parser crash
  (P20-R3 → **`DL0211`** caps type nesting at 128); an adversarial multi-agent authority test
  (Sonnet 5 + Haiku 4.5) confirmed Authority, effect rows and secret-flow hold under attack.
- **Tier 1 — bounded iteration** (2026-08-08, **owner-directed**). Activated four reserved keywords:
  **`for x in xs { … }`**, **`break`**, **`continue`** — the last two also make `while` breakable.
  New diagnostics `DL0411` (non-list iterable) and `DL0412` (break/continue outside a loop). Built
  through every layer with the same effect-transparency and reference-capability soundness as `while`;
  the WASM backend refuses it (`DL1201` interpreter fallback). The other reserved words stay reserved
  with written reasons (`async`/`await` rejected by Constitution decision 12; `trait`/`impl`/`where`
  undesigned; `ref`/`box`/`trn` soundness-critical; `pure` redundant with `!{}`).
- **P21 — cross-account boundary** (2026-08-08). Tested the "separate OS account" recommendation with
  a real second UID: it **holds** on a POSIX filesystem and is **absent on 9p**, where `chmod` is a
  silent no-op. Both `keygen` and the broker now refuse to write a secret onto such a filesystem.
- **Production-readiness sweep** (2026-08-09, `PRODUCTION_READINESS_2026-08-09.md`). Six defects,
  each witnessed against pre-fix code — including a key rotation that a restart undid, a revoked
  federation certificate that a restart resurrected, and two unbounded parser recursions. All four
  recursive-descent nesting classes are now bounded (`DL0210`/`DL0211`/`DL0212`/`DL0213`).
- **Containment + deployment hardening** (2026-08-10, `PRODUCTION_READINESS_2026-08-10.md`). **Eleven
  defects.** Two high: a **dangling symlink escaped filesystem containment** (`canonicalize` fails
  identically for "absent name" and "broken link", so the link's name was re-appended as a plain
  component and the write followed it out of the grant — workspace-deliverable via git, unlike the
  hardlink boundary), and a **guard seal written the natural, relative way gated nothing while the CLI
  answered `ok`**. Also: a dependency's authority pin was escapable by spelling; strict root mode
  failed *open* on a corrupt policy; a valid program aborted the host during value teardown; and
  strict anchored-root mode — the DISC-1 mitigation — turned out to be **unusable** until three
  defects in its own path were fixed. `delulu doctor` gained a `security posture` section, and
  [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) now states what a deployment actually protects.

  **The through-line worth carrying forward:** four of them were the same defect — a security decision
  made on an **unnormalized or unresolved representation**, walked past by a different spelling of the
  same thing. It became the search key, and it is what found the Guard one *after* the campaign's own
  final phase had closed. Ask it of any string compared to decide a security outcome: **what else
  spells the same thing?**

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

## 5. Which document to read, and what each is for

### Start here

| File | Read it when |
| --- | --- |
| **`README.md`** | You want the project in one page, honestly — including what it does *not* have. |
| **`docs/GETTING_STARTED.md`** | You want to write DeluluLang. Install → first program → real programs → your editor. |
| **`docs/for-agents.md`** | **You are an AI agent driving the toolchain.** Exit codes, `--json` envelopes, how to batch `check`, and what to ask the LSP instead of shelling out. |
| **`docs/QUESTIONS.md`** | Someone asks "can this be broken?" It answers the hard questions with evidence, and enumerates the known leaks rather than implying there are none. |
| **`docs/DEPLOYMENT.md`** | **You are about to run code you did not write.** What a deployment actually protects, the three tiers (single-user legacy / strict anchored roots with an offline anchor / separate OS account), the exact commands, how to verify each with `delulu doctor`, per-platform status, an explicit list of what is NOT protected, and why strict mode is not yet the default. |
| **`docs/REMAINING_WORK.md`** | **You are deciding what to build next, or wondering whether a feature exists.** Every gap between what a document in this repository describes and what the code does, each row re-verified against the current binary. Also carries §1: the places where the *documents* were stale and the code had moved ahead. |
| **`docs/NEXT_EVOLUTION_2026/README.md`** | **You are deciding what the project becomes next.** The 2026-09-17 plan: the reassessment against the AI-first goal, twenty-two verified findings (`NE-nn`), the field research, phases P1–P8 and the sandbox phases PS-0…PS-D, the decision records — **proposed, awaiting the owner; nothing implemented.** |
| **`docs/survey/SURVEY.md`** | You are about to change the compiler and want the blast radius. |

### Reference

| File | For |
| --- | --- |
| `docs/REPOSITORY_STRUCTURE.md` | What every directory and significant file is, annotated with *why* it is shaped that way |
| `docs/MATHEMATICS.md` | The formal claims and, for each, which of the seven evidence categories it sits in. **A claim with no category is a claim to be deleted or demoted.** |
| `docs/editors.md` | The language server, per-editor setup, and the editor surface's security history |
| `docs/reference/` | `cli.md`, `diagnostics.md`, `grammar.md`, `tokens.md`, `primitives.md`, the 16 `semantics-5-*.md` chapters, `audit-rules.md`, `coverage.md` |

**If you came to work on the compiler specifically**, read these four in this order:

| File | For |
| --- | --- |
| `docs/design/STAGE1_SPECIFICATION.md` **§9** | *Compiler architecture* — why Rust, and the crate-by-crate pipeline |
| `docs/design/STAGE1_SPECIFICATION.md` **§3** (+ each later stage's additions) | The **normative grammar**. `docs/reference/grammar.md` indexes the productions and names where each is *defined* |
| `docs/release/CHECKPOINT-1.0.md` **§3** | "The compiler" in one page: the soundness core, the code registry, and the refuse-rather-than-guess rule |
| `docs/design/SOUNDNESS_AUDIT.md` | Where the soundness argument **is and is not** complete — findings F-1…F-6 are rejection tests in `crates/delulu-check/tests/laundering.rs` |

Then ask the Survey for the blast radius before you touch anything:
`cargo run -p delulu-survey -- impact mod:crates/delulu-check/src/check.rs`.
| `docs/book/THE_DELULULANG_BOOK.md` | The tutorial. Its samples are conformance-tested — a sample that stops compiling fails the build. |
| `docs/lang/` | Localized human prose (`en-US`, `hi-IN`, `ja-JP`, `de-DE`, `fr-FR`, `es-ES`, `ar-SA`, plus `delulu-slang`). **Codes and JSON never localize.** |

### Design and specification (`docs/design/`)

| File | For |
| --- | --- |
| `CONSTITUTION.md` | The project's own rules. §8.4 forbids editor-specific server features, and that is load-bearing. |
| `LANGUAGE_SPECIFICATION.md`, and `STAGE1_SPECIFICATION.md` through `STAGE10_SPECIFICATION.md` | What each stage is contractually required to do |
| `STAGE*_BUILD_ORDER.md` | The rulings (`D<n>`) that authorized each change. **A bare `D<n>` means the latest stage's numbering.** |
| `AUTHORITY_GUARD_CAPSTONE.md` | Authority and the Guard, together, as one argument |
| `STAGE5_GUARD_ADDENDUM.md` | The Guard's design, including what was adopted from and rejected of `dcg` |
| `STAGE6_PLUGINS_GUIDE.md`, `STAGE7_ACTORS_GUIDE.md` | Plugins and actors, for users |
| `SOUNDNESS_AUDIT.md` | Where the type system's soundness argument is and is not complete |
| `CROSS_PLATFORM_VERIFICATION.md` | **Which platforms have actually been executed**, and which are merely prepared |
| `HARDENING_CAMPAIGN.md` | P16 findings (`C<n>`) — the "what is known to be wrong" list |
| `PROOF_CAMPAIGN.md` | P17/P18 — evidence categories, proof boundaries, and the Miri story |
| `P19_ECOSYSTEM_REVIEW.md` | **The most recent review.** Seven personas, every claim tied to something executed |
| `STABILITY.md`, `VERSION_COEVOLUTION.md` | What may change and when |

### Release and governance

`docs/release/CHECKPOINT-1.0.md` is the **single best status page**: architecture, testing, and
twenty known limitations named without softening. `SUPPORT_MATRIX.md`, `SBOM-1.0.json`,
`PROVENANCE-1.0.json`, `CHECKLIST-1.0.md` sit beside it. Governance lives in `GOVERNANCE.md`,
`CONTRIBUTING.md`, `SECURITY.md`, `TRADEMARK.md`, `INSTALL.md`, `CHANGELOG.md` at the root.

---

## 6. The features, explained

### Delulu Authority

The whole system, and the thing to be most careful with.

- **Effects** — a fixed core set of **ten**: `Read`, `Write`, `Net`, `Async`, `Clock`, `Rand`,
  `Load`, `Actuate`, `ForeignCall`, `Declassify` — carried in a function's row and inferred
  transitively. A module may declare its own with `effect Name`; those are `Effect::User(String)`,
  which is the *variant that carries a user effect*, **not** an eleventh core effect. This line said
  "eleven" and counted `User` as one until 2026-08-23; `CORE_EFFECT_NAMES` and
  `Effect::core_from_name` both hold exactly ten.
- **Capability scopes** — an effect is not enough. A `Cap[FsRead]` is scoped to *paths*; `Cap[Net]` to
  *hosts*; devices to named dimensions with numeric envelopes. `Scopes` carries seven set-valued
  dimensions (`fs_read`, `fs_write`, `net`, `secrets`, `declassify`, `foreign_c`, `foreign_python`)
  plus `device`, which is a **map** from device path to envelope rather than a set — one device has
  exactly one envelope per grant, because two would be an ambiguity the enforcement path must resolve,
  and resolving it silently is how a widening gets in.
- **`child ⊑ parent` is one conjunction over nine dimensions** — the effect set, those seven scopes,
  and `device`. A conjunction, so a widening in *any single* dimension fails the whole check. Proved
  in Z3.
- **`⊑` (attenuation)** — "this authority is contained in that one". A child grant may only ever be
  narrower than its parent. **It is a preorder on the representation and a partial order on the
  quotient**, which is why scopes are stored canonically (P18, findings F1–F3). Canonical form is an
  **antichain**: `{./data, ./data/sub}` collapses to `{./data}`.
- **The broker** (`delulu-broker`) holds the grant tree, revocation epochs, lease tokens, and a
  hash-chained audit log with an external anchor. It is transport-free by design; the daemon and wire
  live in the `delulu` crate.
- **Federation** — brokers can delegate across machines with signed certificates. Two real
  vulnerabilities were found and closed here: certificate replay could undo a revocation, and a child
  with `ttl:None` could outlive its parent's expired uplink lease.

`delulu authority` computes it. `delulu why <Effect>` explains it at function granularity.
`delulu authority --diff` compares two lockfile states and reports widening.

### Delulu Guard

The **policy layer above authority**: how a principal supervises agents that hold authority.

- **Three tiers per rule** — `warn`, `guarded` (a person can approve it at run time), `sealed` (not
  runtime-approvable at all).
- **Permits, not bearer codes.** Approval mints a broker-held permit scoped to a grant, with a TTL and
  a use count. It is *not* a code that whoever runs a CLI verb can claim.
- **Owner codes** — admin verbs require one, printed once and never written down.
- **E-stop** — the emergency stop, and the dead-man timer for devices.

`delulu guard status | policy | request | pending | approve | deny | permits`.

The Guard's idea came from [`dcg`](https://github.com/Dicklesworthstone/destructive_command_guard) by
Jeffrey Emanuel — its *policy* ideas, not its implementation. `dcg` guards untyped shell strings and
needs heavy pattern analysis; DeluluLang's effects are typed and enforced structurally, so only the
supervision model transferred. The credit is recorded in `STAGE5_GUARD_ADDENDUM.md` and stays.

### Plugins

`.dpx` archives, in **two declared classes** — declared, never inferred:

- **`Plugin[Verified]`** ships DIR (the typed IR) and is **re-checked at load** by replaying the
  compiler's own `resolve` + `check_module`. Its exports carry re-verified effect rows and it runs on
  the host engine.
- **`Plugin[Contained]`** ships opaque WASM and is **confined at the module boundary** — imports are
  the only way out.

A `.dpx` claiming Verified whose DIR fails any check is refused (`DL1504`) and **never falls back to
Contained**. `delulu plugin build | inspect | verify`.

### Actors

Message passing, per-sender-pair FIFO (falling out of `mpsc` rather than asserted), bounded mailboxes,
deterministic quiescence. `actors.rs` contains **zero `unsafe`** — the topology earns it: actors never
migrate, `Value` is `Rc`-based and deliberately not `Send`, and messages cross only as an owned
`Send`-by-construction representation. **Deadlock, livelock and starvation are not prevented**; race
freedom is not liveness.

### The language server and editor

One server, `delulu lsp`, LSP 3.17 over stdio, **analysis only** — it never runs code, never loads a
plugin, and holds no broker lease. Diagnostics, hover with the effect row, completion, signature help,
definition, references, rename, document and workspace symbols, semantic tokens, inlay hints showing
*inferred* rows, quick fixes from typed repairs, and formatting that calls the same `format_source`
the CLI does (byte-identical, enforced by a test).

The VS Code extension adds an authority atlas webview, snippets carrying effect rows, tasks, a problem
matcher, a status bar, and an icon. **`delulu.serverPath` is machine-scoped on purpose** — see §8.

### Other surfaces

- **Registry** — sparse index, publish API, **server-side authority recomputation** (the server does
  not trust the client's claim).
- **Morphs** — surface keyword skins, human languages or AI-compact profiles. The *program* is
  unchanged; codes and JSON never pass through a morph.
- **Devices and fleets** — simulated by default (`--broker-profile sim`, deterministic under
  `--seed`), with a real hardware adapter behind an operator-supplied subprocess.
- **Measurement** — `measurements/` is a 15-study reproducible evidence program, not a benchmark suite.

---

## 7. How to work here

```
cargo build --release                              # first build fetches everything
cargo test --workspace --no-fail-fast              # 124 binaries (--no-fail-fast MATTERS: without
                                                   #  it cargo stops at the first failing target)
cargo clippy --workspace --all-targets -- -D warnings   # currently ZERO warnings; keep it there
bash scripts/cli-sweep.sh <abs-path-to-delulu>     # 27 cases, exact exit codes
cargo run -p delulu-survey -- build                # ALWAYS, after any change
./target/release/delulu doctor                     # the health command: environment,
                                                   #  SECURITY POSTURE, and the repo map.
                                                   #  Exit 1 means a real problem remains.
```

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

## 8. Problems — what is open, and why

> **The complete inventory is [`docs/REMAINING_WORK.md`](docs/REMAINING_WORK.md)** (2026-08-23) —
> sixty items across the language, backends, containment, proof, tooling and platform, each
> verified against the binary and each with what closing it takes. This section stays because it is
> the *briefing* version: the things you must know before touching anything. That file is the list
> you plan from. It also carries a §1 this section cannot: **eleven** places where the *documents*
> disagreed with the code — ten corrected, one (Constitution §5.15) left for the project lead
> because the file is entrenched.

### Blocked on the owner (not on engineering)

| | |
| --- | --- |
| **rustfmt** | *(new, 2026-08-07)* There is no `rustfmt.toml`; rustfmt's default style disagrees with this hand-written codebase **3,890** times (2,766 even at `max_width=120`). The choices are: reformat the whole tree (now ~111,000 lines) in one unreviewable commit, keep a permanently red CI step, or say the project has not adopted rustfmt. I removed the step and wrote the reason into `ci.yml`, because a permanently red job teaches people that red is normal. **Adopting rustfmt rewrites every file, and this project's comments carry much of its value — it is the owner's call.** Formatting is currently unenforced. |
| **CODE_OF_CONDUCT.md** | Absent. A policy commitment, not a cleanup task. |
| **The 2026 evolution plan** | *(new, 2026-09-17)* `docs/NEXT_EVOLUTION_2026/` — two passes (the Next Evolution reassessment and the sandbox/VM isolation pass) ended at the STOP the owner asked for. Implementation starts only on his approval of `MASTER_PLAN.md` §9 and §12.2 question 12: the phase order, the microVM's re-sequencing, the interpreter-in-guest deviation from Stage 5 §6, the profile names, the resource-limit defaults, the first network client's dependency, the guest kernel's licensing, the release channel and installer posture. |

**Settled, and no longer blocking — the licence.** `LICENSE` (Apache-2.0), `NOTICE`, `TRADEMARK.md`
and `GOVERNANCE.md` shipped at commit `42702e2` under ruling **D27**, closing hardening finding C9;
before that, default copyright meant nobody could legally use DeluluLang at all. This row sat in the
table above as an open owner blocker long after the decision had landed, which is why it is called
out here rather than silently deleted. **Changing** the licence stays owner-reserved. Deciding it is
done.

### Cannot be done on this machine

| | Why |
| --- | --- |
| ~~**macOS has never been executed.**~~ **Closed 2026-09-14: green on CI.** | The project has no Apple hardware of its own. Re-measured per crate 2026-08-10 with `cargo check --target aarch64-apple-darwin`: **`delulu-diag`, `delulu-syntax`, `delulu-measure` and `delulu-survey` type-check**; every remaining member stops inside a **third-party C build script** (`blake3`, `zstd-sys`, `libffi-sys`) for want of an Apple cross-toolchain, *before* the compiler reaches DeluluLang code. So **no DeluluLang source was shown to fail — and most was not shown to compile either.** Both halves are the claim. Exactly one line in the tree branches on macOS (`broker_transport.rs`, `SUN_PATH_MAX` 104 vs 108); the rest is `cfg(unix)`, which Linux exercises. A matrix entry naming `macos-latest` is a plan, not a result. **`cargo test --workspace` on a Mac was expected to close this, and on CI's macOS runner it did:** the first run stopped building `libffi-sys` (now linked to macOS's own libffi), the second passed 1,654 of 1,655, and the third was green end to end (§1.1). What CI cannot close: a developer's Mac. |
| ~~**CI has never executed.**~~ **Closed 2026-09-14: run 5 was green on every job — all three operating systems in one run.** | Until 2026-09-14 the repository was never pushed, so CI never ran. The first push, to the private testing remote (§1.1), activated `ci.yml`, and the first run went red. This row used to say *"every command in it has been run by hand"*; the Miri command, as written, could never run in this tree (§1.1, *The first run*), so that was not true of it. All five runs are transcribed in `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9: the first four found what they found (§1.1 has each), and the fifth had no failures. *"Written"*, *"triggered"* and *"green"* are three different claims, and the third took five runs to earn. |
| **The Dockerfile has never been built.** | `Dockerfile` and `.devcontainer/devcontainer.json` were written 2026-08-07. Docker CLI 29.5.2 is installed; **the daemon was not running**. Dockerfiles fail for boring reasons that are invisible by reading. Treat the first `docker build` as an experiment. |

### Known technical limits, deliberately not softened

- **No principal types.** Inference is order-dependent; swapping two parameters can change what is
  inferred. (Finding F4.)
- **`Secret.verify` declassifies without requiring `Cap[Declassify]`** — visible in the authority
  report, but not impossible. There is **no implicit-flow tracking**; this is not noninterference.
  (Finding IF-1.)
- **The audit anchor is not tamper-proof.** It catches truncation and naive tampering. An attacker who
  rewrites the log *and* the anchor is not caught; that needs an external witness.
- **The clock ratchet gives monotonicity, not accuracy.** A rewind can no longer resurrect expired
  authority, but it still distorts measured intervals.
- **The WASM backend is a fragment.** The interpreter is the language.
- **The optimizer in spec §2.1 is not implemented**, and there is no native backend.
- **The microVM isolation layer is not built** — and it is a *specified defence layer*, not a
  performance tier. Constitution §5.14 names four layers; `crates/delulu/src/microvm.rs` is 58 lines
  and `probe()` returns `Err` on **every** path, including a fully-provisioned Linux+KVM host. The
  read-only rootfs, virtio-fs scope mounts, egress proxy and vsock broker proxy are unwritten, so
  `--isolation microvm` refuses with `DL1408` everywhere. Nothing weaker ever launches under the
  name — the right refusal — but §5.15 guarantee 5 rests today on the **WASM half only**.
- **The standard library is four list methods** — `len`, `get`, `push`, `map`. No `filter`, `fold`,
  `sort`, `contains`; no `Map`/`Dict`/`Set` among the 16 prelude types; 15 prelude builtins.
  `check.rs::is_higher_order_method` still names `List.filter`, which answers `DL0405`. Nobody ruled
  on this; it is how far the prelude got.
- **CLI-string localization has zero registered strings.** `delulu_diag::catalog::CLI_STRINGS` is an
  empty array, so no CLI prose is localizable in any locale, and every `[cli.*]` key the
  localization guide documents would get `DL1704` and fall back. Diagnostics *do* localize (8 codes
  in the shipped `delulu-slang` catalog, out of 154).
- **Type inference is exponential on a small class of programs** — reproduced 2026-08-23 at
  **10.2 s from 29 lines** (`type Pair[L, R]`, depth 22, doubling per level). No fuel bound, no
  `--max-type-size`, no timeout, and `delulu check` is the agent hot loop. A bound is
  language-visible, so it is an RFC.
- **Deadlock, livelock, starvation and mailbox exhaustion are not prevented.**
- **Multi-tenancy is not provided.** Separate OS accounts are required.
- **`pyo3` stays at 0.25** with two CVEs, ignored on *reachability* — and that argument is now a test
  (`governance.rs::the_ignored_pyo3_advisories_are_still_unreachable`). The upgrade to 0.29 removes
  `Python::with_gil` and is deferred **deliberately**, because GIL handling is exactly where a hasty
  migration introduces undefined behaviour.
- **Miri cannot reach the FFI.** It cannot execute `dlopen` or Windows API calls, so 37 `unsafe` sites
  in `delulu`'s Windows transport stay uninterpreted. This is an explicit assumption, not a to-do.
- **Certification is NONE.** No safety standard, no external audit, no third-party review.
- **Nothing is distributed.** No registry entry, no release binary, and no final public repository —
  the public testing repository of §1.1 is not a release.
- **No physical device has ever been commanded.** Every demonstration drives the simulator.
- **The RFC 0001 governance debt** — a core authority change shipped without its comment period. Open,
  recorded, and never to be restated as compliance.

### What P19 found on 2026-08-07 (read `P19_ECOSYSTEM_REVIEW.md`)

Three defects a green test suite could not see:

1. **The VS Code extension had no working language server**, since P18, in every workspace. It
   registered `delulu.authority`, which the language client *also* registers on the server's behalf;
   `registerCommand` threw inside `client.start()` and killed initialization. The test that should
   have caught it **demanded the bug**, by modelling lens commands and protocol commands as one list.
2. **A cloned repository could choose which binary the extension launched.** `delulu.serverPath` had
   VS Code's default `window` scope, writable by a workspace's own `.vscode/settings.json`. Witnessed:
   the unfixed build ran a planted executable **7 seconds** after the folder opened; the fixed build
   never ran it. **Never add "helpfully find the binary in the workspace" — that convenience is the
   attack.**
3. **Miri was pointed at four crates containing zero `unsafe`**, skipping the two holding 50 of the 54
   sites. Choosing where to point a checker is a bigger decision than how to configure it.

Plus: the CI `lints` job could never have passed; two acceptance-criteria gates (`fmt_laws_100k_gate`,
`criterion3_latency_150ms_on_10kloc_release`) had never been run by anything; and `CHECKPOINT-1.0.md`
still claimed four reachable advisories after the wasmtime 27→47 upgrade had closed them.

### Operational traps that cost real time here

- **Editing the tree while the suite runs** makes the Survey stale and fails three `doctor_cli` tests.
  It looks like a product defect and is not. Regenerate, *then* measure.
- **`cmd | head` gives you `head`'s exit code.** This has turned a refusal into an apparent success
  more than once. Redirect to a file and read `$?`.
- **`pgrep -f "foo"` matches the watching shell's own command line**, so a wait-loop can deadlock
  against itself. Two background jobs died this way in one night.
- **A scan that cannot tell a mention from a use will find its own explanation** — a test searching for
  `pyo3` flagged its own comments.
- **Windows locks a running executable**, so `cargo install` fails with `os error 5` while VS Code is
  running a `delulu.exe` from `target/`. Kill the editor first.
- **PowerShell 5.1 `*>` writes UTF-16LE**; `iconv` before grepping.
- **WSL `nohup setsid` detached jobs do not survive.** Use the harness's background mechanism.

---

## 9. Current numbers

### Measured 2026-08-10, re-verified 2026-08-23 on an untouched tree (current)

The 2026-08-23 run was taken after the documentation pass that produced `REMAINING_WORK.md` and
returned **exactly** the figures below — 124 binaries, 1,645 passed, 0 failed, 4 ignored, cargo's
own exit code 0. That is the point of quoting it: a pass that touched thirteen files and one code
comment moved nothing.

| | Windows | Linux |
| --- | --- | --- |
| test binaries | 124 | 124 |
| tests passing | **1,645** | **1,654** |
| failures | 0 | 0 |
| clippy findings | clean | **0** |

Also verified by execution on this pass: every shipped example checks clean on both platforms · the
**LSP answers a real `initialize` / `didOpen` / `hover` / `shutdown` sequence over stdio** with genuine
`DL` diagnostics (verified by speaking the protocol to the binary, not by trusting the suite — P19's
lesson was that the suite was green while the extension had no server) · VS Code extension **17/17**
and the `.vsix` verifies · `delulu doctor` passes every check · Survey 0 errors / 0 warnings.

**Quote cargo's own exit code, never a pipeline's.** `cargo test … | tail` reports the *pipe's* status,
so a failing suite reads as exit 0 — that is how a red core-invariance gate survived a whole campaign
being described as green (finding CORE-SNAPSHOT-1).

### Measured 2026-08-07, on an untouched tree (kept — the delta below is explained against it)

| | Windows | Linux |
| --- | --- | --- |
| test suites | 122 | 122 |
| tests passing | **1,581** | **1,587** |
| failures | 0 | 0 |

The difference is exactly **6**, and it is a **set** difference rather than a gap. (On the 2026-08-10 run the delta is **9**; the mechanism is the same — platform-gated tests — and the enumeration below is the last one taken test-by-test.) Linux runs 8 tests
Windows does not — 2 Unix-socket transport tests, 5 wasmtime live-engine contained-execution tests,
and 1 verified-plugin-on-wasm test — while Windows runs the 2 refusal counterparts
(`windows_refuses_contained_execution_rather_than_risk_a_fastfail` and
`a_verified_plugin_on_wasm_inherits_the_windows_enforcement_refusal`). 8 − 2 = 6.

Both figures above were taken on the **same tree**, with nothing edited during either run. That
mattered: an earlier attempt compared a Windows number taken *before* three tests were added against
a Linux number taken after, which looked like three tests silently not running on Windows and was
nothing of the kind. **Two measurements are only comparable if they were taken of the same thing.**

Also verified by execution: `cli-sweep.sh` **27/27** · `cargo deny` advisories/bans/licenses/sources
all **ok** · `cargo clippy … -D warnings` **exit 0** · `cargo install` → PATH → the extension works
with **default settings** · Miri `delulu-runtime` FFI decoding 6 tests / 0 UB / 2.4 s and
`delulu-syntax` `fmt::` 16 tests / 0 UB / 1357 s · Trojan Source refused (`DL0107`) · the LSP survives
malformed framing six ways and answers seven pipelined requests · all six `--json` surfaces emit valid
JSON, including on failure.

---

## 10. If you are starting fresh, do this

1. Read `README.md`, then this file's §1 and §8.
2. `cargo build --release` and `cargo test --workspace --no-fail-fast`. Expect 124 test binaries, 0
   failures. **Read cargo's own exit code, not a pipeline's** — `cargo test … | tail` reports the
   *pipe's* status, which is how a red gate once survived a whole campaign described as green.
   If `doctor_cli` fails, run `cargo run -p delulu-survey -- build` and try again.
3. Ask the Survey about anything you are about to change.
4. Read `docs/release/CHECKPOINT-1.0.md` for the honest status,
   `docs/design/P19_ECOSYSTEM_REVIEW.md` for the most recent adversarial pass, and
   `docs/NEXT_EVOLUTION_2026/README.md` for what is proposed next — proposed, not built.
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

There are **26** memory topics as of 2026-08-10. Their content is below, organised by what it is for
rather than one-per-topic, because several topics say the same thing from different angles.

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
- **Disk cleanups have a written discipline** (`docs/maintenance/`): never delete a `.md`, confirm at
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
  plugin.** There is **no signature check** on the adapter itself, and **no driver for any real device
  ships in-tree**. Named as a gap, never blurred.
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
- **CONTAIN-TOCTOU-1 is an accepted, documented residual.** Filesystem containment is a
  check-then-open, so a *concurrent* writer into a granted directory can swap a checked file for a
  symlink in between. The confined program cannot win this race through the primitive table (no
  symlink-creating operation exists), and closing it properly needs `O_NOFOLLOW`/`openat2` — the
  platform-dependent containment this project refuses. **Deployment rule: grant scopes that point at
  directories only the program's own user can write.**
- **The same-uid boundary is category 7 and cannot be closed by code.** To the kernel, a process
  running as your user *is* you. Strict anchored-root mode raises the bar (and, since 2026-08-10, is
  usable, audit-recorded and `doctor`-checkable) but its own residual is a same-user-writable policy
  file. The real boundary is a **separate OS account** — verified with a real second UID, and written
  up with the exact commands in [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md).

**The search key that found four of the 2026-08-10 defects, worth applying to anything new:** a
security decision made on an **unnormalized or unresolved representation**, walked past by a different
*spelling* of the same thing — a dangling link, a `..` left in a comparison, a relative `PATH` entry,
a relative guard pattern. Ask it of every string compared to decide a security outcome: **what else
spells the same thing?**

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

### 11.6 If you are an assistant with memory, keep it current

After verifying work, update `MEMORY.md` and the topic files. One fact per file, with frontmatter, and
a one-line pointer in `MEMORY.md`. Do not record what the repository already says — code structure,
git history, past fixes. Record what was *non-obvious*: the reasoning, the trap, the owner's ruling.

---

## 12. Performance — measured, never promised

The project's stability contract says performance is **measured, never promised**, and the
constitution's performance-honesty rule (§5.11) requires publishing the worse numbers alongside the
better ones. Everything here comes from `measurements/`, where each record names the machine, the
method and the raw figures so you can disagree with the method rather than only the conclusion.

### The finding that matters most for daily use: process startup dominates

On a 35-line file, **26.87 ms of 32.82 ms — 82% — is Windows creating a process**, before a single
byte is compiled. The compiler's own work on that file is under 1.5 ms.

| Layer (min ms) | Windows 11 native | Linux (WSL2) |
| --- | ---: | ---: |
| `empty` — a Rust `fn main() {}` | **26.87** | **3.54** |
| `delulu --version` | 32.04 | 6.19 |
| `delulu check` — 1 line | 32.47 | 6.72 |
| `delulu check` — 35 lines | 32.82 | 7.63 |
| `delulu check` — 1,001 lines | 47.40 | 29.16 |
| `delulu authority` — 35 lines | 33.44 | 7.26 |
| `delulu fmt --check` — 35 lines | 34.91 | 9.10 |

**The practical consequence, and it is large: batch your `check` calls.**

| 20 files, best of repeats | 20 separate invocations | one invocation | speed-up |
| --- | ---: | ---: | ---: |
| Windows | 711.6 ms | **48.0 ms** | **14.8×** |
| Linux | 119.4 ms | **14.2 ms** | **8.4×** |

Nothing was made faster to achieve that — the loop simply stopped paying the process floor once per
file. Every *other* command takes exactly one path and refuses a second rather than silently using
the first. For a long edit→check loop, `delulu lsp` pays the process cost once and every check after
it is the sub-millisecond part.

### Checking scales linearly in program size

A **30,000-line program checks in ~150 ms**. Every shape measured is linear or better in input size.
One published exception: `records_N`, an N-field record with a function reading all N fields, cost
**516 ms at N = 2000** — recorded rather than hidden, and the reason ruling D44 exists.

Dependency resolution: verifying **25 packages across 5 graphs took 98 ms** in total (19 ms mean per
graph).

### Execution speed, against C — and criterion 1 is NOT met

From the pinned Study-C run (release build, minimum of 5 repeats, whole-process wall clock):

| Benchmark | C (gcc -O2) | wasm (opt backend) | interpreter | wasm ÷ C | interp ÷ C |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib_recursive_24` | 6 ms | 12 ms | 77 ms | **2.0×** | 12.8× |
| `loop_sum_1m` | 6 ms | — (`DL1201`) | 363 ms | n/a | 60.5× |
| `string_build_20k` | 6 ms | — (`DL1201`) | 20 ms | n/a | 3.3× |
| `wordcount_macro` | 6 ms | — (`DL1201`) | 12 ms | n/a | 2.0× |
| `list_map_macro` | 6 ms | — (`DL1201`) | 17 ms | n/a | 2.8× |
| `nested_calls_macro` | 5 ms | — (`DL1201`) | 281 ms | n/a | 56.2× |

**Stage 10 criterion 1 asked for a geometric mean ≤ 2.5× C under the optimizing backend, and it is
not met.** `— (DL1201)` means the WASM backend **could not run that program at all**: the 1.x backend
compiles a *subset* of the language and does not lower unbounded loops in `main`, string building, or
the macro workloads. Five of six benchmarks fall outside it. That is published as a real limitation,
not omitted, and the DIR-level optimizer §2.1 describes (cross-package inlining, monomorphization,
escape analysis) is **deferred** under ruling D18.

The honest summary: **the interpreter is the language**, it is between 2× and 60× C depending
entirely on the workload, and the optimizing backend is a narrow fast path rather than a general one.

---

## 13. What has actually been done on each operating system

The distinction this section turns on is the one the whole project turns on: **executed** and
**prepared** are different words.

| | Windows 11 (x86_64-msvc) | Linux (WSL2 Ubuntu) | macOS |
| --- | --- | --- | --- |
| Full test suite | ✅ **124 binaries / 1,645 tests / 0 failed**; on CI's Windows runner **125 / 1,647 / 0** (runs 4 and 5, 2026-09-14) | ✅ **124 binaries / 1,654 tests / 0 failed**; on CI **125 / 1,657 / 0**, x86-64 and arm64 (run 5, 2026-09-14) | ✅ **125 binaries / 1,657 tests / 0 failed** on CI's macOS runner (runs 3 and 5, 2026-09-14) |
| CLI sweep (`cli-sweep.sh`) | ✅ 27/27, and 27/27 on CI (runs 4 and 5) | ✅ run in earlier passes; 27/27 on CI (runs 3 and 5) | ✅ 27/27 on CI (runs 3 and 5) |
| Compiler + interpreter | ✅ | ✅ | ✅ via CI's suite (run 2) |
| WASM engine | ✅ | ✅ (plus 5 live-engine tests Windows refuses by design) | ✅ via CI's suite (run 2) |
| Language server | ✅ | ✅ (via the suite's `lsp_cli.rs`) | ✅ via CI's `lsp_cli.rs` (run 2) |
| VS Code extension, end to end | ✅ **real editor, real server** | ⚠️ **not run** | ❌ |
| Miri | — | ✅ broker/diag/atlas/syntax + the FFI decoder | — |
| `cargo deny`, clippy `-D warnings` | ✅ | — | — |

### Windows — the primary development platform

Everything above was developed and verified here. The extension was installed into a real VS Code, in
an isolated profile, and exercised: activation, server startup, diagnostics, formatting, the atlas
webview, snippets, tasks, and the two security witnesses (a planted binary that ran against the
unfixed build and did not against the fixed one).

Windows-specific behaviour that is real and tested: the `broker_transport` named-pipe path with SID
checks, and `limits::tests::windows_refuses_contained_execution_rather_than_risk_a_fastfail` — Windows
**refuses** contained plugin execution rather than risk a fast-fail, and a test pins that refusal.

### Linux — verified for everything except the editor client

The full suite, the sweep, and all Miri work run here. Linux additionally runs 8 tests Windows does
not: 2 Unix-socket transport tests, 5 wasmtime live-engine contained-execution tests, and 1
verified-plugin-on-wasm test.

**The one gap: `editors/vscode/e2e.js` has never been run on Linux.** The extension is
platform-independent JavaScript and `server-resolve.js` is unit-tested for POSIX lookup, but *"the
unit tests cover the POSIX branch"* and *"the extension works on Linux"* are different claims and this
document does not blur them. Running it needs a display; the command is
`node e2e.js <path-to-delulu>`.

### macOS — green on CI, and only on CI

**First executed 2026-09-14, on CI's macOS runner** (the project has no Apple hardware of its own). The
first run stopped building a dependency; the second built the whole workspace and passed 1,654 of
1,655 tests; the third was green end to end (`docs/design/CROSS_PLATFORM_VERIFICATION.md` §9). In the
fourth, one dead-man test failed there because the runner left its thread unscheduled for over 100 ms
— a test measuring the runner rather than the code; it now judges each revocation against the gap
its thread left (§1.1); run 5 passed with it.
Before that, the standing rested on:

1. The runtime's Unix half is `#[cfg(unix)]`, and *that same code* passes the full suite on Linux —
   which exercises the socket transport, the `0700` directory guard, and the interpreter.
2. Static reading of the macOS-relevant `cfg` branches.
3. **Actual compilation evidence since 2026-08-04**, which is more than reading: 7 crates
   **compile clean** for `x86_64-apple-darwin`, and 3 type-check for `aarch64-apple-darwin`. The full
   workspace cannot be checked from this host because `libffi-sys` picks its MSVC path from the
   Windows host.

Runs 2 and 3 proved more than that reading could: the default build, CPython embedding included,
links and passes on macOS, and in run 3 so did the CLI sweep and the fuzz campaign. What is still
**not** proven: anything outside CI's runner — a developer's Mac, and the VS Code extension on one.

macOS is *green on CI, and only on CI*. The testing remote (§1.1) triggers the `macos-latest`
job on every push to `master`, so every push re-checks it; what no push can check is a Mac outside that
runner.

### The three surfaces, per platform

- **CLI** — Windows and Linux both verified by `cli-sweep.sh`, which asserts an exact exit code for
  each of 27 cases; on CI, 27/27 on macOS and Linux (run 3) and on Windows (run 4).
- **Compiler** — Windows and Linux both run the full suite including the conformance corpus and the
  core-invariance snapshot (the exact bytes the toolchain answers with, for all 108 shipped programs,
  so tooling work cannot quietly move the language). macOS: the same suite runs green on CI (run 3).
- **VS Code extension** — verified end to end on **Windows only**. The `.vsix` is platform-independent
  and its unit tests cover POSIX path resolution, but no editor has been launched against a server on
  Linux or macOS.

### 13.1 Which DeluluLang features reach the editor, and which do not

Verified against `crates/delulu/src/lsp.rs` and the extension manifest, not from memory.

| Feature | In VS Code | How, or why not |
| --- | --- | --- |
| **Authority** | ✅ four ways | the `authority: {…}` code lens on `main`; **Show authority report** (the §10.5 JSON, answered by the server over `workspace/executeCommand`); **Show authority atlas** (the call graph with each function's effect row on it); and hover, which carries the transitively computed authority |
| **Effects** | ✅ | in hover, in **inlay hints showing the *inferred* row** on unannotated lambdas, and as a dedicated semantic-token kind |
| **Capabilities** | ✅ | semantic tokens have their own kinds for capability types, reference capabilities and secrets |
| **Diagnostics + typed repairs** | ✅ | the compiler's own codes, spans and repairs. An authority-widening repair is ⚠-titled and **never** marked preferred, and carries `data.authority_widening` so an agent can refuse it by policy |
| **Formatting** | ✅ | the same `format_source` the CLI runs, byte-identical by test |
| **Tests** | ✅ | `▶ run test` runs *that* test by name |
| **The Guard** | ❌ **nothing at all** | CLI only: `delulu guard status / policy / request / pending / approve / deny / permits` |
| **Broker, grants, leases, audit chain** | ❌ | CLI only: `delulu grants`, `delulu audit`, `delulu broker` |
| **Plugins, devices, fleets, registry** | ❌ | CLI only |

**The Guard's absence is structural, not an oversight.** `lsp.rs` states it in its own header: the
module *never constructs an `Interp`, a broker*, or a lease. The server is analysis-only, which is
what lets it read a hostile file safely — a compromised workspace cannot use it as an effector, and
its availability is explicitly not a security property (spec §11). The Guard supervises things at
**run** time; it has nothing to say about a file you are editing.

**What is genuinely missing rather than deliberately absent:** a **read-only** Guard/broker status
view in the editor — current policy tiers, pending approval requests, live permits. That would not
require the server to hold any authority (it could shell out to `delulu guard status --json`), it
would be useful to anyone supervising an agent, and **it does not exist**. Named here so the gap is a
decision rather than an assumption.

---

## 14. CLI vs compiler vs editor — what each user actually gets

**There is a real compiler. There is no separate compiler *binary*.** Those are different statements
and conflating them misleads in both directions.

**The compiler is a specified component** (`STAGE1_SPECIFICATION.md` §9 "Compiler architecture",
`docs/release/CHECKPOINT-1.0.md` §3 "The compiler"):

| Stage of the pipeline | Crate | What it does |
| --- | --- | --- |
| lex → parse | `delulu-syntax` | tokens, AST, a hand-written recursive-descent parser **with error recovery** — it resyncs and keeps finding faults rather than stopping at the first |
| resolve → typecheck → effect/authority check | `delulu-check` | names, types, effect rows, the authority lattice, `Secret[T]` opacity, reference capabilities and sendability. The docs call it **"the soundness core"** |
| diagnostics | `delulu-diag` | spans, the code registry, the JSON envelope, typed repairs, the human renderer |
| execute | `delulu-runtime` | the tree-walking interpreter — **the interpreter is the language** |
| compile to WASM | `delulu-wasm` | a *subset* backend under a deny-by-default Wasmtime host |

**154 registered diagnostic codes** (the Survey's measured count today; `CHECKPOINT-1.0.md` says 145
and is correct *as a 1.0 snapshot* — release documents are deliberately exempt from the
freshness scan). Codes are **add-only** from 1.0, each with an accepting *and* a rejecting conformance
witness; coverage is 100% and hard-gated per commit. The grammar is normative
(`STAGE1_SPECIFICATION.md` §3 plus each stage's additions, indexed by `docs/reference/grammar.md`).
Where the checker cannot decide, **it refuses rather than guesses**.

### When did the compiler last change?

Two different questions, and the second is the one that matters.

- **Last touched:** 2026-08-07 (today). Three commits edited compiler crates — but only to fix lints
  (a `zip` replacing a hand-rolled index in `deps.rs`, a `const { assert! }` in `codes.rs`, an
  `#[allow]` in `ast.rs`, a `while let` in `parser.rs`) and to shrink two **test-only** `cfg!(miri)`
  budgets in `fmt.rs`.
- **Last change to what the compiler DECIDES:** 2026-08-04, commit `47393b9` — the `DL0210`
  deep-nesting guard, so a valid but pathologically nested module is *refused* instead of overflowing
  the stack. Before that, the P16/P17 soundness fixes of 2026-08-03 (the escapable effect row, and
  `Secret.verify` being typed pure).

**And that is checked, not asserted.** `tests/core-invariance/SNAPSHOT.txt` records the exact bytes
the toolchain answers with for all **109** programs the repository ships, across 362 invocations. It
was last modified on **2026-08-04**, and `the_core_still_answers_exactly_as_recorded` passes against
today's binary — so today's edits provably moved nothing.

> This is the owner's **core-regression rule**: *tooling is for future developers, the core is the
> product; a green suite is not proof.* After any tooling-only change, that snapshot is the evidence
> that the language did not move. Regenerate it deliberately, never incidentally:
> `DELULU_BLESS=1 cargo test -p delulu --test core_invariance`.

What does *not* exist is a separate driver you invoke yourself. There is one binary, `delulu`, and
the compiler is reached through `delulu check` (analyse), `delulu build` (resolve deps, verify pins,
optionally emit `.dwx`) and `delulu run` (check, then execute). There is no `deluluc`, and nothing to
install alongside. **That is why the editor's answers cannot drift from the CLI's** — the language
server calls the same `delulu-check`, so a diagnostic in your editor is the diagnostic
`delulu check --json` prints, not a re-implementation of it.

There are exactly **three doors**:

| Door | What it is | Who uses it |
| --- | --- | --- |
| `delulu <subcommand>` | 32 subcommands. **The whole product.** | people and scripts |
| `delulu lsp` | LSP 3.17 over stdio. **Analysis only.** | every editor, and agent harnesses |
| the VS Code extension | a thin client over `delulu lsp`, plus terminal commands that shell out to the CLI | VS Code users |

### The dividing line, and it is not arbitrary

**The editor gets everything that reads. The CLI gets everything that acts.**

`delulu lsp` never constructs an interpreter, never loads a plugin, and holds no broker connection or
lease — stated structurally in `lsp.rs`'s own header, not merely intended. That is what lets it open a
hostile file safely: a compromised workspace cannot use the language server as an effector, and its
availability is explicitly *not* a security property (spec §11).

So anything that **holds authority or executes code** is CLI-only, by construction:

| Available in the editor | CLI only |
| --- | --- |
| diagnostics, typed repairs, quick fixes | `delulu run` / `test` — *reachable from the editor, but only by shelling out to the CLI in a terminal, and gated on workspace trust* |
| hover, completion, signature help | `delulu guard` — policy tiers, permits, approvals, e-stop |
| definition, references, rename | `delulu broker` / `grants` / `audit` — the grant tree, leases, revocation, the audit chain |
| document + workspace symbols | `delulu plugin` — build, inspect, verify `.dpx` |
| semantic tokens, inlay hints (inferred rows) | `delulu fleet`, device profiles, hardware adapters |
| **authority**: the lens, the report, the atlas | `delulu publish` / `deploy` / `add` / `login` — the registry |
| formatting | `delulu secrets`, `keygen`, `sign`, `verify-sig` |
| | `delulu repl`, `morph`, `locale`, `doctor`, `explain`, `completions` |

### What that means in practice, per audience

**A developer in VS Code** sees the authority system continuously and passively: the effect row in
hover, the inferred row as an inlay hint, `authority: {Write}` as a lens on `main`, and the atlas one
click away. They never need to run a command to know what their program can do. But to *supervise*
anything — approve a guarded operation, inspect a grant tree, revoke a lease — they go to a terminal.

**Someone using only the CLI** loses nothing. Every capability exists there; the editor is a
convenience layer over a subset. `delulu authority --json` and `delulu atlas --format json` give the
same answers the editor renders.

**An AI agent or harness** should use `delulu lsp` and speak the protocol rather than shelling out per
question: it pays the process cost once, and on Windows that cost is 82% of a small `check`. Over the
wire it gets diagnostics identical to `delulu check --json`, typed repairs carrying
`data.authority_widening` so it can refuse them **by policy rather than by parsing prose**, and the
authority report via `workspace/executeCommand` → `delulu.authority`. When it needs to *act* — run,
grant, revoke — it drops to the CLI, which is exactly where the Guard can supervise it.

**A robot or physical-AI deployment** uses the CLI only. Devices, envelopes, dead-man timers, sign-off
records and e-stop have no editor surface at all and are not meant to.

### The one gap worth naming

A **read-only** Guard/broker status view in the editor — current policy tiers, pending approval
requests, live permits — would not require the server to hold any authority (it could shell out to
`delulu guard status --json`), would be genuinely useful to anyone supervising an agent, and **does
not exist**. Recorded here so its absence is a decision rather than an assumption.
