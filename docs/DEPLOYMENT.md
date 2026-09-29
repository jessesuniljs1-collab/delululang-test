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

**The recipe, per platform.** The broker and its state directory belong to your account; the program
runs as another, which cannot read `0700` directories it does not own.
- **macOS** (no unprivileged second identity exists, so this is the only one): create a standard user,
  `sudo sysadminctl -addUser delulu-agent`, and run the program as it —
  `sudo -u delulu-agent delulu run app.delulu --grant …`. The grants are yours to type; the agent
  account cannot reach your broker (it accepts only clients running as its own OS user), your state
  directory or your files, and its own `DELULU_STATE_DIR` must be one it owns.
- **Linux**: the same with `sudo useradd --system delulu-agent` and `sudo -u delulu-agent …`. Check the
  state directory's filesystem first (below).
  A `--sandbox` guest already runs as a subordinate uid where the host allows user namespaces (PS-B-03b):
  it needs `newuidmap`/`newgidmap` (the `uidmap` package), a range for your user in `/etc/subuid` and
  `/etc/subgid` (`useradd` makes one), and — on Ubuntu 23.10 and later — the AppArmor restriction lifted
  (`sysctl kernel.apparmor_restrict_unprivileged_userns=0`), a host-wide choice that is yours to make.
  `delulu doctor`'s `identity separation` line says whether it applied here, and why not if it did not.
- **Windows**: a second local account (`net user delulu-agent /add`) and `runas /user:delulu-agent`.
  A `--sandbox` guest already runs as a per-run AppContainer (PS-B-03); the account is what covers the
  CLI and an unsandboxed run.

### The sandbox — a second boundary, under the account, not instead of it

`delulu run app.delulu --sandbox` runs the program as a guest process that holds **no authority of
its own**: its capabilities are opaque handles, and the host performs every effect under the same
checks a normal run makes. The operating system confines the guest as well, and each platform
enforces what it actually has:

| Platform | What the guest is held to |
|---|---|
| Windows | a Job Object: one process, a memory ceiling, a processor-time ceiling (the job's own limit, which Windows checks late, and since 2026-09-28 the host reading the job's accounting every 100 ms and stopping the guest at the budget — SANDBOX-CPU-LATE-1), killed with the host, no desktop, clipboard or global atoms — applied to a SUSPENDED child, before its first instruction; and since PS-B-03 **a separate identity**: a per-run AppContainer with no capabilities, so no network of any kind and none of the operator's files, the state directory included (T14, measured with a control) |
| Linux | `no_new_privs`, `PDEATHSIG`, heap and processor-time ceilings, no core dump; then, installed by the guest on itself, a Landlock ruleset — **nothing writable anywhere**, reads only from the system paths (`/usr`, `/lib`, `/etc`, `/sys`, its own `/proc/self`, the null, zero, full and random devices, and its own channel directory — since PS-E-03 not all of `/proc` or `/dev`), and no TCP bind or connect — and a seccomp filter: no new programs, no debugger, no namespace, mount or kernel-module calls (nor a namespace through `clone`), **no new sockets of any family**, and none of `memfd`, `io_uring`, `userfaultfd` or `pidfd` (PS-E-03, each witnessed against an escaped guest) Where the host allows user namespaces, **a separate identity** as well (PS-B-03b): a per-run subordinate uid in its own user namespace, with no supplementary groups or capabilities, so none of the files only your account may read (T14, measured with a control); the report says `same OS user` where the host forbids it. |
| macOS | a processor-time ceiling and no core dump (`setrlimit` before `exec`, both measured: a spinning program with a one-second limit was killed by SIGXCPU after one second against a control that ran sixteen, run `35480762820`; **no** memory ceiling is claimed or even requested, because `setrlimit(RLIMIT_DATA)` returns EINVAL on macOS — measured in the same run — and the time ceiling mattered most here, since macOS has no `PDEATHSIG` and a guest that is computing rather than asking would not notice its host had died — since PS-E-02, D-V2-60, **a watcher process outside the guest ends it when its host is gone**, claimed only once armed), plus a **deny-default** Seatbelt profile: nothing is permitted but reads, `sysctl-read`, the guest's own `exec`, and its channel socket — so no file writes, no network but the channel, no new programs, no Mach services, no signals or process info beyond itself. Each of those four allowances was measured load-bearing by removing it (run `35479148216`); reads are NOT narrowed, because every attempt to confine them by subpath aborts the guest, so on macOS a guest can still read the filesystem and only the other layers stop it acting on what it read — except the state directory, which the profile refuses since PS-B-03 (T14). Measured on macOS 26.6.2 arm64: a future release needing another allowance makes the run REFUSE rather than fall back to a weaker profile |

