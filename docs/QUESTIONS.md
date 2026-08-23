# Hard questions about DeluluLang, answered

**Status:** Living document. Last revised **2026-08-03**, after the P16 adversarial pass.

This page exists because someone asked the questions a buyer should ask and refused to accept
adjectives as answers. It follows one rule, which is the same rule the rest of this project follows:

> **Every claim here either cites evidence you can run yourself, or says plainly that it has none.**

Where a question has an uncomfortable answer, the uncomfortable answer is the one written down. Two
of the answers below say *no*. One says *we were wrong until three days ago and here is the proof*.

Audience: humans and AI systems evaluating whether to build on this. Nothing here is written for
one and not the other.

---

## Part 1 — Can it be broken?

### 1.1 Can an AI agent bypass Delulu Authority?

**Under the stated threat model, no route was found in this campaign that a program itself can take
— but "no route was found" is not "no route exists", and one route existed for four months until
2026-08-03.**

That route is worth describing precisely, because it is the strongest evidence on this page and it
is evidence *against* the system:

```delulu
fn go[T](xs: List[Int], f: T) -> Int { let ys = xs.map(f)  1 }
fn main(root: Root) { let out = root.console()
  let n = go([1], fn(x: Int) -> Int ! {Write} { out.println("EFFECT ESCAPED"); x }) }
```

Nine lines. No plugin, no foreign call, no unsafe corner. `delulu check` said **clean**;
`delulu authority` said **"effects: (none — provably pure)"**; `delulu why Write` said **"program
cannot perform `Write`"**; and it printed at run time. A near-identical version leaked a **plaintext
secret** from a function the report called pure.

The cause was one missing `else`. The enforcement site for the builtin-callback law read
"*if the argument is syntactically a function type, add its effect row*" — and dropped the row
silently when it was not, which happens whenever the callback arrives through a bare type parameter.
A dropped row is an empty row.

It is fixed (campaign finding C88): the checker now **refuses** what it cannot determine rather than
assuming purity. The fix produced zero false positives across the entire conformance corpus.

What you should take from it is not "it is fixed" but **the shape**: a security rule can be correct
and still fail in the branch that runs when the checker cannot tell. That class is why the honest
answer to "can it be bypassed" is *we keep looking, and we publish what we find.*

**What bounds the damage when the type system is wrong** — and this is the part that worked:

- **The runtime custody gate is independent of the row.** Even with a lying row, a program can only
  use capabilities the operator actually granted. The escaped effects above were limited to
  capabilities already handed over.
