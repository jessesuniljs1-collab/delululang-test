# Survey discrepancies

**Generated — do not edit by hand.** Regenerate with `cargo run -p delulu-survey -- build`.

Each entry is a place where two parts of this repository disagree, or where something points
at what is not there. Every one names the file and line that produced it. Entries are not
graded by how bad they look — an `error` is a statement that contradicts the tree, a
`warning` is true today and drifting, a `note` is for a human to judge.

| Severity | Class | Count |
|---|---|---:|
| warning | `prose-cites-missing-path` | 5 |
| warning | `stale-count` | 9 |
| note | `c-token-not-a-campaign-finding` | 1 |
| note | `cites-archived-path` | 31 |
| note | `code-cited-but-not-allocated` | 1 |
| note | `dependency-never-used-in-source` | 3 |
| note | `ruling-cited-without-its-stage` | 1 |
| note | `test-count-quoted-but-unverifiable` | 1 |

## warning — `prose-cites-missing-path` (5)

**What to do:** correct the path, or say plainly that it no longer exists

- `docs/assistant-memory/delulu-p21-crossaccount.md:61` — prose names `tests/lib.rs`, which is nowhere in the tree
- `docs/assistant-memory/delulu-survey-map.md:64` — prose names `crates/.../ty.rs`, which is nowhere in the tree
- `docs/assistant-memory/delululang-project.md:309` — prose names `docs/playbooks/README+STAGE4..10_PLAYBOOK`, which is nowhere in the tree
- `docs/design/DeluluLang_V2_Execution_Master_Prompt.md:142` — prose names `docs/design/OLD_PLAN.md`, which is nowhere in the tree
- `docs/design/DeluluLang_V2_Execution_Master_Prompt.md:145` — prose names `docs/archive/v1/design/OLD_PLAN.md`, which is nowhere in the tree

## warning — `stale-count` (9)

**What to do:** update the number, or say plainly that it is a snapshot of a past moment

