//! Effect tracing (spec §6.1): the executable soundness witness. Every EFFECTFUL primitive
//! operation dispatched by `interp::eval_method` appends one `TraceRecord` to the attached
//! `TraceSink`; pure operations (attenuation, `Secret.verify`/`Secret.map`, `Str`/`List` methods,
//! and `Root` capability-minting) are never traced — see `effect_for` below, the single mapping
//! table that decides which (receiver kind, method) pairs are effectful.
//!
//! Redaction (spec §6.1): any traced operation whose receiver or argument is a `Secret` value
//! records `detail = Some(OPAQUE)` — never the secret's contents, never a value derived from
//! `expose`. That decision is made at the call site in `interp.rs` (it is the only place that
//! sees the actual `Value`s); this module only defines the marker and the record shape.
//!
//! Determinism (spec §6.2) lives in `prim.rs` (`set_rand_seed`, `set_fixed_clock_ms`); tracing
//! and determinism are independent knobs that happen to serve the same debugging/fuzzing loop.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

/// The literal redacted marker for any secret-involving trace field (spec §6.1).
pub const OPAQUE: &str = "\u{ab}opaque\u{bb}";

/// One traced effect operation. Field shape is deliberately flat and hand-JSON-serializable (no
/// new dependency): `seq` orders records within a run (strictly increasing), `span` mirrors
/// `delulu_diag::Span` as `(file, start, end)` byte offsets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TraceRecord {
    pub seq: u64,
    pub effect: String,
    pub op: String,
    pub cap_kind: String,
    pub detail: Option<String>,
    pub span: Option<(u32, u32, u32)>,
    /// Stage 7 (spec §6.3): causal actor attribution — `"Ping#3"`. `None` for main-line
    /// records, and then NONE of the actor fields serialize (a program without actors keeps
    /// byte-identical trace output).
    pub actor: Option<String>,
    /// The executing member (`"Ping.ping"`) — what `--assert-trace` checks the effect against.
    pub member: Option<String>,
    /// The globally monotonic turn id this effect ran in.
    pub turn: Option<u64>,
    /// The causal edge: who sent the message that started this turn, from where.
    pub cause: Option<Cause>,
}

/// The cause chain of a turn (spec §6.3): the sender's identity, the sender's executing
/// member (`None` = the main line), and the send site's span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cause {
    pub sender: String,
    pub sender_member: Option<String>,
    pub send_span: Option<(u32, u32, u32)>,
}

impl TraceRecord {
    /// Render as one JSON object (no trailing newline, no dependency on `serde_json`).
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(96);
        out.push('{');
        out.push_str(&format!("\"seq\":{}", self.seq));
        out.push_str(",\"effect\":");
        out.push_str(&json_string(&self.effect));
        out.push_str(",\"op\":");
        out.push_str(&json_string(&self.op));
        out.push_str(",\"cap_kind\":");
        out.push_str(&json_string(&self.cap_kind));
        out.push_str(",\"detail\":");
        match &self.detail {
            Some(d) => out.push_str(&json_string(d)),
            None => out.push_str("null"),
        }
        out.push_str(",\"span\":");
        match self.span {
            Some((file, start, end)) => {
                out.push_str(&format!("{{\"file\":{file},\"start\":{start},\"end\":{end}}}"));
            }
            None => out.push_str("null"),
        }
        // Actor attribution (spec §6.3) serializes ONLY when present — non-actor records
        // stay byte-identical to v0.6.
        if let Some(a) = &self.actor {
            out.push_str(",\"actor\":");
            out.push_str(&json_string(a));
            if let Some(m) = &self.member {
                out.push_str(",\"member\":");
                out.push_str(&json_string(m));
            }
            if let Some(t) = self.turn {
                out.push_str(&format!(",\"turn\":{t}"));
            }
            if let Some(c) = &self.cause {
                out.push_str(",\"cause\":{\"sender\":");
                out.push_str(&json_string(&c.sender));
                out.push_str(",\"sender_member\":");
                match &c.sender_member {
                    Some(m) => out.push_str(&json_string(m)),
                    None => out.push_str("null"),
                }
                out.push_str(",\"send_span\":");
                match c.send_span {
                    Some((file, start, end)) => {
                        out.push_str(&format!("{{\"file\":{file},\"start\":{start},\"end\":{end}}}"));
                    }
                    None => out.push_str("null"),
                }
                out.push('}');
            }
        }
        out.push('}');
        out
    }
}

