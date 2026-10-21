//! RW 3.4 — `Map` on the WASM engine, held to the interpreter: `Map()`, its eight methods (`len`, `is_empty`, `get`,
//! `contains_key`, `insert`, `remove`, `keys`, `values`), `str` and `==` of a map, and the interpreter's sharing (`let n = m`
//! names one map). A map lives in the guest's memory and the host finds and places each key by the interpreter's own
//! `MapKey` order (`delulu:map`), so the two engines cannot order keys two ways — a key built at run time, a non-ASCII one
//! and an upper-case one included. A hand-written module's map is read as untrusted: a header or pairs outside the memory
//! are refused in words, never a panic and never a read past the memory.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_int_fn, run_main, HostConfig};

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

/// Each key kind (`Str`, `Int`, `Bool`) and value kind (`Int`, `Bool`, `Float`), every method, a key inserted twice (its
/// value replaced in place), a removed key removed again (`None`), a map passed to a function, a second name for one map,
/// and `str`, `==` and `!=` of maps.
#[test]
fn maps_of_each_key_kind_match_the_interpreter() {
    let src = r#"module m

fn total(m: ref Map[Str, Int]) -> Int {
  var t = 0
  for k in m.keys() {
    match m.get(k) {
      Some(v) => { t = t + v }
      None => { t = t }
    }
  }
  t
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let m: ref Map[Str, Int] = Map()
  out.println(str(m.len()) + " " + str(m.is_empty()) + " " + str(m))
  m.insert("pear", 3)
  m.insert("apple", 1)
  m.insert("zebra", 26)
  m.insert("mango", 13)
  m.insert("apple", 100)
  out.println(str(m.len()) + " " + str(m.is_empty()) + " " + str(m))
  out.println(str(m.keys()) + " " + str(m.values()))
  out.println(str(m.get("pear")) + " " + str(m.get("kiwi")) + " " + str(m.contains_key("zebra")) + " " + str(m.contains_key("kiwi")))
  out.println(str(m.contains_key("apple")) + " " + str(m.get("apple")))
  out.println(str(m.remove("mango")) + " " + str(m.remove("mango")) + " " + str(m))
  out.println(str(total(m)))
  let n: ref Map[Str, Int] = m
  n.insert("banana", 2)
  out.println(str(m.len()) + " " + str(m))
  let ints: ref Map[Int, Bool] = Map()
  var i = 0
  while i < 20 {
    ints.insert(19 - i, i % 2 == 0)
    i = i + 1
  }
  out.println(str(ints) + " " + str(ints.get(7)) + " " + str(ints.get(-1)))
  let fl: ref Map[Bool, Float] = Map()
  fl.insert(true, 1.5)
  fl.insert(false, 0.25)
  out.println(str(fl) + " " + str(fl.keys()))
  let other: ref Map[Str, Int] = Map()
  other.insert("apple", 100)
  out.println(str(m == other) + " " + str(m != other))
  other.insert("pear", 3)
  other.insert("zebra", 26)
  other.insert("banana", 2)
  out.println(str(m == other) + " " + str(m != other))
}
"#;
    let out = assert_parity(src);
    assert!(out.contains("{apple: 100, mango: 13, pear: 3, zebra: 26}"), "ascending by key, `apple` replaced in place: {out}");
    assert!(out.contains("Some(13) None {apple: 100, pear: 3, zebra: 26}"), "{out}");
    assert!(out.contains("true Some(100)"), "the first pair is found: {out}");
    assert!(out.contains("4 {apple: 100, banana: 2, pear: 3, zebra: 26}"), "one map under two names: {out}");
}

/// A map grown well past its first data (four pairs, doubled as it fills), its keys inserted out of order and half of them
/// removed: the order is the interpreter's at every size. Keys built at run time find the pair a literal made, and
/// `Str` keys order by their bytes — upper case before lower, `é` after `z` — as the interpreter's `BTreeMap` orders them.
#[test]
fn a_map_grows_and_keeps_the_interpreters_order() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let m: ref Map[Int, Int] = Map()
  var i = 0
  while i < 300 {
    m.insert((i * 37) % 300, i)
    i = i + 1
  }
  var j = 0
  while j < 300 {
    if j % 2 == 1 {
      let gone = m.remove(j)
    }
    j = j + 3
  }
  out.println(str(m.len()) + " " + str(m.get(0)) + " " + str(m.get(3)) + " " + str(m.get(299)))
  out.println(str(m.keys()))
  let s: ref Map[Str, Int] = Map()
  s.insert("zeta", 1)
  s.insert("éclair", 2)
  s.insert("Zeta", 3)
  s.insert("", 4)
  s.insert("ze", 5)
  s.insert("z" + "eta", 6)
  out.println(str(s) + " " + str(s.get("ze" + "ta")) + " " + str(s.contains_key("")))
}
"#;
    let out = assert_parity(src);
    assert!(out.contains("{: 4, Zeta: 3, ze: 5, zeta: 6, éclair: 2}"), "bytes order, a built key replacing a literal one: {out}");
}

