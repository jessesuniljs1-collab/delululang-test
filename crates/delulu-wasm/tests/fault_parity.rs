//! Fault parity between the two engines (HARDENING_CAMPAIGN C20; Stage 3 invariant 15).
//!
//! Invariant 15 promises the interpreter and the WASM backend agree — same output, same exit,
//! same fault. For SUCCESS that is covered by the differential fuzz. For FAULTS it was not: the
//! differential harness classifies any `(Err, Err)` as agreement without checking the faults
//! match, and they did not. A program that divides by zero was `DL0902` on the interpreter and a
//! generic `DL0904` on the WASM engine, and a deep recursion printed one clean `DL0905` on the
//! interpreter but ~16,000 lines of guest backtrace (`<wasm function N>`, one per frame) on WASM.
//!
//! These tests pin the fixed behaviour: the deterministic arithmetic/recursion faults report the
//! SAME code on both engines, and a WASM trap never carries a backtrace. Both fail against the
//! pre-fix code.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, set_rand_seed, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_and_run_int, compile_module, run_main, HostConfig};

/// The DL code the interpreter faults with (or "OK" if it does not fault).
fn interp_fault_code(src: &str) -> String {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
    let g = Grants { console: true, ..Grants::default() };
    let interp = Interp::new(&checked.module);
    match interp.run_main(Value::Root(Rc::new(g.build_root()))) {
        Ok(_) => "OK".to_string(),
        Err(f) => f.code.to_string(),
    }
}

/// The WASM engine's fault detail (the string the CLI turns into a diagnostic). It embeds the DL
/// code and, after the fix, carries no backtrace.
fn wasm_fault_detail(src: &str) -> String {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).expect("compile to wasm");
    match run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }) {
        Ok(_) => "OK".to_string(),
        Err(e) => e.message(),
    }
}

const DIVZERO: &str = "module m\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(7 / 0)) }\n";
const OVERFLOW: &str =
    "module m\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(9223372036854775807 + 1)) }\n";
// `%` (2026-10-24, A3 of the agent test campaign): the WASM helper trapped `% 0` with `unreachable`, which the host can
// only read as overflow, so `7 % 0` was DL0902 on the interpreter and DL0901 on WASM. `INT_MIN % -1` is the overflow.
const REMZERO: &str = "module m\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(7 % 0)) }\n";
const REMOVERFLOW: &str =
    "module m\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str((0 - 9223372036854775807 - 1) % (0 - 1))) }\n";
const RECUR: &str =
    "module m\nfn cd(n: Int) -> Int { if n <= 0 { 0 } else { 1 + cd(n - 1) } }\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(str(cd(1000000))) }\n";

/// Run `body` on a thread with a main-thread-sized (16 MiB) stack.
///
/// The interpreter's recursion guard (`MAX_DEPTH = 10_000`) is calibrated for the CLI's ~8 MiB
/// main-thread stack, where a deep recursion reaches the guard and reports DL0905 cleanly. The Rust
/// test harness runs each test on a ~2 MiB thread, on which 10 000 interpreter frames overflow the
/// host stack *before* the guard fires — a real, separate finding (HARDENING_CAMPAIGN C21). To test
/// engine *parity* rather than that stack-size fragility, these run where the CLI runs: a big stack.
fn on_a_big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(body)
        .expect("spawn")
        .join()
        .expect("the parity body must not panic");
}

#[test]
fn the_two_engines_agree_on_arithmetic_fault_codes() {
    // Divide-by-zero and overflow fault without deep recursion, so both engines run directly here.
    // (Recursion is exercised WASM-side below; the interpreter's DL0905 has its own Stage-9 test
    // and would need a large stack to reach — see C21 and `on_a_big_stack`.)
    for (src, code) in [(DIVZERO, "DL0902"), (OVERFLOW, "DL0901"), (REMZERO, "DL0902"), (REMOVERFLOW, "DL0901")] {
        assert_eq!(interp_fault_code(src), code, "the interpreter must fault with {code}");
        let wasm = wasm_fault_detail(src);
        assert!(wasm.contains(code), "the WASM engine must report {code} for the same fault (parity, invariant 15), got: {wasm}");
    }
}

