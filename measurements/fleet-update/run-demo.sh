#!/usr/bin/env bash
# The fleet-update drill (Stage 10 spec §9.3, phase 10j, the second half of acceptance
# criterion 9) — reproducible from a clean checkout, once `delulu fleet update` exists.
#
#   cargo build -p delulu --release
#   bash measurements/fleet-update/run-demo.sh
#
# This script was originally written as a scaffold, in parallel with `delulu fleet update` itself,
# against the contract below rather than the real command (which did not exist yet). It has since
# been run for real, end to end, all four passes matching their expected exit codes, and
# RECORD.md's numbers come from that real run — one reconciliation was needed: `--previous` turned
# out to be REQUIRED on every invocation (see fleet.rs's own doc comment), not only ones expected
# to fail, so passes 1/3/4 below now pass it too.
#
# The contract this script assumes:
#
#   delulu fleet update <artifact-path> --members N --approved <signoff-record-path>
#                        [--fail-health-at I] [--previous <hash-or-path>] [--json]
#
#   - rolls a signed artifact out across N simulated fleet members, staged one at a time.
#   - --approved names a sign-off record: delulu_runtime::device::Approval's EXISTING JSON shape
#     (approved_artifact, approved_hash = "blake3:<64 hex>", approved_under), reused as-is from
#     the 10f device sign-off flow rather than a fleet-specific shape.
#   - the artifact's real content hash must match approved_hash, or the update refuses with
#     DL1905 (the SAME code as the 10f device sim-to-hardware gate, generalized) BEFORE touching
#     any fleet member. A missing/corrupt approval file is ALSO a DL1905 refusal — never treated
#     as "nothing to check".
#   - --fail-health-at I fails member I's health check on purpose, to prove rollback actually
#     happens: the rollout stops, members after I are NEVER staged, and the fleet rolls back to
#     whatever --previous named.
#   - exit 0 only if every member completed; exit 1 on a DL1905 refusal or a rollback.
#
# Four passes exercise it:
#   PASS 1   the clean rollout (control case) — every member completes, exit 0.
#   PASS 2   a health-gate failure at member 2 of 5 — rollback fires, members 3-4 never staged.
#   PASS 3   the hash-mismatch refusal (DL1905) — an artifact edited after its approval was signed.
#   PASS 4   the skip branch: a missing approval file — also DL1905, never a free pass.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="${TMPDIR:-/tmp}/delulu-fleet-update-demo-$$"
mkdir -p "$out"

# Build it here rather than trusting a binary that may already exist. measurements/robotics-demo's
# run-demo.sh learned this the hard way: an earlier draft of that script preferred an existing
# target/release build, found one that predated the feature under test, and printed a confident
# page of zeros — every number wrong and none of them looking it. Every script in this family
# builds fresh for that reason, this one included, even though (see the notice above) there is no
# feature yet for a stale binary to be missing.
echo "building (cargo build -p delulu --release) ..."
( cd "$root" && cargo build -p delulu --release ) || { echo "build failed" >&2; exit 1; }
DELULU="$root/target/release/delulu"
[ -x "$DELULU" ] || DELULU="$root/target/release/delulu.exe"
if [ ! -x "$DELULU" ]; then
  echo "built, but no binary at target/release/delulu[.exe]" >&2
  exit 1
fi
export DELULU_NO_FIRST_RUN=1

echo "=== the fleet-update drill (simulated fleet members) =============================="
echo "binary: $DELULU"
echo

# ----- fixtures: the artifact and its sign-off record -------------------------------------------
#
# "Artifact" fixtures. This project does not care what is inside one, only its bytes (hash) and
# signature status — so these are the smallest programs that will actually RUN under `delulu run`,
# which matters below (see real_hash_of). v0 and v1 differ only in the string each prints, on
# purpose: the approved-hash gate compares BYTES, never intentions. See
# crates/delulu/tests/dead_man_cli.rs, `an_artifact_edited_after_signoff_is_refused_dl1905`, which
# makes exactly this point for the device case DL1905 is generalized from.
cat > "$out/artifact-v0.delulu" <<'EOF'
module fleetdemo

// fleet-update demo fixture: the PREVIOUS version, pinned before this drill's update began.
fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println("fleet payload v0 (baseline)")
}
EOF

cat > "$out/artifact-v1.delulu" <<'EOF'
module fleetdemo

// fleet-update demo fixture: the version being rolled out in this drill.
fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println("fleet payload v1 (update)")
}
EOF

