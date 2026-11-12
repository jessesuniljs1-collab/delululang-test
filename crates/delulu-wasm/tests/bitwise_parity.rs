//! RW 2.35 (D-V2-226) — the bitwise operators on the WASM engine, held to the interpreter: `&`, `|`, `^`, `~`, `<<`,
//! `>>` on an `Int`'s 64 bits, their compound forms, and the `0b` and `0o` literals beside `0x`. `&`, `|`, `^` and `~`
//! are WebAssembly's own instructions; a shift checks its count first, because `i64.shl` takes a count modulo 64, and a
//! count outside 0 to 63 records the interpreter's DL0908 (`prim::shift_count_fault`) and traps. So every answer and
//! every fault must be the interpreter's, word for word. Each expected answer below was worked by Python (with the
//! result taken to 64 bits where a left shift passes the 63rd bit), or by the reference evaluator in this file.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, gen, run_int_fn, run_main, HostConfig};

#[derive(Debug, PartialEq)]
enum Outcome {
    Printed(String),
    Fault(String, String),
}

/// Run `main` on both engines with the console: `(wasm, interpreter)`, a WASM fault read as the interpreter's own
/// `CODE: message` once its `WASM trap: ` is read off.
fn run_both(src: &str) -> (Outcome, Outcome) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = wasm_outcome(&wasm);
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let ran = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root())));
    let out = take_capture().unwrap_or_default();
    let i = match ran {
        Ok(_) => Outcome::Printed(out),
        Err(f) => Outcome::Fault(f.code.to_string(), f.message.to_string()),
    };
    (w, i)
}

fn wasm_outcome(wasm: &[u8]) -> Outcome {
    match run_main(wasm, &HostConfig { console: true, ..HostConfig::default() }) {
        Ok(out) => Outcome::Printed(out),
        Err(e) => {
            let text = e.message();
            let fault = text.strip_prefix("WASM trap: ").unwrap_or_else(|| panic!("a WASM fault is a trap: {text}"));
            let (code, msg) = fault.split_once(": ").unwrap_or_else(|| panic!("a WASM fault names its code: {text}"));
            Outcome::Fault(code.to_string(), msg.to_string())
        }
    }
}

fn program(body: &str) -> String {
    format!("module m\n\nfn main(root: Root) ! {{Write}} {{\n  let c = root.console()\n{body}\n}}\n")
}

