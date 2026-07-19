//! The conformance coverage law (Stage 9, invariant 42), mechanized.
//!
//! Invariant 42: the conformance suite is the specification's executable form. Every grammar
//! production, primitive-table entry, audit rule (R-1…R-7), diagnostic code, and CLI subcommand
//! contract must have at least one **accepting** and one **rejecting** conformance test, and the
//! coverage map is machine-checked. `delulu-conform --coverage` builds the anchor registry from
//! the compiler source, scans the test witnesses, and fails (nonzero exit) on any gap.
//!
//! ## The anchor registry (built mechanically — it cannot drift)
//! - `ref.diag.<CODE>` — one per [`delulu_diag::REGISTRY`] entry.
//! - `ref.prim.<recv>.<method>` — one per [`delulu_check::prim_table::PRIM_TABLE`] entry.
//! - `ref.grammar.<production>` — one per [`delulu_syntax::grammar::GRAMMAR_PRODUCTIONS`] entry.
//! - `ref.audit.R-1`…`ref.audit.R-7` — the soundness-audit rules.
//! - `ref.cli.<subcommand>` — one per CLI subcommand contract ([`CLI_SUBCOMMANDS`]).
//!
//! ## Witnesses
//! - **Conformance headers**: a `.delulu` file may carry a `// anchors: <id> <id> …` comment.
//!   Files under `tests/conformance/accept/` (and `tests/corpus/`) are *positive* witnesses; files
//!   under `tests/conformance/reject/` are *negative* witnesses. A reject file named
//!   `DLxxxx_*.delulu` is additionally an automatic negative witness for `ref.diag.DLxxxx`.
//! - **Witness map** (`tests/conformance/witnesses.toml`): maps an anchor to existing Rust tests,
//!   each tagged `positive`/`negative`. Every named test is existence- and `#[ignore]`-verified;
//!   a dangling or ignored witness is a coverage FAILURE (skip-branch rule).
//!
//! ## Fail-closed conditions (nonzero exit)
//! Any anchor missing a positive witness; any missing a negative witness (for a `ref.diag` anchor
//! this is exactly "the suite never produces this code"); any citation of an unknown anchor; any
//! unparseable witness metadata; a dangling/ignored Rust-test witness; an empty registry.
//!
//! ## `--json` schema (`conform-coverage/1`)
//! ```json
//! {
//!   "tool": "delulu-conform",
//!   "schema": "conform-coverage/1",
//!   "delulu_version": "0.1.0",
//!   "pass": false,
//!   "summary": { "anchors": N, "covered": M, "coverage_pct": 0.0,
//!                "positive_gaps": p, "negative_gaps": q, "validation_errors": e },
//!   "by_category": { "diag": {"anchors":..,"covered":..}, "prim": {...}, ... },
//!   "gaps": [ {"anchor": "ref.diag.DL0101", "category": "diag",
//!              "missing": ["positive","negative"]} ],
//!   "validation_errors": [ "…human string…" ]
//! }
//! ```
//! `covered` = anchors with BOTH a positive and a negative witness. `pass` is true iff there are
//! no gaps, no validation errors, and the registry is non-empty.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub mod reference;
pub mod rules;

pub const SCHEMA: &str = "conform-coverage/1";

/// The audit rules (soundness audit R-1…R-7). Spec-fixed; the drift test cross-checks the doc.
pub const AUDIT_RULES: &[&str] = &["R-1", "R-2", "R-3", "R-4", "R-5", "R-6", "R-7"];

/// The CLI subcommand contracts, enumerated from the `delulu` dispatch (`cli.rs`). Excludes the
/// meta flags (`--help`/`--version`) and the hidden foreign-worker subcommand. The drift test
/// fences this against the dispatch source.
pub const CLI_SUBCOMMANDS: &[&str] = &[
    "check", "fmt", "test", "lsp", "keygen", "sign", "verify-sig", "publish", "add", "login", "build",
    "lock", "run", "plugin", "authority", "why", "atlas", "repl", "audit", "grants", "guard",
    "broker", "secrets", "locale", "explain",
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Category {
    Diag,
    Prim,
    Grammar,
    Audit,
    Cli,
    Rule,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Diag => "diag",
            Category::Prim => "prim",
            Category::Grammar => "grammar",
            Category::Audit => "audit",
            Category::Cli => "cli",
            Category::Rule => "rule",
        }
    }
    pub fn all() -> [Category; 6] {
        [
            Category::Diag,
            Category::Prim,
            Category::Grammar,
            Category::Audit,
            Category::Cli,
            Category::Rule,
        ]
    }
}

