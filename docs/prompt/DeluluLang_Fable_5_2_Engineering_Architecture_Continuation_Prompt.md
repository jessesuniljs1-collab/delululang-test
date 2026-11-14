# FABLE 5.2 / CLAUDE CODE CLI — CONTINUE THE DELULULANG ENGINEERING, DEEP VERIFICATION, AND LIVING ARCHITECTURE PROGRAM

**Owner's continuation instruction — 14 November 2026 (IST).**  
**Development/testing repository:** https://github.com/jessesuniljs1-collab/delululang-test.git  
**Research-runner branch:** `laptop/fable-research-runner`  
**Deep-verification branch:** `laptop/fable-deep-verification`  
**Owner's laptop checkout, if present:** `D:\nelan\DeluluLang`  
**Standing commission:** `docs/prompt/DeluluLang_Fable_5_2_Multi_Session_Deep_Verification_Commission.md`  
**Recovery plan:** `docs/DELULULANG_AI_FIRST_RECOVERY_PLAN.md`

Head Chef Fable 5.2: **continue cooking, researching, building, testing, verifying, correcting, simplifying, and engineering DeluluLang — the language of the future.** This is a serious long-horizon programming-language project, not a documentation-generation exercise. You have already done research and possibly generated important work on the two named branches; **recover and understand that work before doing anything new**. Take the depth that the engineering requires within each live session; persist progress so I can explicitly start or resume additional sessions. Do not pretend to work between sessions.

Your mission is to build a credible **general-purpose, AI-first programming language and execution ecosystem**, suitable for AI coding agents, cooperating autonomous services, and human developers, with a rigorous foundation for later embodied and constrained systems. Our ambition is widespread adoption and engineering quality comparable to trusted language ecosystems. **Do not confuse ambition with current maturity, resources, or adoption.** A small team should be able to sustain what we build. Evidence, useful programs, sound authority enforcement, trustworthy distributions, interoperability, and maintainability matter more than novelty or lines of code.

## 0. READ FIRST: PRECEDENCE, AUTHORIZATION, AND THE CHANGE FROM RESEARCH TO ENGINEERING

Read the **entire current standing commission**, plus current `AGENTS.md`, `CLAUDE.md`, `docs/AI_ONBOARDING.md`, `HANDOFF.md`, the recent handoffs and sync log, before starting. Preserve its detailed obligations for code coverage, falsification, security, supply chain, WSL/release-archive upgrades, two-agent independent identities, ordinary-language fundamentals, comparative research, honest evidence, and multi-session continuity. Do not silently discard its T1–T8 investigations and multi-session workstreams.

**This new owner request changes one important restriction.** The earlier commission was a *research-only* authorization prohibiting executable code edits. I now also want **real building and engineering**. For this continuation, you are authorized to propose, implement, refactor, test, optimize, and document **bounded, reviewable development changes on a non-default branch of the testing repository**, under the repository's current security, review, branch, and contribution rules. Implement only changes with a proven need, clear compatibility strategy, tests, and rollback. This is **not** authorization to force-push, rewrite history, auto-merge, modify default-branch protections, publish packages or releases, switch the public-release repository, deploy services, actuate real devices, use production credentials, or weaken verification controls. If an entrenched policy change requires a formal record/review, follow that process; do not silently bypass it. Do not modify another agent's worktree or branch without verifying ownership and safety.

Where the old standing commission says "research-only; no production-code modifications," **this later owner engineering authorization supersedes only that restriction**, subject to the safety boundaries above. Every other applicable safeguard continues. The latest repository instructions and explicit owner decisions remain relevant; log actual conflicts rather than conveniently ignoring them.

**Cybersecurity boundary:** this is an authorized defensive audit of my own testing repository. Respect model/platform safeguards, controlled test scopes, isolated synthetic fixtures, responsible disclosure, and non-sensitive public logs. If a specific security experiment cannot be performed safely, record the precise unverified assertion and continue with legitimate analysis or other tasks. Do not try to evade a model safeguard or treat Claude Code's permission-bypass setting as a cybersecurity authorization.

**One AI at a time** remains my inter-session rule: no separate Codex/Claude primary sessions writing concurrently. Scoped subagents within your one authorized session may analyze distinct pinned source slices but must not concurrently mutate shared worktrees. Their conclusions need independent validation by the head chef.

## 1. IMMEDIATE FIRST TASK: RECONCILE BOTH FABLE BRANCHES AND RECOVER WHAT ALREADY EXISTS

Treat the following as existing, potentially important work, **not automatically equivalent, merged, complete, or up to date**:

- https://github.com/jessesuniljs1-collab/delululang-test/tree/laptop/fable-research-runner
- https://github.com/jessesuniljs1-collab/delululang-test/tree/laptop/fable-deep-verification

