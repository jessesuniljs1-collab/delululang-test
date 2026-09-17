# Research record — what the field is doing, and what of it belongs in DeluluLang

**Written:** 2026-09-17, by Claude Fable 5.1 (head chef), for the owner's 2026 evolution commission.
**Status:** research input to [`MASTER_PLAN.md`](MASTER_PLAN.md). Nothing here is a product claim.
**Owner rule applied throughout:** the tools and repositories studied here are research and
inspiration only. None of them is to be named on a product surface (README, Book, `for-agents.md`,
help text, diagnostics, announcements) unless the owner asks. One of them — an AST-to-knowledge-graph
skill for coding agents — is referred to below as **"the KG skill"** because its name is banned from
this repository by a standing owner ruling.

Every source was fetched and read on 2026-09-17. Where a page said less than the question asked,
that is recorded as *"not stated"* rather than filled in. Where a claim is a vendor's claim it is
marked as such.

---

## 0. The one-paragraph summary

The field converged in 2025–2026 on five things an agent-facing toolchain is expected to have:
**(1)** a machine channel that is *uniform* (one JSON envelope, stable codes, typed repairs);
**(2)** *structure-aware, stale-state-guarded edits* rather than string patching (ZeroLang's
`--expect-graph-hash`, CodeStruct's AST-entity edits — measured to cut tokens 12–38% and raise
pass rates a few points); **(3)** *skills* — small Markdown instruction packs agents load on demand
(the Agent Skills standard, adopted by 20+ harnesses; Mojo ships one); **(4)** *MCP* as the
transport for tools, now stateless and cache-friendly (2026-07-28 spec); and **(5)** *persistent,
provenance-tagged code graphs* for repository understanding (99% token reductions claimed on
structural queries). DeluluLang already owns the hardest part of all five — a compiler-computed,
byte-deterministic answer to *what can this code do* — and exposes almost none of it through the
sockets agents actually plug into. The research verdict is therefore not "add AI features" but
**"put the existing truth behind the standard sockets, and fix the machine channel where it is
uneven."** Two things the field does that DeluluLang must *not* copy are recorded in §9.

---

## 1. ZeroLang (Vercel Labs) — the graph-native agent language

**Sources:** zerolang.ai (home, getting-started, diagnostics, testing, primitives, reference,
concepts/graph-architecture, concepts/semantic-vs-text, concepts/compile-path,
concepts/projections, learn) and github.com/vercel-labs/zerolang.

**What it is (as stated):** "The programming language for agents — the graph is the program."
Experimental (v0.1.x, Apache-2.0, 5.4k stars, Node/TypeScript with a C native component). The
compiler-owned graph store (`zero.graph`) is the program database; agents `zero query` and
`zero patch` it; `.0` files are *projections* for human review (`zero export`, `zero import`,
`zero verify-projection` detect drift and refuse to guess). Capabilities are explicit: a `World`
parameter is the only route to side effects. Diagnostics carry stable codes (`NAM003`, `GRF013`),
`--json` everywhere, and `zero fix --plan --json` returns a repair plan. Tests are graph nodes;
`xfail:` names expected failures and an unexpected pass fails the run. Install is
`curl -fsSL https://zerolang.ai/install.sh | bash` plus `npx skills add vercel-labs/zerolang`;
hello world is three commands and about two minutes.

**Evidence quality:** the semantic-vs-text page provides **no quantitative data** — no error-rate,
token or success-rate measurements. The architecture pages do not describe concurrency or merging.

**Ideas worth taking:**
- *Stale-state guards on edits.* `--expect-graph-hash` makes an edit against a moved target fail
  before it touches the store. DeluluLang can offer the same on **text** with a content hash per
  file, without a graph store (see `MASTER_PLAN.md` §6, item A4).
- *One loop, prescribed:* `query → patch → check → test → run`. DeluluLang's `for-agents.md` has the
  pieces but not the loop; a skill should state it.
- *Readiness facts:* when a backend cannot build a shape, the CLI says so structurally. DeluluLang
  already does this (`DL1201`); the lesson is to expose the *fragment boundary* as data.
- *The install experience:* one script, one PATH line, three commands. This is the bar.
- *Skills as a first-class deliverable*, installed by the same command every other project uses.

