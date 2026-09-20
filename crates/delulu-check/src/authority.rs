//! The whole-program authority report (spec §10.5) — the data behind `delulu authority`,
//! the flagship command that answers "what can this program actually do to my system?"
//!
//! For a `bin` program the report is computed from `main` outward over the call graph. Effects
//! are the union of every reachable function's declared row — which, by the boundary subset
//! property (T-Fn), equals `row(main)`. Scopes are runtime/manifest data (kind-vs-scope split,
//! Constitution §5.3): the checker fills kinds; the CLI enriches scopes from `delulu.toml`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_json::{json, Value};

use crate::check::CheckResult;
use crate::ty::{Effect, ResourceKind};

/// Manifest-derived scope information the CLI passes in (empty when there is no manifest).
#[derive(Default)]
pub struct ScopeInfo {
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    /// Pre-built `foreign_calls` JSON entries (spec §6, Stage 4 phase 4e). The CLI assembles these
    /// from the program's `foreign` blocks + manifest allowlist, because they mix static declaration
    /// data with manifest/grant data the checker's `CheckResult` does not carry. Empty when the
    /// program declares no foreign blocks — so the report stays byte-identical to Stage 3.
    pub foreign_calls: Vec<Value>,
}

impl ScopeInfo {
    pub fn for_kind(&self, k: ResourceKind) -> Vec<String> {
        match k {
            ResourceKind::FsRead => self.fs_read.clone(),
            ResourceKind::FsWrite => self.fs_write.clone(),
            ResourceKind::Http => self.net.clone(),
            ResourceKind::Console => vec!["stdio".to_string()],
            _ => vec![],
        }
    }
}

/// Compute the report as JSON. `program` is the module name; `scopes` comes from the manifest.
pub fn authority_report(program: &str, result: &CheckResult, scopes: &ScopeInfo) -> Value {
    // Reachable functions from `main` (or all functions if there is no main — e.g. a library).
    let reachable: BTreeSet<String> = if result.main_present {
        reachable_from(result, "main")
    } else {
        result.facts.keys().cloned().collect()
    };

    let mut effects: BTreeSet<Effect> = BTreeSet::new();
    let mut cap_kinds: BTreeSet<ResourceKind> = BTreeSet::new();
    let mut secrets: BTreeSet<String> = BTreeSet::new();
    for name in &reachable {
        if let Some(f) = result.facts.get(name) {
            effects.extend(f.effects.iter().cloned());
            cap_kinds.extend(f.cap_kinds.iter().cloned());
            secrets.extend(f.secret_names.iter().cloned());
        }
    }

    // Group capability kinds; Declassify is reported as an effect, not a wielded resource line.
    // `ForeignLoad`/`Python` are disclosed under the "outside the proof" separator (`foreign_calls`),
    // not as ordinary capability lines — the foreign section is the single place the proof's holes
    // are enumerated (spec §6).
    //
    // `PluginHost` used to be skipped here too, and that was right while it could not be granted: a
    // capability line for a flag that did not exist would have sent an operator looking for one. P2
    // (D-V2-27) gives it `--grant plugin=PATH`, so it is an ordinary capability line now, and
    // `required_grants` can name the flag. The sentence `INSTALL.txt` makes — "run `delulu authority`
    // and the required grants are the list it prints" — is only true if every grantable capability
    // appears here.
    let mut capabilities = Vec::new();
    for k in &cap_kinds {
        if matches!(k, ResourceKind::Declassify | ResourceKind::ForeignLoad | ResourceKind::Python) {
            continue;
        }
        capabilities.push(json!({
            "kind": k.name(),
            "scopes": scopes.for_kind(*k),
        }));
    }

    let mut pure_functions: Vec<String> =
        result.facts.iter().filter(|(_, f)| f.pure).map(|(n, _)| n.clone()).collect();
    pure_functions.sort();

    // A stable, sorted effect list.
    let effect_names: Vec<&str> = effects.iter().map(|e| e.name()).collect();
    // Deduplicate user effects while keeping order stable.
    let mut seen = BTreeMap::new();
    for n in &effect_names {
        *seen.entry(*n).or_insert(0) += 1;
    }
    let effects_json: Vec<&str> = seen.keys().copied().collect();

    json!({
        "program": program,
        "effects": effects_json,
        "capabilities": capabilities,
        "secrets": secrets.into_iter().collect::<Vec<_>>(),
        "foreign_calls": scopes.foreign_calls.clone(),
        "contained_plugins": [],
        "pure_functions": pure_functions,
    })
}

fn reachable_from(result: &CheckResult, root: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_string());
    while let Some(name) = queue.pop_front() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(f) = result.facts.get(&name) {
            for callee in &f.callees {
                if !seen.contains(callee) {
                    queue.push_back(callee.clone());
                }
            }
        }
    }
    seen
}
