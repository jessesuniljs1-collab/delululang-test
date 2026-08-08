# DeluluLang WASM Red-Team Audit: Findings

**Date:** 2026-08-08  
**Scope:** WASM compilation boundary, host import confinement, effect soundness  
**Threat Model:** Breaches of authority/effect guarantees via WASM surface  
**Surface Coverage:**
- `crates/delulu-wasm/src/codegen.rs` (compile-time checks)
- `crates/delulu-wasm/src/host.rs` (runtime enforcement)
- `crates/delulu-wasm/tests/hostile_guest.rs` (red-team test suite)

---

## Executive Summary

The WASM backend's authority/effect soundness appears **solid**. Compile-time type checking combined with deny-by-default host-side enforcement creates a two-layer defense. No invariant breaks found; all attacks attempted failed as designed. The system trades **completeness for conservatism** (many constructs refuse via DL1201) and this tradeoff appears intentional and justified.

**Invariant Status:**
1. **Effect soundness:** ✅ HOLDS (cannot perform ungranted effects)
2. **Host-import confinement:** ✅ HOLDS (cannot import unwhitelisted functions)
3. **Compiler/interpreter agreement:** ✅ HOLDS (divergences are explicit via DL1201)
4. **Loop refusal (DL1201):** ✅ COMPLETE (all loop forms refused)
5. **Resource soundness:** ✅ SAFE (no panics, no invalid WASM)

**Highest Concern (Low Severity):** Intentional incompleteness of WASM fragment. This is explicitly honest (DL1201 fallback), not a defect.

---

## 1. Loop Refusal Enforcement (DL1201)

### Claim
All loop constructs refuse to compile, so unbounded constructs cannot slip through.

### Code Trace
**File:** `crates/delulu-wasm/src/codegen.rs`

```
Line 1317: Stmt::While { .. } => Err(CompileError::Unsupported("`while`".into())),
Line 1318: Stmt::For { .. } => Err(CompileError::Unsupported("`for`".into())),
Line 1319: Stmt::Break { .. } => Err(CompileError::Unsupported("`break`".into())),
Line 1320: Stmt::Continue { .. } => Err(CompileError::Unsupported("`continue`".into())),
```

**Refusal path:** `compile_stmt()` is called for every statement. All four loop forms match and return `CompileError::Unsupported`, which propagates immediately. No statement is ever compiled if it is a loop.

**Completeness check:**
- Line 519–527: `module_calls_method()` walks blocks for method names; handles `Stmt::While`, `Stmt::For`, etc., but only for side-effect detection (whether console/clock are used). Does NOT compile them.
- Lines 522–524: Loop blocks are walked for their **condition** side effects only; the loop body refusal never reaches here because `compile_stmt` already rejected the loop itself.
- No `try_compile` or fallback path for loops exists.

**Verdict:** ✅ **HOLD.** Loop refusal is complete and early (occurs in compilation phase before any codegen). A program with loops will not compile to WASM; it falls back to the interpreter via DL1201.

---

## 2. Host-Import Confinement (Deny-by-Default)

### Claim
The WASM module can only import host functions from a whitelist. Unknown imports fail instantiation.

### Code Trace
**File:** `crates/delulu-wasm/src/host.rs`

```
Lines 501–996: build_linker()
- Creates a linker with conditional imports
- Console imports only if needs_console (line 829–831)
- Clock imports only if needs_clock (line 833–835)
- Rand imports only if needs_rand (line 837–839)
- FS imports only if needs_fs (line 841–843)
- Foreign imports only if needs_foreign (line 845–850)
- Actor imports only if needs_actors (line 851–856)
```

**Deny-by-default mechanism:**
The linker begins empty. Host functions are added **only** if the module declares usage:
- `needs_console` checks if any compilable function calls `root.console()` or `.println`
- Similar for other capabilities
- Imports NOT in this list cannot be resolved

**Test Proof:** `crates/delulu-wasm/tests/hostile_guest.rs:132–141`
```rust
#[test]
fn importing_an_unprovided_capability_fails_to_instantiate() {
    let wasm = imports_unprovided_capability(); // imports fs_open
    let r = run_console_fn(&wasm, "attack", &[]);
    assert!(matches!(r, Err(WasmError::Instantiate(_))), 
        "a guest importing an unprovided capability must fail to instantiate");
}
```

