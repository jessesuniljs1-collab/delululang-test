//! No message this toolchain prints carries a gap in the middle of a sentence.
//!
//! **The defect this pins (found 2026-09-25, during PS-B-02).** A long message in Rust is written as
//! a string continued with a backslash at the end of the line: the compiler drops the newline AND the
//! next line's leading spaces, so the value reads as one sentence. Written through a tool that
//! flattens the newline but keeps the spaces — a shell here-document was the one caught doing it —
//! the source becomes a single line with twenty-odd spaces in the middle, and the program prints them.
//! Nothing fails: the text compiles, the tests that match on a phrase still match, and a user reads
//! `the text to change is the grant                          request in your own invocation`. About a
//! dozen shipped messages carried it — `explain` texts, repair reasons, `run` refusals.
//!
//! **How it is checked.** Not by grepping the source, which cannot tell a string from the code or
//! comment beside it: every ordinary string literal is DECODED the way the compiler decodes it —
//! escapes, and a backslash-newline continuation skipping the next line's leading whitespace — and
//! the VALUE is searched for a run of six or more spaces between two visible characters on one line.
//!
//! **Why the source-line length is part of the rule, and what that costs.** A gap alone is ambiguous:
//! this toolchain prints about fifty deliberate columns — help tables, `key:   value` renders, TOML
//! alignment — and all of them are six-plus spaces between two words. What a flattened continuation
//! has that a column never does is the physical line it leaves behind: several source lines joined
//! into one. Measured on 2026-09-25, every real instance sat on a source line of 171 to 1,198
//! characters and every deliberate column on one of 140 or fewer, so a gap is reported when its
//! source line is longer than [`JOINED`]. The blind spot is stated rather than hidden: two SHORT lines
//! flattened into one stay under the bound and are not caught.
//!
//! One instance is not fixed here because the file is entrenched (`crates/delulu-conform/`,
//! CODEOWNERS): it is listed in [`OWNERS`], and the entry must be removed when the owner fixes it —
//! an entry that no longer matches is itself a failure, so the list cannot become an exemption.

use std::path::{Path, PathBuf};

/// A source line longer than this, carrying a gap, is several lines joined into one.
const JOINED: usize = 150;

/// Known instances in files that are not this repository's to change without the owner.
const OWNERS: &[(&str, &str)] = &[(
    "crates/delulu-conform/src/lib.rs",
    "a rejecting witness",
)];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn rust_sources() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    let crates = repo().join("crates");
    for c in std::fs::read_dir(&crates).expect("crates/").flatten() {
        walk(&c.path().join("src"), &mut out);
    }
    out.sort();
    out
}

/// Every ordinary (non-raw) string literal in `src`, as `(line it starts on, decoded value)`.
fn string_values(src: &str) -> Vec<(usize, String)> {
    string_spans(src).into_iter().map(|(start, _, v)| (start, v)).collect()
}