Read the run report (`--report-out F`) rather than the program's output: it names the requested and
actual level, the backend, the limits, the mode, the guarantees the host **actually applied**, and a
`policy_hash`. It also names what is **not** enforced — `limitations`, with `fully_enforced` true only
when the sole entry is `identity_separation` — and `denied`, every attempt the host refused, with its
code. A run report that listed only guarantees would read as though the rest were covered. The program
can write to its own output but not to that file.

On every platform a process guest may also be given `--limits wall=SECONDS`, a wall-clock ceiling the host's own watchdog enforces,
and a guest stopped by any ceiling is named as such — in words, in the report's `outcome.stopped_by`,
and in an exit code that agrees with the report (SANDBOX-STOP-1).

The Linux Landlock ruleset and seccomp filter are the guest restricting ITSELF. Since 2026-09-25
(`REMAINING_WORK.md` 4.23) the run report counts them: the guest reports them to the host as its first
request, after its lock-down and before the program's first line, and the host accepts that only
first, only once, and only in its own words for those layers — anything else refuses the whole run.
The guest also says what it applied on standard error, and on a kernel with no Landlock it says that
instead — an absent boundary never reads like an applied one. `delulu sandbox probe` asks the
running kernel for its Landlock ABI directly; a kernel below ABI 3 does not mediate `truncate` and a
kernel below ABI 4 does not mediate TCP, and the guest's report says which of the two it got.

**Requiring it (PS-B-06).** `delulu sandbox require --break-glass-key <public key>` makes the sandbox
mandatory for every program `delulu` runs on this host: `run` without `--sandbox`, `test` and `repl`
are refused. The way past it is break-glass, and only you hold it: `delulu keygen --name breakglass`,
move the private key off the host, and in an emergency sign a ticket for one program where the key is
(`delulu sandbox ticket --key K --program app.delulu --ttl 15m --reason "…" --out T`), then
`delulu run app.delulu --break-glass T`. The ticket works once, for that program's exact bytes, for at
most a day; the run says so on standard error, in its report and in the audit chain, and changes
nothing but the sandbox. `sandbox release` with a `--release` ticket takes the policy off. A process
running as you can still delete the policy file — Tier 2 is what stops that.

**What this is not.** It is not a substitute for Tier 2. On Linux and macOS the guest runs as the same
OS user, so it is a second wall under the account boundary, not instead of it. On Windows the guest is
a separate identity, but the broker, the CLI and every run without `--sandbox` are still you. And it does not carry every program yet:
actors, foreign C, Python, plugins and devices are **refused** before anything runs, rather than run
unconfined — which `delulu sandbox policy <file> --json` reports in advance — and a secret is refused
at its first use, because a secret's bytes never cross the channel.

