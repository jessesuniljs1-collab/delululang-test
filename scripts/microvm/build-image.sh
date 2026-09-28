#!/usr/bin/env bash
#
# Build the microVM guest image (PS-C-02): a kernel with vsock and nothing else, and an initramfs whose
# only program is a static, Python-less, network-less `delulu`, run by the kernel as PID 1 in `__guest`
# mode. `crates/delulu/src/microvm.rs` boots what this writes and refuses anything else.
#
# What comes out, in the output directory:
#
#   vmlinux          the guest kernel, built here from kernel.org's source at the version pinned below,
#                    configured by `kernel.config` on top of `tinyconfig`
#   initramfs.cpio   written by `mkinitramfs.py`: /dev, /dev/console, /dev/null, /delulu
#   manifest.json    the sha256 of each, what they were built from, and the format the launcher reads
#
# THE KERNEL IS GPL-2.0, AND THIS SCRIPT NEVER DISTRIBUTES IT (D-NE-27, owner-reserved). It builds
# one for this machine, into `target/` (ignored by git), from source anyone can fetch. Publishing a
# built kernel is a licensing act the owner has not taken; nothing here uploads, attaches or commits
# one, and the CI job that runs this keeps the image inside the job.
#
# Reproducible: every input is pinned (the kernel tarball by sha256, the Rust toolchain by
# `rust-toolchain.toml`, the crates by `Cargo.lock`), every build-time stamp is fixed, and paths are
# remapped out of the Rust binary. `check-reproducible.sh` builds twice from clean and compares.
# Two machines with different C compilers build different kernel bytes; the check is per machine.
#
# Needs: Linux x86_64; gcc, make, flex, bison, bc, perl, libelf-dev (the kernel); musl-tools (the
# guest); python3; curl. On Ubuntu: apt-get install build-essential flex bison bc libelf-dev
# musl-tools python3 curl.
#
# usage: scripts/microvm/build-image.sh [output-dir]      (default: target/microvm-image)
#   DELULU_MICROVM_CACHE   where the kernel tarball is kept (default: ~/.cache/delulu-microvm)
#   DELULU_MICROVM_WORK    where the kernel is built (default: $DELULU_MICROVM_CACHE/work)
#   CARGO_TARGET_DIR       honoured for the guest build (default: <repo>/target)
set -euo pipefail

KERNEL_VERSION=6.18.54
KERNEL_SHA256=9df30b02dd8102bbd0be52556288ef6889ddbe7f1ddb96fbf847d0becf3eacac
KERNEL_URL="https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-${KERNEL_VERSION}.tar.xz"
GUEST_TARGET=x86_64-unknown-linux-musl

die() { echo "build-image: $*" >&2; exit 1; }
step() { echo "== $*"; }

[ "$(uname -s)" = Linux ] && [ "$(uname -m)" = x86_64 ] || die "the guest image is built on Linux x86_64 (this is $(uname -s) $(uname -m))"
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
HERE="$ROOT/scripts/microvm"
OUT=${1:-$ROOT/target/microvm-image}
CACHE=${DELULU_MICROVM_CACHE:-$HOME/.cache/delulu-microvm}
WORK=${DELULU_MICROVM_WORK:-$CACHE/work}
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
mkdir -p "$OUT" "$CACHE" "$WORK"
for tool in gcc make flex bison bc perl musl-gcc python3 curl sha256sum; do
    command -v "$tool" >/dev/null || die "\`$tool\` is not installed"
done

# ---- 1. the kernel source, pinned by hash --------------------------------------------------------
TARBALL="$CACHE/linux-${KERNEL_VERSION}.tar.xz"
if [ ! -f "$TARBALL" ]; then
    step "fetching linux-${KERNEL_VERSION}"
    curl -fsSL --retry 4 --retry-delay 5 -o "$TARBALL.part" "$KERNEL_URL"
    mv "$TARBALL.part" "$TARBALL"
fi
got=$(sha256sum "$TARBALL" | cut -d' ' -f1)
if [ "$got" != "$KERNEL_SHA256" ]; then
    rm -f "$TARBALL"
    die "linux-${KERNEL_VERSION}.tar.xz has sha256 $got, not the pinned $KERNEL_SHA256 — removed, refusing to build"
fi

