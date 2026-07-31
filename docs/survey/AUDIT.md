# First Survey audit — 2026-08-01

**Hand-written companion to the generated [`DISCREPANCIES.md`](DISCREPANCIES.md).** That file lists
what the machine can check on every build. This one records what the *first* audit found, what was
done about each item, and — the part a generated file cannot hold — what was deliberately left
alone and why.

Method: the Survey was built, every finding it produced was examined individually against the
source, and each was classified as a defect in the repository or a defect in the Survey. Both kinds
were found. Both are recorded here.

---

## 1. Fixed

### 1.1 A security-relevant pin justified by a premise that had gone false

`crates/delulu-runtime/Cargo.toml` pinned the two post-quantum crates exactly (`=0.1.1`, `=0.3.2`)
and explained why:

> `Cargo.lock` is gitignored in this repo, so a fresh clone resolves whatever satisfies the
> requirement…

**`Cargo.lock` is tracked.** It has been since ruling S10-D19c, and `.gitignore` says so in as many
words: *"Cargo.lock IS tracked (ruling D19c) … Do not re-add it here by reflex."* So the repository
contained a live comment and a live ignore-file that flatly contradicted each other, and the one
that was wrong was the one justifying how an *unaudited lattice implementation* is allowed to
change.

The pins are still right, for a reason that outlived the reason originally given: a lockfile binds
*this* build, while a `=` requirement binds every consumer of the crate and survives `cargo update`.
The comment now says that instead. **The decision did not change; its stated justification became
true.**

### 1.2 Front-door numbers that had drifted 14%

| Where | Said | Actually |
|---|---|---|
| `README.md:34` | ~82,000 lines of Rust | ~94,000 |
| `README.md:34` | 93 test suites, 1,190 tests | 102 suites, 1,361 tests |
| `docs/design/HARDENING_CAMPAIGN.md:57` | ~82,000 lines, 12 crates, 175 files | ~94,000 lines, 12 crates, 194 files |

The crate count was the only one still true. All are corrected, and the size figures are now
recounted from the tree on every build — a wrong one fails a test rather than sitting quietly.

Test counts are **not** machine-checked, because the Survey reads files and `cargo test` produces
that number. It reports where such counts are quoted so a human knows what to re-check after a run,
and it does not guess. The 102/1,361 above is from an actual full-workspace run on Windows.

### 1.3 A repository map whose dependency graph showed five of twelve crates

`docs/REPOSITORY_STRUCTURE.md` §2 drew the Stage-1 spine — `diag → syntax → check → runtime → CLI`
— under the heading "Crate dependency graph". By Stage 10 the workspace held twelve crates;
`delulu-broker`, `delulu-atlas`, `delulu-wasm`, `delulu-registry` and the rest appeared nowhere.

Worth noting *how* this survived: the C5 re-synchronization in 2026-07 explicitly re-checked §1
against the tree and fixed it, and did not touch §2. A map rots in the section nobody re-reads.
§2 now points at the generated graph and keeps the Stage-1 spine labelled as the historical subset
it always was.

### 1.4 `docs/for-agents.md` advertised the wrong version

The JSON envelope example on the page that calls itself part of the stability contract showed
`"delulu_version": "0.1.0"`. The CLI emits `env!("CARGO_PKG_VERSION")`, which is `1.0.0`. Corrected.

---

## 2. Reported, deliberately not fixed

### 2.1 Ruling numbers collide across stages, and a documented convention resolves them

Ruling ids are allocated **once per stage**. `STAGE9_BUILD_ORDER.md` allocates D1–D22 and
`STAGE10_BUILD_ORDER.md` allocates D1–D66, so every number from 1 to 22 names two unrelated
decisions. **232 citations rely on this being resolved by convention rather than by writing it
down.**

The convention exists and is documented in two places:

- `CHANGELOG.md:9` — "Rulings live in `STAGE10_BUILD_ORDER.md` (`D<n>`) and, for Stage 9,
  `STAGE9_BUILD_ORDER.md` (`S9-D<n>`)."
- `STAGE10_BUILD_ORDER.md:28` — "Rulings ledger (this stage's namespace; Stage-9 rulings are cited
  as `S9-D<n>`)."

So a bare `D<n>` means the latest stage, and `S9-D21` is written when Stage 9 is meant — five times
so far. The Survey follows that rule rather than second-guessing it, and reports only how much
weight it silently carries.

The risk it guards against is not hypothetical: a Stage-9 ruling was recorded as `S9-D21`
specifically because it kept being confused with the Stage-10 `D21` that shipped device-scoped
delegation.

**Not changed.** Rewriting 232 citations across historical records to be explicit would be a large
mechanical edit whose failure mode — attaching a decision to the wrong stage — is worse than the
thing it fixes. What is worth knowing is that the convention is stated in a changelog preamble and
a ledger heading, and nowhere a reader of the specs or the campaign would meet it. **Recommendation:
restate it where citations are actually read.**

