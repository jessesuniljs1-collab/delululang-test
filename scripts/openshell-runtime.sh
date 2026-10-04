#!/usr/bin/env bash
# PS-E-05: the exported policy ENFORCED by a real OpenShell sandbox — the study's §4.5 runtime witnesses,
# by hand first (routine run 8 wrote the first attempt; each run's reading is in `V2_LOG.md`).
#
# `delulu sandbox policy FILE --format openshell` writes the wall for `delulu run`. Here a local OpenShell
# gateway (Docker driver) creates a sandbox from an image holding this build's `delulu` and the program,
# under exactly that exported policy, and then: (1) the granted program runs through `sandbox exec` and
# prints what it read; (2) a program asking for more than its grants is refused by DeluluLang (DL0703);
# (3) `curl` inside the sandbox — a binary the policy never names — to a host the policy never names is
# denied by OpenShell; (4) the sandbox's EFFECTIVE policy (OpenShell adds its baseline) is printed and
# checked by the prover against the export, so what OpenShell added is named, not assumed; (5) — E-05 (b) —
# the GUEST inside a second OpenShell sandbox with no network rule, reached as an `external:` launcher through
# `openshell sandbox exec`: undeclared it fails closed (OpenShell refuses its own syscall filter), declared
# (`--outer-syscall-filter`, D-V2-83) it runs the program end to end at level 3.
#
# Usage: scripts/openshell-runtime.sh DELULU   (needs OpenShell installed — scripts/openshell-install.sh —
# Docker, and a systemd user session; a CI runner has all three). Exit 0 when every expectation held.
set -uo pipefail

DELULU="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
work="$(mktemp -d)"
cd "$work"
failures=0
fail() {
  echo "FAIL: $*"
  failures=$((failures + 1))
}
step() { echo; echo "== $*"; }

step "the gateway (a systemd user service the package installs), pinned to the Docker driver"
sudo loginctl enable-linger "$(id -un)"
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
for _ in $(seq 1 30); do [ -S "$XDG_RUNTIME_DIR/bus" ] && break; sleep 1; done
# Unset, the gateway auto-detects Kubernetes, then Podman, then Docker — and a runner has Podman, while the
# image below is built with Docker (run 3 of this workflow: the service's first act was `podman info`).
conf="${XDG_CONFIG_HOME:-$HOME/.config}/openshell/gateway.toml"
mkdir -p "$(dirname "$conf")"
if [ -s "$conf" ]; then
  echo "(a gateway config already exists — shown, not changed)"
  cat "$conf"
  grep -q 'compute_driver *= *"docker"' "$conf" || fail "the existing gateway config does not pin the Docker driver"
else
  # Schema version 2 is required: run 4's file without `[openshell] version` failed the service's
  # preflight ("missing_version") twenty times over.
  printf '[openshell]\nversion = 2\n\n[openshell.gateway]\ncompute_driver = "docker"\n' > "$conf"
fi
openshell-gateway config preflight --path "$conf" || fail "the gateway config failed its own preflight"
systemctl --user daemon-reload
systemctl --user enable openshell-gateway
systemctl --user restart openshell-gateway || fail "the gateway service did not start"
systemctl --user --no-pager status openshell-gateway | head -20 || true
# Run 3: `openshell status` succeeded with NO gateway registered, so readiness is asked of the gateway itself.
# Run 4: `gateway add --local` needs the client TLS material the service provisions once it has started.
added=0
for _ in $(seq 1 60); do
  if openshell gateway add https://127.0.0.1:17670 --local --name openshell; then added=1; break; fi
  sleep 2
done
[ "$added" = 1 ] || fail "the local gateway could not be registered"
openshell gateway select openshell || true
openshell gateway list || true
up=0
for _ in $(seq 1 60); do
  if openshell sandbox list >/dev/null 2>&1; then up=1; break; fi
  sleep 2
done
[ "$up" = 1 ] || { openshell status; journalctl --user -u openshell-gateway --no-pager | tail -40; fail "the gateway never answered"; }
openshell status || true
docker version --format 'docker server {{.Server.Version}}' || true

