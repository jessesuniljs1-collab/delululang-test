//! `delulu new` — the first five minutes.
//!
//! A scaffold is only worth having if what it produces genuinely works and what it *says* is
//! genuinely true. Both halves are tested here, and the second half is the one that caught a real
//! defect: the printed next steps said `delulu test`, which refuses without a `./tests` directory.
//! The working form is `delulu test .`. Instructions a new user follows first must not be wrong.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-new-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn delulu_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary must run")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

/// The whole promise: what it writes checks, tests and runs, with no editing.
#[test]
fn a_new_binary_package_checks_tests_and_runs() {
    let w = scratch("bin");
    let o = delulu_in(&w, &["new", "myapp"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));

    let pkg = w.join("myapp");
    assert!(pkg.join("delulu.toml").is_file(), "manifest written");
    assert!(pkg.join("src").join("main.delulu").is_file(), "entry module written");
    assert!(pkg.join(".gitignore").is_file(), "build output is ignored from the start");

    let chk = delulu_in(&pkg, &["check", "."]);
    assert_eq!(chk.status.code(), Some(0), "it checks clean:\n{}", stderr(&chk));

    let tst = delulu_in(&pkg, &["test", "."]);
    assert_eq!(tst.status.code(), Some(0), "its test passes:\n{}", stderr(&tst));
    let reported = format!("{}{}", stdout(&tst), stderr(&tst));
    assert!(reported.contains("1 passed"), "and is counted:\n{reported}");

    let run = delulu_in(&pkg, &["run", ".", "--grant", "console"]);
    assert_eq!(run.status.code(), Some(0), "it runs:\n{}", stderr(&run));
    assert!(stdout(&run).contains("hello, world"), "and prints: {}", stdout(&run));
}

/// **Every command the scaffold prints is executed here, and every command executed is printed.**
///
/// This binding is the point. A next-steps block is documentation that ships inside a binary,
/// which means nothing else in the repository proof-reads it — and the first draft of it told
/// people to run `delulu test`, which fails without a `./tests` directory. A scaffold whose very
/// first instruction does not work is worse than no scaffold, because it costs trust rather than
/// time.
#[test]
fn the_generated_package_does_what_the_message_promises() {
    let w = scratch("promise");
    let created = delulu_in(&w, &["new", "myapp"]);
    let message = stderr(&created);
    let pkg = w.join("myapp");

    let steps: [&[&str]; 4] = [
        &["check", "."],
        &["test", "."],
        &["authority", "."],
        &["run", ".", "--grant", "console"],
    ];
    for step in steps {
        let printed = format!("delulu {}", step.join(" "));
        assert!(message.contains(&printed), "the message must name `{printed}`:\n{message}");
        let o = delulu_in(&pkg, step);
        assert_eq!(o.status.code(), Some(0), "`{printed}` must actually work:\n{}", stderr(&o));
    }

    // And the claim it makes about the authority is the authority the tool reports.
    assert!(message.contains("{Write}"), "the message states the ceiling:\n{message}");
    let auth = delulu_in(&pkg, &["authority", "."]);
    assert!(stdout(&auth).contains("Write"), "and the tool agrees:\n{}", stdout(&auth));
}

/// The lesson the scaffold exists to teach must actually hold.
#[test]
fn without_its_grant_the_generated_program_stops() {
    let w = scratch("nogrant");
    let created = delulu_in(&w, &["new", "myapp"]);
    assert!(stderr(&created).contains("DL0703"), "the message names the code:\n{}", stderr(&created));

    let pkg = w.join("myapp");
    let o = delulu_in(&pkg, &["run", ".", "--no-prompt"]);
    assert_ne!(o.status.code(), Some(0), "no grant, no run");
    let all = format!("{}{}", stdout(&o), stderr(&o));
    assert!(all.contains("DL0703"), "and it is that code, not some other failure:\n{all}");
}