**Tradeoffs / why not copy the core:** a graph-as-database inverts the authority of text. It
forks every tool that reads files (git, diff, review, grep, editors, the Survey), and it moves
the trust surface to a binary store whose projection must be verified separately. DeluluLang's
canonical formatter already gives a byte-stable projection of a *text* program, and its
core-invariance snapshot pins what the toolchain says about it. **Verdict: adopt the guards and the
loop; reject the store** (`DECISION_LOG.md` D-NE-4).

## 2. Mojo (Modular) — skills, Python interop, packaging

**Sources:** mojolang.org (home, docs/vision, docs/tools/skills, docs/manual/python and its three
sub-pages, docs/manual/c-ffi, docs/tools/packaging).

**What it is:** "the systems language for the AI era", 1.0.0 (August 2026), Apache-2.0, Python
family syntax, GPU/CPU targets. Its identity is performance and hardware portability, not authority
— no overlap with DeluluLang's defining property, and its own vision page does not mention agents.

**Ideas worth taking:**
- **Skills.** `npx skills add modular/skills [--skill mojo-syntax]`; a separate `modular/skills`
  repository; content is "current guidance on syntax, patterns and workflows so agents generate
  modern, working code"; supported by Claude Code, Cursor and any Agent-Skills-standard client.
  This is exactly the vehicle DeluluLang lacks: the language's rules that trip agents (row
  discipline, `val`/`ref`, cap-relative paths, grant grammar) belong in a skill, not in a
  1,100-line handoff.
- **Python interop honesty.** Python 3.10–3.14 supported; `mojo build` does *not* bundle Python
  packages; imports must be inside functions; no selective imports. DeluluLang's own Python story
  (pyo3 0.25, a build that pins one `python313.dll`, a Python-less portable build) is *narrower but
  more honest* about the same problem. No change recommended beyond what `REMAINING_WORK.md` 7.13/7.14
  already say.
- **C FFI.** `external_call` and `OwnedDLHandle`; "the C header is the contract, and matching it is
  your job"; no signature validation. DeluluLang's `foreign "c" lib` block plus `ForeignCall` in the
  row is the stricter design; keep it.
- **Packaging.** Conda packages built by `rattler-build` from a `recipe.yaml`, hosted on
  prefix.dev/anaconda/S3, installed by `pixi`. **Not for DeluluLang**: it would import a package
  manager and a channel ecosystem for a toolchain that is one static binary. Recorded as rejected
  (`DECISION_LOG.md` D-NE-9).

## 3. Cloudflare — agent infrastructure

**Sources:** developers.cloudflare.com/browser-run/kitesurf, blog.cloudflare.com/kitesurf,
developers.cloudflare.com/browser-run/stagehand, github.com/cloudflare/workers-rs,
github.com/cloudflare/cloudflare-os, github.com/cloudflare/security-audit-skill; plus the Code Mode
posts found by search.

- **Kitesurf** — a stateless browser built for agents on Workers: "AI doesn't care about tabs … it
  cares about token count, context windows, scalability, performance, and costs"; every page load is
  untrusted input; a dedicated outbound worker mediates all network access; 3–7× less memory/CPU
  than Chromium (vendor claim). *Lesson:* design machine surfaces for tokens and isolation first.
  DeluluLang's `--json` channel is already uncapped and its human render capped at 50 — the right
  shape; the diagnostic **cascade** found in verification (145 diagnostics for one defect) violates
  the same principle and is scheduled.
- **Stagehand** — `observe / act / extract` over Playwright with an LLM. Not relevant to a
  language; noted for the shape of an agent tool API (verbs, schemas via Zod).
- **workers-rs** — Rust to `wasm32-unknown-unknown` under V8 isolates with CPU limits and
  `worker::signals` for graceful degradation. *Lesson:* DeluluLang's WASM host already has fuel,
  memory and wall-clock limits; a *signal before the limit* ("you are near your budget") is a small,
  agent-friendly addition for Contained plugins — recorded as a candidate, not scheduled.
- **cloudflare-os** — early-access "operating system for AI productivity": **capability-based
  access control; agents have zero default access; users introduce agents to services; gatekeepers
  mediate and approve asynchronously**; sandboxed gadgets. This is the platform-level twin of
  DeluluLang's holder model (zero ambient authority, explicit grants, a supervising Guard). The
  asynchronous-approval pattern matches the Guard's `request/pending/approve` flow. *Lesson:* the
  Guard is on the right track; what it lacks is a read-only surface an agent can poll (already
  named as `REMAINING_WORK.md` 6.5).
