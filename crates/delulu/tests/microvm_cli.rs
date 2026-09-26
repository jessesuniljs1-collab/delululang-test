//! PS-C-03: the microVM's lifecycle, through the real binary, on a real KVM host.
//!
//! **Gated on `cfg(delulu_kvm)`**, as criterion 8 is: these boot VMs, so they need Linux, `/dev/kvm`,
//! a Firecracker binary and a guest image. The KVM CI job builds the image, fetches the pinned VMM,
//! sets `DELULU_MICROVM_IMAGE` and `DELULU_FIRECRACKER`, and compiles with `--cfg delulu_kvm`; everywhere
//! else they are ignored, and the refusal they would otherwise meet is tested ungated
//! (`sandbox_modes_cli.rs`, `cli.rs`). A gated test that finds the host NOT provisioned fails rather
//! than passing vacuously — a gate that cannot fail is not a gate.
//!
//! The threat model's claims each test answers are named on it (T8, T10, T11, T13; T9 is criterion 8,
//! `microvm_criterion8.rs`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const GATE: &str = "boots microVMs: needs Linux, /dev/kvm, Firecracker and a guest image — run on a KVM \
                    host with RUSTFLAGS=\"--cfg delulu_kvm\", DELULU_MICROVM_IMAGE and DELULU_FIRECRACKER";

fn isolated_state() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("delulu-teststate-microvm-{}-{t}", std::process::id()));
        let _ = std::fs::create_dir_all(d.join("audit"));
        d
    })
    .clone()
}

fn tmp(name: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-mvm-{name}-{}-{t}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The host must be provisioned; a gated test that ran on an unprovisioned one would pass for the
/// wrong reason.
fn provisioned() -> (PathBuf, PathBuf) {
    let image = std::env::var_os("DELULU_MICROVM_IMAGE").map(PathBuf::from);
    let vmm = std::env::var_os("DELULU_FIRECRACKER").map(PathBuf::from);
    match (image, vmm) {
        (Some(i), Some(v)) if i.join("manifest.json").is_file() && v.is_file() => (i, v),
        _ => panic!("the microVM tests are enabled (cfg delulu_kvm) but this host is not provisioned: {GATE}"),
    }
}

fn command(dir: &Path, args: &[&str]) -> Command {
    provisioned();
    let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
    c.current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .env("DELULU_STATE_DIR", isolated_state());
    c
}

fn delulu(dir: &Path, args: &[&str]) -> Output {
    command(dir, args).output().expect("the delulu binary runs")
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn write(dir: &Path, name: &str, src: &str) {
    std::fs::write(dir.join(name), src).unwrap();
}

const HELLO: &str = "module m\n\nfn main(root: Root) ! {Write} {\n  root.console().println(\"hello from the guest\")\n}\n";

/// The VM directories that belong to host process `pid`.
fn vm_dirs_of(pid: u32) -> Vec<PathBuf> {
    let prefix = format!("delulu-vm-{pid}-");
    std::fs::read_dir(std::env::temp_dir())
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default()
}

/// Process ids of every Firecracker whose command line names a VM directory of host `pid`.
fn vmms_of(pid: u32) -> Vec<u32> {
    let needle = format!("delulu-vm-{pid}-");
    let mut found = Vec::new();
    for e in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        let Ok(p) = e.file_name().to_string_lossy().parse::<u32>() else { continue };
        let cmdline = std::fs::read(e.path().join("cmdline")).unwrap_or_default();
        if String::from_utf8_lossy(&cmdline).contains(&needle) {
            found.push(p);
        }
    }
    found
}

#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn a_program_runs_in_the_microvm_and_the_report_says_level_2() {
    let dir = tmp("hello");
    write(&dir, "hello.delulu", HELLO);
    let o = delulu(&dir, &["run", "hello.delulu", "--isolation", "microvm", "--grant", "console", "--report-out", "rep.json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    // Printed by the HOST, on the guest's request: the guest has no console of its own to print on.
    assert!(String::from_utf8_lossy(&o.stdout).contains("hello from the guest"), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("rep.json")).unwrap()).unwrap();
    let sb = &v["sandbox"];
    assert_eq!(sb["backend"], "microvm", "{v}");
    assert_eq!(sb["level"], 2, "{v}");
    assert_eq!(sb["requested_level"], 2, "{v}");
    let g: Vec<&str> = sb["host_guarantees"].as_array().unwrap().iter().filter_map(|x| x.as_str()).collect();
    for word in ["a separate guest kernel under KVM", "no network device", "no filesystem device", "no network stack in its kernel"] {
        assert!(g.contains(&word), "`{word}` missing from {g:?}");
    }
    assert_eq!(sb["posture"]["network"], "only the channel", "{v}");
    assert_eq!(sb["posture"]["filesystem_reads"], "none of the host's: the guest has no filesystem device", "{v}");
    // Identity separation needs the jailer, which is not applied: said, not hidden.
    assert!(sb["limitations"].as_array().unwrap().iter().any(|l| l == "identity_separation"), "{v}");
    // T13's second half: the report and the launch record name the image that was booted, by the
    // hashes its manifest holds.
    let (image, _) = provisioned();
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(image.join("manifest.json")).unwrap()).unwrap();
    for part in ["kernel", "initramfs"] {
        let want = manifest[part]["sha256"].as_str().unwrap();
        assert_eq!(sb["image"][format!("{part}_sha256")], want, "{part} in the report: {v}");
    }
    let audit = delulu(&dir, &["audit", "query", "--json"]);
    let text = String::from_utf8_lossy(&audit.stdout);
    let launches: Vec<serde_json::Value> = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|a| a.as_array().cloned().or_else(|| a["records"].as_array().cloned()))
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.to_string().contains("sandbox-launch"))
        .collect();
    let last = launches.last().unwrap_or_else(|| panic!("no sandbox-launch record: {text}")).to_string();
    for part in ["kernel", "initramfs"] {
        assert!(last.contains(manifest[part]["sha256"].as_str().unwrap()), "{part} hash missing from the launch record: {last}");
    }
}

