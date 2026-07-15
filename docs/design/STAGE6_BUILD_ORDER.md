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

## 4. Close-out

*(filled at the end: the 11 §9 criteria, each with its witnessing test(s); suite totals
461 → N; verification evidence)*
