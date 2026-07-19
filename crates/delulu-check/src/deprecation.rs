//! The deprecation registry and DL1801 (Stage 9c, spec §2.2).
//!
//! ## The policy this implements
//! A deprecation is a **warning, never a break**. Every entry is RFC-gated, lives at least two
//! minor versions before anything is removed, and removal happens only on a major bump. Where the
//! migration is mechanical the entry names its replacement, and `delulu fmt --migrate` can perform
//! it; where it is not, the RFC explains what to do instead.
//!
//! ## Why the table is empty at 1.0
//! Nothing is deprecated at 1.0 — a language deprecating parts of itself on its release day would
//! be announcing that the freeze it just declared is not real. The table is empty **and the
//! mechanism is complete**: the warning, the repair, the fmt rule and the tests all exist and are
//! exercised against synthetic tables, so the first real deprecation is a data change rather than
//! a feature that must be built under pressure.
//!
//! The functions here take the table as a parameter for exactly that reason: the policy is
//! testable without deprecating anything real.

use delulu_diag::{Diagnostic, Span};

/// What kind of thing was deprecated. The kind decides how a use is recognized.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeprKind {
    /// A free function or module-level name.
    Name,
    /// A method on a primitive receiver, written `receiver.method`.
    Method,
    /// A manifest key, written `[section] key`.
    ManifestKey,
}

/// One deprecated item.
#[derive(Clone, Copy, Debug)]
pub struct Deprecation {
    /// The item as a program writes it (`old_fn`, `console.print_line`, `[package] old_key`).
    pub item: &'static str,
    pub kind: DeprKind,
    /// The version in which the deprecation took effect.
    pub since: &'static str,
    /// The mechanical replacement, when there is one. `None` means migration needs judgement and
    /// the RFC has to be read — the honest case, not a gap to paper over.
    pub replacement: Option<&'static str>,
    /// The RFC that decided it. Never empty: a deprecation nobody argued for in public is not
    /// policy, it is a whim.
    pub rfc: &'static str,
}

/// The live registry. **Empty at 1.0** — see the module docs.
pub const DEPRECATIONS: &[Deprecation] = &[];

/// Find a deprecation for `item`, if any.
pub fn lookup<'a>(table: &'a [Deprecation], item: &str) -> Option<&'a Deprecation> {
    table.iter().find(|d| d.item == item)
}

/// The DL1801 warning for a use of `item`, or `None` if it is not deprecated.
///
/// Always a WARNING: a deprecation that failed the build would be a removal wearing a friendlier
/// name, and the policy promises removals happen only on a major version.
pub fn warn(table: &[Deprecation], item: &str, span: Span) -> Option<Diagnostic> {
    let d = lookup(table, item)?;
    let advice = match d.replacement {
        Some(r) => format!("use `{r}` instead"),
        None => format!("see {} for the migration", d.rfc),
    };
    let mut diag = Diagnostic::warning(
        "DL1801",
        format!(
            "`{}` is deprecated since {} — {advice} (decided in {})",
            d.item, d.since, d.rfc
        ),
    )
    .with_span(span, "deprecated here")
    .with_arg("item", d.item.to_string())
    .with_arg("since", d.since.to_string())
    .with_arg("rfc", d.rfc.to_string());
    if let Some(r) = d.replacement {
        diag = diag.with_arg("replacement", r.to_string());
    }
    Some(diag)
}

/// Whether a deprecation can be migrated mechanically — i.e. whether `delulu fmt --migrate` has
/// anything to do for it. A `None` replacement means a human has to decide, and the tool says so
/// rather than guessing.
pub fn is_mechanical(d: &Deprecation) -> bool {
    d.replacement.is_some()
}

