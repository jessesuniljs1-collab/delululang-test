//! The `delulu` CLI (spec §9.5). Terminal-first: everything the language can do is reachable
//! here, with `--json` for machine consumers (agents) and human-rendered output otherwise
//! (Constitution §8.4). Exit codes are part of the stable contract: 0 ok / 1 diagnostics /
//! 2 internal.

mod broker_client;
mod broker_ipc;
mod broker_transport;
mod brokerd;
mod cli;
mod foreign_worker;
mod locale;
// Phase 5i: the microVM profile is Linux-first (spec §6); the probe/launch module compiles only
// there. Every other platform refuses `--isolation microvm` with DL1408 in `cli.rs` (trap 8 —
// never fake a weaker platform's isolation as equivalent).
#[cfg(target_os = "linux")]
mod microvm;
mod repl;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = cli::run(&args);
    ExitCode::from(code as u8)
}
