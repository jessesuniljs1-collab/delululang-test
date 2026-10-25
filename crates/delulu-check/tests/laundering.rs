//! The laundering suite: the exploits from `docs/design/SOUNDNESS_AUDIT.md` encoded as permanent
//! rejection tests. The single question: can authority escape the type? For every Stage-1
//! applicable audit finding there is a program here that MUST be rejected (or, for F-2, forced to
//! declare the effect). F-1, F-6, and R-7 are plugin/broker scenarios that arrive with the
//! runtime plugin loader (Stage 6) and are tracked there.

use delulu_check::check_source;

fn codes(src: &str) -> Vec<String> {
    check_source(0, src).diagnostics.iter().filter(|d| d.is_error()).map(|d| d.code.to_string()).collect()
}

fn rejects_with(src: &str, code: &str) {
    let cs = codes(src);
    // A witness must parse (2026-10-24, D-V2-184): F-3, F-3b and F-4 wrote two statements on one line (`cb() cb()`) — DL0209 — for as long as
    // they existed, and passed through the parser's recovery. A program the parser had to repair is not the program
    // the test names; its refusal may come from the repair.
    assert!(
        !cs.iter().any(|c| c.starts_with("DL01") || c.starts_with("DL02")),
        "the witness does not lex or parse cleanly: {cs:?}\nsource:\n{src}"
    );
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
    rejects_with("module m\nfn leak(d: Cap[Declassify], s: Secret[Str]) -> Str { s.expose(d) }\n", "DL0501");
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
         fn twice(cb: fn() -> Unit ! {}) -> Unit { cb(); cb() }\n\
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
         fn both[e](f: fn() -> Unit ! e, g: fn() -> Unit ! e) -> Unit ! e { f(); g() }\n\
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
         \x20 let ys = xs.map(fn(x: Int) -> Int ! {Write} { out.println(str(x)); x })\n }\n",
        "DL0501",
    );
}

#[test]
fn f4_pure_lambda_into_map_stays_pure() {
    accepts("module m\nfn go() -> List[Int] { let xs = [1, 2, 3]\n xs.map(fn(x: Int) -> Int { x * 2 }) }\n");
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
    rejects_with("module m\ntype Holder { c: Cap[Console] }\nfn f(a: Holder, b: Holder) -> Bool { a == b }\n", "DL0605");
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
    rejects_with("module m\nfn f(out: Cap[Console], s: Secret[Str]) ! {Write} { out.println(s) }\n", "DL0602");
}

// ---- GENERIC-RIGID-1 (2026-10-24, D-V2-184): a generic's own parameters are rigid in its body -----------
//
// Found by the agent test campaign (agent C, Haiku 5.5): a generic function's type and row parameters were
// flexible inference variables inside its own body, so the body could decide them (`T := Int`, `e := {}`) and a
// caller picked them again. `coerce[A, B](x: A) -> B { x }` turned an effectful closure into a pure one: the
// program printed while `check` said `main` never performed `Write`, and `forge[T](x: Int) -> T` made a
// capability of an `Int` (caught at run time only, DL0907).

#[test]
fn generic_rigid_a_body_cannot_decide_its_type_parameter() {
    rejects_with("module m\nfn forge[T](x: Int) -> T { x }\n", "DL0401");
    rejects_with("module m\nfn coerce[A, B](x: A) -> B { x }\n", "DL0401");
    rejects_with("module m\nfn g[T](x: T) -> Int { x }\n", "DL0401");
}

#[test]
fn generic_rigid_an_effectful_closure_cannot_be_coerced_pure() {
    rejects_with(
        "module m\n\
         fn coerce[A, B](x: A) -> B { x }\n\
         fn writer(out: Cap[Console]) -> fn(Str) -> Unit ! {Write} { fn(s: Str) -> Unit ! {Write} { out.println(s) } }\n\
         fn pure_caller(out: Cap[Console]) -> Unit { let f: fn(Str) -> Unit = coerce(writer(out))\n f(\"x\") }\n",
        "DL0401",
    );
}

#[test]
fn generic_rigid_a_body_cannot_close_its_row_parameter() {
    rejects_with(
        "module m\n\
         fn hide[e](f: fn() -> Unit ! e) -> fn() -> Unit { f }\n\
         fn writer(out: Cap[Console]) -> fn() -> Unit ! {Write} { fn() -> Unit ! {Write} { out.println(\"x\") } }\n\
         fn pure_caller(out: Cap[Console]) -> Unit { let f = hide(writer(out))\n f() }\n",
        "DL0401",
    );
}

#[test]
fn generic_rigid_an_actor_body_cannot_decide_its_parameter() {
    rejects_with(
        "module m\n\
         actor Box[T] {\n\
         \x20 var v: Int\n\
         \x20 new() { self.v = 0 }\n\
         \x20 be put(x: val T) { self.v = x }\n\
         }\n",
        "DL0401",
    );
}

#[test]
fn generic_rigid_honest_generics_still_check() {
    accepts(
        "module m\n\
         fn id[T](x: T) -> T { x }\n\
         fn pick[T](c: Bool, a: T, b: T) -> T { if c { a } else { b } }\n\
         fn apply[e](f: fn() -> Unit ! e) -> Unit ! e { f() }\n\
         fn twice[e](f: fn() -> Unit ! e) -> Unit ! e { apply(f); apply(f) }\n\
         fn widen[e](f: fn() -> Unit ! e) -> fn() -> Unit ! e { f }\n\
         fn use_them(out: Cap[Console]) ! {Write} {\n\
         \x20 let n = pick(true, id(1), 2)\n\
         \x20 let s = id(\"x\")\n\
         \x20 twice(fn() -> Unit ! {Write} { out.println(s + str(n)) })\n\
         \x20 let g = widen(fn() -> Unit ! {Write} { out.println(s) })\n\
         \x20 g()\n\
         }\n",
    );
}