**The microVM (L2).** On Linux x86_64 with KVM, `--isolation microvm` runs the same guest as PID 1 of
its own kernel under Firecracker (as root only under Firecracker's jailer; root without it is refused), with vsock and nothing else: no
network device, no filesystem device, the image checked against its manifest on the copy that boots.
You build the image from source (`scripts/microvm/build-image.sh`); it is not distributed (D-NE-27).
Elsewhere, and without KVM or an image, the run refuses with `DL1408` — never a weaker boundary in
silence.

**An external launcher (L3, PS-D-01).** `--sandbox-backend external:CMD` lets a boundary DeluluLang does
not build carry the guest: Docker with gVisor, Kata, a cloud sandbox, a Kubernetes pod, `ssh` to another
machine. Your command runs the guest in YOUR environment and carries the channel on its standard input
and output; DeluluLang tells it the guest's words (`DELULU_GUEST_ARGS`, `__guest --stdio-pipes`) and the
limits the run asked for (`DELULU_LIMIT_MEMORY_BYTES`, `…_CPU_SECONDS`, `…_WALL_SECONDS`). The command's
words are split on whitespace and run with no shell — anything that needs quoting belongs in a script.
The guest still holds no authority: every effect it asks for is decided and performed on the host, under
your grants, lease and Guard, exactly as at L1. What changes is who vouches for the wall around it: the
run report says **level 3, backend `external`, and no host guarantee** — DeluluLang measured none of that
boundary, and says so; it names the launcher's program (never its arguments, which can carry a token).
A reference recipe — **documented, not shipped, and not tested by this project** — for Docker with gVisor
and no network:

```sh
#!/bin/sh
# /usr/local/bin/delulu-gvisor — run with:
#   delulu run app.delulu --grant … --sandbox --sandbox-backend external:/usr/local/bin/delulu-gvisor
# The image must carry the SAME delulu version as the host (the channel is versioned — since
# 2026-09-28 `delulu-sandbox-channel/3`, where the guest confirms its boundary before it is sent the
# program; a guest of another version refuses in words), and needs no
# files of yours: the guest performs no effects, so mount nothing.
exec docker run -i --rm --init --network none --runtime=runsc \
  --read-only --cap-drop ALL --security-opt no-new-privileges --user 65534:65534 \
  --memory "${DELULU_LIMIT_MEMORY_BYTES}" --pids-limit 64 \
  delulu-guest:1.0.0 delulu $DELULU_GUEST_ARGS
```

Two things this recipe does not do for you. The host's wall-clock ceiling (`--limits wall=`) ends the
LAUNCHER's process, and ending a `docker` client does not end its container — give the container its
own time limit, or run the launcher under something that reaps it. And `--memory` bounds the container,
not the program's budget as DeluluLang accounts it: the run report records what was asked, and the
launcher is what enforces it.

**Requiring an attester's word (PS-D-02).** `--require-attestation HEX` (with `external:` only) makes the
run refuse to serve a guest nobody vouched for. The host picks a fresh nonce for the run and gives the
launcher `DELULU_ATTEST_NONCE` (64 hex digits) and `DELULU_ATTEST_OUT` (a path in the host's own per-run
directory). Before the program is sent, a document must appear there, written whole (a temporary file,
then a rename), within 10 seconds of the launch:

```json
{"format": "delulu-attestation-v1",
 "statement": {"attester": "ci-image-builder", "guarantees": ["runsc", "no network"], "nonce": "<DELULU_ATTEST_NONCE>"},
 "signature": "<hex of the signer's 32-byte public key followed by its 64-byte ed25519 signature>"}
```

The signature covers `delulu-attestation-v1` and a newline, followed by the statement's canonical JSON —
keys in byte order, no whitespace; Python's `json.dumps(statement, sort_keys=True, separators=(",", ":"),
ensure_ascii=False)` writes those bytes. The host checks the signature, that the signer is the key you
pinned, and that the nonce is this run's; a claim is printable text of at most 256 characters, at most 32
of them, and an unknown field anywhere is refused. On any failure — another key, no document, a launcher
that exits first, a document replayed from another run — the guest is ended having been told nothing,
the run exits 1 in words, and the refusal is in the audit chain (`sandbox-attestation`). On success the
report carries `sandbox.attestation = {key, attester, guarantees, verified: true}`: the ATTESTER's claims,
beside `host_guarantees` and never merged into them. The level stays 3 — DeluluLang still measured none
of the wall; it checked who said what about it.

Who the attester is decides what that is worth: a verifier service that checked a hardware quote, a CI
system that built the image, or you. `delulu sandbox attest --key SEED --attester NAME --guarantee TEXT..
-- COMMAND..` is a **software** attester — it signs whatever its key's holder tells it to, then becomes
COMMAND with the channel on its standard input and output — useful for binding a run to an image your
pipeline signed, and as a test double; it is not evidence about hardware. With the recipe above:

```sh
#!/bin/sh
# /usr/local/bin/delulu-gvisor-attested — run with:
#   delulu run app.delulu --grant … --sandbox --require-attestation "$(cat ~/.delulu/keys/ci.pub)" \
#     --sandbox-backend external:/usr/local/bin/delulu-gvisor-attested
exec delulu sandbox attest --key /etc/delulu/ci.seed --attester "ci image delulu-guest:1.0.0" \
  --guarantee "gVisor runsc" --guarantee "no network" -- /usr/local/bin/delulu-gvisor
