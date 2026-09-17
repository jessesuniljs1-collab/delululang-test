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

---

## Records added by the sandbox pass (2026-09-17, afternoon)

## D-NE-20 — The owner's commission files live in `docs/design`, the first redacted in place — RULED (by the owner's action); supersedes D-NE-12
- **Evidence:** the owner moved the first commission back to `docs/design` and placed the second
  beside it; the first contained the banned word three times; the Survey walks every markdown file
  on disk, so an untracked file breaks the freshness gate on one side or the other.
- **Why:** the owner's placement is the decision; the banned-word rule is also the owner's; the only
  way to honour both is to redact the word in place (a header note says so) and commit both files,
  keeping the precedent of committing commission texts beside the founding one.
- **Confidence:** high. The old redacted copy became a pointer.

## D-NE-21 — Sandboxing is a cross-cutting execution layer, introduced early; the microVM is re-sequenced, not deferred — PROPOSED
- **Evidence:** `MASTER_PLAN.md` §12.1's four points; `SANDBOX_RESEARCH.md` §0 and §1.6;
  `VERIFICATION_FINDINGS.md` §4; the runners' measured capabilities.
- **Assumptions:** the effect-channel model costs one IPC round trip per synchronous effect and that
  cost is acceptable at the command layer (measured in PS-A before PS-B batches anything).
- **Alternatives:** (a) keep microVM deferred and sandboxing in P8; (b) microVM first (Linux-only,
  leaving Windows and macOS users with nothing); (c) OS-policy translation instead of a channel.
- **Why:** (a) leaves every "untrusted code" claim unbacked on the developer's own machine; (b) builds
  the strongest tier for the fewest users; (c) is three encodings of one authority. The chosen order
  gives every OS a real boundary first and the hypervisor second, on one protocol.
- **Confidence:** high on the layer and the order; medium on effort.
- **Needs verification:** the round-trip measurement; PS-0-08's CI experiments.

## D-NE-22 — The guest performs no effects: capabilities are handles, the host performs everything over one bounded channel — PROPOSED (design)
- **Evidence:** the WASM host already works this way (opaque handles, the host performs effects); the
  foreign worker is the same shape inverted; Nitro, Hyperlight and Docker Sandboxes all keep
  authority outside the guest (`SANDBOX_RESEARCH.md` §3).
- **Alternatives:** the guest holds capabilities and the OS policy mirrors the scopes.
- **Why:** one policy for every backend ("nothing but the channel"); no scope is ever re-encoded;
  secrets and the broker address never enter the guest (`SANDBOX_ARCHITECTURE.md` §2.1).
- **Confidence:** high. **Needs:** the `EffectSink` seam kept byte-identical for the local path (the
  core-invariance snapshot), the channel fuzzed from day one.

## D-NE-23 — The microVM guest runs the interpreter, not the WASM engine — PROPOSED (a Stage 5 §6 deviation; a ruling)
- **Evidence:** the WASM backend compiles 6 of 19 entry programs (`HOT_PATH_TABLE.md`); a tier that
  refuses most programs with `DL1201` is a tier in name.
- **Why:** the microVM supplies the boundary the WASM floor stood in for; the WASM engine keeps its
  in-process containment role. No language semantics change; a build-order deviation, recorded.
- **Confidence:** high.

## D-NE-24 — Levels L0–L4 as honest labels; profiles as named policies; `--isolation process` strengthened and versioned — PROPOSED
- **Evidence:** `STABILITY.md` §2 allows a profile's *strength* to improve while its label stays
  honest; today's `process` isolates foreign code only and its label is absent under `--json`.
- **Why:** keep the flag, strengthen its meaning, announce it; the narrow behaviour stays reachable as
  `--foreign-isolation process`. **Owner:** the profile names (`dev`, `contained`, `hostile-agent`).
- **Confidence:** medium-high.

## D-NE-25 — `Secret.map` under strong profiles: refuse first — PROPOSED (owner)
- **Evidence:** `map` hands a closure the plaintext; under "secrets never enter the guest" it cannot
  run there (`SANDBOX_ARCHITECTURE.md` §8.3).
- **Alternatives:** execute the closure host-side; allow plaintext under a labelled flag.
- **Why:** refusal is honest and safe; the host-side option is a follow-up once measured. It changes
  what a valid program does under a profile, hence the owner.
- **Confidence:** medium.

## D-NE-26 — No VMM crate; drive VMM binaries; rulings for the `landlock` and `seccompiler` crates; `birdcage` rejected — PROPOSED (owner: the two crates)
- **Evidence:** `landlock` 0.4.7 (MIT or Apache, maintained); `seccompiler` (Apache or BSD-3, now in
  the rust-vmm monorepo) — both checked by the Sonnet sous-chef and by the head chef's own fetches;
  `birdcage` is GPL-3.0 and archived (2026-07); Firecracker and Cloud Hypervisor are driven over a
  Unix-socket HTTP API a hand-written client can speak; `bubblewrap` is LGPL and an *external*
  binary — not a linked dependency, so not an allowlist question — and is not relied on anyway,
  because unprivileged user namespaces are blocked on Ubuntu 24.04 defaults.
