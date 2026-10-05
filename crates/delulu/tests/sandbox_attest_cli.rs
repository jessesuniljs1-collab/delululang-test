//! PS-D-02: the attestation seam. `delulu run … --sandbox --sandbox-backend external:CMD
//! --require-attestation HEX` serves the guest only after the launcher's attester has signed a statement
//! over this run's nonce with the pinned key — checked BEFORE the program is sent, so a refused run never
//! ran. The launcher here is the binary itself: `delulu sandbox attest` (the reference, software attester)
//! in front of `delulu __guest --stdio-pipes`, which is the whole protocol with no container in the way.
//!
//! What must hold:
//! - an attested guest is served, and the report carries the attester's claims AS the attester's —
//!   beside `host_guarantees`, never in them; level 3; `fully_enforced` false;
//! - every attestation that does not hold (another key, no attester, a launcher that ends, a document
//!   replayed from another run) is refused in words, and the program's effect NEVER happens;
//! - the flag is refused where it cannot apply (no sandbox, L1, L2, not a key), before anything runs;
//! - the words after `--` belong to the attester's command, never to `delulu`;
//! - (D-V2-87) a claim that names a property — `PROPERTY: how` — answers that property's requirement, as
//!   the attester's word beside a state that stays `unknown`; one it does not name still refuses;
//! - (D-V2-89) a statement that names the launcher its attester measured is checked against the launcher
//!   the host started, and one for another file is refused before the program is sent.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn exe() -> String {
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    assert!(!exe.contains(' '), "the launcher is split on whitespace; this checkout's path has a space: {exe}");
    exe
}