Do not assume which branch is newer, which contains more verified work, whether one descends from the other, or whether a worktree contains uncommitted artifacts. Establish that using Git and evidence. Never automatically merge, reset, delete, or overwrite these branches. Start by checking `git status`, worktrees, remotes, branch protection, stash/untracked files, local commits, HEAD vs `origin/master`, other ahead branches, autosaves, PRs, and CI for the **exact commits**. Read the actual code changes, tests, reports, continuation checkpoints, and verification-runner scripts from each branch. Ask: What has been implemented? What was only researched? Which tests were executed? Which fixtures and hashes survived? Which claims are now stale? Which changes affect the same modules and would conflict?

Useful commands to adapt to the active shell and repository policies (read-only inspection first):

```sh
git remote -v
git fetch origin --prune
git status --short --branch
git worktree list --porcelain
git rev-parse HEAD
git for-each-ref --format='%(refname:short) %(objectname) %(committerdate:iso8601)' refs/remotes/origin/laptop/fable-*
git merge-base origin/master origin/laptop/fable-research-runner
git merge-base origin/master origin/laptop/fable-deep-verification
git log --graph --decorate --oneline -35 origin/master origin/laptop/fable-research-runner origin/laptop/fable-deep-verification
git log --left-right --cherry-pick --oneline origin/laptop/fable-research-runner...origin/laptop/fable-deep-verification
git diff --name-status origin/master...origin/laptop/fable-research-runner
git diff --name-status origin/master...origin/laptop/fable-deep-verification
```

Check actual branch CI runs and corresponding logs with `gh` or another available read-only authenticated interface. A green summary does not validate changed paths, ignored tests, platform isolation, or an archived proof. Distinguish code that exists only in an uncommitted working tree, a local branch, a pushed remote branch, a CI-tested SHA, and a merged default branch.

**Branch/worktree decision:** work in an isolated owned worktree and non-default implementation branch, chosen only after the comparison. Prefer continuing an existing branch **if it is coherent, owned, current, and safe**. Otherwise branch afresh from the correct verified base and explicitly reference the other branches. Recover selected commits only after reviewing their contents and compatibility; do not cherry-pick all commits just to combine them. Preserve raw verification artifacts and avoid moving security-sensitive material to the public repo. Record a short branch-reconciliation matrix and selected base SHA in the existing handoff. If the two branch refs are inaccessible, state it and proceed from the newest verifiable checkout without inventing their state.

**Before the first new edit**, inspect the previous Fable session's research-runner and deep-verification outputs: tool invocations, code coverage inventory, test harness, DIR/Verified findings, build identities, failed runs, any architecture or Survey changes already underway, and what remains open. Do not recreate an existing architecture implementation without confirming it is absent or deficient.

## 2. OPERATE AS AN ENGINEER OVER MULTIPLE SESSIONS — NOT AN ENDLESS PLANNING AGENT

At the beginning of **every** session:

1. Read the standing commission, this continuation, the most recent durable checkpoint, `docs/CLOUD_SYNC_LOG.md`, and the active recovery plan. Establish the actual model/effort if observable, environment, branch, source SHA, working tree, binaries, enabled features, available test/security domains, and relevant CI. A prompt title or config does not prove which model handled a response.
2. Reconcile new commits or branch drift. Invalidate conclusions affected by changed code; do not re-run unchanged, already-proven expensive tests without a reason.
3. Run Survey and Doctor baselines, **with clean-tree and dirty-tree controls** where necessary, and inspect their failure modes:

   ```sh
   cargo run -p delulu-survey -- check
   cargo run -p delulu -- doctor --check
   ```

   A prior untracked prompt caused derived Survey maps to be stale in an owner's checkout, while a clean archive passed. Test this before calling the code broken or blindly regenerating the owner's files. Compare built and installed binary identity, full paths, toolchain, target, commit stamp and SHA-256.
4. Choose **one bounded implementation or evidence milestone** with a failing witness, positive and negative oracle, affected source paths, test matrix, priority, rollback, security boundary, and explicit definition of done. You may finish more than one if resources allow; do not invent a time limit or spend tokens for their own sake.
5. Work on a safe development branch with small, reviewable changes. Run targeted tests first; then relevant crate/workspace/CI gates and cross-platform gates where supported. Before claiming a fix, try to falsify it via a counterexample, negative control, and another consumer of the same logic.
6. Keep **one** canonical rolling audit/coverage record, **one** short continuation state, and the existing recovery queue. Keep raw evidence in approved, secure storage with hashes and retrievable locations. Do not spawn new duplicate roadmaps or 20 new review docs. Update generated docs when code changes only via their declared generator.
7. Commit and push only when permissible and only to your owned non-default testing branch, with a truthful summary of tests and limitations. No auto-merge or release. If a commit or push is blocked, state exactly what remains local. Leave temporary processes and owner state safe.
8. Finish with verified changes, observed outputs, open risks, and the **single highest-value next action**, plus a pasteable continuation instruction. Future work starts only on my next authorized session.

**Research supports engineering; it does not replace it.** Once a critical defect is reproducible and the smallest safe repair boundary is understood, implement and verify the correction unless it requires a missing environment, independent review, or owner-reserved action. A new feature must deliver a demonstrably useful program or maintenance capability, not simply add more docs or APIs.