- **security-audit-skill** — a six-phase audit skill with `findings.json` (confirmed /
  needs_validation / rejected), a coverage ledger, and "deliberately does not execute untrusted code
  without sandboxing". *Lesson:* the three-verdict findings schema is a good shape for
  DeluluLang's own red-team records, which today are prose. Adopt the schema for future
  campaign records (recorded, low priority).
- **Code Mode** (search) — agents *write code* against generated TypeScript types instead of
  emitting one tool call per step; Cloudflare reports 32–81% token reductions and an API of 2,500
  endpoints described in ~1,000 tokens (vendor claims). *Lesson, and it is a strong one for the
  identity:* the industry is moving to "let the agent write a program, run it in a sandbox".
  DeluluLang is a language whose programs carry a compiler-checked authority bound — it is the
  natural target for exactly this pattern. The missing piece is a runnable plugin/host path and a
  distribution, not a new feature.

## 4. Agent frameworks and harnesses

- **Microsoft Agent Framework** (github.com/microsoft/agent-framework) — Python/.NET (Go separate),
  13.6k stars; agents, graph-based workflows with checkpointing, middleware, OpenTelemetry, "Agent
  Skills" as knowledge bases, A2A and MCP hosting. Permissions for tools are *not detailed* on the
  landing page. *Lesson:* observability by OpenTelemetry is table stakes for harnesses; DeluluLang's
  audit chain and effect trace are the stronger primitives but export nothing a harness ingests —
  an OTel-shaped export of the trace/audit is a candidate (not scheduled; needs an owner view on
  dependencies).
- **Anthropic cookbooks** — tool-use, agent patterns, context engineering (compaction, memory tool,
  clearing stale tool results), structured outputs, prompt caching. *Lesson:* long-running agents
  keep durable notes outside the window (`NOTES.md`/`TODO` files, an initializer that sets up
  feature lists and progress files). This repository's `HANDOFF.md` and the plan folder created
  by this commission *are* that pattern; the plan formalizes it (`EXECUTION_LOG.md` is the durable
  note).
- **Agent Skills standard** (agentskills.io/specification, github.com/agentskills/agentskills) —
  `SKILL.md` with YAML frontmatter (`name` ≤64 chars, lowercase/hyphens, must match the folder;
  `description` ≤1024 chars; optional `license`, `compatibility`, `metadata`, experimental
  `allowed-tools`), optional `scripts/`, `references/`, `assets/`; progressive disclosure (~100
  tokens of metadata at startup, <5,000 tokens of body on activation, resources on demand); keep
  `SKILL.md` under 500 lines; `skills-ref validate`. Introduced by Anthropic 2025-10-16, published
  as an open standard 2025-12-18, adopted by 20+ platforms. **Adopt** (`DECISION_LOG.md` D-NE-5).
- **MCP 2026-07-28** (modelcontextprotocol.io changelog) — stateless (no sessions, no
  initialize handshake; version and capabilities ride in `_meta` on every request), `server/discover`,
  Multi Round-Trip Requests replace server-initiated sampling/elicitation, Roots/Sampling/Logging
  deprecated, list results carry `ttlMs`/`cacheScope` and **should be returned in deterministic
  order for prompt-cache stability**, OpenTelemetry trace context in `_meta`. *Lesson:* an MCP
  server for DeluluLang should be stateless and deterministic — both are already this project's
  habits — and read-only (analysis), matching the LSP's "never an effector" rule. **Adopt, scoped**
  (`DECISION_LOG.md` D-NE-6).

## 5. Code intelligence, memory and semantic editing

- **codebase-memory-mcp** (DeusData) — tree-sitter over 162 languages into SQLite; 15 MCP tools
  (`index_repository`, `search_graph`, `trace_path`, `detect_changes`, `get_architecture`,
  `manage_adr`, `ingest_traces`, …); edges `CALLS`, `IMPORTS`, `DATA_FLOWS`, `SIMILAR_TO`,
  `SEMANTICALLY_RELATED`; per-language accuracy tiers stated (75–90% "good"); claims 99.2% token
  reduction on five structural queries and cites a preprint (arXiv:2603.27277) reporting 83% answer
  quality and 10× fewer tokens across 31 repositories. *Lesson:* the **query vocabulary** (trace
  a path, detect what a diff affects, architecture summary, ADRs as first-class records) is the
  right one. The Survey already answers `impact`, `affected-by`, `path`, `query` with a citation on
  every hop and *no inferred edges*; `detect_changes` (map a git diff to affected Survey nodes) is a
  cheap, high-value addition. Their accuracy tiers are the honest form of "inferred"; the Survey's
  answer is stricter (it refuses to infer at all).
