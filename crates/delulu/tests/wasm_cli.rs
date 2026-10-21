//! End-to-end tests for `delulu run --engine wasm`: the WASM sandbox floor reached from the
//! terminal. A console `main` compiles to WebAssembly and runs under Wasmtime with the capability
//! checked host-side; its output must match the interpreter (the reference engine).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// A unique temp path for a `.dwx` built by a test (parallel tests must not collide).
fn temp_dwx(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("delulu_test_{}_{}.dwx", std::process::id(), tag))
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).current_dir(workspace_root()).args(args).output().expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

#[test]
fn run_engine_wasm_prints_console_output() {
    let o = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm", "--grant", "console"]);
    assert!(o.status.success(), "wasm run should succeed: {}", String::from_utf8_lossy(&o.stderr));
    assert!(stdout(&o).contains("hello from the wasm engine"), "{}", stdout(&o));
}

/// The book's guide chapters that the WASM engine runs since 2026-10-20 (RW 3.4), end to end through the command line:
/// `01` (records, `Float`, sums with payloads), `02` (a list begun empty, typed by the checker — what `compile_checked` is
/// for), `03` (`parse_int`), `04` (a function named as a value, `.trim`) — each the interpreter's output, byte for byte.
#[test]
fn the_guide_chapters_the_wasm_engine_runs_match_the_interpreter() {
    for chapter in [
        "examples/guide/01_types.delulu",
        "examples/guide/02_option_and_lists.delulu",
        "examples/guide/03_errors.delulu",
        "examples/guide/04_effects.delulu",
    ] {
        let w = delulu(&["run", chapter, "--engine", "wasm", "--grant", "console"]);
        let i = delulu(&["run", chapter, "--grant", "console"]);
        assert!(w.status.success(), "{chapter} on the WASM engine: {}", String::from_utf8_lossy(&w.stderr));
        assert!(i.status.success(), "{chapter} on the interpreter: {}", String::from_utf8_lossy(&i.stderr));
        assert!(!stdout(&i).is_empty(), "{chapter} prints");
        assert_eq!(stdout(&w), stdout(&i), "{chapter}: WASM and interpreter output must be identical");
    }
}

#[test]
fn run_engine_wasm_matches_the_interpreter() {
    let w = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm", "--grant", "console"]);
    let i = delulu(&["run", "examples/hello_wasm.delulu", "--grant", "console"]);
    assert!(w.status.success() && i.status.success());
    assert_eq!(stdout(&w), stdout(&i), "WASM and interpreter output must be identical");
}

/// WASM-OUTPUT-1: the CLI printed the WASM engine's captured output only when the run succeeded, so a program that
/// printed and then faulted showed nothing on stdout, where the interpreter shows every line it printed. The fault's
/// code is the same on both engines.
#[test]
fn what_a_program_printed_before_a_fault_reaches_stdout_on_both_engines() {
    let path = std::env::temp_dir().join(format!("delulu_printfault_{}.delulu", std::process::id()));
    std::fs::write(
        &path,
        "module m\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(\"first\")\n  let z = 0\n  \
         out.println(\"second\")\n  out.println(str(10 / z))\n  out.println(\"never\")\n}\n",
    )
    .expect("write");
    let p = path.to_string_lossy().to_string();
    let w = delulu(&["run", &p, "--engine", "wasm", "--grant", "console"]);
    let i = delulu(&["run", &p, "--grant", "console"]);
    let _ = std::fs::remove_file(&path);
    assert!(!w.status.success() && !i.status.success(), "both fault");
    assert_eq!(stdout(&i), "first\nsecond\n", "the interpreter");
    assert_eq!(stdout(&w), stdout(&i), "the WASM engine prints what the interpreter printed before the fault");
    let (we, ie) = (String::from_utf8_lossy(&w.stderr), String::from_utf8_lossy(&i.stderr));
    assert!(we.contains("DL0902") && ie.contains("DL0902"), "a division by zero on both:\n{we}\n{ie}");
}

#[test]
fn run_engine_wasm_ungranted_console_is_refused() {
    // No console grant → root.console() is refused host-side; the run fails.
    let o = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm"]);
    assert!(!o.status.success(), "an ungranted console must be refused on the wasm engine");
}

