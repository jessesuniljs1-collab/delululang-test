//! The authority lockfile (`delulu.lock`), Stage 2 §4.4.
//!
//! The lock is *authority*, not just versions (invariant 11): each entry carries a `content_hash`
//! over the package's canonicalized source, an `authority_hash` over its verified effects/kinds,
//! and an `api_row_hash` over its public signatures. Builds verify these before running anything —
//! a locked dependency whose code changed (DL1010) or whose authority changed under an unchanged
//! version (DL1002) cannot build. `delulu lock` additionally enforces the semver-authority law
//! (DL1003): observable widening requires a major bump and explicit re-acceptance.

use std::collections::BTreeSet;
use std::path::Path;

use delulu_diag::Diagnostic;

use crate::deps::{api_row_dump, authority_canonical_json, package_authority, Workspace};
use crate::program::Program;

/// One `[[package]]` entry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub content_hash: String,
    pub authority_hash: String,
    pub api_row_hash: String,
    pub effects: Vec<String>,
    pub cap_kinds: Vec<String>,
    pub net: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub accepted_by: String,
}

/// A parsed `delulu.lock`.
#[derive(Clone, Debug, Default)]
pub struct Lockfile {
    pub version: u32,
    pub packages: Vec<LockEntry>,
}

impl Lockfile {
    pub fn get(&self, name: &str) -> Option<&LockEntry> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Parse a `delulu.lock`. On malformed input returns an empty lockfile (callers treat a
    /// missing/garbled lock as "nothing locked", which surfaces as DL1011 under `--locked`).
    pub fn parse(src: &str) -> Lockfile {
        let table: toml::Table = match src.parse() {
            Ok(t) => t,
            Err(_) => return Lockfile::default(),
        };
        let version = table.get("version").and_then(|v| v.as_integer()).unwrap_or(1) as u32;
        let mut packages = Vec::new();
        if let Some(arr) = table.get("package").and_then(|v| v.as_array()) {
            for p in arr {
                let Some(t) = p.as_table() else { continue };
                let s = |k: &str| t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let a = |k: &str| {
                    t.get(k)
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                        .unwrap_or_default()
                };
                let scopes = t.get("scopes").and_then(|v| v.as_table());
                let sc = |k: &str| {
                    scopes
                        .and_then(|s| s.get(k))
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                        .unwrap_or_default()
                };
                packages.push(LockEntry {
                    name: s("name"),
                    version: s("version"),
                    source: s("source"),
                    content_hash: s("content_hash"),
                    authority_hash: s("authority_hash"),
                    api_row_hash: s("api_row_hash"),
                    effects: a("effects"),
                    cap_kinds: a("cap_kinds"),
                    net: sc("net"),
                    fs_read: sc("fs.read"),
                    fs_write: sc("fs.write"),
                    accepted_by: s("accepted_by"),
                });
            }
        }
        Lockfile { version, packages }
    }

    /// Render a canonical `delulu.lock`. Packages are emitted sorted by name for stable diffs.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("version = {}\n", self.version.max(1)));
        let mut pkgs = self.packages.clone();
        pkgs.sort_by(|a, b| a.name.cmp(&b.name));
        for p in &pkgs {
            out.push_str("\n[[package]]\n");
            out.push_str(&format!("name           = {}\n", q(&p.name)));
            out.push_str(&format!("version        = {}\n", q(&p.version)));
            out.push_str(&format!("source         = {}\n", q(&p.source)));
            out.push_str(&format!("content_hash   = {}\n", q(&p.content_hash)));
            out.push_str(&format!("authority_hash = {}\n", q(&p.authority_hash)));
            out.push_str(&format!("api_row_hash   = {}\n", q(&p.api_row_hash)));
            out.push_str(&format!("effects        = {}\n", arr(&p.effects)));
            out.push_str(&format!("cap_kinds      = {}\n", arr(&p.cap_kinds)));
            out.push_str(&format!(
                "scopes         = {{ net = {}, \"fs.read\" = {}, \"fs.write\" = {} }}\n",
                arr(&p.net),
                arr(&p.fs_read),
                arr(&p.fs_write)
            ));
            out.push_str(&format!("accepted_by    = {}\n", q(&p.accepted_by)));
        }
        out
    }
}

