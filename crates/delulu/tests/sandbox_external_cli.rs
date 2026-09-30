//! PS-D-01 (L3): `--sandbox-backend external:CMD` — the operator's launcher runs the guest in THEIR
//! environment (Docker + gVisor, Kata, a cloud sandbox, `ssh`) and carries the channel on its standard
//! input and output. These tests use the `delulu` binary itself as the launcher
//! (`external:<delulu> __guest --stdio-pipes`), which is the whole protocol with no container in the way.
//!
//! What must hold, whatever launcher is used:
//! - the guest still holds no authority: every effect is decided and performed by the host, under the
//!   grants — an ungranted one is refused, a granted one happens;
//! - the report claims NOTHING DeluluLang did not measure: level 3, backend `external`, no host guarantee,
//!   `fully_enforced: false`, and the launcher's program named (never its arguments);
//! - a launcher that dies, or a command that names no launcher, fails legibly rather than hanging.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
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
    let d = std::env::temp_dir().join(format!("dext-{tag}-{}-{}-{n}", std::process::id(), t % 1_000_000_000));
    std::fs::create_dir_all(d.join("out")).unwrap();
    std::fs::create_dir_all(d.join("s")).unwrap();
    d
}

/// The binary as a launcher: its own path, then the guest's words.
fn self_launcher() -> String {
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    assert!(!exe.contains(' '), "the launcher is split on whitespace; this checkout's path has a space: {exe}");
    format!("external:{exe} __guest --stdio-pipes")
}

#[test]
fn an_external_guest_runs_under_the_hosts_grants_and_the_report_claims_nothing_unmeasured() {
    let d = lab("run");
    let out = d.join("out").display().to_string().replace('\\', "/");
    std::fs::write(
        d.join("w.delulu"),
        format!(
            "module w\n\nfn main(root: Root) ! {{Write}} {{\n    let o = root.console()\n    let fw = root.fs_write(\"{out}\")\n    \
             match fw.write_text(\"made.txt\", \"by the host\") {{\n        Ok(_) => o.println(\"wrote\"),\n        Err(_) => o.println(\"refused\")\n    }}\n}}\n"
        ),
    )
    .unwrap();
    let launcher = self_launcher();
    let report = d.join("r.json");
    let args = [
        "run", "w.delulu", "--grant", "console", "--grant", &format!("fs.write={out}"), "--sandbox",
        "--sandbox-backend", &launcher, "--report-out", report.to_str().unwrap(),
    ];
    let r = delulu(&d, &d.join("s"), &args);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("wrote"), "{}", text(&r));
    assert!(text(&r).contains("DeluluLang measured none of it"), "the run says whose boundary it is: {}", text(&r));
    assert_eq!(std::fs::read_to_string(d.join("out").join("made.txt")).unwrap(), "by the host");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    assert_eq!(s["backend"], "external", "{s}");
    assert_eq!(s["level"], 3, "{s}");
    assert_eq!(s["requested_level"], 3, "{s}");
    assert_eq!(s["fully_enforced"], false, "an external boundary is never reported as enforced: {s}");
    // Nothing here was measured by the HOST, so nothing is claimed as its guarantee: `granted` is "none"
    // and `host_guarantees` empty. What the guest says it applied to itself (Landlock and seccomp, on
    // Linux) is reported as the guest's word, `guest_reported` — RW 4.31, the red-team pass's F7.
    assert_eq!(s["granted"], "none", "{s}");
    assert_eq!(s["host_guarantees"].as_array().unwrap().len(), 0, "no HOST guarantee is claimed: {s}");
    let launcher_named = s["launcher"].as_str().unwrap();
    assert!(launcher_named.ends_with("delulu") || launcher_named.ends_with("delulu.exe"), "{s}");
    assert!(!launcher_named.contains("__guest"), "the launcher's ARGUMENTS are never recorded: {s}");

    // The host still decides: an effect the run did not grant is refused, and nothing happens.
    std::fs::write(d.join("n.delulu"), "module n\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"never\")\n}\n").unwrap();
    let r = delulu(&d, &d.join("s"), &["run", "n.delulu", "--sandbox", "--sandbox-backend", &launcher]);
    assert_ne!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("DL0703"), "an ungranted effect is refused by the host: {}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("never"), "{}", text(&r));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_backend_that_is_not_one_or_a_launcher_that_dies_fails_legibly() {
    let d = lab("refuse");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let st = d.join("s");
    for (args, says) in [
        (vec!["--sandbox-backend", "external:x"], "describes a sandboxed run"),
        (vec!["--sandbox", "--sandbox-backend", "docker"], "is not a backend this command knows"),
        (vec!["--sandbox", "--sandbox-backend", "external:"], "needs the launcher's command"),
        (vec!["--sandbox", "--sandbox-backend", "external:x", "--isolation", "microvm"], "two different boundaries"),
    ] {
        let mut a = vec!["run", "h.delulu", "--grant", "console"];
        a.extend(args.iter().copied());
        let r = delulu(&d, &st, &a);
        assert_eq!(r.status.code(), Some(2), "{args:?}: {}", text(&r));
        assert!(text(&r).contains(says), "{args:?}: {}", text(&r));
        assert!(!text(&r).contains("hi\n"), "{args:?}: nothing ran");
    }
    // A launcher that does not exist, and one that exits without ever running a guest: both end the run
    // with a reason, promptly — never a hang, never a success.
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    for launcher in ["external:no-such-launcher-anywhere".to_string(), format!("external:{exe} --version")] {
        let t = std::time::Instant::now();
        let r = delulu(&d, &st, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &launcher]);
        assert_ne!(r.status.code(), Some(0), "{launcher}: {}", text(&r));
        assert!(t.elapsed() < std::time::Duration::from_secs(50), "{launcher}: refused promptly, not at the channel deadline");
        assert!(!text(&r).contains("os error"), "{launcher}: in words: {}", text(&r));
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// The launcher is told the guest's words and the limits the run asked for, so a recipe can map them
/// (`docker --memory`); enforcing them is the launcher's.
#[cfg(unix)]
#[test]
fn the_launcher_is_told_the_guest_words_and_the_limits() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("env");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let script = d.join("launch.sh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$DELULU_GUEST_ARGS|$DELULU_LIMIT_MEMORY_BYTES|$DELULU_LIMIT_CPU_SECONDS|$DELULU_LIMIT_WALL_SECONDS\" > {}/env.txt\nexec {} $DELULU_GUEST_ARGS\n",
            d.display(),
            env!("CARGO_BIN_EXE_delulu")
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let r = delulu(
        &d,
        &d.join("s"),
        &["run", "h.delulu", "--grant", "console", "--sandbox", "--limits", "mem=268435456,cpu=7,wall=30", "--sandbox-backend", &format!("external:{}", script.display())],
    );
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert_eq!(std::fs::read_to_string(d.join("env.txt")).unwrap().trim(), "__guest --stdio-pipes|268435456|7|30");
    let _ = std::fs::remove_dir_all(&d);
}

