<!-- The owner's commission for the 2026 evolution plan, 2026-09-17, reproduced from the file the owner placed in docs/design under the name DeluluLang_Fable_5.1_Master_Prompt.md. One repository URL and one word are replaced by "[KG-skill]" because that word is banned from this repository by a standing owner ruling; the unredacted original is preserved byte-for-byte beside the repository at D:\nelan\DeluluLang_Fable_5.1_Master_Prompt.md (DECISION_LOG.md D-NE-12). Nothing else was changed. -->

Claude Fable 5.1, max effort. 32-star Michelin head chef is in the kitchen. Time to cook something REALLY big for DeluluLang. 🍳🔥

You are working in:

D:\nelan\DeluluLang

Before touching anything, read these completely:

D:\nelan\DeluluLang\README.md
D:\nelan\DeluluLang\HANDOFF.md

Treat README.md + HANDOFF.md as the newest project authority. A lot of the other docs may be stale. Verify them against the actual code, tests, Survey, doctor, git history, and current binaries before believing them.

Also read and cross-check:

D:\nelan\DeluluLang\docs\book\THE_DELULULANG_BOOK.md
D:\nelan\DeluluLang\docs\DEPLOYMENT.md
D:\nelan\DeluluLang\docs\editors.md
D:\nelan\DeluluLang\docs\for-agents.md
D:\nelan\DeluluLang\docs\GETTING_STARTED.md
D:\nelan\DeluluLang\docs\MATHEMATICS.md
D:\nelan\DeluluLang\docs\QUESTIONS.md
D:\nelan\DeluluLang\docs\REMAINING_WORK.md
D:\nelan\DeluluLang\docs\REPOSITORY_STRUCTURE.md

This is a huge repo, 100k+ lines and mostly Rust. Do NOT blindly read everything. Use the repository's own Survey, impact analysis, doctor, git history, tests and source references to navigate it intelligently.

IMPORTANT MAINTAINER RULES

Use Survey and doctor heavily.

Before changing important code:
- run/check Survey
- use Survey impact / affected-by / query where relevant
- run delulu doctor
- understand the blast radius
- verify claims against source/code/tests

After changes:
- regenerate Survey BEFORE the suite
- run doctor again
- run the relevant focused tests
- then run broader gates
- never claim something is fixed/green unless you actually executed it

The repository already has a strong provenance/evidence culture. Keep it. Never turn an assumption into a fact. Never hide failed experiments. Never fabricate numbers, tests, benchmarks, security findings, or implementation status.

Also inspect and improve Survey, doctor, and Atlas themselves if they are producing weak, stale, ambiguous, or misleading results. I want them to become excellent maintainer/agent tools, not just tools that look good.

GOAL

My original goal for DeluluLang is much bigger than "just another programming language".

I intended it for:
- AI agents
- LLMs
- future AGI / ASI systems
- physical AI
- robots
- autonomous machines
- humans working together with those systems

DeluluLang should not discriminate between humans and AI. The authority model should be based on what the holder is allowed to do, not who the holder is.

I think the project may have drifted from that original goal. Do a serious reassessment.

Do NOT simply add random AI features. Work out what DeluluLang should become for an AI-first future while protecting its original authority/effects/security foundations.

Think in terms of:
- agent-native language design
- authority and capability safety
- machine-readable contracts
- tool/agent interoperability
- deterministic and verifiable execution
- agent-friendly diagnostics
- program understanding
- memory
- repository/codebase understanding
- semantic program manipulation
- safe autonomy
- physical AI / actuator authority
- deployment
- reproducible distribution
- human review of AI-written code

CRITICAL: DO NOT BREAK THE CORE JUST BECAUSE SOMETHING IS COOL.

The Authority and Guard semantics are protected. Harden them, improve them, test them, expand them only where justified, but do not silently redefine what they mean.

WEB RESEARCH

Search the web extensively before making the roadmap.

Start with the links I gave you and actually read them.

ZeroLang:

https://zerolang.ai/
https://zerolang.ai/getting-started
https://zerolang.ai/diagnostics
https://zerolang.ai/testing
https://zerolang.ai/primitives
https://zerolang.ai/reference
https://zerolang.ai/concepts/graph-architecture
https://zerolang.ai/concepts/semantic-vs-text
https://zerolang.ai/concepts/compile-path
https://zerolang.ai/concepts/projections
https://zerolang.ai/learn
https://github.com/vercel-labs/zerolang

