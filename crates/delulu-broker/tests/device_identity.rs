//! P17-7 — the `device` dimension has TWO sources of truth for one fact, and they are kept in
//! agreement by hand.
//!
//! `Scopes::device` is a `BTreeMap<String, DeviceScope>`, and the `DeviceScope` value carries its
//! own `device: String` field. Everything that makes an authorization decision reads the **map
//! key**; everything that produces the **signed and audited** bytes reads the **value's field**:
//!
//! | Reads the KEY | Reads the VALUE's `.device` |
//! |---|---|
//! | `device_scope::all_within` — the `⊑` check (`device_scope.rs:276`) | `Authority::to_json` (`authority.rs:78`) |
//! | `device_scope::grants_device` — Actuate enforcement (`device_scope.rs:302`) | `Authority::render_compact` (`authority.rs:103`) |
//! | `device_scope::intersect_device_sets` — the meet (`device_scope.rs:285`) | — and therefore certificate signatures and the hash-chained audit record |
//! | `device_scope::device_names` — the `--json` report (`device_scope.rs:307`) | |
//!
//! Nothing enforces `key == value.device`. It is true at every construction site in the tree today
//! — including the read side, `cert::authority_from_json`, which inserts under `d.device.clone()`
//! (`cert.rs:286`) and therefore re-establishes the invariant on every certificate load.
//!
//! **This is therefore NOT exploitable from outside the process, and this file does not claim it
//! is.** No certificate, audit record or `--json` payload can create the split, because all of them
//! go through the parse. What it is: a latent divergence maintained by convention, in the exact
//! shape this project has been bitten by before — one fact, two hand-maintained places
//! (design rule 1). A future construction site that inserts under the wrong key would make the
//! broker *enforce* one device while the certificate and the audit chain *attest* a different one,
//! and nothing in the build would notice.
//!
//! The test below constructs that state directly and observes the divergence, so the claim is
//! measured rather than argued.

use std::collections::BTreeMap;

use delulu_broker::authority::{Authority, Scopes};
use delulu_broker::device_scope::{self, DeviceScope};

fn scope(spec: &str) -> DeviceScope {
    device_scope::parse(spec).expect("device spec parses")
}

fn scopes_with(key: &str, value: DeviceScope) -> Scopes {
    let mut m: BTreeMap<String, DeviceScope> = BTreeMap::new();
    m.insert(key.to_string(), value);
    Scopes { device: m, ..Default::default() }
}

/// The invariant, stated as a test so it is at least *checkable* even though it is not *enforced*.
/// Every construction site in the tree satisfies it; this pins what "satisfies it" means.
#[test]
fn the_map_key_and_the_inner_device_field_agree_when_built_normally() {
    let d = scope("sat0/hga:angle_deg=-30..95,heartbeat_ms=1000,ttl_ms=5000,fail=hold");
    let s = scopes_with(&d.device.clone(), d);
    for (k, v) in &s.device {
        assert_eq!(k, &v.device, "the map key must be the device the value describes");
    }
}

/// **The divergence, observed.** Enforcement says one device; the signed bytes say another.
#[test]
fn a_mismatched_key_makes_enforcement_and_attestation_disagree() {
    // Built deliberately wrong: the map is keyed "sat0/safe" while the value describes "sat0/arm".
    let inner = scope("sat0/arm:angle_deg=-30..95,heartbeat_ms=1000,ttl_ms=5000,fail=hold");
    let s = scopes_with("sat0/safe", inner);
    let auth = Authority { effects: Default::default(), scopes: s.clone() };

    // ENFORCEMENT reads the key.
    assert!(
        device_scope::grants_device(&s.device, "sat0/safe"),
        "grants_device keys on the map key (device_scope.rs:302)"
    );
    assert!(
        !device_scope::grants_device(&s.device, "sat0/arm"),
        "the value's own field grants nothing — enforcement never reads it"
    );

    // ATTESTATION reads the value's field. This string is what a certificate signature covers and
    // what lands in the hash-chained audit record.
    let json = auth.to_json().to_string();
    assert!(
        json.contains("sat0/arm"),
        "to_json emits the VALUE's device field (authority.rs:78); got {json}"
    );
    assert!(
        !json.contains("sat0/safe"),
        "the key the broker actually enforces on does not appear in the signed bytes at all; \
         got {json}"
    );

    // The two halves name different devices. An operator reading the audit chain would be told
    // `sat0/arm` about a grant the broker enforces as `sat0/safe`.
    //
    // Not reachable from outside the process today: `cert::authority_from_json` re-keys on
    // `d.device` (cert.rs:286), so every certificate load repairs the invariant. This is a latent
    // second source of truth, not a live escalation, and it is recorded as such.
}

/// The read side really does repair it — which is why this is latent rather than exploitable.
/// If this ever stops holding, the finding above becomes reachable from a certificate.
#[test]
fn the_json_read_side_rekeys_on_the_device_field() {
    let inner = scope("sat0/arm:angle_deg=-30..95,heartbeat_ms=1000,ttl_ms=5000,fail=hold");
    let auth = Authority { effects: Default::default(), scopes: scopes_with("sat0/safe", inner) };

    // Round-trip through the canonical JSON, exactly as a certificate does.
    let json = auth.to_json();
    let devices = json["scopes"]["device"].as_array().expect("device array present").clone();
    let mut rebuilt: BTreeMap<String, DeviceScope> = BTreeMap::new();
    for spec in devices {
        let d = scope(spec.as_str().expect("device spec is a string"));
        rebuilt.insert(d.device.clone(), d); // mirrors cert.rs:286
    }

    assert!(
        rebuilt.contains_key("sat0/arm"),
        "after the round trip the map is keyed by the device the value describes"
    );
    assert!(
        !rebuilt.contains_key("sat0/safe"),
        "the bogus key does not survive a certificate load — this is what bounds the severity"
    );
}