- **`--assert-trace` caught it in the project's own terms**, reporting `DL1101 … not in the
  statically computed row — this is a compiler-bug class failure`. A backstop designed for exactly
  this fired when the layer above it failed. That is the entire argument for defense in depth, and
  it is the reason the layers exist.

### 1.2 Can hackers or rogue AI agents take control of Delulu Authority and Guard?

Split into the three ways someone would try.

**By writing a malicious program: no route found.** A program holds zero ambient authority. It
cannot forge a capability (no constructor, opaque type), cannot widen a grant (attenuation is the
only way to create a child), and cannot exceed its row without the checker refusing — subject to
1.1's caveat that the checker is software.

**By tampering with artifacts: refused, and this was attacked directly.** A certificate signature
must verify *and* be made by the key the certificate names. Certificate adoption is single-shot, so
a replayed credential cannot undo a revocation. Lease-token MACs cover the entire canonical payload.
A plugin with a present-but-invalid signature is refused even when signature checking is off.

**By already being on the machine as you: yes — and this is the honest weak point.** See 1.5. If an
attacker runs as your OS user, they can read the broker's key file and mint any token they like.
DeluluLang does not defend against a peer process running as you, and its source says so.

### 1.3 How can you say for sure that Delulu Authority and Guard are unbreakable?

**We cannot, we do not, and this project treats the word "unbreakable" as forbidden.** That is not
modesty — it is a written rule (`CONSTITUTION.md` §5.14) and `README.md` states it in the same words.

The reason is structural. Every guarantee rests on a **trust base** that is named rather than
hidden:

| You must trust | Because |
|---|---|
| **The compiler** | It decides which programs are well-typed. §1.1 is what it looks like when this is wrong. |
| **The runtime and the primitive table** | A wrong row in the table is a wrong answer everywhere. |
| **The host OS and hardware** | Capability enforcement is host-side code; a kernel that lies makes it moot. |
| **Cryptographic primitives** | BLAKE3, Ed25519 as used. Post-quantum options are gated behind `--unstable` and are **not** validated against NIST vectors here. |
| **The hypervisor**, if you use microVM containment | Standard containment assumption — **and today it is hypothetical**, because the microVM profile is a probe that always refuses (`DL1408`). You cannot currently use it, so this row describes a trust assumption you cannot yet take on. See `REMAINING_WORK.md` §4.1. |
| **Side channels** | Timing, cache and power channels are **out of scope**. `Secret.verify` is constant-time; nothing else claims to be. |

Anything that can be stated honestly is stated **relative to that base**. The correct sentence is
not "DeluluLang is unbreakable." It is:

> *Given a correct compiler and runtime, a program cannot perform an effect outside its declared row,
> and cannot obtain authority beyond what was granted to it.*

The first clause is the load-bearing one, and this campaign showed it can be false. What makes the
claim worth anything is that the project **looks for counterexamples and publishes them**, and that
there are independent layers underneath so one failure is not total.

**Both clauses have now been falsified, on separate occasions, and both are published.** P16 broke the
first: `delulu authority` reported "provably pure" for a program that printed at run time. On
2026-08-10 the *second* went — a program wrote a file **outside its granted scope**, through a dangling
symlink the containment resolver could not settle (`SYMLINK-DANGLE-1`, §1.7). Both were found by
looking, both were witnessed against the pre-fix binary, and both are fixed with a test that fails
without the fix. That is the only warranty on offer here: not that the sentence above has never been
false, but that when it is, this project finds out and says so.

### 1.4 Is any of this backed by real mathematics?

**Yes, and less than you might hope. Both halves matter.**

**What is genuinely mathematical:**

- **Effect rows are sets, and the typing rules compose them.** A function's row must contain the
  union of the rows of everything it calls (`ε_body ⊆ ε_declared`). The whole-program answer is
  `row(main)`.
- **Authority is ordered, and `⊓` is a genuine greatest lower bound.** `G₁ ⊑ G₂` is one conjunction
  over nine dimensions — effects plus eight scope dimensions — where each must be narrower-or-equal
  (paths are descendants, host sets are subsets, numeric envelopes are ≤). That `⊓` is **never wider
  than either input**, and is in fact the *greatest* lower bound, is now verified **exhaustively over
  every subset pair** of a path universe rather than spot-checked
  (`crates/delulu-broker/tests/order_laws.rs`), and reflexivity and transitivity are additionally
  **proved in Z3** over an abstract partial order, so they are not artifacts of the chosen universe.

  **Two claims this page previously made here were false, and were disproved by those same tests:**
  `⊑` is a **preorder, not a partial order** — `./data` and `data` resolve to one path but compare
  unequal structurally, so they attenuate each other while being distinct (206 counterexamples) — and
  **`⊓` is not symmetric**: `A⊓B = {./data}` where `B⊓A = {data}`, because the first argument's
  spelling wins when both directions hold (414 counterexamples). Neither is an authority escalation;
  the no-widening law is untouched. Both are recorded, with a third consequence for audit-chain
  hashing, in `docs/design/PROOF_CAMPAIGN.md` §F1–F3.

  **All three are now FIXED (2026-08-06), and the precise statement matters.** On **raw spellings**
  `⊑` is *still* a preorder and always will be — it is defined through a non-injective resolution,
  and that is arithmetic, not a bug anyone can patch out of the comparison. What changed is that the
  broker now stores exactly one **canonical representative** per equivalence class, so the quotient
  and the representation coincide and `⊑` is a genuine partial order over everything the system can
  build. `⊓` is symmetric because the meet emits representatives, and equivalent authorities hash
  identically because they are now literally equal. The three `#[ignore]`d failing tests are
  un-ignored and passing; a companion test asserts the raw-spelling counterexamples are *still
  there*, so the reason canonicalization is required cannot quietly become folklore.

  The canonical form is an **antichain** of canonical spellings, not merely a normalized spelling:
  `{./data, ./data/sub}` and `{./data}` are also mutually `⊑`, because the first path already
  contains the second. That second collapse was **not predicted** by the Z3 localisation — the model
  abstracts a dimension as a set over an opaque element type, so it cannot express one element
  subsuming another — and it was found by the exhaustive enumerator instead.
- **Attenuation is monotone by construction, not by check.** `sub ⊑ parent` holds because
  attenuation is the *only* way a child node can be created. There is no path where a child exceeds
  a parent, because there is no other path.
- **Unification is by equality, not subsumption.** Rows unify by set equality with row-variable
  binding, and subsumption exists at exactly two sites. This is deliberately more restrictive than
  necessary: covariant treatment of a parameter-position row is a total soundness failure, and
  equality-binding is sound in all positions regardless of variance.
- **The soundness argument itself** is written out as an induction over evaluation in
  `docs/design/SOUNDNESS_AUDIT.md` §D, under explicitly listed assumptions.

**What is NOT true, and is the honest limit:**

- **The FULL type system has no mechanized proof.** "Delulu Core" as a whole — §1–§7 of
  `docs/design/DELULU_CORE.md`, checked by a proof assistant — has been promised since Stage 2 and
  **still does not exist**: no capabilities, no store, no secrets, no attenuation, no Progress and no
  Preservation. One *fragment* of it is machine checked, and it is listed below with the rest of the
  machine-established evidence rather than counted here.
- **The hand argument was wrong in practice.** Not invalid as an argument — its step "the caller's
  row contains the callee's row" assumes the checker *has* the callee's row, and at higher-order
  builtins it silently did not (§1.1). A hand proof of the rules cannot tell you which branch the
  implementation forgot to write.

**What became machine-established from 2026-08-03 onward** — narrower than "proved", but no longer
just tests:

- **The custody broker is model-checked, in two parts** (`docs/design/models/`). The **grant tree**:
  TLA+/TLC explores **585,771 distinct states** of grant / delegate / revoke / expire with no
  violation of attenuation, revoke-covers-subtree, no-resurrection or inherited expiry. **Leases and
  certificate adoption**: 2,421 distinct states of delegate → mint → redeem, single-use nonces, key
  rotation and adoption, with no violation.
