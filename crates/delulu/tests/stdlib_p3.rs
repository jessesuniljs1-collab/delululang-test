//! P3 (ruling D-V2-29): the standard library's new surface, and the gates that keep the promises
//! the reference makes about it.
//!
//! The conformance corpus already witnesses that every new primitive *resolves* and that each one
//! refuses what it should (`tests/conformance/{accept,reject}`, 15 + 8 anchors, positive and
//! negative). This file is for the claims a conformance program cannot state: that two spellings of
//! one operation agree, that an ORDER is the order the reference promises, that the registries two
//! passes read are the same set, and that the R-4 callback position is where the type checker
//! actually looks.
//!
//! Every refusal here is paired with the control that shows the same program succeeding once the
//! cause is removed. A refusal with no control is indistinguishable from a compiler that refuses
//! everything.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary runs")
}

/// Write `src` to a scratch file and run it, returning `(exit code, stdout)`.
fn run(name: &str, src: &str, grants: &[&str]) -> (i32, String) {
    let dir = std::env::temp_dir().join("delulu-stdlib-p3");
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let file = dir.join(format!("{name}.delulu"));
    std::fs::write(&file, src).expect("write the program");
    let path = file.to_string_lossy().to_string();
    let mut args: Vec<&str> = vec!["run", &path];
    args.extend_from_slice(grants);
    let o = delulu(&args);
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).replace("\r\n", "\n"),
    )
}

/// Check `src` and return the diagnostic codes it produced, in order.
fn codes(name: &str, src: &str) -> Vec<String> {
    let dir = std::env::temp_dir().join("delulu-stdlib-p3");
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let file = dir.join(format!("{name}.delulu"));
    std::fs::write(&file, src).expect("write the program");
    let o = delulu(&["check", &file.to_string_lossy(), "--json"]);
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("one envelope: {e}\n{}", String::from_utf8_lossy(&o.stdout)));
    v["diagnostics"]
        .as_array()
        .map(|ds| {
            ds.iter()
                .filter(|d| d["severity"] == "error")
                .map(|d| d["code"].as_str().unwrap_or("?").to_string())
                .collect()
        })
        .unwrap_or_default()
}

const PRINT: &str = "module t\n\nfn main(root: Root) -> Int ! {Write} {\n  let out = root.console()\n";

/// `chars()` is DEFINED as `split("")` (D-V2-29), so the two must agree — including on a multi-byte
/// character, where a byte-wise implementation of either one would diverge from a character-wise one.
///
/// Two spellings of one operation is two things to go stale. This is the gate that stops the second
/// from drifting from the first.
#[test]
fn chars_agrees_with_split_on_the_empty_separator() {
    let src = format!(
        "{PRINT}  let s = \"aö€b\"\n  out.println(str(s.chars()))\n  out.println(str(s.split(\"\")))\n  out.println(str(s.chars().len()))\n  0\n}}\n"
    );
    let (rc, out) = run("chars", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], lines[1], "`chars()` and `split(\"\")` must agree: {out}");
    // Four CHARACTERS, not the six bytes `ö€` occupy in UTF-8.
    assert_eq!(lines[2], "4", "chars must count characters, not bytes: {out}");
}

/// `keys()` and `values()` iterate the same map in the same order, so a caller may pair them
/// position by position. The reference states that as a promise; a `BTreeMap` is what makes it true,
/// and this is what holds the implementation to it.
///
/// The insertion order here is deliberately NOT the key order, so an implementation that returned
/// insertion order would fail rather than coincidentally pass.
#[test]
fn keys_and_values_are_in_the_same_ascending_order() {
    let src = format!(
        "{PRINT}  let m = Map()\n  m.insert(\"pear\", 20)\n  m.insert(\"apple\", 10)\n  m.insert(\"cherry\", 30)\n  out.println(str(m.keys()))\n  out.println(str(m.values()))\n  out.println(str(m))\n  0\n}}\n"
    );
    let (rc, out) = run("maporder", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "[apple, cherry, pear]", "keys must be ascending: {out}");
    assert_eq!(lines[1], "[10, 30, 20]", "values must follow the SAME order as keys: {out}");
    assert_eq!(lines[2], "{apple: 10, cherry: 30, pear: 20}", "display is that order too: {out}");
}

