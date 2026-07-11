# DeluluLang — Stage 4 Implementation Specification

**Version:** 0.4 ("Foreign"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 3 complete (both engines at parity; foreign calls must ship on both).
**Governing documents:** `CONSTITUTION.md` (§5.12, §8.2), `SOUNDNESS_AUDIT.md` (R-4, R-5, R-6a).

---

## 0. Scope and goal

**Goal:** first-class interop with **C and Python** — the adoption lever — behind an honest,
capability-gated fence. The `ForeignCall` effect and `Cap`-gated foreign handles activate; the
whole-program guarantee degrades **visibly in the types**, never silently. After this stage a
DeluluLang program can call `libm`, NumPy, or PyTorch, and `delulu authority` enumerates exactly
where the proof has holes.

**In scope (must ship):**
- The `foreign` declaration block (C ABI), activated `foreign` keyword, `ForeignCall` effect.
- Runtime binding of C shared libraries (libffi-based), scalar+string marshalling, `ForeignPtr`.
- Embedded CPython (`Cap[Python]`, `PyObj`, explicit conversions, module allowlist).
- Manifest/grant extensions (`foreign.c`, `foreign.python`) and the grant prompt for them.
- Both engines: interpreter in-process; WASM engine calls foreign code **host-side** via the
  `delulu:foreign@0.4` host interface (the guest never touches raw pointers).
- Authority report extension: `foreign_calls` populated (Constitution: "holes in the proof,
  enumerated").
- New diagnostics: DL13xx.

**Non-goals (later stages):** foreign-worker process isolation and microVM containment of
foreign code (Stage 5 — until then, foreign code runs with full process authority and the docs
say so at every opportunity); callbacks into foreign code (refused by rule, not deferred — see
§4.4); C structs by value, varargs, non-UTF-8 strings (post-1.0 RFCs); transpilation of foreign
code into DeluluLang (explicitly out, Constitution §8.2).

---

## 1. Invariants (carried + new)

All Stage-1/2/3 invariants hold. New:

19. **No foreign reachability without a grant.** There is no way to execute one foreign
    instruction without: a `foreign.c`/`foreign.python` manifest entry, a runtime grant, a
    capability value threaded from `Root`, and `ForeignCall` in every row on the call path.
20. **Secrets never cross unexposed.** No marshaller accepts `Secret[T]`, `Cap[R]`, `Root`,
    `Plugin[_]`, `PyObj`-containing composites, or any opaque type (R-5 extended to the FFI
    boundary at compile time, DL1301).
21. **Foreign data is untrusted.** Every value returned from foreign code is validated at the
    boundary (UTF-8 checked, lengths bounded by `--foreign-max-ret` default 64 MiB, NaN/Inf
    passed through as ordinary `Float` values); a validation failure is a `Result` error, never
    UB and never a panic.
22. **No foreign function pointers cross in either direction.** DeluluLang closures never become
    C function pointers or Python callables; foreign code cannot call back into the program
    (audit R-6a reasoning: callback timing in unverifiable code cannot be bounded by any row).

---

## 2. Lexical, grammar, and AST additions

`foreign` moves from reserved to active. No other token changes.

```ebnf
item          = [ "pub" ] , ( fn_decl | type_decl | effect_decl | const_decl | foreign_decl ) ;
foreign_decl  = "foreign" , STRING , "lib" , IDENT , "{" , { foreign_fn } , "}" ;
                (* STRING is the ABI: "c" is the only value in v0.4; others are DL1308 *)
foreign_fn    = "fn" , IDENT , "(" , [ params ] , ")" , [ "->" , type ] , term ;
                (* no effect_row syntax: the row is implicitly !{ForeignCall}, always *)
```

```rust
pub struct ForeignDecl {
    pub public: bool,
    pub abi: String,               // "c"
    pub name: Ident,               // nominal lib-shape type AND logical grant name
    pub fns: Vec<ForeignFn>,
    pub id: NodeId, pub span: Span,
}
pub struct ForeignFn { pub name: Ident, pub params: Vec<Param>, pub ret: Option<TypeExpr>, pub span: Span }
```

A `foreign … lib M { … }` block introduces a **nominal opaque type `M`** (the lib handle). New
prelude types: `ForeignPtr` (opaque, R-5), `ForeignErr`, `PyErr` (sums in `std.foreign` /
`std.py`), `PyObj` (opaque), `Cap[Python]` (new `ResourceKind::Python`), `Cap[ForeignLoad]`
(gates binding).

---

## 3. Typing rules (delta)

- **T-ForeignSig (marshallability):** every parameter and return type in a `foreign_fn` must
  satisfy `M(τ)`: `M(Int) M(Float) M(Bool) M(Str) M(Unit) M(ForeignPtr)` and nothing else
  (DL1301, span on the offending type; the message names the rule and links E-DL1301). Function
  types are called out specially: DL1302 with the R-6a explanation.
- **T-ForeignBind:** `root.foreign[M](load: Cap[ForeignLoad]) -> Result[M, ForeignErr]` — pure
  (binding is attenuation-like: deriving a handle is not an effect; *using* it is). Fails at
  runtime if the logical name `M` has no grant (DL1303) or a declared symbol is missing (DL1304 —
  **all symbols resolve at bind time**, fail-fast, never mid-run).
- **T-ForeignCall:** a method call on a lib-handle value `m.cos(x)` types per the block's
  signature with row exactly `{ForeignCall}`.
- **T-Py:** every `Cap[Python]` and `PyObj` operation (§5.2) has row `{ForeignCall}`. `PyObj` is
  opaque: no `str`, no `==`, no serialization, cannot appear in a `foreign "c"` signature.
- `ForeignCall` is an ordinary effect for all row purposes (union, polymorphism, manifest
  ceilings) — nothing special-cases it except reporting.

---

## 4. Runtime — C FFI

### 4.1 Manifest and grants

```toml
[authority]
effects   = ["ForeignCall"]
foreign.c = ["mathlib"]                  # logical names = foreign block names
```

```
--grant foreign.c=mathlib:/usr/lib/libm.so.6      # logical:path (path chosen by the human, not the code)
--grant foreign.c=mathlib:libm.dll                # Windows; bare names resolve via OS loader rules
```

The **path is grant data, not program data** — the program names *what* it wants (`mathlib`,
`cos`, `sqrt`); the human/broker decides *which binary* satisfies it. The grant prompt shows the
full symbol list before asking.

### 4.2 Call mechanics

`libloading` for binding, `libffi` for calls (signatures are runtime data). Marshalling:
`Int`→`int64_t`, `Float`→`double`, `Bool`→`int32_t` (0/1), `Str`→`(const uint8_t*, size_t)`
borrowed for the call duration (callee must not retain — documented contract; retention is
outside any enforceable model and the docs say so), `Str` return→`(ptr,len)` copied then
validated, `ForeignPtr`→`void*` passed opaquely. Every call is wrapped: panics/segfaults inside
foreign code are **out of scope** (a C library can do anything to the process — invariant 21
covers data, not memory safety; the honest fix is Stage 5 worker isolation, and DL13xx docs link
forward to it).

### 4.3 WASM engine

Guest calls `delulu:foreign@0.4` host functions (`bind`, `call(lib, sym-index, args: list<fval>)
-> result<fval, ferr>` with `fval` a scalar/string variant). All of §4.1–4.2 runs host-side;
parity criterion applies (§9.6).

### 4.4 No callbacks — permanent rule, not a gap

Function-typed parameters anywhere in a foreign signature: DL1302. `PyObj.call` with a
DeluluLang closure argument: DL1302. This is R-6a applied to FFI and it is **rule, not debt**:
unverifiable code holding a re-entry point into verified code destroys row soundness (audit F-6).
The escape valve for "Python needs to call my code" is inverted control: DeluluLang drives the
loop and passes data, not code.

---

## 5. Runtime — embedded CPython

### 5.1 Binding

PyO3 with `auto-initialize` off; `root.python(load: Cap[ForeignLoad]) -> Result[Cap[Python],
ForeignErr]` initializes the interpreter on first grant-checked call (interpreter unavailable →
DL1307 as a `ForeignErr` variant, not a crash). Manifest / grant:

```toml
[authority]
foreign.python = ["numpy", "numpy.*", "torch"]    # import allowlist patterns
```

`--grant foreign.python=numpy --grant foreign.python="numpy.*"`.

### 5.2 API surface (`std.py`) — all `! {ForeignCall}`

```delulu
py.import(name: Str)                    -> Result[PyObj, PyErr]   // name must match allowlist (DL1305)
py.of_int(i: Int) / of_float / of_str / of_bool                  -> PyObj
py.list(xs: List[PyObj]) -> PyObj
py.to_int(o: PyObj) -> Result[Int, PyErr]      // + to_float, to_str, to_bool
obj.attr(name: Str) -> Result[PyObj, PyErr]
obj.call(args: List[PyObj]) -> Result[PyObj, PyErr]
obj.call_method(name: Str, args: List[PyObj]) -> Result[PyObj, PyErr]
obj.index(key: PyObj) -> Result[PyObj, PyErr]
```

Python exceptions become `PyErr { kind: Str, message: Str }` values — message text is untrusted
foreign data (length-bounded per invariant 21).

### 5.3 The allowlist honesty note (normative, appears in docs and in `delulu explain E-DL1305`)

The import allowlist gates **the interface** — what the DeluluLang program may reach for by name.
It does **not** bound what Python code transitively imports or does once running: embedded Python
has full process authority at the OS level. The real bounds are (a) the reachability gate (no
`Cap[Python]`, no Python at all) and (b) the process/worker/microVM layer (Stage 5). This is the
Constitution §5.12 degradation, stated where users will actually read it.

---

## 6. Authority report extension

```json
"foreign_calls": [
  { "abi": "c", "lib": "mathlib", "symbols": ["cos", "sqrt"],
    "granted_path": "/usr/lib/libm.so.6", "used_at": [ {"file": "...", "line": 9} ] },
  { "abi": "python", "allowlist": ["numpy", "numpy.*"],
    "imports_seen": ["numpy"], "used_at": [ ... ] }
]
```

`delulu why ForeignCall` works like any effect. Human-mode `delulu authority` prints foreign
entries under a separator line: `-- outside the proof (contained at process level) --`.

---

## 7. Diagnostics (fresh range DL13xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1301 | unmarshallable type in foreign signature (incl. Secret/opaque) | none — `requires_human: true`; never suggests `expose` |
| DL1302 | function-typed value crossing the foreign boundary (either direction) | none — rule R-6a; explanation links audit |
| DL1303 | foreign lib/python used without manifest entry or grant | add manifest entry — `authority_widening: true` |
| DL1304 | symbol not found at bind time | none — `requires_human: true` |
| DL1305 | python import not in granted allowlist | add allowlist pattern — `authority_widening: true` |
| DL1306 | foreign return validation failure (encoding/size) — runtime, surfaces as `ForeignErr`/`PyErr` | — |
| DL1307 | python runtime unavailable | none — `requires_human: true` |
| DL1308 | unsupported ABI string | none |

---

## 8. Standard library additions

`std.foreign`: `ForeignPtr`, `ForeignErr = NotGranted | SymbolMissing(Str) | BadReturn(Str) |
Unavailable(Str)`. `std.py`: `PyObj`, `PyErr`, the §5.2 surface. Nothing else.

---

## 9. Acceptance criteria

1. `foreign "c" lib mathlib { fn cos(x: Float) -> Float }` + grant → `m.cos(1.0)` returns the
   right value on all three OSes, on both engines, with `ForeignCall` in the trace.
2. NumPy demo: `py.import("numpy")`, build a list, call `mean`, convert back — the Constitution's
   "inherit the AI ecosystem" claim, running.
3. `fn(cb: fn(Int) -> Int ! e)` in a foreign block → DL1302; `Secret[Str]` param → DL1301 and the
   repair list does **not** contain `expose`.
4. Unbound logical lib → DL1303 at startup (grant flow), not mid-run; missing symbol → DL1304 at
   bind, not at call.
5. `py.import("os")` under an allowlist of `["numpy"]` → DL1305 as a runtime `PyErr`; the trace
   logs the denied attempt.
6. Engine parity: foreign conformance programs produce identical results and traces on
   interpreter and WASM engines (Python pinned to one version in CI).
7. `delulu authority` lists both foreign entries with the "outside the proof" separator; a
   program with no foreign use has `"foreign_calls": []` and its report is unchanged from Stage 3.
8. A C function that returns invalid UTF-8 or an oversized buffer yields `ForeignErr::BadReturn`,
   never a crash or truncated silent success.

## 10. Honesty and threat-model caveats (carry into docs verbatim)

- Foreign code is **outside the effect guarantee**; the language bounds *reachability*
  (grant + capability + `ForeignCall` in every row), not behavior. Containment of behavior is
  process-level until Stage 5's foreign workers, and microVM-level after.
- A C library can corrupt or crash the process; memory safety across the FFI is not claimed.
- The Python allowlist is an interface gate, not a transitive-import bound (§5.3).
- Returned foreign data is validated for *shape*, not *meaning* — it is untrusted input and
  programs should treat it accordingly.

*Stage 4 opens the door and paints a bright line around it. Stage 5 moves the keys out of the
house.*

---

## 11. Implementation status (2026-07-11)

**Phase 4a — grammar + AST for `foreign` blocks (parse only) — is implemented and green** (205
workspace tests, +5). `foreign` moves from reserved to an **active contextual keyword**: it is
still lexed as an identifier (so `root.foreign(…)` stays a legal member access and `lib`/the ABI
string need no new tokens — spec §2's "no other token changes"), and the parser recognizes
`foreign STRING lib IDENT { … }` at item position (`crates/delulu-syntax/src/parser.rs`,
`parse_foreign_decl`/`parse_foreign_fn`). New AST nodes `ForeignDecl`/`ForeignFn` and the
`Item::Foreign` variant (`ast.rs`) carry the ABI string (with its span, for DL1308), the lib name
(the nominal handle type *and* the logical grant name), and each function's params/return. A
foreign function has **no effect-row syntax** — its row is implicitly `!{ForeignCall}`, always; a
written `!` row there is a parse error (DL0201, "a foreign function has no effect row"). A
non-`"c"` ABI is **DL1308** (registered in `crates/delulu-diag/src/codes.rs`, the first DL13xx),
reported on the ABI-string span at parse time. Tests (`parser.rs`): round-trip of a two-function
block, `pub foreign … { }`, non-`"c"` ABI → DL1308, an effect row on a foreign fn → parse error,
and the regression that `root.foreign(…)` still parses as a method. No type-checking of the block
yet (that is 4b/4c) — this phase is purely lexer/parser/AST + the DL1308 diagnostic.

**Phase 4b — the marshallability fence (T-ForeignSig) + the new prelude types — is implemented
and green** (212 workspace tests, +7). New prelude/opaque types: `ResourceKind::Python` and
`ResourceKind::ForeignLoad` (so `Cap[Python]`/`Cap[ForeignLoad]` lower through the existing
`Cap[…]` path); `Type::ForeignPtr` and `Type::PyObj` (R-5 opaque leaf types); `Type::Foreign(M)`,
the nominal opaque handle introduced by each `foreign … lib M` block (registered in
`DeclTable.foreigns`, resolvable as the type name `M`); and the stdlib sums/records `ForeignErr`
(`NotGranted | SymbolMissing(Str) | BadReturn(Str) | Unavailable(Str)`) and `PyErr { kind: Str,
message: Str }`, appended to the prelude in `resolve.rs`, `program.rs`, and `deps.rs`. `is_opaque`
now covers `ForeignPtr`/`PyObj`/`Foreign` (so `str`/`==`/serialize on any of them is
DL0604/DL0605, R-5). The fence itself (`check.rs::check_marshallable`, run over every foreign
signature in `check_module`): a function type **anywhere** in a parameter/return type — including
nested inside a composite like `List[fn() -> Int]` — is **DL1302** (invariant 22 / R-6a, message
names the rule); otherwise the type must be exactly one of `Int Float Bool Str Unit ForeignPtr`
(no type arguments) or it is **DL1301** (span on the offending type). **Both DL1301 and DL1302 are
emitted with no repairs** — they are `requires_human` situations (spec §7), and an empty repair
list is exactly how every other `requires_human` code in the compiler is expressed, which also
guarantees DL1301 can never suggest `expose` to launder a secret across the FFI (criterion 3).
Tests (`lib.rs`): a fully-marshallable block checks clean; `ForeignPtr` marshals while `PyObj`
does not; `Secret[Str]` → DL1301 with **no `expose`** anywhere in its repairs; `Cap`/lib
handle/`List[..]`/return-`List` → DL1301; `fn(Int)->Int` param → DL1302 referencing R-6a with no
repairs; a nested function type → DL1302; and stringifying a lib handle → DL0604.

**Phase 4c — `ForeignCall` effect + T-ForeignBind + T-ForeignCall (checking, no runtime) — is
implemented and green** (218 workspace tests, +6). `Effect::ForeignCall` is now an **ordinary
core effect** (added to `Effect::core_from_name`/`name`): it unions, is row-polymorphic, is
checked against the manifest ceiling, and is reported by `delulu authority`/`delulu why` with no
special-casing — everything is string/`name()`-driven, so those paths needed no edits. **T-
ForeignBind** (`check.rs`, the `Type::Root` method arm): `root.foreign(load: Cap[ForeignLoad]) ->
Result[M, ForeignErr]` is **pure** — deriving a handle carries no effect. The `[M]` of the
normative signature is realized as inference-from-context (a fresh type variable resolved by the
binding's annotation/use), because the grammar has no method type-argument syntax; e.g. `let m:
mathlib = root.foreign(load)?`. **T-ForeignCall** (the new `Type::Foreign(M)` receiver arm in
`method_sig`): a call `m.cos(x)` looks the method up in the block's `foreign` signatures, checks
argument arity/types, and returns the declared result type with effect **exactly `ForeignCall`**;
an unknown method falls through to DL0405 as usual. Because `ForeignCall` is a normal effect, an
undeclared foreign call is caught by the existing T-Fn boundary check as **DL0501**, and it flows
through the call graph unchanged. Tests (`lib.rs`): `m.cos` without `ForeignCall` in the row →
DL0501; with it declared → clean and `ForeignCall` in the function's facts; a mistyped foreign
argument is rejected; `ForeignCall` propagates `main → mid → leaf → m.cos` (the chain `delulu why`
walks); `root.foreign(load)` is pure; and a program that never touches foreign code has no
effects and `"foreign_calls": []` — its authority report is unchanged. End-to-end CLI spot-check:
`delulu why ForeignCall` prints the `main → compute — ForeignCall` chain, and `delulu explain`
resolves DL1301/DL1302/DL1308.

**Phase 4d — C FFI runtime (interpreter engine) — is implemented and green** (with 4e: 249
workspace tests, +31 over 4c). All native-dependency code (`libloading` 0.8 for binding, `libffi`
3.x for calls — signatures are runtime data) is isolated in **`crates/delulu-runtime/src/foreign.rs`**;
nothing outside that module touches a raw pointer or a `Library`, and every `unsafe` block carries a
`// SAFETY:` comment stating its invariant. The 4a–4c open question is **resolved by head-chef
ruling**: `Cap[ForeignLoad]` is minted by **`root.foreign_load() -> Cap[ForeignLoad]`** — already
normative in the Book (ch. 14) — a pure derivation from `Root` following the same broker
grant-check pattern as every other `root.X()` constructor (`method_sig` Root arm; `prim.rs`
`call_root_method`; refused with DL0703 when no `foreign.c` lib is granted). Binding
(`root.foreign(load)`, `interp.rs::bind_foreign`): the checker records which lib each bind site's
inferred `[M]` resolves to (`CheckResult::foreign_binds`, keyed by the call's `NodeId` — the
grammar has no method type-argument syntax, so the runtime needs the resolved name); the
interpreter looks up the grant path and **resolves ALL declared symbols at bind time** — a missing
symbol is `ForeignErr::SymbolMissing(name)` (DL1304's runtime face, fail-fast, never mid-run), an
unloadable binary is `Unavailable`, an ungranted lib is `NotGranted` (belt-and-suspenders behind
the CLI startup refusal) — all `Result` values, never panics. Calls (`interp.rs::call_foreign` →
`foreign::call`) marshal per §4.2 exactly: `Int`→`int64_t`, `Float`→`double`, `Bool`→`int32_t`
(0/1), `Str`→borrowed `(const uint8_t*, size_t)` for the call duration, `Str` return→`(ptr,len)`
copied THEN validated, `ForeignPtr`→opaque `void*` (a `Value::ForeignPtr(usize)` — no raw pointer
in general evaluation). Return validation (invariant 21, `foreign::validate_c_string`): UTF-8
checked, length bounded by `--foreign-max-ret` (default 64 MiB); a failure is a defined **DL1306**
fault carrying `ForeignErr::BadReturn` — never UB, never a panic, never a silent truncation
(criterion 8; a bare `-> Str` foreign signature has no `Result` channel, so the honest outcome is
a clean abort). Every foreign call records a **`ForeignCall` trace event** (criterion 1) through
the shared seq counter (`cap_kind` = the lib name). *Fixture strategy:* a C-ABI cdylib compiled
once per test run with `rustc --crate-type cdylib` into `CARGO_TARGET_TMPDIR`
(`crates/delulu-runtime/tests/foreign_ffi.rs`) — `rustc` is guaranteed present wherever `cargo
test` runs, unlike `cl.exe`/`gcc`, and an `extern "C"` cdylib is a real C-ABI library on all three
OSes; the validation layer is additionally unit-tested against crafted byte buffers in
`foreign.rs` (invalid UTF-8, oversized/unterminated, null, empty), independent of any library.
Nine live-DLL tests cover: `cos` round-trip with `ForeignCall` traced; fail-fast `SymbolMissing`
with **zero** foreign records in the trace; `NotGranted`; DL0703 with no grant at all;
`Unavailable`; the full marshalling matrix (Int/Bool/Str-as-(ptr,len)/ForeignPtr round-trips);
copied-then-validated `Str` returns; and both criterion-8 hostile returns as DL1306 faults.

**Phase 4e — manifest + grants + grant prompt + authority report — is implemented and green**
(249 workspace tests total with 4d). Manifest: `[authority] foreign.c = ["mathlib"]` (logical
names) parses in both the checker's manifest and the runtime broker; a program binding a lib not
listed there is refused **DL1303 at startup**. Grants: `--grant foreign.c=mathlib:PATH` — split on
the **first** colon only, so Windows drive-letter paths (`mathlib:C:\libs\m.dll`) survive; bare
names resolve via OS loader rules. **The path is grant data, not program data.** The grant flow
(`cli.rs::foreign_grant_preflight`) runs before `main`: manifest ceiling → grant lookup →
(interactive terminals only — never under `--json`, `--no-prompt`, or a non-terminal, so agents
are never prompted) a prompt showing the **full symbol list** before asking for a binary path →
otherwise DL1303 with the exact `--grant` invocation to use (criterion 4: at the grant flow, never
mid-run). `delulu authority`: the `foreign_calls` array per §6 (`abi`, `lib`, `symbols`,
`granted_path` — `null` in the static report since the path is a runtime human decision —
`used_at` at block-declaration granularity, the same altitude `delulu why` reports at), and the
human-mode separator line exactly `-- outside the proof (contained at process level) --`.
`ForeignLoad`/`Python` capability kinds are deliberately **excluded from the ordinary
`capabilities` rows** (like `Declassify`/`PluginHost`) — the foreign section under the separator
is the single place the proof's holes are enumerated. **Criterion 7 is pinned byte-for-byte**: a
CLI test asserts the human authority report of a no-foreign program equals the Stage-3 output
verbatim and `"foreign_calls": []` in JSON. DL1303/DL1304/DL1306 are registered in `codes.rs`, and
`delulu explain` now prints a long-form explanation for every DL13xx code (`codes.rs::code_explain`)
carrying the §10 honesty caveats verbatim — reachability-not-behavior, the Stage-5 forward link on
every code, no `expose` suggestion on DL1301 — with a CLI test asserting the word "sandbox" appears
in none of them. Side task done: the `delulu why` unknown-effect help now lists `ForeignCall` with
the six pre-Stage-4 core effects. `--foreign-max-ret <bytes>` is a `run` option.

**Phase 4f — embedded CPython (`std.py`) — is implemented and green** (263 workspace tests, +14 over
4e; the three live-interpreter tests skip-with-a-printed-notice where CPython/NumPy are absent but RUN
for real where present). All PyO3 code is isolated in **`crates/delulu-runtime/src/python.rs`** behind
a **`python` cargo feature that is ON by default** in the runtime and forwarded on by the `delulu`
CLI's own default `python` feature; `--no-default-features` builds a Python-less `delulu` where
`root.python(...)` returns `ForeignErr::Unavailable` (DL1307) — the same user-visible outcome as a
missing interpreter. PyO3 runs with `auto-initialize` OFF: the embedded interpreter is prepared lazily
by `python::ensure_available` (`prepare_freethreaded_python`, guarded by `catch_unwind` so a
non-starting interpreter is a returned `Err`, never a crash) on the first grant-checked `root.python`
call. **Ownership/GIL model** (documented in `python.rs`): a `PyObj` value is a `Py<PyAny>` —
GIL-independent, stored/cloned/dropped off the GIL; **every** Python touch is inside
`Python::with_gil`, and only `Py<T>` or plain scalars ever cross back into the interpreter (no
`Bound<'py>` escapes), so a `PyObj` between operations holds no lock. **Binding:**
`root.python(load: Cap[ForeignLoad]) -> Result[Cap[Python], ForeignErr]` is PURE (like `root.foreign`;
`Cap[ForeignLoad]` is now minted whenever `foreign.c` **or** `foreign.python` is granted); ungranted →
`Err(NotGranted)` (DL1303's runtime face) behind a CLI grant pre-flight, unavailable → `Err(Unavailable)`
(DL1307). **Manifest/grant:** `[authority] foreign.python = ["numpy", "numpy.*"]`;
`--grant foreign.python=numpy --grant "foreign.python=numpy.*"` (patterns are an exact name or a
`prefix.*` wildcard); the granted allowlist is carried in `CapScope::Python` and checked at
`py.import`. **The `std.py` surface** (`Cap[Python]`: `import`/`of_int`/`of_float`/`of_str`/`of_bool`/
`list`/`to_int`/`to_float`/`to_str`/`to_bool`; `PyObj`: `attr`/`call`/`call_method`/`index`) types in
`check.rs` `method_sig` with row exactly `{ForeignCall}`; each op records a `py.<method>` `ForeignCall`
trace event, appended BEFORE the call so a **denied** `py.import` (allowlist-checked → DL1305 as a
runtime `PyErr` value) is still visible in `--trace-effects` (criterion 5). `PyObj` is opaque (R-5:
DL0604/DL0605) and **not** marshallable in a `foreign "c"` signature (DL1301, tested). A DeluluLang
closure can never become a `PyObj` (spec §4.4 / R-6a): a function type unified against `PyObj` is
**DL1302** (the R-6a diagnostic, not a plain mismatch) — enforced both as a Python-argument fence
(`reject_py_callback`, catching a pure `List[fn]`) and generally in `expect_type` (`fn_pyobj_mismatch`,
catching a closure element inside a `List[PyObj]` literal). Python exception → `PyErr { kind, message }`
where `message` is UNTRUSTED foreign data, length-bounded by `--foreign-max-ret` (invariant 21).
`import` moves from a full keyword to a legal **member name** (`py.import`): `token.rs`'s new
`keyword_lexeme` lets any reserved word be a member after `.` (member position is never a declaration
site). DL1305/DL1307 are registered in `codes.rs`; `delulu explain DL1305` carries spec §5.3's allowlist
honesty note verbatim (the allowlist gates the *interface*, not transitive imports; embedded Python has
full process authority; real bounds are the reachability gate + Stage 5) and the word "sandbox" appears
in none of the DL13xx texts. **`delulu authority`** adds the `{abi:"python", allowlist, imports_seen,
used_at}` entry (spec §6) under the same "outside the proof" separator (`imports_seen` = the
statically-known `py.import("literal")` names; a program that never reaches `root.python` is unchanged,
criterion 7 preserved). **Criterion 2** — the NumPy demo (`py.import("numpy")`, build a list, call
`mean`, convert back, print `2.5`) — is committed as `examples/numpy_mean.delulu` and runs live on this
machine (**CPython 3.13.5, NumPy 2.2.6**, PyO3 0.25).

**Phase 4g — the WASM `delulu:foreign@0.4` host interface + two-engine parity — is implemented and
green** (271 workspace tests, +8 over 4f). **Stage 4 is code-complete pending head-chef close-out.**
The guest calls a four-function `delulu:foreign` host interface (`foreign_load`, `foreign_bind`,
`foreign_call`, `float_to_str`); **all of §4.1–4.2 runs host-side** — grants, bind-time symbol
resolution (DL1304 fail-fast), marshalling, and return validation (`--foreign-max-ret`, DL1306) — and
the guest never touches a raw pointer. The host (`crates/delulu-wasm/src/host.rs`) **reuses the
interpreter's own FFI machinery** (`delulu-runtime::foreign::{load_and_resolve, call}` and its
`lower_foreign_sig`, now `pub`), so verify≡run stays one code path; `delulu-runtime` is a real
dependency of `delulu-wasm` with `default-features = false` (no Python in the WASM crate). Lib handles
and `Cap[ForeignLoad]` are host-table indices like every Stage-3 cap (`CapKind::Foreign`/
`ForeignLoad`); a returned `ForeignPtr` is an index into a host-side pointer table — a raw `void*`
never enters guest memory. `foreign_bind` CONSTRUCTS the `Result[M, ForeignErr]` cell in guest memory
(tag order `NotGranted=0 | SymbolMissing=1 | BadReturn=2 | Unavailable=3` pinned against the
interpreter's sum); arguments cross in a 16-byte-cell tagged buffer the host decodes with fully
checked reads (a hostile buffer is a recorded refusal, never a host panic). **Every policy failure
follows the carried never-Err-from-a-Wasmtime-callback rule** — recorded in `HostState.refused`,
surfaced after execution by `finish()` (DL0703 ungranted loader, DL0904 forged handle, DL0903
out-of-bounds, DL1306 bad return). Codegen (`codegen.rs::compile_module_with`) takes the checker's
`foreign_binds` NodeId→lib map (same recovery the interpreter uses); `Float` literals, `str(Float)`
(host-formatted with `Value::display`'s exact logic), and `str(Bool)` join the fragment. **Traces are
recorded host-side** (criterion 6's "identical traces"): every effect host fn now carries the call's
`(file, start, end)` span, and the host appends `TraceRecord`s — same seq counter, effect, op,
cap_kind, detail, span as the interpreter — exposed through `HostConfig.trace`; the CLI's
`--engine wasm` now supports `--trace-effects`/`--trace-out`/`--assert-trace` and threads
`foreign_grants`/`foreign_sigs`/`--foreign-max-ret` into the host. **Criterion 6 evidence:**
`crates/delulu-wasm/tests/foreign_parity.rs` (8 tests, same rustc-built cdylib fixture as
`foreign_ffi.rs`) asserts byte-identical results AND `TraceRecord`-identical traces on both engines
for: the C `cos` happy path, DL1303 no-grant (`Err(NotGranted)`, zero foreign trace records),
DL1304 missing-symbol fail-fast, both criterion-8 hostile returns (DL1306 on both engines, the
attempted call still traced), the full marshalling matrix (Int/Bool/Str/ForeignPtr, six identical
`ForeignCall` records), and the copied-then-validated `Str` return; verified end-to-end through the
real CLI (`delulu run … --grant foreign.c=… --trace-effects` vs `--engine wasm …`) with
`diff`-identical stdout and trace files. **Python on WASM — the sanctioned fallback was taken:**
`root.python`/`py.*` on the WASM backend is the existing **DL1201** "runs on the interpreter"
mechanism (tested), NOT a half-port — the deciding constraint is that `py.list(xs: List[PyObj])`,
required by the criterion-2 NumPy demo, needs guest-side `List` values, which the WASM fragment does
not have; Python-on-WASM would ship without the flagship demo, which the ruling forbids. Python
remains pinned **CPython 3.13.5** on this machine (4f). `.dwx`: `build --target wasm` now embeds the
`foreign_calls` array (spec §6) in the authority section, and a foreign-using `.dwx` compiles+verifies;
*running* one refuses cleanly at `root.foreign_load()` (DL0703 via the finish() pattern) because a
`.dwx` carries no marshalling signatures to bind — an honest, documented limitation (signature
embedding is a natural Stage-6 `.dpx`-adjacent extension). The Stage-3 hostile-guest suite was
extended for the new span-carrying `console_println` signature and stays green (forged handles,
out-of-bounds/negative pointers, unprovided imports).

### Head-chef close-out (2026-07-11) — STAGE 4 COMPLETE

All seven phases landed (`c7b043d`→`c4698fe`), 200 → **271 workspace tests, 0 failures**. Every
acceptance criterion was re-verified by the head chef against the real binary, not the reports:

| # | Criterion | Status |
|---|---|---|
| 1 | C call, both engines, `ForeignCall` traced | **pass** (Windows; the rustc-built-cdylib fixture design is 3-OS-portable, but the Linux/macOS CI matrix has not run yet — honest caveat, tracked for Stage 8/9 CI work) |
| 2 | NumPy demo | **pass, live** — `examples/numpy_mean.delulu` → `mean = 2.5` (CPython 3.13.5, NumPy 2.2.6) |
| 3 | Fence: DL1302 callbacks (incl. nested), DL1301 secrets with **no `expose` repair** | **pass, live** (empty repairs list — structurally no `expose`) |
| 4 | DL1303 at grant flow / DL1304 at bind, never mid-run | **pass, live** |
| 5 | `py.import("os")` off-allowlist → DL1305 PyErr, denied attempt traced | **pass, live** |
| 6 | Engine parity: identical results AND traces | **pass, live** — head chef diffed stdout + trace files across engines: byte-identical; `--assert-trace` green on WASM. Python ops on WASM are DL1201 interpreter-fallback (documented above) |
| 7 | Authority separator + `"foreign_calls": []` byte-identical no-foreign report | **pass, live** + pinned regression test |
| 8 | Hostile returns → `ForeignErr::BadReturn`/DL1306, never a crash | **pass** (both engines, crafted-buffer unit tests + live-DLL tests) |

**Rulings on the flagged open questions:** (1) a foreign-using `.dwx` that compiles/verifies but
*refuses cleanly at run* (no embedded marshalling signatures) is **accepted for Stage 4** — signature
embedding lands with Stage 6's `.dpx` custom-section machinery, not before. (2) The span-carrying
host-fn signature change that invalidates pre-Stage-4 `.dwx` artifacts is **accepted pre-1.0**; host
interface versioning is formalized when Stage 6 versions `delulu:plugin`. (3) `PyErr.kind =
"ImportNotAllowed"` is the blessed DL1305 runtime spelling; `imports_seen` stays static. Side
benefit shipped: `--engine wasm` now records traces for all effects, closing a Stage-3 tooling gap.

*Stage 4 opened the door and painted the bright line around it. Stage 5 moves the keys out of the
house.*
