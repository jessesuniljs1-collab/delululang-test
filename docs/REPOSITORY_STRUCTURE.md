# DeluluLang — Repository Structure & Move Manifest

**Status:** Living. This document is the map of the repository. It matches the workspace layout
the stage specs assume (Stage-1 spec §9.2). The moves in §3 were executed at repo initialization
(2026-07-05); §1–§2 describe the target structure the build grows into.

---

## 1. Target directory tree

```
DeluluLang/
├── Cargo.toml                      # Rust workspace root (members grow per stage)
├── Cargo.lock                      # committed from Stage 2 (reproducible builds)
├── README.md                       # project front door
├── .gitignore
│
├── crates/                         # the compiler & runtime, one crate per pipeline concern
│   ├── delulu-diag/                # spans, source map, code registry, JSON envelope, repairs, renderer
│   │   ├── Cargo.toml              #   + [Stage 8, early] palette.rs — the role-based color system
│   │   └── src/{lib,span,source,codes,diagnostic,json,render,palette}.rs
│   ├── delulu-syntax/              # tokens, lexer (Go-style termination), AST, error-recovering parser
│   │   ├── Cargo.toml
│   │   └── src/{lib,token,lexer,ast,parser}.rs
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
│   └── delulu/                     # [Stage 1] the `delulu` CLI: check | run | repl | authority
│       ├── src/{main,cli,repl}.rs  # + [Stage 5] broker_ipc, brokerd, broker_client,
│       │                           #   broker_transport, foreign_worker, microvm (Linux)
│       └── tests/                  # binary-level integration: broker_cli, grants_cli, audit_cli,
│                                   #   foreign_worker, microvm_criterion8, guard_cli, guard_e2e,
│                                   #   palette_cli, atlas_cli, atlas_e2e, …
│
├── stdlib/                         # [Stage 1+] the DeluluLang standard library, in DeluluLang
│   └── std/{core,fs,net,io}.delulu # (Stage 1 surface is tiny; §11 of the Stage-1 spec)
│
├── examples/                       # runnable .delulu programs (demo.delulu is the reference)
│
├── tests/                          # cross-crate test corpora, driven by the `delulu` binary
│   ├── conformance/                # Stage-9 coverage law: ≥1 accepting + ≥1 rejecting per rule
│   │   ├── accept/                 # programs that must check clean (+ expected authority JSON)
│   │   └── reject/                 # programs that must fail (+ expected DLxxxx code/span/repair)
│   ├── laundering/                 # SOUNDNESS_AUDIT.md F-1…F-6, R-7 — permanent rejection tests
│   └── corpus/                     # coding-capability tiers (simple → security-expert)
│       ├── tier1-simple/
│       ├── tier2-dsa/
│       ├── tier3-application/
│       ├── tier4-multimodule/
│       └── tier5-security/
│
├── editors/                        # [Stage 8] VS Code extension + generic LSP config
│   └── vscode/
│
├── measurements/                   # [Stage 9] the published proof (studies A/B/C), reproducible
│
└── docs/
    ├── REPOSITORY_STRUCTURE.md     # this file
    ├── design/                     # the committed design corpus (constitution, audit, stages)
    │   ├── CONSTITUTION.md
    │   ├── SOUNDNESS_AUDIT.md
    │   ├── DELULU_CORE.md              # the formal calculus (paper sketches; honesty-labeled)
    │   ├── STAGE1_SPECIFICATION.md … STAGE10_SPECIFICATION.md
    │   ├── STAGE10_AUTONOMY_ADDENDUM.md # [Stage 10] autonomy domains (spec Rev 2): vehicles/
    │   │                                #   aircraft/satellites/robots; energy, safety chains, MCUs
    │   ├── LOCALIZATION_PLUGIN_GUIDE.md # human-language plugins: author/add/remove/edit (Fable 5)
    │   ├── SYNTAX_MORPH_SPEC.md         # keyword/char syntax skins, human + AI compact profiles
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
    └── book/                       # the Delulu Book — THE_DELULULANG_BOOK.md (first complete edition)
```

**Planning-pass note (Fable 5, 2026-07):** `docs/playbooks/`, the localization/AI-design trio in
`docs/design/`, `docs/lang/`, and the Book were authored as pure planning artifacts (no code) so the
implementing model can build Stages 4–10 and the localization system mechanically. Playbooks are *how
to build*; the stage specs remain the normative *what*.

## 2. Crate dependency graph (Stage 1)

```
delulu-diag      (no internal deps; serde, serde_json)
     ▲
delulu-syntax    (→ delulu-diag)
     ▲
delulu-check     (→ delulu-syntax, delulu-diag)
     ▲
delulu-runtime   (→ delulu-check, delulu-syntax, delulu-diag)
     ▲
delulu (CLI)     (→ all of the above)
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
- **DeluluLang programs** for humans to read/run → `examples/`.
- **The standard library** (written in DeluluLang) → `stdlib/std/`.
- A code range (`DLxxxx`) is allocated in exactly one stage and never reused; the registry lives
  in `crates/delulu-diag/src/codes.rs` and grows per stage.
