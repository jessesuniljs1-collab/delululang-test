//! The embedded Wasmtime host (Phase 3a/3b).
//!
//! Pure functions (Phase 3a) instantiate with an EMPTY import set — zero ambient authority.
//! Effectful functions (Phase 3b) get exactly the `delulu:cap` host functions their grant allows;
//! the host performs the effect and its scope check, and reads string bytes out of the guest's
//! exported linear memory. The guest never receives an OS handle — capabilities are opaque i32
//! handles into the host's cap table (§4).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use wasmtime::{Caller, Config, Engine, Instance, Linker, Module, OptLevel, Store, Val};

/// The optimizing execution engine — Track A §2.1's "Cranelift-optimized Wasmtime tier".
///
/// wasmtime compiles every module with Cranelift, its optimizing backend; this pins the
/// optimization level to `Speed` **explicitly** rather than inheriting wasmtime's default, so the
/// tier a `.dwx` runs under is a documented, deliberate choice that cannot silently change if a
/// future wasmtime default does. On the pinned wasmtime 27 (whose default already IS `Speed`) this
/// is behaviorally identical to `Engine::default()`, so Stage 3's two-engine differential is
/// unaffected — the point is intent and drift-resistance, not a behavior change.
///
/// The **DIR-level optimizer** §2.1 *also* describes (cross-package inlining, monomorphization,
/// escape analysis) is NOT part of this tier; it is deferred honestly for 1.x — see build-order
/// D18 and `measurements/study-c/HOT_PATH_TABLE.md` (criterion 1). This engine is the whole of the
/// "optimizing backend" that ships in 1.x.
pub(crate) fn optimizing_engine() -> Engine {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::Speed);
    // `Speed` is an always-valid level, so construction cannot fail on wasmtime 27; fall back to
    // the default engine rather than panic if a future wasmtime ever disagrees.
    Engine::new(&config).unwrap_or_default()
}

// Stage 7 phase 7h: the WASM engine's cooperative single-threaded actor scheduler (spec §6.5).
use crate::actors::{ActorJob, ActorReport, ActorRuntime, ActorTable};

// Stage 4 phase 4g: the WASM host reuses the interpreter's C FFI machinery and its trace types, so
// verify≡run stays one code path (spec §4.3 — all of §4.1–4.2 runs host-side).
use delulu_runtime::foreign::{self, FVal, ForeignErr, ForeignHandle, ForeignSig};
// Stage 5 phase 5f: the WASM host calls the SAME `Custody` seam the interpreter does (playbook §1 —
// "the interpreter and the WASM host both call the trait"). `None` = embedded (Stage 1–4 behavior).
use delulu_runtime::{Custody, CustodyDecision, Op as CustodyOp};
use delulu_runtime::{TraceRecord, TraceSink};

/// The custody handle shared between `HostConfig` and `HostState`: interior-mutable because custody
/// `check` refreshes the epoch cache, `Rc` because the config is `Clone`.
pub type CustodyHandle = Rc<RefCell<Box<dyn Custody>>>;

/// Lexically normalize a path (resolve `.`/`..` without touching the filesystem) — the EXACT
/// algorithm the interpreter uses (`prim.rs::normalize`), so scope checks agree across engines.
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[derive(Clone, Debug)]
pub enum WasmError {
    Module(String),
    Instantiate(String),
    NoExport(String),
    Trap(String),
    BadResult,
}

impl WasmError {
    pub fn message(&self) -> String {
        match self {
            WasmError::Module(e) => format!("invalid WASM module: {e}"),
            WasmError::Instantiate(e) => format!("instantiation failed: {e}"),
            WasmError::NoExport(n) => format!("no exported function `{n}`"),
            WasmError::Trap(e) => format!("WASM trap: {e}"),
            WasmError::BadResult => "unexpected result type from WASM".to_string(),
        }
    }
}

/// Turn a wasmtime call error into a clean, single-line `Trap` detail — the interpreter's precise
/// diagnostic code where the trap is deterministic, and **never** the guest backtrace.
///
/// Two problems this fixes (HARDENING_CAMPAIGN C20). First, `e.to_string()` on a wasmtime trap
/// appends the FULL guest backtrace; for a deep-recursion `StackOverflow` that is one line per
/// frame — tens of thousands of `<wasm function N>` lines that flood a terminal and drown an
/// agent's output, where the interpreter prints a single `DL0905`. Second, the raw string discards
/// the structured trap, so the interpreter's codes (DL0901 overflow, DL0902 divide-by-zero, DL0903
/// out-of-bounds, DL0905 recursion) all collapsed to a generic DL0904 — breaking engine parity
/// (invariant 15) for exactly the deterministic faults where parity is cheapest to keep. The code
/// is embedded in the detail so the CLI's exit-code mapper reads it; the backtrace never is.
pub(crate) fn clean_trap(e: &anyhow::Error) -> String {
    use wasmtime::Trap;
    let first_line = || e.to_string().lines().next().unwrap_or("wasm trap").to_string();
    match e.downcast_ref::<Trap>() {
        Some(t) => match t {
            Trap::IntegerDivisionByZero => "DL0902: division by zero".to_string(),
            // `i64.div_s` of INT_MIN/-1 traps as IntegerOverflow, which the interpreter reports as
            // DL0901 (checked_div returns None) — so it maps to overflow, not divide-by-zero.
            Trap::IntegerOverflow => "DL0901: integer overflow".to_string(),
            Trap::StackOverflow => "DL0905: recursion depth exceeded (guest stack exhausted)".to_string(),
            Trap::MemoryOutOfBounds | Trap::HeapMisaligned | Trap::TableOutOfBounds => {
                "DL0903: memory access out of bounds".to_string()
            }
            // The codegen traps checked `+`,`-`,`*` overflow and `%` by zero / INT_MIN%-1 via
            // `unreachable`. The interpreter reports overflow as DL0901 and rem-by-zero as DL0902;
            // the two are indistinguishable from the trap alone, so the common case (overflow)
            // wins and the residual is documented in C20.
            Trap::UnreachableCodeReached => "DL0901: integer overflow".to_string(),
            _ => first_line(),
        },
        None => first_line(),
    }
}

