# PS-C-06 — the L2 red team: what was attacked, what held, what was fixed, what was not attempted

**Date:** 2026-09-27. **Target:** `--isolation microvm` at `ab1bfdc` and after, on Linux x86_64 with
KVM — this machine's WSL2 (nested KVM, AMD) and GitHub's `ubuntu-latest` runner. **Rule:** every
claim below was produced by a command in this repository, and every defence it credits was falsified
by a mutant that let the attack through (the list is in §6). Kernel and hypervisor exploits are
category 7 throughout: nothing here claims their absence (§7).

The attacker model is the one L2 exists for: **the guest is hostile.** It may be a program nobody
reviewed, or — the stronger case — a compromised guest image, which is the only way arbitrary code
reaches PID 1: the launcher boots only what an image's manifest names, and refuses a changed byte
before boot (T13). Every test below that needs arbitrary guest code builds such an image — the real
kernel, a hostile initramfs, a manifest that vouches for both.

## 1. What a guest can reach — the native probe (T9)

`scripts/microvm/redteam/probe.sh` boots the image's OWN kernel with a static C probe as init
(`redteam/guest-probe.c`), in the exact VM shape the launcher builds: one vCPU, one vsock device, no
network interface, no drive. What it found, identically on both hosts:

| Question | Answer |
|---|---|
| its arguments | `/delulu __guest --vsock 1024` — the launcher's, nothing else |
| its environment | `HOME=/`, `TERM=linux`, and `pci=off`, which Firecracker itself appends to the kernel command line and the kernel, not knowing it, hands to init. Nothing from the host's environment |
| its filesystem | `/delulu`, `/dev`, and the kernel's own empty `/root` — the initramfs, in RAM |
| virtio devices | exactly one: `virtio0`, device id `0x0013` (vsock), driven by `vmw_vsock_virtio_transport` |
| network interfaces | `lo` only; no NIC exists |
| block devices | none — `/sys/class/block` does not exist, because the kernel has no block layer. A block-device node the probe makes by hand opens as ENXIO: no driver answers it |
| sockets | IPv4 (stream and datagram), IPv6, packet and Unix: all refused, "address family not supported". Netlink (kernel-internal) and vsock: created |
| `/proc` | absent (not compiled in) |

The probe is a gate on the KVM CI job: it fails unless every one of those holds.

## 2. Hostile guests against the host

`scripts/microvm/redteam/hostile.sh` runs an ordinary program at L2 against five hostile guests
(`redteam/hostile-guest.c`) and checks the host's side of each. Results (WSL2; the CI job repeats
them as a gate):

| Guest | What it does | What the host did | Time |
|---|---|---|---|
| `flood` | writes 1 MiB to its console, never dials in | relayed exactly the 64 KiB bound, said the rest was discarded, refused the run: "the guest did not dial in within 10s" | 10.7 s |
| `garbage` | dials in, sends the ready byte, then 4 KiB of noise | "frame length exceeds the channel's bound" — refused without allocating | 3.1 s |
| `huge` | announces a 4 GiB frame | the same refusal, before any allocation | 3.1 s |
| `ports` | dials host port 1024 again, then 1025, 52, 2³²−1, context 1 and itself | other ports and contexts: refused (reset). The second 1024 dial: see §3, finding H1 | 3.1 s |
| `silent` | dials in and says nothing | "the guest said nothing for 60s, and the channel's deadline ended the run" | 64.5 s |

In every mode: the program did not run, the command exited 1, no VM directory and no VMM outlived the
run, and nothing but the bounded console reached the operator.

## 3. What the red team found, and what was done

