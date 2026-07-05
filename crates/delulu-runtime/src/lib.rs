//! DeluluLang runtime: values, the capability table, the Stage-1 grant broker, and the
//! tree-walking interpreter. The runtime enforces capability scopes host-side on every use
//! (invariant 7), independently of the compile-time authority proof.

pub mod broker;
pub mod interp;
pub mod prim;
pub mod trace;
pub mod value;

pub use broker::{parse_manifest, Grants, Manifest};
pub use interp::Interp;
pub use prim::{set_capture, set_fixed_clock_ms, set_rand_seed, take_capture};
pub use value::{CapScope, CapVal};
pub use trace::{assert_trace, TraceRecord, TraceSink};
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

    /// Like `run`, but attaches a fresh `TraceSink` and returns it alongside the result, so
    /// tests can inspect exactly which effects were traced (spec §6.1).
    fn run_traced(src: &str, grants: Grants) -> (Result<Value, Fault>, TraceSink) {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "check errors: {:?}", checked.diagnostics);
        let sink = TraceSink::new();
        let interp = Interp::new(&checked.module).with_trace(sink.clone());
        let out = interp.run_main(Value::Root(std::rc::Rc::new(grants.build_root())));
        (out, sink)
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

    // ----- effect tracing (spec §6.1) --------------------------------------

    #[test]
    fn trace_records_write_then_read_in_order() {
        let mut g = Grants::default();
        g.console = true;
        g.fs_read.push(".".into());
        let (out, sink) = run_traced(
            "module m\nfn main(root: Root) ! {Write, Read} { let out = root.console()\n out.println(\"hi\")\n let fs = root.fs_read(\".\")\n match fs.read_text(\"nonexistent_file_xyz.delulu.tmp\") { Ok(_) => {}, Err(_) => {} } }\n",
            g,
        );
        assert!(out.is_ok(), "{:?}", out.err());
        let records = sink.records();
        assert_eq!(records.len(), 2, "{records:?}");
        assert_eq!(records[0].effect, "Write");
        assert_eq!(records[0].op, "println");
        assert_eq!(records[1].effect, "Read");
        assert_eq!(records[1].op, "read_text");
        assert!(records[0].seq < records[1].seq, "seq must strictly increase: {records:?}");
    }

    #[test]
    fn trace_never_leaks_a_secret_and_redacts_expose() {
        let secret_value = "correct-horse-battery-staple-42";
        let mut g = Grants::default();
        g.declassify = true;
        g.secrets.insert("K".into(), secret_value.into());
        let (out, sink) = run_traced(
            "module m\nfn main(root: Root) ! {Declassify} { let s = root.secret(\"K\")\n let d = root.declassify()\n let _v = s.expose(d) }\n",
            g,
        );
        assert!(out.is_ok(), "{:?}", out.err());
        let records = sink.records();
        assert_eq!(records.len(), 1, "{records:?}");
        assert_eq!(records[0].effect, "Declassify");
        assert_eq!(records[0].op, "expose");
        assert_eq!(records[0].cap_kind, "Secret");
        assert_eq!(records[0].detail.as_deref(), Some(trace::OPAQUE));

        // The raw secret must never appear anywhere in the serialized trace.
        let json = sink.to_json_lines();
        assert!(!json.contains(secret_value), "secret leaked into trace JSON: {json}");
        for r in &records {
            if let Some(d) = &r.detail {
                assert!(!d.contains(secret_value), "secret leaked into trace detail: {d}");
            }
        }
    }

    // `set_rand_seed`/`set_fixed_clock_ms` determinism (spec §6.2) is unit-tested directly
    // against the primitive table in `prim.rs` (it owns the thread-local RNG/clock override, so
    // it can assert on raw sequences without fighting `main`'s `-> Unit` return-type rule). This
    // test instead confirms the two effects are wired into tracing end-to-end.
    #[test]
    fn trace_covers_rand_and_clock_effects() {
        let mut g = Grants::default();
        g.rand = true;
        g.clock = true;
        let (out, sink) = run_traced(
            "module m\nfn main(root: Root) ! {Rand, Clock} { let r = root.rand()\n let _v = r.int(0, 10)\n let c = root.clock()\n let _t = c.now_ms() }\n",
            g,
        );
        assert!(out.is_ok(), "{:?}", out.err());
        let records = sink.records();
        assert_eq!(records.len(), 2, "{records:?}");
        assert_eq!(records[0].effect, "Rand");
        assert_eq!(records[0].op, "int");
        assert_eq!(records[1].effect, "Clock");
        assert_eq!(records[1].op, "now_ms");
    }

    #[test]
    fn assert_trace_is_empty_for_row_conformant_run() {
        // The trace ⊆ row law (spec §6.2 / invariant 12), executable: the checker's own
        // `main_row` is a sound upper bound for every effect the traced run actually performs.
        let src = "module m\nfn main(root: Root) ! {Write} { let c = root.console()\n c.println(\"hi\") }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let allowed: std::collections::BTreeSet<String> = checked
            .result
            .main_row
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|e| e.name().to_string())
            .collect();

        let mut g = Grants::default();
        g.console = true;
        let (out, sink) = run_traced(src, g);
        assert!(out.is_ok(), "{:?}", out.err());
        let records = sink.records();
        assert!(!records.is_empty());

        let violations = assert_trace(&allowed, &records);
        assert!(violations.is_empty(), "unexpected trace-assert violations: {violations:?}");
    }
}
