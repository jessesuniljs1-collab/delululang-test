---
name: delulu-root-issuance-bypass
description: "DISC-1 — root issuance is ungated/headless so a same-user agent can mint a root and command a guard-SEALED device. MITIGATED by opt-in strict mode (SHIPPED 2026-08-08); NOT 'fixed' — the airtight boundary is anchor-key custody (category 7), and a strict DEFAULT is still a pending major-version decision."
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-08T12:50:38.663Z
---

**RESOLUTION (2026-08-08, Opus 4.8, Jesse's security-migration experiment): opt-in strict mode SHIPPED,
commits `412fa98` → `2be5839` → `222f617`.** `broker start --require-anchored-roots <anchor-pubkey>`
persists strict mode (`<state>/root_policy.json`, PUBLIC anchor only). Then `issue()` is `pub(crate)`
(reached only by `adopt` after verification + in-crate tests) behind a single public `issue_root` gate
that refuses **DL1421** (new code) in strict mode — so the daemon `Issue` dispatch, `grants delegate`
auto-root, `run --grant`, embedded custody, and any library caller are gated with no bypass; `adopt`
AND `renew` verify against the PINNED anchor (Phase-8 finding: renew had to pin too, else a forged
receipt extends an adopted node's TTL past the uplink lease). Broker-boundary invariant
(`unjustified_root_nodes()` empty) is **category 4** (property-tested + FALSIFIED: reintroducing the
unsigned hole fails attack 1, removing the pin fails attack 2) + a deterministic wire test + a live run.
**The SECURITY is category 7**: it reduces to keeping the anchor private key AND `root_policy.json`
outside same-uid reach (a same-uid agent can still downgrade by tampering the file or restarting without
the mode) — a separate OS account / hardware anchor is the real boundary. NOT "DISC-1 fixed"; a strict
DEFAULT is a fundamental/major-version change (migration analysis: trust-boundary doc §10). "The code
verifies the signature" ≠ "secure against same-user compromise." Conflation-free in MATHEMATICS.md item
11 + §12. Cross-platform: Windows+Linux green, macOS pure-Rust/compile-blocked.

**DISC-1 (2026-08-08, Opus 4.8), found in the post-F-CUSTODY discovery phase. Proven executably.** Full
note + proof: `docs/security/red-team-disc1-root-issuance-2026-08-08/`; architecture + evidence +
migration: `docs/design/ROOT_ISSUANCE_TRUST_BOUNDARY.md`.

**The finding.** The Guard gates DELEGATED nodes only; roots "pass without guard interaction." Two
docs claim a control that makes "only the principal creates roots" true — Constitution §5.16 law 4
("root issuance is never a programmatic path") and STAGE5_SPECIFICATION.md:112 ("never headless-silent;
`--yes` in a TTY-less session is refused"). **Neither is implemented.** `brokerd.rs` `ReqBody::Issue`
is completely ungated (no owner code, no `guard_check_mint`); there is no `grants issue` command — a
root is minted by `grants delegate`'s AUTO-ROOT (`cli.rs:5817`) and `run --grant`, neither prompting,
checking a TTY, nor refusing headless.

**The proof (real broker):** operator seals `device:sat0/hga`; a HEADLESS caller (no owner code) runs
`grants delegate --effects Actuate --device sat0/hga:…`; the delegated child is refused DL1413 (guard
works) **but `grants list` shows a live ungated ROOT holding `{Actuate}` on the sealed device, minted
anyway, left behind, mislabeled `(human)`.** Because roots bypass the Guard, that root can command the
sealed device. So **a guard seal does not contain a same-user agent** — and that is the common
AI-agent deployment DeluluLang markets to. "I sealed the device so the agent can't command it" is
FALSE when the agent shares the OS account.

**Within the model?** Partly: the written model is one-holder-per-host, untrusted agents on separate
OS accounts ([[delulu-federation-scope]] / QUESTIONS.md §1.5), where same-user = principal. But the
Constitution/spec OVERCLAIM a stronger control that isn't there. Honest resolution has two parts:
(A) DOCUMENTATION (always safe, do regardless): correct §5.16 law 4 and spec §112 to what is enforced,
sharpen QUESTIONS.md §1.5/§1.6 to make the Guard/same-user/AI-agent gap explicit. (B) ENFORCEMENT
(owner's call, changes ergonomics): gate root issuance behind the owner code in TTY-less sessions —
raises the bar but a same-user agent that can read the owner code is still not contained, so it must
not be oversold. Also fix the `(human)` mislabel.

Related: [[discovery-over-checklist]] (this is what "the checklist didn't know to ask" looks like),
[[delulu-hw-adapter]], the harden-never-redefine rule in [[delulu-hardening-campaign]].
