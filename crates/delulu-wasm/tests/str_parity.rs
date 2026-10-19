//! RW 3.4 — the `Str` methods on the WASM engine, held to the interpreter: `trim`, `to_upper`, `to_lower`, `chars`,
//! `contains`, `starts_with`, `replace` and `slice` run host-side through the interpreter's own `call_str_method`
//! (`delulu:str.str_method`), so the answers are the interpreter's — Unicode whitespace, case mappings that change a
//! string's length, character-indexed slices clamped as the interpreter clamps them, and an empty `replace` pattern
//! that leaves the receiver alone. With `len` (`__str_len`) and `split` (`delulu:list.str_split`), every `Str` method
//! the checker knows compiles.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_module, run_main, HostConfig};

/// Run `main` on both engines with the console and return `(wasm, interp)`: the output, or the fault's text.
fn run_both(src: &str) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
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

/// Each method, at the edges the interpreter defines: whitespace beyond ASCII, `ß` uppercasing to two letters, a
/// precomposed `é` that does not contain `e`, the empty pattern and the empty string, and slices whose bounds are
/// negative, past the end, or crossed.
#[test]
fn each_str_method_matches_the_interpreter() {
    let src = r#"module m

fn b(x: Bool) -> Str {
  if x { "1" } else { "0" }
}

fn show(xs: List[Str]) -> Str {
  var s = str(xs.len()) + ":"
  for x in xs { s = s + "[" + x + "]" }
  s
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println("[" + "  padded\t\n".trim() + "][" + "\u{3000}wide\u{3000}".trim() + "][" + "".trim() + "]")
  out.println("straße".to_upper() + " " + "ÀÉÎ Ωmega".to_lower() + " " + "MiXeD 123".to_upper())
  out.println(show("héllo ✓".chars()) + " " + show("".chars()))
  out.println(b("haystack".contains("st")) + b("haystack".contains("")) + b("é".contains("e")) + b("abc".starts_with("ab")) + b("abc".starts_with("bc")) + b("".starts_with("")))
  out.println("a-b-c".replace("-", "+") + " " + "aaa".replace("a", "bb") + " " + "keep".replace("", "x") + " " + "ñandú".replace("ú", "u"))
  out.println("[" + "héllo ✓".slice(1, 4) + "][" + "abc".slice(0 - 5, 2) + "][" + "abc".slice(2, 99) + "][" + "abc".slice(2, 1) + "]")
}
"#;
    assert_eq!(
        assert_parity(src),
        "[padded][wide][]\nSTRASSE àéî ωmega MIXED 123\n7:[h][é][l][l][o][ ][✓] 0:\n110101\na+b+c bbbbbb keep ñandu\n\
         [éll][ab][c][]\n"
    );
}

/// The methods inside the rest of the fragment: chained, in a pure function, on a word split from a line or taken from a
/// list, in a `for` over `chars`, with arguments computed from variables — and their answers compared and counted.
#[test]
fn str_methods_compose_with_the_rest_of_the_fragment() {
    let src = r#"module m

fn norm(s: Str) -> Str {
  s.trim().to_lower()
}

fn initials(line: Str) -> Str {
  var out = ""
  for w in line.split(" ") {
    if w.len() > 0 {
      out = out + w.slice(0, 1).to_upper()
    }
  }
  out
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let n = norm("  Hello, World  ")
  out.println(n + " " + str(n.len()) + " " + str(n == "hello, world"))
  out.println(initials("grace  brewster murray hopper"))
  var vowels = 0
  for c in "Programming Language".to_lower().chars() {
    if "aeiou".contains(c) { vowels = vowels + 1 }
  }
  out.println(str(vowels))
  let k = 3
  out.println("abcdefgh".slice(k - 1, k * 2) + " " + "x.y.z".replace(".", "::").to_upper())
  let words = "b a c".split(" ")
  let first = words[0]
  out.println(first.replace(first, "z") + " " + str(first.starts_with("b")) + " " + str(words[2].contains(words[1])))
}
"#;
    assert_eq!(assert_parity(src), "hello, world 12 true\nGBMH\n7\ncdef X::Y::Z\nz true false\n");
}

/// A module that calls no `Str` method imports no `delulu:str`, so it is byte-for-byte what it was before the interface
/// existed; one that calls a method imports it.
#[test]
fn only_a_module_that_calls_a_str_method_imports_the_interface() {
    let imports = |src: &str| {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile: {e:?}\n{src}"));
        wasm.windows(b"delulu:str".len()).any(|w| w == b"delulu:str")
    };
    assert!(!imports("module m\nfn f(s: Str) -> Int {\n  s.len()\n}\n"));
    assert!(!imports("module m\nfn f(s: Str) -> Int {\n  s.split(\",\").len()\n}\n"));
    assert!(imports("module m\nfn f(s: Str) -> Int {\n  s.trim().len()\n}\n"));
    assert!(imports("module m\nfn f(s: Str) -> Bool {\n  s.contains(\"x\")\n}\n"));
}
