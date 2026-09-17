# V1 historical archive

Documents of the **V1 era** that are historical, superseded, or process records. They were moved
here in **phase V2-0 of DeluluLang V2, on 2026-09-17**, so that the active tree has one source of
truth and a reader opening any document outside this folder is reading something maintained.

Nothing was deleted, renamed, or rewritten on the way in. Thirty-two files moved; every one is
listed below.

## The convention

**This folder mirrors the paths it came from, relative to `docs/`.** A path that a record here cites
as `docs/<x>` and that is no longer there is at `docs/archive/v1/<x>` — one substitution, `docs/` to
`docs/archive/v1/`, reads every such citation in the folder. Nothing was renamed on the way in, so
everything after the last `/` is unchanged.

The records were left citing the old paths on purpose. A historical record that is edited to match a
later tree stops being a record of what was true when it was written, and this project has refused
that trade before — the same reason `CHANGELOG.md` and the build orders keep their original counts.
The Survey knows the convention (`delulu-survey`'s `ARCHIVE_ROOT`): it follows such a citation to the
archived file, draws the edge, and reports a **note**, not a warning. A citation of something that
never moved and is simply gone is still reported as missing, which is what makes the rule a rule.

**Nothing in this folder is maintained, and nothing in it is rewritten. Nothing in it is deleted.**
Its numbers, statuses and file paths describe the moment each document was written. For what is true
now, follow the successor named with each group below.

## `design/` — superseded specifications and dated campaign records

Successors: **`docs/release/CHECKPOINT-1.0.md`** for the honest release status;
**`docs/REMAINING_WORK.md`** and **`HANDOFF.md`** for what the campaigns left open;
**`docs/design/CONSTITUTION.md`** and the `STAGE<N>_SPECIFICATION.md` files for what the language
normatively is.

| File | What it is |
|---|---|
| `LANGUAGE_SPECIFICATION.md` | *The Language Specification & Constitution* — the pre-implementation vision document, written before Stage 1. Its own status header says it is superseded, by `CONSTITUTION.md` and the ten stage specifications. Four of its decisions did not survive contact with the implementation. |
| `PRODUCTION_READINESS_REVIEW.md` | *Production-readiness review* — the pass commissioned 2026-08-02, *"a production readiness and architecture stabilization phase, not feature chasing"*. Superseded as a statement of readiness by `docs/release/CHECKPOINT-1.0.md`. |
| `PRODUCTION_READINESS_2026-08-09.md` | *Production-readiness verification — overnight pass, 2026-08-09.* The dated record of a phase-by-phase pass: six security defects, each witnessed against the pre-fix code. |
| `PRODUCTION_READINESS_2026-08-10.md` | *Production-readiness hardening — overnight campaign, 2026-08-10.* The dated record of the P22 containment campaign: thirteen defects fixed, one residual documented, Windows and Linux only. |
| `P19_ECOSYSTEM_REVIEW.md` | *P19 — the ecosystem campaign, and an independent production review*, 2026-08-07. Seven readings of the same tree, treating compiler, CLI, runtime, broker, language server, extension and docs as one product. |
| `STAGE10_AUTONOMY_HONESTY_REVIEW.md` | *Line-by-line honesty review — the autonomy addendum*, 2026-07-20. Stage 10 phase 10g, acceptance criterion 10: every boundary claim in `STAGE10_AUTONOMY_ADDENDUM.md` held against what 10a–10g actually built. Four findings, all fixed. |

## `playbooks/` — how each stage was actually built

Process records. Each was written to guide one stage's construction and is a record of that method,
not a description of the code today. Successors: the stage specifications and build orders in
`docs/design/` for what the code must do, `HANDOFF.md` for how to work in the repository now.

| File | What it is |
|---|---|
| `README.md` | *DeluluLang Implementation Playbooks* — the index of the seven below, with the per-stage table. |
| `STAGE4_PLAYBOOK.md` | Stage 4 — *"Foreign"*: C FFI and embedded Python. |
| `STAGE5_PLAYBOOK.md` | Stage 5 — *"Custody"*: broker, grant tree, revocation, microVM. |
| `STAGE6_PLAYBOOK.md` | Stage 6 — *"Live"*: runtime plugins, DIR, `.dpx`. |
| `STAGE7_PLAYBOOK.md` | Stage 7 — *"Concurrent"*: actors, reference capabilities, async. |
| `STAGE8_PLAYBOOK.md` | Stage 8 — *"Surface"*: LSP, fmt, test runner, localization, welcome. |
| `STAGE9_PLAYBOOK.md` | Stage 9 — *"Delulu"*: the v1.0 release — freeze, prove, govern, ship. |
| `STAGE10_PLAYBOOK.md` | Stage 10 — *"Industrial"*: JIT policy, autonomy, compute, PQC, cloud, LTS. |

## `maintenance/` — dated machine-state records

Records of what was on one disk on one day, and of what was reclaimed. They are evidence for the
disk-cleanup discipline, not instructions. Successor: `HANDOFF.md` for the discipline itself.

| File | What it is |
|---|---|
| `DISK-CLEANUP-2026-08-04.md` | *Disk cleanup — 2026-08-04.* What was removed, what was kept, and the verification behind each decision. |
| `DISK-CLEANUP-2026-08-09.md` | *Disk cleanup — 2026-08-09.* The second pass, including the WSL reclaim done only against content hashes. |

## `NEXT_EVOLUTION_2026/` — the two 2026-09-17 planning passes that V2 executes

The **Next Evolution reassessment** and the **sandbox / VM isolation research pass**, both completed
on 2026-09-17 and both stopped at the owner's approval gate with nothing implemented. DeluluLang V2
is the execution of this planning material, so the plan is history and the execution is not: **the
active plan now lives in `docs/DELULULANG_V2/`.**

Sixteen files. `agent-notes/` holds the two sous-chef evidence records, which are never rewritten.

| File | What it is |
|---|---|
| `README.md` | *Next Evolution 2026 — the plan folder.* The index and reading order for the fifteen below. |
| `MASTER_PLAN.md` | *DeluluLang — Next Evolution 2026: the master plan.* The reassessment against the AI-first goal, phases P1–P8 and the sandbox phases PS-0…PS-D, and §9/§12.2's open questions for the owner. |
| `RESEARCH.md` | *Research record* — what the field is doing, and what of it belongs in DeluluLang. |
| `VERIFICATION_FINDINGS.md` | *Verification findings — the real binary, driven by hand, 2026-09-17.* The twenty-two `NE-nn` findings, each reproduced against the shipped binary. |
| `IMPLEMENTATION_ROADMAP.md` | *Implementation roadmap* — phases, tasks, dependencies, verification. |
| `DECISION_LOG.md` | *Decision log — Next Evolution 2026.* The `D-NE-nn` records, including the two that carry standing owner rulings. |
| `EXECUTION_LOG.md` | *Execution log — Next Evolution 2026.* What each pass did, with commits, pushes and CI run ids. |
| `DOCUMENTATION_AUDIT.md` | *Documentation audit* — every markdown file classified, and the proposed moves. **This audit is the source of the move table that V2-0 executed**, which is why it is itself in the archive. |
| `OWNER_COMMISSION.md` | *The owner's commissions — where they live.* A pointer record; the commission files themselves stay in `docs/design/` where the owner placed them. |
| `SANDBOX_RESEARCH.md` | *Sandbox and VM research* — what the field runs untrusted code in, and what it costs. |
| `SANDBOX_ARCHITECTURE.md` | *Sandbox architecture* — the execution-isolation layer under Delulu Authority. |
| `SANDBOX_THREAT_MODEL.md` | *Sandbox threat model* — boundaries, trusted computing base, assumptions, evidence. |
| `SANDBOX_TEST_PLAN.md` | *Sandbox test plan* — generated adversaries, escape matrices, and the gates that must be able to fail. |
| `SANDBOX_IMPLEMENTATION_PLAN.md` | *Sandbox implementation plan* — phases PS-0 to PS-D, integrated into the main roadmap. |
| `agent-notes/RED-TEAM-SANDBOX-SURFACES-opus5.md` | *Red-team note* — execution attack surfaces and the adversarial sandbox test matrix. An evidence record, never rewritten. |
| `agent-notes/HOST-CAPABILITY-FACTS-sonnet5.md` | *Host-capability facts for sandbox/VM isolation design.* An evidence record, never rewritten; carries one marked head-chef annotation. |

## Where the move is recorded

`docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md` — every row, why it moved, which inbound links changed,
and the documents that were considered and deliberately kept in place.
