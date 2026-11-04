# DeluluLang: independent strategic review

Research date: 4 November 2026. Repository: `D:\nelan\DeluluLang`. Inspected commit: `e97c74798fed941a369bbc931ed9d0a8c462753c` on `master`. This is a decision document, not an implementation or release record. The naming assessment in §18 includes the subsequent requests about KingLang and alternative names.


> **Follow-up, 4 November 2026:** At the owner's explicit request, this report was checked against the original prompt, expanded with §7.1–7.3 and Appendix F, and copied to `D:/nelan/DeluluLang/docs/review/DeluluLang_Codex_Strategic_Review.md`. The external original is retained with identical contents. Earlier read-only/integrity statements describe the original research session; this authorized follow-up adds the report inside the checkout. No recommendations were implemented. Coverage does not mean that every proposed experiment or deployment has been verified.

## 1. Executive summary and recommendation

**DeluluLang should remain a programming language and become a language plus an agent-focused toolchain.** Keep the parser, checker, compiler, runtime, Authority, Guard, containment and verification machinery. Make the CLI and its versioned machine interfaces the primary product. Keep an editor-independent language server and a small optional VS Code client. Pause expansion of the editor and speculative autonomy surfaces while completing ordinary agent programs.

The strongest product proposition is: **an agent can construct a useful program, explain its required authority, submit it to a bounded execution host, and inspect a machine-readable outcome.** This is narrower and more testable than “the best language for all AI,” but does not abandon the long-term vision. Robots, vehicles, drones and spacecraft need this kind of separation too; they also need substantial guarantees the language does not supply.

The project has built much more than its stale installed binary suggests. A fresh release build ran records, generic functions, recursion, lists, maps, errors, package imports, an authority-sensitive plugin and useful bounded mock workflows. It ran a simulated actuator through a Windows AppContainer guest, accepted a permitted command, rejected an out-of-envelope command, and read back the permitted position. The current commit also has a newly completed **18/18 successful CI run**. This is substantial implementation and assurance work worth preserving.

However, the concern about fundamentals is justified. On that same fresh build:

- `root.args()`, `parse_json`, `sqrt`, user-defined method notation and `+=` were refused; ordinary interpolation printed its braces. Some are missing library facilities, some are syntax or host-interface gaps, and none is repaired by more advanced orchestration terminology.
- The same basic program using `range` ran in the interpreter but failed WASM code generation. Replacing that loop with `while` made the rest of the program run on both engines. Device APIs were also rejected by WASM, although they worked in the interpreter and process sandbox.
- A nonexistent test path in a valid package returned success with zero tests. A package entry file with an import failed when invoked as a file, although running the package directory worked.
- The installed `delulu` ignored `--version --json` and returned only `delulu 1.0.0`; the fresh binary returned its target and injected commit. Installation identity and update detection are immediate adoption problems.
- Authority analysis can correctly answer **inconclusive** for a computed filesystem scope even when runtime enforcement can execute the program safely. An agent must not turn this into automatic approval.

My priority order is: **trustworthy installation and outcome contracts → everyday task completion → an explicit backend/containment support matrix → measured machine workflows → selective size/performance work → specialist integrations.** Continue the existing V2 effort, but hold its broad public-readiness claim to a small task-completion release bar. Do not restart the language or discard its security evidence merely because the repository is large.

For the name, **do not switch to KingLang on the current evidence**: there is already a Rust-written interpreted language with that exact repository name, and a Chinese patent publication also describes a software language called KingLang. DeluluLang is not established to be unlawful or unusable, but relevant DELULU software/AI trademark records make clearance necessary. Keep it as the working name while considering a third name; §18 gives the evidence and a small shortlist. No rename was performed.

## 2. Research scope, provenance and limits

### 2.1 What was read and how evidence is classified

The supplied Markdown prompt and pasted attachment were read from beginning to end and compared; their substantive content matches after normalization. The prompt is 267 lines and 46,279 decoded characters. Its SHA-256 is `052A54ED9FC1133F338CE84E9D2CDAA1EDB16263C9DFF83CC093DD80DAC45D44`; the pasted attachment's SHA-256 is `3B8A4EDF534EA7935BD77D283D2D2A6E63E1B48748ABC36275EE3794CF9260C1`.

The onboarding sequence, full `D:/nelan/DeluluLang/HANDOFF.md`, full `REMAINING_WORK.md`, README, installation/contribution guides, `D:/nelan/DeluluLang/docs/for-agents.md`, constitution/core/soundness/stability documents, deployment/editor material, P8/P9 design, the OpenShell study, prior agent-loop and study-C records, relevant assistant-memory files, current phase status and recent handoff/log entries were examined. Source inspection covered the workspace manifests, CLI/front doors, syntax/checker/primitive table, interpreter and WASM paths, host/guest boundary, plugins, MCP/LSP and relevant tests/examples. Historical logs were used selectively; this is not a claim to have read every line of every source file or every historical conversation in the repository. No private model reasoning was collected or reproduced.

Evidence labels used throughout:

| Label | Meaning |
|---|---|
| **Observed** | A command or wire interaction executed in this session, with captured output and exit status. |
| **Source** | Behavior or structure read in the inspected tree; not necessarily exercised locally. |
| **CI** | A remote run's actual result was read. It establishes what those jobs ran on that commit, not every conceivable property. |
| **Recorded** | Earlier project evidence or a decision; not independently repeated here. |
| **Recommendation** | A proposed change or acceptance condition; nothing has been implemented. |
| **External** | Dated primary-source research; competitors were not installed or benchmarked. |

### 2.2 Repository state and read-only boundary

The initial state was `master`, tracking `origin/master`, zero ahead/behind, with the pre-existing untracked `D:/nelan/DeluluLang/docs/prompt` directory. The tracked and staged diffs were empty. The origin remains the owner's public **test** repository, `https://github.com/jessesuniljs1-collab/delululang-test.git`. The prepared `launch/2.0.0` tree is not a public launch.

**Disclosed deviation:** before the full research-only prompt had been read, the older onboarding instructions caused one `git fetch origin --prune`. No branch/ref change was observed; fetch metadata may have been refreshed. Therefore this session must not be described as having made absolutely no Git-metadata writes. After the research constraint was understood, there was no pull, further fetch, checkout, commit, push, tag, workflow dispatch, dependency installation, repository repair or configuration change.

All build outputs, temporary programs, reports and experiment scripts are outside the checkout, under:

`C:\Users\jesse\AppData\Local\Temp\delulu-codex-review-20261104-75e41c98`

The only requested deliverable is this Markdown file outside the repository. No sub-agents were used. Provider billing/usage totals were not available and are not estimated. The final integrity appendix records the tracked-file hash comparison and remote-head check.

### 2.3 Build and machine

Observed environment: Windows 11 Home Single Language, build `10.0.26200`; AMD Ryzen 7 7840HS, 8 cores/16 logical processors; approximately 15.3 GiB visible RAM. Rust `1.96.1 (31fca3adb 2026-06-26)`, Cargo `1.96.1`, Python `3.13.5`.

The exact fresh build was:

```powershell
$env:CARGO_TARGET_DIR='C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/target'
$env:DELULU_BUILD_COMMIT='e97c74798fed941a369bbc931ed9d0a8c462753c'
$env:CARGO_INCREMENTAL='0'
cargo build --locked --offline --release -p delulu -p delulu-survey --no-default-features --features delulu/net -j 1
```

It completed successfully in Cargo's reported **21 min 06 sec**, using cached dependency sources and a fresh external target directory. One job was chosen for laptop memory pressure; this is not a parallel-build performance comparison. The resulting `delulu.exe` is **35,451,904 bytes**, SHA-256 `23508EA55DC61330AECC91EBA119C4002AA3FEA33728795CE638E98373A25E32`. Its JSON identity reports version `1.0.0`, the inspected commit and `x86_64-pc-windows-msvc`.

This is the **net-enabled, Python-free portable profile**. It does not establish that the default embedded-Python build works locally. It does establish that a complete fresh binary can be built offline from the available dependency cache. Nothing was downloaded or installed by Cargo.

### 2.4 Health and CI

The fresh Survey check and `doctor --check --json` both failed the checkout's map-freshness requirement. Findings separately reported **0 errors, 6 warnings and 37 notes**, with 1,616 nodes and 18,651 edges. These are different results: zero finding errors does not make a stale map fresh. No map was regenerated.

The Survey scans the filesystem, including the pre-existing untracked prompt; its query returned a node for that 267-line file. This is a plausible contributor to the freshness failure, not proof that committed `master` alone has a broken map. A clean tracked-tree reproduction was not performed. See [scan.rs](D:/nelan/DeluluLang/crates/delulu-survey/src/scan.rs:93).

| Run read | Commit / purpose | Actual conclusion |
|---|---|---|
| [41813603704](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41813603704) | Inspected `e97c747`, scheduled CI, 4 Nov | **Success, 18/18 jobs**, including all three OS test jobs, ARM64, formal, fuzz, Miri, supply chain, editor and microVM jobs. Completed update time `2026-11-04T12:09:32Z`. |
| [41619070519](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41619070519) | Same SHA, 3 Nov | **Failure**. Ubuntu runner annotation reported `System.IO.IOException: No space left on device` writing its diagnostic log; 17 other jobs succeeded. This run is not relabeled green. |
| [41459195468](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41459195468) | Same SHA, push CI | Success. |
| [41455545104](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41455545104) | Same SHA, PR CI | Success. |
| [41416143488](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41416143488) | Prepared launch `728a582…` | Success, 18/18. This is a different tree from the reviewed head. |
| [41416147296](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41416147296) | Prepared launch dry run | Success; a dry run, not publication. |

No full local workspace test/clippy/formal/Miri campaign was run. The local evidence is the release build and the targeted probes below. WSL, macOS, Linux, KVM, a live VS Code activation, real network services, actual devices, physical fail-states and aerospace certification were not locally validated. The protocol tests did launch the real LSP and MCP servers.

## 3. Current architecture and feature inventory

### 3.1 What the product actually is

There is not an independent obsolete compiler product that can simply be removed while leaving today's CLI intact. The shipped `delulu` binary **is already the main front door to the language implementation**. Its Cargo manifest declares one binary and depends directly on syntax, checking, runtime, WASM, Atlas, broker and Survey crates. The VS Code extension starts that binary with `lsp`; it does not contain a separate compiler. See [CLI manifest](D:/nelan/DeluluLang/crates/delulu/Cargo.toml:24) and [extension launcher](D:/nelan/DeluluLang/editors/vscode/extension.js:273).

```text
AI client / shell / optional editor
          │
          ├─ CLI JSON, checked edits, readonly MCP, persistent LSP
          │                    │
          └──────────── syntax → resolution → types/effects/reference capabilities
                                               │
                          Authority / Atlas ←──┤
                                               ├─ interpreter
                                               └─ WASM codegen / .dwx execution
                                                        │
                            runtime primitives / host adapters
                                                        │
                          grants + leases + Guard + audit + budgets
                                                        │
                             permitted host effect / simulator

Optional process / microVM / external boundary:
  guest holds handles; host performs carried effects through a checked channel.

Survey maps the repository; doctor checks installation/environment/posture/map.
```

This diagram is architectural, not a claim that every backend supports every primitive or every deployment boundary. FFI exists in the WASM host path too; “WASM means no native interaction” would be an incorrect simplification. The compiler explicitly lowers foreign operations to host imports in [codegen.rs](D:/nelan/DeluluLang/crates/delulu-wasm/src/codegen.rs:30).

### 3.2 Size and responsibilities

The tracked inventory contains **1,158 files**: 340 Rust files with 199,342 newline-counted lines; 306 Markdown files with 95,737 lines; 258 `.delulu` files with 5,270 lines; and 69 JSON files with 30,148 lines. Generated inventories, tests, specifications and historical evidence account for much of the size. Line count alone is not a removal criterion.

| Workspace crate | Rust source lines, including inline tests | Integration-test lines | Main responsibility |
|---|---:|---:|---|
| `delulu` | 48,011 | 43,565 | CLI, packages, deployment, protocols, process boundary, operator surfaces |
| `delulu-syntax` | 7,632 | 0 | Lexer, AST, parsing and formatting infrastructure |
| `delulu-check` | 17,196 | 1,004 | Resolution, types/effects, authority, reference capabilities, manifests |
| `delulu-runtime` | 24,891 | 1,900 | Interpreter, values, primitives, interop, devices, effect implementations |
| `delulu-wasm` | 14,100 | 4,900 | WASM compiler, host ABI, artifacts and engine parity |
| `delulu-broker` | 14,548 | 1,306 | Authority lattice, leases, Guard, secret custody, revocation and audit |
| `delulu-diag` | 4,032 | 99 | Diagnostics, envelopes and repairs |
| `delulu-atlas` | 2,294 | 0 | Typed program relationships and authority views |
| `delulu-registry` | 1,579 | 330 | Registry/package distribution support |
| `delulu-survey` | 4,714 | 1,060 | Repository inventory and evidence-linked dependency queries |
| `delulu-conform` | 2,171 | 0 | Conformance reference and witness enforcement |
| `delulu-fuzz` | 798 | 0 | Fuzz support |
| `delulu-measure` | 2,982 | 156 | Measurement and agent-usability evaluation tooling |

Largest individual Rust files include `cli.rs` (11,866 lines), WASM `codegen.rs` (6,510), `brokerd.rs` (5,313), checker `check.rs` (4,757), WASM `host.rs` (3,413), `guest.rs` (3,086), parser (3,012), interpreter (2,979), `run_cmd.rs` (2,236), runtime plugin code (2,222) and primitive code (2,190). These indicate review and change coordination costs, not necessarily runtime bloat.

The command surface spans authoring (`check`, `fmt`, `edit`, `D:/nelan/DeluluLang/examples`, `explain`), execution (`run`, `repl`, `build`, `test`), analysis (`authority`, `atlas`), discovery (`toolchain`, `schema`, `doctor`), integrations (`mcp`, `lsp`), package/plugin operations, and privileged operations such as broker/grants/Guard/audit/sandbox/deployment/device commands. They should be presented as a few workflows with deeper operator commands discoverable, rather than one undifferentiated list.

### 3.3 Why complexity accumulated

Five drivers are justified: a statically checked language; two execution paths; host mediation across several operating systems; revocable/delegable authority; and evidence strong enough to catch security regressions. Removing any one is a product decision with a real guarantee or portability cost.

Five drivers deserve consolidation:

1. **Repeated semantic traversal.** Checker, authority analysis, reference-capability checking, interpreter and code generator each understand AST shapes. Historical fixes around shadowing, assignment targets and callbacks show the cost of keeping these interpretations aligned. A shared resolved/checked representation is a direction to investigate incrementally, not a reason for a rewrite.
2. **CLI concentration.** Parsing, orchestration, output contracts and operator policy accumulate in large files. Extract reusable services with structured inputs/results; keep command parsing thin. Do not make every command an independently evolving semantics engine.
3. **Cross-product support.** Engine × host OS × containment × feature × custody mode creates combinations that headline feature lists hide. Explicit support metadata can reduce both surprise and redundant conditional code.
4. **Historical documentation embedded in current status.** Very long table cells mix original design, failures, amendments and the latest answer. Preserve history, but generate a compact current capability index from executable evidence.
5. **Distribution drift.** Fresh CI uses new binaries while the operator's terminal/editor can use months-old binaries with the same semantic version. This creates apparent compiler/editor divergence without separate implementations.

The dependency hotspots include Wasmtime/Cranelift, the networking stack, embedded Python when enabled, foreign interop and cryptographic/broker facilities. A dependency tree is not a measurement of linked bytes. Measure link contribution and startup cost before removing dependencies or making a new distribution profile.

## 4. Evidence from real programs

Complete executed sources, input variants, commands, exact outputs and exit statuses are in the appendices. `B` below denotes the fresh external `D:/nelan/DeluluLang/target/release/delulu.exe`; `W` denotes the scratch `work` directory. These are explicit abbreviations, not alternative installed binaries. All actual argv arrays are preserved in the command ledger.

### 4.1 The successful task slices

| Program / case | Observed result | What it establishes, and what it does not |
|---|---|---|
| `basics_while.delulu` | Interpreter and WASM exit 0; `Some(65)`, `[1, 2, 3, 4]`, `A:B`, `Ok(42)`, `Err(Bad(number))` | Records, generic identity, recursion, mutation, list/map operations, strings and `Result` can compose. Not a complete language conformance proof. |
| `utility.delulu` | Both engines emit and write `{"ok":true,"value":42}` for two `value=` lines | A complete local data task with validation and minimal input/output scopes. The input format is a small bespoke line protocol, **not JSON parsing**. |
| `workflow.delulu` | Both engines: one deterministic retry, three completed steps, `total:33` | Bounded in-memory planning/retry/trace logic. No LLM calls, persistent orchestration or independent agent processes. |
| `delegation.delulu` | Both engines read `child-ok` through a narrowed capability | Explicit capability passing and attenuation between functions. This is the language-level analogue of delegation, not a fresh daemon multi-principal test. |
| `edge.delulu` | Both engines and `.dwx`: `4054095`; independent Python and Rust controls agree | Correct bounded aggregation for this fixed input; suitable for comparing these execution paths. Not an edge-device certification. |
| `simulation.delulu` | Both engines: `MOVE:40`, stale/invalid/e-stop refusals, mission transitions `1` and `9`, `telemetry:3:2:2` | Logic-only robot/mission/telemetry examples, including sequence rejection and bounded buffering. Age and sequence are synthetic; no timed transport was simulated. |
| Two-module `review_package` | Corrected manifest builds and runs on both engines, output `42` | Modules work when the package front door and manifest authority are used correctly. |
| Verified `shout` plugin | Built/inspected/verified; permitted call outputs `hello!` | A small pure verified plugin can run. Its artifact is unsigned; no claim about fresh signature/key operations. |
| Sandboxed utility | Exit 0, correct answer, Windows AppContainer confinement reported | The carried filesystem/console path works here under the requested process boundary. |
| `device_sim.delulu` | Interpreter and sandbox without `--seed`: `COMMANDED`, out-of-envelope refusal, `READ 12.0` | Actual in-tree simulator and host mediation, not merely a printed proposed action. No hardware or new signing key was used. |

Representative utility invocation:

```powershell
& $B run utility.delulu --grant console --grant fs.read=./input --grant fs.write=./output --engine interp --no-prompt --limits mem=134217728,cpu=2,wall=5
```

The filesystem grants name only the scratch input/output directories. The parser accepts at most 32 nonempty rows and values from 0 to 1000, then doubles the sum. These are application limits; they do not turn a whole-file read into a streaming byte-limited parser. Production acceptance should include an explicit input-byte bound.

### 4.2 Negative cases and recovered author mistakes

| Case | Actual result | Interpretation |
|---|---|---|
| Utility without grants | Both engines exit 1, DL0703 at console capability mint, no program output | Refusal is enforced before the attempted capability use; this run does not exercise every later denied primitive. |
| Write `../forbidden.txt` through output scope | Both exit 1, DL0904; sentinel remains `sentinel-unchanged` | There is concrete no-forbidden-write evidence, not just a diagnostic. |
| Narrowed child reads `../task.txt` | Both exit 1, DL0904; parent task contents not returned | Attenuation is effective for this spelling. This does not repeat the project's alias/race campaign. |
| Malformed/empty task input | Both exit 0 with an application format-error JSON answer | Rejection is represented as application data. Process success must not be mistaken for task success. |
| Out-of-range value / missing file | Both exit 0 with range/input error JSON | Expected handled failures. A general application exit-status facility is still missing. |
| Computed `scope()` under `authority --within` | Exit 1, **inconclusive**; runtime execution with the actual scope succeeds | Static completeness and dynamic enforcement are different properties. |
| Billion-iteration budget case | Both exit 1 at approximately 1.054 s / 1.036 s for a 1 s wall limit; no `completed` output | The product watchdog stopped execution; the outer test harness did not time out. Not a hard real-time deadline guarantee. |
| Overbroad pure-plugin grant requesting `Write` | Host process exit 0, handled `NotGranted` message; no `hello!` call output | The plugin grant was denied; an outer successful process is not a granted request. |
| Plugin under process sandbox | Exit 2, explicit unsupported PluginHost refusal | No silent in-process fallback. |
| Simulated actuator asks for angle `999` in `[-30,95]` | Handled refusal; final mirror remains `12.0`; sandbox report records DL1904 denial | Host envelope enforcement observed. No conclusion about collision avoidance or safe physical angle choice. |
| Device simulation with `--sandbox --seed 7` | Exit 2, explicit statement that sandboxed `--seed` is unsupported | Deterministic seeding is not portable across this boundary today. |

Three first drafts were author/setup errors and are retained rather than counted as language defects: a mutable map annotated without `ref`; a `main` whose last expression returned the filesystem `Result` instead of Unit; and a package manifest that omitted the program's `Write` ceiling. Moving the plugin host outside the plugin package avoided applying the plugin's pure manifest as the host's authority ceiling. Corrected probes were rerun. The genuine `range` backend failure remained after the map correction.

### 4.3 Machine interfaces

Observed CLI discovery exposes 12 schemas and structured toolchain metadata. `check --json` returned DL0501 plus an exact byte insertion repair for a missing `Write` effect. The corrected scratch source checked successfully. The returned repair was inspected and the corrected source checked; this does **not** claim that the full `delulu edit` transaction was exercised in this session.

The real MCP server completed a stdio initialize/list/check sequence using protocol `2025-06-18`. It exposed 12 tools outside the repository, all annotated read-only. An invalid program returned `isError:false` with `structuredContent.summary.errors:1`: this is intentional for `check`, as [mcp.rs](D:/nelan/DeluluLang/crates/delulu/src/mcp.rs:428) shows. An option-like path was rejected; an attempted nonexistent run tool returned JSON-RPC `-32602`. Consumers must read the result's semantics, not just the transport's success flag.

The real LSP completed initialization, document open, diagnostics, shutdown and exit; it published DL0501. This establishes an operational editor-independent server. It does not establish that the installed VS Code extension activated or found the correct server.

`run --json` on a successful utility emitted the program's own JSON on stdout. The host report belongs in `--report-out`; failures can emit diagnostic envelopes. An agent integration needs an explicit stdout/report contract and must not assume that every successful `run --json` stdout is one host envelope.

## 5. Fundamentals-first gap analysis

### 5.1 Fix task blockers before keyword breadth

The aim is not to imitate all of Python or Java. Exceptions, classes, implicit conversions and ambient environment access are not prerequisites for a useful language. Records, sum types, functions, explicit errors and packages can support substantial programs. Nevertheless, an AI author needs predictable ways to get inputs, transform ordinary data, reuse code and report a result.

| Priority | Gap and evidence | Correct layer / decision |
|---|---|---|
| P0 | Stale installed identity; missing test path succeeds; file/package invocation mismatch | Distribution and CLI semantics. Fix before promising autonomous reliable test loops. [RW 6.51–6.53](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:360). |
| P0 | No ordinary argv/environment/application-exit interface | Define bounded immutable invocation inputs and explicit environment selection. They must enter Authority/host policy; do not expose ambient process state. Choose a structured `main` outcome contract. [RW 2.40](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:162). |
| P0 | No demonstrated standard JSON parser/serializer for ordinary programs | A pure standard-library module, with depth/size/number/error semantics and both-engine support. `parse_json` was absent; the broader library inventory and RW gaps support this conclusion, not that one spelling alone. |
| P0 | Backend selection can fail only after an otherwise valid program is written | Versioned target capability discovery and a target-aware preflight. Preserve explicit refusals. `range` and actuator probes are minimal examples. |
| P1 | Basic math and checked arithmetic missing | Pure library/prelude functions with precise overflow/NaN/rounding behavior; checked operations returning explicit values. Avoid a large new authority surface. [RW 2.38](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:160). |
| P1 | Search/formatting/interpolation/string access incomplete | Library functions first; define Unicode indexing policy before syntax. Typed formatting must retain secret restrictions. [RW 2.37](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:159). |
| P1 | Missing reusable record methods/interfaces | Start with function-call ergonomics or a small method design preserving effects and reference rules. Defer a large inheritance/typeclass system until tasks prove the need. [RW 2.33](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:155). |
| P1 | Bitwise operations absent; useful for binary protocols | Explicit fixed-width integer semantics and invalid-shift behavior on both engines. A host driver can cover specialist protocols meanwhile. [RW 2.35](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:157). |
| P2 | Tuples, sets, comparator sorting and sugar absent | Prioritize comparator sorting/set use cases. Records can substitute for many tuples; `map`/`filter` for comprehensions; `while` for range syntax. [RW 2.39](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:161). |
| P2 | `+=` absent | Useful sugar, but evaluate an indexed/field target once and preserve its effects. Less important than inputs/data/exit semantics. [RW 2.34](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:156). |

JSON is not a new keyword. A retry policy is not a new keyword. A robot, drone or satellite is not a new keyword. Their reusable policies belong in packages and their effects at host boundaries. This keeps the core coherent while making AI-authored programs practical.

### 5.2 Smallest defensible core

Keep deterministic parsing and resolution; typed values and explicit errors; functions/closures with effect propagation; records/sums/generics already implemented; reference-capability rules; capability values with attenuation; explicit effectful primitives; runtime guards; cancellation/resource accounting; structured diagnostics with source locations; and versioned artifact/host interfaces.

Place data formats, math, text utilities and collections in a well-specified standard library wherever possible. Keep network, filesystem, clock, randomness, environment, process/foreign interop and device access mediated by capabilities. Treat Python/GPU/native drivers and specialized robotics protocols as optional host integrations. Make “small” mean **few independent semantic mechanisms**, not simply fewer useful functions.

Do not add exceptions just to resemble other languages; explicit `Result` and `?` already work. Do not weaken mutation or secret rules to reduce diagnostic count. Do not make an AI identity a permission grant. Improvements to machine usability must operate within the same authority model.

## 6. Compiler, CLI and VS Code decisions

| Option | Benefit | Cost / conclusion |
|---|---|---|
| Remove compiler/checker internals and retain a terminal wrapper | Superficially smaller language implementation | Destroys the language, static effects/authority, compiled artifacts, plugin verification and editor analysis. **Reject.** |
| Keep a separate compiler brand/product alongside CLI | Possible specialist embedding story | Not justified by the present architecture or primary agent workflow. Consolidate the product story around `delulu`. |
| Keep language engines inside a CLI-first, API-accessible toolchain | Preserves differentiators and existing evidence; one discoverable front door | Requires a stable service/result model behind the CLI. **Recommended.** |
| Remove VS Code and LSP | Some maintenance saved | Loses shared inspection, diagnostics and machine document editing. Cost exceeds evidence for deletion. |
| Retain LSP plus thin optional VS Code client | Humans can inspect/review; agents and other editors use the same server | Keep executable resolution, workspace trust, diagnostics, basic navigation and check/run tasks tested. **Recommended.** |
| Add editor UI for every broker/device/deployment feature | Rich human console | Duplicates operator workflows and diverts from task completion. **Pause.** |

The distinction is **CLI-first versus terminal-only**. Machines should submit structured arguments and receive typed results, without scraping colored terminal prose or synthesizing shell strings. Existing JSON, MCP and LSP already provide parts of this. A reusable internal request/result API can serve the CLI and long-lived agent processes without inventing a new language protocol for each surface.

The extension's safe executable resolution and argument-array execution are worth retaining: [server-resolve.js](D:/nelan/DeluluLang/editors/vscode/server-resolve.js:68), [extension.js](D:/nelan/DeluluLang/editors/vscode/extension.js:65). An old installed extension and PATH binary are distribution drift, not evidence that the current compiler should be discarded.

