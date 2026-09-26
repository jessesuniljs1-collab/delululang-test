//! `delulu-measure` — runs the Stage 9 measurement studies and writes their raw data.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("ai-usability") {
        return run_ai_usability(&args[1..]);
    }
    let mut out: Option<PathBuf> = None;
    let mut study = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "study-a" => study = "a".into(),
            "study-b" => study = "b".into(),
            "study-c" => study = "c".into(),
            "all" => study = "all".into(),
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
            }
            "-h" | "--help" => {
                print!("{}", usage());
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("delulu-measure: unknown argument `{other}`\n\n{}", usage());
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    match study.as_str() {
        "a" => {}
        "b" => return run_b(out),
        "c" => return run_c(out),
        "all" => {
            for f in [run_a as fn(Option<PathBuf>) -> ExitCode, run_b, run_c] {
                let code = f(None);
                if code != ExitCode::SUCCESS {
                    return code;
                }
            }
            return ExitCode::SUCCESS;
        }
        _ => {
            eprint!("{}", usage());
            return ExitCode::from(2);
        }
    }
    run_a(out)
}

/// Study B: the repair-loop measurement.
fn run_b(out: Option<PathBuf>) -> ExitCode {
    let out = out.unwrap_or_else(|| repo_root().join("measurements/study-b"));
    let work = std::env::temp_dir().join(format!("delulu-study-b-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    eprintln!("study B: running the mechanical repair loop over the defect corpus...");
    let r = match delulu_measure::study_b::run_study(&repo_root(), &work) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("delulu-measure: study B failed to run: {e}");
            return ExitCode::from(1);
        }
    };
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("delulu-measure: cannot create {}: {e}", out.display());
        return ExitCode::from(1);
    }
    let _ = std::fs::write(
        out.join("results.json"),
        format!("{}\n", serde_json::to_string_pretty(&delulu_measure::study_b::json(&r)).unwrap()),
    );
    let _ = std::fs::write(out.join("REPORT.md"), delulu_measure::study_b::report(&r));
    let _ = std::fs::remove_dir_all(&work);
    println!(
        "study B: {} tasks, {} with a machine-applicable repair ({:.1}%), {} reached green mechanically",
        r.tasks.len(),
        r.with_repair(),
        r.repair_availability_pct(),
        r.repaired_to_green()
    );
    ExitCode::SUCCESS
}

/// Study C: the performance baseline.
fn run_c(out: Option<PathBuf>) -> ExitCode {
    let out = out.unwrap_or_else(|| repo_root().join("measurements/study-c"));
    let work = std::env::temp_dir().join(format!("delulu-study-c-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    eprintln!("study C: running benchmarks across every available lane...");
    let r = match delulu_measure::study_c::run_study(&work) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("delulu-measure: study C failed to run: {e}");
            return ExitCode::from(1);
        }
    };
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("delulu-measure: cannot create {}: {e}", out.display());
        return ExitCode::from(1);
    }
    let _ = std::fs::write(
        out.join("results.json"),
        format!("{}\n", serde_json::to_string_pretty(&delulu_measure::study_c::json(&r)).unwrap()),
    );
    let _ = std::fs::write(out.join("REPORT.md"), delulu_measure::study_c::report(&r));
    let _ = std::fs::remove_dir_all(&work);
    println!(
        "study C: {} benchmarks across lanes [{}]",
        r.benches.len(),
        r.lanes_available.join(", ")
    );
    ExitCode::SUCCESS
}

/// Study A: the injection campaign.
fn run_a(out: Option<PathBuf>) -> ExitCode {
    let out = out.unwrap_or_else(|| repo_root().join("measurements/study-a"));
    let work = std::env::temp_dir().join(format!("delulu-study-a-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);

    eprintln!("study A: generating corpus and running the injection campaign...");
    let results = match delulu_measure::study_a::run_study(&work) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("delulu-measure: study A failed to run: {e}");
            return ExitCode::from(1);
        }
    };

    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("delulu-measure: cannot create {}: {e}", out.display());
        return ExitCode::from(1);
    }
    // The corpus is committed alongside the results so a reviewer can read the exact packages the
    // numbers came from, rather than having to trust the generator's description of them.
    let corpus_dir = out.join("corpus");
    let _ = std::fs::remove_dir_all(&corpus_dir);
    if let Err(e) = std::fs::create_dir_all(&corpus_dir) {
        eprintln!("delulu-measure: cannot write the corpus: {e}");
        return ExitCode::from(1);
    }
    for chain in delulu_measure::corpus::build() {
        if let Err(e) = delulu_measure::corpus::write_chain(&corpus_dir, &chain) {
            eprintln!("delulu-measure: cannot write chain {}: {e}", chain.name);
            return ExitCode::from(1);
        }
    }

    let json = delulu_measure::study_a::json(&results);
    let _ = std::fs::write(
        out.join("results.json"),
        format!("{}\n", serde_json::to_string_pretty(&json).unwrap()),
    );
    let _ = std::fs::write(out.join("REPORT.md"), delulu_measure::study_a::report(&results));
    let _ = std::fs::remove_dir_all(&work);

    println!(
        "study A: {} injection sites, {} caught ({:.1}%)",
        results.injections.len(),
        results.caught(),
        results.catch_rate()
    );
    println!("wrote {}/results.json and {}/REPORT.md", out.display(), out.display());

    if results.mechanism_holds() {
        ExitCode::SUCCESS
    } else {
        eprintln!("delulu-measure: the mechanism claim does NOT hold — see the report");
        ExitCode::from(1)
    }
}

