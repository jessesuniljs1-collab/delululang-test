//! PS-D-02: the attestation seam (D-V2-48) — an external launcher's attester vouches for the guest's
//! environment, and a run can refuse to serve a guest nobody vouched for.
//!
//! The seam verifies a STATEMENT, not a platform. An attestation document is
//!
//! ```text
//! {"format":"delulu-attestation-v1",
//!  "statement":{"attester":NAME,"guarantees":[TEXT, …],"nonce":HEX},
//!  "signature":HEX}
//! ```
//!
//! and its signature is ed25519 over `"delulu-attestation-v1\n"` followed by the statement's canonical
//! JSON: keys in byte order (`attester`, `guarantees`, `nonce`), no whitespace, strings escaped as JSON
//! requires — which, with control characters refused, is only `\"` and `\\` — and anything else as
//! UTF-8. (Python's `json.dumps(s, sort_keys=True, separators=(",", ":"), ensure_ascii=False)` writes
//! the same bytes.) `signature` is the detached form every other signature here uses, the signer's
//! public key followed by the signature, 96 bytes in hex.
//!
//! Whoever signs it — a verifier service that checked a hardware quote, a CI system that built the
//! launcher's image, an operator — is the attester, and its word is exactly as good as its key's
//! custody. DeluluLang checks three things and nothing else: the signature, that the signer is the key
//! the operator PINNED for this run (`--require-attestation HEX`), and that the statement carries the
//! nonce the host chose for this run, so a document written for another run is refused. The claims
//! themselves are the attester's: the report carries them under `sandbox.attestation`, beside the
//! host's own guarantees and never merged into them, and the level stays 3.
//!
//! The host gives the launcher the nonce in [`ENV_NONCE`] and a path in its own per-run directory in
//! [`ENV_OUT`]; the launcher writes the document there, whole (a temporary file, then a rename), and
//! the host reads it after the launcher starts and BEFORE it sends the program — so on a refusal the
//! program never ran and no effect was performed under the grants or a lease.
//!
//! `delulu sandbox attest` is a reference attester: a SOFTWARE one, which says what its key's holder
//! says, and the tests' fake. A lease-level constraint ("this node may only be used attested") is not
//! part of the seam: it changes the authority model, which is RFC territory (`HANDOFF.md` §0).

use serde::{Deserialize, Serialize};

/// The document's format, and the first line of what is signed.
pub const FORMAT: &str = "delulu-attestation-v1";
/// The environment variable that carries this run's nonce to the launcher, as 64 lowercase hex digits.
pub const ENV_NONCE: &str = "DELULU_ATTEST_NONCE";
/// The environment variable that names where the launcher writes the document.
pub const ENV_OUT: &str = "DELULU_ATTEST_OUT";
/// The document's name in the host's per-run directory.
pub const FILE_NAME: &str = "attestation.json";

/// Bounds on what an attester may say, so a document can neither make the host read without limit nor
/// fill a report or a terminal. Generous for a list of claims; nothing a real one needs is refused.
const MAX_DOCUMENT_BYTES: u64 = 64 * 1024;
const MAX_GUARANTEES: usize = 32;
const MAX_TEXT_CHARS: usize = 256;

/// What the attester says. The fields are declared in the byte order of their names, so serializing
/// this struct IS the canonical form; an unknown field is refused rather than dropped, because a claim
/// that is silently not carried reads exactly like one that was never made.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Statement {
    pub attester: String,
    pub guarantees: Vec<String>,
    pub nonce: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format: String,
    pub statement: Statement,
    pub signature: String,
}

/// The bytes the signature covers.
pub fn signed_bytes(statement: &Statement) -> Vec<u8> {
    let mut msg = format!("{FORMAT}\n").into_bytes();
    msg.extend(serde_json::to_vec(statement).expect("a statement serializes"));
    msg
}

/// A fresh nonce for one run: 32 bytes of the operating system's randomness, in lowercase hex.
pub fn fresh_nonce() -> Result<String, String> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).map_err(|e| format!("the operating system's randomness is unavailable ({e})"))?;
    Ok(crate::breakglass::encode_hex(&b))
}

