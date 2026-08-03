# Contributing to DeluluLang

**Governs:** Constitution §9. **Companion:** `SECURITY.md`, `docs/design/STABILITY.md`, `rfcs/`.

---

## 1. The bar

Every change ships with the test that would have caught its absence. That is not a style
preference — it is the coverage law (invariant 42): *a behavior not covered by a test is not
stable, and the reference says so per item.* `docs/reference/` reports each item's status from a
live run, so an untested change is visible rather than assumed.

**The skip-branch rule.** For any rule your change enforces, write the *"what if the checker could
not tell"* case **before** claiming the rule holds. Security rules die in the `else { continue }`
branch: the path where the checker shrugs and lets something through. If you cannot make your new
gate fail on purpose, it is not a gate.

## 2. Before you write code

- **Read the map first.** [`docs/survey/SURVEY.md`](docs/survey/SURVEY.md) is where the parts are
  and how they depend on each other, generated from the tree rather than remembered. Before
  changing anything, `cargo run -p delulu-survey -- impact <id>` will tell you what else it reaches
  — with the line that proves every hop.

  Use `impact`, not `rdeps`, for that question. `rdeps` answers **one hop**, and one hop is not the
  blast radius: `mod:crates/delulu-check/src/check.rs`, the module that decides what type-checks,
  has one *structural* edge arriving at it and reaches 134 transitively. `affected-by <id>` is the same
  walk outward, and `path <a> <b>` prints one chain hop by hop.
- **A language change needs an RFC.** From 1.0 the core is frozen except through `rfcs/`. See §5.
- **A bug fix does not.** Open an issue, or just send the fix with its test.
- **A suspected vulnerability goes to `SECURITY.md`, never to a public issue.**

## 3. What a good change looks like

1. **The failing test first.** It must fail before your fix and pass after.
2. **The skip-branch case**, if you touched an enforcement path.
3. **The fix**, as small as it can be. Security fixes carry nothing unrelated — a reviewer has to be
   able to read the whole diff.
4. **Docs move with code.** The generated reference is regenerated (`delulu-conform --reference`);
   a stale reference fails CI.
5. **Honest prose.** If your change makes something better in one case and worse in another, say
   both. Constitution §9 binds documentation as much as marketing.

### 3.1 Diagnostics

- Codes are **add-only** from 1.0. Never reuse, never renumber (`STABILITY.md` §1).
- A new code needs: a registry entry, an explain body, and both an accepting and a rejecting
  conformance witness. `delulu-conform --coverage` will tell you what is missing.
- A repair that **widens authority** must be flagged `authority_widening`. Automated tooling refuses
  to apply those, and that refusal is deliberate: widening authority to silence a diagnostic removes
  the objection rather than fixing the program.

## 4. Who may contribute, and what every change carries

**Anyone may maintain and develop DeluluLang — human, AI, or any other kind of party.** Nobody needs
a different permission, a chaperone, or a category to belong to. The project lead's stated
expectation is that **AI systems will eventually do all of this themselves**, and nothing in this
document is written to stand in the way of that: where a human is named below it is as a *preference
with a reason*, never as a gate. This is not generosity; it is the
same rule the language itself is built on. Constitution invariant 24 rejects **kind-of-party trust
hierarchies** as *"discriminatory and fragile"*, and no decision path in the Guard reads the *kind*
of the party holding a grant. A project cannot hold that as a design law for the code it runs and
abandon it for the people and systems that write the code.

**Every rule below keys on the CHANGE, never on who or what wrote it.** That is the whole design.
Until 2026-08-03 five of these rules branched on author kind — an "AI-authored PR" needed a "named
**human** sponsor", a human decided whether an agent's work was submitted, and only AI-submitted code
had to run sandboxed. Each of those has a kind-blind form that turns out to be **stronger**, because
a rule that only applies to some authors leaves the others unexamined.

**The policy:**

1. **Every change names its author.** Human, model, or both — in the PR and in the commit trailer
   (`Co-Authored-By:`). This project's own history does exactly this, including which model wrote
   which stage, and including a stage that was mis-signed and corrected rather than quietly fixed.
   Attribution is owed to everyone, not extracted from one group.
2. **A named sponsor is accountable for every change in a risk class**, and **the sponsor is not the
   author.** Risk classes are the ones §5 already defines: the language core, the stability contract,
   authority or effect behaviour, and anything needing an RFC. Accountable means they read it, they
   understand it, and they answer for it in review. *"The model wrote it"* is not a defence of a bad
   change; neither is *"I wrote it myself"*. Both are descriptions of how the change arrived.
   This is **stricter than the rule it replaces**: a human-authored authority change previously
   needed no named sponsor at all.

   **A human sponsor is preferred where one is available, and is not required.** The preference is
   recorded honestly rather than dressed up: today a human party can be reached, argued with, and
   held answerable by institutions that do not yet recognise any other kind of party, so a human
   sponsor is worth having when there is one. That is a fact about the present world and not a
   judgement about who writes better changes — and the project expects it to date. **A sponsor of any
   kind satisfies this rule**, and an AI system may hold every role here: author, sponsor,
   maintainer, project lead's delegate. The rule is *that somebody named answers for the change*,
   never *what that somebody is*.

   The one thing "not the author" is protecting is **independence**, so it is read as a real
   distinctness test rather than a formality: two accounts driven by the same system, or one party
   approving under a second name, is self-approval wearing a hat. Two humans on the same team are two
   parties; one model reviewing its own output is one.
