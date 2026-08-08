# Survey discrepancies

**Generated — do not edit by hand.** Regenerate with `cargo run -p delulu-survey -- build`.

Each entry is a place where two parts of this repository disagree, or where something points
at what is not there. Every one names the file and line that produced it. Entries are not
graded by how bad they look — an `error` is a statement that contradicts the tree, a
`warning` is true today and drifting, a `note` is for a human to judge.

| Severity | Class | Count |
|---|---|---:|
| warning | `prose-cites-missing-path` | 9 |
| note | `c-token-not-a-campaign-finding` | 1 |
| note | `ruling-cited-without-its-stage` | 1 |
| note | `test-count-quoted-but-unverifiable` | 1 |

## warning — `prose-cites-missing-path` (9)

**What to do:** correct the path, or say plainly that it no longer exists

- `docs/security/red-team-surfaces-2026-08-08/agent-notes/deadman-NOTES.md:99` — prose names `crates/delulu-runtime/src/device.rs:355-428`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/deadman-NOTES.md:287` — prose names `crates/delulu/src/cli.rs:4639-4640`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/deadman-NOTES.md:350` — prose names `crates/delulu/src/broker_transport.rs:258-443`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/deadman-NOTES.md:351` — prose names `crates/delulu/src/device.rs:590-610`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/wasm-NOTES.md:79` — prose names `crates/delulu-wasm/tests/hostile_guest.rs:132–141`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/wasm-NOTES.md:121` — prose names `crates/delulu-wasm/tests/hostile_guest.rs:99–108`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/wasm-NOTES.md:165` — prose names `crates/delulu-wasm/tests/hostile_guest.rs:111–129`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/wasm-NOTES.md:249` — prose names `crates/delulu-wasm/tests/lib.rs:443–455`, which is nowhere in the tree
- `docs/security/red-team-surfaces-2026-08-08/agent-notes/wasm-NOTES.md:296` — prose names `crates/delulu-wasm/tests/lib.rs:185–217`, which is nowhere in the tree

## note — `c-token-not-a-campaign-finding` (1)

**What to do:** no action if these are C-language references; otherwise the ledger is missing a row

- `docs` — these `C<n>` tokens are not campaign findings and were not linked: C82, C83, C84, C85, C86, C87, C88, C89, C90, C91, C92, C99

## note — `ruling-cited-without-its-stage` (1)

**What to do:** write the stage when you mean an earlier one — `S9-D21` — as the Stage-10 ledger already does

- `docs/design` — 249 citation(s) of rulings name a ruling number that more than one stage allocates, and rely on the documented default that a bare `D<n>` means the latest stage (`CHANGELOG.md`, `STAGE10_BUILD_ORDER.md` §2). Numbers affected: D1, D10, D11, D12, D13, D14, D15, D16, D17, D18, D19, D2, D20, D21, D22, D3, D4, D5, D6, D7, D8, D9

## note — `test-count-quoted-but-unverifiable` (1)

**What to do:** re-run the suite and update these by hand; the Survey deliberately does not guess

- `README.md` — these lines quote a test count, which is produced by `cargo test` and cannot be read out of the tree — verify them against a suite run: docs/design/STAGE1_SPECIFICATION.md:941

