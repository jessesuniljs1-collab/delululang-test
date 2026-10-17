//! The message-catalog layer (Stage 8, spec §6.1) — the localization foundation.
//!
//! **Architecture (build-order deviations 7–9):**
//! - **en-US lives in the code.** Every diagnostic is BORN with its en-US message (the
//!   `format!` at its site), and every named CLI string's declaration carries its en-US
//!   text. "en-US is complete by construction — the compiler refuses to build with a
//!   missing en-US key" is therefore LITERAL: the key cannot exist without its text.
//!   Catalog files exist for the other voices.
//! - **The machine envelope never sees a catalog.** `json.rs` reads `Diagnostic.message`
//!   (the in-code en-US prose) and nothing here; only [`crate::render_human_localized`]
//!   consults a catalog. Invariant 39 holds by construction, not by test — though the
//!   tests pin it anyway.
//! - **Zero dependencies.** Catalogs are parsed by a bounded, hardened TOML-subset reader
//!   (the `theme.toml` precedent): `[section]` headers and `key = "basic string"` pairs
//!   with standard escapes. That is the entire schema the format needs.
//!
//! **Failure is always fallback, never silence-with-damage:** an unknown key, an unknown
//! placeholder, a malformed line — each yields a DL1704 *warning* and the affected entry
//! falls back to en-US. Prose must never take the compiler down, and a catalog written
//! for a newer compiler must degrade, not explode.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::codes::is_registered;
use crate::diagnostic::Diagnostic;

/// A named CLI string: `(key, en-US text, allowed placeholders)`.
pub type CliString = (&'static str, &'static str, &'static [&'static str]);

/// `authority`'s first line (RW 6.1).
pub const CLI_AUTHORITY_HEADER: CliString =
    ("cli.authority.header", "Authority of `{program}` — what this program can do to your system:", &["program"]);
/// `authority`'s separator above the foreign libraries — code outside the proof (RW 6.1).
pub const CLI_AUTHORITY_FOREIGN_SEPARATOR: CliString =
    ("cli.authority.foreign-separator", "-- outside the proof (contained at process level) --", &[]);
/// `locale add`'s question before it installs a catalog; the answer is still `y` (RW 6.1).
pub const CLI_LOCALE_ADD_CONFIRM: CliString = ("cli.locale.add.confirm", "install locale `{locale}`? [y/N]: ", &["locale"]);

/// Named CLI strings. The declaration HERE is the key's existence — a call site names the
/// constant ([`render_cli`]), so a missing en-US text is a compile error (the literal
/// "refuses to build" of spec §6.1). Grows with the surfaces (RW 6.1: the first three,
/// 2026-10-17; the first-run picker stays en-US — it is shown before any locale exists).
pub const CLI_STRINGS: &[CliString] = &[CLI_AUTHORITY_HEADER, CLI_AUTHORITY_FOREIGN_SEPARATOR, CLI_LOCALE_ADD_CONFIRM];

/// A named CLI string in `catalog`'s voice, with `args` — or its en-US text when there is
/// no catalog, the catalog has no entry for it, or the entry cannot render (never half a
/// message). Either way it is made safe for a terminal: a catalog is text anyone may
/// write, and an argument may be a name a program chose.
pub fn render_cli(s: CliString, catalog: Option<&Catalog>, args: &[(&str, &str)]) -> String {
    let args: Vec<(String, String)> = args.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let text = catalog.and_then(|c| c.render(s.0, &args)).or_else(|| fill(s.1, &args)).unwrap_or_else(|| s.1.to_string());
    crate::render::terminal_safe(&text).into_owned()
}

/// A template with its `{name}` placeholders filled from `args`; `None` when a value it
/// needs is missing.
fn fill(template: &str, args: &[(String, String)]) -> Option<String> {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let j = after.find('}')?;
        let name = &after[..j];
        let val = args.iter().find(|(k, _)| k == name)?;
        out.push_str(&val.1);
        rest = &after[j + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// Look up a named CLI string's en-US text.
pub fn cli_string(key: &str) -> Option<&'static str> {
    CLI_STRINGS.iter().find(|(k, _, _)| *k == key).map(|(_, t, _)| *t)
}

/// The placeholder vocabulary a diagnostic code's catalog entries may use. Grown site by
/// site as diagnostics gain `with_arg` values; a code not listed here accepts NO
/// placeholders (a catalog entry using one gets DL1704 and falls back).
fn code_placeholders(code: &str) -> &'static [&'static str] {
    match code {
        "DL0501" | "DL0502" => &["fn", "effect"],
        "DL0602" => &["inner"],
        "DL1205" => &["detail"],
        _ => &[],
    }
}

