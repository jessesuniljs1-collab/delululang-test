//! Signing + the registry client groundwork (Stage 8, phase 8h; spec §7).
//!
//! - `delulu keygen` — mint an ed25519 signing seed (`~/.delulu/keys/`, 0600 on unix).
//! - `delulu sign <artifact>` / `delulu verify-sig <artifact> [--key HEX]` — DETACHED
//!   signatures (`<artifact>.sig`, 96 bytes) over the raw bytes of a `.dwx`/`.dpx`/tarball;
//!   the Stage-6 in-band `delulu:sig` mechanism generalized (build-order deviation 16).
//! - The registry **index format** (normative now, hosted in Stage 9): a cargo-style
//!   sparse index whose every line carries the authority summary, so `delulu add` shows
//!   the authority diff BEFORE downloading anything.
//! - `delulu publish --dry-run` / `delulu add <pkg>` — validate + resolve against a LOCAL
//!   index fixture (build-order deviation 4: the HTTP shape is frozen; no network in v0.8).
//!
//! Signing authenticates ORIGIN, not behavior; the index's authority line is the
//! publisher's verified-at-publish claim, re-verified on first build (trust-on-first-verify,
//! spec §11). None of this is a trust POLICY — that is Stage 9 governance.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use delulu_runtime::SignatureStatus;

fn home() -> Option<PathBuf> {
    std::env::var_os("DELULU_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|h| PathBuf::from(h).join(".delulu"))
        })
}

fn keys_dir() -> Option<PathBuf> {
    Some(home()?.join("keys"))
}

/// Read the default signing seed (`keys/id_ed25519`), or a named one.
fn read_seed(name: Option<&str>) -> Result<[u8; 32], String> {
    let dir = keys_dir().ok_or("cannot resolve ~/.delulu/keys (no HOME/USERPROFILE)")?;
    let path = dir.join(name.unwrap_or("id_ed25519"));
    let bytes = std::fs::read(&path).map_err(|e| format!("cannot read signing key {}: {e}", path.display()))?;
    if bytes.len() != 32 {
        return Err(format!("signing key {} is {} bytes, not 32", path.display(), bytes.len()));
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Ok(seed)
}

#[cfg(test)]
fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

#[cfg(unix)]
fn lock_down(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}
#[cfg(not(unix))]
fn lock_down(_path: &Path) {
    // Windows: the file lands under the per-user profile; ACL hardening is a documented
    // v0.8 gap (the broker's key files carry the same caveat).
}

pub fn cmd_keygen(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let name = flag(rest, "--name");
    let Some(dir) = keys_dir() else {
        eprintln!("error: cannot resolve ~/.delulu/keys (no HOME/USERPROFILE)");
        return 2;
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("error: cannot create {}: {e}", dir.display());
        return 2;
    }
    let key_path = dir.join(name.as_deref().unwrap_or("id_ed25519"));
    if key_path.exists() {
        eprintln!("error: {} already exists — remove it first or pass --name", key_path.display());
        return 1;
    }
    let mut seed = [0u8; 32];
    if getrandom::fill(&mut seed).is_err() {
        eprintln!("error: OS randomness (getrandom) unavailable");
        return 2;
    }
    if let Err(e) = std::fs::write(&key_path, seed) {
        eprintln!("error: cannot write {}: {e}", key_path.display());
        return 2;
    }
    lock_down(&key_path);
    let pubhex = delulu_runtime::public_key_hex(&seed);
    let pub_path = key_path.with_extension("pub");
    let _ = std::fs::write(&pub_path, format!("{pubhex}\n"));
    if json {
        println!(
            "{}",
            json!({ "command": "keygen", "key": key_path.display().to_string(), "public_key": pubhex })
        );
    } else {
        println!("keygen: wrote {} (public key {pubhex})", key_path.display());
        println!("  keep the private key secret; distribute {}", pub_path.display());
    }
    0
}

pub fn cmd_sign(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let name = flag(rest, "--key-name");
    let Some(artifact) = positional(rest) else {
        eprintln!("error: `sign` needs an artifact (.dwx / .dpx / tarball)");
        return 2;
    };
    let data = match std::fs::read(&artifact) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot read {artifact}: {e}");
            return 2;
        }
    };
    let seed = match read_seed(name.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}\n  run `delulu keygen` first");
            return 1;
        }
    };
    let sig = delulu_runtime::sign_detached(&seed, &data);
    let sig_path = format!("{artifact}.sig");
    if let Err(e) = std::fs::write(&sig_path, &sig) {
        eprintln!("error: cannot write {sig_path}: {e}");
        return 2;
    }
    let signer = delulu_runtime::public_key_hex(&seed);
    if json {
        println!("{}", json!({ "command": "sign", "artifact": artifact, "signature": sig_path, "signer": signer }));
    } else {
        println!("sign: wrote {sig_path} (signed by {signer})");
    }
    0
}

