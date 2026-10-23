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
    harden_wasm_features(&mut config);
    // `Speed` is an always-valid level, so construction cannot fail on wasmtime 27; fall back to
    // the default engine rather than panic if a future wasmtime ever disagrees.
    Engine::new(&config).unwrap_or_default()
}

/// Turn off every WebAssembly proposal this backend does not emit (campaign P17-F).
///
/// # Why
///
/// The doc comment above pins `cranelift_opt_level` **explicitly** rather than inheriting
/// wasmtime's default, so the tier "cannot silently change if a future wasmtime default does".
/// That argument was never applied to the *feature set*, and the feature set is the larger
/// attack surface: `Config::new()` leaves SIMD, threads, memory64 and the component model at
/// wasmtime's defaults, so the engine happily **accepts** modules using them even though
/// `codegen.rs` emits none of them (its only float instructions are `F64Const`, `F64Load`,
/// `F64ReinterpretI64` and `F64Store` — no `v128` anywhere).
///
/// That gap is not theoretical. `cargo deny check advisories` on the pinned wasmtime 27 reports
/// several vulnerabilities in exactly these proposals, including
/// **RUSTSEC-2026-0087** (`f64x2.splat` on Cranelift x86-64: segfault or out-of-sandbox load) and
/// the component-model string-transcoding family (0091/0092/0093). "Our compiler does not emit it"
/// is a fact about *our* front end; the engine also runs `.dwx` plugin artifacts, which arrive as
/// bytes. Refusing the feature at validation is the fail-closed version of the same claim.
///
/// This narrows what the engine accepts. It cannot widen it, so it can only ever turn a program
/// that would have run into one that is refused — and no program this compiler produces uses any
/// of these. `hardened_engine_refuses_a_simd_module` proves the setting actually takes effect
/// rather than being silently dropped by `unwrap_or_default()`.
pub(crate) fn harden_wasm_features(config: &mut Config) {
    config.wasm_simd(false);
    config.wasm_relaxed_simd(false);
    config.wasm_threads(false);
    config.wasm_memory64(false);
    config.wasm_component_model(false);
    // RW 4.33 (routine run 4, D-V2-68): the WebAssembly 3.0 proposals. `codegen.rs` emits none of them and
    // neither does a default wasm32 toolchain, yet wasmtime turns function references, GC, exceptions,
    // tail calls and multi-memory on by default — and RUSTSEC-2026-0315 (fuel accounting dropped through
    // `call_ref` and an exception `catch`) could reach a Contained plugin's GRANTED fuel through exactly
    // those. The ones wasmtime leaves off today are named too, so a future default cannot switch them
    // on here. What ordinary toolchains do emit — the WebAssembly 2.0 set: bulk memory, reference types,
    // multi-value, sign extension, saturating conversions — stays (a test holds both sides).
    config.wasm_gc(false);
    config.wasm_function_references(false);
    // (The legacy exceptions form is not named: wasmtime keeps its switch for its own spec tests only, and
    // Cranelift cannot compile it — an engine asked for it is refused.)
    config.wasm_exceptions(false);
    config.wasm_stack_switching(false);
    config.wasm_tail_call(false);
    config.wasm_multi_memory(false);
    config.wasm_custom_page_sizes(false);
    config.wasm_wide_arithmetic(false);
    config.wasm_shared_everything_threads(false);
}

// Stage 7 phase 7h: the WASM engine's cooperative single-threaded actor scheduler (spec §6.5).
use crate::actors::{ActorJob, ActorReport, ActorRuntime, ActorTable};

// Stage 4 phase 4g: the WASM host reuses the interpreter's C FFI machinery and its trace types, so
// verify≡run stays one code path (spec §4.3 — all of §4.1–4.2 runs host-side).
use delulu_runtime::foreign::{self, FVal, ForeignErr, ForeignHandle, ForeignSig};
// Stage 5 phase 5f: the WASM host calls the SAME `Custody` seam the interpreter does (playbook §1 —
// "the interpreter and the WASM host both call the trait"). `None` = embedded (Stage 1–4 behavior).
use delulu_runtime::{Custody, CustodyDecision, Op as CustodyOp};
use delulu_runtime::{TraceRecord, TraceSink, Value};

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
// wasmtime 47 stopped re-exporting `anyhow::Error` and introduced its own `wasmtime::Error`
// (P17: upgraded from 27 to close RUSTSEC-2026-0096, the aarch64 Cranelift sandbox escape, and
// RUSTSEC-2026-0222). The body is unchanged — `wasmtime::Error` carries the same `downcast_ref`
// and `Display` this function relies on.
pub(crate) fn clean_trap(e: &wasmtime::Error) -> String {
    use wasmtime::Trap;
    let first_line = || e.to_string().lines().next().unwrap_or("wasm trap").to_string();
    match e.downcast_ref::<Trap>() {
        Some(t) => match t {
            Trap::IntegerDivisionByZero => "DL0902: division by zero".to_string(),
            // `i64.div_s` of INT_MIN/-1 traps as IntegerOverflow, which the interpreter reports as
            // DL0901 (checked_div returns None) — so it maps to overflow, not divide-by-zero.
            Trap::IntegerOverflow => "DL0901: integer overflow".to_string(),
            Trap::StackOverflow => "DL0905: recursion depth exceeded (guest stack exhausted)".to_string(),
            // `__alloc` traps this way when the memory cannot give an allocation (past WebAssembly's 4 GiB, or a growth the
            // engine refuses) — the backend emits no other out-of-bounds access — so the text names both.
            Trap::MemoryOutOfBounds | Trap::HeapMisaligned | Trap::TableOutOfBounds => MEMORY_EXHAUSTED.to_string(),
            // The codegen traps checked `+`,`-`,`*` overflow and `INT_MIN % -1` via `unreachable`, each DL0901 on the
            // interpreter too. `%` by zero is left to `i64.rem_s`, which traps `IntegerDivisionByZero` (DL0902, above);
            // it used `unreachable` as well, and so read as overflow (C20's residual, closed 2026-10-24).
            Trap::UnreachableCodeReached => "DL0901: integer overflow".to_string(),
            _ => first_line(),
        },
        None => first_line(),
    }
}

/// Run an exported PURE function with i64 arguments, under a host that grants nothing (no ambient authority). The host
/// is there for what a pure function may still import: the `delulu:list` interface (RW 3.4), whose `split` and index
/// fault need no grant; a capability import it reached would be refused, as an ungranted `main`'s is.
pub fn run_int_fn(wasm: &[u8], name: &str, args: &[i64]) -> Result<i64, WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = main_host_state(&HostConfig::default(), ActorRuntime::new(empty_actor_table()));
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, name).ok_or_else(|| WasmError::NoExport(name.to_string()))?;
    let params: Vec<Val> = args.iter().map(|&a| Val::I64(a)).collect();
    let mut results = vec![Val::I64(0)];
    func.call(&mut store, &params, &mut results).map_err(|e| trap_or_refusal(&store, &e))?;
    if let Some(reason) = store.data().refused.clone() {
        return Err(WasmError::Trap(reason));
    }
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
    /// A filesystem-read capability scoped to an absolute, normalized subtree (§4), and where that subtree resolved on
    /// disk when the capability was minted or narrowed — its anchor, which every use is decided against (RW 4.72, as
    /// the interpreter's `CapScope::Fs`).
    FsRead(PathBuf, PathBuf),
    /// A filesystem-write capability, scoped and anchored the same way (`root.fs_write(p)`).
    FsWrite(PathBuf, PathBuf),
    /// A network capability (`root.http(hosts)`): the entries it may reach and the subset of them a `net.special=` grant
    /// covers — the interpreter's `CapScope::Net`, minted by its own rule.
    Http(Vec<String>, Vec<String>),
    /// `Cap[ForeignLoad]` — the loader capability minted by `root.foreign_load()` (Stage 4 phase 4g).
    ForeignLoad,
    /// A bound `foreign` lib handle (Stage 4 phase 4g): the loaded library + resolved symbols, held
    /// in the same host-side table as every other capability. `Rc` because `LoadedLib` owns a
    /// `Library` and is not `Clone`.
    Foreign(Rc<ForeignHandle>),
    /// A `Secret[Str]` (`root.secret(name)`, D-V2-174): the interpreter's own value, minted by its own rule — the bytes, or
    /// in daemon mode the broker's handle. The guest holds this entry's index and nothing of what it holds, so no byte of a
    /// secret is in the guest's memory (DL1205); a request sends it from here (`get_with_secret`).
    Secret(Rc<delulu_runtime::value::SecretVal>),
}

/// The run's secret grants, as the interpreter's `RootVal` holds them (`secrets`, `secret_audiences`, `broker_secrets`):
/// `root.secret(name)` mints by the interpreter's own rule against them. No `Debug`: the values are secrets.
#[derive(Clone, Default)]
pub struct SecretGrants {
    /// Embedded custody: each granted secret's bytes (`--grant secret:NAME=VALUE`).
    pub values: HashMap<String, String>,
    /// Where each may be sent (`secret:NAME@host:port=…`, P9-04 (c)).
    pub audiences: HashMap<String, Vec<delulu_runtime::egress::Audience>>,
    /// Daemon custody: the names of the broker-held secrets the run may name — a handle each, the bytes the broker's.
    pub broker: Vec<String>,
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
    /// Granted filesystem-write subtrees, the same way: `root.fs_write(p)` succeeds iff `p` resolves within one.
    fs_write_roots: Vec<PathBuf>,
    /// RW 6.15: the delegated node a `--lease` run's grants came from — which remedy a DL0703 names (`HostConfig::lease`).
    lease: Option<String>,
    /// The granted network entries and their `net.special=` subset (`HostConfig::net`, `net_special`).
    net: Vec<String>,
    net_special: Vec<String>,
    /// The run's secret grants (`HostConfig::secrets`).
    secrets: SecretGrants,
    /// ARTIFACT-CLAIM-1: a `.dwx`'s declared effects, the ceiling `custody_allows` enforces (`HostConfig::declared_effects`).
    declared_effects: Option<std::collections::BTreeSet<String>>,
    /// `Some(ms)` fixes `Cap[Clock].now_ms()` for deterministic replay (spec §6.2); `None` = wall clock.
    fixed_clock_ms: Option<i64>,
    /// The xorshift64 PRNG state for `Cap[Rand]` — seeded identically to the interpreter's so seeded
    /// `rand.int` sequences match byte-for-byte (spec §6.2, two-engine parity).
    rng: u64,
    output: String,
    /// `HostConfig::stream_console`: each printed line goes to stdout when printed, not into `output`.
    stream_console: bool,
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
        if self.foreign_max_ret == 0 {
            foreign::DEFAULT_MAX_RET
        } else {
            self.foreign_max_ret
        }
    }

    /// Consult custody for one gated op (Stage 5 phase 5f). On `Deny` the refusal is RECORDED in
    /// `refused` and `false` returned — **never an `Err` across the wasm frame** (playbook trap 5,
    /// carried from Stage 3/4): the runner surfaces the refusal cleanly after the call, exactly
    /// like the Stage-3 console/fs refusals. `None` custody (embedded) always allows.
    fn custody_allows(&mut self, op: CustodyOp, arg: Option<&str>) -> bool {
        self.custody_allows_at(op, arg, None)
    }

    /// ARTIFACT-CLAIM-1's ceiling: whether a `.dwx`'s manifest declares `effect` (always, for a program compiled here). A
    /// refusal is recorded (DL1202) and `false` returned. Asked by [`HostState::custody_allows_at`] of every custody op, and
    /// directly of an effect the host performs with no custody op, as the interpreter does (`Secret.verify`'s `Declassify`).
    fn declared_allows(&mut self, effect: &str) -> bool {
        let Some(declared) = &self.declared_effects else { return true };
        if declared.contains(effect) {
            return true;
        }
        self.refused = Some(format!(
            "DL1202: the artifact performed `{effect}`, an effect its `delulu:authority` manifest does not declare (it \
             declares {}) — its manifest is not this code's, so nothing it did not declare runs",
            if declared.is_empty() { "none".to_string() } else { declared.iter().cloned().collect::<Vec<_>>().join(", ") }
        ));
        false
    }

    /// [`HostState::custody_allows`] for a network use naming its request path (P9-05, D-V2-169): the custody judges it
    /// and names it in the use's record, as the interpreter's gate does.
    fn custody_allows_at(&mut self, op: CustodyOp, arg: Option<&str>, url_path: Option<&str>) -> bool {
        self.custody_allows_by(op, arg, url_path, None)
    }

    /// [`HostState::custody_allows_at`] by a network use's HTTP method when it is not a GET (P9-05 (d), D-V2-183: `post`),
    /// judged and recorded beside the path, as the interpreter's gate asks it.
    fn custody_allows_by(&mut self, op: CustodyOp, arg: Option<&str>, url_path: Option<&str>, method: Option<&str>) -> bool {
        // ARTIFACT-CLAIM-1: an artifact's declared effects are a ceiling its code cannot pass — asked first, of every effect
        // the host performs, by the rule the grant tree applies (`Op::required_effect`). Its code hash binds only the code,
        // so the manifest is its builder's word; enforced, the review surface that reads it is true of what can run.
        if let Some(effect) = op.required_effect() {
            if !self.declared_allows(effect.name()) {
                return false;
            }
        }
        let Some(custody) = &self.custody else { return true };
        match custody.borrow_mut().check_at(op, arg, url_path, method) {
            CustodyDecision::Allow => true,
            CustodyDecision::Deny(d) => {
                self.refused = Some(format!("{}: {}", d.code, d.message));
                false
            }
        }
    }

    /// Append one effect `TraceRecord` (no-op without a sink). `seq`/fields mirror the interpreter's
    /// so the two engines' traces are identical.
    ///
    /// Eight arguments, one over clippy's threshold, and grouping them would make the two engines
    /// harder to compare rather than easier: every parameter here corresponds positionally to a
    /// field the interpreter's own trace call passes, and the differential test that holds the two
    /// engines to identical traces is read by putting the two call sites side by side. A struct
    /// here and not there would hide exactly the drift that test exists to catch.
    #[allow(clippy::too_many_arguments)]
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

    /// RW 4.62: the record that follows a traced operation the host then REFUSED — `<op>.refused`, the
    /// dispatch's effect, and the refusal in `detail` as `CODE: message`, exactly as the interpreter's
    /// `trace_refusal` writes it, so the two engines' traces stay identical. Reads the refusal just set.
    fn push_refusal_trace(&mut self, effect: &str, op: &str, cap_kind: &str, file: i32, start: i32, end: i32) {
        let detail = self.refused.clone();
        self.push_trace(effect, &format!("{op}.refused"), cap_kind, detail, file, start, end);
    }

    /// A file effect's record (`read_text`, `write_text`, `append_text`), with the two additive filesystem fields the
    /// interpreter's record carries (REMAINING_WORK 6.13): where the path pointed, and the scope it was resolved in.
    #[allow(clippy::too_many_arguments)]
    fn push_fs_trace(
        &mut self,
        (effect, op, cap_kind): (&str, &str, &str),
        rel: &str,
        pointed: &std::path::Path,
        scope: &std::path::Path,
        file: i32,
        start: i32,
        end: i32,
    ) {
        let Some(sink) = &self.trace else { return };
        let seq = self.trace_seq;
        self.trace_seq += 1;
        sink.push(TraceRecord {
            seq,
            effect: effect.to_string(),
            op: op.to_string(),
            cap_kind: cap_kind.to_string(),
            detail: Some(rel.to_string()),
            span: Some((file as u32, start as u32, end as u32)),
            resolved_path: Some(pointed.to_string_lossy().to_string()),
            scope_root: Some(scope.to_string_lossy().to_string()),
            ..Default::default()
        });
    }
}

