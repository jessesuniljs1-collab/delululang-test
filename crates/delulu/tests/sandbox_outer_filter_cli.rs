//! PS-E-05 (b), D-V2-83: a guest under an OUTER wall that forbids adding a syscall filter — the wall NVIDIA
//! OpenShell's sandbox is. Routine run 8 read it on a runner (`openshell.yml`'s runtime job): inside
//! OpenShell's sandbox the guest's own Landlock layer took hold, and its `seccomp` filter was refused with
//! EPERM — so the guest failed closed, the host never sent the program, and nothing ran.
//!
//! The outer wall is simulated here as it looks from inside: before `delulu run` starts, the test installs
//! on it a filter answering `seccomp` with an errno, which the launcher and the guest inherit — a filter in
//! force, and a refusal to add another. What must hold:
//! - by default the guest still fails closed: an outer filter stands in for its own only when its LAUNCHER
//!   declares the outer wall (`__guest --stdio-pipes --outer-syscall-filter`), never on its own inference;
//! - declared, it stands in only when the refusal is EPERM — an outer filter's answer — and never skips a
//!   filter the guest CAN install;
//! - the guest's words say so ("an outer syscall filter, not its own", in place of its filter's four), the
//!   report keeps them as the guest's word at level 3, and authority is unchanged: the host still decides
//!   and performs every effect under the grants;
//! - the flag belongs to an external launcher's guest alone, and on a guest that applies no filter of its
//!   own (macOS, Windows) it means nothing and is refused.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLAG: &str = "--outer-syscall-filter";
#[cfg(target_os = "linux")]
const WORD: &str = "an outer syscall filter, not its own";
/// The guest's own filter, in its words — what the outer one stands in for.
#[cfg(target_os = "linux")]
const OWN_FILTER: [&str; 4] = ["no new programs", "no debugger", "no namespace or module tricks", "no sockets but the channel"];

