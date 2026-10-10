//! P8-04 step 5 — a dead-man FOR A NODE, held by the broker: what happens when a monitor dies
//! (D-V2-106).
//!
//! A dead monitor cannot act, so whatever quarantines on its death must live elsewhere — here, in the
//! broker that holds the tree. `delulu monitor watch` arms a dead-man on its own node `g_M` and beats
//! it every poll; if the beats stop for the armed period, the daemon revokes `g_M` AS ITSELF, and with
//! it every run below, the cause in the revocation's own record (`V2_P8_DESIGN.md` P8-04 step 5,
//! option (c)).
//!
//! **Why this hardens rather than redefines Authority.** Any node may already revoke itself
//! (`caller == target`, `tree.rs`'s `is_self_or_descendant`), so arming a FUTURE self-revocation adds
//! no authority: it can only ever remove some, as the device dead-man (10f) does for a lease. A
//! dead-man never re-grants, never extends a deadline, and never reaches outside the subtree it was
//! armed on.
//!
//! **A beat is a proof of possession.** Arming returns a fresh 128-bit key, held only in the arming
//! process's memory and, as its BLAKE3 hash, in the daemon's — never on disk, never in the chain. A
//! beat or a disarm without it is refused, and a second arming over a live one is refused, so another
//! process cannot keep a dead monitor's dead-man quiet, replace it, or switch it off without reading the
//! monitor's memory. A process that can do that is the same OS user (§11.4, category 7) — the boundary
//! this does not claim to cross.
//!
//! **Time is the monotonic clock**, never the wall clock: every deadline is an [`Instant`] the caller
//! passes, so a wall-clock step neither fires a dead-man early nor holds one off. A beat that arrives
//! after its deadline does not rescue the node: it fires it. The daemon owns the loop that calls
//! [`Broker::fire_expired_deadmen`]; an in-process broker has none, so a dead-man means something only
//! under the daemon.

use std::time::{Duration, Instant};

use crate::{Broker, Denial, EffState, GrantId};

/// The shortest period a dead-man may be armed with: below it, a scheduler's hiccup is a quarantine.
pub const MIN_PERIOD: Duration = Duration::from_millis(100);
/// The longest: past it a dead-man stands for nothing an operator could wait on.
pub const MAX_PERIOD: Duration = Duration::from_secs(3600);

/// One armed dead-man.
#[derive(Clone, Debug)]
pub(crate) struct Deadman {
    period: Duration,
    deadline: Instant,
    key_hash: blake3::Hash,
    armed_seq: u64,
    beats: u64,
}

/// A dead-man that fired: the node it revoked, and what it had waited for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fired {
    pub node: GrantId,
    pub period: Duration,
    pub armed_seq: u64,
    pub beats: u64,
    /// The seq of the revocation's record — what DL1403 tells every holder below.
    pub by_seq: u64,
    pub newly_revoked: Vec<GrantId>,
}

/// Why a dead-man operation was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeadmanRefusal {
    /// The tree's own answer: the node is unknown, revoked or expired.
    Tree(Denial),
    /// No dead-man is armed on the node.
    NotArmed,
    /// A dead-man is already armed on the node; disarm it (with its key) first.
    AlreadyArmed,
    /// The beat's or disarm's key is not the one its arming returned.
    WrongKey,
    /// The period is outside [`MIN_PERIOD`]..=[`MAX_PERIOD`].
    Period,
    /// The beat (or disarm) came after the deadline: the dead-man fired, and this is its revocation.
    Fired(Fired),
}

