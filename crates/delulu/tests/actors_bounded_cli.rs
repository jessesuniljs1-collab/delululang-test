//! Stage 10 phase 10c — bounded mailboxes + mailbox telemetry (Track B2/B3, spec §3).
//!
//! The law: a bounded mailbox bounds MEMORY, never liveness, and nothing is ever dropped
//! silently. `block` (the default for bounded actors) suspends the sending turn at the send
//! site; `drop-new` counts every drop and reports at exit — and in abort mode a drop is DL1902,
//! because abort mode is the statement that losing work is worse than stopping. The structural
//! exemption (a worker cannot wait on a mailbox only it can drain) is witnessed, not implied.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-mbox-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A consumer with a small bound; main produces far faster than the consumer drains. Sends come
/// from the MAIN thread (not a worker), so `block` backpressure engages fully.
fn producer_consumer(bound: &str) -> String {
    format!(
        "module m\n\nactor Sink(mailbox = {bound}) {{\n  var seen: Int\n  new() {{ self.seen = 0 }}\n  be item(n: Int) {{ self.seen = self.seen + n }}\n  be report(out: Cap[Console]) ! {{Write}} {{ out.println(str(self.seen)) }}\n}}\n\nfn main(root: Root) ! {{Async, Write}} {{\n  let s = spawn Sink()\n  var i = 0\n  while i < 500 {{\n    s.item(1)\n    i = i + 1\n  }}\n  s.report(root.console())\n}}\n"
    )
}

/// THE B2 CRITERION: a lopsided producer/consumer rate mismatch sustains at stable memory.
/// Under `block`, the mailbox depth may never exceed its bound (the queue IS the unbounded-
/// memory risk, so the bounded peak is the witness), and — because nothing drops — every
/// message still arrives: the count is exact.
#[test]
fn block_backpressure_sustains_a_lopsided_producer_at_bounded_depth() {
    let dir = scratch("block");
    let f = dir.join("pc.delulu");
    std::fs::write(&f, producer_consumer("8")).unwrap();
    let o = delulu(
        &["run", "pc.delulu", "--grant", "console", "--trace-memory"],
        &dir,
    );
    assert!(o.status.success(), "stderr: {}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim(),
        "500",
        "block loses nothing: every message arrived"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    let peak_line = err
        .lines()
        .find(|l| l.trim_start().starts_with("Sink:"))
        .unwrap_or_else(|| panic!("--trace-memory itemizes the bounded actor: {err}"));
    let peak: u64 = peak_line
        .split("peak depth ")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| panic!("peak parses from `{peak_line}`"));
    assert!(peak <= 8, "the mailbox never exceeded its bound: peak {peak} > 8\n{err}");
    assert!(peak_line.contains("overflow drops 0"), "block drops nothing: {peak_line}");
}

/// `drop-new`: overflow drops the new message, COUNTED — the exit note and the per-actor
/// telemetry both say so, and the received count honestly falls short of the sent count.
#[test]
fn drop_new_counts_and_reports_what_it_dropped() {
    let dir = scratch("dropnew");
    std::fs::write(dir.join("pc.delulu"), producer_consumer("4")).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"m\"\n[authority]\neffects = [\"Async\", \"Write\"]\n[actors]\noverflow = \"drop-new\"\n",
    )
    .unwrap();
    let o = delulu(
        &["run", "pc.delulu", "--grant", "console", "--trace-memory"],
        &dir,
    );
    assert!(o.status.success(), "drop-new outside abort mode is telemetry, not an error");
    let seen: i64 = String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(-1);
    let err = String::from_utf8_lossy(&o.stderr);
    if seen < 500 {
        assert!(
            err.contains("dropped by mailbox overflow"),
            "drops are never silent: {err}"
        );
    } else {
        // The consumer kept pace on this machine and nothing overflowed — a legal outcome;
        // the DL1902 test below forces the overflow deterministically either way.
        assert!(!err.contains("dropped by mailbox overflow"), "no drops → no drop note");
    }
}

/// DL1902: in abort mode, a `drop-new` overflow is an ERROR at the send site. The self-send
/// storm makes the overflow deterministic: a turn that enqueues 100 self-sends into a
/// bound-of-1 mailbox must overflow before they can drain (self-sends bypass `block`, but
/// `drop-new` has no waiting to bypass — a full mailbox drops, period).
#[test]
fn a_drop_new_overflow_in_abort_mode_is_dl1902() {
    let dir = scratch("abort");
    let src = "module m\n\nactor Storm(mailbox = 1) {\n  var n: Int\n  new() { self.n = 0 }\n  be seed() ! {Async} {\n    var i = 0\n    while i < 100 {\n      self.tick()\n      i = i + 1\n    }\n  }\n  be tick() { self.n = self.n + 1 }\n}\n\nfn main(root: Root) ! {Async, Write} {\n  let s = spawn Storm()\n  s.seed()\n  let c = root.console()\n  c.println(\"seeded\")\n}\n";
    std::fs::write(dir.join("storm.delulu"), src).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"m\"\n[authority]\neffects = [\"Async\", \"Write\"]\n[actors]\noverflow = \"drop-new\"\n",
    )
    .unwrap();
    let o = delulu(
        &["run", "storm.delulu", "--grant", "console", "--on-actor-death", "abort"],
        &dir,
    );
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(all.contains("DL1902"), "abort mode makes the drop an error: {all}");
}

/// THE STRUCTURAL EXEMPTION, witnessed: under `block`, a turn that self-sends past its own
/// bound may not wait — the only thread that could drain that mailbox is the one sending.
/// With one worker thread every actor shares that thread, so this program deadlocks in
/// seconds if the exemption is wrong and completes instantly if it is right.
#[test]
fn a_same_worker_send_past_the_bound_never_deadlocks() {
    let dir = scratch("selfsend");
    let src = "module m\n\nactor Chain(mailbox = 1) {\n  var n: Int\n  new() { self.n = 0 }\n  be seed(out: Cap[Console]) ! {Async, Write} {\n    var i = 0\n    while i < 50 {\n      self.tick()\n      i = i + 1\n    }\n    out.println(\"seeded 50\")\n  }\n  be tick() { self.n = self.n + 1 }\n}\n\nfn main(root: Root) ! {Async, Write} {\n  let c = spawn Chain()\n  c.seed(root.console())\n}\n";
    std::fs::write(dir.join("chain.delulu"), src).unwrap();
    let o = delulu(
        &["run", "chain.delulu", "--grant", "console", "--actors-threads=1", "--trace-memory"],
        &dir,
    );
    assert!(o.status.success(), "stderr: {}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "seeded 50");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.lines().any(|l| l.trim_start().starts_with("Chain:") && l.contains("peak depth")),
        "the bypass is visible in telemetry (peak past the bound), not hidden: {err}"
    );
}

/// The formatter round-trips the mailbox clause — the canonical shape is the parsed shape.
#[test]
fn fmt_round_trips_the_mailbox_clause() {
    let dir = scratch("fmt");
    let src = "module m\n\nactor A(mailbox = 100) {\n    var n: Int\n\n    new() {\n        self.n = 0\n    }\n\n    be tick() {\n        self.n = self.n + 1\n    }\n}\n\nfn main(root: Root) ! {Async, Write} {\n    let a = spawn A()\n    a.tick()\n    let c = root.console()\n    c.println(\"ok\")\n}\n";
    let f = dir.join("a.delulu");
    std::fs::write(&f, src).unwrap();
    let o = delulu(&["fmt", "--check", "a.delulu"], &dir);
    assert!(
        o.status.success(),
        "the mailbox clause survives fmt canonically:\n{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    let _ = root();
}