/// Nothing is left behind by a run that ended normally: its VMM is reaped and its directory removed
/// before the command returns (T10's ordinary path; the killed-host path is below).
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn a_finished_run_leaves_no_vmm_and_no_directory() {
    let dir = tmp("clean");
    write(&dir, "hello.delulu", HELLO);
    let host = command(&dir, &["run", "hello.delulu", "--isolation", "microvm", "--grant", "console"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = host.id();
    let o = host.wait_with_output().unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(vmms_of(pid).is_empty(), "a VMM outlived the run that started it");
    assert!(vm_dirs_of(pid).is_empty(), "the run left its VM directory behind");
}

/// T13: the image launched is the pinned one. A single flipped byte — in the initramfs, then in the
/// kernel — is refused BEFORE boot, with the file named and both hashes given.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn a_flipped_byte_in_the_image_is_refused_before_boot() {
    let (image, _) = provisioned();
    for file in ["initramfs.cpio", "vmlinux"] {
        let dir = tmp("flipped");
        let copy = dir.join("image");
        std::fs::create_dir_all(&copy).unwrap();
        for f in ["manifest.json", "initramfs.cpio", "vmlinux"] {
            std::fs::copy(image.join(f), copy.join(f)).unwrap();
        }
        let mut bytes = std::fs::read(copy.join(file)).unwrap();
        let at = bytes.len() / 2;
        bytes[at] ^= 0x01;
        std::fs::write(copy.join(file), &bytes).unwrap();
        write(&dir, "hello.delulu", HELLO);
        let o = command(&dir, &["run", "hello.delulu", "--isolation", "microvm", "--grant", "console"])
            .env("DELULU_MICROVM_IMAGE", &copy)
            .output()
            .unwrap();
        assert_ne!(o.status.code(), Some(0), "{file}: a tampered image booted: {}", out(&o));
        let text = out(&o);
        assert!(text.contains("does not match its manifest") && text.contains("refused before boot"), "{file}: {text}");
        assert!(!text.contains("hello from the guest"), "{file}: the program ran: {text}");
    }
}

/// T8: the processor-time ceiling ends a guest that spins, and the host reports a failed run rather
/// than hanging or succeeding. `--limits cpu=3` under `hostile-agent`.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn the_processor_time_ceiling_ends_a_spinning_guest() {
    let dir = tmp("spin");
    write(&dir, "spin.delulu", "module m\n\nfn main(root: Root) {\n  var i = 0\n  while i >= 0 {\n    i = i + 1\n  }\n}\n");
    let started = Instant::now();
    let o = delulu(&dir, &["run", "spin.delulu", "--isolation", "microvm", "--sandbox-profile", "hostile-agent", "--limits", "cpu=3"]);
    let took = started.elapsed();
    assert_ne!(o.status.code(), Some(0), "a spinning guest cannot succeed: {}", out(&o));
    assert!(took < Duration::from_secs(45), "the ceiling did not end it: {took:?}: {}", out(&o));
}

