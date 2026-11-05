# Claude Code Cloud — DeluluLang AI-First Compiler, Interpreter, Execution Pipeline and Human-Centric Complexity Audit

Claude, continue working on **DeluluLang, the language of the future**, in this **Claude Code cloud session**. This is a new, focused follow-up to the independent strategic verification you previously wrote. **Read and verify the current repository before deciding anything.** My main request is a deeply researched, source-backed and experimentally verified audit of everything in DeluluLang that was designed mainly for the traditional human programming workflow, and whether a simpler, faster, safer **AI-first** workflow could replace, optimize or remove any of it.

**At the end, update the existing strategic verification report and all relevant current-state Markdown documentation; open a pull request against the testing repository; and merge the PR if and only if the required checks are green, the review/governance requirements are satisfied and the base has not changed unexpectedly.** Do not implement architectural changes or delete compiler/runtime/source features in this session. This is a research, experiment and **documentation-change PR**, not authorization to carry out speculative removals.

## 1. My vision and the questions I want answered

DeluluLang is **mainly for all types of AI, agents, multi-agent systems, AGI/ASI if and when their needs become knowable, physical/embodied AI, robots, autonomous vehicles, drones, satellites and future evolutions of autonomous computing**. Human developers remain supported, but they are not the primary design center. It should be **simple, fast, efficient, reliable, verifiable, secure, lightweight where possible, and portable across multiple operating systems and devices**, with the **same DeluluLang source program** working on compatible targets without being rewritten wherever technically feasible.

Traditional languages expose a human-oriented process: writing textual code, lexing and parsing, type-checking, error messages, IDEs, debugging, building, generating machine code, linking, packaging, running, profiling and deploying. I want you to investigate **each stage and each human-facing feature**, not assume all of it is necessary for AI authors and AI execution. Could an AI generate a typed, validated machine-readable program directly instead of editing source text? Could DeluluLang run a checked program without a separate compile-to-binary step? Could it interpret, execute verified IR/bytecode, use WebAssembly, precompile a portable artifact, or selectively produce native code? What is actually faster and easier **for useful AI workloads**, and what is still required for soundness, explicit authority, auditability, portability, correctness and safety?

**Do not confuse removing a human interface with removing essential semantics.** A parser may be avoidable for a machine-authored representation but parsing is not the same as type/effect validation. A compiler may be hidden inside `delulu` and still necessary for a backend. Every execution route must preserve DeluluLang's checked effects, capabilities, grants, Guard, broker, containment and audit where those claims apply. AI is not inherently trustworthy; an AI-first workflow must be at least as safe and testable as the existing one.

The repository is becoming very big and complicated. I want **specific, evidence-based recommendations** about what to keep, simplify, consolidate, optimize, make optional, replace, deprecate or remove — and what genuinely new machine-oriented capabilities are worth adding. I also want you to challenge my assumptions, including the possibility that a small CLI-first toolchain with good typed APIs is better than any exotic AI-to-machine-code shortcut.

## 2. Start here; do not treat prior reports as verified truth

The public **testing/development** remote is `https://github.com/jessesuniljs1-collab/delululang-test.git`. Do **not** use the separate future public-release repository, publish a release, tag, force-push or rewrite history. You are in a cloud checkout, not my Windows laptop: do not assume `D:\nelan\DeluluLang` or `C:\Users\...` is accessible.

First read **in full**, in the current checkout and following their precedence:

1. `docs/AI_ONBOARDING.md` and `AGENTS.md` (if present), then `HANDOFF.md` including its newest commission and assistant-memory instructions.
2. `docs/review/DeluluLang_Claude_Independent_Strategic_Verification.md` — your previous independent review. **Update this file** with new research, corrections and verified results.
3. `docs/review/DeluluLang_Codex_Strategic_Review.md` — Codex's prior report. Both reviews are **dated evidence**, not unquestionable facts.
4. `docs/REMAINING_WORK.md`, `README.md`, `INSTALL.md`, `CONTRIBUTING.md`, `docs/for-agents.md`, `docs/DEPLOYMENT.md`, current V2 phase/decision records and most recent `docs/CLOUD_SYNC_LOG.md` entries.
5. `docs/design/CONSTITUTION.md`, `DELULU_CORE.md`, `STABILITY.md`, `SOUNDNESS_AUDIT.md` and the relevant models/tests and `docs/assistant-memory/` entries. Inspect `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` where relevant.

