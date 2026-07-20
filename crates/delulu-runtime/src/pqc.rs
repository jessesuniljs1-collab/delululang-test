//! Stage 10 phase 10i — post-quantum signatures (Track G, spec §8, invariant 51).
//!
//! **Post-quantum, never "quantum-proof."** FIPS 203/204 algorithms are *believed* resistant to
//! known quantum attacks — a judgment about current cryptanalysis, not a proof. The hybrid
//! construction below exists precisely because that judgment is young, and the banned words appear
//! in this file only in the sentence banning them (spec §8.3).
//!
//! Three things live here, and lattice arithmetic is not one of them (spec §8.2, ruling D14e):
//! the **envelope format**, the **policy gates**, and the **tests**. The mathematics is
//! `ml-dsa`/`ml-kem` from RustCrypto, adopted rather than written, because cryptography is never
//! hand-rolled (house rule 5).
//!
//! # Why nothing here is stable yet
//!
//! Ruling D14b: **KAT validation is necessary, not sufficient.** A known-answer test proves an
//! implementation computes the standard's answers; it says nothing about constant-time behaviour,
//! side channels, or conduct under adversarial input — which is what an audit finds. Two facts
//! block stability today and both are published rather than one standing in for the other:
//!
//! 1. The adopted crates state they have **never been independently audited**.
//! 2. Official NIST ACVP vectors are not in hand (the crates test against Wycheproof, a different
//!    corpus with a different purpose).
//!
//! So every post-quantum operation — signing **and verifying** — refuses with **DL1910** unless the
//! caller passes `--unstable`. Verification is gated too, deliberately: verifying is invoking
//! unvalidated cryptography to make a *trust decision*, which is the more dangerous direction, not
//! the safer one.
//!
//! # The envelope
//!
//! ```text
//! dlsig1
//! alg: ed25519,ml-dsa-65
//! ed25519: <hex>
//! ml-dsa-65: <hex>
//! ```
//!
//! Self-describing by design (crypto-agility, §8.2): the envelope NAMES its algorithms, so one can
//! be replaced without a format break. Anything that is not this format is treated as a **legacy
//! 96-byte detached ed25519 signature**, because v1.0 artifacts are signed that way and the
//! stability contract does not bend for a new feature.

use std::collections::BTreeMap;

/// The domain separator every ML-DSA signature here is made under (FIPS 204 `ctx`). It keeps a
/// DeluluLang artifact signature from being replayable as a signature over anything else that
/// happens to use the same key.
const ML_DSA_CTX: &[u8] = b"delulu-artifact-v1";

pub const MAGIC: &str = "dlsig1";
pub const ALG_ED25519: &str = "ed25519";
pub const ALG_ML_DSA_65: &str = "ml-dsa-65";

/// The algorithm ids this build can evaluate. An id outside this set is refused rather than
/// ignored — see [`Verdict::Refused`] and the note on forward compatibility in `verify`.
pub const KNOWN_ALGS: &[&str] = &[ALG_ED25519, ALG_ML_DSA_65];

/// How much a verifier demands (spec §8.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// v1.0 behaviour: a classical signature alone is acceptable.
    AcceptClassical,
    /// Both algorithms must be present AND verify. Classical-only → DL1908.
    HybridRequired,
}

