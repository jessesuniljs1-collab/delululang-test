#!/usr/bin/env bash
# The arm demonstration (Stage 10 §5.5, criterion 4) — reproducible from a clean checkout.
#
#   cargo build -p delulu --release
#   bash measurements/robotics-demo/run-demo.sh
#
# Runs the four behaviors criterion 4 requires MEASURED, plus the sim-to-hardware gate, and prints
# the numbers this directory's RECORD.md publishes. Everything runs against the in-tree reference
# simulator (`--broker-profile sim`). No hardware is involved and none is claimed.
#
# The four behaviors, and where each number comes from:
#   1. correct edits take effect at rate  — wall clock across N in-envelope commands
#   2. an over-envelope command is refused — the DL1904 refusal record, per command
#   3. heartbeat loss engages the fail-state — `overdue` + `engage` from the trace
#   4. an operator e-stop revokes the subtree — `grants revoke` returning → fail-state engaged
#
# Behavior 4 needs a broker daemon, because that is where the grant tree lives and the e-stop is
# grant-tree revocation. The script starts one in a scratch state dir and stops it on exit.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="${TMPDIR:-/tmp}/delulu-robotics-demo-$$"
mkdir -p "$out"

# Build it here rather than hunting for a binary that may already exist. An earlier draft of this
# script preferred `target/release` if present and silently measured a build from before the
# feature existed — every number it printed was a zero, and it printed them confidently. A
# measurement script that can run against the wrong binary will eventually do so.
echo "building (cargo build -p delulu --release) ..."
( cd "$root" && cargo build -p delulu --release ) || { echo "build failed" >&2; exit 1; }
DELULU="$root/target/release/delulu"
[ -x "$DELULU" ] || DELULU="$root/target/release/delulu.exe"
if [ ! -x "$DELULU" ]; then
  echo "built, but no binary at target/release/delulu[.exe]" >&2
  exit 1
fi
export DELULU_NO_FIRST_RUN=1

# The envelope the operator grants. Note what is in it that the programs do not contain: the
# torque ceiling the agent's edit will exceed, and the three dead-man terms.
ENV_DIMS="angle_deg=-30..95,velocity_dps=0..40,torque_nm=0..2.5"
NOMINAL="actuator=arm0/elbow:$ENV_DIMS,heartbeat_ms=250,ttl_ms=600000,fail=safe-park"
PATIENT="actuator=arm0/elbow:$ENV_DIMS,heartbeat_ms=600000,ttl_ms=600000,fail=safe-park"

echo "=== the arm demonstration (simulated) ============================================"
echo "binary:  $DELULU"
echo "profile: sim (in-tree reference simulator — no hardware)"
echo

# ----- behavior 1: correct edits take effect, at rate -------------------------------------------
echo "--- 1. nominal sweep: 20 in-envelope commands ---"
start=$(date +%s%N)
"$DELULU" run "$here/arm.delulu" --grant console --grant "$PATIENT" \
  --broker-profile sim --no-prompt > "$out/edit1.out" 2> "$out/edit1.err"
end=$(date +%s%N)
ms=$(( (end - start) / 1000000 ))
landed=$(grep -c '^COMMANDED$' "$out/edit1.out" || true)
echo "commands landed: $landed / 20"
echo "wall clock:      ${ms} ms total for the run"
echo

# ----- behavior 2: the over-envelope edit is refused, per command --------------------------------
echo "--- 2. the agent raises torque to 5.0 N.m against a 2.5 N.m envelope ---"
"$DELULU" run "$here/arm-overtorque.delulu" --grant console --grant "$PATIENT" \
  --broker-profile sim --trace-effects --no-prompt > "$out/edit2.out" 2> "$out/edit2.err"
echo "landed:  $(grep -c '^COMMANDED$' "$out/edit2.out" || true)  (expected 2 — before and after)"
echo "refused: $(grep -c '^REFUSED:' "$out/edit2.out" || true)  (expected 1 — the over-torque one)"
grep '^REFUSED:' "$out/edit2.out" | sed 's/^/  /'
echo "DL1904 records in the trace: $(grep -c 'DL1904' "$out/edit2.err" || true)"
echo "exit status: $? — a refused command is a value, so the process exits 0"
echo

# ----- behavior 3: heartbeat loss engages the declared fail-state --------------------------------
echo "--- 3. the controller thinks for longer than its 250 ms heartbeat ---"
"$DELULU" run "$here/arm-wedged.delulu" --grant console --grant "$NOMINAL" \
  --broker-profile sim --trace-effects --no-prompt > "$out/edit3.out" 2> "$out/edit3.err"
grep -o 'beat overdue by [0-9]* µs' "$out/edit3.err" | sed 's/^/  /'
grep -o 'fail-state engaged [0-9]* µs after detection' "$out/edit3.err" | sed 's/^/  /'
grep '^REVOKED:' "$out/edit3.out" | sed 's/^/  /'
echo

# ----- behavior 4: the operator e-stop ----------------------------------------------------------
echo "--- 4. an operator revokes the arm's node while the supervisor is driving ---"
state="$out/state"
mkdir -p "$state"
export DELULU_STATE_DIR="$state"
cleanup() { "$DELULU" broker stop >/dev/null 2>&1 || true; }
trap cleanup EXIT
"$DELULU" broker start >/dev/null 2>&1

"$DELULU" run "$here/arm-supervisor.delulu" --broker daemon --grant console --grant "$PATIENT" \
  --broker-profile sim --trace-effects --no-prompt > "$out/estop.out" 2> "$out/estop.err" &
runpid=$!

# Find the arm's own node — a child of the run's, holder kind `device`. This is the aimed e-stop:
# revoking the PARENT would stop the arm too, transitively, and take the program's console with it.
# `grants list` already prints one line per node ending in `(holder_kind) holder_desc`, so the
# device's node is a grep away — no JSON parser, and therefore no dependency this demo would
# otherwise carry just to find a node an operator finds by eye.
node=""
for _ in $(seq 1 400); do
  node=$("$DELULU" grants list 2>/dev/null \
    | grep '(device) arm0/elbow' | grep '\[live\]' | awk '{print $1}' | head -1)
  [ -n "$node" ] && break
  sleep 0.05
done

if [ -z "$node" ]; then
  echo "  could not find the device node — is the supervisor still running?"
else
  echo "  device node: $node"
  t0=$(date +%s%N)
  "$DELULU" grants revoke "$node" >/dev/null 2>&1
  t1=$(date +%s%N)
  echo "  \`grants revoke\` returned in $(( (t1 - t0) / 1000000 )) ms"
fi
wait $runpid
grep -o 'lease revoked (operator-revoke)[^"]*' "$out/estop.err" | head -1 | sed 's/^/  /'
grep -o 'fail-state engaged [0-9]* µs after detection' "$out/estop.err" | head -1 | sed 's/^/  /'
echo "  the supervisor survived: $(grep -c 'SUPERVISOR DOWN' "$out/estop.out" || true) (1 = yes)"
echo

# ----- the sim-to-hardware gate (DL1905) ---------------------------------------------------------
echo "--- 5. the sim-to-hardware gate: hardware with no sign-off record ---"
"$DELULU" run "$here/arm.delulu" --grant console --grant "$PATIENT" \
  --broker-profile hw:demo-adapter --no-prompt > "$out/hw.out" 2> "$out/hw.err"
echo "  exit: $?  (expected non-zero)"
grep -o 'DL1905[^\n]*' "$out/hw.err" | head -1 | sed 's/^/  /'
echo
echo "artifacts in $out"
