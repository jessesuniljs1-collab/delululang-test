//! Stage 10 phase 10h — heterogeneous compute (Track F, spec §7, invariants 49 and 50).
//!
//! **A kernel is a foreign call with an envelope.** Dispatching one carries the existing core
//! `ForeignCall` effect and no new one, because DeluluLang says nothing about what a kernel
//! computes. What it does say — and enforce — is three narrower things: the kernel is *reachable*
//! only through a granted capability, its *resources* are bounded by the device envelope, and its
//! *provenance* is a named, hashed artifact. `delulu authority` prints compute under the
//! outside-the-proof separator so nobody has to infer that from documentation.
//!
//! **Vendor neutrality is structural, not aspirational** (invariant 49). `class` and `adapter` are
//! recorded and displayed; nothing in the dispatch path branches on either. The day one vendor's
//! adapter needs a privileged hook is the day the authority model has a second class of citizen,
//! so the interface here is deliberately small enough that no adapter can need one: enumerate,
//! bound, submit, report.
//!
//! **Attestation is a property of the adapter, never a claim in a grant.** An operator can *waive*
//! the requirement for independent below-adapter enforcement; they cannot *assert* it. If a grant
//! string could say `attest=yes`, DL1911 would be a checkbox and invariant 50's "so double
//! enforcement is never silently single" would be a sentence rather than a mechanism.
//!
//! The one adapter that ships in-tree attests **false**, and that is the honest answer rather than
//! a limitation to apologise for: it runs in this process, so there is no layer below it that
//! could independently enforce anything. Its envelope checks are the same code, in the same
//! address space, as the thing being bounded. See [`REFERENCE_ADAPTER`].

/// What the runtime knows about a compute adapter before any device is granted.
#[derive(Clone, Debug)]
pub struct AdapterInfo {
    pub name: &'static str,
    /// Does this adapter attest **independent below-adapter** envelope enforcement — a layer
    /// beneath it that would refuse an over-envelope submission even if the adapter itself were
    /// wrong, absent, or lying? For a real GPU runtime this is a claim about the driver and
    /// firmware. For anything running in this process the answer is no, and must be no.
    pub attests_below_adapter: bool,
    /// Kernel artifact formats this adapter accepts.
    pub formats: &'static [&'static str],
}

/// The in-tree CPU reference adapter (spec §7.2). Deterministic, requires no hardware, and makes
/// the vendor-neutral interface conformance-testable on every CI run with or without silicon.
///
/// `attests_below_adapter: false` is not a stub awaiting improvement. This adapter *is* the
/// enforcement; there is nothing underneath it. Claiming otherwise would make the reference
/// implementation the one place the honesty rule is broken, and every later adapter would inherit
/// the precedent.
pub const REFERENCE_ADAPTER: AdapterInfo = AdapterInfo {
    name: "cpu-reference",
    attests_below_adapter: false,
    formats: &["refkernel-1"],
};

/// Every adapter this build knows. A grant naming anything else is refused — an adapter the
/// runtime has never heard of certainly cannot attest anything about its own enforcement, and
/// "unknown" is not a reason to proceed (the skip branch, closed).
pub const ADAPTERS: &[AdapterInfo] = &[REFERENCE_ADAPTER];

pub fn adapter(name: &str) -> Option<&'static AdapterInfo> {
    ADAPTERS.iter().find(|a| a.name == name)
}

/// Why a compute grant was refused before the program ever ran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrantRefusal {
    /// The named adapter is not one this build knows (DL1911 — it cannot attest what it is not).
    UnknownAdapter(String),
    /// The adapter does not attest independent below-adapter enforcement and no human waived the
    /// requirement (DL1911). The waiver is deliberately awkward to supply: it is a policy
    /// decision, and it belongs to a person.
    NoAttestation { adapter: String },
    /// The adapter cannot accept a kernel format the grant lists.
    UnsupportedFormat { adapter: String, format: String },
}

