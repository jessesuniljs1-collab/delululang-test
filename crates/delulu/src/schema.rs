//! P4-09: `delulu schema` — the JSON the toolchain emits, described as JSON Schema, and checked.
//!
//! An agent that has never seen DeluluLang reads the envelope, a diagnostic, a repair, the authority
//! report, the Atlas, the sandbox object and the run report as DATA it can hold a program's output
//! against — not as prose in `docs/for-agents.md` it has to believe.
//!
//! **The schemas are closed.** Every object this module describes lists every field its emitter
//! writes, and says `additionalProperties: false`. So the tests that run each emitter over a corpus
//! and validate the real output (`tests/schema_cli.rs`) fail the moment an emitter gains a field the
//! schema does not name, or loses one it requires: the description cannot drift from the thing it
//! describes, which is the property a hand-written schema never has. Two objects are deliberately
//! open, and say so in their own `description`: a command's payload beside the envelope's fixed
//! fields, and the Atlas's embedded custody view (the broker's grant listing).
//!
//! **The validator is in the binary** (`delulu schema validate NAME FILE`), a small subset of JSON
//! Schema 2020-12 — `type`, `properties`, `required`, `additionalProperties`, `items`, `enum`,
//! `const`, `anyOf`, `maxItems`, `$ref` into `$defs` — so an agent can check its own reading of an
//! output with the same code the tests use, and no dependency is added to do it.

use serde_json::{json, Map, Value};

/// The documents `delulu schema` knows, with what each describes.
pub const NAMES: &[(&str, &str)] = &[
    ("envelope", "the object every `--json` command prints: command, schema, version, diagnostics, summary"),
    ("diagnostic", "one diagnostic: code, severity, message, spans, explanation id, typed repairs"),
    ("repair", "one typed repair: its edits, confidence, and whether it widens authority or needs a human"),
    ("authority", "the authority report `delulu authority --json` carries under `authority`"),
    ("atlas", "the `atlas/1` graph `delulu atlas --json` carries under `atlas`"),
    ("sandbox", "the `sandbox` object of a run report: in-process, previewed, or a sandboxed run"),
    ("policy", "what `delulu sandbox policy --json` carries under `policy`"),
    ("run-report", "the report `delulu run --report-out F` writes"),
    ("toolchain", "what `delulu toolchain --json` carries under `toolchain`"),
];

fn t(ty: &str) -> Value {
    json!({ "type": ty })
}
fn nullable(ty: &str) -> Value {
    json!({ "type": [ty, "null"] })
}
fn arr(items: Value) -> Value {
    json!({ "type": "array", "items": items })
}
fn r(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}
fn en(values: &[&str]) -> Value {
    json!({ "type": "string", "enum": values })
}
/// A closed object: every `required` field must be present, every `optional` one may be, and
/// nothing else may.
fn obj(required: &[(&str, Value)], optional: &[(&str, Value)]) -> Value {
    let mut props = Map::new();
    for (k, v) in required.iter().chain(optional) {
        props.insert(k.to_string(), v.clone());
    }
    json!({
        "type": "object",
        "properties": props,
        "required": required.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
        "additionalProperties": false,
    })
}
fn described(mut v: Value, why: &str) -> Value {
    v["description"] = json!(why);
    v
}

