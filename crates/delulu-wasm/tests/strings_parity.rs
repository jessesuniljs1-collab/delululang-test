//! RW 2.37 (D-V2-225) — strings on the WASM engine, held to the interpreter: f-strings (each hole `str(hole)`), `s[i]`,
//! the new `Str` methods and `Float.to_fixed`. Every method runs host-side through the interpreter's own
//! `call_str_method` and `float_to_fixed`, and a hole through the same lowering as `str` — so every answer, and every
//! fault, must be the interpreter's, word for word. Each expected answer below was worked by hand or by Python.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

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

fn program(body: &str) -> String {
    format!(
        "module m\n\nfn some(o: Option[Int]) -> Str {{\n  match o {{\n    Some(v) => f\"Some({{v}})\",\n    None => \"None\",\n  }}\n}}\n\n\
         fn main(root: Root) ! {{Write}} {{\n  let c = root.console()\n{body}\n}}\n"
    )
}

/// Each line's answer on both engines, against the answer worked by hand (or by Python, where it says).
#[test]
fn every_string_answer_is_the_same_on_both_engines_and_the_one_worked_by_hand() {
    let cases: &[(&str, &str)] = &[
        // f-strings: each hole is `str(hole)`, in order; `{{` and `}}` are braces.
        (
            r#"let n = 41
  c.println(f"n+1={n + 1} {{b}} {[1, 2]} {Some(3)} {true} {2.5} {"s"}")"#,
            "n+1=42 {b} [1, 2] Some(3) true 2.5 s",
        ),
        (
            r#"let name = "Ada"
  c.println(f"{name}{name}" + f"|{f"<{name}>"}|")"#,
            "AdaAda|<Ada>|",
        ),
        // `s[i]`, a character (Python: 'héllo'[1] == 'é').
        (
            r#"let s = "héllo"
  c.println(s[0] + s[1] + s[4] + str(len(s)))"#,
            "héo5",
        ),
        // find / rfind / count, character places (Python: find 2, rfind 10, find('') 0, rfind('') 12, count 3, 13, 2).
        (
            r#"let s = "héllo, wörld"
  c.println(f"{some(s.find("l"))} {some(s.rfind("l"))} {some(s.find("z"))} {some(s.find(""))} {some(s.rfind(""))}")"#,
            "Some(2) Some(10) None Some(0) Some(12)",
        ),
        (r#"c.println(f"{"héllo, wörld".count("l")} {"abc".count("")} {"aaaa".count("aa")}")"#, "3 4 2"),
        (
            r#"c.println(f"{"wörld".ends_with("rld")} {"wörld".ends_with("x")} {"".is_empty()} {" ".is_empty()}")"#,
            "true false true false",
        ),
        (r#"c.println(f"[{"ab".repeat(3)}][{"x".repeat(0)}][{"".repeat(9)}]")"#, "[ababab][][]"),
        // Python: '7'.rjust(3,'0'), '7'.ljust(3,'.'), 'long'.rjust(2), 'é'.rjust(3,'ü'), 'x'.rjust(-4).
        (
            r#"c.println(f"[{"7".pad_left(3, "0")}][{"7".pad_right(3, ".")}][{"long".pad_left(2, " ")}][{"é".pad_left(3, "ü")}][{"x".pad_left(0 - 4, " ")}]")"#,
            "[007][7..][long][üüé][x]",
        ),
        (r#"c.println(f"[{"  x  ".trim_start()}][{"  x  ".trim_end()}]")"#, "[x  ][  x]"),
        // Python's f"{x:.2f}": ties to even on the exact binary value (2.675 is 2.67499…; 0.125 is exact, so 0.12).
        (
            r#"c.println(f"{3.14159.to_fixed(2)} {2.675.to_fixed(2)} {0.125.to_fixed(2)} {(0.0 - 0.001).to_fixed(2)} {2.5.to_fixed(0)} {1234.5678.to_fixed(1)}")"#,
            "3.14 2.67 0.12 -0.00 2 1234.6",
        ),
        (r#"c.println(f"{(1.0 / 0.0).to_fixed(2)} {(0.0 / 0.0).to_fixed(3)} {(0.0 - 1.0 / 0.0).to_fixed(1)}")"#, "inf NaN -inf"),
        // `str` of a Float: the same digits as before from 1e-4 to 1e16, an exponent outside, as a literal spells it.
        (
            r#"c.println(f"{100.0.exp()} {1.0e300} {1.0e-7} {0.0001} {0.00001} {1.0e16} {9999999999999998.0} {123.0} {[1.0e20]}")"#,
            "2.6881171418161356e43 1.0e300 1.0e-7 0.0001 1.0e-5 1.0e16 9999999999999998.0 123.0 [1.0e20]",
        ),
    ];
    for (body, want) in cases {
        let src = program(body);
        let (w, i) = run_both(&src);
        assert_eq!(w, i, "two-engine parity broken for:\n{src}");
        assert_eq!(w, Outcome::Printed(format!("{want}\n")), "the answer worked by hand, for:\n{src}");
    }
}

/// Every fault a string operation can raise, the same code and the same words on both engines.
#[test]
fn every_string_fault_is_the_same_on_both_engines() {
    let cases: &[(&str, &str, &str)] = &[
        ("let s = \"héllo\"\n  c.println(s[5])", "DL0903", "index 5 out of bounds (len 5)"),
        ("let i = 0 - 1\n  c.println(\"abc\"[i])", "DL0903", "index -1 out of bounds (len 3)"),
        ("c.println(\"ab\".repeat(0 - 2))", "DL0908", "`repeat` takes a count of 0 or more, and this is -2"),
        (
            "c.println(\"ab\".repeat(1000000000000))",
            "DL0908",
            "`repeat` would build a string of 2000000000000 bytes, more than one holds (268435456)",
        ),
        ("c.println(\"ab\".pad_left(5, \"xy\"))", "DL0908", "`pad_left` pads with one character, and its fill `xy` has 2"),
        ("c.println(\"ab\".pad_right(5, \"\"))", "DL0908", "`pad_right` pads with one character, and its fill `` has 0"),
        (
            "c.println(\"ab\".pad_right(9000000000000000000, \" \"))",
            "DL0908",
            "`pad_right` would build a string of 9000000000000000000 bytes, more than one holds (268435456)",
        ),
        ("c.println(1.5.to_fixed(101))", "DL0908", "`to_fixed` takes 0 to 100 digits, and this is 101"),
        ("c.println(1.5.to_fixed(0 - 1))", "DL0908", "`to_fixed` takes 0 to 100 digits, and this is -1"),
        // A fault inside a hole stops the f-string where it stands: the text before it is never printed.
        ("c.println(f\"a{\"xy\"[2]}b\")", "DL0903", "index 2 out of bounds (len 2)"),
    ];
    for (body, code, msg) in cases {
        let src = program(&format!("  c.println(\"before\")\n  {body}\n  c.println(\"after\")"));
        let (w, i) = run_both(&src);
        assert_eq!(w, i, "two-engine parity broken for:\n{src}");
        assert_eq!(w, Outcome::Fault(code.to_string(), msg.to_string()), "for:\n{src}");
    }
}

/// Generated f-strings: text runs and holes of `Int`, `Str`, `Bool` and `Float` values, the expected text built here
/// from the same pieces — on both engines. A hand-written list agrees with its author; 300 generated ones do not have to.
#[test]
fn generated_fstrings_print_their_pieces_in_order_on_both_engines() {
    let mut seed: u64 = 0x2_37;
    let mut next = move |n: u64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) % n
    };
    let texts =
        [("a", "a"), (" ", " "), ("é🐦", "é🐦"), ("{{", "{"), ("}}", "}"), ("\\\"", "\""), ("\\\\", "\\"), ("\\u{41}", "A")];
    let holes = [
        ("n", "7"),
        ("n * 6", "42"),
        ("s", "hé"),
        ("s.repeat(2)", "héhé"),
        ("b", "true"),
        ("!b", "false"),
        ("x", "2.5"),
        ("x * 1.0e20", "2.5e20"),
        ("s[1]", "é"),
        ("\"lit\"", "lit"),
        ("f\"<{n}>\"", "<7>"),
        ("[n, n]", "[7, 7]"),
        ("str(n).pad_left(4, \"0\")", "0007"),
    ];
    let mut body = String::from("  let n = 7\n  let s = \"hé\"\n  let b = true\n  let x = 2.5\n");
    let mut want = String::new();
    for _ in 0..300 {
        let mut f = String::from("f\"");
        for _ in 0..=next(5) {
            if next(3) == 0 {
                let (src, text) = texts[next(texts.len() as u64) as usize];
                f.push_str(src);
                want.push_str(text);
            } else {
                let (src, text) = holes[next(holes.len() as u64) as usize];
                f.push_str(&format!("{{{src}}}"));
                want.push_str(text);
            }
        }
        f.push('"');
        body.push_str(&format!("  c.println({f})\n"));
        want.push('\n');
    }
    let src = program(&body);
    let (w, i) = run_both(&src);
    assert_eq!(w, i, "two-engine parity broken");
    assert_eq!(w, Outcome::Printed(want));
}

/// Strings in a module compiled WITHOUT the checker's types — how a Verified plugin's DIR is recompiled
/// (`verified_executable_wasm`) — compile and answer as the typed compile does: an f-string, `s[i]`, `find`, `count`,
/// a padded number and `to_fixed` (RW 2.38's lesson: a feature decided by the types needs its untyped entry tested).
#[test]
fn strings_compile_without_the_checkers_types() {
    let src = program(
        r#"let s = "héllo"
  let xs = [10, 20]
  c.println(f"{s[1]} {some(s.find("l"))} {s.count("l")} {str(xs[1]).pad_left(4, "0")} {2.675.to_fixed(2)} {s.ends_with("lo")}")"#,
    );
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let untyped = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile untyped: {e:?}"));
    let out = run_main(&untyped, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    let typed = compile_checked(&checked.module, &checked.result).expect("typed compile");
    let want = run_main(&typed, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    assert_eq!(out, want);
    assert_eq!(out.as_deref(), Ok("é Some(2) 2 0020 2.67 true\n"));
}

/// The new imports follow every older one and are decided from the syntax: a module that uses none of this run's
/// surface compiles to the same bytes whether or not the module beside it does (a list's index takes no string import
/// in the typed compile), and the list method `find` does not pull in `str_find`.
#[test]
fn a_module_without_the_new_surface_takes_none_of_its_imports() {
    let plain = program("let xs = [1, 2, 3]\n  c.println(str(xs[1]) + str(xs.find(fn(i: Int) -> Bool { i > 1 })))");
    let checked = check_source(0, &plain);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let wasm = compile_checked(&checked.module, &checked.result).expect("compile");
    let text = String::from_utf8_lossy(&wasm);
    for import in ["str_method", "str_find", "float_to_fixed"] {
        assert!(!text.contains(import), "a module with no string method imports `{import}`");
    }
    let with = program("let s = \"ab\"\n  c.println(s[0] + str(s.count(\"a\")) + 1.5.to_fixed(1))");
    let checked = check_source(0, &with);
    let wasm = compile_checked(&checked.module, &checked.result).expect("compile");
    let text = String::from_utf8_lossy(&wasm);
    for import in ["str_method", "str_find", "float_to_fixed"] {
        assert!(text.contains(import), "a module that uses them imports `{import}`");
    }
}

/// A `match` on a `Str`, an `Int` or a `Bool` — literal arms, a name, `_` — runs on both engines, each arm tried in
/// order. Found by `wrap.delulu` (`match how { "left" => … }`), which was DL1201 on this engine alone.
#[test]
fn a_match_on_a_str_an_int_or_a_bool_runs_on_both_engines() {
    let src = program(
        r#"for how in ["left", "right", "centre", ""] {
    let n = match how {
      "left" => 1
      "right" => 2
      other => len(other) * 10
    }
    let word = match n {
      1 => "one"
      2 => "two"
      0 => "zero"
      _ => f"many({n})"
    }
    let yes = match n > 1 {
      true => "big"
      false => "small"
    }
    c.println(f"{how}:{n}:{word}:{yes}")
  }"#,
    );
    let (w, i) = run_both(&src);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    assert_eq!(w, Outcome::Printed("left:1:one:small\nright:2:two:big\ncentre:60:many(60):big\n:0:zero:small\n".into()));
    // Without the checker's types too.
    let checked = check_source(0, &src);
    let untyped = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile untyped: {e:?}"));
    let out = run_main(&untyped, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    assert_eq!(out.as_deref(), Ok("left:1:one:small\nright:2:two:big\ncentre:60:many(60):big\n:0:zero:small\n"));
}

/// `s[i]` alone in a module compiled without the checker's types: nothing else in it calls a `Str` method, so only the
/// index can bring the string interface in — and the untyped compile cannot tell a `Str`'s index from a list's.
#[test]
fn a_str_index_alone_compiles_without_the_checkers_types() {
    let src = program("let s = \"héllo\"\n  let xs = [5, 6]\n  c.println(s[1] + str(xs[0]))");
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let untyped = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile untyped: {e:?}"));
    let out = run_main(&untyped, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    assert_eq!(out.as_deref(), Ok("é5\n"));
}