impl DeadmanRefusal {
    /// The refusal in words, for the daemon's answer.
    pub fn message(&self, node: &GrantId) -> String {
        let node = node.as_str();
        match self {
            DeadmanRefusal::Tree(d) => d.to_diagnostic().message,
            DeadmanRefusal::NotArmed => format!("no dead-man is armed on `{node}`"),
            DeadmanRefusal::AlreadyArmed => {
                format!("a dead-man is already armed on `{node}` — only its holder may disarm it, with the key arming returned")
            }
            DeadmanRefusal::WrongKey => format!("the key does not match the dead-man armed on `{node}`"),
            DeadmanRefusal::Period => format!(
                "a dead-man's period is {} ms to {} ms",
                MIN_PERIOD.as_millis(),
                MAX_PERIOD.as_millis()
            ),
            DeadmanRefusal::Fired(f) => format!(
                "the dead-man on `{node}` fired: no beat within {} ms — `{node}` and its subtree were revoked by audit seq {}",
                f.period.as_millis(),
                f.by_seq
            ),
        }
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl Broker {
    /// The node must be known and live (its ancestors included): a dead-man on a dead node stands for nothing.
    fn deadman_node_live(&self, node: &GrantId) -> Result<(), DeadmanRefusal> {
        match self.effective_state(node) {
            None => Err(DeadmanRefusal::Tree(Denial::UnknownNode { node: node.clone() })),
            Some(EffState::Live) => Ok(()),
            Some(EffState::Revoked { by_seq }) => Err(DeadmanRefusal::Tree(Denial::Revoked { node: node.clone(), by_seq })),
            Some(EffState::Expired { ttl_millis, now_millis }) => {
                Err(DeadmanRefusal::Tree(Denial::Expired { node: node.clone(), ttl_millis, now_millis }))
            }
        }
    }

    /// Arm a dead-man on `node`: unless [`Broker::beat_deadman`] is called with the returned key within
    /// `period` of arming and of every beat, [`Broker::fire_expired_deadmen`] revokes `node` as itself.
    /// Returns the seq of the `deadman-arm` record and the key. Recorded, so an investigator can tell a
    /// monitor that would quarantine on its death from one that would not.
    pub fn arm_deadman(&mut self, node: &GrantId, period: Duration, now: Instant) -> Result<(u64, String), DeadmanRefusal> {
        self.deadman_node_live(node)?;
        if !(MIN_PERIOD..=MAX_PERIOD).contains(&period) {
            return Err(DeadmanRefusal::Period);
        }
        if self.deadmen.contains_key(node) {
            return Err(DeadmanRefusal::AlreadyArmed);
        }
        let deadline = now.checked_add(period).ok_or(DeadmanRefusal::Period)?;
        let mut key = [0u8; 16];
        getrandom::fill(&mut key).expect("OS randomness (getrandom) unavailable");
        let key = to_hex(&key);
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "deadman-arm",
            Some(node.as_str().to_string()),
            Some(node.as_str().to_string()),
            Some(serde_json::json!({ "period_ms": period.as_millis() as u64, "on_death": "revoke-subtree" })),
            "allow",
            None,
        );
        self.deadmen.insert(
            node.clone(),
            Deadman { period, deadline, key_hash: blake3::hash(key.as_bytes()), armed_seq: seq, beats: 0 },
        );
        Ok((seq, key))
    }

    /// A beat: the dead-man on `node` waits another full period from `now`. Not recorded — a monitor
    /// beats every poll, and the chain would fill with them; its arming and its end are what the chain
    /// keeps. A beat after the deadline fires the dead-man instead (`Fired`).
    pub fn beat_deadman(&mut self, node: &GrantId, key: &str, now: Instant) -> Result<Duration, DeadmanRefusal> {
        let Some(d) = self.deadmen.get(node) else {
            self.deadman_node_live(node)?;
            return Err(DeadmanRefusal::NotArmed);
        };
        if blake3::hash(key.as_bytes()) != d.key_hash {
            return Err(DeadmanRefusal::WrongKey);
        }
        if now >= d.deadline {
            return Err(match self.fire_deadman(node) {
                Some(f) => DeadmanRefusal::Fired(f),
                None => self.deadman_node_live(node).err().unwrap_or(DeadmanRefusal::NotArmed),
            });
        }
        self.deadman_node_live(node)?;
        let d = self.deadmen.get_mut(node).expect("looked up above");
        d.deadline = now.checked_add(d.period).unwrap_or(d.deadline);
        d.beats += 1;
        Ok(d.period)
    }

    /// Disarm the dead-man on `node` — its holder ending cleanly. Recorded (`deadman-disarm`): from here
    /// the runs below outlive their monitor, and the chain says when that began. Past its deadline the
    /// dead-man has already lapsed, and fires instead.
    pub fn disarm_deadman(&mut self, node: &GrantId, key: &str, now: Instant) -> Result<u64, DeadmanRefusal> {
        let Some(d) = self.deadmen.get(node) else {
            self.deadman_node_live(node)?;
            return Err(DeadmanRefusal::NotArmed);
        };
        if blake3::hash(key.as_bytes()) != d.key_hash {
            return Err(DeadmanRefusal::WrongKey);
        }
        if now >= d.deadline {
            return Err(match self.fire_deadman(node) {
                Some(f) => DeadmanRefusal::Fired(f),
                None => self.deadman_node_live(node).err().unwrap_or(DeadmanRefusal::NotArmed),
            });
        }
        let d = self.deadmen.remove(node).expect("looked up above");
        self.deadman_node_live(node)?;
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "deadman-disarm",
            Some(node.as_str().to_string()),
            Some(node.as_str().to_string()),
            Some(serde_json::json!({ "armed_seq": d.armed_seq, "beats": d.beats })),
            "allow",
            None,
        );
        Ok(seq)
    }