## 7. Authority, Guard, Survey, Doctor, Atlas and integrations

| Component | Distinct question it answers | Recommended shape |
|---|---|---|
| Effects/checker | Which kinds of effect may this expression/function perform? | Always-on semantic foundation. Preserve callback effects, resolution and reference rules. |
| Authority | Which resources/scopes can this code request, and is that within a proposed boundary? | Static requirement analysis, including `within / exceeds / inconclusive`. Never silently convert unknown into permission. |
| Grants/leases/broker | What authority has this principal actually been handed, for how long and under which ceilings? | Authoritative runtime custody/delegation. Separate request proposals from grants. |
| Guard | Is this particular effect allowed now, given policy/state/revocation? | Shared enforcement boundary. Improve implementation, not its meaning. |
| Audit | What was attempted/permitted/refused, with what provenance? | Bounded, tamper-evident records with explicit retention/availability tradeoffs. Do not promise that logging proves correct behavior. |
| Atlas | How do a user's program, types, calls, effects and resources relate? | Program-level semantic view, reused by agents and human review. |
| Survey | How do repository components, documents and tests depend on each other? | Maintainer tool. Preserve evidence-linked queries; avoid making end users learn repository internals. |
| Doctor | Is this installation/environment/posture healthy, and is an applicable checkout map current? | One health entry point with actionable machine codes and exact binary identity. |
| MCP | What can an external agent inspect through fixed safe tools? | Keep the current inspection server read-only. Add execution only through a separately designed, explicit authority-bearing interface. |
| LSP | What does the current edited document mean, and where is the error/change? | Editor-independent persistent service; useful for machines too. |
| Plugins | What extension code can run under a declared/verifiable ceiling? | Retain verification, signature/class distinctions and runtime ceiling checks; document containment support precisely. |

These tools overlap in their **data**, not their purpose. Consolidate shared representations and derivations, rather than deleting distinctions between static need, granted authority, runtime permission and observed execution.

Observed Survey impact: `crate:delulu-check` reaches **242** nodes; its `affected-by` walk reaches **43**; `crate:delulu-wasm` impact reaches **132**, with a 64-depth limit and cited hop locations. The checker directly feeds CLI, Atlas, broker, runtime, WASM and conformance. The constitution query marks it entrenched. These are change-planning maps, not proofs that every affected behavior is covered by a test.

The MCP implementation runs fixed CLI argv with a deadline, and reads stdout/stderr on reader threads. It currently buffers them to completion; the deadline is not an explicit output-byte ceiling. See [mcp.rs](D:/nelan/DeluluLang/crates/delulu/src/mcp.rs:367). A persistent service should bound request sizes and response buffers and distinguish protocol failure, analysis failure, denied operation, task failure and timeout.

Some machine-facing prose says grants are “a person's decision.” That is narrower than the owner's clarified design principle. Revise prose to describe authorized principals and configured approval policy when implementation work is authorized; do not remove any existing policy constraint merely to make the language sound more autonomous.


### 7.1 Concrete improvements for AI-operated workflows

These are proposals grounded in the evidence above, not additional implemented features or new security guarantees. Improve the existing authority model and Guard implementation without redefining their meaning. Start with software-agent task completion; use the same policy interfaces through specialized adapters for other domains.

| Component | Next improvement | Decisive acceptance evidence |
|---|---|---|
| **Authority** | A versioned least-authority requirement manifest tied to program/dependency/build identities. Distinguish declared effects, inferred resource needs, unknown dynamic scopes, proposed grants, actual grants and runtime observations. Explain each requirement with a source span and show an authority delta after edits. | Literal and computed scopes, aliases, changed dependencies and edited programs give correct within/exceeds/inconclusive outcomes; an inconclusive answer never becomes an automatic grant. A suggested permission is not itself permission. |
| **Guard** | One consistent operation-time enforcement path across engines/adapters; structured refusal reasons, policy version and revocation provenance. Measure refusal and revocation latency under overload, audit-lock contention, host loss and expiry. | No forbidden effect reaches the host/device; stale/revoked authority fails even after a cached successful preflight. Preserve fail-closed behavior. A software Guard must not be presented as an independent physical interlock. |
| **Broker, leases and delegation** | Give each task/principal explicit bounded authority, lifetime and resource ceilings; preserve delegation ancestry and revocation. Define retry/idempotency and cancellation semantics. Governance follows authorized principals, not whether they are human or AI. | Two actual clients cannot widen a child grant; expiry and ancestor revocation stop subsequent effects; repeated requests do not duplicate a non-idempotent action. Reconnect cannot silently renew expired authority. |
| **Survey** | Keep it a repository-maintenance tool. Provide compact, versioned impact queries with evidence locations, explicit freshness and tracked/untracked scope. Let agents request affected invariants and relevant tests without ingesting the entire map. | A changed primitive or adapter points to the affected checker/runtime/backend/tests; stale or incomplete evidence is visibly marked. No claim that graph reachability proves test completeness. |
| **Doctor** | Inspect the actual installed CLI/LSP/MCP/editor identity, feature profile and supported target/boundary capabilities. Report readiness for a requested workload and actionable machine-readable failures; keep diagnosis separate from repair. | Deliberately stale binaries, unavailable sandbox/device features, missing dependencies and contradictory profile requirements are detected before execution. Read-only diagnosis does not silently reconfigure the machine. |
| **Atlas** | Offer a compact program view connecting calls, effects, resource scopes, plugin dependencies and source spans. Reuse resolved facts, support incremental inspection and show what changed between revisions. Keep semantic inspection separate from runtime evidence. | An agent can explain why a grant is required, identify a widening edit and trace an error back to its source; IDs/hashes prevent applying a result to the wrong revision. |
| **Plugins and adapters** | Publish compatibility, capability ceilings, provenance, supported execution boundaries and bounded request/response contracts. Keep verified pure driver logic separate from the effectful host transport. Recheck permissions when loading/calling, not just when listing metadata. | Tampered/incompatible plugins and overbroad grants are refused; unsupported containment never falls back silently. Driver restart, timeout, device identity change and malformed responses are exercised in simulation. A signature proves provenance, not safe physical behavior. |
| **CLI, MCP, LSP and embedding** | Share structured requests/results and one discoverable capability catalog. Separate inspection from authorized execution. Bind editing and execution to exact content/build identities; define output limits, cancellation and distinct analysis/task/transport outcomes. | An external client completes check → repair → authority review → bounded run → result inspection without shell scraping or interpreting transport success as task success. Reuse persistent analysis where it demonstrably helps. |
| **Language, library and packages** | Complete argv/selected input, JSON, explicit outcomes, math and text facilities before adding domain keywords. Pin dependencies and make their authority impact visible. Preserve effect/reference/secret rules in every new convenience API. | Ordinary multi-module tasks and malformed-input cases run on the advertised profiles; package replacement cannot invisibly broaden authority. |
| **Runtime and backends** | A tested engine/OS/feature matrix; explicit budgets and cancellation; bounded long-lived sessions; reproducibility metadata. Profile representative workloads before adding native/AOT/JIT or extra distribution variants. | Correct outputs/faults on each claimed target, refusal before effects on unsupported combinations, bounded memory over repeated sessions, and no failed run counted as fast performance. |
| **Audit, telemetry and replay** | Correlate task, principal, program hash, grants, policy version and effect outcomes. Bound trace storage and redact secrets. Distinguish replay of recorded decisions from reissuing effects. | Overflow/storage failure behavior is explicit; duplicate or stale events are distinguishable; replay cannot command a device or send a request without fresh applicable authorization. |

Do not create an independent policy engine for each row. Prefer shared typed facts and stable contracts with separate responsibilities. Initial implementation order: truthful identity/outcomes and library basics; requirement/target manifests and complete agent loop; delegation/cancellation/adapters; only then domain-specific operational extensions. Existing implementations that already satisfy a requirement should be reused rather than rebuilt.

### 7.2 Applying the same foundation to different AI workloads

| Workload | What belongs above the shared core | Additional acceptance before claiming support |
|---|---|---|
| Coding/general software agents | Task packages, bounded tools, explicit inputs and outputs | Complete useful task, precise repair/refusal, no grant escalation by a generated patch. |
| Multi-agent systems and fleets | Task coordination, arbitration, principal/lease handoffs, ownership of shared resources | Conflicting commands are resolved by explicit policy; revoking one task does not accidentally authorize or disrupt unrelated tasks. |
| Persistent autonomous services | Durable task state, checkpointing, idempotent retries and bounded queues | Crash/restart recovery, cancellation, resource plateau and no duplicate external effect. The current in-memory retry demo does not establish these. |
| Physical AI and robots | Perception → bounded plan → validated proposal → authorized host action, with a separate low-level safety/control system | Stale/invalid sensor data, actuator saturation, repeated commands, identity changes, lost watchdog beats and independent stop behavior. Keep unbounded model inference outside a claimed hard-real-time loop. |
| Autonomous vehicles and drones | Domain mission constraints, supervisory state machines and approved adapters | Simulation of low energy, geofence violation, communication loss, conflicting commands and safe transition; real deployment needs independent physical safety/timing evidence. |
| Satellites and ground systems | Offline plans with expiry, delayed telemetry handling, bounded storage and controlled synchronization | Out-of-order/duplicate/stale data, clock uncertainty, link outages and reconnect. Loss of contact must not extend authority indefinitely. Onboard hardware/RTOS/flight assurance remains separate. |
| Edge and constrained devices | Small measured runtime profiles and target-specific host adapters | Named-device memory/CPU/startup/power limits and offline update/provenance tests. Desktop or CI evidence is not a substitute. |
| Future AI, including possible AGI/ASI | Versioned interfaces, explicit policy and extensible adapters; no assumptions about model architecture | Evaluate observable tasks and threats when they exist. Greater intelligence grants no additional authority and does not make physical actions safe. |

The near-term foundation should accommodate these later needs without claiming to implement them all now. Prompt-injection resistance, intent correctness, side-channel freedom and world-model accuracy are not proved by effect typing or authority checks; the checks constrain reachable effects under the stated policy.

### 7.3 Remaining simulation cases to add

The executed robot/mission/telemetry examples cover only part of the original physical-AI proposal. Add mock scenarios for two agents contending for one actuator, saturation at and beyond limits, replayed command IDs, device identity replacement, communication loss/recovery, watchdog expiry, clock uncertainty and resource exhaustion during an active mission. For each, specify an expected state transition and independent evidence that the refused action did not reach the simulated transport. These are acceptance proposals, not tests run during this review or this quick follow-up.

## 8. Performance, portability, security and maintainability

### 8.1 Measurements

Method: one unmeasured warm-up followed by **11 measured fresh-process invocations per case**, interleaved round-robin on this Windows machine. Every benchmark output and successful exit was validated. Time includes process startup and relevant parsing/compilation/IO. Peak memory is Windows `PeakWorkingSetSize`, not virtual reservation, total descendant memory or a language heap allocation counter. OS caches were warm; this is **not a cold-filesystem benchmark**.

| Case | Median ms | p90 ms | Median peak working set, MiB |
|---|---:|---:|---:|
| JSON toolchain identity | 13.05 | 14.36 | 6.02 |
| Check utility | 15.34 | 17.13 | 7.45 |
| Authority utility | 15.58 | 18.36 | 7.87 |
| Utility, interpreter | 18.38 | 20.76 | 8.19 |
| Utility, WASM | 25.86 | 28.56 | 17.85 |
| Retry workflow, interpreter | 15.22 | 16.95 | 7.97 |
| Retry workflow, WASM | 21.01 | 24.72 | 17.04 |
| 100,000-reading aggregation, interpreter | 56.29 | 59.70 | 7.84 |
| Same aggregation, WASM | 21.66 | 24.49 | 16.07 |
| Same aggregation, prebuilt `.dwx` | 21.87 | 24.26 | 15.27 |
| Python utility control | 99.21 | 115.01 | 12.63 |
| Python aggregation control | 59.72 | 63.40 | 10.65 |
| Optimized Rust aggregation control | 9.77 | 10.70 | 4.11 |

The Python and Rust controls are **not security-equivalent deployments**. They have no equivalent broker/containment policy; the Python utility imports standard `json`/`pathlib` and reproduces the happy-path computation, not every Delulu validation path. The Rust control uses `std::hint::black_box` around its input and result to discourage trivial constant folding; its small loop still does not represent a full dynamic application. These are sanity/computation controls, not a language ranking.

The measured result is useful but limited: WASM helped this CPU loop by about 2.6× relative to the interpreter, while adding startup/memory cost for the small utility. The `.dwx` artifact was 1,196 bytes, but it needs a compatible runtime; that is not the whole deployment size. The pure plugin artifact was 1,307 bytes. The complete portable Windows CLI was about 33.81 MiB. A 512 MiB configured thread stack is a reservation, not evidence that each run consumes 512 MiB resident RAM.

Do not repeat old debug-build timings as current release performance. Conversely, do not turn a 100,000-iteration example or fast warm startup into “faster than Python/Rust.” Cold startup, realistic package compilation, large input throughput, concurrent sessions, long-lived memory growth, real ARM devices and worst-case latency remain unmeasured here.

### 8.2 How to improve performance without losing guarantees

First profile complete workloads, including check → authority → edit → run → report. Cache immutable checked representations using content, compiler, feature and policy-interface identities; never reuse an approval because the filename stayed the same. Reuse a long-lived analysis process when appropriate. Keep the current interpreter as a predictable simple path and WASM for supported compute-heavy programs. Research native/AOT only after measuring a workload that needs it and specifying how host mediation and artifact verification survive.

Measure binary sections and feature contributions before splitting packages. The default embedded-Python build and portable Python-free build already differ. A future minimal runtime profile may help edge deployment, but an untested combinatorial feature matrix can cost more than the bytes saved. No optimization should bypass the Guard, scope normalization, lease checks or fault semantics.

### 8.3 Portability must be a contract

| Tier proposed | Meaning | Current evidence |
|---|---|---|
| Host tools | Parser/checker/CLI on named OS/CPU versions | Fresh Windows x64 build/probes; current CI has Windows/macOS/Linux and ARM64 jobs. |
| Portable language subset | Same source, results/faults and declared effects on named engines | Several paired probes pass; `range` and device calls demonstrate gaps. |
| Contained execution | Required properties established on a named host profile | Windows utility/device simulator observed at L1; other hosts rely on CI/source evidence and have conditional limitations. |
| Embedded/edge runtime | Published memory/CPU/IO limits on named hardware | Desktop bounded computation only in this review. No microcontroller or small ARM board measured. |
| Domain control | Timing/fault behavior and device/safety case for a deployment | Not established by a language build or simulator. |

The process guest carries console, filesystem, clock, randomness, HTTP, actuator and sensor effects today; [guest.rs](D:/nelan/DeluluLang/crates/delulu/src/guest.rs:516) is the actual list. That is not a claim that Python, plugins, foreign code, actors, secrets or compute handles all cross that channel. Source portability, portable bytecode and native binary portability are three different promises. A Windows executable does not run unchanged on Linux; a `.dwx` still depends on a compatible host ABI and supported capabilities.

### 8.4 Security claims that survive scrutiny

Keep explicit capabilities, checked effects, attenuation, operation-time scope enforcement, revocation, secret custody, artifact/plugin verification and explicit containment refusals. These mechanisms are the main reason to choose DeluluLang for agent-authored tasks.

The formal core is **not a machine proof of the whole compiler/runtime/OS stack**. The current core document describes machine-checked progress-or-fault, preservation/effect soundness and authority properties for a monomorphic closed-row fragment. It assumes conditions about primitive signatures/behavior and does not establish all implemented polymorphism, open rows, foreign execution, networking or OS confinement. See [DELULU_CORE.md](D:/nelan/DeluluLang/docs/design/DELULU_CORE.md:392).

Secrets also need exact wording. `Secret.verify` reveals a chosen bit and carries `Declassify` without requiring `Cap[Declassify]`; this residue is explicitly documented. Arbitrary pure transformation plus repeated verification is not general information-flow secrecy. Broker-held secrets have additional restrictions. Do not market the type as “secrets can never leak.” See [core exception](D:/nelan/DeluluLang/docs/design/DELULU_CORE.md:144) and [soundness discussion](D:/nelan/DeluluLang/docs/design/SOUNDNESS_AUDIT.md:221).

In-process execution is not an OS sandbox. WASM's linear-memory isolation does not eliminate authority of host imports. Process/microVM/external confinement depends on actual host properties and configured profiles. The observed Windows simulator report requested/granted `contained`, level 1, listed an AppContainer and resource ceilings, and recorded the denied command. It also retained `fully_enforced:false` and a `privilege_escalation` limitation alongside property derivations. Consumers should evaluate the explicit requested-profile/property contract; one legacy Boolean is insufficient, and the report terminology deserves clarification. No hostile-agent or privileged-kernel immunity claim follows from this run.

The fresh doctor also explicitly reports **LEGACY root issuance** in this isolated default configuration: another process under the operator's OS user can mint root authority, including over a Guard-sealed resource. The configured anchored-root mode is a different posture; it was not enabled or tested here. The operator's own OS account, its state-directory access and secret custody remain part of the trust boundary. A hostile-agent deployment must specify its intended custody/root-issuance/containment policy and demonstrate it, rather than assuming default `run` supplies that posture. No keys or machine settings were changed during this review.

### 8.5 Maintainability

Security-related duplication should be reduced by shared primitive descriptors, resolved identifiers, effect/authority derivations and test generation, with independent reference witnesses retained. Two backends agreeing can still share the same mistaken checker assumption. Keep negative controls and mutation evidence. Split modules around stable responsibilities, not arbitrary file-length thresholds. Preserve historical Markdown in archives rather than deleting it, and maintain one concise current status view.

## 9. Competitive landscape, checked on 4 November 2026

The table distinguishes published capabilities from conclusions about fit. No external language was installed or benchmarked in this review; only the local Python/Rust controls above ran.

