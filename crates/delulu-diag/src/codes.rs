//! The Stage-1 diagnostic code registry (spec §10.3).
//!
//! Codes are allocated within domain ranges and NEVER reused. Every diagnostic the compiler
//! produces must carry a code registered here — the conformance suite's meta-test asserts it.

pub struct CodeInfo {
    pub code: &'static str,
    pub title: &'static str,
}

macro_rules! registry {
    ( $( $code:literal => $title:literal ),* $(,)? ) => {
        pub const REGISTRY: &[CodeInfo] = &[
            $( CodeInfo { code: $code, title: $title } ),*
        ];
    };
}

registry! {
    // DL01xx — lexical
    "DL0101" => "unexpected character",
    "DL0102" => "unterminated string literal",
    "DL0103" => "invalid escape sequence",
    "DL0104" => "invalid numeric literal",
    "DL0105" => "unterminated block comment",
    "DL0106" => "reserved word used as a declared name",

    // DL02xx — parse
    "DL0201" => "expected a different token",
    "DL0202" => "expected an expression",
    "DL0203" => "expected a type",
    "DL0204" => "file must begin with a `module` declaration",
    "DL0205" => "expected a pattern",
    "DL0206" => "comparison operators are non-associative",
    "DL0207" => "invalid assignment target",
    "DL0208" => "expected an item",
    "DL0209" => "expected a statement terminator",

    // DL03xx — names / modules
    "DL0301" => "unknown name",
    "DL0302" => "duplicate definition",
    "DL0303" => "unknown module in import",
    "DL0304" => "import cycle",
    "DL0305" => "module-level mutable state is forbidden",
    "DL0306" => "unknown effect name",
    "DL0307" => "not a capability resource kind",

    // DL04xx — types
    "DL0401" => "type mismatch",
    "DL0402" => "mixed numeric types (no implicit coercion)",
    "DL0403" => "wrong number of arguments",
    "DL0404" => "not callable",
    "DL0405" => "unknown field or method",
    "DL0406" => "wrong number of type arguments",
    "DL0407" => "non-exhaustive match",
    "DL0408" => "condition must be Bool",
    "DL0409" => "`?` requires Result in a Result-returning function",
    "DL0410" => "generic variable used as both type and effect row",

    // DL05xx — effects
    "DL0501" => "function performs an effect not declared in its row",
    "DL0502" => "declared effect never performed",
    "DL0503" => "more than one row variable per signature",
    "DL0504" => "conflicting bindings for row variable (rows never union-merge)",

    // DL06xx — capabilities & secrets
    "DL0601" => "capability type cannot be constructed or forged",
    "DL0602" => "secret value cannot flow here (Secret[T] is not T)",
    "DL0603" => "Secret.map requires a pure function",
    "DL0604" => "opaque type cannot be stringified or serialized",
    "DL0605" => "opaque type has no structural equality",

    // DL07xx — manifest / authority
    "DL0701" => "main's effect row exceeds the authority manifest",
    "DL0702" => "authority grant refused",
    "DL0703" => "root slice not granted",

    // DL08xx — plugins & grants (types reserved in Stage 1; runtime lands Stage 6)
    "DL0801" => "call through revoked plugin reference",
    "DL0802" => "grant exceeds holder's grant (attenuation violation)",
    "DL0803" => "function-typed argument to Contained plugin export",

    // DL10xx — packages / authority versioning (Stage 2)
    "DL1001" => "dependency authority exceeds its pin",
    "DL1002" => "locked authority hash mismatch (same version, different authority)",
    "DL1003" => "semver-authority violation: authority widened without a major version bump",
    "DL1004" => "malformed package manifest",
    "DL1005" => "package or re-export cycle",
    "DL1006" => "import is ambiguous between a local module and a dependency",
    "DL1007" => "git dependency without a pinned rev or tag",
    "DL1008" => "version conflict for one package name in the graph",
    "DL1009" => "package performs an effect not permitted by its own authority manifest",
    "DL1010" => "content hash mismatch (source changed under a locked version)",
    "DL1011" => "locked build requires resolution not present in delulu.lock",

    // DL11xx — trace / fuzz harness (Stage 2)
    "DL1101" => "effect-trace assertion violation (a runtime effect not in the static row)",
    "DL1102" => "repair did not produce an accepting program (fuzz harness)",

    // DL12xx — WASM backend / .dwx artifact (Stage 3)
    "DL1201" => "construct not supported by the WASM backend (runs on the interpreter instead)",
    "DL1202" => "artifact missing or invalid `delulu:authority` section",
    "DL1204" => "delulu:cap interface version unsupported by this toolchain",
    "DL1205" => "secret contents cannot enter the WASM guest (`--target wasm`)",
    "DL1206" => "engine parity self-check failure (compiler-bug class)",

    // DL13xx — foreign (C FFI + embedded Python) (Stage 4)
    "DL1301" => "unmarshallable type in a foreign signature (incl. Secret/opaque)",
    "DL1302" => "function-typed value crossing the foreign boundary (no callbacks — rule R-6a)",
    "DL1303" => "foreign lib used without a manifest entry or runtime grant",
    "DL1304" => "foreign symbol not found at bind time",
    "DL1305" => "python import not in the granted allowlist",
    "DL1306" => "foreign return failed shape validation (encoding or size)",
    "DL1307" => "python runtime unavailable",
    "DL1308" => "unsupported ABI string in a `foreign` block",

    // DL14xx — custody / broker (Stage 5). DL0802 (Stage-1 registry) activates from this stage.
    // NOTE: there is deliberately NO DL1404 — the range skips it (spec §8). Do not invent one.
    "DL1401" => "broker unreachable / protocol failure (fail closed)",
    "DL1402" => "lease expired (TTL)",
    "DL1403" => "lease revoked (carries the revoking audit seq)",
    "DL1405" => "audit chain verification failure",
    "DL1406" => "broker protocol version mismatch",
    "DL1407" => "delegation token invalid or already redeemed",
    "DL1408" => "isolation profile unavailable on this platform",
    "DL1409" => "foreign worker died (process isolation) — the isolated worker crashed; the host survived",

    // DL09xx — runtime
    "DL0901" => "integer overflow",
    "DL0902" => "division by zero",
    "DL0903" => "index out of bounds",
    "DL0904" => "capability scope violation",
    "DL0905" => "recursion depth exceeded",
    "DL0906" => "explicit panic",
    "DL0907" => "match reached no arm (checker bug if ever seen)",
}

