//! Stage 10 phase 10h — heterogeneous compute (Track F, spec §7, invariants 49 and 50).
//!
//! Two laws are tested here, and the second is the one worth the file:
//!
//! 1. **A compute grant that forgot to bound something is refused, by name.** Memory, kernel time,
//!    queue depth and power are all mandatory, because a resource the grant did not bound is a
//!    resource nothing bounds. This is 10f's dead-man rule applied to silicon.
//!
//! 2. **The double-enforcement claim is never silently single.** Spec §7.1 says a device envelope
//!    is checked host-side *and* adapter-side. The only adapter that ships in-tree cannot honestly
//!    make the second half of that claim — it runs in this process, so there is no layer below it
//!    — and DL1911 makes that fact arrive at the operator instead of quietly halving the
//!    guarantee. A human may accept single enforcement; nobody may accept it by accident.
//!
//! Note what no test here can do: assert an attestation into existence. Attestation is read from
//! the adapter's own declaration, never from the grant string, so a grant cannot claim one.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(repo_root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-compute-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A program that mints a compute device and dispatches one kernel, reporting whatever it is told.
const PROG: &str = "\
module cg

fn main(root: Root) ! {Write, ForeignCall} {
  let c = root.console()
  let g = root.compute(\"gpu0\")
  match g.dispatch(\"reduce_sum\", [1.0, 2.0, 3.0]) {
    Ok(v) => c.println(\"OK \" + str(v)),
    Err(e) => match e {
      KernelEnvelope(r) => c.println(\"ENV: \" + r),
      UnknownKernel(n) => c.println(\"UNKNOWN: \" + n),
      NoAdapter => c.println(\"NOADAPTER\")
    }
  }
}
";

fn program(tag: &str) -> String {
    let p = scratch(tag).join("cg.delulu");
    std::fs::write(&p, PROG).unwrap();
    p.to_string_lossy().to_string()
}

/// Every bounding term present, adapter attestation waived.
///
/// The kernel path deliberately points at nothing: every test using `FULL` asserts on a refusal
/// raised at grant-parse or DL1911 time, both of which run BEFORE any artifact is read. The
/// artifact path is exercised by the laundering tests below, which build real signed files.
const FULL: &str = "compute=gpu0:memory_bytes=1048576,kernel_ms=0..50,queue_depth=4,\
power_w=0..120,adapter=cpu-reference,format=refkernel-1,kernel=reduce_sum:/nonexistent/k.refkernel,\
waive-attestation";

fn run_with(tag: &str, grant: &str) -> Output {
    delulu(&["run", &program(tag), "--grant", "console", "--grant", grant, "--no-prompt"])
}

/// Each mandatory term, dropped one at a time, refused BY NAME. An operator who forgot one should
/// be told which — not handed a syntax summary to diff by eye (build-order D11a's rule).
#[test]
fn every_bounding_term_is_mandatory_and_its_absence_is_named() {
    // (term to drop, a distinctive phrase the refusal must contain)
    let cases: &[(&str, &str)] = &[
        ("memory_bytes=1048576,", "`memory_bytes`"),
        ("kernel_ms=0..50,", "`kernel_ms"),
        ("queue_depth=4,", "`queue_depth`"),
        ("power_w=0..120,", "`power_w"),
        ("adapter=cpu-reference,", "`adapter="),
        ("format=refkernel-1,", "`format="),
        ("kernel=reduce_sum:/nonexistent/k.refkernel,", "`kernel=NAME:PATH`"),
    ];
    for (drop, needle) in cases {
        let grant = FULL.replace(drop, "");
        assert_ne!(grant, FULL, "the test's own edit did nothing for `{drop}`");
        let o = run_with("mandatory", &grant);
        let err = stderr(&o);
        assert_eq!(o.status.code(), Some(2), "dropping `{drop}` must be refused:\n{err}");
        assert!(
            err.contains(needle),
            "the refusal must name the missing term ({needle}) rather than describing the syntax:\n{err}"
        );
    }
}

/// The grant form's own fail-closed edges: a range that is not a range, an inverted range, a zero
/// ceiling that would refuse everything, a kernel with no hash, and a term nobody recognises.
#[test]
fn a_malformed_compute_grant_is_refused_rather_than_partially_understood() {
    let bad: &[(&str, &str)] = &[
        ("kernel_ms=0..50", "kernel_ms=fast"),
        ("kernel_ms=0..50", "kernel_ms=50..0"),
        ("memory_bytes=1048576", "memory_bytes=0"),
        ("queue_depth=4", "queue_depth=0"),
        ("kernel=reduce_sum:/nonexistent/k.refkernel", "kernel=reduce_sum"),
        ("power_w=0..120", "power_w=0..120,turbo=yes"),
    ];
    for (from, to) in bad {
        let grant = FULL.replace(from, to);
        let o = run_with("malformed", &grant);
        assert_eq!(
            o.status.code(),
            Some(2),
            "`{to}` must be refused, never silently accepted:\n{}",
            stderr(&o)
        );
    }
}

/// DL1911, the default path: the only in-tree adapter cannot attest a layer below itself, so a
/// grant naming it without an explicit waiver is refused.
///
/// This is the *normal* outcome for the reference adapter, not an edge case — which is exactly why
/// it is honest. A build whose only adapter silently claimed double enforcement would teach the
/// wrong lesson to every adapter written afterwards.
#[test]
fn an_adapter_that_cannot_attest_below_itself_is_refused_the_grant() {
    let grant = FULL.replace(",waive-attestation", "");
    let o = run_with("attest", &grant);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1911"), "{err}");
    assert!(
        err.contains("below-adapter") && err.contains("silently single"),
        "the refusal must say what claim it is protecting:\n{err}"
    );
    assert!(
        err.contains("waive-attestation"),
        "and how a human may accept single enforcement deliberately:\n{err}"
    );
}

/// THE SKIP BRANCH. An adapter this build has never heard of is refused — not trusted, not
/// assumed-attesting, not treated as "probably a real GPU runtime".
///
/// This is the branch that decides whether DL1911 means anything. A gate that only refuses
/// adapters it knows to be non-attesting, and waves through the ones it cannot identify, refuses
/// exactly the honest adapters and admits every unknown one.
#[test]
fn an_unknown_adapter_is_refused_because_it_cannot_attest_what_it_is_not() {
    let grant = FULL.replace("adapter=cpu-reference", "adapter=vendor-x-cuda");
    let o = run_with("unknown", &grant);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1911"), "{err}");
    assert!(
        err.contains("not one this build knows") && err.contains("taken on trust"),
        "the refusal must name unfamiliarity as the reason:\n{err}"
    );
    // And the waiver must NOT rescue an unknown adapter: waiving the attestation requirement is a
    // decision about enforcement depth, not permission to load an adapter that does not exist.
    let waived = format!("{grant},waive-attestation");
    let o = run_with("unknown-waived", &waived);
    assert_eq!(
        o.status.code(),
        Some(1),
        "a waiver must not conjure an adapter this build does not have:\n{}",
        stderr(&o)
    );
    assert!(stderr(&o).contains("DL1911"), "{}", stderr(&o));
}

/// A kernel format the adapter cannot accept is refused, waiver or not. Vendor neutrality is not
/// permissiveness: the interface expresses formats, and an adapter that does not speak one says so.
#[test]
fn a_kernel_format_the_adapter_does_not_accept_is_refused() {
    let grant = FULL.replace("format=refkernel-1", "format=ptx-8");
    let o = run_with("format", &grant);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1911") && err.contains("ptx-8"), "{err}");
}

/// The waiver works, and it is the ONLY way past the gate: with it the grant is issued and the
/// program reaches its dispatch. Without a passing case the refusals above would prove nothing —
/// a gate that refuses everything is not a gate either.
#[test]
fn an_explicit_human_waiver_lets_the_grant_through() {
    let o = run_with("waived", FULL);
    let err = stderr(&o);
    assert!(
        !err.contains("DL1911"),
        "an explicitly waived grant must be issued, not refused again:\n{err}"
    );
}

/// `class` is the one optional term, and deliberately so: it is display-only taxonomy that
/// invariant 49 forbids branching on. Defaulting a *descriptive* field is fine; defaulting a
/// *bounding* field is the thing every other test in this file exists to prevent.
///
/// The two runs must behave identically — if `class` ever changed an outcome, it would have become
/// a decision input, and the vendor-neutrality invariant would be gone.
#[test]
fn class_is_optional_because_nothing_may_branch_on_it() {
    let with = format!("{FULL},class=gpu");
    let a = run_with("class-with", &with);
    let b = run_with("class-without", FULL);
    assert_eq!(
        a.status.code(),
        b.status.code(),
        "naming the device class must not change what happens:\nwith:\n{}\nwithout:\n{}",
        stderr(&a),
        stderr(&b)
    );
}

// ----- the dispatch path (T3) -------------------------------------------------------------------

/// A program that dispatches four kernels and reports each outcome.
const DISPATCH_PROG: &str = "\
module cd

fn say(r: Result[Float, ComputeErr]) -> Str {
  match r {
    Ok(v) => \"OK \" + str(v),
    Err(e) => match e {
      KernelEnvelope(x) => \"ENV: \" + x,
      UnknownKernel(n) => \"UNKNOWN: \" + n,
      NoAdapter => \"NOADAPTER\"
    }
  }
}

fn main(root: Root) ! {Write, ForeignCall} {
  let c = root.console()
  let g = root.compute(\"gpu0\")
  c.println(say(g.dispatch(\"reduce_sum\", [1.0, 2.0, 3.0, 4.0])))
  c.println(say(g.dispatch(\"reduce_max\", [1.0, 9.0, 3.0])))
  c.println(say(g.dispatch(\"not_granted\", [1.0])))
  c.println(say(g.dispatch(\"reduce_sum\", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])))
}
";


/// `memory_bytes=64` holds eight `f64`s — so the four- and three-element dispatches land and the
/// nine-element one does not. One run exercises all four outcomes the error sum can produce.
#[test]
fn dispatch_lands_inside_the_envelope_and_is_refused_outside_it() {
    let dir = scratch("dispatch");
    let sum = kernel_artifact(&dir, "reduce_sum", "refkernel-1 reduce_sum\n", true);
    let max = kernel_artifact(&dir, "reduce_max", "refkernel-1 reduce_max\n", true);
    let grant = format!(
        "compute=gpu0:memory_bytes=64,kernel_ms=0..60000,queue_depth=4,power_w=0..120,\
         adapter=cpu-reference,format=refkernel-1,kernel=reduce_sum:{sum},kernel=reduce_max:{max},\
         waive-attestation"
    );
    let prog = dir.join("cd.delulu");
    std::fs::write(&prog, DISPATCH_PROG).unwrap();
    let o = delulu(&[
        "run", &prog.to_string_lossy(), "--grant", "console", "--grant", &grant, "--no-prompt",
    ]);
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    let err = stderr(&o);
    assert!(o.status.success(), "a refused dispatch is a value, not a fault:\n{out}\n{err}");

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "OK 10.0", "reduce_sum over [1,2,3,4]:\n{out}");
    assert_eq!(lines[1], "OK 9.0", "reduce_max over [1,9,3]:\n{out}");
    assert!(lines[2].starts_with("UNKNOWN:"), "a kernel not in the grant:\n{out}");
    assert!(
        lines[3].starts_with("ENV:") && lines[3].contains("memory_bytes=64"),
        "nine f64s exceed a 64-byte ceiling, and the refusal names the term:\n{out}"
    );
    // The refusal telemetry distinguishes the two: an auditor counting DL1907s is counting
    // devices driven past their envelope, NOT programs that named a kernel they never held.
    assert!(err.contains("DL1907 gpu0/reduce_sum"), "the envelope refusal is DL1907:\n{err}");
    assert!(
        err.contains("kernel-not-granted gpu0/not_granted"),
        "an unknown kernel must NOT be stamped DL1907:\n{err}"
    );
}

/// The `kernel_ms` budget, and the honest thing about it: it is enforced *after* the fact, on the
/// measurement, because you learn how long a kernel took by running it. The result is discarded.
#[test]
fn a_kernel_that_overruns_its_time_budget_is_refused_and_its_result_discarded() {
    // One scratch dir, created once: `scratch` REMOVES the directory it returns, so calling it
    // twice with the same tag deletes the file the first call just wrote.
    let dir = scratch("burn");
    let prog = dir.join("cb.delulu");
    std::fs::write(
        &prog,
        "module cb\nfn main(root: Root) ! {Write, ForeignCall} {\n  let c = root.console()\n  \
         let g = root.compute(\"gpu0\")\n  \
         match g.dispatch(\"burn\", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]) {\n    \
         Ok(v) => c.println(\"OK\"),\n    Err(e) => match e {\n      \
         KernelEnvelope(x) => c.println(\"ENV: \" + x),\n      UnknownKernel(n) => c.println(\"UNK\"),\n      \
         NoAdapter => c.println(\"NOAD\")\n    }\n  }\n}\n",
    )
    .unwrap();
    let burn = kernel_artifact(&dir, "burn", "refkernel-1 burn\n", true);
    let base = format!(
        "memory_bytes=4096,queue_depth=4,power_w=0..120,adapter=cpu-reference,\
         format=refkernel-1,kernel=burn:{burn},waive-attestation"
    );
    let tight = format!("compute=gpu0:{base},kernel_ms=0..1");
    let o = delulu(&[
        "run", &prog.to_string_lossy(), "--grant", "console", "--grant", &tight, "--no-prompt",
    ]);
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(
        out.contains("ENV:") && out.contains("kernel_ms=0..1"),
        "a kernel past its budget is refused, naming the budget:\n{out}"
    );
    assert!(out.contains("discarded"), "and its result is discarded, not returned:\n{out}");

    // The control: the SAME kernel under a budget that accommodates it. Without this, a broker
    // that refused every `burn` would pass the assertion above.
    let generous = format!("compute=gpu0:{base},kernel_ms=0..60000");
    let o = delulu(&[
        "run", &prog.to_string_lossy(), "--grant", "console", "--grant", &generous, "--no-prompt",
    ]);
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(out.contains("OK"), "the same kernel must land under a budget that fits it:\n{out}");
}

// ----- kernels are data: the laundering tests (T4, criterion 7) ---------------------------------

/// A fixed signing seed, so these tests are hermetic: they never read or write the user's keyring.
const SEED: [u8; 32] = [7u8; 32];

/// Write a kernel artifact, optionally signed. `sign: false` produces the unsigned case.
fn kernel_artifact(dir: &Path, name: &str, body: &str, sign: bool) -> String {
    let path = dir.join(format!("{name}.refkernel"));
    std::fs::write(&path, body).unwrap();
    if sign {
        let sig = delulu_runtime::sign_detached(&SEED, body.as_bytes());
        std::fs::write(dir.join(format!("{name}.refkernel.sig")), sig).unwrap();
    }
    path.to_string_lossy().replace('\\', "/")
}

fn grant_for(kernel_path: &str) -> String {
    format!(
        "compute=gpu0:memory_bytes=4096,kernel_ms=0..60000,queue_depth=4,power_w=0..120,\
         adapter=cpu-reference,format=refkernel-1,kernel=reduce_sum:{kernel_path},waive-attestation"
    )
}

fn run_dispatch(dir: &Path, grant: &str) -> Output {
    let prog = dir.join("k.delulu");
    std::fs::write(&prog, DISPATCH_ONE).unwrap();
    delulu(&[
        "run", &prog.to_string_lossy(), "--grant", "console", "--grant", grant, "--no-prompt",
    ])
}

const DISPATCH_ONE: &str = "\
module k
fn main(root: Root) ! {Write, ForeignCall} {
  let c = root.console()
  let g = root.compute(\"gpu0\")
  match g.dispatch(\"reduce_sum\", [1.0, 2.0, 3.0, 4.0]) {
    Ok(v) => c.println(\"OK \" + str(v)),
    Err(e) => match e {
      KernelEnvelope(x) => c.println(\"ENV\"),
      UnknownKernel(n) => c.println(\"UNK\"),
      NoAdapter => c.println(\"NOAD\")
    }
  }
}
";

/// Criterion 7, first half: **an unsigned kernel artifact is refused.**
///
/// There is deliberately no policy switch to accept one. A kernel is foreign code running on
/// hardware the program cannot otherwise reach; DeluluLang bounds its reachability, resources and
/// provenance, and provenance is the only one of the three that says anything about behaviour.
#[test]
fn an_unsigned_kernel_artifact_is_refused() {
    let dir = scratch("unsigned");
    let path = kernel_artifact(&dir, "reduce_sum", "refkernel-1 reduce_sum\n", false);
    let o = run_dispatch(&dir, &grant_for(&path));
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1913"), "unsigned is its own code, not DL1912:\n{err}");
    assert!(err.contains("provenance"), "the refusal says what is missing and why:\n{err}");
}

/// The control: the identical artifact, signed, runs. Without this the test above would pass for a
/// build that refused every kernel.
#[test]
fn the_same_artifact_signed_dispatches_normally() {
    let dir = scratch("signed");
    let path = kernel_artifact(&dir, "reduce_sum", "refkernel-1 reduce_sum\n", true);
    let o = run_dispatch(&dir, &grant_for(&path));
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(o.status.success(), "{out}\n{}", stderr(&o));
    assert_eq!(out.trim(), "OK 10.0", "the signed kernel runs and reduces [1,2,3,4]:\n{out}");
}

/// The substitution attack, and the reason a hash is not enough on its own: an artifact edited
/// AFTER signing — here, quietly changed from a sum to a max under the same name — no longer
/// verifies, and is refused with DL1912 rather than DL1913.
///
/// This is the case the whole mechanism exists for. The program is unchanged, the grant is
/// unchanged, the filename is unchanged, and the kernel now computes something else.
#[test]
fn an_artifact_edited_after_signing_no_longer_verifies() {
    let dir = scratch("tampered");
    let path = kernel_artifact(&dir, "reduce_sum", "refkernel-1 reduce_sum\n", true);
    // Same name, same path, same signature file — different bytes.
    std::fs::write(dir.join("reduce_sum.refkernel"), "refkernel-1 reduce_max\n").unwrap();
    let o = run_dispatch(&dir, &grant_for(&path));
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1912"), "a bad signature is DL1912, not DL1913:\n{err}");
    assert!(
        err.contains("not what it claims"),
        "the refusal names substitution rather than blaming the format:\n{err}"
    );
}

/// THE LAUNDERING TEST. A DeluluLang source file, signed correctly, is still not a kernel.
///
/// The rule is "kernels are data, never code from the row system" (spec §7.1, the spirit of R-6a).
/// A valid signature proves *provenance*, not *eligibility* — so an artifact whose bytes are a
/// DeluluLang module is refused for what it is, not merely for who signed it. There is no
/// interpreter on this path for it to be smuggled into.
#[test]
fn delulu_source_signed_and_named_as_a_kernel_is_still_not_a_kernel() {
    let dir = scratch("launder");
    let source = "module evil\nfn main(root: Root) ! {Write} {\n  let c = root.console()\n  \
                  c.println(\"I am a kernel now\")\n}\n";
    let path = kernel_artifact(&dir, "reduce_sum", source, true);
    let o = run_dispatch(&dir, &grant_for(&path));
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "signed DeluluLang code must not load as a kernel:\n{err}");
    assert!(err.contains("DL1912"), "{err}");
    assert!(
        err.contains("format `module`"),
        "it is refused for not being a kernel artifact — the signature was perfectly valid:\n{err}"
    );
}

/// An artifact that IS well-formed but declares an operation the adapter does not implement is
/// refused too. The adapter's op list is closed: there is no "unknown op, pass it through" branch,
/// which is where a real adapter would otherwise hand unvetted bytes to a driver.
#[test]
fn an_artifact_naming_an_operation_the_adapter_lacks_is_refused() {
    let dir = scratch("badop");
    let path = kernel_artifact(&dir, "reduce_sum", "refkernel-1 mine_bitcoin\n", true);
    let o = run_dispatch(&dir, &grant_for(&path));
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("mine_bitcoin") && err.contains("DL1912"), "{err}");
}

/// The kernel NAME is an alias the human chose; the ARTIFACT decides what runs — exactly like
/// `foreign.c=LIB:PATH` binds a lib name to a binary. Here the grant binds the name `reduce_sum`
/// to an artifact that declares `reduce_max`, and `dispatch("reduce_sum", …)` computes the max.
///
/// This is documented by a test rather than left to be discovered, because reading it the other
/// way — assuming the name guarantees the behaviour — is the mistake worth preventing.
#[test]
fn the_artifact_decides_what_runs_not_the_name_the_program_typed() {
    let dir = scratch("alias");
    let path = kernel_artifact(&dir, "reduce_sum", "refkernel-1 reduce_max\n", true);
    let o = run_dispatch(&dir, &grant_for(&path));
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(o.status.success(), "{out}\n{}", stderr(&o));
    assert_eq!(
        out.trim(),
        "OK 4.0",
        "the artifact declared reduce_max, so [1,2,3,4] reduces to 4 — the name is an alias:\n{out}"
    );
}

/// The other half of the laundering law, at the type level: a DeluluLang function cannot even be
/// *spelled* as a kernel argument. `dispatch` takes a `Str`, so this is DL0401 at compile time —
/// no closure ever reaches the artifact path to be checked.
#[test]
fn a_closure_cannot_be_passed_as_a_kernel() {
    let dir = scratch("closure");
    let prog = dir.join("c.delulu");
    std::fs::write(
        &prog,
        "module c\nfn double(x: Float) -> Float { x * 2.0 }\n\
         fn run(g: Cap[Compute], b: List[Float]) -> Result[Float, ComputeErr] ! {ForeignCall} {\n  \
         g.dispatch(double, b)\n}\n",
    )
    .unwrap();
    let o = delulu(&["check", &prog.to_string_lossy()]);
    let err = format!("{}{}", String::from_utf8_lossy(&o.stdout), stderr(&o));
    assert!(!o.status.success(), "a closure as a kernel must not typecheck:\n{err}");
    assert!(
        err.contains("DL0401") && err.contains("expected `Str`"),
        "the refusal is a TYPE error at the call site, not a runtime check:\n{err}"
    );
}

/// A program reaching for silicon it was never granted is refused at the pre-flight, before a line
/// runs — the same deny-by-default rule every other capability gets (10e's DL0702 path).
#[test]
fn compute_is_deny_by_default_like_every_other_capability() {
    let o = delulu(&["run", &program("ungranted"), "--grant", "console", "--no-prompt"]);
    assert_ne!(o.status.code(), Some(0), "an ungranted device must not run:\n{}", stderr(&o));
}
