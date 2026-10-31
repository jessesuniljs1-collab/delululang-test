#!/usr/bin/env python3
"""Fold `master`'s new CHANGELOG sections into `launch/2.0.0`'s `## [2.0.0]` (release procedure step 1).

`master` writes each change as a `## Unreleased — TITLE` section, newest first. On `launch/2.0.0` every one of them
lives under `## [2.0.0]` as `### TITLE`, in the same order. So a merge of `master` into `launch/2.0.0` conflicts in
`CHANGELOG.md` every time. This script resolves it the one way it is resolved: it keeps the launch side
(`HEAD:CHANGELOG.md`), finds each of `master`'s `## Unreleased` sections whose title is not yet a `###` there, and inserts
those sections, as `###`, above the first `###` under `## [2.0.0]`.

It writes nothing unless every check holds: the sections it adds all sit above every section already folded, in
`master`'s own order, and the launch side has its `## [2.0.0]` heading and a `###` below it. It does not decide the
2.0.0 `**Breaking**` paragraph: read each section it lists, and add the ones that can stop a 1.x program, driver or
deployment by hand — every section that moves a surface `docs/design/STABILITY.md` §1 holds stable (a refused program's
code, a command's exit code for an outcome, a JSON field's type or meaning, syntax that parsed) among them (on-demand
routine run 9: C9 moved a refusal's code and C11 an exit code, each a refusal still). (On-demand routine run 8,
2026-10-30. The laptop had a script of the same purpose, never committed; each merge before this one wrote it again.)

usage (inside the launch worktree, during `git merge --no-ff origin/master`):
    python3 -I scripts/launch-changelog.py [SOURCE_REF]      # SOURCE_REF defaults to origin/master
    git add CHANGELOG.md
"""
import re
import subprocess
import sys

UNRELEASED = re.compile(r"^## Unreleased — (.+)$")


def show(ref):
    return subprocess.run(["git", "show", ref + ":CHANGELOG.md"], capture_output=True, text=True,
                          encoding="utf-8", check=True).stdout


def main():
    source = sys.argv[1] if len(sys.argv) > 1 else "origin/master"
    launch = show("HEAD")
    master = show(source)

    folded = set(re.findall(r"^### (.+)$", launch, re.M))
    lines = master.split("\n")
    heads = [i for i, line in enumerate(lines) if line.startswith("## ")]
    new, first_folded = [], None
    for k, i in enumerate(heads):
        m = UNRELEASED.match(lines[i])
        if not m:
            continue
        if m.group(1) in folded:
            if first_folded is None:
                first_folded = i
            continue
        if first_folded is not None:
            sys.exit(f"refused: `{m.group(1)}` is below a section already folded — fold by hand")
        end = heads[k + 1] if k + 1 < len(heads) else len(lines)
        new.append((m.group(1), lines[i + 1:end]))

    if not new:
        print(f"nothing to fold: every `## Unreleased` section of {source} is already a `###` on HEAD")
        return

    out = launch.split("\n")
    release = [i for i, line in enumerate(out) if line.startswith("## [2.0.0]")]
    if len(release) != 1:
        sys.exit(f"refused: HEAD's CHANGELOG has {len(release)} `## [2.0.0]` headings, not one")
    anchor = next((i for i in range(release[0] + 1, len(out)) if out[i].startswith("### ")), None)
    if anchor is None or any(out[i].startswith("## ") for i in range(release[0] + 1, anchor)):
        sys.exit("refused: no `###` section under `## [2.0.0]` to insert above")

    block = []
    for title, body in new:
        block.append("### " + title)
        block.extend(body)
    with open("CHANGELOG.md", "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out[:anchor] + block + out[anchor:]))

    print(f"folded {len(new)} section(s) of {source} into `## [2.0.0]`:")
    for title, _ in new:
        print("  ### " + title)
    print("Now judge the `**Breaking**` paragraph against each, by hand: a section that moves a surface STABILITY.md §1")
    print("holds stable (a refused program's code, an exit code, a JSON field's type or meaning, syntax that parsed) is one.")


if __name__ == "__main__":
    main()
