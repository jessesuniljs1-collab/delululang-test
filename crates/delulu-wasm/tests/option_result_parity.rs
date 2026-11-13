//! RW 2.50 (D-V2-227) — the methods of `Option` and `Result` on the WASM engine, held to the interpreter: `is_some`,
//! `is_none`, `unwrap_or`, `map`, `and_then` and `ok_or` on an `Option`; `is_ok`, `is_err`, `unwrap_or`, `ok`, `err`,
//! `map`, `map_err` and `and_then` on a `Result`. Every one is total, so no line here faults. The WASM engine compiles
//! each in the guest with no import: a test of the cell's tag, and the function of `map`, `and_then` and `map_err`
//! inlined in the one branch that calls it. So what must hold is that both engines give the same answer, run the
//! function for the same variant and the same number of times, and evaluate an argument whichever variant the receiver
//! is. Each expected answer below was worked by hand.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

/// What `main` printed on the WASM engine and on the interpreter, the program compiled with the checker's types.
fn run_both(src: &str) -> (String, String) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() })
        .unwrap_or_else(|e| panic!("the WASM engine should run it: {}\n{src}", e.message()));
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let ran = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root())));
    let i = take_capture().unwrap_or_default();
    assert!(ran.is_ok(), "the interpreter should run it: {:?}\n{src}", ran.err().map(|f| f.message.to_string()));
    (w, i)
}

/// The helpers every case may call: an `Option` and a `Result` that are each variant for some argument, a sum type as
/// an error, and a record as a payload.
const DECLS: &str = "type Bad = Missing | Broken(Str)\n\
type Point { x: Int, y: Int }\n\n\
fn half(n: Int) -> Option[Int] {\n  if n % 2 == 0 { Some(n / 2) } else { None }\n}\n\n\
fn checked(n: Int) -> Result[Int, Bad] {\n  if n < 0 { Err(Broken(\"negative\")) } else if n == 0 { Err(Missing) } else { Ok(n) }\n}\n\n\
fn show(b: Bad) -> Str {\n  match b {\n    Missing => \"missing\",\n    Broken(why) => \"broken: \" + why,\n  }\n}\n\n\
fn twice(n: Int) -> Int {\n  n * 2\n}\n\n";

fn program(body: &str) -> String {
    format!("module m\n\n{DECLS}fn main(root: Root) ! {{Write}} {{\n  let c = root.console()\n{body}\n}}\n")
}

