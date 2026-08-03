//! P17-IF1 — the declassification oracle: `Secret.map` + `Secret.verify` compose into total
//! plaintext recovery with **no `Declassify` effect and no `Cap[Declassify]` anywhere.**
//!
//! This is a companion to `laundering.rs`, kept separate because it does not encode a finding from
//! `SOUNDNESS_AUDIT.md` — it encodes a finding **against** it. Audit rules R-2
//! (Declassify-is-an-effect) and R-5 (opacity) both assert this cannot happen.
//!
//! ## The mechanism
//!
//! Neither operation is wrong in isolation, which is exactly why the audit missed it:
//!
//!   1. `Secret.map` hands its closure the **PLAINTEXT**. The only gate is purity (DL0603) —
//!      and purity is not confidentiality. A pure closure can compute any predicate over the
//!      plaintext and encode the answer in the returned `Secret[Str]` (say `"Y"` / `"N"`).
//!   2. `Secret.verify` compares two secrets and returns an **ordinary, untainted `Bool`** with
//!      effect `None`. That is the unsealing step: a value derived from secret data leaves the
//!      `Secret` lattice with no declassification recorded.
//!   3. `check_if` carries no pc-label (no implicit-flow tracking), so the `Bool` may drive an
//!      observable effect.
//!
//! Composed, they are an equality oracle against an ATTACKER-CHOSEN plaintext, which amplifies to
//! full recovery one character at a time.
//!
//! ## Observed, not argued (2026-08-03, delulu.exe debug build)
//!
//! ```text
//! $ delulu check    extract.delulu   -> ok: extract.delulu checked clean
//! $ delulu authority extract.delulu  -> effects: Write        (no Declassify)
//! $ delulu why Declassify extract.delulu -> program cannot perform `Declassify`
//! $ delulu run extract.delulu --grant console --grant secret:API_KEY=hunter2
//!       recovered length = 7
//!       RECOVERED SECRET = hunter2
//! $ ... --grant secret:API_KEY=sk-9f0zq_x  -> RECOVERED SECRET = sk-9f0zq_x
//! $ ... --assert-trace                     -> exit 0, no complaint
//! ```
//!
//! The toolchain states the program **cannot** declassify. The program prints the secret.
//!
//! ## Status
//!
//! **OPEN.** These tests are `#[ignore]`d because they currently FAIL — they are committed as
//! evidence, not as a passing gate. Run them with:
//!
//! ```text
//! cargo test -p delulu-check --test secret_oracle -- --ignored
//! ```
//!
//! The fix is a language-semantics change (`Secret.verify` must not yield an untainted `Bool`) and
//! therefore touches the `STABILITY.md` contract. See `docs/design/PROOF_CAMPAIGN.md` §IF-1.
//!
//! The DIRECT surface is NOT implicated and was verified clean: printing, concatenating,
//! `str()`, `==`, `assert_eq`, writing to a file, and putting a secret in a record are all
//! correctly refused (DL0602 / DL0604 / DL0605 / DL0203). R-5 opacity holds for every direct
//! eliminator. The hole is composition, not opacity.

use delulu_check::check_source;

fn error_codes(src: &str) -> Vec<String> {
    check_source(0, src)
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.to_string())
        .collect()
}

/// One bit of the plaintext, extracted with no `Declassify` anywhere.
const ONE_BIT_ORACLE: &str = "\
module m

fn probe(k: Secret[Str], i: Int, c: Str) -> Bool {
  let marked = k.map(fn(x: Str) -> Str { if x.slice(i, i + 1) == c { \"Y\" } else { \"N\" } })
  let yes = k.map(fn(x: Str) -> Str { \"Y\" })
  marked.verify(yes)
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let k = root.secret(\"API_KEY\")
  if probe(k, 0, \"h\") { out.println(\"char 0 == h\") } else { out.println(\"char 0 != h\") }
}
";

/// An equality oracle against an attacker-chosen plaintext — `verify` was documented as a
/// constant-time comparison of two secrets, but a constant `map` supplies the second one.
const EQUALITY_ORACLE: &str = "\
module m

fn guess(k: Secret[Str], g: Str) -> Bool {
  k.verify(k.map(fn(x: Str) -> Str { g }))
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let k = root.secret(\"PIN\")
  if guess(k, \"4242\") { out.println(\"PIN IS 4242\") } else { out.println(\"no\") }
}
";

#[test]
#[ignore = "P17-IF1 OPEN: extracts a secret with no Declassify. Fix changes STABILITY contract; \
            see docs/design/PROOF_CAMPAIGN.md §IF-1."]
fn a_pure_predicate_over_the_plaintext_cannot_reach_an_untainted_bool() {
    let cs = error_codes(ONE_BIT_ORACLE);
    assert!(
        !cs.is_empty(),
        "SECRET ORACLE ACCEPTED: a pure closure computed a predicate over the PLAINTEXT and \
         `verify` returned it as an ordinary Bool, which then drove an observable effect — with no \
         `Declassify` effect and no `Cap[Declassify]` in the program. R-2 and R-5 both say this \
         cannot happen.\nsource:\n{ONE_BIT_ORACLE}"
    );
}

#[test]
#[ignore = "P17-IF1 OPEN: equality oracle against attacker-chosen plaintext. \
            See docs/design/PROOF_CAMPAIGN.md §IF-1."]
fn verify_cannot_be_turned_into_an_oracle_against_a_chosen_plaintext() {
    let cs = error_codes(EQUALITY_ORACLE);
    assert!(
        !cs.is_empty(),
        "EQUALITY ORACLE ACCEPTED: `k.verify(k.map(|_| g))` compares the secret against an \
         arbitrary attacker-supplied string `g` and yields an untainted Bool. `verify` is \
         documented as a constant-time comparison of two secrets; a constant `map` supplies the \
         second one.\nsource:\n{EQUALITY_ORACLE}"
    );
}

// ---- Controls: these MUST keep passing, so a future fix cannot be a blanket refusal ----------

/// The legitimate declassification path still works and still declares its effect.
#[test]
fn the_honest_declassification_path_is_still_accepted() {
    let cs = error_codes(
        "module m\nfn ok(d: Cap[Declassify], s: Secret[Str]) -> Str ! {Declassify} { s.expose(d) }\n",
    );
    assert!(cs.is_empty(), "the honest path must stay open, got {cs:?}");
}

/// `Secret.map` itself is not the defect and must not be broken by a fix: mapping a secret to a
/// secret, with the result kept sealed, is the whole point of the operation.
#[test]
fn mapping_a_secret_to_a_secret_is_still_accepted() {
    let cs = error_codes(
        "module m\nfn ok(s: Secret[Str]) -> Secret[Str] { s.map(fn(x: Str) -> Str { x.trim() }) }\n",
    );
    assert!(cs.is_empty(), "Secret.map must stay usable, got {cs:?}");
}

/// An impure mapper is already refused (DL0603) and must stay refused.
#[test]
fn an_impure_mapper_is_still_refused() {
    let cs = error_codes(
        "module m\nfn bad(o: Cap[Console], s: Secret[Str]) -> Secret[Str] ! {Write} \
         { s.map(fn(x: Str) -> Str ! {Write} { o.println(x)  x }) }\n",
    );
    assert!(cs.iter().any(|c| c == "DL0603"), "expected DL0603, got {cs:?}");
}
