#!/usr/bin/env bash
#
# PS-C-06: run an ordinary program at L2 against a HOSTILE guest, once per hostile mode, and record
# what the host did: its exit, how long it took, how much of the guest's console reached the operator,
# and whether a VMM or a VM directory outlived the run.
#
# The hostile image is the real image's KERNEL with a hostile initramfs, and a manifest that vouches for
# both — it stands for a compromised image build, which is the one way a hostile guest reaches a host:
# the launcher boots only what a manifest names, and refuses anything else before boot (T13).
#
# usage: scripts/microvm/redteam/hostile.sh <image-dir> <delulu> [firecracker]
set -uo pipefail
IMAGE=${1:?usage: hostile.sh <image-dir> <delulu> [firecracker]}
DELULU=${2:?usage: hostile.sh <image-dir> <delulu> [firecracker]}
export DELULU_FIRECRACKER=${3:-${DELULU_FIRECRACKER:-$(command -v firecracker)}}
HERE=$(cd "$(dirname "$0")" && pwd)
W=$(mktemp -d)
trap 'rm -rf "$W"' EXIT
export DELULU_NO_FIRST_RUN=1 DELULU_STATE_DIR="$W/state"
mkdir -p "$W/state/audit"
cat > "$W/hello.delulu" <<'EOF'
module m

fn main(root: Root) ! {Write} {
  root.console().println("the program ran")
}
EOF

# Firecracker processes whose command line names host PID's VM directory. Python, not grep: a grep for
# the pattern finds itself, because its own command line holds the pattern.
vmms_of() {
    python3 - "$1" <<'PY'
import glob, sys
needle = ("delulu-vm-%s-" % sys.argv[1]).encode()
n = 0
for p in glob.glob("/proc/[0-9]*/cmdline"):
    try:
        b = open(p, "rb").read()
    except OSError:
        continue
    if b"firecracker" in b and needle in b:
        n += 1
print(n)
PY
}

bad=0
verdict() { echo "redteam-hostile: FAILED — $*"; bad=1; }
for mode in flood garbage huge ports silent; do
    img="$W/img-$mode"
    mkdir -p "$img"
    cp "$IMAGE/vmlinux" "$img/vmlinux"
    musl-gcc -static -O2 -DHOSTILE_MODE="\"$mode\"" -o "$W/hostile-$mode" "$HERE/hostile-guest.c"
    python3 "$HERE/../mkinitramfs.py" "$W/hostile-$mode" "$img/initramfs.cpio"
    python3 - "$img" <<'PY'
import hashlib, json, os, sys
d = sys.argv[1]
def sha(name):
    with open(os.path.join(d, name), "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()
with open(os.path.join(d, "manifest.json"), "w") as f:
    json.dump({"format": "delulu-microvm-image/1",
               "kernel": {"file": "vmlinux", "sha256": sha("vmlinux"), "version": "hostile"},
               "initramfs": {"file": "initramfs.cpio", "sha256": sha("initramfs.cpio")}}, f)
PY
    start=$(date +%s%N)
    ( cd "$W" && exec env DELULU_MICROVM_IMAGE="$img" "$DELULU" run hello.delulu --isolation microvm --grant console ) \
        > "$W/out-$mode" 2> "$W/err-$mode" &
    pid=$!
    wait "$pid"
    rc=$?
    took=$(( ($(date +%s%N) - start) / 1000000 ))
    left_dirs=$(ls -d /tmp/delulu-vm-"$pid"-* 2>/dev/null | wc -l)
    left_vmms=$(vmms_of "$pid")
    echo "== $mode: exit=$rc ms=$took stderr_bytes=$(wc -c < "$W/err-$mode") program_ran=$(grep -c 'the program ran' "$W/out-$mode") vm_dirs_left=$left_dirs vmms_left=$left_vmms"
    grep -v '^AAAA' "$W/err-$mode" | grep -v '^$' | cut -c1-220 | head -12
    # The verdict: a hostile guest gets no program run, no more than the console bound onto the
    # operator's terminal, no second channel answered, and nothing left behind.
    [ "$rc" -ne 0 ] || verdict "$mode: the run succeeded"
    [ "$(grep -c 'the program ran' "$W/out-$mode")" -eq 0 ] || verdict "$mode: the program ran"
    [ "$left_dirs" -eq 0 ] || verdict "$mode: a VM directory was left behind"
    [ "$left_vmms" -eq 0 ] || verdict "$mode: a VMM outlived the run"
    [ "$(wc -c < "$W/err-$mode")" -lt 70000 ] || verdict "$mode: more than the 64 KiB console bound reached the operator"
    if grep -q "the host SENT a byte" "$W/err-$mode"; then verdict "$mode: the host answered a second channel"; fi
done
[ "$bad" -eq 0 ] && echo "redteam-hostile: every expectation held"
exit "$bad"