- **Those models are shown to have teeth rather than asserted to — three times.** Removing a fix and
  re-running makes TLC reconstruct the corresponding **real historical bug**: a `ttl: None` child
  outliving its parent's expired lease (depth 4); a certificate re-presented after a revocation,
  restoring killed authority (depth 4); and a token redeemed successfully against a revoked grant,
  writing `decision: "allow"` into the audit chain (depth 5). None was described to the model — each
  was reconstructed from the code's guards. **A model that has never caught anything is
  indistinguishable from one that cannot.**
- **Order-theoretic laws are checked exhaustively and symbolically**, as described above.
- **The higher-order fragment of the effect calculus is MACHINE CHECKED, in Lean 4.32.2**
  (`docs/design/models/lean/DeluluCore.lean`, since 2026-08-04). Two theorems: `good_sound` — with
  the corrected rule, a callback's latent row surfaces into the caller and every emitted label is in
  the declared row; and `bad_unsound` — with the rule `DELULU_CORE.md` actually states, there
  **exists** a well-typed program whose trace escapes its row. The second is **C88 mechanized**: the
  worst soundness hole this project has had, turned into a theorem. `#print axioms` reports all
  three theorems *"does not depend on any axioms"* — not even `propext` or `Classical.choice`.

**Those are three different kinds of evidence, and this page will not blur them.** Model checking
explores a **bounded** state space; Z3 discharges obligations over an **abstraction** (and an
abstraction can only be as strong as the properties it can state — the antichain collapse above is
exactly what it could not see); a Lean development is a **checked derivation**. Only the last is
"machine checked" in the sense `docs/MATHEMATICS.md` §12 uses, and it covers **one fragment of the
calculus**, not the type system.

So: **the design is mathematical; the type system's guarantee is still the tests; and three
subsystems now carry machine evidence of three different strengths, each bounded and labelled.** The
conformance suite remains a hard per-commit gate at 100% coverage, with an accepting and a rejecting
witness per rule. The mechanization of the **full** type system does not exist, and until it does,
saying anything stronger would be a lie of exactly the kind this project is built to avoid.

### 1.5 Can several agents work on one machine with different authority from one main user?

**Not safely on one OS account. This is the answer that will disappoint most, and it is the honest
one.**

Two halves, because they differ completely.

**Embedded mode — plain `delulu run --grant …` — holds.** Two programs running concurrently under
different filesystem grants could not reach each other's data, and the refusal was **DeluluLang's
own, not the operating system's**: both directories had identical same-user permissions, and an
ordinary shell could read either file freely. The capability check was the only thing between them,
and it held.

**The broker daemon with a shared state directory does not hold.** Its own source says why:

> *"The broker serves exactly one OS user; peer creds only confirm 'same user,' never multi-tenant
> auth (out of scope, spec §10)."* — `crates/delulu/src/broker_transport.rs:5`

Observed consequences, not theorised ones:

- `delulu grants list` discloses the **entire** grant tree — ids, scopes, holders — with no credential.
- `delulu grants revoke <id>` authorizes on *knowing the id*, which the previous line hands out. Any
  co-tenant can destroy any other's grant.
- `<state-dir>/broker.key` has ordinary same-user permissions and is the root of trust for lease
  MACs. Reading it is enough to mint a valid token for **any** node offline, including your root.

**The condition that actually makes multi-agent work: separate OS accounts, containers, or VMs —
one per agent.** Separate *directories* under one account is not sufficient and must not be sold as
if it were: the state-dir path is visible in an ordinary process listing, and so is any `--lease`
token passed on a command line.

This is not a defect against the model the project wrote down — that model is one holder per host.
It is a gap against a question the project had not answered. Genuine multi-tenancy needs
authenticated sessions, per-node ownership, and key material a co-tenant cannot read. That is an RFC,
not a patch, and it is recorded as open rather than quietly invented.

**The sharper consequence for the Guard, made explicit (DISC-1, 2026-08-08).** The Guard gates
*delegated* grants; **root** nodes pass without guard interaction. And root issuance is *ungated* — a
same-user process can call the broker's `Issue` path (via `grants delegate`'s auto-root or `run
--grant`) and mint a fresh root holding any authority, **headlessly, with no owner code**. Proven: an
operator can `guard policy set device:sat0/hga sealed`, and a same-user agent then mints an ungated
root holding `{Actuate}` on that exact device and commands it — the seal does not contain it. So **the
Guard provides no protection against an agent that shares your OS account.** "I sealed the device so
the agent can't command it" is *false* on one account. The Constitution (§5.16 law 4) and Stage-5 spec
(§112) claim root issuance is "never a programmatic path" / "never headless-silent"; that control is
**not implemented**, and against a same-uid adversary no local secret (owner code, TTY, env var, file)
could implement it — only a separate OS account or an out-of-band key can. The full threat model,
the architecture that would close it, and the residual boundary the code cannot guarantee are in
[`design/ROOT_ISSUANCE_TRUST_BOUNDARY.md`](design/ROOT_ISSUANCE_TRUST_BOUNDARY.md). **Operational rule,
unchanged and now load-bearing for the Guard too: run any untrusted or autonomous agent as a separate
OS user.** An **opt-in** mitigation now exists — `broker start --require-anchored-roots <anchor-pubkey>`
makes root creation require an anchor-verified certificate, closing the unsigned path (`DL1421`). It is
a real hardening, but **not** an airtight boundary on one account: its security reduces to keeping the
anchor private key and `root_policy.json` outside same-uid reach, so a separate OS account (or a
hardware-held anchor) remains the actual boundary.

