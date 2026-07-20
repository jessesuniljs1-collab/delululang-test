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
    // DL0503 was retired pre-1.0 (Stage 9 release, ruling D22): it described an arity rule
    // ("at most one row variable per signature") the checker does not have — a benign
    // multi-row-variable signature is legal, and row honesty is enforced independently
    // (DL0501/DL0306, laundering probe on record). Codes are add-only from 1.0; a code that
    // cannot fire must not freeze. The number is never reused.
    "DL0504" => "conflicting bindings for row variable (rows never union-merge)",

    // DL06xx — capabilities & secrets
    "DL0601" => "capability type cannot be constructed or forged",
    "DL0602" => "secret value cannot flow here (Secret[T] is not T)",
    "DL0603" => "Secret.map requires a pure function",
    "DL0604" => "opaque type cannot be stringified or serialized",
    "DL0605" => "opaque type has no structural equality",

    // DL07xx — manifest / authority
    "DL0701" => "main's effect row exceeds the authority manifest",
    // DL0702 was retired pre-1.0 (ruling D22): "grant refused at startup" is a flow the CLI
    // never had — deny-by-default happens at derivation (DL0703), on both engines, which is
    // strictly better (a program is never refused for a capability it never exercises).
    // The number is never reused.
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
    // The Guard (Stage 5 chunk 6, phases 5k–5m): a dcg-inspired principal-approval layer in the
    // custody broker. Contiguous from DL1410 (the no-DL1404 gap above is untouched).
    "DL1410" => "guard refusal: guarded authority, no permit (names the exact guard request command)",
    "DL1411" => "guard request pending (carries the request id)",
    "DL1412" => "guard request denied (carries the principal's comment verbatim)",
    "DL1413" => "guard sealed refusal — not runtime-approvable; bypass does not lift it",
    "DL1414" => "guard owner code missing or invalid — admin verb refused",

    // DL15xx — runtime plugins (Stage 6 "Live", spec §7). DL0801/DL0803 (reserved in Stage 1)
    // activate alongside these. DL1508 is a build-order-recorded addition (deviation 2): the spec
    // table has no container-corruption code, but a corrupt `.dpx` needs one, exactly as Stage 3
    // allocated DL1202 for the `.dwx`.
    "DL1501" => "manifest export signature does not match the plugin code (the manifest never overrides the code)",
    "DL1502" => "requested grant exceeds the plugin's declared ceiling",
    "DL1503" => "DIR version unsupported (rebuild the plugin)",
    "DL1504" => "Verified re-check failed (plugin code unsound or stale) — never falls back to Contained",
    "DL1505" => "Contained module imports outside its grant slice",
    "DL1506" => "plugin resource limit exceeded (plugin terminated and its grant node revoked)",
    "DL1507" => "plugin API version mismatch (rebuild the plugin)",
    "DL1508" => "malformed or tampered `.dpx` container (not a valid plugin artifact)",
    // The fail-closed side of R-6a (deviation 5). DL0803 refuses a function-typed argument that is
    // CONCRETELY present; this refuses the case where the signature is underdetermined, so R-6a
    // cannot be decided at all. A security rule must never be skippable because inference was
    // underdetermined — the two codes have different remedies, so they are different codes.
    "DL1509" => "a Contained plugin export's signature must be concrete at the `get` site (R-6a is otherwise undecidable)",
    // ed25519 plugin signatures (phase 6h, build-order deviation 8). A present-but-invalid signature
    // and an unsigned-but-required plugin are DIFFERENT faults with different remedies, so they get
    // different codes: DL1510 says "the signature does not verify"; DL1511 says "this grant requires
    // a signature and none is present". Reusing one code would make a message a lie (kitchen rule).
    "DL1510" => "plugin signature present but invalid (tampered content, wrong key, or malformed signature)",
    "DL1511" => "plugin is unsigned but the grant requires a signature (require_signed)",

    // DL16xx — actors and reference capabilities (Stage 7 "Concurrent", spec §10).
    // NOTE: there is deliberately NO DL1609 — the spec's table skips it (house rule mirroring
    // DL1404/DL1784). Do not invent one.
    "DL1601" => "non-sendable value crossing an actor boundary (incl. unconsumed iso; incl. PyObj pinning)",
    "DL1602" => "use of a binding after `consume`",
    "DL1603" => "alias violates a reference-capability deny property (incl. a `val` closure over a `ref` capture)",
    "DL1604" => "access denied by viewpoint/receiver capability (write via box; field via tag; sync call on tag)",
    "DL1605" => "recover block references a non-sendable outer binding",
    "DL1606" => "a behavior declares a return type (behaviors yield Unit at the send site)",
    "DL1607" => "reference capability invalid for this type (e.g. non-tag on an actor type)",
    "DL1608" => "identifier collides with a v0.7 keyword (`consume`/`recover`)",
    "DL1610" => "debug race-checker violation (compiler-bug class — file a bug)",

    // DL17xx — Surface (Stage 8). Spec §10 defines DL1701–DL1706 (tooling); DL1707 is the
    // assertion-failure panic (build-order deviation 5 — a Stage-8 construct faults under a
    // Stage-8 code; the Stage-1 09xx family stays frozen). The Atlas (DL1780/DL1781) and the
    // Palette (DL1790) — Surface addendum §3.2. DL1784 is deliberately never allocated
    // (house rule mirroring DL1404).
    "DL1701" => "LSP/workspace configuration error",
    "DL1702" => "formatter identity/idempotence violation (compiler-bug class — file a bug)",
    "DL1703" => "test authority exceeds the package test ceiling",
    "DL1704" => "catalog invalid: unknown key/placeholder, bad meta, or a welcome-override attempt — entry falls back to en-US",
    "DL1705" => "signature verification failed",
    "DL1706" => "registry index line invalid / semver-authority conflict at publish",
    "DL1707" => "assertion failed (a `test` assertion did not hold at runtime)",
    "DL1780" => "atlas refused: the program has check errors — fix them first (no partial graph)",
    "DL1781" => "custody overlay unavailable — the broker daemon is not reachable; atlas emitted without it",
    "DL1790" => "invalid theme name or malformed theme.toml — using the `default` theme",

    // DL18xx — Delulu (Stage 9). Stability and deprecation ONLY: Stage 9 adds no language
    // features by definition, so this range can never grow a semantic code.
    "DL1801" => "use of a deprecated feature (RFC-linked)",
    "DL1802" => "package declares a newer language edition than this toolchain",

    // DL19xx — Industrial (Stage 10). Budget fixed in the spec §10: DL1901–DL1911 only;
    // anything further needs a build-order ruling.
    "DL1901" => "unknown attribute",
    "DL1902" => "mailbox overflow dropped a message under `drop-new` in abort mode",
    "DL1903" => "dependency version has a published security advisory",
    "DL1904" => "actuator command refused by its envelope",
    "DL1905" => "hardware actuation requested for an artifact that no simulation approved",
    "DL1906" => "native-code emission requested without the `exec.native` grant",
    "DL1907" => "compute dispatch refused by the device envelope",
    "DL1908" => "signature policy requires hybrid; artifact is classical-only or names an unknown algorithm",
    "DL1909" => "deploy plan authority exceeds environment profile",
    "DL1910" => "unvalidated (pre-KAT, unaudited) cryptography invoked without `--unstable`",
    "DL1911" => "compute adapter cannot attest independent below-adapter envelope enforcement",
    // Kernel artifacts (10h). Unsigned and invalid are DIFFERENT faults with different remedies,
    // exactly as DL1510/DL1511 are for plugins: "sign this" is not the same instruction as "these
    // bytes are not what they claim". Reusing one code would make one of the messages a lie.
    "DL1912" => "kernel artifact is malformed, unreadable, or its signature does not verify",
    "DL1913" => "kernel artifact is unsigned (kernels are always signed — spec §7.1)",

    // DL09xx — runtime
    "DL0901" => "integer overflow",
    "DL0902" => "division by zero",
    "DL0903" => "index out of bounds",
    "DL0904" => "capability scope violation",
    "DL0905" => "recursion depth exceeded",
    // DL0906 ("explicit panic") was retired pre-1.0 (ruling D22): no `panic` builtin exists in
    // the language (§5.8 — divergence is abort-class, results are values), so the code could
    // never fire. Freezing a code for a feature that does not exist strands agents keying off
    // it. The number is never reused; if a panic construct ever lands by RFC, it gets a new code.
    "DL0907" => "match reached no arm (checker bug if ever seen)",
}

pub fn is_registered(code: &str) -> bool {
    REGISTRY.iter().any(|c| c.code == code)
}

/// The minimum length that counts as a real explanation rather than a restated title. Chosen so
/// that a one-sentence paraphrase does not pass: `delulu explain` exists to say what happened, why
/// the rule is there, and what to do — three things do not fit in a sentence.
pub const MIN_EXPLAIN_BODY: usize = 160;

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

/// The Stage-5 guard policy-change latency bound (addendum ruling 4 / criterion 11). It reuses the
/// §4.2 revocation sentence PATTERN **verbatim**: everything after the leading subject is
/// byte-identical to [`REVOCATION_BOUND`] (the `guard_policy_bound_reuses_the_revocation_pattern`
/// test enforces it). Emitted at every guard policy edit, exactly as `REVOCATION_BOUND` rides in
/// every revocation.
pub const GUARD_POLICY_BOUND: &str = "guard policy changes take effect: synchronous class — before \
     the next use; epoch class — within one epoch interval (≤ 50 ms default)";

/// The Guard honesty caveat (addendum §2.8, spec §10 tradition), carried VERBATIM by every guard
/// explain body and the `E-GUARD` topic. `warn` tier and bypass are awareness, not enforcement.
pub const GUARD_CAVEAT: &str =
    "Honesty (addendum §2.8, spec §10): The guard supervises what DeluluLang programs holding \
     DELEGATED grants can do. It does not defend against a malicious same-OS-user process that never \
     speaks Delulu — the same honesty as DL1401's \"not against the OS user\". The owner code raises \
     the bar only while the principal keeps it out of agent-visible context. The `warn` tier and \
     bypass mode are awareness mechanisms, not enforcement. This is never claimed otherwise.";