/// Run an exported PURE function with i64 arguments (no imports, no ambient authority).
pub fn run_int_fn(wasm: &[u8], name: &str, args: &[i64]) -> Result<i64, WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, name).ok_or_else(|| WasmError::NoExport(name.to_string()))?;
    let params: Vec<Val> = args.iter().map(|&a| Val::I64(a)).collect();
    let mut results = vec![Val::I64(0)];
    func.call(&mut store, &params, &mut results).map_err(|e| WasmError::Trap(clean_trap(&e)))?;
    match results.first() {
        Some(Val::I64(n)) => Ok(*n),
        Some(Val::I32(n)) => Ok(*n as i64),
        _ => Err(WasmError::BadResult),
    }
}

/// The host's capability table. A handle is an index into `caps`; index 0 is conventionally the
/// root (for `main`) or a directly-granted Console (for a bare cap function).
#[derive(Clone)]
pub enum CapKind {
    Root,
    Console,
    Clock,
    Rand,
    /// A filesystem-read capability scoped to an absolute, normalized subtree (§4).
    FsRead(PathBuf),
    /// `Cap[ForeignLoad]` — the loader capability minted by `root.foreign_load()` (Stage 4 phase 4g).
    ForeignLoad,
    /// A bound `foreign` lib handle (Stage 4 phase 4g): the loaded library + resolved symbols, held
    /// in the same host-side table as every other capability. `Rc` because `LoadedLib` owns a
    /// `Library` and is not `Clone`.
    Foreign(Rc<ForeignHandle>),
}

struct HostState {
    caps: Vec<CapKind>,
    /// Whether the human/broker granted console output (the run-time authority grant).
    console_granted: bool,
    /// Whether the clock capability was granted.
    clock_granted: bool,
    /// Whether the rand capability was granted.
    rand_granted: bool,
    /// Granted filesystem-read subtrees (absolute, normalized). `root.fs_read(p)` succeeds iff `p`
    /// resolves within one of these.
    fs_read_roots: Vec<PathBuf>,
    /// `Some(ms)` fixes `Cap[Clock].now_ms()` for deterministic replay (spec §6.2); `None` = wall clock.
    fixed_clock_ms: Option<i64>,
    /// The xorshift64 PRNG state for `Cap[Rand]` — seeded identically to the interpreter's so seeded
    /// `rand.int` sequences match byte-for-byte (spec §6.2, two-engine parity).
    rng: u64,
    output: String,
    /// Set host-side when a capability check fails. We record it and return without trapping (an
    /// error returned across the wasm frame aborts on some platforms); the runner turns a set flag
    /// into a clean `WasmError` after the call.
    refused: Option<String>,
    // ----- Stage 4 phase 4g: C FFI + effect tracing --------------------------------------------
    /// Whether `root.foreign_load()` may mint a `Cap[ForeignLoad]` (i.e. some `foreign.c` lib or
    /// `foreign.python` pattern is granted — mirrors the interpreter's `RootVal.foreign_load`).
    foreign_load_granted: bool,
    /// `foreign.c` grants: logical lib name → the binary path the human chose. The path is grant
    /// data, never program data (spec §4.1).
    foreign_grants: HashMap<String, String>,
    /// Each `foreign` lib's declared marshalling signatures (lowered from the block), used at bind
    /// time to resolve every symbol fail-fast (spec §4.2 / DL1304) and to drive `foreign::call`.
    foreign_sigs: HashMap<String, Vec<ForeignSig>>,
    /// Ceiling on a returned foreign string (`--foreign-max-ret`, invariant 21). 0 = the default.
    foreign_max_ret: usize,
    /// Opaque `void*` values returned by foreign code — the guest holds an i32 index into this table,
    /// never a raw host address (the wasm ABI is 32-bit; a real pointer can't cross safely).
    foreign_ptrs: Vec<usize>,
    /// `Some` records one `TraceRecord` per effectful op host-side, so `--engine wasm --trace-effects`
    /// yields a trace byte-identical to the interpreter's (criterion 6). Shared `Rc`; the runner
    /// reads it after the run.
    trace: Option<TraceSink>,
    /// Monotonic trace sequence counter (matches the interpreter's `next_trace_seq`).
    trace_seq: u64,
    /// Stage 5 phase 5f: where authority lives. `None` = embedded (the in-process checks above stay
    /// the enforcement, zero behavior change — criterion 11); `Some` routes every gated op through
    /// the custody seam (daemon mode: broker round-trips / epoch snapshot).
    custody: Option<CustodyHandle>,
    /// Stage 7 phase 7h: the cooperative actor scheduler's host-side state (per-actor field slots,
    /// the FIFO job queue, quiescence counters). Empty for a program with no actors.
    actor_rt: ActorRuntime,
}

impl HostState {
    /// The effective foreign-return ceiling (0 in config means "use the default").
    fn max_ret(&self) -> usize {
        if self.foreign_max_ret == 0 { foreign::DEFAULT_MAX_RET } else { self.foreign_max_ret }
    }

    /// Consult custody for one gated op (Stage 5 phase 5f). On `Deny` the refusal is RECORDED in
    /// `refused` and `false` returned — **never an `Err` across the wasm frame** (playbook trap 5,
    /// carried from Stage 3/4): the runner surfaces the refusal cleanly after the call, exactly
    /// like the Stage-3 console/fs refusals. `None` custody (embedded) always allows.
    fn custody_allows(&mut self, op: CustodyOp, arg: Option<&str>) -> bool {
        let Some(custody) = &self.custody else { return true };
        match custody.borrow_mut().check(op, arg) {
            CustodyDecision::Allow => true,
            CustodyDecision::Deny(d) => {
                self.refused = Some(format!("{}: {}", d.code, d.message));
                false
            }
        }
    }

    /// Append one effect `TraceRecord` (no-op without a sink). `seq`/fields mirror the interpreter's
    /// so the two engines' traces are identical.
    fn push_trace(&mut self, effect: &str, op: &str, cap_kind: &str, detail: Option<String>, file: i32, start: i32, end: i32) {
        let Some(sink) = &self.trace else { return };
        let seq = self.trace_seq;
        self.trace_seq += 1;
        sink.push(TraceRecord {
            seq,
            effect: effect.to_string(),
            op: op.to_string(),
            cap_kind: cap_kind.to_string(),
            detail,
            span: Some((file as u32, start as u32, end as u32)),
            // Stage 7: actor attribution is stamped by the actor scheduler; the wasm host's
            // main-line records carry none (byte-identical trace output preserved).
            ..Default::default()
        });
    }
}

/// Format an `f64` EXACTLY as `Value::display` does (`{:.1}` for a finite integral value, else
/// `to_string`) — the guest's `str(Float)` calls this host-side, so both engines print identically.
fn format_float(x: f64) -> String {
    if x.fract() == 0.0 && x.is_finite() {
        format!("{x:.1}")
    } else {
        x.to_string()
    }
}

