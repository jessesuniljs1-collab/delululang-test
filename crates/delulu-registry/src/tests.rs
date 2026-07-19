//! The registry's policies, tested (Stage 9g — release criterion 5).
//!
//! Every test here is about a REFUSAL. The happy path matters, but a registry is defined by what
//! it declines to vouch for.

use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

static SEQ: AtomicU32 = AtomicU32::new(0);

fn scratch(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("delulu-registry-{}-{tag}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A signing key and the signature over some bytes — the real ed25519 path, not a stub.
fn sign(data: &[u8]) -> Vec<u8> {
    let seed = [7u8; 32];
    delulu_runtime::sign_detached(&seed, data)
}

/// The stand-in for "the registry re-derives the authority from the artifact". The real server
/// runs package verification; here the artifact's own bytes carry the truth so the *policy* can be
/// tested without a compiler in the loop.
fn recompute(artifact: &[u8]) -> Option<Value> {
    let text = String::from_utf8_lossy(artifact);
    let effects: Vec<&str> = if text.contains("PERFORMS_NET") {
        vec!["Net"]
    } else if text.contains("PERFORMS_WRITE") {
        vec!["Write"]
    } else {
        vec![]
    };
    Some(json!({ "effects": effects, "capabilities": [], "secrets": [] }))
}

/// A recomputation that cannot decide — the "could not tell" case.
fn cannot_tell(_: &[u8]) -> Option<Value> {
    None
}

/// Publish `artifact` correctly signed, claiming nothing — the common shape.
fn publish_signed(r: &Registry, tok: &str, name: &str, version: &str, artifact: &[u8]) -> PublishOutcome {
    let sig = sign(artifact);
    r.publish_submission(
        &Submission { token: tok, name, version, artifact, signature: Some(&sig), claimed: None },
        recompute,
    )
}

fn reg_with_token(tag: &str, scopes: &[&str]) -> (Registry, String) {
    let r = Registry::open(scratch(tag)).unwrap();
    let t = r.issue_token("alice", scopes.iter().map(|s| s.to_string()).collect());
    (r, t)
}

// ----- criterion 5: the round trip -----------------------------------------

#[test]
fn criterion5_publish_then_resolve_round_trip() {
    let (r, tok) = reg_with_token("roundtrip", &["widget"]);
    let art = b"widget v1 PERFORMS_WRITE".to_vec();
    let sig = sign(&art);

    let out = r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        );
    assert_eq!(out, PublishOutcome::Accepted { name: "widget".into(), version: "1.0.0".into() });

    let line = r.resolve("widget").expect("resolvable after publish");
    assert_eq!(line["version"], "1.0.0");
    assert_eq!(line["effects"], json!(["Write"]));
    assert_eq!(
        line["authority_source"], "recomputed-server-side",
        "the stored authority must be recorded as recomputed, not as a publisher claim"
    );
}

// ----- criterion 5: the doctored index line ---------------------------------

/// THE NON-NEGOTIABLE (spec §5, playbook trap 6): a publisher cannot claim an authority they do
/// not carry. The client here submits a PURE authority for an artifact that performs `Net`.
#[test]
fn criterion5_a_doctored_authority_claim_is_rejected() {
    let (r, tok) = reg_with_token("doctored", &["sneaky"]);
    let art = b"sneaky v1 PERFORMS_NET".to_vec();
    let sig = sign(&art);
    let lie = json!({ "effects": [], "capabilities": [], "secrets": [] });

    let out = r.publish_submission(
            &Submission {
                token: &tok,
                name: "sneaky",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: Some(&lie),
            },
            recompute,
        );
    match out {
        PublishOutcome::Refused { code, reason } => {
            assert_eq!(code, "DL1706");
            assert!(reason.contains("does not match"), "the refusal must name the mismatch: {reason}");
        }
        other => panic!("a doctored authority claim must be REFUSED, got {other:?}"),
    }
    assert!(r.resolve("sneaky").is_none(), "nothing may be published from a doctored claim");
}

/// And the subtler half: even with NO claim submitted, the stored authority is the recomputed one.
/// A registry that only checks a claim when one is offered lets a publisher win by staying silent.
#[test]
fn the_stored_authority_is_recomputed_even_when_nothing_is_claimed() {
    let (r, tok) = reg_with_token("silent", &["quiet"]);
    let art = b"quiet v1 PERFORMS_NET".to_vec();
    let sig = sign(&art);

    r.publish_submission(
            &Submission {
                token: &tok,
                name: "quiet",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        );
    let line = r.resolve("quiet").expect("published");
    assert_eq!(
        line["effects"],
        json!(["Net"]),
        "the index must carry what the artifact does, whether or not the publisher said so"
    );
}

/// THE SKIP BRANCH: when the server cannot derive an authority, publish is REFUSED — never
/// accepted with the submitted claim standing in. "Could not tell" failing open would make the
/// whole recomputation theatre, since an attacker controls the artifact and can aim for the
/// undecidable case on purpose.
#[test]
fn when_the_server_cannot_recompute_the_publish_is_refused() {
    let (r, tok) = reg_with_token("cannot-tell", &["opaque"]);
    let art = b"opaque".to_vec();
    let sig = sign(&art);
    let claim = json!({ "effects": [], "capabilities": [], "secrets": [] });

    match r.publish_submission(
            &Submission {
                token: &tok,
                name: "opaque",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: Some(&claim),
            },
            cannot_tell,
        ) {
        PublishOutcome::Refused { code, reason } => {
            assert_eq!(code, "DL1706");
            assert!(reason.contains("could not derive"), "{reason}");
        }
        other => panic!("an underivable authority must refuse, got {other:?}"),
    }
    assert!(r.resolve("opaque").is_none());
}

// ----- signatures -----------------------------------------------------------

#[test]
fn an_unsigned_publish_is_refused() {
    let (r, tok) = reg_with_token("unsigned", &["widget"]);
    match r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.0.0",
                artifact: b"widget",
                signature: None,
                claimed: None,
            },
            recompute,
        ) {
        PublishOutcome::Refused { code, .. } => assert_eq!(code, "DL1705"),
        other => panic!("publish must require a signature, got {other:?}"),
    }
}

