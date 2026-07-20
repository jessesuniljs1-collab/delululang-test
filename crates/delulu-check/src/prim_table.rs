//! The declarative primitive-table index (Stage 9, invariant 42 — the coverage law).
//!
//! The typing half of the primitive table (spec §7.3) is `check::Checker::method_sig`, a set of
//! match arms — real code, not data. This module is its **machine-readable index**: one
//! [`PrimEntry`] per `(receiver, method)` pair that `method_sig` recognizes, so the conformance
//! coverage tool can enumerate every primitive anchor `ref.prim.<receiver>.<method>` without
//! hand-copying a list that would silently drift.
//!
//! The `arity` column is NORMATIVE: the checker's too-many-arguments gate (DL0403) reads it at
//! the method-call site. `method_sig`'s per-position `expect_arg` calls catch too-few and
//! mistyped arguments but can never see surplus ones — that skip branch (`root.console(1,2,3,4,5)`
//! minting a cap and ignoring the noise) was exactly the fail-open hole the coverage law caught.
//!
//! Drift is fenced by the tests below: every listed entry RESOLVES in `method_sig`; an *unknown*
//! method must be `DL0405` (skip-branch); the arity column is proven against the gate in both
//! directions (exact arity passes the gate, arity+1 trips it) for every constructible receiver.
//! A count tripwire forces any add/remove of an arm to be a deliberate edit here.

/// One primitive-table entry: a method the checker's `method_sig` resolves on a given receiver.
pub struct PrimEntry {
    /// A stable receiver label used in the anchor id. Matches the receiver `Type` in `method_sig`:
    /// `root` = `Root`; `console`/`fs_read`/`fs_write`/`http`/`clock`/`rand` = the matching
    /// `Cap[_]`; `python` = `Cap[Python]`; `pyobj` = `PyObj`; `secret` = `Secret[_]`;
    /// `str`/`list` = the builtin `Str`/`List`; `plugin` = `Plugin[_]`.
    pub receiver: &'static str,
    /// The method name, exactly as `method_sig` matches it.
    pub method: &'static str,
    /// The exact argument count (spec §7.3). Normative: the checker rejects calls with more
    /// arguments than this as DL0403.
    pub arity: u8,
}

impl PrimEntry {
    /// The stable conformance anchor id, e.g. `ref.prim.console.println`.
    pub fn anchor(&self) -> String {
        format!("ref.prim.{}.{}", self.receiver, self.method)
    }
}