/// Encode a validated foreign return `FVal` into the single i64 `foreign_call` yields (the guest
/// re-derives the declared type: Float via `f64.reinterpret_i64`, Bool/Str/ForeignPtr via `i32.wrap`).
/// A `Str` return is copied into a guest string cell; a `ForeignPtr` is stashed in the host table and
/// its index returned (the guest never sees a raw address).
fn encode_fval_return(caller: &mut Caller<'_, HostState>, fv: FVal) -> i64 {
    match fv {
        FVal::Int(i) => i,
        FVal::Bool(b) => b as i64,
        FVal::Float(f) => f.to_bits() as i64,
        FVal::Unit => 0,
        FVal::Str(s) => match write_str_cell(caller, &s) {
            Ok(ptr) => ptr as u32 as i64,
            Err(()) => {
                caller.data_mut().refused = Some("guest heap exhausted copying a foreign string return".into());
                0
            }
        },
        FVal::Ptr(p) => {
            let st = caller.data_mut();
            st.foreign_ptrs.push(p);
            (st.foreign_ptrs.len() - 1) as i64
        }
    }
}

/// Wall-clock milliseconds since the Unix epoch (matches the interpreter's `Cap[Clock].now_ms()`).
fn wall_clock_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// One xorshift64 step — the EXACT generator the interpreter uses (`prim.rs::next_rand`), so a run
/// under the same seed produces an identical `rand.int` sequence on both engines.
fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// The initial PRNG state, matching the interpreter: `set_rand_seed`'s `(s==0 ? GOLDEN : s) | 1` for a
/// fixed seed, or a nanosecond wall seed (`| 1`) when no seed is given.
fn seed_rng(seed: Option<u64>) -> u64 {
    match seed {
        Some(s) => (if s == 0 { 0x9E37_79B9 } else { s }) | 1,
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9)
            | 1,
    }
}

// ----- guest-memory helpers (the host reads/writes the guest's linear memory) ----------------
//
// For the filesystem cap the host must *write* structured data into guest memory (the file content
// string, and the `Result[Str, IoErr]` cells that wrap it) and bump the guest's bump-heap pointer —
// the exported `__heap` global. The cell layout MUST match `codegen.rs` exactly:
//   Str:    `[len:u32-le][bytes]`
//   variant `[tag:i32-le @0][field @4]` (8-byte field slot; cell 12 bytes)
//   Result tags: Ok=0, Err=1.   IoErr tags: NotFound=0, Denied=1, Other=2.

/// Read a length-prefixed guest string at `ptr`; `None` if it is out of bounds.
fn read_guest_str(caller: &mut Caller<'_, HostState>, ptr: i32) -> Option<String> {
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    let data = mem.data(&caller);
    let p = ptr as u32 as usize;
    let hend = p.checked_add(4).filter(|&e| e <= data.len())?;
    let len = u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
    let bend = hend.checked_add(len).filter(|&e| e <= data.len())?;
    Some(String::from_utf8_lossy(&data[hend..bend]).to_string())
}

/// Bump-allocate `n` bytes in the guest heap (via the `__heap` global); returns the old pointer.
fn guest_alloc(caller: &mut Caller<'_, HostState>, n: i32) -> Result<i32, ()> {
    let g = caller.get_export("__heap").and_then(|e| e.into_global()).ok_or(())?;
    let base = match g.get(&mut *caller) {
        Val::I32(v) => v,
        _ => return Err(()),
    };
    g.set(&mut *caller, Val::I32(base + n)).map_err(|_| ())?;
    Ok(base)
}

fn guest_write(caller: &mut Caller<'_, HostState>, ptr: i32, bytes: &[u8]) -> Result<(), ()> {
    let mem = caller.get_export("memory").and_then(|e| e.into_memory()).ok_or(())?;
    mem.write(&mut *caller, ptr as u32 as usize, bytes).map_err(|_| ())
}

/// Write a `[len][bytes]` string cell; returns its pointer.
fn write_str_cell(caller: &mut Caller<'_, HostState>, s: &str) -> Result<i32, ()> {
    let ptr = guest_alloc(caller, 4 + s.len() as i32)?;
    guest_write(caller, ptr, &(s.len() as u32).to_le_bytes())?;
    guest_write(caller, ptr + 4, s.as_bytes())?;
    Ok(ptr)
}

/// Write a `[tag][field]` variant cell (12 bytes); `field` is an i32 payload pointer, if any.
fn write_variant_cell(caller: &mut Caller<'_, HostState>, tag: i32, field: Option<i32>) -> Result<i32, ()> {
    let ptr = guest_alloc(caller, 12)?;
    guest_write(caller, ptr, &tag.to_le_bytes())?;
    if let Some(f) = field {
        guest_write(caller, ptr + 4, &f.to_le_bytes())?;
    }
    Ok(ptr)
}

/// Build an `Ok(content)` : `Result[Str, IoErr]` in guest memory; returns its pointer.
fn build_ok_str(caller: &mut Caller<'_, HostState>, content: &str) -> Result<i32, ()> {
    let s = write_str_cell(caller, content)?;
    write_variant_cell(caller, 0, Some(s)) // Ok
}

/// Build an `Err(<IoErr>)` : `Result[Str, IoErr]` in guest memory (`io_tag`: NotFound=0/Denied=1/
/// Other=2; `other_msg` is the `Other(Str)` payload).
fn build_err_ioerr(caller: &mut Caller<'_, HostState>, io_tag: i32, other_msg: Option<&str>) -> Result<i32, ()> {
    let field = match other_msg {
        Some(m) => Some(write_str_cell(caller, m)?),
        None => None,
    };
    let io = write_variant_cell(caller, io_tag, field)?;
    write_variant_cell(caller, 1, Some(io)) // Err(io)
}

/// Build a `ForeignErr` cell (Stage 4 phase 4g). The tag order MUST match the codegen enum env AND
/// the interpreter's sum: `NotGranted=0 | SymbolMissing=1 | BadReturn=2 | Unavailable=3` (spec §8).
fn build_foreign_err_cell(caller: &mut Caller<'_, HostState>, e: &ForeignErr) -> Result<i32, ()> {
    let (tag, msg): (i32, Option<&str>) = match e {
        ForeignErr::NotGranted => (0, None),
        ForeignErr::SymbolMissing(s) => (1, Some(s.as_str())),
        ForeignErr::BadReturn(s) => (2, Some(s.as_str())),
        ForeignErr::Unavailable(s) => (3, Some(s.as_str())),
        // The WASM host runs foreign code in-process (phase-5h worker isolation is on the interpreter
        // engine), so `WorkerDied` is unreachable here; map it to the `Unavailable` variant defensively
        // so the language sum stays the fixed four-variant §8 shape.
        ForeignErr::WorkerDied(s) => (3, Some(s.as_str())),
    };
    let field = match msg {
        Some(m) => Some(write_str_cell(caller, m)?),
        None => None,
    };
    write_variant_cell(caller, tag, field)
}

