# DeluluLang V2 — Execution Prompt for Claude Fable 5.1

Fable 5.1, approval is given. 🍳🔥

THE OWNER HAS NOW APPROVED THE NEXT EVOLUTION PLAN.

We are moving from planning into EXECUTION.

This is now:

# DELULULANG V2

Do NOT treat the old v1.0 architecture/history as something to rewrite away.

V1 remains historical and preserved.

V2 is the new active evolution path.

---

## 1. FIRST: READ THE APPROVED PLAN

Before implementing anything, read the complete current approved plan:

```text
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\README.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\MASTER_PLAN.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\RESEARCH.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\VERIFICATION_FINDINGS.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\IMPLEMENTATION_ROADMAP.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\DECISION_LOG.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\EXECUTION_LOG.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\DOCUMENTATION_AUDIT.md
```

Also read:

```text
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\SANDBOX_RESEARCH.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\SANDBOX_ARCHITECTURE.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\SANDBOX_THREAT_MODEL.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\SANDBOX_TEST_PLAN.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\SANDBOX_IMPLEMENTATION_PLAN.md
```

Also read current:

```text
D:\nelan\DeluluLang\README.md
D:\nelan\DeluluLang\HANDOFF.md
D:\nelan\DeluluLang\docs\MATHEMATICS.md
D:\nelan\DeluluLang\docs\GETTING_STARTED.md
D:\nelan\DeluluLang\docs\DEPLOYMENT.md
D:\nelan\DeluluLang\docs\for-agents.md
D:\nelan\DeluluLang\docs\QUESTIONS.md
D:\nelan\DeluluLang\docs\REMAINING_WORK.md
D:\nelan\DeluluLang\docs\REPOSITORY_STRUCTURE.md
```

Use the CURRENT TREE + CURRENT BINARY + Survey + doctor as truth when old documents disagree.

---

## 2. THIS IS DELULULANG V2

We are now starting the implementation of:

**DELULULANG V2**

Do not confuse:

- v1.0 historical implementation
- v1 historical documentation
- current v1 production-readiness records
- V2 planning
- V2 implementation

V1 history must remain recoverable.

V2 should become the active forward-development structure.

Create a new active documentation root:

```text
D:\nelan\DeluluLang\docs\DELULULANG_V2\
```

Use V2 naming consistently.

For example:

```text
docs/DELULULANG_V2/
    V2_README.md
    V2_MASTER_PLAN.md
    V2_IMPLEMENTATION_ROADMAP.md
    V2_EXECUTION_LOG.md
    V2_DECISION_LOG.md
    V2_PHASE_STATUS.md
    V2_DOC_MOVE_MANIFEST.md
    V2_AGENT_LOG.md
    V2_SECURITY_MODEL.md
    V2_AI_NATIVE_DESIGN.md
```

and supporting folders as actually needed.

Do not blindly duplicate the old `NEXT_EVOLUTION_2026` folder.

Create the V2 documentation structure as the ACTIVE working source.

The old `NEXT_EVOLUTION_2026` planning folder becomes historical planning material once the V2 migration is complete.

---

## 3. VERY IMPORTANT — DOCUMENTATION CLEANUP

The repository has too many documents and I am wasting tokens maintaining documents that are no longer the active truth.

I want this fixed as part of V2.

Do NOT delete old documentation.

DO NOT COPY old documentation.

MOVE old/superseded/historical documentation into an archive.

Use:

```text
D:\nelan\DeluluLang\docs\archive\v1\
```

or another clearly named V1 historical archive under docs if the current repository structure makes a better choice.

PRESERVE the original relative folder/file structure inside the archive wherever practical.

Example:

```text
old:
docs/design/OLD_PLAN.md

new:
docs/archive/v1/design/OLD_PLAN.md
```

OLD FILE → MOVED, NOT COPIED.

The file must no longer remain active in its old location.

Create:

```text
docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md
```

For every moved file record:

- original path
- new path
- why it was moved
- whether it was historical / superseded / duplicate / campaign record / generated / obsolete
- whether inbound links had to be changed
- whether it remains authoritative anywhere

DO NOT move active normative documentation just because it is old.

First determine whether each document is:

1. V2-active
2. normative and still required
3. current operational documentation
4. historical V1 record
5. superseded
6. generated
7. duplicate
8. stale
9. obsolete

