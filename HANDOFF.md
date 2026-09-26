# HANDOFF — DeluluLang

**For:** whoever picks this repository up next, human or AI.
**What this is:** a current briefing — the standing rules, what DeluluLang is, how to work here, and
the project's memory written down. It was rewritten on **2026-09-27** (V2 phase P6) from a 1,200-line
document that had grown a ledger, a feature tour, dated numbers, a problems list and a per-platform
record alongside the briefing. Those sections are kept **verbatim, with their original numbers**, in
[`docs/archive/v1/HANDOFF_HISTORY.md`](docs/archive/v1/HANDOFF_HISTORY.md); the last section here says
where each one went, so a citation of "`HANDOFF.md` §8" still finds its text. Nothing was deleted.
**Repository:** `D:\nelan\DeluluLang` — a Rust workspace of 13 crates — and on GitHub, **publicly since
2026-09-17**: `origin` → `https://github.com/jessesuniljs1-collab/delululang-test.git` (§1.1).

## Where things stand (2026-09-27)

- **DeluluLang V2 is executing** — the active source of truth is `docs/DELULULANG_V2/` (start at
  `V2_README.md`). Complete, each with its CI run read green: V2-0, P1, PS-0, PS-A, P2, P4a, P3, PS-B,
  P4b–e and **PS-C** — the microVM runs (`--isolation microvm`, Linux + KVM), under Firecracker's jailer
  when run as root — except **PS-C-05**, a distributed guest image, which is the owner's (D-NE-27, a
  built GPL kernel). **Now: P6** (this rewrite, the README and the Book). Then P5 (distribution), P7
  (verification depth), PS-D (external launchers), P8 (owner-gated).
- **Resume from `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, then the newest entry in
  `docs/DELULULANG_V2/V2_LOG.md`.** Decisions taken under the owner's delegation are
  `V2_DECISION_LOG.md` (`D-V2-nn`).
- **What is open**, everywhere: `docs/REMAINING_WORK.md`. **What only the owner can decide:**
  `V2_DECISION_LOG.md`, *Owner decisions carried from V1, still open*, and §1.1's gate for the final
  public repository.

> **Status: PRODUCTION READY WITH DOCUMENTED DEPLOYMENT REQUIREMENTS** — Windows and Linux, in the
> Tier-2 deployment of `docs/DEPLOYMENT.md` (the 2026-08-10 campaign's verdict). macOS is verified by CI
> but not covered by the verdict: the Tier-2 cross-account boundary was tested with a real second UID on
> Linux only.

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
cargo test --workspace --no-fail-fast              # ~145 test binaries (--no-fail-fast MATTERS:
                                                   #  without it cargo stops at the first failing target)
cargo clippy --workspace --all-targets -- -D warnings   # currently ZERO warnings; keep it there
bash scripts/cli-sweep.sh <abs-path-to-delulu>     # every case an exact exit code; it counts its own
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

## 10. If you are starting fresh, do this

1. Read `README.md`, then this file's §1, then `docs/REMAINING_WORK.md` (what is open, and why).
2. `cargo build --release` and `cargo test --workspace --no-fail-fast`. Expect 0 failures (about
   1,930 tests on Windows as of 2026-09-27; the per-date figures are in the V2 log, and before V2 in
   the archived §9). **Read cargo's own exit code, not a pipeline's** — `cargo test … | tail` reports the
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
- **Keep cooking, and do not ask (owner, 2026-09-20 and again 2026-09-25):** finish every phase in
  order, check that the earlier ones work and are finished, be persistent and retry until a thing is
  built, and do not ask questions — *"Even if u ask me questions I will say 'Do whatever good for
  delululang, The lang of the future'"*. Use and update the Survey and `doctor`; keep the `.md` files
  and this file current. Pause about a minute after each phase and major run for a reply, then
  continue. Three things stay the owner's regardless: **entrenched files** (CODEOWNERS), **the final
  public repository**, and **the licence**. Decisions taken under this delegation are recorded as
  `D-V2-nn … TAKEN (head chef, under the owner's delegation)`, never as rulings he made.
- **Agents, currently (owner, 2026-09-20):** Sonnet 5 for small, easily finished jobs only.
- **V2 execution rules (owner, 2026-09-17, evening; `docs/design/DeluluLang_V2_Execution_Master_Prompt.md`):** Opus 5 is the main execution sous-chef and does most of the work — implementation, investigation, tests, refactors, security and adversarial testing, documentation migration, verification, cleanup; one strong agent at a time; the head chef writes the brief (objective, files, constraints, security and verification requirements, expected outputs), supervises, verifies every result against the binary, and commits; an agent asks rather than invents an architectural or security decision. Phases run one at a time in the approved order; after each phase's push and recorded result the work **stops, waits about sixty seconds for the owner, and continues to the next approved phase only if nothing arrives** — one controlled continuation, never a loop. The active source of truth is `docs/DELULULANG_V2/`; historical documents live in `docs/archive/v1/` and are neither maintained nor deleted.

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
