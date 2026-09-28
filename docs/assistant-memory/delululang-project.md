---
name: delululang-project
description: "DeluluLang — Jesse's authority/effect-typed language; Stages 1-4 COMPLETE (4 = Foreign: C FFI + embedded CPython + WASM parity, 271 tests green); Stage 5 (Custody) is next, per docs/playbooks/STAGE5_PLAYBOOK.md"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-09-14T09:51:47.983Z
---

Jesse's long-held dream project: **DeluluLang** (`.delulu`), an open-source language whose identity
is *authority made intrinsic* — every function/module/plugin carries authority + effects in its
type; total whole-program verification; AI agents are the primary users but the principle is
authority, not "AI-first". Repo: `D:\nelan\DeluluLang` (not yet a git repo as of 2026-07-04).

Committed deliverables (2026-07-04):
- `CONSTITUTION.md` — v1.0 language constitution (normative; wins over older drafts
  `LANGUAGE_SPECIFICATION.md` / `DeluluLang_PROMPT.md`).
- `STAGE1_SPECIFICATION.md` — buildable Stage 1 blueprint (EBNF, tokens, AST, effect core,
  runtime, plugins, diagnostics contract, acceptance criteria).

Key hardened decisions (deviations from Jesse's earlier drafts, agreed rationale in Constitution
Appendix A): explicit effect rows on named fns (omitted = pure; inference powers repairs only);
row polymorphism added; no exceptions/continuations (Result + abort-panic); no module-level
mutable state; kind-vs-scope split stated honestly; first compiled backend = WASM (C transpile
dropped); plugins split `Plugin[Verified]` vs `Plugin[Contained]`; implementation language Rust;
authority widening = semver-major. Syntax: braces, `!{Read, Net}` rows, `[]` generics,
`Cap[FsRead]`-style capability params, Go-style newline termination.

Honesty clauses are binding: never "faster than C" (say "competitive with C on hot paths, with
safety C cannot offer"), never "lowest tokens", never "unbreakable" without threat model.

Reference projects (verified 2026-07): aglang v0.3.0 (Z3/SMT deterministic architecture checks),
zerolang v0.1.x (Vercel Labs, graph-native agent language).

2026-07-05 session: `SOUNDNESS_AUDIT.md` committed (normative; rules R-1…R-7: Contained-plugin
exports typed at full grant row; Declassify core effect on expose; invariant unification with
subsumption only at declaration sites; builtin-callback law; opaque types banned from
str/==/serialize; no fn args to Contained exports; monotone attenuation ⊑ for the holder model).
Constitution gained §5.16 holder model, §8.4 terminal-first/LSP, §8.5 localization (en-US +
Delulu Slang catalogs, first-run choice) + Jesse's verbatim welcome note (never alter it; human
first-run only, never JSON/CI/agents). Stage plan (numbering fixed): 2 packages+authority
lockfile+Delulu Core, 3 WASM floor, 4 FFI/Python, 5 broker+microVM, 6 plugins, 7 actors/ref-caps,
8 tooling/LSP/localization, 9 v1.0 stabilization, 10 production (JIT policy, robotics Actuate,
LTS). Stage specs written at full depth: 2 (Provenance — packages/lockfile/Delulu Core), 3
(Containment — WASM/.dwx/delulu:cap), 4 (Foreign — C FFI via foreign blocks + libffi, embedded
CPython via Cap[Python]/PyObj, no callbacks ever, DL13xx), 5 (Custody — broker daemon, grant
tree, delegate/lease tokens, epoch≤50ms vs synchronous revocation classes, hash-chained audit,
foreign workers, microVM Linux-first, DL14xx), 6 (Live — plugins runtime, .dpx + DIR typed IR,
load sequence, R-1/R-6/R-Get enforced, limits as grant data, DL15xx). **All stage specs 1–10 complete** (7 Concurrent — actors/rcaps/Async, no await, contextual
be/new, consume/recover keywords + migration; 8 Surface — LSP/fmt/delulu test/catalogs
en-US+delulu-slang/welcome/signing+registry client; 9 Delulu — v1.0 freeze, coverage law,
measurement studies A/B/C, governance, registry live, reproducible release; 10 Industrial —
P1–P6 production checklist, exec.native grant for JIT, robotics Actuate with envelope scopes +
dead-man heartbeat leases + sim-to-real hash gate, LTS). DL ranges used: 10xx/11xx (S2), 12xx
(S3), 13xx (S4), 14xx (S5), 15xx (S6), 16xx (S7), 17xx (S8), 18xx (S9), 19xx (S10).

**IMPLEMENTATION IN PROGRESS (2026-07-05).** Repo git-init'd at D:\nelan\DeluluLang; design docs
moved to docs/design/ (manifest: docs/REPOSITORY_STRUCTURE.md). Toolchain: rustup INSTALLED
(cargo 1.96.1 at ~/.cargo/bin, NOT on default PATH — prepend it); default toolchain is
stable-x86_64-pc-windows-MSVC which needs link.exe. Machine had NO MSVC linker and only 32-bit
MinGW (C:\MinGW gcc 6.3.0, unusable for x64). VS 2022 Build Tools (VCTools workload) install was
TOOLCHAIN WORKING: rustup + VS 2022 Build Tools (VCTools) both installed. `cargo test --workspace`
= 48 tests green (diag 6, syntax 28, check 14). Prepend $env:USERPROFILE\.cargo\bin to PATH each shell.
CRATES DONE + VERIFIED: delulu-diag, delulu-syntax, and **delulu-check (THE HEART)** — ty.rs,
unify.rs (R-3/R-3b), resolve.rs (DeclTable + prelude IoErr/NetErr + generic classification DL0410),
check.rs (full T-* judgment: T-CapOp via method_sig = compile-time primitive table; subset boundary
DL0501 + add_effect_to_row repair authority_widening=true; DL0502 + remove repair; secret rules
DL0602/0603/0604/0605; opacity R-5; match exhaustiveness DL0407; `?` DL0409; row polymorphism via
RowAcc), authority.rs (§10.5 report + reachability from main), lib.rs. Reference demo checks clean;
report = effects[Read,Write], pure[apply,fib], secrets[API_KEY]. Living-doc fixes: demo/reference
program rewritten (?-in-Unit-main was ill-typed → ? now inside read_config, match in main);
module-level var→DL0305 in parser; parser primary now single-ident so `out.println` = Method.
**REMAINING Stage 1, resume here:** (1) delulu-runtime crate {value.rs, cap.rs (CapVal+Scope),
broker.rs (Root grant flow §7.2 = Stage-1 broker), interp.rs (tree-walk over checked AST), prim.rs
(runtime primitive execution table §7.3 mirroring check.rs method_sig), manifest.rs (delulu.toml)}.
(2) delulu CLI crate {main.rs, cli.rs (check|run|repl|authority + --json + --grant + DL07xx), repl.rs}.
(3) tests/conformance (accept+reject per rule), tests/laundering (F-1..F-6,R-7), tests/corpus tiers 1-5.
(4) END-TO-END: `delulu authority examples/demo.delulu` prints §10.5 report — USER CHOSE
compile+run-demo-end-to-end as the bar. Not git-committed yet (0 commits) — offer commit when it runs.

**STAGE 1 COMPLETE + VERIFIED (2026-07-05).** All 5 crates done; **65 tests green** on MSVC Windows
(`$env:USERPROFILE\.cargo\bin` on PATH; `cargo test --workspace`). delulu-runtime {value,prim
(runtime primitive table §7.3 with host-side scope checks),interp (tree-walker, ?/return via
Escape enum, checked arith DL0901/0902/0903, depth DL0905),broker (Grants + minimal delulu.toml
parse + DL0701),lib} and delulu CLI {main,cli (check|run|authority|repl|explain, --json envelope,
--grant flags),repl} all built. VERIFIED end-to-end: `delulu authority examples/demo.delulu` prints
effects[Read,Write]/caps[FsRead,Console]/secrets[API_KEY]/pure[apply,fib] (human + JSON §10.5);
`delulu run examples/demo.delulu --grant console --grant fs.read=./config --grant secret:API_KEY=...`
outputs hello/fib=55/apply=42/(no config found); `delulu check leak --json` = DL0501 + typed
add_effect_to_row repair authority_widening=true. Laundering suite crates/delulu-check/tests/
laundering.rs (12 tests) = audit F-2/F-3/F-3b/F-4/F-5 + core non-escape. F-1/F-6/R-7 are Stage-6.

**STAGE 1 FULLY DONE + CONFORMANCE CORPUS (2026-07-05). 75 tests green.** Two git commits on
master: 8ee8ff1 (compiler+runtime) and 35289b1 (conformance corpus). Corpus: tests/conformance/
accept/ (8 accepting programs), tests/conformance/reject/DLxxxx_*.delulu (one per Stage-1 static
code DL0106..0605), tests/corpus/tier1-5 (tier4 multimodule deferred to Stage 2 w/ NOTE.md).
Harness crates/delulu/tests/conformance.rs (accept-clean, reject-by-filename-code, coverage law)
+ crates/delulu/tests/cli.rs (7 tests driving the real binary: authority human+JSON, check --json
repair, run demo, DL0703). BUGS FOUND BY CORPUS + FIXED: (a) nullary variant patterns `Red`
parsed as bindings → every match "exhaustive" → DL0407 never fired; fixed in parser.rs
parse_pattern (capitalized name=variant, lowercase=binding). (b) `push` returned list not Unit →
match-arm mismatch; now Unit + unifies element. Also: secret-into-sink now dedicated DL0602 (was
DL0401); Secret.verify added to method table. Toolchain: cargo 1.96.1 + VS BuildTools; prepend
$env:USERPROFILE\.cargo\bin. **STAGE 2 STARTED** (commit 24191ba, 78 tests green). Stage-1 re-reviewed inline 2026-07-05:
GREEN — 75 tests, compile-time (check.rs method_sig) and runtime (prim.rs) primitive tables
AGREE on all effect-bearing ops, 5/5 adversarial escape programs rejected (closure-captured cap,
fn-in-list, secret-in-record ==, expose without row, nested lambda). Honesty notes: http.get is
stubbed (returns Err Refused, no net client bundled), grant prompt simplified to --grant flags —
both documented Stage-1 boundaries.

