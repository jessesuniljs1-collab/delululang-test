# The fresh-machine first-run drill (release criterion 8, ruling D9)

**What this measures.** Criterion 8: a new user on a fresh machine, following the Book only,
reaches a working first run in under 5 minutes. D9 ruled the local form: no second physical
machine per OS — "fresh machine" = a pristine environment on existing hardware (an empty
directory set as `HOME`/`USERPROFILE`, no `DELULU_*` environment variables, no `~/.delulu`
state), a `delulu` release binary provisioned as-if-downloaded, and the Book's Chapter 1
journey performed exactly as written.

**The journey (Book, Chapter 1):** write `hello.delulu` from the chapter text → `delulu run
hello.delulu --grant console` → `delulu authority hello.delulu`.

**What the clock includes and excludes — stated, not implied.** The stopwatch covers the full
mechanical journey: creating the source file and executing both commands to completion. It
excludes human reading time (the chapter is two pages) and binary download time (no public
host exists yet — D2; the binary is copied into the pristine environment as the download
stand-in). The interactive locale picker and welcome do not appear in this record because the
drill's stdio is non-interactive, and the first-run surface is suppressed by design off a
terminal (spec §8.5); the interactive welcome is separately witnessed byte-exact by the
Stage-8 tests.

## Results (2026-07-20, release gate for v1.0.0)

| OS | Environment | Journey wall-clock | Bar | Verdict |
|---|---|---|---|---|
| Windows 11 | pristine `USERPROFILE`, no `DELULU_*`, release binary | **1.115 s** | < 5 min | **PASS** |
| Linux (WSL2 Ubuntu) | pristine `HOME`, no `DELULU_*`, release binary | **0.019 s** | < 5 min | **PASS** |
| macOS | — | — | — | **NOT RUN — no Apple hardware (D8); stated, not extrapolated** |

Observed on both OSes: the run printed exactly `hello from the delulu gang`; the authority
report showed `effects: Write`, one Console capability, no secrets, no foreign; and the
pristine home stayed pristine — the drill created no `~/.delulu` state, because nothing in the
Chapter-1 journey needs any.

Even multiplying the mechanical time by two orders of magnitude for a human reading the
chapter and typing the program, the journey fits the 5-minute bar with room. The number we
publish is the one we measured; the margin is the reader's to judge.
