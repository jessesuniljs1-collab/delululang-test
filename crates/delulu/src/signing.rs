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
        crate::cli::note_json_emitted();
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
    // Stage 10 phase 10i (Track G): `--hybrid` routes through `pqc::sign_hybrid` instead of the
    // classical-only path below. `hybrid == false` (the default) takes the ORIGINAL branch,
    // byte-for-byte — this flag is purely additive, never a change to existing behaviour.
    let hybrid = rest.iter().any(|a| a == "--hybrid");
    let unstable = rest.iter().any(|a| a == "--unstable");
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
    let sig = if hybrid {
        match delulu_runtime::pqc::sign_hybrid(&seed, &data, unstable) {
            Ok(s) => s,
            Err(delulu_runtime::pqc::Verdict::Refused { code, reason }) => {
                report_refusal("sign", &artifact, code, &reason, json);
                return 1;
            }
            // `sign_hybrid` only ever returns `Refused` today; matched generically (rather than
            // assumed) so this stays honest if that ever stops being true.
            Err(other) => {
                eprintln!("error: unexpected refusal from sign_hybrid: {other:?}");
                return 2;
            }
        }
    } else {
        delulu_runtime::sign_detached(&seed, &data)
    };
    let sig_path = format!("{artifact}.sig");
    if let Err(e) = std::fs::write(&sig_path, &sig) {
        eprintln!("error: cannot write {sig_path}: {e}");
        return 2;
    }
    let signer = delulu_runtime::public_key_hex(&seed);
    if json {
        let mut obj = json!({ "command": "sign", "artifact": artifact, "signature": sig_path, "signer": signer });
        if hybrid {
            obj["algorithms"] = json!([delulu_runtime::pqc::ALG_ED25519, delulu_runtime::pqc::ALG_ML_DSA_65]);
        }
        crate::cli::note_json_emitted();
        println!("{obj}");
    } else if hybrid {
        println!(
            "sign: wrote {sig_path} (signed by {signer}, hybrid {}+{})",
            delulu_runtime::pqc::ALG_ED25519,
            delulu_runtime::pqc::ALG_ML_DSA_65
        );
    } else {
        println!("sign: wrote {sig_path} (signed by {signer})");
    }
    0
}

pub fn cmd_verify_sig(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let expect_key = flag(rest, "--key");
    // Stage 10 phase 10i (Track G): `--require-hybrid` swaps in `Policy::HybridRequired`; with
    // neither flag the policy is `AcceptClassical` and `unstable` is `false` — the ORIGINAL
    // defaults — so a plain classical signature verifies exactly as it always has.
    let require_hybrid = rest.iter().any(|a| a == "--require-hybrid");
    let unstable = rest.iter().any(|a| a == "--unstable");
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
            report_sig("verify-sig", &artifact, &SignatureStatus::Unsigned, json);
            return 1;
        }
    };
    let policy = if require_hybrid {
        delulu_runtime::pqc::Policy::HybridRequired
    } else {
        delulu_runtime::pqc::Policy::AcceptClassical
    };
    // `pqc::verify` subsumes `verify_detached`: given a legacy 96-byte signature under
    // `Policy::AcceptClassical`/`unstable=false` (the defaults), it calls the SAME
    // `verify_detached` internally and returns the same signer/reason — so the default path
    // below is byte-for-byte what this command has always printed.
    let status = match delulu_runtime::pqc::verify(&data, &sig, policy, unstable) {
        delulu_runtime::pqc::Verdict::Valid { signer, .. } => SignatureStatus::Valid { signer },
        delulu_runtime::pqc::Verdict::Invalid { reason } => SignatureStatus::Invalid { reason },
        delulu_runtime::pqc::Verdict::Refused { code, reason } => {
            report_refusal("verify-sig", &artifact, code, &reason, json);
            return 1;
        }
    };
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
    // ABSENT and INVALID are different faults and now get different codes (campaign C38).
    //
    // Both used to report DL1705, "signature verification failed" — which for an unsigned artifact is
    // not even true: nothing was verified. The distinction is the one that matters most here. "No
    // signature exists" is a policy question and is often benign; "a signature exists and does not
    // verify" is an attack indicator — tampered content, or the wrong key. A caller handed the same
    // code for both cannot tell them apart, and `docs/for-agents.md` is explicit that the CODE is the
    // contract and the message is not.
    //
    // This is not a new principle: the project already RULED it (Stage-6 deviation 8, whose own test
    // asserts "badly-signed vs unsigned are DIFFERENT faults"), and the plugin path implements it with
    // DL1510 vs DL1511. The detached path simply never followed the rule. DL1511 already means "the
    // artifact carries no signature", so it is reused rather than a new number minted.
    let code = match status {
        SignatureStatus::Unsigned => "DL1511",
        _ => "DL1705",
    };
    if json {
        let mut obj = json!({ "command": command, "artifact": artifact, "verdict": verdict });
        match status {
            SignatureStatus::Valid { signer } => {
                obj["signer"] = json!(signer);
            }
            _ => {
                obj["code"] = json!(code);
                obj["detail"] = json!(detail);
            }
        }
        crate::cli::note_json_emitted();
        println!("{obj}");
    } else {
        match status {
            SignatureStatus::Valid { signer } => println!("verify-sig: {artifact} — valid, signed by {signer}"),
            _ => eprintln!("verify-sig[{code}]: {artifact} — {verdict}: {detail}"),
        }
    }
}