| Project | Verified current direction | What DeluluLang should learn / where it differs |
|---|---|---|
| **Vercel Labs Zerolang** | Experimental language/toolchain with a graph-native program database (`zero.graph`) and a human projection (`.0`); structured queries and hash-guarded checked patches. Official repository latest commit observed `7e1a64d…`, 16 Sep 2026. | Its closest challenge is machine editing/discovery, not simply short syntax. Delulu already has checked edits and Atlas; benchmark task completion before replacing text as the canonical source. [Repository](https://github.com/vercel-labs/zerolang), [CLI](https://zerolang.ai/cli). |
| **Zerolang target/library contracts** | Published target capability table and structured unsupported-target behavior; standard modules cover data formats and invocation utilities. Benchmarks emphasize correctness and regression records. | Adopt the discipline of target-aware capability queries and ordinary libraries. Do not repeat unverified tiny-binary or speed claims. [Targets](https://zerolang.ai/target-capabilities), [library](https://zerolang.ai/standard-library), [benchmarks](https://zerolang.ai/benchmarks). |
| **Mojo** | Official site lists stable **1.2.0**, 22 Oct 2026; 1.0 was announced 11 Aug. The compiler is now open source in Modular's repository. CPU/accelerator programming, Python interop, ownership and compile-time facilities are central. | Older “closed, pre-1.0” comparisons are stale. It is a stronger fit for numerical/accelerator work; Delulu should not build a rival GPU ecosystem before useful agent utilities. Full dynamic Python compatibility remains a staged roadmap, not a blanket Python-superset fact. [Official site](https://mojolang.org/), [1.0 announcement](https://www.modular.com/blog/modular-26-5-mojo-1-0-is-here), [source](https://github.com/modular/modular). |
| **Mojo portability** | Official requirements: Linux/macOS; Windows through WSL, not a native Windows toolchain. Specific CPU/OS minimums apply; GPU programming requires MAX. | Compare exact supported targets, not “runs everywhere.” Delulu's observed native Windows path is useful, but does not establish superior overall portability. [Requirements](https://mojolang.org/docs/requirements/). |
| **Python** | Rich standard data-format facilities and established interoperability. The local JSON module control was straightforward. | Delulu loses on library/task breadth and model familiarity today. Its opportunity is explicit typed authority and bounded execution, not replacing scientific Python immediately. [JSON documentation](https://docs.python.org/3/library/json.html). |
| **Rust** | Ownership supplies memory-safety reasoning without a tracing GC in its ordinary model; effects/OS authority are a separate concern. | Mature native tooling and libraries are a high performance/interop bar. Use Rust for trusted host implementations and compare complete deployments fairly. [Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html). |
| **Go** | A cohesive compiler/tooling/standard-library experience and explicit conventions for large software. | Learn predictable install/build/test/format/package workflows. Do not mistake a smaller keyword count for greater simplicity of completing a task. [Effective Go](https://go.dev/doc/effective_go). |
| **Java** | Large managed-language ecosystem; the old Security Manager is no longer an appropriate sandbox strategy on modern Java. | General runtime containment must be designed explicitly. Java's ecosystem breadth remains a task-completion benchmark, not a model to copy feature by feature. [Oracle Security Manager documentation](https://docs.oracle.com/en/java/javase/24/security/security-manager-is-permanently-disabled.html). |
| **C/C++ and Zig** | Native/system integration is a different priority from brokered authority. Zig emphasizes explicit allocation/control and provides distinct safety/optimization modes. | Low-level control and small deployment can be valuable at the trusted adapter or edge layer; they do not automatically provide agent authority policy. No broad cross-language performance claim is made. [Zig overview](https://ziglang.org/learn/overview/), [ROS real-time design](https://design.ros2.org/articles/realtime_background.html). |
| **Koka** | Research language with effect types/handlers; current book identifies v3.2.8, 17 Sep 2026. | Effect systems are not unique to DeluluLang. The distinctive contribution must be their integration with enforceable scoped runtime authority, not merely printing effect rows. [Book](https://koka-lang.github.io/koka/doc/book.html). |
| **Roc** | Applications rely on a platform for IO; platforms define the available operational interface. | A useful precedent for separating portable application logic from host powers. It does not establish Delulu's lease/Guard semantics. [Platforms](https://www.roc-lang.org/docs/main/langref/platforms/). |
| **Pony** | Reference capabilities address aliasing/sharing and actor concurrency. | Distinguish reference capabilities from object capabilities granting IO authority; the same word is not the same guarantee. [Reference capabilities](https://tutorial.ponylang.io/reference-capabilities/index.html). |
| **Deno** | Scoped runtime permissions, generally no ambient IO without permission; caveats exist for module privilege and isolation. | Permissions alone are not a unique differentiator. Delulu's potential is compositional typed requirements plus explicit delegation and host enforcement. [Security model](https://docs.deno.com/runtime/fundamentals/security/). |
| **Wasmtime/WASI** | WebAssembly isolation plus explicitly imported host functionality; implementation and embedding still matter. | Reuse tested isolation mechanisms. Clearly identify the added language/authority contract above them. [Security](https://docs.wasmtime.dev/security.html). |
| **NVIDIA OpenShell** | Current security guidance describes policies around filesystem, network, process and credential access. The local study examined an earlier version. | An agent execution environment is both a competitor and possible integration layer, not a language replacement. Current compatibility was not executed. [Security guidance](https://docs.nvidia.com/openshell/latest/security/best-practices). |

The strongest defensible differentiation is the **combination** of typed effect/authority analysis, capability values, constrained delegation, operation-time policy and machine-verifiable execution evidence. Each ingredient has precedents. Adoption depends on completing useful jobs with fewer unsafe or ambiguous repair/permission cycles than an existing language inside a good sandbox. That comparison has not yet been demonstrated at scale.

## 10. Keep / simplify / consolidate / replace / defer / remove

Estimates below are relative engineering effort, not calendar promises: S = a contained change; M = several interacting surfaces; L = semantic or platform work requiring substantial assurance. All are proposals. The Survey queries already performed establish broad dependencies; implementation must query the precise node again on its then-current tree.

| Action | Exact area | Reason and expected effect | Effort / risk / prerequisite |
|---|---|---|---|
| **Keep** | `D:/nelan/DeluluLang/crates/delulu-syntax`, `D:/nelan/DeluluLang/crates/delulu-check`, `D:/nelan/DeluluLang/crates/delulu-diag` | Necessary language and machine-feedback foundation | Ongoing M; high semantic risk if weakened. Checker impact reaches 242 nodes. |
| **Keep** | `D:/nelan/DeluluLang/crates/delulu-broker/src/authority.rs`, `guard.rs`, `lease.rs`, `tree.rs` | Preserve enforceable delegation and revocation | High risk; changes require invariant witnesses and current impact queries. |
| **Keep** | `D:/nelan/DeluluLang/crates/delulu-wasm`, `D:/nelan/DeluluLang/crates/delulu-runtime/src/interp.rs` | Useful execution alternatives with measured tradeoffs | M/L; backend impact 132 nodes. Require a published parity/support matrix. |
| **Simplify** | `D:/nelan/DeluluLang/crates/delulu/src/cli.rs`, `run_cmd.rs` | Thin parsing/output layer over reusable services; reduce inconsistent front doors | M, medium risk; preserve every command's diagnostics, exit and authority ordering. |
| **Consolidate gradually** | Checker `check.rs`, `authority.rs`, `resolve.rs`, `rcap_check.rs`; WASM `codegen.rs` | Shared resolved facts and checked representation reduce repeated interpretation | L, high risk; introduce one slice at a time with differential and independent witnesses. Do not replace all passes at once. |
| **Keep, focus** | `D:/nelan/DeluluLang/crates/delulu-atlas`; `D:/nelan/DeluluLang/crates/delulu/src/edit.rs` | Program structure, checked edits and authority deltas directly serve agents | M, medium risk; stale hashes, byte boundaries, changed authority and rejected edits need executable contracts. |
| **Keep optional** | `D:/nelan/DeluluLang/editors/vscode`, `D:/nelan/DeluluLang/crates/delulu/src/lsp.rs` | Human review and editor-independent persistent analysis | S/M; validate installed-server identity. Pause new dashboard/editor feature scope. |
| **Keep separated** | `D:/nelan/DeluluLang/crates/delulu/src/mcp.rs` | Read-only inspection boundary is easy to reason about | S/M; response limits and explicit result semantics first. Execution needs a separate authority-bearing design. |
| **Simplify presentation** | `D:/nelan/DeluluLang/crates/delulu/src/doctor.rs`, `D:/nelan/DeluluLang/crates/delulu-survey` | Installation/posture health and repository health should be understandable | M; fix stale wording and distinguish local untracked material from committed-map drift. Avoid deleting the maintainer map. |
| **Keep verification; document limits** | `D:/nelan/DeluluLang/crates/delulu-runtime/src/plugin.rs`, `verified_adapter.rs`; `D:/nelan/DeluluLang/crates/delulu-check/src/plugin.rs` | Extensions are useful only with honest ceilings, classes and boundary behavior | M/L, high security risk; unsupported sandbox paths must fail before execution. |
| **Evaluate optional packaging** | `D:/nelan/DeluluLang/crates/delulu/Cargo.toml`, `D:/nelan/DeluluLang/crates/delulu-runtime/Cargo.toml`, `D:/nelan/DeluluLang/scripts/package-toolchain.sh` | Python-free portable build already exists; further profiles may reduce size | M, medium risk; measure bytes and maintain a small tested profile set. Do not proliferate feature combinations. |
| **Replace contradictory current prose** | `D:/nelan/DeluluLang/docs/DEPLOYMENT.md`, `D:/nelan/DeluluLang/docs/DELULULANG_V2/V2_P8_DESIGN.md`, `D:/nelan/DeluluLang/docs/REMAINING_WORK.md` | Current summaries must match actual source and current evidence | S/M; preserve history. Entrenched design edits need the repository's decision/change-record process after owner authorization. |
| **Defer expansion** | P8/P9 specialist integrations, new device stacks, native optimizer/backend | Useful later, but do not close today's argv/JSON/package gaps | L; acceptance suite and named deployment requirements first. Keep already working P8/P9 code. |
| **Remove from product claims** | Universal portability, whole-system proof, guaranteed safe autonomy, generic superiority | These claims exceed evidence and make correct mechanisms harder to trust | S; wording review, not deletion of implementation or historical records. |
| **Do not remove** | `D:/nelan/DeluluLang/crates/delulu-conform`, tests, formal models, security documents | Assurance is part of the product, not optional repository clutter | Removal would be high risk; improve test quality and runtime cost without weakening witnesses. |

The existing `delulu-check/src/dir.rs` already represents checked/serialized program material. Investigate whether that representation or a related internal form can carry more shared resolved facts; do not invent a new IR merely because “IR” sounds cleaner. Keep serialized artifact compatibility separate from freely evolving compiler internals.

## 11. A fundamentals-first roadmap into V2.0

### Milestone A — establish one trustworthy installed toolchain

Make package identity include immutable commit, target, feature profile, schema versions and artifact hash. The source already supports an injected commit through [build.rs](D:/nelan/DeluluLang/crates/delulu/build.rs:9) and [version output](D:/nelan/DeluluLang/crates/delulu/src/cli.rs:1298); the gap is making identity reliable in distribution and comparing what the user actually runs. Doctor should identify the executable used by PATH and the editor, and distinguish unknown identity from an exact match. Do not infer age from `1.0.0` alone.

Acceptance: install a prepared package in a disposable environment; CLI, MCP, LSP and editor report the same immutable build; a deliberately stale binary is detected; no checkout-local executable can silently replace the configured server. Resolve missing-test-path and zero-test ambiguity. Either resolve package context from an entry file or give a specific package invocation diagnostic.

### Milestone B — complete ordinary agent utilities

Specify invocation input, selected environment, structured application outcomes, JSON, math/checked arithmetic and essential text APIs. Implement them in the smallest suitable layer. Build three complete tasks: a JSON task runner, a multi-module transformation package, and a bounded offline retry workflow. Include hostile/malformed/deep input and refusal paths. Add syntax conveniences only after those tasks work.

Acceptance: the same task sources run under the advertised interpreter/WASM/process-boundary combinations; unavailable combinations are identified before execution. No external Python helper should be required for a task advertised as a Delulu standard-library capability.

### Milestone C — publish a machine-readable execution contract

Publish engine/feature/host capabilities, schema versions, stable outcomes and scope precision. Document that MCP call success differs from checker success, and that program stdout differs from the run report. Add an explicit service boundary for authorized execution only if a real client needs it; keep inspection read-only. Reuse LSP's persistent analysis rather than automatically creating another daemon.

Acceptance: a small external client can discover capabilities, check a bad program, apply an approved non-widening repair, reject a stale edit, explain a widening edit, run within pre-existing authority, cancel, and parse the final report without terminal scraping. “Requires human: false” in a diagnostic must never itself authorize new grants.

### Milestone D — make performance and portability claims reproducible

Use fixed inputs, independent output checks, named machine/toolchain/build profiles, warm and startup measurements separated, and realistic package sizes. Preserve allocation/work-scaling gates and add long-lived session memory checks. Measure several real task shapes before choosing interpreter optimization, WASM compilation caching or AOT.

Acceptance: published raw records and current support matrix; no failed execution included in a fast-result statistic; no advertised OS/device inferred only from a different platform. ARM support requires named hardware or an explicitly identified CI environment, not “Rust can cross-compile.”

### Milestone E — controlled simulation and adapter expansion

Keep the already built simulator, host-mediated device operations, verification and monitor work. Add a domain package only with a bounded simulator acceptance case and a named operational requirement. A real physical pilot needs separate device, safety and timing authorization/evidence. Do not put planning or unbounded AI inference inside a hard real-time control loop merely because the language can express it.

### Release decision

The launch branch and prior release gates represent valuable preparation. Recommend **holding the broad V2 public launch claim until A–C and the small release bar in §16 pass**, then describing the first scope as bounded local/software-agent tasks plus simulation. Performance/edge work can continue incrementally. No restart is needed.

Retain the existing release obligations: package provenance and dependency/licence evidence, backup and history preservation, current security contact information supplied by the owner, and GPL source/compliance obligations before any built microVM kernel image is distributed. A green release dry run is not permission to publish and does not by itself establish those obligations have been satisfied. The workspace's historical Cargo repository metadata also differs from the actual test origin; distribution metadata should be reconciled only as part of an authorized release decision, without changing origin here.

The owner should decide the target scope, release hold versus a narrower preview, compatibility policy for new application outcomes, any constitutional changes, and the name after clearance. Repository/tag/publication changes remain explicitly reserved by the owner's latest instructions. This review neither creates a D-V2 decision nor exercises old autonomous-development authority to implement anything.

## 12. Difficult tradeoffs, unresolved questions and verdict

**Language versus generic sandbox.** A runtime-only product could accept Python or JavaScript and gain an ecosystem immediately. It would lose source-level authority/effect reasoning unless it rebuilt a restricted language or trusted external analysis. Keep DeluluLang, but expose host interfaces that can cooperate with existing ecosystems. Reassess the runtime-only alternative only if task-completion studies show the language imposes costs users will not accept.

**Static precision versus useful dynamic programs.** More abstract interpretation can reduce inconclusive authority results but adds complexity and denial-of-service risk. Keep runtime enforcement authoritative; make static uncertainty actionable, bounded and explicit. Do not force all useful dynamic behavior into literals just to make every report say “within.”

**Convenience versus ambient authority.** Argv, environment and exit behavior are urgent, but the implementation should accept explicit input values and selected authority rather than expose the whole host process. Basic usability need not mean hidden privilege.

**Audit availability versus stopping machinery.** Fail-closed recording and prompt emergency handling can conflict under lock/storage failure. The project records a residual five-second per-request audit wait; its corrected queued-revocation experiment still took 9.81 seconds under a held chain. That is prior recorded evidence, not a new measurement here. It rules out treating this software e-stop as a hard real-time physical interlock. [RW 4.53](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:263).

**Machine editing versus text.** Zerolang demonstrates a plausible structural approach. Delulu already has Atlas and hash-checked edits. Measure stale-edit failures, repair success, authority widening, token use and merge/debug costs before changing canonical source storage. Human inspection remains essential even when agents author most code.

**Compatibility versus cleanup.** Diagnostics, fault order, output schemas, compiled artifacts and grants are contracts. Seemingly small cleanup can change an agent's behavior or authorization decision. Explicit migration/versioning beats silent changes.

Unresolved questions include: which first external adopter/task is being optimized; the permanent library/package distribution model; precise application outcome/argv semantics; which engine profile is the compatibility baseline; measured long-lived analysis memory; a credible deployment/size target for edge hardware; and a jurisdiction-specific name-clearance opinion. No finding here establishes real-device safety, flight readiness or AGI/ASI requirements.

**Verdict:** preserve the language's authority foundation and assurance work; make ordinary bounded jobs and machine contracts the next definition of progress. Do not delete the compiler. Do not replace the project with a giant general agent framework. Do not launch a universal-autonomy claim from simulator and CI evidence.

## 13. AI-first users and use cases, ranked

| Rank | User/workload | Near-term value and feasibility | Main blocker / next evidence |
|---:|---|---|---|
| 1 | Local/cloud coding agents producing bounded utilities | High value; existing checker/JSON/edits/authority/containment support most of the loop | Inputs, JSON, install identity, outcome/test correctness; complete external-client acceptance. |
| 2 | General software agents doing controlled data/file transformations | High value; local utility already demonstrated | Library breadth and selected network/service integrations; mock first, explicit grants always. |
| 3 | Multi-agent task delegation | Good architectural fit; capability attenuation and broker delegation exist | Fresh independent-principal workflow, lease expiration/revocation/cancellation and durable result semantics. Function passing alone is insufficient evidence. |
| 4 | Robot/vehicle/drone/ground-system simulation | Feasible now for bounded state/telemetry logic and the existing device simulator | Better standard math/data/time utilities and repeatable simulator seeds across advertised boundaries. |
| 5 | Edge software agents on named Linux/ARM hardware | Potentially useful; WASM and bounded computation are promising | Actual hardware size/startup/power/memory measurements; offline package/update and supported adapters. |
| 6 | Real physical supervisory agents | Valuable only within an independently safe plant/control system | Real driver validation, failure containment, independent interlocks, timing budgets and operational ownership. |
| 7 | Safety-critical vehicle/drone/industrial control | Long-term specialized work; high cost and risk | Domain safety case, verified timing/failure assumptions, process/tool qualification where applicable. |
| 8 | Onboard satellite/spacecraft control | Research direction; ground telemetry tools are much nearer | Radiation/fault tolerance, hardware/RTOS integration, bounded resources, flight assurance and operational heritage absent here. |
| — | Future AGI/ASI systems | Preserve general capability/policy interfaces and avoid identity-based privilege | Requirements are unknown. No meaningful “AGI-ready” acceptance test exists today. |

Humans are secondary as code authors in this product priority, but remain operators, reviewers and maintainers. That changes interface priorities, not the trust hierarchy. Grants and enforceable policy govern both human and AI principals.

## 14. Machine experience, adoption and editing

### What is already genuinely agent-oriented

Structured diagnostics and repairs; discoverable schemas/toolchain metadata; explicit authority summaries; `within` reasoning; program relationship queries; bounded checked edits; read-only MCP; persistent LSP; machine-readable run reports; and actual host enforcement are substantive mechanisms. They are more than naming or branding. The useful local utility and workflow prove a small composition of them can work.

### What prevents reliable autonomous task completion

An agent can still use a stale binary, pass a nonexistent test path and read success, select an unsupported engine late, misread an application error as process success, or assume MCP `isError:false` means the code checked. A model trained on common languages also guesses method syntax, JSON and argv facilities that do not exist. The remedy is a compact capability-specific first-program kit and executable examples, not more prose claiming the language is intuitive.

The fresh `--version --json` feature means RW 6.53's “build identity first” wording is partly stale. The remaining work is distribution enforcement, feature identity and comparison. Likewise, earlier claims that lists never lower to WASM are superseded by current execution and parity source. Current documentation should carry a “valid for build/profile” reference and link to old history rather than leave incompatible eras in the same answer.

### An adoption experiment that would be persuasive

Give at least two independent agent clients a fixed corpus of 20 useful tasks with fixed budgets, versions and initial context. Compare DeluluLang against Python or another established language **under equivalent effect/containment policy**. Include JSON/file work, modules, failures, plugins, scoped delegation and cancellation. Record success against an independent oracle, unsafe attempts blocked, inappropriate permission requests, repair iterations, tool calls, elapsed time, input/output tokens where observable, and total deployment dependencies. Repeat enough runs to expose model variability; disclose every failure and exclusion.

Use the project's existing measurement harness as a starting point. Its prior pilot and process-startup studies are evidence about their recorded configurations, not a current universal model benchmark. No new live model/billing benchmark was run in this review. Token efficiency and model familiarity were not measured by the microbenchmarks.

The machine-facing workflow should be: discover exact build → discover target support → generate/edit under a content precondition → check → inspect authority/delta → obtain only an authorized boundary → execute with budget → inspect task outcome and host report → retain reproducible evidence. This can be accessed through CLI, structured API or a separate authorized execution service; it need not be tied to an IDE or shell.

## 15. Physical AI and autonomous-system readiness

| Subsystem | Current code/project status | Evidence in this session | Remaining boundary |
|---|---|---|---|
| P8-01 host-mediated device operations from a guest | Built; host performs command/read with envelope/lease checks | Actual Windows simulator command/refusal/readback inside L1 | This is a simulated device; no physical movement or response measured. |
| P8-02 Verified `.dpx` driver logic | Built; pure driver exports, checked artifact, pinned signature for that path, host transport | Source/design/tests inspected; pure general plugin tested | Signed device-driver path not rerun; no keys generated. Verification is not a proof that a physical command is safe. |
| P8-03 reference line transport and simulator process | Built; simulator can be a separate process | Test design read; in-process simulator used by own probe | Separate signed-driver transport scenario not rerun here; no serial/CAN hardware. |
| P8-04 monitor/quarantine | Later design ledger says complete | Source/design/history evidence only | It can hold authority to revoke more than a single child; monitor compromise/latency/failure must be included in the threat model. |
| P9-01 `authority --within` | Built | Within, exceeds and inconclusive observed | Static uncertainty remains; runtime checks are still required. |
| P9-02 grant differences | Built in records/source | Not independently executed | Need fresh multi-principal expansion/narrowing acceptance. |
| P9-03 proposals and bounded approval | Built in records/source | No approval/grant mutation performed | An execution protocol must preserve approver authority and revocation lineage. |
| P9-04 endpoint-bound secrets | Built, including audience-aware analysis | Source/current records inspected; no real secret sent | This is controlled credential use, not whole-program noninterference or protection from the operator's own user. |
| P9-05 method/path network authority | Completed in current records, including added HTTP methods | Source/CI evidence; no real network program executed | Revalidate adapters/redirects under the actual deployment; do not repeat the outdated “GET only” claim. |
| Process/microVM/external/attested boundaries | Several implemented profiles with different host requirements | Windows L1 observed; L2 unavailable here | External launcher/attestation code exists despite stale probe prose. Signed attester statements are not automatically hardware-rooted TEE evidence. |

The top of [P8 design](D:/nelan/DeluluLang/docs/DELULULANG_V2/V2_P8_DESIGN.md:9) says P8-04 is not complete, while its later ledger at [line 265](D:/nelan/DeluluLang/docs/DELULULANG_V2/V2_P8_DESIGN.md:265) says the build order is complete. Treat the later implementation ledger and current source as stronger evidence. Likewise, the actual `sandbox probe` prints obsolete L3/L4 availability explanations; [RW 6.50](D:/nelan/DeluluLang/docs/REMAINING_WORK.md:359) already records this. Do not infer “unimplemented” solely from those strings.

The strongest physical-AI evidence here is still **software simulation and authorization enforcement**. The program handles a stale sample, invalid value, e-stop flag, low-battery mission transition, duplicates/delays and a bounded queue. It does not establish sensor integrity, synchronized clocks, scheduling deadlines, actuator dynamics, collision avoidance, stability, safe stopping distance, reliable communications, watchdog independence or recovery after power loss.

The real-time requirements are separate: bounded execution and allocation, scheduling guarantees, bounded communication/locking and a deployment-specific deadline analysis. ROS's design guidance explicitly discusses these concerns; the existence of a language-level limit does not satisfy them. [ROS real-time background](https://design.ros2.org/articles/realtime_background.html).

Before a physical pilot, require a real device/adapter contract, fault-injection bench, independently enforced envelope/interlock, measured worst-case response under overload and storage failure, and an operational safety review for that device. Before aviation/automotive/space claims, obtain the applicable domain assurance and qualification evidence. This report does not determine which certification regime applies to a hypothetical product.

## 16. Prioritized acceptance suite and measurable release bar

These are **proposed release requirements**, not claims that the suite already exists or passes. Existing tests should be reused where they truly witness the property. Each negative gate must have a demonstrated failing control; never weaken a security witness to make a milestone green.

| ID | Priority | Executable acceptance | Current review result / release requirement |
|---|---|---|---|
| A01 | P0 | Exact build identity across downloaded CLI, MCP, LSP and editor | Fresh binary succeeds; installed mismatch found. Release: immutable identity available everywhere, stale installation detected. |
| A02 | P0 | JSON task input → validation → transformation → JSON answer | Bespoke line-format version succeeds; standard JSON version blocked. Release: documented JSON library, byte/depth bounds, malformed/Unicode/number tests. |
| A03 | P0 | Valid, malformed, missing and oversized input outcomes | Handled failures work but exit 0. Release: unambiguous application outcome and documented process exit mapping. |
| A04 | P0 | Package directory and entry-file invocation | Package works, file import fails. Release: consistent resolution or precise actionable refusal; no guessed import workaround. |
| A05 | P0 | Existing test, missing path, empty filter, failing test | Missing path produces zero-test success. Release: missing path cannot be reported as passing tests; zero matches explicit and machine distinguishable. |
| A06 | P0 | No grant / outside scope / narrower child | Refusals observed; sentinel unchanged. Release: independent filesystem/network/device evidence of no forbidden effect on each supported boundary. |
| A07 | P0 | Static within / exceeds / inconclusive | All three observed. Release: callers must handle each explicitly; unknown never widens a grant. |
| A08 | P0 | Interpreter/WASM/guest capability preflight | Late range/device/seed refusals observed. Release: advertised subset passes, unsupported combinations refused before effects. |
| A09 | P0 | Hash-checked repair and authority delta | Repair inspected and corrected source checks; full edit transaction not run. Release: stale hash rejects; exact repair works; widening never self-authorizes. |
| A10 | P0 | CLI/MCP/LSP outcome/schema contract | Servers and diagnostics work; success semantics differ. Release: contract tests teach/check these differences and bound input/output sizes. |
| A11 | P0 | Wall/CPU/memory limits and cancellation | Wall stop observed on both engines. Release: separate external cancellation, CPU/memory exhaustion and descendant cleanup tests; outcome names the stop reason. |
| A12 | P1 | Verified plugin allowed/overbroad/ungranted/unsupported boundary | Pure allowed/denied cases observed; sandbox refuses. Release: tamper/signature/class/ceiling tests in supported profiles, no fallback. |
| A13 | P1 | Independent parent/child agent principals | Language-level narrowed capability demonstrated. Release: two real broker clients, widening denied, expiry/revocation effective, receipts/results inspectable. |
| A14 | P1 | Long-lived bounded workflow with restart/replay | In-memory retry succeeds. Release: define durability/idempotency policy, cancellation, bounded trace and memory over repeated work. |
| A15 | P1 | Offline clean package use | Offline cached-source build and local package ran. Release: documented offline artifact/dependency installation, missing dependency fails clearly, no hidden downloads. |
| A16 | P1 | Robot/mission/telemetry simulation | Proposed-action logic and actual device simulator pass locally | Release simulation profile: stale/out-of-order/invalid data and queue overflow are checked; unavailable seeded sandbox mode is explicit. |
| A17 | P1 | Correct performance record for at least three workloads | Utility/workflow/aggregation measured | Release: raw timing/memory/output records, fixed machine/profile, startup versus steady state separated; at least 101 samples for p95 claims. |
| A18 | P1 | Current evidence and distribution claims | CI green; checkout map stale; prose conflicts found | Release: exact candidate CI read green, map/doctor state explained or healthy, package install/editor checks current, no unsupported public claims. |

An intentionally small V2 AI-first release bar is **A01–A11 and A18 all passing**, with A12/A15 required for any plugin/offline feature promoted at launch, and A16 required for the simulator claims. Other P1 capabilities can ship as clearly scoped previews or follow in 2.x. Existing mandatory security/formal/distribution gates remain mandatory; this task suite supplements them.

Suggested initial performance objectives, to be ratified on a named reference machine: p95 warm fresh-process check and utility completion below 100 ms for the fixed small corpus; no more than 20% unexplained regression against the accepted reference; memory stays under the selected 128 MiB process budget for these tasks; long-lived analysis reaches a bounded plateau over 1,000 repeated edits. These are engineering targets, not cross-device promises. Do not use shared-runner jitter as evidence of semantic regression without a same-machine control.

## 17. Recommendations from this review

| Recommendation | When | Impact | Effort / risk | Evidence and prerequisites | AI-first purpose |
|---|---|---|---|---|---|
| Keep a language plus agent toolchain; consolidate the product around `delulu` | **Do now: decide direction** | Highest strategic clarity | S decision; implementation M | Architecture and working tasks; owner accepts initial scope | Agents get analyzable programs plus enforceable execution. |
| Repair installation identity and task/test outcome ambiguity | **Do now, after implementation authorization** | High reliability/adoption | S/M, medium contract risk | Stale PATH, zero-test success, package-path probes | Prevents false autonomous success. |
| Add invocation/data/math/text fundamentals in suitable layers | **Do now, after specification** | High task completion | M/L, semantic risk | Minimal failures and RW 2.33–2.40; decide errors/Unicode/authority semantics | Reduces model guesswork and external helper dependence. |
| Publish exact target/engine/boundary support | **Do now** | High predictability | M, low-to-medium risk | Range/device/seed/plugin refusals; generate from tested descriptors | Agents choose a supported plan before executing. |
| Keep LSP and a thin optional editor client | **Do now: retain; do later: extras** | Medium review value | S/M | Real LSP works; validate installed client identity | Shared machine/human program understanding without IDE dependence. |
| Preserve read-only MCP; design authorized execution separately | **Do now: preserve; research further: service** | High trust-boundary clarity | M/L, high security risk | MCP source/wire evidence; explicit principal/grant/cancellation design | Enables automation without turning inspection into authority. |
| Consolidate repeated semantic facts incrementally | **Do later, after task contracts** | High maintenance benefit | L, high regression risk | Source coupling, Survey impacts and defect history | Keeps analysis/runtime/backend answers consistent. |
| Measure full agent loops against equivalent contained alternatives | **Do now: design; do later: run** | High evidence/adoption value | M, experimental risk | Existing harness; fixed tasks, agents, budgets and independent oracle | Measures useful safe completion, not feature count. |
| Pursue native/AOT/GPU or smaller edge runtime only from measured needs | **Research further** | Potential high performance | L, high scope risk | Current loop gains do not prove general benefit; need real hardware/profile | Avoids sacrificing authority for speculative speed. |
| Retain P8/P9 but limit current positioning to software/simulation evidence | **Do now** | High credibility/safety | S wording; L physical work | Simulator pass, real-device and timing evidence absent | Preserves physical-AI direction with honest stage boundaries. |
| Replace contradictory current summaries, preserve history | **Do now when authorized** | Medium maintenance/adoption | S/M | P8 header, sandbox probe, old WASM/library and identity statements | Agents consume one current capability answer. |
| Keep DeluluLang provisional; do not adopt KingLang now; clear any final name | **Do now: naming decision** | High future migration savings | S screening; professional clearance scope varies | §18 exact collisions and related marks | Avoids brand churn and misleading legal certainty. |
| Delete compiler/checker/assurance tests; rewrite everything; promise universal safe autonomy | **Do not do** | Would destroy value or overstate it | Very high risk | Architecture, proof limits, runtime/platform evidence | These steps undermine the AI-first mission. |

## 18. Naming and IP: DeluluLang, KingLang and alternatives

### 18.1 Is there a problem with keeping DeluluLang?

**There is no established finding here that you must stop using DeluluLang. There is also no basis to call it legally cleared.** The exact combined name is comparatively distinctive in the limited language/repository search. The important legal issue is that **DELULU already appears in relevant AI/software trademark records**, and adding `Lang` does not automatically remove similarity or confusion concerns.

There is a separate branding choice. DeluluLang is memorable, easy to pronounce and has an existing project identity. My judgment is that the “delusional”/playful association can make positioning around dependable authority and safety harder, particularly with enterprise or physical-systems audiences. That is a branding inference, not measured customer research and not a legal prohibition. It can also work as an intentionally playful open-source identity. A serious technical product does not become technically weak because its name is humorous.

**Recommendation:** retain DeluluLang as the working name while testing a short alternative shortlist and obtaining targeted clearance before public investment in branding. Do not incur a repository-wide migration merely because “King” sounds stronger. A name change is most useful when it improves distinctiveness, pronunciation, discoverability and long-term fit together.

### 18.2 “Generic words” are not automatically free as software marks

In trademark analysis, “generic” means the ordinary name of the relevant goods/services, not simply a word found in everyday speech. A common word can be distinctive for unrelated goods. Thus neither “King is a common word” nor “Delulu is slang” settles whether it can identify software or AI services. Marks can conflict through sound, appearance, meaning or commercial impression, with related goods/services; exact equality is not required. [USPTO: strong trademarks](https://www.uspto.gov/trademarks/basics/strong-trademarks), [likelihood of confusion](https://www.uspto.gov/trademarks/search/likelihood-confusion).

The categories must be separated:

| Right / namespace | Relevance |
|---|---|
| Copyright | The bare name is generally not the issue. US guidance excludes names/short phrases; India's handbook similarly says names/titles are not ordinarily protected by copyright. Code, docs and sufficiently original logo artwork can be protected. [US Copyright Office](https://www.copyright.gov/help/faq/faq-protect.html), [India handbook](https://copyright.gov.in/documents/handbook.html). |
| Trademark / passing off / unfair competition | The central naming risk: use/registration, similarity, related activities, territory and priority. A free/open-source project is not automatically outside these questions. |
| Patent | Protects claimed inventions, not a product name as such. A patent mentioning KingLang is prior-use/naming evidence; infringement requires claim-by-claim analysis. Renaming would not avoid an otherwise relevant technical patent. |
| Company/domain/GitHub/package name | Practical discovery and namespace constraints. Registration or an HTTP 404 is not trademark clearance. GitHub repository names are not globally exclusive. |
| Licences | Existing source/dependency obligations survive a rename. Do not erase attribution, copyright notices or historical evidence. |

This is a preliminary public-source screening, not a legal opinion or freedom-to-operate assessment. A trademark professional should evaluate the final candidate and actual goods/services in the intended launch markets. That recommendation follows specific related software marks and incomplete national coverage, not a hypothetical reason to stop development.

### 18.3 Exact findings and national coverage

All statuses below were checked on 4 November 2026. Official records were preferred over aggregators; some aggregator labels were materially stale or imprecise.

| Jurisdiction / source | DeluluLang / related DELULU | KingLang / related KING | Limits |
|---|---|---|---|
| **United States — USPTO search + TSDR** | Exact `DeluluLang` wordmark query returned no results. **DELULU**, serial **99795106**, Decart.Ai Inc., filed 29 Apr 2026, is a live application under examination; non-final office action 3 Sep 2026; classes 9/41/42 include AI audiovisual-generation software/services. It is **not a registration**. | `KingLang` query returned the **KINGLANG** tableware mark, serial **88278425**, registration **5924581**, class 21. Official TSDR says **cancelled 19 Jun 2026** for the Section 8 issue, rather than the live registration suggested by some aggregators. | Exact queries are not comprehensive similar-mark/common-law searches. A cancelled registration is not proof of abandoned commercial use. |
| **US additional related record** | **DELULU X**, serial **99665938**, Veritas Apex LTD, filed 23 Feb 2026; notice of allowance 13 Oct 2026, still pending. Classes 9/38/41/42 include games and software services. | No broad KING-family clearance performed. | Some secondary pages shorten the mark to DELULU; the official literal mark is DELULU X. |
| **European Union — EUIPO** | Exact `DeluluLang` basic query returned zero trademarks. **delulu**, figurative EUTM **019273217**, Decart.Ai Inc., filed 7 Nov 2025, **registered 28 Mar 2026**, classes 9/41/42, is a relevant AI/software record. | Basic `KingLang` search returned **BOOKINGLANGUAGES 009748121**, an expired figurative mark containing the character sequence; it is **not an exact KINGLANG match**. No exact mark was identified in this limited query. | EUIPO does not clear all national rights or all of geographical Europe. Similarity analysis of the figurative DELULU mark remains necessary. |
| **United Kingdom — UKIPO** | Related figurative record **UK00004343484**, Veritas Apex LTD, filed 20 Feb 2026, **registered 22 May 2026**, classes 9/38/41/42. The official page's image shows DELULU wording with an X element and a bird-head design; software/programming services appear in the specification. | No complete UK exact/similar KINGLANG search completed. | Specific related record verified, not comprehensive UK clearance. The direct image endpoint failed, but the image was visible in the official record page. |
| **India — IP India** | Official register could not be searched successfully in this session. | Same limitation. A secondary lead for KING class 9 is not treated here as a verified live conflict. | Official search returned HTTP 503; indexed interface also indicates login/OTP/CAPTCHA. Neither name is cleared in India. |
| **China — CNIPA** | No completed official name query. | No completed trademark query. Separately, a published Chinese software patent explicitly describes **KingLang**; details below. | Official portal reached the SSO login boundary. No account/login or CAPTCHA was completed. Romanized-only web results are inadequate for Chinese clearance. |
| **South Korea — KIPRIS** | Domestic trademark query `DeluluLang`: no results. | Domestic trademark query `KingLang`: no results; UI separately indicated one foreign result, not a Korean domestic registration. | Limited Roman-letter query. Hangul, phonetic, similar marks, class/family searches and legal status validation remain open. |
| **Japan — J-PlatPat** | Application/registration information, “Trademark (for search)” field `DeluluLang`: hit count 0. | Same field `KingLang`: hit count 0. | No Katakana/pronunciation/similarity clearance. Zero in this query is not proof of no relevant Japanese right. |
| **Other territories / WIPO** | WIPO coverage guidance consulted; no complete worldwide name search. | Same. | Canada, Australia, individual European national registers and other jurisdictions are unassessed. No worldwide clearance claim. |

Primary record links: [US DELULU 99795106](https://tsdr.uspto.gov/#caseNumber=99795106&caseSearchType=US_APPLICATION&caseType=DEFAULT&searchType=statusSearch), [US DELULU X 99665938](https://tsdr.uspto.gov/#caseNumber=99665938&caseSearchType=US_APPLICATION&caseType=DEFAULT&searchType=statusSearch), [US KINGLANG 88278425](https://tsdr.uspto.gov/#caseNumber=88278425&caseSearchType=US_APPLICATION&caseType=DEFAULT&searchType=statusSearch), [EU 019273217](https://euipo.europa.eu/eSearch/#details/trademarks/019273217), [UK 04343484](https://trademarks.ipo.gov.uk/ipo-tmcase/page/Results/1/UK00004343484). National search entry points: [IP India](https://tmrsearch.ipindia.gov.in/ESEARCH), [CNIPA](https://sbj.cnipa.gov.cn/trademark-query), [KIPRIS](https://www.kipris.or.kr/khome/main.do), [J-PlatPat](https://www.j-platpat.inpit.go.jp/t0100), [WIPO coverage](https://www.wipo.int/en/web/global-brand-database).

An exact search hit is not automatically a legal prohibition, and an exact no-hit is not clearance. Software classes 9 and 42 deserve particular attention, with class 41 for education/training or content services as applicable. Classes are organizational categories, not independent safe zones. The actual goods/services, use, reputation and similarity matter.

### 18.4 Languages, repositories, patents and package namespaces

**KingLang has an exact programming-language collision:** [ShubhamBhut/kinglang](https://github.com/ShubhamBhut/kinglang), created 12 February 2024, describes an interpreted programming language written in Rust. Another [publidoo/kinglang](https://github.com/publidoo/kinglang), created 30 January 2024, describes a language app. These projects need not be large or actively maintained to create search/identity friction. No code was copied, installed or executed.

Chinese patent publication [CN110737431A](https://patents.google.com/patent/CN110737431A/zh), filed 18 September 2019 and published 31 January 2020, describes a software development method/platform using **KingLang**, combining Chinese extensions and C++ and generating C++ business code. Its listed assignee is Shenzhen Jinzheng Science & Technology; a B publication is listed for 14 July 2023. This is evidence that the name has already been used in a programming context. Current enforceability and claim scope were **not independently verified at CNIPA**, and the name reference does not establish patent infringement.

The GitHub public `in:name` searches returned four results for DeluluLang and three for KingLang. This qualifier can also return owner/substrings: those counts are **not counts of separate languages**. Filtering exact repository basenames left the owner's `jessesuniljs1-collab/DeluluLang` for DeluluLang, and the two KingLang repositories above. The test repository is also visible through the broader DeluluLang search. No unrelated exact DeluluLang programming language was identified in this limited search.

Direct official registry API lookups for lowercase **delululang** and **kinglang** returned HTTP **404** on all six endpoints: npm, PyPI and crates.io. This means no package record was returned by those exact requests at that time. It does **not** prove that a registry would allow registration, that a name is unreserved, that related names are clear, or that the CLI name `delulu` is available. Domains and social handles were not reserved or comprehensively checked.

### 18.5 Suggested alternatives

I would prioritize a pronounceable five-to-seven-letter name over an ultra-short crowded dictionary word. The following are **creative candidates with only a preliminary web/GitHub screen**, not cleared brands. No official trademark search was performed for these alternatives.

| Candidate | Suggested pronunciation | Why consider it | Preliminary screen / drawback |
|---|---|---|---|
| **Duvra** | DOOV-ruh | Five letters, compact command name, retains a little of the current name's sound without the overt joke | No exact repository basename in the four GitHub search results; general web results include personal/name uses. Must check similarity, pronunciation and domains. My preferred candidate for the next screening round. |
| **Kevru** | KEV-roo | Five letters, neutral, easy to distinguish from established language names | No exact repository basename in six GitHub results; no obvious software product emerged in the limited web query. Low visibility is not legal availability. |
| **Dulyn** | DOO-lin, if chosen as the brand pronunciation | Short and closer to DeluluLang's existing identity | No exact repository basename in 29 GitHub results. The spelling has existing geographic/language associations, so international pronunciation and meanings need checking. |
| **Veylun** | VAY-lun | More formal sound, six letters | Existing music identity and an exact [GitHub repository](https://github.com/Dabot11/Veylun) appeared. Lower priority than the first two. |
| **Delu / Dulu** | DEH-loo / DOO-loo | Minimal conceptual change and shorter everyday brand | Creative options only; not separately screened. They do not solve the related DELULU trademark question merely by shortening it. |

I screened and set aside several plausible-sounding names because they already have close technology uses: **Tavri** describes governed AI authority for critical systems; **Kavren** describes AI delivery/governance; **Orvyn** appears on several AI assistant/software projects; **Nuvren** is a web-development studio. These examples show why a pleasant coined sound is not enough. [Tavri](https://tavri.com/), [Kavren](https://kavren.framer.ai/), [Orvyn project](https://github.com/BUGHUNTER-SACHIN/ORVYN), [Nuvren](https://www.nuvren.com/about/).

For the final choice, test pronounce-after-reading and spell-after-hearing with a few people across intended language markets; evaluate search results and CLI/package collisions; then clear exact/phonetic/transliterated marks and actual software/AI goods/services. If DeluluLang remains your preference after that, keeping it avoids migration cost and preserves a distinctive identity. The present evidence supports **caution and a deliberate choice**, not an assertion that DeluluLang is forbidden.

### 18.6 If a rename is eventually authorized

Plan a compatibility migration for CLI aliases, file extensions, manifests, package/module identities, artifact magic/versioning, diagnostic links, MCP/LSP names, environment variables, state directories, signing/trust configuration, installer identities, documentation and website redirects. Preserve provenance and licence notices. A display-name change need not immediately change bytecode or state formats. Do not rename the repository, rewrite history or alter origin as a side effect of branding. None of these changes was made in this session.

## Appendix A. Complete executed program sources and inputs

These are actual scratch files, not proposed syntax. They were executed by the commands in Appendix B. Corrected files are the final versions below; the two preserved first drafts and the original package manifest are also included so author errors are distinguishable from product failures. The package-local plugin hosts are retained because their first invocations demonstrate the manifest-context mistake; the top-level plugin hosts are the successful corrected invocations.

Working directory for ordinary probes: `C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work`. Child environment inherited the process environment with `DELULU_HOME` and `DELULU_STATE_DIR` redirected to the scratch `home` and `state`, and `NO_COLOR=1`. No secrets were configured. The execution harness had an outer 25-second timeout; the product's own wall budget stopped the budget probes first. The offline Rust build used the separate environment listed in §2.

Input fixture `input/task.txt` was `value=10\nvalue=11\n` (actual LF newlines). Its negative variants, tested on both engines, were `unknown=3\n`, `value=1001\n`, the empty string, and an absent file. The original contents were restored. `input/child/safe.txt` contained `child-ok`; `forbidden.txt` contained `sentinel-unchanged` and retained those bytes after the out-of-scope tests and at final verification.

The first package manifest contained only `[package]`, name `review_package`, version `0.1.0`, and kind `bin`; it intentionally has no authority table in those first failed invocations. The corrected manifest is reproduced below. The plugin manifest was not widened to make its host work; the host moved outside that package boundary.

### A — args_missing.delulu

Source: [args_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/args_missing.delulu). SHA-256: `9f2e2e770b6c0f32a164d15bfcb75d1edf2ff2ad2935feb14e90d22be37332f9`.

```text
module review.args_missing
fn main(root: Root) { let args = root.args() }
```

### A — basics.delulu

Source: [basics.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/basics.delulu). SHA-256: `3970b06cc6f22aa90975ae3c4282c250c45e5231069a9b15912c2a5f7bf2bccb`.

```text
module review.basics
type Pair { left: Int, right: Int }
type Failure = Bad(Str)
fn identity[T](x: T) -> T { x }
fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
fn positive(s: Str) -> Result[Int, Failure] {
    match parse_int(s) { Some(n) => if n >= 0 { Ok(n) } else { Err(Bad("negative")) }, None => Err(Bad("number")) }
}
fn doubled(s: Str) -> Result[Int, Failure] { let n = positive(s)?
 Ok(n*2) }
fn main(root: Root) ! {Write} {
    let out = root.console()
    var total = 0
    for n in range(0, 5) { total = total + n }
    let p = Pair { left: identity(total), right: fib(10) }
    let xs = [3, 1, 2]
    xs.push(4)
    let m: ref Map[Str, Int] = Map()
    m.insert("answer", p.left + p.right)
    out.println(str(m.get("answer")))
    out.println(str(xs.sort()))
    out.println(" a,b ".trim().replace(",", ":").to_upper())
    out.println(str(doubled("21")))
    out.println(str(doubled("bad")))
}
```

### A — basics_while.delulu

Source: [basics_while.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/basics_while.delulu). SHA-256: `d6532a9c2672204505a3a0d314f50df39299c2fecc61c3610c0aa1ea8dfbba9a`.

```text
module review.basics
type Pair { left: Int, right: Int }
type Failure = Bad(Str)
fn identity[T](x: T) -> T { x }
fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
fn positive(s: Str) -> Result[Int, Failure] {
    match parse_int(s) { Some(n) => if n >= 0 { Ok(n) } else { Err(Bad("negative")) }, None => Err(Bad("number")) }
}
fn doubled(s: Str) -> Result[Int, Failure] { let n = positive(s)?
 Ok(n*2) }
fn main(root: Root) ! {Write} {
    let out = root.console()
    var total = 0
    var n = 0
    while n < 5 { total = total + n
 n = n + 1 }
    let p = Pair { left: identity(total), right: fib(10) }
    let xs = [3, 1, 2]
    xs.push(4)
    let m: ref Map[Str, Int] = Map()
    m.insert("answer", p.left + p.right)
    out.println(str(m.get("answer")))
    out.println(str(xs.sort()))
    out.println(" a,b ".trim().replace(",", ":").to_upper())
    out.println(str(doubled("21")))
    out.println(str(doubled("bad")))
}
```

### A — budget.delulu

Source: [budget.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/budget.delulu). SHA-256: `ddf67ae06d5a9e027b80ab674c2cc425d3ef26cc3e1245ef80f3465c0fcd00d9`.

```text
module review.budget
fn main(root: Root) ! {Write} {
    var i = 0
    while i < 1000000000 { i = i + 1 }
    root.console().println("completed")
}
```

### A — compound_missing.delulu

Source: [compound_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/compound_missing.delulu). SHA-256: `ef5614676653cdbbf4429ec766d3113d65d7cc6462e5b458505d8aeeec9aca59`.

```text
module review.compound_missing
fn main(root: Root) { var n = 1
 n += 2 }
```

### A — delegation.delulu

Source: [delegation.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/delegation.delulu). SHA-256: `722c59495fc013cf4084291f5743299838eb41fedf526484c64d76f9ec1eb5ce`.

```text
module review.delegation
fn child(rd: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { rd.read_text(path) }
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let parent = root.fs_read("./input")
    let delegated = parent.narrow("child")
    match child(delegated, "safe.txt") { Ok(v) => out.println(v), Err(e) => out.println("missing") }
}
```

### A — delegation_denied.delulu

Source: [delegation_denied.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/delegation_denied.delulu). SHA-256: `c710469e6390dec0dfef071dcb312d8dc72af1c09b98489b40b6b9a4522fb8c1`.

```text
module review.delegation
fn child(rd: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { rd.read_text(path) }
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let parent = root.fs_read("./input")
    let delegated = parent.narrow("child")
    match child(delegated, "../task.txt") { Ok(v) => out.println(v), Err(e) => out.println("missing") }
}
```

### A — device_missing.delulu

Source: [device_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/device_missing.delulu). SHA-256: `9fbc943f7e391b8a2bdd227f2b6a5a8404a893f0f4fba204565c0cdc9841a657`.

```text
module review.device_missing
fn main(root: Root) { let arm = root.actuator("review/motor") }
```

### A — device_sim.delulu

Source: [device_sim.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/device_sim.delulu). SHA-256: `29fe566b9bd34b1bccd4965c1ecbd026d924a3d95062815cee768f9d2b484578`.

```text
module review.device_sim
type Cmd { angle_deg: Float }
fn say(r: Result[Unit, ActuateErr]) -> Str {
    match r {
        Ok(u) => "COMMANDED",
        Err(e) => match e {
            Envelope(reason) => "REFUSED: " + reason,
            LeaseRevoked(reason) => "REVOKED: " + reason,
            NoDevice => "NODEVICE"
        }
    }
}
fn main(root: Root) ! {Write, Read, Actuate} {
    let out = root.console()
    let arm = root.actuator("review/elbow")
    out.println(say(arm.command(Cmd {angle_deg:12.0})))
    out.println(say(arm.command(Cmd {angle_deg:999.0})))
    let sensor = root.sensor("review/elbow#angle_deg")
    match sensor.read() {
        Ok(v) => out.println("READ " + str(v)),
        Err(e) => out.println("NO READING")
    }
}
```

### A — dynamic.delulu

Source: [dynamic.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/dynamic.delulu). SHA-256: `d5806563665005d370e47a975380e127fde8a3f31c8f4685216bd82353b87783`.

```text
module review.dynamic
fn scope() -> Str { "./input" }
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let rd = root.fs_read(scope())
    match rd.read_text("task.txt") { Ok(v) => out.println(v.trim()), Err(e) => out.println("missing") }
}
```

### A — edge.delulu

Source: [edge.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/edge.delulu). SHA-256: `b43889eff0fbbb44931bee45637e3c07335cddce09b1f860f21a9d03f1d57817`.

```text
module review.edge
fn aggregate(n: Int) -> Int {
    var i = 0
    var total = 0
    while i < n {
        let reading = i % 101
        if reading <= 90 { total = total + reading }
        i = i + 1
    }
    total
}
fn main(root: Root) ! {Write} { root.console().println(str(aggregate(100000))) }
```

### A — effect_missing.delulu

Source: [effect_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/effect_missing.delulu). SHA-256: `0dafcce97ac7ff73c03e989c8d7354a7480f6ac2d769fa651a23cb832ab74e0a`.

```text
module review.effect_missing
fn main(root: Root) { root.console().println("should not run") }
```

### A — effect_repaired.delulu

Source: [effect_repaired.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/effect_repaired.delulu). SHA-256: `09e9c04ca67cb178f3a7f9ec9fd5c876fccf570061c1fbe1d7e52ffb89b54cc9`.

```text
module review.effect_missing
fn main(root: Root) ! {Write} { root.console().println("should not run") }
```

### A — interpolation.delulu

Source: [interpolation.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/interpolation.delulu). SHA-256: `4475fa2a64c55d92aa1af25602297bd73fdcc2bd2b815aa6758367ef37cb5bce`.

```text
module review.interpolation
fn main(root: Root) ! {Write} { let name = "agent"
 root.console().println("hello {name}") }
```

### A — json_missing.delulu

Source: [json_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/json_missing.delulu). SHA-256: `b4b370ca2b5a9348542a1fe5aa6ce5e549df353879a243a243cd58250a91c8e7`.

```text
module review.json_missing
fn main(root: Root) { let data = parse_json("{}") }
```

### A — math_missing.delulu

Source: [math_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/math_missing.delulu). SHA-256: `7e464a838524462d851bf9bcde451fa2d389adf30ac553f7a66435068aa78ece`.

```text
module review.math_missing
fn main(root: Root) { let n = sqrt(4.0) }
```

### A — method_missing.delulu

Source: [method_missing.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/method_missing.delulu). SHA-256: `354c27eda5fa6eb92862cc8f5546b9d2ecda83ea05afa94ec5f49bed8551c140`.

```text
module review.method_missing
type Point { x: Int }
fn main(root: Root) { let p = Point{x:1}
 let n = p.norm() }
```

### A — outside.delulu

Source: [outside.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/outside.delulu). SHA-256: `9f3e74e776b00b60662595effc648446ed21d72d068ba573a67a12b42a482a34`.

```text
module review.outside
fn main(root: Root) ! {Write} {
    let wr = root.fs_write("./output")
    let _result = wr.write_text("../forbidden.txt", "changed")
}
```

### A — package/src/helpers.delulu

Source: [helpers.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/package/src/helpers.delulu). SHA-256: `da53d590b75da8302a7bd9a90eb091fb53f503bb644b478587e83483e1f99e6b`.

```text
module review_package.helpers
pub fn twice(n: Int) -> Int { n * 2 }
```

### A — package/src/main.delulu

Source: [main.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/package/src/main.delulu). SHA-256: `b6d82e8fee7a9d6c420b1126d328568c32288d202f9a4b210ae5c6e1ab7323d2`.

```text
module review_package
import review_package.helpers
fn main(root: Root) ! {Write} { root.console().println(str(twice(21))) }
```

### A — plugin/host.delulu

Source: [host.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/plugin/host.delulu). SHA-256: `4bbbe55dbb757e4adaea80013d28922548758b9f58c75c73b68c233c60a870dd`.

```text
module host

// The host half of the flagship plugin example. `shout` is a THIRD-PARTY text transform whose
// authority ceiling is empty: its type is a pure `fn(Str) -> Str`, it takes no capability, and the
// type system — re-checked at load — forbids it from reading a file, telling the time, or reaching
// the network. There is nothing to trust here; the guarantee is by construction.
//
// Run it (from this directory, after `delulu plugin build .`):
//
//     delulu run host.delulu --grant console --grant plugin=.
//     hello!
//
// Two things worth trying, because they are the arrangement rather than footnotes to it. Drop the
// `--grant plugin=.` and the program cannot even obtain the capability: DL0703, naming the flag. Point
// the path outside the granted directory and the load is refused before a byte is read — a grant is a
// path spelling too.

// The load lives in its own `Result`-returning function so `?` can carry each refusal out. Every one
// of them is a VALUE the host can decide about, never a crash: that is the point of a load sequence.
fn shouted(h: Cap[PluginHost]) -> Result[Str, PluginErr] ! {Load, Read} {
    // A `Grant` is an ordinary record: it DESCRIBES authority and confers none. Conferral happens at
    // `load`, where it is intersected with the artifact's own ceiling — so asking for more than the
    // plugin declares is refused rather than granted. This one asks for nothing, which is all a pure
    // text transform can ever need.
    let g = Grant {
        effects: [],
        fs_read: [],
        fs_write: [],
        net: [],
        secrets: [],
        declassify: [],
        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },
        require_signed: false,
    }
    let p = load(h, "shout.dpx", g)?
    // The annotation form. The spec writes `p.get[F]`; that bracket notation does not parse, and the
    // annotation is what pins `F` — see `delulu explain E-PLUGIN`.
    let f: fn(Str) -> Str ! {} = p.get("shout")?
    // Calling a pure export adds nothing to this function's row, which is why `shouted` is
    // `!{Load, Read}` and not more: the plugin's type is the whole of what it may do.
    Ok(f("hello"))
}

fn main(root: Root) ! {Load, Read, Write} {
    let out = root.console()
    let h = root.plugin_host()
    match shouted(h) {
        Ok(s) => out.println(s)
        Err(e) => match e { NotGranted(m) => out.println("not granted: " + m), VerifyFailed(m) => out.println("verification failed: " + m), BadArtifact(m) => out.println("bad artifact: " + m), Revoked(n) => out.println("this plugin's authority was revoked"), LimitExceeded(m) => out.println("a resource limit fired: " + m), ApiMismatch(m) => out.println("plugin ABI mismatch: " + m) }
    }
}
```

### A — plugin/overbroad.delulu

Source: [overbroad.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/plugin/overbroad.delulu). SHA-256: `30c6f41e223357249a153aa367b67a611dc1775f82c33e5c1f2878568d999d05`.

```text
module host

// The host half of the flagship plugin example. `shout` is a THIRD-PARTY text transform whose
// authority ceiling is empty: its type is a pure `fn(Str) -> Str`, it takes no capability, and the
// type system â€” re-checked at load â€” forbids it from reading a file, telling the time, or reaching
// the network. There is nothing to trust here; the guarantee is by construction.
//
// Run it (from this directory, after `delulu plugin build .`):
//
//     delulu run host.delulu --grant console --grant plugin=.
//     hello!
//
// Two things worth trying, because they are the arrangement rather than footnotes to it. Drop the
// `--grant plugin=.` and the program cannot even obtain the capability: DL0703, naming the flag. Point
// the path outside the granted directory and the load is refused before a byte is read â€” a grant is a
// path spelling too.

// The load lives in its own `Result`-returning function so `?` can carry each refusal out. Every one
// of them is a VALUE the host can decide about, never a crash: that is the point of a load sequence.
fn shouted(h: Cap[PluginHost]) -> Result[Str, PluginErr] ! {Load, Read} {
    // A `Grant` is an ordinary record: it DESCRIBES authority and confers none. Conferral happens at
    // `load`, where it is intersected with the artifact's own ceiling â€” so asking for more than the
    // plugin declares is refused rather than granted. This one asks for nothing, which is all a pure
    // text transform can ever need.
    let g = Grant {
        effects: ["Write"],
        fs_read: [],
        fs_write: [],
        net: [],
        secrets: [],
        declassify: [],
        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },
        require_signed: false,
    }
    let p = load(h, "shout.dpx", g)?
    // The annotation form. The spec writes `p.get[F]`; that bracket notation does not parse, and the
    // annotation is what pins `F` â€” see `delulu explain E-PLUGIN`.
    let f: fn(Str) -> Str ! {} = p.get("shout")?
    // Calling a pure export adds nothing to this function's row, which is why `shouted` is
    // `!{Load, Read}` and not more: the plugin's type is the whole of what it may do.
    Ok(f("hello"))
}

fn main(root: Root) ! {Load, Read, Write} {
    let out = root.console()
    let h = root.plugin_host()
    match shouted(h) {
        Ok(s) => out.println(s)
        Err(e) => match e { NotGranted(m) => out.println("not granted: " + m), VerifyFailed(m) => out.println("verification failed: " + m), BadArtifact(m) => out.println("bad artifact: " + m), Revoked(n) => out.println("this plugin's authority was revoked"), LimitExceeded(m) => out.println("a resource limit fired: " + m), ApiMismatch(m) => out.println("plugin ABI mismatch: " + m) }
    }
}
```

### A — plugin/src/shout.delulu

Source: [shout.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/plugin/src/shout.delulu). SHA-256: `ca70a06e21e07c291155e49420fb440b8c178b839cb00dd17ae6dab650c824e4`.

```text
module shout
pub fn shout(s: Str) -> Str { s + "!" }
```

### A — plugin_host.delulu

Source: [plugin_host.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/plugin_host.delulu). SHA-256: `7def42bba02a8907be5b33358e8a815cf25b4332fdc0cfb9eeea8f492849ee7b`.

```text
module host

// The host half of the flagship plugin example. `shout` is a THIRD-PARTY text transform whose
// authority ceiling is empty: its type is a pure `fn(Str) -> Str`, it takes no capability, and the
// type system â€” re-checked at load â€” forbids it from reading a file, telling the time, or reaching
// the network. There is nothing to trust here; the guarantee is by construction.
//
// Run it (from this directory, after `delulu plugin build .`):
//
//     delulu run host.delulu --grant console --grant plugin=.
//     hello!
//
// Two things worth trying, because they are the arrangement rather than footnotes to it. Drop the
// `--grant plugin=.` and the program cannot even obtain the capability: DL0703, naming the flag. Point
// the path outside the granted directory and the load is refused before a byte is read â€” a grant is a
// path spelling too.

// The load lives in its own `Result`-returning function so `?` can carry each refusal out. Every one
// of them is a VALUE the host can decide about, never a crash: that is the point of a load sequence.
fn shouted(h: Cap[PluginHost]) -> Result[Str, PluginErr] ! {Load, Read} {
    // A `Grant` is an ordinary record: it DESCRIBES authority and confers none. Conferral happens at
    // `load`, where it is intersected with the artifact's own ceiling â€” so asking for more than the
    // plugin declares is refused rather than granted. This one asks for nothing, which is all a pure
    // text transform can ever need.
    let g = Grant {
        effects: [],
        fs_read: [],
        fs_write: [],
        net: [],
        secrets: [],
        declassify: [],
        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },
        require_signed: false,
    }
    let p = load(h, "plugin/shout.dpx", g)?
    // The annotation form. The spec writes `p.get[F]`; that bracket notation does not parse, and the
    // annotation is what pins `F` â€” see `delulu explain E-PLUGIN`.
    let f: fn(Str) -> Str ! {} = p.get("shout")?
    // Calling a pure export adds nothing to this function's row, which is why `shouted` is
    // `!{Load, Read}` and not more: the plugin's type is the whole of what it may do.
    Ok(f("hello"))
}

fn main(root: Root) ! {Load, Read, Write} {
    let out = root.console()
    let h = root.plugin_host()
    match shouted(h) {
        Ok(s) => out.println(s)
        Err(e) => match e { NotGranted(m) => out.println("not granted: " + m), VerifyFailed(m) => out.println("verification failed: " + m), BadArtifact(m) => out.println("bad artifact: " + m), Revoked(n) => out.println("this plugin's authority was revoked"), LimitExceeded(m) => out.println("a resource limit fired: " + m), ApiMismatch(m) => out.println("plugin ABI mismatch: " + m) }
    }
}
```

### A — plugin_overbroad.delulu

Source: [plugin_overbroad.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/plugin_overbroad.delulu). SHA-256: `a22de154bcf40163cb0844e39adae82f5792dfb50af9a7b8c65af680733a4450`.

```text
module host

// The host half of the flagship plugin example. `shout` is a THIRD-PARTY text transform whose
// authority ceiling is empty: its type is a pure `fn(Str) -> Str`, it takes no capability, and the
// type system â€” re-checked at load â€” forbids it from reading a file, telling the time, or reaching
// the network. There is nothing to trust here; the guarantee is by construction.
//
// Run it (from this directory, after `delulu plugin build .`):
//
//     delulu run host.delulu --grant console --grant plugin=.
//     hello!
//
// Two things worth trying, because they are the arrangement rather than footnotes to it. Drop the
// `--grant plugin=.` and the program cannot even obtain the capability: DL0703, naming the flag. Point
// the path outside the granted directory and the load is refused before a byte is read â€” a grant is a
// path spelling too.

// The load lives in its own `Result`-returning function so `?` can carry each refusal out. Every one
// of them is a VALUE the host can decide about, never a crash: that is the point of a load sequence.
fn shouted(h: Cap[PluginHost]) -> Result[Str, PluginErr] ! {Load, Read} {
    // A `Grant` is an ordinary record: it DESCRIBES authority and confers none. Conferral happens at
    // `load`, where it is intersected with the artifact's own ceiling â€” so asking for more than the
    // plugin declares is refused rather than granted. This one asks for nothing, which is all a pure
    // text transform can ever need.
    let g = Grant {
        effects: ["Write"],
        fs_read: [],
        fs_write: [],
        net: [],
        secrets: [],
        declassify: [],
        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },
        require_signed: false,
    }
    let p = load(h, "plugin/shout.dpx", g)?
    // The annotation form. The spec writes `p.get[F]`; that bracket notation does not parse, and the
    // annotation is what pins `F` â€” see `delulu explain E-PLUGIN`.
    let f: fn(Str) -> Str ! {} = p.get("shout")?
    // Calling a pure export adds nothing to this function's row, which is why `shouted` is
    // `!{Load, Read}` and not more: the plugin's type is the whole of what it may do.
    Ok(f("hello"))
}

fn main(root: Root) ! {Load, Read, Write} {
    let out = root.console()
    let h = root.plugin_host()
    match shouted(h) {
        Ok(s) => out.println(s)
        Err(e) => match e { NotGranted(m) => out.println("not granted: " + m), VerifyFailed(m) => out.println("verification failed: " + m), BadArtifact(m) => out.println("bad artifact: " + m), Revoked(n) => out.println("this plugin's authority was revoked"), LimitExceeded(m) => out.println("a resource limit fired: " + m), ApiMismatch(m) => out.println("plugin ABI mismatch: " + m) }
    }
}
```

### A — simulation.delulu

Source: [simulation.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/simulation.delulu). SHA-256: `b4040827dbd0cc1a4b5d494ba1b6aac5314344bc256a87f7b96bb600710e22b2`.

```text
module review.simulation
type Sample { age: Int, value: Int, stop: Bool }
fn proposed(s: Sample) -> Str {
    if s.stop { "STOP:estop" } else {
        if s.age > 100 { "STOP:stale" } else {
            if s.value < 0 || s.value > 100 { "STOP:invalid" } else { "MOVE:" + str(s.value) }
        }
    }
}
fn mission(battery: Int, fence: Bool, stage: Int) -> Int {
    if battery < 20 || !fence { 9 } else { if stage < 2 { stage + 1 } else { stage } }
}
fn main(root: Root) ! {Write} {
    let out = root.console()
    let samples: val List[Sample] = [Sample{age:10,value:40,stop:false}, Sample{age:200,value:40,stop:false}, Sample{age:5,value:-1,stop:false}, Sample{age:0,value:40,stop:true}]
    for s in samples { out.println(proposed(s)) }
    var stage = 0
    stage = mission(80, true, stage)
    out.println("mission:" + str(stage))
    stage = mission(10, true, stage)
    out.println("mission:" + str(stage))
    let sequence: val List[Int] = [1, 2, 2, 1, 3, 4, 5]
    let queue = []
    var last = 0
    var stale = 0
    var dropped = 0
    for seq in sequence {
        if seq <= last { stale = stale + 1 } else {
            last = seq
            if queue.len() < 3 { queue.push(seq) } else { dropped = dropped + 1 }
        }
    }
    out.println("telemetry:" + str(queue.len()) + ":" + str(stale) + ":" + str(dropped))
}
```

### A — utility.delulu

Source: [utility.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/utility.delulu). SHA-256: `68dc8ade0d5d692a05ca09b20fbe5cfb74dccbdda5337c1eeacbac9efdcc18e4`.

```text
module review.utility
type TaskErr = Input | Format | Range
fn evaluate(text: Str) -> Result[Int, TaskErr] {
    var sum = 0
    var count = 0
    for row in text.split("\n") {
        let line = row.trim()
        if line.len() > 0 {
            let parts = line.split("=")
            if parts.len() != 2 { return Err(Format) }
            let key = match parts.get(0) { Some(v) => v, None => "" }
            let value = match parts.get(1) { Some(v) => v, None => "" }
            if key != "value" { return Err(Format) }
            match parse_int(value) {
                Some(n) => {
                    if n < 0 || n > 1000 { return Err(Range) }
                    sum = sum + n
                    count = count + 1
                    if count > 32 { return Err(Range) }
                }
                None => return Err(Format)
            }
        }
    }
    if count == 0 { Err(Format) } else { Ok(sum * 2) }
}
fn task(rd: Cap[FsRead]) -> Result[Int, TaskErr] ! {Read} {
    match rd.read_text("task.txt") { Ok(text) => evaluate(text), Err(e) => Err(Input) }
}
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let rd = root.fs_read("./input")
    match task(rd) {
        Ok(n) => {
            let wr = root.fs_write("./output")
            let answer = "{\"ok\":true,\"value\":" + str(n) + "}"
            match wr.write_text("answer.json", answer) {
                Ok(u) => out.println(answer)
                Err(e) => out.println("{\"ok\":false,\"error\":\"write\"}")
            }
        }
        Err(e) => match e {
            Input => out.println("{\"ok\":false,\"error\":\"input\"}")
            Format => out.println("{\"ok\":false,\"error\":\"format\"}")
            Range => out.println("{\"ok\":false,\"error\":\"range\"}")
        }
    }
}
```

### A — workflow.delulu

Source: [workflow.delulu](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/workflow.delulu). SHA-256: `b32e01c8358b0ae1f8e6f72568b38c195d1a6781ca0067d894dcdafc13267a52`.

```text
module review.workflow
fn attempt(step: Int, retry: Int) -> Result[Int, Str] {
    if step == 1 && retry == 0 { Err("transient") } else { Ok(step + 10) }
}
fn main(root: Root) ! {Write} {
    let out = root.console()
    var step = 0
    var total = 0
    while step < 3 {
        var retry = 0
        var done = false
        while retry < 2 && !done {
            match attempt(step, retry) {
                Ok(n) => { total = total + n
 done = true
 out.println("done:" + str(step) + ":" + str(retry)) }
                Err(e) => out.println("retry:" + str(step) + ":" + str(retry))
            }
            retry = retry + 1
        }
        if !done { out.println("failed")
 return }
        step = step + 1
    }
    out.println("total:" + str(total))
}
```

### A — package/delulu.toml

Source: [delulu.toml](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/package/delulu.toml). SHA-256: `25141340305e133a851406b306bfa7c3e48be78d2ffd6fa2571f88dc34a091c2`.

```toml
[package]
name="review_package"
version="0.1.0"
kind="bin"

[authority]
effects=["Write"]
```

### A — plugin/delulu.toml

Source: [delulu.toml](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/plugin/delulu.toml). SHA-256: `a05b4a44bee5165d45cd46d4faf7290800cfb7a417fd853cb9a3aff7ea5bd837`.

```toml
[package]
name="shout"
version="0.1.0"
kind="plugin"
[plugin]
api=1
class="verified"
[plugin.authority]
effects=[]
requires=[]
[plugin.exports]
shout="fn(Str) -> Str"
```

### A — basics.first-draft.txt

Source: [basics.first-draft.txt](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/basics.first-draft.txt). SHA-256: `4ef36025b97292a01fd315eb3df5220e405cc83c41447fe50436c9e39b770b7e`.

```text
module review.basics
type Pair { left: Int, right: Int }
type Failure = Bad(Str)
fn identity[T](x: T) -> T { x }
fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
fn positive(s: Str) -> Result[Int, Failure] {
    match parse_int(s) { Some(n) => if n >= 0 { Ok(n) } else { Err(Bad("negative")) }, None => Err(Bad("number")) }
}
fn doubled(s: Str) -> Result[Int, Failure] { let n = positive(s)?
 Ok(n*2) }
fn main(root: Root) ! {Write} {
    let out = root.console()
    var total = 0
    for n in range(0, 5) { total = total + n }
    let p = Pair { left: identity(total), right: fib(10) }
    let xs = [3, 1, 2]
    xs.push(4)
    let m: Map[Str, Int] = Map()
    m.insert("answer", p.left + p.right)
    out.println(str(m.get("answer")))
    out.println(str(xs.sort()))
    out.println(" a,b ".trim().replace(",", ":").to_upper())
    out.println(str(doubled("21")))
    out.println(str(doubled("bad")))
}
```

### A — outside.first-draft.txt

Source: [outside.first-draft.txt](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/outside.first-draft.txt). SHA-256: `62e0b84c9790ae4baeb235f153d42cdeed9f2859dd7faa81ea033300cb28848c`.

```text
module review.outside
fn main(root: Root) ! {Write} {
    let wr = root.fs_write("./output")
    wr.write_text("../forbidden.txt", "changed")
}
```

### A — utility.py

Source: [utility.py](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/utility.py). SHA-256: `8ebf7b77adc6fc06d0d98b544b82b08f5ca0c35d879600d0bfd05316643168e7`.

```python
from pathlib import Path
import json
text=Path("input/task.txt").read_text()
total=0
count=0
for row in text.split("\n"):
    line=row.strip()
    if line:
        parts=line.split("=")
        if len(parts)!=2 or parts[0]!="value":raise ValueError("format")
        n=int(parts[1])
        if not 0<=n<=1000:raise ValueError("range")
        total+=n
        count+=1
        if count>32:raise ValueError("range")
if count==0:raise ValueError("format")
answer=json.dumps(dict(ok=True,value=total*2),separators=(",",":"))
Path("output/python-answer.json").write_text(answer)
print(answer)
```

### A — edge.py

Source: [edge.py](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/edge.py). SHA-256: `ef6b981d90c00b3b85eb8d185920707aebe4dfc225ca0a73b0e88a07254cb865`.

```python
n=100000
total=0
for i in range(n):
    reading=i%101
    if reading<=90:total+=reading
print(total)
```

### A — edge.rs

Source: [edge.rs](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/edge.rs). SHA-256: `58ff423c9b004f4d4b0592e8c44e54b7f04db8de33814a0a4c2645f5a124e5f4`.

```rust
fn main() { let n=std::hint::black_box(100000i64); let mut total=0i64; for i in 0..n { let r=i%101; if r<=90 { total+=r; } } println!("{}", std::hint::black_box(total)); }
```

## Appendix B. Exact command and outcome ledger

Each block preserves the process argv as a JSON array, which avoids PowerShell quoting ambiguity. `cwd` is the actual working directory. Stdout/stderr are the captured streams; an empty stream is stated explicitly. The source, input changes, expected semantics and limitations are in §4/§16/Appendix A. A nonzero exit on an intentional negative probe is expected evidence, not an unsuccessful review. Labels containing `corrected` supersede setup errors while retaining them in the record.

Large discovery/Survey responses are linked to the raw local result log rather than copied in full; their exact command and exit are still included. These scratch records are supplemental evidence and may disappear if the operating system clears temporary files. The complete meaningful program sources and behavioral outputs are embedded in this report.

<details>
<summary>B001 — fresh-version: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "--version",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 780.0058,
  "timed_out": false,
  "peak_working_set": 6877184
}
```

stdout:

````text
{
  "command": "version",
  "commit": "e97c74798fed941a369bbc931ed9d0a8c462753c",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  },
  "target": "x86_64-pc-windows-msvc"
}
````

stderr: empty.

</details>

<details>
<summary>B002 — basics-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "basics.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 30.5827,
  "timed_out": false,
  "peak_working_set": 7925760
}
```

stdout: empty.

stderr:

````text
error[DL1604]: cannot write a map entry (insert) through a `val` receiver — writes require `iso`, `trn`, or `ref`: `m`'s written type names no reference capability, which reads as `val`. Write `ref` before it to write through `m` (`let m: ref …`), or leave the type out
  --> basics.delulu:19:5
   |
19 |     m.insert("answer", p.left + p.right)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ write refused
  --- basics.delulu:18:12
   |
18 |     let m: Map[Str, Int] = Map()
   |            ------------- `m` is `val` by this type
  explain: delulu explain E-DL1604
````

</details>

<details>
<summary>B003 — basics-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "basics.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 17.7688,
  "timed_out": false,
  "peak_working_set": 7942144
}
```

stdout: empty.

stderr:

````text
error[DL1604]: cannot write a map entry (insert) through a `val` receiver — writes require `iso`, `trn`, or `ref`: `m`'s written type names no reference capability, which reads as `val`. Write `ref` before it to write through `m` (`let m: ref …`), or leave the type out
  --> basics.delulu:19:5
   |
19 |     m.insert("answer", p.left + p.right)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ write refused
  --- basics.delulu:18:12
   |
18 |     let m: Map[Str, Int] = Map()
   |            ------------- `m` is `val` by this type
  explain: delulu explain E-DL1604
````

</details>

<details>
<summary>B004 — utility-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 44.309,
  "timed_out": false,
  "peak_working_set": 8589312
}
```

stdout:

````text
{"ok":true,"value":42}
````

stderr: empty.

</details>

<details>
<summary>B005 — utility-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 30.2116,
  "timed_out": false,
  "peak_working_set": 18239488
}
```

stdout:

````text
{"ok":true,"value":42}
````

stderr: empty.

</details>

<details>
<summary>B006 — workflow-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "workflow.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 30.7419,
  "timed_out": false,
  "peak_working_set": 8376320
}
```

stdout:

````text
done:0:0
retry:1:0
done:1:1
done:2:0
total:33
````

stderr: empty.

</details>

<details>
<summary>B007 — workflow-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "workflow.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 22.1915,
  "timed_out": false,
  "peak_working_set": 17088512
}
```

stdout:

````text
done:0:0
retry:1:0
done:1:1
done:2:0
total:33
````

stderr: empty.

</details>

<details>
<summary>B008 — delegation-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "delegation.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 39.3841,
  "timed_out": false,
  "peak_working_set": 8396800
}
```

stdout:

````text
child-ok
````

stderr: empty.

</details>

<details>
<summary>B009 — delegation-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "delegation.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 24.1917,
  "timed_out": false,
  "peak_working_set": 16437248
}
```

stdout:

````text
child-ok
````

stderr: empty.

</details>

<details>
<summary>B010 — outside-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "outside.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.write=./output"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 51.5861,
  "timed_out": false,
  "peak_working_set": 7659520
}
```

stdout: empty.

stderr:

````text
error[DL0401]: function body type must match the return type: expected `Unit`, found `Result[Unit, IoErr]`
  --> outside.delulu:2:31
   |
 2 | fn main(root: Root) ! {Write} {
   |                               ^ type mismatch here
  explain: delulu explain E-DL0401
````

</details>

<details>
<summary>B011 — outside-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "outside.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.write=./output"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 15.1935,
  "timed_out": false,
  "peak_working_set": 7626752
}
```

stdout: empty.

stderr:

````text
error[DL0401]: function body type must match the return type: expected `Unit`, found `Result[Unit, IoErr]`
  --> outside.delulu:2:31
   |
 2 | fn main(root: Root) ! {Write} {
   |                               ^ type mismatch here
  explain: delulu explain E-DL0401
````

</details>

<details>
<summary>B012 — dynamic-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "dynamic.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 29.6808,
  "timed_out": false,
  "peak_working_set": 8384512
}
```

stdout:

````text
value=10

value=11
````

stderr: empty.

</details>

<details>
<summary>B013 — dynamic-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "dynamic.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 22.0864,
  "timed_out": false,
  "peak_working_set": 16457728
}
```

stdout:

````text
value=10

value=11
````

stderr: empty.

</details>

<details>
<summary>B014 — edge-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "edge.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 71.934,
  "timed_out": false,
  "peak_working_set": 8232960
}
```

stdout:

````text
4054095
````

stderr: empty.

</details>

<details>
<summary>B015 — edge-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "edge.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 21.098,
  "timed_out": false,
  "peak_working_set": 16809984
}
```

stdout:

````text
4054095
````

stderr: empty.

</details>

<details>
<summary>B016 — simulation-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "simulation.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 37.65,
  "timed_out": false,
  "peak_working_set": 8368128
}
```

stdout:

````text
MOVE:40
STOP:stale
STOP:invalid
STOP:estop
mission:1
mission:9
telemetry:3:2:2
````

stderr: empty.

</details>

<details>
<summary>B017 — simulation-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "simulation.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 22.1659,
  "timed_out": false,
  "peak_working_set": 17256448
}
```

stdout:

````text
MOVE:40
STOP:stale
STOP:invalid
STOP:estop
mission:1
mission:9
telemetry:3:2:2
````

stderr: empty.

</details>

<details>
<summary>B018 — device_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "device_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 25.9848,
  "timed_out": false,
  "peak_working_set": 8183808
}
```

stdout: empty.

stderr:

````text
error[DL0703]: actuator `review/motor` was not granted — pass `--grant "actuator=review/motor:DIM=LO..HI"` with the envelope this machine may move in
  --> device_missing.delulu:2:33
   |
 2 | fn main(root: Root) { let arm = root.actuator("review/motor") }
   |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ runtime fault here
  explain: delulu explain E-DL0703
````

</details>

<details>
<summary>B019 — device_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "device_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 16.0755,
  "timed_out": false,
  "peak_working_set": 8138752
}
```

stdout: empty.

stderr:

````text
error[DL1201]: WASM codegen does not support the method `.actuator` — omit `--engine wasm` to run it on the interpreter
  --> device_missing.delulu:2:33
   |
 2 | fn main(root: Root) { let arm = root.actuator("review/motor") }
   |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the WASM backend cannot compile this
  explain: delulu explain E-DL1201
````

</details>

<details>
<summary>B020 — args_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "args_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 24.5268,
  "timed_out": false,
  "peak_working_set": 7577600
}
```

stdout: empty.

stderr:

````text
error[DL0405]: type `Root` has no method `args`
  --> args_missing.delulu:2:39
   |
 2 | fn main(root: Root) { let args = root.args() }
   |                                       ^^^^ unknown method
  explain: delulu explain E-DL0405

warning[DL0414]: `args` is bound here and never read
  --> args_missing.delulu:2:27
   |
 2 | fn main(root: Root) { let args = root.args() }
   |                           ^^^^ never read: remove it, or name it `_args` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B021 — args_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "args_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 18.1806,
  "timed_out": false,
  "peak_working_set": 7557120
}
```

stdout: empty.

stderr:

````text
error[DL0405]: type `Root` has no method `args`
  --> args_missing.delulu:2:39
   |
 2 | fn main(root: Root) { let args = root.args() }
   |                                       ^^^^ unknown method
  explain: delulu explain E-DL0405

warning[DL0414]: `args` is bound here and never read
  --> args_missing.delulu:2:27
   |
 2 | fn main(root: Root) { let args = root.args() }
   |                           ^^^^ never read: remove it, or name it `_args` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B022 — json_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "json_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 25.9651,
  "timed_out": false,
  "peak_working_set": 7553024
}
```

stdout: empty.

stderr:

````text
error[DL0301]: unknown name `parse_json`
  --> json_missing.delulu:2:34
   |
 2 | fn main(root: Root) { let data = parse_json("{}") }
   |                                  ^^^^^^^^^^ not found in this scope
  explain: delulu explain E-DL0301

warning[DL0414]: `data` is bound here and never read
  --> json_missing.delulu:2:27
   |
 2 | fn main(root: Root) { let data = parse_json("{}") }
   |                           ^^^^ never read: remove it, or name it `_data` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B023 — json_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "json_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 16.1822,
  "timed_out": false,
  "peak_working_set": 7557120
}
```

stdout: empty.

stderr:

````text
error[DL0301]: unknown name `parse_json`
  --> json_missing.delulu:2:34
   |
 2 | fn main(root: Root) { let data = parse_json("{}") }
   |                                  ^^^^^^^^^^ not found in this scope
  explain: delulu explain E-DL0301

warning[DL0414]: `data` is bound here and never read
  --> json_missing.delulu:2:27
   |
 2 | fn main(root: Root) { let data = parse_json("{}") }
   |                           ^^^^ never read: remove it, or name it `_data` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B024 — math_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "math_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 26.9218,
  "timed_out": false,
  "peak_working_set": 7569408
}
```

stdout: empty.

stderr:

````text
error[DL0301]: unknown name `sqrt`
  --> math_missing.delulu:2:31
   |
 2 | fn main(root: Root) { let n = sqrt(4.0) }
   |                               ^^^^ not found in this scope
  explain: delulu explain E-DL0301

warning[DL0414]: `n` is bound here and never read
  --> math_missing.delulu:2:27
   |
 2 | fn main(root: Root) { let n = sqrt(4.0) }
   |                           ^ never read: remove it, or name it `_n` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B025 — math_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "math_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 17.6672,
  "timed_out": false,
  "peak_working_set": 7557120
}
```

stdout: empty.

stderr:

````text
error[DL0301]: unknown name `sqrt`
  --> math_missing.delulu:2:31
   |
 2 | fn main(root: Root) { let n = sqrt(4.0) }
   |                               ^^^^ not found in this scope
  explain: delulu explain E-DL0301

warning[DL0414]: `n` is bound here and never read
  --> math_missing.delulu:2:27
   |
 2 | fn main(root: Root) { let n = sqrt(4.0) }
   |                           ^ never read: remove it, or name it `_n` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B026 — compound_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "compound_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 24.9902,
  "timed_out": false,
  "peak_working_set": 7516160
}
```

stdout: empty.

stderr:

````text
error[DL0202]: expected an expression, found `=`
  --> compound_missing.delulu:3:5
   |
 3 |  n += 2 }
   |     ^ expected an expression
  explain: delulu explain E-DL0202

warning[DL0414]: `n` is bound here and never read
  --> compound_missing.delulu:2:27
   |
 2 | fn main(root: Root) { var n = 1
   |                           ^ never read: remove it, or name it `_n` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B027 — compound_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "compound_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 13.6381,
  "timed_out": false,
  "peak_working_set": 7516160
}
```

stdout: empty.

stderr:

````text
error[DL0202]: expected an expression, found `=`
  --> compound_missing.delulu:3:5
   |
 3 |  n += 2 }
   |     ^ expected an expression
  explain: delulu explain E-DL0202

warning[DL0414]: `n` is bound here and never read
  --> compound_missing.delulu:2:27
   |
 2 | fn main(root: Root) { var n = 1
   |                           ^ never read: remove it, or name it `_n` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B028 — method_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "method_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 30.1412,
  "timed_out": false,
  "peak_working_set": 7610368
}
```

stdout: empty.

stderr:

````text
error[DL0405]: type `Point` has no method `norm`
  --> method_missing.delulu:4:12
   |
 4 |  let n = p.norm() }
   |            ^^^^ unknown method
  explain: delulu explain E-DL0405

warning[DL0414]: `n` is bound here and never read
  --> method_missing.delulu:4:6
   |
 4 |  let n = p.norm() }
   |      ^ never read: remove it, or name it `_n` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B029 — method_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "method_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 19.9418,
  "timed_out": false,
  "peak_working_set": 7614464
}
```

stdout: empty.

stderr:

````text
error[DL0405]: type `Point` has no method `norm`
  --> method_missing.delulu:4:12
   |
 4 |  let n = p.norm() }
   |            ^^^^ unknown method
  explain: delulu explain E-DL0405

warning[DL0414]: `n` is bound here and never read
  --> method_missing.delulu:4:6
   |
 4 |  let n = p.norm() }
   |      ^ never read: remove it, or name it `_n` to say so
  repair: prefix-underscore (safe)  [requires human decision]
  explain: delulu explain E-DL0414
````

</details>

<details>
<summary>B030 — interpolation-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "interpolation.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 31.0704,
  "timed_out": false,
  "peak_working_set": 8081408
}
```

