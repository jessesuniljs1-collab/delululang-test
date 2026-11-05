---
name: measure-the-stage-before-removing-it
description: "AI-first pipeline audit 2026-11-05 — parsing was 17% of the front end and DIR 28× larger than text; the real costs were Cranelift per run and libpython per command; and the machine-facing input (DIR) had never been attacked"
metadata:
  node_type: memory
  type: feedback
  originSessionId: session_01Deh6W9bJdfxAeMARtXU9R9
  modified: 2026-11-05T07:30:00.000Z
---

"Skip the parser for AI authors" sounded like a saving. A scratch harness linking the crates by path timed each
stage. Lexing and parsing are 17% of the front end at 16,002 lines. DIR, the checked program as data, is 28× larger
than the text and slower to verify than the whole text front end.

The real costs were elsewhere:
- Cranelift compiles the module on every WASM run, from 1.6 ms to 652 ms; a precompiled module loads in 35 µs to
  5.7 ms;
- loading `libpython` costs 1.1–2.0 ms on every command (RW 3.26).

**Why:** an architecture recommendation built on a guess about where time goes picks the wrong thing to remove.

**How to apply:**
- Time stages separately before recommending a removal. The harness is in
  `docs/review/DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md` Appendix A.2.
- Attack every other way a program can arrive as hard as text. Two review passes never attacked DIR, and a sub-agent
  then found three defects (RW 4.93, 4.94, 6.67): colliding node ids, a lexer bypass, and `why` reading DIR without
  verifying it.
- See `HANDOFF.md` §11.2.