/// Every shared definition. Each document carries all of them, so any `$ref` in it resolves.
fn defs() -> Value {
    let str_list = arr(t("string"));
    let used_at = arr(obj(&[("file", t("string")), ("line", t("integer"))], &[]));
    let node_kinds =
        ["package", "module", "function", "type", "effect", "resource", "foreign", "grant"];
    let edge_kinds = [
        "contains", "depends_on", "imports", "calls", "uses_type", "performs", "requires", "foreign",
        "declassifies", "delegates",
    ];
    let limits = obj(&[("memory_bytes", t("integer")), ("cpu_seconds", t("integer"))], &[]);
    json!({
        "position": obj(&[("line", t("integer")), ("col", t("integer")), ("byte", t("integer"))], &[]),
        "span": obj(
            &[("file", t("string")), ("start", r("position")), ("end", r("position"))],
            &[("label", t("string")), ("secondary", t("boolean"))],
        ),
        "edit": obj(
            &[
                ("file", t("string")),
                ("range", obj(&[("start_byte", t("integer")), ("end_byte", t("integer"))], &[])),
                ("insert", t("string")),
            ],
            &[],
        ),
        "repair": obj(
            &[
                ("id", t("string")),
                ("confidence", en(&["exact", "safe", "suggest"])),
                ("authority_widening", t("boolean")),
                ("requires_human", t("boolean")),
                ("edits", arr(r("edit"))),
                ("reason", nullable("string")),
            ],
            &[],
        ),
        "diagnostic": obj(
            &[
                ("code", t("string")),
                ("severity", en(&["error", "warning", "note"])),
                ("message", t("string")),
                ("spans", arr(r("span"))),
                ("explanation_id", t("string")),
                ("repairs", arr(r("repair"))),
            ],
            &[],
        ),
        "summary": obj(&[("errors", t("integer")), ("warnings", t("integer"))], &[]),
        "capability": obj(
            &[("kind", t("string")), ("scopes", str_list.clone())],
            &[("requested_scopes", str_list.clone())],
        ),
        "foreign_call": json!({ "anyOf": [
            obj(&[
                ("abi", t("string")),
                ("lib", t("string")),
                ("symbols", str_list.clone()),
                ("granted_path", t("null")),
                ("used_at", used_at.clone()),
            ], &[]),
            obj(&[
                ("abi", json!({ "const": "python" })),
                ("allowlist", str_list.clone()),
                ("imports_seen", str_list.clone()),
                ("used_at", used_at.clone()),
            ], &[]),
            obj(&[
                ("abi", json!({ "const": "compute" })),
                ("devices", str_list.clone()),
                ("kernels", str_list.clone()),
                ("used_at", used_at.clone()),
            ], &[]),
        ]}),
        "plugin_load": obj(
            &[
                ("path", nullable("string")),
                ("grant", obj(&[("effects", str_list.clone())], &[])),
                ("loaded_at", used_at.clone()),
                ("name", json!({ "type": ["string", "null"] })),
                ("class", nullable("string")),
                ("signed_by", nullable("string")),
            ],
            &[],
        ),
        "authority": obj(
            &[
                ("program", t("string")),
                ("effects", str_list.clone()),
                ("capabilities", arr(r("capability"))),
                ("secrets", str_list.clone()),
                ("foreign_calls", arr(r("foreign_call"))),
                ("contained_plugins", described(
                    json!({ "type": "array", "maxItems": 0 }),
                    "reserved for plugins held in their own guest; always empty in this build",
                )),
                ("pure_functions", str_list.clone()),
            ],
            &[
                ("requested_scopes", json!({ "type": "object", "additionalProperties": str_list.clone() })),
                ("required_grants", str_list.clone()),
                ("custody", t("string")),
                ("foreign_isolation", t("string")),
                ("isolation", t("string")),
                ("native_emission", obj(&[("requested", t("boolean")), ("via", t("string"))], &[])),
                ("plugins", arr(r("plugin_load"))),
            ],
        ),
        "atlas_node": obj(
            &[("id", t("string")), ("kind", en(&node_kinds)), ("name", t("string"))],
            &[
                ("module", t("string")),
                ("package", t("string")),
                ("span", obj(&[("file", t("string")), ("line", t("integer"))], &[])),
                ("effects", str_list.clone()),
                ("pure", t("boolean")),
                ("resource_class", t("string")),
                ("pattern", t("string")),
            ],
        ),
        "atlas_edge": obj(&[("from", t("string")), ("to", t("string")), ("kind", en(&edge_kinds))], &[]),
        "atlas_god": obj(
            &[("id", t("string")), ("name", t("string")), ("kind", en(&node_kinds)), ("degree", t("integer"))],
            &[],
        ),
        "atlas": obj(
            &[
                ("atlas", json!({ "const": "atlas/1" })),
                ("root", t("string")),
                ("nodes", arr(r("atlas_node"))),
                ("edges", arr(r("atlas_edge"))),
                ("gods", arr(r("atlas_god"))),
                ("authority", r("authority")),
                ("custody", described(
                    json!({ "type": ["object", "null"] }),
                    "open: the broker's grant listing when `--custody` asked for it, else null",
                )),
                ("caveats", str_list.clone()),
            ],
            &[],
        ),
        "limits": limits.clone(),
        "budget": obj(
            &[
                ("memory_bytes", t("integer")),
                ("cpu_seconds", t("integer")),
                ("wall_seconds", nullable("integer")),
                ("enforced_by", t("string")),
            ],
            &[],
        ),
        "posture": obj(
            &[
                ("filesystem_writes", t("string")),
                ("filesystem_reads", t("string")),
                ("network", t("string")),
                ("new_programs", t("string")),
                ("memory", t("string")),
                ("processor_time", t("string")),
                ("privilege_escalation", t("string")),
                ("identity", t("string")),
            ],
            &[],
        ),
        "break_glass_ticket": obj(
            &[
                ("ticket", t("string")),
                ("relax", t("string")),
                ("program_blake3", nullable("string")),
                ("key", t("string")),
                ("reason", t("string")),
                ("not_after", t("string")),
            ],
            &[],
        ),
        "sandbox_inproc": described(obj(
            &[
                ("backend", json!({ "const": "inproc" })),
                ("level", t("integer")),
                ("requested", t("string")),
                ("granted", t("string")),
                ("host_guarantees", str_list.clone()),
                ("limits", r("budget")),
                ("mode", t("string")),
                ("break_glass", t("boolean")),
            ],
            &[("break_glass_ticket", r("break_glass_ticket"))],
        ), "an ordinary run: level 0, the budgets it was held to"),
        "sandbox_preview": described(obj(
            &[
                ("backend", t("string")),
                ("requested_level", t("integer")),
                ("level", t("integer")),
                ("requested", t("string")),
                ("granted", t("string")),
                ("host_guarantees", str_list.clone()),
                ("limits", limits.clone()),
                ("mode", t("string")),
                ("break_glass", t("boolean")),
                ("policy_hash", t("string")),
            ],
            &[("unsupported_surface", nullable("string"))],
        ), "a policy that WOULD hold (`sandbox policy`, `--mode audit`): nothing ran, so no posture"),
        "sandbox_run": described(obj(
            &[
                ("backend", t("string")),
                ("requested_level", t("integer")),
                ("level", t("integer")),
                ("fully_enforced", t("boolean")),
                ("requested", t("string")),
                ("granted", t("string")),
                ("host_guarantees", str_list.clone()),
                ("posture", r("posture")),
                ("limitations", str_list.clone()),
                ("limits", limits),
                ("mode", t("string")),
                ("break_glass", t("boolean")),
                ("denied", str_list.clone()),
                ("denied_total", t("integer")),
                ("policy_hash", t("string")),
            ],
            &[],
        ), "a sandboxed run: what the host applied, what it answers, and what it refused"),
        "sandbox": json!({ "anyOf": [r("sandbox_inproc"), r("sandbox_preview"), r("sandbox_run")] }),
        "egress_hop": obj(
            &[
                ("url", t("string")),
                ("host", t("string")),
                ("port", t("integer")),
                ("addrs", str_list.clone()),
                ("peer", nullable("string")),
                ("status", nullable("integer")),
            ],
            &[],
        ),
        "egress_record": obj(
            &[("url", t("string")), ("delivered", t("boolean")), ("hops", arr(r("egress_hop")))],
            &[("status", t("integer")), ("bytes", t("integer")), ("reason", t("string")), ("explain", t("string"))],
        ),
        "egress": obj(
            &[
                ("requests", t("integer")),
                ("not_delivered", t("integer")),
                ("records", arr(r("egress_record"))),
                ("records_kept", t("integer")),
            ],
            &[],
        ),
        "stopped_by": obj(
            &[("dimension", t("string"))],
            &[
                ("budget_bytes", t("integer")),
                ("observed_bytes", t("integer")),
                ("budget_seconds", t("number")),
                ("observed_seconds", t("number")),
            ],
        ),
    })
}

