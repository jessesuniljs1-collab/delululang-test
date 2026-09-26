//! PS-C — L2, the microVM on Linux + KVM (V2 security model §6–§7).
//!
//! The guest runs the INTERPRETER (D-NE-23) as PID 1 of its own kernel under Firecracker, and holds
//! one thing: a vsock stream to its host. No network device, no filesystem device, no secret, no
//! broker address. Every effect it asks for crosses that stream as a `delulu-sandbox-channel/2`
//! request the host authorizes and performs with the code that serves an L1 guest — the same guest
//! behind a hypervisor, not a second design and not a second Authority.
//!
//! Two halves in one file, because each is the other's only caller:
//!
//! - the HOST half ([`prerequisites`], [`boot`], [`Vm`]) finds KVM, a VMM and a guest image; copies the image
//!   into a private directory while checking it against its manifest, so the bytes it verified are the
//!   bytes it boots; starts the VMM under a CPU and memory ceiling and a wall-clock watchdog; drives it
//!   over its API socket with the small HTTP/1.1 client below (no new crate); and hands `guest.rs` the
//!   stream the guest dials. A VMM that dies is noticed on the channel; a VM is never reported ended
//!   before its VMM has been reaped; its directory is removed on every path out, and one left behind
//!   by a host that was itself killed is swept at the next launch.
//! - the GUEST half ([`run_vm_guest`]) is what `/delulu` runs as PID 1 inside the VM: it measures its
//!   own kernel (no IP stack), dials the host, serves the program exactly as an L1 guest does, and
//!   powers the VM off.
//!
//! The image comes from `scripts/microvm/build-image.sh`: a kernel built from pinned kernel.org
//! source with vsock and nothing else (`scripts/microvm/kernel.config`), and an initramfs holding the
//! static guest (`scripts/microvm/mkinitramfs.py`). This repository never ships a built kernel
//! (D-NE-27, owner-reserved: distributing a GPL kernel is a licensing act).
//!
//! Not yet here, and said wherever it matters: the jailer (a per-VM uid and chroot needs root; the VMM
//! runs as the operator, under Firecracker's own seccomp filters, and the report names
//! `identity_separation` as a limitation), and attestation (L4).
//!
//! Compiled only on Linux (`main.rs` gates the `mod`); every other platform refuses `--isolation
//! microvm` with DL1408 in `cli.rs` without reaching here.

use std::io::{self, Read as _, Write as _};
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// The guest argument that means "your channel is a vsock stream you dial yourself". The host puts
/// it on the guest kernel's command line, after the `--` that hands arguments to init.
pub const VSOCK_FLAG: &str = "--vsock";

/// The guest's context id. Any value above 2 names a guest; each VM has its own vsock device, so
/// every VM may use the same one.
const GUEST_CID: u32 = 3;

/// The port the guest dials on its host. Firecracker delivers a guest's connection to host port `P`
/// as a connection on the Unix socket `<uds_path>_P`, which the host is already listening on.
const GUEST_PORT: u32 = 1024;

/// Where the image is, when the operator says: a directory holding `manifest.json`.
pub const IMAGE_ENV: &str = "DELULU_MICROVM_IMAGE";

/// Which VMM binary, when the operator says; otherwise `firecracker` on `PATH`.
pub const VMM_ENV: &str = "DELULU_FIRECRACKER";

/// The manifest format `build-image.sh` writes and this launcher reads.
pub const IMAGE_FORMAT: &str = "delulu-microvm-image/1";

/// The Firecracker release this launcher was built and tested against (D-V2-39). Another release is
/// used if the operator provides one, and its version is reported — never assumed to be this one.
pub const TESTED_VMM: &str = "Firecracker v1.17.0";

/// Below this the guest cannot unpack its own initramfs: the static interpreter is most of it, and
/// the kernel holds the archive and the unpacked file at once while it boots.
const MIN_MEMORY_BYTES: u64 = 128 * 1024 * 1024;

/// What the VMM itself may use on top of the guest's memory, under its data ceiling.
const VMM_OVERHEAD_BYTES: u64 = 256 * 1024 * 1024;

/// Each call on the VMM's API socket, and the socket's appearance, is bounded by this.
const API_DEADLINE: Duration = Duration::from_secs(5);

/// How long a guest that has said goodbye gets to power its VM off before the VMM is ended.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(3);

/// The most of the guest's console relayed to the operator; the rest is read and discarded, so a
/// guest that floods its console can neither fill the operator's terminal nor stall its own VMM on a
/// full pipe.
const CONSOLE_CAP: usize = 64 * 1024;

/// What the host applies to every VM it boots, in the words the run report and `doctor` use. Each is
/// true by construction of [`boot`]: the API calls below configure no network interface and no
/// drive, the image is checked before the VMM opens it, and the ceilings are set in `pre_exec`.
pub const GUARANTEES: &[&str] = &[
    "a separate guest kernel under KVM",
    "no network device",
    "no filesystem device",
    "the VM's memory is the guest's memory ceiling",
    "processor-time ceiling on the VMM",
    "wall-clock ceiling",
    "no core dump",
    "killed with the host",
    "the image checked against its manifest before boot",
];

