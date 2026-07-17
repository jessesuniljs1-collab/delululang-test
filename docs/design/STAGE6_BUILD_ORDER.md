# Stage 6 "Live" — Build Order

**Status:** COOKING — opened 2026-07-15 by the head chef.
**Binding spec:** `STAGE6_SPECIFICATION.md` (v0.6, Committed). **Binding how-to:**
`docs/playbooks/STAGE6_PLAYBOOK.md` — its phase plan (6a–6i), crate topology (§1), and traps (§3)
govern the build. This document adds nothing to either; it fixes the reporting gates, house
rules, and the close-out discipline. Precedence: **spec > playbook > this order** — record any
conflict as a deviation below instead of silently choosing.

---

## 1. Phases and reporting gates

The **playbook's phase plan is the build sequence**: 6a (DIR round-trip) → 6b (DIR
re-verification) → 6c (`.dpx` + manifest + `plugin build`/`inspect`) → 6d (load steps 1–4) →
6e (Verified verification + interpreter instantiation + `p.get`/calls) → 6f (Contained
verification + WASM instantiation + limits) → 6g (unload/reload R-6c) → 6h (Verified-on-WASM +
isolation + signatures) → 6i (`std.plugin` + `plugin verify` + authority report + `why`).

Each playbook phase ends in its own commit with the full suite green and the spec's
`## Implementation status` log updated (Stage-3 §8a pattern). The head chef's **reporting
gates** group them into four blocks — report and pause at each gate:

| Gate | Playbook phases | Ships |
|---|---|---|
| B1 | 6a, 6b | DIR: lossless round-trip + the replay-checker (hostile-input hardened; DL1503/DL1504) |
| B2 | 6c | `.dpx` container, `kind = "plugin"`, manifests (DL1501), `plugin build`/`inspect` |
| B3 | 6d–6g | the load pipeline, both classes, `p.get`/calls (R-Get, DL0803), limits (DL1506), unload (DL0801) |
| B4 | 6h, 6i + docs | signatures, Verified-on-WASM, `plugin verify` ≡ load, authority report, `why`, E-PLUGIN, §10 caveats verbatim, flagship example, close-out |

The playbook's §3 traps are binding review items: the close-out (§4 below) states, per trap,
which test proves it did not happen.

---

## 2. House rules (binding)

1. **Never push to GitHub.** Commit per phase on `master`; `git commit -F <msgfile>`.
2. **Zero pre-existing tests edited.** Baseline 461 passed / 0 failed / 2 ignored stays green on
   both engines after every phase. New behavior gets new tests.
3. **Machine channels are never styled.** `--json` output is byte-stable and never colored; the
   Palette applies only to human channels (Stage-8 early-drop ruling carries).
4. **Refusal honesty.** No partial artifacts on refusal; exact repairs exactly where the spec's
   §7 table says so; `requires_human: true` where it says none. A Verified failure never falls
   back to Contained (invariant 29).
5. **Dependencies.** Pre-approved new deps: one established CBOR crate (`ciborium` or
   `minicbor`) for DIR, and `ed25519-dalek` (v2) for signatures — cryptography is **never**
   hand-rolled. Wasmtime 27 / blake3 / serde_json are already in the workspace — reuse them.
   Any other new dependency: stop and escalate before adding.
