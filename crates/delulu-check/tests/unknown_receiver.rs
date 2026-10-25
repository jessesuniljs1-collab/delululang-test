//! A method called on a value whose type is not known yet (the agent test campaign's A12, the builder, Sonnet 5.5). `let m =
//! Map()` and then `old.concat([w])` on what `m.get(key)` holds was refused "type `'t9` has no method `concat`" — an
//! inference variable's internal name, and no word of the remedy. The refusal is the same code, DL0405, and now says the
//! type is not known at that call and where to write it.

use delulu_check::check_source;

#[test]
fn a_method_on_a_value_of_unknown_type_says_so_and_names_no_internal_variable() {
    let src = "module g\nfn group(words: List[Str]) -> Int {\n    let m = Map()\n    for w in words {\n        match m.get(w) {\n            \
               Some(old) => m.insert(w, old.concat([w])),\n            None => m.insert(w, [w])\n        }\n    }\n    m.len()\n}\nfn main() { }\n";
    let d = check_source(0, src).diagnostics.into_iter().find(|d| d.code == "DL0405").expect("DL0405 at the call");
    assert!(!d.message.contains('\''), "no inference variable's name: {}", d.message);
    assert!(d.message.contains("`concat`") && d.message.contains("not known"), "{}", d.message);
    assert!(d.message.contains("let m: Map["), "where to write the type: {}", d.message);
    // A known type keeps its words.
    let d = check_source(0, "module g\nfn f(x: Int) -> Int { x.concat([1]) }\nfn main() { }\n").diagnostics;
    assert!(d.iter().any(|d| d.code == "DL0405" && d.message == "type `Int` has no method `concat`"), "{d:?}");
}
