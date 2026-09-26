#!/usr/bin/env bash
#
# Fetch the Firecracker release the microVM launcher was tested against (D-V2-39), check it against
# the pinned sha256, and install `firecracker` and `jailer` into a directory.
#
# The pin is the release archive's sha256 as GitHub publishes it for the release asset; a download that
# does not match is deleted and refused. Firecracker is Apache-2.0 and is not redistributed by this
# repository: this script fetches it from its own release page on the machine that uses it.
#
# usage: scripts/microvm/fetch-firecracker.sh [install-dir]     (default: ~/.local/bin)
#   then: export DELULU_FIRECRACKER=<install-dir>/firecracker
set -euo pipefail
VERSION=v1.17.0
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) SHA256=06094a1108ae9e82aa4c23a775aa92758f53f1175d422270d9d6162cb9ade558 ;;
    aarch64) SHA256=e351ebe4f7a16b5873bbd51005d2e6767103cff4d5ebc829df2d3f95a93e2256 ;;
    *) echo "fetch-firecracker: no Firecracker release for $ARCH" >&2; exit 1 ;;
esac
DEST=${1:-$HOME/.local/bin}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
curl -fsSL -o "$TMP/fc.tgz" "https://github.com/firecracker-microvm/firecracker/releases/download/${VERSION}/firecracker-${VERSION}-${ARCH}.tgz"
got=$(sha256sum "$TMP/fc.tgz" | cut -d' ' -f1)
if [ "$got" != "$SHA256" ]; then
    echo "fetch-firecracker: firecracker-${VERSION}-${ARCH}.tgz has sha256 $got, not the pinned $SHA256 — refused" >&2
    exit 1
fi
tar -xzf "$TMP/fc.tgz" -C "$TMP"
mkdir -p "$DEST"
install -m 0755 "$TMP/release-${VERSION}-${ARCH}/firecracker-${VERSION}-${ARCH}" "$DEST/firecracker"
install -m 0755 "$TMP/release-${VERSION}-${ARCH}/jailer-${VERSION}-${ARCH}" "$DEST/jailer"
"$DEST/firecracker" --version | head -1
echo "fetch-firecracker: installed into $DEST — export DELULU_FIRECRACKER=$DEST/firecracker"
