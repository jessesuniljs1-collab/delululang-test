//! wasmtime's compilation cache stays out of the build (2026-10-27).
//!
//! `crates/delulu-wasm/Cargo.toml` turns wasmtime's `cache` feature off: the cache keeps compiled
//! native code in a directory on disk and loads it back, no code here turns it on, and compiling it in
//! carried `zstd` 0.13 and the four advisories of 2026-10-27 (RUSTSEC-2026-0391, -0398, -0403, -0410).
//! A manifest line does not keep a feature off. Cargo UNIFIES features: any crate anywhere in the
//! graph that asks for `wasmtime/cache`, or for wasmtime's `default` (which carries it), turns it on
//! for everyone, and the line in our manifest would still read as it does today.
//!
//! So the question is asked of the RESOLVED graph — `cargo metadata`, the same resolution a build
//! makes — as `delulu-runtime/tests/egress_features.rs` asks it of reqwest.

use std::collections::BTreeSet;

/// Refused: `cache` itself, and `default`, which turns it on.
const REFUSED: &[&str] = &["cache", "default"];

/// The crates only the cache brings: present in the graph means the cache is compiled in.
const CACHE_ONLY: &[&str] = &["wasmtime-internal-cache", "zstd", "zstd-safe", "zstd-sys"];

fn resolve() -> Vec<serde_json::Value> {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = std::process::Command::new(cargo)
        .current_dir(repo)
        .args(["metadata", "--format-version", "1", "--offline", "--locked"])
        .output()
        .expect("cargo metadata runs");
    assert!(out.status.success(), "cargo metadata failed:\n{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("cargo metadata is JSON");
    v["resolve"]["nodes"].as_array().expect("a resolve graph").clone()
}

/// A node's package name: `registry+...#name@1.2.3` (new form) or `name 1.2.3 (registry+...)` (old).
fn name(node: &serde_json::Value) -> String {
    let id = node["id"].as_str().unwrap_or("");
    match id.rsplit_once('#') {
        Some((_, tail)) => tail.split('@').next().unwrap_or("").to_string(),
        None => id.split(' ').next().unwrap_or("").to_string(),
    }
}

#[test]
fn wasmtime_is_resolved_without_its_compilation_cache() {
    let nodes = resolve();
    let wasmtime: Vec<&serde_json::Value> = nodes.iter().filter(|n| name(n) == "wasmtime").collect();
    assert_eq!(wasmtime.len(), 1, "exactly one wasmtime in the graph: {wasmtime:?}");
    let on: BTreeSet<String> =
        wasmtime[0]["features"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()).collect();
    // The control: the query reads the real wasmtime, the one that compiles and runs guests.
    for needed in ["cranelift", "runtime", "pooling-allocator"] {
        assert!(on.contains(needed), "wasmtime's `{needed}` is off, so this is not the graph that runs guests: {on:?}");
    }
    let refused: Vec<&String> = on.iter().filter(|f| REFUSED.contains(&f.as_str())).collect();
    assert!(
        refused.is_empty(),
        "wasmtime features refused in crates/delulu-wasm/Cargo.toml are ENABLED in the resolved graph: {refused:?} \
         (all enabled: {on:?}). Something asked for them — find it with `cargo tree -e features -i wasmtime`."
    );
    let present: Vec<String> = nodes.iter().map(name).filter(|n| CACHE_ONLY.contains(&n.as_str())).collect();
    assert!(present.is_empty(), "crates only wasmtime's cache brings are in the graph: {present:?}");
}
