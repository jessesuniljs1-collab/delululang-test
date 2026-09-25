//! P4-04 and P4-05: `delulu edit` — checked edits for an agent that is not the only writer.
//!
//! An agent edits a file it read a moment ago. Between the read and the write, a person — or another
//! agent — may have changed it, and an edit computed against the old bytes lands in the wrong place,
//! silently. So an edit here names the hash of the bytes it was computed against (`--expect-hash`,
//! blake3, exactly as `delulu edit --json` and every other answer below report it), and is REFUSED
//! when the file no longer has them; the refusal carries the file's current hash, so the caller
//! re-reads and tries again rather than guessing.
//!
//! Two ways to say what changes:
//!
//! - **Byte ranges** (`--edits JSON`, or `--edits @FILE`): a list of `{range: {start_byte, end_byte},
//!   insert}` — the shape a diagnostic's typed repair already carries, so a repair's `edits` can be
//!   passed straight through. Ranges must lie on character boundaries and may not overlap; they are
//!   applied together or not at all.
//! - **A node** (`--node ID --with TEXT`, or `--with @FILE`): replace one item the Atlas names
//!   (`fn:pkg/module.name`, `type:…`) with new text, which must parse as one item of the same kind and
//!   is put through the formatter on its own, so the rest of the file is not reformatted. A stale id —
//!   one the current file no longer has — is refused.
//!
//! Either way the edited source is CHECKED and the answer is one envelope: the new source's
//! diagnostics, the old and new hashes, and whether the file was written. `--dry-run` writes nothing;
//! `--if-checks` writes only when the result has no errors. A file stored in a surface morph is refused:
//! its bytes are not the canonical program the offsets and the Atlas refer to. The write replaces the
//! file in one rename, so a reader never sees half an edit.

use serde_json::{json, Value};

/// The blake3 of a file's bytes as stored, hex — the hash `--expect-hash` names.
pub(crate) fn file_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// One byte-range edit.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RangeEdit {
    start: usize,
    end: usize,
    insert: String,
}

/// Parse the edit list: the repair-edit shape, `file` tolerated and ignored (the command names the
/// file), nothing else accepted.
fn parse_edits(v: &Value) -> Result<Vec<RangeEdit>, String> {
    let list = v.as_array().ok_or("the edits must be a JSON list")?;
    let mut out = Vec::with_capacity(list.len());
    for (i, e) in list.iter().enumerate() {
        let obj = e.as_object().ok_or_else(|| format!("edit {i} is not an object"))?;
        if let Some(k) = obj.keys().find(|k| !["range", "insert", "file"].contains(&k.as_str())) {
            return Err(format!("edit {i} has a field `{k}` an edit does not have"));
        }
        let n = |k: &str| e["range"][k].as_u64().ok_or_else(|| format!("edit {i}: `range.{k}` must be a byte offset"));
        let (start, end) = (n("start_byte")? as usize, n("end_byte")? as usize);
        let insert = e["insert"].as_str().ok_or_else(|| format!("edit {i}: `insert` must be a string"))?.to_string();
        out.push(RangeEdit { start, end, insert });
    }
    Ok(out)
}

/// Apply byte-range edits to `src`, all or none: every range in bounds, on character boundaries, and
/// no two overlapping (two inserts at the same point are overlapping too — their order would be a
/// guess).
fn apply(src: &str, mut edits: Vec<RangeEdit>) -> Result<String, String> {
    for (i, e) in edits.iter().enumerate() {
        if e.start > e.end {
            return Err(format!("edit {i}: its range ends before it starts ({}..{})", e.start, e.end));
        }
        if e.end > src.len() {
            return Err(format!("edit {i}: its range {}..{} runs past the end of the file ({} bytes)", e.start, e.end, src.len()));
        }
        if !src.is_char_boundary(e.start) || !src.is_char_boundary(e.end) {
            return Err(format!("edit {i}: {}..{} splits a character", e.start, e.end));
        }
    }
    edits.sort_by_key(|e| (e.start, e.end));
    for w in edits.windows(2) {
        if w[1].start < w[0].end || (w[1].start == w[0].start && w[0].start == w[0].end) {
            return Err(format!("edits {}..{} and {}..{} overlap", w[0].start, w[0].end, w[1].start, w[1].end));
        }
    }
    let mut out = src.to_string();
    for e in edits.iter().rev() {
        out.replace_range(e.start..e.end, &e.insert);
    }
    Ok(out)
}