impl GrantRefusal {
    pub fn message(&self) -> String {
        match self {
            GrantRefusal::UnknownAdapter(a) => format!(
                "compute adapter `{a}` is not one this build knows — an unknown adapter cannot \
                 attest anything about enforcement below itself, so the grant is refused rather \
                 than taken on trust"
            ),
            GrantRefusal::NoAttestation { adapter } => format!(
                "compute adapter `{adapter}` does not attest independent below-adapter envelope \
                 enforcement, so the envelope would be enforced in exactly one place. Spec §7.1 \
                 claims the check happens host-side AND adapter-side; granting this device without \
                 saying so out loud would make that claim silently single. Add \
                 `waive-attestation` to the grant to accept single enforcement deliberately"
            ),
            GrantRefusal::UnsupportedFormat { adapter, format } => format!(
                "compute adapter `{adapter}` does not accept kernel format `{format}`"
            ),
        }
    }
}

// ----- kernels are data (spec §7.1, the spirit of audit rule R-6a) ------------------------------

/// A kernel artifact, loaded and verified before the program runs.
///
/// **A kernel is DATA.** It arrives as an opaque artifact on disk — for a real accelerator a PTX or
/// SPIR-V blob, for the reference adapter a one-line declaration — named in the grant, hashed, and
/// signed. What it can never be is a DeluluLang closure: `dispatch` takes the kernel's NAME as a
/// `Str`, so a function value is a type error at the call site, and this loader only ever reads
/// bytes off the filesystem. There is no path by which code from the row system becomes a kernel,
/// and that is the point of the rule rather than a happy accident of the implementation.
#[derive(Clone, Debug)]
pub struct KernelArtifact {
    /// The name programs dispatch by.
    pub name: String,
    /// `blake3:…` over the artifact's bytes — provenance, reported in the authority answer.
    pub hash: String,
    /// The ed25519 public key that signed it, lowercase hex.
    pub signer: String,
    /// The operation the artifact declares. For `refkernel-1` this is one of the reference
    /// adapter's reductions; a real adapter would carry a compiled blob instead.
    pub op: String,
}

/// Why a kernel artifact was refused before the program ran.
#[derive(Clone, Debug)]
pub enum KernelRefusal {
    /// The file could not be read at all.
    Unreadable { name: String, path: String, why: String },
    /// The bytes are not a kernel artifact this build understands.
    Malformed { name: String, why: String },
    /// No detached signature beside the artifact (DL1913).
    Unsigned { name: String, path: String },
    /// A signature is present but does not verify (DL1912).
    BadSignature { name: String, why: String },
}

impl KernelRefusal {
    /// The diagnostic code. Unsigned and invalid are DIFFERENT codes with different remedies —
    /// "sign this" versus "these bytes are not what they claim" — following the DL1510/DL1511
    /// precedent exactly. Reusing one code would make one of the two messages a lie.
    pub fn code(&self) -> &'static str {
        match self {
            KernelRefusal::Unsigned { .. } => "DL1913",
            KernelRefusal::BadSignature { .. } => "DL1912",
            // A file that cannot be read or parsed is not a signature problem; it is a grant that
            // names something which is not a kernel artifact.
            KernelRefusal::Unreadable { .. } | KernelRefusal::Malformed { .. } => "DL1912",
        }
    }

    pub fn message(&self) -> String {
        match self {
            KernelRefusal::Unreadable { name, path, why } => {
                format!("kernel `{name}`: cannot read the artifact at `{path}`: {why}")
            }
            KernelRefusal::Malformed { name, why } => {
                format!("kernel `{name}`: {why}")
            }
            KernelRefusal::Unsigned { name, path } => format!(
                "kernel `{name}` is unsigned — expected a detached signature at `{path}.sig`. \
                 Kernels are foreign code that runs on hardware this program cannot otherwise \
                 reach; an unsigned one has no provenance at all, and provenance is the only thing \
                 DeluluLang can offer about a kernel's behaviour"
            ),
            KernelRefusal::BadSignature { name, why } => format!(
                "kernel `{name}`: the signature does not verify — {why}. The artifact is not what \
                 it claims to be, which is a refusal regardless of policy"
            ),
        }
    }
}

