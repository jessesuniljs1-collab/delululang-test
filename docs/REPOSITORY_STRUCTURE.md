# DeluluLang — Repository Structure & Move Manifest

**Status:** Living. This document is the map of the repository. It matches the workspace layout
the stage specs assume (Stage-1 spec §9.2). The moves in §3 were executed at repo initialization
(2026-07-05).

**Re-synchronized 2026-07-24** against the actual tree (`HARDENING_CAMPAIGN.md` C5). Between
Stage 1 and Stage 10 this file had drifted in both directions: it drew directories that were never
built and omitted most of what the later stages added. §1 below is now what is *there*, not what
was once planned. Anything aspirational is marked as such inline — a map that mixes the two
silently is worse than no map.

**Re-synchronized again 2026-08-03** (P17 proof campaign), because it had drifted a second time and
in the same direction: §1 listed `delulu-check` as eight modules when it has seventeen, gave
`delulu-runtime` a `cap.rs` and a `manifest.rs` that do not exist while omitting `actors`, `custody`,
`trace` and `pqc`, and omitted `cert.rs` and `device_scope.rs` from `delulu-broker` although the
federation work (D21–D22) added both. Every crate's module list in §1 is now generated from the
tree rather than remembered.

**This drifted twice in six weeks, which is the argument for not relying on it.** A hand-maintained
map falls behind the thing it maps — the project's own design rule 1. `docs/survey/` is derived from
the tree by `cargo run -p delulu-survey -- build`, cites a `file:line` on every edge, and **a test
fails when it is stale**. Treat §1 as orientation for a human and the Survey as the current truth.

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
│   │   ├── src/{lib,span,source,codes,catalog,diagnostic,json,render,palette}.rs
│   │   └── tests/unproducible_witnesses.rs   # every registered code must be producible
│   ├── delulu-syntax/              # tokens, lexer (Go-style termination), AST, error-recovering parser
│   │   ├── Cargo.toml
│   │   └── src/{lib,token,lexer,ast,parser,num,fmt,grammar,morph}.rs
│   │                               #   morph.rs [D35]: the canonical-form law for surface morphs
│   │                               #   lexer.rs also holds the DL0107/DL0108 raw-byte scans
│   ├── delulu-check/               # [Stage 1] resolve, types, rows, THE effect/authority checker
│   │   ├── src/{lib,resolve,ty,unify,check,authority,program}.rs        # the core pipeline
│   │   ├── src/{manifest,package,lockfile,deps,deprecation}.rs          # package + dependency layer
│   │   ├── src/{dir,prim_table,plugin,rcaps,rcap_check}.rs              # DIR/CBOR, §7.3 table,
│   │   │                           #   plugin typing, reference capabilities (iso/val)
│   │   └── tests/{laundering,secret_oracle,work_scaling}.rs
│   │                               #   laundering.rs   — SOUNDNESS_AUDIT F-1…F-6 as rejection tests
│   │                               #   secret_oracle.rs — P17-IF1: the map+verify declassification
│   │                               #     oracle, plus controls pinning that the fix is not a
│   │                               #     blanket refusal (docs/design/PROOF_CAMPAIGN.md §IF-1)
│   ├── delulu-runtime/             # [Stage 1] values, interpreter, primitives, custody, actors
│   │   ├── src/{lib,value,interp,prim,broker,custody,trace}.rs          # core execution + trace⊆row
│   │   ├── src/{actors,cycles,foreign,python,plugin}.rs                 # [Stages 4/6/7]
│   │   ├── src/{device,compute,adapter,pqc}.rs                          # [Stage 10] physical boundary,
│   │   │                           #   compute dispatch, the operator-supplied subprocess adapter (D23)
│   │   └── tests/{actors_pingpong,actors_promise,actors_trace,foreign_ffi,python_embed}.rs
│   ├── delulu-broker/              # [Stage 5] custody core: ⊑ lattice, grant tree, validation
│   │   │                           #   classes, audit chain, lease tokens, secrets, THE GUARD
│   │   ├── src/{lib,authority,path,tree,ids,time,validate,diag,audit,lease,secrets,guard}.rs
│   │   ├── src/{cert,device_scope}.rs   # [RFC 0001 / D21–D22] federation certificates and the
│   │   │                           #   `device` scope dimension with interval containment
│   │   └── tests/{audit_wiring,holder_neutrality,order_laws,audit_truncation,device_identity,clock_monotonicity}.rs
│   │                               #   audit_truncation.rs [P17-7, FIXED]: truncation is now
│   │                               #     detected via ANCHOR.json (head + count, outside the log).
│   │                               #     One test PINS the honest limit: rewriting log AND anchor
│   │                               #     is still undetectable without an external witness.
│   │                               #   clock_monotonicity.rs [P17-B2, FIXED]: Broker::now ratchets,
│   │                               #     so a backwards clock cannot resurrect expired authority
│   │                               #   device_identity.rs [P17-7]: authorization reads the map KEY
│   │                               #     while the signed bytes read the value's .device FIELD;
│   │                               #     latent, since cert loads re-key (a test pins that too)
│   │                               #   order_laws.rs — P17: the ⊑/⊓ algebra checked by EXHAUSTIVE
│   │                               #     enumeration, not spot-checks. F1–F3 were found FAILING
│   │                               #     here and are now CLOSED (2026-08-06) by canonicalization
│   │                               #     at the custody boundary: 9 passed, 0 ignored. Each keeps a
│   │                               #     companion test that still OBSERVES the raw-spelling
│   │                               #     preorder, so the reason for canonicalizing cannot become
│   │                               #     folklore. The canonical form is an ANTICHAIN of canonical
│   │                               #     spellings — set redundancy ({./data,./data/sub} ≡
│   │                               #     {./data}) is a second collapse the Z3 model could not see.
│   │                               #   multi_agent_stress.rs [P18] — random authority graphs at
│   │                               #     10/50/100/250/500/1000 agents plus 20 topologies, checking
│   │                               #     attenuation-to-root, inherited revocation, inherited
│   │                               #     expiry, and that every stored authority is canonical.
│   │                               #     Deliberately builds authorities by STRUCT LITERAL: using
│   │                               #     Authority::new made the canonicality law vacuous, and
│   │                               #     deleting every canonicalization call still passed.
│   ├── delulu-atlas/               # [Stage 8, early] the Atlas: typed deterministic code+authority
│   │   │                           #   graph from compiler facts (atlas/1, digest, query verbs)
│   │   └── src/{lib,model,build,render,query,formats}.rs
│   ├── delulu-wasm/                # [Stage 3] the WASM backend: .dwx emission, engine parity
│   ├── delulu-registry/            # [Stage 2/9] index lines, resolution, server-side authority
│   ├── delulu-conform/             # [Stage 9] the conformance runner: --coverage, --check-reference
│   ├── delulu-measure/             # [Stage 9] the measurement harness behind measurements/
│   ├── delulu-fuzz/                # [Stage 2] the differential fuzz harness: generate programs,
│   │   │                           #   check them, and assert the runtime trace ⊆ the statically
│   │   │                           #   computed row (Effect Soundness, executable)
│   │   └── src/{lib,main,danger}.rs
│   │                               #   danger.rs [P17-D]: the families the original generator
│   │                               #     COULD NOT EXPRESS — type parameters reaching higher-order
│   │                               #     builtins (C88, both the List.map and Secret.map twins),
│   │                               #     closures carrying rows, row variables, and the
│   │                               #     Secret.map+verify oracle (IF-1). The old grammar had no
│   │                               #     generics, closures, Secret ops or control flow, so it
│   │                               #     could not write either hole it was hunting.
│   ├── delulu-survey/              # repository tooling (`publish = false`, no sibling deps):
│   │                               #   derives docs/survey/ — the map of THIS REPOSITORY, with a
│   │                               #   file:line citation on every edge. Not language surface.
│   └── delulu/                     # [Stage 1] the `delulu` CLI — every user-facing command lives
│       │                           #   here. `cli.rs` dispatches and owns `usage()`; a test binds
│       │                           #   the dispatcher, `--help` and the generated completion
│       │                           #   script to ONE command list, so none of the three can drift
│       │                           #   (completions_cli.rs). One command per module below:
│       ├── src/{main,cli}.rs       #   dispatch + the Stage-1 commands (check, fmt, test, run, …)
│       ├── src/new.rs              #   `new`         scaffold a package; its ceiling is minimal
│       ├── src/fix.rs              #   `fix`         apply typed repairs; never widens authority
│       ├── src/doctor.rs           #   `doctor`      is this machine — and this checkout — healthy?
│       ├── src/completions.rs      #   `completions` generated shell completion (bash/zsh/fish/pwsh)
│       ├── src/lsp.rs              #   `lsp`         the language server, analysis only (§11)
│       ├── src/repl.rs             #   `repl`
│       ├── src/{signing,deploy,fleet}.rs        # keygen/sign/verify-sig/publish/add/login; deploy; fleet
│       ├── src/{locale,morph_file}.rs           # catalog plugins; surface morphs (see morphs/)
│       ├── src/{advisories,cert_crypto}.rs      # advisory feed; federation certificate crypto
│       ├── src/{brokerd,broker_ipc,broker_client,broker_transport}.rs   # [Stage 5] custody
│       ├── src/{foreign_worker,microvm}.rs      # process isolation; microVM (Linux only)
│       └── tests/                  # binary-level integration: new_cli, fix_cli, completions_cli,
│                                   #   doctor_cli, lsp_cli, cli_contract, json_contract,
│                                   #   broker_cli, grants_cli, audit_cli, foreign_worker,
│                                   #   microvm_criterion8, guard_cli, guard_e2e, palette_cli,
│                                   #   atlas_cli, atlas_e2e, …
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
├── measurements/                   # every published number, with its date and its threats
│   ├── agent-loop/                 # the FLOOR: what one invocation costs before a program grows
│   │                               #   (82% of a small `check` on Windows is process creation) —
│   │                               #   the others measure the MARGINAL cost of size
│   ├── scale/                      # `check` against lines and shapes; monorepos; runtime widths
│   ├── study-a/ study-b/ study-c/  # authority verification · agent repair loops · vs C
│   └── METHODOLOGY.md              # the rules every study follows, incl. its negative controls
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
│   ├── core-invariance/            # SNAPSHOT.txt — the exact bytes the core answers with, for
│   │                               #   every program below (109 targets, 362 cases). GENERATED:
│   │                               #   DELULU_BLESS=1 cargo test -p delulu --test core_invariance
│   │                               #   Guards what the coverage law does not: the conformance
│   │                               #   suite pins each DIAGNOSTIC CODE, this pins the MESSAGES,
│   │                               #   SPANS, REPAIRS and AUTHORITY REPORTS. Tooling built around
│   │                               #   the language cannot move them without a reviewed diff.
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
│   └── vscode/                     #   extension.js is the SOURCE; `npm run package` bundles it
│                                   #   with esbuild into dist/extension.js and packages a .vsix.
│                                   #   BUNDLED ON PURPOSE: shipping the dependency tree instead
│                                   #   produced a .vsix that packaged cleanly and would have
│                                   #   thrown `Cannot find module` on activation (vsce shipped 1
│                                   #   of the 8 packages npm installed). `verify-package.js`
│                                   #   unpacks the built archive and refuses one whose requires
│                                   #   do not resolve — a green package step proves nothing about
│                                   #   whether the thing inside runs.
│                                   #   The server/client command contract is gated by
│                                   #   crates/delulu/tests/editor_contract.rs.
│
├── deny.toml                       # [P17-F] the SUPPLY-CHAIN gate: `cargo deny check advisories
│                                   #   bans licenses sources`. Before it existed nothing checked
│                                   #   the tree against RustSec, and 19 vulnerabilities had
│                                   #   accumulated. Every `ignore` carries a falsifiable reason;
│                                   #   the 4 REACHABLE advisories are deliberately NOT ignored, so
│                                   #   `advisories` is RED on purpose.
├── scripts/
│   ├── package-toolchain.sh        # build the self-contained distributable archive
│   └── cli-sweep.sh                # [P17-F] the CLI + compiler sweep as a SCRIPT (22 cases,
│                                   #   exact exit codes). It was performed by hand every pass
│                                   #   before this, which is the drift design rule 1 warns about.
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
    ├── MATHEMATICS.md              # [P17 capstone] EVERY mathematical structure the language uses:
    │                               #   WHAT it is, WHERE (file:line), WHY that structure, and HOW
    │                               #   STRONG — each claim in exactly ONE of seven proof-boundary
    │                               #   categories. Names the RETRACTED claims (antisymmetry of ⊑,
    │                               #   symmetry of ⊓) and the EMPTY category (machine-checked, for
    │                               #   the type system).
    ├── QUESTIONS.md                # hard questions answered with evidence — can authority be
    │                               #   bypassed, how it compares to a sandbox, what the maths does
    │                               #   and does not prove, and the things this project cannot claim
    ├── editors.md                  # editor/LSP setup
    ├── design/                     # the committed design corpus (constitution, audit, stages)
    │   ├── CONSTITUTION.md
    │   ├── SOUNDNESS_AUDIT.md          # rules R-1…R-8 + R-2b; carries the R-4 and R-2/R-5
    │   │                                #   REOPENING boxes — findings against the audit itself
    │   ├── HARDENING_CAMPAIGN.md       # the "test it to failure" campaign (C1…C92, P1…P16)
    │   ├── PROOF_CAMPAIGN.md           # [P17, 2026-08-03] the PROOF-BOUNDARY LEDGER: every
    │   │                                #   guarantee in exactly one of seven categories (proven /
    │   │                                #   machine-checked / model-checked / property-tested /
    │   │                                #   differentially verified / fuzz verified / OUTSIDE the
    │   │                                #   boundary). No grey areas. Records IF-1 and F1–F4.
    │   ├── models/                 # [P17-C/P17-6] FORMAL MODELS + the checker's verbatim output.
    │   │   ├── Broker.tla           #   the custody grant tree: grant/delegate/revoke/expire, each
    │   │   │                        #   guard citing the tree.rs line it mirrors
    │   │   ├── Broker.cfg           #   today's code — 585,771 distinct states, no error
    │   │   ├── BrokerBug.cfg        #   TEETH TEST: the pre-RFC-0001-F4 read, which TLC must fail —
    │   │   │                        #   it rediscovers the real historical bug at depth 4
    │   │   ├── Custody.tla          #   LEASES + CERTIFICATE ADOPTION — lease.rs (delegate/mint/
    │   │   │                        #   redeem, single-use nonces, key rotation) and cert.rs
    │   │   │                        #   (adoption, single-adoption, uplink deadlines). This is
    │   │   │                        #   where BOTH real vulnerabilities lived.
    │   │   ├── Custody.cfg          #   both fixes on — 2,421 distinct states, no error
    │   │   ├── CustodyReplay.cfg    #   TEETH TEST: SINGLE_ADOPTION=FALSE reconstructs the
    │   │   │                        #   certificate-replay vulnerability at depth 4
    │   │   ├── CustodyC29.cfg       #   TEETH TEST: LIVE_ON_REDEEM=FALSE reconstructs campaign
    │   │   │                        #   finding C29 (redeeming a revoked grant) at depth 5
    │   │   ├── lean/DeluluCore.lean #   [P17-9] MACHINE CHECKED (Lean 4.32.2, no sorry, no
    │   │   │                        #   axioms at all): Effect Soundness for the higher-order
    │   │   │                        #   fragment, AND a proof that the calculus AS WRITTEN admits
    │   │   │                        #   a program whose trace escapes its row — C88 mechanized.
    │   │   ├── authority_algebra.py #   [P17-8] the NINE-DIMENSION order proved in Z3: 17
    │   │   │                        #   obligations — reflexive, transitive, meet-is-GLB and the
    │   │   │                        #   NO-WIDENING law across all dimensions at once. Localises
    │   │   │                        #   F1 to the path ENCODING, not the algebra.
    │   │   └── README.md            #   results, bounds, and what is NOT modelled (the MAC itself,
    │   │                            #   audit hashing, federation, concurrency, clock skew).
    │   │                            #   tla2tools.jar is NOT vendored — fetch it.
    │   ├── CROSS_PLATFORM_VERIFICATION.md # Windows + Linux green; macOS NEVER executed, said plainly
    │   ├── STABILITY.md                # what is promised to stay put (exit codes 0/1/2/3)
    │   ├── DELULU_CORE.md              # the formal calculus (paper sketches; honesty-labeled).
    │   │                                #   §9 records a ~500-line Lean/Coq mechanization as open
    │   │                                #   future work — it does NOT exist (no proof assistant is
    │   │                                #   installed), so "machine-checked" is currently empty
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
    ├── maintenance/                # operations records — machine state, NOT product behaviour
    │   └── DISK-CLEANUP-2026-08-04.md  # what was deleted from this machine, and how each deletion
    │                               #   was VERIFIED safe first (commit-ancestry + content-hash
    │                               #   lookup in the object DB). Records a WRONG diagnosis it had to
    │                               #   discard (build contention) and a WRONG check it had to fix
    │                               #   (CRLF made 15,861 files look unique; 39/40 matched once
    │                               #   normalised). Carries the ten-SIGSEGV finding so it outlives
    │                               #   the dumps it came from.
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
