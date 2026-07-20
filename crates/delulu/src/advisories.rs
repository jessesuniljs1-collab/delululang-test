//! The advisory-feed detector (Stage 10 phase 10k, Track C, spec §4).
//!
//! `delulu build` consults a local advisory feed and warns (DL1903) when a resolved dependency is
//! on a version named in a published security advisory. `--deny-advisories` turns every match into
//! an error — the CI gate a `delulu build` in a pipeline runs behind.
//!
//! **The feed is local, and consulted offline.** It is a JSON file (default `delulu.advisories.json`
//! next to the package, `--advisory-feed <path>` to point elsewhere) synced out-of-band from the
//! registry's advisory feed (`delulu-registry advisory export`). Reading a local copy — rather than
//! phoning the registry at build time — is deliberate: the registry being down must never break a
//! build (the same property the lockfile gives dependency resolution), and it must never *silence*
//! an advisory a build already holds either.
//!
//! **Matching is exact.** An advisory names a package and a list of affected version *strings*; a
//! dependency is affected iff its resolved version is one of them. There is deliberately no semver
//! range parsing here: a range predicate that cannot be evaluated against some unusual version
//! string would be a place the checker silently answers "not affected" because it could not tell —
//! precisely the fail-open this detector exists to avoid. Exact membership is total: every version
//! string is either in the list or it is not.
//!
//! This module is pure — it loads and matches. `cli.rs` turns the result into diagnostics, because
//! only there is the `--deny-advisories` policy (warning vs error, and what a feed that cannot be
//! read means under the gate) decided.

use std::path::Path;

use serde_json::Value;

/// One advisory, as a build reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Advisory {
    pub id: String,
    pub package: String,
    pub affected: Vec<String>,
    pub patched: Option<String>,
    pub severity: String,
    pub summary: String,
}

impl Advisory {
    /// Parse one feed record. Returns `None` for a record missing the load-bearing fields (`id`,
    /// `package`, a non-empty `affected` list of strings) — the caller counts those as *malformed*
    /// rather than dropping them silently, so "a record we could not read" is never mistaken for
    /// "a record that did not match".
    pub fn from_value(v: &Value) -> Option<Advisory> {
        let id = v.get("id").and_then(Value::as_str).filter(|s| !s.is_empty())?;
        let package = v.get("package").and_then(Value::as_str).filter(|s| !s.is_empty())?;
        let affected_raw = v.get("affected").and_then(Value::as_array)?;
        if affected_raw.is_empty() || !affected_raw.iter().all(Value::is_string) {
            return None;
        }
        let affected = affected_raw.iter().filter_map(|x| x.as_str().map(String::from)).collect();
        Some(Advisory {
            id: id.to_string(),
            package: package.to_string(),
            affected,
            patched: v.get("patched").and_then(Value::as_str).map(String::from),
            severity: v.get("severity").and_then(Value::as_str).unwrap_or("unknown").to_string(),
            summary: v.get("summary").and_then(Value::as_str).unwrap_or("").to_string(),
        })
    }

    /// Whether this advisory covers `name`@`version`. Exact package name, exact version membership.
    pub fn affects(&self, name: &str, version: &str) -> bool {
        self.package == name && self.affected.iter().any(|v| v == version)
    }
}

/// The result of trying to load a feed. The three outcomes are kept distinct because the gate
/// (`--deny-advisories`) treats them differently: `Absent` without the gate is silence, but under
/// the gate it is a refusal; `Unreadable` is a refusal under the gate and a visible warning without
/// it; `Loaded` may still carry `malformed` records the gate refuses to enforce over.
#[derive(Debug)]
pub enum FeedStatus {
    /// No feed file exists at the resolved path.
    Absent,
    /// A file exists but could not be read or parsed as a feed at all. Carries the human reason.
    Unreadable(String),
    /// A feed was read. `malformed` counts records present-but-unparseable (see [`Advisory::from_value`]).
    Loaded { advisories: Vec<Advisory>, malformed: usize },
}

/// Load the feed at `path`. Absent file → [`FeedStatus::Absent`] (not an error on its own). A file
/// that is not valid JSON, or lacks an `advisories` array, → [`FeedStatus::Unreadable`].
pub fn load_feed(path: &Path) -> FeedStatus {
    if !path.exists() {
        return FeedStatus::Absent;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return FeedStatus::Unreadable(format!("cannot read `{}`: {e}", path.display())),
    };
    let doc: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return FeedStatus::Unreadable(format!("`{}` is not valid JSON: {e}", path.display())),
    };
    let Some(arr) = doc.get("advisories").and_then(Value::as_array) else {
        return FeedStatus::Unreadable(format!(
            "`{}` has no `advisories` array (expected `{{ \"advisories\": [ … ] }}`)",
            path.display()
        ));
    };
    let mut advisories = Vec::new();
    let mut malformed = 0usize;
    for rec in arr {
        match Advisory::from_value(rec) {
            Some(a) => advisories.push(a),
            None => malformed += 1,
        }
    }
    FeedStatus::Loaded { advisories, malformed }
}