#[test]
fn a_signature_over_different_bytes_is_refused() {
    let (r, tok) = reg_with_token("badsig", &["widget"]);
    let sig = sign(b"some other artifact entirely");
    match r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.0.0",
                artifact: b"widget v1",
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        ) {
        PublishOutcome::Refused { code, reason } => {
            assert_eq!(code, "DL1705");
            assert!(reason.contains("does not verify"), "{reason}");
        }
        other => panic!("a signature over other bytes must refuse, got {other:?}"),
    }
}

// ----- tokens ---------------------------------------------------------------

#[test]
fn a_token_cannot_publish_outside_its_scope() {
    let (r, tok) = reg_with_token("scope", &["gadget"]);
    let art = b"widget v1".to_vec();
    let sig = sign(&art);
    match r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        ) {
        PublishOutcome::Refused { reason, .. } => {
            assert!(reason.contains("not scoped"), "{reason}");
        }
        other => panic!("a token scoped to `gadget` must not publish `widget`, got {other:?}"),
    }
}

/// A token never grants yank on someone else's package — the policy spec §5 names explicitly.
#[test]
fn a_token_cannot_yank_someone_elses_package() {
    let root = scratch("cross-yank");
    let r = Registry::open(&root).unwrap();
    let alice = r.issue_token("alice", vec!["widget".into()]);
    let mallory = r.issue_token("mallory", vec!["mallorys-pkg".into()]);

    let art = b"widget v1".to_vec();
    let sig = sign(&art);
    r.publish_submission(
            &Submission {
                token: &alice,
                name: "widget",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        );

    match r.yank(&mallory, "widget", "1.0.0", true) {
        PublishOutcome::Refused { reason, .. } => assert!(reason.contains("not scoped"), "{reason}"),
        other => panic!("cross-package yank must refuse, got {other:?}"),
    }
    let line = r.resolve("widget").expect("still resolvable");
    assert_eq!(line["yanked"], json!(false), "the package must be untouched");
}

#[test]
fn a_revoked_token_stops_working_immediately() {
    let (r, tok) = reg_with_token("revoke", &["widget"]);
    assert!(r.revoke_token(&tok), "revoking a live token reports success");
    let art = b"widget v1".to_vec();
    let sig = sign(&art);
    match r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        ) {
        PublishOutcome::Refused { reason, .. } => assert!(reason.contains("revoked"), "{reason}"),
        other => panic!("a revoked token must refuse, got {other:?}"),
    }
    assert!(!r.revoke_token(&tok), "revoking twice is not a second success");
}

#[test]
fn an_empty_or_unknown_token_is_refused() {
    let (r, _) = reg_with_token("notoken", &["widget"]);
    let art = b"widget v1".to_vec();
    let sig = sign(&art);
    for bogus in ["", "dlt_not_a_real_token", "   "] {
        match r.publish_submission(
            &Submission {
                token: bogus,
                name: "widget",
                version: "1.0.0",
                artifact: &art,
                signature: Some(&sig),
                claimed: None,
            },
            recompute,
        ) {
            PublishOutcome::Refused { .. } => {}
            other => panic!("token `{bogus}` must be refused, got {other:?}"),
        }
    }
}

// ----- yank ≠ delete --------------------------------------------------------

