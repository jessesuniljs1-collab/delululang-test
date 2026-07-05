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
│   │   ├── Cargo.toml
│   │   └── src/{lib,span,source,codes,diagnostic,json,render}.rs
│   ├── delulu-syntax/              # tokens, lexer (Go-style termination), AST, error-recovering parser
│   │   ├── Cargo.toml
│   │   └── src/{lib,token,lexer,ast,parser}.rs
│   ├── delulu-check/               # [Stage 1] resolve, types, rows, THE effect/authority checker
│   │   └── src/{lib,resolve,types,row,unify,check,authority,secret}.rs
│   ├── delulu-runtime/             # [Stage 1] values, capability table, interpreter, grant broker
│   │   └── src/{lib,value,cap,broker,interp,prim,manifest}.rs
│   └── delulu/                     # [Stage 1] the `delulu` CLI: check | run | repl | authority
│       └── src/{main,cli,repl}.rs
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
    │   ├── STAGE1_SPECIFICATION.md … STAGE10_SPECIFICATION.md
    │   ├── LANGUAGE_SPECIFICATION.md   # superseded early draft — kept for provenance
    │   └── DeluluLang_PROMPT.md        # Jesse's original vision — kept verbatim, never edited
    ├── reference/                  # [Stage 9] generated-in-part language reference
    └── book/                       # [Stage 9] the Delulu Book (learn-by-building tutorial)
```

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
