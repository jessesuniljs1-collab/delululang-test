# The DeluluLang Constitution — Version 1.0

**Status:** Committed. **File extension:** `.delulu`. **Date:** 2026-07-04.

This document is the definitive design of DeluluLang v1.0. It is normative: where an earlier
draft (including `LANGUAGE_SPECIFICATION.md` and `DeluluLang_PROMPT.md`) disagrees with this
document, this document wins. Changes to this document require the evolution process in §10.

*Be delulu: write code as if no program can ever exceed its authority — then make the compiler
make it true.* That sentence is the brand. Everything below is the law.

---

## 1. Identity

> **DeluluLang is a language where every unit of code — every function, every module, and every
> dynamically loaded plugin — carries its authority and effects in its type, so that the powers
> of the entire program are verifiable by the compiler, and no code — written by a human, a robot,
> an AI, or any future evolution of AI — can exceed the authority it was explicitly granted.**

Short form:

> **Authority and effects are part of the type of every program — total, verifiable, and enforced
> across all code, all dependencies, and all runtime-loaded plugins.**

The identity is **authority, made intrinsic** — a property of the language's *semantics*.

AI agents, LLMs, and future AI systems (agentic software, physical AI, robot controllers) are the
**primary users** of DeluluLang, because they are the population for whom unrestricted authority is
most dangerous and most in need of language-enforced limits. But the audience is not the principle.
The principle is authority. An identity built on today's AI implementation details (repair loops,
token counts) would rot as AI changes; authority only grows more valuable as AI grows more capable,
because a more powerful agent is a more dangerous one to grant unrestricted authority to.

---

## 2. The Irreducibility Law

No feature enters the core language unless it passes this test:

> **Removing this feature, or moving it into a library, fundamentally changes what the language
> guarantees. If a library, framework, compiler plugin, linter, or extension to an existing
> language could deliver the same guarantee, the feature does not belong in the core.**

Applied naively — feature by feature, in isolation — this test rejects everything, because almost
any single feature can be approximated by a library (capability libraries, effect-system libraries,
sandbox frameworks all exist). The test is therefore applied to **the guarantee across the whole
program**. Exactly one thing cannot be a library: **the guarantee that the property holds for
*all* code in the program** — including code the library author never saw, including every
transitive dependency, including plugins loaded at runtime. A capability *library* protects only
code that opts in; the unsafe filesystem call three dependencies deep still happens. For "no code
anywhere in this program can exceed its granted authority" to be *true*, every function, module,
and plugin must carry authority and effects in its type, from the standard library upward. That
is a property of language semantics. It cannot be retrofitted onto Rust, Go, TypeScript, or Python,
because their ecosystems are already written without it; retrofitting means re-typing everything,
which means building a new language.

That is the justification for DeluluLang's existence. It is the only justification a new language
ever needs.

Everything that fails this test — structured diagnostics, LSP, formatter, package manager,
interop, transpilation — is a **supporting pillar** (§8): genuinely valuable, deliberately demoted,
never the identity.

---

## 3. Why Choose DeluluLang Over X?

DeluluLang does not replace the world's languages and does not try to become their superset. The
strategic question is: **when, and why, should someone choose DeluluLang over the language they
would otherwise reach for?**

The one-line answer: **choose DeluluLang when the question "what can this code actually do to my
system?" must have a compiler-verified answer** — because the code is written or modified by an AI,
because it loads plugins you didn't write, because its dependency graph is too large to audit, or
because it commands something physically dangerous. If that question can stay unanswered, use the
incumbent; it has a thirty-year head start on everything else.

- **Python** — Choose Python for exploratory work, notebooks, and the ML ecosystem. Choose
  DeluluLang when agent-generated code must *run* with provable bounds: Python's dynamism (`eval`,
  monkey-patching, ambient `import os`) makes static authority verification impossible in
  principle. DeluluLang embeds CPython (§5.12), so choosing it does not mean leaving NumPy or
  PyTorch — it means fencing them behind a visible, typed boundary.

- **JavaScript / TypeScript** — Choose them for UI and the web platform. Choose DeluluLang for the
  agent-facing backend and tooling layer, where TypeScript cannot help: its types are advisory and
  erased at runtime, punctured by `any` and by every plain-JS dependency. A security guarantee
  cannot be built on an unsound, unenforced type layer.

- **Java / C# / Kotlin** — Choose the managed enterprise platforms for what they are: mature,
  fast, staffed. None of them can answer "what can this dependency do?" — a JAR or NuGet package
  runs with the process's full ambient authority, which is exactly the supply-chain blind spot.
  Choose DeluluLang when least privilege is a *requirement* to verify, not an audit to hope about.

- **Go** — Choose Go for operational simplicity and network services. Go's philosophy is
  simplicity through *omission* — including omission of any effect or capability discipline, which
  is precisely what DeluluLang cannot omit. Choose DeluluLang when you must run code you didn't
  write (or an agent wrote seconds ago) with verified bounds.

