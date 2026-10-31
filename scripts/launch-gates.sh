#!/usr/bin/env bash
#
# Release procedure step 1's gates on the launch tree, in one command (on-demand routine run 11, loop engineering).
# Each merge of `master` into `launch/2.0.0` (docs/CLOUD_ROUTINE.md step 4.3) ran the same dozen commands by hand — runs
# 8, 9 and 11 — and read `survey findings`' error count by eye, though that command exits 0 whatever it finds. This runs
# them in order, counts the findings' errors itself, says whether the merged tree's Rust is BASE's (only then do BASE's
# push runs on three systems already cover its code), and ends with one line per gate and `LAUNCH-GATES=ok` or
# `LAUNCH-GATES=failed`. It exits 0 only when every gate ran and passed: a missing `cargo-deny` is a failure, never a skip.
#
# usage (inside the launch worktree, after the merge's LAST edit and before its commit; any checkout's copy will do):
#   scripts/launch-gates.sh [BASE]        # BASE: the ref merged in, default origin/master
#   LAUNCH_GATES_ONLY="survey-findings cargo-deny" scripts/launch-gates.sh   # those gates alone, ending
#                                         # `LAUNCH-GATES=partial-ok` — never `ok`, which only a whole run says
# It writes what `survey build` writes (the map), `clippy.sh`'s stamp, and cargo's target — nothing else.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
base=${1:-origin/master}
branch=$(git rev-parse --abbrev-ref HEAD)
case "$branch" in
    launch/*) ;;
    *) echo "launch-gates.sh: HEAD is \`$branch\`, not a launch/* branch — run it in the launch worktree" >&2; exit 2 ;;
esac
git rev-parse --verify -q "$base^{commit}" >/dev/null || { echo "launch-gates.sh: no such ref \`$base\`" >&2; exit 2; }
jobs=${LAUNCH_GATES_JOBS:-4}
only=${LAUNCH_GATES_ONLY:-}

results=()
fail=0
gate() {
    local name=$1
    shift
    if [ -n "$only" ] && [[ " $only " != *" $name "* ]]; then
        results+=("$(printf '%-18s not run (LAUNCH_GATES_ONLY)' "$name")")
        return
    fi
    echo "== $name: $*"
    "$@"
    local code=$?
    results+=("$(printf '%-18s exit %s' "$name" "$code")")
    [ "$code" -eq 0 ] || fail=1
}

# `survey findings` prints `error: …` / `warning: …` lines and exits 0 either way; the routine's bar is 0 errors.
findings_without_errors() {
    local out code errors warnings
    out=$(cargo run -q -p delulu-survey -- findings)
    code=$?
    [ "$code" -eq 0 ] || return "$code"
    errors=$(printf '%s\n' "$out" | grep -c '^error:')
    warnings=$(printf '%s\n' "$out" | grep -c '^warning:')
    printf '%s\n' "$out" | grep '^error:'
    echo "survey findings: $errors error(s), $warnings warning(s)"
    [ "$errors" -eq 0 ]
}

# The merged tree's Rust against BASE's: a note, not a gate — a launch tree may differ, but then BASE's push runs do not
# cover it and the `everything` run dispatched at it is its only suite.
rust_note() {
    local differ
    differ=$(git diff --name-only "$base" -- crates fuzz ':(glob)**/*.rs')
    if [ -z "$differ" ]; then
        echo "note: the merged tree's Rust under crates/ and fuzz/ is $base's ($(git rev-parse --short "$base"))"
    else
        echo "note: the merged tree's Rust DIFFERS from $base's in:"
        printf '  %s\n' $differ
        echo "  — $base's push runs do not cover these; the launch tree's own \`everything\` run is their suite"
    fi
}

