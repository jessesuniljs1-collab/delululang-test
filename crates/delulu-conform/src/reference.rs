//! The generated half of the language reference (Stage 9b, spec §2.1).
//!
//! Grammar productions, tokens, the primitive table, the diagnostics registry, the audit rules and
//! the CLI contracts are **extracted from the compiler source**, not hand-maintained. A
//! hand-written reference table drifts from the implementation within one release; mechanizing it
//! away is the whole point. `delulu-conform --reference` writes these chapters;
//! `--check-reference` fails if the committed files differ from what the current source produces,
//! so a change to the compiler that would stale the docs breaks the build instead.
//!
//! Each generated chapter also carries its **live coverage status** from [`crate::run_coverage`]:
//! invariant 42 says "a behavior not covered by a test is not stable, and the reference says so
//! per item," so the reference reports its own gaps rather than implying completeness.

use std::collections::BTreeMap;
use std::path::Path;

use crate::{Category, Coverage};

/// The banner every generated chapter opens with. Editing a generated file by hand is a mistake
/// the drift gate will catch; the banner says so where someone would see it first.
fn banner(source_of_truth: &str) -> String {
    format!(
        "<!-- GENERATED FILE — DO NOT EDIT BY HAND.\n     \
         Written by `delulu-conform --reference` from {source_of_truth}.\n     \
         `delulu-conform --check-reference` fails the build if this file drifts from the source. -->\n\n"
    )
}

/// Per-anchor coverage status, rendered as a short cell.
fn status(cov: &Coverage, anchor: &str) -> &'static str {
    match cov.gaps.iter().find(|g| g.anchor == anchor) {
        None => "covered",
        Some(g) if g.missing_positive && g.missing_negative => "**uncovered**",
        Some(g) if g.missing_negative => "accepting only",
        Some(_) => "rejecting only",
    }
}

fn coverage_line(cov: &Coverage, cat: Category) -> String {
    let per = cov.per_category();
    let (total, covered) = per.get(&cat).copied().unwrap_or((0, 0));
    let pct = if total == 0 { 0.0 } else { 100.0 * covered as f64 / total as f64 };
    format!(
        "> **Coverage (invariant 42):** {covered} of {total} anchors in this chapter have both an \
         accepting and a rejecting conformance witness ({pct:.1}%). Items marked otherwise are \
         **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.\n\n"
    )
}

/// Every generated chapter, as (repo-relative path, content).
pub fn generate(root: &Path) -> BTreeMap<String, String> {
    let cov = crate::run_coverage(root);
    let mut out = BTreeMap::new();
    out.insert("docs/reference/tokens.md".into(), tokens_chapter());
    out.insert("docs/reference/grammar.md".into(), grammar_chapter(&cov));
    out.insert("docs/reference/primitives.md".into(), primitives_chapter(&cov));
    out.insert("docs/reference/diagnostics.md".into(), diagnostics_chapter(&cov));
    out.insert("docs/reference/audit-rules.md".into(), audit_chapter(&cov));
    out.insert("docs/reference/cli.md".into(), cli_chapter(&cov));
    out.insert("docs/reference/coverage.md".into(), coverage_chapter(&cov));
    for (num, title) in crate::rules::CHAPTERS {
        let slug = num.replace('.', "-");
        out.insert(format!("docs/reference/semantics-{slug}.md"), semantics_chapter(&cov, num, title));
    }
    out.insert("docs/reference/README.md".into(), index_chapter(&cov));
    out
}

/// One Constitution §5 subsection: its normative rules, each with enforcement and coverage.
fn semantics_chapter(cov: &Coverage, num: &str, title: &str) -> String {
    let mut s = banner("`delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry)");
    s.push_str(&format!("# §{num} {title}\n\n"));
    s.push_str("Each statement below is normative and carries a stable anchor id that conformance \
                metadata can cite. **Enforced by** names the diagnostics that make the rule bite; \
                a rule counts as covered only when *every* one of them has both an accepting and a \
                rejecting witness.\n\n");
    for r in crate::rules::RULES.iter().filter(|r| r.chapter == num) {
        s.push_str(&format!("## `{}`\n\n{}\n\n", r.anchor, r.statement));
        let codes: Vec<String> = r.enforced_by.iter().map(|c| format!("`{c}`")).collect();
        s.push_str(&format!("- **Enforced by:** {}\n", codes.join(", ")));
        s.push_str(&format!("- **Coverage:** {}\n", status(cov, r.anchor)));
        if !r.note.is_empty() {
            s.push_str(&format!("- **Note:** {}\n", r.note));
        }
        s.push('\n');
    }
    s
}