/// The pinned key as given on the command line, in the form it is compared in: 64 hex digits, any case,
/// lowercased. Whether it is a valid point is the signature check's to say.
pub fn pinned_key(given: &str) -> Result<String, String> {
    let k = given.trim().to_ascii_lowercase();
    if k.len() != 64 || crate::breakglass::decode_hex(&k).is_none() {
        return Err(format!(
            "`--require-attestation {given}` is not an ed25519 public key (64 hex characters — the `public key` \
             `delulu keygen` prints, or its `.pub` file)"
        ));
    }
    Ok(k)
}

/// Every claim is printable text of a bounded length: it is shown to an operator and kept in a report.
fn check_text(what: &str, s: &str) -> Result<(), String> {
    if s.trim().is_empty() {
        return Err(format!("{what} is empty"));
    }
    if s.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{what} is longer than {MAX_TEXT_CHARS} characters"));
    }
    if s.chars().any(char::is_control) {
        return Err(format!("{what} contains a control character"));
    }
    Ok(())
}

fn check_statement(s: &Statement) -> Result<(), String> {
    check_text("the attester's name", &s.attester)?;
    if s.guarantees.is_empty() {
        return Err("it claims no guarantee at all".to_string());
    }
    if s.guarantees.len() > MAX_GUARANTEES {
        return Err(format!("it makes more than {MAX_GUARANTEES} claims"));
    }
    for g in &s.guarantees {
        check_text("a guarantee", g)?;
    }
    Ok(())
}

/// Sign a statement (the attester's side).
pub fn sign(seed: &[u8; 32], statement: Statement) -> Result<Document, String> {
    check_statement(&statement)?;
    let signature = crate::breakglass::encode_hex(&delulu_runtime::plugin::sign_detached(seed, &signed_bytes(&statement)));
    Ok(Document { format: FORMAT.to_string(), statement, signature })
}

/// A statement that verified: the attester's claims, labelled as the attester's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attested {
    /// The pinned key that signed it, lowercase hex.
    pub key: String,
    pub attester: String,
    pub guarantees: Vec<String>,
}

impl Attested {
    /// `sandbox.attestation` in a run report.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "key": self.key,
            "attester": self.attester,
            "guarantees": self.guarantees,
            "verified": true,
        })
    }
}

/// `sandbox.attestation` where it was required and nothing was verified: a dry run (`--mode audit`)
/// launches nothing, so there is nothing to check yet.
pub fn required_json(key: &str) -> serde_json::Value {
    serde_json::json!({ "key": key, "attester": null, "guarantees": [], "verified": false })
}

/// Why a guest was not served. Each is said in words; none of them lets the program run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The launcher wrote no document within the deadline.
    Absent(std::time::Duration),
    /// The launcher ended before it wrote one.
    LauncherEnded(String),
    Unreadable(String),
    /// ATTEST-FIFO-1: the path held a link, a pipe or a device, not a document.
    NotAFile,
    TooLarge,
    /// The document ends before it is whole: an attester that wrote it in place, read mid-write (the host
    /// reads it the moment it exists), or one that stopped writing.
    Incomplete,
    NotADocument(String),
    WrongFormat(String),
    BadStatement(String),
    WrongNonce,
    BadSignature(String),
    WrongKey { pinned: String, signer: String },
}

