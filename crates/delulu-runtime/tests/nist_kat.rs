//! The NIST ACVP known-answer vectors for ML-DSA-65 and ML-KEM-768, byte-exact (RW 4.10a, V2 P7).
//!
//! Invariant 51 (spec §8.2): a PQC implementation reaches stable only after byte-exact validation
//! against the official known-answer vectors, with their provenance recorded. The vectors were fetched
//! from NIST's own `usnistgov/ACVP-Server` on 2026-07-20 and saved under `measurements/pqc/vectors/`,
//! each file carrying its `_provenance` (URL and git blob SHA1 of every source file) —
//! `measurements/pqc/KAT_RECORD.md`. Until this file, no test consumed them (one ML-DSA keyGen subset
//! lived in `pqc.rs`'s own tests).
//!
//! Every saved case the pinned crates' public API can express is checked, and the ones it cannot are
//! NAMED, not skipped in silence:
//! - HashML-DSA (`preHash: "preHash"`, sigGen and sigVer): `ml-dsa` 0.1.1 has no pre-hash API; this
//!   project never signs a pre-hash, so nothing it does rests on those groups.
//! - randomized signing over a precomputed μ (sigGen group with `externalMu` AND `rnd`): exposed only
//!   through an RNG argument.
//!
//! This does NOT make PQC stable. Ruling D14b's second gate — an independent audit of the crates, which
//! this project cannot perform — still stands, and every PQC operation still refuses without
//! `--unstable` (DL1910).

use ml_dsa::{EncodedSignature, EncodedVerifyingKey, ExpandedSigningKey, Keypair, MlDsa65, Signature, SigningKey, VerifyingKey};
// `ExpandedKeyEncoding` is deprecated in the crate (it prefers seeds); the NIST answer key IS the
// expanded encoding, so it is the only honest comparison.
#[allow(deprecated)]
use ml_kem::ExpandedKeyEncoding;
use ml_kem::{Decapsulate, DecapsulationKey, EncapsulationKey, KeyExport, MlKem768};
use serde_json::Value;

fn hex(s: &str) -> Vec<u8> {
    assert!(s.len().is_multiple_of(2), "odd hex length");
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex")).collect()
}

fn vectors(text: &str) -> Value {
    let v: Value = serde_json::from_str(text).expect("the saved vectors parse");
    let prov = &v["_provenance"];
    assert_eq!(prov["official_source_repo"], "https://github.com/usnistgov/ACVP-Server", "provenance is NIST's");
    assert!(prov["source_files"]["expectedResults.json"]["git_blob_sha1"].is_string(), "the answer key names its blob");
    v
}

fn field<'a>(t: &'a Value, k: &str) -> &'a str {
    t[k].as_str().unwrap_or_else(|| panic!("case {} has no `{k}`", t["tcId"]))
}

/// Counts what was checked and what was named-and-not-checked, so a test that quietly checked
/// nothing cannot pass.
#[derive(Default)]
struct Tally {
    checked: usize,
    named: Vec<String>,
}

#[test]
#[allow(deprecated)] // `to_expanded`: the NIST keyGen answer key IS the expanded encoding.
fn ml_dsa_65_keygen_matches_nist_byte_for_byte() {
    let v = vectors(include_str!("../../../measurements/pqc/vectors/ML-DSA-keyGen-FIPS204/sample_vectors_subset.json"));
    let mut n = 0;
    for g in v["testGroups"].as_array().unwrap() {
        assert_eq!(g["parameterSet"], "ML-DSA-65");
        for t in g["tests"].as_array().unwrap() {
            let seed = hex(field(t, "seed"));
            let sk = SigningKey::<MlDsa65>::from_seed(&seed.as_slice().try_into().unwrap());
            assert_eq!(sk.verifying_key().encode().as_slice(), hex(field(t, "expected_pk")), "tcId {}: pk", t["tcId"]);
            assert_eq!(sk.expanded_key().to_expanded().as_slice(), hex(field(t, "expected_sk")), "tcId {}: sk", t["tcId"]);
            n += 1;
        }
    }
    assert_eq!(n, 5, "every saved keyGen case was checked");
}

