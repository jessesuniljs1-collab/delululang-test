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

## 4. Close-out

*(filled at the end: the 11 §9 criteria, each with its witnessing test(s); suite totals
461 → N; verification evidence)*
