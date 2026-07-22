//! RFC 0001 phase F2 — the **grant certificate**: an offline, self-describing, chain-verifiable
//! delegation.
//!
//! # Why the lease token could not be stretched to do this
//!
//! [`crate::lease::Token`] is `dlt1_<payload>.<mac>` where the payload is `{v, node, exp_millis,
//! multi, nonce}` — **a reference into the minting broker's tree, carrying no authority bytes** —
//! MAC'd with `blake3::keyed_hash` under that broker's own **symmetric** key. Both properties make
//! it local by construction: a remote party has nothing to look up, and could only verify the MAC
//! by holding a key that would also let it *mint*. Sharing that key is not delegation; it is making
//! both parties able to forge as each other, which destroys the asymmetry a grant tree means.
//!
//! A certificate differs on exactly those two axes and no others:
//!
//! 1. **It carries its authority**, so a verifier with no shared state can read what is granted.
//! 2. **It is asymmetrically signed**, so a party that can verify it cannot mint it.
//!
//! # Federation introduces no new authority mathematics
//!
//! A chain is verified by running the EXISTING [`crate::authority::attenuation_check`] at every
//! hop. There is deliberately no second `⊑` implementation here — a second one is a second place
//! for the rule to die. This module contributes format, chain-walking, and refusals; the lattice is
//! untouched.
//!
//! # Cryptography is injected, never implemented here
//!
//! `delulu-broker` depends only on `delulu-diag`/`delulu-check` (crate ruling 1), so it cannot see
//! `ed25519-dalek`, which lives in `delulu-runtime` (house rule 5: cryptography is never
//! hand-rolled). The signature primitive is therefore injected as a [`SignatureVerifier`], the same
//! way `IdSource` and `ClockSource` already are (ruling 3). This module computes **what** must be
//! true; the caller supplies **who can check a signature**.
//!
//! # The wire form
//!
//! ```text
//! dlcert1
//! alg: ed25519
//! issuer: <hex pubkey>
//! subject: <hex pubkey>
//! parent: <hex hash of the issuing certificate, or "anchor">
//! not_before: <epoch millis>
//! not_after: <epoch millis>
//! nonce: <hex>
//! authority: <canonical JSON, one line>
//! sig: <hex>
//! ```
//!
//! Versioned magic (`dlcert1`) and a **self-describing algorithm field** from the first byte,
//! because a deployed vehicle's credentials can only be re-issued across the link that made them
//! necessary — the worst upgrade environment there is (RFC §6).
//!
//! # Domain separation is mandatory, not decorative
//!
//! `delulu-runtime`'s `pqc.rs` signs *artifacts* under the context `delulu-artifact-v1`. A grant
//! certificate is signed under [`GRANT_CTX`] instead. Without a distinct context, a signature over
//! one would be replayable as a signature over the other — an artifact signature becoming a grant
//! of authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::audit::canonical_json;
use crate::authority::{attenuation_check, Authority, Scopes};
use crate::device_scope;
use crate::diag::Denial;

/// The wire magic. A future incompatible format is `dlcert2`, never a silent reinterpretation.
pub const MAGIC: &str = "dlcert1";

/// The domain separator every grant-certificate signature is made under. **Distinct from
/// `pqc.rs`'s `delulu-artifact-v1` by construction** — see the module docs.
pub const GRANT_CTX: &[u8] = b"delulu-grant-v1";

/// The `parent` value of a certificate issued directly by a trust anchor.
pub const ANCHOR: &str = "anchor";

/// Verifies a detached signature. Injected because this crate cannot depend on the crate that owns
/// the cryptography (module docs).
///
/// Implementors must be **fail-closed on every "cannot tell"**: an unknown algorithm, a malformed
/// key, or a malformed signature returns `None`/`false`, never a permissive default.
pub trait SignatureVerifier {
    /// Does this verifier actually implement `alg`? An algorithm it does not know must be answered
    /// `false` so the caller can REFUSE — never skipped as if unsigned.
    fn supports(&self, alg: &str) -> bool;

    /// Verify `sig` over `msg` under `alg`. Returns the **hex-encoded public key that actually
    /// signed**, so the caller can check it is the key the certificate NAMES. Returning the signer
    /// rather than a bool is deliberate: "this signature is valid" and "the named issuer signed
    /// this" are different claims, and conflating them accepts a validly-signed forgery.
    fn verify(&self, alg: &str, msg: &[u8], sig: &[u8]) -> Option<String>;
}

