---
name: delulu-p22-containment-campaign
description: "P22 overnight campaign 2026-08-10 (Opus 5) — SYMLINK-DANGLE-1 (dangling symlink escaped fs containment, HIGH), ROOTPOLICY-1 (strict root mode failed OPEN on a corrupt policy), CORE-SNAPSHOT-1 (the prior campaign closed claiming green while core_invariance was red)"
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-10T18:21:12.442Z
---

**START HERE with the 2026-08-09 production-readiness file for any DeluluLang session from
2026-08-10 on.** Jesse's commission: 4 phases (A discovery / B hardening / C adversarial / D final
report), autonomous overnight, 2-minute checkpoints with auto-continue, *"what security assumption
haven't we attacked yet?"* as the standing question.

**The commission's premise was STALE and that was caught at the door.** It was written as if DISC-1
were unresolved and instructed that opt-in strict anchored-root mode be built. It shipped 2026-08-08.
Verified against the tree before any work; the phase budget went to *attacking* the mitigation
instead, which is where both real defects came from. Applies [[discovery-over-checklist]] to the
commission itself.

## SYMLINK-DANGLE-1 (HIGH) — filesystem containment escaped by a dangling symlink

`canonical_existing` (`crates/delulu-runtime/src/prim.rs`) falls back to canonicalizing the nearest
existing ancestor and re-appending the rest, on the stated grounds that those components "are plain
names the OS has not yet been asked to interpret". **False for a dangling symlink**: `canonicalize`
fails with `NotFound` exactly as it does for an absent name, so the link's own name was re-appended,
containment passed, and the caller's `fs::write` then followed the link out of the grant.

Witness (Linux, end-to-end, identical program + grant): **dangling target → `write returned Ok`,
file created OUTSIDE the grant; existing target → `DL0904` refused.** One bit apart.

- **Workspace-deliverable**, unlike the hardlink boundary — git stores a symlink as a path string
  (mode `120000`), so a cloned repo/package/agent-target can carry the aimed link. That property is
  exactly what made C84 the urgent class.
- Create-not-clobber (existing files still protected) — still enough for `~/.ssh/authorized_keys`
  where absent, shell rc files, autostart entries.
- **One flaw, three surfaces**: runtime ops via `resolve_in_scope`, capability *minting*
  (`root.fs_read`/`fs_write` — C84's "second door"), and the **WASM host** (`delulu-wasm/src/host.rs`
  shares `contains_on_disk`; engine fault-parity is a tested law).
- Fix: a component that **exists as a symlink** but did not canonicalize can no longer be
  re-appended → fail closed. Deliberately narrow (non-symlinks still re-appended, so the hardlink
  disposition is unchanged; resolvable links still work). Dangling-links-pointing-inside are refused
  too, pinned as its own test.
- **Windows**: junctions dangle with NO privilege (`mklink /J`), and Rust's `is_symlink()` DOES
  report junctions — verified, so the fix covers them. Unprivileged *file* symlinks are not
  creatable on Windows, so that vector is POSIX-first.
- Tests include an **over-narrowing guard** (an ordinary write to a not-yet-existing file must still
  be contained) — without it a fix aimed at the escape could silently break every write.

## ROOTPOLICY-1 (MODERATE) — strict root-issuance mode failed OPEN

`root_policy.json` was the ONE persisted security file in `<state>/` that failed open:
`load_root_policy` swallowed every error with `.ok()?` → `None` → *legacy, unsigned roots allowed*.
Its two siblings loaded a few lines away both fail closed (`guard_policy.json` poisons the Guard,
`revoked_certs.json` poisons adoptions). **Witnessed:** a truncated policy let the daemon answer
`ReqBody::Issue` with a live root — the DISC-1 hole reopened by a bad write.

Fixed: present-but-unreadable **poisons** (both doors — `DL1421` + poisoned adoptions) while the
daemon keeps SERVING so e-stop/revoke still work (refusing to start would trade this for the
availability fail-open IPC-1/DEADMAN-1 closed); written **atomically**; banner now reports the mode
in all three states. **Not** a same-uid escalation (an adversary who can corrupt can also delete) —
a fail-closed/consistency fix; category-7 residual unchanged.

## CORE-SNAPSHOT-1 — the previous campaign closed claiming a green suite that was red

