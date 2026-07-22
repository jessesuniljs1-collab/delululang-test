//! RFC 0001 phases F2/F3 — **broker federation, end to end through the real binary.**
//!
//! This is federation's one process-spawning test (the per-phase 5f rule). Everything else about
//! certificates is unit-tested in `delulu-broker/src/cert.rs` and `delulu/src/cert_crypto.rs`.
//!
//! The gap this closes is `STAGE10_AUTONOMY_ADDENDUM.md` §2.5, which called broker federation "a
//! prerequisite for any real deployment in this addendum's domains". Before it, criterion 10's
//! satellite demonstration ran **both broker roles inside one host over a simulated link** and its
//! recording said so — witnessing grant *semantics*, not the federation transport.
//!
//! What runs here is different in the way that matters:
//!
//! - **Two identities, no shared secret.** The ground and the vehicle hold different ed25519 keys.
//!   Neither can mint the other's credentials. (A lease token could never do this: it carries no
//!   authority bytes and is MAC'd with a *symmetric* key, so a verifier must be able to forge.)
//! - **The credential crosses as a FILE.** No socket is opened between the two sides; the broker
//!   never gains a listener. Carrying the bytes is the operator's existing business — a pass, a
//!   file drop, a store-and-forward queue. DeluluLang does not own the radio.
//! - **The vehicle's broker is a real broker**, not a cache. After adoption, `Actuate` round-trips
//!   to it locally and synchronously, and the link is never in the command path.
//!
//! What it still does NOT witness, so the closure is not read wider than it is: both processes run
//! on one machine, the "link" is a filesystem copy, and there is no radio, no latency, and no
//! partition except the one the test creates by not copying a file. F4 (the uplink lease) and F5
//! (audit reconciliation) are what make an outage mean something.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(cwd: &Path, state: Option<&Path>, args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
    c.current_dir(cwd).env("DELULU_NO_FIRST_RUN", "1");
    match state {
        Some(s) => {
            c.env("DELULU_STATE_DIR", s);
        }
        // The ground side is OFFLINE — it must not need, or touch, a broker state directory.
        None => {
            c.env_remove("DELULU_STATE_DIR");
        }
    }
    c.args(args).output().expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// Stops the vehicle daemon on drop, from a stable cwd, never panicking.
struct DaemonGuard {
    state: PathBuf,
}
impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.state)
            .args(["broker", "stop"])
            .output();
    }
}

/// The high-gain antenna's contact-window corridor: wide, because the ground is watching.
const HGA: &str = "sat0/hga:slew_deg=-45..45,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";
/// The autonomy corridor: the box the spacecraft is trusted to keep itself inside, alone.
const WHEELS: &str = "sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=60000,ttl_ms=60000,fail=hold";

fn sat_program() -> &'static str {
    "module sat\n\
     type Slew { slew_deg: Float }\n\
     fn say(who: Str, r: Result[Unit, ActuateErr]) -> Str {\n\
     \x20 match r {\n\
     \x20   Ok(u) => who + \" COMMANDED\",\n\
     \x20   Err(e) => match e {\n\
     \x20     Envelope(reason) => who + \" REFUSED\",\n\
     \x20     LeaseRevoked(reason) => who + \" REVOKED\",\n\
     \x20     NoDevice => who + \" NODEVICE\"\n\
     \x20   }\n\
     \x20 }\n\
     }\n\
     fn main(root: Root) ! {Write, Actuate} {\n\
     \x20 let c = root.console()\n\
     \x20 let hga = root.actuator(\"sat0/hga\")\n\
     \x20 c.println(say(\"point\", hga.command(Slew { slew_deg: 12.0 })))\n\
     \x20 c.println(say(\"slam\", hga.command(Slew { slew_deg: 80.0 })))\n\
     }\n"
}

struct Fixture {
    cwd: PathBuf,
    state: PathBuf,
    ground_key: PathBuf,
    vehicle_key: PathBuf,
    ground_pub: String,
    vehicle_pub: String,
}

