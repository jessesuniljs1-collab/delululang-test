//! RFC 0001 phases F2–F5 — **broker federation, end to end through the real binary.**
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
//! - **An outage costs the vehicle its actuator** (F4). The uplink lease is the dead-man principle
//!   applied to a link: a long-lived certificate with a short uplink term is bounded by the short
//!   one, so silence *shrinks* authority and only a signed contact receipt renews it.
//!
//! - **The ground finds out afterwards what the vehicle did alone** (F5), without either side
//!   pretending the two logs are one log. Two self-verifying chains joined by a hash reference —
//!   the mechanism a new day file already uses to link to the previous day, generalized across
//!   brokers instead of days.
//!
//! What it still does NOT witness, so the closure is not read wider than it is: both processes run
//! on one machine, the "link" is a filesystem copy, and there is no radio, no latency, and no
//! partition except the one these tests create by letting time pass. Real deployment additionally
//! needs a hardware driver (D23 shipped the adapter *mechanism* — a subprocess line protocol — but
//! no driver for any real device ships in-tree, so every device here is the simulator) and
//! certification regimes this project does not
//! control — `STAGE10_AUTONOMY_ADDENDUM.md` §3/§4 state both, and they are unchanged by this work.

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
    // Short names on purpose. The broker's socket lives inside the state dir, and macOS allows a
    // socket path of 103 bytes under a temp dir that is already 49 long. The first macOS CI run
    // (2026-09-14) refused the old names for the "revoke_restart" tag at 106 bytes, and they left
    // "reconcile" 2 bytes inside the limit; these come to at most 93 for every tag in this file.
    let base = std::env::temp_dir().join(format!("dfed_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("vstate");
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

/// **ADOPT-REPLAY-1 — a revocation must survive a daemon RESTART.**
///
/// The single-adoption / revoked-fingerprint memory that makes `grants revoke` stick against replay
/// (the test above, step 3) lived only in daemon memory. A bearer TOKEN survives a restart failing
/// closed — it is a reference into the tree, and the tree is wiped, so the referent is gone. A
/// CERTIFICATE does not: it is a self-contained, externally-held, signed artifact that re-verifies
/// against the anchor on its own, so wiping the tree does not invalidate it. The only thing that
/// stopped a *revoked* certificate from walking back in was the in-memory `revoked_adoption_fps`
/// set — and a restart cleared it. That is the ROTATE-1 shape one level up: an explicit operator
/// revocation silently reverted by a routine restart (crash, reboot, update), after which the same
/// certificate re-adopts into a fresh LIVE node holding the authority the operator killed.
///
/// The denylist is now persisted (`revoked_certs.json`) and reloaded at startup, so the revocation
/// holds across the restart while a *never-revoked* certificate still re-adopts normally (the
/// intended post-restart recovery path — proved by the sibling test).
#[test]
fn a_revoked_certificate_stays_revoked_across_a_daemon_restart() {
    let f = setup("revoke_restart");
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate,Write", "--device", HGA,
        "--ttl", "1h", "--key", &f.gk(), "--out", "pass.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));

    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    // Adopt, then revoke — the operator kills this credential's authority.
    let o = f.vehicle(&["grants", "adopt", "pass.dlcert", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "adopt: {}", stderr(&o));
    let node = stdout(&o).trim().to_string();
    let o = f.vehicle(&["grants", "revoke", &node]);
    assert!(o.status.success(), "revoke: {}", stderr(&o));

    // In-lifetime, re-adoption is already refused (the working guarantee).
    let o = f.vehicle(&["grants", "adopt", "pass.dlcert", "--anchor", &f.ground_pub]);
    assert!(!o.status.success(), "in-lifetime re-adoption must not undo a revocation:\n{}", stdout(&o));

    // ----- RESTART the vehicle daemon on the SAME state dir -------------------------------------
    assert!(f.vehicle(&["broker", "stop"]).status.success(), "stop for restart");
    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "restart: {}", stderr(&o));

    // The revocation MUST still hold: the same certificate must not re-adopt after a restart.
    let o = f.vehicle(&["grants", "adopt", "pass.dlcert", "--anchor", &f.ground_pub]);
    assert!(
        !o.status.success(),
        "ADOPT-REPLAY-1: a revoked certificate re-adopted after a daemon restart — the revocation \
         did not survive the restart:\n{}",
        stdout(&o)
    );
    assert!(
        stderr(&o).contains("revoked") || stderr(&o).contains("already been adopted"),
        "and the refusal names the revocation: {}",
        stderr(&o)
    );
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

// ===================================================================================================
// RFC 0001 phase F4 — the uplink lease. Revocation cannot cross a partition; expiry can.
// ===================================================================================================

/// **The property the whole phase exists for.** A long-lived certificate with a short uplink term
/// is bounded by the SHORT one, so a vehicle that stops hearing from the ground loses the actuator
/// on a schedule — without anyone sending a message, because at loss-of-signal there is nobody to
/// send one.
///
/// Timings are deliberately loose (a sub-second lease, a multi-second wait) because this asserts a
/// PATTERN — commanded, then not — never a cycle count. The D19 lesson: a timing test that only
/// passes on a fast build is a test that has not been run.
#[test]
fn silence_kills_the_actuator_and_a_contact_receipt_brings_it_back() {
    let f = setup("uplink");

    // A one-hour grant that may only run 800 ms without hearing from the ground.
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate,Write", "--device", HGA,
        "--ttl", "1h", "--uplink-ttl", "800ms", "--key", &f.gk(), "--out", "m.dlcert",
    ]);
    assert!(o.status.success(), "certify: {}", stderr(&o));
    let text = std::fs::read_to_string(f.cwd.join("m.dlcert")).unwrap();
    assert!(text.contains("uplink_ttl_ms: 800"), "the uplink term rides in the credential:\n{text}");

    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    let o = f.vehicle(&["grants", "adopt", "m.dlcert", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "adopt: {}", stderr(&o));
    let node = stdout(&o).trim().to_string();

    // The vehicle can say WHICH credential it holds — without this, an operator with a contact
    // receipt would have no way to find the node it renews.
    let inspect = stdout(&f.vehicle(&["grants", "inspect", &node]));
    let fingerprint = inspect
        .split('[')
        .nth(1)
        .and_then(|s| s.split(']').next())
        .expect("the holder line carries the certificate fingerprint")
        .to_string();
    assert_eq!(fingerprint.len(), 64, "a fingerprint is 32 bytes of hex: {inspect}");

    // A child delegated with NO deadline of its own — the escape a per-node expiry rule would have
    // allowed, and the reason expiry is inherited.
    let o = f.vehicle(&[
        "grants", "delegate", "--parent", &node, "--effects", "Actuate,Write",
        "--device", HGA, "--multi", "--json",
    ]);
    assert!(o.status.success(), "delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let token = v["token"].as_str().unwrap().to_string();

    // In contact: the antenna moves.
    let o = f.vehicle(&["run", "sat.delulu", "--lease", &token, "--broker-profile", "sim", "--no-prompt"]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("point COMMANDED"), "in contact, it points:\n{}", stdout(&o));

    // ----- loss of signal: nothing is sent, nobody is told, time simply passes ------------------
    std::thread::sleep(std::time::Duration::from_millis(2500));

    let o = f.vehicle(&["run", "sat.delulu", "--lease", &token, "--broker-profile", "sim", "--no-prompt"]);
    let combined = format!("{}{}", stdout(&o), stderr(&o));
    assert!(
        !combined.contains("point COMMANDED"),
        "after the uplink lease dies the antenna must NOT move — a child with no deadline of its \
         own must not outlive its root's lease:\n{combined}"
    );
    assert!(
        combined.contains("DL1402"),
        "and the reason is an expired lease, by code:\n{combined}"
    );

    // ----- re-contact: one receipt on the ROOT restores the whole subtree ----------------------
    let o = delulu(&f.cwd, None, &[
        "grants", "receipt", "--for", &fingerprint, "--ttl", "1h", "--key", &f.gk(), "--out", "r1.dlrcpt",
    ]);
    assert!(o.status.success(), "receipt: {}", stderr(&o));
    let o = f.vehicle(&["grants", "renew", "r1.dlrcpt", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "renew: {}", stderr(&o));

    let o = f.vehicle(&["run", "sat.delulu", "--lease", &token, "--broker-profile", "sim", "--no-prompt"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("point COMMANDED"), "contact restored the subtree, not just the root:\n{out}");
    assert!(out.contains("slam REFUSED"), "and the corridor still bounds it:\n{out}");
}

/// A receipt proves contact. It is not a grant, and it is bound to one credential.
#[test]
fn a_contact_receipt_grants_nothing_by_itself() {
    let f = setup("receipt");
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate", "--device", HGA,
        "--ttl", "1h", "--uplink-ttl", "1h", "--key", &f.gk(), "--out", "m.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };
    let o = f.vehicle(&["grants", "adopt", "m.dlcert", "--anchor", &f.ground_pub]);
    assert!(o.status.success(), "{}", stderr(&o));
    let node = stdout(&o).trim().to_string();
    let inspect = stdout(&f.vehicle(&["grants", "inspect", &node]));
    let fp = inspect.split('[').nth(1).and_then(|s| s.split(']').next()).unwrap().to_string();

    // A receipt for a credential this broker does not hold is refused — so a receipt for a
    // harmless grant cannot be replayed onto a powerful one.
    let o = delulu(&f.cwd, None, &[
        "grants", "receipt", "--for", &"a".repeat(64), "--ttl", "1h", "--key", &f.gk(), "--out", "wrong.dlrcpt",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    let o = f.vehicle(&["grants", "renew", "wrong.dlrcpt", "--anchor", &f.ground_pub]);
    assert!(!o.status.success(), "a receipt naming an unheld credential must not apply");
    assert!(stderr(&o).contains("no adopted certificate"), "{}", stderr(&o));

    // A receipt signed by a key that is not an anchor is refused.
    let o = delulu(&f.cwd, None, &[
        "grants", "receipt", "--for", &fp, "--ttl", "1h", "--key", &f.vk(), "--out", "self.dlrcpt",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    let o = f.vehicle(&["grants", "renew", "self.dlrcpt", "--anchor", &f.ground_pub]);
    assert!(!o.status.success(), "a vehicle must not be able to renew its own lease");
    assert!(stderr(&o).contains("DL1415"), "{}", stderr(&o));
}

// ===================================================================================================
// RFC 0001 phase F5 — audit reconciliation. Two chains, cross-linked, never merged.
// ===================================================================================================

/// The ground finds out afterwards what the vehicle did while it was alone — without either side
/// pretending the two logs are one log.
///
/// A chain is `blake3(prev_hash ‖ record)` over one broker's monotone `seq`, so splicing two would
/// invalidate every hash after the splice. A "merged" log would be a lie or a rewrite, and this is
/// the one artifact whose entire value is that it is neither.
#[test]
fn the_vehicles_chain_reconciles_into_the_grounds_without_merging() {
    let f = setup("reconcile");
    let vehicle_audit = f.state.join("audit");
    let ground_audit = f.cwd.join("ground-audit");

    // ----- the vehicle does some work, alone -----------------------------------------------------
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate,Write", "--device", HGA,
        "--ttl", "1h", "--key", &f.gk(), "--out", "m.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    {
        let _guard = DaemonGuard { state: f.state.clone() };
        let o = f.vehicle(&["grants", "adopt", "m.dlcert", "--anchor", &f.ground_pub]);
        assert!(o.status.success(), "{}", stderr(&o));
        let node = stdout(&o).trim().to_string();
        let o = f.vehicle(&[
            "grants", "delegate", "--parent", &node, "--effects", "Actuate,Write",
            "--device", HGA, "--multi", "--json",
        ]);
        assert!(o.status.success(), "{}", stderr(&o));
        let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
        let token = v["token"].as_str().unwrap().to_string();
        let o = f.vehicle(&["run", "sat.delulu", "--lease", &token, "--broker-profile", "sim", "--no-prompt"]);
        assert!(o.status.success(), "{}", stderr(&o));
    } // daemon stops here, flushing its chain

    let va = vehicle_audit.to_string_lossy().to_string();
    let ga = ground_audit.to_string_lossy().to_string();

    // The vehicle's own chain stands on its own.
    let o = delulu(&f.cwd, None, &["audit", "verify", "--dir", &va]);
    assert!(o.status.success(), "the vehicle chain must verify: {}", stderr(&o));

    // ----- it exports a transcript ---------------------------------------------------------------
    let o = delulu(&f.cwd, None, &["audit", "bundle", "--dir", &va, "--out", "v.bundle", "--json"]);
    assert!(o.status.success(), "bundle: {}", stderr(&o));
    let b: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("bundle --json");
    let digest = b["digest"].as_str().unwrap().to_string();
    let head = b["head"].as_str().unwrap().to_string();
    assert!(b["records"].as_u64().unwrap() >= 3, "the vehicle logged real work: {b}");
    let wire = std::fs::read_to_string(f.cwd.join("v.bundle")).unwrap();
    assert!(wire.starts_with("dlbundle1\n"), "versioned magic: {:?}", &wire[..40.min(wire.len())]);

    // ----- the ground reconciles it into ITS OWN chain -------------------------------------------
    let o = delulu(&f.cwd, None, &["audit", "reconcile", "v.bundle", "--dir", &ga, "--json"]);
    assert!(o.status.success(), "reconcile: {}", stderr(&o));
    let r: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("reconcile --json");
    assert_eq!(r["digest"].as_str().unwrap(), digest, "the cross-link names the bundle");
    assert_eq!(r["head"].as_str().unwrap(), head, "and the sender head at that point");

    // The ground chain contains ONE reconcile record naming the other chain — not the other
    // chain's records.
    let o = delulu(&f.cwd, None, &["audit", "tail", "10", "--dir", &ga, "--json"]);
    let t: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let recs = t["records"].as_array().unwrap();
    assert_eq!(recs.len(), 1, "one cross-link, not an import of {} records", b["records"]);
    assert_eq!(recs[0]["action"].as_str().unwrap(), "reconcile");
    assert!(recs[0]["target"].as_str().unwrap().contains(&digest), "it names the digest");

    // Both chains still verify independently. That is the whole design: two self-verifying logs
    // joined by a hash reference, exactly the way a new day file already links to the previous day.
    assert!(delulu(&f.cwd, None, &["audit", "verify", "--dir", &ga]).status.success());
    assert!(delulu(&f.cwd, None, &["audit", "verify", "--dir", &va]).status.success());

    // A second reconciliation continues this chain's numbering rather than repeating a seq.
    let o = delulu(&f.cwd, None, &["audit", "reconcile", "v.bundle", "--dir", &ga]);
    assert!(o.status.success(), "{}", stderr(&o));
    let o = delulu(&f.cwd, None, &["audit", "tail", "10", "--dir", &ga, "--json"]);
    let t: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let recs = t["records"].as_array().unwrap();
    assert_eq!(recs.len(), 2);
    assert_ne!(recs[0]["seq"], recs[1]["seq"], "two reconciliations must not share a seq");
    assert!(delulu(&f.cwd, None, &["audit", "verify", "--dir", &ga]).status.success());
}

/// **A tampered bundle is an INCIDENT, not a denial.**
///
/// The audit log is observability, not enforcement (spec §7, playbook trap 6). A bundle that does
/// not verify must be reported loudly and recorded — and must never gate a vehicle's ability to
/// operate. Getting this backwards would quietly convert the log into an enforcement input, which
/// is the rule this project keeps most carefully.
#[test]
fn a_tampered_bundle_is_reported_and_recorded_but_denies_nothing() {
    let f = setup("tamper");
    let ga = f.cwd.join("ground-audit").to_string_lossy().to_string();

    // Build a real bundle from a real chain.
    let o = f.certify(&[
        "--subject", &f.vehicle_pub, "--effects", "Actuate", "--device", HGA,
        "--ttl", "1h", "--key", &f.gk(), "--out", "m.dlcert",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    let o = f.vehicle(&["broker", "start"]);
    assert!(o.status.success(), "{}", stderr(&o));
    {
        let _guard = DaemonGuard { state: f.state.clone() };
        assert!(f.vehicle(&["grants", "adopt", "m.dlcert", "--anchor", &f.ground_pub]).status.success());
    }
    let va = f.state.join("audit").to_string_lossy().to_string();
    let o = delulu(&f.cwd, None, &["audit", "bundle", "--dir", &va, "--out", "v.bundle"]);
    assert!(o.status.success(), "{}", stderr(&o));

    // Alter one record's decision from allow to deny — a plausible cover-up.
    let good = std::fs::read_to_string(f.cwd.join("v.bundle")).unwrap();
    let bad = good.replacen("\"decision\":\"allow\"", "\"decision\":\"deny\"", 1);
    assert_ne!(bad, good, "the substitution must actually apply");
    std::fs::write(f.cwd.join("bad.bundle"), &bad).unwrap();

    let o = delulu(&f.cwd, None, &["audit", "reconcile", "bad.bundle", "--dir", &ga]);
    assert_eq!(o.status.code(), Some(1), "a failed reconciliation is exit 1 — look at this");
    let err = stderr(&o);
    assert!(err.contains("INCIDENT"), "it is reported as an incident: {err}");
    assert!(
        err.contains("does NOT withdraw any authority") && err.contains("observability, not enforcement"),
        "and says so, because the log must never become an enforcement input: {err}"
    );

    // The failure is RECORDED in the ground chain, and that chain still verifies.
    let o = delulu(&f.cwd, None, &["audit", "tail", "5", "--dir", &ga, "--json"]);
    let t: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let recs = t["records"].as_array().unwrap();
    assert_eq!(recs.len(), 1, "the attempt is on the record");
    assert_eq!(recs[0]["decision"].as_str().unwrap(), "deny", "recorded as a failed reconciliation");
    assert!(recs[0]["target"].as_str().unwrap().contains("UNVERIFIED"), "{:?}", recs[0]);
    assert!(delulu(&f.cwd, None, &["audit", "verify", "--dir", &ga]).status.success());

    // And a dropped middle segment is caught when the expected start is supplied — which is why
    // omitting `--expect-start` is only right for a first contact.
    let o = delulu(&f.cwd, None, &[
        "audit", "reconcile", "v.bundle", "--dir", &ga, "--expect-start", &"9".repeat(64),
    ]);
    assert_eq!(o.status.code(), Some(1), "a segment that does not chain onto the expected head");
    assert!(stderr(&o).contains("does not chain onto"), "{}", stderr(&o));
}