Before experiments, record exact date, Git SHA, branch, upstream divergence, dirty state, current CI results, Rust toolchain, `delulu` build identity and available environment. Fetch/sync in accordance with repository instructions **without overwriting uncommitted work**, inspect `origin/master`, ahead branches, `launch/2.0.0`, autosave snapshots and any unexpected concurrent changes. **One AI development session at a time.** If another author's unexpected commits appear while you work, stop editing and reconcile safely according to the project's policy; never silently overwrite.

**Use `delulu Survey` and `delulu Doctor` at the start and again whenever helpful**. First verify which commands are read-only. Useful documented forms include `cargo run -p delulu-survey -- check`, `cargo run -p delulu-survey -- impact <id>`, `cargo run -p delulu-survey -- query <id>` and `cargo run -p delulu -- doctor --check`. Run `survey build` only in the permitted documentation-update stage, and recheck it afterward. Use Atlas for **DeluluLang-program semantics**; Survey is the **repository impact map**, not Atlas. Prefer a source-built binary, with its actual identity and provenance, to an arbitrary outdated binary on PATH.

Important findings from the last report to **retest if material**: (a) the CLI already contains compiler/checker/runtime machinery, not a detachable standalone compiler product; (b) VS Code is a thin LSP client; (c) a 2026-11-05 review reported important missing basics and agent outcome-contract defects; (d) WASM parity has specific documented gaps; (e) OpenShell was recommended as an **optional backend**, not a replacement for Guard/broker; (f) more than one source-to-execution path already exists. Do not quote historical counts, release numbers or CI status as current without checking them.

## 3. Audit the entire program-to-execution pipeline, from first principles

Draw the **actual current DeluluLang pipeline**, with exact crates/functions/files and trust boundaries, from every supported input through final observable effects:

- Human-authored `.delulu` text, CLI arguments, package/manifests, agent text edits, `edit`/repair, LSP/MCP calls, schemas and other inputs.
- Lexing/tokenization, parsing, AST, names/resolution, types/generics/effects, source maps, diagnostics, permission and authority analysis, package and plugin verification, and any existing DIR or lower-level representation.
- Interpreter execution versus WASM generation and execution; any `.dwx`/`.dpx` artifacts; Wasmtime and its compile/cache behavior; runtime primitives; host calls, FFI/Python integration, device adapters.
- Grants/leases, Guard/broker, host/guest separation, sandbox/OpenShell, audit, cancellation, budgets, successful results, refused effects, application errors, process exits and machine-readable envelopes.
- Packaging, native binary/runtime footprint, cross-platform build requirements and actual deployment profiles.

For **each stage**, answer: What precise job does it do? Is it a semantic/security requirement, an implementation choice, a human-UX feature or an optional optimization? Can a machine-first representation bypass the stage **without** bypassing its guarantee? What would break? What tests prove it? How much latency/memory/binary size/build time does it cost? Which targets need it? What concrete alternative exists? Show a **current versus candidate pipeline** diagram, marking proposed components clearly as unimplemented.

Specifically research and compare these possible execution models:

1. Source text → AST → **direct interpreter** (no user-visible native binary), including overhead and suitability for short-lived agents.
2. AI → **validated structured AST / typed semantic IR** → interpreter, skipping text parsing while keeping type/effect/authority checking.
3. AI or source → checked **portable bytecode / WASM** → interpreter or JIT, including validation, capability imports, WASI and target-specific host adapters.
4. Checked program → Wasm → **precompiled target-specific executable artifact** for smaller deployment runtimes, with provenance, signatures, version binding and security verification.
5. Checked program → **native AOT** or JIT/baseline compiler, and why native code normally requires OS/architecture-specific artifacts.
6. **Hybrid** paths: interpreter for rapid short tasks; cached/baseline compiled execution for repeat runs; optimized AOT only for measured hot or embedded workloads.
7. Machine-oriented structured **operations/workflow plans** that are interpreted or compiled **without inventing a new unrestricted agent framework**.

Answer the exact question: **Must a program go through every conventional stage and become a standalone binary before it can run?** Explain which transformations are mandatory in each chosen model, what is optional, where executable instructions ultimately come from, and why bypassing a UI/parser is not bypassing validation. Distinguish **source portability, portable bytecode, target-specific precompiled code, target-specific native executables, sandbox support and device-driver availability**. Never claim that one native binary runs unchanged on every OS/CPU/device.