step "the image: Ubuntu 24.04, a non-root user, /sandbox, and this build's delulu with the program"
mkdir -p ctx/data
cp "$DELULU" ctx/delulu
echo "hello from the granted file" > ctx/data/in.txt
cat > ctx/p.delulu <<'EOF'
module p

fn main(root: Root) ! {Read, Write} {
    let c = root.console()
    let src = root.fs_read("./data")
    match src.read_text("in.txt") {
        Ok(t) => c.println(t),
        Err(_) => c.println("unreadable")
    }
}
EOF
cat > ctx/greedy.delulu <<'EOF'
module greedy

fn main(root: Root) ! {Read, Write} {
    let c = root.console()
    let src = root.fs_read("/etc")
    match src.read_text("/etc/hostname") {
        Ok(t) => c.println(t),
        Err(_) => c.println("unreadable")
    }
}
EOF
# The base matches the runner that built `delulu` (run 6: Debian bookworm's glibc 2.36 against a build needing 2.39).
cat > ctx/Dockerfile <<'EOF'
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl iproute2 \
    && rm -rf /var/lib/apt/lists/*
RUN groupadd --gid 1500 app && useradd --uid 1500 --gid app --create-home app
RUN install -d -o app -g app /sandbox
COPY delulu /usr/local/bin/delulu
COPY --chown=app:app p.delulu greedy.delulu /sandbox/
COPY --chown=app:app data /sandbox/data
USER app
WORKDIR /sandbox
EOF
docker build -q -t delulu-openshell:ci ctx || fail "the image did not build"

step "the export: the program's authority within its grants"
(cd ctx && "$DELULU" sandbox policy p.delulu --format openshell --workdir /sandbox --run-as 1500:1500 \
  --grant fs.read=./data --grant console) > policy.yaml || fail "the export was refused"
cat policy.yaml

step "the sandbox, under exactly that policy"
openshell sandbox create --name dl --from delulu-openshell:ci --policy policy.yaml --detach -- sleep 3600 \
  || fail "the sandbox was not created"
for _ in $(seq 1 90); do
  openshell sandbox exec -n dl --no-tty --no-login-shell -- /usr/local/bin/delulu --version >/dev/null 2>&1 && break
  sleep 2
done
openshell sandbox list || true

step "(1) the granted program runs"
out="$(openshell sandbox exec -n dl --no-tty --no-login-shell -- /bin/sh -c \
  'cd /sandbox && /usr/local/bin/delulu run p.delulu --no-prompt --grant console --grant fs.read=./data' 2>&1)"
echo "$out"
printf '%s' "$out" | grep -q "hello from the granted file" || fail "(1) the granted program did not print its file"

step "(2) the same program under a narrower grant: DeluluLang refuses what the grant does not cover (DL0703)"
out="$(openshell sandbox exec -n dl --no-tty --no-login-shell -- /bin/sh -c \
  'cd /sandbox && /usr/local/bin/delulu run p.delulu --no-prompt --grant console --grant fs.read=./data/sub' 2>&1)"
echo "$out"
printf '%s' "$out" | grep -q "DL0703" || fail "(2) the program was not refused with DL0703 under the narrower grant"

step "(2b) a program file the export never names is refused by OpenShell's wall"
out="$(openshell sandbox exec -n dl --no-tty --no-login-shell -- /bin/sh -c \
  'cd /sandbox && /usr/local/bin/delulu run greedy.delulu --no-prompt --grant console --grant fs.read=./data' 2>&1)"
echo "$out"
printf '%s' "$out" | grep -qi "permission denied" || fail "(2b) a file outside the exported wall was readable"

step "(3) curl, a binary the policy never names, to a host it never names, is denied by OpenShell"
out="$(openshell sandbox exec -n dl --no-tty --no-login-shell -- curl -sS -m 20 -o /dev/null -w '%{http_code}' https://example.org/ 2>&1)"
code=$?
echo "exit $code: $out"
[ "$code" != 0 ] || fail "(3) curl reached example.org"
# OpenShell's own account of the refusal, READ, not assumed (`--tail` streams, so a bounded window): run 8
# read it as OCSF lines — `NET:REFUSE [MED] DENIED example.org [reason:policy_dns_ineligible]` and
# `NET:OPEN [MED] DENIED /usr/bin/curl(0) -> example.org:443 [reason:transparent_tcp_policy_denied]`.
# The sandbox's log reaches the gateway asynchronously: run 9 read it the instant curl returned and found no
# line that run 8 had found at the same point — so the line is waited for, with a bound, never assumed.
seen=0
for _ in $(seq 1 20); do
  timeout 30 openshell logs dl --since 10m > openshell.log 2>&1 || true
  if grep -qE 'NET:OPEN.*DENIED.*curl.*example\.org' openshell.log; then seen=1; break; fi
  sleep 3
done
grep -E 'NET:(OPEN|REFUSE).*DENIED' openshell.log || { echo "(no DENIED line; the log's last lines:)"; tail -15 openshell.log; }
[ "$seen" = 1 ] || fail "(3) OpenShell's log holds no NET:OPEN DENIED line for curl to example.org within 60 s"

step "(4) the effective policy — what OpenShell added — against the export"
openshell sandbox get dl --policy-only > effective.yaml || fail "(4) no effective policy"
cat effective.yaml
openshell-prover check policy.yaml --boundary effective.yaml --output json 2>/dev/null
echo "export within the effective policy: exit $?"
openshell-prover check effective.yaml --boundary policy.yaml --output json 2>/dev/null
echo "effective within the export: exit $? (non-zero names what OpenShell added)"

step "(5) E-05 (b): the GUEST inside an OpenShell sandbox, as an external launcher (level 3)"
# The guest performs no effects — the host sends the program and performs every effect under the grants — so
# its sandbox needs no network rule at all and only the paths `delulu` itself starts from, read-only. This
# boundary is written by hand here (this repository's own text): it is the guest's, not the program's.
cat > guest-policy.yaml <<'EOF'
version: 1
filesystem_policy:
  include_workdir: false
  read_only: [/usr, /lib, /etc]
  read_write: []
landlock:
  compatibility: hard_requirement
process:
  run_as_user: "1500"
  run_as_group: "1500"
network_policies: {}
EOF
openshell sandbox create --name dlg --from delulu-openshell:ci --policy guest-policy.yaml --detach -- sleep 3600 \
  || fail "(5) the guest's sandbox was not created"
for _ in $(seq 1 90); do
  openshell sandbox exec -n dlg --no-tty --no-login-shell -- /usr/local/bin/delulu --version >/dev/null 2>&1 && break
  sleep 2
done

# Routine run 10: the exec relay, MEASURED. Its first reading of (5b) found the guest's confinement report never reached
# the host, while the guest's standard-error lines did. When does a command's output reach this side — at once, at the
# next newline, or when the command ends? And its input the other way? Each byte is timed as it arrives.
echo "-- (5-) the exec relay: when each byte arrives (A at 0 s, a newline at 3 s, C at 6 s, the end at 12 s)"
openshell sandbox exec -n dlg --no-tty --no-login-shell -- /bin/sh -c \
  'printf A; sleep 3; printf "B\n"; sleep 3; printf C; sleep 6' 2>/dev/null | python3 -c '
import os, time
t0 = time.time()
while True:
    b = os.read(0, 1)
    if not b:
        print("  %5.1f s  stdout ends" % (time.time() - t0)); break
    print("  %5.1f s  stdout %r" % (time.time() - t0, b), flush=True)
'
echo "-- (5-) standard error: E at 0 s, a newline at 3 s, the end at 6 s"
openshell sandbox exec -n dlg --no-tty --no-login-shell -- /bin/sh -c \
  'printf E >&2; sleep 3; printf "F\n" >&2; sleep 3' 2>&1 >/dev/null | python3 -c '
import os, time
t0 = time.time()
while True:
    b = os.read(0, 1)
    if not b:
        print("  %5.1f s  stderr ends" % (time.time() - t0)); break
    print("  %5.1f s  stderr %r" % (time.time() - t0, b), flush=True)
'
echo "-- (5-) standard input: x at 0 s, y at 3 s (no newline), the end at 6 s; each echoed with a newline"
( printf x; sleep 3; printf y; sleep 3 ) | openshell sandbox exec -n dlg --no-tty --no-login-shell -- /bin/sh -c \
  'dd bs=1 count=1 2>/dev/null; echo " <- the first byte in"; dd bs=1 count=1 2>/dev/null; echo " <- the second byte in"' \
  2>&1 | python3 -c '
import sys, time
t0 = time.time()
for line in sys.stdin:
    print("  %5.1f s  %s" % (time.time() - t0, line.rstrip()), flush=True)
'
# The launcher: the channel on the exec's standard streams; the guest's words from the host (no shell word-splits
# a token — DELULU_GUEST_ARGS is DeluluLang's own, `__guest --stdio-pipes`), then the launcher's own EXTRA words.
# D-V2-83 (routine run 9): OpenShell's filter refuses the guest's own (`seccomp`: EPERM, run 8's reading), so this
# launcher DECLARES the outer wall — `--outer-syscall-filter` — and the guest runs under OpenShell's filter,
# saying so in its words; undeclared, it fails closed as before. Both are read here, in that order.
# Routine run 10: run 9's first reading of (5b) was red — the guest looked for the filter in `/proc/self/status`,
# and this sandbox's policy names no `/proc` (OpenShell adds none), so it could not read it and failed closed under a
# filter that was there. It now asks the kernel (`PR_GET_SECCOMP`); the test suite's simulated wall hides `/proc` too.
cat > openshell-guest <<'EOF'
#!/bin/sh
exec openshell sandbox exec -n dlg --no-tty --no-login-shell -- /usr/local/bin/delulu $DELULU_GUEST_ARGS $GUEST_EXTRA
EOF
chmod +x openshell-guest
mkdir -p host/data
echo "hello through the OpenShell guest" > host/data/in.txt
cp ctx/p.delulu host/
guest_run() { # $1: the launcher's extra words; $2: the output file; $3: the report file
  local t0=$SECONDS
  (cd host && GUEST_EXTRA="$1" "$DELULU" run p.delulu --no-prompt --grant console --grant fs.read=./data \
    --sandbox --sandbox-backend "external:$work/openshell-guest" --report-out "$work/$3") > "$2" 2>&1
  echo "exit $? after $((SECONDS - t0)) s"
  cat "$2"
}

echo "-- (5a) undeclared: the guest must fail closed, as run 8 read it"
guest_run "" guest-undeclared.out report-undeclared.json
if grep -q "hello through the OpenShell guest" guest-undeclared.out; then
  fail "(5a) an UNDECLARED guest ran inside OpenShell without its own filter"
else
  grep -q "could not lock itself down" guest-undeclared.out || fail "(5a) the guest did not say why it refused"
  grep -q "never confirmed its boundary" guest-undeclared.out || fail "(5a) the host did not say the program was never sent"
  echo "(5a) held: undeclared, the guest failed closed and the program was never sent"
fi

echo "-- (5b) declared (--outer-syscall-filter): the guest runs under OpenShell's filter, at level 3"
guest_run "--outer-syscall-filter" guest.out report.json
grep -q "hello through the OpenShell guest" guest.out \
  || fail "(5b) the declared guest did not run the program inside OpenShell"
grep -q "refused by a filter already in force" guest.out || fail "(5b) the guest did not say whose filter is in force"
if [ -s report.json ]; then
  python3 - <<'PY' || fail "(5b) the report is not level 3, external, with the guest's outer-filter word"
import json
r = json.load(open("report.json"))
s = r["sandbox"]
print("report:", json.dumps({k: s.get(k) for k in ("level", "backend", "host_guarantees", "guest_reported", "properties")}, indent=1)[:2000])
assert s.get("level") == 3 and s.get("backend") == "external", s
words = s.get("guest_reported") or []
assert "an outer syscall filter, not its own" in words, words
assert not any(w in words for w in ("no new programs", "no sockets but the channel")), words
assert all(p["state"] == "unknown" for p in s["properties"].values()), s["properties"]
PY
else
  fail "(5b) no run report"
fi
openshell sandbox delete dlg >/dev/null 2>&1 || true

openshell sandbox delete dl >/dev/null 2>&1 || true
if [ "$failures" -gt 0 ]; then
  echo "openshell-runtime: $failures expectation(s) failed"
  exit 1
fi
echo "openshell-runtime: every expectation held"
