//! PS-C — L2, the microVM on Linux + KVM (V2 security model §6–§7).
//!
//! The guest runs the INTERPRETER (D-NE-23) as PID 1 of its own kernel under Firecracker, and holds
//! one thing: a vsock stream to its host. No network device, no filesystem device, no secret, no
//! broker address. Every effect it asks for crosses that stream as a `delulu-sandbox-channel/3`
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
//! Run as root, the VMM runs under Firecracker's JAILER (PS-C-03b): a uid of its own, reserved for the
//! VM, in a chroot holding only what it needs; and because the jailer's `setuid` clears the death
//! signal, a small reaper process takes its place — when the host is gone, the reaper ends the VMM.
//! Root WITHOUT a jailer is refused rather than run: the VMM would be root. Run as an ordinary user,
//! the VMM runs as that user under Firecracker's own seccomp filters, and the report names
//! `identity_separation` as a limitation. Not here: attestation (L4).
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

/// Which jailer binary, when the operator says; otherwise `jailer` beside the VMM, then on `PATH`.
pub const JAILER_ENV: &str = "DELULU_JAILER";

/// Where jailed VMs live (PS-C-03b): one chroot per VM, and one reservation file per uid in use,
/// root-owned. Beside Firecracker's own default (`/srv/jailer`), and NOT under `/run`: the jailer makes
/// `/dev/kvm` inside each chroot, and `/run` is mounted `nodev` on most hosts, where that node cannot
/// be opened — the first jailed launch here failed exactly so, with KVM's "permission denied".
const JAIL_BASE: &str = "/srv/delulu-jailer";

/// Where jailed VMs live when the operator says (a filesystem that allows device nodes).
pub const JAIL_BASE_ENV: &str = "DELULU_JAIL_BASE";

fn jail_base() -> PathBuf {
    std::env::var_os(JAIL_BASE_ENV).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(JAIL_BASE))
}

/// Refuse a jail base on a filesystem mounted `nodev`, naming why, rather than let KVM's "permission
/// denied" from inside the chroot be the first anyone hears of it.
fn allows_devices(base: &Path) -> Result<(), String> {
    use std::os::unix::ffi::OsStrExt as _;
    let c = std::ffi::CString::new(base.as_os_str().as_bytes()).map_err(|_| "the jail base holds a NUL byte".to_string())?;
    // SAFETY: `c` is a valid C string and `st` a zeroed, correctly sized buffer the call fills.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return Err(format!("the jail base `{}` cannot be examined: {}", base.display(), io::Error::last_os_error()));
    }
    if st.f_flag & libc::ST_NODEV != 0 {
        return Err(format!(
            "the jail base `{}` is on a filesystem mounted `nodev`, where the jailer's `/dev/kvm` cannot be \
             opened — set {JAIL_BASE_ENV} to a directory on one that allows device nodes",
            base.display()
        ));
    }
    Ok(())
}

/// The uids a jailed VMM runs as: a block no ordinary account is given. Each VM reserves one that no
/// account has and no process is running as, by creating its reservation file exclusively.
const JAIL_UID_BASE: u32 = 900_000;
const JAIL_UID_SPAN: u32 = 65_536;

/// The internal subcommand that ends a jailed VMM when its host is gone. Never advertised.
pub const REAPER_SUBCOMMAND: &str = "__vm_reaper";

/// What a JAILED VMM has on top of [`GUARANTEES`].
pub const JAILED_GUARANTEES: &[&str] = &["a uid of its own for the VMM", "a chroot for the VMM"];

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

/// What was BOOTED: the hashes of the copies the VMM was pointed at, each checked against the
/// manifest before boot. It goes into the `sandbox-launch` audit record and the run report, so the
/// record of a run names the exact image it ran on (the test plan's T13, second half).
#[derive(Clone, Debug)]
pub struct ImageDigest {
    pub kernel_sha256: String,
    pub initramfs_sha256: String,
    pub kernel_version: String,
}

