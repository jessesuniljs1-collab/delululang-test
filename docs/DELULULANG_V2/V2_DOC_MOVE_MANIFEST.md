# V2 documentation move manifest

What this records: the thirty-two documents that **phase V2-0 of DeluluLang V2** moved into the V1
historical archive on **2026-09-17**, the reason for each, and the inbound references that were
rewritten. It also records what was considered and deliberately **kept in place**, because a move
that is not made is a decision too, and the next person to propose it deserves the reason.

Nothing was deleted and nothing was renamed. The classification comes from
`docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md`, re-verified against the tree before
each row was executed.

## The mirror convention

The archive **mirrors the original paths relative to `docs/`**: what was `docs/<x>` is
`docs/archive/v1/<x>`. The "Original path" column below is written **as it was**, which is also how
the historical records still write it — those citations were left as written, because editing a
record to match a later tree stops it being a record.

The Survey understands the convention (`delulu-survey`'s `ARCHIVE_ROOT`): a prose or comment citation
of a pre-move path resolves to the archived file, draws the edge, and is reported as a **note**
(`cites-archived-path` / `comment-cites-archived-path`) rather than a warning. **That is intended,
not drift.** An explicit Markdown link is still an error if it dangles — a link is navigation a
reader clicks, and understanding why it fails does not make it work — so every explicit link was
rewritten instead.

## Moved

| Original path | New path | Why moved | Class | Inbound links changed | Authoritative anywhere? |
|---|---|---|---|---|---|
| `docs/design/LANGUAGE_SPECIFICATION.md` | `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` | Pre-implementation vision draft; its own status header says it is superseded by `CONSTITUTION.md` and the stage specifications. Four of its decisions did not survive contact. | superseded | yes — `docs/REMAINING_WORK.md`, `docs/REPOSITORY_STRUCTURE.md`, `crates/delulu/tests/governance.rs` (`RESTATED_IN`); two outbound links inside the file itself re-pointed | no |
| `docs/design/PRODUCTION_READINESS_REVIEW.md` | `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` | The 2026-08-02 pre-1.0 readiness pass, superseded as a statement of readiness by `docs/release/CHECKPOINT-1.0.md`. | superseded | yes — `crates/delulu-survey/src/codeowners.rs:16` (module comment); one outbound link inside the file itself re-pointed | no |
| `docs/design/PRODUCTION_READINESS_2026-08-09.md` | `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md` | Dated record of one overnight pass. | campaign record | yes — `README.md` (link) | no |
| `docs/design/PRODUCTION_READINESS_2026-08-10.md` | `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` | Dated record of the P22 containment campaign. | campaign record | yes — `README.md` (link). `CHANGELOG.md:173` cites it and was **not** edited: a changelog entry records what was true when written. | no |
| `docs/design/P19_ECOSYSTEM_REVIEW.md` | `docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md` | Dated review, 2026-08-07. | campaign record | yes — `HANDOFF.md`, `crates/delulu/tests/evidence_claims.rs` (`HISTORICAL`) | no |
| `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` | `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` | Dated review, 2026-07-20 (Stage 10 phase 10g). | campaign record | none. `docs/design/STAGE10_BUILD_ORDER.md:1644` cites it and was **not** edited — a build order is a historical record. | no |
| `docs/playbooks/` — `README.md`, `STAGE4_PLAYBOOK.md`, `STAGE5_PLAYBOOK.md`, `STAGE6_PLAYBOOK.md`, `STAGE7_PLAYBOOK.md`, `STAGE8_PLAYBOOK.md`, `STAGE9_PLAYBOOK.md`, `STAGE10_PLAYBOOK.md` (8 files) | `docs/archive/v1/playbooks/…` | How each stage was actually built. A record of one construction method, not a description of the code today. | process record | yes — `docs/REPOSITORY_STRUCTURE.md`, `docs/design/LOCALIZATION_PLUGIN_GUIDE.md`. The five `STAGE<N>_BUILD_ORDER.md` precedence lines and `SURFACE_ATLAS_PALETTE_ADDENDUM.md` cite them and were **not** edited. | no |
| `docs/maintenance/` — `DISK-CLEANUP-2026-08-04.md`, `DISK-CLEANUP-2026-08-09.md` (2 files) | `docs/archive/v1/maintenance/…` | Dated records of what was on one disk on one day. | process record | yes — `HANDOFF.md`, `docs/REMAINING_WORK.md` (×2). One link between the two files kept its spelling: they moved together. | no |
| `docs/NEXT_EVOLUTION_2026/` — `README.md`, `MASTER_PLAN.md`, `RESEARCH.md`, `VERIFICATION_FINDINGS.md`, `IMPLEMENTATION_ROADMAP.md`, `DECISION_LOG.md`, `EXECUTION_LOG.md`, `DOCUMENTATION_AUDIT.md`, `OWNER_COMMISSION.md`, the five `SANDBOX_*.md`, and `agent-notes/` (2 files) — 16 files | `docs/archive/v1/NEXT_EVOLUTION_2026/…` | The two 2026-09-17 planning passes. V2 **is** the execution of this material, so the plan is history and the execution is not. | historical | yes — `HANDOFF.md` (×3), `docs/design/CROSS_PLATFORM_VERIFICATION.md`, `.github/workflows/host-capability-probe.yml` (a YAML comment). `docs/design/DeluluLang_Fable_5.1_Master_Prompt.md:1` cites `DECISION_LOG.md` and was **not** edited — it is an owner commission kept verbatim. The 30 links between files in this folder kept their spelling: they moved together. | the **active** plan is `docs/DELULULANG_V2/` |

**32 files. 5 explicit links rewritten in 3 files. 21 root-relative prose citations rewritten in 9
files. 0 files deleted.**

## Considered and kept in place

| Path | Why it stays |
|---|---|
| `docs/design/STAGE<N>_BUILD_ORDER.md` (all) and `docs/design/HARDENING_CAMPAIGN.md` | The Survey **extracts rulings and findings from them**: `STAGE<N>_BUILD_ORDER.md` is the namespace that allocates `S<N>-D<n>`, and `HARDENING_CAMPAIGN.md` allocates the `C<n>` findings. **258 bare `D<n>` citations** across the repository rely on the documented default namespace. Moving them is a Survey-visible change to how every one of those citations resolves. |
| `docs/design/PROOF_CAMPAIGN.md` | A maintained evidence record with **16 inbound references**, heavily cited by `docs/MATHEMATICS.md` and `docs/QUESTIONS.md`. Moving it touches sixteen links for no reader benefit. |
| `docs/design/CROSS_PLATFORM_VERIFICATION.md` | The **live CI ledger** — §9 is appended to on every run that is read and recorded. A live document is not an archive candidate. |
| `docs/design/DeluluLang_PROMPT.md`, `DeluluLang_Fable_5.1_Master_Prompt.md`, `DeluluLang_Sandbox_VM_Integrated_Next_Evolution_Prompt.md`, `DeluluLang_V2_Execution_Master_Prompt.md` | The owner's commissions, **kept verbatim where the owner placed them**. Not this phase's to move, edit, or re-path. |
| `docs/release/*` — `CHECKPOINT-1.0.md`, `CHECKLIST-1.0.md`, `ANNOUNCEMENT-1.0.md`, `SUPPORT_MATRIX.md`, `SBOM-1.0.json`, `PROVENANCE-1.0.json` | Release snapshots: never rewritten, never moved. They are already exempt from the freshness scan for exactly this reason, so archiving them would add nothing and would break the paths a published release names. |
| Entrenched paths — `docs/design/CONSTITUTION.md`, `DELULU_CORE.md`, `STABILITY.md`, `SOUNDNESS_AUDIT.md`, `SECURITY.md`, `rfcs/`, `crates/delulu-check/tests/laundering.rs`, `crates/delulu-conform/`, `tests/conformance/witnesses.toml` | `.github/CODEOWNERS` marks them as requiring the project lead specifically. Constitution §10 and invariant 44 require an entrenchment analysis before any of them moves; V2-0 is a mechanical migration and does not carry one. |
| `docs/security/` | Entrenched path, and red-team evidence. Never moved. |
| `docs/survey/AUDIT.md`, `docs/survey/REMOVALS.md` | Hand-written maintainer records that are still consulted. (`SURVEY.md`, `DISCREPANCIES.md` and `survey.json` are generated and are regenerated, not moved.) |
| `docs/lang/` — `README.md`, `en-US.md`, `delulu-slang.md`, the nine starter packs | **Pending content, not historical**: the localization work is open in `docs/REMAINING_WORK.md` 6.2/6.3. Archiving work that has not happened yet would say the opposite of what is true. |
| `measurements/` — every `RECORD.md`, `study-*/REPORT.md`, `METHODOLOGY.md` | Reproducible evidence behind published numbers. Still cited as evidence, so still live. |
| `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md` | A stage close-out that the Survey and the addendum's own gates still read. |
| `HANDOFF.md` | Not moved — **shrunk**. Its historical sections become pointers; that is a later phase's work, not a move. |

## Questions raised by the migration, and how they were resolved

The four decisions the sous-chef stopped on rather than taking itself, each settled by the head chef
before the phase closed:

1. **The V2 commission was untracked, and the regenerated map depended on it.** The Survey walks the
   working tree, so `docs/survey/` carried a node, an edge and seven warnings for
   `docs/design/DeluluLang_V2_Execution_Master_Prompt.md`, a file a fresh clone would not have — the
   clean-checkout hazard of 2026-09-14, which turns every CI runner red while the map is locally
   correct. **Resolved: the commission is committed in the V2-0 commit**, so the map is a function of
   the tree again. Its two `OLD_PLAN.md` placeholder citations stay as warnings until the owner
   decides; they are placeholders in the commission's own text, and the commission is kept verbatim.
2. **`docs/REPOSITORY_STRUCTURE.md` §3 is a historical record inside an active document** — the move
   manifest of the 2026-07-05 repository init, whose right-hand column records where *that* move put
   each file. The blanket citation rewrite had changed its `LANGUAGE_SPECIFICATION.md` row to the
   archive path, which would have made a July record claim a September destination. **Resolved: the
   row is kept as written**, and the Survey reports it as a `cites-archived-path` note — which is
   exactly what the mirror rule is for.
3. **Section numbering in `docs/REPOSITORY_STRUCTURE.md` §5.** Removing the playbooks section left
   twelve sections, not thirteen. **Resolved: numbered contiguously** — §5.10 `docs/archive/v1/`,
   §5.11 `docs/DELULULANG_V2/`, §5.12 generated documents — so the document has no missing number.
4. **The archive README's worked example** named a real pre-move path and so reported a note against
   itself. **Resolved: reworded to the placeholder form** `docs/<x>` → `docs/archive/v1/<x>`, the
   same wording as the `ARCHIVE_ROOT` doc comment.

Executed in phase V2-0 on 2026-09-17; the commit hash is in `V2_EXECUTION_LOG.md`.