/// A parsed signature envelope: the algorithms it declares, and the bytes for each.
#[derive(Clone, Debug)]
pub struct Envelope {
    /// Declared in the `alg:` line, in order. The declaration is authoritative: a part present
    /// without being declared is a malformed envelope, not a bonus.
    pub declared: Vec<String>,
    pub parts: BTreeMap<String, Vec<u8>>,
    /// True when this came from a bare 96-byte v1.0 signature rather than a `dlsig1` envelope.
    pub legacy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Every algorithm the policy required verified.
    Valid { signer: String, algs: Vec<String> },
    /// A signature was present and did not verify. Always a refusal, whatever the policy.
    Invalid { reason: String },
    /// Refused by policy rather than by mathematics — carries the diagnostic code.
    Refused { code: &'static str, reason: String },
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

/// Parse a signature file. A `dlsig1` envelope is parsed as one; **anything else is treated as a
/// legacy 96-byte detached ed25519 signature**, which is how every v1.0 artifact is signed.
pub fn parse(bytes: &[u8]) -> Result<Envelope, String> {
    let text = String::from_utf8_lossy(bytes);
    if !text.starts_with(MAGIC) {
        // The v1.0 shape: 32-byte public key followed by a 64-byte signature.
        if bytes.len() != 96 {
            return Err(format!(
                "not a `{MAGIC}` envelope and not a 96-byte v1.0 detached signature ({} bytes)",
                bytes.len()
            ));
        }
        let mut parts = BTreeMap::new();
        parts.insert(ALG_ED25519.to_string(), bytes.to_vec());
        return Ok(Envelope { declared: vec![ALG_ED25519.to_string()], parts, legacy: true });
    }
    let mut declared: Vec<String> = Vec::new();
    let mut parts: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once(':').ok_or_else(|| format!("bad envelope line `{line}`"))?;
        let (key, value) = (key.trim(), value.trim());
        if key == "alg" {
            declared = value.split(',').map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect();
            continue;
        }
        let raw = hex_decode(value).ok_or_else(|| format!("`{key}` is not valid hex"))?;
        if parts.insert(key.to_string(), raw).is_some() {
            return Err(format!("`{key}` appears twice in the envelope"));
        }
    }
    if declared.is_empty() {
        return Err("envelope declares no algorithms (`alg:` line missing or empty)".into());
    }
    // THE SKIP BRANCHES, both of them, before any signature is checked.
    //
    // A declared algorithm with no bytes is an envelope claiming a guarantee it does not carry;
    // a part that was never declared is an envelope carrying something it does not claim. Neither
    // is "verify what you can and move on" — a self-describing format that does not match itself
    // has already failed to be self-describing.
    for a in &declared {
        if !parts.contains_key(a) {
            return Err(format!("envelope declares `{a}` but carries no `{a}:` part"));
        }
    }
    for a in parts.keys() {
        if !declared.contains(a) {
            return Err(format!("envelope carries a `{a}:` part it does not declare in `alg:`"));
        }
    }
    Ok(Envelope { declared, parts, legacy: false })
}

pub fn render(parts: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let algs: Vec<&str> = parts.keys().map(String::as_str).collect();
    let mut s = format!("{MAGIC}\nalg: {}\n", algs.join(","));
    for (a, bytes) in parts {
        s.push_str(&format!("{a}: {}\n", hex_lower(bytes)));
    }
    s.into_bytes()
}

/// Sign `data` with ed25519 alone — the v1.0 path, always available, never gated.
pub fn sign_classical(seed: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let mut parts = BTreeMap::new();
    parts.insert(ALG_ED25519.to_string(), crate::plugin::sign_detached(seed, data));
    render(&parts)
}

/// Sign `data` with BOTH algorithms, from one seed (spec §8.2's hybrid).
///
/// Refuses with DL1910 unless `unstable` is set, because the ML-DSA implementation is neither
/// KAT-validated against official vectors nor independently audited (ruling D14b). The refusal is
/// the honest default, not a placeholder.
pub fn sign_hybrid(seed: &[u8; 32], data: &[u8], unstable: bool) -> Result<Vec<u8>, Verdict> {
    if !unstable {
        return Err(Verdict::Refused { code: "DL1910", reason: unvalidated_reason("sign") });
    }
    let mut parts = BTreeMap::new();
    parts.insert(ALG_ED25519.to_string(), crate::plugin::sign_detached(seed, data));
    parts.insert(ALG_ML_DSA_65.to_string(), ml_dsa_sign(seed, data));
    Ok(render(&parts))
}

fn unvalidated_reason(verb: &str) -> String {
    format!(
        "refusing to {verb} with ML-DSA-65: this build's implementation is neither validated \
         byte-exactly against the official NIST vectors nor independently audited (its authors \
         state it has never been audited). Post-quantum operations are available only under \
         `--unstable`, which is a decision to use unvalidated cryptography deliberately"
    )
}

fn ml_dsa_sign(seed: &[u8; 32], data: &[u8]) -> Vec<u8> {
    use ml_dsa::{Keypair, MlDsa65, SigningKey};
    let sk = SigningKey::<MlDsa65>::from_seed(&(*seed).into());
    // The DETERMINISTIC variant (FIPS 204 Algorithm 2's optional path): the same seed and the same
    // bytes must produce the same signature, or a signed artifact stops being reproducible and the
    // release pipeline's byte-identity claim goes with it.
    let sig = sk
        .expanded_key()
        .sign_deterministic(data, ML_DSA_CTX)
        .expect("ML-DSA context string is a fixed 18 bytes, far under the 255-byte limit");
    let vk = sk.verifying_key();
    let mut out = Vec::new();
    out.extend_from_slice(vk.encode().as_slice());
    out.extend_from_slice(sig.encode().as_slice());
    out
}

fn ml_dsa_verify(data: &[u8], blob: &[u8]) -> Result<String, String> {
    use ml_dsa::{EncodedSignature, EncodedVerifyingKey, MlDsa65, Signature, VerifyingKey};
    let vk_len = <EncodedVerifyingKey<MlDsa65> as Default>::default().len();
    let sig_len = <EncodedSignature<MlDsa65> as Default>::default().len();
    if blob.len() != vk_len + sig_len {
        return Err(format!(
            "ml-dsa-65 part is {} bytes, not the expected {} (key {vk_len} + signature {sig_len})",
            blob.len(),
            vk_len + sig_len
        ));
    }
    let vk_bytes = EncodedVerifyingKey::<MlDsa65>::try_from(&blob[..vk_len])
        .map_err(|_| "ml-dsa-65 verifying key is malformed".to_string())?;
    let sig_bytes = EncodedSignature::<MlDsa65>::try_from(&blob[vk_len..])
        .map_err(|_| "ml-dsa-65 signature is malformed".to_string())?;
    let vk = VerifyingKey::<MlDsa65>::decode(&vk_bytes);
    let sig = Signature::<MlDsa65>::decode(&sig_bytes)
        .ok_or_else(|| "ml-dsa-65 signature is malformed".to_string())?;
    if vk.verify_with_context(data, ML_DSA_CTX, &sig) {
        Ok(hex_lower(&blob[..vk_len]))
    } else {
        Err("the ml-dsa-65 signature does not verify over these bytes".to_string())
    }
}

/// Verify a signature file against `data` under `policy`.
///
/// **An unknown algorithm id is refused under BOTH policies** (DL1908), not ignored. The cost is
/// stated rather than discovered: adding a new algorithm requires verifiers to be updated *before*
/// signers start using it — announce, then adopt. That is the safe rollout order anyway, and the
/// alternative is a verifier that shrugs at a claim it cannot evaluate, which is exactly the
/// "when the checker cannot tell, it says yes" failure this project refuses everywhere else.
pub fn verify(data: &[u8], sig_file: &[u8], policy: Policy, unstable: bool) -> Verdict {
    let env = match parse(sig_file) {
        Ok(e) => e,
        Err(reason) => return Verdict::Invalid { reason },
    };
    for a in &env.declared {
        if !KNOWN_ALGS.contains(&a.as_str()) {
            return Verdict::Refused {
                code: "DL1908",
                reason: format!(
                    "signature envelope declares algorithm `{a}`, which this build cannot \
                     evaluate. An unevaluable claim is refused rather than skipped"
                ),
            };
        }
    }
    let has_pq = env.parts.contains_key(ALG_ML_DSA_65);
    if policy == Policy::HybridRequired && !has_pq {
        return Verdict::Refused {
            code: "DL1908",
            reason: format!(
                "policy requires a hybrid signature, but this artifact carries {} only",
                if env.legacy { "a v1.0 classical signature" } else { "classical algorithms" }
            ),
        };
    }
    // The classical half first: it is validated, audited, and cheap. If it fails there is no
    // reason to invoke unvalidated lattice code at all.
    let Some(ed) = env.parts.get(ALG_ED25519) else {
        return Verdict::Refused {
            code: "DL1908",
            reason: "envelope carries no ed25519 part — hybrid is never PQ-only (spec §8.2)".into(),
        };
    };
    let signer = match crate::plugin::verify_detached(data, ed) {
        crate::plugin::SignatureStatus::Valid { signer } => signer,
        crate::plugin::SignatureStatus::Invalid { reason } => return Verdict::Invalid { reason },
        crate::plugin::SignatureStatus::Unsigned => {
            return Verdict::Invalid { reason: "ed25519 part is empty".into() }
        }
    };
    let mut algs = vec![ALG_ED25519.to_string()];
    if has_pq {
        // Gated on BOTH sides: verifying is invoking unvalidated cryptography to make a trust
        // decision, which is the more dangerous direction, not the safer one.
        if !unstable {
            return Verdict::Refused { code: "DL1910", reason: unvalidated_reason("verify") };
        }
        match ml_dsa_verify(data, &env.parts[ALG_ML_DSA_65]) {
            Ok(_pq_signer) => algs.push(ALG_ML_DSA_65.to_string()),
            Err(reason) => return Verdict::Invalid { reason },
        }
    }
    Verdict::Valid { signer, algs }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [42u8; 32];
    const DATA: &[u8] = b"the bytes a signature is a claim about";

    /// The v1.0 path is untouched: classical signing round-trips under the default policy.
    #[test]
    fn a_classical_signature_verifies_under_the_default_policy() {
        let sig = sign_classical(&SEED, DATA);
        match verify(DATA, &sig, Policy::AcceptClassical, false) {
            Verdict::Valid { algs, .. } => assert_eq!(algs, vec![ALG_ED25519]),
            other => panic!("{other:?}"),
        }
    }

    /// **The stability-contract test.** Every v1.0 artifact — plugins, the signed release — carries
    /// a bare 96-byte detached ed25519 signature. A new envelope format that could not read those
    /// would silently invalidate everything already signed, which is not a feature, it is a break.
    #[test]
    fn a_bare_96_byte_v1_signature_still_verifies() {
        let legacy = crate::plugin::sign_detached(&SEED, DATA);
        assert_eq!(legacy.len(), 96);
        let env = parse(&legacy).expect("a v1.0 signature is still a signature");
        assert!(env.legacy, "it must be recognised AS legacy, not guessed at");
        match verify(DATA, &legacy, Policy::AcceptClassical, false) {
            Verdict::Valid { algs, .. } => assert_eq!(algs, vec![ALG_ED25519]),
            other => panic!("{other:?}"),
        }
    }

    /// DL1910 on the SIGNING side: the implementation is unvalidated, so producing a post-quantum
    /// signature is a deliberate act, not a default.
    #[test]
    fn hybrid_signing_is_refused_without_unstable() {
        match sign_hybrid(&SEED, DATA, false) {
            Err(Verdict::Refused { code, reason }) => {
                assert_eq!(code, "DL1910");
                assert!(reason.contains("never been audited") || reason.contains("audited"), "{reason}");
            }
            other => panic!("expected DL1910, got {other:?}"),
        }
    }

    /// DL1910 on the VERIFYING side too, and this is the direction that matters more: verifying is
    /// invoking unvalidated cryptography to make a TRUST DECISION. A build that refused to sign but
    /// happily verified would have gated the safe half and left the dangerous half open.
    #[test]
    fn verifying_a_hybrid_signature_is_also_refused_without_unstable() {
        let sig = sign_hybrid(&SEED, DATA, true).expect("signing under --unstable");
        match verify(DATA, &sig, Policy::AcceptClassical, false) {
            Verdict::Refused { code, .. } => assert_eq!(code, "DL1910"),
            other => panic!("expected DL1910 on the verify side, got {other:?}"),
        }
    }

    /// The whole point, once a human has opted in: both algorithms present, both verified.
    #[test]
    fn a_hybrid_signature_verifies_both_algorithms_under_unstable() {
        let sig = sign_hybrid(&SEED, DATA, true).expect("signing under --unstable");
        match verify(DATA, &sig, Policy::HybridRequired, true) {
            Verdict::Valid { algs, .. } => {
                assert_eq!(algs, vec![ALG_ED25519.to_string(), ALG_ML_DSA_65.to_string()]);
            }
            other => panic!("{other:?}"),
        }
    }

    /// DL1908: hybrid-required policy against a classical-only artifact. Both spellings of
    /// "classical-only" are tested — the new envelope with one algorithm, and a v1.0 signature —
    /// because a policy that caught only the modern spelling would wave through every existing
    /// artifact, which is exactly the population it most needs to catch.
    #[test]
    fn hybrid_required_refuses_classical_only_in_both_of_its_spellings() {
        for sig in [sign_classical(&SEED, DATA), crate::plugin::sign_detached(&SEED, DATA)] {
            match verify(DATA, &sig, Policy::HybridRequired, true) {
                Verdict::Refused { code, reason } => {
                    assert_eq!(code, "DL1908");
                    assert!(reason.contains("hybrid"), "{reason}");
                }
                other => panic!("{other:?}"),
            }
        }
    }

    /// An algorithm id this build cannot evaluate is REFUSED, not skipped — under both policies.
    /// A verifier that shrugs at a claim it cannot check is the "when the checker cannot tell, it
    /// says yes" failure, wearing a crypto-agility costume.
    #[test]
    fn an_unknown_algorithm_id_is_refused_under_every_policy() {
        let mut parts = BTreeMap::new();
        parts.insert(ALG_ED25519.to_string(), crate::plugin::sign_detached(&SEED, DATA));
        parts.insert("falcon-1024".to_string(), vec![1, 2, 3]);
        let sig = render(&parts);
        for policy in [Policy::AcceptClassical, Policy::HybridRequired] {
            match verify(DATA, &sig, policy, true) {
                Verdict::Refused { code, reason } => {
                    assert_eq!(code, "DL1908", "{policy:?}");
                    assert!(reason.contains("falcon-1024"), "the refusal names it: {reason}");
                }
                other => panic!("{policy:?}: {other:?}"),
            }
        }
    }

    /// THE SKIP BRANCHES. An envelope that declares an algorithm it does not carry, or carries one
    /// it does not declare, is malformed — not "verify the parts that happen to line up". A
    /// self-describing format that does not match itself has failed at being self-describing, and
    /// the first of these is how a signature could claim a post-quantum guarantee it never had.
    #[test]
    fn an_envelope_that_does_not_match_its_own_declaration_is_refused() {
        let ed = hex_lower(&crate::plugin::sign_detached(&SEED, DATA));

        let declares_more_than_it_carries =
            format!("{MAGIC}\nalg: {ALG_ED25519},{ALG_ML_DSA_65}\n{ALG_ED25519}: {ed}\n");
        let err = parse(declares_more_than_it_carries.as_bytes()).expect_err("must not parse");
        assert!(err.contains("carries no"), "{err}");

        let carries_more_than_it_declares =
            format!("{MAGIC}\nalg: {ALG_ED25519}\n{ALG_ED25519}: {ed}\n{ALG_ML_DSA_65}: aabb\n");
        let err = parse(carries_more_than_it_declares.as_bytes()).expect_err("must not parse");
        assert!(err.contains("does not declare"), "{err}");
    }

    /// Tampered bytes fail, which is the only reason any of the above matters.
    #[test]
    fn a_signature_does_not_verify_over_different_bytes() {
        let sig = sign_hybrid(&SEED, DATA, true).expect("signing");
        match verify(b"different bytes entirely", &sig, Policy::HybridRequired, true) {
            Verdict::Invalid { .. } => {}
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    /// Deterministic signing: the same seed over the same bytes produces byte-identical output.
    /// Stage 9's release pipeline claims a byte-identical signed artifact; a randomised signature
    /// would quietly retire that claim.
    #[test]
    fn hybrid_signing_is_deterministic() {
        let a = sign_hybrid(&SEED, DATA, true).expect("signing");
        let b = sign_hybrid(&SEED, DATA, true).expect("signing");
        assert_eq!(a, b, "the same seed and bytes must give the same signature");
    }

    /// **Official NIST ACVP known-answer vectors** — not Wycheproof, not this crate's own
    /// round-trip, not hand-generated. `testdata/nist_acvp_ml_dsa_65_keygen.json` holds three
    /// ML-DSA-65 keyGen test cases pulled from NIST's own `usnistgov/ACVP-Server` repository
    /// (`gen-val/json-files/ML-DSA-keyGen-FIPS204`); full source URLs, git blob hashes, and the
    /// rest of the KAT search are recorded in `measurements/pqc/KAT_RECORD.md`.
    ///
    /// This checks the half that is actually checkable against this crate's public API:
    /// `SigningKey::from_seed(seed).verifying_key()` — exactly what `ml_dsa_sign` computes
    /// internally before it ever signs anything — must reproduce NIST's own `pk` byte-for-byte.
    /// It deliberately does NOT check `sk` (`ml-dsa` 0.1.1 exposes no public encoder for the raw
    /// FIPS-204 secret-key layout, so there is nothing to compare it against) or a signature
    /// (ACVP's sigGen vectors hand you an already-expanded `sk`, which this crate also has no way
    /// to decode from raw bytes). Both gaps are named here and in the KAT record, not papered over
    /// — this test is evidence about key derivation only, not a full KAT clearance for signing.
    #[test]
    fn nist_acvp_ml_dsa_65_keygen_seed_to_pk_matches() {
        use ml_dsa::{EncodedVerifyingKey, MlDsa65};

        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../testdata/nist_acvp_ml_dsa_65_keygen.json"))
                .expect("fixture is valid JSON");
        let vk_len = <EncodedVerifyingKey<MlDsa65> as Default>::default().len();
        let vectors = fixture["vectors"].as_array().expect("fixture carries a `vectors` array");
        assert_eq!(vectors.len(), 3, "expected the three vectors this fixture was built with");

        for v in vectors {
            let tc_id = v["tcId"].as_i64().expect("tcId is an integer");
            let seed_bytes = hex_decode(v["seed"].as_str().expect("seed is a string"))
                .expect("seed is valid hex");
            let seed: [u8; 32] =
                seed_bytes.try_into().expect("NIST ACVP ML-DSA seed is 32 bytes");
            let expected_pk = v["expected_pk"].as_str().expect("expected_pk is a string").to_lowercase();

            // Calls the actual pqc.rs signing function — not a hand-rolled re-derivation of what
            // it does. The verifying key occupies the first `vk_len` bytes of its output blob.
            let blob = ml_dsa_sign(&seed, b"nist-acvp-kat-check");
            let got_pk = hex_lower(&blob[..vk_len]);

            assert_eq!(
                got_pk, expected_pk,
                "NIST ACVP ML-DSA-65 keyGen tcId {tc_id}: pk derived via ml_dsa_sign's key \
                 derivation does not match the official vector"
            );
        }
    }
}
