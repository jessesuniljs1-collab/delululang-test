//! Scoped, revocable publish tokens (Stage 9g, spec §5).
//!
//! Two properties matter and both are about what a token *cannot* do:
//!
//! 1. **A token is scoped to named packages.** Publishing `widget` with a token issued for
//!    `gadget` is refused. The default is not "everything the owner has" — it is exactly the list
//!    the token was issued with, because a leaked token should cost its scope and no more.
//! 2. **A token never grants yank on someone else's package.** Yank is destructive to consumers'
//!    resolution; a stolen token must not be able to reach across the registry with it.
//!
//! Revocation is immediate and permanent: the record stays with `revoked: true` rather than being
//! deleted, so the log of what existed is not rewritten by revoking it.

use std::path::Path;

use serde_json::{json, Value};

#[derive(Debug, PartialEq, Eq)]
pub enum Authorization {
    Ok,
    /// No such token, or it has been revoked. Deliberately one answer: telling an attacker which
    /// of the two it was is a free oracle.
    Unknown,
    OutOfScope,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub value: String,
    pub owner: String,
    /// Packages this token may publish or yank. Empty means **nothing** — never "all".
    pub scopes: Vec<String>,
    pub revoked: bool,
}

#[derive(Default)]
pub struct Store {
    tokens: Vec<Token>,
    counter: u64,
}

impl Store {
    pub fn load(path: &Path) -> std::io::Result<Store> {
        let mut store = Store::default();
        let Ok(text) = std::fs::read_to_string(path) else { return Ok(store) };
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
            store.tokens.push(Token {
                value: v["value"].as_str().unwrap_or_default().to_string(),
                owner: v["owner"].as_str().unwrap_or_default().to_string(),
                scopes: v["scopes"]
                    .as_array()
                    .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
                revoked: v["revoked"].as_bool().unwrap_or(false),
            });
        }
        store.counter = store.tokens.len() as u64;
        Ok(store)
    }

    fn persist(&self, path: &Path) {
        let mut out = String::new();
        for t in &self.tokens {
            out.push_str(
                &json!({
                    "value": t.value, "owner": t.owner, "scopes": t.scopes, "revoked": t.revoked
                })
                .to_string(),
            );
            out.push('\n');
        }
        let _ = std::fs::write(path, out);
    }

    /// Issue a token. The value is derived from OS randomness where available; the fallback is
    /// only reached if `getrandom` fails, and it is still unique per issue.
    pub fn issue(&mut self, owner: &str, scopes: Vec<String>, path: &Path) -> String {
        self.counter += 1;
        let mut raw = [0u8; 24];
        let value = if getrandom::fill(&mut raw).is_ok() {
            format!("dlt_{}", crate::hex_lower(&raw))
        } else {
            // OS randomness unavailable. The fallback is unique but NOT unguessable, so it is
            // named as such rather than passed off as a token — an operator seeing this in a
            // token file knows the registry needs attention.
            format!("dlt_INSECURE_FALLBACK_{}_{}", owner, self.counter)
        };
        self.tokens.push(Token {
            value: value.clone(),
            owner: owner.to_string(),
            scopes,
            revoked: false,
        });
        self.persist(path);
        value
    }

    /// Revoke. Returns whether a live token was actually revoked.
    pub fn revoke(&mut self, value: &str, path: &Path) -> bool {
        let mut hit = false;
        for t in &mut self.tokens {
            if t.value == value && !t.revoked {
                t.revoked = true;
                hit = true;
            }
        }
        if hit {
            self.persist(path);
        }
        hit
    }

    /// May this token act on `package`?
    ///
    /// Fail-closed at every step: an empty token string, an unknown value, a revoked token, or an
    /// empty scope list all refuse. There is no branch here that reaches `Ok` by default.
    pub fn authorize(&self, value: &str, package: &str) -> Authorization {
        if value.is_empty() {
            return Authorization::Unknown;
        }
        let Some(t) = self.tokens.iter().find(|t| t.value == value) else {
            return Authorization::Unknown;
        };
        if t.revoked {
            return Authorization::Unknown;
        }
        if t.scopes.iter().any(|s| s == package) {
            Authorization::Ok
        } else {
            Authorization::OutOfScope
        }
    }
}
