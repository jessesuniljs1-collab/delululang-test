//! RW 2.38 (D-V2-223) — the math methods on the WASM engine, held to the interpreter. `Int` and `Float` methods run
//! host-side through the interpreter's own `call_int_method` and `call_float_method` (`delulu:math`); `is_nan`,
//! `is_finite` and `to_int` compile in the guest. So every answer, and every fault, must be the interpreter's.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

/// The math methods in a module compiled WITHOUT the checker's types — how a Verified plugin's DIR is recompiled
/// (`verified_executable_wasm`) — compile and answer as the typed compile does.
#[test]
fn math_methods_compile_without_the_checkers_types() {
    let src = r#"module m

fn some(o: Option[Int]) -> Str {
  match o {
    Some(v) => "Some(" + str(v) + ")",
    None => "None",
  }
}

fn main(root: Root) ! {Write} {
  let o = root.console()
  let n = 0 - 7
  let x = 2.0
  o.println(str(n.abs()) + " " + str(x.sqrt()) + " " + some(n.checked_div(0)) + " " + some(x.to_int()))
}
"#;
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile untyped: {e:?}"));
    let out = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    let typed = compile_checked(&checked.module, &checked.result).expect("typed compile");
    let want = run_main(&typed, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    assert_eq!(out, want);
    assert_eq!(out.as_deref(), Ok("7 1.4142135623730951 None Some(2)\n"));
}

/// What one engine did with a program: its output, or the fault it stopped with — the fault's code and message.
#[derive(Debug, PartialEq)]
enum Outcome {
    Printed(String),
    Fault(String, String),
}

/// Run `main` on both engines with the console: `(wasm, interpreter)`. The WASM engine's fault text is the
/// interpreter's own `CODE: message` (the host records the interpreter's fault, `host.rs`'s `math_fault`), so a fault
/// is compared code for code and word for word, once the engine's `WASM trap: ` is read off.
fn run_both(src: &str) -> (Outcome, Outcome) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = match run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }) {
        Ok(out) => Outcome::Printed(out),
        Err(e) => {
            let text = e.message();
            let fault = text.strip_prefix("WASM trap: ").unwrap_or_else(|| panic!("a WASM fault is a trap: {text}"));
            let (code, msg) = fault.split_once(": ").unwrap_or_else(|| panic!("a WASM fault names its code: {text}"));
            Outcome::Fault(code.to_string(), msg.to_string())
        }
    };
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

/// A `main` that prints one expression, with a few names bound for it.
fn printing(expr: &str) -> String {
    format!(
        "module m\nfn main(root: Root) ! {{Write}} {{\n  let o = root.console()\n  let mn = 0 - 9223372036854775807 - 1\n  \
         let mx = 9223372036854775807\n  let nan = 0.0 / 0.0\n  let inf = 1.0 / 0.0\n  let z = 0\n  \
         o.println(str({expr}))\n}}\n"
    )
}

/// Every method over edge values, on both engines, against answers worked by hand, and against Python's `math` for the
/// transcendental ones. They agree to the last digit on these inputs except `1.0.exp()`: `libm` answers
/// `2.7182818284590455`, one ulp above e, where glibc's `exp` (CPython's on Linux) is correctly rounded,
/// `2.718281828459045`. The digits are the same on every OS and both engines, which is what `libm` was chosen for.
#[test]
fn every_math_method_answers_as_the_interpreter_does() {
    let cases: &[(&str, &str)] = &[
        ("(0 - 7).abs()", "7"),
        ("3.min(0 - 4)", "-4"),
        ("3.max(0 - 4)", "3"),
        ("12.clamp(0, 10)", "10"),
        ("(0 - 12).clamp(0, 10)", "0"),
        ("2.pow(62)", "4611686018427387904"),
        ("(0 - 2).pow(63)", "-9223372036854775808"),
        ("(0 - 1).pow(mx)", "-1"),
        ("0.pow(0)", "1"),
        ("mx.checked_add(1)", "None"),
        ("mn.checked_sub(1)", "None"),
        ("mx.checked_mul(2)", "None"),
        ("mn.checked_div(0 - 1)", "None"),
        ("mn.checked_rem(0 - 1)", "None"),
        ("7.checked_div(z)", "None"),
        ("7.checked_rem(z)", "None"),
        ("(0 - 7).checked_div(2)", "Some(-3)"),
        ("(0 - 7).checked_rem(2)", "Some(-1)"),
        ("(0.0 - 2.5).abs()", "2.5"),
        ("(-0.0).abs()", "0.0"),
        ("0.0.min(-0.0)", "-0.0"),
        ("(-0.0).max(0.0)", "0.0"),
        ("nan.min(1.0)", "1.0"),
        ("1.0.max(nan)", "1.0"),
        ("nan.clamp(0.0, 1.0)", "NaN"),
        ("(-0.0).clamp(0.0, 1.0)", "0.0"),
        ("2.0.sqrt()", "1.4142135623730951"),
        ("(0.0 - 1.0).sqrt()", "NaN"),
        ("(0.0 - 2.5).floor()", "-3.0"),
        ("(0.0 - 2.5).ceil()", "-2.0"),
        ("2.5.round()", "3.0"),
        ("(0.0 - 2.5).round()", "-3.0"),
        ("0.49999999999999994.round()", "0.0"),
        ("(0.0 - 2.7).trunc()", "-2.0"),
        ("2.0.pow(0.5)", "1.4142135623730951"),
        ("(0.0 - 8.0).pow(1.0 / 3.0)", "NaN"),
        ("nan.pow(0.0)", "1.0"),
        ("1.0.sin()", "0.8414709848078965"),
        ("1.0.cos()", "0.5403023058681398"),
        ("1.0.tan()", "1.5574077246549023"),
        ("0.5.asin()", "0.5235987755982989"),
        ("0.5.acos()", "1.0471975511965979"),
        ("2.0.asin()", "NaN"),
        ("1.0.atan()", "0.7853981633974483"),
        ("1.0.atan2(0.0 - 1.0)", "2.356194490192345"),
        ("1.0.exp()", "2.7182818284590455"),
        ("0.0.ln()", "-inf"),
        ("2.0.ln()", "0.6931471805599453"),
        ("1000.0.log10()", "3.0"),
        ("inf.is_finite()", "false"),
        ("nan.is_nan()", "true"),
        ("1.0.is_nan()", "false"),
        ("2.9.to_int()", "Some(2)"),
        ("(0.0 - 9223372036854775808.0).to_int()", "Some(-9223372036854775808)"),
        ("9223372036854775808.0.to_int()", "None"),
        ("nan.to_int()", "None"),
        ("(0.0 - inf).to_int()", "None"),
    ];
    for (expr, want) in cases {
        let got = assert_parity(&printing(expr));
        assert_eq!(got, Outcome::Printed(format!("{want}\n")), "`{expr}`");
    }
}