/// `delulu` with `args`; with `outer`, under a filter that answers `seccomp` with that errno — installed on
/// the host before it starts, so everything it launches inherits it, as a process inside OpenShell does.
fn delulu(cwd: &Path, args: &[&str], outer: Option<i32>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_delulu"));
    cmd.current_dir(cwd)
        .env("DELULU_STATE_DIR", cwd.join("s"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(args);
    #[cfg(target_os = "linux")]
    if let Some(errno) = outer {
        use std::os::unix::process::CommandExt as _;
        // Built before the fork: after it, the child only installs it (no allocation there).
        let program = outer_filter(errno);
        // SAFETY: `apply_filter` makes two system calls (`prctl`, `seccomp`) on a program built before the fork.
        unsafe {
            cmd.pre_exec(move || {
                seccompiler::apply_filter(&program).map_err(|_| std::io::Error::from(std::io::ErrorKind::PermissionDenied))
            });
        }
    }
    #[cfg(not(target_os = "linux"))]
    assert!(outer.is_none(), "an outer filter is a Linux wall");
    cmd.output().expect("the binary runs")
}

#[cfg(target_os = "linux")]
fn outer_filter(errno: i32) -> seccompiler::BpfProgram {
    use seccompiler::{SeccompAction, SeccompFilter};
    let rules = [(libc::SYS_seccomp, Vec::new())].into_iter().collect();
    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(errno as u32),
        std::env::consts::ARCH.try_into().expect("a seccomp architecture"),
    )
    .expect("a filter");
    filter.try_into().expect("compiled")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn lab(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("douter-{tag}-{}-{}-{n}", std::process::id(), t % 1_000_000_000));
    std::fs::create_dir_all(d.join("s")).unwrap();
    std::fs::write(
        d.join("h.delulu"),
        "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hello from the guest\")\n}\n",
    )
    .unwrap();
    d
}

/// The binary as the launcher, with the guest's words and `extra` after them.
#[cfg(target_os = "linux")]
fn launcher(extra: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    assert!(!exe.contains(' '), "the launcher is split on whitespace; this checkout's path has a space: {exe}");
    format!("external:{exe} __guest --stdio-pipes {extra}").trim_end().to_string()
}

/// The run's words and its report's `sandbox` object.
#[cfg(target_os = "linux")]
fn run(d: &Path, extra: &str, outer: Option<i32>, grant: bool) -> (Output, serde_json::Value) {
    let report = d.join("r.json");
    let _ = std::fs::remove_file(&report);
    let l = launcher(extra);
    let mut args = vec!["run", "h.delulu", "--sandbox", "--sandbox-backend", &l, "--report-out", report.to_str().unwrap()];
    if grant {
        args.extend(["--grant", "console"]);
    }
    let r = delulu(d, &args, outer);
    let v = std::fs::read_to_string(&report)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .map(|v| v["sandbox"].clone())
        .unwrap_or(serde_json::Value::Null);
    (r, v)
}

#[cfg(target_os = "linux")]
fn words(s: &serde_json::Value) -> Vec<String> {
    s["guest_reported"].as_array().map(|a| a.iter().map(|w| w.as_str().unwrap().to_string()).collect()).unwrap_or_default()
}

/// OpenShell's case, reproduced: under an outer filter that refuses another with EPERM, the guest fails
/// closed unless its launcher declares the outer wall — and declared, the program runs, the guest's words
/// say whose filter is in force, and the host still decides every effect.
#[cfg(target_os = "linux")]
#[test]
fn under_an_outer_filter_the_guest_runs_only_when_its_launcher_declares_it() {
    let d = lab("declared");
    // The baseline, as routine run 8 read it inside OpenShell: undeclared, the guest refuses to run.
    let (r, _) = run(&d, "", Some(libc::EPERM), true);
    assert_ne!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("could not lock itself down"), "the guest says why it refused: {}", text(&r));
    assert!(text(&r).contains("never confirmed its boundary"), "and the host sent nothing: {}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("hello from the guest"), "nothing ran: {}", text(&r));

    // Declared: it runs, at level 3, and says the filter in force is the outer wall's, not its own.
    let (r, s) = run(&d, FLAG, Some(libc::EPERM), true);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(String::from_utf8_lossy(&r.stdout).contains("hello from the guest"), "{}", text(&r));
    assert!(text(&r).contains("refused by a filter already in force"), "the guest says so on the operator's screen: {}", text(&r));
    assert_eq!(s["level"], 3, "{s}");
    assert_eq!(s["backend"], "external", "{s}");
    assert_eq!(s["host_guarantees"].as_array().map(Vec::len), Some(0), "no HOST guarantee is claimed: {s}");
    let w = words(&s);
    assert!(w.iter().any(|x| x == WORD), "the guest's words name the outer filter: {s}");
    for own in OWN_FILTER {
        assert!(!w.iter().any(|x| x == own), "`{own}` is its own filter's word, and its own filter is not in force: {s}");
    }
    for (name, p) in s["properties"].as_object().unwrap_or_else(|| panic!("properties: {s}")) {
        assert_eq!(p["state"], "unknown", "{name}: {s}");
    }

    // Authority is unchanged: an effect the run did not grant is refused by the host, and nothing happens.
    let (r, _) = run(&d, FLAG, Some(libc::EPERM), false);
    assert_ne!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("DL0703"), "an ungranted effect is refused by the host: {}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("hello from the guest"), "{}", text(&r));
    let _ = std::fs::remove_dir_all(&d);
}

/// Only EPERM — an outer filter's refusal — lets the outer filter stand in. A refusal with another answer
/// is not one the guest can attribute to a wall, and it fails closed, declared or not.
#[cfg(target_os = "linux")]
#[test]
fn a_refusal_that_is_not_eperm_fails_closed_even_when_declared() {
    let d = lab("errno");
    for errno in [libc::ENOSYS, libc::EACCES, libc::EINVAL] {
        let (r, _) = run(&d, FLAG, Some(errno), true);
        assert_ne!(r.status.code(), Some(0), "errno {errno}: {}", text(&r));
        assert!(text(&r).contains("could not lock itself down"), "errno {errno}: {}", text(&r));
        assert!(!String::from_utf8_lossy(&r.stdout).contains("hello from the guest"), "errno {errno}: nothing ran: {}", text(&r));
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// The declaration never skips a filter the guest can install: with no outer wall, a declared guest locks
/// itself down exactly as an undeclared one does.
#[cfg(target_os = "linux")]
#[test]
fn a_declared_guest_still_installs_its_own_filter_when_it_can() {
    let d = lab("own");
    let (r, s) = run(&d, FLAG, None, true);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    let w = words(&s);
    for own in OWN_FILTER {
        assert!(w.iter().any(|x| x == own), "its own filter's `{own}`: {s}");
    }
    assert!(!w.iter().any(|x| x == WORD), "no outer filter stood in: {s}");
    let _ = std::fs::remove_dir_all(&d);
}

/// The flag is an external launcher's declaration about the wall it starts the guest in: a guest the HOST
/// starts (`--stdio`, a channel directory) never takes it, nor does a launcher's guest take anything else.
#[test]
fn the_declaration_belongs_to_an_external_launchers_guest_alone() {
    let d = lab("refused");
    for (args, says) in [
        (vec!["__guest", "--stdio", FLAG], "only a guest an external launcher starts"),
        (vec!["__guest", d.to_str().unwrap(), FLAG], "only a guest an external launcher starts"),
        (vec!["__guest", "--stdio-pipes", "--outer-filter"], "does not know `--outer-filter`"),
        (vec!["__guest", "--stdio-pipes", FLAG, FLAG], "does not know"),
    ] {
        let r = delulu(&d, &args, None);
        assert_eq!(r.status.code(), Some(2), "{args:?}: {}", text(&r));
        assert!(text(&r).contains(says), "{args:?}: {}", text(&r));
    }
    // Where the guest applies no syscall filter of its own, there is nothing for an outer one to stand in for.
    #[cfg(not(target_os = "linux"))]
    {
        let r = delulu(&d, &["__guest", "--stdio-pipes", FLAG], None);
        assert_eq!(r.status.code(), Some(2), "{}", text(&r));
        assert!(text(&r).contains("applies no syscall filter of its own"), "{}", text(&r));
    }
    let _ = std::fs::remove_dir_all(&d);
}
