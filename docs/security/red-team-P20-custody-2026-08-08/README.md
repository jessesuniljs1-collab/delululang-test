# Red-team P20 — custody core, multi-agent (2026-08-08)

Four AI agents (2× Sonnet 5, 2× Haiku 4.5) run as untrusted parties against the broker, certificates,
lease tokens, audit chain, and Guard. The operator (the supervising session) held the ground anchor
private key; agents received only the vehicle key, a bounded certificate, and the public anchor. The
security claim under test: **without the anchor key, no agent can forge, widen, or evade the authority
the operator controls.** Agent programs and notes are in `agent-notes/`. **Every agent claim was
re-verified independently by the operator against the current tree** — agent output is evidence, not
verdict.

## Verdicts

| Agent | Model | Target | Agent claim | Operator verdict |
|---|---|---|---|---|
| 1 | Sonnet 5 | certificates & adoption | 5 held, 1 BROKE (revocation evasion) | **Already fixed.** The "BROKE" is finding P20-R4, closed in commit `a59ae34` *before* this run; the agent tested a stale binary built earlier in the session. Re-verified against the current binary: `[legit, ext]` and a second extension are both refused `DL1415`, revocation holds. The other 5 (wrong anchor, no anchor, self-signed, widening at mint, field tamper) held. |
| 2 | Haiku 4.5 | lease tokens | all 6 held | **Confirmed.** Single-use replay `DL1407`, MAC tamper/forge `DL1407`, redeem-after-revoke `DL1403`, cross-broker `DL1407`. Matches the operator's own lease analysis (MAC is per-broker `keyed_hash`, redeem checks node liveness incl. ancestors — C29). |
| 3 | Sonnet 5 | audit chain | tamper-detection works; 1 operational finding | **Confirmed.** Modify/reorder/truncate all detected (`DL1405`, nonzero exit, read paths warn "must not be trusted"); the consistent log+anchor rewrite is the documented limit (needs an external witness), not a new hole. **The operational finding is real and tracked below.** |
| 4 | Haiku 4.5 | the Guard | CRITICAL: "sealed rules are runtime-approvable" | **FALSE POSITIVE.** See below — the agent stopped at "a permit was minted" and never tested whether the permit authorizes anything. It does not. |

## Agent 4's "sealed bypass" is a false positive — the guarantee holds

The agent set `declassify:*` to `sealed`, then `request`ed and `approve`d it with the owner code,
minting a permit, and concluded the sealed tier is "no stronger than guarded." That conclusion is
wrong, and the code proves it:

`guard.rs::guard_verdict_use` matches the tier and returns **`GuardTier::Sealed =>
Block(GuardSealed)` — `DL1413` — unconditionally, before any permit is consulted.** Permits are only
read inside the `Guarded` arm. So a permit for a sealed class is **inert**: at use time the sealed op
is refused `DL1413` regardless of whether a permit exists, and bypass does not lift it either. This is
exactly what the existing unit test at `guard.rs` (*"a sealed rule refuses DL1413 even with an
approved request/permit"*) already pins — `guard_verdict_use(...) == Block(DL1413)` **with a permit
present.** The agent never triggered a use-time check, so it never saw the permit do nothing.

The design (`STAGE5_GUARD_ADDENDUM.md`): `sealed` — *"refused with DL1413 always. Not approvable at
runtime; only a principal policy edit (owner-coded) can unseal. Bypass does not lift sealed."* The
runtime guarantee is intact.

## Real findings, tracked (minor, not security escalations)

- **F-CUSTODY-1 — `guard request`/`approve` accept a `sealed` class and mint an inert permit.** The
  approval *workflow* does not refuse a sealed class the way use-time and mint-time do; it mints a
  permit that is inert while the rule is sealed. It is not a runtime bypass (proven above). The
  residual concern is latency: a permit minted while sealed would become **effective if the rule is
  later unsealed** to `guarded`, without a fresh approval — a small gap against the design's intent
  that a sealed rule be "not approvable at runtime; unseal first." Proportionate fix: `request`/
  `approve` refuse a sealed class with `DL1413`, matching use-time and mint-time. Tracked for a
  hardening pass; the load-bearing guarantee (use-time refusal) already holds and is tested.

- **F-CUSTODY-2 — `delulu audit verify|tail|query` default to `~/.delulu/audit`, not the broker's
  `$DELULU_STATE_DIR/audit`.** Forgetting `--dir` verifies a *different, global* log and can report a
  confident `ok` for an unrelated chain. Not a chain-integrity break — the mechanism is sound — but
  the same "reads the wrong store" footgun class as hardening finding C75 (`audit --dir` vs
  `--state-dir`). Worth aligning the default (or refusing without an explicit target when a state dir
  is set). Tracked.

## What held — the custody core is genuinely well-built

Anchoring (no anchor key ⇒ no adoptable authority), attenuation (widening refused at mint *and*
adopt), signature scope (every field covered), lease-token integrity (per-broker MAC, node-liveness,
single-use, constant-time), audit tamper-evidence (modification/reordering/truncation all detected),
and the Guard's sealed/guarded/owner-code enforcement all held under adversarial AI. The one true
security hole of the campaign — **P20-R4, revocation by chain extension** — was found by the operator
before this multi-agent run and is fixed, tested, and committed.
