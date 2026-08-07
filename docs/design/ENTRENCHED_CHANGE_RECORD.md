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
