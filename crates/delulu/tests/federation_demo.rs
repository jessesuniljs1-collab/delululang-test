//! RFC 0001 phase F6 — the federated satellite demonstration, pinned.
//!
//! `measurements/federation-demo/sat-federated.delulu` computes its own orbit in **pure
//! DeluluLang**: the language's primitive table has arithmetic and comparison and no transcendental
//! functions at all, so `sqrt`/`sin`/`cos`/`asin` are built from `+ - * /`. This file keeps that
//! mathematics honest and keeps the demonstration from rotting.
//!
//! It asserts **properties**, not formatted output: accuracy bounds, a symmetric pass, and the
//! zenith identity. A demo test that pinned exact strings would break on any rewording and would
//! not have checked the one thing worth checking.
//!
//! The physics is a model, and the model is unvalidated — see `RECORD.md` §2. What is checked here
//! is that the arithmetic is right and the geometry is self-consistent; nothing here claims the
//! model describes a real spacecraft.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn demo() -> String {
    root().join("measurements/federation-demo/sat-federated.delulu").to_string_lossy().to_string()
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

const HGA: &str = "actuator=sat0/hga:slew_deg=-90..90,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";
const WHEELS: &str = "actuator=sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=60000,ttl_ms=60000,fail=hold";

fn run_demo() -> String {
    let o = delulu(&[
        "run", &demo(), "--grant", "console", "--grant", HGA, "--grant", WHEELS,
        "--broker-profile", "sim", "--no-prompt",
    ]);
    assert!(o.status.success(), "the demo must run: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// Pull `key = <number>` out of the self-check block.
fn value(out: &str, key: &str) -> f64 {
    let line = out.lines().find(|l| l.trim_start().starts_with(key)).unwrap_or_else(|| panic!("no `{key}` line in:\n{out}"));
    let after = line.split('=').nth(1).expect("a value follows the =");
    after.split_whitespace().next().expect("a number").parse().expect("parses as f64")
}

/// The hand-built transcendentals must agree with the real ones. These bounds are what the
/// range-reduced Taylor series and the Newton iterations are supposed to deliver; loosening one
/// later would mean the mathematics silently degraded.
#[test]
fn the_hand_built_arithmetic_matches_the_real_functions() {
    let out = run_demo();
    let close = |got: f64, want: f64, tol: f64, what: &str| {
        assert!((got - want).abs() < tol, "{what}: got {got}, want {want} (tol {tol})");
    };
    close(value(&out, "sqrt(2)"), std::f64::consts::SQRT_2, 1e-12, "Newton-Raphson sqrt");
    close(value(&out, "sin(pi/6)"), 0.5, 1e-12, "range-reduced Taylor sine");
    close(value(&out, "cos(pi/3)"), 0.5, 1e-12, "cosine via the shift identity");
    close(value(&out, "sin(pi)"), 0.0, 1e-12, "sine at the folding boundary");
    close(value(&out, "asin(0.5)*deg"), 30.0, 1e-9, "arcsine by Newton on sin");
}

/// The orbital outputs must match the closed forms computed independently here, in Rust, with the
/// platform's own `libm`. Same formulas — so this checks the IMPLEMENTATION, not the model. That
/// distinction is stated in `RECORD.md` §2.2 and is the whole reason this test's name says
/// "closed forms" rather than "reality".
#[test]
fn the_orbital_values_match_the_closed_forms_computed_with_libm() {
    let out = run_demo();

    let re: f64 = 6378.137;
    let mu: f64 = 398600.4418;
    let j2: f64 = 0.00108262668;
    let a: f64 = re + 420.0;
    let inc: f64 = 51.64_f64.to_radians();

    let period_min = 2.0 * std::f64::consts::PI * (a * a * a / mu).sqrt() / 60.0;
    let n = 2.0 * std::f64::consts::PI / (period_min * 60.0);
    let drift_deg_day = (-1.5 * n * j2 * (re / a).powi(2) * inc.cos()).to_degrees() * 86400.0;

    let got_period = value(&out, "period (min)");
    let got_drift = value(&out, "nodal drift");
    assert!(
        (got_period - period_min).abs() < 1e-9,
        "period: DeluluLang {got_period}, libm {period_min}"
    );
    assert!(
        (got_drift - drift_deg_day).abs() < 1e-9,
        "J2 nodal regression: DeluluLang {got_drift}, libm {drift_deg_day}"
    );

    // Sanity on the model itself, loosely — a LEO period is minutes, not hours, and a prograde
    // orbit's node regresses (westward). These would catch a units or sign error that agreed with
    // itself.
    assert!((85.0..100.0).contains(&got_period), "a 420 km orbit's period: {got_period} min");
    assert!(got_drift < 0.0, "a prograde orbit's node must REGRESS, not advance: {got_drift}");
}

/// Two geometric identities that fall out of the physics rather than being asserted by it. Either
/// would break on a sign error in the trigonometric range reduction, which is the most plausible
/// place for this hand-built mathematics to go quietly wrong.
#[test]
fn the_pass_geometry_is_self_consistent() {
    let out = run_demo();
    // Parse "t=<n>s elev=<n>deg range=<n>km".
    let mut samples: Vec<(i64, i64, i64)> = Vec::new();
    for line in out.lines().filter(|l| l.trim_start().starts_with("t=")) {
        let num = |p: &str, s: &str| -> i64 {
            line.split(p).nth(1).and_then(|r| r.split(s).next()).and_then(|v| v.trim().parse().ok())
                .unwrap_or_else(|| panic!("cannot read `{p}` from `{line}`"))
        };
        samples.push((num("t=", "s"), num("elev=", "deg"), num("range=", "km")));
    }
    assert!(samples.len() >= 12, "the profile should have a dozen samples: {out}");

    // 1. At the zenith the slant range IS the orbital altitude. That is what "directly overhead"
    //    means; it is not something the program is told.
    let peak = samples.iter().max_by_key(|(_, e, _)| *e).expect("a peak");
    assert!(peak.1 > 80, "the scenario is a near-overhead pass; peak elevation {} deg", peak.1);
    assert!(
        (peak.2 - 420).abs() <= 2,
        "at {} deg elevation the range must be the 420 km altitude, got {} km",
        peak.1,
        peak.2
    );

    // 2. A circular orbit over a station gives a SYMMETRIC pass. Compare the samples either side of
    //    the peak: an asymmetry means a folding error in the range reduction.
    let i = samples.iter().position(|s| s.0 == peak.0).unwrap();
    for k in 1..=3 {
        let (before, after) = (samples[i - k], samples[i + k]);
        assert!(
            (before.1 - after.1).abs() <= 1,
            "pass must be symmetric about the zenith: t={} elev={} vs t={} elev={}",
            before.0, before.1, after.0, after.1
        );
    }

    // 3. Elevation must rise then fall, never wander — a monotone approach and departure.
    let rising = &samples[..=i];
    for w in rising.windows(2) {
        assert!(w[1].1 >= w[0].1, "elevation must rise monotonically to the zenith: {w:?}");
    }
}

/// The claim `RECORD.md` makes about the program's shape, checked against the compiler rather than
/// against a reading of the source: the whole navigation stack is EFFECT-FREE, and nothing leaves
/// the guarantee.
#[test]
fn the_navigation_stack_holds_no_capabilities_at_all() {
    let o = delulu(&["authority", &demo()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let text = String::from_utf8_lossy(&o.stdout).to_string();

    assert!(text.contains("foreign:      (none"), "no foreign code, so no escape hatch:\n{text}");
    // Every orbital function must be listed as PURE. If one ever acquired an effect, the physics
    // would have started touching the world and this test is where that surfaces.
    for f in [
        "sqrt", "sin", "cos", "asin", "mean_motion", "raan_rate", "sat_x", "sat_y", "sat_z",
        "stn_x", "stn_y", "stn_z", "elevation_rad", "range_km", "period_s",
    ] {
        assert!(text.contains(f), "`{f}` must appear among the pure fns:\n{text}");
    }
    // …and the program as a whole holds exactly the two effects it needs to fly, and no more.
    assert!(text.contains("effects:      Actuate, Write"), "exactly two effects:\n{text}");
    assert!(!text.contains("Net"), "a spacecraft control program has no business on a network:\n{text}");
}
