//! CRITERION 5 end-to-end: a clean machine talks to a live registry (Stage 9g).
//!
//! The in-crate tests cover the policies. This file covers the thing a user actually does: start a
//! registry, publish to it over HTTP, resolve from a **fresh** directory that has never seen the
//! package, and confirm that the registry going away does not break a build that already resolved.
//!
//! "Clean machine" here means a fresh state directory with no cached index and no prior artifacts —
//! build-order D9's local form. What is being tested is that nothing on the publishing side leaked
//! into the resolving side.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};

use delulu_registry::{hex_lower, Registry};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-c5-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn recompute(artifact: &[u8]) -> Option<Value> {
    let text = String::from_utf8_lossy(artifact);
    let mut effects = Vec::new();
    if text.contains("PERFORMS_WRITE") {
        effects.push("Write");
    }
    if text.contains("PERFORMS_NET") {
        effects.push("Net");
    }
    Some(json!({ "effects": effects, "capabilities": [], "secrets": [] }))
}

struct Client {
    addr: std::net::SocketAddr,
}

impl Client {
    fn call(&self, method: &str, path: &str, body: &str, token: &str) -> String {
        let mut s = TcpStream::connect(self.addr).expect("registry reachable");
        let req = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {token}\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    fn json_body(&self, raw: &str) -> Value {
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or("{}");
        serde_json::from_str(body).unwrap_or(json!({}))
    }
}

/// CRITERION 5: publish → resolve → "build" round-trip from a clean machine.
#[test]
fn criterion5_publish_add_build_round_trip_from_a_clean_machine() {
    let server_root = scratch("server");
    let reg = Arc::new(Registry::open(&server_root).unwrap());
    let tok = reg.issue_token("alice", vec!["widget".into()]);
    let (addr, _h) = delulu_registry::serve(reg, "127.0.0.1:0", Arc::new(recompute)).unwrap();
    let c = Client { addr };

    // --- the publisher's machine ---
    let artifact = "widget 1.0.0 PERFORMS_WRITE";
    let sig = hex_lower(&delulu_runtime::sign_detached(&[9u8; 32], artifact.as_bytes()));
    let resp = c.call(
        "POST",
        "/publish",
        &json!({"name":"widget","version":"1.0.0","artifact":artifact,"signature":sig}).to_string(),
        &tok,
    );
    assert!(resp.contains("\"published\""), "publish must succeed: {resp}");

    // --- a CLEAN machine: a fresh directory that has never seen this package ---
    let client_root = scratch("clean-client");
    assert!(
        std::fs::read_dir(&client_root).unwrap().next().is_none(),
        "the resolving machine starts with nothing cached"
    );

    let idx = c.json_body(&c.call("GET", "/index/widget", "", ""));
    let line = idx["lines"].as_array().and_then(|l| l.last()).expect("index line");
    assert_eq!(line["version"], "1.0.0");

    // `add` shows the authority BEFORE downloading anything — from the index line alone.
    assert_eq!(
        line["effects"],
        json!(["Write"]),
        "the authority summary must be readable from the index line, before any download"
    );
    assert_eq!(line["authority_source"], "recomputed-server-side");

    // Cache it the way a client would, then "build" offline against the cache.
    let cache = client_root.join("index");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(cache.join("widget"), format!("{line}\n")).unwrap();

    let offline = delulu_registry::read_local_line(&cache, "widget").expect("cached line resolves");
    assert_eq!(offline["version"], "1.0.0");
}

/// A registry outage must degrade to the lockfile/vendored path, never break a build that already
/// resolved. Spec §5: "Registry outage degrades to lockfile/vendored builds (never blocks existing
/// users' CI)."
#[test]
fn a_registry_outage_does_not_break_an_already_resolved_build() {
    let server_root = scratch("outage-server");
    let reg = Arc::new(Registry::open(&server_root).unwrap());
    let tok = reg.issue_token("alice", vec!["widget".into()]);
    let (addr, _h) = delulu_registry::serve(reg, "127.0.0.1:0", Arc::new(recompute)).unwrap();
    let c = Client { addr };

    let artifact = "widget 1.0.0 PERFORMS_WRITE";
    let sig = hex_lower(&delulu_runtime::sign_detached(&[9u8; 32], artifact.as_bytes()));
    c.call(
        "POST",
        "/publish",
        &json!({"name":"widget","version":"1.0.0","artifact":artifact,"signature":sig}).to_string(),
        &tok,
    );

    // The client resolves once and caches — this is the lockfile's job.
    let cache = scratch("outage-cache").join("index");
    std::fs::create_dir_all(&cache).unwrap();
    let idx = c.json_body(&c.call("GET", "/index/widget", "", ""));
    let line = idx["lines"].as_array().and_then(|l| l.last()).cloned().expect("line");
    std::fs::write(cache.join("widget"), format!("{line}\n")).unwrap();

    // --- the registry goes away ---
    // Point at a port with nothing behind it: the honest simulation of an outage, since the
    // client's job is to not need the network at all once it has resolved.
    let dead = Client { addr: "127.0.0.1:1".parse().unwrap() };
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dead.call("GET", "/index/widget", "", "")
        }))
        .is_err(),
        "the outage simulation must genuinely be unreachable, or this test proves nothing"
    );

    // The cached resolution still works. No network was involved, which is the entire point.
    let offline = delulu_registry::read_local_line(&cache, "widget")
        .expect("an already-resolved build must survive the registry being down");
    assert_eq!(offline["version"], "1.0.0");
    assert_eq!(offline["effects"], json!(["Write"]));
}

/// The doctored line, end to end over HTTP, from a clean client's point of view: what the registry
/// serves is what the artifact does — not what the publisher said it does.
#[test]
fn criterion5_a_doctored_index_line_never_reaches_a_client() {
    let server_root = scratch("doctored-e2e");
    let reg = Arc::new(Registry::open(&server_root).unwrap());
    let tok = reg.issue_token("mallory", vec!["backdoor".into()]);
    let (addr, _h) = delulu_registry::serve(reg, "127.0.0.1:0", Arc::new(recompute)).unwrap();
    let c = Client { addr };

    // Mallory publishes something that reaches the network while claiming to be pure.
    let artifact = "backdoor 1.0.0 PERFORMS_NET";
    let sig = hex_lower(&delulu_runtime::sign_detached(&[3u8; 32], artifact.as_bytes()));
    let resp = c.call(
        "POST",
        "/publish",
        &json!({
            "name": "backdoor", "version": "1.0.0", "artifact": artifact, "signature": sig,
            "authority": {"effects": [], "capabilities": [], "secrets": []}
        })
        .to_string(),
        &tok,
    );
    assert!(resp.contains("DL1706"), "the doctored publish must be refused: {resp}");

    // Nothing was published, so a client asking for it finds nothing to install.
    let idx = c.call("GET", "/index/backdoor", "", "");
    assert!(idx.contains("404"), "a refused publish must leave no index entry: {idx}");

    // Now the honest version of the same package: published, and the index tells the truth.
    let resp2 = c.call(
        "POST",
        "/publish",
        &json!({"name":"backdoor","version":"1.0.0","artifact":artifact,"signature":sig}).to_string(),
        &tok,
    );
    assert!(resp2.contains("\"published\""), "{resp2}");
    let idx2 = c.json_body(&c.call("GET", "/index/backdoor", "", ""));
    assert_eq!(
        idx2["lines"][0]["effects"],
        json!(["Net"]),
        "the client is told what the artifact actually does"
    );
}