/// One signed delegation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certificate {
    pub alg: String,
    /// Hex public key of the party that signed this certificate.
    pub issuer: String,
    /// Hex public key of the party this authority is delegated TO.
    pub subject: String,
    /// [`ANCHOR`], or the hex [`Certificate::fingerprint`] of the certificate that issued this one.
    pub parent: String,
    pub not_before: i64,
    pub not_after: i64,
    pub nonce: String,
    pub authority: Authority,
    /// The detached signature bytes over [`Certificate::signing_bytes`].
    pub sig: Vec<u8>,
}

impl Certificate {
    /// The exact bytes a signature is made over: the domain separator, then the canonical body.
    ///
    /// The `sig` field is excluded (it cannot sign itself) and every other field is included, so no
    /// field can be altered after signing without invalidating it — including `alg` itself, which
    /// stops an attacker downgrading a certificate to a weaker algorithm the verifier also accepts.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::from(GRANT_CTX);
        out.push(b'\n');
        out.extend_from_slice(canonical_json(&self.body_value()).as_bytes());
        out
    }

    /// A certificate's identity: `blake3` over its signing bytes plus its signature, hex. Used as
    /// the `parent` link of the next certificate down the chain.
    pub fn fingerprint(&self) -> String {
        let mut h = blake3::Hasher::new();
        h.update(&self.signing_bytes());
        h.update(&self.sig);
        h.finalize().to_hex().to_string()
    }

    fn body_value(&self) -> serde_json::Value {
        serde_json::json!({
            "alg": self.alg,
            "authority": self.authority.to_json(),
            "issuer": self.issuer,
            "not_after": self.not_after,
            "not_before": self.not_before,
            "nonce": self.nonce,
            "parent": self.parent,
            "subject": self.subject,
        })
    }

    /// Render to the wire form. Round-trips with [`parse`].
    pub fn to_wire(&self) -> String {
        let mut s = String::new();
        s.push_str(MAGIC);
        s.push('\n');
        s.push_str(&format!("alg: {}\n", self.alg));
        s.push_str(&format!("issuer: {}\n", self.issuer));
        s.push_str(&format!("subject: {}\n", self.subject));
        s.push_str(&format!("parent: {}\n", self.parent));
        s.push_str(&format!("not_before: {}\n", self.not_before));
        s.push_str(&format!("not_after: {}\n", self.not_after));
        s.push_str(&format!("nonce: {}\n", self.nonce));
        s.push_str(&format!("authority: {}\n", canonical_json(&self.authority.to_json())));
        s.push_str(&format!("sig: {}\n", to_hex(&self.sig)));
        s
    }
}

/// Parse the wire form. Every failure is [`Denial::CertMalformed`] — fail-closed, and a certificate
/// this build cannot fully understand is refused rather than partially honored.
pub fn parse(text: &str) -> Result<Certificate, Denial> {
    let bad = |m: &str| Denial::CertMalformed { detail: m.to_string() };
    let mut lines = text.lines();
    match lines.next().map(str::trim) {
        Some(MAGIC) => {}
        Some(other) => return Err(bad(&format!("not a grant certificate (magic `{other}`)"))),
        None => return Err(bad("empty input")),
    }
    let mut f: BTreeMap<&str, &str> = BTreeMap::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line.split_once(':').ok_or_else(|| bad(&format!("bad line `{line}`")))?;
        f.insert(k.trim(), v.trim());
    }
    let get = |k: &str| -> Result<String, Denial> {
        f.get(k).map(|s| s.to_string()).ok_or_else(|| bad(&format!("missing field `{k}`")))
    };
    let num = |k: &str| -> Result<i64, Denial> {
        get(k)?.parse::<i64>().map_err(|_| bad(&format!("field `{k}` is not an integer")))
    };
    let authority_text = get("authority")?;
    let authority_json: serde_json::Value =
        serde_json::from_str(&authority_text).map_err(|e| bad(&format!("authority is not JSON: {e}")))?;
    let authority = authority_from_json(&authority_json)?;
    Ok(Certificate {
        alg: get("alg")?,
        issuer: get("issuer")?,
        subject: get("subject")?,
        parent: get("parent")?,
        not_before: num("not_before")?,
        not_after: num("not_after")?,
        nonce: get("nonce")?,
        authority,
        sig: from_hex(&get("sig")?).ok_or_else(|| bad("sig is not hex"))?,
    })
}

