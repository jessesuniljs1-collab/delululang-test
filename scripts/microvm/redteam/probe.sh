#!/usr/bin/env bash
#
# PS-C-06: boot the guest image's OWN kernel with a native probe as init, in the same VM shape the
# launcher builds (one vCPU, one vsock device, no network interface, no drive), and print what a guest
# can reach. The probe replaces `/delulu` only in a throwaway initramfs; the kernel is the image's.
#
# usage: scripts/microvm/redteam/probe.sh <image-dir> [firecracker]
set -euo pipefail
IMAGE=${1:?usage: probe.sh <image-dir> [firecracker]}
FC=${2:-${DELULU_FIRECRACKER:-firecracker}}
HERE=$(cd "$(dirname "$0")" && pwd)
W=$(mktemp -d)
trap 'rm -rf "$W"' EXIT
command -v musl-gcc >/dev/null || { echo "redteam-probe: needs musl-gcc (musl-tools)" >&2; exit 1; }
musl-gcc -static -O2 -o "$W/probe" "$HERE/guest-probe.c"
python3 "$HERE/../mkinitramfs.py" "$W/probe" "$W/probe.cpio"
cat > "$W/vm.json" <<EOF
{ "boot-source": { "kernel_image_path": "$IMAGE/vmlinux", "initrd_path": "$W/probe.cpio",
                   "boot_args": "console=ttyS0 reboot=k panic=-1 loglevel=0 rdinit=/delulu -- __guest --vsock 1024" },
  "drives": [],
  "machine-config": { "vcpu_count": 1, "mem_size_mib": 128 },
  "vsock": { "guest_cid": 3, "uds_path": "$W/v.sock" } }
EOF
timeout 60 "$FC" --no-api --config-file "$W/vm.json" --level Warning --log-path /dev/null > "$W/console.log" 2>&1 || true
tr -d '\r' < "$W/console.log" | grep '^PROBE' > "$W/probe.txt" || { echo "redteam-probe: the probe printed nothing"; tail -20 "$W/console.log"; exit 1; }
cat "$W/probe.txt"
# The verdict: what a guest of this kernel, in this VM, must NOT be able to reach.
bad=0
need() { grep -qxF "$1" "$W/probe.txt" || { echo "redteam-probe: FAILED — expected: $1"; bad=1; }; }
need "PROBE virtio devices: 1"
need "PROBE virtio device: virtio0 id=0x0013"
need "PROBE block device: cannot open /sys/class/block (No such file or directory)"
for fam in AF_INET/stream AF_INET/dgram AF_INET6/stream AF_PACKET/raw AF_UNIX/stream; do
    need "PROBE socket $fam: refused (Address family not supported by protocol)"
done
need "PROBE /proc: No such file or directory"
if grep "^PROBE net interface: " "$W/probe.txt" | grep -v -e ": lo$" -e ": 1 entries$" -q; then
    echo "redteam-probe: FAILED — a network interface other than loopback"
    bad=1
fi
[ "$bad" -eq 0 ] && echo "redteam-probe: every expectation held"
exit "$bad"