/// A map of records holds the records themselves, as the interpreter's does: a record written through its own name after
/// it was inserted is the record the map holds, and a map of sums carries them whole.
#[test]
fn a_map_of_records_shares_its_values_as_the_interpreters() {
    let src = r#"module m

type Acct {
  owner: Str,
  balance: Int,
}

type Tier = Free | Paid(Int)

fn main(root: Root) ! {Write} {
  let out = root.console()
  let book: ref Map[Str, ref Acct] = Map()
  let a: ref Acct = Acct { owner: "ada", balance: 10 }
  book.insert("ada", a)
  book.insert("bob", Acct { owner: "bob", balance: 5 })
  a.balance = a.balance + 90
  out.println(str(book.get("ada")) + " " + str(book))
  let tiers: ref Map[Int, Tier] = Map()
  tiers.insert(2, Paid(9))
  tiers.insert(1, Free)
  out.println(str(tiers) + " " + str(tiers.values()) + " " + str(tiers.get(2) == Some(Paid(9))))
}
"#;
    let out = assert_parity(src);
    assert!(out.starts_with("Some(Acct { owner: ada, balance: 100 })"), "the map holds the record written through: {out}");
}

/// A `.dwx` can be written by hand, so the host reads a map as untrusted: a header outside the memory, a header whose
/// length passes its capacity, pairs past the memory and a count the memory cannot hold are each refused in words
/// (DL0903) — never a panic, never a read past the memory. (That the pairs are read before anything is reserved for them
/// is not seen here: an operating system grants an untouched reservation, so reserving first refuses alike — M725.)
#[test]
fn a_hand_written_map_outside_the_memory_is_refused_in_words() {
    let module = |body: &str, data: &str| {
        format!(
            r#"(module
  (import "delulu:map" "find" (func $find (param i32 i32 i64) (result i32)))
  (import "delulu:map" "put" (func $put (param i32 i32 i64) (result i32)))
  (import "delulu:map" "keys" (func $keys (param i32) (result i32)))
  (memory (export "memory") 1)
  (global (export "__heap") (mut i32) (i32.const 1024))
  (data (i32.const 0) "{data}")
  (func (export "main") (param i32) {body}))"#
        )
    };
    let cases = [
        // A header at the memory's last bytes: its 12 bytes pass the end.
        ("a header outside", module("(drop (call $find (i32.const 65530) (i32.const 0) (i64.const 1)))", "")),
        // len 2, cap 1.
        (
            "a length past its capacity",
            module(
                "(drop (call $put (i32.const 0) (i32.const 0) (i64.const 1)))",
                "\\02\\00\\00\\00\\01\\00\\00\\00\\40\\00\\00\\00",
            ),
        ),
        // One pair, its data at 0xFFFF0000.
        (
            "pairs past the memory",
            module(
                "(drop (call $find (i32.const 0) (i32.const 0) (i64.const 1)))",
                "\\01\\00\\00\\00\\01\\00\\00\\00\\00\\00\\ff\\ff",
            ),
        ),
        // 2^32 - 1 pairs claimed.
        (
            "a count the memory cannot hold",
            module("(drop (call $keys (i32.const 0)))", "\\ff\\ff\\ff\\ff\\ff\\ff\\ff\\ff\\10\\00\\00\\00"),
        ),
    ];
    for (what, wat) in cases {
        let got = run_main(wat.as_bytes(), &HostConfig { console: true, ..HostConfig::default() });
        let err = got.expect_err(what).message();
        assert!(err.contains("DL0903") && err.contains("a map's pairs lie outside the guest's memory"), "{what}: {err}");
    }
}

/// WASM-IMPORTS-1: a module whose only imports are a map's — a pure function that makes and fills one, with no console
/// and no list — is one the engine can load. The import section was written only for a list of interfaces that left
/// `delulu:map` out, so this module was refused: `failed to parse WebAssembly module`.
#[test]
fn a_module_whose_only_imports_are_a_maps_loads() {
    let src = "module m
fn f() -> Int {
  let m: ref Map[Int, Int] = Map()
  m.insert(1, 2)
  m.insert(3, 4)
  m.insert(1, 5)
  m.len()
}
";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let wasm = compile_checked(&checked.module, &checked.result).expect("should compile to WASM");
    assert_eq!(run_int_fn(&wasm, "f", &[]).expect("the module loads and runs"), 2);
}
