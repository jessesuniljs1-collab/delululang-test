---
name: delulu-hardening-campaign
description: "THE UR-MEMORY for the multi-day DeluluLang hardening campaign (commissioned 2026-07-24): test every stage to failure, fix, document, until it is a real production language. Read this FIRST in any session touching DeluluLang."
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-09-17T02:03:07.536Z
---

**Commissioned 2026-07-24 by Jesse.** A multi-day campaign — explicitly *not* to be finished in one
day — to take DeluluLang from "v1.0.0 released, Stage 10 closed" to **a language anyone (human, AI,
company, robot, future AI) can download, build, install, configure, and use in production.**

## The mandate, in Jesse's terms

> "Even NASA tests new rocket engines until they fail. I want you to do the same. Push DeluluLang
> until it completely breaks, then understand why it failed, fix it properly, improve the design,
> and repeat until no meaningful weaknesses remain."

Eliminate: contradictions, inconsistencies, undefined behaviour, security loopholes, architectural
weaknesses. **Strengthen and maintain Delulu Authority and Delulu Guard across Stages 1–10** so they
stay internally consistent, secure, and enforceable. Every improvement is reflected in the matching
`.md` so **docs always represent the current implementation**.

Framing Jesse gave the language itself, worth preserving verbatim: *"DeluluLang does not discriminate
any users (human or ai or others). But DeluluLang gives 2 types of freedom — freedom with authority
and freedom with responsibility."*

## Reconciling this with the old guardrail — important

The standing rule was **"don't tamper with delulu authority and guard, these are the defining
features."** This commission says **"strengthen and maintain"** them. These are not in conflict, and
the resolution is the operating rule for the whole campaign:

- **Harden, never redefine.** Close a hole, tighten a skip-branch, add a refusal — yes.
- Change what `⊑` *means*, what the Guard *guards*, or the shape of the grant relation — **no**,
  that is a vision-level decision and one of the few things worth stopping to ask about.

## The commission re-stated in full (2026-07-25) — the checklists are the durable part

Jesse re-sent the mission with more specificity. Opening frame, verbatim in spirit: *"We're no longer
building a new lang research prototype. We're building the production implementation"* — intended to
serve *"humanity, AI systems, robots, autonomous agents, distributed systems, and future generations
of intelligent software"*, with the goal *"to become one of the world's great programming
languages"*, judged by **"Would I be proud for this to still exist and be used in all the years to
come?"** Core values it must maximize: **Freedom, Responsibility, Authority, Safety, Transparency,
Auditability, Human dignity, AI dignity.** Authority is never unlimited; delegation, revocation and
audit are explicit; **nothing becomes ambient authority; security is never based on trust.**

**Authority pressure-test checklist (18 items — P15 discharges these one by one; each stage pass
ticks the ones it touches):** capability attenuation · delegation · revocation · custody · authority
lattice · authority propagation · authority serialization · distributed authority · broker federation
· device delegation · adapters · runtime enforcement · audit chain · replay resistance · authority
confusion · privilege escalation · authority leaks · authority forgery.
*"Attempt to defeat it. If you cannot defeat it, attempt again from another direction."*

**Guard surface checklist (14):** parser · compiler · optimizer · runtime · WASM backend · native
backend · FFI · adapters · broker federation · hardware · distributed execution · plugins ·
certificates · authority envelopes.

**Hostile-technique checklist (19):** fuzz · AI-written · malformed · huge · deeply recursive ·
malicious authority graphs · distributed hostile scenarios · privilege escalation · capability
forgery · authority confusion · parser confusion · diagnostic confusion · resource exhaustion · DoS ·
replay · race conditions · specification ambiguity. **"Nothing is *verified* until the hostile
version fails."**

**Real-world domains to keep generating (fix the LANGUAGE, not the example, when one exposes a
weakness):** compilers · databases · OS components · robotics · drones · satellites · industrial
automation · web servers · AI agents · LLM orchestration · distributed systems · autonomous vehicles
· scientific computing · cryptography · game engines · finance · healthcare · manufacturing.

**Docs that must move with every meaningful change:** specification · language reference · the Book ·
architecture docs · RFCs · changelog · README · migration guides · examples · tutorials · threat
model · security docs. *"Documentation is part of the product. Never allow documentation drift."*

**Also explicit:** verify on **Linux, Windows AND macOS**; *"unless running parallel agents is
genuinely more efficient and necessary, do all the work yourself"*; work in phases, report
findings/fixes/risks/verification/docs/next after each, auto-continue on the timer.

## Methodology (the thing that makes findings real)

1. **A finding is not real until a witness fails against the old code.** Write the test, revert the
   fix, *observe* the exact expected failures, restore. This caught two genuine vulnerabilities in
   the federation work and is now house method. See [[skip-branch-verification-rule]].
2. **Breadth sweep before depth.** Fire the engine and see what melts, *then* go stage by stage —
   so the per-stage passes are driven by observed failures, not by reading specs top to bottom.
3. **Commit at green, small and often.** Push only to the testing remote
   ([[delulu-github-remote]], since 2026-09-14, public since 2026-09-17; before that the rule was never push — [[delululang-project]]). Since 2026-09-17, push every commit there without asking ([[testing-repo-autopush]]).
4. **Every deviation gets a numbered ruling** in `docs/design/STAGE10_BUILD_ORDER.md` (D1–D23 as of
   campaign start). Docs move with code in the same commit.
5. **Pause and reassess after each phase** (Jesse asked for ~2 min); reset assumptions before the
   next one rather than coasting on momentum.

## Phase ledger — the durable spine

Campaign plan of record lives in **`docs/design/HARDENING_CAMPAIGN.md`** (in-repo, versioned).
Live per-phase status is in the harness task list. Phases:

| # | Phase | State |
|---|---|---|
| P0 | Baseline, gate harness, breadth sweep | see repo ledger |
| P1 | Front door: download/build/install/use made TRUE | see repo ledger |
| P2–P11 | Stage 1→10 adversarial passes, one per stage | see repo ledger |
| P12 | Scale: 10k–30k+ line programs, monorepos | see repo ledger |
| P13 | Fuzz, malicious input, security | see repo ledger |
| P14 | Performance and memory pressure | see repo ledger |
| P15 | Authority + Guard cross-stage consistency audit (capstone) | see repo ledger |
| P16 | Cross-platform re-verification and close-out | see repo ledger |

Stage identities, for orientation: 1 Skeleton · 2 Provenance · 3 Containment · 4 Foreign ·
5 Custody · 6 Live · 7 Concurrent · 8 Surface · 9 Delulu (v1.0) · 10 Industrial.

## State as of 2026-07-24 (day 1)

**P0 (baseline + breadth sweep) and P1 (front door) are done.** Commits: `92ddaed` (D24, parser),
`5ac399e` (P0 close-out), `b84d87e` (D25, front door + C11/C13). Gates after P1: **93 suites,
1190 passed, 0 failed, clippy 65/0** — the exact pre-campaign baseline (65 is load-bearing; three
new tests pushed it to 68 and were rewritten to `Grants { console: true, ..Default::default() }`).
Findings C1–C15 are in the repo ledger; the ones a future session must not re-derive:

- **C6 is the campaign's thesis.** The docs, not the code, were the wall. One honest program per
  domain, written from the Book/samples/reference, mostly did not compile — and *every* failure
  was a documentation gap, not a language limit. The project tested its docs for **accuracy** and
  never for **sufficiency**.
- **C13 was the most serious defect**: `apply(double, 21)` — a named function as a value —
  type-checked and faulted at runtime. Hidden because the Book's sample gate **checks and never
  runs**. New gate `crates/delulu/tests/examples_run.rs` closes that.
- **C11**: a user `fn parse_int` silently lost to the prelude builtin; the error surfaced at a
  *call site* naming a type the author never wrote. Now DL0302 at the definition.
- **C9 is OPEN and is Jesse's alone: there is NO LICENSE.** Default copyright = nobody may use
  DeluluLang. It is the single hardest blocker to the campaign's own goal, and picking a licence
  is not the chef's call. Ask.

Working method that is paying off: **write the program a real developer would write, then fix what
stops it.** Both C11 and C13 were found writing documentation, not reading code.

**P2 (Stage 1 adversarial) done** — commit `ed80f83` (D26). Headline: **DeluluLang refuses Trojan
Source** — raw Unicode bidi control chars are DL0107, a hard error. The design is one scan of raw
source ahead of tokenizing (un-skip-branchable), and because it reads raw bytes the `\u{202e}`
escape stays legal (visible in review) and RTL *letters* are untouched (no i18n break). This is THE
Stage-1 security property for a review-focused language. Also filed, both verified safe: C16 (cyclic
`type A = A` silently accepted — chased to a soundness verdict: 5000-deep chains resolve, secrets
can't launder through a cycle, so it's hygiene) and C17 (float→inf accepted silently, matches
C/JS). Gates after P2: **93 suites, 1194 passed, clippy 65/0**.

**2026-07-24 elevated-mission note.** Jesse reframed: this is the *production* implementation, judged
by "would this still be correct when no one alive remembers writing it" (**all the years, not just
20**). Priorities he set: pressure-test **Delulu Authority hardest of all**, then Guard; assume
malicious humans/AI/robots; no ambient authority ever. Three verification threads to weave through
the stage passes — **status now established, do not re-derive:**

1. **Delulu Graph = the in-repo Atlas — EXISTS and is real.** `delulu atlas <file|dir>` with
   `--format tree|digest|json|dot|mermaid|html`, plus query verbs `node`/`callers`/`calls`/`why`/
   `path`, described in-tree as "a typed, deterministic code + authority graph". Note the naming gap:
   the phrase *"Delulu Graph"* appears in **no** repo file — Jesse's name for it and the product's
   name differ. Aligning them is a public-naming call → **ask, don't rename.** Verify at P9.
