#!/usr/bin/env python3
"""Run a set of source mutants against a test command and report each one — routine run 15's loop engineering.

Every routine run that falsifies a new test writes the same loop by hand, and the hand-written loops kept paying for
the same traps: a restored source with an OLD mtime (a `move` from a backup) leaves cargo's binary built from the LAST
mutant, so the "green again" run is red, or worse, a later by-hand check reads a mutant's answers (routine runs 8 and
15); a mutant that does not compile reads like a red witness (run 14); an anchor that matches twice mutates the wrong
place. This script refuses an anchor that is not unique, tells a build failure from a red test, restores each file
byte for byte with a NEW mtime, and ends with a CONTROL run of the same command on the restored tree, which must be
green — the proof that the restore took.

usage: scripts/mutants.py SPEC.json [NAME ...]
  SPEC.json: {"test": ["cargo", "test", "-q", "-p", "delulu", "--test", "monitor_cli"],
              "mutants": {"M1": {"file": "crates/…/x.rs", "old": "exact text, once", "new": "replacement",
                                 "test": [optional per-mutant command]},
                          "M2": {"edits": [{"file": …, "old": …, "new": …}, …], …}, …}}
             A mutant with `edits` makes several (every anchor checked before any file is written) — for a check
             built as two walls, where only removing BOTH shows the witness can fail at all (routine run 24).
  NAME …   : run only these mutants (default: all, in the file's order)
  MUTANTS_KEEP=DIR (environment): after each mutant's command, copy the `delulu` binary it built (MUTANTS_KEEP_BIN,
             default target/debug/delulu) to DIR/delulu-NAME, so a survivor can be probed by hand against the mutated
             code — never undo a mutant with `git stash` to build one (on-demand routine run 12: a `git stash drop` took
             the fix with the mutant).
Exit 0 when every mutant is RED and the control is green; 1 otherwise (a SURVIVED or BUILD-ERROR is a question about the
witness or the mutant — routine run 11: read what the mutated code returns before recording a survivor).
"""
import json, os, re, shutil, subprocess, sys


def no_fail_fast(cmd):
    """`cargo test` stops at the first test TARGET that fails, so a mutant red in one target never runs the witnesses in the
    next — routine run 16's M182 read one red witness of two. Every target must run under every mutant: add the flag."""
    if cmd[:2] == ['cargo', 'test'] and '--no-fail-fast' not in cmd:
        cut = cmd.index('--') if '--' in cmd else len(cmd)
        return cmd[:cut] + ['--no-fail-fast'] + cmd[cut:]
    return cmd


def run(cmd):
    cmd = no_fail_fast(cmd)
    r = subprocess.run(cmd, capture_output=True, text=True)
    out = r.stdout + r.stderr
    built = not ('error[E' in out or 'could not compile' in out)
    totals = re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed', out)
    failed = sorted(set(re.findall(r"thread '([^']+)' \(\d+\) panicked", out) + re.findall(r"thread '([^']+)' panicked", out)))
    return r.returncode, built, totals, failed


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    spec = json.load(open(sys.argv[1]))
    only = sys.argv[2:]
    names = [n for n in spec['mutants'] if not only or n in only]
    unknown = [n for n in only if n not in spec['mutants']]
    if unknown:
        print(f'error: no mutant named {unknown}')
        return 2
    ok = True
    for name in names:
        m = spec['mutants'][name]
        edits = m.get('edits') or [{'file': m['file'], 'old': m['old'], 'new': m['new']}]
        # Every anchor is checked against the files as they are before any is written (routine run 18's rule for patch
        # scripts): edits to one file apply in order, each to the text the one before it left.
        originals, texts, bad = {}, {}, None
        for e in edits:
            path = e['file']
            if path not in originals:
                originals[path] = open(path, 'rb').read()
                texts[path] = originals[path].decode('utf-8')
            n = texts[path].count(e['old'])
            if n != 1:
                bad = f'{name}: ANCHOR-NOT-UNIQUE ({n} matches in {path}) — not run'
                break
            texts[path] = texts[path].replace(e['old'], e['new'])
        if bad:
            print(bad)
            ok = False
            continue
        for path, text in texts.items():
            open(path, 'wb').write(text.encode('utf-8'))
        try:
            code, built, totals, failed = run(m.get('test', spec['test']))
            keep = os.environ.get('MUTANTS_KEEP')
            if keep and built:
                os.makedirs(keep, exist_ok=True)
                shutil.copy2(os.environ.get('MUTANTS_KEEP_BIN', 'target/debug/delulu'), os.path.join(keep, f'delulu-{name}'))
        finally:
            for path, original in originals.items():
                open(path, 'wb').write(original)
                os.utime(path, None)  # a NEW mtime, so cargo rebuilds from the restored source
        if not built:
            verdict = 'BUILD-ERROR (not a falsification — the mutant does not compile)'
            ok = False
        elif code == 0:
            verdict = 'SURVIVED'
            ok = False
        else:
            verdict = 'RED'
        print(f'{name}: {verdict} {totals} {failed}', flush=True)
    code, built, totals, failed = run(spec['test'])
    control = 'green' if code == 0 else f'RED {failed} — the restore did not take, or the tree was red before'
    print(f'CONTROL (the restored tree): {control} {totals}')
    return 0 if ok and code == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
