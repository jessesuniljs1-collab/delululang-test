# Survey discrepancies

**Generated — do not edit by hand.** Regenerate with `cargo run -p delulu-survey -- build`.

Each entry is a place where two parts of this repository disagree, or where something points
at what is not there. Every one names the file and line that produced it. Entries are not
graded by how bad they look — an `error` is a statement that contradicts the tree, a
`warning` is true today and drifting, a `note` is for a human to judge.

| Severity | Class | Count |
|---|---|---:|
| note | `build-order-without-citable-rulings` | 3 |
| note | `c-token-not-a-campaign-finding` | 1 |
| note | `code-cited-but-not-allocated` | 1 |
| note | `ruling-cited-without-its-stage` | 1 |
| note | `test-count-quoted-but-unverifiable` | 1 |

## note — `build-order-without-citable-rulings` (3)

**What to do:** no action if the stage genuinely took no recorded decisions; otherwise number them

- `docs/design/STAGE6_BUILD_ORDER.md:1` — this build order allocates no numbered rulings, so a Stage-6 decision cannot be cited the way a Stage-9 or Stage-10 one can
- `docs/design/STAGE7_BUILD_ORDER.md:1` — this build order allocates no numbered rulings, so a Stage-7 decision cannot be cited the way a Stage-9 or Stage-10 one can
- `docs/design/STAGE8_BUILD_ORDER.md:1` — this build order allocates no numbered rulings, so a Stage-8 decision cannot be cited the way a Stage-9 or Stage-10 one can

## note — `c-token-not-a-campaign-finding` (1)

**What to do:** no action if these are C-language references; otherwise the ledger is missing a row

- `docs` — these `C<n>` tokens are not campaign findings and were not linked: C99

## note — `code-cited-but-not-allocated` (1)

**What to do:** consider a RETIRED list beside REGISTRY so a reader can tell a retired code from a typo

- `crates/delulu-diag/src/codes.rs` — 16 code(s) are named in the tree but not allocated by the registry — retired, deliberately skipped, or mistyped, and nothing on record says which: DL0210 (docs/design/STAGE1_SPECIFICATION.md:858); DL0503 (crates/delulu-conform/src/rules.rs:128); DL0702 (crates/delulu-conform/src/tests.rs:281); DL0906 (crates/delulu-conform/src/tests.rs:281); DL1012 (docs/design/STAGE2_SPECIFICATION.md:279); DL1203 (docs/design/STAGE3_SPECIFICATION.md:249); DL1404 (crates/delulu-diag/src/codes.rs:126); DL1419 (docs/design/STAGE10_BUILD_ORDER.md:927); DL1420 (docs/design/STAGE10_BUILD_ORDER.md:927); DL1609 (crates/delulu-diag/src/codes.rs:173); DL1700 (docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:13); DL1708 (crates/delulu-diag/src/codes.rs:198); DL1779 (docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:13); DL1784 (crates/delulu-diag/src/codes.rs:174); DL1799 (docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md:13); DL9999 (crates/delulu-conform/src/rules.rs:571)

## note — `ruling-cited-without-its-stage` (1)

**What to do:** write the stage when you mean an earlier one — `S9-D21` — as the Stage-10 ledger already does

- `docs/design` — 233 citation(s) of rulings name a ruling number that more than one stage allocates, and rely on the documented default that a bare `D<n>` means the latest stage (`CHANGELOG.md`, `STAGE10_BUILD_ORDER.md` §2). Numbers affected: D1, D10, D11, D12, D13, D14, D15, D16, D17, D18, D19, D2, D20, D21, D22, D3, D4, D5, D6, D7, D8, D9

## note — `test-count-quoted-but-unverifiable` (1)

**What to do:** re-run the suite and update these by hand; the Survey deliberately does not guess

- `README.md` — these lines quote a test count, which is produced by `cargo test` and cannot be read out of the tree — verify them against a suite run: README.md:34, README.md:34, docs/design/STAGE1_SPECIFICATION.md:938, docs/design/STAGE2_SPECIFICATION.md:485, docs/design/STAGE2_SPECIFICATION.md:495