/// Print a POLICY refusal from `pqc` — `Verdict::Refused { code, reason }`, i.e. DL1908 or DL1910.
/// Deliberately NOT routed through [`report_sig`]: `SignatureStatus` has no variant for "the CLI
/// declined to attempt this", and giving it one would blur the line `pqc::Verdict` draws on purpose
/// between "a signature was checked and did not verify" (always a fault) and "no check was even
/// run" (a policy decision). Shaped like `report_publish` below: a bare `{code}: {artifact} —
/// {reason}` line, or the JSON equivalent with a `code` field.
fn report_refusal(command: &str, artifact: &str, code: &str, reason: &str, json: bool) {
    if json {
        crate::cli::note_json_emitted();
        println!("{}", json!({ "command": command, "artifact": artifact, "code": code, "error": reason }));
    } else {
        eprintln!("{code}: {artifact} — {reason}");
    }
}

// ===== registry client =====================================================

/// `delulu login --registry <url> --token <value>` (Stage 9g, spec §5).
///
/// Stores a publish token for a registry. Tokens are **scoped and revocable** server-side; the
/// client's only job is to hold one without leaking it, so the file is created 0600 on unix by the
/// same helper that protects signing keys.
///
/// The token is never echoed back. A credential printed to a terminal ends up in a scrollback
/// buffer, a screen recording, and a CI log.
pub fn cmd_login(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let registry = flag(rest, "--registry").unwrap_or_else(|| "http://127.0.0.1:8765".into());
    let Some(token) = flag(rest, "--token") else {
        eprintln!(
            "error: `login` needs --token <value>\n  \
             get one from the registry operator: `delulu-registry issue-token --owner you --scope <package>`"
        );
        return 2;
    };
    let Some(home) = home() else {
        eprintln!("error: cannot resolve ~/.delulu (no HOME/USERPROFILE)");
        return 2;
    };
    if let Err(e) = std::fs::create_dir_all(&home) {
        eprintln!("error: cannot create {}: {e}", home.display());
        return 2;
    }
    let path = home.join("credentials.jsonl");
    // One line per registry; a later login for the same registry supersedes the earlier one.
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut out = String::new();
    for line in existing.lines().filter(|l| !l.trim().is_empty()) {
        let keep = serde_json::from_str::<Value>(line)
            .map(|v| v["registry"].as_str() != Some(registry.as_str()))
            .unwrap_or(true);
        if keep {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(&json!({ "registry": registry, "token": token }).to_string());
    out.push('\n');
    if let Err(e) = std::fs::write(&path, out) {
        eprintln!("error: cannot write {}: {e}", path.display());
        return 2;
    }
    lock_down(&path);

    if json {
        // The token itself is deliberately absent from the output.
        crate::cli::note_json_emitted();
        println!("{}", json!({ "command": "login", "registry": registry, "stored": path.display().to_string() }));
    } else {
        println!("login: token stored for {registry}");
        println!("  credentials: {} (not echoed)", path.display());
    }
    0
}

/// The stored token for a registry, if any.
pub fn stored_token(registry: &str) -> Option<String> {
    let path = home()?.join("credentials.jsonl");
    let text = std::fs::read_to_string(path).ok()?;
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v["registry"].as_str() == Some(registry))
        .and_then(|v| v["token"].as_str().map(str::to_string))
}

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

    // If a registry is named, report whether we hold a token for it — before the publisher gets
    // as far as an upload that would fail on authentication.
    let registry = flag(rest, "--registry");
    let have_token = registry.as_deref().map(|r| stored_token(r).is_some());

    if json {
        crate::cli::note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "publish", "mode": "dry-run", "name": name, "version": version,
                "signed": sig_present, "index_line": line,
                "registry": registry, "authenticated": have_token,
                "prior_version": prior.as_ref().and_then(|p| p.get("version").cloned()),
            })
        );
    } else {
        println!("publish --dry-run: {name} {version} is publishable");
        if !sig_present {
            println!("  note: no signature found — sign the artifact before a real publish (`delulu sign`)");
        }
        if let (Some(r), Some(false)) = (&registry, have_token) {
            println!("  note: no stored token for {r} — run `delulu login --registry {r} --token …`");
        }
        println!("  index line: {line}");
        // The authority in this line is the CLIENT's computation. The registry recomputes it from
        // the uploaded artifact and stores its own answer; a mismatch is refused (DL1706). Said
        // here so a publisher is never surprised by a refusal they could not have predicted.
        println!("  note: the registry recomputes this authority from the artifact and refuses a mismatch");
    }
    0
}

