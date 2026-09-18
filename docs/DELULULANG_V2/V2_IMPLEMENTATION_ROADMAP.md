# V2 implementation roadmap — phases, tasks, verification, dependencies

**Status:** the maintained roadmap V2 executes, in the order the owner approved. Task ids keep their
V1 planning names (`P<n>-<nn>`, `PS-<x>-<nn>`) so the archive's evidence stays citable; tasks added
by the V2 commission carry new ids. Findings `NE-nn`, rows `RW n.m` and decisions `D-NE-nn` are in
the archive (`docs/archive/v1/NEXT_EVOLUTION_2026/`); `D-V2-nn` are in `V2_DECISION_LOG.md`. Effort
is in focused sessions (S), an estimate, kept from the plan. State per phase: `V2_PHASE_STATUS.md`.

---

## 0. The protocol every phase follows

**At the start:** read the phase's objectives here; inspect the affected code; `cargo run -p
delulu-survey -- impact <id>` for every module the phase touches; run the Survey and `doctor`;
establish the baseline (the suite green, the core-invariance snapshot unchanged, the numbers
recorded); write the sous-chef's brief; delegate most implementation to one Opus 5 agent; supervise;
verify every claim against the binary.

**Before declaring the phase complete (the verification gate):** focused tests, falsified; the
broader required tests; the Survey regenerated as the last edit; `doctor`; documentation links
verified; no duplicated file left behind by a move; `git diff` inspected; the V2 logs updated (the
entry is written before the Survey is regenerated); commit; push to the testing remote only; the
remote verified equal to the local head; the CI run read and its actual state recorded; only then
complete. A documentation-only change still needs the Survey and the documentation gates. A security
change needs security-specific tests. A sandbox change needs adversarial tests. A change touching
Authority or the Guard needs direct regression tests for the semantic invariants. A restriction-toggle
change needs explicit transition tests.

**Then STOP**, wait about sixty seconds for the owner, and continue to the next approved phase only
if nothing arrives (D-V2-06).

**Standing rules that bind every task:** harden, never redefine, Authority and the Guard; a gate
that cannot fail is not a gate — every new test is falsified; write the skip-branch case; witness every
fix against the pre-fix binary; the core-invariance snapshot is regenerated only deliberately, with
the diff shown in the phase's log (D-NE-3); no new dependency without a ruling; the banned word never
enters the tree; research sources are named only in the archive's research records.

---

## V2-0 — the V2 workspace and the documentation migration — COMPLETE (`e48f9c3`)

| Id | Task | Verification |
|---|---|---|
| V2-0-01 | create `docs/DELULULANG_V2/` with the ten V2 files | the Survey maps them; links resolve |
| V2-0-02 | move 32 historical, superseded and process documents into `docs/archive/v1/` preserving paths (`git mv`; nothing copied, nothing deleted); rewrite every explicit link the moves would break and the root-relative citations in active documents | `git status` shows 32 renames and no deletion; the Survey reports 0 errors |
| V2-0-03 | the Survey's archive-mirror rule: a prose citation of a pre-archive path resolves to `docs/archive/v1/<same path>` as a note; an explicit link stays an error; archived records are exempt from the present-tense count check | unit tests incl. a falsification; `cargo test -p delulu-survey`; clippy clean |
| V2-0-04 | update the two tests that hardcode a moved path; the archive README; the move manifest; `docs/REPOSITORY_STRUCTURE.md` accounting; `HANDOFF.md` and `README.md` pointers to V2 | `evidence_claims`, `governance`, `book`, `distribution`, `doctor_cli` green |
| V2-0-05 | commit, push, read the CI run, record it | `V2_EXECUTION_LOG.md` entry V2-0 |

No source file changes beyond the Survey rule and the two path strings. Nothing of the language moves.

## P1 — machine-contract truth (3–5 S)

Goal: every machine-facing promise in `docs/for-agents.md` true; the walls an agent hits in its first
program removed. No language-semantics change; one diagnostic-count change (P1-04).

| Id | Task | Finding | Verification |
|---|---|---|---|
| P1-01 | record NE-01 as a `REMAINING_WORK.md` row; correct the Book Ch. 10, `STAGE6_PLUGINS_GUIDE.md`, `examples/plugin_shout/README.md`, `README.md`, `HANDOFF.md` to say the load surface is a runtime stub until P2 | NE-01 | `book`, `evidence_claims` |
| P1-02 | the success-envelope sweep: a test that every `--json` subcommand on valid input emits `command`, `schema`, `delulu_version`, `diagnostics`, `summary`; wrap the five `why` emitters, `add`, `plugin inspect`, `test`; `atlas --format json` keeps `atlas/1` inside the envelope; `--version --json` emits the envelope | NE-05 | falsify by removing one field; `json_contract.rs` extended |
| P1-03 | `explain --json` → the envelope with `explain: {code, title, body, disposition, kind}`; an unknown option other than `--json` still refused | NE-06 | contract test + a `--bogus` refusal test |
| P1-04 | one `DL0210` per overflow site; the recovery placeholder is not type-checked as a call; same for `DL0211/12/13` | NE-04 | conformance unchanged in outcome; snapshot regenerated deliberately, diff in the log; falsify by re-enabling the cascade |
| P1-05 | a repair with `edits: []` carries `requires_human: true` and a reason; `fix` verdicts and `check --json` flags agree over the whole repair registry; LSP code actions without an edit are not preferred | NE-07 | falsify by clearing the flag on one repair |
| P1-06 | `test --json` carries a `diagnostics` array for `check-failed` and ceiling failures; nothing human-rendered on stderr under `--json` | NE-08 | contract test |
| P1-07 | `delulu test` with no path inside a package targets the package; outside one the refusal stays | NE-09 | `new_cli.rs` asserts bare `delulu test` passes in a scaffold |
| P1-08 | `authority --grants` (human) and `required_grants` + `requested_scopes` (JSON, additive) — the exact `--grant` flags plus the literal scope arguments, labelled *requested*; the Atlas carries `requested_scopes` | NE-10, NE-14 | a test that every grant kind the runtime parses is derivable; snapshot diff reviewed |
| P1-09 | `DL1603` for a `val` literal of `ref` contents: a message naming the element, an explanation section, a `safe` repair | NE-02 | witness the guide-shaped program; falsify by removing the repair |
| P1-10 | fix `examples/guide/05_capabilities.delulu` to cap-relative paths; state the rule in `explain E-DL0703`, `for-agents.md`, the Book; `examples_run.rs` asserts the guide's reads succeed | NE-03 | falsify by reintroducing the wrong path |
| P1-11 | *candidate, needs a ruling:* `[run-authority]` in `delulu.toml` bounded by `[authority]` | RESEARCH §6 | ceiling-law test |
| P1-12 | the `REPOSITORY_STRUCTURE.md` accounting gate, or reword its claim | NE-12 | falsify by adding an unlisted file |
| P1-13 | document that an effectful single-file test needs a package `[test-authority]`; *candidate:* `test --test-authority <row>` (D-NE-17) | NE-13 | ceiling test |

Acceptance: every `--json` success emits the envelope (gated); `explain` has a machine channel; the
200-deep program yields one `DL0210`; no repair claims applicability without edits; the scaffold's
`delulu test` passes bare; `authority` prints the grant line; the guide corpus reads its files;
`REMAINING_WORK.md` names NE-01; CI green on three OSes.

**Follow-ups recorded at P1's close (2026-09-18).** Each is a small brief of its own; none blocks
PS-0; when they run is the owner's decision.

| Id | Task | Source | Verification |
|---|---|---|---|
| P1-F1 | the DL0301 → DL0404 cascade: suppress DL0404 only when the callee's type is an inference variable born from an error (poison propagation) | D-V2-19; Entry P1, DECISIONS 3 | the cascade program yields DL0301 alone; `fn apply[F](f: F, x: Int) { f(x) }` still yields DL0404; conformance outcomes unchanged; mutant: suppress on every inference variable and watch the generic witness fail |
| P1-F2 | the `grants` and `guard` verbs print their `--json` successes inside the envelope; a success sweep that starts a broker in a temporary state directory drives them | the head chef's P1 verification | the sweep; `grants` and `guard` leave `NO_SUCCESS_SWEEP` |
| P1-F3 | a flag the shared option parser knows is refused by every command that does not own it (a per-command allowlist), as an unknown flag is today | Entry P1, PROBLEMS 3 | `check x.delulu --grants` and `check x.delulu --diff foo` exit 2; the refusal sweep covers every command against every shared flag |
| P1-F4 | `delulu test <package directory>` resolves the package with all its modules, as bare `delulu test` inside it now does | Entry P1, PROBLEMS 4 | `delulu test examples/greeter` passes; a test that reaches a package by path |

## PS-0 — sandbox truth, probes and the cheap hardenings (5–6 S)

| Id | Task | Finding | Verification |
|---|---|---|---|
| PS-0-01 | `REMAINING_WORK.md` rows for NE-17, NE-19/20, NE-21, NE-22; correct `README.md`, `GETTING_STARTED.md` §6 and the Book where `http.get` is presented as working; say that `--isolation process` isolates foreign code only, in `run --help` and the label | NE-17…22 | evidence and book gates |
| PS-0-02 | the run report (D-V2-21): `run --json --report-out <path>` writes an envelope with an additive `sandbox` object even at L0, `{backend:"inproc", level:0, requested, granted, host_guarantees:[], limits:null, mode:"strict", break_glass:false}`, both when the program ran and when the run was refused; the program's standard output is untouched; a `<path>` inside a scope the program may write is refused before the run | NE-16b | the envelope sweep drives `run` through `--report-out` and `run` leaves `NO_SUCCESS_SWEEP`; a refusal witness for a report path inside an `fs.write` scope; a program that prints a counterfeit `sandbox` object changes nothing in the report |
| PS-0-03 | `DL1408` gains its promised repair (`requires_human: true`, `edits: []`, the fallback command) | NE-16c | coverage gate; the P1-05 rule |
| PS-0-04 | `delulu sandbox probe [--json]`: per level, present/absent with the first missing prerequisite; every line an attempt, never a version string | — | asserted on CI per OS against the measured facts; a mutant probe that reports "present" without attempting must fail |
| PS-0-05 | the `doctor` sandbox section built on PS-0-04: backends, level available, KVM, OS primitives, network and filesystem enforcement, identity separation, resource controls, the active profile, break-glass status, any intentionally relaxed restriction — only lines that change a decision | commission §24 | `doctor_cli.rs` |
| PS-0-06 | primitive-table hardening: refuse Windows reserved device names, trailing dots and spaces, drive-relative spellings, verbatim and device prefixes; record the resolved name in the trace and audit; on POSIX refuse embedded NUL and names that differ after normalization | NE-19, NE-20 (D-NE-29) | characterization tests C-01/02/11 flip red then are rewritten as refusal witnesses; snapshot unchanged for the corpus |
| PS-0-07 | IPC-1's read deadline on `WorkerConn`; a `WorkerDied`-class result on timeout; the worker killed | NE-21 (D-NE-30) | C-05 flips; falsify by removing the timeout |
| PS-0-08 | CI experiments (the dispatch-only workflow): can the runner open `/dev/kvm` and boot a fetched VMM to `/init`; a differential Seatbelt probe; a restricted-token + Job Object child on Windows | — | results read with `gh run view` and transcribed |
| PS-0-09 | refuse or warn on special-use addresses in `--grant net=` unless spelled explicitly — **owner ruling on the spelling (D-NE-28)** | NE-18 | C-04 flips |

Acceptance: every isolation statement in the shipped documents is true; `sandbox probe --json` and
`doctor` report the host; the four runtime hardenings have witnesses; CI knows whether L2 can run
there.

## PS-A — L1: the process jail with the effect channel, and the execution modes (12–17 S)

Goal: `delulu run app.delulu --sandbox` runs the whole program as a guest holding no OS authority on
all three operating systems; the host performs every effect under today's checks; STRICT and AUDIT
modes exist with their transitions tested.

| Id | Task | Verification |
|---|---|---|
| PS-A-01 | the `EffectSink` seam: `LocalSink` is today's path, byte-identical; `ChannelSink` serializes and blocks on the reply; capabilities in guest mode are opaque handles; `narrow` returns a handle from the host | snapshot identical; `delulu-fuzz` local mode unchanged; a guest-mode fuzz run asserts trace ⊆ row and one audit record per performed effect |
| PS-A-02 | the channel protocol `delulu-sandbox-channel/1`: length-prefixed canonical CBOR over the `broker_ipc` framing with a hard per-frame bound; request kinds mirroring the primitive table; the host dispatcher; a `cargo-fuzz` target from day one; read deadlines both ways | the fuzz target green; property tests; the IPC-1 test |
| PS-A-03 | the guest runtime mode (hidden `__guest`): connect, receive program bytes + hash + grant + policy + seed/clock, check the hash, run the interpreter with `ChannelSink`, report over the channel; no filesystem, no environment, no argv beyond the handle | a guest with a stubbed host refuses to start without the hello; the memory/env/argv scan (T1) |
| PS-A-04 | the launchers — Linux: Landlock ruleset, seccomp allowlist, rlimits, `PR_SET_PDEATHSIG`, a user cgroup where the probe says it works, namespaces only where usable; Windows: restricted token, Job Object (kill-on-close, one active process, memory, process time, UI limits, no breakaway), AppContainer where creatable; macOS: a generated Seatbelt profile | the boundary families per OS; **mutant launchers** each failing their tests; the P21 vectors from inside the guest where identity separation applies |
| PS-A-05 | environment and secrets hygiene for every child the host spawns: empty environment plus an allowlist; secrets never on argv for guests; stdin closed | C-09 flips |
| PS-A-06 | `derive(authority, grant, profile) → SandboxPolicy`, pure, JSON-stable, hashable, diffable, explainable (`sandbox policy <file> --json`); profiles `dev`, `contained`, `hostile-agent`, `external:<name>`; `[sandbox]` in `delulu.toml` bounded by `[authority]` | policy tests; a snapshot for the corpus; the ceiling law |
| PS-A-07 | the CLI and machine surface: `--sandbox`, `--sandbox=off`, `--sandbox-profile`, `--limits`; the `sandbox` object with requested and actual level, backend, host capabilities, guarantees, limitations, limits and remaining, network/filesystem/identity posture, state, fully-enforced flag, break-glass flag, `denied[]`; `sandbox status/kill`; `explain E-SANDBOX`; new DL codes with both witnesses; `--isolation process` strengthened and announced | the envelope sweep; the CLI sweep; contract tests |
| PS-A-08 | audit lifecycle records: `sandbox-launch` (level, guarantees, policy hash, image hash), `sandbox-limit`, `sandbox-kill`, `sandbox-death`, `channel-violation`; the guest id on every brokered effect | audit tests; `audit verify` unchanged |
| PS-A-09 | documentation: `DEPLOYMENT.md` Tier 2 made mechanical per OS; `for-agents.md` and the skill gain the profile advice; the Book Ch. 15; `MATHEMATICS.md` §12 gains the claims with their categories | evidence and book gates |
| PS-A-10 | **execution modes STRICT and AUDIT (dry-run)**: AUDIT performs no effects and reports the required authority, the would-be requests and the derived policy; mode visible in `run --json`, `doctor` and the audit chain; the transition matrix of `V2_SECURITY_MODEL.md` §5 as tests — ON→OFF, OFF→ON, strict→audit, audit→strict, failed and unsupported transitions, a transition during execution, after a delegation, after a plugin load, during an escape attempt, a child inheriting a weaker policy, a revoked authority after a transition | each transition a witness plus a mutant; a program that tries to relax its own sandbox or authority is refused with a code and audited |

Acceptance: `delulu run --sandbox` works on all three CI runners for the guide corpus with
byte-identical output versus L0; every boundary family has witnesses on each OS and every mutant
fails its tests; the P21 vectors are refused from inside the guest on Linux; the launch and round-trip
costs are measured and recorded. **Owner:** D-NE-24, D-NE-26, D-NE-31, D-NE-33/D-V2-13, D-NE-25 if
`Secret.map` is met (refused under `hostile-agent` by default).

## P2 — plugins for real (6–10 S)

| Id | Task | Verification |
|---|---|---|
| P2-01 | the design note (a V2 build-order section in `V2_DECISION_LOG.md`): the run-time load path through `PluginEngine`, the grant source (D-NE-10), the broker holder check as a child node, unload → revoke, the Windows Contained refusal retained, `verify ≡ load` retained; for sandboxed programs the load runs host-side | reviewed before code |
| P2-02 | the loading grant: `--grant plugin=<path-or-dir>` and/or a `[plugins]` manifest ceiling by hash, per the ruling; paths through the containment resolver so `..`, symlinks and case cannot widen; `DL0703` names the flag | parser tests; the skip-branch case |
| P2-03 | wire `root.plugin_host()` → `PluginHost`; `load(host, path, grant)` runs steps 1–7 of the Stage 6 load sequence via the `delulu-wasm` engine; `p.get(name)` returns a callable with the re-verified row (Verified) or `effects(grant)` (Contained) | criteria 1, 2, 3, 7 of `STAGE6_BUILD_ORDER.md` §4 re-witnessed at the CLI level; DL1502/0802/1504/1505/1509/1510/1511 each provoked from a program |
| P2-04 | the holder check and revocation: a child grant node under the program's node in embedded and daemon custody; `unload` and `grants revoke` kill it; a revoked export faults with `PluginErr::Revoked` | a `guard_e2e`-style test; falsify by skipping the node |
| P2-05 | fuel, memory and wall limits from `Grant.limits` on the live engine; the Windows refusal unchanged | the criterion-5 witnesses extended to the CLI path |
| P2-06 | an example package `plugin_host` under `examples/`, gated by `examples_run.rs`; the Book Ch. 10 and the guide updated | the gate asserts the plugin's output |
| P2-07 | the loaded node id in the audit record and `--trace-effects` | trace test |
| P2-08 | red team the path: path spelling, a `.dpx` replaced between `verify` and `load`, a grant wider than the ceiling, Contained on Windows, a plugin loading a plugin | adversarial tests, each witnessed failing on the unpatched path |

Acceptance: the host example runs the plugin under `--grant console --grant plugin=…`; every refusal
code is reachable from a program; revocation kills a loaded plugin; the `REMAINING_WORK.md` row
closes with what closed it. The sentence "an untrusted plugin cannot overreach on your own machine"
is published only after PS-A's witnesses are green. **Owner:** D-NE-10.

## P4a — the Agent Skill (1 S)

| Id | Task | Verification |
|---|---|---|
| P4-01 | `skills/delulu/SKILL.md` in the Agent Skills format (name = folder; description with triggers; body under 500 lines: the loop, the rules that trip agents — rows, `val`/`ref`, cap-relative paths, the grant grammar, batching, the widening rule, exit codes, the sandbox profiles and what the `sandbox` object means; `references/` pointing at `for-agents.md` and the reference); `delulu skill` prints it; validated with the reference validator in CI | a gate that the skill's command list matches `--help`; the skill shipped in the archive (P5-02) |

**Owner:** D-NE-5 (folder name).

## P3 — the standard library, additively (5–8 S)

| Id | Task | Verification |
|---|---|---|
| P3-01 | the ruling (in `V2_DECISION_LOG.md`): the method set; `Map[K, V]` ordered by key, keys `Str`/`Int`/`Bool` in 1.x; the WASM policy per method | reviewed |
| P3-02 | `List`: `filter`, `fold`, `find`, `contains`, `sort`, `reverse`, `concat`, `is_empty`, `pop`, `slice`, `join`; higher-order ones carry the callback's row (R-4) | prim-table rows; two witnesses each; `is_higher_order_method` regenerated from the table |
| P3-03 | `Str`: `to_upper`, `to_lower`, `replace`, `join`, `chars` per the ruling | as above |
| P3-04 | `Map[K, V]`: constructor, `get` → `Option`, `insert`, `remove`, `len`, `keys`, `values`, deterministic iteration | witnesses; a determinism test; a Miri-shrunk bulk test |
| P3-05 | `delulu-fuzz` generates the new higher-order shapes (the C88 lesson) | a test that the generator produces each shape |
| P3-06 | `primitives.md` regenerated; `GETTING_STARTED.md` §3, the Book Ch. 20, `REMAINING_WORK.md` 2.1 updated; the snapshot grows, never changes for old programs | `check-reference`; the snapshot diff reviewed |

## PS-B — limits as authority, the egress proxy, identity, BREAK-GLASS (6–8 S)

| Id | Task | Verification |
|---|---|---|
| PS-B-01 | resource limits for the main program on every engine: an interpreter step/allocation budget, wall clock in the host, memory/pid/CPU through the launcher's OS controls; the `limits.rs` attribution rule re-used so a limit kill never suggests widening | the resource family; a mutant without the budget |
| PS-B-02 | the egress proxy = the first network client: host-side, serves `http.get` for guests and L0 alike, allowlist by name, resolve once and pin, special-use ranges refused unless granted by their own spelling, SNI/Host agreement, redirects re-checked, response size bounded, no resolver for guests — **owner ruling on the TLS dependency (D-NE-28)** | the network family; C-03/C-04 flip |
| PS-B-03 | identity separation where feasible: an AppContainer profile per run (Windows), uid mapping where namespaces allow (Linux), a documented recipe (macOS); reported in `host_guarantees` | T14 |
| PS-B-04 | channel batching for epoch-class effects after PS-A's measurement says it pays | the measurement record |
| PS-B-05 | **resource authority formalized**: budgets in the derived policy, the run envelope, the authority report where static, the audit record; the dimensions that admit a containment order join the Z3 model and the enumeration and are documented in `MATHEMATICS.md`; the rest stay explicit launcher controls labelled as such (D-V2-08) | the order laws extended; a widening in a budget dimension refused |
| PS-B-06 | **BREAK-GLASS** as an external operator control: a separate trust boundary (an operator-held credential outside the guest), a banner, a `doctor` line, an audit record, the `--json` field; never activatable by program code; its transitions from the matrix tested | witness plus mutant; a program attempting it is refused and audited |

## P4b–e — agent surfaces, MCP, checked edits, tooling, the benchmark (7–10 S)

| Id | Task | Verification |
|---|---|---|
| P4-02 | `delulu toolchain --json` generated from the binary's tables: version, commands and flags, the grant grammar, the ten effects, the primitive table, limits, engines and fragments, the sandbox levels and profiles | a gate that it agrees with `--help`, `primitives.md`, the grant kinds |
| P4-09 | `delulu schema <name> --json`: the JSON shapes of the envelope, diagnostics, repairs, the authority report, `atlas/1`, the `sandbox` object and `SandboxPolicy`, generated from the emitters | a test that every emitter's output validates against its schema |
| P4-10 | `delulu examples --json`: the shipped, gated examples with their authority report and the grant line that runs each | a gate that every listed example checks and runs |
| P4-03 | `delulu mcp`: stdio, the current MCP shape (stateless, deterministic `tools/list`), tools `check`, `authority`, `why`, `explain`, `atlas_query`, `toolchain`, `schema`, `sandbox_probe`, and inside the source tree `survey_impact`/`survey_query`/`doctor_check` — all read-only, never `run`, never grants, never loads; hand-written JSON-RPC (D-NE-6) | a protocol test like `lsp_cli.rs`; a test that the tool list contains no effector |
| P4-04 | checked edits, step 1: `delulu edit <file> --expect-hash <blake3> --edits <json>` → re-check → one envelope with the new hash; refuses on a hash mismatch; never a morph-stored file | falsify: edit after the hash |
| P4-05 | checked edits, step 2: node-addressed edits by Atlas id, formatted through `format_source` | a witness on the corpus; a stale id refused |
| P4-06 | `delulu-survey diff <git-ref>`: changed files → nodes → `impact` union with citations | a synthetic diff |
| P4-07 | the read-only Guard/broker view (`guard status --json`) in the editor; hover shows declared and performed rows when they differ | `lsp_cli.rs` |
| P4-11 | the Atlas gains the V2 chain — program → authority → effects → capabilities → sandbox policy → resources → plugins → actors → devices → execution boundary — as an additive `atlas/1` view an auditor can query structurally; the Atlas is not turned into the Survey | a snapshot on the corpus |
| P4-08 | **the AI usability benchmark** (`V2_AI_NATIVE_DESIGN.md` §4): five conditions, seven measures, generated tasks, a record `ai-usability` under `measurements/`, its first result published whatever it says | the record's negative controls; replayable runs |

**Owner:** D-NE-6.

## PS-C — L2: the microVM on Linux/KVM (8–10 S)

| Id | Task | Verification |
|---|---|---|
| PS-C-01 | the ruling recorded (D-NE-23 ruled; no virtio-fs, no NIC, vsock only; Firecracker first under its jailer, Cloud Hypervisor second) — and **every practical prerequisite verified against the code first**: the `EffectSink` seam, the channel, a static Python-less guest build, the image build on a Linux runner; a missing prerequisite stops the phase | reviewed; a prerequisite checklist with commands |
| PS-C-02 | the guest image: a pinned kernel with virtio-vsock and nothing else; an initramfs with the static `delulu` as `/init` in `__guest` mode; hashes in a manifest; the build script named in a code comment for the Survey; reproducibility checked twice | hashes match across two builds |
| PS-C-03 | the `microvm` launcher: the VMM driven over its Unix-socket API (a hand-written HTTP/1.1 client), a jailer per VM with a unique uid, the vsock channel, a wall watchdog, VMM-death handling, orphan sweep, jail cleanup, verified-dead semantics | the lifecycle family |
| PS-C-04 | criterion 8 un-gated on a KVM runner, restated for the no-NIC guest (T9) | CI green on the KVM job |
| PS-C-05 | the image as a separate, checksummed, attested artifact (P5); `doctor`/`probe` report presence and integrity | P5 gates |
| PS-C-06 | red team L2; a published record including what was not attempted | the record |

**Owner:** D-NE-27 (a GPL kernel is a licensing act).

## P6 — documentation consolidation (1–2 S; the archive half was V2-0)

Shrink `HANDOFF.md` to a current briefing (§1, §2, §4, §7, §10, §11.1) with pointers to the archive
for its historical sections; bring the Book and `README.md` to V2's state; keep release snapshots and
entrenched paths untouched; regenerate the Survey.

## P5 — distribution (3–5 S)

| Id | Task | Verification |
|---|---|---|
| P5-01 | `release.yml` on a `v*` tag: `scripts/package-toolchain.sh` on windows-x64, linux-x64, linux-arm64, macos-arm64; `SHA256SUMS`; GitHub artifact attestations; a draft release — **publication target per D-NE-7** | a dry run on a test tag if the owner allows; `gh attestation verify` recorded |
| P5-02 | the archive ships `morphs/`, `skills/delulu/`, `examples/` (with `plugin_host`), the guest mode, `INSTALL.txt` with the grant line; the binary looks for morphs beside itself | `distribution.rs` extended |
| P5-03 | installer scripts only if D-NE-8 accepts the posture; otherwise a documented manual path | run on all three OSes against the release assets |
| P5-04 | `--version --json` carries the build target and commit; a `CHANGELOG.md` "Released" section per tag | a test that the fields exist |
| P5-05 | `INSTALL.md` becomes download → verify → PATH → `delulu new` → `test` → `run` → `authority --grants`, tested against a real unpacked archive | the gate |

**Owner:** D-NE-7, D-NE-8.

## P7 — verification depth (3–4 S; each independent)

RW 4.10a wire the NIST KAT vectors; RW 5.4 `cargo-fuzz` targets for the four parsers and the grant
parser (shared with PS-A's channel target); RW 5.6 shrink the three Miri-slow tests under
`cfg!(miri)` and measure; RW 5.2 restate Progress as progress-or-fault in `DELULU_CORE.md`
(entrenched → owner approval, recorded in `ENTRENCHED_CHANGE_RECORD.md`).

## PS-D — external launchers and the attestation seam (3–4 S)

PS-D-01 `--sandbox-backend external:<cmd>`: the operator's launcher runs `delulu __guest` in their
environment and exposes the channel over stdio; the level is labelled `external`, the guarantees
`unknown`; a Docker + `runsc` + `--network none` recipe documented, not shipped. PS-D-02 the
attestation seam: a `guarantees` field a launcher may populate from an attestation document and a
"refuse to delegate unless attested" policy hook — designed and tested with a fake attester, not
integrated with hardware.

## P8 — safe autonomy (owner-gated)

RW 4.7 the signed Verified-class adapter delivered as a `.dpx` (signature policy, a pinned key,
verify-before-dispatch); the control program runs in a guest, the adapter and the dead-man watchdog
stay host-side. A real device, federation model-checking and everything else here wait for hardware
or the owner.

---

## Dependencies

```
V2-0 ─► P1 ─► PS-0 ─► PS-A ─► P2 (claims; P2's code needs only P1)
                        │
                        ├─► P4a (skill teaches the profiles) ─► P3 (independent of P2)
                        │
                        └─► PS-B ─► P4b–e (MCP exposes sandbox_probe; the benchmark needs P4a–d)
                                 │
                                 └─► PS-C ─► P6 ─► P5 (the image artifact; the skill and morphs in the archive)
P7: independent of all (its fuzz scaffolding is shared with PS-A).   PS-D after PS-A.   P8 owner-gated.
```

**What must remain deferred, with the trigger that re-opens each** (archive:
`SANDBOX_IMPLEMENTATION_PLAN.md` §6): snapshots and warm pools; Windows and macOS microVMs; a
DeluluLang-owned container runtime; guest-side foreign libraries; the native backend and optimizer;
principal types; `Secret[Bool]`; multi-tenancy on one OS user (never); certification; the final public
repository (the owner's step).
