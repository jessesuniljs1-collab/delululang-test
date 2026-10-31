//! A7 (the agent test campaign) — `Cap[FsRead].list_dir(path)` on the WASM engine, held to the interpreter. It was refused
//! DL1201 *"WASM codegen does not support the method `.list_dir`"*, so a program that walked a directory could not run on
//! the faster engine or be built as a `.dwx`. The host lists it as the interpreter does (`beneath::list_dir_at`, at the
//! pinned path, after the same custody and scope checks), so the two engines answer alike: the names, a missing
//! directory, a file named as one, an empty one, and a path past the scope (DL0904, in the same words).
//!
//! The entries come in the operating system's order on both engines; each program sorts them before it prints.

use std::path::PathBuf;
use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_main, HostConfig};

/// `data/{a.txt, b.txt, empty/}` in a directory of its own.
fn tree(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_list_dir_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("empty")).unwrap();
    std::fs::write(dir.join("a.txt"), "x\n").unwrap();
    std::fs::write(dir.join("b.txt"), "one\ntwo\n").unwrap();
    dir
}

/// `main` with `body`, `fs` a handle granted on `dir`, run on both engines: each output, or each refusal's text.
fn run_both(dir: &std::path::Path, body: &str) -> (Result<String, String>, Result<String, String>) {
    let at = dir.to_string_lossy().replace('\\', "/");
    let src = format!(
        "module m\n\nfn show(r: Result[List[Str], IoErr]) -> Str {{\n  match r {{\n    Ok(names) => \"ok \" + str(names.sort()),\n    \
         Err(e) => \"err \" + str(e),\n  }}\n}}\n\nfn main(root: Root) ! {{Read, Write}} {{\n  let out = root.console()\n  let fs = \
         root.fs_read(\"{at}\")\n{body}}}\n"
    );
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let cfg = HostConfig { console: true, fs_read_roots: vec![dir.to_path_buf()], ..HostConfig::default() };
    let w = run_main(&wasm, &cfg).map_err(|e| e.message());
    set_capture(true);
    let g = Grants { console: true, fs_read: vec![dir.to_string_lossy().to_string()], ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    (w, i.map(|_| out))
}

#[test]
fn a_directory_is_listed_alike_on_both_engines() {
    let dir = tree("listed");
    let (w, i) = run_both(
        &dir,
        "  out.println(show(fs.list_dir(\".\")))\n  out.println(show(fs.list_dir(\"empty\")))\n  \
         out.println(show(fs.list_dir(\"missing\")))\n  out.println(show(fs.list_dir(\"a.txt\")))\n  \
         match fs.list_dir(\".\") {\n    Ok(names) => {\n      for n in names.sort() {\n        match fs.read_text(n) {\n          \
         Ok(t) => out.println(n + \" \" + str(t.len())),\n          Err(e) => out.println(n + \" \" + str(e)),\n        }\n      \
         }\n    },\n    Err(e) => out.println(str(e)),\n  }\n",
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(w, i, "two-engine parity");
    let w = w.expect("ran");
    let lines: Vec<&str> = w.lines().collect();
    assert_eq!(&lines[..4], ["ok [a.txt, b.txt, empty]", "ok []", "err NotFound", "err Other(not a directory)"], "{w}");
    assert_eq!(&lines[4..6], ["a.txt 2", "b.txt 8"], "each listed file read back: {w}");
}

#[test]
fn a_listing_past_the_scope_is_refused_alike_on_both_engines() {
    let dir = tree("refused");
    let (w, i) = run_both(&dir, "  out.println(\"before\")\n  out.println(show(fs.list_dir(\"../..\")))\n");
    let _ = std::fs::remove_dir_all(&dir);
    let (w, i) = (w.expect_err("refused on WASM"), i.expect_err("refused on the interpreter"));
    assert!(i.contains("DL0904") && i.contains("path `../..` escapes the granted scope"), "the interpreter: {i}");
    assert!(w.ends_with("DL0904: path `../..` escapes the granted scope"), "the WASM engine, in the same words: {w}");
}

/// The listing is asked of the custody as every file effect is: an artifact whose manifest does not declare `Read`
/// (ARTIFACT-CLAIM-1) lists nothing, refused DL1202.
#[test]
fn a_listing_its_artifact_does_not_declare_is_refused() {
    let dir = tree("undeclared");
    let at = dir.to_string_lossy().replace('\\', "/");
    let src = format!(
        "module m\n\nfn main(root: Root) ! {{Read, Write}} {{\n  let fs = root.fs_read(\"{at}\")\n  match fs.list_dir(\".\") {{\n    \
         Ok(names) => root.console().println(str(names.len())),\n    Err(e) => root.console().println(str(e)),\n  }}\n}}\n"
    );
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let wasm = compile_checked(&checked.module, &checked.result).expect("compiles to WASM");
    let declared: std::collections::BTreeSet<String> = ["Write".to_string()].into_iter().collect();
    let cfg = HostConfig {
        console: true,
        fs_read_roots: vec![dir.to_path_buf()],
        declared_effects: Some(declared),
        ..HostConfig::default()
    };
    let w = run_main(&wasm, &cfg).expect_err("an undeclared `Read` is refused").message();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(w.contains("DL1202") && w.contains("`Read`"), "{w}");
}