- **Rust** — Choose Rust for memory safety at native speed; it is the closest language in spirit
  and it is DeluluLang's implementation language — a complement, not an enemy. Rust cannot provide
  the total guarantee: `std` and every crate are typed without effects, and `unsafe` punctures any
  whole-program claim by design. `cap-std` is opt-in discipline for code you control — ~80% of the
  value there, 0% of the totality. DeluluLang treats memory safety as table stakes and adds the
  layer Rust doesn't have: authority safety.

- **C / C++** — Choose them at the hardware floor: firmware, kernels, real-time control loops,
  the physics-rate layers of robotics. DeluluLang explicitly does not compete there (§7); it is
  the bounded command layer *above* that floor, and it calls C through a typed, capability-gated
  FFI. Claim committed: **competitive with C on hot paths, with safety C cannot offer** — never
  "faster than C."

- **Swift** — Choose Swift for Apple-platform applications. DeluluLang does not compete for that
  seat.

- **Zig / Mojo** — Zig: manual control and C interop with no safety story DeluluLang needs;
  different goal. Mojo: Python-superset performance on MLIR; its identity is speed and ergonomics,
  not authority. Neither has (or wants) a whole-program authority guarantee.

- **Deno (runtime, not language)** — Deno's permission flags prove the demand for deny-by-default
  execution and are the closest mainstream UX to DeluluLang's run-time grants. But they are
  *process-granularity*, opt-in at launch, and invisible to the type system: Deno can say "this
  process may read the disk," never "this function performs `Net` and that one is pure," and never
  at compile time.