# There is no standalone `delulu hash <file>` command. Rather than invent a hash value or
# re-implement blake3 in bash, this borrows the ALREADY-SHIPPED 10f device gate purely as a
# hash-computing step: `run --broker-profile sim --signoff` writes an Approval record whose
# approved_hash is the real delulu_broker::content_hash of the exact bytes passed to it. Spec
# §9.3 says fleet updates "ride the existing machinery" — this is that, applied one command
# early, before `fleet update` exists to do its own hashing. Neither fixture program above
# touches Actuate, so nothing about the device simulator's behavior matters here — only its
# hashing does, and that part is real, shipped code, not a guess.
real_hash_of() {
  local artifact="$1"
  local tmp="$out/.hash-probe-$$.json"
  rm -f "$tmp"
  "$DELULU" run "$artifact" --grant console --broker-profile sim --signoff "$tmp" --no-prompt \
    >/dev/null 2>&1
  if [ ! -f "$tmp" ]; then
    return 1
  fi
  grep '"approved_hash"' "$tmp" | sed -E 's/.*"approved_hash":[[:space:]]*"([^"]*)".*/\1/'
  rm -f "$tmp"
}

HASH_V0="$(real_hash_of "$out/artifact-v0.delulu")"
if [ -z "$HASH_V0" ]; then
  echo "fatal: could not compute a real content-hash for artifact-v0.delulu via the sign-off probe" >&2
  echo "       (is \$DELULU broken, independent of \`fleet\` not existing yet?)" >&2
  exit 1
fi
HASH_V1="$(real_hash_of "$out/artifact-v1.delulu")"
if [ -z "$HASH_V1" ]; then
  echo "fatal: could not compute a real content-hash for artifact-v1.delulu via the sign-off probe" >&2
  exit 1
fi
echo "artifact-v0.delulu real content hash: $HASH_V0"
echo "artifact-v1.delulu real content hash: $HASH_V1"
echo

# delulu_runtime::device::Approval's JSON shape, hand-written — there is no CLI that writes one
# for an arbitrary artifact outside the device flow borrowed above. approved_artifact and
# approved_under are free-form provenance text with no bearing on the gate (Approval::parse never
# checks them against the real file); only approved_hash is compared to the artifact's real bytes.
write_approval() {
  local path="$1" artifact_name="$2" hash="$3" provenance="$4"
  cat > "$path" <<EOF2
{
  "approved_artifact": "$artifact_name",
  "approved_hash": "$hash",
  "approved_under": "$provenance"
}
EOF2
}

write_approval "$out/approved-v1.json" "artifact-v1.delulu" "$HASH_V1" \
  "fleet-update-demo: content-hash obtained via delulu run --broker-profile sim --signoff"

# The PASS 3 fixture: approve the CORRECT bytes, then mutate the artifact — the same tamper
# pattern crates/delulu/tests/dead_man_cli.rs uses for the device gate
# (an_artifact_edited_after_signoff_is_refused_dl1905), applied here to the fleet gate instead.
cp "$out/artifact-v1.delulu" "$out/artifact-v1-tampered.delulu"
HASH_TAMPER="$(real_hash_of "$out/artifact-v1-tampered.delulu")"
if [ -z "$HASH_TAMPER" ]; then
  echo "fatal: could not compute a real content-hash for artifact-v1-tampered.delulu" >&2
  exit 1
fi
write_approval "$out/approved-v1-tampered.json" "artifact-v1-tampered.delulu" "$HASH_TAMPER" \
  "fleet-update-demo: approved BEFORE the tamper below"
# The edit: the most innocuous change available, appended AFTER approval was recorded. The gate
# must catch this because it compares bytes, not intentions — a comment changes nothing a program
# does and everything about whether these are the approved bytes.
printf '\n// mutated after approval -- these bytes must never match the approved_hash above\n' \
  >> "$out/artifact-v1-tampered.delulu"

echo "fixtures ready in $out"
echo

# ----- the four passes ---------------------------------------------------------------------------

report() {
  local pass_out="$1" pass_err="$2" expect_exit="$3" actual_exit="$4"
  echo "  exit code: $actual_exit (expected $expect_exit)"
  if [ "$actual_exit" = "$expect_exit" ]; then
    echo "  exit code matches expectation"
  else
    echo "  exit code DOES NOT match expectation — read the raw output below; \`fleet update\`"
    echo "  may not exist yet, or its flags/exit convention may differ from the contract assumed"
    echo "  here (expected and reconciled — see the SCAFFOLD NOTICE at the top of this file)"
  fi
  if grep -q 'DL1905' "$pass_out" "$pass_err" 2>/dev/null; then
    echo "  DL1905 present in output: yes"
    grep -h -o 'DL1905[^"]*' "$pass_out" "$pass_err" 2>/dev/null | head -1 | sed 's/^/    /'
  else
    echo "  DL1905 present in output: no"
  fi
  echo "  --- raw stdout ---"
  sed 's/^/    /' "$pass_out"
  echo "  --- raw stderr ---"
  sed 's/^/    /' "$pass_err"
}