The objective is:

**ONE ACTIVE V2 SOURCE OF TRUTH.**

Historical information must still be available, but Claude should not have to repeatedly update 20 old documents whenever one V2 design decision changes.

Update links so active documentation points to the V2 source.

Do NOT break historical references.

Do NOT rewrite old commit history.

---

## 4. V2 EXECUTION LOGS

From this point onward, keep the main implementation state in:

```text
docs/DELULULANG_V2/V2_EXECUTION_LOG.md
```

Also maintain:

```text
docs/DELULULANG_V2/V2_PHASE_STATUS.md
docs/DELULULANG_V2/V2_DECISION_LOG.md
docs/DELULULANG_V2/V2_AGENT_LOG.md
```

These should be compact and useful.

Record:

- phase started
- phase completed
- files changed
- commands actually run
- tests
- Survey result
- doctor result
- CI result
- commits
- pushes
- failures
- fixes
- unresolved issues
- owner decisions
- agent work
- agent failures
- important architectural discoveries

Do NOT write pointless narration.

Do NOT store private reasoning transcripts.

Save conclusions, evidence, decisions, failed attempts and outputs.

---

## 5. OPUS 5 IS THE MAIN SOUS-CHEF

You are Claude Fable 5.1.

You are the 32-star Michelin head chef.

Use **Opus 5 as the MAIN execution sous-chef.**

Delegate MOST implementation work to Opus 5.

Use high reasoning.

Do NOT run agents just for the sake of running them.

Use Opus 5 for:

- implementation
- code investigation
- focused architecture work
- tests
- refactors
- security review
- adversarial test development
- documentation migration
- verification work
- research-backed implementation tasks
- repository cleanup
- Survey/doctor integration
- sandbox implementation
- microVM work where host/platform permits

You are still the final architect.

Before delegating work, give Opus 5:

- exact objective
- relevant files
- constraints
- security requirements
- verification requirements
- expected outputs

If Opus 5 encounters a problem:

- it should ask YOU
- you decide how to proceed
- do not let the agent silently invent an architectural decision

If Opus produces something important:

- inspect it
- verify it
- run the relevant tests
- do not blindly trust the agent

If an agent discovers something useful, save it to:

```text
docs/DELULULANG_V2/V2_AGENT_LOG.md
```

or a focused V2 agent note when necessary.

Do not waste credits with parallel agents doing duplicate work.

Prefer ONE strong Opus 5 execution agent at a time.

Use another agent only when there is a real independent reason.

---

## 6. REASONING MODE

Use maximum reasoning for the main session.

For Opus 5:

- high reasoning
- deep code understanding
- verify before changing
- do not guess
- do not fabricate
- do not silently simplify security-critical behavior

If an agent reaches a session/usage limitation:

1. stop it safely
2. save what it completed
3. save its findings
4. record the interruption
5. do not lose work

---

## 7. PHASE EXECUTION MODEL

Execute the V2 implementation in PHASES.

DO NOT attempt the entire project in one giant operation.

Follow the approved roadmap and dependencies.

At the start of each phase:

1. read the exact phase objectives
2. inspect affected code
3. run Survey
4. run doctor
5. establish baseline
6. delegate most implementation to Opus 5
7. supervise
8. verify
9. test
10. regenerate Survey
11. run doctor
12. run the relevant gates
13. update V2 logs
14. commit
15. push to the TESTING remote
16. record exact result
17. STOP

Then wait approximately 60 seconds.

If a new user message arrives during that wait:

STOP and process it.

If there is no new input after 60 seconds:

CONTINUE automatically to the next approved phase.

Do NOT create an uncontrolled infinite background loop.

Use exactly one controlled phase-to-phase continuation.

A newer owner instruction always overrides an older plan.

---

## 8. PHASE ORDER

Use the approved roadmap as the baseline.

Do not casually reorder it.

The current V2 execution structure includes:

### P1
Machine-contract truth

### PS-0
Sandbox truth/probes/cheap hardening

### PS-A
L1 whole-program process sandbox

### P2
Real plugin loading

### P4a
Agent Skill

### P3
Standard library expansion

### PS-B
Resource limits + egress + identity separation

### P4b/P4c/P4d
Agent surfaces, MCP, checked edits, Atlas/Survey agent tooling

