//! Stage 8, phase 8f: catalog plugins + `delulu locale add` (criterion 8) — the plugin
//! machinery's first zero-authority dogfood, driven end to end through the real binary:
//! build a verified-class catalog plugin, install it, render with it, fall back on its
//! gaps, remove it.

use std::path::PathBuf;
use std::process::{Command, Output};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_locplug_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn delulu(dir: &PathBuf, locales: &PathBuf, args: &[&str], locale_env: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
    c.current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_LOCALES_DIR", locales)
        .env_remove("DELULU_LOCALE");
    if let Some(l) = locale_env {
        c.env("DELULU_LOCALE", l);
    }
    c.output().expect("run delulu")
}

/// The pirate catalog: DL0501 covered (with the welcome-override attack riding along to
/// witness its DL1704-by-name refusal at add time), everything else deliberately absent.
const PIRATE_TOML: &str = "[meta]\nlocale = \"pirate\"\nversion = \"0.1.0\"\nfallback = \"en-US\"\ncoverage = \"DL0501 only — a demo\"\n\n[DL0501]\nmessage = \"arr, {fn} be doin `{effect}` without declarin it in the row, ye scallywag\"\n\n[cli.first-run.welcome]\nmessage = \"no welcome override for ye\"\n";

/// Escape TOML text into a delulu string literal.
fn delulu_str_lit(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Scaffold a catalog-plugin package and build its .dpx. Returns the artifact path.
fn build_catalog_plugin(work: &PathBuf, toml_text: &str, effects: &str) -> PathBuf {
    let pkg = work.join("pirate_pack");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        format!(
            "[package]\nname = \"pirate-cat\"\nversion = \"0.1.0\"\nkind = \"plugin\"\n\n\
             [plugin]\napi = 1\nclass = \"verified\"\n\n\
             [plugin.authority]\neffects  = [{effects}]\nrequires = []\n\n\
             [plugin.exports]\ncatalog = \"fn() -> Str\"\n"
        ),
    )
    .unwrap();
    std::fs::write(
        pkg.join("src").join("cat.delulu"),
        format!("module cat\n\npub fn catalog() -> Str {{ {} }}\n", delulu_str_lit(toml_text)),
    )
    .unwrap();
    let out = work.join("pirate.dpx");
    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(work)
        .args(["plugin", "build", pkg.to_str().unwrap(), "-o", out.to_str().unwrap()])
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "plugin build: {}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    out
}

const DL0501_SRC: &str = "module m\nfn greet(out: Cap[Console], n: Str) { out.println(n) }\n";
const DL0605_SRC: &str = "module m\nfn f(a: Secret[Str], b: Secret[Str]) -> Bool { a == b }\n";

#[test]
fn criterion8_third_locale_installs_renders_and_falls_back() {
    let work = tmp("crit8");
    let locales = work.join("locales");
    let dpx = build_catalog_plugin(&work, PIRATE_TOML, "");

    // Install (non-TTY: no prompt, invariant 40's spirit) — the welcome-override entry
    // inside the catalog is refused BY NAME at add time, and the install proceeds on the
    // valid remainder.
    let o = delulu(&work, &locales, &["locale", "add", dpx.to_str().unwrap()], None);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(o.status.success(), "locale add: {err}");
    assert!(err.contains("DL1704") && err.contains("welcome is law"), "criterion 8's override half: {err}");
    assert!(err.contains("HUMAN prose only"), "the prose bound is stated: {err}");
    assert!(locales.join("pirate.toml").exists());

    // The third locale RENDERS a covered code…
    let prog = work.join("m.delulu");
    std::fs::write(&prog, DL0501_SRC).unwrap();
    let o2 = delulu(&work, &locales, &["check", prog.to_str().unwrap()], Some("pirate"));
    let err2 = String::from_utf8_lossy(&o2.stderr);
    assert!(err2.contains("ye scallywag"), "pirate DL0501 renders: {err2}");
    assert!(err2.contains("function `greet`"), "the {{fn}} arg fills: {err2}");

    // …and FALLS BACK to en-US on its gaps (DL0605 is not in the pirate set).
    let prog2 = work.join("s.delulu");
    std::fs::write(&prog2, DL0605_SRC).unwrap();
    let o3 = delulu(&work, &locales, &["check", prog2.to_str().unwrap()], Some("pirate"));
    let err3 = String::from_utf8_lossy(&o3.stderr);
    assert!(err3.contains("no structural equality"), "en-US fallback: {err3}");

    // The machine envelope never notices any of this.
    let oj = delulu(&work, &locales, &["check", prog.to_str().unwrap(), "--json"], Some("pirate"));
    let oj2 = delulu(&work, &locales, &["check", prog.to_str().unwrap(), "--json"], None);
    assert_eq!(oj.stdout, oj2.stdout, "json is locale-invariant, installed locales included");

    // list shows it; remove unwires it.
    let ol = delulu(&work, &locales, &["locale", "list"], None);
    assert!(String::from_utf8_lossy(&ol.stdout).contains("pirate"), "listed");
    let orm = delulu(&work, &locales, &["locale", "remove", "pirate"], None);
    assert!(orm.status.success());
    assert!(!locales.join("pirate.toml").exists());
    let o4 = delulu(&work, &locales, &["check", prog.to_str().unwrap()], Some("pirate"));
    let err4 = String::from_utf8_lossy(&o4.stderr);
    assert!(err4.contains("DL1704") && err4.contains("unknown locale"), "{err4}");
}

/// The kitchen-rule case: a plugin whose ceiling declares ANY effect is refused at
/// `locale add` — a catalog plugin is bounded to prose by rule, and the rule holds even
/// though `plugin build` itself is perfectly happy with an effectful plugin.
#[test]
fn a_catalog_plugin_with_authority_is_refused() {
    let work = tmp("authz");
    let locales = work.join("locales");
    let dpx = build_catalog_plugin(&work, PIRATE_TOML, "\"Read\"");
    let o = delulu(&work, &locales, &["locale", "add", dpx.to_str().unwrap()], None);
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("ZERO authority"), "{err}");
    assert!(!locales.join("pirate.toml").exists(), "nothing installed");
}