Stage 2 done so far (commits 24191ba, 5062993; 81 tests green):
- package.rs: load_package(dir) discovers+parses src/**.delulu, indexes by declared module name,
  validates import graph (DL0303 unknown module, DL0304 cycle).
- **program.rs: cross-module resolution DONE.** check_program(pkg) builds a GLOBAL type registry
  (prelude IoErr/NetErr at ids 0/1, then all modules' types), gives each module a scope = own decls
  + imported pub items, checks each module against it (Stage-1 check_module reused unchanged), keys
  facts by "module::fn". `import m` makes m's pub fns/types/consts/effects visible; private → DL0301;
  collisions → DL0302. program_authority(program,name,scopes) walks reachability from entry::main
  across modules (call_owner map resolves callees to owning module), unions effects/caps/secrets.
  ScopeInfo::for_kind made pub. CLI: `delulu check <dir>` + `delulu authority <dir>` now handle a
  package directory (else single-file path unchanged). examples/greeter/ = real 2-module package
  (main imports greeter.greetings; authority reports effects=[Write], imported helpers proven pure
  across boundary). Stage-1 check_source UNTOUCHED.
Package-authority SELF-CHECK DONE (commit 48a62c2, 84 tests green): Stage-2 codes DL10xx/11xx
registered in delulu-diag/src/codes.rs; program_effects(program) exposes reachable-from-main effect
set; CLI `delulu check <dir>` + new `delulu build <dir>` verify computed effects ⊆ manifest.effects,
excess → DL1009 (span at the `effects` line of delulu.toml + repair msg). examples/greeter builds
clean; a Read-declaring Write-performing package fails DL1009. Manifest still parsed by
delulu_runtime::parse_manifest (broker.rs) — self-check done in cli.rs (delulu crate sees both
program + manifest; avoided moving Manifest to delulu-check).
**STAGE 2 PROVENANCE CORE DONE (2026-07-05, 121 tests green).** Built via 2 parallel subagents
(Opus=provenance/delulu-check+cli, Sonnet=tracing/delulu-runtime; disjoint files; I reviewed+
committed). New commits: 368b5f6 (tracing+determinism), ec9ff21 (deps/pins/lockfile/semver),
e85b0af (CLI trace/assert/seed/clock glue). New delulu-check modules: manifest.rs (toml crate,
[package]/[authority]/[dependencies]+pins, DL1004 malformed), deps.rs (resolve_workspace recursive
path deps DL1005/1007/1008, check_workspace one global type registry cross-package pub imports
DL1006, check_pins DL1001 attenuation-order, check_self_authority DL1009), lockfile.rs (blake3
content/authority/api_row hashes, verify_locked DL1010/1002/1011, enforce_semver_law DL1003 +
accepted_by). delulu-runtime: trace.rs (TraceRecord/TraceSink hand-JSON, effect_for map, assert_trace
= trace⊆row / DL1101; secret redaction=«opaque» structural at dispatch), prim.rs set_rand_seed/
set_fixed_clock_ms. CLI: `delulu build [--locked]`, `delulu lock [--accept-authority <pkg>]`, run
flags --trace-effects/--trace-out/--assert-trace(exit 3)/--seed/--clock fixed:MS. Deps added to
delulu-check/Cargo.toml: toml 0.8, blake3 1 (root Cargo.toml untouched; Cargo.lock gitignored).
xz scenario tested (provenance.rs): pinned-pure dep gains !{Net} → DL1001 before any run.
Dependency graph: delulu-diag ← syntax ← check ← runtime ← delulu(cli). Spec §12a records
deviations (DL1004; kind optional=bin; pins need TOML table form; git-dep resolution deferred=DL1007
/refuse, not faked).
**STAGE 2 COMPLETE (2026-07-05, 132 tests green).** Extra commits: 130ade7 (why/authority --diff/
interface.json/DELULU_CORE.md), 83a6dd3 (pub import re-exports + delulu-fuzz). All criteria 1-11
met. NEW: `delulu why <Effect>` (fn-granularity origin path), `delulu authority --diff` (per-pkg
lockfile delta + WIDENING verdict), interface.json per pkg on build (§5.5), DELULU_CORE.md (formal
calculus, Progress/Preservation/Effect-Soundness w/ paper sketches + honesty clause "not
mechanized"), `pub import` re-exports (AST Import.public + parser + deps.rs check_workspace exports
w/ DL1005 cycle), and crate `delulu-fuzz` (differential harness: generate danger-zone programs,
check, run accepted under trace, assert trace⊆row(main) = executable Effect-Soundness; verified
100k iters / 83292 accepted / zero violations; 4k in cargo test). USER SWITCHED OFF subagents
(session limits) — doing everything inline now as head chef. Windows transient LNK1104 = kill stray
delulu*/delulu-fuzz processes then retry.
**STAGE 3 ("Containment") IN PROGRESS.** Phase 3a DONE (commit 4878428, 137 tests green): new crate
crates/delulu-wasm (wasm-encoder 0.221 + wasmtime 27, default-features off + cranelift+runtime —
CONFIRMED builds on Win/MSVC; first wasmtime build ~70s). codegen.rs compiles the pure-Int/Bool
fragment (arith/cmp/&&/||/unary/if-else/let/calls/recursion) to CORE wasm (Int=i64, Bool=i32);
host.rs runs it under Wasmtime with EMPTY imports (deny-by-default). Two-engine PARITY is the
correctness contract: compile_and_run_int vs interp.call_int_fn (NEW additive method on Interp)
agree on fib/gcd/poly/nested-if/even/neg. Unsupported constructs = CompileError (DL1201), stay on
interp (reference engine). wasm-encoder API notes: use TypeSection.ty().function(params,results)
(not .function); Instruction owned variants; Function::new(locals iter of (count,ValType)); untyped
Func::call with &[Val]/&mut[Val]. Section order: Type,Function,Export,Code.
Phase 3b DONE (commit 981ff76, 139 tests green): delulu:cap host interface FIRST SLICE. codegen.rs
now handles Str (string literals length-prefixed in linear memory, Str=i32 ptr), Cap[Console]
(i32 handle), Unit; Cap[Console].println(str) → imported host fn delulu:cap.console_println.
host.rs run_console_fn uses a Wasmtime Linker: performs Write host-side, checks cap handle vs host
cap table (ungranted refused = scope check), reads string from guest's exported memory. KEY FIX:
record cap refusal in HostState + surface AFTER call — returning Err from inside a wasm-invoked
callback ABORTS on Windows ("non-unwinding panic. aborting"). wasmtime switched to DEFAULT features
(clean traps). Runtime: added capturable console prim::set_capture/take_capture + emit_console (used
in Console println/print) and Interp::call_with(name, Vec<Value>); re-exported CapVal/CapScope. The
parity test compares WASM host captured output to interp captured output — IDENTICAL on a Write
effect (not just pure). wasm-encoder 0.221 API used: ImportSection.import(mod,name,EntityType::
Function(idx)); MemorySection.memory(MemoryType{minimum,maximum,memory64,shared,page_size_log2});
DataSection.active(0,&ConstExpr::i32_const(0),bytes); section order Type,Import,Function,Memory,
Export,Code,Data.
Phase 3c DONE (commit e65ee0a, 141 tests green): generative differential parity gate. delulu-wasm/
src/gen.rs generates random PURE programs (f may call g, g calls nothing = terminating; only +/-
over small operands = overflow-free; no division = trap-free); test compiles+runs 800 (>500 valid)
on BOTH engines, asserts identical Ok(i64). Zero divergence.
**KNOWN HONEST DIVERGENCE (recorded in spec §8a):** WASM backend WRAPS on Int overflow (native
i64) while interp FAULTS DL0901 (checked arith); i64.div_s TRAPS on div0/INT_MIN÷-1 where interp
faults DL0902. Generator avoids these so parity holds. Closing = emit overflow/div checks in
codegen per spec §3.3.
Phase 3d DONE (commit f3a6d4d, 141 tests green): CHECKED-ARITHMETIC codegen closes the fault
divergence. codegen.rs emits 4 synthetic helper fns (__ovf_add/sub/mul trap on signed overflow via
sign analysis; __chk_rem traps b==0 & INT_MIN%-1) at indices [n_imports, n_imports+4); user fns
shift to n_imports+4+i (N_ARITH_HELPERS=4, Cx.arith_base=n_imports). +/-/*/% route through helpers;
/ uses i64.div_s (already traps like checked_div). Parity harness upgraded: gen.rs now makes
overflow (large operands + *) and div0 (small divisors incl 0); test treats BOTH-Err as consistent,
asserts both_faulted>50 & agreed_ok>1000; 5000 programs, zero divergence (~24s). Spec §8a marks
divergence CLOSED.
Phase 3e DONE (commit 8d6cb62, 143 tests green): Root/main capability threading through WASM.
codegen.rs: Ty::Root (i32 handle); root.console() → host import delulu:cap.root_console(root)->cap;
console imports now 2 (root_console idx0, console_println idx1), N_CONSOLE_IMPORTS=2, n_imports=2 when
needs_console, helpers shift to [2,6), user fns 6+. host.rs run_main_console(wasm, console_granted):
cap table seeded [Root]@0, build_linker provides both imports (host-side checks, NEVER trap from
callback — record refused + surface after via finish()). Milestone test: main{root.console();
out.println} on WASM == interp output, ungranted console refused. run_console_fn now inits caps=
[Console] (greet-style handle-0 tests still pass).
**NEXT Phase 3f (CLI glue — makes WASM terminal-reachable):** add delulu-wasm as dep of delulu(cli);
register DL12xx in codes.rs (DL1201 codegen-unsupported, DL1202 artifact invalid, DL1204 iface
version, DL1205 secret-closure, DL1206 parity-fail); parse `--engine wasm|interp` into Opts; cmd_run
--engine wasm branch: compile_module (CompileError→DL1201) + run_main_console(wasm, grants.console)
+ print output (refusal→DL0703/DL0904). Add examples/hello_wasm.delulu (simple console main). CLI
test: run --engine wasm hello_wasm --grant console → prints. THEN .dwx artifact (§5: wasm + custom
section delulu:authority via wasm_encoder CustomSection) + `build --target wasm` writes .dwx +
round-trip. Then broaden delulu:cap (clock/rand), string concat, full conformance parity (§9 1-3),
hostile-guest (§9.4), secret-hygiene (§9.8). Interp = reference.
COMMIT TIP: here-strings break on ()/quotes in msg — use `git commit -F <file>` for messages with
special chars. Crates: delulu-diag, syntax, check, runtime, delulu(cli), delulu-fuzz, delulu-wasm. Q1-3 answered: real interpreter (real OS
syscalls now; WASM sandbox Stage 3; microVM Stage 5; robotics Stage 10 = 1-100Hz command layer
only, not kHz firmware); CUSTOM compiler in Rust from scratch (not built over another; VS Code =
Stage-8 LSP integration, it's an editor not a compiler); terminal-first CLI already works.

**STAGE 3 (Containment) THROUGH PHASE 3n (2026-07-06, 186 tests green, HEAD 9c2abff).** Per-phase
detail lives in git log + docs/design/STAGE3_SPECIFICATION.md §8a. Consolidated: WASM backend now
compiles console + str(Int) + Str concat + Cap[Clock] + Cap[Rand] with two-engine parity as the
correctness contract (interp = reference engine). Phases: 3f `delulu run --engine wasm` terminal-
reachable (DL0703/DL0904 mapping via wasm_fault_code); 3g the **.dwx artifact** (wasm + delulu:authority
custom section carrying the authority JSON, blake3-bound to code; build --target wasm / run file.dwx;
DL1202 tamper/DL1204 version; artifact.rs); 3h **hostile-guest test** (hand-crafted adversarial wasm:
forged handle→DL0904, OOB ptr→DL0903, ptr=-1 no host abort, unprovided import fails instantiate) +
host memory hardening (unsigned wasm ptr + checked arith — a hostile ptr could've overflowed usize and
aborted the process on Windows); 3i **Str concat** (__concat bump-allocates in a 1 MiB heap, mutable
i32 global bump pointer, memory.copy); 3j **str(Int)** (__int_to_str, i64::MIN via unsigned-magnitude
trick); 3k **Cap[Clock]** (host-side, fixed via --clock for parity); 3l **Cap[Rand]** (host replicates
the interp's EXACT xorshift64 seeded via --seed → byte-identical sequences); 3m **conformance parity
harness** (conformance_parity.rs: curated all-caps programs + 2000-program console fuzzer + DL1201
fallback assertions); 3n **secrets stay host-side** (CompileError::SecretInGuest → DL1205 for
root.secret/expose; secret code never compiles so secret bytes never enter guest memory). Also fixed:
lexer now skips a leading UTF-8 BOM (was DL0101; commit ba44887). host.rs run_main(wasm,&HostConfig
{console,clock,rand,fixed_clock_ms,rand_seed}); run_main_console is a back-compat wrapper. Import
indices generalized via module_calls_method + Imports struct. **NEXT BIG PIECE (do INTERACTIVELY, not
autonomously — flagged to Jesse): Result/Option + match + `?` codegen in the WASM backend** — the
prerequisite that unlocks the fs delulu:cap ops (fs read returns Result[Str,IoErr]), the §9.4(b)
subtree-escape hostile test, and extending conformance parity to the FULL corpus. Smaller remaining:
§9 criterion 9 (≥50k both-engine fuzz gate). Defender transient LNK1104 on Windows: kill delulu*
procs + `cargo test --workspace --no-run` first, then run.

**PHASE 3o — SUM TYPES IN WASM (2026-07-06, 194 tests green, HEAD 47c271c).** The big interactive
lift (Jesse chose "do it together with checkpoints"). Checkpoint 1 (b72c6b2): Result[T,E]/Option[T]
with scalar payloads — Ty gained Result(Scalar,Scalar)/Option(Scalar); a variant = i32 ptr to heap
[tag:i32][field]; EXPECTED-TYPE-DIRECTED construction (compile_expr_as/compile_block_as thread the
declared type into tail position since the backend does no inference); match lowers to a tag test +
typed field binding. Checkpoint 2 (56c21ea): the ? operator (unwrap Ok / early-return the Err cell).
Checkpoint 3 (5714433): generalized to arbitrary sum types via EnumEnv (prelude IoErr/NetErr + module
enums → stable ids); Ty::Enum(id) + Scalar::Enum(id) so payloads can be enums (Result[Str,IoErr]);
N-ctor multi-field cells [tag][field@4][field@12..] size 4+8*maxfields; nested if tag==t chain; the
fn index now carries PARAM TYPES so a ctor can be a direct call arg. FINDING: Stage-1 source couldn't
CONSTRUCT user variants (checker resolved only Ok/Err/Some/None → DL0301). Jesse chose to FIX THE
CHECKER first. Fix (47c271c): resolve.rs DeclTable::variant_ctor (unique-name resolution; Other is
ambiguous across IoErr/NetErr → deferred); check.rs check_var (nullary) + check_call (payload) with
field-type checks; interp.rs eval_var/eval_call construct Value::variant for a capitalized unbound
name. Now user enums construct+match on BOTH engines byte-identically (name(Green), render(Say("x")),
Result[Str,IoErr] via Err(NotFound) + nested match). Checkpoint 4 = the FILESYSTEM cap DONE
(a75f619, 197 tests). fs.read_text(path) -> Result[Str, IoErr] on --engine wasm, byte-identical to
interp. codegen: Ty::FsRead; root.fs_read -> root_fs_read import, fs.read_text -> fs_read_text import
(-> Result[Str, IoErr] via EnumEnv's IoErr id); the guest bump-heap __heap global is now EXPORTED so
the host can allocate in guest mem. host.rs: CapKind::FsRead(PathBuf) scope; HostConfig.fs_read_roots;
the host reads the file host-side then CONSTRUCTS the Str + Result + IoErr cells directly in guest
memory (matching codegen layout: Result Ok=0/Err=1, IoErr NotFound=0/Denied=1/Other=2), enforcing
scope with the interp's EXACT normalize (lexical ./..); a `..` escape = hard DL0904 (§9.4b done).
CLI threads --grant fs.read=<path> via grants.build_root().fs_read. **STAGE 3 COMPILABLE FRAGMENT NOW
COMPLETE: caps (console/clock/rand/fs) + Str + Result/Option/user-enums + match/? + .dwx artifact,
all two-engine parity.** Interp = reference engine throughout.

**PHASE 3q — §9 CRITERION 9 DONE (6593c71): the >=50k two-engine differential fuzz gate.**
crates/delulu-wasm/tests/differential.rs runs a mix (~60% pure-arith incl overflow/div0 faults, ~40%
console str/concat) on BOTH engines, requiring identical value/output (or both-fault); #[ignore]d,
env-tunable DELULU_FUZZ_N (default 50000). VERIFIED at 50000: 50000 checked, 46408 agreed, 3592
both-faulted, ZERO divergences (~502s). **STAGE 3 ESSENTIALLY COMPLETE** — every automatable §9
criterion met; the only true remainder is §9.8's scan-and-expose-then-appears form (needs secrets
representable in the compiled fragment = a host-mediated secret design, deferred). A few §9 criteria
are partial by DESIGN-dependency, not effort: crit 1 (full demo.delulu -> .dwx) needs generics codegen
(apply[T,U,e]) which the backend skips; crit 6/7 (.dwx receipts/verified-vs-contained grade) are
trust bookkeeping; crit 4c (revoked-grant) is Stage-6 plugins. Jesse chose to polish Stage 3
first (generics codegen), done next.

**PHASE 3r — GENERICS + NON-CAPTURING LAMBDAS via INLINING (cfaab70, 200 tests).** Instead of
monomorphization/function-tables, the backend INLINES: a call to a generic user fn or a bound function
value evaluates value args into fresh locals, binds fn-typed args as Callables (owned lambda/AST
clones), compiles the body in a new frame. apply(fn(x){x*2},21) reduces to inlined arithmetic — no
call_indirect. codegen: generic fns kept out of the top-level export set, collected into a `generics`
map; a `callables` scope stack + inline_depth guard. Per-site inline = per-site mono (id[T] at Str
AND Int in one program). Non-capturing lambdas only; capturing/recursive-generic fall back DL1201.
Covers apply[T,U,e]/twice/id. demo.delulu now compiles all the way to its root.secret line (correctly
DL1205; wasm_cli DL1201 test moved to a `while` program). END-TO-END a program composing generics +
lambda + fs.read_text->Result[Str,IoErr] + nested match + console + str BUILDS TO A 1403-byte .dwx and
runs (apply=42 / file content), authority re-verified — criterion 1 for a representative program.
**STAGE 3 IS DONE: the full compilable language surface — caps(console/clock/rand/fs) + Str/concat/str
+ Result/Option/user-enums + match/? + generics/lambdas + .dwx artifact + hostile-guest proofs + 50k
differential gate — all under two-engine parity, 200 tests.** Only true remainder: §9.8 scan-and-expose
secret-hygiene (needs secrets representable in the fragment = host-mediated secret design). NEXT = Stage
4 (Foreign — C FFI via foreign blocks + libffi, embedded CPython via Cap[Python]/PyObj, NO callbacks,
DL13xx) — a big new frontier. Stages 4-10 (FFI/Python, broker/microVM, plugins, actors, tooling/LSP/
localization+welcome note, v1.0, production) ahead. Checked in with Jesse at this milestone.

**FABLE 5 PLANNING PASS (2026-07-07/08, commits 913f03e..7f5eb6a, ZERO code changes).** Jesse's
directive: Fable 5 (Mythos) does planning/spec .md only (its coding triggers safeguard model-switches
to Opus); Opus 4.8 builds from these. At ~70% weekly limit he asked to prioritize: full depth on
integration docs + Book; starters elsewhere. Delivered: (1) docs/playbooks/README+STAGE4..10_PLAYBOOK
.md — execution companions to specs (build order, phase plans, traps, definition-of-done; spec=what,
playbook=how); Stage 4 playbook = the immediate working plan for the next build stage. (2)
docs/design/ trio: LOCALIZATION_PLUGIN_GUIDE.md (human-language catalog plugins: TOML format, stable
keys, typed placeholders, zero-authority verified plugins via delulu locale add/remove; machine
envelope locale-invariant; welcome note untouchable), SYNTAX_MORPH_SPEC.md (bijective keyword/char
skins incl. AI token-minimizing profiles; canonical-form law: AST/hashes/artifacts always canonical;
identifiers never morph; delulu morph lifecycle), AI_NATIVE_DESIGN.md (machine-side: awareness via
authority/why JSON, repair loop, host-side-checks=free hot path, holder-model multi-agent, 6 standing
commitments, no holder-kind branch anywhere). (3) docs/lang/: README + en-US.md (COMPLETE reference:
concept vocabulary, canonical keyword enumeration for morphs, catalog key space) + delulu-slang.md
(COMPLETE voice guide) + 9 STARTERS with terminology/register/script decisions locked (zh-CN ja-JP
ko-KR double-width, hi-IN grapheme clusters, ar-SA reference RTL/bidi, fr de es pt) — Opus completes
catalogs mechanically. (4) docs/book/THE_DELULULANG_BOOK.md — first complete edition, 19 chapters + 3
appendices (why exists/authority/effects, reading authority, plugins, actors, humans+AI one law, For
Machines, foreign honesty, custody, migration Python/Rust/JS/Go, philosophy, honesty clauses, road
ahead, welcome note, glossary, spec map). (5) Cross-refs: STAGE8 spec §6 → localization docs;
REPOSITORY_STRUCTURE.md updated (also repaired a truncation from an interrupted edit). NOTE: several
docs/ files were written across user /model interruptions — if a docs file ever looks truncated,
check git and re-read before editing. NEXT BUILD WORK (Opus): Stage 4 per STAGE4_PLAYBOOK.md.

**STAGE 4 "FOREIGN" COMPLETE (2026-07-10/11, commits c7b043d→c4698fe + close-out).** Built via the
orchestration pattern Jesse mandated: Fable 5 head chef briefs Opus 4.8 subagents per chunk (exact
file pointers + env traps + head-chef rulings in every briefing), then live-verifies acceptance
criteria against the real binary before approving. Worked flawlessly; twice a subagent hit Jesse's
session limit mid-chunk and was resumed via SendMessage with context intact — uncommitted work
survives in the tree, so: on limit-kill, check git status, then SendMessage the same agent to finish
(don't respawn). Results: 200→271 tests. Key facts: foreign fence (DL1301 empty-repairs/no-expose,
DL1302 R-6a incl. nested), root.foreign_load() minting ruling (Book ch.14 name), C FFI via
libloading+libffi (bind-time fail-fast; rustc-built-cdylib test fixtures — portable, no cl.exe
dependency; msvcrt.dll for Windows cos), CPython 3.13.5 + NumPy 2.2.6 already on machine (PyO3
0.25.1, feature `python` default-on; examples/numpy_mean.delulu → mean=2.5), WASM parity via
delulu:foreign@0.4 host fns (traces byte-identical across engines; Python-on-WASM = DL1201
interpreter-fallback because WASM fragment lacks List values). Honest caveats recorded in spec §11
close-out: 3-OS CI matrix NOT yet run (Windows only), foreign .dwx refuses cleanly at run (no
embedded signatures — Stage 6 extension), pre-Stage-4 .dwx artifacts invalidated by span-carrying
host-fn signature change (accepted pre-1.0). NEXT: Stage 5 Custody per STAGE5_PLAYBOOK.md (new crate
delulu-broker; prove tree/lattice/audit/epochs embedded-mode first behind a Custody trait).

**CROSS-OS STATUS (2026-07-11, commits 5269f48 + 1393dd5):** Linux VERIFIED green (Docker rust:1,
Python 3.13.5 = same minor as Windows, NumPy 2.2.4; Docker Desktop exists on Jesse's machine — use
`bash -c` not `bash -lc` in rust images, script-file + `tr -d '\r'` to dodge PS quoting).
`.github/workflows/ci.yml` committed (3-OS matrix, Python pinned 3.13) but repo has NO remote —
**Jesse explicitly said do NOT push to GitHub yet** (he decided 2026-09-14: pushed to his PRIVATE testing repo — [[delulu-github-remote]]); macOS testing deferred until he decides
(macos-latest runner is the sanctioned path; no macOS VMs on non-Apple hardware). macOS statically
hardened: fixtures use std::env::consts DLL naming, 3-OS cos test mirrors (libm.dylib/libm.so.6),
grant-parse split_once tested against drive-letter paths, deps all Darwin-supported (libloading/
libffi-sys/pyo3 0.25 tier-1/wasmtime), pure-Rust crates cargo-checked clean for x86_64-apple-darwin
(aarch64 + native crates blocked only by missing Darwin C toolchain on Windows — expected, CI-covered).
Branch is `master` (kept; whole project history lives there).

**STAGE 5 "CUSTODY" IN PROGRESS (2026-07-11/12).** Same orchestration pattern (Fable 5 head chef
briefs Opus subagents per chunk, live-verifies before approving). New crate `delulu-broker` =
transport-free custody core. CHUNK 1 (phases 5a-5c) COMPLETE + head-chef-verified (commits ade26e8/
b2e2081/e2b507a, 271→305 tests): src/authority.rs = ⊑ attenuation lattice (effects ⊆; fs paths
pure-lexical descendant-or-equal in src/path.rs — NO fs access, `./database`⋢`./data`, `..` escapes
refused, `\`→`/`, case-sensitive; net/secrets/declassify/foreign_c/foreign_python EXACT-string
name-set ⊆), attenuation_check returns computed INTERSECTION that never widens = DL0802's exact
repair; src/tree.rs = Broker{nodes,epoch,audit_seq} + Node per §3.1, GrantId=g_+32hex from getrandom
behind IdSource trait, issue/attenuate/revoke(caller-or-descendant only, transitive, idempotent,
bumps epoch)/inspect/tree, Holder{kind,desc,peer} STORED-DISPLAYED-NEVER-SWITCHED-ON (criterion 9:
byte-identical outcomes across holder.kind human/process/delegate + a grep-test over src/*.rs that
fails on any `.kind` branch outside KIND_IS_DATA-marked display); src/validate.rs = Op classes
(synchronous Declassify/FsWrite/Net/ForeignBind validate live per-use; epoch FsRead/Clock/Rand/
Console validate vs a Snapshot), DL1403 carries revoking seq, DL1402 TTL vs pluggable ClockSource,
no-sleep tests. DL14xx registered in delulu-diag (DL1401/1402/1403/1405/1406/1407/1408, deliberately
NO DL1404). Head-chef rulings: structured payloads live on broker-native Denial enum (not the shared
Diagnostic, mirroring Stage-4 Fault); topology/scope denials map to existing DL0904; timestamps =
epoch-millis in core (ISO is display). CHUNK 2 (phases 5d-5e) COMPLETE + verified (commits ced68ce/
d9aa1b2, 305→333 tests): src/audit.rs = append-only hash-chained log, record{seq,ts,prev_hash,hash,
actor_node,action,target,authority?,span?,decision}, hash=blake3(prev_hash‖canonical_record) (hash
field excluded, keys sorted, genesis=64 zeros), one YYYYMMDD.jsonl per UTC day under an INJECTED dir,
every file's first line = "observability, not enforcement" verbatim, day heads cross-linked; AuditSink
trait on Broker (optional, default none = chunk-1 behavior byte-for-byte); verify→DL1405 at the
corrupt seq; `delulu audit tail|query|verify` CLI. src/lease.rs = MAC-signed portable tokens
dlt1_<hexpayload>.<hexmac>, payload{v,node,exp_millis,multi,nonce}, mac=blake3::keyed_hash(broker_key,
payload) (RULING: no hmac/sha2 crates), constant-time compare via blake3::Hash PartialEq; broker key
random 256-bit at injected path (0600 on unix); delegate=attenuate+mint, redeem binds to peer
single-use (2nd=DL1407 unless multi), expired=DL1402, rotate-key invalidates outstanding tokens; all
5 token-attack tests present. Only new dep across both chunks: blake3 (workspace-matched). CHUNK 3
(phases 5f-5g, Custody trait + IPC daemon/client + broker secrets) IN PROGRESS — sous-chef built far
before session limits + 2 BSODs (all WIP survived in tree/stash every time): delulu-runtime/src/
custody.rs (Custody trait: check/expose/refresh_epoch/mode; EmbeddedCustody=pass-through so prior
suite passes UNMODIFIED = criterion 11), delulu/src/broker_ipc.rs (length-prefixed canonical CBOR
frames via ciborium, WIRE_VERSION "broker/1", mismatch→DL1406), broker_transport.rs (Windows named
pipe owner-only DACL + GetNamedPipeClientProcessId/EqualSid peer check; unix UDS mode-0700 written
cfg-gated for CI), brokerd.rs (daemon owns Broker+AuditLog+secrets, `delulu broker start|stop|status`),
broker_client.rs (BrokerClientCustody), delulu-broker/src/secrets.rs, delulu/tests/broker_cli.rs.
MUST-HOLD for chunk 3: FAIL CLOSED (invariant 27 — broker unreachable⇒DL1401, NEVER silent embedded
fallback; dedicated kill-daemon test); NEVER Err from a Wasmtime host callback (record refusal in
HostState + surface after, carried trap); broker-held secrets = bytes cross only on expose (Stage-3
host secret table becomes client cache of handles, criterion 6 memory-scan extended to host cap
cache). Task-tracking: TaskList #18/#19 done, #20 in progress, #21 (5h foreign workers) #22 (5i-5j
CLI+microVM close-out) pending. Baseline before chunk-3 completion = 333 tests. See [[laptop-bsod-mitigation]].

**STAGE 5 CHUNK 3 (phases 5f+5g) COMPLETE + head-chef-verified (2026-07-12, commit f8b6a25, 333→354
tests).** Single combined commit (5f/5g interleaved: the daemon handler serves both custody checks and
secret exposes). Custody trait seam in delulu-runtime called by BOTH interpreter and WASM host;
EmbeddedCustody pass-through keeps the whole prior suite unmodified (criterion 11). BrokerClientCustody
(daemon mode): synchronous ops round-trip per use, epoch ops validate a client-cached Snapshot
refreshed ≤ --epoch-ms (clamp 1..250, default 50). FAIL CLOSED proven (invariant 27): unreachable
broker ⇒ DL1401 fast with exact start cmd, no embedded fallback path exists, stale snapshot discarded
before round-trip. Transport: Windows named pipe owner-only DACL + EqualSid peer check / Unix UDS
0700; CBOR broker/1, mismatch DL1406. Daemon (brokerd): one Broker+AuditLog+SecretStore per user,
delulu broker start|status|stop|rotate-key, fail-stop on un-appendable audit record. 5g: SecretStore
broker-side, expose = synchronous declassification (bytes cross only here, audited w/ span), secret_map
broker-side, host cap cache holds handles not bytes (criterion 6). **HEAD CHEF FOUND+FIXED 3 REAL
transport/daemon bugs the in-process unit tests could NOT surface (only the real-binary integration
test did) — these are the key lessons for chunk 4+ daemon/process work:** (1) named-pipe RECONNECT RACE
— single-instance server re-creates the pipe between requests; a client reconnecting in the sub-ms gap
gets ERROR_FILE_NOT_FOUND. Fix: connect() retries FILE_NOT_FOUND within a short bounded window (down
daemon still fails closed fast). (2) DETACHED DAEMON INHERITED PARENT STDIO — Rust spawns with
bInheritHandles=TRUE, so the daemon held copies of the launcher's stdout/stderr pipe → any caller using
Command::output()/shell pipe/CI HANGS forever waiting for EOF. Fix: clear HANDLE_FLAG_INHERIT on the
parent's std handles (windows-sys SetHandleInformation, added Win32_System_Console feature) before the
detached spawn + redirect daemon's own stdio to <state>/broker.log. (3) DETACHED DAEMON LOCKED CALLER
CWD — inherited working dir. Fix: cmd.current_dir(state_dir). ALSO test-harness fix: DaemonGuard::drop
must run from a stable cwd (temp_dir), best-effort, never panic (a Drop panic masks the real assertion).
ALSO criterion-11 fix: authority report custody label is in JSON always but human-rendered only when
non-default (daemon), keeping default embedded report byte-identical. LESSON: always run the real
process-spawning integration test live + a per-step timeout diagnostic; unit tests that call serve() on
a thread miss inherited-handle and detached-process bugs entirely.

**STAGE 5 CHUNK 4 (phase 5h foreign workers) COMPLETE + head-chef-verified LIVE (2026-07-12, commit
8491a16, 354→360 tests).** --foreign-isolation process|inproc (process = default in daemon mode, inproc
= Stage-4 in-proc, labeled in output). Each granted C lib runs in a worker SUBPROCESS = the same delulu
binary re-invoked as hidden `__foreign-worker`, speaking Stage-4 marshalling (FVal scalar set only:
Int/Float/Bool/Str/Unit/Ptr) as CBOR over a PRIVATE per-worker pipe (reuses broker_transport
Listener/connect — worker is server, host is client so the host's wait is bounded). New:
delulu/src/foreign_worker.rs, delulu/tests/foreign_worker.rs; foreign.rs gained ForeignErr::WorkerDied +
BoundForeign/ForeignBinder/InProcBinder seam; DL1409 registered. **Criterion 7 PROVEN LIVE by head chef
against the real binary:** a null-deref C fn under process isolation → host exits code EXACTLY 1 (clean
DL1409 "foreign worker died... the host survived", caret on the call site), NOT an OS crash code; same
worker pipe runs dl_add(20,22)=42 (marshalling round-trips). Isolation mechanics — Windows: Job Object
JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE (windows-sys Win32_System_JobObjects, no new dep) + SetErrorMode
(silence WerFault, CI-safe) + cleared HANDLE_FLAG_INHERIT + null worker stdio + stable worker cwd; Unix
(compiled, CI-only — Windows dev box): prctl(PR_SET_PDEATHSIG,SIGKILL) via pre_exec (new unix-only libc
dep, already in Cargo.lock transitively). The SEGFAULT→WorkerDied mechanism: worker crash sends no
response → host's bounded read_frame returns EOF → WorkerDied, host keeps running (crash is in a
separate process). Head-chef ACCEPTED deviations: (1) Python worker NOT isolated — a live PyObj is
process-local, unmarshallable; Python stays in-proc even under process mode (documented like Stage-4
Python-on-WASM DL1201); C worker is the criterion-7 proof. (2) WASM foreign path stays in-proc
(foreign .dwx already unsupported; interp is the reference). (3) criterion-7b worker-cannot-reach-broker
is NON-DISCLOSURE (no broker addr in argv, DELULU_STATE_DIR → isolated broker-less dir, no inherited
handle), NOT kernel enforcement — consistent with §10 same-user threat model; proven meaningful by a
contrast probe (broker-less dir → unreachable; real broker dir disclosed → reachable). (4) Linux seccomp
= documented stub (PDEATHSIG is the must-have). Sous-chef APPLIED all chunk-3 process/handle lessons
correctly (null stdio, bounded connect/read, non-panicking WorkerGuard::drop from stable cwd, reused
clear_std_handle_inheritance). NEXT: chunk 5 = phases 5i (microVM, DL1408 platform-pending on non-Linux
+ isolation matrix table) + 5j (CLI surface: grants list|tree|inspect|revoke|delegate, run --lease,
audit polish, explain E-REVOKE §4.2 bound verbatim, custody label) + Stage-5 acceptance-criteria
close-out in spec §11 (task #22). This is the FINAL chunk — after it, Stage 5 Custody is done.

**STAGE 5 COMPLETE (2026-07-13, commit 2b6a81a, 360→372 tests, 0 failed, 2 ignored).** Chunk 5
(5i+5j+close-out) delivered by the resumed Opus agent (survived TWO session-limit kills; 1216 lines
survived on disk mid-chunk — resume-same-agent procedure works even mid-build). 5j: `delulu grants
list|tree|inspect|revoke|delegate` over UNCHANGED broker/1 (additive List/Inspect+Listed/Inspected
pairs, boxed NodeInfo); every verb daemon-down = DL1401 + exact start cmd; `run --lease <token>`
redeems (DL1407/1402/1401 pre-main) → inspects own node → derives local grants from node authority
(never widened; only --grant foreign.c may accompany) → BrokerClientCustody::for_node (dead_code
allow removed); `explain E-REVOKE` = new topic_explain in delulu-diag, §4.2 bound VERBATIM, all
DL140x explain bodies carry §10 caveats; §4.2 bound rides every successful revoke's audit record
under "revocation_takes_effect". 5i: `--isolation none|process|microvm`; microvm = DL1408
EVERYWHERE v0.5 runs (non-Linux platform refusal; Linux src/microvm.rs probe — Linux-only compile,
pure std — names missing prereq /dev/kvm|VMM|pending guest launch; NOTHING weaker launches under
the name); process = 5h workers, labeled "explicitly weaker"; isolation matrix = normative TABLE
§6.1. Criterion-8 contract test gated cfg(delulu_kvm) (registered check-cfg, enable
RUSTFLAGS="--cfg delulu_kvm" on KVM CI — NOT bare linux, a KVM-less runner would run red).
Criterion-2 measured: revoke observed ≤150ms (50ms epoch ×3 CI jitter) over real transport.
Orchestration test grants_cli.rs = full story (delegate→lease run→DL1407 second use→DL0802
widening w/ intersection→tree/list/inspect→transitive revoke→DL1403 w/ seq→audit verify→DL1401).
Spec §11: 11-criteria close-out table (1-7,9-11 met w/ named tests; 8 platform-pending) + 9 loud
deviations (key: widening refused at MINT not redemption — token never exists; delegate fs scopes
absolutize against minting cwd). README row BUILT 2026-07-13. **HEAD CHEF VERIFIED ALL LANES
PERSONALLY: (1) Windows full suite 372/0/2; (2) live 11-step demo vs real binary (scratchpad
verify_chunk5.ps1 pattern) — DL0802 intersection, §4.2 bound wording, DL1403+seq, DL1408 honesty,
DL1401 fail-closed all exact; (3) Linux full suite 372/0/2 IDENTICAL on real Ubuntu 20.04 (WSL) —
first-ever live run of grants CLI over UDS + SIGSEGV foreign workers; (4) clippy: 4 warnings in
touched files all blame to pre-chunk-5 commits — zero new.** NOT pushed (standing order).
**CROSS-OS LANE (durable): WSL Ubuntu-20.04 on Jesse's box now has rustup stable 1.97 minimal +
make git python3.8-dev; repo cloned ~/DeluluLang (ext4, NOT /mnt/d — WSL perf); refresh = `git
fetch origin master && git checkout <sha>` (origin = /mnt/d/nelan/DeluluLang), then cargo test.
Windows→Linux/mac `cargo check --target` is BLOCKED by libffi-sys host-cfg build-script bug (picks
msvc.rs from HOST not target) — native WSL build is the only honest Linux lane. Docker rejected
(rides on WSL2 anyway, adds overhead, zero quality gain).** NEXT: Stage 6 "Live" (plugins) when
Jesse orders it.

**macOS PORTABILITY BUG found+fixed post-close-out (2026-07-13, commit 26d910b).** Jesse asked
"what about mac os" → head chef found phase-5h's set_pdeathsig was cfg(unix) but libc::prctl/
PR_SET_PDEATHSIG are LINUX-ONLY in libc (not Apple/BSD) ⇒ delulu could not compile on macOS.
PROVEN via minimal scratchpad crate `cargo check --target x86_64-apple-darwin` → E0425 (targets
x86_64/aarch64-apple-darwin already installed in rustup). Fix: call site + fn + the libc dep all
narrowed to cfg(target_os="linux"); macOS worker reaping = WorkerGuard explicit kill (portable
std); kqueue EVFILT_PROC documented as possible future hardening. Verified: delulu suites green
Windows + WSL Linux (pdeathsig path re-exercised); delulu-syntax/diag/check/BROKER (whole custody
core incl. blake3) blind-typecheck green for x86_64-apple-darwin; runtime/bin/wasm not cross-
checkable from Windows (libffi-sys/C build scripts) but remaining unix code is pure std, mac-clean
by inspection. Spec §11 deviation 10 records it. **LESSON: `cfg(unix)` ≠ portable-Unix — any libc
call must be checked against libc's per-OS coverage (prctl, epoll, pidfd etc. are linux-only);
cheap proof = tiny scratchpad crate + cargo check --target x86_64-apple-darwin. No live macOS lane
exists (no Apple hardware; CI = push, barred).**

**STAGE 5 CHUNK 6 "the GUARD" BUILT 2026-07-14 (commits 28fa620 5k → dbbcd46 5l → 5f4306b 5m →
6dd5a8a close-out; tests 372→398; head-chef live-verified 49-check demo + Windows AND WSL-Linux
full suites both 398/0/2 + clippy clean in guard files + custody-core mac blind-typecheck green).**
Jesse pointed at github.com/Dicklesworthstone/destructive_command_guard (dcg, Jeffrey Emanuel) and
asked to incorporate it into Delulu Authority. Head chef inspected the clone, ADOPTED dcg's policy
concepts (graduated tiers, allow-once approvals, explain mode, awareness banners, audit-everything,
bypass-with-banner) and REJECTED its string-scanning engine (Delulu effects are typed — zero
parsing), its fail-open stance (guard is authority boundary ⇒ fail closed, invariant 27), and
per-agent trust profiles (criterion 9: tree POSITION not identity — root=principal, delegated=
agent). Spec = docs/design/STAGE5_GUARD_ADDENDUM.md (11 criteria all met §8; 8 deviations §7 all
head-chef approved). Model: broker-held policy rules class:pattern→warn|guarded|sealed (defaults:
declassify/foreign_c/foreign_python guarded); owner code gow1_ print-once memory-only rotates per
daemon run gates admin verbs (DL1414); agent escalation `delulu guard request --why` (required) →
principal approve/deny --comment (deny comment REQUIRED, carried back VERBATIM in DL1412); permits
broker-held daemon-memory TTL 15m; DL1410 names exact request command; DL1411 pending id; DL1413
sealed (survives bypass); bypass = --dangerously-bypass-guard or owner-gated toggle, always prints
the why-it's-bad banner, everything still audited (guard_bypassed_use); guarded/warn-ruled classes
route epoch ops synchronously (deviation 4); secret/foreign_python gate at MINT only in v0.5 (no
per-use wire op — deviation 8). Opus 4.8 chef implemented all 3 phases (one limit-kill mid-5m,
resumed same agent, completed clean). E2e = crates/delulu/tests/guard_e2e.rs; my demo script =
scratchpad verify_guard.ps1 (gotchas: Out-String wraps at console width — flatten before exact-
string matches; root runs need explicit --grant fs.read/fs.write; WSL default user is `user` not
jesse, clone at /home/user/DeluluLang; MSYS path-conversion mangles wsl args — pull files via
PowerShell).

## SURFACE EARLY-DROP: the Atlas & the Palette (BUILT 2026-07-15, head-chef verified)

Jesse's order 2026-07-14: build a code-structure graph "like [the banned word]" usable by EVERY consumer
(human/agent/LLM/tools) + configurable CLI colors — but IMPROVED, and (mid-build ruling) the word
"[the banned word]" must NEVER appear in repo/product surfaces (see [[no-banned-word-mentions]]). Head chef
evaluated the tool's repo (32k-line Python: tree-sitter+optional-LLM extraction, Leiden
communities, confidence tags), ADOPTED the query-instead-of-grep insight/verbs/god-nodes/
self-contained-HTML and REJECTED text extraction + confidence tags (compiler HAS ground truth),
LLM passes, statistical communities (packages/modules ARE the communities). Spec =
docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md — Stage 8 "Surface" material early-dropped, DL1780/
1781/1790 from the DL17xx range (DL1700–1779 left for main Stage 8; no DL1784 ever).

Commits: 0a58451 spec → f4ffd01 anonymize (owner ruling) → 5c364dc A1 Palette → 13fa61a A2 Atlas
core → 863f824 A3 renderers+custody → cd5ff20 A3.1 → 07de40a A3.2 → 3896c1a close-out. Tests
398→461 (0 fail, 2 pre-existing ignored; criterion 10: zero pre-existing tests edited). Verified:
53/53 live demo checks (scratchpad verify_atlas_palette.ps1), Windows 461/0/2, WSL Linux 461/0/2
byte-identical, new code clippy-clean. WSL clone synced to 3896c1a. NEVER pushed.

THE ATLAS (`delulu atlas`, crate delulu-atlas on delulu-check): typed deterministic code+authority
graph from compiler facts ONLY (FnFacts.callees + Program.call_owner reused — no checker change).
atlas/1 versioned JSON; formats tree/digest/json/dot/mermaid/self-contained-html (zero external
URLs, >3000-node collapse); query verbs node/path/callers/calls/why (+--budget, explicit
truncation); digest = token-budgeted ATLAS.md ending in fixed "Querying further" footer (default
2000 tok, chars/4 heuristic, irreducible floor = title+notice+footer); --out DIR writes agent
bundle, query verbs answer from persisted atlas.json; performs/requires edges mirror `delulu
authority` exactly (machine-checked parity); foreign:c:/foreign:py: nodes + ForeignCall-gated
edges; --custody overlay grafts grant: nodes/delegates edges via read-only broker List (broker
down ⇒ DL1781 note, atlas still emitted; check errors ⇒ DL1780 refusal, NO partial graph).
E-ATLAS explain topic.

THE PALETTE (delulu-diag/palette.rs, ZERO new deps — hand-rolled ANSI + in-tree windows-sys VT):
13 roles, themes default(colorblind-safe)/bright/mono(zero color SGR); precedence --color >
DELULU_COLOR > NO_COLOR > TTY-auto with NO_COLOR beating DELULU_COLOR=always but NOT explicit
--color always; ~/.delulu/theme.toml [roles] overrides (DELULU_THEME_FILE override; bounded 64KiB
hardened reader); bad theme ⇒ DL1790 warn + default fallback; --json NEVER colored; E-PALETTE.

HEAD-CHEF LESSON (proved its worth twice): the chef's §8 claimed all-met, but live verification
caught (1) foreign nodes/edges defined-but-never-populated (deviation 4 promised them; grep
build.rs for the feature, don't trust model definitions) and (2) digest ignoring --budget (its
witness only covered query verbs; test the exact flag on the exact surface). Both fixed same-day
in A3.1/A3.2 with binary-level witnesses. Verify by running the REAL binary, not by reading the
close-out table.

## STAGE 6 "LIVE": runtime plugins (BUILT 2026-07-18, head-chef verified, HEAD 4b273a2)

The flagship stage — Constitution §4 Possibility 2: code that arrives AFTER compile time and
still cannot exceed its grant. Spec docs/design/STAGE6_SPECIFICATION.md (+§11 status log),
playbook docs/playbooks/STAGE6_PLAYBOOK.md (Jesse's, made binding mid-build), build order
docs/design/STAGE6_BUILD_ORDER.md (8 ruled deviations §3, close-out table §4, post-v0.6 RFC
ledger §5), user guide docs/design/STAGE6_PLUGINS_GUIDE.md, flagship example examples/plugin_shout/.

Two plugin classes, class DECLARED never inferred (DL1504 never falls back to Contained):
Verified ships DIR (post-check typed AST, canonical CBOR via ciborium; dir::verify re-runs the
EXACT check_source pipeline + stored-truth comparison — deviation 1, strictly stronger than the
spec's assert-replay; ~0.75ms/load) and runs on the host's engines; Contained ships opaque WASM,
imports ⊆ grant-derived slice (DL1505, name AND type, wasi/root_* never in any slice), every
export typed at effects(grant) (R-1). 7-step load pipeline (order witnessed: DL1502 ceiling
before DL0802 holder/attenuate child node), p.get R-Get per class, R-6a fail-closed (DL0803 +
NEW DL1509 for underdetermined F), R-6b call-scoped handle table (monotonic never-reused ids —
aliasing would SUCCEED, worse than dangling), R-6c unload/reload (DL0801 + revoking audit seq),
R-7 three-level composition, both custody modes. Limits: Wasmtime fuel/mem/wall with EVIDENCE-
BASED trap attribution (TrapCause; a bug-trap is never DL1506 because DL1506's repair is
authority_widening — mislabeling would advise widening authority to fix a bug); interpreter
limits best-effort and labeled (§5.4). ed25519 signatures (DL1510 badly-signed ≠ DL1511
unsigned-but-required). plugin build/inspect/verify CLI (verify ≡ load, one code path, corpus-
tested); authority "plugins" array; why traverses Verified DIR, labels Contained boundaries;
E-PLUGIN. DL15xx codes DL1501–DL1511; DL0801/DL0803 activated.

WINDOWS CAVEAT (deviation 7, ruled): wasmtime-27's host-initiated trap unwind (fuel/epoch)
__fastfail's the process on Windows (bisected; 3 Config candidates exhausted incl.
wasm_backtrace(false) + signals_based_traps(false)). v0.6: Contained EXECUTION on Windows is
refused honestly (EnforcementUnsupported, safe by construction — the path doesn't compile on
Windows) rather than half-contained (memory-only would hang, not contain). Enforcement-grade
platform = WASM-on-Linux (spec's own §5.4 designation); Verified runs everywhere via the
interpreter. Preferred fix = future wasmtime bump; out-of-process Job Object = RFC fallback only
(fuel has no subprocess analogue; R-6b/R-Get across a process boundary needs a ruling first).
macOS: compiles the full enforcement path, EXPECTED to work, UNVERIFIED (no Mac in the kitchen)
— docs say exactly that. In-language load surface is a v0.6 stub (root.plugin_host() → DL0703 at
runtime; checker fully types load/p.get/Plugin[C]); full `delulu run` load integration is future
work.

Commits dc39a2a/5b4c687 orders → 8214633 6a → fbee563 6b → 6a8905d 6c → 13e1624 rulings →
88e1f36 6d → 6f7f4bc/8ea7fc4 6e.1-2 → 06c6338 R-6a FIX → 33f08d1/348798a 6e.3-4 → e65d6ba 6f.1 →
e44077e 6f/6g → 85dfac4 §11 → ae97611/74a1693 6f.2 → c48c7b3 6f.2b → 525b1f2 dev7 ruled →
2eecc0f 6e.5 → b234953/e298a48 6h → 5fc7e50/f241b60/4133ede 6i → c07ecfc B4 → faa6dae F1 →
4b273a2 close-out. Tests 461→609 Windows / 613 Linux (0 fail; +4 = platform-gated live-engine
criterion-5 witnesses, each seen green by name on Linux at faa6dae). Flagship exercised against
the real binary (rigged variant refused DL1501, zero partial artifact). New code clippy-clean.
Two sous-chefs (both Opus 4.8): the first died at a user-stop after 6f.2b (respawn was forced —
resume refused; the committed docs made the cold handoff cheap), the second finished 6e.5→F1.
Stage 6 survived ~6 interruptions (session limits, 529s, user stop) with ZERO lost work — commit
per sub-phase + hand-off-at-clean-tree discipline. WSL clone synced to 4b273a2. NEVER pushed.

THE BIG SECURITY CATCH (see [[skip-branch-verification-rule]]): R-6a/DL0803 was FAIL-OPEN —
unpinned p.get and generic laundering both escaped with zero diagnostics (generics instantiate
fresh per call site, so a helper's T never unifies with the caller's closure). Head chef caught
it by adversarial read of the skip branch; sous-chef CORRECTED the head chef's proposed fix
(F *is* a concrete Fn in the laundering case — the fault is the Var INSIDE it) and shipped
type_contains_var + DL1509. The kitchen rule ("write the couldn't-tell case first") then caught
more: handle-id aliasing, root_* in no slice, DL1505 name-AND-type, trap attribution, DL1510/11
split. Rule every deviation; escalation-not-thrash worked every time it was used.

STAGE 7 "Concurrent" (2026-07-18, head chef Fable 5 COOKED IT DIRECTLY per Jesse's order; one
Opus 4.8 sous-chef for 7h only). Actor model + Pony rcaps (iso/trn/ref/val/box/tag), Async as
an effect in the same rows — one system, no await (rejected, not deferred). The rcap axis
lives BESIDE Type (deviation 4 — unify.rs untouched; DIR stable): rcap_check.rs is a second
pass over settled node_types with ONE centralized use path (consume flow, recover/lambda
boundaries un-dodgeable). Eleven ruled deviations in STAGE7_BUILD_ORDER.md §3 — key ones:
3 worker-owned actors (pinned at spawn, ZERO unsafe anywhere; messages cross only as owned
Send MsgValue, per-sender FIFO free from mpsc); 7 iso moves by rebuild (observationally the
spec's handoff; --debug-rcaps verifies checker-exported iso_moves, DL1610); 9 fn-typed
positions default box not ref (else every pure-lambda arg refuses); 10 T-Send charges a site
only for row tails its own args can bind (Promise.fulfill stays {Async} — the spec's own
"row e joins THEN's send row"); 11 std.actors Promise[T,e] = ONE canonical source
(STD_ACTORS_SRC) parsed+registered by both resolve and Interp (verify≡run).
Commit chain: ca17874 build order → 668dd18 7a → afc7c02 7b → b649a33 7c → c6eec8b 7d →
32ee5d1 7e → 2896462 7f → f0130e3 7g → f5ea0fc 7i → 5aaebd0 7j → 1fe4895/c09c5df/fd99c5e
close-out 1-3 → 7h merge → 8571eae 7h CLI finish. Windows 726/0 at the 7h merge.
CRITERION 1: 1,000,024 turns EXACT at 1 and 4 workers, 3.00x at 4. CRITERION 8 SHIP-GATE:
10k seeded object-sensitive worlds, zero counterexamples — the gate caught TWO of the head
chef's own model bugs first (viewpoint aliases the FIELD's object; a retained field read is
alias(o▷f)) — the kitchen rule works on models too. Criterion 6 both engines incl. the
real-binary cooperative-label smoke (6 turns exact). The 7h sous-chef was interrupted TWICE
(session limit + process exit); worktree survived both; head chef completed its foundation
(early subset gate — the late gate panicked on an uninterned literal; string collector's
Stage-7 arms). CLOSED OUT at the BUILT commit after 8571eae: Linux 730/0/2, TSAN ZERO warnings (10/10 under -Zsanitizer=thread, 1M-msg pingpong 280s clean), all ELEVEN criteria witnessed, build order flipped BUILT. Two-account relay protocol
established there too: shifts, never parallel stages.

STAGE 8 "Surface" BUILT 2026-07-18 (HEAD f9ac0ab→final, Windows 800/0, WSL Linux 807/0/4 —
+7 platform-gated). The language's FACE:
terminal-first for agents+humans, one LSP for every editor, one canonical formatter, an
authority-isolated test runner, two voices, Jesse's welcome, signing+registry groundwork.
Invariant 38 (no semantics in tooling) held throughout. Phases 8a-8f cooked by Fable 5;
8g-8h + close-out by Opus 4.8 after Jesse switched the session model mid-stage (attributed
honestly in commits — Fable 5 co-author on 8a-8f, Opus 4.8 on 8g-8h/close-out). Commit
chain: 3cc22d0 build order → 1e05cdd 8a → 9bb7758 8b → 4d03331 8c → b1a3034 8d → 3e72665
8e.1 → 899497d 8e.2 → 2ba91c6 8f → 05a0ccf 8g → 0b34ff8 8h → f9ac0ab close-out. Ground
truth docs: STAGE8_BUILD_ORDER.md (16 ruled deviations §3, close-out table §4 all 10
criteria MET with named test witnesses), STAGE8_SPECIFICATION.md §12 status log per phase,
STAGE8_SURFACE_GUIDE.md (§11 caveats VERBATIM, meta-tested surface_guide.rs), docs/editors.md.
KEY DEVIATIONS: 3 LSP is a CLI module ZERO new deps (hand-rolled JSON-RPC, value-level
serde_json, UTF-16 positions); 7 en-US lives in CODE (missing key = compile error, literal);
8 JSON fully locale-invariant incl. message text (envelope API can't see a catalog); 9
catalog = hardened zero-dep TOML subset, every defect DL1704+fallback; 11 fmt laws =
span/id-stripped fingerprint + (text,own_line) comment sequence, verified INLINE pre-write,
DL1702 fail-closed; 12 Stage-7 pingpong bar re-ruled 2.0x→1.5x best-of-two (thermal: 1.91x
warm vs 3.00x cold record, semantics never flaky); 13 locale add zero-auth gate at ADD; 14
test grants derive from declared row bounded by [test-authority] ceiling (pure test holds
NOTHING, invariant 41); 15 signing DETACHED <artifact>.sig (Stage-6 96-byte format
generalized to .dwx/.dpx/tarballs, ed25519 in delulu-runtime, getrandom reused). NEW CODES
DL1701-1707 (+1707 assert panic; DL1784 still never). NEW DEPS: serde_json→delulu-syntax
(fmt fingerprint, already workspace-wide), getrandom→delulu-cli (keygen, already workspace).
DL17xx precedent: Atlas/Palette (DL1780/81/90) already BUILT 2026-07-15. THE 100k FMT GATE:
delulu-syntax::fmt::tests::fmt_laws_100k_gate ran+passed (54s release, 0 violations).
examples/ CANONICALIZED by fmt (identity-preserving — cli.rs byte-exact demo authority pin
still holds); `delulu fmt --check examples` now a permanent CI gate (criterion 10).
LESSON (Opus 4.8, 8g): the broker tree starts EMPTY — a test-session is a fresh ISSUEd
PRINCIPAL, not a child of an ambient root; revoked nodes stay in `grants tree` marked
[revoked@seq], not erased. LESSON (WSL): PowerShell EXPANDS $HOME/$PATH/$? inside
double-quoted wsl commands (→ C:\Users\jesse) corrupting the PATH — use SINGLE quotes for
`wsl -e bash -lc '...'`, and the login shell already has cargo (/home/user/.cargo/bin), so
DON'T re-export PATH. macOS: Stage 8 adds NO new platform gates beyond a cfg(unix) 0600
keyfile chmod (macOS satisfies natively) — expected to work by construction, no Apple
hardware to verify (same honest stance as every prior stage). NEVER pushed.

## Stage 9 "Delulu" — BUILT 2026-07-19 (HEAD 7acc354). **v1.0 NOT RELEASED.**

Nine phases 9a-9i, all committed. **ATTRIBUTION (corrected, ruling D21): MOSTLY OPUS 4.8, NOT
Fable 5.** Fable 5 wrote the build order (93afcaf) and opened 9a; the session model was switched
to Opus 4.8 partway through 9a and Opus 4.8 cooked the rest of 9a (commit 335a875) through 9i and
the close-out (2c59af6). Model switched back to Fable 5 only for the post-hoc verification pass.
**All ten Stage-9 commit trailers say `Co-Authored-By: Claude Fable 5`, which is FALSE for the
Opus-4.8 span** — the head-chef persona kept signing "Fable 5" without checking the running model,
exactly the honesty defect this project treats as unforgivable. Stage 8 handled the same switch
correctly (8g-8h attributed to Opus 4.8); Stage 9 did not. Trailers left as-is (history not
rewritten, unauthorized+destructive); the errata is D21 + the spec status note. (A sous-chef agent
was launched for 9a and pulled off after Jesse ruled it slower/costlier than the head chef; its
scaffold was kept and rebuilt.) Windows 916/0/5. Docs: STAGE9_BUILD_ORDER.md is the
operational ground truth (rulings D1-D20 + the criteria table); STAGE9_SPECIFICATION.md has
the Implementation status section with the per-phase commit map.

**THE HEADLINE: the acceptance gate RETURNED TWO BLOCKERS and the version says so.**
Version is `1.0.0-rc.1`, NOT 1.0.0. docs/release/CHECKLIST-1.0.md says "1.0 DOES NOT SHIP
YET" in those words. Criterion 1 = coverage 273/290 (94.1%), 17 anchors classified in D10.
Criterion 8 = fresh-machine first-run never performed or timed on any OS. 8 of 10 criteria
MET. Ruling D19: stamping 1.0.0 on a build its own checklist refuses would make every other
honesty claim worthless; a gate that cannot say no is not a gate.

**FIVE REAL DEFECTS FOUND AND FIXED (the stage's actual value):**
1. 9a — the primitive table had NO ARITY GATE. `root.console(1,2,3,4,5)` minted a capability
   and ignored the surplus; per-position expect_arg never sees extra args. PRIM_TABLE now
   carries a normative arity column enforced at the call site, proven both directions.
2. 9a — `Secret.map(42)` and `xs.map(42)` checked CLEAN (permissive fall-through on a
   concrete non-function). Both now refuse; unresolved-inference-var case kept separate.
3. 9c — DL0203/DL0408/DL0601 were registered but UNREACHABLE (shadowed by DL0201/DL0401).
   Pre-1.0 is the only window where making an allocated code reachable is not breaking.
   DL0503 stays open (separating it means touching row-var collection = inference).
4. 9e — deep recursion CRASHED THE HOST (raw stack-overflow abort, no diagnostic, no exit
   code). MAX_DEPTH existed but was unreachable: a tree-walker burns several large native
   frames per call. Made ref.rule.runtime.faults-are-diagnostics FALSE. Fixed: the CLI runs
   on a 512 MB thread stack so the depth bound is the limit that fires (delulu/src/main.rs).
5. 9f — `verify-sig` returned EXIT 0 for an UNSIGNED artifact, and **the entire 875-test
   suite passed with the bypass planted**. Every signing test asserted the verdict string;
   none asserted the exit code. Same class as the Stage-6 DL0803 fail-open.

**TWO MEASUREMENT NEAR-MISSES, both written up not erased:**
- Study A's FIRST RUN was a FALSE 100%: the injected code didn't parse, so every build failed
  on syntax and the authority mechanism got credit it hadn't earned (tell: DL0202/DL0401 at
  all 20 sites). Two permanent fences now: the VALIDITY FENCE (every mutation must compile on
  its own) and the NEGATIVE CONTROL (a no-op run must build CLEAN). mechanism_holds is false
  unless both pass. Recorded in measurements/METHODOLOGY.md §1.4.
- Study C first measured a DEBUG build. Now refuses to run without a release build (D16).

**HONEST NUMBERS PUBLISHED (all in measurements/):** Study A 20/20 injections caught (100%,
the mechanism claim). Study B: only 8.5% of 47 real defects offer a machine-applicable repair
and ZERO reach green mechanically (2 offer only authority-widening which the loop refuses BY
POLICY, 2 fix a warning while the error stands). Study C: 2.0x-51.1x SLOWER THAN C — "v1.0 is
NOT competitive with C" in those words. UNRUN and labelled, not dropped: Study B live-model
lane (D4), mechanical-vs-manual comparison (D14), Go baselines (no toolchain).

**BUILT THIS STAGE:** delulu-conform (coverage law + generated reference, `--coverage`,
`--reference`, `--check-reference` a HARD CI gate); delulu-measure (3 studies); delulu-registry
(sparse index, publish API, scoped/revocable tokens, yank!=delete, mandatory signature,
SERVER-SIDE AUTHORITY RECOMPUTATION — cannot-tell REFUSES, hand-rolled HTTP, zero new deps);
docs/reference/ (24 generated chapters, 54 normative rules with DERIVED coverage — a rule is
covered only when EVERY enforcing code is); STABILITY.md; deprecation registry (EMPTY at 1.0
by policy, mechanism complete); language editions (DL1801/DL1802); SECURITY.md + DRILL-001;
CONTRIBUTING.md §AI; rfcs/; CODEOWNERS; docs/for-agents.md; 8 Book samples as conformance
tests; 95 explain bodies written (was 95/150 title-only); release artifacts (SBOM, in-toto
provenance stating the builder honestly as sub-L3 local, announcement + checklist).

**RATCHET:** delulu-conform's `coverage_never_regresses` floor is 273 and MAY ONLY RISE.
Lowering it in a diff is a coverage regression in disguise — review that constant.

LESSON: PowerShell here-strings `@'...'@` break on inner double quotes in commit messages —
write the message to a file and use `git commit -F`. LESSON: `Set-Content -Encoding utf8`
writes a BOM which breaks fmt's comment-attachment law; use
`[IO.File]::WriteAllText(p, s, (New-Object Text.UTF8Encoding $false))`.

macOS: Stage 9 is platform-gate-free std Rust; the ONE change worth confirming on Apple
hardware is the 512 MB interpreter thread stack (9e). Expected fine (virtual reservation),
unverified — no Apple hardware, same honest stance as every prior stage. NEVER pushed.

---

## Stage 10 "Industrial" — BUILT & CLOSED 2026-07-20 (physical stakes)

Spec Rev 2 (a6aec1d, Fable 5, owner-directed): 8 tracks A-H adding heterogeneous compute
(Cap[Compute], vendor-neutral, kernels=ForeignCall+envelope), post-quantum crypto (hybrid
ML-DSA/ML-KEM, KAT-gated, NEVER hand-rolled, "quantum-proof" banned as a claim), cloud/fleets
(deploy plan = authority manifest), autonomy domains addendum (vehicles/aircraft/satellites/fleets;
NO certification claims; broker federation = named RFC-gated gap §2.5); invariants 49-53,
DL1907-11, criteria 1-11. Build order c50de27 (rulings D1-D8). The stability contract was IN FORCE
from commit 1: additive-only, DL19xx budget add-only, grammar changes only where 1.0 reserved the
activation, machine channels byte-identical for untouched programs. `release_requires_full_coverage`
a hard per-commit gate. NEVER pushed to GitHub.

**Model attribution across the stage (honestly recorded):** 10a-10d + the build order = Fable 5;
model switched to Opus 4.8 mid-10e and Opus cooked 10e-10h + 10k + 10l + close-out (trailers correct,
unlike Stage 9's D21 mis-signing); 10i-10j delegated to Sonnet-5 subagents (owner-authorized
mid-stage), head-chef seat's own identity not independently verifiable those two phases (caveat in
ruling D15f). See [[model-attribution-honesty]], [[subagent-cost-strategy]].

**Phase ledger:**
- **10a** attributes DONE 95dc967 (Fable 5): `@aot`/`@interpret`/`@jit`/`@inline(...)` hints; DL1901 on unknown attributes (exact removal repair, never widening); fmt round-trips; invariant-45 twin (run+authority byte-identical with/without hints). Coverage 290, suite 931/0/4.
- **10b** JIT-policy-gate DONE 62039f6 (Fable 5): `--grant exec.native` (default-off, `--grant-manifest` never confers, lease derives false — D6), `[authority] exec.native=true`, authority stamps `native_emission` only when `@jit` present, DL1906 WARNING (program still runs — D7: a hint may not change whether a program runs). NO native tier exists — honesty in code+explain+authority line. Coverage 290, suite 936/0/4.
- **10c** bounded-mailboxes DONE eae77dd (Fable 5): `actor A(mailbox=N)` + `[actors]` defaults; block (CAS-exact, B2 criterion witnessed 500:1 peak≤bound zero-loss) / drop-new (counted, DL1902 abort-mode); same-worker exemption witnessed by a test that deadlocks if wrong; `--trace-memory`. D8. Coverage 291, suite 941/0/4.
- **10d** cycle-collector DONE 9828317 (Fable 5): mark-and-break between turns over worker-wide registry; leak note CLOSED (200/200 cycles broken, output untouched, Weak-proven freeing); Study-C gate -1.0% no regression. D9. Coverage 100%, suite 948/0/4.
- **10e** Actuate DONE 97b76a4 (Opus 4.8, trailer correct — model switched here): root.actuator/root.sensor mints, envelope `--grant "actuator=DEV:dim=lo..hi"`, ActuateErr=Envelope(Str)|NoDevice, DL1904 telemetry (refusal is a VALUE so exit 0), fail-closed envelope incl. skip branch (unbounded dim refused BY NAME), DL0703 wrong-device mint vs pre-flight refusal, invariant 50 null sensor, PRIM_TABLE_VERSION 1→2 (D10a), rate_hz carried NOT enforced. D10 a-f. Coverage 296, suite 955/0/4.
- **10f** dead-man DONE c05befb (Opus 4.8): heartbeat_ms/ttl_ms/fail=hold|coast|safe-park MANDATORY on every actuator grant (each omission refused BY NAME — D11a), watchdog THREAD revokes unaided, ActuateErr::LeaseRevoked(Str) (D11b, PluginErr owns `Revoked`), PRIM_TABLE_VERSION 2→3 + scope widened to prelude types, rate_hz ENFORCED, `--broker-profile sim` deterministic under --seed w/ DEVICE#DIM mirror sensors, DL1905 sim→hw hash gate (all 3 branches incl. skip=refuse + PASS witnessed), "validated twice" claimed only as far as true (D11e), latency measurements/dead-man/RECORD.md (overdue max 6.33ms, engage max 17µs, hb=25ms, n=20, Windows). D11 a-g. Coverage 297, suite 980/0/4.
- **10g** demos DONE cd1ecee (Opus 4.8): Op::Actuate ACTIVATED (required_effect; sensor grants add NOTHING); e-stop = `delulu grants revoke` on per-device CHILD node ({Actuate} alone), watchdog probes tree per tick, ANY non-live answer parks (broker outage stops the arm); both blast radii witnessed; THREE defects found by the demos (device nodes outlived runs→ghosts; `run --lease` silently discarded --grant actuator=/sensor=; TTL expiry mis-journaled as beat-overdue); NAMED GAP: grant node cannot carry a device ENVELOPE (Scopes has no device dim) — RFC-gated. measurements/robotics-demo (cmd 339µs p50, e-stop 12.7ms p50/39.7ms worst, hb-loss→safe-park ~255ms) + measurements/satellite-demo (contact window IS the lease ttl_ms, LOS=TTL expiry, re-contact=NEW delegation, addendum 2.5 note VERBATIM); criterion-10 honesty = docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md (4 findings all fixed); Book Chapter 16 'Commanding Machines'. D12 a-h. Suite 1000/0/4, coverage 297.
- **10h** Track F compute DONE 948d5be (Opus 4.8): Cap[Compute]; dispatch carries EXISTING ForeignCall effect (no `Dispatch` — would imply we say what a kernel computes); PRIM_TABLE_VERSION 3→4; ComputeErr=KernelEnvelope|UnknownKernel|NoAdapter (all-new names, reusing ActuateErr's would make both sums ambiguous); **attestation is a property of the ADAPTER never a claim in a grant** (cpu-reference attests FALSE always → DL1911 refusal is the DEFAULT path, CONST-assertion fails the BUILD if flipped); ENFORCEMENT LEDGER (memory_bytes pre-submission, kernel_ms after-the-fact result-discarded, queue_depth unreachable from sync dispatch, power_w carried NOT enforced); kernels-are-data enforced twice (dispatch takes Str → closure is DL0401; artifacts signed+verified BEFORE parsing); DL1912 invalid/DL1913 unsigned separate, NO require_signed toggle; measurements/compute/RECORD.md accepted ~2.3µs vs refused ~18.7µs (~8× — opposite of 10g); **crit-7 hw-adapter half DEFERRED invariant-45 (invariant 49 tested vs exactly ONE adapter)**. D13 a-h. Suite 1023/0/4, coverage 303.
- **10i** Track G PQC DONE 7243a51 (Sonnet 5, first delegation): self-describing dlsig1 envelope (alg: line, crypto-agility); legacy 96-byte v1.0 sigs still parse+verify (stability); 2 skip branches closed AT PARSE time; DL1908 on classical-only-under-hybrid-required + any unevaluable alg id under EVERY policy; DL1910 gates BOTH sign+verify without --unstable (verify the more important half); NIST ACVP vectors from usnistgov/ACVP-Server for ML-DSA-65+ML-KEM-768, INDEPENDENTLY re-verified by re-fetch+re-hash, one run thru pqc.rs matched byte-for-byte; D15e updates D14b's factual claim (vectors now in hand) but NOT its conclusion (KAT necessary not sufficient, crates unaudited, PQC does NOT reach stable); ML-KEM has ZERO call sites (KEM/transport half not built). D15 a-f. Suite 1040/0/4, coverage 305.
- **10j** Track H cloud/fleets DONE 226b003 (Sonnet 5, agent PAIRS): env profile parsed by the SAME parse_manifest; per-service authority via existing package_authority_value closure; DL1909 refuses WHOLE plan naming exceeding service+effect (plain Diagnostic::error, no Repair span for a TOML ceiling); `delulu fleet update` REUSES DL1905 (spec 9.3); `--previous` unconditionally required (caught for REAL — demo's first draft failed as predicted); staged rollout a PURE state machine ('nothing staged after failure' asserted vs the JOURNAL); measurements/fleet-update/RECORD.md 67-104ms, no separable refuse-vs-accept cost; named-not-built: only EFFECTS checked vs env profile, capability scopes+foreign holes NOT. D16 a-g. Suite 1083/0/4, coverage 306.
- **10k** Track C LTS/advisories DONE 51992dc (Opus 4.8, SOLO — delegation reverted): registry advisory feed (advisories/<name> JSONL, GET /advisories/<pkg>, POST /advisory + advisory file/export CLI, package-scoped token, DL1706 on out-of-scope/unsigned); delulu does NOT depend on delulu-registry (JSON wire shape only); `delulu build` reads LOCAL delulu.advisories.json OFFLINE → DL1903 WARNING, `--deny-advisories` = ERROR CI gate scoped to build; EXACT version-string membership not semver (unevaluable range = fail-open); **SKIP BRANCH D17d: under the gate an absent/unreadable/malformed feed is a FAILURE (DL1905 precedent), without the gate silence**; gate-blocked = note + exit-1 (git_deferred), NOT a DL1903, NOT a new code (budget full); **crit 5 SPLIT D17g: whole loop DRILLED measurements/lts-cycle/ 6/6, TIMED cycle PENDING-ADOPTION never faked**; docs/release/SUPPORT_MATRIX.md + docs/design/VERSION_COEVOLUTION.md. D17 a-g. Coverage 307, clippy 65, suite 1098/0/4.
- **10l** A1/A4 DONE 2d819a9 (Opus 4.8, SOLO) — the LAST phase, both remaining Track-A items land as EVIDENCED DEFERRALS (D4). A1 optimizing tier = Cranelift-optimized Wasmtime tier now EXPLICITLY PINNED (delulu_wasm::optimizing_engine() sets cranelift_opt_level(Speed) not the wasmtime default; 4 run-path Engine sites route thru it; behaviorally identical on wasmtime 27 → Stage-3 differential re-run 3000 programs agreed on every one); **criterion 1 NOT MET & published as-is** (measurements/study-c/HOT_PATH_TABLE.md: opt backend runs 1/6 kernels — fib @ 2.0× C, other 5 hit DL1201; interp 2.0-60.5× C; C lane startup-dominated so ratios UNDERSTATE the gap; v1.0 NOT competitive with C in those words); DIR-level optimizer DEFERRED w/ rationale; authority-preservation holds STRUCTURALLY. A4 multi-threaded WASM DEFERRED w/ honesty note (docs/design/THREADED_WASM_DEFERRAL.md) = criterion 3's sanctioned path (multi-threaded actors already ship TSAN-clean on the interpreter; WASM scheduler cooperative single-threaded by design; shared-everything-GC stack not production-ready at 1.x). NO new codes/anchors — coverage UNCHANGED 307, clippy 65, suite 1098/0/4. Ruling D18 a-f.

**STAGE 10 CLOSED 2026-07-20 (close-out commit 92ef3cc):** build-order §4 close-out table
dispositions criteria 1-11 (status COOKING→CLOSED), spec §11 close-out disposition added. Verdicts:
**6 MET** (crit 2 with its native JIT/AOT-parity clause honestly N/A — no native tier ships per D7;
crit 7 with its hardware-accelerator-adapter clause deferred invariant-45; crit 3/4/9/10 clean);
**crit 1 DEFERRED-HONEST** (D4, hot-path table published as-is, ≤2.5× C target not met);
**crit 8 gates-MET but hybrid-live = the D5 WAIT** (DL1908/DL1910 witnessed + NIST ACVP KAT
provenance recorded, but PQC not stable so every PQ op refuses without --unstable);
**crit 5 mechanism-MET / TIMED-cycle PENDING-ADOPTION**; **crit 6 PENDING-ADOPTION** (registry +
catalog + for-agents.md all ship; the COUNTS need a real external ecosystem, never faked);
**crit 11 SIGNED OFF** (no separate Stage-10 marketing prose authored — v1.0 shipped under Stage 9;
banned-claims scrub re-run repo-wide at close-out = 9 hits, ALL inside prohibition/caveat sentences,
never a claim; every cited evidence path verified to exist pre-commit). Nothing failed silently —
every gap is a named, ruled, published deferral or an honest wait on the real world. 18 rulings
D1-D18. Standing-order memory deleted per its own instruction.

**RATCHET at Stage 10 close:** coverage floor 307 (delulu-conform COVERED_FLOOR), clippy baseline 65,
PRIM_TABLE_VERSION 4, DIR_VERSION 1, WIRE_VERSION broker/1, LANGUAGE_EDITION 1.0. Suite 1098/0/4.

**POST-CLOSE-OUT PRODUCTION-READINESS PASS (ruling D19), 2026-07-21 (Opus 4.8, SOLO).** Jesse's
directive: "test and verify everything in Stage 10, Linux/Windows/macOS operability, make it
production-ready." Two commits on master (NOT pushed): **8c8cf2d** (the pass) + **2734b13**
(docs/design/CROSS_PLATFORM_VERIFICATION.md). D19 a-e appended to the STAGE10_BUILD_ORDER ledger
(**19 rulings now, D1-D19**); close-out §4 got a post-close-out pointer. Both owner-reserved
decisions were PUT TO JESSE and he chose: **commit Cargo.lock**, **raise heartbeat_ms**.
- **VERIFIED cross-platform, ISOLATED + SEQUENTIAL** — lesson: running Windows + a from-scratch WSL
  build CONCURRENTLY starves timing tests and flaked BOTH (different tests each). Windows native:
  `cargo test --workspace` all green, clippy 65/0, fmt 0-change (examples+book), coverage 307/307,
  --check-reference in sync. Linux (WSL Ubuntu-20.04): satellite 4/4 x2 + release 6/6 + earlier full
  run green (build, clippy 66/0 with state_hash gone, fmt, coverage, reference, python-less). **macOS
  STILL ZERO LIVE RUNS** — engineered+analyzed, unproven; Python-less build (`--no-default-features`)
  is the safe path.
- **(a)** `broker_transport::state_hash` -> `#[cfg(windows)]` (dead_code warning Linux/macOS only, 1
  caller grep-proven; not a breakage — CI has no -D warnings).
- **(b) SBOM honesty**: added `ml-dsa 0.1.1` + `ml-kem 0.3.2` (direct delulu-runtime deps, were
  OMITTED), fixed `wasm-encoder 0.252.0->0.221.3` (SBOM had named a transitive wasmtime copy);
  tightened the fool's-gold test (`release.rs::the_sbom_lists_the_real_dependencies` asserted a
  6-crate subset that could not see the gap — skip-branch failure in a checker).
- **(c) Cargo.lock NOW COMMITTED** (discharges D14d's owner-flagged deferral; `.gitignore`
  un-ignored; `REPOSITORY_STRUCTURE.md` "committed from Stage 2" was FALSE -> corrected to 2026-07-21;
  SBOM note now true). Honesty boundary stated: the lock pins the CURRENT resolution, is NOT a
  retroactive certificate for the signed 1.0.0 .dwx (signed 07-20 before any lock was tracked).
- **(d) `.gitattributes`** (`* text=auto eol=lf`; `*.dwx`/`*.sig` -text) — proven 0 renormalization
  (472/472 tracked text files already LF; `git add --renormalize .` stages only .gitattributes);
  closes a CRLF-into-fmt hazard.
- **(e) THE REAL FIND — criterion-10 satellite test flaked in DEBUG.** `cargo test` builds
  unoptimized: one `fib(21)` cycle of `sat-pass.delulu` ~230ms vs HGA `heartbeat_ms=200` -> the
  dead-man watchdog revokes the compute-stalled controller `missed-heartbeat`, the exact cause the
  test forbids (LOS must be `ttl-expired`). Measured release 37ms/cycle (passes), debug ~230ms (1-2 of
  100 wheels). The "1098/0/4 / crit-10 MET" close-out was a FAST-BUILD snapshot, never robust —
  inverts under load (starved watchdog revokes LESS, so it passed concurrent + failed isolated). FIX
  (raise-heartbeat, Jesse's choice): HGA `heartbeat_ms=ttl_ms=1000` (4x the debug cycle; LOS still
  ttl-driven mid-pass in both builds), wheels `heartbeat_ms=ttl_ms=600000`. TTL rose because
  `ttl_ms >= heartbeat_ms` is a hard parse rule (value.rs). `satellite_demo.rs` + `run-demo.sh` +
  `RECORD.md` updated together, Observed block regenerated from a real release run. Verified 4/4 x3
  (Win) + 4/4 x2 (Linux) + 6/6 raw debug deterministic. **DEEPER FINDING logged NOT fixed:** the
  dead-man watchdog uses WALL-CLOCK even under `--broker-profile sim` (device readback seeded, lease
  timing not) — ticking it on the sim's logical clock would decouple demos from interpreter speed;
  future work. Also named: `broker_unreachable` 10s fail-fast budget is load-sensitive; pyo3 default-on
  all-OS build dep has no bundling story. Ratchet unchanged (307 / 65 / suite ~1098).

**D20 — SIM DEAD-MAN ON A LOGICAL CLOCK (finishes D19e), 2026-07-21 (Opus 4.8, SOLO), commit 9ae2053.**
Jesse: "finish the remaining simulator timing improvement," + mid-build guardrail "don't tamper with
delulu authority and guard — defining features." HONORED: change is ONLY in delulu-runtime/src/
device.rs (the device broker's lease CLOCK) + one CLI flag on the run path; `delulu authority`
(effect/authority computation) and the Guard (delulu-broker custody) UNTOUCHED, and the operator
e-stop (authority-probe watchdog the Guard drives) still runs in BOTH clock modes (only heartbeat/TTL
sweep gated to Wall). [[skip-branch-verification-rule]] applied (wedged-program case tested).
- **ClockMode::{Wall, Stepped{step_us}}.** Wall = unchanged real time (all 17 existing device
  dead-man/e-stop unit tests byte-for-byte; `new`/`with_authority_watch` construct Wall). Stepped =
  simulated-µs counter advanced per device interaction, expiry swept SYNCHRONOUSLY there. New
  `with_config` constructor carries the clock; lease timestamps unified to u64 µs via
  BrokerInner::now_us; shared `due()` helper = single source of truth for heartbeat-before-TTL.
- **CLI `--sim-step <ms>`** (run path, sim-ONLY; refused exit 2 on non-sim — a hardware dead-man is a
  real-time promise, not stepped). Watchdog skips heartbeat/TTL under Stepped, keeps the e-stop probe;
  overdue/engage simulated (0 engage, exact overdue) so the trace reproduces too.
- **THE PROOF:** satellite demo under `--sim-step 50` byte-identical (stdout + trace) across debug
  (~230ms/cycle) and release (~37ms/cycle); hga_commanded×step CONSTANT across steps 25/50/100/200,
  revoke exactly one step past TTL (enforcement exact; wide slew refused INTERP-side so 2 clock-
  advancing broker ops/cycle). Deterministic: 10 hga commanded, 90 revoked, 100 wheels, LOS line 43.
- **HONEST LIMIT tested:** stepped advances only on interaction → a WEDGED program stops the clock and
  its lease does NOT expire; that guarantee is Wall's alone (test
  `stepped_mode_does_not_model_the_wedged_program_only_a_wall_clock_can_catch`). Portable by
  construction: device.rs has ZERO `#[cfg]`, pure std → Win+Linux green IS the macOS path; macOS still
  no live run, D20 unchanged its status.
- 3 new device unit tests (20 total). Demos + RECORD.md adopt --sim-step 50 (Observed regenerated).
  Ruling D20 a-e. VERIFIED Windows (test all green, clippy 65/0, cov 307/307, fmt/ref) + Linux (test
  all green, clippy 66/0, fmt/ref/python-less) — ISOLATED/SEQUENTIAL. 20 rulings now (D1-D20). Note:
  .gitattributes (D19d) auto-normalized CRLF→LF on 3 .rs files at commit — the guardrail working as
  designed (committed blobs all i/lf).
Next: awaiting Jesse's direction.
