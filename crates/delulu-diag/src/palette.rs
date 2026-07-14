//! The Palette — a role-based color system for every human-facing CLI surface (Surface addendum
//! §2.5). It lives here in `delulu-diag` so every renderer (diagnostics, guard banners, `ok:`
//! lines, grants tree, the atlas tree) shares one color vocabulary.
//!
//! **Roles, not colors.** A renderer never names a color; it names a semantic [`Role`] and the
//! active [`Theme`] maps that role to an SGR sequence. Color decisions live in exactly one place.
//!
//! **Determinism (addendum §2 / criterion 1 & 10).** A disabled palette is byte-identical to no
//! palette at all: [`Palette::paint`] returns its input unchanged. Non-TTY output is colorless by
//! default, so every pre-existing test — which captures the binary through pipes — is unaffected.
//! Machine channels (`--json`, `atlas/1`) are never handed a live palette (criterion 8).
//!
//! **No new dependency.** SGR sequences are hand-rolled ANSI (`\x1b[…m`); the theme file is parsed
//! by a tiny line reader below. Windows VT-processing is enabled by the CLI via the `windows-sys`
//! already in its tree — this crate stays dependency-clean (addendum §2.5 / §7 deviation 1).

use std::collections::BTreeMap;
use std::path::Path;

/// A semantic role a renderer asks the palette to paint. The renderer names the role; the theme
/// decides the color. Ordered/`Ord` so a theme can key a `BTreeMap` on it deterministically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// `error` severity word and its bracketed code.
    Error,
    /// `warning` severity word.
    Warning,
    /// `note` severity word.
    Note,
    /// A `DLxxxx` diagnostic code.
    Code,
    /// The primary span caret line and its label.
    SpanPrimary,
    /// A secondary span underline and its label.
    SpanSecondary,
    /// An effect name (`Net`, `Read`, …).
    Effect,
    /// An authority / capability value.
    Authority,
    /// The guard bypass banner — the strongest error style.
    GuardBanner,
    /// An `ok:` / success line.
    Success,
    /// A filesystem path or resource pattern.
    Path,
    /// A typed repair suggestion.
    Repair,
    /// A section heading (explain topics, atlas tree headers).
    Heading,
}

impl Role {
    /// Every role, in declaration order — for enumeration in tests and theme construction.
    pub const ALL: [Role; 13] = [
        Role::Error,
        Role::Warning,
        Role::Note,
        Role::Code,
        Role::SpanPrimary,
        Role::SpanSecondary,
        Role::Effect,
        Role::Authority,
        Role::GuardBanner,
        Role::Success,
        Role::Path,
        Role::Repair,
        Role::Heading,
    ];

    /// The stable snake_case key used in `theme.toml`'s `[roles]` table.
    pub fn key(self) -> &'static str {
        match self {
            Role::Error => "error",
            Role::Warning => "warning",
            Role::Note => "note",
            Role::Code => "code",
            Role::SpanPrimary => "span_primary",
            Role::SpanSecondary => "span_secondary",
            Role::Effect => "effect",
            Role::Authority => "authority",
            Role::GuardBanner => "guard_banner",
            Role::Success => "success",
            Role::Path => "path",
            Role::Repair => "repair",
            Role::Heading => "heading",
        }
    }

    /// Parse a `[roles]` key back to a role. Unknown keys yield `None` (→ DL1790).
    pub fn from_key(s: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.key() == s)
    }
}

/// A theme maps roles to SGR parameter strings (the bytes between `\x1b[` and `m`). A role missing
/// from the map — or mapped to `""` — is painted as plain text (no escape emitted at all, so mono's
/// unstyled roles never leak a bare `\x1b[m`).
#[derive(Clone, Debug)]
pub struct Theme {
    name: String,
    sgr: BTreeMap<Role, String>,
}

impl Theme {
    fn from_pairs(name: &str, pairs: &[(Role, &str)]) -> Theme {
        let mut sgr = BTreeMap::new();
        for (role, code) in pairs {
            sgr.insert(*role, (*code).to_string());
        }
        Theme { name: name.to_string(), sgr }
    }

    /// The theme's name (`default` / `bright` / `mono`, or a custom base).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The SGR parameters for a role, or `""` when the role is unstyled in this theme.
    pub fn sgr(&self, role: Role) -> &str {
        self.sgr.get(&role).map(String::as_str).unwrap_or("")
    }

    /// Override one role's SGR (used to apply `theme.toml`'s `[roles]` table).
    pub fn set(&mut self, role: Role, sgr: String) {
        self.sgr.insert(role, sgr);
    }

