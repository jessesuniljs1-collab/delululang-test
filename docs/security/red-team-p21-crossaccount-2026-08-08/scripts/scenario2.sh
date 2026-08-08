#!/bin/bash
# P21 scenario 2: does the perm boundary EXIST on a 9p (Windows-mounted) state dir?
# Run as root under WSL:  wsl -d Ubuntu-20.04 -u root bash scenario2.sh
# Requires: /usr/local/bin/delulu installed; users `user` and `attacker`; /mnt/d is a 9p mount.
set -u
BIN=/usr/local/bin/delulu
S9=/mnt/d/dl-b9p
echo "===== P21 SCENARIO 2: 9p (/mnt/d) — is there any perm boundary? ====="
echo "--- /mnt/d type ---"; df -T /mnt/d | tail -1; ls -lad /mnt/d

echo
echo "--- RAW chmod-stick test as user: mkdir 0700 + file 0600 on 9p, read back ---"
runuser -u user -- bash -c "rm -rf '$S9'; mkdir -p '$S9'; chmod 700 '$S9'; echo topsecret > '$S9/probe.txt'; chmod 600 '$S9/probe.txt'; stat -c 'dir  mode=%a owner=%U' '$S9'; stat -c 'file mode=%a owner=%U' '$S9/probe.txt'"

echo "--- ATTACKER reads the '0600' file on 9p (denied => boundary real; content => LEAK) ---"
out=$(runuser -u attacker -- cat "$S9/probe.txt" 2>&1); echo "rc=$? out=[$out]"
echo "--- ATTACKER lists the '0700' dir on 9p ---"
out=$(runuser -u attacker -- ls -la "$S9" 2>&1); echo "rc=$? out=[$out]"

echo
echo "--- END-TO-END: run the broker with its state dir on 9p ---"
pkill -u user -f 'delulu broker' 2>/dev/null; sleep 0.3
runuser -u user -- bash -c "rm -rf '$S9'"
runuser -u user -- env DELULU_STATE_DIR="$S9" "$BIN" broker start --foreground >/tmp/dl-broker9.log 2>&1 &
ok=0
for i in $(seq 1 100); do [ -S "$S9/broker.sock" ] && { ok=1; break; }; sleep 0.1; done
sleep 0.4
BP=$(pgrep -u user -f 'delulu broker start' | head -1)
echo "socket_ready=$ok broker_pid=$BP"
echo "--- broker9 log (head) ---"; head -8 /tmp/dl-broker9.log
echo "--- achieved perms on the 9p state dir ---"
stat -c 'dir  %n mode=%a owner=%U' "$S9" 2>&1
stat -c 'key  %n mode=%a owner=%U' "$S9/broker.key" 2>&1
stat -c 'sock %n mode=%a owner=%U' "$S9/broker.sock" 2>&1

echo "--- ATTACKER reads broker.key (the lease-MAC secret) from 9p ---"
out=$(runuser -u attacker -- bash -c "wc -c < '$S9/broker.key'" 2>&1); echo "attacker broker.key bytes=[$out]  (32 => LEAK, denied => held)"

echo "--- ATTACKER connects to broker.sock on 9p ---"
out=$(runuser -u attacker -- python3 -c "import socket
s=socket.socket(socket.AF_UNIX)
try:
    s.connect('$S9/broker.sock'); print('CONNECTED')
except Exception as e:
    print('EXC', type(e).__name__, e)" 2>&1); echo "rc=$? out=[$out]"

echo "--- ATTACKER 'grants list' over IPC on 9p ---"
out=$(runuser -u attacker -- env DELULU_STATE_DIR="$S9" "$BIN" grants list 2>&1); echo "rc=$? out=[$out]"

echo "--- cleanup ---"
kill "$BP" 2>/dev/null; pkill -u user -f 'delulu broker' 2>/dev/null
runuser -u user -- bash -c "rm -rf '$S9'" 2>/dev/null
echo "SCENARIO2-DONE"
