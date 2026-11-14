# CODEX COMMISSION — DeluluLang: Evidence-Driven Recovery, AI-First Redesign, Release Readiness, and Independent Fable 5.2 Review

**Date of this commission:** 13 November 2026  
**Repository on my Windows laptop:** `D:\nelan\DeluluLang`  
**Development/testing remote:** `https://github.com/jessesuniljs1-collab/delululang-test`  
**Task type:** deep independent research, current-source inspection, safe experiments, technical decisions, and **one actionable consolidated Markdown plan plus one follow-up prompt for Claude Code CLI (Fable 5.2)**. This is **not** an authorization to rewrite the language, delete components, merge architectural changes, change repository visibility, publish releases, or alter the final/public repository.

## 0. Read this completely. The objective is to end the documentation loop and ship a usable language.

Codex, I want you to investigate DeluluLang as if you were an independent technical lead brought in to rescue a promising but overextended programming-language project. **Do not flatter me, follow the existing roadmap blindly, or produce yet another sprawling wishlist.** Challenge the work of all previous AI agents, including prior Codex and Claude reviews. Challenge this commission where the evidence requires it.

DeluluLang has been under development for months. The Rust codebase is reported to be around **200,000 lines**, but I do not know how much is necessary, reachable, tested, duplicated, stale, security-sensitive, or genuinely useful. Elaborate tooling and autonomy features appear to have outpaced basic programming-language usability. The repository contains extensive plans, V2 phase records, design documents, prompts, release documents, reports and archives. I am concerned that agents are repeatedly producing and citing documentation while ordinary programs remain incomplete, and that some "implemented" features exist only on paper or behind partial/mock paths.

Your job is to establish **what is true at today's actual commit**, recommend what to **keep, repair, finish, simplify, consolidate, defer, replace or remove**, and identify the **shortest evidence-backed sequence from the present repository to a useful third-party testing release**. Make concrete decisions. For each proposed change, explain why it is worth the complexity and how we will know it is done. Prefer implementation over further planning after this review.

**Non-negotiable distinction:** advanced internal architecture is acceptable when it earns its keep. Unnecessary complexity, unreliable results, duplicated subsystems, misleading claims, security gaps, maintenance traps and speculative future work are not.

## 1. What DeluluLang is for; do not reduce AI-first to an AI-themed IDE

DeluluLang is a **serious, general-purpose, AI-first programming language and execution/tooling ecosystem**, designed primarily for AI-to-computer and AI-to-AI work. Its long-term audience includes LLMs, coding agents, tool-using agents, independent/multi-agent systems, embodied AI, robots, autonomous vehicles, drones, edge systems, spacecraft/ground systems and future forms of AI. Humans remain supported as authors, maintainers, operators and auditors, but a conventional IDE-centric human workflow must not dominate every product decision.

I want an AI to be able to discover the language, author a useful program, obtain machine-readable feedback, fix errors, predict required authority, package or delegate it, execute it within enforceable limits, cancel or recover it, inspect the result, and exchange work with another independently operated agent. **The task must actually complete**, not merely pass a synthetic test or appear in a design document.

Preserve or strengthen sound typing, effects, least authority, Guard enforcement, host custody, auditability and honest containment boundaries. **Kind-blind trust:** an actor's permissions and review requirements must follow verifiable policy and change risk, not whether the actor is human or AI. An AI is not inherently trusted. An authority grant does not prove an action is operationally safe, collision-free, timely, certified or suitable for a vehicle/robot.

Future AGI/ASI requirements are unknown: design adaptable contracts, not invented requirements. Distinguish near-term general software agents from long-term safety-critical physical systems. Do not build a new keyword, runtime, middleware or protocol merely because a domain sounds futuristic.

## 2. My product decisions and naming direction

**Keep the name `DeluluLang`; use `.solulu` as the preferred source-code extension.** The identity is deliberate: a seemingly delusional idea becomes an implemented solution. Take this name seriously; a memorable brand can coexist with rigorous engineering. Do not reopen the entire language-naming debate. You may research concrete legal, collision or tooling risks, but flag them factually rather than casually rejecting the name.