/// Every ordinary string literal, as `(first source line, last source line, decoded value)`.
fn string_spans(src: &str) -> Vec<(usize, usize, String)> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let (mut i, mut line) = (0usize, 1usize);
    while i < b.len() {
        let c = b[i];
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            let mut depth = 0usize;
            while i < b.len() {
                if b[i] == '/' && b.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if b[i] == '*' && b.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if b[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
        } else if c == 'r' && (b.get(i + 1) == Some(&'"') || b.get(i + 1) == Some(&'#'))
            && (i == 0 || !(b[i - 1].is_alphanumeric() || b[i - 1] == '_'))
        {
            // A raw string: no escapes, so no continuation can be flattened inside one. Skipped.
            let mut j = i + 1;
            let mut hashes = 0;
            while b.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if b.get(j) != Some(&'"') {
                i += 1;
                continue;
            }
            j += 1;
            'raw: while j < b.len() {
                if b[j] == '\n' {
                    line += 1;
                }
                if b[j] == '"' && (0..hashes).all(|k| b.get(j + 1 + k) == Some(&'#')) {
                    j += 1 + hashes;
                    break 'raw;
                }
                j += 1;
            }
            i = j;
        } else if c == '\'' {
            // A char literal (`'"'`, `'\''`, `'\u{..}'`) or a lifetime; neither may start a string.
            if b.get(i + 1) == Some(&'\\') {
                // Past the escaped character itself, which may be a quote (`'\''`).
                let mut j = i + 3;
                while j < b.len() && b[j] != '\'' {
                    j += 1;
                }
                i = j + 1;
            } else if b.get(i + 2) == Some(&'\'') {
                i += 3;
            } else {
                i += 1;
            }
        } else if c == '"' {
            let start = line;
            let mut v = String::new();
            let mut j = i + 1;
            while j < b.len() && b[j] != '"' {
                if b[j] == '\\' {
                    match b.get(j + 1) {
                        Some('\n') | Some('\r') => {
                            // The continuation: skip the newline and ALL whitespace after it.
                            j += 1;
                            while j < b.len() && b[j].is_whitespace() {
                                if b[j] == '\n' {
                                    line += 1;
                                }
                                j += 1;
                            }
                            continue;
                        }
                        Some('n') => v.push('\n'),
                        Some('t') => v.push('\t'),
                        Some('r') => v.push('\r'),
                        Some('0') => v.push('\0'),
                        Some('u') => {
                            let close = (j..b.len()).find(|&k| b[k] == '}').unwrap_or(j + 1);
                            v.push('\u{fffd}');
                            j = close + 1;
                            continue;
                        }
                        Some('x') => {
                            v.push('?');
                            j += 4;
                            continue;
                        }
                        Some(other) => v.push(*other),
                        None => {}
                    }
                    j += 2;
                } else {
                    if b[j] == '\n' {
                        line += 1;
                    }
                    v.push(b[j]);
                    j += 1;
                }
            }
            out.push((start, line, v));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// The gap, if a line of `value` has one: six or more spaces with a visible character on each side.
fn gap(value: &str) -> Option<String> {
    for l in value.split('\n') {
        let chars: Vec<char> = l.chars().collect();
        let mut k = 0;
        while k < chars.len() {
            if chars[k] == ' ' {
                let s = k;
                while k < chars.len() && chars[k] == ' ' {
                    k += 1;
                }
                if k - s >= 6 && s > 0 && k < chars.len() && chars[s - 1] != ' ' {
                    let from = s.saturating_sub(24);
                    let to = (k + 24).min(chars.len());
                    return Some(chars[from..to].iter().collect());
                }
            } else {
                k += 1;
            }
        }
    }
    None
}

/// The findings in one file's text: `(source line, excerpt)` for every literal whose value has a gap
/// on a source line longer than [`JOINED`].
fn findings(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = Vec::new();
    for (first, last, v) in string_spans(text) {
        let Some(excerpt) = gap(&v) else { continue };
        // Only the literal's OWN source lines: a long line elsewhere proves nothing about this one.
        let joined = lines[first - 1..last.min(lines.len())]
            .iter()
            .any(|l| l.len() > JOINED && l.as_bytes().windows(6).any(|w| w == b"      "));
        if joined {
            out.push((first, excerpt));
        }
    }
    out
}

#[test]
fn no_message_carries_a_flattened_continuation() {
    let mut found = Vec::new();
    let mut owners_hit = vec![false; OWNERS.len()];
    for p in rust_sources() {
        let text = std::fs::read_to_string(&p).unwrap();
        let rel = p.strip_prefix(repo()).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        for (line, excerpt) in findings(&text) {
            if let Some(ix) = OWNERS.iter().position(|(f, frag)| rel.ends_with(f) && excerpt.contains(frag)) {
                owners_hit[ix] = true;
                continue;
            }
            found.push(format!("{rel}:{line}: …{excerpt}…"));
        }
    }
    assert!(
        found.is_empty(),
        "these string literals print a run of spaces in the middle of a line — a backslash-newline \
         continuation whose newline was flattened by the tool that wrote it. Rewrite each as a real \
         continuation (a space, a backslash, the newline) or with `concat!`:\n{}",
        found.join("\n")
    );
    let stale: Vec<&str> = OWNERS.iter().zip(&owners_hit).filter(|(_, hit)| !**hit).map(|((f, _), _)| *f).collect();
    assert!(stale.is_empty(), "OWNERS entries that no longer match — the owner fixed them; remove the entry: {stale:?}");
}

/// The gate can fail: a flattened line is found, the same text as a real continuation is not, and
/// a deliberate column on a short line is not.
#[test]
fn the_gate_finds_a_flattened_continuation_and_nothing_else() {
    let pad = " ".repeat(26);
    let flattened = format!(
        "fn f() {{\n    let m = \"a message long enough that it was written across several lines by its author,{pad}and then one tool joined them into a single physical line\";\n}}\n"
    );
    assert_eq!(findings(&flattened).len(), 1, "the flattened shape must be found");
    let real = "fn f() {\n    let m = \"a message long enough that it was written across several lines by its author, \\\n                 and then nothing joined them\";\n}\n";
    assert!(findings(real).is_empty(), "a real continuation must pass");
    let column = "fn f() {\n    println!(\"  effects:      {}\", e);\n}\n";
    assert!(findings(column).is_empty(), "a deliberate column on a short line must pass");
}

/// The decoder itself, on the shapes that matter: a real continuation leaves no gap, a flattened one
/// does, and a gap in a comment or beside a literal is not a string at all.
#[test]
fn the_decoder_reads_literals_the_way_the_compiler_does() {
    let real = "let s = \"one \\\n        two\";";
    assert_eq!(string_values(real), vec![(1, "one two".to_string())]);
    assert_eq!(gap(&string_values(real)[0].1), None);
    let flat = "let s = \"one         two\";";
    assert!(gap(&string_values(flat)[0].1).is_some());
    let comment = "let s = \"x\";          // aligned comment\n// a \"quoted          gap\" in a comment\n";
    assert_eq!(string_values(comment), vec![(1, "x".to_string())]);
    let chars = "let q = '\"'; let t = \"after a char literal\";";
    assert_eq!(string_values(chars), vec![(1, "after a char literal".to_string())]);
    let raw = "let r = r#\"raw         gap\"#; let s = \"next\";";
    assert_eq!(string_values(raw), vec![(1, "next".to_string())]);
}
