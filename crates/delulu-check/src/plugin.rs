//! `kind = "plugin"` packages: the plugin manifest tables and the manifest-vs-code fence
//! (Stage 6 "Live", spec §2.1).
//!
//! A plugin package's `delulu.toml` carries `[plugin]` (the ABI version and class),
//! `[plugin.authority]` (the **hard ceiling** fixed in Stage-1 §8.1), and `[plugin.exports]`
//! (export signature strings). Two rules are enforced here:
//!
//! - **The manifest never overrides the code.** Every export signature string is parsed with the
//!   *ordinary type grammar* (`delulu_syntax::parse_type_string`), lowered with the checker's own
//!   `lower_type` rule code, and compared for **exact equality** against the checked function type.
//!   Any disagreement — in either direction; a manifest that undersells the code's row is still a
//!   mismatch — is **DL1501**, with the regenerated (code-derived) signature as the exact repair.
//! - **A plugin exposes no `fn main`** (spec §2.1) — a plugin is loaded, never run. Violation is
//!   DL1501 (the manifest's `kind = "plugin"` claim does not match the code).

use std::collections::BTreeMap;

use delulu_diag::{Confidence, Diagnostic, Edit, FileId, Repair, Span};
use delulu_syntax::ast::TypeExpr;
use delulu_syntax::parse_type_string;

use crate::check::{lower_export_signature, CheckResult};
use crate::resolve::DeclTable;
use crate::ty::{ResourceKind, Row, Type};

/// The plugin ABI version this toolchain builds and loads. `[plugin] api` values other than this
/// are refused with DL1507 (rebuild the plugin) — at build, inspect, verify, and load alike.
pub const PLUGIN_API_SUPPORTED: u32 = 1;

/// The declared plugin class (spec §2.1). **Never inferred** (invariant 29): the artifact declares
/// it, the loader verifies the declaration. Defined in [`crate::ty`] because it is also a *type*
/// (`Plugin[Verified]`); re-exported here so manifest code reads naturally.
pub use crate::ty::PluginClass;

/// The `[plugin.authority]` hard ceiling (Stage-1 §8.1): the most a *host* may ever grant this
/// plugin — `grant ⊑ authority` is load step 3 (DL1502). Effects plus required capability kinds,
/// plus optional per-kind scope lists (same shapes as the package `[authority]` table).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginAuthority {
    pub effects: Vec<String>,
    /// Required capability types, as written (`"Cap[FsRead]"`); validated to be well-formed.
    pub requires: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
    pub declassify: Vec<String>,
}

/// A parsed plugin manifest: the `[package]` identity plus the `[plugin]`/`[plugin.authority]`/
/// `[plugin.exports]` tables (spec §2.1). Exports are a `BTreeMap` so every downstream artifact
/// and report is deterministically ordered.
#[derive(Clone, Debug)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub api: u32,
    pub class: PluginClass,
    pub authority: PluginAuthority,
    /// Export name → signature string, exactly as written in the manifest.
    pub exports: BTreeMap<String, String>,
}