- **Why:** dependency austerity; `deny.toml`'s allowlist; the VMM stays a process the jailer
  confines. Windows needs nothing new (`windows-sys` already carries Job Objects, tokens and
  AppContainer); macOS needs one `sandbox_init` FFI declaration.
- **Confidence:** high.

## D-NE-27 — Distributing a built guest kernel is a licensing act — PROPOSED (owner)
- **Evidence:** the Linux kernel is GPL-2.0; shipping a built kernel requires offering its source;
  `deny.toml`'s allowlist governs crates, not this.
- **Alternatives:** ship the image; ship only a build script; point at a vendor's published guest
  kernel by hash.
- **Why:** an owner decision with a licensing consequence, like the licence itself.
- **Confidence:** high that it is the owner's.

## D-NE-28 — The first network client is the host-side egress proxy; special-use addresses need their own spelling — PROPOSED (owner)
- **Evidence:** `http.get` returns `Err(Refused)` unconditionally (NE-17); `--grant net=169.254.169.254`
  is accepted silently (NE-18); the AgentCore DNS channel and the metadata-credential class
  (`SANDBOX_RESEARCH.md` §1.2); the Claude Code runtime's resolve-once-and-pin rule.
- **Alternatives:** a full HTTP crate with TLS (a large tree) versus a minimal client; refusing
  special-use ranges outright versus a distinct grant spelling.
- **Why:** whichever client is chosen, it is one implementation serving L0 and guests alike, built with
  the allowlist, pinned addresses, SNI/Host agreement and special-use refusal from its first line;
  the TLS dependency is the largest this project would take and needs the owner and a `cargo deny`
  pass.
- **Confidence:** high on the design; the dependency is the owner's.

## D-NE-29 — Windows reserved device names, trailing characters and drive-relative spellings are refused at the primitive table — PROPOSED (a P1 hardening)
- **Evidence:** NE-19, NE-20 and the drive-relative case, reproduced by the head chef after the red
  team reported them.
- **Why:** the 2026-08-10 search key — a decision on an unnormalized spelling; the audit record must
  carry the resolved name. Independent of the sandbox: the host performs the write either way.
- **Confidence:** high.

## D-NE-30 — The foreign-worker channel gets IPC-1's read deadline — PROPOSED (a P1 hardening)
- **Evidence:** NE-21 (`WorkerConn::call` reads unbounded; `set_read_timeout` is used only by the daemon).
- **Why:** the one profile that exists to contain foreign code can be hung by it.
- **Confidence:** high.

## D-NE-31 — The main program gets resource budgets on every engine — PROPOSED (owner: the defaults)
- **Evidence:** NE-22 (no bound on either engine; an unbounded mailbox reaches beyond a gigabyte with
  only a console grant).
- **Why:** a limit kill must reuse `limits.rs`'s attribution rule (never a widening repair); "never
  unlimited" is the plugin precedent. Defaults are the owner's.
- **Confidence:** high on need; medium on the mechanism (an interpreter step budget versus OS controls).

## D-NE-32 — Sous-chefs were used on this pass at the owner's direction, under the owner's rule of 2026-09-17 — RULED (owner) / TAKEN (application)
- **Evidence:** the owner's messages ("use opus 5 and sonnet 5 as agents"; then the agent rule). Two
  agents: Opus 5 (attack surfaces and the adversarial matrix) and Sonnet 5 (host-capability facts).
  Both ran in isolated worktrees, modified nothing, and wrote one notes file each; the notes are in
  `agent-notes`, their worktree files are copied to durable storage beside the repository
  (`EXECUTION_LOG.md` Entry 3), and every claim used in these documents was re-verified by the head
  chef (the http stub, the worker deadline, the device names, the trailing characters, CVE-2026-1386).
  One agent claim was **contradicted by measurement** (Windows Hypervisor Platform "off by default" on
  `windows-latest`; the probe measured it enabled) and the measurement stands.
- **Why:** real independent value — a second reader of the containment code found what the head
  chef's own battery had not (the device names, the worker deadline); the host-fact research would
  have cost the head chef forty fetches.
- **Confidence:** high.

## D-NE-33 — `--sandbox` stays off by default in 1.x; a package may require a level — PROPOSED (owner)
- **Evidence:** `DEPLOYMENT.md` §6's reasoning for not flipping strict mode: a default that breaks the
  primary workflow teaches people to disable it.
- **Why:** the honest default is refusal-or-run under the level the operator chose; a
  `[sandbox] require_level` in `delulu.toml` lets a package demand a level without changing every
  user's default. Flipping the default is a major-version act, like strict roots.
- **Confidence:** medium — the owner may prefer sandbox-on for `--json` or non-TTY runs.
