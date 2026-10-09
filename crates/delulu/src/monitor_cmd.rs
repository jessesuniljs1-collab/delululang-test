//! P8-04: `delulu monitor watch` — an out-of-band monitor that quarantines a run by revoking it.
//!
//! The shape of a DPU-side watcher (NVIDIA's Sentry, `V2_OPENSHELL_STUDY.md` §4.7), in software and
//! in DeluluLang's own terms (D-V2-54, `V2_P8_DESIGN.md` P8-04):
//!
//! * **It holds one node and acts only as it.** The operator mints a monitor node `g_M` between
//!   themselves and the runs it watches (`grants delegate`, then each run's lease delegated with
//!   `--parent g_M`). The broker lets a caller revoke only its own node or a DESCENDANT
//!   (`tree.rs`: `is_self_or_descendant`), so a monitor acting as `g_M` can stop the runs under it
//!   and nothing else — not a sibling, not its own parent. The residual, named in D-V2-101: `g_M`
//!   HOLDS the runs' authority, because a parent must bound its child (R-7); what keeps the monitor
//!   from using it is that this program performs no effect but a revoke, and the same-uid boundary
//!   (§11.4, category 7). A revoke-only principal would make "exactly revoke" true; that is the
//!   owner's (option (c)).
//! * **It reads only what the host recorded** — the hash-chained audit, as it is appended — and
//!   trusts nothing a guest says. It resumes after the last record HASH it read, never a seq: a hash
//!   also shows a chain rewritten under it (and until D-V2-102 two writers numbered the chain apart).
//! * **Its rules are declarative and few:** `envelope` (any envelope refusal — a `use` `deny` with
//!   `refused_by: envelope`, D-V2-100) and `denies=N/MS` (N `deny` records within MS milliseconds).
//! * **A quarantine stops one run, not the fleet:** the target is the child of `g_M` on the path to
//!   the record's node — the run's own node — so a sibling run under the same monitor keeps going.
//!   The revocation's own audit record carries the rule, the count, the window and the evidence: a
//!   monitor that fires without saying what it saw is a monitor nobody can audit.
//!
//! It is a second, slower line. Each run's own dead-man (10f) remains the real-time guarantee; what
//! happens when the MONITOR dies is P8-04 step 5, not built here.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::broker_ipc::{ReqBody, Response};

/// One declarative rule.
#[derive(Clone, Debug, PartialEq)]
enum Rule {
    /// Any envelope refusal in the watched subtree.
    Envelope,
    /// `n` `deny` records within `ms` milliseconds (by the records' own timestamps), per run.
    Denies { n: usize, ms: i64 },
}

impl Rule {
    fn parse(s: &str) -> Result<Rule, String> {
        if s == "envelope" {
            return Ok(Rule::Envelope);
        }
        if let Some(spec) = s.strip_prefix("denies=") {
            let (n, ms) = spec.split_once('/').ok_or_else(|| format!("`{s}`: the form is denies=N/MS"))?;
            let n: usize = n.parse().ok().filter(|n| *n >= 1).ok_or_else(|| format!("`{s}`: N is a whole number ≥ 1"))?;
            let ms: i64 = ms.parse().ok().filter(|m| *m >= 1).ok_or_else(|| format!("`{s}`: MS is a whole number ≥ 1"))?;
            return Ok(Rule::Denies { n, ms });
        }
        Err(format!("`{s}` is not a rule — the rules are `envelope` and `denies=N/MS`"))
    }

    fn name(&self) -> String {
        match self {
            Rule::Envelope => "envelope".to_string(),
            Rule::Denies { n, ms } => format!("denies={n}/{ms}"),
        }
    }
}

pub fn cmd_monitor(rest: &[String]) -> i32 {
    let Some(verb) = rest.first().map(String::as_str) else {
        eprintln!("error: `monitor` needs a verb — `watch --node g_ID --rule RULE` (quarantine a run under g_ID by revoking it)");
        return 2;
    };
    match verb {
        "watch" => cmd_watch(&rest[1..]),
        other => {
            eprintln!("error: `monitor` has no verb `{other}` — the verb is `watch`");
            2
        }
    }
}

struct Opts {
    node: String,
    rules: Vec<Rule>,
    poll: Duration,
    for_ms: Option<u64>,
    state_dir: Option<String>,
    json: bool,
}

