//! The microVM guest kernel's absences, pinned on every platform (the owner's question of 2026-10-19).
//!
//! The guest is `tinyconfig` plus `scripts/microvm/kernel.config`, and `build-image.sh` refuses a kernel in which any
//! `# CONFIG_X is not set` line of that file is on. So these lines are the guarantee, and this test keeps each one in the
//! file. Deleting one would let a later option bring the feature back, and the build would not object. Each line is
//! here for a defect class the guest must not be able to reach: PwnEscape (CVE-2026-99999) went through a guest's
//! virtio-net packets into the host's IPv4 stack; Januscape (CVE-2026-53359) and Zapscape (CVE-2026-64561) through a
//! guest that hosts a guest of its own. The launcher's half (no network interface, no drive configured) is
//! `microvm.rs`'s `the_vm_is_given_one_vsock_and_nothing_else`, Linux only, as that module is.

use std::path::Path;

fn kernel_config() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/microvm/kernel.config");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn the_guest_kernel_has_no_network_device_no_socket_family_and_no_hypervisor() {
    let text = kernel_config();
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    for option in [
        // No network device to send through, and no stack to send with.
        "NETDEVICES",
        "VIRTIO_NET",
        "TUN",
        "VHOST_NET",
        "INET",
        "PACKET",
        "UNIX",
        // No VM of its own, and no code loaded after boot.
        "VIRTUALIZATION",
        "KVM",
        "MODULES",
        "BPF_SYSCALL",
        "IO_URING",
    ] {
        let off = format!("# CONFIG_{option} is not set");
        assert!(lines.contains(&off.as_str()), "kernel.config must say `{off}`, which build-image.sh enforces");
        assert!(
            !lines.iter().any(|l| l.starts_with(&format!("CONFIG_{option}="))),
            "kernel.config must not turn CONFIG_{option} on"
        );
    }
    // The guest's one device, and only it.
    assert!(lines.contains(&"CONFIG_VIRTIO_VSOCKETS=y"), "the vsock channel stays");
}