#[test]
fn a_wasm_stack_overflow_is_dl0905_with_no_backtrace() {
    on_a_big_stack(|| {
        // This is the headline C20 fix. A guest stack overflow used to surface as a generic DL0904
        // followed by ~16,000 lines of `<wasm function N>` — one per frame. It must now be the same
        // DL0905 the interpreter reports, on a single line, with no backtrace.
        let detail = wasm_fault_detail(RECUR);
        assert!(detail.contains("DL0905"), "a guest stack overflow must be DL0905, got: {detail}");
        assert_eq!(detail.lines().count(), 1, "the fault detail must be a single line, got:\n{detail}");
        assert!(!detail.contains("backtrace"), "no guest backtrace may appear in the message: {detail}");
    });
}

// The WASM engine's faults and refusals, held to the interpreter's — the defects on-demand routine run 4 found while
// verifying RW 3.4 and could not commit before its usage limit, and those its red-team pass named, redone on the laptop.
//
// Each test was red on the backend before its fix: WASM-NEG-1 (`-x` of `Int::MIN` wrapped), WASM-SHORT-1 (`&&`/`||`
// computed both sides), RAND-SPAN-1 (the interpreter's `rand.int` panicked for a span past `Int::MAX`) and
// WASM-REFUSAL-1 (a refusal masked by the trap that followed it). The interpreter is the reference: the same output,
// or both a fault with the same code.

/// Run `main` on both engines and return `(wasm, interp)`: the output, or the fault's text.
fn run_both(src: &str, cfg: HostConfig, g: Grants, seed: Option<u64>) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = run_main(&wasm, &cfg).map_err(|e| e.message());
    set_capture(true);
    if let Some(s) = seed {
        set_rand_seed(s);
    }
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    (w, i.map(|_| out))
}

fn console() -> (HostConfig, Grants) {
    (HostConfig { console: true, ..HostConfig::default() }, Grants { console: true, ..Default::default() })
}

#[test]
fn negating_int_min_faults_on_both_engines() {
    let src = "module m\nfn f(x: Int) -> Int { -x }\n";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let interp = Interp::new(&checked.module);
    for x in [5, -5, 0, i64::MAX, i64::MIN + 1] {
        assert_eq!(compile_and_run_int(&checked.module, "f", &[x]).ok(), Some(-x), "-({x}) on WASM");
        assert_eq!(interp.call_int_fn("f", &[x]).ok(), Some(-x), "-({x}) on the interpreter");
    }
    let w = compile_and_run_int(&checked.module, "f", &[i64::MIN]);
    let i = interp.call_int_fn("f", &[i64::MIN]);
    assert!(i.is_err(), "the interpreter faults negating Int::MIN: {i:?}");
    assert!(w.is_err(), "so must the WASM engine, not wrap to Int::MIN: {w:?}");
    assert!(format!("{w:?}").contains("DL0901"), "an overflow, as the interpreter says: {w:?}");
}

#[test]
fn and_and_or_short_circuit_on_both_engines() {
    let src = r#"module m

fn noisy(o: Cap[Console], v: Bool) -> Bool ! {Write} {
  o.println("right side ran")
  v
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let z = 0
  let a = z != 0 && 10 / z > 1
  let b = z == 0 || 10 / z > 1
  let c = z == 0 && 10 / (z + 2) > 1
  let d = z != 0 || 10 % (z + 3) == 1
  out.println(str(a) + " " + str(b) + " " + str(c) + " " + str(d))
  let e = false && noisy(out, true)
  let f = true || noisy(out, false)
  let g = true && noisy(out, false)
  let h = false || noisy(out, true)
  out.println(str(e) + " " + str(f) + " " + str(g) + " " + str(h))
  var i = 0
  while i < 5 && 100 / (5 - i) > 0 {
    i = i + 1
  }
  out.println(str(i))
}
"#;
    let (cfg, g) = console();
    let (w, i) = run_both(src, cfg, g, None);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    assert_eq!(
        w.unwrap(),
        "false true true true\nright side ran\nright side ran\nfalse true false true\n5\n",
        "each right side ran only where the left did not decide"
    );
}

