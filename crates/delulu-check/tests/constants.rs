//! Top-level constants (`let NAME[: T] = value` at module level) are checked (CONST-UNCHECKED-1, 2026-10-24).
//!
//! Found by the agent test campaign's zero-authority agent (Haiku 5.5): the checker's item loop matched functions,
//! actors and tests and passed every other item with `_ => {}`, so a constant's value was never type-checked, never
//! name-resolved and never held to a row. The runtime evaluates every constant before `main`, on the comment "consts
//! are pure (checker invariant)" — an invariant nothing checked. `let Y: Int = "not an int"` checked clean and ran;
//! `let W = spawn Worker()` performed `Async` that `authority` never named; `let T = root.clock().now_ms()` checked
//! clean and faulted at run time ("unbound name `root`").

use delulu_check::check_source;

fn errors(src: &str) -> Vec<(String, String)> {
    check_source(0, src).diagnostics.iter().filter(|d| d.is_error()).map(|d| (d.code.to_string(), d.message.clone())).collect()
}

fn refused_with(src: &str, code: &str, words: &str) {
    let es = errors(src);
    assert!(
        es.iter().any(|(c, m)| c == code && m.contains(words)),
        "expected {code} saying `{words}`, got {es:?}\nsource:\n{src}"
    );
}

fn clean(src: &str) {
    let es = errors(src);
    assert!(es.is_empty(), "expected clean, got {es:?}\nsource:\n{src}");
}

#[test]
fn a_constant_value_must_match_its_declared_type() {
    refused_with("module m\nlet Y: Int = \"not an int\"\nfn main() { }\n", "DL0401", "constant");
}

#[test]
fn a_constant_value_names_only_what_is_in_scope() {
    refused_with("module m\nlet X = nosuch_name\nfn main() { }\n", "DL0301", "nosuch_name");
    // `root` is `main`'s parameter: a constant, evaluated before `main`, holds no authority.
    refused_with("module m\nlet T = root.clock().now_ms()\nfn main(root: Root) { }\n", "DL0301", "root");
}

#[test]
fn a_constant_value_performs_no_effect() {
    refused_with(
        "module m\n\
         actor Worker {\n\
         \x20 var done: Int\n\
         \x20 new() { self.done = 0 }\n\
         \x20 be job(n: Int) { self.done = self.done + n }\n\
         }\n\
         let W = spawn Worker()\n\
         fn main() { }\n",
        "DL0501",
        "Async",
    );
}

#[test]
fn an_unannotated_constant_has_one_type_at_every_use() {
    // Its uses were each given a fresh type, so `X + "a"` with `X = 1` checked clean and failed at run time.
    refused_with("module m\nlet X = 1\nfn f() -> Str { X + \"a\" }\n", "DL0401", "");
    clean("module m\nlet X = 1\nfn f() -> Int { X + 2 }\n");
}

#[test]
fn honest_constants_still_check() {
    clean(
        "module m\n\
         let BASE: Str = \"https://api.example.com/v1\"\n\
         let LIMIT = 10\n\
         let NAMES: List[Str] = [\"a\", \"b\"]\n\
         let DOUBLE = fn(x: Int) -> Int { x * 2 }\n\
         fn twice() -> Int { DOUBLE(LIMIT) }\n\
         fn url() -> Str { BASE + \"/items\" }\n\
         fn count() -> Int { NAMES.len() }\n",
    );
}
