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
# checked by the prover against the export, so what OpenShell added is named, not assumed.
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

step "the gateway (a systemd user service the package installs)"
sudo loginctl enable-linger "$(id -un)"
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
for _ in $(seq 1 30); do [ -S "$XDG_RUNTIME_DIR/bus" ] && break; sleep 1; done
systemctl --user daemon-reload
systemctl --user enable --now openshell-gateway || fail "the gateway service did not start"
systemctl --user --no-pager status openshell-gateway | head -20 || true
up=0
for _ in $(seq 1 60); do
  if openshell status >/dev/null 2>&1; then up=1; break; fi
  sleep 2
done
if [ "$up" = 0 ]; then
  openshell gateway add https://127.0.0.1:17670 --local --name openshell || true
  openshell status || fail "the CLI cannot reach the gateway"
fi
openshell gateway list || true
docker version --format 'docker server {{.Server.Version}}' || true

step "the image: Debian, a non-root user, /sandbox, and this build's delulu with the program"
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
cat > ctx/Dockerfile <<'EOF'
FROM debian:bookworm-slim
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
openshell logs dl --tail 200 2>/dev/null | grep -iE 'deny|denied' | tail -10 || true

step "(4) the effective policy — what OpenShell added — against the export"
openshell sandbox get dl --policy-only > effective.yaml || fail "(4) no effective policy"
cat effective.yaml
openshell-prover check policy.yaml --boundary effective.yaml --output json 2>/dev/null
echo "export within the effective policy: exit $?"
openshell-prover check effective.yaml --boundary policy.yaml --output json 2>/dev/null
echo "effective within the export: exit $? (non-zero names what OpenShell added)"

openshell sandbox delete dl >/dev/null 2>&1 || true
if [ "$failures" -gt 0 ]; then
  echo "openshell-runtime: $failures expectation(s) failed"
  exit 1
fi
echo "openshell-runtime: every expectation held"
