//! The canonical compile-time package manifest (`delulu.toml`), Stage 2 §3.2.
//!
//! This is the *checker's* view of a manifest — richer than the minimal runtime parser in
//! `delulu-runtime` (which serves load-time grants and is intentionally left untouched). It models
//! `[package]`, `[authority]`, `[dependencies]`, and per-dependency authority pins, and it is a
//! **pure** function of source text (no code runs at resolution time — spec §3.3). Malformed input
//! becomes diagnostics (DL1004), never a panic.

use delulu_diag::{Diagnostic, FileId, Span};

/// A package is an executable (`bin`, has `fn main`), a library (`lib`), or — Stage 6 — a
/// runtime-loadable plugin (`plugin`, must expose **no** `fn main`; spec §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageKind {
    Bin,
    Lib,
    Plugin,
}

impl PackageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PackageKind::Bin => "bin",
            PackageKind::Lib => "lib",
            PackageKind::Plugin => "plugin",
        }
    }
}

/// A declared authority ceiling: effects plus per-kind scope lists. Used both for a package's own
/// `[authority]` and for a dependency's authority *pin* (`[dependencies.<name>.authority]`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthoritySpec {
    pub effects: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
}

/// Where a dependency's source lives. `git` fetching is out of scope this round (see §11); a git
/// source is parsed and validated (rev/tag presence, DL1007) but resolution is deferred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DepSource {
    Path(String),
    Git { url: String, rev: Option<String>, tag: Option<String> },
}

/// One `[dependencies]` entry: a name, a source, and (per §3.2) an authority pin — the consumer's
/// declared ceiling for this dependency. A missing pin is itself DL1001, enforced at resolution.
#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub source: DepSource,
    pub pin: Option<AuthoritySpec>,
}

/// A parsed `delulu.toml`.
#[derive(Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub kind: PackageKind,
    pub authority: AuthoritySpec,
    /// Dependencies, sorted by name for deterministic resolution and hashing.
    pub dependencies: Vec<Dependency>,
    /// Original manifest bytes, retained for `content_hash` (lockfile §4.4).
    pub raw: String,
    /// The file id in the caller's source map (for diagnostic spans).
    pub file: FileId,
}

impl Manifest {
    /// Parse a manifest. Structural/semantic problems become DL1004 diagnostics with a coarse span.
    /// Returns `Some(manifest)` when the required `[package]` fields are present, together with any
    /// diagnostics (which may be non-empty even on success, e.g. an unknown `kind`).
    pub fn parse(src: &str, file: FileId) -> (Option<Manifest>, Vec<Diagnostic>) {
        let mut diags = Vec::new();
        let table: toml::Table = match src.parse() {
            Ok(t) => t,
            Err(e) => {
                let span = e.span().map(|r| Span::new(file, r.start as u32, r.end as u32)).unwrap_or(Span::new(file, 0, 0));
                diags.push(
                    Diagnostic::error("DL1004", format!("could not parse `delulu.toml`: {}", e.message()))
                        .with_span(span, "TOML syntax error here"),
                );
                return (None, diags);
            }
        };

        let pkg = table.get("package").and_then(|v| v.as_table());
        let name = pkg.and_then(|p| p.get("name")).and_then(|v| v.as_str()).map(str::to_string);
        let version = pkg.and_then(|p| p.get("version")).and_then(|v| v.as_str()).map(str::to_string);
        let kind_str = pkg.and_then(|p| p.get("kind")).and_then(|v| v.as_str());

        let (Some(name), Some(version)) = (name, version) else {
            diags.push(
                Diagnostic::error("DL1004", "`delulu.toml` is missing required `[package] name`/`version`")
                    .with_span(section_span(src, file, "package"), "expected `[package]` with `name` and `version`"),
            );
            return (None, diags);
        };

        // `kind` is honored when present; a missing kind defaults to `bin` (back-compat with
        // Stage-1 manifests that predate the field) rather than failing the build.
        let kind = match kind_str {
            Some("lib") => PackageKind::Lib,
            Some("plugin") => PackageKind::Plugin,
            Some("bin") | None => PackageKind::Bin,
            Some(other) => {
                diags.push(
                    Diagnostic::error("DL1004", format!("unknown package kind `{other}` (expected `bin`, `lib`, or `plugin`)"))
                        .with_span(key_span(src, file, "kind"), "here"),
                );
                PackageKind::Bin
            }
        };

        let authority = parse_authority(table.get("authority").and_then(|v| v.as_table()));

        let mut dependencies = Vec::new();
        if let Some(deps) = table.get("dependencies").and_then(|v| v.as_table()) {
            for (dep_name, val) in deps {
                match parse_dependency(dep_name, val) {
                    Ok(dep) => dependencies.push(dep),
                    Err(msg) => diags.push(
                        Diagnostic::error("DL1004", msg)
                            .with_span(key_span(src, file, dep_name), "in this dependency entry"),
                    ),
                }
            }
        }
        dependencies.sort_by(|a, b| a.name.cmp(&b.name));

        let manifest = Manifest { name, version, kind, authority, dependencies, raw: src.to_string(), file };
        (Some(manifest), diags)
    }

    /// Byte span of the `effects` line under `[authority]` (for DL1009), or the file start.
    pub fn effects_span(&self) -> Span {
        key_span(&self.raw, self.file, "effects")
    }