#[test]
#[allow(deprecated)] // `from_expanded`: the NIST sigGen prompt gives the key in its expanded encoding.
fn ml_dsa_65_siggen_matches_nist_byte_for_byte() {
    let v = vectors(include_str!("../../../measurements/pqc/vectors/ML-DSA-sigGen-FIPS204/sample_vectors_subset.json"));
    let mut tally = Tally::default();
    for g in v["testGroups"].as_array().unwrap() {
        assert_eq!(g["parameterSet"], "ML-DSA-65");
        let (tg, det, iface) = (&g["tgId"], g["deterministic"] == true, g["signatureInterface"].as_str().unwrap());
        let pre = g["preHash"].as_str() == Some("preHash");
        let mu = g["externalMu"] == true;
        if pre || (mu && !det) {
            tally.named.push(format!("tgId {tg} ({})", if pre { "HashML-DSA" } else { "randomized over μ" }));
            continue;
        }
        for t in g["tests"].as_array().unwrap() {
            let esk = ExpandedSigningKey::<MlDsa65>::from_expanded(&hex(field(t, "sk")).as_slice().try_into().unwrap());
            let rnd: ml_dsa::B32 = if det {
                Default::default()
            } else {
                hex(field(t, "rnd")).as_slice().try_into().unwrap()
            };
            let sig: Signature<MlDsa65> = match (iface, mu) {
                ("internal", true) => esk.sign_mu_deterministic(hex(field(t, "mu")).as_slice().try_into().unwrap()),
                ("internal", false) => esk.sign_internal(&[&hex(field(t, "message"))], &rnd),
                ("external", _) if det => {
                    esk.sign_deterministic(&hex(field(t, "message")), &hex(field(t, "context"))).expect("context ≤ 255")
                }
                ("external", _) => {
                    // FIPS 204 Algorithm 2: M' = 0 ‖ |ctx| ‖ ctx ‖ M, then Sign_internal with rnd.
                    let ctx = hex(field(t, "context"));
                    let prefix = [0u8, ctx.len() as u8];
                    esk.sign_internal(&[&prefix, &ctx, &hex(field(t, "message"))], &rnd)
                }
                other => panic!("unknown interface {other:?}"),
            };
            assert_eq!(sig.encode().as_slice(), hex(field(t, "expected_signature")), "tgId {tg} tcId {}", t["tcId"]);
            tally.checked += 1;
        }
    }
    assert_eq!(tally.checked, 15, "checked: deterministic external, both internal forms, randomized external and internal");
    assert_eq!(tally.named.len(), 3, "named and not checked: {:?}", tally.named);
}

#[test]
fn ml_dsa_65_sigver_matches_nist() {
    let v = vectors(include_str!("../../../measurements/pqc/vectors/ML-DSA-sigVer-FIPS204/sample_vectors_subset.json"));
    let mut tally = Tally::default();
    let (mut accepted, mut rejected) = (0, 0);
    for g in v["testGroups"].as_array().unwrap() {
        assert_eq!(g["parameterSet"], "ML-DSA-65");
        let tg = &g["tgId"];
        if g["preHash"].as_str() == Some("preHash") {
            tally.named.push(format!("tgId {tg} (HashML-DSA)"));
            continue;
        }
        let (iface, mu) = (g["signatureInterface"].as_str().unwrap(), g["externalMu"] == true);
        for t in g["tests"].as_array().unwrap() {
            let pk: EncodedVerifyingKey<MlDsa65> = hex(field(t, "pk")).as_slice().try_into().unwrap();
            let vk = VerifyingKey::<MlDsa65>::decode(&pk);
            let sig_bytes = hex(field(t, "signature"));
            // A signature that does not even decode is a rejection, as FIPS 204's sigDecode says.
            let passed = match EncodedSignature::<MlDsa65>::try_from(sig_bytes.as_slice())
                .ok()
                .and_then(|e| Signature::<MlDsa65>::decode(&e))
            {
                None => false,
                Some(sig) => match (iface, mu) {
                    ("internal", true) => vk.verify_mu(hex(field(t, "mu")).as_slice().try_into().unwrap(), &sig),
                    ("internal", false) => vk.verify_internal(&hex(field(t, "message")), &sig),
                    ("external", _) => vk.verify_with_context(&hex(field(t, "message")), &hex(field(t, "context")), &sig),
                    other => panic!("unknown interface {other:?}"),
                },
            };
            let expected = t["expected_testPassed"].as_bool().expect("expected_testPassed");
            assert_eq!(passed, expected, "tgId {tg} tcId {}", t["tcId"]);
            if expected {
                accepted += 1;
            } else {
                rejected += 1;
            }
            tally.checked += 1;
        }
    }
    assert_eq!(tally.checked, 9, "every non-pre-hash sigVer case");
    assert!(accepted > 0 && rejected > 0, "the saved cases include both verdicts ({accepted} pass, {rejected} fail)");
    assert_eq!(tally.named.len(), 1, "named and not checked: {:?}", tally.named);
}