### PS-C
Linux/KVM microVM

### P6
Documentation consolidation

### P5
Distribution

### P7
Verification depth

### PS-D
External sandbox + attestation seam

### P8
Safe autonomy / hardware / owner-gated future work

BUT:

Treat this as an execution dependency graph, not a blind checklist.

If actual code reveals a dependency that requires changing the order:

- stop
- record the evidence
- explain the dependency
- update the V2 plan
- continue only when the decision is justified

Do not reorder phases merely because it is convenient.

---

## 9. CORE ARCHITECTURAL RULE

DO NOT BREAK THE CORE JUST BECAUSE SOMETHING IS COOL.

The meanings of:

- Authority
- Guard
- effects
- capabilities
- broker custody
- revocation
- attenuation
- audit

must remain stable.

You may:

- harden them
- fix bugs
- expand them
- add enforcement layers

You may NOT silently redefine their semantics.

The sandbox is an ENFORCEMENT LAYER underneath the Authority model.

It is NOT a replacement for Authority.

It is NOT a competing permission system.

---

## 10. V2 AUTHORITY + SANDBOX MODEL

Use this conceptual architecture:

```text
principal
    ↓
intent / program
    ↓
static effects
    ↓
static authority analysis
    ↓
operator grant / lease
    ↓
Guard
    ↓
sandbox policy
    ↓
sandbox backend
    ↓
host effect channel
    ↓
broker / custody
    ↓
real effect
    ↓
audit
```

The sandbox may REDUCE environmental reach.

It must NEVER create authority that the program did not have.

A sandbox backend must never reinterpret Authority.

Therefore:

- container
- gVisor
- process sandbox
- microVM
- Kubernetes
- remote sandbox
- cloud sandbox
- future confidential VM

must all ultimately enforce the SAME Delulu authority semantics.

DO NOT create:

```text
Authority semantics #1 for process sandbox
Authority semantics #2 for microVM
Authority semantics #3 for cloud
```

There must be one semantic authority model.

---

## 11. GUARD

Guard remains the approval/authorization layer.

Sandbox is physical/environmental enforcement.

Broker is custody/validation.

Host executes actual effects.

Audit records them.

Conceptually:

**Guard:**
"Is this operation/delegation allowed?"

**Authority:**
"What can this program request?"

**Sandbox:**
"What can this execution environment physically reach?"

**Broker:**
"Is this request backed by valid custody/lease authority?"

**Host:**
"Perform the effect."

**Audit:**
"Record what happened."

Do NOT turn Guard into a giant sandbox implementation.

---

## 12. SAME-USER ROOT TRUST ISSUE

Treat the existing same-OS-user trust boundary as a major security constraint.

Do not pretend that sandboxing alone solves it.

For autonomous agent deployments, eventually prefer:

```text
human identity
    ≠
agent OS identity
    ≠
broker trust root
```

where justified by the deployment model.

Do not make:

**human vs AI**

the security boundary.

Instead use:

- principal
- identity
- authority
- grant
- sandbox
- attestation
- audit

The system must not say:

```text
human = trusted
AI = untrusted
```

or:

```text
robot = special
AGI = special
```

Security depends on authority and identity, not species/category.

---

# 13. RESOURCE AUTHORITY

Do NOT treat CPU/memory/time/actor-count/disk/network limits as merely launcher configuration.

V2 should move toward treating resources as first-class authority dimensions.

Investigate and implement where the approved phase allows:

- CPU budget
- memory budget
- wall-clock budget
- process count
- actor/concurrency budget
- filesystem quota
- network byte/request budget
- device-use budget
- GPU/device budget where applicable

Think:

```text
Authority =
effects
+ resources
+ devices
+ network scope
+ other enforceable constraints
```

Do not change the mathematics without documenting and verifying it.

If something cannot yet be made a formal authority dimension, keep that limitation explicit.

---

## 14. RESTRICTIONS: ON/OFF, BREAK-GLASS, AND SAFE MODES

This is important.

The execution restrictions MUST be usable in practice, but they must not become a silent security bypass.

### Sandbox control

The sandbox should be able to be explicitly selected:

```text
--sandbox
--sandbox=off
--sandbox-profile <name>
--isolation <level/backend>
```

