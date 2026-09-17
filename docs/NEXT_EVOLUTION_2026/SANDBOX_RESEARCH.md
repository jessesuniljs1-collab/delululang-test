# Sandbox and VM research — what the field runs untrusted code in, and what it costs

**Written:** 2026-09-17, by Claude Fable 5.1 (head chef), for the owner's sandbox commission
(`docs/design/DeluluLang_Sandbox_VM_Integrated_Next_Evolution_Prompt.md`).
**Status:** research input to [`SANDBOX_ARCHITECTURE.md`](SANDBOX_ARCHITECTURE.md),
[`SANDBOX_THREAT_MODEL.md`](SANDBOX_THREAT_MODEL.md), [`SANDBOX_TEST_PLAN.md`](SANDBOX_TEST_PLAN.md)
and [`SANDBOX_IMPLEMENTATION_PLAN.md`](SANDBOX_IMPLEMENTATION_PLAN.md); the decision they carry is
integrated into [`MASTER_PLAN.md`](MASTER_PLAN.md) §12. Nothing here is a product claim.
**Owner rule:** sources are named here and on no product surface.

Every source was fetched on 2026-09-17. Vendor numbers are marked as vendor numbers. Where a page
said less than asked, that is recorded as *not stated*. Two pages returned 403 (MITRE's sandbox
page, a BeyondTrust write-up) and one 404 (Azure custom sessions); what is said about them comes
from secondary sources found by search and is labelled so.

---

## 0. The summary in one paragraph

Between 2025 and 2026 every platform that runs agent-written code converged on the same shape:
**a per-session microVM or an application kernel, an allow-only egress proxy that also owns DNS,
an immutable base image, credentials injected from outside the boundary, hard resource limits,
and a disposable lifecycle** — Vercel, E2B, AWS AgentCore and Fly on Firecracker; Modal and
Google's GKE Agent Sandbox on gVisor; Azure dynamic sessions and Foundry on Hyper-V; Docker
Sandboxes on a purpose-built VMM across all three desktop OSes; Daytona and Cloudflare on
containers backed by their own control planes. Local agent tools took the process route:
Claude Code's runtime uses bubblewrap + seccomp + a domain-allowlisting proxy on Linux, Seatbelt on
macOS, and (alpha) a dedicated local account plus a WFP egress fence on Windows; Codex uses
Seatbelt, seccomp and Landlock; Gemini CLI added gVisor. The evaluation world (UK AISI's Inspect,
ControlArena, SandboxEscapeBench) settled on **a VM around a container**, per sample, with
Cilium DNS-aware egress, and measured that frontier models escape *misconfigured* containers
routinely and well-configured runtimes rarely. Every one of these systems enforces the boundary
**outside** the thing being sandboxed, and every one of them has published at least one escape
or exfiltration lesson. DeluluLang already has the piece none of them has — a compiler-computed,
host-enforced authority — and lacks the piece all of them have: an OS or hardware boundary that
holds when the language's own enforcement is bypassed or when the code is foreign.

---

## 1. Systems studied, one paragraph each

### 1.1 Evaluation and government sandboxes

- **UK AISI Inspect + the sandboxing toolkit** (inspect.aisi.org.uk, aisi-sandboxing, inspect_k8s_sandbox).
  Inspect's `SandboxEnvironment` is an *interface* — `exec()`, `exec_remote()`, `read_file()`,
  `write_file()`, `connection()` — with providers as plugins: `local`, `docker`, and external
  `k8s`, `daytona`, `modal`, `ec2`, `proxmox`, `vagrant`. Tasks bind a sandbox at eval/task/sample
  level; every sample gets its own instance; the generated compose sets `network_mode: none`; the
  toolkit classifies isolation on three axes — **tooling, host, network** — and offers Docker
  Compose, a Kubernetes plugin (gVisor by default, **Cilium DNS-aware policies** restricting DNS to
  allowlisted domains to block DNS exfiltration) and a **Proxmox plugin for "strong VM isolation for
  high-risk evaluations"**. Stated future concern: agents that actively attack the sandbox and
  injection into Inspect's own Python. Lesson: *the provider is an interface; the network default
  is none; DNS is part of egress.*