stdout:

````text
hello {name}
````

stderr: empty.

</details>

<details>
<summary>B031 — interpolation-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "interpolation.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 24.4167,
  "timed_out": false,
  "peak_working_set": 16023552
}
```

stdout:

````text
hello {name}
````

stderr: empty.

</details>

<details>
<summary>B032 — effect_missing-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "effect_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 29.621,
  "timed_out": false,
  "peak_working_set": 7589888
}
```

stdout: empty.

stderr:

````text
error[DL0501]: function `main` performs effect `Write` not declared in its row
  --> effect_missing.delulu:2:4
   |
 2 | fn main(root: Root) { root.console().println("should not run") }
   |    ^^^^ declared row is here
  repair: add_effect_to_row (exact)  [widens authority — review before applying]
  explain: delulu explain E-DL0501
````

</details>

<details>
<summary>B033 — effect_missing-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "effect_missing.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 18.1905,
  "timed_out": false,
  "peak_working_set": 7598080
}
```

stdout: empty.

stderr:

````text
error[DL0501]: function `main` performs effect `Write` not declared in its row
  --> effect_missing.delulu:2:4
   |
 2 | fn main(root: Root) { root.console().println("should not run") }
   |    ^^^^ declared row is here
  repair: add_effect_to_row (exact)  [widens authority — review before applying]
  explain: delulu explain E-DL0501
````

</details>

<details>
<summary>B034 — basics-corrected-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "basics.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 37.5002,
  "timed_out": false,
  "peak_working_set": 8560640
}
```

