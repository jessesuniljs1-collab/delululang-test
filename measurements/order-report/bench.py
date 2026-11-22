# The order report, timed: the default engine (`delulu run`), the WASM engine and CPython on the same input.
#
#   python -I bench.py <delulu binary> <work dir outside the checkout> [--reps N] [--json results.json]
#                      [--built-from <commit the binary was built from>]
#
# Two suites. `report` is the gate (D-V2-238, fifth amendment): `programs/orders.delulu` and its twin
# `programs/orders.py`, unchanged, read a generated `data/orders.csv`, parse, group, sort and write
# `out/report.txt` -- a whole process each, start-up included. `micro` says where the time goes.
#
# A repetition counts only if it exited 0 and wrote exactly the expected bytes (the report) or printed exactly the
# expected line (micro). The expected value comes from one untimed run of the Python twin, and its SHA-256 is
# recorded. A lane with any failed repetition is reported FAILED with its reason and no time, and the harness then
# exits 1. Repetition r runs the lanes in an order rotated by r, so no lane always runs first or last. Every raw time
# is kept, in run order, in the JSON.
#
# The fixture is fixed: for n lines, random.Random(n) draws a customer from 60 keys (c00 to c59), an item from 9,
# a quantity from 1 to 8 and a price from 100 to 99,998 cents; every (n // 5)th line is replaced by `broken,row`,
# so 5 lines are refused and listed as skipped.
import hashlib
import json
import platform
import random
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROGRAMS = HERE / "programs"
SIZES = [1_000, 10_000, 100_000]
MICRO = ["startup", "loop10m", "fib30", "sort200k", "group200k", "fstr", "split", "parse", "mapins"]


