# Deploying DeluluLang

This document answers one question: **what does a DeluluLang deployment actually protect, and what do
you have to do to get that protection?**

It is written for the case the language is built for — running programs, including ones written by
autonomous agents, with a bounded and auditable set of capabilities. Every claim here is either
verified on this project's own bench (Windows 11 + Linux) or explicitly marked as unverified.

---

## 1. The one thing to understand first

DeluluLang's containment is enforced by a **broker**: a separate process that holds the grant tree and
authorizes every effectful operation. That gives you real properties — attenuation, revocation,
expiry, a hash-chained audit log, and the Guard.

It does **not** give you protection against a process running as *the same OS user*. To the kernel,
that process and the broker are the same principal: it can read the broker's key, edit its policy
files, or kill it. No amount of code changes that, and this project does not claim otherwise (it is
called *category 7* — outside the proof boundary — in `MATHEMATICS.md`).

**Therefore the single most important deployment decision is: what OS user does the untrusted code
run as?** Everything below follows from that.

---

## 2. Three tiers, and what each is worth

### Tier 0 — one user, legacy mode (the default; development)

```bash
delulu broker start
delulu run app.delulu --grant console --grant fs.write=./out
```

**What you get:** the full capability model — a program can only do what its grant allows, every
effectful op is authorized, everything is audited, and revocation works.

**What you do not get:** protection from the code you are running, if it is hostile *and* runs as
you. Any process on your account can mint root authority (`DISC-1`) and command even a
guard-sealed resource.

**Use it for:** your own code, on your own machine. This is a development posture, and it is a
legitimate one — but do not point it at an untrusted agent and expect the Guard to hold it.

### Tier 1 — one user, strict anchored roots (the anchor key lives elsewhere)

Root authority can now only enter by adopting a certificate that verifies against an anchor whose
**private half is not on this machine**.

```bash
# ON A DIFFERENT MACHINE (or offline media) — this key never touches the delulu host:
delulu grants pubkey --key anchor.key            # prints ANCHOR (the public half)

# On the delulu host:
delulu broker start --require-anchored-roots <ANCHOR>

# Back on the signing machine — mint a root certificate for a holder, with ABSOLUTE scopes:
delulu grants pubkey --key holder.key            # prints HOLDER
delulu grants certify --subject <HOLDER> --effects Write \
      --fs-write /srv/app/out --ttl 1h --key anchor.key --out root.dlcert

# On the delulu host — adopt it, then delegate normally:
delulu grants adopt root.dlcert --anchor <ANCHOR>
delulu grants delegate --parent <node> --effects Write --fs-write /srv/app/out --owner <code>
delulu run app.delulu --lease <token>
```

**What you get on top of Tier 0:** a same-user process **cannot manufacture root authority**. Without
the anchor private key it cannot mint a certificate, and unsigned issuance is refused `DL1421`.

**What you still do not get:** that process can edit `<state>/root_policy.json` or restart the broker
without the flag, downgrading you to Tier 0. It cannot do so *quietly* — every start records its
effective mode in the hash-chained audit log — but it can do it.

> **Scopes in a certificate must be ABSOLUTE.** A relative path names a location only the signing
> machine's working directory could mean; `certify` refuses it and tells you the absolute form.

### Tier 2 — separate OS account (the actual boundary)

Run the broker as one user and the untrusted program as another.

**This is the only configuration in which the boundary is enforced by something other than the
untrusted code's good behaviour**, and it is the one this project verified with a real second UID:
on a POSIX filesystem an attacker account was denied on all five tested vectors, while the
same-account control passed (`docs/security/red-team-p21-crossaccount-2026-08-08/`).

**One prerequisite, and it is not optional:** the state directory must be on a filesystem that
actually enforces permissions. On 9p / DrvFs / NFS / SMB / exFAT a `chmod` can be a silent no-op — a
second account read a `0600` file and the broker key straight off such a mount (finding P21-F1). The
broker now refuses to write a secret onto such a filesystem, and `delulu doctor` reports it.

---

## 3. Verify it — do not assume it

```bash
delulu doctor
```

```
security posture
  ok       root issuance          STRICT — a root may enter only by adopting a certificate that
                                  verifies against `abc123…`; unsigned issuance is refused DL1421
  ok       anchor key custody     no signing key in the state directory
  ok       state dir permissions  on a filesystem that enforces owner-only permissions
```

| What doctor says | What it means |
|---|---|
| `root issuance … STRICT` | Tier 1 or 2. Unsigned root creation is refused. |
| `root issuance … LEGACY` (note) | Tier 0. A same-user process can mint root authority. Supported, but know it. |
| `root issuance … UNREADABLE` (problem) | The policy file is corrupt. The broker is refusing **all** root creation until you repair or remove it. Fail-closed by design. |
| `anchor key custody` (note) | A signing key is sitting next to the state it protects. If that is your anchor, you are back to file permissions — move the private half off the box. |
| `state dir permissions` (problem) | The filesystem cannot keep a secret from other local users. A separate OS account buys you nothing here. Move the state directory. |
| `running broker mode` (problem) | **The policy on disk and the running daemon disagree.** A daemon serves the mode it *booted* with, so a policy written after it started has not taken effect. Restart the broker. |
| `state dir reachability` (note) | Whether **this process** can write the broker's state. See below — this is the one that tests Tier 2. |

### Testing the Tier-2 boundary — run doctor as the agent account

Every other line reports what you *configured*. Two report what is *true*:

- **`running broker mode`** compares the policy file against the mode the live daemon actually booted
  with, read from the `root-policy-mode` record in the hash-chained audit log. Without it, `root
  issuance` would happily say `STRICT` while the running broker went on minting unsigned roots —
  configuration reported as though it were behaviour.
- **`state dir reachability`** is a *capability* test, not an identity claim: it tries the write. Run
  `delulu doctor` **as the account your untrusted programs run as**. If it reports that the process
  *can* write the state directory, **Tier 2 is not in force** — that account can read the broker key
  and edit the root policy. Being refused is what Tier 2 looks like.

Doctor cannot know which account it was invoked as, and does not guess. It tells you what this
process can do; you know which account that is.

Run `delulu doctor` **on the deployment host**, not on your laptop — it reports on the state directory
the broker actually uses (`DELULU_STATE_DIR` if set).

To check after the fact whether a broker was ever downgraded:

```bash
delulu audit verify          # proves the chain was not rewritten
delulu audit tail | grep root-policy-mode
```

---

## 4. Platform status — stated exactly

| Platform | Status |
|---|---|
| **Linux** | Verified. Full suite executed; cross-account boundary tested with a real second UID. |
| **Windows 11** | Verified. Full suite executed. Note that unprivileged symlink creation is not available, which blocks one class of workspace-delivered attack that POSIX permits. |
| **macOS** | **Verified by CI** (Apple Silicon runner, 2026-09-14): the whole suite — 1,657 tests, 0 failed — plus the CLI sweep, the fuzz campaign and every other gate in `ci.yml`. It links macOS's own libffi, because the copy `libffi-sys` bundles does not assemble with current Apple clang. **On a Mac, keep `DELULU_STATE_DIR` short** — the socket lives inside it and macOS allows 103 bytes (`SUN_PATH_MAX`); `broker start` refuses a longer one by name. The Tier-2 cross-account boundary was tested with a real second UID on Linux, not on macOS. |

---

## 5. What is not protected (read this before shipping)

These are known, documented, and not fixable by configuration:

- **A same-OS-user process.** Tiers 0 and 1 do not contain it. Tier 2 does.
- **A concurrent writer into a granted directory.** Filesystem containment is a check-then-open, so a
  second writer can swap a checked file for a symlink in between (`CONTAIN-TOCTOU-1`). **Grant scopes
  that point at directories only the program's own user can write** — never a shared or
  world-writable one.
- **A hardlink inside a granted directory.** A hardlink is a genuine second name for one file, so
  containment reports it as inside the grant, because it *is*. Creating one requires access to the
  target already, and git cannot carry one.
- **Resource exhaustion.** A program may consume its own CPU and memory. Authority is the
  containment, not a quota.
- **Foreign code.** `foreign.c` / Python bound through a grant is outside the effect guarantee: the
  language bounds *reachability*, not behaviour.
- **The hardware adapter.** `--adapter-cmd` runs an operator-supplied subprocess. The envelope is
  enforced host-side before dispatch, but the driver itself is not sandboxed and is not signature-checked.
- **The microVM isolation profile — because it does not exist yet.** `delulu run --isolation microvm`
  is a *probe*: it looks for `/dev/kvm` and a VMM binary, names whichever is missing, and then
  refuses with `DL1408` **even when both are present**, because the guest launch (read-only rootfs,
  virtio-fs mounts matching the granted `fs.*` scopes, default-deny egress proxy, vsock broker
  proxy) is unwritten. **This is the correct failure mode** — you never silently get weaker
  isolation than you asked for — but do not plan a deployment around it. The profiles that work are
  `none` and `process` (the foreign worker). For genuinely untrusted code the boundary is Tier 2
  above, not a VM. Detail: [`REMAINING_WORK.md`](REMAINING_WORK.md) §4.1.

---

## 6. Why strict mode is not the default (ruling, 2026-08-10)

It would be easy to flip the default and call the system safer. That would be a mistake, and the
reasoning is recorded here so it can be argued with:

1. **It would not close the hole it appears to close.** Strict mode's own residual is that
   `root_policy.json` is same-user writable. Defaulting it protects nobody who was not already going
   to run Tier 2 — while *sounding* like it does, which is the worst property a security default can
   have.
2. **It breaks the primary workflow.** `grants delegate` with no `--parent` mints its root through the
   unsigned path. Under strict mode every such call fails until an anchor keypair exists and a
   certificate has been minted and adopted. That is correct for a production deployment and hostile
   for the single-user development case that is most of the usage.
3. **v1.x compatibility.** Changing what an existing `broker start` does is a breaking change to a
   released 1.0.

**The decision:** legacy stays the default; the choice is made **visible** instead — a startup banner
in all three states, a `delulu doctor` line, and a permanent record in the audit chain. Strict +
Tier 2 is the documented production posture, and **strict is the intended default at the next major
version**, at which point the ergonomic break can be paired with the migration it needs.

---

## 7. Quick reference

```bash
delulu doctor                                   # is this deployment sound?
delulu broker start --require-anchored-roots X  # Tier 1/2
delulu audit verify                             # has the log been tampered with?
delulu audit tail | grep root-policy-mode       # was this broker ever downgraded?
delulu grants list                              # what authority exists right now?
delulu guard policy show                        # what is gated, and at which tier?
```

Related reading: [`design/ROOT_ISSUANCE_TRUST_BOUNDARY.md`](design/ROOT_ISSUANCE_TRUST_BOUNDARY.md)
(the full threat model), [`QUESTIONS.md`](QUESTIONS.md) (the hard questions, answered with evidence),
and `security/red-team-p21-crossaccount-2026-08-08/` (the cross-account verification).