**Update 2026-08-10 — the mitigation is now usable, recorded, and checkable.** Three defects had made
strict mode effectively un-runnable (a certificate's relative filesystem scope silently matched
nothing; the refusal reported itself as `DL1401 broker unreachable`; the scope flags were missing from
`--help`), which is the honest reason nobody turned it on. All three are fixed and the whole path
— `certify → adopt → delegate → run --lease` — is verified end to end. Two things changed about the
*residual* as well:

- **It is recorded.** Every broker start writes its effective mode into the hash-chained audit log.
  A same-uid process can still downgrade the policy file, but it can no longer do so quietly: it must
  leave permanent evidence or break `audit verify`, which is itself the alarm.
- **It is checkable.** `delulu doctor` reports a `security posture` section — root-issuance mode,
  anchor-key custody, and whether the filesystem enforces owner-only permissions at all. The
  operational rule below is no longer only prose you have to remember; it is a command that answers
  whether you followed it. See [`DEPLOYMENT.md`](DEPLOYMENT.md) for the verified recipe.

None of this closes the same-uid hole, and none of it is claimed to: to the kernel, a process running
as your user *is* you. What changed is that the boundary is now a deployment requirement you can
verify, rather than a caveat you had to have read.

### 1.6 How exposed is this while agents are writing code?

The realistic risks during an agent authoring loop, and where each stands:

| Risk | Status |
|---|---|
| An agent grants itself more authority | **Not possible from inside a program.** Grants come from the command line or a broker lease, not from source. |
| `delulu fix` quietly widens a row | **Refused by default.** A widening repair is never applied unless the operator names that exact repair id with `--accept-widening`. Six attempts at an accept-all wildcard left the file byte-identical. As of C90 the report line also stops calling an accepted widening "changes nothing". |
| An agent hides code from its reviewer | **Two variants closed.** Bidi controls (DL0107) and, as of C89, six characters that *render* as a line break without acting as one (DL0108) — the second is Trojan Source inverted: a guard clause visible to the reviewer and absent from the compiled program. |
| A dependency's authority drifts | `delulu authority --diff` compares two lockfiles; a manifest ceiling bounds each package. |
| An agent exhausts the machine | **Partly closed, partly open.** *Closed:* the parser is bounded at 128 levels in all four recursive-descent classes (`DL0210`/`DL0211`/`DL0212`/`DL0213`, 2026-08-04/09), so deep input is refused with a diagnostic and exit 1 instead of crashing — see §1.7. *Still open:* type inference is exponential on a small class of programs — 29 lines reach 10.2 s at depth 22, doubling per level — with no fuel bound, no `--max-type-size` and no timeout, and checking is quadratic in nesting depth. `delulu check` is the agent hot loop, so this remains a real denial-of-service surface. A bound is language-visible and belongs in an RFC; see `HARDENING_CAMPAIGN.md` P16 and `REMAINING_WORK.md` §2.3. |

### 1.7 Are there known leaks?

Yes, and they are enumerated rather than implied:

- **Once a secret is exposed, the language stops following it.** `expose` is an effect
  (`Declassify`) so the *capability* is visible in the authority report, but the resulting value is
  an ordinary string. This is a designed limit, stated in the report's own text.
- **A foreign call is a hole in the guarantee.** It is *enumerated*, not hidden. A raw secret cannot
  cross the boundary (DL0602), but foreign code is outside the proof by construction.
- **`Secret.verify` leaks one bit by design** (equal / not equal), in constant time — **and as of
  2026-08-03 that disclosure is known to understate the problem badly. See the entry below.**
- **`--trace-effects` buffers the whole trace in RAM** — about 70 MB per 100k effects, unbounded.
- **A hardlink planted inside a granted directory reads and writes the file it shares content with,
  even when that file also lives outside the grant** (red-team finding P20-R1, 2026-08-08). This is
  narrower than it sounds and was measured, not assumed: it is **not deliverable through a clone** (a
  hardlink does not survive `git` — the clone gets a plain file, no link), it needs an attacker who
  can already open the target, and the file genuinely *is* a member of the granted directory (a
  hardlink is a second name for one file, not a redirection like the symlink/junction that finding
  C84 closed). No cheap cross-platform defense exists — POSIX cannot enumerate an inode's names
  without walking the whole filesystem — so a fix would make containment platform-dependent, which
  the project refuses. Pinned as an executed characterization test; full threat model in
  `HARDENING_CAMPAIGN.md` P20-R1.
- **Filesystem containment is a check-then-open, so a concurrent writer into a granted directory can
  swap a checked file for a symlink in between** (`CONTAIN-TOCTOU-1`, 2026-08-10). **This one is
  open.** It is not reachable by the confined program through this API — the primitive table exposes
  no symlink-creating operation, so winning the race needs a *second* writer — and that writer is
  either a same-uid process (already outside the proof boundary) or anyone who can write into the
  granted directory. Closing it properly needs `O_NOFOLLOW`/`openat2`/`FILE_FLAG_OPEN_REPARSE_POINT`,
  which is the platform-dependent containment this project refuses, so it is stated rather than
  fixed. **Deployment consequence: grant scopes that point at directories only the program's own user
  can write — never a shared or world-writable one.**