pub fn cmd_verify_sig(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let expect_key = flag(rest, "--key");
    let Some(artifact) = positional(rest) else {
        eprintln!("error: `verify-sig` needs an artifact");
        return 2;
    };
    let data = match std::fs::read(&artifact) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot read {artifact}: {e}");
            return 2;
        }
    };
    let sig_path = format!("{artifact}.sig");
    let sig = match std::fs::read(&sig_path) {
        Ok(s) => s,
        Err(_) => {
            // No signature is a FAILURE, and it must fail through the exit code — a caller gating
            // on `$?` is the common case, and "unsigned" printed alongside exit 0 would be read as
            // success. Pinned by `an_unsigned_artifact_fails_verification_by_exit_code`.
            report_sig("verify-sig", &artifact, &SignatureStatus::Unsigned, json);
            return 1;
        }
    };
    let status = delulu_runtime::verify_detached(&data, &sig);
    // `--key <hex>`: the signature must ALSO be by exactly this key (pin the identity).
    if let (SignatureStatus::Valid { signer }, Some(want)) = (&status, &expect_key) {
        if !signer.eq_ignore_ascii_case(want.trim()) {
            let d = SignatureStatus::Invalid {
                reason: format!("signed by {signer}, but --key pinned {want}"),
            };
            report_sig("verify-sig", &artifact, &d, json);
            return 1;
        }
    }
    let ok = matches!(status, SignatureStatus::Valid { .. });
    report_sig("verify-sig", &artifact, &status, json);
    i32::from(!ok)
}

fn report_sig(command: &str, artifact: &str, status: &SignatureStatus, json: bool) {
    let (verdict, detail): (&str, String) = match status {
        SignatureStatus::Valid { signer } => ("valid", signer.clone()),
        SignatureStatus::Unsigned => ("unsigned", "no <artifact>.sig found".into()),
        SignatureStatus::Invalid { reason } => ("invalid", reason.clone()),
    };
    if json {
        // DL1705 for a failed verification (spec §10), requires_human by nature.
        let mut obj = json!({ "command": command, "artifact": artifact, "verdict": verdict });
        match status {
            SignatureStatus::Valid { signer } => {
                obj["signer"] = json!(signer);
            }
            _ => {
                obj["code"] = json!("DL1705");
                obj["detail"] = json!(detail);
            }
        }
        println!("{obj}");
    } else {
        match status {
            SignatureStatus::Valid { signer } => println!("verify-sig: {artifact} — valid, signed by {signer}"),
            _ => eprintln!("verify-sig[DL1705]: {artifact} — {verdict}: {detail}"),
        }
    }
}

// ===== registry client =====================================================

