=== YOUR ASSIGNMENT: agent A, "the builder" — real software with full access inside your own directory ===
(Model: sonnet. Prompt = COMMON.md followed by this section.)
Workdir: /c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/dogfood/A-builder
Your conditions: A-full (Read, Write, Net, Clock, Rand, Async; files read+write anywhere under your workdir; HTTPS to 127.0.0.1 with every method; the console), A-console (the console only), none (no authority at all).

Build real programs — the kind a developer would actually write — and use the whole toolchain on each:
1. textstats: read every .txt file under ./data (at least 6: plain ASCII, Unicode with non-Latin scripts and emoji, an empty file, CRLF line endings, one very long line, one 2 MB file), report per-file and total line/word/char counts and the top-10 words, write ./out/report.txt. Check every number against an independent Python computation.
2. A library package made with `$DL new <name> --lib` — e.g. exact rational arithmetic or big integers — with pure tests run by `$DL test`, and a bin package that depends on it, run as a multi-package program. Also `$DL build <pkg>`, `$DL lock <pkg>`, `$DL authority <pkg> --json`. (Effectful tests need a [test-authority] ceiling, which is access the head chef gives: keep tests pure; record any you could not write.)
3. An expression interpreter: tokenizer, recursive-descent parser and evaluator using sum types, Result and recursion; at least 25 expressions including malformed input and division by zero; compare every answer with Python's.
4. Concurrency with actors (examples\guide\06_actors.delulu): a pipeline such as parallel prime counting or a map-reduce word count; check the answer, determinism under --seed, and repeated runs.
5. Data structures and algorithms: merge sort and quicksort on 10k–100k integers, a BST or persistent map, string algorithms; time them — over 10 s on modest input is a finding.
6. A Monte Carlo estimator using Clock and Rand: reproducibility with --seed N and --clock fixed:MS, and without them.
7. A small HTTP client (GET and POST with headers and a body) against a Python TCP listener on 127.0.0.1 that logs what it receives (TLS ClientHello bytes are expected: HTTPS only, then a NetErr — record what the program and the listener each see).
For EVERY program: `check --json` clean; `fmt` idempotent and meaning-preserving; `authority --json` agrees with what the program does; interpreter vs `--engine wasm` stdout byte for byte; `build --target wasm -o x.dwx` then `run A-full x.dwx`; `--trace-effects`; `--sandbox` for file-touching programs. Run each also under A-console and none: refusals clear, nothing beyond the condition happens.
Findings that matter most: wrong answers, crashes or panics, hangs, interpreter/WASM disagreement, a WASM build that silently falls back or refuses what the interpreter runs, misleading diagnostics, docs describing what the binary does not do.
