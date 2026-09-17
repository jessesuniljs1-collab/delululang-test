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