impl Refusal {
    pub fn code(&self) -> &'static str {
        match self {
            Refusal::Absent(_) => "absent",
            Refusal::LauncherEnded(_) => "launcher-ended",
            Refusal::Unreadable(_) => "unreadable",
            Refusal::NotAFile => "not-a-file",
            Refusal::TooLarge => "too-large",
            Refusal::Incomplete => "incomplete",
            Refusal::NotADocument(_) => "not-a-document",
            Refusal::WrongFormat(_) => "wrong-format",
            Refusal::BadStatement(_) => "bad-statement",
            Refusal::WrongNonce => "wrong-nonce",
            Refusal::BadSignature(_) => "bad-signature",
            Refusal::WrongKey { .. } => "wrong-key",
        }
    }

    pub fn explain(&self) -> String {
        let why = match self {
            Refusal::Absent(d) => format!(
                "the launcher wrote no attestation within {d:?} (an attester writes it to `${ENV_OUT}`, signed over \
                 `${ENV_NONCE}` — `delulu sandbox attest` is one)"
            ),
            Refusal::LauncherEnded(st) => format!("the launcher ended ({st}) before it wrote an attestation"),
            Refusal::Unreadable(e) => format!("the attestation cannot be read ({e})"),
            Refusal::NotAFile => {
                "what the launcher put where the attestation goes is not a regular file (a link, a pipe or a \
                 device) — the host reads a document there, nothing else"
                    .to_string()
            }
            Refusal::TooLarge => format!("the attestation is larger than {MAX_DOCUMENT_BYTES} bytes"),
            Refusal::Incomplete => format!(
                "the attestation is incomplete: the document ends before it is whole. The host reads it the moment \
                 it exists, so an attester writes it whole — a temporary file, then a rename to `${ENV_OUT}` (`delulu \
                 sandbox attest` does)"
            ),
            Refusal::NotADocument(e) => format!("the attestation is not a `{FORMAT}` document ({e})"),
            Refusal::WrongFormat(f) => format!("the attestation's format is `{f}`, and this build reads `{FORMAT}`"),
            Refusal::BadStatement(e) => format!("the attestation's statement is refused: {e}"),
            Refusal::WrongNonce => {
                "the attestation was not made for this run: it carries another nonce (a replayed or stale document)"
                    .to_string()
            }
            Refusal::BadSignature(e) => format!("the attestation's signature does not verify ({e})"),
            Refusal::WrongKey { pinned, signer } => format!(
                "the attestation is signed by {signer}, and the key this run pinned is {pinned}"
            ),
        };
        format!("{why}. Nothing ran: the program was never sent to the guest")
    }
}

/// Check a document's bytes against the pinned key and this run's nonce.
pub fn verify(bytes: &[u8], pinned: &str, nonce: &str) -> Result<Attested, Refusal> {
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(Refusal::TooLarge);
    }
    // A document that ends early was read mid-write, or never finished: said as that, not as a parser's
    // position (routine run 4 — a launcher writing in place lost this race on a macOS runner). Its last
    // character may be cut too: bytes that end INSIDE a character are the same case, not "not UTF-8".
    let text = std::str::from_utf8(bytes).map_err(|e| match e.error_len() {
        None => Refusal::Incomplete,
        Some(_) => Refusal::NotADocument("it is not UTF-8".to_string()),
    })?;
    let doc: Document =
        serde_json::from_str(text).map_err(|e| if e.is_eof() { Refusal::Incomplete } else { Refusal::NotADocument(e.to_string()) })?;
    if doc.format != FORMAT {
        return Err(Refusal::WrongFormat(doc.format.chars().take(64).collect()));
    }
    check_statement(&doc.statement).map_err(Refusal::BadStatement)?;
    let sig = crate::breakglass::decode_hex(&doc.signature).ok_or_else(|| Refusal::BadSignature("it is not hex".to_string()))?;
    // The signature first, so what is compared below is what the signer said.
    let signer = match delulu_runtime::plugin::verify_detached(&signed_bytes(&doc.statement), &sig) {
        delulu_runtime::plugin::SignatureStatus::Valid { signer } => signer,
        delulu_runtime::plugin::SignatureStatus::Invalid { reason } => return Err(Refusal::BadSignature(reason)),
        other => return Err(Refusal::BadSignature(format!("{other:?}"))),
    };
    if !signer.eq_ignore_ascii_case(pinned) {
        return Err(Refusal::WrongKey { pinned: pinned.to_string(), signer });
    }
    if doc.statement.nonce != nonce {
        return Err(Refusal::WrongNonce);
    }
    Ok(Attested { key: pinned.to_string(), attester: doc.statement.attester, guarantees: doc.statement.guarantees })
}

