//! MATCH-NEST-1 — a `match` whose patterns test inside a constructor (`Some(Red)`, `Ok("yes")`, `Square(0)`) on the WASM
//! engine, held to the interpreter. The tag chain sent each constructor to the first arm naming it and read nothing
//! inside, so `Some(Blue)` took `Some(Red) => …` and `Some(7)` took `Some(0) => …`, silently. Such a `match` now compiles
//! arm by arm, in order, as the interpreter's `eval_match` tries them; and a `match` that tests nothing inside a
//! constructor compiles as before, without the `delulu:match` import.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

/// Run `main` on both engines with the console; both outputs, or each fault's text.
fn run_both(src: &str) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    (w, i.map(|_| out))
}

fn assert_parity(src: &str) -> String {
    let (w, i) = run_both(src);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    w.unwrap_or_else(|e| panic!("both engines failed: {e}\n{src}"))
}

/// A constructor, an `Int`, a `Str` and a `Bool` inside a constructor, two levels down, and an arm that names a field
/// after a more general arm for the same constructor — each function called with every case it separates. `"maybe"` is
/// a literal only its pattern holds, so its test cannot borrow one an expression interned.
#[test]
fn a_pattern_inside_a_constructor_is_tested_as_the_interpreter_tests_it() {
    let src = r#"module m

type Color = Red | Green | Blue
type Shape = Circle(Color) | Square(Int) | Pair(Color, Int)

fn color(o: Option[Color]) -> Str {
  match o {
    Some(Red) => "red",
    Some(Blue) => "blue",
    Some(c) => "other " + str(c),
    None => "none",
  }
}

fn num(o: Option[Int]) -> Int {
  match o {
    Some(0) => 100,
    Some(7) => 700,
    Some(n) => n,
    None => 0 - 1,
  }
}

fn word(r: Result[Str, Int]) -> Str {
  match r {
    Ok("yes") => "Y",
    Ok("maybe") => "M",
    Ok("") => "empty",
    Ok(s) => "ok " + s,
    Err(0) => "zero",
    Err(e) => "err " + str(e),
  }
}

fn flag(o: Option[Bool]) -> Int {
  match o {
    Some(true) => 1,
    Some(false) => 0,
    None => 2,
  }
}

fn shape(s: Shape) -> Str {
  match s {
    Circle(Red) => "red circle",
    Square(0) => "dot",
    Pair(Blue, 1) => "blue one",
    Pair(c, 1) => "one " + str(c),
    Pair(Green, n) => "green " + str(n),
    Circle(_) => "circle",
    Square(n) => "square " + str(n),
    Pair(_, _) => "pair",
  }
}

fn first(o: Option[Int]) -> Int {
  match o {
    Some(n) => n,
    Some(0) => 999,
    None => 0,
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(color(Some(Red)) + "," + color(Some(Green)) + "," + color(Some(Blue)) + "," + color(None))
  out.println(str(num(Some(0))) + "," + str(num(Some(7))) + "," + str(num(Some(5))) + "," + str(num(None)))
  out.println(word(Ok("yes")) + "," + word(Ok("may" + "be")) + "," + word(Ok("")) + "," + word(Ok("no")) + "," + word(Err(0)) + "," + word(Err(3)))
  out.println(str(flag(Some(true))) + str(flag(Some(false))) + str(flag(None)))
  out.println(shape(Circle(Red)) + "," + shape(Circle(Green)) + "," + shape(Square(0)) + "," + shape(Square(4)))
  out.println(shape(Pair(Blue, 1)) + "," + shape(Pair(Red, 1)) + "," + shape(Pair(Green, 5)) + "," + shape(Pair(Blue, 2)))
  out.println(str(first(Some(0))) + "," + str(first(Some(3))) + "," + str(first(None)))
}
"#;
    assert_eq!(
        assert_parity(src),
        "red,other Green,blue,none\n\
         100,700,5,-1\n\
         Y,M,empty,ok no,zero,err 3\n\
         102\n\
         red circle,circle,dot,square 4\n\
         blue one,one Red,green 5,pair\n\
         0,3,0\n"
    );
}

/// Inside a loop: an arm that `break`s, one that `continue`s and one that holds another such `match`, as statements; and
/// such a `match` as a value with no type written for it, which its first arm gives.
#[test]
fn a_match_testing_inside_breaks_continues_and_nests_as_the_interpreter_does() {
    let src = r#"module m

fn step(i: Int) -> Option[Int] {
  if i % 3 == 0 { None } else { Some(i % 4) }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var i = 0
  var acc = ""
  while i < 20 {
    i = i + 1
    match step(i) {
      Some(0) => {
        acc = acc + "z "
      },
      Some(3) => {
        if i > 14 {
          break
        }
        acc = acc + "t "
      },
      Some(n) => {
        acc = acc + match Some(n * 2) {
          Some(2) => "two ",
          Some(m) => str(m) + " ",
          None => "? ",
        }
      },
      None => {
        continue
      },
    }
    acc = acc + ". "
  }
  let label = match step(5) {
    Some(1) => "one",
    Some(n) => str(n),
    None => "none",
  }
  out.println(acc + "| " + str(i) + " " + label)
}
"#;
    assert_eq!(assert_parity(src), "two . 4 . z . two . t . z . 4 . t . two . 4 . z . two . | 19 one\n");
}

/// When no arm matches — which the checker refuses (DL0407), so the module is compiled here unchecked — the WASM run ends
/// in the interpreter's own fault, word for word, never in another arm or another code.
#[test]
fn a_match_no_arm_of_which_applies_faults_as_the_interpreter_faults() {
    let src = "module m\n\
        type Color = Red | Blue\n\
        fn name(o: Option[Color]) -> Int {\n\
        \x20 match o {\n\
        \x20   Some(Red) => 1,\n\
        \x20   None => 0,\n\
        \x20 }\n\
        }\n\
        fn main(root: Root) ! {Write} {\n\
        \x20 let out = root.console()\n\
        \x20 out.println(str(name(Some(Red))))\n\
        \x20 out.println(str(name(Some(Blue))))\n\
        }\n";
    let (module, diags) = delulu_syntax::parse_file(0, src);
    assert!(diags.is_empty(), "{diags:?}");
    let wasm = compile_module(&module).expect("should compile to WASM");
    let w = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    let w = w.expect_err("no arm matches `Some(Blue)`");
    assert_eq!(w, "WASM trap: DL0907: no match arm applied (checker guarantees exhaustiveness)");
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let i = Interp::new(&module).run_main(Value::Root(Rc::new(g.build_root())));
    let _ = take_capture();
    let i = format!("{:?}", i.expect_err("the interpreter faults too"));
    assert!(i.contains("DL0907") && i.contains("no match arm applied (checker guarantees exhaustiveness)"), "{i}");
}

/// A module whose matches test nothing inside a constructor is unchanged: it imports no `delulu:match`.
#[test]
fn only_a_match_testing_inside_imports_the_match_interface() {
    let plain = "module m\n\
        type Color = Red | Blue\n\
        fn f(o: Option[Color]) -> Int {\n\
        \x20 match o {\n\
        \x20   Some(c) => 1,\n\
        \x20   None => 0,\n\
        \x20 }\n\
        }\n";
    let nested = plain.replace("Some(c) => 1,", "Some(Red) => 2,\n    Some(_) => 1,");
    let imports = |src: &str| {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        let wasm = compile_checked(&checked.module, &checked.result).expect("should compile to WASM");
        wasm.windows(b"delulu:match".len()).any(|w| w == b"delulu:match")
    };
    assert!(!imports(plain), "a match testing nothing inside imports nothing new");
    assert!(imports(&nested), "a match testing inside carries `delulu:match`");
}