def fixture(n):
    rnd = random.Random(n)
    rows = [
        f"c{rnd.randrange(60):02d}, item{rnd.randrange(9)}, {rnd.randrange(1, 9)}, {rnd.randrange(100, 99999)}"
        for _ in range(n)
    ]
    for k in range(0, n, max(1, n // 5)):
        rows[k] = "broken,row"
    return "\n".join(rows) + "\n"


def lanes(delulu, delulu_file, py_file, grants):
    return {
        "delulu run (default)": [delulu, "run", str(delulu_file), *grants, "--no-prompt"],
        "delulu run --engine wasm": [delulu, "run", str(delulu_file), "--engine", "wasm", *grants, "--no-prompt"],
        "python": [sys.executable, "-I", str(py_file)],
    }


def run(cmd, cwd):
    t = time.perf_counter()
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return time.perf_counter() - t, r


def first_line(r):
    text = (r.stderr or r.stdout or "").strip()
    return text.splitlines()[0][:160] if text else ""


def measure(cases, reps, produce):
    """cases: {lane: cmd}. produce(cmd) -> (seconds, ok, reason). Rotated order; every repetition checked."""
    names = list(cases)
    raw = {k: [] for k in names}
    failed = {}
    for r in range(reps):
        order = names[r % len(names):] + names[: r % len(names)]
        for lane in order:
            if lane in failed:
                continue
            secs, ok, reason = produce(cases[lane])
            if not ok:
                failed[lane] = f"repetition {r + 1}: {reason}"
                continue
            raw[lane].append(round(secs, 6))
    out = {}
    for lane in names:
        if lane in failed:
            out[lane] = {"status": "FAILED", "reason": failed[lane]}
        else:
            ts = raw[lane]
            out[lane] = {"status": "ok", "median_ms": round(statistics.median(ts) * 1000, 1),
                         "min_ms": round(min(ts) * 1000, 1), "max_ms": round(max(ts) * 1000, 1), "raw_s": ts}
    return out


def report_suite(delulu, work, reps):
    grants = ["--grant", "console", "--grant", "fs.read=./data", "--grant", "fs.write=./out"]
    cases = lanes(delulu, PROGRAMS / "orders.delulu", PROGRAMS / "orders.py", grants)
    results = {}
    for n in SIZES:
        (work / "data").mkdir(parents=True, exist_ok=True)
        (work / "data" / "orders.csv").write_text(fixture(n), encoding="utf-8", newline="\n")

        def fresh_out():
            shutil.rmtree(work / "out", ignore_errors=True)
            (work / "out").mkdir()

        fresh_out()
        _, ref = run(cases["python"], work)
        assert ref.returncode == 0, f"the Python twin failed on {n} lines: {first_line(ref)}"
        expected = (work / "out" / "report.txt").read_bytes()

        def produce(cmd):
            fresh_out()
            secs, r = run(cmd, work)
            if r.returncode != 0:
                return secs, False, f"exit {r.returncode}: {first_line(r)}"
            got = (work / "out" / "report.txt").read_bytes() if (work / "out" / "report.txt").is_file() else b""
            if got != expected:
                return secs, False, f"report differs from the expected bytes ({len(got)} vs {len(expected)} bytes)"
            return secs, True, ""

        results[str(n)] = {"expected_sha256": hashlib.sha256(expected).hexdigest(), "expected_bytes": len(expected),
                           "lanes": measure(cases, reps, produce)}
    return results


def micro_suite(delulu, work, reps):
    results = {}
    for name in MICRO:
        cases = lanes(delulu, PROGRAMS / "micro" / f"{name}.delulu", PROGRAMS / "micro" / f"{name}.py",
                      ["--grant", "console"])
        _, ref = run(cases["python"], work)
        assert ref.returncode == 0, f"the Python twin of {name} failed: {first_line(ref)}"
        expected = ref.stdout.strip()

        def produce(cmd):
            secs, r = run(cmd, work)
            if r.returncode != 0:
                return secs, False, f"exit {r.returncode}: {first_line(r)}"
            if r.stdout.strip() != expected:
                return secs, False, f"printed {r.stdout.strip()[:60]!r}, expected {expected!r}"
            return secs, True, ""

        results[name] = {"expected": expected, "lanes": measure(cases, reps, produce)}
    return results


def table(title, rows):
    print(f"\n{title}")
    for key, entry in rows.items():
        cells = []
        for lane, v in entry["lanes"].items():
            cells.append(f"{lane}: " + (f"{v['median_ms']} ms" if v["status"] == "ok" else f"FAILED ({v['reason']})"))
        print(f"  {key:>9}  " + " | ".join(cells))


def main():
    args = sys.argv[1:]
    if len(args) < 2:
        sys.exit(__doc__ if __doc__ else "usage: bench.py <delulu> <work dir> [--reps N] [--json FILE]")
    delulu, work = args[0], Path(args[1]).resolve()
    reps = int(args[args.index("--reps") + 1]) if "--reps" in args else 5
    out_json = Path(args[args.index("--json") + 1]) if "--json" in args else None
    # `delulu --version` does not name its commit yet (the roadmap's M4), so the caller says what it built.
    built_from = args[args.index("--built-from") + 1] if "--built-from" in args else "not given"
    if HERE in work.parents or work == HERE:
        sys.exit("the work directory must be outside the checkout")
    work.mkdir(parents=True, exist_ok=True)
    version = subprocess.run([delulu, "--version"], capture_output=True, text=True).stdout.strip()
    data = {
        "taken_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "machine": {"platform": platform.platform(), "processor": platform.processor(),
                    "python": platform.python_version(), "delulu": version, "delulu_built_from": built_from},
        "repetitions": reps,
        "report": report_suite(delulu, work, reps),
        "micro": micro_suite(delulu, work, reps),
    }
    table("report (lines)", data["report"])
    table("micro", data["micro"])
    if out_json:
        out_json.write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8", newline="\n")
    failed = any(v["status"] != "ok" for s in ("report", "micro") for e in data[s].values() for v in e["lanes"].values())
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