/// The complete anchor registry, built mechanically from the compiler source.
pub struct Registry {
    /// anchor id -> category, sorted by id.
    pub anchors: BTreeMap<String, Category>,
}

impl Registry {
    /// Build the registry from the imported compiler tables. Cannot drift: every source is a
    /// compiled-in const, not a hand-copied list.
    pub fn build() -> Registry {
        let mut anchors = BTreeMap::new();
        for c in delulu_diag::REGISTRY {
            anchors.insert(format!("ref.diag.{}", c.code), Category::Diag);
        }
        for e in delulu_check::prim_table::PRIM_TABLE {
            anchors.insert(e.anchor(), Category::Prim);
        }
        for p in delulu_syntax::grammar::GRAMMAR_PRODUCTIONS {
            anchors.insert(delulu_syntax::grammar::anchor(p), Category::Grammar);
        }
        for r in AUDIT_RULES {
            anchors.insert(format!("ref.audit.{r}"), Category::Audit);
        }
        for s in CLI_SUBCOMMANDS {
            anchors.insert(format!("ref.cli.{s}"), Category::Cli);
        }
        for r in rules::RULES {
            anchors.insert(r.anchor.to_string(), Category::Rule);
        }
        Registry { anchors }
    }

    pub fn contains(&self, anchor: &str) -> bool {
        self.anchors.contains_key(anchor)
    }
}

/// A parsed witness-map record (`[[witness]]` table in `witnesses.toml`).
#[derive(Clone, Debug)]
pub struct WitnessRecord {
    pub anchor: String,
    pub file: String,
    pub test: String,
    pub positive: bool,
}

/// Parse `witnesses.toml` — a strict, line-based subset of TOML (array-of-tables with four string
/// keys). Fail-closed: a record missing a field, an unknown `kind`, or any line we cannot classify
/// is an error, not a silent skip.
pub fn parse_witnesses(text: &str) -> Result<Vec<WitnessRecord>, Vec<String>> {
    let mut records = Vec::new();
    let mut errors = Vec::new();
    let mut cur: Option<PartialRecord> = None;

    fn flush(cur: Option<PartialRecord>, records: &mut Vec<WitnessRecord>, errors: &mut Vec<String>) {
        if let Some(p) = cur {
            match p.finish() {
                Ok(r) => records.push(r),
                Err(e) => errors.push(e),
            }
        }
    }

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[witness]]" {
            flush(cur.take(), &mut records, &mut errors);
            cur = Some(PartialRecord::default());
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            errors.push(format!("witnesses.toml:{}: cannot parse line `{line}`", lineno + 1));
            continue;
        };
        let key = k.trim();
        let val = v.trim().trim_matches('"').to_string();
        let Some(rec) = cur.as_mut() else {
            errors.push(format!("witnesses.toml:{}: key `{key}` before any [[witness]]", lineno + 1));
            continue;
        };
        match key {
            "anchor" => rec.anchor = Some(val),
            "file" => rec.file = Some(val),
            "test" => rec.test = Some(val),
            "kind" => match val.as_str() {
                "positive" => rec.positive = Some(true),
                "negative" => rec.positive = Some(false),
                other => errors
                    .push(format!("witnesses.toml:{}: kind must be positive|negative, got `{other}`", lineno + 1)),
            },
            other => errors.push(format!("witnesses.toml:{}: unknown key `{other}`", lineno + 1)),
        }
    }
    flush(cur.take(), &mut records, &mut errors);

    if errors.is_empty() {
        Ok(records)
    } else {
        Err(errors)
    }
}

#[derive(Default)]
struct PartialRecord {
    anchor: Option<String>,
    file: Option<String>,
    test: Option<String>,
    positive: Option<bool>,
}

impl PartialRecord {
    fn finish(self) -> Result<WitnessRecord, String> {
        let missing = |f: &str| format!("witnesses.toml: a [[witness]] is missing `{f}`");
        Ok(WitnessRecord {
            anchor: self.anchor.ok_or_else(|| missing("anchor"))?,
            file: self.file.ok_or_else(|| missing("file"))?,
            test: self.test.ok_or_else(|| missing("test"))?,
            positive: self.positive.ok_or_else(|| missing("kind"))?,
        })
    }
}

