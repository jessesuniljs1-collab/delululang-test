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