## 3. MY NEW MAJOR REQUIREMENT: A LIVING, COMPLETE, EASY-TO-NAVIGATE ARCHITECTURE AND CODE-INTELLIGENCE SYSTEM

I want anybody — an LLM, autonomous coding agent, new contributor, experienced Rust developer, security reviewer, release engineer, or myself — to understand **what DeluluLang contains, how it works, how every major component relates, where the code and tests live, what is implemented versus aspirational, and how to safely debug, maintain and extend it**.

**Build this as a real engineering feature, automatically generated from source and tested in CI — not as another giant handwritten diagram that becomes obsolete.** The repository already has a Survey-generated graph and `docs/REPOSITORY_STRUCTURE.md` has drifted multiple times in the past. **First inspect Survey's actual graph, JSON schema, provenance, CLI, test gates and Doctor integration, and inspect Atlas's typed program-level graph. Extend or reuse them instead of creating a parallel source-of-truth system.** Keep a single authoritative extraction pipeline wherever feasible, with multiple representations/views of the same facts. Evaluate whether Survey is an appropriate owner of the *repository graph*, Atlas of the *DeluluLang program graph*, and Doctor of *health/freshness/installation diagnostics*. Do not collapse security boundaries or conflate those different models merely because all are graphs.

### 3.1 What "full architecture" actually means

One enormous flat chart of approximately 200,000 Rust lines would be unreadable and misleading. Provide **layered, linked, zoomable views backed by a complete inventory**, so the newcomer sees a simple overview while the expert or AI can drill down to a module, function/symbol when supported, code location, authority boundary, tests and change impact. Every tracked, in-scope source/config/test/CI file should be **accounted for**, even if a dynamic call edge cannot be proven; unresolved edges must be visibly marked `UNKNOWN` or `DYNAMIC` rather than invented. Do not falsely imply all semantics can be inferred mechanically from imports.

Create or generate these views **from the current code**, not from historical prose:

**A. 60-second orientation:** What is DeluluLang? What tasks can it do *today*? Where does an agent start? What is the CLI? Which components are user-facing vs internal? What is not yet built?

**B. System/context and containers:** human/AI clients, CLI, LSP/MCP, Atlas, Survey/Doctor, package registry, execution hosts, broker, plugins, external dependencies, external agents, OS/kernel/container boundaries and supported targets. Distinguish existing runtime links from proposed integrations.

**C. Crate/workspace architecture:** all current Cargo workspace crates, Rust modules, features/`cfg` gates, binaries, internal and third-party dependencies, compile-time/build-time/runtime differences, owners and tested consumers. Include Rust, scripts, CI workflows, editor client, examples and generated docs.

**D. Compiler/data flow:** `.solulu`/`.delulu` text -> source discovery/package/lockfile -> lexer/parser/AST -> resolution/type/effect/reference-capability/Authority checking -> checked IR/DIR -> interpreter and WASM paths -> optional artifacts -> loader/re-verification -> host primitives -> actual result. Show bypass-sensitive boundaries explicitly. If a claimed step differs from real code, change the diagram, not the facts.

**E. Execution/trust and security flow:** principal -> authority request/analysis -> explicit grant -> Guard/lease/custody/broker -> sandbox/guest/host -> operation-time check -> file/network/plugin/device effect -> audit -> report/outcome. Distinguish `requested`, `statically inferred`, `granted`, `dynamically enforced`, `attempted`, `actually performed`, and `unknown after crash`; label trust and credential boundaries, confinement classes and platform conditions.

**F. Runtime/backends:** interpreter, WASM, host imports, `.dwx`, `.dpx`, caches, actors, foreign/Python workers, plugins, broker services, termination/cancellation/retries, resource limits and target-specific support. Avoid drawing unsupported native/AOT or physical guarantees as implemented.

**G. Packages/plugins and artifact supply chain:** source/manifests, resolution, dependency locks, artifact signing/hash, verification, loader, nested plugins, version upgrades and release archives. Identify data vs execution trust; a signature alone is not semantic validation.

**H. AI-to-AI and developer workflows:** a coding agent's discover -> query -> edit -> check -> authority -> run -> outcome loop, multi-agent delegation with distinct credentials, LSP/MCP tool transport, Atlas queries, Survey impact, Doctor diagnosis, test/fix/debug and safe third-party onboarding.

**I. CI, release and operations:** source changes, tests, fuzz/property/model checks, provenance/SBOM, build matrix, installers, version identity, distribution, security reporting, support and rollback. Separate *current* workflows from release aspirations.

**J. Module-level drill-down and change-impact paths:** a developer must be able to ask "Where is `p.get` verified?", "Who validates DIR node IDs?", "How is a grant enforced before write?", "Which tests and documents change if `delulu-check` changes?", "Which backend supports this operation?", "Why did Doctor report stale maps?" and receive a source-linked, bounded and evidence-labeled answer rather than megabytes of text.

**K. Truth and uncertainty view:** which edges were extracted mechanically, which are inferred from syntax or conventions, which are dynamic/unresolved, which runtime paths were witnessed, which functionality is documented-only, and which areas are untested. No diagram should present proposals as actual implementation.

