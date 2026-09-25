//! PS-B-01: resource budgets on the main program — the owner's ruling D-V2-25, answering D-NE-31:
//! **1 GiB of memory and 5 minutes of processor time by default, never unlimited, and the operator
//! may change them.** Before this, an ordinary `run` had no bound at all on either engine (NE-22): an
//! actor's mailbox grew past a gigabyte under `--grant console` alone.
//!
//! # How it is enforced, and why this way
//!
//! A host-side watchdog samples the process's own peak memory and processor time every
//! [`INTERVAL`] and stops the run on the first breach. The alternatives were measured before this was
//! chosen, and each fails on one platform this project ships to:
//!
//! - `RLIMIT_AS` kills the WASM engine at start-up, because Wasmtime reserves address space it never
//!   uses (the scar is recorded in `jail.rs`).
//! - `RLIMIT_DATA` is refused outright on macOS — `setrlimit` returns EINVAL (PS-A experiment run
//!   35480762820) — so a memory ceiling built on it would exist on two systems out of three.
//! - An allocation that fails at an OS ceiling ends in Rust's allocation-failure abort, which prints
//!   one line and leaves no report: the run would be killed without saying what killed it.
//!
//! A sampler works the same way on all three, bounds the interpreter, the WASM engine and every actor
//! thread at once because it measures the PROCESS, and stops the run in a way that can still write
//! the report. Its honest cost is resolution: a program can overshoot by what it allocates in one
//! interval before the breach is seen. That is stated in the report (`enforced_by`) rather than
//! hidden.
//!
//! # Attribution, and the one thing a limit kill must never say
//!
//! A breach is reported from the watchdog's OWN measurement — the dimension, the budget and the
//! observed value — never inferred from the program having stopped. And the message never suggests
//! raising the budget as a repair: a budget is the operator's decision about what a program may
//! consume, and `limits.rs`'s rule (a limit kill is never an authority-widening suggestion) applies
//! here for the same reason it applies to plugins. The message names the flag as a fact, not a fix.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// How often the watchdog measures. Small enough that a runaway loop is stopped within a fraction of
/// a second, large enough that the measurement itself costs nothing a program would notice.
pub const INTERVAL: Duration = Duration::from_millis(25);

/// D-V2-25's memory default: 1 GiB.
pub const DEFAULT_MEMORY_BYTES: u64 = 1024 * 1024 * 1024;
/// D-V2-25's processor-time default: 5 minutes.
pub const DEFAULT_CPU_SECONDS: u64 = 300;

/// The budgets one run is held to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub memory_bytes: u64,
    pub cpu_seconds: u64,
    /// Wall-clock time. No default: D-V2-25 ruled memory and processor time, and a program waiting on
    /// its input is not a program using anything. The operator may set one.
    pub wall_seconds: Option<u64>,
}

impl Default for Budget {
    fn default() -> Self {
        Budget { memory_bytes: DEFAULT_MEMORY_BYTES, cpu_seconds: DEFAULT_CPU_SECONDS, wall_seconds: None }
    }
}

