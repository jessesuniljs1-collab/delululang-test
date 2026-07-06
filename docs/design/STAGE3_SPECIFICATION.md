# DeluluLang — Stage 3 Implementation Specification

**Version:** 0.3 ("Containment"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 2 complete (lockfile/authority verification and the trace harness are
prerequisites for backend parity testing).
**Governing documents:** `CONSTITUTION.md` (§5.11, §5.13, §5.14), `SOUNDNESS_AUDIT.md`.

---

## 0. Scope and goal

**Goal:** give DeluluLang its **runtime enforcement floor** — compile packages to WASM and run
them inside an embedded, deny-by-default Wasmtime host, so that layer 2 of the defense-in-depth
stack (Constitution §5.14) exists: even a compiler or type-system bug is *contained*. This is the
committed replacement for the old "transpile to C" stage (Constitution Appendix A #15): one
portable target that is also the sandbox.

**In scope (must ship):**
- Code generation: typed AST → WASM (WASM-GC + externref), full Stage-1/2 language.
- The **`delulu:cap` host interface**: every capability operation crosses the guest/host boundary;
  all grant and scope checks run **host-side**.
- The **`.dwx` artifact**: WASM module + embedded authority manifest + ABI version, one file,
  runs on all OSes.
- `delulu build --target wasm`, `delulu run <file.dwx>`, `--engine interp|wasm`.
- **Differential parity**: the entire conformance suite runs on both engines with identical
  output and identical effect traces.
- Secrets-stay-host-side hardening (§4.4).
- New diagnostics: DL12xx (build/artifact/engine).

**Non-goals (later stages):** JIT tiering and `@aot`/`@interpret` hints under policy (Stage 10 —
until then the two modes are "interpreter" and "AOT-compiled WASM", both sandbox-equivalent);
microVM nesting and the process-separated broker (Stage 5); plugin loading (Stage 6); threads
(Stage 7); native cross-compilation (post-v1.0, Constitution §5.13).

---

## 1. Invariants (carried + new)

All Stage-1/2 invariants hold. New:

14. **Sandbox parity.** Execution mode never changes what is permitted (Constitution §5.11 rule 1).
    The guest holds **no OS handles**; every effectful operation is a host call that re-validates
    grant id + scope, identically to the interpreter's checks — same code path in
    `delulu-runtime`, linked by both engines, tested by identical traces.
15. **Observable equivalence.** For every program in the conformance suite, interpreter and WASM
    engines produce byte-identical stdout/stderr (modulo an engine tag in `--verbose`), identical
    exit codes, and identical effect traces under fixed `--seed`/`--clock`.
16. **Guest opacity of authority.** Capabilities and secrets are `externref` handles into host
    tables. Neither capability material nor secret bytes ever enter guest linear memory or guest
    GC structures, except a secret's bytes after a lawful `expose` (which returns a guest string —
    that is what declassification *means*).
17. **The artifact carries its authority.** A `.dwx` without a valid embedded authority manifest
    does not run (DL1202). Runtime grants are checked against the *embedded* manifest, exactly as
    source runs check `delulu.toml`.
18. **Trust honesty at the artifact boundary.** Running a `.dwx` you did not build from source is
    **Contained-grade trust** (Constitution §6): the embedded manifest is enforced by the host
    boundary at module granularity, but per-function rows inside a foreign artifact are claims,
    not re-verified proofs, until Stage 6's Verified-loading (signed typed IR) lands. `delulu run`
    prints this distinction; `delulu authority <file.dwx>` labels its report `"grade": "contained"`
    unless the artifact was locally built (`"grade": "verified"`, recorded via local build
    receipts §5.3).

---

## 2. Lexical / grammar / AST additions

**None.** Stage 3 adds no surface syntax. The `@aot`/`@interpret` attribute *tokens* remain
reserved (`@` is already a token; attribute grammar is specified in Stage 10 where policy
enforcement can make them honest).

---

## 3. Code generation

### 3.1 Target features and layout

- Target: WASM core + **GC proposal** + **reference types** (externref) + multi-value, executed
  by the pinned Wasmtime version vendored into `delulu`. No WASI imports **at all** except the
  `delulu:cap` world (§4) — stdout itself goes through `Cap[Console]`.
- One package graph → one module. Dependency functions are compiled into the same module with
  `DefId`-mangled names (`d2_mathkit_stats_mean`); no dynamic linking in Stage 3 (that is what
  plugins are for, Stage 6).

### 3.2 Representation map (normative)

| DeluluLang | WASM |
|---|---|
| `Int` / `Float` / `Bool` / `Unit` | `i64` / `f64` / `i32` / elided |
| `Str` | GC `array i8` (UTF-8, immutable) + length; host calls pass (ptr-into-array via `array.copy` to host) — see §4.2 |
| `List[T]`, records, variants | GC `struct`/`array`; variants = struct { tag: i32, payload… } |
| `Option`/`Result` | variants as above |
| closures (`fn` values) | GC struct { funcref (typed), env struct } — rows are compile-time only; **runtime row erasure is sound** because all row checking already happened statically and effects are host-gated regardless (invariant 14 backstops even a checker bug) |
| `Cap[R]`, `Root`, `Plugin[_]` | `externref` into the host **cap table** — unforgeable by construction (guest cannot mint externrefs) |
| `Secret[T]` | `externref` into the host **secret table** (§4.4) |

### 3.3 Lowering rules

- Checked arithmetic lowers to explicit overflow tests trapping to the panic hook (identical
  DL09xx JSON as the interpreter, invariant 15).
- `?` lowers to variant tag test + early return; `match` to tag dispatch (exhaustiveness already
  proven, no default trap path except unreachable).
- Recursion depth limit: enforced by a guest-side frame counter global (Wasmtime stack limits are
  the backstop), so the *diagnostic* is identical across engines.
- Panics: guest calls `delulu:cap/abort(code, span_id)`; host renders the same DL09xx diagnostic
  as the interpreter, using the span table from the artifact (§5.2).

---

## 4. The `delulu:cap` host interface (the floor itself)

### 4.1 Design law

Every entry in the Stage-1 primitive table (§7.3, including the audit's `Secret` rows) is a host
function. The host function — not the guest — performs: grant-id liveness check, scope check,
the actual OS operation, and trace emission. **The guest is untrusted by construction**; a
malicious or miscompiled module can call host functions in any order with any arguments and still
cannot exceed the grant table (this is the containment claim, and it is module-granular — stated
honestly per Constitution §5.3/§6).

### 4.2 Interface (WIT-style, authoritative list)

```wit
package delulu:cap@0.3.0;

interface caps {
  // handles are externrefs managed by the host; u32s below are host-table indices carried
  // inside externrefs — guests never see raw indices.
  root-console:  func() -> result<cap, cap-err>;
  root-fs-read:  func(path: string) -> result<cap, cap-err>;
  root-fs-write: func(path: string) -> result<cap, cap-err>;
  root-http:     func(hosts: list<string>) -> result<cap, cap-err>;
  root-clock:    func() -> result<cap, cap-err>;
  root-rand:     func() -> result<cap, cap-err>;
  root-secret:   func(name: string) -> result<secret, cap-err>;
  root-declassify: func() -> result<cap, cap-err>;

  console-println: func(c: cap, s: string) -> result<_, io-err>;
  console-print:   func(c: cap, s: string) -> result<_, io-err>;
  console-readline: func(c: cap) -> result<string, io-err>;
  fs-read-text:  func(c: cap, path: string) -> result<string, io-err>;
  fs-list-dir:   func(c: cap, path: string) -> result<list<string>, io-err>;
  fs-narrow:     func(c: cap, path: string) -> result<cap, cap-err>;
  fs-write-text: func(c: cap, path: string, body: string) -> result<_, io-err>;
  fs-append-text: func(c: cap, path: string, body: string) -> result<_, io-err>;
  http-get:      func(c: cap, url: string) -> result<string, net-err>;
  clock-now-ms:  func(c: cap) -> s64;
  rand-int:      func(c: cap, lo: s64, hi: s64) -> s64;
  rand-float:    func(c: cap) -> f64;

  secret-map-id:   func(s: secret, op: mapop) -> secret;   // pure ops whitelist, host-side (§4.4)
  secret-verify:   func(a: secret, b: secret) -> bool;      // constant-time, host-side
  secret-expose:   func(s: secret, d: cap) -> result<string, cap-err>;  // the Declassify gate

  abort: func(code: u32, span-id: u32);
}
```

Versioned as `delulu:cap@MAJOR.MINOR`; a `.dwx` importing an unknown version is DL1204.

### 4.3 Scope checking (host-side, normative)

Identical implementation to Stage 1 (`delulu-runtime` is shared): path canonicalization then
prefix check (symlinks resolved host-side **before** the check; escape → DL09xx scope violation),
host allowlist with exact/`*.` patterns, https-only. The grant table maps externref → `CapVal`;
liveness (GrantId) is checked first — Stage 5's revocation and Stage 6's plugin unload reuse this
exact hook.

### 4.4 Secrets stay host-side (hardening beyond Stage 1)

`Secret[T]` values in WASM are externrefs; the bytes live only in the host secret table
(zeroized on drop). Consequences, all normative:
- `Secret.map` in compiled code is restricted to the **host-implemented pure-op whitelist**
  (`trim`, `slice`, `to-lower`, `append-const` — the ops the stdlib actually ships); arbitrary
  guest closures over secret *contents* are a compile error in `--target wasm` (DL1205, message
  explains the host-side model). The interpreter keeps full `map` with pure closures; this is an
  engine capability difference, documented, and the conformance suite partitions accordingly.
  (Assessment: this restriction is temporary until Stage 6's verified-IR execution can run
  verified-pure closures host-side; the alternative — copying secret bytes into guest memory —
  would silently void invariant 16, which is worse than a visible restriction.)
- A memory-scan test (§9 criterion 8) asserts secret bytes never appear in guest linear memory
  or GC heap snapshots.

---

## 5. The `.dwx` artifact

### 5.1 Layout

A `.dwx` is a WASM module with custom sections:

| Section | Content |
|---|---|
| `delulu:abi` | `{ "dwx": 1, "cap_iface": "0.3.0", "compiler": "0.3.x" }` |
| `delulu:authority` | the canonical authority JSON of the root package (Stage-2 §4.1 output), including per-dependency verified authority |
| `delulu:spans` | span table: span-id → (file, line, col) for runtime diagnostics |
| `delulu:lock` | the `delulu.lock` content used at build (provenance for `--diff`) |

> **Implemented (Phase 3g, §8a):** the `delulu:authority` section ships now, carrying
> `{version, code_blake3, authority}` — the ABI `version` and a blake3 hash **binding the manifest
> to the exact code** are folded into this one section rather than a separate `delulu:abi`.
> `read_and_verify` re-hashes the bare module and rejects a mismatch (DL1202); a newer `version` is
> DL1204. The `delulu:spans` and `delulu:lock` sections, and split-out `delulu:abi`, are not yet
> emitted. Honesty: the hash is an integrity binding, **not** a signature (see §5.3).

### 5.2 Runtime grant flow

`delulu run app.dwx --grant …` reads `delulu:authority` and runs the **identical** Stage-1 §7.2
flow (manifest check → grants → prompt → construct host cap tables). No source needed at run time.

### 5.3 Build receipts (local verified-grade)

`delulu build` writes `target/receipts/<content-hash>.json` (artifact hash, lockfile hash, toolchain
version, timestamp). `delulu run`/`authority` mark an artifact `verified` iff a matching local
receipt exists; otherwise `contained` (invariant 18). Receipts are local trust bookkeeping —
**not** signatures; signing/provenance lands in Stage 8/9 governance and this file says so.

---

## 6. CLI additions and contracts

```
delulu build [--target wasm] [-o out.dwx] [--locked] [--json]
delulu run   <file.delulu|file.dwx> [--engine interp|wasm] [grant flags…] [trace flags…]
delulu authority <file.delulu|file.dwx> [--json]      # for .dwx: reads embedded section; reports "grade"
```

- Default engine: `interp` for `.delulu` sources, `wasm` for `.dwx`. `--engine wasm` on a source
  file builds to a temp artifact then runs it (the parity workhorse).
- `delulu run app.dwx` with no local receipt prints one line before the grant prompt:
  `note: artifact not built locally — contained-grade trust (see delulu explain E-TRUST-DWX)`.
  Suppressed only by `--json` (where it appears as a structured `"trust": "contained"` field),
  never by CI mode.

---

## 7. Diagnostics (fresh range DL12xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1201 | construct unsupported by codegen (compiler-bug class — grammar and codegen must stay total together) | none — file a bug |
| DL1202 | artifact missing/invalid `delulu:authority` or `delulu:abi` section | none — `requires_human: true` |
| DL1203 | artifact hash/receipt conflict (receipt exists but hash differs) | none — `requires_human: true` (possible tamper) |
| DL1204 | `delulu:cap` interface version unsupported | rebuild with current toolchain — exact |
| DL1205 | guest closure over secret contents in `--target wasm` (see §4.4) | use host whitelist op — suggestions listed; never `authority_widening` |
| DL1206 | engine parity self-check failure (`--verify-parity` mode) | none — compiler-bug class |

Runtime violations (scope escape, revoked grant, overflow…) keep their Stage-1 **DL09xx** codes on
both engines — same violation, same code, regardless of engine (invariant 15).

---

## 8. Standard library additions

None user-visible. `std` gains no new API; the Secret pure-op whitelist (§4.4) is the host-side
implementation of the *existing* `Secret.map` stdlib ops.

---

## 8a. Implementation status (2026-07-05)

**Phase 3a — the sandbox floor's foundation — is implemented and green** (137 workspace tests).
New crate `crates/delulu-wasm` (deps: `wasm-encoder` 0.221, `wasmtime` 27, default-features off,
`cranelift`+`runtime`). It compiles the **pure-Int/Bool fragment** (arithmetic, comparisons,
`&&`/`||`, unary `-`/`!`, `if`/`else`, `let`, calls, recursion) to core WASM (`codegen.rs`) and
runs it under an embedded, **zero-import (deny-by-default)** Wasmtime host (`host.rs`). The
correctness contract is **two-engine parity**: `compile_and_run_int` vs the interpreter's new
additive `Interp::call_int_fn` must agree — verified on `fib`, `gcd`, polynomials, nested `if`,
booleans, and negation. Constructs outside the fragment are `CompileError` (DL1201-class) and
stay on the interpreter, which remains the reference engine.

**Phase 3b — the `delulu:cap` host interface, first slice — is implemented and green** (139
tests). `codegen.rs` now handles `Str` (string literals live length-prefixed in the module's
linear memory; a `Str` is an i32 pointer), `Cap[Console]` (an i32 handle), and `Unit`, and
compiles `Cap[Console].println(str)` to an imported host function `delulu:cap.console_println`.
`host.rs::run_console_fn` provides that import via a Wasmtime `Linker`: it performs the Write
effect **host-side**, checks the capability handle against a host cap table (an ungranted handle
is refused — the scope check host-side), and reads the string out of the guest's exported memory
(the guest gets no OS handle). The runtime gained a capturable console (`set_capture`/
`take_capture`) and `Interp::call_with`, so the parity test compares the WASM host's captured
output to the interpreter's — **identical output on a Write effect**, not just pure computation.
(Implementation note: a capability refusal is recorded in host state and surfaced after the call
rather than returned from inside the wasm-invoked callback, which aborts on Windows.)

**Phase 3c — a generative differential parity gate — is implemented and green** (141 tests).
`gen.rs` generates random *pure* programs (terminating: `f` may call `g`, `g` calls nothing;
overflow-free: only `+`/`-` over small bounded operands; trap-free: no division), and the
`wasm_matches_interpreter_on_random_pure_programs` test compiles + runs 800 of them (>500 valid)
on BOTH engines and asserts identical `Ok(i64)` results. Zero divergence — the correctness bar for
a compiler backend, checked continuously in CI.

**Divergence CLOSED in Phase 3d (§3.3):** the WASM backend now has **identical fault semantics** to
the interpreter on arithmetic. Codegen emits synthetic checked-arithmetic helper functions
(`__ovf_add`/`__ovf_sub`/`__ovf_mul` trap on signed overflow; `__chk_rem` traps on `b==0` and
`INT_MIN%-1`) and routes `+`/`-`/`*`/`%` through them; `/` uses `i64.div_s`, which already traps on
div-by-zero and `INT_MIN/-1` exactly like `checked_div`. The Phase-3d parity harness generates
programs that DO overflow and DO divide by zero, and asserts the two engines agree on both the
value (both `Ok` and equal) and the fault (both `Err`) — verified over **5,000 programs, zero
divergences, with the fault path proven exercised** (`both_faulted > 50`). The WASM trap and the
interpreter fault (DL0901/DL0902) are both *errors*; the harness treats both-error as consistent.

**Phase 3e — `Root`/`main` capability threading through WASM — is implemented and green** (143
tests). Codegen handles the `Root` type (i32 handle) and `root.console()` (a host import
`delulu:cap.root_console` that mints a Console handle host-side iff the grant allows).
`host.rs::run_main_console` runs a real `fn main(root: Root)` under Wasmtime: root at handle 0, the
`delulu:cap` import world (`root_console` + `console_println`, both checked host-side, neither
trapping from inside the callback), returning the captured output. The milestone test runs a
`main` that does `root.console()` then `out.println(...)` on WASM and asserts the output equals the
interpreter's — a full program with capability threading, byte-identical across engines; an
ungranted console is refused host-side.

**Phase 3f — `delulu run --engine wasm` reachable from the terminal — is implemented and green**
(147 tests). The CLI (`crates/delulu/src/cli.rs`) gained an `--engine wasm` flag on `run`: after
the checker passes and grants are resolved, it compiles the module via
`delulu_wasm::compile_module` and runs `main` under `run_main_console(wasm, grants.console)`,
printing the guest's captured output to stdout — byte-identical to the interpreter path. The
DL12xx codes are registered (`crates/delulu-diag/src/codes.rs`): a construct the backend can't
compile is reported as **DL1201** (with the repair "omit `--engine wasm` to run it on the
interpreter") rather than silently misrun; a **denied root slice** (`root.console()` with no
grant) surfaces as **DL0703**, and a use-site capability-scope failure as **DL0904**. The host
now preserves the *first* refusal as the root cause (a later use-site error can't clobber a prior
`root_console` denial). New example `examples/hello_wasm.delulu` and a new CLI integration test
file `crates/delulu/tests/wasm_cli.rs` (4 tests) assert: the wasm engine prints the expected line,
its output equals the interpreter's, an ungranted console is refused, and an unsupported program
(`demo.delulu`) yields DL1201.

**Phase 3g — the `.dwx` authority-carrying artifact — is implemented and green** (156 tests). New
module `crates/delulu-wasm/src/artifact.rs` defines the `.dwx` format: a WebAssembly module with a
`delulu:authority` custom section carrying `{version, code_blake3, authority}`, where `authority`
is the same JSON `delulu authority` reports and `code_blake3` is a blake3 hash of the bare module —
**binding the manifest to the exact code it describes**. `embed_authority` appends the section;
`read_and_verify` re-derives the bare module (all sections except `delulu:authority`, original
order), re-hashes it, and rejects a mismatch. The CLI wires this in: `delulu build <file.delulu>
--target wasm [-o out.dwx]` checks the program, compiles `main`, embeds the authority, and writes
the artifact; `delulu run <file>.dwx` re-verifies before running and announces the declared effects.
Faults map to the registered codes: a missing/corrupt/tampered section or non-artifact bytes are
**DL1202**; a manifest from a newer toolchain is **DL1204**. Honesty (Constitution): the hash binds
the authority claim to *these* bytes — corruption and naive code/authority swaps are caught — but it
is **not** a cryptographic signature and does not prove *who* built the artifact; publisher signing
is a later stage. Coverage: 5 artifact unit tests (round-trip, still-runs-under-Wasmtime, tamper
via section-splice onto different code, missing section, non-wasm) + 4 CLI integration tests in
`wasm_cli.rs` (build→run matches the interpreter, ungranted console refused, a byte-flipped
artifact is DL1202, a non-artifact `.dwx` is DL1202).

**Phase 3h — the hostile-guest test (§9.4, first slice) — is implemented and green** (161 tests).
New integration file `crates/delulu-wasm/tests/hostile_guest.rs` hand-crafts adversarial WASM the
compiler never produced (via `wasm-encoder`) and proves the deny-by-default host holds against a
guest that ignores the rules: (a) a **forged capability handle** (calling `console_println` with
handle 999, absent from the cap table) is refused host-side as **DL0904** — the Write never
happens; (b) an **out-of-bounds string pointer** (valid handle, `ptr` past the 64 KiB memory) is
bounds-checked host-side and refused as **DL0903**, not read; (c) a **hugely negative pointer**
(`ptr = -1`) is refused cleanly rather than overflowing `usize` and aborting the process; (d) a
guest that **imports a capability the host does not provide** (`delulu:cap.fs_open`) cannot even
instantiate (deny-by-default is a strict whitelist). A positive control (a well-formed hand-crafted
guest) confirms the same runner *does* perform the effect, so the refusals are real. This slice
hardened the host: `console_println` now treats the wasm pointer as an unsigned offset and uses
checked arithmetic (a `ptr` near `u32::MAX` previously risked a debug-build overflow panic inside
the callback — a process abort on Windows), and out-of-bounds refusals now carry DL0903 (the CLI's
`wasm_fault_code` maps DL0703/DL0903/else-DL0904). Not yet covered by §9.4: filesystem-subtree
escape via `..`/symlink and secret-expose misuse (those `delulu:cap` ops don't exist yet).

**Phase 3i — `Str` concatenation in the WASM backend — is implemented and green** (165 tests).
`Str + Str` now compiles instead of being DL1201: `codegen.rs` emits a synthetic `__concat(a, b)`
helper (a fifth always-present helper, after the four arithmetic ones) that bump-allocates a fresh
`[len:u32-le][bytes]` buffer in guest linear memory and returns its pointer. The bump pointer is a
mutable i32 **global** initialised just past the interned string-literal image; the module reserves
a fixed `HEAP_PAGES` (1 MiB) heap above the literals (no `memory.grow` yet, so a pathological
concatenation loop would trap on the store — an honest error, not a silent wrong answer). The helper
copies both operands with `memory.copy` (bulk memory, on by default in Wasmtime 27). Two-engine
parity is the contract, verified by four new tests: chained literal concat (`"[" + "x" + "]"`),
concat with a **function parameter** (`greet(name) = "hello, " + name`), repeated concat that
proves the allocator advances (each result gets its own buffer), and that `println` of a `Str + Str`
now compiles. End-to-end on the terminal, `greet("world") + "!"` prints `hello, world!` **byte-
identically on `--engine wasm` and the interpreter**. (Observed en route, out of scope here: the
lexer rejects a leading UTF-8 BOM with DL0101 — a small future robustness fix, not a concat issue.)

**Phase 3j — `str(Int)` formatting in the WASM backend — is implemented and green** (170 tests).
The `str` builtin on an `Int` now compiles: `codegen.rs` emits a synthetic `__int_to_str(n)` helper
(a sixth always-present helper) that formats a signed i64 as decimal into a fresh `[len:u32-le][ascii]`
buffer in the bump heap and returns its pointer — matching the interpreter's `i64::to_string`.
`i64::MIN` is handled by the standard unsigned-magnitude trick (`0 - n` wraps to the `i64::MIN` bit
pattern, formatted with unsigned division `i64.div_u`/`i64.rem_u`, which reads it as the correct
magnitude `9223372036854775808`); `str` on a `Str` is the identity. `str` composes with `__concat`
(`"n=" + str(n)` shares the same heap). Five new parity tests: a table of literals (0, ±small,
±large, i64::MAX), i64::MIN reached by computation, `str(fib(10)) == "55"`, and `"n=" + str(-7)`;
end-to-end `"fib(10) = " + str(fib(10))` prints `fib(10) = 55` identically on both engines.

**Lexer robustness (from the Phase 3i observation):** a leading UTF-8 BOM (U+FEFF) is now skipped as
trivia in `Lexer::new` rather than rejected with DL0101 — Windows editors and PowerShell's
`Set-Content -Encoding utf8` prepend one constantly. Only a *leading* BOM is trivia; a U+FEFF
elsewhere still lexes normally. Verified by a lexer test and end-to-end on a real BOM-prefixed file.

**Phase 3k — `Cap[Clock]` in the WASM host — is implemented and green** (173 tests). `root.clock()`
and `clk.now_ms()` now compile: `codegen.rs` gained a `Ty::Clock` handle and routes them to two new
`delulu:cap` host imports, `root_clock` (mints a Clock handle host-side iff granted — DL0703 if not,
like the console) and `clock_now_ms` (returns the clock as an `Int`). The clock is read **host-side**:
`HostConfig.fixed_clock_ms` fixes it for deterministic replay (the same value the interpreter's
`--clock fixed:MS` uses), else the wall clock — so under a fixed clock the two engines are byte-
identical. The import-index scheme was generalised (a `module_calls_method` scan drives which of the
console/clock import pairs the module declares; `Imports` carries the resolved indices through
codegen), and the host runner became `run_main(wasm, &HostConfig{console, clock, fixed_clock_ms})`
(`run_main_console` is now a thin wrapper). The CLI threads `grants.clock` + `--clock` into both
`run --engine wasm` and `run <file>.dwx`. Three new parity tests (now_ms under a fixed clock,
`"t=" + str(now_ms())` composing clock+str+concat, and an ungranted clock refused host-side);
end-to-end `"now = " + str(c.now_ms())` prints identically on both engines under `--clock fixed:MS`.

**Phase 3l — `Cap[Rand]` in the WASM host — is implemented and green** (176 tests). `root.rand()`
and `r.int(lo, hi)` now compile to the `delulu:cap` imports `root_rand` (mints a Rand handle iff
granted — DL0703 if not) and `rand_int(cap, lo, hi)`. The host runs the interpreter's **exact**
xorshift64 generator (`x ^= x<<13; x ^= x>>7; x ^= x<<17`), seeded identically (`(s==0 ? GOLDEN : s)
| 1`) via `HostConfig.rand_seed`, and maps each draw the same way (`lo + next % (hi−lo)`, guarded
`hi > lo` → DL0904, `wrapping_*` so the full-i64 span can't panic the host) — so under `--seed` the
two independent implementations (interpreter thread-local vs host struct) produce byte-identical
sequences. `codegen.rs` gained `Ty::Rand` and a `uses_rand` scan (`int` matches only the `Cap[Rand]`
*method*, not the free `int(float)` builtin, which is a `Call`); the CLI threads `grants.rand` +
`--seed` into `run --engine wasm` and `run <file>.dwx`. Three new parity tests (a four-draw sequence
that must agree across engines and land in range, same-seed determinism, and an ungranted rand
refused host-side); end-to-end a seeded dice program prints `d1=2 d2=5 d3=6` identically on both
engines under `--seed 42`. Clock + rand now complete the deterministic-capability pair.

**Phase 3m — two-engine conformance/differential parity harness — is implemented and green** (182
tests). A new integration file `crates/delulu-wasm/tests/conformance_parity.rs` consolidates
everything from 3a–3l into one gate, three ways. (1) **Curated programs**: an "everything together"
program exercising console + a pure helper + `str(Int)` + concat + `Cap[Clock]` + `Cap[Rand]` in a
single `main` (verified byte-identical, with `fib=55`/fixed-clock spot checks), plus negatives/
nested-arithmetic and deep-concatenation cases. (2) **Generative fuzzer**: `gen.rs` gained
`random_console_program`, a fault-free generator that `println`s `str(Int)` of safe arithmetic and
string concatenations; the harness runs **2000** of them on both engines and asserts byte-identical
output (>1500 actually compared, zero divergences). (3) **Fallback safety**: fragment-external
programs (a `match` body, list builtins `range`/`len`, an unsupported `Str` method) are asserted to
return a `CompileError` (DL1201) so the CLI runs them on the interpreter rather than miscompiling.
This is the "two-engine parity is the correctness contract" thesis turned into a comprehensive CI
gate — the automation the §9 criteria call for, over the fragment shipped so far.

**Phase 3n — secrets stay host-side (§4.4, DL1205) — is implemented and green** (186 tests). The
WASM backend now refuses secret-handling constructs with a dedicated diagnostic instead of the
generic DL1201: `CompileError` gained a `SecretInGuest` variant (`code()` → **DL1205**), and
`codegen.rs` returns it for `root.secret(...)` (minting a secret in the guest) and
`Secret.expose(...)` (revealing secret bytes to the guest). Because such a program does not compile,
**no guest WASM — and thus no guest linear-memory image — containing the secret is ever built**; it
runs on the interpreter, where secret bytes never cross into a guest. This is the strongest hygiene
statement available at this layer (the §9.8 memory-scan is vacuous when the bytes never enter). The
CLI maps `CompileError::code()` so `run --engine wasm` / `build --target wasm` on a secret program
reports DL1205 (verified end-to-end); DL1205's registry title was broadened to match. Four new tests
(a mint+expose program → DL1205, a mint-only program → DL1205, a secret-free positive control, and a
CLI `run --engine wasm` DL1205 case).

**Phase 3o — sum types in the WASM backend (Checkpoint 1: `Result`/`Option` + `match`) — is
implemented and green** (189 tests). The backend's thin `Ty` gained `Result(Scalar, Scalar)` and
`Option(Scalar)` (payloads restricted to scalars — `Int`/`Bool`/`Str`/`Unit`, no nesting yet, keeping
`Ty` `Copy`). A variant value is an i32 pointer to a heap `[tag:i32][field:i64]` cell (tag 0 =
Ok/None, 1 = Err/Some). Because the backend does no inference, construction is **expected-type-
directed**: a new `compile_expr_as`/`compile_block_as` threads the declared type into tail position,
so `Ok(x)`/`Err(e)`/`Some(x)`/`None` learn their full `Result`/`Option` type from the function's
return type (or a `match` scrutinee's known type). `match` lowers to a tag test (`i32.eqz`) with two
arms and typed field binding (`Ok(v)` loads the payload at the arm's width into a fresh local);
wildcard/`_` arms are supported. `match` on a non-variant (e.g. an `Int`) stays DL1201 (unchanged
fallback). Three parity tests (Result construct+match with `Ok`/`Err` binding; Option with a nested
expected-typed `if` and a `None` arm; a wildcard arm) — all byte-identical across engines; end-to-end
a `checkdiv → Result[Int,Str]` program prints `result = 42` / `error: division by zero` identically on
both engines. **Checkpoints 2–4 remain**: the `?` operator (early-return propagation), user-defined
enums (for `IoErr`), then the filesystem capability that composes them.

Remaining Phase 3 increments: sum-type Checkpoints 2–4 (`?`, user enums, then the filesystem
`delulu:cap` ops with host-side subtree scope checks); the rest of the hostile-guest matrix (§9.4 b —
fs subtree escape); the secret-hygiene scan proper (§9.8, once secrets can be *represented* in the
fragment); and the ≥50k both-engine fuzz gate (§9 criterion 9).

## 9. Acceptance criteria (Stage 3 is done when all pass)

1. The Stage-1 reference program builds to `demo.dwx` and runs from that single file on Windows,
   macOS, and Linux with identical output.
2. **Full conformance parity:** every Stage-1/2 conformance program runs on both engines with
   byte-identical stdout, identical exit codes, and identical effect traces under fixed
   `--seed`/`--clock` (criterion is automated: `delulu-conform --both-engines`).
   *(Partial — Phase 3m: `conformance_parity.rs` automates two-engine byte-identical output over the
   compilable fragment — curated console+str+concat+clock+rand programs plus a 2000-program
   generative fuzzer — and asserts fragment-external programs fall back via DL1201. Extending to the
   FULL corpus (match/Result/fs/secrets) waits on `Result`/variant codegen.)*
3. The laundering suite passes under `--engine wasm` (audit rules hold in compiled code).
4. **Hostile-guest test:** a hand-written WAT module importing `delulu:cap` attempts (a) forging
   cap externrefs from integers, (b) `fs-read-text` outside its granted subtree via `..` and
   symlink, (c) calling with a revoked grant id, (d) calling `secret-expose` with a non-declassify
   cap. All refused host-side with correct DL09xx traces; no host panic.
   *(Partial — Phase 3h: (a) forged handle → DL0904, out-of-bounds pointer → DL0903, negative
   pointer doesn't abort the host, and importing an unprovided capability fails to instantiate, are
   done in `crates/delulu-wasm/tests/hostile_guest.rs`. (d) is moot in a different way — Phase 3n:
   secret bytes never reach a guest because secret-handling code doesn't compile to WASM (DL1205), so
   there is no in-guest `secret-expose` to misuse. (b) subtree escape waits on the fs `delulu:cap` ops.)*
5. A `.dwx` with a stripped `delulu:authority` section refuses to run (DL1202).
6. Grant flow on `.dwx` matches source-run behavior exactly (same prompts, same DL0701/DL0702).
7. `delulu authority app.dwx` reports from the embedded section; grade flips
   verified↔contained with/without the local receipt (DL1203 on receipt mismatch).
8. **Secret-hygiene test:** after running a secret-handling program under wasm, a full scan of
   guest linear memory and GC heap contains no secret bytes; after `expose`, they appear (that is
   the definition working).
   *(Partial — Phase 3n: at the current fragment, secrets never enter a guest at all — a
   secret-handling program is refused with DL1205 and runs on the interpreter, so no guest memory
   image containing the secret is ever built (proven by test). The scan-and-`expose`-then-appears
   form of this test needs secrets to be *representable* in the compiled fragment, which waits on a
   host-mediated secret design in a later phase.)*
9. Fuzz harness (Stage 2 §7.2) extended to run accepted programs on **both** engines and diff
   traces: ≥ 50k programs, zero divergences (DL1206 class).
10. Interpreter remains the reference: any parity divergence is resolved by fixing an engine to
    match the *specified* semantics, never by "whichever is convenient" — recorded in the test's
    commit message.

---

## 10. Honesty and threat-model caveats (carry into docs verbatim)

- The WASM floor contains at **module granularity**; per-function precision is the type system's
  layer, not the sandbox's (Constitution §3, WASI comparison — unchanged and still true here).
- Wasmtime, the WASM-GC implementation, and the OS are trusted components of this layer;
  microarchitectural side channels (Spectre-class) remain out of scope (Constitution §5.14).
- Containment of *genuinely hostile* code should additionally use the Stage-5 microVM profile;
  Stage 3's floor is the default posture, not the maximum one.
- A foreign `.dwx` is contained-grade: its embedded manifest bounds what the host will permit,
  but nothing re-proves its internal per-function claims until Verified plugin loading (Stage 6).

*Stage 3 gives every DeluluLang program a floor. Stage 4 opens the door to Python and C — and
fences it. Stage 5 moves the keys out of the process. Stage 6 makes plugins real.*
