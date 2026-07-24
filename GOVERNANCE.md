# DeluluLang Governance

**Status:** Living. Companion to `CONTRIBUTING.md`, `SECURITY.md`, and `rfcs/`.

DeluluLang is free, open-source software (Apache-2.0) intended for global adoption by humans, AI
systems, companies, governments, and robots alike, with no discrimination between users. This
document says how the *project* is run — distinct from the `LICENSE`, which says how the *code* may
be used, and `TRADEMARK.md`, which says how the *name* may be used.

## 1. Roles

- **Original creator and project lead: Jesse Sunil.** Jesse created DeluluLang, holds the copyright
  (see `NOTICE`) and the "DeluluLang" trademark (see `TRADEMARK.md`), and is the final decision
  authority for the official project — in particular for anything that changes the language's
  philosophy, its public specification, its governance, its licensing, or its backward
  compatibility. These are explicitly reserved and are never delegated by default.
- **Contributors.** Anyone — human or AI — who submits code, documentation, tests, or review.
  Contributions are accepted under Apache-2.0 (§5 of the License); a contributor is credited for
  their own work and never as the author of the original language.
- **Maintainers.** Contributors whom the project lead has granted commit or review authority over
  specific areas. Maintainership is a responsibility, not a claim of authorship.

## 2. How the language changes

Behavioral changes to the effect and authority system, the type system, or any public contract go
through the **RFC process** in `rfcs/` (see `rfcs/README.md`):

- an RFC has a comment period (≥14 days for authority/effect-behavior changes) before it is
  accepted;
- an AI-authored RFC needs a **named human sponsor** (`CONTRIBUTING.md` §4), who takes
  responsibility for it — this is a governance safeguard, not a statement that AI contributions are
  worth less;
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
