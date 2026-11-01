---
name: two-engines-agreeing
description: "LATE-LET-1 (2026-11-01) — both engines read a name a later `let` bound inside a closure; parity gates were green because the engines agreed; assert the checker's VALUE, not engine-vs-engine"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-31T22:53:33.836Z
---

Two engines that agree prove only that they agree. LATE-LET-1 (D-V2-199, RW 2.23): a `let` is not recursive and the
checker resolves a closure's names where it is written, but the interpreter rebound the name in the scope the closure
held and the WASM engine inserted it into the frame the inlined body reads by index. A function `authority` called
"provably pure" printed; a `Secret.map` callback printed a plaintext secret with no Declassify — C88's class. Every
parity test and the 50,000-program differential stayed green because both engines were wrong the same way.

**Why:** found by reading the interpreter's scopes (for RW 3.24), not by any gate; the gates compared implementations
with each other, never with the checker's meaning.

**How to apply:** when a runtime behavior has a static meaning, the witness asserts the value the checker's meaning
gives (`late_let_cli`, `loop_parity`'s `a_function_value_never_reads_a_name_bound_after_it`). Never move a `let` back
into a scope a closure holds (A14's one-scope-per-block temptation); never snapshot captured scopes (vars are shared by
reference). And a witness must exercise the rule it names ALONE — M976 survived when the later value was itself a lambda
holding the same scope. Related: [[delulu-hardening-campaign]], [[skip-branch-verification-rule]],
[[mutant-run-per-target]].

Same day, same class, interpreter only: **LOCAL-CALLEE-1** (D-V2-200, RW 2.24) — `eval_call` asked "does a local hold a
CLOSURE?" where the checker asks "does a local bind the name?"; a plugin export (`PluginFn`) in a local named like a
module fn called the module fn (`why Write` said cannot write; it wrote). When hunting this class, read every runtime
name lookup against the checker's resolution order, and test each callable kind, not just closures.
Third the same day: **CTOR-SHADOW-1** (D-V2-201, RW 2.25, #107) — a fn may share a name with the module's own
constructor; the checker picks the CONSTRUCTOR; the interpreter called the fn (effectful `fn Wrap` named like pure ctor
`Wrap` wrote, NO special grant) and for values took any capitalized name as a ctor before fns. Checker order to mirror
everywhere: local → module ctor → module fn → prelude. Not taken: refusing fn-named-like-ctor (breaks programs that check).