/// Each line's answer on both engines, against the one worked by hand.
#[test]
fn every_option_and_result_answer_is_the_same_on_both_engines() {
    let cases: &[(&str, &str)] = &[
        // `Option`: the two questions, for each variant.
        ("c.println(f\"{half(8).is_some()} {half(8).is_none()} {half(7).is_some()} {half(7).is_none()}\")", "true false false true"),
        // `unwrap_or`: the payload, or the default.
        ("c.println(f\"{half(8).unwrap_or(0 - 1)} {half(7).unwrap_or(0 - 1)}\")", "4 -1"),
        // Every scalar a payload may be: a `Str`, a `Float`, a `Bool`.
        ("let n: Option[Str] = None\n  c.println(Some(\"a\").unwrap_or(\"b\") + n.unwrap_or(\"b\"))", "ab"),
        ("let n: Option[Float] = None\n  c.println(f\"{Some(1.5).unwrap_or(0.25)} {n.unwrap_or(0.25)}\")", "1.5 0.25"),
        ("let n: Option[Bool] = None\n  c.println(f\"{Some(true).unwrap_or(false)} {n.unwrap_or(true)}\")", "true true"),
        // A record, a list, a sum and an `Option` as the payload: each is a pointer in its slot.
        ("let p = Some(Point { x: 3, y: 4 }).unwrap_or(Point { x: 0, y: 0 })\n  c.println(str(p.x + p.y))", "7"),
        ("let n: Option[Point] = None\n  let p = n.unwrap_or(Point { x: 5, y: 6 })\n  c.println(str(p.x * p.y))", "30"),
        ("let xss = [[1, 2], [3]]\n  c.println(f\"{len(xss.get(0).unwrap_or([]))} {len(xss.get(5).unwrap_or([7, 7, 7]))}\")", "2 3"),
        ("let oo: Option[Option[Int]] = Some(Some(3))\n  c.println(str(oo.unwrap_or(None).unwrap_or(0)))", "3"),
        ("let n: Option[Bad] = None\n  c.println(show(Some(Missing).unwrap_or(Broken(\"x\"))) + \", \" + show(n.unwrap_or(Broken(\"x\"))))", "missing, broken: x"),
        // `map`: to the same type, to another, by a lambda and by a function's name.
        ("c.println(f\"{half(8).map(fn(x: Int) -> Int { x + 1 })} {half(7).map(fn(x: Int) -> Int { x + 1 })}\")", "Some(5) None"),
        ("c.println(str(half(8).map(fn(x: Int) -> Str { \"got \" + str(x) }).unwrap_or(\"nothing\")))", "got 4"),
        ("c.println(str(half(7).map(fn(x: Int) -> Str { \"got \" + str(x) }).unwrap_or(\"nothing\")))", "nothing"),
        ("c.println(str(half(8).map(fn(x: Int) -> Float { float(x) / 8.0 }).unwrap_or(0.0)))", "0.5"),
        ("c.println(f\"{half(8).map(twice)} {half(7).map(twice)}\")", "Some(8) None"),
        ("c.println(str(half(6).map(fn(x: Int) -> Point { Point { x: x, y: x * x } }).map(fn(p: Point) -> Int { p.y - p.x })))", "Some(6)"),
        // `and_then`: a step that may itself be absent — `half` twice is a quarter, where both divisions are exact.
        ("c.println(f\"{half(8).and_then(half)} {half(6).and_then(half)} {half(7).and_then(half)}\")", "Some(2) None None"),
        (
            "c.println(str(half(8).and_then(fn(x: Int) -> Option[Str] { if x > 3 { Some(\"big\") } else { None } }).unwrap_or(\"small\")))",
            "big",
        ),
        // `ok_or`: `Some(x)` is `Ok(x)`, `None` is `Err(err)`.
        ("c.println(f\"{half(8).ok_or(\"odd\")} {half(7).ok_or(\"odd\")}\")", "Ok(4) Err(odd)"),
        ("c.println(show(half(7).ok_or(Missing).err().unwrap_or(Broken(\"?\"))))", "missing"),
        // `Result`: the two questions, `unwrap_or`, and each side as an `Option`.
        ("c.println(f\"{checked(5).is_ok()} {checked(5).is_err()} {checked(0).is_ok()} {checked(0).is_err()}\")", "true false false true"),
        ("c.println(f\"{checked(5).unwrap_or(0 - 1)} {checked(0 - 5).unwrap_or(0 - 1)}\")", "5 -1"),
        ("c.println(f\"{checked(5).ok()} {checked(0).ok()} {checked(5).err().is_none()}\")", "Some(5) None true"),
        ("c.println(show(checked(0).err().unwrap_or(Broken(\"?\"))) + \"; \" + show(checked(0 - 1).err().unwrap_or(Missing)))", "missing; broken: negative"),
        // `map` and `map_err`: one side each, the other handed on.
        ("c.println(f\"{checked(7).map(twice).unwrap_or(0)} {checked(0 - 7).map(twice).unwrap_or(0)}\")", "14 0"),
        ("c.println(f\"{checked(0 - 7).map_err(show)} {checked(7).map_err(show)}\")", "Err(broken: negative) Ok(7)"),
        ("c.println(str(checked(7).map(fn(n: Int) -> Str { str(n) + \"!\" }).map_err(fn(b: Bad) -> Int { 0 })))", "Ok(7!)"),
        ("c.println(str(checked(0).map(fn(n: Int) -> Str { str(n) + \"!\" }).map_err(fn(b: Bad) -> Int { 40 + 2 })))", "Err(42)"),
        // `and_then`: a step that may itself fail, the same way.
        (
            "c.println(f\"{checked(3).and_then(fn(n: Int) -> Result[Int, Bad] { checked(n - 3) }).is_err()} \
             {checked(3).and_then(fn(n: Int) -> Result[Int, Bad] { checked(n + 3) }).unwrap_or(0)} \
             {checked(0).and_then(fn(n: Int) -> Result[Int, Bad] { checked(n + 3) }).is_err()}\")",
            "true 6 true",
        ),
        // What a method makes is an ordinary value: compared, matched, and a receiver again.
        ("c.println(f\"{half(8).map(twice) == Some(8)} {half(7).map(twice) == None} {half(8).ok_or(\"e\") == Ok(4)}\")", "true true true"),
        ("match half(8).map(twice) {\n    Some(n) => c.println(\"some \" + str(n)),\n    None => c.println(\"none\"),\n  }", "some 8"),
        ("match checked(0).map(twice) {\n    Ok(n) => c.println(\"ok \" + str(n)),\n    Err(b) => c.println(\"err \" + show(b)),\n  }", "err missing"),
        // In a condition, in a loop, in a function value handed to a list's `map`.
        ("if half(8).is_some() && !half(7).is_some() { c.println(\"both\") } else { c.println(\"neither\") }", "both"),
        ("var total = 0\n  for w in \"3 x 4 -1 y\".split(\" \") {\n    total += parse_int(w).unwrap_or(0)\n  }\n  c.println(str(total))", "6"),
        ("c.println(str([\"1\", \"b\", \"3\"].map(fn(w: Str) -> Int { parse_int(w).unwrap_or(0 - 1) })))", "[1, -1, 3]"),
        // A generic function's `Option`, and `?` after `ok_or`.
        ("c.println(f\"{first_or([4, 5], 0)} {first_or([4, 5].slice(2, 2), 9)}\")", "4 9"),
        ("c.println(f\"{port(\"80\")} {port(\"x\")} {port(\"0\")}\")", "Ok(80) Err(not a number) Err(out of range)"),
    ];
    for (body, want) in cases {
        let helpers = "fn first_or[T](xs: List[T], d: T) -> T {\n  xs.get(0).unwrap_or(d)\n}\n\n\
            fn port(text: Str) -> Result[Int, Str] {\n  let n = parse_int(text).ok_or(\"not a number\")?\n  \
            if n < 1 { Err(\"out of range\") } else { Ok(n) }\n}\n\n";
        let src = program(&format!("  {body}")).replace("fn main(", &format!("{helpers}fn main("));
        let (w, i) = run_both(&src);
        assert_eq!(i, format!("{want}\n"), "the interpreter:\n{src}");
        assert_eq!(w, i, "the WASM engine against the interpreter:\n{src}");
    }
}

