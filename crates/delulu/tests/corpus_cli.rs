//! The capability corpus, exercised rather than merely parsed (campaign finding C7).
//!
//! `tests/conformance.rs` proves every corpus program CHECKS clean and every corpus package
//! BUILDS clean. That is a real property and it is not the same as working. A program can type
//! check and still print the wrong number, and an authority report can be produced and still name
//! the wrong effects — so the corpus needs at least some of its claims tested by running them.
//!
//! This file does that for the three programs whose behaviour is a claim about the language:
//! the tier-3 application (reads real input, and must reject a value `Float` cannot hold), the
//! tier-5 attenuation program (two disjoint narrowings of one capability), and the tier-4
//! multi-package pipeline (whose authority is computed across four manifests).
//!
//! The tier-4 assertions here are deliberately about the ANALYSIS surface — the authority join
//! across four manifests and the `why` chain that crosses a package boundary. Running that program is
//! covered separately in `package_run.rs`: `delulu run <package-dir>` used to be impossible (campaign
//! finding C59) and is not any more (ruling D61), so the two files split the claim rather than
//! duplicating it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn write(dir: &Path, rel: &str, body: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

/// Copy a corpus program into a scratch directory so a run cannot write into the repository.
fn stage(name: &str, rel: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu-corpus-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = repo_root().join(rel);
    let file = Path::new(rel).file_name().unwrap();
    std::fs::copy(&src, dir.join(file)).unwrap();
    dir
}

/// The tier-3 application, on input that includes a line no `Float` can hold.
///
/// `bay6=1.0e400` is the interesting row. `parse_float` answers `None` for it — the same rule the
/// lexer applies to a literal in source — so the reading is skipped rather than becoming `inf` and
/// winning "hottest" forever. Without that rule this test prints `inf`, which is exactly the
/// failure C17 described: a number that is not the number anyone wrote.
#[test]
fn the_tier3_application_reads_real_input_and_refuses_a_value_float_cannot_hold() {
    let dir = stage("tier3", "tests/corpus/tier3-application/telemetry_report.delulu");
    write(&dir, "data/telemetry.txt", "bay3=21.5\nbay4=8.25\nbay5=41.0\nbroken\nbay6=1.0e400\n");

    let out = delulu(
        &dir,
        &["run", "telemetry_report.delulu", "--grant", "console", "--grant", "fs.read=./data"],
    );
    assert_eq!(stdout(&out).trim(), "hottest reading: 41.0", "stdout was {:?}", stdout(&out));

    // The failure paths are distinguishable, which is the file's other claim.
    let empty = stage("tier3empty", "tests/corpus/tier3-application/telemetry_report.delulu");
    write(&empty, "data/telemetry.txt", "   \n");
    let out = delulu(
        &empty,
        &["run", "telemetry_report.delulu", "--grant", "console", "--grant", "fs.read=./data"],
    );
    assert_eq!(stdout(&out).trim(), "the telemetry file was empty");
}

/// The tier-5 attenuation program: one capability, narrowed twice, reaching two disjoint subtrees.
#[test]
fn the_tier5_program_reads_through_two_independent_narrowings() {
    let dir = stage("tier5", "tests/corpus/tier5-security/attenuation.delulu");
    write(&dir, "data/public/notice.txt", "reactor bay open");
    write(&dir, "data/archive/index.txt", "log 001");

    let out = delulu(
        &dir,
        &["run", "attenuation.delulu", "--grant", "console", "--grant", "fs.read=./data"],
    );
    let s = stdout(&out);
    assert!(s.contains("reactor bay open"), "stdout was {s:?}");
    assert!(s.contains("log 001"), "stdout was {s:?}");

    // And the grant is the bound, not a suggestion: narrowed to `./data/public`, the same program
    // cannot reach `./data/archive`. The narrowing is a subset of a grant that no longer covers it.
    let out = delulu(
        &dir,
        &["run", "attenuation.delulu", "--grant", "console", "--grant", "fs.read=./data/public"],
    );
    let s = stdout(&out) + &String::from_utf8_lossy(&out.stderr);
    assert!(
        !s.contains("log 001"),
        "a capability narrowed under a grant that does not cover ./data/archive must not read it: {s:?}"
    );
}

/// The tier-4 package: four manifests, and one authority answer computed from all of them.
///
/// The assertions are about the JOIN. No single manifest in this graph says `Read, Write` over
/// both directories — that answer exists only across the four, which is the reason the tier is
/// package-shaped and not a long file.
#[test]
fn the_tier4_package_reports_the_authority_of_its_whole_graph() {
    let root = repo_root();
    let pkg = "tests/corpus/tier4-multimodule/station";

    let out = delulu(&root, &["authority", pkg]);
    let s = stdout(&out);
    for module in
        ["archive", "policy", "reading", "reading.parse", "reading.types", "station", "station.report"]
    {
        assert!(s.contains(module), "authority should name module `{module}`:\n{s}");
    }
    assert!(s.contains("Read, Write"), "the join of the graph is Read+Write:\n{s}");
    assert!(s.contains("./data"), "fs.read should be named:\n{s}");
    assert!(s.contains("./out"), "fs.write should be named:\n{s}");
    // Three of the four packages perform no effect at all, and the report can see that across
    // package boundaries — `policy::classify` is proved pure from another package's source.
    assert!(s.contains("policy::classify"), "cross-package pure fns should be listed:\n{s}");
    assert!(s.contains("(none — no code outside the guarantee)"), "no foreign code:\n{s}");

    // `why` must cross the package boundary, or the answer would stop at the edge of the file
    // that happens to hold `main` — which is where the interesting authority never is.
    let out = delulu(&root, &["why", "Write", pkg]);
    let s = stdout(&out);
    assert!(s.contains("dep:archive/src/archive.delulu"), "why should reach into the dependency:\n{s}");
    assert!(s.contains("main"), "the chain should start at main:\n{s}");
}
