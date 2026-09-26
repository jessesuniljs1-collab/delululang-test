//! P4-08: the V2 AI usability benchmark's scorer, its negative controls and its harness, against the
//! real binary (`delulu_measure::ai_usability`).
//!
//! What is held here: for EVERY task the grammar generates, the scorer's negative controls behave —
//! the reference solution scores perfectly (so every task is solvable, least-authority, sandboxable),
//! an empty file and a file that does not parse fail, and the reference widened by one grant is
//! counted as exactly that authority mistake; a prepared workspace confines a model to its condition
//! — the wrapper refuses a command the condition does not allow, logs every call, and snapshots each
//! checked attempt; and the recorded pilot REPLAYS: scoring the committed runs reproduces the
//! committed `results.json`.

use std::path::{Path, PathBuf};
use std::process::Command;

use delulu_measure::ai_usability as ai;

fn delulu() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_delulu"))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-ai-usability-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn every_tasks_negative_controls_behave() {
    let dir = scratch("controls");
    let tasks = ai::tasks();
    assert_eq!(tasks.len(), 9);
    for t in &tasks {
        let c = ai::controls(&delulu(), t, &dir.join(&t.id)).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(c["over_authorized"], serde_json::json!(["clock"]), "{c}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

fn python() -> &'static str {
    for p in ["python3", "python"] {
        if Command::new(p).arg("--version").output().is_ok_and(|o| o.status.success()) {
            return p;
        }
    }
    panic!("the benchmark's wrapper is a Python script, and neither `python3` nor `python` runs here");
}

#[test]
fn a_prepared_workspace_confines_the_model_to_its_condition() {
    let dir = scratch("prepare");
    let made = ai::prepare(&dir, &repo(), &delulu(), &["t6".to_string()], &["c1".to_string(), "c3".to_string()]).unwrap();
    assert_eq!(made.len(), 2);
    let c1 = dir.join("runs/c1__t6");
    let c3 = dir.join("runs/c3__t6");
    for f in ["TASK.md", "condition.json", "dl.py", "dl.local.json", "data/numbers.txt", ai::CANARY_FILE] {
        assert!(c1.join(f).is_file(), "{f}");
    }
    assert!(!c1.join("knowledge").exists(), "the first condition is given nothing");
    assert!(c3.join("knowledge/skills/delulu/SKILL.md").is_file(), "the third is given the Skill");
    assert!(!std::fs::read_to_string(c1.join("dl.py")).unwrap().contains(&*delulu().to_string_lossy()), "no machine path in the wrapper");

    // A command the condition does not allow is refused and logged; an allowed one runs and a
    // `check` snapshots the attempt.
    let py = python();
    let refused = Command::new(py).args(["dl.py", "toolchain", "--json"]).current_dir(&c1).output().unwrap();
    assert_eq!(refused.status.code(), Some(2), "{}", String::from_utf8_lossy(&refused.stderr));
    std::fs::write(c1.join("solution.delulu"), ai::task("t6").unwrap().reference()).unwrap();
    let checked = Command::new(py).args(["dl.py", "check", "solution.delulu"]).current_dir(&c1).output().unwrap();
    assert_eq!(checked.status.code(), Some(0), "{}", String::from_utf8_lossy(&checked.stderr));
    assert!(c1.join("attempts/attempt-01.delulu").is_file());
    let log = std::fs::read_to_string(c1.join("calls.jsonl")).unwrap();
    let calls: Vec<serde_json::Value> = log.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(calls.len(), 2);
    assert_eq!((calls[0]["allowed"].clone(), calls[0]["exit"].clone()), (serde_json::json!(false), serde_json::json!(2)));
    assert_eq!((calls[1]["allowed"].clone(), calls[1]["attempt"].clone()), (serde_json::json!(true), serde_json::json!(1)));
    let allowed3 = Command::new(py).args(["dl.py", "toolchain", "--json"]).current_dir(&c3).output().unwrap();
    assert_eq!(allowed3.status.code(), Some(0), "the third condition may introspect");

    // Scoring the workspace: the one run with the reference solution completes on the first try.
    let r = ai::score(&dir, &delulu()).unwrap();
    assert_eq!(r["valid"], true, "{r:#}");
    let run = r["runs"].as_array().unwrap().iter().find(|x| x["run"] == "c1__t6").unwrap().clone();
    assert_eq!((run["compile_first_try"].clone(), run["completed"].clone()), (serde_json::json!(true), serde_json::json!(true)), "{run}");
    assert_eq!(run["refused_tool_calls"], 1);
    assert_eq!(run["tokens"], serde_json::Value::Null, "no harness recorded tokens: UNRUN, not zero");
    let report = ai::report(&r);
    assert!(report.contains("UNRUN"), "{report}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The recorded result REPLAYS: scoring the committed runs — the programs the model wrote, the
/// attempts the wrapper snapshotted, the call logs and the run records — reproduces the committed
/// `results.json`, controls and all. Nothing in the published numbers depends on anything not in
/// the record. If an intended change to the toolchain changes a verdict, re-score with
/// `cargo run -p delulu-measure -- ai-usability score measurements/ai-usability --record
/// measurements/ai-usability` and review the difference.
#[test]
fn the_recorded_result_replays_from_its_own_evidence() {
    let record = repo().join("measurements/ai-usability");
    let committed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(record.join("results.json")).expect("the first result is recorded")).unwrap();
    assert_eq!(committed["valid"], true, "the published result's controls held");
    let replayed = ai::score(&record, &delulu()).unwrap();
    assert_eq!(replayed, committed, "scoring the record's own evidence gives its published numbers");
    let report = std::fs::read_to_string(record.join("REPORT.md")).unwrap().replace("\r\n", "\n");
    assert_eq!(report, ai::report(&committed), "REPORT.md is generated from results.json, nothing added");
}