# ---- 2. configure: tinyconfig, then exactly kernel.config -----------------------------------------
SRC="$WORK/linux-${KERNEL_VERSION}"
rm -rf "$SRC"
tar -xf "$TARBALL" -C "$WORK"
cd "$SRC"
step "configuring (tinyconfig + scripts/microvm/kernel.config)"
make -s ARCH=x86_64 tinyconfig
ARCH=x86_64 scripts/kconfig/merge_config.sh -m -O . .config "$HERE/kernel.config" >/dev/null
make -s ARCH=x86_64 olddefconfig
# Refuse a kernel that is not the one kernel.config describes. Kconfig drops an option whose
# dependency is off without an error, and a guest built from that would be missing a line its
# reviewer read as present.
bad=()
while IFS= read -r line; do
    case "$line" in
        CONFIG_*=*) grep -qxF "$line" .config || bad+=("$line") ;;
        "# CONFIG_"*" is not set")
            opt=${line#\# }; opt=${opt%% *}
            if grep -q "^${opt}=" .config; then bad+=("$line"); fi ;;
    esac
done < "$HERE/kernel.config"
[ ${#bad[@]} -eq 0 ] || die "kernel.config did not survive olddefconfig: ${bad[*]}"

# ---- 3. build the kernel, with every stamp fixed --------------------------------------------------
step "building the kernel ($(nproc) jobs)"
export KBUILD_BUILD_TIMESTAMP='1970-01-01 00:00:00 UTC' KBUILD_BUILD_USER=delulu KBUILD_BUILD_HOST=delulu KBUILD_BUILD_VERSION=1
make -s ARCH=x86_64 -j"$(nproc)" vmlinux
cp vmlinux "$OUT/vmlinux"
CONFIG_SHA256=$(sha256sum .config | cut -d' ' -f1)
cd "$ROOT"

# ---- 4. the guest: a static delulu with no Python and no network client ---------------------------
step "building the static guest ($GUEST_TARGET)"
rustup target list --installed 2>/dev/null | grep -qx "$GUEST_TARGET" || rustup target add "$GUEST_TARGET"
# musl-gcc hides every system header, the kernel's included, and libffi's build needs
# <linux/limits.h>: expose ONLY the kernel headers, through one directory of links.
KH="$CACHE/musl-kernel-headers"
mkdir -p "$KH"
ln -sfn /usr/include/linux "$KH/linux"
ln -sfn /usr/include/asm-generic "$KH/asm-generic"
ln -sfn /usr/include/x86_64-linux-gnu/asm "$KH/asm"
# A dependency links `-ldl`; musl's libc.a already defines the dl* functions, and the system's libdl.a
# is glibc's. An EMPTY archive searched first satisfies the flag without mixing the two C libraries.
# (Linking with musl-gcc instead built a binary with two sets of startup files that crashed at once.)
EL="$CACHE/musl-empty-libs"
mkdir -p "$EL"
rm -f "$EL/libdl.a"
ar rcs "$EL/libdl.a"
CARGO_HOME_DIR=${CARGO_HOME:-$HOME/.cargo}
CC_x86_64_unknown_linux_musl=musl-gcc \
CFLAGS_x86_64_unknown_linux_musl="-isystem $KH" \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-L native=$EL -C strip=symbols --remap-path-prefix=$ROOT=/delulu --remap-path-prefix=$CARGO_HOME_DIR=/cargo --remap-path-prefix=$TARGET_DIR=/target" \
CARGO_TARGET_DIR="$TARGET_DIR" \
    cargo build --release --locked -q -p delulu --no-default-features --target "$GUEST_TARGET"
GUEST="$TARGET_DIR/$GUEST_TARGET/release/delulu"
file "$GUEST" | grep -q "static" || ldd "$GUEST" 2>&1 | grep -q "statically linked\|not a dynamic" || die "the guest is not statically linked"
"$GUEST" --version >/dev/null || die "the guest binary does not run"

# ---- 5. the initramfs -----------------------------------------------------------------------------
step "writing the initramfs"
python3 "$HERE/mkinitramfs.py" "$GUEST" "$OUT/initramfs.cpio"

# ---- 6. the manifest ------------------------------------------------------------------------------
step "writing the manifest"
KERNEL_VERSION=$KERNEL_VERSION KERNEL_SHA256=$KERNEL_SHA256 CONFIG_SHA256=$CONFIG_SHA256 \
FRAGMENT_SHA256=$(sha256sum "$HERE/kernel.config" | cut -d' ' -f1) \
GUEST_VERSION=$("$GUEST" --version | awk '{print $2}') GUEST="$GUEST" OUT="$OUT" python3 - <<'PY'
import hashlib, json, os
def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()
out = os.environ["OUT"]
manifest = {
    "format": "delulu-microvm-image/1",
    "kernel": {
        "file": "vmlinux",
        "sha256": sha(os.path.join(out, "vmlinux")),
        "version": os.environ["KERNEL_VERSION"],
        "source_sha256": os.environ["KERNEL_SHA256"],
        "config_sha256": os.environ["CONFIG_SHA256"],
        "fragment_sha256": os.environ["FRAGMENT_SHA256"],
    },
    "initramfs": {"file": "initramfs.cpio", "sha256": sha(os.path.join(out, "initramfs.cpio"))},
    "guest": {"init": "/delulu", "sha256": sha(os.environ["GUEST"]), "delulu_version": os.environ["GUEST_VERSION"]},
}
with open(os.path.join(out, "manifest.json"), "w", newline="\n") as f:
    json.dump(manifest, f, indent=2, sort_keys=True)
    f.write("\n")
PY
cat "$OUT/manifest.json"
echo "build-image: done — $OUT"
