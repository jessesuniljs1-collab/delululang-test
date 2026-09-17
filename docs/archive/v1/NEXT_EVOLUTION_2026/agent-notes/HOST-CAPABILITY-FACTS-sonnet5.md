# Host-capability facts for sandbox/VM isolation design (sous-chef: Sonnet 5)

> *Head chef's annotation (2026-09-17): five link texts below that named an upstream project's
> documentation file as a bare `docs/…` path were prefixed with that project's repository name
> (`firecracker/docs/…`, `cloud-hypervisor/docs/…`, `chromium/src/docs/…`) so the Survey does not
> read them as paths in this tree. The link targets and every other word are the sous-chef's,
> unchanged.*

Scope: host-OS and CI-runner facts bounding what an "L1 process jail" (child with no OS
authority, talking to the parent over a private pipe) and an "L2 microVM" layer can do without
admin/root, on Linux/Windows/macOS, plus what GitHub-hosted CI offers. "Documented" = an
official spec/doc says so. "Reported" = a blog/forum/issue says so, not authoritative.
"Measured" names who measured it. Dates below are 2026-09-17 unless noted.

Repo context read first: `crates/delulu/src/foreign_worker.rs` (lines 1–60) and
`crates/delulu/Cargo.toml`. Today's foreign-worker isolation uses, per OS: Windows —
`windows-sys` 0.61 with `Win32_System_JobObjects` + `Win32_Security_Authorization` (Job Object
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, owner-only-DACL named pipes); Linux — `libc` 0.2
`prctl(PR_SET_PDEATHSIG, SIGKILL)` only, with a code comment that a "full seccomp profile is a
documented post-chunk stub." No Landlock, user-namespace or cgroup code exists yet on either
platform, and no macOS isolation code exists at all.

---

## A. Linux — unprivileged process jail

### A1. Unprivileged user namespaces

