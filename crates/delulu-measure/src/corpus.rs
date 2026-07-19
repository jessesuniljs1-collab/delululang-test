//! Study A's corpus: real-shaped packages with real dependency graphs (Stage 9d, spec §3.1).
//!
//! The corpus is **generated deterministically** rather than hand-written. Not for convenience —
//! for honesty. A hand-assembled corpus invites the accusation that the packages were shaped, one
//! at a time, into the ones the mechanism happens to catch. A generator with no randomness at all
//! produces the same 25 packages on every machine, and the shape of every graph is visible in this
//! file rather than distributed across fifty files nobody reads.
//!
//! ## What "real-shaped" means here, precisely
//! Five archetypes named in the spec (a CLI tool, a parser, an HTTP client stack, a static site
//! generator, an agent-tool server), each a chain of five packages: the application and four
//! libraries beneath it. Depth 4 from application to leaf, which is the spec's floor.
//!
//! These are *shaped* like real packages — layered, each layer calling the next, authority declared
//! per package and pinned per dependency — but they are not real programs doing real work. Study A
//! measures whether the **mechanism** catches an authority change anywhere in a graph; it does not
//! claim the corpus is representative of production code, and the write-up says so.

/// One package in the corpus.
pub struct Package {
    pub name: String,
    /// Directory name under the chain root.
    pub dir: String,
    /// `bin` for the application at the head of a chain, `lib` beneath it.
    pub is_app: bool,
    /// The single dependency beneath this package, if any.
    pub depends_on: Option<String>,
    /// Effects this package's own manifest declares.
    pub effects: Vec<String>,
    pub manifest: String,
    pub source_path: String,
    pub source: String,
}

/// One dependency chain: an application and the libraries beneath it.
pub struct Chain {
    pub name: String,
    /// What kind of software this chain is shaped like.
    pub archetype: String,
    pub packages: Vec<Package>,
}

impl Chain {
    /// The application at the head of the chain.
    pub fn app(&self) -> &Package {
        &self.packages[0]
    }
    /// The libraries beneath the application — every injection site.
    pub fn libs(&self) -> &[Package] {
        &self.packages[1..]
    }
    pub fn depth(&self) -> usize {
        self.packages.len() - 1
    }
}

/// The five archetypes: (chain name, archetype label, the leaf's job).
const ARCHETYPES: &[(&str, &str, &str)] = &[
    ("cliwc", "CLI tool", "counts words"),
    ("parsecfg", "parser", "splits key/value lines"),
    ("httpstack", "HTTP client stack", "builds a request line"),
    ("sitegen", "static site generator", "renders a slug"),
    ("agenttool", "agent-tool server", "formats a tool result"),
];

/// The layer names beneath an application, outermost first.
const LAYERS: &[&str] = &["api", "core", "util", "base"];

/// Build the whole corpus: 5 chains x 5 packages = 25 packages, each chain depth 4.
pub fn build() -> Vec<Chain> {
    ARCHETYPES.iter().map(|(name, archetype, job)| chain(name, archetype, job)).collect()
}

