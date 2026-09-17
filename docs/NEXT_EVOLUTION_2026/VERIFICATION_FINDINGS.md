# Verification findings — the real binary, driven by hand, 2026-09-17

**Binary:** `target/release/delulu.exe` built from `52eecbe` (clean tree) at 11:49 on 2026-09-17,
`cargo build --release`, exit 0. `delulu --version` → `delulu 1.0.0`. Windows 11, native.
**Method:** fourteen `.delulu` programs written for this pass (kept in the session scratchpad, and
the load-bearing ones reproduced verbatim below), every command run with its own exit code captured
(never a pipeline's), the language server driven by protocol with a 40-line Python client, and the
daemon broker exercised in an isolated `DELULU_STATE_DIR`. Nothing below is inferred from a document.

Findings carry ids **NE-nn** (Next Evolution). Each names the claim that was tested, what the binary
did, the document that says otherwise (if any), and the phase of `IMPLEMENTATION_ROADMAP.md` that
owns it. §2 records what **worked**, because a plan built only on defects would misdescribe the
project.

---

## 1. Defects and gaps found

### NE-01 — A program cannot load a plugin at run time; the docs present it as the flagship demo
- **Tested:** a host program (below) that loads the shipped `shout.dpx` with an empty grant.
- **Observed:** `delulu check host.delulu` → clean; `delulu authority host.delulu` → reports
  `plugins: - shout [verified] grant.effects: [(none)] loaded at host.delulu:6`;
  `delulu run host.delulu --grant console` →
  `error[DL0703]: plugin hosting is not available in the Stage-1 runtime`, exit 1. No `--grant`
  spelling enables it (`plugin`, `plugin=…`, `plugin.load`, `load`, `plugins` are all "unknown grant").
- **Source:** `crates/delulu-runtime/src/prim.rs:366` —
  `"plugin_host" => Err(Fault::at("DL0703", "plugin hosting is not available in the Stage-1 runtime", span))`.
- **What the documents say:** `docs/design/STAGE6_SPECIFICATION.md` status log (line ~529) records
  it honestly: *"the in-language load surface (`root.plugin_host()` → an executable `load`) remains a
  runtime stub in v0.6 … the interpreter mechanics are witnessed by the 6e.5 library tests"*. But
  the Book Ch. 10 ("Here is the demo that sells the language. A running program loads a plugin…"),
  `STAGE6_PLUGINS_GUIDE.md`, `examples/plugin_shout/README.md` ("When a host loads this plugin…"),
  `README.md` and `HANDOFF.md` present loading as built, and **`docs/REMAINING_WORK.md` — the
  single gap list — does not list it.** No shipped `.delulu` file anywhere calls `plugin_host()`
  (`grep` over `examples/`, `tests/`, `docs/book/samples/`: zero hits); the conformance witness for
  the primitive is a prim-table unit test (`every_listed_primitive_resolves`).
- **Also:** the fault is reported under `DL0703` ("root slice not granted"), which tells an agent to
  look for a grant that does not exist.
- **Severity:** high for the AI-first goal (the runtime-extension story is the multi-agent story).
  **Phase:** P2 (build it) and P1 (record it in `REMAINING_WORK.md` immediately).

```delulu
module host

fn shout_via_plugin(root: Root, text: Str) -> Result[Str, PluginErr] ! {Load, Read} {
    let h = root.plugin_host()
    let p: Plugin[Verified] = load(h, "shout.dpx", Grant { effects: [] })?
    let f: fn(Str) -> Str ! {} = p.get("shout")?
    Ok(f(text))
}

fn main(root: Root) ! {Load, Read, Write} {
    let out = root.console()
    match shout_via_plugin(root, "hello from the host") {
        Ok(s) => out.println(s)
        Err(_) => out.println("plugin load failed")
    }
}
```

### NE-02 — A `val` list literal of records/sums is refused with no repair and a mismatched explanation
- **Tested:** `let shapes: val List[Shape] = [Dot(Point { x: 1, y: -2 }), …]` (the shape
  `GETTING_STARTED.md` §3 teaches for `List[Int]`).
- **Observed:** `error[DL1603]: cannot store a fresh value whose contents forbid the lift where
  `val` is required (binding)`; `check --json` → `repairs: []`; `delulu explain DL1603` talks about
  "a `val` closure capturing a `ref`" — not this case. Dropping the annotation (`let shapes = …`)
  checks and runs (`sizes = 10`).
- **Assessment:** the rule may well be correct (a record literal is `ref`; a `val` container
  needs `val` contents), but a diagnostic with no repair, a message about "the lift", and an
  explanation for a different case is exactly the wall an agent cannot climb. Pony's answer is
  better diagnostics plus `recover`; the former needs no language change.
- **Phase:** P1 (message, explanation, and a `safe` repair: drop the `val` annotation or annotate
  the intent). A `recover`-style construct would be an RFC and is not proposed.

### NE-03 — A shipped guide example takes a path shape that silently fails
- **Tested:** under `root.fs_read("./config")`, `fs.read_text("app.txt")` vs
  `fs.read_text("./config/app.txt")`, and the same for `fs_write`/`append_text`, with
  `--trace-effects`.
- **Observed:** paths are **relative to the capability's root**: `app.txt` read the file;
  `./config/app.txt` failed (`B failed`); `tool.log` wrote; `./out/tool.log` failed. The trace shows
  the operation attempted with the literal path and no reason for the refusal.
- **What the documents say:** `examples/guide/05_capabilities.delulu` (the file
  `GETTING_STARTED.md` §6 is built from) does `read_config(reader, "./config/app.txt")` with
  `reader = root.fs_read("./config")` — it always takes the `Err` branch and prints "no config".
  The CI gate for guide examples checks that they *run without a compiler bug*, not that they read
  anything. `examples/demo.delulu` uses the correct `"app.txt"`. Two shipped examples disagree and
  nothing states the rule.
- **Phase:** P1 (fix the example; state the rule in `explain E-DL0703`/`for-agents.md`; make the
  guide gate assert the read succeeds; consider an `IoErr` detail that says "outside the
  capability's scope `./config`" rather than a bare `Err`).

### NE-04 — One deep-nesting defect produces 145 diagnostics
- **Tested:** a 200-deep parenthesised expression (`deep.delulu`).
- **Observed:** `check --json` → **73 × DL0210** ("expression nests deeper than 128 levels", one per
  level past the limit) **plus 72 × DL0404** ("value of type `'t0` … `'t70` is not callable") from
  the parser's recovery junk being type-checked. 145 errors for one cause. The machine channel is
  uncapped by contract, so an agent receives all of them.
- **Phase:** P1 (emit DL0210 once per overflow; do not type-check the recovered placeholder as a
  call). Language-visible? No — the accepted language is unchanged; only the diagnostic count for
  already-refused programs moves. The core-invariance snapshot must be regenerated deliberately for
  that case and the diff shown to the owner (core-regression rule).

### NE-05 — The `--json` envelope is not uniform across commands
- **Tested:** `why --json`, `add --path --json`, `plugin inspect --json`, `test --json`,
  `atlas --format json`, `--version --json`.
- **Observed:** `why --json` prints a bare `{"effect","path","performs"}` (source: `cli.rs`
  lines 4263/4311/4371/4423/4437, five emitters) — no `command`, `schema`, `delulu_version`,
  `diagnostics`, `summary`. `add --path --json` → `{"authority","command","name","path","written"}`
  (no `schema`/`delulu_version`/`summary`). `plugin inspect --json` → `command: "plugin"` with no
  `schema`/`delulu_version`/`summary`. `test --json` → has `command`/`schema`/`delulu_version` but
  **no `diagnostics` array**. `atlas --format json` is its own `atlas/1` schema (defensible, but not
  the envelope). `--version --json` prints text.
- **What the documents say:** `docs/for-agents.md` [agents.json-envelope]: *"Every `--json` command
  emits one object: `{command, schema, delulu_version, diagnostics, summary}`"*.
  `crates/delulu/tests/json_contract.rs` gates only the **failure** envelope
  (`a_failure_envelope_carries_the_fields_a_caller_keys_on`) and "exactly one JSON value".
- **Phase:** P1 (a success-envelope sweep test first, then wrap the emitters; additive, no field
  changes meaning).

### NE-06 — `explain` accepts `--json` and ignores it
- **Observed:** `delulu explain DL0501 --json` prints prose, exit 0. Source `cli.rs:7809`:
  `rest.iter().find(|a| a.starts_with('-') && a.as_str() != "--json")` — the flag is exempted from
  the refusal and then unused. There is no machine channel for explanations at all.
- **What the documents say:** "an option nobody understood is refused, never ignored" (`run`
  honours it: `--bogus` → exit 2). `explain` is in `json_contract.rs`'s `SUBCOMMANDS`, which
  therefore passes on a command that never emits JSON on success.
- **Phase:** P1 (`explain --json` → `{code, title, body, disposition}` in the envelope).

### NE-07 — A repair with zero edits is reported as machine-applicable, then refused as human-only
- **Observed:** DL0502 (`main` declares `Write` it never performs) carries repair
  `remove_effect_from_row` with `confidence: "safe"`, `authority_widening: false`,
  `requires_human: false`, **`edits: []`**. `delulu fix --dry-run --json` then reports its
  `verdict: "requires-human"`. The LSP code action for it has `isPreferred: true` and **no `edit`**.
  Two channels disagree about the same repair, and the diagnostic's own flags say "apply me".
- **Phase:** P1 (a repair with no edits must carry `requires_human: true` or carry edits; a test
  that `check --json` flags and `fix` verdicts agree).

### NE-08 — `test --json` hides the diagnostics that failed the run
- **Observed:** a file failing its check under `delulu test --json` prints the human DL render on
  **stderr** and the envelope says only `{"file": …, "status": "check-failed"}`; a test exceeding
  its ceiling gets `failure.message: "DL1703: …"` with no span, code field or repair.
- **Phase:** P1 (carry `diagnostics` in the `test` envelope; codes as fields).

### NE-09 — `delulu test` in a freshly scaffolded package exits 2
- **Observed:** `delulu new hello && cd hello && delulu test` → `error: `delulu test` needs test
  files/directories (no ./tests directory here)`, exit 2, although the scaffold's test lives in
  `src/main.delulu` and `delulu test .` passes. `crates/delulu/tests/new_cli.rs` documents this
  trap and fixed the *printed instructions* rather than the default.
- **What the documents say:** README: "a package that already checks, tests and runs".
- **Phase:** P1 (default target inside a package = the package).

### NE-10 — The authority report never says which `--grant` flags a program needs
- **Observed:** `delulu authority agent_tool.delulu` prints `FsRead (scope granted at runtime)`,
  `Http (scope granted at runtime)`; the JSON has `"scopes": []` for both although the source
  literally requests `root.fs_read("./config")` and `root.http(["example.com"])`. The grant
  grammar (`fs.read=PATH`, `net=HOST`, `secret:NAME=VALUE`, bare `declassify`, `clock`, `rand`,
  `console`, `actuator=…`, `sensor=…`, `compute=…`, `foreign.c=…`, `foreign.python=…`,
  `exec.native`) is discoverable only by reading `crates/delulu-runtime/src/broker.rs` or by
  provoking DL0703 one flag at a time. `--grant declassify:API_KEY` (the natural guess from
  `grants delegate --declassify N`) is "unknown grant".
- **What the documents say:** the archive's `INSTALL.txt`: *"Run `delulu authority <file>` first and
  the required grants are the list it prints."* It is not.
- **Phase:** P1 (`delulu authority --grants` / a `required_grants` array and a `requested_scopes`
  field, additive; the honesty split — kind is static, scope is runtime — is preserved by naming
  them *requested*, never *granted*).

### NE-11 — The shipped morphs do not ship
- **Observed:** from any directory but the repository root, `delulu morph render x.delulu --to
  compact-ai` → `DL1714: morph `compact-ai` is not installed — looked in morphs,
  C:\Users\…\.delulu\morphs`; `morph list` → "no morphs installed". `scripts/package-toolchain.sh`
  copies licences and examples and **no `morphs/`**.
- **What the documents say:** `for-agents.md` [agents.morphs]: "`morphs/compact-ai.toml` ships as a
  short-alias profile".
- **Phase:** P5 (ship `morphs/` in the archive and look beside the binary).

### NE-12 — `REPOSITORY_STRUCTURE.md` claims a mechanical check that does not exist
- **Observed:** lines 26–27 and 459: *"Both directions are now checked mechanically … every
  markdown file is accounted for by name or by its group."* `grep -rn REPOSITORY_STRUCTURE crates/
  --include=*.rs` → **no hits**. No test, no Survey rule reads the document.
- **Phase:** P1 (write the gate — the Survey already knows every markdown file — or reword).

### NE-13 — A single-file effectful test cannot be run at all
- **Observed:** `test "…" ! {Write} { let out = test_root.console() … }` in a standalone file →
  `DL1703: … the package [test-authority] ceiling allows only []`. `delulu test` has no flag to
  grant test authority; the ceiling can only come from a package manifest.
- **What the documents say:** `GETTING_STARTED.md` §9 shows the effectful test form without saying
  it needs a package.
- **Phase:** P1 (document the rule; consider `delulu test --test-authority Write` as an explicit,
  reviewable, additive flag — a ruling is needed because it is a new authority source).

### NE-14 — Requested scopes are dropped from the Atlas
- **Observed:** `delulu atlas demo.delulu --format json` → `"kind": "FsRead", "scopes": []`
  although the scope literal is in the source. Same root as NE-10.
- **Phase:** P1 (with NE-10).

### NE-15 — Diagnostics from the language server are correct; hover reports the declared row only
- **Observed (by protocol):** `initialize` answers the full capability set; `didOpen` publishes
  DL0501 (severity 1) and DL0502 (severity 2) with correct ranges; `codeAction` returns the typed
  repairs with `data.authority_widening` / `requires_human` and the widening one `isPreferred:
  false` and ⚠-titled; `workspace/executeCommand delulu.authority` refuses on a file with errors
  ("fix them first"); hover on the erroring `shout` says `authority: pure` (its *declared* row)
  while the diagnostic says it performs `Write`.
- **Assessment:** correct by design (the row is the declaration); an agent reading hover alone is
  misled. Low priority; a "declared vs performed" hover line is a P4 nicety.

### NE-16 — The daemon broker round trip works, and the audit chain caught the harness's own mistake
- **Observed (isolated state dir):** `broker start` (prints the owner code once), `grants delegate
  --effects Write --ttl 5m` (token), `run hello.delulu --lease <token>` → runs under the delegated
  node with the guard banner; a second redemption → `DL1407 single-use token already redeemed`;
  `run clock_rand.delulu --lease <Write-only token>` → `DL0703 clock was not granted`; `grants
  revoke <root>` → "revoked 2 node(s)" with the latency bound stated; `audit verify` → chain
  verified. My first attempt truncated the token at its `.` and the chain recorded two `redeem deny`
  events for it — the audit log did its job on the tester.
- **Phase:** none (positive). Recorded because the multi-agent story is this path.

---

## 2. What worked, exactly as documented

| Claim | How it was tested | Result |
|---|---|---|
| Zero ambient authority; refusal names the flag | `run hello.delulu --json --no-prompt` with no grant | `DL0703 … pass --grant console`, one envelope, exit 1; with the grant: `Hello, Delulu`, exit 0 |
| Many files, one process | `check` on 8 files at once | one process, per-file verdicts, exit 1 on any error; 7 clean, 1 error |
| Typed repairs and the widening rule | `fix --dry-run --json`, `fix --json`, `fix --accept-widening add_effect_to_row` | file untouched without the named acceptance; with it, `! {Write}` inserted at byte 153 exactly as the repair said |
| Authority report and the exposure join | `authority agent_tool.delulu` | `effects: Declassify, Net, Read, Write`; `exposure: API_KEY declassifiable -> the network, files/console`; `pure fns: normalize` |
| `why` | `why Read examples/demo.delulu` | `main (…:25) -> read_config (…:20) — Read` |
| Deterministic replay | `run clock_rand.delulu --seed 7 --clock fixed:1000` twice | byte-identical (`t = 1000`, `r = 52`) |
| WASM fragment, fail-closed | `run --engine wasm` on `fib` vs on a `while` loop | `fib(20) = 6765`; `DL1201: WASM codegen does not support `while`` — no silent fallback |
| `.dwx` artifact | `build --target wasm`, `run x.dwx` | 1006-byte artifact; "authority verified; declared effects: Write"; runs |
| Actors | `run actors.delulu` (spawn, five sends, a reporting behaviour) | `count = 10` |
| Loops, sums, records, `for/break/continue`, `parse_int`, string methods | `run lr2.delulu` | `sizes = 10`, `sum = 6`, `words = 3`, `parsed 42` |
| Tests hold no ambient authority | `test lr2.delulu --json` | two pure tests pass; the effectful one is refused (`DL1703`) — see NE-13 for the ergonomics |
| Nesting bound | 200-deep expression | refused with `DL0210`, exit 1, no crash (see NE-04 for the noise) |
| Unknown option refused | `run … --bogus` | exit 2, "refused, never ignored" |
| Missing input → one failure envelope | `check nope.delulu --json` | exit 2, `error.kind: usage`, one object |
| Plugin build / inspect / verify | `plugin build examples/plugin_shout`, `inspect --json`, `verify` | 1333-byte verified `.dpx`, empty ceiling, byte-stable digest |
| Package scaffold | `new hello`, `check .`, `authority . --json`, `run . --grant console`, `build`, `lock`, `new --lib`, `add --path` | all exit 0; lockfile carries content, authority and api-row hashes and scopes; the dependency line records a computed pin |
| Formatter | `fmt --check`, `fmt --stdin` | deterministic canonical form; `--check` exits 1 on unformatted input |
| Morph (from the repository root) | `morph render examples/hello_wasm.delulu --to compact-ai` | `//! morph: compact-ai` header, keywords `MOD/F/L` |
| Survey | `check`, `findings`, `query`, `impact`, `--json` | map current (1,121 nodes / 10,018 edges); every hop cites `file:line`; `impact` JSON is uncapped |
| Doctor | `doctor`, `doctor --check --json` | 17 checks; regenerated the map when a new markdown file appeared; security posture names LEGACY root issuance and same-user reachability |
| Shell completions | `completions bash` | generated from the command list |

## 3. Numbers, so they are not remembered wrong

| | |
|---|---|
| Programs written for this pass | 14 (plus 2 variants) |
| CLI invocations with a captured exit code | ~90 |
| Defects/gaps recorded | 14 (NE-01…NE-14) + 2 observations (NE-15, NE-16) |
| Of those already named in `REMAINING_WORK.md` | 0 as rows (NE-01 is named only in the Stage 6 spec's status log) |
| Positive claims re-verified | 22 |

None of the counts above is a test count; test counts come from `cargo test`.
