# DeluluLang — Repository Structure & Move Manifest

**Status:** Living. This document is the map of the repository. It matches the workspace layout
the stage specs assume (Stage-1 spec §9.2). The moves in §3 were executed at repo initialization
(2026-07-05).

**Re-synchronized 2026-07-24** against the actual tree (`HARDENING_CAMPAIGN.md` C5). Between
Stage 1 and Stage 10 this file had drifted in both directions: it drew directories that were never
built and omitted most of what the later stages added. §1 below is now what is *there*, not what
was once planned. Anything aspirational is marked as such inline — a map that mixes the two
silently is worse than no map.

---

## 1. Target directory tree

```
DeluluLang/
├── Cargo.toml                      # Rust workspace root (members grow per stage)
├── Cargo.lock                      # committed for reproducible builds (tracked 2026-07-21, D19c)
├── README.md                       # project front door
├── LICENSE                         # Apache-2.0 (the code), © Jesse Sunil (D27)
├── NOTICE                          # attribution carried by every redistribution (Apache §4d)
├── TRADEMARK.md                    # the DeluluLang NAME policy — derivatives rename (D27)
├── GOVERNANCE.md                   # project governance; Jesse Sunil = lead
├── .gitignore
│
├── crates/                         # the compiler & runtime, one crate per pipeline concern
│   ├── delulu-diag/                # spans, source map, code registry, JSON envelope, repairs, renderer
│   │   ├── Cargo.toml              #   + [Stage 8, early] palette.rs — the role-based color system
│   │   └── src/{lib,span,source,codes,diagnostic,json,render,palette}.rs
│   ├── delulu-syntax/              # tokens, lexer (Go-style termination), AST, error-recovering parser
│   │   ├── Cargo.toml
│   │   └── src/{lib,token,lexer,ast,parser,fmt,grammar,morph}.rs
│   │                               #   morph.rs [D35]: the canonical-form law for surface morphs
│   ├── delulu-check/               # [Stage 1] resolve, types, rows, THE effect/authority checker
│   │   └── src/{lib,resolve,types,row,unify,check,authority,secret}.rs
│   ├── delulu-runtime/             # [Stage 1] values, capability table, interpreter, grant broker
│   │   └── src/{lib,value,cap,broker,interp,prim,manifest}.rs
│   ├── delulu-broker/              # [Stage 5] custody core: ⊑ lattice, grant tree, validation
│   │   │                           #   classes, audit chain, lease tokens, secrets, THE GUARD
│   │   └── src/{lib,authority,path,tree,ids,time,validate,diag,audit,lease,secrets,guard}.rs
│   ├── delulu-atlas/               # [Stage 8, early] the Atlas: typed deterministic code+authority
│   │   │                           #   graph from compiler facts (atlas/1, digest, query verbs)
│   │   └── src/{lib,model,build,render,query,formats}.rs
│   ├── delulu-wasm/                # [Stage 3] the WASM backend: .dwx emission, engine parity
│   ├── delulu-registry/            # [Stage 2/9] index lines, resolution, server-side authority
│   ├── delulu-conform/             # [Stage 9] the conformance runner: --coverage, --check-reference
│   ├── delulu-measure/             # [Stage 9] the measurement harness behind measurements/
│   ├── delulu-fuzz/                # [Stage 9] fuzz targets for the front end
│   ├── delulu-survey/              # repository tooling (`publish = false`, no sibling deps):
│   │                               #   derives docs/survey/ — the map of THIS REPOSITORY, with a
│   │                               #   file:line citation on every edge. Not language surface.
│   └── delulu/                     # [Stage 1] the `delulu` CLI: check | run | repl | authority
│       ├── src/{main,cli,repl}.rs  # + [Stage 5] broker_ipc, brokerd, broker_client,
│       │                           #   broker_transport, foreign_worker, microvm (Linux)
│       └── tests/                  # binary-level integration: broker_cli, grants_cli, audit_cli,
│                                   #   foreign_worker, microvm_criterion8, guard_cli, guard_e2e,
│                                   #   palette_cli, atlas_cli, atlas_e2e, …
│
│                                   # NOTE: there is NO `stdlib/` directory and NO standard
│                                   # library written in DeluluLang. Earlier revisions of this
│                                   # file drew `stdlib/std/{core,fs,net,io}.delulu`; it was
│                                   # never built. The available surface is the PRIMITIVE TABLE
│                                   # (`docs/reference/primitives.md`) plus the prelude builtins.
│
├── morphs/                         # [D35] surface keyword morphs: zh-CN-keywords, compact-ai
│                                   #   loaded by id from here, $DELULU_MORPH_PATH, or ~/.delulu/morphs
│
├── examples/                       # runnable .delulu programs (demo.delulu is the reference)
│   └── guide/                      # the samples docs/GETTING_STARTED.md is built from; a gate
│                                   #   checks each one AND runs it (crates/delulu/tests/examples_run.rs)
│
├── tests/                          # cross-crate test corpora, driven by the `delulu` binary
│   │                               # (the SOUNDNESS_AUDIT F-1…F-6 / R-7 rejection tests are NOT
│   │                               #  here — they live in crates/delulu-check/tests/laundering.rs)
│   ├── conformance/                # Stage-9 coverage law: ≥1 accepting + ≥1 rejecting per rule
│   │   ├── accept/                 # programs that must check clean (+ expected authority JSON)
│   │   └── reject/                 # programs that must fail (+ expected DLxxxx code/span/repair)
│   └── corpus/                     # coding-capability tiers (simple → security-expert)
│       │                           # every file must CHECK clean and every package must BUILD
│       │                           # clean (conformance.rs); three are also RUN (corpus_cli.rs)
│       ├── tier1-simple/           # 2 programs
│       ├── tier2-dsa/              # 4 — recursion, a recursive sum type, row-polymorphic
│       │                           #     higher-order code, and the `iso`/`val` write rule
│       ├── tier3-application/      # 2 — real input, distinguishable failures, Result chains
│       ├── tier4-multimodule/      # 4 PACKAGES / 7 modules, depth 3, one diamond (see its
│       │                           #     README.md) — built AND run (D61)
│       └── tier5-security/         # 3 — confinement, secrets, capability attenuation
│
├── editors/                        # [Stage 8] VS Code extension + generic LSP config
│   └── vscode/
│
├── rfcs/                           # [Stage 10] the RFC process: language/authority changes
├── release-artifacts/              # [Stage 9] built release outputs
├── SECURITY.md                     # reporting policy + rehearsed patch runbook
├── CHANGELOG.md                    # notable changes; every entry names its authorizing ruling
├── CONTRIBUTING.md                 # contribution rules; §4 governs AI-authored RFCs
├── .github/workflows/              # the three-OS CI matrix (NEVER executed — repo is not pushed)
│
├── measurements/                   # [Stage 9] the published proof (studies A/B/C), reproducible
│   └── METHODOLOGY.md              # how every published number was produced
│
└── docs/
    ├── REPOSITORY_STRUCTURE.md     # this file
    ├── GETTING_STARTED.md          # install → first program → real programs (the entry path)
    ├── for-agents.md               # the one page an agent harness should pin
    ├── editors.md                  # editor/LSP setup
    ├── design/                     # the committed design corpus (constitution, audit, stages)
    │   ├── CONSTITUTION.md
    │   ├── SOUNDNESS_AUDIT.md
    │   ├── DELULU_CORE.md              # the formal calculus (paper sketches; honesty-labeled)
    │   ├── STAGE1_SPECIFICATION.md … STAGE10_SPECIFICATION.md
    │   ├── STAGE10_AUTONOMY_ADDENDUM.md # [Stage 10] autonomy domains (spec Rev 2): vehicles/
    │   │                                #   aircraft/satellites/robots; energy, safety chains, MCUs
    │   ├── LOCALIZATION_PLUGIN_GUIDE.md # human-language plugins: author/add/remove/edit (Fable 5)
    │   ├── SYNTAX_MORPH_SPEC.md         # keyword/char syntax skins, human + AI compact profiles
    │   │                                #   BUILT (D35): delulu-syntax/morph.rs + delulu morph
    │   ├── AI_NATIVE_DESIGN.md          # the machine-facing design + standing commitments
    │   ├── LANGUAGE_SPECIFICATION.md   # superseded early draft — kept for provenance
    │   └── DeluluLang_PROMPT.md        # Jesse's original vision — kept verbatim, never edited
    ├── playbooks/                  # execution companions per stage spec (Opus 4.8 builds from these)
    │   ├── README.md                   # how to use a playbook; environment traps; crate/DL map
    │   └── STAGE4_PLAYBOOK.md … STAGE10_PLAYBOOK.md   # the seven unbuilt stages
    ├── lang/                       # per-language packs (catalog content sources)
    │   ├── README.md, en-US.md, delulu-slang.md       # complete (reference base + shipped voice)
    │   └── zh-CN, ja-JP, ko-KR, hi-IN, ar-SA, fr-FR, de-DE, es-ES, pt-BR (.md)  # starters, decisions locked
    ├── reference/                  # [Stage 9] generated-in-part language reference
    ├── release/                    # [Stage 9] announcement, checklist, SBOM, provenance, support matrix
    ├── security/                   # security drill records
    ├── survey/                     # GENERATED by `cargo run -p delulu-survey -- build`:
    │                               #   SURVEY.md (the map), survey.json (schema survey/1),
    │                               #   DISCREPANCIES.md (where the repo disagrees with itself).
    │                               #   A test fails if these are behind the tree — so §1 above is
    │                               #   an orientation, and the Survey is the current truth.
    └── book/                       # the Delulu Book — THE_DELULULANG_BOOK.md (first complete edition)
```