/// Ascending by key for `Int` keys means the INTEGER order, not the string order and not insertion
/// order — `10` sorts before `9`.
#[test]
fn int_keys_iterate_in_integer_order_not_string_order() {
    let src = format!(
        "{PRINT}  let m = Map()\n  m.insert(9, \"nine\")\n  m.insert(10, \"ten\")\n  m.insert(-1, \"neg\")\n  out.println(str(m.keys()))\n  0\n}}\n"
    );
    let (rc, out) = run("intkeys", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    assert_eq!(out.lines().next(), Some("[-1, 9, 10]"), "integer order, so 9 before 10: {out}");
}

/// `sort` is STABLE, so equal elements keep their input order and the answer is reproducible. An
/// unstable sort would make two green runs disagree for no visible reason.
///
/// Stability is unobservable on bare integers — equal integers are indistinguishable — so this sorts
/// strings whose ORDER is equal under the comparison but whose identity is visible: it sorts
/// single-character keys and checks the whole sequence, which pins the comparison itself, and then
/// checks that sorting an already-sorted list is the identity.
#[test]
fn sort_is_total_and_reproducible() {
    let src = format!(
        "{PRINT}  out.println(str([3, 1, 2].sort()))\n  out.println(str([\"b\", \"a\", \"C\"].sort()))\n  out.println(str([true, false, true].sort()))\n  out.println(str([1, 2, 3].sort()))\n  out.println(str([].sort()))\n  0\n}}\n"
    );
    let (rc, out) = run("sort", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    let l: Vec<&str> = out.lines().collect();
    assert_eq!(l[0], "[1, 2, 3]");
    // Uppercase sorts before lowercase: this is code-point order, stated rather than assumed.
    assert_eq!(l[1], "[C, a, b]", "Str sorts by code point: {out}");
    assert_eq!(l[2], "[false, true, true]", "false sorts before true: {out}");
    assert_eq!(l[3], "[1, 2, 3]", "sorting a sorted list is the identity: {out}");
    assert_eq!(l[4], "[]", "an empty list sorts to itself: {out}");
}

/// `find` SHORT-CIRCUITS, and that is observable through its callback's effects — which is exactly
/// why it is a promise the reference makes rather than an implementation detail. A `find` that
/// examined every element would print three times here instead of two.
#[test]
fn find_stops_at_the_first_match_observably() {
    let src = format!(
        "{PRINT}  let xs = [1, 2, 3]\n  let hit = xs.find(fn(x: Int) -> Bool ! {{Write}} {{ out.println(\"saw \" + str(x)); x == 2 }})\n  out.println(str(hit))\n  0\n}}\n"
    );
    let (rc, out) = run("find", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    assert_eq!(
        out, "saw 1\nsaw 2\nSome(2)\n",
        "find must stop at the first match — three `saw` lines would mean it did not: {out}"
    );
}

/// `fold`'s callback is argument **1**, and the R-4 gate reads a POSITION because of it. The row
/// must still surface into the caller: an under-declared `main` is DL0501.
///
/// This is the soundness case P3 created. Had the gate kept reading argument 0, it would have
/// inspected the accumulator, found a non-function, and decided the law about the wrong argument —
/// which for the pre-C88 code meant treating an unknown row as empty. The control below shows the
/// same program accepted once `main` declares what the callback does.
#[test]
fn folds_callback_row_reaches_the_caller() {
    let effectful = "module t\n\nfn main(root: Root) -> Int ! {Write} {\n  let out = root.console()\n  let xs = [1, 2]\n  xs.fold(0, fn(a: Int, x: Int) -> Int ! {Write} { out.println(\"in fold\"); a + x })\n}\n";
    let (rc, out) = run("fold_ok", effectful, &["--grant", "console"]);
    assert_eq!(rc, 0, "the control must RUN: {out}");
    assert_eq!(out, "in fold\nin fold\n", "the fold must actually fold: {out}");

    // The same program with `main` claiming purity. The callback writes, so the row escapes.
    let dishonest = effectful.replace("fn main(root: Root) -> Int ! {Write} {", "fn main(root: Root) -> Int {");
    let cs = codes("fold_escape", &dishonest);
    assert!(
        cs.iter().any(|c| c == "DL0501"),
        "a `fold` callback's row must reach `main` — R-4 read the wrong argument if this passes: {cs:?}"
    );
}

/// The same law for the other three, so it holds by POSITION and not by luck at position 0.
#[test]
fn every_higher_order_builtin_surfaces_its_callbacks_row() {
    for (name, call) in [
        ("map", "let _y = xs.map(fn(x: Int) -> Int ! {Write} { out.println(\"e\"); x })"),
        ("filter", "let _y = xs.filter(fn(x: Int) -> Bool ! {Write} { out.println(\"e\"); true })"),
        ("find", "let _y = xs.find(fn(x: Int) -> Bool ! {Write} { out.println(\"e\"); true })"),
        ("fold", "let _y = xs.fold(0, fn(a: Int, x: Int) -> Int ! {Write} { out.println(\"e\"); a })"),
    ] {
        let src = format!(
            "module t\n\nfn main(root: Root) -> Int {{\n  let out = root.console()\n  let xs = [1]\n  {call}\n  0\n}}\n"
        );
        let cs = codes(&format!("row_{name}"), &src);
        assert!(
            cs.iter().any(|c| c == "DL0501"),
            "`{name}` dropped its callback's row — an effect escaped the type system: {cs:?}"
        );
    }
}

/// **MAP-PARAM-1.** R-4's argument half: a higher-order builtin must agree with its callback's
/// PARAMETERS, not only with its row and its result.
///
/// Before P3 this arm read only the callback's return type, so `[1,2,3].map(fn(s: Str) -> Int {…})`
/// checked CLEAN and faulted at run time with DL0907 — a type error that escaped the checker into
/// the interpreter. `Secret[Str].map` had the identical hole, where the closure is handed the
/// PLAINTEXT under a parameter type its author declared and nobody verified.
#[test]
fn a_callbacks_parameter_type_must_match_what_the_builtin_will_pass_it() {
    let bad_list = "module t\n\nfn main() -> Int {\n  let xs = [1, 2, 3]\n  let ys = xs.map(fn(s: Str) -> Int { 7 })\n  ys.len()\n}\n";
    let cs = codes("mapparam_list", bad_list);
    assert!(
        cs.iter().any(|c| c == "DL0401"),
        "List.map must check its callback's parameter — this program used to fault at RUN time: {cs:?}"
    );

    let bad_secret = "module t\n\nfn main(root: Root) -> Int ! {Read} {\n  let k = root.secret(\"K\")\n  let m = k.map(fn(x: Int) -> Str { \"z\" })\n  1\n}\n";
    let cs = codes("mapparam_secret", bad_secret);
    assert!(
        cs.iter().any(|c| c == "DL0401"),
        "Secret.map must check it too — a rule that holds on one path out of two holds nowhere: {cs:?}"
    );

    // The controls: the same two programs with the parameter their builtin will actually pass.
    let ok_list = bad_list.replace("fn(s: Str) -> Int { 7 }", "fn(s: Int) -> Int { 7 }");
    assert!(codes("mapparam_list_ok", &ok_list).is_empty(), "the control must check clean");
    let ok_secret = bad_secret.replace("fn(x: Int) -> Str", "fn(x: Str) -> Str");
    assert!(codes("mapparam_secret_ok", &ok_secret).is_empty(), "the control must check clean");
}

/// ONE diagnostic for one mistake. A non-function argument to a higher-order builtin is refused by
/// the R-4 gate, which explains WHY the row must be known; the `method_sig` arms used to add a
/// second DL0401 saying less. Two diagnostics for one mistake is how a reader learns to stop reading
/// them.
#[test]
fn exactly_one_diagnostic_for_a_non_function_callback() {
    for (name, call) in [
        ("map", "xs.map(42)"),
        ("filter", "xs.filter(42)"),
        ("find", "xs.find(42)"),
        ("fold", "xs.fold(0, 42)"),
    ] {
        let src = format!("module t\n\nfn main() -> Int {{\n  let xs = [1]\n  let _ = {call}\n  0\n}}\n");
        let cs = codes(&format!("one_{name}"), &src);
        assert_eq!(cs.len(), 1, "`{name}` must report ONE diagnostic, got {cs:?}");
        assert_eq!(cs[0], "DL0401", "and it must be the R-4 one: {cs:?}");
    }
}

/// `contains` is `==` in a loop, so it answers to the same rule and emits the SAME code (DL0605).
///
/// The alternative is what makes this worth a test: `Value::eq` returns `false` for two `Secret`s,
/// so an admitted `xs.contains(k)` over secrets would have returned a confident, wrong `false` — an
/// equality answer derived from secret data, which is the shape of finding IF-1.
#[test]
fn contains_refuses_an_opaque_element_with_the_same_code_as_equality() {
    let src = "module t\n\nfn main(root: Root) -> Int ! {Read} {\n  let k = root.secret(\"K\")\n  let xs = [k]\n  let _ = xs.contains(k)\n  0\n}\n";
    let cs = codes("contains_secret", src);
    assert!(cs.iter().any(|c| c == "DL0605"), "expected the equality code: {cs:?}");

    // `==` on the same values, for the comparison that makes "the same code" meaningful.
    let eq = "module t\n\nfn main(root: Root) -> Int ! {Read} {\n  let k = root.secret(\"K\")\n  if k == k { 1 } else { 0 }\n}\n";
    let cs_eq = codes("eq_secret", eq);
    assert!(cs_eq.iter().any(|c| c == "DL0605"), "`==` must emit it too, or they have diverged: {cs_eq:?}");

    // The control: a list of plain values.
    let ok = "module t\n\nfn main() -> Int {\n  let xs = [1, 2]\n  if xs.contains(2) { 1 } else { 0 }\n}\n";
    assert!(codes("contains_ok", ok).is_empty(), "the control must check clean");
}

/// `Map[K, Secret[V]]` must be opaque, or a map of secrets compares and stringifies structurally.
/// The recursion was added by PATTERN — beside `Result`'s — rather than as an instance, because the
/// `_` arm of `is_opaque` answers `false` and a new composite type that forgets this is silently
/// comparable.
#[test]
fn a_map_containing_a_secret_is_opaque() {
    let cmp = "module t\n\nfn main(root: Root) -> Int ! {Read} {\n  let m = Map()\n  m.insert(\"k\", root.secret(\"K\"))\n  let n = Map()\n  if m == n { 1 } else { 0 }\n}\n";
    assert!(codes("map_eq", cmp).iter().any(|c| c == "DL0605"), "a map of secrets must not compare");

    let show = "module t\n\nfn main(root: Root) -> Int ! {Read} {\n  let m = Map()\n  m.insert(\"k\", root.secret(\"K\"))\n  let _ = str(m)\n  0\n}\n";
    assert!(codes("map_str", show).iter().any(|c| c == "DL0604"), "a map of secrets must not stringify");

    // The control: a map of plain values compares and stringifies.
    let ok = "module t\n\nfn main() -> Int {\n  let m = Map()\n  m.insert(\"k\", 1)\n  let _ = str(m)\n  0\n}\n";
    assert!(codes("map_str_ok", ok).is_empty(), "the control must check clean");
}

/// `Map` is a prelude TYPE name, so `type Map = Int` is REFUSED rather than accepted-and-ignored.
///
/// This is campaign finding C23's rule: a `type Int = Secret[Str]` declaration was once accepted and
/// then had no effect whatsoever, because the builtin name is matched before user scope is searched.
/// For a language whose premise is that authority can be read off the source, a declaration that
/// silently means nothing misleads review without ever failing.
#[test]
fn map_cannot_be_shadowed_by_a_user_type() {
    let cs = codes("map_shadow", "module t\ntype Map = Int\nfn main() -> Int { 0 }\n");
    assert!(cs.iter().any(|c| c == "DL0302"), "redefining `Map` must be refused: {cs:?}");
}

/// The sort refusals, and the three element types that are admitted. `Float` is refused BY NAME —
/// the diagnostic has to say WHY, because "unsupported" would send an author looking for a flag.
#[test]
fn sort_refuses_the_element_types_that_have_no_order() {
    let float = "module t\n\nfn main() -> Int {\n  let xs = [1.5, 0.5]\n  let _ = xs.sort()\n  0\n}\n";
    let cs = codes("sort_float", float);
    assert!(cs.iter().any(|c| c == "DL0401"), "List[Float].sort must be refused: {cs:?}");

    let nested = "module t\n\nfn main() -> Int {\n  let xs = [[1], [2]]\n  let _ = xs.sort()\n  0\n}\n";
    assert!(codes("sort_nested", nested).iter().any(|c| c == "DL0401"), "a list of lists has no order");

    for (name, lit) in [("int", "[1]"), ("str", "[\"a\"]"), ("bool", "[true]")] {
        let src = format!("module t\n\nfn main() -> Int {{\n  let xs = {lit}\n  let _ = xs.sort()\n  0\n}}\n");
        assert!(codes(&format!("sort_ok_{name}"), &src).is_empty(), "`{name}` must sort");
    }
}

/// The Map key discipline, on EVERY method that takes a key. A rule that holds on three paths out of
/// four holds nowhere — and the first implementation of this held on zero, because it ran before the
/// argument had unified `K` and so only ever saw an inference variable.
#[test]
fn a_bad_map_key_is_refused_on_every_method_that_takes_one() {
    for (name, call) in [
        ("insert", "m.insert(1.5, \"x\")"),
        ("get", "let _ = m.get(1.5)"),
        ("remove", "let _ = m.remove(1.5)"),
        ("contains_key", "let _ = m.contains_key(1.5)"),
    ] {
        let src = format!("module t\n\nfn main() -> Int {{\n  let m = Map()\n  {call}\n  0\n}}\n");
        let cs = codes(&format!("key_{name}"), &src);
        assert!(
            cs.iter().any(|c| c == "DL0401"),
            "`Map.{name}` admitted a Float key — the interpreter would fault blaming a checker bug: {cs:?}"
        );
    }

    // A structural key, which is refused for a different reason (no canonical form).
    let structural = "module t\n\nfn main() -> Int {\n  let m = Map()\n  m.insert([1], \"x\")\n  0\n}\n";
    assert!(codes("key_struct", structural).iter().any(|c| c == "DL0401"), "a structural key must be refused");

    // The three controls.
    for (name, k) in [("str", "\"a\""), ("int", "7"), ("bool", "true")] {
        let src = format!("module t\n\nfn main() -> Int {{\n  let m = Map()\n  m.insert({k}, 1)\n  0\n}}\n");
        assert!(codes(&format!("key_ok_{name}"), &src).is_empty(), "a `{name}` key must be admitted");
    }
}

/// Every MUTATING method is refused through a `val` reference, and every non-mutating one is not.
///
/// The negative half is the one that matters: if `keys` or `get` also demanded a writable receiver,
/// the rule would look enforced while actually refusing reads, and the registry would be wrong in a
/// direction no refusal test can see.
#[test]
fn only_the_mutating_methods_demand_a_writable_receiver() {
    let list_cases: &[(&str, &str, bool)] = &[
        ("push", "xs.push(1)", true),
        ("pop", "let _ = xs.pop()", true),
        ("len", "let _ = xs.len()", false),
        ("reverse", "let _ = xs.reverse()", false),
        ("contains", "let _ = xs.contains(1)", false),
    ];
    for (name, call, mutates) in list_cases {
        let src = format!("module t\n\nfn f(xs: val List[Int]) -> Int {{\n  {call}\n  0\n}}\n\nfn main() -> Int {{ 0 }}\n");
        let cs = codes(&format!("val_list_{name}"), &src);
        let refused = cs.iter().any(|c| c == "DL1604");
        assert_eq!(
            refused, *mutates,
            "`List.{name}` mutates={mutates} but a `val` receiver {} it: {cs:?}",
            if refused { "refused" } else { "allowed" }
        );
    }

    let map_cases: &[(&str, &str, bool)] = &[
        ("insert", "m.insert(\"a\", 1)", true),
        ("remove", "let _ = m.remove(\"a\")", true),
        ("get", "let _ = m.get(\"a\")", false),
        ("keys", "let _ = m.keys()", false),
        ("values", "let _ = m.values()", false),
        ("len", "let _ = m.len()", false),
    ];
    for (name, call, mutates) in map_cases {
        let src = format!("module t\n\nfn f(m: val Map[Str, Int]) -> Int {{\n  {call}\n  0\n}}\n\nfn main() -> Int {{ 0 }}\n");
        let cs = codes(&format!("val_map_{name}"), &src);
        let refused = cs.iter().any(|c| c == "DL1604");
        assert_eq!(
            refused, *mutates,
            "`Map.{name}` mutates={mutates} but a `val` receiver {} it: {cs:?}",
            if refused { "refused" } else { "allowed" }
        );
    }
}

/// `replace` with an EMPTY pattern returns the receiver unchanged (D-V2-29 decision 6), rather than
/// inserting between every character the way Rust's `str::replace` would. Unlike `split("")`, which
/// has a natural reading, an empty replacement pattern has none — so this is a definition, and a
/// definition nobody wrote down is a surprise waiting for a reader.
#[test]
fn replace_with_an_empty_pattern_is_the_identity() {
    let src = format!(
        "{PRINT}  out.println(\"abc\".replace(\"\", \"X\"))\n  out.println(\"abc\".replace(\"b\", \"X\"))\n  out.println(\"abc\".replace(\"z\", \"X\"))\n  0\n}}\n"
    );
    let (rc, out) = run("replace", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    assert_eq!(out, "abc\naXc\nabc\n", "{out}");
}

/// Case mapping is full Unicode, and the reference must SAY it is not a security normalization.
///
/// The behavioural half is `ß` → `SS`: a case map that does not round-trip. The documentation half
/// is the sentence, and it is asserted because four of the P22 campaign's defects were security
/// decisions taken on a differently-spelled string, and the paragraph admitting that is the first
/// thing to be cut when a document is edited for length.
#[test]
fn case_mapping_is_unicode_and_documented_as_not_a_normalization() {
    let src = format!("{PRINT}  out.println(\"straße\".to_upper())\n  out.println(\"İ\".to_lower())\n  0\n}}\n");
    let (rc, out) = run("case", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    assert_eq!(out.lines().next(), Some("STRASSE"), "full Unicode case mapping: {out}");

    let doc = std::fs::read_to_string(root().join("docs").join("for-agents.md")).expect("for-agents.md");
    let flat: String = doc.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains("not a security normalization"),
        "the reference must warn that case folding is not a security normalization"
    );
}

/// `slice` clamps rather than faulting, and `List.slice` clamps the SAME way `Str.slice` does — an
/// out-of-range window is empty, and `hi < lo` is empty rather than reversed. Two clamping rules that
/// disagreed would be a trap for anyone who learned one of them.
#[test]
fn list_and_str_slice_clamp_identically() {
    let src = format!(
        "{PRINT}  out.println(str([1, 2, 3].slice(1, 2)))\n  out.println(str([1, 2, 3].slice(2, 1)))\n  out.println(str([1, 2, 3].slice(0, 99)))\n  out.println(str([1, 2, 3].slice(99, 99)))\n  out.println(\"[\" + \"abc\".slice(1, 2) + \"]\")\n  out.println(\"[\" + \"abc\".slice(2, 1) + \"]\")\n  out.println(\"[\" + \"abc\".slice(0, 99) + \"]\")\n  out.println(\"[\" + \"abc\".slice(99, 99) + \"]\")\n  0\n}}\n"
    );
    let (rc, out) = run("slice", &src, &["--grant", "console"]);
    assert_eq!(rc, 0, "{out}");
    let l: Vec<&str> = out.lines().collect();
    assert_eq!(l[0], "[2]");
    assert_eq!(l[1], "[]", "hi < lo is empty, not reversed: {out}");
    assert_eq!(l[2], "[1, 2, 3]", "hi past the end clamps: {out}");
    assert_eq!(l[3], "[]", "lo past the end is empty: {out}");
    assert_eq!(l[4], "[b]");
    assert_eq!(l[5], "[]", "Str.slice must clamp the same way: {out}");
    assert_eq!(l[6], "[abc]");
    assert_eq!(l[7], "[]");
}

/// A callback that MUTATES the list it is iterating sees the list as it was when the call began.
///
/// The three new higher-order methods snapshot their receiver, exactly as `map` already did, and the
/// reason is not tidiness: iterating a `RefCell` while a callback mutates it is either a borrow panic
/// or a loop whose length changes underneath it. "The list as it was when the call began" is a
/// definition a reader can rely on; "whatever the borrow checker permitted that day" is not.
#[test]
fn a_callback_that_mutates_its_receiver_does_not_panic() {
    for (name, call) in [
        ("map", "let _y = xs.map(fn(x: Int) -> Int { push(xs, x)  x })"),
        ("filter", "let _y = xs.filter(fn(x: Int) -> Bool { push(xs, x)  true })"),
        ("find", "let _y = xs.find(fn(x: Int) -> Bool { push(xs, x)  false })"),
        ("fold", "let _y = xs.fold(0, fn(a: Int, x: Int) -> Int { push(xs, x)  a })"),
    ] {
        let src = format!("module t\n\nfn main() -> Int {{\n  let xs = [1, 2]\n  {call}\n  xs.len()\n}}\n");
        let (rc, out) = run(&format!("reentrant_{name}"), &src, &[]);
        assert!(
            rc == 0 || rc == 1,
            "`{name}` with a mutating callback must not crash the interpreter (rc={rc}): {out}"
        );
        assert!(!out.contains("panicked"), "`{name}`: {out}");
    }
}
