# Survey discrepancies

**Generated — do not edit by hand.** Regenerate with `cargo run -p delulu-survey -- build`.

Each entry is a place where two parts of this repository disagree, or where something points
at what is not there. Every one names the file and line that produced it. Entries are not
graded by how bad they look — an `error` is a statement that contradicts the tree, a
`warning` is true today and drifting, a `note` is for a human to judge.

| Severity | Class | Count |
|---|---|---:|
| warning | `prose-cites-missing-path` | 2 |
| note | `c-token-not-a-campaign-finding` | 1 |
| note | `cites-archived-path` | 23 |
| note | `dependency-never-used-in-source` | 1 |
| note | `ruling-cited-without-its-stage` | 1 |

## warning — `prose-cites-missing-path` (2)

**What to do:** correct the path, or say plainly that it no longer exists

- `docs/design/DeluluLang_V2_Execution_Master_Prompt.md:142` — prose names `docs/design/OLD_PLAN.md`, which is nowhere in the tree
- `docs/design/DeluluLang_V2_Execution_Master_Prompt.md:145` — prose names `docs/archive/v1/design/OLD_PLAN.md`, which is nowhere in the tree

## note — `c-token-not-a-campaign-finding` (1)

**What to do:** no action if these are C-language references; otherwise the ledger is missing a row

- `docs` — these `C<n>` tokens are not campaign findings and were not linked: C99

## note — `cites-archived-path` (23)

**What to do:** a current document should cite the archived path

- `CHANGELOG.md:359` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:30` — prose names `docs/design/LANGUAGE_SPECIFICATION.md`, which moved to `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:31` — prose names `docs/design/PRODUCTION_READINESS_REVIEW.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:32` — prose names `docs/design/PRODUCTION_READINESS_2026-08-09.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:33` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:34` — prose names `docs/design/P19_ECOSYSTEM_REVIEW.md`, which moved to `docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md:35` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/REPOSITORY_STRUCTURE.md:427` — prose names `docs/design/LANGUAGE_SPECIFICATION.md`, which moved to `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:160` — prose names `docs/design/LANGUAGE_SPECIFICATION.md`, which moved to `docs/archive/v1/design/LANGUAGE_SPECIFICATION.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:161` — prose names `docs/design/PRODUCTION_READINESS_REVIEW.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:162` — prose names `docs/design/PRODUCTION_READINESS_2026-08-09.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-09.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:163` — prose names `docs/design/PRODUCTION_READINESS_2026-08-10.md`, which moved to `docs/archive/v1/design/PRODUCTION_READINESS_2026-08-10.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:164` — prose names `docs/design/P19_ECOSYSTEM_REVIEW.md`, which moved to `docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/archive/v1/NEXT_EVOLUTION_2026/DOCUMENTATION_AUDIT.md:165` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/design/DeluluLang_Fable_5.1_Master_Prompt.md:1` — prose names `docs/NEXT_EVOLUTION_2026/DECISION_LOG.md`, which moved to `docs/archive/v1/NEXT_EVOLUTION_2026/DECISION_LOG.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE10_BUILD_ORDER.md:7` — prose names `docs/playbooks/STAGE10_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE10_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE10_BUILD_ORDER.md:1644` — prose names `docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, which moved to `docs/archive/v1/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE6_BUILD_ORDER.md:10` — prose names `docs/playbooks/STAGE6_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE6_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE7_BUILD_ORDER.md:14` — prose names `docs/playbooks/STAGE7_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE7_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE8_BUILD_ORDER.md:7` — prose names `docs/playbooks/STAGE8_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/STAGE9_BUILD_ORDER.md:7` — prose names `docs/playbooks/STAGE9_PLAYBOOK.md`, which moved to `docs/archive/v1/playbooks/STAGE9_PLAYBOOK.md` in V2-0; the citation is historical and left as written
- `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:224` — prose names `docs/playbooks/README.md`, which moved to `docs/archive/v1/playbooks/README.md` in V2-0; the citation is historical and left as written
- `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:330` — prose names `docs/playbooks/README.md`, which moved to `docs/archive/v1/playbooks/README.md` in V2-0; the citation is historical and left as written

## note — `dependency-never-used-in-source` (1)

**What to do:** confirm it is reached through a re-export or a feature, or remove it

- `delulu-fuzz-targets/Cargo.toml` — delulu-fuzz-targets depends on `delulu-runtime`, which no source file names

## note — `ruling-cited-without-its-stage` (1)

**What to do:** write the stage when you mean an earlier one — `S9-D21` — as the Stage-10 ledger already does

- `docs/design` — 272 citation(s) of rulings name a ruling number that more than one stage allocates, and rely on the documented default that a bare `D<n>` means the latest stage (`CHANGELOG.md`, `STAGE10_BUILD_ORDER.md` §2). Numbers affected: D1, D10, D11, D12, D13, D14, D15, D16, D17, D18, D19, D2, D20, D21, D22, D3, D4, D5, D6, D7, D8, D9