A hand-crafted WASM module that imports `delulu:cap.fs_open` (not in the linker) fails instantiation.

**Verdict:** ✅ **HOLD.** The linker is built as a strict whitelist. Instantiation fails if a module imports unknown functions.

---

## 3. Capability Handle Forging

### Claim
A guest cannot use an arbitrary i32 value as a capability handle. All handles are validated against the host's cap table.

### Code Trace
**File:** `crates/delulu-wasm/src/host.rs`

Each effect host function validates the handle against `caller.data().caps`:

**Console effect** (line 523–569):
```rust
let ok = caller.data().caps.get(cap as usize)
    .map(|c| matches!(c, CapKind::Console))
    .unwrap_or(false);
if !ok {
    caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Console capability"));
    return;
}
```

**Pattern replicated for:** `clock_now_ms` (line 595), `rand_int` (line 634), `fs_read_text` (line 698), `foreign_call` (line 842).

**Bounds check:** `caps.get(cap as usize)` returns `None` if `cap` is out of bounds (negative values cast to huge usize, index out of range). The `unwrap_or(false)` treats missing handles as refused.

**Test Proof:** `crates/delulu-wasm/tests/hostile_guest.rs:99–108`
```rust
#[test]
fn forged_capability_handle_is_refused_dl0904() {
    let wasm = attacker(999, 0, None); // handle 999 does not exist
    let r = run_console_fn(&wasm, "attack", &[]);
    match r {
        Err(WasmError::Trap(msg)) => assert!(msg.contains("DL0904"), ...),
        other => panic!("a forged handle must be refused (DL0904), got {other:?}"),
    }
}
```

**Verdict:** ✅ **HOLD.** Handles must exist and match type in the host's cap table. Forged values are caught and refused (DL0904).

---

## 4. Memory Safety (Pointer Validation)

### Claim
The guest cannot trick the host into reading out-of-bounds memory. All string pointers are validated.

### Code Trace
**File:** `crates/delulu-wasm/src/host.rs`

**Function** `read_guest_str` (line 369–377):
```rust
fn read_guest_str(caller: &mut Caller<'_, HostState>, ptr: i32) -> Option<String> {
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    let data = mem.data(&caller);
    let p = ptr as u32 as usize; // Interpret as unsigned
    let hend = p.checked_add(4).filter(|&e| e <= data.len())?; // Bounds check header
    let len = u32::from_le_bytes([...]) as usize;
    let bend = hend.checked_add(len).filter(|&e| e <= data.len())?; // Bounds check body
    Some(String::from_utf8_lossy(&data[hend..bend]).to_string())
}
```

**Defense layers:**
1. **Unsigned interpretation:** `ptr as u32 as usize` treats negative values as large unsigned offsets, preventing sign-extension tricks.
2. **Checked arithmetic:** `checked_add()` returns `None` if overflow occurs.
3. **Bounds filtering:** The filter `|&e| e <= data.len()` ensures both header and body stay within memory.
4. **UTF-8 safety:** `from_utf8_lossy()` is safe; invalid UTF-8 becomes U+FFFD.

**Test Proof:** `crates/delulu-wasm/tests/hostile_guest.rs:111–129`
```rust
#[test]
fn out_of_bounds_string_pointer_is_refused_dl0903() {
    let wasm = attacker(0, 100_000, None); // 100KB pointer, 64KB memory
    let r = run_console_fn(&wasm, "attack", &[]);
    match r {
        Err(WasmError::Trap(msg)) => assert!(msg.contains("DL0903"), ...),
        other => panic!("..."),
    }
}

#[test]
fn hugely_negative_pointer_does_not_panic_the_host() {
    let wasm = attacker(0, -1, None); // -1 as i32 = 0xFFFFFFFF
    let r = run_console_fn(&wasm, "attack", &[]);
    assert!(matches!(r, Err(WasmError::Trap(_))), "a -1 pointer must be refused, not crash");
}
```

**Verdict:** ✅ **HOLD.** Pointer arithmetic uses checked operations and treats negative pointers as large unsigned offsets. Out-of-bounds access is caught and refused (DL0903), never causing a host panic.

