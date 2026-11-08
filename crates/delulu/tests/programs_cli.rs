//! **The programs of D-V2-220's step** — after each language basic, programs of varying size are written, run on both
//! engines and kept (the owner, 2026-11-06: *"After each build, write and run programs and softwares of varying
//! complexities and test and verify and fix it"*).
//!
//! Every program in `crates/delulu/tests/programs/` names the engines it runs on in its first line (`// engines: interp
//! wasm`) and has its expected output beside it, a `.out` of the same name. Each is run with `delulu run --grant console` on every engine it
//! names, and must exit 0 and print exactly its `.out`; each must also be canonically formatted (`delulu fmt --check`),
//! so a program kept here is one a reader can copy. A program that one engine cannot run says so by naming one engine,
//! and the row that would let it run on both is named in its header.

use std::path::{Path, PathBuf};
use std::process::Command;

fn programs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("programs")
}

/// Every program, sorted, with its engines and its expected output.
fn programs() -> Vec<(PathBuf, Vec<String>, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(programs_dir()).expect("tests/programs exists") {
        let p = entry.unwrap().path();
        if p.extension().and_then(|e| e.to_str()) != Some("delulu") {
            continue;
        }
        let src = std::fs::read_to_string(&p).unwrap();
        let first = src.lines().next().unwrap_or("");
        let engines: Vec<String> = first
            .strip_prefix("// engines:")
            .unwrap_or_else(|| panic!("{}: the first line names its engines (`// engines: interp wasm`)", p.display()))
            .split_whitespace()
            .map(str::to_string)
            .collect();
        assert!(!engines.is_empty(), "{}: names no engine", p.display());
        for e in &engines {
            assert!(e == "interp" || e == "wasm", "{}: unknown engine `{e}`", p.display());
        }
        let want = std::fs::read_to_string(p.with_extension("out"))
            .unwrap_or_else(|_| panic!("{}: its expected output, NAME.out, is beside it", p.display()))
            .replace("\r\n", "\n");
        out.push((p, engines, want));
    }
    out.sort();
    out
}

fn delulu(args: &[&str], file: &Path) -> std::process::Output {
    let state = std::env::temp_dir().join(format!("delulu-programs-{}", std::process::id()));
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .arg(file)
        .env("DELULU_STATE_DIR", &state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("run the delulu binary")
}

#[test]
fn every_program_prints_its_expected_output_on_each_engine_it_names() {
    let all = programs();
    // The control: the harness reads the programs that are there, never an empty directory.
    assert!(all.len() >= 5, "expected the kept programs, found {}", all.len());
    assert!(all.iter().filter(|(_, e, _)| e.iter().any(|x| x == "wasm")).count() >= 4, "most run on both engines");
    let mut failures = Vec::new();
    for (file, engines, want) in &all {
        for engine in engines {
            let args: &[&str] = if engine == "wasm" {
                &["run", "--engine", "wasm", "--grant", "console"]
            } else {
                &["run", "--grant", "console"]
            };
            let o = delulu(args, file);
            let got = String::from_utf8_lossy(&o.stdout).replace("\r\n", "\n");
            if !o.status.success() || got != *want {
                failures.push(format!(
                    "{} on {engine}: exit {:?}\n--- want ---\n{want}--- got ---\n{got}--- stderr ---\n{}",
                    file.display(),
                    o.status.code(),
                    String::from_utf8_lossy(&o.stderr)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_program_is_canonically_formatted() {
    for (file, _, _) in programs() {
        let o = delulu(&["fmt", "--check"], &file);
        assert!(
            o.status.success(),
            "{} is not as `delulu fmt` prints it:\n{}{}",
            file.display(),
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

/// Nothing stray: every `.out` has its program.
#[test]
fn every_expected_output_has_its_program() {
    for entry in std::fs::read_dir(programs_dir()).unwrap() {
        let p = entry.unwrap().path();
        match p.extension().and_then(|e| e.to_str()) {
            Some("out") => assert!(p.with_extension("delulu").is_file(), "{} has no program", p.display()),
            Some("delulu") => {}
            _ => panic!("{}: only programs and their outputs belong here", p.display()),
        }
    }
}
