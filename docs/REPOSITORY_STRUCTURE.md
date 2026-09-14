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

**Re-synchronized a third time 2026-08-11.** Five crates (`delulu-wasm`, `delulu-registry`,
`delulu-conform`, `delulu-measure`, `delulu-survey`) carried a description and **no module list at
all**, and `delulu` omitted `run_cmd.rs` — so ten real modules appeared nowhere, including
`delulu-wasm/host.rs`, which shares the filesystem-containment helper with the interpreter. §5 was
missing eleven documents: five dated `red-team-*` directories with their agents' working notes,
`DEPLOYMENT.md`, both 2026-08-1x campaign records, and a maintenance record. **Both directions are now
checked mechanically** — every module in every crate appears here, no listed module is absent from the
tree, and every markdown file is accounted for by name or by its group.

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
│   │   └── src/{lib,gen,codegen,artifact,dpx,actors,host,limits}.rs
│   │                               #   host.rs - the deny-by-default Wasmtime host. It shares
│   │                               #     `prim::contains_on_disk` with the interpreter ON PURPOSE:
│   │                               #     engine fault parity is a tested law, so a containment
│   │                               #     rule holding in one engine and not the other would be a
│   │                               #     divergence in the direction that matters most
│   │                               #   limits.rs - fuel, memory and wall-clock bounds
│   ├── delulu-registry/            # [Stage 2/9] index lines, resolution, server-side authority
│   │   └── src/{lib,main,token,tests}.rs
│   ├── delulu-conform/             # [Stage 9] the conformance runner: --coverage, --check-reference
│   │   └── src/{lib,main,rules,reference,tests}.rs
│   ├── delulu-measure/             # [Stage 9] the measurement harness behind measurements/
│   │   └── src/{lib,main,corpus,study_a,study_b,study_c}.rs
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
│   │   └── src/{lib,main,scan,paths,rust,mdown,manifest,codeowners,verify,render,health}.rs
│   │                               #   mdown.rs - reads the prose: rulings, findings, links. Knows
│   │                               #     BOTH shapes a finding is recorded in (the original ledger
│   │                               #     table and the `### C<n>` sections later passes use);
│   │                               #     knowing only one made it report eleven real findings as
│   │                               #     "not campaign findings" (SURVEY-HEADING-1)
│   │                               #   health.rs - the map-integrity knowledge `delulu doctor` and
│   │                               #     the survey's own tests BOTH consume; there is one doctor
│   └── delulu/                     # [Stage 1] the `delulu` CLI — every user-facing command lives
│       │                           #   here. `cli.rs` dispatches and owns `usage()`; a test binds
│       │                           #   the dispatcher, `--help` and the generated completion
│       │                           #   script to ONE command list, so none of the three can drift
│       │                           #   (completions_cli.rs). One command per module below:
│       ├── src/{main,cli,run_cmd}.rs  # dispatch + the Stage-1 commands (check, fmt, test, run)
│       │                           #   main.rs runs the CLI on a thread with a stack sized
│       │                           #     for the interpreter's depth bound, so deep recursion
│       │                           #     is DL0905 and never a host crash
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
│                                   #   evidence_claims.rs [P20] — THE EVIDENCE GATE. What the
│                                   #     repository may SAY about its verification is decided by
│                                   #     what docs/design/models/ CONTAINS. Walks every shipped
│                                   #     .md (not a list — the defect was a file escaping notice
│                                   #     by not being in one) and fails BOTH ways: no document may
│                                   #     deny evidence on disk, and no machine-checked claim may
│                                   #     outlive the artifact backing it. The skip branch is
│                                   #     STRUCTURAL (is the phrase inside a code span / strike /
│                                   #     quotes?) because prose markers misread this gate's own
│                                   #     CHANGELOG entry as an assertion.
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
│       │                           #   with esbuild into dist/extension.js and packages a .vsix.
│       │                           #   BUNDLED ON PURPOSE: shipping the dependency tree instead
│       │                           #   produced a .vsix that packaged cleanly and would have
│       │                           #   thrown `Cannot find module` on activation (vsce shipped 1
│       │                           #   of the 8 packages npm installed).
│       │                           #   The server/client command contract is gated by
│       │                           #   crates/delulu/tests/editor_contract.rs.
│       ├── server-resolve.js       #   Which program gets launched. Separate from extension.js and
│       │                           #   free of the `vscode` API so it is testable without an
│       │                           #   editor. Refuses relative paths and walks PATH itself: a bare
│       │                           #   name handed to spawn is resolved by the OS, and on Windows
│       │                           #   CreateProcess searches the CURRENT DIRECTORY before PATH.
│       ├── snippets/delulu.json    #   Every snippet that declares a function carries its EFFECT
│       │                           #   ROW — a snippet producing `fn f() { … }` without one would
│       │                           #   teach the declaration and leave the checker to object later.
│       ├── test/resolve.test.js    #   `npm test` — node:test, no framework dependency, because
│       │                           #   every dependency an extension carries ships to every user.
│       ├── e2e.js                  #   Launches a REAL VS Code against a REAL server and requires
│       │                           #   three POSITIVE signals: activation, a live `delulu … lsp`
│       │                           #   process, and publishDiagnostics in the trace. Written after
│       │                           #   the extension shipped with a server that never started while
│       │                           #   every test was green — lsp_cli.rs talks to the server but is
│       │                           #   not a VS Code client, and editor_contract.rs reads source
│       │                           #   text, which cannot say what a library does at runtime.
│       │                           #   NOT in `npm test`: it needs a display. Kept out of test/
│       │                           #   because node:test treats every file there as a test.
│       └── verify-package.js       #   Unpacks the built .vsix and refuses one whose requires do
│                                   #   not resolve — a green package step proves nothing about
│                                   #   whether the thing inside runs.
│
├── deny.toml                       # [P17-F] the SUPPLY-CHAIN gate: `cargo deny check advisories
│                                   #   bans licenses sources`. Before it existed nothing checked
│                                   #   the tree against RustSec, and 19 vulnerabilities had
│                                   #   accumulated. Every `ignore` carries a falsifiable reason;
│                                   #   the 4 REACHABLE advisories are deliberately NOT ignored, so
│                                   #   `advisories` is RED on purpose.
├── scripts/
│   ├── package-toolchain.sh        # build the self-contained distributable archive
│   └── cli-sweep.sh                # [P17-F] the CLI + compiler sweep as a SCRIPT (27 cases at
│                                   #   P19; 22 when written at P17-F), each asserting an exact
│                                   #   exit code. It was performed by hand every pass before
│                                   #   this, which is the drift design rule 1 warns about. The
│                                   #   script COUNTS its own cases — read the tail, not this line.
├── rfcs/                           # [Stage 10] the RFC process: language/authority changes
├── release-artifacts/              # [Stage 9] built release outputs
├── SECURITY.md                     # reporting policy + rehearsed patch runbook
├── CHANGELOG.md                    # notable changes; every entry names its authorizing ruling
├── CONTRIBUTING.md                 # contribution rules; §4 governs AI-authored RFCs
├── .github/workflows/              # the three-OS CI matrix (live since the 2026-09-14 push; no result read yet)
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
    ├── DEPLOYMENT.md               # what a deployment actually protects, and what you must do to
    │                               #   get it: three tiers (single-user legacy / strict anchored
    │                               #   roots / separate OS account), how to VERIFY each with
    │                               #   `delulu doctor`, honest per-platform status, the list of
    │                               #   things that are NOT protected, and the recorded ruling on
    │                               #   why strict mode is not yet the default
    ├── REMAINING_WORK.md           # [2026-08-23] everything SPECIFIED, DESCRIBED or IMPLIED that
    │                               #   is not built, in one place — assembled by reading all 161
    │                               #   markdown files and checking each claim against the CURRENT
    │                               #   binary. Sections: the language and compiler, backends,
    │                               #   containment, proof, tooling, platform. §1 runs the other
    │                               #   way: documents that were STALE because the code had moved
    │                               #   ahead of them. Not a schedule and not a promise.
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
    │   │                                #   §9's ~500-line Lean/Coq mechanization of §1–§7 is still
    │   │                                #   OPEN — no capabilities, store, secrets, attenuation,
    │   │                                #   Progress or Preservation. But category 2 is NOT empty:
    │   │                                #   models/lean/DeluluCore.lean machine-checks the
    │   │                                #   HIGHER-ORDER FRAGMENT (Lean 4.32.2, zero axioms).
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

