//! Stage 7 phase 7a — invariant 37 / acceptance criterion 10: `consume`/`recover` became
//! keywords in v0.7 (an acknowledged Stage-1 reserved-list omission). Pre-0.7 code using them
//! as identifiers gets DL1608 with an exact rename repair, and `delulu fmt --migrate 0.7`
//! applies it corpus-wide: identifiers (declaration, use, and member position) are renamed to
//! `consume_`/`recover_`; strings and comments are untouched because the migrator is
//! token-stream based and they are not identifier tokens.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run delulu")
}

/// House-style scratch dir (see tooling.rs/provenance.rs): fresh per test name + pid.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_fmt_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn combined(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

const PRE_07: &str = "module legacy\n\
\n\
// consume the queue eagerly (this comment must survive untouched)\n\
fn consume(n: Int) -> Int { n }\n\
\n\
fn recover(n: Int) -> Int { n }\n\
\n\
fn main() {\n\
  let a = consume(1)\n\
  let b = recover(2)\n\
  let s = \"please consume responsibly\"\n\
}\n";

#[test]
fn criterion10_dl1608_then_migrate_then_clean() {
    let dir = scratch("criterion10");
    let file = dir.join("legacy.delulu");
    std::fs::write(&file, PRE_07).unwrap();

    // 1. Pre-migration: the pre-0.7 corpus fails with DL1608 (never a confusing cascade).
    let before = delulu_in(&dir, &["check", file.to_str().unwrap()]);
    let out = combined(&before);
    assert!(out.contains("DL1608"), "expected DL1608 before migration, got:\n{out}");

    // 2. The migration applies corpus-wide (directory argument).
    let mig = delulu_in(&dir, &["fmt", "--migrate", "0.7", dir.to_str().unwrap()]);
    assert!(mig.status.success(), "migrate failed: {}", combined(&mig));
    let migrated = std::fs::read_to_string(&file).unwrap();

    // Identifiers renamed at declaration, call, and use sites.
    assert!(migrated.contains("fn consume_(n: Int)"), "{migrated}");
    assert!(migrated.contains("fn recover_(n: Int)"), "{migrated}");
    assert!(migrated.contains("consume_(1)"), "{migrated}");
    assert!(migrated.contains("recover_(2)"), "{migrated}");
    // Strings and comments untouched.
    assert!(migrated.contains("// consume the queue eagerly"), "{migrated}");
    assert!(migrated.contains("\"please consume responsibly\""), "{migrated}");

    // 3. Post-migration: the corpus checks clean.
    let after = delulu_in(&dir, &["check", file.to_str().unwrap()]);
    let out = combined(&after);
    assert!(after.status.success(), "post-migration check failed:\n{out}");
    assert!(!out.contains("DL1608"), "DL1608 must be gone after migration:\n{out}");

    // 4. Running the migration again finds nothing to do (renamed identifiers are ordinary).
    let again = delulu_in(&dir, &["fmt", "--migrate", "0.7", dir.to_str().unwrap()]);
    assert!(again.status.success());
    assert!(combined(&again).contains("nothing to do"), "{}", combined(&again));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrate_refuses_unknown_versions() {
    let dir = scratch("badver");
    let out = delulu_in(&dir, &["fmt", "--migrate", "0.9", dir.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(combined(&out).contains("unknown migration"), "{}", combined(&out));
    let _ = std::fs::remove_dir_all(&dir);
}