/// Run the interpreter's builtin `name` (`parse_int`, `parse_float`) on the guest string at `s`: `Some(Some(payload))` for
/// its `Some`, `Some(None)` for its `None`, and `None` once a refusal is recorded — a string outside the guest's memory, a
/// fault, or an answer of another shape.
fn parse_by_builtin(caller: &mut Caller<'_, HostState>, name: &str, s: i32) -> Option<Option<Value>> {
    if caller.data().refused.is_some() {
        return None;
    }
    let Some(text) = read_guest_str(caller, s) else {
        caller.data_mut().refused = Some(format!("DL0903: a string given to `{name}` lies outside the guest's memory"));
        return None;
    };
    let answer = delulu_runtime::prim::call_builtin(name, &[Value::str(text)], delulu_diag::Span::new(0, 0, 0));
    match answer {
        Some(Ok(Value::Variant { name: ctor, fields })) => match (&*ctor, fields.as_slice()) {
            ("Some", [v]) => return Some(Some(v.clone())),
            ("None", []) => return Some(None),
            _ => {}
        },
        Some(Err(fault)) => {
            caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
            return None;
        }
        _ => {}
    }
    caller.data_mut().refused = Some(format!("`{name}` answered a value the WASM engine does not hold"));
    None
}

/// The guest values at `ptrs`, of the type the descriptor at `desc` writes, rebuilt as the interpreter's (`reify.rs`) —
/// through one map of cells, so a cell two of them hold is one cell, as on the interpreter (`p == p` compares a cell with
/// itself there too). `None` once a refusal is recorded: a descriptor this engine did not write, or a value that is not
/// well-formed.
fn reify_values(caller: &mut Caller<'_, HostState>, desc: i32, ptrs: &[i32]) -> Option<Vec<Value>> {
    if caller.data().refused.is_some() {
        return None;
    }
    let Some(d) = read_guest_str(caller, desc).and_then(|t| crate::reify::parse(&t)) else {
        caller.data_mut().refused = Some("DL0903: a value's type descriptor is not one the WASM engine wrote".into());
        return None;
    };
    let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
        caller.data_mut().refused = Some("DL0903: a module with no memory".into());
        return None;
    };
    let rebuilt: Result<Vec<Value>, String> = {
        let data = mem.data(&caller);
        let mut r = crate::reify::Reifier::new(data, &d);
        ptrs.iter().map(|&p| r.root(p)).collect()
    };
    match rebuilt {
        Ok(vs) => Some(vs),
        Err(why) => {
            caller.data_mut().refused = Some(format!("DL0903: {why}"));
            None
        }
    }
}

/// Record that the interpreter's `name` answered `got`, which the guest cannot hold, and return `none`.
fn unheld<T>(caller: &mut Caller<'_, HostState>, name: &str, got: &Value, none: T) -> T {
    caller.data_mut().refused = Some(format!("`{name}` answered {got:?}, which the WASM engine does not hold"));
    none
}

/// `str` of a `Float` in the guest's memory, by the interpreter's own `Value::display` — one rule for both engines, not a
/// copy of it.
fn float_to_guest_str(caller: &mut Caller<'_, HostState>, x: f64) -> i32 {
    if caller.data().refused.is_some() {
        return 0;
    }
    match write_str_cell(caller, &Value::Float(x).display()) {
        Ok(ptr) => ptr,
        Err(()) => {
            caller.data_mut().refused = Some("guest heap exhausted formatting a float".into());
            0
        }
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
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
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
        None => {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E37_79B9)
                | 1
        }
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

/// `get_with_secret` and `post_with_secret` (D-V2-174; P9-05 (d), D-V2-183): `get`'s order — the record at dispatch, its
/// detail `«opaque»` as every secret-involving record's is; the custody on the URL's host (and, for the POST, its method),
/// unless the URL is not `https://`; then the secret judged and, if the broker holds it, its bytes asked for the one origin
/// by the interpreter's own `broker_secret_send`; then the interpreter's own method, its egress client and all. The bytes
/// go from the cap table to the egress client; the guest names the entry and is given the answer.
#[allow(clippy::too_many_arguments)]
fn send_with_secret(
    caller: &mut Caller<'_, HostState>,
    method: &'static str,
    cap: i32,
    url_ptr: i32,
    headers: i32,
    desc: i32,
    header_ptr: i32,
    secret: i32,
    body_ptr: Option<i32>,
    file: i32,
    start: i32,
    end: i32,
) -> i32 {
    let verb = delulu_runtime::prim::http_method_of(method);
    let verb = (verb != "GET").then_some(verb);
    if caller.data().refused.is_some() {
        return 0;
    }
    let (allow, special) = match caller.data().caps.get(cap as usize) {
        Some(CapKind::Http(allow, special)) => (allow.clone(), special.clone()),
        _ => {
            caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Http capability"));
            return 0;
        }
    };
    let held = match caller.data().caps.get(secret as usize) {
        Some(CapKind::Secret(s)) => s.clone(),
        _ => {
            caller.data_mut().refused = Some(format!("DL0904: handle {secret} is not a secret"));
            return 0;
        }
    };
    let Some(url) = read_guest_str(caller, url_ptr) else {
        caller.data_mut().refused = Some("DL0903: http get URL pointer is out of bounds".into());
        return 0;
    };
    let Some(header) = read_guest_str(caller, header_ptr) else {
        caller.data_mut().refused = Some("DL0903: a header's name lies outside the guest's memory".into());
        return 0;
    };
    let Some(mut hs) = reify_values(caller, desc, &[headers]) else { return 0 };
    let headers = hs.pop().unwrap_or(Value::Unit);
    // `post_with_secret`'s body (P9-05 (d), D-V2-183), read where its pointer is given.
    let body = match body_ptr {
        Some(p) => match read_guest_str(caller, p) {
            Some(b) => Some(b),
            None => {
                caller.data_mut().refused = Some("DL0903: a request body lies outside the guest's memory".into());
                return 0;
            }
        },
        None => None,
    };
    let detail = Some(delulu_runtime::trace::OPAQUE.to_string());
    caller.data_mut().push_trace("Net", method, "Http", detail, file, start, end);
    let host = url.starts_with("https://").then(|| delulu_runtime::prim::host_of(&url).to_string());
    if let Some(h) = &host {
        let url_path = delulu_runtime::prim::request_path(&url);
        if !caller.data_mut().custody_allows_by(CustodyOp::Net, Some(h), Some(url_path), verb) {
            caller.data_mut().push_refusal_trace("Net", method, "Http", file, start, end);
            return 0;
        }
        delulu_runtime::egress::begin_fetch();
    }
    let capv = delulu_runtime::value::CapVal {
        kind: delulu_check::ResourceKind::Http,
        scope: delulu_runtime::value::CapScope::Net { allow, special },
    };
    let span = delulu_diag::Span::new(file as u32, start as u32, end as u32);
    let mut args = vec![Value::str(url.clone()), headers, Value::str(header), Value::Secret(held)];
    if let Some(body) = body {
        args.push(Value::str(body));
    }
    // Embedded custody holds no broker secret, as the interpreter's `EmbeddedCustody` does not: a handle met
    // here is refused in its words, never sent empty.
    let send = match &caller.data().custody {
        Some(custody) => delulu_runtime::interp::broker_secret_send(&mut **custody.borrow_mut(), &capv, method, &args, span),
        None => {
            delulu_runtime::interp::broker_secret_send(&mut delulu_runtime::EmbeddedCustody::new(), &capv, method, &args, span)
        }
    };
    let answer = send.and_then(|send| match send {
        delulu_runtime::interp::SecretSend::AsGiven => delulu_runtime::prim::call_cap_method(&capv, method, &args, span),
        delulu_runtime::interp::SecretSend::Answered(v) => Ok(v),
        delulu_runtime::interp::SecretSend::Held(args) => delulu_runtime::prim::call_cap_method(&capv, method, &args, span),
    });
    if let Some(h) = &host {
        if let Some(reason) = delulu_runtime::egress::fetch_refusal() {
            if let Some(custody) = &caller.data().custody {
                custody.borrow_mut().note_egress_refusal(h, reason);
            }
        }
    }
    let answer = match answer {
        Ok(v) => v,
        Err(fault) => {
            caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
            caller.data_mut().push_refusal_trace("Net", method, "Http", file, start, end);
            return 0;
        }
    };
    match build_net_answer(caller, &answer) {
        Some(Ok(ptr)) => ptr,
        Some(Err(())) => {
            caller.data_mut().refused = Some("guest heap exhausted building the http result".into());
            0
        }
        None => unheld(caller, &format!("http.{method}"), &answer, 0),
    }
}

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

/// Bump-allocate `n` bytes in the guest heap (via the `__heap` global); returns the old pointer. The memory grows by the
/// pages the allocation needs, as the guest's own `__alloc` grows it (RW 3.4).
fn guest_alloc(caller: &mut Caller<'_, HostState>, n: i32) -> Result<i32, ()> {
    let g = caller.get_export("__heap").and_then(|e| e.into_global()).ok_or(())?;
    let base = match g.get(&mut *caller) {
        Val::I32(v) => v,
        _ => return Err(()),
    };
    let end = (base as u32 as u64).checked_add(u64::try_from(n).map_err(|_| ())?).filter(|e| *e <= u32::MAX as u64).ok_or(())?;
    let mem = caller.get_export("memory").and_then(|e| e.into_memory()).ok_or(())?;
    let size = mem.data_size(&*caller) as u64;
    if end > size {
        mem.grow(&mut *caller, (end - size).div_ceil(65536)).map_err(|_| ())?;
    }
    g.set(&mut *caller, Val::I32(end as u32 as i32)).map_err(|_| ())?;
    Ok(base)
}

/// What an access outside the guest's memory, or a memory that could not grow, is reported as (WASM-OOM-1).
const MEMORY_EXHAUSTED: &str =
    "DL0903: memory access out of bounds — or the WASM engine's memory exhausted (WebAssembly's 4 GiB)";

/// Write a `List[Str]` of `parts` (RW 3.4) in one allocation and one write: the header `codegen.rs` reads,
/// `[len][cap][data]`, with exactly as many slots as parts; the slots, each holding a string cell's pointer; then the
/// cells, `[len][bytes]` each. Returns the header's pointer. (Written a cell at a time, each with its own allocation and
/// export lookups, `wordcount_macro` ran slower on the WASM engine than on the interpreter — Study C, 2026-10-20.)
fn write_str_list(caller: &mut Caller<'_, HostState>, parts: &[String]) -> Result<i32, ()> {
    let n = u32::try_from(parts.len()).map_err(|_| ())?;
    let head = 12 + 8 * parts.len();
    let total = head + parts.iter().map(|p| 4 + p.len()).sum::<usize>();
    let base = guest_alloc(caller, i32::try_from(total).map_err(|_| ())?)? as u32;
    let mut bytes = Vec::with_capacity(total);
    let data = base + 12;
    for field in [n, n, data] {
        bytes.extend_from_slice(&field.to_le_bytes());
    }
    let mut cell = base + head as u32;
    for p in parts {
        bytes.extend_from_slice(&u64::from(cell).to_le_bytes());
        cell += 4 + p.len() as u32;
    }
    for p in parts {
        bytes.extend_from_slice(&(p.len() as u32).to_le_bytes());
        bytes.extend_from_slice(p.as_bytes());
    }
    guest_write(caller, base as i32, &bytes)?;
    Ok(base as i32)
}

/// `n` bytes of the guest's memory at `at`, or `None` when they lie outside it.
fn guest_read(caller: &mut Caller<'_, HostState>, at: u32, n: usize) -> Option<Vec<u8>> {
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    let a = at as usize;
    mem.data(&*caller).get(a..a.checked_add(n)?).map(<[u8]>::to_vec)
}

/// A `Map` in the guest's memory (RW 3.4, `codegen.rs`'s `Ty::Map`): its header `[len][cap][data]`, as a list's, and
/// `data` holding its pairs, 16 bytes each — the key's 8-byte slot, then the value's — ascending by key. The memory is read
/// as untrusted (a `.dwx` can be written by hand): every read is bounds-checked, and a map whose pairs are out of order is
/// answered wrongly, never unsafely.
struct GuestMap {
    at: u32,
    len: u32,
    cap: u32,
    data: u32,
}

/// What a map whose header or pairs lie outside the guest's memory, or whose data could not grow, is reported as.
const MAP_CORRUPT: &str = "DL0903: a map's pairs lie outside the guest's memory — or the WASM engine's memory exhausted";

impl GuestMap {
    fn read(caller: &mut Caller<'_, HostState>, at: i32) -> Option<GuestMap> {
        let b = guest_read(caller, at as u32, 12)?;
        let word = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        let (len, cap, data) = (word(0), word(4), word(8));
        (len <= cap).then_some(GuestMap { at: at as u32, len, cap, data })
    }

    fn pair(&self, i: u32) -> Option<u32> {
        self.data.checked_add(i.checked_mul(16)?)
    }

    /// The key of pair `i`, read as `codegen.rs`'s `store_scalar` wrote it, by its kind (`map_key_tag`).
    fn key(&self, caller: &mut Caller<'_, HostState>, kind: i32, i: u32) -> Option<delulu_runtime::value::MapKey> {
        let slot = guest_read(caller, self.pair(i)?, 8)?;
        let raw = i64::from_le_bytes(slot.try_into().ok()?);
        map_key_of(caller, kind, if kind == 0 { raw } else { i64::from(raw as u32) })
    }

    /// Where `key` is — `Ok(i)` — or where it would go, `Err(i)`: the interpreter's `BTreeMap` order, by its own `MapKey`.
    fn search(
        &self,
        caller: &mut Caller<'_, HostState>,
        kind: i32,
        key: &delulu_runtime::value::MapKey,
    ) -> Option<Result<u32, u32>> {
        let (mut lo, mut hi) = (0u32, self.len);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match self.key(caller, kind, mid)?.cmp(key) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return Some(Ok(mid)),
            }
        }
        Some(Err(lo))
    }

    fn write_header(&self, caller: &mut Caller<'_, HostState>) -> Option<()> {
        let mut b = Vec::with_capacity(12);
        for w in [self.len, self.cap, self.data] {
            b.extend_from_slice(&w.to_le_bytes());
        }
        guest_write(caller, self.at as i32, &b).ok()
    }

    /// A new list (`[len][cap][data]`, as `codegen.rs` reads one) of each pair's slot at `offset` — 0 its key, 8 its value.
    fn column(&self, caller: &mut Caller<'_, HostState>, offset: u32) -> Option<i32> {
        let n = self.len as usize;
        // The pairs are read before anything is reserved, so a header claiming more pairs than the memory holds is refused
        // with nothing allocated.
        let pairs = guest_read(caller, self.data, n.checked_mul(16)?)?;
        let mut slots = Vec::with_capacity(8 * n);
        for p in pairs.chunks_exact(16) {
            slots.extend_from_slice(&p[offset as usize..offset as usize + 8]);
        }
        let base = guest_alloc(caller, i32::try_from(12 + 8 * n).ok()?).ok()? as u32;
        let mut bytes = Vec::with_capacity(12 + 8 * n);
        for w in [self.len, self.len, base + 12] {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        bytes.extend_from_slice(&slots);
        guest_write(caller, base as i32, &bytes).ok()?;
        Some(base as i32)
    }
}