- **A *dangling* symlink used to escape filesystem containment entirely — fixed 2026-08-10**
  (`SYMLINK-DANGLE-1`). `canonicalize` fails identically for "a name that is absent" and "a link whose
  target is absent", so the containment walk re-appended the link's own name as an ordinary component,
  the check passed, and the write then followed the link out of the grant. Unlike the hardlink above
  this **was** deliverable through a clone — git stores a symlink as a path string. Witnessed
  end-to-end before the fix, with the identical program against a link whose target *existed*
  correctly refused; the only variable was whether the target happened to exist.
- **A guard seal written the natural way gated nothing, and the CLI reported `ok` — fixed 2026-08-10**
  (`GUARD-SPELL-1`). `fs_read`/`fs_write` guard rules are matched against the runtime's *resolved
  absolute* path, so `guard policy set "fs_write:./out/secret.txt" sealed` stored a rule that could
  never fire. Witnessed: the program wrote the sealed file. Rules that cannot match are now refused at
  set time. Worth keeping in this list because a seal that reports success while gating nothing is
  worse than no seal — it stops the operator looking for another mechanism.
- **A valid program could abort the host during value teardown — fixed 2026-08-10**
  (`INTERP-DROP-1`). The interpreter bounded *call* depth (`DL0905`) and nothing bounded *data* depth,
  so a recursive value a few million deep killed the process **after the program had finished**, with
  no diagnostic. Teardown is iterative now.
- **Side channels are out of scope entirely.**
- **The editor was a way in, twice, and the second one needed no click.** The VS Code extension read
  `delulu.serverPath` at VS Code's default configuration scope, which a repository's own
  `.vscode/settings.json` can write — and the extension launches that path as a process the moment a
  `.delulu` file is opened. Opening a cloned repository therefore ran a binary the repository chose.
  Fixed 2026-08-07 by making the setting machine-scoped, and reproduced end-to-end both ways to be
  sure the fix was what refused it. The general lesson is recorded here rather than only in the
  changelog: **the toolchain around a language is part of its attack surface**, and an editor
  extension is the piece most likely to be trusted without being read.
- **The audit chain now detects truncation as well as modification and reordering** (`ANCHOR.json`),
  but it is still **not tamper-proof**: an attacker who rewrites the log *and* the anchor is not
  caught. Genuine tamper-evidence needs an external witness, and the head is now exportable so that
  one is possible.
- **Expiry can no longer be undone by moving the clock backwards** — `Broker::now` ratchets. What
  remains is that monotonicity is not accuracy: a rewind still distorts measured intervals.
- **The compiler could be crashed by deeply nested input, and now refuses it instead** (`DL0210`,
  fixed 2026-08-04). A *valid* module nested 100,000 levels deep used to overflow the stack and kill
  the process — exit 127, no diagnostic code, no span, nothing a caller could catch. That mattered
  more than an ordinary bug: `delulu check` is the gate every other guarantee here is verified
  through, and a gate that can be made to die instead of answering is one that can be skipped.
  Nesting is now capped at 128 levels. Two things are worth stating plainly about it:
  - The fix **narrows the accepted language.** Expressions past 128 levels used to compile.
  - It was found by looking at *disk usage*, not by any test. Ten 784 MB crash dumps had been
    sitting in `%TEMP%` for a day. The fuzzers never generated input that deep.

- **The bit `verify` returns is one you CHOOSE, and it used to be invisible — fixed 2026-08-03.**
  The "one bit by design" above assumed the bit answers a comparison you did not control. It does
  not. `Secret.map` hands its closure the **plaintext** and gates only on *purity* (DL0603) — and
  purity is not confidentiality — so a pure closure computes **any predicate you like** and encodes
  the answer into the returned secret; `k.verify(k.map(fn(x) { g }))` then tests the secret against
  an arbitrary `g`. Iterated, that recovers the whole plaintext. Until this fix, none of it emitted
  `Declassify`: `delulu why Declassify` printed *"program cannot perform `Declassify`"* for a
  program that printed the key in full.

  **Fixed:** `Secret.verify` now carries the `Declassify` effect in both the checker and the runtime
  trace table, so such a program is refused with **DL0501** unless it declares the effect. This
  re-closes **R-2**.

  **Stated honestly, the fix buys visibility, not impossibility.** A program that *declares*
  `!{Declassify}` may still do this — and then `delulu authority` tells you before you run it:

  ```text
  effects:      Declassify, Write
  exposure:     API_KEY declassifiable -> files/console
  ```

  That is the actual guarantee: declassification is an effect, and effects are in the type. What is
  fixed is that the toolchain can no longer deny doing something it does.

  **Still open (residue):** `verify` declassifies without needing `Cap[Declassify]`, while `expose`
  needs it. Closing that requires `verify` to return `Secret[Bool]`, which the runtime cannot
  represent today (`SecretVal` is String-only) — an RFC, not a patch. **So: holding a secret grants
  the ability to learn one chosen bit of it per call, without a declassify capability, but never
  without declaring the effect.** Treat `Secret` accordingly. Mechanism and witness:
  `docs/design/PROOF_CAMPAIGN.md` §IF-1.

  The **direct** surface was re-verified clean across twelve eliminators — printing, concatenation,
  `str()`, `==`, `assert_eq`, file writes and record embedding are all correctly refused
  (DL0602/0604/0605/0203).