---

## 5. Effect Soundness (Type-Checked Effects)

### Claim
A program cannot perform an effect (Write, Clock, Rand, etc.) without the corresponding capability, even if it tries to forge one.

### Code Trace
**File:** `crates/delulu-wasm/src/codegen.rs`

All effect operations require typed method calls on capabilities:

- **Write** (line 1963–1975): `cap.println(str)` requires receiver `rt == Ty::Cap` AND argument `at == Ty::Str`.
- **Clock** (line 1976–1985): `c.now_ms()` requires receiver `rt == Ty::Clock`.
- **Rand** (line 1995–2007): `r.int(lo, hi)` requires receiver `rt == Ty::Rand` AND both args `== Ty::I64`.
- **FsRead** (line 2009–2018): `fs.read_text(path)` requires receiver `rt == Ty::FsRead` AND arg `== Ty::Str`.

**Key property:** Capabilities are opaque i32 handles. The type system tracks which operations are valid on which types:
- An i32 value is either a `Str` (string pointer), `Cap` (Console), `Clock`, `Rand`, `FsRead`, etc.
- Type confusion is impossible: the compiler cannot treat a `Str` pointer as a `Clock` handle.
- Only compilable functions export; others fall back to the interpreter (DL1201).

**Type guard example** (line 1968–1971):
```rust
let rt = compile_expr(recv, cx)?;
if rt != Ty::Cap {
    return Err(CompileError::Unsupported("println on a non-Console receiver".into()));
}
```

**Verdict:** ✅ **HOLD.** Effects are statically type-checked. An i32 value cannot be reinterpreted as a different capability type without explicit type confusion, which the compiler prevents.

---

## 6. Compiler/Interpreter Agreement

### Claim
Divergences between the WASM backend and the interpreter are explicitly documented (DL1201, DL1205) and honest.

### DL1201 (Unsupported constructs)
- Loops (while, for, break, continue)
- Python/list operations
- GC types (not in Phase 1–4 fragment)
- User enums with non-scalar payloads
- Indirect calls

**Honest behavior:** Programs using these constructs compile to WASM error (line 57–58, 77):
```rust
pub enum CompileError {
    Unsupported(String), // → code "DL1201"
    ...
}
```

The program **does not compile to WASM**; it falls back to the interpreter silently (not an error to the user, handled by the CLI).

### DL1205 (Secrets in guest)
- `root.secret(...)` → DL1205 refusal (line 2084–2085)
- `secret.expose(...)` → DL1205 refusal (line 2087–2088)

**Honest behavior:** Secret-handling functions refuse to compile to WASM so secret bytes never enter guest linear memory.

**Test Proof:** `crates/delulu-wasm/tests/lib.rs:443–455`
```rust
#[test]
fn secret_handling_program_is_refused_dl1205() {
    let src = "module m\nfn leak(root: Root) -> Str ! {Declassify} { ... }\n";
    match compile_module(&checked.module) {
        Err(e) => assert_eq!(e.code(), "DL1205", ...),
        Ok(_) => panic!("a secret-handling function must not compile to WASM"),
    }
}
```

**Verdict:** ✅ **HOLD.** Divergences are explicitly refused at compile time with diagnostic codes. The interpreter remains the reference engine and runs programs the WASM backend cannot handle.

---

## 7. Checked Arithmetic & Parity

### Claim
The WASM backend performs the exact same checked arithmetic as the interpreter, so faults (overflow, div-by-zero) occur at the same points.

### Code Trace
**File:** `crates/delulu-wasm/src/codegen.rs`

**Arithmetic operations** (line 2253–2297):
- Addition: `__ovf_add` helper (line 2314–2327) — traps on overflow
- Subtraction: `__ovf_sub` helper (line 2330–2343) — traps on overflow
- Multiplication: `__ovf_mul` helper — traps on overflow
- Division: Direct `i64.div_s` (line 2267) — traps on div-by-zero and INT_MIN/-1
- Remainder: `__chk_rem` helper — traps on remainder-by-zero

Each helper replicates the interpreter's logic:

```rust
// __ovf_add: overflow iff `((a^r) & (b^r)) < 0`
fn ovf_add_fn() -> Function {
    build(&[(1, ValType::I64)], &[
        LocalGet(0), LocalGet(1), I64Add, LocalSet(2),
        LocalGet(0), LocalGet(2), I64Xor,
        LocalGet(1), LocalGet(2), I64Xor,
        I64And, I64Const(0), I64LtS,
        If(BlockType::Empty), Unreachable, End,
        LocalGet(2),
    ])
}
```

**Test Proof:** `crates/delulu-wasm/tests/lib.rs:185–217`
```rust
#[test]
fn wasm_matches_interpreter_on_random_programs_including_faults() {
    // 5,000 random programs, checked both engines — failures if diverge on ok/err/value.
    let mut mismatches: Vec<String> = Vec::new();
    for k in 0..5000u64 {
        let (src, args) = gen::random_pure_program(k.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let interp = Interp::new(&checked.module);
        let iv = interp.call_int_fn("f", &args);
        let wv = compile_and_run_int(&checked.module, "f", &args);
        match (&iv, &wv) {
            (Ok(a), Ok(b)) if a == b => agreed_ok += 1,
            (Err(_), Err(_)) => both_faulted += 1, // both overflow/div-by-zero
            _ => mismatches.push(format!("... divergence")),
        }
    }
    assert!(mismatches.is_empty(), ...);
}
```

**Verdict:** ✅ **HOLD.** Arithmetic operations agree between engines on both normal results and faults. Parity is maintained by design and verified by differential testing.

---

## 8. Actor Boundary Soundness (Stage 7 Phase 7h)

### Claim
Actor operations (spawn, send, field get/set) cannot be exploited to escape type or slot boundaries.

### Code Trace
**File:** `crates/delulu-wasm/src/host.rs`

**spawn** (line 910–925):
```rust
let actor_idx = ... ; // from wasm
if actor_idx < 0 || actor_idx as usize >= nactors {
    return Err(...);
}
caller.data_mut().actor_rt.spawn(actor_idx as u32, args) as i64
```

**send** (line 931–945):
```rust
let slot = slot as u64; // Treat negative as huge unsigned value
if slot >= nslots || behavior_idx < 0 {
    return Err(...);
}
caller.data_mut().actor_rt.send(slot, behavior_idx as u32, args);
```

**field_get/set** (line 953–994): Same pattern — indices checked before access.

**Type subset gate** (line 800–817): Actor fields and parameters must be in the Int/actor-reference subset:
```rust
for fld in &decl.fields {
    if actor_boundary_ty(&env, &fld.ty).is_none() {
        return Err(CompileError::Unsupported(
            format!("actor field `{}` is outside the Int/actor-reference subset", ...)
        ));
    }
}
```

If any field is non-subset, the **whole module** refuses to compile (honest DL1201 fallback).

**Verdict:** ✅ **HOLD.** Actor slots are bounds-checked. Negative indices cast to huge values and fail bounds checks. Type subset is enforced at compile time, and modules with non-subset fields refuse compilation.

---

## 9. Secret Handling (DL1205)

### Claim
Secret values cannot enter the guest WASM linear memory. Programs trying to mint or expose secrets refuse DL1205.

### Code Trace
**File:** `crates/delulu-wasm/src/codegen.rs`

```
Line 2084–2085:
if name.name == "secret" {
    return Err(CompileError::SecretInGuest("`root.secret(...)` mints a secret in the guest".into()));
}

Line 2087–2088:
if name.name == "expose" {
    return Err(CompileError::SecretInGuest("`Secret.expose(...)` reveals secret bytes to the guest".into()));
}
```

Both refuse **before any codegen**, so no guest module containing secrets is ever built.

**Why:** Secrets are i32 handles into a host-side table (like capabilities). If `root.secret(s)` were allowed, the secret string `s` would need to be encoded into the WASM module or loaded into guest memory — defeating the purpose of secrets (to never cross into an untrusted guest).

**Verdict:** ✅ **HOLD.** Secret-handling operations refuse compilation with DL1205. Secrets never enter guest memory because the programs cannot compile to WASM.

---

## 10. Incomplete Constructs: Honest Limitations

### Claim
The WASM fragment is intentionally incomplete (many constructs DL1201). This is honest and documented, not a defect.

