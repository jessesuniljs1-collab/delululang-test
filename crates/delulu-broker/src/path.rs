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

/// The **canonical spelling** of a path scope — the unique representative of its `⊑`-equivalence
/// class (P17-F1/F2/F3).
///
/// `resolve` is not injective: `./data`, `data`, `./data/` and `.\data` all resolve to the same
/// segments. A relation defined through a non-injective function is a **preorder** on its domain,
/// never a partial order — so `⊑` on raw spellings cannot be antisymmetric, and two spellings of
/// one grant hash differently in the audit chain. The repair is to pick one representative per
/// class and store *that*; `⊑` is then a genuine partial order on the image of this function.
///
/// **The contract is `resolve(canonicalize(p)) == resolve(p)` — canonicalization preserves the
/// resolved path exactly, so it can neither widen nor narrow any authority.** It normalizes the
/// *spelling*, never the meaning; it is not a repair for anything `resolve` itself gets wrong.
/// `canonicalize_is_authority_preserving` checks that exhaustively over the adversarial corpus.
pub fn canonicalize(path: &str) -> String {
    render(&resolve(path))
}

/// The canonical form of a path *set*: canonical spellings, reduced to an **antichain**.
///
/// Canonicalizing each element is not enough to make `⊑` antisymmetric, because a path set carries
/// a second, independent redundancy: `{./data, ./data/sub}` and `{./data}` denote exactly the same
/// covered region, since `./data/sub` is already inside `./data`. Both directions of `⊑` hold
/// between them while the sets differ, so the order stays a preorder no matter how well the
/// individual spellings are normalized. Dropping every element that lies within another leaves the
/// `⊑`-maximal elements — one representative per class.
///
/// **Authority-preserving on both counts.** A subsumed element contributes nothing to the covered
/// region (anything inside it is already inside the element that subsumes it), so removing it
/// changes no containment decision. Mutual elimination cannot empty a set: two distinct canonical
/// paths cannot contain each other, since that would force their resolved segments — and therefore
/// their canonical spellings — to be equal.
pub fn canonicalize_set<'a, I>(paths: I) -> std::collections::BTreeSet<String>
where
    I: IntoIterator<Item = &'a String>,
{
    let spelled: std::collections::BTreeSet<String> =
        paths.into_iter().map(|p| canonicalize(p)).collect();
    spelled
        .iter()
        .filter(|x| !spelled.iter().any(|y| y != *x && is_descendant_or_equal(x, y)))
        .cloned()
        .collect()
}

/// Render resolved segments back to a path string that re-resolves to the same segments.
fn render(segs: &[Seg]) -> String {
    match segs.first() {
        None => ".".to_string(),
        Some(Seg::Root) => {
            let rest = join_tail(&segs[1..]);
            if rest.is_empty() { "/".to_string() } else { format!("/{rest}") }
        }
        Some(Seg::Drive(d)) => {
            let rest = join_tail(&segs[1..]);
            if rest.is_empty() { format!("{d}/") } else { format!("{d}/{rest}") }
        }
        // A relative path ALWAYS carries the `./` prefix. Without it a leading component that
        // happens to look like a drive letter re-resolves as a drive: `./C:` is `[Name("C:")]`, a
        // directory named `C:` beneath the origin, but the bare spelling `C:` is `[Drive("C:")]` —
        // the whole of drive C. That is a WIDENING, and it is why this cannot just join segments.
        _ => format!("./{}", join_tail(segs)),
    }
}

