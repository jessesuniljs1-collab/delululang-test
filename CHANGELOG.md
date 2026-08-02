# Changelog

All notable changes to DeluluLang are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow the **semver-authority
law** (Constitution invariant 10): *any* widening of what a package can do to your system requires a
major bump, even when the API is unchanged.

Every entry names the ruling that authorized it. Rulings live in
`docs/design/STAGE10_BUILD_ORDER.md` (`D<n>`) and, for Stage 9, `STAGE9_BUILD_ORDER.md` (`S9-D<n>`).
Campaign findings (`C<n>`) live in `docs/design/HARDENING_CAMPAIGN.md`.

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
