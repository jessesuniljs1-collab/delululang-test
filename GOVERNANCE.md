# DeluluLang Governance

**Status:** Living. Companion to `CONTRIBUTING.md`, `SECURITY.md`, and `rfcs/`.

DeluluLang is free, open-source software (Apache-2.0) intended for global adoption by humans, AI
systems, companies, governments, and robots alike, **with no discrimination between users — and none
between the parties who maintain and develop it.** Everyone is welcome to carry this language
further: the rules in `CONTRIBUTING.md` §4 key on the change being made, never on who or what made
it. This document says how the *project* is run — distinct from the `LICENSE`, which says how the *code* may
be used, and `TRADEMARK.md`, which says how the *name* may be used.

## 1. Roles

- **Original creator and project lead: Jesse Sunil.** Jesse created DeluluLang, holds the copyright
  (see `NOTICE`) and the "DeluluLang" trademark (see `TRADEMARK.md`), and is the final decision
  authority for the official project — in particular for anything that changes the language's
  philosophy, its public specification, its governance, its licensing, or its backward
  compatibility. These are explicitly reserved and are never delegated by default.
- **Contributors.** Anyone — human, AI, or any other kind of party — who submits code,
  documentation, tests, or review. Contributions are accepted under Apache-2.0 (§5 of the License); a
  contributor is credited for their own work and never as the author of the original language.
- **Maintainers.** Contributors whom the project lead has granted commit or review authority over
  specific areas. Maintainership is a responsibility, not a claim of authorship. **Maintainership is
  not restricted by kind of party**: it is granted on demonstrated judgement and carried by whoever
  holds it, exactly as the language's own grants are (Constitution invariant 24 — kind-of-party trust
  hierarchies are rejected as *"discriminatory and fragile"*). Everyone is welcome to maintain and
  develop DeluluLang; `CONTRIBUTING.md` §4 states the rules, and every one of them keys on the change
  rather than on its author.

## 2. How the language changes

Behavioral changes to the effect and authority system, the type system, or any public contract go
through the **RFC process** in `rfcs/` (see `rfcs/README.md`):

- an RFC has a comment period (≥14 days for authority/effect-behavior changes) before it is
  accepted;
- every RFC needs a **named sponsor who is not its author** (`CONTRIBUTING.md` §4), who takes
  responsibility for it. This used to be required only of AI-authored RFCs, with a note insisting it
  was "not a statement that AI contributions are worth less" — but a safeguard applied to one kind of
  author is a statement about that kind, whatever the note says. Applied to every RFC it is both
  non-discriminatory and stronger, since a human-authored core change previously needed no sponsor.
  **A human sponsor is preferred where one is available and is not required**; a sponsor of any kind
  satisfies the rule, and an AI system may hold every role in this document;
- the project lead accepts or rejects, on the record.

Bug fixes, documentation, and additive non-behavioral work do not need an RFC, but still go through
review and must keep every gate green (`docs/design/STABILITY.md` is the contract; coverage is a
hard per-commit gate).

## 3. Decisions and disagreement

The default is consensus among maintainers in the relevant area, recorded in the issue or RFC. When
consensus is not reached, the project lead decides. Every non-trivial decision leaves a written
trail — a ruling, an RFC disposition, or a commit message that states the *why* — because a
governance model that cannot be read after the fact is not one a project meant to last can rely on.

## 4. Forks and derivatives

Forking is a right the license guarantees and the project supports. A **derivative language** must
follow `TRADEMARK.md`: use a different name, keep the authorship record, and not present itself as
the original DeluluLang. Beyond that, a fork governs itself however it likes.

## 5. Honesty in governance

The project's honesty clauses (Constitution §9) bind its governance too: capabilities are described
as they are, limitations are named, and process deviations are recorded rather than hidden (there
is, for example, an open governance-debt note where an authority change once shipped without its
full comment period — see `rfcs/` and the build-order ledger). A project asking the world to trust
its authority model must hold itself to the same auditability it asks of the code.