### 2.2 Stages 6, 7 and 8 allocate no citable rulings

`STAGE6_BUILD_ORDER.md` records decisions as unnumbered "head-chef ruling" prose; Stages 7 and 8
record none at all. A decision taken during those stages cannot be cited the way a Stage-9 or
Stage-10 one can. Reported as a note. Whether those stages genuinely took no recorded decisions is
a question for the owner, not for a checker.

### 2.3 Diagnostic codes that are named but not allocated

Seventeen `DLxxxx` tokens appear in the tree without being in the registry. Every one was examined
and every one is intentional: codes explicitly **retired** (`DL0503`, `DL0702`, `DL0906` — "retired
rather than frozen unreachable"), codes deliberately skipped so a range reads cleanly (`DL1404`,
`DL1609`), range endpoints in prose (`DL1780–DL1799`), and one deliberate non-member asserting that
the registry's own membership check is tight (`DL9999`).

So this is a **note, not an error** — the first version of the check called them errors, which
would have trained every reader to ignore the list.

What is genuinely missing is a *record*. Nothing distinguishes "retired on purpose" from "typo"
except the sentence next to it, so a newcomer or a model reading `DL0503` in a comment cannot tell.
**Recommendation for the owner:** a `RETIRED` list beside `REGISTRY` in
`crates/delulu-diag/src/codes.rs`. Not done here — it touches the diagnostics core, and the
existing behaviour is correct.

### 2.4 A detached agent worktree holding a second copy of the repository

```
.claude/worktrees/agent-a4541bfe777803515   51a378a  [worktree-agent-a4541bfe777803515]
```

A registered git worktree, on its own branch, pinned at a Stage-7 commit from **2026-07-18 —
96 commits behind `master`**. It is hidden from `git status` by `.git/info/exclude`, which is why
it has gone unnoticed.

It matters more than it looks: it is a complete second copy of the tree at a different revision,
and any tool that walks the repository without excluding it will double every file and silently mix
two revisions of the same document into one result. The Survey excludes it explicitly and says why.

**Not removed.** Deleting a worktree and its branch is destructive and irreversible, and it is the
owner's call. To remove it:

```
git worktree remove .claude/worktrees/agent-a4541bfe777803515
git branch -D worktree-agent-a4541bfe777803515
```

---

## 3. What the audit found about the Survey itself

Every finding in the first run was examined before any of it was believed, and **most of the first
run was wrong**. Recorded because a map's credibility rests on how it behaves when it is the thing
that is mistaken.

| First run said | Truth |
|---|---|
| 35 broken links | **0.** All were Delulu and Rust generic syntax inside code spans — `fn map[T, U, e](xs: List[T])` contains the exact `](` that opens a Markdown link target. |
| 9 missing paths | **0.** All exist. A comment in `crates/delulu/src/` writing `tests/cli.rs` means that crate's tests, and a resolver that only tries the repository root is simply wrong about the repository. |
| 17 unregistered codes | Real observation, **wrong severity** — every one is deliberate (§2.3). |
| 16 workspace members | **13.** A `#` comment inside the `members` list was read as three more crates — and the comment in question was the one introducing the Survey. |
| 4 broken links to `docs/reference/` | **0.** The path index knew files and not directories. |

**And this audit itself overstated a finding.** §2.1 was first written as *22 warnings — a reader
cannot tell which decision is meant*, on the strength of the collision alone. The repository does
document a convention, in `CHANGELOG.md` and in the Stage-10 ledger heading, and it is used. The
check now follows that convention and reports one note where it had reported twenty-two warnings.
Recorded because the failure was the same one the Survey exists to prevent: **an unverified reading,
stated confidently.** Nothing about the finding was wrong except its confidence, and confidence is
what makes a report worth acting on or worth ignoring.

Two further defects were caught by the Survey's own tests rather than by inspection:

- **`pub(crate) mod x;` was not recognised as a module declaration**, which would have reported
  perfectly reachable files as unreachable.
- **The Survey read its own output.** `DISCREPANCIES.md` quotes the paths it reports on, so each
  run found new relations inside the report the previous run had written. It would never have
  reached a fixed point, and the staleness gate would have failed forever while being entirely
  correct to do so. `the_map_reaches_a_fixed_point` exists because of this and would fail if it
  returned.

Each of these is now a named regression test. The general lesson is the one the campaign already
records: **a check's value is decided in the branch where it could not tell**, and a checker that
reports confidently on what it has misread is worse than no checker, because people believe it.

---

## 4. Standing recommendations

1. **Restate the ruling-citation convention where citations are read** (§2.1) — it currently lives
   only in a changelog preamble and a ledger heading, while 232 citations depend on it.
2. **Consider a `RETIRED` list in the code registry** (§2.3) — owner's call.
3. **Decide what to do about the stale worktree** (§2.4) — owner's call, destructive either way.
4. **Regenerate the Survey with any change that moves the tree.** `cargo test --workspace` already
   enforces this; the remedy is `cargo run -p delulu-survey -- build`.