// ------------------------------------------------------------------------------------------------
// The image
// ------------------------------------------------------------------------------------------------

/// A guest image, as its manifest describes it. Nothing here has been verified yet: [`verify`] and
/// [`boot`] do that, against these hashes.
pub struct Image {
    pub dir: PathBuf,
    pub kernel: PathBuf,
    pub kernel_sha256: String,
    pub kernel_version: String,
    pub initramfs: PathBuf,
    pub initramfs_sha256: String,
}

/// The image directory: the operator's choice, or `<state dir>/microvm-image`.
pub fn image_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os(IMAGE_ENV) {
        return Some(PathBuf::from(d));
    }
    crate::brokerd::resolve_state_dir(None).map(|s| s.join("microvm-image"))
}

/// Read the manifest. Refuses a manifest that names a file outside its own directory, or a hash that
/// is not one.
pub fn read_image() -> Result<Image, String> {
    let dir = image_dir().ok_or_else(|| format!("no guest image directory: set {IMAGE_ENV}"))?;
    let manifest = dir.join("manifest.json");
    let text = std::fs::read_to_string(&manifest).map_err(|e| {
        format!(
            "no guest image at `{}` ({e}) — build one with `scripts/microvm/build-image.sh <dir>` and set \
             {IMAGE_ENV}=<dir>",
            dir.display()
        )
    })?;
    parse_manifest(&dir, &text)
}

fn parse_manifest(dir: &Path, text: &str) -> Result<Image, String> {
    let m: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("the guest image's manifest is not JSON: {e}"))?;
    if m["format"] != IMAGE_FORMAT {
        return Err(format!("the guest image's manifest is not `{IMAGE_FORMAT}` (it says {})", m["format"]));
    }
    let part = |section: &str| -> Result<(PathBuf, String), String> {
        let file = m[section]["file"].as_str().unwrap_or_default();
        // A plain name in the image's own directory: a manifest cannot point the launcher anywhere else.
        if file.is_empty() || file.contains('/') || file == "." || file == ".." {
            return Err(format!("the manifest's `{section}.file` must be a file name in the image directory"));
        }
        let sha = m[section]["sha256"].as_str().unwrap_or_default();
        if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(format!("the manifest's `{section}.sha256` is not a lowercase sha256"));
        }
        Ok((dir.join(file), sha.to_string()))
    };
    let (kernel, kernel_sha256) = part("kernel")?;
    let (initramfs, initramfs_sha256) = part("initramfs")?;
    Ok(Image {
        dir: dir.to_path_buf(),
        kernel,
        kernel_sha256,
        kernel_version: m["kernel"]["version"].as_str().unwrap_or("unknown").to_string(),
        initramfs,
        initramfs_sha256,
    })
}