**Planning-pass note (Fable 5, 2026-07):** `docs/playbooks/`, the localization/AI-design trio in
`docs/design/`, `docs/lang/`, and the Book were authored as pure planning artifacts (no code) so the
implementing model can build Stages 4–10 and the localization system mechanically. Playbooks are *how
to build*; the stage specs remain the normative *what*.

## 2. Crate dependency graph

**The current graph is generated: [`docs/survey/SURVEY.md`](survey/SURVEY.md), "How the crates
depend on each other".** It is read from the `path = "../…"` entries in each `Cargo.toml`, every
edge cites the line that declares it, and a test fails if it falls behind the tree.

It is generated because the hand-drawn version below was wrong. This section used to show the
five-crate Stage-1 spine as though it were the shape of the project; by Stage 10 the workspace held
twelve, and `delulu-broker`, `delulu-atlas`, `delulu-wasm` and the rest appeared nowhere in it. The
C5 re-synchronization in 2026-07 fixed §1 and left this section untouched, which is exactly how a
map rots — in the part nobody re-read.

The Stage-1 spine, kept because it is still the useful mental model of the *pipeline* (and is now
labelled as the historical subset it always was):

```
delulu-diag  ←  delulu-syntax  ←  delulu-check  ←  delulu-runtime  ←  delulu (CLI)
```