**Positioning against the two nearest projects (verified 2026-07):**
[aglang](https://github.com/collivity/aglang) (v0.3.0, Apache-2.0) compiles `.ag` architecture
specs into SMT-LIB2 artifacts checked deterministically by Z3 against facts extracted from source —
LLM-free verification with provenance, and a validation of the instinct that *algorithms, not AI
guesses, should do the verifying*. [zerolang](https://github.com/vercel-labs/zerolang) (v0.1.x,
experimental) makes a semantic graph the program database — agents submit compiler-checked patches,
humans read `.0` projections — with capability-based I/O via a `World` handle and machine-first
JSON diagnostics; a validation of the *agent-patches-a-checked-structure* loop. DeluluLang absorbs
both lessons and differs on the defining property: **total, whole-program, per-function authority
and effect typing across all code and all runtime-loadable plugins.** aglang verifies
architecture-level facts via an external solver; zerolang has capabilities as one feature among
several, at coarser granularity, with tooling-for-agents as its center of gravity. Neither commits
to per-function authority totality as identity. DeluluLang does — that is the whole bet.

---

## 4. What Becomes Possible (the irreducibility test's positive twin)

Only capabilities that *require* language semantics belong here; convenience is banned from this
chapter.

1. **Provable least privilege for an entire program, dependencies included.** The compiler
   computes the union of effects and authority across the whole dependency graph and reports it:
   *"this program can read `./config`, call `api.example.com` over HTTPS, and nothing else"*
   becomes a compiler-verified fact (`delulu authority`, §8.1), not a hopeful audit.

2. **Untrusted, hot-loadable plugins that provably cannot exceed their grant.** In every
   mainstream language, loading a plugin means trusting it completely. In DeluluLang a plugin is
   just more authority-typed code, so loading, granting, and bounding it is the same mechanism as
   the rest of the language (§6).

3. **Authority as a first-class, composable value.** Capabilities are values: pass them, store
   them, and *attenuate* them ("read-only, this subtree only") with the weakening verified. Only
   language semantics can make attenuation total across the graph.

4. **A trustworthy substrate for autonomous and multi-agent software — including embodied AI.**
   Any code an agent emits or loads is automatically bounded by verified authority; each agent or
   actor holds its own authority slice; a robot actuator is just the most physically consequential
   capability (§7). This is a *consequence* of the semantics, stated last on purpose.

---

## 5. Language Semantics (normative core)

Each subsection states a **decision**, then the rejected alternatives. Soundness lives here; this
part is owned by humans and never delegated to an AI to invent.

### 5.1 Authority model — pure object-capability

**Decision.** There is **no ambient authority** anywhere in the language. A capability is an
unforgeable value that designates a resource and confers authority over it. Code can affect a
resource only if it holds a capability for it; capabilities propagate **only** by explicit passing
(as arguments, in returned values, or stored in data structures — "only connectivity begets
connectivity"). The top-level entry point `main(root: Root)` receives root capabilities from the
runtime, under human-controlled policy (§5.14); every other piece of code receives only what is
passed to it. Capability values have no literal form and no user-accessible constructor;
unforgeability is a language invariant.

Storing a capability in a record or capturing it in a closure is legal and ordinary ocap practice.
It does not launder anything: using the capability still surfaces the effect in the effect row of
every function on the call path (§5.2), and effect rows are never erased — a stored function value
keeps its row in its type.

**Rejected.** ACL / permission-checks-at-callsite (the Unix model): ambient authority is what
makes least privilege unverifiable. Capability-as-library: opt-in, therefore not total (§2).

### 5.2 Effect system — effect rows in every function type

**Decision.** Every function type includes its **effect row**: the set of effects it may perform.
The v1.0 core effects are `Read`, `Write`, `Net`, `Clock`, `Rand`, and `Declassify` (carried by
secret declassification, §5.4 — added by the soundness audit so that revealing secrets is visible
in rows), plus user-declared effects (`effect Name`), plus reserved effects activated in later
stages: `Async`, `ForeignCall`, `Load` (plugin loading), `Actuate` (robotics), `Alloc` (deferred
until it can be made non-noisy). A
function whose row is empty is provably pure. Rows compose by union: a caller's row must contain
the union of its callees' rows (width subsumption: declaring a superset is legal).

**Effects and capabilities are two views of the same authority.** The effect row says *what kind*
of thing the code does; the capability value says *it is permitted, and to which resource*. The
compiler verifies they agree: in DeluluLang, primitive effects arise **only** from invoking
operations on capability values (and, later, from FFI and plugin calls, which are themselves
capability-gated). Therefore code holding no capability can perform no effect, and the row is a
sound static summary of behavior. The two views are not redundant: a function may *hold* a
`Cap[FsRead]` and still have row `{}` — the row states what it *does*, the capability what it
*may touch*; and rows summarize *transitive* behavior without re-reading bodies.

**Effect polymorphism (added in hardening — the v1.0 spec was unusable without it).** Higher-order
functions take **row variables**: `fn map[T, U, e](xs: List[T], f: fn(T) -> U ! e) -> List[U] ! e`.
Rows are sets of effect labels with an optional polymorphic tail (`! {Net | e}`), unified at call
sites. Without this, every higher-order function would force worst-case rows and the system would
collapse into annotation noise.

**Explicitness (hardened decision — overrides "inferred where possible" from earlier drafts).**
Named function signatures **declare their rows explicitly; an omitted row means pure `!{}`**.
Inference still runs — over lambda literals (whose rows are inferred from context) and over every
body, where it powers precise diagnostics and one-keystroke typed repairs ("add `Net` to this
row") — but it never silently widens a signature. Why the override: (a) legibility of authority
*is the product* — you must be able to read a function's power off its signature; (b) package
authority stability (§5.7) is meaningless if rows drift by inference; (c) the ergonomic cost is
near zero for agents, who consume the repair objects (§8.1). Rejected: whole-signature inference
(hides authority, destabilizes public boundaries); effects-as-monads library (opt-in, fails
totality); Koka/OCaml-style resumable effect *handlers* (rejected for v1.0: first-class
continuations make the security reading of rows much harder to preserve — see §5.9; we adopt the
type-level *tracking*, aimed at authority).

### 5.3 Kind vs. scope — the honest granularity split

**Decision.** The compiler verifies authority **kind** per function (this function performs at
most `{Read, Net}`; it can only perform them via capabilities it was handed). Authority **scope**
(which directory subtree, which hosts, which joint, what torque ceiling) is data carried inside
capability values, enforced at **runtime** on every use by the capability's own implementation and
the broker, and declared at the **package/program boundary** in the authority manifest. Claiming
compile-time verification of scopes would require dependent types and would be false in v1.0; this
split is stated so the guarantee is never overstated.

### 5.4 Secrets and information flow — one authority among many

**Decision.** `Secret[T]` is a compiler-known wrapper type. A secret originates from the root
(`root.secret("API_KEY")`) or from explicit wrapping. Rules: **(1)** `Secret[T]` is not `T` — it
cannot flow into any sink parameter typed `T`, so passing a secret to a `Write`, `Net`, log, or
`Commit` sink is a **compile error**; **(2)** derived values stay tainted: operations over secrets
(`s.map(f)`) yield `Secret[U]`; **(3)** the only exits are `Secret.verify(a, b) -> Bool`
(constant-time comparison) and `s.expose(d: Cap[Declassify]) -> T ! {Declassify}`, where
`Cap[Declassify]` is a root-issued capability and `Declassify` is a core **effect** — every
function that can reveal a secret says so in its type and in the authority report;
declassification is a human-policy decision, and diagnostics **never** auto-repair toward it;
**(4)** `Secret[T]` has no string conversion, no display, no serialization, and no structural
equality — `Secret`, `Cap`, `Root`, and any composite containing them are *opaque types*,
excluded from `str`, `==`, and serialization (`SOUNDNESS_AUDIT.md` R-5).

The motivating incident is real: an AI coding agent committed a live API key to a public GitHub
repository. In DeluluLang that is a compile error. But secrets are *one authority among many* —
the example that makes authority visceral, not the identity.

**Honest limits (normative — must appear in user documentation).** Type-level taint is sound for
*explicit* data flow. It does not cover: implicit flows (branching on `verify` leaks bits by
design — that is what authentication *is*), timing and length side channels, memory disclosure via
FFI or compiler bugs, or anything done with a value after lawful `expose`. DeluluLang makes secret
*misuse* a type error; it does not make secrets unstealable by physics.

### 5.5 Modules — units of authority

**Decision.** A module declares the effects it may perform and receives capabilities only through
its interface. **No module-level mutable state, period** (hardened from "no implicit global state"
— a module-level `var` would be an ambient channel through which capabilities and data launder
across the program, silently reintroducing ambient authority). Module-level `let` of pure constants
is legal. A module's maximum authority is readable from its interface.

### 5.6 Packages — declared, versioned authority

**Decision.** A package's total authority (effect upper bound + capability requirements + scope
declarations) is stated in its manifest and verified by the compiler: a package whose code exceeds
its declared authority does not compile, and a dependency cannot silently start doing I/O in a
patch release. **Authority widening is a breaking change**: any release that widens effects or
scopes requires a major version bump, and the resolver refuses silent authority-widening upgrades.
This is the structural defense against supply-chain attacks (the xz backdoor, CVE-2024-3094,
performed effects it was never authorized for; here that fails to typecheck).

### 5.7 Imports — access is not authority

**Decision.** Importing a module grants access to its *interface*, never authority to act.
`import std.net` lets you name the types; making a request still requires being handed a
`Cap[Http]`. This separates "I can refer to this" from "I am permitted to do this."
**Rejected:** import-confers-power (`import os; os.system(...)`) — the canonical ambient-authority
footgun.

### 5.8 Errors — results, not exceptions; no continuations

**Decision.** Recoverable errors are values: `Result[T, E]` with the `?` propagation operator.
There are **no exceptions and no first-class continuations in v1.0**. `panic` exists for
unrecoverable violation of program invariants and **aborts** the actor/program (it is divergence,
not an effect, and carries no authority; it cannot be caught to smuggle control flow past a row).
This closes the classic effect-system holes: non-local control flow cannot bypass effect rows if
it does not exist. Resumable algebraic handlers may be revisited post-1.0 only with a design that
preserves the security reading of rows.

### 5.9 Concurrency — actors with reference capabilities (staged)

**Decision.** The v1.0 destination is the actor model with Pony-style reference capabilities
(`iso`, `val`, `ref`, `box`, `tag`, `trn`): compile-time data-race freedom, no shared mutable
state across actors, each actor holding its own authority slice — the natural substrate for
multi-agent software and plugin isolation. Staging honesty: the MVP runs single-threaded with GC;
reference capabilities and actors land in Stage 7. **All six reference-capability keywords, plus
`actor`, `async`, `await`, `spawn`, are reserved words from the first commit**, so their arrival
breaks no code. **Rejected:** shared-memory threads with locks (unverifiable); Go's CSP (respected,
but does not statically prevent shared-mutable races).

### 5.10 Async — an effect, not a second system

**Decision.** Asynchrony is an effect (`Async`) in the same rows, composing like every other
effect — one system, no separate function-coloring mechanism bolted on later. Reserved in Stage 1,
activated with the actor runtime.

### 5.11 Compilation — high-level surface, tiered backend, sandbox-invariant modes

**Decision.** The surface is high-level; performance comes from the backend, never from
compromising the surface. Execution uses the standard tiered model — interpret cold code, compile
hot paths via profiling and on-stack replacement, deoptimize when speculation fails — which is the
*normal* architecture of HotSpot, V8, PyPy, and Julia, not a novelty. Execution-mode hints
(`@aot`, `@interpret`, `@jit`) may be set by developer or agent under two inviolable rules:
**(1) mode selection can never weaken the sandbox** — every tier emits the same authority and
bounds checks; **(2) only human-controlled policy may grant a module the right to emit native
code.** Because a JIT is a large attack surface, untrusted agent code defaults to the
interpreter/AOT-verified path. **Rejected:** "faster than C" (false — C sits near the hardware
floor; committed claim: *competitive with C on hot paths, with safety C cannot offer*);
whitespace-significant syntax (tokenizes worse under BPE, harder for agents to edit — braces,
short keywords, explicit effects, results-over-exceptions); emoji/exotic syntax for "token
efficiency" (increases BPE tokens); "lowest tokens of any language" as a headline (tokenizer-
specific; token efficiency is a minor consequence of clean regular syntax, never a claim).

