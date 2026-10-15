//! Phase 5a — the `⊑` attenuation lattice (spec §A.3, R-7; Constitution §5.16 law 1).
//!
//! This is the mathematical heart of custody: a single source of truth for `authority ⊑ authority`
//! that returns either "ok" or the **computed intersection**. The intersection is what DL0802's
//! exact repair carries (spec §8) — and it is *never wider than either side*, which is the property
//! that makes attenuation sound: a repair can only ever narrow a request, never widen it.

use std::collections::{BTreeMap, BTreeSet};

use delulu_check::Effect;

use crate::budget_scope::{self, BudgetScope};
use crate::device_scope::{self, DeviceScope};
use crate::path;

/// The scope data carried by a grant (spec §3.1 `scopes`). Every dimension is a `BTreeSet<String>`
/// so serialization is canonical (sorted, deduplicated) regardless of insertion order.
///
/// Subset semantics per dimension (ruling 4):
/// - `fs_read` / `fs_write` — path descendant-or-equal (the [`crate::path`] lattice).
/// - `net` / `secrets` / `secret_to` / `declassify` / `foreign_c` / `foreign_python` — EXACT-STRING name-set
///   subset. No pattern implication (`numpy.*` ⊒ `numpy.linalg`) in v0.5: conservative is sound.
///   Pattern-aware subsumption is a possible post-1.0 refinement.
/// - `device` — RFC 0001 F1: exact device name plus **interval containment** on the envelope (the
///   [`crate::device_scope`] lattice). Device names are exact-only for the same reason the name
///   dimensions are: conservative is sound, and widening later is additive while narrowing is not.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Scopes {
    pub fs_read: BTreeSet<String>,
    pub fs_write: BTreeSet<String>,
    pub net: BTreeSet<String>,
    pub secrets: BTreeSet<String>,
    pub declassify: BTreeSet<String>,
    pub foreign_c: BTreeSet<String>,
    pub foreign_python: BTreeSet<String>,
    /// `device path -> granted envelope`. A `BTreeMap` (not a set) because one device has one
    /// envelope per grant: two envelopes for the same device would be an ambiguity the enforcement
    /// path would have to resolve, and resolving it silently is how a widening gets in.
    pub device: BTreeMap<String, DeviceScope>,
    /// PS-B-05 (D-V2-08): the resource budget a holder's runs may consume — memory and processor
    /// time, ordered componentwise ([`crate::budget_scope`]). `None` is the top of the dimension: a
    /// grant nobody budgeted, which is every grant written before this field existed.
    pub budget: Option<BudgetScope>,
    /// P9-04 (d), D-V2-133: each secret's AUDIENCE — the origins its bytes may be sent to, as `NAME@host:port`
    /// pairs in the egress client's canonical spelling (`parse_audience`: the host lowercased, the port always
    /// named). An exact-set dimension like `secrets`: `⊑` is inclusion, `⊓` intersection. A secret with no pair
    /// is the bottom — sent nowhere. A pair is meaningful only beside its name in `secrets`, so
    /// [`Scopes::canonicalized`] drops one whose name is not there (it grants reach to nothing): every node is
    /// well-formed, and the meet of two well-formed authorities is well-formed.
    pub secret_to: BTreeSet<String>,
}

/// The secret a `secret_to` pair names — the text before its first `@` (a secret's name holds none, D-V2-131).
/// `None` for a pair with no `@`, which names nothing.
pub fn secret_to_name(pair: &str) -> Option<&str> {
    pair.split_once('@').map(|(name, _)| name).filter(|n| !n.is_empty())
}

/// A grant's authority: an effect set plus per-dimension scopes (spec §3.1 `authority`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Authority {
    pub effects: BTreeSet<Effect>,
    pub scopes: Scopes,
}