Do **not** assume the AI can emit trustworthy raw machine code directly. Evaluate it as a security/reproducibility/debuggability hypothesis; compare with deterministic compiler-generated artifacts and verified representations. Include code-signing/attestation, cache integrity, ABI and toolchain drift, reproducible builds, artifact validation and program provenance.

## 4. Identify *everything human-centric* — classify instead of indiscriminately removing

Inspect the **actual implementation** and create a complete inventory of human-oriented and machine-oriented features. Audit at least:

- Text syntax, keywords, precedence, whitespace, comments, naming, pretty printing, formatting, source files, macros/syntactic sugar and manual code-editing ergonomics.
- Compiler phases, optimization passes, linking, builds, caching, executable packaging and runtime selection.
- Diagnostics, English prose, stack traces, explain text, hints, source highlighting, error codes, repair plans, typed edits, schema output, logs and source locations.
- Interactive CLI help/flags, colors, terminal menus/prompts, completion, REPL, shell commands, human-oriented status summaries versus stable structured responses.
- VS Code extension, LSP, debugging/breakpoints, navigation, hover, completions, syntax highlighting and editor updates.
- Testing, assertions, fuzzing, conformance, formal proofs, CI, profiling, source maps, reproducibility, debugging traces and audit receipts.
- Documentation, tutorials, example programs, repository history, Survey/Doctor/Atlas, incident records, release procedures and human review.
- Plugins, interoperability, process isolation, device simulation, safety gates, compatibility and deployment interfaces.

For each, place it in **one** of these categories: **necessary for correctness/security; useful to both AI and humans; human-optimized but worth retaining optionally; redundant/duplicated; genuinely removable with evidence; or unknown**. Distinguish *human-readable* from *human-only*. Human inspection, explainability, debuggability and independent review may be critical for safe autonomous systems even if AIs are the primary authors.

Produce a **path-level decision table** with current behavior, why it exists, AI value, candidate change, benefits and drawbacks, risk, proof dependencies, user/backward-compatibility effects, approximate measurable savings, rollback and acceptance tests. **Do not delete a feature just because it has a human UI or a large Rust file.** The last review found no proven zero-value components; challenge that conclusion with new evidence rather than assuming it is wrong.

## 5. Design and test the ideal AI-first authoring and execution loop

What is the minimum interface an autonomous agent needs to create, understand, validate, execute, debug, evolve and maintain **complete practical programs**? Consider:

- Versioned machine schemas and a discoverable language/capability reference **matched to the installed toolchain**.
- Structured authoring: validated AST/IR edits, semantic node IDs, content hashes, conflict detection, stable edit transactions, incremental check and rollback; compare this against text edits and existing `delulu edit`, LSP, MCP and Atlas. Do not copy a competitor's architecture without measuring benefits.
- Precise diagnostics and auto-repair plans with no silently widened authority. Stable error categories distinguish **transport success, checked-success, test-discovered-no-tests, expected denial, unsupported target, application failure and infrastructure failure**.
- One documented output protocol (`stdout` versus program output versus host report), exit statuses, bounded output, streaming, backpressure, cancellation, idempotency, replay, timeouts, budgets, correlation IDs, audit receipts and crash/loss explanations.
- Target-aware **preflight**: before an agent writes or runs code, discover which operations work on interpreter, WASM, host OS, sandbox level, plugin profile, simulator and device. Report `supported`, `unsupported`, `unknown` or `conditional`, not optimistic silence.
- Cached and incremental analysis rather than repeated full process startup where it actually improves **measured** total task latency.
- Agent handoffs, versioned manifests, permission proposals, minimal grants and verifiable provenance. Keep trust kind-blind: **the actor being human/AI/robot does not itself determine the grant**.
- Retain a compact, optional human-readable projection and LSP/VS Code for inspection and recovery, if their costs are small relative to their value.

Compare the complete workflow with Python and Rust, and with Zerolang's publicly documented machine-editing approach, controlling for sandbox/authority differences. Measure task completion, correctness, repair cycles, wall time, cold/warm latency, memory, artifacts, token/tool-call overhead where available, and failure classification — **not just one microbenchmark or marketing claim**.

## 6. What to verify against the previous review and remaining work

