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
    /// RFC 0001 F4 — **the uplink lease**: how long the holder may run without a fresh
    /// [`Receipt`]. `None` means the certificate's own window is the only bound.
    ///
    /// This is the dead-man principle applied to a link. A 30-day mission certificate with no
    /// uplink term is un-revocable for 30 days across a partition, because revocation cannot cross
    /// one; the same certificate with `uplink_ttl_ms = 1h` bounds that to an hour. **Silence
    /// shrinks authority**, which makes the safe direction the default one.
    pub uplink_ttl_ms: Option<u64>,
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
        let mut m = serde_json::json!({
            "alg": self.alg,
            "authority": self.authority.to_json(),
            "issuer": self.issuer,
            "not_after": self.not_after,
            "not_before": self.not_before,
            "nonce": self.nonce,
            "parent": self.parent,
            "subject": self.subject,
        });
        // Emitted only when present, so a certificate minted before F4 existed hashes and verifies
        // exactly as it did — the same additive discipline `Authority::to_json` uses for `device`.
        if let Some(u) = self.uplink_ttl_ms {
            m.as_object_mut().expect("json! built an object").insert("uplink_ttl_ms".into(), serde_json::json!(u));
        }
        m
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
        if let Some(u) = self.uplink_ttl_ms {
            s.push_str(&format!("uplink_ttl_ms: {u}\n"));
        }
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
        uplink_ttl_ms: match f.get("uplink_ttl_ms") {
            None => None,
            Some(v) => Some(v.parse::<u64>().map_err(|_| bad("uplink_ttl_ms is not a positive integer"))?),
        },
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
            // PS-B-05: exactly one canonical budget string. Zero, or two, is malformed rather than
            // "no budget" or "the tighter one": a certificate is refused whole before it is read in
            // a way its issuer did not write.
            "budget" => {
                let specs = list()?;
                if specs.len() != 1 {
                    return Err(bad(&format!("scope `budget` must hold exactly one budget, found {}", specs.len())));
                }
                let spec = specs.into_iter().next().expect("length checked");
                scopes.budget = Some(
                    crate::budget_scope::BudgetScope::parse(&spec).map_err(|e| bad(&e))?,
                );
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

// ===================================================================================================
// RFC 0001 phase F4 — the contact receipt: the ONLY thing that extends vehicle authority.
// ===================================================================================================

/// The receipt wire magic.
pub const RECEIPT_MAGIC: &str = "dlrcpt1";

/// The domain separator for contact receipts. Distinct from [`GRANT_CTX`] so a receipt can never be
/// replayed as a grant, nor a grant as a receipt — they are signed by the same keys.
pub const RECEIPT_CTX: &[u8] = b"delulu-receipt-v1";

/// A **contact receipt**: proof, signed by the issuer, that the ground was in contact.
///
/// It grants nothing by itself. It only says "this certificate's holder was reachable, and may run
/// until `not_after`". That asymmetry is the point: authority *decays* with silence and is *renewed*
/// by contact, so losing the link can only ever reduce what a vehicle may do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub alg: String,
    pub issuer: String,
    /// The [`Certificate::fingerprint`] this receipt renews. Binding to a specific credential is
    /// what stops a receipt for a harmless grant being replayed onto a powerful one.
    pub certificate: String,
    pub not_after: i64,
    pub nonce: String,
    pub sig: Vec<u8>,
}

impl Receipt {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::from(RECEIPT_CTX);
        out.push(b'\n');
        let body = serde_json::json!({
            "alg": self.alg,
            "certificate": self.certificate,
            "issuer": self.issuer,
            "nonce": self.nonce,
            "not_after": self.not_after,
        });
        out.extend_from_slice(canonical_json(&body).as_bytes());
        out
    }

    pub fn to_wire(&self) -> String {
        format!(
            "{RECEIPT_MAGIC}\nalg: {}\nissuer: {}\ncertificate: {}\nnot_after: {}\nnonce: {}\nsig: {}\n",
            self.alg,
            self.issuer,
            self.certificate,
            self.not_after,
            self.nonce,
            to_hex(&self.sig)
        )
    }
}

