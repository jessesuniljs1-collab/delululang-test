# Line-by-line honesty review — the autonomy addendum

**Stage 10 phase 10g. Acceptance criterion 10, first half:** *"the autonomy addendum's per-domain
honest boundaries pass line-by-line honesty review."*

Reviewed: `STAGE10_AUTONOMY_ADDENDUM.md` (Rev 2), every boundary claim, against what phases
10a–10g actually built. Reviewer: the 10g kitchen. Date: 2026-07-20.

## The test applied to each line

A boundary claim passes if a reader who believes it would **not be surprised by the code**. Three
ways to fail:

1. **Overclaim** — describes as built something that is designed.
2. **Tense drift** — a design document read as a status report, with no marker to tell them apart.
3. **Missing mechanism** — a pattern that *requires* a mechanism the system does not have, stated
   without naming the gap.

An addendum is allowed to describe intended design; what it is not allowed to do is let a reader
mistake intent for inventory. Every finding below was **fixed in the addendum in this phase**, not
merely logged.

## Findings

### F1 — §0, mechanism 1: "enforced host-side and adapter-side on every command" — OVERCLAIM

Both checks live in one process as built (build-order D11e). The sentence reads as though the
host/adapter split exists.

**Fixed:** the line now carries D11e's caveat inline — structural rehearsal, not independent
defense-in-depth — plus what it *does* buy, which is unit-witnessed: if the capability value and
the grant disagree, the grant wins.

### F2 — §1.2, the energy envelope JSON — TENSE DRIFT

The block shows `{"kind": "energy", "envelope": {...}}`. No `kind` tag exists, no BMS adapter
exists in-tree, and the shipped grant form is `actuator=DEV:dim=lo..hi,...`. A reader could
reasonably try to write that JSON.

**Fixed:** the section now states that the JSON is the *scope* an energy device would carry, shows
the same envelope in the real grant syntax, and says plainly that no BMS adapter exists.

Worth noting the substantive half passes: the `soc_pct: [15, …]` floor — "how a mission planner is
mechanically prevented from draining a rover's battery past the level its heaters need" — is
expressible today and would be enforced by exactly the envelope check 10e built.

### F3 — §1.4, MCU flashing — TENSE DRIFT

"flashing is an actuation-class operation behind an explicit grant and the DL1905 approved-hash
gate" describes a real gate (built, 10f) applied to an operation that does not exist.

**Fixed:** marked *design, not built*, naming what does exist (DL1905, the grant machinery) and
what does not (a flashing operation, an MCU adapter).

### F4 — §2.2, the lost-link two-grant pattern — MISSING MECHANISM ⚠ material

The defining rule of the aircraft domain: at mission upload the operator delegates a mission grant
and a lost-link grant, "a strict `⊑` attenuation". **That requires delegating a *bounded* device
authority, and it is not expressible.** `delulu_broker::Scopes` has dimensions for files, network,
secrets and foreign libraries and none for a device, so a delegating party can say "you may
actuate" but not "you may fly this corridor only".

This is the same gap 10g found from the other end, when `run --lease` turned out to be silently
discarding local device grants (build-order D12e). §2.3 already names its federation gap; §2.2
named nothing, and its pattern is the one most likely to be read as a recipe.

**Fixed:** §2.2 now carries a named gap block — the attenuation is enforced where the program
runs rather than where the mission was uploaded, which is the wrong place for this domain, and the
`--lease` refusal exists so nobody discovers it by having the grant vanish.

## Lines that pass, and why they are worth naming

- **§0, mechanism 2's parenthetical** — *"the dead-man defends against silence, not malice, and no
  document in this project may conflate the two."* This was the addendum's sharpest sentence when
  written and it is now backed by mechanism on both sides: the dead-man (10f) handles silence, and
  operator revocation (10g) is the answer to a program that keeps heartbeating while doing the
  wrong thing. **Upgraded from promise to built.**
- **§1.3** — *"`delulu grants revoke` on a device subtree is the software e-stop … with a
  published revoke-to-fail-state latency budget, never the word 'instant'."* Built in 10g and
  published in `measurements/robotics-demo/RECORD.md` (39.7 ms worst observed, Windows, n=20).
  **The main claim this phase converted into a fact.**
- **§1.3's third bullet** — *"No deployment profile may route a hardware safety function through a
  DeluluLang program … refused at review, every time."* A process commitment, not a mechanism, and
  it says so. That is the honest form: a rule a compiler cannot enforce must not be dressed as one.
- **§2.1** — *"could not command what it was never granted — 'by construction,' not 'provably'."*
  The addendum hedges itself correctly; the mechanized core proof is still future work and the
  sentence already says so.
- **§2.3** — the satellite profile, including its federation gap. 10g's demonstration witnesses the
  semantics it describes: the contact window IS the lease TTL, LOS is that TTL expiring, the
  pre-attenuated grant engages by outliving the pass, re-contact is a new delegation.
- **§2.4** — robot fleets: "revoking one robot kills exactly its subtree". Built and witnessed for
  one host in 10g — `revoking_one_device_leaves_its_sibling_driving` and its parent-revoke twin.
  Across machines it needs §2.5's federation, which §2.4 already defers to.
- **§2.5** — the federation gap. Unchanged, correct, and now carried **verbatim** into
  `measurements/satellite-demo/RECORD.md` and the satellite test's module docs, so a reader of the
  demonstration meets it where they are rather than in a design document they may never open.
- **§3** — certification: "none", four regimes, no hedging. Verified against every surface this
  phase produced (both RECORDs, the Book's new Chapter 16, the spec status row). No sentence in any
  of them reads as a certification claim.
- **§3's WCET/real-time refusals** — consistent with what 10g published: worst *observed*, on one
  named platform, explicitly "not a real-time guarantee".
- **§4** — "No hardware ships in Stage 10." Held. Every number this phase published came from the
  in-tree simulator and says so in its first sentence.

## Verdict

**PASS, after four fixes.** Three were tense/scope drift in device-class sections; one (F4) was
material — a domain's defining pattern resting on a mechanism that does not exist. All four are
corrected in the addendum, and the two structural gaps — **device envelopes are not expressible in
a grant node** (D12e) and **broker federation does not exist** (§2.5) — are now named in every
place a reader is likely to encounter the claims they undermine: the addendum, the build order, the
recordings, and the Book.

The review found what it found because 10g built the mechanisms the addendum described. That is the
argument for doing this review at the end of a build phase rather than at the start of one: before
the code exists, a boundary claim cannot be checked against anything except its author's intent.
