---
name: wsl-kvm-lab
description: "WSL2 on this machine has working KVM since 2026-09-26 (owner's go-ahead) — how it is wired, where the PS-C microVM toolchain lives in it, and how to run the gated microVM tests"
metadata:
  node_type: memory
  type: reference
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-09-26T19:28:12.835Z
---

On 2026-09-26 Jesse said "go-ahead — so with wsl … no need to ask" for PS-C, and WSL2 (Ubuntu 20.04,
kernel 6.6.87.2, WSL 2.6.1, AMD CPU) was given a working `/dev/kvm`:

- `C:\Users\jesse\.wslconfig` gained `nestedVirtualization=true` under `[wsl2]` (backup:
  `.wslconfig.bak-2026-09-26`; the two keys WSL warns about were left as they were).
- `/etc/wsl.conf` → `[boot] command = "modprobe kvm_amd; chgrp kvm /dev/kvm; chmod 660 /dev/kvm"`.
  The device appears a few seconds AFTER WSL starts — wait before concluding it is absent.
- user `user` is in group `kvm`. `sudo` needs a password; `wsl -u root -e bash <script>` does not.
- Installed (apt, via `wsl -u root`): musl-tools, busybox-static, flex, bison, libelf-dev,
  libssl-dev, gnupg (dirmngr did NOT work, so the kernel.org signature was not checked). Rust at
  `~/.cargo/bin` (not on PATH in `wsl -e`); rustup target `x86_64-unknown-linux-musl`.
- Ops: PowerShell→WSL quoting breaks — write a script into the scratchpad, run
  `wsl -e bash /mnt/c/.../script.sh` ([[delulu-p21-crossaccount]]). Target dirs: `~/delulu-target`
  (musl guest), `~/delulu-gnu-target` (clippy/unit), `~/delulu-kvm-target` (RUSTFLAGS=--cfg delulu_kvm).

The PS-C toolchain in WSL (2026-09-27): kernel source cache `~/.cache/delulu-microvm/`
(`linux-6.18.54.tar.xz`, Firecracker `fc-v1.17.0.tgz`); Firecracker + jailer copied to `~/bin/`; a
built image at `~/microvm-image-a` (rebuild with `scripts/microvm/build-image.sh`, ~450 s). Gated
tests: `DELULU_MICROVM_IMAGE=~/microvm-image-a DELULU_FIRECRACKER=~/bin/firecracker
RUSTFLAGS="--cfg delulu_kvm" cargo test -p delulu --test microvm_cli --test microvm_criterion8`.
Pins are in D-V2-39 (Firecracker v1.17.0 sha256 `06094a11…`, linux 6.18.54 sha256 `9df30b02…`); the
first probe's v1.10.1 + vendor 6.1.102 kernel are superseded. A built kernel is never committed or
shipped (D-NE-27, owner). Related: [[delulu-v2-execution]].
