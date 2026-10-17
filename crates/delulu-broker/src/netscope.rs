//! P9-05 (a), D-V2-138 — a `net` scope entry is `HOST` or `HOST/PATTERN`, and this is the one reading of it: the grant
//! parser, the runtime's allowlist, the egress client's check of every hop and the broker's use check all call it, as
//! they call [`host_matches`] (NET-WILDCARD-1: two readers of one grant had disagreed before).
//!
//! A PATTERN is `/`-separated segments, each a literal, `*` (exactly one non-empty segment) or `**` (the rest of the
//! path, zero segments included — only as the last segment). Containment is decided by the syntax alone. A pattern is
//! refused when a spelling in it could be read two ways (an empty, `.` or `..` segment, a `%`, a `;`, a `*` inside a
//! literal). A request's path is judged only where an entry names a pattern, and there a spelling that a URL parser or a
//! server would rewrite is refused, never matched: a `.` or `..` segment, an empty segment before the last, a `;`, or an
//! escape of `/`, `\`, `.`, `%`, `;` or a control character.

use crate::validate::host_matches;

/// The longest path pattern accepted, in bytes and in segments — bounded, as every parser in the broker is.
const MAX_PATTERN_BYTES: usize = 1024;
const MAX_SEGMENTS: usize = 64;

/// One segment of a path pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    /// The segment exactly, compared byte for byte (a path is case-sensitive).
    Literal(String),
    /// `*`: exactly one segment, never an empty one.
    One,
    /// `**`: every remaining segment, none included. Only ever the last.
    Rest,
}

/// Split an entry at its first `/`: the host pattern, and the path pattern (with its leading `/`) when one is named.
pub fn split(entry: &str) -> (&str, Option<&str>) {
    match entry.find('/') {
        Some(i) => (&entry[..i], Some(&entry[i..])),
        None => (entry, None),
    }
}

/// Parse a path pattern — `/` then segments — refusing every spelling that could be read two ways.
pub fn parse_pattern(path: &str) -> Result<Vec<Segment>, String> {
    let Some(rest) = path.strip_prefix('/') else {
        return Err("a path pattern begins with `/`".to_string());
    };
    if path.len() > MAX_PATTERN_BYTES {
        return Err(format!("the path pattern is longer than {MAX_PATTERN_BYTES} bytes"));
    }
    if rest.is_empty() {
        return Err("the path is empty — write HOST for every path, or HOST/PREFIX/** for the paths under PREFIX".to_string());
    }
    let parts: Vec<&str> = rest.split('/').collect();
    if parts.len() > MAX_SEGMENTS {
        return Err(format!("the path pattern has more than {MAX_SEGMENTS} segments"));
    }
    let mut segments = Vec::with_capacity(parts.len());
    for (i, s) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        let segment = match *s {
            "" => return Err("an empty segment (`//`, or a trailing `/`) — a server may merge or keep it".to_string()),
            "**" if last => Segment::Rest,
            "**" => return Err("`**` stands only as the last segment".to_string()),
            "*" => Segment::One,
            "." | ".." => return Err(format!("a `{s}` segment, which a URL parser removes")),
            _ if s.contains('*') => {
                return Err(format!("`{s}`: `*` stands alone as a whole segment (`*` one segment, `**` the rest), never inside one"));
            }
            _ => {
                if let Some(c) = s.chars().find(|c| !literal_char(*c)) {
                    return Err(format!(
                        "`{}` in a path segment — a pattern names plain segments: letters, digits and `-._~!$&'()+,=:@`",
                        c.escape_debug()
                    ));
                }
                Segment::Literal(s.to_string())
            }
        };
        segments.push(segment);
    }
    Ok(segments)
}

/// RFC 3986's `pchar` without `%` (an escape would be a second spelling of a literal), `;` (a server's path
/// parameters) and `*` (the pattern's own syntax).
fn literal_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "-._~!$&'()+,=:@".contains(c)
}