/// The reference index.
fn index_chapter(cov: &Coverage) -> String {
    let mut s = banner("the chapter list and a live coverage run");
    s.push_str("# The DeluluLang language reference\n\nThis reference is **generated in part**: \
                the grammar, tokens, primitive table, diagnostics registry, audit rules, CLI \
                contracts and normative rule statements are all extracted from the compiler source \
                by `delulu-conform --reference`. A change to the implementation that would stale a \
                chapter fails `--check-reference` in CI instead of quietly making the docs wrong.\n\n\
                Every normative statement carries a stable anchor id (`ref.rule.*`, `ref.diag.*`, \
                `ref.prim.*`, `ref.grammar.*`, `ref.audit.*`, `ref.cli.*`), and each anchor's \
                conformance status is reported next to it — invariant 42: *a behavior not covered \
                by a test is not stable, and the reference says so per item.*\n\n");
    s.push_str(&format!(
        "**Coverage today: {} of {} anchors ({:.1}%).** See [coverage.md](coverage.md).\n\n",
        cov.covered(),
        cov.total(),
        cov.coverage_pct()
    ));
    s.push_str("## Semantics (Constitution §5)\n\n");
    for (num, title) in crate::rules::CHAPTERS {
        let slug = num.replace('.', "-");
        let n = crate::rules::RULES.iter().filter(|r| r.chapter == *num).count();
        s.push_str(&format!("- [§{num} {title}](semantics-{slug}.md) — {n} rule(s)\n"));
    }
    s.push_str("\n## Extracted from the implementation\n\n\
                - [Tokens](tokens.md)\n\
                - [Grammar productions](grammar.md)\n\
                - [The primitive table](primitives.md)\n\
                - [Diagnostics](diagnostics.md)\n\
                - [The audit rules](audit-rules.md)\n\
                - [CLI contracts](cli.md)\n\
                - [Conformance coverage](coverage.md)\n\n\
                ## Regenerating\n\n\
                ```\n\
                cargo run -p delulu-conform -- --reference        # rewrite these chapters\n\
                cargo run -p delulu-conform -- --check-reference  # fail if they are stale\n\
                ```\n");
    s
}

fn tokens_chapter() -> String {
    let mut s = banner("`delulu_syntax::token` (the `TokenKind` enum and its keyword table)");
    s.push_str("# Tokens\n\nThe complete lexical surface. Every row is a `TokenKind` variant; the\n\
                index is fenced against the enum by a drift guard in `token.rs`, so a token cannot\n\
                be added, renamed or removed without this chapter changing with it.\n\n");
    s.push_str("| Token | Lexeme | Description |\n|---|---|---|\n");
    for t in delulu_syntax::token::token_index() {
        let lex = match t.lexeme {
            Some(l) => format!("`{l}`"),
            None => "*(literal)*".to_string(),
        };
        s.push_str(&format!("| `{}` | {} | {} |\n", t.variant, lex, t.describe.replace('|', "\\|")));
    }
    s.push_str("\n## Reserved words\n\nLexed as identifiers and refused at declaration sites \
                (`DL0106`), so a member name like `root.secret(…)` stays legal while `secret` \
                cannot be *declared*:\n\n");
    let reserved: Vec<String> =
        delulu_syntax::token::RESERVED.iter().map(|r| format!("`{r}`")).collect();
    s.push_str(&reserved.join(", "));
    s.push('\n');
    s
}