impl ImageDigest {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "kernel_sha256": self.kernel_sha256,
            "initramfs_sha256": self.initramfs_sha256,
            "kernel_version": self.kernel_version,
        })
    }
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

fn running_as_root() -> bool {
    // SAFETY: `geteuid` cannot fail.
    unsafe { libc::geteuid() == 0 }
}

/// The jailer: `DELULU_JAILER`, else `jailer` beside the VMM (where `fetch-firecracker.sh` puts it),
/// else on `PATH`.
pub fn locate_jailer(vmm: &Path) -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os(JAILER_ENV) {
        let p = PathBuf::from(p);
        return if is_executable(&p) {
            Ok(p)
        } else {
            Err(format!("{JAILER_ENV} names `{}`, which is not an executable file", p.display()))
        };
    }
    let beside = vmm.with_file_name("jailer");
    if is_executable(&beside) {
        return Ok(beside);
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).map(|d| d.join("jailer")).find(|p| is_executable(p)).ok_or_else(|| {
        format!(
            "this process is root, and without Firecracker's jailer the VMM would run as root too — \
             refused. Put `jailer` beside `{}` (`scripts/microvm/fetch-firecracker.sh` installs both), set \
             {JAILER_ENV}, or run as an ordinary user",
            vmm.display()
        )
    })
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
    if running_as_root() {
        locate_jailer(&vmm)?;
    }
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
    /// Set by the watchdog when it ended the VM, so the run names the stop (SANDBOX-STOP-1).
    wall_fired: Arc<std::sync::atomic::AtomicBool>,
    /// Set by the console relay when the guest said it reached its memory ceiling.
    memory_ceiling: Arc<std::sync::atomic::AtomicBool>,
    status: Option<ExitStatus>,
    /// The image this VM booted, once its copies have been checked.
    pub image: Option<ImageDigest>,
    /// The VMM's own log, as the host names it.
    log: PathBuf,
    /// A jailed VM's uid reservation and its reaper.
    jail: Option<JailState>,
}

/// What a jailed VM holds on the host beside its directory.
struct JailState {
    /// The reservation file for the VM's uid; it holds this host's process id.
    uid_lock: PathBuf,
    /// The reaper, and the write end of the pipe it waits on: closing it (the host finishing, or
    /// dying) is what wakes it.
    reaper: Option<(std::process::ChildStdin, Child)>,
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
/// computing meets the processor-time ceiling first. An operator's `--limits wall=` narrows it,
/// never widens it (SANDBOX-STOP-1).
fn wall_ceiling(limits: crate::jail::Limits) -> Duration {
    let own = Duration::from_secs(limits.cpu_seconds.saturating_mul(2).max(60));
    match limits.wall_seconds {
        Some(w) => own.min(Duration::from_secs(w)),
        None => own,
    }
}

/// Relay the guest's console to the operator's standard error — or keep it, for a probe — up to
/// [`CONSOLE_CAP`], and keep draining after that so the VMM never blocks on a full pipe.
fn relay_console(
    mut out: std::process::ChildStdout,
    keep: bool,
    memory_ceiling: Arc<std::sync::atomic::AtomicBool>,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut passed = 0usize;
        let mut told = false;
        let mut buf = [0u8; 4096];
        // Relayed a LINE at a time, so the guest's ceiling line (SANDBOX-STOP-1) is recognised and
        // withheld whole: the serial console delivers a line in pieces, and a relay that looked at each
        // piece recognised the line but still passed it to the operator (verify-vm-memory3.log). A
        // partial line is held until its newline, but never beyond one read's worth.
        let marker = crate::ceiling::VM_MEMORY_MARKER.trim_ascii();
        let mut pending: Vec<u8> = Vec::new();
        let mut emit = |line: &[u8], kept: &mut Vec<u8>| {
            // RW 4.32: the console is the guest's own text — its kernel's and its standard error — so each
            // line reaches the operator, and the probe's answer, escaped (TERMINAL-TEXT-1): it used to be
            // written raw, and a guest's control sequences drove the operator's terminal.
            let (body, end) = match line.strip_suffix(b"\n") {
                Some(body) => (body, "\n"),
                None => (line, ""),
            };
            let shown = format!("{}{end}", delulu_diag::terminal_line(&String::from_utf8_lossy(body)));
            let bytes = shown.as_bytes();
            let take = bytes.len().min(CONSOLE_CAP.saturating_sub(passed));
            if take > 0 {
                if keep {
                    kept.extend_from_slice(&bytes[..take]);
                } else {
                    let _ = io::stderr().write_all(&bytes[..take]);
                }
                passed += take;
            }
            if take < bytes.len() && !told {
                told = true;
                if !keep {
                    eprintln!("\nsandbox: the guest's console passed {} KiB; the rest was discarded", CONSOLE_CAP / 1024);
                }
            }
        };
        loop {
            let n = match out.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            // The console line discipline turns each newline into CR LF; the operator's does not.
            pending.extend(buf[..n].iter().copied().filter(|b| *b != b'\r'));
            while let Some(i) = pending.iter().position(|b| *b == b'\n') {
                let line: Vec<u8> = pending.drain(..=i).collect();
                if line.trim_ascii() == marker {
                    // The host names the stop in its own words; this line is for the host.
                    memory_ceiling.store(true, std::sync::atomic::Ordering::SeqCst);
                } else {
                    emit(&line, &mut kept);
                }
            }
            if pending.len() > buf.len() {
                let line = std::mem::take(&mut pending);
                emit(&line, &mut kept);
            }
        }
        if pending.trim_ascii() == marker {
            memory_ceiling.store(true, std::sync::atomic::Ordering::SeqCst);
        } else if !pending.is_empty() {
            emit(&pending, &mut kept);
        }
        kept
    })
}