pub fn is_registered(code: &str) -> bool {
    REGISTRY.iter().any(|c| c.code == code)
}

pub fn code_title(code: &str) -> Option<&'static str> {
    REGISTRY.iter().find(|c| c.code == code).map(|c| c.title)
}

/// The reachability-not-behavior honesty caveat carried, verbatim from spec §10, by EVERY DL13xx
/// explain text (Stage 4 trap 1). The word "sandbox" is deliberately absent: Stage 4 bounds
/// reachability, not behavior, and every foreign diagnostic links forward to Stage 5 for actual
/// containment.
const FOREIGN_CAVEAT: &str =
    "Honesty (spec §10): Foreign code is outside the effect guarantee; the language bounds \
     reachability (grant + capability + ForeignCall in every row), not behavior. Containment of \
     behavior is process-level until Stage 5's foreign workers, and microVM-level after.";

/// The Stage-5 revocation latency bound, VERBATIM from spec §4.2 (normative — playbook trap 3:
/// never claim "immediate"). This exact wording appears in `delulu explain E-REVOKE` and in the
/// audit record of every revocation; no stronger claim is made anywhere.
pub const REVOCATION_BOUND: &str = "synchronous class — before the next use; epoch class — within \
     one epoch interval (≤ 50 ms default)";

/// A named explanation topic (not a diagnostic code): `delulu explain E-REVOKE`. Returns
/// `(title, body)`. Topics carry normative honesty text the spec mandates verbatim (Stage 5
/// playbook 5j); unlike codes they explain a *semantics*, not a single failure.
pub fn topic_explain(topic: &str) -> Option<(&'static str, String)> {
    match topic {
        "REVOKE" => Some((
            "revocation semantics and the stated latency bound (spec §4.2, normative)",
            format!(
                "Revocation takes effect: {REVOCATION_BOUND}. This bound appears in `delulu \
                 explain E-REVOKE` and in the audit record of every revocation. No stronger claim \
                 is made anywhere.\n\n\
                 Synchronous class — validated by a broker round-trip per use: Declassify \
                 (expose), FsWrite ops, Net ops, foreign bind, plugin load (Stage 6), and later \
                 Actuate (Stage 10). Epoch class — validated locally against cached scope data + \
                 a revocation epoch the runtime refreshes at most every 50 ms (config \
                 `--epoch-ms`, ceiling 250): Read ops, Clock, Rand, Console.\n\n\
                 Honesty (spec §10): Revocation bounds are those of §4.2 — \"immediate\" is never \
                 claimed."
            ),
        )),
        _ => None,
    }
}