/// The key a guest passes a `delulu:map` import: an `Int` itself, a `Bool` as 0 or 1, a `Str` as its cell's pointer.
fn map_key_of(caller: &mut Caller<'_, HostState>, kind: i32, raw: i64) -> Option<delulu_runtime::value::MapKey> {
    use delulu_runtime::value::MapKey;
    Some(match kind {
        0 => MapKey::Int(raw),
        1 => MapKey::Bool(raw != 0),
        2 => MapKey::Str(read_guest_str(caller, raw as u32 as i32)?),
        _ => return None,
    })
}

/// A map import's failure: recorded in words (the first refusal stands), then a trap, so the guest never goes on past it.
fn map_fault(caller: &mut Caller<'_, HostState>) -> wasmtime::Error {
    caller.data_mut().refused.get_or_insert_with(|| MAP_CORRUPT.to_string());
    wasmtime::Error::msg(MAP_CORRUPT)
}

/// What a list whose header, slots or strings lie outside the guest's memory, or a result that could not be allocated,
/// is reported as (`delulu:list.sort`, `delulu:list.join`).
const LIST_CORRUPT: &str = "DL0903: a list's elements lie outside the guest's memory — or the WASM engine's memory exhausted";

/// A list's length and slots pointer (`[len][cap][data]`, as `codegen.rs` lays it out), its length within its capacity.
fn list_header(caller: &mut Caller<'_, HostState>, list: i32) -> Option<(u32, u32)> {
    let b = guest_read(caller, list as u32, 12)?;
    let word = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    (word(0) <= word(4)).then(|| (word(0), word(8)))
}

/// A string cell's length, the cell checked to lie inside the guest's memory.
fn guest_str_len(caller: &mut Caller<'_, HostState>, ptr: u32) -> Option<u32> {
    let len = u32::from_le_bytes(guest_read(caller, ptr, 4)?.try_into().ok()?);
    let end = (ptr as usize).checked_add(4)?.checked_add(len as usize)?;
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    (end <= mem.data_size(&*caller)).then_some(len)
}

/// A new list of `n` 8-byte `slots`, in one allocation: the header `codegen.rs` reads, then the slots.
fn write_slot_list(caller: &mut Caller<'_, HostState>, n: u32, slots: &[u8]) -> Option<i32> {
    let base = guest_alloc(caller, i32::try_from(12 + slots.len()).ok()?).ok()? as u32;
    let mut bytes = Vec::with_capacity(12 + slots.len());
    for w in [n, n, base + 12] {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    bytes.extend_from_slice(slots);
    guest_write(caller, base as i32, &bytes).ok()?;
    Some(base as i32)
}

/// `List.sort` (RW 3.4): a new list, stably sorted by the interpreter's order — an `Int` (`kind` 0) or a `Bool` (1) by
/// its number, a `Str` (2) by its bytes — its slots moved whole, so the strings it holds are the ones the list held.
/// Strings are compared in the guest's memory where they lie, never copied, so slots naming one string many times cost
/// nothing more.
fn sort_list(caller: &mut Caller<'_, HostState>, list: i32, kind: i32) -> Option<i32> {
    let (n, data) = list_header(caller, list)?;
    let slots = guest_read(caller, data, (n as usize).checked_mul(8)?)?;
    let slot = |i: usize| &slots[8 * i..8 * i + 8];
    let mut order: Vec<usize> = (0..n as usize).collect();
    match kind {
        0 => order.sort_by_key(|&i| i64::from_le_bytes(slot(i).try_into().unwrap_or_default())),
        1 => order.sort_by_key(|&i| u32::from_le_bytes(slot(i)[..4].try_into().unwrap_or_default()) != 0),
        2 => {
            let mut ranges = Vec::with_capacity(n as usize);
            for i in 0..n as usize {
                let p = u32::from_le_bytes(slot(i)[..4].try_into().ok()?);
                let len = guest_str_len(caller, p)?;
                let start = p as usize + 4;
                ranges.push(start..start + len as usize);
            }
            let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
            let bytes = mem.data(&*caller);
            order.sort_by(|&a, &b| bytes[ranges[a].clone()].cmp(&bytes[ranges[b].clone()]));
        }
        _ => return None,
    }
    let mut sorted = Vec::with_capacity(slots.len());
    for i in order {
        sorted.extend_from_slice(slot(i));
    }
    write_slot_list(caller, n, &sorted)
}

/// `List[Str].join(sep)` (RW 3.4): the strings and the separators between them, in one string cell. Every length is
/// checked and the total bounded before anything is allocated, and each piece is copied straight into the result, so the
/// host never holds more than one piece.
fn join_list(caller: &mut Caller<'_, HostState>, list: i32, sep: i32) -> Option<i32> {
    let (n, data) = list_header(caller, list)?;
    let slots = guest_read(caller, data, (n as usize).checked_mul(8)?)?;
    let sep = sep as u32;
    let sep_len = guest_str_len(caller, sep)?;
    let mut pieces = Vec::with_capacity(n as usize);
    let mut total: u64 = 0;
    for (k, s) in slots.chunks_exact(8).enumerate() {
        let p = u32::from_le_bytes(s[..4].try_into().ok()?);
        let len = guest_str_len(caller, p)?;
        total += u64::from(len) + if k > 0 { u64::from(sep_len) } else { 0 };
        pieces.push((p, len));
    }
    let total = u32::try_from(total).ok().filter(|t| *t <= i32::MAX as u32 - 4)?;
    let base = guest_alloc(caller, (4 + total) as i32).ok()? as u32;
    guest_write(caller, base as i32, &total.to_le_bytes()).ok()?;
    let mut at = base + 4;
    for (k, (p, len)) in pieces.into_iter().enumerate() {
        if k > 0 {
            let b = guest_read(caller, sep + 4, sep_len as usize)?;
            guest_write(caller, at as i32, &b).ok()?;
            at += sep_len;
        }
        let b = guest_read(caller, p + 4, len as usize)?;
        guest_write(caller, at as i32, &b).ok()?;
        at += len;
    }
    Some(base as i32)
}

/// `delulu:list.sort` and `delulu:list.join` (RW 3.4); a failure is recorded in words, then a trap.
fn link_list_host(linker: &mut Linker<HostState>) -> Result<(), WasmError> {
    let instantiate = |e: wasmtime::Error| WasmError::Instantiate(e.to_string());
    let fault = |caller: &mut Caller<'_, HostState>| {
        caller.data_mut().refused.get_or_insert_with(|| LIST_CORRUPT.to_string());
        wasmtime::Error::msg(LIST_CORRUPT)
    };
    linker
        .func_wrap(
            "delulu:list",
            "sort",
            move |mut caller: Caller<'_, HostState>, list: i32, kind: i32| -> wasmtime::Result<i32> {
                sort_list(&mut caller, list, kind).ok_or_else(|| fault(&mut caller))
            },
        )
        .map_err(instantiate)?;
    linker
        .func_wrap(
            "delulu:list",
            "join",
            move |mut caller: Caller<'_, HostState>, list: i32, sep: i32| -> wasmtime::Result<i32> {
                join_list(&mut caller, list, sep).ok_or_else(|| fault(&mut caller))
            },
        )
        .map_err(instantiate)?;
    Ok(())
}

