//! Five refusals that named the wrong mistake (the agent test campaign of 2026-10-24: the builder's A6, and the zero-authority
//! agent's C7, C8, C10 and C16). Each keeps its code; what it says is now the mistake that was made:
//! - A6: `"a" < "b"` was "arithmetic expects Int or Float", once per side;
//! - C7: `load("x.dpx")` was called a forged capability beside its count of arguments;
//! - C8: a function kept in a record field, called as a method, was an unknown method, with no word of how it is called;
//! - C10: `tag` was "reserved for a future stage", though the six reference capabilities have been in use since Stage 7;
//! - C16: `const N: Int = 3`, the name the grammar index gives a module constant, was "expected an item".
//!
//! And two that said one mistake more than once (C12, C13):
//! - C12: a name in another script (`fn 计算`) was refused a character at a time, then again by the parser as a missing name;
//! - C13: where the parser could not read an expression, the `0` it put there was typed `Int`, so a condition 200 `if`s deep
//!   was also refused "`if` condition must be Bool" at the place DL0210 had already named.
//!
//! And C6: a write refused through a binding that is `val` by its written type now says that the type made it so.

use delulu_check::check_source;

/// Every diagnostic `src` draws, as `(code, message)`.
fn diags(src: &str) -> Vec<(&'static str, String)> {
    check_source(0, src).diagnostics.into_iter().map(|d| (d.code, d.message)).collect()
}

#[test]
fn comparing_two_strs_is_one_refusal_that_says_a_str_has_no_order() {
    let d = diags("module m\nfn f() -> Bool {\n    \"a\" < \"b\"\n}\nfn main() { }\n");
    assert_eq!(d.len(), 1, "one refusal for one comparison: {d:?}");
    assert_eq!(d[0].0, "DL0401");
    assert!(d[0].1.contains("`Str` has no order"), "{d:?}");
    // Any other comparison of the wrong type says comparison; arithmetic keeps its word.
    let d = diags("module m\nfn f() -> Bool {\n    true < 1\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0401" && m.starts_with("comparison expects Int or Float")), "{d:?}");
    assert!(!d.iter().any(|(_, m)| m.contains("arithmetic")), "{d:?}");
    let d = diags("module m\nfn f() -> Int {\n    true + 1\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0401" && m.starts_with("arithmetic expects Int or Float")), "{d:?}");
}

#[test]
fn load_with_the_wrong_count_is_refused_for_the_count_alone() {
    let d = diags("module m\nfn f() ! {Load, Read} {\n    let p = load(\"x.dpx\")\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0403" && m.contains("expects 3 argument(s), found 1")), "{d:?}");
    assert!(!d.iter().any(|(c, _)| *c == "DL0601"), "a path is not a forged capability: {d:?}");
    // With the right count, each argument is still held to its parameter.
    let d = diags("module m\nfn f() ! {Load, Read} {\n    let p = load(\"x.dpx\", \"x.dpx\", 3)\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, _)| *c == "DL0601"), "a `Str` where the host goes is still refused: {d:?}");
}

#[test]
fn a_function_in_a_record_field_called_as_a_method_says_how_it_is_called() {
    let src = "module m\ntype Handler {\n    run: fn(Int) -> Int,\n    n: Int,\n}\n\
               fn fire(h: Handler) -> Int {\n    h.run(1)\n}\nfn count(h: Handler) -> Int {\n    h.n(1)\n}\nfn main() { }\n";
    let d = diags(src);
    assert!(
        d.iter().any(|(c, m)| *c == "DL0405" && m.contains("`run` is a field holding a function, called as `(h.run)(…)`")),
        "{d:?}"
    );
    // A field that holds no function is no method either, and nothing says to call it.
    assert!(d.iter().any(|(c, m)| *c == "DL0405" && m == "type `Handler` has no method `n`"), "{d:?}");
}

