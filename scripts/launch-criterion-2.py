#!/usr/bin/env python3
"""Measure CHECKLIST-2.0's criterion 2 on the launch tree and write its line (release procedure step 1).

Every merge of `master` into `launch/2.0.0` (docs/CLOUD_ROUTINE.md step 4.3) re-measures criterion 2 — conformance:
every anchor covered, the reference in sync, the samples formatted — and appends one sentence to the criterion's row in
`docs/release/CHECKLIST-2.0.md`. On-demand routine runs 8, 9, 11, 13 and 15 each ran the three commands and wrote that
sentence by hand, and since run 13 `scripts/launch-gates.sh`'s `criterion-2-cited` greps it back
(`` `HASH`: **N/N (100%)** ``): a sentence in another shape is a red gate. This runs the three, refuses to write anything
unless all three are clean, and writes the sentence in the one shape the gate reads (on-demand routine run 15).

usage (inside the launch worktree, after the merge's last edit to Rust or to conformance inputs, BEFORE
`scripts/launch-gates.sh`):
    python3 -I scripts/launch-criterion-2.py LABEL [BASE]   # LABEL: who measured, e.g. "on-demand routine run 15";
                                                            # BASE: the ref merged in, default origin/master
    python3 -I scripts/launch-criterion-2.py --dry-run LABEL [BASE]   # measure and print the sentence; write nothing
    python3 -I scripts/launch-criterion-2.py --self-test              # the parsers against canned outputs; no cargo
    git add docs/release/CHECKLIST-2.0.md

Exit 0: written (or already cited at this count for BASE — nothing to write). Exit 1: a measurement is not clean, or the
row is not where it was. Exit 2: a bad invocation.
"""
import datetime
import re
import subprocess
import sys

CHECKLIST = "docs/release/CHECKLIST-2.0.md"
ROW = re.compile(r"^\| 2 \|.*\|(?=\r?$)", re.M)  # newline="" below: a CRLF checkout keeps its CRLF
KINDS = ["diag", "prim", "grammar", "audit", "cli", "rule"]


def parse_coverage(out):
    """`delulu-conform --coverage`'s report → (covered, anchors, "diag 155, prim 86, …"); refuses anything under 100%."""
    m = re.search(r"^anchors: (\d+) \| covered \(pos\+neg\): (\d+) \| coverage: ([0-9.]+)%", out, re.M)
    if not m:
        raise ValueError("no `anchors: N | covered (pos+neg): M` line in the coverage report")
    anchors, covered = int(m.group(1)), int(m.group(2))
    if covered != anchors:
        raise ValueError(f"coverage is {covered}/{anchors}, not every anchor")
    parts, total = [], 0
    for kind in KINDS:
        k = re.search(rf"^\s+{kind}\s+(\d+)/(\d+)\s*$", out, re.M)
        if not k:
            raise ValueError(f"no `{kind}` line in the coverage report")
        if k.group(1) != k.group(2):
            raise ValueError(f"`{kind}` is {k.group(1)}/{k.group(2)}")
        parts.append(f"{kind} {k.group(1)}")
        total += int(k.group(2))
    if total != anchors:
        raise ValueError(f"the kinds ({', '.join(parts)}) do not add up to {anchors} anchors — a new kind?")
    if "PASS: 100% anchor coverage." not in out:
        raise ValueError("the coverage report does not say `PASS: 100% anchor coverage.`")
    return covered, anchors, ", ".join(parts)


def parse_reference(out):
    m = re.search(r"^reference: in sync with the compiler source \((\d+) chapters\)$", out, re.M)
    if not m:
        raise ValueError("`--check-reference` does not say the reference is in sync")
    return int(m.group(1))


def parse_fmt(out):
    m = re.search(r"^fmt --check: (\d+) would change, (\d+) clean, (\d+) refused", out, re.M)
    if not m:
        raise ValueError("no `fmt --check: N would change, M clean, K refused` line")
    change, clean, refused = (int(g) for g in m.groups())
    if change or refused:
        raise ValueError(f"`fmt --check`: {change} would change, {refused} refused")
    return clean


def sentence(day, label, base7, covered, anchors, kinds, chapters, clean):
    return (f"Again {day} ({label}), on the tree merged with `master` `{base7}`: **{covered}/{anchors} (100%)** — {kinds}; "
            f"`--check-reference` in sync ({chapters} chapters); `fmt --check` 0 would change, {clean} clean.")


def run(cmd):
    p = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    out = p.stdout + p.stderr
    if p.returncode != 0:
        raise ValueError(f"`{' '.join(cmd)}` exited {p.returncode}:\n{out[-2000:]}")
    return out