    /// Look up a built-in theme by name.
    pub fn builtin(name: &str) -> Option<Theme> {
        match name {
            "default" => Some(Theme::default_theme()),
            "bright" => Some(Theme::bright_theme()),
            "mono" => Some(Theme::mono_theme()),
            _ => None,
        }
    }

    /// The colorblind-safe default: severity words carry the meaning; color is a secondary cue and
    /// no two roles are told apart by red/green alone (they always differ in text too).
    pub fn default_theme() -> Theme {
        Theme::from_pairs(
            "default",
            &[
                (Role::Error, "1;31"),
                (Role::Warning, "1;33"),
                (Role::Note, "1;36"),
                (Role::Code, "1"),
                (Role::SpanPrimary, "1;31"),
                (Role::SpanSecondary, "36"),
                (Role::Effect, "35"),
                (Role::Authority, "34"),
                (Role::GuardBanner, "1;31"),
                (Role::Success, "32"),
                (Role::Path, "36"),
                (Role::Repair, "32"),
                (Role::Heading, "1;4"),
            ],
        )
    }

    /// A higher-contrast variant using the bright (90-series) colors.
    pub fn bright_theme() -> Theme {
        Theme::from_pairs(
            "bright",
            &[
                (Role::Error, "1;91"),
                (Role::Warning, "1;93"),
                (Role::Note, "1;96"),
                (Role::Code, "1"),
                (Role::SpanPrimary, "1;91"),
                (Role::SpanSecondary, "96"),
                (Role::Effect, "95"),
                (Role::Authority, "94"),
                (Role::GuardBanner, "1;97;41"),
                (Role::Success, "1;92"),
                (Role::Path, "96"),
                (Role::Repair, "92"),
                (Role::Heading, "1;4"),
            ],
        )
    }

    /// `mono` — bold/underline only, ZERO color SGR (addendum §2.5, criterion 9). Every parameter
    /// here is drawn from {1 (bold), 4 (underline)}; the test `mono_emits_no_color_sgr` enforces it.
    pub fn mono_theme() -> Theme {
        Theme::from_pairs(
            "mono",
            &[
                (Role::Error, "1"),
                (Role::Warning, "1"),
                (Role::Note, "1"),
                (Role::Code, "1"),
                (Role::SpanPrimary, "1"),
                (Role::SpanSecondary, ""),
                (Role::Effect, "1"),
                (Role::Authority, "1"),
                (Role::GuardBanner, "1"),
                (Role::Success, "1"),
                (Role::Path, "4"),
                (Role::Repair, "1"),
                (Role::Heading, "1;4"),
            ],
        )
    }
}

/// Map a named 16-color value (`red`, `bright_blue`, …) to its foreground SGR code, for `theme.toml`
/// `[roles]` overrides. Returns `None` for an unknown name (→ DL1790).
pub fn named_color_sgr(name: &str) -> Option<String> {
    let name = name.trim().to_ascii_lowercase();
    let (base, bright) = match name.strip_prefix("bright_") {
        Some(rest) => (rest, true),
        None => (name.as_str(), false),
    };
    let idx = match base {
        "black" => 0,
        "red" => 1,
        "green" => 2,
        "yellow" => 3,
        "blue" => 4,
        "magenta" => 5,
        "cyan" => 6,
        "white" => 7,
        _ => return None,
    };
    let code = if bright { 90 + idx } else { 30 + idx };
    Some(code.to_string())
}

/// The user's explicit color choice from `--color`/`DELULU_COLOR`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorChoice {
    /// Decide by the stream (on iff a TTY).
    Auto,
    /// Force color on.
    Always,
    /// Force color off.
    Never,
}

impl ColorChoice {
    /// Parse a `--color`/`DELULU_COLOR` value. Accepts the canonical `always|never|auto` plus the
    /// common boolean spellings. Unknown values yield `None` (treated as "unset" by the resolver).
    pub fn parse(s: &str) -> Option<ColorChoice> {
        match s.trim().to_ascii_lowercase().as_str() {
            "always" | "1" | "true" | "yes" | "on" => Some(ColorChoice::Always),
            "never" | "0" | "false" | "no" | "off" => Some(ColorChoice::Never),
            "auto" => Some(ColorChoice::Auto),
            _ => None,
        }
    }
}