/// Wait for the launcher's document at `path`, then check it. `ended` answers whether the launcher has
/// exited (and how), so a launcher that dies without attesting is refused at once, not at the deadline.
pub fn await_and_verify(
    path: &std::path::Path,
    pinned: &str,
    nonce: &str,
    deadline: std::time::Duration,
    mut ended: impl FnMut() -> Option<String>,
) -> Result<Attested, Refusal> {
    let until = std::time::Instant::now() + deadline;
    loop {
        // ATTEST-FIFO-1: this read happens before any watchdog runs, and the launcher chose what is at
        // the path. A named pipe held the host in a blocking `open` for ever (witnessed); a link would
        // have had it read whatever the link names. So: a regular file, judged without following a
        // link, opened without following one and without blocking, and judged again on the handle —
        // the thing opened, not the name, is what is read.
        match std::fs::symlink_metadata(path) {
            Ok(m) if !m.file_type().is_file() => return Err(Refusal::NotAFile),
            Ok(m) if m.len() > MAX_DOCUMENT_BYTES => return Err(Refusal::TooLarge),
            Ok(_) => {
                use std::io::Read as _;
                let mut bytes = Vec::new();
                let f = open_document(path).map_err(|e| match e.raw_os_error() {
                    #[cfg(unix)]
                    Some(libc::ELOOP) => Refusal::NotAFile,
                    _ => Refusal::Unreadable(e.to_string()),
                })?;
                if !f.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    return Err(Refusal::NotAFile);
                }
                // Bounded whatever the file does between the size check and the read.
                f.take(MAX_DOCUMENT_BYTES + 1).read_to_end(&mut bytes).map_err(|e| Refusal::Unreadable(e.to_string()))?;
                return verify(&bytes, pinned, nonce);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Refusal::Unreadable(e.to_string())),
        }
        if let Some(status) = ended() {
            // One last look: a launcher may write the document and exit at once.
            if path.exists() {
                continue;
            }
            return Err(Refusal::LauncherEnded(status));
        }
        if std::time::Instant::now() >= until {
            return Err(Refusal::Absent(deadline));
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// Open the document without following a link and without blocking on a pipe (Unix); elsewhere the
/// `symlink_metadata` check before it and the handle's own metadata after it are what hold.
fn open_document(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    let mut o = std::fs::OpenOptions::new();
    o.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        o.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    o.open(path)
}

// ----- `delulu sandbox attest` — the reference (software) attester --------------------------------

/// `delulu sandbox attest --key SEED --attester NAME (--guarantee TEXT)… -- COMMAND…`
///
/// Run by the host as (the front of) an external launcher: it signs a statement over the nonce in
/// `$DELULU_ATTEST_NONCE`, writes it whole to `$DELULU_ATTEST_OUT`, and then becomes COMMAND — the
/// launcher proper, or the guest itself — with the channel still on its standard input and output. It
/// prints nothing on standard output, which belongs to the channel.
pub fn cmd_attest(args: &[String]) -> i32 {
    let cut = args.iter().position(|a| a == "--");
    let (flags, command) = match cut {
        Some(i) => (&args[..i], &args[i + 1..]),
        None => (args, &[][..]),
    };
    match attest(flags, command) {
        Ok(code) => code,
        Err((code, why)) => {
            eprintln!("error: {why}");
            code
        }
    }
}

fn attest(flags: &[String], command: &[String]) -> Result<i32, (i32, String)> {
    let usage = |why: String| (2, why);
    let mut key = None;
    let mut attester = None;
    let mut guarantees = Vec::new();
    let mut i = 0;
    while i < flags.len() {
        let a = flags[i].as_str();
        if a == "attest" && i == 0 {
            i += 1;
            continue;
        }
        let (name, inline) = match a.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n, Some(v.to_string())),
            _ => (a, None),
        };
        if !matches!(name, "--key" | "--attester" | "--guarantee") {
            return Err(usage(if a == "--json" {
                "`sandbox attest` has no `--json`: its standard output is the guest's channel, and an envelope there \
                 would be read as a frame"
                    .to_string()
            } else {
                format!("`sandbox attest` does not take `{a}` (the command it runs goes after `--`)")
            }));
        }
        let value = match inline {
            Some(v) => v,
            None => {
                i += 1;
                flags.get(i).filter(|v| !v.starts_with("--")).cloned().ok_or_else(|| usage(format!("`{name}` needs a value")))?
            }
        };
        match name {
            "--key" if key.is_none() => key = Some(value),
            "--attester" if attester.is_none() => attester = Some(value),
            "--guarantee" => guarantees.push(value),
            _ => return Err(usage(format!("`{name}` is given more than once"))),
        }
        i += 1;
    }
    let key = key.ok_or_else(|| usage("`--key SEEDFILE` names the private key that signs (see `delulu keygen`)".to_string()))?;
    let attester = attester.ok_or_else(|| usage("`--attester NAME` says who is vouching: the report shows it".to_string()))?;
    if guarantees.is_empty() {
        return Err(usage("at least one `--guarantee TEXT`: an attestation that claims nothing vouches for nothing".to_string()));
    }
    let Some((program, program_args)) = command.split_first() else {
        return Err(usage("the command to run goes after `--` — the launcher, or `delulu __guest --stdio-pipes`".to_string()));
    };
    let nonce = std::env::var(ENV_NONCE).map_err(|_| {
        usage(format!(
            "`${ENV_NONCE}` is not set: `sandbox attest` runs as an external launcher for a host that asked for \
             attestation (`delulu run … --sandbox --sandbox-backend external:… --require-attestation HEX`)"
        ))
    })?;
    let out = std::env::var_os(ENV_OUT).ok_or_else(|| usage(format!("`${ENV_OUT}` is not set: the host names where the attestation goes")))?;
    if nonce.len() != 64 || crate::breakglass::decode_hex(&nonce).is_none() {
        return Err(usage(format!("`${ENV_NONCE}` is not a 32-byte nonce in hex")));
    }
    let seed: [u8; 32] = match std::fs::read(&key) {
        Ok(b) if b.len() == 32 => b.try_into().expect("32 bytes"),
        Ok(b) => return Err(usage(format!("`{key}` is {} bytes, not a 32-byte ed25519 seed", b.len()))),
        Err(e) => return Err(usage(crate::cli::unreadable(&key, &e))),
    };
    let doc = sign(&seed, Statement { attester, guarantees, nonce }).map_err(usage)?;
    let text = serde_json::to_string_pretty(&doc).expect("a document serializes");
    // Whole or not at all: the host reads the file the moment it exists.
    let out = std::path::PathBuf::from(out);
    let tmp = out.with_file_name(format!(".{FILE_NAME}.{}.tmp", std::process::id()));
    std::fs::write(&tmp, format!("{text}\n")).map_err(|e| (1, format!("cannot write the attestation to `{}`: {e}", tmp.display())))?;
    std::fs::rename(&tmp, &out).map_err(|e| (1, format!("cannot put the attestation in place at `{}`: {e}", out.display())))?;
    let mut c = std::process::Command::new(program);
    c.args(program_args).env_remove(ENV_NONCE).env_remove(ENV_OUT);
    // Unix: BECOME the command, so the host's watchdog and its kill reach what it started. Windows has no
    // exec; the command is waited for with the same standard handles, and its exit is this process's.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        let e = c.exec();
        Err((1, format!("`{program}` could not be started: {e}")))
    }
    #[cfg(not(unix))]
    {
        match c.status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => Err((1, format!("`{program}` could not be started: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [7; 32];
    const OTHER: [u8; 32] = [9; 32];

    fn statement(nonce: &str) -> Statement {
        Statement {
            attester: "ci-image-builder".to_string(),
            guarantees: vec!["gVisor runsc".to_string(), "no network".to_string()],
            nonce: nonce.to_string(),
        }
    }

    fn nonce() -> String {
        "ab".repeat(32)
    }

    fn doc_bytes(d: &Document) -> Vec<u8> {
        serde_json::to_vec(d).unwrap()
    }

    #[test]
    fn a_statement_signed_by_the_pinned_key_for_this_run_verifies_and_is_the_attesters_word() {
        let pinned = delulu_runtime::plugin::public_key_hex(&SEED);
        let d = sign(&SEED, statement(&nonce())).unwrap();
        let a = verify(&doc_bytes(&d), &pinned, &nonce()).unwrap();
        assert_eq!(a.attester, "ci-image-builder");
        assert_eq!(a.guarantees, vec!["gVisor runsc", "no network"]);
        assert_eq!(a.to_json()["verified"], true);
        assert_eq!(a.to_json()["key"], pinned.as_str());
        // Whitespace and key order in the FILE do not matter: what is signed is the canonical form.
        let pretty = serde_json::to_string_pretty(&serde_json::json!({
            "signature": d.signature, "statement": { "nonce": nonce(), "guarantees": ["gVisor runsc", "no network"], "attester": "ci-image-builder" },
            "format": FORMAT,
        }))
        .unwrap();
        assert!(verify(pretty.as_bytes(), &pinned, &nonce()).is_ok());
    }

    #[test]
    fn the_canonical_form_is_the_one_the_module_documents() {
        let s = Statement { attester: "a \"q\" \\ é".to_string(), guarantees: vec!["g".to_string()], nonce: "00".to_string() };
        assert_eq!(
            String::from_utf8(signed_bytes(&s)).unwrap(),
            "delulu-attestation-v1\n{\"attester\":\"a \\\"q\\\" \\\\ é\",\"guarantees\":[\"g\"],\"nonce\":\"00\"}"
        );
    }

    #[test]
    fn every_way_an_attestation_can_be_wrong_is_refused_and_named() {
        let pinned = delulu_runtime::plugin::public_key_hex(&SEED);
        let good = sign(&SEED, statement(&nonce())).unwrap();

        // Another run's document: replay.
        let other_run = sign(&SEED, statement(&"cd".repeat(32))).unwrap();
        assert_eq!(verify(&doc_bytes(&other_run), &pinned, &nonce()), Err(Refusal::WrongNonce));

        // A valid signature by a key nobody pinned.
        let stranger = sign(&OTHER, statement(&nonce())).unwrap();
        assert!(matches!(verify(&doc_bytes(&stranger), &pinned, &nonce()), Err(Refusal::WrongKey { .. })));

        // A claim added after signing.
        let mut widened = good.clone();
        widened.statement.guarantees.push("hardware-attested".to_string());
        assert!(matches!(verify(&doc_bytes(&widened), &pinned, &nonce()), Err(Refusal::BadSignature(_))));

        // The pinned key's own bytes swapped in front of a stranger's signature.
        let mut spliced = stranger.clone();
        spliced.signature = format!("{pinned}{}", &stranger.signature[64..]);
        assert!(matches!(verify(&doc_bytes(&spliced), &pinned, &nonce()), Err(Refusal::BadSignature(_))));

        let mut wrong_format = good.clone();
        wrong_format.format = "delulu-attestation-v2".to_string();
        assert!(matches!(verify(&doc_bytes(&wrong_format), &pinned, &nonce()), Err(Refusal::WrongFormat(_))));

        // An unknown field is refused, never dropped: in the statement and in the document.
        let mut v: serde_json::Value = serde_json::to_value(&good).unwrap();
        v["statement"]["level"] = serde_json::json!(4);
        assert!(matches!(verify(&serde_json::to_vec(&v).unwrap(), &pinned, &nonce()), Err(Refusal::NotADocument(_))));
        let mut v: serde_json::Value = serde_json::to_value(&good).unwrap();
        v["verified"] = serde_json::json!(true);
        assert!(matches!(verify(&serde_json::to_vec(&v).unwrap(), &pinned, &nonce()), Err(Refusal::NotADocument(_))));

        assert!(matches!(verify(b"not json", &pinned, &nonce()), Err(Refusal::NotADocument(_))));
        // Read mid-write: nothing yet, or a prefix of a good document. Never a document, and said so.
        let whole = doc_bytes(&good);
        for cut in [0, 1, whole.len() / 2, whole.len() - 2] {
            assert_eq!(verify(&whole[..cut], &pinned, &nonce()), Err(Refusal::Incomplete), "cut at {cut}");
        }
        // ... cut inside a character: a claim in another script, stopped after the first of its bytes.
        let mut accented = statement(&nonce());
        accented.guarantees.push("géré par l'opérateur".to_string());
        let whole = doc_bytes(&sign(&SEED, accented).unwrap());
        let inside = whole.iter().position(|&b| b >= 0x80).expect("a multi-byte character") + 1;
        assert!(std::str::from_utf8(&whole[..inside]).is_err(), "the cut is inside the character");
        assert_eq!(verify(&whole[..inside], &pinned, &nonce()), Err(Refusal::Incomplete));
        assert!(matches!(verify(&[0xff, 0xfe], &pinned, &nonce()), Err(Refusal::NotADocument(_))));
        assert_eq!(verify(&vec![b' '; 70_000], &pinned, &nonce()), Err(Refusal::TooLarge));

        let mut not_hex = good.clone();
        not_hex.signature = "zz".repeat(96);
        assert!(matches!(verify(&doc_bytes(&not_hex), &pinned, &nonce()), Err(Refusal::BadSignature(_))));
    }

    #[test]
    fn a_statement_that_could_fill_a_terminal_or_a_report_is_refused_on_both_sides() {
        let pinned = delulu_runtime::plugin::public_key_hex(&SEED);
        for bad in [
            Statement { guarantees: vec![], ..statement(&nonce()) },
            Statement { attester: " ".to_string(), ..statement(&nonce()) },
            Statement { attester: "evil\u{1b}[2J".to_string(), ..statement(&nonce()) },
            Statement { guarantees: vec!["x".repeat(MAX_TEXT_CHARS + 1)], ..statement(&nonce()) },
            Statement { guarantees: vec!["g".to_string(); MAX_GUARANTEES + 1], ..statement(&nonce()) },
        ] {
            assert!(sign(&SEED, bad.clone()).is_err(), "the attester refuses to sign {bad:?}");
            // Signed anyway, by an attester that does not check: the host refuses it too.
            let sig = crate::breakglass::encode_hex(&delulu_runtime::plugin::sign_detached(&SEED, &signed_bytes(&bad)));
            let d = Document { format: FORMAT.to_string(), statement: bad.clone(), signature: sig };
            assert!(matches!(verify(&doc_bytes(&d), &pinned, &nonce()), Err(Refusal::BadStatement(_))), "{bad:?}");
        }
    }

    #[test]
    fn a_pinned_key_must_be_a_key_and_a_nonce_is_fresh() {
        let k = delulu_runtime::plugin::public_key_hex(&SEED);
        assert_eq!(pinned_key(&k.to_ascii_uppercase()).unwrap(), k);
        assert!(pinned_key("abc").is_err());
        assert!(pinned_key(&"g".repeat(64)).is_err());
        let (a, b) = (fresh_nonce().unwrap(), fresh_nonce().unwrap());
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }

    #[test]
    fn waiting_ends_at_once_when_the_launcher_ends_and_at_the_deadline_when_it_stays_silent() {
        let dir = std::env::temp_dir().join(format!("delulu-attest-wait-{}-{}", std::process::id(), crate::guest::channel_tag()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE_NAME);
        let pinned = delulu_runtime::plugin::public_key_hex(&SEED);
        let t = std::time::Instant::now();
        let r = await_and_verify(&path, &pinned, &nonce(), std::time::Duration::from_secs(30), || Some("exit 0".to_string()));
        assert_eq!(r, Err(Refusal::LauncherEnded("exit 0".to_string())));
        assert!(t.elapsed() < std::time::Duration::from_secs(5));
        let r = await_and_verify(&path, &pinned, &nonce(), std::time::Duration::from_millis(200), || None);
        assert_eq!(r, Err(Refusal::Absent(std::time::Duration::from_millis(200))));
        // Written and then the launcher ended at once: the document still counts.
        std::fs::write(&path, doc_bytes(&sign(&SEED, statement(&nonce())).unwrap())).unwrap();
        assert!(await_and_verify(&path, &pinned, &nonce(), std::time::Duration::from_secs(5), || Some("exit 0".to_string())).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