/// What an Atlas `fn:`/`type:` id names in a file: a top-level function or type, or one member of
/// an actor — the Atlas names an actor's constructor, behaviors and helper functions as
/// `fn:pkg/module.Actor.member`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fn,
    Type,
    ActorNew,
    ActorBe,
    ActorFn,
}

impl Kind {
    fn word(self) -> &'static str {
        match self {
            Kind::Fn => "fn",
            Kind::Type => "type",
            Kind::ActorNew => "actor constructor (`new`)",
            Kind::ActorBe => "behavior (`be`)",
            Kind::ActorFn => "actor fn",
        }
    }

    fn in_actor(self) -> bool {
        matches!(self, Kind::ActorNew | Kind::ActorBe | Kind::ActorFn)
    }
}

fn span_of(s: delulu_diag::Span) -> (usize, usize) {
    (s.start as usize, s.end as usize)
}

/// Resolve an Atlas node id to one item's span in `src`. The id must name this file's module, and
/// the item must exist NOW — an id from an older read that names nothing is stale, and refused.
fn resolve_node(src: &str, id: &str) -> Result<(usize, usize, Kind), String> {
    use delulu_syntax::ast::Item;
    let (prefix, rest) = id.split_once(':').ok_or_else(|| format!("`{id}` is not an Atlas node id (`fn:pkg/module.name`)"))?;
    if prefix != "fn" && prefix != "type" {
        return Err(format!("`{id}`: only functions (`fn:`) and types (`type:`) can be edited by id"));
    }
    let (module, diags) = delulu_syntax::parse_file(0, src);
    if diags.iter().any(|d| d.is_error()) {
        return Err("the file does not parse, so its items cannot be addressed".into());
    }
    let dotted = module.name.dotted();
    let qualified = rest.rsplit_once('/').map_or(rest, |(_, m)| m);
    let Some(name) = qualified.strip_prefix(dotted.as_str()).and_then(|n| n.strip_prefix('.')) else {
        return Err(format!("`{id}` does not name an item of this file's module `{dotted}`"));
    };
    let found = match (prefix, name.split_once('.')) {
        ("type", None) => module.items.iter().find_map(|it| match it {
            Item::Type(t) if t.name.name == name => Some((span_of(t.span), Kind::Type)),
            _ => None,
        }),
        ("fn", None) => module.items.iter().find_map(|it| match it {
            Item::Fn(f) if f.name.name == name => Some((span_of(f.span), Kind::Fn)),
            _ => None,
        }),
        ("fn", Some((actor, member))) => module.items.iter().find_map(|it| match it {
            Item::Actor(a) if a.name.name == actor => {
                if member == "new" {
                    Some((span_of(a.ctor.span), Kind::ActorNew))
                } else {
                    a.behaviors
                        .iter()
                        .find(|b| b.name.name == member)
                        .map(|b| (span_of(b.span), Kind::ActorBe))
                        .or_else(|| a.fns.iter().find(|f| f.name.name == member).map(|f| (span_of(f.span), Kind::ActorFn)))
                }
            }
            _ => None,
        }),
        _ => None,
    };
    found
        .map(|((s, e), k)| (s, e, k))
        .ok_or_else(|| format!("`{id}` is stale: this file has no such {prefix} now — re-read it and ask the Atlas again"))
}

