//! Stage 10 phase 10g — the satellite scenario (spec §5.6, addendum §2.3, criterion 10).
//!
//! Criterion 10 wants a second simulated domain beyond the arm, witnessing three things: a
//! contact-window lease expiring at loss-of-signal, the pre-attenuated autonomy grant engaging,
//! and ground re-contact re-delegating. All three are here, run against the demonstration's own
//! committed program so it cannot rot.
//!
//! **The federation gap, stated where the tests are, not only in the prose** (addendum §2.5):
//! both broker roles run in ONE simulated host. There is no on-board broker, no ground broker and
//! no link between them. What is witnessed below is grant *semantics* — expiry, attenuation,
//! re-delegation — and NOT the cross-link transport, which does not exist and is RFC-gated. No
//! assertion in this file should ever be read as evidence that a spacecraft can hold a delegated
//! subtree across a real link.
//!
//! The mapping from the domain to the mechanism is exact, which is why the demonstration is worth
//! anything: **the contact window is the lease's `ttl_ms`.** Nothing simulates a radio.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn program() -> String {
    repo_root().join("measurements").join("satellite-demo").join("sat-pass.delulu")
        .to_string_lossy().to_string()
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(repo_root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// The ground-delegated authority for one pass. `ttl_ms` IS the contact window. The heartbeat is
/// generous relative to the program's cycle time, so a missed beat cannot masquerade as LOS — the
/// test below asserts the cause, not merely that something died.
const HGA: &str =
    "actuator=sat0/hga:slew_deg=-45..45,heartbeat_ms=200,ttl_ms=250,fail=safe-park";
/// The pre-attenuated autonomy grant: a narrow box, and a TTL that outlives the pass.
const WHEELS: &str =
    "actuator=sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=200,ttl_ms=600000,fail=hold";

fn pass() -> Output {
    delulu(&[
        "run", &program(), "--grant", "console", "--grant", HGA, "--grant", WHEELS,
        "--broker-profile", "sim", "--trace-effects", "--no-prompt",
    ])
}

/// The whole scenario, in one continuous run: contact, loss of signal, and autonomy afterwards.
#[test]
fn the_contact_window_expires_at_los_and_the_autonomy_grant_carries_on() {
    let o = pass();
    let out = stdout(&o);
    let err = stderr(&o);
    assert!(o.status.success(), "losing the ground is not a crash:\n{out}\n{err}");

    let lines: Vec<&str> = out.lines().collect();
    let los = lines
        .iter()
        .position(|l| l.starts_with("hga REVOKED:"))
        .expect("the contact window must end within the run — no LOS, no scenario");

    // ----- before LOS: the ground-delegated authority works -------------------------------------
    assert!(
        lines[..los].iter().any(|l| l.starts_with("hga COMMANDED")),
        "the HGA must have been commanded DURING contact, or nothing was delegated:\n{out}"
    );

    // ----- after LOS: it never works again -------------------------------------------------------
    // Not "mostly" and not "eventually": an expired contact window that let one more command
    // through would be a window that had not closed.
    assert!(
        !lines[los..].iter().any(|l| l.starts_with("hga COMMANDED")),
        "no HGA command may land after loss of signal:\n{out}"
    );

    // ----- the autonomy grant engages by being the one that did not end --------------------------
    assert_eq!(
        lines.iter().filter(|l| l.starts_with("wheels COMMANDED")).count(),
        100,
        "every station-keeping command must land, before and after LOS:\n{out}"
    );
    assert!(
        !out.contains("wheels REVOKED"),
        "the autonomy grant outlives the pass — that is what pre-attenuated means:\n{out}"
    );

    // ----- attenuation never widens ---------------------------------------------------------------
    // A ±40° slew offered to a ±0.5° box is refused on every cycle, INCLUDING after the ground is
    // gone. Losing supervision is precisely when a system must not acquire authority.
    assert_eq!(
        lines.iter().filter(|l| l.starts_with("wheels-wide REFUSED")).count(),
        100,
        "the autonomy box must refuse the wide slew every time:\n{out}"
    );
    assert!(
        lines[los..].iter().any(|l| l.starts_with("wheels-wide REFUSED")),
        "and it must still refuse it AFTER LOS — anomaly response attenuates, never widens:\n{out}"
    );

    // ----- the cause is the contact window, not a missed beat --------------------------------------
    // This is the assertion that makes the scenario mean what it says. A program that simply
    // stopped beating would also lose the HGA, and would look identical in the output above.
    assert!(
        out.contains("ttl-expired"),
        "LOS must be the TTL — the contact window — expiring:\n{out}"
    );
    assert!(
        !err.contains("missed-heartbeat"),
        "nothing here missed a beat; if it did, the scenario is measuring the wrong mechanism:\n{err}"
    );
    assert!(
        err.contains("safe-park"),
        "the HGA's declared fail-state must engage at LOS:\n{err}"
    );
    // The audit line reports the right kind of lateness: a lease held past its TTL, not a beat
    // that arrived late. The beats were arriving perfectly.
    assert!(
        err.contains("past its ttl"),
        "a TTL expiry must not be journaled as an overdue heartbeat:\n{err}"
    );
}

/// Re-contact: the next pass is a NEW delegation with a new lease. Authority does not come back —
/// it is issued again — and the second run is otherwise identical to the first, which is the
/// evidence that nothing was left half-open by the expiry.
#[test]
fn ground_re_contact_re_delegates_and_the_spacecraft_is_commandable_again() {
    let first = pass();
    let second = pass();
    for (n, o) in [("first", &first), ("second", &second)] {
        let out = stdout(o);
        assert!(o.status.success(), "{n} pass: {}", stderr(o));
        assert!(
            out.lines().any(|l| l.starts_with("hga COMMANDED")),
            "the {n} pass must command the HGA under its own fresh lease:\n{out}"
        );
        assert!(
            out.contains("ttl-expired"),
            "and each pass must end when its own contact window closes:\n{out}"
        );
    }
}

/// The second named gap, enforced rather than documented: a delegated node carries the authority
/// to actuate but cannot carry an ENVELOPE, so `--lease` refuses a local device grant and says why.
///
/// This is a regression test for a real fail-open shape. The lease path validates local grants by
/// enumerating the kinds it forbids — and every grant kind invented after that list was written
/// fell through the `else` and was silently DISCARDED. `actuator=` and `sensor=` were exactly
/// that: the operator typed a device grant, was told nothing, and the program died at the mint
/// with `DL0703: actuator was not granted` — a diagnostic blaming the program for the CLI having
/// thrown the grant away.
///
/// No daemon is spawned: the refusal happens during argument validation, before the redeem.
#[test]
fn a_lease_run_refuses_a_local_device_grant_by_name_instead_of_dropping_it() {
    for grant in [HGA, "sensor=sat0/sun-angle"] {
        let o = delulu(&[
            "run", &program(), "--lease", "dlt1_not-a-real-token", "--grant", grant, "--no-prompt",
        ]);
        let err = stderr(&o);
        assert_eq!(o.status.code(), Some(2), "a refused argument is exit 2:\n{err}");
        assert!(
            err.contains("cannot take a local `--grant actuator=`/`sensor=`"),
            "the refusal must name the device grant it is refusing:\n{err}"
        );
        assert!(
            err.to_lowercase().contains("envelope"),
            "and say WHY — a node cannot carry an envelope, so nobody could bound it:\n{err}"
        );
        // It must fail on the GRANT, not later on the bogus token: a refusal that only appeared
        // once the token was rejected would still be dropping device grants on the valid path.
        assert!(
            !err.contains("DL1407") && !err.contains("DL0703"),
            "the device grant is refused up front, before redeeming anything:\n{err}"
        );
    }
}

/// The control, and the reason the scenario is not circular: with no ground delegation at all, the
/// HGA is refused at the MINT (DL0703) — earlier and for a different reason than a lease expiring.
///
/// Without this, "the HGA stopped working" would be consistent with the grant never having done
/// anything. The autonomy grant still works here, so the run is not simply broken.
#[test]
fn with_no_ground_delegation_the_hga_is_refused_at_the_mint_not_at_los() {
    let o = delulu(&[
        "run", &program(), "--grant", "console", "--grant", WHEELS,
        "--broker-profile", "sim", "--no-prompt",
    ]);
    let err = stderr(&o);
    assert!(!o.status.success(), "an ungranted device must not be mintable:\n{}", stdout(&o));
    assert!(
        err.contains("DL0703") && err.contains("sat0/hga"),
        "the refusal names the device nobody granted:\n{err}"
    );
    assert!(
        !err.contains("ttl-expired"),
        "this is a mint refusal, not an expiry — conflating them would hide which one is broken:\n{err}"
    );
}
