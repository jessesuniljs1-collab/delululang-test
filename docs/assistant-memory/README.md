# The assistant's memory, snapshotted for cloud sessions

**What this folder is.** Claude Code keeps per-project notes — "auto memory" — on the machine it runs
on. For this project that is the owner's laptop, at
`C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\`: an index (`MEMORY.md`) and one topic file
per fact, written by the head chef across every campaign since July 2026. Auto memory is
**machine-local**: a Claude Code cloud session does not have it. From 2026-09-28 to 2026-10-16 the work
runs in cloud sessions (`HANDOFF.md` §0), so on the owner's instruction the whole directory is copied
here — **a snapshot taken 2026-09-28**, 39 topic files and the index; one added since, on the laptop
that evening: `openshell-study-2026-09-28.md` (the study of NVIDIA OpenShell).

**What was changed on the way in — only this.** The repository is public, and a word the owner banned
(`HANDOFF.md` §1) appeared in nine of the files. Every occurrence, in every casing, reads
`[the banned word]` here, and the file named after the rule is `no-banned-word-mentions.md`. Nothing else
was edited: the files are the head chef's working notes, in its own shorthand, dated as they were written.
The copy is made by a script that re-scans the result and refuses to finish if the word survives.

## How a cloud session uses it

1. Read `MEMORY.md` — the index, one line per topic, newest and most important first. On the laptop it
   is loaded into every session automatically; here, read it yourself at the start.
2. Open a topic file when its line bears on the task (`[[name]]` links name other topic files here).
3. **Where a file here and `HANDOFF.md` or `docs/DELULULANG_V2/` disagree, those documents win.** A
   snapshot ages: several files describe states that later phases changed (for example, older lines
   calling the microVM "a probe that always fails" — PS-C built it on 2026-09-27).
4. When you learn something durable — an owner instruction, a trap, a finding that must never be
   re-softened — **update or add a file here in the same format** (frontmatter `name`, `description`,
   `metadata.type`; one fact per file; a one-line pointer in `MEMORY.md`), put the same fact in
   `HANDOFF.md` §11, and list the file in `docs/CLOUD_SYNC_LOG.md`.

## Carrying it back to the laptop (after 2026-10-16)

`docs/CLOUD_SYNC_LOG.md` has the whole sync procedure. For this folder:
`git diff --name-status <baseline>..HEAD -- docs/assistant-memory/` lists what the cloud changed; carry
each change into the laptop's memory directory **by hand**. Do not copy these files over the originals
wholesale — the originals are unsanitized, and the banned word belongs in the laptop's copy of the rule
that bans it, not here.
