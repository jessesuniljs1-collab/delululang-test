---
name: python-edit-eats-backslash
description: "Python str.replace via a Bash heredoc dropped a Rust string's backslash-newline continuation — PR #1 red on all 4 runners. Use Edit for Rust string literals; run message_spacing after any scripted .rs edit. 2026-10-23: it hit THREE more times in one session (a `\\` line continuation, `b\"\\r\\n\\r\\n\"` became raw CR LF bytes, `\\` in a format string) — through the Bash tool a doubled backslash arrives single before Python sees it, whatever the heredoc's quoting; use Edit or a Write-n .py file"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 0e8e52cb-64cc-4b7e-8b51-b9e94c1813f1
  modified: 2026-11-01T07:10:27.196Z
---

**2026-11-01: again** — a `python3 - <<'EOF'` patching a mutant script turned `'\\n'` into a real newline (SyntaxError,
caught before any repo edit). A QUOTED delimiter does not protect a doubled backslash inside a Python string that itself
writes code. Rust test code appended with `cat >> file <<'EOF'` kept its single `\n` escapes (verified with `cat -A`).
Rule stays: any Python that writes `\n`-bearing code → Write a .py file; never a heredoc.

**2026-10-29: twice more in one session** — both times a heredoc whose Python held `\n` or `\\` inside string literals (a test generator; a records patch) — failed only at its own `assert`, so nothing was half-written. Before typing `python - <<'EOF'` or `cat > x.py <<'EOF'`, check for ANY backslash in the body; if there is one, use Write for the .py file or Edit on the target. No exceptions.

A Python heredoc replacing a Rust literal turned a `\` + newline continuation into a real line break with indentation, so error text gained a run of spaces and `message_spacing` failed on every OS (PR #1, `5bb8ee7` fix).

**2026-10-23, three more in one session**: a `\` line continuation in a `format!` (diag.rs), `b"\r\n\r\n"` written as real CR LF bytes inside a byte string (egress tests), and a continuation in within.rs. In each, the escapes were already gone before Python ran, even inside `<<'PYEOF'`. Every one was caught, by a build failure, by `cat -A`, or by the lesson itself, but each cost a repair round.

**Why:** what the Bash tool passes in a heredoc is not byte-for-byte what was written: a `\\` arrives as one `\`. Python's `"""…"""` then reads `\` + newline as a line continuation and `\r\n` as control characters.

**How to apply:**
- For any `.rs` text holding a backslash (`\` continuation, `\n`, `\r`, `\"`, `\\`), use the **Edit** tool.
- If a script is needed, write it with the **Write** tool as a `.py` file and run that file. Never pass a heredoc through Bash. For byte-exact work, read and write the file in `rb`/`wb` and assert that no `\r` remains.
- After any scripted `.rs` edit: `grep -n '\\$'` / `cat -A` on the touched lines, and run `message_spacing`.

Related: [[a-gate-that-cannot-fail-is-not-a-gate]] (the CI gate caught it).
