# Stage 4 Playbook — "Foreign" (C FFI + embedded Python)

**Companion to:** `docs/design/STAGE4_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 3 complete (both engines at parity — foreign calls ship on both). **This is
the next stage to build after Stage 3**, so treat this playbook as the immediate working plan.

> **The one-sentence goal:** first-class interop with **C and Python** — the adoption lever — behind
> an honest, capability-gated fence. `ForeignCall` and `Cap`-gated foreign handles activate; the
> whole-program guarantee **degrades visibly in the types, never silently.** After this stage
> `delulu authority` enumerates *exactly where the proof has holes* under a
> `-- outside the proof (contained at process level) --` separator.

---

## 0. Orientation — the one idea that governs everything

**DeluluLang bounds foreign *reachability*, not foreign *behavior*.** The language guarantees you
cannot execute one foreign instruction without (a) a manifest entry, (b) a runtime grant, (c) a
capability value threaded from `Root`, and (d) `ForeignCall` in every row on the call path
(invariant 19). Once those four gates are passed, the C library or Python interpreter runs with
**full process authority** — and every doc, diagnostic, and `delulu authority` line says so plainly.
Containment of *behavior* is Stage 5's foreign-worker process (and Stage 5's microVM after). This
stage's honesty is its whole point: **do not overclaim.** The word "sandbox" does not belong
anywhere near Stage 4 foreign code.

Read before writing:
- Spec §1 invariants 19–22 (reachability gate; secrets never cross unexposed; foreign data
  untrusted; no function pointers either direction).
- Spec §3 typing delta (T-ForeignSig marshallability, T-ForeignBind, T-ForeignCall, T-Py).
- Spec §4.4 (**no callbacks — a permanent rule, not a deferred feature**) and `SOUNDNESS_AUDIT.md`
  R-6a (the reasoning: unverifiable code holding a re-entry point into verified code destroys row
  soundness — audit F-6).
- Spec §5.3 (the Python allowlist honesty note — it gates the *interface*, not transitive imports).

---

## 1. Where the work lands + the dependency reality

- **`delulu-syntax`:** activate the `foreign` keyword; parse `foreign "c" lib M { … }` into
  `ForeignDecl`/`ForeignFn` (spec §2). Small, contained grammar addition.
- **`delulu-check`:** T-ForeignSig (the marshallability predicate `M(τ)`), the nominal opaque lib
  type `M`, the new prelude types (`ForeignPtr`, `ForeignErr`, `PyErr`, `PyObj`, `Cap[Python]`,
  `Cap[ForeignLoad]`, `ResourceKind::Python`/`ForeignLoad`), and `ForeignCall` as an ordinary effect.
- **`delulu-runtime`:** C FFI via `libloading` (bind) + `libffi` (call, because signatures are
  runtime data); embedded CPython via `PyO3` (`auto-initialize` off). **New crate dependencies** —
  this is the first stage that pulls in heavy native deps. Isolate them behind a `foreign` module and
  consider a cargo feature so a minimal build can exclude Python.
- **`delulu-wasm`:** the guest calls a `delulu:foreign@0.4` host interface (`bind`, `call(lib,
  sym-index, args) -> result<fval, ferr>`); **all FFI runs host-side** — the guest never touches a
  raw pointer. Parity with the interpreter is criterion 6.

**Cross-platform + native-dep warning (the biggest practical risk of this stage):** `libffi`, a C
toolchain, and a discoverable CPython must exist on Windows/macOS/Linux CI. Pin the Python version in
CI (criterion 6 requires identical results across engines — a floating Python breaks it). Budget real
time for the three-OS matrix; this is where Stage 4 actually gets hard, far more than the type rules.

---

## 2. Phase plan (each phase: build green → test green → update spec §"status" → commit)

Order chosen so the **type-level fence is provably complete before any native code is dlopen'd** —
the compile-time refusals (secrets, callbacks, unmarshallable types) are pure `delulu-check` work and
must be bulletproof first, because they are the actual security surface.

### Phase 4a — grammar + AST for `foreign` blocks (parse only)
Activate `foreign`; parse `foreign "c" lib M { fn cos(x: Float) -> Float }` into `ForeignDecl`. A
`foreign_fn` has **no effect-row syntax** — its row is implicitly `!{ForeignCall}`, always. ABI other
than `"c"` → DL1308.
*Test:* parse round-trip; a non-`"c"` ABI → DL1308.

### Phase 4b — the marshallability fence (T-ForeignSig) + the new prelude types
Register `ForeignPtr`/`ForeignErr`/`PyErr`/`PyObj`/`Cap[Python]`/`Cap[ForeignLoad]` and the nominal
opaque lib type. Implement `M(τ)`: only `Int Float Bool Str Unit ForeignPtr` marshal (DL1301 on
anything else, span on the offending type). **A function-typed param/return anywhere → DL1302**
(the no-callbacks rule, R-6a). **`Secret[T]`/`Cap`/`Root`/`Plugin`/opaque → DL1301, and the repair
list must NOT contain `expose`** (criterion 3 — never suggest laundering a secret across the FFI).
*Test (criterion 3):* a `Secret[Str]` param → DL1301 without `expose` in repairs; a `fn(Int)->Int`
param → DL1302 with the R-6a explanation.

### Phase 4c — `ForeignCall` effect + T-ForeignBind + T-ForeignCall (checking, no runtime)
Activate `ForeignCall` as an ordinary core effect (union, polymorphism, manifest ceilings,
`delulu why` — nothing special-cases it except reporting). `root.foreign[M](load: Cap[ForeignLoad])
-> Result[M, ForeignErr]` is **pure** (deriving a handle is not an effect; *using* it is). A method
call `m.cos(x)` types per the block signature with row exactly `{ForeignCall}`.
*Test:* a program calling `m.cos` without `ForeignCall` in its row → DL0501; `delulu why ForeignCall`
shows the chain.

### Phase 4d — C FFI runtime (interpreter engine)
`libloading` binds the lib named by the grant; `libffi` calls (signatures are runtime data).
Marshalling per spec §4.2 (`Int`→`int64_t`, `Float`→`double`, `Bool`→`int32_t`, `Str`→borrowed
`(ptr,len)`, `Str` return→copied-then-validated, `ForeignPtr`→opaque `void*`). **All symbols resolve
at bind time** (DL1304 fail-fast, never mid-run); an unbound logical name → DL1303 at the grant flow,
not mid-run. **Foreign return data is untrusted** (invariant 21): UTF-8 checked, length bounded by
`--foreign-max-ret` (default 64 MiB), validation failure → `ForeignErr::BadReturn` (a `Result` error,
**never UB, never a panic**).
*Test (criterion 1, 4, 8):* `libm.cos(1.0)` returns the right value with `ForeignCall` in the trace;
unbound lib → DL1303 at startup; missing symbol → DL1304 at bind; a C fn returning invalid UTF-8 →
`ForeignErr::BadReturn`, no crash.

### Phase 4e — manifest + grants + the grant prompt
`[authority] foreign.c = ["mathlib"]`; `--grant foreign.c=mathlib:/usr/lib/libm.so.6`. **The path is
grant data, not program data** — the program names *what* (`mathlib`, `cos`, `sqrt`); the human/broker
decides *which binary*. The grant prompt shows the full symbol list before asking. Extend `delulu
authority` with the `foreign_calls` array and the `-- outside the proof (contained at process level)
--` human-mode separator.
*Test (criterion 7):* `delulu authority` lists the foreign entry under the separator; a no-foreign
program has `"foreign_calls": []` and a report byte-identical to Stage 3.

### Phase 4f — embedded CPython (`std.py`)
`PyO3`, `auto-initialize` off; `root.python(load) -> Result[Cap[Python], ForeignErr]` initializes on
first grant-checked call (unavailable → DL1307 as a `ForeignErr`, not a crash). Import allowlist
patterns in the manifest (`foreign.python = ["numpy", "numpy.*"]`); `py.import("os")` under a
`["numpy"]` allowlist → **DL1305 as a runtime `PyErr`**, logged in the trace. The `std.py` surface
(§5.2) is all `!{ForeignCall}`. `PyObj` is opaque (no str/==/serialize; cannot appear in a `foreign
"c"` signature); `PyObj.call` with a DeluluLang closure arg → DL1302. Python exceptions →
`PyErr { kind, message }` (message is untrusted, length-bounded).
*Test (criterion 2, 5):* the NumPy demo (`import numpy`, build a list, call `mean`, convert back —
the "inherit the AI ecosystem" claim running); `py.import("os")` off-allowlist → DL1305.

### Phase 4g — the WASM engine `delulu:foreign@0.4` host interface + parity
Guest calls `bind`/`call(lib, sym-index, args: list<fval>) -> result<fval, ferr>` (`fval` = a
scalar/string variant); all of §4.1–4.2 runs host-side (the guest never touches a raw pointer — the
Stage-3 architecture paying off again). Recording the refusal in host state and surfacing after the
call (the carried Wasmtime-callback trap) applies to every foreign host fn.
*Test (criterion 6):* foreign conformance programs produce identical results *and traces* on both
engines (Python pinned to one version in CI).

---

## 3. The traps

1. **"Sandbox" is a banned word for Stage 4 foreign code.** The guarantee is reachability, not
   behavior. Foreign code runs with full process authority until Stage 5. Every DL13xx explain-text
   links *forward* to Stage 5 for actual containment. Copy spec §10 caveats verbatim.
2. **No callbacks — ever, by rule.** DL1302 on any function-typed value crossing the boundary, either
   direction. This is R-6a/audit-F-6, not a deferred feature. The escape valve is inverted control:
   DeluluLang drives the loop and passes *data*, not *code*. Do not add a "temporary" callback path.
3. **Never suggest `expose` to get a secret across the FFI.** DL1301's repair list must not contain
   it (criterion 3). Secrets crossing to foreign code unexposed is invariant 20; exposing them *to*
   foreign code is exactly the laundering the language exists to prevent.
4. **Symbols resolve at bind time, fail-fast.** DL1304 at bind, DL1303 at the grant flow — never
   mid-run. A program that binds successfully never surprises you with a missing symbol during a call.
5. **Foreign return data is untrusted input.** Validate shape (UTF-8, length ≤ `--foreign-max-ret`)
   at the boundary; a bad return is a `Result` error, never UB or a panic (invariant 21). Validate
   *shape*, not *meaning* — the data is still adversarial.
6. **The path is grant data, not program data.** The program never names a filesystem path to a
   `.so`/`.dll`; it names a logical lib. The human/broker maps logical→binary at grant time. This is
   what keeps "which binary runs" a human authority decision.
7. **The Python allowlist gates the interface, not transitive imports** (spec §5.3). Once `numpy` is
   imported, what numpy itself imports/does is unbounded at the OS level. State this in `delulu
   explain E-DL1305` verbatim. The real bounds are the reachability gate + Stage 5's worker/microVM.
8. **Native deps are the real difficulty.** libffi + a C toolchain + a pinned CPython across three
   OSes is where the schedule risk lives — not the type rules. Pin Python in CI; gate Python behind a
   cargo feature; budget the cross-platform matrix.

---

## 4. Definition of done (map to spec §9 acceptance criteria)

Ship when all 8 criteria pass — the load-bearing ones are criterion 1 (C call works on 3 OSes × both
engines with `ForeignCall` traced), criterion 2 (the NumPy demo — the adoption claim running),
criterion 3 (the compile-time fence: DL1302 callbacks, DL1301 secrets-without-`expose`), and
criterion 6 (engine parity). Add `## Implementation status` to `STAGE4_SPECIFICATION.md` per the
Stage-3 §8a pattern.

*Stage 4 opens the door and paints a bright line around it. Stage 5 moves the keys out of the house
and puts foreign code in a worker process.*
