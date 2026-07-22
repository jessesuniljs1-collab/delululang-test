//! RFC 0001 phase F2 — the real signature backend for grant certificates.
//!
//! `delulu-broker` owns the certificate FORMAT and the chain rules but cannot own the
//! cryptography: it depends only on `delulu-diag`/`delulu-check` (crate ruling 1), while
//! `ed25519-dalek` lives in `delulu-runtime` (house rule 5 — cryptography is adopted, never
//! hand-rolled). The broker therefore declares a [`SignatureVerifier`] and this crate — the one
//! place that can see both — supplies it. Same injection idiom as `IdSource`/`ClockSource`.
//!
//! **Only ed25519 is implemented today, deliberately.** `pqc.rs` refuses every post-quantum
//! operation — signing *and verifying* — without `--unstable` (DL1910), because the adopted
//! ML-DSA implementation is neither validated against the official NIST vectors nor independently
//! audited by its authors. Federation inherits that judgement rather than carving an exception:
//! [`Ed25519Verifier::supports`] answers `false` for `ml-dsa-65`, and the chain verifier then
//! REFUSES with DL1418 instead of treating an unverifiable signature as unsigned. The
//! self-describing `alg` field is what makes adding it later a format-compatible change.

use delulu_broker::cert::SignatureVerifier;

/// The algorithm name a certificate must carry for this backend to verify it.
pub const ALG_ED25519: &str = "ed25519";

/// Verifies grant certificates with ed25519, via the runtime's adopted `ed25519-dalek`.
pub struct Ed25519Verifier;

impl SignatureVerifier for Ed25519Verifier {
    fn supports(&self, alg: &str) -> bool {
        alg == ALG_ED25519
    }

    fn verify(&self, alg: &str, msg: &[u8], sig: &[u8]) -> Option<String> {
        if alg != ALG_ED25519 {
            // Fail closed on the "cannot tell" case: an algorithm this backend does not implement
            // is never verified-by-default. The caller turns this into DL1418.
            return None;
        }
        match delulu_runtime::plugin::verify_detached(msg, sig) {
            // `verify_detached` returns the key that ACTUALLY signed, which is what lets the chain
            // verifier reject a valid signature made by a key other than the named issuer.
            delulu_runtime::plugin::SignatureStatus::Valid { signer } => Some(signer),
            _ => None,
        }
    }
}

/// Sign a certificate's bytes with ed25519, producing the detached 96-byte (pubkey ‖ signature)
/// form the verifier above expects. `seed` is the issuer's private key seed.
pub fn sign(seed: &[u8; 32], msg: &[u8]) -> Vec<u8> {
    delulu_runtime::plugin::sign_detached(seed, msg)
}

/// A fresh 128-bit random nonce, hex. Two certificates with otherwise identical fields still
/// fingerprint differently, so re-issuing a grant produces a distinguishable credential rather than
/// one that collides with the certificate it replaces — which matters because a fingerprint is what
/// the single-adoption rule keys on.
pub fn fresh_nonce() -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("OS randomness (getrandom) unavailable");
    b.iter().fold(String::with_capacity(32), |mut s, x| {
        use std::fmt::Write as _;
        let _ = write!(s, "{x:02x}");
        s
    })
}

