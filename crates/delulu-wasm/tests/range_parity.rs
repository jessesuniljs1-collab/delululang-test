//! RW 2.36 (D-V2-224) — ranges on the WASM engine, held to the interpreter: `lo..hi`, `lo..=hi` and `range(lo, hi)` as
//! lists, a `for` over one counted without a list, `List.step_by(k)`, and the free `len(x)`. A range used as a list is
//! counted host-side by the interpreter's own `prim::range_len` (`delulu:list.range_len`), so its DL0908 is the
//! interpreter's, word for word; `step_by`'s slots are copied host-side (`delulu:list.step_by`), its DL0908 the same.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

/// What one engine did with a program: its output, or the fault it stopped with — the fault's code and message.
#[derive(Debug, PartialEq)]
enum Outcome {
    Printed(String),
    Fault(String, String),
}

fn wasm_outcome(r: Result<String, delulu_wasm::WasmError>) -> Outcome {
    match r {
        Ok(out) => Outcome::Printed(out),
        Err(e) => {
            let text = e.message();
            let fault = text.strip_prefix("WASM trap: ").unwrap_or_else(|| panic!("a WASM fault is a trap: {text}"));
            let (code, msg) = fault.split_once(": ").unwrap_or_else(|| panic!("a WASM fault names its code: {text}"));
            Outcome::Fault(code.to_string(), msg.to_string())
        }
    }
}

/// Run `main` on both engines with the console: `(wasm, interpreter)`.
fn run_both(src: &str) -> (Outcome, Outcome) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = wasm_outcome(run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }));
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

fn assert_parity(src: &str) -> Outcome {
    let (w, i) = run_both(src);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    w
}

/// A `main` that prints one expression, with `Int`'s two ends bound for it.
fn printing(expr: &str) -> String {
    format!(
        "module m\nfn main(root: Root) ! {{Write}} {{\n  let o = root.console()\n  let mn = 0 - 9223372036854775807 - 1\n  \
         let mx = 9223372036854775807\n  let n = 4\n  o.println(str({expr}))\n}}\n"
    )
}

/// Every form, empty ranges and `Int`'s ends included, against answers worked by hand (Python's `list(range(…))`).
#[test]
fn every_range_answers_as_the_interpreter_does() {
    let cases: &[(&str, &str)] = &[
        ("0..3", "[0, 1, 2]"),
        ("0..=3", "[0, 1, 2, 3]"),
        ("range(2, 5)", "[2, 3, 4]"),
        ("3..3", "[]"),
        ("3..=3", "[3]"),
        ("5..2", "[]"),
        ("5..=4", "[]"),
        ("0 - 2..2", "[-2, -1, 0, 1]"),
        ("0..n + 1", "[0, 1, 2, 3, 4]"),
        ("n * 2..=n * 2 + 2", "[8, 9, 10]"),
        ("mx - 2..=mx", "[9223372036854775805, 9223372036854775806, 9223372036854775807]"),
        ("mx - 1..mx", "[9223372036854775806]"),
        ("mn..mn + 2", "[-9223372036854775808, -9223372036854775807]"),
        ("mn..=mn", "[-9223372036854775808]"),
        ("(0..5).reverse()", "[4, 3, 2, 1, 0]"),
        ("(0..10).step_by(3)", "[0, 3, 6, 9]"),
        ("(1..=10).step_by(5)", "[1, 6]"),
        ("(0..4).step_by(9)", "[0]"),
        ("(0..0).step_by(2)", "[]"),
        ("(0..=10).reverse().step_by(5)", "[10, 5, 0]"),
        ("[\"a\", \"b\", \"c\", \"d\"].step_by(2)", "[a, c]"),
        ("(0..4).map(fn(i: Int) -> Int { i * i })", "[0, 1, 4, 9]"),
        ("(1..=4).fold(1, fn(a: Int, b: Int) -> Int { a * b })", "24"),
        ("(0..3) == [0, 1, 2]", "true"),
        ("0..3 == 0..=2", "true"),
        ("(0..3).concat(7..9)", "[0, 1, 2, 7, 8]"),
        ("len(0..7) + len(\"héllo\") + len([1, 2])", "14"),
        ("(0..100).len()", "100"),
    ];
    for (expr, want) in cases {
        let got = assert_parity(&printing(expr));
        assert_eq!(got, Outcome::Printed(format!("{want}\n")), "{expr}");
    }
}

