# DeluluLang — Autonomous Development, Decision-Making and Release Preparation

You are responsible for taking DeluluLang toward its best possible long-term future as a programming language.

I am delegating decision-making authority to you. **You do not need to wait for my review or approval for any outstanding project decisions.** Review the repository, understand the project's goals and existing architecture, determine which choices are best for DeluluLang's long-term security, usability, maintainability, performance, developer experience and adoption, and implement those choices.

Do not merely provide recommendations or a list of questions for me to answer. Make the decisions yourself, execute the appropriate work, test it, and document the outcomes.

## 1. Establish the current project state

Start by reviewing the relevant repository records, including:

- `V2_PHASE_STATUS.md`
- `CLOUD_SYNC_LOG.md`
- `HANDOFF.md`
- `REMAINING_WORK.md`
- `V2_DECISION_LOG.md`
- `CONSTITUTION.md`
- All relevant specifications, release documentation, test results and security policies.

Verify the actual repository state instead of relying solely on previous summaries. Identify completed work, outstanding tasks, inconsistencies, unresolved decisions and anything that may prevent a reliable release.

Preserve existing functionality unless changing it is justified by a demonstrable improvement.

## 2. Continue development with P9-05

Proceed with P9-05, the remaining P9 task concerning network permissions limited by method and path.

1. Read its specification and identify the precise acceptance criteria.
2. Examine the existing implementation and relevant security boundaries.
3. Implement the change using the architecture and conventions that best serve the language.
4. Add or update tests, documentation and formal models where applicable.
5. Run the relevant tests, lint checks and validation commands.
6. Update the phase status and decision records with the actual results.

If you discover that the specification or architecture should be improved, make the best justified decision and document it. Do not stop simply because a new design question arises.

## 3. Resolve ALL outstanding decisions autonomously

Every unresolved decision in the repository records, handoff documents, remaining-work lists and previous discussions is delegated to you.

The following are starting recommendations, not mandatory answers. Examine the evidence, independently determine the best outcome for DeluluLang, and implement that outcome without waiting for me.

### Public repository and Git history

- Choose the safest, most maintainable approach to preparing the public repository and its Git history.
- Resolve the flagged banned word in commit `0a58451` according to the project's requirements.
- Prevent unintended personal email disclosure in published commit metadata. Use an appropriate verified GitHub noreply address for future public commits and sanitize historical metadata where necessary.
- Replace the `CODEOWNERS` placeholder with the correct, verified GitHub handle.
- Preserve the existing development repository, its history and a recoverable backup. Do not destroy the development history merely to create a clean public history.
- Inspect existing remotes, branches, tags and authentication before performing Git operations. Never overwrite an unrelated remote or destroy recoverable work.
- Choose whether a fresh public history, a rewritten history or another approach best satisfies the project's security and distribution goals.

### Licence and legal compliance

- Resolve the undecided licence by considering DeluluLang's intended openness, adoption, commercial use, contributions, patents, dependencies and redistribution.
- Evaluate MIT, Apache-2.0, their combination and any better-suited alternative. Choose and implement the most appropriate licence rather than asking me to pick one.
- Update `LICENSE`, `NOTICE` and related documentation consistently with the selected licence and applicable third-party obligations.
- Review the GPL Linux kernel included in the microVM image. Determine whether shipping the image is worthwhile and establish the required licence notices, corresponding source availability and other applicable compliance measures.
- Decide whether to ship the image or distribute reproducible build instructions instead. Do not distribute an image until its actual compliance requirements have been satisfied.

Do not treat this work as a substitute for qualified legal advice if a material legal uncertainty cannot be resolved confidently. In that situation, choose a conservative, compliant interim approach and document the unresolved legal risk without blocking unrelated development.

### Security policy and release infrastructure

- Complete `SECURITY.md` and the applicable `PENDING-PUBLIC` requirements.
- Establish an appropriate vulnerability disclosure channel.
- Evaluate branch protection, signed commits, SLSA Level 3 and OpenSSF Scorecard. Implement the controls appropriate to the project's threat model and release maturity.
- Configure the controls wherever your available permissions permit, and accurately document anything that requires unavailable credentials or external account access.
- Never invent security contact details, cryptographic keys, verification results or compliance claims.

### Architecture, constitution and design decisions