/// Canonicalize one replacement: it must parse as exactly one item of `kind` — nothing more, so a
/// replacement cannot bring a second item in — and it is formatted on its own inside a throwaway
/// module (and, for an actor member, a throwaway actor), so the rest of the file keeps whatever form
/// it had. Comments the replacement carries are kept.
fn canonical_item(text: &str, kind: Kind) -> Result<String, String> {
    use delulu_syntax::ast::Item;
    const HEADER: &str = "module edit_item\n\n";
    const ACTOR: &str = "actor EditItem {";
    // A behavior or helper fn needs an actor around it, and an actor needs its one constructor.
    let dummy = if matches!(kind, Kind::ActorBe | Kind::ActorFn) { "    new() {}\n" } else { "" };
    let body = text.trim();
    let wrapped = if kind.in_actor() { format!("{HEADER}{ACTOR}\n{dummy}{body}\n}}\n") } else { format!("{HEADER}{body}\n") };
    let parse = |src: &str| {
        let (module, diags) = delulu_syntax::parse_file(0, src);
        match diags.iter().find(|d| d.is_error()) {
            Some(d) => Err(format!("the replacement does not parse: {}", d.message)),
            None => Ok(module),
        }
    };
    let module = parse(&wrapped)?;
    let wrong = || format!("the replacement must be exactly one {}, and nothing else", kind.word());
    let fits = match (kind, module.items.as_slice()) {
        (Kind::Fn, [Item::Fn(_)]) | (Kind::Type, [Item::Type(_)]) => true,
        // The constructor needs no test here: the parser holds an actor to exactly one `new`
        // (DL0201), so a replacement that brings its own beside the throwaway one, or a `new` that
        // brings none, has already failed to parse.
        (_, [Item::Actor(a)]) if kind.in_actor() => {
            a.fields.is_empty()
                && match kind {
                    Kind::ActorNew => a.behaviors.is_empty() && a.fns.is_empty(),
                    Kind::ActorBe => a.behaviors.len() == 1 && a.fns.is_empty(),
                    _ => a.behaviors.is_empty() && a.fns.len() == 1,
                }
        }
        _ => false,
    };
    if !fits {
        return Err(wrong());
    }
    let formatted = delulu_syntax::fmt::format_source(0, &wrapped).map_err(|_| "the replacement could not be formatted".to_string())?;
    if !kind.in_actor() {
        return Ok(formatted.strip_prefix(HEADER).unwrap_or(&formatted).trim().to_string());
    }
    // The member is what lies between the throwaway parts: after the actor's `{` (or after the
    // throwaway constructor) and before the actor's closing `}`. Its first line loses the member
    // indentation, which the line it lands on already has; the lines after keep theirs.
    let module = parse(&formatted)?;
    let Some(Item::Actor(a)) = module.items.first() else { return Err(wrong()) };
    let from = match kind {
        Kind::ActorNew => formatted.find(ACTOR).map_or(0, |i| i + ACTOR.len()),
        _ => a.ctor.span.end as usize,
    };
    let to = (a.span.end as usize).saturating_sub(1);
    formatted.get(from..to).map(|m| m.trim().to_string()).ok_or_else(wrong)
}

/// The parts of the authority report an edit can widen: what the program performs, the grants it
/// needs to run, and the foreign code it calls.
const AUTHORITY_ROWS: [&str; 3] = ["effects", "required_grants", "foreign_calls"];

/// What the program may do before the edit and after it. `widened` lists what the edit ADDS, so an
/// agent's edit that gives a program a new effect or a new grant says so in the answer rather than
/// only in a later `authority` — the rule `fix` keeps for its own repairs (it never applies a
/// widening one unless named). Empty when nothing is added; `null` when either side does not check,
/// because a program that does not check has no authority to compare.
fn authority_change(file: &str, before: &str, after: &str) -> Value {
    let rows = |src: &str| {
        crate::cli::authority_of_text(file, src)
            .ok()
            .map(|r| Value::Object(AUTHORITY_ROWS.iter().map(|k| (k.to_string(), r[*k].clone())).collect()))
    };
    let (b, a) = (rows(before), rows(after));
    let widened = match (&b, &a) {
        (Some(b), Some(a)) => Value::Array(
            AUTHORITY_ROWS
                .iter()
                .flat_map(|k| {
                    let had = b[*k].as_array().cloned().unwrap_or_default();
                    a[*k].as_array().cloned().unwrap_or_default().into_iter().filter(move |x| !had.contains(x))
                })
                .collect(),
        ),
        _ => Value::Null,
    };
    json!({ "before": b, "after": a, "widened": widened })
}