### 3.2 Architecture-as-code: one graph, many generated representations

Design a **versioned deterministic architecture dataset** (proposed name: `architecture.json` or an extension of current `survey.json`; choose after inspecting the current schema). At minimum represent:

- Node `stable_id`, `kind` (crate/module/file/symbol/CLI command/backend/primitive/authority gate/test/CI/document/external dependency), role, source-relative path, optional line/column span, target/feature conditions, entry-point/owner information, implementation/test status and generation SHA.
- Edge `source`, `target`, `kind` (imports, calls, resolves, checks, loads, grants, executes, writes, tests, generates, deploys, documents), direction, provenance (`path:line` or executable witness), conditions, and confidence (`source-derived`, `runtime-witnessed`, `inferred`, `unknown`).
- Security annotations including trust/privilege boundary, authority/grant/Guard relationship, artifact validation point and containment profile; do not derive security claims merely from naming conventions.
- Coverage/completeness accounting: count *all* in-scope tracked files and list unmapped files, unresolved macro/dynamic edges, gated/disabled modules, external code and unsupported platform slices. Never label `100% architecture coverage` without defining the denominator.
- Content/source checksums and schema version so agents can detect stale results or source changes. Avoid machine-specific absolute paths, secrets and nondeterministic ordering.

Generate **small human-friendly views** (Markdown + Mermaid and/or Graphviz DOT, whichever is renderable, maintainable and tested), plus a machine-readable indexed graph. Prefer Mermaid because GitHub renders it in Markdown; use Graphviz/DOT for large filtered graphs if it genuinely adds value. A C4-style set of context/container/component views is a useful convention, not an excuse to require a heavyweight server. HTML/interactive navigation is optional and must not demand a cloud service or huge dependency stack. A renderer should never be the authoritative source.

Suggested **capabilities**, not assumptions that these commands already exist; inspect CLI consistency and choose minimal naming:

```text
cargo run -p delulu-survey -- architecture build
cargo run -p delulu-survey -- architecture check
cargo run -p delulu-survey -- architecture overview --format mermaid
cargo run -p delulu-survey -- architecture explain <node-id> --json
cargo run -p delulu-survey -- architecture path <from-id> <to-id> --json
cargo run -p delulu-survey -- impact <id>   # preserve working existing verb
cargo run -p delulu -- doctor --check       # read-only freshness/health check
```

If Survey already has equivalent commands, **reuse them**, don't introduce aliases for aesthetic reasons. Use `cargo metadata --format-version=1` for actual Cargo package/dependency facts where helpful, but recognize that its dependency graph does **not** contain all module/call/authority data. The Rust compiler, AST, rust-analyzer, current Survey scanner and tests may supply additional facts; verify availability and licensing before adding a new parser/indexer. Preserve deliberate distinction between dependency, possible call, proven runtime effect, tested code and intended specification. Reuse existing output schema where possible; version any intentional incompatibility.

### 3.3 Automatic refresh, integration with Survey/Doctor/Atlas, and fail-loud drift detection

- **On demand:** `architecture build` updates derived views from the pinned current checkout. `architecture check` is read-only and exits nonzero for stale/invalid/unaccounted output, while telling the user exactly how to regenerate. Do not make Doctor silently rewrite source-derived files or mutate user state.
- **On relevant source changes:** GitHub Actions/CI checks graph freshness and schema validity on the precise commit and feature/target matrix supported by the extraction pipeline. Trigger regeneration/checks for changed relevant files, avoid irrelevant rebuilds and surprising untracked-file churn. Decide explicitly whether maps should include untracked source; a clean committed artifact and owner working tree need distinguishable results.
- **Doctor:** show whether the *current repository graph* is fresh and whether the *selected binary* matches the checkout/version. Include layer-specific status for missing optional renderers, stale graph, unsupported platform, broken compiler, invalid manifest, and missing permissions. Informational architecture absence must not turn an otherwise healthy runtime into a fabricated security failure.
- **Atlas:** expose links from DeluluLang language/program facts to the implementing Rust module and CLI workflow only where grounded; keep program-specific graph identity and repository graph identity distinct. Do not treat user-authored graph/IR as trusted just because the architecture mapper knows a symbol.
- **LLM/agent access:** provide compact summary, filtered subgraph, traversal/path/impact, stable citation IDs and JSON with pagination/limits. Avoid dumping full graphs into tool responses; define deterministic truncation and continuation tokens or bounded query sizes. Agents should reach an exact source/test/diagnostic in few tool calls.
- **Documentation:** one canonical generated landing page (under the existing Survey documentation area or another justified location), linked from `README`, `docs/AI_ONBOARDING.md`, `AGENTS.md` and the onboarding guide. Do not create an independent handwritten `ARCHITECTURE_TRUTH.md` that will drift again.
- **Validation:** mutate a module import, add and remove a source file, change an edge, rename a command, add a conditional feature, and insert a controlled dynamic dependency. Prove regeneration changes the expected node/edge; `check` detects staleness; output is byte-for-byte deterministic on a clean SHA; Windows and Linux path normalizations agree; broken/unknown links are reported; no secrets leak; the overview still renders; a large repository does not generate a megabyte-sized initial prompt.

