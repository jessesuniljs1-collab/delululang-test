# FABLE 5.2 / CLAUDE CODE CLI — DELULULANG MULTI-SESSION DEEP VERIFICATION, WORLD-CLASS ENGINEERING & RELEASE-RECOVERY COMMISSION

**Commission date:** 14 November 2026 (IST)  
**Expected development/testing remote:** `https://github.com/jessesuniljs1-collab/delululang-test.git`  
**Owner's Windows checkout (only if this machine is that laptop):** `D:\nelan\DeluluLang`  
**Previously audited baseline (not automatically today's HEAD):** `ac2a16202e790657810b819e882b8d721dc0aef4`  
**Primary prior plan:** `docs/DELULULANG_AI_FIRST_RECOVERY_PLAN.md`  
**Task type:** a **multi-session, independently challenged research and verification program**: deep Rust source and runtime inspection; isolated, adversarial and end-to-end tests; cross-platform, supply-chain, standards and competitor research; a traceable coverage and risk ledger; and a prioritized, evidence-backed implementation/release handoff. **No production-code modifications or releases are authorized by this research commission.** The limited permission to maintain research-only records on a dedicated testing-repository branch is defined below. Do not merge, tag, publish a release, change visibility/protection/policy, use real hardware, or access real user secrets without distinct authorization.

---

## MULTI-SESSION PROGRAM CHARTER — READ FIRST; THIS OVERRIDES ONE-SESSION WORDING BELOW

**This is not a single Claude Code CLI session, a quick second opinion, or a fixed number of passes.** The owner explicitly wants Fable 5.2 to pursue the investigation over **as many owner-invoked, sequential sessions as are genuinely required** to understand, recheck and challenge the project. Scope and depth are determined by evidence, not a token/time budget, a target report length, or an arbitrary test count. Continue the same audit program on each subsequent authorized invocation instead of restarting it from scratch or producing a new strategy document.

**Ambition and standard of care.** Treat DeluluLang as a serious contender seeking adoption by individual developers, companies, research labs, autonomous agents and eventually larger organizations. Evaluate it against the **engineering and trust standards appropriate for a widely depended-on programming language**, including mainstream compiler correctness, stable interfaces, security boundaries, reproducible distributions, governance, support, observability, portability, documentation and ecosystem interoperability. Compare honestly with large, well-resourced ecosystems; **do not imply this project already has multinational-company resources, maturity, certification, adoption, or independent review**. Design a process a small team can actually sustain while steadily raising the bar. Avoid copying a corporation's organizational bureaucracy into a single-maintainer repository.

**Depth over speed, but no research treadmill.** Read broadly, then trace deeply. Reopen findings when code changes, a control fails or independent evidence contradicts earlier results. Do not mark a system sound because it passed tests, or bad because it has many lines. Every investigation must produce a reproducible fact, an overturned assumption, a measured trade-off, a bounded risk statement, or an actionable implementation decision. Before researching a new speculative subsystem, ask whether a confirmed blocker prevents useful programs from working. Do not let the demand to research everything postpone the earliest safe code improvement indefinitely.

**Sequential sessions, no imaginary background agent.** A later session starts only when the owner invokes or resumes an eligible Claude Code session/routine. You cannot promise to keep working while no session is running. Preserve enough durable state that a fresh model/context can continue confidently. The owner's one-primary-AI-at-a-time rule remains in effect across Codex, Fable, laptop sessions and cloud runs. Scoped subagents *within* one live session are permitted only under current repository rules; they are not independent sign-off by themselves. Any scheduled routine still follows its own **one-milestone-then-stop** rule; this long-horizon commission does not turn a routine into an endless run.

### Allowed persistence and authorization boundary

The earlier single-session prompt prohibited **all** tracked writes and commits. That would make an extended cloud/laptop investigation fragile. **For this multi-session commission only**, Fable may create or update a **small, bounded set of research and handoff artifacts**, preferably on its own non-default branch in the **testing/development remote** and following the current `AGENTS.md`/`HANDOFF.md` branch convention. If GitHub credentials and branch rules permit, it may commit and push **research-only, nonsensitive documentation and metadata on that branch**, with isolated small commits. This is **not** permission to merge the branch, alter `master`, rewrite history, edit executable Rust or CI workflows, change repository policies, release artifacts or bypass approval gates. If the current repo instructions require further authorization even for a research-only push, honor the restriction; retain the state in a persistent approved scratch destination, provide the exact files for transfer, and mark cross-session continuity blocked. Do not silently claim evidence was preserved remotely.

Use **existing** `docs/CLOUD_SYNC_LOG.md` and the active recovery plan where appropriate; avoid new duplicates. Limit new canonical audit artifacts to: **(a)** one rolling independent audit/coverage ledger, **(b)** one short resumption/handoff state file (structured text or JSON), and **(c)** a nonpublic raw-evidence index referring to safe logs/hashes. Do not publish exploit payloads, bearer tokens, private paths, credentials, personal data or sensitive host logs. A public testing repository is **not** a confidential research vault. Keep private security evidence in approved private storage, report only sanitized facts and hashes in tracked docs.

A multi-session handoff must survive a new process, fresh context, a different machine and a changed `origin/master`. Treat historical session observations as evidence **about their pinned SHA**, never as today's truth. Keep a compact `last verified SHA` and invalidate impacted claims when the source or environment changes. Prefer machine-generated inventories over manually maintained module maps.

### The non-negotiable operating cycle for **every** session

1. **Resume, don't reboot the strategy.** Read this commission, the most recent state and handoff, then current owner directives. Identify actual model/effort, checkout/remotes/HEAD, working-tree ownership, CI drift and whether another AI has advanced the target branch. Recover earlier evidence and its hashes; if inaccessible, mark it unrecoverable rather than treating a prose summary as raw proof.
2. **Select a bounded evidence objective.** Choose one critical trust boundary, a connected workflow, or a contiguous set of modules from the still-open coverage ledger. Write the hypothesis, what would falsify it, platforms, positive/negative controls and the exact acceptance output *before* testing. A session may complete more than one objective when safe; do not aim to spend a usage quota.
3. **Inspect code and execute real tests.** Trace caller → validator → authority check → runtime/host side effect → outcome. Use Survey and Doctor, read source and tests, run isolated fixtures and compare relevant CI. Capture raw commands and outputs with pinned executable hashes. Attack your own interpretation and find an independent negative control.
4. **Cross-check and classify.** Distinguish `source-read`, `executed locally`, `independent oracle`, `CI-only`, `formal-model-only`, `historical`, `inferred`, `not tested` and `unavailable`. Reproduce severe findings in a clean environment and, when practical, another platform or reviewer; do not give a high-risk issue a final verdict from one fragile setup.
5. **Durably checkpoint before leaving.** Update the *same* rolling audit and small handoff state with what ran, what failed, open hypotheses, exact SHA/profile, source paths, raw evidence pointers, impacted docs, one most valuable next test, and where the data can actually be retrieved. Commit/push only the restricted research artifacts if permitted; otherwise export them and flag that handoff needs the owner's transfer. Leave git and temporary services clean.
6. **Stop cleanly when the active session ends.** Never keep uncontrolled background processes, leave secrets in env or logs, or invent progress while offline. A context/usage/time boundary is a checkpoint, not permission to omit uncertainty or claim the program is finished. On a scheduled routine, stop after its authorized milestone even when the wider research remains open.

**No forced project-wide finish date and no performative certainty.** Keep going across authorized sessions until critical unknowns are either resolved with reproducible evidence or are plainly documented as limitations requiring different hardware, a second independent identity, external review or developer implementation. "All 200k lines understood" and "every bug found" are not credible binary states; the goal is measured, risk-driven coverage with transparent residual uncertainty and a defensible readiness decision.

## 0. Read the entire commission. Do not confuse 203 passing tests with a verified language.

Fable 5.2, you are the **head chef** in Claude Code CLI for this independent commission. Your job is not to endorse Codex, Claude, the owner's intuitions, or the existing V2 roadmap. You are an independent systems, compiler, language-security and release reviewer. Try to find a **decisive counterexample** to each major claim, trace the actual code path, compare evidence at the same commit and execution profile, and recommend what to build, repair, retain, simplify, combine, remove or postpone.

Codex's 13 November recovery plan says, in Codex's own words:

> "I ran 203 targeted tests and read the 18 successful CI jobs on the audited commit. Some historical findings still need fresh reproduction. A release archive upgrade, a current WSL build, and/or a two-agent test with separate credentials also remain unverified. The plan marks those gaps explicitly."

**Treat this as a declaration of incomplete investigation, not proof of broad correctness.** The Rust workspace has been described as roughly 200,000 lines. Existing audits, thousands of tests and a successful CI matrix are valuable, but none proves every production route, untrusted data boundary, platform, artifact, install path, dependency or ordinary application is trustworthy. Much of the project was designed and documented before the everyday language became usable. The documentation is larger than is useful to a new contributor, and successive AI agents may have generated plans more quickly than functioning code.

### Your practical question

**Can an independent AI agent install DeluluLang, understand a small supported feature set, create and repair a useful `.solulu` or `.delulu` program, know exactly which authority it needs, run the program or safely receive an unsupported-target refusal, interact with another independently authenticated agent, and obtain an accurate terminal result that will not cause blind retries or duplicated effects?**

Do not substitute architecture diagrams, green CI, marketing, test counts or plans for this demonstration. Do not presume all source is junk because the repository is large; **prove reachability, users, redundancy and behavior before recommending removal.** If the original milestones are wrong, change their order with concrete counterexamples and acceptance tests. If they are right, say why, with **new** evidence beyond Codex's existing witnesses.

### Values, identity and boundaries

- Keep **DeluluLang** as the language and project name, with **`.solulu` as the intended new default source extension**. The story is an apparently delusional idea becoming an implemented solution. Do not restart the naming debate. `.solulu` is not fully supported merely because a loose file parses. Preserve `.delulu` as a supported compatibility alias unless specific evidence justifies a versioned change. The CLI remains `delulu`; artifact extensions `.dwx` and `.dpx`, manifest names, package IDs, LSP language IDs and ABI identities are separate.
- DeluluLang is **AI-first**, not **AI-exclusive**. Target LLM/coding agents, tool-using systems, AI-to-AI workflows and eventually physical/embodied AI. Humans remain authors, operators, maintainers and auditors. No party receives authority or special contributor privileges solely because it is human or AI.
- Retain, verify and where necessary repair static type/effect checking, reference capabilities, Authority analysis, runtime Guard, grants, leases, audit, broker/host custody, fail-closed refusals and honest containment boundaries. An AI's output is untrusted until independently checked. **Machine-generated IR is not a license to bypass validation.**
- A granted action is not necessarily a safe action. Robots, autonomous vehicles, drones and spacecraft require domain-specific safety engineering, timing evidence, fault management, regulation and certification far beyond an effect system. Use only simulations and local mock devices here.
- Challenge any constitution, stability, governance, sponsor, contributor, LICENSE/NOTICE/TRADEMARK, security, build or release policy **as a recommendation** where evidence warrants. This commission does not automatically amend entrenched files or override operating safeguards.
- **Stop the document loop.** The deliverable is one independent report and an executable first-code handoff, not another collection of overlapping roadmaps, ledgers and prompts.

## 1. Environment, model, git and trustworthy provenance come before judgments

1. Report the model and effort level **actually served**, including fallback indications and limitations. Do not claim Fable 5.2 or a particular thinking level because the prompt names it, because a configuration file says so, or because an environment variable is set. If unavailable, continue and mark `UNVERIFIED`.
2. Determine whether you are in Windows, WSL, Linux cloud or macOS. Record date/time, Rust/Cargo/toolchain, machine/OS/architecture, available RAM and test isolation, shell, GitHub CLI/API, external network access, container/VM/KVM/AppContainer support, accessible separate OS accounts, and whether sudo/admin rights exist. Do **not** assume `D:\nelan\DeluluLang` is available in a cloud session or that a cloud host is the owner's laptop.
3. Follow the current `docs/AI_ONBOARDING.md`, `AGENTS.md`, `CLAUDE.md` and `HANDOFF.md` branch/handoff rules. Fetch only the **test/development** remote after checking remotes; compare `git status -sb`, `git remote -v`, `git rev-parse HEAD`, `origin/master`, current branch, ahead branches, PRs, stashes and `origin/autosave/*`. Compare changes by actual ancestry and timestamp; **do not** merge autosaves, reset owner work, rewrite history or silently switch to the separate public-release repository. If master/current branch moves under another active worker, stop editing and resynchronize before further testing; respect the owner's one-primary-AI-at-a-time rule.
4. The Codex plan was authored against `ac2a16202e790657810b819e882b8d721dc0aef4`. On the public GitHub Actions page a scheduled CI run **#833 / run `43409171490`** showed **success** on that SHA on **13 November 2026**; it had an 18-job matrix. This is **dated external observation, not a promise that today's HEAD is unchanged**. Independently fetch actual HEAD and matching push/scheduled/branch CI job results; examine failing and skipped jobs, annotations, coverage boundaries and source SHA. Compare the unfinished `origin/claude/relaxed-brown-2l78i1` branch: the earlier plan cited 14 green jobs and four failing generated-reference/coverage jobs. Its current status must be checked, not copied.
5. Identify every executable selected for testing: built binary, installed PATH binary, VS Code launcher binary, active rust target, profile/features, version JSON, build commit, absolute path and SHA-256. The Codex plan observed an old PATH `delulu` returning plain version `1.0.0`, while a fresh built binary included commit/target. **Never test one binary and attribute results to another.** Set `DELULU_BUILD_COMMIT` using the actual checkout SHA where supported; verify rather than assume injection.
6. Preserve untracked files and scratch. Use temporary directories outside the repository for programs, keys, plugin artifacts, test homes, caches and logs. Isolate `DELULU_HOME`, `DELULU_STATE_DIR`, config and theme overrides where supported. Explicitly record when a tool reaches its normal user state or cannot be isolated. Only use disposable credentials and locally controlled endpoints.
7. Start with the documented Survey and Doctor commands, capturing actual exit codes, stdout/stderr and JSON:

   ```sh
   cargo run -p delulu-survey -- check
   cargo run -p delulu -- doctor --check
   ```

   Codex saw both fail in the owner's checkout because an **untracked research prompt** caused Survey's generated products to be considered stale, while a clean `git archive` passed. Reproduce with a clean archive or detached disposable worktree **before** calling the repository broken. Check whether Survey considers tracked files, ignored files, untracked inputs or all files; test drift deliberately. Do not simply regenerate generated files in the owner's checkout to make an indicator green.

8. Use Survey `impact`, `affected-by`, `query`, `diff`, and `findings`, if supported, before deep reads. Compare the graph to actual `mod`/`use`, call sites, dynamic dispatch, feature gates, plugins, tests and final binaries. Survey is a navigation hypothesis, **not** the final authority on code reachability. Compare Doctor's claimed health with the binary actually invoked, containment profile and configured trust boundary. Always record both the positive control and the known-bad control.

**If current remote or tests are inaccessible:** give the exact obstruction, what was still examined, and a runnable next command. Do not present historical CI, an uploaded plan or a dry-run recipe as a fresh test.

## 2. Read the prior material fully, then dispute it using current Rust

Read the latest checkout versions, **including all appendices, corrections, follow-ups and retractions**, of these four audits:

- `docs/review/DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md`
- `docs/review/DeluluLang_Claude_Independent_Strategic_Verification.md`
- `docs/review/DeluluLang_Codex_Strategic_Review.md`
- `docs/review/DeluluLang_Hidden_Complexity_AI_First_Experience_Audit.md`

Read in full the new `docs/DELULULANG_AI_FIRST_RECOVERY_PLAN.md`, the original `docs/prompt/DeluluLang_Fable_5_2_Independent_Reality_Check.md` (this commission supersedes its operating instructions for this entire multi-session audit program), and the Codex commission if present in `docs/prompt/` or supplied separately. Read these current contracts and operational references where present:

- `README.md`, `INSTALL.md`, `HANDOFF.md`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `LICENSE`, `NOTICE`, `TRADEMARK.md`, `rfcs/`, `CODEOWNERS`, `.github/` workflow/release/protection definitions.
- `docs/AI_ONBOARDING.md`, `docs/for-agents.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md`, `docs/CLOUD_SYNC_LOG.md`, `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, `V2_LOG.md`, `V2_DECISION_LOG.md`, `docs/design/CONSTITUTION.md` (if present), `STABILITY.md`, `SOUNDNESS_AUDIT.md` and other normative design documents **actually cited by current source/tests**.
- `docs/`, `docs/DELULULANG_V2/`, `docs/design/`, `docs/prompt/`, `docs/lang/`, `docs/book/`, `docs/release/`, `docs/archive/`, `docs/security/`, `docs/review/`, generated `docs/reference/`, generated `docs/survey/`, assistant memory and maintenance records; inspect **all filenames and ownership**, then read high-impact current sections rather than reproducing 200,000 lines of obsolete narrative.
- The source, tests, scripts, manifests, examples, packages, FFI boundaries, CI and release files underlying every consequential claim.

**Source priority:** actual pinned source and reproducible runtime observation > relevant current test/witness > CI with exact SHA/config > generated metadata (if freshness verified) > normative contract for intended compatibility > historical audit at its own SHA > README/status claim > unsourced speculation. A contract still governs intended stability even when code violates it; that violation must be explicit rather than silently accepting current behavior.

**Historical-finding reconciliation:** label each important old issue `FIXED AND RETESTED`, `FIX CLAIMED BUT NOT RETESTED`, `OPEN AND REPRODUCED`, `REGRESSION`, `PARTIAL`, `SUPERSEDED`, or `UNKNOWN`. Do **not** revive already fixed R-Get/p.get, range, compound assignment, f-string, math, bitwise, package-aware LSP/MCP or `.dwx` issues just because a November 4–5 audit says they were open. Conversely, do not believe a November 6–13 fix because README or V2 status says "closed." For each disputed case save the old SHA, new SHA, test, raw outcome, code location and why it changed.

## 3. Complete a codebase *coverage inventory* and a risk-driven source audit; do not stop after targeted tests

Codex's 203 tests covered selected integration suites. This follow-up must **inspect the codebase systematically**, rather than only repeating those suites. You need not manually narrate every line. You **must account for every tracked production Rust crate and module**, identify actual entry points and external boundaries, mark what was traced deeply, and explain what remains unexamined.

### 3.1 Mechanical inventory — inspect all production crates

First generate a reproducible inventory from `Cargo.toml`, all crate manifests and the tracked tree. Recount production `.rs` lines separately from tests, benches, examples, generated sources, docs, vendored code and inactive paths. Record per crate: LOC, modules, binaries, entry points, public APIs, linked features, `cfg` profiles, direct/transitive dependencies, callers/consumers, test targets, fuzz/property models, unsafe/FFI, source-to-artifact paths, platform gates and last known execution witness. Do not infer 200k LOC all belongs to production.

Include at least these workspace surfaces (names are orientation, **verify current actual crate list**):

| Surface | Deep-read focus and primary questions |
| --- | --- |
| `delulu-syntax` | lexer, parser, AST, identifiers, string/UTF-8/bidi rules, formatting, source spans, direct and artifact paths; can serialization bypass lexer assumptions? |
| `delulu-check` | resolve, type/effect inference, Authority, reference capabilities, package and lock resolution, DIR serialization/rechecking, plugin verification, source/IR parity, scope algebra, unknown/skip branches. |
| `delulu-diag` | diagnostic registry, truthful exit/success semantics, structured envelopes, byte/code repairs, invalid schema, safe rendering of control characters. |
| `delulu-runtime` | interpreter, primitives, capability custody, actors, plugin engine, Python/foreign/device adapters, resource limits, races, panics, cancellation and host effects. |
| `delulu-wasm` | codegen, target parity, `host.rs` and imports, artifact signatures, `.dwx`/`.dpx`, caching and deserialization, memory/fuel/timeouts, fallback/downgrade. |
| `delulu-broker` | Authority lattice, grants, broker/Guard, leases, secrets, revocation, audit, federation, principal identity and lost-host effects. |
| `delulu-atlas` | graph provenance, stable identities, authority facts, snapshot freshness, query/edit consistency, actual consumers. |
| `delulu-survey` | graph extraction, dynamic omissions, stale/untracked detection, health logic, machine output and impact accuracy. |
| `delulu` CLI | *every* user-visible route: dispatch, run/check/test/fix/format/package/build, LSP, MCP, doctor, brokerd/IPC, guest/jail/microVM, release/signing/deploy; command parsing and machine result contract. |
| `delulu-conform`, `delulu-measure`, `delulu-fuzz` | nonvacuous conformance, corpus diversity, generator reach to high-risk constructs, mutation/fault/coverage gates, measurement correctness. |
| `delulu-registry` and packaging/distribution | manifest and trust boundary, dependencies, lockfiles, signing, registry resolution, tar/zip/install/upgrade and supply-chain verification. |
| `editors/vscode`, `scripts`, `.github`, examples and generated references | activation and binary selection, `.solulu` discovery, release workflow assumptions, stale docs, reproducibility and divergent CLI outcomes. |

Do not assume Survey covers all Rust files merely because it generated a module graph. Follow actual `Cargo.toml` members, nested `mod`, macros, `include!`, `build.rs`, FFI, dynamically loaded code, OS-specific files, build profiles, integration tests, callbacks and platform-dependent code. Note features **compiled but not callable**, **reachable only from tests**, **advertised but gated**, and **real but omitted from docs** separately.

### 3.2 Deep source review and independent negative tests

Prioritize in this order: (a) untrusted input and artifact ingestion, (b) effect/Authority correctness, (c) OS custody/sandbox boundary, (d) process outcome and retry safety, (e) packages/lock/target parity, (f) install identity and update, (g) language basics, (h) interoperability, (i) performance and code reduction. At each major boundary **trace input to effect and result**: caller → parser/decoder → checker/verifier → grant decision → host primitive → audit → output. Inspect and challenge every `return Ok`, `continue`, default/fallback, ignored error, `unwrap`/`expect`, `panic!`, `unsafe`, FFI, shared mutable state, attacker-supplied length, filesystem/network path and serialization limit in reachable production paths.

Use existing `cargo test`, locked builds, rustfmt, clippy, RustSec/cargo-audit, cargo-deny, unused-dependency tools, coverage, static analysis, fuzz/property tests, Miri, sanitizers and differential tests where available **and technically appropriate**. New tooling may be recommended rather than installed if expensive or unsupported. Search for suspicious patterns as *leads*, then inspect actual reachability and negative test. No `grep`-based count, branch coverage percentage or all-green test suite proves dead code or memory safety. Do not lower lints, delete conformance, disable security checks or increase permissiveness to get passing tests.

For every likely unused/duplicate/oversized feature provide: **path/symbol, role, active caller graph, linked feature/profile, dynamic-dispatch caveat, real user/test, removal risk, tests that must fail before deletion and compatibility handling.** Recommend deletion only when the relevant case is proven or made explicitly versioned. Identify independently if two internal layers do the same expensive checking; measure before merging them.

### 3.3 Make source coverage reportable rather than pretending to have read everything

Produce a **coverage ledger** for all production crates, preferably machine-generated to scratch with one row per significant module: `module/path`, `LOC`, `entry point/consumer`, `surface/trust boundary`, `source inspected?`, `positive executed?`, `negative executed?`, `CI-only?`, `not tested?`, `static/dynamic reachability confidence`, `known issue`, `risk`, `next test`. Summarize by *production source LOC actually inspected*, modules traced and critical execution paths; report counting method. **A low manual-inspection percentage is not a failure if the highest-risk boundaries are deeply tested; hiding it is a failure.** Do not call an inventory or an audit "full" when high-risk paths were only scanned.

Use independent or scoped sub-agents only when helpful and allowed. Subagents must operate read-only against the same pinned SHA, claim their own source slices and keep separate findings. A head chef must inspect the decisive paths and personally re-run any finding promoted to the final release verdict. Multiple subagents sharing a repo must not simultaneously change it.

## 4. Try to falsify Codex's **highest-risk new findings** first

The recovery plan identified **five immediate blockers**: untrusted Verified artifact handling, misleading execution outcomes, missing input/JSON, package/front-door/target inconsistency and install/deployment identity. Your highest priority is to test whether these observations are real **on the current exact build and exposure profile**, whether Codex overstated their exploitability, and whether any deeper flaw makes its proposed repair insufficient.

### T1 — Source checker vs DIR / Verified `.dpx` and `.dwx` artifact trust [P0]

- Reconstruct pipeline audit Appendix A.5 in scratch. Confirm a text program is rejected (`DL1604` or the current accurate code), a valid unmodified DIR is treated correctly, a deliberately colliding/duplicate node-ID DIR behaves as claimed, and one independent negative control is still refused. Follow the exact parser/AST → recheck/verifier → `.dpx` writer → `plugin verify` → runtime loader sequence.
- Reproduce with changed AST node IDs, synthetic nodes, duplicate IDs in different scopes, reordered nodes, malformed or oversized CBOR, invalid fields, version mismatches and generated/recomputed metadata, subject to safe bounds. Determine **which invariants live only in the lexer/source builder** and must be independently enforced at the untrusted serialized-artifact boundary. An artifact with a valid hash or signature is not proof that it is semantically valid; test a correctly signed malformed fixture with disposable keys when supported.
- Compare `dir::verify`, `plugin verify`, `plugin inspect`, `why`, `Authority`, `Atlas`, build, load and actual execution. Does one route validate less than another? Test whether `why` can endorse forged authority summary data despite a verifier rejection. Try bidi/U+202E, ANSI escape and malicious artifact names in **captured bytes**, not uncontrolled terminal output. Compare private helper and exported function counts. Record exact what reached runtime and any effect markers; don't claim a successful escape where only verification acceptance was observed.
- Exercise function effect row, reference-capability `val`/`iso`, mutable list/field writes, forged export signature and nested plugins under least grants. Confirm the historical `p.get` R-Get fix still rejects narrowed effects, with a permitted positive control. Test both parser/file source and trusted/Verified artifact form, including a check that the proposed unique-node guard cannot be defeated by alternate encodings or re-keyed maps.
- **Question Codex's P0.1 repair:** Is "enforce unique node IDs before replay" sufficient? What are the irreducible invariants (uniqueness, identifier validity, scope, row origin, binding identity, provenance, AST tree consistency, canonical facts)? At what *single* boundary must validation occur, and what other consumers need it? Do not implement a guard that merely compares two data structures already poisoned by the same colliding IDs.
- Severity is based on demonstrated reachability and impact. Distinguish an untrusted verifier bypass, analysis misinformation, actual ungranted effect, sandbox escape and cross-principal breach. Keep proof-of-concept artifacts private in scratch or coordinated disclosure channels; don't casually commit weaponized fixtures to a public branch.

### T2 — Truthful machine outcomes, crashes, partial effects and safe retry [P0/P1]

- Use a working bounded file transformation. Cause **a committed authorized write followed by `--report-out` failure**; capture effect sentinel, actual stdout/stderr, report file, exit code and structured JSON. Verify Codex's report of a usage-class error after success. Does the API distinguish `execution completed but report persistence failed` from `invalid invocation`? If not, a caller may repeat an already committed effect.
- Cause a **prestart sandbox or checker refusal**, with and without `--report-out`, and verify whether the promised report exists. Test `--preflight` before any side effect and separately after a first effect; do not infer transactionality from early refusal.
- Test closed stdout pipe, overly large output, malformed request, cancellation **before** first effect, **after** the first effect and during output, wall/fuel/memory exhaustion, graceful guest exit, guest crash/no-goodbye, host/broker crash, lost stream, audit sink failure, report sink symlink/race and parent cleanup using disposable processes and files. No process termination on a shared service; bound each run. Trace who owns terminal outcome and whether return code, JSON, on-disk report and actual effect agree.
- Demand explicit statuses for `success`, `application failure`, `security refusal`, `unsupported target`, `cancel requested`, `cancel confirmed`, `deadline/resource termination`, `partial side effects possible`, `outcome unknown` and `report persistence failure` where semantically distinguishable. Do **not** invent certainty after SIGKILL or network partition; document possible `UNKNOWN/INDETERMINATE` state and retry/idempotency requirements.
- Test `fix --dry-run` returning success with errors remaining, a missing explicit `test` path returning zero tests/exit zero, a malformed envelope schema showing `summary.errors:0`, and wrong/unknown CLI options accepted. Determine compatibility-preserving revisions, error codes and machine schema versioning rather than changing all contracts blindly.

### T3 — Ordinary input/data and **complete programs** [release gate]

- Build from fresh source a small **independently specified** task: program receives arguments or typed input, parses valid and invalid JSON, transforms at least one nested value, reads/writes a scoped file, returns a meaningful JSON result and an error, works on supported backends and leaves a sentinel untouched under denied authority. Use a **fixed external oracle** (Python/Rust/TypeScript) and specify expected outputs before testing. Try invalid UTF-8, escapes, number overflow, missing keys, unexpected types, very large inputs and filenames with spaces. Do not hide absent JSON by custom ad hoc `key=value` parsing.
- Test closures, generics, recursion, containers, strings, f-strings, ranges, math, bitwise, compound assignment, records, `Result`, user methods/tuples only to the extent currently claimed. The plan records `Root.args` and standard `parse_json` unavailable; verify whether these remain so. Treat missing fundamental APIs as real adoption constraints, but don't require every language feature to enter preview.
- Explicit source vs interpreter vs WASM parity or an early **typed unsupported** response *before any side effect*. Run a non-generic higher-order function, indexed assignment, actor/device use, optional features and one independently chosen edge case. Compare effects and diagnostics, not only stdout. Don't assume identical semantics for a host/device API that the WASM backend properly refuses.
- Test `run .` vs `run src/main.delulu`, and `run` vs `check` vs `test` vs `build` on identical packages with import graphs and declared ceiling. Test lock/dependency changes, path traversal, file-only vs package mode, offline resolution, package authority ceiling, duplicate/canonical path collisions and caches.
- Score task completion by correct output, correct refusals, accurate structured outcome, safe state, repair turns and setup burden. Green synthetic unit tests are supporting evidence only.

### T4 — `.solulu` source-extension migration [P1]

- Confirm whether `.solulu` is currently only accepted for **explicit loose files**, while package discovery (`src/main.solulu`) fails. Test mixed `.solulu`/`.delulu` imports **in both directions**, both backends, check/fmt/test/authority/build/edit/Atlas/LSP/MCP, unsaved editor overlays and locked local dependencies. Include case sensitivity and Unicode filename normalization by OS, duplicate module names and symlink aliases. A filename extension must never change grant ceilings or artifact signatures inadvertently.
- Identify one shared lowest-level source classifier and all actual consumers in `package.rs`, `deps.rs`, CLI, LSP, manifest, registry, scripts, examples, installer, VS Code `package.json`, activation selectors, textmate scope, code fences and release packaging. A grep sweep is a start, not a migration test. Decide which language IDs and wire formats should intentionally remain `delulu`/`.dpx`/`.dwx`.
- Recommend a **phased migration**: mixed-source end-to-end acceptance first; `.solulu` default scaffold and examples only after installers, editor, package and CI support it; `.delulu` compatibility preserved without a premature removal date. Reject mass renaming and documentation-only changes that hide a broken resolver.

### T5 — Independent principals, Guard, Authority, custody and adversarial delegation [previously unverified]

- **Mandatory attempt, not merely reading existing tests:** two independently authenticated principals/agents with **separate keys or credentials**, ideally separate OS users/containers/process security domains and isolated state. One holds a broader authority; it delegates narrower read/work authority to another; the delegate completes one allowed task and is refused a sibling path, forbidden write, wrong holder, excess scope, expired lease, replayed token and post-revocation use. Record the underlying OS identities and process boundaries, not just two synthetic node IDs inside one account.
- Run a safe end-to-end worker handoff: discovery/introspection → request → scoped approval (if necessary) → delegated execution → artifact/result → acknowledgement → second attempt after revocation. Check identity binding, attenuation, audience/recipient, expiration, nonces, chain integrity, authorizer vs actual executor, cancellation, audit provenance, idempotency and exact effects. Where feasible repeat across different machines or network namespaces with independent storage. No real credentials or uncontrolled network.
- Specifically challenge Codex's warning about **LEGACY/same-user custody**. Separate API principal IDs under one shared OS user are not proof of mutually distrustful agents. Test strict custody if supported, check OS-group, process and broker isolation, credential/lease exposure through argv, logs, inherited environment, process tables, filesystem and IPC. A refusal due to test harness limitation is not proof of security.
- Compare `Authority` static results (`within`, `exceeds`, `inconclusive`) with dynamic `Guard` checks under actual effects; test computed scope and safe runtime allowance. Never convert `inconclusive` into automatic approval. Probe symlink/junction traversal, path normalization and case sensitivity, file descriptor races, HTTP redirects, scope/method/path, DNS resolution and connection pinning, SSRF/loopback exceptions, token reuse and stale audit state with local controlled endpoints.
- Recheck Guard seals, owner/root issuance, nested plugin grants, signed driver identity, containment downgrade and signed vs verified artifact trust. Record where type/effect guarantee ends and OS/process containment begins. If separate credentials/OS accounts cannot be created on this machine, label test **NOT RUN** and provide a concrete GitHub runner/VM plan with identities, fixtures, oracle and output; do not relabel same-user as independent.

### T6 — Reconstruct and evaluate the **actual install/upgrade** path [previously unverified]

- Perform a clean, reproducible candidate install in an isolated prefix/home from **the target SHA**, preferably using a fresh clone/worktree, pinned toolchain and locked dependencies. Distinguish cached-offline build, true first-download, source install, release binary/archive install, PATH selection, editor activation, no Python feature, Python-enabled feature, WASM dependency and platform-specific isolation. Verify checksums/provenance/signature/SBOM claims where shipped. Report exact executable path/hash/version/schema/commit/host features.
- **Mandatory attempt:** install an existing released **v1.0.0 archive** (or nearest actually available predecessor) in disposable state; invoke its exact binary; then upgrade to the candidate using the **documented supported upgrade route**. Compare `delulu --version --json`, `doctor`, package manifest, `.delulu` compatibility, artifact reader, plugin signatures, cache/state migration, CLI output/status schema and rollback. If no valid released archive or supported updater exists, record `UNVERIFIED/UNSUPPORTED` and its precise cause; do not substitute rebuilding two checkouts and calling that a release-archive upgrade.
- Run one external novice/agent setup with no preconfigured checkout state. Can it discover the CLI, install, write a small `.solulu` or `.delulu` package, check/grant/run, observe a refusal, inspect version and understand fixes from only `README`, `INSTALL`, `--help` and `doctor`? Don't use undocumented maintainer recipes to claim first-time usability.
- Verify `.github` release workflow and runners, tar/zip layout, immutable checksum manifests, code signing/notarization, Windows SmartScreen/permissions, macOS quarantine, Linux executable permissions, dynamic library dependencies, user-data paths, stale PATH binaries, release URLs, minimum supported OS and whether the source/project URLs point to test or final repository. Do not publish, tag, promote or copy binaries to the public-release repo.
- **WSL is a separate hard test**: if available, build and execute the latest source inside WSL from a clean Linux-native path, not merely use a Windows `.exe` from `/mnt/c`. Test file/symlink semantics, case sensitivity, Linux sandbox availability, broker custody and network. Record WSL distro/kernel/glibc, target, linker, current source SHA and whether a Linux binary was actually built. Windows CI green is not WSL verification. If unavailable, provide a reproducible WSL2 workflow and label it not executed.
- Match Linux x86-64, Linux ARM64, macOS, Windows, WSL and sandbox classes to **named observed CI or local results**, not vague "cross-platform support." Surface actual missing/weak properties (e.g., a filesystem confinement absent on one platform or same-OS-user identity) and unsupported physical/real-time claims. Cross-target code generation is different from a runnable, secure binary on that target.

### T7 — Survey, Doctor, Atlas, MCP, LSP and machine interface consistency

- **Survey:** compare clean archive, untracked doc, generated maps, changed Rust import, actual dependency and one dynamic call; prove conservatism and blind spots. Measure generated JSON and graph size, time and stale false positives. Is Survey a maintainer map, not an end-user dependency? Should generated files be built at the time of release or maintained in git? Determine before recommending removal.
- **Doctor:** target selected installed executable vs active checkout; stale PATH, feature mismatch, wrong host/target, missing broker/containment, missing Python, bad permissions, empty working directory, L1 LEGACY and strict-custody readiness. `doctor --check` should not call "healthy" a state where core advertised operation fails. Propose layered `install / repository / selected workload / security posture` reporting without turning all optional components into mandatory failures.
- **Atlas/typed edit:** compare package-vs-loose analysis, snapshot digest, stale source/unsaved overlays, concurrent changes, code hash mismatch, authority-widening repair, rename/import graph, source spans, thread safety and export provenance. Measure useful outcome vs payload size.
- **MCP/LSP:** inspect actual supported protocol versions, JSON-RPC framing, tool catalog, schema, cancellation, task status, worker queue, unbounded `read_to_end`, malformed UTF-8, extremely large responses, tool input injection, output escaping, multi-client/long session, closed streams and inability to interrupt in-process work. For LSP, test cross-file package diagnostics, unsaved edits and editor binary-selection rules. Keep a known-good legacy client test while assessing current spec support. Prevent silent version fallback where it would change security or behavior.
- For every advertised command (`run`, `test`, `check`, `edit`, `fix`, `fmt`, `build`, `authority`, `guard`, `why`, `doctor`, `survey`, `atlas`, `mcp`, `lsp`, `schema`, `plugin`, `sign`, `publish`, etc. as actually present), generate a **front-door truth table**: accepts valid input, rejects invalid input, exit status, JSON schema, stable diagnostic, no silent no-op, target/platform restriction, authority outcome, consumer. Include unknown arguments and invalid isolation/broker profiles. Do not demand that specialist commands all be part of preview; mark optional accurately.

### T8 — Physical/embodied simulations and authority limits

- Re-run only safe simulations: an actuator with allowed range, command/readback, heartbeat/watchdog, expired/stale state, out-of-envelope refusal and emergency stop. Compare named simulator profile and null profile. The recovery plan observed a null adapter saying `COMMANDED` without sensor evidence; verify the semantic contract before marking success. Reproduce host loss during a simulated command, safe state, and recovery where practical. Distinguish queued/accepted commands from physical actuation or observed completion.
- Evaluate portability of DeluluLang as a *supervisory program* interacting with external ROS 2, DDS, existing robotics stacks, vehicle V2X and satellite/space protocols, rather than attempting to rebuild all of those in language keywords. Test only narrow mocks during this research program. No real vehicle, drone, actuator or production control signal.

## 5. Expand security and supply-chain review beyond Codex's subset

Build a concise threat model: assets, principals, trust boundaries, who can construct source/IR/package/plugin/network inputs, who signs what, where a verified fact is minted, what keys are held by each process, what can be replayed, what happens on crash, and which host privileges are shared. Follow the source-to-effect path and report exact assumptions. Examine at least:

- Type/effect laundering, `p.get` and alternate effects, generic inference, unsafe export casts, reference capabilities, unsigned/correctly signed malicious artifacts, node ID collisions, null/dummy capabilities, effects buried in closures or dependencies, noncanonical scope syntax and `else { continue }` enforcement branches.
- Untrusted CBOR/JSON/WASM decode limits, integer overflows, path normalization and canonicalization, symlink races, TLS/SSRF/rebinding, dynamic egress redirect/scope, directory/registry traversal, plugin loader mismatch, cache poisoning and package lock pinning.
- Grant/Guard downgrade, capability forgery, bearer token exposure, expiry/replay/cross-tenant, auditing before vs after effect, audit lock blocking, partial effect, atomicity, crash consistency, improper trust in agent-provided metadata or LSP/MCP outputs.
- `unsafe`, FFI, Python extensions, external worker and shared-memory processes; resource limits including recursion, deep nesting, oversized output, actors/mailboxes, unbounded fan-out, cancellation that leaves a subprocess alive, DoS against broker or MCP service. Use Miri/fuzz/sanitizers when applicable; don't infer absence of bugs when a tool cannot exercise the relevant platform.
- Cargo dependency and license graph, known RustSec/advisories against *locked versions*, crate signing/provenance, reproducible artifacts, third-party notices, microVM GPL obligations, release keys, CI action permissions/pins, branch protection and dependency lock tamper tests. An apparently safe source tree can ship an old binary or a non-reproducible insecure archive.

Classify `confirmed vulnerability`, `plausible issue with a reachable path`, `assurance gap`, `historical fixed`, `design limitation` and `not tested` separately. Severity must reflect **demonstrated impact and accessible attacker conditions**, not scary words or pass/fail totals. If you find a serious unreported exploitable bug, keep sensitive details in the private scratch and recommend coordinated disclosure under `SECURITY.md`; do not publish an exploit recipe to the public remote in the name of open research.

## 6. Current public interoperability research: use standards, not speculation

**You must conduct new primary-source research**, check publication dates and versions against the session date, and distinguish protocol draft, published spec, reference SDK, real vendor interoperability and adoption. Do not invent a new universal "AI communication" standard because of the owner's initial premise that none existed.

At minimum inspect:

1. **A2A (Agent2Agent)**, from `https://a2a-protocol.org/v1.0.1/specification/`, `https://github.com/a2aproject/A2A/releases`. On 14 November 2026 the public release listing included **v1.0.1 (26 May 2026)**. Research Agent Cards, supported interfaces and versions, task/messages vs artifacts, authenticated task visibility, streaming/poll/push, cancellation, tenant identity, retries, idempotency, error taxonomy, schema and how *authority grants* would travel **outside** rather than be confused with A2A identity. **Do not assume an A2A task is a safe delegated DeluluLang capability.**
2. **MCP (Model Context Protocol)**, official `https://modelcontextprotocol.io/specification`, especially the **2026-07-28** published revision: stateless request handling, discovery/version selection, tasks extension, authorization changes, caching, transport and deprecated mechanisms. Newer MCP Tasks are an **extension** rather than the 2025 experimental core. Check actual wire behavior of `delulu mcp` and distinguish its older negotiated versions from conformance with 2026-07-28. A clean legacy initialize response is not proof of current-spec compatibility.
3. **Related agent/UI/API technologies** only where needed: IBM ACP and any migration/relationship to A2A; AG-UI; OpenAPI/JSON Schema, AsyncAPI, gRPC/Protobuf, JSON-RPC, WebSockets, OTLP/OpenTelemetry, W3C trace context and OAuth/OIDC/mTLS. Specify whether a choice supplies tool access, agent task exchange, job status, authenticated custody, telemetry, user interface, or transport. Different layers are complementary, not mutually exclusive.
4. **Robots and vehicles:** ROS 2/DDS and DDS-Security, SROS 2 security enclaves, topics/services/actions/QoS; ETSI C-ITS/V2X **CAM, DENM, CPM** and relevant EU C-ITS interoperability work; UNECE WP.29 vehicle security/software-update rules R155/R156; ISO 26262/SOTIF/ISO-SAE 21434 when relevant. Verify official and current sources. Do not equate these domain standards with A2A/MCP or claim that language authority alone makes a real-time or safety-certified system.
5. **Existing isolation/agents and engines:** NVIDIA OpenShell (public releases showed **0.1.8 on 10 November 2026**, but recheck version and changelog); Wasmtime/WASI, Windows AppContainer/Job Objects, Linux namespaces/seccomp, macOS sandbox and optionally gVisor/Firecracker. Evaluate what they enforce *versus* static DeluluLang effects/Guard; do not assume external containment makes all internal boundaries obsolete.

Produce a **layered comparison**, not a marketing leaderboard: authoring IR/language → program validation → capability/policy → execution/isolation → agent discovery/delegation → job lifecycle/artifacts → domain-specific transport/safety. Recommend **adapters before new built-in protocols**. If proposing a small A2A bridge, give a test with two independently authenticated clients, separate allowed scopes, version negotiation, streaming/poll/cancel, replay, duplicate request key, artifact authenticity and unknown partial result; estimate maintenance surface. Recommend against implementing it until ordinary program execution and verified artifacts are trustworthy unless evidence demonstrates an urgent bounded use.

Every version-specific or regulatory statement in your report needs a dated primary link. Clearly separate **source-derived data**, **your measured experiment**, **code inference**, **recommendation**, and **unknown**. Do not imply legal advice or regulatory approval.

## 7. Does the architecture earn its ~200,000 Rust lines? Quantify alternatives and safe removals

Research whether DeluluLang should remain **a language + CLI-first agent-oriented toolchain**, rather than separate branded compiler/VS Code products. Keep the parser/checker/code generation internally if they support soundness. Compare:

- Text source + shared typed checker + interpreter/WASM vs direct agent-authored typed IR or bytecode, optional authenticated precompiled cache, or native AOT. Reject bypass of rechecking or silent backend fallback. Benchmark a *useful multi-file task*, not just parser microseconds.
- LSP/MCP/CLI on a single typed analysis/execution service vs repeated package re-resolution, process launches, ad hoc JSON, duplicated diagnostics. Preserve transport-specific framing and security boundaries; "shared service" must not mean all agents gain the same ambient credentials.
- Survey/Doctor/Atlas as distinct genuinely useful developer tools vs overlapping graph/health logic; isolate where merging reduces work and where separate failure domains are important.
- Embedded Python, external worker, networking, plugin, registry, microVM, federation, complex editor integration and physical/device adapters: KEEP/REPAIR/MAKE OPTIONAL/EXTERNALIZE/DEFER/DEPRECATE/REMOVE based on linked size, task completion, dependency vulnerabilities, startup overhead, behavioral coverage and actual users. Do not delete due to dislike or LOC alone.
- Security-critical but complex Rust modules vs a decomposition into smaller audited modules. Prefer incremental refactors with tests and rollback over wholesale rewrites. Analyze whether very large `cli.rs`, WASM `codegen.rs`, `brokerd.rs`, `check.rs`, parser and host modules are **high-risk responsibility concentrations** that should be split and why; LOC alone is not a bug.

Measure warm and fresh process latency, p50/p95 with recorded samples, peak total process-tree memory (not only host PID), CPU, binary sizes (debug and candidate release profile), startup and check/run/authority/MCP call sequence, package size, bytes/tokens returned, number of subprocesses, repairs and **end-to-end task success**. Codex's old ~70-kB `toolchain` response and ~3.73-MB Survey JSON are **investigation leads**, not proof that simplification improves outcomes. Run like-for-like before/after prototypes only in scratch, with a fixed independent oracle. Measure same isolation, toolchain and feature flags for comparisons.

Test at least one 20-task or other statistically useful paired agent workflow **if feasible**. Compare DeluluLang with a conventional Python/TypeScript/Rust workflow under equivalent sandbox and authorization policy, with identical task inputs and budgets. Score actual correct completions, unsafe attempted effects blocked, grant width, error/repair loops, tool-call counts, response bytes/tokens, runtime and setup cost. If a multi-agent/model experiment is not available, document its pre-registration, fixture set, fairness conditions and **NOT RUN** status. Do not assert "AI-first is faster" by fiat.

## 8. Governance, contribution freedom, license and documentation reduction

The owner's policy decision is **any person or AI system may suggest changes, report issues, open discussions and submit proposals without sponsorship**. Submission is not permission to merge, publish, change licenses or access secrets. Read the current `CONTRIBUTING.md`, `GOVERNANCE.md`, RFC rules and `AGENTS.md` before describing the exact existing sponsor rule: earlier uploads already welcomed AI contributors but required named nonauthor sponsors for some risky merges. Do not misstate that as a ban on AI suggestions. Independently evaluate replacing *pre-submission sponsorship* with **risk-based independent merge review**, accounting for a one-maintainer project, AI-assisted patches, qualified independent reviewers and the difference between two agents under one controller vs genuinely independent review. Recommend safe low-risk handling and a hard hold for security-critical changes without credible independent review.

Review Apache-2.0 `LICENSE`, `NOTICE`, contributor patent terms, trademark distinction, `TRADEMARK.md`, third-party/GPL obligations, branch protection, release authority, security-reporting route, conduct enforcement and public testing repo vs future release repo. Recommend changes to **any** entrenched or governance rule where justified, but identify legal uncertainty and required authorization; do not silently relicense or claim global exclusive rights to common terms. Keep or change Apache-2.0 only for a demonstrated need and a lawful migration. The language may retain a playful name and serious technical quality simultaneously.

Diagnose the **documentation loop** as an engineering problem. Make one canonical ownership map for: live implemented behavior and generated reference; compatibility contracts; open work/RW identifiers; release gates; old V2 decisions; historical research; assistant/routine handoff. Classify docs as normative, generated, current operator help, current queue, historical, stale or speculative. Flag duplicated status claims in `README`, `HANDOFF`, `REMAINING_WORK`, `V2`, `CLOUD_SYNC_LOG` and reviews. Preserve evidence and link integrity; recommend archival/redirects rather than destructive deletion. No new parallel master roadmap. A "done" flag is not proof of source and runtime; a generated map is only current if its freshness check is meaningful.

## 9. Challenge Codex's milestone order and define a real release bar

Codex proposed:

- **M1 / P0.1:** repair Verified artifact ingestion (DIR node-ID / source invariant mismatch).
- **M2 / P1.1:** repair post-effect report sink and prestart sandbox terminal outcomes.
- **M3 / P1.2:** make mixed `.solulu` / `.delulu` packages consistent across front doors.
- Then test/fix/schema exit semantics; basic args+JSON; clean install/upgrade; bounded preview; later A2A and physical adapters.

**Try to break this order.** Determine whether a forged artifact is actually reachable through accepted user/plugin loading and whether a narrower vulnerability preempts M1. Determine whether false-success test selection, broken-pipe outcome or missing `Root.args`/JSON must come before `.solulu` for a real third-party test. Identify any atomicity or crash-state issue making M2 too small. Confirm whether a single shared package resolver is adequate for `.solulu` without altering manifest or lock identity. Test whether repair of a narrower code path is blocked by stale branch work, e.g. RW 2.50 source on `claude/relaxed-brown-2l78i1` with four reported generated-reference/coverage failures; do not merge or discard it just because a historical log says so.

For each milestone offer **one executable failing witness**, a distinct negative control, the narrowest plausible source boundary, a property or adversarial test that defeats a superficial fix, acceptance across affected front doors, rollback/migration plan, target CI jobs and a decisive completion definition. Prefer minimal code slices over adding another subsystem.

### Candidate preview requirements — derive actual release gates

Do not claim "ready" solely from green CI. A public **testing candidate** should at least show: (i) signed/Verified artifact input refuses invalid semantics in each reachable consumer, (ii) correct machine-readable terminal/unknown state and no blind retry after possible effect, (iii) ordinary inputs/JSON plus a meaningful file transform, (iv) supported package/extension/target matrix, (v) disciplined least-authority behavior, (vi) a clean independent install from a real distribution with clear binary identity and documented limitations, (vii) honest security support channel and supply-chain/license artifacts, (viii) versioned compatibility/deprecation, (ix) platform-specific refusal instead of silent downgrade. Be precise about what can wait, including robotics, A2A, full WASM parity, richer editor features and native code generation.

Separate **internal development**, **private or local preview**, **public testing repository**, **signed public release** and **production/security-critical use**. There is no automatic transfer of proof from one to another. Do not tag/publish/merge or alter repo visibility under this research commission.

## 10. Execution methodology, nonvacuous evidence and research completeness

For every claim you validate:

1. Pin source SHA, tested binary SHA-256, host OS/arch, features, containment/broker profile, actual process identity and state isolation.
2. Write a hypothesis with an expected **positive** and **negative** oracle before running; include a third nonvacuous control for security gates (prove the gate would fail if the enforcement were removed where appropriate).
3. Use a scratch fixture with bounded inputs, explicit args/working directory/environment, stdout/stderr/exit, timestamp, elapsed time, report path/bytes, resulting files/audit markers. Scrub secrets and terminal control characters.
4. Show whether the test actually traversed the named production boundary (not only a mock, helper, `.dwx` wrapper or a manifest checker). A `PASS` on `plugin verify` is not a `PASS` on runtime loader and vice versa.
5. Re-run on current HEAD and compare to Codex's audited SHA when possible; don't combine outcomes across SHA or platform into one status. A CI matrix alone is not a current local reproduction.
6. Status must be one of: `PASS`, `FAIL`, `PARTIAL`, `UNSUPPORTED`, `NOT RUN`, `BLOCKED`, or `INDETERMINATE` for task runs; for claims use `CONFIRMED`, `FALSIFIED`, `NARROWED`, `REGRESSED`, or `UNVERIFIED`. Do not transform failure to perform a test into success.
7. Include minimum reproducible code/commands, independent oracle and what finding would overturn your conclusion. Report all corrected harness setup errors separately from true product failures; avoid a "fix" that only changes the test.

For the previous targeted suites, locate exact named tests and rerun where available before claiming coverage. Codex reported **203 passed and one LSP test ignored** from four `cargo test --locked -p delulu` groups (`plugin_load_cli`, `mcp_cli`, `lsp_cli`, `edit_cli`, `ranges_cli`, `strings_cli`, `bitwise_cli`, `compound_assign_cli`, `wasm_cli`, `sandbox_run_cli`, `egress_cli`, `package_run`, `grants_cli`, `guard_e2e`, `plugin_cli`, `signing_cli`). Expand beyond these into **every high-risk production crate**, `delulu-check` DIR tests, `delulu-broker` custody, `delulu-runtime`, WASM verifier, actual released binary, negative multi-agent and source paths. Use locked workspace tests or exact-SHA CI only as complementary broad regression coverage.

Test on GitHub Actions runners if local conditions prevent necessary tests **where current repository permissions and workflow safety permit**. Never dispatch costly or privileged jobs to an unknown repository/branch or bypass workflow requirements; if dispatch requires additional authorization, prepare the exact workflow/action and mark it not run. The developer's "run on GitHub Runners if laptop cannot" preference does **not** make an unavailable or skipped test count as done. Avoid unlimited fuzz loops, real hardware or external targets, production creds and destructive CI.

Prioritize by risk and information gain. If a confirmed critical security bypass is found, stop ancillary benchmarks and report the narrow effect, affected profile and private proof first. Complete a first pass over **all** required areas and a deep pass over the most consequential security and release gaps. If resources are limited, state which obligations remain and why; do not truncate the report into unjustified assurance claims.

## 11. Deliverable: **one rolling** independent audit, final verification verdict and a first-implementation handoff

**Multi-session output:** Maintain **one rolling independent audit and evidence-backed final-verdict document** plus a minimal resume state, as authorized in the charter. At the beginning it is explicitly **IN PROGRESS**, not a final report. Update it throughout sessions without creating new duplicate strategic plans. Place raw command logs, measurements and bounded fixtures in safe scratch/private storage with a hash/pointer index; do not publish sensitive exploit artifacts. You may commit/push *only* the authorized research/handoff artifacts to a non-default branch of the testing remote, subject to repo policy. Keep all executable code, normative docs, CI and main branches unchanged under this research commission. In each CLI final response give what you actually completed, exact persisted paths/commits, next evidence objective, and current blockers. If no persistent destination is available, export the report and resumption state to the owner and mark continuity at risk.

The rolling/final report must contain, in this order (intermediate entries may be `IN PROGRESS` until verified):

1. **Executive verdict (maximum one page):** Do you agree with Codex's CLI-first decision, five blockers, security severity, first three milestones, `.solulu` plan, and preview criteria? Where is Codex mistaken, incomplete, too cautious or too optimistic? What is the **very next code change**, and should it be security, execution truth, basics or packaging?
2. **Ground-truth snapshot:** model/effort as actually observable, SHA, branch, dirty state, binary identity, builds, all accessible CI/jobs, Survey and Doctor clean-vs-dirty control, testing environments and isolation/credential identities.
3. **Coverage-accounting matrix:** all production Rust crates and their important modules with source-inspection and test coverage, reachability, high-risk boundary, code/runtime/test gaps, and **an explicit list of unexamined production paths**. Summarize counts honestly. This specifically answers the owner's worry that code research was partial.
4. **Codex hypothesis challenge table:** at least the T1–T8 tracks above plus any newly discovered high-priority risks. Each result includes new direct evidence, positive/negative control, minimal commands, status, severity, source location and any contradictory Codex statement. Identify problems fixed after the November 4–5 reports and problems that only existed in docs.
5. **Complete-program and AI-to-AI task scorecard:** a small task with known oracle and least-grant refusal; two independent principals if possible; accurate result and retry behavior. Distinguish real end-to-end execution, mock, CI-only, unsupported and not-run.
6. **Real install/release/WSL/platform matrix:** source clone, released archive upgrade, actual WSL Linux build, Windows/macOS/Linux/ARM where supported, isolated execution and missing properties, binary provenance and cross-platform unknowns.
7. **Security and supply-chain findings** with threat boundary, realistic severity, reproduction without exposing secrets, root cause and missing tests. Do not overclaim formal verification, external sandbox guarantees or certification.
8. **Independent external research** with dated primary URLs for A2A, MCP, relevant agent runtimes, ROS 2/DDS, ETSI/EU V2X and UNECE, and a recommended minimal adapter path if warranted. State why a new protocol or broader robotics implementation should/shouldn't be deferred.
9. **Keep / repair / consolidate / make optional / externalize / defer / remove** table backed by code reachability, actual user and measurement. Include license, sponsorship, review policy, security and documentation suggestions, with migration/rollback and owner-authorized steps distinct.
10. **One revised execution queue and preview gates:** at most **five** next code slices, priority/severity, precise file/symbol, dependencies, independent negative oracle, acceptance suite, platforms and expected user benefit. Reuse existing RW and V2 identifiers; do not create a parallel work system. State whether the current recovery plan needs correction and the precise sections to update later.
11. **Verbatim-style, copy-pasteable next coding-agent handoff** with the exact first executable command(s), first failing witness, constrained fix scope, tests proving red/green, security control and rollback, affected CI jobs, review requirement and explicit no-publish/no-merge boundary unless separately authorized. The next agent should **write code**, not issue another request for open-ended strategy.
12. **What you did not verify:** every required but inaccessible experiment, especially separate-credential two-agent, release archive upgrade, native WSL build, platform-specific security and full Rust source coverage. Give runnable steps/environment and do not label these done.

### Stop conditions for the next coding milestone

- **Verified artifacts:** source and every untrusted artifact consumer agree on syntax/identity/effect/reference-capability invariants; valid controls still work; malicious/correctly signed malformed artifacts are rejected before an ungranted effect. Tests fail if the new gate is removed or moved after fact replay.
- **Outcome contract:** after a real side effect and a failed report sink, the status cannot be mistaken for pure usage error or safe blind retry; prestart refusals produce the advertised machine outcome; crash/partition uncertainty remains explicit.
- **Mixed-source package:** `.solulu` and `.delulu` both work from the package and explicit front doors on supported engines, preserve imports and authority ceiling, reject name/path duplicates and pass editor/package/lock tests; only then change scaffold default.

If the evidence establishes a different first blocker, **explain the decisive test and substitute it**. A practical, modest patch with an adverse witness and rollback is better than an impressive unpublished design.

## 12. Nonnegotiable finish-line rules

Do not say **"all code audited," "production ready," "secure," "formally proven," "100% compatible," "portable," "A2A/MCP compliant," "certified," "two-agent isolated," "release tested" or "all tests passed"** without matching scoped, dated evidence. Do not hide a failed or skipped test, weaken a check, infer an unobserved host boundary, fabricate GitHub results, invent legal approval, contact addresses or live credentials, or dismiss Codex's findings just to show independence.

**The correct result is one truthfully scoped decision and one implementable next slice that shortens the path to a reliable, widely testable DeluluLang.** Research should end the loop, not become another phase of it.

---

## 13. Structured multi-session campaign — establish coverage, then systematically close it

Use the following **workstreams across many sessions**. The ordering is a default, not a claim that every subsystem requires sequential completion; escalate a proven critical bug immediately. Do not merely create one document per workstream. Each row must connect a specific code path and threat to an executable test, report status, applicable platforms, independent oracle and a release decision.

| Workstream | What must be deeply understood and rechecked | Minimum exit evidence for its critical paths |
| --- | --- | --- |
| **A. Ground-truth baseline** | All crates/modules/features/test binaries, remotes, dirty data, CI and build variants, current version/release artifacts, real dependency graph, documentation sources of truth | Reproducible inventory at SHA, Survey/Doctor clean and dirty controls, current binary identity, stale/historical distinction |
| **B. Language semantics** | Lexer/parser, name resolution, generics, inference, typed IR/DIR, type/effect/reference-capability soundness, module/package semantics, diagnostic/fix contracts | Independent valid/invalid oracle corpus, metamorphic and mutation tests, source vs artifact consistency, proven negative controls |
| **C. Engines and execution** | Interpreter/WASM parity, host primitives, preflight/actual authority, effect custody, fuel/memory/wall budgets, crashes, cancellation, result schemas, cache and serialization | Same-program differential tests, least-privilege side-effect sentinels, fault injection and explicit unsupported-target contracts |
| **D. Trust and adversarial inputs** | Verified artifacts, plugins, Guard/broker, leases, secrets, scope normalization, untrusted JSON/CBOR/HTTP/MCP, dependency signatures, threat boundaries | Attack-path reproductions and mitigations validated across all reachable consumers, controls that fail if a gate is removed |
| **E. Developer/agent surfaces** | CLI, LSP, MCP, Atlas, Survey, Doctor, error repair, package mode, `.solulu`, stale binaries, task cancellation and automated agents | Fresh no-insider install-to-result path, machine contract matrix, output/error coherence, multi-agent isolation if environment permits |
| **F. Platform and distribution** | Windows native and AppContainer, WSL, Linux x64/ARM64, macOS, toolchains, archive upgrade, GitHub runners, release signatures, SBOM/provenance, recoverability | Per-platform/current-SHA results with independent archive and path checks; missing platform explicitly `NOT RUN` |
| **G. Assurance and sustainability** | Test gaps, fuzz/property/formal scope, unsafe/FFI, availability/DoS, panic handling, observability, on-call response assumptions, maintainer/CI costs | Confidence statement tied to actual test inputs, bounded failure modes, maintainer-sustainable release obligations |
| **H. Competitive relevance and adoption** | Concrete advantages/limitations versus established languages and agent runtimes; interop, install friction, documentation, performance, stability and governance | Reproducible side-by-side real tasks and scorecard, identified early adopters, falsifiable value proposition, no fabricated market adoption |

**First pass / deep pass / recheck pass** applies to every workstream: (1) map the surface and evidence, (2) trace the top-risk paths and execute positive/negative tests, (3) revisit important conclusions after contrary evidence and on newer code. Prioritize by demonstrated harm, reachability, feature centrality, adoption friction and cost of missing a defect. A spectacular edge case does not excuse skipped basic user programs.

### Cross-session evidence registry and invalidation

For every significant proposition, keep one stable entry (reuse existing `RW`/V2 identifiers when they genuinely match). Recommended fields:

```yaml
id: FABLE-001
claim: "Example testable behavior; do not reuse this placeholder"
category: source-soundness | execution | security | usability | distribution | interop
source_sha: <exact git commit>
binary_sha256: <sha256-or-NOT-RUN>
platform_and_profile: <os/arch/toolchain/features/custody/target>
source_paths: []
entrypoints_and_consumers: []
last_verified_at: <timestamp>
evidence_level: local-executed | independent-reproduction | ci-only | source-traced | historical | inference | blocked
positive_control: <command-and-exact-observed-result>
negative_control: <command-and-exact-observed-result>
independent_oracle: <reference, expected bytes or property>
raw_evidence_location: <approved path and sha256, or UNAVAILABLE>
verdict: CONFIRMED | FALSIFIED | NARROWED | REGRESSED | PARTIAL | UNVERIFIED
security_visibility: public-sanitized | private-confidential
impact_and_release_gate: <scope and priority>
recheck_trigger: <changed modules/feature versions/platform/dependency>
next_action: <one executable test or exact code task>
```

This is a **template**, not a claim the fields already exist. Keep it machine-readable without maintaining a second contradictory truth ledger. Changes to dependent modules, compiler schemas, host adapters, verification paths, Cargo.lock, build profile, distribution workflow or protocol version should trigger **targeted regression and claim invalidation**. Use Survey impact to help find affected entries, then verify Survey's own blind spots. A passing check on a new SHA does not silently transfer all old findings' statuses.

### Independent review and adversarial verification

A claim that could allow untrusted code to gain authority, falsify evidence or prompt duplicate effects requires more than repeating an existing test. Use **at least two genuinely different lines of evidence** where feasible: e.g. source-level invariant review plus black-box runtime observation; Linux and Windows independent implementation paths; independent oracle plus differential engines; model checking assumptions plus adversarial counterexample. If available, have another reviewer/agent independently design a new test without seeing the desired conclusion. **A subagent instructed by the same head chef and sharing the same privileges is not an independent security authority or governance approver.** Record when independent review cannot be achieved. For serious disclosures use private channels and restrict reproduction detail.

### Multi-session checkpoints and state machine

Maintain a small state file with: `program_status` (`IN_PROGRESS`, `BLOCKED`, `READY_FOR_IMPLEMENTATION_HANDOFF`, `VERIFICATION_COMPLETE_WITH_LIMITATIONS`), current SHA(s), completed workstream slices, remaining highest-risk unknowns, verified-vs-proposed milestones, active test processes/cleanup status, safely stored evidence IDs, current branch and whether the last branch push succeeded, and **the next three literal commands** with expected observations. Do not change `program_status` to "COMPLETE" merely because a session concluded. The next session should be able to choose the next high-information-value action without reading hundreds of pages of old discussion.

If the owner elects to authorize **implementation** separately, do not terminate all research; transition to an iterative loop of **verified failing test → minimal implementation → new positive/negative verification → affected-platform CI → independent review → documented compatibility → next finding**. Implement **one releasable vertical slice at a time**, not a speculative architectural rewrite. Research-only and implementation permissions must remain distinguishable in git history and handoff.

## 14. Compete on real value, not aspiration: enterprise-grade adoption and sustainability

The owner's long-term ambition is for DeluluLang to be adopted by organizations and autonomous systems alongside well-established languages and platforms. Investigate that ambition as a **falsifiable engineering and ecosystem problem**, not a marketing claim. Companies do not migrate because a language says "future"; they need compelling advantage with controlled adoption costs and credible maintenance.

Study direct and adjacent alternatives with **comparable workloads**: Rust, Go, Python, TypeScript/Node.js, Java, Swift/Zig where relevant; Wasmtime/WASI/WebAssembly components; established orchestration and job-execution services; and agent tools/frameworks using MCP/A2A. Distinguish competitors from compatible integration layers. Re-evaluate the research list as the ecosystem changes. Do not state adoption, benchmarks, CVE status or release versions without current evidence.

Evaluate DeluluLang's potential **differentiation** at a narrowly measurable level: a coding agent can receive a machine-verifiable statement of effects, request constrained capabilities, execute a bounded task, receive honest terminal evidence and delegate work without obtaining ambient host credentials. Ask if an existing combination of Rust/Go/Python + sandbox + SDK already performs the same task more simply, safely or quickly. If it does, say so, and recommend interoperable reuse rather than duplicating functionality. If DeluluLang does better, show **reproducible comparisons at the same task and trust boundary**, not a cherry-picked syntax demo.

Measure adoption friction for four early user groups: (a) independent AI coding agents, (b) individual programmers learning the language, (c) enterprise security/platform teams hosting untrusted agents, and (d) researchers building simulated autonomous/physical agents. For each, test discovery, self-service installation, IDE optionality, docs correctness, examples, first useful program, error repair, CI integration, package dependency/release upgrades and operational troubleshooting. A reviewer unfamiliar with the repo should complete the task without private owner knowledge. Quantify setup commands, median wall-clock, repair attempts, failed steps, stable API omissions and maintenance burden where credible.

Research **real-world language stewardship**: compatibility/versioning and deprecation, language specification and reference conformance, compiler/backend matrix, public issue triage, responsible disclosure, response capacity and honest security support SLAs, dependency licensing and patents, community governance without sponsorship barriers to *proposals*, risk-based independent merges, release-key and artifact provenance custody, signed archives, SBOM, reproducible build claims, platform lifecycle policy, stable diagnostics and package ecosystem trust. Design these to be operable by the actual maintainer resources. A policy without reviewers or capacity is not an assurance mechanism.

Use official reference frameworks to guide practical controls, **not to claim certification or compliance**: NIST SSDF SP 800-218 (final 1.1, verify later revisions and drafts); SLSA build provenance and applicable build-track requirements; RustSec and advisory databases; OpenSSF Scorecard where useful; and published relevant security/interop specifications. Determine which mechanisms actually reduce DeluluLang's supply-chain or maintenance risk and which would be ceremonial checkboxes. Sources: https://csrc.nist.gov/pubs/sp/800/218/final , https://csrc.nist.gov/projects/ssdf/publications , https://slsa.dev/spec/ , https://scorecard.dev/ , https://rustsec.org/ .

**Market/release direction recommendation must be evidence-based:** choose a narrow first adopter and use case, a supported-platform and threat-model statement, a meaningful task corpus, an honest preview support policy, and a compact onboarding path. Separate **research-grade**, **third-party preview**, **general-purpose tool**, **enterprise deployment**, and **safety-critical physical control**. The last two require distinct assurance that cannot be inferred from a small research project or from language-level Authority and Guard. Build a community others can contribute to **without an upfront human sponsor merely to propose a change**, but do not waive merge/security review or real operational accountability.

Do not add futuristic features simply because a large company might someday ask. Provide a **keep/defer/drop** verdict for each major idea and the concrete user/benchmark that would justify revisiting it. Design for extensibility, not imaginary requirements of future AGI/ASI or a guarantee that "everyone" will adopt the language.

## 15. End-of-session handoff and eventual termination criteria

### After **every** authorized session

Report, compactly and truthfully: actual model/effort observed; environment; source SHA and exact binary; tests and source modules inspected; raw evidence pointers; confirmed/falsified/new findings; security-sensitive items withheld from public logs; changes (if any) limited to audit docs; branch/commit/push status; which results need re-verification after newer commits; and the **single highest-value next experiment** plus up to two fallback tests. Do not say "everything tested," "100% secure," or "ready" without scope. Include an **explicit resumption instruction** the owner can paste into the next Fable 5.2 CLI session. If the session stops before persistence, clearly explain how to recover its state rather than fabricating successful save/commit/push.

### Before calling the research program complete

- The crate/module inventory is complete **as an inventory**, and the risk-weighted source and runtime coverage is reported with exact uninspected/high-risk paths. Every high-criticality trust boundary has at least a clear traced implementation, negative test and a reviewer challenge, or a clearly marked unresolved blocker.
- All Codex/high-priority historical findings have **latest-SHA statuses**, with reproducible evidence or concrete `BLOCKED` reasons. Claims tied to old commits are not silently declared fixed/open on HEAD. The checked finding list is **not assumed exhaustive**.
- Every release-critical **end-to-end workflow** has independent oracle(s), denial cases, outcome semantics, a tested correct installation path and an honest supported-platform matrix. Missing separate OS-user identities, WSL, release upgrade, macOS, ARM or privileged isolation are named **unverified** until actually tested.
- The threat model and material supply-chain risks are documented, with no false implication of formal whole-system proof. Sensitive findings remain private and responsible-disclosure-ready.
- Competitive research gives a defensible first use case and measured interoperability/adoption trade-offs, not a claim that DeluluLang already surpasses mature languages.
- There is **one ordered implementation/release queue**, updated from the existing recovery plan, each task with independent failing witness, minimal source boundary, test oracles, review needs and objective completion gate; speculative directions are explicitly deferred. Further study should not block the first safe implementation without an articulated dependency.
- The actual owner can determine at a glance whether the next action is **(1)** another research verification session, **(2)** a separately authorized code milestone, **(3)** obtaining an unavailable environment/independent reviewer, or **(4)** a narrowly scoped preview decision.

A research audit cannot establish absence of all bugs, future adoption or security in every deployment. **"Verification complete with limitations" is an acceptable rigorous result** if residual uncertainties are explicit and release-relevant blockers remain closed; open-ended research for its own sake is not. The project only becomes releasable after independently passing actual release gates in a **separate authorized build/release process**.

### First-session execution instruction

Do **not** start by rewriting a strategy report. Reconcile current HEAD and existing audit status, create a resumable audit state in an approved location, run Survey/Doctor with clean controls, pin an executable, then independently reproduce the highest-impact untrusted Verified artifact finding (or test the current fix if already landed). Trace every consuming path and capture an independent negative oracle; record gaps and start the next investigation. When the session reaches a clean checkpoint, persist evidence/handoff, state the next exact command and stop or take the next bounded evidence slice according to the active session type.

## Primary official external sources to re-verify (starting points, not proof of implementation)

- DeluluLang development/testing repository: <https://github.com/jessesuniljs1-collab/delululang-test>
- DeluluLang dated CI example on baseline: <https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/43409171490>
- A2A release history: <https://github.com/a2aproject/A2A/releases>
- A2A 1.0.1 specification: <https://a2a-protocol.org/v1.0.1/specification/>
- MCP published specification: <https://modelcontextprotocol.io/specification>
- MCP 2026-07-28 release notes: <https://blog.modelcontextprotocol.io/posts/2026-07-28/>
- ROS 2 security enclaves: <https://design.ros2.org/articles/ros2_security_enclaves.html>
- ROS 2 DDS Security integration: <https://design.ros2.org/articles/ros2_dds_security>
- ETSI standards portal: <https://www.etsi.org/standards>
- UNECE WP.29 reference documents: <https://unece.org/transport/road-transport/reference-documents>
- NVIDIA OpenShell releases: <https://github.com/NVIDIA/OpenShell/releases>
- Apache-2.0 text: <https://www.apache.org/licenses/LICENSE-2.0>

**Remember:** An official standard is evidence about that standard; it is not evidence that DeluluLang implements it or that two agents have successfully interoperated.
