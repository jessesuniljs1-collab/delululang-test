#!/usr/bin/env bash
# The LTS-cycle drill (Stage 10 §4, criterion 5) — reproducible from a clean checkout.
#
#   bash measurements/lts-cycle/run-demo.sh
#
# Exercises the WHOLE advisory-feed mechanism end to end, with the registry as the source of truth:
#
#   1. a package `netlib` is released at 1.2.0;
#   2. a vulnerability is found; an advisory is FILED on the registry (`delulu-registry advisory
#      file`) against netlib 1.2.0, naming 1.2.1 as the fix;
#   3. the feed is exported to a local file (`delulu-registry advisory export`) — the same offline
#      copy a build reads, so the registry being down never breaks or silences a build;
#   4. `delulu build` on netlib 1.2.0 WARNS (DL1903) — information, the build still succeeds;
#   5. `delulu build --deny-advisories` on netlib 1.2.0 FAILS — the CI gate catches a
#      known-vulnerable version;
#   6. the backported fix ships as netlib 1.2.1;
#   7. `delulu build --deny-advisories` on netlib 1.2.1 is CLEAN — the cycle closes.
#
# Everything here is hermetic: a scratch registry on disk (no network, no server process needed —
# `advisory file`/`export` write and read the registry directly, exactly as `issue-token` does),
# and a scratch package. No real CVE, CNA, or multi-month LTS window is involved; see RECORD.md for
# the honest boundary between this mechanism drill and the calendar-time cycle criterion 5 names.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="${TMPDIR:-/tmp}/delulu-lts-cycle-$$"
mkdir -p "$out"

# Build the binaries here rather than trusting one that may already exist (the robotics-demo lesson:
# a script that measures a stale binary prints confident nonsense).
echo "building (cargo build --release -p delulu -p delulu-registry) ..."
( cd "$root" && cargo build --release -p delulu -p delulu-registry ) || { echo "build failed" >&2; exit 1; }
DELULU="$root/target/release/delulu";           [ -x "$DELULU" ]   || DELULU="$root/target/release/delulu.exe"
REG="$root/target/release/delulu-registry";      [ -x "$REG" ]      || REG="$root/target/release/delulu-registry.exe"
for b in "$DELULU" "$REG"; do
  [ -x "$b" ] || { echo "built, but no binary at $b" >&2; exit 1; }
done
export DELULU_NO_FIRST_RUN=1

regdata="$out/registry-data"
pkg="$out/netlib"
feed="$pkg/delulu.advisories.json"
mkdir -p "$pkg/src"

# The library under an LTS train, released at 1.2.0. A pure package — no effects — so the build's
# only remark is the advisory, never anything else.
write_netlib() { # $1 = version
  cat > "$pkg/delulu.toml" <<EOF
[package]
name = "netlib"
version = "$1"

[authority]
effects = []
EOF
  cat > "$pkg/src/main.delulu" <<'EOF'
module netlib
fn main(root: Root) {
}
EOF
}

pass=0; fail=0
check() { # $1 = label, $2 = expected exit (0|nonzero), $3 = actual exit, $4 = must-contain (or "")
  local label="$1" want="$2" got="$3" needle="${4:-}"
  local ok=1
  if [ "$want" = "0" ]; then [ "$got" -eq 0 ] || ok=0; else [ "$got" -ne 0 ] || ok=0; fi
  if [ -n "$needle" ] && ! grep -q "$needle" "$out/last.txt"; then ok=0; fi
  if [ "$ok" -eq 1 ]; then echo "  PASS  $label"; pass=$((pass+1));
  else echo "  FAIL  $label (exit=$got, wanted $want${needle:+, needle '$needle'})"; fail=$((fail+1)); fi
}
run() { "$@" > "$out/last.txt" 2>&1; return $?; }

echo
echo "== step 1-2: release netlib 1.2.0, then file the advisory on the registry =="
write_netlib "1.2.0"
tok="$("$REG" --root "$regdata" issue-token --owner ops --scope netlib)"
run "$REG" --root "$regdata" advisory file --token "$tok" --package netlib \
    --id DLSA-2026-0007 --affected 1.2.0 --patched 1.2.1 --severity high \
    --summary "header-folding request smuggling"
check "advisory filed on the registry" 0 $? "filed advisory"

echo
echo "== step 3: export the registry feed to the local file a build reads =="
run "$REG" --root "$regdata" advisory export --out "$feed"
check "feed exported to $feed" 0 $? "advisory"
[ -f "$feed" ] && echo "     feed: $(tr -d '\n' < "$feed" | sed 's/  */ /g')"

echo
echo "== step 4: plain build on 1.2.0 — a WARNING, the build still succeeds =="
run "$DELULU" build "$pkg"
check "delulu build warns DL1903 and exits 0" 0 $? "DL1903"

echo
echo "== step 5: the CI gate on 1.2.0 — --deny-advisories FAILS the build =="
run "$DELULU" build "$pkg" --deny-advisories
check "delulu build --deny-advisories fails on the advised version" nonzero $? "DL1903"

echo
echo "== step 6-7: the backport ships as 1.2.1; the gate is now CLEAN =="
write_netlib "1.2.1"
run "$DELULU" build "$pkg" --deny-advisories
check "delulu build --deny-advisories passes on the patched version" 0 $?

echo
echo "== skip-branch check: the gate refuses when it cannot find its evidence =="
rm -f "$feed"
run "$DELULU" build "$pkg" --deny-advisories
check "no feed + --deny-advisories refuses rather than passing" nonzero $? "no advisory feed"

echo
echo "-------------------------------------------------"
echo "  $pass passed, $fail failed"
echo "-------------------------------------------------"
rm -rf "$out"
[ "$fail" -eq 0 ]
