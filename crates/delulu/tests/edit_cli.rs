//! P4-04 and P4-05: `delulu edit`, driven through the real binary.
//!
//! What is held here: an edit computed against bytes the file no longer has is REFUSED, with the
//! file's current hash and nothing written (the falsifier the roadmap names: *edit after the hash*);
//! a typed repair's `edits` pass straight through and the result is checked; a node addressed by its
//! Atlas id is replaced and formatted, and an id the file no longer has is refused as stale; the
//! answer says when an edit widens what the program may do; `--dry-run` and `--if-checks` write
//! nothing; a file stored in a surface morph is refused; and — the corpus witness — every function
//! and type the Atlas names in the shipped examples resolves, and replacing each with its own text
//! gives back the same bytes.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-edit-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn hash(path: &Path) -> String {
    blake3::hash(&std::fs::read(path).unwrap()).to_hex().to_string()
}

/// Run `delulu edit` with `args` in `cwd`; the exit code and the parsed `--json` envelope.
fn edit(cwd: &Path, args: &[&str]) -> (i32, Value) {
    let out = Command::new(bin())
        .arg("edit")
        .args(args)
        .arg("--json")
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .unwrap();
    let env: Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("not one JSON envelope ({e}): {}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
    // Every answer — applied or refused — is what `delulu schema edit` says it is.
    for (schema, value) in [("envelope", &env), ("edit", &env["edit"])] {
        let f = cwd.join(format!(".answer-{schema}.json"));
        std::fs::write(&f, value.to_string()).unwrap();
        let v = Command::new(bin()).args(["schema", "validate", schema]).arg(&f).arg("--json").output().unwrap();
        let _ = std::fs::remove_file(&f);
        let v: Value = serde_json::from_slice(&v.stdout).unwrap();
        assert_eq!(v["validate"]["valid"], true, "{schema}: {:#}\n{value:#}", v["validate"]["errors"]);
    }
    (out.status.code().unwrap(), env)
}

const PROGRAM: &str = "module m\n\nfn a() -> Int {\n    1\n}\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(\"hi\")\n}\n";

#[test]
fn an_edit_computed_before_the_file_changed_is_refused_with_the_current_hash() {
    let dir = scratch("stale");
    let f = dir.join("m.delulu");
    std::fs::write(&f, PROGRAM).unwrap();
    let read_at = hash(&f);
    // Someone else writes between the agent's read and its edit.
    let theirs = PROGRAM.replace("\"hi\"", "\"hello\"");
    std::fs::write(&f, &theirs).unwrap();
    let edits = json!([{ "range": { "start_byte": 0, "end_byte": 8 }, "insert": "module z" }]).to_string();
    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &read_at, "--edits", &edits]);
    assert_eq!(code, 2, "{env:#}");
    assert!(env["edit"]["refused"].as_str().unwrap().contains("changed since"), "{env:#}");
    assert_eq!(env["edit"]["hash"], json!(hash(&f)), "the refusal carries the hash the file has NOW");
    assert_eq!(env["edit"]["written"], false);
    assert_eq!(env["summary"]["errors"], 1, "a refusal is not a pass");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), theirs, "their write survives untouched");
    // With the hash it reported, the same edit goes through.
    let now = env["edit"]["hash"].as_str().unwrap().to_string();
    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &now.to_ascii_uppercase(), "--edits", &edits]);
    assert_eq!(code, 0, "{env:#}");
    assert!(std::fs::read_to_string(&f).unwrap().starts_with("module z\n"));
    assert_eq!(env["edit"]["previous_hash"], json!(now));
    assert_eq!(env["edit"]["hash"], json!(hash(&f)), "the answer's hash is the file's");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_repairs_edits_pass_straight_through_and_the_result_is_checked() {
    let dir = scratch("repair");
    let f = dir.join("b.delulu");
    std::fs::write(&f, "module b\n\nfn main(root: Root) {\n    let c = root.console()\n    c.println(\"hi\")\n}\n").unwrap();
    let checked: Value = serde_json::from_slice(
        &Command::new(bin()).args(["check", "b.delulu", "--json"]).current_dir(&dir).env("DELULU_NO_FIRST_RUN", "1").output().unwrap().stdout,
    )
    .unwrap();
    let repair = &checked["diagnostics"][0]["repairs"][0];
    assert_eq!(repair["id"], "add_effect_to_row");
    std::fs::write(dir.join("r.json"), repair["edits"].to_string()).unwrap();
    let (code, env) = edit(&dir, &["b.delulu", "--expect-hash", &hash(&f), "--edits", "@r.json"]);
    assert_eq!(code, 0, "{env:#}");
    assert_eq!(env["summary"]["errors"], 0);
    assert_eq!(env["diagnostics"], json!([]), "the edited file is checked, and it is clean");
    assert_eq!(env["edit"]["edits_applied"], 1);
    assert!(std::fs::read_to_string(&f).unwrap().contains("fn main(root: Root) ! {Write} {"));
    // The old program did not check, so it had no authority to compare with: said, not guessed.
    assert_eq!(env["edit"]["authority"]["before"], Value::Null);
    assert_eq!(env["edit"]["authority"]["widened"], Value::Null);
    assert_eq!(env["edit"]["authority"]["after"]["effects"], json!(["Write"]));

    // Edits that overlap, or a field an edit does not have, are refused whole: nothing is written.
    let before = std::fs::read_to_string(&f).unwrap();
    for bad in [
        json!([{ "range": { "start_byte": 0, "end_byte": 4 }, "insert": "" }, { "range": { "start_byte": 2, "end_byte": 6 }, "insert": "" }]),
        json!([{ "range": { "start_byte": 0, "end_byte": 1 }, "insert": "", "confidence": "exact" }]),
        json!([{ "range": { "start_byte": 0, "end_byte": 100000 }, "insert": "" }]),
    ] {
        let (code, env) = edit(&dir, &["b.delulu", "--expect-hash", &hash(&f), "--edits", &bad.to_string()]);
        assert_eq!(code, 2, "{bad}: {env:#}");
        assert_eq!(env["edit"]["written"], false);
    }
    assert_eq!(std::fs::read_to_string(&f).unwrap(), before);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_node_is_replaced_by_its_atlas_id_and_formatted_and_a_stale_id_is_refused() {
    let dir = scratch("node");
    let f = dir.join("m.delulu");
    std::fs::write(&f, PROGRAM).unwrap();
    // The id is the Atlas's own, not one this test spells.
    let atlas: Value = serde_json::from_slice(
        &Command::new(bin()).args(["atlas", "m.delulu", "--json"]).current_dir(&dir).env("DELULU_NO_FIRST_RUN", "1").output().unwrap().stdout,
    )
    .unwrap();
    let id = atlas["atlas"]["nodes"].as_array().unwrap().iter().find(|n| n["name"] == "a").unwrap()["id"].as_str().unwrap().to_string();
    assert_eq!(id, "fn:m/m.a");

    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &hash(&f), "--node", &id, "--with", "fn a()->Int{2}"]);
    assert_eq!(code, 0, "{env:#}");
    assert_eq!(env["edit"]["node"], json!(id));
    assert_eq!(env["edit"]["authority"]["widened"], json!([]), "a pure function stays pure");
    let now = std::fs::read_to_string(&f).unwrap();
    assert!(now.contains("fn a() -> Int {\n    2\n}\n\nfn main"), "the item is formatted, its neighbours untouched:\n{now}");

    // Renamed by someone else: the old id names nothing now, and is refused — never guessed.
    std::fs::write(&f, now.replace("fn a()", "fn b()")).unwrap();
    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &hash(&f), "--node", &id, "--with", "fn a() -> Int { 3 }"]);
    assert_eq!(code, 2, "{env:#}");
    assert!(env["edit"]["refused"].as_str().unwrap().contains("stale"), "{env:#}");

    // A replacement that is two items, or the wrong kind, is refused.
    for with in ["fn b() -> Int { 3 }\nfn c() -> Int { 4 }", "type T = Int"] {
        let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &hash(&f), "--node", "fn:m/m.b", "--with", with]);
        assert_eq!(code, 2, "{with}: {env:#}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_edit_that_widens_what_the_program_may_do_says_so() {
    let dir = scratch("widen");
    let f = dir.join("m.delulu");
    std::fs::write(&f, PROGRAM).unwrap();
    let reads = "fn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let fr = root.fs_read(\"./data\")\n    \
                 match fr.read_text(\"data/in.txt\") {\n        Ok(v) => out.println(v),\n        Err(_) => out.println(\"miss\"),\n    }\n}\n";
    std::fs::write(dir.join("main.txt"), reads).unwrap();
    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &hash(&f), "--node", "fn:m/m.main", "--with", "@main.txt", "--dry-run"]);
    assert_eq!(code, 0, "{env:#}");
    let auth = &env["edit"]["authority"];
    assert_eq!(auth["before"]["effects"], json!(["Write"]));
    assert_eq!(auth["widened"], json!(["Read", "fs.read=./data"]), "{auth:#}");
    assert_eq!(env["edit"]["written"], false, "--dry-run writes nothing");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), PROGRAM);
    // The human channel says it too.
    let out = Command::new(bin())
        .args(["edit", "m.delulu", "--expect-hash", &hash(&f), "--node", "fn:m/m.main", "--with", "@main.txt", "--dry-run"])
        .current_dir(&dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stderr).contains("WIDENS what the program may do: adds Read, fs.read=./data"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn if_checks_writes_only_a_result_that_checks_and_a_morph_file_is_refused() {
    let dir = scratch("gates");
    let f = dir.join("m.delulu");
    std::fs::write(&f, PROGRAM).unwrap();
    // Deleting main's row leaves a program that does not check.
    let start = PROGRAM.find("! {Write} ").unwrap();
    let edits = json!([{ "range": { "start_byte": start, "end_byte": start + "! {Write} ".len() }, "insert": "" }]).to_string();
    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &hash(&f), "--edits", &edits, "--if-checks"]);
    assert_eq!(code, 1, "{env:#}");
    assert_eq!(env["edit"]["written"], false);
    assert_eq!(env["diagnostics"][0]["code"], "DL0501", "the answer carries what is wrong with the result");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), PROGRAM);
    // Without --if-checks the caller asked for the write, and gets it — with the errors reported.
    let (code, env) = edit(&dir, &["m.delulu", "--expect-hash", &hash(&f), "--edits", &edits]);
    assert_eq!((code, env["edit"]["written"].clone()), (1, json!(true)), "{env:#}");

    let g = dir.join("g.delulu");
    std::fs::write(&g, format!("//! morph: compact\n{PROGRAM}")).unwrap();
    let (code, env) = edit(&dir, &["g.delulu", "--expect-hash", &hash(&g), "--edits", "[]"]);
    assert_eq!(code, 2, "{env:#}");
    assert!(env["edit"]["refused"].as_str().unwrap().contains("surface morph"));

    // The usage is refused before any file is touched.
    for args in [
        vec!["m.delulu", "--edits", "[]"],
        vec!["m.delulu", "--expect-hash", "x", "--edits", "[]", "--node", "fn:m/m.a", "--with", "fn a() {}"],
        vec!["m.delulu", "--expect-hash", "x", "--node", "fn:m/m.a"],
        vec!["m.delulu", "--expect-hash", "x", "--edits", "[]", "--force"],
    ] {
        let out = Command::new(bin()).arg("edit").args(&args).current_dir(&dir).env("DELULU_NO_FIRST_RUN", "1").output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The corpus witness (P4-05): every function and type the Atlas names in the shipped examples
/// resolves by that id, and replacing each with its own text — formatted on its own — gives back
/// the file's exact bytes. That holds the splice at both ends, the id resolution against the
/// Atlas's real ids, and the per-item formatter against the whole-file one.
#[test]
fn every_atlas_node_in_the_examples_resolves_and_round_trips_through_its_own_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples").canonicalize().unwrap();
    let mut files = Vec::new();
    let mut stack = vec![root];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "delulu") {
                files.push(p);
            }
        }
    }
    files.sort();
    let work = scratch("corpus");
    let mut witnessed = 0;
    for file in &files {
        let src = std::fs::read_to_string(file).unwrap();
        let atlas = Command::new(bin()).args(["atlas"]).arg(file).arg("--json").env("DELULU_NO_FIRST_RUN", "1").output().unwrap();
        let Ok(atlas) = serde_json::from_slice::<Value>(&atlas.stdout) else { continue };
        let Some(nodes) = atlas["atlas"]["nodes"].as_array() else { continue };
        let (module, _) = delulu_syntax::parse_file(0, &src);
        let copy = work.join("f.delulu");
        std::fs::write(&copy, &src).unwrap();
        let h = hash(&copy);
        for n in nodes {
            let id = n["id"].as_str().unwrap();
            if !(id.starts_with("fn:") || id.starts_with("type:")) || n["module"].as_str() != Some(&module.name.dotted()) {
                continue;
            }
            let name = n["name"].as_str().unwrap();
            use delulu_syntax::ast::Item;
            let span = module.items.iter().find_map(|it| match (it, name.split_once('.')) {
                (Item::Fn(f), None) if f.name.name == name && id.starts_with("fn:") => Some(f.span),
                (Item::Type(t), None) if t.name.name == name && id.starts_with("type:") => Some(t.span),
                (Item::Actor(a), Some((actor, member))) if a.name.name == actor => match member {
                    "new" => Some(a.ctor.span),
                    m => a.behaviors.iter().find(|b| b.name.name == m).map(|b| b.span).or_else(|| a.fns.iter().find(|f| f.name.name == m).map(|f| f.span)),
                },
                _ => None,
            });
            let Some(span) = span else { panic!("{}: the Atlas names `{id}`, the parser has no such item", file.display()) };
            std::fs::write(work.join("item.txt"), &src[span.start as usize..span.end as usize]).unwrap();
            let (code, env) = edit(&work, &["f.delulu", "--expect-hash", &h, "--node", id, "--with", "@item.txt", "--dry-run"]);
            assert_ne!(code, 2, "{}: `{id}` was refused: {env:#}", file.display());
            assert_eq!(env["edit"]["hash"], json!(h), "{}: `{id}` did not round-trip", file.display());
            witnessed += 1;
        }
    }
    eprintln!("witnessed {witnessed} nodes across {} example files", files.len());
    assert!(witnessed >= 40, "the corpus witnessed only {witnessed} nodes");
    let _ = std::fs::remove_dir_all(&work);
}