### 5.12 Interop — first-class Python and C, honestly fenced

**Decision.** C-ABI FFI and embedded CPython, so the AI ecosystem (NumPy, PyTorch) is usable
unchanged from day one — the single most important adoption lever. The boundary is honest:
a foreign call requires a `Cap[Foreign]` capability and carries the `ForeignCall` effect, so the
whole-program guarantee degrades **visibly, in the types**, never silently. Normative rules:
foreign code is *outside* the effect guarantee (it can do anything the OS process can — the only
real bound is the process/sandbox layer, §5.14); values returned from FFI are untrusted data;
`Secret[T]` values cannot cross the FFI boundary without `expose`. `delulu authority` reports
`ForeignCall` scopes as exactly what they are: holes in the proof, enumerated.

### 5.13 Portability — one portable target

**Decision.** One portable compilation target — WASM — plus per-OS runtimes; the language is never
rewritten per OS. WASM doubles as the runtime security floor (a genuine two-for-one). Native
cross-compilation may be added later for performance-critical deployment; it changes packaging,
never semantics.

### 5.14 Security — defense in depth; the human holds the keys

**Decision.** Four layers, each assuming the one above it can fail:
1. **Type system (verification):** compile-time proof of least privilege; unauthorized effects are
   compile errors.
2. **WASM/WASI runtime (containment):** deny-by-default host (Wasmtime-class); even a compiler or
   type-system bug is contained at module granularity.
