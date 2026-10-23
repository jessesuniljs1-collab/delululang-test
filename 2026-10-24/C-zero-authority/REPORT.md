# Agent C (zero authority, Haiku 5.5) — final report, saved by the head chef

The agent could not write this file itself (its Write tool refused a report file); this is its final message, saved
verbatim in substance by the head chef on 2026-10-24 ~01:13 IST. **UNVERIFIED** — every claim is re-run against the
binary before it is used; the head chef's verification is in `_head-chef/VERIFIED.md`. 69 tool calls, ~11 min.
Programs: `progs\`, `fe\`; outputs: `out\`.

## FINDINGS (as the agent reported them)

- **C1 — CRITICAL (claimed) — a function declared pure performs a console write through a generic return.**
  `progs\p3_01_rowbypass_coerce.delulu`: `fn coerce[A, B](x: A) -> B { x }` launders an effectful closure into a pure
  type; `pure_caller` has no row. `dl check` exit 0 (only DL0502 on main); `dl authority --json` effects ["Write"],
  pure_functions includes pure_caller; `dl run C-console … --report-out out\r_console_p3_01b.json` exit 0, prints
  `P3-01-ROWBYPASS-PURE-CALLER` (twice). none / C-none: exit 1, DL0703 at `pure_caller(root.console())`.
  Clock variant `progs\p3_02_rowbypass_clock.delulu`: check 0; run C-clockrand exit 0 prints `P3-02-CLOCK-READ true`;
  `pure_reader` listed pure.
- **C2 — CRITICAL (claimed) — a row variable erased through a generic return.** `progs\p3_03_rowvar_hide.delulu`:
  `fn hide[e](f: fn() -> Unit ! e) -> fn() -> Unit { f }`. check 0; authority pure_functions ["hide","pure_caller",
  "writer"], effects ["Write"]. run C-console exit 0 prints `P3-03-ROWVAR-HIDE-LEAK` (twice). The non-generic twin
  `progs\p3_04_nongeneric_hide.delulu` is refused DL0401.
- **C3 — HIGH — the checker accepts generic bodies that contradict their declared types; the runtime catches them as
  "checker bug" (DL0907).** `progs\p2_09_generic_forge.delulu` (`fn forge[T](x: Int) -> T { x }`),
  `p2_09e_generic_variants.delulu` (`fn g2[T](x: T) -> Int { x }`, g3, g5, g6), `p2_10`…`p2_13`. All check exit 0.
  Runs: `type has no method println on 1`; `Cap[Console]` coerced to `Cap[FsRead]`/`Cap[FsWrite]`/`Root` gives
  `unknown capability method read_text/write_text/fs_read (checker bug)`; Root→Cap[FsRead] `unknown Root method
  read_text (checker bug)`. Root of C1 and C2.
- **C4 — HIGH — top-level `let` initializers are neither type-checked nor name-resolved.** `p5_06_toplevel_typeerr`:
  `let Y: Int = "not an int"` check 0, run prints. `p5_01`: `let X = nosuch_name` check 0, run DL0907 unbound name.
  `p5_02`: `let T = root.clock().now_ms()` check 0, run `unbound name root`.
- **C5 — HIGH — authority omits the Async of a module-level `spawn`.** `p5_05_toplevel_spawn`: `let W = spawn Worker()`.
- **C6 — MEDIUM — an annotated `let` makes a list or map read-only (DL1604); an unannotated `let` is writable; docs say
  a let-bound list is `ref`.** `p6_annot_var_list`, `p6_unann_let`, `s13_map_value_str`; GETTING_STARTED §3, §10.
- **C7 — MEDIUM — an arity error on `load` is reported as forging a capability** (DL0601 beside DL0403). `p5_03`.
- **C8 — MEDIUM — calling a function-typed record field: cascade DL0405 "has no method"** after DL0401. `p1_17`.
- **C9 — LOW — `module t.actor`: generic DL0209 at the module line.**
- **C10 — LOW — record field `tag` refused as reserved for a future stage (DL0106).** `p1_05`.
- **C11 — LOW — invalid UTF-8 source is a usage error (exit 2), not a diagnostic (exit 1).** `fe\f06`.
- **C12 — LOW — Unicode identifiers: DL0101 "unexpected character", not "identifiers are ASCII".** `fe\f10`.
- **C13 — LOW — 200 nested `if`: cascade DL0408 at the same position as DL0210/DL0213.** `fe\f21`.
- **C14 — LOW — repair label `remove_effect_from_row (safe) [requires human decision]` contradicts itself.**
- **C15 — LOW — docs say minting is pure, but a lease run refuses `root.fs_read("./a")` at the mint call.** `p4_01`.
- **C16 — LOW — top-level `const N: Int = …` refused DL0208 though grammar.md lists `const_decl`.** `p1b_05`.

## ATTEMPT TABLES (summary of the agent's tables)
- Part 1, effect-row honesty: ~35 indirections (closures, HOFs, generics, records/sums/lists/maps of caps, returned
  caps, recursion, actors incl. `new` and helpers, `spawn`, aliasing, early return, match arms, List.map/fold, while,
  Read/Clock/Rand, row variables, Secret.map, `?`): all refused DL0501/DL0401 except the generic cases C1–C3.
- Part 2, forging: `Cap[...]()`, `Root()`, root in a helper, top-level const/let of caps, Str/Int→Cap, records,
  uninitialised var, sum-based: all refused (DL0301/DL0601/DL0208/DL0201/DL0501) except generic coercion (C3, caught at
  run time as DL0907, no effect).
- Part 3, attenuation (no fs grant in any condition): `..`, absolute, `C:/`, narrow("../"), `..` + list_dir, write
  escape, `./a/../../`, case and 8.3, NUL byte — all refused DL0703 at the mint. Canary never in any output.
- Part 4, secrets: str (DL0604), == / assert_eq / contains (DL0605), concat / println / literal / map-to-Str (DL0602),
  len (DL0405), map key (DL0401+DL1604), str of containers (DL0604), verify without Declassify (DL0501), expose
  (DL0703 at mint), Err(Secret) (DL0401) — all refused.
- Part 5, budgets (C-console): cpu=2 stopped at 2.0156 s; mem=20 MB list stopped 20.2 MiB; string doubling mem=50 MB
  stopped at 115.4 MiB observed (2.4x overshoot, one sample, documented); wall=2 stopped; map 29.4 MiB; int overflow
  DL0901; list push 33.9 MiB; recursion 1e6 → DL0905 at the 10,000-frame bound; bad --limits refused exit 2.
- Part 6, front end: all under 0.5 s, no panic: unterminated string/comment, 10k parens (DL0210), 100k-element list
  (ok, 458 ms), 10,000-digit int (DL0104), invalid UTF-8 (C11), BOM/CRLF/tabs ok, Unicode idents (C12), empty / comments
  only (DL0204), 200 nested braces, 1 MB string (279 ms), NUL in string (accepted), U+202E (DL0107), 100k-char ident,
  unclosed brace, bad escapes (DL0103), module line only, 200 nested if (C13).
- Part 7, 12 valid programs x 4 conditions (48 runs): every refusal named what was missing; nothing ran beyond its
  condition except C1/C2, which stayed inside the condition's console/clock grants while `authority` called them pure.

## ACCESS REQUESTS
1. `fs.read` on a directory holding a file (use-time `../outside.txt`; whether C1/C2 reach a real file read).
2. `secret:API@127.0.0.1` + `declassify:API` (expose/verify).
3. `net=127.0.0.1:GET` (get_with_secret audiences, redirects).