/// The full allowed-placeholder set for a catalog key, or `None` if the key itself is
/// unknown (not a registered diagnostic code, not a declared CLI string).
pub fn placeholders_for(key: &str) -> Option<&'static [&'static str]> {
    if key.starts_with("DL") {
        return is_registered(key).then(|| code_placeholders(key));
    }
    CLI_STRINGS.iter().find(|(k, _, _)| *k == key).map(|(_, _, p)| *p)
}

/// One loaded, validated catalog: locale metadata + `key → message template`.
#[derive(Debug, Clone)]
pub struct Catalog {
    pub locale: String,
    pub version: String,
    pub fallback: Option<String>,
    /// Free-text coverage declaration from `[meta]` — the picker shows it (spec §6.1).
    pub coverage: Option<String>,
    entries: HashMap<String, String>,
}

impl Catalog {
    /// Parse and validate catalog text. Always returns a catalog (possibly empty) plus the
    /// DL1704 warnings its defects produced — load never fails hard, entries fall back.
    pub fn parse(text: &str) -> (Catalog, Vec<Diagnostic>) {
        let mut cat =
            Catalog { locale: String::new(), version: String::new(), fallback: None, coverage: None, entries: HashMap::new() };
        let mut warnings = Vec::new();
        // Current section: None = prologue, Some(("meta", _)) or Some((key, allowed)).
        enum Section {
            Meta,
            Entry { key: String, allowed: &'static [&'static str] },
            Skipped,
        }
        let mut section: Option<Section> = None;

        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let name = name.trim();
                if name == "meta" {
                    section = Some(Section::Meta);
                } else if name.starts_with("cli.first-run.welcome") {
                    // Spec §6.3: the first-run welcome is never a catalog key — no catalog
                    // may translate, paraphrase, or override it, in any locale, ever.
                    warnings.push(dl1704(format!(
                        "catalog key `{name}` (line {}) attempts to override the first-run \
                         welcome — the welcome is law and no catalog may touch it",
                        lineno + 1
                    )));
                    section = Some(Section::Skipped);
                } else {
                    match placeholders_for(name) {
                        Some(allowed) => section = Some(Section::Entry { key: name.to_string(), allowed }),
                        None => {
                            warnings.push(dl1704(format!(
                                "catalog key `{name}` (line {}) is not a registered diagnostic \
                                 code or a named CLI string — its entry is ignored",
                                lineno + 1
                            )));
                            section = Some(Section::Skipped);
                        }
                    }
                }
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                warnings.push(dl1704(format!("malformed catalog line {} (expected `key = \"value\"`) — ignored", lineno + 1)));
                continue;
            };
            let k = k.trim();
            let Some(val) = parse_basic_string(v.trim()) else {
                // Meta may carry non-string values (tables, numbers) — tolerated and
                // ignored there; inside an entry a non-string value cannot be a template.
                if matches!(section, Some(Section::Entry { .. })) {
                    warnings.push(dl1704(format!("catalog line {} has a non-string value — entry field ignored", lineno + 1)));
                }
                continue;
            };
            match &section {
                Some(Section::Meta) => match k {
                    "locale" => cat.locale = val,
                    "version" => cat.version = val,
                    "fallback" => cat.fallback = Some(val),
                    "coverage" => cat.coverage = Some(val),
                    _ => {} // forward compatibility: name, direction, …
                },
                Some(Section::Entry { key, allowed }) => {
                    if k == "message" {
                        match unknown_placeholder(&val, allowed) {
                            Some(bad) => warnings.push(dl1704(format!(
                                "catalog entry `{key}` uses unknown placeholder `{{{bad}}}` — \
                                 the entry falls back to en-US"
                            ))),
                            None => {
                                cat.entries.insert(key.clone(), val);
                            }
                        }
                    }
                    // `label`/`help` are parsed-and-reserved in v0.8: the renderer keys
                    // instance labels by span, which a code-keyed catalog cannot address.
                }
                Some(Section::Skipped) | None => {}
            }
        }
        (cat, warnings)
    }

    /// Render `key` with the given placeholder values. `None` — entry absent, or a value
    /// the template needs is missing — means THE WHOLE entry falls back to en-US: a
    /// half-rendered message with a literal `{fn}` in it would be worse than no catalog.
    pub fn render(&self, key: &str, args: &[(String, String)]) -> Option<String> {
        fill(self.entries.get(key)?, args)
    }

    /// Number of renderable entries (for the picker's coverage line).
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The embedded `delulu-slang` catalog — parsed once, validated by the suite to load
    /// with ZERO warnings (a defective shipped catalog is a broken build, not a runtime
    /// surprise).
    pub fn delulu_slang() -> &'static Catalog {
        static SLANG: OnceLock<Catalog> = OnceLock::new();
        SLANG.get_or_init(|| {
            let (cat, _warnings) = Catalog::parse(DELULU_SLANG_TOML);
            cat
        })
    }
}