/// `delulu publish --dry-run <package-dir> --index <dir>` (spec §7). Validates manifest
/// completeness, semver-authority (DL1003 reused) against the index's prior line, and
/// signature presence — but performs NO upload (Stage 9). All local.
pub fn cmd_publish(rest: &[String], authority_of: impl Fn(&str) -> Option<Value>) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let dry = rest.iter().any(|a| a == "--dry-run");
    let index = flag(rest, "--index");
    let Some(pkg) = positional(rest) else {
        eprintln!("error: `publish` needs a package directory");
        return 2;
    };
    if !dry {
        eprintln!("error: only `publish --dry-run` is supported in v0.8 (hosted upload is Stage 9)");
        return 2;
    }
    let manifest_path = Path::new(&pkg).join("delulu.toml");
    let manifest = match std::fs::read_to_string(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: cannot read {}: {e}", manifest_path.display());
            return 2;
        }
    };
    let name = toml_scalar(&manifest, "name");
    let version = toml_scalar(&manifest, "version");
    let (Some(name), Some(version)) = (name, version) else {
        report_publish(&pkg, "DL1706", "manifest is missing `name` or `version`", json);
        return 1;
    };

    // The authority this build actually computes (the summary the index line will carry).
    let this_authority = authority_of(&pkg);
    let effects = this_authority
        .as_ref()
        .and_then(|a| a.get("effects").and_then(Value::as_array).cloned())
        .unwrap_or_default();

    // The prior index line for this package, if the index has one.
    let prior = index.as_deref().and_then(|dir| read_index_line(dir, &name));
    if let Some(prior) = &prior {
        // Semver-authority law (DL1003, reused): a non-major bump must not WIDEN authority.
        let prior_effects: Vec<String> = prior
            .get("effects")
            .and_then(Value::as_array)
            .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let widened: Vec<String> = effects
            .iter()
            .filter_map(Value::as_str)
            .filter(|e| !prior_effects.iter().any(|p| p == e))
            .map(str::to_string)
            .collect();
        let prior_v = prior.get("version").and_then(Value::as_str).unwrap_or("0.0.0");
        if !widened.is_empty() && same_major(prior_v, &version) {
            report_publish(
                &pkg,
                "DL1003",
                &format!(
                    "version {version} adds authority {widened:?} over {prior_v} without a major bump — authority widening is semver-major"
                ),
                json,
            );
            return 1;
        }
    }

    // Signature presence: a publishable artifact should be signed (warned, not fatal, in
    // the dry run — the actual gate is Stage-9 registry policy).
    let sig_present = std::fs::metadata(format!("{pkg}.sig")).is_ok()
        || std::fs::read_dir(&pkg)
            .map(|it| it.flatten().any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("sig")))
            .unwrap_or(false);

    let line = index_line(&name, &version, &effects, this_authority.as_ref());
    if json {
        println!(
            "{}",
            json!({
                "command": "publish", "mode": "dry-run", "name": name, "version": version,
                "signed": sig_present, "index_line": line,
                "prior_version": prior.as_ref().and_then(|p| p.get("version").cloned()),
            })
        );
    } else {
        println!("publish --dry-run: {name} {version} is publishable");
        if !sig_present {
            println!("  note: no signature found — sign the artifact before a real publish (`delulu sign`)");
        }
        println!("  index line: {line}");
    }
    0
}

