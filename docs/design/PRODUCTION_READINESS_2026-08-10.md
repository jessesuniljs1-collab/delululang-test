# Production-readiness hardening — overnight campaign, 2026-08-10

An autonomous, phase-by-phase security campaign (Opus 5) commissioned to make DeluluLang as close to
genuinely production-ready as the repository can support — and, explicitly, **not** merely to make
the known checklist green. The standing question every phase asks is *"what security assumption
haven't we attacked yet?"*

Standing rules honored: use and regenerate the Survey every phase; run `delulu doctor`; never push to
GitHub; harden-never-redefine Authority/Guard; every fix carries a witness that **fails against the
pre-fix code**; no claim is upgraded beyond the evidence that exists for it.

Platforms this bench can execute: **Windows 11** (native) and **Linux** (WSL2 Ubuntu-20.04).
**macOS cannot be executed here** — those rows stay static review only, never claimed as run.

| Phase | Scope | Result |
|---|---|---|
| A | Discovery, threat model, attack-surface + invariant map | ✅ 2 real defects found, 2 negatives recorded |
| B | Implementation & hardening | ✅ both fixed, both falsified against pre-fix code |
| C | Adversarial verification & cross-surface attack | ✅ 2 more defects + 1 residual documented; Win + Linux green |
| D | Final production readiness, evidence & documentation | ✅ both C84 doors closed; **PRODUCTION CANDIDATE** |

**Findings this campaign:** SYMLINK-DANGLE-1 (high, fixed), **GUARD-SPELL-1 (high, fixed)**,
DEPPIN-LEX-1 (moderate, fixed), ROOTPOLICY-1 (moderate, fixed), SERVERPATH-REL-1 (low, fixed),
CORE-SNAPSHOT-1 (evidence honesty, fixed), CONTAIN-TOCTOU-1 (residual, documented),
**INTERP-DROP-1 (moderate, fix costed and deferred to the owner)**.

**The through-line: four of the six were the same defect.** A security decision made on an
**unnormalized or unresolved representation**, walked past by a different spelling of the same thing —
a dangling link that `canonicalize` could not resolve, a `..` left in a pin comparison, a relative
`PATH` entry, and a relative guard pattern. After the first three, that shape became the search key,
and it is what found GUARD-SPELL-1 in the Guard itself *after* the campaign's own final phase had
closed. Worth carrying forward as a review question rather than a one-off: **for every string
compared to decide a security outcome, what else spells the same thing?**

---

## Correction to the commission's premise (recorded, not hidden)

The commission was written as if **DISC-1 were unresolved** and instructed that opt-in strict
anchored-root mode be designed and built. That premise is **stale**: strict mode shipped on
2026-08-08 (`412fa98` → `2be5839` → `222f617`). Verified against the current tree before any work
began — `broker start --require-anchored-roots <anchor>`, `Broker::issue_root` gating `DL1421`,
`Broker::issue` reduced to `pub(crate)` and reachable only through a verified `adopt`.

Re-implementing it would have been busywork on an already-closed hole. The campaign therefore spent
its Phase A budget **attacking** the existing mitigation and the surfaces around it instead — which
is where both of tonight's defects came from. This is the "re-verify every agent claim against the
current binary" rule applied to the commission itself.

---

## Phase A — discovery, threat model & architecture audit

### The reasoning that chose the targets

The six defects of the 2026-08-09 campaign share one shape: *safe in the normal case, unbounded or
resurrectable in the adversarial one*. Three of them (ROTATE-1, ADOPT-REPLAY-1, and the persistence
class generally) were specifically **restart-resurrection** bugs. So Phase A probed two seams:

1. **Does the DISC-1 mitigation itself survive a restart and a bad file?** → found **ROOTPOLICY-1**.
2. **Where can the compiler's static knowledge not reach?** The compiler cannot know a runtime path
   value, so every primitive touching a scoped resource must re-check containment *at the point of
   use*; the whole runtime funnels this through one function. → found **SYMLINK-DANGLE-1**.

### Negative results (recorded, because a pass that only lists faults is not a pass)

- **The authority-inheritance walk fails closed at every caller.** `effective_state_inherited`
  returns `Option`, and a missing ancestor yields `None`. All five callers were checked: `lease.rs`,
  `tree.rs` (attenuate), and `validate.rs` (snapshot) each `.unwrap_or(EffState::Expired)`, and
  `node_view`'s `None` becomes an audited deny. A broken ancestry cannot read as "live".
