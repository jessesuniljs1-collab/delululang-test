//! PS-A-07: `delulu run --sandbox`, its profiles, its limits, and its refusals.
//!
//! The one thing a sandbox flag must never do is quietly not apply, so most of this file is about
//! refusals: an unknown profile, an unreadable limit, a value the flag does not know, and a program
//! whose surface the channel cannot carry yet. Each must refuse with a reason and run nothing.

use std::process::{Command, Output};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary runs")
}

fn tmp(name: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-sbx-{name}-{}-{t}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("out")).expect("a temp directory");
    d
}

/// A program that writes one file into `dir/out`, named by the absolute path the host granted.
fn writer(dir: &std::path::Path) -> (std::path::PathBuf, String) {
    let scope = dir.join("out").display().to_string().replace('\\', "/");
    let src = dir.join("p.delulu");
    std::fs::write(
        &src,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{scope}\")\n    \
             let _ = w.write_text(\"made.txt\", \"sandboxed\")\n}}\n"
        ),
    )
    .unwrap();
    (src, scope)
}

/// The whole point, end to end: the program runs jailed, holds nothing, and the file appears because
/// the HOST wrote it inside the granted scope.
#[test]
fn a_sandboxed_run_works_and_reports_the_policy_that_was_applied() {
    let dir = tmp("run");
    let (src, scope) = writer(&dir);
    let report = dir.join("rep.json");
    let o = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        report.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(std::fs::read_to_string(dir.join("out").join("made.txt")).unwrap(), "sandboxed");

    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let sb = &v["sandbox"];
    assert_eq!(sb["requested"], "contained");
    assert_eq!(sb["mode"], "strict");
    assert_eq!(sb["break_glass"], false);
    assert!(sb["policy_hash"].as_str().is_some_and(|h| h.len() == 64), "a policy hash names the policy: {sb}");
    // What the host actually applied, never what the profile hoped for.
    let guarantees = sb["host_guarantees"].as_array().expect("guarantees are a list");
    if cfg!(windows) || cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        assert!(!guarantees.is_empty(), "this host has a jail, so the report must name it: {sb}");
        assert_eq!(sb["level"], 1);
        assert_eq!(sb["backend"], "process");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A tighter profile really is tighter, and the report says which one was asked for.
#[test]
fn a_profile_chooses_the_limits_and_the_report_names_it() {
    let dir = tmp("profile");
    let (src, scope) = writer(&dir);
    let report = dir.join("rep.json");
    let o = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--sandbox-profile",
        "hostile-agent",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        report.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["sandbox"]["requested"], "hostile-agent");
    assert_eq!(v["sandbox"]["limits"]["memory_bytes"], 256 * 1024 * 1024_u64);
    assert_eq!(v["sandbox"]["limits"]["cpu_seconds"], 60);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--limits` may narrow a profile and never widen it: choosing a tight profile cannot be undone by
/// a flag further along the same command line.
#[test]
fn limits_narrow_a_profile_and_never_widen_it() {
    let dir = tmp("limits");
    let (src, scope) = writer(&dir);
    let report = dir.join("rep.json");
    let o = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--sandbox-profile",
        "hostile-agent",
        // 8 GiB and an hour: far wider than `hostile-agent`, and both must be ignored.
        "--limits",
        "mem=8589934592,cpu=3600",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        report.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["sandbox"]["limits"]["memory_bytes"], 256 * 1024 * 1024_u64, "a flag must not widen a profile");
    assert_eq!(v["sandbox"]["limits"]["cpu_seconds"], 60);

    // Narrowing, on the other hand, is exactly what the flag is for.
    let o = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--limits",
        "cpu=7",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        report.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["sandbox"]["limits"]["cpu_seconds"], 7);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Every refusal: nothing runs, and the reason says what to do instead.
#[test]
fn a_sandbox_that_cannot_apply_refuses_instead_of_running_unconfined() {
    let dir = tmp("refuse");
    let (src, _) = writer(&dir);
    let file = src.to_str().unwrap();

    for (args, expect) in [
        (vec!["run", file, "--sandbox", "--sandbox-profile", "nonsense"], "is not a sandbox profile"),
        (vec!["run", file, "--sandbox=maybe"], "is not a value this command knows"),
        (vec!["run", file, "--sandbox", "--limits", "mem=abc"], "is not a number"),
        (vec!["run", file, "--sandbox", "--limits", "disk=1"], "is not a limit this command knows"),
    ] {
        let o = delulu(&args);
        assert_eq!(o.status.code(), Some(2), "`{}` must refuse: {}", args.join(" "), String::from_utf8_lossy(&o.stderr));
        assert!(String::from_utf8_lossy(&o.stderr).contains(expect), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(!dir.join("out").join("made.txt").exists(), "nothing ran");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A surface the channel cannot carry is refused, not run unconfined. Actors are the case that
/// matters most here: they are a language feature rather than a capability, so a gate that read only
/// capability kinds let them through — which is the "quietly did not apply" failure itself.
#[test]
fn a_program_the_channel_cannot_carry_is_refused() {
    let dir = tmp("surface");
    for (name, src) in [
        (
            "actors",
            "module a\n\nactor C {\n    var n: Int\n    new() { self.n = 0 }\n    be tick() { self.n = self.n + 1 }\n}\n\nfn main(root: Root) {\n    let c = C()\n    c.tick()\n}\n",
        ),
        ("foreign code", "module f\n\nforeign \"c\" lib m {\n    fn abs(x: Int) -> Int\n}\n\nfn main(root: Root) {\n}\n"),
    ] {
        let p = dir.join(format!("{}.delulu", name.replace(' ', "_")));
        std::fs::write(&p, src).unwrap();
        let o = delulu(&["run", p.to_str().unwrap(), "--sandbox"]);
        let err = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(2), "{name} must be refused: {err}");
        assert!(err.contains("cannot carry this program yet"), "{err}");
        assert!(err.contains(name), "the refusal must name the surface: {err}");
        assert!(err.contains("--sandbox=off"), "the refusal must say what to do instead: {err}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The audit dry run (PS-A-10): it performs NOTHING and reports what a real run would need — the
/// policy that would hold, the grants the program would want, and any surface the channel cannot
/// carry. This is what an agent reads before letting unfamiliar code run at all.
#[test]
fn an_audit_run_performs_nothing_and_reports_what_a_run_would_need() {
    let dir = tmp("audit");
    let (src, scope) = writer(&dir);
    let o = delulu(&["run", src.to_str().unwrap(), "--sandbox", "--mode", "audit"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!dir.join("out").join("made.txt").exists(), "an audit run performs nothing");

    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).expect("one report");
    assert_eq!(v["sandbox"]["mode"], "audit");
    assert_eq!(v["outcome"]["ran"], false);
    let grants = v["audit"]["required_grants"].as_array().expect("the grants a run would need");
    assert!(
        grants.iter().any(|g| g.as_str().is_some_and(|g| g.starts_with("fs.write=") && g.contains(scope.trim_end_matches('/')))),
        "the audit names the grant this program needs: {grants:?}"
    );
    assert_eq!(v["audit"]["unsupported_surface"], serde_json::Value::Null, "this program is carried");

    // And on a program the channel cannot carry, the audit says so rather than refusing: telling a
    // caller what is missing is the whole job of a dry run.
    let act = dir.join("a.delulu");
    std::fs::write(
        &act,
        "module a\n\nactor C {\n    var n: Int\n    new() { self.n = 0 }\n    be tick() { self.n = self.n + 1 }\n}\n\nfn main(root: Root) {\n    let c = C()\n    c.tick()\n}\n",
    )
    .unwrap();
    let o = delulu(&["run", act.to_str().unwrap(), "--sandbox", "--mode", "audit"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).unwrap();
    assert_eq!(v["audit"]["unsupported_surface"], "actors");
    assert_eq!(v["outcome"]["ran"], false);

    let o = delulu(&["run", src.to_str().unwrap(), "--sandbox", "--mode", "loud"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("is not a mode this command knows"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--sandbox=off` is the explicit opposite, and it still runs the program the ordinary way.
#[test]
fn sandbox_off_runs_the_program_unconfined_and_says_nothing_false() {
    let dir = tmp("off");
    let (src, scope) = writer(&dir);
    let o = delulu(&["run", src.to_str().unwrap(), "--sandbox=off", "--grant", &format!("fs.write={scope}")]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(std::fs::read_to_string(dir.join("out").join("made.txt")).unwrap(), "sandboxed");
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains("confined"),
        "an unconfined run must not claim confinement: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