- `docs/assistant-memory/delulu-hardening-campaign.md:183` — says 12 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-hardening-campaign.md:1201` — says 13 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-hardening-campaign.md:1203` — says 12 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-hardening-campaign.md:1252` — says 13 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-hardening-campaign.md:1254` — says 12 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-licensing-intent.md:15` — says 12 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-p18-uncertainty.md:81` — says 7 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-survey-map.md:53` — says 12 shipped language crates; the tree has 9
- `docs/assistant-memory/delulu-survey-map.md:97` — says 12 shipped language crates; the tree has 9

## note — `c-token-not-a-campaign-finding` (1)

**What to do:** no action if these are C-language references; otherwise the ledger is missing a row

- `docs` — these `C<n>` tokens are not campaign findings and were not linked: C0, C99

## note — `cites-archived-path` (31)

**What to do:** a current document should cite the archived path

- `CHANGELOG.md:893` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:30` — prose names `docs/design/LANGUAGE_SPECIFICATION.md`, which moved to `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:31` — prose names `docs/design/PRODUCTION_READINESS_REVIEW.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:32` — prose names `docs/design/PRODUCTION_READINESS_2026-08-09.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:33` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:34` — prose names `docs/design/P19_ECOSYSTEM_REVIEW.md`, which moved to `docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:35` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/REPOSITORY_STRUCTURE.md:472` — prose names `docs/design/LANGUAGE_SPECIFICATION.md`, which moved to `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:160` — prose names `docs/design/LANGUAGE_SPECIFICATION.md`, which moved to `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:161` — prose names `docs/design/PRODUCTION_READINESS_REVIEW.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:162` — prose names `docs/design/PRODUCTION_READINESS_2026-08-09.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:163` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:164` — prose names `docs/design/P19_ECOSYSTEM_REVIEW.md`, which moved to `docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:165` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/agent-usage-rule-2026-09-17.md:74` — prose names `docs/NEXT_EVOLUTION_2026/EXECUTION_LOG.md`, which moved to `docs/archive/v1/NEXT_EVOLUTION_2026/EXECUTION_LOG.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delulu-disk-cleanup-discipline.md:43` — prose names `docs/maintenance/DISK-CLEANUP-2026-08-09.md`, which moved to `docs/archive/v1/maintenance/DISK-CLEANUP-2026-08-09.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delulu-hardening-campaign.md:1158` — prose names `docs/design/PRODUCTION_READINESS_REVIEW.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delulu-p22-containment-campaign.md:296` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delulu-production-readiness-2026-08-09.md:16` — prose names `docs/design/PRODUCTION_READINESS_2026-08-09.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delululang-project.md:3` — prose names `docs/playbooks/STAGE5_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE5_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delululang-project.md:583` — prose names `docs/playbooks/STAGE6_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE6_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/assistant-memory/delululang-project.md:806` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/design/DeluluLang_Fable_5.1_Master_Prompt.md:1` — prose names `docs/NEXT_EVOLUTION_2026/DECISION_LOG.md`, which moved to `docs/archive/v1/NEXT_EVOLUTION_2026/DECISION_LOG.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE10_BUILD_ORDER.md:7` — prose names `docs/playbooks/STAGE10_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE10_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE10_BUILD_ORDER.md:1644` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE6_BUILD_ORDER.md:10` — prose names `docs/playbooks/STAGE6_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE6_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE7_BUILD_ORDER.md:14` — prose names `docs/playbooks/STAGE7_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE7_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE8_BUILD_ORDER.md:7` — prose names `docs/playbooks/STAGE8_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE9_BUILD_ORDER.md:7` — prose names `docs/playbooks/STAGE9_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE9_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:233` — prose names `docs/playbooks/README.md`, which moved to `docs/archive/v1/playbooks/README.md` in V2-0; the citation is historical and left as written
- `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:339` — prose names `docs/playbooks/README.md`, which moved to `docs/archive/v1/playbooks/README.md` in V2-0; the citation is historical and left as written

## note — `code-cited-but-not-allocated` (1)

**What to do:** add a row to `UNALLOCATED` in the registry saying which it is, or fix the citation

- `crates/delulu-diag/src/codes.rs` — 2 code(s) are named in the tree, are not allocated by the registry, and have no recorded disposition — so nothing says whether each is a retired code or a typo: DL0000 (crates/delulu/tests/terminal_text_cli.rs:242); DL1700 (docs/assistant-memory/delululang-project.md:546)

## note — `dependency-never-used-in-source` (3)

**What to do:** confirm it is reached through a re-export or a feature, or remove it

- `delulu-fuzz-targets/Cargo.toml` — delulu-fuzz-targets depends on `delulu-broker`, which no source file names
- `delulu-fuzz-targets/Cargo.toml` — delulu-fuzz-targets depends on `delulu-check`, which no source file names
- `delulu-fuzz-targets/Cargo.toml` — delulu-fuzz-targets depends on `delulu-runtime`, which no source file names

## note — `ruling-cited-without-its-stage` (1)

**What to do:** write the stage when you mean an earlier one — `S9-D21` — as the Stage-10 ledger already does

- `docs/design` — 359 citation(s) of rulings name a ruling number that more than one stage allocates, and rely on the documented default that a bare `D<n>` means the latest stage (`CHANGELOG.md`, `STAGE10_BUILD_ORDER.md` §2). Numbers affected: D1, D10, D11, D12, D13, D14, D15, D16, D17, D18, D19, D2, D20, D21, D22, D3, D4, D5, D6, D7, D8, D9

## note — `test-count-quoted-but-unverifiable` (1)

**What to do:** re-run the suite and update these by hand; the Survey deliberately does not guess

- `README.md` — these lines quote a test count, which is produced by `cargo test` and cannot be read out of the tree — verify them against a suite run: docs/assistant-memory/delulu-p22-containment-campaign.md:178, docs/assistant-memory/delululang-project.md:3, docs/assistant-memory/delululang-project.md:64, docs/assistant-memory/delululang-project.md:112, docs/assistant-memory/delululang-project.md:125, docs/assistant-memory/delululang-project.md:160, docs/assistant-memory/delululang-project.md:170, docs/assistant-memory/delululang-project.md:184, docs/assistant-memory/delululang-project.md:192, docs/assistant-memory/delululang-project.md:200

