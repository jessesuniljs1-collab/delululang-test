#!/usr/bin/env bash
#
# Run the whole suite the routine's way — alone, `-j 4`, no incremental cache (it fills the VM's disk) — and
# say whether the working tree moved while it ran (routine run 13, loop engineering).
#
# Three routine runs edited the tree during a suite (run 12 wrote a script into `scripts/`, run 13 reworked a
# test file), and each lost `doctor_cli`'s and the Survey's freshness tests to a stale map for that reason
# alone — failures that read like product defects. The rule was written down each time; this makes it
# visible instead: the tree's state is fingerprinted before and after, and a change is reported on the last
# line, beside cargo's own exit code (never a pipeline's).
#
# usage: scripts/suite.sh OUTFILE [extra cargo test arguments…]
#   OUTFILE ends with `EXIT=<cargo's code>` and, when the tree changed, a `TREE-MOVED:` line.
#   Wait on it with: until grep -q '^EXIT=' OUTFILE; do sleep 15; done
set -uo pipefail
out=${1:?usage: scripts/suite.sh OUTFILE [cargo test args…]}
shift
fingerprint() {
    # Tracked changes, untracked files and their contents: what the map and the document gates read.
    { git status --porcelain=v1 -uall; git diff; git ls-files --others --exclude-standard -z | xargs -0 -r sha256sum; } \
        | sha256sum
}
before=$(fingerprint)
CARGO_INCREMENTAL=0 cargo test --workspace --no-fail-fast -j 4 "$@" > "$out" 2>&1
code=$?
after=$(fingerprint)
echo "EXIT=$code" >> "$out"
if [ "$before" != "$after" ]; then
    echo "TREE-MOVED: the working tree changed while the suite ran — a stale map fails doctor_cli and the" \
         "Survey's freshness test for that reason alone; regenerate the map and re-run them (or the suite)" >> "$out"
fi
exit "$code"
