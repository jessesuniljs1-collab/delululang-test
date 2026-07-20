#!/usr/bin/env python3
"""Measurement harness for the arm demonstration (Stage 10 §5.5, criterion 4).

    cargo build -p delulu --release
    python3 measurements/robotics-demo/measure.py [--reps 20]

Produces the numbers published in RECORD.md. This is the *measuring* half of the demonstration;
`run-demo.sh` is the *showing* half and needs only bash.

Criterion 4 requires three things measured — envelope refusal, heartbeat loss to fail-state, and
e-stop latency — plus §5.5's claim that correct edits "take effect at rate", which since 10g means
publishing what an actuator command actually costs when every one of them is a broker round-trip.

Method notes, because a number without its method is a rumor:

* Per-command costs are DIFFERENTIAL. `bench-none` runs the identical loop with the command
  removed; subtracting its wall clock cancels process startup, parse, check and interpreter
  overhead, which otherwise dwarf the thing being measured.
* The e-stop's end-to-end figure is measured from *this* harness's clock: the instant before
  `delulu grants revoke` is invoked, to the instant this harness reads the program's REVOKED line.
  It therefore includes the operator's process spawn, the IPC, the watchdog tick, the fail-state,
  and one command's worth of program-side slack. Every one of those is real latency an operator
  experiences. The sub-terms that can be isolated are reported separately so the budget is not one
  opaque number.
* Nothing here runs against hardware. The adapter is the in-tree simulator.
"""

import argparse
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent

DIMS = "angle_deg=-30..95,velocity_dps=0..40,torque_nm=0..2.5"
# The benchmark grant: no rate_hz (that would measure the rate limiter) and a heartbeat far longer
# than any run here (so the dead-man cannot fire and be mistaken for something else).
PATIENT = f"actuator=arm0/elbow:{DIMS},heartbeat_ms=600000,ttl_ms=600000,fail=safe-park"
# The wedged-controller grant: a 250 ms heartbeat, which the `fib` in arm-wedged.delulu misses.
NOMINAL = f"actuator=arm0/elbow:{DIMS},heartbeat_ms=250,ttl_ms=600000,fail=safe-park"

K = 2000  # commands per benchmark run; must match the loop bound in bench-*.delulu


def binary() -> Path:
    for name in ("delulu", "delulu.exe"):
        p = ROOT / "target" / "release" / name
        if p.exists():
            return p
    sys.exit("no release binary — run: cargo build -p delulu --release")


DELULU = None
ENV = dict(os.environ, DELULU_NO_FIRST_RUN="1")


def run(args, state=None, stderr_to=None, cwd=None):
    env = dict(ENV)
    if state:
        env["DELULU_STATE_DIR"] = str(state)
    err = open(stderr_to, "w", encoding="utf-8") if stderr_to else subprocess.PIPE
    try:
        return subprocess.run(
            [str(DELULU)] + args, env=env, cwd=cwd or ROOT,
            stdout=subprocess.PIPE, stderr=err, text=True, encoding="utf-8", errors="replace",
        )
    finally:
        if stderr_to:
            err.close()


def timed(args, state=None, stderr_to=None) -> float:
    """Wall clock of one full `delulu` invocation, in seconds."""
    t0 = time.perf_counter()
    r = run(args, state=state, stderr_to=stderr_to)
    dt = time.perf_counter() - t0
    if r.returncode != 0:
        sys.exit(f"run failed ({r.returncode}): {' '.join(args)}\n{r.stdout}")
    return dt


def stats(xs, unit):
    xs = sorted(xs)
    if not xs:
        return {"n": 0, "unit": unit}
    def pct(p):
        return xs[min(len(xs) - 1, int(round((len(xs) - 1) * p)))]
    return {
        "n": len(xs), "unit": unit,
        "min": round(xs[0], 3), "p50": round(statistics.median(xs), 3),
        "p95": round(pct(0.95), 3), "max": round(xs[-1], 3),
    }


def show(label, s):
    if not s.get("n"):
        print(f"  {label:<38} (no samples)")
        return
    u = s["unit"]
    print(f"  {label:<38} min {s['min']:>9} {u}   p50 {s['p50']:>9} {u}   "
          f"p95 {s['p95']:>9} {u}   max {s['max']:>9} {u}   n={s['n']}")


# ----- A/B: what a command costs ----------------------------------------------------------------

def command_cost(reps, mode, state=None):
    """Differential per-command cost, in microseconds, for accepted and refused commands."""
    base_args = ["--grant", "console", "--grant", PATIENT, "--broker-profile", "sim", "--no-prompt"]
    if mode == "daemon":
        base_args = ["--broker", "daemon"] + base_args
    accepted, refused = [], []
    for _ in range(reps):
        t_none = timed(["run", str(HERE / "bench-none.delulu")] + base_args, state)
        t_ok = timed(["run", str(HERE / "bench-ok.delulu")] + base_args, state)
        t_ref = timed(["run", str(HERE / "bench-refused.delulu")] + base_args, state)
        accepted.append((t_ok - t_none) / K * 1e6)
        refused.append((t_ref - t_none) / K * 1e6)
    return stats(accepted, "us"), stats(refused, "us")


# ----- C: heartbeat loss engages the fail-state --------------------------------------------------

OVERDUE = re.compile(r"beat overdue by (\d+)")
ENGAGE = re.compile(r"fail-state engaged (\d+)")


