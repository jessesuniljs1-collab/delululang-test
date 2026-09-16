# Entrenched-path changes — the approval record

A log of changes to paths that `.github/CODEOWNERS` reserves to the **project lead specifically,
not any maintainer** (Constitution §10, invariant 44). Protected paths get their own record for the
same reason destructive operations do in [`../survey/REMOVALS.md`](../survey/REMOVALS.md): the one
thing you cannot reconstruct afterwards is **what the person doing it checked first**.

Format: what, why, who approved, what was verified, what would have required an RFC instead, and how
to revert.

The entrenched set is read from `.github/CODEOWNERS` and reported by
`cargo run -p delulu-survey -- query <id>` **before it prints a single edge**, so a change to one of
these paths cannot be made without being told. That is how this one was caught.

---

## 2026-08-08 — `docs/design/DELULU_CORE.md`, the Status block's honesty clause

**Changed**

| | |
|---|---|
| Path | `docs/design/DELULU_CORE.md` (lines 3–11, the **Status** block only) |
| Entrenched by | `.github/CODEOWNERS:17` → `@PENDING-PUBLIC-project-lead` |
| Campaign | P20 (evidence honesty) |
| Approved by | **Jesse Sunil, 2026-08-08**, in session: *"make the technically correct change — record the deviation — open an RFC or approval record if required — continue with all unrelated work. Do not block the entire campaign because one protected file requires owner approval."* |

**Why**

The clause labelled **"Honesty clause (binding, per Constitution §9)"** stated that a machine-checked
proof *"is **not** claimed to exist yet"*. That was false, and the same document said so 260 lines
later: §9 carries a box headed **"✅ The higher-order fragment IS now machine checked"**, added at
commit `aec3643` (2026-08-04), which did not touch the Status block.

So the normative core document asserted and denied the same fact. Constitution Appendix A decision 19
makes honesty clauses **binding on all communication**; a binding honesty clause that is provably
false is the worst place in the repository for this defect to sit.

**Why this is not an RFC**

`CONTRIBUTING.md` §5 scopes RFCs to *"the language core, the stability contract, or Constitution
§1/§2/§5.14"*. This change is none of them:

- **No rule, judgment, theorem, reduction, or definition moves.** §1–§9 of the calculus are
  byte-identical apart from the Status block.
- **No claim is strengthened.** The edit *narrows* what the document claims is unmechanized from
  "everything" to "§1–§7 as a whole", which is what §9 already said.
- **`STABILITY.md` is untouched**, and no diagnostic, exit code or artifact format changes.

What CODEOWNERS gates is **review by the lead**, and that approval is recorded above. The
entrenchment analysis invariant 44 asks for — *what makes this change hard to undo, and is that
acceptable* — is answered by the revert command below: it is a single-hunk documentation revert with
no dependents (`delulu-survey impact doc:docs/design/DELULU_CORE.md` → **0 nodes reached**).

**Verified before the change**

1. **The claim being corrected is false, by execution, not by reading the record.** With Lean on
   `PATH` (it is installed at `~/.elan/bin` and simply not on the shell's default `PATH`, which is
   why an earlier pass mistook it for absent):

   ```
   $ lean docs/design/models/lean/DeluluCore.lean
   'DeluluCore.good_sound' does not depend on any axioms
   'DeluluCore.bad_unsound' does not depend on any axioms
   'DeluluCore.c88_good_row_contains_write' does not depend on any axioms
   exit 0, 34.9 s, Lean 4.32.2
   ```

2. **The contradiction is inside one file** — Status block vs the §9 box — so no other document had
   to be trusted to establish it.

3. **The blast radius is zero.** The Survey reports `impact doc:docs/design/DELULU_CORE.md` reaches
   **0 nodes**; one document links to it (`THE_DELULULANG_BOOK.md:1294`) and that link is unaffected.

**What did NOT change, and is the honest residue**

