# The DeluluLang Registry — policies

**Status:** normative from 1.0. **Implements:** `STAGE9_SPECIFICATION.md` §5.
**Deployment:** the registry ships as a runnable service (`delulu-registry serve`). Putting it
behind a CDN is a deployment act at public launch, not a code artifact — build-order ruling D3.

---

## 1. The rule everything else serves

**A publisher cannot claim an authority they do not carry.**

The authority summary on every index line is **recomputed server-side from the uploaded artifact**.
It is never copied from what the client submitted. If the client submits an authority that
disagrees with the artifact, the publish is **refused** (`DL1706`) — not corrected, not accepted
with a warning.

This is the load-bearing property. `delulu add` shows you a package's authority *before* you
download anything, and that is only worth doing if the number came from the artifact rather than
from the person who wants you to install it.

### 1.1 When the registry cannot tell

If the server cannot derive an authority from the artifact, **the publish is refused.** It is never
accepted with the publisher's claim standing in.

This is the branch that matters most. An attacker controls the artifact and can aim for the
undecidable case on purpose; a "could not tell, so I trusted you" path would make the whole
recomputation theatre. Verification that fails open is worse than no verification, because it also
produces a record saying the authority was checked.

Verification runs under the strongest Stage-5 isolation profile the host provides. On a host that
cannot offer a microVM, it runs under the ruled weaker fallback and the registry says which one it
used — never implying the strongest (`DL1408`'s discipline, applied to ourselves).

## 2. Tokens

- **Scoped.** A token names the packages it may publish. Publishing anything else is refused. The
  default is not "everything its owner can reach" — it is exactly the list it was issued with, so a
  leaked token costs its scope and no more.
- **Revocable, immediately.** Revocation takes effect on the next request. The record is kept with
  `revoked: true` rather than deleted: revoking a token should not rewrite the history of it having
  existed.
- **Never grants yank on someone else's package.** Yank is destructive to consumers' resolution; a
  stolen token must not be able to reach across the registry with it.
- **Unknown and revoked are the same answer.** Distinguishing them would hand an attacker a free
  oracle for which tokens once existed.
- The client stores tokens in `~/.delulu/credentials.jsonl` (0600 on unix) and **never echoes
  them** — a credential printed to a terminal ends up in a scrollback buffer, a screen recording,
  and a CI log.

## 3. Yank ≠ delete

A yanked version:

- **stops satisfying new requirements** — resolution moves to the latest unyanked version;
- **keeps resolving from existing lockfiles** — a build that already pinned it still works;
- **stays in the index**, flagged, rather than disappearing;
- **can be un-yanked**, because yank is reversible and deletion is not.

Deleting a version breaks builds that were working, punishing people who did nothing wrong for a
problem they did not cause. Yanking stops the bleeding without doing that.

## 4. Signatures

**Mandatory on publish.** An unsigned artifact is refused (`DL1705`), as is a signature that does
not verify over the artifact's exact bytes.

Signing authenticates **origin, not behaviour**. A validly signed package can still do something
you do not want — which is why the authority summary exists and why it is recomputed rather than
trusted. The two mechanisms answer different questions: the signature says *who*, the authority
says *what*.

## 5. Immutability

A published version is immutable. Republishing `1.0.0` is refused even by its own author, and even
if the content is identical. Anyone who resolved that version got specific bytes, and the registry's
job is to keep that true.

Immutability is checked **before** the semver-authority rule, so a publisher colliding with an
existing version is told that, rather than being sent to fix an authority problem they do not have.

## 6. Semver-authority

Widening a package's authority requires a **major** version bump (`DL1003`). Authority is part of
the public interface, so it obeys semver like any other part of it. Under `0.x`, the minor is the
compatibility axis (cargo's rule).

## 7. Outage behaviour

**A registry outage must never break a build that already resolved.** Resolution is cached in the
lockfile; an offline build reads the lockfile and the vendored sources and needs no network at all.

The registry is a trusted service for **distribution**. Verification remains client-side and local
— trust-on-first-verify is the honest description, and it is unchanged by the registry existing.

## 8. Namespaces

First-come, with a squatting-review process. A name held without use, and wanted by someone who
will use it, can be reassigned by review — with the prior holder notified and given a chance to
respond. **The reassignment never rewrites published versions**: existing lockfiles continue to
resolve exactly what they pinned, because §5 does not have an exception for administrative
decisions.

## 9. What this does not do

- It does not make a package safe. It makes a package's authority **legible before you install it**.
- It does not detect malice within a granted authority. A dependency that was always allowed `Net`
  and starts using it differently is not caught here — Study A's methodology says the same thing.
- It does not replace the lockfile. The lockfile is what makes your build reproducible; the registry
  is only where the bytes came from the first time.
