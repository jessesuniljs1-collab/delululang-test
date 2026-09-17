# Next Evolution 2026 — the plan folder

**Status: PLAN COMPLETE, AWAITING THE OWNER'S APPROVAL. No implementation has started.**

This folder is the durable record of the 2026 evolution commission: a reassessment of DeluluLang
against its original AI-first goal, web research on what the field does, hands-on verification of
the real binary, a phased roadmap, and the decision record behind it. It was written on 2026-09-17
by Claude Fable 5.1 acting as head chef, and it is meant to be read by whoever picks the work up —
human or AI — without needing this session's memory.

| File | Read it when |
|---|---|
| [`MASTER_PLAN.md`](MASTER_PLAN.md) | you want the whole picture: what DeluluLang is today (verified), where it drifted, the AI-first goal as engineering, gaps, what to build and not build, phases, effort, owner decisions |
| [`VERIFICATION_FINDINGS.md`](VERIFICATION_FINDINGS.md) | you want the evidence: sixteen findings (`NE-01…16`) from driving the real compiler, terminal, language server and broker, plus the twenty-two claims that held |
| [`RESEARCH.md`](RESEARCH.md) | you want to know what ZeroLang, Mojo, the agent frameworks, the code-intelligence and memory tools, the capability languages and the release tooling do — and what of it belongs here |
| [`IMPLEMENTATION_ROADMAP.md`](IMPLEMENTATION_ROADMAP.md) | you are about to do the work: phases P1–P8, tasks with blast radius, verification and acceptance, dependencies, what needs the owner |
| [`DECISION_LOG.md`](DECISION_LOG.md) | you want to argue with a choice: evidence, assumptions, alternatives, why, confidence, what still needs verifying |
| [`DOCUMENTATION_AUDIT.md`](DOCUMENTATION_AUDIT.md) | you want to know which of the 156 markdown files is current, normative, historical, generated, superseded or entrenched — and the proposed moves (none executed) |
| [`EXECUTION_LOG.md`](EXECUTION_LOG.md) | you want to know exactly what this pass inspected, ran, found, created, committed and pushed |
| [`OWNER_COMMISSION.md`](OWNER_COMMISSION.md) | a pointer to the two commission texts, which live in `docs/design` |
| [`SANDBOX_RESEARCH.md`](SANDBOX_RESEARCH.md) | the second pass (2026-09-17, afternoon): what the field runs untrusted code in, the isolation primitives, the escape record, and what the CI runners measurably offer |
| [`SANDBOX_ARCHITECTURE.md`](SANDBOX_ARCHITECTURE.md) | the execution-isolation layer under Delulu Authority: the guest performs no effects, the provider model, the levels L0–L4, the UX and machine interface, the microVM designed from scratch, what must not be built yet |
| [`SANDBOX_THREAT_MODEL.md`](SANDBOX_THREAT_MODEL.md) | boundaries and trusted computing base per level, the claims and the evidence category each owes, assumptions, failure modes designed to fail closed |
| [`SANDBOX_TEST_PLAN.md`](SANDBOX_TEST_PLAN.md) | characterization tests runnable today, channel fuzzing, policy purity, generated boundary scenarios by family, mutant launchers, measurement |
| [`SANDBOX_IMPLEMENTATION_PLAN.md`](SANDBOX_IMPLEMENTATION_PLAN.md) | phases PS-0…PS-D with tasks, blast radius, verification, effort and owner decisions — a supporting plan of `IMPLEMENTATION_ROADMAP.md`, not a second roadmap |
| `agent-notes/` | the two sous-chefs' records from the sandbox pass (an Opus 5 red team of the containment code; a Sonnet 5 host-capability fact sheet), kept as written — the Sonnet file carries one marked head-chef annotation (five upstream link texts prefixed with their repository so the Survey does not read them as paths here); every claim taken from them into the plan was re-verified by the head chef |

**The sandbox pass's decision in one line:** sandboxing is a cross-cutting execution layer
introduced early (its truth-and-probe half inside P1, the L1 process jail right after), the
microVM is re-sequenced from "deferred" to a Linux/KVM phase, and the fifteen integration
questions are answered in `MASTER_PLAN.md` §12.2.

**How the work proceeds after approval** (owner's instruction): one phase at a time; at the end of
each phase — save, update the execution log and the decision log, run the phase's verification,
regenerate the Survey, run `delulu doctor`, commit, push to the testing remote only, record the
result, stop and report. A newer owner instruction always beats older scheduled work.

**Rules this folder observes:** the research sources are named here and nowhere on a product
surface; the Authority and Guard semantics are protected — hardened, never redefined; nothing is
claimed executed that was not; nothing is deleted.
