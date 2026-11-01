#!/usr/bin/env python3
"""Each test job's `actors_pingpong` verdict and its runner's disk, for one CI run, in one line a job (on-demand routine
run 16, loop engineering).

docs/CLOUD_ROUTINE.md steps 3 and 7 record, for every push run and `everything` run read, each OS's `actors_pingpong`
verdict (D-V2-47) and the RUNNER-DISK-1 notices. Both are check-run ANNOTATIONS, which the REST API answers without any
log (run 9); run 16 read them for seven runs with a hand-written loop, and its first form matched the wrong field (the
verdict is in the annotation's title, not its message) and printed nothing for every job — so this prints a job with no
verdict as `no verdict`, never as nothing.

usage: scripts/ci-annotations.py RUN_ID [RUN_ID …]
  For each run: its conclusion and job totals, then per test job (`test (…)` and `arm64`): `pingpong:` the verdict with
  its numbers, `disk:` before → after. Reads https://api.github.com through the session's proxy; needs no token.
"""
import json
import subprocess
import sys

API = "https://api.github.com/repos/jessesuniljs1-collab/delululang-test"


def get(url):
    out = subprocess.run(["curl", "-sf", url], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"ci-annotations.py: could not read {url} (curl exit {out.returncode})")
    return json.loads(out.stdout)


def disk(msg):
    # "82G free of 145G; target/ 2.9G" -> "82G"
    return msg.split(" free", 1)[0]


def one(run_id):
    run = get(f"{API}/actions/runs/{run_id}")
    jobs = get(f"{API}/actions/runs/{run_id}/jobs?per_page=100")["jobs"]
    totals = {}
    for j in jobs:
        totals[j["conclusion"] or j["status"]] = totals.get(j["conclusion"] or j["status"], 0) + 1
    print(f"run {run_id} ({run['name']}, {run['head_branch']} {run['head_sha'][:7]}): {run['status']} "
          f"{run['conclusion']} — " + ", ".join(f"{n} {k}" for k, n in sorted(totals.items())))
    for j in sorted(jobs, key=lambda j: j["name"]):
        if not (j["name"].startswith("test") or j["name"] == "arm64"):
            continue
        verdict, before, after = "no verdict", "?", "?"
        for a in get(f"{API}/check-runs/{j['id']}/annotations"):
            title, msg = a.get("title") or "", " ".join(a["message"].split())
            if title.startswith("actors_pingpong"):
                at = msg.find("speedup ")
                verdict = msg[at + len("speedup "):][:110] if at >= 0 else msg[:110]
            elif title.startswith("disk before"):
                before = disk(msg)
            elif title.startswith("disk after"):
                after = disk(msg)
        print(f"  {j['name']:<22} {j['conclusion']}: pingpong: {verdict} | disk: {before} -> {after}")


def main():
    if len(sys.argv) < 2 or not all(a.isdigit() for a in sys.argv[1:]):
        print(__doc__)
        return 2
    for run_id in sys.argv[1:]:
        one(run_id)
    return 0


if __name__ == "__main__":
    sys.exit(main())