/// Why a request's path cannot be judged against a pattern — `None` when it is plain. Asked only where an entry names
/// a pattern: a host-only grant reaches every path, so no spelling of one can widen it.
pub fn ambiguity(path: &str) -> Option<&'static str> {
    if path.contains(';') {
        return Some("a `;` in the path, which some servers read as the start of path parameters (`..;/`)");
    }
    let segments: Vec<&str> = request_segments(path);
    for (i, s) in segments.iter().enumerate() {
        if *s == "." || *s == ".." {
            return Some("a `.` or `..` segment in the path, which a URL parser removes");
        }
        if s.is_empty() && i + 1 != segments.len() {
            return Some("an empty segment (`//`) in the path, which some servers merge");
        }
    }
    let b = path.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = |x: u8| (x as char).to_digit(16);
            let (Some(hi), Some(lo)) = (b.get(i + 1).and_then(|x| hex(*x)), b.get(i + 2).and_then(|x| hex(*x))) else {
                return Some("a `%` in the path that is not a two-digit escape");
            };
            let byte = (hi * 16 + lo) as u8;
            if matches!(byte, b'/' | b'\\' | b'.' | b'%' | b';') || byte < 0x20 || byte == 0x7f {
                return Some("an escaped `/`, `\\`, `.`, `%`, `;` or control character in the path, which a server may decode into another path");
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    None
}

/// A request path's segments: none for an empty path or `/`; a trailing `/` is a last, empty segment.
fn request_segments(path: &str) -> Vec<&str> {
    match path.strip_prefix('/') {
        None | Some("") => Vec::new(),
        Some(rest) => rest.split('/').collect(),
    }
}

/// Does the request path `path` (already judged plain by [`ambiguity`]) fall under `pattern`?
pub fn path_matches(pattern: &[Segment], path: &str) -> bool {
    let segments = request_segments(path);
    for (i, p) in pattern.iter().enumerate() {
        match p {
            Segment::Rest => return true,
            _ if i >= segments.len() => return false,
            Segment::One if segments[i].is_empty() => return false,
            Segment::One => {}
            Segment::Literal(l) if segments[i] != l => return false,
            Segment::Literal(_) => {}
        }
    }
    segments.len() == pattern.len()
}

/// One entry's verdict on a request, worst last: the order [`judge_all`] reports in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Allow,
    /// The entry names another host.
    OtherHost,
    /// The entry names this host and a pattern the path is outside of (or a pattern that does not parse — an entry
    /// that reached here unparsed, a manifest's say, matches nothing).
    OutsidePath,
    /// The entry names this host and a pattern, and the path is spelled so that it could be read two ways.
    Ambiguous(&'static str),
}

/// `entry`'s verdict on a request to `host` for `path` (the URL's path, without its query or fragment).
pub fn judge(entry: &str, host: &str, path: &str) -> Verdict {
    let (h, p) = split(entry);
    if !host_matches(host, h) {
        return Verdict::OtherHost;
    }
    let Some(p) = p else { return Verdict::Allow };
    let Ok(pattern) = parse_pattern(p) else { return Verdict::OutsidePath };
    if let Some(why) = ambiguity(path) {
        return Verdict::Ambiguous(why);
    }
    if path_matches(&pattern, path) {
        Verdict::Allow
    } else {
        Verdict::OutsidePath
    }
}

/// The verdict of a whole allowlist: `Allow` when any entry allows, else the most telling refusal.
pub fn judge_all<'a>(entries: impl IntoIterator<Item = &'a String>, host: &str, path: &str) -> Verdict {
    let mut worst = Verdict::OtherHost;
    for e in entries {
        match judge(e, host, path) {
            Verdict::Allow => return Verdict::Allow,
            v => worst = worst.max(v),
        }
    }
    worst
}

/// The host part of an entry — what the broker's use check compares (the custody's `Net` op carries the host; the
/// path is the egress client's to judge, on every hop, under every custody).
pub fn host_of(entry: &str) -> &str {
    split(entry).0
}

/// Refuse an entry the grammar does not accept: an empty host, or a path pattern that does not parse. Every parser of a
/// `net` entry calls it — `--grant net=`, `grants delegate --net`, a lease's `--net` — so none stores what none can read.
pub fn check_entry(entry: &str) -> Result<(), String> {
    let (host, path) = split(entry);
    if host.is_empty() {
        return Err("it names no host (write HOST, or HOST/PREFIX/** for the paths under PREFIX)".to_string());
    }
    match path {
        Some(p) => parse_pattern(p).map(|_| ()).map_err(|why| format!("its path is refused: {why}")),
        None => Ok(()),
    }
}

