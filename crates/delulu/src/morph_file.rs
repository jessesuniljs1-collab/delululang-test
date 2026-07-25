//! Loading surface morphs from disk (Stage 8 §6.5; `docs/design/SYNTAX_MORPH_SPEC.md`).
//!
//! The morph *format* lives here rather than in `delulu-syntax` on purpose: the syntax crate owns
//! the canonical-form law and the token rewriting, and knows nothing about files, TOML, or search
//! paths. That split is what lets the law be tested without a filesystem, and it keeps the lexer
//! crate free of a config-format dependency.
//!
//! Search order for a morph id, first hit wins:
//!
//! 1. `<cwd>/morphs/<id>.toml` — a project's own morphs, versioned beside the code that uses them.
//! 2. `$DELULU_MORPH_PATH/<id>.toml` — an explicit override, for testing and for installs that keep
//!    morphs outside the project.
//! 3. `~/.delulu/morphs/<id>.toml` — a user's installed morphs.
//!
//! Nothing is embedded in the binary. Canonical DeluluLang is the only surface the toolchain ships,
//! which keeps "no party's preferred surface is privileged" true in the build as well as the prose:
//! a Chinese keyword morph and an AI's compact profile are equally third-party.

use std::path::{Path, PathBuf};

use delulu_diag::Diagnostic;
use delulu_syntax::morph::{Morph, MorphKind};

/// Everything that can go wrong loading a morph, as diagnostics ready to print.
#[derive(Debug)]
pub struct LoadError(pub Vec<Diagnostic>);

impl LoadError {
    fn one(code: &'static str, msg: impl Into<String>) -> LoadError {
        LoadError(vec![Diagnostic::error(code, msg.into())])
    }
}

/// Parse a morph from TOML text. `id_hint` is used in messages when `[meta] morph` is absent.
pub fn parse_morph(text: &str, id_hint: &str) -> Result<Morph, LoadError> {
    let value: toml::Value = toml::from_str(text)
        .map_err(|e| LoadError::one("DL1714", format!("morph `{id_hint}` is not valid TOML: {e}")))?;

    let meta = value.get("meta");
    let get_meta = |key: &str| -> Option<String> {
        meta.and_then(|m| m.get(key)).and_then(|v| v.as_str()).map(str::to_string)
    };
    let id = get_meta("morph").unwrap_or_else(|| id_hint.to_string());
    let name = get_meta("name").unwrap_or_else(|| id.clone());
    let version = get_meta("version").unwrap_or_else(|| "0.0.0".to_string());
    let kind_str = get_meta("kind").unwrap_or_else(|| "custom".to_string());
    let Some(kind) = MorphKind::parse(&kind_str) else {
        return Err(LoadError::one(
            "DL1714",
            format!("morph `{id}`: `[meta] kind` is `{kind_str}` — expected `human`, `compact`, or `custom`"),
        ));
    };

    // `[keywords]` and `[operators]` are collected into one entry list: the validator does not care
    // which table an alias came from, and neither does the lexer. Operators are accepted here so a
    // morph file written to the spec's shape loads, and any operator alias that is not a renameable
    // keyword is refused by name with DL1713 — an honest refusal rather than a silent ignore.
    let mut entries: Vec<(String, String)> = Vec::new();
    for table in ["keywords", "operators"] {
        let Some(t) = value.get(table).and_then(|v| v.as_table()) else { continue };
        for (canon, alias) in t {
            let Some(alias) = alias.as_str() else {
                return Err(LoadError::one(
                    "DL1712",
                    format!("morph `{id}`: the alias for `{canon}` must be a string"),
                ));
            };
            entries.push((canon.clone(), alias.to_string()));
        }
    }
    if entries.is_empty() {
        return Err(LoadError::one(
            "DL1714",
            format!("morph `{id}` renames nothing — a morph with no `[keywords]` entries is not a surface"),
        ));
    }

    Morph::new(id, name, version, kind, entries)
        .map_err(|errs| LoadError(errs.iter().map(|e| e.to_diagnostic()).collect()))
}

/// The directories searched for `<id>.toml`, in order.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("morphs")];
    if let Ok(p) = std::env::var("DELULU_MORPH_PATH") {
        if !p.is_empty() {
            dirs.push(PathBuf::from(p));
        }
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        dirs.push(PathBuf::from(home).join(".delulu").join("morphs"));
    }
    dirs
}