/// Escape a Rust string into a JSON string literal (quotes included).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A shared, append-only trace buffer. Cheap to clone (an `Rc`): callers keep one handle to read
/// after the run and hand a clone to `Interp::with_trace`.
///
/// # Why this can be bounded, and when it must not be (C56, ruling D49)
///
/// The buffer held every record until the process exited, so memory grew with the number of effects
/// PERFORMED rather than with the program's live data: 100,000 console writes took peak working set
/// from 6.7 MB to 70.1 MB, about 633 bytes retained per effect and no ceiling. At a thousand effects
/// a second — an ordinary rate for the control loops Stage 10 exists to serve — that is roughly
/// 2.3 GB per hour, and the runs that enable `--trace-effects` are exactly the long-lived ones being
/// diagnosed in the field.
///
/// **The cap is only ever applied when the trace is DIAGNOSTIC output.** `--assert-trace` consumes
/// the same records to prove that no effect outside the declared set occurred, and dropping records
/// there would let a violation go unseen — a fail-OPEN on a security-adjacent check, which is worse
/// than the memory it would save. So the bound is chosen by the caller, and the assertion path asks
/// for an unbounded sink.
///
/// Records are kept from the FRONT, and the count withheld is reported — the same shape D38 gave the
/// diagnostic flood, and for the same reason: a deterministic prefix plus an honest number is
/// reproducible evidence, where a ring buffer would silently make two runs of the same program
/// disagree about what happened.
#[derive(Clone, Default)]
pub struct TraceSink(Rc<RefCell<Vec<TraceRecord>>>, Rc<Cap>);

/// The retention policy for one sink: how many records to keep, and how many were dropped.
#[derive(Default)]
pub struct Cap {
    /// `None` = unbounded (the assertion path).
    limit: Option<usize>,
    dropped: std::cell::Cell<u64>,
}

impl TraceSink {
    /// An UNBOUNDED sink. Correct for `--assert-trace`, which needs every record to be sound.
    pub fn new() -> TraceSink {
        TraceSink(Rc::new(RefCell::new(Vec::new())), Rc::new(Cap::default()))
    }

    /// A sink that retains at most `limit` records and counts the rest. For diagnostic tracing only.
    pub fn bounded(limit: usize) -> TraceSink {
        TraceSink(
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(Cap { limit: Some(limit), dropped: std::cell::Cell::new(0) }),
        )
    }

    /// How many records were dropped by the cap. `0` for an unbounded sink.
    pub fn dropped(&self) -> u64 {
        self.1.dropped.get()
    }