/// A function is called for the one variant its method names, once, and for the other not at all; an argument that is
/// a value — `unwrap_or`'s default, `ok_or`'s error — is evaluated whichever variant the receiver is, after the
/// receiver. Each is seen by what it prints.
#[test]
fn a_function_runs_once_for_its_variant_and_an_argument_runs_either_way() {
    let noisy = "fn noisy(c: Cap[Console], label: Str, v: Int) -> Int ! {Write} {\n  c.println(\"eval \" + label)\n  v\n}\n\n\
        fn from(c: Cap[Console], label: Str, n: Int) -> Option[Int] ! {Write} {\n  c.println(\"recv \" + label)\n  half(n)\n}\n\n";
    let cases: &[(&str, &str)] = &[
        // The receiver first, then the default, for `Some` and for `None`.
        ("c.println(str(from(c, \"a\", 8).unwrap_or(noisy(c, \"d\", 9))))", "recv a\neval d\n4\n"),
        ("c.println(str(from(c, \"a\", 7).unwrap_or(noisy(c, \"d\", 9))))", "recv a\neval d\n9\n"),
        ("c.println(str(from(c, \"a\", 8).ok_or(noisy(c, \"e\", 9))))", "recv a\neval e\nOk(4)\n"),
        ("c.println(str(from(c, \"a\", 7).ok_or(noisy(c, \"e\", 9))))", "recv a\neval e\nErr(9)\n"),
        ("c.println(str(checked(noisy(c, \"r\", 5)).unwrap_or(noisy(c, \"d\", 9))))", "eval r\neval d\n5\n"),
        // `map` and `and_then`: for `Some` and `Ok` once, for `None` and `Err` never.
        ("c.println(str(half(8).map(fn(x: Int) -> Int ! {Write} { c.println(\"f \" + str(x)); x * 10 })))", "f 4\nSome(40)\n"),
        ("c.println(str(half(7).map(fn(x: Int) -> Int ! {Write} { c.println(\"never\"); x * 10 })))", "None\n"),
        ("c.println(str(half(8).and_then(fn(x: Int) -> Option[Int] ! {Write} { c.println(\"f \" + str(x)); half(x) })))", "f 4\nSome(2)\n"),
        ("c.println(str(half(7).and_then(fn(x: Int) -> Option[Int] ! {Write} { c.println(\"never\"); half(x) })))", "None\n"),
        ("c.println(str(checked(5).map(fn(x: Int) -> Int ! {Write} { c.println(\"f \" + str(x)); x + 1 }).unwrap_or(0)))", "f 5\n6\n"),
        ("c.println(str(checked(0).map(fn(x: Int) -> Int ! {Write} { c.println(\"never\"); x + 1 }).unwrap_or(0)))", "0\n"),
        (
            "c.println(str(checked(5).and_then(fn(x: Int) -> Result[Int, Bad] ! {Write} { c.println(\"f \" + str(x)); checked(x - 5) }).is_err()))",
            "f 5\ntrue\n",
        ),
        (
            "c.println(str(checked(0).and_then(fn(x: Int) -> Result[Int, Bad] ! {Write} { c.println(\"never\"); checked(x) }).is_err()))",
            "true\n",
        ),
        // `map_err`: for `Err` once, for `Ok` never.
        ("c.println(str(checked(0).map_err(fn(b: Bad) -> Str ! {Write} { c.println(\"g \" + show(b)); \"bad\" })))", "g missing\nErr(bad)\n"),
        ("c.println(str(checked(5).map_err(fn(b: Bad) -> Str ! {Write} { c.println(\"never\"); \"bad\" })))", "Ok(5)\n"),
        // A chain runs left to right, each function at its own step.
        (
            "c.println(str(half(8).map(fn(x: Int) -> Int ! {Write} { c.println(\"one\"); x + 2 }).and_then(fn(x: Int) -> Option[Int] ! {Write} { c.println(\"two\"); half(x) }).unwrap_or(noisy(c, \"three\", 0))))",
            "one\ntwo\neval three\n3\n",
        ),
    ];
    for (body, want) in cases {
        let src = program(&format!("  {body}")).replace("fn main(", &format!("{noisy}fn main("));
        let (w, i) = run_both(&src);
        assert_eq!(&i, want, "the interpreter:\n{src}");
        assert_eq!(w, i, "the WASM engine against the interpreter:\n{src}");
    }
}