/// `delulu add <pkg> --index <dir>` (spec §7): resolve via the index, show the authority
/// summary + diff against the requested pin, from the INDEX LINE ALONE (no download).
pub fn cmd_add(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let index = flag(rest, "--index");
    let Some(pkg) = positional(rest) else {
        eprintln!("error: `add` needs a package name");
        return 2;
    };
    let Some(index) = index else {
        eprintln!("error: `add` needs a local `--index <dir>` in v0.8 (hosted registry is Stage 9)");
        return 2;
    };
    let Some(line) = read_index_line(&index, &pkg) else {
        report_publish(&pkg, "DL1706", "no such package in the index", json);
        return 1;
    };
    let version = line.get("version").and_then(Value::as_str).unwrap_or("?");
    let effects: Vec<&str> =
        line.get("effects").and_then(Value::as_array).map(|xs| xs.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    if json {
        println!(
            "{}",
            json!({ "command": "add", "name": pkg, "version": version, "authority": { "effects": effects }, "from": "index-line-only" })
        );
    } else {
        println!("add: {pkg} {version}");
        println!(
            "  authority (from the index, no download): {}",
            if effects.is_empty() { "pure".to_string() } else { format!("effects {effects:?}") }
        );
        println!("  (v0.8 resolves + shows authority from the index line; the download + lockfile write land with the hosted registry in Stage 9)");
    }
    0
}

/// One JSONL index line (spec §7): the authority summary rides WITH the version, so a
/// consumer sees exactly what the package can do BEFORE downloading it. Keys mirror the
/// §10.5 report: `effects`, `capabilities` (kind + scopes), `secrets`.
fn index_line(name: &str, version: &str, effects: &[Value], authority: Option<&Value>) -> String {
    let get = |k: &str| authority.and_then(|a| a.get(k)).cloned().unwrap_or(Value::Array(vec![]));
    json!({
        "name": name, "version": version,
        "effects": effects,
        "capabilities": get("capabilities"),
        "secrets": get("secrets"),
        "yanked": false,
    })
    .to_string()
}

/// Read the LATEST line for `name` from a cargo-style sparse index dir: `<dir>/<name>`.
fn read_index_line(index_dir: &str, name: &str) -> Option<Value> {
    let path = Path::new(index_dir).join(name);
    let text = std::fs::read_to_string(path).ok()?;
    text.lines().rfind(|l| !l.trim().is_empty()).and_then(|l| serde_json::from_str(l).ok())
}

fn report_publish(pkg: &str, code: &str, message: &str, json: bool) {
    if json {
        println!("{}", json!({ "command": "publish", "package": pkg, "code": code, "error": message }));
    } else {
        eprintln!("{code}: {pkg} — {message}");
    }
}

// ----- tiny helpers (no new deps) ------------------------------------------

fn flag(rest: &[String], name: &str) -> Option<String> {
    let eq = format!("{name}=");
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
        if let Some(v) = a.strip_prefix(&eq) {
            return Some(v.to_string());
        }
    }
    None
}

fn positional(rest: &[String]) -> Option<String> {
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if a == "--key" || a == "--key-name" || a == "--index" || a == "--name" {
            i += 2; // skip the flag AND its value
            continue;
        }
        if a.starts_with("--") {
            i += 1;
            continue;
        }
        return Some(a.clone());
    }
    None
}

/// A bare `key = "value"` scalar from a manifest's `[package]` region (first match wins).
fn toml_scalar(manifest: &str, key: &str) -> Option<String> {
    for line in manifest.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Same major version (semver): `1.x` vs `1.y` share a major; `0.x` treats MINOR as major
/// (cargo's 0.x rule — a 0.x authority widening still needs a 0.(x+1) bump).
fn same_major(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> (u64, u64) {
        let mut it = v.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
        (it.next().unwrap_or(0), it.next().unwrap_or(0))
    };
    let (am, an) = parts(a);
    let (bm, bn) = parts(b);
    if am == 0 || bm == 0 {
        am == bm && an == bn
    } else {
        am == bm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_major_rules() {
        assert!(same_major("1.2.0", "1.5.0"));
        assert!(!same_major("1.0.0", "2.0.0"));
        assert!(!same_major("0.1.0", "0.2.0"), "0.x: minor is the compatibility axis");
        assert!(same_major("0.1.0", "0.1.9"));
    }

    #[test]
    fn hex_round_trips() {
        let b = [0xDEu8, 0xAD, 0xBE, 0xEF];
        assert_eq!(hex_decode(&hex_lower(&b)).unwrap(), b);
        assert!(hex_decode("xyz").is_none());
    }
}
