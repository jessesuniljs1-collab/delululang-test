#!/usr/bin/env bash
#
# Package the DeluluLang toolchain into a self-contained archive that a person or a machine can
# download, unpack, and run WITHOUT a Rust toolchain and without this repository.
#
# WHY THE PORTABLE BUILD IS `--no-default-features`, and this is not a preference:
#
#   A default `cargo build --release` embeds CPython through pyo3, and the resulting binary imports
#   a SPECIFIC interpreter — `python313.dll` on the machine this was written on. Anyone without that
#   exact version cannot start it. Measured, not assumed: the default binary carries that import
#   string and the `--no-default-features` binary carries none.
#
#   So the shipped artifact is the Python-less one. `root.python(...)` in it returns
#   `ForeignErr::Unavailable` (DL1307) — the same user-visible behaviour as a machine with no
#   interpreter, which is honest rather than a crash. A Python-capable build is a documented
#   from-source option, never the download.
#
#   `--no-default-features` also drops the network client (PS-B-02, D-V2-30), which has no
#   portability problem at all — it is pure Rust over rustls. So the build names it back:
#   `--features net`. Without that word the download would answer every `http.get` with
#   `Err(Refused)`; `delulu doctor` in the archive says which it got, and `tests/distribution.rs`
#   pins this line.
#
# WHAT THIS IS NOT: publication. `cargo install delulu` from crates.io cannot work and is not meant
# to — every crate but the CLI is `publish = false`, because `STABILITY.md` §2 promises the internal
# crates are NOT a stable API. Publishing them to make one command shorter would trade a promise for
# a convenience. This archive is how the toolchain travels instead.
#
# usage: scripts/package-toolchain.sh [output-dir]     (default: dist/)
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${1:-$ROOT/dist}
cd "$ROOT"

VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
HOST=$(rustc -vV | awk '/^host:/ {print $2}')
EXE=""
case "$HOST" in *windows*) EXE=".exe" ;; esac

NAME="delulu-$VERSION-$HOST"
STAGE="$OUT/$NAME"

echo "packaging  $NAME"
echo "  portable build: cargo build --release -p delulu --no-default-features --features net"
cargo build --release -p delulu --no-default-features --features net

# Respect CARGO_TARGET_DIR. This project's own Linux instructions REQUIRE it — a build inside the
# Windows working tree dies in libffi-sys on DrvFs — so a packaging script that assumes `target/`
# fails on exactly the configuration the documentation tells people to use.
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
BIN="$TARGET_DIR/release/delulu$EXE"
test -x "$BIN" || { echo "no binary at $BIN (CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-unset})"; exit 1; }

# Clear the CONTENTS rather than the directory. On Windows an output directory that a shell is
# sitting in, or that the indexer has open, cannot be removed — and failing a whole packaging run
# because something is looking at the output folder is brittle for no benefit.
rm -rf "${STAGE:?}"/* "${STAGE:?}"/.[!.]* 2>/dev/null || true
mkdir -p "$STAGE/bin" "$STAGE/examples"
cp "$BIN" "$STAGE/bin/"

# Everything a recipient is legally and practically owed. A binary with no LICENSE beside it is not
# a distribution anyone should accept.
for f in LICENSE NOTICE TRADEMARK.md README.md CHANGELOG.md SECURITY.md; do
  test -f "$f" && cp "$f" "$STAGE/"
done
cp examples/*.delulu "$STAGE/examples/" 2>/dev/null || true
# The package examples too, so `delulu run <dir>` — how real programs are shipped — can be tried
# from the archive rather than only read about.
for d in examples/*/; do
  test -f "$d/delulu.toml" && cp -r "$d" "$STAGE/examples/" 2>/dev/null || true
done
# P4a (D-V2-28): the Agent Skill, in the layout a harness expects (`skills/<name>/SKILL.md`). Most
# recipients of this archive are not people — and the binary can print the same bytes with
# `delulu skill`, so a harness with only the binary is not stuck either.
mkdir -p "$STAGE/skills/delulu"
cp skills/delulu/SKILL.md "$STAGE/skills/delulu/" 2>/dev/null || true

cat > "$STAGE/INSTALL.txt" <<TXT
DeluluLang $VERSION — $HOST

  1. Put bin/delulu$EXE somewhere on your PATH.
  2. Check it:            delulu --version
  3. Check a program:     delulu check examples/hello_wasm.delulu
  4. See what it may do:  delulu authority examples/hello_wasm.delulu
  5. Run it, refused:     delulu run examples/hello_wasm.delulu
  6. Run it, granted:     delulu run examples/hello_wasm.delulu --grant console

Step 5 is meant to fail. It exits 1 with DL0703 and names the exact flag that
would allow it. That refusal is the language working, not a misconfiguration —
a program gets no authority it was not handed.

A program asks for exactly what it needs, so a bigger one needs more. The
richer example wants three grants, and refuses one at a time until it has them:

  delulu run examples/demo.delulu \\
      --grant console --grant fs.read=./config --grant secret:API_KEY=demo

Run 'delulu authority <file> --grants' first and the required grants are the
list it prints, one '--grant' flag per line, ready to paste. An UPPERCASE word
in one of them is a placeholder the program does not name in its own source
(a secret's value, a path it takes at runtime) and you choose it.
Package examples are directories: 'delulu run examples/greeter --grant console'.

This build has no embedded Python: root.python(...) returns DL1307
(unavailable), exactly as on a machine with no interpreter. Build from source
without --no-default-features if you need it, and note that such a build then
requires that specific CPython version at runtime.

This build HAS the network client: http.get fetches over verified HTTPS from
the hosts a '--grant net=HOST' names, checking certificates against this
machine's own trust store. 'delulu doctor' says how many trust roots it found.

Verify what you downloaded before trusting it:
  sha256sum -c SHA256SUMS      (Linux/macOS)
  Get-FileHash bin\delulu.exe  (Windows PowerShell)
TXT

# Checksums over everything shipped, computed from the staged tree so the manifest describes exactly
# what is in the archive rather than what was intended to be.
( cd "$STAGE" && find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum > SHA256SUMS )

( cd "$OUT" && tar -czf "$NAME.tar.gz" "$NAME" )
( cd "$OUT" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256" )

echo
echo "  staged  $STAGE"
echo "  archive $OUT/$NAME.tar.gz"
echo "  files   $(find "$STAGE" -type f | wc -l)"
echo "PACKAGE-DONE"