/// Each of the three kinds of reserved word `token::RESERVED` keeps gives its own reason.
#[test]
fn a_reserved_word_is_refused_for_its_own_reason() {
    let d = diags("module m\ntype Logger {\n    tag: Str,\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0106" && m.starts_with("`tag` is a reference capability")), "{d:?}");
    let d = diags("module m\nfn secret() { }\nfn main() { }\n");
    assert!(
        d.iter().any(|(c, m)| *c == "DL0106" && m.contains("`secret` is reserved so that no declared name can pose as an")),
        "{d:?}"
    );
    let d = diags("module m\nfn trait() { }\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0106" && m.starts_with("`trait` is reserved for a future stage")), "{d:?}");
}

#[test]
fn const_at_the_top_of_a_module_says_a_constant_is_written_with_let() {
    let d = diags("module m\nconst N: Int = 3\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0208" && m.contains("written with `let`, not `const`")), "{d:?}");
    // What `let` writes checks clean.
    assert!(diags("module m\nlet N: Int = 3\nfn main() { }\n").is_empty());
}

#[test]
fn a_name_in_another_script_is_refused_once_as_a_name() {
    let d = diags("module m\nfn 计算(x: Int) -> Int {\n    x + 1\n}\nfn main() {\n    let café = 计算(1)\n}\n");
    let words: Vec<&String> = d.iter().filter(|(c, _)| *c == "DL0101").map(|(_, m)| m).collect();
    assert_eq!(words.len(), 3, "one refusal per word written, each where it is: {d:?}");
    assert!(words.iter().all(|m| m.contains("is not a name: identifiers are ASCII")), "{d:?}");
    assert!(words.iter().any(|m| m.starts_with("`计算`")) && words.iter().any(|m| m.starts_with("`café`")), "{d:?}");
    assert!(!d.iter().any(|(c, _)| *c == "DL0201"), "no name is missing after it: {d:?}");
}