impl Budget {
    /// `--limits mem=BYTES,cpu=SECONDS,wall=SECONDS` for an ordinary run. Unnamed dimensions keep
    /// their defaults. Zero is refused in every dimension: it means either "stop at once", which
    /// nobody means, or "unlimited", which D-V2-25 forbids — and a spelling that could be read as
    /// either is not accepted as one of them.
    pub fn parse(spec: Option<&str>) -> Result<Budget, String> {
        let mut b = Budget::default();
        let Some(spec) = spec else { return Ok(b) };
        for part in spec.split(',').filter(|p| !p.is_empty()) {
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| format!("`--limits {part}` needs the form mem=BYTES, cpu=SECONDS or wall=SECONDS"))?;
            let n: u64 = value.trim().parse().map_err(|_| format!("`{value}` is not a number in `--limits {part}`"))?;
            if n == 0 {
                return Err(format!(
                    "`--limits {part}`: a budget of zero is refused — it would mean either \"stop at once\" \
                     or \"unlimited\", and a run is never unlimited (D-V2-25)"
                ));
            }
            match key.trim() {
                "mem" => b.memory_bytes = n,
                "cpu" => b.cpu_seconds = n,
                "wall" => b.wall_seconds = Some(n),
                other => {
                    return Err(format!("`--limits {other}=…` is not a limit this command knows (mem, cpu, wall)"))
                }
            }
        }
        Ok(b)
    }

    /// PS-B-05: a `--lease` run's budget when its node carries one. The delegated budget is both the
    /// ceiling and the default. A dimension `--limits` leaves unnamed takes the delegated value, not
    /// D-V2-25's, because the delegator already decided what this holder's runs may consume. A
    /// dimension it names may ask for less, never more. Asking for more is refused rather than
    /// quietly clipped: an operator who typed `mem=8589934592` must learn that it was not what the
    /// run got. Wall time is not part of the delegated dimension (a wall budget is the launcher's
    /// own control, `budget_scope.rs`), so `--limits wall=` keeps its ordinary meaning.
    pub fn under_delegation(spec: Option<&str>, ceiling: &delulu_broker::BudgetScope) -> Result<Budget, String> {
        let asked = Budget::parse(spec)?;
        let named = |key: &str| {
            spec.is_some_and(|s| s.split(',').any(|p| p.split_once('=').is_some_and(|(k, _)| k.trim() == key)))
        };
        let pick = |key: &str, asked: u64, ceiling: u64, unit: &dyn Fn(u64) -> String| -> Result<u64, String> {
            if !named(key) {
                return Ok(ceiling);
            }
            if asked > ceiling {
                return Err(format!(
                    "`--limits {key}={asked}` asks for {} but this lease was delegated {} — a lease run may \
                     be held to less than its delegation, never more",
                    unit(asked),
                    unit(ceiling)
                ));
            }
            Ok(asked)
        };
        Ok(Budget {
            memory_bytes: pick("mem", asked.memory_bytes, ceiling.memory_bytes, &human_bytes)?,
            cpu_seconds: pick("cpu", asked.cpu_seconds, ceiling.cpu_seconds, &|s| format!("{s} s of processor time"))?,
            wall_seconds: asked.wall_seconds,
        })
    }

    /// The part of this budget that is an authority dimension, in the broker's form — what a
    /// `--broker daemon` run's root node records it was held to, so every delegation below it
    /// inherits it or narrows it.
    pub fn to_scope(self) -> delulu_broker::BudgetScope {
        delulu_broker::BudgetScope { memory_bytes: self.memory_bytes, cpu_seconds: self.cpu_seconds }
    }

    /// The `limits` object of the run report.
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "memory_bytes": self.memory_bytes,
            "cpu_seconds": self.cpu_seconds,
            "wall_seconds": self.wall_seconds,
            "enforced_by": if usage().is_some() {
                format!("a host watchdog sampling this process every {} ms; a run can overshoot by what it allocates in one interval", INTERVAL.as_millis())
            } else {
                "nothing on this platform: the process cannot be measured here".to_string()
            },
        })
    }
}

/// What a sample measured: the process's PEAK memory so far (a spike between two samples is still
/// seen at the next one) and its processor time, every thread included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub peak_memory_bytes: u64,
    pub cpu: Duration,
}

/// Which budget was spent, with the watchdog's own measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breach {
    Memory { budget: u64, observed: u64 },
    Cpu { budget: u64, observed: Duration },
    Wall { budget: u64 },
}

impl Breach {
    pub fn dimension(&self) -> &'static str {
        match self {
            Breach::Memory { .. } => "memory",
            Breach::Cpu { .. } => "cpu",
            Breach::Wall { .. } => "wall",
        }
    }

    /// The operator's sentence. States what happened and whose decision a budget is; never offers
    /// raising it as a fix.
    pub fn explain(&self) -> String {
        match self {
            Breach::Memory { budget, observed } => format!(
                "the run was stopped: it used {} of memory and its budget is {} (D-V2-25). Budgets are the \
                 operator's, set per run with `--limits mem=BYTES`",
                human_bytes(*observed),
                human_bytes(*budget)
            ),
            Breach::Cpu { budget, observed } => format!(
                "the run was stopped: it used {:.1} s of processor time and its budget is {budget} s (D-V2-25). \
                 Budgets are the operator's, set per run with `--limits cpu=SECONDS`",
                observed.as_secs_f64()
            ),
            Breach::Wall { budget } => format!(
                "the run was stopped: it ran for {budget} s of wall-clock time, its budget. Budgets are the \
                 operator's, set per run with `--limits wall=SECONDS`"
            ),
        }
    }

    /// The `stopped_by` object of the run report's outcome.
    pub fn to_json(self) -> serde_json::Value {
        let dimension = self.dimension();
        match self {
            Breach::Memory { budget, observed } => {
                serde_json::json!({ "dimension": dimension, "budget_bytes": budget, "observed_bytes": observed })
            }
            Breach::Cpu { budget, observed } => serde_json::json!({
                "dimension": dimension, "budget_seconds": budget, "observed_seconds": observed.as_secs_f64(),
            }),
            Breach::Wall { budget } => serde_json::json!({ "dimension": dimension, "budget_seconds": budget }),
        }
    }
}

