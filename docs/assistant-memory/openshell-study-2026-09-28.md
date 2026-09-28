---
name: openshell-study-2026-09-28
description: "NVIDIA OpenShell (Open Agent Safety Platform runtime, launched 2026-09-28) studied on Jesse's commission; what DeluluLang takes (PS-E, P8-04, P9), what it does not, the no-copy terms"
metadata:
  type: project
---

On 2026-09-28 (evening) Jesse paused the routine and commissioned a study of **NVIDIA OpenShell** — the
open runtime of NVIDIA's Open Agent Safety Platform, launched that day with Sentry (a DPU watchdog) and
100+ partners — *"studied and incorporated not just as copy but as real engineering for the sandbox"*.
Record: `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md`; decisions D-V2-52 (terms, order), D-V2-53 (PS-E),
D-V2-54 (P8-04), D-V2-55 (P9).

- **Read:** 57 doc pages as Markdown (`https://docs.nvidia.com/openshell/latest/llms.txt` lists them;
  append `.md` to any page), the 7 diagrams from their SVG sources in the repo's `docs/images/`, the repo
  at `36b0386` (architecture/, RFC 0012 typestate contract, prover = Z3, binary identity = SHA-256 TOFU).
- **The key difference:** OpenShell governs binaries it cannot read (intercepts syscalls, per-request
  policy); DeluluLang governs programs it compiles (authority from code, guest performs no effects, `⊑`
  proved once) → DeluluLang can DENY what OpenShell must intercept, and check PROGRAMS against a boundary.
- **Taken:** PS-E-01 boundary confirmed by construction + generation per run + profile required sets
  (`hostile-agent` refuses without a second identity); E-02 host loss ends the guest (macOS watcher,
  launcher death signal); E-03 six HYPOTHESES (unnamed syscalls, `socket` after lock-down — Landlock is
  TCP-only, `/proc` → `/proc/self`, host `PR_SET_DUMPABLE=0`, Landlock ABI, macOS/Windows equivalents) —
  witness each red before fixing; E-04 launcher resolved/hashed/pinned; E-05 `sandbox policy --format
  openshell` + OpenShell as tested L3 + `openshell-prover` cross-check in a manual `openshell.yml`; E-06
  OCSF export keeping the hash chain; P8-04 out-of-band monitor (revoke only); P9-01 `authority --within`
  (4 results, source-located counterexample); P9-02 `grants diff` findings; P9-03 proposals with
  `⊑`-bounded auto-approval; P9-04 endpoint-bound secrets; P9-05 method/path scopes as a `⊑` dimension.
- **Not taken:** deny rules, hot-widening, binary identity for the guest, audit-by-default, an HTTP
  endpoint in the guest, middleware, a fleet control plane, L7 beyond REST (for now).
- **Terms:** OpenShell is Apache-2.0 — NOTHING copied into the repo (code, schema, text, diagrams), no
  crate dependency; NOTICE and the licence are Jesse's.

**Why:** Jesse's commission, 2026-09-28. **How to apply:** build PS-E → P8 → P9 in that order from the
study's §4; never call a §4.3 hypothesis a finding until its witness is red; never copy OpenShell
material. Related: [[cloud-period-2026-09-28]], [[delulu-v2-execution]], [[discovery-over-checklist]].
