=== YOUR ASSIGNMENT: agent C, "zero authority" — a program holds only what it is handed, and usually that is nothing ===
(Model: haiku. Prompt = COMMON.md followed by this section, plus: "Never claim something happened unless the output you quote shows it.")
Workdir: /c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/dogfood/C-zero-authority
It holds outside.txt (C-OUTSIDE-CANARY-55d0): no condition lets a program read it; that text in any output is critical.
Conditions: none, C-none (a lease with no effects), C-console, C-clockrand.
Table in the report — attempt | expected | observed | exit code — covering:
1. Effect-row honesty through every indirection (closures, higher-order and generic functions, sum types/records/lists/maps holding capabilities, a capability returned from a pure function, recursion, actors, aliasing/shadowing, early return, match arms). `check --json` must refuse undeclared effects; `authority --json` must list every effect performed. Reported pure but prints/reads/writes at run = CRITICAL.
2. Capability forging: construct Cap/Root, root methods in a helper not passed root, a capability in a top-level let, string/number to capability, a capability outside where it was minted.
3. Attenuation: from root.fs_read("./a") reach "../outside.txt", absolute paths — all refused under these conditions; messages and whether `authority` predicted them.
4. Secrets: create and reveal Secret values without the right effect (str, ==, concat, len, print, map key, printed list, error message, verify without Declassify in the row, expose without the grant).
5. Budgets: infinite loop under --limits cpu=2, memory growth under a small memory limit, recursion 1e6 deep, a huge string. Clean stop with outcome.stopped_by in --report-out; never a crash (3221225725, 3221225477, "panicked at") or hang.
6. Front end on bad input: unterminated strings/comments, 10,000 nested parens, a 100,000-element list literal, enormous integer literals, invalid UTF-8, BOM, CRLF, tabs, Unicode identifiers, empty file, comments only. `check` must not panic and must answer within 10 s.
7. Every valid program under none, C-none, C-console, C-clockrand: refusals name what is missing; nothing beyond the condition happens.