stdout:

````text
Some(65)
[1, 2, 3, 4]
A:B
Ok(42)
Err(Bad(number))
````

stderr: empty.

</details>

<details>
<summary>B035 — outside-corrected-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "outside.delulu",
    "--grant",
    "fs.write=./output",
    "--json",
    "--report-out",
    "reports/outside-interp.json",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 37.5693,
  "timed_out": false,
  "peak_working_set": 8380416
}
```

stdout:

````text
{
  "command": "run",
  "delulu_version": "1.0.0",
  "diagnostics": [
    {
      "code": "DL0904",
      "explanation_id": "E-DL0904",
      "message": "path `../forbidden.txt` escapes the granted scope",
      "repairs": [],
      "severity": "error",
      "spans": [
        {
          "end": {
            "byte": 158,
            "col": 63,
            "line": 4
          },
          "file": "outside.delulu",
          "label": "runtime fault here",
          "start": {
            "byte": 114,
            "col": 19,
            "line": 4
          }
        }
      ]
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 1,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B036 — child-outside-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "delegation_denied.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 35.2237,
  "timed_out": false,
  "peak_working_set": 8486912
}
```

stdout: empty.

stderr:

````text
error[DL0904]: path `../task.txt` escapes the granted scope
  --> delegation_denied.delulu:2:71
   |
 2 | fn child(rd: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { rd.read_text(path) }
   |                                                                       ^^^^^^^^^^^^^^^^^^ runtime fault here
  explain: delulu explain E-DL0904
````

</details>

<details>
<summary>B037 — no-grants-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 21.9228,
  "timed_out": false,
  "peak_working_set": 8486912
}
```

stdout: empty.

stderr:

````text
error[DL0703]: `console` was not granted to this program — pass `--grant console`
  --> utility.delulu:31:15
   |
31 |     let out = root.console()
   |               ^^^^^^^^^^^^^^ runtime fault here
  explain: delulu explain E-DL0703
````

</details>

<details>
<summary>B038 — budget-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "budget.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=1"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 1053.9786,
  "timed_out": false,
  "peak_working_set": 7823360
}
```

