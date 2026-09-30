#!/usr/bin/env bash
# Install a pinned OpenShell release on a CI runner, checked against the release's own SHA-256 list
# (PS-E-05, routine run 8). Used by `.github/workflows/openshell.yml` only: the VM cannot reach another
# repository's release assets. Nothing it downloads is kept, cached or committed (D-V2-52).
#
# Usage: scripts/openshell-install.sh TAG   (e.g. v0.1.2; the Debian package carries the CLI, the gateway
# and `openshell-prover` — routine run 8 read `openshell_0.1.2-1_amd64.deb`, 74.5 MB)
set -euo pipefail
TAG="${1:?usage: openshell-install.sh TAG}"
base="https://github.com/NVIDIA/OpenShell/releases/download/$TAG"
dir="$(mktemp -d)"
cd "$dir"
curl -fsSL -o sums.txt "$base/openshell-checksums-sha256.txt"
echo "== the release's assets ($TAG)"
awk '{ n = $2; sub(/^\*/, "", n); print n }' sums.txt
arch="$(dpkg --print-architecture)"
# Every Debian package for this architecture: the CLI's, and the prover's if it ships apart.
mapfile -t debs < <(awk -v a="$arch" '{ n = $2; sub(/^\*/, "", n); if (n ~ ("^openshell.*[-_]" a "[.]deb$")) print n }' sums.txt)
[ "${#debs[@]}" -gt 0 ] || { echo "no Debian package for $arch in $TAG"; exit 1; }
pick=()
for d in "${debs[@]}"; do
  case "$d" in *prover*) pick+=("$d") ;; esac
done
[ "${#pick[@]}" -gt 0 ] || pick=("${debs[0]}")
for d in "${pick[@]}"; do
  curl -fsSL -o "$d" "$base/$d"
  want="$(awk -v n="$d" '{ m = $2; sub(/^\*/, "", m); if (m == n) { print $1; exit } }' sums.txt)"
  [ -n "$want" ] || { echo "no checksum for $d"; exit 1; }
  echo "$want  $d" | sha256sum -c -
done
sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y "${pick[@]/#/./}"
cd /
rm -rf "$dir"
command -v openshell openshell-prover
openshell-prover --version || true
openshell --version || true