impl Scopes {
    /// Replace both path dimensions with their canonical representatives (P17-F1/F2/F3).
    ///
    /// Only `fs_read`/`fs_write` are affected: the name dimensions compare by exact string, so they
    /// have no equivalence classes to collapse, and `device` names are exact-only for the same
    /// reason. [`path::canonicalize_set`] preserves the covered region exactly, so this can neither
    /// widen nor narrow the scope — it only removes the choice of spelling and the redundant
    /// elements that were already inside another.
    pub fn canonicalized(&self) -> Scopes {
        let canon = |d: &BTreeSet<String>| path::canonicalize_set(d);
        Scopes {
            fs_read: canon(&self.fs_read),
            fs_write: canon(&self.fs_write),
            net: self.net.clone(),
            secrets: self.secrets.clone(),
            declassify: self.declassify.clone(),
            foreign_c: self.foreign_c.clone(),
            foreign_python: self.foreign_python.clone(),
            device: self.device.clone(),
            budget: self.budget,
            secret_to: self.well_formed_secret_to(),
        }
    }

    /// `secret_to` without a pair whose name `secrets` does not hold (P9-04 (d)): such a pair grants reach to
    /// nothing, and keeping it would make two authorities that grant the same thing unequal under `⊑`.
    fn well_formed_secret_to(&self) -> BTreeSet<String> {
        self.secret_to
            .iter()
            .filter(|p| secret_to_name(p).is_some_and(|n| self.secrets.contains(n)))
            .cloned()
            .collect()
    }
}

impl Authority {
    /// A convenience builder from effect names + scope lists (mostly for tests/CLI sugar).
    ///
    /// Canonicalizes, so an authority built through the documented constructor is canonical by
    /// construction. Struct-literal construction bypasses this — which is exactly what
    /// `order_laws.rs` exploits to keep the raw-spelling preorder finding visible.
    pub fn new(effects: impl IntoIterator<Item = Effect>, scopes: Scopes) -> Authority {
        Authority { effects: effects.into_iter().collect(), scopes: scopes.canonicalized() }
    }

    /// This authority with every path scope replaced by its canonical representative.
    ///
    /// **Why this exists (P17-F1/F3).** `⊑` is defined through `path::resolve`, which is not
    /// injective — `./data`, `data`, `./data/` and `.\data` are one path under four names. A
    /// relation defined through a non-injective function is a *preorder*, never a partial order, so
    /// `⊑` on raw spellings cannot be antisymmetric no matter how the comparison is written; and
    /// the same logical grant, spelled two ways, produced two different audit-chain hashes. Neither
    /// is fixable inside the comparison. Both dissolve once the system stores one representative
    /// per class, which is what this does.
    pub fn canonicalized(&self) -> Authority {
        Authority { effects: self.effects.clone(), scopes: self.scopes.canonicalized() }
    }

    /// Is every path scope already its own canonical representative?
    ///
    /// Used by the broker's ingest gate to assert the invariant holds at the boundary rather than
    /// trusting that every caller remembered to canonicalize.
    pub fn is_canonical(&self) -> bool {
        let s = &self.scopes;
        path::canonicalize_set(&s.fs_read) == s.fs_read
            && path::canonicalize_set(&s.fs_write) == s.fs_write
            && s.well_formed_secret_to() == s.secret_to
    }

