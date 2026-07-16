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
    err: Option<&anyhow::Error>,
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
        let boom: anyhow::Error = Trap::MemoryOutOfBounds.into();
        assert_eq!(attribute(Some(&boom), &s, None), TrapCause::Memory);
    }

    #[test]
    fn fuel_is_attributed_from_the_engines_own_discriminant() {
        let s = state(false, false);
        let e: anyhow::Error = Trap::OutOfFuel.into();
        assert_eq!(attribute(Some(&e), &s, None), TrapCause::Fuel);
        // Corroboration path: the store reports zero fuel left.
        assert_eq!(attribute(None, &s, Some(0)), TrapCause::Fuel);
    }

    #[test]
    fn wall_needs_both_our_watchdog_and_an_epoch_interrupt() {
        let interrupt: anyhow::Error = Trap::Interrupt.into();
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
            let e: anyhow::Error = t.into();
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
        let odd: anyhow::Error = anyhow::anyhow!("host call failed in a way we did not model");
        let cause = attribute(Some(&odd), &s, Some(500));
        assert!(matches!(cause, TrapCause::Unattributable(_)));
        assert!(!cause.is_limit(), "an unattributable stop is never DL1506");
        assert!(cause.limit_name().is_none(), "and never names a limit to raise");
        assert!(cause.message().contains("no cause is claimed"), "{}", cause.message());
        assert!(cause.message().contains("no limit change is advised"), "{}", cause.message());
    }

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