6. **Crate placement — per playbook §1.** DIR in `delulu-check` (or a thin `delulu-dir` beside
   it); `dir::verify` **reuses the exact rule code** of `check_source` (playbook trap 3). The
   loader in `delulu-runtime`, reaching the WASM engine without creating a dependency cycle
   (trait/injection wired in the `delulu` crate if needed — record how). `.dpx` reuses the
   `.dwx` custom-section machinery in `delulu-wasm`. CLI wiring in `delulu`. *(The head chef's
   original ruling of a new `delulu-plugin` crate is withdrawn — the playbook's topology wins.)*
7. **Deviations ledger.** Any departure from the spec or this order is appended to §3 below,
   numbered, with what/why — and awaits a head-chef ruling. If blocked, in doubt, or the same
   error repeats more than twice: stop and report instead of thrashing.
8. **Determinism.** `plugin inspect --json` and `plugin verify --json` are byte-identical across
   runs on identical inputs.

## 3. Deviations

*(appended during the build; numbered; each awaits a head-chef ruling)*

**Deviation 1 (Phase 6b) — `dir::verify` reuses the real checker (resolve + check_module) rather
than a bespoke "no name resolution / not inferred" assert-only pass.**
*What:* Spec §2.3 describes DIR re-verification as replaying the checking pass with "types and rows
asserted, then verified — **not inferred** … O(nodes) and requires **no name resolution**." The
implementation instead reconstructs the module from DIR and re-runs the *exact same* pipeline
`check_source` uses (single-module `resolve` + `check_module`), then refuses (DL1504) unless the
recomputed facts/types/rows equal the DIR's stored ones and re-checking raises no error. This does
perform single-module name resolution and the checker's ordinary local inference.
*Why:* The playbook (trap 3) and the head-chef amendment mandate that `dir::verify` **reuse the
exact rule code of `check_source`, never a second implementation** — criterion 9 (verify ≡ load)
depends on zero drift. A hand-written assert-only re-derivation *is* the drift-prone second
implementation the mandate forbids. The spec's efficiency properties are met in spirit (single-module
resolve + check is linear and deterministic), and the soundness is strictly *stronger*: the load-time
check is byte-for-byte the same code path as the original compile-time check. Where spec and playbook
disagree (precedence: spec > playbook > this order), this records the conflict for a head-chef ruling
rather than silently choosing. *Status: **ruled: approved (conditions a, b)** — (a) docs honesty at
B4 close-out: the Implementation status log and user docs must state plainly that re-verification is
a full same-code-path re-check plus stored-truth comparison (strictly stronger than the §2.3
assert-replay description), never claiming the assert-only mechanism; (b) B4 close-out includes one
rough timing witness that `verify` of a realistic plugin is comfortably fast for per-load use.*

**Deviation 2 (Phase 6c) — DL1508 allocated for a malformed/tampered `.dpx` container.**
*What:* The spec §7 table allocates DL1501–DL1507 but has no code for a `.dpx` that is structurally
corrupt (bad magic, truncated sections, missing/unreadable `delulu:plugin` manifest, a Contained
module that fails its blake3 content binding). The runtime shape exists (`PluginErr::BadArtifact`)
but CLI diagnostics must carry a registered code (the conformance meta-test enforces it).
*Why:* Stage 3 hit the identical gap for `.dwx` and allocated DL1202 for exactly this class; DL1508
follows that precedent inside the fresh DL15xx range. Scope guard: DL1507 stays strictly "plugin API
version mismatch"; a tampered **DIR** body inside an otherwise-valid container is DL1504 (a failed
Verified re-check precondition — criterion 6 wording), never DL1508. *Status: **ruled: approved** —
condition: at B4, DL1508 joins the spec's diagnostics story via the Implementation status log and
`E-PLUGIN` explains it.*

**Deviation 3 (Phase 6c) — plugin packages are single-module in v0.6.**
*What:* `delulu plugin build` requires the plugin package to contain exactly one module; a
multi-module plugin package is refused cleanly at build (DL1004-class, clear message), never built
partially.
*Why:* DIR serializes one module and `dir::verify` replays the single-module `check_source` pipeline
(the Deviation-1-approved construction). Multi-module DIR would need a whole-program replay path
(`check_program`) with cross-module interface metadata — real work with no acceptance-criterion
coverage: every §9 criterion and the flagship demo use single-module plugins. Deferred, honestly
refused, and recorded rather than silently half-supported. *Status: **ruled: approved** as
honest-refusal-and-defer — conditions: (a) the B3/B4 report names the refusal's diagnostic, which
must say plainly that multi-module plugin packages are unsupported in v0.6 with no fake repair (it
is DL1004: "plugin packages are single-module in v0.6 — found N module file(s) under `src/`", zero
repairs); (b) the limitation lands in the spec's Implementation status, `E-PLUGIN`, and the
honesty-caveats list at B4; (c) it joins the post-v0.6 RFC ledger beside declared-effect plugins.*