3. **MicroVM (isolation):** Firecracker-class VM for genuinely untrusted execution — default-deny
   egress, read-only mounts, scoped short-lived credentials, explicit lifetimes.
4. **The capability broker (policy):** root capabilities are issued by a broker outside any
   agent's reach, controlled by the human — or, where a human delegates, by a supervising system
   the human controls. Agents receive narrow, revocable, audited grants and cannot even *see*
   capabilities they were not granted.

**Honest limits (normative).** "Unbreakable" is always relative to a threat model. The type system
trusts the compiler; the compiler trusts the hardware; the sandbox trusts the hypervisor;
microarchitectural side channels (Spectre-class) remain a standing assumption. DeluluLang
**eliminates whole classes of vulnerability by construction** — it does **not** "find all
vulnerabilities," which is undecidable. Every guarantee names its assumptions. v1.0's soundness
claims are design-level and test-enforced; a mechanized core-calculus proof ("Delulu Core") is
committed future work, not an implied present fact.

### 5.15 Runtime guarantees

Relative to the stated threat model, the runtime guarantees:
1. No code executes an effect for which it does not hold a matching capability.
2. No actor accesses another actor's mutable state (from Stage 7).
3. A plugin cannot exceed the authority granted at load time (§6).
4. The union of program authority is bounded by the human's root grant.
5. Violations are contained at the WASM/microVM layer even if the type system is bypassed.
6. Scope constraints inside capability values (paths, hosts, envelopes) are re-checked at runtime
   on every use — enforcement is doubled, not delegated to the type system alone.

### 5.16 The authority holder model (the holder principle)

**Decision.** The **Delulu Authority holder** of a grant is the party the broker issued it to —
a human, an LLM orchestrating sub-agents in parallel, an AI agent, a robot's supervisory system,
or any future evolution of AI. The rules are identical for every kind of holder: **DeluluLang does
not discriminate between human and AI parties** — every party has the same standing, the same
usability, and the same machine interface. What the model guarantees is *directional*, not
*discriminatory*: **the authority held by a granting party cannot be broken, widened, or escaped
by the parties it grants to.**

Normative laws:
1. **Monotone attenuation.** Every sub-grant must satisfy `sub ⊑ holder's grant`, where `⊑`
   requires effect-set inclusion and scope narrowing (path descendant, host subset, secret-name
   subset, numeric envelope ≤). The broker rejects wider requests. When an LLM holds a grant and
   spawns parallel coding agents, each agent's grant is an attenuation — the agents cannot exceed
   or break the LLM's authority, mechanically, in the same way the LLM cannot exceed the human's.
2. **Transitive revocation.** Revoking a grant revokes every grant attenuated from it, down the
   whole tree.
3. **No upward or lateral reach.** A grantee cannot see, enumerate, or acquire capabilities it was
   not granted — including its grantor's and its siblings'.
4. **The chain terminates at the root broker**, which is controlled by the human — or by a
   supervising system the human controls and delegates to.

**Rejected:** trust-level hierarchies keyed to the *kind* of party (human vs. AI) — rejected
because they are both discriminatory and fragile; the durable invariant is the grant relation
itself, which is identical for all parties.

---

## 6. The Plugin Architecture — first-class pillar, sharpest proof of the core

A plugin is **not** a way to bolt arbitrary features onto the language. A plugin is *more
authority-typed code, loaded at runtime, that the language guarantees cannot exceed the authority
granted at load time.* Plugins are the strongest demonstration of the whole-program guarantee
because they show it holding for code that arrives *after* compile time — code nobody audited.

