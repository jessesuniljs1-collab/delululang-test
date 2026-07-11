//! Pure-lexical path descendant semantics for the `⊑` lattice (Stage 5, spec §A.3 "path is a
//! descendant").
//!
//! **Ruling 2 (head chef):** path comparison is PURE LEXICAL — no filesystem access ever. We never
//! call `canonicalize()`, `Path::exists`, or touch the disk. A path is normalized by:
//!   1. replacing `\` with `/` (so a Windows `.\data\sub` compares equal to a POSIX `./data/sub`),
//!   2. resolving `.` and `..` segments lexically (a `..` that would rise above the notional origin
//!      of a *relative* path becomes an explicit "up" marker — such a path escapes its root and is
//!      therefore NOT a descendant of any in-tree grant),
//!   3. component-wise prefix comparison, **case-sensitive**.
//!
//! **Windows case-insensitivity caveat:** real Windows/macOS filesystems are case-insensitive, so
//! `./Data` and `./data` denote the same directory there but compare as *distinct* here. This is
//! deliberately conservative: treating them as distinct can only ever *narrow* what a lease is
//! judged to cover (fail-closed), never widen it. A case-folding refinement keyed to the host
//! filesystem is a possible post-1.0 change; conservative-lexical is sound today.

/// One resolved path segment. A canonicalized relative path is a run of zero-or-more [`Seg::Up`]
/// followed by zero-or-more [`Seg::Name`]; `Up` never follows a `Name` (the `..` already popped it).
#[derive(Clone, Debug, PartialEq, Eq)]
enum Seg {
    /// The POSIX filesystem root, from a leading `/`.
    Root,
    /// A Windows drive prefix such as `C:` (rooted, like `Root` but drive-scoped).
    Drive(String),
    /// An unresolved `..` — the path rose above its notional origin (a relative escape).
    Up,
    /// An ordinary path component.
    Name(String),
}

/// Resolve a path string to its canonical lexical segments. No disk access (ruling 2).
fn resolve(path: &str) -> Vec<Seg> {
    let unified = path.replace('\\', "/");
    let mut rest = unified.as_str();
    let mut segs: Vec<Seg> = Vec::new();

    // A leading Windows drive prefix (`C:` optionally followed by `/…`). We split on the FIRST
    // colon only, mirroring the grant parser's Windows-path handling in the Stage-1 broker.
    if unified.len() >= 2 {
        let bytes = unified.as_bytes();
        if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            segs.push(Seg::Drive(unified[..2].to_string()));
            rest = &unified[2..];
        }
    }
    // A leading `/` on a path with no drive prefix is the POSIX root. (If a drive is already
    // present it implies the root, so the leading `/` there is just a separator.)
    if rest.starts_with('/') && segs.is_empty() {
        segs.push(Seg::Root);
    }

    for part in rest.split('/') {
        match part {
            "" | "." => {} // skip empties (from leading/trailing/double slashes) and `.`
            ".." => match segs.last() {
                Some(Seg::Name(_)) => {
                    segs.pop();
                }
                // Cannot rise above a filesystem root; `..` there is a no-op (matches real FS).
                Some(Seg::Root) | Some(Seg::Drive(_)) => {}
                // Empty, or already a run of `..`: this relative path escapes its origin.
                _ => segs.push(Seg::Up),
            },
            name => segs.push(Seg::Name(name.to_string())),
        }
    }
    segs
}

/// Is `child` a descendant-or-equal of `parent`, purely lexically (spec §A.3)?
///
/// True iff, after normalization, `parent`'s segments are a prefix of `child`'s AND the remaining
/// child tail contains no [`Seg::Up`] (an `Up` in the tail means the child reaches *above* the
/// parent's root — an escape, never a descendant). Equal paths are descendants of themselves.
pub fn is_descendant_or_equal(child: &str, parent: &str) -> bool {
    let c = resolve(child);
    let p = resolve(parent);
    if c.len() < p.len() {
        return false;
    }
    if c[..p.len()] != p[..] {
        return false;
    }
    // The child must not step above the parent root via unmatched `..`.
    !c[p.len()..].iter().any(|s| matches!(s, Seg::Up))
}

/// The lexical intersection ("meet") of two path scopes under the descendant lattice.
///
/// For every `(a, b)` pair where one path lies within the other, the deeper (narrower) path is in
/// the meet. Every element of the result is within BOTH inputs, so the meet is never wider than
/// either side — exactly what DL0802's "repair never widens" requires (spec §8).
pub fn intersect_path_sets<'a, A, B>(a: A, b: B) -> std::collections::BTreeSet<String>
where
    A: IntoIterator<Item = &'a String> + Clone,
    B: IntoIterator<Item = &'a String> + Clone,
{
    let mut out = std::collections::BTreeSet::new();
    for x in a.clone() {
        for y in b.clone() {
            if is_descendant_or_equal(x, y) {
                out.insert(x.clone()); // x ⊆ y, x is the narrower
            } else if is_descendant_or_equal(y, x) {
                out.insert(y.clone()); // y ⊆ x, y is the narrower
            }
        }
    }
    out
}

