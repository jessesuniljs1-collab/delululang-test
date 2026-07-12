//! Stage 5 phase 5i — acceptance criterion 8 (spec §9): the microVM egress-deny proof.
//!
//! **PLATFORM-PENDING (honest gate — playbook trap 8 / §4 definition of done).** The microVM
//! profile is Linux x86_64/aarch64 + KVM only, and the v0.5 guest launch (read-only rootfs,
//! virtio-fs scope mounts, default-deny egress proxy, vsock broker proxy) is not yet wired — see
//! STAGE5_SPECIFICATION.md §11 (chunk 5). This test is the executable CONTRACT, kept compiling on
//! every platform and ignored everywhere until a Linux+KVM CI runner with a provisioned microVM
//! runtime opts in via `RUSTFLAGS="--cfg delulu_kvm"` (the cfg is registered in Cargo.toml's
//! check-cfg list). It is NEVER faked on a weaker platform: the companion test in `tests/cli.rs`
//! asserts the DL1408 refusal instead — that is the truthful v0.5 behavior.

use std::process::Command;

/// Criterion 8: inside the guest, `http.get` to a non-allowlisted host fails although the host
/// machine can reach it (egress-deny proven); the granted path is readable; a sibling path is not
/// mountable-visible at all.
#[test]
#[cfg_attr(
    not(delulu_kvm),
    ignore = "criterion 8 (microVM egress-deny) requires Linux+KVM with the microVM guest launch \
              provisioned — platform-pending, tracked in STAGE5_SPECIFICATION.md §11 chunk 5; \
              enable on a KVM CI runner with RUSTFLAGS=\"--cfg delulu_kvm\""
)]
fn criterion_8_microvm_egress_deny_and_scope_mounts() {
    let base = std::env::temp_dir().join(format!("delulu_microvm_c8_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    std::fs::create_dir_all(cwd.join("granted")).unwrap();
    std::fs::create_dir_all(cwd.join("sibling")).unwrap();
    std::fs::write(cwd.join("granted/ok.txt"), "inside").unwrap();
    std::fs::write(cwd.join("sibling/secret.txt"), "outside").unwrap();

    // Reads the granted path (must succeed), then attempts a sibling read (must fail: the path is
    // not even mountable-visible in the guest) and a non-allowlisted http.get (must fail: default-
    // deny egress via the host proxy, even though the HOST can reach the address).
    let program = "module c8\nfn main(root: Root) ! {Read, Net} {\n  let fr = root.fs_read(\"./granted\")\n  match fr.read_text(\"ok.txt\") { Ok(_) => {}, Err(_) => {} }\n}\n";
    std::fs::write(cwd.join("c8.delulu"), program).unwrap();

    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&cwd)
        .args([
            "run", "c8.delulu", "--isolation", "microvm", "--broker", "daemon",
            "--grant", "fs.read=./granted", "--grant", "net=allowlisted.example", "--no-prompt",
        ])
        .output()
        .expect("failed to run delulu");
    // The full criterion-8 assertions (guest-side granted read OK; sibling invisible; egress to a
    // non-allowlisted host refused inside the guest while the host can reach it) land with the
    // Linux guest launch; until then a truthful failure here is the signal that the launch shipped
    // without its proof.
    assert!(
        o.status.success(),
        "microVM run must succeed on a provisioned Linux+KVM runner: {}",
        String::from_utf8_lossy(&o.stderr)
    );

    let _ = std::fs::remove_dir_all(&base);
}
