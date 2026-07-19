//! The normative rule index (Stage 9b, spec §2.1) — one entry per normative statement of
//! Constitution §5, each with a stable anchor id.
//!
//! ## Why the rules live in code rather than in prose
//! A hand-written reference chapter and the compiler drift apart within one release. Every rule
//! here names the diagnostic codes that **enforce** it, so the generated reference can state, per
//! rule, whether the enforcement is witnessed by the conformance suite — and a rule whose
//! enforcing code does not exist fails a drift guard rather than becoming a comfortable lie.
//!
//! ## How a rule is covered
//! A rule is *witnessed* exactly when **every** code that enforces it is witnessed in both
//! directions. That is deliberately strict: a rule enforced by three codes, one of which no test
//! ever produces, is a rule with an untested edge — the reference says so.
//!
//! Rules that hold "by construction" are **not** exempt. Where the violation cannot be written,
//! the attempt still produces a diagnostic — `throw e` is an unknown name, `async` is a reserved
//! word — and that diagnostic is the rule's enforcement. Exempting a rule because it feels
//! obviously true is the same fail-open shrug the skip-branch rule exists to prevent, so
//! `enforced_by` is never empty.

/// One normative statement of the language.
pub struct Rule {
    /// Stable anchor id, e.g. `ref.rule.effects.declared-row`.
    pub anchor: &'static str,
    /// The Constitution §5 subsection this rule belongs to.
    pub chapter: &'static str,
    /// The normative statement itself.
    pub statement: &'static str,
    /// The diagnostic codes that enforce it. Never empty — see the module docs.
    pub enforced_by: &'static [&'static str],
    /// What makes the rule hold, or the precision the statement alone would lose.
    pub note: &'static str,
}

/// The Constitution §5 subsections, in order — the reference's chapter list.
pub const CHAPTERS: &[(&str, &str)] = &[
    ("5.1", "Authority model — pure object-capability"),
    ("5.2", "Effect system — effect rows in every function type"),
    ("5.3", "Kind vs. scope — the honest granularity split"),
    ("5.4", "Secrets and information flow"),
    ("5.5", "Modules — units of authority"),
    ("5.6", "Packages — declared, versioned authority"),
    ("5.7", "Imports — access is not authority"),
    ("5.8", "Errors — results, not exceptions"),
    ("5.9", "Concurrency — actors with reference capabilities"),
    ("5.10", "Async — an effect, not a second system"),
    ("5.11", "Compilation — high-level surface, tiered backend"),
    ("5.12", "Interop — first-class Python and C, honestly fenced"),
    ("5.13", "Portability — one portable target"),
    ("5.14", "Security — defense in depth"),
    ("5.15", "Runtime guarantees"),
    ("5.16", "The authority holder model"),
];