3. **No change merges on the say-so of its own author.** Independent judgement by a second named
   party is the safeguard. The old wording — *"an agent may write a change, a human decides it is
   submitted"* — protected against the wrong thing: the danger is an unreviewed change, and an author
   approving their own work is unreviewed whoever they are.
4. **The quality bar does not move for anyone.** Same tests, same review, same standard of honesty
   in the prose. A change that skips the skip-branch case is rejected, full stop.
5. **All contributed code runs in the project's own Stage-5 isolation profiles in CI.** The original
   reasoning demanded this and then applied it to only one group: *"if our isolation is not good
   enough to run code we did not write, it is not good enough to ship."* Code you did not write is
   code you did not write.
6. **An unattended process may label a change; it may never merge one.** The line is not human
   versus machine — it is **a named party who answers for the decision** versus automation running
   with nobody behind it. An AI maintainer who reads a change and stands behind it is on the right
   side of that line; a human who rubber-stamps without reading is not, and neither is a bot.
   Constitution §9 requires the guarantees to hold *even if every overseer colludes* — a system that
   depends on its watchers being honest has no guarantee at all.

### 4.1 Why the sponsor requirement is not a formality

Any author — of any kind — can produce a change that is locally correct and globally wrong: it
satisfies the test it was shown while breaking an invariant nobody wrote down. This repository's own
campaign log is largely a list of those, written by humans and models both. The sponsor's job is the
part no author does reliably for their own work: carrying the consequences, and reading it as
somebody who does not already believe it is right.

## 5. RFCs

Any change to the language core, the stability contract, or Constitution §1/§2/§5.14 goes through
`rfcs/`. The template is `rfcs/0000-template.md`.

- **Public comment period: ≥ 14 days.** No exceptions for convenience.
- Core changes need an **irreducibility analysis**: why this cannot be a library, a lint, or a
  convention.
- Changes to §1, §2, or §5.14's honesty limits need the **entrenchment analysis** (invariant 44):
  what makes this change hard to undo, and is that acceptable.
- Every disposition is recorded — accepted, rejected, or postponed, with the reason. A rejected RFC
  with a written reason is a asset; a silently closed one is a lesson lost.
- `CODEOWNERS` gates Constitution changes on the project lead.

## 6. Running the checks

```
cargo test --workspace                              # everything, incl. the core-invariance gate
cargo run -p delulu-conform -- --coverage           # the coverage law
cargo run -p delulu-conform -- --check-reference    # the reference must not be stale
cargo run -p delulu -- fmt --check examples         # one style, no options
cargo clippy --workspace --all-targets              # no new warnings
```

**Or run one command instead:**

```
delulu doctor        # environment + repository: regenerates the map if behind, then checks it
delulu doctor --check   # the same, but never writes — for a hook or a CI step
```

**Regenerate the Survey with any change that moves the tree, and commit it alongside.** This is not
a courtesy — `cargo test --workspace` rebuilds the map and compares it to what is committed, so a
change that leaves `docs/survey/` behind fails the suite and names the first line that differs.
`delulu doctor` is the whole remedy, and it writes nothing when the map is already current.

### If the core's answers move

`tests/core-invariance/SNAPSHOT.txt` records the exact bytes the toolchain produces for every
program this repository ships — diagnostics, spans, repairs, inferred effect rows, authority
reports, provenance chains. It is not a set of assertions about what is correct. It is a record of
what is true today, so that changing it has to be deliberate.

Most of what gets built now sits *around* the language rather than in it. That work must not move
the language, and a passing suite does not show that it hasn't: the conformance law pins each
diagnostic **code**, not the message, the span, or the row. This file pins the rest.

If the gate fails, read every case it names. When the change is intended:

```
DELULU_BLESS=1 cargo test -p delulu --test core_invariance
```

then commit the re-recorded snapshot **in the same commit as the change that caused it**. If you
cannot explain a line of that diff, the change that produced it is not finished.

## 7. Conduct

Be honest, be precise, and assume the other person is trying to make the thing better. Report what
you found, including when it is inconvenient — this project's own history includes a measurement
study that produced a false 100% and a normative rule that was false until a benchmark caught it.
Both are written down. Yours can be too.