/// Hash a file, copying it to `dst` as it is read when one is given. The copy is created fresh
/// (`create_new`, owner-only), so it cannot be a file someone else prepared.
fn copy_hashed(src: &Path, dst: Option<&Path>) -> io::Result<String> {
    use sha2::Digest as _;
    let mut input = std::fs::File::open(src)?;
    let mut output = match dst {
        Some(d) => Some(std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(d)?),
        None => None,
    };
    let mut hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        if let Some(o) = output.as_mut() {
            o.write_all(&buf[..n])?;
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn mismatch(what: &str, src: &Path, got: &str, want: &str) -> String {
    format!(
        "the guest {what} `{}` does not match its manifest (sha256 {got}; the manifest says {want}) — \
         refused before boot",
        src.display()
    )
}

/// Check the image in place, for `probe` and `doctor`. A launch does not rely on this: it checks the
/// copy it boots.
pub fn verify(image: &Image) -> Result<(), String> {
    for (what, src, want) in
        [("kernel", &image.kernel, &image.kernel_sha256), ("initramfs", &image.initramfs, &image.initramfs_sha256)]
    {
        let got = copy_hashed(src, None).map_err(|e| format!("the guest {what} `{}` cannot be read: {e}", src.display()))?;
        if got != *want {
            return Err(mismatch(what, src, &got, want));
        }
    }
    Ok(())
}

// ------------------------------------------------------------------------------------------------
// KVM and the VMM
// ------------------------------------------------------------------------------------------------

/// Open `/dev/kvm` read-write and ask it for its API version, as a VMM will. An attempt, never a
/// reading of whether the file exists.
pub fn attempt_kvm() -> Result<String, String> {
    use std::os::fd::AsRawFd as _;
    let kvm = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .map_err(|e| format!("KVM is unavailable: `/dev/kvm` cannot be opened read-write ({e})"))?;
    const KVM_GET_API_VERSION: libc::c_ulong = 0xAE00;
    // The argument is an explicit 0, not omitted: KVM answers EINVAL unless it is exactly zero, and
    // `ioctl` is variadic, so an omitted argument is whatever the register held (found by the first
    // boot, which was refused as "API version -1" on a host where KVM works).
    // SAFETY: an ioctl whose argument is a plain integer, on a descriptor this function owns.
    let v = unsafe { libc::ioctl(kvm.as_raw_fd(), KVM_GET_API_VERSION as _, 0 as libc::c_ulong) };
    if v != 12 {
        return Err(format!("`/dev/kvm` answered API version {v}, not 12"));
    }
    Ok("opened /dev/kvm read-write (API 12)".to_string())
}

fn is_executable(p: &Path) -> bool {
    std::fs::metadata(p).map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

/// The VMM binary: `DELULU_FIRECRACKER`, else `firecracker` on `PATH`.
pub fn locate_vmm() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os(VMM_ENV) {
        let p = PathBuf::from(p);
        return if is_executable(&p) {
            Ok(p)
        } else {
            Err(format!("{VMM_ENV} names `{}`, which is not an executable file", p.display()))
        };
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|d| d.join("firecracker"))
        .find(|p| is_executable(p))
        .ok_or_else(|| format!("no microVM monitor: `firecracker` is not on PATH and {VMM_ENV} is not set ({TESTED_VMM} is the tested one)"))
}

/// Ask the VMM what it is. Its first line, e.g. `Firecracker v1.17.0`.
pub fn vmm_version(vmm: &Path) -> Result<String, String> {
    let out = Command::new(vmm)
        .arg("--version")
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("`{}` cannot be run: {e}", vmm.display()))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().next().unwrap_or("").trim().to_string();
    if !out.status.success() || !line.starts_with("Firecracker v") {
        return Err(format!("`{}` did not answer `--version` as Firecracker does", vmm.display()));
    }
    Ok(line)
}

/// What a run checks before it commits to L2: KVM opened, the VMM answering, and a manifest to boot
/// from. The image's bytes are NOT hashed here — [`boot`] hashes the copy it boots, and hashing the
/// original first as well would read the image twice and prove less than the second read does.
/// (`sandbox probe` and `doctor`, which boot nothing unless everything is present, hash in place with
/// [`verify`] — `sandbox.rs`, `l2_attempts`.)
pub fn prerequisites() -> Result<(PathBuf, String, Image), String> {
    attempt_kvm()?;
    let vmm = locate_vmm()?;
    let version = vmm_version(&vmm)?;
    let image = read_image()?;
    Ok((vmm, version, image))
}

// ------------------------------------------------------------------------------------------------
// The VMM's API: a small HTTP/1.1 client over its Unix socket
// ------------------------------------------------------------------------------------------------

/// The most of one API response this client reads. Firecracker's answers are a few hundred bytes.
const API_RESPONSE_MAX: usize = 64 * 1024;

/// Read one HTTP/1.1 response: its status code and its body. Bounded, and strict about the shape —
/// the peer is a VMM this process started, but a client that trusts a length it was told is the
/// shape IPC-1 and ADAPTER-LINE-1 were.
pub(crate) fn read_http_response(r: &mut impl io::Read) -> Result<(u16, String), String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if buf.len() > API_RESPONSE_MAX {
            return Err(format!("the VMM's response headers passed {} KiB", API_RESPONSE_MAX / 1024));
        }
        let n = r.read(&mut chunk).map_err(|e| format!("the VMM's API did not answer: {e}"))?;
        if n == 0 {
            return Err("the VMM closed its API connection before answering".to_string());
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = std::str::from_utf8(&buf[..head_end]).map_err(|_| "the VMM's response headers are not text".to_string())?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let mut words = status_line.splitn(3, ' ');
    let (proto, code) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
    if !proto.starts_with("HTTP/1.") || code.len() != 3 {
        return Err(format!("the VMM's API answered `{status_line}`, which is not an HTTP/1.1 status line"));
    }
    let status: u16 = code.parse().map_err(|_| format!("the VMM's API answered status `{code}`"))?;
    let mut length = 0usize;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().map_err(|_| format!("the VMM's API sent a Content-Length of `{}`", v.trim()))?;
            }
        }
    }
    if length > API_RESPONSE_MAX {
        return Err(format!("the VMM's API announced a {length}-byte body; the most this client reads is {API_RESPONSE_MAX}"));
    }
    let mut body = buf[head_end + 4..].to_vec();
    while body.len() < length {
        let n = r.read(&mut chunk).map_err(|e| format!("the VMM's API stopped mid-answer: {e}"))?;
        if n == 0 {
            return Err("the VMM closed its API connection mid-answer".to_string());
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);
    Ok((status, String::from_utf8_lossy(&body).into_owned()))
}