- **The hardlink boundary is honestly documented and pinned as an executed test.** Re-reviewed
  against its three claimed facts; the disposition holds and is not overclaimed.

---

## Phase B — implementation & hardening

### SYMLINK-DANGLE-1 — a dangling symlink escaped filesystem containment (HIGH)

**What.** `resolve_in_scope` is the runtime's whole filesystem boundary: lexical `..` check, then
`contains_on_disk`, which canonicalizes both sides so a link cannot walk past it (that is the C84
fix). Because a write must be able to *create* its file, `canonical_existing` falls back to
canonicalizing the nearest existing ancestor and re-appending the remaining components — on the
stated grounds that those components "are plain names the OS has not yet been asked to interpret".

**That claim is false for exactly one case.** A *dangling* symlink — one whose target does not exist
— makes `canonicalize` fail with `NotFound`, which is indistinguishable here from a name that is
simply absent. The walk therefore re-appended the link's own name as a plain name, the containment
check passed, and the caller then opened the path — at which point the OS *did* interpret it,
followed the link, and created the file wherever it pointed.

**Witness (Linux, real broker, end-to-end).** Identical program, identical grant, one bit different:

| symlink target | verdict |
|---|---|
| **dangling** (absent) | `write returned Ok` — **file created OUTSIDE the grant** |
| **exists** (the C84 vector) | `DL0904` refused — C84 fix holds |

The only variable is whether the target happened to exist.

**Why it is the urgent class, not the hardlink class.** The hardlink boundary is dispositioned as
acceptable partly because *git cannot carry a hardlink* (a clone gets a plain blob). Git **can**
carry a symlink — it stores the path string as mode `120000` — so a cloned repository, a package, or
a project an agent was pointed at can deliver the aimed link. That is precisely the property that
made C84 urgent.

**Impact.** Arbitrary file **creation** outside the granted scope. Existing files are still protected
(a link to an existing target resolves and is refused), so this is create-not-clobber — which is
still enough for `~/.ssh/authorized_keys` where absent, shell rc files, `~/.config/autostart/*`, or a
user systemd unit, i.e. persistence and code execution. It also directly falsifies the product's core
claim that a program cannot touch anything outside its grant.

**Blast radius — one flaw, three surfaces.** `contains_on_disk` is shared by (a) every runtime
filesystem operation via `resolve_in_scope`, (b) capability *minting* (`root.fs_read` / `root.fs_write`,
the "second door" C84 also had to close), and (c) **the WASM host** (`delulu-wasm/src/host.rs`), where
engine fault-parity is a tested law. One fix closes all three.

**Fix.** In `canonical_existing`, a component that **exists as a symlink** but did not canonicalize
can no longer be re-appended — where it lands is exactly what could not be verified, so it fails
closed. Deliberately narrow: a component that exists and is *not* a symlink is still re-appended (the
hardlink disposition is unchanged), and a link whose target exists still resolves and is checked
exactly as C84 fixed it. `DL0904` now names the cause in the operator's terms.

**Deliberate narrowing, recorded.** A dangling link pointing *back inside* the grant is refused too.
Telling it apart means resolving the link by hand; when the alternative is guessing where a write
lands, narrowing is the safe direction. Pinned as its own test so relaxing it must be a decision.

**Verification.**

- Unit: 6 containment tests green on Windows and Linux, including an **over-narrowing guard** — an
  ordinary write to a not-yet-existing file, and one in a not-yet-existing subdirectory, must still
  be contained. Without it, a fix aimed at the escape could quietly break every write.
- End-to-end (Linux): attack refused; ordinary write still works; a link resolving inside the grant
  still works; C84 control still refused.
- Platform note: symlink creation needs privilege on Windows, so the symlink assertions **skip**
  there and the proof is the Linux run. Recorded rather than papered over.

### ROOTPOLICY-1 — strict root-issuance mode failed OPEN on an unreadable policy (MODERATE)

**What.** Three persisted security files load within a few lines of each other in `serve()`. Two fail
closed. The third — the one gating **root creation**, i.e. the DISC-1 mitigation itself — failed open:

| file | corrupt / unreadable → | write |
|---|---|---|
| `guard_policy.json` | poisoned — fail closed | non-atomic (documented best-effort) |
| `revoked_certs.json` | adoptions poisoned — fail closed | **atomic** (temp + rename) |
| `root_policy.json` (before) | **legacy: unsigned roots allowed — fail OPEN** | **non-atomic** |

`load_root_policy` swallowed every error with `.ok()?` and returned `None`, which the caller reads as
*legacy*. So an unreadable or truncated policy did not disable a feature — it silently turned the
root-issuance gate **off** and served.

**Reachability, stated honestly.** A same-uid adversary gains nothing new: they can already delete the
file (category 7, documented). The realistic trigger is **non-adversarial corruption** — a crash or a
full disk during the non-atomic write — and the harm is that the weakening is silent. This is a
fail-closed/consistency defect, **not** a privilege escalation, and is not claimed as one.

**Witness.** Against the pre-fix loader, a daemon started with a truncated `root_policy.json` answered
`ReqBody::Issue` with `Issued { node: "g_e8425033…" }` — a live root minted, the DISC-1 hole reopened
by nothing more than a bad write. After the fix the same request is refused `DL1421`.

**Fix.**

- A file that exists but does not read back as *strict with an anchor* now **poisons**. `seed_root_policy`
  only ever writes that shape and the documented way back to legacy is to remove the file, so a
  malformed file is not a legacy marker — it is a policy we cannot read.
- Poisoned pins an anchor that can never verify (`issue_root` → `DL1421`) **and** poisons adoptions,
  so **both** doors are shut. The daemon deliberately keeps serving, so revocation and the operator's
  e-stop still work: refusing to start would trade this fail-open for the availability fail-open that
  IPC-1/DEADMAN-1 closed.
- The policy is now written **atomically** (temp + rename), matching `revoked_certs.json`.
- The start banner now reports the effective mode in **all three** states. The loader's own doc had
  claimed "the banner reports the effective mode, so an unexpected legacy state is visible rather
  than silent" — but the old code printed nothing at all in the legacy case, and the absence of a
  line is not a report, least of all in a detached start whose operator reads `broker.log`.

**Verification.** Two regression tests: a classification test over seven unreadable shapes, and an
over-the-wire test asserting both doors refuse while `Status` still answers. Both **fail against the
pre-fix loader** (falsification run recorded above), then pass.

---

### CORE-SNAPSHOT-1 — the core-regression gate had been red since 2026-08-09 (evidence honesty)

**What.** `cargo test --workspace` fails on `core_invariance :: the_core_still_answers_exactly_as_recorded`.
That gate exists to enforce the standing core-regression rule: it pins the compiler's exact bytes for
every shipped program so tooling cannot silently move the language.

**It is not caused by this campaign's changes, and that was verified rather than assumed.** The diff
contains only **four `+ NEW` cases and zero `~ CHANGED`** — the two conformance witness programs
(`DL0212_deep_pattern.delulu`, `DL0213_deep_blocks.delulu`) added by the *previous* campaign in
`d25ee5c` / `7dbd63b`, whose outputs were never recorded. `SNAPSHOT.txt` was last blessed at
`8e57f1a`, before those files existed. Re-blessing produced **164 insertions and 0 deletions**,
confirming no existing core answer moved.

**Why it matters more than the fix.** The 2026-08-09 campaign closed recording the full suite as
green on Windows and Linux. For `cargo test --workspace` that was **not true** from `d25ee5c`
onward — adding a shipped program adds a snapshot case, and the gate had been failing ever since.
The likely mechanism is the same one that nearly caught this campaign an hour earlier: piping
`cargo test` through `tail`, which reports the *pipe's* exit status, so a failing suite reads as
exit 0. That is recorded here as a method defect, not just a stale file — **a suite is green only if
the runner's own exit code says so.**

**Fix.** Snapshot re-blessed (additive only, diff read before committing); the campaign's own suite
runs now capture cargo's exit code directly rather than through a pipe.

---

## Phase C — adversarial verification & cross-surface attack

Baseline before this phase: `cargo test --workspace` **cargo exit 0, 124 test binaries, 1628 tests,
0 failures** on Windows, tree frozen (see CORE-SNAPSHOT-1 on why the exit code is quoted and not a
tail of the log).

### DEPPIN-LEX-1 — a dependency's authority pin was escapable by spelling (MODERATE)