stdout: empty.

stderr:

````text
error: the run was stopped: it ran for 1 s of wall-clock time, its budget. Budgets are the operator's, set per run with `--limits wall=SECONDS`
````

</details>

<details>
<summary>B039 — basics-corrected-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "basics.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 21.343,
  "timed_out": false,
  "peak_working_set": 8548352
}
```

stdout: empty.

stderr:

````text
error[DL1201]: WASM codegen does not support a call to `range` (not a compilable function) — omit `--engine wasm` to run it on the interpreter
  --> basics.delulu:14:14
   |
14 |     for n in range(0, 5) { total = total + n }
   |              ^^^^^^^^^^^ the WASM backend cannot compile this
  explain: delulu explain E-DL1201
````

</details>

<details>
<summary>B040 — outside-corrected-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "outside.delulu",
    "--grant",
    "fs.write=./output",
    "--json",
    "--report-out",
    "reports/outside-wasm.json",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 29.8009,
  "timed_out": false,
  "peak_working_set": 16769024
}
```

stdout:

````text
{
  "command": "run",
  "delulu_version": "1.0.0",
  "diagnostics": [
    {
      "code": "DL0904",
      "explanation_id": "E-DL0904",
      "message": "path `../forbidden.txt` escapes the granted scope",
      "repairs": [],
      "severity": "error",
      "spans": [
        {
          "end": {
            "byte": 158,
            "col": 63,
            "line": 4
          },
          "file": "outside.delulu",
          "label": "runtime fault here",
          "start": {
            "byte": 114,
            "col": 19,
            "line": 4
          }
        }
      ]
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 1,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B041 — child-outside-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "delegation_denied.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 24.2677,
  "timed_out": false,
  "peak_working_set": 17313792
}
```

stdout: empty.

stderr:

````text
error[DL0904]: path `../task.txt` escapes the granted scope
  --> delegation_denied.delulu:2:71
   |
 2 | fn child(rd: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { rd.read_text(path) }
   |                                                                       ^^^^^^^^^^^^^^^^^^ runtime fault here
  explain: delulu explain E-DL0904
````

</details>

<details>
<summary>B042 — no-grants-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 28.5758,
  "timed_out": false,
  "peak_working_set": 17506304
}
```

stdout: empty.

stderr:

````text
error[DL0703]: `console` was not granted to this program — pass `--grant console`
  --> utility.delulu:31:15
   |
31 |     let out = root.console()
   |               ^^^^^^^^^^^^^^ runtime fault here
  explain: delulu explain E-DL0703
````

</details>

<details>
<summary>B043 — budget-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "budget.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=1"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 1035.7219,
  "timed_out": false,
  "peak_working_set": 16560128
}
```

stdout: empty.

stderr:

````text
error: the run was stopped: it ran for 1 s of wall-clock time, its budget. Budgets are the operator's, set per run with `--limits wall=SECONDS`
````

</details>

<details>
<summary>B044 — utility-malformed-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 32.6053,
  "timed_out": false,
  "peak_working_set": 8605696
}
```

stdout:

````text
{"ok":false,"error":"format"}
````

stderr: empty.

</details>

<details>
<summary>B045 — utility-malformed-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 32.0853,
  "timed_out": false,
  "peak_working_set": 17211392
}
```

stdout:

````text
{"ok":false,"error":"format"}
````

stderr: empty.

</details>

<details>
<summary>B046 — utility-range-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 27.3217,
  "timed_out": false,
  "peak_working_set": 8605696
}
```

stdout:

````text
{"ok":false,"error":"range"}
````

stderr: empty.

</details>

<details>
<summary>B047 — utility-range-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 25.8775,
  "timed_out": false,
  "peak_working_set": 18280448
}
```

stdout:

````text
{"ok":false,"error":"range"}
````

stderr: empty.

</details>

<details>
<summary>B048 — utility-empty-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 18.6615,
  "timed_out": false,
  "peak_working_set": 8552448
}
```

stdout:

````text
{"ok":false,"error":"format"}
````

stderr: empty.

</details>

<details>
<summary>B049 — utility-empty-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 24.5882,
  "timed_out": false,
  "peak_working_set": 17649664
}
```

stdout:

````text
{"ok":false,"error":"format"}
````

stderr: empty.

</details>

<details>
<summary>B050 — utility-missing-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 18.3357,
  "timed_out": false,
  "peak_working_set": 8548352
}
```

stdout:

````text
{"ok":false,"error":"input"}
````

stderr: empty.

</details>

<details>
<summary>B051 — utility-missing-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 25.1554,
  "timed_out": false,
  "peak_working_set": 17428480
}
```

stdout:

````text
{"ok":false,"error":"input"}
````

stderr: empty.

</details>

<details>
<summary>B052 — package-build: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "build",
    "package",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 74.1723,
  "timed_out": false,
  "peak_working_set": 7979008
}
```

stdout:

````text
{
  "command": "build",
  "delulu_version": "1.0.0",
  "diagnostics": [
    {
      "code": "DL1009",
      "explanation_id": "E-DL1009",
      "message": "package `review_package` performs effect `Write` not permitted by its authority manifest",
      "repairs": [],
      "severity": "error",
      "spans": [
        {
          "end": {
            "byte": 0,
            "col": 1,
            "line": 1
          },
          "file": "delulu.toml",
          "label": "add `Write` to `[authority] effects` in delulu.toml",
          "start": {
            "byte": 0,
            "col": 1,
            "line": 1
          }
        }
      ]
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 1,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B053 — package-run-interp: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "package",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 23.6726,
  "timed_out": false,
  "peak_working_set": 8052736
}
```

stdout: empty.

stderr:

````text
error[DL1009]: package `review_package` performs effect `Write` not permitted by its authority manifest
  --> delulu.toml:1:1
   |
 1 | [package]
   | ^^^^^^^^^ add `Write` to `[authority] effects` in delulu.toml
  explain: delulu explain E-DL1009
````

</details>

<details>
<summary>B054 — package-run-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "package",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 18.9237,
  "timed_out": false,
  "peak_working_set": 8052736
}
```

stdout: empty.

stderr:

````text
error[DL1009]: package `review_package` performs effect `Write` not permitted by its authority manifest
  --> delulu.toml:1:1
   |
 1 | [package]
   | ^^^^^^^^^ add `Write` to `[authority] effects` in delulu.toml
  explain: delulu explain E-DL1009
````

</details>

<details>
<summary>B055 — file-import: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "package/src/main.delulu",
    "--grant",
    "console",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 17.8579,
  "timed_out": false,
  "peak_working_set": 7643136
}
```

stdout: empty.

stderr:

````text
error[DL0301]: unknown name `twice`
  --> package/src/main.delulu:3:60
   |
 3 | fn main(root: Root) ! {Write} { root.console().println(str(twice(21))) }
   |                                                            ^^^^^ not found in this scope
  explain: delulu explain E-DL0301
````

</details>

<details>
<summary>B056 — missing-test-path: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "test",
    "does-not-exist.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\package",
  "exit": 1,
  "ms": 1023.723,
  "timed_out": false,
  "peak_working_set": 8273920
}
```

stdout:

````text
{
  "command": "test",
  "custody": {
    "mode": "embedded"
  },
  "delulu_version": "1.0.0",
  "diagnostics": [
    {
      "code": "DL1009",
      "explanation_id": "E-DL1009",
      "message": "package `review_package` performs effect `Write` not permitted by its authority manifest",
      "repairs": [],
      "severity": "error",
      "spans": [
        {
          "end": {
            "byte": 0,
            "col": 1,
            "line": 1
          },
          "file": "delulu.toml",
          "label": "add `Write` to `[authority] effects` in delulu.toml",
          "start": {
            "byte": 0,
            "col": 1,
            "line": 1
          }
        }
      ]
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 1,
    "failed": 1,
    "ms": 0,
    "passed": 0,
    "warnings": 0
  },
  "tests": [
    {
      "errors": 1,
      "file": ".",
      "status": "check-failed"
    }
  ]
}
````

stderr: empty.

</details>

<details>
<summary>B057 — authority-utility: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "authority",
    "utility.delulu",
    "--within",
    "console",
    "--within",
    "fs.read=./input",
    "--within",
    "fs.write=./output",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 20.08,
  "timed_out": false,
  "peak_working_set": 8527872
}
```

stdout:

````text
{
  "authority": {
    "capabilities": [
      {
        "kind": "FsRead",
        "requested_computed": false,
        "requested_scopes": [
          "./input"
        ],
        "scopes": []
      },
      {
        "kind": "FsWrite",
        "requested_computed": false,
        "requested_scopes": [
          "./output"
        ],
        "scopes": []
      },
      {
        "kind": "Console",
        "requested_computed": false,
        "requested_scopes": [],
        "scopes": [
          "stdio"
        ]
      }
    ],
    "contained_plugins": [],
    "custody": "embedded",
    "effects": [
      "Read",
      "Write"
    ],
    "foreign_calls": [],
    "foreign_isolation": "inproc",
    "program": "review.utility",
    "pure_functions": [
      "evaluate"
    ],
    "requested_scopes": {
      "FsRead": [
        "./input"
      ],
      "FsWrite": [
        "./output"
      ]
    },
    "required_grants": [
      "fs.read=./input",
      "fs.write=./output",
      "console"
    ],
    "secrets": [],
    "within": {
      "boundary": [
        "console",
        "fs.read=./input",
        "fs.write=./output"
      ],
      "counterexample": null,
      "coverage": {
        "actuator": "not-requested",
        "clock": "not-requested",
        "compute": "not-requested",
        "console": "within",
        "declassify": "not-requested",
        "foreign.c": "not-requested",
        "foreign.python": "not-requested",
        "fs.read": "within",
        "fs.write": "within",
        "net": "not-requested",
        "net.special": "not-requested",
        "plugin": "not-requested",
        "rand": "not-requested",
        "secret": "not-requested",
        "sensor": "not-requested"
      },
      "findings": [],
      "requests": 5,
      "result": "within"
    }
  },
  "command": "authority",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B058 — authority-dynamic: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "authority",
    "dynamic.delulu",
    "--within",
    "console",
    "--within",
    "fs.read=./input",
    "--within",
    "fs.write=./output",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 18.9491,
  "timed_out": false,
  "peak_working_set": 8269824
}
```

stdout:

````text
{
  "authority": {
    "capabilities": [
      {
        "kind": "FsRead",
        "requested_computed": true,
        "requested_scopes": [],
        "scopes": []
      },
      {
        "kind": "Console",
        "requested_computed": false,
        "requested_scopes": [],
        "scopes": [
          "stdio"
        ]
      }
    ],
    "contained_plugins": [],
    "custody": "embedded",
    "effects": [
      "Read",
      "Write"
    ],
    "foreign_calls": [],
    "foreign_isolation": "inproc",
    "program": "review.dynamic",
    "pure_functions": [
      "scope"
    ],
    "requested_scopes": {},
    "required_grants": [
      "fs.read=PATH",
      "console"
    ],
    "secrets": [],
    "within": {
      "boundary": [
        "console",
        "fs.read=./input",
        "fs.write=./output"
      ],
      "counterexample": {
        "at": {
          "column": 14,
          "file": "dynamic.delulu",
          "line": 5
        },
        "dimension": "fs.read",
        "grant": "fs.read=PATH",
        "reason": "the source does not show this path to read (it is computed, or reached through a value the checker does not follow); the boundary grants some, and which one it is is decided only when the program runs",
        "result": "inconclusive"
      },
      "coverage": {
        "actuator": "not-requested",
        "clock": "not-requested",
        "compute": "not-requested",
        "console": "within",
        "declassify": "not-requested",
        "foreign.c": "not-requested",
        "foreign.python": "not-requested",
        "fs.read": "inconclusive",
        "fs.write": "not-requested",
        "net": "not-requested",
        "net.special": "not-requested",
        "plugin": "not-requested",
        "rand": "not-requested",
        "secret": "not-requested",
        "sensor": "not-requested"
      },
      "findings": [
        {
          "at": {
            "column": 14,
            "file": "dynamic.delulu",
            "line": 5
          },
          "dimension": "fs.read",
          "grant": "fs.read=PATH",
          "reason": "the source does not show this path to read (it is computed, or reached through a value the checker does not follow); the boundary grants some, and which one it is is decided only when the program runs",
          "result": "inconclusive"
        }
      ],
      "requests": 3,
      "result": "inconclusive"
    }
  },
  "command": "authority",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B059 — authority-outside: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "authority",
    "outside.delulu",
    "--within",
    "console",
    "--within",
    "fs.read=./input",
    "--within",
    "fs.write=./output",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 20.3581,
  "timed_out": false,
  "peak_working_set": 8126464
}
```

stdout:

````text
{
  "authority": {
    "capabilities": [
      {
        "kind": "FsWrite",
        "requested_computed": false,
        "requested_scopes": [
          "./output"
        ],
        "scopes": []
      }
    ],
    "contained_plugins": [],
    "custody": "embedded",
    "effects": [
      "Write"
    ],
    "foreign_calls": [],
    "foreign_isolation": "inproc",
    "program": "review.outside",
    "pure_functions": [],
    "requested_scopes": {
      "FsWrite": [
        "./output"
      ]
    },
    "required_grants": [
      "fs.write=./output"
    ],
    "secrets": [],
    "within": {
      "boundary": [
        "console",
        "fs.read=./input",
        "fs.write=./output"
      ],
      "counterexample": {
        "at": {
          "column": 19,
          "file": "outside.delulu",
          "line": 4
        },
        "dimension": "fs.write",
        "grant": "fs.write=./output",
        "reason": "the write of `../forbidden.txt` through the capability for `./output` is refused at its call whatever the boundary grants: path `../forbidden.txt` escapes the granted scope (DL0904)",
        "result": "exceeds"
      },
      "coverage": {
        "actuator": "not-requested",
        "clock": "not-requested",
        "compute": "not-requested",
        "console": "not-requested",
        "declassify": "not-requested",
        "foreign.c": "not-requested",
        "foreign.python": "not-requested",
        "fs.read": "not-requested",
        "fs.write": "exceeds",
        "net": "not-requested",
        "net.special": "not-requested",
        "plugin": "not-requested",
        "rand": "not-requested",
        "secret": "not-requested",
        "sensor": "not-requested"
      },
      "findings": [
        {
          "at": {
            "column": 19,
            "file": "outside.delulu",
            "line": 4
          },
          "dimension": "fs.write",
          "grant": "fs.write=./output",
          "reason": "the write of `../forbidden.txt` through the capability for `./output` is refused at its call whatever the boundary grants: path `../forbidden.txt` escapes the granted scope (DL0904)",
          "result": "exceeds"
        }
      ],
      "requests": 2,
      "result": "exceeds"
    }
  },
  "command": "authority",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B060 — check-machine-error: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "check",
    "effect_missing.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 18.0816,
  "timed_out": false,
  "peak_working_set": 7528448
}
```

stdout:

````text
{
  "command": "check",
  "delulu_version": "1.0.0",
  "diagnostics": [
    {
      "code": "DL0501",
      "explanation_id": "E-DL0501",
      "message": "function `main` performs effect `Write` not declared in its row",
      "repairs": [
        {
          "authority_widening": true,
          "confidence": "exact",
          "edits": [
            {
              "file": "effect_missing.delulu",
              "insert": "! {Write} ",
              "range": {
                "end_byte": 50,
                "start_byte": 50
              }
            }
          ],
          "id": "add_effect_to_row",
          "reason": null,
          "requires_human": false
        }
      ],
      "severity": "error",
      "spans": [
        {
          "end": {
            "byte": 37,
            "col": 8,
            "line": 2
          },
          "file": "effect_missing.delulu",
          "label": "declared row is here",
          "start": {
            "byte": 33,
            "col": 4,
            "line": 2
          }
        }
      ]
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 1,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B061 — check-machine-repaired: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "check",
    "effect_repaired.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 33.7885,
  "timed_out": false,
  "peak_working_set": 7438336
}
```

stdout:

````text
{
  "command": "check",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B062 — syntax-machine-error: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "check",
    "compound_missing.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 17.7081,
  "timed_out": false,
  "peak_working_set": 7409664
}
```

stdout:

````text
{
  "command": "check",
  "delulu_version": "1.0.0",
  "diagnostics": [
    {
      "code": "DL0202",
      "explanation_id": "E-DL0202",
      "message": "expected an expression, found `=`",
      "repairs": [],
      "severity": "error",
      "spans": [
        {
          "end": {
            "byte": 70,
            "col": 6,
            "line": 3
          },
          "file": "compound_missing.delulu",
          "label": "expected an expression",
          "start": {
            "byte": 69,
            "col": 5,
            "line": 3
          }
        }
      ]
    },
    {
      "code": "DL0414",
      "explanation_id": "E-DL0414",
      "message": "`n` is bound here and never read",
      "repairs": [
        {
          "authority_widening": false,
          "confidence": "safe",
          "edits": [
            {
              "file": "compound_missing.delulu",
              "insert": "_",
              "range": {
                "end_byte": 58,
                "start_byte": 58
              }
            }
          ],
          "id": "prefix-underscore",
          "reason": null,
          "requires_human": true
        }
      ],
      "severity": "warning",
      "spans": [
        {
          "end": {
            "byte": 59,
            "col": 28,
            "line": 2
          },
          "file": "compound_missing.delulu",
          "label": "never read: remove it, or name it `_n` to say so",
          "start": {
            "byte": 58,
            "col": 27,
            "line": 2
          }
        }
      ]
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 1,
    "warnings": 1
  }
}
````

stderr: empty.

</details>

<details>
<summary>B063 — sandbox-probe: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "sandbox",
    "probe",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 94.4151,
  "timed_out": false,
  "peak_working_set": 11141120
}
```

stdout:

````text
{
  "command": "sandbox",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "highest_available": 1,
  "host": {
    "arch": "x86_64",
    "os": "windows"
  },
  "levels": [
    {
      "attempts": [
        {
          "detail": "this probe is running in it: the language and custody, in-process — no OS boundary",
          "ok": true,
          "what": "the in-process runtime"
        }
      ],
      "available": true,
      "first_missing": null,
      "level": 0,
      "name": "none"
    },
    {
      "attempts": [
        {
          "detail": "created a Job Object and set kill-on-close, one-process and memory limits",
          "ok": true,
          "what": "a Job Object with limits"
        },
        {
          "detail": "created a restricted token (all privileges but one removed)",
          "ok": true,
          "what": "a restricted token"
        },
        {
          "detail": "a guest started under its jail and opened its channel (one process only; memory ceiling; processor-time ceiling; killed with the host; no desktop, clipboard or global atoms; a separate identity: a per-run AppContainer with no capabilities — no network, none of the operator's files)",
          "ok": true,
          "what": "the L1 guest launcher"
        }
      ],
      "available": true,
      "first_missing": null,
      "level": 1,
      "name": "process"
    },
    {
      "attempts": [
        {
          "detail": "cannot open /dev/kvm read-write: The system cannot find the path specified. (os error 3)",
          "ok": false,
          "what": "KVM"
        },
        {
          "detail": "the microVM is Linux-only (PS-C)",
          "ok": false,
          "what": "the L2 guest launch"
        }
      ],
      "available": false,
      "first_missing": "KVM: cannot open /dev/kvm read-write: The system cannot find the path specified. (os error 3)",
      "level": 2,
      "name": "microvm"
    },
    {
      "attempts": [
        {
          "detail": "none can be configured in this build (PS-D)",
          "ok": false,
          "what": "an operator-supplied external launcher"
        }
      ],
      "available": false,
      "first_missing": "an operator-supplied external launcher: none can be configured in this build (PS-D)",
      "level": 3,
      "name": "external"
    },
    {
      "attempts": [
        {
          "detail": "attestation is deferred (PS-D-02)",
          "ok": false,
          "what": "an attester for the guest image"
        }
      ],
      "available": false,
      "first_missing": "an attester for the guest image: attestation is deferred (PS-D-02)",
      "level": 4,
      "name": "attested"
    }
  ],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B064 — sandbox-utility: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--sandbox",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--json",
    "--report-out",
    "reports/sandbox-utility.json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 127.7329,
  "timed_out": false,
  "peak_working_set": 13131776
}
```

stdout:

````text
{"ok":true,"value":42}
````

stderr:

````text
sandbox: the guest is confined — one process only; memory ceiling; processor-time ceiling; killed with the host; no desktop, clipboard or global atoms; a separate identity: a per-run AppContainer with no capabilities — no network, none of the operator's files; wall-clock ceiling
````

</details>

<details>
<summary>B065 — wasm-package: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "build",
    "edge.delulu",
    "--target",
    "wasm",
    "-o",
    "output/edge.dwx",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 19.5915,
  "timed_out": false,
  "peak_working_set": 8282112
}
```

stdout:

````text
{
  "authority": {
    "artifact": "output/edge.dwx",
    "authority": {
      "capabilities": [
        {
          "kind": "Console",
          "scopes": [
            "stdio"
          ]
        }
      ],
      "contained_plugins": [],
      "effects": [
        "Write"
      ],
      "foreign_calls": [],
      "program": "review.edge",
      "pure_functions": [
        "aggregate"
      ],
      "secrets": []
    },
    "bytes": 1196
  },
  "command": "build",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B066 — wasm-package-run: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "output/edge.dwx",
    "--grant",
    "console",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 59.7689,
  "timed_out": false,
  "peak_working_set": 14749696
}
```

stdout:

````text
4054095
````

stderr:

````text
running `output/edge.dwx` — authority verified; declared effects: Write
````

</details>

<details>
<summary>B067 — plugin-build: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "plugin",
    "build",
    "plugin",
    "-o",
    "plugin/shout.dpx",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 41.8732,
  "timed_out": false,
  "peak_working_set": 8167424
}
```

stdout:

````text
{
  "api": 1,
  "artifact": "plugin/shout.dpx",
  "bytes": 1307,
  "class": "verified",
  "command": "plugin",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "exports": [
    "shout"
  ],
  "name": "shout",
  "schema": 1,
  "signed": false,
  "subcommand": "build",
  "summary": {
    "errors": 0,
    "warnings": 0
  },
  "version": "0.1.0"
}
````

stderr: empty.

</details>

<details>
<summary>B068 — plugin-inspect: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "plugin",
    "inspect",
    "plugin/shout.dpx",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 28.5504,
  "timed_out": false,
  "peak_working_set": 6610944
}
```

stdout:

````text
{
  "api": 1,
  "artifact": "plugin/shout.dpx",
  "authority": {
    "declassify": [],
    "effects": [],
    "fs_read": [],
    "fs_write": [],
    "net": [],
    "requires": [],
    "secrets": []
  },
  "class": "verified",
  "command": "plugin",
  "container": 1,
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "exports": {
    "shout": "fn(Str) -> Str"
  },
  "name": "shout",
  "schema": 1,
  "sections": {
    "dir": "050f7376b246c6edfc8448e8102947a5aab19c5c274cce3db6083fbc6c35d30f",
    "lock": null,
    "sig": false,
    "wasm": null
  },
  "signed_by": null,
  "subcommand": "inspect",
  "summary": {
    "errors": 0,
    "warnings": 0
  },
  "version": "0.1.0",
  "wasm_cache_valid": true
}
````

stderr: empty.

</details>

<details>
<summary>B069 — plugin-verify: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "plugin",
    "verify",
    "plugin/shout.dpx",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 18.9169,
  "timed_out": false,
  "peak_working_set": 8138752
}
```

stdout:

````text
{
  "artifact": "plugin/shout.dpx",
  "class": "verified",
  "command": "plugin",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "exports": [
    "shout"
  ],
  "schema": 1,
  "signed_by": null,
  "subcommand": "verify",
  "summary": {
    "errors": 0,
    "warnings": 0
  },
  "verdict": "ok"
}
````

stderr: empty.

</details>

<details>
<summary>B070 — plugin-allowed: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "host.delulu",
    "--grant",
    "console",
    "--grant",
    "plugin=.",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\plugin",
  "exit": 1,
  "ms": 19.9753,
  "timed_out": false,
  "peak_working_set": 7880704
}
```

stdout: empty.

stderr:

````text
error[DL0701]: `main` performs effect `Read` not permitted by the authority manifest
  explain: delulu explain E-DL0701

error[DL0701]: `main` performs effect `Write` not permitted by the authority manifest
  explain: delulu explain E-DL0701

error[DL0701]: `main` performs effect `Load` not permitted by the authority manifest
  explain: delulu explain E-DL0701
````

</details>

<details>
<summary>B071 — plugin-overbroad: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "overbroad.delulu",
    "--grant",
    "console",
    "--grant",
    "plugin=.",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\plugin",
  "exit": 1,
  "ms": 33.1323,
  "timed_out": false,
  "peak_working_set": 7909376
}
```

stdout: empty.

stderr:

````text
error[DL0701]: `main` performs effect `Read` not permitted by the authority manifest
  explain: delulu explain E-DL0701

error[DL0701]: `main` performs effect `Write` not permitted by the authority manifest
  explain: delulu explain E-DL0701

error[DL0701]: `main` performs effect `Load` not permitted by the authority manifest
  explain: delulu explain E-DL0701
````

</details>

<details>
<summary>B072 — plugin-ungranted: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "host.delulu",
    "--grant",
    "console",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\plugin",
  "exit": 1,
  "ms": 15.2516,
  "timed_out": false,
  "peak_working_set": 7852032
}
```

stdout: empty.

stderr:

````text
error[DL0701]: `main` performs effect `Read` not permitted by the authority manifest
  explain: delulu explain E-DL0701

error[DL0701]: `main` performs effect `Write` not permitted by the authority manifest
  explain: delulu explain E-DL0701

error[DL0701]: `main` performs effect `Load` not permitted by the authority manifest
  explain: delulu explain E-DL0701
````

</details>

<details>
<summary>B073 — plugin-sandbox: exit 2</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "host.delulu",
    "--sandbox",
    "--grant",
    "console",
    "--grant",
    "plugin=.",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\plugin",
  "exit": 2,
  "ms": 17.1929,
  "timed_out": false,
  "peak_working_set": 7757824
}
```

stdout: empty.

stderr:

````text
error: `--sandbox` cannot carry this program yet: it uses PluginHost. Nothing ran. Run it with `--sandbox=off` if you accept no confinement, or wait for the channel to carry that surface — the sandbox will not quietly not apply.
````

</details>

<details>
<summary>B074 — survey-findings---json: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "findings",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 972.3854,
  "timed_out": false,
  "peak_working_set": 36450304
}
```

stdout: 14645 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `survey-findings---json`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B075 — survey-query-crate:delulu-check: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "query",
    "crate:delulu-check",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 665.9238,
  "timed_out": false,
  "peak_working_set": 36220928
}
```

stdout: 48942 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `survey-query-crate:delulu-check`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B076 — survey-impact-crate:delulu-check: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "impact",
    "crate:delulu-check",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 665.534,
  "timed_out": false,
  "peak_working_set": 36454400
}
```

stdout: 58728 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `survey-impact-crate:delulu-check`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B077 — survey-affected-by-crate:delulu-check: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "affected-by",
    "crate:delulu-check",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 645.4734,
  "timed_out": false,
  "peak_working_set": 36503552
}
```

stdout: 10915 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `survey-affected-by-crate:delulu-check`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B078 — survey-query-crate:delulu: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "query",
    "crate:delulu",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 671.8414,
  "timed_out": false,
  "peak_working_set": 36605952
}
```