/// Resolve whether color is ON for one stream. Precedence (addendum §2.5 / criterion 8):
/// `--color` flag > `DELULU_COLOR` > `NO_COLOR` > auto(TTY) — with the single exception that
/// `NO_COLOR` beats `DELULU_COLOR=always` (but never the explicit `--color always`, the user
/// speaking now). `--color auto` is not a forced decision: it falls through to the env layers.
pub fn color_enabled(
    flag: Option<ColorChoice>,
    delulu_color: Option<&str>,
    no_color: bool,
    is_tty: bool,
) -> bool {
    // 1. Explicit flag — the user speaking now. `always` even beats NO_COLOR.
    match flag {
        Some(ColorChoice::Always) => return true,
        Some(ColorChoice::Never) => return false,
        Some(ColorChoice::Auto) | None => {}
    }
    // 2. DELULU_COLOR — but NO_COLOR beats DELULU_COLOR=always.
    match delulu_color.and_then(ColorChoice::parse) {
        Some(ColorChoice::Always) => return !no_color,
        Some(ColorChoice::Never) => return false,
        Some(ColorChoice::Auto) | None => {}
    }
    // 3. NO_COLOR (present & non-empty ⇒ off; https://no-color.org).
    if no_color {
        return false;
    }
    // 4. auto: on iff the stream is a TTY.
    is_tty
}

/// The active palette: whether color is on, and the theme to paint with. Cheap to clone.
#[derive(Clone, Debug)]
pub struct Palette {
    enabled: bool,
    theme: Theme,
}

impl Palette {
    /// A disabled palette. Every `paint` returns its input unchanged — byte-identical to no color.
    pub fn none() -> Self {
        Palette { enabled: false, theme: Theme::default_theme() }
    }

    /// Build a palette with an explicit on/off decision and theme.
    pub fn new(enabled: bool, theme: Theme) -> Self {
        Palette { enabled, theme }
    }

    /// Whether this palette emits color.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The active theme's name.
    pub fn theme_name(&self) -> &str {
        self.theme.name()
    }

    /// Paint `text` in `role`'s style. A disabled palette — or a role that is unstyled in the active
    /// theme — returns `text` unchanged (never a bare/empty escape), preserving byte-determinism.
    pub fn paint(&self, role: Role, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        let sgr = self.theme.sgr(role);
        if sgr.is_empty() {
            return text.to_string();
        }
        format!("\x1b[{sgr}m{text}\x1b[0m")
    }
}

/// The reset sequence, exposed for renderers that must colorize a multi-line block manually.
pub const RESET: &str = "\x1b[0m";

/// Resolve the active theme from (in precedence order) `--theme` → `DELULU_THEME` →
/// `theme.toml` (`theme = "…"` + optional `[roles]`), falling back to `default`. A bad theme NAME,
/// an unreadable/unparseable file, or an unknown role/color in `[roles]` never hard-fails: the
/// default theme is used and a DL1790 warning message is returned (addendum §2.5 / criterion 9).
pub fn resolve_theme(
    flag: Option<&str>,
    env_theme: Option<&str>,
    config_path: Option<&Path>,
) -> (Theme, Option<String>) {
    // Read the config file, if any. A missing/unreadable file means "no config" (no warning); an
    // oversized, non-UTF-8, or structurally broken file is a DL1790 (never a hard failure).
    let mut cfg_theme_name: Option<String> = None;
    let mut cfg_roles: Vec<(String, String)> = Vec::new();
    let mut file_bad = false;
    if let Some(path) = config_path {
        match read_capped(path, THEME_FILE_MAX) {
            Ok(Some(text)) => match parse_theme_toml(&text) {
                Ok((name, roles)) => {
                    cfg_theme_name = name;
                    cfg_roles = roles;
                }
                Err(()) => file_bad = true,
            },
            Ok(None) => {}       // no file / unreadable → simply no config
            Err(()) => file_bad = true, // too large or not UTF-8 → malformed, fall back
        }
    }

    // Choose the base theme by name precedence.
    let requested: Option<String> = flag
        .map(str::to_string)
        .or_else(|| env_theme.map(str::to_string))
        .or_else(|| cfg_theme_name.clone());
    let (mut theme, name_bad) = match requested.as_deref() {
        Some(n) => match Theme::builtin(n) {
            Some(t) => (t, false),
            None => (Theme::default_theme(), true),
        },
        None => (Theme::default_theme(), false),
    };

    // Apply `[roles]` overrides on top of the base (skip when the file itself failed to parse).
    let mut role_bad = false;
    if !file_bad {
        for (k, v) in &cfg_roles {
            match (Role::from_key(k), named_color_sgr(v)) {
                (Some(role), Some(sgr)) => theme.set(role, sgr),
                _ => role_bad = true,
            }
        }
    }

    let warning = if name_bad {
        Some(format!(
            "unknown theme `{}` — using `default` (built-ins: default, bright, mono)",
            requested.unwrap_or_default()
        ))
    } else if file_bad {
        Some("malformed theme.toml — using `default`".to_string())
    } else if role_bad {
        Some("theme.toml has an unknown role or color name — ignoring it, using `default` for it".to_string())
    } else {
        None
    };
    (theme, warning)
}

