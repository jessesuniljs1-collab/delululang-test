//! The Stage-1 capability broker (spec §7.2). In Stage 1 the CLI *is* the human-controlled
//! broker: root capabilities are constructed here from grants the human passes, and `main`
//! receives only that slice. Stage 5 replaces this with a process-separated broker; the grant
//! model and manifest format do not change.

use std::collections::HashMap;

use delulu_check::{Effect, ResourceKind, ScopeInfo};
use delulu_diag::Diagnostic;

use crate::prim::granted_root;
use crate::value::RootVal;

/// A parsed authority manifest (`delulu.toml`, `[authority]` section). Deliberately minimal for
/// Stage 1 — enough to run the DL0701 check and drive the grant reconciliation.
#[derive(Default, Debug)]
pub struct Manifest {
    pub name: Option<String>,
    pub effects: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
}

/// Parse the tiny subset of TOML the Stage-1 manifest uses: `[section]` headers and
/// `key = "s"` / `key = ["a", "b"]` lines. Dotted keys (`fs.read`) are supported literally.
pub fn parse_manifest(src: &str) -> Manifest {
    let mut m = Manifest::default();
    let mut section = String::new();
    for raw in src.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = inner.trim().to_string();
            continue;
        }
        let Some((key, val)) = line.split_once('=') else { continue };
        let key = key.trim();
        let val = val.trim();
        let values = parse_toml_strings(val);
        match (section.as_str(), key) {
            ("package", "name") => m.name = values.into_iter().next(),
            ("authority", "effects") => m.effects = values,
            ("authority", "fs.read") => m.fs_read = values,
            ("authority", "fs.write") => m.fs_write = values,
            ("authority", "net") => m.net = values,
            ("authority", "secrets") => m.secrets = values,
            _ => {}
        }
    }
    m
}

fn parse_toml_strings(val: &str) -> Vec<String> {
    let body = val.trim();
    let inner = body.strip_prefix('[').and_then(|s| s.strip_suffix(']')).unwrap_or(body);
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

impl Manifest {
    /// A `ScopeInfo` for the authority report, from the manifest's declared scopes.
    pub fn scope_info(&self) -> ScopeInfo {
        ScopeInfo {
            fs_read: self.fs_read.clone(),
            fs_write: self.fs_write.clone(),
            net: self.net.clone(),
        }
    }

    /// DL0701: `main`'s effect row must be within the manifest's declared effects.
    pub fn check_main_row(&self, main_row: &std::collections::BTreeSet<Effect>) -> Vec<Diagnostic> {
        let declared: std::collections::HashSet<&str> = self.effects.iter().map(|s| s.as_str()).collect();
        let mut diags = Vec::new();
        for e in main_row {
            if !declared.contains(e.name()) {
                diags.push(Diagnostic::error(
                    "DL0701",
                    format!("`main` performs effect `{}` not permitted by the authority manifest", e.name()),
                ));
            }
        }
        diags
    }
}

/// The set of grants collected from `--grant` flags (and/or an accepted manifest).
#[derive(Default)]
pub struct Grants {
    pub console: bool,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub clock: bool,
    pub rand: bool,
    pub declassify: bool,
    pub secrets: HashMap<String, String>,
}

impl Grants {
    /// Parse one `--grant` argument. Returns an error string on malformed input.
    pub fn add(&mut self, spec: &str) -> Result<(), String> {
        if let Some(rest) = spec.strip_prefix("secret:") {
            let (name, source) = rest.split_once('=').ok_or_else(|| format!("bad secret grant `{spec}` (use secret:NAME=VALUE)"))?;
            let value = if let Some(var) = source.strip_prefix("env:") {
                std::env::var(var).map_err(|_| format!("env var `{var}` for secret `{name}` is not set"))?
            } else {
                source.to_string()
            };
            self.secrets.insert(name.to_string(), value);
            return Ok(());
        }
        match spec.split_once('=') {
            Some((k, v)) => match k.trim() {
                "fs.read" => self.fs_read.push(v.trim().to_string()),
                "fs.write" => self.fs_write.push(v.trim().to_string()),
                "net" => self.net.push(v.trim().to_string()),
                other => return Err(format!("unknown grant `{other}`")),
            },
            None => match spec.trim() {
                "console" => self.console = true,
                "clock" => self.clock = true,
                "rand" => self.rand = true,
                "declassify" => self.declassify = true,
                other => return Err(format!("unknown grant `{other}`")),
            },
        }
        Ok(())
    }

    /// Accept an entire manifest's declared authority as grants (`--grant-manifest`). Secrets
    /// are pulled from environment variables of the same name when available.
    pub fn accept_manifest(&mut self, m: &Manifest) {
        self.console = self.console || m.effects.iter().any(|e| e == "Write" || e == "Read");
        self.fs_read.extend(m.fs_read.iter().cloned());
        self.fs_write.extend(m.fs_write.iter().cloned());
        self.net.extend(m.net.iter().cloned());
        for name in &m.secrets {
            if let Ok(v) = std::env::var(name) {
                self.secrets.insert(name.clone(), v);
            }
        }
    }

    /// Build the root capability slice `main` receives.
    pub fn build_root(&self) -> RootVal {
        RootVal {
            console: self.console,
            fs_read: self.fs_read.iter().map(|p| granted_root(p)).collect(),
            fs_write: self.fs_write.iter().map(|p| granted_root(p)).collect(),
            net: self.net.clone(),
            clock: self.clock,
            rand: self.rand,
            declassify: self.declassify,
            secrets: self.secrets.clone(),
        }
    }

    pub fn scope_info(&self) -> ScopeInfo {
        ScopeInfo { fs_read: self.fs_read.clone(), fs_write: self.fs_write.clone(), net: self.net.clone() }
    }
}

/// Which capability kinds a program needs but the grants do not cover (for the prompt / DL0702).
pub fn missing_kinds(needs: &std::collections::BTreeSet<ResourceKind>, grants: &Grants) -> Vec<ResourceKind> {
    needs
        .iter()
        .filter(|k| match k {
            ResourceKind::Console => !grants.console,
            ResourceKind::FsRead => grants.fs_read.is_empty(),
            ResourceKind::FsWrite => grants.fs_write.is_empty(),
            ResourceKind::Http => grants.net.is_empty(),
            ResourceKind::Clock => !grants.clock,
            ResourceKind::Rand => !grants.rand,
            ResourceKind::Declassify => !grants.declassify,
            ResourceKind::PluginHost => true,
        })
        .copied()
        .collect()
}