/// Each line's answer on both engines, against Python's.
#[test]
fn every_bitwise_answer_is_the_same_on_both_engines_and_pythons() {
    let cases: &[(&str, &str)] = &[
        // The six operators (Python: 6 & 3 == 2, 6 | 3 == 7, 6 ^ 3 == 5, ~5 == -6, 1 << 4 == 16, 256 >> 4 == 16).
        ("c.println(f\"{6 & 3} {6 | 3} {6 ^ 3} {~5} {1 << 4} {256 >> 4}\")", "2 7 5 -6 16 16"),
        // `~` is every bit flipped: -n - 1, so no `Int` is outside its answer.
        ("c.println(f\"{~0} {~(0 - 1)} {~9223372036854775807}\")", "-1 0 -9223372036854775808"),
        // `>>` copies the sign in on the left (Python: -8 >> 1 == -4, -1 >> 63 == -1, -9 >> 2 == -3).
        ("c.println(f\"{-8 >> 1} {-1 >> 63} {-9 >> 2} {9 >> 2}\")", "-4 -1 -3 2"),
        // `<<` drops the bits moved past the 64th and never faults for that: 1 << 63 is Int's smallest value, and the
        // top bit of 0x7FFF_FFFF_FFFF_FFFF moves out (Python, taken to 64 bits: -2**63, -2, -2).
        ("c.println(f\"{1 << 63} {0x7FFF_FFFF_FFFF_FFFF << 1} {-1 << 1}\")", "-9223372036854775808 -2 -2"),
        // A shifted-out sign is gone, and the right shift then copies the new sign: (1 << 63) >> 63 is -1.
        ("c.println(f\"{1 << 63 >> 63} {1 << 62 >> 62}\")", "-1 1"),
        // The three bases and the `_` grouping (Python: 0xFF & 0b1010_1010 == 170, 0o17 | 0x10 == 31).
        ("c.println(f\"{0xFF & 0b1010_1010} {0o17 | 0x10} {0b0} {0o777} {0xdead_BEEF}\")", "170 31 0 511 3735928559"),
        ("let m = 9223372036854775807\n  c.println(f\"{m ^ m} {m & ~m} {m | ~m}\")", "0 0 -1"),
        // Precedence, as Python's and Rust's: every bitwise operator binds tighter than a comparison, looser than
        // arithmetic; `&` before `^` before `|`; the shifts between `&` and `+`; all left-associative.
        ("c.println(str(5 & 3 == 1))", "true"),
        ("c.println(f\"{1 | 2 ^ 3 & 4} {6 & 3 | 8} {1 << 2 + 1} {1 << 3 - 1 >> 1} {0xFF00 >> 8 & 0xFF}\")", "3 10 8 2 255"),
        ("c.println(f\"{~5 & 0xFF} {-(5 & 3)} {(0xDEAD_BEEF >> 16) ^ (0xDEAD_BEEF & 0xFFFF)}\")", "250 -1 24642"),
        ("c.println(str(6 & 3 != 0 && 1 << 2 == 4))", "true"),
        // A range is looser than `|`: 0..1 << 3 is 0..8.
        ("c.println(str((0..1 << 3).len()))", "8"),
        // The compound forms, by RW 2.34's rewrite (Python: 664).
        ("var x = 0xF0\n  x &= 0b1010_1010\n  x |= 0o7\n  x ^= 1\n  x <<= 4\n  x >>= 2\n  c.println(str(x))", "664"),
        // A loop that counts the bits set (Python: bin(0xF0F0).count('1') == 8).
        ("var n = 0xF0F0\n  var count = 0\n  while n != 0 {\n    count += n & 1\n    n >>= 1\n  }\n  c.println(str(count))", "8"),
        // In a function value, and through `fold` (Python: [i << i for i in [1, 2, 3]], 1 | 2 | 4 | 8).
        ("c.println(str([1, 2, 3].map(fn(i: Int) -> Int { i << i })))", "[2, 8, 24]"),
        ("c.println(str([1, 2, 4, 8].fold(0, fn(a: Int, b: Int) -> Int { a | b })))", "15"),
        // A shift count computed from data, held in range by a mask.
        ("let k = 70\n  c.println(str(1 << (k & 63)))", "64"),
        // The logical shift `docs/for-agents.md` teaches for `n` from 1 to 63, `(x >> n) & ~(-1 << (64 - n))`: -8's 64 bits
        // read unsigned, moved right (Python: (2**64 - 8) >> 1, >> 63 and >> 4).
        (
            "let x = 0 - 8\n  c.println(f\"{(x >> 1) & ~(-1 << 63)} {(x >> 63) & ~(-1 << 1)} {(x >> 4) & ~(-1 << 60)}\")",
            "9223372036854775804 1 1152921504606846975",
        ),
    ];
    for (body, want) in cases {
        let src = program(&format!("  {body}"));
        let (w, i) = run_both(&src);
        assert_eq!(w, i, "two-engine parity broken for:\n{src}");
        assert_eq!(w, Outcome::Printed(format!("{want}\n")), "for:\n{src}");
    }
}