fn human_bytes(n: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    if n >= 1024 * MIB {
        format!("{:.2} GiB", n as f64 / (1024 * MIB) as f64)
    } else {
        format!("{:.1} MiB", n as f64 / MIB as f64)
    }
}

/// Decide, from one sample, whether a budget is spent. Pure, so the rule is tested without a process
/// that has to actually exhaust anything.
pub fn check(b: &Budget, u: &Usage, elapsed: Duration) -> Option<Breach> {
    if u.peak_memory_bytes > b.memory_bytes {
        return Some(Breach::Memory { budget: b.memory_bytes, observed: u.peak_memory_bytes });
    }
    if u.cpu > Duration::from_secs(b.cpu_seconds) {
        return Some(Breach::Cpu { budget: b.cpu_seconds, observed: u.cpu });
    }
    if let Some(w) = b.wall_seconds {
        if elapsed > Duration::from_secs(w) {
            return Some(Breach::Wall { budget: w });
        }
    }
    None
}

/// Set once the run has finished normally, so a breach measured in the same instant cannot also stop
/// it; and set by the watchdog before it stops the run, so the run's own ending cannot race it. The
/// first to set it owns the exit.
static CLAIMED: AtomicBool = AtomicBool::new(false);

/// The run finished. `true` means this caller owns the exit; `false` means the watchdog is already
/// stopping the process and the caller must not write a second report or a different exit code.
pub fn claim_normal_exit() -> bool {
    !CLAIMED.swap(true, Ordering::SeqCst)
}

/// Start the watchdog. On the first breach it claims the exit and calls `stop`, which is expected to
/// report and end the process. A budget that cannot be measured on this platform is not pretended:
/// only the wall clock is then enforced, and the report says so.
pub fn watch(b: Budget, stop: impl FnOnce(Breach) + Send + 'static) {
    let started = Instant::now();
    let spawned = std::thread::Builder::new().name("delulu-budget".into()).spawn(move || loop {
        std::thread::sleep(INTERVAL);
        if CLAIMED.load(Ordering::SeqCst) {
            return;
        }
        let u = usage().unwrap_or(Usage { peak_memory_bytes: 0, cpu: Duration::ZERO });
        if let Some(breach) = check(&b, &u, started.elapsed()) {
            if !CLAIMED.swap(true, Ordering::SeqCst) {
                stop(breach);
            }
            return;
        }
    });
    if spawned.is_err() {
        eprintln!("warning: the budget watchdog could not start; this run is NOT bounded");
    }
}

// ----- measuring the process -----------------------------------------------------------------

