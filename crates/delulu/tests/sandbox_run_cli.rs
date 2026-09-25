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
        // Never the real one: a test must not write into the developer own audit chain, which is
        // exactly how these records first reached `~/.delulu` and broke `doctor`.
        .env("DELULU_STATE_DIR", isolated_state())
        .output()
        .expect("the delulu binary runs")
}

/// Everything the command said, on either stream, for a failure message that is worth reading.
fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
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

/// PS-B-03 (T14) end to end on Windows: the guest runs as a separate identity, the program still does
/// what it was granted — because the HOST performs every effect, an identity that can reach none of
/// the operator's files costs a granted write nothing — and the report names the identity. The
/// boundary itself is measured in `identity::win::tests` (the same attempts, contained and not); this
/// is the claim the operator reads.
#[cfg(windows)]
#[test]
fn on_windows_the_guest_runs_as_a_separate_identity_and_the_report_says_so() {
    let dir = tmp("identity");
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
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    assert_eq!(o.status.code(), Some(0), "{err}");
    assert_eq!(std::fs::read_to_string(dir.join("out").join("made.txt")).unwrap(), "sandboxed", "the host wrote it");
    assert!(err.contains("a separate identity"), "the operator is told: {err}");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let guarantees = v["sandbox"]["host_guarantees"].as_array().expect("guarantees are a list");
    assert!(
        guarantees.iter().any(|g| g.as_str().is_some_and(|g| g.starts_with("a separate identity"))),
        "the report names the identity it applied: {guarantees:?}"
    );
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

/// PS-A-06: `sandbox policy` answers what a run WOULD be confined by, without running it. The
/// derivation is pure, so it must agree with the policy a real run reports, hash and all — a preview
/// that disagreed with the thing it previews would be worse than none.
#[test]
fn sandbox_policy_previews_exactly_what_a_run_would_use() {
    let dir = tmp("policy");
    let (src, scope) = writer(&dir);
    let file = src.to_str().unwrap();

    let o = delulu(&["sandbox", "policy", file, "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).expect("one envelope");
    let previewed = v["policy"].clone();
    assert_eq!(previewed["requested"], "contained");
    assert_eq!(previewed["unsupported_surface"], serde_json::Value::Null);

    // The real run's report must carry the SAME policy hash.
    let report = dir.join("rep.json");
    let o = delulu(&[
        "run",
        file,
        "--sandbox",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        report.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let run: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(
        run["sandbox"]["policy_hash"], previewed["policy_hash"],
        "the preview and the run must name the same policy"
    );

    // A program the channel cannot carry is named as such, and a bad profile is refused rather than
    // quietly previewing a different policy from the one a run would use.
    let act = dir.join("a.delulu");
    std::fs::write(&act, "module a\n\nactor C {\n    var n: Int\n    new() { self.n = 0 }\n    be tick() { self.n = self.n + 1 }\n}\n\nfn main(root: Root) {\n    let c = C()\n    c.tick()\n}\n").unwrap();
    let o = delulu(&["sandbox", "policy", act.to_str().unwrap(), "--sandbox-profile", "hostile-agent", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).unwrap();
    assert_eq!(v["policy"]["requested"], "hostile-agent");
    assert_eq!(v["policy"]["unsupported_surface"], "actors");

    let o = delulu(&["sandbox", "policy", file, "--sandbox-profile", "bogus"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("is not a sandbox profile"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// PS-A-08: a sandboxed run leaves evidence in the project's OWN audit chain — same file, same
/// hashes, verified by the same command. A sandbox with a private log would be evidence nobody
/// checks, and a chain the sandbox broke would be worse than no record at all.
#[test]
fn a_sandboxed_run_is_recorded_in_the_audit_chain_and_the_chain_still_verifies() {
    let dir = tmp("audit-chain");
    let state = dir.join("state");
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let (src, scope) = writer(&dir);

    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(args)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .env("DELULU_STATE_DIR", &state)
            .output()
            .expect("the delulu binary runs")
    };

    let o = run(&["run", src.to_str().unwrap(), "--sandbox", "--grant", &format!("fs.write={scope}")]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));

    let o = run(&["audit", "query", "--json"]);
    let text = String::from_utf8_lossy(&o.stdout);
    assert!(text.contains("sandbox-launch"), "the launch is recorded: {text}");
    assert!(text.contains("sandbox-death"), "and so is the end of the guest: {text}");
    assert!(text.contains("policy_hash"), "the record names the policy that was in force: {text}");

    // The chain must still verify: these records are part of it, not beside it.
    let o = run(&["audit", "verify"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "the chain must still verify:\n{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );

    // And the records are numbered, each one after the last. `audit verify` checks the HASH chain
    // and not the numbering, so verifying alone would pass even if every record claimed seq 1 —
    // found by falsifying exactly that. Without this assertion the numbering had no gate at all.
    let seqs: Vec<u64> = text
        .lines()
        .filter(|l| l.contains("sandbox-"))
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|v| v["seq"].as_u64())
        .collect();
    let seqs = if seqs.is_empty() {
        // The query may answer as one document rather than one record per line.
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["records"].as_array().cloned())
            .map(|rs| {
                rs.iter()
                    .filter(|r| r["action"].as_str().is_some_and(|a| a.starts_with("sandbox-")))
                    .filter_map(|r| r["seq"].as_u64())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        seqs
    };
    assert!(seqs.len() >= 2, "both sandbox records carry a sequence number: {text}");
    assert!(seqs.windows(2).all(|w| w[1] > w[0]), "each record is numbered after the last: {seqs:?}");
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

/// A state directory of this test binary's own, so a run never touches the developer's real one.
/// The audit chain is single-writer: parallel test processes sharing `~/.delulu` corrupted it, which
/// is how the append lock in `guest.rs` came to exist.
fn isolated_state() -> std::path::PathBuf {
    static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("delulu-teststate-{}-{t}", std::process::id()));
        let _ = std::fs::create_dir_all(d.join("audit"));
        d
    })
    .clone()
}

/// PS-A-07's `denied[]`: the report says what the program TRIED and was refused, not only what it was
/// allowed. The pair is the whole test — a run that was refused something must list it, and a run that
/// was refused nothing must list nothing, or the field would be noise rather than evidence.
#[test]
fn the_report_lists_what_the_host_refused_and_stays_empty_when_it_refused_nothing() {
    let dir = tmp("denied");
    let scope = dir.join("out").display().to_string().replace('\\', "/");
    let sibling = dir.join("elsewhere").display().to_string().replace('\\', "/");
    std::fs::create_dir_all(dir.join("elsewhere")).unwrap();

    // Refused: the program reaches for a directory the operator never granted.
    let bad = dir.join("bad.delulu");
    std::fs::write(
        &bad,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{sibling}\")\n    \
             let _ = w.write_text(\"no.txt\", \"x\")\n}}\n"
        ),
    )
    .unwrap();
    let rep = dir.join("bad.json");
    let o = delulu(&[
        "run",
        bad.to_str().unwrap(),
        "--sandbox",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        rep.to_str().unwrap(),
    ]);
    assert_ne!(o.status.code(), Some(0), "{}", out(&o));
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&rep).expect("the run report")).unwrap();
    let denied = r["sandbox"]["denied"].as_array().expect("denied is an array");
    assert!(!denied.is_empty(), "a refused run listed no refusal: {r}");
    assert!(
        r["sandbox"]["denied_total"].as_u64().unwrap_or(0) >= denied.len() as u64,
        "the total must never be smaller than the list it summarises: {r}"
    );
    assert!(
        denied.iter().any(|d| d.as_str().is_some_and(|s| s.contains("DL"))),
        "a refusal must name its code, or a reader cannot look it up: {denied:?}"
    );

    // Refused nothing: the same shape of program, inside the scope it was granted.
    let good = dir.join("good.delulu");
    std::fs::write(
        &good,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{scope}\")\n    \
             let _ = w.write_text(\"yes.txt\", \"x\")\n}}\n"
        ),
    )
    .unwrap();
    let rep2 = dir.join("good.json");
    let o = delulu(&[
        "run",
        good.to_str().unwrap(),
        "--sandbox",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        rep2.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let r2: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&rep2).expect("the run report")).unwrap();
    assert_eq!(
        r2["sandbox"]["denied_total"].as_u64(),
        Some(0),
        "a run that was refused nothing must say nothing was refused: {r2}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The posture object answers the questions a reader has, and `limitations` names every question
/// nothing is enforcing. Identity separation is among them wherever it was not applied — everywhere
/// but a Windows guest since PS-B-03 (RW 4.4) — because a report that omitted it would let a long
/// list of real guarantees imply it.
#[test]
fn the_report_names_what_is_not_confined_as_well_as_what_is() {
    let dir = tmp("posture");
    let (src, scope) = writer(&dir);
    let rep = dir.join("r.json");
    let o = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        rep.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&rep).expect("the run report")).unwrap();
    let sb = &r["sandbox"];
    assert_eq!(sb["requested_level"], 1, "{r}");
    assert_eq!(sb["level"], 1, "{r}");
    let lim: Vec<&str> = sb["limitations"].as_array().expect("limitations").iter().filter_map(|v| v.as_str()).collect();
    if cfg!(windows) {
        // PS-B-03: this host starts the guest as a per-run AppContainer (measured by the identity
        // tests), so the report says so and does not list what it applied as a limitation.
        assert_eq!(sb["posture"]["identity"], "a per-run AppContainer", "{r}");
        assert!(!lim.contains(&"identity_separation"), "an applied identity is not a limitation: {r}");
    } else {
        assert_eq!(sb["posture"]["identity"], "same OS user", "{r}");
        assert!(lim.contains(&"identity_separation"), "identity separation must be named where it is absent: {r}");
    }
    // At least one question must be answered by something that was applied, or the test would pass
    // for a report that called everything a limitation. Which ones is PER PLATFORM, and asserting
    // otherwise is how this test first went red on macOS: it said "every platform caps memory and
    // processor time", and macOS caps neither by rlimit — it gets a processor-time ceiling (PS-A2
    // round three) and deliberately no memory ceiling, because macOS does not meaningfully enforce
    // the rlimit that would give one.
    let mut enforced: Vec<&str> = vec!["processor_time"];
    if cfg!(windows) || cfg!(target_os = "linux") {
        enforced.push("memory");
    }
    for q in enforced {
        assert!(!lim.contains(&q), "`{q}` is enforced here but was listed as a limitation: {r}");
        assert_ne!(sb["posture"][q], "not confined", "{r}");
    }
    if cfg!(target_os = "macos") {
        assert!(lim.contains(&"memory"), "macOS must name the memory ceiling it does not enforce: {r}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `sandbox status` must agree with reality, and this test is the cross-check: whatever the probe says
/// about the L1 launcher, a real `run --sandbox` must behave that way.
///
/// The line it guards used to be a hard-coded `false` reading "the L1 guest launcher: not in this
/// build". It kept saying that after PS-A built the launcher, so `probe`, `status` and `doctor` all
/// reported L1 ABSENT on a host where `run --sandbox` confines. A verdict nothing attempts is not an
/// attempt, and nothing could have caught it — this test is what would have.
#[test]
fn status_agrees_with_what_a_real_sandboxed_run_does() {
    let o = delulu(&["sandbox", "status", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).expect("one JSON envelope");
    assert_eq!(v["command"], "sandbox", "{v}");
    let st = &v["status"];
    let claims_l1 = st["l1_available"].as_bool().expect("l1_available is a bool");

    let dir = tmp("status");
    let src = dir.join("p.delulu");
    std::fs::write(&src, "module g\n\nfn main(root: Root) {\n}\n").unwrap();
    let run = delulu(&["run", src.to_str().unwrap(), "--sandbox"]);
    let ran = run.status.code() == Some(0);
    assert_eq!(
        claims_l1, ran,
        "`sandbox status` says l1_available={claims_l1} and a real `run --sandbox` {}. \
         A probe that disagrees with the thing it probes is worse than no probe.\nstatus: {st}\nrun: {}",
        if ran { "succeeded" } else { "failed" },
        out(&run)
    );
    // And the read-only promise: status must not have appended to the chain it reports on.
    //
    // In a state directory of its OWN, and that is the point rather than tidiness. The first version
    // of this shared the test binary's state directory with every other test here, and those tests
    // run in parallel and DO append sandbox records — so the count moved between the two calls and the
    // test blamed `status`. A read-only claim can only be measured where nothing else writes.
    let solo = dir.join("state");
    std::fs::create_dir_all(solo.join("audit")).unwrap();
    let status_in = |d: &std::path::Path| -> serde_json::Value {
        let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(["sandbox", "status", "--json"])
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .env("DELULU_STATE_DIR", d)
            .output()
            .expect("the delulu binary runs");
        serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&o.stdout))
            .map(|v| v["status"]["chain"].clone())
            .unwrap_or(serde_json::Value::Null)
    };
    let quiet = status_in(&solo);
    assert_eq!(quiet, status_in(&solo), "`sandbox status` changed the chain it was reading");
    // Falsification: something that DOES write must move that number, or the equality above would be
    // satisfied by a `status` that never looked at the chain at all.
    let wrote = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(["run", src.to_str().unwrap(), "--sandbox"])
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .env("DELULU_STATE_DIR", &solo)
        .output()
        .expect("the delulu binary runs");
    if wrote.status.code() == Some(0) {
        assert_ne!(
            quiet,
            status_in(&solo),
            "a sandboxed run left the chain unchanged, so this test cannot tell a writer from a reader"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A verb `sandbox` does not have is refused with a reason, not silently treated as `probe`. `kill` is
/// the case that matters: PS-A-07 lists it, so an operator will type it, and it is deliberately not
/// built — a guest cannot outlive its host on Windows or Linux, and killing by pid alone would
/// eventually kill an innocent process after pid reuse.
#[test]
fn an_unknown_sandbox_verb_is_refused_rather_than_guessed_at() {
    let o = delulu(&["sandbox", "kill"]);
    assert_ne!(o.status.code(), Some(0), "`sandbox kill` must not silently succeed: {}", out(&o));
    // Falsification: the verbs that DO exist succeed, so the refusal above is about `kill` and not
    // about `sandbox` refusing everything.
    for verb in ["probe", "status"] {
        let ok = delulu(&["sandbox", verb, "--json"]);
        assert_eq!(ok.status.code(), Some(0), "`sandbox {verb}` should work: {}", out(&ok));
    }
}

/// PS-A-08: a refusal on the channel gets its OWN audit record, and a clean run does not.
///
/// The pair is the test. A `channel-violation` record on every run would be noise nobody reads; one
/// only when the host actually said no is the single most interesting fact about a program nobody
/// wrote. And the chain must still verify afterwards, because these records are part of it rather than
/// beside it.
#[test]
fn a_refusal_on_the_channel_is_its_own_audit_record_and_a_clean_run_writes_none() {
    let dir = tmp("violation");
    let state = dir.join("state");
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let scope = dir.join("out").display().to_string().replace('\\', "/");
    let sibling = dir.join("elsewhere").display().to_string().replace('\\', "/");
    std::fs::create_dir_all(dir.join("elsewhere")).unwrap();

    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(args)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .env("DELULU_STATE_DIR", &state)
            .output()
            .expect("the delulu binary runs")
    };

    // A clean run first, so the absence below is measured on a chain that already has records in it.
    let (good, _) = writer(&dir);
    let o = run(&["run", good.to_str().unwrap(), "--sandbox", "--grant", &format!("fs.write={scope}")]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let after_clean = String::from_utf8_lossy(&run(&["audit", "query", "--json"]).stdout).into_owned();
    assert!(after_clean.contains("sandbox-launch"), "{after_clean}");
    assert!(
        !after_clean.contains("channel-violation"),
        "a run that was refused nothing must write no violation record: {after_clean}"
    );

    // Then a run that reaches outside the scope it was granted.
    let bad = dir.join("bad.delulu");
    std::fs::write(
        &bad,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{sibling}\")\n    \
             let _ = w.write_text(\"no.txt\", \"x\")\n}}\n"
        ),
    )
    .unwrap();
    let o = run(&["run", bad.to_str().unwrap(), "--sandbox", "--grant", &format!("fs.write={scope}")]);
    assert_ne!(o.status.code(), Some(0), "{}", out(&o));
    let text = String::from_utf8_lossy(&run(&["audit", "query", "--json"]).stdout).into_owned();
    assert!(text.contains("channel-violation"), "the refusal must be recorded: {text}");
    assert!(text.contains("refusal(s) on the channel"), "the record must say what happened: {text}");

    // The chain still verifies with the new record kind in it.
    let v = run(&["audit", "verify"]);
    assert_eq!(v.status.code(), Some(0), "the chain must still verify:\n{}", out(&v));
    let _ = std::fs::remove_dir_all(&dir);
}
