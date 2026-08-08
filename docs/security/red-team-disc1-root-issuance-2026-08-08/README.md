# Red-team DISC-1 — root issuance is ungated and headless, defeating a device seal (2026-08-08)

**Status: FOUND, proven executably. Resolution PENDING an owner decision (it touches the Authority
model — do NOT unilaterally change root-issuance semantics).** Discovered in the post-F-CUSTODY
discovery phase (the standing directive: *"what security assumption have we not attacked yet?"*).

## The claim under attack

The Guard gates **delegated** (non-root) grants; root nodes are "principal-held by construction and
pass without guard interaction" (`guard.rs`). The whole model therefore rests on: *only the principal
creates roots.* Two documents state a control that is supposed to make that true:

- **Constitution §5.16 law 4** — root issuance "is never a programmatic path."
- **STAGE5_SPECIFICATION.md:112** — `issue` requires interactive confirmation; *"root issuance is
  never headless-silent; `--yes` in a TTY-less session is refused; CI uses pre-issued delegation
  tokens instead."*

**Neither control is implemented.** The daemon's `ReqBody::Issue` (`brokerd.rs:288`) is completely
ungated — no owner code, no `guard_check_mint`, unlike `Delegate`/`Attenuate`. There is no
`grants issue` command at all; a root is minted by `grants delegate`'s **auto-root** path
(`cli.rs:5817-5845`, when no `--parent`) and by `run --grant` — and neither prompts, checks a TTY, or
refuses headless. The comment at `cli.rs:5821` even asserts the aspiration ("Root issuance stays a
human action … never a programmatic path") directly above code that a headless program invokes freely.

## The proof (executable, against the real broker)

```
# operator seals the device — the strongest guard tier
guard policy set "device:sat0/hga" sealed        # device:sat0/hga → sealed

# a HEADLESS caller (stdin is NOT a tty, NO owner code) mints Actuate authority
grants delegate --effects Actuate --device "sat0/hga:slew_deg=-45..45,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park"
#   → the DELEGATED child is refused DL1413 (the guard works for delegation)
#   … but:

grants list
#   g_4b7dcf…  [live]  parent=-  effects={Actuate}  device=[sat0/hga:slew_deg=-45..45,…]  (human) "delulu grants delegate (root)"
```

A **live, ungated ROOT** holding `{Actuate}` on the sealed device now exists — minted headlessly, with
no owner code, **left behind even though the subsequent delegation was guard-refused.** Because roots
pass the Guard without interaction, that root can command the device the operator just sealed.

## Three facets

1. **Root issuance is ungated and available to any same-user program, headlessly** — contradicting
   Constitution §5.16 law 4 and spec §112. The stated interactive/non-programmatic control does not
   exist in code.
2. **A guard seal does not contain a same-user agent.** An operator who seals `device:sat0/hga` to
   stop an agent from commanding it is defeated by the agent minting a fresh root with that authority.
   The Guard gates delegation, not root creation. For the AI-agent deployment DeluluLang explicitly
   targets — agent sharing the principal's OS account, the common case — the Guard provides **no**
   containment.
3. **Provenance is mislabeled.** The auto-root is recorded `(human)` / `holder_desc = "delulu grants
   delegate (root)"` regardless of who invoked it, so a programmatic mint is audited as a human act.
   (`holder.kind` is DATA, never switched on — KIND_IS_DATA — so this is an audit-honesty issue, not
   an authorization one. But it means the audit trail cannot distinguish a human root from an agent's.)

## Is this "within the threat model"?

Partly — and the honest answer is the point. The written model is *one holder per host; untrusted
agents on separate OS accounts* (`QUESTIONS.md` §1.5). Under THAT model a same-user program is the
principal, and minting a root is legitimate. **But the Constitution and spec claim a STRONGER control
that would protect even against a same-user program, and that control is not implemented** — so the
docs overclaim a protection that does not exist. And the deployment the project markets to (AI agents,
`AI_NATIVE_DESIGN.md`) overwhelmingly runs the agent same-user, where the overclaim is exactly the gap
that bites: "I sealed the device so the agent can't command it" is **false**.

## Recommendation (OWNER DECISION — not taken unilaterally; Authority/Guard guardrail)

Two honest resolutions, not mutually exclusive:

- **(A) Documentation, always correct, do first.** Correct Constitution §5.16 law 4 and spec §112 to
  state what is actually enforced (root issuance is a same-user action; the boundary is the OS
  account; there is no interactive/headless enforcement), and sharpen `QUESTIONS.md` §1.5/§1.6 so the
  Guard/same-user/AI-agent consequence is explicit: *a same-user process can mint a root and command
  a sealed device; run untrusted or AI agents as a separate OS user.*
- **(B) Enforce the control (a real hardening, but it changes root-issuance ergonomics — owner's
  call).** Make root issuance refuse in a TTY-less session unless the **owner code** is presented
  (mirroring the guard-admin gate), so a headless agent without the code cannot mint a root. Trade-off:
  it changes `grants delegate`'s auto-root and `run --grant` for scripts/CI (which the spec already
  says should use pre-issued tokens), and a same-user agent that can read the owner code (e.g. from the
  principal's env) is still not contained — so (B) raises the bar without being airtight, and must not
  be sold as more than it is. Also fix the `(human)` mislabel to reflect actual provenance.

The load-bearing guarantee that DOES hold, unchanged: **delegated** grants are guard-gated (the DL1413
on the child above is real), and a **separate-OS-account** agent cannot reach the broker at all.
