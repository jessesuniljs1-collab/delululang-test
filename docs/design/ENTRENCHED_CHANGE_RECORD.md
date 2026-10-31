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

## 2026-10-17 — `SECURITY.md` §1 and §6, and `.github/CODEOWNERS`: every control decided, the owner named

**Changed**

| | |
|---|---|
| Path | `SECURITY.md` §1 (the reporting channel: the public repository's private reporting form, the only channel; no `security@` address or PGP key) and §6 (each `PENDING-PUBLIC` row decided: two-person review and signed commits **not adopted**, with reasons; SLSA **L2** live per release and **L3 only after** `gh attestation verify --signer-workflow` passes; Scorecard runs with no floor claimed; private reporting, secret scanning, push protection and dependency alerts live); `.github/CODEOWNERS` — the placeholder replaced by `@jessesuniljs1-collab` on its eleven lines |
| Entrenched by | `.github/CODEOWNERS` (`/SECURITY.md`); build-order D2 (nothing claimed live that is not) |
| Campaign | the owner's commission of 2026-10-17 (*"SECURITY.md"*, *"CODEOWNERS"* among the decisions it delegates); `HANDOFF.md` §1.1 items 3 and 4; RW 7.8 |
| Approved by | **the head chef, 2026-10-17, under the owner's delegation of that day** (D-V2-154) — the owner: *"You do not need to wait for my review or approval for any outstanding project decisions"*; its limit honoured: *"Never invent security contact details, cryptographic keys, verification results or compliance claims"* |

**Why**

The public repository exists and its controls are switched on (D-V2-153, read back), so "pending a publication" was no
longer true of any row: each became live, or not adopted with its reason. No address or key is listed because none would be
monitored; L3 is written as a condition because it has not been verified; two-person review is not claimed by a project
with one maintainer. `governance.rs`'s test was rewritten to hold the new line — no placeholder, no pending row, L3 a
condition — and was watched failing on the old text before the edit.

**Why this is not an RFC**

`SECURITY.md` is policy, not language: no rule, diagnostic or guarantee of the language moves. Every change narrows a
claim to what runs or names a real owner where a placeholder stood — build-order D2 strengthened, not relaxed.

---

## 2026-10-17 — `docs/design/CONSTITUTION.md` §5.14 item 3 and §5.15 guarantee 5: a layer counts only where it runs

**Changed**

| | |
|---|---|
| Path | `docs/design/CONSTITUTION.md` — §5.14's third layer (a sentence added) and §5.15's fifth guarantee (reworded) |
| Entrenched by | `.github/CODEOWNERS` (`/docs/design/CONSTITUTION.md`); §10 — §5.14's honesty limits |
| Campaign | the owner's commission of 2026-10-17 (`docs/design/DeluluLang_Autonomous_Development_Prompt.md`: *"Review `CONSTITUTION.md` §5.15 and correct any inaccurate claims about microVM containment or platform availability"*); REMAINING_WORK 7.10a |
| Approved by | **the head chef, 2026-10-17, under the owner's delegation of that day** (D-V2-141) — the owner: *"You do not need to wait for my review or approval for any outstanding project decisions"* |

**Why**

§5.15 guaranteed: *"Violations are contained at the WASM/microVM layer even if the type system is bypassed."* The
microVM layer exists only on Linux x86-64 with KVM (V2 PS-C, 2026-09-27); an ordinary `run` executes the interpreter
in-process with no operating-system boundary, and a WebAssembly floor holds only for a program run on that engine. So
the clause promised containment on hosts and paths where none is built. §5.14's third layer read as present on every
host in the same way. RW 7.10a recorded both and left them for the owner, because the file is entrenched; the owner's
commission delegated exactly this correction.

**Why this is not an RFC**

§10: the Constitution changes by RFC, and §5.14's honesty limits only by a replacement that *preserves or strengthens*
the whole-program authority guarantee. This change adds no capability, removes no guarantee that holds, and narrows two
claims to what is built — it strengthens the honesty limits, and the guarantee is unchanged where it held. The precedent
is this record's first entry (2026-08-08): a binding honesty clause found factually false was corrected under the
project lead's approval with the deviation recorded, not left false through a comment period. The same three sites in
non-entrenched documents (README's Honesty section, QUESTIONS §1.3/§2.1, the Book Ch. 14/15/20) were corrected on
2026-08-23 and already say what this now says.

**Verified before the change**

The layers as the binary states them: `delulu run --help` names `--engine wasm` (the WebAssembly backend), `--sandbox`
(the jailed guest), `--isolation none|process|microvm` (`process` isolates foreign code only) and `--sandbox-backend
external:CMD` (L3); `doctor --check` on this Windows host reports `L1 — L2 is absent` and KVM absent; REMAINING_WORK
4.1 and README's Honesty section record the microVM as built on Linux x86-64 with KVM only. The conformance rules for
§5.15 (`delulu-conform/src/rules.rs`) cite guarantees 1–6 by statement, not this sentence; `governance.rs` reads the
file's structure, not this clause.

**What makes this hard to undo**

Nothing: a two-sentence documentation revert.

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

---

## 2026-09-28 — `docs/design/DELULU_CORE.md` v0.3: the fault configuration, progress-or-fault, and the higher-order primitive

**Changed**

| | |
|---|---|
| Path | `docs/design/DELULU_CORE.md` — the title (v0.2 → v0.3), one clause of the Status block, a v0.3 note, §1 (`hop`), §3 (`T-HOp`; Fact A), §4 (`fault(c)`, `E-Refuse`, `E-HOp`, `E-Fault`, `permits`), §7 (Theorems 1–3), §8 (three traceability rows), §9 (a "repaired in v0.3" line on each of the two finding boxes) |
| Entrenched by | `.github/CODEOWNERS:17` → `@PENDING-PUBLIC-project-lead` |
| Campaign | V2 P7, `REMAINING_WORK.md` 5.2 and 5.3; decision D-V2-49 |
| Approved by | **The owner's delegation of 2026-09-28** (`docs/CLOUD_ROUTINE.md`, *The authority a run has*): *"any file or folder is allowed to modified or created or even deleted"*, with RW 5.2 named in the routine as allowed and to be recorded here. Taken by the head chef in the first cloud routine run, **flagged for the owner's review** on his return. |

**Why**

The proof campaign found two defects in v0.2, both recorded in the document's own §9 since
2026-08-04 and neither repaired: **P17-T2**, Theorem 1 (Progress) is false as stated — a present,
well-typed capability whose scope does not cover its argument makes `E-Op` inapplicable and no rule
applies, while the runtime faults `DL0904` — and **P17-T1**, the calculus has no construct for a
primitive that invokes a function argument, so mechanizing §1–§7 as written would prove the wrong
theorem (C88). Both are prerequisites for RW 5.1, the mechanization. v0.3 adds the fault
configuration and the rules that reach it, restates Progress as progress-or-fault, and adds the
higher-order primitive with the typing rule the Lean fragment already proves sound (`good_sound`),
generalized from a literal closure to any term of arrow type.

**Why this is not an RFC**

`CONTRIBUTING.md` §5 sends changes to *the language core* through `rfcs/`. **The language does not
move:** no surface rule, diagnostic, exit code, artifact format or `STABILITY.md` line changes, and the
checker and runtime are untouched. The calculus is corrected to describe the language as it is
implemented, which is what §6 says it is for ("the calculus faithfully models the implemented
language"). Both behaviours it now models were witnessed on the current binary before the edit
(below). A reader who holds that the calculus IS the language core would want an RFC; that reading is
the owner's to take on his return, and the revert below is one command.

**Verified before the change**

1. **E-Refuse is what the runtime does**, run on the binary built from `fef8ccd`: a program granted
   `fs.read=./data` that reads `data/in.txt` and then `../outside.txt` checks clean; the run prints
   the first, then ends `error[DL0904]: path ../outside.txt escapes the granted scope`, exit 1, and the
   program's own `Err` arm never runs (a fault, not a result). `--trace-effects` holds a `Read` record
   for the refused operation (seq 2) — the trace records the ATTEMPT — so `E-Refuse` emits the label.
2. **T-HOp is what the checker does:** a function declared pure whose body passes a callback
   declared `! {Write}` to `List.map` is refused `DL0501` ("performs effect `Write` not declared in its
   row"); declared `! {Write}`, it checks clean and `delulu authority` reports `Write`.
3. **The blast radius is zero:** `delulu-survey impact doc:docs/design/DELULU_CORE.md` → **0 nodes
   reached**; eight lines link to the file, none to a section this edit renumbers (no section moves).
4. **Lean was not re-run:** the cloud VM has no Lean toolchain, and `DeluluCore.lean` is unchanged, so
   its 2026-08-08 result (above) stands for it; v0.3's new rules are paper, like the rest of §1–§7.

**What did NOT change, and is the honest residue**

Nothing new is machine-checked. §1–§7 — now including faults — remain paper sketches; RW 5.1 (a Lean
or Coq development of the whole calculus) is still open, and is now aimed at a calculus that states
the construct that failed (C88) and the outcome the runtime actually has. §9's two finding boxes are
kept, each with a line saying it was repaired in v0.3. P17-T3 (§6's faithfulness claim) is untouched.

**How to revert**

```
git checkout <this-commit>^ -- docs/design/DELULU_CORE.md
cargo run -p delulu-survey -- build
```

---

## 2026-10-14 — `tests/conformance/witnesses.toml`: the anchor `ref.prim.http.get_with`

**Changed**

| | |
|---|---|
| Path | `tests/conformance/witnesses.toml` — four `[[witness]]` rows for the new anchor `ref.prim.http.get_with`, after `ref.prim.http.get`'s two, with a three-line comment. No existing row is touched. |
| Entrenched by | `.github/CODEOWNERS:37` → `@PENDING-PUBLIC-project-lead` |
| Campaign | V2 P9-04 step (a), `REMAINING_WORK.md` 2.11; decision D-V2-128 |
| Approved by | **The owner's delegation of 2026-09-28** (`docs/CLOUD_ROUTINE.md`, *The authority a run has*): *"any file or folder is allowed to modified or created or even deleted"* — the routine names the conformance witnesses among the entrenched files a run may edit, recorded here. Taken by the head chef in routine run 23, **for the owner's review** on his return. |

**Why**

The coverage law (invariant 42) gives every `PRIM_TABLE` row an anchor, and `release_requires_full_coverage` demands an
accepting and a rejecting witness for each. P9-04 step (a) adds the row `http.get_with`; without these rows the reference
would carry an uncovered anchor (353 of 354) and the gate that requires 100% would refuse a release. The rows name the
table's own two witnesses, as every primitive has (`every_listed_primitive_resolves`, `every_listed_primitive_rejects_a_five_
argument_call`), and the egress client's two — the headers it must send and every header it must refuse.

**Why this is not an RFC**

A witness row maps an existing test to an anchor; it changes no rule, no diagnostic and no behaviour. The behaviour it covers —
a new primitive — is the language change, taken as D-V2-128 under the same delegation (`PRIM_TABLE_VERSION` 5 → 6, additive:
every version-5 primitive still exists and means the same thing). `COVERED_FLOOR` in `crates/delulu-conform/` is untouched.

**Verified before the change**

1. The four named tests exist and pass at `bee695f` (`cargo test -p delulu-check --lib prim_table`, 28 passed;
   `cargo test -p delulu-runtime --lib egress::`, 35 passed).
2. `cargo test -p delulu-conform` green; `delulu-conform --reference` regenerated `docs/reference/` (354 of 354 anchors, 100.0%)
   and `--check-reference` says in sync.

**How to revert**

```
git checkout bee695f^ -- tests/conformance/witnesses.toml
```
— together with D-V2-128's code, or the release gate refuses the uncovered anchor.
## 2026-10-14 — `tests/conformance/witnesses.toml`: the anchor `ref.prim.http.get_with_secret`

**Changed**

| | |
|---|---|
| Path | `tests/conformance/witnesses.toml` — four `[[witness]]` rows for the new anchor `ref.prim.http.get_with_secret`, after `ref.prim.http.get_with`'s four, with a three-line comment. No existing row is touched. |
| Entrenched by | `.github/CODEOWNERS:37` → `@PENDING-PUBLIC-project-lead` |
| Campaign | V2 P9-04 steps (b) and (c), `REMAINING_WORK.md` 2.11; decision D-V2-131 |
| Approved by | **The owner's delegation of 2026-09-28** (`docs/CLOUD_ROUTINE.md`, *The authority a run has*): *"any file or folder is allowed to modified or created or even deleted"* — the routine names the conformance witnesses among the entrenched files a run may edit, recorded here. Taken by the head chef in routine run 24, **for the owner's review** on his return. |

**Why**

The coverage law (invariant 42) gives every `PRIM_TABLE` row an anchor, and `release_requires_full_coverage` demands an
accepting and a rejecting witness for each. P9-04 steps (b) and (c) add the row `http.get_with_secret`; without these rows the
reference would carry an uncovered anchor (354 of 355) and the gate that requires 100% would refuse a release. The rows name the
table's own two witnesses (`every_listed_primitive_resolves`, `every_listed_primitive_rejects_a_five_argument_call`) and the
egress client's two — the secret sent on exactly the hops its audience and its origin allow, and refused before anything is
resolved outside its audience.

**Why this is not an RFC**

A witness row maps an existing test to an anchor; it changes no rule, no diagnostic and no behaviour. The behaviour it covers —
a new primitive and the grant form it reads — is the language change, taken as D-V2-131 under the same delegation
(`PRIM_TABLE_VERSION` 6 → 7, additive: every version-6 primitive still exists and means the same thing). `COVERED_FLOOR` in
`crates/delulu-conform/` is untouched.

**Verified before the change**

1. The four named tests exist and pass (`cargo test -p delulu-check --lib prim_table`; `cargo test -p delulu-runtime --lib
   egress::`, 40 passed).
2. `delulu-conform --reference` regenerated `docs/reference/` (355 of 355 anchors, 100.0%).

**How to revert**

```
git checkout 91a1eb7^ -- tests/conformance/witnesses.toml
```
— together with D-V2-131's code, or the release gate refuses the uncovered anchor.

## 2026-10-22 — `tests/conformance/witnesses.toml`: `ref.diag.DL1205`'s rejecting witness

**Changed**

| | |
|---|---|
| Path | `tests/conformance/witnesses.toml`: the `negative` `[[witness]]` row for `ref.diag.DL1205` names `a_secret_compiles_as_a_handle_and_what_would_read_it_stays_dl1205` in place of `minting_a_secret_alone_is_refused_dl1205`, in the same file (`crates/delulu-wasm/src/lib.rs`). No other row is touched. |
| Entrenched by | `.github/CODEOWNERS:33` → `@jessesuniljs1-collab` |
| Campaign | RW 3.4; decision D-V2-174 |
| Approved by | **the head chef, 2026-10-22, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"* — the entrenched files among them, each change with its `D-V2-nn` and this record. |

**Why**

D-V2-174 compiles `root.secret` to a handle the host keeps, so the test this row named, which pinned that minting a secret
alone was DL1205, was replaced by one that pins what stays DL1205: `Secret.map` and `Secret.expose`, with `root.declassify()`.
The row went on naming the removed test, and the coverage law refused the dangling witness: PR #48's run `39825874508`,
`test (ubuntu-24.04)`, with `coverage_never_regresses`, `release_requires_full_coverage` and
`the_generated_reference_is_not_stale` red. The laptop's run of the change had not included `delulu-conform`.

**Why this is not an RFC**

A witness row maps an existing test to an anchor; it changes no rule, no diagnostic and no behaviour. DL1205 still means
that secret contents cannot enter the WASM guest, and the test the row now names produces it twice. The narrowing of what
DL1205 refuses is the language change, taken as D-V2-174. `COVERED_FLOOR` in `crates/delulu-conform/` is untouched.

**Verified before the change**

1. The named test exists and passes (`cargo test -p delulu-wasm --lib a_secret_compiles_as_a_handle`).
2. `delulu-conform --reference` regenerated `docs/reference/` with no difference from what is committed, and
   `cargo test -p delulu-conform` is green.

**How to revert**

Together with D-V2-174's code. Reverting the row alone would name a test that no longer exists, which the coverage law
refuses.

## 2026-10-22 — `docs/design/DELULU_CORE.md` v0.4: Theorems 1–3 machine-checked for a fragment; `T-Verify` and `E-Expose` stated as implemented

**Changed**

| | |
|---|---|
| Path | `docs/design/DELULU_CORE.md` — the title (v0.3 → v0.4), the Status block's honesty clause, a v0.4 note, §1 (`narrow`), §3 (`T-Verify` carries `Declassify`; `T-Narrow`; Facts A and B), §4 (`E-Expose`, `E-Verify`, `E-Narrow` and `E-NarrowRefuse` written), §7 (Theorem 3's capability-witness clause names `verify`'s label as the exception), §9 (a box for the new development; the P17-9 box's "what remains" marked as v0.3's; the closing paragraph's pointer to `CONTRIBUTING.md`, which never recorded the work, replaced by RW 5.1). |
| Entrenched by | `.github/CODEOWNERS:13` → `@jessesuniljs1-collab` |
| Campaign | RW 5.1; decision D-V2-175 |
| Approved by | **the head chef, 2026-10-22, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"*, the entrenched files among them, each change with its `D-V2-nn` and this record. |

**Why**

`models/lean/DeluluCoreSafety.lean` machine-checks Theorems 1–3 and the No-forgery corollary for the monomorphic,
closed-row fragment of §1–§4, attenuation included. The
Status block's "§1–§7 as a whole remain unmechanized — no capabilities, no store, no secrets, … no Progress and no
Preservation" stopped being true, and the honesty clause binds in both directions. Mechanizing also found two places where
the calculus did not state the implemented language, which §6 says it models faithfully:
- **`T-Verify` gave `verify` no label.** Since campaign finding P17-IF1 the checker types `verify` as carrying `Declassify`,
  and the runtime traces it so.
- **`E-Expose` was never written.** `T-Expose` emitted a label that no reduction rule produced.
- **Attenuation was relied on and never stated.** §4 said tokens are created only by `narrow`, and the No-forgery
  corollary rests on it, but `narrow` was not a term and had no rule. It has `T-Narrow`, `E-Narrow` and `E-NarrowRefuse`
  now, with the one law the corollary needs: a narrowed scope is under its parent's.

Stating `T-Verify` as implemented makes Fact B's "a label demands a capability" and Theorem 3's capability-witness clause
false for that one label: `verify` declassifies without `Cap[Declassify]`, the R-5 residue already recorded as RW 2.6. v0.4
says so where those claims are made, rather than keeping a rule that hid it.

**Why this is not an RFC**

The language does not move: no surface rule, diagnostic, exit code, artifact format or `STABILITY.md` line changes, and the
checker and runtime are untouched. The calculus is corrected to describe the language as implemented, as v0.3 was. The
mechanization takes `verify`'s label as a parameter and proves the theorems for v0.3's rule and v0.4's alike.

**Verified before the change**

1. **`T-Verify` is what the checker does:** a function declared pure that calls `a.verify(b)` is refused `DL0501`
   ("function `same` performs effect `Declassify` not declared in its row").
2. **`E-Verify` is what the runtime does:** `--trace-effects` records `verify` as `"effect":"Declassify"`, detail `«opaque»`.
3. **The proofs:** `lean DeluluCoreSafety.lean` (Lean 4.32.2) checks, and `#print axioms` lists only `propext`,
   `Classical.choice` and `Quot.sound`. A planted `sorry` is caught by the new CI gate, and the three mutants (`T-Op`'s
   label, `T-HOp`'s latent row, the narrowing law weakened) each fail to check.
4. **The blast radius is zero:** `delulu-survey impact doc:docs/design/DELULU_CORE.md` reaches 0 nodes. No section is
   renumbered.

**What did NOT change, and is the honest residue**

Type variables and open rows (§5), and `Σ`'s internals, stay paper. RW 2.6 is stated, not closed.
`DeluluCore.lean` is unchanged.

**How to revert**

```
git checkout <this-commit>^ -- docs/design/DELULU_CORE.md
cargo run -p delulu-survey -- build
```

## 2026-10-23 — `tests/conformance/witnesses.toml`: the anchors `ref.prim.http.post` and `ref.prim.http.post_with_secret`

**Changed**

| | |
|---|---|
| Path | `tests/conformance/witnesses.toml` — four `[[witness]]` rows for the new anchor `ref.prim.http.post`, after `ref.prim.http.get_with_secret`'s four, with a three-line comment, and four for `ref.prim.http.post_with_secret` after them, with a two-line comment. No existing row is touched. |
| Entrenched by | `.github/CODEOWNERS:33` → `@jessesuniljs1-collab` |
| Campaign | V2 P9-05 (d), `REMAINING_WORK.md` 2.12; decision D-V2-183 |
| Approved by | **the head chef, 2026-10-23, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"* — the entrenched files among them, each change with its `D-V2-nn` and this record. |

**Why**

The coverage law (invariant 42) gives every `PRIM_TABLE` row an anchor, and `release_requires_full_coverage` demands an
accepting and a rejecting witness for each. D-V2-183 adds the row `http.post`, and without these rows the reference would
carry uncovered anchors. D-V2-183 adds `http.post_with_secret` too, with the same four kinds of rows. The rows name
the table's own two witnesses, as every primitive has
(`every_listed_primitive_resolves`, `every_listed_primitive_rejects_a_five_argument_call`). They also name the egress
client's POST, with its body kept or dropped by each redirect's status
(`a_post_sends_its_body_by_its_method_and_a_redirect_keeps_or_drops_them_by_its_status`), and the judge that refuses a
POST under an entry naming no methods (`netscope`'s `an_entry_judges_its_host_then_its_path_then_its_method`). For
`post_with_secret` they name the egress client's POST carrying its secret and body inside the audience only
(`a_post_carries_its_secret_and_its_body_only_inside_the_audience`), and the secret refused outside its audience before
anything is resolved (`a_secret_outside_its_audience_is_refused_before_anything_is_resolved`, the rule both share).

**Why this is not an RFC**

A witness row maps an existing test to an anchor; it changes no rule, no diagnostic and no behaviour. The behaviour it
covers, a new primitive, is the language change, taken as D-V2-183 under the same delegation. `PRIM_TABLE_VERSION` goes
from 7 to 8, an additive change: every version-7 primitive still exists and means the same thing. A grant naming no
methods still allows GET alone (D-V2-182), so no existing grant reaches a POST. `COVERED_FLOOR` in
`crates/delulu-conform/` is untouched.

**Verified before the change**

1. The named tests exist and pass on the laptop: `cargo test -p delulu-check --lib prim_table` (11 passed, the
   pinned count 86), `cargo test -p delulu-runtime --lib egress` (44 passed, 1 ignored), and
   `cargo test -p delulu-broker --lib netscope`.
2. `delulu-conform --coverage` reported the one gap (`ref.prim.http.post — no accepting test, no rejecting test`)
   before the rows and **357 of 357, 100.0%** after both anchors' rows. `cargo test -p delulu-conform` is green. `--reference`
   regenerated `docs/reference/` (README, coverage, primitives), and `--check-reference` says in sync (24 chapters).

**How to revert**

```
git checkout <this-commit>^ -- tests/conformance/witnesses.toml
```
— together with D-V2-183's code, or the release gate refuses the uncovered anchor.

## 2026-10-24 — `crates/delulu-check/tests/laundering.rs`: five GENERIC-RIGID-1 witnesses; a witness must parse

**Changed**

| | |
|---|---|
| Path | `crates/delulu-check/tests/laundering.rs`: `rejects_with` also asserts the witness lexes and parses (no DL01xx or DL02xx). Two statements on one line are separated in F-3 (`cb(); cb()`), F-3b (`f(); g()`) and F-4 (`out.println(str(x)); x`). Five tests appended: `generic_rigid_a_body_cannot_decide_its_type_parameter`, `generic_rigid_an_effectful_closure_cannot_be_coerced_pure`, `generic_rigid_a_body_cannot_close_its_row_parameter`, `generic_rigid_an_actor_body_cannot_decide_its_parameter`, `generic_rigid_honest_generics_still_check`. |
| Entrenched by | `.github/CODEOWNERS:28` → `@jessesuniljs1-collab` |
| Campaign | the agent test campaign of 2026-10-24; `REMAINING_WORK.md` 4.81; decision D-V2-184 |
| Approved by | **the head chef, 2026-10-24, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"* |

**Why**

The suite asks one question: can authority escape the type? A generic answered yes (GENERIC-RIGID-1), and the suite had
no witness for it. Writing one showed that three of its own witnesses never parsed. They still met their codes, through
the parser's recovery, so a refusal could have come from the repair rather than the rule.

**Nothing is weakened.** No witness is removed or relaxed. The three repaired programs are now the programs their
comments describe, and each is still refused with its own code (DL0401, DL0504, DL0501). The new assertion makes every
witness, present and future, prove it parses.

**Verified before the change**

1. `cargo test -p delulu-check --test laundering`: with the stricter helper and the old programs, F-3, F-3b and F-4
   failed ("does not lex or parse cleanly: DL0209 …"). Repaired, 17 passed.
2. Mutants of D-V2-184's code turn the new witnesses red (see the decision).

**How to revert**

```
git checkout <this-commit>^ -- crates/delulu-check/tests/laundering.rs
```
— together with D-V2-184's code, or the generic witnesses fail.

## 2026-10-27 — `crates/delulu-conform/src/rules.rs` and `tests/conformance/witnesses.toml`: DL0908, an argument outside what the operation accepts

**Changed**

| | |
|---|---|
| Path | `crates/delulu-conform/src/rules.rs`: the rule `ref.rule.runtime.faults-are-diagnostics` names *"an argument outside what an operation accepts"* among runtime faults in its statement, and `DL0908` in its `enforced_by`. `tests/conformance/witnesses.toml`: two `[[witness]]` rows for the new anchor `ref.diag.DL0908`, a rejecting one after DL0907's and an accepting one after DL0907's. No existing row or rule is touched otherwise. |
| Entrenched by | `.github/CODEOWNERS:32` and `:33` → `@jessesuniljs1-collab` |
| Campaign | the agent test campaign of 2026-10-24 (A15); `REMAINING_WORK.md` 6.25; decision D-V2-190 |
| Approved by | **the head chef, 2026-10-27, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"*, a new `DL` code where a refusal needs one among them |

**Why**

D-V2-190 registers DL0908 for `r.int(lo, hi)` with `hi <= lo`, which both engines had reported as DL0904, a capability
scope violation. The coverage law (invariant 42) demands an accepting and a rejecting witness for every registered code,
so without these rows the reference would carry an uncovered anchor. The rows name `fault_parity`'s
`an_empty_rand_range_is_dl0908_on_both_engines` (rejecting; its body names `DL0908`, as C37 requires) and
`a_one_wide_rand_range_draws_its_one_value_on_both_engines` (accepting). The rule is the one that says a runtime fault is a
diagnostic with a code. DL0908 is such a fault, so leaving it out would make the rule's list incomplete.

**Why this is not an RFC**

`STABILITY.md` §5 lists a new diagnostic code as a minor, additive change. No rule changes meaning; one rule's list of
codes grows by the new one. The fault moves from DL0904 to DL0908 in a major release (2.0.0), and DL0904 keeps its
identity: an operation reaching outside its capability's scope.

**Verified before the change**

1. The rejecting witness was red on the unchanged engines (`WASM trap: DL0904: rand.int requires lo < hi`) and is green
   with D-V2-190's code. The accepting witness is green on both.
2. `delulu-conform --coverage`: *PASS: 100% anchor coverage*. `--reference` regenerated the reference: 358 of 358
   anchors, 155 diagnostics.
3. Mutants M855–M858 of D-V2-190's code turn the rejecting witness red (see the decision).

**How to revert**

```
git checkout <this-commit>^ -- crates/delulu-conform/src/rules.rs tests/conformance/witnesses.toml
```
— together with D-V2-190's code and the registry entry, or the coverage gate fails on an unwitnessed DL0908.

## 2026-11-01 — `docs/design/SOUNDNESS_AUDIT.md`: §D's claim negated a second time (LATE-LET-1)

**Changed**

| | |
|---|---|
| Path | `docs/design/SOUNDNESS_AUDIT.md`: one box added at the end of §D, after *"the tests are the actual guarantee"*. It records that the claim was negated by the runtime's name resolution, gives the program, and names the witnesses. No existing sentence changes. |
| Entrenched by | `.github/CODEOWNERS:27` → `@jessesuniljs1-collab` |
| Campaign | the weekly verification sweep of 2026-11-01; `REMAINING_WORK.md` 2.23; decision D-V2-199 |
| Approved by | **the head chef, 2026-11-01, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"* |

**Why**

The audit's §D claims that in a well-typed program every runtime effect is in the static row of every function on the
stack, and it keeps C88's box saying so once already proved false. LATE-LET-1 proved it false again: the checker typed a
closure's callee as a pure function, and both engines called the effectful one a later `let` bound under the same name.
A document that names its claim must say when the claim failed, or the next reader relies on it (HANDOFF §11.8: *a
document that names a residual must be re-read when the code moves*).

**Why this is not an RFC**

Nothing in the language's meaning changes: the box records a defect in the runtime, now fixed, against the meaning the
checker already gave. No rule is added, removed or reworded.

**Verified before the change**

1. The program in the box checked clean, `authority` said *"(none — provably pure)"*, and `run --grant console`
   printed, on the binary before D-V2-199; after it, it prints nothing on either engine.
2. The witnesses named in the box are red before D-V2-199 and green after it (the decision's evidence).

**How to revert**

```
git checkout <this-commit>^ -- docs/design/SOUNDNESS_AUDIT.md
```

## 2026-11-01 — `docs/design/SOUNDNESS_AUDIT.md`: LOCAL-CALLEE-1 added to §D's box of the same day

**Changed**

| | |
|---|---|
| Path | `docs/design/SOUNDNESS_AUDIT.md`: one paragraph appended to the LATE-LET-1 box at the end of §D. It records a second negation of §D's claim found the same day (LOCAL-CALLEE-1) and names its witness. No other sentence changes. |
| Entrenched by | `.github/CODEOWNERS:27` → `@jessesuniljs1-collab` |
| Campaign | the weekly verification sweep of 2026-11-01; `REMAINING_WORK.md` 2.24; decision D-V2-200 |
| Approved by | **the head chef, 2026-11-01, under the owner's delegation of 2026-10-17** (his commission, `docs/design/DeluluLang_Autonomous_Development_Prompt.md`): *"You do not need to wait for my review or approval for any outstanding project decisions"* |

**Why**

§D's claim was negated again by the runtime's name resolution: a call the checker resolved to a local holding a pure
plugin export ran an effectful module function of the same name. The box already recording LATE-LET-1 is where a reader
of §D looks, and it must not stop at the first of the day's two.

**Why this is not an RFC**

No rule changes. The paragraph records a runtime defect, now fixed, against the meaning the checker already gave.

**Verified before the change**

1. On the binary before D-V2-200, the paragraph's program checked clean, `delulu why Write` said *"program cannot
   perform `Write`"*, and `run --grant console --grant plugin=.` printed. After it, nothing is printed.
2. The witness is red before D-V2-200 and green after it, and both mutants of the fix turn it red (the decision's
   evidence).

**How to revert**

```
git checkout <this-commit>^ -- docs/design/SOUNDNESS_AUDIT.md
```
