# Execution log — Next Evolution 2026

Append-only. One entry per working session or phase. Facts only: what was inspected, what was
run, what was found, what was created, what was committed and pushed, what is unresolved. Commands
are the ones actually executed; results are the ones actually observed.

---

## Entry 1 — 2026-09-17, planning pass (Claude Fable 5.1, head chef)

**Starting state.** `master` at `52eecbe` (clean), `origin` = the public testing repository,
345 commits, tag `v1.0.0` at `198bf44`. One untracked file placed by the owner in `docs/design`,
named `DeluluLang_Fable_5.1_Master_Prompt.md`.

### Inspected (read in full unless noted)
`README.md`, `HANDOFF.md` (all 1,149 lines), `docs/REMAINING_WORK.md`, `docs/for-agents.md`,
`docs/GETTING_STARTED.md`, `docs/DEPLOYMENT.md`, `docs/editors.md`, `docs/MATHEMATICS.md`,
`docs/QUESTIONS.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/book/THE_DELULULANG_BOOK.md`,
`docs/design/AI_NATIVE_DESIGN.md`, `docs/design/CONSTITUTION.md`, `docs/design/STABILITY.md`,
`docs/design/DeluluLang_PROMPT.md`, `docs/release/CHECKPOINT-1.0.md`, `docs/survey/README.md`,
`docs/survey/SURVEY.md` (head), `docs/survey/DISCREPANCIES.md`, `INSTALL.md`,
`scripts/package-toolchain.sh`, `.github/CODEOWNERS`, `.github/workflows/ci.yml` (grep),
`rust-toolchain.toml`, `Cargo.toml`, `crates/delulu/Cargo.toml`, `docs/reference/primitives.md`,
the shipped examples and book samples, `tests/corpus/tier5-security/*.delulu`; targeted reads of
`crates/delulu/src/cli.rs` (`cmd_explain`, the `why` emitters, `--help`),
`crates/delulu-runtime/src/broker.rs` (grant grammar), `crates/delulu-runtime/src/prim.rs:366`,
`crates/delulu-check/src/resolve.rs` (the `Grant` record), `crates/delulu/tests/json_contract.rs`,
`evidence_claims.rs`, `distribution.rs`, `governance.rs`, `new_cli.rs`, `plugin_cli.rs`,
`crates/delulu-survey/src/{scan,paths,mdown}.rs`, `crates/delulu-atlas/src/lib.rs` (header),
`docs/design/STAGE6_SPECIFICATION.md` (status log), `STAGE6_BUILD_ORDER.md` (§4). Assistant
memory: `MEMORY.md` and `head-chef-handoff.md`.

### Web research (40 fetches/searches, 2026-09-17)
All twelve ZeroLang pages and the repository; nine Mojo pages; six Cloudflare pages/repositories;
Microsoft Agent Framework; the Claude cookbooks; codebase-memory-mcp; the KG skill (name withheld);
mem0; cognee; openai/codex-security and its CLI docs; the Agent Skills specification; the MCP
2026-07-28 changelog; cargo-dist; aglang; the Pel paper; searches on agent languages, capability
languages (Austral, Pony 2026, Koka/Flix/Effekt), WASI 0.3, Deno permissions, Code Mode, semantic
editing (CodeStruct), long-running-agent harness practice, 2026 agent security incidents, Rust
release attestation practice. Summarized with sources in `RESEARCH.md`.

### Commands run (abridged; every exit code was captured directly, never through a pipe)
- `cargo build --release` → exit 0 (fresh `target/release/delulu.exe`, 11:49).
- `delulu doctor` → 17 checks passed, 1 fixed (the map was behind because of the owner's new
  markdown file; regenerated). `delulu doctor --check --json` → one envelope, `errors: 0`.
- `delulu-survey check` → current; `findings` → 2 notes; `query doc:docs/design/CONSTITUTION.md`
  → ENTRENCHED; `impact mod:crates/delulu/src/lsp.rs` → 68 nodes;
  `impact mod:crates/delulu-check/src/check.rs --json` → 35 KB, uncapped.
