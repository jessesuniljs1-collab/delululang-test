#!/usr/bin/env bash
# PS-E-05 (a), D-V2-82 (routine run 8): DeluluLang's OpenShell export, checked by OpenShell's own prover.
#
# `delulu sandbox policy FILE --format openshell` derives the wall an OpenShell sandbox should put around
# `delulu run` from the program's authority and its grants. This script asks NVIDIA's independent SMT
# model the same question: given a boundary the operator wrote BY HAND (below — this repository's own
# text), does the export stay inside it? And it proves the question can be answered no: every mutation of
# the export that widens it (a preset that adds HEAD and OPTIONS, a writable /tmp, another binary, a host
# the boundary lacks) must come back `exceeds_boundary`, and a field the format does not have must be
# refused by the prover's parser — the same fail-closed parser OpenShell's runtime uses — so a
# `within_boundary` also says the document is OpenShell's format.
#
# Usage: scripts/openshell-prove.sh DELULU [PROVER]   (PROVER defaults to `openshell-prover` on PATH)
# It needs OpenShell's prover, which the VM cannot download (release assets are scoped to this
# repository): `.github/workflows/openshell.yml` installs a pinned, checksum-verified release on a runner,
# runs this, and keeps nothing. Exit 0 when every expectation held; 1 on the first that did not.
set -euo pipefail

DELULU="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
PROVER="${2:-openshell-prover}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cd "$work"

failures=0
fail() {
  echo "FAIL: $*"
  failures=$((failures + 1))
}

# expect WANT CANDIDATE BOUNDARY [PATTERN]: the prover's result is WANT, its exit code the documented one,
# and PATTERN (an extended regular expression: a host, a method, a path) matches what it said.
expect() {
  local want="$1" cand="$2" bound="$3" word="${4:-}" out code result
  set +e
  out="$("$PROVER" check "$cand" --boundary "$bound" --output json 2>&1)"
  code=$?
  set -e
  echo "-- $cand against $bound: exit $code"
  echo "$out"
  result="$(printf '%s' "$out" | python3 -c 'import json,sys
try: print(json.load(sys.stdin).get("result", "?"))
except Exception: print("unparsed")')"
  case "$want" in
    within_boundary) want_code=0 ;;
    exceeds_boundary) want_code=1 ;;
    error) want_code=2 ;;
    *) want_code=3 ;;
  esac
  [ "$result" = "$want" ] || fail "$cand vs $bound: result $result, expected $want"
  [ "$code" = "$want_code" ] || fail "$cand vs $bound: exit $code, expected $want_code"
  if [ -n "$word" ] && ! printf '%s' "$out" | grep -qE -- "$word"; then
    fail "$cand vs $bound: the answer does not name $word"
  fi
  if [ "$want" = within_boundary ]; then
    for d in filesystem network_l4 network_rest process landlock; do
      printf '%s' "$out" | grep -q "$d" || fail "$cand vs $bound: coverage lacks $d"
    done
  fi
}

# A mutated copy of FILE: python3 replaces OLD (exactly once) with NEW.
mutate() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
src, dst, old, new = sys.argv[1:5]
s = open(src).read()
assert s.count(old) == 1, f"the mutation's anchor is not in {src} exactly once: {old!r}"
open(dst, "w").write(s.replace(old, new))
PY
}

# ---- Program A: files and two exact hosts (the prover declines an exact and a wildcard host on one port).
cat > a.delulu <<'EOF'
module a

fn main(root: Root) ! {Read, Write, Net} {
    let src = root.fs_read("./data")
    let out = root.fs_write("./out")
    let h = root.http(["api.example.com", "data.example.net"])
    let t = src.read_text("./data/in.txt")
    let r = h.get("https://api.example.com/v1")
    let _ = out.write_text("./out/x.txt", "hi")
}
EOF
"$DELULU" sandbox policy a.delulu --format openshell --workdir /sandbox \
  --grant fs.read=./data --grant fs.write=./out --grant net=api.example.com --grant net=data.example.net \
  --grant console > a.yaml
echo "== the export (A)"
cat a.yaml

