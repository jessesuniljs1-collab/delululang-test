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
}
