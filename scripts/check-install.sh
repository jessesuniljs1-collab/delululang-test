#!/usr/bin/env bash
#
# P5-05's gate: install a real toolchain archive EXACTLY as `INSTALL.md` §1 tells a person to, and run
# the first commands it lists. The commands are not copied into this script — they are read out of
# `INSTALL.md`, from the one fenced block whose info string says `install-gate`, so the page and the
# gate cannot drift apart: change a command on the page and this runs the new one.
#
# usage: scripts/check-install.sh <delulu-VERSION-TARGET.tar.gz>
#   (its `.sha256` must sit beside it, as `package-toolchain.sh` and the release workflow leave it)
set -euo pipefail
ARCHIVE=${1:?usage: check-install.sh <delulu-VERSION-TARGET.tar.gz>}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
case "$ARCHIVE" in /*|[A-Za-z]:*) ;; *) ARCHIVE="$PWD/$ARCHIVE" ;; esac
[ -f "$ARCHIVE" ] && [ -f "$ARCHIVE.sha256" ] || { echo "check-install: need $ARCHIVE and $ARCHIVE.sha256" >&2; exit 2; }
STEM=$(basename "$ARCHIVE" .tar.gz)

# The block, from the page. Exactly one: two would leave it unclear which the page means.
BLOCKS=$(grep -c '^```sh install-gate$' "$ROOT/INSTALL.md" || true)
[ "$BLOCKS" = 1 ] || { echo "check-install: INSTALL.md must have exactly one \`\`\`sh install-gate block (it has $BLOCKS)" >&2; exit 2; }
BLOCK=$(awk '/^```sh install-gate$/ {on=1; next} /^```$/ && on {exit} on' "$ROOT/INSTALL.md")
[ -n "$BLOCK" ] || { echo "check-install: the install-gate block in INSTALL.md is empty" >&2; exit 2; }
# The page writes the archive's name the way a reader sees it on the release page.
BLOCK=${BLOCK//delulu-<version>-<target>/$STEM}

W=$(mktemp -d)
trap 'rm -rf "$W"' EXIT
cp "$ARCHIVE" "$ARCHIVE.sha256" "$W/"
cd "$W"
# A fresh user: no state and no home of their own yet. Nothing else is switched off — the first-run
# experience stays what a person gets, which on a non-interactive stdin is no prompt at all.
export DELULU_STATE_DIR="$W/state" HOME="$W/home"
mkdir -p "$HOME"
# macOS has `shasum -a 256` and may not have `sha256sum`; the page says so, and the gate runs what a
# Mac user would.
if ! command -v sha256sum >/dev/null; then sha256sum() { shasum -a 256 "$@"; }; fi
echo "== the install-gate block from INSTALL.md, run as written:"
printf '%s\n' "$BLOCK" | sed 's/^/   | /'
# The block's own commands say whether they succeeded: `-e` stops at the first one that did not.
set -x
eval "$BLOCK"
set +x
echo "check-install: every step of INSTALL.md's install gate ran and succeeded"