---

## Part 2 — How it compares

### 2.1 Why are Delulu Authority and Guard better than a sandbox?

**They are not "better". They answer a different question, and the honest position is that you want
both.** Anyone telling you a type system replaces a sandbox is selling something.

| | A sandbox (container, seccomp, VM, WASI) | Delulu Authority |
|---|---|---|
| **When it acts** | At run time, when the call is made | At **compile** time, before anything runs — plus at run time |
| **What it knows** | That *this process* tried a syscall | That *this function* can perform this effect, and **why** |
| **Granularity** | Process | Function, module, package, dependency, plugin |
| **Answers "what can this do?" before running** | **No.** You must run it and watch. | **Yes** — `delulu authority` computes it from the code |
| **Answers "why can it do that?"** | No | **Yes** — `delulu why Net <file>` names the functions |
| **Covers dependencies distinctly** | No — one blob | Yes — each dependency has its own ceiling, pinned in the lockfile |
| **Survives being wrong** | It is the enforcement | It is one of several layers |

The real difference is **legibility before execution**. A sandbox tells you *no* at the moment of
the attempt; a type system tells you *what would be attempted*, in a report you can read in review,
diff between versions, and hand to a human before the program ever starts. For reviewing code an AI
wrote — which is the use case this language was built for — that difference is the whole point.

And the honest counterweight: **a sandbox does not care whether the compiler has a bug.** §1.1 is
exactly the case where the type-level answer was wrong and a lower layer was the thing that held.
Which is why the design is explicitly defense in depth — type proof → WASM/WASI floor → microVM
isolation → human-held broker keys — rather than a claim that any one layer suffices.

**Three of those four layers are built; the microVM one is not.** `--isolation microvm` is a probe
that names the missing prerequisite and refuses with `DL1408` on every host, including a
fully-provisioned Linux+KVM one — so the layer that would contain *genuinely untrusted* execution
is, today, the layer that is absent. That is stated here rather than left to be discovered, because
this section is an argument *for* defense in depth and it would be a poor one if it counted a layer
nobody can turn on. What holds instead is the WASM/WASI floor plus the broker, and — for untrusted
code — a separate OS account (`DEPLOYMENT.md` Tier 2), which is the boundary this project actually
verified with a second UID.

### 2.2 Can DeluluLang run inside a sandbox?

**Yes, and it degrades in the right direction.** It is an ordinary native binary with no daemon
requirement and no network dependency.

Tested: with the state directory made unreachable (`DELULU_HOME` pointed at a file), a program with
its grant still ran, and a program **without** its grant was still refused with DL0703 — enforcement
is in-process and does not depend on external state being writable. A custody that cannot reach its
broker fails **closed** (DL1401), which is the direction that matters.

Practical notes: build with `--no-default-features` for a binary with no embedded CPython (the
default build links `python313.dll`); the WASM/WASI backend exists precisely so a program can be run
under a second enforcement floor.

---

## Part 3 — Living with the language

### 3.1 Do the Survey and `doctor` affect people *using* DeluluLang?

**The Survey: no — it never reaches you.** It maps *this repository* for the people who maintain the
language. It is `publish = false`, and the packaging script builds only the `delulu` binary. It is
not a subcommand and it is not in the archive.

**`doctor`: yes, and only helpfully.** Run inside your own package it checks your machine — toolchain
version, state directory, broker mode, audit chain — and **writes nothing into your tree** (verified
by diffing the file list before and after). Outside DeluluLang's own repository it says so
explicitly:

```
repository
  note  delulu source tree   not inside DeluluLang's OWN source tree, so its repository-map checks
                             do not apply here — nothing about your project is being skipped
```

That last clause is deliberate: a skipped check that does not say it was skipped is a lie by omission.

### 3.2 Do plugins and add-ons actually work?

**Yes — built, inspected, verified and loaded, not just specified.** The flagship is a
zero-authority text transform:

```console
$ delulu plugin build examples/plugin_shout -o shout.dpx
ok: wrote shout.dpx (1333 bytes) — verified plugin `shout` v0.1.0, 1 export(s)

$ delulu plugin inspect shout.dpx
plugin `shout` v0.1.0 — class verified, api 1
  authority ceiling: effects []; requires []
  exports:    shout: fn(Str) -> Str
  sections:   delulu:plugin present · delulu:dir blake3 304a5992…
  signature:  none
```

The empty ceiling is the point: code that arrives *after* compile time and still cannot exceed its
grant. `plugin verify` runs the real load sequence's verification steps and gives identical verdicts
to a real load.

The manifest never overrides the code — the `rigged/` variant in the same example claims a pure row
while its code reads the clock, and **the build refuses it** (DL1501).

Two classes exist and the difference is visible in the types: a **Verified** plugin is re-checked at
load and keeps per-export rows; a **Contained** plugin is opaque, so every export is typed with the
plugin's *entire* granted authority (rule R-1). That is the honest price of containment-only trust.

**Limits, stated:** the hardware adapter is an operator-supplied **subprocess**, not a signed
in-process plugin; unless you pin a key with `--adapter-signer`, there is **no trust policy**; and
**no driver for any real device ships in this repository.**

