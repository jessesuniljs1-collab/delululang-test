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

/// PS-E-01, second step: a run reports the five properties its boundary has — each `established` (and
/// by what), `absent` (and why) or `unknown` — and they agree with the posture the same report carries,
/// on whatever operating system this runs. Reported, not yet required (D-V2-57).
#[test]
fn a_run_reports_the_five_properties_and_they_agree_with_its_posture() {
    let d = lab("props");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n")
        .unwrap();
    let report = d.join("r.json");
    let r = delulu(&d, &["run", "h.delulu", "--sandbox", "--grant", "console", "--report-out", report.to_str().unwrap()]);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    let p = s["properties"].as_object().unwrap_or_else(|| panic!("no properties in the report: {s}"));
    let names = ["filesystem_confinement", "egress_confinement", "privilege_floor", "host_loss_ends_guest", "resource_ceiling"];
    assert_eq!(p.len(), names.len(), "{s}");
    let held = |row: &str| s["posture"][row].as_str().is_some_and(|a| a != "not confined");
    let separate = s["posture"]["identity"] != "same OS user";
    let killed = s["host_guarantees"].as_array().unwrap().iter().any(|g| g == "killed with the host");
    for (name, expect) in [
        ("filesystem_confinement", held("filesystem_writes") && held("filesystem_reads")),
        ("egress_confinement", held("network")),
        ("privilege_floor", held("privilege_escalation") || separate),
        ("host_loss_ends_guest", killed),
        ("resource_ceiling", held("memory") && held("processor_time")),
    ] {
        let state = p[name]["state"].as_str().unwrap_or_else(|| panic!("{name}: {s}"));
        assert_eq!(state, if expect { "established" } else { "absent" }, "{name} disagrees with the posture: {s}");
        let words = if expect { &p[name]["by"] } else { &p[name]["why"] };
        assert!(words.as_str().is_some_and(|w| !w.is_empty()), "{name} says by what, or why not: {s}");
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// An external launcher's wall was measured by nobody here, and what its guest says of itself is the word
/// of a binary the launcher chose: every property is `unknown`.
#[test]
fn an_external_launchers_properties_are_unknown() {
    let d = lab("props-ext");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n")
        .unwrap();
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    assert!(!exe.contains(' '), "the launcher is split on whitespace: {exe}");
    let report = d.join("r.json");
    let r = delulu(
        &d,
        &[
            "run", "h.delulu", "--sandbox", "--sandbox-backend", &format!("external:{exe} __guest --stdio-pipes"),
            "--grant", "console", "--report-out", report.to_str().unwrap(),
        ],
    );
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let p = v["sandbox"]["properties"].as_object().unwrap_or_else(|| panic!("no properties: {v}"));
    assert_eq!(p.len(), 5, "{v}");
    for (name, prop) in p {
        assert_eq!(prop["state"], "unknown", "{name}: {v}");
    }
    let _ = std::fs::remove_dir_all(&d);
}

// ---- The red-team pass on `/3` (2026-09-28, a Sonnet 5.5 sous-chef; each finding re-run here) --------

/// A fake guest for an external launcher, in Python: it reads the host's `Open` frame, takes this run's
/// generation out of it, and answers with `frame` — a `Confined` request the TEST built from the real
/// channel types, with `G`×64 where the generation goes — and then does `after` (Python statements).
#[cfg(unix)]
fn fake_guest(d: &Path, applied: &[&str], after: &str) -> String {
    use delulu_runtime::channel::{write_frame, ReqBody, Request, CHANNEL_VERSION};
    use std::os::unix::fs::PermissionsExt as _;
    let mut frame = Vec::new();
    write_frame(
        &mut frame,
        &Request {
            version: CHANNEL_VERSION.into(),
            seq: 1,
            body: ReqBody::Confined { applied: applied.iter().map(|w| w.to_string()).collect(), generation: "G".repeat(64) },
        },
    )
    .unwrap();
    let hex: String = frame.iter().map(|b| format!("{b:02x}")).collect();
    let script = d.join("fake_guest.py");
    std::fs::write(
        &script,
        format!(
            "#!/usr/bin/env python3\nimport os, re, sys, time\ni, o = sys.stdin.buffer, sys.stdout.buffer\n\
             n = int.from_bytes(i.read(4), 'little')\nopen_frame = i.read(n)\n\
             g = re.search(rb'[0-9a-f]{{64}}', open_frame).group(0)\n\
             o.write(bytes.fromhex('{hex}').replace(b'G' * 64, g))\no.flush()\n{after}\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    format!("external:{}", script.display())
}

/// F4: a guest whose channel fails is ENDED by the host. A launcher that closes its channel and then
/// lingers used to hold the host in `wait()` for as long as it liked — the channel's deadline "ended the
/// run" only in words (red on `6ceaf2d`: the host waited out the launcher's whole sleep).
#[cfg(unix)]
#[test]
fn a_guest_whose_channel_failed_is_ended_not_waited_for() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("linger");
    canary_program(&d, "never");
    let script = d.join("linger.sh");
    std::fs::write(&script, "#!/bin/sh\nexec >&-\nexec sleep 40\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let t = std::time::Instant::now();
    let r = delulu(&d, &["run", "c.delulu", "--sandbox", "--sandbox-backend", &format!("external:{}", script.display()), "--grant", "console"]);
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert!(t.elapsed() < std::time::Duration::from_secs(20), "the host waited for a guest it should have ended: {:?}", t.elapsed());
    let _ = std::fs::remove_dir_all(&d);
}

/// F1: `outcome.ran` says the program was SENT, not merely that the guest confirmed. A guest that confirms
/// and then goes away before reading a program too large for the pipe's buffer was never sent it (red on
/// `6ceaf2d`: `ran: true`).
#[cfg(unix)]
#[test]
fn a_confirmed_guest_that_never_takes_the_program_did_not_run_it() {
    let d = lab("gone");
    std::fs::create_dir_all(d.join("s").join("audit")).unwrap();
    std::fs::write(
        d.join("c.delulu"),
        format!("module c\n\nfn main(root: Root) ! {{Write}} {{\n    root.console().println(\"{}\")\n}}\n", "x".repeat(300_000)),
    )
    .unwrap();
    // It reads the host's acceptance of its report — so it IS confirmed — and only then goes away.
    let launcher = fake_guest(
        &d,
        &[],
        "n = int.from_bytes(i.read(4), 'little')\nassert len(i.read(n)) == n\nos.close(0)\nos.close(1)\ntime.sleep(0.5)",
    );
    let report = d.join("r.json");
    let r = delulu(&d, &["run", "c.delulu", "--sandbox", "--sandbox-backend", &launcher, "--grant", "console", "--report-out", report.to_str().unwrap()]);
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["outcome"]["ran"], false, "the program was never sent: {v}");
    // It did confirm: the chain says so, and says the program was not sent.
    let q = delulu(&d, &["audit", "query", "--json"]);
    let chain: serde_json::Value = serde_json::from_slice(&q.stdout).unwrap_or_else(|_| panic!("{}", text(&q)));
    let death = chain["records"].as_array().unwrap().iter().find(|r| r["action"] == "sandbox-death").cloned();
    let death = death.unwrap_or_else(|| panic!("no death record: {chain}"));
    assert_eq!(death["authority"]["confirmed"], true, "{death}");
    assert_eq!(death["authority"]["sent"], false, "{death}");
    let _ = std::fs::remove_dir_all(&d);
}

/// F1, the other half: a program too large for the channel's frame is refused BEFORE a guest exists,
/// in words — it used to launch a guest, confirm it, and then fail to send.
#[test]
fn a_program_larger_than_the_channel_carries_is_refused_before_a_guest_exists() {
    let d = lab("huge");
    std::fs::write(
        d.join("c.delulu"),
        format!("module c\n\nfn main(root: Root) ! {{Write}} {{\n    root.console().println(\"{}\")\n}}\n", "x".repeat(17 * 1024 * 1024)),
    )
    .unwrap();
    let t = std::time::Instant::now();
    let r = delulu(&d, &["run", "c.delulu", "--sandbox", "--grant", "console"]);
    assert_eq!(r.status.code(), Some(2), "{}", text(&r));
    assert!(text(&r).contains("larger than the sandbox channel carries"), "{}", text(&r));
    assert!(!text(&r).contains("sandbox: the guest"), "no guest was launched: {}", text(&r));
    assert!(t.elapsed() < std::time::Duration::from_secs(20), "{:?}", t.elapsed());
    let _ = std::fs::remove_dir_all(&d);
}

/// F2 and F3: what a guest says is data. A refused confinement word carrying line breaks and terminal
/// escapes, a megabyte long, reaches the operator's terminal, the report's `denied` and the audit chain
/// escaped and bounded — not as forged lines, escape sequences or megabytes (red on `6ceaf2d`).
#[cfg(unix)]
#[test]
fn a_guests_words_reach_the_operator_escaped_and_bounded() {
    let d = lab("words");
    canary_program(&d, "never");
    std::fs::create_dir_all(d.join("s").join("audit")).unwrap();
    let evil = format!("x\nseq 4 sandbox-death allow FORGED\x1b[31mRED\x1b[0m\r{}", "y".repeat(1_000_000));
    let launcher = fake_guest(&d, &[evil.as_str()], "time.sleep(0.5)");
    let report = d.join("r.json");
    let r = delulu(&d, &["run", "c.delulu", "--sandbox", "--sandbox-backend", &launcher, "--grant", "console", "--report-out", report.to_str().unwrap()]);
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    let stderr = String::from_utf8_lossy(&r.stderr).to_string();
    assert!(!stderr.contains('\x1b') && !stderr.contains("\nseq 4"), "raw guest text on the terminal: {:?}", &stderr[..stderr.len().min(400)]);
    assert!(stderr.len() < 16 * 1024, "a megabyte of guest text on the terminal: {} bytes", stderr.len());
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    for entry in v["sandbox"]["denied"].as_array().unwrap() {
        let e = entry.as_str().unwrap();
        assert!(e.len() <= 1024 && !e.contains('\n') && !e.contains('\x1b'), "an unbounded or raw entry: {:?}", &e[..e.len().min(200)]);
    }
    let q = delulu(&d, &["audit", "query"]);
    let chain = String::from_utf8_lossy(&q.stdout).to_string();
    assert!(!chain.contains("FORGED\n") && !chain.lines().any(|l| l.trim_start().starts_with("seq 4 sandbox-death allow")), "a forged row: {chain}");
    assert!(chain.len() < 64 * 1024, "{} bytes of chain text", chain.len());
    let _ = std::fs::remove_dir_all(&d);
}