**Deviation 4 (Phase 6e) — `load[C]` / `p.get[F]` are realized as inference-from-context, not
literal bracket syntax.**
*What:* The spec writes `load[C](host, path, grant)` (§3.1), `p.get[F](name)` (§3.3), and
`p.get[fn(Str) -> Str ! {}]` (§9.1). The Stage-1 grammar cannot parse those: in expression position
`[` is unambiguously `Index` (it parses an *expression*), so `p.get[F]("x")` would read as
`Index(Field(p, get), Var(F))` and `F` as an unknown name. The `[C]`/`[F]` are therefore implemented
as **inference-from-context** — a fresh variable pinned by the binding's annotation, e.g.
`let shout: fn(Str) -> Str ! {} = p.get("shout")?` and `let p: Plugin[Contained] = load(host, …)?`.
*Why:* This is settled by two normative precedents, not a free choice. (1) Stage-1 spec §6 states the
language rule outright: *"Generics are checked per call site by unification (**no turbofish, no
explicit instantiation** in v0.1)."* (2) Stage 4 hit the identical notation and resolved it the same
way, recording it as normative in the Stage-4 spec §3: *"The `[M]` of the normative signature is
realized as inference-from-context … because the grammar has no method type-argument syntax."* The
spec's brackets are notation for the normative signature, exactly as `root.foreign[M](load)` was.
R-Get and **DL0803 are unaffected**: `F` is fully known at the `get` call site (from the annotation),
so the compile-time checks fire exactly where the spec requires.
*Consequence to note at B4:* the flagship demo (§9.1) and the criterion-2/7 programs ship in the
annotation form; their spec text keeps the bracket notation. *Status: **ruled: approved** — no
rework; a turbofish would contradict a stated Stage-1 language rule to satisfy a notation. Condition
(B4): the spec's Implementation status names this explicitly, and `E-PLUGIN` shows the annotation
form so no user copies unparseable bracket syntax out of the spec.*

**Deviation 5 (Phase 6e) — DL1509 allocated: R-6a is fail-closed at a Contained `get` site.**
*What:* Head-chef review found DL0803 **failing open**, confirmed by probe: R-6a was decided by
`type_contains_fn(F)`, which silently **skipped** whenever `F` was underdetermined. Two programs
escaped with **zero diagnostics**:
(a) an unpinned `let f = p.get("x")?` — `F` stays a variable;
(b) **generic laundering** — `fn helper[T](p: Plugin[Contained], x: T) { let f: fn(T) -> Str ! {} =
p.get("g")?; f(x) }` called as `helper(p, some_closure)`. Because a generic's variables are
instantiated **fresh per call site**, the body's `T` is never unified with the caller's closure:
`F` reads as `fn('t0) -> Str`, `type_contains_fn` says false, and a closure reaches an opaque module
while R-6a never fires — exactly the re-entry point R-6a exists to forbid.
*Fix:* At a Contained `get` site, after substitution, the only accepting case is an `F` that is a
concrete `Type::Fn` containing **no inference variable at any depth** (new `type_contains_var`,
which — unlike `type_contains_fn` — recurses into function parameters and returns). A concretely
present function-typed parameter stays **DL0803**; an underdetermined `F` is the new **DL1509**,
refusing and demanding a concrete annotation. Note the hole was *not* "`F` is not a `Fn`" — in (b)
`F` *is* a `Fn`; it is the type **variable inside** it that could later be instantiated with a
function type. A rule that only demanded Fn-ness would still have let (b) through.
*Why a new code, not DL0803:* different fault, different remedy. DL0803 says "you passed a
function"; at the (b) `get` site no function is visible and that message would be a lie. DL1509 says
"this signature is underdetermined, so R-6a cannot be decided here — annotate it". Honest
diagnostics beat a reused code. Scope is tight: Contained `get` sites only; concrete function-free
Contained signatures and all Verified generics are unaffected (both witnessed).
*Witnesses (permanent, both were escapes):* `an_unpinned_get_on_a_contained_plugin_is_dl1509_not_a_silent_skip`,
`generic_laundering_of_a_closure_into_a_contained_export_is_dl1509`, plus the two scope guards
`a_concrete_function_free_contained_get_still_checks_clean` and
`a_generic_get_on_a_verified_plugin_is_not_refused`. *Status: awaiting ruling.*