/// One match: a resolved dependency that an advisory covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub package: String,
    pub version: String,
    pub advisory_id: String,
    pub severity: String,
    pub summary: String,
    pub patched: Option<String>,
}

/// Every (dependency, advisory) match, in a stable order (by package, then version, then id) so a
/// build's diagnostics are reproducible regardless of feed or dependency ordering.
pub fn scan(advisories: &[Advisory], deps: &[(String, String)]) -> Vec<Hit> {
    let mut hits = Vec::new();
    for (name, version) in deps {
        for a in advisories {
            if a.affects(name, version) {
                hits.push(Hit {
                    package: name.clone(),
                    version: version.clone(),
                    advisory_id: a.id.clone(),
                    severity: a.severity.clone(),
                    summary: a.summary.clone(),
                    patched: a.patched.clone(),
                });
            }
        }
    }
    hits.sort_by(|x, y| {
        (&x.package, &x.version, &x.advisory_id).cmp(&(&y.package, &y.version, &y.advisory_id))
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adv(id: &str, pkg: &str, affected: &[&str], patched: Option<&str>) -> Advisory {
        Advisory {
            id: id.into(),
            package: pkg.into(),
            affected: affected.iter().map(|s| s.to_string()).collect(),
            patched: patched.map(String::from),
            severity: "high".into(),
            summary: "test".into(),
        }
    }

    #[test]
    fn exact_membership_is_total_no_range_guessing() {
        let a = adv("DLSA-1", "http-mini", &["1.2.0", "1.2.1"], Some("1.2.2"));
        assert!(a.affects("http-mini", "1.2.0"));
        assert!(a.affects("http-mini", "1.2.1"));
        // Not in the list — including the patched version and an unusual string — is simply "not
        // affected". Nothing is guessed.
        assert!(!a.affects("http-mini", "1.2.2"));
        assert!(!a.affects("http-mini", "1.2.0-weird+build.7"));
        assert!(!a.affects("other-pkg", "1.2.0"));
    }

    #[test]
    fn from_value_rejects_half_records() {
        // Missing affected, empty affected, and non-string affected are all malformed — never a
        // silent "matches nothing".
        assert!(Advisory::from_value(&serde_json::json!({"id": "x", "package": "p"})).is_none());
        assert!(Advisory::from_value(&serde_json::json!({"id": "x", "package": "p", "affected": []})).is_none());
        assert!(Advisory::from_value(&serde_json::json!({"id": "x", "package": "p", "affected": [1, 2]})).is_none());
        assert!(Advisory::from_value(&serde_json::json!({"id": "", "package": "p", "affected": ["1.0.0"]})).is_none());
        assert!(Advisory::from_value(&serde_json::json!({"package": "p", "affected": ["1.0.0"]})).is_none());
        // A whole record is accepted.
        let ok = Advisory::from_value(&serde_json::json!({
            "id": "DLSA-1", "package": "p", "affected": ["1.0.0"], "patched": "1.0.1"
        }));
        assert_eq!(ok, Some(adv_no_summary("DLSA-1", "p", &["1.0.0"], Some("1.0.1"))));
    }

    fn adv_no_summary(id: &str, pkg: &str, affected: &[&str], patched: Option<&str>) -> Advisory {
        Advisory {
            id: id.into(),
            package: pkg.into(),
            affected: affected.iter().map(|s| s.to_string()).collect(),
            patched: patched.map(String::from),
            severity: "unknown".into(),
            summary: String::new(),
        }
    }

    #[test]
    fn scan_is_stable_and_finds_every_match() {
        let advisories = vec![
            adv("DLSA-2", "b", &["2.0.0"], None),
            adv("DLSA-1", "a", &["1.0.0"], Some("1.0.1")),
        ];
        let deps = vec![
            ("a".to_string(), "1.0.0".to_string()),
            ("b".to_string(), "2.0.0".to_string()),
            ("c".to_string(), "9.9.9".to_string()), // no advisory
        ];
        let hits = scan(&advisories, &deps);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].package, "a"); // stable: sorted by package
        assert_eq!(hits[1].package, "b");
    }
}