/// A yanked version stops satisfying NEW requirements and keeps resolving for a lockfile that
/// already pinned it. Deleting would break builds that were working; yanking stops the bleeding
/// without doing that.
#[test]
fn yank_is_not_delete() {
    let (r, tok) = reg_with_token("yank", &["widget"]);
    let sig1 = sign(b"widget v1");
    let sig2 = sign(b"widget v2");
    r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.0.0",
                artifact: b"widget v1",
                signature: Some(&sig1),
                claimed: None,
            },
            recompute,
        );
    r.publish_submission(
            &Submission {
                token: &tok,
                name: "widget",
                version: "1.1.0",
                artifact: b"widget v2",
                signature: Some(&sig2),
                claimed: None,
            },
            recompute,
        );

    assert_eq!(r.resolve("widget").unwrap()["version"], "1.1.0");
    assert!(matches!(r.yank(&tok, "widget", "1.1.0", true), PublishOutcome::Accepted { .. }));

    // New requirements fall back to the last unyanked version...
    assert_eq!(
        r.resolve("widget").unwrap()["version"],
        "1.0.0",
        "a yanked version must stop satisfying new requirements"
    );
    // ...and the yanked one is still there for a lockfile that pinned it.
    let exact = r.resolve_exact("widget", "1.1.0").expect("yank must NEVER delete");
    assert_eq!(exact["yanked"], json!(true), "it is present and flagged, not erased");

    // Un-yanking restores it — yank is reversible, deletion would not be.
    assert!(matches!(r.yank(&tok, "widget", "1.1.0", false), PublishOutcome::Accepted { .. }));
    assert_eq!(r.resolve("widget").unwrap()["version"], "1.1.0");
}

// ----- semver-authority ------------------------------------------------------

#[test]
fn widening_authority_without_a_major_bump_is_refused() {
    let (r, tok) = reg_with_token("semver", &["widget"]);
    publish_signed(&r, &tok, "widget", "1.0.0", b"widget v1");

    let net_art = b"widget v2 PERFORMS_NET";
    match publish_signed(&r, &tok, "widget", "1.1.0", net_art) {
        PublishOutcome::Refused { code, reason } => {
            assert_eq!(code, "DL1003");
            assert!(reason.contains("major bump"), "{reason}");
        }
        other => panic!("authority widening on a minor bump must refuse, got {other:?}"),
    }

    // THE SKIP BRANCH: the same widening WITH a major bump is allowed. A rule that refused both
    // would look identical in the test above while making the registry unusable.
    assert!(
        matches!(
            publish_signed(&r, &tok, "widget", "2.0.0", net_art),
            PublishOutcome::Accepted { .. }
        ),
        "a major bump may widen authority — that is what major means"
    );
}

#[test]
fn republishing_an_existing_version_is_refused() {
    let (r, tok) = reg_with_token("immutable", &["widget"]);
    publish_signed(&r, &tok, "widget", "1.0.0", b"widget v1");
    match publish_signed(&r, &tok, "widget", "1.0.0", b"widget v1 PERFORMS_NET") {
        PublishOutcome::Refused { reason, .. } => assert!(reason.contains("immutable"), "{reason}"),
        other => panic!("versions must be immutable, got {other:?}"),
    }
    assert_eq!(
        r.resolve("widget").unwrap()["effects"],
        json!([]),
        "the original must be untouched by the attempt"
    );
}

// ----- the HTTP surface ------------------------------------------------------

#[test]
fn the_http_surface_serves_the_index_and_refuses_a_doctored_publish() {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let root = scratch("http");
    let reg = Arc::new(Registry::open(&root).unwrap());
    let tok = reg.issue_token("alice", vec!["widget".into()]);
    let (addr, _h) = serve(reg.clone(), "127.0.0.1:0", Arc::new(recompute)).expect("serve");

    let request = |method: &str, path: &str, body: &str, token: &str| -> String {
        let mut s = TcpStream::connect(addr).expect("connect");
        let req = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {token}\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    };

    assert!(request("GET", "/health", "", "").contains("\"status\":\"ok\""));

    // An honest publish over the wire.
    let art = "widget v1 PERFORMS_WRITE";
    let sig = hex_lower(&sign(art.as_bytes()));
    let body = json!({"name": "widget", "version": "1.0.0", "artifact": art, "signature": sig}).to_string();
    let resp = request("POST", "/publish", &body, &tok);
    assert!(resp.contains("\"published\":\"widget\""), "{resp}");

    // The index serves the recomputed authority.
    let idx = request("GET", "/index/widget", "", "");
    assert!(idx.contains("\"effects\":[\"Write\"]"), "{idx}");
    assert!(idx.contains("recomputed-server-side"), "{idx}");

    // A doctored claim over the wire is refused exactly as it is in-process.
    let art2 = "widget v2 PERFORMS_NET";
    let sig2 = hex_lower(&sign(art2.as_bytes()));
    let lie = json!({
        "name": "widget", "version": "2.0.0", "artifact": art2, "signature": sig2,
        "authority": {"effects": []}
    })
    .to_string();
    let resp2 = request("POST", "/publish", &lie, &tok);
    assert!(resp2.contains("DL1706"), "a doctored claim must be refused over HTTP too: {resp2}");
    assert!(resp2.contains("does not match"), "{resp2}");

    // And an unauthenticated publish gets nowhere.
    let resp3 = request("POST", "/publish", &body, "");
    assert!(resp3.contains("DL1706"), "{resp3}");
}

#[test]
fn same_major_follows_the_cargo_zero_x_rule() {
    assert!(same_major("1.2.0", "1.9.0"));
    assert!(!same_major("1.0.0", "2.0.0"));
    assert!(!same_major("0.1.0", "0.2.0"), "under 0.x the minor is the compatibility axis");
    assert!(same_major("0.1.0", "0.1.7"));
}
