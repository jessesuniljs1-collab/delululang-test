<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from a live `delulu-conform --coverage` run.
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# Conformance coverage

Invariant 42: *the conformance suite is the specification's executable form.* This chapter is the machine's own account of how much of the reference is executable today. It is generated from a live coverage run — it cannot flatter itself.

**328 of 329 anchors (99.7%)** carry both an accepting and a rejecting witness.

| Chapter | Anchors | Covered |
|---|---|---|
| diag | 153 | 152 |
| prim | 59 | 59 |
| grammar | 27 | 27 |
| audit | 7 | 7 |
| cli | 29 | 29 |
| rule | 54 | 54 |

A `rule` anchor is covered only when *every* diagnostic that enforces it is covered in both directions, so this row is the strictest of the six.

## The 1 open anchors

Each is classified in `docs/design/STAGE9_BUILD_ORDER.md` D10. Release criterion 1 requires this list to be empty.

| Anchor | Missing |
|---|---|
| `ref.diag.DL0212` | accepting |