```

**NVIDIA OpenShell — designed, not yet built or tested (PS-E-05).** OpenShell (NVIDIA's open agent
runtime, 2026-09) is planned as a second L3 recipe and as a target DeluluLang writes policy for:
`delulu sandbox policy <file> --format openshell` will emit the OpenShell policy a program's authority
and grants imply — never wider — for running `delulu run` inside an OpenShell sandbox, and the guest
will run inside an OpenShell sandbox with no network rule through `external:`, still at level 3 and
still reported as unmeasured unless attested. Until PS-E-05's workflow has run green, nothing here is
a recipe to follow. The design: `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` §4.5.

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
- **A concurrent writer into a granted directory — closed as a race, kept as advice.** Filesystem
  containment used to be a check-then-open, so a second writer could swap a checked file for a symlink
  in between (`CONTAIN-TOCTOU-1`). Since V2 P5c (2026-09-27, FS-RACE-1) every file effect opens the
  approved path one component at a time and follows no link, so a swapped-in link is refused at the
  open. Still **grant scopes that point at directories only the program's own user can write** —
  never a shared or world-writable one: a writer you do not control can still change what a file
  holds, and a hardlink (below) is a real second name.
- **A hardlink inside a granted directory.** A hardlink is a genuine second name for one file, so
  containment reports it as inside the grant, because it *is*. Creating one requires access to the
  target already, and git cannot carry one.
- **Resource exhaustion.** A program may consume its own CPU and memory. Authority is the
  containment, not a quota.
- **Foreign code.** `foreign.c` / Python bound through a grant is outside the effect guarantee: the
  language bounds *reachability*, not behaviour.
- **The hardware adapter.** `--adapter-cmd` runs an operator-supplied subprocess. The envelope is
  enforced host-side before dispatch, but the driver itself is not sandboxed and is not signature-checked.
- **The microVM, beyond Linux x86_64 + KVM.** `--isolation microvm` exists since V2 PS-C (see the
  sandbox section above) on Linux x86_64 with KVM and an image you built; there is no Windows or macOS
  microVM, no snapshot or warm pool, and no distributed image (D-NE-27 is the owner's). Everywhere else
  it refuses with `DL1408` — the correct failure: you never silently get a weaker boundary than you
  asked for. For genuinely untrusted code on one host, Tier 2 above is still the outer boundary.
- **An external launcher's wall.** At L3 (`--sandbox-backend external:CMD`) the boundary around the
  guest is whatever your launcher builds; DeluluLang measures none of it, and the report says level 3
  with no host guarantee. Authority is unaffected — the host still decides every effect — but
  containment is only as good as the launcher. `--require-attestation` (PS-D-02) adds an attester's
  signed word about it, checked for key and freshness; the word is only as good as the attester's key
  custody and what it checked.

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
