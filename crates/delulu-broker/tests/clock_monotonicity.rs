//! P17-7 — expiry is judged against a **wall clock**, so time moving backwards resurrects
//! authority that had already expired.
//!
//! `time.rs:14-25` — the production `ClockSource` is `SystemClock`, i.e. `SystemTime::now()`
//! measured from `UNIX_EPOCH`. `effective_state` compares a node's absolute `ttl_millis` deadline
//! against that reading, and `effective_state_inherited` folds the same comparison up the tree. A
//! wall clock is not monotonic: NTP steps it, an operator sets it, a device restores it from RTC.
//!
//! ## Why this is not merely theoretical for the domains this project targets
//!
//! The stated users include satellites, autonomous aircraft and robots. On exactly those platforms
//! a **backwards clock step is routine, not adversarial**: GPS/GNSS time acquisition after a cold
//! start, an NTP correction after drift, an RTC read at power-on. Each of those can move the clock
//! backwards across a deadline that had already passed — and every grant, lease and adopted
//! certificate that expired in that window becomes live again, with no revocation, no audit event,
//! and nothing anywhere recording that authority was restored.
//!
//! The uplink lease (RFC 0001 F4) makes this sharper. Its entire purpose is to be *the bound that
//! survives a partition*, because revocation cannot cross one. That bound is a wall-clock deadline.
//!
//! ## What this test does and does not claim
//!
//! It claims exactly what it executes: with the clock moved backwards, a node that reported
//! `Expired` reports `Live` again. It does **not** claim an attacker can set your clock — on most
//! hosts that needs privilege. The point is that the guarantee rests on clock monotonicity, which
//! is an assumption the design never states.
//!
//! A fix would judge expiry against a monotonic reading (`Instant`) anchored once at start, or
//! refuse to move a deadline that has already been observed as passed. Both change the persistence
//! and clock contract, so this is recorded rather than patched. Ruling D20 already moved the
//! simulator's dead-man onto a *logical* clock for the same class of reason; broker expiry did not
//! get the same treatment.

use std::collections::BTreeSet;

use delulu_broker::authority::{Authority, Scopes};
use delulu_broker::time::ManualClock;
use delulu_broker::tree::{Broker, Holder};
use delulu_broker::ids::SeqIdSource;

fn empty_authority() -> Authority {
    Authority { effects: BTreeSet::new(), scopes: Scopes::default() }
}

#[test]
fn moving_the_clock_backwards_resurrects_an_expired_grant() {
    let clock = std::rc::Rc::new(ManualClock::new(1_000));
    let mut b = Broker::with_sources(Box::new(SeqIdSource::default()), Box::new(clock.clone()));

    // A root grant with an absolute deadline at t = 5_000.
    let root = b.issue(Holder::new("test", "operator", "root"), empty_authority(), Some(5_000));

    // t = 1_000: alive.
    assert!(
        b.effective_state(&root).is_some_and(|s| matches!(s, delulu_broker::tree::EffState::Live)),
        "the grant is live before its deadline"
    );

    // t = 9_000: the deadline has passed.
    clock.set(9_000);
    let expired = b.effective_state(&root).expect("node exists");
    assert!(
        matches!(expired, delulu_broker::tree::EffState::Expired { .. }),
        "past its deadline the grant must be Expired, got {expired:?}"
    );

    // t = 2_000 — the clock steps BACKWARDS across the deadline. An NTP correction, a GNSS fix
    // after a cold start, an RTC read at power-on.
    clock.set(2_000);
    let after = b.effective_state(&root).expect("node still exists");

    assert!(
        matches!(after, delulu_broker::tree::EffState::Live),
        "OBSERVED: the expired grant is LIVE again after the clock moved backwards — expiry is \
         judged against a wall clock and nothing records that authority was restored. Got {after:?}"
    );
}

/// The control: while the clock only moves FORWARD, expiry is permanent. This is what the design
/// actually relies on, and naming the limitation above must not be read as "expiry does not work".
#[test]
fn expiry_is_permanent_while_the_clock_only_moves_forward() {
    let clock = std::rc::Rc::new(ManualClock::new(0));
    let mut b = Broker::with_sources(Box::new(SeqIdSource::default()), Box::new(clock.clone()));
    let root = b.issue(Holder::new("test", "operator", "root"), empty_authority(), Some(100));

    for t in [50, 99, 100, 101, 500, 10_000] {
        clock.set(t);
        let st = b.effective_state(&root).expect("node exists");
        let live = matches!(st, delulu_broker::tree::EffState::Live);
        assert_eq!(
            live,
            t < 100,
            "at t={t} the grant should be {} — monotonic time gives monotonic expiry",
            if t < 100 { "live" } else { "expired" }
        );
    }
}
