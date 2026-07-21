//! Phase 5a — the `⊑` attenuation lattice (spec §A.3, R-7; Constitution §5.16 law 1).
//!
//! This is the mathematical heart of custody: a single source of truth for `authority ⊑ authority`
//! that returns either "ok" or the **computed intersection**. The intersection is what DL0802's
//! exact repair carries (spec §8) — and it is *never wider than either side*, which is the property
//! that makes attenuation sound: a repair can only ever narrow a request, never widen it.

use std::collections::{BTreeMap, BTreeSet};

use delulu_check::Effect;

use crate::device_scope::{self, DeviceScope};
use crate::path;

/// The scope data carried by a grant (spec §3.1 `scopes`). Every dimension is a `BTreeSet<String>`
/// so serialization is canonical (sorted, deduplicated) regardless of insertion order.
///
/// Subset semantics per dimension (ruling 4):
/// - `fs_read` / `fs_write` — path descendant-or-equal (the [`crate::path`] lattice).
/// - `net` / `secrets` / `declassify` / `foreign_c` / `foreign_python` — EXACT-STRING name-set
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
}

/// A grant's authority: an effect set plus per-dimension scopes (spec §3.1 `authority`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Authority {
    pub effects: BTreeSet<Effect>,
    pub scopes: Scopes,
}

impl Authority {
    /// A convenience builder from effect names + scope lists (mostly for tests/CLI sugar).
    pub fn new(effects: impl IntoIterator<Item = Effect>, scopes: Scopes) -> Authority {
        Authority { effects: effects.into_iter().collect(), scopes }
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
        && child.scopes.declassify.is_subset(&parent.scopes.declassify)
        && child.scopes.foreign_c.is_subset(&parent.scopes.foreign_c)
        && child.scopes.foreign_python.is_subset(&parent.scopes.foreign_python)
        && device_scope::all_within(&child.scopes.device, &parent.scopes.device);
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
}
