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
