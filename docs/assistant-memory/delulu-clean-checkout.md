---
name: delulu-clean-checkout
description: "Artifacts built from the working tree (core-invariance SNAPSHOT, the Survey map) must equal what a CLEAN CLONE produces — CRLF on disk and a gitignored file poisoned both until CI's first run"
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-09-14T10:55:56.476Z
---

CI's first run (2026-09-14, run `34830053479`) failed on ubuntu, windows and arm64 because two committed
artifacts had been generated from **this machine's disk**, not from what git stores. Every local run
passed, because Windows and WSL (`/mnt/d`) read the same disk.

1. **58 tracked files had CRLF in the working tree while the index held LF.** `.gitattributes`
   (`* text=auto eol=lf`) normalizes what is COMMITTED. It does nothing to a file a Windows tool
   writes (PowerShell, some editors), and `git status` stays clean. Three of those files were the
   DL0211/0212/0213 fixtures, so `tests/core-invariance/SNAPSHOT.txt` recorded byte offsets that
   counted CRs. CI's offsets were exactly 6/10/8 bytes smaller.
   - Detect: `git ls-files --eol | Select-String 'w/(crlf|mixed)'`.
   - Fix: rewrite a file only if its CRLF→LF form equals `git cat-file blob :path`, then re-bless.
   - After such a rewrite, `git status` shows ` M` while the diff is EMPTY: the index recorded the
     old size, and git reports a size change without re-hashing. `git add` re-hashes, and identical
     files drop out.
2. **The Survey mapped a gitignored file** (`editors/vscode/*.vsix`): 1120 nodes locally vs 1119 in
   any clone. Fixed by `EXCLUDED_FILE_EXTENSIONS` plus `crates/delulu-survey/tests/clean_checkout.rs`,
   which asks `git check-ignore`. The test was falsified first: it failed naming the .vsix before the
   fix.

**Why it matters:** a gate verified only against a non-reproducible tree is the "safe in the normal
case" shape again: green locally, red everywhere else.

**How to apply:** before trusting a green local run of anything that records bytes or walks the tree,
check `git ls-files --eol` for `w/crlf` and `git status --ignored`, or clone to a temp dir and run the
gate there.

See [[delulu-github-remote]], [[delulu-reproduce-the-shape]], [[delulu-survey-map]].
