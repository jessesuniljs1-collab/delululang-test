//! Feature accounting for the egress client is a gate, not a comment (D-V2-30).
//!
//! `crates/delulu-runtime/Cargo.toml` explains at length why `gzip`, `brotli`, `zstd`, `deflate`,
//! `cookies`, `charset`, `http2` and `json` are refused: a decompressor defeats the response-size
//! bound, a cookie jar is ambient authority, a second protocol is a second parser. A comment
//! explaining that decompression is off does not keep decompression off. Cargo UNIFIES features: any
//! crate anywhere in the graph that asks for `reqwest/gzip` turns it on for everyone, and the manifest
//! line that this test also reads would still look exactly as virtuous as it does today.
//!
//! So the question is asked of the RESOLVED graph — `cargo metadata`, the same resolution a build
//! makes — and answered from what is actually enabled.

use std::collections::BTreeSet;

/// Enabled for a reason recorded in D-V2-30, plus what those two imply inside reqwest.
const ALLOWED: &[&str] = &[
    "blocking",
    "rustls-tls-native-roots",
    "rustls-tls-native-roots-no-provider",
    "__rustls",
    "__rustls-ring",
    "__tls",
];

/// Refused, each for a reason recorded beside the dependency. `default` is here because it carries
/// `default-tls` (OpenSSL/native-tls), `charset`, `http2` and `system-proxy` all at once.
const REFUSED: &[&str] = &[
    "default",
    "gzip",
    "brotli",
    "zstd",
    "deflate",
    "cookies",
    "charset",
    "http2",
    "http3",
    "json",
    "multipart",
    "stream",
    "socks",
    "hickory-dns",
    "native-tls",
    "native-tls-alpn",
    "native-tls-vendored",
    "default-tls",
    "system-proxy",
    "macos-system-configuration",
    "rustls-tls-webpki-roots",
    "rustls-tls-webpki-roots-no-provider",
];

fn repo() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn resolved_reqwest_features() -> BTreeSet<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = std::process::Command::new(cargo)
        .current_dir(repo())
        .args(["metadata", "--format-version", "1", "--offline", "--locked"])
        .output()
        .expect("cargo metadata runs");
    assert!(out.status.success(), "cargo metadata failed:\n{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("cargo metadata is JSON");
    let nodes = v["resolve"]["nodes"].as_array().expect("a resolve graph");
    let reqwest: Vec<&serde_json::Value> = nodes
        .iter()
        .filter(|n| {
            let id = n["id"].as_str().unwrap_or("");
            // `registry+...#reqwest@0.12.28` (new form) or `reqwest 0.12.28 (registry+...)` (old form).
            id.contains("#reqwest@") || id.starts_with("reqwest ")
        })
        .collect();
    assert_eq!(reqwest.len(), 1, "exactly one reqwest in the graph: {reqwest:?}");
    reqwest[0]["features"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()).collect()
}

#[test]
fn no_refused_reqwest_feature_is_enabled_anywhere_in_the_resolved_graph() {
    let on = resolved_reqwest_features();
    let refused: Vec<&String> = on.iter().filter(|f| REFUSED.contains(&f.as_str())).collect();
    assert!(
        refused.is_empty(),
        "refused reqwest features are ENABLED in the resolved graph: {refused:?} (all enabled: {on:?}). \
         Something asked for them — find it with `cargo tree -e features -i reqwest`. Each was refused \
         for a reason in crates/delulu-runtime/Cargo.toml; a decompressor, for one, defeats the \
         response-size bound."
    );
    let unexplained: Vec<&String> = on.iter().filter(|f| !ALLOWED.contains(&f.as_str())).collect();
    assert!(
        unexplained.is_empty(),
        "reqwest features enabled that D-V2-30 does not account for: {unexplained:?}. Add each to \
         ALLOWED with its reason, or turn it off."
    );
    for needed in ["blocking", "rustls-tls-native-roots"] {
        assert!(on.contains(needed), "`{needed}` is what the client is built on: {on:?}");
    }
}

/// The manifest line itself: default features off, and exactly the two named.
#[test]
fn the_manifest_names_exactly_the_features_the_ruling_allows() {
    let text = std::fs::read_to_string(repo().join("crates/delulu-runtime/Cargo.toml")).unwrap();
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("reqwest ="))
        .expect("the reqwest dependency line");
    assert!(line.contains("default-features = false"), "{line}");
    assert!(line.contains("features = [\"blocking\", \"rustls-tls-native-roots\"]"), "{line}");
    for f in REFUSED {
        assert!(!line.contains(&format!("\"{f}\"")), "`{f}` is refused: {line}");
    }
}