Mojo:

https://mojolang.org/
https://mojolang.org/docs/vision/
https://mojolang.org/docs/tools/skills/
https://mojolang.org/docs/manual/python/
https://mojolang.org/docs/manual/python/python-from-mojo/
https://mojolang.org/docs/manual/python/mojo-from-python/
https://mojolang.org/docs/manual/python/types/
https://mojolang.org/docs/manual/c-ffi/
https://mojolang.org/docs/tools/packaging/

Cloudflare / agent infrastructure:

https://developers.cloudflare.com/browser-run/kitesurf/
https://blog.cloudflare.com/kitesurf/
https://developers.cloudflare.com/browser-run/stagehand/
https://github.com/cloudflare/workers-rs
https://github.com/cloudflare/cloudflare-os
https://github.com/cloudflare/security-audit-skill

Also study:

https://github.com/microsoft/agent-framework
https://github.com/anthropics/claude-cookbooks
https://github.com/DeusData/codebase-memory-mcp
[KG-skill repository — name withheld under the owner rule]
https://github.com/mem0ai/mem0
https://github.com/topoteretes/cognee
https://github.com/openai/codex-security

Also search for other AI/agent-oriented programming languages, graph-native languages, capability/security-oriented languages, agent frameworks, code intelligence systems, semantic programming systems, memory systems, and agent tooling that appeared or materially changed recently.

Do not copy competitors blindly. Extract useful ideas, identify tradeoffs, and decide what actually belongs in DeluluLang.

These repos are for mainly research/inspiration, Well u copy anything from them including codes, I don't care. No need mention them into the DeluluLang product/docs/branding unless I explicitly ask for it.

CURRENT IMPLEMENTATION

We already have:
- a DeluluLang compiler/toolchain
- the terminal `delulu` experience
- interpreter/runtime
- WASM backend
- authority/capability system
- broker
- Guard
- plugins
- actors
- LSP/editor support
- Atlas
- Survey
- doctor
- registry/deployment infrastructure
- tests/fuzzing/formal verification/etc.

Do NOT assume every backend supports every language feature.

Test the REAL compiler and REAL terminal.

Write actual `.delulu` programs and run/check/analyse them.

Find gaps between:
- language specification
- compiler
- terminal
- interpreter
- WASM backend
- LSP
- Atlas
- agent tooling
- docs
- distribution/install experience

The current docs explicitly say some things are still partial/not built, so verify the current tree instead of assuming a roadmap item is implemented. For example, the repository's current status distinguishes the missing microVM layer, limited standard library, and missing native/DIR backend.

AGENT/DEVELOPER EXPERIENCE

DeluluLang should become much easier for AI agents to use correctly.

Study ideas from ZeroLang, [KG-skill], codebase-memory, Mem0, Cognee, Microsoft Agent Framework and similar systems.

Think especially about:
- semantic/codebase querying
- provenance
- blast-radius analysis
- persistent project memory
- agent-safe edits
- machine-readable diagnostics
- structured repair actions
- checked semantic patches
- deterministic tool interfaces
- agent-specific docs/skills
- Atlas for agents
- Survey for maintainers and agents
- long-running autonomous development workflows
- minimizing context/token waste without sacrificing correctness

I care much more about grounded intelligence than "AI magic".

DISTRIBUTION

I want normal people to be able to install and use DeluluLang, like a real modern language/toolchain.

Study ZeroLang's install/getting-started experience and Mojo's packaging approach.

Work toward a realistic path for:
- releases
- downloadable binaries
- checksums/provenance
- Windows/Linux/macOS
- source builds
- package/distribution strategy
- examples
- first 5-minute experience
- versioning
- upgrade story
- documentation
- CI/release automation

Do not invent a package-manager strategy just because other languages have one. Decide what fits DeluluLang's architecture.

CREDIT / COST RULE

I have promotional Claude credit available, but I do NOT want it wasted.

Use the strongest reasoning available for the main session.

Use sub-agents ONLY when they provide real independent value.