Re-examine these previously recorded issues as **hypotheses** against the newest source and targeted re-runs: no ordinary args/selected environment or explicit application exit; incomplete JSON parsing/serialization, math and string tools; user-type method ergonomics; interpreter/WASM differences for documented `range`, non-generic higher-order function parameters and a particular `if`-as-value case; missing test path passing with 0 tests; `fix` exit-status inconsistencies; program-vs-host JSON ambiguity; unverifiable source-installed commit identity; null-device success semantics; inaccurate capability hints; missing/late target support discovery; `MCP` subprocess costs and output caps; Survey's limited module granularity; security and portability assertions. **Check whether any have already been fixed; do not add duplicate work rows.**

Also investigate whether the work a traditional compiler does for humans (pretty source/error text, debugger integration) can be made optional **without reducing the quality of authority reports, failure witnesses, differential conformance or human emergency inspection**. Test both positive and deliberately failing cases; inspect actual effects, not just return codes.

## 7. Research external technologies from current primary sources

Browse the web. Cross-check current versions and dates instead of repeating earlier reviews. Include an **evidence-based comparison** with:

- **Vercel Labs Zerolang**: experimental agent-native language, structured diagnostics and program/semantic representation, validated editing and its claimed portability/size; verify claims from source or executed benchmarks, and note experimental/security warnings.
- **Mojo (Modular)**: native AOT compilation, MLIR/LLVM-oriented lowering and heterogeneous AI/CPU/GPU workloads. Clarify that it targets a different performance/compute problem from DeluluLang's authority-first agent execution.
- **Python/CPython**: source-to-bytecode interpretation and runtime costs; why compiling need not mean shipping a standalone native executable.
- **Rust, Go, Java/JVM and modern JIT/AOT runtimes**, and relevant languages such as Koka, Pony, Zig, MoonBit or Roc **only where they illuminate a decision**.
- **WebAssembly specification, Wasmtime** (Cranelift, Winch baseline compiler, Pulley interpreter, caching and pre-compilation), **WAMR** for embedded runtimes, **WASI/Component Model** when relevant, and **LLVM ORC / MLIR** as design alternatives rather than automatically adding new dependencies.
- **NVIDIA OpenShell** and other external isolation backends only where they affect the compiler/runtime boundary and simplification. Reassess the existing optional-backend verdict if new evidence warrants it; never substitute OpenShell for DeluluLang's Authority/Guard/broker semantics without a proof.

Useful official starting links (verify currency):

- https://github.com/vercel-labs/zerolang and https://zerolang.app/docs/intro/
- https://mojolang.org/docs/faq/
- https://www.w3.org/TR/wasm-core/ and https://webassembly.github.io/spec/
- https://docs.wasmtime.dev/examples-fast-compilation.html and https://docs.wasmtime.dev/examples-pre-compiling-wasm.html and https://docs.wasmtime.dev/stability-platform-support.html
- https://github.com/wasm-micro-runtime/wasm-micro-runtime
- https://llvm.org/docs/ORCv2.html and https://mlir.llvm.org/docs/LangRef/
- https://github.com/python/cpython/blob/main/InternalDocs/compiler.md and https://github.com/python/cpython/blob/main/InternalDocs/interpreter.md
- https://docs.nvidia.com/openshell/latest/ and https://github.com/NVIDIA/OpenShell

Cite specific upstream docs, tags, commits, publication dates and benchmark methods. Distinguish **observed locally, read from current source, verified CI, historical documentation, official vendor claim, third-party report, inference and unknown**. Do not present Zerolang's small-binary or speed claims as independently measured by us unless you reproduce them.

## 8. Practical experiments — test the important paths, not an impossible “everything” claim

You said to test everything. Make that actionable: enumerate the complete relevant surface and assess what can actually be exercised in this cloud environment, then prioritize **coverage of all major execution paths and critical failure modes**. Do not claim to have exhaustively tested all programs, OSes, hardware, compilers or future AI.

Use a current source-backed `delulu` build and **scratch directories** for generated programs, binary artifacts and benchmarks. Do not modify production data or invoke real robots, actuators, vehicles, drones, satellites, paid services, external credentials or unsafe network effects. Check support first and use minimal grants. For each experiment, record source, exact command, build SHA, target, exit code, captured output, intended and actual effects, elapsed time, memory where available, verdict and limitations.

At minimum attempt, when supported:

