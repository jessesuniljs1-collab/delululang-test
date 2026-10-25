//! A block whose last statement is `return` (found 2026-10-25 by on-demand routine run 6, checking the agent test campaign's
//! A9). `return` leaves the function, so the block it ends produces no value; the checker typed it `Unit`, and a function
//! ending `return x / 2` was refused DL0401 ("expected `Int`, found `Unit`") though every early `return` before it checked.
//! Such a block now takes whatever type its context expects — a fresh variable — and the returned value is still held to
//! the function's return type.

use delulu_check::check_source;

fn errors(src: &str) -> Vec<(String, String)> {
    check_source(0, src).diagnostics.iter().filter(|d| d.is_error()).map(|d| (d.code.to_string(), d.message.clone())).collect()
}

fn clean(src: &str) {
    let es = errors(src);
    assert!(es.is_empty(), "expected clean, got {es:?}\nsource:\n{src}");
}

fn refused(src: &str, code: &str) {
    let es = errors(src);
    assert!(es.iter().any(|(c, _)| c == code), "expected {code}, got {es:?}\nsource:\n{src}");
}

#[test]
fn a_function_may_end_with_return() {
    clean("module m\nfn half(x: Int) -> Int {\n    if x < 0 {\n        return 0\n    }\n    return x / 2\n}\nfn main() { }\n");
    clean("module m\nfn name() -> Str {\n    return \"n\"\n}\nfn main() { }\n");
    // A branch may end in `return` beside one that yields the value.
    clean("module m\nfn f(c: Bool) -> Int {\n    let y = if c { return 1 } else { 2 }\n    y + 1\n}\nfn main() { }\n");
    clean("module m\nfn g() {\n    return\n}\nfn main() { }\n");
}

/// The agent test campaign's A9: `return` where an expression is expected — `None => return Err(…)` in a match arm — was
/// DL0202 "expected an expression, found `return`" and ten diagnostics of cascade. It is that statement in a block of its
/// own now, and checks as one.
#[test]
fn return_where_an_expression_is_expected_is_a_block_that_returns() {
    clean(
        "module m\ntype E = Lex(Str)\nfn d(s: Str) -> Result[Int, E] {\n    let n = match parse_int(s) {\n        Some(v) => v,\n        \
         None => return Err(Lex(s))\n    }\n    Ok(n)\n}\nfn main() { }\n",
    );
    clean("module m\nfn h(o: Option[Str]) -> Str {\n    let s = match o {\n        None => return \"early\",\n        Some(v) => v\n    }\n    s\n}\nfn main() { }\n");
    // Held to the return type all the same: an arm returning the wrong type is refused, and nothing cascades from it.
    let es = errors("module m\nfn h(o: Option[Str]) -> Str {\n    match o {\n        None => return 5,\n        Some(v) => v\n    }\n}\nfn main() { }\n");
    assert_eq!(es.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(), ["DL0401"], "one diagnostic, the type's: {es:?}");
}

#[test]
fn what_is_returned_is_still_held_to_the_return_type() {
    refused("module m\nfn half(x: Int) -> Int {\n    return \"two\"\n}\nfn main() { }\n", "DL0401");
    refused("module m\nfn f(c: Bool) -> Int {\n    if c { return 1 } else { \"two\" }\n}\nfn main() { }\n", "DL0401");
    // A block that does not end in `return` is typed by its last expression, as before.
    refused("module m\nfn f() -> Int {\n    let x = 1\n}\nfn main() { }\n", "DL0401");
}