### 3.3 Can the characters of the language be changed?

**Yes — keywords, operators and punctuation, in any script, including emoji.** It is called a
**syntax morph**, and it is a real shipped feature (`delulu morph list | info | check | render`).
Chinese keywords ship as a working example, as does a short-alias profile aimed at agents.

The law that makes it safe: a morph is a **bijective, token-level mapping to and from canonical
DeluluLang**. Every hash, artifact, diagnostic span, DIR and authority report is computed on the
**canonical** form, always. Two people reading one file through different morphs are provably
reading the same program.

What a morph may **not** do: rename identifiers, strings or comments; add or remove syntax; change
precedence; introduce macros. And it may not lie to a reader — an alias that is another keyword's
canonical spelling is refused (DL1711), because `let = "fn"` round-trips perfectly while meaning the
opposite of what it says.

**One thing to know, found on 2026-08-03 (C82):** under a morph, the aliases **are** the keywords, so
they are reserved. A program naming a variable `T` cannot be written in a morph where `T` means
`type`, any more than it could name one `if`. This used to be silently mis-rendered — a program came
back as `let type = 41` with both commands reporting success — and is now refused (DL1715). Note the
irony worth keeping: the **human-language** morphs were always safe by construction (identifiers are
ASCII-only, so a Chinese or emoji alias cannot collide); it was the **compact profile aimed at AI**
that was dangerous.

### 3.4 Is there any whitespace or indentation trap, like Python's?

**No. Indentation carries no meaning at all.** Verified rather than asserted — this program, with
mixed tabs and spaces at inconsistent depths:

```delulu
fn main(root: Root) ! {Write} {
            let out = root.console()
  let a = 1
			let b = 2
        out.println(str(a + b))
}
```

and the same program written **entirely on one line** with explicit `;` both check clean and both
print `3`. Blocks are delimited by braces; you can indent however you like, or not at all.

One thing that *is* true and is not indentation: **a newline terminates a statement** (the same
automatic-semicolon rule Go and JavaScript use), and `;` always works explicitly. `delulu fmt` has
one canonical style and zero options, so formatting is never a discussion.

Since 2026-08-03 there is also a guard here: six characters that *look* like a line break to a human
but are not one to the lexer are now refused (DL0108), because they could hide a line of code inside
a comment while a reviewer looked straight at it.

---

## Part 4 — Why this, and why "of the future"

### 4.1 Why would anyone switch?

Switch **if** you have this problem: *code you did not write, that you must run, and you need to know
what it can do to your system before you run it.* That is the case DeluluLang is built for, and the
one where nothing mainstream gives you a good answer.

Do **not** switch for speed: v1.0 is **2.0×–60.5× slower than C** on the measured workloads, and
`measurements/study-c/REPORT.md` says exactly that. Do not switch for ecosystem: there isn't one.

### 4.2 What does it offer an AI that other languages don't?

- **An authority report computed from the code, before execution** — a machine-readable answer to
  "what can this do", not a promise in a README.
- **Diagnostics carrying typed, machine-applicable repairs**, with a `authority_widening` flag on
  each, so an agent can apply the safe ones autonomously and escalate the rest by construction.
- **A stable machine envelope**: `--json` on every command, additive schema, exit codes contracted.
- **A refusal that is specific.** `DL0703` names the missing grant *and the flag that would supply
  it*. An agent can act on that without parsing prose.
- **A surface it may choose.** A compact keyword profile exists so an agent can cut tokenizer cost
  without forking the language — and the canonical form stays the shared coordinate system.
- **Batch checking that respects its cost.** On Windows, 82% of a small `check` is process creation;
  twenty separate invocations cost 711 ms where one costs 48.

### 4.3 What does it offer humans?

Mostly **the ability to review code an AI wrote.** `delulu authority` gives a complete list of what
a program can do; `delulu why <Effect>` names the functions responsible, at function granularity.
That is a review you can finish, on a codebase you did not write.

Also: one canonical format with no options, long-form `delulu explain` for every diagnostic, and a
20-chapter book. And the two Trojan Source defenses exist specifically to protect the reviewer —
they are not about the machine, which was never fooled.

### 4.4 What about AGI, ASI, robots, autonomous vehicles, aircraft, satellites?

Here the honest answer needs to be very precise, because this is the area where overclaiming would
be most dangerous.

**What genuinely exists and is tested:**

- **Physical-actuation authority is a first-class thing.** An actuator grant carries an *envelope* —
  per-dimension bounds, rate limits, a **dead-man heartbeat**, a TTL, and a mandatory `fail=` state.
  A grant with no dead-man is refused, and so is one that does not say what the machine does when the
  software stops: that is not a decision the runtime may make for you.
- **The envelope is enforced host-side, before one byte reaches vendor code** — demonstrated by
  reading the driver's own log, where a program commanding 12° (in envelope) and 999° (out) leaves
  exactly **one** line.
- **Revocation works mid-run and is measured**, not asserted: operator-to-stopped is 12.7 ms at p50
  and 39.7 ms worst over n=20.
- **Delegation across a lost link works** — the two-grant UAS pattern — and a delegated child can no
  longer outlive its parent's expired lease.
- **A device dimension exists in the authority lattice**, so authority can be scoped to a specific
  machine and joint.

