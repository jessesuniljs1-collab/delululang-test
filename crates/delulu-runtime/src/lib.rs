//! DeluluLang runtime: values, the capability table, the Stage-1 grant broker, and the
//! tree-walking interpreter. The runtime enforces capability scopes host-side on every use
//! (invariant 7), independently of the compile-time authority proof.

pub mod broker;
pub mod interp;
pub mod prim;
pub mod value;

pub use broker::{parse_manifest, Grants, Manifest};
pub use interp::Interp;
pub use value::{Fault, RootVal, Value};

#[cfg(test)]
mod tests {
    use super::*;
    use delulu_check::check_source;

    /// Run a program with a fully-granted root and capture its stdout is out of scope for a unit
    /// test, so we assert on the returned value / absence of faults instead.
    fn run(src: &str, grants: Grants) -> Result<Value, Fault> {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
        let interp = Interp::new(&checked.module);
        interp.run_main(Value::Root(std::rc::Rc::new(grants.build_root())))
    }

    #[test]
    fn runs_pure_arithmetic_and_returns_unit() {
        let mut g = Grants::default();
        g.console = true;
        let out = run(
            "module m\nfn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }\nfn main(root: Root) ! {Write} { let c = root.console()\n c.println(str(fib(10))) }\n",
            g,
        );
        assert!(out.is_ok(), "{:?}", out.err());
    }

    #[test]
    fn ungranted_capability_is_dl0703_at_runtime() {
        // main is well-typed (declares Write) but we grant no console.
        let g = Grants::default();
        let out = run(
            "module m\nfn main(root: Root) ! {Write} { let c = root.console()\n c.println(\"hi\") }\n",
            g,
        );
        assert_eq!(out.err().map(|f| f.code), Some("DL0703"));
    }

    #[test]
    fn division_by_zero_is_dl0902() {
        let g = Grants::default();
        let out = run("module m\nfn main(root: Root) { let x = 1 / 0 }\n", g);
        assert_eq!(out.err().map(|f| f.code), Some("DL0902"));
    }

    #[test]
    fn fs_scope_escape_is_dl0904() {
        let mut g = Grants::default();
        g.fs_read.push("./config".into());
        let out = run(
            "module m\nfn read(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} { let t = fs.read_text(\"../secret.txt\")?\n Ok(t) }\nfn main(root: Root) ! {Read} { let fs = root.fs_read(\"./config\")\n match read(fs) { Ok(_) => {}, Err(_) => {} } }\n",
            g,
        );
        assert_eq!(out.err().map(|f| f.code), Some("DL0904"));
    }

    #[test]
    fn secret_never_reaches_stdout_but_expose_returns_it() {
        // A grant of the secret + declassify; expose returns the value (that is declassification
        // working), while the program without expose can never stringify it (checker DL0604).
        let mut g = Grants::default();
        g.declassify = true;
        g.secrets.insert("K".into(), "swordfish".into());
        let out = run(
            "module m\nfn main(root: Root) ! {Declassify} { let s = root.secret(\"K\")\n let d = root.declassify()\n let _v = s.expose(d) }\n",
            g,
        );
        assert!(out.is_ok(), "{:?}", out.err());
    }
}