The present README describes `.delulu` as the extension. **Do not claim `.solulu` is already supported.** Audit the complete change surface: parser/CLI input discovery, module resolution, package manifests, formatters, LSP, VS Code syntax highlighting, MIME/language IDs, file associations, examples, tests, docs, scaffolding, package registry, build artifacts, scripts, CI and backwards compatibility. Propose a safe phased migration: add `.solulu`, preserve `.delulu` as an explicitly deprecated or indefinitely compatible alias if justified, test mixed-extension packages, and avoid silently breaking existing work. Assess what default should be generated for new users and when old examples may change. Distinguish the **language name**, **source extension**, **CLI executable**, **package manifest**, **WASM artifact extensions**, and **plugin artifact extensions**.

Re-evaluate whether the **compiler as a separately marketed product** and the complex **VS Code product** are needed. Do not confuse removing a separate compiler executable or GUI with deleting the parser, checker, code generator, LSP or compiler internals. Prefer a coherent CLI-first core with optional editor-independent integrations if evidence supports it. Consider a tiny VS Code client that simply exposes the stable language server and run/check commands. Preserve human readability and debuggability where useful.

## 3. Read the repository, not just its self-description

Begin at the actual accessible checkout. If you are in Windows, use the path above. If you are in WSL, a cloud container, or another environment, locate the checkout **without assuming Windows paths exist**. Identify the branch, HEAD SHA, remotes, working-tree status, Rust toolchain, last synchronized commit and latest accessible CI. Respect uncommitted owner work. Do not assume `master`, a branch, or an installed `delulu` is current. Compare ahead branches and `autosave` snapshots as untrusted drafts; do not merge them automatically. If unable to reach a tool or remote, record the limitation instead of inventing results.

Read these **four audits in full**, including appendices, follow-ups, retractions and evidence classifications:

1. `D:\nelan\DeluluLang\docs\review\DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md`
2. `D:\nelan\DeluluLang\docs\review\DeluluLang_Claude_Independent_Strategic_Verification.md`
3. `D:\nelan\DeluluLang\docs\review\DeluluLang_Codex_Strategic_Review.md`
4. `D:\nelan\DeluluLang\docs\review\DeluluLang_Hidden_Complexity_AI_First_Experience_Audit.md`

