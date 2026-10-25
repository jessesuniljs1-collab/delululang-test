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

/// A function whose last statement is `return`, and `return` in a match arm or an `if` used as a value (on-demand routine
/// run 6, the campaign's A9): refused DL0401 and DL0202 until 2026-10-25; they run now, and print the same on both engines
/// (the WASM codegen let a returning branch decide a `match`'s or an `if`'s type, and refused the next branch DL1201).
#[test]
fn a_function_ending_in_return_runs_alike_on_both_engines() {
    let dir = std::env::temp_dir().join(format!("delulu-wasm-return-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("ret.delulu");
    std::fs::write(
        &f,
        "module ret\n\nfn half(x: Int) -> Int {\n    if x < 0 {\n        return 0\n    }\n    return x / 2\n}\n\n\
         fn f(c: Bool) -> Str {\n    let y = if c { return \"one\" } else { 2 }\n    str(y + 10)\n}\n\n\
         fn g(o: Option[Str]) -> Result[Str, Int] {\n    let s = match o {\n        None => return Err(7),\n        Some(v) => v\n    }\n    Ok(s + \"!\")\n}\n\n\
         fn show(r: Result[Str, Int]) -> Str {\n    match r {\n        Ok(s) => s,\n        Err(n) => \"err \" + str(n)\n    }\n}\n\n\
         fn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(str(half(8)))\n    out.println(str(half(-3)))\n    \
         out.println(f(true))\n    out.println(f(false))\n    out.println(show(g(None)))\n    out.println(show(g(Some(\"late\"))))\n}\n",
    )
    .unwrap();
    let f = f.to_string_lossy().to_string();
    let interp = delulu(&["run", &f, "--grant", "console"]);
    assert_eq!(
        (interp.status.code(), stdout(&interp).as_str()),
        (Some(0), "4\n0\none\n12\nerr 7\nlate!\n"),
        "{}",
        String::from_utf8_lossy(&interp.stderr)
    );
    let wasm = delulu(&["run", &f, "--grant", "console", "--engine", "wasm"]);
    assert_eq!((wasm.status.code(), stdout(&wasm)), (Some(0), stdout(&interp)), "{}", String::from_utf8_lossy(&wasm.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

/// An empty list filled later with a sum or a record (the campaign's A10, on-demand routine run 6): `let out = []`, then
/// `out.push(TNum(1))`, was refused on WASM — DL1201, "an empty list whose element type the backend was not given", with
/// no line — though the checker had inferred `List[Tok]`; an annotation or a first item ran. It runs now and prints what
/// the interpreter prints: a sum with payloads, a sum of constructors only returned inside a `Result`, and a record.
#[test]
fn an_empty_list_filled_later_with_sums_or_records_runs_alike_on_both_engines() {
    let dir = std::env::temp_dir().join(format!("delulu-wasm-empty-sums-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("toks.delulu");
    std::fs::write(
        &f,
        r#"module toks

type Tok = TNum(Int) | TPlus | TEnd

type Dir = North | South

type P {
  x: Int,
  name: Str,
}

fn lex() -> List[Tok] {
    let out = []
    out.push(TNum(1))
    out.push(TPlus)
    out.push(TNum(2))
    out.push(TEnd)
    out
}

fn dirs(n: Int) -> Result[List[Dir], Str] {
    let ds = []
    var i = 0
    while i < n {
        if i % 2 == 0 { ds.push(North) } else { ds.push(South) }
        i = i + 1
    }
    Ok(ds)
}

fn show(t: Tok) -> Str {
    match t {
        TNum(n) => str(n),
        TPlus => "+",
        TEnd => "."
    }
}

fn way(d: Dir) -> Str {
    match d {
        North => "north",
        South => "south"
    }
}

fn main(root: Root) ! {Write} {
    let out = root.console()
    var s = ""
    for t in lex() { s = s + show(t) }
    out.println(s)
    match dirs(3) {
        Ok(ds) => out.println(str(ds.len()) + " " + way(ds[1])),
        Err(e) => out.println(e)
    }
    let ps = []
    ps.push(P { x: 3, name: "c" })
    out.println(ps[0].name + str(ps[0].x))
}
"#,
    )
    .unwrap();
    let f = f.to_string_lossy().to_string();
    let interp = delulu(&["run", &f, "--grant", "console"]);
    assert_eq!(
        (interp.status.code(), stdout(&interp).as_str()),
        (Some(0), "1+2.\n3 south\nc3\n"),
        "{}",
        String::from_utf8_lossy(&interp.stderr)
    );
    let wasm = delulu(&["run", &f, "--grant", "console", "--engine", "wasm"]);
    assert_eq!((wasm.status.code(), stdout(&wasm)), (Some(0), stdout(&interp)), "{}", String::from_utf8_lossy(&wasm.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A construct the WASM backend refuses is refused AT ITS LINE (the campaign's A10, second half): DL1201 named no line
/// and carried no span, so a program of any size left its author to guess which expression the backend could not
/// compile, while every checker refusal points at its own. The innermost expression it refused is the span now — the
/// list of closures on line 6, not the `for` around it or `main` — on `run --engine wasm` (human and `--json`) and on
/// `build --target wasm`, and the message is the one it was.
#[test]
fn a_wasm_refusal_names_the_line_it_refuses() {
    let dir = std::env::temp_dir().join(format!("delulu-wasm-refusal-line-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("noline.delulu");
    std::fs::write(
        &f,
        "module noline\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    var t = 0\n    for g in [fn(x: Int) -> Int { x }] { t = t + g(1) }\n    out.println(\"done\")\n}\n",
    )
    .unwrap();
    let f = f.to_string_lossy().to_string();
    let interp = delulu(&["run", &f, "--grant", "console"]);
    assert_eq!((interp.status.code(), stdout(&interp).as_str()), (Some(0), "done\n"), "{}", String::from_utf8_lossy(&interp.stderr));

    let run = delulu(&["run", &f, "--grant", "console", "--engine", "wasm"]);
    let err = String::from_utf8_lossy(&run.stderr).to_string();
    assert_eq!(run.status.code(), Some(1), "{err}");
    assert!(err.contains("error[DL1201]: WASM codegen does not support this expression form"), "the message is the one it was:\n{err}");
    assert!(err.contains("noline.delulu:6:14"), "the refusal names the list of closures, line 6 column 14:\n{err}");

    let json = delulu(&["run", &f, "--grant", "console", "--engine", "wasm", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&json.stdout).expect("a JSON envelope");
    let d = &v["diagnostics"][0];
    assert_eq!(d["code"], "DL1201", "{v}");
    assert_eq!((d["spans"][0]["start"]["line"].as_u64(), d["spans"][0]["start"]["col"].as_u64()), (Some(6), Some(14)), "{v}");

    let build = delulu(&["build", &f, "--target", "wasm", "--out", &dir.join("noline.dwx").to_string_lossy()]);
    let err = String::from_utf8_lossy(&build.stderr).to_string();
    assert_eq!(build.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1201") && err.contains("noline.delulu:6:14"), "`build --target wasm` names the line too:\n{err}");
    let _ = std::fs::remove_dir_all(&dir);
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

/// D-V2-174: `demo.delulu` mints a secret, which a WASM program holds as a handle — the host keeps the secret — so all of
/// it runs on WASM, printing what the interpreter prints, from the source and from the `.dwx` built from it; the secret's
/// value is in neither the artifact nor the output. (It stopped at its `root.secret` line, DL1205, until then; a program
/// that `expose`s a secret still does, `run_engine_wasm_on_a_secret_program_is_dl1205`.) Ungranted, both engines refuse
/// the secret in the interpreter's words.
#[test]
fn demo_runs_on_wasm_as_on_the_interpreter_and_its_secret_stays_on_the_host() {
    const VALUE: &str = "demo-qv5k-value";
    let grant = format!("secret:API_KEY={VALUE}");
    let grants = ["--grant", "console", "--grant", "fs.read=./config", "--grant", grant.as_str()];
    let run = |target: &str, extra: &[&str]| delulu(&[&["run", target][..], &grants, extra].concat());
    let interp = run("examples/demo.delulu", &[]);
    assert_eq!(interp.status.code(), Some(0), "{}", String::from_utf8_lossy(&interp.stderr));
    assert!(stdout(&interp).contains("fib(10) = 55"), "{}", stdout(&interp));
    let wasm = run("examples/demo.delulu", &["--engine", "wasm"]);
    assert_eq!(wasm.status.code(), Some(0), "{}", String::from_utf8_lossy(&wasm.stderr));
    assert_eq!(stdout(&wasm), stdout(&interp), "the WASM engine prints what the interpreter prints");
    let dwx = temp_dwx("demo-secret");
    let dwx_path = dwx.to_string_lossy().to_string();
    let o = delulu(&["build", "examples/demo.delulu", "--target", "wasm", "--out", &dwx_path]);
    assert_eq!(o.status.code(), Some(0), "build: {}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));
    let built = run(&dwx_path, &[]);
    assert_eq!(built.status.code(), Some(0), "{}", String::from_utf8_lossy(&built.stderr));
    assert_eq!(stdout(&built), stdout(&interp), "the artifact prints what the interpreter prints");
    let bytes = std::fs::read(&dwx).unwrap();
    assert!(!bytes.windows(VALUE.len()).any(|w| w == VALUE.as_bytes()), "the secret is not in the artifact");
    for o in [&interp, &wasm, &built] {
        assert!(!format!("{}{}", stdout(o), String::from_utf8_lossy(&o.stderr)).contains(VALUE), "nor in what a run said");
    }
    // Ungranted: refused on both engines, in the interpreter's words.
    for engine in [&[][..], &["--engine", "wasm"][..]] {
        let o =
            delulu(&[&["run", "examples/demo.delulu", "--grant", "console", "--grant", "fs.read=./config"][..], engine].concat());
        assert_eq!(o.status.code(), Some(1), "{engine:?}");
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(err.contains("DL0703") && err.contains("secret `API_KEY` was not granted"), "{engine:?}: {err}");
    }
    let _ = std::fs::remove_file(&dwx);
}

/// D-V2-174: `Secret.verify` on the WASM engine compares two secrets the host holds, by the interpreter's own code — the
/// same answers and the same trace (`Declassify`, detail `«opaque»`) from the source and from its `.dwx` — and, since it
/// is a `Declassify` effect with no custody op, an artifact whose manifest does not declare `Declassify` is refused at it
/// (ARTIFACT-CLAIM-1, DL1202) before it answers. (Chapter five of the guide holds a function taking two secrets; once a
/// `Secret[Str]` parameter compiled, that chapter stopped at its `verify` until `verify` did.)
#[test]
fn a_secret_is_verified_on_wasm_as_on_the_interpreter_and_only_where_the_artifact_declares_it() {
    let dir = std::env::temp_dir().join(format!("delulu_wasm_verify_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let program = "module m\nfn same(a: Secret[Str], b: Secret[Str]) -> Bool ! {Declassify} { a.verify(b) }\n\
                   fn main(root: Root) ! {Declassify, Write} {\n    let out = root.console()\n    \
                   let a = root.secret(\"A\")\n    let b = root.secret(\"B\")\n    let c = root.secret(\"C\")\n    \
                   out.println(\"a~b \" + str(same(a, b)))\n    out.println(\"a~c \" + str(same(a, c)))\n}\n";
    std::fs::write(dir.join("v.delulu"), program).unwrap();
    let b = delulu_in(&dir, &["build", "v.delulu", "--target", "wasm", "-o", "v.dwx"]);
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    let grants = ["--grant", "console", "--grant", "secret:A=pw-1", "--grant", "secret:B=pw-1", "--grant", "secret:C=pw-2"];
    let mut traces = Vec::new();
    for (target, engine) in [("v.delulu", None), ("v.delulu", Some("wasm")), ("v.dwx", None)] {
        let trace = format!("t-{target}-{}.jsonl", engine.unwrap_or("default"));
        let mut args = vec!["run", target];
        args.extend(grants);
        args.extend(["--trace-effects", "--trace-out", &trace]);
        if let Some(e) = engine {
            args.extend(["--engine", e]);
        }
        let o = delulu_in(&dir, &args);
        assert!(o.status.success(), "{target} {engine:?}: {}", String::from_utf8_lossy(&o.stderr));
        assert_eq!(stdout(&o), "a~b true\na~c false\n", "{target} {engine:?}");
        traces.push(std::fs::read_to_string(dir.join(&trace)).expect("a trace"));
    }
    assert!(traces[0].contains("\"op\":\"verify\"") && traces[0].contains("«opaque»"), "{}", traces[0]);
    assert!(!traces[0].contains("pw-"), "no secret in the trace: {}", traces[0]);
    assert_eq!(traces[1], traces[0], "the WASM engine's trace is the interpreter's");
    assert_eq!(traces[2], traces[0], "the artifact's trace is the interpreter's");
    // The manifest made to declare `Write` alone — `"Declassify",` blanked, so it stays well-formed — is refused at the
    // first `verify`, before anything is printed.
    let dwx = std::fs::read(dir.join("v.dwx")).unwrap();
    let needle = b"\"Declassify\",";
    let at = dwx.windows(needle.len()).position(|w| w == needle).expect("the manifest names Declassify");
    let mut lie = dwx.clone();
    lie[at..at + needle.len()].fill(b' ');
    std::fs::write(dir.join("lie.dwx"), &lie).unwrap();
    let mut args = vec!["run", "lie.dwx"];
    args.extend(grants);
    let o = delulu_in(&dir, &args);
    let e = String::from_utf8_lossy(&o.stderr).to_string();
    assert_eq!(o.status.code(), Some(1), "{e}");
    assert!(e.contains("error[DL1202]") && e.contains("performed `Declassify`"), "{e}");
    assert!(stdout(&o).is_empty(), "nothing it did not declare ran: {}", stdout(&o));
    let _ = std::fs::remove_dir_all(&dir);
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