#[test]
fn where_an_expression_could_not_be_read_its_placeholder_draws_no_refusal() {
    // 200 `if`s deep: DL0210 and DL0213 say it, and nothing says the condition is an `Int`. Checked on a stack as large as
    // the CLI's `delulu-main` gives it: a debug build's test thread (2 MiB) is not where this depth is meant to be read.
    let deep = format!("module m\nfn main() {{\n{}1{}\n}}\n", "if true {\n".repeat(200), "\n}".repeat(200));
    let d = std::thread::Builder::new().stack_size(512 << 20).spawn(move || diags(&deep)).unwrap().join().unwrap();
    assert!(d.iter().any(|(c, _)| *c == "DL0210"), "{d:?}");
    assert!(!d.iter().any(|(c, _)| *c == "DL0408"), "the placeholder is no `Int`: {d:?}");
    // An expression missing where a condition goes.
    let d = diags("module m\nfn f() -> Int {\n  if ) {\n    1\n  } else {\n    2\n  }\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, _)| *c == "DL0202"), "{d:?}");
    assert!(!d.iter().any(|(c, _)| *c == "DL0408"), "{d:?}");
}

/// C6 (the zero-authority agent): `let b: List[Int] = []` then `b.push(2)` was refused "cannot mutate through a `val`
/// reference", though nothing in the program said `val` — the written type did, by naming no reference capability.
#[test]
fn a_write_refused_through_a_written_type_names_the_type() {
    let src = "module m\nfn main() {\n    let b: List[Int] = []\n    b.push(2)\n}\n";
    let d = check_source(0, src).diagnostics.into_iter().find(|d| d.code == "DL1604").expect("DL1604 at the write");
    assert!(d.message.contains("`b`'s written type names no reference capability"), "{}", d.message);
    assert!(d.message.contains("let b: ref"), "the remedy: {}", d.message);
    let at = d.spans.iter().find(|s| s.secondary).expect("a second span, at the type");
    assert_eq!(&src[at.span.start as usize..at.span.end as usize], "List[Int]", "{:?}", d.spans);
    assert_eq!(at.label.as_deref(), Some("`b` is `val` by this type"));
    // `ref` keeps the write; leaving the type out does too.
    for ok in ["let b: ref List[Int] = []", "let b = []"] {
        let d = diags(&format!("module m\nfn main() {{\n    {ok}\n    b.push(2)\n}}\n"));
        assert!(d.is_empty(), "{ok}: {d:?}");
    }
    // A `val` written in words needs no reason given.
    let d = diags("module m\nfn main() {\n    let b: val List[Int] = []\n    b.push(2)\n}\n");
    let (_, m) = d.iter().find(|(c, _)| *c == "DL1604").expect("DL1604");
    assert!(!m.contains("written type"), "{m}");
    // A binding `val` by its type and then passed to a `val` parameter was `val` before the call: the type is the reason.
    let d =
        diags("module m\nfn keep(xs: List[Int]) { }\nfn main() {\n    let b: List[Int] = []\n    keep(b)\n    b.push(2)\n}\n");
    let (_, m) = d.iter().find(|(c, _)| *c == "DL1604").expect("DL1604");
    assert!(m.contains("written type") && !m.contains("became `val`"), "{m}");
}

/// C9 (the agent test campaign): a keyword in a module's name was *"expected end of statement"*: at the `.` before it
/// (`module t.actor`), or, for a first part, at the next line's item (`module fn`), since the line break after a keyword
/// does not end a statement. It is refused as a keyword, once, at the keyword, and the next line's item is still read; an
/// `import` the same. What read before still reads: `true`, a contextual word, a dotted name.
#[test]
fn a_keyword_in_a_module_name_is_refused_as_a_keyword() {
    for (head, word) in
        [("module t.actor", "actor"), ("module fn", "fn"), ("module a.type.b", "type"), ("module m\nimport a.actor", "actor")]
    {
        let src = format!("{head}\n\nfn main() {{ }}\n");
        let checked = check_source(0, &src);
        let errors: Vec<_> = checked.diagnostics.iter().filter(|d| d.is_error()).collect();
        assert_eq!(errors.len(), 1, "{head}: one refusal: {errors:?}");
        let message = format!("`{word}` is a keyword and cannot be part of a module's name");
        assert_eq!((errors[0].code, errors[0].message.as_str()), ("DL0201", message.as_str()), "{head}");
        let at = errors[0].spans[0].span;
        assert_eq!(&src[at.start as usize..at.end as usize], word, "{head}: placed at the keyword");
        let main = |it: &delulu_syntax::ast::Item| matches!(it, delulu_syntax::ast::Item::Fn(f) if f.name.name == "main");
        assert!(checked.module.items.iter().any(main), "{head}: the next line's `fn main` is still read");
    }
    for ok in ["module true", "module new", "module a.new", "module ok.fine"] {
        let d = diags(&format!("{ok}\n\nfn main() {{ }}\n"));
        assert!(d.is_empty(), "{ok} reads as it did: {d:?}");
    }
}

/// RW 6.41 (seen while closing B5): a write through an index on a `val` list was *"cannot write a element through a `val`
/// receiver"*. It names a list element now, as the other writes to a list do, and every DL1604 reads as a sentence.
#[test]
fn a_write_through_an_index_names_a_list_element() {
    let d = diags("module m\nfn main() {\n    let xs: val List[Int] = [1, 2]\n    xs[0] = 3\n}\n");
    let (_, m) = d.iter().find(|(c, _)| *c == "DL1604").expect("DL1604 at the write");
    assert!(m.starts_with("cannot write a list element through a `val` receiver"), "{m}");
    for (src, what) in [
        ("let xs: val List[Int] = [1, 2]\n    xs[0] = 3", "an index"),
        ("let xs: val List[Int] = [1, 2]\n    xs.push(3)", "a push"),
        ("let m: val Map[Str, Int] = Map()\n    m.insert(\"a\", 1)", "an insert"),
    ] {
        let d = diags(&format!("module m\nfn main() {{\n    {src}\n}}\n"));
        let (_, m) = d.iter().find(|(c, _)| *c == "DL1604").unwrap_or_else(|| panic!("{what}: {d:?}"));
        assert!(!m.contains(" a e") && !m.contains(" a a") && !m.contains(" a i"), "{what}: an article before a vowel: {m}");
    }
}