fn grammar_chapter(cov: &Coverage) -> String {
    let mut s = banner("`delulu_syntax::grammar::GRAMMAR_PRODUCTIONS` (fenced against `parser.rs`)");
    s.push_str("# Grammar productions\n\nThe parser is hand-written recursive descent, so this is \
                the index of its productions: one row per user-facing production, each backed by a \
                `parse_<name>` function that a drift guard verifies exists.\n\n");
    s.push_str(&coverage_line(cov, Category::Grammar));
    s.push_str("| Production | Anchor | Coverage |\n|---|---|---|\n");
    for p in delulu_syntax::grammar::GRAMMAR_PRODUCTIONS {
        let anchor = delulu_syntax::grammar::anchor(p);
        s.push_str(&format!("| `{p}` | `{anchor}` | {} |\n", status(cov, &anchor)));
    }
    s
}

fn primitives_chapter(cov: &Coverage) -> String {
    let mut s = banner("`delulu_check::prim_table::PRIM_TABLE` (fenced against `check::method_sig`)");
    s.push_str("# The primitive table\n\nEvery primitive operation, by receiver. **Arity is \
                normative**: the checker refuses a call carrying more arguments than the row \
                states (`DL0403`), and the column is proven in both directions against that gate.\n\n\
                Capability operations are the only source of primitive effects (T-CapOp), which is \
                why this table is the checker's single source of truth rather than prose.\n\n");
    s.push_str(&coverage_line(cov, Category::Prim));
    let mut current = "";
    for e in delulu_check::prim_table::PRIM_TABLE {
        if e.receiver != current {
            current = e.receiver;
            s.push_str(&format!("\n## `{current}`\n\n| Method | Arity | Anchor | Coverage |\n|---|---|---|---|\n"));
        }
        let a = e.anchor();
        s.push_str(&format!("| `{}` | {} | `{a}` | {} |\n", e.method, e.arity, status(cov, &a)));
    }
    s
}

fn diagnostics_chapter(cov: &Coverage) -> String {
    let mut s = banner("`delulu_diag::REGISTRY` (the diagnostic code registry)");
    s.push_str("# Diagnostics\n\nEvery diagnostic code, with its title and its conformance status.\n\n\
                A code's **rejecting** witness proves it fires when it should; its **accepting** \
                witness proves it does *not* fire on valid input. A diagnostic that always fires is \
                as broken as one that never fires, so invariant 42 requires both.\n\n");
    s.push_str(&coverage_line(cov, Category::Diag));
    s.push_str("Codes marked other than `covered` are **not stable**: the classification of every \
                such code — shadowed by a more general code, producible but untested, or \
                unproducible by construction — is in `docs/design/STAGE9_BUILD_ORDER.md` D10.\n\n");
    s.push_str("| Code | Title | Anchor | Coverage |\n|---|---|---|---|\n");
    for c in delulu_diag::REGISTRY {
        let a = format!("ref.diag.{}", c.code);
        s.push_str(&format!(
            "| `{}` | {} | `{a}` | {} |\n",
            c.code,
            c.title.replace('|', "\\|"),
            status(cov, &a)
        ));
    }

    // The other half of the question. A reader who meets a code in a specification or an old
    // build order and looks it up here used to find nothing, which reads identically to a typo.
    s.push_str("\n## Codes this compiler cannot emit\n\n\
                These `DLxxxx` are named somewhere in this repository but are **not** in the table \
                above. Each carries a recorded disposition, so a code you cannot find is an \
                answered question rather than a dead end — `delulu explain <code>` prints the \
                reason, and a code in neither table is reported by the Survey as unexplained.\n\n\
                `specified-not-implemented` marks a **real gap**: a specification names the code \
                and nothing raises it. Those are kept visible rather than closed by inventing a \
                code or editing the specification to match.\n\n");
    s.push_str("| Code | Disposition | Why |\n|---|---|---|\n");
    for u in delulu_diag::UNALLOCATED {
        s.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            u.code,
            u.disposition.word(),
            u.why.replace('|', "\\|")
        ));
    }
    s
}

fn audit_chapter(cov: &Coverage) -> String {
    let mut s = banner("`delulu_conform::AUDIT_RULES` (cross-checked against `SOUNDNESS_AUDIT.md`)");
    s.push_str("# The audit rules\n\nThe soundness audit's rules R-1…R-7 — the properties the \
                laundering suite exists to attack. Each rule's rejecting witness is an exploit \
                attempt that must fail; its accepting witness is the legitimate program that must \
                still pass.\n\nThe normative statement of each rule lives in \
                `docs/design/SOUNDNESS_AUDIT.md`; this chapter records enforcement and coverage.\n\n");
    s.push_str(&coverage_line(cov, Category::Audit));
    s.push_str("| Rule | Anchor | Coverage |\n|---|---|---|\n");
    for r in crate::AUDIT_RULES {
        let a = format!("ref.audit.{r}");
        s.push_str(&format!("| `{r}` | `{a}` | {} |\n", status(cov, &a)));
    }
    s
}

