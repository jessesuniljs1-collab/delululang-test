//! Every shipped example checks clean — and every one that can run, runs.
//!
//! The Book's samples already had the first half (`book.rs`, criterion 7), and its reasoning is
//! right: a tutorial whose examples do not compile teaches people something false. But checking
//! is only half of it. `examples/guide/04_effects.delulu` checked clean and then faulted at
//! runtime with "unbound name `double`", because passing a NAMED function as a value was
//! accepted by the checker and unimplemented in the interpreter (HARDENING_CAMPAIGN C13). The
//! reference sample for row polymorphism has that exact shape and nobody noticed, because
//! nothing ever ran it.
//!
//! So this file adds the other half. The running assertion is deliberately narrow: a program
//! that checks clean must never fail with **DL0907**, the code the runtime raises when the
//! checker let something through. Every other runtime outcome is legitimate — an example may
//! want a file that is not there, or a grant this test did not hand it — and the test says
//! nothing about those.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// Every `.delulu` file directly under `examples/` or `examples/guide/`. Package directories
/// (`greeter`, `plugin_shout`) are checked as packages by `tooling.rs` and are skipped here.
fn example_files() -> Vec<PathBuf> {
    let mut v = Vec::new();
    for dir in ["examples", "examples/guide"] {
        let d = root().join(dir);
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

#[test]
fn every_example_checks_clean() {
    let files = example_files();
    assert!(files.len() >= 7, "expected the example set, found {}", files.len());
    for f in &files {
        let o = delulu(&["check", &rel(f)]);
        assert!(
            o.status.success(),
            "{} does not check clean:\n{}",
            rel(f),
            text(&o)
        );
    }
}

#[test]
fn no_example_that_checks_clean_fails_with_a_checker_bug_code() {
    // A generous grant set: the point is to reach as much of each program as possible, not to
    // model least privilege. Examples that still refuse for want of a grant or a file are fine.
    let grants = [
        "--grant", "console",
        "--grant", "clock",
        "--grant", "rand",
        "--grant", "fs.read=.",
        "--grant", "fs.write=.",
    ];
    for f in &example_files() {
        let src = std::fs::read_to_string(f).expect("read example");
        if !src.contains("fn main(") {
            continue; // nothing to run — `check` already covered it
        }
        let path = rel(f);
        let mut args: Vec<&str> = vec!["run", &path, "--no-prompt"];
        args.extend_from_slice(&grants);
        let o = delulu(&args);
        let out = text(&o);
        assert!(
            !out.contains("DL0907"),
            "{path} checks clean but fails at runtime with DL0907 — the checker let something \
             through. This is a compiler bug, not the example's fault:\n{out}"
        );
    }
}

/// NE-03. The CI gate above asserts a guide example does not fail with a **compiler-bug** code. It
/// says nothing about whether the example does what the guide says it does — and
/// `examples/guide/05_capabilities.delulu`, the file `GETTING_STARTED.md` §6 is built from, passed
/// it for its whole life while reading nothing: it minted `root.fs_read("./config")` and then asked
/// for `"./config/app.txt"`, which resolves under the capability's own root to
/// `./config/config/app.txt`. It took the `Err` branch every time and printed "no config". Two
/// shipped examples disagreed — `examples/demo.delulu` had it right — and nothing stated the rule.
///
/// So this asserts the outcome, not the absence of a crash: with the files present and the grants
/// given, the guide's capability example **reads its file and writes its log**.
#[test]
fn the_capabilities_guide_actually_reads_and_writes() {
    let w = std::env::temp_dir().join(format!("delulu-guide-caps-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&w);
    std::fs::create_dir_all(w.join("config")).unwrap();
    std::fs::create_dir_all(w.join("out")).unwrap();
    std::fs::write(w.join("config").join("app.txt"), "hello-from-config").unwrap();
    let src = root().join("examples").join("guide").join("05_capabilities.delulu");
    std::fs::copy(&src, w.join("05_capabilities.delulu")).expect("copy the guide example");

    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&w)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args([
            "run",
            "05_capabilities.delulu",
            "--no-prompt",
            "--grant",
            "console",
            "--grant",
            "fs.read=./config",
            "--grant",
            "fs.write=./out",
            "--grant",
            "net=example.com",
        ])
        .output()
        .expect("run delulu");
    let out = text(&o);
    assert_eq!(o.status.code(), Some(0), "the example must run:\n{out}");
    assert!(
        out.contains("config: hello-from-config"),
        "the guide's READ must succeed and show the file's contents:\n{out}"
    );
    assert!(out.contains("wrote log"), "and its WRITE must succeed:\n{out}");
    assert!(
        w.join("out").join("log.txt").is_file(),
        "the file must be where the capability's scope puts it, not where the literal reads"
    );
    // The network branch is allowed to fail: a test must not depend on reaching a host. What it must
    // not do is fail for the same reason the read used to.
    assert!(
        !out.contains("no config") && !out.contains("could not write"),
        "neither filesystem branch may take its Err path:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// The falsification, standing: the wrong spelling must still fail, so the assertion above is
/// measuring the rule and not the weather. Written as its own program rather than by editing the
/// shipped one, because a test that mutates a repository file is a test that can leave it mutated.
#[test]
fn a_path_that_repeats_the_capability_scope_finds_nothing() {
    let w = std::env::temp_dir().join(format!("delulu-guide-caps-neg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&w);
    std::fs::create_dir_all(w.join("config")).unwrap();
    std::fs::write(w.join("config").join("app.txt"), "hello-from-config").unwrap();
    std::fs::write(
        w.join("wrong.delulu"),
        "module wrongpath\n\n\
         fn main(root: Root) ! {Read, Write} {\n\
         \x20   let out = root.console()\n\
         \x20   let reader = root.fs_read(\"./config\")\n\
         \x20   match reader.read_text(\"./config/app.txt\") {\n\
         \x20       Ok(t) => out.println(\"read: \" + t)\n\
         \x20       Err(e) => out.println(\"no config\")\n\
         \x20   }\n}\n",
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&w)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(["run", "wrong.delulu", "--no-prompt", "--grant", "console", "--grant", "fs.read=./config"])
        .output()
        .expect("run delulu");
    let out = text(&o);
    assert!(
        out.contains("no config"),
        "a path that repeats the capability's own scope must NOT find the file — otherwise the \
         assertion in the test above proves nothing:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// P2-06: the flagship plugin example RUNS now — build the `.dpx`, load it from `host.delulu`, and
/// assert the plugin's own output.
///
/// This gate is the one the example's README was missing. Until 2026-09-20 that README said "there is
/// no host program in this directory to run", because `root.plugin_host()` was a stub. A README that
/// describes a shape nothing executes is how documentation drifts from a product, so the shape is now a
/// program and the program is now a test.
///
/// Hermetic on purpose: the artifact is built into a temp directory rather than beside the package, so
/// running the suite never leaves a `.dpx` in the checkout. The Survey maps a clean checkout, and build
/// output in the tree is the 2026-09-14 lesson (a packaged `.vsix` was a node in the map until a fresh
/// clone counted one node fewer).
#[test]
fn the_plugin_example_builds_loads_and_prints_what_the_plugin_returns() {
    let work = std::env::temp_dir().join(format!("delulu-ex-plugin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).expect("a temp directory");
    let dpx = work.join("shout.dpx");

    let build = delulu(&["plugin", "build", "examples/plugin_shout", "-o", dpx.to_str().unwrap()]);
    assert!(build.status.success(), "the example plugin must build:
{}", text(&build));
    assert!(dpx.exists(), "the artifact must be written:
{}", text(&build));

    std::fs::copy(root().join("examples/plugin_shout/host.delulu"), work.join("host.delulu")).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&work)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(["run", "host.delulu", "--grant", "console", "--grant", "plugin=."])
        .output()
        .expect("run delulu");
    let got = text(&run);
    assert!(run.status.success(), "the host program must run:
{got}");
    // The PLUGIN's output, not the host's. `shout` appends "!", so this string can only come from code
    // that arrived at run time and was re-verified on the way in.
    assert!(got.contains("hello!"), "the plugin's own answer must appear:
{got}");

    // And the refusal the README tells a reader to try, so the example's two claims are both gated.
    let ungranted = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&work)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(["run", "host.delulu", "--grant", "console", "--no-prompt"])
        .output()
        .expect("run delulu");
    let refused = text(&ungranted);
    assert!(!ungranted.status.success(), "without the grant it must not run:
{refused}");
    assert!(refused.contains("DL0703"), "and the refusal names the not-granted code:
{refused}");
    let _ = std::fs::remove_dir_all(&work);
}