/// Whether a Rust source names `fn <test>` and, if so, whether it carries `#[ignore]`.
pub fn test_presence(src: &str, test: &str) -> TestPresence {
    let needle = format!("fn {test}");
    // Match `fn <test>(` or `fn <test> (` — guard against a prefix collision (`fn foo_bar` vs `foo`).
    let Some(idx) = find_fn(src, test) else { return TestPresence::Absent };
    let _ = needle;
    // Walk backwards over the contiguous attribute/comment lines directly above the fn.
    let before = &src[..idx];
    let mut ignored = false;
    for line in before.lines().rev() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("#[") || t.starts_with("//") {
            if t.starts_with("#[ignore") {
                ignored = true;
            }
            continue;
        }
        break;
    }
    if ignored {
        TestPresence::Ignored
    } else {
        TestPresence::Active
    }
}

/// Find `fn <test>` at an identifier boundary; returns the byte index of the `fn` keyword.
fn find_fn(src: &str, test: &str) -> Option<usize> {
    let pat = format!("fn {test}");
    let mut from = 0;
    while let Some(rel) = src[from..].find(&pat) {
        let idx = from + rel;
        let after = idx + pat.len();
        let next = src[after..].chars().next();
        // The char after the name must not continue an identifier (so `fn foo` != `fn foobar`).
        if !matches!(next, Some(c) if c.is_alphanumeric() || c == '_') {
            return Some(idx);
        }
        from = after;
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TestPresence {
    Active,
    Ignored,
    Absent,
}

/// Collected witnesses: anchor -> the set of witness descriptions, split by polarity.
#[derive(Default)]
pub struct Witnesses {
    pub positive: BTreeMap<String, Vec<String>>,
    pub negative: BTreeMap<String, Vec<String>>,
}

impl Witnesses {
    fn add(&mut self, anchor: &str, positive: bool, desc: String) {
        let map = if positive { &mut self.positive } else { &mut self.negative };
        map.entry(anchor.to_string()).or_default().push(desc);
    }
}

/// One coverage gap: an anchor missing a positive and/or a negative witness.
#[derive(Clone, Debug)]
pub struct Gap {
    pub anchor: String,
    pub category: Category,
    pub missing_positive: bool,
    pub missing_negative: bool,
}

/// The full coverage result.
pub struct Coverage {
    pub registry: Registry,
    pub witnesses: Witnesses,
    pub gaps: Vec<Gap>,
    pub validation_errors: Vec<String>,
}

impl Coverage {
    pub fn covered(&self) -> usize {
        self.registry.anchors.len() - self.gaps.len()
    }
    pub fn total(&self) -> usize {
        self.registry.anchors.len()
    }
    pub fn coverage_pct(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            100.0 * self.covered() as f64 / self.total() as f64
        }
    }
    /// Exit-0 only: full coverage, no validation errors, non-empty registry (never vacuous-pass).
    pub fn pass(&self) -> bool {
        !self.registry.anchors.is_empty() && self.gaps.is_empty() && self.validation_errors.is_empty()
    }
    pub fn per_category(&self) -> BTreeMap<Category, (usize, usize)> {
        let mut totals: BTreeMap<Category, (usize, usize)> = BTreeMap::new();
        for cat in &self.registry.anchors {
            totals.entry(*cat.1).or_insert((0, 0)).0 += 1;
        }
        let gap_anchors: BTreeSet<&str> = self.gaps.iter().map(|g| g.anchor.as_str()).collect();
        for (anchor, cat) in &self.registry.anchors {
            if !gap_anchors.contains(anchor.as_str()) {
                totals.entry(*cat).or_insert((0, 0)).1 += 1;
            }
        }
        totals
    }
}

/// Scan a `.delulu` source for `// anchors: …` header lines. Returns every anchor id cited.
pub fn header_anchors(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        // Only leading comment/blank lines form the header block; stop at the first real token.
        if t.is_empty() || t.starts_with("//") {
            if let Some(rest) = t.strip_prefix("//").map(str::trim).and_then(|s| s.strip_prefix("anchors:")) {
                for tok in rest.split_whitespace() {
                    out.push(tok.to_string());
                }
            }
            continue;
        }
        break;
    }
    out
}