    pub fn push(&self, record: TraceRecord) {
        if let Some(limit) = self.1.limit {
            if self.0.borrow().len() >= limit {
                self.1.dropped.set(self.1.dropped.get() + 1);
                return;
            }
        }
        self.0.borrow_mut().push(record);
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    /// A snapshot copy of the records recorded so far, in order.
    pub fn records(&self) -> Vec<TraceRecord> {
        self.0.borrow().clone()
    }

    /// Drain and return all records (Stage 7: the per-worker sink empties after each turn so
    /// the records can be stamped with actor/turn/cause and shipped to the shared collector).
    pub fn take_records(&self) -> Vec<TraceRecord> {
        std::mem::take(&mut *self.0.borrow_mut())
    }

    /// Append a record verbatim (Stage 7: the CLI merges worker-collected records into the
    /// main sink before emitting/asserting).
    pub fn append(&self, r: TraceRecord) {
        self.0.borrow_mut().push(r);
    }

    /// One JSON object per line, newline-separated, no trailing newline (spec §6.1).
    pub fn to_json_lines(&self) -> String {
        self.0.borrow().iter().map(TraceRecord::to_json).collect::<Vec<_>>().join("\n")
    }
}

/// The one place that says which (receiver capability kind, method) pairs are effectful, and
/// which `Effect` label they carry. `cap_kind` is `ResourceKind::name()` for capability
/// receivers (`"Console"`, `"FsRead"`, `"FsWrite"`, `"Http"`, `"Clock"`, `"Rand"`) or the literal
/// `"Secret"` for `Secret` receivers. Every other (kind, method) pair — attenuation (`narrow`),
/// `Secret.verify`/`Secret.map`, `Str`/`List` methods, and `Root` capability-minting methods — is
/// pure and returns `None`, so `interp::eval_method` never traces it. Keep this table in
/// lockstep with the effect meanings documented atop `prim.rs`.
pub fn effect_for(cap_kind: &str, method: &str) -> Option<&'static str> {
    match (cap_kind, method) {
        ("Console", "println") | ("Console", "print") => Some("Write"),
        ("Console", "readline") => Some("Read"),
        ("FsRead", "read_text") | ("FsRead", "list_dir") => Some("Read"),
        ("FsWrite", "write_text") | ("FsWrite", "append_text") => Some("Write"),
        ("Http", "get") => Some("Net"),
        ("Clock", "now_ms") => Some("Clock"),
        ("Rand", "int") | ("Rand", "float") => Some("Rand"),
        ("Secret", "expose") => Some("Declassify"),
        // Stage 10 (10e): the physical boundary. A command is `Actuate`; a sensor read is plain
        // `Read` (observation is observation — no new effect for it, spec §5.1).
        ("Actuator", "command") => Some("Actuate"),
        ("Sensor", "read") => Some("Read"),
        _ => None,
    }
}

/// The trace ⊆ row law (spec §6.2 / invariant 12), executable: every record's effect must be in
/// the statically computed row passed by the caller (typically the checker's `main_row`).
/// Returns one violation message per offending record; empty means the trace is a sound witness
/// of the row. The CLI attaches DL1101 to a non-empty result (§9); this function only computes
/// the violations.
pub fn assert_trace(allowed_effects: &BTreeSet<String>, records: &[TraceRecord]) -> Vec<String> {
    records
        .iter()
        .filter(|r| !allowed_effects.contains(&r.effect))
        .map(|r| {
            format!(
                "seq {}: effect `{}` (op `{}.{}`) is not in the statically computed row",
                r.seq, r.effect, r.cap_kind, r.op
            )
        })
        .collect()
}