/// A range longer than a list holds is DL0908 on both engines, word for word, before anything is allocated — and so is
/// one whose count overflows `i64` (the interpreter panicked on it, `capacity overflow`) — while the longest list that is
/// allowed is not refused for its length. `step_by` below 1 is DL0908 too.
#[test]
fn a_range_too_long_to_hold_and_a_step_below_one_are_dl0908_on_both_engines() {
    for (expr, needle) in [
        ("0..1000000000000", "`0..1000000000000` names 1000000000000 values"),
        ("mn..mx", "names 18446744073709551615 values"),
        ("mn..=mx", "names 18446744073709551616 values"),
        ("range(0, 268435457)", "`0..268435457` names 268435457 values"),
        ("0..=268435456", "`0..=268435456` names 268435457 values"),
        ("(0..3).step_by(0)", "it was given 0"),
        ("(0..3).step_by(0 - 2)", "it was given -2"),
    ] {
        match assert_parity(&printing(expr)) {
            Outcome::Fault(code, msg) => {
                assert_eq!(code, "DL0908", "{expr}: {msg}");
                assert!(msg.contains(needle), "{expr}: {msg}");
            }
            other => panic!("{expr} should fault DL0908: {other:?}"),
        }
    }
}

/// A `for` over a range builds no list — so a range far longer than a list holds iterates, on both engines, and stops at
/// a `break` — and `continue` takes the step. A closed range reaches `Int`'s largest value without overflowing; a range
/// at the smallest starts there. Its bounds are read once, before the first pass, though the body moves what they read.
#[test]
fn a_for_over_a_range_counts_without_a_list() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let o = root.console()
  var total = 0
  for i in 0..1000000000000 {
    if i == 7 {
      break
    }
    if i % 2 == 0 {
      continue
    }
    total += i
  }
  o.println("odd below 7: " + str(total))
  let mx = 9223372036854775807
  var last = 0
  var count = 0
  for i in mx - 2..=mx {
    last = i
    count += 1
  }
  o.println(str(count) + " " + str(last))
  let mn = 0 - 9223372036854775807 - 1
  for i in mn..mn + 2 {
    o.println(str(i))
  }
  var hi = 3
  for i in 0..hi {
    hi = 100
    o.println("pass " + str(i))
  }
  for i in range(10, 12) {
    o.println("range " + str(i))
  }
  for i in 5..=4 {
    o.println("never " + str(i))
  }
  var nested = 0
  for a in 1..=3 {
    for b in a..=3 {
      nested += a * b
    }
  }
  o.println("nested " + str(nested))
}
"#;
    let got = assert_parity(src);
    assert_eq!(
        got,
        Outcome::Printed(
            "odd below 7: 9\n3 9223372036854775807\n-9223372036854775808\n-9223372036854775807\npass 0\npass 1\npass 2\n\
             range 10\nrange 11\nnested 25\n"
                .into()
        )
    );
}

/// A local named `range` is the function the checker calls, never the builtin — in a `for` head too.
#[test]
fn a_local_named_range_is_called_in_a_for_head() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let o = root.console()
  let range = fn(a: Int, b: Int) -> List[Int] { [b, a] }
  for i in range(1, 2) {
    o.println(str(i))
  }
}
"#;
    let checked = check_source(0, src);
    if checked.has_errors() {
        // The checker may refuse the shadowing outright; then there is nothing to run, and nothing to disagree about.
        return;
    }
    assert_eq!(assert_parity(src), Outcome::Printed("2\n1\n".into()));
}

/// Ranges, `step_by` and `len` in a module compiled WITHOUT the checker's types — how a Verified plugin's DIR is
/// recompiled — compile and answer as the typed compile does (RW 2.38's lesson: the imports are decided from the syntax).
#[test]
fn ranges_compile_without_the_checkers_types() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let o = root.console()
  var s = 0
  for i in 1..=4 {
    s += i
  }
  let xs = (0..10).step_by(4)
  o.println(str(s) + " " + str(len(range(0, 3))) + " " + str(xs.len()) + " " + str(len(0..=9)))
}
"#;
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let untyped = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile untyped: {e:?}"));
    let out = run_main(&untyped, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    let typed = compile_checked(&checked.module, &checked.result).expect("typed compile");
    let want = run_main(&typed, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    assert_eq!(out, want);
    assert_eq!(out.as_deref(), Ok("10 3 3 10\n"));
}