/// The diagnostic code a reject file name encodes (`DLxxxx_*.delulu`), mirroring the conformance
/// runner's `expected_code`.
pub fn reject_file_code(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let code = name.split('_').next()?;
    if code.starts_with("DL") && code.len() == 6 && code[2..].chars().all(|c| c.is_ascii_digit()) {
        Some(code.to_string())
    } else {
        None
    }
}

fn delulu_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            if p.is_dir() {
                delulu_files(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                out.push(p);
            }
        }
    }
}

/// Run the coverage law over a repository root. Reads `tests/conformance/**`, `tests/corpus/**`,
/// and `tests/conformance/witnesses.toml`; verifies every Rust-test witness against `<root>/<file>`.
pub fn run_coverage(root: &Path) -> Coverage {
    let registry = Registry::build();
    let mut witnesses = Witnesses::default();
    let mut validation_errors = Vec::new();

    let check_anchor = |anchor: &str, source: &str, errs: &mut Vec<String>| -> bool {
        if registry.contains(anchor) {
            true
        } else {
            errs.push(format!("{source} cites unknown anchor `{anchor}`"));
            false
        }
    };

    // --- Conformance headers (accept = positive, reject = negative) + corpus (positive) ---
    for (dir, positive) in [
        ("tests/conformance/accept", true),
        ("tests/corpus", true),
        ("tests/conformance/reject", false),
    ] {
        let mut files = Vec::new();
        delulu_files(&root.join(dir), &mut files);
        for file in &files {
            let rel = file.strip_prefix(root).unwrap_or(file).display().to_string().replace('\\', "/");
            let src = match std::fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    validation_errors.push(format!("cannot read {rel}: {e}"));
                    continue;
                }
            };
            for anchor in header_anchors(&src) {
                let src_desc = format!("{rel} (header)");
                if check_anchor(&anchor, &src_desc, &mut validation_errors) {
                    witnesses.add(&anchor, positive, format!("conformance:{rel}"));
                }
            }
            // A reject file's name is an automatic negative witness for its diagnostic code.
            if !positive {
                if let Some(code) = reject_file_code(file) {
                    let anchor = format!("ref.diag.{code}");
                    if registry.contains(&anchor) {
                        witnesses.add(&anchor, false, format!("reject-file:{rel}"));
                    } else {
                        validation_errors
                            .push(format!("{rel}: file name encodes unregistered code `{code}`"));
                    }
                }
            }
        }
    }

    // --- The witness map (existing Rust tests) ---
    let wt_path = root.join("tests/conformance/witnesses.toml");
    match std::fs::read_to_string(&wt_path) {
        Ok(text) => match parse_witnesses(&text) {
            Ok(records) => {
                for r in &records {
                    if !check_anchor(&r.anchor, "witnesses.toml", &mut validation_errors) {
                        continue;
                    }
                    let src = match std::fs::read_to_string(root.join(&r.file)) {
                        Ok(s) => s,
                        Err(_) => {
                            validation_errors.push(format!(
                                "witness for `{}` names missing file `{}`",
                                r.anchor, r.file
                            ));
                            continue;
                        }
                    };
                    match test_presence(&src, &r.test) {
                        TestPresence::Active => {
                            witnesses.add(&r.anchor, r.positive, format!("rust:{}::{}", r.file, r.test));
                        }
                        TestPresence::Ignored => validation_errors.push(format!(
                            "witness for `{}` names `#[ignore]`d test `{}::{}` (inactive witness)",
                            r.anchor, r.file, r.test
                        )),
                        TestPresence::Absent => validation_errors.push(format!(
                            "witness for `{}` names nonexistent test `{}::{}`",
                            r.anchor, r.file, r.test
                        )),
                    }
                }
            }
            Err(errs) => validation_errors.extend(errs),
        },
        Err(_) => { /* no witness map yet — headers/reject files alone; not an error */ }
    }

    // --- Rule witnesses are DERIVED from the codes that enforce them ---
    // A rule is witnessed in a direction exactly when EVERY enforcing code is witnessed in that
    // direction. Deliberately strict: a rule enforced by three codes, one of which no test ever
    // produces, is a rule with an untested edge, and the reference must say so rather than round
    // up to "covered".
    for rule in rules::RULES {
        for positive in [true, false] {
            let map = if positive { &witnesses.positive } else { &witnesses.negative };
            let all_enforced = !rule.enforced_by.is_empty()
                && rule.enforced_by.iter().all(|code| {
                    map.get(&format!("ref.diag.{code}")).is_some_and(|v| !v.is_empty())
                });
            if all_enforced {
                let via = rule.enforced_by.join("+");
                witnesses.add(rule.anchor, positive, format!("via:{via}"));
            }
        }
    }

    // --- Gaps ---
    let mut gaps = Vec::new();
    for (anchor, category) in &registry.anchors {
        let has_pos = witnesses.positive.get(anchor).is_some_and(|v| !v.is_empty());
        let has_neg = witnesses.negative.get(anchor).is_some_and(|v| !v.is_empty());
        if !has_pos || !has_neg {
            gaps.push(Gap {
                anchor: anchor.clone(),
                category: *category,
                missing_positive: !has_pos,
                missing_negative: !has_neg,
            });
        }
    }

    Coverage { registry, witnesses, gaps, validation_errors }
}