    /// Canonical JSON (sorted keys, sorted arrays). Effects render by their stable `name()` since
    /// `delulu_check::Effect` is not `Serialize` (and we must not modify that crate — ruling 6).
    pub fn to_json(&self) -> serde_json::Value {
        let effects: Vec<&str> = self.effects.iter().map(|e| e.name()).collect();
        let s = &self.scopes;
        // Keys inserted in sorted order so output is byte-canonical even if serde_json is ever
        // built with `preserve_order`.
        let mut scopes = serde_json::json!({
            "declassify": to_vec(&s.declassify),
            "fs.read": to_vec(&s.fs_read),
            "fs.write": to_vec(&s.fs_write),
            "foreign.c": to_vec(&s.foreign_c),
            "foreign.python": to_vec(&s.foreign_python),
            "net": to_vec(&s.net),
            "secrets": to_vec(&s.secrets),
        });
        // `device` is emitted ONLY when non-empty, unlike the seven dimensions above which always
        // emit (possibly empty) arrays. This is deliberate and is a compatibility decision, not an
        // inconsistency: this value is embedded in the `authority` field of hash-chained audit
        // records, so always-emitting `"device": []` would change the canonical JSON — and thus the
        // chain hash — of every authority that has no device scope, including every record already
        // written. Omitting when empty follows the same discipline `AuditRecord::body_value` already
        // uses for absent optionals, and keeps existing records byte-identical.
        if !s.device.is_empty() {
            let devices: Vec<String> = s.device.values().map(|d| d.to_grant_string()).collect();
            scopes.as_object_mut().expect("json! built an object").insert("device".into(), serde_json::json!(devices));
        }
        // PS-B-05: `budget` follows `device`'s rule for the same reason — emitted ONLY when present, so
        // every authority without one keeps the canonical JSON, the chain hash and the certificate
        // bytes it had before this dimension existed. A one-element array of the canonical grant
        // string, so every scope stays an array of strings to a reader that walks them.
        if let Some(b) = &s.budget {
            scopes
                .as_object_mut()
                .expect("json! built an object")
                .insert("budget".into(), serde_json::json!([b.to_grant_string()]));
        }
        // P9-04 (d): `secret_to` follows the same rule — emitted ONLY when non-empty, so every authority whose
        // secrets name no audience keeps its canonical JSON, chain hash and certificate bytes.
        if !s.secret_to.is_empty() {
            scopes.as_object_mut().expect("json! built an object").insert("secret_to".into(), serde_json::json!(to_vec(&s.secret_to)));
        }
        serde_json::json!({ "effects": effects, "scopes": scopes })
    }

    /// A compact one-line rendering for diagnostic messages.
    pub fn render_compact(&self) -> String {
        let effects: Vec<&str> = self.effects.iter().map(|e| e.name()).collect();
        let s = &self.scopes;
        let mut parts = vec![format!("effects={{{}}}", effects.join(","))];
        for (label, dim) in [
            ("fs.read", &s.fs_read),
            ("fs.write", &s.fs_write),
            ("net", &s.net),
            ("secrets", &s.secrets),
            ("secret_to", &s.secret_to),
            ("declassify", &s.declassify),
            ("foreign.c", &s.foreign_c),
            ("foreign.python", &s.foreign_python),
        ] {
            if !dim.is_empty() {
                parts.push(format!("{label}=[{}]", to_vec(dim).join(",")));
            }
        }
        if !s.device.is_empty() {
            let devices: Vec<String> = s.device.values().map(|d| d.to_grant_string()).collect();
            parts.push(format!("device=[{}]", devices.join(" ")));
        }
        if let Some(b) = &s.budget {
            parts.push(format!("budget=[{}]", b.to_grant_string()));
        }
        parts.join(" ")
    }

    /// The **meet** (`⊓`) of two authorities: effect-set ∩, and per-dimension scope ∩ (path meet
    /// for fs, exact-set ∩ for the name dimensions). Every element of the result is within BOTH
    /// inputs, so the meet is never wider than either side (spec §8 "the repair never widens").
    pub fn intersect(&self, other: &Authority) -> Authority {
        Authority {
            effects: self.effects.intersection(&other.effects).cloned().collect(),
            scopes: Scopes {
                fs_read: path::intersect_path_sets(&self.scopes.fs_read, &other.scopes.fs_read),
                fs_write: path::intersect_path_sets(&self.scopes.fs_write, &other.scopes.fs_write),
                net: name_intersect(&self.scopes.net, &other.scopes.net),
                secrets: name_intersect(&self.scopes.secrets, &other.scopes.secrets),
                declassify: name_intersect(&self.scopes.declassify, &other.scopes.declassify),
                foreign_c: name_intersect(&self.scopes.foreign_c, &other.scopes.foreign_c),
                foreign_python: name_intersect(
                    &self.scopes.foreign_python,
                    &other.scopes.foreign_python,
                ),
                device: device_scope::intersect_device_sets(&self.scopes.device, &other.scopes.device),
                budget: budget_scope::meet(self.scopes.budget.as_ref(), other.scopes.budget.as_ref()),
                secret_to: name_intersect(&self.scopes.secret_to, &other.scopes.secret_to),
            },
        }
    }
}