    /// The nearest deadline of any armed dead-man — how long the daemon may wait for its next request.
    pub fn next_deadman_deadline(&self) -> Option<Instant> {
        self.deadmen.values().map(|d| d.deadline).min()
    }

    /// Whether a dead-man is armed on `node`.
    pub fn deadman_armed(&self, node: &GrantId) -> bool {
        self.deadmen.contains_key(node)
    }

    /// Fire every dead-man whose deadline is at or before `now`: each revokes its node as itself, the
    /// cause in the record. A dead-man whose node is no longer live (an operator revoked it, its lease
    /// expired) has nothing left to stop, and is dropped without a record.
    pub fn fire_expired_deadmen(&mut self, now: Instant) -> Vec<Fired> {
        let gone: Vec<GrantId> = self
            .deadmen
            .keys()
            .filter(|n| !matches!(self.effective_state(n), Some(EffState::Live)))
            .cloned()
            .collect();
        for n in gone {
            self.deadmen.remove(&n);
        }
        let mut due: Vec<GrantId> =
            self.deadmen.iter().filter(|(_, d)| d.deadline <= now).map(|(n, _)| n.clone()).collect();
        due.sort();
        due.iter().filter_map(|n| self.fire_deadman(n)).collect()
    }

    fn fire_deadman(&mut self, node: &GrantId) -> Option<Fired> {
        let d = self.deadmen.remove(node)?;
        self.deadman_node_live(node).ok()?;
        let why = format!(
            "dead-man: the holder of `{}` sent no beat within {} ms (armed at seq {}, {} beat(s) before it stopped) — \
             its subtree is revoked: a watchdog gone quiet is treated as one that fired (P8-04, D-V2-106)",
            node.as_str(),
            d.period.as_millis(),
            d.armed_seq,
            d.beats
        );
        let out = self.revoke_saying(node, node, Some(&why)).ok()?;
        Some(Fired {
            node: node.clone(),
            period: d.period,
            armed_seq: d.armed_seq,
            beats: d.beats,
            by_seq: out.by_seq,
            newly_revoked: out.newly_revoked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Authority, Holder, ManualClock, Scopes, SeqIdSource};
    use delulu_check::Effect;

    fn broker() -> (Broker, GrantId, GrantId, GrantId) {
        let mut b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(ManualClock::new(1_000)));
        let auth = || Authority::new(vec![Effect::Write], Scopes::default());
        let root = b.issue_root(Holder::new("process", "operator", "x"), auth(), None).unwrap();
        let m = b.attenuate(&root, auth(), Holder::new("process", "monitor", "x"), None).unwrap();
        let run = b.attenuate(&m, auth(), Holder::new("process", "run", "x"), None).unwrap();
        (b, root, m, run)
    }

    fn live(b: &Broker, n: &GrantId) -> bool {
        matches!(b.effective_state(n), Some(EffState::Live))
    }

    const P: Duration = Duration::from_millis(500);

    /// **A dead-man that is not beaten revokes its node and the subtree below it — and nothing above.**
    #[test]
    fn an_unbeaten_dead_man_revokes_its_node_and_subtree_and_nothing_above() {
        let (mut b, root, m, run) = broker();
        let t0 = Instant::now();
        let (armed, _key) = b.arm_deadman(&m, P, t0).unwrap();
        assert_eq!(b.next_deadman_deadline(), Some(t0 + P));
        assert!(b.fire_expired_deadmen(t0 + P - Duration::from_millis(1)).is_empty(), "not before its deadline");
        assert!(live(&b, &m) && live(&b, &run));
        let fired = b.fire_expired_deadmen(t0 + P);
        assert_eq!(fired.len(), 1, "{fired:?}");
        assert_eq!((fired[0].node.clone(), fired[0].armed_seq, fired[0].beats), (m.clone(), armed, 0));
        assert_eq!(fired[0].newly_revoked, {
            let mut v = vec![m.clone(), run.clone()];
            v.sort();
            v
        });
        assert!(!live(&b, &m) && !live(&b, &run), "the monitor's node and the run below it");
        assert!(live(&b, &root), "never upward: the operator's root lives");
        assert!(!b.deadman_armed(&m) && b.next_deadman_deadline().is_none(), "a dead-man fires once");
        assert!(b.fire_expired_deadmen(t0 + P * 10).is_empty());
    }

    /// **Each beat buys one more period from the beat, not from the arming.**
    #[test]
    fn a_beat_moves_the_deadline_one_period_past_the_beat() {
        let (mut b, _root, m, run) = broker();
        let t0 = Instant::now();
        let (_, key) = b.arm_deadman(&m, P, t0).unwrap();
        // Each beat just inside the period since the LAST one: five periods and more pass, and the
        // dead-man holds — it measures quiet, not age.
        let gap = P - Duration::from_millis(10);
        for i in 1..=5u32 {
            let t = t0 + gap * i;
            assert_eq!(b.beat_deadman(&m, &key, t), Ok(P), "beat {i}");
            assert!(b.fire_expired_deadmen(t).is_empty());
        }
        let last = t0 + gap * 5;
        assert!(last > t0 + P * 4, "the beats outlived four periods from the arming");
        assert!(b.fire_expired_deadmen(last + P - Duration::from_millis(1)).is_empty());
        assert!(live(&b, &run));
        let fired = b.fire_expired_deadmen(last + P);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].beats, 5, "the record says how long it was beaten");
        assert!(!live(&b, &run));
    }

    /// **A beat is a proof of possession:** a wrong key keeps nothing alive, a second arming cannot
    /// replace the first, and a disarm without the key switches nothing off.
    #[test]
    fn only_the_arming_key_beats_or_disarms_and_a_second_arming_is_refused() {
        let (mut b, _root, m, run) = broker();
        let t0 = Instant::now();
        let (_, key) = b.arm_deadman(&m, P, t0).unwrap();
        let wrong = "0".repeat(32);
        assert_eq!(b.beat_deadman(&m, &wrong, t0 + P / 2), Err(DeadmanRefusal::WrongKey));
        assert_eq!(b.disarm_deadman(&m, &wrong, t0 + P / 2), Err(DeadmanRefusal::WrongKey));
        assert_eq!(b.arm_deadman(&m, MAX_PERIOD, t0 + P / 2).map(|_| ()), Err(DeadmanRefusal::AlreadyArmed));
        // None of them moved the deadline.
        assert_eq!(b.fire_expired_deadmen(t0 + P).len(), 1);
        assert!(!live(&b, &run));
        let _ = key;
    }

    /// **A beat after the deadline fires the dead-man**, even when the daemon has not yet looked: late is
    /// dead, never rescued.
    #[test]
    fn a_late_beat_fires_the_dead_man_rather_than_rescuing_it() {
        let (mut b, _root, m, run) = broker();
        let t0 = Instant::now();
        let (_, key) = b.arm_deadman(&m, P, t0).unwrap();
        match b.beat_deadman(&m, &key, t0 + P) {
            Err(DeadmanRefusal::Fired(f)) => assert_eq!(f.node, m),
            other => panic!("a late beat must fire it: {other:?}"),
        }
        assert!(!live(&b, &run));
        // The next beat is told why: the node is revoked.
        assert!(matches!(b.beat_deadman(&m, &key, t0 + P * 2), Err(DeadmanRefusal::Tree(Denial::Revoked { .. }))));
    }

    /// **A clean end disarms, and is recorded; the runs below outlive the monitor.**
    #[test]
    fn a_disarm_with_the_key_ends_the_dead_man_and_the_runs_live_on() {
        let (mut b, _root, m, run) = broker();
        let t0 = Instant::now();
        let (armed, key) = b.arm_deadman(&m, P, t0).unwrap();
        let disarmed = b.disarm_deadman(&m, &key, t0 + P / 2).unwrap();
        assert!(disarmed > armed);
        assert!(b.fire_expired_deadmen(t0 + P * 100).is_empty());
        assert!(live(&b, &m) && live(&b, &run));
        assert_eq!(b.beat_deadman(&m, &key, t0 + P), Err(DeadmanRefusal::NotArmed));
    }

    /// **A dead-man on a node already revoked by someone else records nothing when its time comes** —
    /// there is nothing left for it to stop, and a second revocation record would be a false account.
    #[test]
    fn a_dead_man_whose_node_was_revoked_meanwhile_fires_nothing() {
        let (mut b, root, m, _run) = broker();
        let t0 = Instant::now();
        b.arm_deadman(&m, P, t0).unwrap();
        b.revoke(&root, &m).unwrap();
        let seq_before = b.next_audit_seq();
        assert!(b.fire_expired_deadmen(t0 + P).is_empty());
        assert_eq!(b.next_audit_seq(), seq_before, "no seq consumed, no record");
        assert!(!b.deadman_armed(&m));
    }

    /// **The edges refuse:** an unknown or dead node, and a period outside the bounds, arm nothing.
    #[test]
    fn arming_refuses_unknown_and_dead_nodes_and_periods_out_of_bounds() {
        let (mut b, root, m, run) = broker();
        let t0 = Instant::now();
        let ghost = GrantId::from_trusted("g_nope");
        assert!(matches!(b.arm_deadman(&ghost, P, t0), Err(DeadmanRefusal::Tree(Denial::UnknownNode { .. }))));
        assert_eq!(b.arm_deadman(&m, MIN_PERIOD - Duration::from_millis(1), t0).map(|_| ()), Err(DeadmanRefusal::Period));
        assert_eq!(b.arm_deadman(&m, MAX_PERIOD + Duration::from_millis(1), t0).map(|_| ()), Err(DeadmanRefusal::Period));
        b.revoke(&root, &m).unwrap();
        assert!(matches!(b.arm_deadman(&run, P, t0), Err(DeadmanRefusal::Tree(Denial::Revoked { .. }))), "a revoked ancestor counts");
        assert!(b.next_deadman_deadline().is_none());
    }
}