fn setup(tag: &str) -> Fixture {
    let base = std::env::temp_dir().join(format!("delulu_fed_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("vehicle-state");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(cwd.join("sat.delulu"), sat_program()).unwrap();
    let ground_key = cwd.join("ground.key");
    let vehicle_key = cwd.join("vehicle.key");
    let gk = ground_key.to_string_lossy().to_string();
    let vk = vehicle_key.to_string_lossy().to_string();
    let ground_pub = stdout(&delulu(&cwd, None, &["grants", "pubkey", "--key", &gk])).trim().to_string();
    let vehicle_pub = stdout(&delulu(&cwd, None, &["grants", "pubkey", "--key", &vk])).trim().to_string();
    assert_eq!(ground_pub.len(), 64, "a public key is 32 bytes of hex");
    assert_ne!(ground_pub, vehicle_pub, "two independent identities, no shared secret");
    Fixture { cwd, state, ground_key, vehicle_key, ground_pub, vehicle_pub }
}

impl Fixture {
    fn gk(&self) -> String {
        self.ground_key.to_string_lossy().to_string()
    }
    fn vk(&self) -> String {
        self.vehicle_key.to_string_lossy().to_string()
    }
    /// The ground mints a certificate. Offline: no state dir, no daemon, nothing to connect to.
    fn certify(&self, args: &[&str]) -> Output {
        let mut a = vec!["grants", "certify"];
        a.extend_from_slice(args);
        delulu(&self.cwd, None, &a)
    }
    fn vehicle(&self, args: &[&str]) -> Output {
        delulu(&self.cwd, Some(&self.state), args)
    }
}

/// The whole loop: two keys, an offline mint, a file crossing, an anchored verification, a local
/// adoption, an on-board delegation, and a physical command bounded by what the ground signed.
#[test]
fn a_grant_certificate_carries_bounded_device_authority_across_a_broker_boundary() {
    let f = setup("loop");

    // ----- ground: mint a contact-window certificate, entirely offline -------------------------
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate,Write", "--device", HGA,
        "--ttl", "20m", "--key", &f.gk(), "--out", "pass1.dlcert",
    ]);
    assert!(o.status.success(), "offline certify failed: {}", stderr(&o));
    let cert_text = std::fs::read_to_string(f.cwd.join("pass1.dlcert")).unwrap();
    assert!(cert_text.starts_with("dlcert1\n"), "versioned magic from the first byte:\n{cert_text}");
    assert!(cert_text.contains("alg: ed25519"), "self-describing algorithm:\n{cert_text}");
    assert!(cert_text.contains("parent: anchor"), "this one is anchored");
    assert!(cert_text.contains("slew_deg=-45..45"), "the corridor travels WITH the credential");

    // ----- vehicle: its own broker, its own state, no shared key ------------------------------
    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "vehicle broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    // An unknown issuer is refused — never assumed trustworthy for being well-formed.
    let o = f.vehicle(&["grants", "adopt", "pass1.dlcert", "--anchor", &f.vehicle_pub]);
    assert!(!o.status.success(), "a chain rooted in an unconfigured key must not adopt");
    assert!(stderr(&o).contains("DL1415"), "…and it fails by code: {}", stderr(&o));

    // With no anchors at all, nothing is trusted (fail-closed, not fail-open).
    let o = f.vehicle(&["grants", "adopt", "pass1.dlcert"]);
    assert_eq!(o.status.code(), Some(2), "no anchors trusts nothing");
    assert!(stderr(&o).contains("no trust anchors"), "{}", stderr(&o));

    // ----- adopt against the real anchor -------------------------------------------------------
    let o = f.vehicle(&["grants", "adopt", "pass1.dlcert", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "adopt failed: {}", stderr(&o));
    let node = stdout(&o).trim().to_string();
    assert!(node.starts_with("g_"), "adoption yields a local grant id: {node}");
    assert!(
        stderr(&o).contains("expires:") && stderr(&o).contains("ONLY bound left"),
        "the expiry is surfaced, and named as what survives a partition:\n{}",
        stderr(&o)
    );

    // The adopted node is a REAL local node: a root here, bounded by what the ground signed.
    let o = f.vehicle(&["grants", "inspect", &node]);
    let text = stdout(&o);
    assert!(text.contains("(root)"), "a root in the vehicle's own tree:\n{text}");
    assert!(text.contains("slew_deg=-45..45"), "carrying the signed corridor:\n{text}");
    assert!(text.contains("federated"), "and marked as federated in origin:\n{text}");

    // ----- the vehicle delegates onward to its own program ------------------------------------
    let o = f.vehicle(&[
        "grants", "delegate", "--parent", &node, "--effects", "Actuate,Write",
        "--device", HGA, "--multi", "--json",
    ]);
    assert!(o.status.success(), "on-board delegate failed: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let token = v["token"].as_str().expect("token").to_string();

    // ----- the program flies under authority that crossed a machine boundary as a file ---------
    let o = f.vehicle(&["run", "sat.delulu", "--lease", &token, "--broker-profile", "sim", "--no-prompt"]);
    assert!(o.status.success(), "a refusal is a value; the run exits 0: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("point COMMANDED"), "12° is inside the signed ±45° corridor:\n{out}");
    assert!(
        out.contains("slam REFUSED"),
        "80° is OUTSIDE it — the ground's bound reached the actuator:\n{out}"
    );
}

/// Anomaly response attenuates; it never widens — across a chain, and caught at MINT time so an
/// operator learns now rather than the vehicle learning it out of contact.
#[test]
fn a_chained_certificate_attenuates_and_a_widening_is_refused_at_mint() {
    let f = setup("chain");

    // Ground → vehicle: both subsystems.
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate,Write", "--device", HGA, "--device", WHEELS,
        "--ttl", "20m", "--key", &f.gk(), "--out", "root.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));

    // Vehicle → payload: the autonomy corridor only, signed with the VEHICLE's key.
    let o = f.certify(&[
        "--subject", "0".repeat(64).as_str(), "--effects", "Actuate", "--device", WHEELS,
        "--ttl", "10m", "--key", &f.vk(), "--parent-cert", "root.dlcert", "--out", "leaf.dlcert",
    ]);
    assert!(o.status.success(), "a narrowing chain must mint: {}", stderr(&o));

    // Asking for MORE than the parent holds is caught at mint, by code, with the intersection.
    let wider = "sat0/wheels:slew_deg=-40..40,heartbeat_ms=60000,ttl_ms=60000,fail=hold";
    let o = f.certify(&[
        "--subject", "0".repeat(64).as_str(), "--effects", "Actuate", "--device", wider,
        "--ttl", "10m", "--key", &f.vk(), "--parent-cert", "root.dlcert",
    ]);
    assert!(!o.status.success(), "a widening chain must not mint:\n{}", stdout(&o));
    let err = stderr(&o);
    assert!(err.contains("DL1416"), "and it names the code the vehicle would have used: {err}");
    assert!(err.contains("most it can carry"), "…with the never-widening intersection: {err}");

    // A device the parent never held is the same refusal.
    let other = "sat0/thruster:burn_s=0..1,heartbeat_ms=60000,ttl_ms=60000,fail=hold";
    let o = f.certify(&[
        "--subject", "0".repeat(64).as_str(), "--effects", "Actuate", "--device", other,
        "--ttl", "10m", "--key", &f.vk(), "--parent-cert", "root.dlcert",
    ]);
    assert!(!o.status.success(), "a device the parent never held must not mint");

    // Signing a child with the WRONG key is caught at mint too: only the holder may delegate onward.
    let o = f.certify(&[
        "--subject", "0".repeat(64).as_str(), "--effects", "Actuate", "--device", WHEELS,
        "--ttl", "10m", "--key", &f.gk(), "--parent-cert", "root.dlcert",
    ]);
    assert!(!o.status.success(), "the ground does not hold what it delegated to the vehicle");
    assert!(
        stderr(&o).contains("Only the holder may delegate onward"),
        "and the message says why: {}",
        stderr(&o)
    );

    // The full two-hop chain adopts, and the holder gets the LEAF's authority (wheels only).
    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };
    let o = f.vehicle(&["grants", "adopt", "root.dlcert", "leaf.dlcert", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "the two-hop chain adopts: {}", stderr(&o));
    let node = stdout(&o).trim().to_string();
    let text = stdout(&f.vehicle(&["grants", "inspect", &node]));
    assert!(text.contains("sat0/wheels"), "the leaf's corridor:\n{text}");
    assert!(!text.contains("sat0/hga"), "NOT the root's — attenuation held across the link:\n{text}");
}

/// Tampering, replay, and expiry — the three things a bearer credential must survive.
#[test]
fn a_certificate_resists_tampering_replay_and_outliving_its_window() {
    let f = setup("attack");
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate,Write", "--device", HGA,
        "--ttl", "20m", "--key", &f.gk(), "--out", "pass.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));

    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    // 1. TAMPERING: widen the corridor in the signed text. The signature covers every field.
    let good = std::fs::read_to_string(f.cwd.join("pass.dlcert")).unwrap();
    let widened = good.replace("slew_deg=-45..45", "slew_deg=-90..90");
    assert_ne!(widened, good, "the substitution must actually apply");
    std::fs::write(f.cwd.join("tampered.dlcert"), &widened).unwrap();
    let o = f.vehicle(&["grants", "adopt", "tampered.dlcert", "--anchor", &f.ground_pub]);
    assert!(!o.status.success(), "a widened certificate must not adopt:\n{}", stdout(&o));
    assert!(stderr(&o).contains("DL1415"), "the signature no longer verifies: {}", stderr(&o));

    // 2. The genuine article adopts…
    let o = f.vehicle(&["grants", "adopt", "pass.dlcert", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "{}", stderr(&o));
    let node = stdout(&o).trim().to_string();

    // 3. REPLAY vs REVOCATION. The operator revokes; re-presenting the same certificate must NOT
    //    restore the authority, or revocation — the only tool an operator has while the link is
    //    up — would be defeatable by anyone holding a copy of the credential.
    let o = f.vehicle(&["grants", "revoke", &node]);
    assert!(o.status.success(), "revoke: {}", stderr(&o));
    let o = f.vehicle(&["grants", "adopt", "pass.dlcert", "--anchor", &f.ground_pub]);
    assert!(!o.status.success(), "re-adoption must not undo a revocation:\n{}", stdout(&o));
    assert!(
        stderr(&o).contains("already been adopted"),
        "and the refusal says exactly that: {}",
        stderr(&o)
    );

    // 4. EXPIRY. A certificate whose window has closed does not adopt. Expiry is the only bound
    //    that survives a partition, so it must be enforced by the RECEIVER's clock.
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate", "--device", HGA,
        "--ttl", "1ms", "--key", &f.gk(), "--out", "stale.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    std::thread::sleep(std::time::Duration::from_millis(20));
    let o = f.vehicle(&["grants", "adopt", "stale.dlcert", "--anchor", &f.ground_pub]);
    assert!(!o.status.success(), "an expired certificate must not adopt:\n{}", stdout(&o));
    assert!(stderr(&o).contains("DL1417"), "by code: {}", stderr(&o));
}

/// The ground side must work with no broker at all — an air-gapped machine can mint credentials.
/// If `certify` ever needed a daemon, the ground segment would have to run one to fly a spacecraft.
#[test]
fn the_ground_side_mints_credentials_with_no_broker_running() {
    let f = setup("offline");
    // `delulu(.., None, ..)` removes DELULU_STATE_DIR entirely; no daemon was ever started here.
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate", "--device", HGA,
        "--ttl", "20m", "--key", &f.gk(),
    ]);
    assert!(o.status.success(), "offline mint must not need a broker: {}", stderr(&o));
    assert!(stdout(&o).starts_with("dlcert1\n"), "the certificate goes to stdout:\n{}", stdout(&o));
    assert!(
        !stderr(&o).contains("DL1401") && !stderr(&o).to_lowercase().contains("unreachable"),
        "nothing here may reach for a daemon: {}",
        stderr(&o)
    );

    // A certificate with no expiry cannot be minted: across a partition, expiry is the only bound.
    let o = f.certify(&["--subject", &f.vehicle_pub, "--effects", "Actuate", "--key", &f.gk()]);
    assert_eq!(o.status.code(), Some(2), "--ttl is mandatory");
    assert!(stderr(&o).contains("ONLY bound that"), "and the reason is stated: {}", stderr(&o));
}
