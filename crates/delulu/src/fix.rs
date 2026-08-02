//! `delulu fix` — apply the repairs the checker already computed.
//!
//! Nothing here invents a repair. Every one is a [`delulu_diag::Repair`] the checker attached to a
//! diagnostic, with byte-precise edits, and `delulu check --json` has reported them since Stage 1.
//! What was missing was a way to *apply* them without an editor: an agent, a CI job, or anyone at
//! a terminal had to re-implement the byte splicing themselves, and a hand-rolled splice against a
//! machine-readable contract is how the contract stops being followed.
//!
//! **What it refuses to do is the point.** [`delulu_diag::Confidence`] states the policy this
//! command implements — an `Exact` repair is byte-precise and safe to apply blindly *unless* it is
//! flagged `authority_widening` or `requires_human`:
//!
//! - **Authority-widening repairs are never applied by default, and never in bulk.** Widening what
//!   a program may do to your system is the one decision this language exists to keep with a
//!   person. You may accept one, but you must name it — `--accept-widening <repair-id>` — because
//!   a blanket flag on a batch command means "widen authority everywhere, unattended", and that
//!   flag ends up in a CI script.
//! - **`requires_human` repairs carry no edits at all.** They are documentation of a decision
//!   someone has to make; there is nothing to apply and nothing is pretended.
//! - **Only `Exact` repairs are applied.** `Safe` and `Suggest` are good guesses, and this command
//!   does not guess into your source. They are reported so you can see them.

use delulu_check::check_source;
use delulu_diag::{Confidence, Edit, Repair};

/// What happened to one repair, and why.
///
/// The reason travels with the repair to both surfaces, so the human report and the JSON envelope
/// cannot disagree about what this command did.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Applied,
    /// It would widen what the program may do. Nameable with `--accept-widening`, never automatic.
    Widening,
    /// Documentation of a decision, not an edit — there is nothing to apply.
    NeedsHuman,
    /// Not `Exact`: a good guess is still a guess, and this command does not guess into source.
    NotExact,
    /// Its bytes collide with a repair already applied in this run.
    Overlaps,
    /// Its edits name a file other than the one being fixed, or fall outside it. Either is a
    /// compiler bug rather than a user error, so it is reported loudly instead of skipped quietly.
    Unusable,
}

impl Verdict {
    fn word(self) -> &'static str {
        match self {
            Verdict::Applied => "applied",
            Verdict::Widening => "widens-authority",
            Verdict::NeedsHuman => "requires-human",
            Verdict::NotExact => "not-exact",
            Verdict::Overlaps => "overlaps",
            Verdict::Unusable => "unusable",
        }
    }

    /// Why a reader is seeing this, in one line.
    fn because(self) -> &'static str {
        match self {
            Verdict::Applied => "exact, and changes nothing about what this program may do",
            Verdict::Widening => {
                "would widen what this program may do — a tool must not decide that for you"
            }
            Verdict::NeedsHuman => "needs a human decision; it carries no edit to apply",
            Verdict::NotExact => "is a suggestion, not a mechanical certainty",
            Verdict::Overlaps => "touches bytes another repair in this run already changed",
            Verdict::Unusable => "names bytes outside this file — please report this as a bug",
        }
    }
}

struct Decision {
    code: &'static str,
    id: &'static str,
    confidence: &'static str,
    line: usize,
    verdict: Verdict,
}