def heartbeat_loss(reps, tmp):
    overdue, engage = [], []
    for i in range(reps):
        log = tmp / f"wedged-{i}.err"
        run(["run", str(HERE / "arm-wedged.delulu"), "--grant", "console", "--grant", NOMINAL,
             "--broker-profile", "sim", "--trace-effects", "--no-prompt"], stderr_to=log)
        text = log.read_text(encoding="utf-8", errors="replace")
        m, n = OVERDUE.search(text), ENGAGE.search(text)
        if not (m and n):
            sys.exit(f"the wedged controller did not lose its arm — the measurement is void:\n{text[:800]}")
        overdue.append(int(m.group(1)) / 1000.0)  # us -> ms
        engage.append(float(n.group(1)))
    return stats(overdue, "ms"), stats(engage, "us")


# ----- D: the operator e-stop ---------------------------------------------------------------------

def device_node(state):
    """The arm's own grant node, found the way an operator finds it."""
    r = run(["grants", "list"], state=state)
    for line in r.stdout.splitlines():
        if "(device) arm0/elbow" in line and "[live]" in line:
            return line.split()[0]
    return None


def estop_once(state, tmp, i):
    errlog = tmp / f"estop-{i}.err"
    err = open(errlog, "w", encoding="utf-8")
    env = dict(ENV, DELULU_STATE_DIR=str(state))
    # stderr to a FILE, not a pipe: --trace-effects emits a record per command, and a pipe nobody
    # is draining fills and deadlocks the child at exactly the wrong moment.
    proc = subprocess.Popen(
        [str(DELULU), "run", str(HERE / "bench-supervisor.delulu"), "--broker", "daemon",
         "--grant", "console", "--grant", PATIENT, "--broker-profile", "sim",
         "--trace-effects", "--no-prompt"],
        env=env, cwd=ROOT, stdout=subprocess.PIPE, stderr=err,
        text=True, encoding="utf-8", errors="replace", bufsize=1)

    seen = {}
    def reader():
        for line in proc.stdout:
            line = line.strip()
            if line.startswith("REVOKED:") and "revoked" not in seen:
                seen["revoked"] = time.perf_counter()
            elif line == "UP" and "up" not in seen:
                seen["up"] = time.perf_counter()
    t = threading.Thread(target=reader, daemon=True)
    t.start()

    deadline = time.perf_counter() + 30
    node = None
    while time.perf_counter() < deadline:
        if "up" in seen:
            node = device_node(state)
            if node:
                break
        time.sleep(0.01)
    if not node:
        proc.kill(); err.close()
        return None

    t0 = time.perf_counter()
    run(["grants", "revoke", node], state=state)
    t_call = (time.perf_counter() - t0) * 1000.0

    proc.wait(timeout=60)
    t.join(timeout=5)
    err.close()
    if "revoked" not in seen:
        return None
    end_to_end = (seen["revoked"] - t0) * 1000.0
    m = ENGAGE.search(errlog.read_text(encoding="utf-8", errors="replace"))
    return {"call_ms": t_call, "end_to_end_ms": end_to_end,
            "engage_us": float(m.group(1)) if m else None}


def estop(reps, state, tmp):
    calls, e2e, engage = [], [], []
    for i in range(reps):
        r = estop_once(state, tmp, i)
        if not r:
            print(f"  (e-stop rep {i} produced no sample; skipped)")
            continue
        calls.append(r["call_ms"])
        e2e.append(r["end_to_end_ms"])
        if r["engage_us"] is not None:
            engage.append(r["engage_us"])
    return stats(calls, "ms"), stats(e2e, "ms"), stats(engage, "us")


def main():
    global DELULU
    ap = argparse.ArgumentParser()
    ap.add_argument("--reps", type=int, default=20)
    args = ap.parse_args()
    DELULU = binary()
    reps = args.reps

    tmp = Path(tempfile.mkdtemp(prefix="delulu-robotics-measure-"))
    out = {"reps": reps, "binary": str(DELULU), "profile": "sim (in-tree reference simulator)"}
    print(f"binary: {DELULU}\nreps:   {reps}\nscratch: {tmp}\n")

    print("A. what one command costs — embedded custody (no broker in the path)")
    ok, ref = command_cost(reps, "embedded")
    show("accepted command", ok); show("refused command (over-envelope)", ref)
    out["embedded_accepted"], out["embedded_refused"] = ok, ref

    state = tmp / "state"
    state.mkdir()
    run(["broker", "start"], state=state)
    try:
        print("\nB. what one command costs — daemon custody (a broker round-trip per command)")
        ok_d, ref_d = command_cost(reps, "daemon", state)
        show("accepted command", ok_d); show("refused command (over-envelope)", ref_d)
        out["daemon_accepted"], out["daemon_refused"] = ok_d, ref_d

        print("\nD. the operator e-stop: `grants revoke` on the arm's node")
        call, e2e, eng = estop(reps, state, tmp)
        show("`grants revoke` call returns", call)
        show("revoke issued -> program sees REVOKED", e2e)
        show("fail-state engaged (adapter)", eng)
        out["estop_call"], out["estop_end_to_end"], out["estop_engage"] = call, e2e, eng
    finally:
        run(["broker", "stop"], state=state)

    print("\nC. heartbeat loss -> fail-state (250 ms heartbeat, wedged controller)")
    over, eng2 = heartbeat_loss(reps, tmp)
    show("beat overdue when noticed", over); show("fail-state engaged (adapter)", eng2)
    out["heartbeat_overdue"], out["heartbeat_engage"] = over, eng2

    (HERE / "results.json").write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print(f"\nwrote {HERE / 'results.json'}")
    shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
