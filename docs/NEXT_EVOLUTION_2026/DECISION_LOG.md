# Decision log — Next Evolution 2026

Each record: **evidence** (what was observed or read), **assumptions** (what is taken as true
without being verified here), **alternatives** (what else was considered), **why** (the reason the
direction was chosen), **confidence** (high / medium / low, and what would change it), and
**needs verification** (what a later phase must still establish). Records are append-only; a
reversed decision gets a new record that names the old one.

Status key: **PROPOSED** (needs the owner), **TAKEN** (within the head chef's remit, reversible),
**RULED** (owner decided).

---

## D-NE-1 — The plan optimizes for reach, not for more verification depth — TAKEN
- **Evidence:** 202 commits since v1.0.0, nearly all review/proof/docs/CI; the binary refuses to
  load a plugin from a program (NE-01); an agent's first ordinary program hits NE-02/NE-03/NE-10;
  nothing is installable. The field's convergence (`RESEARCH.md` §0).
- **Assumptions:** the owner's stated goal (AI agents and humans using the language easily) is the
  metric; the trust already earned does not need to be re-earned before reach work starts.
- **Alternatives:** (a) continue verification-first (mechanize the core, model federation);
  (b) big-bang feature work; (c) this: contract truth → flagship → stdlib → sockets → distribution,
  with the cheap verification items kept as a phase.
- **Why:** the guarantee holds and has been attacked repeatedly; what is missing is reach. Doing
  (a) first would add depth nobody can use; (b) would break the evidence discipline.
- **Confidence:** high on the order P1→P2; medium on P3 vs P4 ordering.
- **Needs verification:** none for the decision itself; each phase has its own gates.

## D-NE-2 — No subagents were used for this plan — TAKEN
- **Evidence:** owner's cost rule (agents only for real independent value); the head-chef-handoff
  ruling of 2026-07-19 (the head chef cooks directly); every finding here needed the real binary in
  a single consistent environment.
- **Alternatives:** a research agent for the web pages; a red-team agent for the binary.
- **Why:** the research was 40 page fetches read by the head chef with a consistent rubric; a
  second reader would have doubled cost without an independent check on the binary, which is where
  value lies. Future phases may use one sous-chef per well-bounded dish (P3's stdlib rows, P7's
  fuzz targets) if the owner wants; each would need a cold-start brief.
- **Confidence:** high.

## D-NE-3 — Snapshot regeneration is a reviewed act, per phase — PROPOSED
- **Evidence:** P1-04 (one `DL0210` instead of 73) and P1-08 (a grant line in the human report)
  change bytes the core-invariance snapshot pins; the owner's core-regression rule says regenerate
  deliberately, never incidentally.
- **Alternatives:** (a) avoid any change that moves the snapshot; (b) regenerate freely.
- **Why:** (a) forbids fixing the cascade, which is a real agent cost; (b) is how a tooling change
  moves the language unnoticed. Middle: every regeneration ships with the diff in the phase report
  and a one-line "what moved and why" in `EXECUTION_LOG.md`.
- **Confidence:** high. **Needs:** the owner's agreement to review each diff.

## D-NE-4 — Adopt stale-state guards and node-addressed edits; reject the graph store — TAKEN (design), PROPOSED (scope)
- **Evidence:** ZeroLang's `--expect-graph-hash`, `verify-projection` (RESEARCH §1); CodeStruct's
  measured +1.2–5.0% Pass@1 and 12–38% fewer tokens for AST-entity edits (RESEARCH §5); the Atlas
  already assigns stable ids; the checker already emits byte-range repairs; `fix` already refuses
  morph-stored files.
- **Assumptions:** agents in 2026 use stale-state-guarded structured edits when offered; the
  measured gains transfer to a language with a canonical formatter.
- **Alternatives:** (a) a graph program database with projections; (b) text edits only.
- **Why:** the store forks every text tool (git, diff, review, the Survey) and moves the trust
  surface to a binary artifact; the guard and the node address deliver the measured part of the
  benefit on text. Two steps keep step 1 trivial and testable.
- **Confidence:** high on rejecting the store; medium on node-addressed edits earning their keep
  (needs a measurement in P4-05: tokens per successful edit on the guide corpus).
- **Needs verification:** the measurement above; that `format_source` round-trips a replaced body.

## D-NE-5 — Ship an Agent-Skills-standard skill — PROPOSED
- **Evidence:** the specification (frontmatter fields, 500-line limit, progressive disclosure);
  Mojo and ZeroLang both ship one; 20+ harnesses consume the format; the owner's "agent-specific
  docs/skills" line.
- **Alternatives:** (a) rely on `for-agents.md` alone; (b) a proprietary agent doc.
- **Why:** the standard is what harnesses load; `for-agents.md` stays the pinned reference and the
  skill points at it. Folder `skills/delulu/` (name must equal `name:`).
- **Confidence:** high. **Needs:** owner's ok on the folder name and on validating with the
  reference tool in CI (a Node step already exists).

## D-NE-6 — `delulu mcp`: read-only, stateless, hand-written JSON-RPC in the CLI — PROPOSED
- **Evidence:** MCP 2026-07-28 removed sessions and the handshake, wants deterministic list order
  and cache hints; the LSP is analysis-only by construction and hand-written; dependency austerity.
- **Alternatives:** (a) an SDK dependency; (b) a separate binary; (c) expose only the LSP.
- **Why:** the door rule — the server that reads must never be an effector — is what makes it safe
  to point at a hostile workspace; the LSP proves the hand-written protocol layer is affordable; a
  separate binary complicates distribution.
- **Confidence:** medium-high. **Needs:** owner's ok to add a subcommand; verification that every
  tool is annotated `readOnlyHint` and that `run`/`grants`/`load` are absent (a test).

## D-NE-7 — Release channel — PROPOSED (owner-only)
- **Evidence:** README "Not distributed"; the owner's rule that the testing repository is not a
  distribution channel; the four pre-public decisions; `cargo-dist`/attestation practice.
- **Alternatives:** (a) prepare the workflow, run it on the testing repository as *pre-releases
  clearly labelled "testing"*; (b) prepare only, publish at the final repository; (c) publish
  archives elsewhere (a domain), which needs infrastructure the project does not have.
- **Recommendation:** (b) by default, with (a) only if the owner explicitly lifts the "not a
  distribution channel" rule for labelled pre-releases — because an archive on the testing
  repository is discoverable and the owner has said that repository is not for distribution.
- **Confidence:** high that this is the owner's call, not the head chef's.

## D-NE-8 — A hand-written release workflow, not `cargo-dist`; installer posture is the owner's — PROPOSED
- **Evidence:** `scripts/package-toolchain.sh` already encodes the decisions (Python-less build,
  what a recipient is owed, checksums from the staged tree); `cargo-dist` would re-decide them
  and adds a tool the house rule on dependencies would need a ruling for; attestations are one
  action step.
- **Alternatives:** `cargo-dist` init; installer scripts (`curl | sh`) vs none.
- **Why:** reuse what is verified; `curl | sh` is the field's norm and also a posture the security
  policy should choose knowingly (a script verified by checksum before it runs is the middle path).
- **Confidence:** medium. **Needs:** the owner's choice on installers.

## D-NE-9 — No package-manager ecosystem (conda/pixi-style), no crates.io — TAKEN (restates a ruling)
- **Evidence:** `INSTALL.md` §3 and `STABILITY.md` §2 (crates are not an API); Mojo's conda
  channel model (RESEARCH §2).
- **Why:** one static binary needs one archive; a channel ecosystem is a dependency and a
  governance surface for no user benefit. Homebrew/winget/scoop *manifests* (thin pointers to the
  archive) are different and are prepared only after the final public repository exists.
- **Confidence:** high.

## D-NE-10 — The run-time loading grant — PROPOSED
- **Evidence:** NE-01; the grant parser has no plugin dimension; Stage 6 specifies the `Load`
  effect, the ceiling and the holder check but not the *operator-side* grant spelling.
- **Alternatives:** (a) `--grant plugin=<path-or-dir>` (paths through the containment resolver, so
  `..`, symlinks and case cannot widen); (b) a `[plugins]` ceiling in `delulu.toml` naming
  permitted artifacts by hash; (c) both, with (b) reviewable and (a) for single files.
- **Recommendation:** (c). Hash-pinning in a manifest is the supply-chain-honest form and matches
  the lockfile's habit; the flag is the five-minute form.
- **Confidence:** medium. **Needs:** a ruling in the build order (the owner may delegate); the
  skip-branch cases in P2-08.

## D-NE-11 — The archive folder is `docs/archive/`, mirroring original paths — PROPOSED
- **Evidence:** the owner's instruction to move stale material into an organized folder preserving
  structure; the Survey maps every markdown file and lists inbound links, so moves are checkable.
- **Alternatives:** `docs/history/`; per-campaign folders.
- **Why:** mirroring paths makes every move reversible by `git mv` alone and keeps citations
  readable (`archive/design/P19_ECOSYSTEM_REVIEW.md`).
- **Confidence:** high on the mechanism; the name is the owner's taste.

## D-NE-12 — The owner's commission file: original moved beside the repository, redacted copy committed — TAKEN (reversible)
- **Evidence:** the file quotes the word the owner banned from every product surface and the
  repository; the Survey walks every markdown file on disk (not `git ls-files`), so an untracked
  file makes the committed map disagree with CI's tree and fails the freshness gate; committing it
  puts the banned word into the public testing repository.
- **Alternatives:** (a) commit as-is (violates the rule); (b) leave untracked (breaks the gate on
  one side or the other); (c) `.gitignore` it (the walker ignores nothing, so (b) again);
  (d) move the original outside the tree and commit a redacted copy.
- **Why:** (d) is the only option that satisfies the rule, the gate, and durability. The original
  is preserved byte-for-byte at `D:\nelan\DeluluLang_Fable_5.1_Master_Prompt.md` (beside the
  repository, not inside it); the redacted copy is `OWNER_COMMISSION.md` in this folder with the
  one URL and one word replaced by a marker.
- **Confidence:** high that this is the right outcome; the owner may prefer another location.

## D-NE-13 — Research sources are named only in this folder; ideas are copied, code is not — TAKEN
- **Evidence:** the owner's message of 2026-09-17 ("no need to mention them in the product/docs/
  branding unless I explicitly ask"; copying code is permitted by the owner).
- **Why:** naming sources in a research record is provenance, which this project values; naming
  them on product surfaces is what the owner declined. On code: the surveyed repositories are
  Apache-2.0/MIT, so copying code would require carrying their notices (Apache §4) into `NOTICE`
  — a licensing act the owner reserves. Copying *ideas* carries no such obligation; every item in
  the roadmap is a from-scratch design against this codebase's own seams.
- **Confidence:** high.

## D-NE-14 — No LLM-evaluated semantics inside the language — TAKEN
- **Evidence:** Pel's natural-language conditions (RESEARCH §6); DeluluLang's deterministic replay,
  `--assert-trace`, and snapshot depend on determinism.
- **Why:** an agent belongs outside the program, holding a grant; the program stays checkable.
- **Confidence:** high.

## D-NE-15 — No inferred edges in the Survey or the Atlas — TAKEN (restates the provenance law)
- **Evidence:** the KG skill's `INFERRED` tag and codebase-memory's `SEMANTICALLY_RELATED`;
  the Survey's law: a relation that cannot be pointed at in the text is not in the map.
- **Why:** an edge nobody can cite is a discrepancy, not a fact. What *is* adopted: the query
  vocabulary (`diff` → impact) and god-node/community *renderings* if they cite their inputs.
- **Confidence:** high.

## D-NE-16 — The standard library is minor-version work, not an RFC — TAKEN (restates `REMAINING_WORK.md` 2.1)
- **Evidence:** `STABILITY.md` §5 (new syntax/codes are minor, additive); R-4 governs higher-order
  rows; `map` is the precedent.
- **Why:** nothing refuses a program that compiles today. `Map[K, V]` is a new prelude type, still
  additive; its determinism (ordered iteration) is a design choice recorded in the build order.
- **Confidence:** high. **Needs:** the generator coverage task (P3-05), or the C88 lesson repeats.

## D-NE-17 — A command-line test ceiling (`test --test-authority`) needs a ruling — PROPOSED
- **Evidence:** NE-13; invariant 41 (tests hold no ambient authority); today the only ceiling
  source is the manifest.
- **Why:** a second authority source must be explicit and reviewable; a flag on the command line is
  as visible as `--grant` and is refused when wider than the package ceiling. Until ruled, P1-13
  documents the rule only.
- **Confidence:** medium.

## D-NE-18 — The diagnostic-cascade fix is not language-visible — TAKEN
- **Evidence:** NE-04; the 200-deep program is refused before and after; only the *number* of
  diagnostics for an already-refused program changes.
- **Why:** `STABILITY.md` §1 pins that a refused program stays refused with the same code; it does
  not pin the count of follow-on errors. Still regenerated deliberately under D-NE-3.
- **Confidence:** high.

## D-NE-19 — Plugins before the standard library — TAKEN (order), PROPOSED (the owner may swap)
- **Evidence:** NE-01 is the identity's own demo and is documented as running; the stdlib gap is
  documented as open (RW 2.1).
- **Why:** an undocumented gap in the flagship costs more credibility than a documented breadth gap.
- **Confidence:** medium — a user-first view could reasonably put P3 first.