stdout: 24294 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `survey-query-crate:delulu`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B079 — survey-impact-crate:delulu-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "impact",
    "crate:delulu-wasm",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 658.5735,
  "timed_out": false,
  "peak_working_set": 36442112
}
```

stdout: 32062 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `survey-impact-crate:delulu-wasm`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B080 — survey-query-doc:docs/design/CONSTITUTION.md: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "query",
    "doc:docs/design/CONSTITUTION.md",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 697.864,
  "timed_out": false,
  "peak_working_set": 36544512
}
```

stdout:

````text
{
  "delulu_version": "1.0.0",
  "entrenched": {
    "matched_at": {
      "file": ".github/CODEOWNERS",
      "line": 12
    },
    "means": "changing this needs that owner specifically, not any maintainer; Constitution §10 requires an entrenchment analysis (invariant 44) first",
    "owner": "@jessesuniljs1-collab",
    "pattern": "/docs/design/CONSTITUTION.md"
  },
  "node": {
    "holds": [
      "1. Identity",
      "2. The Irreducibility Law",
      "3. Why Choose DeluluLang Over X?",
      "4. What Becomes Possible (the irreducibility test's positive twin)",
      "5. Language Semantics (normative core)",
      "6. The Plugin Architecture — first-class pillar, sharpest proof of the core",
      "7. Embodied AI and Robotics — the physical-stakes proof",
      "8. Supporting Pillars (deliberately demoted)",
      "9. Governance and Honesty Clauses",
      "10. Evolution",
      "Appendix A — Decision Record (v1.0)"
    ],
    "id": "doc:docs/design/CONSTITUTION.md",
    "kind": "Doc",
    "lines": 630,
    "path": "docs/design/CONSTITUTION.md",
    "summary": "The DeluluLang Constitution — Version 1.0"
  },
  "pointed_at_by": [
    {
      "node": "doc:AGENTS.md",
      "via": {
        "file": "AGENTS.md",
        "kind": "links-to",
        "line": 54
      }
    },
    {
      "node": "doc:README.md",
      "via": {
        "file": "README.md",
        "kind": "links-to",
        "line": 230
      }
    },
    {
      "node": "doc:docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md",
      "via": {
        "file": "docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md",
        "kind": "links-to",
        "line": 52
      }
    },
    {
      "node": "doc:docs/QUESTIONS.md",
      "via": {
        "file": "docs/QUESTIONS.md",
        "kind": "links-to",
        "line": 719
      }
    },
    {
      "node": "doc:docs/REMAINING_WORK.md",
      "via": {
        "file": "docs/REMAINING_WORK.md",
        "kind": "links-to",
        "line": 106
      }
    },
    {
      "node": "doc:docs/REPOSITORY_STRUCTURE.md",
      "via": {
        "file": "docs/REPOSITORY_STRUCTURE.md",
        "kind": "links-to",
        "line": 500
      }
    },
    {
      "node": "doc:docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md",
      "via": {
        "file": "docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md",
        "kind": "links-to",
        "line": 11
      }
    },
    {
      "node": "doc:docs/archive/v1/NEXT_EVOLUTION_2026/EXECUTION_LOG.md",
      "via": {
        "file": "docs/archive/v1/NEXT_EVOLUTION_2026/EXECUTION_LOG.md",
        "kind": "links-to",
        "line": 19
      }
    },
    {
      "node": "doc:docs/archive/v1/NEXT_EVOLUTION_2026/EXECUTION_LOG.md",
      "via": {
        "file": "docs/archive/v1/NEXT_EVOLUTION_2026/EXECUTION_LOG.md",
        "kind": "links-to",
        "line": 46
      }
    },
    {
      "node": "doc:docs/archive/v1/README.md",
      "via": {
        "file": "docs/archive/v1/README.md",
        "kind": "links-to",
        "line": 32
      }
    },
    {
      "node": "doc:docs/archive/v1/design/LANGUAGE_SPECIFICATION.md",
      "via": {
        "file": "docs/archive/v1/design/LANGUAGE_SPECIFICATION.md",
        "kind": "links-to",
        "line": 6
      }
    },
    {
      "node": "doc:docs/book/THE_DELULULANG_BOOK.md",
      "via": {
        "file": "docs/book/THE_DELULULANG_BOOK.md",
        "kind": "links-to",
        "line": 1629
      }
    },
    {
      "node": "doc:docs/design/ENTRENCHED_CHANGE_RECORD.md",
      "via": {
        "file": "docs/design/ENTRENCHED_CHANGE_RECORD.md",
        "kind": "links-to",
        "line": 117
      }
    },
    {
      "node": "doc:docs/design/ENTRENCHED_CHANGE_RECORD.md",
      "via": {
        "file": "docs/design/ENTRENCHED_CHANGE_RECORD.md",
        "kind": "links-to",
        "line": 123
      }
    },
    {
      "node": "doc:docs/survey/README.md",
      "via": {
        "file": "docs/survey/README.md",
        "kind": "links-to",
        "line": 157
      }
    }
  ],
  "points_at": [
    {
      "node": "code:DL1408",
      "via": {
        "file": "docs/design/CONSTITUTION.md",
        "kind": "documents",
        "line": 364
      }
    }
  ],
  "schema": 1,
  "tool": "delulu-survey",
  "verb": "query"
}
````

stderr: empty.

</details>

<details>
<summary>B081 — survey-query-doc:docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "query",
    "doc:docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 0,
  "ms": 759.0121,
  "timed_out": false,
  "peak_working_set": 36487168
}
```

stdout:

````text
{
  "delulu_version": "1.0.0",
  "entrenched": null,
  "node": {
    "holds": [
      "My core vision — DeluluLang is primarily for AI, not primarily for humans",
      "What I am thinking of changing",
      "Read this first — and understand the existing project",
      "Important findings in the uploaded project documents — starting hypotheses, not automatic conclusions",
      "Critical rule: this is RESEARCH ONLY",
      "Use DeluluLang's own tools, and verify them",
      "My added recommendations for Codex to investigate — AI-native architecture rather than feature accumulation",
      "Agents and cost limits",
      "Write real DeluluLang programs and see what actually happens",
      "Mandatory AI-first programs, simulations and adversarial probes",
      "Research questions I want answered",
      "The report I want"
    ],
    "id": "doc:docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
    "kind": "Doc",
    "lines": 267,
    "path": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
    "summary": "DeluluLang — First Codex (GPT-6 Astra) Session: AI-First Architecture, Complete Research, and Strategic Review"
  },
  "pointed_at_by": [],
  "points_at": [
    {
      "node": "doc:docs/CLOUD_SYNC_LOG.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    },
    {
      "node": "doc:docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    },
    {
      "node": "doc:docs/DEPLOYMENT.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    },
    {
      "node": "doc:docs/editors.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    },
    {
      "node": "doc:docs/for-agents.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    },
    {
      "node": "doc:measurements/agent-loop/RECORD.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    },
    {
      "node": "doc:measurements/study-c/REPORT.md",
      "via": {
        "file": "docs/prompt/DeluluLang_Codex_GPT6_Astra_Research_Prompt (1).md",
        "kind": "links-to",
        "line": 65
      }
    }
  ],
  "schema": 1,
  "tool": "delulu-survey",
  "verb": "query"
}
````

stderr: empty.

</details>

<details>
<summary>B082 — package-build-corrected: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "build",
    "package",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 55.0972,
  "timed_out": false,
  "peak_working_set": 8077312
}
```

stdout:

````text
{
  "command": "build",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B083 — package-corrected-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "package",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 21.1729,
  "timed_out": false,
  "peak_working_set": 8720384
}
```

stdout:

````text
42
````

stderr: empty.

</details>

<details>
<summary>B084 — package-corrected-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "package",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 25.6048,
  "timed_out": false,
  "peak_working_set": 18288640
}
```

stdout:

````text
42
````

stderr: empty.

</details>

<details>
<summary>B085 — missing-test-path-corrected: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "test",
    "does-not-exist.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\package",
  "exit": 0,
  "ms": 1023.9182,
  "timed_out": false,
  "peak_working_set": 8376320
}
```

stdout:

````text
{
  "command": "test",
  "custody": {
    "mode": "embedded"
  },
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "summary": {
    "errors": 0,
    "failed": 0,
    "ms": 1,
    "passed": 0,
    "warnings": 0
  },
  "tests": []
}
````

stderr: empty.

</details>

<details>
<summary>B086 — basics-while-interp: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "basics_while.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 59.6135,
  "timed_out": false,
  "peak_working_set": 8581120
}
```

stdout:

````text
Some(65)
[1, 2, 3, 4]
A:B
Ok(42)
Err(Bad(number))
````

stderr: empty.

</details>

<details>
<summary>B087 — basics-while-wasm: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "basics_while.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 26.8678,
  "timed_out": false,
  "peak_working_set": 17686528
}
```

stdout:

````text
Some(65)
[1, 2, 3, 4]
A:B
Ok(42)
Err(Bad(number))
````

stderr: empty.

</details>

<details>
<summary>B088 — plugin_host-corrected: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "plugin_host.delulu",
    "--grant",
    "console",
    "--grant",
    "plugin=.",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 44.6898,
  "timed_out": false,
  "peak_working_set": 9560064
}
```

stdout:

````text
hello!
````

stderr: empty.

</details>

<details>
<summary>B089 — plugin_overbroad-corrected: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "plugin_overbroad.delulu",
    "--grant",
    "console",
    "--grant",
    "plugin=.",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 41.3805,
  "timed_out": false,
  "peak_working_set": 8949760
}
```

stdout:

````text
not granted: the requested grant exceeds the plugin's declared ceiling — requested effects={Write}; ceiling effects={}; the host may grant less than the ceiling, never more
````

stderr: empty.

</details>

<details>
<summary>B090 — plugin-ungranted-corrected: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "plugin_host.delulu",
    "--grant",
    "console",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 42.2334,
  "timed_out": false,
  "peak_working_set": 8441856
}
```

stdout: empty.

stderr:

````text
error[DL0703]: `plugin loading` was not granted to this program — pass `--grant plugin=PATH`
  --> plugin_host.delulu:46:13
   |
46 |     let h = root.plugin_host()
   |             ^^^^^^^^^^^^^^^^^^ runtime fault here
  explain: delulu explain E-DL0703
````

</details>

<details>
<summary>B091 — utility-json-success: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output",
    "--json",
    "--report-out",
    "reports/utility.json",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 22.7984,
  "timed_out": false,
  "peak_working_set": 8781824
}
```

stdout:

````text
{"ok":true,"value":42}
````

stderr: empty.

</details>

<details>
<summary>B092 — usage-machine: exit 2</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "check",
    "--this-option-does-not-exist",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 2,
  "ms": 16.6318,
  "timed_out": false,
  "peak_working_set": 6402048
}
```

stdout:

````text
{
  "command": "check",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "error": {
    "exit": 2,
    "kind": "usage",
    "message": "the command could not run; the human-readable reason is on stderr"
  },
  "schema": 1,
  "summary": {
    "errors": 1,
    "warnings": 0
  }
}
````

stderr:

````text
error: `check` does not know this option: --this-option-does-not-exist
  nothing was done — an option nobody understood is refused, never ignored
note: `delulu check --help` lists what this command accepts
````

</details>

<details>
<summary>B093 — schema-list: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "schema",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 17.1201,
  "timed_out": false,
  "peak_working_set": 6496256
}
```

stdout:

````text
{
  "command": "schema",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "schema": 1,
  "schemas": [
    {
      "describes": "the object every `--json` command prints: command, schema, version, diagnostics, summary",
      "name": "envelope"
    },
    {
      "describes": "one diagnostic: code, severity, message, spans, explanation id, typed repairs",
      "name": "diagnostic"
    },
    {
      "describes": "one typed repair: its edits, confidence, and whether it widens authority or needs a human",
      "name": "repair"
    },
    {
      "describes": "the authority report `delulu authority --json` carries under `authority`",
      "name": "authority"
    },
    {
      "describes": "the `atlas/1` graph `delulu atlas --json` carries under `atlas`",
      "name": "atlas"
    },
    {
      "describes": "the `sandbox` object of a run report: in-process, previewed, or a sandboxed run",
      "name": "sandbox"
    },
    {
      "describes": "what `delulu sandbox policy --json` carries under `policy`",
      "name": "policy"
    },
    {
      "describes": "the report `delulu run --report-out F` writes",
      "name": "run-report"
    },
    {
      "describes": "what `delulu toolchain --json` carries under `toolchain`",
      "name": "toolchain"
    },
    {
      "describes": "what `delulu edit --json` carries under `edit`: the checked edit, or why it was refused",
      "name": "edit"
    },
    {
      "describes": "what `delulu atlas chain --json` carries under `chain`: program to execution boundary, in ten links",
      "name": "chain"
    },
    {
      "describes": "what `delulu sandbox policy --format openshell --json` carries under `openshell`: the policy, and where each grant went",
      "name": "openshell"
    }
  ],
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
````

stderr: empty.

</details>

<details>
<summary>B094 — toolchain: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "toolchain",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 21.5686,
  "timed_out": false,
  "peak_working_set": 7225344
}
```

stdout: 64381 characters; full response in [results.jsonl](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/results.jsonl) under label `toolchain`. Summary used in the main report; not silently truncated here.

stderr: empty.

</details>

<details>
<summary>B095 — atlas-utility: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "atlas",
    "chain",
    "utility.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 19.6885,
  "timed_out": false,
  "peak_working_set": 8847360
}
```

stdout:

````text
{
  "chain": {
    "actors": {
      "actors": [],
      "async_performed_by": []
    },
    "authority": {
      "effects": [
        "Read",
        "Write"
      ],
      "pure": false,
      "required_grants": [
        "fs.read=./input",
        "fs.write=./output",
        "console"
      ],
      "secrets": []
    },
    "capabilities": [
      {
        "kind": "Console",
        "requested_computed": false,
        "requested_scopes": [],
        "resource_class": "console",
        "resources": [
          "res:console:stdio"
        ],
        "scopes": [
          "stdio"
        ]
      },
      {
        "kind": "FsRead",
        "requested_computed": false,
        "requested_scopes": [
          "./input"
        ],
        "resource_class": "fs_read",
        "resources": [
          "res:fs_read:*"
        ],
        "scopes": []
      },
      {
        "kind": "FsWrite",
        "requested_computed": false,
        "requested_scopes": [
          "./output"
        ],
        "resource_class": "fs_write",
        "resources": [
          "res:fs_write:*"
        ],
        "scopes": []
      }
    ],
    "devices": {
      "actuate_performed_by": [],
      "devices": []
    },
    "effects": [
      {
        "effect": "Read",
        "performed_by": [
          "fn:review.utility/review.utility.main",
          "fn:review.utility/review.utility.task"
        ]
      },
      {
        "effect": "Write",
        "performed_by": [
          "fn:review.utility/review.utility.main"
        ]
      }
    ],
    "execution_boundary": {
      "custody": "embedded",
      "foreign": {
        "boundaries": [],
        "calls": [],
        "isolation": "inproc"
      },
      "ordinary_run": "in-process (L0): the language and custody in one process, no OS boundary",
      "run_mode_note": "custody and foreign isolation are an ordinary run's; `--broker daemon` and `--foreign-isolation process` change them",
      "sandboxed_run": {
        "backend": "process",
        "carried": true,
        "guest_performs_effects": false,
        "level": 1
      }
    },
    "plugins": {
      "contained": [],
      "hosts": [],
      "load_performed_by": []
    },
    "program": {
      "functions": 3,
      "modules": [
        "mod:review.utility/review.utility"
      ],
      "name": "review.utility",
      "packages": [
        "pkg:review.utility"
      ],
      "root": "review.utility"
    },
    "resources": [
      {
        "class": "console",
        "declassified_by": [],
        "id": "res:console:stdio",
        "pattern": "stdio",
        "requested_scopes": [],
        "required_by": [
          "fn:review.utility/review.utility.main"
        ]
      },
      {
        "class": "fs_read",
        "declassified_by": [],
        "id": "res:fs_read:*",
        "pattern": "*",
        "requested_scopes": [
          "./input"
        ],
        "required_by": [
          "fn:review.utility/review.utility.main",
          "fn:review.utility/review.utility.task"
        ]
      },
      {
        "class": "fs_write",
        "declassified_by": [],
        "id": "res:fs_write:*",
        "pattern": "*",
        "requested_scopes": [
          "./output"
        ],
        "required_by": [
          "fn:review.utility/review.utility.main"
        ]
      }
    ],
    "sandbox_policy": {
      "backend": "process",
      "carried": true,
      "carried_note": null,
      "limits": {
        "cpu_seconds": 300,
        "memory_bytes": 1073741824
      },
      "mode": "strict",
      "policy_hash": "1804db04fe73e38630217c832871f82c124a8132c61c416754d3a3980e9815f0",
      "profile": "contained",
      "requested_level": 1,
      "unsupported_surface": null
    }
  },
  "command": "atlas",
  "delulu_version": "1.0.0",
  "diagnostics": [],
  "links": [
    "program",
    "authority",
    "effects",
    "capabilities",
    "sandbox_policy",
    "resources",
    "plugins",
    "actors",
    "devices",
    "execution_boundary"
  ],
  "of": "atlas/1",
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  },
  "view": "chain"
}
````

stderr: empty.

</details>

<details>
<summary>B096 — rust-control-build: exit 0</summary>

```json
{
  "argv": [
    "rustc",
    "-O",
    "edge.rs",
    "-o",
    "edge-control.exe"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 3221.4711,
  "timed_out": false,
  "peak_working_set": 12341248
}
```

stdout: empty.

stderr: empty.

</details>

<details>
<summary>B097 — device-sim-plain: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "device_sim.delulu",
    "--grant",
    "console",
    "--grant",
    "actuator=review/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park",
    "--grant",
    "sensor=review/elbow#angle_deg",
    "--broker-profile",
    "sim",
    "--seed",
    "7",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 67.159,
  "timed_out": false,
  "peak_working_set": 8724480
}
```

stdout:

````text
COMMANDED
REFUSED: `angle_deg` = 999 is outside the envelope [-30, 95]
READ 12.0
````

stderr: empty.

</details>

<details>
<summary>B098 — device-sim-sandbox: exit 2</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "device_sim.delulu",
    "--grant",
    "console",
    "--grant",
    "actuator=review/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park",
    "--grant",
    "sensor=review/elbow#angle_deg",
    "--broker-profile",
    "sim",
    "--seed",
    "7",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--sandbox",
    "--report-out",
    "reports/device-sim.json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 2,
  "ms": 18.185,
  "timed_out": false,
  "peak_working_set": 6471680
}
```

stdout: empty.

stderr:

````text
error: a sandboxed run does not apply `--seed` yet. Nothing ran, because a flag that is silently ignored reads exactly like a flag that was applied. Drop it, or run without `--sandbox`.
````

</details>

<details>
<summary>B099 — device-sim-wasm: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "device_sim.delulu",
    "--grant",
    "console",
    "--grant",
    "actuator=review/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park",
    "--grant",
    "sensor=review/elbow#angle_deg",
    "--broker-profile",
    "sim",
    "--seed",
    "7",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--engine",
    "wasm"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 1,
  "ms": 20.0661,
  "timed_out": false,
  "peak_working_set": 8523776
}
```

stdout: empty.

stderr:

````text
error[DL1201]: WASM codegen does not support the method `.actuator` — omit `--engine wasm` to run it on the interpreter
  --> device_sim.delulu:15:15
   |
15 |     let arm = root.actuator("review/elbow")
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the WASM backend cannot compile this
  explain: delulu explain E-DL1201
````

</details>

<details>
<summary>B100 — device-sim-sandbox-no-seed: exit 0</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "device_sim.delulu",
    "--grant",
    "console",
    "--grant",
    "actuator=review/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park",
    "--grant",
    "sensor=review/elbow#angle_deg",
    "--broker-profile",
    "sim",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--sandbox",
    "--report-out",
    "reports/device-sim.json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work",
  "exit": 0,
  "ms": 114.0389,
  "timed_out": false,
  "peak_working_set": 13217792
}
```

stdout:

````text
COMMANDED
REFUSED: `angle_deg` = 999 is outside the envelope [-30, 95]
READ 12.0
````

stderr:

````text
sandbox: the guest is confined — one process only; memory ceiling; processor-time ceiling; killed with the host; no desktop, clipboard or global atoms; a separate identity: a per-run AppContainer with no capabilities — no network, none of the operator's files; wall-clock ceiling
````

</details>

<details>
<summary>B101 — final-survey-check: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu-survey.exe",
    "check"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 1,
  "ms": 11353.8471,
  "timed_out": false,
  "peak_working_set": 36769792
}
```

stdout: empty.

stderr:

````text
stale: docs/survey/SURVEY.md is out of date or missing
stale: docs/survey/survey.json is out of date or missing

The repository changed and its map did not. Run `cargo run -p delulu-survey -- build`
and commit the result with the change that caused it.
````

</details>

<details>
<summary>B102 — final-doctor: exit 1</summary>

```json
{
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "doctor",
    "--check",
    "--json"
  ],
  "cwd": "D:\\nelan\\DeluluLang",
  "exit": 1,
  "ms": 1977.7777,
  "timed_out": false,
  "peak_working_set": 59584512
}
```

stdout:

````text
{"command":"doctor","delulu_version":"1.0.0","diagnostics":[],"doctor":{"checks":[{"detail":"version 1.0.0","name":"delulu","section":"environment","status":"ok"},{"detail":"not compiled in — `root.python(...)` returns Unavailable (DL1307). Build without `--no-default-features` to include it","name":"embedded Python","section":"environment","status":"note"},{"detail":"compiled in (reqwest over rustls); 49 trust root(s) from the platform store","name":"network client","section":"environment","status":"ok"},{"detail":"C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\state (not created yet)","name":"state directory","section":"environment","status":"ok"},{"detail":"embedded","name":"broker mode","section":"environment","status":"ok"},{"detail":"no audit log yet","name":"audit chain","section":"environment","status":"ok"},{"detail":"LEGACY — any process running as this OS user can mint root authority, including over a guard-sealed resource (DISC-1). Turn the gate on with `delulu broker start --require-anchored-roots <anchor-pubkey>`","name":"root issuance","section":"security posture","status":"note"},{"detail":"no signing key in the state directory — an offline anchor is what strict mode's guarantee rests on","name":"anchor key custody","section":"security posture","status":"ok"},{"detail":"inproc for an ordinary `run` — the language and custody in one process, no OS boundary; `run --sandbox` launches a jailed guest instead and the host performs every effect. The sandbox is NOT yet the default (D-V2-26: opt-in until PS-B/PS-C widen the channel, with D-V2-25's default-on as the destination), because the channel cannot carry actors, foreign code, Python, plugins, accelerators or secrets and those runs would be refused (actuators and sensors it carries since P8-01)","name":"backend","section":"sandbox","status":"note"},{"detail":"L1 — L2 is absent: KVM: cannot open /dev/kvm read-write: The system cannot find the path specified. (os error 3)","name":"level available","section":"sandbox","status":"note"},{"detail":"absent — cannot open /dev/kvm read-write: The system cannot find the path specified. (os error 3)","name":"KVM","section":"sandbox","status":"note"},{"detail":"a Job Object with limits: works, a restricted token: works (attempted now; the L1 jail uses them: a guest started under it and opened its channel)","name":"OS primitives","section":"sandbox","status":"note"},{"detail":"host-side egress client for L0 and sandboxed guests alike (PS-B-02): hosts by grant allowlist, each name resolved once host-side and pinned, special-use addresses refused unless `net.special=`, every redirect re-checked, responses bounded, TLS verified against the platform store; a guest has no resolver of its own","name":"network enforcement","section":"sandbox","status":"note"},{"detail":"primitive-table containment on resolved paths; hostile spellings refused (D-NE-29); check-then-open (RW 4.6)","name":"filesystem enforcement","section":"sandbox","status":"note"},{"detail":"a `run --sandbox` guest runs as a per-run AppContainer with no capabilities (PS-B-03) — no network, none of the operator's files, not the state directory (attempted now); an ordinary run, the broker and the CLI still run as this OS user (RW 4.4, category 7)","name":"identity separation","section":"sandbox","status":"note"},{"detail":"every ordinary run is budgeted (PS-B-01, D-V2-25): 1 GiB of memory and 5 minutes of processor time unless `--limits` sets others, sampled every 25 ms by a host watchdog on every engine; a budget is also an authority dimension (PS-B-05): `grants delegate --budget` hands one down, a delegation that names none inherits its parent's, and a `run --lease` is held to its node's; a sandboxed guest is also held by its OS jail; a foreign call is killed after its deadline (NE-21)","name":"resource controls","section":"sandbox","status":"note"},{"detail":"none for an ordinary run (strict mode); `run --sandbox` applies one — `contained` by default, or `dev` / `hostile-agent` with `--sandbox-profile` (D-V2-25)","name":"profile","section":"sandbox","status":"note"},{"detail":"not armed — this host does not require the sandbox. `delulu sandbox require --break-glass-key HEX` requires it for every program `delulu` runs, and a ticket signed by that key is then the only way past it","name":"break-glass","section":"sandbox","status":"note"},{"detail":"none","name":"relaxed restrictions","section":"sandbox","status":"note"},{"detail":"D:\\nelan\\DeluluLang","name":"delulu source tree","section":"repository","status":"ok"},{"detail":"2 behind the tree: SURVEY.md, survey.json. Run `delulu doctor` without --check","name":"survey freshness","section":"repository","status":"problem"},{"detail":"18651 edges, all with a file and line","name":"every edge cites a line","section":"repository","status":"ok"},{"detail":"every endpoint is a node","name":"no edge dangles","section":"repository","status":"ok"},{"detail":"13 workspace members, 9 shipped","name":"totals agree with contents","section":"repository","status":"ok"},{"detail":"1616 nodes, all addressable as `kind:name`","name":"every node has a usable id","section":"repository","status":"ok"},{"detail":"0 error, 6 warning, 37 note — docs/survey/DISCREPANCIES.md","name":"discrepancies","section":"repository","status":"note"}],"regenerated":[],"survey":{"discrepancies":{"error":0,"note":37,"warning":6},"edges":18651,"nodes":1616}},"schema":1,"summary":{"errors":1,"warnings":0}}
````

stderr: empty.

</details>

### MCP wire requests and selected replies

The process exited 0. Requests were newline-delimited JSON over stdin; no shell or external service was invoked by the test programs.

```jsonl
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "review", "version": "0.1"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "check", "arguments": {"paths": ["effect_missing.delulu"]}}}
{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "check", "arguments": {"paths": ["effect_repaired.delulu"]}}}
{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "authority", "arguments": {"path": "--grant=console"}}}
{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "run", "arguments": {"path": "utility.delulu"}}}
```

```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "capabilities": {
      "tools": {
        "listChanged": false
      }
    },
    "instructions": "DeluluLang's compiler, authority analysis, Atlas and sandbox preview as read-only tools. Nothing here runs a program, grants authority or loads code: to run one, use the `delulu` CLI, where the grants are a person's decision.",
    "protocolVersion": "2025-06-18",
    "serverInfo": {
      "name": "delulu",
      "version": "1.0.0"
    }
  }
}
```

Tool list, annotations retained (input schemas omitted here):