/// Rebuild an [`Authority`] from the canonical JSON `Authority::to_json` emits.
///
/// **The load-bearing skip branch of this whole phase (RFC §4.9.3).** An unrecognized effect name
/// or an unrecognized scope dimension REFUSES THE WHOLE CERTIFICATE.
///
/// `docs/for-agents.md` tells consumers to **ignore unknown fields** — that is what makes the JSON
/// envelope's additive promise usable, and it is correct *for a reporting surface*. Applying that
/// habit to **authority parsing** is a fail-open of exactly the DL0803 shape: a dimension an old
/// verifier cannot see is a dimension it cannot enforce, so ignoring it silently **widens** the
/// grant. The two rules must be stated together wherever either is stated, or one will be applied
/// to the other's domain.
fn authority_from_json(v: &serde_json::Value) -> Result<Authority, Denial> {
    let unknown = |what: &str, name: &str| Denial::CertUnsupported {
        detail: format!(
            "this build does not understand {what} `{name}`. A certificate is refused whole rather \
             than honored in part: an authority dimension a verifier cannot see is one it cannot \
             enforce, so ignoring it would silently WIDEN the grant"
        ),
    };
    let bad = |m: &str| Denial::CertMalformed { detail: m.to_string() };
    let obj = v.as_object().ok_or_else(|| bad("authority is not an object"))?;
    for key in obj.keys() {
        if key != "effects" && key != "scopes" {
            return Err(unknown("the authority key", key));
        }
    }
    let mut effects = BTreeSet::new();
    for e in obj.get("effects").and_then(|x| x.as_array()).ok_or_else(|| bad("no `effects` array"))? {
        let name = e.as_str().ok_or_else(|| bad("effect is not a string"))?;
        // An unknown effect name is refused, not dropped. Dropping would be narrower (safe) but it
        // would also mean two builds disagree about what a certificate says while both accept it.
        let eff = delulu_check::Effect::core_from_name(name).ok_or_else(|| unknown("the effect", name))?;
        effects.insert(eff);
    }
    let scopes_obj = obj
        .get("scopes")
        .and_then(|x| x.as_object())
        .ok_or_else(|| bad("no `scopes` object"))?;
    let mut scopes = Scopes::default();
    for (key, val) in scopes_obj {
        let list = || -> Result<BTreeSet<String>, Denial> {
            val.as_array()
                .ok_or_else(|| bad(&format!("scope `{key}` is not an array")))?
                .iter()
                .map(|x| x.as_str().map(str::to_string).ok_or_else(|| bad("scope entry is not a string")))
                .collect()
        };
        match key.as_str() {
            "declassify" => scopes.declassify = list()?,
            "fs.read" => scopes.fs_read = list()?,
            "fs.write" => scopes.fs_write = list()?,
            "foreign.c" => scopes.foreign_c = list()?,
            "foreign.python" => scopes.foreign_python = list()?,
            "net" => scopes.net = list()?,
            "secrets" => scopes.secrets = list()?,
            "device" => {
                for spec in list()? {
                    let d = device_scope::parse(&spec)
                        .map_err(|e| bad(&format!("device scope `{spec}`: {e}")))?;
                    scopes.device.insert(d.device.clone(), d);
                }
            }
            other => return Err(unknown("the scope dimension", other)),
        }
    }
    Ok(Authority { effects, scopes })
}

