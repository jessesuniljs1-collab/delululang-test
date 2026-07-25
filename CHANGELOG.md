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
