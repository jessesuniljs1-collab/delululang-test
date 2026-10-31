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
#   It passes `--workspace --no-fail-fast -j 4` itself: a second `--no-fail-fast` stops cargo before a test runs.
#   OUTFILE ends with `EXIT=<cargo's code>` and, when the tree changed, a `TREE-MOVED:` line. Below
#   SUITE_MIN_FREE_GB (default 10) of free disk, or on a stale Survey map, it does not start: `EXIT=3` and why.
#   Wait on it with: until grep -q '^EXIT=' OUTFILE; do sleep 15; done
set -uo pipefail
out=${1:?usage: scripts/suite.sh OUTFILE [cargo test args…]}
shift
fingerprint() {
    # Every file the map and the document gates read — tracked and untracked — by its CONTENT, never by its diff
    # against HEAD: a commit made while the suite runs (the routine allows one) moves `git status` and `git diff`
    # and no file, and routine run 25's suite reported TREE-MOVED for exactly that. A deleted file hashes as missing.
    git ls-files -z --cached --others --exclude-standard | sort -z | xargs -0 -r sha256sum 2>&1 | sha256sum
}
# Routine run 23: a full disk ends a suite mid-link (`No space left on device`), after a dependency change has
# re-hashed every crate above it — run 6's trap, written down and still paid for. Refuse to start below the floor,
# and say what to delete; a suite that dies at its 90th binary costs more than the check.
avail_gb=$(df -Pk . | awk 'NR==2 {print int($4 / 1048576)}')
if [ "${avail_gb:-0}" -lt "${SUITE_MIN_FREE_GB:-10}" ]; then
    {
        echo "NOT STARTED: ${avail_gb} GB free, under the ${SUITE_MIN_FREE_GB:-10} GB a suite needs. Free it first:"
        echo "  rm -rf target/debug/incremental target/<triple>   # caches, cross-lint builds"
        echo "  find target/debug/deps -maxdepth 1 -type f -perm -u+x ! -name '*.so' ! -name '*.dylib' ! -name '*.dll' -mmin +N -delete"
        echo "      # test executables older than the last build — not the proc-macro libraries, which are executable too"
        echo "EXIT=3"
    } | tee "$out" >&2
    exit 3
fi
# Routine run 23 again: a script edited AFTER the map was built and BEFORE this started cost the suite four tests
# (`doctor_cli` three times, the Survey's freshness) — `TREE-MOVED` compares the tree before and after, so an edit
# made before the start is invisible to it. Refuse to start on a stale map: it can only fail for that reason.
if ! cargo run -q -p delulu-survey -- check > /dev/null 2>&1; then
    {
        echo "NOT STARTED: the Survey's map does not match the tree — doctor_cli and the freshness test would fail for that"
        echo "  reason alone. Run \`cargo run -p delulu-survey -- build\` after the last edit, then start the suite."
        echo "EXIT=3"
    } | tee "$out" >&2
    exit 3
fi
before=$(fingerprint)
CARGO_INCREMENTAL=0 cargo test --workspace --no-fail-fast -j 4 "$@" > "$out" 2>&1
code=$?
after=$(fingerprint)
echo "EXIT=$code" >> "$out"
# On-demand run 10: 12 GB free passed the floor beside a 16 GB `target/`, and the suite died at link — an `EXIT=101` that
# read like a failing test. Say it is the disk.
if grep -q "No space left on device" "$out"; then
    echo "DISK-FULL: the disk filled while the suite built or ran — no test result; free it (delete target/, or its stale" \
         "test binaries) and run the suite again" >> "$out"
fi
if [ "$before" != "$after" ]; then
    echo "TREE-MOVED: the working tree changed while the suite ran — a stale map fails doctor_cli and the" \
         "Survey's freshness test for that reason alone; regenerate the map and re-run them (or the suite)" >> "$out"
fi
exit "$code"
