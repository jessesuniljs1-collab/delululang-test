# DeluluLang RFCs

From 1.0 the language core is **change-frozen except through this process** (`STAGE9_SPECIFICATION.md`
§0, invariant 43).

## When you need one

| Change | RFC? |
|---|---|
| New syntax, typing rule, effect/authority behaviour | **yes** |
| Deprecating or removing anything | **yes** |
| Changing the stability contract | **yes** |
| Constitution §1, §2, or §5.14 honesty limits | **yes**, plus the entrenchment analysis (§6 of the template) |
| New diagnostic code for existing behaviour | no — but it needs its witnesses |
| Bug fix, doc fix, performance work, new tooling | no |

## The process

1. Copy `0000-template.md` to `NNNN-short-name.md`, next free number.
2. Open it for comment. **The period is ≥ 14 days**, and it does not shrink because a release is
   near — a deadline is not a reason to skip the part where people disagree with you.
3. Discussion happens in public.
4. **Disposition is recorded** in the RFC itself: accepted, rejected, or postponed, with the reason.
5. Accepted RFCs are implemented with their tests, their diagnostics, and their reference entries.
   Constitution Appendix A grows only through merged RFCs from here on.

## What gets an RFC rejected

Most often: it fails the **irreducibility analysis** (§3). If the thing can be a library, a lint, or
a convention, it does not go in the core. This is the constitution's central discipline and it is
meant to hurt — a language that accepts every good idea becomes a language nobody can hold in their
head, and every rule the checker must reason about is a rule that can have a fail-open branch.

## Open RFCs

| RFC | Title | Sponsor | Opened | Comment closes |
|---|---|---|---|---|
| [0001](0001-broker-federation.md) | Broker federation: a grant tree that spans machines | Jesse Sunil | 2026-07-22 | **2026-08-05** |

Closes `STAGE10_AUTONOMY_ADDENDUM.md` §2.5 and subsumes build-order D12e.

**Implementation status, stated plainly** (the RFC's own status line carries the long form):
phase **F1** (the device scope dimension) **shipped on 2026-07-22 *before* this RFC had a sponsor or
a comment period at all** — build-order ruling **D21(g)** records that deviation and it is not
reframed as compliance. Phases **F2–F6** are being built **concurrently with** the open comment
period at the sponsor's direction, which is a smaller deviation but still one. For every phase the
commitment is the same: **if this RFC is amended or rejected at the close, the code changes with
it.** Nothing is grandfathered by having landed first.
