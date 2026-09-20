//! PS-0-04 / PS-0-05: `delulu sandbox probe` and the `doctor` sandbox section. Every verdict must
//! come from an attempt: these tests re-attempt what they can independently and require the probe
//! to agree, so a probe that reports "present" without attempting fails here.

use std::process::{Command, Output};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary must run")
}

fn probe() -> serde_json::Value {
    let o = delulu(&["sandbox", "probe", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).expect("one envelope")
}

#[test]
fn every_level_is_a_verdict_on_attempts() {
    let v = probe();
    assert_eq!(v["command"], "sandbox");
    let levels = v["levels"].as_array().expect("levels");
    assert_eq!(levels.iter().map(|l| l["level"].as_u64().unwrap()).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
    for l in levels {
        let attempts = l["attempts"].as_array().expect("attempts");
        assert!(!attempts.is_empty(), "a level with no attempt cannot have a verdict: {l}");
        let all_ok = attempts.iter().all(|a| a["ok"] == true);
        assert_eq!(l["available"], all_ok, "present exactly when every attempt succeeded: {l}");
        if !all_ok {
            let first = attempts.iter().find(|a| a["ok"] == false).unwrap();
            let want = format!("{}: {}", first["what"].as_str().unwrap(), first["detail"].as_str().unwrap());
            assert_eq!(l["first_missing"], want.as_str(), "{l}");
        }
        for a in attempts {
            let d = a["detail"].as_str().unwrap_or("").to_lowercase();
            assert!(!d.contains("version "), "a version string is not an attempt: {a}");
        }
    }
    // `highest_available` must be the highest level whose attempts ALL succeeded, computed here from
    // the same list rather than compared against a fixed number. It used to assert 0, which was true
    // when PS-0 wrote it and false the moment PS-A built the L1 launcher — a test pinning a number is
    // a test that has to be edited every time the product improves, and the thing worth pinning is
    // the RELATION between the verdict and the attempts.
    let highest = levels
        .iter()
        .filter(|l| l["attempts"].as_array().is_some_and(|a| !a.is_empty() && a.iter().all(|x| x["ok"] == true)))
        .filter_map(|l| l["level"].as_u64())
        .max()
        .unwrap_or(0);
    assert_eq!(v["highest_available"].as_u64(), Some(highest), "{v}");
    // And the levels no build can reach yet must still be absent, or the assertion above would be
    // satisfied by a probe that simply claimed everything.
    for lvl in [2usize, 3, 4] {
        assert_eq!(levels[lvl]["available"], false, "L{lvl} has no launcher in this build: {}", levels[lvl]);
    }
}

/// The mutant test: the probe's KVM verdict must equal this test's own attempt to open `/dev/kvm`
/// read-write. A probe that reported KVM from a version string or a file's mere existence — or
/// hard-coded it — disagrees with the attempt on some host, and CI runs three.
#[test]
fn the_kvm_line_matches_an_independent_attempt() {
    let v = probe();
    let l2 = &v["levels"][2];
    let kvm = l2["attempts"].as_array().unwrap().iter().find(|a| a["what"] == "KVM").expect("a KVM attempt");
    let mine = std::fs::OpenOptions::new().read(true).write(true).open("/dev/kvm").is_ok();
    assert_eq!(kvm["ok"], mine, "the probe's KVM line disagrees with an attempt: {kvm}");
}

/// PS-0-05: `doctor` carries a sandbox section built on the probe, holding only decision lines.
#[test]
fn doctor_has_a_sandbox_section_that_agrees_with_the_probe() {
    let o = delulu(&["doctor", "--check", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("one envelope");
    let checks = v["doctor"]["checks"].as_array().expect("checks");
    let sandbox: Vec<&serde_json::Value> = checks.iter().filter(|c| c["section"] == "sandbox").collect();
    let names: Vec<&str> = sandbox.iter().filter_map(|c| c["name"].as_str()).collect();
    for want in [
        "backend", "level available", "KVM", "OS primitives", "network enforcement", "filesystem enforcement",
        "identity separation", "resource controls", "profile", "break-glass", "relaxed restrictions",
    ] {
        assert!(names.contains(&want), "the sandbox section names `{want}`: {names:?}");
    }
    let kvm = sandbox.iter().find(|c| c["name"] == "KVM").unwrap();
    let mine = std::fs::OpenOptions::new().read(true).write(true).open("/dev/kvm").is_ok();
    assert_eq!(kvm["detail"].as_str().unwrap().starts_with("available"), mine, "{kvm}");
    // Agreement with the probe, not a fixed level: `doctor` must name whatever the probe found, and
    // this line asserted `L0` until PS-A built the L1 launcher and made that wrong on every host.
    let level = sandbox.iter().find(|c| c["name"] == "level available").unwrap();
    let p = probe();
    let want = format!("L{}", p["highest_available"].as_u64().unwrap_or(0));
    assert!(
        level["detail"].as_str().unwrap().starts_with(&want),
        "doctor says `{}` and the probe says `{want}`: {level}",
        level["detail"]
    );
    // Decision lines, not failures: the section never fails the run.
    assert!(sandbox.iter().all(|c| c["status"] != "problem"), "{sandbox:?}");
}