fn parse(rest: &[String]) -> Result<Opts, String> {
    let (mut node, mut rules, mut poll, mut for_ms, mut state_dir, mut json) =
        (None, Vec::new(), Duration::from_millis(250), None, None, false);
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let mut value = || -> Result<String, String> {
            i += 1;
            rest.get(i).cloned().ok_or_else(|| format!("`{arg}` needs a value"))
        };
        match arg {
            "--json" => json = true,
            "--node" => node = Some(value()?),
            "--rule" => rules.push(Rule::parse(&value()?)?),
            "--poll" => {
                let v = value()?;
                let ms: u64 = v.parse().ok().filter(|m| (10..=60_000).contains(m)).ok_or_else(|| {
                    format!("`--poll {v}`: milliseconds between reads, 10 to 60000")
                })?;
                poll = Duration::from_millis(ms);
            }
            "--for" => {
                let v = value()?;
                for_ms = Some(v.parse().ok().filter(|m| *m >= 1).ok_or_else(|| format!("`--for {v}`: milliseconds, ≥ 1"))?);
            }
            "--state-dir" => state_dir = Some(value()?),
            other => return Err(format!("unknown `monitor watch` argument `{other}`")),
        }
        i += 1;
    }
    let node = node.ok_or("`monitor watch` needs `--node g_ID` — the monitor's own node, an ancestor of the runs it watches")?;
    if rules.is_empty() {
        return Err("`monitor watch` needs at least one `--rule` (`envelope`, `denies=N/MS`) — a monitor with no rule watches nothing".into());
    }
    Ok(Opts { node, rules, poll, for_ms, state_dir, json })
}

/// For every node under (and including) `root`, the RUN it belongs to: the child of `root` on its
/// path, or `root` itself. Built from the daemon's own listing each poll, so a node minted mid-run (a
/// device's node) is watched from the moment it exists. Nodes outside the subtree are absent — the
/// monitor neither counts nor revokes them.
fn runs_under(root: &str, nodes: &[crate::broker_ipc::NodeInfo]) -> HashMap<String, String> {
    let parent: HashMap<&str, Option<&str>> = nodes.iter().map(|n| (n.id.as_str(), n.parent.as_deref())).collect();
    let mut out = HashMap::new();
    for n in nodes {
        let mut cur = n.id.as_str();
        let mut below: Option<&str> = None;
        // Bounded like the broker's own ancestry walk: ids are never reused, so a cycle is unreachable.
        for _ in 0..1024 {
            if cur == root {
                out.insert(n.id.clone(), below.unwrap_or(root).to_string());
                break;
            }
            match parent.get(cur).copied().flatten() {
                Some(p) => {
                    below = Some(cur);
                    cur = p;
                }
                None => break,
            }
        }
    }
    out
}

/// What the rules saw, for one quarantine.
struct Evidence {
    rule: String,
    records: Vec<(u64, String, i64)>, // (seq, hash, ts)
}