| # | Finding | Severity | Done |
|---|---|---|---|
| H1 | The dial-in listener stayed open until the host had read the guest's ready byte, so the `ports` guest's second dial to port 1024 CONNECTED — into the listening socket's backlog. Nothing ever read it; it was closed when the boot returned | low: no second channel was ever served | the listener is closed, and its socket file removed, the moment the first connection is accepted. A dial that races the first can still complete into the backlog before that accept; the guest then reads end-of-file on it (`hostile-guest.c` checks, and the gate fails if the host ever sends a byte) |
| H2 | Two host errors reached the operator as raw OS text: "failed to fill whole buffer" (a guest that hung up) and "Resource temporarily unavailable (os error 11)" (one silent until the deadline) | message quality | worded: "the guest closed the channel without saying goodbye", "the guest said nothing for 60s, and the channel's deadline ended the run" — at L1 as well, which shares the code |
| H3 | The `sandbox-launch` audit record did not name the image the VM booted — SANDBOX_TEST_PLAN §5.7 requires it | audit completeness | the record and the run report now carry `image: {kernel_sha256, initramfs_sha256, kernel_version}` — the hashes of the copies checked and booted; `microvm_cli.rs` asserts they equal the manifest's |
| J1 | The first jailed launch failed inside the chroot with KVM's "permission denied": the jail base was under `/run`, which is mounted `nodev`, so the `/dev/kvm` node the jailer creates there cannot be opened | blocking (fail-closed) | jails live under `/srv/delulu-jailer` (`DELULU_JAIL_BASE` to move them); a base on a `nodev` filesystem is refused with that reason |
| J2 | The jailer's `setuid` clears `PR_SET_PDEATHSIG`: a jailed VMM does not die with its host. Measured: host AND reaper killed, the console pipe held open — the VMM still running after 15.7 s, bounded only by its processor-time ceiling | **high** for the jailed path, had it shipped without an answer | the reaper (D-V2-40): blocked on a pipe only its host writes to; when it closes, it kills the VMM by pid AND jail id and removes the jail. The test requires the VMM gone within 2 s of the host's death |
| J3 | The first version of the reaper test allowed 8 s, and a mutant reaper that killed nothing SURVIVED it: the VM was ending ~5 s after its host anyway, through its console pipe breaking | a gate that could not fail | the bound is 2 s — only the reaper is that prompt — and the mutant is now killed ("outlived its host by 2.0s") |
| J4 | The first chroot assertion read `/proc/<pid>/root` and got `/`: the jailer pivots into the chroot inside a new mount namespace, so from outside the root reads as `/` | a test that failed on a correct jail | the test checks what the VMM can SEE (its jail's files, none of `/etc`, `/home`, `/usr`, `/proc`, `/root`) and that its mount namespace differs from the host's |

Found while building the launcher, recorded in `V2_PS_C_PREREQUISITES.md` and `V2_LOG.md`: the
first-run language picker captured PID 1; `KVM_GET_API_VERSION` refused an omitted variadic argument;
`tinyconfig` silently dropped Landlock.

## 4. The jailed VMM (as root)

Verified by `microvm_cli.rs::as_root_the_vmm_is_jailed_as_its_own_uid_and_dies_with_its_host`, on
this machine as root and on the CI runner through `sudo -n`: the VMM runs as a uid from the reserved
block; its view of the filesystem is its jail — `firecracker`, `vmlinux`, `initramfs.cpio`, `dev`, and
nothing of the host's; its mount namespace is its own; the report's identity row says "the VMM: a uid
of its own, in a chroot (Firecracker's jailer)" and `identity_separation` is not a limitation; a host
killed with SIGKILL takes the VMM with it through the reaper and the jail is removed; and root without
a jailer is refused, never run as root.

## 5. Measurements (not a gate)

The test plan's §7, once, on one machine: WSL2 on an AMD Ryzen 7 7840HS laptop (4 vCPUs to WSL, KVM
nested under Hyper-V), release build, the median of five runs of the whole command, wall clock.

| Program | L0 in-process | L1 `--sandbox` | L2 `--isolation microvm` |
|---|---|---|---|
| hello (one `println`) | 8 ms | 23 ms | 1,248 ms |
| 1,000 `println`s | 9 ms | 79 ms | 1,209 ms |
| a 3,000,000-step loop, then one `println` | 1,573 ms | 1,583 ms | 2,395 ms |

What they say: **L2 costs about 1.2 s per run** — copying and hashing the 38 MB image, starting the
VMM, booting the kernel, unpacking the initramfs, the guest dialling in, and ending the VM — against
tens of milliseconds for L1. **Per effect, L2 is not measurably dearer than L1**: a thousand round trips
over vsock added nothing the run-to-run spread could separate from the launch (L1's thousand cost about
56 µs each). **Compute does not slow down in the VM**: the loop took about 1.15 s in the guest after
subtracting the launch, against about 1.57 s in-process. That difference was not investigated and is
not claimed as a speed-up — the guest is a different build (static musl, no Python, no network client),
and the claim is only the one the plan asked for: the VM does not slow compute. On nested KVM these
figures are an upper bound for a bare-metal host.

## 6. The mutants — each defence, removed, and the test that caught it

| Defence removed | Test that failed |
|---|---|
| the image hash is not compared | `a_flipped_byte_in_the_image_is_refused_before_boot` — read by hand: the tampered image BOOTED and ran the program while the report claimed "the image checked against its manifest before boot" |
| the VMM is not killed with its (unjailed) host | `a_killed_host_takes_its_vm_with_it_and_the_next_launch_sweeps_what_is_left` |
| the orphan sweep | the same test's sweep assertion |
| the probe does not really boot | `the_probe_boots_a_vm_to_report_l2`, through its control — a fake VMM that answers `--version` like Firecracker and cannot boot |
| no processor-time ceiling | `the_processor_time_ceiling_ends_a_spinning_guest` (the wall-clock ceiling ended it at 60 s instead) |
| the VM directory is not removed | `a_finished_run_leaves_no_vmm_and_no_directory` |
| the reaper does not kill | `as_root_the_vmm_is_jailed_as_its_own_uid_and_dies_with_its_host` (after J3) |
| root runs the VMM unjailed | the same test: the jailed guarantees are missing from the report |

## 7. What was NOT attempted

- **Guest-kernel exploits, KVM exploits, Firecracker device-emulation bugs** (the vsock muxer, the
  serial port, the i8042 reset device, the MMIO bus) — category 7. The guest kernel's attack surface was
  cut instead (§1: no IP stack, no PCI, no ACPI, no block layer, no modules, no `/proc`), and the VMM
  runs under Firecracker's own seccomp filters and, as root, the jailer.
- **Side channels** (Spectre-class, cache, timing between VMs) — category 7.
- **SandboxEscapeBench-style difficulty-4/5 scenarios** — out of scope for a pass/fail gate, as the
  test plan says; not run.
- **A fuzzer against the host's channel parser from inside a VM.** The parser is the same one L1 uses,
  fuzzed by `fuzz/fuzz_targets/channel_frame.rs` and replayed in-suite; the hostile guests exercise its
  bounds, not its breadth.
- **cgroups and a network namespace for the VMM** (D-V2-40 §5) — not applied, so not tested.
- **Denial of service on the host** beyond the ceilings the tests exercise (CPU, memory, wall clock,
  console): e.g. many VMs at once exhausting host memory is bounded per VM, not across VMs.
- **aarch64**, and **Cloud Hypervisor**.
- **A distributed image** — PS-C-05, owner-reserved (D-NE-27).