`cargo test --workspace` fails on `core_invariance :: the_core_still_answers_exactly_as_recorded`,
and **has since `d25ee5c`/`7dbd63b` (2026-08-09)** — those commits added two conformance witness
programs (DL0212, DL0213) without re-blessing `tests/core-invariance/SNAPSHOT.txt`. Proven not-mine:
the diff was **4 `+ NEW`, 0 `~ CHANGED`**, and re-blessing was 164 insertions / 0 deletions.

**The method defect matters more than the stale file:** piping `cargo test` through `tail` reports
the *pipe's* exit status, so a failing suite reads as exit 0 — and the harness's own completion
notice said "exit code 0" while cargo had returned **101**. Rule: *a suite is green only if the
runner's own exit code says so* — capture `$?` from cargo directly, never through a pipe.

## DEPPIN-LEX-1 (MODERATE) — a dependency's authority pin was escapable by spelling

`dep_scope_within_paths` (`crates/delulu-check/src/deps.rs`) is a prefix test whose normalizer only
swapped `\`→`/` and trimmed trailing slashes, leaving `..` in the string. Witnessed with
`delulu check` on a real two-package workspace: `fs.read=["../outside"]` → **DL1001 refused**;
`fs.read=["data/../../outside"]` → **checked clean**. Same destination, opposite verdicts.

**Honest scope:** the SUPPLY-CHAIN boundary, not the runtime one — runtime access is bounded by the
grant and `resolve_in_scope` (which normalizes), and `--grant-manifest` reads only the ROOT package's
manifest, never a dependency's (verified). Fixed by resolving `.`/`..` while KEEPING unresolvable
leading `..` (else `../outside` normalizes to `outside` and lands inside the pin it climbs out of).

## GUARD-SPELL-1 (HIGH) — a guard seal written the natural way gated nothing and said `ok`

Found by CONTINUING after the campaign's own final phase closed, using the shape the other findings
shared. `pattern_matches` in `crates/delulu-broker/src/guard.rs` was byte-exact equality, but for
`fs_read`/`fs_write` the token is the runtime's **resolved ABSOLUTE** path (`fs_scope_arg` →
`resolve_norm`). So `guard policy set "fs_write:./out/secret.txt" sealed` stored a rule that could
never fire and answered `ok: … → sealed`.

**Witnessed end-to-end (real broker + real lease):** with that seal in place the program **WROTE THE
FILE**; `fs_write:*` was correctly refused `DL1413`. The audit log showed the broker matching
`target=C:\Users\…\out\secret.txt` against a rule holding `./out/secret.txt`.

Grants no new authority — it defeats the control a principal reaches for AFTER granting ("write in
./out, but never this file"), while telling them it is in place. A seal that reports success and
gates nothing is worse than no seal.

**The known `effect:<typo>` footgun was deferred as "minor" and that assessment did not transfer:**
`effect:`'s token space is CLOSED (typo is the only way in); the path classes are OPEN and the
*natural* spelling is the broken one, because the grant beside it is relative (`--grant fs.write=./out`).

Fixed at the broker boundary: `dead_pattern_reason` refuses un-matchable rules (**after** the owner
gate, so an unauthorized caller still gets DL1414 and cannot probe) → **DL0904, deliberately NOT a new
DL code** (a new code needs a conformance witness — that is how CORE-SNAPSHOT-1 went red); plus one
lexical normalizer applied to BOTH pattern and arg for path classes. `overlaps` (pattern-to-pattern)
untouched, so the 30k-policy differential test still holds. Closes the deferred `effect:` case too.

## SERVERPATH-REL-1 (LOW) + CONTAIN-TOCTOU-1 (residual)

- `editors/vscode/server-resolve.js` promised an absolute path but did not require PATH *entries* to
  be absolute: with `PATH="."` and a planted `delulu.exe` in the cwd it returned bare `"delulu.exe"`
  — the P19 planted-binary hole via a PATH misconfiguration. Relative entries now skipped. 17/17.
- **CONTAIN-TOCTOU-1**: containment is a check-then-open, so a concurrent writer into the granted
  directory can swap a checked file for a symlink. DOCUMENTED, not fixed — the confined program
  cannot win it through this API (no symlink-creating primitive), the second writer is same-uid
  (category 7) or anyone who can write there, and a real fix needs `O_NOFOLLOW`/`openat2`, i.e. the
  platform-dependent containment this project refuses. **Deployment rule: grant scopes pointing at
  directories only the program's own user can write, never a shared one.**

## Final state (commits `56f53ec`, `a028913`, + Phase D close-out)

- **Windows**: cargo exit 0, 124 binaries, 1629 tests, 0 fail. **Linux**: cargo exit 0, 124 binaries,
  1638 tests, 0 fail (delta = platform-gated tests). **macOS: UNVERIFIED** (no Mac; blake3 C-toolchain
  blocker) — and it is POSIX, the platform where SYMLINK-DANGLE-1 was fully exploitable rather than
  privilege-blocked. Say so plainly.
- Miri: small targeted batch, 6 containment tests, **0 UB** (`-Zmiri-disable-isolation` so the symlink
  syscalls really ran). Tonight's changes add no `unsafe`.
- **Verdict: PRODUCTION CANDIDATE** — because the first serious probe of an already-closed area
  (C84) produced a HIGH escape, so the discovery curve has not flattened. Not "ready".

## INTERP-DROP-1 (MODERATE) — **FIXED** (`6aef21d`); the first cost estimate was of the WRONG DESIGN

The never-fuzzed surface (runtime on adversarial-but-VALID programs) yielded one finding that breaks
a rule the project states itself. `crates/delulu/src/main.rs` names
`ref.rule.runtime.faults-are-diagnostics`: *"a runtime fault must be a diagnostic (DL0905), never a
host crash"* — the whole reason the CLI runs on a 512 MiB `delulu-main` thread.

**`MAX_DEPTH` (10,000) bounds CALL depth; NOTHING bounds DATA depth.** `Value::Variant` holds
`Rc<Vec<Value>>` and its derived `Drop` recurses.

```delulu
type Chain = Nil | Link(Chain)          // build 5,000,000 deep in a while loop
```
- 1M-deep recursive CALL → `DL0905` on both platforms (the control — the guard works)
- 5M-deep recursive VALUE → Windows `0xC00000FD` STATUS_STACK_OVERFLOW; Linux exit 134 SIGABRT.
  **Prints its output FIRST**, then the host dies — the crash is in the runtime's own teardown.

**Not an authority escape** and must not be sold as one; a program may consume its own resources.
It is a defect because the project already decided depth exhaustion must be a diagnostic and built a
mechanism that covers only one of the two doors. Dead-man watchdog still fails a device safe.

**THE LESSON — I costed the wrong design and nearly deferred a cheap fix.** First estimate:
`impl Drop for Value` → **9 `E0509` errors** in `delulu-runtime` alone → "invasive, defer". But the
recursion is a CYCLE — `Value → Rc<Vec<Value>> → Vec<Value> → Value` — and it can be cut at EITHER
link. Cutting it at the **payload** costs almost nothing:

- `Variant`'s payload becomes `VariantFields(Rc<Vec<Value>>)`, which owns the destructor. `Value`
  gains no `Drop`, so **E0509 never arises**.
- `VariantFields: Deref<Target=Vec<Value>>` → every site that only READS (`.iter()`, `.len()`,
  indexing) compiles unchanged. **3 sites edited, not 9**: the constructor, the actor-message
  decoder, and `cycles.rs`'s pointer identity (`ptr_id()`).
- `Drop` moves children onto a worklist: **depth becomes breadth** — teardown costs heap (bounded by
  the structure that already fit in memory) instead of native stack (unbounded). A still-shared
  payload is left alone; the last owner does the work.

**Generalize:** when a recursive drop blows the stack, put the destructor on the PAYLOAD newtype, not
on the recursive enum. Verified: 1M/5M/20M all exit 0 on Win+Linux; `DL0905` control still fires;
3 regression tests; **falsified — the test BINARY dies with `0xc00000fd STATUS_STACK_OVERFLOW`**
(a stack overflow is not a catchable panic, so the signal is the process, not an assertion).
`core_invariance` passed, which is what proves a change to the runtime's most pervasive type moved
none of the compiler's answers. Win 1634 / Linux 1643 tests green.

## DEPLOYMENT PHASES 1–4 (same session) — the two named residuals, finished

**Verdict raised to `PRODUCTION READY WITH DOCUMENTED DEPLOYMENT REQUIREMENTS` — Windows + Linux,
Tier-2 deployment only. macOS explicitly NOT covered.**

**Strict anchored-root mode was opt-in because it was UNUSABLE.** Driving the documented flow end to
end for the first time found three defects:
- **CERT-SCOPE-REL-1** — `certify --fs-write ./out` signed + adopted cleanly, then made every
  delegation fail `DL0802` (cert stores the path AS TYPED; every other surface resolves to ABSOLUTE).
  Now refused at MINT time — a cert is the thing that travels out of contact.
- **DL1421-RENDER-1** — the DISC-1 refusal printed `error[DL1401] broker unreachable`.
  `broker_client::static_code` was a HAND-WRITTEN allowlist; diffing it against every `Denial::code()`
  gave exactly one gap: DL1421, that mitigation's own flagship. Deleted the list —
  `delulu_diag::static_code` derives it from the registry; test walks the WHOLE registry.
- `certify`'s `--fs-read/--fs-write/--net` flags existed but were **absent from `--help`**.

**Category 7 was CONVERTED, not closed** (it never can be — to the kernel that process is you):
- **Detection**: `Broker::record_root_policy_mode` writes the effective mode into the hash chain at
  every start. A downgrade must leave evidence or break `audit verify`, which is itself the alarm.
- **Checkable**: `delulu doctor` gained a `security posture` section (root-issuance mode / anchor-key
  custody / can the FS enforce owner-only at all). LEGACY = `note` (supported choice), UNREADABLE =
  `problem` (fails the run).
- **DOCTOR-STATEDIR-1** found while verifying it: doctor gave IDENTICAL answers for three different
  state dirs — it read `DELULU_HOME`/`$HOME/.delulu` and **never `DELULU_STATE_DIR`**. That is the
  "reads the wrong store" class F-CUSTODY-2 fixed for `delulu audit`; **nobody fixed doctor**, the one
  command whose job is "is this deployment sound?".
- **`docs/DEPLOYMENT.md`** — 3 tiers, exact commands, how to verify, what is NOT protected, and the
  recorded ruling that **strict stays opt-in for v1.x** (defaulting it would not close the hole it
  appears to close, breaks the primary workflow, and breaks a released 1.0; intended default at 2.0).

**macOS measured, not asserted:** `cargo check --target aarch64-apple-darwin` per crate → 4 members
type-check; every other member is blocked by a THIRD-PARTY C build script (`blake3`/`zstd-sys`/
`libffi-sys`), i.e. an absent Apple cross-toolchain, **not a delulu portability defect**. Exactly ONE
line branches on macOS (`SUN_PATH_MAX` 104 vs 108). Still **never executed**; `cargo test --workspace`
on any Mac is the one missing command.

**Phase-4 surfaces all verified:** Win 1641 / Linux 1650 tests, clippy 0 on Linux, all shipped
examples check clean, **LSP verified LIVE by speaking the protocol to the real binary** (P19's lesson
— the suite was green while the extension had no server), VS Code 17/17 + `.vsix` verifies.

## DOC + TOOL PASS (same session) — two more findings, both in the checkers

- **SURVEY-HEADING-1** — the Survey reported **C82–C92 as "not campaign findings"**. They are (C84 =
  filesystem escape, C88 = effect-row escape). `finding_definition` knew only the **table-row** shape
  of the original 2026-07-24 ledger; every later pass records findings as `### C<n> · …` sections
  under its own dated heading. **My first fix was wrong** — adding table rows would have made the note
  go away by *misfiling* a 2026-08-03 finding into a 2026-07-24 ledger. The defect was in the
  CHECKER. It had also silently falsified `PRODUCTION_READINESS_REVIEW.md`'s claim that the note's
  "only trigger is C99". Fixed in `mdown.rs` (both shapes), 2 tests incl. the negative.
