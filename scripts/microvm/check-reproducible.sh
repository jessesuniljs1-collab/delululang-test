#!/usr/bin/env bash
#
# PS-C-02's verification: build the guest image TWICE, from clean, in different directories, and
# require every hash to match — the kernel, the initramfs, and the guest binary inside it.
#
# "From clean" is the point. A second build that reused the first one's objects would match whether or
# not the build is reproducible, so each build gets its own kernel work directory and its own cargo
# target directory, and the two directories are named differently so a path that leaked into an
# output would show as a difference.
#
# usage: scripts/microvm/check-reproducible.sh [scratch-dir]     (default: a fresh mktemp directory)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
SCRATCH=${1:-$(mktemp -d)}
mkdir -p "$SCRATCH"
for side in first second; do
    echo "== build: $side"
    DELULU_MICROVM_WORK="$SCRATCH/work-$side" CARGO_TARGET_DIR="$SCRATCH/target-$side" \
        "$ROOT/scripts/microvm/build-image.sh" "$SCRATCH/image-$side" > "$SCRATCH/build-$side.log" 2>&1 \
        || { tail -30 "$SCRATCH/build-$side.log"; echo "check-reproducible: the $side build failed"; exit 1; }
done
python3 - "$SCRATCH/image-first/manifest.json" "$SCRATCH/image-second/manifest.json" <<'PY'
import json, sys
a, b = (json.load(open(p)) for p in sys.argv[1:3])
rows = [("kernel", a["kernel"]["sha256"], b["kernel"]["sha256"]),
        ("initramfs", a["initramfs"]["sha256"], b["initramfs"]["sha256"]),
        ("guest binary", a["guest"]["sha256"], b["guest"]["sha256"]),
        ("kernel config", a["kernel"]["config_sha256"], b["kernel"]["config_sha256"])]
bad = [r for r in rows if r[1] != r[2]]
for name, x, y in rows:
    print(f"{'same' if x == y else 'DIFFERENT':9} {name:13} {x}" + ("" if x == y else f"\n{'':23}{y}"))
if bad:
    sys.exit("check-reproducible: the two builds differ — the image is not reproducible on this machine")
print("check-reproducible: two clean builds produced identical bytes")
PY