/// Read `argc` marshalled foreign-argument cells (`[tag:i32 @0][pad @4][payload:8 @8]`, 16 bytes each)
/// from the guest args buffer at `args_ptr` into `FVal`s for `foreign::call`. `None` on any
/// out-of-bounds pointer (a hostile guest can never fault the host). Str payloads are copied out of
/// guest memory; `ForeignPtr` payloads resolve through the host's foreign-pointer table.
fn read_fvals(caller: &mut Caller<'_, HostState>, args_ptr: i32, argc: i32) -> Option<Vec<FVal>> {
    if argc < 0 {
        return None;
    }
    let n = (argc as usize).checked_mul(16)?;
    let base = args_ptr as u32 as usize;
    // Copy the buffer region first so subsequent `read_guest_str` calls (which reborrow memory) are
    // free of an outstanding borrow.
    let buf = {
        let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
        let data = mem.data(&caller);
        let end = base.checked_add(n)?;
        if end > data.len() {
            return None;
        }
        data[base..end].to_vec()
    };
    let rd_i32 = |b: &[u8]| i32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    let mut out = Vec::with_capacity(argc as usize);
    for i in 0..argc as usize {
        let cell = i * 16;
        let tag = rd_i32(&buf[cell..cell + 4]);
        let pay = &buf[cell + 8..cell + 16];
        let fv = match tag {
            0 => FVal::Int(i64::from_le_bytes(pay.try_into().ok()?)),
            1 => FVal::Float(f64::from_le_bytes(pay.try_into().ok()?)),
            2 => FVal::Bool(rd_i32(pay) != 0),
            3 => {
                let sp = rd_i32(pay);
                let s = read_guest_str(caller, sp)?;
                FVal::Str(Rc::from(s.as_str()))
            }
            4 => {
                let idx = rd_i32(pay) as u32 as usize;
                let p = *caller.data().foreign_ptrs.get(idx)?;
                FVal::Ptr(p)
            }
            _ => FVal::Unit,
        };
        out.push(fv);
    }
    Some(out)
}