/// Is every path in `child` a descendant-or-equal of some path in `parent`? (Empty `child` is
/// vacuously within; a non-empty `child` against an empty `parent` is NOT within — fail-closed.)
pub fn all_within<'a, C, P>(child: C, parent: P) -> bool
where
    C: IntoIterator<Item = &'a String>,
    P: IntoIterator<Item = &'a String> + Clone,
{
    child
        .into_iter()
        .all(|c| parent.clone().into_iter().any(|p| is_descendant_or_equal(c, p)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn subtree_is_a_descendant() {
        assert!(is_descendant_or_equal("./data/sub", "./data"));
        assert!(is_descendant_or_equal("./data", "./data")); // equal → descendant-or-equal
        assert!(is_descendant_or_equal("./data/a/b/c", "./data"));
    }

    #[test]
    fn sibling_prefix_is_not_a_descendant_no_naive_string_prefix_bug() {
        // The classic bug: "./database".starts_with("./data") is TRUE as strings, but `database`
        // is NOT under `data` as a path. Component-wise comparison catches it.
        assert!(!is_descendant_or_equal("./database", "./data"));
        assert!(!is_descendant_or_equal("./data-backup", "./data"));
        assert!(!is_descendant_or_equal("./datax", "./data"));
    }

    #[test]
    fn dotdot_traversal_that_stays_inside_is_fine() {
        // ./data/x/../y resolves to ./data/y — still under ./data.
        assert!(is_descendant_or_equal("./data/x/../y", "./data"));
    }

    #[test]
    fn dotdot_traversal_that_escapes_is_refused() {
        // ./data/../secret resolves to ./secret — a sibling, not under ./data.
        assert!(!is_descendant_or_equal("./data/../secret", "./data"));
        // ./data/../../etc rises above the origin entirely.
        assert!(!is_descendant_or_equal("./data/../../etc", "./data"));
        // A bare escaping request is never within an in-tree grant (the empty-parent trap).
        assert!(!is_descendant_or_equal("../foo", "."));
        assert!(!is_descendant_or_equal("../foo", "./data"));
    }

    #[test]
    fn windows_separators_normalize_to_forward_slash() {
        assert!(is_descendant_or_equal(r".\data\sub", "./data"));
        assert!(is_descendant_or_equal("./data/sub", r".\data"));
        assert!(!is_descendant_or_equal(r".\database\sub", "./data"));
    }

    #[test]
    fn absolute_and_drive_paths() {
        assert!(is_descendant_or_equal("/usr/lib/x", "/usr/lib"));
        assert!(!is_descendant_or_equal("/usr/libfoo", "/usr/lib"));
        assert!(!is_descendant_or_equal("/usr/lib", "usr/lib")); // absolute vs relative differ
        assert!(is_descendant_or_equal(r"C:\libs\a", r"C:\libs"));
        assert!(!is_descendant_or_equal(r"D:\libs\a", r"C:\libs")); // different drive
        // `..` cannot rise above a filesystem root (matches real filesystems).
        assert!(is_descendant_or_equal("/usr/../usr/lib", "/usr/lib"));
        assert!(is_descendant_or_equal("/../etc", "/etc"));
    }

    #[test]
    fn case_sensitive_by_design() {
        // Conservative: distinct case → distinct path (never widens). See module caveat.
        assert!(!is_descendant_or_equal("./Data/sub", "./data"));
    }

    #[test]
    fn path_meet_is_never_wider_than_either_side() {
        // ./data ∩ ./data/sub = ./data/sub (the narrower).
        assert_eq!(
            intersect_path_sets(&set(&["./data"]), &set(&["./data/sub"])),
            set(&["./data/sub"])
        );
        // Disjoint subtrees meet to nothing.
        assert!(intersect_path_sets(&set(&["./data"]), &set(&["./other"])).is_empty());
        // Multiple roots: only the overlapping pair contributes, narrower side.
        assert_eq!(
            intersect_path_sets(&set(&["./a", "./b/c"]), &set(&["./b", "./z"])),
            set(&["./b/c"])
        );
    }

    #[test]
    fn all_within_semantics() {
        let empty: BTreeSet<String> = BTreeSet::new();
        assert!(all_within(&set(&["./data/sub"]), &set(&["./data"])));
        assert!(all_within(&empty, &set(&["./data"]))); // empty child vacuously within
        assert!(!all_within(&set(&["./data/sub"]), &empty)); // non-empty child, empty parent
        assert!(!all_within(&set(&["./data", "./other"]), &set(&["./data"])));
    }
}