impl Vm {
    /// The tail of the VMM's own log, for an error that needs it.
    fn log_tail(&self) -> String {
        let text = std::fs::read_to_string(&self.log).unwrap_or_default();
        let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        match lines.len() {
            0 => String::new(),
            n => format!(" — the VMM said: {}", lines[n.saturating_sub(3)..].join(" | ")),
        }
    }

    fn start_watchdog(&mut self, pid: u32, wall: Duration) {
        let (tx, rx) = mpsc::channel::<()>();
        let reaped = Arc::clone(&self.reaped);
        let fired = Arc::clone(&self.wall_fired);
        let handle = std::thread::spawn(move || {
            if let Err(mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(wall) {
                let reaped = reaped.lock().unwrap_or_else(|p| p.into_inner());
                if !*reaped {
                    fired.store(true, std::sync::atomic::Ordering::SeqCst);
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

    /// Whether the wall-clock watchdog ended this VM.
    pub fn wall_fired(&self) -> bool {
        self.wall_fired.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Whether the guest said, on its console, that it reached its memory ceiling. Read after the VMM
    /// has been waited for, when the relay has seen everything the console carried.
    pub fn memory_ceiling_reached(&self) -> bool {
        self.memory_ceiling.load(std::sync::atomic::Ordering::SeqCst)
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
    pub fn take_console(&mut self) -> Option<Box<dyn io::Read + Send>> {
        let kept = self.console.take()?.join().ok()?;
        Some(Box::new(io::Cursor::new(kept)))
    }

    fn cleanup(&mut self) {
        self.stop_watchdog();
        if let Some(h) = self.console.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
        if let Some(jail) = self.jail.as_mut() {
            release_uid(&jail.uid_lock, std::process::id());
            // The VMM is already reaped: closing the pipe wakes the reaper, which finds nothing of
            // this VM left to end and exits.
            if let Some((pipe, mut reaper)) = jail.reaper.take() {
                drop(pipe);
                let _ = reaper.wait();
            }
        }
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
    // Root runs the VMM under the jailer or not at all.
    let jailer = if running_as_root() { Some(locate_jailer(&vmm).map_err(fail)?) } else { None };
    let image = read_image().map_err(fail)?;
    let tag = format!("{}-{}", std::process::id(), crate::guest::channel_tag());

    // Where the VM's files go. Unjailed: a private directory, named the same by host and VMM. Jailed:
    // the chroot the jailer will enter — `<base>/<vmm name>/<id>/root` — which the host fills before
    // the VMM starts and the VMM sees as `/`.
    let (dir, host_root, jail_plan) = match &jailer {
        None => {
            sweep_orphans();
            let dir = std::env::temp_dir().join(format!("{DIR_PREFIX}{tag}"));
            std::fs::DirBuilder::new().mode(0o700).create(&dir)?;
            (dir.clone(), dir, None)
        }
        Some(_) => {
            let base_dir = jail_base();
            let base = base_dir.as_path();
            std::fs::DirBuilder::new().recursive(true).mode(0o700).create(base)?;
            std::fs::set_permissions(base, std::fs::Permissions::from_mode(0o700))?;
            allows_devices(base).map_err(fail)?;
            let name = vmm.file_name().map(|n| n.to_owned()).unwrap_or_else(|| "firecracker".into());
            sweep_jails(base, &name);
            // The jailer's id: letters, digits and dashes, at most 64.
            let id = format!("delulu-{tag}");
            let dir = base.join(&name).join(&id);
            let root = dir.join("root");
            std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&root)?;
            let (uid, lock) = match reserve_uid(base) {
                Ok(r) => r,
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&dir);
                    return Err(e);
                }
            };
            (dir, root, Some((id, uid, lock)))
        }
    };
    // From here the directory is the VM's, and `Drop` removes it on every path out.
    let mut vm = Vm {
        child: None,
        dir: dir.clone(),
        channel: None,
        console: None,
        watchdog: None,
        reaped: Arc::new(Mutex::new(false)),
        wall_fired: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        memory_ceiling: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        status: None,
        image: None,
        log: host_root.join("vmm.log"),
        jail: jail_plan.as_ref().map(|(_, _, lock)| JailState { uid_lock: lock.clone(), reaper: None }),
    };
    // What the VMM calls a file the host put at `host_root/<name>`.
    let vmm_path = |name: &str| -> PathBuf {
        if jail_plan.is_some() {
            Path::new("/").join(name)
        } else {
            host_root.join(name)
        }
    };
    // A file the jailed VMM must use belongs to its uid; unjailed, it is already this user's.
    let give = |p: &Path| -> io::Result<()> {
        if let Some((_, uid, _)) = &jail_plan {
            std::os::unix::fs::chown(p, Some(*uid), Some(*uid))?;
        }
        Ok(())
    };

    // The bytes verified are the bytes booted: each file is checked while it is copied into the
    // VM's own directory, and the VMM is pointed at the copy. Checking the original and then booting
    // it would leave the time in between for someone to change it.
    for (what, src, want, name) in [
        ("kernel", &image.kernel, &image.kernel_sha256, "vmlinux"),
        ("initramfs", &image.initramfs, &image.initramfs_sha256, "initramfs.cpio"),
    ] {
        let got = copy_hashed(src, Some(&host_root.join(name)))
            .map_err(|e| fail(format!("the guest {what} `{}` cannot be read: {e}", src.display())))?;
        if got != *want {
            return Err(fail(mismatch(what, src, &got, want)));
        }
        give(&host_root.join(name))?;
    }
    vm.image = Some(ImageDigest {
        kernel_sha256: image.kernel_sha256.clone(),
        initramfs_sha256: image.initramfs_sha256.clone(),
        kernel_version: image.kernel_version.clone(),
    });

    let api = host_root.join("api.sock");
    let dial_in = host_root.join(format!("v.sock_{GUEST_PORT}"));
    // `sun_path` holds 108 bytes; a longer path would be cut short by the kernel, not refused.
    if dial_in.as_os_str().len() > 100 {
        return Err(fail(format!(
            "the VM's socket path `{}` is too long for a Unix socket — set TMPDIR to a shorter directory",
            dial_in.display()
        )));
    }
    let listener = UnixListener::bind(&dial_in)?;
    listener.set_nonblocking(true)?;
    // The jailed VMM connects to this socket when the guest dials; it must be its to connect to.
    give(&dial_in)?;
    let log = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(host_root.join("vmm.log"))?;
    give(&host_root.join("vmm.log"))?;

    let mut cmd = match (&jailer, &jail_plan) {
        (Some(jailer), Some((id, uid, _))) => {
            let mut c = Command::new(jailer);
            c.arg("--id")
                .arg(id)
                .arg("--exec-file")
                .arg(&vmm)
                .arg("--uid")
                .arg(uid.to_string())
                .arg("--gid")
                .arg(uid.to_string())
                .arg("--chroot-base-dir")
                .arg(jail_base())
                .arg("--");
            c
        }
        _ => Command::new(&vmm),
    };
    cmd.arg("--api-sock")
        .arg(vmm_path("api.sock"))
        .arg("--log-path")
        .arg(vmm_path("vmm.log"))
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
        vm.console = Some(relay_console(out, keep_console, Arc::clone(&vm.memory_ceiling)));
    }
    vm.start_watchdog(pid, wall_ceiling(limits));
    // The jailer's `setuid` cleared the death signal `harden_vmm` set, so a jailed VMM would outlive a
    // host killed mid-run. The reaper puts that back: it waits on a pipe only this host writes to, and
    // when the pipe closes it ends this VMM — named by pid AND jail id, so never another process.
    if let Some((id, _, lock)) = &jail_plan {
        let exe = std::env::current_exe()?;
        let mut reaper = Command::new(exe);
        reaper
            .arg(REAPER_SUBCOMMAND)
            .arg(pid.to_string())
            .arg(id)
            .arg(&dir)
            .arg(lock)
            .env_clear()
            .current_dir("/")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for name in crate::guest::LOADER_ENV {
            if let Ok(v) = std::env::var(name) {
                reaper.env(name, v);
            }
        }
        let mut reaper = reaper.spawn().map_err(|e| fail(format!("the jailed VM's reaper could not be started: {e}")))?;
        let pipe = reaper.stdin.take().ok_or_else(|| fail("the reaper has no pipe".to_string()))?;
        if let Some(j) = vm.jail.as_mut() {
            j.reaper = Some((pipe, reaper));
        }
    }

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
                "kernel_image_path": vmm_path("vmlinux"),
                "initrd_path": vmm_path("initramfs.cpio"),
                "boot_args": boot_args(),
            }),
        )?;
        // The one device. No `/network-interfaces`, no `/drives`: those calls are never made.
        api_put(&api, "/vsock", &serde_json::json!({ "guest_cid": GUEST_CID, "uds_path": vmm_path("v.sock") }))?;
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
    // One connection is the channel, and the listener goes the moment it is accepted — closed, and its
    // socket file removed. Found by the red team's `ports` guest (PS-C-06), which dials port 1024 a
    // second time straight after its ready byte: that connection can still complete into the listening
    // socket's backlog before the first is accepted, but nothing ever reads it, and closing the
    // listener here closes it (the guest reads end-of-file on it). A dial-in after this has nothing to
    // land on at all.
    drop(listener);
    let _ = std::fs::remove_file(&dial_in);
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
    let mut applied = GUARANTEES.to_vec();
    if jail_plan.is_some() {
        applied.extend(JAILED_GUARANTEES);
    }
    Ok((vm, applied))
}

/// The uids of every running process.
fn uids_in_use() -> std::collections::HashSet<u32> {
    let mut uids = std::collections::HashSet::new();
    for e in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        let Ok(status) = std::fs::read_to_string(e.path().join("status")) else { continue };
        if let Some(line) = status.lines().find(|l| l.starts_with("Uid:")) {
            uids.extend(line.split_whitespace().skip(1).filter_map(|u| u.parse::<u32>().ok()));
        }
    }
    uids
}

/// Whether an account in `/etc/passwd` has this uid.
fn account_has(uid: u32) -> bool {
    std::fs::read_to_string("/etc/passwd")
        .map(|t| t.lines().any(|l| l.split(':').nth(2).and_then(|u| u.parse::<u32>().ok()) == Some(uid)))
        .unwrap_or(true)
}

/// Reserve a uid for one jailed VM: one no account has, no process runs as, and no other VM holds.
/// The reservation is a file created exclusively, holding this host's process id — the create is
/// what makes two hosts starting at once pick different uids.
fn reserve_uid(base: &Path) -> io::Result<(u32, PathBuf)> {
    use std::io::Write as _;
    let busy = uids_in_use();
    let start = std::process::id().wrapping_mul(2_654_435_761) % JAIL_UID_SPAN;
    for i in 0..JAIL_UID_SPAN {
        let uid = JAIL_UID_BASE + (start + i) % JAIL_UID_SPAN;
        if busy.contains(&uid) || account_has(uid) {
            continue;
        }
        let lock = base.join(format!("uid-{uid}"));
        match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&lock) {
            Ok(mut f) => {
                writeln!(f, "{}", std::process::id())?;
                return Ok((uid, lock));
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other(format!(
        "every uid from {JAIL_UID_BASE} to {} is in use — no jailed VM can be started",
        JAIL_UID_BASE + JAIL_UID_SPAN - 1
    )))
}

/// Give a uid back — only if the reservation is still `owner`'s, so a late reaper never releases a uid
/// a newer VM has since reserved under the same name.
fn release_uid(lock: &Path, owner: u32) {
    if std::fs::read_to_string(lock).ok().and_then(|t| t.trim().parse::<u32>().ok()) == Some(owner) {
        let _ = std::fs::remove_file(lock);
    }
}

fn process_gone(pid: libc::pid_t) -> bool {
    // SAFETY: signal 0 only asks whether the process exists.
    unsafe { libc::kill(pid, 0) != 0 && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) }
}

/// Whether process `pid` is the jailed VMM of jail `id`: its command line carries `--id <id>`.
fn is_vmm_of(pid: libc::pid_t, id: &str) -> bool {
    let Ok(cmdline) = std::fs::read(format!("/proc/{pid}/cmdline")) else { return false };
    let args: Vec<&[u8]> = cmdline.split(|b| *b == 0).collect();
    args.windows(2).any(|w| w[0] == b"--id" && w[1] == id.as_bytes())
}

/// Jails whose host is gone. The reaper normally removes them; this covers a reaper that could not —
/// the whole machine losing power between the two, say. Their VMMs are ended first, by jail id.
fn sweep_jails(base: &Path, vmm_name: &std::ffi::OsStr) {
    if let Ok(entries) = std::fs::read_dir(base.join(vmm_name)) {
        for e in entries.flatten() {
            let id = e.file_name().to_string_lossy().into_owned();
            let Some(pid) = id.strip_prefix("delulu-").and_then(|r| r.split('-').next()).and_then(|p| p.parse().ok()) else {
                continue;
            };
            if pid == std::process::id() as libc::pid_t || !process_gone(pid) {
                continue;
            }
            for p in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
                if let Ok(vmm) = p.file_name().to_string_lossy().parse::<libc::pid_t>() {
                    if is_vmm_of(vmm, &id) {
                        // SAFETY: the process was just identified by its jail id.
                        unsafe { libc::kill(vmm, libc::SIGKILL) };
                    }
                }
            }
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
    if let Ok(entries) = std::fs::read_dir(base) {
        for e in entries.flatten() {
            if !e.file_name().to_string_lossy().starts_with("uid-") {
                continue;
            }
            let owner = std::fs::read_to_string(e.path()).ok().and_then(|t| t.trim().parse::<libc::pid_t>().ok());
            if owner.is_some_and(process_gone) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

/// `__vm_reaper <vmm pid> <jail id> <jail dir> <uid reservation>`: wait for the host to close this
/// process's standard input — by finishing, or by dying — then end the VMM if it is still that jail's,
/// and remove what the jail left. Its parent's pid is read FIRST, while the parent is still the host,
/// because the reservation it may release is the one holding that pid.
pub fn run_reaper(args: &[String]) -> i32 {
    use std::io::Read as _;
    // SAFETY: `getppid` cannot fail.
    let host = unsafe { libc::getppid() } as u32;
    let [pid, id, dir, lock] = args else { return 2 };
    let Ok(pid) = pid.parse::<libc::pid_t>() else { return 2 };
    let mut sink = [0u8; 64];
    loop {
        match io::stdin().read(&mut sink) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
    if is_vmm_of(pid, id) {
        // SAFETY: the process carries this jail's id, so it is this jail's VMM.
        unsafe { libc::kill(pid, libc::SIGKILL) };
        let until = Instant::now() + Duration::from_secs(5);
        while is_vmm_of(pid, id) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    let _ = std::fs::remove_dir_all(dir);
    release_uid(Path::new(lock), host);
    0
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
    crate::ceiling::enter_vm_guest_mode();
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

/// Set this process's `RLIMIT_DATA` to 85% of the memory the kernel reports free now (the rest is the
/// kernel's, the in-memory root filesystem's, and headroom for the page cache). `sysinfo`, so it needs no
/// `/proc`. `false` when either call fails — the guest still runs, just without the named stop.
fn cap_data_below_free_memory() -> bool {
    // SAFETY: `sysinfo` writes one struct we own; `setrlimit` reads one.
    unsafe {
        let mut info: libc::sysinfo = std::mem::zeroed();
        if libc::sysinfo(&mut info) != 0 {
            return false;
        }
        let free = (info.freeram as u64).saturating_mul(info.mem_unit.max(1) as u64);
        let cap = free / 100 * 85;
        if cap == 0 {
            return false;
        }
        let lim = libc::rlimit { rlim_cur: cap as libc::rlim_t, rlim_max: cap as libc::rlim_t };
        libc::setrlimit(libc::RLIMIT_DATA, &lim) == 0
    }
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
    // SANDBOX-STOP-1 at L2: the VM's memory IS the guest's ceiling, but nothing limited the guest's
    // own address space, so Linux overcommitted, the allocator never saw a refusal, and the guest
    // kernel's OOM handling ended the VM with nothing said (verified: `verify-vm-memory2.log`). A data
    // limit below the memory still free lets the ALLOCATOR meet the ceiling first, so `ceiling.rs`
    // can say so on the console the host relays.
    if cap_data_below_free_memory() {
        measured.push("memory refused to the guest before its kernel runs out");
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

    /// RW 4.32: the guest's console is the guest's own text — its kernel's and its standard error — and
    /// reaches the operator (and the probe's answer) escaped, a line at a time; an ordinary line unchanged.
    #[test]
    fn the_console_relay_shows_a_guests_control_sequences_escaped() {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(r"printf '\033]0;PWNED\007booted\r\nplain line\r\n'")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let out = child.stdout.take().unwrap();
        let kept = relay_console(out, true, Arc::new(std::sync::atomic::AtomicBool::new(false))).join().unwrap();
        let _ = child.wait();
        let text = String::from_utf8(kept).unwrap();
        assert!(!text.chars().any(|c| c.is_control() && c != '\n'), "a control character was relayed: {text:?}");
        assert!(text.contains(r"\u{1b}]0;PWNED\u{7}booted"), "shown, escaped: {text:?}");
        assert!(text.contains("plain line\n"), "an ordinary line is unchanged: {text:?}");
    }

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
        let l = |cpu| crate::jail::Limits { memory_bytes: 1 << 30, cpu_seconds: cpu, wall_seconds: None };
        assert_eq!(wall_ceiling(l(300)), Duration::from_secs(600));
        assert_eq!(wall_ceiling(l(5)), Duration::from_secs(60));
        let w = |cpu, wall| crate::jail::Limits { memory_bytes: 1 << 30, cpu_seconds: cpu, wall_seconds: Some(wall) };
        assert_eq!(wall_ceiling(w(300, 9)), Duration::from_secs(9), "the operator's wall narrows it");
        assert_eq!(wall_ceiling(w(5, 900)), Duration::from_secs(60), "and never widens it");
    }
}
