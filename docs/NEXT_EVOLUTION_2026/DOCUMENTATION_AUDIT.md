# Documentation audit — every markdown file classified, and the proposed moves

**Written:** 2026-09-17. **Status:** classification complete; **no file has been moved**. Moves
are phase P6 of `IMPLEMENTATION_ROADMAP.md` and need the owner's approval of the plan and the
folder name (D-NE-11). Counts come from `git ls-files '*.md'` (155 tracked files at `52eecbe`)
plus the seven files this folder adds. Inbound-reference counts come from the Survey's edge list
(`docs/survey/survey.json`, `links_to` + `references` edges arriving at each `doc:` node) as
regenerated on 2026-09-17; they change as documents change and must be recomputed before a move.

**Rules applied.** Never delete a `.md` (owner's disk-cleanup discipline). Never rewrite a release
snapshot. Never move an entrenched path (`.github/CODEOWNERS`: `docs/design/CONSTITUTION.md`,
`DELULU_CORE.md`, `STABILITY.md`, `SOUNDNESS_AUDIT.md`, `/rfcs/`, `SECURITY.md`,
`/docs/security/`). Historical records keep their pre-fix statements (they are dated). A move is
`git mv` into `docs/archive/<original path relative to docs/>` (root files go to
`docs/archive/root/`), followed by updating every inbound link the Survey lists and regenerating
the map; the Survey's own scanner walks the whole tree, so moved files stay mapped and any broken
link is reported as a discrepancy — the gate that keeps a move honest.

**A constraint found while auditing:** the Survey extracts ruling nodes (`ruling:D<n>`) and
finding nodes (`finding:C<n>`) from the build orders and the campaign records by their headings,
and `CHANGELOG.md` and other documents cite them by number. Moving those files is possible but
is a Survey-visible change with 258 bare-`D<n>` citations relying on the documented default; they
are therefore classified **keep in place** here, not because they are current, but because they
are load-bearing for the map. Revisit only with a scanner change in the same commit.

## Categories

| Code | Category | Treatment |
|---|---|---|
| **A** | Current authoritative (root) | keep; maintain |
| **B** | Current user/agent documentation | keep; maintain |
| **C** | Normative specification | keep in place; change only by RFC / ruling / entrenchment process |
| **D** | Active maintainer or evidence record | keep in place |
| **E** | Generated | never hand-edit; regenerate |
| **F** | Release snapshot | never rewrite; never move |
| **G** | Historical campaign or process record, still cited | keep in place **or** move with link updates — per row |
| **H** | Superseded or duplicate | move to `docs/archive/` with a status header |
| **I** | Starter/pending content (not historical) | keep |
| **J** | Entrenched path | never move |

## Root

| File | Cat | In-refs | Note |
|---|---|---|---|
| `README.md` | A | 1 | front door; P1-01 corrects the plugin sentence |
| `HANDOFF.md` | A | 1 | 1,149 lines; P6 shrinks it to a current briefing with pointers; contains the banned word in two rule lines (pre-public item 1 — owner's) |
| `INSTALL.md` | A | 1 | P5 rewrites the five-minute path |
| `CHANGELOG.md` | A/G | 0 | the ledger; append-only; keep |
| `CONTRIBUTING.md`, `GOVERNANCE.md`, `TRADEMARK.md` | A | 1 each | keep |
| `SECURITY.md` | A/J | 2 | entrenched; keep |

## `docs/` — user and agent documentation

| File | Cat | In-refs | Note |
|---|---|---|---|
| `GETTING_STARTED.md` | B | 5 | P1-10 fixes §6's path shape; P3 removes the four-method box |
| `for-agents.md` | B | 33 | the most-referenced document in the tree; P1 makes its envelope claim true; the skill points here |
| `QUESTIONS.md` | B | 16 | keep |
| `DEPLOYMENT.md` | B | 17 | keep |
| `MATHEMATICS.md` | B/D | 13 | keep |
| `REMAINING_WORK.md` | B | 16 | the single gap list; P1-01 adds NE-01 |
| `REPOSITORY_STRUCTURE.md` | B | 9 | NE-12: its "checked mechanically" claim needs a gate (P1-12); §5 gains this folder |
| `editors.md` | B | 11 | keep |
| `book/THE_DELULULANG_BOOK.md` (+ `samples/`) | B | 4 | Ch. 10 corrected in P1-01, rewritten in P2-06 |

## `docs/design/` — specifications, records, reviews

| File | Cat | In-refs | Proposed treatment |
|---|---|---|---|
| `CONSTITUTION.md` | C/J | 8 | entrenched; §5.15 wording pending (RW 7.10a) |
| `DELULU_CORE.md` | C/J | 8 | entrenched; P7 restates Progress under the entrenched process |
| `STABILITY.md` | C/J | 10 | entrenched |
| `SOUNDNESS_AUDIT.md` | C/J | 8 | entrenched |
| `STAGE1_SPECIFICATION.md` … `STAGE10_SPECIFICATION.md` | C | 1–3 | normative; keep in place |
| `STAGE10_AUTONOMY_ADDENDUM.md`, `STAGE5_GUARD_ADDENDUM.md`, `SYNTAX_MORPH_SPEC.md`, `LOCALIZATION_PLUGIN_GUIDE.md`, `REGISTRY_POLICY.md`, `AI_NATIVE_DESIGN.md`, `VERSION_COEVOLUTION.md`, `THREADED_WASM_DEFERRAL.md`, `SURFACE_ATLAS_PALETTE_ADDENDUM.md` | C | 2–12 | normative or design reference; keep |
| `STAGE6_PLUGINS_GUIDE.md`, `STAGE7_ACTORS_GUIDE.md`, `STAGE8_SURFACE_GUIDE.md` | B | 0–1 | user guides; keep; plugins guide corrected in P1-01 |
| `models/README.md` (+ TLA/Lean/Z3 sources) | D | 1 | evidence; keep |
| `STAGE6_BUILD_ORDER.md`, `STAGE7_BUILD_ORDER.md`, `STAGE8_BUILD_ORDER.md`, `STAGE9_BUILD_ORDER.md`, `STAGE10_BUILD_ORDER.md` | G (map-load-bearing) | 0–6 | rulings `D<n>` are extracted from these; **keep in place** (see constraint above) |
| `HARDENING_CAMPAIGN.md` (235 KB) | G (map-load-bearing) | 10 | findings `C<n>` are extracted from it; README links it; **keep in place** |
| `PROOF_CAMPAIGN.md` | G | 16 | P17/P18 record; heavily cited by `MATHEMATICS.md`/`QUESTIONS.md`; **keep in place** (moving would touch 16 links for no reader benefit) |
| `PRODUCTION_READINESS_2026-08-09.md`, `PRODUCTION_READINESS_2026-08-10.md` | G | 1, 2 | dated campaign records cited by README/HANDOFF; **move** to `docs/archive/design/` and update the 3 links — *or* keep; recommended: move, because README's "continuing hardening effort" paragraph can cite the archive path |
| `PRODUCTION_READINESS_REVIEW.md` | H | 1 | the pre-1.0 readiness pass, superseded by `release/CHECKPOINT-1.0.md`; **move** |
| `P19_ECOSYSTEM_REVIEW.md` | G | 1 | dated review; **move** with its one link |
| `STAGE10_AUTONOMY_HONESTY_REVIEW.md` | G | 1 | dated honesty scrub; **move** |
| `CROSS_PLATFORM_VERIFICATION.md` | D | 17 | the CI run ledger; keep (it is where §9 records each run) |
| `AUTHORITY_GUARD_CAPSTONE.md`, `ROOT_ISSUANCE_TRUST_BOUNDARY.md`, `ENTRENCHED_CHANGE_RECORD.md` | D | 4–7 | current-state audits and the approval log; keep |
| `LANGUAGE_SPECIFICATION.md` | H | 2 | the superseded pre-implementation draft (its own header says so since 2026-08-23); **move** to `docs/archive/design/`, update the 2 links |
| `DeluluLang_PROMPT.md` | G | 1 | the founding commission, kept verbatim; **keep in place** (origin document; the plan cites it) |
| `DeluluLang_Fable_5.1_Master_Prompt.md` | G (commission) | 0 | the 2026-09-17 morning commission, kept where the owner placed it, one banned word redacted in place (D-NE-20; the earlier move under D-NE-12 was reversed by the owner) |
| `DeluluLang_Sandbox_VM_Integrated_Next_Evolution_Prompt.md` | G (commission) | 0 | the 2026-09-17 afternoon commission (sandbox and VM isolation), verbatim |

## `docs/playbooks/`

| Files | Cat | In-refs | Treatment |
|---|---|---|---|
| `README.md`, `STAGE4…STAGE10_PLAYBOOK.md` | G | 0–2 | process records of how stages were built; nothing current depends on them; **move** the whole folder to `docs/archive/playbooks/`, update the 2 links in `REPOSITORY_STRUCTURE.md`/`HANDOFF.md` |

## `docs/reference/` — generated in part

| Files | Cat | Treatment |
|---|---|---|
| `README.md`, `cli.md`, `diagnostics.md`, `grammar.md`, `tokens.md`, `primitives.md`, `coverage.md`, `audit-rules.md`, `semantics-5-1 … 5-16.md` | E/C | generated by `delulu-conform --reference` where marked; never hand-edit those; keep |

## `docs/survey/`

| Files | Cat | Treatment |
|---|---|---|
| `SURVEY.md`, `survey.json`, `DISCREPANCIES.md` | E | generated; regenerate |
| `README.md`, `AUDIT.md`, `REMOVALS.md` | D | hand-written maintainer records; keep |

## `docs/release/`

| Files | Cat | Treatment |
|---|---|---|
| `CHECKPOINT-1.0.md`, `CHECKLIST-1.0.md`, `ANNOUNCEMENT-1.0.md`, `SUPPORT_MATRIX.md` (+ `SBOM-1.0.json`, `PROVENANCE-1.0.json`) | F | release snapshots; never rewrite, never move (release documents are exempt from the freshness scan for this reason) |

## `docs/security/`

| Files | Cat | Treatment |
|---|---|---|
| `DRILL-001.md`, the five `red-team-*` directories and their notes | D/J | entrenched path; evidence; never move |

## `docs/maintenance/`

| Files | Cat | Treatment |
|---|---|---|
| `DISK-CLEANUP-2026-08-04.md`, `DISK-CLEANUP-2026-08-09.md` | G | machine-state records; **move** to `docs/archive/maintenance/` (1 link to update) — or keep; low value either way |

## `docs/lang/`

| Files | Cat | Treatment |
|---|---|---|
| `README.md`, `en-US.md`, `delulu-slang.md`, nine starter packs | I | pending localization content (RW 6.2/6.3), not historical; keep |

## `measurements/`, `rfcs/`, the corpus and example READMEs, the editor extension's documents

| Files | Cat | Treatment |
|---|---|---|
| every `measurements/*/RECORD.md` and `study-*/REPORT.md`, `METHODOLOGY.md` | D | reproducible evidence behind published numbers; keep |
| `rfcs/*` | C/J | entrenched; keep |
| `tests/corpus/tier4-multimodule/README.md` and `examples/plugin_shout/README.md` | B | keep; the plugin README corrected in P1-01 |
| `editors/vscode/README.md`, `CHANGELOG.md` | B | keep |

## This folder (`docs/NEXT_EVOLUTION_2026/`)

| File | Cat |
|---|---|
| `README.md`, `MASTER_PLAN.md`, `RESEARCH.md`, `VERIFICATION_FINDINGS.md`, `IMPLEMENTATION_ROADMAP.md`, `DECISION_LOG.md`, `EXECUTION_LOG.md`, `DOCUMENTATION_AUDIT.md`, `OWNER_COMMISSION.md` | D (active planning record) |
| `SANDBOX_RESEARCH.md`, `SANDBOX_ARCHITECTURE.md`, `SANDBOX_THREAT_MODEL.md`, `SANDBOX_TEST_PLAN.md`, `SANDBOX_IMPLEMENTATION_PLAN.md` | D (active planning record; supporting documents of the master plan) |
| `agent-notes/RED-TEAM-SANDBOX-SURFACES-opus5.md`, `agent-notes/HOST-CAPABILITY-FACTS-sonnet5.md` | D (evidence records; never rewritten — the red-team precedent; the Sonnet file carries one marked head-chef annotation) |

## Proposed move manifest (to be executed in P6, each row re-verified first)

The destination column is written as `archive/<subfolder>` and means the mirror of the original
path under the archive folder inside `docs/` (the folder does not exist yet, which is why the
column does not spell a full path the Survey would look for).

| Original path | Destination (same file name) | Reason | Category | Inbound refs (2026-09-17) |
|---|---|---|---|---|
| `docs/design/LANGUAGE_SPECIFICATION.md` | archive/design | superseded pre-implementation draft | H | 2 (`REPOSITORY_STRUCTURE.md`, `CONSTITUTION.md`'s pointer) |
| `docs/design/PRODUCTION_READINESS_REVIEW.md` | archive/design | pre-1.0 pass superseded by the checkpoint | H | 1 |
| `docs/design/PRODUCTION_READINESS_2026-08-09.md` | archive/design | dated campaign record | G | 1 |
| `docs/design/PRODUCTION_READINESS_2026-08-10.md` | archive/design | dated campaign record | G | 2 |
| `docs/design/P19_ECOSYSTEM_REVIEW.md` | archive/design | dated review | G | 1 |
| `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` | archive/design | dated review | G | 1 |
| `docs/playbooks/` (8 files) | archive/playbooks | process records | G | 2 total |
| `docs/maintenance/` (2 files) | archive/maintenance | machine-state records | G | 1 total |

Fourteen files moved, none deleted, ten links updated, one `archive/README.md` written with this
table plus the commit hash of the move. Every other document stays where it is, for the reason in
its row. **HANDOFF.md** is not moved; it is shrunk (P6), and its historical sections (§3 campaigns,
§8's closed rows, §9's old measurements, §13's pre-CI narrative) are the parts that move into
pointers.