/// Verify a certificate chain and return the authority the LEAF holds.
///
/// `chain` is ordered root-most first. `anchors` is the set of hex public keys this verifier trusts
/// to issue directly. `now_millis` comes from the caller's clock.
///
/// Every check below refuses on "cannot tell":
///
/// 1. A chain must be non-empty, and the first certificate must be issued by a **configured
///    anchor** with `parent == "anchor"` — never "assume the anchor".
/// 2. Each certificate's algorithm must be one this build actually implements ([`Denial::CertUnsupported`]).
/// 3. Each signature must verify **and be made by the key the certificate names as issuer**. A
///    valid signature by some other key is a forgery, not a pass.
/// 4. Each certificate must be inside its validity window against `now_millis`.
/// 5. Each link must match: `parent == previous.fingerprint()` and `issuer == previous.subject`
///    (only the holder may delegate onward).
/// 6. Each hop must satisfy `⊑` against its parent, via the existing lattice.
pub fn verify_chain(
    chain: &[Certificate],
    anchors: &BTreeSet<String>,
    verifier: &dyn SignatureVerifier,
    now_millis: i64,
) -> Result<Authority, Denial> {
    if chain.is_empty() {
        return Err(Denial::CertUntrusted { detail: "empty certificate chain grants nothing".into() });
    }
    let mut prev: Option<&Certificate> = None;
    for (i, c) in chain.iter().enumerate() {
        // 2. Algorithm agility means the format NAMES its algorithm so it can be replaced — never
        //    that an unrecognized name is waved through. Verifying is a trust decision; refusing to
        //    verify must refuse the trust.
        if !verifier.supports(&c.alg) {
            return Err(Denial::CertUnsupported {
                detail: format!(
                    "certificate {i} is signed with `{}`, which this build cannot verify. An \
                     unverifiable signature is refused, never treated as unsigned",
                    c.alg
                ),
            });
        }
        // 3. Signature, and that the SIGNER is the named issuer.
        let signer = verifier.verify(&c.alg, &c.signing_bytes(), &c.sig).ok_or_else(|| {
            Denial::CertUntrusted {
                detail: format!("certificate {i}'s signature does not verify over its own bytes"),
            }
        })?;
        if signer != c.issuer {
            return Err(Denial::CertUntrusted {
                detail: format!(
                    "certificate {i} names issuer `{}` but was actually signed by `{signer}` — a \
                     valid signature by the wrong key is a forgery, not an authorization",
                    c.issuer
                ),
            });
        }
        // 4. Validity window. Checked per certificate: a chain is only as live as its shortest hop.
        if now_millis < c.not_before || now_millis >= c.not_after {
            return Err(Denial::CertExpired {
                not_before: c.not_before,
                not_after: c.not_after,
                now_millis,
            });
        }
        match prev {
            // 1. Root of the chain: must be anchored, and the anchor must be one we configured.
            None => {
                if c.parent != ANCHOR {
                    return Err(Denial::CertUntrusted {
                        detail: format!(
                            "the first certificate claims parent `{}` but nothing precedes it; a \
                             chain that does not start at an anchor is not rooted in anything",
                            c.parent
                        ),
                    });
                }
                if !anchors.contains(&c.issuer) {
                    return Err(Denial::CertUntrusted {
                        detail: format!(
                            "issuer `{}` is not a configured trust anchor. An unknown issuer is \
                             refused — never assumed to be trusted because it looks well-formed",
                            c.issuer
                        ),
                    });
                }
            }
            // 5 + 6. Linkage and attenuation.
            Some(p) => {
                let fp = p.fingerprint();
                if c.parent != fp {
                    return Err(Denial::CertUntrusted {
                        detail: format!(
                            "certificate {i} links to parent `{}` but the preceding certificate \
                             fingerprints to `{fp}` — the chain does not connect",
                            c.parent
                        ),
                    });
                }
                if c.issuer != p.subject {
                    return Err(Denial::CertUntrusted {
                        detail: format!(
                            "certificate {i} was issued by `{}` but the authority was delegated to \
                             `{}` — only the holder may delegate onward",
                            c.issuer, p.subject
                        ),
                    });
                }
                // The existing lattice, unchanged. Federation adds no new authority mathematics.
                if let Err(intersection) = attenuation_check(&c.authority, &p.authority) {
                    return Err(Denial::CertAttenuation {
                        index: i,
                        requested: Box::new(c.authority.clone()),
                        intersection: Box::new(intersection),
                    });
                }
            }
        }
        prev = Some(c);
    }
    Ok(chain[chain.len() - 1].authority.clone())
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use delulu_check::Effect;

    /// A stand-in signer: "signatures" are `blake3(key ‖ msg)` and the signer id is the key's hex.
    /// This is NOT cryptography and is never compiled into the binary — it exists so the chain
    /// logic can be tested without pulling a signature crate into a layer that must not have one.
    /// Real callers inject the runtime's ed25519 (`plugin::verify_detached`).
    struct FakeVerifier;

    fn fake_sign(key: &str, msg: &[u8]) -> Vec<u8> {
        let mut h = blake3::Hasher::new();
        h.update(key.as_bytes());
        h.update(msg);
        let mut out = key.as_bytes().to_vec();
        out.push(0);
        out.extend_from_slice(h.finalize().as_bytes());
        out
    }

    impl SignatureVerifier for FakeVerifier {
        fn supports(&self, alg: &str) -> bool {
            alg == "test-sig"
        }
        fn verify(&self, alg: &str, msg: &[u8], sig: &[u8]) -> Option<String> {
            if alg != "test-sig" {
                return None;
            }
            let split = sig.iter().position(|b| *b == 0)?;
            let key = std::str::from_utf8(&sig[..split]).ok()?;
            let expect = fake_sign(key, msg);
            if expect == sig {
                Some(key.to_string())
            } else {
                None
            }
        }
    }

    fn auth(effects: &[&str], device: &[&str]) -> Authority {
        let scopes = Scopes {
            device: device
                .iter()
                .map(|s| device_scope::parse(s).unwrap())
                .map(|d| (d.device.clone(), d))
                .collect(),
            ..Default::default()
        };
        Authority::new(effects.iter().map(|n| Effect::core_from_name(n).unwrap()), scopes)
    }

    /// Build and sign a certificate with the fake signer.
    fn cert(issuer: &str, subject: &str, parent: &str, a: Authority, window: (i64, i64)) -> Certificate {
        let mut c = Certificate {
            alg: "test-sig".into(),
            issuer: issuer.into(),
            subject: subject.into(),
            parent: parent.into(),
            not_before: window.0,
            not_after: window.1,
            nonce: "abcd".into(),
            authority: a,
            sig: Vec::new(),
        };
        c.sig = fake_sign(issuer, &c.signing_bytes());
        c
    }

    const WIDE: &str = "sat0/hga:slew_deg=-45..45,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park";
    const NARROW: &str = "sat0/hga:slew_deg=-5..5,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park";

    fn anchors(keys: &[&str]) -> BTreeSet<String> {
        keys.iter().map(|s| s.to_string()).collect()
    }

    fn ground_to_vehicle() -> Vec<Certificate> {
        let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));
        let leaf = cert("vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 10_000));
        vec![root, leaf]
    }

    #[test]
    fn a_well_formed_chain_verifies_and_yields_the_leafs_authority() {
        let chain = ground_to_vehicle();
        let got = verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, 500).expect("chain verifies");
        assert_eq!(
            got.scopes.device.get("sat0/hga").and_then(|d| d.dims.get("slew_deg").copied()),
            Some((-5.0, 5.0)),
            "the LEAF's authority is what the holder gets, not the root's"
        );
    }

    #[test]
    fn an_unknown_anchor_is_refused_never_assumed() {
        let chain = ground_to_vehicle();
        let err = verify_chain(&chain, &anchors(&["someone-else"]), &FakeVerifier, 500).unwrap_err();
        assert_eq!(err.code(), "DL1415");
        assert!(format!("{err:?}").contains("not a configured trust anchor"));
        // …and an EMPTY anchor set trusts nothing, rather than everything.
        assert!(verify_chain(&chain, &BTreeSet::new(), &FakeVerifier, 500).is_err());
    }

    #[test]
    fn an_algorithm_this_build_cannot_verify_is_refused_not_treated_as_unsigned() {
        let mut chain = ground_to_vehicle();
        chain[0].alg = "ml-dsa-65".into();
        let err = verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, 500).unwrap_err();
        assert_eq!(err.code(), "DL1418");
        assert!(format!("{err:?}").contains("cannot verify"));
    }

    /// A *valid* signature made by a key other than the one the certificate names must fail. This
    /// is why `SignatureVerifier::verify` returns the signer instead of a bool.
    #[test]
    fn a_valid_signature_by_the_wrong_key_is_a_forgery_not_an_authorization() {
        let mut chain = ground_to_vehicle();
        // `impostor` signs it correctly — but the cert claims `ground` issued it.
        chain[0].sig = fake_sign("impostor", &chain[0].signing_bytes());
        let err = verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, 500).unwrap_err();
        assert_eq!(err.code(), "DL1415");
        assert!(format!("{err:?}").contains("actually signed by"));
    }

    #[test]
    fn tampering_with_any_signed_field_breaks_the_signature() {
        for mutate in [
            (|c: &mut Certificate| c.subject = "attacker".into()) as fn(&mut Certificate),
            |c: &mut Certificate| c.not_after = i64::MAX,
            |c: &mut Certificate| c.alg = "weaker".into(),
            |c: &mut Certificate| c.authority = auth(&["Actuate"], &[WIDE]),
        ] {
            let mut chain = ground_to_vehicle();
            mutate(&mut chain[1]);
            assert!(
                verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, 500).is_err(),
                "every signed field must be covered by the signature"
            );
        }
    }

    #[test]
    fn a_widening_hop_is_refused_with_the_never_widening_intersection() {
        let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[NARROW]), (0, 10_000));
        // The vehicle tries to hand onward MORE than it holds.
        let leaf = cert("vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[WIDE]), (0, 10_000));
        let err = verify_chain(&[root, leaf], &anchors(&["ground"]), &FakeVerifier, 500).unwrap_err();
        assert_eq!(err.code(), "DL1416");
        match &err {
            Denial::CertAttenuation { intersection, .. } => {
                assert_eq!(
                    intersection.scopes.device.get("sat0/hga").and_then(|d| d.dims.get("slew_deg").copied()),
                    Some((-5.0, 5.0)),
                    "the repair narrows to what the parent held"
                );
            }
            other => panic!("expected CertAttenuation, got {other:?}"),
        }
    }

    #[test]
    fn a_broken_link_or_a_non_holder_issuer_does_not_connect() {
        // Wrong parent fingerprint.
        let mut chain = ground_to_vehicle();
        chain[1].parent = "0".repeat(64);
        chain[1].sig = fake_sign("vehicle", &chain[1].signing_bytes()); // re-sign so it fails on the LINK
        let err = verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, 500).unwrap_err();
        assert!(format!("{err:?}").contains("does not connect"));

        // A third party issuing under someone else's delegation.
        let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));
        let leaf = cert("bystander", "payload", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 10_000));
        let err = verify_chain(&[root, leaf], &anchors(&["ground"]), &FakeVerifier, 500).unwrap_err();
        assert!(format!("{err:?}").contains("only the holder may delegate onward"));
    }

    #[test]
    fn the_validity_window_is_checked_at_every_hop() {
        let chain = ground_to_vehicle();
        for now in [-1, 10_000, 50_000] {
            let err = verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, now).unwrap_err();
            assert_eq!(err.code(), "DL1417", "outside the window at {now}");
        }
        // A chain is only as live as its shortest hop: shorten the LEAF and the chain dies with it.
        let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));
        let leaf = cert("vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 100));
        assert!(verify_chain(&[root, leaf], &anchors(&["ground"]), &FakeVerifier, 500).is_err());
    }

    #[test]
    fn an_unrecognized_scope_dimension_refuses_the_whole_certificate() {
        // The RFC §4.9.3 trap: "ignore unknown fields" is right for a REPORT and catastrophic for
        // an AUTHORITY. A dimension we cannot see is one we cannot enforce.
        let json = serde_json::json!({
            "effects": ["Actuate"],
            "scopes": { "fs.read": [], "orbital_slot": ["GEO-119W"] }
        });
        let err = authority_from_json(&json).unwrap_err();
        assert_eq!(err.code(), "DL1418");
        assert!(format!("{err:?}").contains("orbital_slot"));
        // Same rule for an effect this build has never heard of.
        let json = serde_json::json!({ "effects": ["Teleport"], "scopes": {} });
        assert_eq!(authority_from_json(&json).unwrap_err().code(), "DL1418");
        // …and for a stray top-level key, which could be anything.
        let json = serde_json::json!({ "effects": [], "scopes": {}, "override": true });
        assert_eq!(authority_from_json(&json).unwrap_err().code(), "DL1418");
    }

    #[test]
    fn the_wire_form_round_trips_byte_canonically() {
        let chain = ground_to_vehicle();
        for c in &chain {
            let wire = c.to_wire();
            let back = parse(&wire).expect("round-trip parses");
            assert_eq!(&back, c, "parse ∘ render is the identity");
            assert_eq!(back.to_wire(), wire, "…and render is canonical");
            assert_eq!(back.fingerprint(), c.fingerprint(), "identity survives the wire");
        }
    }

    #[test]
    fn malformed_input_is_refused_rather_than_partially_read() {
        for text in [
            "",
            "not-a-cert\nalg: test-sig\n",
            "dlcert1\nalg: test-sig\n",                       // missing everything else
            "dlcert1\nalg: x\nissuer: a\nsubject: b\nparent: anchor\nnot_before: nope\n",
        ] {
            assert!(parse(text).is_err(), "must refuse: {text:?}");
        }
    }

    /// The domain separator is not decoration: a signature over an *artifact* must not be reusable
    /// as a signature over a *grant*. Same bytes, different context, different message.
    #[test]
    fn grant_signatures_are_domain_separated_from_artifact_signatures() {
        let c = &ground_to_vehicle()[0];
        let signed = c.signing_bytes();
        assert!(signed.starts_with(GRANT_CTX), "the context is a prefix of what is signed");
        assert_ne!(GRANT_CTX, b"delulu-artifact-v1", "distinct from pqc.rs's artifact context");
        // The body alone (what an artifact signer would sign) is NOT what a grant signature covers.
        let body = canonical_json(&c.body_value()).into_bytes();
        assert_ne!(signed, body);
    }
}