- **The KG skill** (name withheld) — AST extraction for ~40 languages, LLM extraction for prose,
  edges tagged `EXTRACTED` vs `INFERRED`, community detection with LLM-labelled subsystems, god
  nodes, `query/path/explain`, an MCP server, HTML/JSON/GraphML outputs; benchmark numbers are
  published on memory datasets (LOCOMO recall@10 0.497 vs mem0 0.048; blind-judged). *Lesson:* the
  `EXTRACTED`/`INFERRED` tag is the single most transferable idea, and the Survey already lives at
  the `EXTRACTED`-only end; god-node and community views are useful *renderings* the Atlas partly
  has (top-10 degree list) and the Survey lacks. Not adopting LLM-labelled communities (a label
  nobody can cite is a discrepancy, by the Survey's own law).
- **mem0** — user/session/agent memory, an April-2026 "ADD-only" extraction (nothing overwritten),
  entity linking, multi-signal retrieval; published benchmark scores (LoCoMo 92.5, LongMemEval 94.4;
  vendor-run). **cognee** — remember/cognify/recall over a graph plus vectors, ontologies, code-repo
  ingestion, whole layer on one Postgres, v1.0, Apache-2.0. *Lesson:* memory for *conversations* is
  a harness concern, not a language concern. For a **repository**, the durable memory that matters
  is the decision record, the finding ledger and the execution log — all of which this project
  keeps as Markdown with provenance. The plan keeps it that way and makes the records
  machine-addressable (Survey node ids), rather than adding a vector store.
- **Semantic editing research** (CodeStruct, ACL 2026; `spockz/semantic-editor`; Comby / ast-grep /
  Semgrep for pattern rewrites) — "code agents over structured action spaces": `readCode`/`editCode`
  on named AST entities, syntax-validated, evaluated on SWE-Bench Verified across six LLMs:
  **+1.2–5.0% Pass@1 and 12–38% fewer tokens** for most models. *Lesson:* DeluluLang's Atlas already
  names every function with a stable id (`fn:demo/demo.fib`) and the checker already produces exact
  byte-range repairs. A **node-addressed, checked edit** (`replace the body of fn:<id>`, re-check,
  return the new hash and diagnostics in one envelope) is a small extension with measured upside.
  **Adopt, in two steps** (`DECISION_LOG.md` D-NE-4).
- **Codex Security** (OpenAI) — a vulnerability scanning CLI/SDK on a reasoning model; its
  learn.chatgpt.com page documents scans, diff scans, findings with severity/confidence/evidence and
  false-positive feedback; sandboxing internals are *not stated* there. *Lesson:* the findings
  schema (severity, confidence, location, evidence, remediation) is again the shape to adopt for
  records; no product overlap.

## 6. Capability- and effect-oriented languages (the kin)

- **Austral** — linear types plus capability-based security; capabilities are linear values,
  consumed or passed, never duplicated; explicitly aimed at bounding third-party dependencies.
  Small community; actively documented in 2026. *Relation:* the closest kin on the *why*; DeluluLang
  adds effect rows, a runtime broker, plugins and actors. Nothing to adopt beyond confirming the
  "supply-chain bound in the type" argument is shared by independent designers.
- **Pony** — 0.68.0 (August 2026); reference capabilities `iso val ref box tag trn`; 2026 breaking
  change: named types' default capability now applies to type-parameter constraints; FFI safety
  tightening (immutable references passed to mutating C are now refused). *Relation:* DeluluLang's
  rcaps are Pony's; the **DL1603 ergonomics finding** in verification (a `val` list literal of
  records) is a known Pony-family pain point, and Pony's answer is better diagnostics plus
  `recover` blocks. A `recover`-like construct is language-visible (RFC); better diagnostics are not.
