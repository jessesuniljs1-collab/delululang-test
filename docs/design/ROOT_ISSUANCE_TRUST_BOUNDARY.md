# Root issuance and the same-user trust boundary (DISC-1)

**Status: DESIGN + DECISION POINT. The airtight fix is a fundamental, backwards-incompatible change to
how root authority is created, so it is recorded here for the owner's decision, not shipped
unilaterally (Jesse's Authority/Guard guardrail).** Evidence and the executable proof:
`docs/security/red-team-disc1-root-issuance-2026-08-08/`. This document is the rigorous analysis Jesse
commissioned: model the threat, name the boundary, find the root cause, design the minimal change,
state the machine-testable invariant, and name honestly what the code cannot guarantee.

## 1. The threat, modelled precisely

**Adversary:** a process running **as the same OS user as the principal** — a compromised or
autonomous AI agent, a prompt-injected tool-runner, or any same-account program. This is the deployment
DeluluLang markets to (`AI_NATIVE_DESIGN.md`): agents that write and run code, usually in the
principal's own account.

**Adversary capabilities (all follow from same-uid on a normal OS):**
- Read any file the principal can read: `broker.key`, any anchor key on disk, any owner-code file.
- Read the principal's environment (`DELULU_GUARD_OWNER`, `DELULU_STATE_DIR`, …).
- Connect to the broker IPC (the DACL / 0700 socket admits same-user).
- Invoke any `delulu` CLI the principal can, with any flags.
- Read another same-uid process's memory (Windows `PROCESS_VM_READ`; Linux `ptrace` of same-uid unless
  hardened) — so even the daemon's *in-memory* ephemeral owner code is reachable by a determined
  same-user attacker.

**Goal:** obtain **root** authority (which the Guard does not gate) and thereby command a
Guard-**sealed** resource — proven possible today (the note above: seal `device:sat0/hga`, then a
headless caller mints an ungated root holding `{Actuate}` on it).

**Consequence for "secrets".** Against this adversary **no local, same-user-readable secret is a
boundary.** That eliminates, as *security* mechanisms: an owner code (env or daemon memory), a TTY
check (spoofable, and absent for a piped agent anyway), a `--yes`/caller flag, a source-embedded
secret, a label like `holder.kind = "human"`, and any file the agent can read. Each may *raise the
bar*; none is a boundary. This is not a DeluluLang weakness — it is the definition of an OS trust
domain.

## 2. The exact trust boundary

The only durable boundary against a same-uid adversary is something it **cannot possess**:
- **(a) OS-identity separation** — the agent runs as a *different* OS user / container / VM. Then the
  broker's own same-user boundary (DACL / 0700 socket) refuses it, and it can obtain authority only via
  Guard-gated delegations/leases. This is already the written model (`QUESTIONS.md` §1.5) and is
  airtight, but it is a **deployment** choice the code cannot make for the operator.
- **(b) An out-of-band root of trust** — a private key held **outside the same-uid domain** (a hardware
  token / TPM / enclave with a presence or PIN check, a separate service account, or an offline signer).
  A same-uid agent may *ask* for a signature but cannot *exfiltrate* the key, and — with a presence/PIN
  factor — cannot silently obtain signatures either.

Everything else is defense-in-depth on top of one of these, not a substitute for them.

## 3. Root cause — what currently allows same-user root issuance

Root authority enters the broker by **two** paths, and they are not symmetric:

| Path | Authorization required today | Same-user agent can use it? |
|---|---|---|
| `ReqBody::Issue` (`brokerd.rs:288`) — unsigned root from an `AuthoritySpec` | **none** (no owner, no `guard_check_mint`, no signature) | **Yes — this is the hole.** Reachable via `grants delegate` auto-root (`cli.rs:5817`), `run --grant`, `broker_client::issue_root`. |
| `ReqBody::Adopt` (`brokerd.rs:323`) — adopt a certificate **chain** | the chain must **verify against a configured trust anchor**; you cannot forge an anchored cert without the anchor **private** key (`cert.rs`) | Only if it already holds an anchor-signed cert. |

The `Adopt` path is *already* the right shape: root authority is **cryptographically authorized**
against an anchor whose private key need not be on the box. The `Issue` path lets any caller **conjure
a root from nothing**. DISC-1 is exactly the gap between them: the model's security argument assumes
"only the principal creates roots," but `Issue` lets any same-user caller do it, unsigned.

## 4. The minimal architectural change that actually closes it

**Root authority may enter the broker only as an anchored, signed certificate — i.e. via the existing
`Adopt` path — verified against a trust anchor configured at broker init. Unsigned `Issue` no longer
creates a root.** Concretely:

1. The operator configures a **trust anchor public key** when initializing the broker state
   (`<state-dir>/anchor.pub`). Its **private** key is held out-of-band per §2(b) — the code never
   requires the private key to be present, and by default must not store it same-user-readable.
2. Creating a root = presenting a certificate signed by the anchor private key and `adopt`-ing it. A
   same-uid agent lacking the private key **cannot** produce a verifying cert → cannot mint a root.
3. `ReqBody::Issue` (unsigned) is removed as a root-creation path (or hard-restricted to a bootstrap
   that itself requires the anchor signature). `grants delegate` auto-root and `run --grant` change to
   require a pre-issued anchored root, exactly as spec §112 already says ("CI uses pre-issued
   delegation tokens instead").

This introduces **no new authority mathematics** — it reuses `cert.rs` chain verification and the
`⊑` attenuation already model-checked in `Custody.tla` (revocation-survives-readoption, single-adoption
— note P20-R4's chain-extension fix already hardened this path). It is the same insight recorded for
federation ([[delulu-federation-scope]]): "federation needs no new authority mathematics; a certificate
chain runs the existing `attenuation_check` at every hop."

## 5. The machine-testable invariant — and what the code can and cannot guarantee

**What the code CAN guarantee (testable):**
> *No live root node exists in the broker whose creation was not authorized by a certificate chain
> that verifies against the configured trust anchor.*

i.e. `Issue` cannot create a root without a valid anchor-signed cert; every root traces to an `adopt`
of an anchor-verified chain. This is a broker-state invariant checkable in a test and, in principle,
expressible in a `Custody.tla` extension (an `Issue` action guarded by "the spec carries an
anchor-verifying cert").

**What the code CANNOT guarantee alone — named, not papered over:**
> The *security* of that invariant reduces to **"the anchor private key is not in the same-uid trust
> domain."** If the operator stores the anchor private key in a same-user-readable file, a same-uid
> agent reads it, mints its own anchored cert, and the boundary collapses. The code enforces "root
> requires an anchor signature"; the **deployment** must keep the anchor key out-of-band (hardware /
> separate account / offline). This residual is the honest boundary, and it must be documented at every
> surface that describes root creation.

So the strong invariant Jesse stated — *"a requester that does not possess the required owner
authorization cannot mint root authority, regardless of ... same-OS-user process"* — is achievable
**only** with §2(a) or §2(b) supplying the out-of-band factor. The code makes root creation *require*
that factor; it cannot *provide* it. Claiming otherwise would be inventing a guarantee, which this
project refuses.

## 6. Why the weaker options were rejected (option 2's owner-code gate included)

- **Owner-code gate on `Issue`** — the owner code is a same-uid-readable secret (env, or daemon memory
  via same-uid `PROCESS_VM_READ`/`ptrace`). It **raises the bar** (the agent must now actively obtain
  the code rather than call `Issue`), so it is legitimate *defense-in-depth* — but it is **not** the
  boundary and must never be labeled as one. Per Jesse's constraint ("do not solve with a secret the
  agent can read"), it is not the fix.
- **TTY / interactive confirmation** (what spec §112 *claims*) — absent for a piped agent, and
  spoofable; a same-uid agent can allocate a PTY. Not a boundary.
- **A `--yes` flag or `holder.kind="human"` label** — caller-controlled data; the agent sets them.
  (The current `(human)` label on the auto-root is actively misleading and is corrected as independent
  hardening — provenance honesty, not authorization.)

## 7. Decision point (fundamental / backwards-incompatible — owner's call)

Making root creation anchor-signed **breaks** `grants delegate` auto-root and `run --grant` for anyone
who relied on conjuring a root from the CLI, and requires operators to manage an anchor key. That is a
fundamental Authority-model change and a backwards-incompatible CLI/security semantic. Per Jesse's
standing instruction it is **recorded here and presented, not shipped unilaterally.** The independent,
non-fundamental hardening that proceeds regardless: correct the docs to what is enforced, sharpen
`QUESTIONS.md`, fix the `(human)` mislabel, and name this residual boundary in `MATHEMATICS.md` (§7,
outside the proof boundary) and every spec surface that currently overclaims.

## 8. What remains true and unchanged

Delegated grants **are** Guard-gated (the DL1413 on the delegated child in the proof is real). A
**separate-OS-account** agent cannot reach the broker at all. The custody core (attenuation, revocation
incl. the P20-R4 chain-extension fix, single-adoption, lease integrity) holds. DISC-1 is not a break in
any of those — it is the `Issue` path never having been held to the same standard as `Adopt`.