- **BOOKFMT-1** — `delulu fmt --check docs/book/samples` is a **ci.yml gate recorded as green** and was
  reporting **2 would change**. The samples had not moved; the FORMATTER had (canonical effect-row
  order + 4-space indent), and the gate is **not part of `cargo test`** so nothing noticed. Reformatting
  then broke `book.rs` — correctly: Chapter 14's block is a **literal slice** of `08_foreign.delulu`
  (the C68 gate). Book block updated to match. **Lesson: a gate outside `cargo test` rots silently.**
- `survey/README.md` tabled 3 files saying "all three are generated"; the directory holds **5** —
  `AUDIT.md` and `REMOVALS.md` are hand-written and were listed nowhere.

## FINAL VERIFIED STATE (2026-08-10, commit `35a1f02`)

| | Windows | Linux |
|---|---|---|
| suite | cargo exit 0, 124 binaries, **1,645** | cargo exit 0, 124 binaries, **1,654** |
| clippy | clean | **0 findings** |
| fmt gates | examples 0/13, book samples 0/10 | same |
| doctor | all checks pass | all checks pass |
| LSP | **LIVE** (real protocol) | **LIVE** |
| VS Code | 17/17 + `.vsix` verifies | (no node in WSL image) |

All security witnesses re-run green on Linux: SYMLINK-DANGLE-1 contained (+ no over-narrowing),
INTERP-DROP-1 at 20M deep exits 0, DEPPIN-LEX-1 refused DL1001, GUARD-SPELL-1 relative seal refused /
absolute seal seals, doctor mode-agreement problem→exit 1 then ok→exit 0, audit chain verifies.