// ----- the order (P9-05 (b), D-V2-139) -------------------------------------------------------------

/// Is every path `child` matches one `parent` matches? Decided by the syntax, segment by segment: a literal under an
/// equal literal or `*`, `*` under `*`, and everything that remains — none included — under a final `**`. Exact for these
/// patterns (an exhaustive test compares it with the paths each matches): a `*` is never under a literal (it matches
/// other segments), and a `**` is under nothing but a `**` (it matches the empty rest, and any length).
pub fn pattern_within(child: &[Segment], parent: &[Segment]) -> bool {
    for (i, p) in parent.iter().enumerate() {
        if *p == Segment::Rest {
            return true;
        }
        let Some(c) = child.get(i) else { return false };
        let ok = match (c, p) {
            (Segment::Literal(a), Segment::Literal(b)) => a == b,
            (Segment::Literal(_) | Segment::One, Segment::One) => true,
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    child.len() == parent.len()
}

/// The pattern matching exactly the paths both match, or `None` when no path does: segment by segment the narrower of
/// the two (two different literals meet in nothing), and where one side's `**` begins, the other side's rest.
pub fn pattern_meet(a: &[Segment], b: &[Segment]) -> Option<Vec<Segment>> {
    let mut out = Vec::new();
    let mut i = 0;
    loop {
        match (a.get(i), b.get(i)) {
            (Some(Segment::Rest), _) => {
                out.extend(b[i..].iter().cloned());
                return Some(out);
            }
            (_, Some(Segment::Rest)) => {
                out.extend(a[i..].iter().cloned());
                return Some(out);
            }
            (None, None) => return Some(out),
            (None, Some(_)) | (Some(_), None) => return None,
            (Some(x), Some(y)) => out.push(match (x, y) {
                (Segment::One, Segment::One) => Segment::One,
                (Segment::One, Segment::Literal(l)) | (Segment::Literal(l), Segment::One) => Segment::Literal(l.clone()),
                (Segment::Literal(p), Segment::Literal(q)) if p == q => Segment::Literal(p.clone()),
                _ => return None,
            }),
        }
        i += 1;
    }
}

/// A pattern as it is written after its host.
pub fn render(pattern: &[Segment]) -> String {
    let mut s = String::new();
    for seg in pattern {
        s.push('/');
        s.push_str(match seg {
            Segment::Literal(l) => l,
            Segment::One => "*",
            Segment::Rest => "**",
        });
    }
    s
}

/// Is the entry `child` within `parent`? The hosts must be the same string — a `*.SUFFIX` pattern does not contain a host
/// under it here, as before (conservative: the order never calls more within than the matcher allows) — and then: a
/// parent with no path reaches every path; a child with none is within only such a parent; two patterns by
/// [`pattern_within`]. An entry whose pattern does not parse is within only an identical entry.
pub fn entry_within(child: &str, parent: &str) -> bool {
    if child == parent {
        return true;
    }
    let ((ch, cp), (ph, pp)) = (split(child), split(parent));
    if ch != ph {
        return false;
    }
    match (cp, pp) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(c), Some(p)) => match (parse_pattern(c), parse_pattern(p)) {
            (Ok(c), Ok(p)) => pattern_within(&c, &p),
            _ => false,
        },
    }
}

/// The entry reaching exactly what both reach, or `None` when they share no path (or no host).
pub fn entry_meet(a: &str, b: &str) -> Option<String> {
    if a == b {
        return Some(a.to_string());
    }
    let ((ah, ap), (bh, bp)) = (split(a), split(b));
    if ah != bh {
        return None;
    }
    match (ap, bp) {
        (None, _) => Some(b.to_string()),
        (_, None) => Some(a.to_string()),
        (Some(x), Some(y)) => {
            let (Ok(px), Ok(py)) = (parse_pattern(x), parse_pattern(y)) else { return None };
            pattern_meet(&px, &py).map(|m| format!("{ah}{}", render(&m)))
        }
    }
}

