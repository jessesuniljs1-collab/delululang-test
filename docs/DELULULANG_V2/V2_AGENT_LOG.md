# V2 agent log

What each sous-chef agent was asked, did, found and failed — conclusions, evidence and outputs only.
No private reasoning or chain-of-thought is stored here or anywhere (owner's rule, 2026-09-17).
Briefs, progress files, scripts and reports live in project storage beside the repository:
`D:\nelan\DeluluLang-agent-transcripts\<date>-<phase>\`. Every claim an agent makes is verified by
the head chef against the current binary or source before it is used; the verification is recorded
beside the claim.

**Standing rule (owner, commission §5–§6, §30):** Opus 5 is the main execution sous-chef and does most
implementation, investigation, tests, refactors, security review, adversarial tests, documentation
migration, verification and repository cleanup; one strong agent at a time, another only for a real
independent reason; every brief carries the exact objective, files, constraints, security and
verification requirements and expected outputs; an agent that meets an architectural or security
decision asks the head chef and never invents one; an agent never changes Authority or Guard
semantics, weakens a sandbox guarantee, changes an owner decision, fabricates a test or CI result,
deletes a historical document, destroys audit evidence, rewrites pushed history, grants itself
authority, disables its own security policy or uses break-glass. On a session or usage limit an agent
is stopped safely, its completed work and findings saved, the interruption recorded, nothing lost.

---

## V2-0 — 2026-09-17 — Opus 5, the documentation migration

- **Brief:** `D:\nelan\DeluluLang-agent-transcripts\2026-09-17-v2-0\BRIEF-migration-opus5.md` —
  32 `git mv` moves into `docs/archive/v1/`, link and citation rewriting under fixed rules, the
  Survey's archive-mirror rule with unit tests and a falsification, two test path updates, the archive
  README, the manifest draft, the accounting edits in `docs/REPOSITORY_STRUCTURE.md`, seven named
  verification commands; no commit, no push, no deletion, no source change beyond the named ones.
- **Outcome — pass 1 (the migration).** 274,080 tokens, 138 tool uses, 29 min 33 s. Executed all six
  objectives: 32 `git mv` moves (`git status` 30 `R` + 2 `RM`, 0 deletions; the staged diff is
  *32 files changed, 0 insertions, 0 deletions*, so the moves are pure renames); 5 explicit links
  rewritten in 3 files; 21 root-relative prose citations rewritten in 9 files, path strings only;
  the Survey's archive-mirror rule in five source files with 9 new unit tests; the two hardcoded test
  paths; the archive README and the manifest draft; the accounting edits in
  `docs/REPOSITORY_STRUCTURE.md`.
- **Falsified, not assumed.** Removing the existence gate from `PathIndex::archived()` made the
  agent's own tests report 30 passed / **3 failed** — the three falsification tests — and the source
  was then restored and re-verified green. A rewritten link was broken back to its pre-move target in
  place and the Survey reported `broken-link` at `README.md:47`, then it was restored. Before relying
  on `docs/design/*.md` staying intact, the agent checked
  `delulu-conform::every_grammar_production_is_defined_in_a_normative_specification` (a
  **non-recursive** scan of that folder): exactly one production name, `point`, was defined only in a
  moved file, and it is in neither `GRAMMAR_PRODUCTIONS` nor `NORMATIVE_NAME`; the test then ran and
  passed.
- **Stopped rather than decided, four times** — all four resolved by the head chef and recorded in
  `V2_DECISION_LOG.md` and this log's V2-0 entry: the committed map depending on the untracked owner
  prompt; `docs/REPOSITORY_STRUCTURE.md` §3 being a historical record inside an active document; the
  §5 section numbering; the archive README's worked example. It also self-reported and fixed two
  warnings it had introduced itself in the `ARCHIVE_ROOT` doc comment.
- **Outcome — pass 2 (finishing the phase).** Brief
  `D:\nelan\DeluluLang-agent-transcripts\2026-09-17-v2-0\BRIEF-finish-opus5.md`: placed the nine V2
  documents (copied byte-identical, then edited in place only), reworded the two citations of paths
  that do not exist yet, filled this log and the execution log, set the phase-status row, applied the
  `HANDOFF.md` and `README.md` pointer texts, finished the manifest, and re-verified with the Survey
  regenerated as the last edit.
- **Outputs, in project storage** (`D:\nelan\DeluluLang-agent-transcripts\2026-09-17-v2-0\`):
  `PROGRESS-opus5.md` (state across both passes), `migrate.py` (the re-runnable migration:
  `moves` / `links` / `prose` / `all`, each idempotent, `--dry` to preview),
  `REPORT-migration-opus5.md` (the pass-1 report), `BRIEF-migration-opus5.md` and
  `BRIEF-finish-opus5.md` (the two briefs).
- **Usage, both passes (from the harness):** 338,757 tokens, 192 tool uses, 41 min in total; pass 2
  alone about 64,700 tokens, 54 tool uses, 11 min. The head chef verified the tree before the phase
  commit `e48f9c3` (renames, deletions, Survey freshness, doctor, the banned word, the pointer edits,
  the resolver's existence gate) and saved both reports beside the briefs.

---

## P1 — machine-contract truth — Opus 5 sous-chef, 2026-09-18

- **Asked.** Brief `D:\nelan\DeluluLang-agent-transcripts\2026-09-17-p1\BRIEF-P1-opus5.md`: execute
  the thirteen P1 tasks (P1-11 deferred by D-V2-17, P1-13 documentation only); reproduce all fourteen
  NE findings on the current binary **before** changing anything; witness every fix against a copy of
  the pre-fix binary; falsify every new test and write the exact edit down; regenerate the
  core-invariance snapshot only deliberately and attribute every changed line; regenerate the Survey
  as the last edit; commit nothing, push nothing, delete nothing, touch no never-edit path.
- **Did.** Twelve tasks done, one deferred. `repro-before.txt` / `repro-after.txt` hold the fourteen
  findings driven by hand on both binaries. The gates it wrote: the success half of the `--json`
  envelope (33 invocations + a completeness gate that reads its own table + the DRILL-001 converse),
  `explain --json` for all three registries, the one-diagnostic-per-overflow-site gate over all four
  nesting classes, the repair-registry gate reading the `Repair` literals out of the source, the LSP
  `isPreferred` rule, bare `delulu test`, the grant-grammar gate reading `Grants::add` out of the
  runtime, the guide-corpus read/write gate with its negative control, and the
  `REPOSITORY_STRUCTURE.md` §5 accounting gate with a permanent mutant. **Eleven falsifications
  performed and restored**, each with the exact edit, in `V2_EXECUTION_LOG.md` entry P1.
- **Found beyond the findings.** `secrets list --json` and `secrets set --json` accepted `--json` and
  ignored it — the NE-06 shape on a security surface, found by the new sweep rather than by the
  finding list. `measurements/METHODOLOGY.md` was named nowhere in §5, found by the new accounting
  gate. Nine further `--json` success paths beyond NE-05's eight were non-conforming.
- **Reported a finding against itself.** NE-02's abbreviated program **does not reproduce** — a
  record or sum of only-`val` components defaults to `val`, so the guide-shaped literal checks clean.
  The agent recorded that, found the shape that does reproduce, and wrote P1-09 against it rather
  than against the description.
- **Nearly broke a semantic invariant, and said so.** Its first `required_grants` derived
  `exec.native` from a `@jit` hint, which made the hinted and unhinted twins' authority reports
  differ — invariant 45. `attributes_cli::invariant45_hints_change_nothing_observable` caught it; the
  grant was removed from the derived list and is asserted on `native_emission` instead.
- **Stopped rather than decided, once.** The DL0301 → DL0404 cascade is the same shape as NE-04 and
  one edit from fixed, but the obvious form of that edit (suppress `DL0404` on any type variable)
  would let `fn apply[F](f: F, x: Int) { f(x) }` compile — a language change. Witnessed, recorded in
  the execution log's DECISIONS, and left for the head chef.
- **Outputs, in project storage** (`D:\nelan\DeluluLang-agent-transcripts\2026-09-17-p1\`):
  `PROGRESS-P1-opus5.md`, `repro.sh`, `repro-before.txt`, `repro-after.txt`,
  `snapshot-diff-P1.txt`, `snapshot-diff-P1-second.txt`, `suite-2.txt`, `clippy-1.txt`,
  `cli-sweep-2.txt`, `doctor.txt`, `conform-coverage.txt`, `conform-ref.txt`, `delulu-pre-p1.exe`,
  `probes/`, `patches/`.
- **Interrupted once by a usage limit, then completed.** Stopped by the harness while starting
  P1-08 (`You've hit your monthly spend limit`, HTTP 429, request `req_011Cf9aBC9zQ6XVyDgNVkdUB`).
  Nothing on disk was lost; the same agent was resumed and completed the phase. Usage from the
  harness: 646,313 tokens, 452 tool uses, 7 h 26 min of wall-clock time including the stop.
- **Asked the head chef three questions** (execution log entry P1, "Recorded by the head chef"): the
  DL0301 → DL0404 cascade; confirmation of two judgements; where PS-0-02's `sandbox` object goes
  when the standard output of `run --json` belongs to the program. Unanswered while the owner's
  pause holds.
- **Saved by the head chef:** the report verbatim (`REPORT-P1-opus5.md`) and a patch of the
  uncommitted P1 tree with copies of its two new files, in the phase's storage folder.
- **Verified by the head chef (2026-09-18, on the owner's word).** Every claim the head chef
  re-witnessed on the binary, against the pre-P1 binary, held. The snapshot's 58 changed cases are
  all attributable; its table's second and third rows say 6 and 3 where the structural comparison
  gives 5 and 2. The report's "every *reachable* success emitter wrapped" was exact: the `grants`
  and `guard` successes, which only a running broker reaches, are still unwrapped (follow-up
  P1-F2). Its three questions were ruled in D-V2-19 to D-V2-21.