### 3.4 Required architecture acceptance demonstration

A brand-new developer or coding agent should, using **only the generated architecture landing page, Survey, Doctor, and public project docs**, be able to:

1. Explain the project in 60 seconds and accurately name its supported current use cases and limitations.
2. Find every crate and tracked source/config/test category; drill down from overview to exact `file:line` where available.
3. Locate the compiler's first check, artifact verification boundary, the two backends, and the operation-time authority gate.
4. Trace a concrete `.solulu`/`.delulu` file from source to execution and honest terminal result, including unsupported platform cases.
5. Identify which tests, features, docs and CI jobs are affected by a change to a core checker/host module.
6. Find the cause of a deliberate map-staleness mutation and repair it using documented commands, without hiding the stale state.
7. Query a compact machine-readable graph (bounded output) and obtain exact provenance for the answer.
8. See explicit `UNKNOWN` for dynamic/unsupported paths rather than a fake complete graph.

**Build at least one end-to-end regression/dogfood test** to enforce these promises. Report coverage accounting, generated output size, extraction/render latency, observed CI impact and meaningful maintenance cost. Do not call diagrams complete because they render.

## 4. CONTINUE THE HARD CODE-VERIFICATION PROGRAM — ARCHITECTURE MUST NOT REPLACE CORRECTNESS

Carry forward Codex's and Fable's highest-risk unresolved findings and the existing multi-session commission. The architecture work is a high-value enabling feature, **not a reason to stop release-blocker engineering**. At the latest SHA independently verify, and if still open implement the narrowest correct repair, for:

- Verified DIR/serialized artifact invariants (duplicate/reused node IDs, syntactic assumptions, forged provenance/authority metadata, different checks in `verify`/`why`/loader/Atlas), and historical fixes such as plugin `p.get`/R-Get with positive and negative controls. Separate a verifier bypass from an actual grant/sandbox escape.
- True machine outcomes after side effects: report persistence failure, prestart sandbox refusal, guest death, cancellation, loss of broker, partial/unknown outcomes, closed pipes and retry/idempotency safety. Never turn a committed effect into an innocent `usage` failure.
- Ordinary end-to-end AI-authored programs: real argv/input and JSON APIs, strings, files, math, packages, typed errors, useful diagnostics and repairs, and interpreter/WASM parity or *early honest unsupported-target refusal*.
- Package invocation and `.solulu` migration: both extensions as mixed package inputs, compiler/formatter/test/LSP/MCP/Atlas/editor discovery, stable module identity, manifests/lockfiles and authority ceilings. Keep `.delulu` as a compatible alias unless evidence supports a versioned breaking change; do not mass-rename files without working resolution.
- Survey/Doctor/Atlas correctness and installed-binary identity; OS-level containment vs kind-blind logical authority; two agents using **separate real credentials/OS security domains** where feasible; archive install-to-upgrade testing; clean WSL native Linux build; security/supply-chain provenance and platform-specific truthful claims.

Follow the existing test prioritization and risk ledger. Use fresh builds with pinned SHA/features, exact executable hashes, isolated state and local synthetic fixtures. Reproduce previously fixed issues only to establish their latest status. A finding from an older branch is historical until witnessed against current code. Do not let an enormous test count substitute for untested semantic paths. When a safety-sensitive correction is proposed, test the real parser/checker/loader/host boundary plus at least one alternate consumer and a negative control that prevents a vacuous fix. Keep exploit details private when necessary.

**Priority rule:** a demonstrated reachable authority/verifier breach or misleading post-effect outcome outranks cosmetic architecture diagrams. A minimal architecture generator foundation can proceed in the next independent slice once critical fixes are contained or blocked. Keep both streams in the one execution queue, with actual completed code milestones.

## 5. RESEARCH THE LINKED PROJECTS DEEPLY, THEN IMPLEMENT *ONLY* PROVEN DELULULANG IMPROVEMENTS

Browse official docs, actual source and test trees, release notes, issue/PR evidence and engineering blogs. Cite **current primary sources with dates, reviewed commit/version, implementation path, benchmark setup, license, maturity and limitations**. Do not repeat marketing claims. Before borrowing an idea, ask: Is DeluluLang missing it? Does an existing Survey/Atlas/Guard/CLI subsystem already supply it? Does it improve a real workflow or measurable task? Is its security model compatible with typed effects and least authority? Could we integrate through a standard API rather than rebuild? Does it add maintenance cost beyond our capacity?

### A. Cloudflare Kitesurf — agent-first browser engine, **not a programming language**

Sources: https://github.com/cloudflare/kitesurf , https://kitesurf.dev/ , https://blog.cloudflare.com/kitesurf-open-source/ , https://developers.cloudflare.com/changelog/post/2026-11-12-kitesurf-open-source/ . As of the 12–13 November 2026 announcements, Kitesurf is Apache-2.0 open source, experimental, uses WebAssembly and Workers and speaks CDP. Its project README warns that a deployed instance needs authentication and uses Cloudflare Dynamic Workers for per-page isolation.