pub fn cmd_fix(rest: &[String]) -> i32 {
    let mut file: Option<String> = None;
    let mut dry_run = false;
    let mut json = false;
    let mut accept: Vec<String> = Vec::new();

    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json = true,
            "--accept-widening" => {
                i += 1;
                match rest.get(i) {
                    Some(v) => accept.push(v.clone()),
                    None => {
                        eprintln!("error: `--accept-widening` needs a repair id\n\n{}", help());
                        return 2;
                    }
                }
            }
            // Human reason to stderr, exit 2, nothing on stdout: the single JSON envelope for a
            // usage error belongs to the outer dispatch, and emitting one here too would put two
            // objects on stdout — the exact defect `json_contract` exists to catch.
            other if other.starts_with('-') => {
                eprintln!("error: unknown option `{other}`\n\n{}", help());
                return 2;
            }
            other => {
                if file.is_some() {
                    eprintln!("error: `fix` takes one file\n\n{}", help());
                    return 2;
                }
                file = Some(other.to_string());
            }
        }
        i += 1;
    }

    let Some(file) = file else {
        eprintln!("error: `fix` needs a `.delulu` file\n\n{}", help());
        return 2;
    };
    let path = std::path::Path::new(&file);
    if path.is_dir() {
        eprintln!("error: `{file}` is a directory, and `fix` edits one file at a time");
        eprintln!(
            "note: name the file, e.g. `{}` — `delulu check {file}` reports the whole package",
            path.join("src").join("main.delulu").display()
        );
        return 2;
    }

    // `load` owns the good diagnostics for an unreadable file or a `.dwx` handed to a source
    // command, so it goes first and its exit code stands.
    let (map, id, src) = match crate::cli::load(&file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let Ok(on_disk) = std::fs::read_to_string(&file) else {
        eprintln!("error: `{file}` became unreadable while being fixed");
        return 2;
    };

    // **What was analysed must be what is on disk, or nothing may be written.**
    //
    // `load` translates a file stored in a surface morph into canonical DeluluLang, and every span
    // and repair offset downstream indexes that canonical text. The repaired result is therefore
    // canonical too — so writing it back would replace the author's chosen notation with English
    // keywords across the WHOLE file while leaving the `//! morph:` pragma still claiming
    // otherwise. That was observed, not deduced: with this check removed, a `zh-CN-keywords` file
    // asked to rename one identifier came back entirely in canonical keywords, and `delulu check`
    // then reported it clean, so nothing downstream would ever have flagged it.
    //
    // The comparison is against the bytes rather than against a pragma on purpose: a pragma test
    // would silently miss any future translation `load` learns to do.
    if src != on_disk {
        eprintln!(
            "error: `{file}` is not stored as canonical DeluluLang, so `fix` will not write to it"
        );
        eprintln!(
            "note: this file is translated to canonical text before it is analysed (a `//! morph:` \
             pragma does that), and the repairs describe that canonical text. Writing the repaired \
             result back would replace every keyword you wrote with its canonical spelling and \
             leave the pragma claiming your surface — so `fix` refuses instead."
        );
        eprintln!(
            "note: to repair it anyway, do the translation yourself so you can see it:\n  \
             delulu morph render {file} --to-canonical > canonical.delulu\n  \
             delulu fix canonical.delulu\n  \
             delulu morph render canonical.delulu --to <your-morph-id>"
        );
        eprintln!(
            "note: `delulu check {file}` reports every repair either way, and the language server \
             applies them one at a time where you can see the result"
        );
        return 1;
    }

    let checked = check_source(id, &src);
    let errors_before = checked.diagnostics.iter().filter(|d| d.is_error()).count();

    // Decide, in diagnostic order, so the same file always produces the same answer.
    let mut decisions: Vec<Decision> = Vec::new();
    let mut chosen: Vec<&Repair> = Vec::new();
    let mut claimed: Vec<(u32, u32)> = Vec::new();

    for d in &checked.diagnostics {
        for r in &d.repairs {
            let line = r.edits.first().map(|e| line_of(&src, e.start_byte)).unwrap_or(0);
            let verdict = classify(r, id, src.len(), &accept, &claimed);
            if verdict == Verdict::Applied {
                for e in &r.edits {
                    claimed.push((e.start_byte, e.end_byte));
                }
                chosen.push(r);
            }
            decisions.push(Decision {
                code: d.code,
                id: r.id,
                confidence: r.confidence.as_str(),
                line,
                verdict,
            });
        }
    }

    // Apply back to front, so every offset still indexes the text it was computed against.
    let mut out = src.clone();
    let mut edits: Vec<&Edit> = chosen.iter().flat_map(|r| r.edits.iter()).collect();
    // **Back to front.** Every offset was computed against the unedited text, so applying in
    // ascending order would leave each later edit indexing bytes an earlier one had already
    // shifted. Observed, with an ascending sort: two identifier renames one line apart produced
    // `consume_e` and swallowed a space — the file still parsed, which is what makes that class of
    // bug expensive to find later.
    edits.sort_by_key(|e| std::cmp::Reverse(e.start_byte));
    for e in edits {
        out.replace_range(e.start_byte as usize..e.end_byte as usize, &e.insert);
    }

    // **A mechanical repair must never break parsing.** If it does, something is wrong with the
    // repair and not with the file, and the right move is to write nothing. This is not "the error
    // count must not rise" — fixing a parse error legitimately reveals the type errors it was
    // masking, and a guard that punished that would block the most useful fixes there are.
    let broke_parsing = parse_errors(&src) == 0 && parse_errors(&out) > 0;
    let applied = decisions.iter().filter(|d| d.verdict == Verdict::Applied).count();
    let mut written = false;

    if broke_parsing {
        eprintln!(
            "error: applying {applied} repair(s) to `{file}` produced source that no longer \
             parses, so nothing was written"
        );
        eprintln!("note: this is a defect in the repair, not in your file — please report it with `{file}`");
        return 1;
    }

    if applied > 0 && !dry_run {
        if let Err(e) = std::fs::write(&file, &out) {
            eprintln!("error: cannot write `{file}`: {e}");
            return 2;
        }
        written = true;
    }

    let errors_after = if applied > 0 {
        let recheck = check_source(id, &out);
        recheck.diagnostics.iter().filter(|d| d.is_error()).count()
    } else {
        errors_before
    };

    if json {
        crate::cli::note_json_emitted();
        let mut env = delulu_diag::envelope("fix", &checked.diagnostics, None, &map);
        env["fix"] = serde_json::json!({
            "file": file,
            "written": written,
            "dry_run": dry_run,
            "applied": applied,
            "skipped": decisions.len() - applied,
            "errors_before": errors_before,
            "errors_after": errors_after,
            "repairs": decisions.iter().map(|d| serde_json::json!({
                "code": d.code,
                "id": d.id,
                "confidence": d.confidence,
                "line": d.line,
                "verdict": d.verdict.word(),
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&env).expect("envelope serialization cannot fail"));
    } else {
        render(&file, &decisions, applied, written, dry_run, errors_before, errors_after);
    }
    0
}

/// Whether this repair may be applied, and if not, why not.
///
/// The order of the tests is the policy, and it is deliberate: **the authority question is asked
/// before the mechanical ones.** A repair that widens authority is reported as widening even if it
/// would also have been skipped for being inexact, because that is the fact a reader needs.
fn classify(
    r: &Repair,
    file: delulu_diag::FileId,
    len: usize,
    accept: &[String],
    claimed: &[(u32, u32)],
) -> Verdict {
    if r.authority_widening && !accept.iter().any(|a| a == r.id) {
        return Verdict::Widening;
    }
    if r.requires_human || r.edits.is_empty() {
        return Verdict::NeedsHuman;
    }
    if r.confidence != Confidence::Exact {
        return Verdict::NotExact;
    }
    if r.edits.iter().any(|e| {
        e.file != file || e.start_byte > e.end_byte || e.end_byte as usize > len
    }) {
        return Verdict::Unusable;
    }
    if r.edits.iter().any(|e| claimed.iter().any(|c| conflicts((e.start_byte, e.end_byte), *c))) {
        return Verdict::Overlaps;
    }
    Verdict::Applied
}

/// Whether two edit ranges cannot both be applied.
///
/// Two ranges conflict when they share a byte. A pure insertion (`start == end`) additionally
/// conflicts with any range it lands strictly inside, because "replace these bytes" and "insert
/// among them" have no defined joint meaning. Two insertions at the same point do **not** conflict
/// — they concatenate, in the order the diagnostics were reported, which is deterministic.
fn conflicts(a: (u32, u32), b: (u32, u32)) -> bool {
    let shares_a_byte = a.0.max(b.0) < a.1.min(b.1);
    let a_inside_b = a.0 == a.1 && b.0 < a.0 && a.0 < b.1;
    let b_inside_a = b.0 == b.1 && a.0 < b.0 && b.0 < a.1;
    shares_a_byte || a_inside_b || b_inside_a
}

/// How many errors the parser alone reports. Used only to prove a repair did not break the file.
fn parse_errors(src: &str) -> usize {
    let mut map = delulu_diag::SourceMap::new();
    let id = map.add_file("<fix>", src.to_string());
    let (_module, diags) = delulu_syntax::parse_file(id, src);
    diags.iter().filter(|d| d.is_error()).count()
}

/// The 1-based line a byte offset falls on.
fn line_of(src: &str, byte: u32) -> usize {
    src.bytes().take(byte as usize).filter(|b| *b == b'\n').count() + 1
}

fn render(
    file: &str,
    decisions: &[Decision],
    applied: usize,
    written: bool,
    dry_run: bool,
    errors_before: usize,
    errors_after: usize,
) {
    if decisions.is_empty() {
        eprintln!("fix: {file} — nothing to repair");
        return;
    }
    let headline = if dry_run {
        format!("fix --dry-run: {file} — {applied} repair(s) would be applied, nothing written")
    } else if written {
        format!("fix: {file} — {applied} repair(s) applied")
    } else {
        format!("fix: {file} — no repair could be applied automatically")
    };
    eprintln!("{headline}");

    for verdict in [
        Verdict::Applied,
        Verdict::Widening,
        Verdict::NeedsHuman,
        Verdict::NotExact,
        Verdict::Overlaps,
        Verdict::Unusable,
    ] {
        let group: Vec<&Decision> = decisions.iter().filter(|d| d.verdict == verdict).collect();
        if group.is_empty() {
            continue;
        }
        // Under `--dry-run` nothing was applied, and a heading that says otherwise is the kind of
        // small lie that makes someone trust the next report less.
        let heading = match (verdict, dry_run) {
            (Verdict::Applied, true) => "would apply",
            _ => verdict.word(),
        };
        eprintln!("\n  {heading} — {}", verdict.because());
        for d in &group {
            eprintln!("    {:<8} {:<28} line {}  [{}]", d.code, d.id, d.line, d.confidence);
        }
        if verdict == Verdict::Widening {
            // Naming the exact command matters: a refusal a reader cannot act on is an obstacle,
            // not a safeguard. One repair id at a time, never a blanket accept-all.
            eprintln!(
                "\n    Review each with `delulu check {file}`. To accept ONE, name it:\n      \
                 delulu fix {file} --accept-widening {}",
                group[0].id
            );
        }
    }

    if errors_after != errors_before {
        eprintln!("\n  errors: {errors_before} → {errors_after}");
    }
    if errors_after > 0 {
        eprintln!("  {errors_after} error(s) remain — `delulu check {file}`");
    }
}

fn help() -> String {
    "delulu fix — apply the repairs the checker already computed\n\n\
     USAGE:\n  \
       delulu fix <file.delulu>                       apply every exact, non-widening repair\n  \
       delulu fix <file.delulu> --dry-run             report what would change; write nothing\n  \
       delulu fix <file.delulu> --json                one JSON envelope\n  \
       delulu fix <file.delulu> --accept-widening ID  also apply the named authority-widening repair\n\n\
     A repair that would widen what your program may do is NEVER applied unless you name it.\n\
     There is deliberately no flag that accepts all of them at once.\n\n\
     Exit 0 when the file was handled, 1 when nothing could safely be written, 2 on a bad invocation."
        .to_string()
}
