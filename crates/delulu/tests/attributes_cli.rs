//! Stage 10 phase 10a — attribute activation (Track A2, spec §2.2).
//!
//! The single law every execution mode answers to is invariant 45: a hint never changes what a
//! program does, means, or may do. These tests make that law mechanical at the CLI boundary —
//! the twin test compares the attributed program against its attribute-free twin on every
//! channel a user or agent can observe.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-attrs-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

const BODY: &str = "fn hot(x: Int) -> Int {\n  x * 2\n}\n\nfn main(root: Root) ! {Write} {\n  let c = root.console()\n  c.println(str(hot(21)))\n}\n";

/// INVARIANT 45, the whole point of the phase: the attributed program and its attribute-free
/// twin are identical on every observable channel — run output, exit code, and the authority
/// report byte-for-byte. A hint that changes any of these is not a hint and does not ship.
#[test]
fn invariant45_hints_change_nothing_observable() {
    let dir = scratch("twin");
    let plain = dir.join("plain.delulu");
    let hinted = dir.join("hinted.delulu");
    std::fs::write(&plain, format!("module twin\n\n{BODY}")).unwrap();
    std::fs::write(
        &hinted,
        format!("@interpret\nmodule twin\n\n@jit\n{}", BODY.replacen("fn main", "@aot\nfn main", 1)),
    )
    .unwrap();

    let run_a = delulu(&["run", &plain.to_string_lossy(), "--grant", "console"]);
    let run_b = delulu(&["run", &hinted.to_string_lossy(), "--grant", "console"]);
    assert!(run_a.status.success() && run_b.status.success(), "both twins run clean");
    assert_eq!(run_a.stdout, run_b.stdout, "run output is identical with and without hints");

    let mut va: serde_json::Value =
        serde_json::from_slice(&delulu(&["authority", &plain.to_string_lossy(), "--json"]).stdout)
            .expect("plain authority json");
    let mut vb: serde_json::Value =
        serde_json::from_slice(&delulu(&["authority", &hinted.to_string_lossy(), "--json"]).stdout)
            .expect("hinted authority json");
    // Build-order D7: invariant 45 binds SEMANTICS and authority FACTS — effects, capabilities,
    // secrets, scopes. The `native_emission` stamp is not a fact about what the program may do;
    // it is the `@jit` REQUEST made reviewable, and spec §2.3 orders it onto the report. So the
    // strong form of this test: the stamp is the ONLY difference between the twins.
    let stamp = vb["authority"]
        .as_object_mut()
        .expect("authority object")
        .remove("native_emission")
        .expect("the hinted twin carries the request stamp");
    assert_eq!(stamp["requested"], true);
    assert!(
        va["authority"].as_object_mut().unwrap().remove("native_emission").is_none(),
        "the plain twin carries no stamp"
    );
    assert_eq!(
        va, vb,
        "with the request stamp removed, the reports are IDENTICAL — a hint can never widen, \
         narrow, or reshape what a program may do"
    );
}

/// DL1901 carries an EXACT removal repair that is not authority-widening: removing a hint never
/// changes behavior, so the machine may apply it. The skip branch (an unknown attribute NOT
/// being diagnosed) is exactly what the reject fixture + this shape test forbid.
#[test]
fn unknown_attribute_is_dl1901_with_an_exact_removal_repair() {
    let dir = scratch("unknown");
    let f = dir.join("bad.delulu");
    std::fs::write(&f, "module m\n\n@fastpath\nfn f(x: Int) -> Int {\n  x\n}\n").unwrap();
    let o = delulu(&["check", &f.to_string_lossy(), "--json"]);
    assert!(!o.status.success(), "an unknown attribute fails the check");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("json envelope");
    let d = &v["diagnostics"][0];
    assert_eq!(d["code"], "DL1901");
    let r = &d["repairs"][0];
    assert_eq!(r["confidence"], "exact");
    assert_eq!(r["authority_widening"], false, "removing a hint never widens authority");
    let edit = &r["edits"][0];
    assert_eq!(edit["insert"], "", "the repair is a pure removal");
}

/// Known name, wrong shape: `@inline` demands its argument, and anything but
/// `"never"`/`"always"` is refused under the same code — no lenient parse grows a dialect.
#[test]
fn a_known_attribute_with_the_wrong_shape_is_dl1901() {
    let dir = scratch("shape");
    for (tag, src) in [
        ("bare-inline", "module m\n\n@inline\nfn f(x: Int) -> Int {\n  x\n}\n"),
        ("bad-arg", "module m\n\n@inline(\"sometimes\")\nfn f(x: Int) -> Int {\n  x\n}\n"),
        ("arg-on-jit", "module m\n\n@jit(\"hot\")\nfn f(x: Int) -> Int {\n  x\n}\n"),
    ] {
        let f = dir.join(format!("{tag}.delulu"));
        std::fs::write(&f, src).unwrap();
        let o = delulu(&["check", &f.to_string_lossy(), "--json"]);
        assert!(!o.status.success(), "{tag} must be refused");
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        let codes: Vec<&str> =
            v["diagnostics"].as_array().unwrap().iter().filter_map(|d| d["code"].as_str()).collect();
        assert!(codes.contains(&"DL1901"), "{tag}: expected DL1901, got {codes:?}");
    }
}

/// Attributes sit on `fn`, `actor`, and the module header — nowhere else. An attribute on a
/// `type` or a constant is DL1901, not a silent shrug (the no-vendor-space rule, applied to
/// placement as well as names).
#[test]
fn an_attribute_on_an_unsupported_item_is_dl1901() {
    let dir = scratch("placement");
    let f = dir.join("misplaced.delulu");
    std::fs::write(&f, "module m\n\n@jit\ntype Color { Red, Blue }\n\nfn main(root: Root) ! {Write} {\n  let c = root.console()\n  c.println(\"x\")\n}\n").unwrap();
    let o = delulu(&["check", &f.to_string_lossy(), "--json"]);
    assert!(!o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["diagnostics"][0]["code"], "DL1901");
}

/// The formatter round-trips attributes (identity + idempotence, the Stage-8 laws): fmt of the
/// attributed conformance fixture changes nothing, so the canonical shape IS the parsed shape.
#[test]
fn fmt_round_trips_attributed_programs() {
    let fixture = root().join("tests/conformance/accept/25_attributes_hints.delulu");
    let o = delulu(&["fmt", "--check", &fixture.to_string_lossy()]);
    assert!(
        o.status.success(),
        "the attributed fixture is already canonical — fmt must agree:\n{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
}