/// Parse a receipt's wire form. Fail-closed at every branch.
pub fn parse_receipt(text: &str) -> Result<Receipt, Denial> {
    let bad = |m: &str| Denial::CertMalformed { detail: m.to_string() };
    let mut lines = text.lines();
    match lines.next().map(str::trim) {
        Some(RECEIPT_MAGIC) => {}
        Some(other) => return Err(bad(&format!("not a contact receipt (magic `{other}`)"))),
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
    Ok(Receipt {
        alg: get("alg")?,
        issuer: get("issuer")?,
        certificate: get("certificate")?,
        not_after: get("not_after")?.parse().map_err(|_| bad("not_after is not an integer"))?,
        nonce: get("nonce")?,
        sig: from_hex(&get("sig")?).ok_or_else(|| bad("sig is not hex"))?,
    })
}

// ===================================================================================================
// RFC 0001 phase F3 — the subordinate broker: adopting a chain as a local root.
// ===================================================================================================

impl crate::tree::Broker {
    /// **Adopt** a verified certificate chain as a local root node (RFC 0001 F3).
    ///
    /// This is what makes a vehicle's broker a *real* broker rather than a cache. After adoption
    /// the node lives in this tree with this tree's rules: `Actuate` round-trips to it locally and
    /// synchronously at full speed, `grants revoke` reaches it, the Guard applies, and the audit
    /// chain records it. **The link is never in the command path.**
    ///
    /// Three properties are enforced here rather than left to the caller:
    ///
    /// 1. **The node's TTL is the MINIMUM `not_after` across the whole chain**, not the leaf's. A
    ///    chain is only as live as its shortest hop, and expiry is the only bound that survives a
    ///    partition — so the credential's expiry becomes the node's expiry, enforced by the same
    ///    TTL machinery every other grant uses. This is what makes "revocation degrades to lease
    ///    TTL across a partition" (RFC §4.5) a mechanism rather than a promise.
    /// 2. **A chain may be adopted only ONCE per broker lifetime.** Without this, `grants revoke`
    ///    on an adopted node is undone by presenting the same certificate again — a
    ///    revocation-evasion path, and a serious one, since revocation is the only tool an operator
    ///    has while the link is up. The scope is the process lifetime because that is exactly the
    ///    scope of the revocation it protects: this broker's tree is in memory, so a restart clears
    ///    both together and a legitimate post-restart re-adoption still works.
    /// 3. **The adoption is audited with the chain's fingerprint**, so the vehicle's own log answers
    ///    "where did this authority come from?" without reference to the ground's log.
    ///
    /// The returned node is a root *locally* and bounded *globally*: nothing about holding it lets
    /// the holder exceed what the ground signed, because the authority stored is exactly what
    /// [`verify_chain`] returned.
    pub fn adopt(
        &mut self,
        chain: &[Certificate],
        anchors: &BTreeSet<String>,
        verifier: &dyn SignatureVerifier,
        holder: crate::tree::Holder,
    ) -> Result<crate::tree::GrantId, Denial> {
        // ADOPT-REPLAY-1 fail-closed: if the persisted revoked-certificate denylist could not be read
        // at startup, this broker cannot prove a presented chain is not one it revoked. Refuse every
        // adoption until an operator repairs it — the safe direction, mirroring the guard `poisoned`
        // posture (the daemon keeps serving revoke/inspect/e-stop and local issue).
        if self.adoptions_poisoned() {
            let seq = self.consume_seq();
            self.record_op(seq, "adopt", None, None, None, "deny", None);
            return Err(Denial::CertUntrusted {
                detail: "this broker's revoked-certificate memory (revoked_certs.json) could not be \
                         read at startup, so it cannot prove this chain was not previously revoked. \
                         Adoptions are refused until an operator repairs or removes that file (removing \
                         it forgets which certificates were revoked)"
                    .to_string(),
            });
        }
        let now = self.effective_now();
        // Strict root-issuance mode (DISC-1): a root may enter ONLY under the broker's PINNED anchor,
        // never a caller-supplied one — otherwise a same-uid client would present its own anchor with a
        // self-signed chain and mint a root. The pinned anchor replaces the caller's set entirely; a
        // chain not rooted in it fails `verify_chain` with DL1415. In the legacy default (no pin), the
        // caller-supplied anchors are honored exactly as before.
        let effective_anchors: BTreeSet<String> = match self.strict_anchor() {
            Some(pinned) => std::iter::once(pinned.to_string()).collect(),
            None => anchors.clone(),
        };
        let authority = verify_chain(chain, &effective_anchors, verifier, now)?;
        let leaf = chain.last().expect("verify_chain rejects an empty chain");
        let fingerprint = leaf.fingerprint();
        let chain_fps: Vec<String> = chain.iter().map(|c| c.fingerprint()).collect();
        if self.adopted_node(&fingerprint).is_some() {
            let seq = self.consume_seq();
            self.record_op(seq, "adopt", None, Some(fingerprint.clone()), None, "deny", None);
            return Err(Denial::CertUntrusted {
                detail: format!(
                    "certificate `{fingerprint}` has already been adopted by this broker. A second \
                     adoption is refused because it would restore authority an operator may have \
                     revoked — re-presenting a credential must not undo a revocation"
                ),
            });
        }
        // P20-R4: the leaf check above only catches re-presenting the SAME certificate. A holder who
        // extends a revoked chain by one fresh self-delegation gets a new leaf, so that check passes
        // while the credential is unchanged. Revocation retires every fingerprint in the chain, so a
        // new chain sharing ANY of them — in practice always the anchored root — is refused here.
        if self.chain_hits_revoked_adoption(&chain_fps) {
            let seq = self.consume_seq();
            self.record_op(seq, "adopt", None, Some(fingerprint.clone()), None, "deny", None);
            return Err(Denial::CertUntrusted {
                detail: format!(
                    "a certificate in the chain ending at `{fingerprint}` belongs to a credential \
                     this broker has revoked. Extending a revoked chain with a fresh delegation does \
                     not restore it: revocation retires the whole credential, not only the leaf that \
                     was presented when it was revoked"
                ),
            });
        }
        // The shortest hop wins. Using the leaf's window alone would let a short-lived root be
        // outlived by the authority it delegated, which is precisely backwards.
        let window = chain.iter().map(|c| c.not_after).min();
        // F4: the uplink lease. The tightest uplink term anywhere in the chain applies, and it is
        // measured from NOW — the moment of adoption is the moment of contact. A vehicle that never
        // hears again loses authority after that term even though the certificate itself is valid
        // for far longer, which is the entire point: revocation cannot cross a partition, so the
        // bound that survives one has to be time.
        let uplink = chain.iter().filter_map(|c| c.uplink_ttl_ms).min();
        let ttl_millis = match (window, uplink) {
            (w, None) => w,
            (None, Some(u)) => Some(now.saturating_add(u as i64)),
            (Some(w), Some(u)) => Some(w.min(now.saturating_add(u as i64))),
        };
        let node = self.issue(holder, authority, ttl_millis);
        self.mark_adopted(&fingerprint, &node, window);
        // Remember the whole chain's fingerprints so revoking this node retires the credential (P20-R4).
        self.record_adopted_chain(&node, chain_fps);
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "adopt",
            Some(node.as_str().to_string()),
            Some(fingerprint),
            None,
            "allow",
            None,
        );
        Ok(node)
    }

    /// Apply a **contact receipt**, extending an adopted node's uplink lease (RFC 0001 F4).
    ///
    /// The certificate window the node was adopted under is remembered by the broker and is the
    /// ceiling a receipt may not lift — a receipt proves contact, not authority. Keeping it here
    /// rather than taking it as an argument removes the chance of a caller supplying the wrong one.
    /// Returns the deadline in force afterwards.
    ///
    /// Deliberately NOT a new enforcement path: the uplink lease *is* the node's TTL, swept by the
    /// same machinery every other grant uses, so an expired uplink is the ordinary `DL1402` with
    /// the ordinary remedy shape. Adding a parallel expiry mechanism would have meant a second
    /// place for liveness to be wrong.
    pub fn renew(
        &mut self,
        receipt: &Receipt,
        anchors: &BTreeSet<String>,
        verifier: &dyn SignatureVerifier,
    ) -> Result<i64, Denial> {
        if !verifier.supports(&receipt.alg) {
            return Err(Denial::CertUnsupported {
                detail: format!(
                    "contact receipt is signed with `{}`, which this build cannot verify",
                    receipt.alg
                ),
            });
        }
        let signer = verifier.verify(&receipt.alg, &receipt.signing_bytes(), &receipt.sig).ok_or_else(
            || Denial::CertUntrusted { detail: "contact receipt signature does not verify".into() },
        )?;
        if signer != receipt.issuer {
            return Err(Denial::CertUntrusted {
                detail: format!(
                    "contact receipt names issuer `{}` but was signed by `{signer}`",
                    receipt.issuer
                ),
            });
        }
        // Strict mode (DISC-1) pins the anchor for RENEWAL too, mirroring `adopt`. Otherwise a
        // same-uid agent could forge a contact receipt under its OWN anchor and extend an adopted
        // node's TTL past the operator's uplink lease — the one revocation bound that survives a
        // partition (a Phase-8 cross-surface finding: strict mode pinned root creation but not renewal).
        let effective_anchors: BTreeSet<String> = match self.strict_anchor() {
            Some(pinned) => std::iter::once(pinned.to_string()).collect(),
            None => anchors.clone(),
        };
        if !effective_anchors.contains(&receipt.issuer) {
            return Err(Denial::CertUntrusted {
                detail: format!(
                    "contact receipt issuer `{}` is not a configured trust anchor",
                    receipt.issuer
                ),
            });
        }
        // The receipt names the credential it renews, so a receipt for a harmless grant cannot be
        // replayed onto a powerful one.
        let Some((node, window)) = self.adopted_node(&receipt.certificate).cloned() else {
            return Err(Denial::CertUntrusted {
                detail: format!(
                    "no adopted certificate `{}` — a receipt renews a specific credential, and this \
                     broker holds no such one",
                    receipt.certificate
                ),
            });
        };
        // A receipt proves contact; it does not enlarge the grant. The certificate's own window is
        // still the ceiling.
        let deadline = match window {
            Some(w) => receipt.not_after.min(w),
            None => receipt.not_after,
        };
        let in_force = self.extend_ttl(&node, deadline).ok_or(Denial::UnknownNode { node: node.clone() })?;
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "renew",
            Some(node.as_str().to_string()),
            Some(receipt.certificate.clone()),
            None,
            "allow",
            None,
        );
        Ok(in_force)
    }
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
        cert_uplink(issuer, subject, parent, a, window, None)
    }

    fn cert_uplink(
        issuer: &str,
        subject: &str,
        parent: &str,
        a: Authority,
        window: (i64, i64),
        uplink_ttl_ms: Option<u64>,
    ) -> Certificate {
        let mut c = Certificate {
            alg: "test-sig".into(),
            issuer: issuer.into(),
            subject: subject.into(),
            parent: parent.into(),
            not_before: window.0,
            not_after: window.1,
            nonce: "abcd".into(),
            uplink_ttl_ms,
            authority: a,
            sig: Vec::new(),
        };
        c.sig = fake_sign(issuer, &c.signing_bytes());
        c
    }

    fn receipt(issuer: &str, certificate: &str, not_after: i64) -> Receipt {
        let mut r = Receipt {
            alg: "test-sig".into(),
            issuer: issuer.into(),
            certificate: certificate.into(),
            not_after,
            nonce: "beef".into(),
            sig: Vec::new(),
        };
        r.sig = fake_sign(issuer, &r.signing_bytes());
        r
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

    // ----- F3: adoption ---------------------------------------------------------------------

    fn broker_at(now: i64) -> crate::tree::Broker {
        broker_clocked(now).0
    }

    /// A broker plus a handle to its clock, so a test can advance time with no sleeps.
    fn broker_clocked(now: i64) -> (crate::tree::Broker, std::rc::Rc<crate::time::ManualClock>) {
        let clock = std::rc::Rc::new(crate::time::ManualClock::new(now));
        let b = crate::tree::Broker::with_sources(
            Box::new(crate::ids::SeqIdSource::new()),
            Box::new(clock.clone()),
        );
        (b, clock)
    }
    fn holder() -> crate::tree::Holder {
        crate::tree::Holder::new("process", "vehicle", "local")
    }

    // ===== DISC-1: strict root-issuance mode (opt-in anchor-verified roots) =======================

    fn strict_broker_at(now: i64, anchor: &str) -> crate::tree::Broker {
        let mut b = broker_at(now);
        b.require_anchored_roots(anchor.to_string());
        b
    }

    /// THE broker-boundary invariant (DISC-1, Jesse's most-important test): **in strict mode no root
    /// node can exist unless it entered via a certificate chain that verifies against the CONFIGURED
    /// (pinned) trust anchor.** Established against the three attacks a same-uid adversary actually has,
    /// checking `unjustified_root_nodes()` stays empty after each. Falsify by making `issue_root` skip
    /// the strict check (attack 1 then creates a root) or by having `adopt` honor caller anchors in
    /// strict mode (attack 2 then adopts).
    #[test]
    fn strict_mode_no_root_without_a_chain_verifying_against_the_pinned_anchor() {
        let mut b = strict_broker_at(500, "ground");

        // Attack 1 — unsigned root issuance. This is the DISC-1 hole: `ReqBody::Issue`, `grants
        // delegate` auto-root, and `run --grant` all reach `issue_root`. Refused DL1421; no root made.
        let e = b.issue_root(holder(), auth(&["Actuate"], &[WIDE]), None).unwrap_err();
        assert_eq!(e.code(), "DL1421", "unsigned root issuance must be refused in strict mode");
        assert!(b.unjustified_root_nodes().is_empty(), "the refused issue created no root");

        // Attack 2 — a same-uid adversary self-signs a chain and presents it under its OWN anchor. The
        // broker PINS `ground` and ignores the caller's `attacker` anchor, so verify_chain fails DL1415.
        let evil = vec![cert("attacker", "attacker", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000))];
        let e = b.adopt(&evil, &anchors(&["attacker"]), &FakeVerifier, holder()).unwrap_err();
        assert_eq!(e.code(), "DL1415", "a self-anchored chain cannot mint a root when the anchor is pinned");
        assert!(b.unjustified_root_nodes().is_empty(), "the adversary created no root");

        // Attack 3 — a forged `ground` cert: it claims issuer `ground` but is signed with the
        // attacker's key. The signature does not verify as `ground`, so it is refused.
        let forged = vec![{
            let mut c = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));
            c.sig = fake_sign("attacker", &c.signing_bytes());
            c
        }];
        assert!(
            b.adopt(&forged, &anchors(&["ground"]), &FakeVerifier, holder()).is_err(),
            "a forged `ground` signature must not verify"
        );
        assert!(b.unjustified_root_nodes().is_empty());

        // The ONLY path that creates a root: a chain verifying against the PINNED anchor. Note the
        // caller passes a bogus anchor set — proving the PIN governs, not the caller.
        let node = b
            .adopt(&ground_to_vehicle(), &anchors(&["caller-picked-nonsense"]), &FakeVerifier, holder())
            .expect("a chain verifying against the PINNED anchor adopts, whatever the caller passes");
        assert_eq!(b.inspect(&node).map(|n| n.parent.is_none()), Some(true), "the adopted node is a root");
        assert!(b.unjustified_root_nodes().is_empty(), "the sole root is a justified adoption");
    }

    /// Strict mode is strictly additive: the legacy default is unchanged. Unsigned roots still work,
    /// and adoption still honors CALLER anchors — which is exactly why the legacy default is not a
    /// boundary against a same-uid agent (it trusts whatever anchor the caller names). Documented, not
    /// hidden — this is the DISC-1 point.
    #[test]
    fn legacy_default_still_allows_unsigned_roots_and_trusts_caller_anchors() {
        let mut b = broker_at(500);
        let root = b.issue_root(holder(), auth(&["Actuate"], &[WIDE]), None).expect("legacy issue_root ok");
        assert_eq!(b.inspect(&root).map(|n| n.parent.is_none()), Some(true));
        assert!(b.adopt(&ground_to_vehicle(), &anchors(&["ground"]), &FakeVerifier, holder()).is_ok());

        // In legacy mode a same-uid adversary's self-anchored chain ADOPTS — the weakness DISC-1 names.
        let mut b2 = broker_at(500);
        let evil = vec![cert("attacker", "attacker", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000))];
        assert!(
            b2.adopt(&evil, &anchors(&["attacker"]), &FakeVerifier, holder()).is_ok(),
            "legacy mode trusts caller anchors; only strict mode pins one"
        );
    }

    /// Under strict mode the existing certificate defenses still apply THROUGH the pinned anchor:
    /// an expired chain and an attenuation-violating chain are refused, so strict mode does not create
    /// a weaker adoption path — it only removes the unsigned one.
    #[test]
    fn strict_mode_still_enforces_expiry_and_attenuation() {
        // Expired: the chain's window has closed by `now`.
        let mut b = strict_broker_at(20_000, "ground");
        let expired = {
            let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));
            let leaf = cert("vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 10_000));
            vec![root, leaf]
        };
        assert_eq!(
            b.adopt(&expired, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap_err().code(),
            "DL1417",
            "an expired chain is refused even under a valid pinned anchor"
        );
        assert!(b.unjustified_root_nodes().is_empty());

        // Widening: a leaf that widens its parent's device envelope is refused DL1416.
        let mut b2 = strict_broker_at(500, "ground");
        let widen = {
            let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[NARROW]), (0, 10_000));
            let leaf = cert("vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[WIDE]), (0, 10_000));
            vec![root, leaf]
        };
        assert_eq!(
            b2.adopt(&widen, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap_err().code(),
            "DL1416",
            "a widening chain is refused even under a valid pinned anchor"
        );
        assert!(b2.unjustified_root_nodes().is_empty());
    }

    /// (DISC-1 Phase-8 cross-surface finding) Strict mode pins the anchor for RENEWAL too: a same-uid
    /// agent's contact receipt signed under its OWN anchor cannot extend an adopted node's TTL past the
    /// operator's uplink lease. Refused DL1415, mirroring `adopt`. In legacy mode (caller anchors) it
    /// would succeed — which is why strict mode must pin renewal, not just creation.
    #[test]
    fn strict_mode_pins_the_anchor_for_renewal_too() {
        let mut b = strict_broker_at(500, "ground");
        let chain = ground_to_vehicle();
        let fp = chain.last().unwrap().fingerprint();
        b.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).expect("legit adopt under the pin");

        // The adversary forges a receipt for that certificate under its OWN anchor and presents it.
        let evil_receipt = receipt("attacker", &fp, 9_000);
        let err = b.renew(&evil_receipt, &anchors(&["attacker"]), &FakeVerifier).unwrap_err();
        assert_eq!(err.code(), "DL1415", "a self-anchored receipt cannot renew under a pinned anchor");

        // Sanity: in LEGACY mode the same forged receipt DOES renew (the documented weakness).
        let mut legacy = broker_at(500);
        let c2 = ground_to_vehicle();
        let fp2 = c2.last().unwrap().fingerprint();
        legacy.adopt(&c2, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert!(
            legacy.renew(&receipt("attacker", &fp2, 9_000), &anchors(&["attacker"]), &FakeVerifier).is_ok(),
            "legacy mode trusts the caller's anchor for renewal — the weakness strict mode closes"
        );
    }

    #[test]
    fn adopting_a_chain_creates_a_local_root_holding_exactly_the_leafs_authority() {
        let mut b = broker_at(500);
        let chain = ground_to_vehicle();
        let node = b.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).expect("adopts");
        let stored = b.inspect(&node).expect("the node is in the local tree");
        assert!(stored.parent.is_none(), "it is a ROOT locally…");
        assert_eq!(
            stored.authority.scopes.device.get("sat0/hga").and_then(|d| d.dims.get("slew_deg").copied()),
            Some((-5.0, 5.0)),
            "…and bounded globally by what the ground signed"
        );
        // And it behaves like any other node: the local Actuate check runs against it, at full
        // speed, with no link in the path.
        assert!(b.check(&node, crate::validate::Op::Actuate, Some("sat0/hga")).is_allow());
        assert!(
            b.check(&node, crate::validate::Op::Actuate, Some("sat0/thruster")).denial().is_some(),
            "a device the certificate never granted is refused locally"
        );
    }

    /// **P20-R4 — revocation cannot be undone by extending a revoked chain.** The holder of the leaf
    /// key can always mint a child of a certificate it holds, and that child has a NEW leaf
    /// fingerprint. The single-adoption check keys on the leaf, so before this fix re-adopting the
    /// extended chain `[root, child]` built a fresh, un-revoked node and handed the revoked authority
    /// straight back. Now revoking an adopted node retires every fingerprint in its chain; every
    /// extension still contains the anchored root, so all of them are refused. Found by the red team,
    /// reproduced end-to-end against the real broker, and pinned here at the unit boundary.
    #[test]
    fn revoking_an_adopted_node_cannot_be_undone_by_extending_the_chain() {
        let mut b = broker_at(500);
        let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));

        // 1. Adopt [root]; the authority is live.
        let node = b
            .adopt(std::slice::from_ref(&root), &anchors(&["ground"]), &FakeVerifier, holder())
            .expect("adopts");
        assert!(b.check(&node, crate::validate::Op::Actuate, Some("sat0/hga")).is_allow());

        // 2. The operator revokes it. The node grants nothing afterward.
        b.revoke(&node, &node).expect("revoked");
        assert!(
            b.check(&node, crate::validate::Op::Actuate, Some("sat0/hga")).denial().is_some(),
            "a revoked node grants nothing"
        );

        // 3. THE ATTACK: extend the chain by one self-delegation the vehicle key can mint offline,
        //    then re-adopt. This is the exact move that undid the revocation before the fix.
        let child =
            cert("vehicle", "vehicle", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 10_000));
        let err = b
            .adopt(&[root.clone(), child], &anchors(&["ground"]), &FakeVerifier, holder())
            .expect_err("adopting an extension of a revoked chain must be refused");
        assert_eq!(err.code(), "DL1415", "must refuse as untrusted, got {err:?}");

        // 4. A SECOND, different extension is refused too — every extension shares the anchored root.
        let child2 =
            cert("vehicle", "other", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 9_500));
        assert!(
            b.adopt(&[root.clone(), child2], &anchors(&["ground"]), &FakeVerifier, holder()).is_err(),
            "no extension of the revoked credential may adopt"
        );

        // 5. Control — a genuinely DIFFERENT credential (different window => different fingerprint)
        //    still adopts. The fix retires the revoked credential, it does not block adoption at large.
        let other = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 9_000));
        assert!(
            b.adopt(std::slice::from_ref(&other), &anchors(&["ground"]), &FakeVerifier, holder()).is_ok(),
            "an un-revoked, unrelated credential must still adopt"
        );
    }

    /// **The federation case for the P17-F1/F2/F3 canonicalization, which is format-affecting.**
    ///
    /// Every other round-trip test here uses paths that are already canonical (`/srv/in`), so none
    /// of them could tell whether a *non-canonical* spelling survives a signature boundary. This one
    /// signs `data` at the root and `.\data\sub` at the leaf — two spellings a Windows operator
    /// would plausibly write — and checks three separate things:
    ///
    /// 1. the chain still **verifies**: signatures cover the bytes as issued, and canonicalization
    ///    happens on the parsed value afterwards, so it cannot invalidate a certificate;
    /// 2. what lands in the local tree is **canonical**, so the adopted grant hashes the same as an
    ///    identical grant issued locally — the whole point of F3, and the thing federation audit
    ///    reconciliation depends on;
    /// 3. the authority **means the same thing**: still inside `./data`, still not `./other`.
    #[test]
    fn a_certificate_carrying_a_non_canonical_path_adopts_canonically_without_changing_meaning() {
        fn fs_auth(paths: &[&str]) -> Authority {
            // Struct literal on purpose: `Authority::new` canonicalizes, which would defeat the
            // test by making the certificate canonical before it was ever signed.
            Authority {
                effects: [Effect::core_from_name("Read").unwrap()].into_iter().collect(),
                scopes: Scopes {
                    fs_read: paths.iter().map(|s| s.to_string()).collect(),
                    ..Default::default()
                },
            }
        }

        let root = cert("ground", "vehicle", ANCHOR, fs_auth(&["data"]), (0, 10_000));
        let leaf =
            cert("vehicle", "payload", &root.fingerprint(), fs_auth(&[r".\data\sub"]), (0, 10_000));
        let chain = vec![root, leaf];

        // 1. A non-canonical spelling does not break verification.
        let got = verify_chain(&chain, &anchors(&["ground"]), &FakeVerifier, 500)
            .expect("a non-canonical spelling must not invalidate a signature");
        assert_eq!(
            got.scopes.fs_read,
            [r".\data\sub".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "verification reports the authority AS SIGNED — canonicalization is the broker's job, \
             not the verifier's, or the signed bytes and the checked value would disagree"
        );

        // 2. What the local tree stores is canonical.
        let mut b = broker_at(500);
        let node = b.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).expect("adopts");
        let stored = b.inspect(&node).expect("the node is in the local tree");
        assert!(
            stored.authority.is_canonical(),
            "an adopted certificate entered the tree non-canonically ({:?}) — a federated grant \
             would then hash differently from the identical grant issued locally",
            stored.authority.scopes.fs_read
        );
        assert_eq!(
            stored.authority.scopes.fs_read,
            ["./data/sub".to_string()].into_iter().collect::<BTreeSet<_>>()
        );

        // 3. And it means exactly what it meant on the wire.
        let inside = Authority {
            effects: [Effect::core_from_name("Read").unwrap()].into_iter().collect(),
            scopes: Scopes {
                fs_read: ["./data/sub/deep".to_string()].into_iter().collect(),
                ..Default::default()
            },
        };
        assert!(
            crate::authority::attenuation_check(&inside, &stored.authority).is_ok(),
            "a path inside the granted subtree must still attenuate after adoption"
        );
        let outside = Authority {
            effects: [Effect::core_from_name("Read").unwrap()].into_iter().collect(),
            scopes: Scopes {
                fs_read: ["./other".to_string()].into_iter().collect(),
                ..Default::default()
            },
        };
        assert!(
            crate::authority::attenuation_check(&outside, &stored.authority).is_err(),
            "canonicalization must not have WIDENED the adopted grant"
        );
    }

    /// The TTL is the SHORTEST hop's, not the leaf's. A short-lived root must not be outlived by
    /// the authority it delegated.
    #[test]
    fn the_adopted_ttl_is_the_shortest_hop_in_the_chain() {
        let root = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 2_000));
        let leaf = cert("vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 9_000));
        let mut b = broker_at(500);
        let node = b.adopt(&[root, leaf], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert_eq!(
            b.inspect(&node).unwrap().ttl_millis,
            Some(2_000),
            "the root's earlier expiry bounds the adopted node"
        );
    }

    /// **Revocation must not be undoable by replay.** Without single-adoption, an operator's
    /// `grants revoke` — the only tool they have while the link is up — is defeated by presenting
    /// the same certificate again.
    #[test]
    fn a_certificate_cannot_be_adopted_twice_so_replay_cannot_undo_a_revocation() {
        let mut b = broker_at(500);
        let chain = ground_to_vehicle();
        let node = b.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        b.revoke(&node, &node).expect("the operator revokes it");
        assert!(b.check(&node, crate::validate::Op::Actuate, Some("sat0/hga")).denial().is_some());

        let err = b.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap_err();
        assert_eq!(err.code(), "DL1415");
        assert!(format!("{err:?}").contains("already been adopted"));
    }

    /// **ADOPT-REPLAY-1**: the single-adoption memory above lived only in this `Broker` — a restart
    /// built a fresh, empty one, and the revoked certificate walked back in because a certificate is a
    /// self-contained artifact, not a reference into the wiped tree. The daemon now persists
    /// `revoked_adoption_fps_snapshot()` and re-seeds it via `restore_revoked_adoption_fps` at startup.
    /// This exercises that hand-off directly: the revoked chain is refused by the reconstructed broker,
    /// while a broker WITHOUT the hand-off (the pre-fix behavior) re-adopts it — pinning both the bug
    /// and its fix.
    #[test]
    fn a_revoked_chains_denylist_survives_a_restart_via_snapshot_restore() {
        // Lifetime 1: adopt, then revoke — the chain's fingerprints are retired.
        let mut b1 = broker_at(500);
        let chain = ground_to_vehicle();
        let node = b1.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        b1.revoke(&node, &node).expect("the operator revokes it");
        let snapshot = b1.revoked_adoption_fps_snapshot();
        assert!(!snapshot.is_empty(), "revoking an adopted node retires its chain fingerprints");

        // Lifetime 2 WITHOUT the persisted denylist (pre-fix): a fresh broker has an empty tree, so the
        // same certificate re-adopts — exactly the ADOPT-REPLAY-1 hole.
        let mut b_lost = broker_at(500);
        assert!(
            b_lost.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).is_ok(),
            "without the persisted denylist a revoked certificate re-adopts — the bug this closes"
        );

        // Lifetime 2 WITH the denylist re-seeded at startup (the fix): still refused.
        let mut b2 = broker_at(500);
        b2.restore_revoked_adoption_fps(snapshot);
        let err = b2.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap_err();
        assert_eq!(err.code(), "DL1415");
        assert!(
            format!("{err:?}").to_lowercase().contains("revoked"),
            "refused specifically as a revoked credential: {err:?}"
        );
    }

    /// **ADOPT-REPLAY-1 fail-closed**: if the persisted denylist was unreadable at startup, the daemon
    /// poisons adoptions and refuses EVERY certificate — even a never-revoked one — rather than risk
    /// re-adopting one it can no longer prove it revoked.
    #[test]
    fn poisoned_adoptions_refuse_every_certificate() {
        let mut b = broker_at(500);
        b.poison_adoptions();
        let chain = ground_to_vehicle(); // a perfectly valid, never-revoked chain
        let err = b.adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder()).unwrap_err();
        assert!(
            format!("{err:?}").contains("revoked_certs.json"),
            "a poisoned broker refuses adoption and names the file to repair: {err:?}"
        );
    }

    /// **Phase M — a fuzzing-style battery: every adversarial certificate/receipt is a Denial, never a
    /// panic.** A certificate crosses a trust boundary as a file; the operator who adopts it did not
    /// write it. `parse`/`parse_receipt` must treat every malformed, oversized, integer-overflowing, or
    /// deeply-nested input as a refusal — never an index panic, an `as`/arithmetic overflow, or a stack
    /// overflow. The assertions prove each result is `Err`; reaching the end of the loop proves nothing
    /// panicked (a panic in a `#[test]` is a failure). The valid template is checked first so the
    /// battery is known to vary exactly one thing at a time.
    #[test]
    fn adversarial_certificates_and_receipts_are_denials_never_panics() {
        let auth = "{\"effects\":[],\"scopes\":{}}";
        let valid = format!(
            "dlcert1\nalg: ed25519\nissuer: aa\nsubject: bb\nparent: anchor\n\
             not_before: 0\nnot_after: 1000\nnonce: nn\nauthority: {auth}\nsig: 00"
        );
        assert!(parse(&valid).is_ok(), "the valid template must parse, else the battery tests nothing");

        // 600 deep — far past serde_json's 128 recursion limit, which turns this into an Err at parse
        // time rather than a stack overflow.
        let nested = format!("{}{}", "[".repeat(600), "]".repeat(600));
        let cases: Vec<String> = vec![
            String::new(),                               // empty
            "dlcert1".to_string(),                       // magic only, no fields
            "dlcert1\nno-colon-line".to_string(),        // a line with no `key: value`
            "dlcert1\nalg: ed25519".to_string(),         // missing required fields
            valid.replace("not_after: 1000", "not_after: 99999999999999999999999999999"), // i64 overflow
            valid.replace("nonce: nn", "nonce: nn\nuplink_ttl_ms: 99999999999999999999999999"), // u64 overflow
            valid.replace(auth, "not-json"),             // authority is not JSON
            valid.replace(auth, &nested),                // authority nested past the recursion limit
            valid.replace("sig: 00", "sig: abc"),        // odd-length hex
            valid.replace("sig: 00", "sig: zz"),         // even-length but not hex
            valid.replace("not_before: 0", "not_before: not-a-number"), // non-numeric int field
        ];
        for c in &cases {
            assert!(parse(c).is_err(), "expected a Denial (not a panic/Ok) for:\n{c}");
        }

        // Receipts share the line-oriented shape and the same failure modes.
        let receipts: Vec<String> = vec![
            String::new(),
            "dlrcpt1".to_string(),
            "dlrcpt1\nno-colon".to_string(),
            "dlrcpt1\nalg: ed25519\nissuer: a\ncertificate: c\nnot_after: 99999999999999999999999999\nnonce: n\nsig: 00".to_string(),
            "dlrcpt1\nalg: ed25519\nissuer: a\ncertificate: c\nnot_after: 10\nnonce: n\nsig: xyz".to_string(),
        ];
        for r in &receipts {
            assert!(parse_receipt(r).is_err(), "expected a Denial for receipt:\n{r}");
        }
    }

    #[test]
    fn adoption_refuses_exactly_what_chain_verification_refuses() {
        let chain = ground_to_vehicle();
        // Unknown anchor.
        assert!(broker_at(500).adopt(&chain, &anchors(&["nobody"]), &FakeVerifier, holder()).is_err());
        // Outside the window: the broker's OWN clock decides, not the presenter's.
        assert_eq!(
            broker_at(50_000)
                .adopt(&chain, &anchors(&["ground"]), &FakeVerifier, holder())
                .unwrap_err()
                .code(),
            "DL1417"
        );
        // Empty chain grants nothing.
        assert!(broker_at(500).adopt(&[], &anchors(&["ground"]), &FakeVerifier, holder()).is_err());
    }

    // ----- F4: the uplink lease --------------------------------------------------------------

    /// The point of the whole phase: a long-lived certificate with a short uplink term is bounded
    /// by the SHORT one. Without this, a 30-day mission grant is un-revocable for 30 days across a
    /// partition, because revocation cannot cross one.
    #[test]
    fn the_uplink_lease_bounds_a_long_lived_certificate() {
        let long_window = (0, 30_000_000);
        let c = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), long_window, Some(1_000));
        let mut b = broker_at(500);
        let node = b.adopt(&[c], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert_eq!(
            b.inspect(&node).unwrap().ttl_millis,
            Some(1_500),
            "now + uplink_ttl, not the certificate's own far-off expiry"
        );
    }

    #[test]
    fn with_no_uplink_term_the_certificate_window_is_the_only_bound() {
        let mut b = broker_at(500);
        let node = b.adopt(&ground_to_vehicle(), &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert_eq!(b.inspect(&node).unwrap().ttl_millis, Some(10_000), "F3 behaviour, preserved");
    }

    #[test]
    fn the_tightest_uplink_term_anywhere_in_the_chain_applies() {
        let root = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 90_000), Some(5_000));
        let leaf = cert_uplink(
            "vehicle", "payload", &root.fingerprint(), auth(&["Actuate"], &[NARROW]), (0, 90_000), Some(2_000),
        );
        let mut b = broker_at(1_000);
        let node = b.adopt(&[root, leaf], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert_eq!(b.inspect(&node).unwrap().ttl_millis, Some(3_000), "1000 + the tighter 2000");
    }

    /// Contact renews; silence does not. And a receipt may never push a lease past the certificate
    /// window it renews — proof of contact is not a grant of authority.
    #[test]
    fn a_contact_receipt_extends_the_lease_but_never_past_the_certificate() {
        let c = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 8_000), Some(1_000));

        let fp = c.fingerprint();
        let mut b = broker_at(500);
        let node = b.adopt(&[c], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert_eq!(b.inspect(&node).unwrap().ttl_millis, Some(1_500));

        // Contact: the lease moves forward.
        let got = b
            .renew(&receipt("ground", &fp, 4_000), &anchors(&["ground"]), &FakeVerifier)
            .expect("a valid receipt renews");
        assert_eq!(got, 4_000);
        assert_eq!(b.inspect(&node).unwrap().ttl_millis, Some(4_000));

        // A receipt that reaches past the certificate is CLAMPED to it, not honoured.
        let got = b
            .renew(&receipt("ground", &fp, 999_999), &anchors(&["ground"]), &FakeVerifier)
            .unwrap();
        assert_eq!(got, 8_000, "the certificate window is the ceiling a receipt cannot lift");
    }

    /// Replaying an old receipt must be a harmless no-op — never a way to strip authority from a
    /// vehicle. Extension is monotone; shortening is `revoke`'s job.
    #[test]
    fn replaying_an_old_receipt_cannot_shorten_a_lease() {
        let c = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 9_000), Some(1_000));
        let fp = c.fingerprint();
        let mut b = broker_at(500);
        let node = b.adopt(&[c], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        b.renew(&receipt("ground", &fp, 6_000), &anchors(&["ground"]), &FakeVerifier).unwrap();
        // An attacker replays a stale receipt with an earlier deadline.
        let got = b
            .renew(&receipt("ground", &fp, 2_000), &anchors(&["ground"]), &FakeVerifier)
            .unwrap();
        assert_eq!(got, 6_000, "the lease did not move backwards");
        assert_eq!(b.inspect(&node).unwrap().ttl_millis, Some(6_000));
    }

    /// **The hole F4 would have had, and the witness that keeps it shut.**
    ///
    /// `attenuate` bounds a child's authority by `⊑` but does NOT bound its deadline, and
    /// `effective_state` judges one node. So a holder could delegate itself a child with no TTL and
    /// keep commanding after its own lease died. Locally that is a latent wrong; under federation
    /// it is fatal — the uplink lease is the only bound that survives a partition, and the party it
    /// bounds is precisely the party that can mint children.
    ///
    /// Fails against per-node expiry; passes with inherited expiry.
    #[test]
    fn a_child_cannot_outlive_the_expired_uplink_lease_of_its_parent() {
        let c = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 90_000), Some(1_000));
        let (mut b, clock) = broker_clocked(500);
        let root = b.adopt(&[c], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        assert_eq!(b.inspect(&root).unwrap().ttl_millis, Some(1_500));

        // The vehicle delegates itself a child with NO deadline of its own — the escape attempt.
        let child = b
            .attenuate(&root, auth(&["Actuate"], &[WIDE]), holder(), None)
            .expect("delegating within the lease is legitimate");
        assert!(b.inspect(&child).unwrap().ttl_millis.is_none(), "the child carries no TTL itself");
        assert!(b.check(&child, crate::validate::Op::Actuate, Some("sat0/hga")).is_allow());

        // Time passes with no contact. The root's uplink lease dies…
        clock.set(2_000);
        assert!(matches!(b.effective_state(&root), Some(crate::tree::EffState::Expired { .. })), "root expired");
        // …and the child must die WITH it, or the uplink lease bounds nothing.
        let d = b.check(&child, crate::validate::Op::Actuate, Some("sat0/hga"));
        let denial = d.denial().expect("a child must not outlive its root's lease");
        assert_eq!(denial.code(), "DL1402");
        assert!(
            matches!(b.effective_state(&child), Some(crate::tree::EffState::Expired { .. })),
            "and the operator is SHOWN the same answer the enforcement path gives"
        );
        // A grandchild cannot be born under the dead ancestry either.
        assert!(b.attenuate(&child, auth(&["Actuate"], &[WIDE]), holder(), None).is_err());
    }

    /// A contact receipt revives the whole subtree, not just the root — which is why expiry is
    /// inherited at READ time rather than clamped into children at write time.
    #[test]
    fn renewing_the_root_lease_carries_the_whole_subtree_with_it() {
        let c = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 90_000), Some(1_000));
        let fp = c.fingerprint();
        let (mut b, clock) = broker_clocked(500);
        let root = b.adopt(&[c], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        let child = b.attenuate(&root, auth(&["Actuate"], &[WIDE]), holder(), None).unwrap();

        clock.set(2_000);
        assert!(b.check(&child, crate::validate::Op::Actuate, Some("sat0/hga")).denial().is_some());

        // Contact re-established: one receipt on the ROOT restores the child too.
        b.renew(&receipt("ground", &fp, 50_000), &anchors(&["ground"]), &FakeVerifier).unwrap();
        assert!(
            b.check(&child, crate::validate::Op::Actuate, Some("sat0/hga")).is_allow(),
            "the subtree comes back with its root"
        );
    }

    #[test]
    fn a_receipt_is_refused_unless_it_verifies_to_an_anchor_and_names_a_held_certificate() {
        let c = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 9_000), Some(1_000));
        let fp = c.fingerprint();
        let mut b = broker_at(500);
        b.adopt(&[c], &anchors(&["ground"]), &FakeVerifier, holder()).unwrap();
        let a = anchors(&["ground"]);

        // Signed by someone who is not an anchor.
        let err = b.renew(&receipt("impostor", &fp, 5_000), &a, &FakeVerifier).unwrap_err();
        assert_eq!(err.code(), "DL1415");
        // Signature does not match the named issuer.
        let mut forged = receipt("ground", &fp, 5_000);
        forged.sig = fake_sign("impostor", &forged.signing_bytes());
        assert_eq!(b.renew(&forged, &a, &FakeVerifier).unwrap_err().code(), "DL1415");
        // A receipt for a credential this broker does not hold.
        let err = b.renew(&receipt("ground", &"f".repeat(64), 5_000), &a, &FakeVerifier).unwrap_err();
        assert!(format!("{err:?}").contains("no adopted certificate"));
        // An algorithm this build cannot verify.
        let mut alien = receipt("ground", &fp, 5_000);
        alien.alg = "ml-dsa-65".into();
        assert_eq!(b.renew(&alien, &a, &FakeVerifier).unwrap_err().code(), "DL1418");
    }

    /// A receipt and a grant are signed by the same keys, so their domain separators must differ or
    /// one would be replayable as the other.
    #[test]
    fn receipts_and_grants_are_domain_separated_from_each_other() {
        assert_ne!(RECEIPT_CTX, GRANT_CTX);
        let r = receipt("ground", &"a".repeat(64), 1);
        assert!(r.signing_bytes().starts_with(RECEIPT_CTX));
        let back = parse_receipt(&r.to_wire()).expect("round-trips");
        assert_eq!(back, r);
        assert!(parse_receipt("dlcert1\nalg: x\n").is_err(), "a certificate is not a receipt");
    }

    /// A certificate minted before F4 existed must still verify byte-for-byte: the new field is
    /// omitted when absent, exactly as `Authority::to_json` omits an empty `device`.
    #[test]
    fn the_uplink_field_is_omitted_when_absent_so_older_certificates_are_unchanged() {
        let c = cert("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000));
        assert!(!c.to_wire().contains("uplink_ttl_ms"), "absent means absent, not `0`");
        assert!(verify_chain(&[c], &anchors(&["ground"]), &FakeVerifier, 500).is_ok());
        let with = cert_uplink("ground", "vehicle", ANCHOR, auth(&["Actuate"], &[WIDE]), (0, 10_000), Some(7));
        assert!(with.to_wire().contains("uplink_ttl_ms: 7"));
        assert_eq!(parse(&with.to_wire()).unwrap(), with, "and it round-trips");
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
    // ----- the authority SERIALIZATION seam (hardening P15) -------------------------------------

    /// **Every authority dimension must survive the write/read round trip, and the compiler must
    /// force this test to be revisited when a dimension is added.**
    ///
    /// `Authority::to_json` (the write side) and `authority_from_json` (the read side) are two
    /// hand-enumerated lists of the same eight dimensions, on opposite sides of a certificate, an
    /// audit record and a `--json` report. The read side already fails closed on a dimension it does
    /// not recognize — that is RFC §4.9.3's load-bearing skip branch, and it is tested. The WRITE
    /// side had no such protection: a ninth dimension added to `Scopes` would simply not be emitted,
    /// and nothing would notice.
    ///
    /// That failure is quiet rather than loud. Omitting a dimension is fail-CLOSED for the grant (a
    /// certificate would convey less than it should), so nothing becomes more permissive — but the
    /// authority embedded in every hash-chained AUDIT record would silently under-report what a
    /// holder actually held, and `render_compact` feeds the same list into DL0802's repair text. An
    /// audit trail that under-reports authority is the C29/C30 defect class: not an escalation, but a
    /// loss of exactly the accountability this system sells.
    ///
    /// PS-B-05: a budget a verifier cannot read exactly is refused, never read loosely — two of them,
    /// none, a zero, a dimension this build does not know.
    #[test]
    fn a_malformed_budget_refuses_the_whole_authority() {
        for bad in [
            serde_json::json!([]),
            serde_json::json!(["mem=1,cpu=1", "mem=2,cpu=2"]),
            serde_json::json!(["mem=0,cpu=1"]),
            serde_json::json!(["mem=1,cpu=1,wall=5"]),
            serde_json::json!(["mem=1"]),
            serde_json::json!("mem=1,cpu=1"),
        ] {
            let v = serde_json::json!({ "effects": [], "scopes": { "budget": bad } });
            assert!(authority_from_json(&v).is_err(), "a budget of {bad} must be refused");
        }
        let ok = serde_json::json!({ "effects": [], "scopes": { "budget": ["mem=1,cpu=1"] } });
        assert!(authority_from_json(&ok).is_ok());
    }

    /// The destructuring below is the enforcement. It is not decoration: adding a field to `Scopes`
    /// makes this test fail to COMPILE until someone decides how the new dimension serializes.
    #[test]
    fn every_authority_dimension_survives_the_json_round_trip() {
        let scopes = Scopes {
            fs_read: ["/srv/in".to_string()].into_iter().collect(),
            fs_write: ["/srv/out".to_string()].into_iter().collect(),
            net: ["api.example.com".to_string()].into_iter().collect(),
            secrets: ["API_KEY".to_string()].into_iter().collect(),
            declassify: ["API_KEY".to_string()].into_iter().collect(),
            foreign_c: ["libm".to_string()].into_iter().collect(),
            foreign_python: ["numpy".to_string()].into_iter().collect(),
            device: [device_scope::parse(
                "arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold",
            )
            .unwrap()]
            .into_iter()
            .map(|d| (d.device.clone(), d))
            .collect(),
            budget: Some(crate::budget_scope::BudgetScope { memory_bytes: 268_435_456, cpu_seconds: 60 }),
        };
        // Exhaustive by construction: a new field breaks this pattern at compile time. (It did, for
        // `budget` at PS-B-05, which is how the dimension got its serialization decided.)
        let Scopes {
            fs_read,
            fs_write,
            net,
            secrets,
            declassify,
            foreign_c,
            foreign_python,
            device,
            budget,
        } = &scopes;
        for (what, empty) in [
            ("fs_read", fs_read.is_empty()),
            ("fs_write", fs_write.is_empty()),
            ("net", net.is_empty()),
            ("secrets", secrets.is_empty()),
            ("declassify", declassify.is_empty()),
            ("foreign_c", foreign_c.is_empty()),
            ("foreign_python", foreign_python.is_empty()),
            ("device", device.is_empty()),
            ("budget", budget.is_none()),
        ] {
            assert!(!empty, "the fixture must populate `{what}`, or the round trip proves nothing");
        }

        let original = Authority::new(
            ["Read", "Write", "Net", "Declassify", "ForeignCall", "Actuate"]
                .iter()
                .map(|n| Effect::core_from_name(n).unwrap()),
            scopes.clone(),
        );
        let json = original.to_json();
        let parsed = authority_from_json(&json).expect("the canonical form must parse back");
        assert_eq!(parsed, original, "an authority must survive write -> read unchanged");

        // And the rendering a human reads must mention every non-empty dimension, since it is the
        // same hand-written list and feeds DL0802's repair text.
        let compact = original.render_compact();
        for needle in ["fs.read", "fs.write", "net", "secrets", "declassify", "foreign.c", "foreign.python", "device", "budget"] {
            assert!(compact.contains(needle), "`render_compact` omits `{needle}`: {compact}");
        }
    }
}