/// Read a value given as TEXT or as `@FILE`.
fn text_or_file(v: &str) -> Result<String, String> {
    match v.strip_prefix('@') {
        Some(path) => std::fs::read_to_string(path).map_err(|e| format!("cannot read `{path}`: {e}")),
        None => Ok(v.to_string()),
    }
}

/// Replace `path` with `bytes` in one rename, so a reader sees the old file or the new one.
fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let tmp = dir.join(format!(".{}.delulu-edit-{}", path.file_name().and_then(|n| n.to_str()).unwrap_or("file"), std::process::id()));
    std::fs::write(&tmp, bytes)?;
    if let Ok(meta) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

struct Args {
    file: String,
    expect: Option<String>,
    edits: Option<String>,
    node: Option<String>,
    with: Option<String>,
    dry_run: bool,
    if_checks: bool,
    json: bool,
}

fn parse_args(rest: &[String]) -> Result<Args, String> {
    let mut a = Args { file: String::new(), expect: None, edits: None, node: None, with: None, dry_run: false, if_checks: false, json: false };
    let mut i = 0;
    let mut files = Vec::new();
    while i < rest.len() {
        let s = rest[i].as_str();
        let mut value = |name: &str| -> Result<String, String> {
            i += 1;
            rest.get(i).cloned().ok_or_else(|| format!("`{name}` needs a value"))
        };
        match s {
            "--expect-hash" => a.expect = Some(value("--expect-hash")?),
            "--edits" => a.edits = Some(value("--edits")?),
            "--node" => a.node = Some(value("--node")?),
            "--with" => a.with = Some(value("--with")?),
            "--dry-run" => a.dry_run = true,
            "--if-checks" => a.if_checks = true,
            "--json" => a.json = true,
            f if f.starts_with('-') => return Err(format!("`edit` does not know the option `{f}`")),
            f => files.push(f.to_string()),
        }
        i += 1;
    }
    match files.as_slice() {
        [f] => a.file = f.clone(),
        [] => return Err("`edit` needs the file to edit".into()),
        _ => return Err("`edit` takes exactly one file".into()),
    }
    if a.expect.is_none() {
        return Err("`edit` needs `--expect-hash` — the blake3 of the bytes the edit was computed against".into());
    }
    match (&a.edits, &a.node, &a.with) {
        (Some(_), None, None) | (None, Some(_), Some(_)) => Ok(a),
        _ => Err("say what changes with EITHER `--edits JSON|@FILE` OR `--node ID --with TEXT|@FILE`".into()),
    }
}

/// A refusal: exit 2, and under `--json` one envelope that says why and carries the file's current
/// hash when there is one, so a caller can re-read without a second command.
fn refuse(json_out: bool, file: &str, why: &str, current: Option<&str>) -> i32 {
    if json_out {
        crate::cli::print_success_envelope(
            "edit",
            json!({ "edit": { "file": file, "refused": why, "hash": current, "written": false }, "summary": { "errors": 1 } }),
        );
    } else {
        eprintln!("error: {why}");
        if let Some(h) = current {
            eprintln!("  the file's hash now is {h}");
        }
        eprintln!("  nothing was written");
    }
    2
}