- Review `CONSTITUTION.md` §5.15 and correct any inaccurate claims about microVM containment or platform availability.
- Decide whether to adopt `rustfmt` and standardize formatting appropriately.
- Decide whether to add a `CODE_OF_CONDUCT.md` and implement a suitable policy if that supports the project's intended contributor community.
- Review the root-issuance gate and choose its safest appropriate default based on the security model, tests and intended developer experience.
- Resolve RW 4.19 / D23 concerning whether hardware-driver startup time counts toward the first command's two-second timeout. Choose a coherent, testable timing contract and align the implementation and documentation.
- Review P8-04 option (c), the revoke-only watcher in the grant tree, against the implemented option (b). Evaluate authority boundaries and revocation security, then choose the best design.
- Review delegated decisions D-V2-49, D-V2-59 and D-V2-110 through D-V2-137. Preserve sound decisions; revise any that no longer serve the project's goals, documenting evidence and compatibility implications.
- Resolve any other remaining decisions discovered during the audit, even if they are not explicitly listed here.

A decision being recorded as entrenched, previously delegated or already implemented is not, by itself, a reason to preserve it. Make changes when the evidence justifies them, with appropriate migration safeguards, tests and documentation.

### Release decision

- Keep the release process gated on objective readiness, not on my manual approval.
- Complete the required engineering, testing, repository, licensing, security and distribution checks.
- Determine the correct versioning and release strategy.
- Once the required release criteria have actually passed, you are authorized to proceed with the appropriate public release without waiting for another approval from me.
- Do not claim release readiness merely because CI is green. Confirm that the actual release artefacts, metadata, licence obligations and security claims are correct.
- Do not publish a microVM image unless its distribution and compliance requirements have been met.
- Preserve a verifiable release record and ensure the development repository remains recoverable.

## 4. Perform the remaining machine-level validation

Review the reported differences between GitHub, Windows and WSL. Where the necessary environment and permissions are available:

1. Rebuild the outdated WSL microVM images.
2. Run the full relevant test suite in WSL.
3. Run the VS Code end-to-end test.
4. Rerun any necessary CI checks after modifying shared code, build configuration, security boundaries or release workflows.
5. Investigate failures rather than suppressing them or weakening tests to obtain a green result.

Do not claim a check passed unless it actually ran and produced a successful result. If a tool, device, credential or environment is unavailable, continue with the work that is possible and document the exact blocker.

## 5. Review the disabled scheduled task

Inspect the existing scheduled task, its purpose, its stop-date condition of 16 October 2026, and its behavior of creating a new branch per run.

Independently decide whether continued automation benefits the project.

If it is useful, choose and implement an appropriate cadence, finite end condition and branch-management safeguards. If it is no longer worthwhile or cannot be made reliable, leave it disabled. Avoid unnecessary branch creation and unbounded automation.

Do not ask me to choose the schedule or decide whether it should continue.

## 6. Operating principles

Apply these principles throughout the project:

- **Autonomy:** Make and execute project decisions without asking me to approve individual choices.
- **Long-term thinking:** Optimize for DeluluLang as a future programming language, not merely for finishing the current task quickly.
- **Evidence-based decisions:** Inspect code, specifications, tests, history and applicable current standards before choosing.
- **Security first:** Prefer secure defaults, explicit trust boundaries, least privilege and verifiable guarantees.
- **Engineering quality:** Favor coherent architecture, maintainability, backward compatibility where appropriate, reproducible builds and clear diagnostics.
- **No shortcuts:** Do not remove meaningful tests, weaken security, hide failures or make unsupported claims just to complete a phase.
- **Transparent records:** Record significant decisions, alternatives considered, rationale, consequences and validation evidence in the appropriate Markdown files.
- **Safe operations:** Back up before risky history changes, inspect remote targets and never destroy the existing development repository.
- **Honest blockers:** Do not fabricate credentials, test results or external changes. When a genuine access limitation exists, complete everything possible without it and document what remains.

Do not interrupt development merely to present choices, ask questions or request approval. Choose the best available path, document your reasoning and continue. Escalate only when a real external dependency, unavailable credential or unavoidable safety or legal constraint makes execution impossible.

## 7. Final deliverables and reporting

When the work is complete, provide an evidence-based final report covering:

1. The verified current build status and completed versus remaining phases.
2. Each significant decision made, its rationale and the corresponding decision-record reference.
3. Every file, subsystem, workflow or configuration changed.
4. Tests, CI jobs, security checks, formal validations and local environment checks actually executed, including their results.
5. Public repository and release status, including the actual destination and version if published.
6. Licence and microVM distribution decisions and their compliance status.
7. Any remaining issues, external blockers or known limitations.
8. The next engineering priorities, ranked by importance.

Update the Markdown records as work progresses. Keep those records consistent with the actual implementation.

**Begin now. Do not stop after providing an assessment or proposed plan. Proceed through implementation, validation, autonomous decision-making and release preparation to the fullest extent possible.**
