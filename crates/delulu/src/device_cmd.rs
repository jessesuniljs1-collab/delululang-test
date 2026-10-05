//! P8-03: `delulu device sim` — the in-tree simulator run as a **device**, on a line.
//!
//! A `--broker-profile sim` run drives the simulator inside the host's own process. This verb runs
//! the same simulator ([`delulu_runtime::sim`]) as a separate process speaking `adapter.rs`'s
//! `CMD`/`READ` protocol on its standard input and output — the shape a serial bridge, a CAN gateway
//! or a vendor shim has. So the pieces P8-02 built can be exercised across a real process boundary,
//! against a device whose every reading this repository can predict:
//!
//! ```text
//! delulu run arm.delulu --grant console --grant 'actuator=arm0/elbow:…' \
//!   --broker-profile hw:bench --approved signoff.json \
//!   --adapter-dpx driver.dpx --adapter-signer HEX \
//!   --adapter-transport 'delulu device sim --seed 7 --actuator arm0/elbow:angle_deg=-60..60'
//! ```
//!
//! **The bounds this process is given are the DEVICE's, not the grant's.** `--actuator` here says
//! what the bench can physically do; the run's `--grant` says what the program is allowed to ask
//! for. Two different limits, enforced in two different places, and a run can tell which refused:
//! the host's envelope refusal never reaches the device at all, while the bench's own refusal comes
//! back as `ERR` and is reported as the hardware's. Giving the bench a NARROWER range than the grant
//! is how a hard stop is simulated — which is exactly what a real machine's limit switches are.
//!
//! Nothing here means hardware moved: it is a simulator, it models no physics, and it says so.

use serde_json::json;

use delulu_runtime::sim::Sim;
use delulu_runtime::value::ActuatorEnvelope;

pub fn cmd_device(rest: &[String]) -> i32 {
    let Some(verb) = rest.first().map(String::as_str) else {
        eprintln!("error: `device` needs a verb — `sim` (run the reference simulator as a device on standard input/output)");
        return 2;
    };
    match verb {
        "sim" => cmd_sim(&rest[1..]),
        other => {
            eprintln!("error: `device` has no verb `{other}` — the verb is `sim`");
            2
        }
    }
}

fn cmd_sim(rest: &[String]) -> i32 {
    let mut seed: u64 = 0;
    let mut actuators: Vec<ActuatorEnvelope> = Vec::new();
    let mut sensors: Vec<String> = Vec::new();
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let mut value = |name: &str| -> Option<String> {
            if i + 1 < rest.len() {
                i += 1;
                Some(rest[i].clone())
            } else {
                eprintln!("error: `{name}` needs a value");
                None
            }
        };
        match arg {
            "--json" => json = true,
            "--seed" => match value("--seed") {
                Some(v) => match v.parse::<u64>() {
                    Ok(n) => seed = n,
                    Err(_) => {
                        eprintln!("error: `--seed` takes a non-negative whole number (got `{v}`)");
                        return 2;
                    }
                },
                None => return 2,
            },
            // The bench's own bounds, in the grant form an operator already knows
            // (`DEVICE:dim=lo..hi,…,heartbeat_ms=…,ttl_ms=…,fail=…`). The dead-man terms are part of
            // that form and are ignored here: a lease is the HOST's business, and a device that
            // enforced one would be a second dead-man nobody declared.
            "--actuator" => match value("--actuator") {
                Some(spec) => match ActuatorEnvelope::parse(&spec) {
                    Ok(env) => actuators.push(env),
                    Err(why) => {
                        eprintln!("error: `--actuator {spec}`: {why}");
                        return 2;
                    }
                },
                None => return 2,
            },
            "--sensor" => match value("--sensor") {
                Some(name) if !name.trim().is_empty() => sensors.push(name.trim().to_string()),
                Some(_) => {
                    eprintln!("error: `--sensor` needs a device name");
                    return 2;
                }
                None => return 2,
            },
            other => {
                eprintln!("error: `device sim` does not know the option `{other}`");
                eprintln!("  nothing was done — an option nobody understood is refused, never ignored");
                return 2;
            }
        }
        i += 1;
    }
    if actuators.is_empty() && sensors.is_empty() {
        eprintln!(
            "error: `device sim` needs at least one `--actuator DEVICE:dim=lo..hi,heartbeat_ms=N,ttl_ms=N,fail=F` \
             or `--sensor NAME` — a bench with no devices would answer NODEV to everything, which reads \
             like a broken cable rather than a missing argument"
        );
        return 2;
    }
    // `--json` describes the bench and exits: a caller that wants to know what this process would be
    // can ask without starting a conversation it then has to end.
    if json {
        let devices: Vec<_> = actuators
            .iter()
            .map(|e| {
                json!({
                    "device": e.device,
                    "dims": e.dims.iter().map(|(d, lo, hi)| json!({ "dim": d, "lo": lo, "hi": hi })).collect::<Vec<_>>(),
                })
            })
            .collect();
        crate::cli::print_success_envelope(
            "device",
            json!({
                "command": "device", "subcommand": "sim", "seed": seed,
                "actuators": devices, "sensors": sensors,
                "protocol": "CMD <device> <dim>=<value>[,...] -> OK | ERR <reason>; READ <device> -> VAL <float> | NODEV",
                "simulated": true,
            }),
        );
        return 0;
    }
    let mut sim = Sim::new(seed, &actuators, &sensors);
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    delulu_runtime::sim::serve(&mut input, &mut output, &mut sim, &actuators);
    0
}