**Do NOT quote a doctor check-count** — it varies by environment (17 on Windows here, 14 on Linux)
because a state dir/audit log that exists adds checks. Say "all checks pass".

## THE SEARCH KEY that found four of six defects

**A security decision made on an unnormalized / unresolved representation, walked past by a different
spelling of the same thing.** A dangling link `canonicalize` could not resolve; a `..` left in a pin
comparison; a relative `PATH` entry; a relative guard pattern. Once the first three were in hand this
became the query, and it is what found GUARD-SPELL-1 **after the campaign's own final phase had
closed**. Carry it forward as a standing review question: *for every string compared to decide a
security outcome, what else spells the same thing?*

## Operational lessons (this session)

- **Attack your own fix.** Testing the new code against a Windows junction whose target *existed*
  showed my new `DL0904` message asserting "its target does not exist" — a diagnostic stating a
  false cause. Fixed to ask whether the deepest link actually resolves.
- **A falsification that does not change the binary proves nothing.** One guard-removal patch
  SILENTLY MISSED (CRLF vs LF in the match text) and the test "passed" against supposedly-broken
  code. Caught only because its sibling test failed and the pair disagreed. Always confirm the edit
  landed (`grep -c FALSIFICATION`), never just that the runner ran.
- **A witness that never ran is not a negative result.** A guard test reported "sealed: no file"
  when in fact `run` had rejected an unknown `--state-dir` and the program never executed. Check the
  witness produced the *baseline* effect before believing the refusal.
