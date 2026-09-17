# Implementation roadmap — phases, tasks, dependencies, verification

**Status:** proposed, 2026-09-17. Nothing below has started. The order is a recommendation; the
owner may reorder. Effort is in focused sessions (**S**), an engineering estimate, not a measurement.
Task ids are `P<phase>-<nn>`. Findings are `NE-nn` from `VERIFICATION_FINDINGS.md`; decisions are
`D-NE-nn` from `DECISION_LOG.md`; `RW n.m` is a row of `docs/REMAINING_WORK.md`.

**The protocol at the end of every phase** (owner's instruction, verbatim in intent): save all work
→ update `EXECUTION_LOG.md` → update `DECISION_LOG.md` if a decision was made → run the phase's
verification (focused tests, falsified; then the broader gates) → regenerate the Survey
(`cargo run -p delulu-survey -- build`) → `delulu doctor` → commit → push to the testing remote
only → record the commit hash and the CI run id when read → stop and report.

**Standing rules that bind every task:** harden, never redefine, Authority and Guard; a gate that
cannot fail is not a gate (falsify every new test); write the skip-branch case; witness every fix
against the pre-fix binary; consult `impact` before and regenerate the Survey after; the banned
word never enters the tree; no new dependency without a ruling; the core-invariance snapshot is
regenerated only deliberately, with the diff shown.

---

## P1 — Machine contract truth

**Goal.** Make every machine-facing promise in `docs/for-agents.md` true, and remove the walls an
agent hits in its first program. No language semantics change; one diagnostic-count change
(P1-04) that narrows nothing.

| Id | Task | Finding / row | Blast radius (Survey) | Verification | Effort |
|---|---|---|---|---|---|
| P1-01 | Record `NE-01` as a row in `REMAINING_WORK.md` §6 and correct the Book Ch. 10, `STAGE6_PLUGINS_GUIDE.md`, `examples/plugin_shout/README.md`, `README.md` and `HANDOFF.md` §6 to say the load surface is a runtime stub until P2 | NE-01 | docs only | `book.rs`, `evidence_claims.rs` | 0.3 S |
| P1-02 | A **success-envelope sweep**: a test that runs every `--json` subcommand on a valid input and asserts `command`, `schema`, `delulu_version`, `diagnostics`, `summary` are present and typed; then wrap the five `why` emitters, `add`, `plugin inspect`, `test`; `atlas --format json` keeps `atlas/1` **inside** the envelope; `--version --json` emits the envelope | NE-05 | `mod:crates/delulu/src/cli.rs` (`impact` reaches the whole CLI test set) | falsify: remove one field, watch it fail; `json_contract.rs` extended | 0.8 S |
| P1-03 | `explain --json` → envelope with `explain: {code, title, body, disposition, kind}`; an unknown option other than `--json` stays refused | NE-06 | `cli.rs::cmd_explain` | contract test + a `--bogus` refusal test | 0.3 S |
| P1-04 | Diagnostic cascade: emit `DL0210` **once** per overflow site; the parser's recovery placeholder is not type-checked as a call (no `DL0404` on `'tN`); same for `DL0211/12/13` | NE-04 | `mod:crates/delulu-syntax/src/parser.rs`, `mod:crates/delulu-check/src/check.rs` (`impact` = 134 nodes) | conformance witnesses unchanged in outcome; **core-invariance snapshot regenerated deliberately** and the diff reviewed by the owner (D-NE-3); falsify by re-enabling the cascade | 1 S |
| P1-05 | Repairs without edits: a repair object with `edits: []` must carry `requires_human: true` and a `reason`; `fix` verdicts and `check --json` flags must agree (test over the whole repair registry); LSP code actions without an edit are not `isPreferred` | NE-07 | `crates/delulu-diag/src/diagnostic.rs`, `cli.rs::fix`, `lsp.rs` | falsify by clearing the flag on one repair | 0.5 S |
| P1-06 | `test --json` carries a `diagnostics` array (codes, spans, repairs) for `check-failed` and ceiling failures; nothing human-rendered on stderr under `--json` | NE-08 | `cli.rs` test path | contract test | 0.5 S |
| P1-07 | `delulu test` with no path inside a package targets the package (`.`); outside a package the current refusal stays | NE-09 | `cli.rs` test path; `new_cli.rs` | `new_cli.rs` asserts bare `delulu test` passes in a scaffold | 0.3 S |
| P1-08 | `authority --grants` (human) and `required_grants: [string]` + `requested_scopes` (JSON, additive) — the exact `--grant` flags derived from the report plus the **literal** scope arguments visible in source, labelled *requested*; the Atlas carries `requested_scopes` too; `INSTALL.txt`'s sentence becomes true | NE-10, NE-14 | `crates/delulu-check/src/authority.rs`, `cli.rs`, `crates/delulu-atlas` | snapshot regeneration (JSON is additive but the human report changes → deliberate); a test that every grant kind the runtime parses is derivable | 1 S |
| P1-09 | `DL1603` for a `val` literal of `ref` contents: a message naming the element that forbids the lift, an explanation section for this case, and a `safe` repair (drop the `val` annotation) | NE-02 | `crates/delulu-check/src/rcap_check.rs`, `codes.rs` explain text | witness: the guide-shaped program; falsify by removing the repair | 0.8 S |
| P1-10 | Fix `examples/guide/05_capabilities.delulu` to cap-relative paths; state the rule in `explain E-DL0703`, `for-agents.md` and the Book Ch. 5; make `examples_run.rs` assert the guide's reads **succeed** (not merely run); consider an `IoErr` detail naming the capability scope on a miss | NE-03 | examples, docs, `examples_run.rs`; runtime `prim.rs` if the detail is added | falsify: reintroduce the wrong path, watch the gate fail | 0.5 S |
| P1-11 | *Candidate, needs a ruling:* `[run-authority]` in `delulu.toml` — declared grants for `delulu run <package>`, a reviewable, diffable alternative to a long `--grant` line; `--grant` on the command line still wins and the manifest `[authority]` ceiling still bounds it | RESEARCH §6 (Deno) | manifest parser, `run_cmd.rs` | ceiling law test: a `[run-authority]` wider than `[authority]` is refused | 1 S (if ruled in) |
| P1-12 | `REPOSITORY_STRUCTURE.md` accounting gate: a test that every markdown file the Survey walks is named in §5 by path or by a group glob; or reword the claim | NE-12 | `crates/delulu/tests/` | falsify by adding an unlisted file | 0.4 S |
| P1-13 | Single-file test authority: document that an effectful test needs a package `[test-authority]` today; *candidate, needs a ruling:* `delulu test --test-authority <row>` as an explicit command-line ceiling | NE-13 | docs; `cli.rs` if ruled in | ceiling test | 0.3 S (+0.5 S if ruled in) |

**Acceptance.** Every `--json` success emits the envelope (gated). `explain` has a machine channel.
The 200-deep program yields one `DL0210`. No repair claims applicability without edits. The
scaffold's `delulu test` passes bare. `authority` prints the grant line. The guide corpus reads its
files. `REMAINING_WORK.md` names NE-01. All existing gates green on CI, three OSes.

**Implications.** *Security:* none new; P1-08 exposes only what the source already states.
*Compiler/runtime:* P1-04 and P1-09 touch the checker; both keep the accepted language unchanged.
*Agents:* the loop becomes uniform. *Docs:* `for-agents.md`, the Book, `explain` texts.
*Dependencies:* none.

## P2 — Plugins for real

**Goal.** `delulu run` executes the Stage 6 load sequence for a program that calls `load`, so the
flagship demo runs from a shipped example, under a grant the operator controls, with revocation.

| Id | Task | Verification | Effort |
|---|---|---|---|
| P2-01 | Design note in the build order (a new `NEXT_EVOLUTION_BUILD_ORDER.md` or an addendum to `STAGE6_BUILD_ORDER.md`): the run-time load path through `PluginEngine`, the grant source (D-NE-10), the broker holder check as a child node, unload → revoke, Windows Contained refusal retained, `verify ≡ load` retained | reviewed before code | 0.5 S |
| P2-02 | Grant dimension for loading: operator-side (`--grant plugin=<path-or-dir>` or a `[plugins]` manifest ceiling, per the ruling); the `Load` effect already in rows; `DL0703` message names the flag | parser tests; skip-branch case (a plugin path spelled differently — `..`, symlink, case — must not widen: reuse the containment resolver) | 1 S |
| P2-03 | Wire `root.plugin_host()` → `PluginHost` capability; `load(host, path, grant)` in the interpreter: steps 1–7 of the load sequence via the `delulu-wasm` `PluginEngine` implementor; `p.get(name)` returns a callable whose row is the export's re-verified row (Verified) or `effects(grant)` (Contained, R-1) | criteria 1, 2, 3, 7 of `STAGE6_BUILD_ORDER.md` §4 re-witnessed **at the CLI level**; DL1502/DL0802/DL1504/DL1505/DL1509/DL1510/DL1511 each provoked from a `.delulu` program | 3 S |
| P2-04 | Holder check and revocation: the load creates a child grant node under the program's node in embedded and daemon custody; `unload` and `grants revoke` kill it; a revoked plugin's export faults with `PluginErr::Revoked` on next call | `guard_e2e`-style test with the daemon; falsify by skipping the node creation | 1.5 S |
| P2-05 | Limits: fuel/mem/wall from `Grant.limits` enforced (Linux live engine; Windows refusal path unchanged, D7 of Stage 6); the *near-limit signal* candidate from RESEARCH §3 recorded, not built | existing criterion-5 witnesses extended to the CLI path | 1 S |
| P2-06 | `examples/plugin_host/` — a runnable host for `plugin_shout` with a README, gated by `examples_run.rs`; the Book Ch. 10 and guide updated to the annotation form that parses | gate asserts the plugin's output appears | 0.5 S |
| P2-07 | `authority`/`why`/atlas already report `plugins:`; add the loaded node id to the audit log record and to `--trace-effects` | trace test | 0.5 S |
| P2-08 | Red team the new path: path spelling, a `.dpx` replaced between `verify` and `load` (TOCTOU — check the opened bytes' hash, which `.dpx` already carries), a grant wider than the ceiling, a Contained plugin on Windows, a plugin loading a plugin (R-7 composition) | adversarial tests, each witnessed failing on the unpatched path | 1.5 S |

**Acceptance.** `delulu run examples/plugin_host --grant console --grant plugin=…` prints the
plugin's output; every refusal code above is reachable from a program; revocation kills a loaded
plugin; the Stage 6 §4 table gains a "CLI-level" column with witnesses; `REMAINING_WORK.md` row
closed with what closed it. **Owner decision:** D-NE-10.

**Implications.** *Security:* highest of the plan — a new code-loading path; mitigated by reusing
the verified library mechanics, the containment resolver for paths, and the red-team task.
*Runtime:* `prim.rs`, `interp.rs`, `plugin.rs`, `run_cmd.rs`, `brokerd`. *Agents:* the
"extend a running system" story becomes runnable. *Docs:* Book Ch. 10, plugins guide, example
README, `E-PLUGIN`.

## P3 — Standard library, additively

**Goal.** Ordinary programs without hand-rolled loops for `filter`, `fold`, `sort`, and a map type.

| Id | Task | Verification | Effort |
|---|---|---|---|
| P3-01 | Ruling in the build order: method set, `Map[K, V]` representation (ordered by key for determinism; keys `Str`/`Int`/`Bool` in v1.x), WASM policy (lower or `DL1201` per method, stated) | reviewed | 0.5 S |
| P3-02 | `List`: `filter`, `fold`, `find`, `contains`, `sort`, `reverse`, `concat`, `is_empty`, `pop`, `slice`, `join`; the higher-order ones carry the callback's row under R-4 (the `map` precedent) | prim-table rows; two conformance witnesses each; `is_higher_order_method` regenerated from the table (RW 2.2) | 2 S |
| P3-03 | `Str`: `to_upper`, `to_lower`, `replace`, `join` (on `List[Str]`), `chars`? (ruling) | as above | 1 S |
| P3-04 | `Map[K, V]`: literal or constructor, `get` → `Option`, `insert`, `remove`, `len`, `keys`, `values`, iteration order deterministic; rcap defaults stated | witnesses; determinism test; Miri-shrunk bulk test | 2 S |
| P3-05 | Fuzz generator coverage: `delulu-fuzz` `danger.rs` emits the new higher-order methods with row-carrying callbacks (the C88 lesson: the grammar is the coverage) | a test asserting the generator produces each shape | 0.5 S |
| P3-06 | Docs: `primitives.md` regenerates; `GETTING_STARTED.md` §3 loses its "four methods" box; the Book Ch. 20 and `REMAINING_WORK.md` 2.1 updated; the core-invariance snapshot grows with new programs, never changes for old ones | `check-reference` gate; snapshot diff reviewed | 0.5 S |

**Acceptance.** `xs.filter(...)` compiles and runs; `Map` exists; coverage stays 100%; the
snapshot for existing programs is byte-identical (the tooling did not move the language).
Minor-version work; no RFC (additive, `STABILITY.md` §5).

## P4 — Agent surfaces

| Id | Task | Verification | Effort |
|---|---|---|---|
| P4-01 | **Skill**: `skills/delulu/SKILL.md` per the Agent Skills specification (name = folder, description with trigger keywords, body < 500 lines: the loop, the rules that trip agents — rows, `val`/`ref`, cap-relative paths, grant grammar, batching, the widening rule, exit codes — and `references/` pointing at `for-agents.md` and the reference); validated with `skills-ref validate` in CI (a Node step exists already for the editor) | a gate that the skill's command list matches `--help` (the completions precedent) | 1 S |
| P4-02 | `delulu toolchain --json`: version, commands and flags, the grant grammar, the ten effects, the prim table, limits (nesting 128, depth 10,000), engines and their fragments — **generated from the binary's own tables** | a gate that it agrees with `--help`, `primitives.md`, `broker.rs`'s grant kinds | 1 S |
| P4-03 | `delulu mcp`: stdio, MCP 2026-07-28 shape (stateless, `server/discover`, deterministic `tools/list`, `ttlMs`), tools = `check`, `authority`, `why`, `explain`, `atlas_query`, `toolchain`, and inside the source tree `survey_impact`/`survey_query`/`doctor_check` — **all read-only** (`readOnlyHint: true`), never `run`, never grants, never loads; hand-written JSON-RPC as the LSP is (no SDK dependency, D-NE-6) | protocol test like `lsp_cli.rs`; a test that the tool list contains no effector | 2 S |
| P4-04 | **Checked edits, step 1**: `delulu edit <file> --expect-hash <blake3> --edits <json>` (byte-range edits, applied back to front) → re-check → one envelope with the new hash, diagnostics and repairs; refuses on a hash mismatch (stale state) and never touches a morph-stored file (the `fix` precedent) | falsify: edit after the hash, expect refusal | 1 S |
| P4-05 | **Checked edits, step 2**: node-addressed edits by Atlas id (`replace_body fn:demo/demo.fib`), formatted through `format_source`, same envelope | witness on the corpus; a test that an id that no longer exists is refused | 1.5 S |
| P4-06 | `delulu-survey diff <git-ref>`: changed files → touched nodes → `impact` union, with citations (the `detect_changes` idea) | test against a synthetic diff | 1 S |
| P4-07 | Read-only Guard/broker view in the editor via `delulu guard status --json` (RW 6.5); hover shows *declared* and *performed* rows when they differ (NE-15) | `lsp_cli.rs` | 1 S |

**Acceptance.** A harness can load one skill, call one MCP server, and drive an
edit→check→repair loop with stale-state protection, all deterministic. **Owner decisions:**
D-NE-5, D-NE-6.

## P5 — Distribution

| Id | Task | Verification | Effort |
|---|---|---|---|
| P5-01 | `release.yml` on a `v*` tag: `scripts/package-toolchain.sh` on windows-x64, linux-x64, linux-arm64, macos-arm64 (the CI matrix already builds them), `SHA256SUMS`, GitHub artifact attestations (`actions/attest-build-provenance`), a draft GitHub Release with the archives; **publication target per D-NE-7** | a dry run on a `v1.0.1-test` tag on the testing repo if the owner allows; `gh attestation verify` recorded | 1.5 S |
| P5-02 | The archive ships `morphs/`, `skills/delulu/`, `examples/` (including `plugin_host` after P2), `INSTALL.txt` with the grant line from P1-08; the binary looks for morphs beside itself (`<bin>/../morphs`) as well as `./morphs` and `~/.delulu/morphs` | `distribution.rs` extended; an unpacked-archive test runs the morph | 0.5 S |
| P5-03 | Installer scripts (`install.sh`, `install.ps1`) that download the archive for the host target, verify `SHA256SUMS`, unpack to `~/.delulu/bin` and print the PATH line — **only if D-NE-8 accepts the posture**; otherwise a documented manual path | run on all three OSes in CI against the release assets | 1 S |
| P5-04 | Versioning and upgrade: `delulu --version --json` carries the build target and commit; upgrade = download the next archive; language editions (`STABILITY.md` §4) cover compatibility; a `CHANGELOG.md` "Released" section per tag | docs + a test that the version fields exist | 0.3 S |
| P5-05 | The five-minute page: `INSTALL.md` becomes download → verify → PATH → `delulu new` → `delulu test` → `delulu run` → `delulu authority --grants`, tested by `distribution.rs` against a real unpacked archive | gate | 0.5 S |

**Acceptance.** A person on any of the three OSes downloads one file, verifies it, and runs a
program in five minutes without Rust. **Owner decisions:** D-NE-7 (channel), D-NE-8 (installer).
Homebrew/winget/scoop manifests are prepared only after the final public repository exists.

## P6 — Documentation consolidation

Execute `DOCUMENTATION_AUDIT.md`: create `docs/archive/` mirroring original paths, move the
category-G and category-H files, update every inbound link (the Survey lists them), keep release
snapshots and entrenched paths untouched, regenerate the Survey, add an `archive/README.md`
index with the move manifest (original path, new path, reason, category, inbound references at
the time). Then shrink `HANDOFF.md` to a current briefing (§1, §2, §4, §7, §10, §11.1) with
pointers to the archive for its historical sections. **Effort:** 2–3 S. **Owner decision:** D-NE-11.

## P7 — Verification depth (cheap, already listed)

`RW 4.10a` wire the NIST KAT vectors (1 S); `RW 5.4` `cargo-fuzz` targets for the four parsers
and the grant parser (1 S); `RW 5.6` shrink the three Miri-slow tests under `cfg!(miri)` and
measure (1 S); `RW 5.2` restate Progress as progress-or-fault in `DELULU_CORE.md` (entrenched →
owner approval, recorded in `ENTRENCHED_CHANGE_RECORD.md`) (0.5 S). Each independent.

## P8 — Safe autonomy (owner-gated)

`RW 4.7` the signed Verified-class adapter — an adapter delivered as a `.dpx` (signature policy,
pinned key, verify-before-dispatch) instead of a bare subprocess; no hardware needed (3–5 S; a
ruling on the adapter protocol). Everything else here waits: a real device (`RW 7.6`), the microVM
guest launch (`RW 4.1`, Linux+KVM host), federation model-checking (`RW 5.5`).

---

## Dependencies

```
P1 ──► P2 ──► P4-03 (mcp exposes plugins) ──► P5-02 (archive ships plugin_host)
 │      └───► P2-06 example ─────────────────┘
 ├────► P3 (independent of P2)
 ├────► P4-01 skill (needs P1's grant line and rules) ──► P5-02
 ├────► P4-02 toolchain manifest (needs P1's envelope) ──► P4-03
 ├────► P4-04 edit step 1 ──► P4-05 edit step 2 (needs Atlas ids; P1-08 adds requested scopes)
 ├────► P6 (any time; best after P1 so the docs it moves are stable)
 └────► P5-01 workflow (independent; publication needs D-NE-7)
P7: independent of all.   P8: owner-gated.
```

## What can be done now (after approval, no owner decision needed)
P1 except P1-11/P1-13's candidates; P2-01…P2-08 once D-NE-10 is ruled (a build-order ruling is
enough if the owner delegates it); P3 entirely; P4-01, P4-02, P4-04, P4-05, P4-06, P4-07; P5-02,
P5-04; P6 with a folder name; P7 except the entrenched restatement.

## What requires major architecture changes
None of P1–P7 does. P2 is the largest change and it activates a *specified* mechanism through an
*existing* seam. The rejected alternatives (a graph store, a native tier, a second execution
model for plugins) would have been architecture changes and are not proposed.

## What requires owner decisions
D-NE-3 (snapshot regeneration review in P1-04/P1-08), D-NE-5, D-NE-6, D-NE-7, D-NE-8, D-NE-10,
D-NE-11, D-NE-12, D-NE-17; the entrenched edits in P7; anything in P8.

## What must remain deferred
The microVM layer, native backend and optimizer, principal types, `Secret[Bool]`, multi-tenancy,
federation model-checking, a physical device, certification, the final public repository.
