//! `delulu-conform` — the conformance coverage tool (Stage 9, invariant 42).
//!
//! Usage:
//!   delulu-conform --coverage [--json] [--root <dir>]
//!
//! Exit 0 only at 100% anchor coverage with no validation errors; nonzero otherwise.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut coverage = false;
    let mut json = false;
    let mut reference = false;
    let mut check_reference = false;
    let mut root: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--coverage" => coverage = true,
            "--reference" => reference = true,
            "--check-reference" => check_reference = true,
            "--json" => json = true,
            "--root" => {
                i += 1;
                root = args.get(i).map(PathBuf::from);
            }
            "-h" | "--help" => {
                print!("{}", usage());
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("delulu-conform: unknown argument `{other}`\n\n{}", usage());
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    if !coverage && !reference && !check_reference {
        eprint!("{}", usage());
        return ExitCode::from(2);
    }

    let root = root.unwrap_or_else(delulu_conform::repo_root);

    if reference {
        match delulu_conform::reference::write(&root) {
            Ok(paths) => {
                for p in &paths {
                    println!("wrote {p}");
                }
            }
            Err(e) => {
                eprintln!("delulu-conform: cannot write the reference: {e}");
                return ExitCode::from(2);
            }
        }
    }

    if check_reference {
        let stale = delulu_conform::reference::drift(&root);
        if stale.is_empty() {
            println!("reference: in sync with the compiler source ({} chapters)",
                     delulu_conform::reference::generate(&root).len());
        } else {
            eprintln!(
                "delulu-conform: the generated reference is STALE — run `delulu-conform --reference`:"
            );
            for s in &stale {
                eprintln!("  ✗ {s}");
            }
            return ExitCode::from(1);
        }
    }

    if !coverage {
        return ExitCode::SUCCESS;
    }

    let cov = delulu_conform::run_coverage(&root);

    if json {
        println!("{}", serde_json::to_string_pretty(&delulu_conform::json_report(&cov)).unwrap());
    } else {
        print!("{}", delulu_conform::human_report(&cov));
    }

    if cov.pass() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn usage() -> String {
    "delulu-conform — the DeluluLang conformance coverage law (invariant 42)\n\
     \n\
     USAGE:\n\
     \x20 delulu-conform --coverage [--json] [--root <dir>]\n\
     \x20 delulu-conform --reference [--root <dir>]        regenerate docs/reference/\n\
     \x20 delulu-conform --check-reference [--root <dir>]  fail if the reference is stale\n\
     \n\
     Maps the conformance suite's tests to the reference anchor registry (diagnostics, primitive\n\
     table, grammar productions, audit rules R-1..R-7, CLI subcommands) and fails unless every\n\
     anchor has at least one accepting AND one rejecting witness. Exit 0 only at 100%.\n"
        .to_string()
}