/// A longer, human-facing explanation for a diagnostic code, printed by `delulu explain <code>`
/// beneath the title. Only codes whose behavior carries a normative caveat define one; the rest
/// return `None` and `explain` prints the title alone. Every DL13xx (foreign) explanation appends
/// [`FOREIGN_CAVEAT`] verbatim (README honesty clause 4 / Stage-4 trap 1).
pub fn code_explain(code: &str) -> Option<String> {
    let body = match code {
        "DL1301" => "A `foreign` signature may only marshal `Int`, `Float`, `Bool`, `Str`, `Unit`, \
             and `ForeignPtr`. `Secret[T]`, `Cap[R]`, `Root`, `Plugin[_]`, `PyObj`, and every other \
             opaque type are refused: a secret must never cross to foreign code (invariant 20), and \
             the repair list never suggests `expose` — laundering a secret across the FFI is exactly \
             what the language exists to prevent.",
        "DL1302" => "A function-typed value cannot cross the foreign boundary in either direction — \
             no callbacks, by rule R-6a (soundness audit F-6): unverifiable code holding a re-entry \
             point into verified code cannot be bounded by any effect row. This is a permanent rule, \
             not a deferred feature. The escape valve is inverted control: DeluluLang drives the \
             loop and passes data, not code.",
        "DL1303" => "This program reaches a `foreign` lib with no matching `[authority] foreign.c` \
             manifest entry or no `--grant foreign.c=LOGICAL:PATH`. The program names *what* it \
             wants (a logical lib and its symbols); the human/broker decides *which binary* — the \
             path is grant data, never program data. Deny-by-default: an ungranted lib is refused at \
             the grant flow, before the program runs, never mid-run.",
        "DL1304" => "A symbol the `foreign` block declares was not found in the granted library at \
             bind time. All declared symbols resolve up front, fail-fast — a program that binds \
             successfully never surprises you with a missing symbol during a later call. Check the \
             symbol name and that the granted binary actually exports it.",
        "DL1305" => "This program's `py.import(name)` names a module not permitted by the granted \
             `foreign.python` allowlist. Add the pattern to `[authority] foreign.python` and grant it \
             (`--grant foreign.python=numpy` or `--grant \"foreign.python=numpy.*\"`; a pattern is an \
             exact name or a `prefix.*` wildcard). The import allowlist gates the interface — what \
             the DeluluLang program may reach for by name. It does not bound what Python code \
             transitively imports or does once running: embedded Python has full process authority at \
             the OS level. The real bounds are (a) the reachability gate (no Cap[Python], no Python at \
             all) and (b) the process/worker/microVM layer (Stage 5). This is the Constitution §5.12 \
             degradation, stated where users will actually read it.",
        "DL1307" => "The embedded CPython interpreter is unavailable: this build of delulu was made \
             without the `python` feature, or the interpreter failed to initialize on the host. This \
             surfaces as a `ForeignErr::Unavailable` value from `root.python(load)` — a `Result` \
             error, never a crash. Install a compatible CPython (or use a delulu built with the \
             `python` feature) and try again.",
        "DL1306" => "A value returned from foreign code failed shape validation at the boundary: a \
             returned string was not valid UTF-8, or exceeded `--foreign-max-ret` (default 64 MiB) \
             with no terminator. Returned foreign data is validated for *shape*, not *meaning* — it \
             is untrusted input and programs should treat it accordingly. A C library can corrupt or \
             crash the process; memory safety across the FFI is not claimed — validation covers \
             returned data, not the callee's memory safety.",
        "DL1308" => "The only ABI string supported in v0.4 is `\"c\"`. Other ABIs (C++, structs by \
             value, varargs) are post-1.0 RFCs.",
        // ----- DL14xx custody/broker (Stage 5). The §10 honesty caveats appear word-for-word
        // in the relevant bodies below (playbook 5j: "Copy spec §10 caveats into docs and
        // explain-text word-for-word").
        "DL1401" => "The broker daemon could not be reached (or a broker-protocol failure \
             occurred) while this run's custody lives in the daemon. Effectful operations FAIL \
             CLOSED (invariant 27): they never fall back to embedded custody, silently or \
             otherwise. Start the broker with `delulu broker start` and re-run. Honesty (spec \
             §10): The broker defends against the program and its delegates, not against the OS \
             user: any process running as the same user with default OS permissions could read \
             the broker key. The boundary is process compromise and code-behavior, per the threat \
             model — not local-user malware, not root, not the kernel, not the hypervisor, not \
             microarchitectural channels (Constitution §5.14 unchanged).",
        "DL1402" => "The lease's TTL deadline has passed; the node no longer authorizes anything. \
             Re-delegate a fresh lease (`delulu grants delegate …`) — a human/orchestrator \
             decision, never automatic. Honesty (spec §10): TTLs bound duration of compromise, \
             not its existence.",
        "DL1403" => "The lease was revoked; the denial carries the revoking operation's audit \
             sequence number, so the JSON error states why and when your authority died (spec \
             §4.3 — feedback quality for agents is a feature). Revocation takes effect: \
             synchronous class — before the next use; epoch class — within one epoch interval \
             (≤ 50 ms default) — see `delulu explain E-REVOKE`. Honesty (spec §10): Revocation \
             bounds are those of §4.2 — \"immediate\" is never claimed.",
        "DL1405" => "The hash-chained audit log failed verification at the reported sequence \
             number: a recomputed record hash or a cross-record `prev_hash` link did not match. \
             The log may have been tampered with — a human must inspect it. Honesty (spec §10): \
             The audit log detects tampering after the fact; it does not prevent it. The log is \
             observability, not enforcement: no authority decision ever reads it.",
        "DL1406" => "The client and the broker daemon speak different wire-protocol versions \
             (`broker/N`). The daemon answers the mismatch and never acts on the request. \
             Upgrade so both ends are the same `delulu` build, then retry.",
        "DL1407" => "The delegation lease token failed to redeem: a bad or tampered MAC, a token \
             signed by a rotated-away broker key, a malformed token, a token bound to an unknown \
             node, or a second redemption of a single-use token (mint with `--multi` for \
             multi-redemption). Re-mint via `delulu grants delegate …` — a human/orchestrator \
             decision. Note that `delulu broker rotate-key` deliberately invalidates ALL \
             outstanding tokens.",
        "DL1408" => "The requested isolation profile is not available here. `--isolation microvm` \
             requires Linux (x86_64/aarch64) with KVM and a provisioned microVM runtime \
             (Firecracker/cloud-hypervisor); everywhere else it is refused — never silently \
             approximated. The documented fallback is `--isolation process`: worker-style OS \
             containment of the code outside the proof; explicitly weaker — no guest boundary, \
             no virtio-fs scope mounts, no default-deny egress. Honesty (spec §10): Foreign \
             workers bound blast radius, not foreign behavior; the microVM profile is the strong \
             container and it is Linux-first — the fallback matrix is honest about weaker \
             platforms.",
        "DL1409" => "Under `--foreign-isolation process` a granted C library runs in an isolated \
             worker subprocess. This worker died mid-call — a segfault, an abort, a hard crash in the \
             native code. That is exactly the blast-radius containment the process-isolation profile \
             exists to provide: the crash was confined to the worker, and the host process survived \
             and turned the worker's death into this clean fault instead of dying alongside it. Re-run \
             with `--foreign-isolation inproc` to reproduce the crash in-process for debugging (Stage \
             4 behaviour — a crash there takes the whole process down). Foreign workers bound blast \
             radius, not foreign behaviour (spec §10).",
        _ => return None,
    };
    // Every foreign (DL13xx) explanation carries the reachability-not-behavior caveat verbatim.
    if code.starts_with("DL13") {
        Some(format!("{body}\n\n{FOREIGN_CAVEAT}"))
    } else {
        Some(body.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique_and_well_formed() {
        for (i, a) in REGISTRY.iter().enumerate() {
            assert!(a.code.starts_with("DL") && a.code.len() == 6, "bad code {}", a.code);
            assert!(a.code[2..].chars().all(|c| c.is_ascii_digit()));
            for b in &REGISTRY[i + 1..] {
                assert_ne!(a.code, b.code, "duplicate code {}", a.code);
            }
        }
    }

    #[test]
    fn lookup_works() {
        assert!(is_registered("DL0501"));
        assert!(!is_registered("DL9999"));
        assert_eq!(code_title("DL0305"), Some("module-level mutable state is forbidden"));
    }

    /// Stage 5 playbook trap 3: `explain E-REVOKE` states the spec §4.2 latency bound VERBATIM —
    /// and never claims "immediate" (the word appears only inside the §10 quote denying the claim).
    #[test]
    fn e_revoke_topic_states_the_4_2_bound_verbatim() {
        let (title, body) = topic_explain("REVOKE").expect("E-REVOKE topic exists");
        assert!(title.contains("§4.2"));
        assert!(
            body.contains(
                "Revocation takes effect: synchronous class — before the next use; epoch class — \
                 within one epoch interval (≤ 50 ms default)."
            ),
            "the §4.2 bound must appear verbatim: {body}"
        );
        assert!(body.contains("No stronger claim is made anywhere."), "{body}");
        assert!(body.contains("\"immediate\" is never claimed"), "the §10 caveat, word-for-word: {body}");
        assert!(topic_explain("NOPE").is_none());
    }

    /// Every DL14xx custody code has a longer explain body, and the §10 caveats it must carry
    /// appear word-for-word (playbook 5j).
    #[test]
    fn dl14xx_explains_carry_the_spec_10_caveats() {
        for code in ["DL1401", "DL1402", "DL1403", "DL1405", "DL1406", "DL1407", "DL1408", "DL1409"] {
            assert!(code_explain(code).is_some(), "{code} needs an explain body");
        }
        assert!(code_explain("DL1401").unwrap().contains("not against the OS user"));
        assert!(code_explain("DL1402").unwrap().contains("TTLs bound duration of compromise, not its existence"));
        assert!(code_explain("DL1403").unwrap().contains("\"immediate\" is never claimed"));
        assert!(code_explain("DL1405").unwrap().contains("detects tampering after the fact; it does not prevent it"));
        assert!(code_explain("DL1408").unwrap().contains("Foreign workers bound blast radius, not foreign behavior"));
        assert!(code_explain("DL1408").unwrap().contains("--isolation process"), "the fallback command is shown");
    }
}