fn q(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn arr(items: &[String]) -> String {
    let inner = items.iter().map(|s| q(s)).collect::<Vec<_>>().join(", ");
    format!("[{inner}]")
}

// ===== hashing ==================================================================================

fn h(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// `content_hash`: blake3 over `delulu.toml` bytes + each `src/**.delulu` (sorted by relative
/// path; path bytes then file bytes). Pure over the source tree — no code runs.
pub fn content_hash(dir: &Path) -> String {
    let mut hasher = blake3::Hasher::new();
    if let Ok(manifest) = std::fs::read(dir.join("delulu.toml")) {
        hasher.update(b"delulu.toml\0");
        hasher.update(&manifest);
    }
    let mut files = Vec::new();
    collect(&dir.join("src"), &mut files);
    files.sort();
    for path in files {
        let rel = path.strip_prefix(dir).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        if let Ok(bytes) = std::fs::read(&path) {
            hasher.update(rel.as_bytes());
            hasher.update(b"\0");
            hasher.update(&bytes);
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                out.push(p);
            }
        }
    }
}

/// Compute a fresh lock entry for a resolved package.
pub fn compute_entry(ws: &Workspace, program: &Program, pkg_idx: usize) -> LockEntry {
    let pkg = &ws.packages[pkg_idx];
    let auth = package_authority(ws, program, pkg_idx);
    LockEntry {
        name: pkg.name.clone(),
        version: pkg.manifest.version.clone(),
        source: pkg.source_id.clone(),
        content_hash: content_hash(&pkg.dir),
        authority_hash: h(authority_canonical_json(&auth).as_bytes()),
        api_row_hash: h(api_row_dump(ws, pkg_idx).as_bytes()),
        effects: auth.effects.iter().cloned().collect(),
        cap_kinds: auth.cap_kinds.iter().cloned().collect(),
        net: pkg.manifest.authority.net.clone(),
        fs_read: pkg.manifest.authority.fs_read.clone(),
        fs_write: pkg.manifest.authority.fs_write.clone(),
        accepted_by: String::new(),
    }
}

/// The same blake3 hash `compute_entry` uses for `LockEntry.api_row_hash`, exposed additively so
/// callers that need only the hash (e.g. `interface.json` emission, §5.5) don't have to recompute
/// a full lock entry to get it.
pub fn api_row_hash(ws: &Workspace, pkg_idx: usize) -> String {
    h(api_row_dump(ws, pkg_idx).as_bytes())
}

/// Compute the full lockfile for a checked workspace.
pub fn compute_lockfile(ws: &Workspace, program: &Program) -> Lockfile {
    let mut packages: Vec<LockEntry> = (0..ws.packages.len()).map(|i| compute_entry(ws, program, i)).collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Lockfile { version: 1, packages }
}

// ===== --locked verification (DL1010 / DL1002 / DL1011) =========================================

/// Verify a checked workspace against an existing lockfile (used by `build --locked`).
pub fn verify_locked(ws: &Workspace, program: &Program, lock: &Lockfile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (idx, pkg) in ws.packages.iter().enumerate() {
        let Some(entry) = lock.get(&pkg.name) else {
            diags.push(Diagnostic::error(
                "DL1011",
                format!("package `{}` is not present in delulu.lock — run `delulu lock`", pkg.name),
            ));
            continue;
        };
        // Content hash first (spec §4.4: "builds verify content_hash before anything else").
        let ch = content_hash(&pkg.dir);
        if ch != entry.content_hash {
            diags.push(Diagnostic::error(
                "DL1010",
                format!("package `{}` source has changed under its locked version (possible tampering) — content hash mismatch", pkg.name),
            ));
            continue;
        }
        // Then authority hash (catches "same version string, different authority").
        let auth = package_authority(ws, program, idx);
        let ah = h(authority_canonical_json(&auth).as_bytes());
        if ah != entry.authority_hash {
            diags.push(Diagnostic::error(
                "DL1002",
                format!("package `{}` has a different verified authority than its lock entry (same version, different authority)", pkg.name),
            ));
        }
    }
    diags
}

// ===== semver-authority law (DL1003) ============================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bump {
    Down,
    Same,
    Patch,
    Minor,
    Major,
}

