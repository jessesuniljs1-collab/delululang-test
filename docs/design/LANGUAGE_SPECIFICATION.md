# The Language Specification & Constitution

> ## ⚠ STATUS: SUPERSEDED EARLY DRAFT — kept for provenance, not for planning
>
> **This is the pre-implementation vision document, written before Stage 1.** It is superseded by
> [`CONSTITUTION.md`](CONSTITUTION.md) (the normative core) and the ten
> `STAGE<N>_SPECIFICATION.md` files (what each stage is contractually required to do).
> `docs/REPOSITORY_STRUCTURE.md` has described it as superseded for some time; **the document
> itself did not say so until 2026-08-23**, so a reader who opened it directly met the line below
> calling it *"the definitive design specification"* and had no way to know otherwise. A document's
> status belongs in the document.
>
> **Read it for the arguments, not the inventory.** The reasoning here — the irreducibility test,
> why each rejected alternative was rejected, why this had to be a new language — is still the best
> statement of *why* DeluluLang exists, and none of it has been retracted. But its "Decision:"
> paragraphs describe an intended system, and several describe things that were later changed or
> were never built:
>
> - **§5.9, §5.11 (guarantee 5) and Part III name a Firecracker-class microVM** as the runtime
>   containment layer for untrusted code. **It is not built** — `--isolation microvm` is a probe
>   that refuses with `DL1408` on every host (see [`../REMAINING_WORK.md`](../REMAINING_WORK.md)
>   §4.1). The Stage 5 roadmap entry in Part IV says the same.
> - **§5.10 describes a tiered JIT** with profiling, on-stack replacement and deoptimization.
>   **None of it exists**; `@jit` is a leash (`DL1906`), and there is no native tier.
> - **§5.3 promises Pony-style reference capabilities** as the destination, with a GC MVP and the
>   rcaps as "the north-star the language migrates toward". All six *did* land in Stage 7 and work
>   today — this one came out ahead of the draft.
> - **§5.8 makes asynchrony an effect** tracked in the row. That held: `Async` is one of the ten
>   core effects. What did **not** hold is any `async`/`await` *surface syntax* — rejected outright
>   by Constitution decision 12, so the words stay reserved and unbuilt.
>
> Where this file and `CONSTITUTION.md` disagree, the constitution wins. Where the constitution and
> the code disagree, `REMAINING_WORK.md` records which.

*A production programming language defined by one durable, intrinsic property — with a staged path to build it. This is the definitive design specification, not a research exploration.*

---

## How to read this document

This is the consolidation of everything in this project: four deep-research passes, every design argument, and every correction. It commits to **one** technical direction. It does not hedge between alternatives. Where a previous draft was wrong, this one overrides it and says so.

It leads with the **constitution** — the philosophical foundation and the language semantics that justify the language's existence — and ends with a **condensed build roadmap**. The order is deliberate: you must know *what* you're building and *why it must be a new language* before you write a parser.

**The governing rule of this entire document**, applied to every major decision:

> **A new programming language is justified only if its defining property cannot realistically be implemented as a library, framework, compiler plugin, linter, or extension to an existing language.**

This is the *irreducibility test*. Most of this document is the work of taking that test seriously — including being honest about where features fail it and must be demoted, and where the core genuinely passes it.

---

# PART I — THE CONSTITUTION

## 1. The One Sentence

Every successful language has one sentence that drives every other decision. Rust: *memory safety without a garbage collector.* TypeScript: *JavaScript that catches type errors before runtime.* Go: *simple concurrency.* That sentence is not a feature list — it is the single intrinsic property everything else serves.

Here is ours, committed, not offered as a candidate among several:

> **A language where every unit of code — every function, every module, and every dynamically loaded plugin — carries its authority and effects in its type, so the powers of the entire program are verifiable by the compiler, and no code (human-written or AI-generated) can exceed the authority it was explicitly granted.**

The short form, for when you need one line:

> **Authority and effects are part of the type of every program — total, verifiable, and enforced by the compiler across all code and all dependencies.**

This is the identity of the language. Not "AI-first." Not "fastest." Not "fewest tokens." **Authority, made intrinsic.**

### Why this sentence and not the earlier ones

Earlier drafts of this project led with *"AI agents first."* That was a mistake, and this document overrides it. Here is the precise reasoning, because you should be able to defend the change:

"AI agents first" defines the language by **who uses it today**, and by **today's** AI limitations — repair loops, token efficiency, structured diagnostics. Languages live for decades. If those AI implementation details change in five years, an identity built on them becomes fragile. More damningly, "AI-first" is not a *language-semantic* property at all — it's a market position. It cannot survive the irreducibility test, because "be good for AI agents" can be pursued by *any* language through tooling.

**Authority is different.** It is a property of the language's semantics, it remains valuable no matter how capable AI becomes (a more powerful agent is a more dangerous one to grant unrestricted authority), and — as Part II proves — it is *irreducible*: it cannot be added to an existing language after the fact. AI agents remain the **primary user** of this language, because they are the population for whom unrestricted authority is most dangerous and most in need of compiler-enforced limits. But they are the *user*, not the *principle*. The principle is authority.

