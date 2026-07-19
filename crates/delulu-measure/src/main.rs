//! `delulu-measure` — runs the Stage 9 measurement studies and writes their raw data.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out: Option<PathBuf> = None;
    let mut study = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "study-a" => study = "a".into(),
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

    if study != "a" {
        eprint!("{}", usage());
        return ExitCode::from(2);
    }

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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn usage() -> String {
    "delulu-measure — the DeluluLang measurement program (Stage 9 §3)\n\
     \n\
     USAGE:\n\
     \x20 delulu-measure study-a [--out <dir>]\n\
     \n\
     Study A generates a 25-package corpus with dependency depth 4, verifies each graph clean,\n\
     then injects an effect at every library position and checks that every injection is refused.\n\
     Exit 0 only at a 100% catch rate over a non-empty set.\n"
        .to_string()
}
