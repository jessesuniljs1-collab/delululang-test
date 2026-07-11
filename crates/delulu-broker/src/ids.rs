//! GrantId sources (ruling 3: determinism injection).
//!
//! Id generation sits behind a tiny trait with an OS-backed default; tests inject a deterministic
//! counter source so tree outcomes are reproducible (criterion 9).

use std::fmt::Write;

use crate::tree::GrantId;

/// A source of opaque, unguessable grant handles.
pub trait IdSource {
    fn next_id(&mut self) -> GrantId;
}

/// The production source: 16 bytes of OS randomness → `g_` + 32 hex chars. Unguessable handle.
pub struct OsIdSource;

impl IdSource for OsIdSource {
    fn next_id(&mut self) -> GrantId {
        let mut buf = [0u8; 16];
        getrandom::fill(&mut buf).expect("OS randomness (getrandom) unavailable");
        let mut s = String::with_capacity(34);
        s.push_str("g_");
        for b in buf {
            let _ = write!(s, "{b:02x}");
        }
        GrantId(s)
    }
}

/// A deterministic counter source for tests and any reproducible/replay mode: `g_` + the counter
/// as 32 zero-padded hex digits. Still a well-shaped GrantId, just predictable.
#[derive(Default)]
pub struct SeqIdSource {
    next: u64,
}

impl SeqIdSource {
    pub fn new() -> SeqIdSource {
        SeqIdSource::default()
    }

    /// Start the counter at a chosen base (so two brokers can be given disjoint id spaces).
    pub fn starting_at(n: u64) -> SeqIdSource {
        SeqIdSource { next: n }
    }
}

impl IdSource for SeqIdSource {
    fn next_id(&mut self) -> GrantId {
        let n = self.next;
        self.next += 1;
        GrantId(format!("g_{n:032x}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_ids_are_well_shaped_and_distinct() {
        let mut src = OsIdSource;
        let a = src.next_id();
        let b = src.next_id();
        assert_ne!(a, b, "OS ids should not collide");
        for id in [&a, &b] {
            assert!(id.as_str().starts_with("g_"));
            assert_eq!(id.as_str().len(), 34);
            assert!(id.as_str()[2..].chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn seq_ids_are_deterministic() {
        let mut src = SeqIdSource::new();
        assert_eq!(src.next_id().as_str(), "g_00000000000000000000000000000000");
        assert_eq!(src.next_id().as_str(), "g_00000000000000000000000000000001");
    }
}
