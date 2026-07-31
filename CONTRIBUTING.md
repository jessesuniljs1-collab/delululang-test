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
  changing anything, `cargo run -p delulu-survey -- rdeps crate:<name>` will tell you what else
  touches it — with the line that proves each one.
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

## 4. AI contributions

DeluluLang is a language built for a world where much code is written by machines. Pretending our
own contributions are all hand-typed would be dishonest, and worse, it would mean the policy nobody
admits to needing is the one nobody reviews.

**The policy:**

1. **Disclosure is required.** If a change was substantially authored by an AI system, say so in the
   PR and name the system. The commit trailer carries it (`Co-Authored-By:`). This project's own
   history does exactly this, including recording which model wrote which stage.
2. **A named human sponsor is accountable** for every AI-authored PR. Accountable means: they read
   it, they understand it, and they answer for it in review. "The model wrote it" is not a defence
   of a bad change; it is a description of how the bad change arrived.
3. **No unsupervised autonomous PRs.** An agent may write a change. A human decides it is submitted.
4. **The quality bar does not move.** Same tests, same review, same standard of honesty in the
   prose. An AI-authored change that skips the skip-branch case is rejected exactly like a
   human-authored one.
5. **AI-submitted code runs only in sandboxed CI**, under the project's own Stage-5 isolation
   profiles. The project dogfoods its own containment for its own contributions. If our isolation is
   not good enough to run code we did not write, it is not good enough to ship.
6. **AI-overseer monitoring is advisory only.** Automated review may label a PR; it may never merge
   one. Constitution §9 is explicit that the guarantees must hold *even if every overseer colludes* —
   a system that depends on its watchers being honest has no guarantee at all.

### 4.1 Why the sponsor requirement is not a formality

An AI system can produce a change that is locally correct and globally wrong: it satisfies the test
it was shown while breaking an invariant nobody wrote down. The sponsor's job is the part a model
cannot do for itself — carrying the consequences.

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
cargo test --workspace                              # everything
cargo run -p delulu-conform -- --coverage           # the coverage law
cargo run -p delulu-conform -- --check-reference    # the reference must not be stale
cargo run -p delulu -- fmt --check examples         # one style, no options
cargo clippy --workspace --all-targets              # no new warnings
cargo run -p delulu-survey -- build                 # regenerate the map of the repository
```

**Regenerate the Survey with any change that moves the tree, and commit it alongside.** This is not
a courtesy — `cargo test --workspace` rebuilds the map and compares it to what is committed, so a
change that leaves `docs/survey/` behind fails the suite and names the first line that differs.
Nothing about it needs remembering; the command above is the whole remedy.

## 7. Conduct

Be honest, be precise, and assume the other person is trying to make the thing better. Report what
you found, including when it is inconvenient — this project's own history includes a measurement
study that produced a false 100% and a normative rule that was false until a benchmark caught it.
Both are written down. Yours can be too.
