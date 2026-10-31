#!/usr/bin/env python3
"""Time the operator's e-stop (`grants revoke`) against a broker daemon, in the two shapes RW 4.53 named — on-demand
routine run 14 had to rebuild routine run 16's measurement from its record; this keeps both (Linux and macOS: the held
shape takes the chain's lock with `flock`, as `File::lock` does on Unix).

usage: scripts/estop-measure.py BIN WORKDIR contention LOOPS [SAMPLES] [plain]
       scripts/estop-measure.py BIN WORKDIR held QUEUED [HOLD_SECONDS]

  contention: LOOPS threads each run a program in a loop (`--sandbox`, whose host writes the audit chain; `plain`: plain
              runs, which write nothing — the control) while SAMPLES delegate→revoke pairs and daemon runs are timed.
              Prints median / p90 / max for each, the chain's length and `audit verify`'s exit.
  held:       another writer holds the chain's append lock for HOLD_SECONDS (default 40), QUEUED `grants delegate`s
              queue at the daemon, then the e-stop: prints when it answered, its exit and words, and the node's state.
              Since D-V2-198 the e-stop waits for the request in hand only (about 10 s on a held chain, 5 s of it its own
              record's wait); before, every queued request's 5 s (DL1401 at 15 s).

BIN is a `delulu` binary — copy it out of `target/` first, so a build during the measurement cannot change it. WORKDIR is
removed and made again; keep it short (`/tmp/…`): the broker's socket path is limited to about 104 bytes.
"""
import json, os, shutil, statistics, subprocess, sys, threading, time


def main():
    if len(sys.argv) < 5 or sys.argv[3] not in ("contention", "held"):
        print(__doc__)
        return 2
    binary, work, shape = sys.argv[1], sys.argv[2], sys.argv[3]
    shutil.rmtree(work, ignore_errors=True)
    os.makedirs(work)
    env = dict(os.environ, DELULU_STATE_DIR=os.path.join(work, "s"), DELULU_NO_FIRST_RUN="1")
    prog = os.path.join(work, "p.delulu")
    with open(prog, "w") as f:
        f.write('module m\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println("hi")\n}\n')

    def dl(*args, timeout=90):
        t = time.monotonic()
        p = subprocess.run([binary, *args], env=env, cwd=work, capture_output=True, text=True, timeout=timeout)
        return time.monotonic() - t, p

    def node():
        _, d = dl("grants", "delegate", "--effects", "Write", "--json")
        if d.returncode != 0:
            raise SystemExit(f"grants delegate failed: {d.stderr.strip()[-300:]}")
        return json.loads(d.stdout)["node"]

    started = dl("broker", "start")[1]
    if started.returncode != 0:
        raise SystemExit(f"broker start failed: {started.stderr.strip()[-300:]}")
    try:
        if shape == "contention":
            return contention(binary, work, env, prog, dl, node)
        return held(binary, work, env, dl, node)
    finally:
        dl("broker", "stop")


def summary(xs):
    if not xs:
        return "none"
    xs = sorted(xs)
    return (f"n={len(xs)} median={statistics.median(xs) * 1000:.0f} ms p90={xs[int(0.9 * (len(xs) - 1))] * 1000:.0f} ms "
            f"max={xs[-1] * 1000:.0f} ms")


def contention(binary, work, env, prog, dl, node):
    loops = int(sys.argv[4])
    samples = int(sys.argv[5]) if len(sys.argv) > 5 and sys.argv[5].isdigit() else 15
    plain = sys.argv[-1] == "plain"
    stop, runs_done = threading.Event(), [0]
    args = ["run", prog, "--grant", "console", "--no-prompt"] + ([] if plain else ["--sandbox"])

    def loop():
        while not stop.is_set():
            subprocess.run([binary, *args], env=env, cwd=work, capture_output=True)
            runs_done[0] += 1

    threads = [threading.Thread(target=loop, daemon=True) for _ in range(loops)]
    for t in threads:
        t.start()
    time.sleep(2)
    revokes, runs, failures = [], [], []
    for _ in range(samples):
        t, r = dl("grants", "revoke", node())
        (revokes.append(t) if r.returncode == 0 else failures.append(("revoke", round(t, 3), r.stderr.strip()[-200:])))
        t, r = dl("run", prog, "--broker", "daemon", "--grant", "console", "--no-prompt")
        (runs.append(t) if r.returncode == 0 else failures.append(("run", round(t, 3), r.stderr.strip()[-200:])))
    stop.set()
    for t in threads:
        t.join(timeout=60)
    audit = os.path.join(work, "s", "audit")
    records = sum(sum(1 for _ in open(os.path.join(audit, f))) for f in os.listdir(audit) if f.endswith(".jsonl"))
    verified = dl("audit", "verify")[1].returncode
    print(f"{loops} {'plain' if plain else 'sandboxed'} loops ({runs_done[0]} runs), a {records}-record chain, "
          f"audit verify exit {verified}")
    print("revoke:    ", summary(revokes))
    print("daemon run:", summary(runs))
    print("failures:  ", len(failures), failures[:3])
    return 1 if failures or verified != 0 else 0


def held(binary, work, env, dl, node):
    import fcntl
    queued = int(sys.argv[4])
    hold = float(sys.argv[5]) if len(sys.argv) > 5 else 40.0
    target = node()
    lock = open(os.path.join(work, "s", "audit", "append.lock"), "a+")
    fcntl.flock(lock, fcntl.LOCK_EX)
    release = threading.Timer(hold, lambda: fcntl.flock(lock, fcntl.LOCK_UN))
    release.start()
    waiting = [subprocess.Popen([binary, "grants", "delegate", "--effects", "Write"], env=env, cwd=work,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) for _ in range(queued)]
    time.sleep(0.2)
    t, r = dl("grants", "revoke", target)
    print(f"e-stop behind {queued} queued on a held chain: answered after {t:.2f} s, exit {r.returncode}")
    for line in (r.stdout + r.stderr).strip().splitlines():
        print("   ", line[:300])
    for p in waiting:
        p.communicate()
    release.join()
    time.sleep(0.5)
    _, ins = dl("grants", "inspect", target, "--json")
    try:
        state = json.loads(ins.stdout)["node"]
        print(f"the node afterwards: by_seq {state.get('by_seq')}, state {state.get('state', state.get('status'))}")
    except (ValueError, KeyError):
        print("the node afterwards:", ins.stdout.strip()[:300], ins.stderr.strip()[:300])
    return 0


if __name__ == "__main__":
    sys.exit(main())
