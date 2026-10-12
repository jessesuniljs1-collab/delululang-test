#!/usr/bin/env python3
"""What every program in the corpus asks for — routine run 19's loop engineering, for P9-01 (`authority --within`).

P9-01's witnesses are the corpus: each program `within` its own `--grants`, and `exceeds` each one-dimension narrowing.
Run 19 measured the corpus before designing them (`docs/DELULULANG_V2/V2_P9_DESIGN.md`), and its first read was vacuous:
a wrong JSON path made all 23 programs read as pure — an absence that proved nothing, caught only because programs that
plainly print said "no grants". This reads the authority report where it is (`authority.required_grants` in the envelope)
and carries its own CONTROL: it exits 1 when no program asks for anything, so a path that reads nothing cannot pass for a
corpus of pure programs again.

usage: scripts/authority-corpus.py [--bin PATH] [--json] [FILE_OR_PACKAGE …]
  default corpus: examples/*.delulu, examples/guide/*.delulu, docs/book/samples/*.delulu, and each package (a `delulu.toml`) in examples/
  each program is one of: pure (no grant), literal (every grant spelled from the source), or placeholder — split into
  grant DATA the program never requests (a secret's VALUE, a library's PATH, an actuator's DIM bounds: the request is
  the name, decidable) and a scope the source COMPUTES (fs PATH, net HOST, DEVICE: no static answer)
exit 0 when the read is real (at least one program asks for something), 1 when it is not, 2 on a bad invocation.
"""
import glob, json, os, re, subprocess, sys

DATA = {"VALUE", "LOGICAL", "DIM", "PATTERN"}
COMPUTED = {"PATH", "HOST", "DEVICE"}


def grant_class(grant):
    """`data`, `computed` or `literal` for one required grant line."""
    key, _, value = grant.partition("=")
    words = set(re.findall(r"\b[A-Z]{3,}\b", value))
    # `foreign.c=LIB:PATH` — the PATH is the binary the operator chooses: grant data, not a request.
    if key == "foreign.c" and words <= {"PATH", "LOGICAL"}:
        return "data" if "LOGICAL" not in value.split(":")[0] else "computed"
    # `actuator=NAME:DIM=lo..hi` — the bounds are the operator's envelope; only the device name is the request.
    if key in ("actuator", "compute") and value.split(":")[0] not in COMPUTED:
        return "data" if words else "literal"
    if words & COMPUTED:
        return "computed"
    return "data" if words & DATA else "literal"


def main():
    args = sys.argv[1:]
    exe, as_json, files = "target/debug/delulu", False, []
    while args:
        a = args.pop(0)
        if a == "--bin" and args:
            exe = args.pop(0)
        elif a == "--json":
            as_json = True
        elif a.startswith("-"):
            print(__doc__)
            return 2
        else:
            files.append(a)
    if not files:
        files = sorted(glob.glob("examples/*.delulu") + glob.glob("examples/guide/*.delulu") + glob.glob("docs/book/samples/*.delulu"))
        files += sorted(d for d in glob.glob("examples/*") if os.path.isfile(os.path.join(d, "delulu.toml")))
    if not os.path.exists(exe):
        print(f"error: no `{exe}` — `cargo build -p delulu` first, or --bin PATH", file=sys.stderr)
        return 2
    rows = []
    for f in files:
        r = subprocess.run([exe, "authority", f, "--json"], capture_output=True, text=True)
        try:
            grants = json.loads(r.stdout)["authority"]["required_grants"]
        except (ValueError, KeyError, TypeError):
            rows.append({"program": f, "error": (r.stderr or r.stdout).strip()[:200]})
            continue
        classes = sorted({grant_class(g) for g in grants})
        kind = "pure" if not grants else ("computed" if "computed" in classes else "data" if "data" in classes else "literal")
        rows.append({"program": f, "class": kind, "required_grants": grants})
    asks = [r for r in rows if r.get("required_grants")]
    if as_json:
        print(json.dumps({"programs": rows}, indent=1))
    else:
        for r in rows:
            if "error" in r:
                print(f"{r['program']:48} ERROR {r['error']}")
            else:
                print(f"{r['program']:48} {r['class']:9} {' '.join(r['required_grants']) or '-'}")
        counts = {k: sum(1 for r in rows if r.get("class") == k) for k in ("pure", "literal", "data", "computed")}
        print(f"\n{len(rows)} programs: " + ", ".join(f"{v} {k}" for k, v in counts.items()) +
              f", {sum(1 for r in rows if 'error' in r)} unreadable")
    if not asks:
        print("CONTROL FAILED: no program asks for anything — the read is vacuous, not a corpus of pure programs", file=sys.stderr)
        return 1
    return 1 if any("error" in r for r in rows) else 0


if __name__ == "__main__":
    sys.exit(main())
