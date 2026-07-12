//! Phase 5i — the microVM isolation profile (spec §6). **Linux-first, stated honestly** (playbook
//! trap 8): `delulu run --isolation microvm` requires Linux x86_64/aarch64 with KVM and a
//! provisioned microVM runtime (Firecracker or cloud-hypervisor). This module compiles ONLY on
//! Linux (`main.rs` gates the `mod` declaration); every other platform refuses with DL1408 in
//! `cli.rs` without ever reaching here.
//!
//! **v0.5 status (platform-pending, spec §11 chunk 5).** This module is the Linux-side *gate*: it
//! probes the prerequisites (KVM device, a VMM binary) and reports precisely which one is missing.
//! The guest launch itself — read-only rootfs image, virtio-fs mounts matching the granted `fs.*`
//! scopes exactly, the default-deny egress proxy enforcing the `net` allowlist, and the vsock
//! broker proxy holding ONE node (invariant 25) — is not yet wired; a fully-provisioned host still
//! receives an honest DL1408 naming that pending work rather than a fake "microVM" that is really
//! something weaker. Criterion 8 (the egress-deny test) is gated on `cfg(delulu_kvm)` in
//! `tests/microvm_criterion8.rs` and unignores the day this launch path lands on Linux+KVM CI.

use std::path::Path;

/// The VMM binaries the microVM profile can drive (spec §6: Firecracker-class).
const VMM_CANDIDATES: &[&str] = &["firecracker", "cloud-hypervisor"];

/// Probe the microVM prerequisites on this Linux host. Always returns `Err(detail)` in v0.5: the
/// detail names the FIRST missing prerequisite, or — with all prerequisites present — the honest
/// "guest launch is platform-pending" message (never a silent fallback to weaker isolation).
pub fn probe() -> Result<(), String> {
    if !Path::new("/dev/kvm").exists() {
        return Err("KVM is unavailable on this host (`/dev/kvm` not present)".to_string());
    }
    let Some(vmm) = find_vmm() else {
        return Err(format!(
            "no microVM monitor found on PATH (looked for: {})",
            VMM_CANDIDATES.join(", ")
        ));
    };
    Err(format!(
        "`{vmm}` and KVM are present, but the v0.5 guest launch (read-only rootfs, virtio-fs \
         scope mounts, default-deny egress proxy, vsock broker proxy) is platform-pending — \
         tracked in STAGE5_SPECIFICATION.md §11 (criterion 8, Linux+KVM CI)"
    ))
}

/// Locate the first available VMM binary on PATH (executable-bit checked).
fn find_vmm() -> Option<&'static str> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for cand in VMM_CANDIDATES {
            let p = dir.join(cand);
            if is_executable(&p) {
                return Some(cand);
            }
        }
    }
    None
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::metadata(p).map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}