## 5. Every document in this repository, and what it is for

**Every markdown file in the repository.** The count itself is not written here on purpose — it moves
with every campaign, and the Survey recounts it from the tree (`docs/survey/SURVEY.md` § Measured
facts) where a stale figure fails a test. This section exists because a large fraction of the prose
was once named nowhere in this document, and a structure guide that omits most of the writing is a
guide to the code only.

Series (the 16 semantics chapters, the 11 localizations, the per-study measurement records, the
red-team agents' working notes) are grouped where the group is the useful unit; every file is
accounted for by name or by its group.

### 5.1 Root — the front door

| File | Purpose |
| --- | --- |
| `README.md` | The project in one page, including a "not distributed / not certified" table that is deliberately the first thing a reader meets. |
| `HANDOFF.md` | **Start here in a new session.** Standing rules, what is built, which document to read, the feature explanations, and the open-problem list with reasons. |
| `INSTALL.md` | Building from source and the portable archive; why the shipped binary embeds no Python, and why `cargo install delulu` from crates.io is deliberately not offered. |
| `CHANGELOG.md` | Keep-a-Changelog, every entry naming the ruling that authorized it. Versions follow the **semver-authority law**: any widening of what a package may do is a major bump, even with an unchanged API. |
| `GOVERNANCE.md` | Who decides what, and how a decision is recorded. |
| `CONTRIBUTING.md` | How to propose a change, and the rules a change must satisfy. It binds this project too, which `governance.rs` tests. |
| `SECURITY.md` | The disclosure policy and the threat model's edges. |
| `TRADEMARK.md` | Name and mark usage. Owner-reserved territory. |

### 5.2 `docs/` — what a user or an agent reads

| File | Purpose |
| --- | --- |
| `GETTING_STARTED.md` | Install to first program to real programs to your editor. The path a new developer walks. |
| `for-agents.md` | **Driving the toolchain as an AI agent**: exit codes, JSON envelopes, why to batch `check` and nothing else, and what to ask the language server instead of shelling out. |
| `QUESTIONS.md` | The hard questions answered with evidence — can an agent bypass Authority, is any of this real mathematics — and an enumerated list of known leaks. |
| `DEPLOYMENT.md` | **What a deployment actually protects, and what you must do to get it.** Three tiers (single-user legacy / strict anchored roots with an offline anchor / separate OS account), the exact commands, how to verify each with `delulu doctor`, per-platform status, an explicit list of what is NOT protected, and the recorded ruling on why strict anchored-root mode is not yet the default. |
| `MATHEMATICS.md` | Every formal claim with its evidence category (1–7). **A claim with no category is a claim to be deleted or demoted.** |
| `REMAINING_WORK.md` | **Everything specified, described or implied that is not built** — the join across `CHECKPOINT-1.0.md` §8/§9, `QUESTIONS.md` Part 5, `MATHEMATICS.md` §12, the Book's Ch. 20 and `HANDOFF.md` §8, with every row re-checked against the current binary instead of inherited from prose. §1 records the reverse case: current-state documents that had gone stale because the code moved ahead of them. |
| `REPOSITORY_STRUCTURE.md` | This file. |
| `editors.md` | One server, every editor. Per-editor setup, what the server does and cannot do, and the editor surface's security history. |

### 5.3 `docs/design/` — normative specifications and campaign records

**Specifications** — what each stage is contractually required to do. `LANGUAGE_SPECIFICATION.md` is
the language itself and `DELULU_CORE.md` the irreducible core the rest rests on; the per-stage specs
are `STAGE1_SPECIFICATION.md` (core language), `STAGE2_SPECIFICATION.md` (packages),
`STAGE3_SPECIFICATION.md` (WASM), `STAGE4_SPECIFICATION.md` (foreign/Python),
`STAGE5_SPECIFICATION.md` (custody), `STAGE6_SPECIFICATION.md` (plugins),
`STAGE7_SPECIFICATION.md` (actors), `STAGE8_SPECIFICATION.md` (editor surface),
`STAGE9_SPECIFICATION.md` (ecosystem) and `STAGE10_SPECIFICATION.md` (industrial).

**Build orders** — the ruling ledgers (`D<n>`) that authorized each change: `STAGE6_BUILD_ORDER.md`,
`STAGE7_BUILD_ORDER.md`, `STAGE8_BUILD_ORDER.md`, `STAGE9_BUILD_ORDER.md`, `STAGE10_BUILD_ORDER.md`.
See §4 for how to cite a bare `D<n>`.

**Addenda and guides** — a subsystem explained rather than specified:

| File | Purpose |
| --- | --- |
| `CONSTITUTION.md` | The project's own rules. §8.4 forbids editor-specific server features, and that is load-bearing. |
| `AUTHORITY_GUARD_CAPSTONE.md` | Authority and the Guard presented together as one argument rather than two subsystems. |
| `STAGE5_GUARD_ADDENDUM.md` | The Guard's design: three tiers, broker-held permits, owner codes — and what was adopted from and rejected of `dcg`, with credit. |
| `STAGE6_PLUGINS_GUIDE.md` | Plugins for users: the two classes, and why a Verified plugin that fails re-checking is refused rather than demoted to Contained. |
| `STAGE7_ACTORS_GUIDE.md` | Actors for users: mailboxes, FIFO ordering, quiescence, and what is *not* prevented. |
| `STAGE8_SURFACE_GUIDE.md` | The editor surface, carrying the specification's §11 honesty caveats verbatim — a test pins them character-for-character. |
| `STAGE10_AUTONOMY_ADDENDUM.md` | Autonomy: dead-man timers, sign-off records, e-stop. |
| `SURFACE_ATLAS_PALETTE_ADDENDUM.md` | The Atlas and the palette as user-facing surfaces. |
| `SYNTAX_MORPH_SPEC.md` | Morphs — surface keyword skins. The program is unchanged; codes and JSON never pass through one. |
| `LOCALIZATION_PLUGIN_GUIDE.md` | Catalog plugins: verified-class, **zero authority**, prose only. |
| `REGISTRY_POLICY.md` | What the registry accepts, and why it recomputes authority server-side rather than trusting the client's claim. |
| `AI_NATIVE_DESIGN.md` | Why the language is shaped for machine consumers as much as human ones. |
| `DeluluLang_PROMPT.md` | The original brief the project was built from. Historical. |

**Reviews, audits and campaigns** — the adversarial history:

| File | Purpose |
| --- | --- |
| `SOUNDNESS_AUDIT.md` | Where the type system's soundness argument is, and is not, complete. |
| `HARDENING_CAMPAIGN.md` | **P16.** The `C<n>` finding ledger — the "what is known to be wrong" list README points readers at. Its opening table covers the original campaign; every later pass records its findings as `### C<n> · …` sections under its own dated heading. |
| `PRODUCTION_READINESS_2026-08-09.md` | The 2026-08-09 overnight sweep: six defects, each with a witness that failed against the pre-fix code, and the four evidenced negatives. |
| `PRODUCTION_READINESS_2026-08-10.md` | The 2026-08-10 containment + deployment campaign: thirteen defects and one documented residual, the through-line that found four of them, and the production-readiness verdict with its scope stated. |
| `PROOF_CAMPAIGN.md` | **P17 and P18.** Evidence categories, proof boundaries, the F1–F4 authority findings, and the Miri story end to end. |
| `P19_ECOSYSTEM_REVIEW.md` | **P19, the most recent pass.** Seven personas; every claim tied to something executed and every gap named. |
| `PRODUCTION_READINESS_REVIEW.md` | The pre-1.0 readiness pass. |
| `STAGE10_AUTONOMY_HONESTY_REVIEW.md` | An honesty scrub of the autonomy claims specifically. |
| `CROSS_PLATFORM_VERIFICATION.md` | **Which platforms have actually been executed**, and which are merely prepared. macOS, CI and containers each have a section saying plainly that they have not run. |
| `STABILITY.md` | What may change, when, and what is frozen. |
| `ENTRENCHED_CHANGE_RECORD.md` | **The approval log for CODEOWNERS-protected paths.** Every change to a path Constitution §10 reserves to the project lead, with who approved it, what was verified first, why it was or was not an RFC, and the revert command. Shaped after `docs/survey/REMOVALS.md`. |
| `VERSION_COEVOLUTION.md` | How the language, the toolchain and packages move together. |
| `THREADED_WASM_DEFERRAL.md` | A deferral, recorded rather than dropped. |

### 5.4 `docs/reference/` — the precise surfaces

| File | Purpose |
| --- | --- |
| `README.md` | What the reference section covers and how to navigate it. |
| `cli.md` | Every command and flag. |
| `diagnostics.md` | Every `DLxxxx` code. |
| `grammar.md` | The concrete grammar. |
| `tokens.md` | The token model. |
| `primitives.md` | **The callable surface.** There is no standard library; this plus the prelude builtins is all of it. |
| `semantics-5-1.md`, `semantics-5-2.md`, `semantics-5-3.md`, `semantics-5-4.md`, `semantics-5-5.md`, `semantics-5-6.md`, `semantics-5-7.md`, `semantics-5-8.md`, `semantics-5-9.md`, `semantics-5-10.md`, `semantics-5-11.md`, `semantics-5-12.md`, `semantics-5-13.md`, `semantics-5-14.md`, `semantics-5-15.md`, `semantics-5-16.md` | Sixteen chapters of operational semantics, one per §5 subsection of the specification — the normative meaning of each construct. |
| `audit-rules.md` | What the audit log records, and what verifying it does and does not prove. |
| `coverage.md` | The conformance coverage law (invariant 42) and how witnesses are counted. |

### 5.5 `docs/book/` — the tutorial

`THE_DELULULANG_BOOK.md` and `samples/`. **The samples are conformance tests.** A tutorial whose
examples do not compile teaches something false and wastes an afternoon proving it, so `book.rs`
checks every sample with the real binary on every run.

### 5.6 `docs/lang/` — localized human prose

`en-US.md` (the source of truth), `delulu-slang.md` (an AI-compact profile), and the human
localizations: `ar-SA.md`, `de-DE.md`, `es-ES.md`, `fr-FR.md`, `hi-IN.md`, `ja-JP.md`, `ko-KR.md`,
`pt-BR.md`, `zh-CN.md`.
**Diagnostic codes and JSON never localize** — only prose does, and the machine envelope never passes
through a morph at all.

### 5.7 `docs/playbooks/` — how a stage was actually built

`README.md` plus `STAGE4_PLAYBOOK.md`, `STAGE5_PLAYBOOK.md`, `STAGE6_PLAYBOOK.md`,
`STAGE7_PLAYBOOK.md`, `STAGE8_PLAYBOOK.md`, `STAGE9_PLAYBOOK.md` and `STAGE10_PLAYBOOK.md`. Process
records: the order the work was done in and the decisions taken along the way. Read these when you want to know *why* a stage
looks the way it does, rather than what it does.

### 5.8 `docs/release/` — the 1.0 artifacts

| File | Purpose |
| --- | --- |
| `CHECKPOINT-1.0.md` | **The best single status page**: architecture, testing, and twenty known limitations named without softening. |
| `CHECKLIST-1.0.md` | The release checklist and its dispositions. |
| `ANNOUNCEMENT-1.0.md` | The announcement text. Unpublished, because nothing is distributed. |
| `SUPPORT_MATRIX.md` | Platforms, release trains, and what support means. |
| `SBOM-1.0.json` | The software bill of materials. |
| `PROVENANCE-1.0.json` | Build provenance. |

### 5.9 `docs/security/` and `docs/maintenance/`

- `security/DRILL-001.md` — a recorded incident drill: a verdict string and an exit code that
  disagreed. `for-agents.md` points at it as the reason an agent must read exit codes rather than
  output.
- `security/red-team-*/` — **five dated red-team records**, each a directory rather than a file
  because the working notes are part of the evidence:
  - `red-team-P20-2026-08-08/` and `red-team-P20-custody-2026-08-08/` — the multi-agent adversarial
    authority tests, including each agent's raw `agent-programs/` and `agent-notes/`. Kept because a
    red-team pass that publishes only its conclusions cannot be audited.
  - `red-team-surfaces-2026-08-08/` — the untested-surface sweep (IPC, dead-man, WASM), with an
    `ADJUDICATION.md` recording which agent claims survived re-verification against the current
    binary and which evaporated. **That file is the point of the directory**: agents reported a
    "CRITICAL" that a code read disproved, and a "BROKE" that was a stale build.
  - `red-team-disc1-root-issuance-2026-08-08/` — DISC-1: a same-user process minting root authority
    and commanding a guard-sealed device. Carries a status banner for what has since shipped; the
    finding itself is **not** retracted, because on a default broker it still reproduces.
  - `red-team-p21-crossaccount-2026-08-08/` — the separate-OS-account boundary tested with a real
    second UID: holds on POSIX, absent on 9p.
- `maintenance/DISK-CLEANUP-2026-08-04.md`, `DISK-CLEANUP-2026-08-09.md` — maintenance actions,
  recorded rather than forgotten: what was deleted, what was verified first, and what was reclaimed.

### 5.10 `measurements/` and `rfcs/`

Every `measurements/*/RECORD.md` (and `study-*/REPORT.md`) is the **reproducible evidence** behind one
published claim: `agent-loop` (why to batch `check`), `compute`, `dead-man`, `federation-demo`,
`first-run`, `fleet-update`, `lts-cycle`, `robotics-demo`, `satellite-demo`, `scale`,
`pqc/KAT_RECORD.md`, and studies `a`, `b`, `c` (`study-c` also carries `HOT_PATH_TABLE.md`). These are
**measurements, not benchmarks**: each records the machine, the method and the numbers, so a reader
can disagree with the method rather than only with the conclusion.

`rfcs/0000-template.md` is the RFC template — `governance.rs` pins its hard sections so they cannot be
dropped. `rfcs/0001-broker-federation.md` is the federation RFC: **sponsored, and partly built during
its own comment period**, which is recorded as a governance deviation and must never be restated as
compliance.

### 5.11 Generated documents — do not hand-edit

`docs/survey/SURVEY.md`, `survey.json`, `DISCREPANCIES.md` and `REMOVALS.md` are produced by
`cargo run -p delulu-survey -- build` and **committed**, so the map travels with the tree. A test
(`the_committed_map_matches_the_tree`) fails when they are stale and `delulu doctor` regenerates them.
Editing them by hand is editing the output of a scan.