/// The hex public key corresponding to a private seed — an issuer's identity, as it appears in a
/// certificate's `issuer`/`subject` fields and in a trust-anchor list.
pub fn public_key_hex(seed: &[u8; 32]) -> String {
    // Derived by signing an empty message and reading back the key half rather than duplicating
    // key-derivation here: the point is to have exactly one place that knows the encoding.
    let sig = delulu_runtime::plugin::sign_detached(seed, b"");
    sig[..32].iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use delulu_broker::authority::{Authority, Scopes};
    use delulu_broker::cert::{verify_chain, Certificate, ANCHOR};
    use delulu_broker::device_scope;
    use delulu_check::Effect;
    use std::collections::BTreeSet;

    const WIDE: &str = "sat0/hga:slew_deg=-45..45,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park";
    const NARROW: &str = "sat0/hga:slew_deg=-5..5,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park";

    fn auth(dev: &str) -> Authority {
        let d = device_scope::parse(dev).unwrap();
        Authority::new(
            [Effect::core_from_name("Actuate").unwrap()],
            Scopes { device: [(d.device.clone(), d)].into_iter().collect(), ..Default::default() },
        )
    }

    fn signed(seed: &[u8; 32], subject: &str, parent: &str, a: Authority) -> Certificate {
        let mut c = Certificate {
            alg: ALG_ED25519.into(),
            issuer: public_key_hex(seed),
            subject: subject.into(),
            parent: parent.into(),
            not_before: 0,
            not_after: 10_000,
            nonce: "0011".into(),
            authority: a,
            sig: Vec::new(),
        };
        c.sig = sign(seed, &c.signing_bytes());
        c
    }

    /// The whole point of F2, with REAL cryptography rather than the unit tests' stand-in signer:
    /// a ground station delegates a bounded corridor to a vehicle, the vehicle delegates a narrower
    /// one onward, and a verifier holding only the ground's public key can check the entire chain
    /// offline — no shared secret, no round-trip, nothing but the bytes and an anchor.
    #[test]
    fn a_real_ed25519_chain_verifies_offline_from_an_anchor_alone() {
        let ground = [7u8; 32];
        let vehicle = [9u8; 32];
        let root = signed(&ground, &public_key_hex(&vehicle), ANCHOR, auth(WIDE));
        let leaf = signed(&vehicle, "payload-key", &root.fingerprint(), auth(NARROW));

        let anchors: BTreeSet<String> = [public_key_hex(&ground)].into_iter().collect();
        let got = verify_chain(&[root.clone(), leaf], &anchors, &Ed25519Verifier, 500)
            .expect("a real chain verifies");
        assert_eq!(
            got.scopes.device.get("sat0/hga").and_then(|d| d.dims.get("slew_deg").copied()),
            Some((-5.0, 5.0)),
            "the holder gets the LEAF's corridor"
        );

        // The vehicle's key is NOT an anchor: a chain must be rooted in something we configured,
        // not merely in something that signed correctly.
        let wrong: BTreeSet<String> = [public_key_hex(&vehicle)].into_iter().collect();
        assert!(verify_chain(&[root], &wrong, &Ed25519Verifier, 500).is_err());
    }

    /// Real cryptography, real tampering: flip one byte of the signed authority and the signature
    /// stops verifying. This is what the fake verifier in `cert.rs` cannot prove.
    #[test]
    fn tampering_with_a_real_certificate_breaks_its_real_signature() {
        let ground = [7u8; 32];
        let mut c = signed(&ground, "vehicle-key", ANCHOR, auth(NARROW));
        let anchors: BTreeSet<String> = [public_key_hex(&ground)].into_iter().collect();
        assert!(verify_chain(std::slice::from_ref(&c), &anchors, &Ed25519Verifier, 500).is_ok());

        // Widen the corridor after signing — the classic attack this format exists to stop.
        c.authority = auth(WIDE);
        let err = verify_chain(&[c], &anchors, &Ed25519Verifier, 500).unwrap_err();
        assert_eq!(err.code(), "DL1415", "a tampered certificate does not verify");
    }

    /// Post-quantum inherits `pqc.rs`'s refusal rather than carving an exception: an `ml-dsa-65`
    /// certificate is REFUSED (DL1418), never accepted-as-unsigned.
    #[test]
    fn a_post_quantum_certificate_is_refused_not_silently_accepted() {
        assert!(!Ed25519Verifier.supports("ml-dsa-65"));
        assert!(Ed25519Verifier.verify("ml-dsa-65", b"x", b"y").is_none());
        let ground = [7u8; 32];
        let mut c = signed(&ground, "vehicle-key", ANCHOR, auth(NARROW));
        c.alg = "ml-dsa-65".into();
        let anchors: BTreeSet<String> = [public_key_hex(&ground)].into_iter().collect();
        let err = verify_chain(&[c], &anchors, &Ed25519Verifier, 500).unwrap_err();
        assert_eq!(err.code(), "DL1418");
    }

    /// A grant signature must not be replayable as an artifact signature, or vice versa. Same key,
    /// same underlying bytes, different domain separator → different message, so a signature over
    /// one does not verify over the other.
    #[test]
    fn a_grant_signature_does_not_verify_as_an_artifact_signature() {
        let seed = [7u8; 32];
        let c = signed(&seed, "vehicle-key", ANCHOR, auth(NARROW));
        let body_without_context = {
            let signed_bytes = c.signing_bytes();
            // Everything after "delulu-grant-v1\n" — i.e. what an artifact signer would have signed.
            signed_bytes[delulu_broker::cert::GRANT_CTX.len() + 1..].to_vec()
        };
        assert!(
            !matches!(
                delulu_runtime::plugin::verify_detached(&body_without_context, &c.sig),
                delulu_runtime::plugin::SignatureStatus::Valid { .. }
            ),
            "the grant signature must not verify over the un-separated body"
        );
    }
}
