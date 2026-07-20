//! DeluluLang runtime: values, the capability table, the Stage-1 grant broker, and the
//! tree-walking interpreter. The runtime enforces capability scopes host-side on every use
//! (invariant 7), independently of the compile-time authority proof.

pub mod actors;
pub mod broker;
pub mod compute;
pub mod cycles;
pub mod custody;
pub mod device;
pub mod foreign;
pub mod interp;
pub mod plugin;
pub mod pqc;
pub mod prim;
pub mod python;
pub mod trace;
pub mod value;

pub use broker::{parse_manifest, Grants, Manifest};
pub use custody::{Custody, CustodyDecision, CustodyDenial, EmbeddedCustody, Liveness, Op};
pub use device::{
    Approval, AuthorityProbe, AuthorityState, DeviceBroker, DeviceEvent, FailState, Profile,
    RevokeCause,
};
pub use foreign::{BoundForeign, FKind, FVal, ForeignBinder, ForeignErr, ForeignSig, InProcBinder};
pub use interp::Interp;
pub use plugin::{
    cap_slice, kill_on_limit, load_prepare, load_verified, plugin_err_code, r_get_contained,
    public_key_hex, r_get_verified, sig_message, sign_detached, sign_plugin, step5_verified,
    step6_signature, unload, verify_detached,
    verify_plugin, verify_signature, Grant, HandleTable, ImportSig, ImportSlice, Limits, LoadRefusal,
    LoadedPlugin, PluginArtifact, PluginClass, PluginEngine, PluginErr, PluginRef, PluginRunError,
    PreparedLoad, SignatureStatus, VerifiedInterpInstance, VerifiedPlugin, VerifyReport, WasmVal,
    DL_SIGNATURE_INVALID, DL_SIGNATURE_REQUIRED, PLUGIN_API_SUPPORTED,
};
pub use prim::{set_capture, set_fixed_clock_ms, set_rand_seed, take_capture};
pub use value::{CapScope, CapVal};
pub use trace::{assert_trace, assert_trace_causal, Cause, TraceRecord, TraceSink};
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

    // ----- Stage 5 phase 5f/5g: the custody seam (daemon mode via a fake custody) ----------------

    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    /// A fake daemon custody for unit tests: optionally denies every `check`, serves `expose` from
    /// an in-memory map, and records every expose call (name + span) for assertions. No transport,
    /// no daemon — this exercises the SEAM, not the wire (the wire is tested in the CLI crate).
    struct FakeCustody {
        deny: Option<(&'static str, String)>,
        secrets: HashMap<String, String>,
        exposed: Rc<RefCell<Vec<(String, Option<String>)>>>,
    }

    impl Custody for FakeCustody {
        fn check(&mut self, _op: Op, _arg: Option<&str>) -> CustodyDecision {
            match &self.deny {
                Some((code, msg)) => CustodyDecision::Deny(CustodyDenial::new(code, msg.clone())),
                None => CustodyDecision::Allow,
            }
        }
        fn expose(&mut self, name: &str, span: Option<&str>) -> Result<String, CustodyDenial> {
            self.exposed.borrow_mut().push((name.to_string(), span.map(str::to_string)));
            self.secrets
                .get(name)
                .cloned()
                .ok_or_else(|| CustodyDenial::new("DL0904", format!("secret `{name}` not held")))
        }
        fn refresh_epoch(&mut self) {}
        fn mode(&self) -> &'static str {
            "daemon"
        }
    }

    /// Criterion 6 (strengthened): a daemon-mode secret HANDLE carries no byte content at all —
    /// the program-process cap cache holds only the name; `reveal` yields nothing; two handles
    /// never verify equal (bytes are not here to compare).
    #[test]
    fn daemon_secret_handle_holds_no_bytes() {
        let h = value::SecretVal::handle("API_KEY");
        assert_eq!(h.handle_name(), Some("API_KEY"));
        assert_eq!(h.reveal(), "", "a handle has NO in-process bytes pre-expose (invariant 23)");
        let h2 = value::SecretVal::handle("API_KEY");
        assert!(!h.verify(&h2), "handles carry no bytes, so verify is false (broker-side verify is post-chunk-3)");
    }

    /// Phase 5g end-to-end at the seam: `root.secret(name)` under `broker_secrets` yields a handle,
    /// and `expose` routes through `Custody::expose` — the bytes enter the program process ONLY
    /// there, and the call carries the source span (audited daemon-side).
    #[test]
    fn daemon_mode_expose_routes_through_custody_with_span() {
        let src = "module m\nfn main(root: Root) ! {Declassify, Write} { let s = root.secret(\"K\")\n let d = root.declassify()\n let v = s.expose(d)\n let c = root.console()\n c.println(v) }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);

        let exposed = Rc::new(RefCell::new(Vec::new()));
        let custody = FakeCustody {
            deny: None,
            secrets: [("K".to_string(), "swordfish".to_string())].into_iter().collect(),
            exposed: Rc::clone(&exposed),
        };

        let mut g = Grants::default();
        g.console = true;
        g.declassify = true;
        let mut root = g.build_root();
        root.broker_secrets = vec!["K".to_string()]; // daemon mode: handles, not bytes
        assert!(root.secrets.is_empty(), "no local secret bytes in daemon mode");

        set_capture(true);
        let interp = Interp::new(&checked.module).with_custody(Box::new(custody));
        let out = interp.run_main(Value::Root(Rc::new(root)));
        let printed = take_capture();
        assert!(out.is_ok(), "{:?}", out.err());
        assert_eq!(printed.as_deref(), Some("swordfish\n"), "the exposed bytes reached the program only via custody");

        let calls = exposed.borrow();
        assert_eq!(calls.len(), 1, "exactly one expose round-trip");
        assert_eq!(calls[0].0, "K");
        let span = calls[0].1.as_deref().expect("expose carries the calling span");
        assert!(span.contains(':'), "span is file:start:end shaped: {span}");
    }

    /// Phase 5f: a custody denial faults the effectful op with the BROKER'S code (here DL1403 with
    /// the revoking seq in the message) — the interpreter surfaces it as a clean Fault, and the
    /// denied effect is never performed.
    #[test]
    fn custody_denial_faults_the_op_with_the_brokers_code() {
        let src = "module m\nfn main(root: Root) ! {Write} { let c = root.console()\n c.println(\"never\") }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);

        let custody = FakeCustody {
            deny: Some(("DL1403", "lease `g_x` was revoked by audit seq 42".to_string())),
            secrets: HashMap::new(),
            exposed: Rc::new(RefCell::new(Vec::new())),
        };
        let mut g = Grants::default();
        g.console = true;

        set_capture(true);
        let interp = Interp::new(&checked.module).with_custody(Box::new(custody));
        let out = interp.run_main(Value::Root(Rc::new(g.build_root())));
        let printed = take_capture();
        let fault = out.err().expect("denied custody must fault");
        assert_eq!(fault.code, "DL1403");
        assert!(fault.message.contains("42"), "the broker's revoking seq travels to the program: {}", fault.message);
        assert_eq!(printed.unwrap_or_default(), "", "the denied effect was never performed");
    }

    // ===== Stage 8, phase 8a: `test` blocks are compiled out; asserts panic DL1707 =====

    /// Invariant 41's runtime half: a `test` block NEVER executes under `delulu run` — its
    /// body's side effects are absent from the program's output (tests are compiled out).
    #[test]
    fn test_blocks_are_compiled_out_of_a_run() {
        let src = "module m\nfn main(root: Root) ! {Write} { let c = root.console()\n c.println(\"main ran\") }\n\
                   test \"side effect\" ! {Write} { let c = test_root.console()\n c.println(\"test ran\") }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let g = Grants { console: true, ..Grants::default() };
        set_capture(true);
        let interp = Interp::new(&checked.module);
        let out = interp.run_main(Value::Root(Rc::new(g.build_root())));
        let printed = take_capture();
        assert!(out.is_ok(), "{:?}", out.err());
        assert_eq!(printed.as_deref(), Some("main ran\n"), "the test body must never run");
    }

    #[test]
    fn assert_failure_is_a_dl1707_panic() {
        let src = "module m\nfn f() { assert(1 == 2) }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let interp = Interp::new(&checked.module);
        let fault = interp.call_with("f", vec![]).expect_err("assert(false) must fault");
        assert_eq!(fault.code, "DL1707");
    }

    #[test]
    fn assert_eq_failure_reports_both_values() {
        let src = "module m\nfn f() { assert_eq(2 + 2, 5) }\nfn g() { assert_eq(21 * 2, 42) }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let interp = Interp::new(&checked.module);
        let fault = interp.call_with("f", vec![]).expect_err("4 != 5 must fault");
        assert_eq!(fault.code, "DL1707");
        assert!(
            fault.message.contains('4') && fault.message.contains('5'),
            "both compared values travel in the panic: {}",
            fault.message
        );
        assert!(interp.call_with("g", vec![]).is_ok(), "equal values pass");
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