or the equivalent final CLI/configuration that the architecture determines is best.

But:

- secure sandboxing should be the normal/default posture where appropriate
- disabling sandboxing must be explicit
- a request for a stronger level must NEVER silently downgrade
- an unsupported level must refuse
- an AI/agent program must NOT be allowed to disable its own sandbox
- a sandboxed principal must not be able to grant itself a weaker isolation level
- break-glass operations must be initiated outside the sandbox by an authorized principal

### Authority control

Do NOT implement "Authority OFF" as "compiler stops caring".

Authority remains a semantic property of the program.

Instead, design distinct execution modes if needed:

```text
STRICT / ENFORCED
    static authority + runtime custody + Guard + sandbox

AUDIT / DRY-RUN
    do not perform effects
    calculate and report required authority
    show would-be requests

PRIVILEGED / BREAK-GLASS
    explicit external operator action
    separate trust boundary
    highly visible
    fully audited
    never activated by program code
```

A future privileged mode may allow a trusted developer/operator to run with broader host rights, but it must be obvious that this is an operational escape hatch, not a change to the language's semantics.

DO NOT allow:

```text
program code
    ↓
disable Authority
```

DO NOT allow:

```text
program code
    ↓
disable Guard
```

DO NOT allow:

```text
AI agent
    ↓
turn off its own security
```

The safest principle is:

> Restrictions can be relaxed by an authorized external principal, but a program cannot relax its own authority.

Study whether sandbox ON/OFF and authority strict/audit/break-glass modes can be made clean, composable, reversible, and fully observable.

Test:

- ON → OFF
- OFF → ON
- strict → audit
- audit → strict
- break-glass → strict
- failed/unsupported transitions
- transitions during execution
- transitions after delegation
- transitions after plugin load
- transitions during a sandbox escape attempt
- whether a child process can inherit a weaker policy
- whether a revoked authority can accidentally become usable after a transition

Prefer revocation/kill + restart over hot weakening of a running sandbox when that is the safer semantics.

The final design must explicitly distinguish:

**language semantics**
from
**execution enforcement**
from
**operator break-glass controls**.

---

# 15. SANDBOX ARCHITECTURE

Implement the approved sandbox architecture.

The major goal:

A DeluluLang program should be able to execute under different isolation backends while retaining one authority model.

Possible levels include the approved V2 L0/L1/L2/L3/L4 architecture.

Do not silently downgrade a requested security level.

If the host cannot provide the requested guarantee:

**REFUSE.**

Do not say:

"microVM requested, but container was close enough."

Machine-readable output should explicitly report:

- requested level
- actual level
- backend
- host capabilities
- guarantees
- limitations
- resource limits
- network posture
- filesystem posture
- identity posture
- sandbox state
- whether restrictions are fully enforced
- whether a break-glass mode is active

---

## 16. MICROVM

Implement the approved microVM architecture when the PS-C phase is reached.

Current intended principle:

The microVM guest should not perform arbitrary host effects.

Use a bounded host/guest channel.

Keep authority enforcement outside the guest.

Do NOT make the microVM a second Authority implementation.

Respect the approved V2 ruling regarding the guest:

The microVM guest is intended to execute the Delulu interpreter rather than simply assuming WASM is the universal guest.

This is an owner-approved V2 architectural decision.

However, before implementation, verify every practical implication against the actual code and existing backend capabilities.

Do not blindly implement a design from a document if the code proves a prerequisite is missing.

---

## 17. AI-NATIVE DESIGN

This is a major V2 goal.

DeluluLang's PRIMARY anticipated users are:

- AI agents
- LLM-driven agents
- local models
- cloud models
- autonomous software
- robotics systems
- physical AI
- future AGI-like systems
- future AI architectures we cannot predict yet

Humans remain first-class users.

There must be NO discrimination in the language model between:

- human
- AI
- agent
- robot
- AGI
- ASI
- physical AI
- future AI

Use one computational-principal model.

Different principals receive different authority according to explicit policy.

Do not invent "AI mode" or "robot mode" unless there is a real technical reason.

---

## 18. ZERO-SHOT / NEVER-TRAINED AI

One of the most important V2 objectives:

An AI model that has NEVER been trained on DeluluLang should still be able to learn enough about DeluluLang from the toolchain to write correct programs.

Do not assume model pretraining.

