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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceRecord {
    pub seq: u64,
    pub effect: String,
    pub op: String,
    pub cap_kind: String,
    pub detail: Option<String>,
    pub span: Option<(u32, u32, u32)>,
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
#[derive(Clone, Default)]
pub struct TraceSink(Rc<RefCell<Vec<TraceRecord>>>);

impl TraceSink {
    pub fn new() -> TraceSink {
        TraceSink(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn push(&self, record: TraceRecord) {
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
        });
        sink.push(TraceRecord {
            seq: 1,
            effect: "Read".into(),
            op: "read_text".into(),
            cap_kind: "FsRead".into(),
            detail: None,
            span: None,
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
            TraceRecord { seq: 0, effect: "Write".into(), op: "println".into(), cap_kind: "Console".into(), detail: None, span: None },
            TraceRecord { seq: 1, effect: "Net".into(), op: "get".into(), cap_kind: "Http".into(), detail: None, span: None },
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
            TraceRecord { seq: 0, effect: "Write".into(), op: "println".into(), cap_kind: "Console".into(), detail: None, span: None },
            TraceRecord { seq: 1, effect: "Read".into(), op: "read_text".into(), cap_kind: "FsRead".into(), detail: None, span: None },
        ];
        assert!(assert_trace(&allowed, &records).is_empty());
    }
}