/// Build the `delulu:cap` host import world: `root_console` mints a Console handle from the root
/// (host-side grant check), and `console_println` performs the Write and reads the string from the
/// guest's exported memory. Neither ever traps from inside the callback.
fn build_linker(engine: &Engine) -> Result<Linker<HostState>, WasmError> {
    let mut linker = Linker::new(engine);
    linker
        .func_wrap("delulu:cap", "root_console", |mut caller: Caller<'_, HostState>, root: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1; // a prior refusal already poisoned this run; don't overwrite its cause
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            if !caller.data().console_granted {
                caller.data_mut().refused = Some("DL0703: console was not granted to this program".into());
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Console);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "console_println", |mut caller: Caller<'_, HostState>, cap: i32, ptr: i32, file: i32, start: i32, end: i32| {
            if caller.data().refused.is_some() {
                return; // a prior refusal stands as the root cause; don't clobber it with a use-site error
            }
            let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Console)).unwrap_or(false);
            if !ok {
                caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Console capability"));
                return;
            }
            // Custody gate (Stage 5 phase 5f, epoch class). A denial is recorded, never an Err (trap 5).
            if !caller.data_mut().custody_allows(CustodyOp::Console, None) {
                return;
            }
            let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
                caller.data_mut().refused = Some("guest exports no `memory`".into());
                return;
            };
            let s = {
                let data = mem.data(&caller);
                // A wasm pointer is an UNSIGNED 32-bit offset; interpret it that way and use checked
                // arithmetic so a hostile `ptr` (e.g. -1 / near u32::MAX) can never overflow `usize`
                // and panic the host — a host panic inside a wasm callback would abort the process.
                let p = ptr as u32 as usize;
                let header_end = match p.checked_add(4) {
                    Some(e) if e <= data.len() => e,
                    _ => {
                        caller.data_mut().refused = Some(format!("DL0903: string header at {p} is out of bounds (memory is {} bytes)", data.len()));
                        return;
                    }
                };
                let len = u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
                let body_end = match header_end.checked_add(len) {
                    Some(e) if e <= data.len() => e,
                    _ => {
                        caller.data_mut().refused = Some(format!("DL0903: string of {len} bytes at {header_end} runs past the end of memory ({} bytes)", data.len()));
                        return;
                    }
                };
                String::from_utf8_lossy(&data[header_end..body_end]).to_string()
            };
            // Record the Write `TraceRecord` (detail = the printed string, matching the interpreter's
            // `trace_detail`) before performing the effect.
            caller.data_mut().push_trace("Write", "println", "Console", Some(s.clone()), file, start, end);
            let out = &mut caller.data_mut().output;
            out.push_str(&s);
            out.push('\n');
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "root_clock", |mut caller: Caller<'_, HostState>, root: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            if !caller.data().clock_granted {
                caller.data_mut().refused = Some("DL0703: clock was not granted to this program".into());
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Clock);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "clock_now_ms", |mut caller: Caller<'_, HostState>, cap: i32, file: i32, start: i32, end: i32| -> i64 {
            if caller.data().refused.is_some() {
                return 0;
            }
            let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Clock)).unwrap_or(false);
            if !ok {
                caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Clock capability"));
                return 0;
            }
            // Custody gate (Stage 5 phase 5f, epoch class). Recorded refusal, never an Err (trap 5).
            if !caller.data_mut().custody_allows(CustodyOp::Clock, None) {
                return 0;
            }
            caller.data_mut().push_trace("Clock", "now_ms", "Clock", None, file, start, end);
            // The clock read happens host-side: fixed for deterministic replay, else the wall clock —
            // the same source the interpreter uses, so a fixed clock gives byte-identical output.
            caller.data().fixed_clock_ms.unwrap_or_else(wall_clock_ms)
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "root_rand", |mut caller: Caller<'_, HostState>, root: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            if !caller.data().rand_granted {
                caller.data_mut().refused = Some("DL0703: rand was not granted to this program".into());
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Rand);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "rand_int", |mut caller: Caller<'_, HostState>, cap: i32, lo: i64, hi: i64, file: i32, start: i32, end: i32| -> i64 {
            if caller.data().refused.is_some() {
                return 0;
            }
            let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Rand)).unwrap_or(false);
            if !ok {
                caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Rand capability"));
                return 0;
            }
            // Custody gate (Stage 5 phase 5f, epoch class). Recorded refusal, never an Err (trap 5).
            if !caller.data_mut().custody_allows(CustodyOp::Rand, None) {
                return 0;
            }
            caller.data_mut().push_trace("Rand", "int", "Rand", None, file, start, end);
            if hi <= lo {
                caller.data_mut().refused = Some("DL0904: rand.int requires lo < hi".into());
                return lo;
            }
            // Advance the PRNG one step (host-side), then map exactly as the interpreter does:
            // `lo + (next_rand() % (hi - lo)) as i64`. `wrapping_*` avoids a debug-build host panic on
            // the full-i64-range span while matching the interpreter's release semantics.
            let st = caller.data_mut();
            let x = xorshift64(st.rng);
            st.rng = x;
            let span = hi.wrapping_sub(lo) as u64;
            lo.wrapping_add((x % span) as i64)
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "root_fs_read", |mut caller: Caller<'_, HostState>, root: i32, path_ptr: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            let Some(path) = read_guest_str(&mut caller, path_ptr) else {
                caller.data_mut().refused = Some("DL0903: fs_read path pointer is out of bounds".into());
                return -1;
            };
            // Mint a scope exactly as the interpreter does: normalize(cwd/path), granted iff within a
            // granted subtree (§4 / prim.rs::fs_read).
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&path));
            let granted = caller.data().fs_read_roots.iter().any(|g| want.starts_with(g));
            if !granted {
                caller.data_mut().refused = Some(format!("DL0703: filesystem read of `{path}` was not granted"));
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::FsRead(want));
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "fs_read_text", |mut caller: Caller<'_, HostState>, cap: i32, path_ptr: i32, file: i32, start: i32, end: i32| -> i32 {
            if caller.data().refused.is_some() {
                return 0;
            }
            let scope = match caller.data().caps.get(cap as usize) {
                Some(CapKind::FsRead(scope)) => scope.clone(),
                _ => {
                    caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted FsRead capability"));
                    return 0;
                }
            };
            let Some(rel) = read_guest_str(&mut caller, path_ptr) else {
                caller.data_mut().refused = Some("DL0903: read_text path pointer is out of bounds".into());
                return 0;
            };
            // Record the Read `TraceRecord` (detail = the relative path, matching `trace_detail`).
            caller.data_mut().push_trace("Read", "read_text", "FsRead", Some(rel.clone()), file, start, end);
            // Resolve within scope; a `..`/symlink escape is a hard DL0904 refusal, not an `Err`.
            let resolved = normalize(&scope.join(&rel));
            if !resolved.starts_with(&scope) {
                caller.data_mut().refused = Some(format!("DL0904: path `{rel}` escapes the granted scope"));
                return 0;
            }
            // Custody gate (Stage 5 phase 5f, epoch class) with the RESOLVED path — the same string
            // the broker node's fs scope was granted against. Recorded refusal, never an Err (trap 5).
            let resolved_str = resolved.to_string_lossy().to_string();
            if !caller.data_mut().custody_allows(CustodyOp::FsRead, Some(&resolved_str)) {
                return 0;
            }
            // Read host-side, then construct the Result[Str, IoErr] cell in guest memory (matching the
            // interpreter's io-error mapping: NotFound / PermissionDenied / Other(message)).
            let built = match std::fs::read_to_string(&resolved) {
                Ok(content) => build_ok_str(&mut caller, &content),
                Err(e) => {
                    use std::io::ErrorKind::*;
                    match e.kind() {
                        NotFound => build_err_ioerr(&mut caller, 0, None),
                        PermissionDenied => build_err_ioerr(&mut caller, 1, None),
                        _ => build_err_ioerr(&mut caller, 2, Some(&e.to_string())),
                    }
                }
            };
            match built {
                Ok(ptr) => ptr,
                Err(()) => {
                    caller.data_mut().refused = Some("guest heap exhausted building the fs result".into());
                    0
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // ----- Stage 4 phase 4g: the `delulu:foreign@0.4` host interface -------------------------------
    // `root.foreign_load()` mints a `Cap[ForeignLoad]` iff foreign loading is granted (a pure Root
    // derivation — not traced, mirroring `root.console()`).
    linker
        .func_wrap("delulu:foreign", "foreign_load", |mut caller: Caller<'_, HostState>, root: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            if !caller.data().foreign_load_granted {
                caller.data_mut().refused = Some("DL0703: foreign loading was not granted (grant a `foreign.c` lib or `foreign.python`)".into());
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::ForeignLoad);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `root.foreign(load)` binds the named lib host-side and returns a `Result[M, ForeignErr]` cell.
    // Binding resolves EVERY declared symbol fail-fast (spec §4.2 / DL1304) — a program that binds
    // clean never surprises the caller mid-run. Binding is pure (not traced), like the interpreter.
    linker
        .func_wrap("delulu:foreign", "foreign_bind", |mut caller: Caller<'_, HostState>, load: i32, name_ptr: i32| -> i32 {
            if caller.data().refused.is_some() {
                return 0;
            }
            let is_load = caller.data().caps.get(load as usize).map(|c| matches!(c, CapKind::ForeignLoad)).unwrap_or(false);
            if !is_load {
                caller.data_mut().refused = Some(format!("DL0904: handle {load} is not a Cap[ForeignLoad] capability"));
                return 0;
            }
            let Some(name) = read_guest_str(&mut caller, name_ptr) else {
                caller.data_mut().refused = Some("DL0903: foreign lib name pointer is out of bounds".into());
                return 0;
            };
            // Custody gate (Stage 5 phase 5f, SYNCHRONOUS class): a broker round-trip authorizes the
            // bind BEFORE any native code loads. The denial is recorded in HostState and surfaced
            // after the call — NEVER an Err returned from inside this callback (playbook trap 5).
            if !caller.data_mut().custody_allows(CustodyOp::ForeignBind, Some(&name)) {
                return 0;
            }
            // Mirror the interpreter's `bind_foreign` exactly: no grant → Err(NotGranted) (DL1303's
            // runtime face, defense-in-depth behind the CLI grant pre-flight); else load + resolve.
            let path = caller.data().foreign_grants.get(&name).cloned();
            let built = match path {
                None => build_foreign_err_cell(&mut caller, &ForeignErr::NotGranted)
                    .and_then(|fe| write_variant_cell(&mut caller, 1, Some(fe))),
                Some(path) => {
                    let sigs = caller.data().foreign_sigs.get(&name).cloned().unwrap_or_default();
                    match foreign::load_and_resolve(&path, sigs) {
                        Ok(lib) => {
                            // Stage 5 phase 5h: the WASM host's foreign path stays IN-PROCESS for v0.5
                            // (worker isolation is wired on the interpreter engine — see the phase-5h
                            // status note). Box the in-process lib through the same `BoundForeign` seam.
                            let handle = Rc::new(ForeignHandle { name: name.clone(), exec: Box::new(lib) });
                            let idx = {
                                let st = caller.data_mut();
                                st.caps.push(CapKind::Foreign(handle));
                                (st.caps.len() - 1) as i32
                            };
                            write_variant_cell(&mut caller, 0, Some(idx)) // Ok(handle)
                        }
                        Err(e) => build_foreign_err_cell(&mut caller, &e)
                            .and_then(|fe| write_variant_cell(&mut caller, 1, Some(fe))),
                    }
                }
            };
            match built {
                Ok(ptr) => ptr,
                Err(()) => {
                    caller.data_mut().refused = Some("guest heap exhausted building the foreign bind result".into());
                    0
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `m.method(args)` — marshal the args buffer, run `foreign::call` host-side (one code path with
    // the interpreter), validate the return (invariant 21), and return the marshalled value as an i64.
    // A `ForeignErr::BadReturn` is recorded as a DL1306 refusal and surfaced AFTER the run — never an
    // `Err` returned from inside the callback (the carried Wasmtime trap).
    linker
        .func_wrap(
            "delulu:foreign",
            "foreign_call",
            |mut caller: Caller<'_, HostState>, lib: i32, method_ptr: i32, args_ptr: i32, argc: i32, file: i32, start: i32, end: i32| -> i64 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let handle = match caller.data().caps.get(lib as usize) {
                    Some(CapKind::Foreign(h)) => h.clone(),
                    _ => {
                        caller.data_mut().refused = Some(format!("DL0904: handle {lib} is not a bound foreign lib"));
                        return 0;
                    }
                };
                let Some(method) = read_guest_str(&mut caller, method_ptr) else {
                    caller.data_mut().refused = Some("DL0903: foreign method name pointer is out of bounds".into());
                    return 0;
                };
                // Defensive: only call a method the bound lib actually resolved (else `foreign::call`
                // would `expect`-panic — a host panic aborts the process inside a wasm callback).
                if handle.exec.sig(&method).is_none() {
                    caller.data_mut().refused = Some(format!("DL0904: `{}` has no bound foreign method `{method}`", handle.name));
                    return 0;
                }
                // Record the ForeignCall trace BEFORE the call (a bad-return still shows the attempt),
                // exactly as the interpreter's `trace_foreign` does.
                let detail = format!("{}.{}", handle.name, method);
                caller.data_mut().push_trace("ForeignCall", &method, &handle.name, Some(detail), file, start, end);
                let Some(args) = read_fvals(&mut caller, args_ptr, argc) else {
                    caller.data_mut().refused = Some("DL0903: foreign args buffer is out of bounds".into());
                    return 0;
                };
                let max_ret = caller.data().max_ret();
                match handle.exec.call(&method, &args, max_ret) {
                    Ok(fv) => encode_fval_return(&mut caller, fv),
                    Err(ForeignErr::BadReturn(reason)) => {
                        caller.data_mut().refused =
                            Some(format!("DL1306: foreign return validation failed: {reason} (ForeignErr::BadReturn)"));
                        0
                    }
                    Err(other) => {
                        caller.data_mut().refused = Some(format!("DL1306: foreign call failed: {other:?}"));
                        0
                    }
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `str(Float)` formats host-side with the EXACT `Value::Float` display logic, so both engines
    // print a foreign Float identically. Pure (not traced).
    linker
        .func_wrap("delulu:foreign", "float_to_str", |mut caller: Caller<'_, HostState>, x: f64| -> i32 {
            if caller.data().refused.is_some() {
                return 0;
            }
            let s = format_float(x);
            match write_str_cell(&mut caller, &s) {
                Ok(ptr) => ptr,
                Err(()) => {
                    caller.data_mut().refused = Some("guest heap exhausted formatting a float".into());
                    0
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // ----- Stage 7 phase 7h: the `delulu:actors` host interface (cooperative scheduler) ---------
    // `spawn`, `send`, and field access all cross as i64 (Int and actor-ref slot ids are i64).
    // The callbacks only mutate `actor_rt`; the DRAIN loop (outside any wasm call) runs the turns.
    // A bad index cannot come from our codegen, but a callback must NEVER panic (that aborts the
    // process inside a wasm frame), so every access is bounds-checked and records a refusal instead.
    linker
        .func_wrap(
            "delulu:actors",
            "actor_spawn",
            |mut caller: Caller<'_, HostState>, actor_idx: i32, args_ptr: i32, argc: i32| -> i64 {
                if caller.data().refused.is_some() {
                    return -1;
                }
                let nactors = caller.data().actor_rt.table.actors.len();
                if actor_idx < 0 || actor_idx as usize >= nactors {
                    caller.data_mut().refused = Some(format!("actor index {actor_idx} out of range"));
                    return -1;
                }
                let Some(args) = read_i64_args(&mut caller, args_ptr, argc) else {
                    caller.data_mut().refused = Some("DL0903: actor spawn args buffer is out of bounds".into());
                    return -1;
                };
                caller.data_mut().actor_rt.spawn(actor_idx as u32, args) as i64
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:actors",
            "actor_send",
            |mut caller: Caller<'_, HostState>, slot: i64, behavior_idx: i32, args_ptr: i32, argc: i32| {
                if caller.data().refused.is_some() {
                    return;
                }
                let slot = slot as u64;
                let nslots = caller.data().actor_rt.slots.len() as u64;
                if slot >= nslots || behavior_idx < 0 {
                    caller.data_mut().refused = Some(format!("send to slot {slot} / behaviour {behavior_idx} out of range"));
                    return;
                }
                let Some(args) = read_i64_args(&mut caller, args_ptr, argc) else {
                    caller.data_mut().refused = Some("DL0903: actor send args buffer is out of bounds".into());
                    return;
                };
                caller.data_mut().actor_rt.send(slot, behavior_idx as u32, args);
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:actors",
            "actor_field_get",
            |mut caller: Caller<'_, HostState>, slot: i64, field_idx: i32| -> i64 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let v = {
                    let rt = &caller.data().actor_rt;
                    rt.slots.get(slot as usize).and_then(|s| s.fields.get(field_idx as usize)).copied()
                };
                match v {
                    Some(x) => x,
                    None => {
                        caller.data_mut().refused = Some(format!("field_get out of range (slot {slot}, field {field_idx})"));
                        0
                    }
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:actors",
            "actor_field_set",
            |mut caller: Caller<'_, HostState>, slot: i64, field_idx: i32, value: i64| {
                if caller.data().refused.is_some() {
                    return;
                }
                let ok = {
                    let rt = &mut caller.data_mut().actor_rt;
                    match rt.slots.get_mut(slot as usize).and_then(|s| s.fields.get_mut(field_idx as usize)) {
                        Some(f) => {
                            *f = value;
                            true
                        }
                        None => false,
                    }
                };
                if !ok {
                    caller.data_mut().refused = Some(format!("field_set out of range (slot {slot}, field {field_idx})"));
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    Ok(linker)
}

/// Read `argc` i64 arguments (8 bytes each, little-endian) from the guest args buffer at `ptr`.
/// `None` on any out-of-bounds pointer (a compiled guest never hits this; defensive against a
/// host panic inside a wasm callback). Stage 7 phase 7h.
fn read_i64_args(caller: &mut Caller<'_, HostState>, ptr: i32, argc: i32) -> Option<Vec<i64>> {
    if argc < 0 {
        return None;
    }
    if argc == 0 {
        return Some(Vec::new());
    }
    let n = (argc as usize).checked_mul(8)?;
    let base = ptr as u32 as usize;
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    let data = mem.data(&caller);
    let end = base.checked_add(n)?;
    if end > data.len() {
        return None;
    }
    let mut out = Vec::with_capacity(argc as usize);
    for i in 0..argc as usize {
        let o = base + i * 8;
        out.push(i64::from_le_bytes(data[o..o + 8].try_into().ok()?));
    }
    Some(out)
}

fn finish(mut store: Store<HostState>, func: wasmtime::Func, params: &[Val]) -> Result<String, WasmError> {
    let mut results: [Val; 0] = [];
    func.call(&mut store, params, &mut results).map_err(|e| WasmError::Trap(clean_trap(&e)))?;
    let state = store.into_data();
    if let Some(reason) = state.refused {
        return Err(WasmError::Trap(reason));
    }
    Ok(state.output)
}

/// Run an exported function that takes Console-capability handles directly (index 0 = a granted
/// Console). Returns the captured console output.
pub fn run_console_fn(wasm: &[u8], name: &str, cap_handles: &[usize]) -> Result<String, WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = HostState {
        caps: vec![CapKind::Console],
        console_granted: true,
        clock_granted: false,
        rand_granted: false,
        fs_read_roots: Vec::new(),
        fixed_clock_ms: None,
        rng: seed_rng(None),
        output: String::new(),
        refused: None,
        foreign_load_granted: false,
        foreign_grants: HashMap::new(),
        foreign_sigs: HashMap::new(),
        foreign_max_ret: 0,
        foreign_ptrs: Vec::new(),
        trace: None,
        trace_seq: 0,
        custody: None, // embedded (Stage 1–4 behavior)
        actor_rt: ActorRuntime::new(empty_actor_table()),
    };
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, name).ok_or_else(|| WasmError::NoExport(name.to_string()))?;
    let params: Vec<Val> = cap_handles.iter().map(|&h| Val::I32(h as i32)).collect();
    finish(store, func, &params)
}

/// An empty actor table for the non-actor entry points (`run_console_fn`/`run_main`) — the actor
/// imports are defined in the linker unconditionally, but a program without actors never calls them.
fn empty_actor_table() -> ActorTable {
    ActorTable { actors: Vec::new(), by_name: HashMap::new() }
}

/// Build the root `HostState` for running `main` (shared by [`run_main`] and [`run_main_actors`]).
fn main_host_state(cfg: &HostConfig, actor_rt: ActorRuntime) -> HostState {
    HostState {
        caps: vec![CapKind::Root],
        console_granted: cfg.console,
        clock_granted: cfg.clock,
        rand_granted: cfg.rand,
        fs_read_roots: cfg.fs_read_roots.clone(),
        fixed_clock_ms: cfg.fixed_clock_ms,
        rng: seed_rng(cfg.rand_seed),
        output: String::new(),
        refused: None,
        foreign_load_granted: cfg.foreign_load,
        foreign_grants: cfg.foreign_grants.clone(),
        foreign_sigs: cfg.foreign_sigs.clone(),
        foreign_max_ret: cfg.foreign_max_ret,
        foreign_ptrs: Vec::new(),
        trace: cfg.trace.clone(),
        trace_seq: 0,
        custody: cfg.custody.clone(),
        actor_rt,
    }
}

/// Grants and determinism for running `main` under the host (the run-time authority a `--grant`/
/// `--clock`/`--seed` flow resolves to).
#[derive(Clone, Default)]
pub struct HostConfig {
    pub console: bool,
    pub clock: bool,
    pub rand: bool,
    /// Granted filesystem-read subtrees (absolute, normalized — the same the interpreter's
    /// `granted_root` produces). `root.fs_read(p)` succeeds iff `p` resolves within one.
    pub fs_read_roots: Vec<PathBuf>,
    /// `Some(ms)` fixes `Cap[Clock].now_ms()` for deterministic replay (spec §6.2); `None` = wall clock.
    pub fixed_clock_ms: Option<i64>,
    /// `Some(seed)` seeds `Cap[Rand]` deterministically (spec §6.2); `None` = a nondeterministic seed.
    pub rand_seed: Option<u64>,
    // ----- Stage 4 phase 4g: foreign C FFI + effect tracing ------------------------------------
    /// Whether `root.foreign_load()` may mint a `Cap[ForeignLoad]` (any `foreign.c` lib or
    /// `foreign.python` pattern granted — mirrors the interpreter's `RootVal.foreign_load`).
    pub foreign_load: bool,
    /// `foreign.c` grants: logical lib name → the binary path the human chose (spec §4.1).
    pub foreign_grants: HashMap<String, String>,
    /// Each `foreign` lib's declared marshalling signatures (lowered from the block).
    pub foreign_sigs: HashMap<String, Vec<ForeignSig>>,
    /// Ceiling on a returned foreign string (`--foreign-max-ret`, invariant 21). 0 = the default.
    pub foreign_max_ret: usize,
    /// `Some` records one `TraceRecord` per effectful op host-side (`--engine wasm --trace-effects`),
    /// so the trace is byte-identical to the interpreter's (criterion 6).
    pub trace: Option<TraceSink>,
    /// Stage 5 phase 5f: `Some` routes gated ops through the custody seam (daemon mode). `None` =
    /// embedded, byte-identical Stage 1–4 behavior (criterion 11).
    pub custody: Option<CustodyHandle>,
}

/// Run `main(root: Root)` under Wasmtime with the given grants/determinism. The root handle (index 0)
/// is passed in; `root.console()`/`root.clock()`/`root.rand()` mint their handles host-side iff granted.
/// Returns the captured console output (or a `WasmError` if a capability was refused).
pub fn run_main(wasm: &[u8], cfg: &HostConfig) -> Result<String, WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = main_host_state(cfg, ActorRuntime::new(empty_actor_table()));
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, "main").ok_or_else(|| WasmError::NoExport("main".to_string()))?;
    finish(store, func, &[Val::I32(0)]) // root handle
}

/// Run an actor `main(root: Root)` under the WASM engine's cooperative single-threaded scheduler
/// (Stage 7 phase 7h, spec §6.5): call `main` (which `spawn`s actors and enqueues sends host-side),
/// then drain the FIFO job queue to quiescence, running one turn at a time. Returns the console
/// output and the quiescence [`ActorReport`] — the same accounting the native `QuiesceReport`
/// makes, so a deterministic program's counts match on both engines (criterion 6).
pub fn run_main_actors(
    wasm: &[u8],
    cfg: &HostConfig,
    table: &ActorTable,
) -> Result<(String, ActorReport), WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = main_host_state(cfg, ActorRuntime::new(table.clone()));
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let main = instance.get_func(&mut store, "main").ok_or_else(|| WasmError::NoExport("main".to_string()))?;
    // `main`'s own turn: a trap here fails the whole run (it is not an actor turn — no poison).
    let mut results: [Val; 0] = [];
    main.call(&mut store, &[Val::I32(0)], &mut results).map_err(|e| WasmError::Trap(clean_trap(&e)))?;
    if let Some(reason) = store.data().refused.clone() {
        return Err(WasmError::Trap(reason));
    }
    // Cooperative drain to quiescence (spec §6.1 exit condition: `main` returned AND the queue is
    // empty AND no turn is running — trivially true single-threaded once the queue drains).
    drain_actors(&mut store, &instance)?;
    let data = store.into_data();
    if let Some(reason) = data.refused {
        return Err(WasmError::Trap(reason));
    }
    Ok((data.output, data.actor_rt.report()))
}

/// Drain the actor job queue one turn at a time until quiescence (Stage 7 phase 7h). A `Create`
/// runs the ctor as the actor's first turn; a `Send` runs a behaviour (dropped+counted if the
/// target is poisoned — spec §6.6). A wasm trap during a turn poisons that actor and the system
/// stays live (later sends to it drop). The store stays usable after a trap, so other actors run on.
fn drain_actors(store: &mut Store<HostState>, instance: &Instance) -> Result<(), WasmError> {
    // Queue empty -> quiescence.
    while let Some(job) = store.data_mut().actor_rt.queue.pop_front() {
        match job {
            ActorJob::Create { slot, actor_idx, args } => {
                let export = store.data().actor_rt.table.actors[actor_idx as usize].ctor_export.clone();
                let ok = call_actor_export(store, instance, &export, slot, &args)?;
                store.data_mut().actor_rt.total_turns += 1;
                if !ok {
                    poison(store, slot);
                }
            }
            ActorJob::Send { slot, behavior_idx, args } => {
                let (dead, actor_idx) = {
                    let s = &store.data().actor_rt.slots[slot as usize];
                    (s.dead, s.actor_idx)
                };
                if dead {
                    store.data_mut().actor_rt.dropped_sends += 1;
                    continue;
                }
                let export = {
                    let info = &store.data().actor_rt.table.actors[actor_idx as usize];
                    match info.behaviors.get(behavior_idx as usize) {
                        Some(b) => b.export.clone(),
                        // An unknown behaviour index cannot come from our codegen; drop+count.
                        None => {
                            store.data_mut().actor_rt.dropped_sends += 1;
                            continue;
                        }
                    }
                };
                let ok = call_actor_export(store, instance, &export, slot, &args)?;
                store.data_mut().actor_rt.total_turns += 1;
                if !ok {
                    poison(store, slot);
                }
            }
        }
    }
    Ok(())
}

/// Call one actor member export `(self_slot, args…)`. Returns `Ok(true)` on success, `Ok(false)`
/// when the turn traps (a fault → poison, spec §6.6). Only a missing export (a codegen bug) is a
/// hard `Err`.
fn call_actor_export(
    store: &mut Store<HostState>,
    instance: &Instance,
    name: &str,
    slot: u64,
    args: &[i64],
) -> Result<bool, WasmError> {
    let func = instance.get_func(&mut *store, name).ok_or_else(|| WasmError::NoExport(name.to_string()))?;
    let mut params = Vec::with_capacity(1 + args.len());
    params.push(Val::I64(slot as i64));
    for &a in args {
        params.push(Val::I64(a));
    }
    let mut results: [Val; 0] = [];
    match func.call(&mut *store, &params, &mut results) {
        Ok(()) => Ok(true),
        Err(_trap) => Ok(false), // a behaviour fault poisons its actor; the system stays live
    }
}

/// Poison an actor after a faulting turn: mark it dead (once) and count it. Later sends to it drop.
fn poison(store: &mut Store<HostState>, slot: u64) {
    let rt = &mut store.data_mut().actor_rt;
    let newly_dead = match rt.slots.get_mut(slot as usize) {
        Some(s) if !s.dead => {
            s.dead = true;
            true
        }
        _ => false,
    };
    if newly_dead {
        rt.dead_actors += 1;
    }
}

/// Back-compat convenience: run `main` with only the console grant (no clock/rand, wall clock).
pub fn run_main_console(wasm: &[u8], console_granted: bool) -> Result<String, WasmError> {
    run_main(wasm, &HostConfig { console: console_granted, ..HostConfig::default() })
}