    /// Byte span of a dependency's `[dependencies.<name>` header, or `[dependencies]`.
    pub fn dep_span(&self, dep: &str) -> Span {
        let needle = format!("dependencies.{dep}");
        if let Some(off) = self.raw.find(&needle) {
            return line_span_at(&self.raw, self.file, off);
        }
        if let Some(off) = self.raw.find(dep) {
            return line_span_at(&self.raw, self.file, off);
        }
        section_span(&self.raw, self.file, "dependencies")
    }
}

fn parse_authority(table: Option<&toml::Table>) -> AuthoritySpec {
    let mut a = AuthoritySpec::default();
    let Some(t) = table else { return a };
    a.effects = arr_of_str(t.get("effects"));
    a.net = arr_of_str(t.get("net"));
    a.secrets = arr_of_str(t.get("secrets"));
    // `fs.read` / `fs.write` arrive as a nested `fs` table (dotted keys) in TOML.
    if let Some(fs) = t.get("fs").and_then(|v| v.as_table()) {
        a.fs_read = arr_of_str(fs.get("read"));
        a.fs_write = arr_of_str(fs.get("write"));
    }
    a
}

fn parse_dependency(name: &str, val: &toml::Value) -> Result<Dependency, String> {
    let t = val.as_table().ok_or_else(|| format!("dependency `{name}` must be a table (e.g. `{{ path = \"...\" }}`)"))?;
    let source = if let Some(p) = t.get("path").and_then(|v| v.as_str()) {
        DepSource::Path(p.to_string())
    } else if let Some(url) = t.get("git").and_then(|v| v.as_str()) {
        DepSource::Git {
            url: url.to_string(),
            rev: t.get("rev").and_then(|v| v.as_str()).map(str::to_string),
            tag: t.get("tag").and_then(|v| v.as_str()).map(str::to_string),
        }
    } else {
        return Err(format!("dependency `{name}` needs a `path` or `git` source"));
    };
    let pin = t.get("authority").and_then(|v| v.as_table()).map(|a| parse_authority(Some(a)));
    Ok(Dependency { name: name.to_string(), source, pin })
}

fn arr_of_str(v: Option<&toml::Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

// ----- coarse span helpers (manifest diagnostics are line-granular by design) --------------------

fn line_span_at(src: &str, file: FileId, off: usize) -> Span {
    let start = src[..off].rfind('\n').map(|n| n + 1).unwrap_or(0);
    let end = src[off..].find('\n').map(|n| off + n).unwrap_or(src.len());
    Span::new(file, start as u32, end as u32)
}

fn key_span(src: &str, file: FileId, key: &str) -> Span {
    match src.find(key) {
        Some(off) => line_span_at(src, file, off),
        None => Span::new(file, 0, 0),
    }
}

fn section_span(src: &str, file: FileId, section: &str) -> Span {
    let header = format!("[{section}]");
    match src.find(&header) {
        Some(off) => line_span_at(src, file, off),
        None => Span::new(file, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_manifest() {
        let src = r#"
[package]
name = "mypkg"
version = "0.2.1"
kind = "bin"

[authority]
effects = ["Read", "Write"]
fs.read = ["./config"]
net = []

[dependencies]
mathkit = { path = "../mathkit", authority = { effects = [] } }

[dependencies.webby]
git = "https://github.com/x/webby"
rev = "9f2c41d"

[dependencies.webby.authority]
effects = ["Net"]
net = ["api.example.com"]
"#;
        let (m, diags) = Manifest::parse(src, 0);
        assert!(diags.is_empty(), "{diags:?}");
        let m = m.unwrap();
        assert_eq!(m.name, "mypkg");
        assert_eq!(m.version, "0.2.1");
        assert_eq!(m.kind, PackageKind::Bin);
        assert_eq!(m.authority.effects, vec!["Read", "Write"]);
        assert_eq!(m.authority.fs_read, vec!["./config"]);
        assert_eq!(m.dependencies.len(), 2);
        // sorted by name: mathkit, webby
        assert_eq!(m.dependencies[0].name, "mathkit");
        assert!(matches!(m.dependencies[0].source, DepSource::Path(_)));
        assert_eq!(m.dependencies[0].pin.as_ref().unwrap().effects, Vec::<String>::new());
        assert_eq!(m.dependencies[1].name, "webby");
        match &m.dependencies[1].source {
            DepSource::Git { rev, .. } => assert_eq!(rev.as_deref(), Some("9f2c41d")),
            _ => panic!("expected git source"),
        }
        assert_eq!(m.dependencies[1].pin.as_ref().unwrap().net, vec!["api.example.com"]);
    }

    #[test]
    fn missing_pin_parses_as_none() {
        let src = "[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"lib\"\n[dependencies]\nfoo = { path = \"../foo\" }\n";
        let (m, diags) = Manifest::parse(src, 0);
        assert!(diags.is_empty(), "{diags:?}");
        let m = m.unwrap();
        assert!(m.dependencies[0].pin.is_none());
    }

    #[test]
    fn malformed_toml_is_dl1004_not_panic() {
        let (m, diags) = Manifest::parse("this is not = = toml", 0);
        assert!(m.is_none());
        assert_eq!(diags[0].code, "DL1004");
    }

    #[test]
    fn missing_package_fields_is_dl1004() {
        let (m, diags) = Manifest::parse("[authority]\neffects = []\n", 0);
        assert!(m.is_none());
        assert!(diags.iter().any(|d| d.code == "DL1004"));
    }
}
