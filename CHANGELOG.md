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

### Security

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