/// `PUT <path>` with a JSON body; `Ok` on any 2xx. A refusal carries the VMM's own `fault_message`.
fn api_put(sock: &Path, path: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut s = UnixStream::connect(sock).map_err(|e| format!("the VMM's API socket refused a connection: {e}"))?;
    s.set_read_timeout(Some(API_DEADLINE)).map_err(|e| e.to_string())?;
    s.set_write_timeout(Some(API_DEADLINE)).map_err(|e| e.to_string())?;
    let body = body.to_string();
    let request = format!(
        "PUT {path} HTTP/1.1\r\nHost: localhost\r\nAccept: application/json\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    s.write_all(request.as_bytes()).map_err(|e| format!("the VMM's API did not take `PUT {path}`: {e}"))?;
    let (status, text) = read_http_response(&mut s)?;
    if (200..300).contains(&status) {
        return Ok(());
    }
    let fault = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v["fault_message"].as_str().map(str::to_string))
        .unwrap_or(text);
    Err(format!("the VMM refused `PUT {path}` (HTTP {status}): {fault}"))
}

/// The guest kernel's command line. Only parameters the guest kernel knows: an unknown word would be
/// handed to init as an ARGUMENT, ahead of the ones after `--`, and the guest refuses arguments it
/// does not expect. `loglevel=0` keeps the kernel's own messages off the console, so what reaches the
/// operator is what the guest process wrote — the first boot printed the kernel's "Restarting system"
/// after the program's output. A guest kernel that dies is still reported: by the host, from the
/// channel that ended and the VMM's exit.
fn boot_args() -> String {
    format!(
        "console=ttyS0 reboot=k panic=-1 loglevel=0 rdinit=/delulu -- {} {VSOCK_FLAG} {GUEST_PORT}",
        crate::guest::GUEST_SUBCOMMAND
    )
}

// ------------------------------------------------------------------------------------------------
// The launch
// ------------------------------------------------------------------------------------------------

/// A running VM: its VMM, its private directory, and — once the guest has dialled — its channel.
pub struct Vm {
    child: Option<Child>,
    dir: PathBuf,
    pub channel: Option<UnixStream>,
    console: Option<std::thread::JoinHandle<Vec<u8>>>,
    watchdog: Option<(mpsc::Sender<()>, std::thread::JoinHandle<()>)>,
    /// Set, under this lock, the moment the VMM has been reaped. The watchdog kills only while it is
    /// unset, under the same lock, so it can never signal a process id the system has reused.
    reaped: Arc<Mutex<bool>>,
    status: Option<ExitStatus>,
}

/// A VM directory's name: the host's process id first, so a sweep can tell a live host's VM from one
/// whose host is gone.
const DIR_PREFIX: &str = "delulu-vm-";

/// Remove VM directories this user's earlier hosts left behind when they were killed before they
/// could clean up. Their VMMs died with them (`PR_SET_PDEATHSIG`); only the directories remain.
fn sweep_orphans() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else { return };
    // SAFETY: `geteuid` cannot fail.
    let me = unsafe { libc::geteuid() };
    for e in entries.flatten() {
        let name = e.file_name();
        let Some(rest) = name.to_str().and_then(|n| n.strip_prefix(DIR_PREFIX)) else { continue };
        let Some(pid) = rest.split('-').next().and_then(|p| p.parse::<libc::pid_t>().ok()) else { continue };
        let Ok(meta) = std::fs::symlink_metadata(e.path()) else { continue };
        use std::os::unix::fs::MetadataExt as _;
        if !meta.is_dir() || meta.uid() != me || pid == std::process::id() as libc::pid_t {
            continue;
        }
        // SAFETY: signal 0 only asks whether the process exists.
        let gone = unsafe { libc::kill(pid, 0) } != 0 && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
        if gone {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

/// The ceilings the VMM runs under, set between `fork` and `exec` so they hold from its first
/// instruction: the processor time the operator allowed (the guest's vCPU is one of the VMM's
/// threads, so the guest's computing is counted), a data ceiling that is the guest's memory plus
/// room for the VMM, no core dump, no privilege gained through `exec`, and death with the host.
fn harden_vmm(cmd: &mut Command, limits: crate::jail::Limits) {
    use std::os::unix::process::CommandExt as _;
    let data = limits.memory_bytes.saturating_add(VMM_OVERHEAD_BYTES);
    let cpu = limits.cpu_seconds;
    // SAFETY: `pre_exec` runs in the forked child and calls only async-signal-safe functions.
    unsafe {
        cmd.pre_exec(move || {
            libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong, 0, 0, 0);
            let set = |res, value: u64| {
                let lim = libc::rlimit { rlim_cur: value as libc::rlim_t, rlim_max: value as libc::rlim_t };
                libc::setrlimit(res, &lim);
            };
            set(libc::RLIMIT_DATA, data);
            set(libc::RLIMIT_CPU, cpu);
            set(libc::RLIMIT_CORE, 0);
            Ok(())
        });
    }
}

/// The wall-clock ceiling: twice the processor time allowed, and never under a minute. A VM that is
/// neither computing nor asking — a hung guest kernel, a stuck VMM — is ended by it; one that is
/// computing meets the processor-time ceiling first.
fn wall_ceiling(limits: crate::jail::Limits) -> Duration {
    Duration::from_secs(limits.cpu_seconds.saturating_mul(2).max(60))
}

/// Relay the guest's console to the operator's standard error — or keep it, for a probe — up to
/// [`CONSOLE_CAP`], and keep draining after that so the VMM never blocks on a full pipe.
fn relay_console(mut out: std::process::ChildStdout, keep: bool) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut passed = 0usize;
        let mut told = false;
        let mut buf = [0u8; 4096];
        loop {
            let n = match out.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            // The console line discipline turns each newline into CR LF; the operator's does not.
            let text: Vec<u8> = buf[..n].iter().copied().filter(|b| *b != b'\r').collect();
            let take = text.len().min(CONSOLE_CAP.saturating_sub(passed));
            if take > 0 {
                if keep {
                    kept.extend_from_slice(&text[..take]);
                } else {
                    let _ = io::stderr().write_all(&text[..take]);
                }
                passed += take;
            }
            if take < text.len() && !told {
                told = true;
                if !keep {
                    eprintln!("\nsandbox: the guest's console passed {} KiB; the rest was discarded", CONSOLE_CAP / 1024);
                }
            }
        }
        kept
    })
}