/// Find and load the morph with this id.
pub fn load(id: &str) -> Result<Morph, LoadError> {
    // A morph id becomes a filename, so it must not be able to walk out of the search directories.
    // `--morph ../../etc/passwd` is not a supply-chain attack (the file would fail to parse), but a
    // path traversal that reads outside the search path is not a thing a language toolchain should
    // do on request, and refusing is one line.
    if id.is_empty() || id.contains(['/', '\\', ':']) || id.contains("..") {
        return Err(LoadError::one(
            "DL1714",
            format!("`{id}` is not a valid morph id — ids are plain names, not paths"),
        ));
    }
    for dir in search_dirs() {
        let path = dir.join(format!("{id}.toml"));
        if path.is_file() {
            let text = std::fs::read_to_string(&path).map_err(|e| {
                LoadError::one("DL1714", format!("cannot read morph `{}`: {e}", path.display()))
            })?;
            return parse_morph(&text, id);
        }
    }
    Err(LoadError::one(
        "DL1714",
        format!(
            "morph `{id}` is not installed — looked in {}",
            search_dirs().iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
        ),
    ))
}

/// Load a morph file by explicit path (for `delulu morph check <file.toml>`).
pub fn load_path(path: &Path) -> Result<Morph, LoadError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| LoadError::one("DL1714", format!("cannot read `{}`: {e}", path.display())))?;
    let hint = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    parse_morph(&text, &hint)
}

/// Every morph id found in the search path, deduplicated, in search order.
pub fn installed() -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for dir in search_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut found: Vec<(String, PathBuf)> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
            .filter_map(|p| {
                p.file_stem().map(|s| (s.to_string_lossy().to_string(), p.clone()))
            })
            .collect();
        found.sort();
        for (id, path) in found {
            if !out.iter().any(|(seen, _)| *seen == id) {
                out.push((id, path));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(e: &LoadError) -> Vec<String> {
        e.0.iter().map(|d| d.code.to_string()).collect()
    }

    const COMPACT: &str = r#"
[meta]
morph = "compact-ai"
name = "Compact (AI profile)"
version = "1.0.0"
kind = "compact"

[keywords]
fn = "f"
let = "l"
return = "r"
"#;

    #[test]
    fn a_well_formed_morph_file_loads() {
        let m = parse_morph(COMPACT, "compact-ai").expect("must load");
        assert_eq!(m.id, "compact-ai");
        assert_eq!(m.kind, MorphKind::Compact);
        assert_eq!(m.alias_of("fn"), Some("f"));
    }

    #[test]
    fn a_morph_that_renames_nothing_is_refused() {
        let e = parse_morph("[meta]\nmorph=\"x\"\nkind=\"custom\"\n", "x").expect_err("refused");
        assert_eq!(codes(&e), vec!["DL1714"]);
    }

    #[test]
    fn an_unknown_kind_is_refused() {
        let src = "[meta]\nmorph=\"x\"\nkind=\"sneaky\"\n[keywords]\nfn=\"f\"\n";
        let e = parse_morph(src, "x").expect_err("refused");
        assert_eq!(codes(&e), vec!["DL1714"]);
    }

    #[test]
    fn validation_errors_come_through_from_the_syntax_crate() {
        // The loader must not soften the canonical-form law: this file is well-formed TOML and is
        // refused for what it MEANS (the alias `fn` for `let` would mislead every reader).
        let src = "[meta]\nmorph=\"x\"\nkind=\"custom\"\n[keywords]\nlet=\"fn\"\n";
        let e = parse_morph(src, "x").expect_err("refused");
        assert_eq!(codes(&e), vec!["DL1711"]);
    }

    #[test]
    fn a_morph_id_may_not_be_a_path() {
        for bad in ["../secrets", "a/b", "..", "c:\\x"] {
            let e = load(bad).expect_err("must refuse a path-shaped id");
            assert_eq!(codes(&e), vec!["DL1714"], "id {bad:?}");
        }
    }

    #[test]
    fn a_missing_morph_names_where_it_looked() {
        let e = load("definitely-not-installed-xyz").expect_err("must refuse");
        assert_eq!(codes(&e), vec!["DL1714"]);
        assert!(e.0[0].message.contains("looked in"), "{}", e.0[0].message);
    }
}