/// The CAUSAL trace ⊆ row law (Stage 7, invariant 35 executable): an actor-attributed record's
/// effect must be in (a) the EXECUTING member's static row and (b) the SEND SITE's static row
/// — the sender's member row, or `main_row` for a main-line send. Main-line records keep the
/// original law against `main_row`. `member_rows` is keyed `"Actor.member"`.
pub fn assert_trace_causal(
    main_row: &BTreeSet<String>,
    member_rows: &std::collections::HashMap<String, BTreeSet<String>>,
    records: &[TraceRecord],
) -> Vec<String> {
    let mut out = Vec::new();
    let row_of = |member: Option<&String>| -> Option<&BTreeSet<String>> {
        match member {
            Some(m) => member_rows.get(m),
            None => Some(main_row),
        }
    };
    for r in records {
        match &r.actor {
            None => {
                if !main_row.contains(&r.effect) {
                    out.push(format!(
                        "seq {}: effect `{}` (op `{}.{}`) is not in the statically computed row",
                        r.seq, r.effect, r.cap_kind, r.op
                    ));
                }
            }
            Some(actor) => {
                // (a) the executing member's row contains the effect.
                match row_of(r.member.as_ref()) {
                    Some(row) if row.contains(&r.effect) => {}
                    Some(_) => out.push(format!(
                        "seq {}: actor {actor}: effect `{}` is not in executing member {}'s static row",
                        r.seq,
                        r.effect,
                        r.member.as_deref().unwrap_or("?")
                    )),
                    None => out.push(format!(
                        "seq {}: actor {actor}: executing member {} has no known static row",
                        r.seq,
                        r.member.as_deref().unwrap_or("?")
                    )),
                }
                // (b) the send site's row contains it too (T-Send: {Async} ∪ row(beh) flows
                // into the sender — this is that containment, replayed on the witness).
                if let Some(c) = &r.cause {
                    match row_of(c.sender_member.as_ref()) {
                        Some(row) if row.contains(&r.effect) => {}
                        Some(_) => out.push(format!(
                            "seq {}: actor {actor}: effect `{}` is not in the SEND SITE's static row (sender {}, member {})",
                            r.seq,
                            r.effect,
                            c.sender,
                            c.sender_member.as_deref().unwrap_or("main line")
                        )),
                        None => out.push(format!(
                            "seq {}: actor {actor}: sender member {} has no known static row",
                            r.seq,
                            c.sender_member.as_deref().unwrap_or("main line")
                        )),
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_for_maps_every_effectful_primitive() {
        assert_eq!(effect_for("Console", "println"), Some("Write"));
        assert_eq!(effect_for("Console", "print"), Some("Write"));
        assert_eq!(effect_for("Console", "readline"), Some("Read"));
        assert_eq!(effect_for("FsRead", "read_text"), Some("Read"));
        assert_eq!(effect_for("FsRead", "list_dir"), Some("Read"));
        assert_eq!(effect_for("FsWrite", "write_text"), Some("Write"));
        assert_eq!(effect_for("FsWrite", "append_text"), Some("Write"));
        assert_eq!(effect_for("Http", "get"), Some("Net"));
        assert_eq!(effect_for("Clock", "now_ms"), Some("Clock"));
        assert_eq!(effect_for("Rand", "int"), Some("Rand"));
        assert_eq!(effect_for("Rand", "float"), Some("Rand"));
        assert_eq!(effect_for("Secret", "expose"), Some("Declassify"));
        assert_eq!(effect_for("Actuator", "command"), Some("Actuate"));
        assert_eq!(effect_for("Sensor", "read"), Some("Read"));
    }

    #[test]
    fn effect_for_is_none_for_pure_operations() {
        // Attenuation, verification, and pure Str/List/Secret methods carry no effect.
        assert_eq!(effect_for("FsRead", "narrow"), None);
        assert_eq!(effect_for("Secret", "verify"), None);
        assert_eq!(effect_for("Secret", "map"), None);
        assert_eq!(effect_for("Str", "len"), None);
        assert_eq!(effect_for("List", "push"), None);
        assert_eq!(effect_for("Console", "unknown_method"), None);
    }

    #[test]
    fn to_json_escapes_and_shapes_correctly() {
        let r = TraceRecord {
            seq: 7,
            effect: "Write".into(),
            op: "println".into(),
            cap_kind: "Console".into(),
            detail: Some("a \"quoted\" line\nwith a newline".into()),
            span: Some((3, 10, 20)),
            ..Default::default()
        };
        let json = r.to_json();
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("\"seq\":7"));
        assert!(json.contains("\"effect\":\"Write\""));
        assert!(json.contains("\"op\":\"println\""));
        assert!(json.contains("\"cap_kind\":\"Console\""));
        assert!(json.contains("\\\"quoted\\\""));
        assert!(json.contains("\\n"));
        assert!(json.contains("\"span\":{\"file\":3,\"start\":10,\"end\":20}"));
    }

    #[test]
    fn to_json_renders_null_detail_and_span() {
        let r = TraceRecord {
            seq: 0,
            effect: "Rand".into(),
            op: "int".into(),
            cap_kind: "Rand".into(),
            detail: None,
            span: None,
            ..Default::default()
        };
        let json = r.to_json();
        assert!(json.contains("\"detail\":null"));
        assert!(json.contains("\"span\":null"));
    }

    #[test]
    fn sink_push_and_to_json_lines_preserve_order() {
        let sink = TraceSink::new();
        sink.push(TraceRecord {
            seq: 0,
            effect: "Write".into(),
            op: "println".into(),
            cap_kind: "Console".into(),
            detail: None,
            span: None,
            ..Default::default()
        });
        sink.push(TraceRecord {
            seq: 1,
            effect: "Read".into(),
            op: "read_text".into(),
            cap_kind: "FsRead".into(),
            detail: None,
            span: None,
            ..Default::default()
        });
        assert_eq!(sink.len(), 2);
        let joined = sink.to_json_lines();
        let lines: Vec<&str> = joined.split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"seq\":0"));
        assert!(lines[1].contains("\"seq\":1"));
        let recs = sink.records();
        assert_eq!(recs[0].effect, "Write");
        assert_eq!(recs[1].effect, "Read");
    }

    #[test]
    fn assert_trace_flags_effects_outside_the_allowed_row() {
        let allowed: BTreeSet<String> = ["Write"].into_iter().map(String::from).collect();
        let records = vec![
            TraceRecord { seq: 0, effect: "Write".into(), op: "println".into(), cap_kind: "Console".into(), detail: None, span: None, ..Default::default() },
            TraceRecord { seq: 1, effect: "Net".into(), op: "get".into(), cap_kind: "Http".into(), detail: None, span: None, ..Default::default() },
        ];
        let violations = assert_trace(&allowed, &records);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Net"));
        assert!(violations[0].contains("seq 1"));
    }

    #[test]
    fn assert_trace_is_empty_when_every_effect_is_allowed() {
        let allowed: BTreeSet<String> = ["Write", "Read"].into_iter().map(String::from).collect();
        let records = vec![
            TraceRecord { seq: 0, effect: "Write".into(), op: "println".into(), cap_kind: "Console".into(), detail: None, span: None, ..Default::default() },
            TraceRecord { seq: 1, effect: "Read".into(), op: "read_text".into(), cap_kind: "FsRead".into(), detail: None, span: None, ..Default::default() },
        ];
        assert!(assert_trace(&allowed, &records).is_empty());
    }
}

#[cfg(test)]
mod cap_tests {
    use super::*;

    fn rec(i: usize) -> TraceRecord {
        TraceRecord {
            seq: i as u64,
            effect: "Write".to_string(),
            op: "println".to_string(),
            cap_kind: "Console".to_string(),
            detail: Some(format!("{i}")),
            ..Default::default()
        }
    }

    /// **A diagnostic trace is bounded; an assertion trace is not.** The buffer used to keep every
    /// record until exit, so memory grew with the number of effects PERFORMED — 100k console writes
    /// took peak working set from 6.7 MB to 70.1 MB with no ceiling (C56).
    ///
    /// The asymmetry is the point and must not be "simplified" away later: `--assert-trace` proves
    /// that no effect outside the declared set occurred, so a dropped record could hide a violation.
    /// Capping that path would be a fail-OPEN on a security-adjacent check — strictly worse than the
    /// memory it would save.
    #[test]
    fn a_bounded_sink_stops_at_its_limit_and_counts_the_rest() {
        let s = TraceSink::bounded(10);
        for i in 0..25 {
            s.push(rec(i));
        }
        assert_eq!(s.len(), 10, "the cap holds");
        assert_eq!(s.dropped(), 15, "and the number withheld is reported, not swallowed");
        // Kept from the FRONT, deterministically: two runs of one program must agree about what
        // happened. A ring buffer would make them disagree.
        let kept = s.records();
        assert_eq!(kept.first().unwrap().detail.as_deref(), Some("0"));
        assert_eq!(kept.last().unwrap().detail.as_deref(), Some("9"));
    }

    #[test]
    fn an_unbounded_sink_keeps_everything_because_assert_trace_must_be_sound() {
        let s = TraceSink::new();
        for i in 0..1000 {
            s.push(rec(i));
        }
        assert_eq!(s.len(), 1000, "the assertion path is never capped");
        assert_eq!(s.dropped(), 0);
    }
}