impl Vm {
    /// The tail of the VMM's own log, for an error that needs it.
    fn log_tail(&self) -> String {
        let text = std::fs::read_to_string(self.dir.join("vmm.log")).unwrap_or_default();
        let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        match lines.len() {
            0 => String::new(),
            n => format!(" — the VMM said: {}", lines[n.saturating_sub(3)..].join(" | ")),
        }
    }

    fn start_watchdog(&mut self, pid: u32, wall: Duration) {
        let (tx, rx) = mpsc::channel::<()>();
        let reaped = Arc::clone(&self.reaped);
        let handle = std::thread::spawn(move || {
            if let Err(mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(wall) {
                let reaped = reaped.lock().unwrap_or_else(|p| p.into_inner());
                if !*reaped {
                    eprintln!("sandbox: the microVM passed its wall-clock ceiling ({} s) — ended", wall.as_secs());
                    // SAFETY: the VMM has not been reaped (checked under the lock that reaping takes),
                    // so this process id is still the VMM's.
                    unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
                }
            }
        });
        self.watchdog = Some((tx, handle));
    }

    fn stop_watchdog(&mut self) {
        if let Some((tx, handle)) = self.watchdog.take() {
            drop(tx);
            let _ = handle.join();
        }
    }

    /// Whether the VMM has exited, reaping it if it has.
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(s) = self.status {
            return Ok(Some(s));
        }
        let Some(child) = self.child.as_mut() else { return Ok(None) };
        let mut reaped = self.reaped.lock().unwrap_or_else(|p| p.into_inner());
        let got = child.try_wait()?;
        if let Some(s) = got {
            *reaped = true;
            self.status = Some(s);
        }
        Ok(got)
    }

    /// End the VM and reap its VMM: a grace period for a guest that has said goodbye to power off,
    /// then `SIGKILL`. Returns only once the VMM is dead and reaped (T10: a VM is never reported ended
    /// before it is), and removes the VM's directory.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(s) = self.status {
            self.cleanup();
            return Ok(s);
        }
        let until = Instant::now() + SHUTDOWN_GRACE;
        let status = loop {
            if let Some(s) = self.try_wait()? {
                break s;
            }
            if Instant::now() >= until {
                break self.kill_and_reap()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        self.cleanup();
        Ok(status)
    }

    pub fn kill(&mut self) -> io::Result<()> {
        if self.status.is_none() {
            self.kill_and_reap()?;
        }
        Ok(())
    }

    fn kill_and_reap(&mut self) -> io::Result<ExitStatus> {
        let Some(child) = self.child.as_mut() else {
            return Err(io::Error::other("the VM was never started"));
        };
        let mut reaped = self.reaped.lock().unwrap_or_else(|p| p.into_inner());
        let _ = child.kill();
        let status = child.wait()?;
        *reaped = true;
        self.status = Some(status);
        Ok(status)
    }

    /// The console, kept, for a probe that captured it. Waits for the relay, so call it after the VMM
    /// has exited.
    pub fn take_console(&mut self) -> Option<Box<dyn io::Read>> {
        let kept = self.console.take()?.join().ok()?;
        Some(Box::new(io::Cursor::new(kept)))
    }

    fn cleanup(&mut self) {
        self.stop_watchdog();
        if let Some(h) = self.console.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        self.stop_watchdog();
        if self.status.is_none() && self.child.is_some() {
            let _ = self.kill_and_reap();
        }
        self.cleanup();
    }
}

