---
name: table-row-is-one-line
description: "Never wrap an edit inside a Markdown table row (REMAINING_WORK.md) — a row is one line; 7 rows were broken, 2 by my own wrapped edits (2026-10-22); gated now by repository_structure every_remaining_work_row_is_one_line"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-22T16:13:16.786Z
---

When editing a `docs/REMAINING_WORK.md` row (or any Markdown table row), keep the inserted text on the SAME line — no
newline inside the cell, however long the line gets. On 2026-10-22 my RW 4.17 and 7.17 edits wrapped at ~120 columns
like prose, and five older rows (4.58, 4.59, 4.53, 4.46, 5.6) had the same break from earlier sessions; each rendered as
a stub row plus loose paragraphs.

**Why:** a broken row silently mangles the gap list every session plans from; nothing caught it for weeks.

**How to apply:** an Edit's new_string inside a table row has no `\n`. The gate `repository_structure.rs`
`every_remaining_work_row_is_one_line` (PR #45, `a47516c`) now fails on it; the joiner script that fixed them compared
the file's word sequence before/after. Related: [[python-edit-eats-backslash]].

**Same night, same class (CLOUD_SYNC_LOG.md):** anchoring a new entry's insert on a line in the MIDDLE of the previous entry
(its **What:** paragraph) put the new entry ahead of that entry's `- Files and folders` / `- Survey and doctor` lines — three
entries scrambled, landed on master, fixed in PR #47 by moving the blocks back (sorted-lines check: nothing changed but order).
**How to apply:** for an append, anchor on the previous entry's final line (`- Survey and doctor (after the last edit): in the
commit message.` + the next `###` or EOF), then check the order with `grep -n "^### \|^- Files and folders"`.