fn cli_chapter(cov: &Coverage) -> String {
    let mut s = banner("`delulu_conform::CLI_SUBCOMMANDS` (fenced against the `delulu` dispatch)");
    s.push_str("# CLI contracts\n\nEvery subcommand of the `delulu` binary. Exit codes and the \
                `--json` envelopes are part of the stability contract from 1.0 (invariant 43).\n\n\
                A subcommand's **accepting** witness is a valid invocation that succeeds; its \
                **rejecting** witness is an invalid one that must refuse with a nonzero exit and an \
                honest message — never a silent success, never a panic.\n\n");
    s.push_str(&coverage_line(cov, Category::Cli));
    s.push_str("| Subcommand | Anchor | Coverage |\n|---|---|---|\n");
    for c in crate::CLI_SUBCOMMANDS {
        let a = format!("ref.cli.{c}");
        s.push_str(&format!("| `delulu {c}` | `{a}` | {} |\n", status(cov, &a)));
    }
    s
}

fn coverage_chapter(cov: &Coverage) -> String {
    let mut s = banner("a live `delulu-conform --coverage` run");
    s.push_str("# Conformance coverage\n\nInvariant 42: *the conformance suite is the \
                specification's executable form.* This chapter is the machine's own account of how \
                much of the reference is executable today. It is generated from a live coverage \
                run — it cannot flatter itself.\n\n");
    s.push_str(&format!(
        "**{} of {} anchors ({:.1}%)** carry both an accepting and a rejecting witness.\n\n",
        cov.covered(),
        cov.total(),
        cov.coverage_pct()
    ));
    s.push_str("| Chapter | Anchors | Covered |\n|---|---|---|\n");
    let per = cov.per_category();
    for cat in Category::all() {
        let (total, covered) = per.get(&cat).copied().unwrap_or((0, 0));
        s.push_str(&format!("| {} | {total} | {covered} |\n", cat.as_str()));
    }
    s.push_str(
        "\nA `rule` anchor is covered only when *every* diagnostic that enforces it is covered in \
         both directions, so this row is the strictest of the six.\n",
    );
    if !cov.gaps.is_empty() {
        s.push_str(&format!(
            "\n## The {} open anchors\n\nEach is classified in `docs/design/STAGE9_BUILD_ORDER.md` \
             D10. Release criterion 1 requires this list to be empty.\n\n| Anchor | Missing |\n|---|---|\n",
            cov.gaps.len()
        ));
        for g in &cov.gaps {
            let mut m = Vec::new();
            if g.missing_positive {
                m.push("accepting");
            }
            if g.missing_negative {
                m.push("rejecting");
            }
            s.push_str(&format!("| `{}` | {} |\n", g.anchor, m.join(", ")));
        }
    }
    s
}

/// Write the generated chapters. Returns the paths written.
pub fn write(root: &Path) -> std::io::Result<Vec<String>> {
    let files = generate(root);
    std::fs::create_dir_all(root.join("docs/reference"))?;
    let mut written = Vec::new();
    for (rel, content) in &files {
        std::fs::write(root.join(rel), content)?;
        written.push(rel.clone());
    }
    Ok(written)
}

/// Compare the committed chapters to what the current source produces. Returns the paths that
/// differ (or are missing) — empty means the reference is in sync.
pub fn drift(root: &Path) -> Vec<String> {
    let mut stale = Vec::new();
    for (rel, expected) in generate(root) {
        match std::fs::read_to_string(root.join(&rel)) {
            Ok(actual) if actual.replace("\r\n", "\n") == expected => {}
            Ok(_) => stale.push(format!("{rel} (differs from the source)")),
            Err(_) => stale.push(format!("{rel} (missing)")),
        }
    }
    stale
}