/// A shift count outside 0 to 63 is DL0908 on both engines, in the same words, at the shift: the line printed before it
/// is kept and the line after it never runs. Never a count taken modulo 64, which is WebAssembly's own rule.
#[test]
fn every_shift_count_outside_0_to_63_faults_the_same_on_both_engines() {
    let fault =
        |op: &str, n: &str| format!("`{op}` moves an `Int`'s 64 bits by a count from 0 to 63; it was given a count of {n}");
    let cases: Vec<(&str, String)> = vec![
        ("c.println(str(1 << 64))", fault("<<", "64")),
        ("c.println(str(8 >> -1))", fault(">>", "-1")),
        ("var x = 1\n  x <<= 70\n  c.println(str(x))", fault("<<", "70")),
        ("let n = 0 - 5\n  c.println(str(1 >> n))", fault(">>", "-5")),
        ("c.println(str(1 << 9223372036854775807))", fault("<<", "9223372036854775807")),
        ("c.println(str(1 >> (1 << 63)))", fault(">>", "-9223372036854775808")),
        // A count that faults inside a function value faults where the function runs.
        ("c.println(str([60, 70].map(fn(i: Int) -> Int { 1 << i })))", fault("<<", "70")),
    ];
    for (body, msg) in cases {
        let src = program(&format!("  c.println(\"before\")\n  {body}\n  c.println(\"after\")"));
        let (w, i) = run_both(&src);
        assert_eq!(w, i, "two-engine parity broken for:\n{src}");
        // The interpreter's capture is not part of its fault; the WASM engine's printed lines are not either, so the
        // "before" line is checked through the binary in `bitwise_cli`.
        assert_eq!(w, Outcome::Fault("DL0908".to_string(), msg), "for:\n{src}");
    }
}

// ---- Generated expressions against a reference evaluator ------------------------------------------------------------

