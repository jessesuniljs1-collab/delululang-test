//! Locale selection + the first-run experience (Stage 8, spec §6.2–§6.3).
//!
//! Selection priority: `--locale` > `DELULU_LOCALE` > `~/.delulu/config.toml` > first-run
//! picker (interactive TTY only) > `en-US`. A locale changes HUMAN prose only — the machine
//! envelope never sees a catalog (invariant 39; build-order deviation 8).
//!
//! **Invariant 40 — agents run cold, zero prompts.** Any one of `--json`, the `CI` env var,
//! a non-interactive stdio, or `DELULU_NO_FIRST_RUN` suppresses the picker AND the welcome,
//! independently. The hidden foreign-worker subcommand is machine-only by construction.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::OnceLock;

use delulu_diag::Catalog;

/// The first-run welcome (spec §6.3). **This text is law**: byte-exact, never translated,
/// never paraphrased, never overridden by any catalog (an attempt is DL1704), identical in
/// both locales, and never present in any machine output, log, trace, LSP payload, or CI
/// stream. Do not edit it — not the spelling, not the emoji, not one byte.
pub const WELCOME_TEXT: &str = "U r here becoz u maybe a delulu like me & wanna create something others think is not possible. There's nothing wrong with being delulu. Anyway, u can't decide what others think abt u. So start building with everything u've got. Welcome to the Delulu Gang\u{1F426}\u{200D}\u{1F525}\u{1F525}\u{1FAE1}\u{1F680}";

/// The attribution line beneath the welcome, exactly (spec §6.3).
pub const WELCOME_ATTRIBUTION: &str = "— Jesse, The Creator of DeluluLang";

/// The two picker lines (spec §6.2) — fixed spec text, shown before any locale exists, so
/// they render in en-US always.
const PICKER_TITLE: &str =
    "pick your compiler's vibe (changes human text only — codes & JSON never change):";
const PICKER_OPTIONS: &str = "  1) English (US)   2) Delulu Slang        [1]: ";

static LOCALE: OnceLock<String> = OnceLock::new();
static INSTALLED: OnceLock<Catalog> = OnceLock::new();

/// The catalog for the active locale — `None` is en-US (the in-code prose; build-order
/// deviation 7). Every human-rendering site consults this; no machine channel does.
pub fn active_catalog() -> Option<&'static Catalog> {
    match LOCALE.get().map(String::as_str) {
        Some("delulu-slang") => Some(Catalog::delulu_slang()),
        Some(_) => INSTALLED.get(),
        _ => None,
    }
}

/// Where installed catalog files live: `$DELULU_LOCALES_DIR` (tests), else
/// `~/.delulu/locales/`. Installed via `delulu locale add` (a verified-class,
/// zero-authority catalog plugin — Stage 6's machinery, phase 8f's dogfood).
pub fn locales_dir() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("DELULU_LOCALES_DIR") {
        return Some(PathBuf::from(explicit));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".delulu").join("locales"))
}

/// Load an installed catalog by locale name (bounded read; a defective file simply
/// yields no catalog — the DL1704 warnings were shown at `locale add` time).
pub fn installed_catalog(name: &str) -> Option<Catalog> {
    // A locale name is a bare identifier, never a path — allowlist, don't blocklist.
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    let path = locales_dir()?.join(format!("{name}.toml"));
    let text = std::fs::read_to_string(path).ok()?;
    if text.len() > 256 * 1024 {
        return None;
    }
    let (cat, _warnings) = Catalog::parse(&text);
    (!cat.is_empty()).then_some(cat)
}

/// The config file path: `$DELULU_CONFIG_FILE` if set (tests and power users), else
/// `~/.delulu/config.toml` (HOME, then USERPROFILE — the house home resolution).
fn config_path() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("DELULU_CONFIG_FILE") {
        return Some(PathBuf::from(explicit));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".delulu").join("config.toml"))
}

/// Read `locale` and `welcomed` from the config file (missing/unreadable = fresh home).
/// The reader is the same hardened line-based shape as the theme file's.
fn read_config() -> (Option<String>, bool) {
    let Some(path) = config_path() else { return (None, false) };
    let Ok(text) = std::fs::read_to_string(&path) else { return (None, false) };
    if text.len() > 64 * 1024 {
        return (None, false); // an oversized config is no config
    }
    let mut locale = None;
    let mut welcomed = false;
    for raw in text.lines() {
        let line = match raw.find('#') {
            Some(i) => &raw[..i],
            None => raw,
        }
        .trim();
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim();
        let v = v.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(v);
        match k.trim() {
            "locale" => locale = Some(v.to_string()),
            "welcomed" => welcomed = v == "true",
            _ => {}
        }
    }
    (locale, welcomed)
}

/// Update `locale` (when `Some`) and `welcomed` in the config file, preserving every other
/// line. Best-effort: a home we cannot write to must never break the command that ran.
fn write_config(locale: Option<&str>, welcomed: bool) {
    let Some(path) = config_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| {
            let key = l.split('=').next().unwrap_or("").trim();
            key != "welcomed" && (locale.is_none() || key != "locale")
        })
        .map(str::to_string)
        .collect();
    if let Some(loc) = locale {
        lines.push(format!("locale = \"{loc}\""));
    }
    if welcomed {
        lines.push("welcomed = true".to_string());
    }
    let _ = std::fs::write(&path, lines.join("\n") + "\n");
}

/// Is this an interactive human session? `DELULU_ASSUME_TTY` (a test-only override, the
/// `DELULU_THEME_FILE` precedent) forces the answer; otherwise BOTH stdout and stdin must
/// be terminals — the picker prompts on one and reads the other.
pub(crate) fn interactive_tty() -> bool {
    match std::env::var("DELULU_ASSUME_TTY").ok().as_deref() {
        Some("1") => true,
        Some("0") => false,
        _ => std::io::stdout().is_terminal() && std::io::stdin().is_terminal(),
    }
}