/// The envelope's five fixed fields, as `(name, schema)` pairs, for the documents that extend it.
fn envelope_fields() -> Vec<(&'static str, Value)> {
    vec![
        ("command", t("string")),
        ("schema", json!({ "const": 1 })),
        ("delulu_version", t("string")),
        ("diagnostics", arr(r("diagnostic"))),
        ("summary", r("summary")),
    ]
}

fn toolchain_payload() -> Value {
    let str_list = arr(t("string"));
    obj(
        &[
            ("version", t("string")),
            (
                "commands",
                arr(obj(
                    &[("name", t("string")), ("json", t("boolean")), ("flags", str_list.clone()), ("usage", str_list.clone())],
                    &[],
                )),
            ),
            ("effects", str_list.clone()),
            (
                "grants",
                arr(obj(
                    &[("key", t("string")), ("form", t("string")), ("example", t("string")), ("grants", t("string"))],
                    &[],
                )),
            ),
            (
                "primitives",
                obj(
                    &[
                        ("table_version", t("integer")),
                        (
                            "entries",
                            arr(obj(&[("receiver", t("string")), ("method", t("string")), ("arity", t("integer"))], &[])),
                        ),
                    ],
                    &[],
                ),
            ),
            (
                "budgets",
                obj(
                    &[
                        ("run_default", r("limits")),
                        (
                            "sandbox_profiles",
                            arr(obj(
                                &[("name", t("string")), ("memory_bytes", t("integer")), ("cpu_seconds", t("integer"))],
                                &[],
                            )),
                        ),
                    ],
                    &[],
                ),
            ),
            ("engines", str_list.clone()),
            (
                "sandbox",
                obj(
                    &[
                        ("levels", arr(obj(&[("level", t("integer")), ("name", t("string"))], &[]))),
                        ("modes", str_list.clone()),
                        ("channel", t("string")),
                    ],
                    &[],
                ),
            ),
            (
                "diagnostics",
                obj(
                    &[
                        ("codes", arr(obj(&[("code", t("string")), ("title", t("string"))], &[]))),
                        ("topics", str_list),
                    ],
                    &[],
                ),
            ),
        ],
        &[],
    )
}