/// `rand.int(lo, hi)` for a span past `Int::MAX`: the interpreter computed `hi - lo` in `Int` and a debug build
/// panicked ("attempt to subtract with overflow") — the WASM host computed it modulo 2^64. Seeded, both engines
/// draw the same values, every one inside `lo..hi`.
#[test]
fn rand_int_over_a_span_past_int_max_draws_the_same_values_on_both_engines() {
    let src = r#"module m

fn main(root: Root) ! {Write, Rand} {
  let out = root.console()
  let r = root.rand()
  let min = 0 - 9223372036854775807 - 1
  var k = 0
  var line = ""
  while k < 8 {
    let a = r.int(0 - 1, 9223372036854775807)
    let b = r.int(min, 9223372036854775807)
    let c = r.int(min, 0)
    if a < 0 - 1 || c >= 0 { line = line + "OUT " }
    line = line + str(a) + "," + str(b) + "," + str(c) + ";"
    k = k + 1
  }
  out.println(line)
}
"#;
    for seed in [1u64, 42, 0xDEAD_BEEF] {
        let cfg = HostConfig { console: true, rand: true, rand_seed: Some(seed), ..HostConfig::default() };
        let g = Grants { console: true, rand: true, ..Default::default() };
        let (w, i) = run_both(src, cfg, g, Some(seed));
        assert_eq!(w, i, "two-engine parity broken under seed {seed}");
        let out = w.unwrap();
        assert!(!out.contains("OUT"), "every draw inside its range: {out}");
    }
}

/// `main` drawing once from `r.int(lo, hi)`, the bounds written as given.
fn one_draw(lo: &str, hi: &str) -> String {
    format!(
        "module m\n\nfn main(root: Root) ! {{Write, Rand}} {{\n  let out = root.console()\n  let r = root.rand()\n  out.println(str(r.int({lo}, {hi})))\n}}\n"
    )
}

fn rand_granted(seed: u64) -> (HostConfig, Grants) {
    (
        HostConfig { console: true, rand: true, rand_seed: Some(seed), ..HostConfig::default() },
        Grants { console: true, rand: true, ..Default::default() },
    )
}

/// A15 (the agent test campaign, 2026-10-24): `r.int(lo, hi)` draws from `lo` up to but not including `hi`, so a range
/// with `hi <= lo` holds no value. Both engines faulted as DL0904, "capability scope violation" — the code an operator
/// reads as a path or host outside a grant — though nothing reached outside one. It is DL0908, an argument outside what
/// the operation accepts, with the same text on both engines (D-V2-190).
#[test]
fn an_empty_rand_range_is_dl0908_on_both_engines() {
    let min = "0 - 9223372036854775807 - 1";
    for (lo, hi) in [("5", "5"), ("6", "5"), ("0", min), ("9223372036854775807", min)] {
        let (cfg, g) = rand_granted(7);
        let (w, i) = run_both(&one_draw(lo, hi), cfg, g, Some(7));
        let (w, i) = (w.expect_err("the WASM engine faults"), i.expect_err("the interpreter faults"));
        for (engine, text) in [("WASM", &w), ("interpreter", &i)] {
            assert!(text.contains("DL0908"), "{engine}: r.int({lo}, {hi}) is DL0908: {text}");
            assert!(!text.contains("DL0904"), "{engine}: and not a scope violation: {text}");
            assert!(text.contains("up to but not including"), "{engine}: the text says the range is half-open: {text}");
        }
    }
}

