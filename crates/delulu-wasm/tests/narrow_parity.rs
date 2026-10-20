//! RW 3.4 — `Cap[FsRead].narrow(sub)` on the WASM engine, held to the interpreter. The host mints the narrower handle by
//! the interpreter's own `resolve_in_scope`, so the two engines draw one boundary: a narrowed handle reads inside its
//! subtree, two narrowings of one handle are independent, and nothing — a `..`, a read past the subtree, a narrowing
//! back up — reaches past the scope a handle starts from (DL0904 on both, in the same words).

use std::path::PathBuf;
use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_main, HostConfig};

/// A tree `data/{public/notice.txt, archive/index.txt, secret.txt}` in a directory of its own.
fn tree(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_narrow_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("public")).unwrap();
    std::fs::create_dir_all(dir.join("archive")).unwrap();
    std::fs::write(dir.join("public/notice.txt"), "public notice").unwrap();
    std::fs::write(dir.join("archive/index.txt"), "archive index").unwrap();
    std::fs::write(dir.join("secret.txt"), "secret").unwrap();
    dir
}

/// `main` with `body`, `fs` a handle granted on `dir`, run on both engines: each output, or each refusal's text.
fn run_both(dir: &std::path::Path, body: &str) -> (Result<String, String>, Result<String, String>) {
    let at = dir.to_string_lossy().replace('\\', "/");
    let src = format!(
        "module m\n\nfn read(fs: Cap[FsRead], name: Str) -> Str ! {{Read}} {{\n  match fs.read_text(name) {{\n    Ok(t) => t,\n    \
         Err(_) => \"none\",\n  }}\n}}\n\nfn main(root: Root) ! {{Read, Write}} {{\n  let out = root.console()\n  let fs = \
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

/// The corpus's attenuation shape: two narrowings of one handle, each reading inside its own subtree, and a handle
/// narrowed twice — the second narrowing within the first.
#[test]
fn a_narrowed_handle_reads_inside_its_subtree_on_both_engines() {
    let dir = tree("reads");
    let (w, i) = run_both(
        &dir,
        "  let public = fs.narrow(\"./public\")\n  let archive = fs.narrow(\"archive\")\n  out.println(read(public, \
         \"notice.txt\") + \" / \" + read(archive, \"index.txt\") + \" / \" + read(public, \"index.txt\"))\n  let again = \
         fs.narrow(\".\").narrow(\"public\")\n  out.println(read(again, \"notice.txt\"))\n",
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(w, i, "two-engine parity");
    assert_eq!(w.as_deref(), Ok("public notice / archive index / none\npublic notice\n"));
}

/// Nothing reaches past the scope a handle starts from: a narrowing back up (`..`), a read through a narrowed handle
/// into its sibling or its parent, a narrowing to a sibling by way of `..`. Each is DL0904 on both engines, in the
/// interpreter's words, after what was printed before it.
#[test]
fn no_narrowing_or_read_reaches_past_the_scope_on_both_engines() {
    let dir = tree("refusals");
    for (body, words) in [
        (
            "  let sub = fs.narrow(\"public\")\n  out.println(read(sub, \"notice.txt\"))\n  let up = sub.narrow(\"..\")\n  \
          out.println(\"widened\")\n",
            "path `..` escapes the granted scope",
        ),
        (
            "  let sub = fs.narrow(\"public\")\n  out.println(read(sub, \"../secret.txt\"))\n",
            "path `../secret.txt` escapes the granted scope",
        ),
        (
            "  let sub = fs.narrow(\"public\")\n  out.println(read(sub, \"../archive/index.txt\"))\n",
            "path `../archive/index.txt` escapes the granted scope",
        ),
        (
            "  let sub = fs.narrow(\"public\").narrow(\"../archive\")\n  out.println(read(sub, \"index.txt\"))\n",
            "path `../archive` escapes the granted scope",
        ),
    ] {
        let (w, i) = run_both(&dir, body);
        let (w, i) = (w.expect_err("refused on WASM"), i.expect_err("refused on the interpreter"));
        assert!(w.contains("DL0904") && w.contains(words), "WASM, for:\n{body}\n{w}");
        assert!(i.contains("DL0904") && i.contains(words), "the interpreter, for:\n{body}\n{i}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A module whose only filesystem operation is `narrow` — a function handed a handle, never minting one — carries the
/// filesystem imports: the scan that decides them names `narrow`, so what the backend writes is a module the engine
/// accepts.
#[test]
fn a_module_that_only_narrows_carries_the_filesystem_imports() {
    let src = "module m\nfn sub(fs: Cap[FsRead]) -> Cap[FsRead] {\n  fs.narrow(\"a\")\n}\n";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let wasm = compile_checked(&checked.module, &checked.result).expect("compiles");
    wasmtime::Module::new(&wasmtime::Engine::default(), &wasm).expect("a module the engine accepts");
}