/// One named document, complete: the root schema plus every definition it may reference.
pub fn document(name: &str) -> Option<Value> {
    let root = match name {
        "envelope" => {
            let mut e = obj(&envelope_fields(), &[]);
            e["additionalProperties"] = json!(true);
            described(e, "open beyond the five fixed fields: each command adds its own payload beside them")
        }
        "diagnostic" => r("diagnostic"),
        "repair" => r("repair"),
        "authority" => r("authority"),
        "atlas" => r("atlas"),
        "sandbox" => r("sandbox"),
        "policy" => r("sandbox_preview"),
        "run-report" => {
            let mut req = envelope_fields();
            req.push(("sandbox", r("sandbox")));
            req.push(("outcome", obj(&[("ran", t("boolean")), ("exit", t("integer"))], &[("stopped_by", r("stopped_by"))])));
            obj(
                &req,
                &[
                    ("egress", r("egress")),
                    (
                        "audit",
                        obj(
                            &[("required_grants", arr(t("string"))), ("unsupported_surface", nullable("string"))],
                            &[],
                        ),
                    ),
                ],
            )
        }
        "toolchain" => toolchain_payload(),
        _ => return None,
    };
    let (_, what) = NAMES.iter().find(|(n, _)| *n == name)?;
    let mut doc = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("delulu:schema/{name}"),
        "title": format!("delulu {name}"),
        "description": what,
    });
    // A `$ref` root is written as `allOf` of one, so the document's own keywords sit beside it.
    if root.get("$ref").is_some() {
        doc["allOf"] = json!([root]);
    } else if let (Some(d), Some(r)) = (doc.as_object_mut(), root.as_object()) {
        for (k, v) in r {
            if k == "description" {
                d.insert(k.clone(), json!(format!("{what} — {}", v.as_str().unwrap_or(""))));
            } else {
                d.insert(k.clone(), v.clone());
            }
        }
    }
    doc["$defs"] = defs();
    Some(doc)
}

/// Validate `value` against the document `doc`. Every failure is reported with its JSON pointer.
pub fn validate(doc: &Value, value: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check(doc, doc, value, "", &mut errors);
    errors
}

