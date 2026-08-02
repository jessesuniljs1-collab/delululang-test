//! The core still answers exactly as recorded.
//!
//! Almost everything built after v1.0 sits *around* the language rather than in it — the language
//! server, `fix`, `new`, `completions`, `add --path`, the Survey, the diagnostic dispositions.
//! Those exist for the people and agents who build DeluluLang. The language is what everyone else
//! depends on, and a green test suite does not protect it: the suite proves the assertions someone
//! wrote still hold, not that the compiler still *decides the same things* about real programs.
//!
//! Those are different claims. A refactor can keep every assertion passing while quietly changing
//! a span, a repair, an inferred row, or the order of an effect list — and nothing would say so,
//! because no test asserts on output nobody thought to pin.
//!
//! So this pins it. Every program the repository ships is run through the core surfaces and the
//! exact bytes are committed to `tests/core-invariance/SNAPSHOT.txt`. Any change to what the
//! compiler says about any shipped program turns into a diff in a reviewed file. The snapshot is
//! not a set of assertions about what is *right* — it is a record of what is *true today*, so that
//! changing it has to be deliberate.
//!
//! Only deterministic surfaces are recorded. `run` is deliberately excluded: it reaches the clock,
//! the random source and the filesystem, and a gate that is flaky is a gate that gets deleted.
//!
//! To bless an intended change:
//!
//! ```text
//! DELULU_BLESS=1 cargo test -p delulu --test core_invariance
//! ```
//!
//! then read the diff before committing it. If you cannot explain a line of that diff, the change
//! that produced it is not finished.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn snapshot_path() -> PathBuf {
    root().join("tests").join("core-invariance").join("SNAPSHOT.txt")
}

/// One command, rendered so that a difference in *any* channel shows up as a text difference.
fn surface(args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(args)
        .output()
        .expect("run delulu");

    let clean = |b: &[u8]| {
        // Line endings are the one thing that legitimately differs between platforms.
        // The version is stamped into every JSON envelope; pinning it here would make a
        // release bump look like a semantic change to all 83 programs.
        String::from_utf8_lossy(b)
            .replace("\r\n", "\n")
            .replace(&format!("\"delulu_version\": \"{}\"", env!("CARGO_PKG_VERSION")), "\"delulu_version\": \"<version>\"")
    };

    format!(
        "exit {}\n--- stdout ---\n{}--- stderr ---\n{}",
        o.status.code().map(|c| c.to_string()).unwrap_or_else(|| "signal".into()),
        clean(&o.stdout),
        clean(&o.stderr),
    )
}

/// Every program the repository ships, as a repo-relative forward-slash path.
///
/// Single modules *and* package directories, because `check some/pkg` resolves imports, merges
/// module rows and applies the manifest ceiling — none of which happens on a lone file, and all
/// of which is core.
///
/// Forward slashes are not cosmetic: the CLI echoes back the path it was given, so passing them
/// this way is what makes one snapshot valid on Windows, Linux and macOS alike.
fn corpus() -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.join("delulu.toml").is_file() {
                    out.push(p.clone());
                }
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                out.push(p);
            }
        }
    }

    let base = root();
    let mut found = Vec::new();
    for area in ["examples", "tests/conformance", "tests/corpus"] {
        walk(&base.join(area), &mut found);
    }

    let mut rel: Vec<String> = found
        .iter()
        .map(|p| {
            p.strip_prefix(&base)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    rel.sort();
    rel
}

fn render() -> String {
    let mut s = String::new();
    s.push_str(
        "# DeluluLang core invariance snapshot\n\
         #\n\
         # GENERATED — do not edit by hand.\n\
         #   DELULU_BLESS=1 cargo test -p delulu --test core_invariance\n\
         #\n\
         # What the compiler says about every program this repository ships. A diff here is a\n\
         # change in the language's answers, whatever the commit claimed to be about. See\n\
         # crates/delulu/tests/core_invariance.rs for why this file exists.\n",
    );

    for path in corpus() {
        let checked = surface(&["check", &path, "--json"]);
        let clean = checked.starts_with("exit 0\n");

        let mut cases: Vec<(String, String)> = vec![
            ("check --json".to_string(), checked),
            ("check".to_string(), surface(&["check", &path])),
        ];
        // On a program that does not check, `authority` and `why` re-print the same
        // diagnostics. Recording them would triple the file with duplicates and hide the
        // signal, so the authority surfaces are recorded for programs that have one.
        if clean {
            cases.push(("authority --json".into(), surface(&["authority", &path, "--json"])));
            cases.push(("authority".into(), surface(&["authority", &path])));
            cases.push(("why Write".into(), surface(&["why", "Write", &path])));
        }

        for (label, body) in cases {
            s.push_str(&format!("\n===== {path} :: {label} =====\n{body}"));
            if !s.ends_with('\n') {
                s.push('\n');
            }
        }
    }
    s
}

/// The gate.
#[test]
fn the_core_still_answers_exactly_as_recorded() {
    // A snapshot generated from an empty corpus would match an empty snapshot, and the gate
    // would pass while guarding nothing. Name the floor rather than trust the walk.
    let n = corpus().len();
    assert!(
        n >= 100,
        "the corpus walk found only {n} programs — it is looking in the wrong place, and a \
         snapshot built from it would guard nothing"
    );

    let fresh = render();
    let path = snapshot_path();

    if std::env::var_os("DELULU_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create snapshot dir");
        std::fs::write(&path, fresh.as_bytes()).expect("write snapshot");
        eprintln!("blessed {} ({n} programs)", path.display());
        return;
    }

    let recorded = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}\nbless it with DELULU_BLESS=1", path.display()))
        .replace("\r\n", "\n");

    if recorded == fresh {
        return;
    }

    // A 200 KB "left != right" is not a failure report. Name the cases that moved.
    let split = |doc: &str| -> Vec<(String, String)> {
        let mut out = Vec::new();
        for chunk in doc.split("\n===== ").skip(1) {
            let (head, body) = chunk.split_once(" =====\n").unwrap_or((chunk, ""));
            out.push((head.to_string(), body.to_string()));
        }
        out
    };
    let (was, now) = (split(&recorded), split(&fresh));

    let mut report = String::new();
    for (name, body) in &now {
        match was.iter().find(|(n, _)| n == name) {
            None => report.push_str(&format!("\n+ NEW  {name}\n{body}")),
            Some((_, old)) if old != body => report.push_str(&format!(
                "\n~ CHANGED  {name}\n--- recorded ---\n{old}--- now ---\n{body}"
            )),
            Some(_) => {}
        }
    }
    for (name, _) in &was {
        if !now.iter().any(|(n, _)| n == name) {
            report.push_str(&format!("\n- GONE  {name}\n"));
        }
    }

    panic!(
        "the core's answers moved.\n\
         \n\
         This test does not know whether that is a fix or a regression — it knows only that\n\
         something outside the language changed what the language says. Read every case below.\n\
         If all of it is intended, re-record with:\n\
         \n    DELULU_BLESS=1 cargo test -p delulu --test core_invariance\n\
         \n\
         and commit {} with the change that caused it.\n{}",
        snapshot_path().display(),
        report
    );
}