**Deviation 6 (Phase 6f.2) — `anyhow` made a direct dependency of `delulu-wasm`.**
*What:* The trap-attribution core (`limits.rs`) downcasts Wasmtime's boxed error to `wasmtime::Trap`
and constructs honest `Unattributable` messages, which needs `anyhow::Error`/`anyhow::anyhow!`
directly. `anyhow` was already an in-tree transitive dependency of `wasmtime`, so declaring it
directly adds **no new build** — it is a visibility change, not a supply-chain addition.
*Why recorded:* House rule 5 pre-approves only `ciborium`/`minicbor` and `ed25519-dalek`; making a
transitive dep direct is neither a new dep nor clearly covered, so per the head-chef instruction it
is logged here for ledger honesty rather than treated as an unremarked change. *Status: noted per
head-chef's standing allowance; awaiting confirmation.*

**Deviation 7 (Phase 6f.2b) — Windows in-process CPU/wall enforcement is refused, not executed.**
*What:* On Windows + wasmtime 27, in-process CPU/wall limit enforcement uses host-initiated wasm
traps (fuel exhaustion / epoch interruption), and unwinding one `__fastfail`s the host process
(exit 0xc0000409) — an **uncatchable** crash, i.e. a plugin that merely spins would take the host
down, breaking trap 5's "host continues" promise. All three head-chef-ruled candidates were tried
as actual Windows runs and each still fastfailed inside `func.call`: (1) `wasm_backtrace(false)`,
(2) `signals_based_traps(false)`, (3) their combination. Bisected to the mechanism (fuel-only and
epoch-only each crash, with or without a guest memory, in debug and release); **guest** traps
(`unreachable`, div-by-zero) unwind cleanly, which is why Stage 3's differential and the
memory-bomb/bug-trap attribution witnesses are unaffected.
*Degradation (safety by construction):* on Windows, `run_contained_export` returns
`TrapCause::EnforcementUnsupported` **before creating any store** — the fastfailing path is
`#[cfg(not(windows))]` and does not compile into the Windows binary at all (non-negotiable 1: never
execute a path that can fastfail; a fastfail cannot be caught). The refusal is honest: not a limit
hit (never DL1506, never an authority-widening repair), not a plugin fault; it names the reason
(non-negotiable 2: no silent weakening). The evidence-based attribution core and its both-direction
witnesses run on **every** platform (unit tests); the live-engine criterion-5 witnesses (fuel, wall,
memory, host-survival, bug-trap-not-a-limit) run on the **non-Windows** cross-check, the
enforcement-grade platform (spec §5.2/§5.4).
*Why the BROAD cfg-out is correct, not over-broad (on the record):* a Contained plugin that can be
memory-limited but **not** CPU/wall-limited is not "Contained" — it could spin forever and hang the
host (a DoS by hang instead of by crash), which *looks like* "working" and is arguably worse to
reason about than an honest refusal. Refusing **all** Contained execution on Windows is more honest
than offering a half-containment that does not actually contain.
*Status: **ruled: approved (accept honest refusal for v0.6)**. The out-of-process Job Object path is
**NOT** built now and is deferred to the post-v0.6 RFC ledger (§5). Head-chef reasons of record:
(1) the spec's own enforcement-grade platform for hostile Contained code is the WASM engine on Linux
(§5.4/§10) — this degradation lands on the axis the spec already designates non-primary, and Verified
plugins run on all platforms via the interpreter; (2) out-of-process is a second execution
architecture that splits the enforcement model (fuel has no subprocess analogue) and collides with
R-6b invalidate-on-return and R-Get (marshalling host capability values across a process boundary) —
bolting it on under ship pressure is how the next fail-open gets built; (3) the root cause is most
likely an upstream wasmtime-27-on-Windows host-trap-unwind bug, whose right long-term fix is a
version bump that restores the in-process path with zero model split.*