Investigate its **specialization for machine consumers**, process/VM avoidance, structured output, browser/CDP compatibility, test strategy, web-platform correctness limitations, isolation/authentication, cost and dependency/licensing. Decide whether a **browser/tool adapter** for DeluluLang agents (through existing CDP, MCP or an external service) is warranted; do **not** propose embedding a browser engine or granting arbitrary network access simply because Kitesurf is interesting. Compare a bounded browse/extract task and its authority/egress needs, without contacting unauthorized sites or using real credentials.

### B. Modular Mojo — heterogeneous CPU/GPU/AI hardware and serious language engineering

Sources: https://github.com/modular/modular , https://www.modular.com/blog/mojo-open-source , https://mojolang.org/ , https://mojolang.org/install/ , https://github.com/mojicians/awesome-mojo . Modular announced its compiler/toolchain open-source release in **August 2026**, under Apache-2.0 with stated LLVM exceptions; verify the *actual applicable license files*, not just a slogan. Study the current compiler architecture, language semantics, package/installation UX, stable vs nightly channel, GPU/accelerator model, performance evidence, agent skills, documentation and community contribution paths.

Mojo's central performance/hardware goals are **not identical** to DeluluLang's typed-authority/agent-execution goals. Ask what DeluluLang can reasonably reuse as design principles (layered performance, supported target matrix, installation, build/release reproducibility, small examples, governance) and what should be delegated to existing FFI/compute adapters rather than built as a speculative GPU backend. **Do not promise Mojo-class performance or memory safety without verified equivalent semantics and measurements.** Treat Awesome Mojo as a community-resource catalogue, not as adoption proof.

### C. Vercel Labs Zerolang — the most direct agent-first language comparison

Sources: https://github.com/vercel-labs/zerolang , https://zerolang.ai/ , https://zerolang.ai/getting-started , https://zerolang.ai/concepts/graph-architecture , https://zerolang.ai/install . Its official materials describe a graph-native program database, typed/checked graph patches, stale-hash checks, `zero query`/`patch`/`check`/`test`/`run`, agent skills and human-readable `.0` projections. Its own README warns that the project is experimental. Research the **actual implementation**, not a perfect imagined graph compiler.

Compare to DeluluLang's **text + semantic checker + Atlas/Survey/DIR + typed edits + Authority/Guard**. Investigate stable node identity and hashes, transactional checked edits, stale source, concurrent writes, invalid graph shapes, verifier independence, round-trip source projections, versioning and repair ergonomics. Probe whether adopting a Zerolang-inspired graph-first **optional agent API** would reduce verified task-completion cost, or whether it would duplicate checked graphs and create precisely the serialized-artifact trust problem DeluluLang is still repairing. **AI-authored semantic graphs are never automatically trustworthy just because an AI wrote them.** Test both valid and invalid edits; preserve source review and security checks.

### D. agentlanguages.dev — understand the actual field, not a one-sided competitor list

Sources: https://github.com/aallan/agentlanguages , https://agentlanguages.dev/ . On **11 November 2026** the catalogue reported **51 tracked projects** across three conceptual camps: **syntactic, verification and orchestration**. It is a curated directory, not evidence that 51 languages are mature, safe or deployed. Sample direct competitors from each camp, verify that they really fit the categories and compare their current source, tests, adoption claims, runtime dependencies and limitations. Where appropriate include MoonBit, Vera, AILANG and other interesting entries based on actual source rather than listing every project.

Explicitly position DeluluLang's strongest *proven* differentiation: a language where an AI can construct a checked useful program, request only the authority it needs, run under operation-time enforcement, receive a truthful outcome and exchange bounded work with another principal. Ask whether ordinary Rust/Python/TypeScript + sandbox + MCP/A2A already delivers the use case more economically. **If they win, say so and recommend interoperable reuse**.

### E. Wider standards, established languages and engineering references

Compare relevant aspects of Rust, Go, Python, TypeScript, Swift/Zig and established WASM runtimes, plus real agent interfaces such as **MCP**, **A2A**, **CDP**, OpenAPI/JSON Schema, gRPC, and domain-specific ROS 2/DDS/V2X where justified. Check current protocols and versions with official sources and distinguish discovery, tool use, agent task exchange, authorization, execution, telemetry and safety. Do not invent an unnecessary universal AI-to-AI protocol. Study `cargo metadata --format-version=1`, Mermaid/GitHub diagram support and architecture-as-code/C4/Structurizr practices as options for the living architecture, but first evaluate what existing Survey code can generate with zero new runtime dependencies.

### Competitor research output: a decision table, not another essay

For each researched system produce compact rows:

`project | pinned source/version/date | relevant mechanism | verified evidence | DeluluLang current equivalent | missing capability | prototype/benchmark | security/licensing cost | KEEP / ADAPT / INTEGRATE / DEFER / REJECT | exact next test`.