- **ControlArena** (AISI + Redwood): settings with a benign *main task* and a hidden harmful *side
  task*; policies (honest/attacking); monitors producing suspicion scores; Docker sandboxes.
  Lesson for a test program: adversarial evaluation is *task pairs plus a monitor*, not a list of
  exploits.
- **SandboxEscapeBench** (Oxford + AISI, arXiv 2603.02277): 18 scenarios in three layers —
  orchestration (4), runtime (8: privileged mode, exposed Docker socket, capability mismanagement,
  runc CVEs), kernel (6: namespace escapes, cgroup abuse, Dirty Pipe, eBPF) — run as a container
  inside a VM per sample; success = reading `/flag.txt` on the host. Frontier models "consistently
  escape through obvious weaknesses" (difficulty 1–2), reach ~40% on difficulty 3, and 0% on 4–5;
  success scales log-linearly with tokens; an attempt costs about a dollar. Recommendation: treat
  plain Docker isolation as insufficient by default; a correctly configured, up-to-date runtime
  remains an adequate boundary for current models; stronger primitives (microVMs, gVisor) as
  capabilities grow.
- **MITRE Federal AI Sandbox** (page 403; MITRE fact sheet and press via search): an NVIDIA DGX
  H100 SuperPOD (248 GPUs, exaFLOP 8-bit) for federal model training, approved for CUI. It is
  *compute*, not execution isolation. **NayaOne's sandbox** (gov.uk assurance page): a
  disconnected data-and-model testing environment with bias/drift/hallucination tooling —
  governance and data isolation, not code isolation. **AI Verify** (Singapore): a governance
  testing toolkit; explicitly no execution sandboxing. These three are recorded so nobody cites
  them as isolation designs.

### 1.2 Cloud sandboxes for agents

- **Vercel Sandbox** — Firecracker microVMs on Vercel's build fleet; `Sandbox.create()`,
  `runCommand()`, git-seeded sources, port exposure; 5 min default / 45 min hobby / 24 h max;
  up to 8 (32 enterprise) vCPU at 2 GB per vCPU; runs as `ubuntu` with passwordless sudo
  (vendor numbers).
- **E2B** — Firecracker; a guest API server `envd` (processes, PTYs, files, watchers, port
  forwarding) upgradable in place; **templates are pre-booted memory+disk snapshots restored
  lazily with `userfaultfd`** over copy-on-write overlays; per-sandbox nftables egress with
  SNI/Host-inspecting domain allow/deny lists; per-sandbox URLs with tokens.
- **Modal** — gVisor; "less than a second" creation, 50,000+ concurrent (vendor); memory and
  filesystem snapshots; granular outbound control; SOC2/HIPAA.
- **AWS Bedrock AgentCore Code Interpreter** — Firecracker; three network modes (sandbox / public
  / VPC). **2026 finding (BeyondTrust/Phantom Labs, via search; the post itself was 403):** in
  "sandbox" mode DNS queries still resolved externally, giving a DNS command-and-control and
  exfiltration channel, and the microVM metadata service exposed credential paths; AWS first
  called the DNS behaviour intended, then remediated after disclosure. *Lesson: "no network" that
  still resolves names is a network; metadata endpoints are a credential store.*
- **Azure Container Apps dynamic sessions / Foundry Code Interpreter** — each session "isolated by
  a Hyper-V boundary"; sessions are prevented from outbound requests by default (`EgressEnabled`
  is opt-in); session pools with `readySessionInstances` (a warm pool) and `cooldownPeriodInSeconds`;
  code-interpreter sessions live 1 h with a 30 min idle timeout; the docs warn that enabling the
  managed identity inside a session lets "any code running in the session create tokens", and that
  in toolbox mode "user isolation isn't supported" (all users of a project share a container).
  *Lesson: the platform states what a session does not isolate, in the same voice as what it does.*