/// A cell of the variant a method does not read is handed on, not copied: a `None` through `map`, an `Err` through
/// `map` and `and_then`, an `Ok` through `map_err`. It is then a value of the call's type, whose payload type differs
/// from the receiver's — so it must read as that type wherever a value is read: shown, compared, matched, and as a
/// receiver again.
#[test]
fn a_cell_handed_on_reads_as_the_calls_type() {
    let cases: &[(&str, &str)] = &[
        // `None`, from an `Option[Int]`, as an `Option[Str]` and then an `Option[Float]`.
        ("let o = half(7).map(fn(x: Int) -> Str { str(x) })\n  c.println(f\"{o} {o == None} {o.unwrap_or(\"dflt\")} {o.map(fn(s: Str) -> Float { 1.5 })}\")", "None true dflt None"),
        // `Err(Bad)`, from a `Result[Int, Bad]`, as a `Result[Str, Bad]`.
        ("let r = checked(0).map(fn(x: Int) -> Str { str(x) })\n  c.println(f\"{r.is_err()} {r.unwrap_or(\"dflt\")} {show(r.err().unwrap_or(Broken(\"?\")))}\")", "true dflt missing"),
        ("let r = checked(0 - 1).and_then(fn(x: Int) -> Result[Int, Bad] { checked(x + 1) })\n  match r {\n    Ok(x) => c.println(str(x)),\n    Err(b) => c.println(show(b)),\n  }", "broken: negative"),
        // `Ok(Int)`, from a `Result[Int, Bad]`, as a `Result[Int, Str]`.
        ("let r = checked(9).map_err(show)\n  c.println(f\"{r} {r == Ok(9)} {r.unwrap_or(0)} {r.err().unwrap_or(\"no error\")}\")", "Ok(9) true 9 no error"),
        // And held in a list beside one the method made.
        ("let rs = [checked(9).map_err(show), checked(0).map_err(show)]\n  c.println(str(rs))", "[Ok(9), Err(missing)]"),
    ];
    for (body, want) in cases {
        let src = program(&format!("  {body}"));
        let (w, i) = run_both(&src);
        assert_eq!(i, format!("{want}\n"), "the interpreter:\n{src}");
        assert_eq!(w, i, "the WASM engine against the interpreter:\n{src}");
    }
}