/// T8: a guest that exhausts its VM's memory ends its VM, not the host — and the host says the run
/// failed.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn a_guest_that_exhausts_its_memory_ends_its_vm_not_the_host() {
    let dir = tmp("oom");
    write(
        &dir,
        "grow.delulu",
        "module m\n\nfn main(root: Root) {\n  let l: ref List[Int] = []\n  var i = 0\n  while i >= 0 {\n    l.push(i)\n    i = i + 1\n  }\n}\n",
    );
    let started = Instant::now();
    let o = delulu(&dir, &["run", "grow.delulu", "--isolation", "microvm", "--sandbox-profile", "hostile-agent", "--limits", "cpu=30"]);
    assert_ne!(o.status.code(), Some(0), "{}", out(&o));
    assert!(started.elapsed() < Duration::from_secs(90), "{}", out(&o));
    // The host is still here to say so, and the next run is unaffected.
    write(&dir, "hello.delulu", HELLO);
    let o = delulu(&dir, &["run", "hello.delulu", "--isolation", "microvm", "--grant", "console"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
}

/// T10 and the orphan sweep: a host killed mid-run takes its VMM with it (`PR_SET_PDEATHSIG`), and the
/// directory it could not clean up is removed by the next launch.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn a_killed_host_takes_its_vm_with_it_and_the_next_launch_sweeps_what_is_left() {
    let dir = tmp("host-death");
    write(&dir, "spin.delulu", "module m\n\nfn main(root: Root) {\n  var i = 0\n  while i >= 0 {\n    i = i + 1\n  }\n}\n");
    let mut host = command(&dir, &["run", "spin.delulu", "--isolation", "microvm"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let pid = host.id();
    // Wait for its VMM to exist.
    let until = Instant::now() + Duration::from_secs(20);
    let vmms = loop {
        let v = vmms_of(pid);
        if !v.is_empty() {
            break v;
        }
        assert!(Instant::now() < until, "no VMM appeared for host {pid}");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(vm_dirs_of(pid).len(), 1, "one VM directory while it runs");
    host.kill().unwrap();
    host.wait().unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    while vmms.iter().any(|p| Path::new(&format!("/proc/{p}")).exists()) {
        assert!(Instant::now() < until, "the VMM {vmms:?} outlived its host");
        std::thread::sleep(Duration::from_millis(20));
    }
    // The host was killed before it could clean up, so its directory is still there...
    assert_eq!(vm_dirs_of(pid).len(), 1, "the killed host's directory");
    // ...until the next launch, which sweeps it.
    write(&dir, "hello.delulu", HELLO);
    let o = delulu(&dir, &["run", "hello.delulu", "--isolation", "microvm", "--grant", "console"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(vm_dirs_of(pid).is_empty(), "the orphaned directory was not swept");
}

/// T11: two VMs at once, each with its own directory and channel; each program's output reaches its
/// own host and no other.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn two_vms_at_once_do_not_share() {
    let dir = tmp("two");
    for n in ["one", "two"] {
        write(
            &dir,
            &format!("{n}.delulu"),
            &format!("module m\n\nfn main(root: Root) ! {{Write}} {{\n  root.console().println(\"i am {n}\")\n}}\n"),
        );
    }
    let a = command(&dir, &["run", "one.delulu", "--isolation", "microvm", "--grant", "console"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let b = command(&dir, &["run", "two.delulu", "--isolation", "microvm", "--grant", "console"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let (a, b) = (a.wait_with_output().unwrap(), b.wait_with_output().unwrap());
    assert_eq!(a.status.code(), Some(0), "{}", out(&a));
    assert_eq!(b.status.code(), Some(0), "{}", out(&b));
    let (sa, sb) = (String::from_utf8_lossy(&a.stdout), String::from_utf8_lossy(&b.stdout));
    assert!(sa.contains("i am one") && !sa.contains("i am two"), "{sa}");
    assert!(sb.contains("i am two") && !sb.contains("i am one"), "{sb}");
}

/// PS-A-07's rule at L2: `sandbox probe` reports L2 from a BOOT, not from a sentence.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn the_probe_boots_a_vm_to_report_l2() {
    let dir = tmp("probe");
    let o = delulu(&dir, &["sandbox", "probe", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).unwrap();
    let l2 = v["levels"].as_array().unwrap().iter().find(|l| l["level"] == 2).unwrap().clone();
    assert_eq!(l2["available"], true, "{l2}");
    let launch = l2["attempts"].as_array().unwrap().iter().find(|a| a["what"] == "the L2 guest launch").unwrap().clone();
    assert!(launch["detail"].as_str().unwrap().contains("booted a guest"), "{launch}");
    assert!(v["highest_available"].as_u64().unwrap() >= 2, "{v}");

    // The control, without which the sentence above proves nothing: a "VMM" that answers `--version`
    // exactly as Firecracker does and cannot boot anything. Every attempt before the launch passes,
    // so only a probe that really launches can tell — and it must say the launch failed. (Unix-only
    // code in a file every platform compiles; the test itself only runs on a KVM host.)
    #[cfg(unix)]
    probe_control(&dir);
}

#[cfg(unix)]
fn probe_control(dir: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    let fake = dir.join("firecracker");
    std::fs::write(&fake, "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'Firecracker v1.17.0'; exit 0; fi\nexit 1\n").unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    let o = command(dir, &["sandbox", "probe", "--json"]).env("DELULU_FIRECRACKER", &fake).output().unwrap();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).unwrap();
    let l2 = v["levels"].as_array().unwrap().iter().find(|l| l["level"] == 2).unwrap().clone();
    let attempts = l2["attempts"].as_array().unwrap();
    assert!(attempts.iter().filter(|a| a["what"] != "the L2 guest launch").all(|a| a["ok"] == true), "{l2}");
    let launch = attempts.iter().find(|a| a["what"] == "the L2 guest launch").unwrap();
    assert_eq!(launch["ok"], false, "a VMM that cannot boot was reported as booting: {l2}");
    assert_eq!(l2["available"], false, "{l2}");
}

/// The command as root: directly when this test already is, otherwise through `sudo -n` (the CI runner
/// has passwordless sudo). The environment the run needs is passed explicitly, because sudo resets it.
#[cfg(unix)]
fn as_root(dir: &Path, program: &Path, args: &[&str]) -> Command {
    let (image, vmm) = provisioned();
    // SAFETY-free: reading the effective uid through `id -u` keeps this test free of `libc`.
    let root = Command::new("id").arg("-u").output().map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0").unwrap_or(false);
    let env = [
        format!("DELULU_MICROVM_IMAGE={}", image.display()),
        format!("DELULU_FIRECRACKER={}", vmm.display()),
        "DELULU_NO_FIRST_RUN=1".to_string(),
        "DELULU_NO_COLOR=1".to_string(),
        format!("DELULU_STATE_DIR={}", dir.join("root-state").display()),
    ];
    let mut c = if root { Command::new("env") } else {
        let mut s = Command::new("sudo");
        s.arg("-n").arg("env");
        s
    };
    c.current_dir(dir).args(&env).arg(program).args(args);
    c
}

/// The process ids whose command line carries a jailer id beginning `delulu-`.
#[cfg(unix)]
fn jailed_vmms() -> Vec<(u32, String)> {
    let mut found = Vec::new();
    for e in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        let Ok(p) = e.file_name().to_string_lossy().parse::<u32>() else { continue };
        let cmdline = std::fs::read(e.path().join("cmdline")).unwrap_or_default();
        let args: Vec<String> = cmdline.split(|b| *b == 0).map(|a| String::from_utf8_lossy(a).into_owned()).collect();
        if let Some(i) = args.iter().position(|a| a == "--id") {
            if let Some(id) = args.get(i + 1).filter(|id| id.starts_with("delulu-")) {
                if args.first().is_some_and(|a| a.ends_with("firecracker")) {
                    found.push((p, id.clone()));
                }
            }
        }
    }
    found
}

/// PS-C-03b: run as root, the VMM runs under Firecracker's jailer — a uid of its own, in a chroot —
/// the report says so and drops the identity limitation, and a host killed mid-run takes the VMM with
/// it through the reaper, because the jailer's `setuid` cleared the death signal. Root WITHOUT a jailer
/// is refused. Needs root: the test runs the host as root itself, directly or through `sudo -n`.
#[test]
#[cfg(unix)]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs as root; see GATE")]
fn as_root_the_vmm_is_jailed_as_its_own_uid_and_dies_with_its_host() {
    let (_, vmm) = provisioned();
    assert!(vmm.with_file_name("jailer").is_file(), "the jailer must sit beside {}", vmm.display());
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_delulu"));
    let dir = tmp("jailed");
    std::fs::create_dir_all(dir.join("root-state/audit")).unwrap();
    write(&dir, "hello.delulu", HELLO);

    // 1. A jailed run, and what its report says.
    let o = as_root(&dir, &exe, &["run", "hello.delulu", "--isolation", "microvm", "--grant", "console", "--report-out", "rep.json"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(String::from_utf8_lossy(&o.stdout).contains("hello from the guest"), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("rep.json")).unwrap()).unwrap();
    let sb = &v["sandbox"];
    let g: Vec<&str> = sb["host_guarantees"].as_array().unwrap().iter().filter_map(|x| x.as_str()).collect();
    assert!(g.contains(&"a uid of its own for the VMM") && g.contains(&"a chroot for the VMM"), "{g:?}");
    assert_eq!(sb["posture"]["identity"], "the VMM: a uid of its own, in a chroot (Firecracker's jailer)", "{v}");
    assert!(!sb["limitations"].as_array().unwrap().iter().any(|l| l == "identity_separation"), "{v}");

    // 2. While one runs: its VMM's uid is a reserved one, and it is chrooted.
    write(&dir, "spin.delulu", "module m\n\nfn main(root: Root) {\n  var i = 0\n  while i >= 0 {\n    i = i + 1\n  }\n}\n");
    let before: Vec<u32> = jailed_vmms().into_iter().map(|(p, _)| p).collect();
    let mut sudo = as_root(&dir, &exe, &["run", "spin.delulu", "--isolation", "microvm"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(20);
    let (vmm_pid, id) = loop {
        if let Some(v) = jailed_vmms().into_iter().find(|(p, _)| !before.contains(p)) {
            break v;
        }
        assert!(Instant::now() < until, "no jailed VMM appeared");
        std::thread::sleep(Duration::from_millis(50));
    };
    let status = std::fs::read_to_string(format!("/proc/{vmm_pid}/status")).unwrap();
    let uid: u32 = status.lines().find(|l| l.starts_with("Uid:")).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
    assert!((900_000..965_536).contains(&uid), "the VMM runs as uid {uid}, not a reserved one");
    // What the VMM can see of the filesystem is its jail and nothing else. (Not `readlink` of its root:
    // the jailer pivots into the chroot inside a new mount namespace, so from outside the root reads as
    // `/` — true in the VMM's own namespace, and no evidence either way. The first version of this test
    // asserted on that and failed on a correctly jailed VMM.)
    let seen = as_root(&dir, Path::new("ls"), &["-A", &format!("/proc/{vmm_pid}/root/")]).output().unwrap();
    let seen: Vec<String> = String::from_utf8_lossy(&seen.stdout).lines().map(str::to_string).collect();
    for inside in ["firecracker", "vmlinux", "initramfs.cpio", "dev"] {
        assert!(seen.iter().any(|e| e == inside), "`{inside}` is not in the VMM's view: {seen:?}");
    }
    for host in ["etc", "home", "usr", "proc", "root"] {
        assert!(!seen.iter().any(|e| e == host), "the VMM can see the host's `/{host}`: {seen:?}");
    }
    let ns = |pid: &str| {
        let o = as_root(&dir, Path::new("readlink"), &[&format!("/proc/{pid}/ns/mnt")]).output().unwrap();
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    };
    assert_ne!(ns(&vmm_pid.to_string()), ns("self"), "the VMM shares this host's mount namespace");
    let _ = &id;

    // 3. Kill the HOST (the delulu process sudo started), not the VMM: the reaper must end the VMM.
    let host_pid = std::fs::read_dir("/proc")
        .unwrap()
        .flatten()
        .filter_map(|e| e.file_name().to_string_lossy().parse::<u32>().ok())
        .find(|p| {
            let stat = std::fs::read_to_string(format!("/proc/{p}/stat")).unwrap_or_default();
            let cmd = std::fs::read(format!("/proc/{p}/cmdline")).unwrap_or_default();
            // The delulu `run` whose id the jail carries: `delulu-<host pid>-…`.
            id.starts_with(&format!("delulu-{p}-")) && !stat.is_empty() && !cmd.is_empty()
        })
        .expect("the host that owns the jail");
    let k = as_root(&dir, Path::new("kill"), &["-9", &host_pid.to_string()]).output().unwrap();
    assert!(k.status.success(), "could not kill the host: {}", out(&k));
    let killed = Instant::now();
    // Two seconds, not more: the reaper ends the VMM in milliseconds. The first version of this test
    // allowed eight, and a mutant reaper that killed nothing SURVIVED it — the VM was ending ~5 s after
    // its host anyway, through its console pipe breaking. That is incidental: with the console pipe held
    // open (measured, PS-C-06) a jailed VMM whose host and reaper were both killed was still running
    // after fifteen seconds, bounded only by its processor-time ceiling. Only the reaper is prompt.
    let until = killed + Duration::from_secs(2);
    while Path::new(&format!("/proc/{vmm_pid}")).exists() {
        assert!(
            Instant::now() < until,
            "the jailed VMM {vmm_pid} outlived its host by {:?} — the reaper did not end it",
            killed.elapsed()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = sudo.wait();
    let jail = format!("/srv/delulu-jailer/firecracker/{id}");
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let still = as_root(&dir, Path::new("test"), &["-e", &jail]).status().unwrap().success();
        if !still {
            break;
        }
        assert!(Instant::now() < until, "the reaper left the jail `{jail}` behind");
        std::thread::sleep(Duration::from_millis(50));
    }

    // 4. Root without a jailer is refused, not run as root.
    let alone = dir.join("fc-alone");
    std::fs::create_dir_all(&alone).unwrap();
    std::fs::copy(&vmm, alone.join("firecracker")).unwrap();
    let mut c = as_root(&dir, Path::new("env"), &[
        &format!("DELULU_FIRECRACKER={}", alone.join("firecracker").display()),
        "PATH=/usr/bin:/bin",
        exe.to_str().unwrap(),
        "run", "hello.delulu", "--isolation", "microvm", "--grant", "console",
    ]);
    let o = c.output().unwrap();
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(out(&o).contains("DL1408") && out(&o).contains("jailer"), "{}", out(&o));
    assert!(!out(&o).contains("hello from the guest"), "root ran a VMM without a jailer: {}", out(&o));
}

/// A program whose surface the channel does not carry is refused before a VM is booted, as it is
/// under `--sandbox`.
#[test]
#[cfg_attr(not(delulu_kvm), ignore = "boots microVMs; see GATE")]
fn an_uncarried_surface_is_refused_before_boot() {
    let dir = tmp("actors");
    write(
        &dir,
        "a.delulu",
        "module m\n\nactor A {\n  var n: Int\n  new() { self.n = 0 }\n  be go() { self.n = 1 }\n}\n\nfn main(root: Root) ! {Async} {\n  let a = spawn A()\n  a.go()\n}\n",
    );
    let o = delulu(&dir, &["run", "a.delulu", "--isolation", "microvm"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("cannot carry this program yet") && out(&o).contains("actors"), "{}", out(&o));
}
