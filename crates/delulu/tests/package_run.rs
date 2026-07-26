//! Running a multi-package program (campaign finding C59, ruling D61).
//!
//! `delulu run` took a single `.delulu` file or a `.dwx`. A package could be checked, built,
//! reported on and reasoned about — and never executed. `kind = "bin"` was a manifest field the
//! toolchain could not honour, and that was true of the shipped two-module `examples/greeter/` too.
//!
//! The order these tests assert is the ruling. `check_workspace` stays authoritative — per-module
//! visibility, package authority ceilings, dependency pins — and nothing runs unless it passes. Only
//! then is the program flattened for the interpreter.
//!
//! Flattening is done on SOURCE, not on merged ASTs, and one test pins the reason: separately-parsed
//! modules have overlapping `NodeId`s, so an AST merge makes the checker's node-keyed side tables
//! read one module's entry for another module's expression. The first implementation did that and
//! produced a nonsense reference-capability complaint about a correct program — a wrong answer rather
//! than an error, which is the worst kind.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn write(dir: &Path, rel: &str, body: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

/// Copy a tree so a run cannot write into the repository.
fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap().flatten() {
        let p = e.path();
        let name = p.file_name().unwrap();
        if p.is_dir() {
            if name == "target" {
                continue; // build output, not source
            }
            copy_tree(&p, &dst.join(name));
        } else {
            std::fs::copy(&p, dst.join(name)).unwrap();
        }
    }
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-pkgrun-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The corpus tier-4 program — four packages, seven modules, depth three, a diamond — RUN, with its
/// output and the files it wrote asserted. Not "checks clean": runs and is correct.
#[test]
fn a_four_package_seven_module_program_runs_and_produces_the_right_answer() {
    let dir = scratch("tier4");
    copy_tree(&repo_root().join("tests/corpus/tier4-multimodule"), &dir);
    let station = dir.join("station");
    write(
        &station,
        "data/telemetry.txt",
        "bay3=21.5\nbay4=8.25\n\nbay5=41.0\nbroken\nbay6=abc\n",
    );
    std::fs::create_dir_all(station.join("out")).unwrap();

    let o = delulu(
        &station,
        &["run", ".", "--grant", "console", "--grant", "fs.read=./data", "--grant",
          "fs.write=./out", "--no-prompt"],
    );
    assert!(o.status.success(), "the package must run: {}\n{}", stdout(&o), stderr(&o));
    let s = stdout(&o);

    // Each of the four packages contributed: `reading` decoded, `policy` classified, `archive`
    // wrote, `station` reported. A flatten that dropped a module would lose one of these lines.
    assert!(s.contains("bay3: nominal"), "policy must classify a nominal reading:\n{s}");
    assert!(s.contains("bay4: COLD at 8.25"), "and a cold one:\n{s}");
    assert!(s.contains("bay5: HOT at 41.0"), "and a hot one:\n{s}");
    assert!(s.contains("malformed line: broken"), "reading.parse must reject a malformed line:\n{s}");
    assert!(s.contains("not a temperature: abc"), "and a non-numeric one:\n{s}");
    assert!(s.contains("2 reading(s) outside limits"), "station.report must total the alarms:\n{s}");

    // `archive` is the only package granted Write, and it used it.
    let readings = std::fs::read_to_string(station.join("out/readings.log")).expect("readings.log");
    assert!(readings.contains("bay3,21.5"), "the archive must hold the decoded rows: {readings}");
    let alarms = std::fs::read_to_string(station.join("out/alarms.log")).expect("alarms.log");
    assert!(alarms.contains("COLD") || alarms.contains("HOT"), "alarms must be archived: {alarms}");
}

/// Authority is not weakened by the package path. Every capability the program needs is still a
/// grant, and dropping any one of them refuses by name before or during the run.
#[test]
fn a_package_run_still_holds_every_grant() {
    let dir = scratch("grants");
    copy_tree(&repo_root().join("tests/corpus/tier4-multimodule"), &dir);
    let station = dir.join("station");
    write(&station, "data/telemetry.txt", "bay3=21.5\n");
    std::fs::create_dir_all(station.join("out")).unwrap();

    for (missing, args) in [
        ("console", vec!["fs.read=./data", "fs.write=./out"]),
        ("filesystem read", vec!["console", "fs.write=./out"]),
        ("filesystem write", vec!["console", "fs.read=./data"]),
    ] {
        let mut a = vec!["run", "."];
        for g in &args {
            a.push("--grant");
            a.push(g);
        }
        a.push("--no-prompt");
        let o = delulu(&station, &a);
        assert!(!o.status.success(), "dropping {missing} must refuse the run");
        let all = stdout(&o) + &stderr(&o);
        assert!(all.contains("DL0703"), "the refusal is DL0703: {all}");
        assert!(all.contains(missing), "and it names what was missing ({missing}): {all}");
    }
}