Then consult the current `README.md`, `INSTALL.md`, `HANDOFF.md`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `SECURITY.md`, `GOVERNANCE.md`, `TRADEMARK.md`, `LICENSE`, `NOTICE`, `docs/AI_ONBOARDING.md`, `docs/for-agents.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `V2_LOG.md`, `V2_DECISION_LOG.md`, active RFCs, current source and tests. Read the latest project instructions for build, test, branch and handoff safety; identify conflicting standing instructions and apply my latest explicit direction where it changes an older preference. This commission authorizes **recommendations** concerning any policy or entrenched file; it does not bypass safe change controls for editing them.

Audit these documentation areas as a **single inventory**, not ten separate mini-reports:

- `docs/`
- `docs/DELULULANG_V2/`
- `docs/design/`
- `docs/prompt/`
- `docs/lang/`
- `docs/book/`
- `docs/release/`
- `docs/archive/`
- `docs/security/`
- `docs/review/`, plus generated references, assistant memory and Survey output.

For every document or group, determine: **authoritative active contract / generated truth / currently actionable work / historical evidence / stale duplicate / aspirational design / misleading or contradictory claim / safe archive candidate**. Trace claims to code, tests, runnable commands and commits. A document's word "done," "closed," "verified," "production-ready" or "formally proven" is not proof of current implementation. Compare README, code, release notes, test coverage and phase-status claims. Produce one concise **canonical source-of-truth map and documentation-reduction plan**, with redirects and archival rather than destructive deletion. Do not add further policy ledgers just to solve policy-ledger bloat.

**Critical time correction:** The audits are mostly dated **4–5 November 2026**. The more recent uploaded README reports subsequent fixes, including the plugin `p.get`/R-Get flaw on **6 November**, package-aware tooling and MCP/outcome fixes, plus compound assignment, math and range work on **8–10 November**. **Reproduce every high-priority historical defect on the latest source before marking it open.** Mark each prior finding `FIXED AND RETESTED`, `FIX CLAIMED BUT UNVERIFIED`, `STILL REPRODUCIBLE`, `REGRESSED`, `PARTIAL` or `OBSOLETE`; include the proof and exact commit. Do not waste the roadmap re-fixing bugs already closed. Conversely, do not believe the README without testing.

## 4. Start with Survey and Doctor, then audit Survey and Doctor themselves

Run the documented forms, if supported on this checkout:

```sh
cargo run -p delulu-survey -- check
cargo run -p delulu -- doctor --check
```

Before examining a proposed source change, use Survey's documented `impact`, `affected-by`, `query` and `diff` modes where applicable. Read `survey.json` and generated maps as **derivative evidence**, never as a substitute for examining Rust implementation. If the tool is stale, lies about graph edges, misses dynamic invocation or fails in ordinary conditions, demonstrate the failure safely. Distinguish broken map generation from broken source.

Evaluate Survey and Doctor as products: false positives/negatives, coverage, drift detection, runtime safety, correctness of exit codes and JSON envelopes, startup costs, portability, incremental performance, UX for agents, what happens when their own dependency data is stale, and whether they can be made smaller or combined. Test whether Doctor checks **the installed binary actually being used**, not just the source checkout.

Do the same for **Atlas** (program-level navigation and changes), **Guard** (authorization enforcement), **Authority** (static vs runtime required-grant calculation), **plugins** (`.dpx`, typed exports, verification, scope and nested loading), **package manager/lockfile**, **MCP server**, **LSP**, **sandbox/host/broker**, **WASM artifacts** (`.dwx`), execution engines, audit chain and release tooling. Ask whether each serves a real use case, has overlapping logic, depends on a feature that is not implemented, can be collapsed into one typed service layer, or needs a hard security boundary that must remain separate.

For each, report one row: purpose; code path; current status; reproducible positive case; adversarial/negative case; actual consumers; maintenance/latency cost; vulnerabilities; verdict; removal/repair prerequisites; concrete acceptance test. **Never infer a feature is useless merely from low test coverage or grep results; prove actual reachability and supported uses before recommending removal.**

## 5. Perform a codebase reality and complexity audit, not only an LOC count

Measure actual tracked Rust/source/generated/test/documentation lines separately; count crates, modules, public APIs, packages, dependencies, feature flags, binary sizes, artifact sizes, compile/test times and dependency or responsibility hotspots. Report by crate and by runtime path. Identify likely dead code, duplicate parsing/checking or error paths, unused adapters, unreachable runtime modes, misleading wrappers, stale experiments, generated-code duplication, security-critical `unsafe`/FFI, panics, silent error suppression, feature-gate inconsistency, deadlocks, unchecked allocations and unnecessary process starts. Follow references, dynamic registries, feature gates, integration tests and production entry points before deciding a function or module is dead.

Study the **full AI workflow** from fresh install to task result, not just hot-loop compiler speed: discovery → scaffold/edit → validate → check authority → preview/refuse → execute → observe → cancel/retry/recover → package/transfer. Find avoidable text parsing, conversions, repeated package scans, process churn, cache invalidation, bundled unused runtimes, mismatched interpreter/WASM semantics, verbose tool schemas, megabyte responses, blocking requests, inconsistent envelopes, unclear grants, and poor agent error repair. Try to disprove each performance theory with a measurement. Quantify cold and warm latency, p50/p95 if feasible, memory/peak RSS, CPU, binary size, token/byte overhead, subprocess count and full task completion; isolate measurement environment.

Compare architectural alternatives: small parser + typed checker + interpreter/WASM; optional precompiled caches; no extra standalone compiler product; LSP and MCP over one versioned service layer; optional native backend only if a verified workload demands it; modular host/device adapters rather than built-in robotics language semantics. Explain why every semantic validation and effect/authority gate cannot simply be bypassed by machine-generated IR. Recommend **no wholesale Rust rewrite** without an exceptional proof that incremental simplification is insufficient.

## 6. Prove ordinary programming works before celebrating autonomy infrastructure

Create **temporary, reproducible test fixtures outside the repo**, unless current tests or scripts suffice. Build a **fresh** executable and record its identity; compare with the installed binary. For every scenario, test `check`, `run`, expected output, error paths, and **both interpreter and WASM wherever supported**; do not disguise unsupported parity. Run under least authority, a missing grant, a denied operation and a bounded timeout. Use mocks/local fixtures instead of real devices or uncontrolled network calls.

At minimum verify end-to-end:

1. Hello world and a CLI program receiving arguments; exit codes, stdout/stderr and stable structured result.
2. Real JSON decode, field access, transformation and encode, including invalid JSON and UTF-8/error reporting.
3. Strings, formatting/interpolation, arrays/lists, maps, records, methods, tuples, generics, recursion, compound assignment, ranges and math where claimed.
4. File read/transform/write with a scoped filesystem grant and deterministic output.
5. HTTP API consumption using an approved local test endpoint if available; DNS, redirects, scope and method refusals.
6. Multi-module package imports, dependencies, tests, lockfile resolution and changed dependency invalidation.
7. Typed errors/results, structured diagnostics, machine-applicable repairs and no silent success with zero tests.
8. Same source program run by interpreter and WASM with equivalent semantics, or an explicit unsupported-target refusal **before side effects**.
9. Plugin load/call using legitimate effects and a negative probe for effect laundering, argument mismatch, nested plugin grants and plugin crash.
10. Two independent agents/principals with separate credentials and least-privilege delegation; observe ownership, expiry, revocation, replay and cross-principal refusal.
11. Long-running task status, cancellation, timeouts, resource budgets, guest crash, broker/supervisor loss and machine-readable terminal outcome.
12. A simple **simulated** robot/actuator task with command envelope limits, a watchdog or emergency stop, and an explicit denied unsafe request. No real-world actuation.
13. Optional `.solulu` prototype via an isolated fixture or compatibility assessment (do not assert support until it exists).
14. A completely fresh third-party setup: clone/build/install, discover CLI and supported target, run an example, obtain useful failure guidance, and upgrade without accidentally using a stale binary.

The important score is **useful tasks completed correctly and safely**, including negative paths and first-try agent ergonomics. Categorize each `PASS / FAIL / PARTIAL / UNSUPPORTED / NOT RUN` with exact commands and observed evidence. A failed task must become one actionable work item, not a new long-term strategy document.

## 7. Security, correctness, platform truth and removal safety

Treat this as a security-sensitive programming language, not an ordinary developer tool. Check whether any prior "proof" relies on a narrower model than marketing suggests. Audit effect laundering, capability forging, `p.get` and plugin signatures, grant scope algebra, confused-deputy cases, side effects before preflight refusal, cancellation after effects, time-of-check/time-of-use, filesystem traversal/symlinks, scope normalization, network/SSRF and redirects, credential exfiltration, logging secrets, trust-boundary failures, signed package/plugin authenticity, serialization and cache-poisoning, stale attestation, malicious dependencies, sandbox downgrade, guest escape claims, host loss, memory limits, crash-only failures and denial-of-service. Include versioned MCP/A2A input and output validation, prompt-injection risks, delegation identity and audit integrity.

Use the project's red-team evidence but verify its freshness. Where available, use applicable existing `cargo test`, clippy, rustfmt, cargo-audit/RustSec, dependency/license analysis, property tests, fuzz targets, Miri, sanitizers, cross-platform CI and mutation/failure-injection tests. Do not install heavyweight tools or run destructive fuzzing unless appropriate to this environment. Never claim a formal or end-to-end security proof from a green test suite or a narrow TLA+ model. Do not weaken checks to make a test pass.

For Windows, Linux, macOS, WSL, x86-64, ARM64, sandbox classes, optional runtimes and edge hosts: give a **feature/support matrix** with evidence and observed failures. Distinguish language-source portability from binary portability, OS-level isolation, optional devices and real-time guarantees. External systems such as NVIDIA OpenShell, Wasmtime/WASI, gVisor and Firecracker may be useful; compare their verified current responsibilities and versions to DeluluLang's, but **never assume an external sandbox replaces the language's effect system, Guard, or safety engineering**. Recheck OpenShell rather than quoting the audits' older version.

## 8. AI-to-AI interoperability: research what already exists first

I want DeluluLang to be valuable for **one AI delegating a task to another AI**, potentially from another model, organization, machine, operating system or security domain. I previously suspected there were no widely used protocols for this. **Verify and correct that premise**: investigate existing interoperable agent protocols and their adoption rather than building a new one reflexively.

At a minimum research and compare, using **dated primary sources and their actual specifications**:

- **A2A / Agent2Agent:** agent discovery/cards, tasks, artifacts, streaming, cancellation, authentication, interop and current stable versions; inspect `https://github.com/a2aproject/A2A` and Linux Foundation project material.
- **MCP / Model Context Protocol:** tool/resource discovery, JSON-RPC, stdio and HTTP transports, authorization, streaming/cancellation, current spec and security; inspect `https://modelcontextprotocol.io/specification` and the official repository.
- Other relevant candidates only where justified: IBM ACP and its relationship to A2A, AG-UI/Agent Client Protocol for front ends, OpenAPI, JSON Schema, AsyncAPI, gRPC/Protobuf, CloudEvents, WebSockets/HTTP, MQTT, NATS, OpenTelemetry, service identity (mTLS/SPIFFE), OAuth/OIDC, and safe artifact packaging.
- **Physical autonomous systems use different layers:** ROS 2/DDS, fleet coordination, V2X and European C-ITS/ETSI message families (CAM, DENM, Collective Perception), and appropriate vehicle/robotics functional-safety standards. The EU AI Act is a regulatory framework, **not** itself an AI-agent communication protocol. Investigate what the EU, ETSI, CEN/CENELEC, ISO and relevant industry groups are actually standardizing; distinguish published standards, drafts and laws, and verify current applicability dates.

