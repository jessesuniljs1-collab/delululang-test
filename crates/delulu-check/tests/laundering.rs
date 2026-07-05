//! The laundering suite: the exploits from `docs/design/SOUNDNESS_AUDIT.md` encoded as permanent
//! rejection tests. The single question: can authority escape the type? For every Stage-1
//! applicable audit finding there is a program here that MUST be rejected (or, for F-2, forced to
//! declare the effect). F-1, F-6, and R-7 are plugin/broker scenarios that arrive with the
//! runtime plugin loader (Stage 6) and are tracked there.

use delulu_check::check_source;

fn codes(src: &str) -> Vec<String> {
    check_source(0, src)
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.to_string())
        .collect()
}

fn rejects_with(src: &str, code: &str) {
    let cs = codes(src);
    assert!(cs.iter().any(|c| c == code), "expected {code}, got {cs:?}\nsource:\n{src}");
}

fn accepts(src: &str) {
    let cs = codes(src);
    assert!(cs.is_empty(), "expected clean, got {cs:?}\nsource:\n{src}");
}

// ---- F-2: declassification must be visible in the row (Declassify is an effect) -------------

#[test]
fn f2_expose_without_declassify_row_is_rejected() {
    // A function that reveals a secret must declare `!{Declassify}`; omitting it is DL0501.
    rejects_with(
        "module m\nfn leak(d: Cap[Declassify], s: Secret[Str]) -> Str { s.expose(d) }\n",
        "DL0501",
    );
}

#[test]
fn f2_expose_with_declassify_row_is_accepted() {
    accepts("module m\nfn ok(d: Cap[Declassify], s: Secret[Str]) -> Str ! {Declassify} { s.expose(d) }\n");
}

// ---- F-3: no subsumption inside unification (contravariant row exploit) ----------------------

#[test]
fn f3_contravariant_row_ascription_is_rejected() {
    // `twice` is honest given a pure callback; ascribing it a type whose parameter row is {Write}
    // must fail — otherwise an effectful callback laundes into a function that believes it pure.
    rejects_with(
        "module m\n\
         fn twice(cb: fn() -> Unit ! {}) -> Unit { cb() cb() }\n\
         fn bad() -> Unit { let h: fn(fn() -> Unit ! {Write}) -> Unit ! {} = twice\n }\n",
        "DL0401",
    );
}

// ---- F-3b: conflicting row-variable bindings are a conflict, never a union -------------------

#[test]
fn f3b_conflicting_row_variable_bindings_are_dl0504() {
    // A single row variable shared by two parameters, fed a pure and an effectful callback,
    // must conflict (DL0504) rather than silently widening to {Write}.
    rejects_with(
        "module m\n\
         fn both[e](f: fn() -> Unit ! e, g: fn() -> Unit ! e) -> Unit ! e { f() g() }\n\
         fn use_it(out: Cap[Console]) ! {Write} {\n\
         \x20 both(fn() -> Unit { }, fn() -> Unit ! {Write} { out.println(\"x\") })\n }\n",
        "DL0504",
    );
}

// ---- F-4: higher-order builtins surface the callback's row -----------------------------------

#[test]
fn f4_effectful_lambda_into_map_in_pure_fn_is_rejected() {
    // `[1,2,3].map(effectful)` inside a function that declares no row must be DL0501 — the
    // callback's Write cannot vanish through `map`.
    rejects_with(
        "module m\n\
         fn go(out: Cap[Console]) {\n\
         \x20 let xs = [1, 2, 3]\n\
         \x20 let ys = xs.map(fn(x: Int) -> Int ! {Write} { out.println(str(x)) x })\n }\n",
        "DL0501",
    );
}

#[test]
fn f4_pure_lambda_into_map_stays_pure() {
    accepts(
        "module m\nfn go() -> List[Int] { let xs = [1, 2, 3]\n xs.map(fn(x: Int) -> Int { x * 2 }) }\n",
    );
}

// ---- F-5: opaque types have no str / == / serialization -------------------------------------

#[test]
fn f5_stringify_secret_is_dl0604() {
    rejects_with("module m\nfn f(s: Secret[Str]) -> Str { str(s) }\n", "DL0604");
}

#[test]
fn f5_stringify_capability_is_dl0604() {
    rejects_with("module m\nfn f(c: Cap[Console]) -> Str { str(c) }\n", "DL0604");
}

#[test]
fn f5_equality_on_secret_is_dl0605() {
    rejects_with("module m\nfn f(a: Secret[Str], b: Secret[Str]) -> Bool { a == b }\n", "DL0605");
}

#[test]
fn f5_record_containing_a_cap_is_opaque() {
    // A record that structurally contains a capability is itself opaque (propagated).
    rejects_with(
        "module m\ntype Holder { c: Cap[Console] }\nfn f(a: Holder, b: Holder) -> Bool { a == b }\n",
        "DL0605",
    );
}

// ---- The core non-escape guarantee: no effect without a capability ---------------------------

#[test]
fn no_effect_without_a_capability_is_structural() {
    // There is no way to perform Write without holding a Cap[Console] threaded from Root:
    // a function with an empty environment simply cannot name `println`.
    rejects_with("module m\nfn f() { println(\"hi\") }\n", "DL0301");
}

#[test]
fn secret_cannot_flow_into_a_sink() {
    // Secret[Str] is not Str: passing it where Str is expected is the secret-non-flow error.
    rejects_with(
        "module m\nfn f(out: Cap[Console], s: Secret[Str]) ! {Write} { out.println(s) }\n",
        "DL0602",
    );
}