/// The shipped example. `examples/greeter/` is a two-module `kind = "bin"` package that has been in
/// this tree since Stage 2 and could only ever be checked — the plainest demonstration that the gap
/// was real rather than theoretical.
#[test]
fn the_shipped_greeter_example_runs() {
    let dir = scratch("greeter");
    copy_tree(&repo_root().join("examples/greeter"), &dir);
    let o = delulu(&dir, &["run", ".", "--grant", "console", "--no-prompt"]);
    assert!(o.status.success(), "greeter must run: {}\n{}", stdout(&o), stderr(&o));
    let s = stdout(&o);
    assert!(s.contains("hello, world"), "the imported `salutation` must be called:\n{s}");
    assert!(s.contains('3'), "and the imported `shout_count`:\n{s}");
}

/// The fail-closed edge. Two modules may legally declare the same private name — visibility is per
/// module — and flattening cannot tell them apart. That case is REFUSED, and the refusal says the
/// program is correct, because it is: `check`, `build` and `authority` all handle it. A runner that
/// silently picked whichever module merged last would be answering the wrong question quietly.
#[test]
fn two_modules_sharing_a_name_are_refused_by_the_runner_and_accepted_by_everything_else() {
    let dir = scratch("collide");
    write(
        &dir,
        "delulu.toml",
        "[package]\nname = \"collide\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n[authority]\neffects = [\"Write\"]\n",
    );
    write(
        &dir,
        "src/collide.delulu",
        "module collide\n\
         import collide.helper\n\
         fn shared(n: Int) -> Int { n + 1 }\n\
         fn main(root: Root) ! {Write} {\n\
         \x20 let out = root.console()\n\
         \x20 out.println(str(shared(1)) + \" \" + str(exported(2)))\n\
         }\n",
    );
    write(
        &dir,
        "src/helper.delulu",
        "module collide.helper\n\
         fn shared(n: Int) -> Int { n * 100 }\n\
         pub fn exported(n: Int) -> Int { shared(n) }\n",
    );

    // The program is correct, and the tools that analyse it say so.
    for cmd in ["check", "build", "authority"] {
        let o = delulu(&dir, &[cmd, "."]);
        assert!(o.status.success(), "`{cmd}` must accept this program: {}", stderr(&o));
    }

    // The runner refuses it, names the collision, and does not blame the author.
    let o = delulu(&dir, &["run", ".", "--grant", "console", "--no-prompt"]);
    assert!(!o.status.success(), "the runner must refuse a name collision rather than guess");
    let e = stderr(&o);
    assert!(e.contains("shared"), "the refusal must name the colliding declaration: {e}");
    assert!(
        e.contains("the program is not wrong"),
        "and must say the limit is the RUNNER's, or it sends someone hunting a bug that is not there: {e}"
    );
    assert!(e.contains("C59"), "and cite the finding: {e}");
}

/// Flattening happens on SOURCE, and this is the regression that pins why.
///
/// The first implementation merged the module ASTs. Each module is parsed separately, so their
/// `NodeId`s both start at zero and overlap; the checker's `node_types` (and through it the
/// reference-capability analysis) then reads one module's entry for another's expression. On the
/// tier-4 corpus that produced `cannot store 'box' where 'val' is required` — about a program the
/// workspace had just accepted. A wrong answer, not an error.
///
/// A program that exercises the same shape must therefore run cleanly, with no reference-capability
/// complaint anywhere in the output.
#[test]
fn flattening_does_not_corrupt_the_checkers_node_keyed_tables() {
    let dir = scratch("nodeids");
    write(
        &dir,
        "delulu.toml",
        "[package]\nname = \"nid\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n[authority]\neffects = [\"Write\"]\n",
    );
    // Two modules, each with records, capability parameters threaded through helpers, and a loop —
    // the shapes whose types live in node-keyed tables.
    write(
        &dir,
        "src/nid.delulu",
        "module nid\n\
         import nid.util\n\
         fn main(root: Root) ! {Write} {\n\
         \x20 let out = root.console()\n\
         \x20 var i = 0\n\
         \x20 var total = 0\n\
         \x20 while i < 3 {\n\
         \x20   total = total + score(Row { id: i, label: \"r\" })\n\
         \x20   emit(out, i)\n\
         \x20   i = i + 1\n\
         \x20 }\n\
         \x20 out.println(\"total=\" + str(total))\n\
         }\n",
    );
    write(
        &dir,
        "src/util.delulu",
        "module nid.util\n\
         pub type Row { id: Int, label: Str }\n\
         pub fn score(r: Row) -> Int { r.id * 2 + r.label.len() }\n\
         pub fn emit(out: Cap[Console], n: Int) -> Unit ! {Write} { out.println(\"row \" + str(n)) }\n",
    );

    let o = delulu(&dir, &["run", ".", "--grant", "console", "--no-prompt"]);
    let all = stdout(&o) + &stderr(&o);
    assert!(
        !all.contains("DL1603") && !all.contains("DL1604"),
        "flattening must not invent a reference-capability error — this is the NodeId-overlap \
         regression: {all}"
    );
    assert!(o.status.success(), "the program must run: {all}");
    // id*2 + len("r") for i in 0..3  =  1 + 3 + 5 = 9
    assert!(stdout(&o).contains("total=9"), "and compute the right answer: {}", stdout(&o));
}