/// The `delulu:map` interface (RW 3.4): a `Map` lives in the guest's memory, its values the guest's own, and the host
/// finds and places each key by the interpreter's own `MapKey` order — so the two engines cannot order keys two ways.
fn link_maps(linker: &mut Linker<HostState>) -> Result<(), WasmError> {
    let instantiate = |e: wasmtime::Error| WasmError::Instantiate(e.to_string());
    // An empty map: a header with no pairs and no data.
    linker
        .func_wrap("delulu:map", "new", |mut caller: Caller<'_, HostState>| -> wasmtime::Result<i32> {
            let at = guest_alloc(&mut caller, 12).map_err(|()| map_fault(&mut caller))?;
            GuestMap { at: at as u32, len: 0, cap: 0, data: 0 }
                .write_header(&mut caller)
                .ok_or_else(|| map_fault(&mut caller))?;
            Ok(at)
        })
        .map_err(instantiate)?;
    // The index of `key`'s pair, or -1.
    linker
        .func_wrap(
            "delulu:map",
            "find",
            |mut caller: Caller<'_, HostState>, map: i32, kind: i32, raw: i64| -> wasmtime::Result<i32> {
                let found = (|| {
                    let m = GuestMap::read(&mut caller, map)?;
                    let key = map_key_of(&mut caller, kind, raw)?;
                    m.search(&mut caller, kind, &key)
                })();
                match found {
                    Some(Ok(i)) => Ok(i as i32),
                    Some(Err(_)) => Ok(-1),
                    None => Err(map_fault(&mut caller)),
                }
            },
        )
        .map_err(instantiate)?;
    // The address of `key`'s value slot, its pair made first when the map holds no such key — in its place by the order,
    // the data grown (doubled, at least four pairs) when it is full. The guest fills the slot.
    linker
        .func_wrap(
            "delulu:map",
            "put",
            |mut caller: Caller<'_, HostState>, map: i32, kind: i32, raw: i64| -> wasmtime::Result<i32> {
                let slot = (|| {
                    let mut m = GuestMap::read(&mut caller, map)?;
                    let key = map_key_of(&mut caller, kind, raw)?;
                    let at = match m.search(&mut caller, kind, &key)? {
                        Ok(i) => i,
                        Err(i) => {
                            if m.len == m.cap {
                                let cap = m.cap.checked_mul(2)?.max(4);
                                let data = guest_alloc(&mut caller, i32::try_from(cap.checked_mul(16)?).ok()?).ok()? as u32;
                                if m.len > 0 {
                                    let old = guest_read(&mut caller, m.data, (m.len as usize).checked_mul(16)?)?;
                                    guest_write(&mut caller, data as i32, &old).ok()?;
                                }
                                (m.cap, m.data) = (cap, data);
                            }
                            let tail = guest_read(&mut caller, m.pair(i)?, ((m.len - i) as usize).checked_mul(16)?)?;
                            guest_write(&mut caller, m.pair(i + 1)? as i32, &tail).ok()?;
                            let mut pair = raw.to_le_bytes().to_vec();
                            pair.extend_from_slice(&[0; 8]);
                            guest_write(&mut caller, m.pair(i)? as i32, &pair).ok()?;
                            m.len += 1;
                            m.write_header(&mut caller)?;
                            i
                        }
                    };
                    i32::try_from(m.pair(at)?.checked_add(8)?).ok()
                })();
                slot.ok_or_else(|| map_fault(&mut caller))
            },
        )
        .map_err(instantiate)?;
    // Take pair `index` out (its value already read by the guest), the pairs after it moved down.
    linker
        .func_wrap("delulu:map", "remove_at", |mut caller: Caller<'_, HostState>, map: i32, index: i32| -> wasmtime::Result<()> {
            let done = (|| {
                let mut m = GuestMap::read(&mut caller, map)?;
                let i = u32::try_from(index).ok().filter(|i| *i < m.len)?;
                let tail = guest_read(&mut caller, m.pair(i + 1)?, ((m.len - i - 1) as usize).checked_mul(16)?)?;
                guest_write(&mut caller, m.pair(i)? as i32, &tail).ok()?;
                m.len -= 1;
                m.write_header(&mut caller)
            })();
            done.ok_or_else(|| map_fault(&mut caller))
        })
        .map_err(instantiate)?;
    for (name, offset) in [("keys", 0), ("values", 8)] {
        linker
            .func_wrap("delulu:map", name, move |mut caller: Caller<'_, HostState>, map: i32| -> wasmtime::Result<i32> {
                let list = GuestMap::read(&mut caller, map).and_then(|m| m.column(&mut caller, offset));
                list.ok_or_else(|| map_fault(&mut caller))
            })
            .map_err(instantiate)?;
    }
    Ok(())
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

