//! The pluggable TTL clock (ruling 3: determinism injection).
//!
//! Liveness/TTL checks read "now" through a [`ClockSource`] with an OS-backed default; tests inject
//! a [`ManualClock`] they advance by hand, so TTL expiry is exercised with NO sleeps (playbook 5c).

use std::cell::Cell;
use std::rc::Rc;

/// A monotone-ish wall clock in epoch milliseconds.
pub trait ClockSource {
    fn now_millis(&self) -> i64;
}

/// The production clock: the system wall clock.
pub struct SystemClock;

impl ClockSource for SystemClock {
    fn now_millis(&self) -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

/// A hand-driven clock for tests. Interior mutability so a test can advance it while the broker
/// holds the same instance (via `Rc<ManualClock>`).
pub struct ManualClock {
    now: Cell<i64>,
}

impl ManualClock {
    pub fn new(start_millis: i64) -> ManualClock {
        ManualClock { now: Cell::new(start_millis) }
    }
    pub fn set(&self, t: i64) {
        self.now.set(t);
    }
    pub fn advance(&self, dt: i64) {
        self.now.set(self.now.get() + dt);
    }
}

impl ClockSource for ManualClock {
    fn now_millis(&self) -> i64 {
        self.now.get()
    }
}

// So a broker can hold `Box<dyn ClockSource>` while the test keeps another handle to advance it.
impl ClockSource for Rc<ManualClock> {
    fn now_millis(&self) -> i64 {
        self.now.get()
    }
}
