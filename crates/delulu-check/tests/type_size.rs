//! A type whose variables share their bindings is checked in time near its size as written, not as written out
//! (RW 2.3, D-V2-179). `let a1 = Pair { l: a0, r: a0 }` … `aN` is N small types through their variables and 2^(N+1)-1
//! parts written out; the checker wrote it out (the DIR's side table, a field read, `==`), and unified and searched it
//! (the occurs check, `==`'s equality walk) as the tree, so 22 such lines took 5.8 s to check and each line doubled it.
//!
//! Now a type is written out only within 100,000 parts (`TYPE_PARTS_LIMIT`), and the large ones in a module within
//! 2,000,000 in all (`TYPE_PARTS_TOTAL`); past either the program is refused with DL0211 at the expression, and the walks
//! that followed bindings follow each once. Every program here is checked on a thread and given a minute: before the
//! change, the smallest refused one (16 lines) took 0.15 s in a release build and each further line doubled it (22
//! lines: 5.8 s; 30 lines, by that rate, about 25 minutes).

use std::sync::mpsc;
use std::time::Duration;

const PAIR: &str = "module m\n\ntype Pair[L, R] {\n  l: L,\n  r: R,\n}\n\nfn main(root: Root) ! {} {\n";

/// A program pairing `a` (and, with `twin`, a second chain `b` built from `twin`) with itself `depth` times, then `tail`.
fn pairs(depth: usize, twin: Option<&str>, tail: &str) -> String {
    let mut src = String::from(PAIR);
    src.push_str("  let a0 = 1\n");
    if let Some(b0) = twin {
        src.push_str(&format!("  let b0 = {b0}\n"));
    }
    for i in 1..=depth {
        src.push_str(&format!("  let a{i} = Pair {{ l: a{}, r: a{} }}\n", i - 1, i - 1));
        if twin.is_some() {
            src.push_str(&format!("  let b{i} = Pair {{ l: b{}, r: b{} }}\n", i - 1, i - 1));
        }
    }
    src.push_str(tail);
    src.push_str("}\n");
    src
}

/// Each diagnostic's code and the line (1-based) its first span starts on, the program checked on its own thread within
/// a minute.
fn check_within_a_minute(src: String) -> Vec<(&'static str, Option<usize>)> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let checked = delulu_check::check_source(0, &src);
        // DL0414 (a binding never read) is left out: these programs build the chain to time the type-size bound, and its
        // last link, or the comparison made of it, is never read. Every other diagnostic counts.
        let found = checked
            .diagnostics
            .iter()
            .filter(|d| d.code != "DL0414")
            .map(|d| (d.code, d.spans.first().map(|s| src[..s.span.start as usize].matches('\n').count() + 1)))
            .collect::<Vec<_>>();
        let _ = tx.send(found);
    });
    rx.recv_timeout(Duration::from_secs(60)).expect("the check must finish within a minute")
}

/// The line `let a{i} = …` is on.
fn line_of_a(src: &str, i: usize) -> usize {
    let at = src.find(&format!("  let a{i} = ")).expect("the line");
    src[..at].matches('\n').count() + 1
}

/// Paired with itself at every step, a value's type passes 100,000 parts at `a16` (131,071) and the program is refused
/// there, once, however many lines follow. `a15` (65,535 parts) checks clean.
#[test]
fn a_value_paired_with_itself_is_refused_where_its_type_passes_the_limit() {
    for depth in [16, 30, 60] {
        let src = pairs(depth, None, "");
        let found = check_within_a_minute(src.clone());
        assert_eq!(found, vec![("DL0211", Some(line_of_a(&src, 16)))], "depth {depth}");
    }
    assert_eq!(check_within_a_minute(pairs(15, None, "")), vec![], "65,535 parts are within the limit");
}

/// Reading a field of a type too large to write out is refused, not written out at every read.
#[test]
fn a_field_read_of_a_type_too_large_is_refused() {
    let found = check_within_a_minute(pairs(40, None, "  let x = a40.l.l.r\n"));
    assert!(found.iter().all(|(c, _)| *c == "DL0211") && !found.is_empty(), "{found:?}");
}

/// `==` of two chains built alike is unified and asked for equality without walking either as its tree: at the largest
/// depth under the limit it checks (the equality walk wrote the type out again at every part it visited); past it, it is
/// refused (unification followed both chains as trees); and when the chains differ at their root (`1` and `"x"`), the
/// mismatch is still found, at every depth.
#[test]
fn two_chains_compared_are_unified_once_per_pair_of_variables() {
    assert_eq!(check_within_a_minute(pairs(15, Some("1"), "  let same = a15 == b15\n")), vec![]);
    let found = check_within_a_minute(pairs(40, Some("1"), "  let same = a40 == b40\n"));
    assert!(found.iter().any(|(c, _)| *c == "DL0211"), "{found:?}");
    for depth in [12, 40] {
        let tail = format!("  let same = a{depth} == b{depth}\n");
        let found = check_within_a_minute(pairs(depth, Some("\"x\""), &tail));
        assert!(found.iter().any(|(c, _)| *c == "DL0401"), "depth {depth}: Int against Str must be a mismatch: {found:?}");
    }
}

/// Many references to one type within the limit are bounded by the module's total: 200 references to a 32,767-part
/// type are refused, 40 references to a 8,191-part type check.
#[test]
fn many_references_to_a_large_type_are_bounded_by_the_total() {
    let many = |depth: usize, n: usize| format!("  let xs = [{}]\n", vec![format!("a{depth}"); n].join(", "));
    let found = check_within_a_minute(pairs(14, None, &many(14, 200)));
    assert!(found.iter().any(|(c, _)| *c == "DL0211"), "{found:?}");
    assert_eq!(check_within_a_minute(pairs(12, None, &many(12, 40))), vec![]);
}
