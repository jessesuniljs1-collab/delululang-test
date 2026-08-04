//! P17-B2 — **FIXED.** Expiry is judged against a wall clock, so a backwards step used to
//! resurrect authority that had already expired. It cannot any more.
//!
//! ## What was observed, before the fix
//!
//! `time.rs:14-25` — the production `ClockSource` is `SystemTime::now()`, and `effective_state`
//! compares a node's absolute deadline against it. A wall clock is not monotonic. A grant deadlined
//! at t=5,000 reported `Live` at t=1,000, `Expired` at t=9,000, and **`Live` again** once the clock
//! was set back to t=2,000 — with no revocation, no audit event, and nothing anywhere recording
//! that authority had been restored.
//!
//! That is not an exotic scenario for the platforms this project targets. On satellites, autonomous
//! aircraft and robots a backwards step is **routine, not adversarial**: GNSS time acquisition after
//! a cold start, an NTP correction after drift, an RTC read at power-on. And the uplink lease
//! (RFC 0001 F4) — the bound that exists precisely because revocation cannot cross a partition — is
//! a wall-clock deadline.
//!
//! ## The fix, and why it is a ratchet rather than a monotonic clock
//!
//! `Broker::now` takes the running maximum of every reading it has ever taken. `Instant` was not an
//! option: certificate `not_before`/`not_after` are **signed absolute epoch-millis**, so the
//! comparison must stay wall-clock-comparable or a certificate minted by the ground could not be
//! evaluated at all. The ratchet keeps it comparable while making it non-decreasing.
//!
//! **It can only ever withhold authority, never grant it.** Clamping upward can expire something
//! early; it can never un-expire anything. Fail-closed in the direction that matters.
//!
//! ## What it does NOT fix, stated so the guarantee is not read wider than it is
//!
//! A backwards step still loses *time* — a grant that would have expired at t=5,000 does not become
//! immortal, but a broker whose clock is set back and then forward again measures the interval
//! differently from wall time. The ratchet guarantees **monotonicity**, not accuracy. And it is
//! per-broker state held in memory: a broker that restarts begins its ratchet afresh, which is
//! sound only because the grant tree does not persist either (P17-B1) — every node a restarted
//! broker holds was created after the restart.

use std::collections::BTreeSet;

use delulu_broker::authority::{Authority, Scopes};
use delulu_broker::ids::SeqIdSource;
use delulu_broker::time::ManualClock;
use delulu_broker::tree::{Broker, EffState, Holder};

fn empty_authority() -> Authority {
    Authority { effects: BTreeSet::new(), scopes: Scopes::default() }
}

/// **The regression witness.** This test asserted the opposite before the ratchet existed: the
/// grant came back to life. If it ever fails again, expiry has stopped being monotonic.
#[test]
fn moving_the_clock_backwards_cannot_resurrect_an_expired_grant() {
    let clock = std::rc::Rc::new(ManualClock::new(1_000));
    let mut b = Broker::with_sources(Box::new(SeqIdSource::default()), Box::new(clock.clone()));
    let root = b.issue(Holder::new("test", "operator", "root"), empty_authority(), Some(5_000));

    // t = 1,000 — alive, before its deadline.
    assert!(
        matches!(b.effective_state(&root), Some(EffState::Live)),
        "the grant is live before its deadline"
    );

    // t = 9,000 — the deadline has passed.
    assert!(
        matches!(b.effective_state(&root), Some(EffState::Expired { .. })) || {
            clock.set(9_000);
            matches!(b.effective_state(&root), Some(EffState::Expired { .. }))
        },
        "past its deadline the grant must be Expired"
    );

    // t = 2,000 — the clock steps BACKWARDS across the deadline. An NTP correction, a GNSS fix
    // after a cold start, an RTC read at power-on.
    clock.set(2_000);
    let after = b.effective_state(&root).expect("node still exists");

    assert!(
        matches!(after, EffState::Expired { .. }),
        "a backwards clock step MUST NOT resurrect an expired grant — the ratchet in `Broker::now` \
         takes the running maximum, so what expired stays expired. Got {after:?}"
    );
}

/// The ratchet must not break ordinary forward-only time: expiry still happens, at the right moment.
#[test]
fn expiry_still_works_normally_while_the_clock_moves_forward() {
    let clock = std::rc::Rc::new(ManualClock::new(0));
    let mut b = Broker::with_sources(Box::new(SeqIdSource::default()), Box::new(clock.clone()));
    let root = b.issue(Holder::new("test", "operator", "root"), empty_authority(), Some(100));

    for t in [50, 99, 100, 101, 500, 10_000] {
        clock.set(t);
        let st = b.effective_state(&root).expect("node exists");
        let live = matches!(st, EffState::Live);
        assert_eq!(
            live,
            t < 100,
            "at t={t} the grant should be {}",
            if t < 100 { "live" } else { "expired" }
        );
    }
}

/// **The ratchet must never GRANT authority.** A grant issued while the clock is behind must not
/// become live merely because the ratchet is ahead — clamping upward can only expire things early.
#[test]
fn the_ratchet_only_ever_withholds_authority_never_grants_it() {
    let clock = std::rc::Rc::new(ManualClock::new(10_000));
    let mut b = Broker::with_sources(Box::new(SeqIdSource::default()), Box::new(clock.clone()));

    // Pull the ratchet up to 10,000, then step the wall clock back to 1,000.
    let early = b.issue(Holder::new("test", "operator", "a"), empty_authority(), Some(20_000));
    assert!(matches!(b.effective_state(&early), Some(EffState::Live)), "live at t=10,000");
    clock.set(1_000);

    // A grant deadlined at 5,000 is issued while the WALL clock reads 1,000 — but the ratchet is at
    // 10,000, so it is already past its deadline and must be Expired immediately. Withheld, not granted.
    let late = b.issue(Holder::new("test", "operator", "b"), empty_authority(), Some(5_000));
    assert!(
        matches!(b.effective_state(&late), Some(EffState::Expired { .. })),
        "the ratchet must judge against 10,000, not the rewound 1,000 — withholding, never granting"
    );
}