/// The theme file is user-controlled input; cap how much of it we will read so a hostile or
/// accidentally-huge file can never exhaust memory. A real `theme.toml` is a handful of lines; 64
/// KiB is astronomically generous. Anything larger is treated as malformed (→ DL1790, default theme).
const THEME_FILE_MAX: u64 = 64 * 1024;

/// Read at most `cap` bytes of a file as UTF-8. `Ok(None)` = missing/unreadable (no config, no
/// warning). `Err(())` = larger than `cap` or not valid UTF-8 (→ caller emits DL1790). Bounds
/// memory to `cap + 1` bytes regardless of the file's real size.
fn read_capped(path: &Path, cap: u64) -> Result<Option<String>, ()> {
    use std::io::Read;
    let Ok(file) = std::fs::File::open(path) else {
        return Ok(None);
    };
    let mut buf = Vec::new();
    if file.take(cap + 1).read_to_end(&mut buf).is_err() {
        return Ok(None);
    }
    if buf.len() as u64 > cap {
        return Err(()); // oversized — refuse to grow further
    }
    String::from_utf8(buf).map(Some).map_err(|_| ())
}

/// Parsed contents of a `theme.toml`: the optional `theme = "…"` name and the `[roles]` overrides.
type ThemeToml = (Option<String>, Vec<(String, String)>);

