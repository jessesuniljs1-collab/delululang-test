# Sandbox implementation plan — phases PS-0 to PS-D, integrated into the main roadmap

**Written:** 2026-09-17. **Status:** proposed; **nothing here is built**. This document is a
supporting plan for `IMPLEMENTATION_ROADMAP.md`, which carries the merged order; it exists so the
sandbox work has one place for its tasks, blast radius, evidence obligations and host facts.
Design: `SANDBOX_ARCHITECTURE.md`. Threat model: `SANDBOX_THREAT_MODEL.md`. Tests:
`SANDBOX_TEST_PLAN.md`. Decisions: `DECISION_LOG.md` D-NE-20…D-NE-33.

**Effort** is in focused sessions (S), an estimate. **Blast radius** names Survey ids to run
`impact` on before the task starts. **Owner** marks a decision the owner must take first.

---

## 0. Host facts this plan rests on (measured, 2026-09-17)

From the dispatch-only CI probe `host-capability-probe` (run `35218542442`, read and transcribed;
the workflow builds nothing and every step is `continue-on-error`):

| Runner | Facts |
|---|---|
| `ubuntu-latest` (Ubuntu 24.04.5, kernel **6.17.0-1022-azure**, 4 vCPU, 16 GB) | **`/dev/kvm` exists** (`crw-rw---- root kvm`; whether the runner user may open it is not yet verified — the standard "coaxing" is a group or mode change, to be probed in PS-0-08). **Unprivileged user namespaces are blocked**: `apparmor_restrict_unprivileged_userns = 1` and `unshare -Ur true` fails with `write failed /proc/self/uid_map: Operation not permitted` — so a bubblewrap-style jail does not work here without an AppArmor profile or a sysctl. **Landlock ABI 7.** seccomp available. cgroup v2 with `cpu io memory pids` controllers, and `systemd-run --user --scope -p MemoryMax=64M` **works** (exit 0) — user-level memory/pid limits are enforceable. `bwrap`, `firecracker`, `cloud-hypervisor`, `virtiofsd`, `runsc` absent; `docker` and `podman` present. |
| `macos-latest` (macOS 26.6.2, arm64, 3 CPU, 7 GB) | **`kern.hv_support: 0`** — no hypervisor support on the free runner, so no Virtualization.framework / libkrun there. `/usr/bin/sandbox-exec` present. My two Seatbelt probes were **inconclusive** (a deny-all profile aborted because the profile was too strict for the dynamic loader, not because Seatbelt failed; a `curl` to a closed port fails whether or not the network is denied) — PS-0-08 writes a proper differential probe. No `container` CLI. |
| `windows-latest` (Windows Server 2025, 10.0.26100, 4 CPU, 16 GB) | The runner itself is a Hyper-V VM; **`HypervisorPlatform: Enabled`**, `VirtualMachinePlatform: Enabled`, `Containers: Enabled`, WSL enabled, VBS running; the session is **admin**; `docker` present. So the Windows Hypervisor Platform is on — nested virtualization for a Hyperlight-class micro-VM is *plausible*, not verified. |

On the development machine (Windows 11, no KVM, admin not assumed): L1 is the only local tier; L2
is a CI-and-Linux-host tier until a Windows path exists.

Two consequences shape everything below: **L1 on Linux must not depend on user namespaces** (they
are off on Ubuntu 24.04 defaults and on CI); Landlock 7 + seccomp + rlimits + a user cgroup are the
provable baseline, with namespaces an upgrade where a probe finds them usable. And **L2's criterion 8
can run on CI** if `/dev/kvm` is openable and a VMM binary plus image are fetched in the job — an
experiment for PS-0-08, not an assumption.

## 1. Phase PS-0 — Truth, probes and the cheap hardenings (inside P1)

**Goal.** Make every sandbox-related statement true, make the host's capabilities visible to humans
and agents, and close the runtime holes the red team and the head chef verified — none of which
needs a sandbox to exist. Ships with P1; no owner decision needed except where marked.