This was the hardest correction in the entire project, and it came from adversarial review. It is the right one.

---

## 2. The Irreducibility Test (the law of the language)

Before any feature enters the core language, it must pass this test:

> **Removing this feature, or moving it into a library, fundamentally changes what the language *guarantees*. If a library or tool could deliver the same guarantee, the feature does not belong in the core.**

This test is the reason most of what felt important in earlier drafts gets **demoted** in this one. Structured JSON diagnostics, an LSP server, transpilation, interoperability, a formatter, a package manager — every one of these is genuinely valuable, and every one of them **fails** the irreducibility test, because each can be built as tooling on top of any language. They are *pillars that support adoption*. They are not the *identity*. Part IV covers them in their proper, demoted place.

A subtle but decisive point about how the test must be applied — because applied naively it is *too strong and would be wrong*:

**Almost any single feature, in isolation, can be approximated by a library.** Capabilities? There are capability libraries. Effect tracking? There are effect-system libraries and monadic frameworks. A plugin loader? That's just a framework. If you apply the test feature-by-feature in isolation, you would reject *everything* and conclude no language is ever justified — which is absurd, since languages plainly do get justified.

The test must be applied to the **guarantee across the whole program**, not to features in isolation. And there is exactly one thing that cannot be a library: **the guarantee that the property holds for *all* code in the program — including code the library author never saw, including dependencies, including plugins loaded at runtime.** A capability *library* protects only the code that opts into it; the `unsafe` filesystem call three dependencies deep still happens, because that dependency wasn't written against your library. For the guarantee to be **total** — for "no code anywhere in this program can exceed its granted authority" to actually be *true* — every function, every module, and every plugin must carry authority and effects in its type *from the standard library upward*. That is a property of the language's semantics. It cannot be retrofitted, because the existing ecosystems of existing languages are already written without it.

That is the whole argument for why this must be a new language. Part II makes it rigorous.

---

## 3. Why This Must Be A New Programming Language

This is the chapter that most projects skip and most projects die for skipping. The question is not "is my language nice?" The question is: **why couldn't this simply be added to Rust, Go, or TypeScript?** If the honest answer is "it could," a new language faces a near-hopeless adoption climb and should not be built.

For each existing language and system, the question is the same: *can the core guarantee — total, whole-program, dependency-inclusive authority/effect verification — be added to it?*

### Against Rust
Rust has an ownership and borrow system, and it is the closest existing language in spirit. Could you add whole-program effect/authority typing to Rust as a compiler extension or framework? **No — and the reason is decisive.** Rust's entire existing ecosystem (`std`, and hundreds of thousands of crates) is written **without** effect typing. `std::fs::File::open` does not declare "I perform a filesystem effect" in its type; it just does I/O. To make the guarantee *total*, you would have to re-type the entire standard library and every transitive dependency — which means you are no longer extending Rust, you are building a new language that happens to look like Rust. Rust also deliberately has `unsafe`, an escape hatch that punctures any whole-program guarantee by design. A capability *framework* on Rust (and these exist — e.g., the `cap-std` family) gets you *opt-in* capability discipline for the code that uses it, which is genuinely useful and is ~80% of the value *for code you control* — but it is **not** the total guarantee, because the moment one dependency doesn't opt in, the guarantee is void. **The total guarantee requires language semantics from the ground up. Rust cannot provide it retroactively. This is the crux.**

### Against Go
Go's defining strength is simplicity and goroutine-based concurrency. It has no effect system, no capability model, and a culture of pragmatic, unrestricted standard-library access. Adding total effect typing to Go would contradict Go's core design value (simplicity through *omission* of such systems) and would, again, require re-typing the whole ecosystem. **No.**

### Against TypeScript
TypeScript is a structural type layer *over* JavaScript, and it is explicitly **unsound by design** (it has `any`, type assertions, and erases types at runtime so nothing is *enforced* during execution). It is the proof-by-counterexample for our project: TypeScript's types are *advisory*, checked at compile time and gone at runtime. Our core guarantee must be *enforced*, including at the boundary where code actually runs. A capability discipline expressed in TypeScript types could be bypassed by any `any` or any plain-JavaScript dependency. **No — unsoundness is fatal to a security guarantee.**

### Against Python
Python is dynamically typed, with `eval`, monkey-patching, and reflection — the opposite of statically verifiable authority. It is, however, our most important **interop target** (Part IV), because its ecosystem is the AI world's ecosystem. But add a total authority guarantee *to* Python? **No** — its dynamism makes whole-program static verification impossible in principle.

### Against Zig
Zig offers manual control, `comptime` metaprogramming, and excellent C interop, with no GC. It has no effect or capability system and embraces low-level unrestricted access. It is a fine language for a different goal. It cannot provide the total guarantee without becoming a different language. **No.**

