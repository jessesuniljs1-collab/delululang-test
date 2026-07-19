# RFC 0000 — <title>

- **Status:** draft | comment | accepted | rejected | postponed
- **Author(s):**
- **Sponsor:** *(required for AI-authored RFCs — a named human who is accountable, per `CONTRIBUTING.md` §4)*
- **Opened:** YYYY-MM-DD
- **Comment period closes:** YYYY-MM-DD *(≥ 14 days after opening; no exceptions for convenience)*
- **Touches Constitution §1 / §2 / §5.14 honesty limits:** yes | no *(if yes, §6 is mandatory)*

---

## 1. Summary

One paragraph. What changes, for whom.

## 2. Motivation

What problem does this solve? Who has it? What do they do today instead, and why is that not good
enough?

Prefer evidence to intuition. A measurement, a real program that cannot be written, an issue with
users on it — any of these beats "it would be nicer".

## 3. Irreducibility analysis *(required for core changes)*

Constitution §2: a feature belongs in the language core only if it **cannot** be a library, a lint,
or a convention. Answer each, and answer honestly — most proposals die here, and that is the process
working:

- **Could this be a library?** If not, what specifically prevents it?
- **Could this be a lint or a checker rule?** If not, why must it change the language?
- **Could this be a convention plus documentation?** If not, what breaks?
- **What does the language lose** by absorbing this — in surface area, in things a reader must know,
  in things a checker must reason about?

## 4. Design

The proposal in detail. Grammar, typing rules, effect/authority behaviour, diagnostics with codes,
and how it interacts with the audit rules (R-1…R-7).

Every new diagnostic needs: a code, an explain body, and **both** an accepting and a rejecting
conformance witness. `delulu-conform --coverage` will not let it ship otherwise.

### 4.1 The skip branch

**What happens when the checker cannot tell?** Every enforcement rule needs an answer, and the
answer must be *refuse*. Describe the case here, and the test that pins it. This project's own
history — a `DL0803` fail-open, an arity gate that never fired, a signature check that returned
success for an unsigned artifact — is three reminders that this section is the load-bearing one.

## 5. Drawbacks

What gets worse. Every real change makes something worse; an RFC that lists none has not been
thought about hard enough.

## 6. Entrenchment analysis *(required if this touches Constitution §1, §2, or §5.14)*

Invariant 44. Constitution §10.

- **How hard is this to undo** once shipped? Who would be broken by reversing it?
- **What does it foreclose?** Which future designs become unreachable?
- **Is that acceptable**, and why? Entrenchment is not automatically wrong — the effect system is
  deeply entrenched and that is the point. It must be *chosen*, not stumbled into.

## 7. Rejected alternatives

What else was considered, and why it lost. Be specific: "we considered X" is worthless without the
reason X was worse.

## 8. Unresolved questions

What is still open. An RFC may be accepted with open questions; it may not be accepted with hidden
ones.

## 9. Impact on the stability contract

- Is this **additive** (minor) or **breaking** (major)? See `docs/design/STABILITY.md` §5.
- Does it move the language edition?
- Does it deprecate anything? If so, name the replacement, or state that migration needs judgement.

---

## Disposition

*Filled in when the comment period closes. Every RFC gets one — accepted, rejected, or postponed,
with the reason. A rejected RFC with a written reason is an asset; a silently closed one is a lesson
lost.*