**Deviation 8 (Phase 6h) — DL1510 and DL1511 allocated for signature faults; signature audit is
embedded-mode in v0.6.**
*What:* The spec §7 table has no diagnostic for a signature that fails to verify, nor for an unsigned
plugin under a `require_signed` grant. Two codes are allocated in the DL15xx range: **DL1510** — a
present-but-invalid signature (tampered content, a wrong key, or a malformed `delulu:sig` section),
refused regardless of `require_signed`; and **DL1511** — an unsigned plugin under a `require_signed`
grant (a policy refusal). Additionally, the signer identity is written to the audit log by
**embedded** custody (which holds the in-process broker + sink); the daemon-client `note_plugin_signature`
is a no-op in v0.6.
*Why:* A present-but-invalid signature and an unsigned-but-required plugin are **different faults with
different remedies** — one says "this signature does not verify", the other "this grant needs a
signature and none is present". Reusing one code (or reusing DL1508 "corrupt container") would make a
message a lie (the kitchen rule / skip-branch rule). The allocation follows the deviation-2 precedent
(DL1508) inside the fresh DL15xx range. Signatures authenticate **origin, not behavior** (spec §10):
neither code asserts anything about safety. The embedded-only audit recording is honest for v0.6 —
the load (and thus the signature verification) runs in the program process; wiring the daemon to log
a client-verified signature server-side is a small future addition, recorded here rather than
silently skipped. *Status: **ruled: approved as recorded** — the head chef's standard: "DL1510 vs
DL1511 as different faults with different remedies is the kitchen rule applied exactly right, and
'signatures authenticate origin, not behavior — neither code asserts anything about safety' is the
honesty standard." Conditions discharged: DL1510/DL1511 join the Implementation status log, the code
registry, and `E-PLUGIN`; the daemon-side signature-audit gap is stated in the honesty caveats and
joins the post-v0.6 RFC ledger (§5).*

## 4. Close-out

**Suite:** 461 (Stage-6 open) → **608 passed / 0 failed / 2 ignored** on Windows (full workspace).
Zero pre-existing tests edited. The head chef runs the Linux/WSL cross-check at close-out; the
platform split is stated per criterion below.

### The 11 §9 acceptance criteria, each with its witnessing test(s)