// ===== PS-E-04 · the launcher resolved once, hashed, and pinnable ===============================
//
// `launch_external` handed the command's first word to the operating system, which looks a bare name
// up on `PATH` at spawn — a relative entry included — and the report named the word, never the bytes.
// ADAPTER-SPELL-1 was the same shape for hardware drivers (D-V2-50). `V2_OPENSHELL_STUDY.md` §4.4.

/// BLAKE3 of a file, computed here, independently of the host that reports it.
fn blake3_of(p: &Path) -> String {
    let mut h = blake3::Hasher::new();
    h.update_reader(std::fs::File::open(p).unwrap()).unwrap();
    h.finalize().to_hex().to_string()
}

fn hello(d: &Path) {
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
}

/// The report and the launch record name the file the launcher's word resolved to, and its digest —
/// the digest of the bytes, not of the name.
#[test]
fn the_report_names_the_launchers_file_and_its_digest() {
    let d = lab("digest");
    hello(&d);
    std::fs::create_dir_all(d.join("s").join("audit")).unwrap();
    let report = d.join("r.json");
    let r = delulu(
        &d,
        &d.join("s"),
        &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &self_launcher(), "--report-out", report.to_str().unwrap()],
    );
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    let exe = Path::new(env!("CARGO_BIN_EXE_delulu"));
    let path = s["launcher_path"].as_str().unwrap_or_else(|| panic!("the report names the launcher's file: {s}"));
    assert_eq!(Path::new(path), exe, "{s}");
    assert_eq!(s["launcher_blake3"], blake3_of(exe), "the digest is the file's, computed independently: {s}");
    let q = delulu(&d, &d.join("s"), &["audit", "query", "--json"]);
    let chain: serde_json::Value = serde_json::from_slice(&q.stdout).unwrap_or_else(|_| panic!("{}", text(&q)));
    let launch = chain["records"].as_array().unwrap().iter().find(|r| r["action"] == "sandbox-launch").cloned();
    let launch = launch.unwrap_or_else(|| panic!("a launch record: {chain}"));
    assert_eq!(launch["authority"]["launcher_blake3"], blake3_of(exe), "the launch record names it too: {launch}");
    let _ = std::fs::remove_dir_all(&d);
}

