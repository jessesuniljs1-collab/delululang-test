//! `delulu-fuzz [iterations] [seed]` — run a differential fuzz campaign (spec §7.2). Default is
//! 100_000 iterations, the Stage-2 acceptance floor. Exit code 0 iff no soundness violation.
//!
//! Note: accepted programs are actually executed, and the `Write` op prints to stdout, so a large
//! campaign is verbose; redirect stdout if you only want the final verdict on stderr.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let iterations: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(100_000);
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0x00C0_FFEE);

    eprintln!("delulu-fuzz: {iterations} iterations, seed {seed} …");
    let report = delulu_fuzz::run(iterations, seed);

    eprintln!(
        "generated={} accepted={} rejected={} unexpected_rejections={} check_only={}",
        report.generated,
        report.accepted,
        report.rejected,
        report.unexpected_rejections,
        report.accepted_check_only
    );

    if report.is_sound() {
        // Report the number of programs whose trace was ACTUALLY verified, not the number
        // accepted. The `Secret` families are checked but never run (the harness cannot grant a
        // secret), and counting them here would claim a verification that did not happen.
        let traced = report.accepted.saturating_sub(report.accepted_check_only);
        eprintln!(
            "SOUND: no trace escaped its row across {traced} executed programs \
             ({} more were accept/reject-checked only, never run).",
            report.accepted_check_only
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("UNSOUND — {} soundness violation(s), {} missed rejection(s):", report.soundness_violations.len(), report.missed_rejections.len());
        for (seed, detail) in report.soundness_violations.iter().take(5) {
            eprintln!("  [seed {seed}] {detail}");
        }
        for (seed, detail) in report.missed_rejections.iter().take(5) {
            eprintln!("  [seed {seed}] {detail}");
        }
        ExitCode::FAILURE
    }
}
