---
name: delulu-survey-map
description: "DeluluLang has a generated, provenance-carrying map of itself at docs/survey/ — consult it BEFORE any change and regenerate it AFTER"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-08-01T04:26:34.855Z
---

`docs/survey/` is **the Survey**: a map of the DeluluLang repository derived from the repository,
built by `crates/delulu-survey` (commissioned by Jesse 2026-08-01). Not the same thing as
`crates/delulu-atlas`, which maps a checked Delulu *program* from compiler facts — Jesse corrected
me on that distinction mid-build, so keep them apart.

- `docs/survey/SURVEY.md` — the human map (~25 KB: crates, dependency graph, every module, code ranges)
- `docs/survey/survey.json` — schema `survey/1`, one record per line
- `docs/survey/DISCREPANCIES.md` — generated; where the repo currently disagrees with itself
- `docs/survey/AUDIT.md` — hand-written record of the first audit; historical, do not "update" its numbers
- `docs/survey/README.md` — hand-written entry point

**The law it is built on:** every edge names the file and line it was read from; nothing is inferred
from name similarity or proximity. Extraction is lexical, so every relation is cross-checked against
an independent source (a `use` against the manifest, a `mod` against the filesystem, a `DLxxxx`
against the registry, a quoted count against a recount) and **disagreement is reported, never
silently resolved**.

## Architecture — independently verified 2026-08-01 at commit 9618f7a

```
delulu (CLI)  ──▶  delulu-survey  ──▶  serde, serde_json  (NO sibling crate)
  doctor.rs          health.rs
  • environment      • inspect() — build, staleness, write, in one pass
  • orchestration    • integrity() — THE invariant list
  • rendering        • tally(), find_source_tree(), OUTPUT_DIR, Severity::word
```

- **`delulu doctor` is the public interface** and stays that way (Jesse, 2026-08-01: "the
  one-command workflow is more important than reducing one dependency"). It makes exactly TWO calls
  into the Survey — `find_source_tree()` and `inspect(root, Repair::{Regenerate,ReportOnly})` — and
  computes no invariant of its own.
- **`delulu_survey::health::integrity()` is the single home of the invariants.** Add one there and
  `delulu doctor` reports it and `freshness.rs` enforces it with no other edit. Never re-implement
  an invariant in the CLI or in a test.
- **Never add a sibling dependency to `delulu-survey`** — that is what lets the map be read while
  the compiler does not build. `tests/architecture.rs` fails if anyone does.
- Environment checks (Python, `DELULU_HOME`, broker mode, audit chain) stay in the CLI. Moving them
  into the Survey would require `delulu-broker` and cost the property above.
- Verified by cargo, not by the map: survey → `serde`+`serde_json` only, one `delulu-*` line in its
  full tree (itself); `delulu → delulu-survey`; no cycle.

**Why:** a hand-maintained map is wrong by the third commit — `docs/REPOSITORY_STRUCTURE.md` §2 had
drifted to showing 5 of 12 crates, and the front door was 14% off on size. Generating it and gating
it means the map cannot lag the code. It also makes "what breaks if I change this?" a one-command
question instead of a grep.

**How to apply — Jesse's standing workflow, issued 2026-08-01. Follow it for EVERY code change
unless the task is explicitly about the Survey itself.**

*Before any change:*

1. Read `docs/survey/README.md`, then `SURVEY.md`, then `DISCREPANCIES.md`.
2. `query`/`rdeps` **every module you intend to modify** — check reverse dependencies before
   touching any public API. Ids: `crate:delulu-check`, `mod:crates/.../ty.rs`, `code:DL0501`,
   `ruling:S10-D64`, `finding:C69`, `doc:README.md`, `dir:docs/design`.
3. **Never rely on repository memory when the Survey has the answer** — including these memory
   files. The Survey is regenerated; memory is not.
4. **If Survey and source disagree, the SOURCE is truth.** Fix the source if needed, then
   regenerate. Never reconcile by editing the map.
5. **Never hand-edit `SURVEY.md`, `survey.json`, `DISCREPANCIES.md`** — generated. Only
   `cargo run -p delulu-survey -- build`. (`README.md`, `AUDIT.md`, `REMOVALS.md` in that directory
   ARE hand-written.)

*After the change:* update affected docs → run the full suite → regenerate the Survey (`delulu
doctor` does it) → confirm `cargo test --workspace` passes → confirm the committed map matches the
tree → **report new discrepancies, never silently ignore them.**

*When Jesse asks for independent verification:* start from the repository, not from the
conversation, and assume the implementation is wrong. That instruction has already paid — a
verification pass on 2026-08-01 found four duplicates (`OUT_DIR`, a second weaker source-tree
detection, a hand-rolled tally, a second `Severity→word`) that the restructure commit had left in
`delulu-survey`'s own binary while claiming they were gone. **Grep the whole repo for each claim
rather than checking the file you just edited.**

*Report at the end:* files changed · why each · Survey impact · reverse dependencies affected ·
new discrepancies · was the Survey regenerated · is the map still consistent · does any
documentation now need updating.

This is **enforced, not remembered**: `cargo test --workspace` runs
`the_committed_map_matches_the_tree`, which fails and names the first differing line. Five more
tests hold the map to its own standard (every edge cited, nothing dangling, totals self-consistent,
every workspace member present, two builds agree).
4. Do **not** run the Survey against `.claude/worktrees/` — see [[delulu-stale-worktree]]. It is
   excluded by name and must stay excluded.
5. The crate depends on **no sibling crate**, on purpose: the map must open when the compiler does
   not. Do not add a workspace dependency to it.
6. It is `publish = false` tooling, so "12 crates" in prose still means the 12 shipped language
   crates; the Survey reports `crates` and `crates_shipped` separately for exactly this reason.

Related: [[delulu-hardening-campaign]] (the campaign this landed inside), [[delulu-project]].