echo "--- PASS 1: the clean rollout (control case) ---"
echo "  delulu fleet update artifact-v1.delulu --members 5 --approved approved-v1.json --previous artifact-v0.delulu"
# --previous is UNCONDITIONALLY required (the builder agent's own design call, reasoned in
# fleet.rs's doc comment: a rollback target that is only sometimes supplied is a bound nobody
# enforces) -- even on a run expected to succeed cleanly, since the rollback target must be
# pinned before rollout starts, not fetched only once a failure has already happened.
"$DELULU" fleet update "$out/artifact-v1.delulu" --members 5 \
  --approved "$out/approved-v1.json" --previous "$out/artifact-v0.delulu" \
  > "$out/pass1.out" 2> "$out/pass1.err"
code1=$?
report "$out/pass1.out" "$out/pass1.err" 0 "$code1"
echo "  every one of the 5 members is expected to complete; none should be skipped or rolled back."
echo "  without this control passing, a failure in pass 2/3/4 would prove nothing — it would be"
echo "  equally consistent with the mechanism never having worked at all."
echo

echo "--- PASS 2: a health-gate failure at member 2 of 5 triggers rollback ---"
echo "  delulu fleet update artifact-v1.delulu --members 5 --fail-health-at 2 \\"
echo "                       --previous artifact-v0.delulu --approved approved-v1.json"
"$DELULU" fleet update "$out/artifact-v1.delulu" --members 5 --fail-health-at 2 \
  --previous "$out/artifact-v0.delulu" --approved "$out/approved-v1.json" \
  > "$out/pass2.out" 2> "$out/pass2.err"
code2=$?
report "$out/pass2.out" "$out/pass2.err" 1 "$code2"
echo "  per the contract: members 0 and 1 complete, member 2 fails its health check, rollback"
echo "  fires to the artifact/hash named by --previous, and members 3 and 4 are NEVER staged —"
echo "  not touched-then-skipped, never touched at all. This is asserted here in prose because"
echo "  the exact per-member output text is not yet known (the command does not exist yet);"
echo "  confirm it by eye against the raw output above, and tighten this check once that text is"
echo "  known (member 3/4 must not appear as touched/staged/commanded in any form)."
echo

echo "--- PASS 3: hash-mismatch refusal (DL1905), before any member is touched ---"
echo "  delulu fleet update artifact-v1-tampered.delulu --members 5 --approved approved-v1-tampered.json --previous artifact-v0.delulu"
# --previous is present here too (required on every invocation) but its VALUE plays no role in
# this pass: the DL1905 hash check runs before --previous is ever resolved, so this refusal fires
# for the same reason whether --previous points anywhere sensible or not.
"$DELULU" fleet update "$out/artifact-v1-tampered.delulu" --members 5 \
  --approved "$out/approved-v1-tampered.json" --previous "$out/artifact-v0.delulu" \
  > "$out/pass3.out" 2> "$out/pass3.err"
code3=$?
report "$out/pass3.out" "$out/pass3.err" 1 "$code3"
echo "  the approval above was signed for artifact-v1-tampered.delulu's ORIGINAL bytes; a line"
echo "  was appended afterward, so the artifact's real hash no longer matches approved_hash."
echo "  DL1905 must fire before any member is touched — the same code, and the same reason, as"
echo "  the 10f device sim-to-hardware gate: an edit after sign-off is a different artifact at"
echo "  the end of a wire that moves something."
echo

echo "--- PASS 4: the skip branch -- a missing approval file ---"
echo "  delulu fleet update artifact-v1.delulu --members 5 --approved does-not-exist.json --previous artifact-v0.delulu"
"$DELULU" fleet update "$out/artifact-v1.delulu" --members 5 \
  --approved "$out/does-not-exist.json" --previous "$out/artifact-v0.delulu" \
  > "$out/pass4.out" 2> "$out/pass4.err"
code4=$?
report "$out/pass4.out" "$out/pass4.err" 1 "$code4"
echo "  this is the case that matters most. A missing sign-off record is not \"nothing to"
echo "  check\" — it is a refusal. A gate that read an absent record as nothing to check would be"
echo "  the gate opening on damage: exactly the DL1905 skip-branch lesson the device gate was"
echo "  built to enforce (crates/delulu/tests/dead_man_cli.rs,"
echo "  a_hardware_profile_with_no_signoff_record_is_refused_dl1905), now asked of the fleet gate."
echo

echo "=== summary (expected exit -> actual exit) ========================================"
echo "  PASS 1 (clean rollout):        expected 0 -> actual $code1"
echo "  PASS 2 (health-gate rollback): expected 1 -> actual $code2"
echo "  PASS 3 (hash mismatch):        expected 1 -> actual $code3"
echo "  PASS 4 (missing approval):     expected 1 -> actual $code4"
echo
echo "artifacts and fixtures in $out"