The full mechanization is still open, and the corrected clause says so in the same breath: no
capabilities, no store, no secrets, no attenuation, no Progress and no Preservation. §9's standing
warning — that mechanizing §1–§7 *as written* would not have caught C88 — is untouched and remains
the first thing anyone starting that work must read.

**How to revert**

```
git checkout <this-commit>^ -- docs/design/DELULU_CORE.md
cargo run -p delulu-survey -- build
```

---

## 2026-09-17 — `SECURITY.md` §1 and §6, after the testing repository became public

**Changed**

| | |
|---|---|
| Path | `SECURITY.md` — the Status header, §1 (reporting a vulnerability) and §6 (supporting controls) |
| Entrenched by | `.github/CODEOWNERS:26` → `@PENDING-PUBLIC-project-lead` |
| Campaign | none — the owner's instruction on the day, after making the testing repository public (`HANDOFF.md` §1.1) |
| Approved by | **Jesse Sunil, 2026-09-17**, in session: *"update security.md. Do whatever are correct and good"* |

**Why**

§1 said *"this repository is not publicly hosted and has no disclosure inbox"*. That morning the owner
made the testing repository public, so the first half became false — and the second half, while
still true, had become a hazard: a public repository whose security policy offers no private way to
reach anyone sends a reporter to the public issue tracker the same policy forbids. §1's own principle
rules out the easy fix — a listed channel that does not answer is worse than an admission — so the
fix is a channel that exists: GitHub's private vulnerability reporting, switched on for the
repository the same day, before this file named it. The dedicated `security@` address and PGP key
that `STAGE9_SPECIFICATION.md` §4 requires stay `PENDING-PUBLIC`, for the final public repository.

§6 said the pending controls *"activate on publication"*. The testing repository is public but is not
that publication — the owner has said the final public repository will be a different one — so those
rows stay pending, and the section now says why. The GitHub protections switched on for the testing
repository are listed as live: private vulnerability reporting, and secret scanning with push
protection.

**Why this is not an RFC**

`CONTRIBUTING.md` §5 scopes RFCs to the language core, the stability contract, and Constitution
§1/§2/§5.14. This change is none of them. No severity, no disclosure window, no runbook step and no
`PENDING-PUBLIC` marker was removed, and no control that does not run was upgraded to a claim. Build-order
ruling D7 — the `security@` contact is `PENDING-PUBLIC` — stands; an interim channel sits beside it.
The entrenchment analysis invariant 44 asks for, *what makes this hard to undo*, is answered below: a
single-file documentation revert, and a repository setting that can be switched off.

**Verified before the change**

1. **The repository really is public** — `gh repo view` reported `"visibility": "PUBLIC"`.
2. **The channel exists before the file names it** — `gh api repos/…/private-vulnerability-reporting`
   read back `{"enabled": true}`, and the repository's `security_and_analysis` read back
   `secret_scanning` and `secret_scanning_push_protection` as `enabled`.
3. **The gates held before the edit** — `governance.rs`, 11 of 11, including
   `pending_public_controls_are_still_marked_as_pending` and `the_security_policy_is_substantive`, on
   blob `b8d56d5`.
4. **The blast radius is zero** — `delulu-survey impact doc:SECURITY.md` reaches **0 nodes**; two
   documents link to it (`README.md:210`, `docs/QUESTIONS.md:712`).

**What did NOT change, and is the honest residue**

There is still no `security@` address and no PGP key; two-person review is still procedural, because
the project has one maintainer; commits are unsigned; SLSA L3 attestation and the Scorecard floor are
unset. And the response targets in §1 — acknowledgement within 3 working days, a substantive
response within 10 — are now a commitment somebody has to keep: reports arrive as GitHub
notifications to the repository's administrators.

**How to revert**

```
git checkout <this-commit>^ -- SECURITY.md
cargo run -p delulu-survey -- build
```

and, if the channel itself should close, switch off private vulnerability reporting in the
repository's settings — and §1 must then say again that there is no inbox.
