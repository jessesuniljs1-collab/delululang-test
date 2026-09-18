# V2 execution log

> **Frozen at P1 (owner, 2026-09-18, D-V2-22).** The running log is [`V2_LOG.md`](V2_LOG.md).

Append-only; one entry per phase (or per interrupted attempt). Facts only: what was implemented,
which files changed, which commands ran and what they returned, the Survey, `doctor` and CI results,
the commits and pushes, what failed, what was fixed, what is unresolved, the owner's decisions, what
the agents did. No narration, no private reasoning. Every number comes from a command that ran;
"activated" is not "executed" — a CI run is recorded only once its result was read.

The V1-era planning passes that produced the plan V2 executes are logged in the archive
(`docs/archive/v1/NEXT_EVOLUTION_2026/EXECUTION_LOG.md`, Entries 1–3).

---

## Entry V2-0 — 2026-09-17 — the V2 workspace and the documentation migration

**Starting state.** `master` at `0bf8b11` (clean), `origin` = the public testing repository; one
untracked owner file, `docs/design/DeluluLang_V2_Execution_Master_Prompt.md` (the V2 commission,
0 occurrences of the banned word). The push run of `0bf8b11`, **`35224262541`**, was read at the start
of this phase: `completed`, `success`, every push job green (test on ubuntu, windows and macOS,
arm64, supply-chain, miri ×2, miri-ffi, editor, lints, formal); `heavy-gates` and `miri-slow` skipped
by design; 12:58:59Z → 13:09:14Z. That closes the V1 planning record: every run of 2026-09-17 read
and green on its final commit.

### Read before acting (owner's instruction, commission §1)
All sixteen files of `docs/NEXT_EVOLUTION_2026/` (now archived), `README.md`, `HANDOFF.md` (all
1,164 lines), `docs/MATHEMATICS.md`, `docs/GETTING_STARTED.md`, `docs/DEPLOYMENT.md`,
`docs/for-agents.md`, `docs/QUESTIONS.md`, `docs/REMAINING_WORK.md`, `docs/REPOSITORY_STRUCTURE.md`;
the commission in full; `crates/delulu-survey/src/{paths,mdown,rust,verify,lib,health}.rs` (the
resolver the migration extends); `crates/delulu/tests/{evidence_claims,governance}.rs` (the two tests
that name a moved path); `.github/CODEOWNERS`; `crates/delulu/src/doctor.rs` (how discrepancy
severities map to verdicts: only an error is a problem).

### COMPLETED
The five V2-0-0x tasks of `V2_IMPLEMENTATION_ROADMAP.md`:

- **V2-0-01** — `docs/DELULULANG_V2/` created with its ten files: `V2_README.md`,
  `V2_MASTER_PLAN.md`, `V2_IMPLEMENTATION_ROADMAP.md`, `V2_EXECUTION_LOG.md`, `V2_DECISION_LOG.md`,
  `V2_PHASE_STATUS.md`, `V2_DOC_MOVE_MANIFEST.md`, `V2_AGENT_LOG.md`, `V2_SECURITY_MODEL.md`,
  `V2_AI_NATIVE_DESIGN.md`. Two citations of paths that do not exist yet were reworded rather than
  left to dangle (`measurements/ai-usability/` and `examples/plugin_host/` became "a record
  `ai-usability` under `measurements/`" and "an example package `plugin_host` under `examples/`"), so
  the folder lands with no broken citation of its own.
- **V2-0-02** — 32 documents moved into `docs/archive/v1/` by `git mv`, paths preserved relative to
  `docs/`, nothing copied and nothing deleted. 5 explicit links rewritten in 3 files and 21
  root-relative prose citations rewritten in 9 active documents; historical records were left citing
  the paths they were written with.
- **V2-0-03** — the Survey's archive-mirror rule, in five source files: `ARCHIVE_ROOT` in `lib.rs`;
  `Resolution::Archived { cited, archived }` and `PathIndex::archived()` in `paths.rs`, tried after
  every reading of the present tree and gated on the archived file existing; the `Archived` arms in
  `mdown.rs` (prose → edge + note; explicit link → still an error, with the new location named) and
  `rust.rs` (comment → `References` edge + note); and `verify.rs::describes_the_present` returning
  `false` for `docs/archive/`, so an archived record's counts are never checked against today's tree.
  9 unit tests, three of them falsifications.
- **V2-0-04** — `crates/delulu/tests/evidence_claims.rs` (`HISTORICAL`) and
  `crates/delulu/tests/governance.rs` (`RESTATED_IN`) updated to the archived paths;
  `docs/archive/v1/README.md` written; `V2_DOC_MOVE_MANIFEST.md` written and finished;
  `docs/REPOSITORY_STRUCTURE.md` accounting (§1 tree, §5.3, new §5.10/§5.11, §5.12);
  `HANDOFF.md` and `README.md` pointed at V2.
- **V2-0-05** — commit, push and the CI run: see CI and GIT below.

### FILES
**Moved — 32, by `git mv`, paths preserved relative to `docs/`:** `docs/NEXT_EVOLUTION_2026/**`
(16, including `agent-notes/`) → `docs/archive/v1/NEXT_EVOLUTION_2026/**`; `docs/playbooks/**` (8) →
`docs/archive/v1/playbooks/**`; `docs/maintenance/**` (2) → `docs/archive/v1/maintenance/**`; and six
files from `docs/design/` → `docs/archive/v1/design/` (`LANGUAGE_SPECIFICATION.md`,
`PRODUCTION_READINESS_REVIEW.md`, `PRODUCTION_READINESS_2026-08-09.md`,
`PRODUCTION_READINESS_2026-08-10.md`, `P19_ECOSYSTEM_REVIEW.md`,
`STAGE10_AUTONOMY_HONESTY_REVIEW.md`). `git diff --cached --stat` reports *32 files changed,
0 insertions, 0 deletions* — pure renames, nothing rewritten in transit.

**Edited — 20.** Source (5): `crates/delulu-survey/src/{lib,paths,mdown,rust,verify}.rs`. Tests (2):
`crates/delulu/tests/{evidence_claims,governance}.rs`. Front doors (2): `README.md`, `HANDOFF.md`.
Documents (6): `docs/REPOSITORY_STRUCTURE.md`, `docs/REMAINING_WORK.md`,
`docs/design/CROSS_PLATFORM_VERIFICATION.md`, `docs/design/LOCALIZATION_PLUGIN_GUIDE.md`, and the two
archived files whose own links needed re-depthing after the move
(`docs/archive/v1/design/LANGUAGE_SPECIFICATION.md`,
`docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md`). Comments (2):
`crates/delulu-survey/src/codeowners.rs:16`, `.github/workflows/host-capability-probe.yml:1`.
Generated (3): `docs/survey/SURVEY.md`, `DISCREPANCIES.md` and `survey.json`.

**Created — 11:** `docs/archive/v1/README.md`; `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md`; and the
nine other V2 documents listed under V2-0-01.

**Committed for the first time — 1:** `docs/design/DeluluLang_V2_Execution_Master_Prompt.md`, the
owner's V2 commission. It was untracked, and the Survey walks the working tree, so the committed map
had come to depend on a file a fresh clone would not have; committing it makes the map a function of
the tree again (PROBLEMS, item 1).

**Deleted — 0.**

### TESTS
Pass 1 (the migration), each exit code captured directly:

| Command | Exit | Result |
|---|---|---|
| `python migrate.py moves` | 0 | 32 `git mv`, 0 failures |
| `python migrate.py links` | 0 | 5 links in 3 files |
| `python migrate.py prose` | 0 | 21 citations in 9 files; a re-run reports 0 (idempotent) |
| `cargo test -p delulu-survey` | 0 | 65 passed, 0 failed |
| `cargo clippy -p delulu-survey --all-targets -- -D warnings` | 0 | clean |
| `cargo build --release -p delulu` | 0 | finished |
| `cargo test -p delulu --test evidence_claims --test governance --test book --test distribution --test doctor_cli --no-fail-fast` | 0 | 38 passed, 0 failed (9 / 4 / 11 / 3 / 11) |
| `cargo test -p delulu-conform every_grammar_production_is_defined_in_a_normative_specification` | 0 | 1 passed |

**Falsified, not assumed.** Removing the `self.exists(&candidate)` gate from `PathIndex::archived()`
— so that any `docs/…` path would be reported as archived — made `cargo test -p delulu-survey --lib`
report **30 passed, 3 failed**, the three failures being exactly the three falsification tests. The
source was restored byte-for-byte and the suite ran green again. Separately, one rewritten link was
broken back to its pre-move target in place and the Survey reported
`error: README.md:47 [broken-link] … (the file moved to docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md)`,
which exercised the rewriter and the rule's new message in one run; it was then restored.

**A test whose meaning could have changed, checked before it was relied on.**
`delulu-conform::every_grammar_production_is_defined_in_a_normative_specification` scans
`docs/design/*.md` **non-recursively** for EBNF, so moving a design file out of that folder could
have emptied a production's definition. Exactly one production name, `point`, was defined only in a
moved file, and `point` is in neither `GRAMMAR_PRODUCTIONS` nor the normative spellings in
`NORMATIVE_NAME`; 107 productions remain in `docs/design/` against the test's floor of 50. The test
then ran and passed.

Pass 2 (placing the V2 documents and the front-door pointers), final runs of this phase:

| Command | Exit | Result |
|---|---|---|
| `cargo run -p delulu-survey -- build` | 0 | see SURVEY |
| `./target/release/delulu.exe doctor --check` | 0 | see DOCTOR |
| `cargo test -p delulu-survey` | 0 | 65 passed, 0 failed |
| `cargo test -p delulu --test evidence_claims --test governance --test book --test distribution --test doctor_cli --no-fail-fast` | 0 | 38 passed, 0 failed |
| `grep -rn "<the banned word>" --include=*.md docs/DELULULANG_V2 docs/archive/v1/README.md` | 1 | no match, as required |

### SECURITY
No security-relevant behaviour changed. The only source changes are the Survey's archive-mirror rule
(documentation tooling; `publish = false`) and two path strings in tests. Nothing in the language,
the runtime, custody, the Guard or containment moved; the core-invariance snapshot is untouched.

### SURVEY
`cargo run -p delulu-survey -- build` → exit 0, regenerated as the last edit of the phase:
**1151 nodes, 10398 edges, 27 discrepancies — 0 error, 2 warning, 25 note.**

**0 errors** is the gate this phase had to meet, and it is met: every explicit Markdown link in the
tree resolves after the moves.

**Warnings — 2**, both in the owner's commission, which is kept verbatim:
`docs/design/DeluluLang_V2_Execution_Master_Prompt.md:142` and `:145` name a plan file that has never
existed, first at a design path and then at its mirror under the archive, illustrating the
convention. Accepted for now (PROBLEMS, item 5).

**Notes — 25**, in three classes: **23 `cites-archived-path`**, plus the two that predate this
phase (`c-token-not-a-campaign-finding`, `ruling-cited-without-its-stage`). Every one of the 23 is
listed by file and line in `docs/survey/DISCREPANCIES.md`, and every one is a document that is
*correct as written*: `CHANGELOG.md`, five `STAGE<N>_BUILD_ORDER.md` precedence lines,
`SURFACE_ATLAS_PALETTE_ADDENDUM.md`, the owner's 2026-09-17 commission, the archived documentation
audit, `docs/REPOSITORY_STRUCTURE.md` §3's 2026-07-05 manifest, and this phase's own move manifest,
whose "Original path" column exists precisely to record the pre-move spelling. There are **0**
`comment-cites-archived-path` notes: the one Rust comment that cited a moved path
(`crates/delulu-survey/src/codeowners.rs:16`) is an active source file and was rewritten, so that
arm of the rule is held down by its two unit tests rather than by a live citation.

`comment-cites-missing-path` and `broken-link` are both **0**.

### DOCTOR
`./target/release/delulu.exe doctor --check` → exit **0**, `ok: 17 check(s) passed`. The repository
section, verbatim:

```
repository
  ok       delulu source tree           D:\nelan\DeluluLang
  ok       survey freshness             the map matches the tree — 1151 nodes, 10398 edges
  ok       every edge cites a line      10398 edges, all with a file and line
  ok       no edge dangles              every endpoint is a node
  ok       totals agree with contents   13 workspace members, 9 shipped
  ok       every node has a usable id   1151 nodes, all addressable as `kind:name`
  note     discrepancies                0 error, 2 warning, 25 note — docs/survey/DISCREPANCIES.md
```

### CI
Push run **`35258166713`** on `e48f9c3`, read with `gh run view` after it completed: **success** — every push job green (test on ubuntu-latest, windows-latest and macos-latest; arm64; supply-chain; miri on delulu-diag and delulu-atlas; miri-ffi; editor; lints; formal); heavy-gates and miri-slow skipped by design; 18:19Z to 18:30Z, 10.7 min.
The recording commit that adds this paragraph starts a run of its own; it is read and recorded
at the start of the next phase (the convention since the planning passes).

### GIT
Phase commit **`e48f9c3`** (`e48f9c316436fd1d6c623afd6bdece02753e839e`) on `master`: 32 renames, 18 modified,
12 added (the ten V2 files, the archive README, the owner's commission text), 0 deleted; the
Survey regenerated as the last edit. Pushed to `origin` (the testing repository);
`git ls-remote origin master` equal to the local head. No skip token in the message. The
recording commit follows.

### AGENTS
**Pass 1 — Opus 5, the documentation migration.** 274,080 tokens, 138 tool uses, 29 min 33 s.
Brief: `D:\nelan\DeluluLang-agent-transcripts\2026-09-17-v2-0\BRIEF-migration-opus5.md` — the 32
moves, the link and citation rules, the Survey's archive-mirror rule with unit tests and a
falsification, the two test path updates, the archive README, the manifest draft, the accounting
edits in `docs/REPOSITORY_STRUCTURE.md`, and seven named verification commands; no commit, no push,
no deletion, no source change beyond the named ones. It executed all six objectives, falsified its
own gate by mutation and its own link rewriter by breaking a link back, and checked the one test
whose meaning the moves could have changed before relying on it (all three recorded under TESTS).
It also found and fixed two warnings it had introduced itself: its first `ARCHIVE_ROOT` doc comment
illustrated the mirror with an invented playbook file name on both sides of the move, and the Survey
correctly reported both as missing paths — reworded to the `docs/<x>` placeholder form, 9 warnings
down to 7.

**It stopped on four decisions rather than taking them**, which is the standing rule and is the
reason this entry can be trusted: the committed map depending on the untracked owner prompt; §3 of
`docs/REPOSITORY_STRUCTURE.md` being a historical record inside an active document; the §5 section
numbering; and the archive README's worked example. All four are resolved under PROBLEMS and
recorded in `V2_DOC_MOVE_MANIFEST.md`.

**Pass 2 — Opus 5, finishing the phase.** Brief: `BRIEF-finish-opus5.md` in the same folder — place
the nine remaining V2 documents, reword the two citations of paths that do not exist yet, fill this
log, the agent log and the phase-status row, apply the `HANDOFF.md` and `README.md` pointer texts,
finish the manifest, and re-verify with the Survey regenerated as the last edit. Usage figures for
this pass are added by the head chef.

Every claim above was verified against the current binary or source before it was written here.
Outputs in project storage (`D:\nelan\DeluluLang-agent-transcripts\2026-09-17-v2-0\`):
`PROGRESS-opus5.md`, `migrate.py` (the re-runnable migration), `REPORT-migration-opus5.md`, and the
two briefs. No chain-of-thought is stored, here or there.

### PROBLEMS
1. **The regenerated map depended on an untracked file.** `docs/design/DeluluLang_V2_Execution_Master_Prompt.md`
   was present on this disk and untracked; the Survey walks the working tree, so the map acquired a
   node, an edge and seven warnings for a file a fresh clone would not have — every CI runner would
   have failed `the_committed_map_matches_the_tree` while the map was locally correct, which is the
   clean-checkout lesson of 2026-09-14 arriving by a new route. **Resolved: the commission is
   committed in this commit.**
2. **A historical record inside an active document.** `docs/REPOSITORY_STRUCTURE.md` §3 is the move
   manifest of the 2026-07-05 repository init; the blanket citation rewrite had changed its
   `LANGUAGE_SPECIFICATION.md` row to the archive path, making a July record claim a September
   destination. **Resolved: the row is kept as written** and the Survey reports it as a
   `cites-archived-path` note. The migration script carries an `EXEMPT_LINES` entry so a re-run
   preserves it.
3. **Section numbering.** Removing the playbooks section left `docs/REPOSITORY_STRUCTURE.md` §5 with
   twelve sections, not thirteen. **Resolved: numbered contiguously**, §5.10 archive, §5.11 V2,
   §5.12 generated. No file anywhere cites those numbers.
4. **The archive README's worked example** named a real pre-move path and so reported a note against
   itself. **Resolved: reworded to the placeholder form.**
5. **Accepted for now — two warnings.** `docs/design/DeluluLang_V2_Execution_Master_Prompt.md:142`
   and `:145` name an `OLD_PLAN.md` that has never existed, once at a design path and once at its
   mirror under the archive; both are placeholders in the owner's own text, illustrating the mirror
   convention. The commission is kept verbatim, so they stay until the owner decides. They are the
   only warnings in the map.
6. **Nothing else.** No file was deleted, no never-edit path was modified, `.claude/` was never
   walked or touched, and `docs/survey/` was only ever regenerated.

### DECISIONS
D-V2-01…D-V2-16 recorded in `V2_DECISION_LOG.md` (the approval; the archive folder; the mirror rule;
what moved; Opus 5 as the main sous-chef; the phase pause; P1 and PS-0 as two phases; resource
authority; the execution modes; no holder-kind branch; the usability benchmark; `SandboxPolicy`; the
sandbox default narrowed; the two AI-native design files; the commission texts; `fix` stays the repair
command).

### NEXT
P1 — machine-contract truth (`V2_IMPLEMENTATION_ROADMAP.md`), after the sixty-second pause.

---

## Entry P1 — 2026-09-18 — machine-contract truth

Executed by one Opus 5 sous-chef under the head chef (Claude Fable 5.1), brief
`D:\nelan\DeluluLang-agent-transcripts\2026-09-17-p1\BRIEF-P1-opus5.md`. Thirteen tasks; **P1-11
deferred by D-V2-17** (not implemented, not documented as coming). Baseline: HEAD `94c6e29`, clean
tree, `cargo build --release` exit 0, pre-fix binary kept as `delulu-pre-p1.exe`.

### Reproduced first, on the pre-fix binary
All fourteen findings NE-01…NE-14 were re-driven by hand before any edit
(`repro.sh` → `repro-before.txt`, 1,136 lines; exit codes captured per command, never from a
pipeline). **Thirteen reproduce exactly.** NE-04 reproduces to the number: 145 diagnostics,
73 × DL0210 + 72 × DL0404.

**One does not, and the difference matters.** NE-02's abbreviated program, reconstructed
(`let shapes: val List[Shape] = [Dot(Point{…}), Box(…)]` over `type Point {x:Int,y:Int}` and
`type Shape = Dot(Point) | Box(Point,Point)`) **checks clean**: a record or sum of only-`val`
components defaults to `val` (`rcaps.rs::default_rcap_inner`, build-order deviation 8). The
load-bearing shape does reproduce — a `val` composite literal one of whose elements is an explicitly
`ref` binding — with NE-02's exact message, no repair, and an `explain DL1603` about a `val` closure
over a `ref` capture. P1-09 is written against that shape, and the reproduction is recorded rather
than the description (the *reproduce the shape, not the description* rule).

### COMPLETED
- **P1-01** `docs/REMAINING_WORK.md` gained **4.11** (a program cannot load a plugin at run time;
  `prim.rs:366`; `check` and `authority` both accept such a program and only `run` refuses; no grant
  spelling enables it) and **6.13** (a filesystem miss does not name the capability scope — recorded
  as a *candidate*, not a gap: `IoErr`'s variants are part of the accepted language, so giving
  `NotFound` a payload is not a message-only change). The Book Ch. 10, `STAGE6_PLUGINS_GUIDE.md`,
  `examples/plugin_shout/README.md`, `README.md` and `HANDOFF.md` now say the in-language load
  surface is a runtime stub until P2 — sentences added, nothing deleted.
- **P1-02** the success-envelope sweep, written first and watched fail on **17 commands**. Wrapped:
  the five `why` emitters, `add` (3 sites), `plugin build|verify|inspect`, `test`, `atlas` (graph and
  the five query verbs), `--version`, `fmt` (check and migrate), `locale add|list`, `morph list|info`,
  `secrets list|set`, `audit tail|query|verify|bundle|reconcile`, `keygen`, `sign`, `verify-sig`,
  `login`, `publish`, `deploy plan`, `authority --diff`, `authority <artifact>`. One helper
  (`cli::success_envelope`) carries the five fields; a report's keys merge at the TOP level so no
  existing field moves, `summary` merges key by key, and a command whose exit can be nonzero sets
  `summary.errors` itself. Beyond NE-05: **`secrets list --json` and `secrets set --json` accepted
  `--json` and ignored it entirely** — the NE-06 shape on a security surface.
- **P1-03** `explain --json` → `explain: {code, title, body, disposition, kind}` in the envelope,
  `kind` ∈ {`topic`, `code`, `unallocated`}; `body` and `disposition` always present, `null` where
  they do not apply. `--bogus`, `--jsonn`, `-j`, `--json=1` all still exit 2.
- **P1-04** one `DL0210` per overflow site, and the recovery placeholder is no longer type-checked as
  a call: **145 → 1**. Two mechanisms, each independently witnessed (below). `DL0211/0212/0213` are
  gated to the same rule; all three already emitted once and the gate now holds them there.
- **P1-05** `Repair` gained `reason`, and `Diagnostic::with_repair` — the single path a repair reaches
  a diagnostic by — makes an editless repair `requires_human`. Two such repairs existed
  (`remove_effect_from_row` on DL0502, `R-DL0802-attenuate-to-intersection` in the broker) and both
  claimed machine-applicability. The LSP no longer marks an action `isPreferred` without an edit.
- **P1-06** `test --json` carries `diagnostics` (per-file, positioned against that file's own
  `SourceMap`), a ceiling failure gains `failure.code`, `summary` gains `errors`, and under `--json`
  nothing human-rendered reaches stderr.
- **P1-07** bare `delulu test` inside a package targets the package; outside one the refusal stands,
  with a message that now names both conditions.
- **P1-08** `authority --grants` (human), `required_grants` and `requested_scopes` (JSON, additive,
  per capability and as a top-level map), and the same walk feeding the Atlas.
  `scripts/package-toolchain.sh`'s `INSTALL.txt` sentence is true and points at `--grants`.
- **P1-09** DL1603 for a `val` container over `ref` contents names the element (a secondary span on
  it), says why a `val` container needs `val` contents, and offers a **`safe`**
  `drop-val-annotation` repair that removes the whole annotation — removing only the keyword does not
  fix it, because an annotated `List[List[Int]]` still defaults to `val`. `explain DL1603` gained the
  case it always talked around, with the three ways out. The closure case keeps its own shape.
- **P1-10** `examples/guide/05_capabilities.delulu` reads and writes; the rule is stated in
  `explain E-DL0703`, `docs/for-agents.md` [agents.cap-paths] and the Book Ch. 5 (with a checked
  sample, `11_capability_paths.delulu`); `examples_run.rs` asserts the read and the write **succeed**
  and carries the negative control.
- **P1-12** the accounting gate exists, reads the generated `survey.json` and §5, and carries a
  mutant. It found `measurements/METHODOLOGY.md` unaccounted; §5 now names it, and both claims of
  mechanical checking say what is checked.
- **P1-13** `GETTING_STARTED.md` §9 and `docs/for-agents.md` [agents.tests] say an effectful test
  needs a package `[test-authority]`, show the manifest, and say `delulu test --test-authority` does
  not exist and why (D-NE-17 is the owner's).

### FILES
45 touched: **43 modified, 2 added** (`crates/delulu/tests/repository_structure.rs`,
`docs/book/samples/11_capability_paths.delulu`), **0 deleted, 0 renamed**.

Source: `crates/delulu/src/cli.rs` (+808/−), `signing.rs`, `deploy.rs`, `lsp.rs`;
`crates/delulu-syntax/src/parser.rs`; `crates/delulu-check/src/rcap_check.rs`,
`crates/delulu-check/src/check.rs`, `crates/delulu-check/src/deps.rs`,
`crates/delulu-check/src/plugin.rs`, `crates/delulu-check/src/lib.rs`;
`crates/delulu-diag/src/diagnostic.rs`, `crates/delulu-diag/src/json.rs`,
`crates/delulu-diag/src/codes.rs`; `crates/delulu-broker/src/diag.rs`.
Tests: `json_contract.rs` (+427), `cli.rs` (+353), `fix_cli.rs` (+207), `examples_run.rs` (+96),
`new_cli.rs` (+55), `lsp_cli.rs` (+49), `test_runner_cli.rs`, `atlas_cli.rs`, `atlas_e2e.rs`,
`repository_structure.rs` (new).
Documents: `CHANGELOG.md` (an `Unreleased` entry), `README.md`, `HANDOFF.md`,
`docs/REMAINING_WORK.md`, `docs/REPOSITORY_STRUCTURE.md`, `docs/GETTING_STARTED.md`,
`docs/for-agents.md`, `docs/book/THE_DELULULANG_BOOK.md`,
`docs/design/STAGE6_PLUGINS_GUIDE.md`, `examples/plugin_shout/README.md`,
`examples/guide/05_capabilities.delulu`, `scripts/cli-sweep.sh`,
`scripts/package-toolchain.sh`, `tests/core-invariance/SNAPSHOT.txt`, and the generated
`docs/survey/` files.

**Never-edit paths: none touched.** No entrenched file, no `docs/release/*`, no `docs/archive/**`, no
commission text, no `.claude/`, no conformance witness file, no laundering suite. `CHANGELOG.md` was
appended to, as the rule requires.

### TESTS (every command's exit code read directly, never from a pipeline)
| Command | Result |
|---|---|
| `cargo build --release` | **0** |
| `cargo test --workspace --no-fail-fast` | **0** — 126 test binaries, **1,674 passed, 0 failed, 4 ignored** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `bash scripts/cli-sweep.sh target/release/delulu.exe` | **0** — `SWEEP OK — 28/28`, 0 problems (27 before: P1-07 split the bare-`test` case into inside-a-package and outside-a-package) |
| `cargo run -q -p delulu -- fmt --check docs/book/samples` | **0** — 11 clean, 0 would change |
| `cargo run -q -p delulu-conform -- --coverage` | **0** — `PASS: 100% anchor coverage` (grammar 27/27, audit 7/7, cli 29/29, rule 54/54) |
| `cargo run -q -p delulu-conform -- --check-reference` | **0** — in sync (24 chapters), nothing regenerated |
| `cargo test -p delulu --test core_invariance` | **0** after the bless |
| `cargo run -p delulu-survey -- build` then `-- check` | **0** — 1,153 nodes, 10,609 edges |
| `cargo run -p delulu-survey -- findings` | **0 errors**, 2 warnings — the two known `OLD_PLAN.md` placeholders and nothing else |
| `./target/release/delulu.exe doctor --check` | **0** — 17/17 |
| `git grep -n -i <the banned word> -- . ':!HANDOFF.md'` | no output (grep exit 1) |

Focused tests, each **falsified and restored**, with the exact edit:
| Task | Gate | Falsification | Observed failure |
|---|---|---|---|
| P1-02 | `json_contract::every_json_success_emits_the_documented_envelope` | reverted the `why` "performs: true" emitter to its bare `println!` | `why: command = null`, `schema = null`, `delulu_version missing`, `diagnostics missing`, `summary missing`, `exited 0 but summary.errors = null` |
| P1-03 | `json_contract::explain_answers_on_the_machine_channel_for_all_three_registries` | dropped `"kind": "code"` from the allocated-code answer | the test printed the answer it got and named the missing field |
| P1-04a | `cli::one_over_deep_expression_yields_exactly_one_diagnostic` | removed `skip_over_deep_operand()` | **145 diagnostics** (1 × DL0210 + 72 × DL0201 + 72 × DL0404) |
| P1-04b | the same test's multi-argument case | disabled the per-site latch | **5 × DL0210** at 126 levels around a 5-argument call |
| P1-05 | `fix_cli::a_repair_with_no_edits_says_a_human_must_decide_and_why` | removed the normalization in `with_repair` **and** cleared the literal flag (clearing the flag alone is silently repaired — that is the fix working) | "has no edits but requires_human = false"; "`fix` says it requires a human and `check --json` says it does not" |
| P1-05 | `lsp_cli::a_code_action_with_no_edit_is_never_preferred` | `isPreferred: !widening` | the action with no `edit` came back `isPreferred: true` |
| P1-07 | `new_cli::bare_delulu_test_passes_inside_a_freshly_scaffolded_package` | disabled the package default | exit 2, "no `delulu.toml` and no ./tests directory here" |
| P1-08 | `cli::every_grant_kind_the_runtime_parses_is_derivable_from_the_report` | dropped the `sensor` flag | `not derivable from any authority report: ["sensor"]` |
| P1-09 | `delulu-check::a_val_container_over_ref_contents_names_the_element_and_offers_a_repair` and the `fix_cli` registry gate | disabled the repair | "a repair must be offered: []"; "val_over_ref.delulu must produce at least one repair" |
| P1-10 | `examples_run::the_capabilities_guide_actually_reads_and_writes` | reintroduced `"./config/app.txt"` in the shipped example | "the guide's READ must succeed": `no config` |
| P1-12 | `repository_structure::the_accounting_rule_refuses_a_file_nobody_listed` | the mutant is permanent: five synthetic unlisted paths must be refused on every run | (a vacuous rule fails the test by construction) |

### SNAPSHOT (core-invariance, regenerated deliberately — D-NE-3)
Unblessed run saved to `snapshot-diff-P1.txt` (8,358 lines, `cargo test` exit 101). **58 cases
changed; every changed line is attributable to P1-04, P1-05 or P1-08 and to nothing else:**

| Cases | Command | What moved | Task |
|---|---|---|---|
| 49 | `authority --json` | **only** the two new keys `required_grants` and `requested_scopes` (the latter also inside each `capabilities[]` entry). No existing field changed type, value or position. | P1-08 |
| 6 | `check --json` | **only** the new `reason` key on each repair; in 2 of them `requires_human` also `false → true` for DL0502's `remove_effect_from_row` | P1-05 |
| 3 | `check` (human) | `repair: remove_effect_from_row (safe)` gained `  [requires human decision]` | P1-05 |
| 1 | `DL0210_deep_nesting :: check --json` | 145 diagnostics → 1; `summary.errors` 145 → 1 | P1-04 |
| 1 | `DL0210_deep_nesting :: check` | `145 error(s)` and the 50-cap note → `1 error(s)` | P1-04 |

No case moved for P1-09: no corpus program has the `val`-over-`ref` shape. A **second** bless followed
the invariant-45 correction below and moved exactly one line: `"exec.native"` left
`25_attributes_hints.delulu`'s `required_grants` (saved as `snapshot-diff-P1-second.txt`). Net
`SNAPSHOT.txt`: 475 insertions, 3,851 deletions — the deletions are the 144 cascade diagnostics.

### SECURITY
No change to what Authority or the Guard mean. No holder-kind branch. No new grant source (P1-11
deferred). No new dependency. No change to the accepted language: every program refused before is
refused now, and the four nesting caps keep their values and their conformance witnesses.
`requested_scopes` is labelled **requested** everywhere and never *granted* — `scopes` (from the
manifest and the grant) remains the only field that may be read as a decision the operator made, and
a test asserts the two stay distinct for a program with no manifest.

**One semantic invariant was nearly broken, and the suite caught it.** The first version of
`required_grants` derived `exec.native` from a `@jit` hint, so the hinted and unhinted twins' reports
differed — **invariant 45** says a hint may not change what a program may do, and `required_grants` is
part of the authority report. It is also simply false: the program runs interpreted without the grant
and says so (DL1906). `exec.native` was removed from the derived list and is asserted on
`native_emission.requested` instead, where the request already lives.
(`attributes_cli::invariant45_hints_change_nothing_observable`.)

### SURVEY
`cargo run -p delulu-survey -- build` was the last edit: **1,153 nodes, 10,609 edges, 0 errors, 2
warnings, 25 notes**. `-- check` → 0, "the Survey matches the tree". The two warnings are the known
`OLD_PLAN.md` placeholders in the owner's commission text. **Five warnings this phase introduced were
reworded away rather than accepted**: two comments citing `path::symbol` (a form the path checker
reads as a path) and three brace-grouped path lists in this very entry. `impact` before any edit: `cli.rs` 68, `parser.rs` 162, `check.rs` 145,
`rcap_check.rs` 145, `authority.rs` 145.

### DOCTOR
`./target/release/delulu.exe doctor --check` → exit 0, **17 checks passed**, `0 error, 2 warning,
25 note`.

### CI
Push run **`35299533343`** on `d8dbc24`, read with `gh run view` after it completed: **success**: every push job green (arm64; formal; test (macos-latest); lints; miri (delulu-atlas); supply-chain; editor; miri (delulu-diag); test (windows-latest); test (ubuntu-latest); miri-ffi); skipped by design: heavy-gates, miri-slow; 02:29Z to 02:40Z, 10.8 min.
The recording commit that adds this paragraph starts a run of its own; it is read and recorded
at the start of the next phase (the convention since the planning passes).
Read at the start of this phase: run `35259510471` on V2-0's recording commit `94c6e29` —
`completed`, `success`, every push job green. Both of V2-0's commits are therefore on green runs.

### GIT
Phase commit **`d8dbc24`** (`d8dbc2498aef9d2833119f276b7d7100b8f5739c`) on `master`: 46 modified,
2 added, 0 deleted, 0 renamed; the Survey regenerated as the last edit. Pushed to `origin` (the
testing repository); `git ls-remote origin master` equal to the local head. No skip token in the
message. The sous-chef made no commit and no push, as the brief required.
Re-run by the head chef after the recording edits and before the commit, each exit status read
directly: the documentation gates (`evidence_claims`, `governance`, `book`, `distribution`,
`doctor_cli`, `repository_structure`: six binaries, 40 tests, all passed, exit 0); the Survey's
freshness test (5 passed, exit 0); `survey build` (1,153 nodes, 10,636 edges, 0 errors, 2 warnings,
the known `OLD_PLAN.md` pair, 25 notes); `survey check` (matches the tree, exit 0); `doctor --check`
(17 checks passed, exit 0). Line endings of every staged file checked (no CRLF); the banned word:
no hit. The recording commit follows.

### AGENTS
One Opus 5 sous-chef, one session, no sub-delegation. See `V2_AGENT_LOG.md`.

### PROBLEMS
1. **NE-02's abbreviated program does not reproduce** (above). Recorded rather than papered over; P1-09
   is written against the shape that does.
2. **`exec.native` and invariant 45** (above). Found by the existing twin test, not by the new gate.
3. **A flag parsed by the shared option parser is accepted by every command.** `--grants` is
   `authority`'s, and `delulu check x.delulu --grants` exits 0 having ignored it — because
   `refuse_unknown_flags` reads `opts.unknown_flags`, and a flag the shared parser knows is never
   unknown to anyone. **This is pre-existing**, not introduced here: `delulu check x.delulu --diff foo`
   behaves the same way on the pre-fix binary (witnessed). It is the NE-06 shape one level up, it
   affects `--diff`, `--sign`, `--out` and others, and fixing it means a per-command allowlist across
   every subcommand. Left for a ruling rather than widened quietly.
4. **`delulu test examples/greeter` does not resolve the package.** Reaching it as a directory of
   files, `test` loads `src/main.delulu` alone and reports `check-failed` on names another module
   exports. Found while building the reproduction; out of P1's scope; not a new defect.
5. **The `IoErr` detail for a capability-scope miss was NOT built**, by the brief's own condition: it
   needs a payload on `IoErr::NotFound`, which changes every `match` over the sum, so it is a
   language change and not a message. Recorded as `REMAINING_WORK.md` 6.13 with the three options.
6. **The CLI sweep is 28 cases, not 27.** P1-07 split "bare test refuses" into "inside a package"
   (exit 0) and "outside a package refuses" (exit 2), because a single case cannot witness both.

### DECISIONS
No build-order-level decision was taken by the sous-chef. Three judgements inside the brief's grant,
each recorded here for the head chef to confirm or reverse:
1. `required_grants` excludes `exec.native` (invariant 45; asserted on `native_emission` instead).
2. The `drop-val-annotation` repair removes the **whole** annotation, not the `val` keyword, because
   removing the keyword alone leaves the same refusal.
3. The DL0301 → DL0404 cascade (`unknown name` followed by "value of type `'t0` is not callable")
   was **not** fixed. It is the same shape as NE-04 and one edit away — suppress DL0404 when the
   callee's type is an error-born inference variable — but it is a type-checker change the brief did
   not name, and the naive form of it (suppress on any `Type::Var`) would let
   `fn apply[F](f: F, x: Int) { f(x) }` compile, which is a language change. Witnessed:
   `examples/greeter/src/main.delulu` reached as loose files yields DL0301 + DL0404 twice over.

### RECORDED BY THE HEAD CHEF AFTER THE SOUS-CHEF'S REPORT (2026-09-18)
- **The sous-chef was stopped once by a usage limit, then completed.** The harness reported
  `Agent terminated early due to an API error: You've hit your monthly spend limit` (HTTP 429,
  `rate_limit`, request id `req_011Cf9aBC9zQ6XVyDgNVkdUB`, model `claude-opus-5`) while the agent was
  starting P1-08. Nothing on disk was lost: its progress file and every evidence file written before
  the stop were intact. The same agent was resumed and finished the phase with the report above.
  Usage from the harness: 646,313 tokens, 452 tool uses, 7 h 26 min of wall-clock time including the
  stop.
- **Its three questions for the head chef.** (1) Rule on the DL0301 → DL0404 cascade — DECISIONS item 3
  above. (2) Confirm the two judgements taken inside the brief — DECISIONS items 1 and 2 above.
  (3) `run --json` is the one command excluded from the success sweep, because under `--json` the
  standard output belongs to the program itself; this bears on PS-0-02, which adds a `sandbox` object
  to `run --json`, so that phase must decide where the object goes. None is answered yet.
- **The owner paused the work on reading the report** (2026-09-18): no further run starts until the
  owner says so, and the head chef's verification, the commit, the push and the CI read of P1 wait
  with it. P1 is in the working tree, uncommitted (43 files modified, 2 new). A patch of it that
  applies cleanly to `94c6e29` (checked with `git apply --cached --check`) and copies of the two new
  files are in the phase's storage folder, so the work survives even a damaged tree.
- **Where the records live.** Brief, progress file, falsifications, reproductions before and after,
  snapshot diffs, suite and gate outputs, the pre-fix binary, the report (saved verbatim by the head
  chef) and the patch: `D:\nelan\DeluluLang-agent-transcripts\2026-09-17-p1\`. Claude Code also keeps
  each agent's raw transcript on this machine automatically, in the session's `subagents` folder of
  its projects directory; that transcript is not copied into project storage, because it carries the
  agent's private reasoning, which the owner's rule excludes.

### HEAD CHEF VERIFICATION AND RULINGS (2026-09-18, on the owner's word)
The owner said go on 2026-09-18: "finish P1 with a quick check, the three rulings, the commit, the
push and the CI result", and the next phase only on the owner's word. What the head chef checked,
on the tree as the sous-chef left it:
- **The tree is the one the sous-chef verified.** Every source diff under `crates/`, `scripts/`,
  `tests/` and `examples/` (28 files) is byte-identical to the patch saved when the report arrived,
  and the release binary is newer than every changed source file. The head chef's own edits touch
  only this folder and the Survey.
- **Witnesses re-run by the head chef**, on the P1 binary against the pre-P1 binary (both kept in
  the phase's storage folder):

| Witness | Pre-P1 binary | P1 binary |
|---|---|---|
| the 200-deep expression, `check --json` | 145 diagnostics (73 × DL0210, 72 × DL0404) | 1 (DL0210) |
| `explain DL0501 --json` | human text, not JSON | the envelope with `explain: {body, code, disposition, kind, title}`; `--bogus` exits 2 on both |
| `authority kitchen.delulu --grants` | refused as an unknown option, exit 2 | 11 `--grant` flags, exit 0 |
| `test --json` on a file that fails its check | exit 1, 287 bytes of human text on stderr, no `diagnostics` | exit 1, stderr empty, DL1603 with its span and one repair |
| bare `delulu test` in a fresh `delulu new` scaffold | refused, exit 2 | `1 passed, 0 failed`, exit 0; outside a package still refused, exit 2 |
| DL1603 on the NE-02 shape | no repair offered | `drop-val-annotation` (`safe`); its edit applied → `checked clean` |

- **Diff review of the four source files the sous-chef named first.** `parser.rs`: one refusal per
  site and class, the flag clearing when that class's depth returns to zero; `skip_over_deep_operand`
  is one iterative, balanced pass that consumes a token or stops on every iteration; a repeat is
  suppressed only after the file already carries an error, so no program refused before is accepted
  now. `diagnostic.rs`: `with_repair` only ever raises `requires_human` (for an editless repair), and
  it is the only place a repair is pushed onto a diagnostic. `rcap_check.rs`: the `lift_val` and
  `lift_iso` computation of `fresh_composite` is unchanged, `blame` is recorded after it, and
  `explain_val_binding` rewrites only a DL1603 already emitted. `cli.rs`, `success_envelope`: the
  header keys are `delulu_version`, `schema`, `command`, `diagnostics` and `summary`; there is no `ok`
  key a header could overwrite; the report's own keys stay at the top level and its `summary` fields
  win.
- **The snapshot, compared structurally** (all 377 cases parsed, old against new): 58 changed and no
  exit status moved. 49 × `authority --json` additive only; 5 × `check --json` additive only
  (`reason`), 2 of them also `requires_human` false → true; 2 × `check` (human) the marker only;
  2 × the DL0210 case, 145 → 1. **The SNAPSHOT table above counts 6 and 3 in its second and third
  rows; the right figures are 5 and 2.** Its total, 58, is right, and every change is attributable
  to P1-04, P1-05 or P1-08.
- **Found by the head chef: some `grants` and `guard` verbs still print their `--json` success
  without the envelope**, for example `grants list`, `grants tree`, `grants pubkey` and
  `guard pending` (bare JSON from their handlers in `cli.rs`). The sweep excuses both commands in
  writing because each verb needs a running broker, and the report said "every *reachable* success
  emitter wrapped", which is exact. So the acceptance line "every `--json` success emits the
  envelope (gated)" holds for every command the sweep can reach, not for these two. Recorded as
  follow-up P1-F2 rather than fixed here: a change with no test that could fail would not be
  verification.
- **Observed, not a P1 change:** `delulu fix` exits 0 when it applied nothing and errors remain (the
  same on the pre-P1 binary), so a harness must read the report, not the exit status. For the repair
  loop in P4.
- **Rulings on the sous-chef's three questions:** D-V2-19 (the cascade becomes follow-up P1-F1, by
  poison propagation), D-V2-20 (both judgements confirmed, each re-witnessed), D-V2-21 (the `sandbox`
  object travels in a run report the runtime writes to `--report-out <path>`, never on the program's
  standard output; PS-0-02 re-specified). The follow-ups P1-F1 to P1-F4 are listed under P1 in
  `V2_IMPLEMENTATION_ROADMAP.md`.
- **Not re-run by the head chef:** the full suite and clippy. The source is byte-identical to the tree
  the sous-chef's full run passed on (126 binaries, 1,674 passed, 0 failed), and CI runs both on three
  operating systems. Re-run after the head chef's edits and before the commit: the documentation
  gates, the Survey's freshness test, `survey check` and `doctor --check`; their results are in GIT.

### NEXT
PS-0, sandbox truth, probes and the cheap hardenings (`V2_IMPLEMENTATION_ROADMAP.md`), with PS-0-02 as
re-specified by D-V2-21, **only on the owner's word** (2026-09-18: "Do not start next phase until I say
so"). The owner may put first the P1 follow-ups P1-F1 to P1-F4, or P4b (D-V2-18's open offer).
