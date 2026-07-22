#!/usr/bin/env bash
# The federated satellite demonstration (RFC 0001 F6).
#
# Two brokers, two ed25519 identities, NO shared secret. A ground station mints a bounded device
# grant offline; the credential crosses as a FILE (the broker never opens a socket); the spacecraft
# verifies it against a trust anchor, adopts it as a local root, delegates to its own flight
# program, and flies. When the pass ends the uplink lease expires and the antenna stops answering —
# and nobody sent a message to take it away, because at loss-of-signal there is nobody to send one.
#
# The contact window is not a magic number: it comes from the orbital geometry the flight program
# itself computes (see `sat-federated.delulu`, and read its honesty header before believing any
# number here).
#
# Re-run: bash measurements/federation-demo/run-demo.sh
set -u

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DELULU="${DELULU_BIN:-$ROOT/target/debug/delulu}"
[ -x "$DELULU" ] || DELULU="$DELULU.exe"
WORK="${TMPDIR:-/tmp}/delulu-fed-demo-$$"
GROUND="$WORK/ground"
VEHICLE="$WORK/vehicle"
rm -rf "$WORK"; mkdir -p "$GROUND" "$VEHICLE"
export DELULU_NO_FIRST_RUN=1

cleanup() { DELULU_STATE_DIR="$VEHICLE/state" "$DELULU" broker stop >/dev/null 2>&1 || true; }
trap cleanup EXIT

HGA="sat0/hga:slew_deg=-90..90,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park"
WHEELS="sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=60000,ttl_ms=60000,fail=hold"

echo "=============================================================================="
echo " 0. THE PHYSICS — computed by the flight program, in pure DeluluLang"
echo "=============================================================================="
echo "    No foreign calls, no Python: sqrt/sin/cos/asin are built from + - * / and"
echo "    the whole navigation half of the program is EFFECT-FREE."
echo
"$DELULU" run "$ROOT/measurements/federation-demo/sat-federated.delulu" \
  --grant console --grant "actuator=$HGA" --grant "actuator=$WHEELS" \
  --broker-profile sim --no-prompt 2>/dev/null

echo
echo "=============================================================================="
echo " 1. TWO IDENTITIES — no shared secret exists anywhere in this demo"
echo "=============================================================================="
G=$("$DELULU" grants pubkey --key "$GROUND/grant.key")
V=$("$DELULU" grants pubkey --key "$VEHICLE/grant.key")
echo "    ground : $G"
echo "    vehicle: $V"

echo
echo "=============================================================================="
echo " 2. THE GROUND MINTS A CONTACT-WINDOW CERTIFICATE — offline, no daemon"
echo "=============================================================================="
echo "    The pass above is ~9 minutes above the 5-degree mask. The uplink lease is"
echo "    set to 3 SECONDS here so the demonstration fits in a terminal; on a real"
echo "    vehicle it would be the contact cadence. Everything else is unchanged."
echo
"$DELULU" grants certify --subject "$V" --effects Actuate,Write \
  --device "$HGA" --ttl 1h --uplink-ttl 3s \
  --key "$GROUND/grant.key" --out "$WORK/pass1.dlcert"
echo
sed 's/^/    /' "$WORK/pass1.dlcert"

echo
echo "=============================================================================="
echo " 3. THE CREDENTIAL CROSSES AS BYTES — the vehicle adopts it"
echo "=============================================================================="
export DELULU_STATE_DIR="$VEHICLE/state"
mkdir -p "$DELULU_STATE_DIR"
"$DELULU" broker start >/dev/null 2>&1
NODE=$("$DELULU" grants adopt "$WORK/pass1.dlcert" --anchor "$G")
echo "    local root: $NODE"
"$DELULU" grants inspect "$NODE" 2>/dev/null | sed 's/^/    /'
FP=$("$DELULU" grants inspect "$NODE" 2>/dev/null | grep -o '\[[a-f0-9]\{64\}\]' | tr -d '[]')