Rule (carried from the spec): keep files small; the conformance suite is the primary control.
Each crate builds and tests independently, so a coding session can verify bottom-up.

## 3. Move manifest — executed at repo init (2026-07-05)

Every design `.md` that previously sat in the repository root moved into `docs/design/`:

| From (repo root) | To |
|---|---|
| `CONSTITUTION.md` | `docs/design/CONSTITUTION.md` |
| `SOUNDNESS_AUDIT.md` | `docs/design/SOUNDNESS_AUDIT.md` |
| `STAGE1_SPECIFICATION.md` | `docs/design/STAGE1_SPECIFICATION.md` |
| `STAGE2_SPECIFICATION.md` … `STAGE10_SPECIFICATION.md` | `docs/design/STAGE2_…` … `docs/design/STAGE10_…` |
| `LANGUAGE_SPECIFICATION.md` | `docs/design/LANGUAGE_SPECIFICATION.md` |
| `DeluluLang_PROMPT.md` | `docs/design/DeluluLang_PROMPT.md` |

New at root/created: `Cargo.toml`, `README.md`, `.gitignore`, `crates/`, `tests/`, `examples/`,
`docs/design/`.

## 4. Naming and placement conventions

- **Design docs** → `docs/design/` (normative specs) — never in the crate tree.
- **Compiler/runtime code** → `crates/<name>/src/` — one crate per pipeline concern (§9.2).
- **DeluluLang programs** used as tests → `tests/**` (checked by the `delulu` binary, not `cargo`).
- **DeluluLang programs** for humans to read/run → `examples/`; teaching samples referenced by
  `docs/GETTING_STARTED.md` → `examples/guide/`, where a gate both checks and runs them.
- **There is no standard library.** The callable surface is the primitive table plus the prelude
  builtins; see `docs/reference/primitives.md`.
- A code range (`DLxxxx`) is allocated in exactly one stage and never reused; the registry lives
  in `crates/delulu-diag/src/codes.rs` and grows per stage.
- **Ruling ids are allocated once per stage, so cite the stage when you mean an earlier one.**
  `STAGE9_BUILD_ORDER.md` allocates D1–D22 and `STAGE10_BUILD_ORDER.md` allocates D1–D66, so a bare
  number from 1 to 22 exists in both. The convention is that **a bare `D<n>` means the latest stage**
  and an earlier one is written `S9-D<n>` — as `CHANGELOG.md` and the Stage-10 ledger heading both
  state, and as the Stage-10 order does in practice. It is repeated here because 232 citations
  depend on it and neither of those two places is where a reader meets one. Nothing is currently
  mis-attributed; `docs/survey/` resolves every citation by this rule and reports how many rely on
  the default.