/// The `delulu` next to this binary — the one `cargo build` made beside it.
fn sibling_delulu() -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let dir = exe.parent().expect("target dir");
    dir.join(if cfg!(windows) { "delulu.exe" } else { "delulu" })
}

/// `ai-usability prepare --out DIR [--tasks t1,t2] [--conditions c1,c2] [--delulu PATH]` and
/// `ai-usability score DIR [--delulu PATH] [--record OUT]` (V2 P4-08).
fn run_ai_usability(args: &[String]) -> ExitCode {
    use delulu_measure::ai_usability as ai;
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let list = |name: &str, all: Vec<String>| -> Vec<String> {
        flag(name).map(|v| v.split(',').map(str::to_string).collect()).unwrap_or(all)
    };
    let delulu = flag("--delulu").map(PathBuf::from).unwrap_or_else(sibling_delulu);
    if !delulu.is_file() {
        eprintln!("delulu-measure: no delulu binary at {} — build it (`cargo build -p delulu`) or pass --delulu", delulu.display());
        return ExitCode::from(2);
    }
    match args.first().map(String::as_str) {
        Some("prepare") => {
            let Some(out) = flag("--out").map(PathBuf::from) else {
                eprintln!("delulu-measure: `ai-usability prepare` needs --out DIR");
                return ExitCode::from(2);
            };
            let tasks = list("--tasks", ai::tasks().iter().map(|t| t.id.clone()).collect());
            let conds = list("--conditions", ai::CONDITIONS.iter().map(|c| c.id.to_string()).collect());
            match ai::prepare(&out, &repo_root(), &delulu, &tasks, &conds) {
                Ok(made) => {
                    println!("ai-usability: {} workspace(s) under {}", made.len(), out.join("runs").display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("delulu-measure: prepare failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("score") => {
            let Some(dir) = args.get(1).filter(|a| !a.starts_with('-')).map(PathBuf::from) else {
                eprintln!("delulu-measure: `ai-usability score` needs the prepared directory");
                return ExitCode::from(2);
            };
            let r = match ai::score(&dir, &delulu) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("delulu-measure: score failed: {e}");
                    return ExitCode::from(1);
                }
            };
            let _ = std::fs::write(dir.join("results.json"), format!("{}\n", serde_json::to_string_pretty(&r).unwrap()));
            let _ = std::fs::write(dir.join("REPORT.md"), ai::report(&r));
            if let Some(out) = flag("--record").map(PathBuf::from) {
                if let Err(e) = ai::record(&dir, &out) {
                    eprintln!("delulu-measure: could not write the record: {e}");
                    return ExitCode::from(1);
                }
            }
            println!("ai-usability: {} run(s) scored; valid: {}", r["runs"].as_array().map_or(0, Vec::len), r["valid"]);
            if r["valid"] == true { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        _ => {
            eprint!("{}", usage());
            ExitCode::from(2)
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn usage() -> String {
    "delulu-measure — the DeluluLang measurement program (Stage 9 §3)\n\
     \n\
     USAGE:\n\
     \x20 delulu-measure study-a [--out <dir>]   authority verification at scale\n\
     \x20 delulu-measure study-b [--out <dir>]   agent repair loops\n\
     \x20 delulu-measure study-c [--out <dir>]   the performance honesty baseline\n\
     \x20 delulu-measure all                     all three, to their default directories\n\
     \x20 delulu-measure ai-usability prepare --out DIR [--tasks t1,..] [--conditions c1,..]\n\
     \x20 delulu-measure ai-usability score DIR [--record OUT]   the V2 AI usability benchmark (P4-08)\n\
     \n\
     Study A generates a 25-package corpus with dependency depth 4, verifies each graph clean,\n\
     then injects an effect at every library position and checks that every injection is refused.\n\
     Exit 0 only at a 100% catch rate over a non-empty set.\n"
        .to_string()
}