fn type_matches(ty: &str, v: &Value) -> bool {
    match ty {
        "null" => v.is_null(),
        "boolean" => v.is_boolean(),
        "string" => v.is_string(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "array" => v.is_array(),
        "object" => v.is_object(),
        _ => false,
    }
}

fn check(doc: &Value, schema: &Value, v: &Value, at: &str, errors: &mut Vec<String>) {
    let here = if at.is_empty() { "/" } else { at };
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        match reference.strip_prefix("#/$defs/").and_then(|n| doc["$defs"].get(n)) {
            Some(target) => check(doc, target, v, at, errors),
            None => errors.push(format!("{here}: the schema refers to `{reference}`, which it does not define")),
        }
        return;
    }
    if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
        for s in all {
            check(doc, s, v, at, errors);
        }
    }
    if let Some(any) = schema.get("anyOf").and_then(Value::as_array) {
        let tries: Vec<Vec<String>> = any
            .iter()
            .map(|s| {
                let mut e = Vec::new();
                check(doc, s, v, at, &mut e);
                e
            })
            .collect();
        if !tries.iter().any(Vec::is_empty) {
            // Say what is wrong against the NEAREST shape, not only that none fitted: "matches none
            // of 3" names no field, and the field is what the reader has to fix.
            let nearest = tries.iter().min_by_key(|e| e.len()).cloned().unwrap_or_default();
            errors.push(format!(
                "{here}: matches none of the {} allowed shapes; against the nearest: {}",
                any.len(),
                nearest.join("; ")
            ));
        }
    }
    if let Some(ty) = schema.get("type") {
        let ok = match ty {
            Value::String(s) => type_matches(s, v),
            Value::Array(ts) => ts.iter().filter_map(Value::as_str).any(|s| type_matches(s, v)),
            _ => true,
        };
        if !ok {
            errors.push(format!("{here}: expected {ty}, found {}", kind_of(v)));
            return;
        }
    }
    if let Some(c) = schema.get("const") {
        if c != v {
            errors.push(format!("{here}: expected {c}, found {v}"));
        }
    }
    if let Some(e) = schema.get("enum").and_then(Value::as_array) {
        if !e.contains(v) {
            errors.push(format!("{here}: {v} is not one of {}", Value::Array(e.clone())));
        }
    }
    if let (Some(max), Some(a)) = (schema.get("maxItems").and_then(Value::as_u64), v.as_array()) {
        if a.len() as u64 > max {
            errors.push(format!("{here}: {} items, at most {max} allowed", a.len()));
        }
    }
    if let (Some(items), Some(a)) = (schema.get("items"), v.as_array()) {
        for (i, x) in a.iter().enumerate() {
            check(doc, items, x, &format!("{at}/{i}"), errors);
        }
    }
    if let Some(o) = v.as_object() {
        let props = schema.get("properties").and_then(Value::as_object);
        if let Some(req) = schema.get("required").and_then(Value::as_array) {
            for k in req.iter().filter_map(Value::as_str) {
                if !o.contains_key(k) {
                    errors.push(format!("{here}: the required field `{k}` is missing"));
                }
            }
        }
        for (k, x) in o {
            let path = format!("{at}/{k}");
            match props.and_then(|p| p.get(k)) {
                Some(s) => check(doc, s, x, &path, errors),
                None => match schema.get("additionalProperties") {
                    Some(Value::Bool(false)) => {
                        errors.push(format!("{path}: a field the schema does not describe"))
                    }
                    Some(extra @ Value::Object(_)) => check(doc, extra, x, &path, errors),
                    _ => {}
                },
            }
        }
    }
}

fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.is_f64() => "number",
        Value::Number(_) => "integer",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn cmd_schema(rest: &[String]) -> i32 {
    let json_out = rest.iter().any(|a| a == "--json");
    if let Some(bad) = rest.iter().find(|a| a.starts_with('-') && a.as_str() != "--json") {
        eprintln!("error: `schema` does not know the option `{bad}`");
        eprintln!("  nothing was done — an option nobody understood is refused, never ignored");
        return 2;
    }
    let args: Vec<&str> = rest.iter().filter(|a| !a.starts_with('-')).map(String::as_str).collect();
    match args.as_slice() {
        [] => {
            if json_out {
                let list: Vec<Value> = NAMES.iter().map(|(n, w)| json!({ "name": n, "describes": w })).collect();
                crate::cli::print_success_envelope("schema", json!({ "schemas": list }));
            } else {
                println!("delulu schema <name> [--json] — the JSON Schema of one output:");
                for (n, w) in NAMES {
                    println!("  {n:<12}{w}");
                }
                println!("delulu schema validate <name> <file.json> — check an output against it");
            }
            0
        }
        ["validate", name, file] => {
            let Some(doc) = document(name) else { return unknown(name) };
            let value: Value = match std::fs::read_to_string(file).map_err(|e| e.to_string()).and_then(|s| {
                serde_json::from_str(&s).map_err(|e| format!("it is not JSON: {e}"))
            }) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("error: cannot read `{file}`: {e}");
                    return 2;
                }
            };
            let errors = validate(&doc, &value);
            if json_out {
                crate::cli::print_success_envelope(
                    "schema",
                    json!({ "validate": { "schema": name, "file": file, "valid": errors.is_empty(), "errors": errors } }),
                );
            } else if errors.is_empty() {
                println!("ok: `{file}` is a valid `{name}`");
            } else {
                println!("`{file}` is NOT a valid `{name}` — {} problem(s):", errors.len());
                for e in &errors {
                    println!("  {e}");
                }
            }
            i32::from(!errors.is_empty())
        }
        [name] => {
            let Some(doc) = document(name) else { return unknown(name) };
            if json_out {
                // `document`, not `schema`: the envelope already has a `schema` field (its own version).
                crate::cli::print_success_envelope("schema", json!({ "name": name, "document": doc }));
            } else {
                println!("{}", serde_json::to_string_pretty(&doc).expect("a schema serializes"));
            }
            0
        }
        _ => {
            eprintln!("error: `schema` takes a name, or `validate <name> <file.json>`");
            2
        }
    }
}

fn unknown(name: &str) -> i32 {
    let known: Vec<&str> = NAMES.iter().map(|(n, _)| *n).collect();
    eprintln!("error: no schema named `{name}` — the names are: {}", known.join(", "));
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every document resolves every reference it makes, so a `$ref` to a definition that was renamed
    /// is a failure here and not a silent pass in the validator.
    #[test]
    fn every_document_is_complete_and_every_reference_resolves() {
        fn refs(v: &Value, out: &mut Vec<String>) {
            match v {
                Value::Object(o) => {
                    if let Some(r) = o.get("$ref").and_then(Value::as_str) {
                        out.push(r.to_string());
                    }
                    o.values().for_each(|x| refs(x, out));
                }
                Value::Array(a) => a.iter().for_each(|x| refs(x, out)),
                _ => {}
            }
        }
        for (name, _) in NAMES {
            let doc = document(name).unwrap_or_else(|| panic!("`{name}` is listed but has no document"));
            let mut all = Vec::new();
            refs(&doc, &mut all);
            for r in all {
                let n = r.strip_prefix("#/$defs/").expect("local references only");
                assert!(doc["$defs"].get(n).is_some(), "`{name}` refers to `{r}`");
            }
        }
        assert!(document("nope").is_none());
    }

    /// The validator refuses what a closed schema does not describe, and accepts what it does.
    #[test]
    fn the_validator_holds_the_line_a_closed_schema_draws() {
        let doc = document("repair").unwrap();
        let good = json!({
            "id": "x", "confidence": "exact", "authority_widening": false, "requires_human": false,
            "edits": [{ "file": "a", "range": { "start_byte": 0, "end_byte": 1 }, "insert": "" }], "reason": null,
        });
        assert!(validate(&doc, &good).is_empty(), "{:?}", validate(&doc, &good));
        let mut extra = good.clone();
        extra["new_field"] = json!(1);
        assert!(validate(&doc, &extra).iter().any(|e| e.contains("new_field")), "an undescribed field is refused");
        let mut missing = good.clone();
        missing.as_object_mut().unwrap().remove("requires_human");
        assert!(validate(&doc, &missing).iter().any(|e| e.contains("requires_human")));
        let mut wrong = good;
        wrong["confidence"] = json!("certainly");
        assert!(!validate(&doc, &wrong).is_empty(), "an unknown enum value is refused");
    }

    /// `toolchain --json`'s own payload is valid against its schema — the same check the CLI test
    /// makes through the binary, here without a process.
    #[test]
    fn the_toolchain_description_validates() {
        let doc = document("toolchain").unwrap();
        let errors = validate(&doc, &crate::toolchain::describe());
        assert!(errors.is_empty(), "{errors:#?}");
    }
}
