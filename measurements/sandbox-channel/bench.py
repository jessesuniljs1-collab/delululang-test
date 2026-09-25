"""PS-B-04's gate: what does one channel round trip cost? Measured, with the decision rule fixed first.

Rule (written before any number): batching PAYS only if the sandbox channel adds more than 50 us per
effect. Per-effect channel cost = [(clock_sbx - base_sbx) - (clock_l0 - base_l0)] / N, so interpreter
loop cost and every fixed cost (process start, check, guest launch) cancel out.
"""
import os, statistics, subprocess, sys, tempfile, time, json, platform

EXE = sys.argv[1]
REPS = 5
SIZES = [2000, 10000]

HEADER = "module bench\n\nfn main(root: Root) ! {Clock} {\n    let clk = root.clock()\n    var i = 0\n    var t = 0\n"
CLOCK = HEADER + "    while i < %d {\n        t = clk.now_ms()\n        i = i + 1\n    }\n}\n"
BASE = HEADER + "    while i < %d {\n        t = i\n        i = i + 1\n    }\n}\n"

work = tempfile.mkdtemp(prefix="delulu-chan-")
env = dict(os.environ, DELULU_NO_FIRST_RUN="1", DELULU_STATE_DIR=os.path.join(work, "state"))

def timed(src, sandbox):
    path = os.path.join(work, "b.delulu")
    open(path, "w").write(src)
    args = [EXE, "run", path, "--grant", "clock"] + (["--sandbox"] if sandbox else [])
    samples = []
    for _ in range(REPS):
        t0 = time.perf_counter()
        r = subprocess.run(args, capture_output=True, env=env)
        dt = time.perf_counter() - t0
        if r.returncode != 0:
            sys.exit(f"run failed ({args}): {r.stderr.decode(errors='replace')[:400]}")
        samples.append(dt)
    return statistics.median(samples), samples

results = {"machine": f"{platform.system()} {platform.release()} {platform.machine()}", "reps": REPS, "sizes": {}}
for n in SIZES:
    row = {}
    for name, src in (("clock", CLOCK % n), ("base", BASE % n)):
        for mode in ("l0", "sandbox"):
            med, samples = timed(src, mode == "sandbox")
            row[f"{name}_{mode}"] = {"median_s": round(med, 4), "samples_s": [round(s, 4) for s in samples]}
    per_effect_l0 = (row["clock_l0"]["median_s"] - row["base_l0"]["median_s"]) / n
    per_effect_sbx = (row["clock_sandbox"]["median_s"] - row["base_sandbox"]["median_s"]) / n
    row["per_effect_us_l0"] = round(per_effect_l0 * 1e6, 2)
    row["per_effect_us_sandbox"] = round(per_effect_sbx * 1e6, 2)
    row["channel_adds_us_per_effect"] = round((per_effect_sbx - per_effect_l0) * 1e6, 2)
    results["sizes"][str(n)] = row
    print(f"N={n}: L0 {row['per_effect_us_l0']} us/effect, sandbox {row['per_effect_us_sandbox']} us/effect, "
          f"channel adds {row['channel_adds_us_per_effect']} us/effect "
          f"(medians: clock_l0 {row['clock_l0']['median_s']}s base_l0 {row['base_l0']['median_s']}s "
          f"clock_sbx {row['clock_sandbox']['median_s']}s base_sbx {row['base_sandbox']['median_s']}s)")
print(json.dumps(results, indent=1))
