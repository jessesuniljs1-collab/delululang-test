//! `delulu fix` — the command that writes to your source, and what it refuses to write.
//!
//! Every repair applied here is one the checker already computed and `delulu check --json` already
//! reported. So these tests are mostly about the refusals, because those are the whole reason a
//! batch repair command is safe to have at all.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-fix-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        // The shipped morphs, so the morph test exercises a real one rather than a fixture.
        .env("DELULU_MORPH_PATH", repo_root().join("morphs"))
        .output()
        .expect("the delulu binary must run")
}

fn write(p: &Path, s: &str) {
    std::fs::write(p, s).unwrap();
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// A function that performs `Write` without declaring it. Its repair ADDS an effect to the row —
/// the canonical authority-widening repair.
const WIDENS: &str = "module m\nfn greet(out: Cap[Console], n: Str) { out.println(n) }\n";

/// Two uses of `consume` as an identifier. DL1608's repair renames them, changing nothing about
/// what the program may do — the canonical mechanical repair.
const MECHANICAL: &str = "module m\nfn f() -> Int {\n  let consume = 1\n  consume + 1\n}\n";

/// **A repair that would widen what a program may do is never applied on its own.**
///
/// This is the line the whole command exists to hold. `delulu fix` is the kind of thing that ends
/// up in a pre-commit hook and a CI job, run unattended on every file; if it could add `Write` to
/// a row by itself, then the guarantee the language sells — that authority is decided by a person
/// and visible in the type — would be quietly negotiable by a tool.
#[test]
fn a_repair_that_would_widen_authority_is_never_applied() {
    let d = scratch("widen");
    let f = d.join("m.delulu");
    write(&f, WIDENS);

    let o = delulu(&["fix", f.to_str().unwrap()]);
    assert_eq!(read(&f), WIDENS, "the file must be untouched");

    let err = stderr(&o);
    assert!(err.contains("widens-authority"), "the refusal is named as such:\n{err}");
    assert!(err.contains("add_effect_to_row"), "and the repair is identified:\n{err}");
    // A refusal a reader cannot act on is an obstacle, not a safeguard.
    assert!(err.contains("--accept-widening add_effect_to_row"), "and the way forward is spelled out:\n{err}");
}

/// It applies when — and only when — you name that exact repair.
///
/// Naming one is a decision about one thing. There is deliberately no flag that accepts all of
/// them, because on a batch command that would mean "widen authority everywhere, unattended".
#[test]
fn a_widening_repair_applies_only_when_it_is_named() {
    let d = scratch("named");
    let f = d.join("m.delulu");
    write(&f, WIDENS);

    let o = delulu(&["fix", f.to_str().unwrap(), "--accept-widening", "add_effect_to_row"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let after = read(&f);
    assert!(after.contains("! {Write}"), "the row was added: {after}");

    // And the result is genuinely correct, not merely different.
    let chk = delulu(&["check", f.to_str().unwrap()]);
    assert_eq!(chk.status.code(), Some(0), "the repaired file checks clean:\n{}", stderr(&chk));

    // Naming a DIFFERENT repair must not unlock this one.
    let d2 = scratch("named2");
    let g = d2.join("m.delulu");
    write(&g, WIDENS);
    let o2 = delulu(&["fix", g.to_str().unwrap(), "--accept-widening", "some_other_repair"]);
    assert_eq!(read(&g), WIDENS, "an unrelated name unlocks nothing:\n{}", stderr(&o2));
}

/// The ordinary case: exact, non-widening repairs are applied and the file goes green.
///
/// Two of them, on different lines, which is also the test that edits are spliced back to front —
/// applied in ascending order the second edit's offsets would already be stale.
#[test]
fn exact_repairs_are_applied_and_the_file_goes_green() {
    let d = scratch("exact");
    let f = d.join("m.delulu");
    write(&f, MECHANICAL);

    let before = delulu(&["check", f.to_str().unwrap()]);
    assert_eq!(before.status.code(), Some(1), "precondition: the file starts broken");

    let o = delulu(&["fix", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let after = read(&f);
    assert_eq!(
        after, "module m\nfn f() -> Int {\n  let consume_ = 1\n  consume_ + 1\n}\n",
        "both uses renamed, and nothing else touched"
    );

    let chk = delulu(&["check", f.to_str().unwrap()]);
    assert_eq!(chk.status.code(), Some(0), "the file now checks clean:\n{}", stderr(&chk));
}

/// `--dry-run` reports and writes nothing, byte for byte.
#[test]
fn dry_run_writes_nothing() {
    let d = scratch("dry");
    let f = d.join("m.delulu");
    write(&f, MECHANICAL);

    let o = delulu(&["fix", f.to_str().unwrap(), "--dry-run"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    assert_eq!(read(&f), MECHANICAL, "a dry run must not touch the file");

    let err = stderr(&o);
    assert!(err.contains("would be applied"), "it says what it would do:\n{err}");
    assert!(err.contains("nothing written"), "and that it did not do it:\n{err}");
    assert!(!err.contains("\n  applied "), "and does not claim to have applied anything:\n{err}");
}

/// **A file stored in a surface morph is refused, and left exactly as it was.**
///
/// A morph file is translated to canonical DeluluLang before it is analysed, so the repairs
/// describe the canonical text, and the repaired result is canonical too. Writing that back does
/// not mis-splice anything — it replaces *every keyword the author wrote* with its canonical
/// spelling while leaving the `//! morph:` pragma still claiming their surface.
///
/// That was observed rather than deduced: with the guard removed, a `zh-CN-keywords` file asked to
/// rename one identifier came back entirely in English keywords — and `delulu check` then called
/// it clean, so nothing downstream would ever have reported the loss.
#[test]
fn a_file_stored_in_a_morph_is_refused_and_left_intact() {
    let d = scratch("morph");
    let canonical = d.join("src.delulu");
    write(&canonical, MECHANICAL);

    let rendered = delulu(&["morph", "render", canonical.to_str().unwrap(), "--to", "zh-CN-keywords"]);
    assert_eq!(rendered.status.code(), Some(0), "precondition: render works:\n{}", stderr(&rendered));
    let morphed_text = stdout(&rendered);
    assert!(morphed_text.contains("//! morph: zh-CN-keywords"), "precondition: pragma present");
    assert!(morphed_text.contains('模'), "precondition: it really is in another surface:\n{morphed_text}");

    let morphed = d.join("zh.delulu");
    write(&morphed, &morphed_text);

    // Precondition: the repair the checker offers here is exactly the kind `fix` would apply.
    let chk = delulu(&["check", morphed.to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&chk)).expect("one envelope");
    let r = &v["diagnostics"][0]["repairs"][0];
    assert_eq!(r["confidence"], "exact", "precondition: an exact repair");
    assert_eq!(r["authority_widening"], false, "precondition: not a widening one");

    let o = delulu(&["fix", morphed.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1), "it refuses:\n{}", stderr(&o));
    assert_eq!(read(&morphed), morphed_text, "and the file is byte-identical afterwards");

    let err = stderr(&o);
    assert!(err.contains("not stored as canonical"), "the reason is stated:\n{err}");
    assert!(err.contains("--to-canonical"), "and the way through is given:\n{err}");
}

/// The documented way through actually works — translate, fix, translate back.
///
/// The refusal above prints these three commands. A workaround that has never been run is a
/// suggestion, not a remedy, so it is run here: the author's keywords must survive and the repair
/// must land.
#[test]
fn the_documented_morph_workaround_preserves_the_authors_keywords() {
    let d = scratch("workaround");
    let start = d.join("src.delulu");
    write(&start, MECHANICAL);

    let zh = d.join("zh.delulu");
    write(&zh, &stdout(&delulu(&["morph", "render", start.to_str().unwrap(), "--to", "zh-CN-keywords"])));

    // 1. translate to canonical
    let canon = d.join("canon.delulu");
    write(&canon, &stdout(&delulu(&["morph", "render", zh.to_str().unwrap(), "--to-canonical"])));
    // 2. fix
    let fixed = delulu(&["fix", canon.to_str().unwrap()]);
    assert_eq!(fixed.status.code(), Some(0), "{}", stderr(&fixed));
    // 3. translate back
    let back = stdout(&delulu(&["morph", "render", canon.to_str().unwrap(), "--to", "zh-CN-keywords"]));
    write(&zh, &back);

    assert!(back.contains('模'), "the author's keywords survived:\n{back}");
    assert!(back.contains("consume_"), "and the repair landed:\n{back}");
    let chk = delulu(&["check", zh.to_str().unwrap()]);
    assert_eq!(chk.status.code(), Some(0), "and the result checks clean:\n{}", stderr(&chk));
}

/// One JSON object on stdout, with a verdict recorded for every repair — applied or not.
///
/// The skipped ones matter more than the applied ones here: an agent needs to know that a repair
/// exists and *why this tool would not apply it*, or it will conclude there was nothing to do.
#[test]
fn the_json_envelope_holds_one_object_and_a_verdict_per_repair() {
    let d = scratch("json");
    let f = d.join("m.delulu");
    write(&f, WIDENS);

    let o = delulu(&["fix", f.to_str().unwrap(), "--json"]);
    let out = stdout(&o);
    let mut stream = serde_json::Deserializer::from_str(&out).into_iter::<serde_json::Value>();
    let first = stream.next().expect("stdout must hold a JSON value");
    let v = first.expect("stdout must parse as JSON");
    assert!(stream.next().is_none(), "stdout must hold EXACTLY one JSON value:\n{out}");

    assert_eq!(v["command"], "fix");
    assert_eq!(v["schema"], 1);
    assert_eq!(v["fix"]["applied"], 0);
    assert_eq!(v["fix"]["written"], false);
    let repairs = v["fix"]["repairs"].as_array().expect("repairs are reported");
    assert_eq!(repairs.len(), 1, "the skipped repair is still reported: {repairs:?}");
    assert_eq!(repairs[0]["verdict"], "widens-authority");
    assert_eq!(repairs[0]["code"], "DL0501");
    assert_eq!(repairs[0]["id"], "add_effect_to_row");
}

/// A file with nothing to repair is handled, not treated as an error.
#[test]
fn a_clean_file_is_left_alone() {
    let d = scratch("clean");
    let f = d.join("m.delulu");
    let src = "module m\nfn f() -> Int { 1 }\n";
    write(&f, src);

    let o = delulu(&["fix", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    assert_eq!(read(&f), src, "an untouched file stays untouched");
    assert!(stderr(&o).contains("nothing to repair"), "{}", stderr(&o));
}

/// The negative half of the CLI contract: an unknown option is refused, not ignored.
#[test]
fn fix_refuses_an_option_it_does_not_know() {
    let d = scratch("bad");
    let f = d.join("m.delulu");
    write(&f, MECHANICAL);

    let o = delulu(&["fix", f.to_str().unwrap(), "--accept-everything"]);
    assert_eq!(o.status.code(), Some(2), "a bad invocation exits 2");
    assert_eq!(read(&f), MECHANICAL, "and changes nothing");
    assert!(stderr(&o).contains("unknown option"), "{}", stderr(&o));

    // `--accept-widening` without a value must not silently mean "accept all".
    let o2 = delulu(&["fix", f.to_str().unwrap(), "--accept-widening"]);
    assert_eq!(o2.status.code(), Some(2), "a dangling flag exits 2");
    assert_eq!(read(&f), MECHANICAL, "and changes nothing");

    // `fix` with no file at all.
    let o3 = delulu(&["fix"]);
    assert_eq!(o3.status.code(), Some(2), "no file exits 2");
}

/// `--help` must answer the question, never perform the action.
///
/// This is the D11 rule that `delulu keygen --help` used to break by generating a key. A command
/// that edits source files is the worst possible place to get that wrong.
#[test]
fn help_does_not_edit_anything() {
    let d = scratch("help");
    let f = d.join("m.delulu");
    write(&f, MECHANICAL);

    for args in [vec!["fix", "--help"], vec!["fix", f.to_str().unwrap(), "--help"]] {
        let o = delulu(&args);
        assert_eq!(o.status.code(), Some(0), "{args:?}");
        assert_eq!(read(&f), MECHANICAL, "asking what `fix` does must not fix anything: {args:?}");
    }
}

/// **A file `fix` cannot act on is REFUSED, not reported as clean.**
///
/// `delulu fix notes.txt` printed `nothing to repair` and exited **0** — "nothing done, success
/// claimed, on a path the user named deliberately" — while `delulu check` on the same bytes gave
/// `DL0204`. That is campaign finding **C66** exactly, which `fmt` was corrected for in D59; `fix`
/// was written *after* that ruling and did not inherit it (C73/D72).
///
/// The comparison is the whole point: three commands, one file, and only one of them was honest.
#[test]
fn a_file_that_is_not_delulu_source_is_refused_rather_than_called_clean() {
    let d = scratch("not-delulu");
    let notes = d.join("notes.txt");
    write(&notes, "notes, not delulu\n");
    let p = notes.to_str().unwrap();

    let o = delulu(&["fix", p]);
    let out = format!("{}{}", String::from_utf8_lossy(&o.stdout), stderr(&o));
    assert!(
        !o.status.success(),
        "`fix` on a non-source file must refuse; reporting success is C66's exact shape: {out}"
    );
    assert!(
        !out.contains("nothing to repair"),
        "a file `fix` cannot process is not a file with nothing to repair: {out}"
    );
    assert!(out.contains("not a `.delulu` source file"), "the refusal must say why: {out}");

    // And the file is untouched — a refusal that edited something would be worse than the bug.
    assert_eq!(read(&notes), "notes, not delulu\n", "a refused file is never rewritten");
}

/// THE SKIP-BRANCH CASE: a real `.delulu` file with nothing to repair must still say so and succeed.
/// A refusal that fired on every path would pass the test above and break the command.
#[test]
fn a_clean_delulu_file_still_reports_nothing_to_repair_and_succeeds() {
    let d = scratch("clean-source");
    let src = d.join("clean.delulu");
    write(&src, "module clean\n\nfn main(root: Root) {\n}\n");
    let o = delulu(&["fix", src.to_str().unwrap()]);
    let out = format!("{}{}", String::from_utf8_lossy(&o.stdout), stderr(&o));
    assert!(o.status.success(), "a clean source file is not an error: {out}");
    assert!(out.contains("nothing to repair"), "and it says so plainly: {out}");
}
// ----- the repair registry, both channels (verification finding NE-07) ---------------------------

/// Every repair id the compiler can construct, read out of the source that constructs them.
///
/// A hand-written list falls behind the thing that defines it (C31/C34/C35/C44/C52), so the
/// definition is read instead: the `id:` line of every `Repair { … }` literal in the crates that
/// build repairs.
fn repair_ids_in_source() -> Vec<String> {
    let files = [
        "crates/delulu-syntax/src/parser.rs",
        "crates/delulu-check/src/check.rs",
        "crates/delulu-check/src/deps.rs",
        "crates/delulu-check/src/plugin.rs",
        "crates/delulu-check/src/rcap_check.rs",
        "crates/delulu-broker/src/diag.rs",
    ];
    let mut ids: Vec<String> = Vec::new();
    for f in files {
        let src = std::fs::read_to_string(repo_root().join(f))
            .unwrap_or_else(|e| panic!("{f} must be readable — it is what this gate reads: {e}"));
        let mut rest = src.as_str();
        while let Some(at) = rest.find("Repair {") {
            rest = &rest[at + 8..];
            // The `id:` of this literal is the first one after the brace.
            let Some(idx) = rest.find("id: \"") else { break };
            let after = &rest[idx + 5..];
            let Some(end) = after.find('"') else { break };
            let id = after[..end].to_string();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    assert!(
        ids.len() >= 10,
        "the registry scan found only {} ids — a gate that reads nothing passes everything",
        ids.len()
    );
    ids
}

/// Repair ids no single-file `check` can provoke, each with the reason. Everything else must be
/// exercised by the corpus below, so a new repair cannot be born unexamined.
const REPAIRS_NOT_REACHABLE_FROM_ONE_FILE: &[(&str, &str)] = &[
    (
        "insert_dependency_pin",
        "needs a PACKAGE with an unpinned dependency, not a single file (covered by `tooling.rs`)",
    ),
    (
        "regenerate_manifest_export",
        "needs a plugin manifest that disagrees with its code (covered by `plugin_cli.rs`)",
    ),
    (
        "R-DL0802-attenuate-to-intersection",
        "built by the BROKER on a live attenuation denial, never by `check` (covered by `guard_e2e.rs`)",
    ),
    (
        "name-the-marshallable-type",
        "needs a `foreign` block with an unmarshallable type in its signature",
    ),
    ("delete-behavior-return-type", "needs an actor behavior declared with a return type"),
    ("normalize-actor-rcap", "needs an actor-typed binding carrying a non-`tag` rcap"),
    ("consume-iso-send", "needs an `iso` value sent across an actor boundary without `consume`"),
    ("rename-v07-keyword", "needs pre-0.7 source using `consume`/`recover` as an identifier"),
];

/// Source files that provoke the repairs this gate sweeps. Each is written here rather than
/// borrowed from the corpus so the gate states, in one place, exactly which repair it is about.
fn repair_corpus() -> Vec<(&'static str, &'static str)> {
    vec![
        // DL0502 `remove_effect_from_row` — the NE-07 repair: safe, and with no edits.
        (
            "unused_effect.delulu",
            "module unusedeffect\n\nfn main(root: Root) ! {Write} {\n    let x = 1\n}\n",
        ),
        // DL0501 `add_effect_to_row` — exact, WIDENING, with edits.
        (
            "undeclared_effect.delulu",
            "module undeclared\n\nfn main(root: Root) {\n    let out = root.console()\n    out.println(\"x\")\n}\n",
        ),
        // DL0201 `brace-match-arm-assignment` — exact, with edits.
        (
            "match_assign.delulu",
            "module matchassign\n\nfn f(flag: Bool) {\n    var n = 0\n    match flag {\n        true => n = 1\n        false => n = 2\n    }\n}\n",
        ),
        // `remove-unknown-attribute` — exact, with edits.
        (
            "bad_attribute.delulu",
            "module badattr\n\n@nosuchattribute\nfn f() -> Int { 1 }\n",
        ),
        // DL1603 `drop-val-annotation` (NE-02) — safe, with edits, and NOT auto-applied.
        (
            "val_over_ref.delulu",
            "module valoverref\n\nfn f() {\n    let a: ref List[Int] = [1, 2]\n    let rows: val List[List[Int]] = [a]\n}\n",
        ),
    ]
}

/// NE-07. DL0502 offered `remove_effect_from_row` with `confidence: "safe"`,
/// `authority_widening: false`, `requires_human: false` and **`edits: []`** — every flag reading
/// *apply me* — while `delulu fix --dry-run --json` refused the same repair with
/// `verdict: "requires-human"` and the LSP offered it as `isPreferred: true` with no `edit`. Three
/// channels, three answers, about one repair.
///
/// The rule this gate holds: **a repair with no edits carries `requires_human: true` and says
/// why**, and the flags `check --json` publishes and the verdicts `fix` reaches never disagree.
#[test]
fn a_repair_with_no_edits_says_a_human_must_decide_and_why() {
    let dir = scratch("registry");
    let mut seen: Vec<String> = Vec::new();
    let mut bad: Vec<String> = Vec::new();

    for (name, src) in repair_corpus() {
        let f = dir.join(name);
        std::fs::write(&f, src).unwrap();
        let path = f.to_str().unwrap();

        let chk = delulu(&["check", path, "--json"]);
        let cv: serde_json::Value =
            serde_json::from_slice(&chk.stdout).unwrap_or_else(|e| panic!("{name}: check --json: {e}"));
        let fix = delulu(&["fix", path, "--dry-run", "--json"]);
        let fv: serde_json::Value =
            serde_json::from_slice(&fix.stdout).unwrap_or_else(|e| panic!("{name}: fix --json: {e}"));

        let mut repairs_here = 0usize;
        for d in cv["diagnostics"].as_array().unwrap_or(&Vec::new()) {
            for r in d["repairs"].as_array().unwrap_or(&Vec::new()) {
                repairs_here += 1;
                let id = r["id"].as_str().unwrap_or("").to_string();
                if !seen.contains(&id) {
                    seen.push(id.clone());
                }
                let editless = r["edits"].as_array().map(|a| a.is_empty()).unwrap_or(true);
                if editless {
                    if r["requires_human"] != true {
                        bad.push(format!(
                            "{name}: repair `{id}` has no edits but requires_human = {} — the flags \
                             say a machine can apply a repair that cannot be applied at all",
                            r["requires_human"]
                        ));
                    }
                    match r["reason"].as_str() {
                        Some(s) if s.len() > 20 => {}
                        _ => bad.push(format!(
                            "{name}: repair `{id}` has no edits and no usable `reason`: {}",
                            r["reason"]
                        )),
                    }
                } else if !r["reason"].is_null() {
                    bad.push(format!(
                        "{name}: repair `{id}` carries edits AND a reason — `reason` is why there is \
                         nothing to apply, so it must be null here"
                    ));
                }

                // The cross-channel agreement. `fix` classifies widening BEFORE requires_human, so
                // the implication is one-directional for a widening repair — stated exactly rather
                // than approximated, because an approximate gate passes what it does not model.
                let verdict = fv["fix"]["repairs"]
                    .as_array()
                    .and_then(|a| a.iter().find(|x| x["id"] == r["id"]))
                    .map(|x| x["verdict"].as_str().unwrap_or("").to_string())
                    .unwrap_or_default();
                if verdict.is_empty() {
                    bad.push(format!("{name}: repair `{id}` has no verdict in `fix --dry-run --json`"));
                    continue;
                }
                let requires_human = r["requires_human"] == true;
                let widening = r["authority_widening"] == true;
                if verdict == "requires-human" && !requires_human {
                    bad.push(format!(
                        "{name}: `fix` says `{id}` requires a human and `check --json` says it does not"
                    ));
                }
                if requires_human && !widening && verdict != "requires-human" {
                    bad.push(format!(
                        "{name}: `check --json` flags `{id}` requires_human and `fix` reached `{verdict}`"
                    ));
                }
                if widening && verdict != "widens-authority" {
                    bad.push(format!(
                        "{name}: `check --json` flags `{id}` authority_widening and `fix` reached `{verdict}`"
                    ));
                }
            }
        }
        assert!(repairs_here > 0, "{name} must produce at least one repair, or it witnesses nothing");
    }

    // Completeness: every repair the source can construct is either exercised above or excused in
    // writing. Without this the corpus could quietly stop covering a repair and the gate would
    // still pass.
    let mut unswept: Vec<String> = Vec::new();
    for id in repair_ids_in_source() {
        let excused = REPAIRS_NOT_REACHABLE_FROM_ONE_FILE.iter().any(|(n, _)| *n == id);
        if !seen.contains(&id) && !excused {
            unswept.push(id);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        unswept.is_empty(),
        "these repairs are constructed by the compiler but no case here exercises them: {unswept:?}\n\
         Add a case to `repair_corpus`, or an entry to REPAIRS_NOT_REACHABLE_FROM_ONE_FILE with the reason."
    );
    assert!(bad.is_empty(), "the repair channels must agree:\n  {}", bad.join("\n  "));
}