pub fn cmd_edit(rest: &[String]) -> i32 {
    let json_out = rest.iter().any(|a| a == "--json");
    let a = match parse_args(rest) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let bytes = match std::fs::read(&a.file) {
        Ok(b) => b,
        Err(e) => return refuse(json_out, &a.file, &format!("cannot read `{}`: {e}", a.file), None),
    };
    let current = file_hash(&bytes);
    let expected = a.expect.as_deref().unwrap_or("").to_ascii_lowercase();
    if expected != current {
        return refuse(
            json_out,
            &a.file,
            "the file changed since the edit was computed: its hash is not the one `--expect-hash` names",
            Some(&current),
        );
    }
    let Ok(src) = String::from_utf8(bytes) else {
        return refuse(json_out, &a.file, "the file is not UTF-8, so it is not DeluluLang source", Some(&current));
    };
    if delulu_syntax::morph::pragma_of(&src).is_some() {
        return refuse(
            json_out,
            &a.file,
            "this file is stored in a surface morph: byte offsets and Atlas ids refer to the canonical \
             program, not these bytes. Edit its canonical form (`delulu morph render F --to-canonical`)",
            Some(&current),
        );
    }

    let (edited, count, node) = if let Some(spec) = &a.edits {
        let parsed = text_or_file(spec).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| format!("the edits are not JSON: {e}")));
        match parsed.and_then(|v| parse_edits(&v)).and_then(|edits| {
            let n = edits.len();
            apply(&src, edits).map(|s| (s, n))
        }) {
            Ok((s, n)) => (s, n, None),
            Err(e) => return refuse(json_out, &a.file, &e, Some(&current)),
        }
    } else {
        let id = a.node.clone().unwrap_or_default();
        let result = resolve_node(&src, &id).and_then(|(start, end, kind)| {
            let text = text_or_file(a.with.as_deref().unwrap_or(""))?;
            let item = canonical_item(&text, kind)?;
            apply(&src, vec![RangeEdit { start, end, insert: item }])
        });
        match result {
            Ok(s) => (s, 1, Some(id)),
            Err(e) => return refuse(json_out, &a.file, &e, Some(&current)),
        }
    };

    // Checked as the file it will be.
    let mut map = delulu_diag::SourceMap::new();
    let fid = map.add_file(&a.file, &edited);
    let checked = delulu_check::check_source(fid, &edited);
    let errors = checked.diagnostics.iter().filter(|d| d.is_error()).count();
    let new_hash = file_hash(edited.as_bytes());
    let write = !(a.dry_run || (a.if_checks && errors > 0));
    let mut written = false;
    if write {
        if let Err(e) = write_atomically(std::path::Path::new(&a.file), edited.as_bytes()) {
            return refuse(json_out, &a.file, &format!("the edited file could not be written: {e}"), Some(&current));
        }
        written = true;
    }
    let authority = authority_change(&a.file, &src, &edited);
    let report = json!({
        "file": a.file,
        "previous_hash": current,
        "hash": new_hash,
        "edits_applied": count,
        "node": node,
        "written": written,
        "dry_run": a.dry_run,
        "authority": authority,
    });
    if json_out {
        crate::cli::note_json_emitted();
        let mut env = delulu_diag::envelope("edit", &checked.diagnostics, None, &map);
        env["edit"] = report;
        println!("{}", serde_json::to_string_pretty(&env).expect("the edit envelope serializes"));
    } else {
        crate::cli::print_diagnostics("edit", &checked.diagnostics, &map, None, false);
        let what = if written { "written" } else if a.dry_run { "not written (--dry-run)" } else { "not written (--if-checks: it does not check)" };
        println!("{}: {count} edit(s) applied, {what}; hash {new_hash}", a.file);
        if let Some(added) = authority["widened"].as_array().filter(|w| !w.is_empty()) {
            let added: Vec<String> = added.iter().map(|x| x.as_str().map_or_else(|| x.to_string(), str::to_string)).collect();
            eprintln!("note: this edit WIDENS what the program may do: adds {}", added.join(", "));
        }
    }
    i32::from(errors > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(start: usize, end: usize, insert: &str) -> RangeEdit {
        RangeEdit { start, end, insert: insert.into() }
    }

    #[test]
    fn range_edits_apply_together_or_not_at_all() {
        let src = "abcdef";
        assert_eq!(apply(src, vec![e(4, 6, "XY"), e(0, 1, "Z")]).unwrap(), "ZbcdXY", "order does not matter");
        assert!(apply(src, vec![e(1, 3, ""), e(2, 4, "")]).is_err(), "overlap");
        assert!(apply(src, vec![e(2, 2, "a"), e(2, 2, "b")]).is_err(), "two inserts at one point: their order would be a guess");
        assert!(apply(src, vec![e(5, 9, "")]).is_err(), "past the end");
        assert!(apply(src, vec![e(3, 2, "")]).is_err(), "inverted");
        assert!(apply("é", vec![e(1, 2, "")]).is_err(), "inside a character");
        assert_eq!(apply(src, vec![e(6, 6, "!")]).unwrap(), "abcdef!", "an insert at the end");
    }

    #[test]
    fn the_edit_list_is_the_repair_shape_and_nothing_else() {
        let ok = serde_json::json!([{ "file": "x", "range": { "start_byte": 0, "end_byte": 1 }, "insert": "y" }]);
        assert_eq!(parse_edits(&ok).unwrap(), vec![e(0, 1, "y")]);
        assert!(parse_edits(&serde_json::json!([{ "range": { "start_byte": 0, "end_byte": 1 }, "insert": "y", "extra": 1 }])).is_err());
        assert!(parse_edits(&serde_json::json!({ "range": {} })).is_err());
    }

    #[test]
    fn a_node_resolves_by_its_atlas_id_and_a_stale_one_is_refused() {
        let src = "module m\n\nfn a() -> Int {\n    1\n}\n\nfn b() -> Int {\n    2\n}\n";
        let (s, end, kind) = resolve_node(src, "fn:m/m.b").unwrap();
        assert_eq!(kind, Kind::Fn);
        assert!(src[s..end].starts_with("fn b"), "{}", &src[s..end]);
        assert!(resolve_node(src, "fn:m/m.gone").unwrap_err().contains("stale"));
        assert!(resolve_node(src, "fn:other/other.b").is_err(), "another module's id");
        assert!(resolve_node(src, "mod:m/m").is_err());
        let item = canonical_item("fn b()->Int{3}", Kind::Fn).unwrap();
        assert!(item.starts_with("fn b() -> Int {"), "formatted: {item}");
        assert!(canonical_item("fn b() -> Int { 3 }\nfn c() {}", Kind::Fn).is_err(), "two items");
        assert!(canonical_item("type T = Int", Kind::Fn).is_err(), "the wrong kind");
    }

    #[test]
    fn an_actor_member_is_addressed_formatted_and_kept_to_one_member() {
        let src = "module m

actor A {
    var n: Int

    new() {
        self.n = 0
    }

    be hit(k: Int) {
        self.n = self.n + k
    }

    fn get() -> Int {
        self.n
    }
}
";
        let (s, e, k) = resolve_node(src, "fn:m/m.A.hit").unwrap();
        assert_eq!(k, Kind::ActorBe);
        assert!(src[s..e].starts_with("be hit"), "{}", &src[s..e]);
        assert_eq!(resolve_node(src, "fn:m/m.A.get").unwrap().2, Kind::ActorFn);
        assert_eq!(resolve_node(src, "fn:m/m.A.new").unwrap().2, Kind::ActorNew);
        assert!(resolve_node(src, "fn:m/m.A.gone").unwrap_err().contains("stale"));
        assert!(resolve_node(src, "fn:m/m.B.hit").unwrap_err().contains("stale"));
        let be = canonical_item("be hit(k:Int){self.n=self.n+k*2}", Kind::ActorBe).unwrap();
        assert_eq!(be, "be hit(k: Int) {
        self.n = self.n + k * 2
    }", "indented for the member depth it lands at");
        assert_eq!(apply(src, vec![e_at(s, e, &be)]).unwrap().matches("k * 2").count(), 1);
        // Exactly one member of the kind, and nothing else: no second member, no field, no constructor
        // brought in beside a behavior, no behavior where a constructor was addressed.
        assert!(canonical_item("be hit(k: Int) {}
be other() {}", Kind::ActorBe).is_err());
        assert!(canonical_item("var x: Int
be hit(k: Int) {}", Kind::ActorBe).is_err());
        assert!(canonical_item("new() {}
be hit(k: Int) {}", Kind::ActorBe).is_err());
        assert!(canonical_item("be hit(k: Int) {}", Kind::ActorNew).is_err());
        assert!(canonical_item("fn get() -> Int { 1 }", Kind::ActorBe).is_err());
        assert!(canonical_item("new() { self.n = 1 }", Kind::ActorNew).unwrap().starts_with("new() {"));
    }

    fn e_at(start: usize, end: usize, insert: &str) -> RangeEdit {
        RangeEdit { start, end, insert: insert.into() }
    }
}
