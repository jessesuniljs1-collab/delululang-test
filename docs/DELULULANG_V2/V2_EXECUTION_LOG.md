# V2 execution log

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
Recorded in the edit that follows the push (the phase commit's hash, the push, the run id and its
result are not known until then).

### GIT
Recorded in the edit that follows the push (the phase commit's hash, the push, the run id and its
result are not known until then).

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
