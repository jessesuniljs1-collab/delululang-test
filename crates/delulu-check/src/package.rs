//! Package loading and module-graph discovery (Stage 2, §3). A package is a directory with a
//! `delulu.toml` manifest and a `src/` tree; each `.delulu` file is a module named by its
//! `module` header. This pass discovers and parses every module in a package, indexes them by
//! declared name, and validates the intra-package import graph — unknown-module imports (DL0303)
//! and cycles (DL0304). Cross-module name resolution and authority verification build on this.
//!
//! This foundation deliberately does NOT resolve names across modules yet; it produces the parsed
//! module graph the next increment checks. It is pure over the filesystem + parser.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use delulu_diag::{Diagnostic, FileId, SourceMap, Span};
use delulu_syntax::ast::Module;

/// One parsed module within a package.
pub struct ModuleUnit {
    /// Dotted module name from the `module` header (e.g. `demo.util`).
    pub name: String,
    pub file: FileId,
    pub path: PathBuf,
    pub module: Module,
}

/// A loaded package: its parsed modules (indexed by declared name), the source map that owns
/// their text/spans, and any loading diagnostics (parse errors, DL0303/DL0304).
pub struct Package {
    pub root_dir: PathBuf,
    pub modules: Vec<ModuleUnit>,
    pub index: HashMap<String, usize>,
    pub source_map: SourceMap,
    pub diagnostics: Vec<Diagnostic>,
}

impl Package {
    pub fn get(&self, name: &str) -> Option<&ModuleUnit> {
        self.index.get(name).map(|&i| &self.modules[i])
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// The package's entry module, if any: the one declaring `fn main`.
    pub fn entry_module(&self) -> Option<&ModuleUnit> {
        self.modules.iter().find(|m| {
            m.module.items.iter().any(|it| matches!(it, delulu_syntax::ast::Item::Fn(f) if f.name.name == "main"))
        })
    }
}

/// Load and parse a package rooted at `root_dir` (which contains `src/`). Never fails outright:
/// returns a `Package` whose `diagnostics` carry any problems.
pub fn load_package(root_dir: impl AsRef<Path>) -> Package {
    let root_dir = root_dir.as_ref().to_path_buf();
    let src_dir = root_dir.join("src");
    let mut source_map = SourceMap::new();
    let mut modules = Vec::new();
    let mut index = HashMap::new();
    let mut diagnostics = Vec::new();

    let mut files = Vec::new();
    collect_delulu_files(&src_dir, &mut files);
    files.sort();

    for path in files {
        let src = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                diagnostics.push(Diagnostic::error("DL0303", format!("cannot read `{}`: {e}", path.display())));
                continue;
            }
        };
        let display = path.strip_prefix(&root_dir).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let file: FileId = source_map.add_file(display, src.clone());
        let (module, mut mdiags) = delulu_syntax::parse_file(file, &src);
        diagnostics.append(&mut mdiags);
        let name = module.name.dotted();
        if let Some(&prev) = index.get(&name) {
            let _ = prev;
            diagnostics.push(
                Diagnostic::error("DL0302", format!("duplicate module `{name}`"))
                    .with_bare_span(Span::new(file, 0, 0)),
            );
            continue;
        }
        index.insert(name.clone(), modules.len());
        modules.push(ModuleUnit { name, file, path, module });
    }

    let pkg = Package { root_dir, modules, index, source_map, diagnostics };
    let mut extra = Vec::new();
    validate_imports(&pkg, &mut extra);
    let mut pkg = pkg;
    pkg.diagnostics.extend(extra);
    pkg
}

fn collect_delulu_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_delulu_files(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                out.push(p);
            }
        }
    }
}

/// Validate intra-package imports: each import must name a module in this package (DL0303), and
/// the import graph must be acyclic (DL0304). Imports of external packages (dependencies) are a
/// later increment and are skipped here when unresolved — flagged distinctly so they are not
/// mistaken for typos once dependency resolution lands.
fn validate_imports(pkg: &Package, diags: &mut Vec<Diagnostic>) {
    for unit in &pkg.modules {
        for imp in &unit.module.imports {
            let target = imp.path.dotted();
            if !pkg.index.contains_key(&target) {
                diags.push(
                    Diagnostic::error("DL0303", format!("unknown module `{target}` imported by `{}`", unit.name))
                        .with_span(imp.span, "no such module in this package"),
                );
            }
        }
    }
    detect_cycles(pkg, diags);
}

fn detect_cycles(pkg: &Package, diags: &mut Vec<Diagnostic>) {
    // DFS with a recursion stack over the intra-package import graph.
    let mut color: HashMap<usize, u8> = HashMap::new(); // 0=white,1=gray,2=black
    let mut reported: HashSet<usize> = HashSet::new();

    fn visit(
        pkg: &Package,
        i: usize,
        color: &mut HashMap<usize, u8>,
        reported: &mut HashSet<usize>,
        diags: &mut Vec<Diagnostic>,
    ) {
        color.insert(i, 1);
        for imp in &pkg.modules[i].module.imports {
            if let Some(&j) = pkg.index.get(&imp.path.dotted()) {
                match color.get(&j).copied().unwrap_or(0) {
                    1 => {
                        if reported.insert(j) {
                            diags.push(
                                Diagnostic::error(
                                    "DL0304",
                                    format!("import cycle involving `{}` and `{}`", pkg.modules[i].name, pkg.modules[j].name),
                                )
                                .with_span(imp.span, "this import closes a cycle"),
                            );
                        }
                    }
                    0 => visit(pkg, j, color, reported, diags),
                    _ => {}
                }
            }
        }
        color.insert(i, 2);
    }

    for i in 0..pkg.modules.len() {
        if color.get(&i).copied().unwrap_or(0) == 0 {
            visit(pkg, i, &mut color, &mut reported, diags);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("delulu_pkg_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        dir
    }

    fn write(dir: &Path, rel: &str, contents: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, contents).unwrap();
    }

    #[test]
    fn loads_a_multi_module_package() {
        let dir = scratch("basic");
        write(&dir, "src/main.delulu", "module app\nimport app.util\nfn main(root: Root) { }\n");
        write(&dir, "src/util.delulu", "module app.util\nfn helper(n: Int) -> Int { n + 1 }\n");
        let pkg = load_package(&dir);
        assert!(!pkg.has_errors(), "{:?}", pkg.diagnostics);
        assert_eq!(pkg.modules.len(), 2);
        assert!(pkg.get("app").is_some() && pkg.get("app.util").is_some());
        assert_eq!(pkg.entry_module().unwrap().name, "app");
    }

    #[test]
    fn unknown_module_import_is_dl0303() {
        let dir = scratch("unknown");
        write(&dir, "src/main.delulu", "module app\nimport app.missing\nfn main(root: Root) { }\n");
        let pkg = load_package(&dir);
        assert!(pkg.diagnostics.iter().any(|d| d.code == "DL0303"), "{:?}", pkg.diagnostics);
    }

    #[test]
    fn import_cycle_is_dl0304() {
        let dir = scratch("cycle");
        write(&dir, "src/a.delulu", "module a\nimport b\nfn fa() -> Int { 1 }\n");
        write(&dir, "src/b.delulu", "module b\nimport a\nfn fb() -> Int { 2 }\n");
        let pkg = load_package(&dir);
        assert!(pkg.diagnostics.iter().any(|d| d.code == "DL0304"), "{:?}", pkg.diagnostics);
    }
}