**What does not exist, stated as plainly as possible:**

- **No physical device has ever been commanded.** Every demonstration drives a simulator.
- **No driver for any real device ships in this repository.**
- **Certification is NONE** — no functional-safety or security certification of any kind, for any
  domain. Not DO-178C, not ISO 26262, not IEC 61508.
- **macOS has never been executed**, not once.

So the correct sentence is: *the authority model was designed with these systems in mind, and the
device-scoping, envelope and dead-man machinery is built and tested against a simulator.* It is
**not**: "DeluluLang flies aircraft." Nobody should put this near a physical system that can hurt
someone without doing the certification work that has not been done.

### 4.5 Why call it "the language of the future"?

Because of the bet it makes, and a bet is what it is: **that most code will soon be written by
systems that are not human, and that the scarce resource will stop being the writing and become the
*reviewing*.**

If that is right, the thing you want in a language is not terser syntax — it is a mechanical,
checkable answer to *what can this program do to my system*, that survives dependencies, plugins,
and code that arrived after compile time. That is what authority-in-the-type buys.

If that bet is wrong, this is a slow language with a small ecosystem and an unusual type system.
It is entirely fair to say so, and the project says so.

The name is a claim about **intent**, not a measurement. There is no benchmark for "of the future",
and this project does not pretend there is one.

---

## Part 5 — The short list of things we cannot claim

Repeated here so no reader has to assemble it from the rest:

1. **Not unbreakable.** The word is forbidden. Guarantees are relative to a named trust base, and
   this campaign broke one of them.
2. **No mechanized proof of the FULL type system.** `Delulu Core` §1–§7 remains unformalized — no
   capabilities, no store, no secrets, no attenuation, no Progress and no Preservation. Lean 4.32.2
   **is** installed and machine-checks the **higher-order fragment** with no axioms at all; the
   custody grant tree is **model**-checked; the order laws are **symbolically** checked in Z3. Three
   different strengths, each bounded, and **none of them is the type system** (§1.4).
3. **Federation is not model-checked** — nor are concurrency, partitions or clock skew, and that last
   one matters, because the uplink lease exists to bound behaviour during exactly a partition. Leases
   and certificate adoption **are** model-checked (`Custody.tla`, 2,421 distinct states), which is
   where both of this project's real vulnerabilities were found.
4. **`Secret.verify` reveals one chosen bit per call without a declassify capability.** It now
   declares the `Declassify` effect, so it is always visible in the authority report — but visible
   is not impossible, and holding a secret is not the same as being allowed to read it.
5. **macOS has never been executed.**
6. **No physical device has ever been commanded**, and no real driver ships in-tree.
7. **No certification, in any regime, for any domain.**
8. **Multi-tenancy is not provided** — several agents with different authority need separate OS
   accounts or containers.
9. **Not competitive with C** on the measured workloads (2.0×–60.5× slower).
10. **Foreign calls are outside the proof.** Enumerated, not eliminated.
11. **Side channels are out of scope.**
12. **Post-quantum options are unvalidated** and gated behind `--unstable`.
13. **Nothing is hosted.** No registry, no download page, no public repository — you build from
    source or produce your own archive.
14. **Open robustness defects**: exponential type inference on a small class of programs, and
    quadratic checking in nesting depth. Both are unbounded — no fuel, no `--max-type-size`, no
    timeout. (The third defect this item used to name — an unbounded parser recursion that crashed
    outside the exit-code contract — was **closed** in 2026-08-04/09; all four recursive-descent
    nesting classes are now capped at 128 levels, `DL0210`–`DL0213`. It is listed here as fixed
    rather than deleted, because this list is what a reader checks against.)
15. **Type inference is order-dependent and has no principal types** — swapping two parameters can
    decide whether a program compiles. Fail-closed, so no authority escapes, but undocumented until
    now.

---

## Where the evidence lives

| | |
|---|---|
| Every defect this campaign found, with witnesses | [`docs/design/HARDENING_CAMPAIGN.md`](design/HARDENING_CAMPAIGN.md) |
| The proof-boundary ledger — every guarantee in exactly one of seven categories | [`docs/design/PROOF_CAMPAIGN.md`](design/PROOF_CAMPAIGN.md) |
| Formal models + the checker's verbatim output, including the run that proves the model has teeth | [`docs/design/models/`](design/models/) |
| The rules that keep authority in the type, and the R-4 reopening | [`docs/design/SOUNDNESS_AUDIT.md`](design/SOUNDNESS_AUDIT.md) |
| The 18-item authority and 14-surface Guard audit | [`docs/design/AUTHORITY_GUARD_CAPSTONE.md`](design/AUTHORITY_GUARD_CAPSTONE.md) |
| Trust assumptions and honesty clauses | [`docs/design/CONSTITUTION.md`](design/CONSTITUTION.md) §5.14, §9 |
| What is promised to stay put | [`docs/design/STABILITY.md`](design/STABILITY.md) |
| Ten known limitations in one page | [`docs/release/CHECKPOINT-1.0.md`](release/CHECKPOINT-1.0.md) |
| Performance, measured and unflattering | [`measurements/study-c/REPORT.md`](../measurements/study-c/REPORT.md) |
| Reporting a vulnerability | [`SECURITY.md`](../SECURITY.md) |
