<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::AUDIT_RULES` (cross-checked against `SOUNDNESS_AUDIT.md`).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# The audit rules

The soundness audit's rules R-1…R-7 — the properties the laundering suite exists to attack. Each rule's rejecting witness is an exploit attempt that must fail; its accepting witness is the legitimate program that must still pass.

The normative statement of each rule lives in `docs/design/SOUNDNESS_AUDIT.md`; this chapter records enforcement and coverage.

> **Coverage (invariant 42):** 7 of 7 anchors in this chapter have both an accepting and a rejecting conformance witness (100.0%). Items marked otherwise are **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.

| Rule | Anchor | Coverage |
|---|---|---|
| `R-1` | `ref.audit.R-1` | covered |
| `R-2` | `ref.audit.R-2` | covered |
| `R-3` | `ref.audit.R-3` | covered |
| `R-4` | `ref.audit.R-4` | covered |
| `R-5` | `ref.audit.R-5` | covered |
| `R-6` | `ref.audit.R-6` | covered |
| `R-7` | `ref.audit.R-7` | covered |
