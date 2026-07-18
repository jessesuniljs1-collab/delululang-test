# Stage 8 Build Order — "Surface" (operational companion)

**Status:** IN PROGRESS (started 2026-07-18). Normative spec: `STAGE8_SPECIFICATION.md`;
how-to: `docs/playbooks/STAGE8_PLAYBOOK.md`. Precedence: spec > playbook > this file — but
*this file's* deviations ledger (§3) records every ruled departure, and its close-out table
(§4) is the ground truth of DONE vs PENDING.

Prior state this stage builds on: Stages 1–7 BUILT (Stage 7 at `6b511be`; Windows 726/0,
Linux 730/0/2, TSAN zero warnings). The Atlas and the Palette — Stage-8 "Surface" material —
were **early-dropped and BUILT 2026-07-15** under `SURFACE_ATLAS_PALETTE_ADDENDUM.md` (its own
§8 close-out, all 11 criteria witnessed). Stage 8 proper does not re-litigate them; §4 below
covers the *remaining* Surface scope: spec §9 criteria 1–10.

## 0. Scope and bookkeeping

- Phases 8a–8h per the playbook; each phase = build green → test green → spec status log →
  commit. The language core is **feature-frozen** (invariant 38): nothing in this stage may
  change what compiles, what rows mean, or what runs.
- **DL range:** DL1701–DL1706 to register per spec §10. DL1780/DL1781 (Atlas) and DL1790
  (Palette) are already live from the addendum. **DL1784 is deliberately never allocated**
  (house rule mirroring DL1404/DL1609; meta-tested in `codes.rs`).
- Where the work lands (playbook §1): `delulu-syntax` (test blocks), `delulu-check`
  (assert/assert_eq + test-body typing), `delulu-diag` (the catalog layer — the stage's
  center of gravity; the seam is `render.rs::render_human*`, which runs AFTER the machine
  envelope exists), `delulu` CLI (fmt/test/locale/keygen/sign/publish/add + picker/welcome),
  LSP server (crate ruling pending — §3), `editors/vscode/` + `docs/editors.md`.

## 1. Gates

- **G1 (after 8a):** the grammar addition is invisible to non-test builds — every prior
  suite green UNMODIFIED; a `test` item never reaches codegen/interp in a normal build.
- **G2 (after 8b+8c):** locale invariance is STRUCTURAL — the full conformance `--json`
  output byte-identical under `en-US` and `delulu-slang`; the five welcome-suppression
  channels (`--json`, `CI`, non-TTY, `DELULU_NO_FIRST_RUN`, second-run) each witnessed
  INDEPENDENTLY; the welcome text hash-pinned.
- **G3 (after 8d):** fmt identity + idempotence hold over the conformance + fuzz corpora
  (≥100k programs); `--migrate 0.7` still passes the Stage-7 migration corpus.
- **G4 (after 8e):** LSP push diagnostics ≡ `delulu check --json` (same codes/spans/repairs)
  on the reference workspace; the widening code action is ⚠-tagged + `isPreferred: false`;
  latency ≤150 ms edit-to-diagnostics on the 10-kLoC reference, measured.
- **G5 (after 8f+8g):** a third-locale catalog plugin loads via `delulu locale add`
  (zero-authority verified-class — the Stage-6 dogfood); `delulu test` isolation runs under
  a REAL Stage-5 broker child node with session-end transitive revocation witnessed.
- **G6 (close-out):** all 10 spec §9 criteria carry named witnesses in §4; forbidden-word
  scrub clean; full suites green Windows + WSL Linux; docs moved with code; memory updated.

## 2. House rules (carried + Stage-8 specifics)

1. **NEVER push to GitHub.** Commit locally, per phase, at green.
2. The word that must never appear in the repo or any product surface does not appear.
   Scrub (`grep -ri`) before close-out.
3. **THE KITCHEN RULE:** for every rule, write the "what if the checker couldn't tell" case
   as a named test BEFORE claiming the rule holds. Stage-8 skip branches to hunt: catalog
   load failure must fall back WITHOUT touching the machine envelope; each welcome
   suppression channel tested alone (not just together); fmt on unparseable input refuses
   (never "formats" garbage); a missing en-US key fails the BUILD, not the render;
   signature-verify failure paths; test-authority check when the manifest has no
   `[test-authority]` table (default = pure, not default = allow).