### Against Mojo
Mojo (built by the creator of LLVM and Swift, on MLIR) is the closest "Python-simple but fast" effort, and it targets the performance of C++/Rust/Zig. Its goal is *performance and Python-superset ergonomics*, not authority verification. It has no whole-program effect system as its core. Different identity. **No.**

### Against Zero (vercel-labs/zero)
This is the most important comparison, because Zero (May 2026) is the nearest existing thing to "agent-native language" and *does* have capability-based I/O (its `World` handle) and machine-first diagnostics. Two honest points. First: Zero **validates** the direction — capability-passing I/O and effects-in-signatures are exactly right, and Zero proves serious people see the same gap. Second: Zero's headline is **agent ergonomics and machine-readable tooling** (structured JSON diagnostics, typed repairs), with capabilities as one feature among several; it is also explicitly experimental, pre-1.0, with an immature borrow checker. **Our identity is different and deeper:** we make *total, whole-program, dependency-inclusive authority verification* the **defining** property, not one feature. Zero is the closest cousin and the best language to learn from — but its center of gravity is tooling-for-agents, and ours is authority-as-semantics. (And critically: Zero's structured-diagnostics headline is itself a feature that *fails* our irreducibility test — it could be tooling on any language — which is exactly why we demote it and Zero, arguably, should not lead with it either.)

### Against existing capability systems (E, Joe-E, Pony, KeyKOS/seL4)
These are the intellectual ancestors and they are excellent. The object-capability model (Dennis & Van Horn 1966; Mark Miller 2006) is our foundation. Pony's reference-capability type system (`iso`, `val`, `ref`, `box`, `tag`, `trn`) statically guarantees data-race freedom and is the closest existing *language-level* capability discipline. So why not just use Pony or revive E? Because none of them unifies **(a)** total authority/effect typing, **(b)** *runtime-loadable plugins that are themselves authority-verified*, and **(c)** first-class interop with the Python/C ecosystem that the AI world actually runs on. Pony is the closest, and we borrow its reference-capability mechanics directly — but Pony was not designed around hot-loadable, authority-bounded plugins or around being the agent's working surface. We stand on these systems; we are not redundant with them.

