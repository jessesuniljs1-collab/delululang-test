//! Five refusals that named the wrong mistake (the agent test campaign of 2026-10-24: the builder's A6, and the zero-authority
//! agent's C7, C8, C10 and C16). Each keeps its code; what it says is now the mistake that was made:
//! - A6: `"a" < "b"` was "arithmetic expects Int or Float", once per side;
//! - C7: `load("x.dpx")` was called a forged capability beside its count of arguments;
//! - C8: a function kept in a record field, called as a method, was an unknown method, with no word of how it is called;
//! - C10: `tag` was "reserved for a future stage", though the six reference capabilities have been in use since Stage 7;
//! - C16: `const N: Int = 3`, the name the grammar index gives a module constant, was "expected an item".

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

#[test]
fn a_reference_capability_is_refused_as_one_and_a_future_word_as_one() {
    let d = diags("module m\ntype Logger {\n    tag: Str,\n}\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0106" && m.starts_with("`tag` is a reference capability")), "{d:?}");
    let d = diags("module m\nfn secret() { }\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0106" && m.starts_with("`secret` is reserved for a future stage")), "{d:?}");
}

#[test]
fn const_at_the_top_of_a_module_says_a_constant_is_written_with_let() {
    let d = diags("module m\nconst N: Int = 3\nfn main() { }\n");
    assert!(d.iter().any(|(c, m)| *c == "DL0208" && m.contains("written with `let`, not `const`")), "{d:?}");
    // What `let` writes checks clean.
    assert!(diags("module m\nlet N: Int = 3\nfn main() { }\n").is_empty());
}