- Fourteen programs written and driven: `check` (8 at once; 5 failing at once, `--json`),
  `authority` (human and JSON), `why`, `atlas` (tree/json/why/callers), `run` with and without
  grants (`--json --no-prompt`), `--engine wasm` (in and out of the fragment), `build --target
  wasm` + `run x.dwx`, `test --json`, `fix --dry-run --json` / `fix --json` /
  `--accept-widening`, `fmt --check` / `--stdin`, `morph render` (from two directories),
  `explain` (DL0501, E-REVOKE, E-PLUGIN, DL1603, DL0703, DL9999, `--json`), `new` / `new --lib` /
  `add --path` / `build` / `lock` / `test .`, `plugin build/inspect --json/verify`,
  `completions bash`, `--trace-effects`, `--seed`/`--clock fixed`, unknown flag, missing file.
- The language server driven by protocol (`initialize`, `didOpen`, `hover`, `executeCommand
  delulu.authority`, `codeAction`, `shutdown`, `exit`) with a 40-line Python client.
- The daemon broker in an isolated `DELULU_STATE_DIR`: `broker start/status/stop`, `grants
  delegate/list/revoke`, `run --lease` (twice with the same token), `guard status`, `audit tail`,
  `audit verify`.
- Repository facts: `git ls-files '*.md'` with sizes and last-commit dates (155 files), commits
  per month (204 / 127 / 14), per-crate line counts, `git grep -il` for the banned word (1 tracked
  hit: `HANDOFF.md`'s two rule lines — the pre-public item the owner already knows).

### Found
Sixteen findings, `NE-01…NE-16`, in `VERIFICATION_FINDINGS.md`. Headline: a program cannot load a
plugin at run time (`prim.rs:366` is a stub faulting under `DL0703`), documented as the flagship
demo and absent from `REMAINING_WORK.md`. Twenty-two positive claims re-verified, including the
full daemon-lease round trip.

### Failed attempts (kept)
- A first heredoc batch failed to parse (shell quoting) and wrote nothing; rewritten in two calls.
- Python read `/tmp/x.json` as a Windows path and failed; re-read via `cygpath -w`.
- The lease token was truncated at its `.` by a grep alternative that excluded dots → `DL1407
  malformed token envelope`; the audit chain recorded the two denied redemptions; redone with
  `grep '^dlt1_'` and the round trip succeeded.
- `--grant declassify:API_KEY` and five guessed plugin grant spellings were refused; the real
  grammar was read from `broker.rs` (bare `declassify`; no plugin dimension exists).

### Created
`docs/NEXT_EVOLUTION_2026/`: `README.md`, `MASTER_PLAN.md`, `RESEARCH.md`,
`VERIFICATION_FINDINGS.md`, `IMPLEMENTATION_ROADMAP.md`, `DECISION_LOG.md`,
`DOCUMENTATION_AUDIT.md`, `EXECUTION_LOG.md` (this file), `OWNER_COMMISSION.md` (redacted copy).

### Housekeeping
- The owner's commission file quotes the banned word. The Survey walks every markdown file on disk,
  so it could neither be committed as written nor left untracked (the committed map would disagree
  with CI's tree). The original was copied byte-for-byte to
  `D:\nelan\DeluluLang_Fable_5.1_Master_Prompt.md` (beside the repository, outside it) and removed
  from `docs/design/`; the redacted copy is `OWNER_COMMISSION.md`. Decision record: D-NE-12. The
  owner may move it elsewhere.
- `docs/REPOSITORY_STRUCTURE.md` §5 gained a subsection naming this folder.
- No source file, test, example, specification or entrenched document was changed.
- The Survey was regenerated by `delulu doctor` (see below) before the suite.

### Verification before commit
Recorded in Entry 2 (the commands and their results), so that a commit hash can be quoted beside
them.

### Unresolved
Everything in `MASTER_PLAN.md` §9 (owner decisions). The plan waits for approval.

---

## Entry 2 — 2026-09-17, the plan committed and pushed

### Verification run before the commit (all on this machine, Windows 11)
- `delulu doctor` → 17 checks passed; the Survey regenerated (the folder's nine files are now
  nodes); discrepancies **0 error, 0 warning, 2 note** (the two standing notes).
- The five `prose-cites-missing-path` warnings the first draft produced were real: the audit
  named archive destinations and the moved commission file in path form. Reworded, not silenced.
- `cargo test -p delulu --test evidence_claims --test distribution --test governance
  --test doctor_cli --test book --test json_contract --test cli_contract` → **all passed**
  (3 / 4 / 11 / 11 / 9 / 6 / 17), cargo exit 0.
- `cargo test -p delulu-survey` → all passed, exit 0.
- The full workspace suite was **not** run locally for this docs-only change; CI runs it.

### Commit and push
- Commit **`1ac8ecb`** on `master` (13 files: the nine plan documents, `REPOSITORY_STRUCTURE.md`
  §5, and the three regenerated Survey files). Message carries no skip token.
- `git push origin master` → `52eecbe..1ac8ecb`; `git ls-remote origin master` = local HEAD.
- CI run **`35193023549`** started by the push (status at the time of writing: *queued*).
  **Activated is not executed**: its result is to be read with `gh run view 35193023549` and
  recorded here by whoever continues, before it is described as green anywhere.

### State after this entry
Tree clean but for this entry. The plan waits for the owner's approval (`MASTER_PLAN.md` §9).
Nothing else is scheduled; no autonomous continuation was set up.

### Correction, same day
Commit `0fca1ef` appended Entry 2 to this file **after** the Survey had been regenerated, so the
committed map was stale against the tree (`delulu-survey check` → stale, exit 1) — the
self-inflicted failure `HANDOFF.md` §4 warns about, committed by the head chef anyway. Fixed by the
next commit, which regenerates the map as its last edit. Rule kept from it: the Survey is
regenerated after the final edit of a commit, and the log entry that records a commit is written
before that regeneration, never after. CI runs for `0fca1ef` are expected to fail the freshness
test; the run for the fixing commit is the one to read.

---

## Entry 3 — 2026-09-17 (afternoon), the sandbox + VM isolation research pass (Claude Fable 5.1, head chef; two sous-chefs)

**Commission:** `docs/design/DeluluLang_Sandbox_VM_Integrated_Next_Evolution_Prompt.md` (the owner
also returned the first commission to `docs/design`). **Instruction honoured:** no implementation;
the previous roadmap treated as unapproved; the result integrated into the existing plan.

### Read
`MASTER_PLAN.md`, `RESEARCH.md`, `VERIFICATION_FINDINGS.md`, `IMPLEMENTATION_ROADMAP.md`,
`DECISION_LOG.md`, `EXECUTION_LOG.md` (this pass's own inputs); `crates/delulu/src/microvm.rs`
(all 58 lines), `run_cmd.rs` 200–270 (the isolation gate), `foreign_worker.rs` 1–60 and 345–370,
`broker_transport.rs` 1–40, `crates/delulu-wasm/src/host.rs` and `limits.rs` headers,
`crates/delulu-runtime/src/prim.rs` (containment and the `http.get` arm), `crates/delulu/src/main.rs`
(the Linux gate), `tests/microvm_criterion8.rs`; `STAGE5_SPECIFICATION.md` §5, §6, §6.1, §10, §11
chunk 5; `ROOT_ISSUANCE_TRUST_BOUNDARY.md` in full; `THREADED_WASM_DEFERRAL.md`; the autonomy
addendum's invariant 52; `deny.toml`; the never-built `Dockerfile` and devcontainer;
`CROSS_PLATFORM_VERIFICATION.md`'s container and microVM lines.

### Web research (about 65 fetches and searches; sources in `SANDBOX_RESEARCH.md` §4)
Every URL the commission listed (AISI Inspect and its sandboxing toolkit and k8s provider, MITRE's
page — 403, NayaOne, AI Verify, Cloudflare Sandbox SDK, Vercel Sandbox, Firecracker's design,
production-host, snapshot, vsock and image docs, gVisor's security and platform guides, Kata's
architecture, Cloud Hypervisor and its virtio-fs doc, Nitro Enclaves and attestation, Azure dynamic
sessions — the custom-sessions page was 404, Foundry's code interpreter, Modal, Daytona), plus
Hyperlight and hyperlight-wasm, libkrun, Apple Containerization, Landlock's kernel doc, Windows
Hyper-V isolation, Wasmtime's security page, Chromium's Windows sandbox design, Claude Code's
sandbox runtime, Codex's security page, E2B's infrastructure, Docker Sandboxes' architecture post,
SandboxEscapeBench (arXiv 2603.02277), ControlArena, the runc breakout CVEs, Confidential
Containers, the bubblewrap/nsjail/firejail comparison, the Rust `landlock`/`seccompiler`/`birdcage`
crates, GitHub Actions KVM reports, WSL2's shared-kernel discussion, macOS `sandbox-exec`
deprecation, AWS AgentCore's DNS finding, Google's GKE Agent Sandbox, and CVE-2026-1386 (verified
against NVD/OSV/the GitHub advisory after the Sonnet sous-chef cited it).

### Commands run
- `delulu run hello.delulu --isolation none|process|microvm` (+ `--json`), `explain DL1408`,
  `doctor` (no isolation section); `grep` for fuel/epoch/limiter/timeout/rlimit in the run path,
  the WASM host and the worker; the net check at `prim.rs:275`; the `microvm` module gate.
- `gh run view 35193023549 / 35193076106 / 35193171249` — the three runs from the morning pass,
  read and recorded below.
- Added `.github/workflows/host-capability-probe.yml` (dispatch-only; every step
  `continue-on-error`; builds nothing), regenerated the Survey, committed **`bd074ea`**, pushed,
  `gh workflow run`, and read run **`35218542442`** (`completed success`, 17 s): the facts in
  `SANDBOX_IMPLEMENTATION_PLAN.md` §0.
- Reproduced the red team's device-name and trailing-character claims in a scratch directory
  (`write_text` of `NUL`, `CON`, `trail.txt.`, `space.txt ` under `--grant fs.write=.`); checked the
  `--json` envelope for an isolation field (none).
- Sous-chefs (owner-directed, D-NE-32): Opus 5 "red-team sandbox attack surfaces" (81 tool uses,
  ~14 min) and Sonnet 5 "host sandbox capability facts" (95 tool uses, ~17 min), each in an
  isolated worktree, each writing one notes file; both finished; nothing modified, built or pushed
  by either. Their notes: `agent-notes/RED-TEAM-SANDBOX-SURFACES-opus5.md` (344 lines) and
  `agent-notes/HOST-CAPABILITY-FACTS-sonnet5.md` (521 lines). Their worktree scratch files (probe
  programs, intermediate notes) copied to durable storage beside the repository at
  `D:\nelan\DeluluLang-agent-transcripts\2026-09-17-sandbox-pass\`. No reasoning transcript is
  kept — the owner's rule, as revised later the same day, forbids saving chain-of-thought; the
  agents' findings, decisions and outputs are their notes and those scratch files.

### CI results read this pass
| Run | Commit | Result |
|---|---|---|
| `35193023549` | `1ac8ecb` (the plan) | **green** — every push job (test ×3 OSes, arm64, supply-chain, miri ×2, miri-ffi, editor, lints, formal); heavy-gates and miri-slow skipped by design |
| `35193076106` | `0fca1ef` (the log entry that made the map stale) | **red** — the four test jobs failed (exit 101/1: the Survey freshness gate, as predicted in Entry 2); every other job green |
| `35193171249` | `9584011` (the fix) | **green** — every push job |
| `35218542442` | `bd074ea` (the probe, by hand) | **success** — three jobs, facts transcribed |
| `35218534498` | `bd074ea` (push) | **red** — the four test jobs (ubuntu, windows, macos, arm64) failed on the Survey freshness gate alone (`3 behind the tree: SURVEY.md, DISCREPANCIES.md, survey.json`), exactly as the note below predicted; the other nine jobs green |
| `35223153049` | `3fd69f2` (the sandbox pass) | **green** — every push job (test ×3 OSes, arm64, supply-chain, miri ×2, miri-ffi, editor, lints, formal), 10.5 min; heavy-gates and miri-slow skipped by design |

Note on `bd074ea`: its committed map was regenerated while four then-untracked draft files (the
sandbox documents of the interrupted first attempt of this pass) were on disk, so on CI its
freshness gate is expected to be **red**, exactly as `0fca1ef`'s was. The pass's final commit
regenerates the map as its last edit with every file committed. Recorded here rather than hidden.

### Found
NE-16b, NE-16c, NE-17 (no network client), NE-18, NE-19, NE-20, NE-21, NE-22, and the runners'
capabilities — `VERIFICATION_FINDINGS.md` §4.

### Failed attempts and corrections (kept)
- The first attempt at this pass was interrupted by the owner while writing
  `SANDBOX_TEST_PLAN.md`; four draft files survived on disk with the test plan cut mid-sentence.
  They were identified as this session's own drafts (section titles matching the head chef's
  design line by line; no other Claude process on the machine), reviewed, and completed.
- The first probe-log extraction used a `/tmp` path Windows Python cannot open; redone with a
  converted path.
- The two macOS Seatbelt probes in the CI workflow were badly designed (a too-strict profile; a
  closed port) and are recorded as inconclusive, to be redone in PS-0-08.
- A Bash echo string containing backticks executed `delulu run` by command substitution; harmless,
  and the wanted grep output was still produced.

### Created / changed
Created: `SANDBOX_RESEARCH.md`, `SANDBOX_ARCHITECTURE.md`, `SANDBOX_THREAT_MODEL.md`,
`SANDBOX_TEST_PLAN.md`, `SANDBOX_IMPLEMENTATION_PLAN.md`, `agent-notes/` (two files),
`.github/workflows/host-capability-probe.yml`. Changed: `MASTER_PLAN.md` (status, §8's order,
§9, new §12), `IMPLEMENTATION_ROADMAP.md` (the PS section), `DECISION_LOG.md` (D-NE-20…D-NE-33),
`VERIFICATION_FINDINGS.md` (§4), `README.md` (this folder), `OWNER_COMMISSION.md` (now a pointer),
`docs/REPOSITORY_STRUCTURE.md` §5.3 and §5.11, `HANDOFF.md` §11.1 (the owner's agent rule, then
its revision) and its pointer to this folder, `DOCUMENTATION_AUDIT.md` (rows for the two commission
texts, the five sandbox documents and the agent notes), `docs/design/DeluluLang_Fable_5.1_Master_Prompt.md`
(the banned word redacted in place, header note), `docs/design/DeluluLang_Sandbox_VM_Integrated_Next_Evolution_Prompt.md`
(committed for the first time, verbatim), and this file. No source file, test, example,
specification or entrenched document changed.

### Committed and pushed
- **`3fd69f2`** — everything this pass created and changed (22 files), the Survey regenerated as
  the last edit (1,139 nodes, 10,265 edges, 0 errors, 0 warnings — the five path warnings that
  appeared were upstream documentation paths in the Sonnet notes, resolved by a marked annotation);
  beforehand the four documentation gates (`evidence_claims` 9, `book` 4, `governance` 3,
  `distribution` 11) and `delulu doctor` (17 checks) green; pushed to the testing remote;
  `git ls-remote` equal to the local head. Its push run is `35223153049` (table above).
- The commit after it records this paragraph and the two run results, with the map regenerated
  last again, and corrects `HANDOFF.md`'s commit count.

### Owner rules recorded this pass
- Sous-chefs (stated by the owner during this pass and revised by the owner later the same day;
  the revision governs): Opus/Sonnet only, and only where an agent adds real value; high effort for
  difficult briefs; if the session limit is reached, agents are stopped gracefully and all completed
  work preserved — task progress, decisions, completed actions, pending tasks, relevant outputs and
  work state saved to the `.md` progress files and project storage before stopping; no private
  chain-of-thought or hidden reasoning saved or reproduced (`HANDOFF.md` §11.1; assistant memory
  `agent-usage-rule-2026-09-17`).

### Unresolved
The owner's decisions in `MASTER_PLAN.md` §9 items 9 and §12.2 question 12; whether PS-A precedes
P2 (the plan's recommendation) or follows it.