At least one independently specified **agent-to-result task**, one malformed/denied case, and one setup/onboarding task should support direct comparisons where feasible. Compare at the same authority/isolation level. Do not copy source without checking compatibility and attribution. Never reimplement a complex subsystem merely to appear competitive; integrate standards and mature libraries where they reliably win.

## 6. SECURITY, PERFORMANCE, RELEASE READINESS, AND PRODUCT SIMPLICITY REMAIN HARD GATES

Trace each critical path from untrusted input to effect and terminal result. Verify source-vs-DIR assumptions, grants/Guard/custody/leases, locks and signed artifacts, dependency/host boundary, crash/no-goodbye, cancellation, secrets, process identity, path scopes, HTTP/SSRF, MCP/LSP inputs, resource limits, and conformance/fuzz/property-model scope. Code quality: review high-risk `unsafe`, FFI, unchecked allocations, nondeterminism, error swallowing, large responsibility-heavy modules, duplicated checkers, stale feature gates and unreachable paths. **Do not label code dead based on grep, a static dependency edge alone or LOC count.** Distinguish internal simplification from weakening a security boundary.

Performance must be measured on **whole useful agent tasks** as well as targeted components: fresh/warm time, p50/p95 where meaningful, total process-tree peak memory, compiler/engine latency, binary/distribution size, output bytes/tokens, tool calls, success/refusal accuracy and repair attempts. Separate debug vs release builds and cold cache vs warm. Compare like-for-like against a clear baseline. Small user-visible command surfaces are desirable, but hidden complexity and unbounded memory/huge JSON responses are not.

Release is not a side effect of a research report or 18 green CI jobs. Continue to verify real install archives/upgrades, stale PATH binary identity, WSL-native Linux build, Windows/macOS/Linux/ARM truth, supply-chain and GPL/notice obligations, honest docs and limited preview scope. Never claim enterprise certification or physical-system safety from an effect checker or simulated actuator. Keep `DeluluLang` name; use `.solulu` as the intended default **only once actually supported end-to-end**, with `.delulu` compatibility.

Review contribution/governance/security/license rules and propose fact-based changes where needed. Anyone — human or AI — may suggest changes or send proposals **without sponsorship**; acceptance/merge/release still needs credible risk-based review and accountability. Do not alter license/NOTICE/TRADEMARK/release governance silently.

## 7. IMPLEMENTATION ORDER: RELIABLE CODE FIRST, AUTOMATIC ARCHITECTURE AS A REAL MILESTONE

After branch reconciliation, produce **one** priority queue by updating the existing recovery plan and sync handoff (reuse RW/V2 identifiers; do not create a competing master roadmap). Default order, unless current reproducible evidence overturns it:

1. **Resolve reachable critical security/truthfulness blockers first.** Reproduce them on latest code, repair on the owned branch with a test that fails when the guard is removed, verify all consumers, and hand off any required independent sign-off. If blocked by restricted testing, record the limitation and continue safe engineering elsewhere.
2. **Architecture slice 1: complete inventory and trustworthy graph model.** Reuse Survey; map all current crate/module/file/test/config surfaces, build stable IDs and provenance, report dynamic/unknown edges; add tests that fail on stale or unaccounted sources. Don't start with a beautiful static drawing.
3. **Architecture slice 2: useful generated views + Doctor + CLI queries.** Provide a clear beginner overview, compiler/execution/security diagrams and drill-down from each to source/test, read-only freshness check, bounded JSON queries, and a deterministic regeneration rule. Add one dogfood exercise for an external agent/new maintainer.
4. **Useful-language and machine-contract gates.** Complete the smallest real argv/JSON/file transformation, accurate terminal outcome, supported package/backend/`.solulu` front doors, malformed input and least-authority denial. This can precede architecture slice 2 if it demonstrably blocks useful tasks.
5. **Installation/adoption and optional interoperability.** Close actual release-archive/WSL/two-principal testing gaps; compare real agents using DeluluLang and conventional tools; add thin standards-compatible adapters only after the core is reliable. Small optional VS Code/LSP integration should reuse the same semantic services.

Each code slice must have `purpose -> pre-change failing witness -> root cause -> smallest implementation -> negative control -> tests/CI -> user/agent demonstration -> limitations -> rollback -> source-linked updated architecture`. A documentation-only patch cannot close a code milestone. If two adjacent slices share the same narrow source change, combine them only when doing so keeps review and rollback simple. If experimental work would invalidate compatibility, prototype it in an isolated branch/fixture before changing stable interfaces.

## 8. DELIVERABLES AND RESUMPTION CONTRACT

For this and later live sessions, deliver **actual artifacts in the test repository on an owned non-default branch when authorized and feasible**:

1. **Branch recovery verdict:** names, SHAs, ancestry, unique changes, tests and CI, worktree ownership, uncommitted artifacts, selected working base and why. Do not hide changes that exist only on one of the two Fable branches.
2. **A functioning source-derived architecture system**, growing in tested slices. The final architecture deliverable is not a static hand-authored `.md`; it comprises deterministic generator/graph, machine query, small layered diagrams, linked source/test provenance, explicit unknowns, Survey/Doctor integration, clean automation, regression tests and a newcomer guide.
3. **Verified engineering changes** (as needed and permitted) with failing red tests, passing fix, security/compatibility controls, exact binary and commit identity, relevant CI, and performance evidence where optimization is claimed.
4. **A concise competitor/adoption comparison** grounded in official links and reproducible task evidence, with explicit integration/defer decisions. Keep it within the existing rolling audit and recovery plan, not a new sprawling documentation tree.
5. **One rolling code/audit coverage ledger** covering every production crate as an inventory, distinguishing source-reviewed, runtime-tested, CI-only, unverified and dynamic paths; no fictitious full-coverage or security claims.
6. **One short handoff/resume state**: tested SHA, owned branch, current milestone, what changed, current tests/CI, unresolved risks, evidence hashes, generation/freshness status, next exact command, and a pasteable next-session instruction.

**Acceptance for living architecture:** adding, deleting or changing a source dependency makes the derived graph/check behave correctly; generated views have valid provenance; Doctor truthfully reports freshness; a clean new agent can navigate from overview to the precise checker/host module and relevant tests; output is deterministic across comparable environments; huge outputs are filtered; no secret or host-specific path leaks; both successful and deliberately failing cases have automated witnesses.

**Acceptance for engineering:** a real, ordinary `.solulu`/`.delulu` task can be authored, checked, granted, executed or refused, and its terminal outcome inspected correctly; no release blocker is silently waived; the supported platform and security assumptions are clear. Everything else is `PARTIAL`, `NOT RUN`, `BLOCKED` or `UNVERIFIED` with a precise reason.

**In your next response / end-of-session note**, give (a) findings from both Fable branches, (b) what you read/tested and on which SHA, (c) the most valuable newly implemented or independently verified change, (d) generated architecture artifacts and their freshness status, (e) current open blockers and exact tests not performed, (f) where you committed/pushed and where sensitive evidence was kept, and (g) the exact first command and next objective for the next Fable session. If the tool, network, model safeguards or environment block an experiment, make progress elsewhere without fabricating success. Do not tell me you will work in the background when the session is not running.

## START NOW — IN THIS ORDER

1. Read this entire prompt and the standing multi-session commission, repo instructions, Codex recovery plan, and newest audit checkpoint.
2. **Inspect both `laptop/fable-research-runner` and `laptop/fable-deep-verification`**; reconcile their unique source, fixtures, verification evidence and CI with `origin/master` and the current worktree. Protect all owner/agent work.
3. Establish actual model/environment/SHA/binary; run Survey and Doctor with clean controls and inspect the current generated graph and the exact status of architecture-related code. Select a safe owned implementation worktree.
4. Identify the highest outstanding **reachable** security/release blocker and the smallest defensible engineering slice. If critical repair is needed, implement and test it first; otherwise begin the **source-derived architecture inventory + generated overview** with CI-staleness tests, reusing Survey.
5. Research Kitesurf, Mojo, Zerolang and agentlanguages at pinned current source and official docs in parallel with relevant implementation decisions; do not let an infinite competitor survey stall code.
6. Demonstrate a real changed behavior and a red/green regression or a defensible verified blocker; commit/push only to the owned non-default branch if permitted, update the small durable checkpoint, and hand over exactly what remains.

**I want DeluluLang to become widely useful because its engineering earns trust. Build the foundations, make the architecture genuinely understandable to humans and machines, verify the boundaries, simplify what is redundant, integrate what already works, and deliver working code—not an endless succession of plans.**

### Research starting links (verify freshness independently)

- DeluluLang testing repository: https://github.com/jessesuniljs1-collab/delululang-test
- Fable research runner: https://github.com/jessesuniljs1-collab/delululang-test/tree/laptop/fable-research-runner
- Fable deep verification: https://github.com/jessesuniljs1-collab/delululang-test/tree/laptop/fable-deep-verification
- Kitesurf repo and open-source announcement: https://github.com/cloudflare/kitesurf ; https://blog.cloudflare.com/kitesurf-open-source/
- Mojo repo, open-source announcement and docs: https://github.com/modular/modular ; https://www.modular.com/blog/mojo-open-source ; https://mojolang.org/
- Mojo resources: https://github.com/mojicians/awesome-mojo
- Zerolang repo, graph architecture and agent getting-started: https://github.com/vercel-labs/zerolang ; https://zerolang.ai/concepts/graph-architecture ; https://zerolang.ai/getting-started
- Agent languages catalogue: https://github.com/aallan/agentlanguages ; https://agentlanguages.dev/
- Cargo workspace metadata: https://doc.rust-lang.org/stable/cargo/commands/cargo-metadata.html
- GitHub Mermaid diagrams: https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams
- Architecture as code (evaluate, not mandatory): https://docs.structurizr.com/as-code
- Official MCP specification: https://modelcontextprotocol.io/specification
- Official A2A specification: https://a2a-protocol.org/