/// Every primitive `method_sig` recognizes, grouped by receiver in source order. Adding or
/// removing a `method_sig` arm requires the matching edit here — the `count_is_pinned` tripwire
/// and the `every_listed_primitive_resolves` guard both fail otherwise.
pub const PRIM_TABLE: &[PrimEntry] = &[
    // Root — mints capabilities (attenuation, pure).
    PrimEntry { receiver: "root", method: "console", arity: 0 },
    PrimEntry { receiver: "root", method: "fs_read", arity: 1 },
    PrimEntry { receiver: "root", method: "fs_write", arity: 1 },
    PrimEntry { receiver: "root", method: "http", arity: 1 },
    PrimEntry { receiver: "root", method: "clock", arity: 0 },
    PrimEntry { receiver: "root", method: "rand", arity: 0 },
    PrimEntry { receiver: "root", method: "declassify", arity: 0 },
    PrimEntry { receiver: "root", method: "foreign_load", arity: 0 },
    PrimEntry { receiver: "root", method: "plugin_host", arity: 0 },
    PrimEntry { receiver: "root", method: "secret", arity: 1 },
    PrimEntry { receiver: "root", method: "foreign", arity: 1 },
    PrimEntry { receiver: "root", method: "python", arity: 1 },
    // Cap[Console].
    PrimEntry { receiver: "console", method: "println", arity: 1 },
    PrimEntry { receiver: "console", method: "print", arity: 1 },
    PrimEntry { receiver: "console", method: "readline", arity: 0 },
    // Cap[FsRead].
    PrimEntry { receiver: "fs_read", method: "read_text", arity: 1 },
    PrimEntry { receiver: "fs_read", method: "list_dir", arity: 1 },
    PrimEntry { receiver: "fs_read", method: "narrow", arity: 1 },
    // Cap[FsWrite].
    PrimEntry { receiver: "fs_write", method: "write_text", arity: 2 },
    PrimEntry { receiver: "fs_write", method: "append_text", arity: 2 },
    // Cap[Http].
    PrimEntry { receiver: "http", method: "get", arity: 1 },
    // Cap[Clock].
    PrimEntry { receiver: "clock", method: "now_ms", arity: 0 },
    // Root device mints + Cap[Actuator]/Cap[Sensor] (Stage 10 phase 10e, Track D).
    PrimEntry { receiver: "root", method: "actuator", arity: 1 },
    PrimEntry { receiver: "root", method: "sensor", arity: 1 },
    PrimEntry { receiver: "actuator", method: "command", arity: 1 },
    PrimEntry { receiver: "sensor", method: "read", arity: 0 },
    // Compute devices (Stage 10 phase 10h, Track F). `dispatch` takes the kernel NAME and a
    // buffer — never a function, so no closure can cross into a kernel (§7.1).
    PrimEntry { receiver: "root", method: "compute", arity: 1 },
    PrimEntry { receiver: "compute", method: "dispatch", arity: 2 },
    // Cap[Rand].
    PrimEntry { receiver: "rand", method: "int", arity: 2 },
    PrimEntry { receiver: "rand", method: "float", arity: 0 },
    // Plugin[_] (Stage 6).
    PrimEntry { receiver: "plugin", method: "get", arity: 1 },
    PrimEntry { receiver: "plugin", method: "unload", arity: 0 },
    // Cap[Python] (Stage 4).
    PrimEntry { receiver: "python", method: "import", arity: 1 },
    PrimEntry { receiver: "python", method: "of_int", arity: 1 },
    PrimEntry { receiver: "python", method: "of_float", arity: 1 },
    PrimEntry { receiver: "python", method: "of_str", arity: 1 },
    PrimEntry { receiver: "python", method: "of_bool", arity: 1 },
    PrimEntry { receiver: "python", method: "list", arity: 1 },
    PrimEntry { receiver: "python", method: "to_int", arity: 1 },
    PrimEntry { receiver: "python", method: "to_float", arity: 1 },
    PrimEntry { receiver: "python", method: "to_str", arity: 1 },
    PrimEntry { receiver: "python", method: "to_bool", arity: 1 },
    // PyObj (Stage 4).
    PrimEntry { receiver: "pyobj", method: "attr", arity: 1 },
    PrimEntry { receiver: "pyobj", method: "call", arity: 1 },
    PrimEntry { receiver: "pyobj", method: "call_method", arity: 2 },
    PrimEntry { receiver: "pyobj", method: "index", arity: 1 },
    // Secret[_].
    PrimEntry { receiver: "secret", method: "map", arity: 1 },
    PrimEntry { receiver: "secret", method: "verify", arity: 1 },
    PrimEntry { receiver: "secret", method: "expose", arity: 1 },
    // Str (builtin).
    PrimEntry { receiver: "str", method: "len", arity: 0 },
    PrimEntry { receiver: "str", method: "trim", arity: 0 },
    PrimEntry { receiver: "str", method: "contains", arity: 1 },
    PrimEntry { receiver: "str", method: "starts_with", arity: 1 },
    PrimEntry { receiver: "str", method: "split", arity: 1 },
    PrimEntry { receiver: "str", method: "slice", arity: 2 },
    // List (builtin).
    PrimEntry { receiver: "list", method: "len", arity: 0 },
    PrimEntry { receiver: "list", method: "get", arity: 1 },
    PrimEntry { receiver: "list", method: "push", arity: 1 },
    PrimEntry { receiver: "list", method: "map", arity: 1 },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check_source;

    /// The table is non-empty and anchors are unique and well-formed (`ref.prim.*.*`).
    #[test]
    fn table_is_well_formed() {
        assert!(!PRIM_TABLE.is_empty(), "the primitive table index must not be empty");
        let mut seen = std::collections::HashSet::new();
        for e in PRIM_TABLE {
            assert!(!e.receiver.is_empty() && !e.method.is_empty(), "empty field");
            let a = e.anchor();
            assert!(a.starts_with("ref.prim."), "bad anchor {a}");
            assert!(seen.insert(a.clone()), "duplicate primitive anchor {a}");
        }
    }

    /// Count tripwire: any `method_sig` arm added or removed must be reflected here deliberately.
    #[test]
    fn count_is_pinned() {
        assert_eq!(
            PRIM_TABLE.len(),
            59,
            "the primitive table index changed — update this count and add/remove the matching \
             `method_sig` arm (Stage 9 coverage law)"
        );
    }

    /// A `main` header with a broad row so every capability op is in-row; the drift guard only
    /// gates on DL0405 (unknown method), so unrelated arg/type errors are irrelevant.
    const HEADER: &str =
        "module m\nfn main(root: Root) ! {Read, Write, Net, Clock, Rand, Declassify, ForeignCall} {\n";

    /// Wrap a receiver-binding + call body in a checkable program.
    fn program(body: &str) -> String {
        format!("{HEADER}{body}\n}}\n")
    }

    /// For each constructible receiver, the source that binds `recv` to a value of that type.
    /// `None` means the receiver is not cheaply constructible in a standalone snippet (plugin) and
    /// is covered by the structural + count tripwire only.
    fn receiver_binding(receiver: &str) -> Option<&'static str> {
        Some(match receiver {
            "root" => "",
            "console" => "let recv = root.console();",
            "fs_read" => "let recv = root.fs_read(\".\");",
            "fs_write" => "let recv = root.fs_write(\".\");",
            "http" => "let recv = root.http([\"h\"]);",
            "clock" => "let recv = root.clock();",
            "rand" => "let recv = root.rand();",
            "secret" => "let recv = root.secret(\"K\");",
            "str" => "let recv = \"x\";",
            "list" => "let recv = [1];",
            // Python / PyObj bindings live inside a match arm; handled specially below.
            "python" | "pyobj" | "plugin" => return None,
            _ => return None,
        })
    }

    fn dl0405_present(src: &str) -> bool {
        check_source(0, src).diagnostics.iter().any(|d| d.code == "DL0405")
    }

    /// A call `recv.<method>()` — DL0405 fires iff the method name is unknown, regardless of args.
    fn resolves(binding: &str, method: &str) -> bool {
        let recv = if binding.is_empty() { "root" } else { "recv" };
        let body = format!("{binding}\nlet _z = {recv}.{method}();");
        !dl0405_present(&program(&body))
    }

    /// Every listed primitive on a constructible receiver RESOLVES (no DL0405). Proves the index
    /// has no stale/renamed entry for those receivers.
    #[test]
    fn every_listed_primitive_resolves() {
        for e in PRIM_TABLE {
            let Some(binding) = receiver_binding(e.receiver) else { continue };
            assert!(
                resolves(binding, e.method),
                "primitive `{}` does not resolve in method_sig — the index drifted",
                e.anchor()
            );
        }
    }

    /// Python + PyObj receivers are constructed inside the `Ok` arm of `root.python(...)`.
    #[test]
    fn python_and_pyobj_primitives_resolve() {
        for e in PRIM_TABLE.iter().filter(|e| e.receiver == "python" || e.receiver == "pyobj") {
            let inner = if e.receiver == "python" {
                format!("let _z = py.{}();", e.method)
            } else {
                format!("let o = py.of_int(1);\nlet _z = o.{}();", e.method)
            };
            let body = format!(
                "match root.python(root.foreign_load()) {{\nOk(py) => {{\n{inner}\n}}\nErr(e) => {{}}\n}}"
            );
            assert!(
                !dl0405_present(&program(&body)),
                "primitive `{}` does not resolve in method_sig — the index drifted",
                e.anchor()
            );
        }
    }

    /// THE SKIP-BRANCH CASE (house rule 3): an *unknown* method on a real receiver must be DL0405
    /// — the checker fails closed, never silently accepting an unmodeled primitive. If this broke,
    /// the "every listed primitive resolves" guard above would be vacuously satisfiable.
    #[test]
    fn unknown_method_is_dl0405_on_every_constructible_receiver() {
        let receivers = ["root", "console", "fs_read", "fs_write", "http", "clock", "rand", "secret", "str", "list"];
        for r in receivers {
            let binding = receiver_binding(r).unwrap();
            assert!(
                !resolves(binding, "definitely_not_a_real_method_9a"),
                "receiver `{r}`: an unknown method must produce DL0405 (fail-closed skip-branch)"
            );
        }
    }

    /// Whether a call with a deliberately wrong argument list is rejected with an error.
    fn rejected_with_error(src: &str) -> bool {
        check_source(0, src).diagnostics.iter().any(|d| d.is_error())
    }

    /// The REJECTING side of every primitive-table entry (invariant 42): a call with five
    /// arguments — no primitive takes five — must be an error for every constructible receiver.
    /// This is the per-entry rejecting conformance witness for `ref.prim.*` anchors.
    #[test]
    fn every_listed_primitive_rejects_a_five_argument_call() {
        for e in PRIM_TABLE {
            let Some(binding) = receiver_binding(e.receiver) else { continue };
            let recv = if binding.is_empty() { "root" } else { "recv" };
            let body = format!("{binding}\nlet _z = {recv}.{}(1, 2, 3, 4, 5);", e.method);
            assert!(
                rejected_with_error(&program(&body)),
                "primitive `{}` accepted a five-argument call — the rejecting side is broken",
                e.anchor()
            );
        }
    }

    /// Whether the ARITY GATE's DL0403 (its exact message shape) fired — distinguishes the gate
    /// from `expect_arg`'s "missing argument" DL0403 and from type mismatches.
    fn gate_fired(src: &str, label: &str, method: &str) -> bool {
        let needle = format!("`{label}.{method}` expects");
        check_source(0, src)
            .diagnostics
            .iter()
            .any(|d| d.code == "DL0403" && d.message.contains(&needle))
    }

    /// ARITY DRIFT GUARD, both directions: for every constructible entry, a call with exactly
    /// `arity` arguments does NOT trip the gate, and a call with `arity + 1` DOES. A wrong arity
    /// column fails one of the two sides.
    #[test]
    fn arity_column_matches_the_checker_gate() {
        for e in PRIM_TABLE {
            let Some(binding) = receiver_binding(e.receiver) else { continue };
            let recv = if binding.is_empty() { "root" } else { "recv" };
            let exact_args = (0..e.arity).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
            let extra_args =
                (0..e.arity + 1).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
            let exact = format!("{binding}\nlet _z = {recv}.{}({exact_args});", e.method);
            let extra = format!("{binding}\nlet _z = {recv}.{}({extra_args});", e.method);
            assert!(
                !gate_fired(&program(&exact), e.receiver, e.method),
                "arity gate fired on an exact-arity call of `{}` — the arity column is wrong",
                e.anchor()
            );
            assert!(
                gate_fired(&program(&extra), e.receiver, e.method),
                "arity gate did NOT fire on an arity+1 call of `{}` — the arity column is wrong",
                e.anchor()
            );
        }
    }

    /// The same rejecting witness for the python/pyobj receivers (constructed in a match arm).
    #[test]
    fn python_and_pyobj_primitives_reject_a_five_argument_call() {
        for e in PRIM_TABLE.iter().filter(|e| e.receiver == "python" || e.receiver == "pyobj") {
            let inner = if e.receiver == "python" {
                format!("let _z = py.{}(1, 2, 3, 4, 5);", e.method)
            } else {
                format!("let o = py.of_int(1);\nlet _z = o.{}(1, 2, 3, 4, 5);", e.method)
            };
            let body = format!(
                "match root.python(root.foreign_load()) {{\nOk(py) => {{\n{inner}\n}}\nErr(e) => {{}}\n}}"
            );
            assert!(
                rejected_with_error(&program(&body)),
                "primitive `{}` accepted a five-argument call — the rejecting side is broken",
                e.anchor()
            );
        }
    }
}