| # | Criterion | Witness(es) |
|---|---|---|
| 1 | The flagship demo (zero-authority transform loads/gets/runs; host row unchanged; rigged refused at load) | `flagship_a_zero_authority_text_transform_loads_gets_and_runs`, `flagship_a_rigged_variant_that_tries_to_tell_the_clock_is_refused_at_load` (delulu-runtime `plugin.rs`); runnable example `examples/plugin_shout/` (honest builds+verifies; `rigged/` refused DL1501) |
| 2 | Audit F-1: a Contained export types at `effects(grant)` (R-1); the honest annotation is the only one accepted | `r_get_contained_types_every_export_at_the_full_grant_r1` (delulu-runtime) |
| 3 | R-6c: unload → DL0801 w/ audit seq; reload → old ref dead forever, new handle works | `r6c_unload_then_a_retained_reference_is_dl0801_with_the_revoking_seq`, `r6c_reload_mints_a_fresh_node_and_the_old_reference_stays_dead_forever` (delulu-runtime) |
| 4 | R-7 composition across three levels; host revocation kills all transitively | `criterion4_r7_composition_a_sub_plugin_can_never_exceed_its_parent` (delulu-runtime) |
| 5 | Limits: infinite loop dies at fuel/wall; memory bomb at mem_mb; host continues; a bug-trap is NOT a limit | **Non-Windows (live engine):** `criterion5_infinite_loop_dies_at_fuel_and_the_host_survives`, `criterion5_infinite_loop_dies_at_wall_when_fuel_is_generous`, `criterion5_memory_bomb_dies_at_mem_mb_and_the_host_survives`, `criterion5_a_bug_trap_under_generous_limits_is_not_a_limit_through_the_real_engine` (delulu-wasm `limits.rs`). **All platforms:** the evidence-based attribution unit tests (`a_bug_trap_under_generous_limits_is_never_a_limit`, `an_unattributable_stop_claims_nothing_and_advises_no_widening`). **Windows:** `windows_refuses_contained_execution_rather_than_risk_a_fastfail`. **Interpreter best-effort (all platforms):** `best_effort_fuel_kills_an_infinite_loop_and_is_labeled_best_effort`, `best_effort_memory_accounting_stops_a_runaway_allocation_labeled_best_effort` (delulu-runtime) |
| 6 | DIR byte-flip → DL1504 at load; wasm-cache byte-flip → ignored, recompiled from DIR, load succeeds | `criterion6_dir_flip_is_dl1504_and_a_wasm_cache_flip_recompiles_from_dir` (delulu-wasm), `criterion6_load_succeeds_with_an_invalid_wasm_cache_never_rests_on_machine_code` (delulu-runtime), `tampered_dir_is_dl1504_dir_tampered` + `tampered_verified_wasm_cache_is_ignored_not_an_error` (delulu-wasm `dpx.rs`) |
| 7 | A function-typed parameter in `F` on a Contained `get` → DL0803 at compile time | `a_function_typed_param_to_a_contained_export_is_dl0803_at_compile_time`, `a_nested_function_typed_param_to_a_contained_export_is_also_dl0803` (delulu-check); fail-closed side: `an_unpinned_get_on_a_contained_plugin_is_dl1509_not_a_silent_skip`, `generic_laundering_of_a_closure_into_a_contained_export_is_dl1509` |
| 8 | A signed plugin verifies and its identity lands in the audit log; `require_signed` refuses an unsigned plugin | `criterion8_a_signed_plugin_verifies_and_its_identity_lands_in_the_audit_log`, `criterion8_require_signed_refuses_an_unsigned_plugin_as_dl1511` (delulu-runtime) |
| 9 | `plugin verify` ≡ real load across the corpus (no verify/load divergence) | `criterion9_verify_gives_identical_verdicts_to_a_real_load_across_the_corpus` (delulu-runtime; 9-artifact corpus proven to exercise ok/DL1504/DL1507/DL1510) |
| 10 | `why` traverses a Verified plugin to the primitive op; labels a Contained boundary | `criterion10_verified_why_traverses_the_real_chain_to_the_primitive_op`, `plugin_why_chain_is_cycle_safe` (delulu `cli.rs`); live: `why Read reader.dpx` → `[verified plugin reader] scan -> slurp — Read`; `why Net opaque.dpx` → `→ [contained plugin opaque-tool] — Net` |
| 11 | Full prior conformance suites green on both engines, embedded and daemon custody | The full 608-test suite green; two-engine parity (interpreter reference vs WASM, incl. the 5,000-program differential `wasm_matches_interpreter_on_random_programs_including_faults`); both custody modes (`EmbeddedCustody` default + daemon via `FakeCustody`/`DenyAll`/`AllowAll` seam tests) |

**Criterion 5 / 11 platform split (explicit, nothing claimed passing where unrun).** The live-engine
criterion-5 witnesses (fuel/wall/memory kills, host-survival, bug-trap-not-a-limit) are
`#[cfg(not(windows))]` and run on the Linux/WSL cross-check — the spec's enforcement-grade platform
(§5.2/§5.4). On Windows they do **not** run (in-process host-trap unwind fastfails there, deviation 7);
Windows instead runs the evidence-based attribution unit tests (every platform) and the honest
`EnforcementUnsupported` refusal witness. Verified-on-WASM inherits that Windows refusal by
construction; **Verified plugins run on every platform via the interpreter.** Criterion 11's
"both engines / both custody" is green on Windows here and re-run on the Linux/WSL cross-check.

### Per-trap proof lines (playbook §3)