/// The machine report (`conform-coverage/1`).
pub fn json_report(cov: &Coverage) -> serde_json::Value {
    use serde_json::json;
    let per_cat = cov.per_category();
    let mut by_category = serde_json::Map::new();
    for cat in Category::all() {
        let (total, covered) = per_cat.get(&cat).copied().unwrap_or((0, 0));
        by_category.insert(cat.as_str().to_string(), json!({ "anchors": total, "covered": covered }));
    }
    let gaps: Vec<serde_json::Value> = cov
        .gaps
        .iter()
        .map(|g| {
            let mut missing = Vec::new();
            if g.missing_positive {
                missing.push("positive");
            }
            if g.missing_negative {
                missing.push("negative");
            }
            json!({ "anchor": g.anchor, "category": g.category.as_str(), "missing": missing })
        })
        .collect();
    let positive_gaps = cov.gaps.iter().filter(|g| g.missing_positive).count();
    let negative_gaps = cov.gaps.iter().filter(|g| g.missing_negative).count();
    json!({
        "tool": "delulu-conform",
        "schema": SCHEMA,
        "delulu_version": env!("CARGO_PKG_VERSION"),
        "pass": cov.pass(),
        "summary": {
            "anchors": cov.total(),
            "covered": cov.covered(),
            "coverage_pct": (cov.coverage_pct() * 100.0).round() / 100.0,
            "positive_gaps": positive_gaps,
            "negative_gaps": negative_gaps,
            "validation_errors": cov.validation_errors.len(),
        },
        "by_category": by_category,
        "gaps": gaps,
        "validation_errors": cov.validation_errors,
    })
}

/// The human report. Category rollup, then validation errors (fail-closed first), then gaps.
pub fn human_report(cov: &Coverage) -> String {
    let mut s = String::new();
    s.push_str("delulu-conform --coverage (invariant 42: the conformance suite is the spec)\n");
    s.push_str(&format!(
        "anchors: {} | covered (pos+neg): {} | coverage: {:.1}%\n",
        cov.total(),
        cov.covered(),
        cov.coverage_pct()
    ));
    let per_cat = cov.per_category();
    for cat in Category::all() {
        let (total, covered) = per_cat.get(&cat).copied().unwrap_or((0, 0));
        s.push_str(&format!("  {:8} {covered:>3}/{total:<3}\n", cat.as_str()));
    }
    if !cov.validation_errors.is_empty() {
        s.push_str(&format!("\nVALIDATION ERRORS ({}): fail-closed\n", cov.validation_errors.len()));
        for e in &cov.validation_errors {
            s.push_str(&format!("  ✗ {e}\n"));
        }
    }
    if !cov.gaps.is_empty() {
        s.push_str(&format!("\nGAPS ({}):\n", cov.gaps.len()));
        for g in &cov.gaps {
            let mut miss = Vec::new();
            if g.missing_positive {
                miss.push("no accepting test");
            }
            if g.missing_negative {
                miss.push("no rejecting test");
            }
            s.push_str(&format!("  {:8} {} — {}\n", g.category.as_str(), g.anchor, miss.join(", ")));
        }
    }
    s.push('\n');
    if cov.pass() {
        s.push_str("PASS: 100% anchor coverage.\n");
    } else {
        s.push_str(&format!(
            "FAIL: {} gap(s), {} validation error(s). Coverage is not 100%.\n",
            cov.gaps.len(),
            cov.validation_errors.len()
        ));
    }
    s
}

/// Locate the repository root: the directory two levels above this crate's manifest.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[cfg(test)]
mod tests;
