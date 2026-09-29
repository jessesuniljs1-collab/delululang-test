#!/usr/bin/env bash
#
# Type-check and lint `delulu`'s macOS code from Linux (routine run 3, 2026-09-29 — loop engineering).
#
# The cloud VM is Linux and CI is the only macOS there is, so a macOS-only mistake used to cost a whole
# push run to find. `cargo check` and `cargo clippy` never link, so the macOS C that dependencies
# compile in their build scripts (ring, zstd) needs no Apple SDK: a stand-in compiler that leaves an
# empty object is enough, and `PYO3_CROSS_PYTHON_VERSION` lets pyo3 configure itself without a macOS
# Python. What this proves is that the Rust compiles and lints for macOS — NOT that it runs there:
# that is still the runner's (`witness.yml`, the push run). Nothing built here is ever executed.
#
# Windows cannot be checked this way yet: libffi-sys's build script runs `configure` for an msvc
# target from Linux, and fails.
#
# usage: scripts/check-macos.sh [aarch64|x86_64]   (default: both)
#   needs: rustup target add aarch64-apple-darwin x86_64-apple-darwin
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
STUB="$ROOT/target/macos-check-stub"
mkdir -p "$STUB"
cat > "$STUB/cc" <<'EOF'
#!/bin/sh
# A stand-in C compiler: leaves an empty object at `-o`. For `cargo check` only — nothing links.
out=""
while [ $# -gt 0 ]; do
  case "$1" in -o) out="$2"; shift ;; -o*) out="${1#-o}" ;; esac
  shift
done
[ -n "$out" ] && : > "$out"
exit 0
EOF
cat > "$STUB/ar" <<'EOF'
#!/bin/sh
# A stand-in archiver: an empty archive wherever one is named.
for a in "$@"; do case "$a" in *.a) printf '!<arch>\n' > "$a" ;; esac; done
exit 0
EOF
chmod +x "$STUB/cc" "$STUB/ar"

ARCHES=${1:-"aarch64 x86_64"}
for arch in $ARCHES; do
  target="$arch-apple-darwin"
  var=$(echo "$target" | tr '-' '_')
  echo "== $target: cargo clippy -p delulu --all-targets -- -D warnings"
  env "CC_$var=$STUB/cc" "CXX_$var=$STUB/cc" "AR_$var=$STUB/ar" PYO3_CROSS_PYTHON_VERSION=3.13 \
    cargo clippy -q -p delulu --all-targets --target "$target" -- -D warnings
done
echo "ok: delulu type-checks and lints clean for macOS ($ARCHES) — compiled, not run"
