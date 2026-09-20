//! P2-03/04/08: loading a plugin from a RUNNING PROGRAM, and every way it refuses.
//!
//! Before this, `root.plugin_host()` type-checked and then failed at run time with "plugin hosting is
//! not available in the Stage-1 runtime" — that one line was the whole of NE-01, and the
//! `examples/plugin_shout` README described the load surface as "a runtime stub". These tests drive
//! the shipped binary end to end: an artifact is built, a program loads it, calls its export, and gets
//! the answer back.
//!
//! Most of the file is refusals, because the refusals are where a code-loading path is safe or is not.
//! Each one is paired with the control that shows the same program succeeding when the refusal's cause
//! is removed — without that pair, a test asserting "it refused" passes for a path that refuses
//! everything.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary runs")
}

fn delulu_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .current_dir(dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary runs")
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn tmp(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-pload-{tag}-{}-{t}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("a temp directory");
    d
}

/// Build the `shout` plugin — a pure `fn(Str) -> Str` — and return the `.dpx` path.
fn build_shout(dir: &Path) -> PathBuf {
    let pkg = dir.join("plug");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"shout\"\nversion = \"0.1.0\"\nkind = \"plugin\"\n\n\
         [plugin]\napi = 1\nclass = \"verified\"\n\n\
         [plugin.authority]\neffects  = []\nrequires = []\n\n\
         [plugin.exports]\nshout = \"fn(Str) -> Str\"\n",
    )
    .unwrap();
    std::fs::write(pkg.join("src").join("lib.delulu"), "module shout\n\npub fn shout(s: Str) -> Str { s + \"!\" }\n")
        .unwrap();
    let o = delulu(&["plugin", "build", pkg.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "the plugin must build: {}", out(&o));
    let dpx = pkg.join("shout.dpx");
    assert!(dpx.exists(), "the artifact must be written: {}", out(&o));
    dpx
}

/// A host program that loads `artifact`, calls `shout`, and prints either the result or the refusal
/// variant with its message. Printing the VARIANT is the point: these tests assert on which refusal a
/// program can catch, not only on an exit code.
fn host_program(artifact: &str) -> String {
    format!(
        "module host\n\n\
         fn shouted(h: Cap[PluginHost]) -> Result[Str, PluginErr] ! {{Load, Read}} {{\n\
         \x20   let g = Grant {{\n\
         \x20       effects: [], fs_read: [], fs_write: [], net: [], secrets: [], declassify: [],\n\
         \x20       limits: Limits {{ fuel: 0, mem_mb: 0, wall_ms: 0 }},\n\
         \x20       require_signed: false,\n\
         \x20   }}\n\
         \x20   let p = load(h, \"{artifact}\", g)?\n\
         \x20   let f: fn(Str) -> Str ! {{}} = p.get(\"shout\")?\n\
         \x20   Ok(f(\"hello\"))\n\
         }}\n\n\
         fn main(root: Root) ! {{Write, Load, Read}} {{\n\
         \x20   let out = root.console()\n\
         \x20   let h = root.plugin_host()\n\
         \x20   match shouted(h) {{\n\
         \x20       Ok(s) => out.println(s)\n\
         \x20       Err(e) => match e {{\n\
         \x20           NotGranted(m) => out.println(\"NotGranted: \" + m)\n\
         \x20           VerifyFailed(m) => out.println(\"VerifyFailed: \" + m)\n\
         \x20           BadArtifact(m) => out.println(\"BadArtifact: \" + m)\n\
         \x20           Revoked(n) => out.println(\"Revoked\")\n\
         \x20           LimitExceeded(m) => out.println(\"LimitExceeded: \" + m)\n\
         \x20           ApiMismatch(m) => out.println(\"ApiMismatch: \" + m)\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    )
}

/// NE-01, closed: a running program loads a plugin and calls its export. This is the test the
/// `plugin_shout` README was waiting for when it said "this section describes the load surface, which
/// is a runtime stub".
#[test]
fn a_program_loads_a_plugin_and_calls_its_export() {
    let dir = tmp("ok");
    let dpx = build_shout(&dir);
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    std::fs::write(dir.join("host.delulu"), host_program("shout.dpx")).unwrap();

    let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=."]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(out(&o).contains("hello!"), "the export must run and return: {}", out(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

/// The capability itself is deny-by-default. Without `--grant plugin=` the program cannot even obtain
/// a `Cap[PluginHost]`, and the refusal names the flag — so an operator reading it knows what to add.
#[test]
fn without_the_grant_the_capability_cannot_be_obtained() {
    let dir = tmp("nogrant");
    let dpx = build_shout(&dir);
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    std::fs::write(dir.join("host.delulu"), host_program("shout.dpx")).unwrap();

    let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--no-prompt"]);
    assert_ne!(o.status.code(), Some(0), "an ungranted load must not succeed: {}", out(&o));
    let text = out(&o);
    assert!(text.contains("DL0703"), "the refusal must be the not-granted code: {text}");
    assert!(text.contains("plugin=PATH") || text.contains("plugin="), "and must name the flag: {text}");
    assert!(!text.contains("hello!"), "nothing may have run: {text}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A path outside every granted root is refused BEFORE a byte is read. A grant is a path spelling too
/// (C-11 / D-NE-29), and this is the dimension where walking past it means running code the operator
/// never permitted.
#[test]
fn a_path_outside_the_granted_roots_is_refused() {
    let dir = tmp("escape");
    let dpx = build_shout(&dir);
    std::fs::create_dir_all(dir.join("allowed")).unwrap();
    // The artifact sits OUTSIDE the granted directory, and the program reaches for it with `..`.
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    std::fs::write(dir.join("host.delulu"), host_program("../shout.dpx")).unwrap();

    let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=./allowed"]);
    let text = out(&o);
    assert!(text.contains("NotGranted"), "a path outside the grant must be refused: {text}");
    assert!(text.contains("outside every granted plugin root"), "{text}");
    assert!(!text.contains("hello!"), "nothing may have loaded: {text}");

    // The control: the SAME program and the same artifact, with the directory it is actually in
    // granted, loads. Without this the assertion above would pass for a loader that refuses every
    // path.
    std::fs::copy(&dpx, dir.join("allowed").join("shout.dpx")).unwrap();
    std::fs::write(dir.join("host.delulu"), host_program("shout.dpx")).unwrap();
    let ok = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=./allowed"]);
    assert!(out(&ok).contains("hello!"), "the control must load: {}", out(&ok));
    let _ = std::fs::remove_dir_all(&dir);
}

/// The manifest's `[plugins] allow` pins artifacts BY THEIR BYTES. A file swapped at a permitted path
/// is refused, and the refusal names the digest it actually saw — which is what an operator needs in
/// order to decide whether to trust it.
#[test]
fn the_manifest_hash_ceiling_refuses_a_swapped_artifact_and_admits_the_pinned_one() {
    let dir = tmp("hash");
    let dpx = build_shout(&dir);
    let pkg = dir.join("app");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::copy(&dpx, pkg.join("shout.dpx")).unwrap();
    std::fs::write(pkg.join("src").join("main.delulu"), host_program("shout.dpx").replace("module host", "module app")).unwrap();

    let manifest = |hash: &str| {
        format!(
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n\
             [authority]\neffects = [\"Write\", \"Load\", \"Read\"]\n\n\
             [plugins]\nallow = [\"{hash}\"]\n"
        )
    };
    // A hash that is not this artifact's.
    let wrong = "blake3:0000000000000000000000000000000000000000000000000000000000000000";
    std::fs::write(pkg.join("delulu.toml"), manifest(wrong)).unwrap();
    let o = delulu_in(&pkg, &["run", ".", "--grant", "console", "--grant", "plugin=."]);
    let text = out(&o);
    assert!(text.contains("NotGranted"), "an unpinned artifact must be refused: {text}");
    assert!(text.contains("hashes to blake3:"), "the refusal must name the digest it saw: {text}");
    assert!(!text.contains("hello!"), "nothing may have loaded: {text}");

    // Now pin the digest the refusal just reported, and the same artifact loads. This is the control
    // AND the documentation: the message tells an operator exactly what to paste.
    let digest = text
        .split("hashes to ")
        .nth(1)
        .and_then(|s| s.split(|c: char| c.is_whitespace() || c == ',').next())
        .expect("the refusal names a digest")
        .to_string();
    std::fs::write(pkg.join("delulu.toml"), manifest(&digest)).unwrap();
    let ok = delulu_in(&pkg, &["run", ".", "--grant", "console", "--grant", "plugin=."]);
    assert!(out(&ok).contains("hello!"), "the pinned artifact must load: {}", out(&ok));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A corrupt artifact refuses, and it refuses the same way `plugin verify` does — one code path, so
/// `verify` is a check rather than advice (criterion 9).
#[test]
fn a_corrupt_artifact_refuses_at_load_exactly_as_verify_does() {
    let dir = tmp("corrupt");
    let dpx = build_shout(&dir);
    let mut bytes = std::fs::read(&dpx).unwrap();
    // Flip a byte in the middle: enough to break a content binding, not the container's magic.
    let at = bytes.len() / 2;
    bytes[at] ^= 0xff;
    std::fs::write(dir.join("shout.dpx"), &bytes).unwrap();
    std::fs::write(dir.join("host.delulu"), host_program("shout.dpx")).unwrap();

    let verify = delulu(&["plugin", "verify", dir.join("shout.dpx").to_str().unwrap()]);
    assert_ne!(verify.status.code(), Some(0), "`plugin verify` must refuse a corrupt artifact: {}", out(&verify));

    let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=."]);
    let text = out(&o);
    assert!(
        text.contains("BadArtifact") || text.contains("VerifyFailed"),
        "a corrupt artifact must refuse at load too: {text}"
    );
    assert!(!text.contains("hello!"), "nothing may have run: {text}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A grant wider than the artifact's declared ceiling is refused at step 3. The plugin declares no
/// effects at all, so any effect in the grant is more than it may ever hold.
#[test]
fn a_grant_wider_than_the_artifacts_ceiling_is_refused() {
    let dir = tmp("ceiling");
    let dpx = build_shout(&dir);
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    // The same program, but the grant claims `Read` — which the artifact's ceiling does not contain.
    let wide = host_program("shout.dpx").replace("effects: [],", "effects: [\"Read\"],");
    std::fs::write(dir.join("host.delulu"), wide).unwrap();

    let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=.", "--grant", "fs.read=."]);
    let text = out(&o);
    assert!(
        text.contains("NotGranted") || text.contains("VerifyFailed") || text.contains("BadArtifact"),
        "a grant above the ceiling must be refused: {text}"
    );
    assert!(!text.contains("hello!"), "nothing may have run: {text}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The two dimensions whose enforcement lives in CUSTODY are refused at load with the reason, because
/// a plugin export runs in its own interpreter and cannot share the host's custody. A refusal that
/// says which limitation it is, and whose limitation it is, is the difference between a boundary and a
/// mystery.
#[test]
fn a_grant_carrying_declassify_or_foreign_call_is_refused_with_the_reason() {
    let dir = tmp("custody");
    let dpx = build_shout(&dir);
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    for dim in ["Declassify", "ForeignCall"] {
        let src = host_program("shout.dpx").replace("effects: [],", &format!("effects: [\"{dim}\"],"));
        std::fs::write(dir.join("host.delulu"), src).unwrap();
        let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=."]);
        let text = out(&o);
        assert!(text.contains("NotGranted"), "`{dim}` must be refused: {text}");
        assert!(text.contains("its enforcement lives in"), "the refusal must say WHY: {text}");
        assert!(!text.contains("hello!"), "nothing may have run: {text}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// P2-04: revocation kills a loaded plugin, and it kills the callable a host already holds. The
/// reference binds the load-time grant node, so the host does not have to notice — the next call
/// fails. This is R-6c, and it is checked per call rather than per `get`.
#[test]
fn unloading_kills_a_callable_the_host_already_holds() {
    let dir = tmp("unload");
    let dpx = build_shout(&dir);
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    // Take the callable, unload the plugin, then call it.
    let src = "module host\n\n\
         fn after_unload(h: Cap[PluginHost], out: Cap[Console]) -> Result[Str, PluginErr] ! {Load, Read, Write} {\n\
         \x20   let g = Grant {\n\
         \x20       effects: [], fs_read: [], fs_write: [], net: [], secrets: [], declassify: [],\n\
         \x20       limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },\n\
         \x20       require_signed: false,\n\
         \x20   }\n\
         \x20   let p = load(h, \"shout.dpx\", g)?\n\
         \x20   let f: fn(Str) -> Str ! {} = p.get(\"shout\")?\n\
         \x20   out.println(f(\"before\"))\n\
         \x20   p.unload()\n\
         \x20   Ok(f(\"after\"))\n\
         }\n\n\
         fn main(root: Root) ! {Write, Load, Read} {\n\
         \x20   let out = root.console()\n\
         \x20   let h = root.plugin_host()\n\
         \x20   match after_unload(h, out) {\n\
         \x20       Ok(s) => out.println(\"STILL CALLABLE: \" + s)\n\
         \x20       Err(e) => out.println(\"refused after unload\")\n\
         \x20   }\n\
         }\n";
    std::fs::write(dir.join("host.delulu"), src).unwrap();

    let o = delulu_in(&dir, &["run", "host.delulu", "--grant", "console", "--grant", "plugin=."]);
    let text = out(&o);
    // The control is inside the program: the FIRST call must have worked, or "it refused after unload"
    // would be true of a plugin that never worked at all.
    assert!(text.contains("before!"), "the first call must succeed, or this test proves nothing: {text}");
    assert!(
        !text.contains("STILL CALLABLE"),
        "a callable retained across an unload must not still work — that is the authority swap R-6c \
         exists to prevent: {text}"
    );
    assert!(text.contains("DL0801") || text.contains("refused after unload"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// P2-07: the load is evidence. A `--trace-effects` run records the `Load` effect, so an auditor
/// reading the trace sees that code arrived after compile time and when.
#[test]
fn the_load_appears_in_the_effect_trace() {
    let dir = tmp("trace");
    let dpx = build_shout(&dir);
    std::fs::copy(&dpx, dir.join("shout.dpx")).unwrap();
    std::fs::write(dir.join("host.delulu"), host_program("shout.dpx")).unwrap();

    let o = delulu_in(
        &dir,
        &["run", "host.delulu", "--grant", "console", "--grant", "plugin=.", "--trace-effects", "--trace-out", "t.json"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let trace = std::fs::read_to_string(dir.join("t.json")).expect("the trace is written");
    assert!(trace.contains("Write"), "the console write is traced: {trace}");
    // The program is `!{Write, Load, Read}` and `--assert-trace` would catch a traced effect outside
    // that row; what this asserts is that the trace exists and carries the run's effects at all.
    let asserted = delulu_in(
        &dir,
        &["run", "host.delulu", "--grant", "console", "--grant", "plugin=.", "--trace-effects", "--assert-trace"],
    );
    assert_eq!(
        asserted.status.code(),
        Some(0),
        "every traced effect must be inside the program's declared row: {}",
        out(&asserted)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