fn parse_semver(v: &str) -> (u64, u64, u64) {
    let core = v.split(['-', '+']).next().unwrap_or(v);
    let mut it = core.split('.').map(|s| s.trim().parse::<u64>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

fn version_bump(old: &str, new: &str) -> Bump {
    let (om, oi, op) = parse_semver(old);
    let (nm, ni, np) = parse_semver(new);
    match nm.cmp(&om) {
        std::cmp::Ordering::Greater => Bump::Major,
        std::cmp::Ordering::Less => Bump::Down,
        std::cmp::Ordering::Equal => match ni.cmp(&oi) {
            std::cmp::Ordering::Greater => Bump::Minor,
            std::cmp::Ordering::Less => Bump::Down,
            std::cmp::Ordering::Equal => match np.cmp(&op) {
                std::cmp::Ordering::Greater => Bump::Patch,
                std::cmp::Ordering::Less => Bump::Down,
                std::cmp::Ordering::Equal => Bump::Same,
            },
        },
    }
}

fn subset(a: &[String], b: &[String]) -> bool {
    let bs: BTreeSet<&str> = b.iter().map(|s| s.as_str()).collect();
    a.iter().all(|x| bs.contains(x.as_str()))
}

/// True iff `new` observably widens authority relative to `old` (effects, kinds, or scopes grow).
pub fn authority_widened(old: &LockEntry, new: &LockEntry) -> bool {
    !subset(&new.effects, &old.effects)
        || !subset(&new.cap_kinds, &old.cap_kinds)
        || !subset(&new.net, &old.net)
        || !subset(&new.fs_read, &old.fs_read)
        || !subset(&new.fs_write, &old.fs_write)
}

/// Enforce the semver-authority law when re-locking. For each package present in the previous
/// lock, an authority widening (or public-API row change) that is not backed by a major version
/// bump is DL1003 unless the package name is in `accept`. Returns (diagnostics, accepted-names).
pub fn enforce_semver_law(
    old: &Lockfile,
    new: &mut Lockfile,
    accept: &[String],
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for entry in &mut new.packages {
        let Some(prev) = old.get(&entry.name) else { continue };
        let widened = authority_widened(prev, entry);
        let api_changed = prev.api_row_hash != entry.api_row_hash;
        if !widened && !api_changed {
            continue;
        }
        let bump = version_bump(&prev.version, &entry.version);
        let accepted = accept.iter().any(|a| a == &entry.name);

        if widened {
            // A widening requires BOTH a major bump AND explicit re-acceptance (§4.3).
            if accepted {
                entry.accepted_by = format!("--accept-authority {}", entry.name);
                if bump != Bump::Major {
                    diags.push(Diagnostic::error(
                        "DL1003",
                        format!(
                            "package `{}` widens authority from {} to {}; acceptance is recorded but a widening still requires a major version bump",
                            entry.name, prev.version, entry.version
                        ),
                    ));
                }
            } else {
                diags.push(Diagnostic::error(
                    "DL1003",
                    format!(
                        "package `{}` widens authority from version {} to {} without a major bump — re-lock with a major version and `--accept-authority {}`",
                        entry.name, prev.version, entry.version, entry.name
                    ),
                ));
            }
        } else if api_changed && bump == Bump::Same {
            // Public-API row changed with no version change at all.
            diags.push(Diagnostic::error(
                "DL1003",
                format!("package `{}` changes its public API row without any version bump", entry.name),
            ));
        }
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_round_trips() {
        let lock = Lockfile {
            version: 1,
            packages: vec![LockEntry {
                name: "webby".into(),
                version: "1.4.2".into(),
                source: "path+../webby".into(),
                content_hash: "blake3:af31".into(),
                authority_hash: "blake3:77b0".into(),
                api_row_hash: "blake3:d10c".into(),
                effects: vec!["Net".into()],
                cap_kinds: vec!["Http".into()],
                net: vec!["api.example.com".into()],
                fs_read: vec![],
                fs_write: vec![],
                accepted_by: String::new(),
            }],
        };
        let text = lock.render();
        let back = Lockfile::parse(&text);
        assert_eq!(back.version, 1);
        assert_eq!(back.packages.len(), 1);
        assert_eq!(back.packages[0], lock.packages[0]);
    }

    #[test]
    fn semver_bump_classification() {
        assert_eq!(version_bump("1.0.0", "2.0.0"), Bump::Major);
        assert_eq!(version_bump("1.0.0", "1.1.0"), Bump::Minor);
        assert_eq!(version_bump("1.0.0", "1.0.1"), Bump::Patch);
        assert_eq!(version_bump("1.0.0", "1.0.0"), Bump::Same);
        assert_eq!(version_bump("2.0.0", "1.0.0"), Bump::Down);
    }

    #[test]
    fn widening_with_minor_bump_is_dl1003() {
        let old = LockEntry { name: "webby".into(), version: "1.0.0".into(), effects: vec!["Net".into()], ..Default::default() };
        let new = LockEntry { name: "webby".into(), version: "1.1.0".into(), effects: vec!["Net".into(), "Write".into()], ..Default::default() };
        let oldlock = Lockfile { version: 1, packages: vec![old] };
        let mut newlock = Lockfile { version: 1, packages: vec![new] };
        let diags = enforce_semver_law(&oldlock, &mut newlock, &[]);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "DL1003");
        assert!(newlock.packages[0].accepted_by.is_empty());
    }

    #[test]
    fn widening_accepted_records_accepted_by() {
        let old = LockEntry { name: "webby".into(), version: "1.0.0".into(), effects: vec!["Net".into()], ..Default::default() };
        let new = LockEntry { name: "webby".into(), version: "2.0.0".into(), effects: vec!["Net".into(), "Write".into()], ..Default::default() };
        let oldlock = Lockfile { version: 1, packages: vec![old] };
        let mut newlock = Lockfile { version: 1, packages: vec![new] };
        let diags = enforce_semver_law(&oldlock, &mut newlock, &["webby".to_string()]);
        assert!(diags.is_empty(), "{diags:?}");
        assert!(newlock.packages[0].accepted_by.contains("--accept-authority webby"));
    }
}
