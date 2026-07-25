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
    /// The names of the secrets this package reads (`root.secret("NAME")`). Part of the package's
    /// authority (Constitution invariant 10 lists *scopes*, and a secret name is a scope), and
    /// tracked here so the semver-authority law and `authority --diff` can see a secret-scope
    /// widening. Reading a secret adds no effect and no capability kind, so without this field a
    /// dependency could start reading a new secret on a patch bump and neither review tool would
    /// notice — see HARDENING_CAMPAIGN C18.
    pub secrets: Vec<String>,
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
                    secrets: a("secrets"),
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
            out.push_str(&format!("secrets        = {}\n", arr(&p.secrets)));
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
        secrets: auth.secrets.iter().cloned().collect(),
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

    // A lockfile this build cannot read is a lockfile it cannot verify. `version = 999` used to be
    // accepted and then interpreted as if it were version 1 — the "when the checker cannot tell, it
    // says yes" failure this project refuses everywhere else (an unverifiable certificate algorithm
    // is DL1908, not a shrug). Reported as DL1011 rather than a new code because the CONSEQUENCE is
    // exactly what DL1011 already names: under `--locked`, nothing is pinned (C52).
    if lock.version != 1 {
        diags.push(Diagnostic::error(
            "DL1011",
            format!(
                "delulu.lock declares format version {} and this toolchain understands version 1 — \
                 a lockfile it cannot read pins nothing, so no resolution is verified",
                lock.version
            ),
        ));
        return diags;
    }

    // One package, one entry. Two entries for the same name is an ambiguity, and resolving it
    // silently (by taking the first) is how a widening gets in — the rule `Scopes::device` and the
    // device grant grammar already apply (C40). A reader seeing two conflicting entries cannot know
    // which one the tool used; neither could the tool justify its choice.
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for e in &lock.packages {
        if !seen.insert(e.name.as_str()) {
            diags.push(Diagnostic::error(
                "DL1011",
                format!(
                    "delulu.lock has more than one entry for package `{}` — a duplicated entry is an \
                     ambiguity about what was pinned, and it is refused rather than resolved",
                    e.name
                ),
            ));
        }
    }
    if !diags.is_empty() {
        return diags;
    }

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
            continue;
        }
        // Then the RECORDED FIELDS, which are what a human reads.
        //
        // The two checks above compare hashes computed from reality against hashes stored in the
        // file. Neither one looks at `effects`, `cap_kinds`, `secrets` or the scope lists — so those
        // could be edited to say anything and `build --locked` still reported "built clean". A
        // lockfile could claim a dependency has no effects and no capabilities while that dependency
        // reaches the network, which is precisely the question a reviewer opens a lockfile to answer
        // (`HARDENING_CAMPAIGN.md` C52).
        //
        // What made it plain: `authority --diff` on the very same forged pair reports
        // `+ effects Net` and `verdict: WIDENING`. The interactive review command caught what the
        // automated CI gate did not, which is backwards — CI is where nobody is looking.
        let expected = compute_entry(ws, program, idx);
        // Destructured on purpose: a field added to `LockEntry` later will not compile until someone
        // decides whether it belongs in this comparison. The campaign has found four hand-maintained
        // authority lists that fell behind the type defining them (C31/C34/C35/C44); this is the
        // shape that cannot.
        let LockEntry {
            name: _,
            version,
            source: _,
            content_hash: _,
            authority_hash: _,
            api_row_hash: _,
            effects,
            cap_kinds,
            secrets,
            net,
            fs_read,
            fs_write,
            // An operator's recorded decision about a widening, not a fact about the package, so it
            // is not re-derivable from source and cannot be compared against it.
            accepted_by: _,
        } = &expected;
        // Sorted before comparison: a differently-ordered list means the same authority, and this
        // check is about what the entry CLAIMS, not about canonical formatting.
        let sorted = |v: &Vec<String>| {
            let mut c = v.clone();
            c.sort();
            c
        };
        // The version is re-derivable (it is the package's own manifest version), so a lock entry
        // naming a different one is stale or forged. Under `--locked` — whose contract is "refuse
        // any resolution not already pinned" — that must not build.
        if &entry.version != version {
            diags.push(Diagnostic::error(
                "DL1002",
                format!(
                    "package `{}`'s lock entry records version `{}` but the package is version `{}` \
                     — re-run `delulu lock`",
                    pkg.name, entry.version, version
                ),
            ));
        }
        for (field, recorded, actual) in [
            ("effects", &entry.effects, effects),
            ("cap_kinds", &entry.cap_kinds, cap_kinds),
            ("secrets", &entry.secrets, secrets),
            ("net", &entry.net, net),
            ("fs.read", &entry.fs_read, fs_read),
            ("fs.write", &entry.fs_write, fs_write),
        ] {
            if sorted(recorded) != sorted(actual) {
                diags.push(Diagnostic::error(
                    "DL1002",
                    format!(
                        "package `{}`'s lock entry records `{field} = {:?}` but its verified \
                         authority is {:?} — the lockfile misrepresents what this dependency can do",
                        pkg.name,
                        sorted(recorded),
                        sorted(actual)
                    ),
                ));
            }
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
        // A new secret name is a widening even though it adds no effect and no capability kind.
        // Omitting this was the hole C18 closed: without it a dependency could begin reading a
        // new secret on a patch bump and the semver-authority law would wave it through.
        || !subset(&new.secrets, &old.secrets)
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
                secrets: vec!["API_KEY".into()],
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
    fn a_new_secret_is_a_widening_and_needs_a_major_bump() {
        // HARDENING_CAMPAIGN C18. Reading a new secret adds no effect and no capability kind, so
        // before secrets entered the lock entry this pair had IDENTICAL observable authority and
        // the semver-authority law waved the patch bump through. Now it is DL1003 like any other
        // widening. The effects and cap_kinds are equal on purpose — the secret set is the only
        // thing that moves, which is exactly the case that used to be invisible.
        let old = LockEntry {
            name: "logger".into(),
            version: "1.0.0".into(),
            effects: vec!["Net".into()],
            secrets: vec!["TELEMETRY_TOKEN".into()],
            ..Default::default()
        };
        let new = LockEntry {
            name: "logger".into(),
            version: "1.0.1".into(), // a PATCH bump
            effects: vec!["Net".into()],
            secrets: vec!["TELEMETRY_TOKEN".into(), "DB_PASSWORD".into()],
            ..Default::default()
        };
        assert!(authority_widened(&old, &new), "a new secret name must count as a widening");
        let oldlock = Lockfile { version: 1, packages: vec![old] };
        let mut newlock = Lockfile { version: 1, packages: vec![new] };
        let diags = enforce_semver_law(&oldlock, &mut newlock, &[]);
        assert_eq!(diags.len(), 1, "the patch-bump secret widening must be refused: {diags:?}");
        assert_eq!(diags[0].code, "DL1003");
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
