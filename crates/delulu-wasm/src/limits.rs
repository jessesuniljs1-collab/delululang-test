//! Contained plugin execution limits and **honest trap attribution** (Stage 6 §5.1, trap 5).
//!
//! One Wasmtime store per load, carrying the plugin's granted `Limits`: store-level **fuel**, a
//! **memory cap** (a recording `ResourceLimiter`), and a host-side **wall-clock watchdog** (epoch
//! interruption). Exceeding any of them terminates the plugin — instance dropped, node revoked,
//! DL1506 (trap 5: a limit-killed plugin is *gone, not wounded*).
//!
//! # Why attribution is a soundness concern, not a message-quality one
//!
//! A guest can trap for reasons that are **not** limits: `unreachable`, divide-by-zero, an
//! out-of-bounds access — ordinary bugs. DL1506's repair is *"raise limits"*, flagged
//! **`authority_widening: true`**. So mislabelling a bug-trap as DL1506 would make this language
//! advise a host to **widen a plugin's authority** in order to fix a bug that more authority cannot
//! fix — the compiler recommending the wrong direction on the one axis the whole project exists to
//! protect. That is worse than an unhelpful diagnostic.
//!
//! Therefore [`attribute`] decides **by evidence the engine itself produced** — our own limiter's
//! refusal record, the store's remaining fuel, our own watchdog's flag, and Wasmtime's own `Trap`
//! discriminant — and **never** by the inference "it trapped, limits were set, so it must be
//! limits". When the evidence does not attribute the trap, the result is
//! [`TrapCause::Unattributable`]: the plugin is still killed (trap 5 holds unconditionally), but the
//! *diagnosis says it does not know* and **no authority-widening repair is emitted on a guess**.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::{ResourceLimiter, Trap};

use delulu_runtime::Limits;

// The live-execution path is compiled only off Windows (see `run_contained_export`): on Windows
// it would be a code path that can fastfail the host, so it literally does not exist there.
#[cfg(not(windows))]
use std::time::Duration;
#[cfg(not(windows))]
use wasmtime::{Config, Engine, Linker, Module, Store, Val};

/// Default fuel when `limits.fuel == 0` ("broker/profile default" — never "unlimited", spec §4).
pub const DEFAULT_FUEL: u64 = 50_000_000;
/// Default memory cap in MiB when `limits.mem_mb == 0`.
pub const DEFAULT_MEM_MB: usize = 64;
/// Default wall-clock budget in ms when `limits.wall_ms == 0`.
pub const DEFAULT_WALL_MS: u64 = 5_000;

/// Why a Contained instance stopped running — **attributed from evidence**, never inferred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrapCause {
    /// Store fuel was exhausted (evidence: Wasmtime's own `Trap::OutOfFuel`, or fuel at zero).
    Fuel,
    /// The memory cap refused a growth (evidence: our own limiter recorded the refusal).
    Memory,
    /// The wall-clock watchdog fired (evidence: our own watchdog flag + an epoch interrupt).
    Wall,
    /// A trap that is **not** a limit: `unreachable`, divide-by-zero, out-of-bounds — a **bug**.
    /// Raising limits cannot fix this, so it must never be reported as DL1506.
    Fault(String),
    /// The engine did not give evidence that attributes this stop. **Honest ignorance**: the plugin
    /// is still killed, but nothing is claimed about why, and no widening repair is offered.
    Unattributable(String),
    /// This platform + engine config cannot **enforce** the granted CPU/wall limits safely, so the
    /// run was **refused before it started** (Windows, Phase 6f.2b — see [`run_contained_export`]).
    /// Not a limit that was hit; not a plugin fault. Never DL1506, never an authority-widening
    /// repair — the honest degradation for a platform where in-process enforcement would fastfail.
    EnforcementUnsupported(String),
}

impl TrapCause {
    /// Whether this cause is a **limit** — the only kind DL1506 (and its authority-widening
    /// "raise limits" repair) may be emitted for.
    pub fn is_limit(&self) -> bool {
        matches!(self, TrapCause::Fuel | TrapCause::Memory | TrapCause::Wall)
    }