/// The `fmt --migrate` rewrite rules derived from a deprecation table: `(from, to)` pairs for the
/// mechanical entries only. Non-mechanical deprecations deliberately produce no rule.
pub fn migration_rules(table: &[Deprecation]) -> Vec<(&'static str, &'static str)> {
    table.iter().filter_map(|d| d.replacement.map(|r| (d.item, r))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span::new(0, 0, 3)
    }

    fn arg<'a>(d: &'a Diagnostic, key: &str) -> Option<&'a str> {
        d.args.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// The synthetic fixture (criterion 9): one mechanical deprecation and one that needs judgement.
    const FIXTURE: &[Deprecation] = &[
        Deprecation {
            item: "old_greet",
            kind: DeprKind::Name,
            since: "1.1",
            replacement: Some("greet"),
            rfc: "RFC-0002",
        },
        Deprecation {
            item: "console.print_line",
            kind: DeprKind::Method,
            since: "1.2",
            replacement: None,
            rfc: "RFC-0003",
        },
    ];

    #[test]
    fn a_deprecated_name_warns_and_names_its_replacement() {
        let d = warn(FIXTURE, "old_greet", span()).expect("old_greet is deprecated");
        assert_eq!(d.code, "DL1801");
        assert!(!d.is_error(), "a deprecation must be a WARNING — a failing build is a removal");
        assert!(d.message.contains("since 1.1"), "{}", d.message);
        assert!(d.message.contains("use `greet` instead"), "{}", d.message);
        assert!(d.message.contains("RFC-0002"), "every deprecation names its RFC: {}", d.message);
        assert_eq!(arg(&d, "replacement"), Some("greet"));
    }

    /// A deprecation with no mechanical replacement must say so honestly and point at the RFC,
    /// never invent a substitute.
    #[test]
    fn a_non_mechanical_deprecation_points_at_the_rfc_instead_of_guessing() {
        let d = warn(FIXTURE, "console.print_line", span()).expect("deprecated");
        assert!(d.message.contains("see RFC-0003 for the migration"), "{}", d.message);
        assert!(arg(&d, "replacement").is_none(), "no replacement may be fabricated");
    }

    /// THE SKIP-BRANCH CASE (house rule 3): a name that is NOT in the table must produce no
    /// warning. Without this, a lookup that matched everything would look like a working policy.
    #[test]
    fn an_undeprecated_name_never_warns() {
        assert!(warn(FIXTURE, "greet", span()).is_none());
        assert!(warn(FIXTURE, "", span()).is_none());
        assert!(warn(FIXTURE, "old_gree", span()).is_none(), "prefix must not match");
        assert!(warn(FIXTURE, "old_greet2", span()).is_none(), "suffix must not match");
    }

    /// The empty live table warns about nothing at all — the 1.0 state, asserted rather than
    /// assumed.
    #[test]
    fn the_live_registry_is_empty_at_1_0() {
        assert!(DEPRECATIONS.is_empty(), "nothing is deprecated at 1.0");
        for probe in ["greet", "old_greet", "console.println", "main"] {
            assert!(warn(DEPRECATIONS, probe, span()).is_none());
        }
    }

    /// Only mechanical deprecations become `fmt --migrate` rules; a judgement call must never be
    /// rewritten automatically.
    #[test]
    fn migration_rules_cover_exactly_the_mechanical_deprecations() {
        let rules = migration_rules(FIXTURE);
        assert_eq!(rules, vec![("old_greet", "greet")]);
        assert!(is_mechanical(&FIXTURE[0]));
        assert!(!is_mechanical(&FIXTURE[1]));
    }

    /// Every live entry is well-formed. Enforced now so the first real deprecation cannot land
    /// without its RFC.
    #[test]
    fn every_live_deprecation_names_an_rfc_and_a_version() {
        for d in DEPRECATIONS {
            assert!(!d.rfc.is_empty(), "`{}` has no RFC — deprecation is RFC-gated policy", d.item);
            assert!(!d.since.is_empty(), "`{}` has no since-version", d.item);
            assert!(!d.item.is_empty());
        }
    }
}
