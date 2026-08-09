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
| B | Implementation & hardening | ✅ both defects fixed, both falsified against pre-fix code |
| C | Adversarial verification & cross-surface attack | in progress |
| D | Final production readiness, evidence & documentation | pending |

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

## Documentation corrected in this phase

Stale claims found and fixed rather than merely appended to (see the entries themselves for detail):

- `HARDENING_CAMPAIGN.md` P20 — "the whole family of path-spelling escapes short of the hardlink
  boundary" held. Tonight's finding falsifies that sentence; corrected in place with a pointer here.
- `crates/delulu-broker/src/tree.rs` — a doc comment still asserted "nothing programmatic creates
  root nodes, Constitution §5.16 law 4", the exact claim DISC-1 disproved, left behind as a
  copy artifact on `require_anchored_roots`.
- `ROOT_ISSUANCE_TRUST_BOUNDARY.md` — records ROOTPOLICY-1 and the corrected failure semantics.
