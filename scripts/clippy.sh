#!/usr/bin/env bash
#
# The workspace's lint, the one CI's `lints` job runs, and a stamp of what it passed (on-demand routine run 4, loop
# engineering). On exit 0 it records a hash of every Rust input as the working tree held it; the pre-commit hook refuses
# a commit that stages a Rust input unless the index hashes the same — so "clippy 0" in a commit message is about the
# commit, not about the tree before its last test was written (`81da0ae` went red on `lints` for exactly that).
#
# usage: scripts/clippy.sh [extra cargo arguments, e.g. -j 4]
#   The stamp lives in the git directory (`git rev-parse --git-path clippy-stamp`), so each worktree keeps its own.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
stamp=$(git rev-parse --git-path clippy-stamp)
rm -f "$stamp"
before=$(scripts/hooks/rust-inputs-hash worktree) || exit 2
# CI's `lints` job runs rustfmt before clippy, so a tree rustfmt would change is not stamped either: pull request #20 went
# red on `lints` for two unformatted files, clippy having passed (2026-10-21). `--check` changes nothing.
if ! cargo fmt --all -- --check >/dev/null; then
    echo "clippy.sh: \`cargo fmt --all -- --check\` would change files — no stamp written; run \`cargo fmt --all\`" >&2
    exit 1
fi
cargo clippy --workspace --all-targets "$@" -- -D warnings
code=$?
if [ "$code" -ne 0 ]; then
    echo "clippy.sh: cargo clippy exited $code — no stamp written" >&2
    exit "$code"
fi
after=$(scripts/hooks/rust-inputs-hash worktree) || exit 2
if [ "$before" != "$after" ]; then
    echo "clippy.sh: a Rust input changed while clippy ran — no stamp written; run it again" >&2
    exit 1
fi
printf '%s\n' "$after" > "$stamp"
echo "clippy.sh: clean; stamped $after"