echo
echo "=============================================================================="
echo " 4. THE VEHICLE DELEGATES TO ITS OWN FLIGHT PROGRAM, AND FLIES"
echo "=============================================================================="
TOK=$("$DELULU" grants delegate --parent "$NODE" --effects Actuate,Write \
  --device "$HGA" --multi --json | grep -o 'dlt1_[a-f0-9.]*')
cat > "$WORK/point.delulu" <<'PEOF'
module point
type Slew { slew_deg: Float }
fn say(r: Result[Unit, ActuateErr]) -> Str {
  match r {
    Ok(u) => "hga COMMANDED",
    Err(e) => match e {
      Envelope(reason) => "hga REFUSED (envelope)",
      LeaseRevoked(reason) => "hga LOST (" + reason + ")",
      NoDevice => "hga NODEVICE"
    }
  }
}
fn main(root: Root) ! {Write, Actuate} {
  let c = root.console()
  let hga = root.actuator("sat0/hga")
  c.println(say(hga.command(Slew { slew_deg: 42.0 })))
}
PEOF
echo "    in contact:"
"$DELULU" run "$WORK/point.delulu" --lease "$TOK" --broker-profile sim --no-prompt 2>/dev/null | sed 's/^/      /'

echo
echo "=============================================================================="
echo " 5. LOSS OF SIGNAL — nothing is sent, nobody is told, time simply passes"
echo "=============================================================================="
sleep 4
echo "    after the uplink lease expires:"
"$DELULU" run "$WORK/point.delulu" --lease "$TOK" --broker-profile sim --no-prompt 2>&1 \
  | grep -E "DL1402|hga|expired" | sed 's/^/      /' | head -4
echo
echo "    Nobody revoked anything. Revocation CANNOT cross a partition; expiry can,"
echo "    and that is the whole design."

echo
echo "=============================================================================="
echo " 6. RE-CONTACT — one signed receipt restores the whole subtree"
echo "=============================================================================="
"$DELULU" grants receipt --for "$FP" --ttl 1h --key "$GROUND/grant.key" --out "$WORK/r1.dlrcpt"
"$DELULU" grants renew "$WORK/r1.dlrcpt" --anchor "$G" >/dev/null
echo "    after the receipt:"
"$DELULU" run "$WORK/point.delulu" --lease "$TOK" --broker-profile sim --no-prompt 2>/dev/null | sed 's/^/      /'

echo
echo "=============================================================================="
echo " 7. THE VEHICLE HANDS BACK WHAT IT DID ALONE — two chains, cross-linked"
echo "=============================================================================="
"$DELULU" broker stop >/dev/null 2>&1
"$DELULU" audit bundle --dir "$VEHICLE/state/audit" --out "$WORK/v.bundle"
"$DELULU" audit reconcile "$WORK/v.bundle" --dir "$GROUND/audit"
echo "    the ground's own chain now holds ONE cross-link record:"
"$DELULU" audit tail 2 --dir "$GROUND/audit" | cut -c1-118 | sed 's/^/      /'
echo
echo "    Both chains still verify independently:"
echo -n "      vehicle: "; "$DELULU" audit verify --dir "$VEHICLE/state/audit"
echo -n "      ground:  "; "$DELULU" audit verify --dir "$GROUND/audit"

echo
echo "=============================================================================="
echo " WHAT THIS DOES NOT SHOW — read RECORD.md before believing anything here"
echo "=============================================================================="
echo "    * Both brokers ran on ONE machine; the 'link' was a file copy. No radio,"
echo "      no latency, no real partition."
echo "    * NO HARDWARE ADAPTER EXISTS. Every device above is the in-tree simulator."
echo "    * The orbit model is unvalidated and written by this project. It is not"
echo "      checked against any operational ephemeris."
echo "    * DeluluLang holds NO certification under any regime, and claims none."
rm -rf "$WORK"