- **A gate outside `cargo test` rots silently** (BOOKFMT-1). `fmt --check docs/book/samples` is a
  `ci.yml` gate recorded as green and had been red since the formatter changed. Run the *declared*
  gates, not just the suite.
- **Read the document's STRUCTURE before "fixing" what a checker reports.** The obvious fix to
  SURVEY-HEADING-1 (add ledger rows) would have corrupted the ledger's meaning; the defect was in the
  checker. A note that says "for a human to judge" means judge it.
- **A Python heredoc that prints an emoji dies on Windows cp1252 BEFORE it writes** — one doc patch
  reported two successful replacements and saved nothing. Verify the file changed, not that the
  script printed "ok".
- **`git add -A` after a doc pass emits a wall of CRLF warnings** — harmless (`.gitattributes`
  normalisation), but check `git show --stat` to confirm the commit is only the files you meant.
- **Stale binary trap**: a background `cargo test` holds `delulu.exe`, so a concurrent `cargo build`
  fails with "failed to remove file" and the subsequent run silently uses the OLD binary. Never
  evidence — rebuild after the suite releases the lock.
- WSL from **PowerShell**, never Git-Bash (Git-Bash rewrites `/mnt/...` into `C:/Program Files/Git/mnt/...`).
- `delulu doctor` auto-regenerates a stale Survey and says so; the `doctor_cli` tests deliberately
  refuse to self-repair it ("the freshness gate is what should be failing here").

Related: [[delulu-production-readiness-2026-08-09]], [[delulu-root-issuance-bypass]],
[[delulu-core-regression-rule]], [[delulu-survey-map]], [[discovery-over-checklist]].
Full log: `docs/design/PRODUCTION_READINESS_2026-08-10.md`.