Build toward:

```text
delulu toolchain --json
delulu skill
delulu schema
delulu examples --json
delulu explain
delulu repair
delulu authority --json
delulu atlas --json
delulu doctor --json
```

and the appropriate MCP/LSP surfaces.

The desired loop is:

```text
unknown model
    ↓
discover language
    ↓
read compact machine-readable contract
    ↓
generate program
    ↓
check
    ↓
receive structured diagnostics
    ↓
repair
    ↓
authority analysis
    ↓
sandbox analysis
    ↓
test
    ↓
security analysis
    ↓
run
```

Do NOT require every AI system to have memorized the language beforehand.

The toolchain should teach the model enough to use it.

---

## 19. AI-FRIENDLY SYNTAX

Do not simplify DeluluLang by blindly making it look like Python.

AI-friendly means:

- regular syntax
- low ambiguity
- explicit semantics
- stable grammar
- strong diagnostics
- machine-readable errors
- structured repairs
- discoverability
- canonical representation
- good examples
- predictable rules

Use syntax morphs where they genuinely help, but preserve one canonical semantic representation.

Treat the current syntax as something to MEASURE, not merely defend.

Measure things like:

- time to first successful program
- number of correction loops
- token/context cost
- number of undocumented surprises
- diagnostic repair success
- agent task success
- authority-review effort

Use `delulu-measure` where appropriate.

### Add a V2 AI usability benchmark

Create a reproducible benchmark for models that have:

1. never seen DeluluLang
2. only the compact Agent Skill
3. Skill + toolchain introspection
4. Skill + MCP/LSP
5. full documentation

Measure:

- compile-first-try rate
- successful task completion
- repair iterations
- tokens consumed
- authority mistakes
- sandbox-policy mistakes
- security-test failures

This will tell us whether Delulu is actually AI-friendly rather than merely claiming it.

---

## 20. AI SECURITY AUDITS AI

Design the V2 workflow around:

```text
AI writes code
    ↓
AI explains code
    ↓
AI queries Atlas
    ↓
AI runs Survey
    ↓
AI checks authority
    ↓
AI checks sandbox
    ↓
AI generates adversarial tests
    ↓
AI attempts attacks
    ↓
AI reviews audit data
    ↓
AI proposes remediation
    ↓
human security expert reviews important boundaries
    ↓
execution
```

Do NOT trust model claims like:

"this code looks secure."

Require machine-grounded evidence.

The AI auditor should consume:

- compiler output
- Authority
- Guard state
- sandbox state
- Atlas
- diagnostics
- tests
- fuzz results
- audit chain
- CI
- doctor
- deployment facts

---

## 21. FUTURE ALGORITHMS

Do not design DeluluLang around today's algorithms.

Future AI may invent algorithms and computational techniques we cannot currently anticipate.

Therefore keep:

### STABLE CORE

- types
- effects
- authority
- capabilities
- deterministic semantics
- audit
- sandbox
- resource control

### EXTENSIBLE OUTER LAYERS

- plugins
- standard library
- foreign interfaces
- device adapters
- runtime backends
- sandbox providers
- remote execution
- agent protocols
- new accelerators
- future hardware

Do not hard-code assumptions about what "AI" means.

---

## 22. PHYSICAL AI

Future physical AI must use the same principal/authority model.

Do not expose raw device access merely because the caller is a robot.

A future actuator path should conceptually be:

```text
program
 ↓
Cap[Actuator]
 ↓
Authority
 ↓
Guard
 ↓
Sandbox
 ↓
host adapter
 ↓
safety envelope
 ↓
device
```

Do not allow the agent to bypass the Delulu enforcement path.

---

## 23. SURVEY

KEEP SURVEY AS REPOSITORY TRUTH.

Survey answers:

"What is actually in this repository?"

It must remain:

- provenance based
- file/line based
- deterministic
- non-inferential
- generated
- stale when not regenerated

Do not turn Survey into host capability detection.

When V2 adds:

- sandbox modules
- VM modules
- policy modules
- guest runtime
- image builder
- launchers
- probes

Survey must map them.

Regenerate Survey after changes.

---

## 24. DOCTOR

Doctor answers:

"What can this host actually enforce?"

Extend it when architecture requires it.

It should eventually expose machine-readable facts about:

