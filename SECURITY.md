# Security Policy

**Status:** normative from 1.0. **Governs:** Constitution §9 (governance and honesty clauses),
`STAGE9_SPECIFICATION.md` §4. **The patch runbook in §4 has been rehearsed** — the drill timeline is
in `docs/security/DRILL-001.md`. **Updated 2026-09-17**, when the testing repository became public: §1
(a private reporting channel is open) and §6 (which GitHub protections are live). Approval record:
`docs/design/ENTRENCHED_CHANGE_RECORD.md`.

---

## 1. Reporting a vulnerability

**Report privately through GitHub.** On this repository, open the **Security** tab and choose
**Report a vulnerability** — GitHub's private vulnerability reporting: on the public repository,
[`jessesuniljs1-collab/DeluluLang`](https://github.com/jessesuniljs1-collab/DeluluLang/security/advisories/new), since
2026-10-17, and on the development repository since 2026-09-17. A report is visible only to you and to the repository's
administrators, and nothing about it is public unless and until an advisory is published.

**It is the only channel** (D-V2-154). No `security@` address and no PGP key are offered, because none would be
monitored: the project has one maintainer, and GitHub's form already reaches him privately and carries the report
through to a published advisory and, where one is warranted, a CVE.

That is stated rather than papered over. A security policy that lists a channel which does not answer
is worse than one that admits it is not open yet: it costs a reporter the window in which the bug
could have been fixed. The channel above answers, on these terms:

- it is private — **not** a public issue tracker;
- an acknowledgement within **3 working days**, and a substantive response within **10**;
- a fix is announced as a GitHub Security Advisory on the public repository, crediting the reporter unless they ask
  otherwise.

**Please do not** open a public issue for a suspected vulnerability, and please do not test against
infrastructure you do not own.

## 2. Coordinated disclosure

**90 days by default**, from acknowledgement to public advisory. Shorter if a fix ships sooner;
longer only by agreement with the reporter.

If a vulnerability is being **actively exploited**, the timeline compresses to whatever is needed to
protect users, and the advisory is published with the fix rather than after it.

We will not ask a reporter to stay silent past the agreed window. A researcher who has waited out a
disclosure period in good faith is free to publish.

## 3. Severity rubric

Severity is judged by what an attacker gains, not by how clever the bug is.

| Severity | Meaning | Examples | Target fix |
|---|---|---|---|
| **Critical** | The authority guarantee is broken: a program obtains authority it never declared, or the checker accepts a program that violates an audit rule (R-1…R-7). | Effect laundering that type-checks; capability forgery; a Contained plugin escaping its grant. | 7 days |
| **High** | Custody or verification is subverted without breaking the type system. | Signature verification bypass; lockfile authority pin bypass; audit-chain forgery; a fail-open path where custody cannot answer. | 14 days |
| **Medium** | Containment weakens, or a refusal can be avoided, without direct authority gain. | Isolation profile weaker than its label; resource-limit escape; a diagnostic that can be suppressed. | 30 days |
| **Low** | Defence in depth reduced; no direct exploit path. | Missing hardening; a denial-of-service against the compiler; information disclosure in diagnostics. | 90 days |

**A crash is not automatically Low.** A runtime fault that aborts the host instead of producing a
diagnostic is at least Medium, because callers lose the ability to distinguish failure modes. Stage
9 fixed exactly such a bug (`DL0905`, found by Study C).

### 3.1 What is explicitly in scope

- Anything that makes `delulu authority` **under-report** what a program can do.
- Anything that makes the checker accept a program violating a rule in `docs/reference/`.
- Any path where custody, signature, or lockfile verification **fails open**.
- Any way to make a diagnostic disappear that should have fired.
- **Anything that makes source render differently than it compiles.** Because a first-class use of
  this language is a human or an AI *reviewing* code another AI wrote, an attack on the reviewer —
  making the glyphs disagree with the tokens — is an attack on the guarantee. The lexer refuses
  Unicode bidirectional control characters for this reason (DL0107, "Trojan Source", CVE-2021-42574;
  identifiers are ASCII-only, which closes the homoglyph/invisible-character vector on names). A new
  way to desynchronize rendering from meaning is in scope.
- **Anything that makes a declaration mean something other than it appears to mean.** Same reasoning
  one level up from glyphs: a declaration that is accepted and then silently has no effect misleads
  review without ever failing. Redeclaring a builtin type or a core effect was exactly this and is now
  refused (DL0302 — `effect Write` used to leave every `! {Write}` meaning the real, filesystem-reaching
  `Write`). A new way to make a name resolve differently than it reads is in scope.
- **Anything that makes the audit log lie, including by omission.** The log is observability, not
  enforcement — it detects rather than prevents, and that is by design. But a read surface that
  displays a tampered record as authentic, or silently drops a corrupted one from a listing, breaks the
  only property the log has. Both were true until campaign C30; `tail`/`query` now verify the chain,
  warn loudly, exit nonzero, and state that entries may be missing entirely.
- **Any way a revoked or expired grant is treated as live** on any path — including paths that do not
  themselves authorize anything. Redeeming a lease token for a revoked grant used to succeed and record
  `decision: "allow"` (C29); enforcement refused the grant afterwards, so nothing was authorized, and
  it was still a defect worth fixing because the log is read by people making decisions.
- **Any way a `Secret[T]` becomes an ordinary value without `Cap[Declassify]`**, and any way a secret
  crosses the foreign boundary without passing `expose` (DL1301 fences the FFI signature, DL0602
  refuses the value, DL0604/DL0605 keep it unprintable and uncomparable). Note the *converse* is not a
  vulnerability: see §3.2.

### 3.2 What is not a vulnerability

Stated so reporters do not spend their time:

- **Foreign code doing anything.** `ForeignCall` is documented as a hole in the proof; the authority
  report enumerates it. That it is a hole is the design, not a bug.
- **A program with `Root` deriving any capability.** Holding `Root` is holding everything beneath
  it. Passing `Root` to a dependency is an anti-pattern the authority report exposes — and, as
  Study A records, exactly the shape that makes a supply-chain effect change possible at all.
- **Performance.** Slow is not a vulnerability unless it is a usable denial of service.
- **Isolation profiles being weaker on a platform that cannot host them** — provided the label says
  so. `DL1408` reporting a weaker fallback honestly is the system working.
- **An authorized `expose` sending a credential anywhere it likes.** `expose(Cap[Declassify])` is the
  sanctioned way to declassify a secret, and once exposed it is an ordinary `Str` the type system no
  longer tracks. Reaching that point requires `Declassify` in the row, permission from the package
  manifest, and an explicit human grant of both `declassify` and the secret's value — and
  `delulu authority` now prints an `exposure:` line naming the secret and the egress it could take.
  That chain working as designed is not a vulnerability; a way to *skip a link* in it is.

---

## 4. The patch runbook

This is the procedure, and it has been **executed as a drill** (§5). A security process nobody has
run is not a security process; it is a document.

### Phase 0 — Receipt (target: within 3 working days)
1. Acknowledge to the reporter. Give a tracking id.
2. **Reproduce.** If it cannot be reproduced, say so and ask for detail — never close silently.
3. Assign severity per §3. When uncertain, take the higher one.

### Phase 1 — Containment (Critical/High: same day)
4. Determine the affected versions and whether a **published artifact** is compromised.
5. If a registry package is implicated, **yank** the affected version. Yank ≠ delete: existing
   lockfiles keep resolving, so nobody's build breaks while they upgrade.
6. Decide whether the fix can be disclosed with the release or needs an embargoed branch.

### Phase 2 — Fix
7. Work on a private branch. **Write the failing test first** — including the skip-branch case: the
   "what if the checker could not tell" path is where security rules die.
8. Fix. The test must fail before and pass after; a fix without a test that pins it is not a fix.
9. Run the full suite, the conformance coverage law, and the audit exploit set (F-1…F-6, R-7).

### Phase 3 — Release
10. Bump the patch version. Security fixes never carry unrelated changes — a reviewer must be able
    to read the whole diff.
11. Build, **sign the artifacts**, generate provenance and the SBOM.
12. Verify the signature from a clean checkout before publishing anything.
13. Publish; update the registry index.

### Phase 4 — Advisory
14. Publish the advisory: affected versions, severity, impact, the fix, and credit to the reporter
    unless they decline.
15. **State what was not fixed**, if anything. A partial fix described as complete is a second
    vulnerability.
16. Record the timeline — internally at minimum, publicly where it helps others.

### Phase 5 — Retrospective
17. Ask the only question that matters: **why did the existing tests not catch this?** Add the gate
    that would have. A vulnerability class that can recur has not been fully addressed.

---

## 5. Drill status

The runbook has been rehearsed against a staged vulnerability planted in a release-candidate
branch. Timeline, target times, and what the drill exposed: **`docs/security/DRILL-001.md`**.

## 6. Supporting controls

Until 2026-10-17 several rows here waited for the project's public repository, marked as pending. That repository
exists now, and each row is decided (D-V2-154): **live**, or **not adopted**, with the reason. Nothing here is claimed to
be live that is not — see `STAGE9_BUILD_ORDER.md` D2.

| Control | Status |
|---|---|
| Two-person review on every change | **Not adopted** — the project has one maintainer, and a rule nobody can satisfy would be a claim, not a control. `main` refuses force-pushes and deletion, every change passes CI, and CODEOWNERS names the maintainer. |
| Signed commits and tags | **Not adopted** — what a user installs is vouched for by the release's build attestations (next rows), not by commit signatures. |
| Release artifact signatures | **LIVE** — the project's own ed25519 machinery, verified by `delulu verify-sig` |
| SLSA provenance | **Build Level 2, live per release**: every archive carries a GitHub artifact attestation from the reusable workflow `.github/workflows/release-build.yml`. **L3 is claimed only after** `gh attestation verify <archive> --signer-workflow jessesuniljs1-collab/DeluluLang/.github/workflows/release-build.yml` has passed on a published release, and the release notes say which. |
| Reproducible builds | **LIVE** — see `STAGE9_BUILD_ORDER.md` D6 |
| SBOM (CycloneDX) per release | **LIVE** |
| OpenSSF Scorecard | **Runs** on the public repository (`.github/workflows/scorecard.yml`); **no floor is claimed** — a score is published, not promised. |
| AI-contribution policy | **LIVE** — `CONTRIBUTING.md` §AI |
| AI-overseer monitoring | Advisory only, never merge authority — Constitution §9: the guarantees hold even if every overseer colludes |
| Private vulnerability reporting | **LIVE** on the public repository since 2026-10-17 and the development repository since 2026-09-17 (GitHub) — the channel in §1 |
| Secret scanning and push protection | **LIVE** on both repositories (GitHub): a push carrying a credential GitHub recognizes is refused |
| Dependency alerts | **LIVE** on the public repository (Dependabot alerts), beside `cargo deny` in CI |

## 7. Our own limits

- Soundness claims are design-level plus audit-rule plus test-enforced. The Delulu Core
  mechanization is open work, and the release notes say so.
- Governance reduces supply-chain and contributor risk; it does not eliminate it. The structural
  defence — the semantics themselves — remains the strongest layer, and it is the one that does not
  depend on anyone behaving well.
- The registry is a trusted service for **distribution**. Verification is client-side and local;
  trust-on-first-verify remains the honest description.