- **Koka 3.2.3** (2026-03), **Flix**, **Effekt** — effect systems with handlers (Koka/Effekt) or
  a direct-style type-and-effect system with sub-effecting and purity reflection (Flix).
  *Relation:* Constitution §5.8 rejects resumable handlers for the security reading of rows;
  nothing here changes that. Flix's *effect exclusion* and *purity reflection* are the nearest
  ideas to `delulu why` and pure-function reporting; no adoption needed.
- **WebAssembly / WASI 0.3** (2026-06-11): async in the Component Model (`stream<T>`, `future<T>`),
  Wasmtime 43+ supports it; imports-as-capabilities is the standing model; WASI 1.0 targeted for
  2026. *Relation:* DeluluLang's `THREADED_WASM_DEFERRAL.md` waits on shared-everything threads;
  WASI 0.3 does not change that. The component model's `wasi:*/imports` worlds are the shape a
  future `Plugin[Contained]` import slice could be expressed in — noted, not scheduled.
- **Deno** — `--allow-*`/`--deny-*` (deny wins), permission sets per subcommand in `deno.json`
  (2.5). *Relation:* the closest mainstream UX to `--grant`; Deno's **deny-overrides-allow** and
  **per-command permission sets in a config file** are both worth having: a `[run-authority]`
  table in `delulu.toml` (declared, reviewable, diffable grants for `delulu run <package>`) is a
  candidate for the roadmap's ergonomics phase (`IMPLEMENTATION_ROADMAP.md` P1-11), consistent with
  the manifest-as-ceiling design. Kept as a candidate because it is a new grant *source* and needs
  a ruling that the ceiling law still holds.
- **Pel** (arXiv 2505.13453) — a Lisp-flavoured orchestration language with "capability control
  at the syntax level", natural-language conditions evaluated by LLMs, and a REPL with restarts.
  No evaluation results in the abstract. *Relation:* the one idea to **reject explicitly**: an LLM
  evaluating a condition *inside* the program makes execution non-deterministic and unverifiable,
  which contradicts DeluluLang's deterministic replay and `--assert-trace`. Agents stay outside the
  program; the program stays checkable.
- **aglang** (collivity) — `.ag` architecture specs compiled to SMT-LIB2 and checked by Z3 over
  facts extracted by tree-sitter; emits `AGENTS.md` and `skill.json` for agents; stable violation
  ids; 13 stars. *Relation:* confirms two habits — deterministic, LLM-free verification with
  provenance, and *emitting the agent brief from the checked artifact* (their `emit-context`). The
  latter is worth copying as a *pattern*: DeluluLang's skill and toolchain manifest should be
  generated from the binary's own tables where possible (the `--help`/completions gate already does
  this for commands).

## 7. Distribution practice for a Rust CLI

**Sources:** github.com/axodotdev/cargo-dist and its releases; GitHub artifact-attestations docs;
search results on SLSA levels.

- `cargo-dist` (actively maintained, 3,500+ commits) generates a `release.yml` that plans, builds
  per-platform archives and installers (shell, PowerShell, Homebrew, MSI, npm), publishes and
  announces on a version tag; supports SHA256 checksums and **GitHub artifact attestations**
  (SLSA Build L2; L3 with reusable workflows and isolation). `gh attestation verify` checks them.
- The project's own posture already matches the honest half: `scripts/package-toolchain.sh`
  produces a Python-less archive with `SHA256SUMS`, `INSTALL.txt`, licences and examples, and
  `INSTALL.md` explains why `cargo install` from crates.io is deliberately not offered.
- **Verdict:** the archive format is right; what is missing is *automation on a tag*, *all four
  targets in one run* (CI already builds them), *attestations*, and *shipping the morphs and the
  skill*. Whether `cargo-dist` or a hand-written workflow: recommend a **hand-written workflow**
  reusing `package-toolchain.sh`, because the script already encodes decisions (no Python, what is
  owed to a recipient) that `cargo-dist` would re-decide, and because dependency austerity is a house
  rule (`DECISION_LOG.md` D-NE-8). Installer scripts (`install.sh`/`install.ps1`) are an **owner
  decision** on posture (`curl | sh` vs. download-and-verify).

## 8. Field evidence that the problem is real (2026)