- sandbox backends
- available isolation level
- KVM
- OS security primitives
- network enforcement
- filesystem enforcement
- guest image
- broker
- identity separation
- resource controls
- actual security posture
- current active security profile
- current break-glass status
- whether any restriction has been intentionally relaxed

Never show a security guarantee that was not measured.

---

## 25. ATLAS

Atlas answers:

"What does this checked program mean and what does it connect to?"

V2 should move toward Atlas understanding:

```text
Program
 ↓
Authority
 ↓
Effects
 ↓
Capabilities
 ↓
Sandbox
 ↓
Resources
 ↓
Plugins
 ↓
Actors
 ↓
Devices
 ↓
Execution boundary
```

An AI auditor should be able to query this structurally.

Do not turn Atlas into Survey.

---

## 26. DOCUMENTATION STRATEGY

From V2 onward, avoid constantly updating dozens of historical documents.

Use the V2 docs as the active source.

Historical docs are moved into:

```text
docs/archive/v1/
```

and preserved.

Do not make archival documents part of normal V2 execution context unless needed.

This is intentionally designed to reduce token waste.

Only update:

- active V2 docs
- current normative docs when genuinely necessary
- current README/HANDOFF pointers
- generated Survey artifacts
- required compatibility/historical references

Do NOT rewrite every historical campaign document after every V2 change.

---

## 27. LOGGING

At the END of each phase record:

### COMPLETED

Exactly what was implemented.

### FILES

Files added/changed/moved.

### TESTS

Actual commands and actual results.

### SECURITY

Security implications.

### SURVEY

Freshness/integrity result.

### DOCTOR

Actual result.

### CI

Actual run/result.

### GIT

Commit + push.

### AGENTS

What Opus 5 did.

### PROBLEMS

What failed.

### DECISIONS

What changed architecturally.

### NEXT

Next phase.

Do not claim anything that was not actually executed.

---

## 28. GIT RULES

Push only to:

```text
origin
https://github.com/jessesuniljs1-collab/delululang-test.git
```

Never:

- create the final public repository
- push to the future final public repository
- rewrite pushed history
- force-push
- erase historical commits

Commit every completed phase.

Push every completed commit.

Keep local and testing remote synchronized.

---

## 29. VERIFICATION GATE

Before declaring any phase complete:

1. run focused tests
2. run broader required tests
3. regenerate Survey
4. run doctor
5. verify documentation links
6. verify no unintended files remain duplicated after moves
7. inspect git diff
8. commit
9. push
10. verify remote
11. record actual CI state
12. only then declare the phase complete

A documentation-only change still needs the appropriate Survey/docs gates.

A security change needs security-specific testing.

A sandbox change needs adversarial testing.

A change involving Authority/Guard needs direct regression tests for semantic invariants.

A restriction-toggle change needs explicit transition tests.

---

## 30. AGENT SAFETY

Opus 5 is allowed to do most of the implementation.

But it must NEVER:

- silently change Authority semantics
- silently change Guard semantics
- silently weaken sandbox guarantees
- silently change owner decisions
- fabricate test results
- fabricate CI results
- delete historical docs
- destroy audit evidence
- rewrite pushed history
- grant itself additional authority
- disable its own sandbox/security policy
- use break-glass mode without an authorized external decision

When uncertain about an architectural/security decision:

**ASK FABLE 5.1.**

---

## 31. PHASE PAUSE

After every phase:

STOP.

Wait 60 seconds for owner input.

If the owner replies:

stop automatic progression and follow the new instruction.

If no reply after 60 seconds:

continue to the next approved phase.

Never skip the phase-end logging/verification before continuing.

---

## 32. FIRST EXECUTION ACTION

Do not immediately start changing source.

FIRST perform:

# PHASE V2-0 — V2 Workspace + Documentation Migration

Objectives:

1. Create the V2 active documentation structure.
2. Move superseded/historical V1 planning/documentation into the archive.
3. Preserve paths where possible.
4. Create `V2_DOC_MOVE_MANIFEST.md`.
5. Create `V2_EXECUTION_LOG.md`.
6. Create `V2_PHASE_STATUS.md`.
7. Create `V2_DECISION_LOG.md`.
8. Create `V2_AGENT_LOG.md`.
9. Create the active V2 README.
10. Link the repository/HANDOFF to V2.
11. Remove unnecessary duplicated active planning files.
12. Do NOT delete historical material.
13. Do NOT touch source code unless required to preserve paths/imports/documentation correctness.
14. Run Survey.
15. Run doctor.
16. Run documentation gates.
17. Commit.
18. Push.
19. Log everything.