# What the operator allows program A — written by hand, not by delulu.
boundary_a() {
  local hosts="$1"
  cat <<EOF
version: 1
filesystem_policy:
  include_workdir: false
  read_only: [/usr, /lib, /etc, /sandbox/data, /sandbox/a.delulu]
  read_write: [/sandbox/out]
landlock:
  compatibility: hard_requirement
process:
  run_as_user: sandbox
  run_as_group: sandbox
network_policies:
EOF
  local i=0
  for h in $hosts; do
    i=$((i + 1))
    cat <<EOF
  allowed_$i:
    endpoints:
      - host: $h
        port: 443
        protocol: rest
        enforcement: enforce
        rules:
          - allow: { method: GET, path: "/" }
          - allow: { method: GET, path: "/**" }
    binaries:
      - path: /usr/local/bin/delulu
EOF
  done
}
boundary_a "api.example.com data.example.net" > boundary-a.yaml
boundary_a "api.example.com" > boundary-a-one-host.yaml

expect within_boundary a.yaml boundary-a.yaml
# The falsifier the study names: a boundary missing one granted host, and the host is the counterexample.
expect exceeds_boundary a.yaml boundary-a-one-host.yaml data.example.net

# Each widening of the export must be caught.
rules_block='        rules:
          - allow:
              method: "GET"
              path: "/"
          - allow:
              method: "GET"
              path: "/**"
    binaries:
      - path: "/usr/local/bin/delulu"
  "delulu_https_2":'
mutate a.yaml a-preset.yaml "$rules_block" '        access: read-only
    binaries:
      - path: "/usr/local/bin/delulu"
  "delulu_https_2":'
expect exceeds_boundary a-preset.yaml boundary-a.yaml 'HEAD|OPTIONS'
mutate a.yaml a-tmp.yaml '  read_write:
    - "/sandbox/out"' '  read_write:
    - "/sandbox/out"
    - "/tmp"'
expect exceeds_boundary a-tmp.yaml boundary-a.yaml /tmp
sed 's#      - path: "/usr/local/bin/delulu"#      - path: "/usr/bin/curl"#' a.yaml > a-curl.yaml
grep -q '/usr/bin/curl' a-curl.yaml || fail "the binary mutation did not land"
expect exceeds_boundary a-curl.yaml boundary-a.yaml /usr/bin/curl
# The parser is fail-closed: a field the format does not have is an error, not ignored.
mutate a.yaml a-unknown.yaml 'landlock:
  compatibility: "hard_requirement"' 'landlock:
  compatibility: "hard_requirement"
  delulu_extra: true'
expect error a-unknown.yaml boundary-a.yaml

# ---- Program B: one wildcard host — DeluluLang's `*.cdn.example.org`, OpenShell's `**.cdn.example.org`.
cat > b.delulu <<'EOF'
module b

fn main(root: Root) ! {Net} {
    let h = root.http(["*.cdn.example.org"])
    let r = h.get("https://img.cdn.example.org/logo")
}
EOF
"$DELULU" sandbox policy b.delulu --format openshell --workdir /sandbox --grant 'net=*.cdn.example.org' > b.yaml
echo "== the export (B)"
cat b.yaml
cat > boundary-b.yaml <<'EOF'
version: 1
filesystem_policy:
  include_workdir: false
  read_only: [/usr, /lib, /etc, /sandbox/b.delulu]
  read_write: []
landlock:
  compatibility: hard_requirement
process:
  run_as_user: sandbox
  run_as_group: sandbox
network_policies:
  cdn:
    endpoints:
      - host: "**.cdn.example.org"
        port: 443
        protocol: rest
        enforcement: enforce
        rules:
          - allow: { method: GET, path: "/" }
          - allow: { method: GET, path: "/**" }
    binaries:
      - path: /usr/local/bin/delulu
EOF
expect within_boundary b.yaml boundary-b.yaml
sed 's#^network_policies:$#network_policies: {}#; /^  cdn:/,$d' boundary-b.yaml > boundary-b-none.yaml
expect exceeds_boundary b.yaml boundary-b-none.yaml cdn.example.org

if [ "$failures" -gt 0 ]; then
  echo "openshell-prove: $failures expectation(s) failed"
  exit 1
fi
echo "openshell-prove: every expectation held"