/// Every fault a math method can raise: the same code and the same words on both engines, and the program stops there.
#[test]
fn each_math_fault_is_the_same_fault_on_both_engines() {
    let cases: &[(&str, &str)] = &[
        ("mn.abs()", "DL0901"),
        ("2.pow(63)", "DL0901"),
        ("3.pow(2147483648)", "DL0901"),
        ("3.pow(0 - 1)", "DL0908"),
        ("5.clamp(3, 1)", "DL0908"),
        ("1.5.clamp(2.0, 1.0)", "DL0908"),
        ("1.5.clamp(nan, 1.0)", "DL0908"),
        ("1.5.clamp(0.0, nan)", "DL0908"),
    ];
    for (expr, code) in cases {
        match assert_parity(&printing(expr)) {
            Outcome::Fault(c, msg) => {
                assert_eq!(&c, code, "`{expr}`: {msg}");
                assert!(!msg.is_empty());
            }
            other => panic!("`{expr}` should fault {code}, printed {other:?}"),
        }
    }
}

/// The law the checked forms exist for, on both engines: `n.checked_op(m)` is `None` exactly where `n op m` stops the
/// program (DL0901 or DL0902), and `Some` of the operator's own answer everywhere else. One program walks the grid,
/// asking the operator only where its checked form said `Some`; each `None` is then run as `n op m`, which must fault.
#[test]
fn a_checked_form_is_none_exactly_where_its_operator_faults_on_both_engines() {
    let edges = ["0", "1", "(0 - 1)", "2", "mx", "mn", "(mn + 1)"];
    let ops = [("checked_add", "+"), ("checked_sub", "-"), ("checked_mul", "*"), ("checked_div", "/"), ("checked_rem", "%")];
    let mut body = String::new();
    let mut nones = Vec::new();
    for n in edges {
        for m in edges {
            for (method, op) in ops {
                body.push_str(&format!(
                    "  match {n}.{method}({m}) {{\n    Some(v) => o.println(str(v == {n} {op} {m}))\n    None => o.println(\"N\")\n  }}\n"
                ));
                nones.push((format!("{n} {op} {m}"), format!("{n}.{method}({m})")));
            }
        }
    }
    let src = format!(
        "module m\nfn main(root: Root) ! {{Write}} {{\n  let o = root.console()\n  let mn = 0 - 9223372036854775807 - 1\n  \
         let mx = 9223372036854775807\n{body}}}\n"
    );
    let Outcome::Printed(out) = assert_parity(&src) else { panic!("the grid program never faults") };
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), nones.len());
    let mut faulted = 0;
    for (line, (direct, checked)) in lines.iter().zip(&nones) {
        match *line {
            "true" => {}
            "N" => {
                match assert_parity(&printing(direct)) {
                    Outcome::Fault(c, _) => assert!(c == "DL0901" || c == "DL0902", "`{direct}` faulted {c}"),
                    other => panic!("`{checked}` is None but `{direct}` printed {other:?}"),
                }
                faulted += 1;
            }
            other => panic!("`{checked}` is Some, but not `{direct}`'s answer: {other}"),
        }
    }
    // The grid must reach both sides of the law, or it proves nothing: zero divisors, `mn / -1`, and the overflows.
    assert!(faulted >= 20, "only {faulted} faulting cells");
    assert!(nones.len() - faulted >= 150, "only {} answering cells", nones.len() - faulted);
}

/// The math methods evaluate the receiver, then the arguments left to right, then call — on both engines, so an argument
/// with an effect is seen in the same order.
#[test]
fn the_receiver_and_arguments_run_in_order_on_both_engines() {
    let src = r#"module m

fn f(out: Cap[Console], n: Int) -> Int ! {Write} {
  out.println("f " + str(n))
  n
}

fn g(out: Cap[Console], x: Float) -> Float ! {Write} {
  out.println("g " + str(x))
  x
}

fn main(root: Root) ! {Write} {
  let o = root.console()
  o.println(str(f(o, 10).clamp(f(o, 1), f(o, 5))))
  o.println(str(g(o, 1.0).atan2(g(o, 0.0))))
  o.println(str(f(o, 2).checked_mul(f(o, 3))))
}
"#;
    assert_eq!(
        assert_parity(src),
        Outcome::Printed("f 10\nf 1\nf 5\n5\ng 1.0\ng 0.0\n1.5707963267948966\nf 2\nf 3\nSome(6)\n".into())
    );
}
