#!/bin/bash
# P21 scenario 1: cross-OS-account boundary on ext4 (/tmp). Broker=user(1002), attacker(1003).
# Run as root under WSL:  wsl -d Ubuntu-20.04 -u root bash scenario1.sh
# Requires: /usr/local/bin/delulu installed; users `user` and `attacker` to exist.
set -u
STATE=/tmp/dl-boundary
BIN=/usr/local/bin/delulu
echo "===== P21 SCENARIO 1: ext4 (/tmp), 0700 dir boundary ====="
echo "broker user: $(id user)"
echo "attacker   : $(id attacker)"

pkill -u user -f 'delulu broker' 2>/dev/null
sleep 0.3
rm -rf "$STATE"

echo "--- start broker as user (foreground, backgrounded by script) ---"
runuser -u user -- env DELULU_STATE_DIR="$STATE" "$BIN" broker start --foreground >/tmp/dl-broker.log 2>&1 &
ok=0
for i in $(seq 1 100); do
  if [ -S "$STATE/broker.sock" ]; then ok=1; break; fi
  sleep 0.1
done
sleep 0.3
BROKER_PID=$(pgrep -u user -f 'delulu broker start' | head -1)
echo "socket_ready=$ok broker_pid=$BROKER_PID"
echo "--- broker log (head) ---"; head -6 /tmp/dl-broker.log
echo "--- THE CRUX: perms of the state dir + key (proves 0700/0600 stuck on ext4) ---"
stat -c 'dir  %n : mode=%a owner=%U' "$STATE"
stat -c 'sock %n : mode=%a owner=%U' "$STATE/broker.sock" 2>/dev/null
stat -c 'key  %n : mode=%a owner=%U' "$STATE/broker.key" 2>/dev/null

if [ "$ok" != 1 ]; then echo "FATAL: broker did not bind socket"; cat /tmp/dl-broker.log; pkill -u user -f 'delulu broker'; exit 1; fi

echo
echo "########## ATTACKER (uid 1003, SEPARATE account) — every one should be DENIED ##########"

echo "--- A1: attacker ls state dir ---"
out=$(runuser -u attacker -- ls -la "$STATE" 2>&1); echo "rc=$? out=[$out]"

echo "--- A2: attacker cat broker.key (the lease-MAC secret) ---"
out=$(runuser -u attacker -- cat "$STATE/broker.key" 2>&1); echo "rc=$? out=[$out]"

echo "--- A3: attacker 'grants list' over IPC ---"
out=$(runuser -u attacker -- env DELULU_STATE_DIR="$STATE" "$BIN" grants list 2>&1); echo "rc=$? out=[$out]"

echo "--- A4: attacker raw AF_UNIX connect to broker.sock ---"
out=$(runuser -u attacker -- python3 -c "import socket
s=socket.socket(socket.AF_UNIX)
try:
    s.connect('$STATE/broker.sock'); print('CONNECTED')
except Exception as e:
    print('EXC', type(e).__name__, e)" 2>&1); echo "rc=$? out=[$out]"

echo "--- A5: attacker kill broker pid $BROKER_PID ---"
out=$(runuser -u attacker -- kill "$BROKER_PID" 2>&1); echo "rc=$? out=[$out]"
if kill -0 "$BROKER_PID" 2>/dev/null; then echo "broker STILL ALIVE after attacker kill = boundary HELD"; else echo "broker DIED = boundary FAILED"; fi

echo
echo "########## FALSIFICATION: same uid (user 1002) — every one should SUCCEED ##########"

echo "--- F1: user ls state dir ---"
out=$(runuser -u user -- ls -la "$STATE" 2>&1); echo "rc=$? lines=$(printf '%s\n' "$out" | wc -l)"

echo "--- F2: user read broker.key size ---"
out=$(runuser -u user -- bash -c "wc -c < '$STATE/broker.key'" 2>&1); echo "rc=$? bytes=[$out]"

echo "--- F3: user 'grants list' over IPC ---"
out=$(runuser -u user -- env DELULU_STATE_DIR="$STATE" "$BIN" grants list 2>&1); echo "rc=$? out=[$out]"

echo "--- F4: user raw AF_UNIX connect ---"
out=$(runuser -u user -- python3 -c "import socket
s=socket.socket(socket.AF_UNIX)
try:
    s.connect('$STATE/broker.sock'); print('CONNECTED')
except Exception as e:
    print('EXC', type(e).__name__, e)" 2>&1); echo "rc=$? out=[$out]"

echo
echo "--- cleanup ---"
kill "$BROKER_PID" 2>/dev/null
pkill -u user -f 'delulu broker' 2>/dev/null
echo "SCENARIO1-DONE"