- Short pure calculation and a larger compute program; compare direct interpreter and WASM cold/warm execution.
- A complete structured-input utility, or a minimal failing example that proves the missing JSON/args/outcome blocker.
- Multi-module package, plugins and allowed/refused host effects on both applicable backends.
- Agent's check → typed repair → authority preflight → run → outcome workflow, including a zero-tests case, malformed input and missing capability.
- Cross-engine parity for documented constructs and target-specific refused features; no silent fallback.
- A mock multi-agent delegation and a mock robot/drone/telemetry program; never use real devices.
- Source-size scaling, startup, compilation/validation cost, process-vs-persistent service overhead and the effect of cache/precompilation **only if supported and measured fairly**.
- Evidence from current Linux/Windows/macOS/ARM CI, with actual run IDs and SHAs; name device/RTOS/flight platforms as **unverified** unless tested.

Do not use security checks merely to show a diagnostic: show that forbidden actions **did not occur** and that a timeout/revocation/cancellation is enforced. Do not weaken hooks or tests or grant extra authority for convenience. Avoid carrying undocumented scratch writes into the repository.

## 9. Subagents: use Sonnet 5.5 *if actually available*

Use **Claude Sonnet 5.5 as subagents** for bounded, complementary independent reviews **only if the cloud environment allows you to explicitly select and verify that exact model**. Do not silently substitute another model or assert that it ran when it did not. If unavailable, perform the work directly, record the limitation and do not turn model selection into a blocker.

Suggested separate scopes: (a) compiler/execution paths and actual code references; (b) agent UX, protocols and ergonomics; (c) competitor research and benchmarks; (d) adversarial security/portability review. Keep delegation efficient and bounded, avoid redundant expensive experiments, ask agents to return reproducible evidence, and **independently verify their consequential findings**. Subagents must not perform risky writes or Git mutations or approve their own work. Record models actually used, tasks and evidence, not invented cost figures. An agent separate only by account but controlled by the same author is **not** the independent PR reviewer the contribution policy requires.

## 10. The recommendations I specifically want you to test, not accept blindly

1. **CLI-first but not shell-only:** keep `delulu` as the primary product; expose one reusable typed service layer to CLI, MCP, LSP and direct integrations. Separate operator surfaces from agent-facing structured APIs.
2. **Machine-first authoring is a candidate, not a foregone conclusion:** consider validated semantic edits and typed IR; preserve stable text interchange, provenance, diff/review and reproducibility. Determine whether the incremental benefit justifies a new persistent program database.
3. **Keep the semantics, optimize the path:** effects, Authority, Guard, broker and secure boundaries are differentiators. Remove redundant computations/serializations/UI layers, **not** validation to save a fraction of a millisecond.
4. **Portable semantic core with target profiles:** host-independent computation plus capability-mediated adapters, target preflight and honest unsupported-device refusal. Explore WASM interpreter and precompiled deployment profiles before inventing another compiler backend.
5. **Make everyday AI tasks work first:** agent utility inputs/outputs, JSON, math, strings, complete-program errors, run/test/fix outcomes and backend parity before speculative AGI or fleet features.
6. **Treat human readability as a safety feature when needed:** minimal optional editor, readable diagnostics/source projections and audit trails may reduce rather than add long-term maintenance cost.
7. **Avoid another giant subsystem:** prefer narrow modules, mature maintained dependencies, measured caches and thin protocols. Do not clone an entire compiler/agent runtime/security stack that upstream already solves unless the existing security contract requires it.
8. **No unproven “universal binary”:** choose declared targets and separate portable source/bytecode from target-specific execution, drivers and safety certifications.

For each recommendation provide a **best alternative, counterargument, migration sequence, measurable acceptance criteria, estimated engineering complexity, code locations and rollback**. Make one clear recommended architecture and a realistic fundamentals-first roadmap to a usable V2.0 / later versions.

## 11. Deliverables and permitted repository changes

**Update** `docs/review/DeluluLang_Claude_Independent_Strategic_Verification.md` with a clearly dated **new follow-up section**, preserving original provenance and the prior findings as historical evidence. Do not silently rewrite earlier conclusions: where they changed, say **old claim → new observation → supporting evidence → effect on recommendation**.

Because a full pipeline audit may be too large for the existing report, **also create one focused companion report** at:

`docs/review/DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md`

Cross-link both reports. The companion should have: executive recommendation; exact source-to-effect map; stage-by-stage human-vs-machine classification; execution-model options and direct-execution answer; AI-first interface design; artifact/OS portability matrix; current code/commands and measured probes; competitor comparison; risks/security invariants; prioritized keep/simplify/replace/remove table; release roadmap; failed/unrun cases; primary-source bibliography; and a reproducibility appendix with precise commands and test sources. Be explicit about **proposal vs implemented**.