/// The shipped `delulu-slang` catalog text (docs/lang/delulu-slang.md is its authoring
/// spec; the voice rules there bind every entry).
pub const DELULU_SLANG_TOML: &str = include_str!("../catalogs/delulu-slang.toml");

fn dl1704(msg: String) -> Diagnostic {
    Diagnostic::warning("DL1704", msg)
}

/// First placeholder in `template` that is not in `allowed`, if any.
fn unknown_placeholder(template: &str, allowed: &[&str]) -> Option<String> {
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        let after = &rest[i + 1..];
        let Some(j) = after.find('}') else {
            // An unclosed brace can never render — treat it as an unknown placeholder.
            return Some(after.chars().take(12).collect());
        };
        let name = &after[..j];
        if !allowed.contains(&name) {
            return Some(name.to_string());
        }
        rest = &after[j + 1..];
    }
    None
}

/// Parse a TOML basic string: `"…"` with `\\ \" \n \t \r \uXXXX \UXXXXXXXX` escapes.
/// Returns `None` when the value is not a (single-line) basic string. Trailing content
/// after the closing quote is tolerated if it is whitespace or a `#` comment.
fn parse_basic_string(v: &str) -> Option<String> {
    let inner = v.strip_prefix('"')?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    loop {
        match chars.next()? {
            '"' => break,
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => out.push(unicode_escape(&mut chars, 4)?),
                'U' => out.push(unicode_escape(&mut chars, 8)?),
                _ => return None,
            },
            c => out.push(c),
        }
    }
    let rest: String = chars.collect();
    let rest = rest.trim();
    (rest.is_empty() || rest.starts_with('#')).then_some(out)
}