fn to_vec(s: &BTreeSet<String>) -> Vec<String> {
    s.iter().cloned().collect() // BTreeSet iterates in sorted order → canonical
}

fn name_intersect(a: &BTreeSet<String>, b: &BTreeSet<String>) -> BTreeSet<String> {
    a.intersection(b).cloned().collect()
}

/// The `⊑` check (spec §A.3): is `child ⊑ parent`?
///
/// Returns `Ok(())` when the child's authority is an attenuation of the parent's. On failure,
/// returns `Err(intersection)` where the intersection is `child ⊓ parent` — the exact,
/// never-widening repair DL0802 hands back (spec §8). `child` first so the intersection is computed
/// against the caller's *request*, but note `⊓` is symmetric so the value is identical either way.
///
/// The `Err` payload is deliberately the whole [`Authority`] repair value (not an error code): it is
/// consumed immediately by the caller to build DL0802, so `clippy::result_large_err` is intentional.
#[allow(clippy::result_large_err)]
pub fn attenuation_check(child: &Authority, parent: &Authority) -> Result<(), Authority> {
    let ok = child.effects.is_subset(&parent.effects)
        && path::all_within(&child.scopes.fs_read, &parent.scopes.fs_read)
        && path::all_within(&child.scopes.fs_write, &parent.scopes.fs_write)
        && child.scopes.net.is_subset(&parent.scopes.net)
        && child.scopes.secrets.is_subset(&parent.scopes.secrets)
        && child.scopes.secret_to.is_subset(&parent.scopes.secret_to)
        && child.scopes.declassify.is_subset(&parent.scopes.declassify)
        && child.scopes.foreign_c.is_subset(&parent.scopes.foreign_c)
        && child.scopes.foreign_python.is_subset(&parent.scopes.foreign_python)
        && device_scope::all_within(&child.scopes.device, &parent.scopes.device)
        && budget_scope::within(child.scopes.budget.as_ref(), parent.scopes.budget.as_ref());
    if ok {
        Ok(())
    } else {
        Err(child.intersect(parent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }
    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// Build an authority quickly: effects, then (dim, values) pairs.
    fn auth(effects: &[&str], scopes: Scopes) -> Authority {
        Authority { effects: eff(effects), scopes }
    }

    fn budget(mib: u64, cpu_seconds: u64) -> BudgetScope {
        BudgetScope { memory_bytes: mib * 1024 * 1024, cpu_seconds }
    }

    /// The compatibility promise PS-B-05 made: an authority WITHOUT a budget serializes exactly as it
    /// did before the dimension existed, so no hash-chained audit record and no signed certificate
    /// written earlier changes its bytes.
    #[test]
    fn an_unbudgeted_authority_serializes_exactly_as_before_the_budget_existed() {
        let a = auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() });
        assert_eq!(
            a.to_json().to_string(),
            r#"{"effects":["Read"],"scopes":{"declassify":[],"foreign.c":[],"foreign.python":[],"fs.read":["./data"],"fs.write":[],"net":[],"secrets":[]}}"#
        );
        let b = auth(&["Read"], Scopes { budget: Some(budget(256, 60)), ..Default::default() });
        assert!(b.to_json().to_string().contains(r#""budget":["mem=268435456,cpu=60"]"#), "{}", b.to_json());
    }

    // A table row: (name, child, parent, expected). expected = None → ok; Some → the intersection.
    struct Row {
        name: &'static str,
        child: Authority,
        parent: Authority,
        expected: Option<Authority>,
    }

    #[test]
    fn attenuation_table_covers_every_dimension() {
        let rows = vec![
            Row {
                name: "identical authorities attenuate",
                child: auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() }),
                parent: auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "effect subset ok",
                child: auth(&["Read"], Scopes::default()),
                parent: auth(&["Read", "Net"], Scopes::default()),
                expected: None,
            },
            Row {
                name: "effect superset rejected -> effect intersection",
                child: auth(&["Read", "Net"], Scopes::default()),
                parent: auth(&["Read"], Scopes::default()),
                expected: Some(auth(&["Read"], Scopes::default())),
            },
            Row {
                name: "fs subtree ok (./data/sub ⊑ ./data)",
                child: auth(&["Read"], Scopes { fs_read: names(&["./data/sub"]), ..Default::default() }),
                parent: auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "fs widening rejected (./data ⋢ ./data/sub) -> narrower survives",
                child: auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() }),
                parent: auth(&["Read"], Scopes { fs_read: names(&["./data/sub"]), ..Default::default() }),
                expected: Some(auth(&["Read"], Scopes { fs_read: names(&["./data/sub"]), ..Default::default() })),
            },
            Row {
                name: "fs sibling-prefix rejected (./database ⋢ ./data) -> empty fs meet",
                child: auth(&["Read"], Scopes { fs_read: names(&["./database"]), ..Default::default() }),
                parent: auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() }),
                expected: Some(auth(&["Read"], Scopes::default())),
            },
            Row {
                name: "fs.write dimension checked independently",
                child: auth(&["Write"], Scopes { fs_write: names(&["./out/logs"]), ..Default::default() }),
                parent: auth(&["Write"], Scopes { fs_write: names(&["./out"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "net host-set subset ok",
                child: auth(&["Net"], Scopes { net: names(&["api.example.com"]), ..Default::default() }),
                parent: auth(&["Net"], Scopes { net: names(&["api.example.com", "cdn.example.com"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "net host not in parent rejected -> host intersection",
                child: auth(&["Net"], Scopes { net: names(&["evil.example.com"]), ..Default::default() }),
                parent: auth(&["Net"], Scopes { net: names(&["api.example.com"]), ..Default::default() }),
                expected: Some(auth(&["Net"], Scopes::default())),
            },
            Row {
                name: "secrets name-set subset ok",
                child: auth(&[], Scopes { secrets: names(&["API_KEY"]), ..Default::default() }),
                parent: auth(&[], Scopes { secrets: names(&["API_KEY", "DB_PW"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "secrets extra name rejected -> secret intersection",
                child: auth(&[], Scopes { secrets: names(&["API_KEY", "ROOT_PW"]), ..Default::default() }),
                parent: auth(&[], Scopes { secrets: names(&["API_KEY"]), ..Default::default() }),
                expected: Some(auth(&[], Scopes { secrets: names(&["API_KEY"]), ..Default::default() })),
            },
            Row {
                name: "secret_to pair-set subset ok (fewer origins for the same secret)",
                child: auth(&[], Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443"]), ..Default::default() }),
                parent: auth(
                    &[],
                    Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443", "K@b.com:8443"]), ..Default::default() },
                ),
                expected: None,
            },
            Row {
                name: "secret_to: another origin rejected -> pair intersection (the secret kept, sent nowhere new)",
                child: auth(
                    &[],
                    Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443", "K@evil.com:443"]), ..Default::default() },
                ),
                parent: auth(&[], Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443"]), ..Default::default() }),
                expected: Some(auth(&[], Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443"]), ..Default::default() })),
            },
            Row {
                name: "secret_to: the bottom (no pair) is within any audience",
                child: auth(&[], Scopes { secrets: names(&["K"]), ..Default::default() }),
                parent: auth(&[], Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "declassify name-set subset ok",
                child: auth(&["Declassify"], Scopes { declassify: names(&["API_KEY"]), ..Default::default() }),
                parent: auth(&["Declassify"], Scopes { declassify: names(&["API_KEY"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "foreign.c exact subset (no path implication)",
                child: auth(&["ForeignCall"], Scopes { foreign_c: names(&["mathlib"]), ..Default::default() }),
                parent: auth(&["ForeignCall"], Scopes { foreign_c: names(&["mathlib", "z"]), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "foreign.python exact-string only (numpy.* does NOT imply numpy.linalg)",
                child: auth(&["ForeignCall"], Scopes { foreign_python: names(&["numpy.linalg"]), ..Default::default() }),
                parent: auth(&["ForeignCall"], Scopes { foreign_python: names(&["numpy.*"]), ..Default::default() }),
                expected: Some(auth(&["ForeignCall"], Scopes::default())),
            },
            Row {
                name: "escape via .. is not a descendant -> empty fs meet",
                child: auth(&["Read"], Scopes { fs_read: names(&["./data/../secret"]), ..Default::default() }),
                parent: auth(&["Read"], Scopes { fs_read: names(&["./data"]), ..Default::default() }),
                expected: Some(auth(&["Read"], Scopes::default())),
            },
            Row {
                name: "multi-dimension failure meets on every dimension at once",
                child: auth(
                    &["Read", "Net", "Write"],
                    Scopes {
                        fs_read: names(&["./data"]),
                        net: names(&["a.com", "b.com"]),
                        ..Default::default()
                    },
                ),
                parent: auth(
                    &["Read", "Net"],
                    Scopes {
                        fs_read: names(&["./data/sub"]),
                        net: names(&["a.com"]),
                        ..Default::default()
                    },
                ),
                expected: Some(auth(
                    &["Read", "Net"],
                    Scopes {
                        fs_read: names(&["./data/sub"]),
                        net: names(&["a.com"]),
                        ..Default::default()
                    },
                )),
            },
            // PS-B-05: the budget dimension, componentwise, with `None` as the top.
            Row {
                name: "a smaller budget attenuates",
                child: auth(&[], Scopes { budget: Some(budget(256, 60)), ..Default::default() }),
                parent: auth(&[], Scopes { budget: Some(budget(1024, 300)), ..Default::default() }),
                expected: None,
            },
            Row {
                name: "a budgeted child under an unbudgeted parent attenuates (None is the top)",
                child: auth(&[], Scopes { budget: Some(budget(1024, 300)), ..Default::default() }),
                parent: auth(&[], Scopes::default()),
                expected: None,
            },
            Row {
                name: "more memory than delegated is refused -> the componentwise minimum",
                child: auth(&[], Scopes { budget: Some(budget(2048, 60)), ..Default::default() }),
                parent: auth(&[], Scopes { budget: Some(budget(1024, 300)), ..Default::default() }),
                expected: Some(auth(&[], Scopes { budget: Some(budget(1024, 60)), ..Default::default() })),
            },
            Row {
                name: "an UNbudgeted child under a budgeted parent is a widening -> the parent's budget",
                child: auth(&[], Scopes::default()),
                parent: auth(&[], Scopes { budget: Some(budget(1024, 300)), ..Default::default() }),
                expected: Some(auth(&[], Scopes { budget: Some(budget(1024, 300)), ..Default::default() })),
            },
        ];

        for row in rows {
            let got = attenuation_check(&row.child, &row.parent);
            match (&row.expected, &got) {
                (None, Ok(())) => {}
                (Some(want), Err(inter)) => {
                    assert_eq!(inter, want, "intersection mismatch for `{}`", row.name);
                }
                (None, Err(inter)) => {
                    panic!("`{}` should attenuate but was rejected with {inter:?}", row.name)
                }
                (Some(_), Ok(())) => panic!("`{}` should be rejected but attenuated", row.name),
            }
        }
    }

    #[test]
    fn intersection_is_never_wider_than_either_side() {
        // Property spot-check: the meet is ⊑ both operands.
        let a = auth(
            &["Read", "Net"],
            Scopes { fs_read: names(&["./data"]), net: names(&["a.com"]), ..Default::default() },
        );
        let b = auth(
            &["Read", "Write"],
            Scopes { fs_read: names(&["./data/sub"]), net: names(&["b.com"]), ..Default::default() },
        );
        let m = a.intersect(&b);
        assert!(attenuation_check(&m, &a).is_ok(), "meet must be ⊑ a");
        assert!(attenuation_check(&m, &b).is_ok(), "meet must be ⊑ b");
        assert!(m.effects.is_subset(&a.effects) && m.effects.is_subset(&b.effects));
    }

    #[test]
    fn meet_is_symmetric() {
        let a = auth(&["Read", "Net"], Scopes { fs_read: names(&["./data"]), ..Default::default() });
        let b = auth(&["Read"], Scopes { fs_read: names(&["./data/sub"]), ..Default::default() });
        assert_eq!(a.intersect(&b), b.intersect(&a));
    }

    #[test]
    fn empty_child_attenuates_anything() {
        let empty = Authority::default();
        let parent = auth(&["Read", "Net"], Scopes { fs_read: names(&["./data"]), ..Default::default() });
        assert!(attenuation_check(&empty, &parent).is_ok());
    }

    /// P9-04 (d), D-V2-133: a `secret_to` pair whose name the authority's `secrets` does not hold grants reach to
    /// nothing — canonicalization drops it (so `⊑` never refuses a delegation for a pair that grants nothing, and two
    /// authorities that grant the same thing are equal), `is_canonical` sees one kept, and the meet keeps every
    /// authority well-formed. An authority with no pair serializes as it did before the dimension existed.
    #[test]
    fn a_secret_to_pair_lives_only_beside_its_name() {
        let wf = Authority::new(
            [],
            Scopes { secrets: names(&["K"]), secret_to: names(&["K@a.com:443", "J@a.com:443", "noname", "@a.com:443"]), ..Default::default() },
        );
        assert_eq!(wf.scopes.secret_to, names(&["K@a.com:443"]), "an orphan pair, a pair with no `@` and one with no name are dropped");
        assert!(wf.is_canonical());
        let raw = Authority { effects: Default::default(), scopes: Scopes { secret_to: names(&["K@a.com:443"]), ..Default::default() } };
        assert!(!raw.is_canonical(), "a kept orphan is not canonical");
        assert_eq!(raw.canonicalized(), Authority::default());
        let other = Authority::new(
            [],
            Scopes { secrets: names(&["K", "J"]), secret_to: names(&["K@a.com:443", "J@b.com:443"]), ..Default::default() },
        );
        let m = wf.intersect(&other);
        assert!(m.is_canonical() && attenuation_check(&m, &wf).is_ok() && attenuation_check(&m, &other).is_ok());
        assert_eq!(secret_to_name("K@a.com:443"), Some("K"));
        assert_eq!((secret_to_name("K"), secret_to_name("@a.com:443")), (None, None));
        let none = Authority::new([], Scopes { secrets: names(&["K"]), ..Default::default() });
        assert!(!none.to_json().to_string().contains("secret_to"), "no pair, no key: {}", none.to_json());
        assert!(wf.to_json()["scopes"]["secret_to"] == serde_json::json!(["K@a.com:443"]), "{}", wf.to_json());
        assert!(wf.render_compact().contains("secret_to=[K@a.com:443]"), "{}", wf.render_compact());
    }
}