2. **Character morphs — WERE specified-but-absent (C22); now BUILT (ruling D35, 2026-07-25).**
   Doc contradiction fixed first (`5aea47d`), then implemented on Jesse's instruction. What exists:
   `delulu-syntax/src/morph.rs` (the law + validation + both render directions), `lexer::lex_with_morph`
   (the ONLY place a non-canonical surface becomes tokens), `delulu/src/morph_file.rs` (TOML + search
   path `./morphs`, `$DELULU_MORPH_PATH`, `~/.delulu/morphs`), `delulu morph list|info|check|render`,
   a `//! morph: <id>` pragma honoured by check/run/authority/fmt, DL1710–DL1714, and two shipped
   morphs (`morphs/zh-CN-keywords.toml`, `morphs/compact-ai.toml`). **Proof: a Chinese-keyword program
   prints `fib(10) = 55` via `delulu run`; round-trip is byte-identical; the authority report is
   byte-identical across surfaces.** Emoji/Cyrillic/Greek/mixed round-trip in property tests.
   **Two decisions to never undo:** (a) the PARSER does not resolve the pragma — that would mean
   reading a file off disk from a comment in its own input = ambient authority inside the compiler;
   the CLI does the lookup. (b) Conversion happens at exactly TWO edges, so nothing downstream knows
   morphs exist. **NOT built (enumerated in the spec header, don't let it blur): `.dpx` plugin
   delivery, per-reader LSP view morphs, `fmt --to-morph` flags, `[style] morph` policy key, and
   morph-aware PACKAGE builds — a package's `src/` must be canonical.**
   **The spec's own law had a hole and it is the reusable lesson:** bijective + single-token +
   no-alias-collision still permits `let = "fn"`, which renders and round-trips perfectly while making
   the word `fn` mean `let`. For a review-focused language that is the D26 attack in another costume →
   closed as DL1711, added to the spec as rule 1a.
3. **Credential/secret-leak detection** — DeluluLang should tell users about exposed API
   keys/credentials in AI-written code; the `Secret[T]` opacity (DL0604/DL0605) + authority report are
   the machinery — verify and strengthen (P5/P15).

**LICENSING DONE (D27, commit 42702e2): Apache-2.0 + trademark, derivatives rename** — Jesse
approved both. C9 closed. LICENSE/NOTICE/TRADEMARK.md/GOVERNANCE.md at root, `license`/`authors` on
all 12 crates, SBOM component licence added, governance test guards it. See [[delulu-licensing-intent]].

**Auto-continue mechanism (Jesse, 2026-07-24):** ending a turn to "wait 2 min" did NOT auto-resume;
Jesse asked for a real timer. Use **ScheduleWakeup** (delaySeconds≈120) at the END of a phase to
re-invoke and continue the next phase, re-arming each time. This is how overnight work actually
continues. Stop only for owner-reserved decisions.

**P3 (Stage 2 Provenance) DONE — the campaign's first supply-chain hole, an Authority finding.**
**C18 CLOSED (D28):** the semver-authority law (DL1003) and `authority --diff` were **blind to
secret-scope widening**. `root.secret("X")` adds NO effect and NO cap_kind — only a name — and the
lock entry never stored secret names, so a dep adding a secret read on a patch bump was waved
through. Contradicts invariant 10 ("scopes"). Fixed: lock entry gains `secrets`; `authority_widened`
+ `authority --diff` now see it. **Deliberately did NOT fold secrets into `authority_hash`** — that
would invalidate existing lockfiles (DL1002), a format break, backcompat is owner-reserved;
same-version secret changes still caught via content_hash/DL1010. Two witnesses fail against old
code.

**C19 OWNER-RESERVED (present, don't auto-fix):** SAME blindness one layer earlier —
`check_self_authority` (DL1009) doesn't force a package to declare secrets it reads;
`check_pins`/`scope_violations` (DL1001) doesn't constrain a dep's secrets vs the pin (AuthoritySpec
HAS a secrets field, just unchecked). Fix small, breaks nothing in-tree, BUT changes what the CHECKER
ACCEPTS → **backward-compat = owner-reserved**. Recommend making it. Waiting on Jesse.

**Reusable lesson: authority is tracked well on {effects, cap_kinds} but the finer SCOPE dimensions
(secrets, device envelopes, foreign targets) are unevenly enforced across lockfile/pin/report.**
When auditing authority anywhere, check EACH scope dimension separately — effects/kinds being right
does NOT mean scopes are. [[skip-branch-verification-rule]] applied to authority dimensions.

**P4 (Stage 3 Containment) DONE — commit `4ca1e4c` (D29). C20 CLOSED: the two engines did not fault
alike.** The differential fuzz harness classifies any `(Err, Err)` as *agreement* without comparing
the faults, so invariant 15 was never actually tested on the fault path. `7/0` was DL0902 on the
interpreter and a generic DL0904 on WASM; overflow likewise; and deep recursion printed **16,326
lines** of `<wasm function N>` guest backtrace — a DoS against the reader and against an agent's
context window, plus a leak of generated-code shape. Fix = `clean_trap` in `delulu-wasm/src/host.rs`
maps every wasmtime `Trap` to the code the interpreter would report and keeps only the first line.
Residual divergence NAMED not hidden: `%`-by-zero is DL0901 on WASM (wasmtime raises one trap for
both integer division traps). Invariant 15's wording corrected — identical stdout/exit/fault CODES,
never byte-identical stderr. **C21 filed OPEN:** 10 000 interpreter frames need >16 MiB of host
stack, so `MAX_DEPTH` cannot be reached on small-stack embeddings — the D15 host-crash class
resurfacing off the main thread.

**Cross-platform after P4: Windows 94 suites / 1199 passed / 0 failed / clippy 65 (baseline exactly);
Linux 94 / 1203 / 0 / clippy 66 (known delta; Linux runs 4 tests Windows ignores). macOS STILL NEVER
EXECUTED — no hardware. Say this plainly every time; never imply three-platform coverage.**

**P5 (Stage 4 Foreign) DONE — commit `c5e59f5` (D30–D34). Windows 94 suites / 1213 passed / 0 failed /
clippy 65.** Invariant 20 HOLDS: a raw secret cannot cross the FFI (DL0602 refuses it even with a
shadowing trick), the grant gate is enforced at RUNTIME not just in the checker (DL1303 at startup),
arity is checked (DL0403), a missing row is DL0501, and `trace_foreign` fires BEFORE the call so it
cannot be omitted by crashing. Real C executes: `msvcrt.puts` printed from inside the C function.

Five findings closed, one held:
- **C23/D30 — ALL 16 builtin type names and ALL 10 core effect names were shadowable, every shadow
  silently inert.** Found by attacking the name-matched FFI allowlist (`type Int = Secret[Str]` checked
  clean). NOT exploitable — both sides lower by name and agree — but `effect Write` left every
  `! {Write}` meaning the real filesystem-reaching effect while its author thought otherwise. DL0302 at
  the definition, on **all three** DeclTable paths (resolve/program/deps — the package path had its own
  gap). **Reusable: `PRELUDE_TYPES` + `CORE_EFFECT_NAMES` in check.rs are the registries; keep them in
  lockstep with `lower_type`/`lower_row`.**
- **C24/D31** — foreign-signature alias refusal now names the target + offers the edit; alias→fn is
  DL1302. Fence stays name-based DELIBERATELY (loosening would make the runtime marshal `FKind::Unit`).
  Alias walk bounded at 32 hops because `type A = (A)` is accepted (C16) — an unbounded walk would
  have shipped a hung compiler inside the fix.
- **C25/D32 — the credential-exposure line**, discharging commission thread 3. The report had every
  fact (`Declassify`, secret names, foreign reach) on three separate lines and never joined them.
  Now: `exposure: API_KEY declassifiable -> foreign code (outside the proof), files/console`. Reports
  CAPABILITY not behaviour; names the safe case too; `--json` deliberately unchanged (agents could
  always derive it, only humans couldn't). Witness: a program really does print `SUPERSECRET-abc123`
  from inside C, with every gate passing correctly.
- **C26/C27/D33** — a package whose sources sat beside `delulu.toml` instead of under `src/` printed
  `built clean (0 module(s))` and exited **0**; and `run <dir>` gave `Access is denied. (os error 5)`.
- **C19/D34 — owner-approved 2026-07-25** — manifest ceiling + dependency pin now bound SECRETS.
  `scope_violations` checked effects/fs/net and never secrets, so a `secrets` pin was decoration.
  **Gotcha that made the first witnesses pass against broken code: `check_self_authority`/`check_pins`
  are NOT part of `check_workspace` — the library computes facts, the CLI composes the gate.**
- **C28 OPEN, OWNER-RESERVED: `type A = B` is ambiguous in the normative grammar.** Matches both the
  sum and the alias production; the parser silently prefers a single-variant SUM. So
  `fn g() -> Meters { Int }` checks clean (`Int` is a constructor!), **no alias to a bare type name can
  be written at all** (`type Meters = (Int)` — parenthesised — is the only spelling that reaches the
  alias production), and a mistyped value says `expected 'T9'`. Two coherent fixes, both change what
  compiles → present, don't decide.

**P6 (Stage 5 Custody + Guard) DONE — commit `e5dd401` (D36–D37). Windows 94/1249/0/clippy 65.**
Three findings, and the framing matters: **none is an authority bypass** (`validate` re-reads effective
state per op), they all broke **accountability**, which is what this system sells.
- **C29/D36 — a lease token for a REVOKED grant redeemed `Ok`.** Also under a revoked ancestor, and
  under an EXPIRED ancestor when the token had no deadline of its own (`exp = i64::MAX`). Only the
  case where the token's deadline mirrored the node's TTL was refused — incidentally, not by design;
  there was no state check at all. Cost: the audit record said `decision: "allow"` for a grant an
  operator killed, and `set_holder_peer` wrote the redeemer's text onto the revoked node. Fix = one
  `effective_state_inherited` call, after the MAC and before the nonce burn/any write.
- **C30/D36 — `audit tail`/`query` verified NOTHING.** `verify` recomputes every hash+link and works;
  the read path never called it. Flip a `decision` → the forged value displayed, no warning. Corrupt
  one line → **the record VANISHES** (g_1,g_2,g_4 shown; g_3 gone, no gap marker). Fix: verify on
  read, still SHOW the records (the investigator needs them), loud warning, nonzero exit, and
  `chain_verified` always present in `--json`.
- **C31/D37 — `Scopes` has 8 dimensions; the Guard enumerated 7.** `tier_for_mint` walked a fixed
  `[…;7]` array and `use_axis_class` ended in `_ => None`, so `device` (RFC 0001 F1) was never added
  and **neither could fail to compile**. Actuation was still gated via `effect:Actuate` — do NOT
  overstate this — but `device` was the only axis with no per-item rules, on the one axis that moves
  hardware. Added `GuardClass::Device`, **no default rule** (operator's call), and made
  `use_axis_class` **exhaustive** so the next `Op` cannot be born ungated in silence.

**Verified-and-held (don't re-derive):** `attenuation_check` covers all 9 dimensions in ONE
conjunction and `attenuate_core` is the ONLY child-creation path (fail-closed on missing/dead parent
incl. ancestors); expiry inheritance is a bounded walk that fails closed; `extend_ttl` is monotone +
crate-private + neutered by inheritance; the token MAC covers the ENTIRE canonical payload and is
checked BEFORE the claims parse; a token dies at the MAC on a different broker (witnessed);
no-discrimination is real (`outcome_json` excludes the holder, no decision path reads `.kind`, a test
varies the kind). **NOT covered, don't claim it: all 14 Guard surfaces individually, and revocation
racing an in-flight op — both belong to P15.**

**CRASH HUNT (commissioned 2026-07-25 mid-P7) DONE — commit `dfb1fd5` (D38). Windows 95/1253/0/clippy
65; Linux 95/1257/0/clippy 66 — BOTH at baseline, which also closed P6's Linux drift.**

**Jesse's framing correction, and it is load-bearing: there is NO discrimination between the surfaces.**
I had written "humans on the compiler, machines on the CLI"; he corrected it — a human may drive the
CLI and an agent may drive the compiler, so a broken machine contract and an unreadable wall of stderr
are the SAME class of defect. Do not re-introduce that split in any doc or plan.

**Negative result worth keeping (don't re-run blind): the front end is robust.** 26 hostile programs
(2000-deep parens, 1500-deep blocks, 800-deep generic types, 20k-term expressions, 200 KB literal,
100k-char identifier, 6000 fns, 3000-field record, 2000-variant match, unterminated string/comment,
NUL bytes, empty, BOM-only) + every subcommand × malformed args in both modes = **no panic, no hang,
no signal death**.
- **C2/D38 — `--json` emitted NOTHING on failure across ~all subcommands.** Filed as one narrow case;
  it was the whole surface. Fixed in ONE wrapper (`cli::run`), not at the ~161 `return 2` sites.
  Envelope has documented fields + `summary.errors=1` + additive `error` object, and **invents no DL
  code** (registry is a contract; a usage error isn't a language diagnostic). **The gate asserts
  EXACTLY ONE object** — that's what caught the mirror bug in `test`, then `fleet`, then `deploy`.
  38 stdout JSON emitters across 5 files now call `note_json_emitted()`; `println!("{obj}")`-style
  Display emissions are invisible to grep heuristics, so hand-audit when adding one.
- **C32/D38 — a 10 KB file emitted 76,518,387 bytes of stderr in 14.2 s.** Every diagnostic quoted its
  whole source line twice (text + underline) × ~5000 errors. D29's flood in a new costume. Fixed with a
  160-char snippet window (char-indexed, `...` marker, caret corrected) + a 50-diagnostic human cap that
  states what it withheld. `--json` deliberately uncapped. → **23,530 bytes / 0.125 s (3,252×)**.
- **C33/D38 — `deploy` and `fleet` worked and `--help` listed neither**, which is why a sweep couldn't
  reach them and deploy's double-emit survived. A gate now asserts every dispatched subcommand is in
  `--help`: **an undocumented command is a command nothing sweeps.**

**Gate to keep green: `crates/delulu/tests/json_contract.rs`** (4 tests: exactly-one-object sweep,
failure-envelope fields, no-panic sweep, help completeness). Suite count is now **95** (was 94).

**P7 (Stage 6 Live) DONE — commit `835b088` (D39). Windows 95/1256/0/clippy 65; Linux 95/1260/0/clippy
66; coverage 100%.** One finding; the rest of Stage 6 held under attack.
- **C34/D39 — a plugin manifest could declare `device`/`foreign_c`/`foreign_python` and have it
  SILENTLY DROPPED.** `Grant::to_authority` + `ceiling()` hard-code all three empty (correct: a plugin
  never commands a machine or binds a native lib), but reading a manifest that declared them just
  dropped it → the artifact loaded clean advertising a ceiling it did not have. **Not exploitable**
  (drop is toward LESS authority; `cap_slice`'s `_ => {}` gives an unlisted effect no host import at
  all, and that catch-all is documented as fail-closed). Ruled anyway on C23's reasoning: DL1508 at
  step 1, naming the dimension; an **empty** list stays legal (claims nothing).
  **Deliberately NOT ruled:** a ceiling may still name `Actuate`/`ForeignCall` — `cap_slice` marks those
  as "no Contained host import in v0.6" = forward work, so refusing would prejudge it. Inertness
  witnessed instead.

**VERIFIED-GOOD in Stage 6 — do not re-derive:** (1) **ONE path** to a loaded plugin's authority:
`to_authority()` → `step3_ceiling` (reuses `attenuation_check`, all 9 dims) → `step4_holder`; the
`PreparedLoad` value IS the checked one, no alternative constructor. (2) Class never inferred or
substituted; Verified with missing/bad DIR is DL1504 with **no fallback to Contained**. (3) **Signatures
are the best code in the campaign so far**: `Invalid` is matched FIRST and UNCONDITIONALLY, so a
present-but-invalid signature refuses even when `require_signed` is false — the exact skip branch a
"not required so don't check" reading opens; unsigned-but-required is DL1511 vs badly-signed DL1510.
(4) Reload mints a FRESH node → unload/reload authority swap impossible; unknown node confers nothing;
limit kill revokes in the same act; failed step revokes and instantiates nothing. (5) **`.dpx` reader
was already hardened**: every length `checked_add` + `<= bytes.len()`, so nothing allocates on a
DECLARED size; `read_uleb` bounds shift at 32 bits; walk strictly advances; an existing test flips every
byte + truncates at every offset. Added the shape that sweep can't reach — a CRAFTED 5-byte ULEB
declaring 0xFFFF_FFFF (allocation bomb) + unterminated ULEB + 64 zero-length sections.

**P8 (Stage 7 Concurrent) DONE — commit `02bba11` (D40). Windows 95/1257/0/clippy 65; Linux
95/1261/0/clippy 66; coverage 100%.** The concurrency model held under every attack; the defect was a
hand-written list.
- **C35/D40 — a `Root` slice silently loses `computes` crossing an actor boundary.** `RootMsg` is a
  hand-enumerated copy of `RootVal`'s dimensions; phase 10h added `computes` and didn't extend it, while
  10e's `actuators`/`sensors` DO cross (and `ComputeEnvelope` is the same plain-data shape as
  `ActuatorEnvelope`, so there was no obstacle). Fail-closed but an OMISSION, not a decision.
  **Fixed the silence, not the capability:** a gate reads BOTH struct defs **out of the source** and
  fails if any `RootVal` dimension neither crosses nor is in an explicit `WITHHELD_FROM_ACTORS` list —
  and fails the other way too, so a stale "withheld" claim can't outlive the fact. Whether `computes`
  SHOULD cross widens what actors may do → **capability decision, left for Jesse**; restrictive reading
  stands.

**⚠ THE RECURRING PATTERN — name it, check it first in every future phase:** *a hand-maintained list of
authority dimensions falls behind `Scopes`/`RootVal` and nothing notices.* Three instances now: **C31**
(Guard's fixed `[…;7]` array + `_ => None`), **C34** (plugin ceiling dropped 3 dimensions), **C35**
(actor boundary dropped `computes`). **Every such list needs a gate.** Rust has no reflection, so a
source-scanning test is legitimate here — `delulu-conform` already does it.

**VERIFIED-GOOD in Stage 7 (best skip-branch discipline in the tree — don't re-derive):**
(1) **Sendability refuses when it cannot tell** — `check_boundary_params`' `None` arm: *"sendability
could not be determined … a boundary guarantee is never guessed"*; `default_rcap` is **exhaustive** over
every `Type` with `Type::Var(_) => return None` ("never guess (kitchen rule)") and composites propagate
undecidability. Tested: `ref List` / closure param / generic `T` all refuse (last = *undecidable*).
`Cap`/`Secret`/`Root` are sendable **by decision** (documented: unforgeable immutable handles, inv 36).
(2) **`consume` is flow-sensitive on every shape**: branch join, loop-carried (own message: "by the next
loop iteration this binding is already dead"), match arm, double consume → all DL1602; `recover` over a
non-sendable outer binding → DL1605.
(3) rcap deny properties hold: write via `box`, sync method on `tag`, field read via `tag`, `ref`
captured in a `val` closure → DL1603/DL1604.
(4) **Aliasing an `iso` is SAFE and looks like a hole — don't re-file it.** `let alias = xs` is legal and
the alias stays readable after `consume xs`; the alias degrades to read-only (mutate=DL1604,
send=DL1601), and **`MsgValue` is fully owned (no `Rc`/`RefCell`), so a send DEEP-COPIES by
construction**. Scheduler is genuinely multi-threaded, each worker owns its actors' heaps, cells never
cross threads → the alias reads the sender's own data.

**P9 (Stage 8 Surface) DONE — commit `407aefb` (D41). Windows 95/1261/0/clippy 65; Linux 95/1265/0/clippy
66; coverage 100%.** Two findings, both about a READER being misled rather than a program being wrong.
- **C15/D41 CLOSED (open since P0) — `fmt` merged comment paragraphs.** The transferable lesson is why
  no law caught it: the identity law's comment projection is `(text, own_line)` **in order**, and a merge
  changes none of the three — only the *spacing between* comments, which is the part carrying meaning.
  **When a projection is chosen to prove a property, ask what the projection cannot see.** Fix = track
  the source line each own-line comment ends on, emit one blank when the next starts >1 line later. Runs
  of blanks still collapse to one (structure ≠ whitespace). **Skip branch: a TRAILING comment must not
  end a paragraph**, else fmt invents blanks on top of item separation.
- **C36/D41 — `atlas --format mermaid` renders 3 nodes/1 edge for a 12-node/23-edge graph** (packages +
  modules only; `dot` shows all). Scope is correct and WAS documented — one line in an addendum — but a
  mermaid diagram gets pasted into READMEs/agent context away from any docs, so the available conclusion
  is "this program has no effects". Fixed by making the **artifact self-describing** (two `%%` lines) +
  `--help`. **NOTE for future me: the HTML renderer was already right** (full graph; collapses only above
  `HTML_NODE_CAP`, *with* a visible notice) — I misread it as unconditionally module-level at first.

**VERIFIED-GOOD in Stage 8 — don't re-derive:** (1) **Locale invariance proved mechanically on BOTH
platforms**: same diagnostic under `--locale en-US` vs `delulu-slang` → **byte-identical `--json`**, human
prose changes. (2) Atlas: all 4 verbs (`node`/`callers`/`calls`/`why`) correct — **note the verb comes
FIRST** (`atlas callers fib <file>`, not `atlas <file> callers fib`); `digest` byte-stable across runs;
a program with check errors → **DL1780 alongside** the real diagnostic (no partial graph). (3) LSP
survives empty input, non-JSON, unknown method, truncated frame (`Content-Length: 99999`), hover on a
missing file — no panic/hang/signal.

**P10 (Stage 9 release integrity) DONE — commit `16e1092` (D42). Windows 95/1263/0/clippy 65; Linux
95/1267/0/clippy 66; coverage 100%; reference in sync.** Attacked the CLAIMS, not the code paths.
- **C37/D42 — THE BIG ONE: the conformance coverage law proved a witness EXISTS, not that it EXERCISES
  its anchor.** Repointing DL1710's *rejecting* witness at a real-but-unrelated test left the gate saying
  `PASS: 100% anchor coverage`. (Dangling and `#[ignore]`d witnesses ARE caught — verified by breaking
  both.) Fix: a **rejecting** witness's body must NAME its code — measured first, **108/109 already did**.
  Scoped to rejecting only: accepting witnesses prove a code does NOT fire (82/144 name no code,
  correctly). One reasoned exception (DL1907 — its refusal is a catchable `ComputeErr`, not a DL-coded
  diagnostic), listed the `WITHHELD_FROM_ACTORS` way.
- **C38/D42 — unsigned and badly-signed both reported DL1705** on the detached path. Unsigned is now
  **DL1511**. **The project had ALREADY RULED this** (Stage-6 deviation 8: "badly-signed vs unsigned are
  DIFFERENT faults", with a test asserting the phrase) and the plugin path implemented it — only the
  detached path never followed. Everything else there refuses correctly (sig over another artifact,
  truncated to 95/96, flipped key byte, flipped sig byte, `--require-hybrid` → DL1908).
- **C14/D42 CLOSED — `DL0907` was titled "match reached no arm"** but is raised for ~10 conditions
  (unbound name, `?` on non-Result, assignment to non-record, unknown fn, actor turn w/o address, foreign
  value at an actor boundary…). `delulu explain DL0907` told readers something false about their program.
  Now names the CLASS; message names the condition.

**VERIFIED-GOOD: the SBOM is accurate** — 17 direct deps declared, 17 listed, zero drift both ways, every
version matches what the lock resolves for the DIRECT declaration; its note explains the transitive
omission honestly. **D19's fix held.** ⚠ **METHOD WARNING for future me: I nearly mis-reported drift** by
building a name→version map from Cargo.lock — `wasm-encoder` and `getrandom` each appear at THREE versions
there; the SBOM correctly names the one bound by the direct declaration. Compare against the *declaration*,
not a flat map.

**P11 (Stage 10 Industrial) DONE — commit `87e5766` (D43). Windows 95/1272/0/clippy 65; Linux
95/1276/0/clippy 66; coverage 100%; reference in sync. THE LAST PER-STAGE PASS — P2–P11 all complete.**
Seven findings closed (C39–C45), one raised owner-reserved (C46). Two of them moved a machine.

- **C39/D43a — THE BIG ONE: a simulation that could not run out of time.** `--sim-step` (D20) advances
  simulated time one step per *device INTERACTION*; `interp.rs::call_actuator` refuses an out-of-envelope
  command and returns BEFORE `broker.command()`. Composed: a program whose every command was refused
  **froze simulated time** and held its device forever, while the identical program+grant on the WALL clock
  lost it (`beat overdue by 657 µs` vs no revocation at 1000× the heartbeat, 6×). **Why it mattered: DL1905
  refuses hardware without an approved SIMULATION of those exact bytes** — so the one environment that
  authorizes hardware could not rehearse the revocation hardware would produce, for exactly the fault class
  a dead-man exists for (every setpoint out of range; a units bug is the ordinary cause). Fix =
  `DeviceBroker::note_refused_attempt` (step+sweep, report a dead lease). **Dead-man UNTOUCHED** — `due()`
  unchanged, wall watchdog unchanged, a refused command still does NOT beat. Jesse's guardrail honored.
- **⚠ THE SAME FIX CLOSED A SECOND, BROADER DEFECT ON THE WALL CLOCK — and I did not predict it.** Lease
  now checked before envelope on the refusal path, so a program that already LOST its device and sends a bad
  command is told it lost the DEVICE, not that its setpoint was wrong. Before it clamped and retried against
  a machine it did not hold — defeating why 10f split `Envelope` from `LeaseRevoked` (D11b). **METHOD
  WARNING: my first attempt to demonstrate it FAILED** (six rapid refusals finish before the watchdog's
  first tick — no observable difference); it needs real time between commands (a `fib(25)` burn). The
  witness settled it, not the reasoning. **Also: my FIRST reading of the mechanism was wrong** — I thought
  refusals were beating the watchdog and that D11a(c) ("the beat rides every ACCEPTED operation") was
  contradicted. D11a(c) was accurate all along; the gap was in the CLOCK, not the beat rule. Read the
  early-return before concluding.
- **C40/D43b — a term stated twice was resolved silently, and the two parsers resolved dims OPPOSITELY.**
  Broker `BTreeMap::insert` = LAST wins; runtime `Vec::push` + first-match = FIRST wins. So
  `angle_deg=-30..95,angle_deg=-1..1` = `[-1,1]` to the recorded authority and `[-30,95]` to the code that
  moves the machine. **The dangerous edit is the safe-looking one: appending a tighter bound recorded a
  tightening it did not apply** (witnessed with a control). `authority.rs` had ALREADY written this rule one
  level up (`Scopes::device` is a map "because resolving it silently is how a widening gets in") and never
  applied it per TERM. Both sides now refuse a repeat.
- **C41/D43c — non-finite bounds accepted by BOTH runtime parsers**; the broker has refused them since D12e
  and documents the refusal as load-bearing (`impl Eq` soundness). `angle_deg=-inf..inf` commanded 12°.
  Sharper: `kernel_ms=0..inf` — that term is mandatory *because* "a kernel with no time budget can occupy
  the device forever", and `0..inf` satisfied the requirement while being exactly that. (`NaN..NaN` was
  already harmless — every NaN comparison is false — but by accident.)
- **C42/D43d — `fail=` was free text in the broker, a closed enum in the runtime.** `fail=hodl`, `fail=`,
  `fail=safe_park`, `fail=Hold` → a grant NO program could mint, found when a robot tried to move. One list
  now: **`device_scope::FAIL_STATES` in the LOWER crate** (runtime→broker, not the reverse).
- **C43/D43e — the cross-parser law was ONE-DIRECTIONAL over 4 hand-picked good specs** ("runtime-accepts ⇒
  broker-agrees"), so it could see none of C40/C41/C42. **D42/C37's defect in another subsystem: a law that
  proves less than it claims.** Now bidirectional over a hostile corpus (`runtime_ok == broker_ok` + the
  corpus's own expected verdict + field agreement). Each finding observed failing it in the right direction.
- **C44/D43f — `module_requests_native`'s `_ => false`** is correct today (only Module/FnDecl/ActorDecl have
  `attrs`) but gated now — a source-scan of `ast.rs`. **4th instance of the recurring pattern** (C31/C34/C35).
- **C45/D43g — an approved `deploy plan` claimed "within the authority ceiling"** while comparing 1 of 9
  dimensions. Now `EFFECT ceiling` + `compared`/`not_compared` on BOTH surfaces (no-discrimination rule).
- **C46 OPEN, OWNER-RESERVED: should a refused command prove liveness?** Dead-man's remit is "silence, not
  malice"; a controller whose every setpoint is out of range is not silent but IS malfunctioning. Both
  readings defensible → must not be settled by an implementation detail. **Do not decide.**

**VERIFIED-GOOD in Stage 10 — don't re-derive:** (1) **DL1909 cannot be bypassed** — empty / unparseable
garbage / `[authorities]` typo / singular `effect =` / `effects = "Clock"` all give a ceiling of NO effects
(everything refused); unreadable = plain exit 2. (2) **`@jit` leash holds**: DL1906 warns, hint ignored,
`native-emission` in human + `--json`, and **a lease can NEVER confer it** (`exec_native: false` hard-coded
on the lease path). (3) **Two `--grant actuator=` for one device are MET** (tighter wins either order) — the
per-device ambiguity is genuinely handled; only per-TERM was broken. (4) `device_scope`'s lattice is the
best-reasoned module in Stage 10 (whitelist direction documented + tested both ways; meet never wider,
symmetric, preserves ttl≥heartbeat). (5) **Cert authority parse refuses an unknown key/effect/dimension by
refusing the cert WHOLE** — the recurring pattern answered correctly on the READ side. (6) **A long forged
chain is NOT a DoS** — sequential verification dies at the first unanchored/unsigned hop (~1–2 sig checks).
(7) **Single adoption is keyed per CERTIFICATE**, so a subordinate may adopt TWO roots, each separately
bounded/revocable/audited — federation policy, not a defect; the docstring's "once per broker lifetime"
describes the memory's scope, not a one-chain limit.

**METHOD NOTE (cost me two failed attempts): don't put backticks, em-dashes or apostrophes in bash
heredocs/`python -c` strings.** Backticks become command substitution; em-dashes get mangled. Use the
Write tool → a file, then a tiny python script that reads target+addition+anchor from FILES
(`scratchpad/insert.py` is the reusable one). Also: **Linux and Windows must NOT share `target/`** — each
invalidates the other's fingerprints and you get an empty test summary and a nonsense clippy count; set
`CARGO_TARGET_DIR=/tmp/delulu-linux-target` in WSL. And WSL needs `. "$HOME/.cargo/env"` (non-login shell
has no cargo).

**P12 (Scale) DONE — commit `bc47f4a` (D44). Windows 95/1273/0/clippy 65; Linux 95/1277/0/clippy 66;
coverage 100%; reference in sync.** Four closed (C47–C50), two raised (C47b, C51). Corpora GENERATED by
shape, not hand-written (`scratchpad/p12/gen.py`, `isolate.py`, `measure.ps1` — never in the repo).

- **C48/D44b — a QUADRATIC field lookup hidden behind a `.clone()`.** `field_type` cloned the WHOLE
  type definition per field access → N² field-entry deep copies. A 2000-field record took **632 ms**
  vs 173 ms for a 40k-line file 5× its size. **The isolation table is the method worth reusing**:
  decl-alone flat, accesses-alone flat, N-term `+` chain flat, only the PRODUCT exploding → per-access
  work ∝ field count. The clone existed only to release the `self.table` borrow before `lower_type`
  takes `&mut self`; a scoped block fixed it. **632→~42 ms (15×).** Residual `find()` scan is still
  O(fields×accesses) — PUBLISHED with its curve in `measurements/scale/RECORD.md`, next step is a
  name→index map.
- **C49/D44c — THE BIG ONE: an empty `delulu.toml` PANICKED `build`/`check`, and the project's own
  no-panic gate was structurally BLIND to it.** `main.rs` runs the CLI on a 512 MiB-stack worker thread
  (so MAX_DEPTH fires before the native stack) and maps a worker panic to **exit 2** ("internal") —
  deliberate, documented, correct. `json_contract.rs` keyed its sweep on **exit 101**, so it could not
  see ANY crash in the path where all the work happens. That is why this survived P0's sweep, P1's front
  door AND D38's crash hunt. **Both sweeps now match the panic MESSAGE, not the exit code.**
  Root cause: **C26/D33's own fix introduced it** — its "no modules found under `<dir>`" note reaches for
  the root package's dir, and an unreadable manifest fails resolution before a root package exists.
  DL1004 was always computed; the tool crashed while being helpful about something else. `root_pkg()`
  now returns `Option`. **Lesson: a repair needs its own skip-branch analysis — "what if there is
  nothing to name?"**
- **C50/D44d — `delulu authority` reported `summary.errors: 0` + exit 0 for a package `check` refuses**
  (DL1004), byte-identical to a good-manifest report, on BOTH surfaces. `load_package` walks `src/` and
  never opens the manifest. Now: PRESENT manifest is parsed + diagnostics unioned; ABSENT stays legal
  (plain dir of modules, C26/D33). NOT ruled: `authority` also never evaluates the manifest CEILING —
  code doing `Write` under `effects = []` reports clean there while `check` gives DL1009. Flagged, not
  decided.
- **C47/D44a — the normative grammar could not describe `delulu fmt`'s own output.** Every bracketed
  comma list REQUIRES a trailing comma when multi-line (`match` arms excepted); §3 said the opposite in
  BOTH directions — `[ "," ]` where the parser demands one, and no trailing comma at all for params /
  args / record literals / list literals, which the parser accepts and fmt EMITS. An independent impl
  built from the spec would reject every formatted file with a wide list. **Fixed the SPEC** (newline
  rule now normative — an EBNF with no NEWLINE terminal can't express it). **C47b owner-reserved:**
  should the parser accept the comma-less multi-line form?
- **C51 OPEN (first item for P13): `delulu authority <dir>` CANNOT report on any package with a
  dependency** — DL0303, while `build` on the same package succeeds. It uses the single-package loader;
  `build`/`check`/`lock`/`authority --diff` all resolve the graph. Fix identified (resolve the workspace
  here too) but NOT applied: it changes what the authority report CONTAINS for a whole class of
  packages, and that report is a published contract surface.

**VERIFIED-GOOD at scale — don't re-derive:** 30k-line program checks in ~150 ms; peak memory NEVER
exceeded 52 MB anywhere; `atlas` all formats <0.5 s with `digest` byte-stable; `fmt` on 30,009 lines
1.29 s and output still checks; **D38's cap is perfect at scale** — 5,000 real errors → 14 KB/155 ms
human with an honest withheld-count note, `--json` complete at 5,000 with a truthful summary; monorepos
50-deep chains + 200-package diamonds check/build/lock clean; **lockfiles byte-identical across repeated
writes at every size** (the semver-authority law's determinism survives depth).

⚠ **METHOD WARNINGS from P12, all three cost real time.** (1) **A witness that PASSES against the old
code witnesses nothing** — my manifest test's fixture had no dependency, so resolution still loaded the
entry module and the panicking note was never reached; it only became a witness once it gained a sibling
dep (the C19 trap again). (2) **Never conclude "no panic" from `head -3`** — the panic line sat below the
diagnostic, and I twice read absence into truncated output. (3) **A patch script must ASSERT its
replacement applied** — a `python - <<EOF` heredoc turned `\\n` into a real newline, the replace silently
matched nothing, and the script printed success anyway. Use the Edit tool (it asserts) or assert the count.

**P13 (Fuzz / malicious input / security) DONE — commit `1163329` (D45). Windows 95/1277/0/clippy 65;
Linux 95/1281/0/clippy 66; coverage 100%; reference in sync.** Two closed (C51, C52), both
supply-chain, plus a big negative result.

- **C51/D45a CLOSED — `delulu authority` ran the wrong loader.** Single-package loader while
  `build`/`check`/`lock`/`authority --diff` all resolve the graph → DL0303 on EVERY package with a
  dependency, while `build` on the same dir succeeded. **⚠ THE OBVIOUS FIX WOULD HAVE TRADED IT FOR A
  WORSE BUG**: `resolve_workspace` REQUIRES a manifest (DL1004 without one), so routing everything
  through it would have refused a plain directory of modules — legal since C26/D33, preserved by C50.
  **A manifest's presence now selects the loader.** Verified byte-identical no-dep reports (human +
  json, simple + multi-module/secrets/Net) BEFORE shipping. Bonus: a library package with no `fn main`
  now names itself instead of the placeholder `package`.
- **C52/D45b CLOSED — THE BIG ONE: a lockfile could lie about a dependency and `build --locked` said
  "built clean".** `verify_locked` compared recomputed hashes to stored hashes and NEVER looked at the
  recorded `effects`/`cap_kinds`/`secrets`/scope lists — the fields a human opens a lockfile to read.
  **The proof that made it undeniable: `authority --diff` on the SAME forged file says
  `+ effects Net` / `verdict: WIDENING`. The interactive review tool caught what the automated CI gate
  did not — backwards, because CI is where nobody looks.** Four checks added, each restating an
  existing project rule: recorded authority vs computed (DL1002) written as a **DESTRUCTURING
  `let LockEntry {…}`** so a new field can't compile until someone decides (C31/C34/C35/C44 answered
  structurally, not with a 5th list); recorded version vs actual (DL1002); duplicate entry refused not
  resolved (DL1011, the C40 rule 3rd time); unknown lock format version pins NOTHING (DL1011, same
  rule as DL1908's unverifiable algorithm — **no new code needed**, "nothing is pinned" is what DL1011
  already means, and `Lockfile::parse` documents that posture). **15-case tampering matrix: 15
  accepted → 2.** FRAME IT PRECISELY: review-integrity defect, NOT authority escalation — the manifest
  pin bounds a dependency independently of the lockfile.
- **2 residuals NAMED:** (a) a lock entry for a package absent from the resolved graph is still
  accepted (never examined; refusing could break a legitimate superset lockfile); (b) `accepted_by` is
  editable — **VERIFIED to confer nothing** (written by `--accept-authority`, never READ for a
  decision), and not re-derivable, which is why it's excluded from the destructured comparison by name.

**VERIFIED-GOOD in P13 — don't re-derive:** (1) **561 fuzz invocations across 4 parsers** (`.delulu`,
`delulu.toml`, `delulu.lock`, morph TOML) — truncation at 12 offsets, byte flips, deletions,
inflations, injections (NUL, BOM, `1e400`, 200-deep brackets, huge ints), self-duplication × check/fmt/
atlas/build/lock/authority/morph → **no panics, no hangs**. Crashes detected by panic MESSAGE not exit
code (without D44c's lesson the sweep would have been blind). (2) Hash-protected tampering was already
caught on every shape: forged `authority_hash` DL1002, forged/emptied `content_hash` DL1010, deleted
entry DL1011, dependency source edited under a valid lock DL1010. (3) **Deep recursion on the CLI is
DL0905, never a host crash** — C21's residual is about LIBRARY EMBEDDINGS on a small stack, which
`main.rs`'s 512 MiB worker thread mitigates for the CLI; still open, but not a CLI defect.

⚠ **METHOD WARNING (nearly a false catastrophe): the first lockfile attack used plain `delulu build`
and reported ALL 15 TAMPERINGS ACCEPTED.** `build` does not consult the lockfile — **`--locked` is the
verb that verifies it.** Before reporting a surface as unprotected, confirm the command under test is
the one making the guarantee.

**🔓 THE FOUR OWNER-RESERVED QUESTIONS ARE DECIDED — commit `a049605` (D46), 2026-07-26. Windows
95/1283/0/clippy 65; Linux 95/1287/0/clippy 66; coverage 100%.** Jesse gave explicit authority:
*"If there is a problem fix them chef, no need to wait for my approval or permission"*, quoting
C28/C35/C46/C47b by name. **NO OWNER-RESERVED ITEM REMAINS OPEN.** (The standing rule still stands for
anything NEW that is vision-level — this was a specific grant for these four.)

- **D46a — `type A = B` is an ALIAS** (C28). A variant list is signalled **syntactically and ONLY by
  `(` or `|`**; the rule does NOT consult name resolution, so the grammar stays context-free. One-line
  fix in `looks_like_variant` (drop `Term | Eof` from the lookahead). `type U = Nothing()` is the
  spelling for a single field-less variant. **Zero single-variant field-less sums in the tree**, so
  nothing changed meaning.
- **D46b — `computes` CROSSES an actor boundary** (C35). The decisive argument: **actuators and sensors
  already crossed, and actuation moves physical machines** — refusing the less consequential dimension
  while permitting the more consequential one was an omission, not a safety position. (The old comment
  even said "fail closed, like the actuator list" one line above where the actuator list crosses.)
  `WITHHELD_FROM_ACTORS` is now **EMPTY**; the drift gate STAYS because "nothing is withheld" must keep
  being true. Witness is BEHAVIOURAL (round-trip through `value_to_msg`/`msg_to_value`) — a source scan
  cannot see a conversion dropping a field it declares.
- **D46c — a refused command does NOT prove liveness** (C46). Ruled toward the stricter reading, which
  is what the code already did. **No behaviour change** — what changed is that it is now a decision
  with a reason in spec §5.2 rather than an accident.
- **D46d — a multi-line bracketed list needs NO trailing comma** (C47b). All four spellings parse in
  all NINE bracketed lists. **It was never a design decision**: §2.2 inserts a `Term` only after a
  token that can end a statement, and a comma cannot — so `a,\n)` always parsed and `a\n)` did not,
  purely because that one terminator was never skipped before the closer. Hence a 9-line fix
  (`skip_terms_before_closer`), not a grammar redesign.

**C53 FILED OPEN (found in D46a's migration check, queued for P15): an UNUSED type alias is never
resolved.** `type Meters = Metres` (typo) checks clean; DL0301 fires only at a use site — and in a
library that never uses it, on the consumer. **PRE-EXISTING, not caused by D46a** (verified:
`type X = (Nonexistent)` and `type Y = List[Nonexistent]` were always accepted too). Fix = resolve the
alias target at its declaration; two hazards deserve their own budget — **forward references** must
keep working and **cycles** (C16, still open) would hang an unbounded walk, the trap that made D31's
alias walk bounded at 32 hops.

**P14 (Performance / memory) DONE — commit `0603f05` (D47). Windows 95/1286/0/clippy 65; Linux
95/1290/0/clippy 66; coverage 100%; reference in sync.** Two closed, one earlier verdict CORRECTED, two
limits measured and published.

- **C54/D47a — THE BIG ONE: a USED cyclic type alias ABORTED THE COMPILER with a stack overflow.**
  `type A = A` + `fn f(x: A)` → `has overflowed its stack`, exit `0xC00000FD`. Same for `type A = B;
  type B = A`, `type A = List[A]`, `type A = iso A`, `type A = fn(A) -> Int`. `lower_type` expands an
  alias by RECURSING, so a cycle is unbounded recursion. **This CORRECTS C16** — P2 filed cyclic
  aliases as "hygiene" on the evidence that 5000-deep TERMINATING chains resolve and secrets can't
  launder through a cycle (both still true); **it never tested a cycle that is USED**, and the
  declaration alone is harmless precisely because nothing lowers it. C16 closed with its severity
  corrected, not left at "low".
  **⚠ AND THE THIRD BLIND GATE: a stack overflow prints NO `panicked at`**, so the no-panic sweeps
  (which match exactly that message, per D44c) could not see it. Running tally of gates blind to their
  own failure: D42a (coverage law proved existence, not exercise), D44c (keyed on exit 101 while a
  worker panic maps to exit 2), D47a (matched a panic message against a failure that prints none).
  **THE DURABLE LESSON: ask what SIGNAL a gate keys on, then ask what failure produces a DIFFERENT
  signal.**
  Fix: `Checker::check_type_aliases` runs BEFORE anything lowers a type, reports **DL0304** per
  participating declaration naming the chain (`A = B = C = A`), and `lower_type` consults
  `cyclic_aliases` and refuses to expand — structurally impossible, not merely diagnosed. **Only
  alias→alias edges are walked**, which is exactly why recursive RECORDS and SUMS stay legal (nominal,
  never expanded) — both tested. **DL0304 GENERALIZED, not duplicated** ("a cycle in the declaration
  graph (imports, or type aliases)") — the DL1511/D42b discipline; no new code, no new conformance
  witnesses needed.
- **C53/D47b — an UNUSED alias target was never resolved.** Same pass lowers each target at its
  declaration (DL0301). Reuses the REAL resolver rather than duplicating the builtin-name list — that
  duplication is the drift shape closed four times already. Forward refs, 200-deep chains and generic
  aliases all still work, all tested.
- **C55 NAMED LIMIT (D47c) — runtime record field access is O(record WIDTH) per read.** The interpreter
  does NOT have C48's clone-per-access shape (`Interp::field` clones only the value found) but does
  scan linearly. Reads held CONSTANT at ~200k, width varied: **165 → 300 → 1,570 → 5,071 µs/1k reads at
  50 → 200 → 800 → 3,200 fields.** Published NOT fixed: linear (not quadratic), small constant, and for
  5–20-field records a short `Vec` scan is the FASTER representation. Real fix = static field indices
  through the DIR.
- **C56 OPEN (measured, published) — `--trace-effects` buffers the ENTIRE trace in RAM.** 100k effects:
  peak 6.7 MB → **70.1 MB** (~633 B/record), unbounded. ≈2.3 GB/hour at 1k effects/sec — and Stage 10's
  domain is controllers running for hours, which is exactly when you'd enable it. **The audit chain
  already streams to day files; the trace does not.** Fix = stream (better) or bound with an honest
  notice (the D38 pattern); streaming must preserve the actor runtime's per-turn causality stamping.

**VERIFIED-GOOD in P14:** the performance honesty clause is intact everywhere ("competitive with C on
hot paths… only where a published benchmark shows it. Where DeluluLang loses, the table says so") — and
C55/C56 are published under exactly that rule. The audit chain's unbounded growth is BY DESIGN (day
files, append-only; a prunable audit log is not an audit log). The reference-staleness gate caught the
DL0304 title change — the gate working.

**P15 (Authority + Guard CAPSTONE — the commission's highest-priority phase) DONE — commit `e5bc242`
(D48). Windows 95/1287/0/clippy 65; Linux 95/1291/0/clippy 66; coverage 100%; reference in sync.**

**📕 THE DELIVERABLE IS A STANDALONE DOC: `docs/design/AUTHORITY_GUARD_CAPSTONE.md` (249 lines).** The
18-item authority checklist and the 14-item Guard surface checklist, discharged ITEM BY ITEM with what
was attacked / what the evidence is / where it lives, and a §3 stating what the audit did NOT settle in
the same voice as the successes. **Read this before re-deriving any authority claim.**

- **C57/D48a CLOSED — the authority SERIALIZATION seam had no gate.** `Authority::to_json` (write) and
  `authority_from_json` (read) are two hand-enumerated lists of the same 8 dimensions on opposite sides
  of a certificate / audit record / `--json` report. The READ side already refuses an unknown dimension
  by rejecting the whole cert (RFC §4.9.3, tested); the WRITE side had NOTHING. Fix = a round-trip gate
  that **destructures `Scopes`** so a new field fails to COMPILE. Verified non-vacuous by deleting
  `foreign.python` from the writer. Why it mattered despite being fail-closed: the authority in every
  hash-chained AUDIT record would silently UNDER-REPORT, and `render_compact` feeds DL0802's repair
  text — the C29/C30 accountability class. **6th instance of the recurring pattern.**
- **P6's "revocation racing an in-flight op" DISCHARGED — there is NO data race to find.** The broker
  daemon is **single-threaded and serializes at REQUEST granularity** (one accept → one frame → one
  handle → one response; the only `thread::spawn` in brokerd.rs is a `#[cfg(test)]` helper, and
  `cmd.spawn()` is the detached-daemon launch). What remains is the LOGICAL window already documented:
  effective before the next USE, never retroactively — measured 12.7 ms p50 / **39.7 ms worst**, tested
  e2e WITH a control in `estop_cli.rs`. Debt paid by demonstrating the bound, not by finding a bug.
- **3 of the 14 Guard surfaces DO NOT EXIST in v1.x** and saying so IS the honest discharge: optimizer
  (spec §2.1 describes it, never implemented), native backend (leashed — `exec_native: false` is
  hard-coded on the lease path so the leash holds across federation), distributed execution (not a
  separate surface; federation + a single-host actor runtime). Two of the other eleven carry NAMED GAPS
  rather than clean passes: the adapter has no signature check, and hardware has never been exercised.

**⚠ TWO FAILURE SHAPES PROMOTED TO DESIGN RULES (D48d) — these are the campaign's durable output:**
1. **A hand-maintained list of authority-bearing things falls behind the type that defines it.** SIX
   instances: C31, C34, C35, C44, C52, C57. The answer is NEVER "remember to update the list" — it is a
   compiler-enforced pattern (destructuring) or a source-scanning gate. Where a dependency edge permits,
   better still: ONE list referenced by both sides (`device_scope::FAIL_STATES`).
2. **A gate is blind to the failure it exists to catch.** Three + a near-miss: D42a (proved a witness
   EXISTED, not that it EXERCISED), D44c (keyed on exit 101 while a worker panic maps to exit 2), D47a
   (matched `panicked at` against a stack overflow, which prints none), D43e (checked agreement only
   where both sides said YES). **THE RULE: ask what SIGNAL a gate keys on, then ask what failure
   produces a DIFFERENT signal.**

**P16 (close-out) DONE — commit `adac789` (D49/D50). 🏁 THE CAMPAIGN IS COMPLETE.** Windows
95/1289/0/clippy 65; Linux 95/1293/0/clippy 66; coverage 100%; reference in sync.

- **C56/D49 CLOSED — the trace buffer is bounded, and the ASYMMETRY IS THE RULING** (do not
  "simplify" it later): `--trace-effects` alone gets `TraceSink::bounded(200_000)` keeping records
  from the FRONT with the withheld count reported; **`--assert-trace` is NEVER capped**, because it
  proves no effect outside the declared set occurred and a dropped record could hide a violation — a
  fail-OPEN on a security-adjacent check. Front, not a ring buffer, so two runs can't disagree.
  Measured: 500k effects → 134 MB (from ~316 MB) + `300000 further effect record(s) not traced`.
- **D50 — both platforms RE-VERIFIED from the committed tree** (the 2026-07-21 figures predated 16
  phases). **Clippy UNCHANGED at 65/66 across ~4,000 added lines.** P1's front door re-tested from a
  **fresh `git clone`**: build → check → authority → run → DL0703 without the grant. All as documented.
- **macOS: NEVER EXECUTED, not once, in any phase.** Every cell reads "never run", not "untested" or
  "pending" — those invite a reader to assume someone tried.
- Honesty scrub: "quantum-proof"/"quantum-safe" ×12, **every one inside a prohibition sentence**; SBOM
  **17 declared / 17 listed, zero drift** (re-checked by PARSING `[dependencies]`, not grepping — P10's
  method warning); **all 9 measurement records now dated** (4 were not, including the one THIS campaign
  wrote, which had itself argued a table should say when it was taken).

**📊 FINAL LEDGER (counted by script, not estimated): 58 findings filed — 53 CLOSED, 1 published limit
(C55 O(record-width) field access), 1 DOES NOT REPRODUCE (C12), 3 OPEN: C7 (corpus tiers, an evidence
gap), C17 (float→inf silent, matches C/JS), C21 (REFINED — the CLI is safe at 100k frames on the
512 MiB worker thread; only a small-stack library EMBEDDING is uncovered).**
⚠ My first draft of that paragraph said "57/48/3/4/2" — written from memory, wrong on every count. The
correction is recorded IN the doc rather than quietly fixed.

**⚠ A THIRD PATTERN, no ruling but worth carrying: THREE of this campaign's findings were INTRODUCED BY
EARLIER FIXES IN THIS SAME CAMPAIGN** (C26/D33's courtesy note caused C49's panic; D46a's grammar change
surfaced C53). **A repair needs its own skip-branch analysis.**

**POST-CLOSE-OUT: the three follow-ups Jesse authorised ("Yes my head chef continue and do it") —
commit `ed1b027` (D51/D52). Windows 95/1292/0/clippy 65; Linux 95/1296/0/clippy 66.** Two done, one
honestly not done.

- **C21 CLOSED (D51) — the interpreter's depth bound is now a CONTRACT, not an undocumented
  requirement.** `Interp::with_max_depth(n)` + public `DEFAULT_MAX_DEPTH` / `STACK_BYTES_PER_DEPTH`;
  default unchanged. Witness runs the guard on a deliberately small (8 MiB) thread.
  **⚠ THE PER-FRAME COST WAS MEASURED AND C21's RECORDED FIGURE IS ~10× TOO LOW.** Bracketed on 8 MiB:
  **16 KiB/depth OVERFLOWS, 40 KiB/depth OVERFLOWS, 80 KiB/depth is CLEAN** (debug — what an embedder's
  tests run). C21 said "10,000 frames need more than 16 MiB", which is TRUE but reads as if 16 MiB were
  nearly enough. **My first draft of `STACK_BYTES_PER_DEPTH` took it literally, derived 2 KiB, and would
  have advised an embedder into the exact crash the contract prevents.** Release is cheaper (~52 KiB,
  from main.rs's 512 MiB); a test asserts the published budget covers what main.rs reserves.
- **D52 — the adapter signature gap NARROWED (not gone).** Four branches, the asymmetry IS the ruling:
  (1) present-but-INVALID refuses **regardless of policy** (DL1510, Stage-6 deviation 8's rule);
  (2) absent = policy question, allowed + disclosed loudly; (3) `--require-signed-adapter` → DL1511;
  (4) **first token not a readable file → says it COULD NOT CHECK.** That 4th branch matters: an
  interpreter-hosted driver (`powershell -File x.ps1`) names the INTERPRETER, so verifying the first
  token would vouch for the wrong bytes — **passing silently there would be worse than not checking,
  because it would look checked.** No new code minted, no crypto hand-rolled. **Still an
  operator-supplied subprocess, NOT spec §5.4's Verified-class signed plugin: signing buys PROVENANCE,
  not behaviour.**
- **macOS — NOT DONE, and not for want of a decision: there is no Mac.** What was done is a
  READINESS audit recorded explicitly as NOT evidence (`CROSS_PLATFORM_VERIFICATION.md`): macOS takes
  the `unix` branch (shared with Linux, green), the not-windows branch (same as Linux), and the
  not-linux microVM refusal DL1408 (same as Windows); the three FFI arms are mutually exclusive so
  nothing collides; **ONE arm (the `libm.dylib` FFI test) is macOS-exclusive and has never compiled or
  run anywhere.** A path that SHOULD work and a path that HAS been run are different claims.

**📊 LEDGER NOW: 58 filed — 54 CLOSED, 1 limit (C55), 1 no-repro (C12), 2 OPEN: C7 (corpus tiers,
evidence gap), C17 (float→inf silent, matches C/JS).** Counted by script.

## Post-close-out pass 2 — 2026-07-26, commit 7e5e9ba (D53-D57), Opus 5

Jesse: "fix c7 and c17 too ... Fix everything remaining" + asked whether D52's four-branch adapter
gate was right for a production/future language. **It was not.** Closed C7, C17, C55's extrapolation,
C62. Opened **four**: C58, C59, C60, C61.

- **D53 - the adapter hole, found by taking Jesse's question seriously.** `verify_detached` reads the
  public key out of the FIRST 32 BYTES OF THE SIGNATURE FILE, and the `.sig` sits beside the driver.
  An attacker who can overwrite `drive.exe` overwrites `drive.exe.sig` too, self-signed, and D52's
  strongest flag ACCEPTED it. `--adapter-signer <hex>` now pins the key (other signer = DL1510);
  `--adapter-artifact <path>` names which bytes were signed (without it `--require-signed-adapter`
  was UNUSABLE for every script-hosted driver); an unpinned verify now states it is not a trust
  decision. Witness plays the attack out before fixing it.
- **D54 - C17 was mis-framed for the whole campaign.** Filed as "matches C/JavaScript, low". The real
  comparison was the arm of the SAME function 12 lines below, which always refused an oversized Int
  literal. And the half never filed is worse: from_str also FLUSHES TO ZERO (`1.0e-400` -> `0.0`) -
  a silently zeroed gain makes a control law quietly do nothing. Both DL0104. Subnormals accepted.
- **`parse_float` added** - the language could not read a Float out of text at all. It calls the SAME
  function the lexer calls (`delulu_syntax::num::float_from_text`); written separately the obvious
  impl accepts the WORD `inf`, letting a data file inject infinity the source may not write (C40's
  shape).
- **D55 - tier 4 built**: 4 packages / 7 modules / depth 3 / diamond. Harness now BUILDS packages;
  `corpus_cli.rs` RUNS three programs and asserts output. **`pub import` is how a package re-exports
  a type to consumers**; a transitive dep is NOT importable.
- **D56 - C55 stays a limit but its reasoning was an extrapolation** (claim about 5-20 fields, table
  started at 50). Measured: U-SHAPED, minimum 188 us/1k at width 20. Field scan is not the cost there.
- **D57 - `.gitattributes` LF invariant had NO gate** and one file (HARDENING_CAMPAIGN.md) was stored
  CRLF for the whole campaign. Found by noticing a 1,927-line diff on a 102-line change. Gate added
  (`governance.rs::no_tracked_text_file_is_stored_with_crlf`, reads the INDEX).

**Ledger now: 62 filed, 56 closed, 1 limit (C55), 1 no-repro (C12), 4 OPEN.** Windows 96/1,305/0
clippy 65; Linux 96/1,309/0 clippy 66; coverage 100%. macOS STILL NEVER RUN.

**The four open, in priority order — this is the handoff:**
1. **C59 (high) - A MULTI-PACKAGE PROGRAM CANNOT BE RUN.** `delulu run` takes one file or a `.dwx`;
   `build` emits only `interface.json`. `kind = "bin"` is declarable and unexecutable — true of the
   shipped `examples/greeter/` too. Largest capability gap in the project. Designed but NOT built:
   `Interp` keys `funcs` by BARE NAME in one flat HashMap; multi-module needs per-module resolution,
   and `check_workspace` already computes `call_owner` (module -> name -> owning module), which is
   exactly the map required. Risks: closures capture a defining module, `Value::Record{name}` uses
   bare names, actors, and `--engine wasm` would need DL1201.
2. **C58 (medium)** - a `pub fn` naming a type the package doesn't re-export builds clean alone, then
   fails INSIDE THE DEPENDENCY'S OWN SOURCE saying a type is "not a type" where it IS in scope.
3. **C60 (medium)** - adapter provenance verdict is printed, not recorded. Neither existing home
   fits: audit chain is a no-op without a sink; DL1905 sign-off is written by a SIMULATION, before an
   adapter exists.
4. **C61 (low)** - `let _ = expr` refused though `_` is a valid match pattern; discarding still works
   under any other name, so the rule buys nothing.

**Method warnings earned this pass:** (a) building for Linux in the Windows tree fails in
`libffi-sys` configure on DrvFs — use `CARGO_TARGET_DIR=$HOME/delulu-target`; (b) a `tail -70` in a
counting pipeline silently reported 70 suites instead of 96 — **third truncating-pipe wrong number of
this campaign. Count first, truncate never.**

## Post-close-out pass 3 — 2026-07-26, commit 550973e (D59/D60), Opus 5

Jesse asked four things, each answered by RUNNING it: do authority/Guard/atlas work on both CLI and
compiler; write and test a 10k-25k line program; test interoperability with other languages.

- **D59 (C65/C66)** — `delulu authority` could NOT read a `.dwx`, the format you ship. `run` read the
  embedded manifest fine; `authority`/`check`/`why`/`atlas` died on `stream did not contain valid
  UTF-8`. Now `authority <x>.dwx` reports it through **the same `read_and_verify` the runner uses** (a
  witness flips a byte -> DL1202 from both). Detection by magic bytes, not extension. `fmt notes.txt`
  no longer reports success on a file it ignored.
- **D60 (C67/C68)** — (a) the authority report printed 2,536 pure-fn names on ONE 28,242-char line;
  D38's rule (bounded human / complete machine) now applies: 912 bytes. (b) **The Book's code-block
  gate compared COUNTS, never contents — 0 of 10 blocks were slices of a compiled sample**, and
  Chapter 14 taught `root.foreign[mathlib](...)` which does NOT compile. Fixed + correspondence gate.
  **This is C37/C43/C63/C68 = one pattern: ask what a gate looks like if the thing it guards were
  completely wrong.**
- **C64 FILED, NOT FIXED (the top item for anyone writing DeluluLang).** `let p = P{x:1}; f(p)` is
  DL1603; inlined, returned-from-a-call, or match-bound is fine. Extract-variable breaks programs. Hit
  3x in one session. Mechanism: `fresh_lift` is carried onto the binding and consulted ONLY in
  `check_return_position`, never at an argument; `subcap(Iso,Val)` already holds. Fix needs to demote
  the caller's write access when it lifts (else `val`'s immutability is violated), plus the negative
  witness `f(p); p.x = 5` must still fail. Soundness-bearing -> own pass.

**KEY FACTS ESTABLISHED (do not re-derive):**
- **The WASM backend is a NARROW fragment, measured**: Int/Bool/Str arithmetic, if/else, calls,
  recursion, `match` on a sum, Str concat, `str(Int)`, console. **DL1201 for `while`, Float, records,
  lists, all string methods, clock, `.narrow`, `.fs_write`, Python.** 6 of 19 entry-point programs
  compile. The interpreter is the language.
- **Interop WORKS on Windows**: C FFI vs `msvcrt.dll`/`ucrtbase.dll` (cos/sqrt/pow correct); embedded
  CPython `statistics.pstdev`=2.0, base64 round trip. Foreign handle syntax is
  `root.foreign(root.foreign_load())` with the **lib type annotated on a PARAMETER** (no method
  type-args exist). Grant syntax: `--grant foreign.c=LIB:PATH`, `--grant foreign.python=MODULE`.
- **Secret grant syntax is `secret:NAME=VALUE`; `env:VAR` is the opt-in env source.** `secret:X=env`
  sets the literal string "env" — my usage error, not a bug.
- **`--lease` runs accept NO `--grant`** (the lease IS the authority).
- **Guard verified live**: broker start seals declassify/foreign_c/foreign_python, prints an owner code
  once; delegate -> redeem -> revoke -> re-redeem gives DL1403 and the refusal is audited as `deny`.
- 25k-line generated program: checks 1,492 ms, runs correctly, **2,536/2,539 fns proved pure**.

**Ledger: 68 filed, 61 closed, 1 limit, 1 no-repro, 5 OPEN (C58, C59, C60, C61, C64).**
Windows 97 suites/1,312/0 clippy 65; Linux 97/1,316/0 clippy 66. macOS STILL NEVER RUN.

**Method warning (4th time this campaign): a truncating pipe gave a confident wrong number** — this
time `tail -70` reported 70 suites of 96. Also: heredoc'd Python collapses `\` to `\`, which put a
real NUL byte into CHANGELOG.md; build replacement bytes numerically (`bytes([92,48])`) when escaping
matters. **Prefer the Edit tool over heredocs for anything with backslashes.**

## Post-close-out pass 4 — 2026-07-26/27, commits d66e20b / 9249aca / 3ad1fdb (D61-D63), Opus 5

Jesse: "Is anything left to verify and integrate and fix... Implement and harden everything. This is a
real lang and lang of the future." Swept all 59 .md files for deferral markers (96 mentions), triaged,
and closed the three highest-value open findings. **NO degradation of anything — full build
parallelism, full suite, both platforms, every pass.**

- **D61 (C59) — A MULTI-PACKAGE PROGRAM NOW RUNS.** `delulu run <package-dir>`. The largest capability
  gap the campaign found; `examples/greeter/` had shipped un-runnable since Stage 2. Order is the
  ruling: `check_workspace` stays authoritative, and only a program that passes it is flattened.
  **Flatten SOURCE, never merge ASTs** — separately-parsed modules have OVERLAPPING NodeIds, so every
  checker table keyed by node id reads one module's entry for another's expression. The first
  implementation did that and produced a nonsense rcap error about a correct program: a WRONG ANSWER,
  not an error. Caught by disbelieving the diagnostic and hand-flattening 7 modules into one file,
  which checked clean. A cross-module top-level name collision is REFUSED (fail-closed) with "the
  program is not wrong"; lifting needs per-module resolution in `Interp` (`Program::call_owner` already
  has the map). Also makes STAGE6_BUILD_ORDER's "multi-module plugin packages deferred — needs
  `check_program`" deferral STALE: `check_program` exists.
- **D62 (C64) — `let p = P{x:1}; f(p)` works.** `fresh_lift` was carried onto the binding and consulted
  ONLY in `check_return_position`. Three clauses: lift at `val` args; **the lift COSTS the caller its
  write access** (a later `p.x = 5` is refused — witnessed); **an author-written `ref` is NEVER lifted**.
  That third clause exists because the first version lacked it and AN EXISTING REGRESSION TEST caught
  it (the `val`-parameter laundering channel). The test was right, my fix was wrong. **Lesson: a
  loosening of a soundness-bearing rule checked only by the author's own new tests is a loosening
  nobody has checked.**
- **D63 (C61) — `let _` / `var _`.** Safe BY CONSTRUCTION: a bare `_` lexes as `TokenKind::Underscore`,
  never an `Ident`, so nothing can read it. The conformance witness first cited a FAKE anchor
  (`ref.grammar.let_stmt`) and the coverage law **failed closed** on it — D42a earning its keep.

**Ledger: 68 filed, 64 closed, 1 limit (C55), 1 no-repro (C12), 2 OPEN.** Windows 98 suites/1,326/0
clippy 65; Linux 98/1,330/0 clippy 66; coverage 100%; reference in sync. macOS STILL NEVER RUN.

**Both of these are now CLOSED — see pass 6 below.** Kept for the shape of what they were:
1. **C58** — a `pub fn` naming a type its package doesn't re-export builds clean alone, then fails
   INSIDE THE DEPENDENCY'S OWN SOURCE saying a type is "not a type" where it IS in scope. The rule
   (`pub import` = explicit re-export) is right; the diagnostic blamed the wrong line in the wrong
   package. **Closed by D65.**
2. **C60** — the adapter provenance verdict is printed, not recorded. **Closed by D66** — and the
   premise "neither existing home fits" was half wrong: the broker's audit chain is complete and has
   a reader; nothing was writing the adapter decision into it.

**THE CAMPAIGN IS CLOSED. Do not re-arm a phase timer.** Durable artifacts: `HARDENING_CAMPAIGN.md` §6
(close-out), `AUTHORITY_GUARD_CAPSTONE.md` (the 18+14 discharge), `CROSS_PLATFORM_VERIFICATION.md` §8,
`measurements/scale/RECORD.md`, and rulings D24–D64 in `STAGE10_BUILD_ORDER.md`.

Everything the close-out had deferred is now closed: C54 (interpreter MAX_DEPTH vs host stack), C16
(cyclic alias), C17 (float→inf), **C12 (D64, pass 5)**, C7 (corpus tiers).
**OWNER-RESERVED, awaiting Jesse — four, and they are DESIGN questions, not defects: C28** (`type A =
B` grammar ambiguity), **C35** (should `computes` cross an actor boundary?), **C46** (should a refused
command prove liveness?), **C47b** (should a multi-line list require its trailing comma?). Do not
decide these; present and wait.

## Post-close-out pass 5 — 2026-07-27, commit 80f5e8a (D64, closing C12), Opus 5

**Started as a request to EXPLAIN the four remaining findings; re-running the reproductions instead
of trusting the ledger is what turned it into a fix.** C12 was recorded **DOES NOT REPRODUCE**. It
reproduced verbatim on the first try (`expected Result[Str, T0], found Result[Str, T1]`).

**Why the clear was wrong, and the transferable lesson:** the re-test that cleared C12 used the
monomorphic and generic shapes — `Int` vs `Str`. Those print themselves. `Type::Record`/`Type::Sum`
are the ONLY variants stored as a table **index**, so they were the only shapes that could fail and
the only ones not re-tested. **Campaign rule 2 (a check keyed on a signal the failure does not
produce), applied to the campaign's own ledger.** "Does not reproduce" did real work there: it let an
everyday defect sit behind a verdict that reads like diligence. **When re-testing to CLEAR a finding,
the test must cover the shape the finding is about, not a shape that resembles it.**

**The finding was also filed far too narrowly** — not a `Cap[Http]` corner but *every user-declared
type in every message* (`expected T11, found T12` for `Verdict` vs `Status`), plus three surfaces
nobody had connected to it: the **LSP hover**, the **REPL**, and **`interface.json`** — the published
machine-readable artifact whose stated purpose is agent introspection, which said
`"type": "fn(T9) -> Float"` and is checked into the tree. The durable artifact was the worst of the
four.

**Fix shape (D64), and the part worth reusing:** `impl Display for Type` was **DELETED**, not
repaired — rendering now requires supplying names (`t.show(&names)`), so the compiler enumerates every
site. That is what found all 22, three of which nobody would have gone looking for; a `display_with`
helper added *alongside* `Display` would have fixed the remembered sites and left the rest, which is
exactly how the four surfaces drifted apart. Campaign rule 1 in its strongest form. The no-table case
(runtime plugin loader, types recovered from a DIR) renders `<type #11>` — not better information,
better **honesty**, because `T11` is spellable by an author and reads as an answer.
**`api_row_hash` is computed from the AST (`TypeExpr`), not from `Type`, so the corrected
`interface.json` left every hash byte-identical — verified, so no lockfile moved.** The two had been
disagreeing inside one file.

**Also chased and DISCARDED a false pattern** (recorded because it was convincing): 4 of 5 laptop
BSODs had a Kernel-Power "power source change" within ~20 s — all boot enumeration. See
[[laptop-bsod-mitigation]].

Windows 98 suites / **1,330** passed / 0 failed; clippy **65** (baseline held). Ledger now
**68 filed, 65 closed, 1 published limit (C55), 2 open (C58, C60)** — and **nothing is carried as
"does not reproduce" any more.** The 4-package/7-module corpus program was re-run end to end and is
correct (its `data/` input is NOT tracked — recreate `sensor=celsius` lines to run it).

## Post-close-out pass 6 — 2026-07-27, D65/D66 (closing C58, C60, C69), Opus 5

**THE LEDGER IS NOW EMPTY: 69 filed, 68 closed, 1 published limit (C55), 0 open.** Jesse: "Do all
the necessary fixes Chef. This is a lang of future."

**C58 → D65.** The tempting fix was measured and REJECTED, and that is the transferable part. Rust's
private-in-public rule (refuse the `pub fn` where it is written) does **not** hold here: DeluluLang
lowers an imported signature in the **importer's** scope, so a consumer that independently imports
the declaring module can use the function fine. **Three shipped tier-4 modules do exactly that**
(`reading.parse`, `policy`, `archive` all name `Sample` behind a plain `import` and build clean) — the
rule would have outlawed the diamond the tier exists to demonstrate. **Checking the corpus before
writing the rule is what caught it.** The rule was never wrong; only the report was. Fix: a
foreign-file diagnostic is re-framed onto the import that brought the signature in; an own-file one
keeps its precise span and only gains the answer; **a misspelling gains nothing** (no home = no true
advice). Plus dedupe by (code, message, span). 25 errors at 14 locations → 15, and a witness applies
the printed advice and requires a clean build.

**C60 → D66.** Wrote the adapter provenance decision into the broker's **existing** hash-chained
audit log (`action: adapter.provenance`) rather than inventing an artifact — the earlier "neither
home fits" reading was half wrong: the chain is complete, has `verify|tail|query|bundle`, and a
default location; nothing was writing INTO it. **Refusals recorded BEFORE the refusal is acted on**
(witnessed by inverting the order and observing the failure).

**C69 — THE ONE TO REMEMBER, found by causing it.** The first D66 defaulted the sink to the shared
`~/.delulu/audit`. **A hash chain has exactly one writer** (`AuditLog::open` reads the head, then
appends): the broker is one long-lived process and satisfies that; **`delulu run` is short-lived and
many run at once.** The parallel test suite promptly produced a chain that failed `delulu audit
verify` — `prev_hash` break plus **two physically interleaved half-lines** — and it polluted Jesse's
real `~/.delulu/audit` with 13 test records. Caught by `cli_contract`'s
`every_listed_subcommand_honors_a_valid_invocation`, **a gate written for something else entirely.**
Fix: no default sink at all; `--adapter-record <dir>` or nothing (with a warning). The polluted
`20260727.jsonl` was **moved, not deleted**, to the session scratchpad; Jesse's real 2026-07-26 file
(12 broker records) was never touched and `delulu audit verify` is green again (head `06cacd9d`).
**Rule: a fix that damages the artifact it was written to create is worth filing, not quietly
correcting.**

**Method notes that cost time this pass:** (1) a Python patch script's Rust `\\`-line-continuations
were consumed by **Python**, baking 18 spaces into a user-facing message — the recurring escaping
trap in a new costume; use the Edit tool for anything containing a backslash. (2) Inserting a new item
with an anchor on `fn name(` places it **between the function's doc comment and the function**,
silently re-parenting the doc (clippy `doc_lazy_continuation` caught it, +1 over baseline). (3) A test
fixture that merely *imports* a function whose parameter type it cannot name checks **clean** —
signatures are lowered per call site, so the witness must actually CALL it.

Windows 98 suites / **1,338** passed / 0 failed; clippy **65** (baseline held).

## Long-term production development — 2026-08-02, Opus 5 (5-phase roadmap, autonomous)

Jesse moved from "harden what exists" to a **5-phase roadmap**, with explicit authority to
restructure it: *"Always choose the best architecture, not necessarily my architecture. Think like
the Chief Architect."* Phases 1–3 shipped, 4–5 open:

| # | Phase | Commits |
|---|---|---|
| 1 | Language server — analysis colocated with the document (killed a 32× re-check AND a nondeterminism bug at once), incremental sync, workspace symbols, workspace-wide definition/references, rename that REFUSES when the name lives in unopened files, signature help | `a57416e` `2a718cd` `f9bb622` `5eb3302` `15f6a97` |
| 2 | Authoring — `delulu fix`, `delulu new`, `delulu completions`, `delulu add --path` | `43e3e9d` `56f91af` `bfda948` `b6da04d` |
| 3 | Diagnostics with no dead ends — `Disposition` for the 14 codes this compiler cannot emit; `explain` answers instead of saying "unknown" | `5fefef6` |
| — | **Core-invariance gate** (below) | `abf547a` |
| 4 | Performance — measured the FLOOR nobody had measured; multi-file `check`; allocation-scaling gate | `783a4b2` |
| 5 | AI-native Survey queries — `impact`/`affected-by`/`path` built; `why`/`owners` DISPROVED | `75fb032` |

**ALL FIVE PHASES CLOSED 2026-08-02.** Windows 112 suites / **1,460** passed / 0 failed / clippy
**65**; Linux 112 / **1,464** / 0 / clippy **66**; coverage 100%, reference in sync, fmt 0-change,
`doctor --check` 12/12 on both; Survey 920 nodes / 8,164 edges / 6 notes. macOS NEVER EXECUTED.

**PHASE 5 — the transitive verbs, and the rule that makes them admissible.** `rdeps` is ONE HOP;
`mod:crates/delulu-check/src/check.rs` has **one structural** edge arriving and reaches **134**
transitively — and the Survey's own README told readers to reach for `rdeps` first. Now:
`impact <id>` (reverse walk), `affected-by <id>` (forward), `path <a> <b>` (one chain), `--depth N`.

**THE DESIGN FINDING, and it is the reusable one: an EDGE being cited does not make a CHAIN true.**
The first version composed every edge kind and reported **236 nodes reachable from EVERY node in
the repo**, including `doc:README.md`. Narrative edges (`links-to`, `cites`, `references`,
`documents`) connect everything to everything: "README links-to CONTRIBUTING" + "CONTRIBUTING
references cli.rs" is two unrelated sentences end to end. Fix: `EdgeKind::composes()` — exhaustive,
so a new kind cannot compile until classified — and a walk follows only propagating relations
(depends-on, declares, uses, tests). Numbers now order correctly: delulu-diag 164 > delulu-syntax
148 > check.rs 134 > delulu 62; a code and a doc reach **0**.

**DISPROVED BY INSPECTION, not built** (the `atlas`/`graph` lesson applied): `why <id>` = a filtered
subset of `query`'s existing output; `owners <id>` = `.github/CODEOWNERS` where every rule names the
same placeholder `@PENDING-PUBLIC-project-lead` (unassigned until public launch) = a constant
function. Both recorded in `docs/survey/README.md` §"What it will not do". **NOT built, named:**
CODEOWNERS marks which paths are ENTRENCHED (constitution, STABILITY, /rfcs/, soundness audit,
conform machinery) — belongs as a node attribute, not a verb.

**Observability effect worth remembering: documents are part of the map, so writing about a file
changes its in-degree.** `check.rs` went 1 → 8 incoming edges purely because the change notes for
this feature name it. Caught by my own test failing. Compare like with like — structural in-degree
vs a structural walk — and the caveat is now in the survey README.

**PHASE 4 — the finding is that the compiler was never the cost.** Every table this project
publishes measures the *marginal* cost of size (`scale/`, `study-c`). None measured the **floor**:
what one invocation costs on a program small enough to be free — which is exactly what an agent
pays per edit→check iteration. Decomposed with controls that each add one layer (a Rust binary whose
whole source is `fn main() {}`, then `main.rs`'s 512 MiB-stack thread, then `--version`, then
`check`). **On a 35-line file, 26.9 of 32.8 ms — 82% — is Windows creating a process**; Linux 46%.
Compiler's own work <1.5 ms on both. **Crossover: a program needs ~2,080 lines (Windows) / ~270
(Linux) before compiling costs as much as starting.** Making the checker 2× faster saves 1.2% of a
Windows loop. Published: `measurements/agent-loop/RECORD.md`.

**Two hypotheses killed by controls, not argument:** embedded CPython = 1–3 ms (real, small, NOT the
floor — a binary with no DeluluLang in it costs 26.9 ms); the 512 MiB stack = 0.5–0.7 ms, below the
control's own spread, which is the **first evidence** for a `main.rs` docstring that has claimed
"costs nothing" since Study C.

**A CORRECTNESS DEFECT FOUND BY MEASURING.** Asking whether one process could do N files exposed
that `parse_opts` kept the first non-flag argument and **dropped the rest in silence**:
`delulu check a.delulu bad.delulu` printed `ok: a.delulu checked clean` and exited **0** while
`bad.delulu` — never opened — held two errors; a shell glob did the same. Observed against the
unmodified binary first. `check` now takes many files (20 files: 711→48 ms Windows **14.8×**,
119→14.2 ms Linux **8.4×**); `authority`/`run`/`why`/`build`/`lock`/3× `plugin` **refuse** a second
path. Single-file behaviour byte-identical — the core-invariance gate proved it one commit after
being built.

**The gate left behind is NOT a timing gate** (a wall-clock assertion on a shared machine is a
flake; this repo already had one). `crates/delulu-check/tests/work_scaling.rs` counts
**allocations** and asserts a **SHAPE, not a constant**: doubling the input may not more than double
the work. Healthy `records` 1.88 / `wide` 1.97; **C48 deliberately reintroduced → 3.89, observed
failing** before the gate was trusted; bound 2.6. A third test asserts the counter is counting.

**Named, not fixed:** a 400-term expression overflows a default 2 MiB thread stack in the checker —
C21's still-open residual for library embeddings; it is why `main.rs` uses 512 MiB.

**Disproved by inspection, do not rebuild:** `delulu graph` and `delulu stats` were on the roadmap
and already exist as `delulu atlas`. Jesse spotted this himself.

**THE LOAD-BEARING ADDITION — `tests/core-invariance/SNAPSHOT.txt` (commit `abf547a`).** Everything
in phases 1–3 sits *around* the language. Nothing proved it had not moved the language, and a green
suite does not: **the conformance law pins each diagnostic CODE, never the message, span, repair,
inferred row, or authority report.** The snapshot records the exact bytes for all **108 targets**
(every `.delulu` under `examples/`, `tests/conformance/`, `tests/corpus/`, plus the 7 package
directories) × 360 cases. Bless with `DELULU_BLESS=1 cargo test -p delulu --test core_invariance`.

**Witnessed, per house method:** changing ONE WORD of DL0106's message in
`delulu-syntax/src/parser.rs` left the entire pre-existing suite green and was caught by this gate
alone. Confirmed exhaustively — every existing assertion about that code is `x.code == "DL0106"`.

**Also run once, against a binary built from `8bfa894` (the commit before phase 1):** 657 CLI
comparisons over the same corpus → 498/498 core-language comparisons byte-identical, the only 13
differences being the `explain` codes `5fefef6` deliberately gave answers to; plus a runtime
differential (both engines, `build`, `lock`, each case run twice on the new binary first to separate
nondeterminism from regression) → 0 differences in 23 cases. Method is now a standing order:
[[delulu-core-regression-rule]].

**Cross-platform:** the snapshot recorded on **Windows passes byte-for-byte on Linux** (222,762
bytes, md5 `bdd56c6a6e40e791c394b4b4e9bc0b99`). Windows 109 suites / **1,437** / 0 failed / clippy
**65**; Linux 109 / **1,441** / 0 / clippy **66**; both coverage 100%, reference in sync, fmt
0-change, `doctor --check` 12/12; release and `--no-default-features` builds green. Every macOS
divergence mechanism was removed *and checked* (separators, CRLF, `read_dir` order, NFD/NFC
filenames, version churn, platform words) — recorded in `CROSS_PLATFORM_VERIFICATION.md` §6.
**macOS still never executed.**

**The scheduler was rebuilt this pass** — Jesse asked twice why it kept resuming after he
interrupted. See [[delulu-continuation-protocol]]: recurring cron → one-shot chain with a nonce.

## Production-readiness + architecture stabilization — 2026-08-02, Opus 5 (5 phases, autonomous)

Jesse: *"a production readiness and architecture stabilization phase, not feature chasing … Do not
assume previous conclusions are correct. Challenge them independently."* Plus: **full authority, no
approvals needed**; *"update all .md files also"*; verify Linux + Windows, macOS by evidence only.
Phases: **A** review+register · **B** C21/macOS/Survey notes/CODEOWNERS · **C** architecture ·
**D** docs · **E** release readiness + v1.0 verdict.

**PHASE A DONE — commit `87eca8e`. Windows 112 suites/1,465/0 clippy 65; Linux 112/1,469/0 clippy 66;
coverage 100%, reference in sync, fmt 0-change, doctor 12/12.** Deliverable:
`docs/design/PRODUCTION_READINESS_REVIEW.md` (the disposition register — 9 implement, 5 reject with
reasons, 4 accepted limits).

**🔴 C70/D67 — THE FINDING, and it was in the shipped product.** `ref.rule.runtime.faults-are-
diagnostics` names recursion depth, promises *"never a host crash"*, and the reference marked it
**covered**. It was FALSE inside an actor: `down(1000)` printed `1000` from `fn main` and killed the
process from a behavior — **`0xC00000FD` above depth 43 (debug) / ~350 (release) vs a documented
bound of 10,000.** No embedder, no hostile input, just `spawn`. Cause: `main.rs` reserves a big stack
so the guard fires first; that reservation belongs to ONE thread, and `actors.rs` — added two stages
later, running the same `Interp` — set **no stack size at all**.
- **Design rule 1's 7th instance**; **design rule 2 in a new costume: the right signal, ON THE WRONG
  THREAD** (the witness recurses in `main`, the one thread where the rule held); and **a stack
  overflow prints no `panicked at`** (D47a) so every no-panic sweep was blind.
- Fix: budget moved to `delulu-runtime` (one definition, `main.rs` imports it); workers reserve it
  with `max_depth_for_stack` sizing the bound to the stack they got — **the PAIR is the invariant**;
  source-scanning gate over every thread site (verified non-vacuous → names `actors.rs:471`).
- **Also closed a contradiction nobody had compared:** `main.rs` reserved 512 MiB while the published
  budget was 80 KiB × 10,000 = **800 MiB**. It now reserves what it publishes.
- **And the repair opened a hole its last clause closes** (3rd time this project has done that): a
  degraded reservation used to crash, then silently lowered the bound. `reduced_depth_bound()` now
  reports it — `isolation-labels-are-honest` by analogy.
- Verified: overflow inside an actor is already DL0901, so C70's scope is exactly the one fault class
  that depends on the HOST STACK. Other paths safe: WASM actors are cooperative single-threaded;
  plugin/foreign-worker run on `delulu-main`.

**⚠ C21 WAS MIS-FILED FOR ITS WHOLE LIFE** — as a hypothetical *library-embedding* residual. D51 built
exactly the right mechanism (`with_max_depth`) and **nothing in the tree called it**; the caller that
needed it most was DeluluLang's own actor runtime. **A contract with no caller is a contract nobody is
keeping** — check for callers when closing a finding with an API.

**FOUR of my own review findings dissolved under checking, and that is the method working:**
(1) "STABILITY.md says nothing about the Rust API" — it says *"Internal crate APIs"*; my grep used my
vocabulary, not the document's. Survived smaller: the promise exists with **no mechanism**.
(2) CI's `|| true` on coverage is not a hole — `release_requires_full_coverage` in the suite is
stronger. Only the comment is stale. (3) The `macos-latest` matrix is not an overclaim —
`CROSS_PLATFORM_VERIFICATION.md` already says it has never executed. (4) The
`c-token-not-a-campaign-finding` note is correct and deliberate; its only trigger `C99` lives in the
Survey's OWN comment explaining why C99 isn't a finding, and **C11/C17 ARE real findings** so any
allowlist would be wrong.

**🔑 PHASE C's REAL SHAPE (found early, changes the fix): `publish = false` already means TWO things
here.** `delulu-survey` reads it → `tooling_crates` → `crates_shipped = 13−1 = 12`, the number README
quotes and a test gates. But `STABILITY.md` §2 promises the Rust crates are *not a stable interface*,
and 12 of 13 crates are publishable at 1.0.0 with **no mechanism** behind that promise. The eleven
libraries need OPPOSITE answers to "is this the language product?" and "may this be published?" —
so naive `publish = false` would turn README's "12 crates" into "1". **Separate the two signals first.**

**CODEOWNERS disposition REVERSED** (Phase 5 had shelved it): rejecting the `owners` *verb* was right
(constant function — every rule names `@PENDING-PUBLIC-project-lead`), but that was wrongly applied to
the **entrenchment attribute** too. Eight paths marked project-lead-only is a citable fact and exactly
what an agent needs before editing. **Implement as a node attribute in Phase B.**

**Survey notes, judged:** the 3 `build-order-without-citable-rulings` are RIGHT — Stage 6/7/8 record
"Deviation *n*" but three stages each have a Deviation 3, so they genuinely aren't citable; fix is an
additive stage-qualified index (`S6-D1`…), not a checker change. `test-count-quoted` — README was
stale (1,460→1,465, corrected); the other 4 lines are **dated historical records**, correctly left.
`ruling-cited-without-its-stage` — working as intended, keep.

**macOS static audit done (no hardware, never executed):** every `cfg` site enumerated and
**exhaustive** — `unix`/`not(unix)` and `windows`/`not(windows)` pairs cover macOS; the two Linux-only
mechanisms are deliberate (`PR_SET_PDEATHSIG` with the macOS substitute named in a comment; microVM →
DL1408). **Named risk with arithmetic: the Unix socket path limit is 104 bytes on macOS vs 108 on
Linux**, and macOS temp dirs are long — ~14 bytes of margin on the longest test state dir. Phase B
turns that into a named diagnostic (testable on Linux).

**Method note that cost real time (4th+ occurrence): DO NOT edit files while a test battery runs** —
it restales the committed Survey mid-flight and produces phantom failures. Regenerate the Survey AFTER
the last doc edit, then run. Also: **PowerShell mangles `|` inside `wsl -e bash -lc "…"`** — write a
`.sh` to the scratchpad and run that instead.

**PHASE B DONE — commit `47e339e` (D68). Windows 113 suites/1,477/0 clippy 65; Linux 113/1,483/0
clippy 66; coverage 100%, reference in sync, fmt 0-change, doctor 12/12. Survey 960 nodes / 8,341
edges / 3 notes (WAS 6).** Four register items:
1. **CODEOWNERS entrenchment as a Survey NODE ATTRIBUTE** (`crates/delulu-survey/src/codeowners.rs`).
   `query` prints it BEFORE any edge, citing the CODEOWNERS line; owner carried verbatim. 19 nodes.
   **The `*` catch-all is deliberately IGNORED** (a rule matching everything separates nothing) and
   **a rule matching NO path is an ERROR** (renaming an entrenched file silently un-entrenches it) —
   the error branch exercised against a synthetic tree. **Near-miss worth remembering: a directory
   rule must cover the directory ITSELF**, else `/crates/delulu-conform/` marks 5 modules and leaves
   `crate:delulu-conform` — the node an agent names — unmarked.
2. **Stage 6/7/8 rulings made citable ADDITIVELY** — a "Ruling index" section per build order naming
   each existing Deviation/ledger entry as `S6-D1`…/`S7-D1`…/`S8-D1`…. Nothing renamed. Cleared the 3
   `build-order-without-citable-rulings` notes. **Checked the backfire: bare-`D<n>` ambiguity did NOT
   grow** (affected numbers identical, citations 244→243).
3. **C21's residual closed: `delulu_runtime::on_interpreter_thread`** runs a closure on a
   correctly-sized thread. It **reports** a refused reservation (an embedder must choose) where the
   actor scheduler **degrades** (it must keep running) — asymmetry deliberate.
4. **macOS socket path named**: `sun_path` 104 macOS / 108 Linux; checked before `bind`, message
   carries both figures + remedy. **Tested on Linux where the branch compiles; the macOS constant is
   reasoned** — the ruling states that distinction rather than blurring it.

**PHASE C DONE — commit `910e9f5` (D69). Windows 113 suites/1,478/0 clippy 65; Linux 113/1,484/0
clippy 66; Survey 962 nodes / 3 notes.** Architecture stabilization.
- **THE `publish` OVERLOAD SPLIT.** `STABILITY.md` §2 ("the Rust crates are an implementation
  detail") had **no mechanism** — 12 of 13 crates were publishable at 1.0.0. But `delulu-survey`
  read `publish = false` → `tooling_crates` → `crates_shipped`, so the naive fix would have turned
  README's "12 crates" into "1". Now **`[package.metadata.delulu] surface = "language"|"tooling"`**
  answers *is this the product* and `publish` answers *may this go to crates.io*; only the CLI
  publishes; gate `every_crate_declares_its_surface_and_only_the_cli_publishes` (verified
  non-vacuous). **The split exposed a count that was RIGHT BY COINCIDENCE: 12 was "13 minus the
  unpublishable ones" LABELLED "language crates" — true only while one crate was both. Truth is 9
  language + 4 tooling.** Reusable: *a count computed from a proxy is a measurement waiting to be
  wrong.*
- **⚠ I PUT A FALSE CLAIM IN A RULING AND CAUGHT IT BEFORE COMMIT.** First draft justified the CLI
  exception as "so `cargo install delulu` keeps working". **It does not work** — nothing is
  distributed and the CLI's path deps carry **no `version` fields**, so `cargo publish` refuses it
  outright. Corrected in the ruling, STABILITY.md, CHANGELOG and the gate's docstring. True reason:
  the CLI is the only crate that could ever BE a distributed artifact; **the gate prevents an
  ACCIDENT, it does not preserve an install path.**
- **`cmd_run` extracted to `run_cmd.rs`; cli.rs 8,856 → 7,740.** Coupling MEASURED FIRST: of 143
  top-level items in cli.rs the run subsystem reaches **24, sixteen of them its own helpers**.
  **Shared helpers (`mint_device_nodes`, `authority_spec_from_grants`, `grants_from_lease`)
  DELIBERATELY STAYED** — they have call sites in grants/guard, and moving shared code into one
  command's module is a **false ownership claim**. *A file-size target is not an architecture.*
  **core-invariance passed BYTE-IDENTICAL and was NOT re-blessed** — that is the whole proof a
  1,116-line move changed nothing.
- **Audited, left alone, reasons recorded:** all 9 extension-point traits have real implementors;
  **`PluginEngine`'s single implementor is dependency INVERSION** (in `delulu-runtime`, implemented
  in `delulu-wasm` which depends on runtime — removing it makes a cycle); broker→check for `Effect`
  alone is the "one list, both sides" pattern. Every generated artifact declares itself generated.
- **Remaining cli.rs clusters characterised, not cut:** broker-facing commands ≈1,050 lines,
  authority reporting ≈800.

**PHASE D DONE — commit `ed8a769` (D70, closing C71). Windows 113 suites/1,479/0 clippy 65; Linux
113/1,485/0 clippy 66; coverage 100%, reference in sync, fmt 0-change. Survey 964 nodes / 3 notes.**
- **AUDIT FIRST, AND THE HONEST ANSWER WAS "IT ALREADY EXISTS."** Every doc area Jesse listed (spec,
  grammar, semantics, effects, capabilities, ownership, diagnostics, package format, plugin
  architecture, Survey architecture) is already written. **Recorded the non-action** instead of
  padding the phase with prose.
- **C71/D70 — A PUBLISHED GRAMMAR ANCHOR LED NOWHERE.** `docs/reference/grammar.md` publishes
  `ref.grammar.<name>` per production (names from `parser.rs`, fenced by a drift guard); the
  normative EBNF in the stage specs uses fuller spellings and **6 of 27 diverge**.
  `ref.grammar.args` had BOTH witnesses and a `parse_args`, and **no spec defined `args`**. All six
  were covered under other names (`call`, `effect_row`, + 4 `_decl`/`_expr`) — **nothing was
  undocumented, only the PATH was broken.** Fix: `grammar::NORMATIVE_NAME` + a **Defined as** column
  + a 3-way gate (`every_grammar_production_is_defined_in_a_normative_specification`).
- **DELIBERATELY DID NOT author a consolidated EBNF.** The normative grammar is 149 production lines
  across 8 files. Re-deriving one by hand from a 2,000-line recursive-descent parser risks shipping a
  normative grammar that is WRONG — **C47's exact lesson.** Reasoned deferral, not a skip.
- Three stale statements corrected in place: STAGE2 (authority loader closed by D45a; "CI is
  Windows-only" → the workflow declares 3 OSes and has NEVER executed), STAGE6 deviation 3 (refusal
  stands, blocker gone since D61), ci.yml ("flip at the 1.0 cut" → it flipped, in the test suite).

**⚠⚠ TWO METHOD FAILURES THAT COST FOUR VERIFICATION ROUNDS — READ THESE BEFORE ANY NON-VACUITY TEST:**
1. **`Copy-Item` RESTORE gives the destination the BACKUP's mtime.** After breaking `NORMATIVE_NAME`
   to watch the gate fail, the restore predated the build that compiled the broken version → cargo
   **skipped the rebuild** → source correct, artifact stale. **Then `--reference` ran against it and
   wrote `| args | — |` INTO A COMMITTED FILE.** The corruption OUTLIVED the rebuild that fixed the
   cause, so the gate started passing while `the_generated_reference_is_not_stale` kept failing.
   **NEW RULE: a stale build that feeds a GENERATOR does not stay in the build directory.** Touch the
   file after a restore, or verify the artifact.
2. **I published THREE wrong diagnoses before the evidence supported any** (undetermined cause →
   wrong test named, from misreading my own `---- <test> stdout ----` filter → a race between the two
   platform runs, refuted by running serially). *A confident cause that has not been reproduced is a
   guess.*
3. **Edited files during a running battery for the 5TH time**, staling the Survey mid-flight (5
   `doctor_cli` failures on Linux). The rule was already in memory. **The fix is ordering, not
   memory: finish EVERY doc edit before verification starts.**

**Jesse also asked for a WATCHDOG** (2026-08-02): a one-shot cron every ~15 min that checks stalled
background tasks + stale `cargo`/`rustc`/`delulu` processes on Windows AND WSL, kills orphans, and
re-arms while phase work is live. **Reliable liveness signals: `pgrep -c cargo` and the growth of
`/tmp/dl-lin-b.txt` (one line per finished suite, ~113 total). NOT reliable: WSL `init` elapsed time
across separate `wsl -e` invocations, and a background task's `.output` file, which stays 0 bytes
until the script's summary prints at the end.**

**PHASE E DONE — commit `d433a38` (D71). 🏁 ALL FIVE PHASES CLOSED 2026-08-02.** Windows 113
suites/1,479/0 clippy 65; Linux 113/1,485/0 clippy 66; Survey 966 nodes / 8,434 edges / **3 notes,
0 error, 0 warning**.
- **THE CLEAN CLONE PROVES ITSELF.** `git clone` to a fresh dir with its own `target/`: 572 tracked
  files, Cargo.lock present, **cold build 1m21s**, full suite **113/1,485/0**, coverage 100%,
  reference in sync, fmt 0-change, doctor 12/12. Then the README front door replayed **VERBATIM**
  (binary invoked by absolute path — `cargo run -p delulu` CANNOT be used outside the workspace):
  `new hello` → `run . --grant console` → **`hello, world`**; hand-written `hello.delulu` →
  check/authority → **DL0703 without the grant** → **`Hello, Delulu`** with it. **The refusal is the
  load-bearing line** — the campaign's first finding was that the front door was FALSE; this is it
  true, from nothing.
- **Deliverable: `docs/release/CHECKPOINT-1.0.md`** — companion to CHECKLIST-1.0.md (which stays the
  GATE). Architecture / Survey / compiler / runtime / CLI / package ecosystem / testing / **10 known
  limitations** / roadmap. Deliberately explains why `broker→check` (one enum) and `PluginEngine`
  (single implementor = dependency inversion) are **correct rather than debt** — both look like
  defects and someone will "fix" them.
- **Two of my own scripts failed and both were MY bugs**: one mislabelled an under-granted run as
  "with the grant" (demo.delulu needs Read/Write/FsRead, not just console); one used
  `cargo run -p delulu` from outside the workspace.

**📊 THE PASS'S RESULT IN ONE LINE: one defect in shipped behaviour (C70), and several things that
were true only BY ACCIDENT** — a stability promise with no mechanism, a crate count derived from a
proxy (right by coincidence, wrong by 3 when asked directly), a grammar index that led nowhere. Each
small; each survived every previous pass because nothing had asked that particular question.
**Four of the pass's own findings dissolved under checking and are recorded as dissolved.**

## Post-pass hostile sweep of the AUTHORING + CUSTODY CLI — 2026-08-03, commit `930c614` (D72)

Jesse: *"fix everything… I want the world to use it"* + *"Is delulu authority, guard, atlas … survey
and doctor still working?"* **Windows 113 suites/1,486/0 clippy 65; Linux 113/1,492/0 clippy 66;
Survey 971 nodes / 3 notes.**

**THE GAP THAT MADE THIS PRODUCTIVE: P2–P11 attacked the LANGUAGE stage by stage. `new`, `fix`,
`secrets`, `audit` were written afterwards or never fired at — no hostile pass had ever touched
them.** One found four defects, all of them families this campaign had already ruled on. **A command
written after a lesson does not inherit it.**
- **C75/D72d (highest severity) — `delulu audit` silently ignored unknown options and READ THE WRONG
  STORE.** Parser's last arm was `_ => {}`. `audit` takes **`--dir`**; `grants`/`guard`/`secrets` all
  take **`--state-dir`**, so the habitual flag was dropped and records from the default
  `~/.delulu/audit` printed as the answer about a different store (witnessed: a real
  `expose deny target=DB_PASSWORD` line). **Evidence from somewhere else is worse than none, in the
  one subsystem whose purpose is accountability.** Now refused, with a note explaining audit reads
  FILES while its siblings talk to a BROKER.
- **C74/D72c — `secrets list` printed NOTHING on an empty store, exit 0.** `grants list` already said
  `(no grants — the tree is empty)`. C26/C66 shape on a SECURITY store. Message on stderr so stdout
  stays a pipeable list.
- **C73/D72b — `fix notes.txt` printed `nothing to repair`, exit 0**, while `check` gave DL0204.
  **C66 EXACTLY — and `fix` was written AFTER C66 was closed for `fmt`.** The refusal is deliberately
  NOT worded "nothing to repair" (the success line); my first draft reused it and the new test caught
  me.
- **C72/D72a — `delulu new con` SUCCEEDED on Linux/macOS**, producing a directory Windows can never
  `git clone`. On Windows it failed with TWO different raw OS errors for one cause (87 / 2). Reserved
  device names (`con aux nul prn com1-9 lpt1-9`) now refused on **EVERY** platform; `console`,
  `context`, `nullable` untouched.

**⚠ METHOD — THE SWEEP'S OWN BUGS OUTNUMBERED THE PRODUCT'S 6:1.** First run: 23 pass / 7 "fail".
**Six of the seven were MY script**: a relative `$D` path that broke after `cd`; wrong expected
strings for `atlas` (its version key is `atlas`, not `schema`); a `grep -qF ""` helper that cannot
match an empty expectation. **Check every failure before reporting it** — otherwise you hand the
owner phantom defects. Corrected sweep: **35 checks, 0 failures.**

**FEATURE HEALTH ANSWERED WITH EVIDENCE (35/35):** *writing* — check (incl. multi-file), authority
(human+json), run (DL0703 refusal AND success), atlas (graph/json/query verbs), why, explain, fmt,
test, new→check→run→build→lock→authority on a package dir, plugin verify. *authority machinery* —
grants **fails closed DL1401** with no broker, guard, audit, secrets. *maintaining* — survey
query/impact/entrenchment/path/findings, doctor 12/12 read-only + one JSON object.
**Reusable harness: `scratchpad/feature_health.sh`** (not committed — the four findings became real
Rust tests instead).

## C76/D73 — the silent-argument class, swept across the whole CLI — commit `0c98a58`, 2026-08-03

**Windows 113 suites/1,488/0 clippy 34; Linux 113/1,494/0 clippy 53 (was 66 — clippy `--fix` applied).
Survey 973 nodes / 3 notes.**

**D72d fixed `audit`; C76 is the CLASS.** Gave every subcommand a flag that cannot exist and counted
exit 0: **12 of 22 ignored it** — check, authority, why, atlas, explain, run, build, lock, test,
secrets, locale, morph. `delulu check app.delulu --strict` → `checked clean`, exit 0.
**Then the same defect one position over**: every value-taking arm was `if i + 1 < rest.len() { take }`
with **no `else`**, so a flag in final position vanished. 7 did it, sharpest:
**`run app --grant console --isolation` ran with NO ISOLATION, exit 0.**

**The twin already existed**: `refuse_extra_positionals` (Phase 4) with the docstring *"silently
dropping an argument … reports success about work it never did"* — built for PATHS; FLAGS were never
examined and the shared parser's last arm stayed `_ => {}`.
Fix: parser **collects** (`unknown_flags`, `missing_values`), **command decides**
(`refuse_unknown_flags` / `refuse_unlisted_flags` — locale/morph/secrets parse their own subverbs;
atlas/test parse their own argv), plus **two sweeps** over the whole surface.

**⚠ THE METHOD LESSON: my patching regex MISSED `"--out" | "-o"`** — the alias changed the shape and
the textual patch reported success. **The BEHAVIOURAL sweep caught it.** Test behaviour, not shape.

**HOW EVERYTHING THIS SESSION WAS FOUND: run every command with an argument that cannot be valid and
read the exit status.** The authoring + custody CLI had never had a hostile pass (P2–P11 walked the
LANGUAGE). **A command written after a lesson does not inherit it** — `fix` post-dated C66 and
repeated it; the flag class post-dated the positional fix and repeated that.

**FRESHNESS GATE NOW SELF-DIAGNOSES.** It had misreported "tree moved mid-run" as "map is stale"
**seven times in one session**, sending the reader to regenerate and commit a half-finished edit. Now
names BOTH causes + prints the newest input file and its age; caught its author's own edit at
"24 SECONDS ago" on first firing. **A gate that reports the wrong cause is barely better than none.**

**BOOK FULLY UPDATED** (Ch.11 actor depth bound + that it was false until this week; **Ch.16 gains
the plainest missing fact — NO PHYSICAL DEVICE HAS EVER BEEN COMMANDED**; Ch.19 gains three-platform
+ hardware refusals; **Ch.20 was stale in the UNDERCLAIMING direction** — called shipped Stage-10
work "not shipped software" — now history + a real remaining list; Appendix C → CHECKPOINT-1.0).

**CLIPPY — the measurement was wrong TWO ways, and the second half was only found on 2026-08-03
(commit `f39f763`). These are the authoritative numbers; earlier figures in this file were superseded.**
It is Rust's linter and **no `clippy::correctness` lint was ever among them**. The counter
`grep -cE '^warning:|^error:'` (a) also matched cargo's per-crate **summary** lines
(``warning: `delulu-wasm` (lib) generated 1 warning``, 13–18 of them) — this half I caught earlier — and
(b) **a WARM `cargo clippy` does not re-emit warnings for units it did not re-lint**, so the same tree
measured **26 and then 42 within the hour**. That second half is why my own records disagreed (this
memory said 34, the CHANGELOG said 26). **Only cold, isolated-target-dir, summary-excluded counts
mean anything.** Measured that way at `0c98a58` and after: **Windows 34 → 14, Linux 35 → 15.** Both
fell by exactly 20 and the +1 Linux surplus held — the old measure was *consistently* wrong, not
randomly wrong. The reduction = 12 `let mut X::default()` sites → initializer form, **all test-only**,
`mut` kept where `.insert()`/`.push()` follows. **Baseline is PER-PLATFORM; the macOS number has never
been seen.** Dated historical figures LEFT AS RECORDED with an annotation — rewriting them would hide
the mistake. **Design rule 2 again: the gate keyed on a line prefix, and two different things produce it.**

**STANDING RULE (Jesse):** keep a scheduled wakeup armed whenever shell commands run in the
background, to catch never-ending loops; re-arm ~15 min while anything is in flight.

## 2026-08-03 (commit `f39f763`) — the platform delta NAMED, and the CLI/compiler driven by hand

**Both platforms green on the committed tree: Windows 113 suites / 1,488 / 0 / 4 ignored; Linux
113 / 1,494 / 0 / 4.** Coverage 100%, reference in sync (24 ch), fmt 0 would change, doctor 12/12,
Survey 973 nodes / 8,537 edges / 0 error / 0 warning.

**THE 6-TEST PLATFORM DELTA IS NAMED, NOT ESTIMATED — don't re-panic at 1488 vs 1494.** Docs said
"four tests Windows skips": stale (it is six) *and wrong in kind*. Re-derived the only way that
settles it — `cargo test --workspace -- --list` on each platform, sorted `LC_ALL=C`, diffed:
**8 Linux-only** (2 unix-socket `broker_transport`, 6 live wasmtime engine) − **2 Windows-only**
(`windows_refuses_contained_execution_rather_than_risk_a_fastfail`,
`a_verified_plugin_on_wasm_inherits_the_windows_enforcement_refusal`). **Windows does NOT silently
skip contained WASM execution — it compiles its own witnesses that the refusal happens.** A decision
with tests on both sides, not a hole on one. (`interp::on_interpreter_thread`'s doctest appears in
both listings under `/` vs `\` and is not part of the delta.) Windows lists 1,492 and Linux 1,498;
minus 4 ignored each = the passing counts.

**CLI + COMPILER driven by hand on BOTH OSes — `scratchpad/cli_compiler.sh`, 21 cases asserting EXIT
STATUS. 21/21 Windows, 21/21 Linux, identical case for case.** Covers check accepting/rejecting
(parse, type, undeclared effect), `run` refused **DL0703** without the grant, unknown subcommand /
unknown flag / **flag missing its value**, `new` refusing reserved device name `con` **on Linux too**,
`doctor --check`, and `--json` still exiting 1 while emitting one object. **Exit codes are the stable
contract (`STABILITY.md`): 0 ok / 1 diagnostics / 2 usage** — my sweep demanded 1 for usage errors and
raised 6 false alarms; also treated a 2nd positional to `check` as extra when **`check` takes many
files by design**. Sweep bugs before product bugs, again.

**macOS `cfg` fallthrough CHECKED and safe:** `#[cfg(not(target_os = "linux"))] microvm_unavailable`
returns an honest refusal naming `std::env::consts::OS` — never a silent downgrade to weaker isolation.

⚠ **METHOD, three costs paid this session:**
1. **Editing docs during a running battery broke a run for the 8th time** — the Survey went stale and
   `the_committed_map_matches_the_tree` failed at 92 suites. Finish ALL edits, regenerate the Survey,
   *then* start the battery and touch nothing.
2. **Do NOT detach WSL work with `nohup setsid`** — WSL tears down the process group when the
   launching session exits; the job died instantly and left a **0-byte log**, which looks exactly like
   a slow build. Use `wsl -e bash <script>` with `run_in_background` (that method has always worked).
3. **`wsl` paths need `MSYS_NO_PATHCONV=1`** from Git Bash, or the path gets a `C:/Program Files/Git`
   prefix. And a Linux build in `/mnt/d` dies in `libffi-sys` (DrvFs) — always set
   `CARGO_TARGET_DIR=$HOME/...`. Both already documented in-repo; I hit them anyway by not reading first.

## 2026-08-03 second pass (commit `ddbb14f`) — machine channel, and NO DISCRIMINATION

**Both platforms green: Windows 114 suites / 1,496 / 0; Linux 114 / 1,502 / 0. Cold clippy 14 on
BOTH** — the Linux-only `let job = ();` placeholder in `foreign_worker.rs` (the non-Windows arm of a
Windows-only Job Object) was the *entire* historical +1 delta going back to the old 65/66. CLI +
compiler sweep 21/21 on both. Rulings **D74–D77**, findings **C77–C81**.

**🔑 JESSE'S RULING — NO DISCRIMINATION BETWEEN MAINTAINERS.** *"There shall no descrimination for
human and ai and other maintainers of delululang. Everyone is welcome to maintain and develop
delululang further and beyond."* Then refined: *"I don't think too much of human approval is needed
in the future. Make it preferably human sponsor and accountability is good, but otherwise also
allowed. I am expecting ai doing everything by itself in the future."*
**C80:** the governance docs branched on AUTHOR KIND in five places while Constitution **invariant 24**
rejects kind-of-party trust hierarchies — and the policy was restated **by hand in SIX documents,
including `CONSTITUTION.md` §9 itself**, so the constitution contradicted its own invariant.
Every rule now keys on the **change**: named sponsor **who is not the author** for every risk-class
change; no merge on the author's own say-so; **all** contributed code sandboxed; every change names
its author. **Human sponsor = PREFERENCE with its reason stated, never a gate; an AI may hold every
role.** `sponsor ≠ author` KEPT — it is independence, not kind. The surviving line is **a named party
who answers** vs **unattended automation**, not human vs machine. `CONSTITUTION.md` IS entrenched →
entrenchment analysis (invariant 44) recorded; dated STAGE9 docs annotated **SUPERSEDED**, not
rewritten. Gate `no_governance_document_makes_a_rule_depend_on_the_kind_of_party` reads all four
current docs; proven non-vacuous.
⚠ **I published a wrong claim mid-way** — "CONSTITUTION.md did not need to change" — because I checked
invariant 24, found it supportive, and stopped. Corrected in place. **Read the whole document before
concluding it agrees with you.**

**C77/C78 — `doctor --json` emitted TWO objects on any run reporting a problem** (printed its envelope
without `note_json_emitted()`, so the nonzero exit collected the C2 fallback). It survived because the
completeness gate checked *list ⊆ help* and never *dispatcher ⊆ list*, so `doctor` — dispatched,
documented, in neither list — was swept by nothing. Gate now READS the `match cmd.as_str()` block.
`doctor` is NOT_SWEPT deliberately: it **writes** `docs/survey/` without `--check`.

**C79 — `delulu-survey` accepted `--json` and IGNORED it** (C76/D73's ruling surviving in a sibling
binary written before the lesson). Now every read-only verb emits one object; **every edge and hop
keeps `via:{kind,file,line}`** (the provenance law must survive into the machine channel or an agent
can check the map *less* than a human can); **`entrenched` always present, `null` when ordinary**; the
JSON walk is **uncapped**. Suite `crates/delulu-survey/tests/machine_channel.rs`.
**FRAMING JESSE CORRECTED ME ON: a structured channel is NOT an AI feature.** A person writing CI
needs `--json`; an agent chasing a bad edge reads the prose. Both channels are first-class **because
any maintainer may use either**. Never justify tooling with "the maintainers will be AI".

**C81 — a LINE BREAK defeated the gate built to catch stale counts.** `HARDENING_CAMPAIGN` promised
*"a stale figure here now fails a test"* while reading ~93,500 lines / 12 shipped + 1 tooling / 193
files against a tree of **102,581 / 9 shipped + 4 tooling / 213**. The `stale-count` check matched
number-then-unit **on one line**, and `193` ended a line while `files.` began the next; `~93,500`
escaped separately inside the **10% approximate slack**. My first write-up said "no test gated this
line" — **wrong**, corrected in place. Gate now reads across the wrap AND honours an explicitly
ISO-dated historical quote (the compliance its own advice demanded and never offered).
**Never restate a measured number in prose — cite where it is measured.**

⚠ **THE STALE-MAP VARIANT, cost a whole Linux run:** it is not only "don't edit DURING a battery" —
it is **regenerate the Survey immediately BEFORE every battery**. Editing *between* batteries left the
map stale and failed 3 doctor/survey tests that looked like a code regression. **And never detach WSL
work with `nohup setsid`** — WSL tears down the process group; the job died instantly leaving a 0-byte
log that looks exactly like a slow build. Use `wsl -e bash <script>` with `run_in_background`.

## 2026-08-03 third pass (commit `005025e`) — DEPLOYABLE: the toolchain can be handed to someone

Jesse: *"Make it deployable"* / *"DeluluLang should be a production ready and deployable lang of the
future."* **Windows 115 suites / 1,500 / 0; Linux 115 / 1,506 / 0** (delta still 6).

**`scripts/package-toolchain.sh`** stages binary + LICENSE/NOTICE/TRADEMARK/README/CHANGELOG/SECURITY
+ examples (files AND package dirs) + `SHA256SUMS` computed from the staged tree. **Verified by
unpacking into a directory sharing nothing with the workspace, no cargo, no source: 8/8 front-door
steps and 0 problems on BOTH platforms**, incl. checksums, the DL0703 refusal naming the exact flag,
and a package DIRECTORY running. Script: `scratchpad/verify_dist.sh`.

**🔑 THE SHIPPED BINARY MUST BE `--no-default-features`.** A default release build embeds CPython and
imports **`python313.dll` SPECIFICALLY** — not "Python 3", *that* build; without it the user gets a
loader error before `main`. **Measured, not assumed** (default binary carries the import string, the
portable one carries none). Shipped build: `root.python(...)` → **DL1307**, reported not crashed.

**`cargo install delulu` from crates.io CANNOT work and must stay that way** — `delulu` depends on
nine siblings by path, all `publish = false`, because **STABILITY.md §2** promises the Rust crates are
not a stable interface. Publishing them to shorten one command trades a written promise for
convenience. `INSTALL.md` states all three paths incl. this non-path. **README no longer says "there
is no download" — it says nothing is HOSTED**, which is the true statement (no registry, no download
page anywhere).

Gate **`crates/delulu/tests/distribution.rs`** (4 tests, non-vacuous): `INSTALL.md` and the archive's
generated `INSTALL.txt` must teach the SAME commands, both must show the refusal, the script must keep
the portable flag + licence files, no doc may promise crates.io, exactly one crate is publishable.

⚠ **Three defects found by DOING it, none in the compiler:**
1. The first `INSTALL.txt` said `--grant console` runs `demo.delulu`. It needs **three** grants — it
   printed, then correctly refused the fs read. **The archive would have taught its recipient
   something false in their first minute.** Now walks `hello_wasm.delulu` (refuses cleanly, succeeds
   with the one grant it names), and the verifier runs *exactly the documented steps* so they cannot
   drift. **Verify the instructions you ship, by executing them.**
2. The script ignored `CARGO_TARGET_DIR` — broke on precisely the Linux config the docs REQUIRE.
3. The Survey mapped `dist/` → 11 duplicate-file warnings that were packaging working correctly.
   **`dist` joined `target` in `EXCLUDED_DIRS`.**

## The first finding, recorded here because it sets the tone

`README.md` — the front door of a **v1.0.0-tagged** tree with Stage 10 closed and 23 rulings — said
*"Stage 1 ('Skeleton') — under construction"*, and told the reader to `rustup default stable`
against a `rust-toolchain.toml` pinned to 1.96.1. The project's honesty discipline was aimed
inward at specs and outward at claims, and **skipped the one page every new user reads first.**
That is the campaign's thesis in one artifact: correctness in the core does not survive contact
with users if the front door is false.

## P16 — the adversarial pass, 2026-08-03 (commits 19b621b, 4e1e825; findings C82–C92, rulings D78–D88)

Jesse re-fired the NASA commission and added a list of direct questions (can agents bypass
authority, vs sandbox, unbreakable?, real maths?, multi-agent on one host, plugins, can characters
change, spacing, why "lang of the future"). He asked mid-run for **subagents**: two Opus 5 (authority,
guard) + one Sonnet 5 (multi-tenancy). All three delivered. The Sonnet one was killed by Jesse's
session limit and was **resumed via SendMessage, not respawned** ([[subagent-cost-strategy]] — worked
exactly as that memory says).

**THE BIG ONE — C88/D78: the effect row was escapable.** A callback reaching a higher-order builtin
through a **bare type parameter** had its row silently dropped (`if let` with no `else`), and a
dropped row is an empty row. Nine lines: `check` clean, `authority` = "(none — provably pure)",
`why Write` = "cannot perform Write", and it **printed at run time**. The `Secret.map` twin printed a
**live secret** with no Declassify — so R-2 reopened alongside R-4. Fixed: the unknown case now
REFUSES (DL0401). **Zero false positives** across the whole corpus. `SOUNDNESS_AUDIT.md` now carries a
box saying its §D claim was false in practice, and that this is the argument for why the promised
mechanized "Delulu Core" proof is not optional.

**Lesson worth more than the fix (generalizes):** *a rule can be correct while the branch that fires
when the checker CANNOT TELL is missing — an audit of the rules cannot find that.* Both of R-4's
stated enforcement clauses were satisfied; they constrain the primitive table and a laundering
corpus, and the defect was in neither. This is [[skip-branch-verification-rule]] recurring, and the
capstone's 18-item + 14-surface checklists all passed while the central claim was false.

Also closed, each with an observed witness (and a CONTROL where a fix could have refused too much):
C84 symlink/junction escaped fs scope for read AND write, while STAGE3 §4.3 stated symlink
resolution as normative law (two doors: per-op check + minting a cap AT the link; one shared
fail-closed `prim::contains_on_disk`, used by interpreter and WASM host). C85 `*.example.com`
matched `evilexample.com` (project's own python matcher already had the dot boundary). C86 URL
userinfo → wrong host authorized AND wrongly audited (two copies of the parse, now one). C87
`--grant fs.read=` granted the whole CWD. C82 `morph render` rewrote identifiers into keywords,
exit 0 (the morph aimed at **AI** was the unsafe one; human-language morphs are safe by
construction since identifiers are ASCII-only). C83 first spanned morph diagnostic crashed the CLI.
C89 six chars render as a line break and don't act as one → a comment swallows the next VISIBLE line
(Trojan Source **inverted**; DL0108). C90 `--accept-widening` filed its widening as "changes
nothing". C91 STABILITY.md omitted exit code 3.

**NAMED, NOT FIXED (deliberate, in the record):** exponential type inference (28 lines exhaust
memory), quadratic checking in nesting depth, unbounded parser recursion crashing outside the
exit-code contract — `delulu check` is the agent hot loop, so these are DoS surfaces against the
intended workflow; a bound is language-visible and belongs in an RFC. And **C92: multi-tenancy is
NOT provided.** Embedded mode isolates concurrent programs with DeluluLang's own check (verified
with identical ACLs — the OS was doing nothing); the broker daemon does not, and its source already
said so (`broker_transport.rs:5`). Several agents with different authority need **separate OS
accounts or containers**; separate directories under one account is NOT sufficient.

**New deliverable: `docs/QUESTIONS.md`** — answers all of Jesse's questions with evidence, leads
with the C88 failure rather than burying it, and ends with the twelve things the project cannot
claim. Linked from README. Refuses the word "unbreakable" (constitution forbids it).

**Verified:** Windows 115 suites/1512 passed/0 failed; Linux 115/1518/0; clippy **14** both (baseline
unchanged); CLI+compiler sweep 21/21 both; coverage 100%; reference in sync; fmt clean; doctor 12/12.
The C84 fix was attacked with **each platform's own link type** (Windows junction, Linux symlink),
each with a control. **macOS STILL NEVER EXECUTED.** Recorded honestly: macOS shares POSIX symlink
semantics with Linux and case-insensitivity with Windows, so each half ran on a platform that shares
it — but no platform ran both halves in macOS's combination. That is an argument, filed as one.

**Trap avoided worth remembering:** `canonicalize` returns real on-disk casing, so on
case-insensitive filesystems (Windows AND macOS) *replacing* the lexical check would have WIDENED
scope (`./data` grant matching a `./DATA` mint). The fix **ANDs** onto the lexical test, so it can
only narrow. Verified still DL0703.

**Method note:** `cargo fmt --all --check` is NOT a gate in this repo — it reports ~3,795 diffs
across untouched files because the project has its own house style. The real fmt gate is
`delulu fmt --check examples`. Do not run `cargo fmt` here.

## Standing constraints (unchanged, absolute)

**Push only to the testing remote** ([[delulu-github-remote]]; public since 2026-09-17; the rule was
NEVER push to GitHub until 2026-09-14). Never hand-roll crypto; the assistant cannot sign artifacts. The word
"[the banned word]" never appears in any repo surface ([[no-banned-word-mentions]]). Commit trailer names the
actually-running model ([[model-attribution-honesty]]). Coverage 100% is a live per-commit gate.
"quantum-proof" only inside prohibition sentences. Jesse runs long overnight sessions and may not
answer — decide when the direction is clear, stop only for vision-level changes.

Related: [[delululang-project]], [[delulu-federation-scope]], [[delulu-hw-adapter]],
[[rfc-process-deviation-d21]], [[subagent-cost-strategy]].