/// Load and verify one kernel artifact (spec §7.1). Fail-closed at every branch.
///
/// The `refkernel-1` format is one line: `refkernel-1 <op>`. It is deliberately not a program —
/// the reference adapter maps `<op>` to a built-in reduction, and there is no interpreter here for
/// an artifact to smuggle behaviour through. A real adapter's format would be a compiled blob,
/// equally opaque to DeluluLang and equally outside its proof.
pub fn load_kernel(name: &str, path: &str, formats: &[String]) -> Result<KernelArtifact, KernelRefusal> {
    let bytes = std::fs::read(path).map_err(|e| KernelRefusal::Unreadable {
        name: name.to_string(),
        path: path.to_string(),
        why: e.to_string(),
    })?;
    // Signature FIRST: unverified bytes are not parsed. Reading structure out of an artifact
    // before knowing it is authentic is how a malformed-input bug becomes a supply-chain one.
    let sig_path = format!("{path}.sig");
    let sig = match std::fs::read(&sig_path) {
        Ok(s) => s,
        Err(_) => {
            // THE SKIP BRANCH. A missing signature file is not "unsigned, therefore skip the
            // check" — it IS the refusal. Criterion 7 asks that an unsigned kernel artifact be
            // refused, and the way that rule dies is an `if let Ok(sig)` with no `else`.
            return Err(KernelRefusal::Unsigned {
                name: name.to_string(),
                path: path.to_string(),
            });
        }
    };
    let signer = match crate::plugin::verify_detached(&bytes, &sig) {
        crate::plugin::SignatureStatus::Valid { signer } => signer,
        crate::plugin::SignatureStatus::Invalid { reason } => {
            return Err(KernelRefusal::BadSignature { name: name.to_string(), why: reason })
        }
        crate::plugin::SignatureStatus::Unsigned => {
            return Err(KernelRefusal::Unsigned {
                name: name.to_string(),
                path: path.to_string(),
            })
        }
    };
    let text = String::from_utf8_lossy(&bytes);
    let line = text.lines().find(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));
    let Some(line) = line else {
        return Err(KernelRefusal::Malformed {
            name: name.to_string(),
            why: "the artifact is empty".to_string(),
        });
    };
    let mut parts = line.split_whitespace();
    let format = parts.next().unwrap_or("");
    if !formats.iter().any(|f| f == format) {
        return Err(KernelRefusal::Malformed {
            name: name.to_string(),
            why: format!(
                "the artifact declares format `{format}`, which this device's grant does not list \
                 (it lists: {})",
                formats.join(", ")
            ),
        });
    }
    let Some(op) = parts.next() else {
        return Err(KernelRefusal::Malformed {
            name: name.to_string(),
            why: format!("the `{format}` artifact names no operation"),
        });
    };
    if !REFERENCE_OPS.contains(&op) {
        return Err(KernelRefusal::Malformed {
            name: name.to_string(),
            why: format!(
                "`{op}` is not an operation the reference adapter implements (it implements: {})",
                REFERENCE_OPS.join(", ")
            ),
        });
    }
    Ok(KernelArtifact {
        name: name.to_string(),
        hash: delulu_broker::content_hash(&bytes),
        signer,
        op: op.to_string(),
    })
}

/// The reductions the reference adapter implements. Buffer in, scalar out — the honest minimum
/// shape of "the host drives; devices get buffers".
pub const REFERENCE_OPS: &[&str] = &["reduce_sum", "reduce_max", "reduce_min", "dot_self", "burn"];

// ----- the dispatch path ------------------------------------------------------------------------

/// Why a dispatch did not reach the device. Both become `ComputeErr` VALUES in the program — the
/// dispatch dies, the process lives, exactly as an over-envelope actuator command does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchRefusal {
    /// Outside the granted envelope (DL1907).
    Envelope(String),
    /// A kernel name this grant never carried. Distinct from `Envelope` because the mistakes
    /// differ: one asked too much of a kernel it holds, the other named one it does not.
    UnknownKernel(String),
}

