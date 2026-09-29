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
//! - the words after `--` belong to the attester's command, never to `delulu`.

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
        (vec!["--grant", "console", "--", exe.as_str()], "does not know this option"),
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