# Step 4.3 re-measures CHECKLIST-2.0's criterion 2 on the merged tree and writes it into that row; runs 8, 9, 11 and 13
# wrote it by hand and nothing read it back (on-demand routine run 13). The row's newest citation at the anchor count
# measured now ("`HASH`: **N/N (100%)**") must be BASE, or a commit whose criterion-2 inputs are BASE's — a records-only
# merge (the citations merge after a run's closing commit) needs no new line. Write the row first, then run the gates.
criterion_2_inputs=(crates tests examples docs/reference docs/book/samples Cargo.toml Cargo.lock)
criterion_2_cited() {
    local measured row cited
    measured=$(cargo run -q -p delulu-conform -j "$jobs" -- --coverage \
        | sed -n 's/^anchors: \([0-9]*\) | covered (pos+neg): \([0-9]*\) .*/\2\/\1/p')
    [ -n "$measured" ] || { echo "could not read delulu-conform --coverage's anchor count"; return 1; }
    row=$(grep '^| 2 |' docs/release/CHECKLIST-2.0.md)
    cited=$(printf '%s\n' "$row" | grep -o "\`[0-9a-f]\{7,40\}\`: \*\*$measured (100%)\*\*" | grep -o '[0-9a-f]\{7,40\}' | tail -1)
    if [ -z "$cited" ]; then
        echo "CHECKLIST-2.0 criterion 2 cites no commit at **$measured (100%)** — re-measure it on this tree and write it"
        return 1
    fi
    if ! git cat-file -e "$cited^{commit}" 2>/dev/null; then
        echo "CHECKLIST-2.0 criterion 2's newest citation \`$cited\` is not a commit here (a shallow clone? \`git fetch --unshallow\`)"
        return 1
    fi
    if git diff --quiet "$cited" "$base" -- "${criterion_2_inputs[@]}"; then
        echo "CHECKLIST-2.0 criterion 2 cites $cited at $measured; criterion 2's inputs are the same at $(git rev-parse --short=7 "$base")"
    else
        echo "CHECKLIST-2.0 criterion 2's newest citation \`$cited\` (at $measured) predates a change to criterion 2's inputs:"
        git diff --name-only "$cited" "$base" -- "${criterion_2_inputs[@]}" | sed 's/^/  /' | head -10
        echo "  — re-measure it on this tree and write it, citing \`$(git rev-parse --short=7 "$base")\`"
        return 1
    fi
}

cargo_deny() {
    if ! command -v cargo-deny >/dev/null 2>&1; then
        echo "cargo-deny is not installed (\`cargo install cargo-deny --locked\`, about four minutes) — criterion 4 unchecked"
        return 127
    fi
    cargo deny --all-features check advisories && cargo deny --all-features check bans licenses sources
}

gate survey-build cargo run -q -p delulu-survey -- build
gate survey-check cargo run -q -p delulu-survey -- check
gate survey-findings findings_without_errors
gate clippy scripts/clippy.sh -j "$jobs"
gate doctor cargo run -q -p delulu -j "$jobs" -- doctor --check
gate document-gates cargo test -q -p delulu -j "$jobs" --test book --test core_invariance --test distribution \
    --test doctor_cli --test editor_contract --test evidence_claims --test governance --test release \
    --test repository_structure
gate survey-suite cargo test -q -p delulu-survey -j "$jobs"
gate conform-coverage cargo run -q -p delulu-conform -j "$jobs" -- --coverage
gate check-reference cargo run -q -p delulu-conform -j "$jobs" -- --check-reference
gate fmt-samples cargo run -q -p delulu -j "$jobs" -- fmt --check examples docs/book/samples
gate criterion-2-cited criterion_2_cited
gate cargo-deny cargo_deny

echo
rust_note
echo "== launch gates on $branch at $(git rev-parse --short HEAD) (merging $base $(git rev-parse --short "$base")):"
printf '  %s\n' "${results[@]}"
if [ "$fail" -eq 0 ] && [ -n "$only" ]; then
    echo "LAUNCH-GATES=partial-ok"
elif [ "$fail" -eq 0 ]; then
    echo "LAUNCH-GATES=ok"
else
    echo "LAUNCH-GATES=failed"
fi
exit "$fail"