If agents are used:
- Opus or Sonnet only
- use high reasoning
- keep the number of agents small
- do not duplicate work
- stop unnecessary agents immediately if limits are being approached
- save useful agent findings/results to durable `.md` files before anything can be lost
- never rely on agent output that has not been independently checked

Do NOT spawn agents just because you can.

PLAN FIRST

This is extremely important.

FIRST create a complete execution/research plan.

Do NOT start large implementation work immediately.

Create a NEW folder under:

D:\nelan\DeluluLang\docs\

Use a clear name such as:

docs\NEXT_EVOLUTION_2026\

Inside it create at minimum:

MASTER_PLAN.md
RESEARCH.md
IMPLEMENTATION_ROADMAP.md
DECISION_LOG.md
EXECUTION_LOG.md

The plan must cover:
- what DeluluLang is today
- where it drifted
- what the real AI-first goal should be
- what competitors/tools are doing
- what DeluluLang already does better/uniquely
- major gaps
- what should be implemented
- what should NOT be implemented
- dependencies between tasks
- security implications
- compiler/runtime implications
- agent implications
- distribution implications
- documentation implications
- testing/verification requirements
- phased implementation order
- estimated difficulty/effort
- what can be done now
- what requires major architecture changes
- what requires owner decisions
- what must remain deferred

Do not give me hidden chain-of-thought.

Instead, record useful decision rationale:
- evidence
- assumptions
- alternatives considered
- why a direction was selected
- confidence
- what still needs verification

I want a strong engineering decision record, not fake certainty.

PHASES

After the plan is complete, STOP.

Wait for my explicit approval before expensive implementation.

After I approve, work in phases.

At the end of EVERY phase:
1. save all work
2. update EXECUTION_LOG.md
3. update DECISION_LOG.md if needed
4. run the appropriate verification
5. regenerate Survey
6. run doctor
7. commit
8. push to the TESTING remote only
9. record the commit/result
10. stop and report exactly what happened

If autonomous continuation is explicitly enabled after approval, use only a one-shot continuation mechanism, never an uncontrolled recurring loop. A newer instruction from me always overrides older scheduled work.

REPOSITORY / GIT RULES

Follow HANDOFF.md exactly.

For GitHub:
- push only to the existing testing remote
- never create/push to the future final public repo
- never rewrite already-pushed history
- keep local and testing remote synchronized
- commit and push completed work
- never pretend a local result is CI verification
- before the final public repository is ever created/used, remind me of the owner-gated decisions and wait for my decision

DOCUMENT CLEANUP

The repo has become very large and has many documents.

Do a documentation audit.

Separate:
- current authoritative docs
- normative specs
- active maintainer docs
- historical campaign records
- generated docs
- obsolete/stale docs
- duplicates
- superseded docs
- release snapshots that must not be rewritten

Do NOT casually delete history.

Move unwanted/stale material into a new organized folder under docs, preserving the original folder/filename structure as much as possible.

For every moved file record:
- original path
- new path
- reason
- whether it was duplicate/superseded/historical/etc.
- whether anything still references it

Never destroy historical evidence just to make the tree prettier.

MEMORY

Keep Claude project memory updated, but only after independent verification.

Because memory outside the repo is not portable, durable important knowledge must also exist in repository `.md` files.

LOG EVERYTHING IMPORTANT

Keep a clear markdown log of:
- what was inspected
- commands actually run
- web research performed
- important findings
- failed attempts
- bugs found
- fixes
- tests
- CI results
- decisions
- files moved
- files created
- commits
- push results
- unresolved questions

Do not fill logs with pointless narration.

QUALITY BAR

I want DeluluLang to become a serious language/toolchain for the AI-native future.

Not vaporware.
Not AI-generated docs stacked on top of AI-generated docs.
Not fake benchmarks.
Not fake security claims.
Not "AGI language" marketing without engineering underneath.

Build the evidence first.

When there is disagreement:
code/tests/current binary > current README/HANDOFF > other project docs > old historical docs > assumptions.

Use the web to learn.
Use the repository to verify.
Use tests to prove.
Use logs to remember.

And remember the original mission:

DeluluLang is a language of the future.

Let's cook this my 32 star head Michelin head chef and most powerful model in the world Claude Fable 5.1 for entire world of ai and humans and future ai evolutions such as agi, asi, physical ai, robots etc to use and work easily.