This first phase exists specifically to stop V2 development from wasting tokens maintaining V1 historical documents.

Then STOP for 60 seconds.

---

## 33. AFTER V2-0

Proceed into the approved implementation sequence.

Prioritize real functionality over cosmetic documentation.

Do not spend an entire phase rewriting prose while a corresponding source implementation remains missing.

Remember the important known gaps:

- plugin runtime path must actually work
- machine JSON must become reliable
- diagnostics must be agent-usable
- standard library must grow
- agent Skill/MCP/checked edits must become real
- sandbox enforcement must become real
- resource authority must become more explicit
- Linux/KVM microVM must become real
- distribution must become real
- AI-native usability must be measured
- physical AI integration must remain capability/authority controlled

---

# 34. ADDITIONAL V2 ARCHITECTURAL RECOMMENDATIONS

These are now part of the execution brief.

## A. Treat AI-native usability as measurable

Do not merely claim "AI-friendly."

Measure:

- time to first compiling program
- correction iterations
- token/context cost
- undocumented surprises
- diagnostic repair success
- agent task success
- authority-review effort
- sandbox-policy errors

Use `delulu-measure` and build a dedicated V2 benchmark.

## B. Build the zero-shot language-learning interface

The toolchain should become the teaching interface for models that have never seen DeluluLang.

The model should be able to discover the language from the installed toolchain rather than depending on pretraining.

## C. Make sandbox policy a first-class formal object

Do not reduce sandboxing to a CLI flag.

Work toward a machine-readable `SandboxPolicy` model containing concepts such as:

- requested level
- actual level
- backend
- filesystem
- network
- secrets
- devices
- resources
- identity
- channel
- guest
- guarantees
- limitations
- revocation state

The policy should eventually be explainable, hashable, auditable and diffable.

## D. Resource authority should become fundamental

CPU, memory, time, actor count, disk, network and device budgets should evolve from simple launcher controls toward formal, inspectable execution authority.

## E. Do not let sandbox semantics diverge by backend

All backends must preserve one Delulu Authority meaning.

The backend changes the enforcement boundary, not the language semantics.

## F. Keep natural language outside the language core

Humans should be able to describe goals naturally and AI can compile those goals into DeluluLang.

Do not put opaque LLM-evaluated semantics inside DeluluLang itself.

The compiler/runtime/security properties must remain deterministic and reproducible.

## G. Physical AI uses the same model

No "robot authority" or "AGI authority" as a special semantic category.

Use principal + identity + authority + capability + sandbox + resource budgets.

## H. Preserve evidence

Never trade evidence quality for speed.

If a feature cannot be proven yet, label it:

- implemented
- measured
- designed
- partially implemented
- unverified
- deferred

Do not collapse those states.

---

# 35. WHAT SUCCESS LOOKS LIKE

Do not optimize for "many commits".

Optimize for:

A developer or AI agent can say:

**"Build this."**

The AI generates DeluluLang.

The toolchain teaches itself to the AI if necessary.

The compiler explains:

**WHAT THE PROGRAM CAN DO.**

Authority explains:

**WHAT IT IS ALLOWED TO REQUEST.**

Guard explains:

**WHAT WAS APPROVED.**

Sandbox explains:

**WHAT THE EXECUTION ENVIRONMENT CAN PHYSICALLY REACH.**

Doctor explains:

**WHAT THE HOST CAN ACTUALLY ENFORCE.**

Atlas explains:

**WHAT THE PROGRAM IS MADE OF.**

Survey explains:

**WHAT THE REPOSITORY ACTUALLY CONTAINS.**

Tests/security tooling explains:

**WHAT WAS ACTUALLY VERIFIED.**

Audit explains:

**WHAT ACTUALLY HAPPENED.**

And none of those answers depend on whether the principal is:

- human
- AI
- LLM
- agent
- robot
- AGI
- ASI
- physical AI
- or a future computational system we haven't named yet.

That is V2.

Let's cook the actual thing now. 🔥