/// Both compile entries: `compile_checked` carries the checker's types and `compile_module` does not (a Verified
/// plugin's DIR recompile). None of these methods is decided from the checker's types — each reads the receiver's type
/// as the backend built it — so a program that names its types compiles to the same bytes through both.
#[test]
fn the_methods_compile_through_both_entries() {
    let src = program(
        "  let a: Option[Int] = half(8)\n  let b: Result[Int, Bad] = checked(0)\n  \
         c.println(f\"{a.is_some()} {a.unwrap_or(0)} {a.map(twice).unwrap_or(0)} {a.and_then(half).unwrap_or(0)} {a.ok_or(\"e\").is_ok()}\")\n  \
         c.println(f\"{b.is_err()} {b.unwrap_or(7)} {b.ok().is_none()} {show(b.err().unwrap_or(Broken(\"?\")))} {b.map(twice).is_err()}\")\n  \
         c.println(f\"{b.map_err(show).err().unwrap_or(\"?\")} {b.and_then(checked).is_err()} {a.is_none()} {b.is_ok()}\")",
    );
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let typed = compile_checked(&checked.module, &checked.result).expect("compiles with the checker's types");
    let untyped = compile_module(&checked.module).expect("compiles without them");
    assert_eq!(typed, untyped, "one program, one module, whichever entry compiled it");
    let out = run_main(&untyped, &HostConfig { console: true, ..HostConfig::default() }).expect("runs");
    assert_eq!(out, "true 4 8 2 true\ntrue 7 true missing true\nmissing true false false\n");
    let (w, i) = run_both(&src);
    assert_eq!(w, i);
    assert_eq!(w, out);
}

/// What stays outside the WASM fragment (RW 3.34): a function whose parameter is a `Unit` compiles nowhere on this
/// engine, so one handed to `map` or `and_then` over a `Result[Unit, E]` is refused when the program is compiled, in
/// words, and never miscompiled. The questions and `map_err` over the same `Result` need no such function and compile.
#[test]
fn a_function_of_a_unit_payload_is_refused_and_the_rest_of_a_unit_result_compiles() {
    let decls = "module m\n\nfn step(ok: Bool, c: Cap[Console]) -> Result[Unit, Str] ! {Write} {\n  \
        if ok { Ok(c.println(\"stepped\")) } else { Err(\"refused\") }\n}\n\n";
    let refused = format!(
        "{decls}fn main(root: Root) ! {{Write}} {{\n  let c = root.console()\n  \
         let r = step(true, c).and_then(fn(u: Unit) -> Result[Unit, Str] {{ Ok(u) }})\n  c.println(str(r.is_ok()))\n}}\n"
    );
    let checked = check_source(0, &refused);
    assert!(!checked.has_errors(), "the checker accepts it, and the interpreter runs it: {:?}", checked.diagnostics);
    let e = compile_checked(&checked.module, &checked.result).expect_err("outside the WASM fragment");
    assert!(e.message().contains("a payload of Unit"), "the refusal names what it cannot compile: {}", e.message());

    let fine = format!(
        "{decls}fn main(root: Root) ! {{Write}} {{\n  let c = root.console()\n  let a = step(true, c)\n  let b = step(false, c)\n  \
         c.println(f\"{{a.is_ok()}} {{b.is_err()}} {{a.ok().is_some()}} {{b.err().unwrap_or(\"none\")}}\")\n  \
         c.println(str(b.map_err(fn(e: Str) -> Int {{ len(e) }}).err().unwrap_or(0)))\n  \
         c.println(str(a.map_err(fn(e: Str) -> Int {{ len(e) }}).is_ok()))\n}}\n"
    );
    let (w, i) = run_both(&fine);
    assert_eq!(i, "stepped\ntrue true true refused\n7\ntrue\n");
    assert_eq!(w, i);
}