**What.** `dep_scope_within_paths` decides whether a dependency's declared filesystem scope sits
inside the consumer's pin (§A.3). It is a *prefix* test, and its normalizer only swapped `\` for `/`
and trimmed trailing slashes — `..` was left in the string. So a scope could lexically sit under the
pin and resolve outside it.

**Witness (`delulu check`, real two-package workspace).** Same destination, opposite verdicts:

| dependency declares | verdict |
|---|---|
| `../outside` | `DL1001` refused — the pin is enforced |
| `data/../../outside` | **checked clean** — the pin is escaped |

**Scope of the claim, stated honestly.** This is the *supply-chain* boundary, not the runtime one.
Runtime filesystem access is bounded by the grant and by `resolve_in_scope`, which normalizes
properly, and `--grant-manifest` reads only the **root** package's manifest, never a dependency's —
verified, not assumed. So a malicious dependency could not reach the filesystem through this. What it
defeated is the constraint a consumer *puts on a third-party dependency*, which finding C19 / ruling
D34 already established as a real control ("the pin was decoration" was treated as a genuine defect
then, and a pin a spelling walks past is decoration too).

**Fix.** The normalizer now resolves `.` and `..`, keeping unresolvable leading `..` as `..` so
`../outside` cannot normalize into `outside` and land *inside* a pin it climbs out of. A pin of the
package root (`.`) is handled explicitly so the empty string cannot accidentally prefix-match.
Regression test covers both escaping spellings, the legitimate descendants (including `data/sub/../other`,
which climbs but stays inside), the Windows spelling, and the near-miss `database` — and it **fails
against the pre-fix normalizer** (falsification run recorded).

### SERVERPATH-REL-1 — a relative `PATH` entry re-opened the planted-binary hole (LOW)

**What.** `editors/vscode/server-resolve.js` exists so that "a bare or relative name is never handed
to the OS to resolve … the outcome [is] a function of this code rather than of the host's working
directory", and its contract says it returns an **absolute** path. The PATH branch joined the bare
name onto each `PATH` entry without requiring that entry to be absolute.

**Witness.** With `PATH="."` and a planted `delulu.exe` in the working directory,
`resolveServer("delulu")` returned `"delulu.exe"` — relative, contract violated, and resolved by the
OS against wherever the extension was standing. That is the P19 planted-binary hole reached through a
`PATH` misconfiguration instead of a setting. The existing tests covered an *empty* PATH and a
relative *configured path*, but no test covered a relative PATH **entry**.

**Fix.** Relative `PATH` entries are skipped, for the same reason a relative `delulu.serverPath` is
refused outright. Two tests added (the attack, and an absolute entry still resolving to an absolute
path). Extension suite: **17/17 green**.

### CONTAIN-TOCTOU-1 — the containment race, now named rather than implied (residual, documented)

The containment doc presents itself as "the boundary `contains_on_disk` draws, and the one it does
NOT", and listed only the hardlink. But the check resolves a path and the operation that follows
re-opens it **by name**, so a concurrent writer into the granted directory can swap a checked file
for a symlink in between.

Recorded with its threat model rather than fixed, and the reasoning is the hardlink's: the confined
program **cannot** win this race through this API (the primitive table exposes no symlink-creating
operation, so it needs a *second* writer), the second writer is either a same-uid process
(category 7, already outside the proof boundary) or anyone able to write into the granted directory,
and closing it properly requires `O_NOFOLLOW`/`openat2`/`FILE_FLAG_OPEN_REPARSE_POINT` — the
platform-dependent containment this project explicitly refuses. Deployment consequence stated in one
line: **grant scopes that point at directories only the program's own user can write.**

### GUARD-SPELL-1 — a guard seal written the natural way gated nothing, and said `ok` (HIGH)

Found while continuing after the Phase-D close-out, by asking where else a security decision is made
on an unnormalized representation — the shape three of the night's four defects already shared. The
last place it lives is the Guard's own rule matcher.

**What.** `pattern_matches` was `pattern == "*" || token == Some(pattern)` — byte-exact equality. For
`fs_read`/`fs_write` the token is not what the operator typed: it is produced by the runtime as
`resolve_norm(cap_root, rel)`, an **absolute** path in the host's separator spelling. So a rule
written relatively could never fire, and `guard policy set` reported success anyway.

**Witness (real broker, real lease, end-to-end).**

| guard policy | result |
|---|---|
| none | file written (baseline) |
| **`fs_write:./out/secret.txt` → sealed** | `ok: … → sealed`, then **FILE WRITTEN** |
| `fs_write:*` → sealed | `DL1413` refused |