/// Initialize the locale surface: strip `--locale`, run the first-run flow when (and only
/// when) it may run, resolve the locale, and pin it for `active_catalog`. Returns the
/// cleaned args and any warning lines to print on stderr (human channel only).
pub fn init_locale(args: &[String]) -> (Vec<String>, Vec<String>) {
    let mut flag: Option<String> = None;
    let mut cleaned: Vec<String> = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--locale" {
            if i + 1 < args.len() {
                flag = Some(args[i + 1].clone());
                i += 1;
            }
        } else if let Some(v) = a.strip_prefix("--locale=") {
            flag = Some(v.to_string());
        } else {
            cleaned.push(args[i].clone());
        }
        i += 1;
    }
    let mut warnings = Vec::new();

    // Invariant 40: each of these channels independently suppresses the picker + welcome.
    let json = cleaned.iter().any(|a| a == "--json");
    let machine_cmd = cleaned
        .first()
        .is_some_and(|c| c == crate::foreign_worker::WORKER_SUBCOMMAND || c == "lsp");
    let ci = std::env::var_os("CI").is_some();
    let no_first_run = std::env::var_os("DELULU_NO_FIRST_RUN").is_some();
    let interactive = interactive_tty();

    let (config_locale, welcomed) = read_config();
    let mut picked: Option<String> = None;

    if !json && !machine_cmd && !ci && !no_first_run && interactive && !welcomed {
        // The picker runs only when nothing above it in the priority chain decided already.
        let locale_pinned = flag.is_some()
            || std::env::var("DELULU_LOCALE").is_ok()
            || config_locale.is_some();
        if !locale_pinned {
            picked = Some(run_picker());
        }
        show_welcome();
        write_config(picked.as_deref(), true);
    }

    // Priority: --locale > DELULU_LOCALE > config > picker choice > en-US.
    let chosen = flag
        .or_else(|| std::env::var("DELULU_LOCALE").ok())
        .or(config_locale)
        .or(picked)
        .unwrap_or_else(|| "en-US".to_string());
    let known = matches!(chosen.as_str(), "en-US" | "delulu-slang")
        || match installed_catalog(&chosen) {
            Some(cat) => {
                let _ = INSTALLED.set(cat);
                true
            }
            None => false,
        };
    let effective = if known {
        chosen
    } else {
        warnings.push(format!(
            "warning[DL1704]: unknown locale `{chosen}` — using `en-US` (built-ins: en-US, delulu-slang; installed: `delulu locale list`)"
        ));
        "en-US".to_string()
    };
    let _ = LOCALE.set(effective);
    (cleaned, warnings)
}

/// The two-line picker (spec §6.2). Reads one stdin line; `2` picks Delulu Slang, anything
/// else (including just Enter) keeps the default. Never loops, never insists.
fn run_picker() -> String {
    use std::io::Write as _;
    println!("{PICKER_TITLE}");
    print!("{PICKER_OPTIONS}");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    if line.trim() == "2" { "delulu-slang".to_string() } else { "en-US".to_string() }
}

/// The welcome, in a plain box, both locales identically (spec §6.3). Stdout is the human's
/// terminal here by construction — every machine channel was excluded before this runs.
fn show_welcome() {
    println!();
    println!("  ┌──────────────────────────────────────────────────────────────┐");
    for line in wrap_welcome(WELCOME_TEXT, 60) {
        println!("  │ {line:<60} │");
    }
    println!("  │ {:<60} │", "");
    println!("  │ {:>60} │", WELCOME_ATTRIBUTION);
    println!("  └──────────────────────────────────────────────────────────────┘");
    println!();
}

/// Greedy word-wrap by char count (the box is padded by chars; emoji width wobble is
/// cosmetic and accepted — the TEXT is what is law, not the frame).
fn wrap_welcome(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split(' ') {
        let cur_len = cur.chars().count();
        let add = word.chars().count() + if cur.is_empty() { 0 } else { 1 };
        if cur_len + add > width && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The welcome text is LAW — byte-exact, hash-pinned (criterion 6). If this test ever
    /// fails, the text was altered; revert the text, never the pin.
    #[test]
    fn the_welcome_text_is_byte_exact_and_pinned() {
        let expected = concat!(
            "U r here becoz u maybe a delulu like me & wanna create something others think ",
            "is not possible. There's nothing wrong with being delulu. Anyway, u can't ",
            "decide what others think abt u. So start building with everything u've got. ",
            "Welcome to the Delulu Gang",
            "\u{1F426}\u{200D}\u{1F525}", // 🐦‍🔥 (bird + ZWJ + fire — one glyph, three scalars)
            "\u{1F525}\u{1FAE1}\u{1F680}", // 🔥🫡🚀
        );
        assert_eq!(WELCOME_TEXT, expected, "the welcome text is law — revert, never edit");
        assert_eq!(WELCOME_ATTRIBUTION, "— Jesse, The Creator of DeluluLang");
        // The pin: byte length + a position-weighted checksum, so no edit can hide.
        assert_eq!(WELCOME_TEXT.len(), 277, "welcome byte length moved");
        let sum: u64 = WELCOME_TEXT.bytes().enumerate().map(|(i, b)| (i as u64 + 1) * b as u64).sum();
        assert_eq!(sum, 4_043_712, "welcome checksum moved — the text is law");
    }

    #[test]
    fn wrap_never_loses_a_word() {
        let joined = wrap_welcome(WELCOME_TEXT, 60).join(" ");
        assert_eq!(joined, WELCOME_TEXT);
    }
}