fn chain(name: &str, archetype: &str, job: &str) -> Chain {
    let pkg_name = |layer: Option<&str>| match layer {
        None => format!("{name}_app"),
        Some(l) => format!("{name}_{l}"),
    };

    // An import brings names into scope UNQUALIFIED, so every layer's entry point needs a
    // distinct name or the graph collides with itself (DL0302).
    let entry_name = |layer: &str| format!("{name}_{layer}_entry");

    let mut packages = Vec::new();

    // The application: depends on the outermost library, performs Write (it prints its result).
    let app_dep = pkg_name(Some(LAYERS[0]));
    packages.push(Package {
        name: pkg_name(None),
        dir: format!("{name}_app"),
        is_app: true,
        depends_on: Some(app_dep.clone()),
        effects: vec!["Write".into()],
        manifest: manifest(
            &pkg_name(None),
            "bin",
            &["Write"],
            Some((&app_dep, &format!("../{app_dep}"), &[])),
        ),
        source_path: "src/main.delulu".into(),
        source: format!(
            "module {}\nimport {app_dep}\n\n\
             fn main(root: Root) ! {{Write}} {{\n  \
             let out = root.console()\n  \
             out.println({}(root, \"input\"))\n\
             }}\n",
            pkg_name(None),
            entry_name(LAYERS[0])
        ),
    });

    // The libraries: each calls the next, all pure. Purity is the point — an effect appearing
    // anywhere beneath the application is exactly the change Study A injects.
    for (i, layer) in LAYERS.iter().enumerate() {
        let me = pkg_name(Some(layer));
        let below = LAYERS.get(i + 1).map(|l| pkg_name(Some(l)));
        let (body, dep_line) = match (&below, LAYERS.get(i + 1)) {
            (Some(b), Some(next_layer)) => (
                format!("  {}(root, x)\n", entry_name(next_layer)),
                Some((b.clone(), format!("../{b}"))),
            ),
            // The leaf does the actual work of the archetype.
            _ => ("  x.trim()\n".to_string(), None),
        };
        let imports = below.as_ref().map(|b| format!("import {b}\n")).unwrap_or_default();
        packages.push(Package {
            name: me.clone(),
            dir: me.clone(),
            is_app: false,
            depends_on: below.clone(),
            effects: vec![],
            manifest: manifest(
                &me,
                "lib",
                &[],
                dep_line.as_ref().map(|(n, p)| (n.as_str(), p.as_str(), &[][..])),
            ),
            source_path: "src/lib.delulu".into(),
            // Every layer receives `root` and passes it down. This is a common real-world shape —
            // and an anti-pattern the authority report is designed to expose: a library holding
            // `Root` can derive any capability it likes. Study A measures whether a library that
            // *starts* using that latitude is caught. Deriving is pure, so these stay pure until
            // something is injected.
            source: format!(
                "module {me}\n{imports}\n\
                 // {archetype}: this layer {}\n\
                 pub fn {}(root: Root, x: Str) -> Str {{\n{body}}}\n",
                if below.is_some() { "delegates downward" } else { job },
                entry_name(layer)
            ),
        });
    }

    Chain { name: name.to_string(), archetype: archetype.to_string(), packages }
}

/// A manifest with a declared authority and an optional pinned dependency.
fn manifest(name: &str, kind: &str, effects: &[&str], dep: Option<(&str, &str, &[&str])>) -> String {
    let eff = effects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
    let mut s = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nkind = \"{kind}\"\n\n\
         [authority]\neffects = [{eff}]\n"
    );
    if let Some((dname, dpath, deffects)) = dep {
        let de = deffects.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", ");
        s.push_str(&format!(
            "\n[dependencies]\n{dname} = {{ path = \"{dpath}\", authority = {{ effects = [{de}] }} }}\n"
        ));
    }
    s
}

/// Write a chain to disk under `root/<chain.name>/`.
pub fn write_chain(root: &std::path::Path, chain: &Chain) -> std::io::Result<()> {
    for p in &chain.packages {
        let dir = root.join(&chain.name).join(&p.dir);
        std::fs::create_dir_all(dir.join("src"))?;
        std::fs::write(dir.join("delulu.toml"), &p.manifest)?;
        std::fs::write(dir.join(&p.source_path), &p.source)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The corpus meets the spec's floors: >= 25 packages, dependency depth >= 4 (§3.1).
    #[test]
    fn the_corpus_meets_the_specified_floors() {
        let chains = build();
        let total: usize = chains.iter().map(|c| c.packages.len()).sum();
        assert!(total >= 25, "spec §3.1 requires >= 25 packages, corpus has {total}");
        for c in &chains {
            assert!(c.depth() >= 4, "chain `{}` has depth {}, spec requires >= 4", c.name, c.depth());
        }
        assert_eq!(chains.len(), 5, "five archetypes");
    }

    /// Package names are unique across the whole corpus — a duplicate would make a dependency
    /// ambiguous (DL1008) and quietly change what the study is measuring.
    #[test]
    fn every_package_name_is_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for c in build() {
            for p in c.packages {
                assert!(seen.insert(p.name.clone()), "duplicate package name {}", p.name);
            }
        }
    }

    /// Every library is PURE as generated. The whole experiment is "an effect appears where none
    /// was before", so a library that already declared one would have nothing to detect.
    #[test]
    fn every_library_starts_pure() {
        for c in build() {
            for p in c.libs() {
                assert!(p.effects.is_empty(), "library `{}` is not pure to begin with", p.name);
                assert!(
                    !p.source.contains("console"),
                    "library `{}` performs an effect before injection",
                    p.name
                );
            }
        }
    }

    /// The generator is deterministic: same corpus, byte for byte, on every run and every machine.
    #[test]
    fn generation_is_deterministic() {
        let a = build();
        let b = build();
        for (x, y) in a.iter().zip(b.iter()) {
            for (px, py) in x.packages.iter().zip(y.packages.iter()) {
                assert_eq!(px.manifest, py.manifest);
                assert_eq!(px.source, py.source);
            }
        }
    }
}