    /// The limit's name for the DL1506 message (`fuel` / `mem_mb` / `wall_ms`).
    pub fn limit_name(&self) -> Option<&'static str> {
        match self {
            TrapCause::Fuel => Some("fuel"),
            TrapCause::Memory => Some("mem_mb"),
            TrapCause::Wall => Some("wall_ms"),
            _ => None,
        }
    }

    /// The honest, human-facing statement of what happened.
    pub fn message(&self) -> String {
        match self {
            TrapCause::Fuel => "exhausted its granted fuel".into(),
            TrapCause::Memory => "exceeded its granted memory cap".into(),
            TrapCause::Wall => "exceeded its granted wall-clock budget".into(),
            TrapCause::Fault(w) => format!(
                "trapped on a fault, not a limit: {w} — this is a bug in the plugin; raising its limits will not fix it and would only widen its authority"
            ),
            TrapCause::Unattributable(w) => format!(
                "stopped for a reason this engine cannot attribute ({w}) — the plugin was terminated, but no cause is claimed and no limit change is advised"
            ),
            TrapCause::EnforcementUnsupported(w) => format!(
                "was not run because its CPU/wall limits cannot be enforced here: {w}"
            ),
        }
    }
}

/// The recording side of the memory cap. A plain `StoreLimits` refuses a growth silently (the
/// guest's `memory.grow` just returns −1), which leaves no evidence — and evidence is precisely what
/// [`attribute`] must not do without. This records the refusal so a memory kill is provable rather
/// than guessed.
pub struct PluginLimiter {
    max_bytes: usize,
    /// Set the moment we refuse a growth. This is the *evidence* for [`TrapCause::Memory`].
    pub memory_refused: bool,
}

impl PluginLimiter {
    pub fn new(max_bytes: usize) -> PluginLimiter {
        PluginLimiter { max_bytes, memory_refused: false }
    }
}

impl ResourceLimiter for PluginLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        if desired > self.max_bytes {
            self.memory_refused = true;
            return Ok(false);
        }
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(true)
    }
}

/// The store state a Contained plugin runs against: the memory limiter plus the watchdog flag.
pub struct PluginStoreState {
    pub limiter: PluginLimiter,
    /// Set by the watchdog thread when the wall-clock budget elapsed. The *evidence* for
    /// [`TrapCause::Wall`] — an epoch interrupt alone does not prove it was ours.
    pub wall_expired: Arc<AtomicBool>,
}

/// The effective limits for a load: `0` means "profile default", never "unlimited" (spec §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effective {
    pub fuel: u64,
    pub mem_bytes: usize,
    pub wall_ms: u64,
}

impl Effective {
    pub fn from(limits: &Limits) -> Effective {
        Effective {
            fuel: if limits.fuel > 0 { limits.fuel as u64 } else { DEFAULT_FUEL },
            mem_bytes: (if limits.mem_mb > 0 { limits.mem_mb as usize } else { DEFAULT_MEM_MB }) * 1024 * 1024,
            wall_ms: if limits.wall_ms > 0 { limits.wall_ms as u64 } else { DEFAULT_WALL_MS },
        }
    }
}