1. **Class never inferred; Verified never silently falls back to Contained** — `step2_refuses_a_class_mismatch_and_never_substitutes`, `a_verified_class_never_falls_back_when_step5_fails_and_the_node_is_revoked` (delulu-runtime); a malformed DIR verifies as DL1504 with no fallback path (`a_malformed_dir_verifies_as_dl1504_never_falls_back`, delulu-check).
2. **R-1 is the honesty keystone** — `r_get_contained_types_every_export_at_the_full_grant_r1`; the F-1 scenario is refused at the gate unless the annotation covers the whole grant.
3. **`dir::verify` reuses the checker's rule code** — `valid_dir_verifies_and_agrees_with_check_source` (agrees with `check_source` across a corpus) + criterion 9's verify≡load corpus (one code path, proven, not spot-checked).
4. **No plugin reaches the broker** — `dl1505_a_root_constructor_is_never_in_any_slice` (a plugin can never mint a capability); invariant 31 is guaranteed by *absence* (the loader holds the `GrantId` host-side; the plugin's value vocabulary has no `attenuate`/`revoke`/lease/IPC).
5. **A limit-killed plugin is gone, not wounded** — `trap5_a_limit_killed_plugin_is_gone_not_wounded` (drops the instance AND revokes the node in one act); the criterion-5 host-survival witnesses confirm the host continues.
6. **Interpreter limits are best-effort, and it says so** — `best_effort_fuel_...` / `best_effort_memory_...`: the DL1506 message states it is best-effort and names the WASM engine as the enforcement-grade path; the interpreter is never claimed to contain hostile code.
7. **Signatures authenticate origin, not behavior** — the §10 caveat is verbatim in `delulu explain E-PLUGIN` (`plugin_codes_and_e_plugin_topic`); `a_badly_signed_plugin_is_dl1510_a_different_fault_from_unsigned` keeps the fault honest.
8. **Exports are functions only in v1.0** — a generic export is refused DL1501 at build (delulu-check plugin fence); plugin packages are single-module (deviation 3, DL1004), refused cleanly, never half-built.

### Deviation ledger status

Deviations 1–7 were ruled during the build (§3); deviation 8 (DL1510/DL1511 + embedded-mode
signature audit) is recorded above and awaits a head-chef ruling. All deviation conditions are
discharged: the honesty text lives in `delulu explain E-PLUGIN`, `docs/design/STAGE6_PLUGINS_GUIDE.md`,
and the spec §11 Implementation status; the deviation-1(b) timing witness is
`verify_is_comfortably_fast_for_per_load_use` (~0.75 ms/load).

## 5. Post-v0.6 RFC ledger

Deferred by ruling, recorded so no future builder starts blind:

1. **Multi-module plugin packages** (Deviation 3). Needs a whole-program DIR replay path
   (`check_program`) with cross-module interface metadata. Refused honestly (DL1004) in v0.6.
2. **Declared-effect plugins** (spec non-goal). Plugins exporting types or effects would
   complicate row identity across load boundaries — RFC-gated by the spec itself.
3. **Windows Contained-execution enforcement** (Deviation 7). v0.6 refuses Contained execution
   on Windows honestly (`EnforcementUnsupported`) because wasmtime-27's host-initiated trap
   unwind fastfails the process there (bisected; three Config candidates exhausted).
   **Preferred resolution:** revisit the in-process path on a future wasmtime whose Windows
   host-trap unwind is fixed — zero enforcement-model split. **Fallback only if that never
   lands:** out-of-process execution via the Stage-5 foreign-worker subprocess +
   `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` — knowing going in that (a) fuel (deterministic CPU
   metering) has no subprocess analogue, so the enforcement model splits by platform; and
   (b) host capability values must never be marshalled across the process boundary in a way
   that weakens R-6b invalidate-on-return or R-Get — that design must be ruled on before it
   is built.

4. **Daemon-side plugin-signature audit** (Deviation 8). v0.6 records a verified plugin
   signature's identity in the audit log via **embedded** custody (which holds the in-process
   broker + sink); the daemon-client `Custody::note_plugin_signature` is a no-op. The load — and
   thus the signature verification — runs in the program process, so wiring the broker daemon to log
   a client-verified signature server-side is a small future addition (a new IPC note), deferred here
   rather than silently skipped.