Search results surfaced, with their own sources: a coding agent deleting a production database
during a code freeze (July 2025, ~2,400 records); another wiping a production database in under
ten seconds (2026); a supply-chain compromise (litellm, 2026-03-24) that exfiltrated environment
variables, SSH keys and cloud credentials from every machine that installed the package; a
2026 report of 28.65 million new hard-coded secrets in public commits with AI-assisted commits
leaking at ~3.2% against a 1.5% baseline; a sandbox escape in a coding agent via a symlink
(CVE-2026-39861, severity 9.8); and a survey finding 93% of organizations had at least one
AI-caused infrastructure incident. These are third-party reports, not this project's measurements,
and they are cited for one reason: every one of them is "code exceeded the authority it was meant
to have" — the sentence DeluluLang was built around, and the symlink escape is the same class the
project itself found and fixed on 2026-08-10 (`SYMLINK-DANGLE-1`). The thesis has not aged; the
gap is reach.

## 9. What DeluluLang must NOT copy

1. **Graph-as-program-database** as the authoring surface (ZeroLang). Text stays authoritative;
   adopt the guards, not the store.
2. **LLM-evaluated semantics inside the language** (Pel). Non-deterministic execution is
   incompatible with `--assert-trace`, deterministic replay and the core-invariance snapshot.
3. **Inferred edges in the repository map** (the KG skill's `INFERRED`, codebase-memory's
   `SEMANTICALLY_RELATED`). The Survey's provenance law forbids an edge nobody can point at; keep
   it, and keep discrepancies as the honest substitute.
4. **A package manager ecosystem** (Mojo/conda). One static binary; one archive; checksums and
   attestations.
5. **Vendor-style token claims.** The project's honesty clauses ban them; the only numbers that
   ship are measured here.
6. **Holder-kind branches anywhere** (some frameworks key trust to "human vs AI"). Constitution
   §5.16 rejects them; every roadmap item was checked against this.

## 10. Source list

ZeroLang: https://zerolang.ai/ · /getting-started · /diagnostics · /testing · /primitives ·
/reference · /concepts/graph-architecture · /concepts/semantic-vs-text · /concepts/compile-path ·
/concepts/projections · /learn · https://github.com/vercel-labs/zerolang
Mojo: https://mojolang.org/ · /docs/vision/ · /docs/tools/skills/ · /docs/manual/python/ (and
python-from-mojo, mojo-from-python, types) · /docs/manual/c-ffi/ · /docs/tools/packaging/
Cloudflare: https://developers.cloudflare.com/browser-run/kitesurf/ · https://blog.cloudflare.com/kitesurf/ ·
https://developers.cloudflare.com/browser-run/stagehand/ · https://github.com/cloudflare/workers-rs ·
https://github.com/cloudflare/cloudflare-os · https://github.com/cloudflare/security-audit-skill ·
https://blog.cloudflare.com/code-mode/ · https://blog.cloudflare.com/code-mode-mcp/
Frameworks and standards: https://github.com/microsoft/agent-framework ·
https://github.com/anthropics/claude-cookbooks · https://agentskills.io/specification ·
https://github.com/agentskills/agentskills · https://modelcontextprotocol.io/specification/2026-07-28/changelog
Code intelligence and memory: https://github.com/DeusData/codebase-memory-mcp · (the KG skill,
name withheld) · https://github.com/mem0ai/mem0 · https://github.com/topoteretes/cognee ·
https://github.com/openai/codex-security · https://learn.chatgpt.com/docs/security/cli ·
https://arxiv.org/html/2604.05407v2 (CodeStruct) · https://github.com/spockz/semantic-editor
Languages: https://github.com/austral/austral · https://www.ponylang.io/ (2026 release posts) ·
https://koka-lang.github.io/koka/doc/book.html · https://doc.flix.dev/effect-system.html ·
https://wasi.dev/releases/wasi-p3 · https://docs.deno.com/runtime/fundamentals/security/ ·
https://arxiv.org/abs/2505.13453 (Pel) · https://github.com/collivity/aglang
Distribution: https://github.com/axodotdev/cargo-dist ·
https://docs.github.com/en/actions/concepts/security/artifact-attestations
Field incidents (third-party): https://www.morphllm.com/ai-coding-agent-security ·
https://www.docker.com/blog/ai-coding-agent-horror-stories-security-risks/ ·
https://www.osohq.com/developers/ai-agents-gone-rogue