/// A bitwise expression and the evaluator that does not go through either engine: Rust's own `i64` operators, a shift
/// count checked as `prim::shift` checks it, operands evaluated left before right.
enum E {
    Leaf(&'static str, i64),
    Not(Box<E>),
    Bin(&'static str, Box<E>, Box<E>),
}

fn eval(e: &E) -> Result<i64, (&'static str, i64)> {
    Ok(match e {
        E::Leaf(_, v) => *v,
        E::Not(x) => !eval(x)?,
        E::Bin(op, l, r) => {
            let (a, b) = (eval(l)?, eval(r)?);
            match *op {
                "&" => a & b,
                "|" => a | b,
                "^" => a ^ b,
                "<<" | ">>" if !(0..=63).contains(&b) => return Err((op, b)),
                "<<" => a << b,
                _ => a >> b,
            }
        }
    })
}

fn text(e: &E) -> String {
    match e {
        E::Leaf(s, _) => (*s).to_string(),
        E::Not(x) => format!("~{}", text(x)),
        E::Bin(op, l, r) => format!("({} {op} {})", text(l), text(r)),
    }
}

const LEAVES: [(&str, i64); 12] = [
    ("0", 0),
    ("1", 1),
    ("(0 - 1)", -1),
    ("7", 7),
    ("(0 - 8)", -8),
    ("0xFF", 0xFF),
    ("0b1010_0101", 0b1010_0101),
    ("0o7654", 0o7654),
    ("9223372036854775807", i64::MAX),
    ("~9223372036854775807", i64::MIN),
    ("0x5555_5555_5555_5555", 0x5555_5555_5555_5555),
    ("0x0123_4567_89AB_CDEF", 0x0123_4567_89AB_CDEF),
];

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

fn gen(r: &mut Rng, depth: u32) -> E {
    if depth == 0 || r.below(4) == 0 {
        let (s, v) = LEAVES[r.below(LEAVES.len() as u64) as usize];
        return E::Leaf(s, v);
    }
    match r.below(6) {
        0 => E::Not(Box::new(gen(r, depth - 1))),
        1 => E::Bin("&", Box::new(gen(r, depth - 1)), Box::new(gen(r, depth - 1))),
        2 => E::Bin("|", Box::new(gen(r, depth - 1)), Box::new(gen(r, depth - 1))),
        3 => E::Bin("^", Box::new(gen(r, depth - 1)), Box::new(gen(r, depth - 1))),
        k => {
            let op = if k == 4 { "<<" } else { ">>" };
            // A count: a literal in range most of the time, one the 64 bits do not have now and then, or an
            // expression masked into range.
            const OUTSIDE: [(&str, i64); 3] = [("64", 64), ("(0 - 1)", -1), ("100", 100)];
            const COUNTS: [(&str, i64); 8] =
                [("0", 0), ("1", 1), ("4", 4), ("8", 8), ("31", 31), ("32", 32), ("62", 62), ("63", 63)];
            let count = match r.below(8) {
                0 => {
                    let (s, v) = OUTSIDE[r.below(OUTSIDE.len() as u64) as usize];
                    E::Leaf(s, v)
                }
                1 => E::Bin("&", Box::new(gen(r, depth - 1)), Box::new(E::Leaf("63", 63))),
                _ => {
                    let (s, v) = COUNTS[r.below(COUNTS.len() as u64) as usize];
                    E::Leaf(s, v)
                }
            };
            E::Bin(op, Box::new(gen(r, depth - 1)), Box::new(count))
        }
    }
}

/// 400 generated expressions: those the reference evaluator answers are printed by one program and must print its
/// answers on both engines; the first 25 it refuses each fault on both engines with the DL0908 it names. A hand-written
/// list agrees with its author; generated ones do not have to.
#[test]
fn generated_bitwise_expressions_answer_as_the_reference_evaluator_on_both_engines() {
    let mut r = Rng(0x2_35);
    let mut body = String::new();
    let mut want = String::new();
    let mut faults = Vec::new();
    for _ in 0..400 {
        let e = gen(&mut r, 4);
        match eval(&e) {
            Ok(v) => {
                body.push_str(&format!("  c.println(str({}))\n", text(&e)));
                want.push_str(&format!("{v}\n"));
            }
            Err((op, n)) => faults.push((text(&e), op, n)),
        }
    }
    let src = program(&body);
    let (w, i) = run_both(&src);
    assert_eq!(w, i, "two-engine parity broken");
    assert_eq!(w, Outcome::Printed(want));
    assert!(faults.len() >= 10, "the generator should reach the fault now and then: {}", faults.len());
    for (e, op, n) in faults.iter().take(25) {
        let src = program(&format!("  c.println(str({e}))"));
        let (w, i) = run_both(&src);
        assert_eq!(w, i, "two-engine parity broken for:\n{src}");
        let msg = format!("`{op}` moves an `Int`'s 64 bits by a count from 0 to 63; it was given a count of {n}");
        assert_eq!(w, Outcome::Fault("DL0908".into(), msg), "for:\n{src}");
    }
}

/// The bitwise operators in a module compiled WITHOUT the checker's types — how a Verified plugin's DIR is recompiled
/// (`verified_executable_wasm`) — compile and answer as the typed compile does (RW 2.38's lesson: a feature decided by
/// the types needs its untyped entry tested). The shift's import is decided from the syntax, so it is there too.
#[test]
fn bitwise_compiles_without_the_checkers_types() {
    let src = program(
        "  var x = 0b1100\n  x |= 0o3\n  x <<= 2\n  c.println(f\"{x} {x & 0xF} {x ^ 1} {~x} {x >> 1} {-x >> 63}\")\n  \
         c.println(str(1 << 64))",
    );
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let untyped = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile untyped: {e:?}"));
    let typed = compile_checked(&checked.module, &checked.result).expect("typed compile");
    let (u, t) = (wasm_outcome(&untyped), wasm_outcome(&typed));
    assert_eq!(u, t);
    assert_eq!(
        u,
        Outcome::Fault(
            "DL0908".into(),
            "`<<` moves an `Int`'s 64 bits by a count from 0 to 63; it was given a count of 64".into()
        )
    );
    // Without the faulting line, the answers themselves (Python: x = (12 | 3) << 2 == 60).
    let src = src.replace("\n  c.println(str(1 << 64))", "");
    let checked = check_source(0, &src);
    let untyped = compile_module(&checked.module).expect("untyped compile");
    assert_eq!(wasm_outcome(&untyped), Outcome::Printed("60 12 61 -61 30 -1\n".into()));
}

/// The shift's import, `delulu:math.shift_fault`, is decided from the syntax and follows every older import: a module
/// with no `<<` or `>>` takes none (so its bytes are what they were before RW 2.35), `&`, `|`, `^` and `~` take nothing
/// new, and a shift anywhere — a compound `<<=`, a function value, a function `main` never calls — takes it.
#[test]
fn only_a_module_that_shifts_imports_the_shift_fault() {
    let wasm_of = |src: &str| {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        compile_checked(&checked.module, &checked.result).expect("compile")
    };
    let has = |wasm: &[u8]| String::from_utf8_lossy(wasm).contains("shift_fault");
    let plain = program("  c.println(str(1 + 2))");
    let masks = program("  c.println(f\"{6 & 3} {6 | 3} {6 ^ 3} {~5} {0b11} {0o7}\")");
    assert!(!has(&wasm_of(&plain)), "a module with no operator of RW 2.35 imports `shift_fault`");
    assert!(!has(&wasm_of(&masks)), "a module with `&`, `|`, `^` and `~` alone imports `shift_fault`");
    for shifting in [
        program("  c.println(str(1 << 2))"),
        program("  var x = 8\n  x >>= 1\n  c.println(str(x))"),
        program("  c.println(str([1].map(fn(i: Int) -> Int { i >> 1 })))"),
        format!("{}\nfn unused(n: Int) -> Int {{\n  n << 1\n}}\n", program("  c.println(\"x\")")),
    ] {
        assert!(has(&wasm_of(&shifting)), "a module that shifts imports `shift_fault`:\n{shifting}");
    }
    // After every other import: here, after `float_to_fixed`, the newest before it.
    let both = wasm_of(&program("  c.println(1.5.to_fixed(1) + str(1 << 2))"));
    let text = String::from_utf8_lossy(&both);
    let (fixed, shift) =
        (text.find("float_to_fixed").expect("imports float_to_fixed"), text.find("shift_fault").expect("imports shift_fault"));
    assert!(fixed < shift, "`shift_fault` follows `float_to_fixed`");
}

/// The differential's generators draw the bitwise operators (`gen.rs`'s `gen_bits`) — the pure programs with a shift count
/// that may be an argument, so the shift's fault is drawn, and the console programs fault-free over literals in every
/// base. The gate skips a program that does not check and asks only that most do, so here every generated program with
/// a bitwise part must check clean and agree on both engines: a generator that wrote nothing checkable would pass the
/// gate unseen.
#[test]
fn the_differentials_bitwise_programs_check_clean_and_agree_on_both_engines() {
    let (mut pure, mut faulted, mut shift_faults, mut console) = (0, 0, 0, 0);
    for k in 0..600u64 {
        let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let (src, args) = gen::random_pure_program(seed);
        if src.contains(" ^ ") {
            pure += 1;
            let checked = check_source(0, &src);
            assert!(!checked.has_errors(), "a generated program should check clean:\n{src}\n{:?}", checked.diagnostics);
            let iv = Interp::new(&checked.module).call_int_fn("f", &args).map_err(|f| f.code.to_string());
            let wasm = compile_checked(&checked.module, &checked.result).expect("compile");
            let wv = run_int_fn(&wasm, "f", &args).map_err(|e| e.message());
            match (&iv, &wv) {
                (Ok(a), Ok(b)) => assert_eq!(a, b, "seed {seed}:\n{src}"),
                (Err(i), Err(w)) => {
                    faulted += 1;
                    // A shift's fault is the only DL0908 a pure program can raise: counted, and the same code on WASM.
                    if i == "DL0908" {
                        assert!(w.contains("DL0908"), "seed {seed}: interp={i} wasm={w}\n{src}");
                        shift_faults += 1;
                    }
                }
                _ => panic!("seed {seed}: interp={iv:?} wasm={wv:?} args={args:?}\n{src}"),
            }
        }
        let src = gen::random_console_program(seed);
        if ["<<", ">>", "~", " ^ "].iter().any(|op| src.contains(op)) {
            console += 1;
            let checked = check_source(0, &src);
            assert!(!checked.has_errors(), "a generated program should check clean:\n{src}\n{:?}", checked.diagnostics);
            let (w, i) = run_both(&src);
            assert_eq!(w, i, "seed {seed}:\n{src}");
            assert!(matches!(w, Outcome::Printed(_)), "a console program is fault-free: seed {seed}: {w:?}\n{src}");
        }
    }
    assert!(
        pure >= 150 && faulted >= 10 && shift_faults >= 5 && console >= 100,
        "drawn too rarely: pure {pure}, faulted {faulted}, shift faults {shift_faults}, console {console}"
    );
}
