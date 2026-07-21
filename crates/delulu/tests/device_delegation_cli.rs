//! RFC 0001 phase F1 (build-order D12e) — **device-scoped delegation, end to end.**
//!
//! This is F1's one process-spawning test (the per-phase 5f rule; `grants_cli` held it for 5j,
//! `estop_cli` for 10g). Everything else about the dimension is unit-tested in
//! `delulu-broker/src/device_scope.rs` and `validate.rs`.
//!
//! The story it witnesses is the UAS lost-link pattern from `STAGE10_AUTONOMY_ADDENDUM.md` §2.2,
//! which was inexpressible until this phase:
//!
//! > At mission upload the operator delegates two grants — the mission grant (wide, live while the
//! > link heartbeats) and the lost-link grant (a strict `⊑` attenuation, typically a
//! > return-to-launch corridor only). The aircraft never has to *decide* what it may do when alone;
//! > it was told, mechanically, before takeoff.
//!
//! Before F1 the delegating side could say "you may actuate" and could not say "this far" — so both
//! grants above were the same grant, and `run --lease` refused a device by name rather than pretend
//! otherwise. Four things are witnessed here, in order:
//!
//! 1. A delegation carries an ENVELOPE, and a run under that lease commands the machine with it.
//! 2. The envelope actually bounds: a command outside the delegated corridor is refused as a VALUE,
//!    and the program keeps flying.
//! 3. The lost-link grant is a strict attenuation of the mission grant — and the attenuation is
//!    enforced at delegation time, so a *wider* second grant cannot be minted at all (DL0802).
//! 4. The holder cannot re-bound itself: a local `--grant actuator=` under `--lease` is refused,
//!    because choosing your own corridor is the delegating side's authority, not yours.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu_in(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

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

/// The mission grant: a wide corridor for the ordinary flight phase.
const MISSION: &str = "uav0/rudder:deflect_deg=-25..25,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";
/// The lost-link grant: the same device, a strict attenuation — the return-to-launch corridor.
const LOSTLINK: &str = "uav0/rudder:deflect_deg=-5..5,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";

/// Commands the rudder twice: once inside the narrow corridor, once outside it. Both outcomes are
/// printed, because a refusal is a VALUE here and the aircraft keeps its control loop.
fn rudder_program() -> &'static str {
    "module uav\n\n\
     type Cmd { deflect_deg: Float }\n\n\
     fn say(who: Str, r: Result[Unit, ActuateErr]) -> Str {\n\
     \x20 match r {\n\
     \x20   Ok(u) => who + \" COMMANDED\",\n\
     \x20   Err(e) => match e {\n\
     \x20     Envelope(reason) => who + \" REFUSED\",\n\
     \x20     LeaseRevoked(reason) => who + \" REVOKED\",\n\
     \x20     NoDevice => who + \" NODEVICE\"\n\
     \x20   }\n\
     \x20 }\n\
     }\n\n\
     fn main(root: Root) ! {Write, Actuate} {\n\
     \x20 let c = root.console()\n\
     \x20 let r = root.actuator(\"uav0/rudder\")\n\
     \x20 let small = Cmd { deflect_deg: 3.0 }\n\
     \x20 let large = Cmd { deflect_deg: 20.0 }\n\
     \x20 c.println(say(\"small\", r.command(small)))\n\
     \x20 c.println(say(\"large\", r.command(large)))\n\
     }\n"
}