/// `--launcher-digest HEX` pins the launcher: another file refuses before anything starts, the pinned
/// one runs; a pin that is not a digest, or a pin with no external launcher, is refused in words.
#[test]
fn a_pinned_launcher_runs_only_as_the_file_it_pins() {
    let d = lab("pin");
    hello(&d);
    let st = d.join("s");
    std::fs::create_dir_all(st.join("audit")).unwrap();
    let launcher = self_launcher();
    let exe = Path::new(env!("CARGO_BIN_EXE_delulu"));
    let wrong = blake3::hash(b"not the launcher").to_hex().to_string();
    let r = delulu(&d, &st, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &launcher, "--launcher-digest", &wrong]);
    assert_eq!(r.status.code(), Some(1), "refused in words, as an attester's refusal is: {}", text(&r));
    assert!(text(&r).contains("not the pinned launcher"), "{}", text(&r));
    assert!(text(&r).contains(&blake3_of(exe)) && text(&r).contains(&wrong), "both digests are named: {}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("hi"), "nothing ran: {}", text(&r));
    // The refusal is evidence: a launcher that changed under a pin is in the chain, and nothing launched.
    let q = delulu(&d, &st, &["audit", "query", "--json"]);
    let chain: serde_json::Value = serde_json::from_slice(&q.stdout).unwrap_or_else(|_| panic!("{}", text(&q)));
    let records = chain["records"].as_array().unwrap_or_else(|| panic!("{chain}"));
    assert!(
        records.iter().any(|r| r["action"] == "sandbox-launcher"
            && r["decision"] == "deny"
            && r["authority"]["pinned"] == wrong.as_str()
            && r["authority"]["launcher_blake3"] == blake3_of(exe).as_str()),
        "the refused launcher is recorded: {chain}"
    );
    assert!(!records.iter().any(|r| r["action"] == "sandbox-launch"), "and nothing was launched: {chain}");

    let right = blake3_of(exe);
    let r = delulu(&d, &st, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &launcher, &format!("--launcher-digest={}", right.to_uppercase())]);
    assert_eq!(r.status.code(), Some(0), "the pinned file runs (a pin's case does not matter): {}", text(&r));
    assert!(String::from_utf8_lossy(&r.stdout).contains("hi"), "{}", text(&r));

    for (args, says) in [
        (vec!["--sandbox", "--sandbox-backend", launcher.as_str(), "--launcher-digest", "abc"], "is not a BLAKE3 digest"),
        (vec!["--sandbox", "--launcher-digest", right.as_str()], "is for an external launcher"),
        (vec!["--launcher-digest", right.as_str()], "describes a sandboxed run"),
    ] {
        let mut a = vec!["run", "h.delulu", "--grant", "console"];
        a.extend(args.iter().copied());
        let r = delulu(&d, &st, &a);
        assert_eq!(r.status.code(), Some(2), "{args:?}: {}", text(&r));
        assert!(text(&r).contains(says), "{args:?}: {}", text(&r));
        assert!(!String::from_utf8_lossy(&r.stdout).contains("hi"), "{args:?}: nothing ran");
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[cfg(unix)]
fn script(p: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::write(p, body).unwrap();
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A launcher the pin refuses is never STARTED — not started and then ended.
#[cfg(unix)]
#[test]
fn a_launcher_the_pin_refuses_is_never_started() {
    let d = lab("never");
    hello(&d);
    let lnch = d.join("lnch");
    script(&lnch, &format!("#!/bin/sh\ntouch {}/STARTED\nexec {} $DELULU_GUEST_ARGS\n", d.display(), env!("CARGO_BIN_EXE_delulu")));
    let run = |pin: &str| {
        delulu(&d, &d.join("s"), &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &format!("external:{}", lnch.display()), "--launcher-digest", pin])
    };
    let r = run(&blake3::hash(b"another launcher").to_hex());
    assert_ne!(r.status.code(), Some(0), "{}", text(&r));
    assert!(!d.join("STARTED").exists(), "a launcher that is not the pinned file was started: {}", text(&r));
    let r = run(&blake3_of(&lnch));
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(d.join("STARTED").exists(), "the pinned launcher runs: {}", text(&r));
    let _ = std::fs::remove_dir_all(&d);
}

/// A bare launcher name is looked up in PATH's ABSOLUTE directories, by DeluluLang: an empty or relative
/// entry — `.` ahead of the operator's directory — means the working directory to the operating system's
/// own search, and a `lnch` planted there must not be what runs.
#[cfg(unix)]
#[test]
fn a_bare_launcher_name_never_means_a_file_in_the_working_directory() {
    let d = lab("plant");
    hello(&d);
    let exe = env!("CARGO_BIN_EXE_delulu");
    let good = d.join("good");
    std::fs::create_dir_all(&good).unwrap();
    script(&good.join("lnch"), &format!("#!/bin/sh\ntouch {}/GOOD-RAN\nexec {exe} $DELULU_GUEST_ARGS\n", d.display()));
    script(&d.join("lnch"), &format!("#!/bin/sh\ntouch {}/PLANTED-RAN\nexec {exe} $DELULU_GUEST_ARGS\n", d.display()));
    let path = format!(".:{}:{}", good.display(), std::env::var("PATH").unwrap_or_default());
    let report = d.join("r.json");
    let r = Command::new(exe)
        .current_dir(&d)
        .env("DELULU_STATE_DIR", d.join("s"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .env("PATH", &path)
        .args(["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", "external:lnch", "--report-out", report.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!d.join("PLANTED-RAN").exists(), "the file planted in the working directory ran: {}", text(&r));
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(d.join("GOOD-RAN").exists(), "{}", text(&r));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(Path::new(v["sandbox"]["launcher_path"].as_str().unwrap_or_default()), good.join("lnch"), "{v}");
    let _ = std::fs::remove_dir_all(&d);
}

/// On Linux the file started is the file that was hashed (`fexecve` of the descriptor the digest was
/// read from): a path swapped between the hash and the start is not what runs. The window is held open
/// by a launcher long enough to take a while to hash, and the swap is made the moment the host is seen
/// holding the launcher open — never by re-running until a race shows (HANDOFF §11.5).
#[cfg(target_os = "linux")]
#[test]
fn the_launcher_started_is_the_file_that_was_hashed() {
    use std::io::Write as _;
    let d = lab("swap");
    hello(&d);
    let exe = env!("CARGO_BIN_EXE_delulu");
    let lnch = d.join("lnch");
    let body = |who: &str| format!("#!/bin/sh\necho {who} >> {}/ran\nexec {exe} $DELULU_GUEST_ARGS\n", d.display());
    // A: the pinned file, its tail a comment the shell never reaches (it `exec`s first), 64 MiB long.
    let mut a = body("A").into_bytes();
    a.extend(std::iter::repeat_n(b'#', 64 << 20));
    a.push(b'\n');
    {
        let mut f = std::fs::File::create(&lnch).unwrap();
        f.write_all(&a).unwrap();
    }
    script(&d.join("b"), &body("B"));
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&lnch, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let pin = blake3::hash(&a).to_hex().to_string();
    let seen_as = std::fs::canonicalize(&lnch).unwrap();
    let mut host = Command::new(exe)
        .current_dir(&d)
        .env("DELULU_STATE_DIR", d.join("s"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &format!("external:{}", lnch.display()), "--launcher-digest", &pin])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // The moment the host holds the launcher open — it is reading it to hash it — the path is swapped.
    let fds = format!("/proc/{}/fd", host.id());
    let swapped = loop {
        if host.try_wait().unwrap().is_some() {
            break false;
        }
        let open = std::fs::read_dir(&fds)
            .map(|rd| rd.flatten().any(|e| std::fs::read_link(e.path()).is_ok_and(|t| t == seen_as)))
            .unwrap_or(false);
        if open {
            std::fs::rename(d.join("b"), &lnch).unwrap();
            break true;
        }
        std::thread::yield_now();
    };
    let out = host.wait_with_output().unwrap();
    assert!(swapped, "the host never held the launcher open before starting it, so nothing was hashed: {}", text(&out));
    let ran = std::fs::read_to_string(d.join("ran")).unwrap_or_default();
    assert_eq!(ran.trim(), "A", "A was hashed and pinned; B was swapped in under its name and must not be what ran: {}", text(&out));
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let _ = std::fs::remove_dir_all(&d);
}

/// Which processes hold `p` open, as Windows answers it (`FileProcessIdsUsingFileInformation`) — the
/// Windows witness's view of the host, as `/proc/<pid>/fd` is the Linux one's. Asked through a handle
/// that reads attributes only, which takes part in no sharing check: it cannot stop the host opening
/// the file, and the host's open cannot stop it.
#[cfg(windows)]
fn holders(p: &Path) -> Vec<usize> {
    use std::os::windows::fs::OpenOptionsExt as _;
    use std::os::windows::io::AsRawHandle as _;
    #[repr(C)]
    struct IoStatus {
        status: isize,
        information: usize,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQueryInformationFile(h: *mut std::ffi::c_void, io: *mut IoStatus, info: *mut std::ffi::c_void, len: u32, class: i32) -> i32;
    }
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const FILE_PROCESS_IDS_USING_FILE_INFORMATION: i32 = 47;
    let Ok(f) = std::fs::OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES).share_mode(7).open(p) else {
        return Vec::new();
    };
    // { ULONG count; ULONG_PTR ids[] } — the count in the first word's low half, the ids after it.
    let mut buf = vec![0usize; 1024];
    let mut io = IoStatus { status: 0, information: 0 };
    // SAFETY: a handle this function owns, a buffer of the length given, an out-parameter on the stack.
    let st = unsafe {
        NtQueryInformationFile(
            f.as_raw_handle(),
            &mut io,
            buf.as_mut_ptr().cast(),
            (buf.len() * std::mem::size_of::<usize>()) as u32,
            FILE_PROCESS_IDS_USING_FILE_INFORMATION,
        )
    };
    if st < 0 {
        return Vec::new();
    }
    let n = (buf[0] as u32 as usize).min(buf.len() - 1);
    buf[1..=n].to_vec()
}

/// PS-E-04 on Windows, which has no `fexecve`: the file started is the file that was hashed, because the
/// host holds the launcher open, sharing reads only, from the hash until the run ends — so it cannot be
/// written, or renamed or deleted away (a rename over it is a deletion), in between. The window is held
/// open by a launcher that takes a while to hash (the binary, then 64 MiB its loader never reads), and B
/// is renamed over it the moment Windows says the host holds it — then tried again for as long as the
/// host does. B is not a guest at all (it prints a line and exits), so if B is what starts, the run fails.
#[cfg(windows)]
#[test]
fn the_launcher_started_is_the_file_that_was_hashed_on_windows() {
    let d = lab("swap-win");
    hello(&d);
    let exe = env!("CARGO_BIN_EXE_delulu");
    let lnch = d.join("lnch.exe");
    let mut a = std::fs::read(exe).unwrap();
    a.extend(std::iter::repeat_n(0u8, 64 << 20));
    std::fs::write(&lnch, &a).unwrap();
    let b = d.join("b.exe");
    std::fs::copy(r"C:\Windows\System32\hostname.exe", &b).unwrap();
    let pin = blake3::hash(&a).to_hex().to_string();
    let words = format!("external:{} __guest --stdio-pipes", lnch.display().to_string().replace('\\', "/"));
    assert_eq!(words.split_whitespace().count(), 3, "the launcher's words are split on whitespace: {words}");
    let mut host = Command::new(exe)
        .current_dir(&d)
        .env("DELULU_STATE_DIR", d.join("s"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &words, "--launcher-digest", &pin])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let hpid = host.id() as usize;
    let mut seen = false;
    let mut swapped = false;
    while host.try_wait().unwrap().is_none() {
        if !seen {
            seen = holders(&lnch).contains(&hpid);
        }
        if seen && !swapped {
            swapped = std::fs::rename(&b, &lnch).is_ok();
        }
        std::thread::yield_now();
    }
    let out = host.wait_with_output().unwrap();
    assert!(seen, "the host was never seen holding the launcher open, so nothing was tried: {}", text(&out));
    assert_eq!(
        out.status.code(),
        Some(0),
        "A was hashed and pinned; B, renamed over it once the host was seen holding it (the rename {}), must not \
         be what ran: {}",
        if swapped { "succeeded" } else { "never succeeded" },
        text(&out)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("hi"), "{}", text(&out));
    let _ = std::fs::remove_dir_all(&d);
}