/// Join the non-anchor segments. `Root`/`Drive` are anchors that [`resolve`] only ever emits at
/// index 0, where [`render`] has already consumed them; rendering them by their own text here keeps
/// the function total without a panic branch, and `render_is_total_on_resolved_segments` pins that
/// the anchor-in-tail case is genuinely unreachable rather than merely unobserved.
fn join_tail(segs: &[Seg]) -> String {
    segs.iter()
        .map(|s| match s {
            Seg::Up => "..".to_string(),
            Seg::Name(n) => n.clone(),
            Seg::Root => "/".to_string(),
            Seg::Drive(d) => d.clone(),
        })
        .collect::<Vec<_>>()
        .join("/")
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
///
/// **The result is [`canonicalize`]d (P17-F2).** When `x` and `y` are two spellings of the same
/// path, *both* descendant tests hold, so the `if`/`else if` inserted whichever side happened to be
/// the outer loop — making `⊓` depend on argument order, in direct contradiction of the
/// "note `⊓` is symmetric" comment in `authority.rs`. Emitting the canonical representative makes
/// the two orders agree by construction rather than by luck.
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
    canonicalize_set(&out)
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

    /// Adversarial spellings. Anything whose canonical form could plausibly re-resolve differently
    /// belongs here — especially components that *look* like anchors (`C:`), runs of dots, and the
    /// separators/empties that `resolve` silently drops.
    const CORPUS: &[&str] = &[
        // empties and bare anchors
        "", ".", "./", ".//", "/", "//", "///", "\\", ".\\", "..", "../", "./..", "../..", "..\\..",
        // ordinary relative
        "data", "./data", "./data/", "data/", ".\\data", ".\\data\\", "./data//sub", "./data/sub",
        "data/sub/", "./other", "a", "./a/b/c", "a/b/c/",
        // `..` interactions
        "./data/../secret", "./data/x/../y", "./data/../../etc", "../foo", "../../foo", "a/../b",
        "a/../../b", "/..", "/../..", "/a/../..", "/a/../b", "./a/../..", "a/b/../../..",
        // rooted and drive-anchored
        "/usr/lib", "/usr/lib/", "/usr//lib", "\\usr\\lib", "C:", "C:/", "C:\\", "C:/a", "C:\\a\\b",
        "c:/a", "C:extra", "C:/a/../..", "D:\\libs",
        // components that LOOK like anchors — the widening trap `render`'s `./` prefix exists for
        "./C:", "./C:/x", "a/C:", "a/../C:", "./c:", "./C:/../D:", "././C:",
        // dot-ish names that are NOT `.` or `..`
        "...", "....", "./...", "a..b", "..a", "a..", "./.git", "./.hidden/x",
        // whitespace and unicode
        "a b/c", " ", "./ /x", "日本/データ", "./日本/../データ", "./naïve/café",
        // separators only
        "/////", "\\\\\\", "./\\/.\\/",
    ];

    /// A deterministic LCG. Reproducibility beats quality here: a failing case must replay
    /// identically on every platform, and `rand` is not a dependency of this crate.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 =
                self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 16
        }
        fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
            xs[(self.next() % xs.len() as u64) as usize]
        }
    }

    /// Build a random path by concatenating fragments — including the ones that make `resolve`
    /// interesting: separators of both kinds, `.`/`..`, empty segments, and anchor lookalikes.
    fn random_path(rng: &mut Rng) -> String {
        const HEAD: &[&str] = &["", "./", "/", "C:", "C:/", ".\\", "//", "d:"];
        const SEG: &[&str] = &[
            "a", "data", "sub", "..", ".", "C:", "c:", "...", ".git", "naïve", "x y", "日本", "",
            "database", "Data",
        ];
        const SEP: &[&str] = &["/", "\\", "//", "/./"];
        let mut s = String::from(rng.pick(HEAD));
        let n = 1 + (rng.next() % 7) as usize;
        for i in 0..n {
            if i > 0 {
                s.push_str(rng.pick(SEP));
            }
            s.push_str(rng.pick(SEG));
        }
        if rng.next().is_multiple_of(4) {
            s.push_str(rng.pick(SEP));
        }
        s
    }

    /// **The generated counterpart to `CORPUS`, and the reason it exists.** The campaign's own rule
    /// is *"do not write examples, generate inputs"* — a corpus can only contain the shapes someone
    /// thought of, and this is the most safety-critical function in the module. Every law the
    /// hand-written cases check is re-checked here over 20,000 machine-built paths, so a spelling
    /// nobody imagined has to satisfy them too.
    #[test]
    fn canonicalization_laws_hold_on_generated_paths() {
        let mut rng = Rng(0x0D31_11A1_5EED_1234);
        let mut seen: Vec<(Vec<Seg>, String)> = Vec::new();
        for i in 0..20_000u32 {
            let p = random_path(&mut rng);
            let c = canonicalize(&p);

            // 1. Authority-preserving: canonicalizing cannot change the resolved path.
            assert_eq!(
                resolve(&c),
                resolve(&p),
                "iteration {i}: canonicalize({p:?}) = {c:?} resolves differently — authority CHANGED"
            );
            // 2. Idempotent, or "store the canonical form" is not a well-defined instruction.
            assert_eq!(canonicalize(&c), c, "iteration {i}: not idempotent at {p:?}");
            // 3. An anchor may only ever appear first — `join_tail`'s totality rests on this.
            for (j, s) in resolve(&p).iter().enumerate() {
                assert!(
                    !(matches!(s, Seg::Root | Seg::Drive(_)) && j != 0),
                    "iteration {i}: {p:?} resolved an anchor at index {j}"
                );
            }
            // 4. Completeness: equal resolved segments ⟹ identical canonical spelling. Checked
            //    against everything generated so far, which is what makes it a real quotient claim
            //    rather than a per-input one.
            let segs = resolve(&p);
            if let Some((_, prev)) = seen.iter().find(|(s, _)| *s == segs) {
                assert_eq!(*prev, c, "iteration {i}: {p:?} resolves like an earlier path but canonicalizes apart");
            } else if seen.len() < 400 {
                seen.push((segs, c.clone()));
            }
        }

        // 5. Containment decisions are identical before and after, over every pair of a sample.
        let mut rng = Rng(0x00C0_FFEE_1234_5678);
        let sample: Vec<String> = (0..90).map(|_| random_path(&mut rng)).collect();
        for a in &sample {
            for b in &sample {
                assert_eq!(
                    is_descendant_or_equal(a, b),
                    is_descendant_or_equal(&canonicalize(a), &canonicalize(b)),
                    "containment of {a:?} in {b:?} flipped under canonicalization"
                );
            }
        }
    }

    /// The set-level law on generated input: `canonicalize_set` must leave the covered region
    /// unchanged. Dropping a subsumed element is only sound if nothing that was inside the set
    /// before falls outside it after.
    #[test]
    fn set_canonicalization_preserves_the_covered_region_on_generated_input() {
        let mut rng = Rng(0x000A_11CE_5EED);
        for i in 0..3_000u32 {
            let raw: BTreeSet<String> =
                (0..1 + (rng.next() % 4) as usize).map(|_| random_path(&mut rng)).collect();
            let canon = canonicalize_set(&raw);

            assert!(!canon.is_empty(), "iteration {i}: {raw:?} canonicalized to nothing");
            // Every probe inside the raw set is inside the canonical one, and vice versa.
            for probe in raw.iter().chain(canon.iter()) {
                assert_eq!(
                    all_within(std::iter::once(probe), &raw),
                    all_within(std::iter::once(probe), &canon),
                    "iteration {i}: {probe:?} changed membership; raw={raw:?} canon={canon:?}"
                );
            }
            // It is an antichain: no element lies within another.
            for x in &canon {
                for y in &canon {
                    assert!(
                        !(x != y && is_descendant_or_equal(x, y)),
                        "iteration {i}: {x:?} is inside {y:?} — not an antichain: {canon:?}"
                    );
                }
            }
            assert_eq!(canonicalize_set(&canon), canon, "iteration {i}: set canonicalization not idempotent");
        }
    }

    /// **THE safety property.** Canonicalization must preserve the resolved path exactly. If it
    /// ever changed `resolve`, a grant's meaning would shift the moment it was stored — silently
    /// widening or narrowing an authority nobody re-approved.
    #[test]
    fn canonicalize_is_authority_preserving() {
        for p in CORPUS {
            assert_eq!(
                resolve(&canonicalize(p)),
                resolve(p),
                "canonicalize({p:?}) = {:?} resolves differently — the authority CHANGED",
                canonicalize(p)
            );
        }
    }

    /// A canonical form must be a fixed point, or "store the canonical spelling" would not be a
    /// well-defined instruction.
    #[test]
    fn canonicalize_is_idempotent() {
        for p in CORPUS {
            let once = canonicalize(p);
            assert_eq!(canonicalize(&once), once, "canonicalize is not idempotent at {p:?}");
        }
    }

    /// The completeness half: same resolved path ⟹ *identical* canonical spelling. This is what
    /// makes `⊑` antisymmetric on canonical representatives (P17-F1) and makes ⊑-equivalent
    /// authorities hash identically in the audit chain (P17-F3).
    #[test]
    fn equivalent_spellings_get_one_representative() {
        for a in CORPUS {
            for b in CORPUS {
                if resolve(a) == resolve(b) {
                    assert_eq!(
                        canonicalize(a),
                        canonicalize(b),
                        "{a:?} and {b:?} resolve alike but canonicalize apart"
                    );
                }
            }
        }
        // The four spellings that motivated the finding really are one class.
        let want = canonicalize("./data");
        for spelling in ["data", "./data/", r".\data", ".//data//", r"./data\"] {
            assert_eq!(canonicalize(spelling), want, "{spelling:?} is not in ./data's class");
        }
    }

    /// The trap `render`'s mandatory `./` prefix exists for: a component named like a drive letter
    /// must not be promoted into an *anchor*. Without the prefix, a grant over a subdirectory
    /// called `C:` would canonicalize to the whole of drive C.
    #[test]
    fn a_component_that_looks_like_a_drive_is_not_promoted_to_one() {
        assert_eq!(canonicalize("./C:"), "./C:");
        assert_ne!(canonicalize("./C:"), canonicalize("C:"));
        // The relative name is not within the drive root, and the drive root is not within it.
        assert!(!is_descendant_or_equal(&canonicalize("./C:"), &canonicalize("C:")));
        assert!(!is_descendant_or_equal(&canonicalize("C:"), &canonicalize("./C:")));
        // And the same after a `..` rewrite lands the lookalike at the front.
        assert_eq!(canonicalize("a/../C:"), "./C:");
    }

    /// `render` consumes an anchor only at index 0. This pins that `resolve` cannot put one
    /// anywhere else, so `join_tail`'s anchor arms are unreachable rather than merely untested.
    #[test]
    fn render_is_total_on_resolved_segments() {
        for p in CORPUS {
            let segs = resolve(p);
            for (i, s) in segs.iter().enumerate() {
                if matches!(s, Seg::Root | Seg::Drive(_)) {
                    assert_eq!(i, 0, "{p:?} resolved an anchor at index {i}, not 0: {segs:?}");
                }
            }
            // `Up` never follows a `Name` — the module doc-comment's claim about the shape.
            let first_name = segs.iter().position(|s| matches!(s, Seg::Name(_)));
            if let Some(n) = first_name {
                assert!(
                    !segs[n..].iter().any(|s| matches!(s, Seg::Up)),
                    "{p:?} put an Up after a Name: {segs:?}"
                );
            }
        }
    }

    /// Canonicalizing either side of the descendant test cannot change its answer — the direct
    /// consequence of authority-preservation, checked over every ordered pair in the corpus.
    #[test]
    fn canonicalizing_never_changes_a_containment_decision() {
        for c in CORPUS {
            for p in CORPUS {
                let raw = is_descendant_or_equal(c, p);
                assert_eq!(
                    is_descendant_or_equal(&canonicalize(c), &canonicalize(p)),
                    raw,
                    "containment of {c:?} in {p:?} flipped under canonicalization"
                );
            }
        }
    }

    /// The meet must now be order-independent even when handed raw, differently-spelled input —
    /// this is P17-F2 closed at its source rather than at the call sites.
    #[test]
    fn path_meet_is_symmetric_even_on_mixed_spellings() {
        let a = set(&["./data"]);
        let b = set(&["data"]);
        assert_eq!(intersect_path_sets(&a, &b), intersect_path_sets(&b, &a));
        assert_eq!(intersect_path_sets(&a, &b), set(&["./data"]));
        let c = set(&[r".\data\sub"]);
        assert_eq!(intersect_path_sets(&a, &c), intersect_path_sets(&c, &a));
        assert_eq!(intersect_path_sets(&a, &c), set(&["./data/sub"]));
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