def self_test():
    cov = ("anchors: 358 | covered (pos+neg): 358 | coverage: 100.0%\n  diag     155/155\n  prim      86/86 \n"
           "  grammar   27/27 \n  audit      7/7  \n  cli       29/29 \n  rule      54/54 \n\nPASS: 100% anchor coverage.\n")
    assert parse_coverage(cov) == (358, 358, "diag 155, prim 86, grammar 27, audit 7, cli 29, rule 54")
    refused = [
        cov.replace("covered (pos+neg): 358", "covered (pos+neg): 357"),  # an anchor uncovered
        cov.replace("86/86", "85/86"),                                      # one kind short
        cov.replace("  rule      54/54 \n", ""),                            # a kind missing
        cov.replace("anchors: 358 | covered (pos+neg): 358", "anchors: 359 | covered (pos+neg): 359"),  # a new kind
        cov.replace("PASS: 100% anchor coverage.", "FAIL"),
    ]
    for bad in refused:
        try:
            parse_coverage(bad)
        except ValueError:
            continue
        raise AssertionError(f"accepted a coverage report it must refuse:\n{bad}")
    assert parse_reference("reference: in sync with the compiler source (24 chapters)\n") == 24
    for bad in ["reference: OUT OF SYNC (2 chapters differ)\n", ""]:
        try:
            parse_reference(bad)
        except ValueError:
            continue
        raise AssertionError(f"accepted a reference check it must refuse: {bad!r}")
    assert parse_fmt("fmt --check: 0 would change, 26 clean, 0 refused (parse errors)\n") == 26
    for bad in ["fmt --check: 1 would change, 25 clean, 0 refused (parse errors)\n",
                "fmt --check: 0 would change, 25 clean, 1 refused (parse errors)\n", ""]:
        try:
            parse_fmt(bad)
        except ValueError:
            continue
        raise AssertionError(f"accepted a fmt check it must refuse: {bad!r}")
    s = sentence("2026-10-31", "on-demand routine run 15", "19629a7", 358, 358,
                 "diag 155, prim 86, grammar 27, audit 7, cli 29, rule 54", 24, 26)
    # the shape `scripts/launch-gates.sh`'s criterion-2-cited reads back
    assert re.search(r"`[0-9a-f]{7,40}`: \*\*358/358 \(100%\)\*\*", s), s
    print("launch-criterion-2.py --self-test: ok (1 coverage report accepted, 5 refused; reference 1/2; fmt 1/3)")


def main(argv):
    if argv[1:] == ["--self-test"]:
        self_test()
        return 0
    args = argv[1:]
    dry = bool(args) and args[0] == "--dry-run"
    if dry:
        args = args[1:]
    if not 1 <= len(args) <= 2 or args[0].startswith("-"):
        print(__doc__.split("\n\n")[2], file=sys.stderr)
        return 2
    label, base = args[0], (args[1] if len(args) == 2 else "origin/master")
    branch = subprocess.run(["git", "rev-parse", "--abbrev-ref", "HEAD"], capture_output=True, text=True).stdout.strip()
    if not branch.startswith("launch/"):
        print(f"launch-criterion-2.py: HEAD is `{branch}`, not a launch/* branch — run it in the launch worktree",
              file=sys.stderr)
        return 2
    base7 = subprocess.run(["git", "rev-parse", "--short=7", base + "^{commit}"], capture_output=True, text=True)
    if base7.returncode != 0:
        print(f"launch-criterion-2.py: no such ref `{base}`", file=sys.stderr)
        return 2
    base7 = base7.stdout.strip()
    try:
        covered, anchors, kinds = parse_coverage(run(["cargo", "run", "-q", "-p", "delulu-conform", "--", "--coverage"]))
        chapters = parse_reference(run(["cargo", "run", "-q", "-p", "delulu-conform", "--", "--check-reference"]))
        clean = parse_fmt(run(["cargo", "run", "-q", "-p", "delulu", "--", "fmt", "--check", "examples",
                               "docs/book/samples"]))
    except ValueError as e:
        print(f"criterion 2 is NOT met on this tree — nothing written: {e}")
        return 1
    day = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    line = sentence(day, label, base7, covered, anchors, kinds, chapters, clean)
    text = open(CHECKLIST, encoding="utf-8", newline="").read()
    rows = ROW.findall(text)
    if len(rows) != 1:
        print(f"{CHECKLIST}: {len(rows)} criterion-2 rows (`| 2 | … |`), expected one — nothing written")
        return 1
    row = rows[0]
    cited = f"`{base7}`: **{covered}/{anchors} (100%)**" in row
    print(line)
    if cited:
        print(f"already cited: the row names `{base7}` at {covered}/{anchors} — nothing to write")
        return 0
    if dry:
        print("(--dry-run: nothing written)")
        return 0
    if not row.endswith(" |"):
        print(f"{CHECKLIST}: the criterion-2 row does not end in ` |` — nothing written")
        return 1
    text = text.replace(row, row[:-2] + " " + line + " |", 1)
    open(CHECKLIST, "w", encoding="utf-8", newline="").write(text)
    print(f"written into {CHECKLIST}'s criterion-2 row — `git add` it, then run scripts/launch-gates.sh")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