Update **only relevant current-state `.md` files** where evidence warrants it — particularly `docs/REMAINING_WORK.md` (add/update verified rows, do not relabel untested ideas as defects), `README.md`, `docs/for-agents.md`, `INSTALL.md`, `docs/DEPLOYMENT.md`, current V2 status if materially affected, `HANDOFF.md`/`docs/assistant-memory/` for durable handoff, and `docs/CLOUD_SYNC_LOG.md`. Preserve historical/archive documents. Do not casually change entrenched design/constitution/security/RFC files: follow existing RFC, decision and independent-review rules; recommendations alone belong in the review, not in a silently modified specification. Do not delete Markdown history. Make source claims match actually verified capabilities.

**No `.rs`, tests, configuration, editor code, workflow code, package manifests or implementation changes in this task** except generated Survey bookkeeping explicitly required by the repository's documentation freshness gate. Do not implement a direct-to-binary compiler, remove the parser, rewrite Guard or swap sandbox backends now. Those are proposals for a later separately authorized development task.

## 12. Create PR and merge *on verified green*, with safeguards

This request **explicitly authorizes a documentation PR and its merge** to the existing **`delululang-test` testing repo**, subject to repository governance; it does not authorize a new public release. The old research-only instructions from the earlier review are superseded **only for the documentation changes and Git operations specified here**.

1. Work on your own **`claude/<topic>`** branch based on the appropriate freshly inspected testing-repo base. Preserve unrelated changes and drafts; do not merge autosave snapshots or rewrite pushed history. Confirm `origin` exactly before pushing.
2. Before editing each important area, consult **Survey impact/query** and the current implementation. Keep the diff focused and inspect it for secrets, false assertions, link errors, generated artifacts and banned terminology mandated by project policy.
3. Update the report, relevant Markdown, memory/handoff and the sync log **before the final Survey regeneration**. Regenerate the repository's Survey map **after the last file-changing edit**; if any Markdown changes again, regenerate and recheck the map again. Run `survey check`, `survey findings`, `doctor --check`, link/reference/content checks and other applicable project documentation gates. Run the broader relevant tests/CI, including whatever the repository requires even for documentation changes. Do not use `--no-verify`, weaken tests, disable CI or hide red results.
4. Use the repository's configured commit identity, attribution and handoff conventions. Commit and push **only to** `jessesuniljs1-collab/delululang-test`. Open a PR against its actual integration branch (normally `master`) describing scope, sources, observations versus proposals, exact files changed, validation, risks and pending decisions.
5. **Inspect the completed CI for the exact PR SHA.** Green means the required jobs completed successfully, not “started,” “likely to pass,” “all except one,” or “a past run was green.” Check whether the PR base moved; make sure branch protection, approval and **independent review** requirements in `CONTRIBUTING.md` are satisfied. **Do not self-approve or have an unattended process merge merely because CI is green.**
6. When all these conditions are met and you have the necessary authorization/access, **merge the PR** using the repository-supported method **without force-pushing or bypassing protections**. Where the repo prefers an ancestry-preserving fast-forward, do not substitute a merge commit that violates policy. Verify that the resulting target SHA includes the reviewed diff and that its post-merge checks/state are green or clearly recorded.
7. **If checks are red, pending, unobservable, approvals are missing, merge permissions are unavailable, independent review cannot be obtained, or unexpected concurrent commits appear, leave the PR open and tell me the exact blocker and PR URL.** Do not claim a merge occurred. Do not start endless polling or invent success. Use the PR description/comment for final CI or merge metadata that only becomes known after the documentation commit; if changing a tracked Markdown file for it, rerun the required freshness gates and CI on the new SHA.
8. Never push to the future public-release repository, tag, create a release, trigger autonomous routines, expand public access or distribute microVM/kernel images. Leave a complete sync-log/handoff and a clean, recoverable branch state in accordance with repo policy.

**Take as much time and depth as the cloud session allows. Read the actual code; search and research the web; independently check the previous review; run focused realistic tests; double-verify important claims. I prefer a smaller number of correct, substantiated decisions over a huge list of assumptions. Do not give me a fast answer based only on Markdown or marketing.** Finish with links to the new/updated reports and the PR, the actual PR/merge status, exact commit and CI evidence, top recommendations, what remains unverified and any decisions I need to make.
