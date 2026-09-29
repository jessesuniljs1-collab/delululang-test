#!/usr/bin/env bash
#
# Type-check and lint `delulu`'s macOS and Windows code from Linux (routine run 3, 2026-09-29 — loop
# engineering; macOS first as `check-macos.sh`, Windows added the same run).
#
# The cloud VM is Linux and CI is the only macOS and Windows there are, so a mistake in their code used
# to cost a whole push run to find. `cargo check` and `cargo clippy` never link, so the C that
# dependencies compile in their build scripts (ring, zstd, ittapi) needs no Apple SDK or MSVC: a
# stand-in compiler that leaves empty objects, and a stand-in archiver that understands both `ar` and
# `lib.exe`'s `-out:`, are enough. `PYO3_CROSS_PYTHON_VERSION` lets pyo3 configure itself without the
# target's Python, and libffi-sys's build script — which runs `configure`, and `configure` knows no Rust
# triple — is replaced through its `links = "ffi"` key, as Cargo allows for any `links` crate. What this
# proves is that the Rust compiles and lints for those targets — NOT that it runs there: that is still
# the runner's (`witness.yml`, the push run). Nothing built here is ever executed.
#
# Linux on arm64 is checked too: its syscall table differs (no `fork`, other numbers), which is where a
# seccomp list that compiles on x86-64 can fail to (the arm64 CI job once caught exactly that).
#
# usage: scripts/check-other-os.sh [TARGET…]
#   default: aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc aarch64-unknown-linux-gnu
#   needs:   rustup target add aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc \
#              aarch64-unknown-linux-gnu
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
STUB="$ROOT/target/other-os-check-stub"
mkdir -p "$STUB"
cat > "$STUB/cc" <<'STUB_CC'
#!/bin/sh
# A stand-in C compiler: leaves an empty object at `-o`. For `cargo check` only — nothing links.
out=""
while [ $# -gt 0 ]; do
  case "$1" in -o) out="$2"; shift ;; -o*) out="${1#-o}" ;; esac
  shift
done
[ -n "$out" ] && : > "$out"
exit 0
STUB_CC
cat > "$STUB/ar" <<'STUB_AR'
#!/bin/sh
# A stand-in archiver: an empty archive wherever one is named, as `ar` or as `lib.exe -out:`.
for a in "$@"; do
  case "$a" in
    -out:*|-OUT:*|/out:*|/OUT:*) printf '!<arch>\n' > "${a#*:}" ;;
    *.a|*.lib) printf '!<arch>\n' > "$a" ;;
  esac
done
exit 0
STUB_AR
chmod +x "$STUB/cc" "$STUB/ar"

TARGETS=${*:-"aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc aarch64-unknown-linux-gnu"}
for target in $TARGETS; do
  var=$(echo "$target" | tr '-' '_')
  echo "== $target: cargo clippy -p delulu --all-targets -- -D warnings"
  env "CC_$var=$STUB/cc" "CXX_$var=$STUB/cc" "AR_$var=$STUB/ar" PYO3_CROSS_PYTHON_VERSION=3.13 \
    cargo clippy -q -p delulu --all-targets --target "$target" \
      --config "target.$target.ffi.rustc-link-lib=[\"ffi\"]" -- -D warnings
done
echo "ok: delulu type-checks and lints clean for $TARGETS — compiled, not run"