#[test]
fn run_engine_wasm_on_unsupported_program_is_dl1201() {
    // A list of functions is outside the WASM fragment, so `--engine wasm` reports DL1201 and the program stays on the
    // interpreter rather than being misrun. (A `while` loop was the example until RW 3.4 compiled it, then a `for` over
    // a list until the WASM `List` compiled that, then a list of lists until D-V2-173 nested them.)
    let path = std::env::temp_dir().join(format!("delulu_for_{}.delulu", std::process::id()));
    std::fs::write(&path, "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n var t = 0\n for g in [fn(x: Int) -> Int { x }] { t = t + g(1) }\n out.println(\"done\") }\n").expect("write");
    let p = path.to_string_lossy().to_string();
    let o = delulu(&["run", &p, "--engine", "wasm", "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1201"), "expected DL1201: {}", stdout(&o));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn run_engine_wasm_on_demo_reports_dl1205_secret() {
    // With generics/fs/Result/match all compiling now, demo.delulu gets all the way to its
    // `root.secret("API_KEY")` line — which is DL1205 (secrets never enter a guest), keeping the
    // program on the interpreter. (Progress marker: everything BEFORE the secret compiles.)
    let o = delulu(&["run", "examples/demo.delulu", "--engine", "wasm", "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1205"), "expected DL1205: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

// ----- Phase 3g: the .dwx authority-carrying artifact ---------------------------------------

#[test]
fn build_dwx_then_run_matches_the_interpreter() {
    // build --target wasm produces a self-describing artifact; running it must be byte-identical to
    // running the source on the interpreter (the reference engine).
    let dwx = temp_dwx("roundtrip");
    let dwx_s = dwx.to_string_lossy().to_string();
    let b = delulu(&["build", "examples/hello_wasm.delulu", "--target", "wasm", "-o", &dwx_s]);
    assert!(b.status.success(), "build should succeed: {}", String::from_utf8_lossy(&b.stderr));
    assert!(dwx.exists(), "the .dwx artifact must be written");

    let run = delulu(&["run", &dwx_s, "--grant", "console"]);
    assert!(run.status.success(), "running the .dwx should succeed: {}", String::from_utf8_lossy(&run.stderr));

    let interp = delulu(&["run", "examples/hello_wasm.delulu", "--grant", "console"]);
    assert_eq!(stdout(&run), stdout(&interp), "artifact output must match the interpreter");
    let _ = std::fs::remove_file(&dwx);
}

/// A `.dwx` run refuses, before anything runs, every `run` flag its path does not apply (the red-team pass of on-demand
/// routine run 5, F4, RW 3.19): `--trace-effects`, `--trace-out` and `--assert-trace` wrote no trace, and `--broker daemon`
/// and `--lease` ran the artifact under no custody — with no broker running, and a lease nobody redeemed — while it wrote
/// under the grant. The flags it applies still run it.
#[test]
fn a_dwx_run_refuses_the_flags_it_does_not_apply() {
    let dir = std::env::temp_dir().join(format!("delulu_dwxflags_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("out")).unwrap();
    let program = "module m\n\nfn main(root: Root) ! {Read, Write} {\n  let w = root.fs_write(\"./out\")\n  \
                   match w.write_text(\"log.txt\", \"started\") {\n    Ok(u) => root.console().println(\"wrote\"),\n    \
                   Err(e) => root.console().println(\"could not\"),\n  }\n}\n";
    std::fs::write(dir.join("w.delulu"), program).unwrap();
    let b = delulu_in(&dir, &["build", "w.delulu", "--target", "wasm", "-o", "w.dwx"]);
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    let granted = ["run", "w.dwx", "--grant", "console", "--grant", "fs.write=./out"];
    // Since 2026-10-21 the custody and trace flags are applied (`grants_cli.rs`
    // `a_sealed_artifact_runs_under_a_lease_and_the_daemon_with_its_trace`); what the route still does not apply is refused.
    for (flags, named) in [
        (&["--debug-rcaps"][..], "--debug-rcaps"),
        (&["--foreign-isolation", "process"][..], "--foreign-isolation"),
        (&["--actors-threads", "2"][..], "--actors-threads"),
        (&["--engine", "interp"][..], "--engine"),
    ] {
        let _ = std::fs::remove_file(dir.join("out").join("log.txt"));
        let args: Vec<&str> = granted.iter().chain(flags).copied().collect();
        let o = delulu_in(&dir, &args);
        let e = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(2), "{flags:?}: {e}");
        assert!(e.contains("a `.dwx` run does not apply") && e.contains(&format!("`{named}`")), "{flags:?}: {e}");
        assert!(!dir.join("out").join("log.txt").exists(), "{flags:?}: nothing ran");
        assert!(!dir.join("t.jsonl").exists());
    }
    // The flags it applies: the grants, the seed, the clock, `--engine wasm`, and the trace — written, and asserted
    // against the effects the artifact declares.
    let applied =
        ["--seed", "1", "--clock", "fixed:0", "--engine", "wasm", "--trace-effects", "--trace-out", "t.jsonl", "--assert-trace"];
    let args: Vec<&str> = granted.iter().chain(&applied).copied().collect();
    let o = delulu_in(&dir, &args);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(stdout(&o), "wrote\n");
    assert_eq!(std::fs::read_to_string(dir.join("out").join("log.txt")).unwrap(), "started");
    let traced = std::fs::read_to_string(dir.join("t.jsonl")).expect("the trace was written");
    assert!(traced.contains("\"op\":\"write_text\"") && traced.contains("\"op\":\"println\""), "{traced}");
    // ARTIFACT-CLAIM-1: an artifact whose manifest says `Clock` where its code performs `Write` (the same length, so the
    // section stays well-formed — its code hash binds only its code, so it still verifies) is refused at its first
    // undeclared effect, DL1202, and nothing is written; under `--assert-trace` the trace's dispatch record is DL1101 too.
    let dwx = std::fs::read(dir.join("w.dwx")).unwrap();
    let at = dwx.windows(7).position(|w| w == b"\"Write\"").expect("the manifest names Write");
    let mut lie = dwx.clone();
    lie[at..at + 7].copy_from_slice(b"\"Clock\"");
    std::fs::write(dir.join("lie.dwx"), &lie).unwrap();
    for (extra, exit) in [(None, 1), (Some("--assert-trace"), 3)] {
        let _ = std::fs::remove_file(dir.join("out").join("log.txt"));
        let mut args = vec!["run", "lie.dwx", "--grant", "console", "--grant", "fs.write=./out"];
        args.extend(extra);
        let o = delulu_in(&dir, &args);
        let e = String::from_utf8_lossy(&o.stderr).to_string();
        assert_eq!(o.status.code(), Some(exit), "{extra:?}: {e}");
        assert!(e.contains("error[DL1202]") && e.contains("performed `Write`"), "{extra:?}: {e}");
        assert!(extra.is_none() || e.contains("DL1101"), "{e}");
        assert!(!dir.join("out").join("log.txt").exists() && stdout(&o).is_empty(), "{extra:?}: nothing it did not declare ran");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dwx_run_without_console_grant_is_refused() {
    let dwx = temp_dwx("nogrant");
    let dwx_s = dwx.to_string_lossy().to_string();
    assert!(delulu(&["build", "examples/hello_wasm.delulu", "--target", "wasm", "-o", &dwx_s]).status.success());
    let o = delulu(&["run", &dwx_s]);
    assert!(!o.status.success(), "an ungranted console must be refused when running the artifact");
    let _ = std::fs::remove_file(&dwx);
}

#[test]
fn tampered_dwx_is_rejected_dl1202() {
    // Alter a byte of the artifact after it's built: re-verification catches the code/manifest
    // mismatch before the program ever runs.
    let dwx = temp_dwx("tampered");
    let dwx_s = dwx.to_string_lossy().to_string();
    assert!(delulu(&["build", "examples/hello_wasm.delulu", "--target", "wasm", "-o", &dwx_s]).status.success());

    let mut bytes = std::fs::read(&dwx).expect("read dwx");
    bytes[20] ^= 0xff; // a code-region byte (well before the trailing authority section)
    std::fs::write(&dwx, &bytes).expect("write tampered dwx");

    let o = delulu(&["run", &dwx_s, "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1202"), "tampered artifact must be DL1202: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
    let _ = std::fs::remove_file(&dwx);
}

#[test]
fn run_engine_wasm_on_a_secret_program_is_dl1205() {
    // A program that mints/exposes a secret must not compile to WASM — the CLI reports DL1205 and
    // runs nothing, so secret bytes never reach a guest. (Omitting --engine wasm runs it on the interp.)
    let path = std::env::temp_dir().join(format!("delulu_secret_{}.delulu", std::process::id()));
    std::fs::write(
        &path,
        "module m\nfn main(root: Root) ! {Declassify} { let key = root.secret(\"TOKEN\")\n let d = root.declassify()\n let _r = key.expose(d) }\n",
    )
    .expect("write secret program");
    let p = path.to_string_lossy().to_string();
    let o = delulu(&["run", &p, "--engine", "wasm", "--json"]);
    assert!(stdout(&o).contains("DL1205"), "expected DL1205: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn non_artifact_dwx_is_rejected_dl1202() {
    // A .dwx that isn't a Delulu artifact at all (random bytes) is refused, not run.
    let dwx = temp_dwx("garbage");
    std::fs::write(&dwx, b"this is not a wasm module").expect("write garbage");
    let dwx_s = dwx.to_string_lossy().to_string();
    let o = delulu(&["run", &dwx_s, "--json"]);
    assert!(stdout(&o).contains("DL1202"), "a non-artifact .dwx must be DL1202: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
    let _ = std::fs::remove_file(&dwx);
}

fn delulu_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).current_dir(cwd).args(args).output().expect("failed to run delulu")
}

/// RW 3.4: `root.fs_write`, `write_text` and `append_text` on the WASM engine, end to end through the command line —
/// the interpreter's output, the same bytes on disk, and the same effect trace byte for byte (the dispatch record, with
/// where the path pointed and the scope; a refusal's record after it). A write that escapes the scope is DL0904 on both
/// and leaves no file; an ungranted mint is DL0703 on both, in the interpreter's words.
#[test]
fn file_writes_on_the_wasm_engine_match_the_interpreter() {
    let dir = std::env::temp_dir().join(format!("delulu_fswrite_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("out")).unwrap();
    let program = "module m\n\nfn main(root: Root) ! {Read, Write} {\n  let out = root.console()\n  let w = root.fs_write(\"./out\")\n  \
                   match w.write_text(\"log.txt\", \"started\") {\n    Ok(u) => out.println(\"wrote\"),\n    Err(e) => out.println(\"could not\"),\n  }\n  \
                   match w.append_text(\"log.txt\", \" and more\") {\n    Ok(u) => out.println(\"appended\"),\n    Err(e) => out.println(\"could not\"),\n  }\n  \
                   match w.write_text(\"no/such/dir.txt\", \"x\") {\n    Ok(u) => out.println(\"deep\"),\n    Err(e) => out.println(match e { NotFound => \"not found\", Denied => \"denied\", Other(m) => \"other\" }),\n  }\n}\n";
    let escape = "module m\nfn main(root: Root) ! {Write} {\n  let w = root.fs_write(\"./out\")\n  let r = w.write_text(\"../escaped.txt\", \"x\")\n  root.console().println(\"after\")\n}\n";
    std::fs::write(dir.join("w.delulu"), program).unwrap();
    std::fs::write(dir.join("esc.delulu"), escape).unwrap();
    let run = |file: &str, engine: &str, grants: &[&str]| {
        let _ = std::fs::remove_file(dir.join("out").join("log.txt"));
        let trace = format!("trace_{file}_{engine}.jsonl");
        let mut args = vec!["run", file, "--trace-effects", "--trace-out", &trace, "--grant", "console"];
        args.extend_from_slice(grants);
        if engine == "wasm" {
            args.extend_from_slice(&["--engine", "wasm"]);
        }
        let o = delulu_in(&dir, &args);
        let written = std::fs::read_to_string(dir.join("out").join("log.txt")).ok();
        let traced = std::fs::read_to_string(dir.join(&trace)).unwrap_or_default();
        (o, written, traced)
    };
    let (i, i_file, i_trace) = run("w.delulu", "interp", &["--grant", "fs.write=./out"]);
    let (w, w_file, w_trace) = run("w.delulu", "wasm", &["--grant", "fs.write=./out"]);
    assert!(
        i.status.success() && w.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&i.stderr),
        String::from_utf8_lossy(&w.stderr)
    );
    assert_eq!(stdout(&w), stdout(&i));
    assert_eq!(stdout(&w), "wrote\nappended\nnot found\n");
    assert_eq!((w_file.as_deref(), i_file.as_deref()), (Some("started and more"), Some("started and more")));
    assert_eq!(w_trace, i_trace, "the effect traces");
    assert_eq!(w_trace.matches("\"cap_kind\":\"FsWrite\"").count(), 3, "three writes recorded: {w_trace}");

    let (i, _, i_trace) = run("esc.delulu", "interp", &["--grant", "fs.write=./out"]);
    let (w, _, w_trace) = run("esc.delulu", "wasm", &["--grant", "fs.write=./out"]);
    let (ie, we) = (String::from_utf8_lossy(&i.stderr).to_string(), String::from_utf8_lossy(&w.stderr).to_string());
    assert!(ie.contains("DL0904") && we.contains("DL0904") && we.contains("escapes the granted scope"), "{ie}\n{we}");
    assert!(!dir.join("escaped.txt").exists(), "nothing was written outside the scope");
    assert_eq!(w_trace, i_trace, "the escape's traces: the dispatch and its refusal");
    assert!(w_trace.contains("write_text.refused"), "{w_trace}");

    let (i, _, _) = run("w.delulu", "interp", &[]);
    let (w, _, _) = run("w.delulu", "wasm", &[]);
    let (ie, we) = (String::from_utf8_lossy(&i.stderr).to_string(), String::from_utf8_lossy(&w.stderr).to_string());
    let words = "filesystem write of `./out` was not granted — pass `--grant fs.write=./out`";
    assert!(ie.contains("DL0703") && ie.contains(words) && we.contains("DL0703") && we.contains(words), "{ie}\n{we}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A read through a link whose target does not exist is refused on both engines in the interpreter's words, with the
/// same trace (the red-team pass of on-demand routine run 5, F1): the WASM read checked its scope with a copy of
/// `resolve_in_scope`'s rule, which lost SYMLINK-DANGLE-1's explanation from the refusal and from its trace record.
#[cfg(unix)]
#[test]
fn a_read_through_a_dangling_link_is_refused_in_the_interpreters_words_on_wasm() {
    let dir = std::env::temp_dir().join(format!("delulu_fsdangle_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scope")).unwrap();
    std::fs::create_dir_all(dir.join("outside")).unwrap();
    std::os::unix::fs::symlink("../outside/created.txt", dir.join("scope").join("dang")).unwrap();
    let program = "module m\n\nfn main(root: Root) ! {Read, Write} {\n  let r = root.fs_read(\"./scope\")\n  \
                   match r.read_text(\"dang\") {\n    Ok(s) => root.console().println(\"read\"),\n    \
                   Err(e) => root.console().println(\"err\"),\n  }\n}\n";
    std::fs::write(dir.join("r.delulu"), program).unwrap();
    let run = |engine: &str| {
        let trace = format!("trace_{engine}.jsonl");
        let mut args =
            vec!["run", "r.delulu", "--trace-effects", "--trace-out", &trace, "--grant", "console", "--grant", "fs.read=./scope"];
        if engine == "wasm" {
            args.extend_from_slice(&["--engine", "wasm"]);
        }
        let o = delulu_in(&dir, &args);
        let traced = std::fs::read_to_string(dir.join(&trace)).unwrap_or_default();
        (String::from_utf8_lossy(&o.stderr).to_string(), traced, o.status.code())
    };
    let (ie, i_trace, i_code) = run("interp");
    let (we, w_trace, w_code) = run("wasm");
    let words = "path `dang` escapes the granted scope — a component is a symbolic link whose target does not exist";
    assert!(ie.contains("DL0904") && ie.contains(words), "{ie}");
    assert!(we.contains("DL0904") && we.contains(words), "the interpreter's words on WASM:\n{we}");
    assert_eq!(w_code, i_code);
    assert_eq!(w_trace, i_trace, "the traces: the dispatch and its refusal");
    assert!(w_trace.contains("read_text.refused"), "{w_trace}");
    assert!(!dir.join("outside").join("created.txt").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

/// A loopback listener that keeps the first bytes of each of the first `n` connections, then hangs up on it. It gives
/// up after `wait` with no connection, so a run that never connects cannot hang the test.
fn listen(n: usize, wait: std::time::Duration) -> (u16, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    use std::io::Read;
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let port = l.local_addr().unwrap().port();
    let h = std::thread::spawn(move || {
        let mut seen = Vec::new();
        let mut start = std::time::Instant::now();
        while seen.len() < n {
            match l.accept() {
                Ok((mut s, _)) => {
                    s.set_nonblocking(false).unwrap();
                    s.set_read_timeout(Some(std::time::Duration::from_secs(10))).unwrap();
                    let mut buf = vec![0u8; 5];
                    let mut got = 0;
                    while got < buf.len() {
                        match s.read(&mut buf[got..]) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => got += n,
                        }
                    }
                    buf.truncate(got);
                    seen.push(buf);
                    start = std::time::Instant::now();
                }
                Err(_) if start.elapsed() < wait => std::thread::sleep(std::time::Duration::from_millis(5)),
                Err(_) => break,
            }
        }
        seen
    });
    (port, h)
}

/// RW 3.4: `root.http(hosts)` and `Cap[Http].get(url)` on the WASM engine, end to end on loopback — the interpreter's
/// mint, order, egress client and words, since the host calls the interpreter's own. A URL that is not `https://` is
/// answered `Refused` before anything is reached; a granted request (loopback, so `net.special=`) leaves as TLS to a
/// listener that hangs up, the program told `Other`; a host outside the allowlist faults DL0904, its refusal recorded
/// after its dispatch; a host the grant does not cover is DL0703 at the mint. Output, outcome, effect trace and the run
/// report's egress reasons, engine against engine.
#[test]
fn http_on_the_wasm_engine_matches_the_interpreter() {
    let dir = std::env::temp_dir().join(format!("delulu_wasm_http_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // One listener for both engines' runs, so the two read one program, its spans and its port alike.
    let (port, seen) = listen(2, std::time::Duration::from_secs(20));
    let program = format!(
        "module fetch\n\nfn show(r: Result[Str, NetErr], out: Cap[Console]) ! {{Write}} {{\n  match r {{\n    \
         Ok(body) => out.println(\"delivered \" + body),\n    \
         Err(e) => out.println(match e {{ Refused => \"refused\", Timeout => \"timeout\", Other(m) => \"other\" }}),\n  }}\n}}\n\n\
         fn main(root: Root) ! {{Net, Write}} {{\n  let out = root.console()\n  let h = root.http([\"127.0.0.1\"])\n  \
         show(h.get(\"http://127.0.0.1/plain\"), out)\n  \
         show(h.get(\"https://127.0.0.1:{port}/probe\"), out)\n  \
         let r = h.get(\"https://elsewhere.example/\")\n  out.println(\"not reached\")\n}}\n"
    );
    std::fs::write(dir.join("fetch.delulu"), &program).unwrap();
    std::fs::write(
        dir.join("mint.delulu"),
        "module mint\nfn main(root: Root) ! {Net} {\n  let h = root.http([\"evil.example\"])\n}\n",
    )
    .unwrap();
    let run = |file: &str, engine: &str, grant: &str| {
        let trace = format!("t_{engine}.jsonl");
        let _ = std::fs::remove_file(dir.join("rep.json"));
        let mut args = vec!["run", file, "--grant", "console", "--grant", grant, "--trace-effects", "--trace-out", &trace];
        args.extend_from_slice(&["--report-out", "rep.json"]);
        if engine == "wasm" {
            args.extend_from_slice(&["--engine", "wasm"]);
        }
        let o = delulu_in(&dir, &args);
        let traced = std::fs::read_to_string(dir.join(&trace)).unwrap_or_default();
        let reasons: Vec<String> = std::fs::read_to_string(dir.join("rep.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(t.trim()).ok())
            .and_then(|v| v["egress"]["records"].as_array().cloned())
            .unwrap_or_default()
            .iter()
            .map(|r| r["reason"].as_str().unwrap_or("").to_string())
            .collect();
        (o, traced, reasons)
    };
    let err = |o: &Output| String::from_utf8_lossy(&o.stderr).to_string();

    let (i, i_trace, i_reasons) = run("fetch.delulu", "interp", "net.special=127.0.0.1");
    let (w, w_trace, w_reasons) = run("fetch.delulu", "wasm", "net.special=127.0.0.1");
    let first = seen.join().unwrap();
    assert_eq!(stdout(&i), "refused\nother\n", "the interpreter: {}", err(&i));
    assert_eq!(stdout(&w), stdout(&i), "the WASM engine's output: {}", err(&w));
    assert_eq!((w.status.code(), i.status.code()), (Some(1), Some(1)));
    for e in [err(&i), err(&w)] {
        assert!(
            e.contains("DL0904") && e.contains("host of `https://elsewhere.example/` is not in the granted allowlist"),
            "{e}"
        );
    }
    assert_eq!(first.len(), 2, "each engine's request reached the listener: {first:02x?}");
    assert!(first.iter().all(|b| b.len() >= 3 && b[0] == 0x16 && b[1] == 0x03), "each left as TLS: {first:02x?}");
    assert_eq!(w_trace, i_trace, "the effect traces");
    assert_eq!(w_trace.matches("\"cap_kind\":\"Http\"").count(), 4, "three dispatches and one refusal: {w_trace}");
    assert!(w_trace.contains("\"op\":\"get.refused\""), "{w_trace}");
    assert_eq!(w_reasons, i_reasons, "the run report's egress reasons");
    assert_eq!(i_reasons.first().map(String::as_str), Some("scheme"), "{i_reasons:?}");

    // A host the grant does not cover is refused at the mint, in the interpreter's words.
    let (i, _, _) = run("mint.delulu", "interp", "net=example.com");
    let (w, _, _) = run("mint.delulu", "wasm", "net=example.com");
    let words = "`network host` was not granted to this program — pass `--grant net=HOST`";
    for e in [err(&i), err(&w)] {
        assert!(e.contains("DL0703") && e.contains(words), "{e}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// RW 3.21 (DWX-ACTORS-1): `build --target wasm` wrote a `.dwx` of an actor program, and `run` of it failed "actor index 1
/// out of range" — the `.dwx` route had no actor table. The module now carries its own (each actor's field count an exported
/// global beside its members), and the artifact runs as its source does: the same quiescence report as the source on the
/// WASM engine and on the interpreter, an actor dying in a turn included. One built before — no field counts, assembled here
/// by renaming the export — is refused before it runs, DL1202, never out of range.
#[test]
fn an_actor_programs_dwx_runs_as_its_source_does() {
    let dir = std::env::temp_dir().join(format!("delulu_dwx_actors_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = "module a
actor W {
  var n: Int
  var m: Int
  new() { self.n = 0
 self.m = 0 }
                 be bump(k: Int) ! {Async} { self.n = self.n + k
 if k > 0 { self.bump(k - 1) } }
                 be boom(d: Int) { self.m = 100 / d }
}
actor R {
  var peer: tag W
  new(p: tag W) { self.peer = p }
                 be go() ! {Async} { self.peer.bump(2) }
}
               fn main(root: Root) ! {Async} {
  let w = spawn W()
  w.bump(3)
  let r = spawn R(w)
  r.go()
                 let v = spawn W()
  v.boom(0)
  v.bump(1)
}
";
    std::fs::write(dir.join("a.delulu"), src).unwrap();
    let o = delulu_in(&dir, &["build", "a.delulu", "--target", "wasm", "-o", "a.dwx"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let report = |args: &[&str]| {
        let mut all = vec!["run"];
        all.extend_from_slice(args);
        all.extend_from_slice(&["--on-quiesce", "report"]);
        let o = delulu_in(&dir, &all);
        let e = String::from_utf8_lossy(&o.stderr).to_string();
        assert!(o.status.success(), "{args:?}: {e}");
        e.lines().filter(|l| l.starts_with("quiesce:") || l.starts_with("actors:")).collect::<Vec<_>>().join(
            "
",
        )
    };
    let artifact = report(&["a.dwx"]);
    assert!(artifact.contains("quiesce: 2 surviving actor(s)") && artifact.contains("actors: 1 died"), "{artifact}");
    assert_eq!(artifact, report(&["a.delulu", "--engine", "wasm"]), "the artifact and its source on the WASM engine");
    assert_eq!(artifact, report(&["a.delulu"]), "and on the interpreter");

    // One built before RW 3.21 carries no field counts (here: the export renamed, the artifact assembled as `build` would).
    let checked = delulu_check::check_source(0, src);
    let mut wasm = delulu_wasm::compile_checked(&checked.module, &checked.result).expect("the actor program compiles");
    for at in (0..wasm.len().saturating_sub(7)).filter(|&i| &wasm[i..i + 7] == b"#fields").collect::<Vec<_>>() {
        wasm[at..at + 7].copy_from_slice(b"#fieldz");
    }
    std::fs::write(dir.join("old.dwx"), delulu_wasm::embed_authority(&wasm, &serde_json::json!({ "effects": ["Async"] })))
        .unwrap();
    let o = delulu_in(&dir, &["run", "old.dwx"]);
    let e = String::from_utf8_lossy(&o.stderr).to_string();
    assert_eq!(o.status.code(), Some(1), "{e}");
    assert!(e.contains("error[DL1202]") && e.contains("carries no field count") && !e.contains("out of range"), "{e}");
    let _ = std::fs::remove_dir_all(&dir);
}