- **Google GKE Agent Sandbox / Gemini Enterprise code execution** (via search): gVisor per
  container; sub-second creation; state persistence up to 14 days; 100 MB file I/O per request;
  Gemini CLI added native `runsc`.
- **Daytona** — sandboxes with stateless and stateful interpreters, `executeCommand` with a
  10-second default timeout, background sessions, snapshots and volumes; the isolation technology
  is *not stated* in the pages read; Claude Code / Agent SDK / Managed Agents run inside them.
- **Cloudflare Sandbox SDK** (beta) — one container per sandbox backed by a Durable Object;
  `exec`, `readFile`/`writeFile`, tunnels over `cloudflared`; `sleepAfter` idle timeout; egress
  policy *not stated*.
- **Docker Sandboxes** (GA 2026) — a microVM per agent session with its own kernel **and its own
  Docker daemon**, on a **new purpose-built VMM using Apple Hypervisor.framework on macOS, Windows
  Hypervisor Platform on Windows and KVM on Linux** ("Firecracker is Linux/KVM only; agents run on
  laptops"); scoped file allowlists, domain allowlists and credentials injected from outside the
  boundary are fixed *before* the agent runs ("infrastructure-enforced, not LLM-decided");
  Sandbox Kits are YAML declaring tools, env, credentials, allowed domains, files and startup
  commands. No numbers published on the page read.

### 1.3 Isolation primitives

| Primitive | What it is | Boundary | Numbers (source) | Platforms |
|---|---|---|---|---|
| **Firecracker** | Rust VMM, one process per microVM (API thread, VMM thread, vCPU threads); virtio net/block/vsock/balloon, no virtio-fs; MMDS; rate limiters; the **jailer** (chroot, cgroups, namespaces, per-thread seccomp, unique uid per VM) is mandatory in production; "performs no network filtering" — host nft rules, block IMDS; disable SMT/KSM/swap for tenant separation; snapshot = memory file `MAP_PRIVATE` + device state, **restoring one snapshot into two VMs is insecure** (entropy, identifiers) unless VMGenID reseeds and the rest is accepted | KVM (VT-x/AMD-V) | boot ≤125 ms to `/sbin/init`, ≤5 MiB overhead per microVM, 150 microVMs/s per host (SPECIFICATION.md, vendor); snapshot restore ~28 ms (secondary) | Linux x86_64/aarch64 + KVM only; no nested KVM in guest; no GPU |
| **Cloud Hypervisor** | Rust VMM for general workloads; virtio-fs (via `virtiofsd`, `--memory shared=on`, kernel ≥5.10, DAX not available), vsock, pmem, VFIO passthrough; snapshot/restore and live migration experimental; TDX experimental; KVM and MSHV; x86_64/aarch64/riscv64 | KVM / MSHV | <200 ms boot (a blogger's measurement) | Linux hosts; MSHV is Hyper-V *on Linux* (Azure), not a Windows host |
| **libkrun** | a VMM as a **dynamic library** with a bundled kernel (`libkrunfw`); virtio-fs ("no protection against the guest accessing other directories"), virtio-vsock with **TSI** (socket impersonation, no NIC) or virtio-net + passt; GPU via venus; `libkrun-sev`/`-tdx` variants; used by crun/Podman | KVM on Linux, **Hypervisor.framework on macOS/arm64** | not stated | Linux, macOS |
| **Hyperlight / hyperlight-wasm** | Microsoft's Rust library for micro-VMs **without a guest kernel**: a memory slice + a vCPU; guests are `no_std` ELF or **WebAssembly**; host functions are the only capability; KVM, mshv and **Windows Hypervisor Platform** | hardware virtualization | 1–2 ms start (Microsoft blog) vs <0.03 ms Wasmtime vs >120 ms VM | Linux, **Windows**; pre-1.0; hyperlight-wasm "not considered production-grade by its developers" |
| **gVisor** | `runsc`: the Sentry (a Go application kernel, ~274 syscalls reimplemented, 53 host syscalls used) + the Gofer for files; platforms **systrap** (seccomp `SECCOMP_RET_TRAP`, default), KVM, ptrace (deprecated); explicitly no protection against hardware side channels; resource DoS left to cgroups; network policy left to the container layer | a second, memory-safe kernel between app and host | ~50 ms start; syscalls 2.2–2.8× slower, I/O 10–30% slower, ~50 MiB/pod (a 2026 decision-framework post's numbers) | Linux ≥5.6, x86_64/arm64 only |
| **Kata Containers** | OCI runtime = shim v2 + a Rust **agent in the guest over vsock (ttRPC)**; QEMU / Cloud Hypervisor / Firecracker / Dragonball; virtio-fs rootfs, DAX-mapped image; namespaces and cgroups *inside* the guest too; the base of Confidential Containers | hardware virtualization | 150–300 ms cold start, 130–200 MiB/pod, 8–12% steady CPU (same post); "3.4× more on-call pages than containerd" (one 90-day study, same post) | Linux + KVM; Kubernetes |
| **Nitro Enclaves** | an enclave carved from an EC2 instance: no storage, no network, no interactive access, **vsock only**; attestation documents signed by the Nitro hypervisor with PCR0 (image), PCR1 (kernel+boot), PCR2 (application), PCR3 (IAM role), PCR4 (instance id), PCR8 (signing certificate); KMS policies bind to PCRs; debug-mode enclaves attest all-zero PCRs | Nitro hypervisor + hardware | up to 4 enclaves per instance | EC2 only; Linux guests |
| **Confidential Containers** (via search) | Kata + SEV-SNP/TDX; RATS-style remote attestation; a key broker releases secrets only to measured guests; SNP hosts need kernel ≥6.16.1 and firmware ≥1.55; TDX migration not production-ready | CPU memory encryption + attestation | not measured here | Linux + specific CPUs |
| **Apple Containerization** (0.1.0, 2026) | Swift; one lightweight Linux VM per container on Virtualization.framework; `vminitd` over vsock; ext4 + virtio-blk + virtio-fs; TAP networking; **macOS 26 + Apple silicon**; also a cloud-hypervisor/KVM Linux backend | hardware virtualization | "sub-second" | macOS 26+ only |
| **Hyper-V isolation (Windows containers)** | each container in a utility VM with its own kernel, `--isolation=hyperv`; default on Windows 10/11 Pro/Enterprise | Hyper-V | not stated | Windows containers only (not Linux workloads) |
| **WSL2** (via search) | one utility VM, **one shared kernel** for every distro; distros are container-grade (own PID/mount/user namespaces, shared network namespace); Docker Desktop VM-escape techniques published; a kernel privilege escalation escapes all distros | Hyper-V around *all* of WSL, not around one distro | — | Windows |
| **bubblewrap / nsjail / firejail / minijail** (via search) | bwrap: ~50 KB, unprivileged user namespaces, the Flatpak base, no profiles; nsjail: Google's production hostile-code jail (protobuf config, kafel seccomp); firejail: setuid with a privilege-escalation CVE history; minijail: ChromeOS services | Linux namespaces + seccomp | — | Linux |
| **Landlock** | unprivileged, stackable LSM: filesystem rights (ABI 1+; `REFER` 2, `TRUNCATE` 3, `IOCTL_DEV` 5, `RESOLVE_UNIX` 9), **TCP bind/connect by port (ABI 4)**, UDP (ABI 10), signal and abstract-socket **scoping (ABI 6)**, audit (7), quiet rules (10); cannot restrict `chdir/stat/chmod/chown/mount`; 16 layers max | kernel LSM | negligible | Linux ≥5.13 (ABI 1) … ≥6.x for later ABIs |
| **seccomp-bpf** | syscall allowlists with argument conditions; actions Allow/Trap/Errno/KillProcess/KillThread/Log; `seccompiler` compiles JSON or Rust definitions (Firecracker's; now in the rust-vmm monorepo; Apache-2.0 OR BSD-3) | kernel filter | negligible | Linux x86_64/aarch64/riscv64 |
| **Seatbelt / `sandbox-exec`** (via search) | macOS profile-based process sandbox; **deprecated by Apple, still functional, still what macOS itself, Chrome, Codex and Claude Code's runtime use**; no published replacement for CLI sandboxing (App Sandbox needs a signed app bundle) | kernel MAC | negligible | macOS |
| **Windows restricted process** (Chromium design) | restricted token (deny-only SIDs, no privileges, untrusted integrity), **Job Object** (no child processes, UI limits, memory/CPU/process caps), alternate desktop, integrity levels; **AppContainer / LPAC** low-box tokens with capability SIDs; broker/target model; a non-admin process can apply all of it to a child; everything with a null security descriptor stays reachable | kernel object security | negligible | Windows |
| **Wasmtime** | linear memory + bounds checks, typed control flow, explicit imports only, 2 GB guard regions, stack guard pages, memory zeroing on drop, Spectre mitigations; the page recommends **defense in depth with OS sandboxes** and names embedder bugs and side channels as out of scope | language-level sandbox in-process | <0.03 ms start | everywhere |
| **Rust crates** | `landlock` (MIT/Apache, v0.4.7, active; best-effort vs required compatibility), `seccompiler` (Apache/BSD-3; archived into rust-vmm monorepo 2026-08), `birdcage` (cross-platform fs+net; **GPL-3.0, archived 2026-07** — unusable here on both counts), `sandlock-core` (Landlock + seccomp + user-notify) | — | — | — |

### 1.4 Local agent tools

- **Claude Code's sandbox runtime** (open-source preview): Linux = bubblewrap namespaces + bind
  mounts + **seccomp blocking unix-socket creation** + network namespace removed so all traffic
  must cross **HTTP and SOCKS5 proxies** that enforce `network.allowedDomains`/`deniedDomains` and
  pin resolved addresses (anti DNS-rebinding); macOS = Seatbelt profiles with localhost-only
  access to the proxy ports and live violation monitoring; **Windows (alpha) = a dedicated local
  user account `srt-sandbox`, a Windows Filtering Platform egress fence keyed on that SID, and
  per-session ACLs** — i.e. *identity separation as the mechanism*. Reads are allowed-by-default
  with `denyRead`; writes are deny-by-default with `allowWrite`; shell rc files, git hooks and IDE
  dirs are always denied. Stated non-protections: no traffic inspection; domain fronting may
  bypass filtering; writing into `$PATH` directories is escalation. *Lesson: the Windows answer the
  field found is a separate account plus a kernel egress fence, which is exactly DeluluLang's
  Tier 2 made mechanical.*
- **Codex** (learn.chatgpt.com/docs/security): modes read-only / workspace-write / full access;
  `.git` protected; internet access is a configured permission (also for cloud sandboxes);
  approvals gate sensitive operations; OS mechanisms are not detailed on the page (search results
  name Seatbelt, seccomp and Landlock).

### 1.5 Escape record worth carrying

runc CVE-2025-31133 / -52565 / -52881 (masked-path and `/dev/console` mount races, procfs write
redirects; disclosed 2025-11, still exploited in 2026); CVE-2024-21626 (Leaky Vessels);
CVE-2025-9074 (Docker Desktop); CVE-2025-23266 (NVIDIA toolkit, 9.0); CVE-2025-38617 (kernel
packet socket); CVE-2026-39861 (a coding agent's sandbox escaped through a symlink, 9.8 — the class
DeluluLang fixed in `SYMLINK-DANGLE-1`); the AgentCore DNS channel; WSL2 VM-escape techniques
against Docker Desktop. Firecracker and gVisor have had few boundary-crossing CVEs; the industry
framing ("a container escape gives you root on the host; a VM escape needs a hypervisor CVE that
commands a six-figure bounty") is a blogger's framing, quoted as such.

### 1.6 CI reality — measured, not searched

The search results said KVM on free Linux runners "just needs a little coaxing" (Determinate Systems,
actuated) and that macOS runners support nested virtualization; both were checked with a
dispatch-only workflow (`host-capability-probe`, run `35218542442` on 2026-09-17, every step
`continue-on-error`, read with `gh run view`):

| Runner | Measured |
|---|---|
| `ubuntu-latest` — Ubuntu 24.04.5, kernel 6.17.0-1022-azure, 4 vCPU, 16 GB | **`/dev/kvm` present** (`crw-rw---- root kvm`; openability by the runner user not yet tested); **unprivileged user namespaces blocked** (`apparmor_restrict_unprivileged_userns = 1`; `unshare -Ur true` → `write failed /proc/self/uid_map: Operation not permitted`); **Landlock ABI 7**; seccomp available; cgroup v2 with `cpu io memory pids`; `systemd-run --user --scope -p MemoryMax=64M` succeeds; no `bwrap`, `firecracker`, `cloud-hypervisor`, `virtiofsd` or `runsc`; `docker` and `podman` present |
| `macos-latest` — macOS 26.6.2, arm64, 3 CPU, 7 GB | **`kern.hv_support: 0`** — the search result about nested virtualization on regular macOS runners is **not** what the free arm64 runner reports; `sandbox-exec` present; the two Seatbelt probes were inconclusive (a too-strict profile aborted the loader; a closed port fails either way) and are redone properly in PS-0-08 |
| `windows-latest` — Windows Server 2025 (10.0.26100), 4 CPU, 16 GB | a Hyper-V guest with **`HypervisorPlatform: Enabled`**, `VirtualMachinePlatform: Enabled`, `Containers: Enabled`, WSL enabled, VBS running; the session is admin; `docker` present — WHP inside the runner makes a Hyperlight-class micro-VM plausible there, unverified |

One documentary claim was contradicted by this measurement and the measurement stands: the Sonnet 5
sous-chef's fact sheet (`agent-notes/HOST-CAPABILITY-FACTS-sonnet5.md` §D3, from runner-image
reports) says the Windows Hypervisor Platform is off by default on `windows-latest`; the probe
found it enabled on the current `windows-2025` image. The same sheet's other constraining facts —
Ubuntu's AppArmor restriction on unprivileged user namespaces, Landlock ABI 7 on this kernel, WFP
egress filters needing administrator rights, AppContainer needing none, `sandbox-exec` having no
supported replacement, and the jailer's root-then-drop step (CVE-2026-1386, verified against NVD
and the project advisory) — agree with the probe or with primary documentation.

Consequences: criterion 8 (the microVM egress-deny proof) can plausibly run on `ubuntu-latest`
once `/dev/kvm` openability and a VMM boot are shown (PS-0-08); **an L1 jail on Linux must not
depend on user namespaces** — Landlock 7, seccomp, rlimits and a user cgroup are the provable
baseline; macOS microVMs are out of reach on CI; and the Windows runner is the one place a
Windows micro-VM could be tried. The same honesty the existing `cfg(delulu_kvm)` gate encodes
applies: probe first, then claim.

---

## 2. What DeluluLang already has, measured against the field

| Field practice | DeluluLang today (verified 2026-09-17) |
|---|---|
| Authority declared outside the guest, enforced by infrastructure | **Stronger**: computed from code, enforced host-side in-process; scopes canonicalized; the broker holds one node per guest by design (invariant 25) |
| Per-run boundary (VM/app-kernel) | **Absent for the verified program** — `--isolation none` and `process` both run it in the `delulu` process; `process` moves only foreign code into workers; `microvm` refuses everywhere |
| Egress allow-only with DNS ownership | The runtime checks `Cap[Http]` hosts in-process and then **returns `Err(Refused)` unconditionally — there is no network client in the runtime at all** (`prim.rs`: "Stage-1 runtime bundles no network client; the authority path is what matters"; NE-17). No OS-level egress fence; DNS does not exist; `--grant net=169.254.169.254` is accepted (NE-18) |
| Immutable base image, program delivered in | `.dwx` carries a hash-bound authority section; no guest image exists |
| Credentials injected from outside | Broker-held secrets and leases — **the right design**; but a same-user process reads `broker.key` (category 7) |
| Resource limits | Contained plugins: fuel / memory / wall (Linux live engine; Windows refuses). **The main program has none on either engine** — only the 10,000-frame depth bound; the foreign worker has kill-on-close only |
| Lifecycle, cleanup, forensics | Audit chain with anchor; lease TTLs; no sandbox lifecycle events; no guest-death verification |
| Escape testing | Containment red-teams (symlink, hardlink, junction, TOCTOU) at the primitive table; no OS-boundary escape tests (nothing to escape yet) |
| Machine-readable sandbox state | `run --json` carries no isolation field; `doctor` has no isolation section; DL1408 carries no repair although the Stage 5 table promises one |

**Architectural limitations found in the existing design (not defects — the spec is honest):**
1. Stage 5 §6 mandates the **WASM engine inside the microVM** while the WASM backend compiles a
   measured fragment (6 of 19 entry programs) — a microVM tier that cannot run most programs.
2. Stage 5 §6 mounts granted `fs.*` scopes into the guest with **virtio-fs**, which puts a
   filesystem device, `virtiofsd` and the path-resolution class of bugs (C84, SYMLINK-DANGLE-1)
   *inside* the trust boundary; every cloud sandbox above went the other way (no shared filesystem
   by default).
3. The `process` profile's label says "isolation" for a mechanism that isolates none of the
   verified program; agents reading the label may over-trust it.
4. The seccomp filter for workers is a documented stub; Landlock is unused; Windows workers have
   no memory/CPU limits; `birdcage` is GPL and archived, so there is no ready cross-platform crate.
5. The foreign-worker channel never received the IPC-1 read deadline — `WorkerConn::call` reads
   unbounded, so a hostile or hung foreign library hangs the host forever (NE-21; the red team's
   finding, verified against `foreign_worker.rs`).
6. Filesystem containment decides on the requested spelling and the OS acts on another: on Windows
   a write to `NUL` inside a grant reports success and lands nowhere, a write to `CON` creates a
   real file named `CON` through the verbatim path, and `trail.txt.`/`space.txt ` create
   `trail.txt`/`space.txt` while the trace records the unstripped name (NE-19, NE-20; red team's
   findings, reproduced by the head chef). The same class as C86 and GUARD-SPELL-1.
7. `--isolation` is reported only on stderr and never under `--json` (NE-16b), and DL1408 carries
   no repair although the Stage 5 table promises one (NE-16c).

---

## 3. Lessons that shape the architecture

1. **Hold the authority outside the sandbox and give the guest nothing.** Nitro enclaves (vsock
   only), Hyperlight (host functions only), Docker Sandboxes (policy fixed before the agent runs)
   and DeluluLang's own WASM host all agree. The guest should perform *no* effect itself.
2. **Network "off" must mean no resolver and no metadata endpoint**, or it is not off (AgentCore).
3. **Never restore one snapshot twice** without reseeding, and treat warm pools as a later
   optimization (Firecracker's own guidance).
4. **Identity separation is the Windows answer** (Claude Code's alpha) and the category-7 answer
   in `ROOT_ISSUANCE_TRUST_BOUNDARY.md` §2(a); a restricted token plus a Job Object is what an
   unprivileged process can apply today.
5. **A provider is an interface with a small, stable surface** (Inspect's five methods; Kata's
   agent over vsock; E2B's `envd`). The launcher varies; the wire does not.
6. **Measure escape resistance with generated scenarios in layers**, and expect easy
   misconfigurations to be found by any capable agent (SandboxEscapeBench).
7. **Say what the boundary does not cover, in the same voice** (gVisor on side channels, Azure on
   session-internal secrets, Anthropic on domain fronting).
8. **Deprecated primitives are still the primitives** on macOS (Seatbelt) — use them, label them.
9. **A Rust-native, kernel-less, Windows-capable micro-VM exists (Hyperlight)** but is
   experimental and would inherit the WASM-fragment limitation; it is the right thing to *watch*,
   not to adopt.

## 4. Source list

AISI/evaluation: https://www.aisi.gov.uk/blog/the-inspect-sandboxing-toolkit-scalable-and-secure-ai-agent-evaluations ·
https://inspect.aisi.org.uk/sandboxing.html · https://github.com/UKGovernmentBEIS/inspect_ai ·
https://github.com/UKGovernmentBEIS/aisi-sandboxing · https://github.com/UKGovernmentBEIS/inspect_k8s_sandbox ·
https://k8s-sandbox.aisi.org.uk/security/network-access/ · https://github.com/UKGovernmentBEIS/control-arena ·
https://arxiv.org/html/2603.02277v1 · https://www.gov.uk/ai-assurance-techniques/nayaones-ai-sandbox ·
https://github.com/aiverify-foundation/aiverify · MITRE Federal AI Sandbox (fact sheet via search; page 403)
Cloud sandboxes: https://github.com/vercel/sandbox · https://github.com/e2b-dev/infra · https://modal.com/blog/sandbox-launch ·
https://modal.com/solutions/coding-agents · https://learn.microsoft.com/en-us/azure/container-apps/sessions-usage ·
https://learn.microsoft.com/en-us/azure/foundry/agents/how-to/tools/code-interpreter ·
https://www.daytona.io/docs/process-code-execution · https://www.daytona.io/docs/en/claude/ ·
https://github.com/cloudflare/sandbox-sdk · https://www.docker.com/blog/why-microvms-the-architecture-behind-docker-sandboxes/ ·
AgentCore (via search: BeyondTrust/Phantom Labs write-up 403; https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/code-interpreter-tool.html) ·
Google (via search: https://docs.cloud.google.com/kubernetes-engine/docs/how-to/agent-sandbox)
Primitives: https://github.com/firecracker-microvm/firecracker/blob/main/docs/design.md · …/docs/prod-host-setup.md ·
…/docs/snapshotting/snapshot-support.md · …/docs/vsock.md · …/docs/rootfs-and-kernel-setup.md ·
https://github.com/cloud-hypervisor/cloud-hypervisor · …/blob/main/docs/fs.md · https://github.com/containers/libkrun ·
https://github.com/hyperlight-dev/hyperlight · https://github.com/hyperlight-dev/hyperlight-wasm ·
https://opensource.microsoft.com/blog/2024/11/07/introducing-hyperlight-virtual-machine-based-security-for-functions-at-scale/ ·
https://gvisor.dev/docs/architecture_guide/security/ · https://gvisor.dev/docs/architecture_guide/platforms/ · https://github.com/google/gvisor ·
https://github.com/kata-containers/kata-containers/blob/main/docs/design/architecture/README.md ·
https://docs.aws.amazon.com/enclaves/latest/user/nitro-enclave.html · …/set-up-attestation.html ·
https://github.com/apple/containerization · https://learn.microsoft.com/en-us/virtualization/windowscontainers/manage-containers/hyperv-container ·
https://docs.kernel.org/userspace-api/landlock.html · https://docs.wasmtime.dev/security.html ·
https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md ·
https://github.com/rust-vmm/seccompiler · https://github.com/landlock-lsm/rust-landlock · https://github.com/phylum-dev/birdcage
Agent tools: https://github.com/anthropic-experimental/sandbox-runtime · https://learn.chatgpt.com/docs/security
Surveys and incidents (secondary, quoted as such): https://emirb.github.io/blog/microvm-2026/ ·
https://bex.co/blog/2026/07/09/kata-containers-vs-gvisor-runtimeclass-selection ·
https://www.cncf.io/blog/2025/11/28/runc-container-breakout-vulnerabilities-a-technical-overview/ ·
https://determinate.systems/blog/kvm-on-github-actions/ · WSL2 shared-kernel discussion (via search)