pub const RULES: &[Rule] = &[
    // ---- 5.1 Authority model ----
    Rule {
        anchor: "ref.rule.authority.root-is-the-only-source",
        chapter: "5.1",
        statement: "All authority originates in the `Root` value passed to `main`. There is no \
                    ambient authority: no global, import, or constructor yields a capability.",
        enforced_by: &["DL0601", "DL0301"],
        note: "A capability type has no literal syntax and no constructor; the only way to obtain \
               one is to derive it from a `Root` you were handed.",
    },
    Rule {
        anchor: "ref.rule.authority.derivation-is-pure",
        chapter: "5.1",
        statement: "Deriving a capability from `Root` (`root.fs_read(path)`, `root.http(hosts)`, …) \
                    is a PURE operation carrying no effect. Holding authority is not using it.",
        enforced_by: &["DL0501"],
        note: "The effect appears when the capability is *used*, which is why a program that \
               derives but never calls has an empty row.",
    },
    Rule {
        anchor: "ref.rule.authority.attenuation-is-monotone",
        chapter: "5.1",
        statement: "A derived capability is never wider than the one it came from. `narrow` may \
                    only shrink a scope; no operation widens one.",
        enforced_by: &["DL0802"],
        note: "The broker enforces the same law across process boundaries (audit rule R-7).",
    },
    Rule {
        anchor: "ref.rule.authority.no-forgery",
        chapter: "5.1",
        statement: "A capability value cannot be constructed, forged, cast, or deserialized into \
                    existence.",
        enforced_by: &["DL0601", "DL0904"],
        note: "Statically there is no constructor; at the WASM boundary a forged handle is refused \
               by the host (DL0904).",
    },
    // ---- 5.2 Effect system ----
    Rule {
        anchor: "ref.rule.effects.declared-row",
        chapter: "5.2",
        statement: "A function may perform only the effects its row declares. Performing an \
                    undeclared effect is a compile error.",
        enforced_by: &["DL0501"],
        note: "This is the identity claim's load-bearing rule.",
    },
    Rule {
        anchor: "ref.rule.effects.rows-propagate-through-calls",
        chapter: "5.2",
        statement: "A caller's row must contain every callee's row. Effects compose upward through \
                    the call graph with no escape hatch.",
        enforced_by: &["DL0501"],
        note: "There is no `unsafe`, no dynamic dispatch that erases a row, and no reflection.",
    },
    Rule {
        anchor: "ref.rule.effects.declared-but-unperformed-warns",
        chapter: "5.2",
        statement: "Declaring an effect a function never performs is a warning, not an error: \
                    over-declaring is safe but dishonest about what the code does.",
        enforced_by: &["DL0502"],
        note: "A warning rather than an error, because a widened row is a *smaller* claim of \
               trustworthiness — it never lets a program do more than it says.",
    },
    Rule {
        anchor: "ref.rule.effects.one-row-variable",
        chapter: "5.2",
        statement: "A signature carries at most one row variable, and a row variable never \
                    union-merges two conflicting bindings.",
        enforced_by: &["DL0503", "DL0504"],
        note: "Audit rule R-3: a single subsumption site is what makes row inference decidable and \
               keeps a polymorphic row from silently absorbing an effect.",
    },
    Rule {
        anchor: "ref.rule.effects.builtin-callbacks-compose",
        chapter: "5.2",
        statement: "A higher-order builtin (`List.map`, …) has the row of the callback it is given: \
                    a pure builtin cannot launder an effectful function into a pure context.",
        enforced_by: &["DL0501"],
        note: "Audit rule R-4 — the classic laundering hole, closed by making builtins \
               row-polymorphic rather than pure.",
    },
    Rule {
        anchor: "ref.rule.effects.generic-var-is-type-or-row",
        chapter: "5.2",
        statement: "A generic variable is a type variable or a row variable, never both.",
        enforced_by: &["DL0410"],
        note: "",
    },
    // ---- 5.3 Kind vs scope ----
    Rule {
        anchor: "ref.rule.granularity.effect-kind-is-static",
        chapter: "5.3",
        statement: "The effect KIND (Read, Write, Net, Clock, Rand, Declassify, ForeignCall) is \
                    static and appears in the type. The SCOPE (which path, which host) is a runtime \
                    value carried by the capability.",
        enforced_by: &["DL0306", "DL0307"],
        note: "The honest split: the type system proves *what kind* of thing a function can do; \
               the capability's scope decides *which* resource. Claiming static path-level proof \
               would be a lie the design refuses to tell.",
    },
    Rule {
        anchor: "ref.rule.granularity.scope-checked-at-use",
        chapter: "5.3",
        statement: "A capability's scope is enforced when the operation runs; escaping it is a \
                    runtime refusal, not a type error.",
        enforced_by: &["DL0904", "DL0703"],
        note: "",
    },
    // ---- 5.4 Secrets ----
    Rule {
        anchor: "ref.rule.secrets.no-implicit-flow",
        chapter: "5.4",
        statement: "`Secret[T]` never coerces to `T`. A secret cannot flow into a position \
                    expecting a plain value.",
        enforced_by: &["DL0602"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.secrets.expose-requires-declassify",
        chapter: "5.4",
        statement: "`expose` is the only unwrap, it requires `Cap[Declassify]`, and it carries the \
                    `Declassify` effect — so declassification is visible in the authority report.",
        enforced_by: &["DL0501"],
        note: "Audit rule R-2: declassification is an effect, which is what stops a secret from \
               leaving silently.",
    },
    Rule {
        anchor: "ref.rule.secrets.map-must-be-pure",
        chapter: "5.4",
        statement: "`Secret.map` requires a pure function, and its result stays `Secret`.",
        enforced_by: &["DL0603"],
        note: "An effectful mapper could exfiltrate the plaintext without ever calling `expose`.",
    },
    Rule {
        anchor: "ref.rule.secrets.opaque-has-no-observers",
        chapter: "5.4",
        statement: "An opaque value has no stringification, no serialization, and no structural \
                    equality — only constant-time `verify`.",
        enforced_by: &["DL0604", "DL0605"],
        note: "Audit rule R-5. Structural equality would leak the contents a byte at a time.",
    },
    Rule {
        anchor: "ref.rule.secrets.never-cross-into-wasm",
        chapter: "5.4",
        statement: "Secret contents never enter the WASM guest.",
        enforced_by: &["DL1205"],
        note: "",
    },
    // ---- 5.5 Modules ----
    Rule {
        anchor: "ref.rule.modules.file-declares-its-module",
        chapter: "5.5",
        statement: "Every file begins with a `module` declaration.",
        enforced_by: &["DL0204"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.modules.no-mutable-module-state",
        chapter: "5.5",
        statement: "Module-level mutable state is forbidden: a module cannot hold a hidden channel \
                    between its functions.",
        enforced_by: &["DL0305"],
        note: "Shared mutable module state would be ambient authority in disguise.",
    },
    Rule {
        anchor: "ref.rule.modules.no-duplicate-definitions",
        chapter: "5.5",
        statement: "A name is defined at most once per module.",
        enforced_by: &["DL0302"],
        note: "",
    },
    // ---- 5.6 Packages ----
    Rule {
        anchor: "ref.rule.packages.manifest-bounds-the-code",
        chapter: "5.6",
        statement: "A package's code may not perform an effect its own manifest does not permit.",
        enforced_by: &["DL1009"],
        note: "The manifest is a ceiling the code is checked against, never a claim taken on trust.",
    },
    Rule {
        anchor: "ref.rule.packages.dependency-authority-is-pinned",
        chapter: "5.6",
        statement: "A dependency may not exceed the authority pinned for it in the lockfile; a \
                    changed authority under an unchanged version is refused.",
        enforced_by: &["DL1001", "DL1002", "DL1010"],
        note: "This is the xz scenario, refused mechanically: a patch release that adds an effect \
               anywhere in the graph fails the build before it runs.",
    },
    Rule {
        anchor: "ref.rule.packages.semver-authority",
        chapter: "5.6",
        statement: "Widening a package's authority requires a major version bump.",
        enforced_by: &["DL1003"],
        note: "Authority is part of the public interface, so it obeys semver like any other part.",
    },
    Rule {
        anchor: "ref.rule.packages.sources-are-pinned",
        chapter: "5.6",
        statement: "A git dependency must pin a rev or tag, one package name resolves to one \
                    source, and a locked build requires a lockfile.",
        enforced_by: &["DL1007", "DL1008", "DL1011"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.packages.no-cycles",
        chapter: "5.6",
        statement: "Package and re-export graphs are acyclic.",
        enforced_by: &["DL1005"],
        note: "",
    },
    // ---- 5.7 Imports ----
    Rule {
        anchor: "ref.rule.imports.access-is-not-authority",
        chapter: "5.7",
        statement: "Importing a module grants the ability to CALL its functions, never the \
                    authority those functions need. The caller must still hold and pass the \
                    capability.",
        enforced_by: &["DL0501"],
        note: "The rule that makes dependency review tractable: a malicious import cannot act on \
               its own, only ask.",
    },
    Rule {
        anchor: "ref.rule.imports.resolution-is-unambiguous",
        chapter: "5.7",
        statement: "An import resolves to exactly one module; local/dependency collisions and \
                    import cycles are refused.",
        enforced_by: &["DL0303", "DL0304", "DL1006"],
        note: "",
    },
    // ---- 5.8 Errors ----
    Rule {
        anchor: "ref.rule.errors.results-not-exceptions",
        chapter: "5.8",
        statement: "Failure is a `Result` value. There are no exceptions, no unwinding across \
                    frames, and no continuations.",
        enforced_by: &["DL0301"],
        note: "There is no throw/catch syntax and no continuation capture: `throw e` is simply an \
               unknown name. Runtime faults (DL09xx) terminate the program with a diagnostic rather \
               than transferring control, so no construct moves control non-locally.",
    },
    Rule {
        anchor: "ref.rule.errors.try-requires-result-context",
        chapter: "5.8",
        statement: "`?` is only valid on a `Result` inside a function that itself returns `Result`.",
        enforced_by: &["DL0409"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.errors.match-is-exhaustive",
        chapter: "5.8",
        statement: "A `match` must cover every case.",
        enforced_by: &["DL0407"],
        note: "Exhaustiveness is what makes handling a new error variant a compile error rather \
               than a silent fallthrough.",
    },
    // ---- 5.9 Concurrency ----
    Rule {
        anchor: "ref.rule.actors.only-sendable-values-cross",
        chapter: "5.9",
        statement: "Only sendable values cross an actor boundary. An unconsumed `iso`, an aliased \
                    mutable reference, or a pinned foreign object may not be sent.",
        enforced_by: &["DL1601", "DL1605"],
        note: "This is data-race freedom by typing rather than by lock discipline.",
    },
    Rule {
        anchor: "ref.rule.actors.consume-is-final",
        chapter: "5.9",
        statement: "After `consume`, the original binding is dead; using it is a compile error.",
        enforced_by: &["DL1602"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.actors.viewpoint-adaptation",
        chapter: "5.9",
        statement: "A reference capability's deny properties are enforced at every access: no write \
                    through `box`, no field read through `tag`, no synchronous call on `tag`.",
        enforced_by: &["DL1603", "DL1604", "DL1607"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.actors.behaviors-return-unit",
        chapter: "5.9",
        statement: "A behavior yields `Unit` at the send site; it cannot declare a return type.",
        enforced_by: &["DL1606"],
        note: "A behavior is asynchronous, so a return value would require a hidden await.",
    },
    // ---- 5.10 Async ----
    Rule {
        anchor: "ref.rule.async.is-an-effect-not-a-colour",
        chapter: "5.10",
        statement: "Asynchrony is expressed in the effect row, not by a parallel universe of \
                    `async` function types. There is no function-colouring split.",
        enforced_by: &["DL0106"],
        note: "`async` and `await` are reserved words with no grammar — declaring one is refused, \
               so a second calling convention cannot be introduced by user code. Concurrency is \
               reached through actors (§5.9), whose sends are already asynchronous.",
    },
    // ---- 5.11 Compilation ----
    Rule {
        anchor: "ref.rule.compilation.engines-agree",
        chapter: "5.11",
        statement: "The interpreter and the WASM backend produce the same observable behavior for \
                    any program both support; an unsupported construct falls back honestly rather \
                    than silently differing.",
        enforced_by: &["DL1201", "DL1206"],
        note: "Parity is checked differentially, and a disagreement is compiler-bug class.",
    },
    Rule {
        anchor: "ref.rule.compilation.artifacts-carry-their-authority",
        chapter: "5.11",
        statement: "A compiled artifact carries its authority section; a missing, tampered, or \
                    unsupported one is refused rather than run.",
        enforced_by: &["DL1202", "DL1204"],
        note: "",
    },
    // ---- 5.12 Interop ----
    Rule {
        anchor: "ref.rule.interop.foreign-needs-declared-authority",
        chapter: "5.12",
        statement: "A foreign library is usable only with a manifest entry and a runtime grant, and \
                    every foreign call carries the `ForeignCall` effect.",
        enforced_by: &["DL1303", "DL0501"],
        note: "Foreign code is outside the proof, so the authority report discloses it under a \
               separate heading rather than pretending it is proven.",
    },
    Rule {
        anchor: "ref.rule.interop.no-callbacks-across-the-boundary",
        chapter: "5.12",
        statement: "A function-typed value never crosses the foreign boundary: unverifiable code \
                    must not hold a re-entry point into verified code.",
        enforced_by: &["DL1302", "DL0803"],
        note: "Audit rule R-6a.",
    },
    Rule {
        anchor: "ref.rule.interop.marshalling-is-an-allowlist",
        chapter: "5.12",
        statement: "Only allowlisted types marshal across a foreign signature; secrets and opaque \
                    types never do, and a return value that fails shape validation is a fault, not \
                    a coerced value.",
        enforced_by: &["DL1301", "DL1306"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.interop.python-imports-are-allowlisted",
        chapter: "5.12",
        statement: "A Python import outside the granted allowlist is refused, and the denied \
                    attempt is recorded in the trace.",
        enforced_by: &["DL1305"],
        note: "Refusals are traced, so an attempt to reach beyond the grant is visible evidence \
               rather than a silent no-op.",
    },
    // ---- 5.13 Portability ----
    Rule {
        anchor: "ref.rule.portability.one-portable-target",
        chapter: "5.13",
        statement: "WASM is the portable target. A construct the backend cannot express is \
                    reported, and the program runs on the interpreter — never silently degraded.",
        enforced_by: &["DL1201"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.portability.isolation-labels-are-honest",
        chapter: "5.13",
        statement: "An isolation profile unavailable on the host is refused or labelled as a weaker \
                    fallback; the label always states what was actually used.",
        enforced_by: &["DL1408"],
        note: "Naming a profile you did not get is the failure mode this rule exists to prevent.",
    },
    // ---- 5.14 Security ----
    Rule {
        anchor: "ref.rule.security.fail-closed",
        chapter: "5.14",
        statement: "When custody cannot be established — broker unreachable, protocol mismatch, \
                    token invalid — the answer is refusal, never a default-allow.",
        enforced_by: &["DL1401", "DL1406", "DL1407"],
        note: "The skip branch is where security rules die; every 'cannot tell' path refuses.",
    },
    Rule {
        anchor: "ref.rule.security.revocation-is-immediate-and-final",
        chapter: "5.14",
        statement: "A revoked or expired grant fails at the next use and carries the revoking audit \
                    sequence; a reloaded plugin gets a fresh node, never the old one.",
        enforced_by: &["DL0801", "DL1402", "DL1403"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.security.the-human-holds-the-keys",
        chapter: "5.14",
        statement: "Guarded authority requires a human decision. A pending request blocks, a denial \
                    carries the principal's words verbatim, and a sealed refusal cannot be lifted \
                    by bypass.",
        enforced_by: &["DL1410", "DL1411", "DL1412", "DL1413", "DL1414"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.security.audit-chain-is-verifiable",
        chapter: "5.14",
        statement: "The audit log is a hash chain; a corrupted or reordered record fails \
                    verification.",
        enforced_by: &["DL1405"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.security.signatures-verify-locally",
        chapter: "5.14",
        statement: "Signature verification happens on the client. A present-but-invalid signature \
                    is a different, louder failure than an absent one.",
        enforced_by: &["DL1510", "DL1511", "DL1705"],
        note: "Distinguishing 'tampered' from 'unsigned' matters: they call for different responses.",
    },
    // ---- 5.15 Runtime guarantees ----
    Rule {
        anchor: "ref.rule.runtime.faults-are-diagnostics",
        chapter: "5.15",
        statement: "A runtime fault — overflow, division by zero, index out of bounds, recursion \
                    depth — is a diagnostic with a code, never a host crash or silent wrap.",
        enforced_by: &["DL0901", "DL0902", "DL0903", "DL0905"],
        note: "",
    },
    Rule {
        anchor: "ref.rule.runtime.trace-is-within-the-row",
        chapter: "5.15",
        statement: "Every effect performed at runtime lies within the statically declared row. A \
                    violation is compiler-bug class.",
        enforced_by: &["DL1101"],
        note: "This is the static claim checked against reality: `--assert-trace` makes the \
               soundness argument falsifiable at runtime rather than only on paper.",
    },
    Rule {
        anchor: "ref.rule.runtime.plugin-limits-terminate-and-revoke",
        chapter: "5.15",
        statement: "A plugin that exceeds its resource limits is terminated and its grant node \
                    revoked — dead, not wounded.",
        enforced_by: &["DL1506"],
        note: "",
    },
    // ---- 5.16 Holder model ----
    Rule {
        anchor: "ref.rule.holder.grant-cannot-exceed-the-holder",
        chapter: "5.16",
        statement: "No grantee receives more authority than its grantor holds, at any depth of \
                    delegation.",
        enforced_by: &["DL0802", "DL1502"],
        note: "Audit rule R-7. Composition is where attenuation systems usually leak, so the bound \
               is checked at every level rather than only at the first.",
    },
    Rule {
        anchor: "ref.rule.holder.tests-hold-no-ambient-authority",
        chapter: "5.16",
        statement: "A test holds only the authority its declared row requires, bounded by the \
                    package's test ceiling; a pure test holds none.",
        enforced_by: &["DL1703"],
        note: "Invariant 41 — the test runner is a holder like any other, not a privileged context.",
    },
    Rule {
        anchor: "ref.rule.holder.plugin-manifest-never-overrides-code",
        chapter: "5.16",
        statement: "A plugin's manifest never widens what its code actually does; a mismatch is \
                    refused at build, and a failed re-check never falls back to a weaker class.",
        enforced_by: &["DL1501", "DL1504", "DL1509"],
        note: "Falling back to Contained on a failed Verified re-check would turn a broken proof \
               into a silent downgrade.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn rules_are_well_formed_and_unique() {
        let mut seen = BTreeSet::new();
        for r in RULES {
            assert!(r.anchor.starts_with("ref.rule."), "bad anchor {}", r.anchor);
            assert!(seen.insert(r.anchor), "duplicate rule anchor {}", r.anchor);
            assert!(!r.statement.is_empty(), "{} has no statement", r.anchor);
            assert!(
                !r.enforced_by.is_empty(),
                "{} names no enforcement — a rule nothing checks is a wish, not a rule",
                r.anchor
            );
            assert!(
                CHAPTERS.iter().any(|(n, _)| *n == r.chapter),
                "{} names unknown chapter {}",
                r.anchor,
                r.chapter
            );
        }
    }

    /// DRIFT GUARD: every code a rule claims as enforcement is a REGISTERED diagnostic. A rule
    /// citing a code that does not exist is a reference that lies about its own enforcement.
    #[test]
    fn every_enforcing_code_is_registered() {
        let registered: BTreeSet<&str> = delulu_diag::REGISTRY.iter().map(|c| c.code).collect();
        for r in RULES {
            for code in r.enforced_by {
                assert!(
                    registered.contains(code),
                    "rule `{}` claims enforcement by `{code}`, which is not in the diagnostic registry",
                    r.anchor
                );
            }
        }
    }

    /// THE SKIP-BRANCH CASE (house rule 3): the guard above must be able to fail.
    #[test]
    fn an_unregistered_code_would_be_caught() {
        let registered: BTreeSet<&str> = delulu_diag::REGISTRY.iter().map(|c| c.code).collect();
        assert!(!registered.is_empty(), "the registry must be readable for the guard to mean anything");
        assert!(!registered.contains("DL9999"), "the guard's membership test is too loose to fail");
    }

    /// Every Constitution §5 subsection has at least one normative rule — a chapter with no rules
    /// is a hole in the reference, not an empty section.
    #[test]
    fn every_chapter_has_at_least_one_rule() {
        for (num, title) in CHAPTERS {
            assert!(
                RULES.iter().any(|r| r.chapter == *num),
                "chapter {num} ({title}) has no normative rule"
            );
        }
    }
}