/// The other side of A15: the narrowest range, `r.int(5, 6)`, holds exactly one value and draws it on both engines.
#[test]
fn a_one_wide_rand_range_draws_its_one_value_on_both_engines() {
    for seed in [1u64, 7, 0xDEAD_BEEF] {
        let (cfg, g) = rand_granted(seed);
        let (w, i) = run_both(&one_draw("5", "6"), cfg, g, Some(seed));
        assert_eq!(w, i, "two-engine parity under seed {seed}");
        assert_eq!(w.as_deref().map(str::trim), Ok("5"), "under seed {seed}");
    }
}

/// A refused host call returns a placeholder and the guest runs on; matching on the refused read's result then
/// trapped as an access out of bounds, and that trap was reported instead of the refusal.
#[test]
fn a_refusal_is_reported_even_when_the_guest_traps_after_it() {
    let dir = std::env::temp_dir().join(format!("delulu_wasm_refusal_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "hello\n").unwrap();
    let root = dir.to_string_lossy().replace('\\', "/");
    let src = format!(
        r#"module m

fn show(r: Result[Str, IoErr], o: Cap[Console]) ! {{Write}} {{
  match r {{
    Ok(c) => o.println("ok " + str(c.len())),
    Err(e) => match e {{
      NotFound => o.println("NotFound"),
      _ => o.println("other"),
    }},
  }}
}}

fn main(root: Root) ! {{Write, Read}} {{
  let out = root.console()
  let fs = root.fs_read("{root}")
  show(fs.read_text("a.txt"), out)
  show(fs.read_text("nope.txt"), out)
  show(fs.read_text("../outside.txt"), out)
  out.println("not reached")
}}
"#
    );
    let granted = (
        HostConfig { console: true, fs_read_roots: vec![dir.clone()], ..HostConfig::default() },
        Grants { console: true, fs_read: vec![root.clone()], ..Default::default() },
    );
    let (cfg, g) = granted;
    let (w, i) = run_both(&src, cfg, g, None);
    let (cfg, g) = console();
    let (w_ungranted, i_ungranted) = run_both(&src, cfg, g, None);
    let _ = std::fs::remove_dir_all(&dir);
    let (w, i) = (w.expect_err("the escape is refused on WASM"), i.expect_err("and on the interpreter"));
    assert!(i.contains("DL0904"), "the interpreter: {i}");
    assert!(w.contains("DL0904") && w.contains("escapes"), "the WASM engine names the refusal, not the trap after it: {w}");
    let (w, i) = (w_ungranted.expect_err("ungranted on WASM"), i_ungranted.expect_err("ungranted on the interpreter"));
    assert!(i.contains("DL0703"), "the interpreter: {i}");
    assert!(w.contains("DL0703"), "the WASM engine names the ungranted read: {w}");
}

/// B6 (the agent test campaign): the WASM engine quotes a refused path as the interpreter does — as written, a
/// backslash once, a control character escaped. Both quoted it through `escape_debug`, which doubled every backslash.
#[test]
fn a_refused_path_is_quoted_as_written_on_the_wasm_engine() {
    let src = "module m\nfn main(root: Root) ! {Write, Read} {\n  let fs = root.fs_read(\"a\\\\b\\u{0}c\")\n  \
               root.console().println(\"not reached\")\n}\n";
    let (cfg, g) = console();
    let (w, i) = run_both(src, cfg, g, None);
    let (w, i) = (w.expect_err("refused on WASM"), i.expect_err("refused on the interpreter"));
    assert!(i.contains("DL0904"), "the interpreter: {i}");
    assert!(w.contains("DL0904: path `a\\b\\u{0}c` is refused: "), "the WASM engine: {w}");
}

/// B11 (the agent test campaign): a file operation the operating system refuses for a reason with no variant of its own
/// is `Other(m)`, and `m` was the OS's own message — on Windows *"The filename, directory name, or volume label syntax is
/// incorrect. (os error 123)"*, in the machine's language, and other words on Linux and macOS. `m` is now the kind of
/// failure, in the same words on every OS and both engines; nothing of the machine's own text reaches the program.
#[test]
fn an_os_refusal_reaches_the_program_in_the_same_words_everywhere() {
    let dir = std::env::temp_dir().join(format!("delulu-b11-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let root = dir.to_string_lossy().replace('\\', "/");
    let long = "a".repeat(300);
    let src = format!(
        "module m\nfn main(root: Root) ! {{Write, Read}} {{\n  let out = root.console()\n  let fs = root.fs_read(\"{root}\")\n  \
         let w = root.fs_write(\"{root}\")\n  \
         match fs.read_text(\"{long}\") {{\n    Ok(s) => out.println(\"read\"),\n    Err(e) => match e {{\n      \
         NotFound => out.println(\"NotFound\"),\n      Denied => out.println(\"Denied\"),\n      Other(m) => out.println(m),\n    }},\n  }}\n  \
         match w.write_text(\"{long}\", \"x\") {{\n    Ok(u) => out.println(\"wrote\"),\n    Err(e) => match e {{\n      \
         NotFound => out.println(\"NotFound\"),\n      Denied => out.println(\"Denied\"),\n      Other(m) => out.println(m),\n    }},\n  }}\n}}\n"
    );
    let cfg = HostConfig {
        console: true,
        fs_read_roots: vec![dir.clone()],
        fs_write_roots: vec![dir.clone()],
        ..HostConfig::default()
    };
    let g = Grants { console: true, fs_read: vec![root.clone()], fs_write: vec![root.clone()], ..Default::default() };
    let (w, i) = run_both(&src, cfg, g, None);
    let _ = std::fs::remove_dir_all(&dir);
    let both = Ok("invalid filename\ninvalid filename\n".to_string());
    assert_eq!(i, both, "the interpreter, reading and writing");
    assert_eq!(w, both, "the WASM engine, reading and writing");
}

/// RW 6.15: an ungranted mint is refused in the same words on both engines, its remedy included — the `--grant` that
/// covers it on an ordinary run, the delegation under `--lease` (a lease run refuses a local `--grant`, and both engines
/// named one). The WASM engine's console, clock and rand refusals named no remedy at all before.
#[test]
fn an_ungranted_mint_names_the_same_remedy_on_both_engines() {
    for lease in [None, Some("g_feed")] {
        for (mint, grant) in [
            ("root.console()", "console"),
            ("root.clock()", "clock"),
            ("root.rand()", "rand"),
            ("root.fs_read(\"./nowhere\")", "fs.read=./nowhere"),
            ("root.fs_write(\"./nowhere\")", "fs.write=./nowhere"),
            // B4: the entry asked for, never the placeholder `net=HOST`.
            ("root.http([\"example.com\"])", "net=example.com"),
        ] {
            let src = format!("module m\nfn main(root: Root) {{\n  let c = {mint}\n}}\n");
            let checked = check_source(0, &src);
            assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
            let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
            let cfg = HostConfig { lease: lease.map(str::to_string), ..HostConfig::default() };
            let w = run_main(&wasm, &cfg).expect_err("refused on WASM").message();
            let g = Grants { lease: lease.map(str::to_string), ..Default::default() };
            let i = Interp::new(&checked.module)
                .run_main(Value::Root(Rc::new(g.build_root())))
                .expect_err("refused on the interpreter");
            assert_eq!(i.code, "DL0703", "`{mint}`: {}", i.message);
            assert!(
                w.ends_with(&format!("DL0703: {}", i.message)),
                "the same words on both engines for `{mint}`:\n  wasm:   {w}\n  interp: {}",
                i.message
            );
            match lease {
                None => assert!(i.message.ends_with(&format!("pass `--grant {grant}`")), "{}", i.message),
                Some(node) => assert!(
                    i.message.contains(&format!("node `{node}`")) && i.message.contains(&format!("carry `{grant}`")),
                    "{}",
                    i.message
                ),
            }
        }
    }
}
