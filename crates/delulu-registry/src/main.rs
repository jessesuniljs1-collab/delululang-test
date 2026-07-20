//! `delulu-registry` — run the package registry (Stage 9g, spec §5).
//!
//! The authority recomputation wired in `main` shells out to the real `delulu` binary and asks it
//! what the artifact's authority is. That is the point: the registry does not reimplement the
//! authority calculation and then drift from it — it asks the compiler, which is the only thing
//! entitled to answer.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use serde_json::{json, Value};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut root = PathBuf::from("registry-data");
    let mut addr = "127.0.0.1:8765".to_string();
    let mut cmd = String::new();
    let mut sub = String::new();
    let mut owner = "local".to_string();
    let mut scopes: Vec<String> = Vec::new();
    let mut token: Option<String> = None;
    // Advisory-filing fields (`advisory file`) and the export path (`advisory export`).
    let mut adv: std::collections::BTreeMap<&'static str, String> = std::collections::BTreeMap::new();
    let mut affected: Vec<String> = Vec::new();
    let mut out: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "serve" | "issue-token" | "revoke-token" | "advisory" => cmd = args[i].clone(),
            // `advisory` takes a subcommand: `file` | `export`.
            "file" | "export" if cmd == "advisory" && sub.is_empty() => sub = args[i].clone(),
            "--root" => {
                i += 1;
                root = args.get(i).map(PathBuf::from).unwrap_or(root);
            }
            "--addr" => {
                i += 1;
                addr = args.get(i).cloned().unwrap_or(addr);
            }
            "--owner" => {
                i += 1;
                owner = args.get(i).cloned().unwrap_or(owner);
            }
            "--scope" => {
                i += 1;
                if let Some(s) = args.get(i) {
                    scopes.push(s.clone());
                }
            }
            "--token" => {
                i += 1;
                token = args.get(i).cloned();
            }
            k @ ("--id" | "--package" | "--patched" | "--severity" | "--summary") => {
                i += 1;
                let key = &k[2..]; // strip "--"
                adv.insert(
                    match key {
                        "id" => "id",
                        "package" => "package",
                        "patched" => "patched",
                        "severity" => "severity",
                        _ => "summary",
                    },
                    args.get(i).cloned().unwrap_or_default(),
                );
            }
            // `--affected v1,v2,v3` (comma-separated) — repeatable and additive.
            "--affected" => {
                i += 1;
                if let Some(s) = args.get(i) {
                    affected.extend(s.split(',').map(str::trim).filter(|p| !p.is_empty()).map(String::from));
                }
            }
            "--out" => {
                i += 1;
                out = args.get(i).cloned();
            }
            "-h" | "--help" => {
                print!("{}", usage());
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("delulu-registry: unknown argument `{other}`\n\n{}", usage());
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let reg = match delulu_registry::Registry::open(&root) {
        Ok(r) => Arc::new(r),
        Err(e) => {
            eprintln!("delulu-registry: cannot open {}: {e}", root.display());
            return ExitCode::from(2);
        }
    };

    match cmd.as_str() {
        "issue-token" => {
            if scopes.is_empty() {
                eprintln!(
                    "delulu-registry: a token needs at least one --scope <package>. A token with \
                     no scope can publish nothing, which is the safe default but rarely what you \
                     meant."
                );
                return ExitCode::from(2);
            }
            let t = reg.issue_token(&owner, scopes);
            println!("{t}");
            ExitCode::SUCCESS
        }
        "revoke-token" => {
            let Some(tok) = token.as_deref().or_else(|| scopes.first().map(String::as_str)) else {
                eprintln!("delulu-registry: revoke-token needs --token <value>");
                return ExitCode::from(2);
            };
            if reg.revoke_token(tok) {
                println!("revoked");
                ExitCode::SUCCESS
            } else {
                eprintln!("delulu-registry: no live token with that value");
                ExitCode::from(1)
            }
        }
        "advisory" => match sub.as_str() {
            "file" => {
                let Some(tok) = token.as_deref() else {
                    eprintln!(
                        "delulu-registry: `advisory file` needs a --token scoped to the package \
                         (advisories cannot be filed anonymously — the feed is load-bearing)"
                    );
                    return ExitCode::from(2);
                };
                if affected.is_empty() {
                    eprintln!(
                        "delulu-registry: `advisory file` needs at least one --affected <version> \
                         (an advisory that names no affected version warns about nothing)"
                    );
                    return ExitCode::from(2);
                }
                let record = json!({
                    "id": adv.get("id").cloned().unwrap_or_default(),
                    "package": adv.get("package").cloned().unwrap_or_default(),
                    "affected": affected,
                    "patched": adv.get("patched"),
                    "severity": adv.get("severity").cloned().unwrap_or_else(|| "unknown".into()),
                    "summary": adv.get("summary").cloned().unwrap_or_default(),
                });
                match reg.file_advisory(tok, &record) {
                    delulu_registry::PublishOutcome::Accepted { name, version } => {
                        println!("filed advisory {version} for {name}");
                        ExitCode::SUCCESS
                    }
                    delulu_registry::PublishOutcome::Refused { code, reason } => {
                        eprintln!("delulu-registry: advisory refused [{code}]: {reason}");
                        ExitCode::from(1)
                    }
                }
            }
            // Dump a whole-feed file in exactly the shape `delulu build` reads: `{ "advisories": [..] }`.
            // With --package, only that package's advisories; otherwise the whole feed.
            "export" => {
                let advisories: Vec<Value> = match adv.get("package") {
                    Some(pkg) if !pkg.is_empty() => reg.advisories(pkg),
                    _ => reg.all_advisories(),
                };
                let feed = json!({ "advisories": advisories });
                let text = serde_json::to_string_pretty(&feed).unwrap_or_else(|_| feed.to_string());
                match &out {
                    Some(path) => {
                        if let Err(e) = std::fs::write(path, format!("{text}\n")) {
                            eprintln!("delulu-registry: cannot write {path}: {e}");
                            return ExitCode::from(2);
                        }
                        println!("wrote {} advisory record(s) to {path}", advisories.len());
                    }
                    None => println!("{text}"),
                }
                ExitCode::SUCCESS
            }
            other => {
                eprintln!("delulu-registry: `advisory` needs a subcommand: file | export (got `{other}`)\n\n{}", usage());
                ExitCode::from(2)
            }
        },
        "serve" => {
            let recompute: delulu_registry::Recompute = Arc::new(recompute_authority);
            match delulu_registry::serve(reg, &addr, recompute) {
                Ok((bound, handle)) => {
                    println!("delulu-registry listening on http://{bound}");
                    println!("  index:     GET  /index/<package>");
                    println!("  advisories:GET  /advisories/<package>");
                    println!("  publish:   POST /publish    (Authorization: Bearer <token>)");
                    println!("  yank:      POST /yank       (Authorization: Bearer <token>)");
                    println!("  advisory:  POST /advisory   (Authorization: Bearer <token>)");
                    let _ = handle.join();
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("delulu-registry: cannot bind {addr}: {e}");
                    ExitCode::from(2)
                }
            }
        }
        _ => {
            eprint!("{}", usage());
            ExitCode::from(2)
        }
    }
}