/// Attribute a stopped instance **from evidence**. `err` is the error Wasmtime returned (if any);
/// `state` carries our limiter's and watchdog's records; `fuel_remaining` is the store's own
/// reading.
///
/// Order matters, and each arm names the evidence it stands on:
/// 1. **Memory** — our limiter recorded a refusal. Checked first because a refused growth often
///    provokes a *secondary* trap (an OOB access on the memory that never grew) that would
///    otherwise read as a bug.
/// 2. **Fuel** — Wasmtime's own `Trap::OutOfFuel`, or the store reporting zero fuel left.
/// 3. **Wall** — our watchdog fired *and* the trap is an epoch interrupt. Both, because an epoch
///    interrupt we did not cause proves nothing.
/// 4. **Fault** — a `Trap` that is none of the above: a bug. Never DL1506.
/// 5. **Unattributable** — anything else, including a non-`Trap` error. Honest ignorance.
pub fn attribute(
    // wasmtime 47 introduced its own `wasmtime::Error` instead of re-exporting `anyhow::Error`
    // (P17: upgraded from 27 to close RUSTSEC-2026-0096 and -0222). The call sites that feed this
    // live in `#[cfg(not(windows))]` code, so Windows compiled clean and only Linux caught it —
    // which is precisely why both platforms are built every pass.
    err: Option<&wasmtime::Error>,
    state: &PluginStoreState,
    fuel_remaining: Option<u64>,
) -> TrapCause {
    // (1) Our own limiter's refusal — the strongest evidence we hold.
    if state.limiter.memory_refused {
        return TrapCause::Memory;
    }
    let trap = err.and_then(|e| e.downcast_ref::<Trap>()).copied();

    // (2) Fuel: the engine's own discriminant, corroborated by the store's reading.
    if trap == Some(Trap::OutOfFuel) || fuel_remaining == Some(0) {
        return TrapCause::Fuel;
    }

    // (3) Wall: OUR watchdog fired AND the engine reports an epoch interrupt.
    if state.wall_expired.load(Ordering::SeqCst) && trap == Some(Trap::Interrupt) {
        return TrapCause::Wall;
    }

    match (trap, err) {
        // (4) A real trap that is not a limit — a bug in the plugin.
        (Some(t), _) => TrapCause::Fault(format!("{t}")),
        // (5) An error we cannot attribute at all. Say so; claim nothing.
        (None, Some(e)) => TrapCause::Unattributable(format!("{e}")),
        // Stopped with no error and no limit evidence: it simply returned.
        (None, None) => TrapCause::Unattributable("the instance stopped without an error".into()),
    }
}

/// Run one export of a Contained module under its granted [`Effective`] limits, on a real Wasmtime
/// store (Stage 6 §5.1). The store carries **fuel** metering, the [`PluginLimiter`] as its
/// `ResourceLimiter` (the memory cap), and a host-side **wall-clock watchdog** (epoch interruption).
///
/// Returns `Ok(result)` if the export returned, or `Err(cause)` — **attributed by evidence** — if
/// the instance stopped. The caller (the loader) turns a `cause.is_limit()` into DL1506 and drops
/// the instance + revokes the node in one act; a `Fault`/`Unattributable` is **never** DL1506 and
/// **never** yields an authority-widening repair.
///
/// This takes only a no-import (or self-contained) module — enough to enforce and witness the
/// limits. Binding a plugin's *granted* `delulu:cap` host imports for a Contained plugin that does
/// real I/O reuses `host.rs`'s cap surface and layers on top of this; the limits machinery is the
/// same either way (spec §5.2: a Verified plugin on the WASM engine gets it too).
pub fn run_contained_export(
    wasm: &[u8],
    export: &str,
    limits: Effective,
) -> Result<Option<i64>, TrapCause> {
    // --- Windows safety by construction (Phase 6f.2b, NOT-FIXED branch) ------------------------
    // In-process CPU/wall enforcement uses host-initiated wasm traps (fuel exhaustion / epoch
    // interruption). On Windows + wasmtime 27, unwinding one of those `__fastfail`s the host
    // process — an UNCATCHABLE crash. All three ruled candidates (wasm_backtrace(false),
    // signals_based_traps(false), and their combination) were tried and each still fastfailed at
    // `func.call`. A fastfail cannot be caught, so the only safe move is to NOT REACH IT: on
    // Windows we refuse the run up front rather than execute a path that can crash the host
    // (non-negotiable 1). This is stated in diagnostics and docs (non-negotiable 2); it is neither
    // a limit hit (never DL1506, never authority-widening) nor a plugin fault. The out-of-process
    // enforcement path is a separate head-chef ruling. See build-order deviation 7.
    #[cfg(windows)]
    {
        // Tail expression (not `return`): on Windows the `not(windows)` arm is cfg-stripped, so this
        // block IS the function's tail — no `return` needed (clippy `needless_return`).
        let _ = (wasm, export, limits);
        Err(TrapCause::EnforcementUnsupported(
            "in-process CPU/wall limits use host-initiated wasm traps, whose unwind fastfails the \
             host on Windows with this engine; Contained plugins are refused here until \
             out-of-process enforcement lands"
                .into(),
        ))
    }
    #[cfg(not(windows))]
    {
        run_contained_export_impl(wasm, export, limits)
    }
}