### Against effect systems (Koka, Eff, Unison, OCaml 5 effect handlers)
Algebraic effect systems (Koka, Eff, the Unison language, and OCaml 5's effect handlers) are the other ancestor, and they are where our effect typing comes from intellectually. They are mostly research or niche languages focused on effects as a *control-flow* and *composability* mechanism (resumable continuations, handlers), not as a *security/authority* boundary enforced across an entire dependency graph including untrusted plugins. We take their type-level effect tracking and aim it at authority and security, across the whole program. Could you add OCaml-5-style effects to an existing mainstream language? The handlers, maybe, as a feature — but **not the total, security-grade, whole-program guarantee**, for the same ecosystem reason as Rust. **No.**

### Against WASI and WebAssembly
This is the sharpest "why not a framework?" challenge, because WASM+WASI **already** gives capability-based, deny-by-default sandboxing: a module starts with zero ambient authority and can only do what the host grants. So why not just compile any language to WASM and call it done? Three reasons, and they define our relationship to WASM. **(1) Granularity:** WASI's boundary is the *module*. It tells you "this whole module may touch this preopened directory." It cannot tell you "this *function* performs a network effect but that one does not," which is what whole-program reasoning and fine-grained least privilege require. Our effect types are **per-function**; WASI is per-module. **(2) Verification vs. containment:** WASM *contains* a module at runtime but does not *prove*, from the source, what authority each part of the program needs — and a memory-unsafe program compiled to WASM can still corrupt *itself* within its sandbox (USENIX Security 2020). We want compile-time *verification* of the authority graph, not only runtime *containment*. **(3) WASM is our enforcement floor, not our identity.** We **use** WASM/WASI as the runtime sandbox layer (Part III, security semantics) — it is the best available isolation substrate. But "compile to WASM" is a *backend and enforcement* decision; it is not the language. The language is the per-function, whole-program authority typing that *targets* WASM for enforcement. WASM is the jail; our type system is the law that says who was ever allowed near it.

### The verdict
The core guarantee — *total, whole-program, per-function, dependency-and-plugin-inclusive authority/effect verification, enforced at compile time and contained at runtime* — **cannot** be realistically delivered as a library, plugin, linter, framework, or extension to any existing language, because every existing language's ecosystem is already written without it, and retrofitting it means re-typing everything, which means building a new language. **That is the justification. It is the only justification a new language ever needs, and we have it.**

---

## 4. What Becomes Possible That Existing Languages Cannot Do

The irreducibility test's positive twin. Not "what becomes nicer" — convenience improvements are forbidden from this chapter. Only **what becomes possible that requires language semantics.**

### Possibility 1 — Provable least privilege for an entire program, including its dependencies
Today, you cannot answer the question "what can this program, in total, actually do to my system?" without auditing every line of every dependency by hand — and even then you can be wrong, because a dependency can do anything its language permits. In this language, the answer is **mechanical and total**: the compiler computes the union of all effects and authorities across the whole dependency graph and reports it. "This program can read `./data`, make HTTPS calls to `api.example.com`, and nothing else" becomes a *compiler-verified fact*, not a hopeful audit. **This is impossible in Rust/Go/TS/Python** because their dependencies aren't effect-typed. It requires the property to be intrinsic.

### Possibility 2 — Untrusted, hot-loadable plugins that provably cannot exceed their grant
This is where your plugin vision becomes the **proof** of the language, not a convenience. In every existing language, loading a plugin at runtime (a `.so`, a Python module, a JS package) means **trusting it completely** — once loaded, it runs with the host's full authority. There is no language in mainstream use where you can load a plugin you did not write, *grant it a specific bounded authority* ("you may transform text and nothing else — no I/O, no network, no clock"), and have the **language guarantee** it cannot exceed that, even as it runs. WASI gets close at module granularity but cannot express per-function effects or compose plugins into a verified whole-program graph. In our language, a plugin is just *more code carrying its effects in its type* — so loading one, granting it authority, and verifying it cannot escape that grant is the **same mechanism** as the rest of the language. **Plugins that are simultaneously dynamic and authority-bounded are a genuinely new capability**, and they exist *because* authority is intrinsic. (This is why your plugin idea survives the irreducibility test where a generic "plugin framework" would not: a framework can load plugins, but only language semantics can *guarantee* their authority across the whole program.)

### Possibility 3 — Authority as a first-class, composable value
Because authority lives in the type system, you can *reason about it, pass it, attenuate it, and revoke it* as a normal part of the language — "take this filesystem authority and produce a weaker one that can only read, not write, and only this subdirectory," verified by the compiler. This composition of *diminishing* authority (attenuation) is an ocap idea that, made intrinsic and total, lets you build large systems where each component provably holds only the slice of authority it needs — across the whole graph. Libraries can offer attenuation *for code that uses them*; only the language can make it *total*.

### Possibility 4 — A trustworthy substrate for autonomous and multi-agent software
The durable, AI-relevant payoff — stated as a *consequence* of the semantics, not as the identity. When agents write and run code autonomously in loops, the unsolved problem is not "can they write code" — it's "can we let them run it without unbounded risk." A language where *any* code an agent emits or loads is **automatically bounded by compiler-verified authority** turns "audit and pray" into "the language won't permit it." For multi-agent systems, each agent's code can be granted a distinct authority slice, and their composition remains verifiable. This is valuable *today* for agents and *more* valuable as agents grow more capable — which is exactly the durability test the identity had to pass. It is a consequence of authority-as-semantics, available to humans and AI alike.

Everything else this language does well — fast feedback, good errors, interop — is **convenience**, and convenience is not in this chapter. These four are the things that *cannot be done* without the language existing.

---

## 5. Language Semantics — the formal core

This is the constitution proper: the precise semantic commitments that define the language. Each is stated as a decision, with rejected alternatives and why. These are the parts you must design carefully and **must not delegate to an AI to invent**, because soundness lives here and AI is least reliable here.

### 5.1 Authority model
**Decision:** Pure object-capability. There is **no ambient authority** anywhere in the language. A capability is an unforgeable value that both designates a resource and confers the authority to use it. Code can only affect a resource if it holds a capability for it, and capabilities propagate **only** by explicit passing ("only connectivity begets connectivity"). The top-level entry point receives the root capabilities from the runtime (controlled by the human); everything else receives only what is passed down.
**Rejected:** ACL/permission-checks-at-callsite (the Unix model) — rejected because ambient authority is the thing that makes least privilege unverifiable; if any code *can* ask for any resource, you cannot prove what it won't do. Capability-as-library (opt-in) — rejected because it isn't total (§3).

### 5.2 Effect system
**Decision:** Every function's type includes its **effect row** — the set of effects it may perform (e.g., `Read`, `Write`, `Net`, `Clock`, `Rand`, `Alloc`, plus user-declared effects). A function with an empty effect row is provably pure. Effects are **inferred** where possible (so the surface stays ergonomic) and **checked** always. Effects compose: a caller's effect row is at least the union of its callees'. Performing an effect requires holding the corresponding capability — **effects and capabilities are two views of the same authority**: the effect type says *what kind* of thing the code does; the capability value says *it is permitted to*. The compiler verifies they agree across the whole program.
**Rejected:** Effects purely as control-flow/handlers (Koka/OCaml-5 style) without the security binding — rejected because our purpose is authority, not just composability; we adopt their *type-level tracking* but bind it to capabilities. Monadic effects as a library (Haskell-style) — rejected because it's opt-in and doesn't cover untyped dependencies, failing totality.

### 5.3 Ownership / capability interaction
**Decision:** Adopt **Pony-style reference capabilities** for memory and concurrency safety (`iso`, `val`, `ref`, `box`, `tag`, `trn`), unifying memory-safety guarantees with the authority model so the type system delivers *both* "no data races / no use-after-free" *and* "no unauthorized effects" in one coherent system. Reference capabilities govern *aliasing and mutation*; object capabilities govern *external authority*; they share the principle that the type controls what you may do with a reference.
**Rejected:** A garbage collector with no ownership discipline (Go/Java style) — rejected for v-next because it weakens the static guarantees and the data-race freedom we want; **however**, see the roadmap honesty note: the *MVP* may use GC for tractability, with reference capabilities as the north-star the language migrates toward. Manual memory (C/Zig) — rejected as unsafe and hostile to the guarantee. Full Rust borrow system — rejected as more complex than needed and not designed around actor/plugin isolation; Pony's model is closer to our plugin-isolation goal.

### 5.4 Module semantics
**Decision:** A module is a unit of authority. A module declares the effects it may perform and the capabilities it requires; it cannot acquire authority it did not receive. Modules are **opaque by default** — no implicit global state, no ambient imports that carry authority. This makes a module's maximum authority readable from its interface.
**Rejected:** Modules as mere namespaces (most languages) — rejected because if a module can reach ambient authority, module boundaries don't bound authority, and whole-program reasoning breaks.

### 5.5 Package semantics
**Decision:** A package's **total authority is declared and compiler-verified** at its boundary. When you depend on a package, you can see — and the compiler enforces — the maximum set of effects/capabilities that package can ever require. A package cannot silently start doing I/O in a patch release; that would change its declared authority and fail verification. This is the direct structural defense against supply-chain attacks (the xz lesson, §III): a backdoor that performs unexpected effects **cannot typecheck** under the package's declared authority.
**Rejected:** Packages with unrestricted ambient capability (npm/PyPI/crates model) — rejected because it is precisely the model that makes supply-chain compromise invisible; a dependency doing something it shouldn't is, today, indistinguishable at the type level from one behaving. Here it is a compile error.

### 5.6 Import semantics
**Decision:** Importing a module grants **access to its interface, not authority to act**. Authority still flows only by explicit capability passing at runtime. Importing `net` does not let you make network calls; you must also be *handed* a network capability. This separates *"I can refer to this"* from *"I am permitted to do this."*
**Rejected:** Import-confers-power (Python `import os; os.system(...)`) — rejected as the canonical ambient-authority footgun.

### 5.7 Concurrency semantics
**Decision:** **Actor model** with reference-capability-enforced isolation (Pony's proven design): actors communicate by message passing; the reference-capability system statically guarantees **no shared mutable state across actors**, hence **no data races, at compile time**. Each actor can hold its own authority slice — a natural fit for multi-agent software (§4, Possibility 4) and for plugin isolation.
**Rejected:** Shared-memory threads with locks (C/Java) — rejected as unsafe-by-default and impossible to verify free of races. Go's CSP/goroutines+channels — closer, and respected, but does not *statically* prevent shared-mutable-state races the way reference capabilities do; we want the stronger static guarantee.

### 5.8 Async semantics
**Decision:** Asynchrony is an **effect**, tracked in the type like any other (aligning with the effect system, §5.2), with `async` operations requiring the appropriate capability/effect and composing through the same effect rows. This avoids the "function coloring" problem becoming a *separate* parallel system — async is just one more effect in the row.
**Rejected:** A bolt-on async runtime with its own incompatible typing (the historical JS/Python async retrofit) — rejected because two parallel effect-like systems (async-ness and effects) is needless complexity and breaks uniform whole-program reasoning. Make async an effect and there is one system.

### 5.9 Security semantics
**Decision:** Defense-in-depth with the type system as the **first** line, not the only one. (1) **Compile-time:** the authority/effect system proves least privilege and rejects code that exceeds its grant. (2) **Runtime enforcement floor:** compile to **WASM/WASI** and run in a capability-deny-by-default host (Wasmtime), so even a compiler or type-system bug is contained; for untrusted execution, nest in a **Firecracker-class microVM** with default-deny egress, scoped short-lived credentials, read-only mounts, and explicit lifetimes. (3) **The human holds the keys:** root capabilities are issued by a human-controlled broker outside any agent's reach; agents receive only narrow, revocable, audited grants and cannot even *see* capabilities they were not granted.
**Honest limit (stated as semantics, because it must never be forgotten):** "unbreakable" is always relative to a threat model. The type system trusts the compiler; the compiler trusts the hardware; the sandbox trusts the hypervisor; *all* of it is exposed to microarchitectural side channels (Spectre-class attacks cross WASM's boundary on almost all CPUs). The language **eliminates whole classes of vulnerability by construction** (unauthorized effects become compile errors) — it does **not** "find all vulnerabilities," which is undecidable. Every guarantee names its assumptions.
**Rejected:** Type-system-only security (no runtime containment) — rejected because a single soundness bug would be catastrophic with no backstop; defense-in-depth is non-negotiable. Runtime-only security (sandbox without the type system, i.e., "just use WASI") — rejected because it gives containment without verification or per-function granularity (§3, vs WASI).

### 5.10 Compilation semantics
**Decision:** The surface is a **high-level language**; performance comes from the **backend**, never from compromising the surface. Mixed-mode execution by the **standard tiered-JIT model** (interpret cold code, compile hot paths via runtime profiling and on-stack replacement, deoptimize when speculation fails) — this is your "some code compiled, some interpreted, switched automatically" idea, and it is the *normal* architecture of HotSpot/V8/PyPy/Julia, not a novelty. **Execution-mode hints** (`@aot`, `@interpret`) may be set by developer or agent, with two inviolable rules: mode selection **cannot weaken the sandbox** (a JIT must emit the same authority/bounds checks), and only **human-controlled policy** may grant a module the right to emit native code. Because a JIT is a large attack surface (~45% of V8's recent CVEs are JIT-related), the **default for untrusted agent code is interpreter/AOT-verified**, with JIT reserved for trusted workloads.
**Rejected:** "Faster than C" as a goal — **rejected outright as false.** C sits near the hardware overhead floor; transpiled/bytecode execution is a layer *on top of* a C/Rust backend and cannot generally beat it. The honest, committed claim is **"competitive with C on hot paths, with safety C cannot offer."** Whitespace-significant syntax — rejected (tokenizes worse for LLMs, harder for agents to edit); **braces, short keywords, explicit effects in signatures, results-over-exceptions.** Emoji/exotic-symbol syntax for token savings — rejected (it *increases* BPE token counts and destroys readability). "Lowest tokens of any language" — rejected as a headline (tokenizer-specific, no universal metric); token-efficiency kept only as a minor consequence of clean syntax.

### 5.11 Runtime guarantees
**Decision:** The runtime guarantees, *relative to the stated threat model*: (1) no code executes an effect for which it does not hold a capability; (2) no actor accesses another actor's mutable state; (3) a plugin cannot exceed the authority granted at load time; (4) the union of program authority is bounded by what the human granted at the root; (5) violations are *contained* at the WASM/microVM layer even if the type system is bypassed. The runtime also provides the boring necessities: memory management (GC in the MVP, migrating toward reference-capability discipline), dynamic dispatch, the FFI bridge, and the capability broker.
**Rejected:** Guaranteeing absolute security (impossible — §5.9 honesty). Guaranteeing performance parity with C (impossible — §5.10).

---

# PART II — SUPPORTING PILLARS (deliberately demoted)

These are valuable. None is the identity. Each **fails** the irreducibility test on its own — each could be built as tooling on another language — and so each is a *pillar that supports adoption*, not a defining property. They are demoted on purpose. Naming them honestly as secondary is part of the discipline.

## 6. The plugin/algorithm system (the pillar closest to the core)

Your plugin idea — algorithms/plugins added and removed, automatically or by agent/developer choice, to make the language do different things — is kept as a **major pillar**, and it is the pillar that most directly *demonstrates* the core. The honest framing, holding the irreducibility line:

- **A generic "plugin system" is just a framework** — it fails the irreducibility test, and *any* language can load plugins. So a plugin system alone does **not** justify a new language.
- **What justifies it** is §4 Possibility 2: plugins that are **dynamically loadable yet authority-bounded by the language's semantics** — load an untrusted plugin, grant it a specific bounded authority, and have the *compiler/runtime guarantee* it cannot exceed that. *That* property is irreducible, and it exists only because authority is intrinsic. So plugins are a pillar, but they ride on the core; they are the core's most compelling *application*, not an independent feature.
- **Mechanism:** a plugin is ordinary code carrying its effects in its type. Loading it means type-checking its declared authority against the grant you give it; running it means the same WASM/microVM containment as everything else. Add/remove at runtime is loading/unloading authority-typed modules. "Choose plugins per task" = composing authority-typed modules into the verified whole-program graph.
- **Reframed from earlier drafts:** plugins do **not** let you "have everything" (fast *and* simple *and* low-token *and* safe at once). Tradeoffs are real; plugins let the developer/agent *choose their point* on the tradeoff surface per task. "Choose your tradeoff," never "escape it."

## 7. Machine-first diagnostics (demoted from former headline)
Structured JSON diagnostics by default, stable error codes, typed repair objects, exact spans, a single unified CLI — genuinely excellent for agents, and the right design. **But it is tooling, not identity** (it could be added to any compiler — it fails irreducibility), so it is a pillar. The honest correction to earlier drafts: diagnostics make the agent loop *faster*; they are not why anyone adopts a *language*. Use JSON (every agent parses it; a binary format buys nothing); the value is *machine-actionable* errors, not the wire format. You cannot eliminate syntax errors — you make every error *precise and auto-repairable*.

## 8. Interoperability (a pillar, and the adoption lever)
First-class interop with **Python and C** via FFI and **embedding CPython** (PyO3 path) — call NumPy/PyTorch unchanged, inheriting the AI world's ecosystem on day one. This is the single most important *adoption* lever (interop is how Kotlin/TS/C++ won). **But it is a pillar, not the identity** — interop is implementable in many languages. *Honest tension to design around:* code called over FFI into untyped C/Python is **outside** the effect guarantee; the boundary must be modeled as an explicit, capability-gated effect (`ForeignCall`) so the whole-program guarantee degrades *gracefully and visibly* rather than silently. Transpilation (translating other languages *into* this one) is a **separate, optional, harder** sub-project; deterministic AST-rewriting beats AI translation, and dynamic Python is lossy.

## 9. Portability, LSP, formatter, package manager (pillars)
Portability via WASM (which doubles as the security floor — a genuine two-for-one) or native cross-compilation; an **LSP server** (one server → every editor *and* agentic tools like Claude Code at once); a formatter; a package manager that surfaces declared authority. All valuable, all tooling, all pillars. The package manager is the most core-adjacent, because it *exposes* the package-authority semantics (§5.5) — but the *guarantee* is in the language; the manager just displays it.

---

# PART III — SECURITY & GOVERNANCE (condensed; full detail in the uploaded security report)

The defense-in-depth stack (type system → WASM/WASI → Firecracker microVM → human-held capability keys) is specified in §5.9. The supply-chain and governance posture, because you will open-source this and *will* receive AI-submitted contributions:

- **Structural defense first:** §5.5 package-authority semantics make many supply-chain backdoors *fail to typecheck* — the language itself is the strongest defense (the xz backdoor, CVE-2024-3094, performed effects it would not have been authorized for).
- **Process defense:** two-person review; signed commits/tags (Sigstore/Cosign); SLSA L3 provenance; reproducible builds; SBOM; OpenSSF Scorecard.
- **Contribution policy — kind-blind (project lead's ruling, 2026-08-03):** **anyone may maintain and develop DeluluLang — human, AI, or any other kind of party.** Every change names its author; every risk-class change has a named sponsor **who is not its author** (a human sponsor is *preferred where available*, never required); no change merges on its own author's say-so; unchanged quality bar; **all** contributed code runs inside the sandbox, untrusted-by-default, provenance required. (The curl "AI slop" crisis — ~20% slop, bounty ended Jan 2026 — is the warning line, and it is a warning about unreviewed changes, not about who wrote them.)
- **AI overseeing AI** as *defense-in-depth only:* trusted-monitoring/trusted-editing of agent contributions (measured ~62%/~92% detection in the Redwood AI-control work) with human audit — but the type/runtime enforcement (§5) must hold *even if every AI overseer is compromised or colludes*. Oversight is a layer, never the foundation. (The Amazon-VP "humans are inconsistent" argument cuts toward *machine-enforced* guarantees — exactly what §5 provides — not toward trusting agents.)
- **Contingency for v1 bugs:** assume exploitable bugs will exist; `SECURITY.md`, a private disclosure channel, a fast patch-and-signed-release runbook, and the sandbox as blast-radius limiter.

---

# PART IV — THE CONDENSED BUILD ROADMAP

You committed to building the ambitious effect-typed core **via a staged path that still ships an MVP first**. This is the honest reconciliation of "most justified" (the from-the-ground-up effect language) with "actually buildable by a solo student with AI" (ship something real at every stage). The north star is the constitution above; the path is pragmatic.

**The crucial sequencing decision, stated honestly:** the *full* whole-program effect/authority guarantee with reference capabilities is the **destination**, not Stage 1. Building it all at once is the multi-year version. Instead, each stage ships something usable while moving toward the destination — and critically, **the authority/effect skeleton goes in from the very first stage**, because it is the one thing that cannot be retrofitted. You add *depth* of the guarantee over stages; you never add the guarantee *late*.

### Stage 0 — Answer the irreducibility question for *your* specific design, and learn the craft (2–6 weeks)
Before code: write the precise version of "what becomes possible, intrinsic to my language, that fails the irreducibility test for every alternative" — Part I is the template; make it concrete and, ideally, *measurable* (e.g., "verified whole-program authority across N dependencies, which Rust+cap-std cannot total"). **This deliverable outranks parser, type system, and GC.** In parallel: work through *Crafting Interpreters* (build jlox fully, begin clox) and Thorsten Ball's *Writing an Interpreter/Compiler in Go*. Type the code yourself. Write your **threat model** (the template is in the uploaded security report).
**Ships:** the justification document + a working tree-walking interpreter for a toy language + a written threat model.

### Stage 1 — Minimal language with the authority skeleton from day one (2–4 weeks)
Design the tiny language: braces, short keywords, functions, `if`/`while`. **Effects in function signatures and capability-passing from the first commit** — even if the initial effect set is tiny (`IO` vs pure). This is the non-negotiable: the skeleton of the irreducible property is present immediately.
**Ships:** a REPL where a pure function provably cannot do I/O and an I/O function must be handed a capability — running `fib(10)`. The "I built a language *and* its core guarantee is visible" moment.

### Stage 2 — Formal grammar + the effect/authority type checker (4–8 weeks)
Write the grammar in EBNF. Build the type checker that enforces effects and capability-passing. **You own this stage** — soundness lives here; do not let an AI invent it. Start with a small effect row; design it to grow.
**Ships:** the compiler rejects unauthorized effects and type errors, with structured diagnostics.

### Stage 3 — First backend: transpile to C (or TypeScript) (1–3 months)
Transpile to a proven target so you ship runnable, portable, native-speed-via-the-target code fast. Defer WASM/LLVM until the language works. (Honest note: at this stage the *guarantee* is compile-time; runtime containment comes in Stage 5.)
**Ships:** `hello world` → a small CLI tool that compiles and runs cross-platform.

### Stage 4 — Interop: FFI + embed Python (3–6 weeks)
C-ABI FFI, then embed CPython (PyO3). Model the FFI boundary as an explicit `ForeignCall` effect so the guarantee degrades *visibly* (§8). Call NumPy from your language.
**Ships:** your headline adoption lever — the Python/C ecosystem, usable, with the foreign boundary honestly typed.

### Stage 5 — The runtime enforcement floor: WASM/WASI + microVM + capability broker (1–2 months)
Add the WASM/WASI backend (portability **and** the security floor — two-for-one). Run agent code in Wasmtime, deny-by-default; nest untrusted execution in a Firecracker microVM with default-deny egress. Build the **human-controlled capability broker** (agents never hold root keys).
**Ships:** defense-in-depth is real — agent code is *contained* even if the type system is bypassed; the human controls all authority.

### Stage 6 — Authority-bounded hot-loadable plugins (1–2 months)
Now the plugin pillar pays off: load a module at runtime, grant it bounded authority, verify (compile-time) and contain (runtime) that it cannot exceed the grant. Add/remove at runtime. This is §4 Possibility 2, shipped — the most compelling demonstration of why the language exists.
**Ships:** a plugin you did not write, loaded live, provably unable to exceed "transform text, no I/O." The demo that sells the language.

### Stage 7 — Concurrency + reference capabilities (deepening the guarantee) (2–4 months)
Introduce the actor model and Pony-style reference capabilities, migrating memory management from MVP-GC toward static reference-capability discipline. This deepens the guarantee toward the full constitution (§5.3, §5.7) and enables multi-agent isolation (§4 Possibility 4).
**Ships:** statically race-free concurrency; per-actor authority slices.

### Stage 8 — Tooling, LSP, and open-source hardening (ongoing, formalized here)
LSP server (every editor + agentic tools at once), formatter, package manager that surfaces declared authority, `AGENTS.md`, and the full governance stack (Part III): two-person review, signed commits, SLSA L3, reproducible builds, OpenSSF Scorecard, the AI-contribution policy, `SECURITY.md`, patch runbook, and AI-overseer monitoring as defense-in-depth.
**Ships:** a language pleasant for humans, consumable by agents, and *safe to accept contributions from both* — the precondition for community scaling.

### Stage 9 — Stabilize, measure, open to community
Lock a stable core; write real docs and a tutorial; **publish the measurable proof** — verified whole-program authority across real dependency graphs, and agent task-success/repair-iteration metrics vs. Python/Go. *Then* invite the community to compound it. Community is the amplifier that comes *after* a stable, genuinely-irreducible core — never a substitute for it.
**Ships:** a deployable v1.0 whose defining guarantee is real, measured, and demonstrably impossible to retrofit onto existing languages.

**Honest effort summary:** Stages 0–3 (a usable, effect-typed-at-the-surface language that transpiles and runs) are a **months-long** solo effort and entirely achievable with AI assistance. Stages 4–6 (interop, sandbox, authority-bounded plugins) are the heart and another several months. Stages 7–9 (full reference-capability concurrency, tooling, community) push into a **multi-year** arc — which is fine, because every stage before it already shipped something real and the *guarantee* was present from Stage 1. The AI era compresses this timeline meaningfully; it does not delete the stages. You ship continuously toward a destination that is justified because it cannot be built any other way.

---

## How to work with Claude Code on this (carry-over, still true)
Let AI write scaffolding (lexer tables, AST node families, recursive-descent skeletons, **tests**); **you own the grammar, the type system, the effect/authority semantics, and the soundness** — AI is trained on code that *compiles*, so it is weakest at *rejecting* invalid programs, which is exactly your type checker's job (the parallel-Claudes C-compiler experiment's blind spot was precisely missing semantic checks). Keep files small (agents lose attention on long files); own your test suite as your primary control; when you find yourself fighting the AI on type/effect/capability code, **stop and hand-write it** — that is the predicted failure zone, and it is also where your entire value proposition lives.

---

## Closing — the through-line from your first message to here

You started with "I hate bugs and current languages weren't built for the AI era." Across this whole project that sharpened, through honest adversarial pressure, into something defensible: not "a faster/simpler/lower-token language" (those goals conflict or fail the irreducibility test), but **a language whose one intrinsic, irreducible property is that authority and effects are part of every program — total, verifiable, enforced — which no existing language can retrofit, and which makes provably-bounded autonomous and plugin-based software possible for the first time.** Your plugin vision survived as the sharpest *proof* of that property. AI agents remain the primary user. The build path is staged and real. The justification is the one a new language actually needs: *it cannot be a library.*

That is worth putting your heart into. Now go write Stage 1 — and put the authority skeleton in from the first commit, because that one thing is the only thing you can never add later.
