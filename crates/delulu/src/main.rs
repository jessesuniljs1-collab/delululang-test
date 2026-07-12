//! The `delulu` CLI (spec §9.5). Terminal-first: everything the language can do is reachable
//! here, with `--json` for machine consumers (agents) and human-rendered output otherwise
//! (Constitution §8.4). Exit codes are part of the stable contract: 0 ok / 1 diagnostics /
//! 2 internal.

mod broker_client;
mod broker_ipc;
mod broker_transport;
mod brokerd;
mod cli;
mod repl;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = cli::run(&args);
    ExitCode::from(code as u8)
}