/// The real execution path (non-Windows). Split out so the Windows guard in
/// [`run_contained_export`] is a clean early return and this body never compiles into a Windows
/// binary at all — the fastfailing path literally does not exist there (safety by construction).
#[cfg(not(windows))]
fn run_contained_export_impl(
    wasm: &[u8],
    export: &str,
    limits: Effective,
) -> Result<Option<i64>, TrapCause> {
    // A fresh Engine per run: the watchdog increments THIS engine's epoch only, so a late fire can
    // never disturb another plugin (or the host).
    let mut config = Config::new();
    config.consume_fuel(true);
    config.epoch_interruption(true);
    // P17-F: the plugin store runs bytes the host did not compile, so it needs the feature
    // narrowing at least as much as the Stage-3 engine does. See `host::harden_wasm_features`.
    crate::host::harden_wasm_features(&mut config);
    // --- Windows host-safety by construction (Phase 6f.2b) ------------------------------------
    // Applied to the PLUGIN store's Config ONLY — Stage 3's engine Config is untouched, so its
    // 5,000-program differential is unaffected.
    //
    // Host-initiated traps (fuel exhaustion, epoch interruption) unwind the guest differently from a
    // guest `unreachable`. On Windows + wasmtime 27, that unwind `__fastfail`s the host process
    // (0xc0000409) — an uncatchable crash, so the only safe move is to not reach it. Two settings,
    // both of which our evidence-based attribution makes free:
    //   1. wasm_backtrace_max_frames(None): wasmtime otherwise stack-walks the guest on every trap to
    //      build a backtrace — a known Windows fastfail source. We never surface a guest backtrace
    //      (attribution reads the store's own fuel/limiter/watchdog state, not a stack trace).
    //      `None` is wasmtime's own documented equivalent of the deprecated `wasm_backtrace(false)`.
    config.wasm_backtrace_max_frames(None);
    //   2. signals_based_traps(false): deliver traps as an ordinary returned error instead of via
    //      the SEH/signal unwind path that fastfails. Costs some throughput; a plugin sandbox trades
    //      throughput for not being able to crash its host.
    config.signals_based_traps(false);
    let engine = match Engine::new(&config) {
        Ok(e) => e,
        Err(e) => return Err(TrapCause::Unattributable(format!("engine setup failed: {e}"))),
    };

    let module = match Module::new(&engine, wasm) {
        Ok(m) => m,
        // A module that does not compile cannot be run; that is not a limit and not a plugin bug in
        // the trap sense — it never started. Honest ignorance about a "trap" it never reached.
        Err(e) => return Err(TrapCause::Unattributable(format!("module failed to compile: {e}"))),
    };

    let wall_expired = Arc::new(AtomicBool::new(false));
    let state = PluginStoreState {
        limiter: PluginLimiter::new(limits.mem_bytes),
        wall_expired: wall_expired.clone(),
    };
    let mut store = Store::new(&engine, state);
    store.limiter(|s| &mut s.limiter);
    if store.set_fuel(limits.fuel).is_err() {
        return Err(TrapCause::Unattributable("could not set store fuel".into()));
    }
    // The guest traps when the engine epoch advances by one past now — the watchdog does exactly
    // that after the wall budget.
    store.set_epoch_deadline(1);

    // The watchdog: after `wall_ms`, flag the cause and advance the epoch. `done` lets it exit
    // early (and NOT fire) when the call finished on its own, so it never interrupts a later run.
    let done = Arc::new(AtomicBool::new(false));
    let watchdog = {
        let engine = engine.clone();
        let wall_expired = wall_expired.clone();
        let done = done.clone();
        let wall_ms = limits.wall_ms;
        std::thread::spawn(move || {
            let mut waited = 0u64;
            let step = 5u64;
            while waited < wall_ms {
                if done.load(Ordering::SeqCst) {
                    return; // the call finished; do not fire
                }
                std::thread::sleep(Duration::from_millis(step.min(wall_ms - waited)));
                waited += step;
            }
            if !done.load(Ordering::SeqCst) {
                wall_expired.store(true, Ordering::SeqCst); // the evidence for TrapCause::Wall
                engine.increment_epoch();
            }
        })
    };

    let linker: Linker<PluginStoreState> = Linker::new(&engine);
    let outcome = (|| {
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| attribute(Some(&e), store.data(), store.get_fuel().ok()))?;
        let func = instance
            .get_func(&mut store, export)
            .ok_or_else(|| TrapCause::Unattributable(format!("no export `{export}`")))?;
        // Call with no args; capture a single optional i64 result (enough for the limit witnesses).
        let ty = func.ty(&store);
        let mut results = vec![Val::I32(0); ty.results().len()];
        match func.call(&mut store, &[], &mut results) {
            Ok(()) => Ok(results.first().and_then(|v| match v {
                Val::I32(n) => Some(*n as i64),
                Val::I64(n) => Some(*n),
                _ => None,
            })),
            Err(e) => Err(attribute(Some(&e), store.data(), store.get_fuel().ok())),
        }
    })();

    // Stop the watchdog and reap it, whatever happened. The instance (`store`) is dropped when this
    // function returns — a killed plugin's engine state does not outlive the call.
    done.store(true, Ordering::SeqCst);
    let _ = watchdog.join();
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(memory_refused: bool, wall: bool) -> PluginStoreState {
        PluginStoreState {
            limiter: PluginLimiter { max_bytes: 1024, memory_refused },
            wall_expired: Arc::new(AtomicBool::new(wall)),
        }
    }

    #[test]
    fn zero_limits_mean_profile_defaults_never_unlimited() {
        // spec §4: `0` = broker/profile default. It must never read as "no limit".
        let e = Effective::from(&Limits { fuel: 0, mem_mb: 0, wall_ms: 0 });
        assert_eq!(e.fuel, DEFAULT_FUEL);
        assert_eq!(e.mem_bytes, DEFAULT_MEM_MB * 1024 * 1024);
        assert_eq!(e.wall_ms, DEFAULT_WALL_MS);
        assert!(e.fuel > 0 && e.mem_bytes > 0 && e.wall_ms > 0, "a default is a bound, not an absence");
        let e = Effective::from(&Limits { fuel: 7, mem_mb: 3, wall_ms: 11 });
        assert_eq!((e.fuel, e.mem_bytes, e.wall_ms), (7, 3 * 1024 * 1024, 11));
    }

    #[test]
    fn a_memory_refusal_is_attributed_by_our_own_limiter() {
        // Evidence: the limiter recorded it. Even a secondary bug-looking trap does not mask it.
        let s = state(true, false);
        assert_eq!(attribute(None, &s, None), TrapCause::Memory);
        let boom: wasmtime::Error = Trap::MemoryOutOfBounds.into();
        assert_eq!(attribute(Some(&boom), &s, None), TrapCause::Memory);
    }

    #[test]
    fn fuel_is_attributed_from_the_engines_own_discriminant() {
        let s = state(false, false);
        let e: wasmtime::Error = Trap::OutOfFuel.into();
        assert_eq!(attribute(Some(&e), &s, None), TrapCause::Fuel);
        // Corroboration path: the store reports zero fuel left.
        assert_eq!(attribute(None, &s, Some(0)), TrapCause::Fuel);
    }

    #[test]
    fn wall_needs_both_our_watchdog_and_an_epoch_interrupt() {
        let interrupt: wasmtime::Error = Trap::Interrupt.into();
        // Watchdog fired AND epoch interrupt → Wall.
        assert_eq!(attribute(Some(&interrupt), &state(false, true), None), TrapCause::Wall);
        // An epoch interrupt we did NOT cause proves nothing — never claim Wall on it.
        assert_ne!(attribute(Some(&interrupt), &state(false, false), None), TrapCause::Wall);
    }

    /// THE HEAD-CHEF CASE, both directions. A bug-trap must never be reported as a limit, because
    /// DL1506's repair is `authority_widening: true` — mislabelling would make the language advise
    /// widening a plugin's authority to fix a bug that authority cannot fix.
    #[test]
    fn a_bug_trap_under_generous_limits_is_never_a_limit() {
        let s = state(false, false); // nothing refused, watchdog silent: limits were generous
        for t in [
            Trap::UnreachableCodeReached,
            Trap::IntegerDivisionByZero,
            Trap::MemoryOutOfBounds,
            Trap::TableOutOfBounds,
            Trap::StackOverflow,
        ] {
            let e: wasmtime::Error = t.into();
            let cause = attribute(Some(&e), &s, Some(1_000_000));
            assert!(!cause.is_limit(), "{t} must NOT be attributed to a limit: {cause:?}");
            assert!(matches!(cause, TrapCause::Fault(_)), "{t} is a fault: {cause:?}");
            assert!(cause.limit_name().is_none(), "a fault names no limit to raise");
            assert!(
                cause.message().contains("raising its limits will not fix it"),
                "the message must say raising limits is the wrong direction: {}",
                cause.message()
            );
        }
    }

    #[test]
    fn an_unattributable_stop_claims_nothing_and_advises_no_widening() {
        // "The engine cannot tell why" is sayable, and it is NOT a limit.
        let s = state(false, false);
        let odd: wasmtime::Error = wasmtime::Error::msg("host call failed in a way we did not model");
        let cause = attribute(Some(&odd), &s, Some(500));
        assert!(matches!(cause, TrapCause::Unattributable(_)));
        assert!(!cause.is_limit(), "an unattributable stop is never DL1506");
        assert!(cause.limit_name().is_none(), "and never names a limit to raise");
        assert!(cause.message().contains("no cause is claimed"), "{}", cause.message());
        assert!(cause.message().contains("no limit change is advised"), "{}", cause.message());
    }

    // On Windows, in-process CPU/wall enforcement is refused up front (6f.2b, NOT-FIXED branch):
    // the honest degradation returns `EnforcementUnsupported` and never executes into a fastfail.
    #[cfg(windows)]
    #[test]
    fn windows_refuses_contained_execution_rather_than_risk_a_fastfail() {
        // A minimal module — even a well-behaved one — is refused, BEFORE any store is created, so
        // no code path that could fastfail the host is ever reached. The refusal is honest: not a
        // limit (never DL1506), not a plugin fault, and it names the reason.
        let e = run_contained_export(b"\0asm\x01\0\0\0", "run", Effective { fuel: 1, mem_bytes: 1, wall_ms: 1 })
            .expect_err("Windows refuses in-process Contained execution");
        assert!(matches!(e, TrapCause::EnforcementUnsupported(_)));
        assert!(!e.is_limit(), "an enforcement-unsupported refusal is never a limit (never DL1506)");
        assert!(e.limit_name().is_none(), "and never advises raising a limit");
        assert!(e.message().contains("cannot be enforced here"), "{}", e.message());
    }

    // ----- criterion 5 against the LIVE engine (not the semantics). Gated off Windows: in-process
    // CPU/wall enforcement fastfails the host there (6f.2b), so these run on the Linux cross-check,
    // which is the enforcement-grade platform (spec §5.2/§5.4). The evidence-based ATTRIBUTION
    // itself is witnessed on every platform by the unit tests above. -----------------------------
    #[cfg(not(windows))]
    mod live_engine {
        use super::*;
        use wasm_encoder::{
            CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
            MemorySection, MemoryType, Module as WasmModule, TypeSection, ValType,
        };

    /// A module whose `run` export spins forever: `(loop (br 0))`. Consumes fuel, touches no memory.
    fn infinite_loop_module() -> Vec<u8> {
        let mut m = WasmModule::new();
        let mut t = TypeSection::new();
        t.ty().function([], []);
        m.section(&t);
        let mut f = FunctionSection::new();
        f.function(0);
        m.section(&f);
        let mut e = ExportSection::new();
        e.export("run", ExportKind::Func, 0);
        m.section(&e);
        let mut code = CodeSection::new();
        let mut body = Function::new([]);
        body.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
        body.instruction(&Instruction::Br(0));
        body.instruction(&Instruction::End); // loop
        body.instruction(&Instruction::End); // func
        code.function(&body);
        m.section(&code);
        m.finish()
    }

    /// A module whose `run` grows memory one page at a time until the cap refuses it, then traps.
    /// The refusal (recorded by the limiter) is the evidence; the trap after it is incidental.
    fn memory_bomb_module() -> Vec<u8> {
        let mut m = WasmModule::new();
        let mut t = TypeSection::new();
        t.ty().function([], []);
        m.section(&t);
        let mut f = FunctionSection::new();
        f.function(0);
        m.section(&f);
        let mut mem = MemorySection::new();
        mem.memory(MemoryType { minimum: 1, maximum: None, memory64: false, shared: false, page_size_log2: None });
        m.section(&mem);
        let mut e = ExportSection::new();
        e.export("run", ExportKind::Func, 0);
        m.section(&e);
        let mut code = CodeSection::new();
        let mut body = Function::new([]);
        body.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
        body.instruction(&Instruction::I32Const(1));
        body.instruction(&Instruction::MemoryGrow(0));
        body.instruction(&Instruction::I32Const(-1));
        body.instruction(&Instruction::I32Eq); // grow == -1 ?
        body.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
        body.instruction(&Instruction::Unreachable); // cap refused → stop
        body.instruction(&Instruction::End); // if
        body.instruction(&Instruction::Br(0));
        body.instruction(&Instruction::End); // loop
        body.instruction(&Instruction::End); // func
        code.function(&body);
        m.section(&code);
        m.finish()
    }

    /// A module whose `run` immediately hits `unreachable` — a bug-trap, not a limit.
    fn bug_trap_module() -> Vec<u8> {
        let mut m = WasmModule::new();
        let mut t = TypeSection::new();
        t.ty().function([], []);
        m.section(&t);
        let mut f = FunctionSection::new();
        f.function(0);
        m.section(&f);
        let mut e = ExportSection::new();
        e.export("run", ExportKind::Func, 0);
        m.section(&e);
        let mut code = CodeSection::new();
        let mut body = Function::new([]);
        body.instruction(&Instruction::Unreachable);
        body.instruction(&Instruction::End);
        code.function(&body);
        m.section(&code);
        m.finish()
    }

    /// A well-behaved module: `run() -> i32` returns 42. Stands in for "the host keeps working".
    fn honest_module() -> Vec<u8> {
        let mut m = WasmModule::new();
        let mut t = TypeSection::new();
        t.ty().function([], [ValType::I32]);
        m.section(&t);
        let mut f = FunctionSection::new();
        f.function(0);
        m.section(&f);
        let mut e = ExportSection::new();
        e.export("run", ExportKind::Func, 0);
        m.section(&e);
        let mut code = CodeSection::new();
        let mut body = Function::new([]);
        body.instruction(&Instruction::I32Const(42));
        body.instruction(&Instruction::End);
        code.function(&body);
        m.section(&code);
        m.finish()
    }

    fn limits(fuel: u64, mem_mb: usize, wall_ms: u64) -> Effective {
        Effective { fuel, mem_bytes: mem_mb * 1024 * 1024, wall_ms }
    }

    #[test]
    fn criterion5_infinite_loop_dies_at_fuel_and_the_host_survives() {
        // Low fuel, generous wall → fuel is the cause, decided by the engine's own discriminant.
        let cause = run_contained_export(&infinite_loop_module(), "run", limits(100_000, 64, 60_000))
            .expect_err("an infinite loop must be killed");
        assert_eq!(cause, TrapCause::Fuel, "attributed to fuel by evidence, not inference");
        assert!(cause.is_limit());
        // THE HOST SURVIVES AND CONTINUES: run real work afterward and get the real answer.
        let ok = run_contained_export(&honest_module(), "run", limits(1_000_000, 64, 1_000))
            .expect("the host keeps working after a kill");
        assert_eq!(ok, Some(42), "post-kill work returns the real result — the host was unharmed");
    }

    #[test]
    fn criterion5_infinite_loop_dies_at_wall_when_fuel_is_generous() {
        // Huge fuel, tiny wall → the watchdog wins, and it is proven by OUR flag + an epoch interrupt.
        let cause = run_contained_export(&infinite_loop_module(), "run", limits(u64::MAX, 64, 40))
            .expect_err("must be killed by the wall watchdog");
        assert_eq!(cause, TrapCause::Wall, "attributed to wall by evidence (watchdog flag + interrupt)");
        assert!(cause.is_limit());
        let ok = run_contained_export(&honest_module(), "run", limits(1_000_000, 64, 1_000)).expect("host survives");
        assert_eq!(ok, Some(42));
    }

    #[test]
    fn criterion5_memory_bomb_dies_at_mem_mb_and_the_host_survives() {
        // A 2 MiB cap, generous fuel/wall → the limiter refuses the grow and that is the evidence.
        let cause = run_contained_export(&memory_bomb_module(), "run", limits(u64::MAX, 2, 60_000))
            .expect_err("a memory bomb must be killed");
        assert_eq!(cause, TrapCause::Memory, "attributed to the memory cap by the limiter's own record");
        assert!(cause.is_limit());
        let ok = run_contained_export(&honest_module(), "run", limits(1_000_000, 64, 1_000)).expect("host survives");
        assert_eq!(ok, Some(42));
    }

    /// THE COULDN'T-TELL WITNESS, END-TO-END THROUGH THE LIVE ENGINE (the head chef's case). A
    /// bug-trap under GENEROUS limits must NOT be a limit — because DL1506's repair widens
    /// authority, and a bug is not fixed by more authority.
    #[test]
    fn criterion5_a_bug_trap_under_generous_limits_is_not_a_limit_through_the_real_engine() {
        let cause = run_contained_export(&bug_trap_module(), "run", limits(u64::MAX, 4096, 60_000))
            .expect_err("unreachable is a trap");
        assert!(!cause.is_limit(), "a bug-trap must never be attributed to a limit: {cause:?}");
        assert!(matches!(cause, TrapCause::Fault(_)), "it is a fault: {cause:?}");
        assert!(cause.limit_name().is_none(), "and names no limit to raise");
        // And the host keeps working — a bug-trap is contained just like a limit kill.
        let ok = run_contained_export(&honest_module(), "run", limits(1_000_000, 64, 1_000)).expect("host survives");
        assert_eq!(ok, Some(42));
    }

    #[test]
    fn a_well_behaved_module_returns_its_result_untouched() {
        assert_eq!(
            run_contained_export(&honest_module(), "run", limits(1_000_000, 64, 1_000)).expect("returns"),
            Some(42)
        );
    }
    } // mod live_engine

    #[test]
    fn only_the_three_real_limits_are_limits() {
        assert!(TrapCause::Fuel.is_limit() && TrapCause::Memory.is_limit() && TrapCause::Wall.is_limit());
        assert!(!TrapCause::Fault("x".into()).is_limit());
        assert!(!TrapCause::Unattributable("x".into()).is_limit());
        assert_eq!(TrapCause::Fuel.limit_name(), Some("fuel"));
        assert_eq!(TrapCause::Memory.limit_name(), Some("mem_mb"));
        assert_eq!(TrapCause::Wall.limit_name(), Some("wall_ms"));
    }
}