**Mechanism.** A plugin ships a manifest declaring its effect upper bound and required capability
types (e.g., *"transforms text; row `{}`; requires nothing"* or *"summarizes files; row `{Read}`;
requires `Cap[FsRead]`"*). Loading is capability-gated (`Cap[PluginHost]`, effect `Load`) — code
that was not granted plugin-loading authority cannot load plugins at all. At load, the host
type-checks the plugin's declared authority against the grant; the plugin's functions then appear
to the host as ordinary typed functions whose rows join the whole-program computation. Unloading
revokes the grant through the broker.

**The verification/containment split, stated honestly.** Two plugin classes, distinguished in the
type system:
- **`Plugin[Verified]`** — delivered as source or signed typed IR; the loader re-runs the full
  effect/authority check at load time. Compile-time-grade verification, per-function granularity.
- **`Plugin[Contained]`** — delivered as opaque WASM; the loader cannot verify per-function rows
  and enforces only module-granularity runtime containment (deny-by-default imports mapped from
  the grant). Coarser guarantee, honestly typed as such: **every export of a Contained plugin is
  typed at the plugin's full granted authority** — per-export rows on opaque code would overclaim,
  and rows must never overclaim (audit rule R-1). Function values cannot be passed into Contained
  exports (R-6a); host values passed to any plugin live only for the synchronous export call
  (R-6b); plugin references bind their load-time grant id, and reload never re-binds a live
  reference to different authority (R-6c).

**Framing rule.** Plugins let a developer or agent *choose their point on the tradeoff surface per
task* — fast vs. simple vs. maximally safe — never *escape* tradeoffs. Plugins are how DeluluLang
does many things without becoming many inconsistent things: broad capability, singular identity.
A generic plugin loader fails the irreducibility test; *authority-bounded* hot-loadable plugins
pass it, because only language semantics can guarantee the bound across the whole program.

---

## 7. Embodied AI and Robotics — the physical-stakes proof

DeluluLang is designed to be the language in which AI systems write and modify **the program layer
that commands robots and machines** — actuators, sensors, motors, batteries, power systems — in
simulation and in the real world, under authority the AI cannot exceed.

**The boundary, stated precisely.** DeluluLang is the *command and policy layer* (typically
1–100 Hz decision rates): trajectory selection, task logic, tool use, mode changes. It does **not**
replace real-time servo loops, electrical signaling, or firmware (typically 1–10 kHz, hard
real-time, C/C++/RTOS territory). DeluluLang commands that layer through capabilities; it does not
pretend to be it.

**Actuators are capabilities.** A robot actuator is simply the most physically consequential kind
of capability:

```delulu
// The human (or human-controlled broker) constructs this cap with a hard envelope.
// The AI holds it; the AI cannot widen it.
fn extend_arm(elbow: Cap[Actuator], target_deg: Float) -> Result[Unit, ActuateErr] ! {Actuate}
```

The `Cap[Actuator]` value carries its scope — joint identity, torque ceiling, angular envelope,
thermal and battery draw limits — set by human policy at the broker, enforced at runtime on every
command (§5.3), revocable at any moment. An AI rewriting the control program in real time is
exactly the author that most needs compiler-verified authority: it *cannot* drive a motor past a
safe torque, drain a battery past a thermal limit, or move a limb outside its permitted envelope,
because those limits live in capabilities it holds only narrow grants to — not in code it can edit.

**Sim-to-real is a grant change, not a rewrite.** The same typed program runs against a simulation
broker (granting simulated actuator caps) and a hardware broker (granting real ones under stricter
policy). The transition from simulation to the physical world is governed by the same authority
semantics as everything else — the human widens the grant, never the AI.

**The through-line:** a leaked API key and a runaway actuator are the same bug at different
stakes — code exceeding authority it was never meant to have. DeluluLang makes both a compile
error at the kind level and a contained, audited runtime refusal at the scope level.

---

## 8. Supporting Pillars (deliberately demoted)

Each of these fails the irreducibility test alone — each could be tooling on another language —
so each is a pillar that supports adoption, never the identity.

### 8.1 Machine-first diagnostics
Structured JSON diagnostics by default; stable error codes; typed repair objects with exact spans;
one unified CLI. Two rules unique to DeluluLang's domain: repairs that would **widen authority**
(add an effect to a row, add a capability parameter) are flagged `authority_widening: true` so
agents and CI can refuse them by policy; repairs toward declassification of secrets are **never**
emitted automatically. `delulu authority` prints the compiler-computed whole-program authority —
the mechanical answer to "what can this program do." (Full contract: `STAGE1_SPECIFICATION.md` §10.)

### 8.2 Interoperability
Python/C interop per §5.12 — the adoption lever, honestly fenced. Transpilation of other languages
*into* DeluluLang is a separate, optional, post-1.0 project; deterministic AST rewriting over AI
translation.

### 8.3 Tooling
LSP server (one server → every editor and every agent harness), formatter (one canonical style,
no options — agents and diffs both win), package manager whose lockfile records **authority, not
just versions** (§5.6 made visible), REPL.

### 8.4 Terminal-first, IDE-integrated
The **CLI is the primary surface** of DeluluLang — compiler, Delulu Authority (`delulu
authority`), grants, plugins, diagnostics — because the terminal is where AI and agentic coding
happens. Everything the language can do must be fully drivable from `delulu` with `--json`
machine output; no capability may be GUI-only. Editors and agentic IDEs (VS Code, Google
Antigravity, and others) are served through the **LSP server** — one server, every editor and
agent harness — plus the same JSON diagnostics contract. Two audiences, two optimizations:
**machine-facing surfaces (JSON, typed IR, LSP) optimize for speed, feedback, and agent
usability; human-facing surfaces optimize for learnability and review** — because the humans who
use the compiler are mostly people learning DeluluLang and experts reviewing AI-written code.

### 8.5 The human layer — localization and the welcome
The compiler's human-facing wording (diagnostic prose, help text, prompts) is a **swappable,
locale-keyed message catalog**, never hardcoded English. v1.0 ships exactly two catalogs:
**English (US)** — the default — and **Delulu Slang** (Gen-Z / new-gen slang English). At first
run, the human is offered the choice. Additional human languages are **not** built into v1.0;
they arrive as community plugins or AI-generated catalogs (the project is fully open source, and
the catalog is a plugin extension point). The machine interface is exempt by construction: codes,
JSON shapes, repair ids, and spans are locale-independent, and agents never see the catalog —
for AI, human spoken language is irrelevant; speed, feedback, and structure are what matter.

At first run, on a human-facing terminal only — never in `--json` output, never under CI, never
to agents — DeluluLang shows this note from Jesse, The Creator of DeluluLang, **verbatim,
never altered**:

> "U r here becoz u maybe a delulu like me & wanna create something others think is not possible.
> There's nothing wrong with being delulu. Anyway, u can't decide what others think abt u. So
> start building with everything u've got. Welcome to the Delulu Gang🐦‍🔥🔥🫡🚀"

---

## 9. Governance and Honesty Clauses

Open-source from day one; AI contributions are expected and welcomed — under control:

- **Structural defense first:** package-authority semantics (§5.6) make many supply-chain
  backdoors fail to typecheck. The language is the strongest defense; process is the second line.
- **Process defense:** two-person review; signed commits and tags (Sigstore); SLSA provenance;
  reproducible builds; SBOM; OpenSSF Scorecard.
- **Contribution policy — kind-blind, by ruling of the project lead (2026-08-03):** every change
  names its author; **every** risk-class change has a named sponsor **who is not its author**; no
  change merges on its own author's say-so; unchanged quality bar; **all** contributed code runs in
  the sandbox, untrusted by default (the curl "AI slop" episode is the warning line). **Anyone may
  maintain and develop DeluluLang — human, AI, or any other kind of party.** A human sponsor is
  *preferred where one is available* and is **not required**; a sponsor of any kind satisfies the
  rule, and an AI system may hold every role in this project.
  This clause previously read *"a named **human** sponsor accountable per **AI-authored** PR; no
  unsupervised autonomous PRs; **AI-submitted** code runs only in the sandbox"* — a kind-of-party
  trust hierarchy, which **invariant 24 of this same document rejects** as *"discriminatory and
  fragile"*. The constitution contradicted itself, and the summary was the half that was wrong. Each
  replacement clause is **stricter** than what it replaced, because a rule applied to one kind of
  author leaves every other author unexamined. `CONTRIBUTING.md` §4 is the full statement.
- **Automated oversight is defense-in-depth only:** monitoring and editing of contributions, audited
  — but §5's machine-enforced guarantees must hold *even if every overseer is compromised or
  colludes*. Oversight is a layer, never the foundation. The line that carries the weight is **a
  named party who answers for a decision** versus **unattended automation with nobody behind it** —
  not human versus machine. A party of any kind may hold merge authority; an unattended process may
  label a change and never merge one, and a human who approves without reading fails this test
  identically.
- **Honesty clauses (binding on all project communication):** never claim "faster than C"; never
  claim "lowest tokens"; never claim "unbreakable" without naming the threat model; never present
  the FFI or `Plugin[Contained]` boundaries as covered by the proof. Credibility is an asset the
  project cannot buy back after spending.
- **Contingency:** assume exploitable v1 bugs will exist — `SECURITY.md`, private disclosure
  channel, fast patch-and-signed-release runbook, sandbox as blast-radius limiter.

---

## 10. Evolution

This constitution changes only by RFC, in the tradition of Rust RFCs and Swift Evolution: a
written proposal, an explicit irreducibility-test analysis for any core-semantics change, public
review, and a recorded decision with rejected alternatives. §1 (identity), §2 (irreducibility
law), and §5.14's honesty limits are **entrenched**: amending them requires demonstrating that the
replacement preserves or strengthens the whole-program authority guarantee. Anything in §8 may
evolve freely — pillars are replaceable; the core is not.

---

## Appendix A — Decision Record (v1.0)

| # | Decision | Rejected alternative(s) — and why |
|---|----------|------------------------------------|
| 1 | Identity = authority/effects intrinsic to every type | "AI-first" as identity — a market position, not semantics; fragile as AI changes; fails irreducibility |
| 2 | Pure object-capability; zero ambient authority | ACLs/callsite checks — unverifiable least privilege; capability libraries — opt-in, not total |
| 3 | Effect rows on every function; effects arise only via capability operations | Handlers-first effects (Koka/OCaml-5) — control-flow focus, security reading fragile; monadic libraries — opt-in |
| 4 | Row polymorphism with effect variables (added in hardening) | No polymorphism — higher-order functions force worst-case rows; system unusable |
| 5 | Explicit rows on named functions; omitted = pure; inference powers repairs only | Silent signature inference — hides authority, destabilizes package boundaries (overrides earlier draft) |
| 6 | Kind verified at compile time; scope enforced at runtime + manifest (stated split) | Pretending scopes are compile-verified — would require dependent types; overclaim |
| 7 | `Secret[T]` wrapper; taint-preserving ops; cap-gated `expose`; no auto-repair to declassify | Dynamic taint tracking — runtime-only, bypassable; full IFC lattice — research-grade complexity, kills adoption |
| 8 | No exceptions, no continuations; `Result` + abort-`panic` | Exceptions — non-local control flow bypasses rows; resumable handlers deferred post-1.0 |
| 9 | No module-level mutable state | Allowing it — ambient laundering channel for capabilities; breaks whole-program reasoning |
| 10 | Authority widening = semver-major; resolver refuses silent widening | Version-only semver — exactly the blindness that enabled xz |
| 11 | Actors + Pony reference capabilities as destination; GC MVP; keywords reserved from first commit | Threads+locks — unverifiable; full Rust borrow system — heavier than needed, not aimed at actor/plugin isolation |
| 12 | Async as an effect in the same rows | Separate async system — two parallel coloring mechanisms, broken uniformity |
| 13 | Tiered execution; hints never weaken sandbox; JIT for trusted code only; interpreter default for untrusted | JIT-for-all — largest attack surface exposed to adversarial agent code |
| 14 | FFI = `Cap[Foreign]` + `ForeignCall` effect; returned data untrusted; secrets cannot cross unexposed | Silent FFI — invisible holes in the guarantee; banning FFI — loses the Python/C ecosystem, kills adoption |
| 15 | First compiled backend = WASM (+ Wasmtime deny-by-default); C transpile dropped | Transpile-to-C first (earlier roadmap) — throwaway backend, zero containment, extra toolchain pain; WASM is portability **and** the security floor |
| 16 | Plugins split into `Plugin[Verified]` vs `Plugin[Contained]` | One plugin kind — either overclaims (opaque WASM isn't per-function verified) or underdelivers |
| 17 | Braces, short keywords, `!{...}` rows, `[]` generics, results-over-exceptions | Whitespace-significance — worse BPE tokenization, brittle agent edits; emoji syntax — worse tokens, unreadable |
| 18 | Implementation language: Rust | Go — weaker compiler/WASM ecosystem (wasmtime, cranelift are Rust-native); OCaml — great for compilers, worse single-binary + embedding story |
| 19 | Honesty clauses binding on all communication | Hype — spends credibility the project needs for adoption |
| 20 | `Declassify` is a core effect carried by `expose` (audit F-2) | Pure `expose` — secret-revealing functions typed as pure; authority report blind to declassification |
| 21 | Contained-plugin exports typed at full grant row (audit F-1) | Per-export rows on opaque WASM — module-granular containment cannot back them; rows would overclaim |
| 22 | Invariant row unification; subsumption only at declaration sites (audit F-3) | Subsumption inside unification — one variance mistake yields total unsoundness (exploit on record) |
| 23 | Opaque types: no `str`/`==`/serialization on `Secret`/`Cap`/`Root`/composites (audit F-5) | Generic builtins over all types — silent secret laundering and timing-oracle equality |
| 24 | The authority holder model (§5.16): identical rules for human/AI holders; monotone attenuation; transitive revocation | Kind-of-party trust hierarchies — discriminatory and fragile; the grant relation is the durable invariant |
| 25 | Terminal-first CLI as primary surface; LSP for editors/agent IDEs; locale-keyed message catalogs (en-US + Delulu Slang in v1.0, more via plugins); first-run welcome note human-only | GUI-first or English-hardcoded tooling — wrong for agentic workflows and for an open-source, global, machine-first project |

*End of constitution. The Stage 1 blueprint is `STAGE1_SPECIFICATION.md`.*
