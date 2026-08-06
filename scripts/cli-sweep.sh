#!/usr/bin/env sh
# CLI + compiler sweep — exercise every user-facing command and assert its exit code.
#
# Written for campaign P17-F. Before this, the "CLI + compiler sweep 21/21" recorded in
# CROSS_PLATFORM_VERIFICATION.md was performed by hand each time, which makes it exactly the kind
# of hand-maintained procedure this project's own design rule 1 says will drift. It is now a script
# so the claim is reproducible by anyone, on any platform, without trusting a transcript.
#
# Usage:  scripts/cli-sweep.sh [path-to-delulu-binary]
# Default binary: target/debug/delulu (or target/debug/delulu.exe on Windows).
#
# Exit code 0 iff every case matched its expected exit code.

set -u

ROOT=$(cd "$(dirname "$0")/.." && pwd)
DL=${1:-}
if [ -z "$DL" ]; then
    if [ -x "$ROOT/target/debug/delulu.exe" ]; then DL="$ROOT/target/debug/delulu.exe"
    else DL="$ROOT/target/debug/delulu"; fi
fi
if [ ! -x "$DL" ]; then
    echo "FATAL: no delulu binary at $DL (build with: cargo build)" >&2
    exit 2
fi

# Make the binary path ABSOLUTE before the `cd` below, or a relative one stops resolving the moment
# we move. `scripts/cli-sweep.sh ./target/release/delulu.exe` is the natural thing to type, and it
# produced eighteen "FAIL … exit 127" lines that read exactly like eighteen product defects.
case "$DL" in
    /*|[A-Za-z]:[/\\]*) ;;                       # already absolute (POSIX or Windows drive)
    *) DL="$(cd "$(dirname "$DL")" && pwd)/$(basename "$DL")" ;;
esac

WORK=$(mktemp -d 2>/dev/null || mktemp -d -t dlsweep)
trap 'rm -rf "$WORK"' EXIT
cd "$WORK" || exit 2

PASS=0
FAIL=0

# run <expected-exit> <label> <command...>
run() {
    want=$1; label=$2; shift 2
    "$@" >out.txt 2>err.txt
    got=$?
    if [ "$got" = "$want" ]; then
        PASS=$((PASS + 1))
        printf '  ok    %-46s exit %s\n' "$label" "$got"
    else
        FAIL=$((FAIL + 1))
        printf '  FAIL  %-46s exit %s (wanted %s)\n' "$label" "$got" "$want"
        sed 's/^/          /' err.txt | head -4
    fi
}

# ---- fixtures -----------------------------------------------------------------------------
cat > hello.delulu <<'EOF'
module hello

fn main(root: Root) ! {Write} {
    let out = root.console()
    out.println("Hello, Delulu")
}
EOF

# A program with an UNDECLARED effect — must be refused (DL0501), exit 1.
cat > bad.delulu <<'EOF'
module bad

fn main(root: Root) {
    let out = root.console()
    out.println("undeclared")
}
EOF

echo "CLI + compiler sweep"
echo "  binary: $DL"
echo "  workdir: $WORK"
echo

echo "-- compiler: check / authority / why --"
run 0 "check (clean program)"            "$DL" check hello.delulu
run 1 "check (undeclared effect)"        "$DL" check bad.delulu
run 0 "check --json"                     "$DL" check --json hello.delulu
run 0 "authority"                        "$DL" authority hello.delulu
run 0 "authority --json"                 "$DL" authority --json hello.delulu
run 0 "why Write"                        "$DL" why Write hello.delulu

echo
echo "-- compiler: run --"
run 0 "run (granted)"                    "$DL" run hello.delulu --grant console
run 1 "run (NOT granted -> DL0703)"      "$DL" run hello.delulu
run 0 "run --assert-trace"               "$DL" run hello.delulu --grant console --assert-trace

echo
echo "-- formatting --"
run 0 "fmt --check (canonical)"          "$DL" fmt --check hello.delulu
run 0 "fmt (rewrite in place)"           "$DL" fmt hello.delulu

echo
echo "-- packages --"
run 0 "new (binary package)"             "$DL" new pkgbin
run 0 "new --lib"                        "$DL" new pkglib --lib
# NOTE: these must run INSIDE the package dir, so they cannot use `run` directly (it would `cd`
# nowhere). Capture the real exit code in a subshell — an earlier draft passed `true` to `run`,
# which made both cases pass unconditionally. A check that cannot fail is not a check.
if [ -d pkgbin ]; then
    ( cd pkgbin && "$DL" check . ) >out.txt 2>err.txt
    got=$?
    if [ "$got" = 0 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "check a generated package" "$got"
    else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 0)\n' "check a generated package" "$got"; sed 's/^/          /' err.txt | head -4; fi

    # `delulu test .`, NOT bare `delulu test` — the bare form looks for a `./tests` directory and
    # correctly refuses without one. The dotted form is what the scaffold's own next-steps message
    # prints, and `new.rs` explains why. An earlier draft of this sweep used the bare form and
    # reported a defect that did not exist.
    ( cd pkgbin && "$DL" test . ) >out.txt 2>err.txt
    got=$?
    if [ "$got" = 0 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "test a generated package" "$got"
    else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 0)\n' "test a generated package" "$got"; sed 's/^/          /' err.txt | head -4; fi

    # The bare form MUST refuse cleanly (usage error), not crash.
    ( cd pkgbin && "$DL" test ) >out.txt 2>err.txt
    got=$?
    if [ "$got" = 2 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "bare test refuses (no ./tests)" "$got"
    else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 2)\n' "bare test refuses (no ./tests)" "$got"; fi
fi

echo
echo "-- diagnostics + help + completions --"
run 0 "explain DL0501"                   "$DL" explain DL0501
run 0 "--help"                           "$DL" --help
run 0 "completions bash"                 "$DL" completions bash
run 0 "completions powershell"           "$DL" completions powershell
run 2 "usage error (unknown command)"    "$DL" definitely-not-a-command
run 2 "typo gets a suggestion"           "$DL" chekc
run 0 "help <subcommand>"                "$DL" help check
run 2 "help <typo> refuses"              "$DL" help chekc

# The exit code alone cannot tell a suggestion from a hundred lines of usage — both exit 2. The
# point of the change was WHICH of the two you get, so the content is what has to be asserted.
"$DL" chekc >out.txt 2>err.txt
if grep -q 'did you mean `check`' err.txt; then
    PASS=$((PASS + 1)); printf '  ok    %-46s\n' "typo names the intended command"
else
    FAIL=$((FAIL + 1)); printf '  FAIL  %-46s\n' "typo names the intended command"
    sed 's/^/          /' err.txt | head -3
fi

# …and the other half: a word that is NOT nearly a command must get no guess. A confident wrong
# suggestion is followed, so declining is the behaviour under test here.
"$DL" package >out.txt 2>err.txt
if grep -q 'did you mean' err.txt; then
    FAIL=$((FAIL + 1)); printf '  FAIL  %-46s\n' "an unrelated word gets NO guess"
    sed 's/^/          /' err.txt | head -3
else
    PASS=$((PASS + 1)); printf '  ok    %-46s\n' "an unrelated word gets NO guess"
fi

echo
echo "-- repository tooling --"
# `doctor` inspects the checkout it is run FROM and takes no path argument.
( cd "$ROOT" && "$DL" doctor --check ) >out.txt 2>err.txt
got=$?
if [ "$got" = 0 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "doctor --check" "$got"
else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 0)\n' "doctor --check" "$got"; sed 's/^/          /' err.txt | head -4; fi

echo
if [ "$FAIL" -eq 0 ]; then
    echo "SWEEP OK — $PASS/$((PASS + FAIL)) cases, 0 problems"
    exit 0
else
    echo "SWEEP FAILED — $FAIL of $((PASS + FAIL)) cases did not match"
    exit 1
fi