fn unicode_escape(chars: &mut std::str::Chars<'_>, n: usize) -> Option<char> {
    let mut v: u32 = 0;
    for _ in 0..n {
        v = v.checked_mul(16)?.checked_add(chars.next()?.to_digit(16)?)?;
    }
    char::from_u32(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    /// RW 6.1: every named CLI string is one key, `cli.`-spelled, never the welcome, and its en-US text uses exactly
    /// the placeholders it declares — a catalog may use no others, and en-US must fill every one.
    #[test]
    fn every_named_cli_string_declares_exactly_the_placeholders_its_text_uses() {
        let mut keys: Vec<&str> = CLI_STRINGS.iter().map(|(k, _, _)| *k).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), CLI_STRINGS.len(), "a key declared twice");
        for (key, en_us, allowed) in CLI_STRINGS {
            assert!(key.starts_with("cli.") && !key.starts_with("cli.first-run.welcome"), "{key}");
            let used: Vec<&str> = en_us.split('{').skip(1).filter_map(|s| s.split_once('}').map(|(n, _)| n)).collect();
            let mut want: Vec<&str> = allowed.to_vec();
            let mut got = used.clone();
            want.sort_unstable();
            got.sort_unstable();
            assert_eq!(got, want, "{key}: the en-US text uses {used:?}");
            assert_eq!(placeholders_for(key), Some(*allowed), "{key} is in the key space");
        }
    }

    /// RW 6.1: the shipped slang voice covers every named CLI string, each rendering whole with its arguments.
    #[test]
    fn the_shipped_slang_catalog_speaks_every_named_cli_string() {
        let cat = Catalog::delulu_slang();
        for s @ (key, en_us, allowed) in CLI_STRINGS.iter().copied() {
            let a: Vec<(&str, &str)> = allowed.iter().map(|p| (*p, "X")).collect();
            let slang = render_cli(s, Some(cat), &a);
            assert!(cat.render(key, &args(&a)).is_some(), "{key} has a slang entry");
            assert_ne!(slang, render_cli(s, None, &a), "{key} speaks slang, not {en_us:?}");
            assert!(!slang.contains('{'), "{key} renders whole: {slang}");
        }
    }

    /// RW 6.1: `render_cli` falls back to en-US whole — no catalog, no entry, or an entry missing its argument — and
    /// what it prints is safe for a terminal, whoever wrote the catalog or chose the argument.
    #[test]
    fn render_cli_falls_back_whole_and_is_terminal_safe() {
        let h = CLI_AUTHORITY_HEADER;
        let en = "Authority of `m` — what this program can do to your system:";
        assert_eq!(render_cli(h, None, &[("program", "m")]), en);
        let (empty, _) = Catalog::parse("[meta]\nlocale = \"x\"\n");
        assert_eq!(render_cli(h, Some(&empty), &[("program", "m")]), en, "no entry → en-US");
        let (hostile, w) = Catalog::parse("[meta]\nlocale = \"x\"\n[cli.authority.header]\nmessage = \"\\u001b[8m{program}\"\n");
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(
            render_cli(h, Some(&hostile), &[]),
            "Authority of `{program}` — what this program can do to your system:",
            "an argument missing → the template, never half a message"
        );
        assert!(
            !render_cli(h, Some(&hostile), &[("program", "m")]).contains('\u{1b}'),
            "a catalog's escape is shown, not obeyed"
        );
        assert!(!render_cli(h, None, &[("program", "m\u{1b}[2J")]).contains('\u{1b}'), "so is an argument's");
    }

    /// The validity gate for the SHIPPED catalog: it must load with zero DL1704 warnings —
    /// every key registered, every placeholder declared. A defect here fails the suite,
    /// which is what "a broken shipped catalog cannot ship" means in practice.
    #[test]
    fn the_shipped_slang_catalog_loads_with_zero_warnings() {
        let (cat, warnings) = Catalog::parse(DELULU_SLANG_TOML);
        assert!(warnings.is_empty(), "shipped catalog must be defect-free: {warnings:?}");
        assert_eq!(cat.locale, "delulu-slang");
        assert_eq!(cat.fallback.as_deref(), Some("en-US"));
        assert!(cat.coverage.is_some(), "the picker needs the coverage declaration");
        assert!(!cat.is_empty());
    }

    #[test]
    fn dl0501_renders_the_slang_template_with_args() {
        // `{fn}` carries the full subject desc — "function `greet`" or `test "t"` — so the
        // slang voice stays honest for test blocks too (they are not functions).
        let cat = Catalog::delulu_slang();
        let rendered = cat
            .render("DL0501", &args(&[("fn", "function `greet`"), ("effect", "Write")]))
            .expect("DL0501 is in the shipped set");
        assert!(rendered.contains("function `greet`"), "{rendered}");
        assert!(rendered.contains("`Write`"), "{rendered}");
        assert!(!rendered.contains('{'), "no unfilled placeholder may survive: {rendered}");
    }

    /// THE COULDN'T-TELL CASE: a template that needs `{fn}` but a diagnostic that carries
    /// no args — the entry must fall back entirely (None), never render `{fn}` literally.
    #[test]
    fn a_missing_arg_falls_back_to_en_us_never_renders_half() {
        let cat = Catalog::delulu_slang();
        assert_eq!(cat.render("DL0501", &[]), None);
        assert_eq!(cat.render("DL0501", &args(&[("fn", "greet")])), None, "one of two args");
    }

    #[test]
    fn an_unknown_placeholder_is_dl1704_and_the_entry_falls_back() {
        let src = "[meta]\nlocale = \"x\"\n[DL0501]\nmessage = \"fn {fn} did {nope}\"\n";
        let (cat, warnings) = Catalog::parse(src);
        assert!(warnings.iter().any(|d| d.code == "DL1704" && d.message.contains("{nope}")), "{warnings:?}");
        assert_eq!(cat.render("DL0501", &args(&[("fn", "f")])), None, "entry dropped");
    }

    /// Spec §6.3 / criterion 8's DL1704 half: the welcome is not a catalog key, and an
    /// attempt to claim it is called out BY NAME, not as a generic unknown key.
    #[test]
    fn a_welcome_override_attempt_is_dl1704_by_name() {
        let src = "[cli.first-run.welcome]\nmessage = \"new welcome who dis\"\n";
        let (cat, warnings) = Catalog::parse(src);
        assert!(warnings.iter().any(|d| d.code == "DL1704" && d.message.contains("welcome is law")), "{warnings:?}");
        assert!(cat.is_empty(), "the entry must not load");
    }

    #[test]
    fn an_unknown_key_is_dl1704_and_skipped() {
        let src = "[DL9999]\nmessage = \"who dis\"\n[cli.not.a.thing]\nmessage = \"nope\"\n";
        let (cat, warnings) = Catalog::parse(src);
        assert_eq!(warnings.iter().filter(|d| d.code == "DL1704").count(), 2, "{warnings:?}");
        assert!(cat.is_empty());
    }

    #[test]
    fn hostile_text_warns_and_never_panics() {
        for bad in [
            "[DL0501]\nmessage = \"unterminated\n",
            "[DL0501\nmessage = \"x\"\n",
            "= = =\n[]\n\u{0000}\n",
            "[DL0501]\nmessage = \"bad escape \\q\"\n",
            "[DL0501]\nmessage = \"unclosed {brace\"\n",
        ] {
            let (_cat, _warnings) = Catalog::parse(bad); // must simply not panic
        }
    }

    #[test]
    fn escapes_and_emoji_round_trip() {
        let src = "[meta]\nlocale = \"x\"\n[DL0605]\nmessage = \"say \\\"no\\\" \\u2014 \\\\always 💀\"\n";
        let (cat, w) = Catalog::parse(src);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(cat.render("DL0605", &[]).unwrap(), "say \"no\" — \\always 💀");
    }
}
