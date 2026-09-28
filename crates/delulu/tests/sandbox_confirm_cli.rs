//! PS-E-01 (`V2_OPENSHELL_STUDY.md` §4.1): the boundary is confirmed BEFORE the program is sent.
//!
//! The host opens the channel with a frame that carries this run's generation and no program; the guest
//! locks itself down and reports what it applied, echoing the generation; only a report the host accepts
//! builds the `Confirmed` value that `send_program` needs. Until 2026-09-28 the host's FIRST frame was the
//! program itself, and the guest reported its confinement afterwards — so a guest that never confined
//! itself had already been handed the program (witnessed below, red on `ff701ae`).
//!
//! What must hold:
//! - a guest that never confirms its boundary never receives a byte of the program, and the run fails
//!   in words that say so;
//! - every sandboxed run has a generation, fresh per run, in the report and in the launch record of the
//!   audit chain, the same in both.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", cwd.join("s"))
        .env("DELULU_HOME", cwd.join("home"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(args)
        .output()
        .expect("the binary runs")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn lab(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("dcnf-{tag}-{}-{}-{n}", std::process::id(), t % 1_000_000_000));
    for sub in ["out", "s", "home"] {
        std::fs::create_dir_all(d.join(sub)).unwrap();
    }
    d
}

/// A program whose text carries a marker no other frame could: if the marker reaches the launcher, the
/// program did.
fn canary_program(d: &Path, canary: &str) {
    std::fs::write(
        d.join("c.delulu"),
        format!("module c\n\nfn main(root: Root) ! {{Write}} {{\n    root.console().println(\"{canary}\")\n}}\n"),
    )
    .unwrap();
}

/// PS-E-01's first witness. The launcher is a guest that NEVER confines itself and never answers: it
/// records every byte the host sends it for two seconds, then goes away. The program must not be among
/// them. (On `ff701ae` it was: the host's first frame was the program.)
#[cfg(unix)]
#[test]
fn a_guest_that_never_confirms_its_boundary_is_never_sent_the_program() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("never");
    let canary = format!("CANARY-PS-E-01-{}", std::process::id());
    canary_program(&d, &canary);
    let captured = d.join("captured.bin");
    let script = d.join("capture.sh");
    // A background list's standard input is /dev/null unless it is redirected from a descriptor opened
    // before it — hence descriptor 3.
    std::fs::write(
        &script,
        format!("#!/bin/sh\nexec 3<&0\ncat <&3 > '{}' &\nsleep 2\nkill $! 2>/dev/null\nexit 0\n", captured.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::create_dir_all(d.join("s").join("audit")).unwrap();
    let report = d.join("r.json");
    let t = std::time::Instant::now();
    let r = delulu(
        &d,
        &[
            "run", "c.delulu", "--sandbox", "--sandbox-backend", &format!("external:{}", script.display()), "--grant", "console",
            "--report-out", report.to_str().unwrap(),
        ],
    );
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert!(t.elapsed() < std::time::Duration::from_secs(30), "it failed promptly: {:?}", t.elapsed());
    // The launcher ran and heard the host: an empty capture would make the absence below vacuous.
    let got = std::fs::read(&captured).expect("the launcher ran and captured the host's frames");
    assert!(!got.is_empty(), "the host sent nothing at all, so this proves nothing: {}", text(&r));
    let got = String::from_utf8_lossy(&got);
    assert!(!got.contains(&canary), "the program reached a guest that never confirmed its boundary: {got:?}");
    assert!(!got.contains("module c"), "the program reached a guest that never confirmed its boundary: {got:?}");
    assert!(!text(&r).contains(&canary), "and it never ran: {}", text(&r));
    assert!(text(&r).contains("confirm"), "the refusal says the boundary was never confirmed: {}", text(&r));
    // The report and the chain say the program never ran — not that it ran and failed.
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["outcome"]["ran"], false, "{v}");
    let g = generation_of(&report);
    let q = delulu(&d, &["audit", "query", "--json"]);
    let chain: serde_json::Value = serde_json::from_slice(&q.stdout).unwrap_or_else(|_| panic!("{}", text(&q)));
    let deaths: Vec<&serde_json::Value> = chain["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["action"] == "sandbox-death" && r["authority"]["generation"] == g.as_str())
        .collect();
    assert_eq!(deaths.len(), 1, "{chain}");
    assert_eq!(deaths[0]["authority"]["confirmed"], false, "{chain}");
    let _ = std::fs::remove_dir_all(&d);
}

fn generation_of(report: &Path) -> String {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(report).unwrap()).unwrap();
    v["sandbox"]["generation"].as_str().unwrap_or_else(|| panic!("no generation in the report: {v}")).to_string()
}

/// Every sandboxed run has its own generation — not only an attested one — and the report and the
/// audit chain's launch record name the same one.
#[test]
fn every_sandboxed_run_has_a_fresh_generation_in_its_report_and_its_launch_record() {
    let d = lab("gen");
    // A sandboxed run records into the chain the host already keeps (PS-A-08); start one.
    std::fs::create_dir_all(d.join("s").join("audit")).unwrap();
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n")
        .unwrap();
    let mut seen = Vec::new();
    for i in 0..2 {
        let report = d.join(format!("r{i}.json"));
        let r = delulu(&d, &["run", "h.delulu", "--sandbox", "--grant", "console", "--report-out", report.to_str().unwrap()]);
        assert_eq!(r.status.code(), Some(0), "{}", text(&r));
        let g = generation_of(&report);
        assert_eq!(g.len(), 64, "32 bytes of the OS's randomness, in hex: {g}");
        assert!(g.bytes().all(|b| b.is_ascii_hexdigit()), "{g}");
        seen.push(g);
    }
    assert_ne!(seen[0], seen[1], "each run has its own generation");
    let q = delulu(&d, &["audit", "query", "--json"]);
    let chain: serde_json::Value = serde_json::from_slice(&q.stdout).unwrap_or_else(|_| panic!("{}", text(&q)));
    let records = chain["records"].as_array().unwrap_or_else(|| panic!("{chain}"));
    for g in &seen {
        let of = |action: &str| {
            records.iter().filter(|r| r["action"] == action && r["authority"]["generation"] == g.as_str()).count()
        };
        assert_eq!(of("sandbox-launch"), 1, "one launch record names generation {g}: {chain}");
        assert_eq!(of("sandbox-death"), 1, "one death record names generation {g}: {chain}");
        assert!(
            records.iter().any(|r| r["action"] == "sandbox-death"
                && r["authority"]["generation"] == g.as_str()
                && r["authority"]["confirmed"] == true),
            "and it says the guest confirmed its boundary: {chain}"
        );
    }
    let v = delulu(&d, &["audit", "verify"]);
    assert_eq!(v.status.code(), Some(0), "{}", text(&v));
    let _ = std::fs::remove_dir_all(&d);
}
