//! `delulu doctor` — the one-command health check.
//!
//! The property that matters most here is not that doctor reports correctly; it is that **doctor
//! does not damage the thing it is checking**. It runs inside the repository, it can write, and
//! the suite runs it in parallel with tests that read those same files. Every test below exists
//! because some version of that goes wrong if nobody is holding it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    let home = std::env::temp_dir().join(format!("delulu-doctor-{}-{:?}", std::process::id(), args.first()));
    let _ = std::fs::create_dir_all(&home);
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .env("DELULU_HOME", &home)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// The accepting witness. `--check` is deliberate: see `doctor_never_writes_when_the_map_is_current`
/// for why a *writing* doctor must not be the one the suite runs by default.
#[test]
fn doctor_reports_a_healthy_checkout() {
    let o = delulu(&["doctor", "--check"]);
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "doctor should exit 0 on a healthy tree:\n{out}");
    assert!(out.contains("environment"), "the environment section must be present:\n{out}");
    assert!(out.contains("repository"), "run inside the source tree, the repository section must appear:\n{out}");
    assert!(out.contains("survey freshness"), "the survey freshness check must appear:\n{out}");
}

/// The rejecting witness (house rule 3): a bad invocation refuses, and says so on stderr.
#[test]
fn doctor_refuses_an_option_it_does_not_know() {
    let o = delulu(&["doctor", "--not-a-real-flag"]);
    assert_eq!(o.status.code(), Some(2), "an unknown option is a usage error, which is exit 2");
    assert!(stdout(&o).is_empty(), "a usage error must put nothing on stdout; the reason goes to stderr");
}

/// Exactly one JSON object on stdout, on success and on failure.
///
/// This failed when first written. The outer CLI already emits the single usage envelope for any
/// exit-2 under `--json`, and doctor emitted its own as well — two objects, which is precisely the
/// defect campaign C2's fix introduced and `json_contract` was written to catch. A test that only
/// asked "does this look like JSON?" would have passed it.
#[test]
fn doctor_emits_exactly_one_json_object() {
    for args in [vec!["doctor", "--json", "--check"], vec!["doctor", "--json", "--not-a-real-flag"]] {
        let o = delulu(&args);
        let out = stdout(&o);
        let mut stream = serde_json::Deserializer::from_str(&out).into_iter::<serde_json::Value>();
        let first = stream.next();
        assert!(first.is_some() && first.unwrap().is_ok(), "stdout must parse as JSON for {args:?}:\n{out}");
        assert!(stream.next().is_none(), "stdout must hold EXACTLY one JSON value for {args:?}:\n{out}");
    }
}

/// The envelope carries the documented fields, and the doctor payload is additive.
#[test]
fn the_json_envelope_keeps_its_shape() {
    let o = delulu(&["doctor", "--json", "--check"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("one envelope");
    assert_eq!(v["command"], "doctor");
    assert_eq!(v["schema"], 1);
    assert_eq!(v["delulu_version"], env!("CARGO_PKG_VERSION"));
    assert!(v["diagnostics"].as_array().is_some_and(|d| d.is_empty()), "a health check invents no DL codes");
    assert!(v["doctor"]["checks"].as_array().is_some_and(|c| !c.is_empty()), "checks must be reported");
    assert!(v["doctor"]["survey"]["nodes"].as_u64().is_some(), "inside the source tree, survey facts are reported");
}

/// **Doctor must not modify a healthy repository.**
///
/// It runs where the map lives, and the suite runs it alongside tests that read that map. If a
/// no-op run rewrote the files, every `delulu doctor` would be a repository change and concurrent
/// readers would see torn writes — campaign finding C69 in a new costume. Regeneration touches
/// only files that actually differ, and this is what holds that.
#[test]
fn doctor_never_writes_when_the_map_is_current() {
    let survey_dir = workspace_root().join("docs/survey");
    let before = mtimes(&survey_dir);
    assert!(!before.is_empty(), "docs/survey must exist for this test to mean anything");

    // The precondition is asserted, never repaired. This test is the only one that runs doctor in
    // writing mode, and if it were allowed to regenerate a stale map it would quietly fix the very
    // thing `delulu-survey`'s freshness gate exists to fail on — a test that repairs what another
    // test checks is worse than no test. On a stale tree this says so and stops.
    let pre = delulu(&["doctor", "--check"]);
    assert_eq!(
        pre.status.code(),
        Some(0),
        "precondition: the committed map must already be current. It is not — run \
         `cargo run -p delulu-survey -- build`. This test will not repair it, because the freshness \
         gate is what should be failing here.\n{}",
        stdout(&pre)
    );

    let o = delulu(&["doctor"]);
    assert_eq!(o.status.code(), Some(0), "doctor should exit 0:\n{}", stdout(&o));

    assert_eq!(
        mtimes(&survey_dir),
        before,
        "doctor rewrote generated files on a healthy tree — running it must not be a change"
    );
}

fn mtimes(dir: &Path) -> Vec<(String, std::time::SystemTime)> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<(String, std::time::SystemTime)> = rd
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let m = e.metadata().ok()?.modified().ok()?;
            Some((name, m))
        })
        .collect();
    out.sort();
    out
}

/// `--check` reports and never writes, which is what makes it safe from a hook or a CI step that
/// must not mutate the checkout.
#[test]
fn check_mode_is_read_only() {
    let survey_dir = workspace_root().join("docs/survey");
    let before = mtimes(&survey_dir);
    let o = delulu(&["doctor", "--check"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(mtimes(&survey_dir), before, "--check must never write");
}

/// Outside the DeluluLang source tree there is no map to check, and doctor says so rather than
/// reporting on a checkout it is not standing in.
#[test]
fn outside_the_source_tree_the_repository_section_is_skipped() {
    let elsewhere = std::env::temp_dir().join(format!("delulu-doctor-elsewhere-{}", std::process::id()));
    std::fs::create_dir_all(&elsewhere).expect("scratch dir");
    let home = elsewhere.join("home");
    std::fs::create_dir_all(&home).expect("home");

    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&elsewhere)
        .env("DELULU_HOME", &home)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(["doctor"])
        .output()
        .expect("failed to run delulu");

    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "a healthy machine outside the repo is still healthy:\n{out}");
    assert!(out.contains("environment"), "environment checks run everywhere:\n{out}");
    // Substance, not a phrase. The wording here is user-facing prose and will be improved again;
    // what must hold is that the section is NAMED rather than silently omitted, that it says which
    // source tree it means, and that nothing which could not run is reported as having passed.
    assert!(out.contains("repository"), "the section is named, not dropped:\n{out}");
    assert!(out.contains("source tree"), "and it says which tree it means:\n{out}");
    assert!(
        !out.contains("survey freshness"),
        "a check that could not run must not appear at all:\n{out}"
    );
    let _ = std::fs::remove_dir_all(&elsewhere);
}
