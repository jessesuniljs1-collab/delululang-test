<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from a live `delulu-conform --coverage` run.
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# Conformance coverage

Invariant 42: *the conformance suite is the specification's executable form.* This chapter is the machine's own account of how much of the reference is executable today. It is generated from a live coverage run — it cannot flatter itself.

**263 of 287 anchors (91.6%)** carry both an accepting and a rejecting witness.

| Chapter | Anchors | Covered |
|---|---|---|
| diag | 123 | 106 |
| prim | 53 | 53 |
| grammar | 26 | 26 |
| audit | 7 | 7 |
| cli | 24 | 24 |
| rule | 54 | 47 |

A `rule` anchor is covered only when *every* diagnostic that enforces it is covered in both directions, so this row is the strictest of the six.

## The 24 open anchors

Each is classified in `docs/design/STAGE9_BUILD_ORDER.md` D10. Release criterion 1 requires this list to be empty.

| Anchor | Missing |
|---|---|
| `ref.diag.DL0203` | rejecting |
| `ref.diag.DL0408` | rejecting |
| `ref.diag.DL0503` | rejecting |
| `ref.diag.DL0601` | rejecting |
| `ref.diag.DL0701` | rejecting |
| `ref.diag.DL0702` | rejecting |
| `ref.diag.DL0905` | rejecting |
| `ref.diag.DL0906` | rejecting |
| `ref.diag.DL1101` | rejecting |
| `ref.diag.DL1102` | rejecting |
| `ref.diag.DL1204` | rejecting |
| `ref.diag.DL1206` | rejecting |
| `ref.diag.DL1304` | rejecting |
| `ref.diag.DL1307` | rejecting |
| `ref.diag.DL1610` | rejecting |
| `ref.diag.DL1701` | rejecting |
| `ref.diag.DL1702` | rejecting |
| `ref.rule.authority.no-forgery` | rejecting |
| `ref.rule.authority.root-is-the-only-source` | rejecting |
| `ref.rule.compilation.artifacts-carry-their-authority` | rejecting |
| `ref.rule.compilation.engines-agree` | rejecting |
| `ref.rule.effects.one-row-variable` | rejecting |
| `ref.rule.runtime.faults-are-diagnostics` | rejecting |
| `ref.rule.runtime.trace-is-within-the-row` | rejecting |