/// A minimal reader for the theme file: a top-level `theme = "name"` key and an optional `[roles]`
/// table of `role = "color"` pairs. Deliberately tiny — no TOML crate is pulled in. Returns
/// `Err(())` on a structurally broken line so the caller can emit DL1790 and fall back.
fn parse_theme_toml(text: &str) -> Result<ThemeToml, ()> {
    let mut theme_name: Option<String> = None;
    let mut roles: Vec<(String, String)> = Vec::new();
    let mut in_roles = false;
    for raw in text.lines() {
        // Strip a `#` comment, then trim.
        let line = match raw.find('#') {
            Some(i) => &raw[..i],
            None => raw,
        }
        .trim();
        if line.is_empty() {
            continue;
        }
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_roles = section.trim() == "roles";
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            return Err(());
        };
        let key = key.trim();
        let val = val.trim();
        // Values are bare or double-quoted strings.
        let val = val.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(val).trim();
        if key.is_empty() || val.is_empty() {
            return Err(());
        }
        if in_roles {
            roles.push((key.to_string(), val.to_string()));
        } else if key == "theme" {
            theme_name = Some(val.to_string());
        }
        // Unknown top-level keys are ignored (forward compatibility).
    }
    Ok((theme_name, roles))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_palette_is_byte_identical() {
        let p = Palette::none();
        assert_eq!(p.paint(Role::Error, "boom"), "boom");
        assert_eq!(p.paint(Role::Success, "ok: done"), "ok: done");
        assert!(!p.is_enabled());
    }

    #[test]
    fn enabled_palette_wraps_in_sgr() {
        let p = Palette::new(true, Theme::default_theme());
        assert_eq!(p.paint(Role::Error, "x"), "\x1b[1;31mx\x1b[0m");
        assert_eq!(p.paint(Role::Success, "ok"), "\x1b[32mok\x1b[0m");
    }

    /// Criterion 9: `mono` emits bold/underline only — no color SGR at all.
    #[test]
    fn mono_emits_no_color_sgr() {
        let p = Palette::new(true, Theme::mono_theme());
        for role in Role::ALL {
            let painted = p.paint(role, "T");
            // Extract the SGR body between the first `\x1b[` and `m`.
            if let Some(rest) = painted.strip_prefix("\x1b[") {
                let body = &rest[..rest.find('m').unwrap()];
                for param in body.split(';') {
                    let n: u32 = param.parse().unwrap_or(0);
                    let is_color = (30..=37).contains(&n)
                        || (40..=47).contains(&n)
                        || (90..=97).contains(&n)
                        || (100..=107).contains(&n)
                        || n == 38
                        || n == 48;
                    assert!(!is_color, "mono role {role:?} used a color SGR `{param}`");
                }
            }
            // A disabled/unstyled role must never leak an empty escape.
            assert!(!painted.contains("\x1b[m"), "empty escape for {role:?}");
        }
    }

    /// Criterion 8: the full precedence lattice, including the NO_COLOR-beats-DELULU_COLOR=always
    /// exception and the --color-always-beats-NO_COLOR carve-out.
    #[test]
    fn color_precedence_lattice() {
        use ColorChoice::*;
        // Explicit flag wins absolutely.
        assert!(color_enabled(Some(Always), Some("never"), true, false));
        assert!(!color_enabled(Some(Never), Some("always"), false, true));
        // --color always beats NO_COLOR (the user speaking now).
        assert!(color_enabled(Some(Always), None, true, false));
        // DELULU_COLOR next.
        assert!(color_enabled(None, Some("always"), false, false));
        assert!(!color_enabled(None, Some("never"), false, true));
        // NO_COLOR beats DELULU_COLOR=always.
        assert!(!color_enabled(None, Some("always"), true, true));
        // NO_COLOR alone.
        assert!(!color_enabled(None, None, true, true));
        // auto: TTY decides.
        assert!(color_enabled(None, None, false, true));
        assert!(!color_enabled(None, None, false, false));
        // --color auto falls through to env/TTY, honoring NO_COLOR.
        assert!(!color_enabled(Some(Auto), None, true, true));
        assert!(color_enabled(Some(Auto), None, false, true));
    }

    #[test]
    fn named_colors_map_to_sgr() {
        assert_eq!(named_color_sgr("red").as_deref(), Some("31"));
        assert_eq!(named_color_sgr("bright_blue").as_deref(), Some("94"));
        assert_eq!(named_color_sgr("white").as_deref(), Some("37"));
        assert_eq!(named_color_sgr("mauve"), None);
    }

    #[test]
    fn theme_toml_role_override_is_honored() {
        let text = "theme = \"bright\"\n[roles]\nerror = \"green\"\n";
        let (name, roles) = parse_theme_toml(text).unwrap();
        assert_eq!(name.as_deref(), Some("bright"));
        assert_eq!(roles, vec![("error".to_string(), "green".to_string())]);
        // Apply through resolve_theme with a temp file.
        let dir = std::env::temp_dir().join(format!("delulu_theme_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.toml");
        std::fs::write(&path, text).unwrap();
        let (theme, warn) = resolve_theme(None, None, Some(&path));
        assert!(warn.is_none(), "clean override: {warn:?}");
        assert_eq!(theme.name(), "bright");
        assert_eq!(theme.sgr(Role::Error), "32", "error overridden to green");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bad_theme_name_falls_back_with_warning() {
        let (theme, warn) = resolve_theme(Some("neon"), None, None);
        assert_eq!(theme.name(), "default");
        assert!(warn.unwrap().contains("neon"));
    }

    #[test]
    fn malformed_theme_file_falls_back_with_warning() {
        let dir = std::env::temp_dir().join(format!("delulu_theme_bad_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.toml");
        std::fs::write(&path, "this is not = = valid\n[[[").unwrap();
        let (theme, warn) = resolve_theme(None, None, Some(&path));
        assert_eq!(theme.name(), "default");
        assert!(warn.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_theme_file_is_rejected_gracefully() {
        // A hostile/huge theme file must never be read into memory whole — it falls back to default
        // with a DL1790 warning, no panic, no OOM.
        let dir = std::env::temp_dir().join(format!("delulu_theme_huge_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.toml");
        let big = "x = 1\n".repeat(20_000); // ~120 KiB, well over THEME_FILE_MAX
        std::fs::write(&path, &big).unwrap();
        let (theme, warn) = resolve_theme(None, None, Some(&path));
        assert_eq!(theme.name(), "default", "oversized file falls back to default");
        assert!(warn.is_some(), "oversized file warns (DL1790)");
        // Bounded read: we never materialize the whole file.
        assert!(read_capped(&path, THEME_FILE_MAX).is_err(), "read is refused past the cap");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn theme_precedence_flag_over_env_over_file() {
        let dir = std::env::temp_dir().join(format!("delulu_theme_prec_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.toml");
        std::fs::write(&path, "theme = \"mono\"\n").unwrap();
        // Flag wins over env wins over file.
        assert_eq!(resolve_theme(Some("bright"), Some("mono"), Some(&path)).0.name(), "bright");
        assert_eq!(resolve_theme(None, Some("bright"), Some(&path)).0.name(), "bright");
        assert_eq!(resolve_theme(None, None, Some(&path)).0.name(), "mono");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
