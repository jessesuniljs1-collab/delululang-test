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
# `openshell sandbox exec`, the program run end to end at level 3.
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
    match src.read_text("./data/in.txt") {
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

step "(2) a program asking for more than its grants is refused by DeluluLang"
out="$(openshell sandbox exec -n dl --no-tty --no-login-shell -- /bin/sh -c \
  'cd /sandbox && /usr/local/bin/delulu run greedy.delulu --no-prompt --grant console --grant fs.read=./data' 2>&1)"
echo "$out"
printf '%s' "$out" | grep -q "DL0703" || fail "(2) the greedy program was not refused with DL0703"

step "(3) curl, a binary the policy never names, to a host it never names, is denied by OpenShell"
out="$(openshell sandbox exec -n dl --no-tty --no-login-shell -- curl -sS -m 20 -o /dev/null -w '%{http_code}' https://example.org/ 2>&1)"
code=$?
echo "exit $code: $out"
[ "$code" != 0 ] || fail "(3) curl reached example.org"
# OpenShell's own account of the refusal (`--tail` streams, so a bounded window is read instead).
echo "OpenShell's log for this sandbox, the last 30 lines (run 6 matched no deny line; the format is read here):"
timeout 30 openshell logs dl --since 10m 2>&1 | tail -30 || echo "(no log read)"

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
# The launcher: the channel on the exec's standard streams; the guest's words from the host (no shell word-splits
# a token — DELULU_GUEST_ARGS is DeluluLang's own, `__guest --stdio-pipes`).
cat > openshell-guest <<'EOF'
#!/bin/sh
exec openshell sandbox exec -n dlg --no-tty --no-login-shell -- /usr/local/bin/delulu $DELULU_GUEST_ARGS
EOF
chmod +x openshell-guest
mkdir -p host/data
echo "hello through the OpenShell guest" > host/data/in.txt
cp ctx/p.delulu host/
(cd host && "$DELULU" run p.delulu --no-prompt --grant console --grant fs.read=./data \
  --sandbox --sandbox-backend "external:$work/openshell-guest" --report-out "$work/report.json") > guest.out 2>&1
echo "exit $?"
cat guest.out
grep -q "hello through the OpenShell guest" guest.out || fail "(5) the program did not run through the OpenShell guest"
if [ -s report.json ]; then
  python3 - <<'PY' || fail "(5) the report does not say level 3, external"
import json
r = json.load(open("report.json"))
s = r["sandbox"]
print("report:", json.dumps({k: s.get(k) for k in ("level", "backend", "host_guarantees", "properties")}, indent=1)[:1500])
assert s.get("level") == 3 and s.get("backend") == "external", s
PY
else
  fail "(5) no run report"
fi
openshell sandbox delete dlg >/dev/null 2>&1 || true

openshell sandbox delete dl >/dev/null 2>&1 || true
if [ "$failures" -gt 0 ]; then
  echo "openshell-runtime: $failures expectation(s) failed"
  exit 1
fi
echo "openshell-runtime: every expectation held"