#[cfg(windows)]
pub fn usage() -> Option<Usage> {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
    // SAFETY: both calls write into stack structures of the declared size, for the pseudo-handle of
    // this process, which needs no closing.
    unsafe {
        let me = GetCurrentProcess();
        let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        if K32GetProcessMemoryInfo(me, &mut pmc, pmc.cb) == 0 {
            return None;
        }
        let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
        if GetProcessTimes(me, &mut created, &mut exited, &mut kernel, &mut user) == 0 {
            return None;
        }
        let ticks = |f: FILETIME| ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64;
        // FILETIME counts 100-nanosecond intervals. Peak COMMIT, the measure a Job Object's memory
        // ceiling uses for sandboxed guests, so the two budgets mean the same thing.
        Some(Usage {
            peak_memory_bytes: pmc.PeakPagefileUsage as u64,
            cpu: Duration::from_nanos((ticks(kernel) + ticks(user)).saturating_mul(100)),
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn usage() -> Option<Usage> {
    // SAFETY: `getrusage` writes one `rusage` for this process.
    let r = unsafe {
        let mut r: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut r) != 0 {
            return None;
        }
        r
    };
    // `ru_maxrss` is the peak resident set: kibibytes on Linux, bytes on macOS.
    let peak = if cfg!(target_os = "macos") { r.ru_maxrss as u64 } else { (r.ru_maxrss as u64).saturating_mul(1024) };
    let tv = |t: libc::timeval| Duration::new(t.tv_sec as u64, (t.tv_usec as u32).saturating_mul(1000));
    Some(Usage { peak_memory_bytes: peak, cpu: tv(r.ru_utime) + tv(r.ru_stime) })
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn usage() -> Option<Usage> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_the_owners_ruling() {
        let b = Budget::default();
        assert_eq!(b.memory_bytes, 1 << 30, "D-V2-25: 1 GiB");
        assert_eq!(b.cpu_seconds, 300, "D-V2-25: 5 minutes");
        assert_eq!(b.wall_seconds, None);
    }

    #[test]
    fn a_budget_is_never_zero_and_never_an_unknown_dimension() {
        for bad in ["mem=0", "cpu=0", "wall=0", "disk=1", "mem", "mem=lots", "cpu=-1"] {
            assert!(Budget::parse(Some(bad)).is_err(), "`{bad}` must be refused");
        }
        let b = Budget::parse(Some("mem=268435456,cpu=2,wall=9")).unwrap();
        assert_eq!((b.memory_bytes, b.cpu_seconds, b.wall_seconds), (268_435_456, 2, Some(9)));
        // The operator may RAISE a default at L0 — "the operator may change them" (D-V2-25).
        assert_eq!(Budget::parse(Some("mem=4294967296")).unwrap().memory_bytes, 4 << 30);
    }

    /// PS-B-05: the delegation is the ceiling AND the default; wall time stays the launcher's.
    #[test]
    fn a_lease_run_is_held_to_its_delegation_and_may_only_ask_for_less() {
        let ceiling = delulu_broker::BudgetScope { memory_bytes: 2 << 30, cpu_seconds: 600 };
        let b = Budget::under_delegation(None, &ceiling).unwrap();
        assert_eq!((b.memory_bytes, b.cpu_seconds, b.wall_seconds), (2 << 30, 600, None), "unnamed = delegated");
        let b = Budget::under_delegation(Some("cpu=60,wall=9"), &ceiling).unwrap();
        assert_eq!((b.memory_bytes, b.cpu_seconds, b.wall_seconds), (2 << 30, 60, Some(9)));
        let b = Budget::under_delegation(Some("mem=2147483648,cpu=600"), &ceiling).unwrap();
        assert_eq!((b.memory_bytes, b.cpu_seconds), (2 << 30, 600), "AT the ceiling is within it");
        for over in ["mem=2147483649", "cpu=601", "cpu=1,mem=4294967296"] {
            let e = Budget::under_delegation(Some(over), &ceiling).unwrap_err();
            assert!(e.contains("never more"), "`{over}`: {e}");
        }
        // A ceiling BELOW D-V2-25's defaults binds even though nothing was typed: the defaults are the
        // operator's for an undelegated run, not a floor under a delegation.
        let tight = delulu_broker::BudgetScope { memory_bytes: 1 << 20, cpu_seconds: 1 };
        let b = Budget::under_delegation(None, &tight).unwrap();
        assert_eq!((b.memory_bytes, b.cpu_seconds), (1 << 20, 1));
        assert!(Budget::under_delegation(Some("mem=0"), &ceiling).is_err(), "the ordinary spelling rules still hold");
        assert_eq!(b.to_scope(), tight);
    }

    #[test]
    fn a_breach_is_decided_by_the_measurement_and_only_by_it() {
        let b = Budget { memory_bytes: 100, cpu_seconds: 2, wall_seconds: Some(5) };
        let calm = Usage { peak_memory_bytes: 100, cpu: Duration::from_secs(2) };
        assert_eq!(check(&b, &calm, Duration::from_secs(5)), None, "AT the budget is within it");
        let fat = Usage { peak_memory_bytes: 101, ..calm };
        assert_eq!(check(&b, &fat, Duration::ZERO), Some(Breach::Memory { budget: 100, observed: 101 }));
        let busy = Usage { cpu: Duration::from_millis(2001), ..calm };
        assert!(matches!(check(&b, &busy, Duration::ZERO), Some(Breach::Cpu { budget: 2, .. })));
        assert_eq!(check(&b, &calm, Duration::from_millis(5001)), Some(Breach::Wall { budget: 5 }));
    }

    /// The message states the fact and whose decision a budget is — it never proposes raising it.
    #[test]
    fn a_limit_kill_never_reads_as_a_suggestion_to_widen() {
        for b in [
            Breach::Memory { budget: 1 << 30, observed: 3 << 29 },
            Breach::Cpu { budget: 300, observed: Duration::from_secs(301) },
            Breach::Wall { budget: 60 },
        ] {
            let m = b.explain().to_lowercase();
            for word in ["raise", "increase", "try ", "should", "to fix"] {
                assert!(!m.contains(word), "`{word}` in: {m}");
            }
            assert!(m.contains("operator"), "{m}");
        }
    }

    #[test]
    fn this_platform_can_measure_itself() {
        let u = usage().expect("Windows, Linux and macOS all measure the process");
        assert!(u.peak_memory_bytes > 0, "{u:?}");
    }
}