/// `delulu add <pkg> --index <dir>` (spec §7): resolve via the index, show the authority
/// summary + diff against the requested pin, from the INDEX LINE ALONE (no download).
pub fn cmd_add(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let index = flag(rest, "--index");
    // `--path` is the form that works today: there is no hosted registry, so a real dependency is
    // a directory beside yours. Until now the only way to declare one was to hand-write the TOML
    // and guess the authority pin, discovering the right value by reading DL1001.
    if let Some(dir) = flag(rest, "--path") {
        return add_path_dependency(&dir, rest.iter().any(|a| a == "--accept-authority"), json);
    }
    let Some(pkg) = positional(rest) else {
        eprintln!("error: `add` needs a package name, or `--path <dir>` for a local dependency");
        return 2;
    };
    let Some(index) = index else {
        eprintln!("error: `add` needs a local `--index <dir>` in v0.8 (hosted registry is Stage 9)");
        eprintln!("note: for a package beside yours, `delulu add --path <dir>`");
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
        crate::cli::note_json_emitted();
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

/// `delulu add --path <dir>` — declare a dependency on the package in that directory.
///
/// **The pin is computed, not guessed.** A dependency line carries `authority = { effects = … }`,
/// which is the ceiling the *consumer* grants; get it wrong and you learn the right answer by
/// reading DL1001. The toolchain already knows it — it is exactly what `delulu authority <dir>`
/// reports and exactly what `delulu publish` stamps into an index line — so this writes that.
///
/// **But it will not grant authority on your behalf.** A pure dependency is added outright: there
/// is no decision to make, because the package cannot do anything. A dependency that needs an
/// effect is *shown and refused*, with the command to accept it printed. Adding a dependency is
/// the moment a supply chain acquires new authority, and a tool that quietly widened a manifest at
/// that moment would be doing the one thing this language exists to prevent. The rule is the same
/// one `delulu fix` follows for authority-widening repairs.
fn add_path_dependency(dir: &str, accept: bool, json: bool) -> i32 {
    let here = std::path::Path::new("delulu.toml");
    if !here.is_file() {
        eprintln!("error: no `delulu.toml` here — `add` declares a dependency of the package you are in");
        eprintln!("note: `cd` into the package, or create one with `delulu new <name>`");
        return 2;
    }
    let dep_dir = std::path::Path::new(dir);
    if !dep_dir.join("delulu.toml").is_file() {
        eprintln!("error: `{dir}` is not a DeluluLang package — no `delulu.toml` there");
        return 2;
    }
    let Some(name) = manifest_package_name(&dep_dir.join("delulu.toml")) else {
        eprintln!("error: `{dir}/delulu.toml` does not name a package (`[package] name = \"…\"`)");
        return 2;
    };

    let Ok(current) = std::fs::read_to_string(here) else {
        eprintln!("error: cannot read `delulu.toml`");
        return 2;
    };
    // Never silently rewrite an existing pin: that is precisely the line a reviewer reads.
    if current.lines().any(|l| l.trim_start().starts_with(&format!("{name} ="))) {
        eprintln!("error: `{name}` is already a dependency here");
        eprintln!("note: `add` never rewrites an existing pin — that line is the one a reviewer reads");
        return 2;
    }

    // The dependency's OWN computed authority: the same number `delulu authority` prints and
    // `publish` stamps. Computing it means checking the dependency, so a broken one is refused
    // here rather than pinned at a value nobody could verify.
    let Some(report) = crate::cli::package_authority_value(dir) else {
        eprintln!("error: `{dir}` does not check clean, so its authority cannot be computed");
        eprintln!("note: `delulu check {dir}` — a pin nobody can verify is worse than no pin");
        return 1;
    };
    let effects: Vec<String> = report
        .get("effects")
        .and_then(|e| e.as_array())
        .map(|xs| xs.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();

    let rel = dir.replace('\\', "/");
    let list = effects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
    let line = format!("{name} = {{ path = \"{rel}\", authority = {{ effects = [{list}] }} }}");

    if !effects.is_empty() && !accept {
        if json {
            crate::cli::note_json_emitted();
            println!(
                "{}",
                json!({
                    "command": "add", "name": name, "path": rel, "written": false,
                    "authority": { "effects": effects },
                    "refused": "granting authority to a dependency is a decision for a person",
                })
            );
        } else {
            eprintln!("add: `{name}` needs authority you have not granted — nothing was written.");
            eprintln!("\n  it can perform: {{{}}}", effects.join(", "));
            eprintln!("\nAdding it grants your package's supply chain that authority. Review it with");
            eprintln!("  delulu authority {dir}");
            eprintln!("and if it is what you intend:");
            eprintln!("  delulu add --path {dir} --accept-authority");
        }
        return 1;
    }

    let mut next = current.clone();
    if !next.contains("[dependencies]") {
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str("\n# What this package may reach, and what it is granted. The `authority` on\n");
        next.push_str("# each line is a CEILING the dependency cannot exceed — `delulu check` refuses\n");
        next.push_str("# with DL1001 when it tries, and `delulu authority --diff` treats a later\n");
        next.push_str("# widening of one of these lines as the supply-chain event it is.\n");
        next.push_str("[dependencies]\n");
    }
    if !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&line);
    next.push('\n');
    if let Err(e) = std::fs::write(here, &next) {
        eprintln!("error: cannot write `delulu.toml`: {e}");
        return 2;
    }

    if json {
        crate::cli::note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "add", "name": name, "path": rel, "written": true,
                "authority": { "effects": effects },
            })
        );
    } else {
        println!("add: {name} (path {rel})");
        println!(
            "  granted: {}",
            if effects.is_empty() { "nothing — it is provably pure".to_string() } else { format!("{{{}}}", effects.join(", ")) }
        );
        println!("  pinned in delulu.toml; `delulu check .` verifies it");
    }
    0
}

/// `[package] name` from a manifest, read line-wise like the rest of this toolchain's TOML.
fn manifest_package_name(path: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_package = false;
    for raw in text.lines() {
        let t = raw.trim();
        if let Some(section) = t.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_package = section.trim() == "package";
            continue;
        }
        if in_package {
            if let Some(v) = t.strip_prefix("name") {
                let v = v.trim_start().strip_prefix('=')?.trim();
                return Some(v.trim_matches('"').to_string());
            }
        }
    }
    None
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
        crate::cli::note_json_emitted();
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