/// A library scaffold declares — and is proved to have — no authority at all.
#[test]
fn a_new_library_is_provably_pure() {
    let w = scratch("lib");
    let o = delulu_in(&w, &["new", "mylib", "--lib"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));

    let pkg = w.join("mylib");
    assert!(pkg.join("src").join("mylib.delulu").is_file(), "a lib's entry is named for it");
    assert!(!pkg.join("src").join("main.delulu").exists(), "and has no main");

    let chk = delulu_in(&pkg, &["check", "."]);
    assert_eq!(chk.status.code(), Some(0), "{}", stderr(&chk));
    let tst = delulu_in(&pkg, &["test", "."]);
    assert_eq!(tst.status.code(), Some(0), "{}", stderr(&tst));

    let auth = delulu_in(&pkg, &["authority", "."]);
    assert!(
        stdout(&auth).contains("provably pure"),
        "a library that computes should hold nothing:\n{}",
        stdout(&auth)
    );
}

/// **The declared ceiling is exactly what the code does — no more.**
///
/// This is the policy test, and it is the one worth keeping if every other test here were deleted.
/// A scaffold that granted `Read`, `Write` and `Net` "to save you time" would teach the opposite
/// of the thing this language teaches, once per project, forever. It is also the most tempting
/// convenience to add later, which is why the number is pinned rather than merely produced.
#[test]
fn the_declared_ceiling_is_exactly_what_the_code_does() {
    let w = scratch("ceiling");
    delulu_in(&w, &["new", "myapp"]);
    let manifest = read(&w.join("myapp").join("delulu.toml"));
    assert!(manifest.contains("effects  = [\"Write\"]"), "one effect, the one it uses:\n{manifest}");
    for wider in ["\"Read\"", "\"Net\"", "\"Clock\"", "\"Rand\"", "\"ForeignCall\"", "\"Actuate\""] {
        assert!(!manifest.contains(wider), "the ceiling must not include {wider}:\n{manifest}");
    }
    // A *table*, not the words. The manifest's comments name `[test-authority]` in order to
    // explain why there isn't one, so a substring search here would fail on prose that is doing
    // exactly the right thing.
    assert!(
        !manifest.lines().any(|l| l.trim_start().starts_with("[test-authority]")),
        "no test ceiling: an absent table grants nothing, and the generated test needs nothing:\n{manifest}"
    );

    delulu_in(&w, &["new", "mylib", "--lib"]);
    let libm = read(&w.join("mylib").join("delulu.toml"));
    assert!(libm.contains("effects = []"), "a library starts at nothing:\n{libm}");
}

/// A name that could not be a module name is refused before anything is written.
///
/// The package name becomes `module <name>` in the generated source, so accepting `my-app` would
/// hand someone a brand-new package that does not parse — a far worse first experience than being
/// told the name is not usable, with the usable one suggested.
#[test]
fn a_name_that_cannot_be_a_module_is_refused() {
    let w = scratch("names");
    for bad in ["my-app", "9lives", "fn", "my.app"] {
        let o = delulu_in(&w, &["new", bad]);
        assert_eq!(o.status.code(), Some(2), "`{bad}` must be refused:\n{}", stderr(&o));
        assert!(!w.join(bad).exists(), "and nothing written for `{bad}`");
    }
    // Where there is an obvious fix, it is offered rather than left as an exercise.
    let o = delulu_in(&w, &["new", "my-app"]);
    assert!(stderr(&o).contains("my_app"), "suggest the legal name:\n{}", stderr(&o));
}

/// It never writes into a directory that already holds something.
#[test]
fn it_never_writes_into_a_non_empty_directory() {
    let w = scratch("occupied");
    let pkg = w.join("myapp");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    let precious = pkg.join("src").join("main.delulu");
    std::fs::write(&precious, "module myapp\n// a whole afternoon's work\n").unwrap();

    let o = delulu_in(&w, &["new", "myapp"]);
    assert_eq!(o.status.code(), Some(2), "it refuses:\n{}", stderr(&o));
    assert!(stderr(&o).contains("not empty"), "{}", stderr(&o));
    assert_eq!(
        read(&precious),
        "module myapp\n// a whole afternoon's work\n",
        "and does not touch what was there"
    );
    assert!(!pkg.join("delulu.toml").exists(), "nor add to it");
}