4. **Invariant 38 — no semantics in tooling.** Any Stage-8 diff that changes check/run
   behavior of an existing program is a bug by definition. The prior suites are the fence.
5. **Invariant 39 — render prose LAST.** The machine envelope (codes/spans/repairs/JSON) is
   frozen before any catalog lookup. Build the ordering, don't test-and-hope it.
6. **Invariant 40 — agents run cold.** Zero interactive surprise under `--json`/`CI`/
   non-TTY/`DELULU_NO_FIRST_RUN`, each independently.
7. **Invariant 41 — tests hold no ambient authority.** No "test mode" relaxation, ever.
8. **The welcome text is LAW** — byte-exact per spec §6.3 including emoji and the
   attribution line, hash-pinned in the test, never translated/paraphrased/overridden
   (override attempt = DL1704), never in any machine output/log/trace/LSP/CI stream.
9. Docs move with code: spec status log per phase, this table at close-out, user guide with
   spec §11 caveats VERBATIM (meta-tested), `delulu explain` topic(s).
10. New dependencies need a ruling in §3. Dependency austerity is the default.
11. **Owner's order (2026-07-18): the head chef cooks EVERYTHING in Stage 8 — no
    sous-chef agents.** Commit messages end
    `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.

## 3. Deviations ledger (implementing chef appends; head chef rules)

1. **RULED — the Atlas + Palette are prior art, not Stage-8 work items.** Built and
   closed out under the addendum (2026-07-15). Stage 8's close-out references the addendum
   §8 instead of re-witnessing; the Palette IS the color layer Stage-8 surfaces use.
2. **RULED — `delulu fmt` extends the existing command.** Stage 7 shipped `cmd_fmt` as
   migration-only (Stage-7 deviation 6). 8d grows the same verb into the full canonical
   formatter; `--migrate` remains a mode of it; the Stage-7 migration corpus is a
   regression fence (criterion 4 names it).
3. *(open — rule at 8e)* LSP packaging: new crate `delulu-lsp` vs CLI module, and the
   protocol dependency question (hand-rolled JSON-RPC vs `lsp-server`/`lsp-types`).
   Dependency austerity vs correctness-of-types tradeoff — argue it, then rule it here.
4. *(open — rule at 8h)* Registry index transport for v0.8: the format is normative HTTP,
   but hosted ops are Stage 9 — the v0.8 client is expected to run against LOCAL index
   fixtures (dir/file), with the HTTP shape frozen. No network dependency without a ruling.
5. **RULED — assertion failure = DL1707.** Spec §2 says assertion failure is "panic with a
   structured payload" but §10's table names only tooling codes. A panic needs a code; the
   Stage-1 DL09xx runtime family is frozen bookkeeping, so the Stage-8 construct faults
   under a Stage-8 code: DL1707 registered with an explain body. `assert_eq`'s message is
   symmetric (`` `a` != `b` `` — no expected/actual guess at the value level); the 8g
   runner's JSON maps the pair into the spec's `{expected, actual}` payload.
6. **RULED — tests in plugin artifacts: strip at build, refuse at load.** `dir::serialize`
   strips test items and re-derives the stored truth from the stripped module (verify
   replays the check on exactly what the artifact carries); `validate_item` refuses a DIR
   that still carries a `Test` item (hand-crafted/hostile), mirroring the Actor precedent —
   never silently dropped at load.
7. **RULED — en-US lives in the code; the build-refusal is literal.** The spec's "en-US
   catalog, complete by construction, compiler refuses to build with a missing key" is
   implemented as: every diagnostic is BORN with its en-US message at its site, and every
   named CLI string's declaration (`catalog::CLI_STRINGS`) carries its en-US text — a key
   cannot exist without its en-US prose, so the refusal is a compile error, strictly
   stronger than any file-completeness check. Catalog FILES exist for the other voices.
8. **RULED — the machine envelope is fully locale-invariant, message text included.**
   `--json` (and every machine channel) renders the in-code en-US prose always; catalogs
   apply only in `render_human_localized`. Criterion 7 asks byte-identical "(machine
   fields)"; we ship byte-identical FULL JSON — strictly stronger, and structural: the
   envelope API takes no catalog parameter.
9. **RULED — catalogs are parsed by a hardened, bounded, zero-dependency TOML-subset
   reader** (the Palette's `theme.toml` precedent): `[section]` + `key = "basic string"`
   with standard escapes — the entire schema the format needs. Every defect (unknown key,
   unknown placeholder, malformed line, non-string entry value) is a DL1704 WARNING and
   that entry falls back to en-US: prose never takes the compiler down, and a catalog
   written for a newer compiler degrades instead of exploding. A template needing an arg
   the diagnostic doesn't carry falls back WHOLLY — a literal `{fn}` must never print.
10. **RULED — first-run mechanics (8c).** (a) Suppression envs read as PRESENCE:
    `CI` set (any value) and `DELULU_NO_FIRST_RUN` set (not only `=1`) each suppress —
    stricter than the spec's letter, in the agent-safe direction (invariant 40 fails
    open-to-suppression, never open-to-prompting). (b) Interactive = stdout AND stdin
    are terminals (the picker prompts on one and reads the other); the hidden
    `DELULU_ASSUME_TTY` override (the `DELULU_THEME_FILE` precedent) exists so the
    criterion-6 witnesses can drive the SHOWN cases through the real binary headless.
    (c) The picker runs only when nothing above it in the priority chain pinned a locale;
    the welcome shows on the first interactive run regardless, then `welcomed = true`.
    (d) Only the PICKER writes `locale` into config — `--locale`/env are per-invocation.
    (e) An unknown locale name warns (DL1704) and falls back to en-US, mirroring DL1790.
    (f) The suite asserts en-US human prose; the kitchen runs with no `DELULU_LOCALE`
    and no config locale set (recorded, not fenced, in v0.8).

## 4. Close-out table (spec §9 criteria → witnesses)

| # | Criterion | Witness | Status |
|---|---|---|---|
| 1 | LSP smoke: diagnostics ≡ check --json; hover signature+row; DL0501 code action applies → green; cross-package rename | — | PENDING |
| 2 | Authority lens: lambda inlay hints; ⚠ widening action distinguishable (`authority_widening` in `data`) | — | PENDING |
| 3 | ≤150 ms edit-to-diagnostics, 10-kLoC reference, measured | — | PENDING |
| 4 | fmt identity + idempotence ≥100k programs; `--check` exit 1; `--migrate 0.7` passes Stage-7 corpus | — | PENDING |
| 5 | test runner: pure tests zero grants; undeclared effect fails DL0501/DL0701; `effects_traced` in JSON; session-end revocation in `grants tree` | — | PENDING |
| 6 | welcome/picker: once on fresh TTY; never on the five suppressed channels; text hash-pinned | — | PENDING |
| 7 | locale invariance: conformance JSON byte-identical en-US vs delulu-slang; slang DL0501 matches catalog | — | PENDING |
| 8 | catalog plugin: third locale via `locale add`; en-US fallback on missing keys; welcome override → DL1704 | — | PENDING |
| 9 | `publish --dry-run` catches widening minor bump (DL1003) vs local index fixture; `add` renders authority summary from the index line alone | — | PENDING |
| 10 | all prior suites green; `fmt --check` + locale-invariance in CI permanently | — | PENDING |

## 5. Post-v0.8 RFC ledger (deferred with eyes open)

- Hosted registry operations, governance, publish upload endpoints — Stage 9 by design.
- Catalog translations beyond en-US + delulu-slang — community/AI plugins (Constitution §8.5).
- `delulu morph` (syntax skins, AI token-minimizing profiles) — `SYNTAX_MORPH_SPEC.md` is
  written; building it is not in the spec §9 gate. Rule it in only if the owner orders it.
- Editor plugins beyond the in-repo VS Code skeleton — community via LSP.