impl PluginManifest {
    /// Parse the plugin tables out of a `delulu.toml` whose `[package] kind` is `"plugin"`.
    /// Structural problems are DL1004 (like every other manifest fault); an unsupported
    /// `[plugin] api` is DL1507. Returns `None` when the `[plugin]` table is unusable.
    pub fn parse(src: &str, file: FileId) -> (Option<PluginManifest>, Vec<Diagnostic>) {
        let mut diags = Vec::new();
        let table: toml::Table = match src.parse() {
            Ok(t) => t,
            Err(e) => {
                diags.push(
                    Diagnostic::error("DL1004", format!("could not parse `delulu.toml`: {}", e.message()))
                        .with_span(Span::new(file, 0, 0), "TOML syntax error"),
                );
                return (None, diags);
            }
        };

        let pkg = table.get("package").and_then(|v| v.as_table());
        let name = pkg.and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let version = pkg.and_then(|p| p.get("version")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        let Some(plugin) = table.get("plugin").and_then(|v| v.as_table()) else {
            diags.push(
                Diagnostic::error("DL1004", "a `kind = \"plugin\"` package needs a `[plugin]` table (`api`, `class`)")
                    .with_span(section_span(src, file, "package"), "declared as a plugin here"),
            );
            return (None, diags);
        };

        // `api`: required, integer, and supported (DL1507 otherwise — spec §3.1 step 1).
        let api = match plugin.get("api").and_then(|v| v.as_integer()) {
            Some(a) if a >= 0 => a as u32,
            Some(_) | None => {
                diags.push(
                    Diagnostic::error("DL1004", "`[plugin] api` must be a non-negative integer")
                        .with_span(section_span(src, file, "plugin"), "in this table"),
                );
                return (None, diags);
            }
        };
        if api != PLUGIN_API_SUPPORTED {
            diags.push(
                Diagnostic::error(
                    "DL1507",
                    format!("plugin API version {api} is not supported by this toolchain (api = {PLUGIN_API_SUPPORTED}) — rebuild the plugin"),
                )
                .with_span(key_span(src, file, "api"), "declared here"),
            );
            return (None, diags);
        }

        // `class`: required, exactly "verified" | "contained" (invariant 29 — never inferred, so
        // never defaulted either).
        let class = match plugin.get("class").and_then(|v| v.as_str()).and_then(PluginClass::from_str) {
            Some(c) => c,
            None => {
                diags.push(
                    Diagnostic::error("DL1004", "`[plugin] class` must be `\"verified\"` or `\"contained\"`")
                        .with_span(key_span(src, file, "class"), "here"),
                );
                return (None, diags);
            }
        };

        // `[plugin.authority]` — the hard ceiling. Each `requires` entry must be a well-formed
        // `Cap[Kind]` with a known resource kind (hostile-manifest hardening: garbage is refused at
        // parse, not misinterpreted at load).
        let mut authority = PluginAuthority::default();
        if let Some(a) = plugin.get("authority").and_then(|v| v.as_table()) {
            authority.effects = arr_of_str(a.get("effects"));
            authority.requires = arr_of_str(a.get("requires"));
            authority.net = arr_of_str(a.get("net"));
            authority.secrets = arr_of_str(a.get("secrets"));
            authority.declassify = arr_of_str(a.get("declassify"));
            if let Some(fs) = a.get("fs").and_then(|v| v.as_table()) {
                authority.fs_read = arr_of_str(fs.get("read"));
                authority.fs_write = arr_of_str(fs.get("write"));
            }
            for req in &authority.requires {
                if parse_cap_kind(req).is_none() {
                    diags.push(
                        Diagnostic::error(
                            "DL1004",
                            format!("`[plugin.authority] requires` entry `{req}` is not a `Cap[Kind]` with a known resource kind"),
                        )
                        .with_span(key_span(src, file, "requires"), "here"),
                    );
                }
            }
        }

        // `[plugin.exports]`: name → signature string. May be empty; must be strings.
        let mut exports = BTreeMap::new();
        if let Some(e) = plugin.get("exports").and_then(|v| v.as_table()) {
            for (k, v) in e {
                match v.as_str() {
                    Some(s) => {
                        exports.insert(k.clone(), s.to_string());
                    }
                    None => diags.push(
                        Diagnostic::error("DL1004", format!("`[plugin.exports] {k}` must be a signature string"))
                            .with_span(key_span(src, file, k), "here"),
                    ),
                }
            }
        }

        if diags.iter().any(|d| d.is_error()) {
            return (None, diags);
        }
        (Some(PluginManifest { name, version, api, class, authority, exports }), diags)
    }
}

/// Parse a `requires` entry (`"Cap[FsRead]"`) to its resource kind, or `None` if malformed.
/// String-level: no declaration table is needed for the core `Cap` grammar.
pub fn parse_cap_kind(s: &str) -> Option<ResourceKind> {
    let (ty, diags) = parse_type_string(u32::MAX, s);
    if diags.iter().any(|d| d.is_error()) {
        return None;
    }
    match ty {
        TypeExpr::Named { path, args, .. } if path.segs.len() == 1 && path.segs[0].name == "Cap" && args.len() == 1 => {
            match &args[0] {
                TypeExpr::Named { path, args, .. } if path.segs.len() == 1 && args.is_empty() => {
                    ResourceKind::from_name(&path.segs[0].name)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// The manifest-vs-code fence (spec §2.1): `fn main` must be absent, every export must exist, be
/// monomorphic, and carry a signature string that lowers to **exactly** the checked function type.
/// Every violation is DL1501; a signature mismatch carries the regenerated code-derived signature
/// as an exact repair (spec §7: "regenerate manifest — exact").
pub fn check_plugin_module(
    pm: &PluginManifest,
    table: &DeclTable,
    result: &CheckResult,
    manifest_src: &str,
    manifest_file: FileId,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // A plugin is loaded, never run (spec §2.1): `fn main` is a code/manifest contradiction.
    if result.main_present {
        diags.push(
            Diagnostic::error(
                "DL1501",
                "a `kind = \"plugin\"` package must not expose `fn main` — a plugin is loaded by a host, never run",
            )
            .with_span(key_span(manifest_src, manifest_file, "kind"), "declared as a plugin here"),
        );
    }

    for (name, sig_str) in &pm.exports {
        let line = key_span(manifest_src, manifest_file, name);

        // The exported function must exist in the code.
        let Some(sig) = table.fns.get(name) else {
            diags.push(
                Diagnostic::error(
                    "DL1501",
                    format!("manifest exports `{name}`, but the plugin code declares no such function"),
                )
                .with_span(line, "exported here"),
            );
            continue;
        };

        // Exports are monomorphic functions in v1.0 (spec §0 non-goals: no type/effect exports,
        // and signature strings are the ordinary — generic-free — type grammar).
        if !sig.generics.is_empty() {
            diags.push(
                Diagnostic::error(
                    "DL1501",
                    format!("export `{name}` is generic — plugin exports are monomorphic functions in v1.0"),
                )
                .with_span(line, "exported here"),
            );
            continue;
        }

        // Parse the signature string with the ordinary type grammar. Spans inside the string are
        // meaningless outside it, so only the messages are carried into the diagnostic.
        let (sig_ty, sig_diags) = parse_type_string(manifest_file, sig_str);
        if let Some(d) = sig_diags.iter().find(|d| d.is_error()) {
            diags.push(
                Diagnostic::error(
                    "DL1501",
                    format!("export `{name}` signature string does not parse: {}", d.message),
                )
                .with_span(line, "in this signature string"),
            );
            continue;
        }
        if !matches!(sig_ty, TypeExpr::Fn { .. }) {
            diags.push(
                Diagnostic::error(
                    "DL1501",
                    format!("export `{name}` signature must be a function type (exports are functions only in v1.0)"),
                )
                .with_span(line, "in this signature string"),
            );
            continue;
        }

        // Lower with the checker's own rule code, then demand exact equality with the checked type.
        let manifest_ty = match lower_export_signature(&sig_ty, table) {
            Ok(t) => t,
            Err(msg) => {
                diags.push(
                    Diagnostic::error(
                        "DL1501",
                        format!("export `{name}` signature does not lower against the plugin code: {msg}"),
                    )
                    .with_span(line, "in this signature string"),
                );
                continue;
            }
        };
        let code_ty = result.fn_types.get(name).expect("fn in table has a checked type");
        if manifest_ty != *code_ty {
            let regenerated = render_type(code_ty, table);
            diags.push(
                Diagnostic::error(
                    "DL1501",
                    format!(
                        "export `{name}` signature does not match the plugin code — the manifest never overrides the code\n  manifest: {sig_str}\n  code:     {regenerated}"
                    ),
                )
                .with_span(line, "signature declared here")
                .with_repair(Repair {
                    id: "regenerate_manifest_export",
                    confidence: Confidence::Exact,
                    authority_widening: false,
                    requires_human: false,
                    edits: vec![Edit {
                        file: manifest_file,
                        start_byte: line.start,
                        end_byte: line.end,
                        insert: format!("{name} = \"{regenerated}\""),
                    }],
                }),
            );
        }
    }

    diags
}

/// Render a checked `Type` back to **source-grammar text** (the exact text `parse_type_string` +
/// `lower_export_signature` map back to the same type). Used for DL1501 messages and the
/// regenerate-manifest repair. User types render by their declared name from the table.
pub fn render_type(t: &Type, table: &DeclTable) -> String {
    match t {
        Type::Int => "Int".into(),
        Type::Float => "Float".into(),
        Type::Bool => "Bool".into(),
        Type::Str => "Str".into(),
        Type::Unit => "Unit".into(),
        Type::Root => "Root".into(),
        Type::ForeignPtr => "ForeignPtr".into(),
        Type::PyObj => "PyObj".into(),
        Type::List(x) => format!("List[{}]", render_type(x, table)),
        Type::Option(x) => format!("Option[{}]", render_type(x, table)),
        Type::Result(a, b) => format!("Result[{}, {}]", render_type(a, table), render_type(b, table)),
        Type::Secret(x) => format!("Secret[{}]", render_type(x, table)),
        Type::Cap(k) => format!("Cap[{}]", k.name()),
        Type::Foreign(n) => n.clone(),
        Type::Record(id, args) | Type::Sum(id, args) => {
            let name = &table.type_def(*id).name;
            if args.is_empty() {
                name.clone()
            } else {
                let inner: Vec<String> = args.iter().map(|a| render_type(a, table)).collect();
                format!("{name}[{}]", inner.join(", "))
            }
        }
        Type::Fn { params, ret, row } => {
            let ps: Vec<String> = params.iter().map(|p| render_type(p, table)).collect();
            let mut s = format!("fn({})", ps.join(", "));
            if **ret != Type::Unit {
                s.push_str(&format!(" -> {}", render_type(ret, table)));
            }
            if !row.is_pure() {
                s.push_str(&format!(" ! {}", render_row(row)));
            }
            s
        }
        Type::Plugin(c) => format!("Plugin[{}]", render_type(c, table)),
        Type::Verified => "Verified".into(),
        Type::Contained => "Contained".into(),
        // Defensive: inference variables never appear in a monomorphic export type; render
        // something inert rather than panicking on a hostile/degenerate input.
        Type::Var(_) => "?".into(),
    }
}

fn render_row(row: &Row) -> String {
    let names: Vec<&str> = row.effects.iter().map(|e| e.name()).collect();
    format!("{{{}}}", names.join(", "))
}

// ----- coarse span helpers (same line-granular convention as `manifest.rs`) ---------------------

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

fn arr_of_str(v: Option<&toml::Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check_source;

    const MANIFEST: &str = r#"
[package]
name = "summarize"
version = "0.1.0"
kind = "plugin"

[plugin]
api = 1
class = "verified"

[plugin.authority]
effects  = ["Read"]
requires = ["Cap[FsRead]"]

[plugin.exports]
summarize = "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}"
"#;

    const CODE: &str = "module summarize\n\
        pub fn summarize(fs: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { fs.read_text(path) }\n";

    fn manifest_and_diags(src: &str) -> (Option<PluginManifest>, Vec<Diagnostic>) {
        PluginManifest::parse(src, 0)
    }

    fn export_diags(manifest_src: &str, code: &str) -> Vec<Diagnostic> {
        let (pm, mdiags) = manifest_and_diags(manifest_src);
        assert!(mdiags.iter().all(|d| !d.is_error()), "manifest must parse: {mdiags:?}");
        let pm = pm.unwrap();
        let c = check_source(1, code);
        assert!(!c.has_errors(), "code must check clean: {:?}", c.diagnostics);
        check_plugin_module(&pm, &c.table, &c.result, manifest_src, 0)
    }

    #[test]
    fn parses_the_spec_2_1_manifest() {
        let (pm, diags) = manifest_and_diags(MANIFEST);
        assert!(diags.is_empty(), "{diags:?}");
        let pm = pm.unwrap();
        assert_eq!(pm.name, "summarize");
        assert_eq!(pm.api, 1);
        assert_eq!(pm.class, PluginClass::Verified);
        assert_eq!(pm.authority.effects, vec!["Read"]);
        assert_eq!(pm.authority.requires, vec!["Cap[FsRead]"]);
        assert_eq!(pm.exports["summarize"], "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}");
    }

    #[test]
    fn missing_plugin_table_is_dl1004() {
        let src = "[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"plugin\"\n";
        let (pm, diags) = manifest_and_diags(src);
        assert!(pm.is_none());
        assert!(diags.iter().any(|d| d.code == "DL1004"), "{diags:?}");
    }

    #[test]
    fn bad_class_is_dl1004_never_defaulted() {
        // Invariant 29: class is never inferred — and never defaulted either.
        for bad in ["[plugin]\napi=1\n", "[plugin]\napi=1\nclass=\"turbo\"\n"] {
            let src = format!("[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"plugin\"\n{bad}");
            let (pm, diags) = manifest_and_diags(&src);
            assert!(pm.is_none(), "class must never default: {bad}");
            assert!(diags.iter().any(|d| d.code == "DL1004"), "{diags:?}");
        }
    }

    #[test]
    fn unsupported_api_is_dl1507() {
        let src = "[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"plugin\"\n[plugin]\napi = 2\nclass = \"verified\"\n";
        let (pm, diags) = manifest_and_diags(src);
        assert!(pm.is_none());
        assert!(diags.iter().any(|d| d.code == "DL1507"), "{diags:?}");
    }

    #[test]
    fn malformed_requires_entry_is_dl1004() {
        for bad in ["FsRead", "Cap[Turbo]", "Cap", "fn() -> Int", "Cap[FsRead, FsWrite]"] {
            let src = format!(
                "[package]\nname=\"p\"\nversion=\"0.1.0\"\nkind=\"plugin\"\n[plugin]\napi=1\nclass=\"verified\"\n[plugin.authority]\nrequires=[\"{bad}\"]\n"
            );
            let (pm, diags) = manifest_and_diags(&src);
            assert!(pm.is_none(), "`{bad}` must be refused");
            assert!(diags.iter().any(|d| d.code == "DL1004"), "`{bad}`: {diags:?}");
        }
    }

    #[test]
    fn matching_export_checks_clean() {
        let diags = export_diags(MANIFEST, CODE);
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn manifest_overselling_the_row_is_dl1501() {
        // Manifest claims {Read}; the code is pure. The manifest never overrides the code.
        let code = "module m\npub fn summarize(fs: Cap[FsRead], path: Str) -> Result[Str, IoErr] { Ok(path) }\n";
        let diags = export_diags(MANIFEST, code);
        assert!(diags.iter().any(|d| d.code == "DL1501"), "{diags:?}");
    }

    #[test]
    fn manifest_underselling_the_row_is_dl1501_too() {
        // Manifest claims a pure export; the code Reads. STILL a mismatch (head-chef reminder:
        // both directions) — an undersold row would let a host believe a lie.
        let manifest = MANIFEST.replace(
            "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}",
            "fn(Cap[FsRead], Str) -> Result[Str, IoErr]",
        );
        let diags = export_diags(&manifest, CODE);
        assert!(diags.iter().any(|d| d.code == "DL1501"), "{diags:?}");
    }

    #[test]
    fn wrong_param_type_is_dl1501_with_regenerate_repair() {
        let manifest = MANIFEST.replace("fn(Cap[FsRead], Str)", "fn(Cap[FsWrite], Str)");
        let diags = export_diags(&manifest, CODE);
        let d = diags.iter().find(|d| d.code == "DL1501").expect("DL1501");
        // Spec §7: repair is "regenerate manifest — exact".
        let r = d.repairs.first().expect("the regenerate repair");
        assert_eq!(r.id, "regenerate_manifest_export");
        assert!(!r.authority_widening && !r.requires_human);
        assert!(
            r.edits[0].insert.contains("fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}"),
            "the repair regenerates the code-derived signature: {}",
            r.edits[0].insert
        );
    }

    #[test]
    fn missing_export_fn_is_dl1501() {
        let code = "module m\npub fn other(x: Int) -> Int { x }\n";
        let diags = export_diags(MANIFEST, code);
        assert!(diags.iter().any(|d| d.code == "DL1501" && d.message.contains("no such function")), "{diags:?}");
    }

    #[test]
    fn unparseable_signature_string_is_dl1501() {
        let manifest = MANIFEST.replace("fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}", "fn(Cap[");
        let diags = export_diags(&manifest, CODE);
        assert!(diags.iter().any(|d| d.code == "DL1501" && d.message.contains("does not parse")), "{diags:?}");
    }

    #[test]
    fn non_function_signature_is_dl1501() {
        let manifest = MANIFEST.replace("fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}", "Int");
        let diags = export_diags(&manifest, CODE);
        assert!(diags.iter().any(|d| d.code == "DL1501" && d.message.contains("must be a function type")), "{diags:?}");
    }

    #[test]
    fn generic_export_is_dl1501() {
        let manifest = MANIFEST.replace(
            "summarize = \"fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}\"",
            "id = \"fn(Int) -> Int\"",
        );
        let code = "module m\npub fn id[T](x: T) -> T { x }\n";
        let diags = export_diags(&manifest, code);
        assert!(diags.iter().any(|d| d.code == "DL1501" && d.message.contains("monomorphic")), "{diags:?}");
    }

    #[test]
    fn fn_main_in_a_plugin_package_is_dl1501() {
        let code = "module m\npub fn summarize(fs: Cap[FsRead], path: Str) -> Result[Str, IoErr] ! {Read} { fs.read_text(path) }\nfn main(root: Root) { }\n";
        let diags = export_diags(MANIFEST, code);
        assert!(diags.iter().any(|d| d.code == "DL1501" && d.message.contains("fn main")), "{diags:?}");
    }

    #[test]
    fn render_type_round_trips_through_the_ordinary_grammar() {
        // The regenerated signature must parse + lower back to the very type it was rendered from.
        let c = check_source(0, CODE);
        assert!(!c.has_errors());
        let code_ty = &c.result.fn_types["summarize"];
        let rendered = render_type(code_ty, &c.table);
        let (te, pdiags) = parse_type_string(0, &rendered);
        assert!(pdiags.iter().all(|d| !d.is_error()), "{rendered}: {pdiags:?}");
        let lowered = lower_export_signature(&te, &c.table).expect("lowers");
        assert_eq!(&lowered, code_ty, "render → parse → lower must be the identity: {rendered}");
    }
}
