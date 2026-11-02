#!/usr/bin/env python3
"""Summarize a CI job's log: the totals, every failure, every compiler warning, and the lines you name.

Routine run 12's loop engineering. The GitHub MCP tool `get_job_logs` saves an over-long log to a file — JSON whose
`logs_content` (or `logs[0].logs_content`) is the log — and reading it whole costs the context it was saved to spare.
This prints only what a reading records:

  - the summed `test result:` totals and how many test binaries reported;
  - each `FAILED` test and each panic, with the line after it (the assertion's message);
  - each compiler `warning:` with its location (run 12: an `unused_mut` that `-D warnings` would refuse, seen only in a
    Windows witness's build output);
  - every line holding one of PATTERNs (a test's name, `memory stop`, `##[notice]`);
  - with `--slowest N`, the N slowest tests, each timed as the gap between its result line's timestamp and the one
    before it (the `running N tests` line for a binary's first). True only where a job runs its tests one at a time, as
    Miri does (on-demand routine run 19: `miri-slow (delulu-broker)` ran 3 h 11 min, 8,119 s of it in one test, found by
    hand from the log's stamps).

usage: scripts/ci-log-summary.py LOG [--slowest N] [PATTERN ...]   (LOG: the saved JSON, or a plain log)
Exit status: 1 when a test failed or a warning was printed, else 0.
"""
import datetime
import json
import re
import sys

STAMP = re.compile(r"^﻿?\d{4}-\d\d-\d\dT[\d:.]+Z ")
COLOR = re.compile(r"\x1b\[[0-9;]*m")


def load(path):
    text = open(path, encoding="utf-8", errors="replace").read()
    try:
        d = json.loads(text)
    except ValueError:
        return text
    if isinstance(d, dict):
        if isinstance(d.get("logs_content"), str):
            return d["logs_content"]
        logs = d.get("logs")
        if isinstance(logs, list) and logs and isinstance(logs[0], dict):
            return "\n".join(l.get("logs_content", "") for l in logs)
    return text


def slowest(raw, n):
    """Print the n slowest tests: each test's result line's stamp against the stamp before it."""
    times, prev = [], None
    for l in raw:
        m = re.match(r"^\ufeff?(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{1,6})?)\d*Z (.*)$", COLOR.sub("", l.rstrip("\r")))
        if not m:
            continue
        at, text = datetime.datetime.fromisoformat(m[1]), m[2]
        if re.match(r"running \d+ tests?$", text):
            prev = at
        elif re.match(r"test .* \.\.\. (ok|FAILED|ignored)", text) and prev is not None:
            times.append(((at - prev).total_seconds(), text[:160]))
            prev = at
    for secs, text in sorted(times, reverse=True)[:n]:
        print(f"{secs:9.0f} s  {text}")
    print(f"slowest: {min(n, len(times))} of {len(times)} timed tests, {sum(s for s, _ in times):.0f} s in all")


def main(argv):
    args = argv[1:]
    top = 0
    if "--slowest" in args:
        k = args.index("--slowest")
        if k + 1 >= len(args) or not args[k + 1].isdigit():
            print("--slowest takes a count", file=sys.stderr)
            return 2
        top = int(args[k + 1])
        del args[k:k + 2]
    if not args:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    raw = load(args[0]).split("\n")
    lines = [COLOR.sub("", STAMP.sub("", l.rstrip("\r"))) for l in raw]
    patterns = args[1:]
    passed = failed = ignored = binaries = 0
    bad = False
    for i, l in enumerate(lines):
        m = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", l)
        if m:
            binaries += 1
            passed, failed, ignored = passed + int(m[1]), failed + int(m[2]), ignored + int(m[3])
            continue
        if re.match(r"test .* \.\.\. FAILED", l) or "panicked at" in l:
            bad = True
            print(f"{i}: {l[:300]}")
            if i + 1 < len(lines):
                print(f"{i + 1}:   {lines[i + 1][:300]}")
            continue
        if l.startswith("warning:") and "generated" not in l:
            bad = True
            where = next((x.strip() for x in lines[i + 1:i + 3] if "-->" in x), "")
            print(f"{i}: {l[:200]} {where}")
            continue
        if any(p in l for p in patterns):
            print(f"{i}: {l[:300]}")
    print(f"totals: {passed} passed, {failed} failed, {ignored} ignored, {binaries} binaries ({len(lines)} lines)")
    if top:
        slowest(raw, top)
    return 1 if bad or failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