/// Ask the compiler what the artifact's authority is.
///
/// Returns `None` when the answer cannot be obtained — and the registry treats `None` as a
/// refusal, never as permission to fall back on the publisher's claim. A verification step that
/// fails open is worse than no verification step, because it also produces a record saying the
/// authority was checked.
///
/// Isolation (build-order D3): the verification runs under the strongest Stage-5 profile the host
/// can provide. On a host without microVM support that is the ruled weaker fallback, and the
/// registry says which one it used rather than implying the strongest.
fn recompute_authority(artifact: &[u8]) -> Option<Value> {
    let dir = std::env::temp_dir().join(format!("delulu-reg-verify-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("upload.dwx");
    std::fs::write(&path, artifact).ok()?;

    let out = std::process::Command::new("delulu")
        .args(["authority", &path.to_string_lossy(), "--json"])
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .ok()?;
    let _ = std::fs::remove_dir_all(&dir);
    if !out.status.success() {
        return None;
    }
    let v: Value = serde_json::from_slice(&out.stdout).ok()?;
    v.get("authority").cloned()
}

fn usage() -> String {
    "delulu-registry — the DeluluLang package registry (spec §5)\n\
     \n\
     USAGE:\n\
     \x20 delulu-registry serve [--root <dir>] [--addr <host:port>]\n\
     \x20 delulu-registry issue-token --owner <who> --scope <package> [--scope <package>]...\n\
     \x20 delulu-registry revoke-token --token <value>\n\
     \x20 delulu-registry advisory file --token <t> --package <pkg> --id <DLSA-…> \\\n\
     \x20     --affected <v1,v2,…> [--patched <v>] [--severity <low|medium|high|critical>] [--summary <text>]\n\
     \x20 delulu-registry advisory export [--package <pkg>] [--out <feed.json>]\n\
     \n\
     Policies: publish requires a signature; the index line's authority is RECOMPUTED from the\n\
     uploaded artifact (a publisher cannot claim an authority they do not carry); tokens are\n\
     scoped and revocable; yank never deletes; an advisory can be filed only with a token scoped\n\
     to the package it names.\n"
        .to_string()
}