/// One JSON object, describing what was written.
#[test]
fn the_json_envelope_holds_one_object() {
    let w = scratch("json");
    let o = delulu_in(&w, &["new", "myapp", "--json"]);
    let out = stdout(&o);
    let mut stream = serde_json::Deserializer::from_str(&out).into_iter::<serde_json::Value>();
    let v = stream.next().expect("a JSON value").expect("valid JSON");
    assert!(stream.next().is_none(), "exactly one:\n{out}");

    assert_eq!(v["command"], "new");
    assert_eq!(v["schema"], 1);
    assert_eq!(v["new"]["name"], "myapp");
    assert_eq!(v["new"]["kind"], "bin");
    assert_eq!(v["new"]["entry"], "src/main.delulu");
    assert_eq!(v["new"]["authority"][0], "Write");
    let files = v["new"]["files"].as_array().expect("files listed");
    assert_eq!(files.len(), 3, "{files:?}");
}

/// A path with directories in it names the directory; the last component names the package.
#[test]
fn a_nested_path_names_the_directory_and_the_package() {
    let w = scratch("nested");
    let o = delulu_in(&w, &["new", "apps/inner"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let pkg = w.join("apps").join("inner");
    assert!(pkg.join("delulu.toml").is_file(), "created under the path given");
    assert!(read(&pkg.join("delulu.toml")).contains("name = \"inner\""), "named for the last part");
    assert!(read(&pkg.join("src").join("main.delulu")).starts_with("module inner"), "and so is the module");
    let chk = delulu_in(&pkg, &["check", "."]);
    assert_eq!(chk.status.code(), Some(0), "{}", stderr(&chk));
}

/// The negative half of the CLI contract: an unknown option is refused, not ignored.
#[test]
fn new_refuses_an_option_it_does_not_know() {
    let w = scratch("bad");
    let o = delulu_in(&w, &["new", "myapp", "--with-everything"]);
    assert_eq!(o.status.code(), Some(2), "a bad invocation exits 2");
    assert!(!w.join("myapp").exists(), "and creates nothing");
    assert!(stderr(&o).contains("unknown option"), "{}", stderr(&o));

    let o2 = delulu_in(&w, &["new"]);
    assert_eq!(o2.status.code(), Some(2), "no name exits 2");

    let o3 = delulu_in(&w, &["new", "a", "b"]);
    assert_eq!(o3.status.code(), Some(2), "two names exits 2");
    assert!(!w.join("a").exists() && !w.join("b").exists(), "and creates neither");
}

/// `--help` answers the question and creates nothing (the D11 rule).
#[test]
fn help_creates_nothing() {
    let w = scratch("help");
    let o = delulu_in(&w, &["new", "--help"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(std::fs::read_dir(&w).unwrap().count(), 0, "asking must not create a package");
}

/// **A name that only works on your platform is not a package name.**
///
/// `delulu new con` succeeded on Linux and macOS and produced a directory Windows can never check
/// out — `git clone` fails on the directory itself, so the author would not find out until a
/// colleague did. On Windows it failed already, but with whichever raw OS error came first: `The
/// parameter is incorrect. (os error 87)` for `con`, `The system cannot find the file specified.
/// (os error 2)` for `aux`. **Two meaningless texts for one cause**, which is the shape D33 refused
/// once already for `run <dir>`.
///
/// Checked on every platform deliberately: the point is to stop a Unix author creating a package
/// that is broken for everyone else (C72, ruling D72).
#[test]
fn a_name_windows_reserves_as_a_device_is_refused_on_every_platform() {
    let home = scratch("reserved-device");
    for name in ["con", "aux", "nul", "prn", "com1", "lpt9", "CON", "Aux"] {
        let o = delulu_in(&home, &["new", name]);
        let out = format!("{}{}", stdout(&o), stderr(&o));
        assert!(
            !o.status.success(),
            "`delulu new {name}` must be refused — Windows cannot host that directory: {out}"
        );
        assert!(
            out.contains("device name"),
            "the refusal must say WHY, not leak an OS error: {out}"
        );
        assert!(
            !home.join(name).exists(),
            "nothing may be created for a name that cannot exist on a supported platform"
        );
    }
}

/// THE SKIP-BRANCH CASE: ordinary names that merely *contain* a reserved word must still work, or
/// the check would outlaw `console`, `context` and `nullable`.
#[test]
fn a_name_that_merely_contains_a_device_word_is_still_fine() {
    let home = scratch("reserved-device-ok");
    for name in ["console", "context", "connection", "auxiliary", "nullable", "computer"] {
        let o = delulu_in(&home, &["new", name]);
        assert!(o.status.success(), "`{name}` is a perfectly good package name: {}", stderr(&o));
    }
}

/// NE-09. `delulu new hello && cd hello && delulu test` exited **2** — "needs test
/// files/directories (no ./tests directory here)" — although the scaffold's one test lives in
/// `src/main.delulu` and `delulu test .` passed. The trap was on record here and the printed
/// instructions were fixed rather than the default, so the tool went on disagreeing with every
/// other package tool a person arrives with.
///
/// Inside a package, bare `delulu test` now targets the package. Outside one it still refuses,
/// because there is nothing to infer and guessing a directory would be worse than asking.
#[test]
fn bare_delulu_test_passes_inside_a_freshly_scaffolded_package() {
    let w = scratch("baretest");
    let created = delulu_in(&w, &["new", "myapp"]);
    assert_eq!(created.status.code(), Some(0), "{}", stderr(&created));
    let pkg = w.join("myapp");

    let bare = delulu_in(&pkg, &["test"]);
    assert_eq!(
        bare.status.code(),
        Some(0),
        "bare `delulu test` must pass in a scaffold:\n{}{}",
        stdout(&bare),
        stderr(&bare)
    );
    let reported = format!("{}{}", stdout(&bare), stderr(&bare));
    assert!(reported.contains("1 passed"), "and must actually RUN the test:\n{reported}");

    // The same answer as the explicit form, so the default is the package and not a narrower guess.
    let explicit = delulu_in(&pkg, &["test", "."]);
    assert_eq!(
        stdout(&bare).trim(),
        stdout(&explicit).trim(),
        "bare `delulu test` and `delulu test .` must agree inside a package"
    );

    // A package that keeps its tests in `tests/` still runs them, and also runs the ones in `src/`:
    // the package is the target, so this is a superset of what `tests/` alone produced.
    std::fs::create_dir_all(pkg.join("tests")).unwrap();
    std::fs::write(
        pkg.join("tests").join("extra.delulu"),
        "module extra\n\ntest \"extra runs too\" {\n    assert_eq(str(1), str(1))\n}\n",
    )
    .unwrap();
    let both = delulu_in(&pkg, &["test"]);
    assert_eq!(both.status.code(), Some(0), "{}", stderr(&both));
    let both_out = format!("{}{}", stdout(&both), stderr(&both));
    assert!(both_out.contains("2 passed"), "both the src/ and tests/ tests ran:\n{both_out}");

    // Outside a package the refusal is unchanged: exit 2, and it says what to do.
    let outside = scratch("baretest-outside");
    let refused = delulu_in(&outside, &["test"]);
    assert_eq!(refused.status.code(), Some(2), "outside a package the refusal stays");
    let err = stderr(&refused);
    assert!(err.contains("needs test files/directories"), "and says so: {err}");
}
