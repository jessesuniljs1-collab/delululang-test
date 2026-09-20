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

# P3 stdlib: several new List methods, run through the real binary rather than just checked.
cat > list_p3.delulu <<'EOF'
module listrun

fn main(root: Root) ! {Write} {
    let out = root.console()
    let xs = [3, 1, 2]
    let was_empty = xs.is_empty()
    let popped = xs.pop()
    let reversed = xs.reverse()
    let grown = reversed.concat([9, 8])
    let mid = grown.slice(1, 3)
    let has_nine = mid.contains(9)
    let sorted = mid.sort()
    let evens = mid.filter(fn(x: Int) -> Bool { x > 1 })
    let found = mid.find(fn(x: Int) -> Bool { x == 9 })
    let total = mid.fold(0, fn(acc: Int, x: Int) -> Int { acc + x })
    let names = ["a", "b", "c"]
    let joined = names.join(", ")
    out.println(joined)
    out.println(str(total))
}
EOF

# P3 stdlib: `Map`, iterated in ascending-by-key order (keys() returns Str keys sorted).
cat > map_p3.delulu <<'EOF'
module maprun

fn main(root: Root) ! {Write} {
    let out = root.console()
    let m = Map()
    m.insert("pear", 2)
    m.insert("apple", 1)
    m.insert("cherry", 3)
    let ks = m.keys()
    for k in ks {
        out.println(k)
    }
}
EOF

# P3 stdlib: `sort` on a `List[Float]` has no total order to sort by — refused (DL0401).
cat > sort_float.delulu <<'EOF'
module sortfloat

fn f(xs: List[Float]) -> List[Float] {
    xs.sort()
}
EOF

# P3 stdlib: a `Float` `Map` key has no total order either — refused (DL0401).
cat > map_float_key.delulu <<'EOF'
module mapfloatkey

fn f() -> Option[Str] {
    let m = Map()
    m.get(1.5)
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
echo "-- stdlib P3: List + Map --"
run 0 "run (List: is_empty/pop/reverse/concat/slice/contains/sort/filter/find/fold/join)" \
                                          "$DL" run list_p3.delulu --grant console

# Ascending-by-key order is the whole point of this case, so the exit code alone cannot cover it —
# check the actual printed order (`apple`, `cherry`, `pear`), not just that the run succeeded.
printf 'apple\ncherry\npear\n' > want.txt
"$DL" run map_p3.delulu --grant console >out.txt 2>err.txt
got=$?
if [ "$got" = 0 ] && diff -q want.txt out.txt >/dev/null 2>&1; then
    PASS=$((PASS + 1)); printf '  ok    %-46s exit %s\n' "run (Map: ascending-by-key order)" "$got"
else
    FAIL=$((FAIL + 1)); printf '  FAIL  %-46s exit %s (wanted 0, apple/cherry/pear)\n' "run (Map: ascending-by-key order)" "$got"
    sed 's/^/          /' out.txt err.txt | head -8
fi

run 1 "check (sort List[Float] -> DL0401 refused)"    "$DL" check sort_float.delulu
run 1 "check (Map Float key -> DL0401 refused)"        "$DL" check map_float_key.delulu

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

    ( cd pkgbin && "$DL" test . ) >out.txt 2>err.txt
    got=$?
    if [ "$got" = 0 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "test a generated package" "$got"
    else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 0)\n' "test a generated package" "$got"; sed 's/^/          /' err.txt | head -4; fi

    # Bare `delulu test` INSIDE a package targets the package (V2 P1-07, finding NE-09). It used to
    # refuse with "no ./tests directory here" although the scaffold's one test lives in `src/`, so
    # the first command a new user ran after `delulu new` failed. An earlier draft of this sweep
    # recorded that refusal as correct; it was the defect.
    ( cd pkgbin && "$DL" test ) >out.txt 2>err.txt
    got=$?
    if [ "$got" = 0 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "bare test inside a package" "$got"
    else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 0)\n' "bare test inside a package" "$got"; sed 's/^/          /' err.txt | head -4; fi

    # And OUTSIDE a package it still refuses cleanly (usage error), because there is nothing to
    # infer and guessing a directory would be worse than asking.
    ( "$DL" test ) >out.txt 2>err.txt
    got=$?
    if [ "$got" = 2 ]; then PASS=$((PASS+1)); printf '  ok    %-46s exit %s\n' "bare test outside a package refuses" "$got"
    else FAIL=$((FAIL+1)); printf '  FAIL  %-46s exit %s (wanted 2)\n' "bare test outside a package refuses" "$got"; fi
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
echo "-- the sandbox --"
# A state directory of the sweep's own. `run --sandbox` APPENDS sandbox-launch and sandbox-death
# records to the audit chain, and a sweep must not write into the developer's own chain — that is
# exactly how those records first reached `~/.delulu` and made `doctor` refuse its own machine.
DELULU_STATE_DIR="$WORK/state"
mkdir -p "$DELULU_STATE_DIR/audit"
export DELULU_STATE_DIR
# `probe` and `status` ATTEMPT things: they launch a guest under a jail and kill it. They must exit 0
# on every platform, because "this host cannot confine" is an answer rather than a failure.
run 0 "sandbox probe"                     "$DL" sandbox probe
run 0 "sandbox probe --json"               "$DL" sandbox probe --json
run 0 "sandbox status"                    "$DL" sandbox status
run 0 "sandbox status --json"              "$DL" sandbox status --json
# A verb `sandbox` does not have is refused, not guessed at as `probe`. `kill` is the one an operator
# will actually type, because PS-A-07 lists it and it is deliberately not built.
run 2 "sandbox kill (not a verb here)"     "$DL" sandbox kill
run 0 "explain E-SANDBOX"                  "$DL" explain E-SANDBOX
# P4a: the Agent Skill. A harness reads this; it must never fail to print.
run 0 "skill"                             "$DL" skill
run 0 "skill --json"                       "$DL" skill --json
run 2 "skill (takes no argument)"           "$DL" skill extra
run 0 "sandbox policy"                    "$DL" sandbox policy hello.delulu
run 0 "sandbox policy --json"              "$DL" sandbox policy --json hello.delulu
run 0 "run --sandbox"                     "$DL" run hello.delulu --sandbox --grant console
run 2 "run --mode without --sandbox"       "$DL" run hello.delulu --mode audit --grant console
unset DELULU_STATE_DIR

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