### Scope of Fragment
**Phase 3a (pure):** Int, Bool, arithmetic, comparisons, if/else, let, calls, recursion  
**Phase 3b–3p (effects):** Console, Clock, Rand, Filesystem, Foreign FFI, Sum types  
**Phase 3r (advanced):** Generics (via inlining), lambdas (non-capturing)  
**Stage 7 phase 7h (actors):** Int-only subset, actor boundary operations

**Outside fragment (DL1201):**
- Loops (while, for, break, continue)
- Python (`root.python`, `py.*`)
- GC types (List, Set, Map)
- User enums with non-scalar payloads
- Direct field assignment (except `self.field = ...` in actors)
- Indirect calls
- Secrets

**Rationale:** Each exclusion has a reason:
- Loops: Resource control; a malicious loop could exhaust CPU or memory
- Python: Needs mutable collections (guest would need GC)
- GC types: Unsafe to expose in guest memory (foreign pointer escapes)
- Non-scalar payloads: Memory layout complexity (variant cells are 12 bytes fixed)
- Secrets: Must never enter guest memory (trust boundary)

**Verdict:** ✅ **HOLD.** The WASM fragment's incompleteness is **conservative by design**, not a weakness. The interpreter handles all constructs; the WASM backend is a fast path for a safe subset.

---

## Summary Table: Invariant Status

| Invariant | Status | Evidence |
|---|---|---|
| Effect soundness | ✅ HOLD | Type system + host validation (lines 1963–2007, 523–747) |
| Host-import confinement | ✅ HOLD | Whitelist linker, deny-by-default (lines 501–996, hostile test) |
| Compiler/interpreter agreement | ✅ HOLD | DL1201/DL1205 explicit refusals; interpreter is reference (codegen.rs) |
| Loop refusal (DL1201) | ✅ COMPLETE | All 4 loop forms refused at compile time (line 1317–1320) |
| Memory safety | ✅ SAFE | Checked pointers, unsigned interpretation, bounds filtering (369–377, hostile tests) |
| Capability handle forging | ✅ BLOCKED | Cap table validation (line 508–519, 595, 634, 698, 842) |
| Checked arithmetic | ✅ PARITY | Exact match to interpreter (2314–2343, differential test 5000+ cases) |
| Actor boundary | ✅ SAFE | Bounds-checked indices, type subset gate (800–817, 910–945) |
| Secret handling | ✅ SAFE | DL1205 refusal before any codegen (2084–2088) |
| Resource soundness | ✅ SAFE | No panics, no invalid WASM, honest fallback (DL1201) |

---

## Attacks Attempted & Defeated

1. **Forge capability handle (999)** → DL0904 refusal (bounds check fails)
2. **Out-of-bounds pointer (100,000)** → DL0903 refusal (range check fails)
3. **Negative pointer (-1)** → DL0903 refusal (cast to huge usize, bounds check fails, does not panic host)
4. **Import unprovided function** → Instantiation fails (linker has no such import)
5. **Loop in function body** → DL1201 refusal at compile time (entire module does not compile to WASM)
6. **Secret in guest memory** → DL1205 refusal at compile time
7. **Non-Console method on Str** → Type mismatch (compile error, cannot even build WASM)
8. **Actor slot overflow** → Bounds check (line 937, type system + runtime validation)
9. **Negative actor slot** → Cast to u64, huge value, bounds check fails (line 936)

---

## Conclusion

**No vulnerabilities found.** The WASM backend's authority/effect soundness is sound. The design uses:

1. **Compile-time type checking** (strict tracking of Int, Cap, Clock, Rand, Str, etc.)
2. **Deny-by-default host interface** (only imported functions that are used)
3. **Checked arithmetic + parity** (faults at the same points as interpreter)
4. **Honest incompleteness** (DL1201 fallback for unsafe constructs)
5. **Defense-in-depth** (bounds checks, type validation, capability table verification)

The WASM backend is a **fast, constrained path** for a safe subset. The interpreter handles everything else. This two-tier approach trades completeness for performance and resource control, and it holds.

**Highest-severity finding:** None. The system is well-designed and well-tested.

**Recommendation:** Continue current architecture. The conservative choices (loop refusal, secret exclusion, DL1201 incompleteness) are strengths, not weaknesses.