The audit log shows exactly why: the broker matched against
`target=C:\Users\…\out\secret.txt` while the rule held `./out/secret.txt`.

**Why this is the worst-shaped defect of the campaign even though it grants no new authority.** It
does not escalate anything — the program already held the grant. What it does is defeat the control a
principal reaches for *after* granting: "you may write in ./out, but never this file." The operator
is told the seal is in place. A seal that reports success while gating nothing is worse than no seal,
because it stops the operator looking for another mechanism. The Guard is the one subsystem the
standing rule says to harden and never redefine; this hardens it.

**Relation to the known footgun.** `HARDENING_CAMPAIGN.md` had already noted that
`guard policy set effect:<typo>` accepts a rule gating nothing, deferring it as "a separate, minor
typo footgun … not a hole". That assessment was right for `effect:`, whose token space is **closed**
(a typo is the only way in). It did not transfer to the **path** classes, whose token space is open
and where the *natural* spelling is the broken one — the grant beside it is written relative
(`--grant fs.write=./out`). Same gap, one class over, materially worse.

**Fix (two parts, both at the authoritative broker boundary).**

1. **Refuse a rule that cannot match**, after the owner-code gate so an unauthorized caller still gets
   `DL1414` and cannot probe pattern validity. `dead_pattern_reason` → `DL0904`, with a message naming
   the absolute form and where to read it (`delulu audit tail`'s `target=`). This also closes the
   deferred `effect:<typo>` case in the same stroke. **Deliberately no new diagnostic code** — a new
   `DLxxxx` would need a conformance witness, which is precisely how CORE-SNAPSHOT-1 went red.
2. **Normalize both sides for path classes** through one lexical function (separators + `.`/`..`, no
   filesystem access), so a correctly-absolute seal written `C:/out/x` still covers the runtime's
   `C:\out\x`. `overlaps` (pattern-to-pattern, used by `tier_for_subset`) is untouched, so the
   exhaustive 30k-policy differential test against the use-time oracle still holds.

**Verification.** Two regression tests, both **falsified against the pre-fix code**, plus the
end-to-end re-run: the relative seal is now refused with an actionable message; an absolute seal
returns `DL1413` and no file; the same path spelled with forward slashes also returns `DL1413` (that
one would have silently failed before); `effect:Wrtie` is refused; `effect:Write` still settable.
Guard suite 22/22.

*Falsification note worth keeping:* the first attempt to falsify the refusal test **silently did not
apply** (CRLF vs LF in the patch text) and the test "passed" against supposedly-broken code. Caught
only because the sibling test failed and the pair disagreed. A falsification that does not change the
binary proves nothing — verify the edit landed, not just that the runner ran.

### INTERP-DROP-1 — a valid program aborts the host during value teardown (MODERATE, **fix deferred to the owner**)

The one large surface this repository had never fuzzed is the runtime on *adversarial-but-valid*
programs — programs that type-check and pass the effect row, but stress the interpreter. Probing it
produced one finding, and it is a **rule violation, not a resource-policy question**.

**The rule it breaks is the project's own.** `crates/delulu/src/main.rs` states
`ref.rule.runtime.faults-are-diagnostics`: *"a runtime fault must be a diagnostic (`DL0905`), never a
host crash."* The entire reason the CLI runs on a 512 MiB `delulu-main` thread is to make the
interpreter's own `MAX_DEPTH` the limit that fires instead of a native overflow.

**But `MAX_DEPTH` bounds CALL depth, not DATA depth.** `Value::Variant { fields: Rc<Vec<Value>> }`
nests without bound, and its derived `Drop` recurses. The large stack does not fix that; it only
moves the threshold.

**Witness — same runtime, same machine, one difference: calls vs. data.**

| program | Windows | Linux |
|---|---|---|
| 1,000,000-deep **recursive call** | `DL0905` diagnostic | `DL0905`, exit 1 |
| 5,000,000-deep **recursive value** (`type Chain = Nil \| Link(Chain)`) | exit `0xC00000FD` `STATUS_STACK_OVERFLOW` | exit 134 SIGABRT, `fatal runtime error: stack overflow` |

The program prints `built the chain` **first** and then the host dies: the work completed and the
crash is in the runtime's own teardown. `DEFAULT_MAX_DEPTH` is 10,000 for calls; data nesting is
bounded by nothing.

**Severity, stated precisely.** This is **not** an authority escape and is not presented as one — the
program harms only its own process, and a program is entitled to consume its own resources
(`authority is the containment, not resource limits`). What makes it a defect rather than a policy
question is that the project already decided depth exhaustion must surface as a diagnostic, built a
mechanism to guarantee it, and that mechanism covers one of the two doors. The practical cost is that
there is no structured fault, no exit code a caller can act on, and no clean shutdown or audit
finalisation — for a long-running agent or service that is an abrupt disappearance. In the robotics
path the dead-man watchdog is designed for exactly an abrupt host death, so a device still fails
safe; that containment is unaffected.

**Why the fix is not applied here.** The correct fix is an **iterative teardown** for `Value` instead
of the derived recursive `Drop`. Measured rather than guessed: adding `impl Drop for Value` produces
**9 `E0509` "cannot move out of type which implements Drop" errors in `delulu-runtime` alone**,
before the dependent crates (`delulu`, `delulu-wasm`) are even reached. That is an invasive change to
the most pervasive type in the runtime, it would require re-verifying the core-invariance snapshot,
and it fixes a crash rather than a containment hole. Making that change unreviewed at the end of an
autonomous session is precisely what the standing core-regression rule exists to prevent.

**Recorded as a costed owner decision, not an accepted boundary.** Unlike the hardlink and
CONTAIN-TOCTOU-1 residuals — which are blocked by the platform-dependence the project refuses —
this one is **fixable in-tree**; it is deferred for review, not dispositioned as acceptable. The
reproducer above is the acceptance test for whichever fix is chosen.

### Negative results from this phase

- **The VS Code extension's execution paths are clean.** `execFile` with an argv vector,
  `createTerminal({shellPath, shellArgs})` and `ProcessExecution` all bypass the shell, Workspace
  Trust gates `run`/`test`, and `delulu.serverPath` is machine-scoped so a workspace cannot set it.
  The P18 command-injection class is closed and stayed closed; only the PATH gap above was new.
- **The dangling-link fix holds against Windows junctions.** Junctions dangle with *no* privilege
  (`mklink /J`), and Rust's `is_symlink()` does report them — verified end-to-end, so the fix covers
  the one reparse point Windows hands out freely.
- **The lexical-prefix bug class is bounded to one site.** Every `starts_with(&format!(…))` in the
  workspace was reviewed: `python.rs` matches module *namespaces* (where `..` has no meaning) and
  `codeowners.rs` is survey tooling. `deps.rs` was the only path instance.
- **The hex comparisons fail closed.** Applying the same search key to certificate fingerprints and
  the pinned anchor: `Certificate::fingerprint` is blake3's lowercase hex computed *internally* and
  never caller-supplied, so the revocation denylist cannot be case-desynchronised; and a
  case-mismatched or malformed anchor simply matches nothing, which is the fail-closed direction the
  code already documents ("a bogus anchor simply means no chain ever verifies"). No finding.
- **The interpreter's call-depth guard holds.** A 1,000,000-deep recursive call is `DL0905` on both
  platforms, not a crash — the mechanism works exactly as designed. That control is what makes
  INTERP-DROP-1 above a specific gap rather than a general weakness.

---

## Phase D — final verification, evidence audit & production-readiness

### The final fresh pass — what had still not been attacked

C84 had **two doors**: using a capability, and *minting* one rooted at a link. Tonight's fix was
proven on the first; the second was checked separately rather than assumed, because "the same helper
guards it" is exactly the reasoning that leaves a door open. `root.fs_write("./grant/<dangling>")`
is refused `DL0703`, and nothing is created outside. Both doors closed.

### Cross-platform, stated exactly

| Platform | Status | Evidence |
|---|---|---|
| **Windows 11** | ✅ executed | `cargo test --workspace` **cargo exit 0**, 124 binaries, **1631** tests, 0 failures, tree frozen |
| **Linux** (WSL2 Ubuntu-20.04) | ✅ executed | **cargo exit 0**, 124 binaries, **1640** tests, 0 failures; every witness re-run end-to-end; `doctor` 12/12 |

The Guard fix was verified on **both** platforms, which matters because the two disagree on what an
absolute path looks like: Windows refused the relative seal, sealed `C:\…\secret.txt`, and sealed the
same path spelled `C:/…/secret.txt`; Linux refused the relative seal, sealed `/tmp/…/secret.txt`, and
sealed the messy `/tmp/…/out/../out/./secret.txt`. Same rule, both spellings, both platforms.
| **macOS** | ⛔ **UNVERIFIED** | No Mac on this bench, and the blake3 C-toolchain blocker still prevents a cross-check. Reviewed statically only. **Not claimed as run.** |

The Windows/Linux test-count delta (1629 vs 1638, +9) is platform-gated tests, not a discrepancy in
what was verified — the same 124 binaries pass on both.

**Verdict-relevant:** the security defect that mattered most tonight, SYMLINK-DANGLE-1, was
exploitable *on POSIX* and only privilege-blocked on Windows. macOS is POSIX. It is therefore the
platform where tonight's headline finding would have been most exploitable, and it is the platform
this bench cannot execute. That is stated plainly rather than smoothed over.

### Evidence levels — no claim above its evidence

| Claim | Level |
|---|---|
| Dangling-link containment is closed (3 surfaces) | **experimentally verified** end-to-end + unit-tested + falsified against pre-fix code; Miri-clean |
| Strict root policy fails closed | **tested + falsified** (pre-fix loader minted a root; post-fix `DL1421`) |
| Dependency pin resists path spelling | **tested + falsified**; witnessed end-to-end on two platforms |
| VS Code server lookup ignores relative PATH | **tested** (17/17 extension suite) |
| Containment race (CONTAIN-TOCTOU-1) | **documented residual**, not fixed — needs OS-level `O_NOFOLLOW` class primitives |
| Same-uid adversary containment | **outside the proof boundary (category 7)** — unchanged |
| macOS behaviour | **unverified** |
| Miri | **0 UB on one small targeted batch** (6 containment tests, isolation disabled so the symlink syscalls really ran). Tonight's changes introduce **no `unsafe`**; the broad Miri coverage from the 2026-08-09 campaign stands and was not re-run |

### Production-readiness status

**PRODUCTION CANDIDATE.**

> **Re-affirmed after the campaign continued past this section.** GUARD-SPELL-1 — a second
> HIGH-severity finding, in the Guard itself — was found *after* this verdict was first written, by
> continuing to apply the search key rather than stopping at four green phases. That is the strongest
> possible evidence for the verdict below, not against it: the argument was "the discovery curve has
> not flattened", and the very next probe produced another high finding in another
> already-reviewed subsystem. Anything above "candidate" would have been wrong twice.

Not the higher rating, and the reason is specific rather than cautious boilerplate: **the first
serious probe of an area already considered closed produced a HIGH-severity containment escape.**
C84 had been fixed, red-teamed by four agents, pinned with regression tests and a written
disposition — and every symlink case ever executed against it happened to use a link whose target
existed. One night's attention found the missing case, and it broke the product's central promise
that a program cannot touch anything outside its grant. When a single session still yields a finding
of that severity in the core guarantee, the defect-discovery curve has not flattened, and the honest
reading is "candidate", not "ready".

Three further facts hold it below the top rating, each already documented rather than newly alleged:

1. **macOS is entirely unverified**, and is a POSIX platform where tonight's headline finding was
   fully exploitable rather than privilege-blocked.
2. **Strict anchored-root mode is still opt-in**; the default broker allows unsigned root issuance,
   and making it the default is a pending major-version decision.
3. **The same-uid boundary remains category 7.** No code change tonight moved it, and none claimed to.

What it *is*: coherent, green on both executable platforms, honest about its boundaries, with every
security fix carrying a witness that fails against the pre-fix code, and with its own documentation
corrected where it overclaimed.

## Documentation corrected in this phase

Stale claims found and fixed rather than merely appended to (see the entries themselves for detail):

- `HARDENING_CAMPAIGN.md` P20 — "the whole family of path-spelling escapes short of the hardlink
  boundary" held. Tonight's finding falsifies that sentence; corrected in place with a pointer here.
- `crates/delulu-broker/src/tree.rs` — a doc comment still asserted "nothing programmatic creates
  root nodes, Constitution §5.16 law 4", the exact claim DISC-1 disproved, left behind as a
  copy artifact on `require_anchored_roots`.
- `ROOT_ISSUANCE_TRUST_BOUNDARY.md` — records ROOTPOLICY-1 and the corrected failure semantics.