For each, separate **agent task interoperability**, **tool access**, **device telemetry/control**, **vehicle-to-vehicle safety messages**, **authentication/delegation**, and **safety/compliance**. Propose the smallest realistic DeluluLang interop architecture: preferably external adapters and standard typed envelopes atop existing protocols, with no new general-purpose protocol unless a specific, unfilled requirement is proved. Analyze capability-bound delegation, provenance, replay protection, idempotency, cancellation, identity, deadlines, rate/resource budgets, inter-agent prompt injection, untrusted artifacts and revocation. Define a simulation-only two-agent cross-process demonstration and acceptance criteria. Explicitly ask: **Why would another agent choose DeluluLang instead of Python/TypeScript/Rust + MCP/A2A + a sandbox?**

Differentiate **existing ecosystem compatibility** from speculative future capabilities for robots, autonomous cars, drones and satellites. A standards interface does not make DeluluLang suitable for hard real-time, safety-critical control or certified deployment.

## 9. Contribution model, license, governance, security files and entrenched rules are open for recommendation

**My updated policy preference: anyone — human, AI, individual, team, company or autonomous software system — may propose ideas, open issues, recommend changes, submit patches and participate without first obtaining a sponsor.** I do not want an AI to need a named human sponsor just to make a suggestion, and I do not want a bureaucratic sponsorship process to discourage contributions. Analyze the current wording carefully: `CONTRIBUTING.md` already states that any party may contribute, but it retains a **named, non-author sponsor for certain risk-class merges**, with a human sponsor preferred. Do not falsely say it bans AI suggestions today. Recommend precise wording that **unambiguously permits unsponsored suggestions and submissions**, and determine whether the sponsor requirement for risky merges should be removed or replaced by simpler, **risk-based independent technical review and verifiable CI/security evidence**. Do not confuse the right to propose with an automatic right to merge, release, hold secrets or bypass reviews. Make the policy equally applicable to human- and AI-authored changes. Identify what actually can work with one maintainer; do not invent nonexistent reviewers or approvals.