/// The guest's `Result[Str, NetErr]` for the interpreter's answer to `http.get` — `Ok(body)`, or `Err` of `Refused` (0),
/// `Timeout` (1) or `Other(message)` (2), the cells `IoErr`'s are. `None` for an answer of another shape.
fn build_net_answer(caller: &mut Caller<'_, HostState>, answer: &Value) -> Option<Result<i32, ()>> {
    let Value::Variant { name, fields } = answer else { return None };
    match (&**name, fields.as_slice()) {
        ("Ok", [Value::Str(body)]) => Some(build_ok_str(caller, body)),
        ("Err", [Value::Variant { name: e, fields: ef }]) => match (&**e, ef.as_slice()) {
            ("Refused", []) => Some(build_err_ioerr(caller, 0, None)),
            ("Timeout", []) => Some(build_err_ioerr(caller, 1, None)),
            ("Other", [Value::Str(m)]) => Some(build_err_ioerr(caller, 2, Some(&**m))),
            _ => None,
        },
        _ => None,
    }
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
                let remedy = delulu_runtime::prim::grant_remedy(caller.data().lease.as_deref(), "console");
                // RW 6.15: the interpreter's words, remedy included.
                caller.data_mut().refused = Some(format!("DL0703: `console` was not granted to this program — {remedy}"));
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Console);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:cap",
            "console_println",
            |mut caller: Caller<'_, HostState>, cap: i32, ptr: i32, file: i32, start: i32, end: i32| {
                if caller.data().refused.is_some() {
                    return; // a prior refusal stands as the root cause; don't clobber it with a use-site error
                }
                let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Console)).unwrap_or(false);
                if !ok {
                    caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Console capability"));
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
                            caller.data_mut().refused =
                                Some(format!("DL0903: string header at {p} is out of bounds (memory is {} bytes)", data.len()));
                            return;
                        }
                    };
                    let len = u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
                    let body_end = match header_end.checked_add(len) {
                        Some(e) if e <= data.len() => e,
                        _ => {
                            caller.data_mut().refused = Some(format!(
                                "DL0903: string of {len} bytes at {header_end} runs past the end of memory ({} bytes)",
                                data.len()
                            ));
                            return;
                        }
                    };
                    String::from_utf8_lossy(&data[header_end..body_end]).to_string()
                };
                // Record the Write `TraceRecord` (detail = the printed string, matching the interpreter's
                // `trace_detail`) at DISPATCH — before the custody decides, as the interpreter records it (RW 4.64:
                // reading the guest's string performs nothing, so the order costs nothing and the traces agree).
                caller.data_mut().push_trace("Write", "println", "Console", Some(s.clone()), file, start, end);
                // Custody gate (Stage 5 phase 5f, epoch class). A denial is recorded, never an Err (trap 5) — and
                // followed in the trace by its refusal (D-V2-129).
                if !caller.data_mut().custody_allows(CustodyOp::Console, None) {
                    caller.data_mut().push_refusal_trace("Write", "println", "Console", file, start, end);
                    return;
                }
                if caller.data().stream_console {
                    println!("{s}");
                    return;
                }
                let out = &mut caller.data_mut().output;
                out.push_str(&s);
                out.push('\n');
            },
        )
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
                let remedy = delulu_runtime::prim::grant_remedy(caller.data().lease.as_deref(), "clock");
                // RW 6.15: the interpreter's words, remedy included.
                caller.data_mut().refused = Some(format!("DL0703: `clock` was not granted to this program — {remedy}"));
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Clock);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:cap",
            "clock_now_ms",
            |mut caller: Caller<'_, HostState>, cap: i32, file: i32, start: i32, end: i32| -> i64 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Clock)).unwrap_or(false);
                if !ok {
                    caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Clock capability"));
                    return 0;
                }
                // Traced at DISPATCH, then the custody gate (Stage 5 phase 5f, epoch class): a refusal is recorded,
                // never an Err (trap 5), and follows the dispatch in the trace (RW 4.64, D-V2-129).
                caller.data_mut().push_trace("Clock", "now_ms", "Clock", None, file, start, end);
                if !caller.data_mut().custody_allows(CustodyOp::Clock, None) {
                    caller.data_mut().push_refusal_trace("Clock", "now_ms", "Clock", file, start, end);
                    return 0;
                }
                // The clock read happens host-side: fixed for deterministic replay, else the wall clock —
                // the same source the interpreter uses, so a fixed clock gives byte-identical output.
                caller.data().fixed_clock_ms.unwrap_or_else(wall_clock_ms)
            },
        )
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
                let remedy = delulu_runtime::prim::grant_remedy(caller.data().lease.as_deref(), "rand");
                // RW 6.15: the interpreter's words, remedy included.
                caller.data_mut().refused = Some(format!("DL0703: `rand` was not granted to this program — {remedy}"));
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Rand);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:cap",
            "rand_int",
            |mut caller: Caller<'_, HostState>, cap: i32, lo: i64, hi: i64, file: i32, start: i32, end: i32| -> i64 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Rand)).unwrap_or(false);
                if !ok {
                    caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Rand capability"));
                    return 0;
                }
                // Traced at DISPATCH, then the custody gate (Stage 5 phase 5f, epoch class): a refusal is recorded,
                // never an Err (trap 5), and follows the dispatch in the trace (RW 4.64, D-V2-129).
                caller.data_mut().push_trace("Rand", "int", "Rand", None, file, start, end);
                if !caller.data_mut().custody_allows(CustodyOp::Rand, None) {
                    caller.data_mut().push_refusal_trace("Rand", "int", "Rand", file, start, end);
                    return 0;
                }
                if hi <= lo {
                    caller.data_mut().refused = Some("DL0904: rand.int requires lo < hi".into());
                    caller.data_mut().push_refusal_trace("Rand", "int", "Rand", file, start, end);
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
            },
        )
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
            if let Some(why) = delulu_runtime::prim::hostile_path(&path, false) {
                caller.data_mut().refused = Some(format!("DL0904: path `{}` is refused: {why}", path.escape_debug()));
                return -1;
            }
            // Mint a scope exactly as the interpreter does: normalize(cwd/path), granted iff within a
            // granted subtree (§4 / prim.rs::fs_read).
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&path));
            // Both gates, exactly as the interpreter applies them (C84): lexical containment, then
            // filesystem truth. Fault parity between the engines (D29) is a tested law, so a scope
            // rule that held in one engine and not the other would be a divergence in the direction
            // that matters most.
            // On disk only, as the interpreter's mint (a resolved lease scope against a path spelled from the working
            // directory) — and by its own `mint_anchor` (RW 4.72): the decision and the anchor are one resolution.
            let Some(anchor) = delulu_runtime::prim::mint_anchor(&caller.data().fs_read_roots, &want) else {
                let remedy = delulu_runtime::prim::grant_remedy(caller.data().lease.as_deref(), &format!("fs.read={path}"));
                caller.data_mut().refused = Some(format!("DL0703: filesystem read of `{path}` was not granted — {remedy}"));
                return -1;
            };
            let st = caller.data_mut();
            st.caps.push(CapKind::FsRead(want, anchor));
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap(
            "delulu:cap",
            "fs_read_text",
            |mut caller: Caller<'_, HostState>, cap: i32, path_ptr: i32, file: i32, start: i32, end: i32| -> i32 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let (scope, anchor) = match caller.data().caps.get(cap as usize) {
                    Some(CapKind::FsRead(scope, anchor)) => (scope.clone(), anchor.clone()),
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
                // Like the interpreter, it also names where the path pointed and the scope (RW 6.13). At DISPATCH:
                // a hostile spelling is refused after it, as the interpreter refuses it (RW 4.64).
                let pointed = normalize(&scope.join(&rel));
                caller.data_mut().push_fs_trace(("Read", "read_text", "FsRead"), &rel, &pointed, &scope, file, start, end);
                // Then the interpreter's order, as the write's: the custody gate (Stage 5 phase 5f, epoch class) with the
                // RESOLVED path — the same string the broker node's fs scope was granted against; a recorded refusal,
                // never an Err (trap 5); GUARD-ALIAS-1: decided on the path it resolves to, which is then opened exactly.
                let pin = delulu_runtime::prim::pin_fs_at(&scope, &anchor, &rel);
                let resolved_str = match &pin {
                    Some(real) => delulu_runtime::prim::disk_spelling(real),
                    None => pointed.to_string_lossy().to_string(),
                };
                if !caller.data_mut().custody_allows(CustodyOp::FsRead, Some(&resolved_str)) {
                    caller.data_mut().push_refusal_trace("Read", "read_text", "FsRead", file, start, end);
                    return 0;
                }
                // The scope, by the interpreter's own `resolve_in_scope` and in its words: a hostile spelling, a `..` or a
                // link out of the scope (C84), or a link whose target does not exist (SYMLINK-DANGLE-1) is a hard DL0904
                // refusal, not an `Err`. A copy of the rule here lost the last one's explanation (on-demand run 5, F1).
                if let Err(fault) =
                    delulu_runtime::prim::resolve_in_anchor(&scope, &anchor, &rel, delulu_diag::Span::new(0, 0, 0))
                {
                    caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                    caller.data_mut().push_refusal_trace("Read", "read_text", "FsRead", file, start, end);
                    return 0;
                }
                // Read host-side, then construct the Result[Str, IoErr] cell in guest memory (matching the
                // interpreter's io-error mapping: NotFound / PermissionDenied / Other(message)).
                // FS-RACE-1: opened through `beneath`, as the interpreter opens it — no link that
                // appeared after the check above is followed, and only a regular file is read.
                // RW 4.72: opened at a path pinned inside the anchor — the one asked about, or pinned now as the
                // interpreter pins it; a use that no longer resolves inside the anchor is refused.
                let read = match pin.or_else(|| delulu_runtime::prim::pin_fs_at(&scope, &anchor, &rel)) {
                    Some(real) => delulu_runtime::beneath::read_text_at(&real),
                    None => delulu_runtime::beneath::Outcome::Escaped(delulu_runtime::beneath::OUTSIDE),
                };
                let built = match read {
                    delulu_runtime::beneath::Outcome::Done(content) => build_ok_str(&mut caller, &content),
                    delulu_runtime::beneath::Outcome::Io(e) => {
                        use std::io::ErrorKind::*;
                        match e.kind() {
                            NotFound => build_err_ioerr(&mut caller, 0, None),
                            PermissionDenied => build_err_ioerr(&mut caller, 1, None),
                            _ => build_err_ioerr(&mut caller, 2, Some(&e.to_string())),
                        }
                    }
                    delulu_runtime::beneath::Outcome::Escaped(why) => {
                        caller.data_mut().refused = Some(format!("DL0904: path `{rel}` escapes the granted scope — {why}"));
                        // As the interpreter records an effect's own refusal after its dispatch record (RW 4.62).
                        caller.data_mut().push_refusal_trace("Read", "read_text", "FsRead", file, start, end);
                        return 0;
                    }
                };
                match built {
                    Ok(ptr) => ptr,
                    Err(()) => {
                        caller.data_mut().refused = Some("guest heap exhausted building the fs result".into());
                        0
                    }
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // `root.http(hosts)`: minted by the interpreter's own `call_root_method` over the list rebuilt from its descriptor — each
    // requested entry within a granted one (D-V2-139), the `net.special=` subset carried, and its DL0703 in its words.
    // Pure, as on the interpreter: no trace, no custody round-trip.
    linker
        .func_wrap("delulu:cap", "root_http", |mut caller: Caller<'_, HostState>, root: i32, hosts: i32, desc: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            let Some(hosts) = reify_values(&mut caller, desc, &[hosts]) else { return -1 };
            let st = caller.data();
            let root = delulu_runtime::RootVal {
                net: st.net.clone(),
                net_special: st.net_special.clone(),
                lease: st.lease.clone(),
                ..Default::default()
            };
            match delulu_runtime::prim::call_root_method(&root, "http", &hosts, delulu_diag::Span::new(0, 0, 0)) {
                Ok(Value::Cap(c)) => match &c.scope {
                    delulu_runtime::value::CapScope::Net { allow, special } => {
                        let st = caller.data_mut();
                        st.caps.push(CapKind::Http(allow.clone(), special.clone()));
                        (st.caps.len() - 1) as i32
                    }
                    _ => unheld(&mut caller, "root.http", &Value::Cap(c.clone()), -1),
                },
                Ok(other) => unheld(&mut caller, "root.http", &other, -1),
                Err(fault) => {
                    caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                    -1
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `Cap[Http].get(url)`: the interpreter's order (`Interp::call_method`) — the record at dispatch, the URL only
    // (D-V2-128); the custody, on the URL's host, unless the URL is not `https://`, which the effect itself answers
    // `Refused` before anything is reached (HTTP-SCHEME-1), so there is no use to authorize; then the interpreter's own
    // `get` — the allowlist, its egress client (PS-B-02) and all. An egress refusal after the custody allowed the host is
    // said to the custody (RW 4.46).
    linker
        .func_wrap(
            "delulu:cap",
            "http_get",
            |mut caller: Caller<'_, HostState>, cap: i32, url_ptr: i32, file: i32, start: i32, end: i32| -> i32 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let (allow, special) = match caller.data().caps.get(cap as usize) {
                    Some(CapKind::Http(allow, special)) => (allow.clone(), special.clone()),
                    _ => {
                        caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Http capability"));
                        return 0;
                    }
                };
                let Some(url) = read_guest_str(&mut caller, url_ptr) else {
                    caller.data_mut().refused = Some("DL0903: http get URL pointer is out of bounds".into());
                    return 0;
                };
                caller.data_mut().push_trace("Net", "get", "Http", Some(url.clone()), file, start, end);
                let host = url.starts_with("https://").then(|| delulu_runtime::prim::host_of(&url).to_string());
                if let Some(h) = &host {
                    let url_path = delulu_runtime::prim::request_path(&url);
                    if !caller.data_mut().custody_allows_at(CustodyOp::Net, Some(h), Some(url_path)) {
                        caller.data_mut().push_refusal_trace("Net", "get", "Http", file, start, end);
                        return 0;
                    }
                    delulu_runtime::egress::begin_fetch();
                }
                let capv = delulu_runtime::value::CapVal {
                    kind: delulu_check::ResourceKind::Http,
                    scope: delulu_runtime::value::CapScope::Net { allow, special },
                };
                let span = delulu_diag::Span::new(file as u32, start as u32, end as u32);
                let answer = delulu_runtime::prim::call_cap_method(&capv, "get", &[Value::str(url.clone())], span);
                if let Some(h) = &host {
                    if let Some(reason) = delulu_runtime::egress::fetch_refusal() {
                        if let Some(custody) = &caller.data().custody {
                            custody.borrow_mut().note_egress_refusal(h, reason);
                        }
                    }
                }
                let answer = match answer {
                    Ok(v) => v,
                    Err(fault) => {
                        caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                        caller.data_mut().push_refusal_trace("Net", "get", "Http", file, start, end);
                        return 0;
                    }
                };
                match build_net_answer(&mut caller, &answer) {
                    Some(Ok(ptr)) => ptr,
                    Some(Err(())) => {
                        caller.data_mut().refused = Some("guest heap exhausted building the http result".into());
                        0
                    }
                    None => unheld(&mut caller, "http.get", &answer, 0),
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // `Cap[Http].post(url, headers, body)` (P9-05 (d), D-V2-183), as `http_get` is: the custody on the URL's host, path AND
    // method (POST), unless the URL is not `https://`; then the interpreter's own `post` — the allowlist by method, the
    // egress client with its redirect rules and all. An egress refusal after the custody allowed it is said to the custody.
    linker
        .func_wrap(
            "delulu:cap",
            "http_post",
            |mut caller: Caller<'_, HostState>,
             cap: i32,
             url_ptr: i32,
             headers: i32,
             desc: i32,
             body_ptr: i32,
             file: i32,
             start: i32,
             end: i32|
             -> i32 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let (allow, special) = match caller.data().caps.get(cap as usize) {
                    Some(CapKind::Http(allow, special)) => (allow.clone(), special.clone()),
                    _ => {
                        caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Http capability"));
                        return 0;
                    }
                };
                let Some(url) = read_guest_str(&mut caller, url_ptr) else {
                    caller.data_mut().refused = Some("DL0903: http post URL pointer is out of bounds".into());
                    return 0;
                };
                let Some(body) = read_guest_str(&mut caller, body_ptr) else {
                    caller.data_mut().refused = Some("DL0903: a request body lies outside the guest's memory".into());
                    return 0;
                };
                let Some(mut hs) = reify_values(&mut caller, desc, &[headers]) else { return 0 };
                let headers = hs.pop().unwrap_or(Value::Unit);
                caller.data_mut().push_trace("Net", "post", "Http", Some(url.clone()), file, start, end);
                let host = url.starts_with("https://").then(|| delulu_runtime::prim::host_of(&url).to_string());
                if let Some(h) = &host {
                    let url_path = delulu_runtime::prim::request_path(&url);
                    if !caller.data_mut().custody_allows_by(CustodyOp::Net, Some(h), Some(url_path), Some("POST")) {
                        caller.data_mut().push_refusal_trace("Net", "post", "Http", file, start, end);
                        return 0;
                    }
                    delulu_runtime::egress::begin_fetch();
                }
                let capv = delulu_runtime::value::CapVal {
                    kind: delulu_check::ResourceKind::Http,
                    scope: delulu_runtime::value::CapScope::Net { allow, special },
                };
                let span = delulu_diag::Span::new(file as u32, start as u32, end as u32);
                let args = [Value::str(url.clone()), headers, Value::str(body)];
                let answer = delulu_runtime::prim::call_cap_method(&capv, "post", &args, span);
                if let Some(h) = &host {
                    if let Some(reason) = delulu_runtime::egress::fetch_refusal() {
                        if let Some(custody) = &caller.data().custody {
                            custody.borrow_mut().note_egress_refusal(h, reason);
                        }
                    }
                }
                let answer = match answer {
                    Ok(v) => v,
                    Err(fault) => {
                        caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                        caller.data_mut().push_refusal_trace("Net", "post", "Http", file, start, end);
                        return 0;
                    }
                };
                match build_net_answer(&mut caller, &answer) {
                    Some(Ok(ptr)) => ptr,
                    Some(Err(())) => {
                        caller.data_mut().refused = Some("guest heap exhausted building the http result".into());
                        0
                    }
                    None => unheld(&mut caller, "http.post", &answer, 0),
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // `root.secret(name)` (D-V2-174): minted by the interpreter's own `call_root_method` against the run's secret grants —
    // the bytes, or under the broker daemon its handle — and kept in the cap table; the guest is given the entry's index,
    // so no byte of it enters the guest's memory. Its DL0703 in its words. Pure, as on the interpreter: no trace.
    linker
        .func_wrap("delulu:cap", "root_secret", |mut caller: Caller<'_, HostState>, root: i32, name_ptr: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            let Some(name) = read_guest_str(&mut caller, name_ptr) else {
                caller.data_mut().refused = Some("DL0903: a secret's name lies outside the guest's memory".into());
                return -1;
            };
            let st = caller.data();
            let root = delulu_runtime::RootVal {
                secrets: st.secrets.values.clone(),
                secret_audiences: st.secrets.audiences.clone(),
                broker_secrets: st.secrets.broker.clone(),
                lease: st.lease.clone(),
                ..Default::default()
            };
            match delulu_runtime::prim::call_root_method(&root, "secret", &[Value::str(name)], delulu_diag::Span::new(0, 0, 0)) {
                Ok(Value::Secret(s)) => {
                    let st = caller.data_mut();
                    st.caps.push(CapKind::Secret(s));
                    (st.caps.len() - 1) as i32
                }
                Ok(other) => unheld(&mut caller, "root.secret", &other, -1),
                Err(fault) => {
                    caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                    -1
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `Secret.verify(other)` (D-V2-174): the interpreter's order — the record at dispatch, `Declassify` with detail
    // `«opaque»` — and no custody op, as there; a `.dwx` that does not declare `Declassify` is refused (ARTIFACT-CLAIM-1).
    // Then the interpreter's own `secret_verify`: compared here, or by the broker when it holds both.
    linker
        .func_wrap(
            "delulu:cap",
            "secret_verify",
            |mut caller: Caller<'_, HostState>, a: i32, b: i32, file: i32, start: i32, end: i32| -> i32 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let held = |caller: &Caller<'_, HostState>, h: i32| match caller.data().caps.get(h as usize) {
                    Some(CapKind::Secret(s)) => Some(s.clone()),
                    _ => None,
                };
                let (Some(sa), Some(sb)) = (held(&caller, a), held(&caller, b)) else {
                    caller.data_mut().refused = Some(format!("DL0904: handle {a} or {b} is not a secret"));
                    return 0;
                };
                let detail = Some(delulu_runtime::trace::OPAQUE.to_string());
                caller.data_mut().push_trace("Declassify", "verify", "Secret", detail, file, start, end);
                if !caller.data_mut().declared_allows("Declassify") {
                    caller.data_mut().push_refusal_trace("Declassify", "verify", "Secret", file, start, end);
                    return 0;
                }
                let span = delulu_diag::Span::new(file as u32, start as u32, end as u32);
                let args = [Value::Secret(sb)];
                let answer = match &caller.data().custody {
                    Some(custody) => delulu_runtime::interp::secret_verify(&mut **custody.borrow_mut(), &sa, &args, span),
                    None => delulu_runtime::interp::secret_verify(&mut delulu_runtime::EmbeddedCustody::new(), &sa, &args, span),
                };
                match answer {
                    Ok(Value::Bool(equal)) => i32::from(equal),
                    Ok(other) => unheld(&mut caller, "Secret.verify", &other, 0),
                    Err(fault) => {
                        caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                        caller.data_mut().push_refusal_trace("Declassify", "verify", "Secret", file, start, end);
                        0
                    }
                }
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `Cap[Http].get_with_secret(url, headers, header, secret)` (D-V2-174): `get`'s order — the record at dispatch, its
    // detail `«opaque»` as every secret-involving record's is; the custody on the URL's host, unless the URL is not
    // `https://`; then the secret judged and, if the broker holds it, its bytes asked for the one origin by the
    // interpreter's own `broker_secret_send`; then the interpreter's own `get_with_secret`, its egress client and all. The
    // bytes go from the cap table to the egress client; the guest names the entry and is given the answer.
    linker
        .func_wrap(
            "delulu:cap",
            "http_get_with_secret",
            |mut caller: Caller<'_, HostState>,
             cap: i32,
             url_ptr: i32,
             headers: i32,
             desc: i32,
             header_ptr: i32,
             secret: i32,
             file: i32,
             start: i32,
             end: i32|
             -> i32 {
                send_with_secret(
                    &mut caller,
                    "get_with_secret",
                    cap,
                    url_ptr,
                    headers,
                    desc,
                    header_ptr,
                    secret,
                    None,
                    file,
                    start,
                    end,
                )
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `Cap[Http].post_with_secret(url, headers, header, secret, body)` (P9-05 (d), D-V2-183): `get_with_secret`'s order, by
    // POST, with the body.
    linker
        .func_wrap(
            "delulu:cap",
            "http_post_with_secret",
            |mut caller: Caller<'_, HostState>,
             cap: i32,
             url_ptr: i32,
             headers: i32,
             desc: i32,
             header_ptr: i32,
             secret: i32,
             body_ptr: i32,
             file: i32,
             start: i32,
             end: i32|
             -> i32 {
                send_with_secret(
                    &mut caller,
                    "post_with_secret",
                    cap,
                    url_ptr,
                    headers,
                    desc,
                    header_ptr,
                    secret,
                    Some(body_ptr),
                    file,
                    start,
                    end,
                )
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // `Cap[FsRead].narrow(sub)`: a new handle for the subtree `sub` names inside the handle's scope, by the interpreter's own
    // `resolve_in_scope` — a hostile spelling, a `..` or a link out of the scope is refused (DL0904) as it is there, and no
    // narrowing reaches past the scope it starts from. Pure, as on the interpreter: no trace, no custody round-trip.
    linker
        .func_wrap("delulu:cap", "fs_narrow", |mut caller: Caller<'_, HostState>, cap: i32, path_ptr: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let (scope, anchor) = match caller.data().caps.get(cap as usize) {
                Some(CapKind::FsRead(scope, anchor)) => (scope.clone(), anchor.clone()),
                _ => {
                    caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted FsRead capability"));
                    return -1;
                }
            };
            let Some(sub) = read_guest_str(&mut caller, path_ptr) else {
                caller.data_mut().refused = Some("DL0903: narrow path pointer is out of bounds".into());
                return -1;
            };
            match delulu_runtime::prim::resolve_in_anchor(&scope, &anchor, &sub, delulu_diag::Span::new(0, 0, 0)) {
                Ok((narrowed, anchor)) => {
                    let st = caller.data_mut();
                    st.caps.push(CapKind::FsRead(narrowed, anchor));
                    (st.caps.len() - 1) as i32
                }
                Err(fault) => {
                    caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                    -1
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // `root.fs_write(p)`: a write capability minted exactly as the interpreter mints one (`prim`'s root method): a hostile
    // spelling refused, `p` resolved from the working directory, granted iff it lies on disk within a granted subtree.
    linker
        .func_wrap("delulu:cap", "root_fs_write", |mut caller: Caller<'_, HostState>, root: i32, path_ptr: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1;
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            let Some(path) = read_guest_str(&mut caller, path_ptr) else {
                caller.data_mut().refused = Some("DL0903: fs_write path pointer is out of bounds".into());
                return -1;
            };
            if let Some(why) = delulu_runtime::prim::hostile_path(&path, false) {
                caller.data_mut().refused = Some(format!("DL0904: path `{}` is refused: {why}", path.escape_debug()));
                return -1;
            }
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&path));
            let Some(anchor) = delulu_runtime::prim::mint_anchor(&caller.data().fs_write_roots, &want) else {
                let remedy = delulu_runtime::prim::grant_remedy(caller.data().lease.as_deref(), &format!("fs.write={path}"));
                caller.data_mut().refused = Some(format!("DL0703: filesystem write of `{path}` was not granted — {remedy}"));
                return -1;
            };
            let st = caller.data_mut();
            st.caps.push(CapKind::FsWrite(want, anchor));
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `write_text(rel, body)` / `append_text` (`append` 1): the interpreter's order, step for step — the record, the custody
    // gate on the path the write will open (GUARD-ALIAS-1), the scope (`resolve_in_scope`, its words), then the write
    // through `beneath` (FS-RACE-1), answered as `Result[Unit, IoErr]`. A refusal after the record is recorded as one.
    linker
        .func_wrap(
            "delulu:cap",
            "fs_write_text",
            |mut caller: Caller<'_, HostState>,
             cap: i32,
             path_ptr: i32,
             body_ptr: i32,
             append: i32,
             file: i32,
             start: i32,
             end: i32|
             -> i32 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let op = if append != 0 { "append_text" } else { "write_text" };
                let (scope, anchor) = match caller.data().caps.get(cap as usize) {
                    Some(CapKind::FsWrite(scope, anchor)) => (scope.clone(), anchor.clone()),
                    _ => {
                        caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted FsWrite capability"));
                        return 0;
                    }
                };
                let (Some(rel), Some(body)) = (read_guest_str(&mut caller, path_ptr), read_guest_str(&mut caller, body_ptr))
                else {
                    caller.data_mut().refused = Some(format!("DL0903: a `{op}` argument lies outside the guest's memory"));
                    return 0;
                };
                let pointed = delulu_runtime::prim::resolve_norm(&scope, &rel);
                caller.data_mut().push_fs_trace(("Write", op, "FsWrite"), &rel, &pointed, &scope, file, start, end);
                let pin = delulu_runtime::prim::pin_fs_at(&scope, &anchor, &rel);
                let at = match &pin {
                    Some(real) => delulu_runtime::prim::disk_spelling(real),
                    None => pointed.to_string_lossy().to_string(),
                };
                if !caller.data_mut().custody_allows(CustodyOp::FsWrite, Some(&at)) {
                    caller.data_mut().push_refusal_trace("Write", op, "FsWrite", file, start, end);
                    return 0;
                }
                if let Err(fault) =
                    delulu_runtime::prim::resolve_in_anchor(&scope, &anchor, &rel, delulu_diag::Span::new(0, 0, 0))
                {
                    caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                    caller.data_mut().push_refusal_trace("Write", op, "FsWrite", file, start, end);
                    return 0;
                }
                // RW 4.72: as the read, at a path pinned inside the anchor.
                let wrote = match pin.or_else(|| delulu_runtime::prim::pin_fs_at(&scope, &anchor, &rel)) {
                    Some(real) => delulu_runtime::beneath::write_text_at(&real, &body, append != 0),
                    None => delulu_runtime::beneath::Outcome::Escaped(delulu_runtime::beneath::OUTSIDE),
                };
                let built = match wrote {
                    delulu_runtime::beneath::Outcome::Done(()) => write_variant_cell(&mut caller, 0, None), // Ok(())
                    delulu_runtime::beneath::Outcome::Io(e) => {
                        use std::io::ErrorKind::*;
                        match e.kind() {
                            NotFound => build_err_ioerr(&mut caller, 0, None),
                            PermissionDenied => build_err_ioerr(&mut caller, 1, None),
                            _ => build_err_ioerr(&mut caller, 2, Some(&e.to_string())),
                        }
                    }
                    delulu_runtime::beneath::Outcome::Escaped(why) => {
                        caller.data_mut().refused = Some(format!("DL0904: path `{rel}` escapes the granted scope — {why}"));
                        caller.data_mut().push_refusal_trace("Write", op, "FsWrite", file, start, end);
                        return 0;
                    }
                };
                match built {
                    Ok(ptr) => ptr,
                    Err(()) => {
                        caller.data_mut().refused = Some("guest heap exhausted building the fs result".into());
                        0
                    }
                }
            },
        )
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
                caller.data_mut().refused =
                    Some("DL0703: foreign loading was not granted (grant a `foreign.c` lib or `foreign.python`)".into());
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
                        Err(e) => {
                            build_foreign_err_cell(&mut caller, &e).and_then(|fe| write_variant_cell(&mut caller, 1, Some(fe)))
                        }
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
            |mut caller: Caller<'_, HostState>,
             lib: i32,
             method_ptr: i32,
             args_ptr: i32,
             argc: i32,
             file: i32,
             start: i32,
             end: i32|
             -> i64 {
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
            float_to_guest_str(&mut caller, x)
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // ----- RW 3.4: the `delulu:list` host interface (pure: neither traced nor gated) ---------------------------------
    // `Str.split` by the interpreter's own rule (`prim.rs`): the parts between occurrences of the separator, and one part
    // per character for an empty one — written into the guest's memory as a `List[Str]`.
    linker
        .func_wrap("delulu:list", "str_split", |mut caller: Caller<'_, HostState>, s: i32, sep: i32| -> i32 {
            if caller.data().refused.is_some() {
                return 0;
            }
            let (Some(s), Some(sep)) = (read_guest_str(&mut caller, s), read_guest_str(&mut caller, sep)) else {
                caller.data_mut().refused = Some("DL0903: a string given to `split` lies outside the guest's memory".into());
                return 0;
            };
            let parts: Vec<String> = if sep.is_empty() {
                s.chars().map(|c| c.to_string()).collect()
            } else {
                s.split(sep.as_str()).map(str::to_string).collect()
            };
            match write_str_list(&mut caller, &parts) {
                Ok(list) => list,
                Err(()) => {
                    caller.data_mut().refused = Some(MEMORY_EXHAUSTED.into());
                    0
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // An index outside a list: the guest calls this and then traps, and the run reports what is recorded here — the
    // interpreter's own fault (`interp.rs`'s `index`), word for word.
    linker
        .func_wrap("delulu:list", "index_fault", |mut caller: Caller<'_, HostState>, index: i64, len: i32| {
            caller.data_mut().refused.get_or_insert_with(|| format!("DL0903: index {index} out of bounds (len {len})"));
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // A `match` testing inside a constructor that no arm matched (`codegen.rs`'s `compile_match_ordered`): the guest calls
    // this and then traps, and the run reports the interpreter's own fault (`interp.rs`'s `eval_match`), word for word.
    linker
        .func_wrap("delulu:match", "no_arm", |mut caller: Caller<'_, HostState>| {
            caller
                .data_mut()
                .refused
                .get_or_insert_with(|| "DL0907: no match arm applied (checker guarantees exhaustiveness)".to_string());
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    link_maps(&mut linker)?;
    link_list_host(&mut linker)?;

    // ----- RW 3.4: the `delulu:str` host interface (pure: neither traced nor gated) ----------------------------------
    // A `Str` method (`codegen.rs`'s `STR_METHODS`) run by the interpreter's own `call_str_method`, so the WASM engine
    // answers exactly what the interpreter answers — Unicode case mappings, character-indexed `slice`, `replace` of an
    // empty pattern — rather than a second implementation that could drift. Its answer is written back as the guest
    // reads it: a `Str` as a string cell, a `Bool` as 0 or 1, a `List[Str]` as a list.
    linker
        .func_wrap(
            "delulu:str",
            "str_method",
            |mut caller: Caller<'_, HostState>, s: i32, method: i32, a: i32, b: i32, lo: i64, hi: i64| -> i32 {
                if caller.data().refused.is_some() {
                    return 0;
                }
                let Some((name, strs, ints)) = crate::codegen::str_method_arity(method) else {
                    caller.data_mut().refused = Some(format!("DL0907: no `Str` method numbered {method}"));
                    return 0;
                };
                let recv = read_guest_str(&mut caller, s);
                let given: Option<Vec<String>> = [a, b].into_iter().take(strs).map(|p| read_guest_str(&mut caller, p)).collect();
                let (Some(recv), Some(given)) = (recv, given) else {
                    caller.data_mut().refused =
                        Some(format!("DL0903: a string given to `{name}` lies outside the guest's memory"));
                    return 0;
                };
                let mut args: Vec<Value> = given.into_iter().map(Value::str).collect();
                args.extend([lo, hi].into_iter().take(ints).map(Value::Int));
                let answer = match delulu_runtime::prim::call_str_method(&recv, name, &args, delulu_diag::Span::new(0, 0, 0)) {
                    Ok(v) => v,
                    Err(fault) => {
                        caller.data_mut().refused = Some(format!("{}: {}", fault.code, fault.message));
                        return 0;
                    }
                };
                let parts: Option<Vec<String>> = match &answer {
                    Value::List(xs) => xs
                        .borrow()
                        .iter()
                        .map(|v| match v {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        })
                        .collect(),
                    _ => None,
                };
                let written = match (answer, parts) {
                    (Value::Bool(yes), _) => return i32::from(yes),
                    (Value::Str(t), _) => write_str_cell(&mut caller, &t),
                    (Value::List(_), Some(parts)) => write_str_list(&mut caller, &parts),
                    _ => {
                        caller.data_mut().refused =
                            Some(format!("DL0907: `{name}` answered a value the WASM engine does not hold"));
                        return 0;
                    }
                };
                written.unwrap_or_else(|()| {
                    caller.data_mut().refused = Some(MEMORY_EXHAUSTED.into());
                    0
                })
            },
        )
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `parse_int(s)` by the interpreter's own rule (`call_builtin`): whether it parsed, and the number — the guest builds
    // the `Option[Int]` from the two.
    linker
        .func_wrap("delulu:str", "parse_int", |mut caller: Caller<'_, HostState>, s: i32| -> (i32, i64) {
            match parse_by_builtin(&mut caller, "parse_int", s) {
                Some(Some(Value::Int(n))) => (1, n),
                Some(None) => (0, 0),
                Some(Some(other)) => unheld(&mut caller, "parse_int", &other, (0, 0)),
                None => (0, 0),
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `parse_float(s)` the same way; the interpreter's rule answers `None` for text a `Float` literal may not be.
    linker
        .func_wrap("delulu:str", "parse_float", |mut caller: Caller<'_, HostState>, s: i32| -> (i32, f64) {
            match parse_by_builtin(&mut caller, "parse_float", s) {
                Some(Some(Value::Float(x))) => (1, x),
                Some(None) => (0, 0.0),
                Some(Some(other)) => unheld(&mut caller, "parse_float", &other, (0, 0.0)),
                None => (0, 0.0),
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;

    // ----- RW 3.4: the `delulu:float` host interface (pure: neither traced nor gated) --------------------------------
    // `str` of a `Float`, by the interpreter's own display, for a module without the foreign interface (which brings its
    // own `float_to_str`); and `Float % Float`, which WebAssembly has no instruction for — Rust's `%`, as the interpreter's.
    linker
        .func_wrap("delulu:float", "to_str", |mut caller: Caller<'_, HostState>, x: f64| -> i32 {
            float_to_guest_str(&mut caller, x)
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:float", "rem", |_caller: Caller<'_, HostState>, a: f64, b: f64| -> f64 { a % b })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    // `str` and `==` of a list, an option, a result, a sum or a record: the value rebuilt from the guest's memory by its
    // type's descriptor (`reify.rs`), then the interpreter's own `Value::display` and `Value::eq`.
    linker
        .func_wrap("delulu:value", "display", |mut caller: Caller<'_, HostState>, v: i32, desc: i32| -> i32 {
            let Some(shown) = reify_values(&mut caller, desc, &[v]).map(|vs| vs[0].display()) else {
                return 0;
            };
            match write_str_cell(&mut caller, &shown) {
                Ok(ptr) => ptr,
                Err(()) => {
                    caller.data_mut().refused = Some("guest heap exhausted rendering a value".into());
                    0
                }
            }
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:value", "eq", |mut caller: Caller<'_, HostState>, a: i32, b: i32, desc: i32| -> i32 {
            reify_values(&mut caller, desc, &[a, b]).map_or(0, |vs| i32::from(vs[0].eq(&vs[1])))
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
        .func_wrap("delulu:actors", "actor_field_get", |mut caller: Caller<'_, HostState>, slot: i64, field_idx: i32| -> i64 {
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
        })
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

/// The error of a call that trapped: the host's recorded refusal if there is one, else the trap. A refused host call
/// returns a placeholder and the guest runs on, so what traps next is a consequence of the refusal, not the cause the
/// interpreter reports (WASM-REFUSAL-1: a refused `fs.read_text` matched on its error read a placeholder pointer and
/// reported DL0903, an access out of bounds, where the interpreter reported DL0904).
fn trap_or_refusal(store: &Store<HostState>, e: &wasmtime::Error) -> WasmError {
    WasmError::Trap(store.data().refused.clone().unwrap_or_else(|| clean_trap(e)))
}

fn finish(mut store: Store<HostState>, func: wasmtime::Func, params: &[Val]) -> Result<String, WasmError> {
    let mut results: [Val; 0] = [];
    func.call(&mut store, params, &mut results).map_err(|e| trap_or_refusal(&store, &e))?;
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
        fs_write_roots: Vec::new(),
        lease: None,
        net: Vec::new(),
        net_special: Vec::new(),
        secrets: SecretGrants::default(),
        declared_effects: None,
        fixed_clock_ms: None,
        rng: seed_rng(None),
        output: String::new(),
        stream_console: false,
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
        fs_write_roots: cfg.fs_write_roots.clone(),
        lease: cfg.lease.clone(),
        net: cfg.net.clone(),
        net_special: cfg.net_special.clone(),
        secrets: cfg.secrets.clone(),
        declared_effects: cfg.declared_effects.clone(),
        fixed_clock_ms: cfg.fixed_clock_ms,
        rng: seed_rng(cfg.rand_seed),
        output: String::new(),
        stream_console: cfg.stream_console,
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
    /// Granted filesystem-write subtrees, the same way (`root.fs_write(p)`).
    pub fs_write_roots: Vec<PathBuf>,
    /// RW 6.15: the delegated node a `--lease` run's grants came from (`Grants::lease`), so a refusal names the remedy the
    /// interpreter's would. Not authority.
    pub lease: Option<String>,
    /// The granted network entries (`--grant net=…`, `net.special=…` included) and the subset granted as `net.special=`
    /// — `Grants::net` and `net_special`. `root.http(hosts)` mints by the interpreter's rule against them.
    pub net: Vec<String>,
    pub net_special: Vec<String>,
    /// The run's secret grants, which `root.secret(name)` mints against (D-V2-174).
    pub secrets: SecretGrants,
    /// ARTIFACT-CLAIM-1: for a `.dwx` run, the effects its manifest declares — a ceiling: an effect outside it is refused
    /// (DL1202) before the custody is asked. `None` for a program compiled here, whose row the checker computed.
    pub declared_effects: Option<std::collections::BTreeSet<String>>,
    /// `Some(ms)` fixes `Cap[Clock].now_ms()` for deterministic replay (spec §6.2); `None` = wall clock.
    pub fixed_clock_ms: Option<i64>,
    /// `Some(seed)` seeds `Cap[Rand]` deterministically (spec §6.2); `None` = a nondeterministic seed.
    pub rand_seed: Option<u64>,
    /// `true`: each `println` goes to stdout as it is printed, as the interpreter's does, and the run returns an
    /// empty string. `false` (the default): the output is captured and returned. The CLI streams, so what a program
    /// printed before a fault, a refusal or a budget's stop is on stdout on both engines (WASM-OUTPUT-1: the CLI
    /// printed the captured output only when the run succeeded).
    pub stream_console: bool,
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
pub fn run_main_actors(wasm: &[u8], cfg: &HostConfig, table: &ActorTable) -> Result<(String, ActorReport), WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = main_host_state(cfg, ActorRuntime::new(table.clone()));
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    run_actors_in(store, instance)
}

/// Run a `.dwx` (RW 3.21): its `main` — under the cooperative scheduler when the module exports actor members, by the actor
/// table the module itself carries (`actors::table_from_module`, then each actor's field count from its global once
/// instantiated, before `main` runs). Compiled once. The output, and an actor program's quiescence report.
pub fn run_artifact(wasm: &[u8], cfg: &HostConfig) -> Result<(String, Option<ActorReport>), WasmError> {
    let engine = optimizing_engine();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let table = crate::actors::table_from_module(&module).map_err(WasmError::Module)?;
    let state = main_host_state(cfg, ActorRuntime::new(table.clone().unwrap_or_else(empty_actor_table)));
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let Some(table) = table else {
        let func = instance.get_func(&mut store, "main").ok_or_else(|| WasmError::NoExport("main".to_string()))?;
        return finish(store, func, &[Val::I32(0)]).map(|out| (out, None));
    };
    let counts = crate::actors::field_counts(&table, |name| instance.get_global(&mut store, name).map(|g| g.get(&mut store)))
        .map_err(WasmError::Module)?;
    store.data_mut().actor_rt.set_field_counts(&counts);
    run_actors_in(store, instance).map(|(out, report)| (out, Some(report)))
}

/// `main`'s turn, then the cooperative drain to quiescence (Stage 7 phase 7h) — for a module instantiated with its actor table.
fn run_actors_in(mut store: Store<HostState>, instance: Instance) -> Result<(String, ActorReport), WasmError> {
    let main = instance.get_func(&mut store, "main").ok_or_else(|| WasmError::NoExport("main".to_string()))?;
    // `main`'s own turn: a trap here fails the whole run (it is not an actor turn — no poison).
    let mut results: [Val; 0] = [];
    main.call(&mut store, &[Val::I32(0)], &mut results).map_err(|e| trap_or_refusal(&store, &e))?;
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

#[cfg(test)]
mod feature_hardening_tests {
    use super::*;
    use wasm_encoder::{CodeSection, Function, FunctionSection, Instruction, Module as WasmModule, TypeSection};

    /// A minimal but VALID module whose body uses `f64x2.splat` — the exact instruction named by
    /// RUSTSEC-2026-0087 (Cranelift x86-64: segfault or out-of-sandbox load).
    fn simd_module() -> Vec<u8> {
        let mut types = TypeSection::new();
        types.ty().function([], []);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        let mut code = CodeSection::new();
        let mut f = Function::new([]);
        f.instruction(&Instruction::F64Const(1.0));
        f.instruction(&Instruction::F64x2Splat);
        f.instruction(&Instruction::Drop);
        f.instruction(&Instruction::End);
        code.function(&f);
        let mut m = WasmModule::new();
        m.section(&types);
        m.section(&funcs);
        m.section(&code);
        m.finish()
    }

    /// The hardening must actually take effect. `optimizing_engine` ends in
    /// `Engine::new(&config).unwrap_or_default()`, so a config wasmtime rejected would silently
    /// fall back to the DEFAULT engine — which still accepts SIMD. Without this test, the
    /// narrowing could be a no-op and nothing would say so.
    #[test]
    fn hardened_engine_refuses_a_simd_module() {
        let wasm = simd_module();

        // Control FIRST: a stock engine accepts it. This proves the module is well-formed and that
        // any refusal below comes from our configuration, not from a malformed test fixture.
        let stock = Engine::default();
        assert!(
            wasmtime::Module::new(&stock, &wasm).is_ok(),
            "the fixture must be VALID wasm on a default engine, otherwise the refusal below \
             proves nothing about the hardening"
        );

        // The real assertion. A `match` rather than `expect_err`, because `wasmtime::Module` is not
        // `Debug` and `expect_err` requires it.
        let hardened = optimizing_engine();
        let err = match wasmtime::Module::new(&hardened, &wasm) {
            Ok(_) => panic!(
                "the hardened engine must REFUSE a SIMD module — if this passes, \
                 harden_wasm_features was silently dropped by unwrap_or_default()"
            ),
            Err(e) => e,
        };
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            msg.contains("simd") || msg.contains("not enabled") || msg.contains("disabled"),
            "expected a feature-disabled validation error, got: {err:#}"
        );
    }

    /// RW 4.33 (routine run 4, D-V2-68): the WebAssembly 3.0 proposals that neither `codegen.rs` nor a
    /// default wasm32 toolchain emits are refused — RUSTSEC-2026-0315 (a plugin's granted fuel dropped
    /// through `call_ref` and an exception `catch`) was reachable through two of them while each was on
    /// by wasmtime's default. Each fixture is enabled ALONE on a control engine first, so a refusal
    /// below is the hardening's and never a malformed fixture's. (Legacy exceptions have no fixture: no
    /// engine here can switch them on — "not supported on this compiler configuration" — so a fixture
    /// could never be proven valid, and no engine this crate builds can run them.)
    const WASM3_ONLY: &[(&str, &str)] = &[
        ("function references", "(module (type $t (func)) (func $f (type $t)) (elem declare func $f) (func (call_ref $t (ref.func $f))))"),
        ("exceptions", "(module (tag $e) (func (block $h (try_table (catch_all $h) (throw $e)))))"),
        ("gc", "(module (type $s (struct (field i32))) (func (result (ref $s)) (struct.new $s (i32.const 1))))"),
        ("tail calls", "(module (func $f) (func (return_call $f)))"),
        ("wide arithmetic", "(module (func (param i64 i64 i64 i64) (result i64 i64) (i64.add128 (local.get 0) (local.get 1) (local.get 2) (local.get 3))))"),
        ("custom page sizes", "(module (memory 1 (pagesize 1)))"),
        ("multi-memory", "(module (memory 1) (memory 1))"),
        ("stack switching", "(module (type $f (func)) (type $c (cont $f)) (func (param (ref null $c)) (drop (local.get 0))))"),
    ];

    /// The control for one fixture: an engine with that proposal (and what it rests on) switched on.
    fn engine_with(proposal: &str) -> Option<Engine> {
        let mut c = Config::new();
        match proposal {
            "function references" => c.wasm_function_references(true),
            "exceptions" => c.wasm_exceptions(true),
            "gc" => c.wasm_function_references(true).wasm_gc(true),
            "tail calls" => c.wasm_tail_call(true),
            "wide arithmetic" => c.wasm_wide_arithmetic(true),
            "custom page sizes" => c.wasm_custom_page_sizes(true),
            "multi-memory" => c.wasm_multi_memory(true),
            "stack switching" => c.wasm_function_references(true).wasm_exceptions(true).wasm_stack_switching(true),
            other => panic!("no control for {other}"),
        };
        // A proposal this platform's engine cannot run at all (stack switching is x86-64 only) is refused
        // there by construction; the fixture is proven valid on the platforms that can.
        Engine::new(&c).ok()
    }

    #[cfg(not(windows))]
    fn plugin_limits() -> crate::limits::Effective {
        crate::limits::Effective { fuel: 1_000_000, mem_bytes: 64 * 1024 * 1024, wall_ms: 5_000 }
    }

    #[test]
    fn both_hardened_engines_refuse_every_wasm3_proposal_codegen_never_emits() {
        let hardened = optimizing_engine();
        let mut proven = 0;
        let mut accepted = Vec::new();
        for (proposal, wat) in WASM3_ONLY {
            if let Some(control) = engine_with(proposal) {
                if let Err(e) = wasmtime::Module::new(&control, wat) {
                    panic!("{proposal}: the fixture must be VALID with its proposal on, or its refusal proves nothing: {e:#}");
                }
                proven += 1;
            }
            // The Stage-3 engine (`.dwx` programs).
            if wasmtime::Module::new(&hardened, wat).is_ok() {
                accepted.push(format!("{proposal} (the program engine)"));
            }
            // The plugin store (Contained `.dpx`), which meters granted fuel: refused at compilation, not
            // at the missing export the fixture would have hit next.
            #[cfg(not(windows))]
            match crate::limits::run_contained_export(wat.as_bytes(), "run", plugin_limits()) {
                Err(crate::limits::TrapCause::Unattributable(why)) if why.contains("failed to compile") => {}
                other => accepted.push(format!("{proposal} (the plugin engine: {other:?})")),
            }
        }
        // Stack switching runs only on x86-64 today; there, every fixture is proven valid.
        let expected = if cfg!(all(target_arch = "x86_64", unix)) { WASM3_ONLY.len() } else { WASM3_ONLY.len() - 1 };
        assert!(proven >= expected, "only {proven} of {} fixtures were proven valid", WASM3_ONLY.len());
        assert!(accepted.is_empty(), "the hardened engine still accepts: {accepted:?}");
    }

    /// The plugin store is the engine that meters GRANTED fuel, so the refusal must hold there, end to
    /// end: a Contained module that uses `call_ref` never runs.
    #[cfg(not(windows))]
    #[test]
    fn a_contained_plugin_using_a_wasm3_proposal_never_runs() {
        let calls = "(module (type $t (func (result i64))) (func $f (type $t) (i64.const 7)) (elem declare func $f) \
                     (func (export \"run\") (result i64) (call_ref $t (ref.func $f))))";
        // Control: the same module with a direct call runs and answers 7 — the path works.
        let direct = "(module (func $f (result i64) (i64.const 7)) (func (export \"run\") (result i64) (call $f)))";
        // (wasmtime's `wat` feature reads the text form wherever it reads a module.)
        let ran = crate::limits::run_contained_export(direct.as_bytes(), "run", plugin_limits());
        assert!(matches!(ran, Ok(Some(7))), "the control must run: {ran:?}");
        match crate::limits::run_contained_export(calls.as_bytes(), "run", plugin_limits()) {
            Err(crate::limits::TrapCause::Unattributable(why)) => {
                assert!(why.contains("failed to compile"), "{why}");
            }
            other => panic!("a plugin using call_ref must be refused before it runs, got {other:?}"),
        }
    }

    /// What third-party toolchains emit by default — the WebAssembly 2.0 set: bulk memory, reference
    /// types, multi-value, sign extension, saturating conversions, mutable globals — keeps loading. A
    /// Contained plugin built by an ordinary wasm32 compiler is not refused by the narrowing.
    #[test]
    fn the_webassembly_2_baseline_still_loads_on_the_hardened_engine() {
        let hardened = optimizing_engine();
        for (what, wat) in [
            ("bulk memory", "(module (memory 1) (func (memory.copy (i32.const 0) (i32.const 8) (i32.const 8))))"),
            (
                "reference types",
                "(module (table 1 funcref) (table 1 externref) (func (result funcref) (table.get 0 (i32.const 0))))",
            ),
            ("multi-value", "(module (func (result i32 i64) (i32.const 1) (i64.const 2)))"),
            ("sign extension", "(module (func (param i32) (result i32) (i32.extend8_s (local.get 0))))"),
            ("saturating conversions", "(module (func (param f64) (result i32) (i32.trunc_sat_f64_s (local.get 0))))"),
            ("mutable globals", "(module (global (export \"g\") (mut i32) (i32.const 0)))"),
            ("extended constants", "(module (global i32 (i32.add (i32.const 1) (i32.const 2))))"),
        ] {
            if let Err(e) = wasmtime::Module::new(&hardened, wat) {
                panic!("{what} is what ordinary toolchains emit and must keep loading: {e:#}");
            }
        }
    }

    /// Narrowing may not break what the compiler actually emits. `codegen.rs`'s float instructions
    /// are `F64Const`/`F64Load`/`F64ReinterpretI64`/`F64Store`; a scalar-float module must still
    /// validate on the hardened engine.
    #[test]
    fn hardened_engine_still_accepts_scalar_float_code() {
        let mut types = TypeSection::new();
        types.ty().function([], []);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        let mut code = CodeSection::new();
        let mut f = Function::new([]);
        f.instruction(&Instruction::F64Const(1.5));
        f.instruction(&Instruction::Drop);
        f.instruction(&Instruction::End);
        code.function(&f);
        let mut m = WasmModule::new();
        m.section(&types);
        m.section(&funcs);
        m.section(&code);
        let wasm = m.finish();

        let hardened = optimizing_engine();
        assert!(wasmtime::Module::new(&hardened, &wasm).is_ok(), "scalar f64 is what codegen emits and must keep validating");
    }
}

#[cfg(test)]
mod secret_handle_tests {
    use super::*;

    /// The secret below, and the part of it searched for.
    const SECRET: &str = "s3cr3t-handle-zx8q-never-in-the-guest";
    const CANARY: &str = "zx8q";

    fn holds(bytes: &[u8], needle: &str) -> bool {
        bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
    }

    /// Run `main` of `src` under `cfg`, and give back what it printed (or why it stopped) and the guest's memory once it
    /// has run.
    fn run_keeping_memory(src: &str, cfg: &HostConfig) -> (Result<String, WasmError>, Vec<u8>) {
        let checked = delulu_check::check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let wasm = crate::compile_checked(&checked.module, &checked.result).expect("compiles to WASM");
        assert!(!holds(&wasm, CANARY), "the module was built without the secret");
        let engine = optimizing_engine();
        let module = Module::new(&engine, &wasm).expect("a valid module");
        let mut store = Store::new(&engine, main_host_state(cfg, ActorRuntime::new(empty_actor_table())));
        let instance = build_linker(&engine)
            .and_then(|l| l.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string())));
        let instance = instance.unwrap_or_else(|e| panic!("instantiates: {}", e.message()));
        let main = instance.get_func(&mut store, "main").expect("a main");
        let called = main.call(&mut store, &[Val::I32(0)], &mut []).map_err(|e| trap_or_refusal(&store, &e));
        let memory = instance.get_memory(&mut store, "memory").expect("a memory").data(&store).to_vec();
        let state = store.into_data();
        let out = called.and(match state.refused {
            Some(why) => Err(WasmError::Trap(why)),
            None => Ok(state.output),
        });
        (out, memory)
    }

    /// D-V2-174: a WASM program that mints a secret and sends it holds an index into the host's table and nothing of the
    /// secret — after the run, after the host has put its bytes in a request, no byte of it is in the guest's memory, which
    /// is what DL1205 exists for. The request got past the secret's audience and its headers to the egress client's
    /// transport, and ends there: `no-client` when this crate is built alone (the runtime without `net`), `connect` in a
    /// workspace build, where cargo turns `net` on for every crate and nothing listens on port 9. `egress_cli.rs` sees the
    /// same request leave as TLS from the real binary, on both engines.
    #[test]
    fn a_minted_and_sent_secret_never_enters_the_guests_memory() {
        let url = "https://127.0.0.1:9/probe";
        let src = format!(
            "module m\nfn main(root: Root) ! {{Net, Write}} {{\n let out = root.console()\n let h = root.http([\"127.0.0.1\"])\n \
             let tok = root.secret(\"API_TOKEN\")\n let hs = Map()\n \
             match h.get_with_secret(\"{url}\", hs, \"X-Api-Key\", tok) {{\n \
             Ok(b) => out.println(\"delivered\")\n Err(e) => out.println(\"not delivered\")\n }}\n}}\n"
        );
        let audience = delulu_runtime::egress::Audience { host: "127.0.0.1".into(), port: 9 };
        let cfg = HostConfig {
            console: true,
            net: vec!["127.0.0.1".into()],
            net_special: vec!["127.0.0.1".into()],
            secrets: SecretGrants {
                values: [("API_TOKEN".to_string(), SECRET.to_string())].into_iter().collect(),
                audiences: [("API_TOKEN".to_string(), vec![audience])].into_iter().collect(),
                broker: Vec::new(),
            },
            ..HostConfig::default()
        };
        let (out, memory) = run_keeping_memory(&src, &cfg);
        assert_eq!(out.map_err(|e| e.message()), Ok("not delivered\n".to_string()));
        let log = delulu_runtime::egress::snapshot().to_json();
        let reached = log["records"].as_array().into_iter().flatten().find(|r| r["url"] == url).cloned();
        let reason = reached.as_ref().and_then(|r| r["reason"].as_str());
        assert!(matches!(reason, Some("no-client" | "connect")), "past its audience and headers, to the transport: {log}");
        assert!(!memory.is_empty() && !holds(&memory, CANARY), "no byte of the secret in the guest's memory");
        // The control: the search reads the guest's memory, which holds what the program put there.
        assert!(holds(&memory, "not delivered") && holds(&memory, "X-Api-Key"), "the search reads the guest's memory");
    }
}