fn cmd_watch(rest: &[String]) -> i32 {
    let o = match parse(rest) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let Some(state_dir) = crate::brokerd::resolve_state_dir(o.state_dir.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };
    let audit_dir: PathBuf = state_dir.join("audit");
    let fail = |message: String| -> i32 {
        eprintln!("error[DL1401]: {message}");
        1
    };
    let list = || -> Result<Vec<crate::broker_ipc::NodeInfo>, String> {
        match crate::brokerd::request(&state_dir, ReqBody::List) {
            Ok(Response::Listed { nodes }) => Ok(nodes),
            Ok(other) => Err(format!("unexpected answer to a listing: {other:?}")),
            Err(e) => Err(format!(
                "broker unreachable: {} — a monitor that cannot reach its broker cannot quarantine anything",
                crate::cli::broker_unreachable_detail(&e)
            )),
        }
    };
    let read_chain = || -> Result<Vec<delulu_broker::audit::AuditRecord>, String> {
        if !audit_dir.exists() {
            return Ok(Vec::new());
        }
        delulu_broker::audit::query(&audit_dir, &delulu_broker::audit::QueryFilter::default())
            .map_err(|e| format!("the audit chain at `{}` cannot be read: {e}", audit_dir.display()))
    };

    // The watched node must exist and be live NOW — a monitor started on a typo watches nothing.
    let nodes = match list() {
        Ok(n) => n,
        Err(e) => return fail(e),
    };
    match nodes.iter().find(|n| n.id == o.node) {
        Some(n) if n.state == "live" => {}
        Some(n) => return fail(format!("`{}` is {} — there is nothing under it to watch", o.node, n.state)),
        None => return fail(format!("`{}` is not a node of this broker's tree", o.node)),
    }
    // Watch from NOW: the chain's last record at start is the cursor, by hash (AUDIT-SEQ-1).
    let mut cursor: Option<String> = match read_chain() {
        Ok(recs) => recs.last().map(|r| r.hash.clone()),
        Err(e) => return fail(e),
    };
    let from = cursor.clone();
    let rule_names: Vec<String> = o.rules.iter().map(Rule::name).collect();
    eprintln!(
        "monitor: watching `{}` from {} — rules: {}",
        delulu_diag::terminal_line(&o.node),
        from.as_deref().map(|h| format!("record {}", &h[..h.len().min(16)])).unwrap_or_else(|| "the chain's start".into()),
        rule_names.join(", ")
    );

    let started = Instant::now();
    let mut windows: HashMap<String, VecDeque<(u64, String, i64)>> = HashMap::new();
    let mut quarantined: HashSet<String> = HashSet::new();
    let mut reports: Vec<serde_json::Value> = Vec::new();
    let mut refused_any = false;
    // How many records the rules judged — so "quarantined nothing" can be told from "read nothing".
    let mut records_read: usize = 0;
    let ended: String;
    loop {
        let nodes = match list() {
            Ok(n) => n,
            Err(e) => {
                refused_any = true;
                ended = e;
                break;
            }
        };
        if let Some(n) = nodes.iter().find(|n| n.id == o.node) {
            if n.state != "live" {
                ended = format!("`{}` is {} — nothing left to watch", o.node, n.state);
                break;
            }
        }
        let runs = runs_under(&o.node, &nodes);
        let recs = match read_chain() {
            Ok(r) => r,
            Err(e) => {
                refused_any = true;
                ended = e;
                break;
            }
        };
        let start = match &cursor {
            None => 0,
            Some(h) => match recs.iter().position(|r| &r.hash == h) {
                Some(i) => i + 1,
                None => {
                    // The record the monitor last read is gone: the chain was rewritten under it. Say
                    // so, and read on from the head — never re-judge records already judged.
                    eprintln!("monitor: WARNING — the record it last read is no longer in the chain; reading on from the head");
                    recs.len()
                }
            },
        };
        // Evidence per run, in chain order; each run fires at most once.
        let mut fire: BTreeMap<String, Evidence> = BTreeMap::new();
        records_read += recs.len() - start;
        for r in &recs[start..] {
            let Some(run) = r.actor_node.as_deref().and_then(|a| runs.get(a)) else { continue };
            if quarantined.contains(run) || fire.contains_key(run) || r.decision != "deny" {
                continue;
            }
            for rule in &o.rules {
                match rule {
                    Rule::Envelope => {
                        let by_envelope = r.action == "use"
                            && r.authority.as_ref().and_then(|a| a.get("refused_by")).and_then(|v| v.as_str())
                                == Some("envelope");
                        if by_envelope {
                            fire.insert(
                                run.clone(),
                                Evidence { rule: rule.name(), records: vec![(r.seq, r.hash.clone(), r.ts)] },
                            );
                        }
                    }
                    Rule::Denies { n, ms } => {
                        let w = windows.entry(run.clone()).or_default();
                        w.push_back((r.seq, r.hash.clone(), r.ts));
                        while w.front().is_some_and(|f| f.2 < r.ts - ms) {
                            w.pop_front();
                        }
                        if w.len() >= *n {
                            fire.insert(run.clone(), Evidence { rule: rule.name(), records: w.iter().cloned().collect() });
                        }
                    }
                }
                if fire.contains_key(run) {
                    break;
                }
            }
        }
        cursor = recs.last().map(|r| r.hash.clone()).or(cursor);
        for (run, ev) in fire {
            quarantined.insert(run.clone());
            let seqs: Vec<String> = ev.records.iter().map(|r| r.0.to_string()).collect();
            let hashes: Vec<String> = ev.records.iter().map(|r| r.1[..r.1.len().min(16)].to_string()).collect();
            let window_ms = ev.records.last().map(|l| l.2).unwrap_or(0) - ev.records.first().map(|f| f.2).unwrap_or(0);
            let why = format!(
                "monitor `{}` quarantined `{run}`: rule `{}` — {} deny record(s) within {window_ms} ms (seq {}; hashes {})",
                o.node,
                ev.rule,
                ev.records.len(),
                seqs.join(", "),
                hashes.join(", ")
            );
            let last_ts = ev.records.last().map(|l| l.2).unwrap_or(0);
            let resp = crate::brokerd::request(
                &state_dir,
                ReqBody::Quarantine { caller: o.node.clone(), target: run.clone(), why: why.clone() },
            );
            let after_ms = now_millis() - last_ts;
            let mut report = json!({
                "target": run, "rule": ev.rule, "count": ev.records.len(), "window_ms": window_ms,
                "evidence": ev.records.iter().map(|r| json!({"seq": r.0, "hash": r.1})).collect::<Vec<_>>(),
                "ms_after_last_evidence": after_ms,
            });
            match resp {
                Ok(Response::Revoked { by_seq, newly_revoked, .. }) => {
                    report["revoked_by_seq"] = json!(by_seq);
                    report["revoked"] = json!(newly_revoked);
                    if !o.json {
                        println!(
                            "quarantine: `{}` — rule {}: {} deny record(s) within {window_ms} ms (seq {}) — revoked at seq {by_seq} ({} node(s)), {after_ms} ms after the last",
                            delulu_diag::terminal_line(&run),
                            ev.rule,
                            ev.records.len(),
                            seqs.join(", "),
                            newly_revoked.len()
                        );
                    }
                }
                Ok(Response::Error { code, message, .. }) => {
                    refused_any = true;
                    report["refused"] = json!({"code": code, "message": message});
                    eprintln!(
                        "monitor: the broker REFUSED the quarantine of `{}`: {code}: {}",
                        delulu_diag::terminal_line(&run),
                        delulu_diag::terminal_line(&message)
                    );
                }
                Ok(other) => {
                    refused_any = true;
                    report["refused"] = json!({"code": "DL1401", "message": format!("unexpected answer {other:?}")});
                }
                Err(e) => {
                    refused_any = true;
                    report["refused"] = json!({"code": "DL1401", "message": e.to_string()});
                    eprintln!("monitor: the quarantine of `{}` did not reach the broker: {e}", delulu_diag::terminal_line(&run));
                }
            }
            reports.push(report);
        }
        if let Some(f) = o.for_ms {
            if started.elapsed() >= Duration::from_millis(f) {
                ended = format!("watched for {f} ms");
                break;
            }
        }
        std::thread::sleep(o.poll);
    }
    eprintln!("monitor: ended — {}", delulu_diag::terminal_line(&ended));
    if o.json {
        crate::cli::print_success_envelope(
            "monitor",
            json!({
                "subcommand": "watch", "node": o.node, "rules": rule_names, "from": from,
                "quarantines": reports, "records_read": records_read, "ended": ended,
            }),
        );
    }
    i32::from(refused_any)
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, parent: Option<&str>) -> crate::broker_ipc::NodeInfo {
        crate::broker_ipc::NodeInfo { id: id.into(), parent: parent.map(str::to_string), state: "live".into(), ..Default::default() }
    }

    /// The step-3 rule as the monitor sees it: every node under `g_M` maps to its run (the child of
    /// `g_M` on its path), and nothing outside the subtree — a sibling, the parent — is there at all.
    #[test]
    fn a_monitor_maps_its_subtree_to_runs_and_sees_nothing_else() {
        let nodes = [
            node("g_root", None),
            node("g_M", Some("g_root")),
            node("g_run1", Some("g_M")),
            node("g_dev1", Some("g_run1")),
            node("g_run2", Some("g_M")),
            node("g_sibling", Some("g_root")),
            node("g_cousin", Some("g_sibling")),
        ];
        let runs = runs_under("g_M", &nodes);
        assert_eq!(runs.get("g_M").map(String::as_str), Some("g_M"));
        assert_eq!(runs.get("g_run1").map(String::as_str), Some("g_run1"));
        assert_eq!(runs.get("g_dev1").map(String::as_str), Some("g_run1"), "a device's node belongs to its run");
        assert_eq!(runs.get("g_run2").map(String::as_str), Some("g_run2"));
        for outside in ["g_root", "g_sibling", "g_cousin"] {
            assert!(!runs.contains_key(outside), "`{outside}` is not under the monitor: {runs:?}");
        }
    }

    #[test]
    fn the_rules_parse_and_refuse_what_they_cannot_mean() {
        assert_eq!(Rule::parse("envelope"), Ok(Rule::Envelope));
        assert_eq!(Rule::parse("denies=3/60000"), Ok(Rule::Denies { n: 3, ms: 60_000 }));
        for bad in ["denies=0/100", "denies=3/0", "denies=3", "denies=x/1", "everything", ""] {
            assert!(Rule::parse(bad).is_err(), "`{bad}` must be refused");
        }
    }
}