/// One dispatch's outcome, for the trace.
#[derive(Clone, Debug)]
pub struct DispatchRecord {
    pub device: String,
    pub kernel: String,
    pub elements: usize,
    pub micros: u64,
    /// The refusal, as `(diagnostic code, reason)`. The code matters: DL1907 is the ENVELOPE
    /// refusal, and stamping it on a kernel the grant never carried would tell an auditor counting
    /// DL1907s that a device was over-driven when in fact a program asked for something that was
    /// never on the menu. Different mistakes, different codes (the 10f lesson about `overdue`).
    pub refused: Option<(&'static str, String)>,
}

/// The compute broker for one run: the granted envelopes, the in-flight counter, and the journal.
///
/// This is the whole vendor-neutral interface as it exists today — bound, submit, report. It is
/// deliberately small: an interface an adapter cannot need a privileged hook into is one no vendor
/// can quietly become a first-class citizen of (invariant 49).
pub struct ComputeBroker {
    devices: Vec<crate::value::ComputeEnvelope>,
    /// Verified artifacts, per device: `device -> [KernelArtifact]`. A dispatch resolves its
    /// kernel HERE, not from the grant string, so only a signed artifact can ever run.
    artifacts: std::collections::BTreeMap<String, Vec<KernelArtifact>>,
    in_flight: std::sync::Mutex<std::collections::BTreeMap<String, u32>>,
    journal: std::sync::Mutex<Vec<DispatchRecord>>,
}

impl ComputeBroker {
    pub fn new(devices: &[crate::value::ComputeEnvelope]) -> ComputeBroker {
        ComputeBroker {
            devices: devices.to_vec(),
            artifacts: std::collections::BTreeMap::new(),
            in_flight: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            journal: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Attach the verified artifacts for one device (done by the pre-flight, after `load_kernel`).
    pub fn with_artifacts(mut self, device: &str, arts: Vec<KernelArtifact>) -> ComputeBroker {
        self.artifacts.insert(device.to_string(), arts);
        self
    }

    /// Every verified artifact, for the authority report's provenance line.
    pub fn artifacts(&self) -> &std::collections::BTreeMap<String, Vec<KernelArtifact>> {
        &self.artifacts
    }

    pub fn records(&self) -> Vec<DispatchRecord> {
        self.journal.lock().unwrap().clone()
    }

    /// Submit one kernel. The envelope is checked HOST-SIDE, before submission, in the order a
    /// human would ask about it: do you hold this kernel at all, does the buffer fit, is there
    /// room in the queue — and only then is any work done.
    ///
    /// The kernel-time budget is the one term that cannot be checked before the fact: you learn
    /// how long a kernel took by running it. So it is enforced AFTER, on the measurement, and the
    /// result is discarded when it overruns. That ordering is stated rather than hidden, because
    /// "enforced" means something weaker here than for the other three, and a reader deserves to
    /// know which kind of bound they are relying on.
    pub fn dispatch(
        &self,
        device: &str,
        kernel: &str,
        buffer: &[f64],
    ) -> Result<f64, DispatchRefusal> {
        let Some(env) = self.devices.iter().find(|d| d.device == device) else {
            return Err(DispatchRefusal::UnknownKernel(format!(
                "no compute device `{device}` in this run"
            )));
        };
        let refuse = |reason: String| -> Result<f64, DispatchRefusal> {
            self.journal.lock().unwrap().push(DispatchRecord {
                device: device.to_string(),
                kernel: kernel.to_string(),
                elements: buffer.len(),
                micros: 0,
                refused: Some(("DL1907", reason.clone())),
            });
            Err(DispatchRefusal::Envelope(reason))
        };

        // Provenance first: kernels are DATA, and the name resolves to a VERIFIED ARTIFACT rather
        // than to the grant string. A device with no loaded artifacts can dispatch nothing, which
        // is the correct answer when signature verification refused them all.
        let art = self
            .artifacts
            .get(device)
            .and_then(|arts| arts.iter().find(|a| a.name == kernel))
            .cloned();
        let Some(art) = art else {
            let known: Vec<&str> = self
                .artifacts
                .get(device)
                .map(|a| a.iter().map(|k| k.name.as_str()).collect())
                .unwrap_or_default();
            self.journal.lock().unwrap().push(DispatchRecord {
                device: device.to_string(),
                kernel: kernel.to_string(),
                elements: buffer.len(),
                micros: 0,
                // NOT DL1907: this grant never carried the kernel, which is a provenance mistake,
                // not a device over-driven past its envelope.
                refused: Some(("kernel-not-granted", format!("kernel `{kernel}` is not in this grant"))),
            });
            return Err(DispatchRefusal::UnknownKernel(format!(
                "`{kernel}` is not a kernel this grant carries (it carries: {})",
                known.join(", ")
            )));
        };

        // Memory: the buffer is the allocation, 8 bytes per f64.
        let bytes = (buffer.len() as u64).saturating_mul(8);
        if bytes > env.memory_bytes {
            return refuse(format!(
                "`{kernel}` needs {bytes} bytes for {} elements; `{device}` is granted \
                 memory_bytes={}",
                buffer.len(),
                env.memory_bytes
            ));
        }

        // Queue depth: how much work may be in flight at once.
        {
            let mut q = self.in_flight.lock().unwrap();
            let cur = q.entry(device.to_string()).or_insert(0);
            if *cur >= env.queue_depth {
                let depth = env.queue_depth;
                return refuse(format!(
                    "`{device}` already has {cur} dispatches in flight; granted queue_depth={depth}"
                ));
            }
            *cur += 1;
        }
        let started = std::time::Instant::now();
        // The artifact's DECLARED op selects the reduction — not the name the program typed. A
        // dispatch runs what was signed, never what was asked for.
        let out = run_reference_kernel(&art.op, buffer);
        let micros = started.elapsed().as_micros() as u64;
        {
            let mut q = self.in_flight.lock().unwrap();
            if let Some(c) = q.get_mut(device) {
                *c = c.saturating_sub(1);
            }
        }

        let ms = micros as f64 / 1000.0;
        let (lo, hi) = env.kernel_ms;
        if ms > hi {
            return refuse(format!(
                "`{kernel}` ran {ms:.3} ms; `{device}` is granted kernel_ms={lo}..{hi} — the \
                 result is discarded"
            ));
        }
        self.journal.lock().unwrap().push(DispatchRecord {
            device: device.to_string(),
            kernel: kernel.to_string(),
            elements: buffer.len(),
            micros,
            refused: None,
        });
        Ok(out)
    }
}

/// The reference adapter's kernels. Deterministic, integer-stable where they can be, and few —
/// this exists to make the *interface* conformance-testable without silicon, not to be a compute
/// library. Every one of these is a reduction: buffer in, scalar out, which is the honest minimum
/// shape of "the host drives; devices get buffers".
fn run_reference_kernel(kernel: &str, buffer: &[f64]) -> f64 {
    match kernel {
        "reduce_sum" => buffer.iter().sum(),
        "reduce_max" => buffer.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        "reduce_min" => buffer.iter().copied().fold(f64::INFINITY, f64::min),
        "dot_self" => buffer.iter().map(|x| x * x).sum(),
        // `burn` exists so the `kernel_ms` budget has something to catch. The inner constant is
        // large enough that even a short buffer takes milliseconds on a fast machine — a test
        // whose kernel finishes under the budget it is meant to overrun proves the opposite of
        // what it claims, and passes while doing so.
        "burn" => {
            let mut acc = 0.0f64;
            for (i, x) in buffer.iter().enumerate() {
                for k in 0..200_000u32 {
                    acc += (x + k as f64 + i as f64).sqrt();
                }
            }
            acc
        }
        // Unreachable in a well-formed run: the grant enumerates kernel names and `dispatch`
        // checks against it before arriving here. Fail closed anyway rather than invent a number
        // — a fabricated result from a kernel that does not exist is the invariant-50 failure,
        // and it would look entirely normal in the output.
        _ => f64::NAN,
    }
}

/// What this adapter actually enforces, stated plainly so no doc has to be trusted about it.
///
/// | term | status |
/// |---|---|
/// | `memory_bytes` | **enforced** before submission, against the buffer |
/// | `kernel_ms` | **enforced**, but *after* the fact — you learn a kernel's duration by running it, so an overrun is caught on the measurement and the result discarded |
/// | `queue_depth` | **enforced** in this broker, and currently **unreachable from a program**: dispatch is synchronous, so a single-threaded run never has more than one in flight. Unit-tested with threads; the language surface cannot exceed it until async dispatch exists (RFC-gated) |
/// | `power_w` | **carried, NOT enforced.** This adapter draws no measurable power and has no way to attribute any. It is parsed, displayed, and refused-if-absent so a real adapter inherits the term — and it is not counted as a bound anything checks |
///
/// The last row is the one that matters. `power_w` is mandatory in the grant but enforced by
/// nothing here, and calling that "enforced" in a table nobody reads carefully is exactly how a
/// double-enforcement claim becomes silently single.
pub const ENFORCEMENT_NOTE: &str =
    "cpu-reference enforces memory_bytes and kernel_ms; queue_depth is enforced but unreachable \
     from synchronous dispatch; power_w is carried and NOT enforced by this adapter";

/// The DL1911 gate (spec §7.1, invariant 50): may this compute grant be issued at all?
///
/// Called at the pre-flight, before a line of the program runs, for the same reason 10e refuses a
/// zero-grant device before `main`: a program that cannot lawfully reach the silicon should not
/// get far enough to try.
pub fn check_grant(env: &crate::value::ComputeEnvelope) -> Result<(), GrantRefusal> {
    let Some(info) = adapter(&env.adapter) else {
        // THE SKIP BRANCH. An adapter the runtime cannot identify is not "probably fine" — it is
        // the case where the checker could not tell, and the answer to that is always no.
        return Err(GrantRefusal::UnknownAdapter(env.adapter.clone()));
    };
    for f in &env.formats {
        if !info.formats.contains(&f.as_str()) {
            return Err(GrantRefusal::UnsupportedFormat {
                adapter: env.adapter.clone(),
                format: f.clone(),
            });
        }
    }
    if !info.attests_below_adapter && !env.waived {
        return Err(GrantRefusal::NoAttestation { adapter: env.adapter.clone() });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ComputeEnvelope;

    /// The verified artifacts a test broker dispatches through. In a real run these come from
    /// `load_kernel`, which will not return one without a valid signature; a unit test constructs
    /// them directly because it is testing the dispatch path, not the loader.
    fn arts() -> Vec<KernelArtifact> {
        ["reduce_sum", "reduce_max", "burn"]
            .iter()
            .map(|op| KernelArtifact {
                name: op.to_string(),
                hash: format!("blake3:test-{op}"),
                signer: "test".to_string(),
                op: op.to_string(),
            })
            .collect()
    }

    fn broker(e: ComputeEnvelope) -> ComputeBroker {
        ComputeBroker::new(std::slice::from_ref(&e)).with_artifacts(&e.device, arts())
    }

    fn env(memory_bytes: u64, kernel_ms: (f64, f64), queue_depth: u32) -> ComputeEnvelope {
        ComputeEnvelope {
            device: "gpu0".to_string(),
            class: "unspecified".to_string(),
            adapter: "cpu-reference".to_string(),
            memory_bytes,
            kernel_ms,
            queue_depth,
            power_w: (0.0, 120.0),
            formats: vec!["refkernel-1".to_string()],
            kernels: vec![
                ("reduce_sum".to_string(), "reduce_sum.refkernel".to_string()),
                ("burn".to_string(), "burn.refkernel".to_string()),
            ],
            attested: false,
            waived: true,
        }
    }

    /// The reference adapter attests nothing about a layer below itself, and that must stay true.
    /// If this test ever "fails" because someone flipped the flag to make DL1911 quieter, the
    /// honesty rule died at the reference implementation and every adapter after it inherits the
    /// precedent.
    #[test]
    fn the_reference_adapter_does_not_claim_enforcement_beneath_itself() {
        // A `const` assertion, so flipping the flag fails the BUILD rather than a test somebody
        // could mark `#[ignore]`. cpu-reference runs in this process — there is no layer below it
        // that could independently enforce anything, and claiming otherwise would make the
        // reference implementation the one place this rule is broken, with every adapter written
        // afterwards inheriting the precedent.
        const { assert!(!REFERENCE_ADAPTER.attests_below_adapter) };
    }

    /// Determinism (spec §7.2): the interface must be conformance-testable without hardware, which
    /// means the same buffer through the same kernel gives the same answer, run after run.
    #[test]
    fn the_reference_adapter_is_deterministic() {
        let b = broker(env(4096, (0.0, 5000.0), 4));
        let buf: Vec<f64> = (0..64).map(|i| i as f64 * 0.5).collect();
        let first = b.dispatch("gpu0", "reduce_sum", &buf).expect("in envelope");
        for _ in 0..8 {
            assert_eq!(b.dispatch("gpu0", "reduce_sum", &buf).unwrap(), first);
        }
    }

    /// Provenance is checked BEFORE the buffer: a kernel this grant never carried is refused
    /// without anything looking at how big the work would have been. The refusal is
    /// `UnknownKernel`, never `Envelope` — naming a kernel you do not hold is a different mistake
    /// from asking too much of one you do, and an auditor counting envelope refusals must not see
    /// this one.
    #[test]
    fn a_kernel_the_grant_never_carried_is_refused_as_unknown_not_as_envelope() {
        let b = broker(env(8, (0.0, 5000.0), 4));
        // This buffer is far over `memory_bytes`, so if provenance were checked second the
        // refusal would come back as an envelope overrun and hide the real problem.
        let huge = vec![1.0f64; 4096];
        match b.dispatch("gpu0", "never_granted", &huge) {
            Err(DispatchRefusal::UnknownKernel(m)) => {
                assert!(m.contains("never_granted"), "{m}");
                assert!(m.contains("reduce_sum"), "the refusal lists what IS carried: {m}");
            }
            other => panic!("expected UnknownKernel, got {other:?}"),
        }
    }

    /// `queue_depth` is a real check, exercised here with threads because the language surface
    /// cannot reach it: dispatch is synchronous, so a single-threaded program never has more than
    /// one kernel in flight. Testing the mechanism directly is the honest way to claim it exists
    /// while `ENFORCEMENT_NOTE` says plainly that a program cannot currently exceed it.
    #[test]
    fn queue_depth_refuses_work_past_the_granted_depth() {
        let b = std::sync::Arc::new(broker(env(4096, (0.0, 60_000.0), 1)));
        let buf = vec![1.0f64; 4];
        let refused = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut hs = Vec::new();
        for _ in 0..4 {
            let (b, buf, refused) = (b.clone(), buf.clone(), refused.clone());
            hs.push(std::thread::spawn(move || {
                if let Err(DispatchRefusal::Envelope(m)) = b.dispatch("gpu0", "burn", &buf) {
                    assert!(m.contains("queue_depth"), "the refusal names the term: {m}");
                    refused.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }));
        }
        for h in hs {
            h.join().unwrap();
        }
        assert!(
            refused.load(std::sync::atomic::Ordering::SeqCst) > 0,
            "four concurrent dispatches against queue_depth=1 must refuse at least one"
        );
    }

    /// The control for the test above: at a depth that accommodates them, the same four concurrent
    /// dispatches all land. Without this, a broker that refused every concurrent dispatch would
    /// pass.
    #[test]
    fn a_generous_queue_depth_lets_the_same_concurrent_work_through() {
        let b = std::sync::Arc::new(broker(env(4096, (0.0, 60_000.0), 8)));
        let buf = vec![1.0f64; 4];
        let mut hs = Vec::new();
        for _ in 0..4 {
            let (b, buf) = (b.clone(), buf.clone());
            hs.push(std::thread::spawn(move || b.dispatch("gpu0", "reduce_sum", &buf).is_ok()));
        }
        let ok = hs.into_iter().map(|h| h.join().unwrap()).filter(|landed| *landed).count();
        assert_eq!(ok, 4, "queue_depth=8 must not refuse four dispatches");
    }

    /// Nothing in the dispatch path may branch on `class` or `adapter` (invariant 49). Two
    /// envelopes differing only in those fields must behave identically — the day one adapter's
    /// name changes an outcome is the day the authority model has a second class of citizen.
    #[test]
    fn nothing_branches_on_class_or_adapter_name() {
        let mut plain = env(4096, (0.0, 5000.0), 4);
        let mut fancy = env(4096, (0.0, 5000.0), 4);
        fancy.class = "gpu".to_string();
        fancy.adapter = "vendor-x-cuda".to_string();
        plain.class = "cpu".to_string();
        let buf: Vec<f64> = (0..32).map(|i| i as f64).collect();
        let a = broker(plain).dispatch("gpu0", "reduce_sum", &buf);
        let b = broker(fancy).dispatch("gpu0", "reduce_sum", &buf);
        assert_eq!(a.unwrap(), b.unwrap(), "the adapter's NAME must not change what it computes");
    }
}