/// Boot a VM and wait for its guest to dial in. Returns the VM, holding its channel, and what was
/// applied. Every failure is reported with what the VMM or the guest said, and leaves nothing
/// running and no directory behind.
pub fn boot(limits: crate::jail::Limits, keep_console: bool) -> io::Result<(Vm, Vec<&'static str>)> {
    let fail = |m: String| io::Error::other(m);
    if limits.memory_bytes < MIN_MEMORY_BYTES {
        return Err(fail(format!(
            "a microVM needs at least {} MiB of memory to boot its guest; this run allows {} MiB",
            MIN_MEMORY_BYTES >> 20,
            limits.memory_bytes >> 20
        )));
    }
    attempt_kvm().map_err(fail)?;
    let vmm = locate_vmm().map_err(fail)?;
    let image = read_image().map_err(fail)?;
    sweep_orphans();

    let dir = std::env::temp_dir().join(format!("{DIR_PREFIX}{}-{}", std::process::id(), crate::guest::channel_tag()));
    std::fs::DirBuilder::new().mode(0o700).create(&dir)?;
    // From here the directory is the VM's, and `Drop` removes it on every path out.
    let mut vm = Vm {
        child: None,
        dir: dir.clone(),
        channel: None,
        console: None,
        watchdog: None,
        reaped: Arc::new(Mutex::new(false)),
        status: None,
    };

    // The bytes verified are the bytes booted: each file is checked while it is copied into the
    // VM's own directory, and the VMM is pointed at the copy. Checking the original and then booting
    // it would leave the time in between for someone to change it.
    for (what, src, want, name) in [
        ("kernel", &image.kernel, &image.kernel_sha256, "vmlinux"),
        ("initramfs", &image.initramfs, &image.initramfs_sha256, "initramfs.cpio"),
    ] {
        let got = copy_hashed(src, Some(&dir.join(name)))
            .map_err(|e| fail(format!("the guest {what} `{}` cannot be read: {e}", src.display())))?;
        if got != *want {
            return Err(fail(mismatch(what, src, &got, want)));
        }
    }

    let api = dir.join("api.sock");
    let vsock = dir.join("v.sock");
    let dial_in = dir.join(format!("v.sock_{GUEST_PORT}"));
    // `sun_path` holds 108 bytes; a longer path would be cut short by the kernel, not refused.
    if dial_in.as_os_str().len() > 100 {
        return Err(fail(format!(
            "the VM's socket path `{}` is too long for a Unix socket — set TMPDIR to a shorter directory",
            dial_in.display()
        )));
    }
    let listener = UnixListener::bind(&dial_in)?;
    listener.set_nonblocking(true)?;
    let log = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(dir.join("vmm.log"))?;

    let mut cmd = Command::new(&vmm);
    cmd.arg("--api-sock")
        .arg(&api)
        .arg("--log-path")
        .arg(dir.join("vmm.log"))
        .arg("--level")
        .arg("Warning")
        .env_clear()
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log));
    harden_vmm(&mut cmd, limits);
    let mut child = cmd.spawn().map_err(|e| fail(format!("`{}` could not be started: {e}", vmm.display())))?;
    let pid = child.id();
    let console = child.stdout.take();
    vm.child = Some(child);
    if let Some(out) = console {
        vm.console = Some(relay_console(out, keep_console));
    }
    vm.start_watchdog(pid, wall_ceiling(limits));

    // The API socket appears once the VMM is listening.
    let until = Instant::now() + API_DEADLINE;
    while !api.exists() {
        if let Some(st) = vm.try_wait()? {
            return Err(fail(format!("the VMM exited before it opened its API ({st}){}", vm.log_tail())));
        }
        if Instant::now() >= until {
            return Err(fail(format!("the VMM did not open its API within {API_DEADLINE:?}{}", vm.log_tail())));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let configure = || -> Result<(), String> {
        // One vCPU, and the memory the run allows: the guest cannot use more, because it has no more.
        api_put(
            &api,
            "/machine-config",
            &serde_json::json!({ "vcpu_count": 1, "mem_size_mib": limits.memory_bytes >> 20, "smt": false }),
        )?;
        api_put(
            &api,
            "/boot-source",
            &serde_json::json!({
                "kernel_image_path": dir.join("vmlinux"),
                "initrd_path": dir.join("initramfs.cpio"),
                "boot_args": boot_args(),
            }),
        )?;
        // The one device. No `/network-interfaces`, no `/drives`: those calls are never made.
        api_put(&api, "/vsock", &serde_json::json!({ "guest_cid": GUEST_CID, "uds_path": vsock }))?;
        api_put(&api, "/actions", &serde_json::json!({ "action_type": "InstanceStart" }))
    };
    configure().map_err(|e| fail(format!("{e}{}", vm.log_tail())))?;

    // The guest dials in once its kernel has booted and `/delulu` is running.
    let until = Instant::now() + crate::guest::CONNECT_DEADLINE;
    let mut conn = loop {
        match listener.accept() {
            Ok((s, _)) => break s,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
        if let Some(st) = vm.try_wait()? {
            return Err(fail(format!("the VM ended before its guest dialled in ({st}){}", vm.log_tail())));
        }
        if Instant::now() >= until {
            return Err(fail(format!(
                "the guest did not dial in within {:?}{}",
                crate::guest::CONNECT_DEADLINE,
                vm.log_tail()
            )));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    conn.set_nonblocking(false)?;
    conn.set_read_timeout(Some(crate::guest::CONNECT_DEADLINE))?;
    let mut first = [0u8; 1];
    match conn.read(&mut first) {
        Ok(1) if first[0] == crate::guest::STDIO_READY => {}
        Ok(1) => return Err(fail("the guest's first byte was not the one a guest sends".to_string())),
        Ok(_) => return Err(fail("the guest hung up before it was ready".to_string())),
        Err(e) => return Err(fail(format!("the guest was not ready within {:?}: {e}", crate::guest::CONNECT_DEADLINE))),
    }
    conn.set_read_timeout(None)?;
    vm.channel = Some(conn);
    Ok((vm, GUARANTEES.to_vec()))
}

/// PS-A-07's rule, at L2: ATTEMPT the launcher, so `sandbox probe` and `doctor` answer from a boot.
/// A VM is booted, its guest's dial-in awaited, and the VM ended — no program is run, so nothing is
/// written to the audit chain.
pub fn attempt_launch() -> Result<String, String> {
    let limits = crate::policy::Profile::HostileAgent.limits();
    let started = Instant::now();
    let (mut vm, _) = boot(limits, true).map_err(|e| e.to_string())?;
    let booted = started.elapsed();
    drop(vm.channel.take());
    let _ = vm.kill();
    let _ = vm.wait();
    Ok(format!("booted a guest and heard it dial in, in {} ms", booted.as_millis()))
}

// ------------------------------------------------------------------------------------------------
// The guest half: PID 1 inside the VM
// ------------------------------------------------------------------------------------------------

/// Whether this kernel has no IP stack at all, found by asking it for an IPv4 and an IPv6 socket.
/// Only "address family not supported" for BOTH counts; any other answer — a socket, or a refusal
/// for some other reason — does not.
fn kernel_has_no_ip() -> bool {
    for family in [libc::AF_INET, libc::AF_INET6] {
        // SAFETY: a plain `socket` call; a descriptor it returns is closed at once.
        let fd = unsafe { libc::socket(family, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
        if fd >= 0 {
            unsafe { libc::close(fd) };
            return false;
        }
        if io::Error::last_os_error().raw_os_error() != Some(libc::EAFNOSUPPORT) {
            return false;
        }
    }
    true
}

/// Dial the host: a vsock stream to context 2 (the host), on `port`. Retried briefly, because the
/// guest may reach this before its vsock device has finished probing.
fn dial_host(port: u32) -> io::Result<UnixStream> {
    use std::os::fd::{FromRawFd as _, OwnedFd};
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        // SAFETY: the descriptor is owned by `fd` from the moment it exists, so every path closes it.
        let raw = unsafe { libc::socket(libc::AF_VSOCK, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        // SAFETY: a zeroed `sockaddr_vm` is a valid value; the fields that matter are set below.
        let mut addr: libc::sockaddr_vm = unsafe { std::mem::zeroed() };
        addr.svm_family = libc::AF_VSOCK as libc::sa_family_t;
        addr.svm_port = port;
        addr.svm_cid = libc::VMADDR_CID_HOST;
        // SAFETY: `addr` is a live `sockaddr_vm` and the length passed is its size.
        let r = unsafe {
            libc::connect(
                raw,
                (&addr as *const libc::sockaddr_vm).cast::<libc::sockaddr>(),
                std::mem::size_of::<libc::sockaddr_vm>() as libc::socklen_t,
            )
        };
        if r == 0 {
            // A connected stream socket; the standard library's Unix stream reads, writes and sets
            // deadlines on any stream socket, and nothing here asks it for an address.
            return Ok(UnixStream::from(fd));
        }
        let e = io::Error::last_os_error();
        if Instant::now() >= until {
            return Err(e);
        }
        drop(fd);
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `__guest --vsock <port>`: the microVM guest. Returns only when it is not PID 1 (a test running it
/// by hand); as PID 1 it powers the VM off, because init returning panics the kernel.
pub fn run_vm_guest(args: &[String]) -> i32 {
    let code = serve_vm_guest(args);
    if std::process::id() == 1 {
        // SAFETY: `sync` and `reboot` take no pointers. With `reboot=k` on the command line the VMM
        // treats the reset as the VM's end and exits.
        unsafe {
            libc::sync();
            libc::reboot(libc::RB_AUTOBOOT);
        }
    }
    code
}

fn serve_vm_guest(args: &[String]) -> i32 {
    let port = match args {
        [p] => match p.parse::<u32>() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("error: `{VSOCK_FLAG}` needs a port number, not `{p}`");
                return 2;
            }
        },
        _ => {
            eprintln!("error: the microVM guest takes exactly `{VSOCK_FLAG} <port>`, and was given {args:?}");
            return 2;
        }
    };
    // Measured before anything else runs, while this is still the toolchain's own code speaking.
    let mut measured: Vec<&'static str> = Vec::new();
    if kernel_has_no_ip() {
        measured.push("no network stack in its kernel");
    }
    let mut conn = match dial_host(port) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: the microVM guest cannot reach its host on vsock port {port}: {e}");
            return 2;
        }
    };
    if conn.set_read_timeout(Some(crate::guest::CHANNEL_DEADLINE)).is_err() {
        eprintln!("error: the microVM guest cannot set its channel deadline — refusing to run unbounded");
        return 2;
    }
    if conn.write_all(&[crate::guest::STDIO_READY]).is_err() {
        eprintln!("error: the microVM guest cannot write to its host");
        return 2;
    }
    crate::guest::serve_as_guest(conn, None, &measured)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(bytes: &[u8]) -> Result<(u16, String), String> {
        read_http_response(&mut io::Cursor::new(bytes.to_vec()))
    }

    #[test]
    fn an_api_response_is_read_by_its_status_and_length() {
        assert_eq!(response(b"HTTP/1.1 204 \r\nServer: Firecracker API\r\n\r\n").unwrap(), (204, String::new()));
        let fault = b"HTTP/1.1 400 \r\nContent-Type: application/json\r\nContent-Length: 29\r\n\r\n{\"fault_message\":\"no kernel\"}";
        assert_eq!(response(fault).unwrap(), (400, "{\"fault_message\":\"no kernel\"}".to_string()));
        // Bytes after the announced length are not part of the answer.
        assert_eq!(response(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nokEXTRA").unwrap(), (200, "ok".to_string()));
    }

    #[test]
    fn a_malformed_or_unbounded_api_response_is_refused() {
        assert!(response(b"").unwrap_err().contains("before answering"));
        assert!(response(b"SSH-2.0-x\r\n\r\n").unwrap_err().contains("not an HTTP/1.1 status line"));
        assert!(response(b"HTTP/1.1 20x \r\n\r\n").is_err());
        assert!(response(b"HTTP/1.1 200 \r\nContent-Length: 10\r\n\r\nshort").unwrap_err().contains("mid-answer"));
        assert!(response(b"HTTP/1.1 200 \r\nContent-Length: 99999999\r\n\r\n").unwrap_err().contains("most this client reads"));
        let endless = [b"HTTP/1.1 200 \r\nX: ".as_slice(), &vec![b'a'; API_RESPONSE_MAX + 10]].concat();
        assert!(response(&endless).unwrap_err().contains("passed"));
    }

    fn manifest(kernel_file: &str, sha: &str) -> String {
        serde_json::json!({
            "format": IMAGE_FORMAT,
            "kernel": { "file": kernel_file, "sha256": sha, "version": "6.18.54" },
            "initramfs": { "file": "initramfs.cpio", "sha256": "a".repeat(64) },
        })
        .to_string()
    }

    #[test]
    fn a_manifest_names_files_in_its_own_directory_and_real_hashes_only() {
        let dir = Path::new("/images/one");
        let ok = parse_manifest(dir, &manifest("vmlinux", &"0".repeat(64))).unwrap();
        assert_eq!(ok.kernel, dir.join("vmlinux"));
        assert_eq!(ok.kernel_version, "6.18.54");
        for escape in ["../vmlinux", "/boot/vmlinuz", "..", "", "sub/vmlinux"] {
            let e = parse_manifest(dir, &manifest(escape, &"0".repeat(64))).map(|_| ()).unwrap_err();
            assert!(e.contains("file name in the image directory"), "{escape}: {e}");
        }
        for bad in ["0".repeat(63), "G".repeat(64), "A".repeat(64)] {
            assert!(parse_manifest(dir, &manifest("vmlinux", &bad)).is_err(), "{bad}");
        }
        let other = manifest("vmlinux", &"0".repeat(64)).replace(IMAGE_FORMAT, "delulu-microvm-image/0");
        assert!(parse_manifest(dir, &other).map(|_| ()).unwrap_err().contains("is not"));
    }

    /// T13's mechanism: a copy whose bytes differ from the manifest by one byte is refused, and the
    /// hash is of the bytes COPIED.
    #[test]
    fn the_copy_is_hashed_as_it_is_written() {
        let d = std::env::temp_dir().join(format!("delulu-microvm-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("src"), b"abc").unwrap();
        let got = copy_hashed(&d.join("src"), Some(&d.join("dst"))).unwrap();
        assert_eq!(got, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(std::fs::read(d.join("dst")).unwrap(), b"abc");
        // The copy refuses to write over something already there.
        assert!(copy_hashed(&d.join("src"), Some(&d.join("dst"))).is_err());
        std::fs::write(d.join("src"), b"abd").unwrap();
        assert_ne!(copy_hashed(&d.join("src"), None).unwrap(), got);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_boot_line_ends_with_the_guest_arguments_and_nothing_after() {
        let line = boot_args();
        let (_, init) = line.split_once(" -- ").unwrap();
        assert_eq!(init, format!("{} {VSOCK_FLAG} {GUEST_PORT}", crate::guest::GUEST_SUBCOMMAND));
    }

    #[test]
    fn the_wall_ceiling_is_twice_the_processor_time_and_at_least_a_minute() {
        let l = |cpu| crate::jail::Limits { memory_bytes: 1 << 30, cpu_seconds: cpu };
        assert_eq!(wall_ceiling(l(300)), Duration::from_secs(600));
        assert_eq!(wall_ceiling(l(5)), Duration::from_secs(60));
    }
}