Review and freely **recommend** changes to `LICENSE`, `NOTICE`, `TRADEMARK.md`, `GOVERNANCE.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `AGENTS.md`, `CLAUDE.md`, `HANDOFF.md`, constitution, stability/soundness rules, RFC process, branch policies, CODEOWNERS and release checklists. No policy or legacy AI recommendation is immune from criticism. For each suggested change, give benefits, risks, legal/compliance implications, and the exact clause or wording to replace. Distinguish copyright license, patent license, trademark use and the language's file extension. Compare Apache-2.0, MIT, dual licensing or other credible options without assuming a license change will increase adoption. Do not promise trademark clearance or give definitive legal advice; identify counsel-dependent questions. Preserve contributor attribution, security disclosure, honest release status, supply-chain integrity and required license notices. Do not invent a monitored email or imply that an automated responder is an independent reviewer.

## 10. Stop the documentation expansion and establish one execution queue

The most important process repair is **one authoritative, maintained plan whose work items correspond to code and acceptance tests**. Do not solve documentation overload by writing twenty more documents.

Recommend a clear hierarchy such as: checked code/tests/CI evidence → generated Survey/Doctor/reference → current executable work queue → stable normative specs → historical audits/archives. Resolve conflicts against verified source and dates, while preserving intentionally normative constraints until they are explicitly changed. Designate which existing file owns which fact, which should become an index, and which can be archived only after links and source-of-truth references are updated. A plan is not progress. A test that never ran is not evidence. A completed phase that lacks a working end-to-end user task is not necessarily release-ready.

For every proposed work item include **ID, user-visible outcome, severity, source paths, current evidence, prerequisite, minimal implementation slice, tests to add, failure-injection test, acceptance command, supported platforms, estimated difficulty, risk, definition of done and what may safely be removed after it passes**. Avoid duplicate work items across old reviews. Keep an explicit `NOT NOW` section that prevents speculative robotics, new agent protocols, heavyweight editor features and new subsystems from repeatedly returning to the top of the backlog.

Create a release strategy with **two different bars**: (a) an honest, clearly labelled **public alpha/preview for outside testers** with a small supported matrix, and (b) a later stable release with stronger portability/security/compatibility obligations. I want third parties using the basics soon; do not demand aerospace certification for a developer preview, and do not call an alpha production safe. Recommend a narrow first target user (e.g. local coding/tool-using agents working on bounded workflows) and 3–5 compelling demonstrators, plus installation/upgrade instructions, examples, issue templates, versioned schemas, package and source-extension behavior, release artifacts and basic support policy. Establish a **go/no-go decision** tied to evidence, not dates, LOC or green CI alone.

## 11. Mandatory prioritization: what happens next, not what sounds impressive

You must end with ONE clear recommendation and a constrained execution order, for example:

- **P0: stop-the-line integrity/security**, reproducible unsoundness, authority or containment bypass, corrupt artifacts, irreversible data loss, misleading trust claims.
- **P1: public-preview task blockers**, basic language and standard-library completeness, installation identity, `.solulu` compatibility, useful package workflows, consistent structured results, backend parity or honest target refusals.
- **P2: friction and maintenance**, slow or oversized tool results, unnecessary dependencies, duplicated interfaces, stale docs, unused features and missing performance evidence.
- **P3: optional integrations** such as external agent adapters, richer editor experiences, specialized simulations and verified hardware connectors.
- **PARKED:** new universal agent protocols, speculative AGI architecture, safety-critical deployment promises and domain-specific products with no tested demand.

Do **not** assume the example priority labels are correct: change them based on current evidence. Specify the next **three small implementation milestones** a coding agent should execute, in order, each with its exact tests and completion gate. Favor 1–3 verifiable code changes over a sweeping redesign. If a proposed deletion is unsafe now, name the dependency or test that would make it safe later.

## 12. Deliver exactly TWO primary Markdown artifacts, not a forest of reports

### Artifact A — the single combined plan

Create or update **one** canonical document at:

`docs/DELULULANG_AI_FIRST_RECOVERY_PLAN.md`

If the repository already contains an equivalently authoritative active plan, consolidate into that rather than duplicate it; explain the choice in the handoff. This document must be actionable, not a transcript. Include:

1. **One-page executive verdict**; the most important mistaken assumptions; what the project should become.
2. **Verified current state**: SHA, tree/CI/build/tool identities, current V2 status, test evidence and limitations.
3. **Historical-audit reconciliation table**: all consequential claims from the four audits; fixed, still open, regressed, partial, or unverified, with exact witness.
4. **Implementation reality matrix** for ordinary language, CLI, compiler/checker, interpreter/WASM, editor, package, extensions, Guard, Authority, Survey, Doctor, Atlas, plugins, sandbox/broker, MCP, docs/release.
5. **Measured source/complexity/risk audit**, top hotspots, dead/duplicate candidate assessment with confidence.
6. **Keep / fix / simplify / merge / replace / archive / remove / defer** decision matrix, dependency and risk of each.
7. **Documentation truth map** and minimal consolidation/archive proposal.
8. **Naming and `.solulu` migration** with exact compatibility tests.
9. **Contribution/sponsorship/governance/license/security recommendations** including proposed wording and review safeguards.
10. **Independent external research** on AI-agent protocols, EU/ETSI/V2X, competitor comparisons, optional runtimes, dates and primary links; classify known vs hypothetical.
11. **P0–P3/PARKED backlog**, the next three implementation milestones, and explicit `NOT NOW` decisions.
12. **Public alpha go/no-go checklist**, demo programs and CI/platform matrix.
13. **Evidence ledger and reproduction appendix**: commands, outputs, source locations, pass/fail/unknown, and no unearned claims.
14. **Unresolved decisions and reversibility**: what would change your recommendation, cost of being wrong, and one final path forward.

Use Markdown tables only where useful. Make the top one or two pages valuable without reading hundreds of appendices. Link existing evidence instead of duplicating 7,000 lines of a historical review. No speculative new source files or architecture rewrites in this session.

### Artifact B — a real adversarial research prompt for Claude Code CLI (Fable 5.2)

Create:

`docs/prompt/DeluluLang_Fable_5_2_Independent_Reality_Check.md`

Write a **standalone, copy-pasteable, exceptionally demanding Claude Code CLI prompt addressed to Fable 5.2**. Fable must start from the current repository, inspect all four audits and your Artifact A, **independently attempt to falsify your findings**, test the ten highest-risk unresolved claims and your three proposed milestones, investigate meaningful omissions and current external research, audit Survey/Doctor/Guard/Authority/Atlas/plugins/protocol interoperability again, and deliver **a concise independent verdict with disagreements and runnable next steps**, not repeat Codex's text. Tell Fable to verify its **actually configured model/effort and available tools** rather than assuming an environment variable guarantees it. It may use limited sub-agents for independently testable scopes if useful; main agent must reproduce claims before elevating them. Give explicit source/tests/CI discipline, repository sync and owner-work protections, realistic budget/stop conditions, and no fictitious results.

**Fable's prompt must also break the documentation loop.** Its end must identify the **first immediately implementable and independently testable code milestone**, with a follow-up implementation prompt or precise execution handoff, rather than more endless strategic research. Unless I explicitly authorize implementation in that later session, the Fable research session should not silently change security-sensitive code, entrenched policy, releases or remote state. It should recommend exact code/test changes, not just propose more Markdown.

## 13. Working and git safety; reporting

- Start read-only; inspect the tree, permissions, rules and remotes first. Temporary experiments should be isolated and cleaned up. Do not use real credentials, public attack targets, vehicles or hardware. If a test might mutate the checkout or internet state, read what it does first.
- **Only the two requested primary Markdown artifacts** may be created/edited under this commission, plus strictly necessary existing handoff/index/sync-log updates required by repository invariants **if** an edit/commit is actually authorized and safe. If existing rules prohibit a suggested documentation change, report the proposed patch instead of pretending it was made. Avoid churning generated references for a research-only task.
- Keep the development/testing remote. No public release, final-repository push, visibility change, tag, force-push, history rewrite or unauthorized merge. Use an isolated `codex/...` branch for any approved documentation edits and respect latest remote changes. Never overwrite the owner's work. Consult repository governance before opening/merging a PR; **do not merge this review PR automatically**.
- Don't bypass audit/test/CI rules. Re-run Survey and Doctor after permitted edits according to the project's current guidance; if map regeneration would broaden the change beyond the authorized scope, explain that and deliver an uncommitted report or proposed patch instead. Do not falsify a green result.
- Work directly to a useful result rather than asking me for every small decision. When a needed tool or platform is unavailable, continue with the strongest evidence possible and mark the gap. You may recommend changing **any** file, policy or roadmap—including the constitution, license, contribution model, security rules and automation—without treating them as sacred. **Recommendations are not permission to make unreviewed destructive edits.**
- Research public standards and products from **current official sources** (not only previous audit citations); date and qualify version/legislation claims. Don't fabricate access to latest tools or a Fable runtime.

**Final reply to me:** provide links/paths to Artifact A and Artifact B if actually created; one paragraph giving your recommended product direction, the top five evidence-backed blockers, the next three code milestones, `.solulu` and contributor-policy decisions, and what must happen before public testing. State precisely what you did **not** implement. Do not call documentation alone a development milestone.

**The success condition:** after this review, a coding agent should know **exactly what to build next, what tests prove it, what to stop building, and what to remove later**, and I should no longer have to guess which of the project's many documents actually governs the current code.
