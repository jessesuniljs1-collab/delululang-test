Fable 5.1, kitchen update. 🍳

I have NOT approved the previous Next Evolution 2026 implementation roadmap yet.

You already completed the first planning/reassessment pass and created:

D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\

Read these FIRST:

D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\MASTER_PLAN.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\RESEARCH.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\VERIFICATION_FINDINGS.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\IMPLEMENTATION_ROADMAP.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\DECISION_LOG.md
D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\EXECUTION_LOG.md

Also read the current README/HANDOFF and the relevant existing microVM/sandbox specifications and source.

IMPORTANT:

Do NOT start implementation.

Do NOT treat the previous roadmap as approved.

U can create a completely separate sandbox roadmap that ignores the existing plan. Well do whatever that is good. U can change any instructions given by me. Anything that makes delululang better and good, U can do it, No restrictions on u.

This is a NEW architectural research pass that must be integrated into the existing Next Evolution plan.

The previous plan currently treats the microVM layer as deferred. I now want you to seriously reassess that decision.

QUESTION:

Should sandboxing + VM isolation become:

1. an early implementation phase,
2. a later phase,
3. a cross-cutting architecture underneath runtime/plugins/agents/autonomy,
4. or some combination of these?

Do not assume the answer.

Research it and decide from evidence.

==================================================
SANDBOX + VM RESEARCH
==================================================

I want a MUCH better sandbox + VM system for DeluluLang users.

DeluluLang is intended for:

- AI agents
- LLMs
- autonomous systems
- physical AI
- robots
- future AI systems
- humans supervising those systems
- untrusted / hostile programs

Do NOT think of this as simply "run it in a container".

I want a proper execution-isolation architecture.

FIRST: SEARCH THE WEB YOURSELF

Search extensively before deciding anything.

Study:

AISI / Inspect:

https://www.aisi.gov.uk/blog/the-inspect-sandboxing-toolkit-scalable-and-secure-ai-agent-evaluations
https://www.mitre.org/focus-areas/artificial-intelligence/federal-ai-sandbox
https://www.gov.uk/ai-assurance-techniques/nayaones-ai-sandbox
https://inspect.aisi.org.uk/
https://github.com/UKGovernmentBEIS/inspect_ai.git
https://github.com/UKGovernmentBEIS/aisi-sandboxing.git
https://github.com/UKGovernmentBEIS/inspect_k8s_sandbox.git

AI Verify:

https://github.com/aiverify-foundation/aiverify.git

Cloudflare:

https://github.com/cloudflare/sandbox-sdk.git

Vercel:

https://github.com/vercel/sandbox.git

AWS Firecracker:

https://github.com/firecracker-microvm/firecracker
https://firecracker-microvm.github.io/

Google gVisor:

https://github.com/google/gvisor
https://gvisor.dev/

Kata Containers:

https://github.com/kata-containers/kata-containers
https://katacontainers.io/

Cloud Hypervisor:

https://github.com/cloud-hypervisor/cloud-hypervisor

AWS Nitro Enclaves:

https://aws.amazon.com/ec2/nitro/nitro-enclaves/
https://docs.aws.amazon.com/enclaves/latest/user/

Microsoft Azure Container Apps Dynamic Sessions:

https://learn.microsoft.com/en-us/azure/container-apps/sessions-usage
https://learn.microsoft.com/en-us/azure/container-apps/sessions-custom
https://learn.microsoft.com/en-us/azure/container-apps/ai-integration

Microsoft Foundry Code Interpreter:

https://learn.microsoft.com/en-us/azure/foundry/agents/how-to/tools/code-interpreter

Modal:

https://modal.com/blog/sandbox-launch
https://modal.com/solutions/coding-agents

Daytona:

https://www.daytona.io/docs/process-code-execution
https://www.daytona.io/docs/en/claude/

ALSO SEARCH FOR MORE CURRENT SYSTEMS

Find current sandbox / VM / isolated-execution systems used by:

- major cloud companies
- major AI companies
- governments
- AI safety organisations
- security companies
- coding-agent platforms
- large-scale multi-tenant systems

Do not stop at the links above.

==================================================
RESEARCH TOPICS
==================================================

Research:

- microVMs
- VM-backed containers
- application-kernel sandboxes
- KVM
- Hyper-V isolation
- containers
- gVisor
- seccomp
- namespaces
- Landlock
- AppArmor
- SELinux
- network namespaces
- egress filtering
- DNS isolation
- proxy-based networking
- filesystem isolation
- read-only filesystems
- immutable images
- ephemeral filesystems
- snapshots
- copy-on-write
- CPU limits
- memory limits
- disk limits
- PID/process limits
- device isolation
- GPU isolation
- secrets isolation
- credential brokering
- attestation
- confidential computing
- TEE / confidential VMs
- escape resistance
- destructive-agent protection
- multi-tenant isolation
- lifecycle management
- fast startup
- warm pools
- snapshot/restore
- checkpoint/rollback
- forensic logging
- audit trails
- escape testing
- adversarial sandbox evaluation

Do not reduce this to "containers vs VMs".

Study the complete architecture and trust boundaries.

==================================================
DELULULANG ARCHITECTURE
==================================================

The sandbox must fit the existing DeluluLang Authority + Guard model.

Do NOT invent a competing permissions model.

Think about:

Authority
↓
sandbox policy
↓
host isolation
↓
filesystem policy
↓
network policy
↓
process/device policy
↓
execution
↓
audit / telemetry

The sandbox should be an enforcement layer underneath DeluluLang's authority system.

Study how this interacts with:

- Authority
- Guard
- broker
- leases
- revocation
- effects
- capabilities
- plugins
- actors
- LSP
- CLI
- Atlas
- Survey
- doctor
- WASM
- deployment
- future physical AI

==================================================
SECURITY TIERS
==================================================

I want multiple isolation levels, but do NOT assume a fixed tier structure.

Evaluate possible layers such as:

- normal local execution
- restricted process sandbox
- hardened container
- application-kernel sandbox
- gVisor-style isolation
- microVM
- hardware-backed VM
- confidential environment

Determine which layers actually make sense for DeluluLang.

For each one, specify:

- security boundary
- attack surface
- startup cost
- performance cost
- platform availability
- what it protects against
- what it does NOT protect against
- required host capabilities
- suitability for AI-generated/untrusted code
- suitability for physical AI
- suitability for multi-tenant use

==================================================
USER EXPERIENCE
==================================================

Investigate what the CLI should eventually look like.

Possible examples:

delulu run app.delulu --sandbox

delulu run app.delulu --isolation microvm

delulu run app.delulu --sandbox-profile hostile-agent

or project configuration.

Do not blindly implement those commands.

Design the UX from the existing CLI architecture.

The machine interface must remain strong.

`--json` is important.

An agent should be able to discover:

- current sandbox backend
- isolation level
- filesystem access
- network access
- tools
- capabilities
- effects
- resource limits
- timeout/TTL
- what is denied
- why it is denied
- remaining resources
- host guarantees
- sandbox state

==================================================
SANDBOX PROVIDER MODEL
==================================================

Research whether DeluluLang should have a sandbox-provider abstraction.

Possible providers:

- local process
- container
- application-kernel sandbox
- gVisor
- microVM
- KVM VM
- Kubernetes
- remote sandbox
- cloud sandbox

Do NOT simply create an enum of provider names.

Design a backend abstraction that can survive future providers without weakening the security model.

Clearly separate:

policy
backend
host capability
runtime
authority
audit

==================================================
MICROVM
==================================================

The previous plan marked microVM as deferred.

Now reassess it from scratch.

Study Firecracker, Kata, Cloud Hypervisor, VMM designs and DeluluLang's existing microVM specifications/code.

Determine:

- VM contents
- guest kernel
- root filesystem
- image construction
- program delivery
- file injection/mounts
- networking
- default network posture
- broker communication
- authority acquisition
- authority revocation
- secrets
- stdout/stderr
- exit codes
- timeouts
- resource quotas
- snapshots
- restore
- cleanup
- crash handling
- guest death verification
- state isolation
- image integrity
- kernel/rootfs pinning
- update strategy
- escape detection
- forensic evidence

Also determine what must NOT be implemented yet because the evidence or host environment is insufficient.

==================================================
SECURITY / THREAT MODEL
==================================================

Assume sandboxed code may be:

- buggy
- malicious
- AI-generated
- intentionally trying to escape
- trying to exfiltrate secrets
- attacking the network
- attacking the host
- attacking another sandbox
- attacking the runtime
- abusing dependencies
- abusing plugins

Study real escape surfaces from the systems above.

Do not make blanket claims such as "VM = secure".

Document:

- security boundary
- trusted computing base
- assumptions
- known limitations
- host dependencies
- failure modes
- evidence required before stronger claims

Use DeluluLang's existing evidence discipline from MATHEMATICS.md.

==================================================
ADVERSARIAL TESTING
==================================================

Design a serious sandbox test program.

Prefer generated/adversarial inputs over only hand-written examples.

Cover:

- traversal
- symlink attacks
- hardlinks
- path confusion
- device access
- process escape
- namespace escape
- network escape
- DNS abuse
- localhost access
- metadata access
- credential leakage
- environment-variable leakage
- secret leakage
- sandbox-to-sandbox attacks
- host file access
- CPU exhaustion
- memory exhaustion
- disk exhaustion
- process exhaustion
- fork bombs
- timeout bypass
- zombie/background process persistence
- IPC
- sockets
- malformed images
- malicious plugins
- malicious dependencies
- VM lifecycle races
- cleanup races
- snapshot contamination
- restore contamination
- concurrent sandboxes
- realistic runtime/kernel attack surfaces

Also research existing sandbox escape benchmarks and adversarial AI evaluation frameworks.

==================================================
SURVEY / DOCTOR / ATLAS
==================================================

Integrate the architecture with:

- Survey
- doctor
- Atlas
- Authority
- Guard
- LSP
- CLI

Investigate what `delulu doctor` should eventually report, such as:

- available sandbox backends
- active isolation level
- KVM availability
- host capabilities
- filesystem guarantees
- network isolation status
- broker status
- sandbox image validity
- configuration weaknesses
- host/platform limitations

Do not add meaningless checks.

Atlas should eventually be able to relate:

program
authority
sandbox
capabilities
effects
resources

Survey should understand the repository changes.

Agents should be able to consume sandbox status through machine-readable output.

==================================================
DOCUMENTATION
==================================================

Create research/design documents under:

D:\nelan\DeluluLang\docs\NEXT_EVOLUTION_2026\

Use:

SANDBOX_RESEARCH.md
SANDBOX_ARCHITECTURE.md
SANDBOX_THREAT_MODEL.md
SANDBOX_IMPLEMENTATION_PLAN.md
SANDBOX_TEST_PLAN.md

But ALSO integrate the conclusions into the existing:

MASTER_PLAN.md
IMPLEMENTATION_ROADMAP.md
DECISION_LOG.md
EXECUTION_LOG.md

Do not create a competing second roadmap.

The sandbox documents should become supporting documents for the main plan.

==================================================
CRITICAL INTEGRATION QUESTION
==================================================

After completing the research, explicitly answer:

1. Does sandbox/VM change the previous roadmap?
2. Should microVM remain deferred?
3. Should sandboxing move earlier than P8?
4. Should it become a cross-cutting layer for P2/P4/P8?
5. Does real plugin loading depend on sandboxing?
6. Does agent execution/MCP/tool execution need the sandbox architecture first?
7. Does distribution need to account for sandbox backend availability?
8. Which sandbox features belong in the core language?
9. Which belong in the runtime?
10. Which belong in the host/launcher?
11. Which belong in deployment/cloud infrastructure?
12. Which parts require owner decisions?
13. Which parts can be implemented immediately?
14. Which parts require Linux/KVM/cloud infrastructure?
15. What is the minimum useful secure sandbox DeluluLang can ship before a full microVM?

I want an architecture decision, not just a literature review.

==================================================
PLAN FIRST
==================================================

Do NOT implement sandboxing yet.

First:

1. Read the existing Next Evolution plan.
2. Inspect the current sandbox/microVM code/specification.
3. Search the web extensively.
4. Compare the approaches.
5. Identify what DeluluLang already has.
6. Identify what is missing.
7. Identify architectural mistakes/limitations.
8. Design the sandbox architecture.
9. Design the provider/backend model.
10. Design the threat model.
11. Design the testing strategy.
12. Determine whether microVM should move earlier/later.
13. Update the main roadmap accordingly.
14. Record owner decisions.
15. Update the execution log.

THEN STOP.

I will review the integrated plan before implementation.

==================================================
SAME MAINTAINER RULES
==================================================

Keep all previous rules:

- use Survey
- use doctor
- use impact before major changes
- verify against actual code
- test real DeluluLang programs
- keep evidence
- don't hallucinate
- don't fabricate security claims
- don't waste agents
- don't waste credits
- Opus/Sonnet only for agents if necessary
- save agent findings
- push only to the testing remote
- never touch the future final public repo
- never rewrite pushed history
- one phase at a time after approval
- Survey regenerated after the final edit of a commit
- doctor after verification
- record actual CI results
- never claim something ran when it didn't

Most importantly:

The sandbox research is now part of the DeluluLang Next Evolution decision.

Do not just give me:

"here is a nice sandbox architecture."

Give me:

"here is what sandboxing changes about the entire DeluluLang roadmap, why, what should happen first, what should remain deferred, and how the sandbox fits into Authority + Guard + agents + plugins + physical AI."

Let's cook the proper execution layer. 🔥
