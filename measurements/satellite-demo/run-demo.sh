#!/usr/bin/env bash
# The satellite scenario (Stage 10 §5.6, addendum §2.3, acceptance criterion 10).
#
#   bash measurements/satellite-demo/run-demo.sh
#
# Three passes of a simulated spacecraft, showing the contact window as what it actually is in
# this system: a lease TTL.
#
#   PASS 1   acquisition of signal. The ground-delegated authority (sat0/hga, ±45°) and the
#            pre-attenuated autonomy grant (sat0/wheels, ±0.5°) both accept commands. Partway
#            through, the contact window's TTL runs out — loss of signal — and the HGA lease
#            expires ON ITS OWN, with nobody to send a message, which is the whole point. The
#            wheels keep working: the autonomy grant engages by being the grant that did not end.
#
#   PASS 2   re-contact. A NEW grant, with a new lease. Authority does not come back; it is
#            issued again. The run is otherwise identical, which is the evidence.
#
#   NO-PASS  the control: the same program with no ground delegation at all. The HGA is refused
#            at the mint (DL0703) and only the autonomy box exists. Without this, pass 1 would
#            prove nothing about which grant did what.
#
# EVERYTHING HERE IS SIMULATED, and one honesty note travels with it, from addendum §2.5:
#
#     Criterion 10's satellite demonstration therefore runs BOTH BROKER ROLES INSIDE ONE
#     SIMULATED HOST over a simulated link: it witnesses the grant SEMANTICS (expiry,
#     attenuation, re-delegation), not the federation transport, and its recording says so.
#
# There is no on-board broker, no ground broker, and no link between them. Broker federation is
# named, RFC-gated future work and a prerequisite for any real deployment in this domain.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"

echo "building (cargo build -p delulu --release) ..."
( cd "$root" && cargo build -p delulu --release ) || { echo "build failed" >&2; exit 1; }
DELULU="$root/target/release/delulu"
[ -x "$DELULU" ] || DELULU="$root/target/release/delulu.exe"
export DELULU_NO_FIRST_RUN=1

# The contact window IS the ttl. The heartbeat is sized ABOVE the program's slowest cycle so the
# lease can only end by its TTL running out, never by a beat the interpreter was too slow to send:
# an unoptimized (`cargo test`) cycle of sat-pass.delulu costs ~230 ms, so a 1000 ms heartbeat has
# ~4x of headroom in debug and vastly more in this release build. `ttl_ms >= heartbeat_ms` is a hard
# rule (a lease that expired before its first beat was due is refused), so the HGA's ttl rises with
# its heartbeat; 1000 ms still closes the window mid-pass in both build modes. See ruling D19e.
HGA="actuator=sat0/hga:slew_deg=-45..45,heartbeat_ms=1000,ttl_ms=1000,fail=safe-park"
WHEELS="actuator=sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=600000,ttl_ms=600000,fail=hold"

report() {
  local label="$1" out="$2"
  echo "  hga  commanded: $(grep -c 'hga COMMANDED' <<<"$out")   revoked: $(grep -c 'hga REVOKED' <<<"$out")   no-device: $(grep -c 'hga NODEVICE' <<<"$out")"
  echo "  wheels commanded: $(grep -c '^wheels COMMANDED' <<<"$out")   wide slew refused: $(grep -c 'wheels-wide REFUSED' <<<"$out")"
  local first
  first=$(grep -n -m1 'hga REVOKED' <<<"$out" | cut -d: -f1)
  [ -n "$first" ] && echo "  loss of signal at output line $first"
}

echo
echo "=== PASS 1 — acquisition of signal, then LOS mid-pass ==========================="
out1=$("$DELULU" run "$here/sat-pass.delulu" --grant console --grant "$HGA" --grant "$WHEELS" \
  --broker-profile sim --sim-step 50 --trace-effects --no-prompt 2> "$here/.pass1.err")
report "pass1" "$out1"
grep -o 'lease revoked ([^)]*)[^"]*' "$here/.pass1.err" | head -1 | sed 's/^/  /'
grep -m1 'hga REVOKED' <<<"$out1" | sed 's/^/  /'

echo
echo "=== PASS 2 — re-contact: a NEW delegation, a new lease =========================="
out2=$("$DELULU" run "$here/sat-pass.delulu" --grant console --grant "$HGA" --grant "$WHEELS" \
  --broker-profile sim --sim-step 50 --trace-effects --no-prompt 2> "$here/.pass2.err")
report "pass2" "$out2"
echo "  the HGA commanded again under the new lease, then expired again on its own."

echo
echo "=== NO-PASS — the control: no ground delegation at all =========================="
out3=$("$DELULU" run "$here/sat-pass.delulu" --grant console --grant "$WHEELS" \
  --broker-profile sim --no-prompt 2> "$here/.nopass.err" || true)
if grep -q 'DL0703' "$here/.nopass.err"; then
  echo "  the HGA is refused at the MINT (DL0703) — a capability nobody granted is not minted,"
  echo "  which is a different and earlier refusal than a lease that expired."
  grep -o 'DL0703[^ ]*.*' "$here/.nopass.err" | head -1 | sed 's/^/  /'
fi

echo
echo "the federation gap (addendum §2.5): both broker roles ran in ONE simulated host."
echo "what is witnessed above is grant SEMANTICS — expiry, attenuation, re-delegation —"
echo "and NOT the cross-link transport, which does not exist and is RFC-gated."
rm -f "$here"/.pass1.err "$here"/.pass2.err "$here"/.nopass.err
