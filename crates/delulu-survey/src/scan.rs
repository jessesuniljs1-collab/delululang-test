//! Walking the tree and deciding what each file *is*.
//!
//! Classification is by location and extension only — never by content sniffing. A reader can
//! confirm any classification by looking at the path, which is the property that makes the rest of
//! the map auditable.

use crate::{NodeKind, EXCLUDED_DIRS, EXCLUDED_FILE_EXTENSIONS};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// A Cargo manifest.
    Manifest,
    /// Rust under a crate's `src/`.
    RustSource,
    /// Rust under a crate's `tests/`, `benches/`, or `examples/`.
    RustTest,
    Doc,
    /// A DeluluLang program.
    Program,
    /// A conformance expectation or other JSON fixture.
    Json,
    Shell,
    /// Text the Survey records but does not parse.
    Other,
    /// Not read: bytes, not text.
    Binary,
}

#[derive(Debug, Clone)]
pub struct ScannedFile {
    /// Repository-relative, forward-slashed on every platform. Two runs on Windows and Linux
    /// produce the same ids, so the committed Survey is not OS-dependent.
    pub rel: String,
    pub abs: PathBuf,
    pub kind: FileKind,
    pub lines: u32,
    /// The workspace member that owns this file, if it is under `crates/<name>/`.
    pub crate_name: Option<String>,
    /// File text. Empty for [`FileKind::Binary`].
    pub text: String,
}

impl ScannedFile {
    /// The node this file contributes.
    ///
    /// A manifest gets a plain file node even though the crate it describes has one of its own:
    /// the crate node carries the manifest's *facts*, but documents cite the manifest by path, and
    /// an edge to a path with no node behind it is a dangling edge.
    pub fn node_identity(&self) -> Option<(String, NodeKind)> {
        let k = match self.kind {
            FileKind::RustSource => NodeKind::RustModule,
            FileKind::RustTest => NodeKind::TestSuite,
            FileKind::Doc => NodeKind::Doc,
            FileKind::Program => NodeKind::Program,
            _ => NodeKind::Other,
        };
        Some((format!("{}:{}", id_prefix(k), self.rel), k))
    }
}

pub fn id_prefix(k: NodeKind) -> &'static str {
    match k {
        NodeKind::Crate => "crate",
        NodeKind::ExternalCrate => "ext",
        NodeKind::RustModule => "mod",
        NodeKind::TestSuite => "test",
        NodeKind::Doc => "doc",
        NodeKind::Program => "prog",
        NodeKind::DiagnosticCode => "code",
        NodeKind::Ruling => "ruling",
        NodeKind::Finding => "finding",
        NodeKind::CliVerb => "cli",
        NodeKind::Measurement => "measure",
        NodeKind::Rfc => "rfc",
        NodeKind::Directory => "dir",
        NodeKind::Other => "file",
    }
}

/// Walk `root`, skipping [`EXCLUDED_DIRS`] and [`EXCLUDED_FILE_EXTENSIONS`]. Entries are sorted at
/// every level, so the resulting order is the same on any filesystem.
pub fn walk(root: &Path) -> Vec<ScannedFile> {
    let mut out = Vec::new();
    descend(root, root, &mut out);
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    out
}

fn descend(root: &Path, dir: &Path, out: &mut Vec<ScannedFile>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    entries.sort();

    for path in entries {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        if path.is_dir() {
            if EXCLUDED_DIRS.iter().any(|(d, _)| *d == name) {
                continue;
            }
            descend(root, &path, out);
        } else if EXCLUDED_FILE_EXTENSIONS
            .iter()
            .any(|(e, _)| name.rsplit_once('.').is_some_and(|(_, x)| x == *e))
        {
            continue;
        } else if let Some(f) = classify(root, &path, &name) {
            // The Survey's own generated output is never read back. See `OUTPUT_FILES`.
            if crate::OUTPUT_FILES.contains(&f.rel.as_str()) {
                continue;
            }
            out.push(f);
        }
    }
}

fn classify(root: &Path, path: &Path, name: &str) -> Option<ScannedFile> {
    let rel = path.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");

    let ext = name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
    let kind = if name == "Cargo.toml" {
        FileKind::Manifest
    } else {
        match ext {
            "rs" => {
                // `src/` is the crate's own code; `tests/`, `benches/` and `examples/` are Cargo's
                // separate compilation targets, which is a real distinction and not a naming one.
                if rel.contains("/src/") {
                    FileKind::RustSource
                } else {
                    FileKind::RustTest
                }
            }
            "md" => FileKind::Doc,
            "delulu" => FileKind::Program,
            "json" => FileKind::Json,
            "sh" => FileKind::Shell,
            "dwx" | "sig" | "wasm" => FileKind::Binary,
            _ => FileKind::Other,
        }
    };

    let text = if kind == FileKind::Binary {
        String::new()
    } else {
        // A file that is not valid UTF-8 is recorded but not parsed, rather than crashing the map.
        std::fs::read_to_string(path).unwrap_or_default()
    };
    let lines = text.lines().count() as u32;

    let crate_name = rel
        .strip_prefix("crates/")
        .and_then(|r| r.split_once('/'))
        .map(|(c, _)| c.to_string());

    Some(ScannedFile { rel, abs: path.to_path_buf(), kind, lines, crate_name, text })
}

/// Strip Rust comments so that a `use` inside a comment is not read as a dependency, returning one
/// entry per original line so line numbers survive. Comment text is handed back separately,
/// because comments are where this repository puts its citations.
pub fn split_code_and_comments(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut in_block = false;

    for line in text.lines() {
        let (mut code, mut comment) = (String::new(), String::new());
        let b: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut in_str = false;
        while i < b.len() {
            let c = b[i];
            let next = b.get(i + 1).copied().unwrap_or('\0');
            if in_block {
                if c == '*' && next == '/' {
                    in_block = false;
                    i += 2;
                    continue;
                }
                comment.push(c);
                i += 1;
            } else if in_str {
                // Skip escapes so a `\"` does not end the literal early.
                if c == '\\' {
                    code.push(c);
                    if i + 1 < b.len() {
                        code.push(next);
                    }
                    i += 2;
                    continue;
                }
                if c == '"' {
                    in_str = false;
                }
                code.push(c);
                i += 1;
            } else if c == '/' && next == '/' {
                comment.push_str(&b[i..].iter().collect::<String>());
                break;
            } else if c == '/' && next == '*' {
                in_block = true;
                i += 2;
            } else {
                if c == '"' {
                    in_str = true;
                }
                code.push(c);
                i += 1;
            }
        }
        out.push((code, comment));
    }
    out
}