| Id | Task | Finding | Blast radius | Verification | Effort |
|---|---|---|---|---|---|
| PS-0-01 | `REMAINING_WORK.md` rows for NE-17 (no network client), NE-19/20 (device names, trailing characters), NE-21 (worker read timeout), NE-22 (no resource bounds); correct `README.md`, `GETTING_STARTED.md` §6 and the Book where `http.get` is presented as working; state that `--isolation process` isolates foreign code only, in the `run` help and the `isolation:` label | NE-17…22 | docs | evidence gate; book gate | 0.5 S |
| PS-0-02 | `run --json` gains an additive `sandbox`/`isolation` object even at L0: `{backend:"inproc", level:0, requested, granted, host_guarantees:[], limits:null}` | NE-16b | `mod:crates/delulu/src/run_cmd.rs` | envelope sweep (P1-02) | 0.3 S |
| PS-0-03 | `DL1408` gains the promised repair: `requires_human: true`, `edits: []`, the fallback command in the message; conformance witnesses updated | NE-16c | `crates/delulu-diag/src/codes.rs`, `run_cmd.rs` | coverage gate; the P1-05 rule (a repair with no edits is human-only) | 0.2 S |
| PS-0-04 | `delulu sandbox probe [--json]`: per level, present/absent with the first missing prerequisite; every line an *attempt* (create a Landlock ruleset on a throwaway thread, open `/dev/kvm`, apply a job limit to a throwaway child, run a Seatbelt no-op), never a version string | — | new module in `crates/delulu/src`, `cli.rs` dispatch, completions, `--help` gate | probe results asserted on CI per OS against the facts above; a mutant probe that reports "present" without attempting must fail | 1.5 S |
| PS-0-05 | `delulu doctor` gains the `sandbox` section from `SANDBOX_ARCHITECTURE.md` §7, built on PS-0-04; no line that changes no decision | — | `doctor.rs` | `doctor_cli.rs` | 0.5 S |
| PS-0-06 | Primitive-table hardening: refuse Windows reserved device names in any component (with and without extensions), trailing dots and spaces, drive-relative spellings (`C:x`), verbatim/device prefixes (`\\?\`, `\\.\`), and record the **resolved** name in the trace and audit; on POSIX refuse embedded NUL and names that differ after NFC/NFD normalization from what was granted — the 2026-08-10 search key applied to the write path | NE-19, NE-20 | `mod:crates/delulu-runtime/src/prim.rs` (`impact` reaches the interpreter, the WASM host and every containment test) | characterization tests C-01/C-02/C-11 flip red then are rewritten as refusal witnesses; core-invariance snapshot unchanged for the corpus (no shipped program names a device) | 1 S |
| PS-0-07 | Apply IPC-1's fix to the foreign-worker channel: `set_read_timeout` on `WorkerConn`, a deadline per call, `ForeignErr::WorkerDied`-class result on timeout, the worker killed | NE-21 | `foreign_worker.rs` | C-05 flips; falsify by removing the timeout | 0.5 S |
| PS-0-08 | CI experiments (dispatch-only workflow extended): (a) can the runner user open `/dev/kvm` after the documented group/mode change, and can a fetched Firecracker or Cloud Hypervisor boot a stock guest kernel to `/init` inside the job; (b) a *differential* Seatbelt probe on macOS (allowed path readable, denied path refused, network refused versus allowed); (c) on Windows, can a non-elevated child be launched under a restricted token + Job Object with a memory limit (a tiny Rust probe built in the job) | — | `.github/workflows` | results read with `gh run view` and transcribed into `SANDBOX_RESEARCH.md` §1.6 | 1 S |
| PS-0-09 | Refuse or warn on special-use addresses in `--grant net=` (loopback, link-local, RFC1918, the metadata literals) unless spelled explicitly as such — **owner ruling on the spelling** (D-NE-28) | NE-18 | `broker.rs` grant parser | C-04 flips | 0.3 S |

**Acceptance.** Every claim about isolation in the shipped documents is true; `sandbox probe --json`
and `doctor` report the host; the four runtime hardenings have witnesses; CI knows whether L2 can
run there.

## 2. Phase PS-A — L1: the process jail with the effect channel (the minimum useful secure sandbox)

**Goal.** `delulu run app.delulu --sandbox` runs the whole program as a guest that holds no OS
authority, on all three operating systems, with the host performing every effect under today's
checks, and with identity separation where the OS lets an unprivileged launcher have it.

| Id | Task | Blast radius | Verification | Effort |
|---|---|---|---|---|
| PS-A-01 | **The `EffectSink` seam in the runtime**: the primitive table calls a sink; `LocalSink` is today's code path (byte-identical behaviour — the core-invariance snapshot must not move); `ChannelSink` serializes a request and blocks on the reply. Capabilities in guest mode are opaque handles (`CapVal { kind, handle }`), `narrow` returns a new handle from the host | `mod:crates/delulu-runtime/src/prim.rs`, `interp.rs`, `value.rs` (`impact` = the whole runtime and every CLI test) | snapshot identical; `delulu-fuzz` in local mode unchanged; a guest-mode fuzz run asserts trace ⊆ row *and* one audit record per performed effect (T2) | 4 S |
| PS-A-02 | **The channel protocol** `delulu-sandbox-channel/1`: length-prefixed canonical CBOR frames over the existing `broker_ipc` framing with a hard per-frame bound; request kinds mirroring the primitive table; the host dispatcher; a `cargo-fuzz` target from day one (test plan §3); read deadlines both ways | new module in `crates/delulu/src`; `broker_ipc.rs` | fuzz target green; property tests; the IPC-1 test | 2 S |
| PS-A-03 | **The guest runtime mode**: hidden `__guest` subcommand (the `__foreign-worker` precedent) — connect the channel, receive program bytes + hash + grant + policy + seed/clock, check the hash, run the interpreter with `ChannelSink`, report the exit over the channel; **no filesystem, no environment, no argv beyond the channel handle** | `main.rs`, `cli.rs`, `run_cmd.rs` | a guest with a stubbed host refuses to start without the hello; T1's memory/env/argv scan | 1.5 S |
| PS-A-04 | **Launchers**: Linux — Landlock ruleset (deny all fs except the guest binary's own read-only mapping if the loader needs it; no `NET_*` rights), seccomp allowlist (`seccompiler`), `RLIMIT_*`, `PR_SET_PDEATHSIG`, optional user cgroup via `systemd-run --user` where the probe says it works, **user/mount/net namespaces only where the probe says they are usable**; Windows — restricted token (deny-only SIDs, no privileges, low/untrusted integrity), Job Object (kill-on-close, active-process = 1, memory, process-time, UI restrictions, no breakaway), AppContainer where creatable; macOS — a generated Seatbelt profile (`deny default`, allow the loader's needs, `deny network*`, `deny process-fork`) applied with `sandbox_init` from the guest side or `sandbox-exec` from the host side | new `sandbox` module(s) in the CLI; `Cargo.toml` (rulings D-NE-26) | boundary tests §5 per OS; **mutant launchers** §6; the P21 vectors from inside the guest (T14) where identity separation applies | 4 S |
| PS-A-05 | **Environment and secrets hygiene** for every child the host spawns (guest, foreign worker, adapter): empty environment plus an allowlist; secrets never on argv for guests; the guest's stdin closed | `foreign_worker.rs`, `adapter.rs`, the launcher | C-09 flips | 0.5 S |
| PS-A-06 | **Policy derivation**: `derive(authority, grant, profile) → SandboxPolicy`, pure, JSON-stable; profiles `dev`, `contained`, `hostile-agent`; `[sandbox]` in `delulu.toml` (`profile`, `limits`, `require_level`) bounded by `[authority]` | `manifest.rs`, new policy module | policy tests §4; a snapshot for the corpus | 1 S |
| PS-A-07 | **CLI and machine surface**: `--sandbox`, `--sandbox-profile`, `--limits`; `run --json` `sandbox` object with `denied[]`, `remaining`, `host_guarantees`, `state`, `guest_death`; `delulu sandbox status/kill`; `explain E-SANDBOX`; new DL codes with both witnesses; the `--isolation process` label strengthened and announced (`CHANGELOG.md`, `STABILITY.md` §2's allowance for stronger profiles) | `cli.rs`, `run_cmd.rs`, `codes.rs`, completions/help gate | envelope sweep; cli sweep; contract tests | 1.5 S |
| PS-A-08 | **Audit lifecycle**: `sandbox-launch` (level, guarantees, policy hash, image hash), `sandbox-limit`, `sandbox-kill`, `sandbox-death`, `channel-violation`; the guest id on every brokered effect | `brokerd.rs`, `audit.rs` | audit tests; `audit verify` unchanged | 0.5 S |
| PS-A-09 | **Documentation**: `DEPLOYMENT.md` gains "Tier 2, mechanical" per OS with its honest limits; `for-agents.md` and the skill gain the profile advice; the Book Ch. 15 gains the guest model; `MATHEMATICS.md` §12 gains the claims with their categories from `SANDBOX_THREAT_MODEL.md` §3 | docs | evidence gate; book gate | 1 S |

**Acceptance.** `delulu run --sandbox` works on all three CI runners for the guide corpus with
byte-identical program output versus L0; every §5 boundary family has witnesses on each OS and every
mutant fails its tests; the P21 vectors are refused from inside the guest on Linux; the measurement
record exists. **Owner:** D-NE-24 (labels and profiles), D-NE-26 (crates), D-NE-28 (addresses),
D-NE-31 (limits), D-NE-25 if `Secret.map` is encountered (it is refused under `hostile-agent` by
default).

**Effort: 12–17 S** — the largest phase in the roadmap. It is large because it is the mechanism that
makes every other "untrusted code" sentence in this repository true on the developer's own machine.

## 3. Phase PS-B — Limits, egress, and the first network client

| Id | Task | Verification | Effort |
|---|---|---|---|
| PS-B-01 | **Resource limits for the main program on every engine**: a step/allocation budget in the interpreter (the `fuel` idea, host-configurable; never "unlimited", the plugin precedent), wall-clock in the host, memory/pid/CPU through the launcher's OS controls; the attribution rule from `limits.rs` re-used so a limit kill never suggests widening | resource family §5.5; mutant without the budget | 2 S |
| PS-B-02 | **The egress proxy = the first network client** (there is none today, NE-17): host-side, performs `http.get` for guests *and* for L0 (one implementation), allowlist by name, **resolve once and pin the address**, refuse special-use ranges unless granted by their own spelling, SNI/Host must match the allowlisted name, redirects re-checked, response size bounded, no resolver access for guests. **Owner ruling (D-NE-28)** on the HTTP/TLS dependency (a TLS stack is the largest dependency decision this project has faced; `cargo deny` must stay green) | network family §5.3; C-03/C-04 flip | 3 S |
| PS-B-03 | **Identity separation where feasible**: Windows AppContainer profile per run (no network capability); Linux uid mapping via user namespaces where the probe allows, else documented operator recipe; macOS documented recipe (no unprivileged second identity) — reported in `host_guarantees` as `identity-separation` present/absent | T14 | 1.5 S |
| PS-B-04 | **Channel batching for epoch-class effects** after PS-A's measurement says it pays | measurement record | 1 S |

**Effort: 6–8 S.** **Owner:** D-NE-28.

## 4. Phase PS-C — L2: the microVM on Linux/KVM

| Id | Task | Verification | Effort |
|---|---|---|---|
| PS-C-01 | **Ruling** (build order): the guest runs the interpreter, not the WASM engine (a deviation from Stage 5 §6; D-NE-23); no virtio-fs, no NIC, vsock only; Firecracker first (jailer, most-restrictive seccomp), Cloud Hypervisor as the second launcher | reviewed | 0.5 S |
| PS-C-02 | **Guest image build** (Linux only): a pinned kernel from Firecracker's guest configs (6.1 series) with virtio-vsock and nothing else, an initramfs with a static, Python-less `delulu` as `/init` in `__guest` mode; hashes into a manifest the CLI carries; the build script named in a code comment for the Survey; reproducibility checked twice | hashes match across two builds | 2 S |
| PS-C-03 | **The `microvm` launcher**: drive the VMM over its Unix-socket API from the host (hand-written HTTP/1.1 client, no new crate), jailer per VM with a unique uid, vsock channel, wall watchdog, VMM-death handling, orphan sweep, jail cleanup, verified-dead semantics | lifecycle §5.6 | 3 S |
| PS-C-04 | **Criterion 8, at last**: the Stage 5 test un-gated on a KVM runner (PS-0-08's experiment decides which runner), re-stated for the no-NIC guest (T9) | CI green on the KVM job; `cfg(delulu_kvm)` removed or flipped | 1 S |
| PS-C-05 | **Distribution**: the image as a separate, checksummed, attested artifact (P5); `doctor`/`probe` report its presence and integrity | P5 gates | 0.5 S |
| PS-C-06 | **Red team L2**: the §5.8 scenarios; a published record including what was not attempted | record | 1 S |

**Effort: 8–10 S.** **Owner:** D-NE-23, D-NE-27 (kernel/image distribution is a licensing act —
the kernel is GPL-2.0; distributing a built kernel requires offering its source, which the
allowlist in `deny.toml` does not cover because it is not a crate — a decision, not a blocker).
**Needs:** a Linux/KVM host (CI has one, subject to PS-0-08) and Linux to build the image.

## 5. Phase PS-D — L3 external launchers and the L4 seam

| Id | Task | Effort |
|---|---|---|
| PS-D-01 | `--sandbox-backend external:<cmd>`: the operator's launcher runs `delulu __guest` in their environment (Docker + gVisor, Kata, a cloud sandbox, ssh) and exposes the channel over stdio; DeluluLang labels the level `external` and the guarantees `unknown`; a reference recipe for Docker with `--runtime=runsc` and `--network none` is documented, not shipped | 2 S |
| PS-D-02 | The attestation seam: a `guarantees` field the launcher may populate from an attestation document (Nitro PCRs, SEV-SNP/TDX reports), and a policy hook "refuse to delegate a lease unless attested" — **designed, tested with a fake attester, not integrated with real hardware** | 1.5 S |

**Effort: 3–4 S.** Owner-gated on hardware for anything beyond the seam.

## 6. What must not be implemented yet (restated from the architecture, with the trigger that re-opens each)

| Item | Why not now | Re-opens when |
|---|---|---|
| Snapshots / warm pools | entropy-duplication class; boot is already ≤125 ms (vendor) | PS-C measured and VMGenID handling tested |
| Windows L2 | Hyperlight is pre-1.0 and WASM-bound; WHP is enabled on CI but unverified for nested use | Hyperlight stabilizes *and* the WASM backend covers the language, or a native guest build exists |
| macOS L2 | no hypervisor support on the free runner; the project has no Mac; libkrun needs signing entitlements | a Mac with `hv_support = 1` in the loop |
| A DeluluLang-owned container runtime / gVisor wrapper | L3 covers it without new TCB | never, unless L3 proves insufficient |
| Guest-side foreign libraries | needs operator-supplied libraries with hashes inside images | after PS-C ships and a use case names the library |
| Any silent fallback | a rule, not a deferral | never |

## 7. Dependencies on and from the main roadmap

- **P1 → PS-0**: PS-0 is part of P1 (the envelope sweep, the repair rule and the REMAINING_WORK rows
  are the same commits).
- **PS-0 → PS-A**: the probe and the hardenings land before the launcher.
- **PS-A → P2 (claims)**: P2's code can proceed in parallel; the sentence "an untrusted plugin cannot
  overreach on your own machine" is published only after PS-A's witnesses are green.
- **PS-A → P4-01 (skill)**: the skill tells agents which profile to use and what the envelope's
  `sandbox` object means.
- **PS-A → PS-B → PS-C**: the channel is shared; L2 is L1 behind a hypervisor.
- **PS-C → P5**: the image artifact and the probe in the installer's post-install report.
- **PS-A → P8**: the control program runs in a guest; the adapter and the dead-man watchdog stay
  host-side (invariant 52 unchanged).
- **P7 → PS-A**: the `cargo-fuzz` scaffolding (RW 5.4) is done once and the channel target joins it.