#[test]
fn the_two_grant_lost_link_pattern_delegates_a_bounded_corridor_end_to_end() {
    let base = std::env::temp_dir().join(format!("delulu_devdel_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(cwd.join("uav.delulu"), rudder_program()).unwrap();

    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };

    // ----- 1. the mission grant: a delegation that CARRIES an envelope ---------------------------
    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--effects", "Actuate,Write", "--device", MISSION, "--multi", "--json"],
    );
    assert!(o.status.success(), "mission delegate failed: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let mission_token = v["token"].as_str().expect("lease token").to_string();
    let mission_node = v["node"].as_str().expect("node id").to_string();

    // The tree shows the corridor, not merely "Actuate" — the whole point of the phase.
    let o = delulu_in(&cwd, &state, &["grants", "inspect", &mission_node, "--json"]);
    assert!(o.status.success(), "inspect: {}", stderr(&o));
    let text = stdout(&o);
    assert!(
        text.contains("uav0/rudder") && text.contains("deflect_deg=-25..25"),
        "the grant tree carries the ENVELOPE, not just the effect:\n{text}"
    );

    // ----- 2. a run under that lease commands the machine, bounded by the delegation -------------
    let o = delulu_in(
        &cwd,
        &state,
        &["run", "uav.delulu", "--lease", &mission_token, "--broker-profile", "sim", "--no-prompt"],
    );
    assert!(o.status.success(), "a refusal is a value; the run exits 0: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("small COMMANDED"), "3° is inside the ±25° mission corridor:\n{out}");
    assert!(out.contains("large COMMANDED"), "20° is also inside ±25°:\n{out}");

    // ----- 3. the lost-link grant: a STRICT attenuation of the mission grant ---------------------
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "grants", "delegate", "--parent", &mission_node, "--effects", "Actuate,Write",
            "--device", LOSTLINK, "--multi", "--json",
        ],
    );
    assert!(o.status.success(), "the narrower corridor must delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let lostlink_token = v["token"].as_str().unwrap().to_string();

    // Under the lost-link grant the SAME program keeps flying, but the large deflection is refused.
    // Nothing in the program changed; only the authority it holds did.
    let o = delulu_in(
        &cwd,
        &state,
        &["run", "uav.delulu", "--lease", &lostlink_token, "--broker-profile", "sim", "--no-prompt"],
    );
    assert!(o.status.success(), "the aircraft keeps its control loop: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("small COMMANDED"), "3° is inside the ±5° return corridor:\n{out}");
    assert!(
        out.contains("large REFUSED"),
        "20° is OUTSIDE the ±5° corridor and must be refused as a value:\n{out}"
    );

    // ----- 4. anomaly response attenuates; it never widens ---------------------------------------
    // A child asking for MORE deflection than its parent holds is DL0802 at delegation time, so a
    // widened corridor cannot be minted at all — never mind used.
    let wider = "uav0/rudder:deflect_deg=-90..90,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";
    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--parent", &mission_node, "--effects", "Actuate", "--device", wider],
    );
    assert!(!o.status.success(), "a wider corridor must not delegate:\n{}", stdout(&o));
    let err = stderr(&o);
    assert!(err.contains("DL0802"), "widening is the attenuation failure, by code:\n{err}");

    // A device the parent never granted at all is equally refused — not silently dropped.
    let other = "uav0/elevator:deflect_deg=-5..5,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";
    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--parent", &mission_node, "--effects", "Actuate", "--device", other],
    );
    assert!(!o.status.success(), "a device the parent never held must not delegate:\n{}", stdout(&o));
    assert!(stderr(&o).contains("DL0802"), "…and it fails the same way: {}", stderr(&o));

    // ----- 5. the holder cannot re-bound itself --------------------------------------------------
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run", "uav.delulu", "--lease", &lostlink_token, "--grant",
            &format!("actuator={wider}"), "--broker-profile", "sim", "--no-prompt",
        ],
    );
    assert_eq!(o.status.code(), Some(2), "a local envelope under a lease is a usage error");
    let err = stderr(&o);
    assert!(
        err.contains("comes FROM the delegation"),
        "the refusal says WHY, and points at `grants delegate --device`:\n{err}"
    );
}

/// A malformed `--device` must be reported at the prompt with the parser's own message, never
/// accepted-and-dropped. A dropped envelope would delegate less than asked — safe, but baffling —
/// and the same reasoning already guards a typo'd `--effects` name one function over.
#[test]
fn a_malformed_device_envelope_is_refused_at_the_prompt_with_its_reason() {
    let base = std::env::temp_dir().join(format!("delulu_devdel_bad_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    // No daemon needed: the refusal happens during argument parsing, before any RPC.
    for (spec, want) in [
        ("uav0/rudder:deflect_deg=-25..25,heartbeat_ms=60000", "ttl_ms"),
        ("uav0/rudder:deflect_deg=-25..25,ttl_ms=60000,fail=hold", "heartbeat_ms"),
        ("uav0/rudder:heartbeat_ms=1,ttl_ms=2,fail=hold", "bounded dimensions"),
        ("uav0/rudder:deflect_deg=NaN..25,heartbeat_ms=1,ttl_ms=2,fail=hold", "non-finite"),
        ("uav0/rudder:deflect_deg=25..-25,heartbeat_ms=1,ttl_ms=2,fail=hold", "inverted"),
        ("uav0/rudder:deflect_deg=-25..25,heartbeat_ms=600,ttl_ms=60,fail=hold", "shorter than"),
    ] {
        let o = delulu_in(&cwd, &state, &["grants", "delegate", "--effects", "Actuate", "--device", spec]);
        assert_eq!(o.status.code(), Some(2), "`{spec}` must be a usage error");
        let err = stderr(&o);
        assert!(err.contains("bad --device"), "the refusal names the flag: {err}");
        assert!(err.contains(want), "the refusal carries the parser's reason (`{want}`): {err}");
    }
}
