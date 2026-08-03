//! The machine-facing JSON envelope (spec §10.1–§10.2). Field names and shapes are
//! stable API: additions are allowed, renames and removals are not.

use serde_json::{json, Value};

use crate::diagnostic::Diagnostic;
use crate::source::SourceMap;
use crate::span::Span;

fn position_json(map: &SourceMap, span: Span) -> (Value, Value) {
    let (sl, sc) = map.position(span.file, span.start);
    let (el, ec) = map.position(span.file, span.end);
    (
        json!({ "line": sl, "col": sc, "byte": span.start }),
        json!({ "line": el, "col": ec, "byte": span.end }),
    )
}

fn diagnostic_json(map: &SourceMap, d: &Diagnostic) -> Value {
    let spans: Vec<Value> = d
        .spans
        .iter()
        // A span naming a file this map never loaded cannot be positioned, and indexing for it
        // panicked (C83). Omit the span; `code`, `severity` and `message` — the fields a machine
        // actually keys on — are unaffected, so the envelope stays useful and stays valid.
        .filter(|ls| map.has(ls.span.file))
        .map(|ls| {
            let (start, end) = position_json(map, ls.span);
            let mut o = json!({
                "file": map.name(ls.span.file),
                "start": start,
                "end": end,
            });
            if let Some(label) = &ls.label {
                o["label"] = json!(label);
            }
            if ls.secondary {
                o["secondary"] = json!(true);
            }
            o
        })
        .collect();

    let repairs: Vec<Value> = d
        .repairs
        .iter()
        // All-or-nothing, unlike spans above: dropping ONE edit would emit a repair that applies
        // only part of itself, and `delulu fix` would then write a half-repair into the user's
        // source. An omitted repair is safe; a partial one is not.
        .filter(|r| r.edits.iter().all(|e| map.has(e.file)))
        .map(|r| {
            let edits: Vec<Value> = r
                .edits
                .iter()
                .map(|e| {
                    json!({
                        "file": map.name(e.file),
                        "range": { "start_byte": e.start_byte, "end_byte": e.end_byte },
                        "insert": e.insert,
                    })
                })
                .collect();
            json!({
                "id": r.id,
                "confidence": r.confidence.as_str(),
                "authority_widening": r.authority_widening,
                "requires_human": r.requires_human,
                "edits": edits,
            })
        })
        .collect();

    json!({
        "code": d.code,
        "severity": d.severity.as_str(),
        "message": d.message,
        "spans": spans,
        "explanation_id": d.explanation_id(),
        "repairs": repairs,
    })
}

/// Build the full envelope. `authority` is `Some` for `delulu authority`
/// (and optionally for `check`); `None` omits the field entirely.
pub fn envelope(
    command: &str,
    diagnostics: &[Diagnostic],
    authority: Option<Value>,
    map: &SourceMap,
) -> Value {
    let errors = diagnostics.iter().filter(|d| d.is_error()).count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == crate::Severity::Warning)
        .count();
    let mut env = json!({
        "delulu_version": env!("CARGO_PKG_VERSION"),
        "schema": 1,
        "command": command,
        "diagnostics": diagnostics.iter().map(|d| diagnostic_json(map, d)).collect::<Vec<_>>(),
        "summary": { "errors": errors, "warnings": warnings },
    });
    if let Some(a) = authority {
        env["authority"] = a;
    }
    env
}

pub fn envelope_to_string(
    command: &str,
    diagnostics: &[Diagnostic],
    authority: Option<Value>,
    map: &SourceMap,
) -> String {
    serde_json::to_string_pretty(&envelope(command, diagnostics, authority, map))
        .expect("envelope serialization cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Confidence, Diagnostic, Edit, Repair};

    /// Campaign finding C83, machine half: the envelope must stay valid when a span names a file
    /// this map never loaded, and a repair that cannot be fully located must not be offered at all.
    #[test]
    fn a_span_or_repair_naming_an_unloaded_file_is_omitted_not_panicked_on() {
        let mut map = SourceMap::new();
        let f = map.add_file("src/main.delulu", "fn x() {}\n");
        let d = Diagnostic::error("DL0501", "about a file this map never loaded")
            .with_span(Span::new(f, 3, 4), "this one is locatable")
            .with_span(Span::new(9, 0, 1), "this one is not")
            .with_repair(Repair {
                id: "half_locatable",
                confidence: Confidence::Exact,
                authority_widening: false,
                requires_human: false,
                // One edit IS locatable and one is not. Emitting the locatable half would let
                // `delulu fix` write a repair that only partly applied.
                edits: vec![
                    Edit { file: f, start_byte: 0, end_byte: 0, insert: "a".into() },
                    Edit { file: 9, start_byte: 0, end_byte: 0, insert: "b".into() },
                ],
            });
        let env = envelope("check", &[d], None, &map);
        let diag = &env["diagnostics"][0];
        assert_eq!(diag["code"], "DL0501");
        assert_eq!(diag["spans"].as_array().unwrap().len(), 1, "only the locatable span survives");
        assert_eq!(diag["spans"][0]["label"], "this one is locatable");
        assert_eq!(
            diag["repairs"].as_array().unwrap().len(),
            0,
            "a repair that cannot be fully located must be dropped whole, never partially"
        );
    }

    #[test]
    fn envelope_shape_is_stable() {
        let mut map = SourceMap::new();
        let f = map.add_file("src/main.delulu", "fn x() {}\n");
        let d = Diagnostic::error("DL0501", "function `x` performs effect `Net` not declared in its row")
            .with_span(Span::new(f, 3, 4), "this call performs `Net`")
            .with_repair(Repair {
                id: "add_effect_to_row",
                confidence: Confidence::Exact,
                authority_widening: true,
                requires_human: false,
                edits: vec![Edit { file: f, start_byte: 6, end_byte: 6, insert: " ! {Net}".into() }],
            });
        let env = envelope("check", &[d], None, &map);
        assert_eq!(env["schema"], 1);
        assert_eq!(env["command"], "check");
        assert_eq!(env["summary"]["errors"], 1);
        let diag = &env["diagnostics"][0];
        assert_eq!(diag["code"], "DL0501");
        assert_eq!(diag["explanation_id"], "E-DL0501");
        assert_eq!(diag["spans"][0]["start"]["byte"], 3);
        assert_eq!(diag["spans"][0]["start"]["line"], 1);
        assert_eq!(diag["repairs"][0]["authority_widening"], true);
        assert_eq!(diag["repairs"][0]["confidence"], "exact");
        // The envelope must round-trip as JSON (acceptance criterion 9's seed).
        let s = envelope_to_string("check", &[], None, &map);
        let _: Value = serde_json::from_str(&s).unwrap();
    }
}