- Ubuntu 23.10 added `kernel.apparmor_restrict_unprivileged_userns`: an **unconfined** process
  may only `unshare(CLONE_NEWUSER)`/`clone(CLONE_NEWUSER)` if an AppArmor profile confining it
  carries the `userns,` rule (or `default_allow`). Documented:
  [Ubuntu blog](https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces),
  [Ubuntu Discourse spec](https://discourse.ubuntu.com/t/spec-unprivileged-user-namespace-restrictions-via-apparmor-in-ubuntu-23-10/37626).
- **Enabled by default from Ubuntu 24.04** ([LP #2046477](https://bugs.launchpad.net/ubuntu/+source/apparmor/+bug/2046477)).
  `bwrap` ships no AppArmor profile by default on Ubuntu Desktop, so bare `bwrap` from an
  unconfined process is denied — real breakage confirmed in
  [VS Code #316046](https://github.com/microsoft/vscode/issues/316046).
- Workarounds (same Ubuntu blog): install the shipped `bwrap-userns-restrict` profile from
  `/usr/share/apparmor/extra-profiles/` into `/etc/apparmor.d/`; write a custom profile granting
  `userns,`; or `sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0` (root-only, so
  useless for our own process to self-unlock).
- This AppArmor gate is **Ubuntu-only** (a kernel patch). Elsewhere, only plain
  unprivileged-userns availability matters: **Debian** —
  `kernel.unprivileged_userns_clone` (Debian-only sysctl) defaults to **1** since Bullseye
  ([Debian manpages, user_namespaces(7)](https://manpages.debian.org/bookworm/manpages/user_namespaces.7.en.html)).
  **Fedora/RHEL-family** — no such sysctl; the analogous `user.max_user_namespaces` is non-zero
  by default (Toolbox/Silverblue depend on it) — reported:
  [Fedora Discussion](https://discussion.fedoraproject.org/t/confining-user-namespaces-with-selinux/142995).
  **Arch** — `kernel.unprivileged_userns_clone` ships enabled — reported:
  [Arch Forums](https://bbs.archlinux.org/viewtopic.php?id=247016).

### A2. Landlock

- ABI history (each a strict superset of the last), documented in
  [kernel.org Landlock docs](https://docs.kernel.org/userspace-api/landlock.html) and the Rust
  binding's doc comments,
  [`landlock` crate `enum ABI`](https://landlock.io/rust-landlock/landlock/enum.ABI.html):
  **ABI 1** (Linux 5.13) filesystem rights (execute/read/write/dir-remove/etc.);
  **ABI 2** (5.19) `FS_REFER` (cross-dir link/rename); **ABI 3** (6.2) `FS_TRUNCATE`;
  **ABI 4** (6.7) `NET_BIND_TCP`/`NET_CONNECT_TCP` — first network coverage; **ABI 5** (6.10)
  `FS_IOCTL_DEV`; **ABI 6** (6.12) scopes — `SCOPE_ABSTRACT_UNIX_SOCKET`, `SCOPE_SIGNAL`;
  **ABI 7** (6.15) audit/log-control flags; **ABI 8** (7.0) `RESTRICT_SELF_TSYNC`
  (multithread-wide enforcement in one call). ABI 9–11 exist (UDP bind/connect-send, a
  Unix-socket path-resolution right, `RESTRICT_SELF_NO_NEW_PRIVS`) but exact kernel-version
  numbers for each weren't pinned down in this pass — see "Not established."
- **`ubuntu-latest`'s kernel, Sept 2026**: GitHub's Ubuntu 24.04 image (`20260907.300`, `24.04.5
  LTS`) reports **`6.17.0-1022-azure`** — measured/documented by GitHub:
  [runner-images Ubuntu 24.04 readme](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md).
  6.17 sits between ABI 7 (6.15) and ABI 8 (7.0), so **`ubuntu-latest` offers Landlock ABI 7**,
  not ABI 8's thread-sync self-restrict.
- What Landlock cannot restrict (same kernel.org page): `chdir`, `stat`, `flock`, `chmod`,
  `chown`, `setxattr`, `utime`, `fcntl`, `access`; filesystem-topology changes (`mount`,
  `pivot_root`); "special" filesystems (pipefs/sockfs/`nsfs`); network coverage is TCP/UDP
  bind/connect only — no payload filtering, no raw sockets, no netlink.

### A3. seccomp-bpf and Landlock from Rust

- `seccompiler`: latest **0.5.0**, licence **`Apache-2.0 OR BSD-3-Clause`** (allowlist-clean),
  `repository` → `github.com/rust-vmm/seccompiler`. Measured directly from the crates.io
  registry API (`GET /api/v1/crates/seccompiler`, queried 2026-09-17). That repo is now
  **archived** (GitHub API `archived: true`) and the code lives on in the `rust-vmm/rust-vmm`
  monorepo, confirmed by a `seccompiler/` directory there —
  [migration tracked in `rust-vmm/community` #105](https://github.com/rust-vmm/community/issues/105).
  Crate publishing under the original name is unaffected.
- `landlock` (the `rust-landlock` crate): latest **0.4.7**, licence **`MIT OR Apache-2.0`**,
  homepage `landlock.io`, repo `github.com/landlock-lsm/rust-landlock` — same crates.io API
  method. Compatibility is explicitly "best effort": `Ruleset` "determine[s] compatibility with
  the intersection of the currently running kernel's features and those required by the
  caller" — it silently degrades rather than failing outright. Documented:
  [`landlock` crate docs](https://landlock.io/rust-landlock/landlock/).

### A4. bubblewrap as an optional external binary

- Requires unprivileged user namespaces; the historical setuid-root mode was **removed in
  v0.12.0** (current latest tag, published 2026-08-26 — GitHub Releases API). Own README: "this
  has been removed" ([containers/bubblewrap README](https://github.com/containers/bubblewrap/blob/main/README.md)).
  It uses `PR_SET_NO_NEW_PRIVS`, keeps a minimal capability set, always maps back to the invoking
  uid — same README. It is a thin driver of exactly the `CLONE_NEWUSER` primitive A1 gates: it
  adds no capability beyond A1 and inherits every AppArmor restriction there verbatim. No setuid
  bit, no root — only a kernel/AppArmor posture that permits `unshare(CLONE_NEWUSER)`.
- **Licence flag for the owner**: bubblewrap's `LICENSE` file (read directly) is **LGPL-2.1**,
  not on the project's allowlist (GitHub's detector shows `NOASSERTION`, but the file text is
  unambiguous LGPL-2.1). Since the design drives it as a separate `exec`'d binary — the same
  relationship this project already has to Firecracker/Cloud Hypervisor — the "new crate needs a
  ruling" gate may not literally apply, but the LGPL fact itself is not cleared here; flagging
  for the owner's licensing-intent process rather than deciding it.

### A5. cgroups v2 from an unprivileged process

- Single-writer rule: each cgroup subtree has exactly one manager; an unprivileged process
  cannot write control files outside a subtree handed to it. **Delegation** (`Delegate=yes` on a
  systemd unit) is what hands over a subtree, chowning `cgroup.procs` (and, if named,
  `cgroup.subtree_control`) to that unit's user. Documented:
  [systemd.io, "Control Group APIs and Delegation"](https://systemd.io/CGROUP_DELEGATION/).
- In practice: `systemd-run --user --scope -p MemoryMax=64M -p CPUQuota=50% -- <cmd>` **works
  without root** on a modern systemd + cgroup-v2-only host, because `user@<uid>.service` (started
  by logind at login) is delegated by default — the scope lands under
  `user@<uid>.service/app.slice`, owned by the user. That is the practical unprivileged CPU/
  memory containment path on Linux — not a raw `mkdir /sys/fs/cgroup/...`, which is refused
  outside a delegated subtree, and there is **no self-delegation path** for an arbitrary process.

### A6. `setrlimit`/`prlimit` as the portable fallback

All from [man7.org `setrlimit(2)`](https://man7.org/linux/man-pages/man2/setrlimit.2.html) (2026-09-17):

- **`RLIMIT_AS`** — caps *virtual* address space; `brk`/`mmap`/`mremap` fail `ENOMEM`, auto
  stack-growth raises `SIGSEGV`. Bounds virtual reservation, not resident memory
  (`MAP_NORESERVE`/overcommit reserves cheaply) — a first line of defense, not a precise
  physical cap (cgroup v2 `memory.max`, A5, is precise).
- **`RLIMIT_CPU`** — CPU-seconds; soft → `SIGXCPU` (repeatable if caught), hard → `SIGKILL`.
  Quirk since Linux 2.6.12: a `SIGXCPU` handler silently bumps the soft limit by one second each
  time it fires — non-portable behavior.
- **`RLIMIT_NPROC`** — caps processes **and, "more precisely on Linux, threads,"** for the
  **real UID**; `fork()`/thread-creation fails `EAGAIN` at the limit. One counter shared by
  every process/thread under that uid — if host `delulu` is multithreaded under the same uid as
  a worker, host threads eat the child's budget and vice versa unless the worker uses a distinct
  uid; also unenforced for `CAP_SYS_ADMIN`/`CAP_SYS_RESOURCE`/uid 0.
- **`RLIMIT_FSIZE`** — max file-growth bytes; exceeding it sends `SIGXFSZ` (default: kill); a
  caught handler instead fails the syscall with `EFBIG`.
- None of the four is a confidentiality/integrity boundary against a determined in-process
  adversary — exhaustion caps only, but cheap, no-privilege backstops under Landlock/seccomp/
  userns regardless of kernel age or AppArmor posture.

---

## B. Windows — unprivileged process jail

### B1. Job Objects — limits a non-admin process can set on its own child

All from Microsoft Learn; none need admin — a normal process creates the job, sets limits, and
assigns a child it spawned itself:

- `JOB_OBJECT_LIMIT_PROCESS_MEMORY` / `_JOB_MEMORY` — per-process / whole-job commit caps; an
  over-limit commit fails. Documented:
  [`JOBOBJECT_EXTENDED_LIMIT_INFORMATION`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information).
- `JOB_OBJECT_LIMIT_ACTIVE_PROCESS` — caps simultaneous processes via `ActiveProcessLimit`; a
  `CreateProcess`/`AssignProcessToJobObject` past it **terminates the new process** and fails
  the association. Documented:
  [`JOBOBJECT_BASIC_LIMIT_INFORMATION`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information).
- `JOB_OBJECT_LIMIT_PROCESS_TIME` / `_JOB_TIME` — per-process / whole-job CPU-time ceilings,
  same structure.
- CPU **rate** control is separate: `JobObjectCpuRateControlInformation` with
  `JOB_OBJECT_CPU_RATE_CONTROL_ENABLE` as master switch, then `_WEIGHT_BASED`, `_HARD_CAP` (no
  job threads run again until the next scheduling interval once spent), or `_MIN_MAX_RATE`.
  Documented:
  [`JOBOBJECT_CPU_RATE_CONTROL_INFORMATION`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_cpu_rate_control_information).
- `JOB_OBJECT_LIMIT_BREAKAWAY_OK` / `_SILENT_BREAKAWAY_OK` — **opt-in only**: a child escapes
  the job only if the job sets one of these flags *and* the child used
  `CREATE_BREAKAWAY_FROM_JOB`. **Set neither to prevent breakaway** — the default already keeps
  every descendant in the job. Documented:
  [Job Objects — Win32 apps](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects).
- UI restrictions (`JobObjectBasicUIRestrictions`) can deny desktop/window-station
  creation/switching, clipboard access, `SystemParametersInfo` writes, global atoms/hooks — same
  page, independent of admin rights.

### B2. Restricted tokens + `CreateProcessAsUser` without admin

- `CreateRestrictedToken` builds a restricted copy of an existing token: `DISABLE_MAX_PRIVILEGE`
  strips every privilege but `SeChangeNotifyPrivilege`; SIDs can be marked **deny-only**
  (`SE_GROUP_USE_FOR_DENY_ONLY` — can no longer *grant* access, including the user's own SID,
  but can still deny); a "restricting SIDs" list adds a required-membership check to every
  access check. Documented:
  [`CreateRestrictedToken`](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-createrestrictedtoken),
  [Restricted Tokens](https://learn.microsoft.com/en-us/windows/win32/secauthz/restricted-tokens).
  Integrity level is set separately via `SetTokenInformation(..., TokenIntegrityLevel, ...)`.
- `CreateProcessAsUser` normally needs `SE_ASSIGNPRIMARYTOKEN_NAME` (an admin/`SYSTEM`-class
  privilege) — **except**: "If `hToken` is a restricted version of the caller's own primary
  token, [it] is not required." Documented:
  [`CreateProcessAsUserA`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessasusera).
  This is exactly how an ordinary process spawns a locked-down child.
- Chromium's Windows sandbox is the reference implementation: the unprivileged browser process
  derives a restricted token from **its own** token — untrusted integrity (`S-1-16-0x0`), all
  SIDs but the logon SID deny-only, no privileges — and calls `CreateProcessAsUser` with it,
  needing no special privilege. It layers a Job Object on top and, for network denial, a **Low
  Box token / AppContainer** withholding `INTERNET_CLIENT`. Documented (Chromium project doc):
  [`chromium/src/docs/design/sandbox.md`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/design/sandbox.md).

### B3. AppContainer

- A standard (non-admin) user **can** call `CreateAppContainerProfile` and launch a process in
  it — no admin documented as required (contrast WFP, B4). Documented:
  [AppContainer isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation).
- Default-blocked (same page): device isolation (camera/mic/GPS/cellular need a capability
  grant); file isolation (read-write needs explicit grants); network isolation (Internet/
  Intranet/server access are each separately grantable — no capabilities means no network);
  credential isolation (can't use ambient user credentials); process/window isolation.
- **Loopback**: official guidance on the related UWP mechanism confirms a contained app is
  blocked from `127.0.0.1` by default, needing explicit exemption
  (`CheckNetIsolation.exe LoopbackExempt` / `NetworkIsolationSetAppContainerConfig`).
  Documented: [Troubleshooting UWP App Connectivity](https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/troubleshooting-uwp-firewall).
  Community deep-dives on driving this for arbitrary Win32 processes (third-party):
  [blahcat](https://blahcat.github.io/2020-12-29-cheap-sandboxing-with-appcontainers/),
  [Yosifovich](https://scorpiosoftware.net/2019/01/15/fun-with-appcontainers/).
- **LPAC** tightens further: loses things a regular AppContainer gets free (no registry without
  `registryRead`, no COM without `lpacCom`). Reported (Mozilla adopting it):
  [Bugzilla 1783669](https://bugzilla.mozilla.org/show_bug.cgi?id=1783669).

### B4. WFP egress filtering needs admin — confirmed

- Fetched directly: BFE's default security descriptor grants `GENERIC_ALL` to Administrators and
  `GR/GW/GX` to Network Configuration Operators; a plain user gets only `FWPM_ACTRL_OPEN` and
  `FWPM_ACTRL_CLASSIFY` — **not** `FWPM_ACTRL_ADD`, which `FwpmFilterAdd0` requires. So **adding
  filters needs Administrator (or Network Configuration Operators) membership**. Documented:
  [Access control (WFP)](https://learn.microsoft.com/en-us/windows/win32/fwp/access-control).

### B5. Giving a child no network without admin

- **AppContainer with no `internetClient`/`internetClientServer` capability** is the no-admin
  route: per B3 each network class is opt-in, so zero capabilities means no Internet/Intranet/
  server access, and loopback isolation (B3) additionally blocks `127.0.0.1` unless exempted —
  a bare AppContainer with no network capabilities blocks essentially all network, loopback
  included, with no WFP (and thus no admin) involved. Same mechanism Chromium uses (B2). Sources
  as cited in B3.

---

## C. macOS — unprivileged process jail

### C1. `sandbox_init` / `sandbox-exec` (Seatbelt)

- Man page marks it **deprecated**, recommending App Sandbox instead; the warning still fires on
  **macOS 26.3** and the tool still runs — usable from an **unsigned, non-entitled CLI binary**,
  unlike App Sandbox (C2). Reported (man page text + direct testing, third-party):
  [manp.gs sandbox-exec(1)](https://manp.gs/mac/1/sandbox-exec),
  [HN discussion](https://news.ycombinator.com/item?id=44283454),
  [Apple Developer Forums crash report on current macOS](https://developer.apple.com/forums/thread/777509).
- Profile language (SBPL): a Scheme dialect — `(version 1)`, a default posture (conventionally
  `(deny default)`), then `(allow <op>* <filter>*)` rules, e.g.
  `(allow file-read* (subpath "/usr/lib"))`, `(deny network*)`. **Apple publishes no official
  documentation of this language** — every reference is reverse-engineered. Reported:
  [Chromium's `seatbelt_sandbox_design.md`](https://chromium.googlesource.com/chromium/src/+/HEAD/sandbox/mac/seatbelt_sandbox_design.md)
  (Chromium's own doc, not Apple's),
  [fG!, "Apple's Sandbox Guide v1.0" (2011)](https://reverse.put.as/wp-content/uploads/2011/09/Apple-Sandbox-Guide-v1.0.pdf).
- Known limitation, same sources: the filter/operation set and compiler behavior are
  undocumented and have shifted across OS versions without notice — no stable public API
  contract, and the underlying `mac_syscall(381, "Sandbox", …)` path carries no committed
  removal date despite deprecation.

### C2. Apple's guidance: is there a supported replacement?

- **No.** An issue filed directly against Apple's own `apple/containerization` repo states:
  "Despite the deprecation banner, there is no published replacement that covers the use case of
  server-side / CLI process sandboxing." Open, **no Apple maintainer response** as of this pass.
  Documented (primary, Apple's own repo):
  [`apple/containerization` #737](https://github.com/apple/containerization/issues/737).
- App Sandbox (what Apple does point developers at) requires code signing and an entitlements
  file — not a drop-in for sandboxing an already-built, unsigned/ad-hoc CLI binary.
- Apple's Containerization framework (`Containerization` Swift package + `container` CLI) runs
  Linux containers in a VM-per-container model, not a jail for an arbitrary native macOS
  process. Its own README states the floor plainly: **"You need a Mac with Apple silicon to run
  `container`... supported on macOS 26... We do not support older versions of macOS."** Fetched
  from the primary source:
  [`apple/container` README](https://github.com/apple/container/blob/main/README.md).
  `container` 1.0.0 shipped 2026-06-09 (reported:
  [explainx.ai](https://explainx.ai/blog/apple-container-1-linux-containers-macos-26-swift-2026)).
  Built on `Containerization` + `Virtualization.framework` (same README).

### C3. Virtualization.framework from a non-App-Store CLI

- Entitlement `com.apple.security.virtualization` (Boolean) is required to use the Virtualization
  APIs; `VZBridgedNetworkDeviceAttachment` additionally needs `com.apple.vm.networking`.
  Documented:
  [entitlement reference](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.virtualization).
  Applied like any entitlement, e.g. `codesign --entitlements vz.entitlements -s - <binary>`.
  Whether **pure ad-hoc signing with no Apple Developer Program account** suffices at runtime, vs.
  needing at least a free-tier account behind the signing identity, is not cleanly established:
  one forum thread reports an ad-hoc-signed binary failing the check until the developer obtained
  an account — reported, single anecdote:
  [Developer Forums 698220](https://developer.apple.com/forums/thread/698220). A separate forum
  post claims some virtualization-adjacent capability is "restricted to developers of
  virtualization software" requiring Apple contact — reported, not confirmed against primary doc
  wording in this pass: [Developer Forums thread](https://discussions.apple.com/thread/255626985).
- Contrast `libkrun`, which uses the lower-level **Hypervisor.framework** (not
  Virtualization.framework), gated by **`com.apple.security.hypervisor`**; its build docs show
  this working with a bare local self-signature —
  `codesign --sign - --force --entitlements=example.entitlements ./binary` — no account
  provisioning documented as needed. Documented:
  [`libkrun` repo](https://github.com/libkrun/libkrun) (error `krun_start_enter failed: Invalid
  argument (errno 22)` is documented as exactly the unsigned-entitlement case).

---

## D. CI runners

### D1. `/dev/kvm` on GitHub-hosted `ubuntu-latest` (public repos, free tier)

- Documented, official: GitHub's **larger** Linux runners support hardware-accelerated
  nested virtualization as an opted-in/paid feature —
  [GitHub Changelog](https://github.blog/changelog/2023-02-23-hardware-accelerated-android-virtualization-on-actions-windows-and-linux-larger-hosted-runners/).
- For the **free/standard** runner there is **no official documentation** either way — a
  community documentation request was closed without a policy statement:
  [`runner-images` #12933](https://github.com/actions/runner-images/issues/12933). What's
  **reported** (conflicting): one source says `/dev/kvm` "exists for all free/non-large
  `ubuntu-latest` runners since a runner update in January 2024" —
  [actuated.com blog](https://actuated.com/blog/kvm-in-github-actions) — others call standard
  runners KVM-unsuitable with inconsistent device presence. **Treat as reported and
  inconsistent, not documented**; gate any L2 microVM CI path behind a runtime probe (as
  `delulu`'s own `delulu_kvm` cfg flag already does) and/or larger runners.
- Real workaround when `/dev/kvm` exists but permissions don't (reported): a udev rule
  `KERNEL=="kvm", GROUP="kvm", MODE="0666", OPTIONS+="static_node=kvm"` in
  `/etc/udev/rules.d/`, then `udevadm control --reload-rules && udevadm trigger
  --name-match=kvm` — needs `sudo` (granted to the default runner user) but not interactive
  admin. Same actuated.com source; also
  [`runner-images` #7541](https://github.com/actions/runner-images/issues/7541).

### D2. `macos-latest` (Apple silicon): nested virtualization

- **Not supported**, explicitly declined. Apple added nested-virtualization support
  (`hv_vm_config_set_el2_enabled`) starting macOS 15 on M3+, but GitHub's own issue requesting it
  for Actions macOS runners (14/15/26, all arm64) was **closed "not planned."** Documented
  (primary, GitHub's repo):
  [`runner-images` #13505](https://github.com/actions/runner-images/issues/13505). Any macOS L2
  story needs a self-hosted Apple-silicon Mac.

### D3. `windows-latest`: Hyper-V / Windows Hypervisor Platform

- `windows-latest` now points at **Windows Server 2025** (rollout complete 2025-09-30).
  Documented: [`runner-images` #12677](https://github.com/actions/runner-images/issues/12677),
  [Windows Server 2025 readme](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md).
- Hyper-V/WHP is enabled on GitHub's **larger** Windows runners but **not by default on the
  standard free runner**. Reported (maintainer discussion, not formal policy):
  [`runner-images` discussion #9285](https://github.com/actions/runner-images/discussions/9285).
  Same conclusion as D2: no nested-virt assumption on the free tier.

---

## E. VMMs as external binaries

### E1. Firecracker

- Driven entirely over an **HTTP API on a Unix socket** — boot-source, drives,
  network-interfaces, vsock, machine-config, and an `actions` endpoint (e.g. `InstanceStart`).
  ([docs tree](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md)).
- The **jailer** must itself start as **root** (needed to set up cgroups/namespaces/chroot), then
  **drops to an unprivileged uid/gid** before exec'ing Firecracker: unshares into new mount/PID/
  network namespaces, `pivot_root`s into a minimal chroot, places the process in a cgroup, and
  puts the API socket inside the jail owned by the dropped-to identity. Reported (project
  architecture, consistent with documented jailer CLI/behavior):
  [DeepWiki, "Jailer"](https://deepwiki.com/firecracker-microvm/firecracker/6.1-jailer). That
  root-then-drop step is real attack surface: **CVE-2026-1386**, a symlink privilege-escalation
  flaw in the jailer letting a local user overwrite host files, fixed in **v1.13.2**/**v1.14.1**.
  Documented: [SentinelOne CVE-2026-1386](https://www.sentinelone.com/vulnerability-database/cve-2026-1386/).
- Guest needs: uncompressed **ELF `vmlinux`** recommended on x86_64 (`bzImage` accepted but costs
  boot time/memory); aarch64 wants **PE `Image`**. Rootfs is any filesystem image with an init
  system and its driver compiled into the guest kernel (getting-started uses ext4 + OpenRC). A
  supported guest kernel version stays supported ≥2 years, with ≥2 major guest/host versions
  concurrently supported. Documented:
  [`firecracker/docs/getting-started.md`](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md),
  [`firecracker/docs/kernel-policy.md`](https://github.com/firecracker-microvm/firecracker/blob/main/docs/kernel-policy.md).
- Version/cadence, measured via GitHub Releases API 2026-09-17: latest **v1.17.0** (2026-09-10);
  recent history v1.16.2 (09-10), v1.16.1 (07-02), v1.16.0 (06-04), v1.15.1 (04-07), v1.14.4
  (04-07), v1.14.3 (03-13), v1.15.0 (03-09) — roughly every 4–6 weeks with parallel patch
  releases on more than one active branch. Licence: **Apache-2.0**.

### E2. Cloud Hypervisor

- Also driven over an **API Unix socket** (`--api-socket`). `virtio-fs` needs the separate
  **`virtiofsd`** daemon (own socket via `--socket-path`, referenced from `--fs`). Documented:
  [`cloud-hypervisor/docs/virtiofs-root.md`](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/docs/virtiofs-root.md).
- Hypervisors: **KVM** on Linux, and **MSHV** — confirmed this means the Microsoft Hypervisor
  **as deployed on Linux hosts in Azure** (`/dev/mshv` on that Linux host), **not** Hyper-V on a
  Windows desktop. Documented:
  [`cloud-hypervisor/docs/mshv.md`](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/docs/mshv.md)
  — matches Hyperlight's identical framing in E3.
- Version, measured via GitHub Releases API: latest **v53.0** (2026-07-12). Licence:
  **`Apache-2.0 OR BSD-3-Clause`**, confirmed by reading the repo's `LICENSES/` directory
  directly (GitHub's license-detector field was null).

### E3. Hyperlight and hyperlight-wasm

- Licence: **Apache-2.0** for both `hyperlight-dev/hyperlight` and `hyperlight-dev/hyperlight-wasm`
  (GitHub API `license.spdx_id`, queried 2026-09-17); neither repo archived.
- Hypervisor backends: **KVM** and **`/dev/mshv`** on Linux, **Windows Hypervisor Platform** on
  Windows. Documented/reported consistently:
  [Microsoft Open Source Blog](https://opensource.microsoft.com/blog/2025/03/26/hyperlight-wasm-fast-secure-and-os-free/),
  [`hyperlight-wasm` repo](https://github.com/hyperlight-dev/hyperlight-wasm).
- `hyperlight-wasm` embeds **Wasmtime** as guest runtime — "strongly integrated... allowing code
  built for wasmtime to run in Hyperlight without modification" — targeting **WASI + the
  WebAssembly Component Model**. Same Microsoft blog.
- Maturity: **pre-1.0** both (`hyperlight` v0.17.0, 2026-08-28; `hyperlight-wasm` v0.15.0,
  2026-09-04 — GitHub Releases API). Hyperlight is a **CNCF Sandbox** project (early-maturity
  governance tier) — [hyperlight.org](https://hyperlight.org/). No independently-measured
  latency/throughput numbers were found in this pass — treat specific figures as **not
  established** beyond the project's own qualitative "very low latency" framing.

### E4. libkrun

- Licence: **Apache-2.0** (GitHub API). Platforms: **Linux + KVM**, **macOS/ARM64 + HVF** —
  "a dynamic library that allows programs to easily acquire the ability to run processes in a
  partially isolated environment using KVM Virtualization on Linux and HVF on macOS/ARM64."
  Documented: [`libkrun/libkrun`](https://github.com/libkrun/libkrun).
- `libkrunfw` bundles the guest Linux kernel as a dynamic library — "does not execute any code
  from it, acting as a mere storage format," stated as why it isn't a Linux-kernel derivative
  work for licensing purposes. Latest release **v5.6.1** (2026-09-16 — GitHub Releases API).
  Documented: [`libkrun/libkrunfw`](https://github.com/containers/libkrunfw). Bundled-kernel size
  in MB: **not established** in this pass.
- macOS signing: needs **`com.apple.security.hypervisor`**; build docs show this satisfied with
  a bare local self-signature — no App Store/developer-account provisioning documented as
  necessary (contrast the murkier `com.apple.security.virtualization` picture in C3). Same repo.

---

## F. Egress and DNS

### F1. AWS AgentCore code-interpreter DNS exfiltration (2026)

- Disclosed 2026-03-16 (BeyondTrust Phantom Labs): AgentCore Code Interpreter's **"Sandbox"
  network mode** blocked outbound HTTP/HTTPS/TCP but left **DNS on UDP 53 fully open**, despite
  AWS marketing it as "complete isolation with no external access" — DNS-based exfiltration/C2
  (A-record-encoded data, credential theft, S3 enumeration) was possible end to end. Reported:
  [BeyondTrust](https://www.beyondtrust.com/blog/entry/pwning-aws-agentcore-code-interpreter),
  corroborated by
  [Unit 42](https://unit42.paloaltonetworks.com/bypass-of-aws-sandbox-network-isolation-mode/).
- Remediation reported as partial: AWS's first response updated docs to recommend "VPC mode"
  (full customer egress control) over "Sandbox mode," rather than closing the DNS hole directly;
  a later report claims Sandbox mode's DNS path is now closed, while a separate follow-up
  reports VPC mode still leaking DNS post-disclosure — both **reported, not reconciled** here:
  [dev.to follow-up](https://dev.to/haitmg/aws-bedrock-agentcore-vpc-mode-still-leaks-dns-after-unit-42-disclosure-588o).
- **Lesson**: an egress-deny posture blocking TCP/UDP-to-arbitrary-hosts but leaving the
  resolver reachable is not isolation — DNS needs the same allow-list enforcement as every other
  protocol, not an exemption as "just" a control-plane service.

### F2. DNS-aware allow-listing (Cilium, Anthropic sandbox-runtime)

- Cilium's `toFQDNs` lets an egress rule name a domain (`matchName`) or wildcard
  (`matchPattern`, e.g. `*.twitter.com`); the dataplane snoops allowed DNS answers and
  dynamically admits the returned IPs, so policy stays valid as a name's IPs rotate — DNS
  queries to the trusted resolver must be separately allowed. Documented:
  [Cilium, DNS-Based Policies](https://docs.cilium.io/en/stable/security/dns/).
- Anthropic's `sandbox-runtime` targets rebinding specifically at the proxy layer: hostnames
  match by name, but "whoever controls a permitted name's DNS can control what it resolves to" —
  so before dialing, the proxy **resolves once, drops any address in a denied set (loopback,
  link-local, the sandbox host's own addresses, or explicit deny entries), and connects to the
  one surviving address** — no second lookup between check and dial, closing the TOCTOU gap.
  Documented (primary):
  [`anthropic-experimental/sandbox-runtime`](https://github.com/anthropic-experimental/sandbox-runtime).
  Its own tracker shows this as an active, imperfect boundary, e.g.
  [issue #88, DNS exfiltration via `allowLocalBinding: true`](https://github.com/anthropic-experimental/sandbox-runtime/issues/88)
  — a pattern with known edge cases, not a closed problem.

### F3. Cloud metadata endpoints to deny by default

- **AWS**: IMDS at `169.254.169.254`, and (if IPv6 enabled) **`fd00:ec2::254`**. Documented:
  [AWS, "Limit access to IMDS"](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-metadata-limiting-access.html).
- **AWS ECS task metadata**: `169.254.170.2` (v2/v3/v4), source of
  `AWS_CONTAINER_CREDENTIALS_RELATIVE_URI`-style per-task credentials. Documented:
  [ECS task metadata v4](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-metadata-endpoint-v4.html).
- **GCP**: `metadata.google.internal` (→ `169.254.169.254`), requiring header
  `Metadata-Flavor: Google` — defeats simple GET-forgery SSRF, not `gopher://`-capable request
  smuggling. Documented:
  [Google Cloud, VM metadata](https://docs.cloud.google.com/compute/docs/metadata/querying-metadata).
- **Azure**: same `169.254.169.254`, gated by required header `Metadata: true`. Documented:
  [Azure IMDS](https://learn.microsoft.com/en-us/azure/virtual-machines/instance-metadata-service)
  (AKS also documents pod-level IMDS restriction:
  [Block pod access to IMDS (preview)](https://learn.microsoft.com/en-us/azure/aks/imds-restriction)).
- **Alibaba Cloud**: `100.100.100.200`, default **token-free** access (vendor-flagged SSRF risk);
  "security hardening mode" requires a `PUT`-obtained token first, same pattern as AWS IMDSv2.
  Documented: [Alibaba Cloud metadata guide](https://www.alibabacloud.com/help/en/ecs/user-guide/view-instance-metadata/).
- Takeaway: `169.254.169.254` is shared by AWS/GCP/Azure, so one deny rule plus the documented
  companions (`fd00:ec2::254`, `169.254.170.2`, `100.100.100.200`) covers the likely CI/cloud
  hosts; a sandboxed child should never reach any of them regardless of declared authority — no
  legitimate DeluluLang program authority should resolve to "read the host's cloud credentials."

---

## What a sandbox designer should conclude
1. **Ubuntu 24.04's AppArmor userns gate is ON by default** (LP #2046477) — `unshare(CLONE_NEWUSER)`
   and `bwrap` are silently denied on `ubuntu-latest` unless `delulu` ships/loads a profile
   granting `userns,` — a CI-image prerequisite, not a runtime fallback decision.
2. Landlock on `ubuntu-latest` (kernel 6.17) tops out at **ABI 7** — design for that floor.
3. `seccompiler`/`landlock` crates are allowlist-clean; `bubblewrap` (LGPL-2.1) needs an explicit
   owner licensing call even as an unlinked external binary.
4. cgroups v2 limits are reachable without root only via `systemd-run --user --scope`, riding
   logind's default delegation — no path on non-systemd hosts.
5. `RLIMIT_NPROC` is per-real-uid, shared with the host's own threads — isolates a worker only if
   it runs under a uid distinct from `delulu` itself.
6. Windows: Job Objects are already wired in `foreign_worker.rs`; missing are a **restricted
   token from the process's own token** (no admin, B2) and a **capability-less AppContainer**
   for network denial — WFP is a dead end without admin.
7. macOS has no supported CLI-sandboxing story: `sandbox-exec` works but is deprecated and even
   Apple's own containerization team has no answer (issue #737, open) — name this risk, don't
   hide it behind "we use Seatbelt."
8. L2 microVM CI is **effectively Linux-only**: macOS nested virt closed "not planned" (D2),
   Windows WHP off by default (D3), Linux `/dev/kvm` on free runners reported-inconsistent (D1) —
   `delulu_kvm`'s existing cfg-gating is confirmed as the right posture, not over-caution.
9. Firecracker's jailer needs root-then-drop (CVE-2026-1386 lived there) — "external binary"
   doesn't mean "no privileged setup," even though `delulu` itself never runs as root.
10. Treat DNS as a first-class allow-listed protocol (F1) and deny the metadata-address family by
    default regardless of declared authority (F3) — both look done until someone checks.

## Not established

- Exact kernel version for Landlock ABI 9, 10, 11 (capabilities confirmed, versions not).
- Whether free/standard `ubuntu-latest` reliably exposes `/dev/kvm` today — reports conflict, no
  GitHub policy statement found (D1).
- Whether pure ad-hoc signing with **no** Apple Developer account suffices for
  `com.apple.security.virtualization` specifically (vs. `com.apple.security.hypervisor`, which
  libkrun documents working with bare self-signing) — one forum anecdote only; Apple's own
  "Adding the Virtualization Entitlement" page could not be fetched past its title.
- `libkrunfw`'s bundled-kernel size in MB.
- Independently measured (non-vendor-qualitative) latency/throughput numbers for Hyperlight or
  hyperlight-wasm.
- Whether AWS AgentCore's Sandbox-mode DNS path is fully closed today or only documented-around
  (F1) — AWS's fix claim and a "VPC mode still leaks" follow-up are unreconciled.
- Precise vCPU/RAM specs of free-tier GitHub-hosted runners as of 2026-09-17 (not directly
  asked; D1–D3's conclusions assume "standard" vs. "larger" tiers without pinning today's exact
  numbers).
