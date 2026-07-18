//! Stage 8, phase 8c: locale selection, the first-run picker, and the welcome — driven
//! through the REAL binary (criteria 6 and 7).
//!
//! TTY note: test processes capture output, so stdio is never a real terminal here. The
//! hidden `DELULU_ASSUME_TTY` override (the `DELULU_THEME_FILE` precedent) forces the
//! interactive answer both ways, which is what lets the SHOWN cases run headless while the
//! non-TTY suppression case runs with no override — the real channel, really tested.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// The welcome's first words — enough to detect it anywhere without repeating the law here
/// (the byte-exact pin lives in `locale.rs`'s unit test).
const WELCOME_HEAD: &str = "U r here becoz u maybe a delulu";
const PICKER_HEAD: &str = "pick your compiler's vibe";

/// A DL0501 program: the criterion-7 rendering subject.
const DL0501_SRC: &str = "module m\nfn greet(out: Cap[Console], n: Str) { out.println(n) }\n";

struct Ctx {
    dir: PathBuf,
    config: PathBuf,
    program: PathBuf,
}

impl Ctx {
    fn fresh(tag: &str) -> Ctx {
        let dir = std::env::temp_dir().join(format!("delulu_locale_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let program = dir.join("m.delulu");
        std::fs::write(&program, DL0501_SRC).unwrap();
        Ctx { config: dir.join("config.toml"), dir, program }
    }

    /// A hermetic command: every locale-relevant env var is pinned or removed.
    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
        c.current_dir(workspace_root())
            .args(args)
            .env("DELULU_CONFIG_FILE", &self.config)
            .env_remove("DELULU_LOCALE")
            .env_remove("DELULU_NO_FIRST_RUN")
            .env_remove("DELULU_ASSUME_TTY")
            .env_remove("CI");
        c
    }

    fn run(&self, args: &[&str], tty: bool, extra: &[(&str, &str)], stdin: Option<&str>) -> Output {
        let mut c = self.cmd(args);
        if tty {
            c.env("DELULU_ASSUME_TTY", "1");
        }
        for (k, v) in extra {
            c.env(k, v);
        }
        if let Some(input) = stdin {
            use std::io::Write as _;
            c.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
            let mut child = c.spawn().expect("spawn delulu");
            child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
            child.wait_with_output().expect("run delulu")
        } else {
            c.stdin(Stdio::null());
            c.output().expect("run delulu")
        }
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn criterion6_welcome_and_picker_show_exactly_once_on_a_fresh_tty_home() {
    let ctx = Ctx::fresh("once");
    let file = ctx.program.to_str().unwrap().to_string();

    // First interactive run: picker + welcome, and the picked locale takes effect.
    let o = ctx.run(&["check", &file], true, &[], Some("2\n"));
    let t = text(&o);
    assert!(t.contains(PICKER_HEAD), "picker shown on a fresh home: {t}");
    assert!(t.contains(WELCOME_HEAD), "welcome shown once: {t}");
    assert!(t.contains("— Jesse, The Creator of DeluluLang"), "attribution line: {t}");
    assert!(t.contains("no cap 💀"), "picking 2 selects Delulu Slang immediately: {t}");

    // The config recorded both decisions…
    let cfg = std::fs::read_to_string(&ctx.config).unwrap();
    assert!(cfg.contains("locale = \"delulu-slang\""), "{cfg}");
    assert!(cfg.contains("welcomed = true"), "{cfg}");

    // …so the second interactive run shows NEITHER (criterion 6's fifth channel).
    let o2 = ctx.run(&["check", &file], true, &[], None);
    let t2 = text(&o2);
    assert!(!t2.contains(PICKER_HEAD), "no second picker: {t2}");
    assert!(!t2.contains(WELCOME_HEAD), "no second welcome: {t2}");
    assert!(t2.contains("no cap 💀"), "the picked locale persists: {t2}");
}

/// Criterion 6, channels 1–4: `--json`, `CI`, non-TTY stdio, `DELULU_NO_FIRST_RUN` — each
/// ALONE suppresses the picker and the welcome on a completely fresh home (invariant 40:
/// an agent can run `delulu` cold with zero interactive surprise).
#[test]
fn criterion6_each_machine_channel_alone_suppresses_the_first_run() {
    for (tag, args, tty, extra) in [
        ("json", vec!["check", "PROGRAM", "--json"], true, vec![]),
        ("ci", vec!["check", "PROGRAM"], true, vec![("CI", "1")]),
        ("notty", vec!["check", "PROGRAM"], false, vec![]),
        ("nofr", vec!["check", "PROGRAM"], true, vec![("DELULU_NO_FIRST_RUN", "1")]),
    ] {
        let ctx = Ctx::fresh(tag);
        let file = ctx.program.to_str().unwrap().to_string();
        let args: Vec<&str> =
            args.iter().map(|a| if *a == "PROGRAM" { file.as_str() } else { *a }).collect();
        let o = ctx.run(&args, tty, &extra, None);
        let t = text(&o);
        assert!(!t.contains(PICKER_HEAD), "channel `{tag}` must suppress the picker: {t}");
        assert!(!t.contains(WELCOME_HEAD), "channel `{tag}` must suppress the welcome: {t}");
        assert!(!ctx.config.exists(), "channel `{tag}` must not mint a config file");
    }
}

#[test]
fn locale_priority_flag_beats_env_beats_config() {
    let ctx = Ctx::fresh("prio");
    let file = ctx.program.to_str().unwrap().to_string();
    std::fs::write(&ctx.config, "locale = \"en-US\"\nwelcomed = true\n").unwrap();

    // env beats config:
    let o = ctx.run(&["check", &file], false, &[("DELULU_LOCALE", "delulu-slang")], None);
    assert!(text(&o).contains("no cap 💀"), "env wins over config: {}", text(&o));

    // flag beats env:
    let o2 = ctx.run(
        &["check", &file, "--locale", "en-US"],
        false,
        &[("DELULU_LOCALE", "delulu-slang")],
        None,
    );
    let t2 = text(&o2);
    assert!(t2.contains("performs effect"), "flag wins — en-US prose: {t2}");
    assert!(!t2.contains("no cap"), "{t2}");

    // config alone:
    std::fs::write(&ctx.config, "locale = \"delulu-slang\"\nwelcomed = true\n").unwrap();
    let o3 = ctx.run(&["check", &file], false, &[], None);
    assert!(text(&o3).contains("no cap 💀"), "config applies: {}", text(&o3));
}

/// Criterion 7 at the CLI level, strengthened per build-order deviation 8: the FULL
/// `--json` output is byte-identical across locales — message text included — while the
/// human stderr differs. The machine interface cannot be localized.
#[test]
fn criterion7_json_bytes_identical_across_locales_human_differs() {
    // Tag must be unique across ALL tests in this binary — Ctx dirs are keyed (tag, pid)
    // and tests run in parallel; a shared tag means two tests clobbering one directory.
    let ctx = Ctx::fresh("crit7");
    let file = ctx.program.to_str().unwrap().to_string();
    std::fs::write(&ctx.config, "welcomed = true\n").unwrap();

    let en = ctx.run(&["check", &file, "--json"], false, &[("DELULU_LOCALE", "en-US")], None);
    let slang =
        ctx.run(&["check", &file, "--json"], false, &[("DELULU_LOCALE", "delulu-slang")], None);
    assert_eq!(en.stdout, slang.stdout, "the machine envelope is locale-invariant, byte-for-byte");
    assert!(
        String::from_utf8_lossy(&en.stdout).contains("performs effect"),
        "and it speaks en-US"
    );

    let en_h = ctx.run(&["check", &file], false, &[("DELULU_LOCALE", "en-US")], None);
    let slang_h = ctx.run(&["check", &file], false, &[("DELULU_LOCALE", "delulu-slang")], None);
    assert_ne!(en_h.stderr, slang_h.stderr, "the human channel is where locale lives");
    assert!(String::from_utf8_lossy(&slang_h.stderr).contains("no cap 💀"));
}

#[test]
fn an_unknown_locale_warns_dl1704_and_falls_back_to_en_us() {
    let ctx = Ctx::fresh("unknown");
    let file = ctx.program.to_str().unwrap().to_string();
    std::fs::write(&ctx.config, "welcomed = true\n").unwrap();
    let o = ctx.run(&["check", &file], false, &[("DELULU_LOCALE", "klingon")], None);
    let t = text(&o);
    assert!(t.contains("DL1704") && t.contains("klingon"), "{t}");
    assert!(t.contains("performs effect"), "falls back to en-US: {t}");
}

/// The welcome never leaks into machine output: the `--json` stream of a fresh interactive
/// home parses as pure JSON and carries no welcome bytes (spec §6.3's "never appears in
/// any machine output" — the CI/log/trace/LSP channels are covered by suppression above).
#[test]
fn the_welcome_never_appears_in_json_output() {
    let ctx = Ctx::fresh("leak");
    let file = ctx.program.to_str().unwrap().to_string();
    let o = ctx.run(&["check", &file, "--json"], true, &[], None);
    let out = String::from_utf8_lossy(&o.stdout);
    let v: serde_json::Value = serde_json::from_str(&out).expect("pure JSON, no welcome bytes");
    assert_eq!(v["schema"], 1);
    assert!(!out.contains("Delulu Gang"), "{out}");
}
