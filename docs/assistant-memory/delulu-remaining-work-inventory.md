---
name: delulu-remaining-work-inventory
description: "docs/REMAINING_WORK.md is the single gap inventory (60 items, 2026-08-23); the three structural gaps nobody had collected in one place"
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-23T17:12:31.514Z
---

`docs/REMAINING_WORK.md` (created 2026-08-23, commit `2189554`) is the **one place** listing
everything the repository specifies, describes or implies and does not build — 60 items in six
categories, each with a *Verified* column saying how it was checked and a *what closing it takes*
column. It is the join across `CHECKPOINT-1.0.md` §8/§9, `QUESTIONS.md` Part 5, `MATHEMATICS.md`
§12, the Book Ch. 20, `DEPLOYMENT.md` §5 and `HANDOFF.md` §8 — which is why it did not exist
before: **every fact was already written down, just never in one place.**

**Consult it before planning work or before believing a feature exists.** Linked from README,
HANDOFF §5 and REPOSITORY_STRUCTURE.

## The three structural gaps that surprised me (none was a secret; none was collected)

1. **The microVM layer is a probe.** Constitution §5.14 names four defence layers;
   `crates/delulu/src/microvm.rs` is 58 lines and `probe()` returns `Err` on **every** path,
   including a fully-provisioned Linux+KVM host. §5.15 guarantee 5 ("contained at the WASM/microVM
   layer") rests on the WASM half alone. Honestly labelled everywhere — nothing weaker ever
   launches under the `microvm` name — but the layer does not exist.
2. **The standard library is four list methods**: `len`, `get`, `push`, `map`. No `filter`, `fold`,
   `sort`, `contains`; no `Map`/`Dict`/`Set` among the 16 prelude types; 15 prelude builtins.
   `is_higher_order_method` (`check.rs:3281`) still names `List.filter`, which answers `DL0405`.
3. **`CLI_STRINGS` is an empty array** (`delulu-diag/src/catalog.rs`). The catalog mechanism is
   complete and diagnostics localize end to end, but **no CLI prose is localizable in any locale**,
   so every `[cli.*]` key the guide documents would get DL1704 and fall back.

Cheapest high-value items, in order: run `cargo test --workspace` on any Mac · wire the already-
committed NIST KAT vectors (`measurements/pqc/vectors/`) into a test · write the `cargo-fuzz`
targets · register the CLI strings · grow the stdlib · restate Progress as progress-or-fault.

## The microVM overclaim, and the one file it is still in (commit `cdcc1e5`)

The unbuilt layer was being **counted as a source of strength**: README's *Honesty* section listed
`type proof → WASM/WASI floor → microVM containment → human-held broker keys`, and so did
`QUESTIONS.md` §1.3/§2.1 and the Book Ch. 14/15/20 (Ch. 15 presented the profile as available for
untrusted execution; Ch. 20's "Not built" list omitted it). All corrected 2026-08-23.

**`CONSTITUTION.md` §5.15 guarantee 5 still says it, and must NOT be quietly fixed.** The Survey
reports that file **ENTRENCHED** — reserved to the project lead specifically, not any maintainer
(§10, invariant 44). It is recorded as `REMAINING_WORK.md` §7.10a with the precedent that would
settle it (`ENTRENCHED_CHANGE_RECORD.md`, 2026-08-08 — a false honesty clause in `DELULU_CORE.md`
corrected *under owner approval*, deviation recorded). **Ask Jesse; do not edit it.** The `dist/`
archive copy is also deliberately untouched — a frozen record of what 1.0.0 shipped.

**Check `query <id>` for ENTRENCHED before editing any `docs/design/` file** — it prints before any
edge, which is how this was caught rather than committed.

Two generalizable lessons from the same pass. **A document's status belongs in the document:**
`LANGUAGE_SPECIFICATION.md` called itself *"the definitive design specification"* while
`REPOSITORY_STRUCTURE.md` had long called it superseded — a reader opening it directly was told the
opposite. Same failure mode as the morph spec claiming implemented when it wasn't. And **the
generated reference never carried the microVM claim at all**, because `docs/reference/` only records
rules with an enforcing diagnostic — the conformance machinery refuses an unenforced guarantee
automatically, which is worth remembering as a cross-check on prose.

See [[delulu-reproduce-the-shape]] for the methodology trap this pass hit, and
[[delulu-survey-map]] — the Survey caught four bad numbers in the first draft.