#[test]
#[allow(deprecated)] // `to_expanded_bytes`: the NIST keyGen answer key is the expanded dk.
fn ml_kem_768_keygen_matches_nist_byte_for_byte() {
    let v = vectors(include_str!("../../../measurements/pqc/vectors/ML-KEM-keyGen-FIPS203/sample_vectors_subset.json"));
    let mut n = 0;
    for g in v["testGroups"].as_array().unwrap() {
        assert_eq!(g["parameterSet"], "ML-KEM-768");
        for t in g["tests"].as_array().unwrap() {
            // FIPS 203's seed for KeyGen_internal is d ‖ z.
            let mut seed = hex(field(t, "d"));
            seed.extend(hex(field(t, "z")));
            let dk = DecapsulationKey::<MlKem768>::from_seed(seed.as_slice().try_into().unwrap());
            assert_eq!(dk.encapsulation_key().to_bytes().as_slice(), hex(field(t, "expected_ek")), "tcId {}: ek", t["tcId"]);
            assert_eq!(dk.to_expanded_bytes().as_slice(), hex(field(t, "expected_dk")), "tcId {}: dk", t["tcId"]);
            n += 1;
        }
    }
    assert_eq!(n, 5, "every saved keyGen case was checked");
}

#[test]
#[allow(deprecated)] // `from_expanded`: the NIST decapsulation prompt gives dk in its expanded encoding.
fn ml_kem_768_encaps_decaps_and_key_checks_match_nist() {
    let v = vectors(include_str!("../../../measurements/pqc/vectors/ML-KEM-encapDecap-FIPS203/sample_vectors_subset.json"));
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    for g in v["testGroups"].as_array().unwrap() {
        assert_eq!(g["parameterSet"], "ML-KEM-768");
        let function = g["function"].as_str().unwrap().to_string();
        for t in g["tests"].as_array().unwrap() {
            let tc = &t["tcId"];
            match function.as_str() {
                "encapsulation" => {
                    let ek = EncapsulationKey::<MlKem768>::new(&hex(field(t, "ek")).as_slice().try_into().unwrap())
                        .expect("a NIST encapsulation key is valid");
                    let (c, k) = ek.encapsulate_deterministic(&hex(field(t, "m")).as_slice().try_into().unwrap());
                    assert_eq!(c.as_slice(), hex(field(t, "expected_c")), "tcId {tc}: c");
                    assert_eq!(k.as_slice(), hex(field(t, "expected_k")), "tcId {tc}: k");
                }
                "decapsulation" => {
                    let dk = DecapsulationKey::<MlKem768>::from_expanded(&hex(field(t, "dk")).as_slice().try_into().unwrap())
                        .expect("a NIST decapsulation key is valid");
                    let c: ml_kem::Ciphertext<MlKem768> = hex(field(t, "c")).as_slice().try_into().unwrap();
                    let k = dk.decapsulate(&c);
                    assert_eq!(k.as_slice(), hex(field(t, "expected_k")), "tcId {tc}: k (incl. implicit rejection)");
                }
                "encapsulationKeyCheck" => {
                    let ok = hex(field(t, "ek"))
                        .as_slice()
                        .try_into()
                        .ok()
                        .is_some_and(|ek| EncapsulationKey::<MlKem768>::new(&ek).is_ok());
                    assert_eq!(ok, t["expected_testPassed"].as_bool().unwrap(), "tcId {tc}: ek check");
                }
                "decapsulationKeyCheck" => {
                    let ok = hex(field(t, "dk"))
                        .as_slice()
                        .try_into()
                        .ok()
                        .is_some_and(|dk| DecapsulationKey::<MlKem768>::from_expanded(&dk).is_ok());
                    assert_eq!(ok, t["expected_testPassed"].as_bool().unwrap(), "tcId {tc}: dk check");
                }
                other => panic!("unknown function {other}"),
            }
            *seen.entry(function.clone()).or_default() += 1;
        }
    }
    assert_eq!(seen.values().sum::<usize>(), 12, "every saved case: {seen:?}");
    assert_eq!(seen.len(), 4, "all four functions exercised: {seen:?}");
}