/// The exact banner printed whenever guard bypass is ENABLED (`--dangerously-bypass-guard` or
/// `delulu guard bypass on`), echoing Claude Code's `--dangerously-skip-permissions` shape
/// (addendum §2.6, spec-fixed text).
pub const GUARD_BYPASS_BANNER: &str =
    "!! GUARD BYPASSED — --dangerously-bypass-guard !!\n\
     The guard is the approval checkpoint between delegated agents and the classes where a mistake \
     is catastrophic or the type system's guarantees end: declassification of secrets, and native \
     (foreign) code. Bypassing it means any agent holding any lease uses those classes WITHOUT YOUR \
     KNOWLEDGE until you read the audit log. Sealed rules still hold; every bypassed use is still \
     audited. Turn the guard back on with `delulu guard bypass off`.";

/// A named explanation topic (not a diagnostic code): `delulu explain E-REVOKE` / `E-GUARD`. Returns
/// `(title, body)`. Topics carry normative honesty text the spec mandates verbatim (Stage 5
/// playbook 5j); unlike codes they explain a *semantics*, not a single failure.
pub fn topic_explain(topic: &str) -> Option<(&'static str, String)> {
    match topic {
        "ACTOR" => Some((
            "actors and reference capabilities: compile-time data-race freedom (Stage 7)",
            String::from(
                "DeluluLang's concurrency is the actor model with Pony-style reference \
                 capabilities. Every value has an rcap — iso (unique), trn (sole writer), ref \
                 (shared mutable within one actor), val (deeply immutable), box (read-only \
                 view), tag (opaque identity) — checked as a SECOND axis beside effect rows: a \
                 `val` closure can still be `!{Write}`; both axes are checked at send sites.\n\n\
                 Everything crossing an actor boundary must be sendable: `iso` (consumed — \
                 uniqueness proven; DL1601 with an exact `consume` repair otherwise), `val`, or \
                 `tag`. Behaviors are atomic turns; `fn` methods run only in the actor's own \
                 turn (a sync call on an actor reference is DL1604 — outsiders hold tag, and \
                 messages are the only cross-actor interface). Asynchrony is the `Async` \
                 EFFECT in the same rows — no futures runtime, no await, no colored functions; \
                 `Promise[T]` is a library actor. Every send site's row contains the target \
                 behavior's row (`{Async} ∪ row(beh)`), so whole-program authority remains \
                 row(main) across the actor boundary — `delulu why <Effect>` walks the chain, \
                 and `--assert-trace` replays it on the runtime witness with causal \
                 actor/turn/cause attribution.\n\n\
                 Honesty and threat-model caveats (spec §11, carried verbatim):\n\
                 - Compile-time data-race freedom covers DeluluLang code; foreign code and \
                 Contained plugins are bounded by their Stage-4/5/6 layers, not by rcaps.\n\
                 - Deadlock, livelock, starvation, and mailbox exhaustion are not prevented — \
                 the guarantee is race freedom, not liveness. Unbounded mailboxes can exhaust \
                 memory; backpressure is post-1.0.\n\
                 - The rcap tables are adopted from Pony's proven design; our own property-test \
                 validation (criterion 8) is a ship-gate, and mechanized proof remains Delulu \
                 Core future work.\n\
                 - WASM-engine concurrency is cooperative single-threaded in v0.7 — semantics \
                 identical, parallelism absent, labeled in output.\n\
                 - Message ordering is per-sender-pair FIFO only; no global order, no \
                 delivery-time bounds.\n\n\
                 The native scheduler pins each actor to one worker at spawn (build-order \
                 deviation 3 — worker-owned actors, zero unsafe); an iso send MOVES by rebuild \
                 (deviation 7 — observationally the spec's pointer handoff, verified unaliased \
                 under `--debug-rcaps`, DL1610 on a violated proof). `--debug-rcaps` is a \
                 compiler-bug detector, never the guarantee: race freedom is static.",
            ),
        )),
        "GUARD" => Some((
            "the Guard: a principal-approval layer over delegated custody (Stage 5 addendum)",
            format!(
                "The Guard supervises what DELEGATED (non-root) grants may do with the classes where \
                 the type system's guarantees end or a mistake is catastrophic — declassification of \
                 secrets, and native (foreign) code. Enforcement keys on TREE POSITION, never holder \
                 identity: root nodes are principal-held by construction and pass without guard \
                 interaction; delegated nodes are agent-held and are gated.\n\n\
                 A policy is a set of rules `class:pattern -> tier`:\n\
                 - warn    — the use proceeds; the agent is warned and a `guard_warn` event is audited.\n\
                 - guarded — refused (DL1410) unless a matching PERMIT exists. An agent escalates with \
                 `delulu guard request <node> --use <class:pattern> --why \"...\"`; the principal \
                 approves (`delulu guard approve <id>`) or denies (`deny <id> --comment \"...\"`, \
                 carried back to the agent as DL1412). While a request is pending a retried use is \
                 DL1411.\n\
                 - sealed  — refused (DL1413) ALWAYS; not runtime-approvable and bypass does not lift \
                 it. Only a principal policy edit unseals it.\n\n\
                 Permits are broker-held, never bearer tokens; daemon-memory only (a restart clears \
                 them). Guard ADMIN verbs (approve/deny/policy/bypass) need the owner code the daemon \
                 prints once at `broker start`. Bypass (`--dangerously-bypass-guard` / `guard bypass \
                 on`) lifts every guarded rule to audited-and-warned; it never touches sealed rules \
                 and never touches auditing. {GUARD_POLICY_BOUND}.\n\n\
                 {GUARD_CAVEAT}"
            ),
        )),
        "ATLAS" => Some((
            "the Atlas: a typed, deterministic code + authority graph (Surface addendum §2)",
            String::from(
                "`delulu atlas` builds a typed graph of a CHECKED program from compiler facts only — \
                 resolved names, typed effect rows, checked authority. There are no confidence tags: \
                 an edge either is a checked fact (and appears) or it is not (and does not).\n\n\
                 Node kinds: package, module, function, type, effect, resource (an fs path class, a \
                 net host, a secret name), foreign (a C symbol or Python module). Edge kinds: \
                 contains, depends_on, imports, calls, uses_type, performs (function→effect, from \
                 the checked row), requires (function/package→resource, from the authority report), \
                 foreign, declassifies, and delegates (the custody overlay only).\n\n\
                 Formats per consumer: `--format tree` (the default terminal overview), `--format \
                 digest` (token-budgeted Markdown for an LLM, self-describing, ending in a \
                 \"Querying further\" footer that teaches the query verbs), `--format json` (the \
                 versioned `atlas/1` machine channel — additive-only, NEVER colored), and \
                 `--format dot`/`mermaid`/`html`. Query verbs answer small questions without loading \
                 the whole graph: `delulu atlas node <name>`, `callers <fn>`, `calls <fn>`, `path \
                 <A> <B>` (typed hops), and `why <Effect|resource>` (a graph-shaped sibling of \
                 `delulu why`). `--budget <N>` caps any textual answer at ~N tokens (a chars/4 \
                 heuristic); truncation is always explicit.\n\n\
                 Refusals are honest: if `delulu check` reports errors the atlas refuses with DL1780 \
                 and builds no partial graph; if `--custody` is asked for but the broker daemon is \
                 unreachable the overlay degrades to a DL1781 note and the atlas is still emitted \
                 without it.\n\n\
                 Honesty: the atlas is a static map of checked facts, not a runtime trace; calls \
                 through function values may be under-approximated. The custody overlay reflects \
                 broker state at the moment of the query and is awareness, not enforcement (the \
                 Guard enforces). No claim of a \"complete call graph\" or \"always up to date\" is \
                 ever made.",
            ),
        )),
        "PALETTE" => Some((
            "the Palette: role-based color for human CLI output (Surface addendum §2.5)",
            String::from(
                "The Palette gives every human-facing surface one color vocabulary. Renderers name a \
                 semantic ROLE — error, warning, note, code, span_primary, span_secondary, effect, \
                 authority, guard_banner, success, path, repair, heading — and the active THEME maps \
                 the role to a color. Renderers never hardcode a color, so themes and accessibility \
                 stay in one place.\n\n\
                 Whether color is emitted (first match wins): the `--color never|always|auto` flag, \
                 then `DELULU_COLOR`, then `NO_COLOR` (any value ⇒ off; https://no-color.org), then \
                 auto (on iff the stream is a TTY). `NO_COLOR` beats `DELULU_COLOR=always`, but the \
                 explicit `--color always` flag beats `NO_COLOR` — a per-invocation flag is the user \
                 speaking now. Piped (non-TTY) output is colorless by default.\n\n\
                 Themes: `default` (colorblind-safe — never distinguishes by red/green alone; the \
                 severity WORD is the primary signal), `bright` (higher contrast), and `mono` \
                 (bold/underline only, zero color SGR). Select with `--theme <name>`, then \
                 `DELULU_THEME`, then a theme file (`$DELULU_THEME_FILE` if set, else \
                 `~/.delulu/theme.toml`) whose `theme = \"name\"` key plus an optional `[roles]` \
                 table overrides individual roles with named 16-color values. A bad theme name, an \
                 oversized/unreadable/non-UTF-8 file, or a malformed one is DL1790 (a warning) and \
                 falls back to `default` — never a hard failure.\n\n\
                 Machine channels are NEVER colored: `--json` output and the `atlas/1` graph carry \
                 zero SGR bytes regardless of any of the above, so an agent parsing them never has to \
                 strip escapes. Full syntax highlighting of source snippets is deferred to Stage 8 \
                 proper."
            ),
        )),
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
        "PLUGIN" => Some((
            "runtime plugins: Verified and Contained, the load sequence, and the honesty caveats (Stage 6 \"Live\")",
            String::from(
                "A plugin is code that arrives AFTER compile time and still cannot exceed its grant. \
                 Two classes ship, and the class is DECLARED, never inferred (a `.dpx` claiming \
                 Verified whose DIR fails any check is refused — DL1504 — and NEVER falls back to \
                 Contained):\n\
                 - Plugin[Verified] ships DIR (the Delulu typed IR) and is RE-CHECKED at load: the \
                 loader replays the compiler's own resolve + check over the shipped module, so a \
                 Verified export carries its re-verified per-function row. Verified plugins run on the \
                 host's own engine.\n\
                 - Plugin[Contained] ships opaque WASM and is confined at the MODULE boundary: its \
                 imports must fit the grant-derived slice (DL1505), and — rule R-1 — every export \
                 types at `effects(grant)`, whatever its manifest claims.\n\n\
                 The load sequence is an ORDER (spec §3.1): (1) container + `plugin.api` (DL1507); (2) \
                 class check; (3) manifest ceiling `grant ⊑ plugin.authority` (DL1502, intersection = \
                 exact narrowing repair); (4) holder check `grant ⊑ holder` at the broker (DL0802) → a \
                 child grant node; (5) class-specific verification; (6) signature policy; (7) \
                 instantiate. `delulu plugin verify` runs steps 1, 2, 5 without instantiating and gives \
                 identical verdicts to a real load.\n\n\
                 NOTATION (build-order deviation 4): the spec writes `load[C](…)` and `p.get[F](…)`, \
                 but the grammar has no turbofish (Stage-1 §6). These are inference-from-context: \
                 `let p: Plugin[Contained] = load(host, path, grant)?` and \
                 `let f: fn(Str) -> Str ! {} = p.get(\"shout\")?`. Do NOT copy the bracket form out of \
                 the spec — it will not parse. R-6a still fires at the `get` site: a function-typed \
                 parameter in a Contained `F` is DL0803, and an underdetermined `F` is DL1509.\n\n\
                 SINGLE-MODULE (build-order deviation 3): a plugin package is exactly one module in \
                 v0.6. A multi-module package is refused cleanly at build with DL1004 (\"plugin \
                 packages are single-module in v0.6\") — no partial artifact, no fake repair. \
                 Multi-module packages are on the post-v0.6 RFC ledger.\n\n\
                 DIAGNOSTICS (spec §7 + build-order deviations 2 and 8): DL1501 manifest/export string \
                 disagrees with the code; DL1502 grant exceeds the ceiling; DL1503 DIR version \
                 unsupported; DL1504 Verified re-check failed (never a Contained fallback); DL1505 \
                 Contained imports outside the grant slice; DL1506 resource limit exceeded (terminated \
                 + node revoked); DL1507 API mismatch; DL1508 a malformed or tampered `.dpx` container \
                 (deviation 2 — the container-corruption code the spec table lacked, mirroring Stage \
                 3's DL1202); DL1509 an underdetermined Contained `get` signature (R-6a undecidable); \
                 DL1510 a present-but-invalid signature; DL1511 an unsigned plugin under a \
                 `require_signed` grant. DL1510 and DL1511 are DIFFERENT faults with different \
                 remedies (deviation 8): a badly-signed artifact is not the same as an unsigned one.\n\n\
                 WINDOWS (build-order deviation 7): in-process CPU/wall enforcement for a Contained (or \
                 Verified-on-WASM) plugin uses host-initiated wasm traps, whose unwind fastfails the \
                 host process on Windows with wasmtime 27. So on Windows, contained execution is \
                 REFUSED up front with an honest `EnforcementUnsupported` — before any store exists — \
                 rather than risk an uncatchable host crash: not a limit hit (never DL1506, never an \
                 authority-widening repair), not a plugin fault. The enforcement-grade platform for \
                 hostile Contained code is the WASM engine on Linux (spec §5.4); Verified plugins run \
                 on all platforms via the interpreter. Verified platforms are Windows (with the caveat \
                 above) and Linux; macOS compiles the full enforcement path by construction (the gate \
                 is `cfg(not(windows))`, and wasmtime treats macOS as a first-class Unix signal-path \
                 platform) and is EXPECTED to work but is UNVERIFIED in this kitchen — no witness has \
                 run on a Mac, and nothing is claimed passing where unrun. Out-of-process enforcement \
                 is RFC-deferred.\n\n\
                 HONESTY AND THREAT-MODEL CAVEATS (spec §10, verbatim):\n\
                 - The Verified/Contained split is a TRUST STATEMENT, NOT A QUALITY RANKING: Verified = \
                 re-proved per-function at load; Contained = confined at module boundary. The type \
                 system keeps them honest by construction (R-1) — a Contained plugin's \"read-only\" \
                 export TYPES AS everything its module was granted.\n\
                 - Signatures authenticate ORIGIN, not behavior; a signed plugin is not a safe plugin.\n\
                 - Resource limits bound CPU/memory/wall-clock, NOT I/O volume within granted scopes \
                 (an I/O-quota grant dimension is a post-1.0 RFC).\n\
                 - Interpreter-engine limits are best-effort (§5.4); hostile code belongs on the WASM \
                 engine.\n\
                 - Plugins share the host's microVM in v1.0; per-plugin VMs are future work.\n\n\
                 Stage 6 is the promise kept: code that arrives at runtime and still cannot exceed its \
                 grant.",
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
        // ===== DL01xx — lexing =============================================
        "DL0101" => "The lexer met a character that cannot begin any token. Usually a stray \
             non-ASCII character pasted from a document, a smart quote where a straight one was \
             meant, or a control character. Delete it, or replace it with the ASCII it stands for.",
        "DL0102" => "A string literal opened and the file ended before it closed. Add the closing \
             quote. If the string was meant to span lines, note that a string literal does not: \
             break it into pieces or use an escape.",
        "DL0103" => "A backslash in a string was followed by a character that is not an escape. The \
             recognized escapes are `\\n`, `\\t`, `\\r`, `\\0`, `\\\\`, `\\\"`. To write a literal \
             backslash, double it.",
        "DL0104" => "A numeric literal is not a valid number: an integer too large for `Int`, a \
             float with a malformed exponent, or digits in a base that does not have them. Integer \
             literals must fit in a signed 64-bit `Int` — there is no automatic promotion, because \
             a silent widening is a silent change of meaning.",
        "DL0105" => "A block comment opened with `/*` and the file ended before `*/`. Block \
             comments NEST, so an inner `/*` needs its own close — count them from the top of the \
             comment. Nesting is deliberate: it lets you comment out a region that already contains \
             comments, which is the one time you actually need to.",
        "DL0106" => "This name is RESERVED for a future stage and cannot be declared. The word is \
             still legal as a member name after `.`, which is how `root.secret(…)` coexists with \
             `secret` being reserved. Reserving a word early is a promise that adding it later will \
             not break your code — the alternative is a breaking change dressed as a feature.",

        // ===== DL02xx — parsing ============================================
        "DL0201" => "The parser expected one specific token and found another. The message names \
             both. This is the general parse error; where a more specific one exists (a type \
             position, an expression position, a pattern) you will get that instead, because a \
             precise code is something tooling can act on.",
        "DL0202" => "An expression was required here and the parser found something that cannot \
             start one. Common causes: a trailing comma, an operator with no right-hand side, or a \
             `{` where a value was expected — remember that a record literal needs its type name in \
             front of the brace.",
        "DL0203" => "A TYPE was required here. Types appear after `:` in a parameter or binding, \
             after `->` in a return, and inside `[ ]` type arguments. A literal or an operator in a \
             type position produces this.",
        "DL0204" => "Every file begins with a `module` declaration naming the module it defines. \
             This is not ceremony: a module is the unit of authority (§5.5), and a file that did \
             not say which module it belongs to could not be reasoned about as one.",
        "DL0205" => "A PATTERN was required — the left of `=>` in a `match` arm, or a binding \
             position. Patterns are variant names, record shapes, literals, an identifier, or `_`. \
             An arbitrary expression is not a pattern.",
        "DL0206" => "Comparison operators are NON-ASSOCIATIVE: `a < b < c` is refused rather than \
             silently meaning `(a < b) < c`, which in most languages compares a boolean against a \
             number. Write `a < b && b < c` and say what you meant.",
        "DL0207" => "The left of `=` must be something that can be assigned to: a `var` binding, a \
             field of one, or an index into one. A literal, a call result, or a `let` binding \
             cannot be assigned — `let` is a binding, not a variable.",
        "DL0208" => "An ITEM was required at the top level of a module: `fn`, `type`, `effect`, \
             `actor`, `let` (a module constant), `foreign`, `test`, or `pub` in front of one. \
             Statements do not float at module level, because module-level mutable state is \
             forbidden (DL0305).",
        "DL0209" => "Two statements ran together where one had to end. Statements are separated by \
             a newline or a `;`. This usually means a missing newline, or an expression that \
             consumed less than you expected.",

        // ===== DL03xx — resolution =========================================
        "DL0301" => "This name is not defined in any scope reachable from here. Check the spelling, \
             whether it needs an `import`, and — if it comes from another module — whether it is \
             declared `pub` there. An import brings a module's public names into scope UNQUALIFIED.",
        "DL0302" => "This name is already defined in this module. Two definitions with one name \
             would make every use of it ambiguous, so the second is refused rather than shadowing \
             the first.",
        "DL0303" => "The imported module does not exist in this package or in any declared \
             dependency. Check the module name, and check that the package providing it is in \
             `[dependencies]` — access is not authority (§5.7), but you still need access.",
        "DL0304" => "The imports form a cycle. Module graphs are acyclic so that initialization \
             order and re-export resolution are well-defined. Break the cycle by moving the shared \
             declarations into a module both sides import.",
        "DL0305" => "Module-level MUTABLE state is forbidden. A `let` constant is fine; a `var` is \
             not. Shared mutable module state is ambient authority in disguise — it lets two \
             functions communicate through a channel that appears in neither of their signatures, \
             which is exactly what the effect system exists to make impossible.",
        "DL0306" => "This effect name is not one the language knows. The core effects are `Read`, \
             `Write`, `Net`, `Clock`, `Rand`, `Declassify`, and `ForeignCall`; a module may declare \
             more with `effect`. A row variable is written bare (`! e`), not in braces.",
        "DL0307" => "This is not a capability resource kind. The kinds are `Console`, `FsRead`, \
             `FsWrite`, `Http`, `Clock`, `Rand`, `Declassify`, `ForeignLoad`, `PluginHost`, and \
             `Python`. Capability kinds are fixed by the language, because each one corresponds to \
             a primitive the runtime actually implements.",

        // ===== DL04xx — typing =============================================
        "DL0401" => "The type found does not match the type required. There are no implicit \
             conversions anywhere in the language — not between numeric types, not to string, not \
             to bool — because an implicit conversion is a place where the program means something \
             other than what it says.",
        "DL0402" => "Numeric types do not mix. `Int` and `Float` are distinct and there is no \
             promotion: write the conversion you want. Silent numeric promotion is a classic source \
             of precision bugs that only appear at scale.",
        "DL0403" => "The call passes the wrong NUMBER of arguments. For a primitive, the expected \
             count is the arity column in `docs/reference/primitives.md`, and it is normative: a \
             call with surplus arguments is refused rather than having them ignored.",
        "DL0404" => "This value is not a function and cannot be called. Check for a stray `(` after \
             a value, or a local binding that shadows the function you meant. Note that there is no \
             implicit call: a function referred to by name is a value, and calling it always takes \
             an explicit argument list.",
        "DL0405" => "This type has no such field or method. For a capability or builtin, the \
             complete set is the primitive table (`docs/reference/primitives.md`) — it is the \
             checker's single source of truth, so anything missing from it does not exist.",
        "DL0406" => "A generic type was given the wrong number of type arguments — `Box[Int, Str]` \
             where `Box[T]` takes one. The declaration's parameter list is the authority. There is \
             no partial application of type arguments and no inference of the missing ones at a use \
             site: a type is written completely or not at all.",
        "DL0407" => "This `match` does not cover every case. Exhaustiveness is what makes adding a \
             variant a COMPILE error at every place that handles it, instead of a silent \
             fallthrough discovered in production. Add the missing arms, or `_` if you genuinely \
             mean everything else.",
        "DL0408" => "The condition of an `if` or `while` must be `Bool`. There is no truthiness: a \
             non-empty string, a non-zero number, and a present value are not `true`. Write the \
             comparison you mean.",
        "DL0409" => "`?` propagates a `Result` error, so it is only valid on a `Result` inside a \
             function that itself returns `Result`. In a function that does not, handle the error \
             with `match` — there is no exception to fall back on, by design (§5.8).",
        "DL0410" => "A generic variable is either a TYPE variable or a ROW variable, never both. \
             Using `e` as `fn(e) -> e` and as `! e` in one signature is ambiguous; give them \
             separate names.",

        // ===== DL05xx — effects ============================================
        "DL0501" => "This function performs an effect its row does not declare. This is the \
             language's central rule: a function may do only what its type says it may. Either add \
             the effect to the row — and accept that every caller now carries it too — or stop \
             performing it. The repair that adds it is flagged `authority_widening`, so automated \
             tooling will not apply it for you: widening authority to silence a diagnostic removes \
             the objection rather than fixing the program.",
        "DL0502" => "This function DECLARES an effect it never performs. A warning, not an error: \
             over-declaring is safe — it never lets the program do more than it says — but it is \
             dishonest about what the code actually does, and it makes the authority report less \
             useful to everyone reading it.",
        "DL0504" => "One row variable was bound to two conflicting effect sets. Rows never \
             union-merge to resolve a conflict — merging is precisely how an effect would enter a \
             row nobody wrote (audit rule R-3). Make the two uses agree, or separate them.",

        // ===== DL06xx — capabilities and secrets ===========================
        "DL0601" => "A capability cannot be constructed, forged, cast, or deserialized into being. \
             Every capability is DERIVED from the `Root` handed to `main` — that is the whole \
             object-capability model in one sentence (§5.1). If a function needs a capability, it \
             takes one as a parameter; there is no other way to obtain one.",
        "DL0602" => "A `Secret[T]` cannot flow where a plain `T` is expected. Secrets never coerce. \
             The only unwrap is `expose`, which requires `Cap[Declassify]` and carries the \
             `Declassify` effect — so declassification is visible in the authority report rather \
             than happening quietly.",
        "DL0603" => "`Secret.map` requires a PURE function. An effectful mapper could exfiltrate \
             the plaintext without ever calling `expose`, which would make the secret machinery \
             decorative. The result stays `Secret`.",
        "DL0604" => "An opaque type has no string form and no serialization. This is not an \
             oversight: a secret that can be printed is a secret in every log file (audit rule \
             R-5). To reveal it deliberately, use `expose` and hold `Cap[Declassify]`.",
        "DL0605" => "An opaque type has no structural equality. `==` on a secret would leak its \
             contents a byte at a time through timing. Use `verify`, which compares in constant \
             time and returns only a `Bool`.",

        // ===== DL07xx — authority ==========================================
        "DL0701" => "`main`'s effect row exceeds what the package's `[authority]` manifest permits. \
             The manifest is a CEILING the code is checked against, not a claim taken on trust. \
             Either narrow what the program does, or widen the manifest — the second is a real \
             review decision, because the manifest is what a reader trusts.",
        "DL0703" => "The program tried to use a root slice it was not granted. The slice — a path \
             prefix, a host list — is runtime scope carried by the capability, checked when the \
             operation runs (§5.3). The static row proves what KIND of thing the program can do; \
             the scope decides which resource.",

        // ===== DL08xx — custody ============================================
        "DL0801" => "This reference was revoked and cannot be called through. Revocation is \
             immediate and final: a reloaded plugin gets a FRESH node, never the old one, so a \
             retained reference can never come back to life. The diagnostic carries the audit \
             sequence that revoked it.",
        "DL0802" => "A grant may never exceed the authority its grantor holds — at any depth of \
             delegation (audit rule R-7). Attenuation is monotone: derivation only ever narrows. \
             The repair is the intersection of what was asked for and what may be given.",
        "DL0803" => "A function-typed argument cannot be passed to a Contained plugin export (rule \
             R-6a). Unverifiable code holding a re-entry point into verified code cannot be bounded \
             by any effect row. This is refused at COMPILE time, at the call site.",

        // ===== DL09xx — runtime faults =====================================
        "DL0901" => "An integer operation overflowed. Arithmetic is checked, not wrapping: a silent \
             wrap turns a bug into wrong data that flows onward. Use a wider computation or check \
             the operands.",
        "DL0902" => "Division (or remainder) by zero. Reported as a diagnostic with a code, never \
             as a host crash — a runtime fault must always be something a caller can distinguish \
             and handle.",
        "DL0903" => "An index was outside the bounds of the collection. Bounds are ALWAYS checked — \
             there is no unchecked indexing anywhere in the language, and no flag to turn the check \
             off, because a program that reads past the end of a buffer is the oldest security bug \
             there is. Use `get`, which returns an `Option`, when the index may be out of range.",
        "DL0904" => "An operation reached outside its capability's SCOPE — a path outside the \
             granted prefix, a host not on the granted list. The static row said the program may \
             read; the capability decides what it may read. Refused at the moment of use.",
        "DL0905" => "Recursion exceeded the interpreter's depth bound. Reported as a diagnostic \
             rather than crashing the host: a raw stack-overflow abort gives a caller no code and \
             no way to tell what happened. If the recursion is legitimate, restructure it \
             iteratively — the bound exists so that a runaway program fails in a way you can act \
             on.",
        "DL0907" => "A `match` reached no arm at runtime. Exhaustiveness is checked statically \
             (DL0407), so seeing this means the CHECKER let something through: it is a compiler \
             bug, not your code's fault. Please report it with the program that produced it.",

        // ===== DL10xx — packages and provenance ============================
        "DL1001" => "A dependency's authority exceeds what the lockfile pinned for it. This is the \
             xz scenario refused mechanically: a patch release that quietly gained an effect \
             anywhere in the graph fails the build BEFORE it runs. Review what changed, then either \
             re-pin deliberately or do not upgrade.",
        "DL1002" => "The locked authority hash does not match: the same version now carries a \
             different authority. A version's authority is part of what was pinned, so a change to \
             it under an unchanged version number is refused rather than absorbed.",
        "DL1003" => "Authority widened without a MAJOR version bump. Authority is part of the \
             public interface and obeys semver like any other part of it. Under `0.x` the minor is \
             the compatibility axis (cargo's rule).",
        "DL1004" => "`delulu.toml` is malformed or missing a required field. The manifest is parsed \
             as a pure function of its text — no code runs at resolution time — so every problem \
             here is a text problem with a span.",
        "DL1005" => "The package or re-export graph contains a cycle. Both are acyclic so that \
             resolution terminates and authority composes in one direction only. A cycle would also \
             make the authority of each package in it depend on itself, which has no least \
             solution. Break it by extracting what both sides need into a package they both depend \
             on.",
        "DL1006" => "An import is ambiguous between a local module and a dependency of the same \
             name. Rename one, or qualify the import — resolving it silently either way would make \
             the program's meaning depend on a coincidence.",
        "DL1007" => "A git dependency must pin a `rev` or `tag`. A moving branch is not a \
             dependency, it is a subscription: the same lockfile would produce different code on \
             different days, which defeats the point of having one.",
        "DL1008" => "Two different sources claim the same package name in one graph. One name \
             resolves to one source, always — otherwise the authority report would describe code \
             that is not the code being built.",
        "DL1009" => "This package performs an effect its own `[authority]` manifest does not \
             permit. The manifest is checked against the code, never the other way round: a \
             package cannot describe itself into having permission.",
        "DL1010" => "A dependency's source content hash does not match the lockfile. The bytes \
             changed under a locked version. This is tampering, a mutated cache, or an upstream \
             that rewrote a tag — none of which should be absorbed silently.",
        "DL1011" => "A `--locked` build requires a resolution that `delulu.lock` does not contain. \
             `--locked` means exactly that: resolve nothing new. Run `delulu lock` deliberately, \
             review the diff, and commit it.",

        // ===== DL11xx — harnesses ==========================================
        "DL1101" => "A runtime effect was performed that is NOT in the statically declared row. \
             This is the soundness claim being checked against reality by `--assert-trace`, and a \
             violation is compiler-bug class: the type system promised something the runtime did \
             not honor. Please report it with the program.",
        "DL1102" => "A repair the compiler offered did not produce an accepting program. Emitted by \
             the fuzz harness, which applies repairs mechanically and re-checks. A repair that does \
             not fix what it claims to fix is a bug in the repair, not in your code.",

        // ===== DL12xx — the WASM backend ===================================
        "DL1201" => "This construct is not supported by the WASM backend, so the program runs on \
             the interpreter instead. Reported rather than silently degraded — you should always \
             know which engine executed your code, because that is the difference between a \
             measured number and a guess.",
        "DL1202" => "The artifact's `delulu:authority` section is missing or invalid. An artifact \
             carries its own authority; one that cannot produce it is refused rather than run, \
             because running it would mean running code whose authority nobody can state.",
        "DL1204" => "The artifact declares a `delulu:cap` interface version this toolchain does not \
             support. Upgrade the toolchain. Refusing is the honest answer: interpreting an unknown \
             interface version by guessing would be a silent compatibility break.",
        "DL1205" => "Secret contents cannot enter the WASM guest. The guest's memory is a different \
             trust domain, and a secret that crosses into it is no longer protected by the rules \
             that made it a secret.",
        "DL1206" => "The two engines disagreed on the same program in a parity self-check. \
             Compiler-bug class: the interpreter and the WASM backend must produce the same \
             observable behavior for any program both support. Please report it.",

        // ===== DL15xx — plugins ============================================
        "DL1501" => "The plugin manifest's export signature does not match the plugin's actual \
             code. The manifest NEVER overrides the code — it is checked against it. A mismatch is \
             refused at build so a plugin cannot be described into having a shape it does not have.",
        "DL1502" => "The requested grant exceeds the plugin's declared ceiling. The ceiling is the \
             most a plugin may ever receive, decided when the plugin was built; the grant is what \
             it receives today. The repair is the intersection.",
        "DL1503" => "The plugin's DIR version is not supported by this toolchain. Rebuild the \
             plugin. A DIR is a verified intermediate representation, and accepting an unknown \
             version would mean trusting a structure this toolchain cannot check.",
        "DL1504" => "A Verified plugin failed re-verification: its code no longer checks as \
             shipped. It is REFUSED — never silently downgraded to Contained. Falling back would \
             turn a broken proof into a quiet loss of guarantee, which is worse than a failure \
             because nobody would notice.",
        "DL1505" => "A Contained module imports something outside its grant slice. Contained code \
             may reach only what its grant covers; an import beyond it is refused at load, not at \
             first use.",
        "DL1506" => "The plugin exceeded a resource limit and was TERMINATED, and its grant node \
             revoked. Dead, not wounded: a plugin that hit a limit does not get to continue with \
             reduced capacity, because a partially-running plugin is a plugin in an unexamined \
             state.",
        "DL1507" => "The plugin was built against a different plugin API version. Rebuild it. The \
             API version gates the host/plugin contract, and a mismatch there is not something to \
             paper over.",
        "DL1508" => "The `.dpx` container is malformed or tampered. It is refused cleanly — never \
             partially loaded — because a partially loaded plugin is code running from a file the \
             host could not fully parse.",
        "DL1509" => "A Contained plugin export's signature must be CONCRETE at the `get` site. With \
             an unresolved type variable, rule R-6a (no function-typed parameters into Contained \
             code) is undecidable — and an undecidable security rule must REFUSE, never skip. \
             Annotate the binding with the export's exact type.",
        "DL1510" => "The plugin has a signature and it does not verify: tampered content, the wrong \
             key, or a malformed signature. This is a DIFFERENT and louder failure than being \
             unsigned (DL1511) — they call for different responses, so they get different codes.",
        "DL1511" => "The grant requires a signature and the plugin is unsigned. `require_signed` \
             means what it says. See DL1510 for the case where a signature is present but invalid.",

        // ===== DL16xx — actors and reference capabilities ==================
        "DL1601" => "A non-sendable value cannot cross an actor boundary — an unconsumed `iso`, an \
             aliased mutable reference, or a pinned foreign object. This is data-race freedom by \
             TYPING rather than by lock discipline: what cannot be shared cannot be raced.",
        "DL1602" => "This binding was `consume`d and is now dead. Consumption transfers ownership, \
             so the original name must not be usable afterwards — that is what makes the transfer \
             exclusive rather than a copy.",
        "DL1603" => "This alias violates a reference capability's deny property — for example a \
             `val` (immutable, shareable) closure capturing a `ref` (mutable, local). The deny \
             properties are what let the compiler conclude that a shared value cannot change under \
             another actor's feet.",
        "DL1604" => "The access is denied by the receiver's capability: no write through `box`, no \
             field read through `tag`, no synchronous call on `tag`. A `tag` is an identity you may \
             send messages to, not a window into another actor's state.",
        "DL1605" => "A `recover` block cannot reference a non-sendable binding from outside it. \
             `recover` produces a value with a stronger capability than its parts; that is only \
             sound if nothing outside can still reach into it.",
        "DL1606" => "A behavior cannot declare a return type: behaviors are ASYNCHRONOUS and yield \
             `Unit` at the send site. A return value would require a hidden await, and hidden \
             awaits are how concurrency becomes unpredictable.",
        "DL1607" => "This reference capability is not valid for this type — most often a non-`tag` \
             capability on an actor type. An actor reference is `tag`: you may send to it, and you \
             may not read it.",
        "DL1608" => "This identifier collides with a v0.7 keyword (`consume` or `recover`). Rename \
             it, or run `delulu fmt --migrate 0.7`, which performs the rename mechanically across a \
             whole tree. The collision is reported as its own code rather than as a confusing \
             cascade of parse errors, so the fix is obvious from the first line you read.",
        "DL1610" => "The debug race-checker found a uniqueness violation under `--debug-rcaps`. \
             Compiler-bug class: the reference-capability system is supposed to make this \
             impossible statically. The checker is a detector, never the guarantee. Please report \
             it.",

        // ===== DL17xx — tooling ============================================
        "DL1701" => "The LSP could not read its workspace configuration, or was given a workspace \
             root it cannot use. The language server is analysis-only — it never constructs an \
             interpreter, a broker, or a plugin host — so this is always a configuration problem \
             rather than a program problem.",
        "DL1780" => "`delulu atlas` refuses to draw a graph for a program with check errors. A \
             partial graph of a program that does not compile would show relationships that are not \
             real. Fix the errors first; the atlas will then describe something that exists.",
        "DL1781" => "The custody overlay is unavailable because the broker daemon is not reachable, \
             so the atlas was emitted WITHOUT it. Said out loud rather than drawn as though the \
             overlay were empty — an absent overlay and an empty one mean very different things.",

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
        // ----- DL141x — the Guard (Stage 5 addendum). Every body carries GUARD_CAVEAT verbatim
        // (appended below, like the DL13xx FOREIGN_CAVEAT). DL1410's body carries the request-command
        // remediation; DL1412's states the comment is the principal's words (addendum §3.3).
        "DL1410" => {
            return Some(format!(
                "A DELEGATED grant tried to use (or mint into a child) authority a guard rule marks \
                 `guarded`, and no permit covers it. The guard is the approval checkpoint between \
                 delegated agents and the classes where a mistake is catastrophic or the type \
                 system's guarantees end — by default: declassification of secrets, and native \
                 (foreign) code. Escalate with the exact command the refusal names: `delulu guard \
                 request <node> --use <class:pattern> --why \"<justification>\"` — the `--why` is \
                 mandatory (the principal reads it in `delulu guard pending`). Once the principal \
                 approves (`delulu guard approve <id>`), simply retry: the broker holds the permit; \
                 no token is handed to you. Root (principal-held) grants are never gated — the guard \
                 keys on tree position, not identity. See `delulu explain E-GUARD`.\n\n{GUARD_CAVEAT}"
            ))
        }
        "DL1411" => {
            return Some(format!(
                "Your guard request for this access is PENDING — the refusal carries the request id. \
                 The principal sees it (with your `--why`) in `delulu guard pending` and decides with \
                 `delulu guard approve <id>` or `delulu guard deny <id> --comment \"...\"`. Retry \
                 after the decision; pending requests expire after 30 minutes. See `delulu explain \
                 E-GUARD`.\n\n{GUARD_CAVEAT}"
            ))
        }
        "DL1412" => {
            return Some(format!(
                "The principal DENIED your guard request; the message carries the principal's \
                 comment VERBATIM — those are the principal's words, stating why. Do not retry the \
                 same access expecting a different outcome; adjust your approach per the comment, or \
                 make a NEW request with a different `--use`/`--why` if the comment invites one. See \
                 `delulu explain E-GUARD`.\n\n{GUARD_CAVEAT}"
            ))
        }
        "DL1413" => {
            return Some(format!(
                "The matched guard rule is `sealed`: this authority is NOT runtime-approvable — no \
                 request/approve flow can lift it, and bypass mode does not lift it either. Only a \
                 principal policy edit (owner-coded) can: `delulu guard policy unset <class:pattern> \
                 --owner <code>`, or set a weaker tier (`guarded`/`warn`). Sealing is the principal's \
                 opt-in hardening for classes that must never be granted mid-session. See `delulu \
                 explain E-GUARD`.\n\n{GUARD_CAVEAT}"
            ))
        }
        "DL1414" => {
            return Some(format!(
                "A guard ADMIN verb (approve, deny, policy set/unset, bypass, permits revoke) was \
                 refused: the owner code is missing or wrong. The daemon prints the code exactly \
                 once at `delulu broker start` (`gow1_…`); it lives only in daemon memory, never on \
                 disk, and rotates on every daemon restart. Pass it with `--owner <code>` or the \
                 DELULU_GUARD_OWNER environment variable. If you lost it, restart the broker and \
                 capture the fresh one. See `delulu explain E-GUARD`.\n\n{GUARD_CAVEAT}"
            ))
        }
        "DL1902" => "A bounded mailbox was full and the `drop-new` overflow policy dropped the \
             message — and because the program runs in abort mode, the drop is an error rather \
             than telemetry: abort mode is the statement that losing work is worse than stopping, \
             and a silently dropped message IS lost work. Outside abort mode the same drop is \
             counted per actor and reported at exit (`--trace-memory` shows peaks and drops). \
             The alternatives: raise the bound (`actor A(mailbox = N)` or `[actors] mailbox`), \
             or use the default `block` policy, which suspends the SENDING turn at the send site \
             until space frees — bounding memory instead of losing messages. Honesty note: \
             backpressure bounds memory, never liveness — a cycle of full `block` mailboxes can \
             deadlock, and same-worker sends bypass the bound (a worker cannot wait on a mailbox \
             only it can drain).",
        "DL1903" => "A resolved dependency is on a version named in a published security advisory \
             (the registry's advisory feed, spec §4). By default this is a WARNING and the build \
             proceeds: an advisory is information, and a toolchain that turned every advisory into \
             a hard wall would only teach people to reach for `--ignore`. In CI, pass \
             `--deny-advisories` and every match becomes an error — a build that uses a \
             known-vulnerable version stops. The repair is exact when the advisory names a patched \
             version: upgrade to it and re-lock (`delulu add <pkg>@<patched>`, then `delulu lock`). \
             The feed is read from a local file (default `delulu.advisories.json` next to the \
             package, or `--advisory-feed <path>`), synced from the registry — so an offline build \
             still sees the advisories it last fetched, and the registry being down never silences \
             a warning you already hold. Skip-branch honesty: under `--deny-advisories`, a feed \
             that is missing, unreadable, or carries a record this toolchain cannot parse is itself \
             a build failure, not a silent pass — a CI gate that opens because it could not find \
             its evidence is the gate opening on damage (the same rule DL1905 draws for a missing \
             sign-off record). WITHOUT `--deny-advisories`, an absent feed is simply silence: there \
             is genuinely nothing known to warn about, so nothing is said.",
        "DL1904" => "An actuator command asked for something its envelope does not vouch for, and \
             the envelope refused it — the COMMAND dies, never the process (spec §5.1): the \
             program receives `Err(Envelope(reason))` and keeps running, free to clamp, retry, \
             or degrade gracefully. The envelope is fail-closed on every branch: a field naming \
             a dimension the envelope never bounded, a non-numeric field, a NaN, and a value \
             outside the inclusive `lo..hi` are all refused alike, because the envelope cannot \
             vouch for what it never bounded. This code is TELEMETRY, not a fault: each refusal \
             appends a trace record (op `command.refused`, the reason in `detail`) so an auditor \
             sees both the attempt and the refusal. Honesty note: the envelope is the runtime's \
             own last line, and it is DOUBLE enforcement, not the only enforcement — a physical \
             deployment still needs hardware interlocks; software bounds are necessary, never \
             sufficient. The repair is on the sender: clamp the command to the envelope, or \
             renegotiate the grant with the human who wrote it.",
        "DL1905" => "A run asked to bind REAL hardware (`--broker-profile hw:<adapter>`) for an \
             artifact that no simulation has approved (spec §5.4, invariant 48). The rule is that \
             the bytes exercised in simulation are the bytes a hardware grant covers: sign off a \
             sim run with `--broker-profile sim --signoff <record>`, then pass that record as \
             `--approved <record>`. Two situations produce this code and both are refusals. The \
             artifact's content hash differs from the approved one — an edit after sign-off, even \
             a comment, is a different program at the end of a wire that moves something. Or \
             there is no sign-off record at all, which is the more important case: when the gate \
             cannot tell whether these bytes were ever simulated, it says NO. A gate that opens \
             when it cannot find its evidence is not a gate. Re-approving is a HUMAN act \
             (`requires_human: true`) because the question it answers — has anyone watched this \
             exact program drive this exact machine in simulation — is not one a toolchain can \
             answer for itself. Honesty note: this gate governs which artifact may be bound to an \
             adapter. It is not a safety case, it does not inspect what the program does, and it \
             does not replace the hardware interlocks the deployment needs anyway. The same rule \
             generalizes past hardware (spec §9.3): `delulu fleet update` reuses this exact code \
             for a fleet member's staged rollout, where the artifact is an OTA payload rather \
             than a program bound to an adapter, and the sign-off is a human's approval of that \
             payload's bytes rather than of a simulation run. There too a missing or unreadable \
             approval record is refused exactly like a mismatched hash — never read as nothing to \
             check — and re-approval is still a human act.",
        "DL1912" => "A kernel artifact named in a compute grant could not be accepted: the file              is unreadable, its bytes are not a kernel artifact this build understands, or its              detached signature does not verify over them (spec §7.1). The last case is the              serious one and it is a refusal regardless of policy — a signature that does not              verify means the artifact is not what it claims to be, whether that is tampering, the              wrong key, or a truncated file. This is a separate code from DL1913 on purpose: an              unsigned artifact needs signing, a badly-signed one needs investigating, and one              message cannot honestly say both. Note the order this check runs in: the signature is              verified BEFORE the artifact is parsed, because reading structure out of bytes you              have not authenticated is how a malformed-input bug becomes a supply-chain one.",
        "DL1913" => "A kernel artifact has no detached signature (expected beside it as              `<artifact>.sig`), and kernels are always signed (spec §7.1, criterion 7). A kernel is              foreign code that runs on hardware the program cannot otherwise reach; DeluluLang              bounds its reachability, its resources and its PROVENANCE, and provenance is the only              one of the three that says anything at all about what the kernel will do. An unsigned              kernel has none. There is deliberately no policy switch to accept one: unlike a              plugin, where `require_signed` is a grant-level decision, a kernel with no provenance              is refused everywhere, always.",
        "DL1907" => "A kernel dispatch asked for more than the compute device's granted envelope \
             allows (spec §7.1, invariant 50) — a buffer past `memory_bytes`, a kernel past its \
             `kernel_ms` budget, or more work in flight than `queue_depth` permits. The refusal is \
             a VALUE, not a fault: `ComputeErr::KernelEnvelope` kills the dispatch and leaves the \
             program running, for the same reason an over-envelope actuator command does — a host \
             that dies mid-pipeline is worse than one that is told no and carries on. The envelope \
             is not in the program: it is in the grant a human wrote, and no code on this side of \
             the boundary can widen it. Honesty note, and it is the important part: DeluluLang \
             bounds a kernel's REACHABILITY, RESOURCES and PROVENANCE. It says nothing whatsoever \
             about what the kernel computes — that is foreign code, outside the proof, and \
             `delulu authority` prints it under the outside-the-proof separator so the boundary is \
             visible before anyone runs anything.",
        "DL1908" => "A signature was refused by POLICY rather than by mathematics (spec §8.2,              invariant 51). Two situations produce it. The artifact carries a classical-only              signature — either a v1.0 detached ed25519 signature or a `dlsig1` envelope naming              only classical algorithms — while the verifier was told to require hybrid. Or the              envelope names an algorithm this build cannot evaluate. The second case is refused              under EVERY policy, not just hybrid-required, and the reason is the house rule: a              verifier that shrugs at a claim it cannot check is the 'when the checker cannot tell,              it says yes' failure wearing a crypto-agility costume. The cost is real and stated              rather than discovered — adding a new algorithm means updating verifiers BEFORE              signers start using it (announce, then adopt), which is the safe rollout order              anyway. Note what hybrid means here: never PQ-only. An envelope carrying a              post-quantum signature and no classical one is also refused, because the guarantee              must never be weaker than what v1.0 already ships.",
        "DL1909" => "A `delulu deploy plan` computed the whole-deployment authority answer for \
             every named service — the same effect summary `delulu authority` prints for one \
             package — and at least one service's effects include something the environment \
             profile's ceiling does not list (spec §9.2, invariant 53: no plan, no launch). The \
             refusal covers the WHOLE plan, never a partial approval: passing every service except \
             the one that widened would still deploy something nobody checked against the profile, \
             which defeats the reason to compute an authority answer before anything runs. The \
             message names the exceeding service and its exceeding effect(s) — never a generic \
             'authority exceeded' — because the repair is either narrowing the plan (drop or \
             replace the service performing that effect) or widening the environment profile, and \
             only a human should choose which. This is a policy refusal in the shape of DL1905's \
             hardware sign-off gate, not a text edit: there is no byte-offset span in a `.delulu` \
             source file that would make a TOML environment profile wider, so this diagnostic \
             carries no machine-applicable repair. Honesty note: the ceiling check compares \
             EFFECTS only — it says nothing about capability SCOPES, foreign holes, or what a \
             service actually does once running. It is the authority-widening gate for \
             infrastructure, not a safety case for it.",
        "DL1910" => "A post-quantum operation was attempted without `--unstable`. This build's              ML-DSA/ML-KEM implementations are adopted from RustCrypto — cryptography is never              hand-rolled — and they are not yet stable HERE for two independent reasons, both              published (build-order D14b): the official NIST known-answer vectors have not been              validated against byte-exactly, and the implementations state they have never been              independently audited. Either alone is disqualifying. The ruling that matters: **KAT              validation is necessary, not sufficient** — a known-answer test proves an              implementation computes the standard's answers and says nothing about constant-time              behaviour or conduct under adversarial input, which is what an audit finds. Both              signing AND verifying are gated, and the verify side is the more important of the              two: verifying is invoking unvalidated cryptography to make a TRUST DECISION.              `--unstable` is available and is a deliberate decision to use unvalidated              cryptography, not a formality.",
        "DL1911" => "A compute grant named an adapter that cannot attest INDEPENDENT \
             BELOW-ADAPTER envelope enforcement — a layer beneath the adapter that would refuse an \
             over-envelope submission even if the adapter itself were wrong, absent, or lying. \
             Spec §7.1 claims the envelope is checked host-side AND adapter-side; this code exists \
             so that claim is never silently single. Three situations produce it: the adapter \
             attests nothing, the adapter is one this build has never heard of (an unknown adapter \
             cannot attest anything about itself, and `unknown` is not a reason to proceed), or it \
             does not accept a kernel format the grant lists. Note what a grant CANNOT do: assert \
             an attestation. Attestation is a property of the adapter's code, because a grant \
             string that could claim it would make this gate a checkbox. What a human CAN do is \
             waive the requirement deliberately, with `waive-attestation` in the grant — a \
             policy-explicit, `requires_human: true` decision to accept single enforcement, \
             recorded in the authority report rather than hidden. The in-tree `cpu-reference` \
             adapter attests false and always will: it runs in this process, so there is no layer \
             below it, and its envelope checks are the same code in the same address space as the \
             thing being bounded.",
        "DL1906" => "The program carries a `@jit` hint, but native-code emission was not granted, \
             so the hint was IGNORED and the program ran under the interpreter — correctly, with \
             the sandbox intact. This is a warning, never an error: a hint may not change what a \
             program does (invariant 45), including whether it runs. Getting the grant is two \
             separate, deliberate acts: the package declares its request in the manifest \
             (`[authority] exec.native = true`, a reviewable statement of intent), and the human \
             grants it at the prompt (`--grant exec.native`) — `--grant-manifest` deliberately \
             does not confer it, because emitting native code enlarges the attack surface, and \
             widening an attack surface is an AUTHORITY-WIDENING decision no tool should make for \
             you. Honesty note: v1.x ships no native tier at all — today the grant changes nothing \
             but this note. The gate exists before the engine so that no engine ever exists \
             ungated.",
        "DL1901" => "This attribute is not one v1.x defines. The full set is `@aot`, \
             `@interpret`, `@jit`, and `@inline(\"never\"|\"always\")`, and they may sit on `fn` \
             and `actor` declarations and the module header. All of them are HINTS: the \
             scheduler may ignore any of them, and none changes what a program means or may do \
             (invariant 45 — the conformance suite passes identically with and without every \
             hint). Unknown names are refused rather than skipped because a silently tolerated \
             attribute becomes a vendor extension space nobody ruled on — new attributes arrive \
             through the RFC process or not at all. The repair removes the attribute; removing \
             a hint never changes behavior, which is exactly the point of hints.",
        "DL1801" => "This program uses a feature that has been DEPRECATED. It still works — a \
             deprecation is a warning, never a break — but it will be removed in a future MAJOR \
             version, and never before then. The deprecation policy (spec §2.2) binds the project: \
             every deprecation is RFC-gated, lives at least two minor versions before any removal, \
             and removals happen only on a major bump. Where the migration is mechanical, \
             `delulu fmt --migrate <version>` rewrites it for you and the diagnostic carries the \
             exact replacement; where it is not, the RFC named here explains what to do instead and \
             why the change was worth making. Nothing about your build changes today.",
        "DL1802" => "This package declares a language edition NEWER than the toolchain you are \
             running: `[package] language = \"1.5\"` under a 1.3 toolchain, for example. The build is \
             refused rather than attempted, because compiling code written for newer rules under \
             older ones would not fail loudly — it would silently reinterpret the program. Upgrade \
             the toolchain (the repair is exact). The reverse is always fine: an OLDER edition needs \
             no action, because minor versions are strictly additive (invariant 43), so a 1.0 \
             package still means exactly what it said under a later toolchain. A package that \
             declares no edition is read at the toolchain's own — that is the default and it is \
             back-compatible with every manifest written before 1.0.",
        "DL1702" => "The formatter produced output that is not equivalent to its input, or is not \
             idempotent — a COMPILER BUG, never your code's fault. `delulu fmt` verifies both laws \
             inline before writing any byte: identity (the reprinted program parses to the same AST, \
             comments preserved) and idempotence (formatting the output again is a no-op). On a \
             violation the file is left UNTOUCHED — the formatter never corrupts code — and this is \
             reported so you can file a bug with the offending source.",
        "DL1703" => "A `test` block's declared effect row exceeds the package's `[test-authority]` \
             ceiling in `delulu.toml`. Tests hold no ambient authority (invariant 41): each gets \
             exactly its declared row, and that row must be ⊑ the package ceiling. Either narrow the \
             test's row to what it truly needs, or widen `[test-authority]` in `delulu.toml` — the \
             latter is a real review decision (the ceiling is the most any test in this package may \
             ever do). An absent `[test-authority]` table means the ceiling is PURE.",
        "DL1705" => "A signature did not verify. `delulu verify-sig` checks a detached \
             `<artifact>.sig` (a 32-byte ed25519 public key ‖ a 64-byte signature) over the \
             artifact's raw bytes. This fault means the bytes changed since signing, the `.sig` is \
             malformed, the key is not a valid ed25519 key, or (`--key HEX`) the signer is not the \
             pinned identity. Signing authenticates ORIGIN, not behavior: a valid signature says who \
             produced the artifact, never that it is safe to run.",
        "DL1706" => "A registry index line is invalid, or `delulu publish --dry-run` found a \
             semver-authority conflict against the package's prior index line (see also DL1003: \
             widening authority is a semver-MAJOR change — a minor/patch bump may not add effects, \
             capabilities, or scopes). The index line carries the authority summary so `delulu add` \
             can show the authority diff before downloading anything. v0.8 validates against a local \
             `--index <dir>` fixture; hosted registry operations are Stage 9.",
        "DL1704" => "A message catalog had a defect: a key that is not a registered diagnostic \
             code or named CLI string, a placeholder the key does not declare, a malformed \
             line — or an attempt to override the first-run welcome, which no catalog may \
             touch. This is only a warning: the affected entry FALLS BACK to the built-in \
             en-US text and everything keeps working. Catalogs are prose — they can never \
             change codes, spans, repairs, JSON, or exit codes (the machine interface is \
             locale-invariant by construction).",
        "DL1707" => "An `assert` or `assert_eq` did not hold at runtime. This is a PANIC, like \
             integer overflow: the program (or, under `delulu test`, the failing test) stops at \
             the assertion's span, and `assert_eq` reports both compared values in the message. \
             Assertions are pure prelude builtins — they add no effects to a row — and \
             `assert_eq` on an opaque type (Secret/Cap/Root…) is refused at check time (DL0605, \
             rule R-5): comparing secrets in tests is refused like everywhere else.",
        "DL1790" => "The requested color theme could not be used: either the theme NAME (from \
             `--theme`, `DELULU_THEME`, or `~/.delulu/theme.toml`) is not a built-in \
             (`default`/`bright`/`mono`), or the `theme.toml` file was malformed, or a `[roles]` \
             override named an unknown role or color. This is only a warning — the `default` theme \
             is used and the command runs normally. Fix the name or the file, or run with \
             `--color never` to sidestep theming entirely. See `delulu explain E-PALETTE`.",
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

    /// Addendum ruling 4 / criterion 11: the guard policy-change bound reuses the §4.2 revocation
    /// sentence PATTERN verbatim — the clause after the subject is byte-identical.
    #[test]
    fn guard_policy_bound_reuses_the_revocation_pattern() {
        assert!(
            GUARD_POLICY_BOUND.ends_with(REVOCATION_BOUND),
            "the guard bound must reuse the revocation sentence verbatim: {GUARD_POLICY_BOUND}"
        );
        assert!(GUARD_POLICY_BOUND.starts_with("guard policy changes take effect: "));
    }

    /// The five guard codes (DL1410–DL1414) are registered; the no-DL1404 gap is preserved.
    #[test]
    fn guard_codes_registered_and_no_dl1404() {
        for code in ["DL1410", "DL1411", "DL1412", "DL1413", "DL1414"] {
            assert!(is_registered(code), "{code} must be registered");
        }
        assert!(!is_registered("DL1404"), "DL1404 is deliberately absent (spec §8)");
    }

    /// Phase 5m: the `E-GUARD` topic exists, states the model + the policy-change bound, and every
    /// guard code has a full explain body carrying the §2.8 honesty caveat verbatim. DL1410's body
    /// carries the request-command remediation; DL1412's says the comment is the principal's words.
    #[test]
    fn e_guard_topic_and_guard_explain_bodies() {
        let (title, body) = topic_explain("GUARD").expect("E-GUARD topic exists");
        assert!(title.contains("Guard"));
        assert!(body.contains(GUARD_POLICY_BOUND), "the policy-change bound, verbatim: {body}");
        assert!(body.contains(GUARD_CAVEAT), "the honesty caveat, verbatim");
        assert!(body.contains("TREE POSITION"), "position-not-identity stated");
        for code in ["DL1410", "DL1411", "DL1412", "DL1413", "DL1414"] {
            let b = code_explain(code).unwrap_or_else(|| panic!("{code} needs an explain body"));
            assert!(b.contains(GUARD_CAVEAT), "{code} carries the guard caveat verbatim");
        }
        assert!(code_explain("DL1410").unwrap().contains("delulu guard request"), "DL1410 names the request command");
        assert!(code_explain("DL1412").unwrap().contains("the principal's words"), "DL1412 says whose words the comment is");
        assert!(code_explain("DL1413").unwrap().contains("bypass mode does not lift it"), "DL1413: bypass never lifts a seal");
        // The bypass banner states the addendum-mandated content (agent-drafted, spec-fixed).
        assert!(GUARD_BYPASS_BANNER.contains("approval checkpoint"));
        assert!(GUARD_BYPASS_BANNER.contains("WITHOUT YOUR KNOWLEDGE until you read the audit log"));
        assert!(GUARD_BYPASS_BANNER.contains("Sealed rules still hold"));
        // Never claim "immediate" anywhere in the guard texts.
        for text in [body.as_str(), GUARD_BYPASS_BANNER, GUARD_CAVEAT] {
            assert!(!text.to_lowercase().contains("immediate"), "never claim immediate: {text}");
        }
    }

    /// Stage 6 "Live": the plugin codes DL1501–DL1511 are registered, and the `E-PLUGIN` topic
    /// exists and carries the spec §10 honesty caveats VERBATIM plus the ruled-deviation conditions
    /// (2 = DL1508, 3 = single-module, 4 = annotation form, 7 = Windows, 8 = DL1510/DL1511).
    #[test]
    fn plugin_codes_and_e_plugin_topic() {
        for code in [
            "DL1501", "DL1502", "DL1503", "DL1504", "DL1505", "DL1506", "DL1507", "DL1508",
            "DL1509", "DL1510", "DL1511",
        ] {
            assert!(is_registered(code), "{code} must be registered");
        }
        let (title, body) = topic_explain("PLUGIN").expect("E-PLUGIN topic exists");
        assert!(title.contains("Verified") && title.contains("Contained"), "the two classes named");
        // Spec §10 caveats, verbatim.
        assert!(body.contains("TRUST STATEMENT, NOT A QUALITY RANKING"), "§10: trust not quality");
        assert!(body.contains("Signatures authenticate ORIGIN, not behavior"), "§10: origin not behavior");
        assert!(body.contains("NOT I/O volume within granted scopes"), "§10: limits are CPU/mem/wall");
        assert!(body.contains("Interpreter-engine limits are best-effort"), "§10: interpreter best-effort");
        assert!(body.contains("share the host's microVM"), "§10: shared microVM in v1.0");
        // Ruled-deviation conditions.
        assert!(body.contains("DL1508"), "deviation 2: DL1508 in the diagnostics story");
        assert!(body.contains("single-module") && body.contains("DL1004"), "deviation 3: single-module + DL1004");
        assert!(body.contains("no turbofish") || body.contains("inference-from-context"), "deviation 4: annotation form");
        assert!(body.contains("will not parse"), "deviation 4: warns not to copy bracket syntax");
        assert!(body.contains("EnforcementUnsupported") && body.contains("Windows"), "deviation 7: Windows caveat, plain");
        assert!(body.contains("DL1510") && body.contains("DL1511"), "deviation 8: the two signature codes");
        assert!(body.contains("DIFFERENT faults"), "deviation 8: badly-signed vs unsigned are different faults");
        // Never claim "immediate" anywhere in the plugin text.
        assert!(!body.to_lowercase().contains("immediate"), "never claim immediate: {body}");
    }

    /// Stage 7 "Concurrent": the actor codes DL1601–DL1608 + DL1610 are registered (DL1609 is
    /// deliberately absent — house rule mirroring DL1404/DL1784), and the `E-ACTOR` topic
    /// exists and carries the spec §11 honesty caveats VERBATIM plus the load-bearing
    /// deviation notes (3 = worker-owned, 7 = move-by-rebuild).
    #[test]
    fn actor_codes_and_e_actor_topic() {
        for code in [
            "DL1601", "DL1602", "DL1603", "DL1604", "DL1605", "DL1606", "DL1607", "DL1608",
            "DL1610",
        ] {
            assert!(is_registered(code), "{code} must be registered");
        }
        assert!(!is_registered("DL1609"), "DL1609 is deliberately absent (spec §10 table skips it)");
        let (title, body) = topic_explain("ACTOR").expect("E-ACTOR topic exists");
        assert!(title.contains("data-race freedom"), "the guarantee named in the title");
        // Spec §11 caveats, verbatim.
        assert!(body.contains("bounded by their Stage-4/5/6 layers, not by rcaps"), "§11: scope of the guarantee");
        assert!(body.contains("Deadlock, livelock, starvation, and mailbox exhaustion are not prevented"), "§11: liveness never claimed");
        assert!(body.contains("the guarantee is race freedom, not liveness"), "§11: race freedom, full stop");
        assert!(body.contains("Unbounded mailboxes can exhaust memory; backpressure is post-1.0"), "§11: OOM honesty");
        assert!(body.contains("adopted from Pony's proven design"), "§11: provenance");
        assert!(body.contains("mechanized proof remains Delulu Core future work"), "§11: proof honesty");
        assert!(body.contains("cooperative single-threaded in v0.7"), "§11: WASM honesty");
        assert!(body.contains("semantics identical, parallelism absent, labeled in output"), "§11: WASM labeling");
        assert!(body.contains("per-sender-pair FIFO only; no global order, no delivery-time bounds"), "§11: ordering honesty");
        // The two axes never bleed; the rules' shape is stated.
        assert!(body.contains("SECOND axis"), "rcaps beside rows");
        assert!(body.contains("`val` closure can still be `!{Write}`"), "orthogonality example");
        // Ruled-deviation notes.
        assert!(body.contains("worker-owned") && body.contains("zero unsafe"), "deviation 3");
        assert!(body.contains("MOVES by rebuild"), "deviation 7");
        assert!(body.contains("never the guarantee"), "trap 3: the debug checker's honest framing");
    }

    /// Stage 8 (phase 8a): DL1707 — assertion failure — is a registered runtime-panic code
    /// whose explain body carries the check-time cross-reference (DL0605/R-5: opaque values
    /// never reach a runtime comparison), and the never-DL1784 house rule still holds.
    #[test]
    fn assertion_failure_code_dl1707() {
        assert!(is_registered("DL1707"), "DL1707 (assertion failed) must be registered");
        let body = code_explain("DL1707").expect("DL1707 has an explain body");
        assert!(body.contains("DL0605"), "explain links the check-time opacity refusal");
        assert!(body.contains("assert_eq"), "explain names both assertion forms");
        assert!(!is_registered("DL1784"), "DL1784 is deliberately never allocated (house rule)");
    }

    /// RELEASE CRITERION 7: **every** registered code has a long-form en-US explanation.
    ///
    /// `delulu explain <code>` is the machine surface's escape hatch and the human's first stop.
    /// A code whose explanation is only its title tells a reader nothing they did not already have
    /// on the diagnostic line. Measured at the start of Stage 9h: 95 of 150 codes were title-only.
    #[test]
    fn criterion7_every_code_has_a_long_form_explanation() {
        let mut thin: Vec<(&str, usize)> = Vec::new();
        for c in REGISTRY {
            match code_explain(c.code) {
                Some(body) if body.trim().len() >= MIN_EXPLAIN_BODY => {}
                Some(body) => thin.push((c.code, body.trim().len())),
                None => thin.push((c.code, 0)),
            }
        }
        assert!(
            thin.is_empty(),
            "these codes have no long-form explanation (criterion 7 requires 100%): {thin:?}"
        );
    }

    /// THE SKIP-BRANCH CASE (house rule 3): the coverage check must be able to FAIL. If
    /// `code_explain` returned a long string for anything at all, the test above would pass
    /// without measuring anything.
    #[test]
    fn the_explain_coverage_check_can_detect_a_missing_body() {
        assert!(
            code_explain("DL9999").is_none(),
            "an unregistered code must have no explanation, or the coverage test is vacuous"
        );
        assert!(MIN_EXPLAIN_BODY > 100, "the threshold must exclude a restated title");
    }

    /// Stage 8: the full tooling DL range (DL1701–DL1707) is registered with explain bodies,
    /// and DL1706 links the semver-authority law (DL1003) it enforces at publish.
    #[test]
    fn stage8_tooling_codes_registered_with_explanations() {
        for code in ["DL1701", "DL1702", "DL1703", "DL1704", "DL1705", "DL1706", "DL1707"] {
            assert!(is_registered(code), "{code} must be registered");
        }
        // The load-bearing ones carry real explain bodies (house rule 6).
        for code in ["DL1702", "DL1703", "DL1704", "DL1705", "DL1706", "DL1707"] {
            assert!(code_explain(code).is_some(), "{code} needs an explain body");
        }
        assert!(
            code_explain("DL1706").unwrap().contains("DL1003"),
            "DL1706 links the semver-authority law it enforces"
        );
        assert!(
            code_explain("DL1703").unwrap().contains("invariant 41"),
            "DL1703 cites the tests-hold-no-ambient-authority invariant"
        );
    }

    /// Surface addendum §3.2: the Palette code DL1790 is registered, the `E-PALETTE` topic exists
    /// and states the resolution order + NO_COLOR compliance + why JSON is never colored, and the
    /// never-DL1784 house rule holds.
    #[test]
    fn palette_code_and_e_palette_topic() {
        assert!(is_registered("DL1790"), "DL1790 (invalid theme) must be registered");
        assert!(!is_registered("DL1784"), "DL1784 is deliberately never allocated (house rule)");
        let (title, body) = topic_explain("PALETTE").expect("E-PALETTE topic exists");
        assert!(title.contains("Palette"));
        assert!(body.contains("NO_COLOR"), "NO_COLOR compliance stated");
        assert!(body.contains("--color always` flag beats `NO_COLOR"), "the flag exception stated");
        assert!(body.contains("NEVER colored"), "machine channels never colored");
        assert!(body.contains("mono"), "the mono theme is named");
        assert!(code_explain("DL1790").unwrap().contains("E-PALETTE"), "DL1790 links to E-PALETTE");
    }

    /// The Atlas codes (DL1780 = refuse on check errors, DL1781 = broker down note) are registered
    /// in the Surface range.
    #[test]
    fn atlas_codes_and_e_atlas_topic() {
        assert!(is_registered("DL1780"), "DL1780 (atlas refused) must be registered");
        assert!(is_registered("DL1781"), "DL1781 (custody overlay unavailable) must be registered");
        let (title, body) = topic_explain("ATLAS").expect("E-ATLAS topic exists");
        assert!(title.contains("Atlas"));
        // The verbatim static caveat (addendum §2.6) ships in the explain body.
        assert!(
            body.contains("static map of checked facts, not a runtime trace; calls through function \
                 values may be under-approximated"),
            "the verbatim caveat must appear: {body}"
        );
        assert!(body.contains("DL1780") && body.contains("DL1781"), "refusal codes documented");
        assert!(body.contains("Querying further"), "the digest footer is named");
        assert!(body.contains("atlas/1"), "the JSON schema is named");
        // Honesty: never claim a complete/always-current graph.
        assert!(body.contains("not a runtime trace"));
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