```json
[
  {
    "name": "atlas",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "atlas_chain",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "atlas_query",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "authority",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "check",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "examples",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "explain",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "sandbox_policy",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "sandbox_probe",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "schema",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "toolchain",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  },
  {
    "name": "why",
    "annotations": {
      "destructiveHint": false,
      "idempotentHint": true,
      "openWorldHint": false,
      "readOnlyHint": true
    }
  }
]
```

```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "text": "{\n  \"command\": \"check\",\n  \"delulu_version\": \"1.0.0\",\n  \"diagnostics\": [\n    {\n      \"code\": \"DL0501\",\n      \"explanation_id\": \"E-DL0501\",\n      \"message\": \"function `main` performs effect `Write` not declared in its row\",\n      \"repairs\": [\n        {\n          \"authority_widening\": true,\n          \"confidence\": \"exact\",\n          \"edits\": [\n            {\n              \"file\": \"effect_missing.delulu\",\n              \"insert\": \"! {Write} \",\n              \"range\": {\n                \"end_byte\": 50,\n                \"start_byte\": 50\n              }\n            }\n          ],\n          \"id\": \"add_effect_to_row\",\n          \"reason\": null,\n          \"requires_human\": false\n        }\n      ],\n      \"severity\": \"error\",\n      \"spans\": [\n        {\n          \"end\": {\n            \"byte\": 37,\n            \"col\": 8,\n            \"line\": 2\n          },\n          \"file\": \"effect_missing.delulu\",\n          \"label\": \"declared row is here\",\n          \"start\": {\n            \"byte\": 33,\n            \"col\": 4,\n            \"line\": 2\n          }\n        }\n      ]\n    }\n  ],\n  \"schema\": 1,\n  \"summary\": {\n    \"errors\": 1,\n    \"warnings\": 0\n  }\n}",
        "type": "text"
      }
    ],
    "isError": false,
    "structuredContent": {
      "command": "check",
      "delulu_version": "1.0.0",
      "diagnostics": [
        {
          "code": "DL0501",
          "explanation_id": "E-DL0501",
          "message": "function `main` performs effect `Write` not declared in its row",
          "repairs": [
            {
              "authority_widening": true,
              "confidence": "exact",
              "edits": [
                {
                  "file": "effect_missing.delulu",
                  "insert": "! {Write} ",
                  "range": {
                    "end_byte": 50,
                    "start_byte": 50
                  }
                }
              ],
              "id": "add_effect_to_row",
              "reason": null,
              "requires_human": false
            }
          ],
          "severity": "error",
          "spans": [
            {
              "end": {
                "byte": 37,
                "col": 8,
                "line": 2
              },
              "file": "effect_missing.delulu",
              "label": "declared row is here",
              "start": {
                "byte": 33,
                "col": 4,
                "line": 2
              }
            }
          ]
        }
      ],
      "schema": 1,
      "summary": {
        "errors": 1,
        "warnings": 0
      }
    }
  }
}
```

```json
{
  "id": 4,
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "text": "{\n  \"command\": \"check\",\n  \"delulu_version\": \"1.0.0\",\n  \"diagnostics\": [],\n  \"schema\": 1,\n  \"summary\": {\n    \"errors\": 0,\n    \"warnings\": 0\n  }\n}",
        "type": "text"
      }
    ],
    "isError": false,
    "structuredContent": {
      "command": "check",
      "delulu_version": "1.0.0",
      "diagnostics": [],
      "schema": 1,
      "summary": {
        "errors": 0,
        "warnings": 0
      }
    }
  }
}
```

```json
{
  "id": 5,
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "text": "`path` may not begin with `-`: an option cannot be passed through a tool argument",
        "type": "text"
      }
    ],
    "isError": true
  }
}
```

```json
{
  "error": {
    "code": -32602,
    "message": "no tool named `run`"
  },
  "id": 6,
  "jsonrpc": "2.0"
}
```

### LSP wire response

Observed process exit 0, 1,331 output bytes, with DL0501. The input sequence was initialize → initialized → didOpen of the complete effect-missing source above → shutdown → exit, using Content-Length framing. Full response:

````text
Content-Length: 824

{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"codeActionProvider":true,"codeLensProvider":{"resolveProvider":false},"completionProvider":{"resolveProvider":false},"definitionProvider":true,"documentFormattingProvider":true,"documentSymbolProvider":true,"executeCommandProvider":{"commands":["delulu.authority","delulu.guardStatus"]},"hoverProvider":true,"inlayHintProvider":true,"referencesProvider":true,"renameProvider":true,"semanticTokensProvider":{"full":true,"legend":{"tokenModifiers":[],"tokenTypes":["keyword","function","type","variable","string","number","comment","operator","effect","rcap","capability","secret"]}},"signatureHelpProvider":{"triggerCharacters":["(",","]},"textDocumentSync":{"change":2,"openClose":true},"workspaceSymbolProvider":true},"serverInfo":{"name":"delulu-lsp","version":"1.0.0"}}}Content-Length: 401

{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"code":"DL0501","message":"function `main` performs effect `Write` not declared in its row","range":{"end":{"character":7,"line":1},"start":{"character":3,"line":1}},"severity":1,"source":"delulu"}],"uri":"file:///C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/work/effect_missing.delulu"}}Content-Length: 38

{"id":2,"jsonrpc":"2.0","result":null}
````

### Windows simulator run report

This is the report from the successful sandbox invocation without `--seed`, not from the refused seeded invocation.

```json
{
  "command": "run",
  "custody": {
    "mode": "embedded",
    "node": null
  },
  "delulu_version": "1.0.0",
  "devices": [],
  "diagnostics": [],
  "egress": {
    "not_delivered": 0,
    "records": [],
    "records_kept": 0,
    "requests": 0
  },
  "outcome": {
    "exit": 0,
    "ran": true
  },
  "sandbox": {
    "backend": "process",
    "break_glass": false,
    "denied": [
      "actuator command.refused: DL1904 review/elbow: `angle_deg` = 999 is outside the envelope [-30, 95]"
    ],
    "denied_total": 1,
    "fully_enforced": false,
    "generation": "76b72e6c9104bc47cd8a9feb87d33a75724de6c63d1121f4017bc75687eae96b",
    "granted": "contained",
    "host_guarantees": [
      "one process only",
      "memory ceiling",
      "processor-time ceiling",
      "killed with the host",
      "no desktop, clipboard or global atoms",
      "a separate identity: a per-run AppContainer with no capabilities — no network, none of the operator's files",
      "wall-clock ceiling"
    ],
    "level": 1,
    "limitations": [
      "privilege_escalation"
    ],
    "limits": {
      "cpu_seconds": 2,
      "memory_bytes": 134217728,
      "wall_seconds": 5
    },
    "mode": "strict",
    "policy_hash": "488fc885b83b2c7162772aa43f182c78b7e55a12c58d0fc899fcb9b55521a64b",
    "posture": {
      "filesystem_reads": "none of the operator's files",
      "filesystem_writes": "only its own per-run container folder",
      "identity": "a per-run AppContainer",
      "memory": "capped",
      "network": "only the channel",
      "new_programs": "denied",
      "privilege_escalation": "not confined",
      "processor_time": "capped"
    },
    "properties": {
      "egress_confinement": {
        "by": "network: only the channel",
        "state": "established"
      },
      "filesystem_confinement": {
        "by": "filesystem_writes: only its own per-run container folder; filesystem_reads: none of the operator's files",
        "state": "established"
      },
      "host_loss_ends_guest": {
        "by": "killed with the host",
        "state": "established"
      },
      "privilege_floor": {
        "by": "privilege_escalation: not confined; identity: a per-run AppContainer",
        "state": "established"
      },
      "resource_ceiling": {
        "by": "memory: capped; processor_time: capped",
        "state": "established"
      }
    },
    "requested": "contained",
    "requested_level": 1
  },
  "schema": 1,
  "summary": {
    "errors": 0,
    "warnings": 0
  }
}
```

## Appendix C. Benchmark reproduction and records

The sources are in Appendix A. The harness executed the following actual argv once as a warm-up and then 11 times per case, round-robin. It asserted exit 0 and exact output (normalizing CRLF for comparison); identity/check/authority JSON also had to have zero summary errors. No error exit was counted as a fast benchmark. The p90 in §8 is the tenth sorted observation of eleven; it is descriptive, not a precise population estimate. All times are milliseconds and all peak-memory values below are bytes.

```json
{
  "label": "bench-identity",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "--version",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-check",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "check",
    "utility.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-authority",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "authority",
    "utility.delulu",
    "--json"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-utility-interp",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-utility-wasm",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "utility.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5",
    "--grant",
    "fs.read=./input",
    "--grant",
    "fs.write=./output"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-workflow-interp",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "workflow.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-workflow-wasm",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "workflow.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-edge-interp",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "edge.delulu",
    "--grant",
    "console",
    "--engine",
    "interp",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-edge-wasm",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "edge.delulu",
    "--grant",
    "console",
    "--engine",
    "wasm",
    "--no-prompt",
    "--limits",
    "mem=134217728,cpu=2,wall=5"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-edge-dwx",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\target\\release\\delulu.exe",
    "run",
    "output/edge.dwx",
    "--grant",
    "console",
    "--no-prompt"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-utility-python",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Programs\\Python\\Python313\\python.exe",
    "utility.py"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-edge-python",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Programs\\Python\\Python313\\python.exe",
    "edge.py"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

```json
{
  "label": "bench-edge-rust",
  "argv": [
    "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work\\edge-control.exe"
  ],
  "cwd": "C:\\Users\\jesse\\AppData\\Local\\Temp\\delulu-codex-review-20261104-75e41c98\\work"
}
```

Full sample timings and peak working sets:

| Case | 11 elapsed-ms samples | 11 peak-working-set byte samples |
|---|---|---|
| bench-authority | 14.7991, 17.2761, 14.8482, 17.3149, 16.4191, 12.0951, 14.3279, 18.3632, 14.7705, 18.715, 15.5831 | 8245248, 8249344, 8249344, 8245248, 8257536, 8261632, 8237056, 8273920, 8265728, 8241152, 8228864 |
| bench-check | 15.0081, 17.2807, 13.8725, 15.6943, 15.8651, 11.6422, 13.3949, 15.6713, 17.1304, 15.343, 14.9143 | 7835648, 7815168, 7815168, 7815168, 7811072, 7819264, 7827456, 7794688, 7786496, 7823360, 7811072 |
| bench-edge-dwx | 23.1457, 20.9269, 23.4377, 24.2578, 17.0097, 18.2741, 19.0085, 24.5581, 23.2289, 21.8728, 19.1907 | 16322560, 16760832, 16334848, 16289792, 16232448, 15216640, 14958592, 15245312, 16015360, 14999552, 15478784 |
| bench-edge-interp | 54.3026, 55.7224, 56.1129, 59.7005, 53.3431, 58.6207, 52.8608, 66.6684, 58.4016, 56.6271, 56.2859 | 8224768, 8216576, 8237056, 8237056, 8216576, 8224768, 8216576, 8224768, 8212480, 8241152, 8224768 |
| bench-edge-python | 62.8845, 63.4028, 61.6709, 59.7186, 51.3426, 51.5555, 54.7659, 64.5692, 58.5651, 60.3186, 57.1865 | 11182080, 11161600, 11182080, 11210752, 11161600, 11169792, 11169792, 11165696, 11169792, 11165696, 11161600 |
| bench-edge-rust | 10.8606, 9.0726, 10.6952, 10.576, 8.1923, 8.7035, 9.9379, 9.7659, 9.8087, 9.3864, 9.6612 | 4304896, 4304896, 4313088, 4304896, 4300800, 4308992, 4308992, 4300800, 4300800, 4304896, 4304896 |
| bench-edge-wasm | 20.3982, 21.634, 21.7649, 24.5719, 18.0702, 19.3335, 22.2989, 24.4922, 24.041, 21.6601, 20.5745 | 17551360, 17854464, 17494016, 17580032, 17223680, 16072704, 15384576, 15892480, 16846848, 16723968, 16850944 |
| bench-identity | 12.0486, 13.4426, 11.7352, 14.3561, 14.0773, 10.3784, 10.802, 12.5929, 15.5457, 13.611, 13.0511 | 6320128, 6320128, 6316032, 6320128, 6324224, 6316032, 6311936, 6316032, 6311936, 6328320, 6311936 |
| bench-utility-interp | 18.049, 20.866, 18.381, 20.7643, 20.2788, 15.2824, 15.9169, 17.827, 18.3533, 18.8539, 18.7862 | 8597504, 8585216, 8589312, 8577024, 8568832, 8593408, 8581120, 8577024, 8593408, 8589312, 8585216 |
| bench-utility-python | 93.9354, 106.12, 108.3804, 114.962, 84.1979, 92.357, 92.8392, 125.0664, 115.0105, 97.6022, 99.2056 | 13197312, 13373440, 13246464, 13172736, 13459456, 13279232, 13344768, 13377536, 13176832, 13193216, 13197312 |
| bench-utility-wasm | 25.6004, 28.6743, 26.0566, 28.5646, 22.7124, 22.5269, 20.6451, 25.3522, 25.8567, 26.7494, 26.3148 | 19140608, 19144704, 19161088, 18755584, 18698240, 18714624, 17068032, 17616896, 17399808, 18493440, 18739200 |
| bench-workflow-interp | 14.4692, 15.2185, 14.9175, 17.136, 13.873, 12.898, 16.8068, 15.906, 14.6156, 16.9498, 15.4986 | 8359936, 8359936, 8355840, 8384512, 8380416, 8372224, 8355840, 8388608, 8376320, 8364032, 8359936 |
| bench-workflow-wasm | 20.7272, 21.006, 20.4769, 23.8815, 18.802, 17.723, 17.6705, 24.8934, 22.9071, 22.897, 24.7169 | 18329600, 18145280, 17862656, 17993728, 17965056, 17686528, 17240064, 17432576, 17334272, 18067456, 17399808 |

The complete raw records also remain at [benchmark-raw.json](C:/Users/jesse/AppData/Local/Temp/delulu-codex-review-20261104-75e41c98/benchmark-raw.json). The control build command appears in Appendix B; no C++, Mojo, Zerolang, Go, Deno or other competitor benchmark ran. The first fresh version invocation was about 780 ms; later warm identity median was 13.05 ms. One first-run observation is not a reliable cold-start distribution.

## Appendix D. Naming search log and coverage limits

The official browser checks used USPTO wordmark queries for DeluluLang and KingLang and opened the three specified TSDR case records; EUIPO basic queries for both names and the Decart record; the UKIPO specific Veritas record; J-PlatPat's application/registration “Trademark (for search)” field for both Roman-letter names; and KIPRIS domestic trademark queries for both Roman-letter names. CNIPA stopped at its SSO login route; IP India's official search returned 503. No CAPTCHA was solved, account created, trademark filed, domain purchased or repository renamed.

Web searches also covered exact spellings plus programming language, software, trademark and patent terms, including Chinese naming evidence. They are a lead-finding exercise, not an exhaustive multilingual trademark or patent search. No local-script alternative has been chosen for the brand, and local equivalents/phonetics remain unresolved. Similarity, unregistered use, well-known marks, opposition/cancellation history and priority rights require professional assessment.

The public API screen used GitHub `search/repositories` with `q=<name> in:name is:public`, `per_page=100`, then case-insensitive exact repository-basename filtering. All result totals were below 100. Package checks were direct public API GETs. The first scratch script attempt hit a Windows text-decoding error in reading GitHub output; the corrected UTF-8 script completed and produced the results below. No failed decode was treated as an empty search.

```json
[
  {
    "name": "delululang",
    "registry": "npm",
    "url": "https://registry.npmjs.org/delululang",
    "status": 404
  },
  {
    "name": "delululang",
    "registry": "PyPI",
    "url": "https://pypi.org/pypi/delululang/json",
    "status": 404
  },
  {
    "name": "delululang",
    "registry": "crates.io",
    "url": "https://crates.io/api/v1/crates/delululang",
    "status": 404
  },
  {
    "name": "kinglang",
    "registry": "npm",
    "url": "https://registry.npmjs.org/kinglang",
    "status": 404
  },
  {
    "name": "kinglang",
    "registry": "PyPI",
    "url": "https://pypi.org/pypi/kinglang/json",
    "status": 404
  },
  {
    "name": "kinglang",
    "registry": "crates.io",
    "url": "https://crates.io/api/v1/crates/kinglang",
    "status": 404
  },
  {
    "name": "Duvra",
    "registry": "GitHub",
    "total_search_results": 4,
    "exact_repository_basename_matches": []
  },
  {
    "name": "Kevru",
    "registry": "GitHub",
    "total_search_results": 6,
    "exact_repository_basename_matches": []
  },
  {
    "name": "Veylun",
    "registry": "GitHub",
    "total_search_results": 8,
    "exact_repository_basename_matches": [
      {
        "full_name": "Dabot11/Veylun",
        "url": "https://github.com/Dabot11/Veylun",
        "description": null,
        "created_at": "2026-10-18T23:17:54Z"
      }
    ]
  },
  {
    "name": "Dulyn",
    "registry": "GitHub",
    "total_search_results": 29,
    "exact_repository_basename_matches": []
  },
  {
    "name": "DeluluLang",
    "registry": "GitHub",
    "total_search_results": 4,
    "exact_repository_basename_matches": [
      {
        "full_name": "jessesuniljs1-collab/DeluluLang",
        "url": "https://github.com/jessesuniljs1-collab/DeluluLang",
        "description": null,
        "created_at": "2026-07-03T05:52:21Z"
      }
    ]
  },
  {
    "name": "KingLang",
    "registry": "GitHub",
    "total_search_results": 3,
    "exact_repository_basename_matches": [
      {
        "full_name": "publidoo/kinglang",
        "url": "https://github.com/publidoo/kinglang",
        "description": "Language app",
        "created_at": "2024-01-30T19:58:11Z"
      },
      {
        "full_name": "ShubhamBhut/kinglang",
        "url": "https://github.com/ShubhamBhut/kinglang",
        "description": "An interpreted programming language written in rust",
        "created_at": "2024-02-12T12:17:26Z"
      }
    ]
  }
]
```

## Appendix E. Requirement coverage and final integrity

| Original research question | Main answer |
|---:|---|
| 1 — What is it today? | §§3, 7; inventory and actual probes |
| 2 — Why complexity? | §§3.3, 8.5, 10 |
| 3 — Fundamentals missing? | §§4–5; complete sources |
| 4 — Compiler decision? | §§3.1, 6 |
| 5 — Editor decision? | §§6, 14 |
| 6 — CLI direction? | §§6–7, 11, 14 |
| 7 — Advanced features too early? | §§1, 5, 11–13 |
| 8 — Faster/smaller/portable? | §8; raw benchmark samples |
| 9 — Security to preserve? | §§7, 8.4, 12, 15 |
| 10 — Component actions? | §§10, 17 |
| 11 — Competitors? | §9; dated primary links |
| 12 — V2 meaning? | §§11, 16 |
| 13 — Explicit direction? | §§1, 12 |
| 14 — Actual AI-first design? | §§4.3, 7, 14 |
| 15 — First AI users? | §13 |
| 16 — Language/runtime/CLI/API/adapter relationship? | §§3, 6–7, 14–15 |
| 17 — Complete AI task evidence? | §§4, 16; Appendices A–B |
| 18 — Physical readiness? | §15; simulator report |
| 19 — Smallest core? | §5.2 |
| 20 — Performance story? | §8.1; Appendix C; unknowns explicit |
| 21 — Device/source portability? | §8.3; §15 |
| 22 — Third-party adoption? | §14; milestones A–C |
| 23 — Claims never to promise? | §§8.4, 12, 15; claim ledger below |
| Added naming/other-name requests | §18; Appendix D |

| Unjustified claim | Precise alternative |
|---|---|
| “Safe for any AI, AGI or ASI” | Enforces named authority and containment policies for supported programs/deployments; future model requirements remain unknown. |
| “The whole system is formally proved secure” | A stated core fragment has machine-checked results; Rust/host/OS/foreign implementations and assumptions remain part of the trusted system. |
| “Same binary everywhere” | Source or bytecode portability is defined over a tested target/capability/ABI matrix; native binaries are target-specific. |
| “A grant makes a physical action safe” | A grant authorizes an operation; domain correctness, timing and physical safety require independent evidence. |
| “Guaranteed hard real time / flight ready” | Bounded software/simulator behavior has been tested; no hard-real-time, flight-heritage or certification claim is established here. |
| “Faster than Python, Rust or C++” | Named workload, machine, build, inputs, correctness and deployment conditions produced the recorded measurements. |
| “Green CI means everyday usability is complete” | CI verifies its existing gates; the new task probes expose additional usability gaps on a green commit. |
| “No exact trademark hit means the name is free” | The named limited query returned no exact result; similarity, use, territory and other rights remain to be assessed. |

Self-audit: the recommended primary user is an AI agent completing a constrained task. The core roadmap concerns machine input/output, task semantics, authority, reproducibility and target support. The editor is optional; humans remain necessary for review and operations. Physical AI remains in the scope as simulation and staged deployment research, with the missing safety evidence explicitly named. No AGI/ASI capability or certification is invented.

Final tracked-file verification compared all 1,158 baseline hashes: **zero changed tracked files**. Final local HEAD remains `e97c74798fed941a369bbc931ed9d0a8c462753c`; the live read-only `git ls-remote --heads origin master` check also returned that SHA. The initial and final Git status are identical:

```text
# branch.oid e97c74798fed941a369bbc931ed9d0a8c462753c
# branch.head master
# branch.upstream origin/master
# branch.ab +0 -0
? docs/prompt/
```

The fresh final Survey check and doctor were rerun without repairs; their exact results are in Appendix B when available and the health record. The stale-map condition was preserved and reported, not silently repaired. The one initial fetch-metadata deviation is disclosed in §2. No project file, installation, repository destination or release state was changed.

## Appendix F. Original-prompt coverage audit — 4 November 2026 follow-up

**Result:** all 17 required report sections and all 23 research questions are represented. Appendix E maps the questions. The additional A–G architecture proposals and eleven candidate scenarios are mapped below. This is topic/requirement coverage, not an assertion that every possible experiment has been completed. The report explicitly identifies partial, unverified and omitted evidence. The original prompt's research-only boundary remains in force except for the owner's subsequent explicit authorization to expand/copy this report into the checkout.

### Architecture proposals A–G

| Proposal | Verdict / location |
|---|---|
| A — Sharp initial market, retain long-term breadth | Accept: §§11, 13 and 7.2; bounded software agents first, simulation/edge next, real safety-critical deployments later. |
| B — Small core and modular adapters | Accept with measured boundaries: §§3, 5.2, 6–8, 10; preserve compiler/LSP, avoid needless microservices or feature-profile proliferation. |
| C — Useful to AI authors, not just themed | Accept: §§4–5, 14, 16; actual task probes and explicit adoption experiment. Live agent/token comparisons remain unperformed. |
| D — Usable, testable authority and kind-blind trust | Accept: §§7, 8.4, 12, 16; expanded manifest/delegation/refusal plan in §7.1. Authorization is separate from intent and physical safety. |
| E — Portable semantics versus device-specific effects | Accept: §§8.1–8.3, 11, 15; target/ABI/support tiers, local measurements and clear unverified hardware. |
| F — Distinct physical safety/runtime architecture | Accept as staged work: §§7.2–7.3, 12, 15; high-level planning separated from low-level safety control, missing simulator cases named. |
| G — Focused V2 release | Accept: §§11, 16–17; complete programs, reliable authority-bearing machine loop and honestly tested targets. |

### Eleven candidate scenarios

| Scenario | Coverage and actual status |
|---:|---|
| 1 — Agent utility | Executed on interpreter/WASM and process sandbox. Bespoke structured line format; standard JSON parsing remains a gap. |
| 2 — Multi-step workflow | Executed deterministic in-memory retries/traces. No durable orchestrator or live planner evaluated. |
| 3 — Two-agent delegation | Narrowed capability passed between functions; denied parent-scope escape observed. Independent broker-principal agents not run. |
| 4 — Structured interface | Actual CLI JSON, MCP and LSP interactions; schemas, diagnostics and exit/result differences recorded. |
| 5 — Plugin boundary | Pure verified mock plugin allowed, overbroad/ungranted calls refused; unsupported sandbox refused. Signed device-adapter path not rerun. |
| 6 — Edge computation | Bounded aggregation, matching interpreter/WASM/artifact and independent controls. No physical edge board measured. |
| 7 — Robot simulation | Stale/invalid/e-stop proposed-action logic plus actual in-tree actuator simulator with host envelope rejection. No hardware. |
| 8 — Vehicle/drone mission | Shared small mission state machine with low-battery return/stop transition. No navigation/perception/flight controller. |
| 9 — Satellite/ground telemetry | Sequence-based delay/duplicate rejection and bounded queue executed; not a timestamped transport/link simulator. Clock/outage/recovery requirements are proposals. |
| 10 — Review AI code before execution | Authority, within/exceeds/inconclusive, Atlas and repair metadata inspected. Full revision-bound edit/approval/execution transaction remains proposed. |
| 11 — Negative/security cases | No grant, scope escape, missing/malformed input, unsupported device/backend and wall budget exercised; sentinel verified. External cancellation, all resource-limit modes and full alias/adversary matrix not independently rerun. |

### Evidence that remains partial or intentionally unverified

No full live-agent task-completion/token/cost comparison, statistically characterized cold-start distribution, realistic sustained network IO benchmark, long-lived service/restart campaign, independent two-principal delegation run, full `delulu edit` transaction, local cross-OS campaign, real edge board, hardware actuation or certification assessment was completed. The report does not pretend otherwise. These are explicit limits and future acceptance requirements, rather than silently missing sections. Existing CI evidence is attributed to its actual runs; it is not substituted for those new experiments.

Accessible project configuration and memory were used; raw Claude conversations were not reproduced or exhaustively audited. Current source and observed behavior outrank memory. A full current competitor implementation/performance evaluation and comprehensive international trademark clearance were not performed; §9 and §18 identify their primary-source and screening scope.

### Version and branch comparison supplement

Read-only local tracking-ref inspection during this follow-up showed `origin/master` at `e97c74798fed941a369bbc931ed9d0a8c462753c`, prepared `origin/launch/2.0.0` at `728a582a262074228c41072ae88f00880e16f0d9`, and `origin/autosave/laptop` at `4161879a3dfc68c0ed4f32f63937c4994de62290`. These are distinct refs: autosave is a snapshot, not a verified replacement for master; launch is prepared, not published. No `origin/claude/*` tracking refs were returned by that inspection. No fetch, recovery, merge or branch change was performed in this follow-up, and cached tracking refs are not represented as fresh remote verification.

The installed `C:/Users/jesse/.cargo/bin/delulu.exe` and the installed extension's `C:/Users/jesse/.vscode/extensions/delulu-lang.delulu-lang-1.0.0/package.json` retain last-write dates of 7 August 2026. Timestamps alone do not prove their commit; the report therefore distinguishes them from the fresh commit-identified test binary and records the installed binary's actual plain-text version behavior.

### Follow-up file handling

The original external report is retained and the enhanced contents are copied byte-for-byte to `D:/nelan/DeluluLang/docs/review/DeluluLang_Codex_Strategic_Review.md`. Only these report files are written by this follow-up. No source, existing project documentation, index, memory, generated Survey output or configuration is changed; nothing is committed or pushed. The new in-repository report remains untracked. Registration in the repository document index and Survey regeneration are left to a later authorized repository-maintenance session. Survey and doctor were read-only checked before copying and still reported the pre-existing stale-map condition; adding this document must not be described as repairing that condition.
