# Version co-evolution: broker, protocol, and DIR majors

**Status:** normative from 1.0 (Stage 10 §4, phase 10k). **Governs:** how the three
independently-versioned wire/interchange surfaces — the **broker/IPC protocol**, the **`delulu:cap`
wire**, and the **DIR** (the checker's dependency-interface representation, with its primitive
table) — change major versions without stranding a supported release. **Related:**
`docs/design/STABILITY.md` (what is stable within a major), `docs/release/SUPPORT_MATRIX.md`
(release trains and LTS windows).

`STABILITY.md` §1 already promises compatibility *within* a major for each of these surfaces. This
page answers the question it deliberately leaves open: **what happens at a major boundary**, and for
how long the old and new majors are supported side by side.

## 1. The three surfaces and their versions today

| Surface | Constant (today) | What a major bump means |
|---|---|---|
| Broker / IPC protocol | `WIRE_VERSION = "broker/1"` (`crates/delulu/src/broker_ipc.rs`) | The client↔daemon framing or request/response contract changed incompatibly. |
| `delulu:cap` broker wire | broker major `1` | The capability-lease wire changed incompatibly. |
| DIR + primitive table | `DIR_VERSION = 1`, `PRIM_TABLE_VERSION = 4` (`crates/delulu-check/src/dir.rs`) | A DIR the toolchain accepted no longer round-trips, or the primitive table's contract changed shape (the prim-table version bumps additively far more often — it has already reached 4 across Stage 10 — without a DIR major, because adding a primitive is not breaking). |

These version independently. The prim-table version rising (1→2→3→4 across phases 10e/10f/10h) is
routine and additive; a **DIR major** or a **protocol major** is the rare, breaking event this
policy is about.

## 2. The co-evolution rule

**During any LTS window, the toolchain supports n−1 majors of each wire/DIR surface concurrently.**

Concretely, when surface *S* moves from major `k` to `k+1`:

- The toolchain that introduces `k+1` **still reads and speaks `k`**. A daemon on `broker/k+1` still
  serves a client on `broker/k`; a toolchain on `DIR k+1` still accepts a `DIR k` artifact. It does
  so for **at least the remaining life of every LTS whose 24-month window is open at the moment
  `k+1` ships** — never less than one full LTS window from the bump.
- Support for major `k` is dropped only after the **last LTS that depended on `k` has left its
  24-month window.** A major is never retired while a supported LTS still speaks it.
- `k+1` and `k+2` are **never** both required at once: the guarantee is n−1 (the current major and
  the one before it), not the whole history. A surface three majors old is EOL and refused with a
  clear diagnostic (`DL1801`/`DL1802` for edition/DIR-era mismatches; the broker refuses an
  unsupported `WIRE_VERSION` at handshake), never silently mis-parsed.

## 3. How a bump is announced and enforced

- A major bump to any of these surfaces ships **on a major DeluluLang release**, never on a minor
  train — it is a stable-surface break by definition (`STABILITY.md` §2).
- The bump is announced in the release notes and in this page's history, with the **concurrent-
  support window stated explicitly**: which old major is still spoken, and the date (tied to an LTS
  window) it will stop being spoken.
- Enforcement is by version check at the boundary, fail-closed: an artifact or peer on an
  out-of-window major is **refused with its code**, not best-effort parsed. A refusal that names the
  version mismatch is safe; a silent reinterpretation of an incompatible wire is the failure this
  policy exists to prevent.

## 4. Why n−1, and not "forever"

Supporting every historical major forever turns each wire reader into an open-ended museum of
formats, which is where compatibility bugs and parser-differential vulnerabilities breed. Supporting
exactly the current major and the one before it — for no less than a full LTS window — gives every
operator a real, unhurried upgrade path (you are never forced to jump two majors at once, and never
on a patch-release timescale) while keeping each reader's job bounded. The LTS window is the unit of
"unhurried" precisely because it is the window an operator already planned their support around.

## 5. What this does not promise

- It does not promise that a **major** transition is additive — it is not; that is what makes it a
  major. It promises the *overlap* during which both majors work, so the transition is not a flag
  day.
- It does not promise cross-major behavior *identity* — a `DIR k` artifact read by a `k+1` toolchain
  is accepted and checked, but the toolchain's own newer analysis applies; what is promised is that
  it is accepted, not that it is treated byte-identically to how major `k` treated it.
- It does not extend to non-wire surfaces (grammar, typing, diagnostics): those are add-only within
  a major under `STABILITY.md` and do not have an "n−1 majors" story because they do not cross a
  wire.