fn delulu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", cwd.join("s"))
        .env("DELULU_HOME", cwd.join("home"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .env_remove("DELULU_ATTEST_NONCE")
        .env_remove("DELULU_ATTEST_OUT")
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
    let d = std::env::temp_dir().join(format!("datt-{tag}-{}-{}-{n}", std::process::id(), t % 1_000_000_000));
    for sub in ["out", "s", "home"] {
        std::fs::create_dir_all(d.join(sub)).unwrap();
    }
    d
}

/// A key minted the way an operator mints one: `delulu keygen`. Returns the seed's path and the public key.
fn keygen(d: &Path, name: &str) -> (String, String) {
    let r = delulu(d, &["keygen", "--name", name, "--json"]);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    let key = v["key"].as_str().unwrap().replace('\\', "/");
    assert!(!key.contains(' '), "{key}");
    (key, v["public_key"].as_str().unwrap().to_string())
}

/// The program: it writes a file through its grant, so "it ran" and "it never ran" are both visible.
fn program(d: &Path) -> String {
    let out = d.join("out").display().to_string().replace('\\', "/");
    std::fs::write(
        d.join("w.delulu"),
        format!(
            "module w\n\nfn main(root: Root) ! {{Write}} {{\n    let o = root.console()\n    let fw = root.fs_write(\"{out}\")\n    \
             match fw.write_text(\"made.txt\", \"by the host\") {{\n        Ok(_) => o.println(\"wrote\"),\n        Err(_) => o.println(\"refused\")\n    }}\n}}\n"
        ),
    )
    .unwrap();
    out
}

fn attesting_launcher(seed: &str) -> String {
    let exe = exe();
    format!(
        "external:{exe} sandbox attest --key {seed} --attester test-attester --guarantee no-network \
         --guarantee runs-under-gvisor -- {exe} __guest --stdio-pipes"
    )
}

fn run_attested(d: &Path, launcher: &str, pinned: &str, report: &Path) -> Output {
    let out = d.join("out").display().to_string().replace('\\', "/");
    delulu(
        d,
        &[
            "run", "w.delulu", "--grant", "console", "--grant", &format!("fs.write={out}"), "--sandbox",
            "--sandbox-backend", launcher, "--require-attestation", pinned, "--report-out", report.to_str().unwrap(),
        ],
    )
}

#[test]
fn an_attested_guest_is_served_and_the_report_carries_the_attesters_word_as_its_own() {
    let d = lab("ok");
    program(&d);
    let (seed, public) = keygen(&d, "attester");
    let report = d.join("r.json");
    let r = run_attested(&d, &attesting_launcher(&seed), &public.to_ascii_uppercase(), &report);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(String::from_utf8_lossy(&r.stdout).contains("wrote"), "{}", text(&r));
    assert_eq!(std::fs::read_to_string(d.join("out").join("made.txt")).unwrap(), "by the host");
    assert!(text(&r).contains("attested by `test-attester`"), "the run says who vouched: {}", text(&r));
    assert!(text(&r).contains("the attester's word, not a measurement"), "{}", text(&r));

    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    assert_eq!(s["attestation"]["verified"], true, "{s}");
    assert_eq!(s["attestation"]["attester"], "test-attester", "{s}");
    assert_eq!(s["attestation"]["key"], public.as_str(), "the key as pinned, in one spelling: {s}");
    assert_eq!(s["attestation"]["guarantees"], serde_json::json!(["no-network", "runs-under-gvisor"]), "{s}");
    // The attester's claims are NOT the host's: the level stays 3 and nothing is enforced by DeluluLang.
    assert_eq!(s["level"], 3, "{s}");
    assert_eq!(s["backend"], "external", "{s}");
    assert_eq!(s["fully_enforced"], false, "{s}");
    for g in s["host_guarantees"].as_array().unwrap() {
        assert!(!g.as_str().unwrap().contains("gvisor") && !g.as_str().unwrap().contains("no-network"), "claims merged: {s}");
    }
    // The launcher is named by its program; its arguments — the key's path among them — never are.
    assert!(!s["launcher"].as_str().unwrap().contains("attest"), "{s}");
    assert!(!std::fs::read_to_string(&report).unwrap().contains(&seed), "the seed's path is never recorded");
    // The report is what `delulu schema sandbox` says it is.
    let f = d.join("sandbox.json");
    std::fs::write(&f, s.to_string()).unwrap();
    let val = delulu(&d, &["schema", "validate", "sandbox", f.to_str().unwrap(), "--json"]);
    let val: serde_json::Value = serde_json::from_slice(&val.stdout).unwrap();
    assert_eq!(val["validate"]["valid"], true, "{:#}", val["validate"]["errors"]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn every_attestation_that_does_not_hold_is_refused_before_the_program_is_sent() {
    let d = lab("refuse");
    program(&d);
    let (seed, public) = keygen(&d, "attester");
    let (_, other_public) = keygen(&d, "someone-else");
    let exe = exe();
    let made = d.join("out").join("made.txt");
    let cases = [
        // A statement signed by a key this run did not pin.
        (attesting_launcher(&seed), other_public.clone(), "the key this run pinned is", 40),
        // A launcher that runs the guest and never attests: refused at the attestation deadline.
        (format!("external:{exe} __guest --stdio-pipes"), public.clone(), "wrote no attestation within", 40),
        // A launcher that ends without attesting: refused at once.
        (format!("external:{exe} --version"), public.clone(), "before it wrote an attestation", 8),
        // An attester that refuses to sign (no guarantee): it ends, and the host says so.
        (format!("external:{exe} sandbox attest --key {seed} --attester a -- {exe} __guest --stdio-pipes"), public.clone(), "before it wrote an attestation", 8),
    ];
    for (i, (launcher, pinned, says, within)) in cases.iter().enumerate() {
        let t = std::time::Instant::now();
        let r = run_attested(&d, launcher, pinned, &d.join(format!("r{i}.json")));
        assert_eq!(r.status.code(), Some(1), "case {i}: {}", text(&r));
        assert!(text(&r).contains(says), "case {i}: {}", text(&r));
        assert!(text(&r).contains("the program was never sent"), "case {i}: {}", text(&r));
        assert!(!String::from_utf8_lossy(&r.stdout).contains("wrote"), "case {i}: the program ran: {}", text(&r));
        assert!(!made.exists(), "case {i}: the program's effect happened on a refused attestation");
        assert!(t.elapsed() < std::time::Duration::from_secs(*within), "case {i}: took {:?}", t.elapsed());
        assert!(!text(&r).contains("os error"), "case {i}: in words: {}", text(&r));
    }
    // The same launcher and key, pinned correctly, is served — so each refusal above was the attestation's.
    let r = run_attested(&d, &attesting_launcher(&seed), &public, &d.join("ok.json"));
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(made.exists());
    let _ = std::fs::remove_dir_all(&d);
}

/// A document made for another run — valid, signed by the pinned key — is refused on its nonce.
#[cfg(unix)]
#[test]
fn a_document_replayed_from_another_run_is_refused_on_its_nonce() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("replay");
    program(&d);
    let (seed, public) = keygen(&d, "attester");
    let exe = exe();
    // An honest attestation, for a run whose nonce was 00…01.
    let old = d.join("old.json");
    let r = Command::new(&exe)
        .current_dir(&d)
        .env("DELULU_ATTEST_NONCE", format!("{}01", "0".repeat(62)))
        .env("DELULU_ATTEST_OUT", &old)
        .args(["sandbox", "attest", "--key", &seed, "--attester", "test-attester", "--guarantee", "no-network", "--", &exe, "--version"])
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(std::fs::read_to_string(&old).unwrap().contains("delulu-attestation-v1"));
    // A launcher that hands the host that old document — whole, as the protocol asks: a temporary file, then
    // a rename (a plain `cp` creates the file before it fills it, and the host, which reads the document the
    // moment it exists, once read it empty on a macOS runner — `36537537718`) — and then runs the guest.
    let script = d.join("replay.sh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\ncp {} \"$DELULU_ATTEST_OUT.part\"\nmv \"$DELULU_ATTEST_OUT.part\" \"$DELULU_ATTEST_OUT\"\nexec {exe} __guest --stdio-pipes\n",
            old.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let r = run_attested(&d, &format!("external:{}", script.display()), &public, &d.join("r.json"));
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert!(text(&r).contains("not made for this run"), "{}", text(&r));
    assert!(!d.join("out").join("made.txt").exists(), "a replayed attestation let the program run");
    let _ = std::fs::remove_dir_all(&d);
}

/// The host reads the document the moment it exists, so an attester writes it WHOLE — a temporary file,
/// then a rename. One that writes it in place (creates the file, then fills it) can be read half-written:
/// routine run 4 found the replay test's own `cp` losing that race on a macOS runner (`36537537718`,
/// "EOF while parsing a value at line 1 column 0"). Such a document is refused before the program is
/// sent, and the words name the protocol it broke rather than a parser's position.
#[cfg(unix)]
#[test]
fn a_document_written_in_place_is_refused_in_words_that_name_the_rename() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("inplace");
    program(&d);
    let (_, public) = keygen(&d, "attester");
    let exe = exe();
    // What the host can see mid-write, held still: the file created and nothing in it yet, and the first
    // half of a document. Neither launcher ever completes it, so the witness is not a race.
    for (tag, write) in [
        ("empty", ": > \"$DELULU_ATTEST_OUT\"".to_string()),
        ("half", "printf '%s' '{\"format\":\"delulu-attestation-v1\",\"statement\":{' > \"$DELULU_ATTEST_OUT\"".to_string()),
    ] {
        let script = d.join(format!("{tag}.sh"));
        std::fs::write(&script, format!("#!/bin/sh\n{write}\nexec {exe} __guest --stdio-pipes\n")).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let r = run_attested(&d, &format!("external:{}", script.display()), &public, &d.join(format!("{tag}.json")));
        assert_eq!(r.status.code(), Some(1), "{tag}: {}", text(&r));
        assert!(text(&r).contains("the attestation is incomplete"), "{tag}: {}", text(&r));
        assert!(text(&r).contains("a temporary file, then a rename"), "{tag}: the words name the protocol: {}", text(&r));
        assert!(!text(&r).contains("EOF while parsing"), "{tag}: a parser's position is not the reason: {}", text(&r));
        assert!(text(&r).contains("the program was never sent"), "{tag}: {}", text(&r));
        assert!(!d.join("out").join("made.txt").exists(), "{tag}: the program ran on half a document");
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_flag_is_refused_where_it_cannot_apply_and_a_dry_run_says_it_was_required() {
    let d = lab("usage");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let (_, public) = keygen(&d, "attester");
    let exe = exe();
    let launcher = format!("external:{exe} __guest --stdio-pipes");
    for (args, says) in [
        (vec!["--require-attestation", public.as_str()], "describes a sandboxed run"),
        (vec!["--sandbox", "--require-attestation", public.as_str()], "is for an external launcher"),
        (vec!["--sandbox", "--isolation", "microvm", "--require-attestation", public.as_str()], "is for an external launcher"),
        (vec!["--sandbox", "--sandbox-backend", launcher.as_str(), "--require-attestation", "abc"], "is not an ed25519 public key"),
        (vec!["--sandbox", "--sandbox-backend", launcher.as_str(), "--require-attestation"], "--require-attestation"),
    ] {
        let mut a = vec!["run", "h.delulu", "--grant", "console"];
        a.extend(args.iter().copied());
        let r = delulu(&d, &a);
        assert_eq!(r.status.code(), Some(2), "{args:?}: {}", text(&r));
        assert!(text(&r).contains(says), "{args:?}: {}", text(&r));
        assert!(!String::from_utf8_lossy(&r.stdout).contains("hi\n"), "{args:?}: nothing ran");
    }
    // A dry run launches nothing, so it verifies nothing — and says the attestation was required.
    let r = delulu(
        &d,
        &["run", "h.delulu", "--grant", "console", "--sandbox", "--mode", "audit", "--sandbox-backend", &launcher, "--require-attestation", &public],
    );
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v["sandbox"]["attestation"], serde_json::json!({ "key": public, "attester": null, "guarantees": [], "verified": false }));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_reference_attester_refuses_in_words_and_never_reads_its_commands_words() {
    let d = lab("attester");
    let (seed, _) = keygen(&d, "attester");
    let exe = exe();
    let base = ["sandbox", "attest", "--key", seed.as_str(), "--attester", "a", "--guarantee", "g"];
    // Not started by a host that asked for attestation.
    let mut a = base.to_vec();
    a.extend(["--", exe.as_str(), "--version"]);
    let r = delulu(&d, &a);
    assert_eq!(r.status.code(), Some(2), "{}", text(&r));
    assert!(text(&r).contains("DELULU_ATTEST_NONCE"), "{}", text(&r));
    // Its standard output is the channel: no envelope, and no command, is refused in words.
    for (extra, says) in [
        (vec!["--json", "--", exe.as_str(), "--version"], "has no `--json`"),
        (vec![], "goes after `--`"),
        // Refused by `attest`'s own parser since `--grant` is documented for `sandbox` (PS-E-05's
        // `policy --format openshell`, routine run 8); the dispatcher refused it before. Exit 2 either way.
        (vec!["--grant", "console", "--", exe.as_str()], "does not take `--grant`"),
    ] {
        let mut a = base.to_vec();
        a.extend(extra.iter().copied());
        let r = delulu(&d, &a);
        assert_eq!(r.status.code(), Some(2), "{extra:?}: {}", text(&r));
        assert!(text(&r).contains(says), "{extra:?}: {}", text(&r));
    }
    // The words after `--` are the command's: `-h` there is the COMMAND's help, not `delulu sandbox`'s,
    // and the document is written before the command runs.
    let out = d.join("doc.json");
    let r = Command::new(&exe)
        .current_dir(&d)
        .env("DELULU_ATTEST_NONCE", "ab".repeat(32))
        .env("DELULU_ATTEST_OUT", &out)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(["sandbox", "attest", "--key", &seed, "--attester", "a", "--guarantee", "g", "--", &exe, "check", "-h"])
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(String::from_utf8_lossy(&r.stdout).contains("delulu check"), "the command's own help: {}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("delulu sandbox"), "delulu read its command's words: {}", text(&r));
    let doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(doc["format"], "delulu-attestation-v1");
    assert_eq!(doc["statement"]["nonce"], "ab".repeat(32));
    let _ = std::fs::remove_dir_all(&d);
}

/// ATTEST-FIFO-1: the host reads the attester's document before any watchdog runs, so what it opens
/// must be a document. A hostile launcher that makes the path a named pipe would hold a blocking open
/// for ever; one that makes it a link would have the host read whatever the link names. Both are
/// refused, promptly, as not a regular file.
#[cfg(unix)]
#[test]
fn a_launcher_cannot_make_the_host_open_a_pipe_or_follow_a_link() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("fifo");
    program(&d);
    let (_, public) = keygen(&d, "attester");
    let exe = exe();
    for (tag, make) in [("fifo", "mkfifo \"$DELULU_ATTEST_OUT\""), ("link", "ln -s /etc/hostname \"$DELULU_ATTEST_OUT\"")] {
        let script = d.join(format!("{tag}.sh"));
        std::fs::write(&script, format!("#!/bin/sh\n{make}\nexec {exe} __guest --stdio-pipes\n")).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let out = d.join("out").display().to_string();
        let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(&d)
            .env("DELULU_STATE_DIR", d.join("s"))
            .env("DELULU_HOME", d.join("home"))
            .env("DELULU_NO_FIRST_RUN", "1")
            .args([
                "run", "w.delulu", "--grant", "console", "--grant", &format!("fs.write={out}"), "--sandbox",
                "--sandbox-backend", &format!("external:{}", script.display()), "--require-attestation", &public,
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let t = std::time::Instant::now();
        let status = loop {
            if let Some(st) = child.try_wait().unwrap() {
                break Some(st);
            }
            if t.elapsed() > std::time::Duration::from_secs(30) {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        let o = child.wait_with_output().unwrap();
        let st = status.unwrap_or_else(|| panic!("{tag}: the host hung on what the launcher put at the attestation's path"));
        assert_eq!(st.code(), Some(1), "{tag}: {}", text(&o));
        assert!(text(&o).contains("not a regular file"), "{tag}: {}", text(&o));
        assert!(!d.join("out").join("made.txt").exists(), "{tag}: the program ran");
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// The five properties a run reports, as `boundary::PROPERTIES` names them.
const PROPERTIES: [&str; 5] =
    ["filesystem_confinement", "egress_confinement", "privilege_floor", "host_loss_ends_guest", "resource_ceiling"];

/// PS-E-01's remainder (D-V2-87): an attester's claim that NAMES a property — `PROPERTY: how` — answers that
/// property's requirement at L3, as the attester's word. `hostile-agent`'s refusal (DL1408) named "an
/// external launcher whose attester vouches for it" as a way out, and no attestation could be one: every
/// L3 property stayed `unknown` whatever the pinned key signed. Red on `2cb3f87`: refused with all five
/// vouched for. The property's state stays `unknown` — DeluluLang measured none of it — and the claim is
/// kept beside it, with the attester's name; a property the attester did not name still refuses, and so
/// does a claim that only resembles a property's name.
#[test]
fn an_attester_that_vouches_for_each_property_by_name_meets_hostile_agent_and_the_report_says_whose_word_it_is() {
    let d = lab("vouch");
    let out = program(&d);
    let (seed, public) = keygen(&d, "attester");
    let exe = exe();
    let made = d.join("out").join("made.txt");
    let launcher = |claims: &[String]| {
        let flags: Vec<String> = claims.iter().map(|c| format!("--guarantee {c}")).collect();
        format!(
            "external:{exe} sandbox attest --key {seed} --attester test-attester {} -- {exe} __guest --stdio-pipes",
            flags.join(" ")
        )
    };
    let run = |launcher: &str, report: &Path| {
        delulu(
            &d,
            &[
                "run", "w.delulu", "--grant", "console", "--grant", &format!("fs.write={out}"), "--sandbox",
                "--sandbox-backend", launcher, "--sandbox-profile", "hostile-agent", "--require-attestation", &public,
                "--report-out", report.to_str().unwrap(),
            ],
        )
    };
    let all: Vec<String> = PROPERTIES.iter().map(|p| format!("{p}:by-the-image-{p}")).collect();

    // Every property vouched for by name: the run is served, and the program's effect happens.
    let report = d.join("all.json");
    let r = run(&launcher(&all), &report);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(String::from_utf8_lossy(&r.stdout).contains("wrote"), "{}", text(&r));
    assert!(made.exists(), "the program's effect did not happen");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    assert_eq!(s["requested"], "hostile-agent", "{s}");
    assert_eq!(s["level"], 3, "{s}");
    assert_eq!(s["fully_enforced"], false, "{s}");
    assert_eq!(s["host_guarantees"].as_array().unwrap().len(), 0, "the attester's claims are not the host's: {s}");
    for p in PROPERTIES {
        let prop = &s["properties"][p];
        assert_eq!(prop["state"], "unknown", "{p}: DeluluLang measured none of an external wall: {s}");
        assert_eq!(prop["attested"]["attester"], "test-attester", "{p}: whose word it is: {s}");
        assert_eq!(prop["attested"]["by"], format!("by-the-image-{p}"), "{p}: how the attester says it holds: {s}");
    }
    let f = d.join("sandbox.json");
    std::fs::write(&f, s.to_string()).unwrap();
    let val = delulu(&d, &["schema", "validate", "sandbox", f.to_str().unwrap(), "--json"]);
    let val: serde_json::Value = serde_json::from_slice(&val.stdout).unwrap();
    assert_eq!(val["validate"]["valid"], true, "{:#}", val["validate"]["errors"]);
    std::fs::remove_file(&made).unwrap();

    // Each way of NOT vouching for one property: refused before the program is sent, naming it.
    let mut cases: Vec<(String, Vec<String>)> = Vec::new();
    for (i, p) in PROPERTIES.iter().enumerate() {
        let mut fewer = all.clone();
        fewer.remove(i);
        cases.push((format!("{p} left out"), fewer));
    }
    // Claims that only resemble a property's name are free text, and vouch for nothing.
    for near in ["Resource_Ceiling:cgroup", "resource-ceiling:cgroup", "resource_ceilings:cgroup", "no-resource_ceiling:cgroup"] {
        let mut claims = all.clone();
        claims[4] = near.to_string();
        cases.push((format!("`{near}` in its place"), claims));
    }
    for (i, (name, claims)) in cases.iter().enumerate() {
        let missing = PROPERTIES.iter().find(|p| !claims.iter().any(|c| c.starts_with(&format!("{p}:")))).unwrap();
        let r = run(&launcher(claims), &d.join(format!("r{i}.json")));
        assert_eq!(r.status.code(), Some(2), "{name}: {}", text(&r));
        assert!(text(&r).contains("DL1408"), "{name}: {}", text(&r));
        assert!(text(&r).contains(missing), "{name}: it names what is missing: {}", text(&r));
        assert!(text(&r).contains("did not vouch for"), "{name}: it says the attester left it out: {}", text(&r));
        assert!(!String::from_utf8_lossy(&r.stdout).contains("wrote"), "{name}: the program ran: {}", text(&r));
        assert!(!made.exists(), "{name}: the program's effect happened on a boundary nobody vouched for");
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// PS-E-04's attestation binding (D-V2-89): an attester may name the launcher it MEASURED — the BLAKE3 of
/// the file, computed by the attester itself, never the host's digest echoed back — and the host checks it
/// against the launcher it started (`launcher_blake3`, PS-E-04) before the program is sent. Absent on
/// `3ee18ac`: a statement could not say which launcher it vouched for (RW 4.28's open item), and the
/// reference attester had no way to measure one. A statement for another file is refused, naming both
/// digests, and the program never runs.
#[test]
fn an_attester_that_measured_the_launcher_binds_it_and_a_statement_for_another_file_is_refused() {
    let d = lab("bind");
    program(&d);
    let (seed, public) = keygen(&d, "attester");
    let exe = exe();
    let made = d.join("out").join("made.txt");
    // The host starts `exe` (the command's first word); the attester measures the file it is told to.
    let launcher = |measured: &str| {
        format!(
            "external:{exe} sandbox attest --key {seed} --attester test-attester --guarantee g --measure-launcher {measured} \
             -- {exe} __guest --stdio-pipes"
        )
    };
    let report = d.join("bound.json");
    let r = run_attested(&d, &launcher(&exe), &public, &report);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(made.exists(), "the program's effect did not happen");
    assert!(text(&r).contains("for the launcher it measured"), "the run says the statement binds the launcher: {}", text(&r));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    let started = s["launcher_blake3"].as_str().unwrap_or_else(|| panic!("the host hashed its launcher: {s}"));
    assert_eq!(s["attestation"]["launcher_blake3"], started, "the attester's measurement is the launcher started: {s}");
    assert_eq!(s["attestation"]["verified"], true, "{s}");
    let f = d.join("sandbox.json");
    std::fs::write(&f, s.to_string()).unwrap();
    let val = delulu(&d, &["schema", "validate", "sandbox", f.to_str().unwrap(), "--json"]);
    let val: serde_json::Value = serde_json::from_slice(&val.stdout).unwrap();
    assert_eq!(val["validate"]["valid"], true, "{:#}", val["validate"]["errors"]);
    std::fs::remove_file(&made).unwrap();

    // The same attester, the same key, measuring another file: the statement is not for this launcher.
    let other = d.join("w.delulu").display().to_string().replace('\\', "/");
    let r = run_attested(&d, &launcher(&other), &public, &d.join("other.json"));
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert!(text(&r).contains("not the launcher it measured"), "{}", text(&r));
    assert!(text(&r).contains(started), "it names the launcher the host started: {}", text(&r));
    assert!(text(&r).contains("the program was never sent"), "{}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("wrote"), "the program ran: {}", text(&r));
    assert!(!made.exists(), "the program's effect happened under a statement for another launcher");
    let _ = std::fs::remove_dir_all(&d);
}
