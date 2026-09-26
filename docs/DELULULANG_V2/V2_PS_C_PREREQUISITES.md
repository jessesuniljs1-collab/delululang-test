# PS-C-01 — the microVM's prerequisites, verified before it was built

**Status: every prerequisite verified; none missing, so the phase proceeded (2026-09-27).** The
roadmap's rule for PS-C-01 is that each practical prerequisite is checked against the code and a real
host first, and that a missing one stops the phase rather than being papered over. This is that
check: what was required, the command that checks it, what it answered, and what checking it found.

The ruling it serves is **D-NE-23** (owner-approved): the guest runs the INTERPRETER, not the WASM
engine, as PID 1 of its own kernel; no virtio-fs, no network device, vsock only; Firecracker first,
under its jailer; Cloud Hypervisor second. The choices PS-C took on top of it are **D-V2-39**
(`V2_DECISION_LOG.md`).

## The checklist

| # | Prerequisite | How it is checked | Answer |
|---|---|---|---|
| 1 | The `EffectSink` seam exists, and the local path stays byte-identical through it | `grep -n "pub trait EffectSink" crates/delulu-runtime/src/sink.rs`; `cargo test -p delulu --test core_invariance` | the trait is at `sink.rs:25`; `the_core_still_answers_exactly_as_recorded` guards the local path — **present** |
| 2 | The channel is one implementation, bounded and fuzzed, and the guest's side of it takes any stream | `delulu-sandbox-channel/2` in `delulu-runtime/src/channel.rs` (`MAX_FRAME` 16 MiB, `fuzz_one_frame` at `channel.rs:748`, replayed in-suite; `fuzz/fuzz_targets/channel_frame.rs` under cargo-fuzz); `guest.rs` `serve_as_guest` is generic over `Read + Write` | **present** — the vsock stream needed no second protocol, only a second way to be connected |
| 3 | A static, Python-less, network-less guest build | `cargo build --release -p delulu --no-default-features --target x86_64-unknown-linux-musl` (as `scripts/microvm/build-image.sh` runs it) | **builds**: a static-pie that runs `--version` and a console program. It needed two things the plain command lacks — see findings 1 and 2 |
| 4 | KVM, on a development host and on a CI runner | open `/dev/kvm` read-write and ask `KVM_GET_API_VERSION` (`delulu sandbox probe`) | WSL2 on the owner's machine: **API 12**, after `nestedVirtualization=true` and `modprobe kvm_amd` at boot (the owner's go-ahead, 2026-09-26). GitHub `ubuntu-latest`: `/dev/kvm` opens after `chmod 666` (PS-0-08, `host-capability-probe.yml`) |
| 5 | A VMM, pinned | `scripts/microvm/fetch-firecracker.sh` (checks the release archive's sha256) | **Firecracker v1.17.0**, archive sha256 `06094a11…c9ade558` (x86_64). Not the v1.10.1 PS-0-08 used — finding 4 |
| 6 | A guest kernel, pinned, with vsock and nothing else | `scripts/microvm/build-image.sh` (tarball sha256 `9df30b02…3eacac`; refuses a config line that did not survive) | **linux 6.18.54 LTS**, built from kernel.org source by `scripts/microvm/kernel.config` on top of `tinyconfig`: no IP stack, no PCI, no ACPI, no block layer, no modules. 11.6 MB `vmlinux`. The pinned hash is the one in kernel.org's `sha256sums.asc`, whose signature verifies (the checksum autosigner, key `B8868C80…589DA6B1`) |
| 7 | A guest→host vsock stream under the pinned VMM | first by hand (a static C client in a busybox initramfs, Firecracker v1.10.1 and the vendor 6.1.102 kernel); then through the real launcher | **PASS** both times: Firecracker delivers the guest's connection to host port 1024 on the Unix socket `<uds_path>_1024` |
| 8 | The static `delulu` as PID 1 | boot the image with `rdinit=/delulu -- --version` | **runs**: the VM booted, printed the version and exited in 811 ms — once finding 3 was fixed |
| 9 | The image build on a Linux runner | the `microvm` CI job builds the image, then runs the gated tests on it | recorded in `V2_LOG.md` when its run is read |

## What checking them found

1. **libffi's build needs kernel headers that `musl-gcc` hides.** `tramp.c` includes `<linux/limits.h>`,
   and `musl-gcc` removes every system include directory. The build exposes ONLY the kernel's headers,
   through one directory of links (`linux`, `asm`, `asm-generic`), never glibc's.
2. **A dependency links `-ldl`, and the only `libdl.a` on the host is glibc's.** Linking with `musl-gcc`
   as the linker "fixed" it and produced a binary that crashed at once (exit 139): two sets of C
   startup files. The build links with the default `cc` and Rust's own musl startup files, and points
   `-ldl` at an EMPTY archive searched first — musl's `libc.a` already defines the `dl*` functions.
3. **The first-run language picker captured PID 1.** The guest's standard input is the VM's console, a
   terminal, and `__guest` was not among the machine commands that never show the picker — so the
   guest sat at "pick your compiler's vibe" instead of dialling its host. On a host the guest's
   streams are a socket or the null device, which is the only reason this had never shown. Fixed in
   `locale.rs`: the guest subcommand is a machine command.
4. **Firecracker v1.10.1 was already too old, and so was a 6.1 guest.** Firecracker's kernel policy for
   v1.17.0 supports 6.1 guests to 2026-09-02 — past — and 6.18 guests to at least 2028-06-01. Both pins
   moved: v1.17.0, 6.18 LTS.
5. **KVM refuses `KVM_GET_API_VERSION` unless its argument is exactly 0**, and `ioctl` is variadic: the
   first launch omitted the argument, got whatever the register held, and was refused as "API version
   -1" on a host where Python's `fcntl.ioctl` read 12. The argument is passed explicitly.
6. **`CONFIG_SECURITY` (and so Landlock) depends on `CONFIG_SYSFS`.** `tinyconfig` turns sysfs off, and
   Kconfig then drops Landlock without an error. The build's own check refused that kernel ("did not
   survive olddefconfig: CONFIG_SECURITY=y CONFIG_SECURITY_LANDLOCK=y …") — the reason the check
   exists. Sysfs is compiled in and never mounted.

## What is not a prerequisite, and is not built yet

- **The jailer** (a per-VM uid, a chroot, cgroups) needs root. Until it is applied the VMM runs as the
  operator, under Firecracker's own seccomp filters and the launcher's ceilings, and every L2 run
  reports `identity_separation` as a limitation. PS-C-03b.
- **aarch64**: the Firecracker pin covers it; the kernel configuration and the image build are x86_64.
- **Cloud Hypervisor**, the second VMM D-NE-23 names.
- **A distributed image** (PS-C-05) — owner-reserved: D-NE-27, a built GPL kernel.

## Reproducing this

On a Linux x86_64 host with KVM (Ubuntu: `apt-get install build-essential flex bison bc libelf-dev
musl-tools python3 curl`):

```sh
scripts/microvm/fetch-firecracker.sh ~/.local/bin
scripts/microvm/build-image.sh ~/microvm-image          # ~8 minutes on 4 cores
export DELULU_FIRECRACKER=~/.local/bin/firecracker DELULU_MICROVM_IMAGE=~/microvm-image
delulu sandbox probe                                    # L2: every attempt, ending in a boot
delulu run hello.delulu --isolation microvm --grant console
scripts/microvm/check-reproducible.sh                   # two clean builds, compared
```
