# Surface early-drop addendum — the Atlas & the Palette

**Status:** BUILT 2026-07-15 (implementing chef; all 11 §4 criteria witnessed in §8 — head-chef
live re-verification pending). Specified 2026-07-14 (head chef). Stage 8 "Surface" material, built
early by the project owner's order. Implemented by the Opus 4.8 chef under this brief in three
phases: A1 the Palette, A2 the Atlas core, A3 renderers + custody overlay + close-out.

This addendum is **binding** the same way `STAGE5_GUARD_ADDENDUM.md` was: the implementing model
works phase by phase, appends deviations to §7 instead of silently departing, and fills the §8
close-out table with witnessing tests. Diagnostic codes come from the Stage 8 range (DL17xx);
this addendum allocates only DL1780–DL1799, leaving DL1700–DL1779 free for the main Stage 8
build.

---

## 1. Prior art and verdicts

The current generation of knowledge-graph code-mapping tools for AI coding agents shares one
architecture: parse source text with tree-sitter (plus an optional LLM semantic pass), build a
graph, cluster it statistically, and expose HTML viz + query/path/explain verbs. The Atlas is
written from scratch in Rust from the Delulu compiler's own facts; nothing is copied, and no
external tool is named in user-facing surfaces (owner's ruling).

### Adopt / reject table

| Prior-art concept | Verdict | Why |
|---|---|---|
| Graph as the primary navigation surface (query instead of grep) | **ADOPT** | The core insight is right: agents and humans both waste effort re-deriving structure. |
| `query` / `path` / `explain`-style verbs against a persistent graph | **ADOPT** (as `atlas node/path/callers/calls/why`) | Agents must be able to ask small questions without loading the whole graph. |
| God nodes (most-connected concepts) | **ADOPT** | Cheap (degree count), genuinely orienting. |
| Honest audit trail; never invent an edge | **ADOPT (strengthened)** | We go further: every atlas edge comes from the checker, so there is nothing to hedge. |
| Self-contained HTML visualization | **ADOPT (improved)** | Ours embeds data + JS inline (no CDN, works offline), colors by *effect row*, and renders authority edges distinctly. |
| Deterministic, local, no-LLM code extraction | **ADOPT (trivially)** | The compiler *is* the extractor. Zero heuristics survive. |
| tree-sitter text extraction + `EXTRACTED/INFERRED/AMBIGUOUS` confidence tags | **REJECT** | Confidence tags exist because text-extraction tools must guess. Delulu's checker has ground truth (resolved names, typed effect rows, checked authority). An atlas edge either is a checked fact or it does not appear. |
| LLM semantic pass over docs/media | **REJECT** | Out of scope and nondeterministic; the Atlas maps *programs*. |
| Leiden community detection + LLM community labels | **REJECT** | Statistical clusters with invented names. Delulu already has real communities — packages and modules — with real names. |
| Embedding/vector anything | **REJECT** | A traversable graph answers structural questions; embeddings add nondeterminism for nothing here. |
| MCP server | **DEFER** | Stage 8 proper (LSP work) is the right home for protocol servers. |

---

## 2. The model

### 2.1 The Atlas (`delulu atlas`)

A typed, deterministic graph of a checked program, derived **only** from compiler and (optionally)
broker facts. Consumers and their native formats:

| Consumer | Surface |
|---|---|
| Human, terminal | default colored tree/overview; `--format tree` |
| Human, browser | `--format html` — single self-contained file |
| AI agent / LLM | `--format digest` (token-budgeted Markdown, self-describing) + query verbs + `--format json` |
| Other tools | `--format dot` (Graphviz), `--format mermaid` (module-level) |

**Node kinds:** `package`, `module`, `function`, `type`, `effect`, `resource` (fs path / net host
/ secret name), `foreign` (C symbol / Python module).
**Edge kinds:** `contains`, `depends_on` (package→package), `imports` (module→module), `calls`
(function→function), `uses_type`, `performs` (function→effect, from the checked row),
`requires` (function/package→resource, from the authority report), `foreign` (function→foreign),
`declassifies` (function→resource of kind secret), `delegates` (custody overlay only).

**Stable id scheme** (deterministic, diff-friendly): `pkg:<name>`, `mod:<pkg>/<module>`,
`fn:<pkg>/<module>.<name>`, `type:<pkg>/<module>.<name>`, `effect:<Name>`,
`res:<class>:<pattern>` (classes as in the Guard: `fs_read`, `fs_write`, `net`, `secret`, …),
`foreign:c:<symbol>`, `foreign:py:<module>`, `grant:<node-id>` (custody overlay).

**Data sources (ground truth — read these, do not re-parse):** `delulu-check`
`resolve.rs` (`DeclTable`), `check.rs` (`FnFacts`: purity, effect rows), `program.rs`
(`Program`: facts, fn_types, call ownership, entry module), `deps.rs` (`Workspace`,
`ResolvedPackage`, `PackageAuthority`), `authority.rs` (`authority_report`). If the checker does
not already record function→function call edges in a reusable side table, add one to
`CheckResult` populated during call checking — never by re-lexing source. The custody overlay
uses the existing broker client (read-only verbs only).

**Determinism:** with color off, two runs over the same input produce byte-identical output in
every format. All collections sort by id before emission. The HTML layout is seeded/hierarchical,
not random.

**God nodes:** top-N by degree (default 10), reported in every format.

**No partial graphs:** if `check` reports errors, the atlas refuses with **DL1780** ("the atlas
is built from checked facts — fix the N error(s) above first") after printing the check
diagnostics. Overlays degrade gracefully instead: broker unreachable ⇒ **DL1781** (note
severity) and the atlas is emitted without the custody overlay.

### 2.2 Agent-side access (the improvement Jesse asked for)

Agents must never need the whole graph in context:

- `delulu atlas node <name>` — one node: kind, effects, authority, in/out edges (grouped, capped).
- `delulu atlas path <A> <B>` — shortest path, hops printed with typed edge kinds.
- `delulu atlas callers <fn>` / `delulu atlas calls <fn>` — reverse/forward call slice.
- `delulu atlas why <Effect|resource>` — which functions perform/require it and through which
  call chains (graph-shaped sibling of the existing `delulu why`, which stays byte-stable).
- `--budget <N>` — cap any textual output at ~N tokens (chars/4 heuristic, stated as heuristic);
  truncation is explicit ("… truncated at budget; narrow with `atlas node <id>`"), never silent.
- The digest ends with a fixed **"Querying further"** footer teaching exactly these verbs, so any
  agent that reads `ATLAS.md` learns how to drill down without being told.

### 2.3 The digest (`--format digest`)

Deterministic Markdown engineered for LLM context windows: header (root, counts, caveats), one
line per package, one line per module (name, function count, effect union), god-node table,
authority table (mirrors `delulu authority` exactly), foreign-boundary list, then the querying
footer. Default budget 2000 tokens. `--out DIR` writes `ATLAS.md` + `atlas.json` (+ `atlas.html`
when requested) instead of stdout.

### 2.4 JSON schema `atlas/1`

Versioned like `broker/1`; additive evolution only. Single object:
`{ "atlas": "atlas/1", "root", "nodes": [...], "edges": [...], "gods": [...], "authority": {...},
"custody": null | {...}, "caveats": [...] }`. Node objects carry `id`, `kind`, `name`, and
kind-specific fields (`module`, `package`, `span` {file, line}, `effects`, `pure`). Machine
channels are **never** colored.

### 2.5 The Palette

A role-based color system for every human-facing CLI surface. Lives in `delulu-diag` so every
renderer shares it.

**Roles, not hardcoded colors:** `error`, `warning`, `note`, `code` (DL numbers), `span_primary`,
`span_secondary`, `effect`, `authority`, `guard_banner`, `success`, `path`, `repair`, `heading`.

**Resolution order (first match wins):** `--color never|always|auto` flag → `DELULU_COLOR` env →
`NO_COLOR` env (any value ⇒ off; https://no-color.org) → auto (on iff the stream is a TTY).
`NO_COLOR` beats `DELULU_COLOR=always` but not the explicit `--color always` flag (an explicit
per-invocation flag is the user speaking *now*). JSON output is never colored regardless of all
of the above.

**Themes:** built-ins `default` (colorblind-safe: never distinguish by red/green alone; severity
words remain the primary signal), `bright`, `mono` (bold/underline only, zero color SGR).
Selected by `--theme <name>` → `DELULU_THEME` → `~/.delulu/theme.toml` (`theme = "name"` plus
optional `[roles]` table overriding individual roles with named 16-color values). An invalid
theme name or unparseable theme file yields **DL1790** (warning) and falls back to `default` —
never a hard failure.

**What gets colored:** diagnostics via `render_human` (severity, code, span carets/labels),
guard banners (bypass banner in the strongest error style), `ok:`/success lines, grants tree,
atlas tree view, `explain` headings. Full syntax highlighting of source snippets is **deferred**
to Stage 8 proper (LSP/fmt) — record in docs, do not sneak it in.

**Windows:** enable VT processing on startup when coloring a console. Prefer existing
dependencies; adding a minimal, widely-used styling crate (`anstyle`/`anstream`) is permitted
only if hand-rolling would itself require a new dependency — pick the smallest total footprint
and record the choice as a §7 deviation.

### 2.6 Honesty caveats (binding, verbatim where marked)

- The atlas shows **static, checked structure** — it is not a runtime trace. Calls through
  function values may be under-approximated; this caveat ships in `caveats` in every format
  (verbatim sentence: "the atlas is a static map of checked facts, not a runtime trace; calls
  through function values may be under-approximated").
- The custody overlay reflects broker state **at the moment of the query** and requires the
  daemon; it is awareness, not enforcement (the Guard enforces).
- The token budget is a chars/4 **heuristic**, stated as such wherever a budget is reported.
- Never claim "complete call graph", "always up to date", or benchmark superiority.

---

## 3. Surface

### 3.1 CLI

```
delulu atlas <file.delulu | package-dir> [--format tree|digest|json|dot|mermaid|html]
             [--out DIR] [--budget N] [--custody] [--gods N] [--json]
delulu atlas node <name-or-id> [--budget N] [--json]
delulu atlas path <A> <B> [--json]
delulu atlas callers <fn> | calls <fn> [--json]
delulu atlas why <Effect|resource> [--json]
```

Global (all commands): `--color never|always|auto`, `--theme <name>`; envs `DELULU_COLOR`,
`DELULU_THEME`, `NO_COLOR`. `--json` remains the machine channel everywhere. Update `usage()`.

### 3.2 Diagnostics (registered in `delulu-diag`, with explain bodies)

| Code | Severity | Meaning |
|---|---|---|
| DL1780 | error | atlas refused: program has check errors (names the count) |
| DL1781 | note | custody overlay unavailable — broker daemon not reachable; atlas emitted without it |
| DL1790 | warning | invalid theme name or malformed `theme.toml` — using `default` |

Explain topics: **E-ATLAS** (what the atlas is, formats per consumer, query verbs, caveats
verbatim) and **E-PALETTE** (roles, themes, resolution order, NO_COLOR compliance, why JSON is
never colored). No DL1784 is ever allocated (house rule mirroring DL1404).

### 3.3 Crates

New crate `delulu-atlas` (model, builder, formats, queries) depending on `delulu-check` (+
`delulu-diag`); the CLI stays thin. Palette lives in `delulu-diag` (`palette.rs`). No changes to
runtime/broker semantics; the custody overlay uses existing read-only wire verbs only.

---

## 4. Acceptance criteria (each needs a witnessing test named in §8)

1. **Determinism** — same input, color off: byte-identical output across two runs, every format.
2. **`atlas/1` JSON** — versioned envelope, stable ids, round-trips through serde; contains
   zero ANSI bytes even under `--color always`.
3. **Digest discipline** — default ≤ 2000-token budget; stable ordering; contains authority
   table, god nodes, caveats, and the "Querying further" footer verbatim.
4. **Query verbs** — `node`, `path`, `callers`, `calls`, `why` answer from the graph without
   full-graph output; `path` prints typed hops; `--budget` truncates explicitly, never silently.
5. **Authority parity** — `performs`/`requires` edges agree exactly with `delulu authority` on
   the same package (machine-checked in a test, not eyeballed).
6. **Refusal honesty** — check errors ⇒ DL1780 + no partial graph; broker down ⇒ DL1781 note +
   graph still emitted without overlay.
7. **Self-contained HTML** — artifact contains embedded data + inline JS, zero external URLs
   (test greps the artifact for `http://`/`https://`), and a node-count cap that collapses to
   module level above 3000 nodes with an explicit notice.
8. **Palette precedence** — `--color` flag > `DELULU_COLOR` > `NO_COLOR` > auto, with the single
   exception that `NO_COLOR` beats `DELULU_COLOR=always`; `--json` output never contains SGR
   bytes; piped output (non-TTY) is colorless by default.
9. **Themes** — three built-ins present; `mono` emits no color SGR; `theme.toml` role override
   honored; invalid theme ⇒ DL1790 + fallback (all tested).
10. **Zero regression** — the entire pre-existing workspace suite passes unchanged (color
    defaults off for non-TTY, so no existing assertion may need edits; editing an existing test
    to accommodate color is a defect, not a fix).
11. **Docs & explain** — E-ATLAS + E-PALETTE registered with bodies; `usage()` updated;
    `docs/REPOSITORY_STRUCTURE.md`, `docs/playbooks/README.md` (Stage 8 row gains the early-drop
    note), and this addendum's §8 close-out all updated; no external tool named anywhere.

---

## 5. Phases (one phase = one commit, suite green at each)

- **Phase A1 — the Palette.** `delulu-diag/src/palette.rs` (roles, themes, resolution,
  Windows VT), plumb through `render_human`, guard banners, `ok:` lines, grants tree; theme.toml;
  DL1790; E-PALETTE; unit + CLI tests (criteria 8, 9, 10).
- **Phase A2 — the Atlas core.** `delulu-atlas` crate: model/ids/builder (file, package,
  workspace), call-edge side table if needed, god nodes, `tree`/`digest`/`json` formats, query
  verbs, `--budget`, DL1780/DL1781, E-ATLAS; tests (criteria 1–6).
- **Phase A3 — the Atlas rendered + close-out.** `dot`, `mermaid`, self-contained `html`,
  `--custody` overlay, colored tree via the Palette, e2e orchestration test, docs, §8 close-out
  (criteria 7, 11; re-verify 1–6 stand).

## 6. Head-chef rulings (pre-answered, do not re-litigate)

1. No LLM calls anywhere in the Atlas. Deterministic compiler facts only.
2. No community detection. Packages and modules are the communities; use their real names.
3. No confidence tags. A fact appears or it does not; under-approximation is a stated caveat.
4. Machine channels (`--json`, `atlas/1`, lockfiles) are never styled. NO_COLOR is honored.
5. New crate `delulu-atlas`: yes. New third-party deps: only per §2.5's Windows rule, recorded
   as a deviation.
6. Existing tests are untouchable (criterion 10).
7. Custody overlay is read-only awareness; no owner code involved; it never blocks the atlas.
8. `delulu why` output stays byte-stable; `atlas why` is a sibling, not a replacement.

## 7. Deviations (implementing model appends; head chef rules)

1. **The Palette adds ZERO dependencies — hand-rolled ANSI + reuse of the existing `windows-sys`**
   (§2.5's Windows rule; "smallest total footprint wins"). SGR sequences are emitted directly
   (`\x1b[…m`) and `theme.toml` is read by a ~30-line line-parser, so `delulu-diag` gains no
   dependency at all. Windows VT processing is enabled once at CLI startup (`cli::enable_vt`) via the
   `windows-sys` `Win32_System_Console` API **already in the `delulu` crate's tree** (added for the
   broker's named-pipe transport in chunk 3) — nothing new enters the lockfile. `anstyle`/`anstream`
   were therefore not needed, and this is not the "new third-party dependency" escalation case.
   Recorded here per §2.5's instruction to record the choice.
2. **`DELULU_THEME_FILE` env override for the theme-file path (additive).** The documented theme
   file stays `~/.delulu/theme.toml`; the implementation also honors `$DELULU_THEME_FILE` (an
   explicit path) ahead of that default. Unset ⇒ behavior is exactly as specified. It is the
   hermetic test lever for `[roles]` overrides and malformed-file fallback (the same role
   `DELULU_STATE_DIR` plays for the broker), plus a power-user convenience. Minor and superset-only.
   Documented in the `E-PALETTE` explain body (head-chef requirement) and in A3 docs. The theme file
   is read through a bounded reader (64 KiB cap, UTF-8 checked): an oversized/hostile file can never
   exhaust memory — it degrades to DL1790 + the default theme (`palette::tests::oversized_theme_file_is_rejected_gracefully`).
3. **No new call-edge side table was added — the checker already records reusable call edges.**
   §2.1 said "if the checker does not already record function→function call edges in a reusable side
   table, add one to `CheckResult`". It DOES: `FnFacts.callees` (per-function callee names) plus
   `Program.call_owner` (per-module name→owner resolution) are exactly the reusable, resolved call
   edges — the same pair `delulu why`, `authority_report`, and `program_authority` already walk. The
   Atlas reuses them and re-lexes nothing (build.rs `EdgeKind::Calls`). No `CheckResult` change was
   needed, so none was made.
4. **A2 phase boundary (not a departure): renderers + custody are A3.** The `foreign` and `delegates`
   edge kinds and the `foreign`/`grant` node kinds are defined in the `atlas/1` model but populated
   in A3 (foreign per-call-site edges need an expression walk; the custody overlay needs the broker);
   A2 surfaces the foreign boundary in the digest from the authority report's `foreign_calls`.
   `--format dot|mermaid|html` return a clean "arrives in phase A3" message in A2; `--custody` is
   accepted but the overlay (and its DL1781 broker-down note) lands in A3. `atlas/1` evolution to add
   them is additive, as the schema requires.
   **[A3.1 correction, head-chef verification gap]:** the A3 commit populated `delegates`/`grant`
   but NOT `foreign` — the model/ids/renderer support existed with no builder path constructing a
   Foreign node or edge. Landed in the A3.1 finishing commit: declared C symbols
   (`foreign:c:<symbol>`, from the module's checked `foreign` blocks — a declared symbol always
   gets a node, naming its lib) and literal Python imports (`foreign:py:<module>`, the same
   `py.import("literal")` rule as the authority report's `imports_seen`); a function→foreign
   `foreign` edge is added only where the function's CHECKED row carries `ForeignCall` AND its
   already-parsed body reaches the boundary (a read-only AST walk — never re-lexed; non-literal
   import names are skipped, covered by the §2.6 under-approximation caveat). Witnesses:
   `delulu_atlas::tests::{foreign_c_symbols_get_nodes_and_gated_edges, python_imports_get_nodes_and_edges}`
   (unit) and `atlas_cli.rs::foreign_boundary_appears_in_atlas_json_with_nodes_and_edges`
   (through the binary on `examples/numpy_mean.delulu`, incl. byte-identical re-run + a typed
   `--foreign-->` path hop).

## 8. Close-out (criterion → witnessing test → status)

Workspace tests **398 (Guard baseline) → 425 (A1) → 447 (A2) → 456 (A3) → 459 (A3.1)**. The 2
pre-existing ignored are untouched — criterion 10 held at every phase. All 11 criteria are **met**.

| # | Criterion (§4) | Status | Witnessing test |
|---|---|---|---|
| 1 | Determinism — byte-identical across runs, every format | **met (A2)** | `atlas_cli.rs::output_is_byte_identical_across_runs_every_format` (tree/digest/json through the binary); unit `delulu_atlas::tests::build_is_deterministic` |
| 2 | `atlas/1` JSON — versioned, stable ids, serde round-trip, zero ANSI under `--color always` | **met (A2)** | `atlas_cli.rs::{atlas_json_is_a_versioned_envelope_with_stable_ids, json_carries_zero_ansi_even_under_color_always}`; units `model::tests::{atlas_round_trips_through_serde, json_carries_zero_ansi_bytes}` |
| 3 | Digest discipline — ≤2000-token budget, ordering, authority/gods/caveats + "Querying further" footer | **met (A2)** | `atlas_cli.rs::{digest_has_budget_ordering_authority_gods_caveats_and_footer, digest_is_never_colored_even_under_color_always}`; unit `delulu_atlas::tests::digest_has_footer_gods_authority_and_caveat` |
| 4 | Query verbs — node/path/callers/calls/why, typed hops, explicit `--budget` truncation | **met (A2)** | `atlas_cli.rs::{query_verbs_answer_without_the_whole_graph, query_json_is_structured_and_uncolored, budget_truncation_is_explicit_never_silent}`; units `delulu_atlas::tests::{query_verbs_answer_from_the_graph, budget_truncation_is_explicit}` |
| 5 | Authority parity — `performs`/`requires` == `delulu authority` (machine-checked) | **met (A2)** | `atlas_cli.rs::atlas_authority_matches_delulu_authority_exactly` (compares effects/capabilities/secrets/foreign_calls/pure_functions + performs edges); unit `delulu_atlas::tests::authority_parity_effects_match_delulu_authority` |
| 6 | Refusal honesty — DL1780 (no partial graph); DL1781 (broker down, graph still emitted) | **met (A2+A3)** | DL1780: `atlas_cli.rs::{check_errors_refuse_with_dl1780_and_no_partial_graph, refusal_in_json_mode_is_a_diagnostics_envelope_not_a_graph}` (exit 1, no `atlas/1` on refusal). DL1781: `atlas_e2e.rs::atlas_custody_overlay_live_then_degraded_end_to_end` step 3 (real broker stopped ⇒ DL1781 note on stderr, atlas still emitted, `custody: null`, exit 0) |
| 7 | Self-contained HTML — embedded data + inline JS, zero external URLs, 3000-node collapse | **met (A3)** | `atlas_cli.rs::html_is_self_contained_through_the_binary` + `atlas_e2e.rs` step 4 (both grep the artifact for `http://`/`https://`); units `formats::tests::{html_is_self_contained_with_zero_external_urls, html_collapses_above_the_node_cap}` (the collapse test builds a >3000-node graph and asserts the explicit notice + module-level data) |
| 8 | **Palette precedence** — flag > DELULU_COLOR > NO_COLOR > auto (NO_COLOR beats DELULU_COLOR=always); `--json` never SGR; piped colorless | **met (A1)** | `palette_cli.rs::{piped_output_is_colorless_by_default, color_always_flag_paints_the_diagnostic, json_is_never_colored_even_under_color_always, delulu_color_env_forces_color_when_piped, no_color_beats_delulu_color_always, color_always_flag_beats_no_color, color_never_flag_disables_even_with_delulu_color_always, ok_success_line_is_painted}`; unit `palette::tests::color_precedence_lattice` |
| 9 | **Themes** — three built-ins; `mono` no color SGR; `theme.toml` role override; invalid ⇒ DL1790 + fallback | **met (A1)** | `palette_cli.rs::{mono_theme_emits_no_color_sgr, bright_theme_uses_bright_colors, theme_toml_role_override_is_honored, invalid_theme_is_dl1790_warning_and_falls_back, malformed_theme_file_is_dl1790_and_still_runs}`; units `palette::tests::{mono_emits_no_color_sgr, theme_toml_role_override_is_honored, bad_theme_name_falls_back_with_warning, malformed_theme_file_falls_back_with_warning, theme_precedence_flag_over_env_over_file}` |
| 10 | **Zero regression** — pre-existing suite passes unchanged | **met (A1–A3)** | the full 398-test Guard-baseline suite runs UNMODIFIED at every phase (456 total, 0 fail); `render::tests::disabled_palette_matches_render_human_byte_for_byte` proves color-off is byte-identical |
| 11 | Docs & explain — E-ATLAS + E-PALETTE bodies, `usage()`, REPOSITORY_STRUCTURE/README, this close-out | **met (A3)** | E-PALETTE (`codes::tests::palette_code_and_e_palette_topic`) + E-ATLAS (`codes::tests::atlas_codes_and_e_atlas_topic`, `atlas_cli.rs::explain_e_atlas_describes_the_model`) registered with bodies; `usage()` covers `atlas` + the global `--color`/`--theme`; `docs/REPOSITORY_STRUCTURE.md` (palette.rs + delulu-atlas + new test files) and `docs/playbooks/README.md` (Stage 8 early-drop row) updated; this close-out complete; no external tool named anywhere (repo-wide grep clean, §1 owner's ruling) |

Additional A3/A3.1 surfaces beyond the criteria: the FOREIGN BOUNDARY is in the graph (§7
deviation 4's A3.1 correction — `foreign:c:`/`foreign:py:` nodes + `ForeignCall`-gated `foreign`
edges; `atlas path` routes through the boundary with a typed `--foreign-->` hop, witnessed on
`examples/numpy_mean.delulu` through the binary); the custody overlay (`--custody`) attaches `grant:` nodes
+ `delegates` edges from the broker's read-only `List` verb (no new wire verbs, no owner code —
ruling 7), ships the §2.6 custody caveat verbatim, and never blocks the atlas
(`atlas_e2e.rs::atlas_custody_overlay_live_then_degraded_end_to_end` steps 2–3, unit
`delulu_atlas::tests::attach_custody_adds_grants_delegates_and_caveat`); `dot`/`mermaid` render
deterministically (`atlas_cli.rs::dot_and_mermaid_render_through_the_binary`, e2e step 1 covers all
six formats byte-identical twice); the tree view is Palette-colored only on request/TTY
(`atlas_cli.rs::tree_is_colored_only_when_asked_and_plain_when_piped`); `--out DIR` writes
`ATLAS.md` + `atlas.json` (+ `atlas.html` under `--format html`).