/// `child ⊑ parent` for a `net` set: every child entry within some parent entry. A preorder, not an order — `{h/a/**}`
/// and `{h/a/**, h/a/b}` are each within the other (as P18 found of path sets; the canonical form is an antichain).
pub fn set_within<'a>(child: impl IntoIterator<Item = &'a String>, parent: &std::collections::BTreeSet<String>) -> bool {
    child.into_iter().all(|c| parent.iter().any(|p| entry_within(c, p)))
}

/// A `net` set's canonical representative (P17-F3's rule, kept for this dimension): no entry within another, and of two
/// entries each within the other the smaller spelling — so two sets each within the other canonicalize to one value and
/// hash alike, as path sets do (`path::canonicalize_set`).
pub fn canonicalize_set(net: &std::collections::BTreeSet<String>) -> std::collections::BTreeSet<String> {
    net.iter()
        .filter(|x| !net.iter().any(|y| y != *x && entry_within(x, y) && (!entry_within(y, x) || y < *x)))
        .cloned()
        .collect()
}

/// The meet of two `net` sets: every non-empty meet of an entry of each — never wider than either side, and the
/// greatest set that is not (each entry of a common lower bound is within one entry of each side, so within their meet).
pub fn set_meet(a: &std::collections::BTreeSet<String>, b: &std::collections::BTreeSet<String>) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for x in a {
        for y in b {
            if let Some(m) = entry_meet(x, y) {
                out.insert(m);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pat(p: &str) -> Vec<Segment> {
        parse_pattern(p).unwrap_or_else(|e| panic!("{p}: {e}"))
    }

    #[test]
    fn a_pattern_is_literals_one_segment_wildcards_and_a_last_rest() {
        assert_eq!(pat("/repos/*/issues/**"), vec![
            Segment::Literal("repos".into()),
            Segment::One,
            Segment::Literal("issues".into()),
            Segment::Rest
        ]);
        for bad in ["", "repos", "/", "/a//b", "/a/", "/**/a", "/a/**/b", "/v1*", "/*x", "/a/./b", "/a/..", "/a%2Fb", "/a;b",
                    "/a?b", "/a#b", "/a b", "/a\\b", "/é"] {
            assert!(parse_pattern(bad).is_err(), "`{bad}` is refused");
        }
        assert!(parse_pattern(&format!("/{}", "a/".repeat(MAX_SEGMENTS) + "b")).is_err(), "too many segments");
    }

    #[test]
    fn a_path_matches_segment_by_segment() {
        let p = pat("/repos/*/issues/**");
        for ok in ["/repos/org/issues", "/repos/org/issues/", "/repos/org/issues/1", "/repos/org/issues/1/comments"] {
            assert!(path_matches(&p, ok), "{ok}");
        }
        for no in ["/", "", "/repos", "/repos/org", "/repos//issues", "/repos/org/pulls/1", "/Repos/org/issues", "/repos/org/issuesx"] {
            assert!(!path_matches(&p, no), "{no}");
        }
        let exact = pat("/v1/status");
        assert!(path_matches(&exact, "/v1/status"));
        assert!(!path_matches(&exact, "/v1/status/"), "a trailing `/` is another path");
        assert!(!path_matches(&exact, "/v1/status/x"));
        assert!(path_matches(&pat("/**"), "/") && path_matches(&pat("/**"), "") && path_matches(&pat("/**"), "/a/b"));
        assert!(!path_matches(&pat("/a/*"), "/a/"), "`*` is never an empty segment");
    }

    #[test]
    fn a_path_a_server_could_read_another_way_is_refused_not_matched() {
        for bad in ["/a/../b", "/a/./b", "/a//b", "/a;x/b", "/..;/admin", "/a%2Fb", "/a%2fb", "/a%5Cb", "/a%2E%2E/b", "/a%252Fb",
                    "/a%00", "/a%3Bb", "/a%", "/a%2", "/a%zz"] {
            assert!(ambiguity(bad).is_some(), "`{bad}` is ambiguous");
        }
        for plain in ["", "/", "/a/b", "/a/b/", "/a%20b", "/caf%C3%A9", "/a-b_c.d~e"] {
            assert_eq!(ambiguity(plain), None, "`{plain}` is plain");
        }
    }

    #[test]
    fn an_entry_judges_its_host_then_its_path() {
        assert_eq!(judge("api.example.com", "api.example.com", "/a/../b"), Verdict::Allow, "a host-only grant reaches every path");
        assert_eq!(judge("api.example.com/v1/**", "api.example.com", "/v1/x"), Verdict::Allow);
        assert_eq!(judge("api.example.com/v1/**", "api.example.com", "/v2/x"), Verdict::OutsidePath);
        assert!(matches!(judge("api.example.com/v1/**", "api.example.com", "/v1/../v2"), Verdict::Ambiguous(_)));
        assert_eq!(judge("api.example.com/v1/**", "evil.example.com", "/v1/x"), Verdict::OtherHost);
        assert_eq!(judge("*.example.com/v1/**", "api.example.com", "/v1/x"), Verdict::Allow, "the host part is `host_matches`'");
        assert_eq!(judge("*.example.com/v1/**", "example.com", "/v1/x"), Verdict::OtherHost, "C85: the apex is not under `*.`");
        assert_eq!(judge("api.example.com/v1/*x", "api.example.com", "/v1/ax"), Verdict::OutsidePath, "an unparsed entry matches nothing");
        assert_eq!(judge("[::1]/v1/**", "[::1]", "/v1/x"), Verdict::Allow);
        let list = ["a.com/x/**".to_string(), "b.com".to_string()];
        assert_eq!(judge_all(&list, "a.com", "/y"), Verdict::OutsidePath);
        assert_eq!(judge_all(&list, "b.com", "/y"), Verdict::Allow);
        assert_eq!(judge_all(&list, "c.com", "/x"), Verdict::OtherHost);
        let both = ["a.com/x/**".to_string(), "a.com".to_string()];
        assert_eq!(judge_all(&both, "a.com", "/y/../z"), Verdict::Allow, "an unrestricted entry for the host allows any path");
        assert_eq!(host_of("api.example.com/v1/**"), "api.example.com");
    }

    /// P9-05 (b), D-V2-139: the order and the meet decided by the syntax are EXACT on a bounded domain — every pattern of up
    /// to three segments over two literals and `*`, with or without a final `**`, against every path of up to five
    /// segments over those literals, a third one and a trailing `/`: `pattern_within(c, p)` holds exactly when every path
    /// `c` matches `p` matches, and the meet matches exactly the paths both match (`None` exactly when none does).
    #[test]
    fn the_order_and_the_meet_are_exact_over_a_bounded_domain() {
        let (max_pat, max_path) = if cfg!(miri) { (2, 3) } else { (3, 5) };
        let syms = [Segment::Literal("a".into()), Segment::Literal("b".into()), Segment::One];
        let mut pats: Vec<Vec<Segment>> = vec![vec![Segment::Rest]];
        let mut layer: Vec<Vec<Segment>> = vec![Vec::new()];
        for _ in 0..max_pat {
            layer = layer.iter().flat_map(|p| syms.iter().map(move |s| [p.clone(), vec![s.clone()]].concat())).collect();
            for p in &layer {
                pats.push(p.clone());
                pats.push([p.clone(), vec![Segment::Rest]].concat());
            }
        }
        let mut paths: Vec<String> = vec![String::new(), "/".into()];
        let mut prefixes: Vec<String> = vec![String::new()];
        for _ in 0..max_path {
            prefixes = prefixes.iter().flat_map(|p| ["a", "b", "c"].map(|s| format!("{p}/{s}"))).collect();
            for p in &prefixes {
                paths.push(p.clone());
                paths.push(format!("{p}/"));
            }
        }
        let lang = |p: &[Segment]| -> Vec<bool> { paths.iter().map(|x| path_matches(p, x)).collect() };
        let langs: Vec<Vec<bool>> = pats.iter().map(|p| lang(p)).collect();
        let mut within_pairs = 0;
        for (i, c) in pats.iter().enumerate() {
            for (j, p) in pats.iter().enumerate() {
                let semantic = langs[i].iter().zip(&langs[j]).all(|(x, y)| !x || *y);
                assert_eq!(pattern_within(c, p), semantic, "{} within {}", render(c), render(p));
                within_pairs += usize::from(semantic);
                let both: Vec<bool> = langs[i].iter().zip(&langs[j]).map(|(x, y)| *x && *y).collect();
                match pattern_meet(c, p) {
                    Some(m) => {
                        assert!(parse_pattern(&render(&m)).is_ok(), "the meet is a pattern: {}", render(&m));
                        assert_eq!(lang(&m), both, "{} ⊓ {} = {}", render(c), render(p), render(&m));
                    }
                    None => assert!(both.iter().all(|x| !x), "{} ⊓ {} is empty, yet a path matches both", render(c), render(p)),
                }
            }
        }
        // The control: a domain where nothing (or everything) was within proved nothing.
        assert!(within_pairs > pats.len() && within_pairs < pats.len() * pats.len() / 2, "{within_pairs} of {}", pats.len().pow(2));
    }

    /// P17-F3 for `net` (D-V2-139): over every subset of a small universe of entries, the canonical form is ⊑-equivalent
    /// to the set, canonicalizing twice changes nothing, and two sets each within the other canonicalize identically.
    #[test]
    fn equivalent_net_sets_canonicalize_identically() {
        let universe = ["h.com", "h.com/a/**", "h.com/a/b", "h.com/*/b", "h.com/b/**", "g.com", "g.com/a/*"];
        let subsets: Vec<std::collections::BTreeSet<String>> = (0..1u32 << universe.len())
            .map(|m| universe.iter().enumerate().filter(|(i, _)| m & (1 << i) != 0).map(|(_, e)| e.to_string()).collect())
            .collect();
        let step = if cfg!(miri) { 7 } else { 1 };
        let mut equivalent_pairs = 0;
        for a in subsets.iter().step_by(step) {
            let ca = canonicalize_set(a);
            assert!(set_within(&ca, a) && set_within(a, &ca), "{a:?} -> {ca:?}");
            assert_eq!(canonicalize_set(&ca), ca, "idempotent: {a:?}");
            for b in subsets.iter().step_by(step) {
                if set_within(a, b) && set_within(b, a) {
                    equivalent_pairs += usize::from(a != b);
                    assert_eq!(ca, canonicalize_set(b), "{a:?} and {b:?} are equivalent");
                }
            }
        }
        assert!(cfg!(miri) || equivalent_pairs > 100, "{equivalent_pairs} unequal equivalent pairs — the control");
    }

    #[test]
    fn entries_and_sets_are_ordered_by_host_then_path() {
        assert!(entry_within("h.com/v1/x", "h.com/v1/**") && entry_within("h.com/v1/**", "h.com") && entry_within("h.com", "h.com"));
        assert!(!entry_within("h.com", "h.com/v1/**"), "every path is not within some paths");
        assert!(!entry_within("h.com/v2/x", "h.com/v1/**") && !entry_within("g.com/v1/x", "h.com/v1/**"));
        assert!(!entry_within("api.h.com/v1/x", "*.h.com/v1/**"), "a host under a pattern is not contained, as before");
        assert!(entry_within("h.com/v1*", "h.com/v1*") && !entry_within("h.com/v1*", "h.com/**"), "an unparsed entry: itself only");
        assert_eq!(entry_meet("h.com", "h.com/v1/**").as_deref(), Some("h.com/v1/**"));
        assert_eq!(entry_meet("h.com/*/x/**", "h.com/v1/**").as_deref(), Some("h.com/v1/x/**"));
        assert_eq!(entry_meet("h.com/v1/**", "h.com/v2/**"), None);
        assert_eq!(entry_meet("h.com/v1/**", "g.com/v1/**"), None);
        let set = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<std::collections::BTreeSet<String>>();
        assert!(set_within(&set(&["h.com/v1/a", "g.com"]), &set(&["h.com/v1/**", "g.com"])));
        assert!(!set_within(&set(&["h.com/v1/a", "f.com"]), &set(&["h.com/v1/**", "g.com"])));
        assert_eq!(set_meet(&set(&["h.com", "g.com/x/**"]), &set(&["h.com/v1/**", "g.com/x/y", "f.com"])), set(&["h.com/v1/**", "g.com/x/y"]));
        assert_eq!(canonicalize_set(&set(&["h.com", "h.com/v1/**", "g.com/x/*", "g.com/x/y"])), set(&["h.com", "g.com/x/*"]));
        assert_eq!(check_entry("h.com/v1/**"), Ok(()));
        assert!(check_entry("/v1/**").is_err() && check_entry("h.com/v1/").is_err() && check_entry("h.com/a/../b").is_err());
    }
}
