---
name: delulu-next-evolution-2026
description: "The 2026 evolution commission (2026-09-17): TWO planning passes (the Next Evolution reassessment; the sandbox + VM isolation pass) written, committed and pushed to the testing repo — AWAITING JESSE'S APPROVAL, nothing built; where everything lives, the decisions, the verified findings, what the next session must do first"
metadata:
  node_type: memory
  type: project
  originSessionId: 1dc77f30-fb11-48ee-a213-6e2c7e89d80f
  modified: 2026-09-17T17:47:57.263Z
---

# Next Evolution 2026 — both passes complete; APPROVED by the owner on 2026-09-17 evening and now executing as DeluluLang V2 (see [[delulu-v2-execution]]; the folder below was archived to `docs/archive/v1/NEXT_EVOLUTION_2026/` in phase V2-0 — the paths in this file are the pre-archive ones)

**Where:** `docs/NEXT_EVOLUTION_2026/` — README, MASTER_PLAN (§12 = the sandbox decision + the 15
integration answers), RESEARCH, VERIFICATION_FINDINGS (NE-01…NE-22), IMPLEMENTATION_ROADMAP
(P1–P8 + PS-0…PS-D, revised order), DECISION_LOG (D-NE-01…D-NE-33), DOCUMENTATION_AUDIT,
EXECUTION_LOG (Entries 1–3), OWNER_COMMISSION (a pointer), SANDBOX_RESEARCH / _ARCHITECTURE /
_THREAT_MODEL / _TEST_PLAN / _IMPLEMENTATION_PLAN, `agent-notes/` (Opus 5 red team; Sonnet 5 host
facts — one marked head-chef annotation in the Sonnet file). HANDOFF now points at the folder
(header, §5 start-here row, §8 owner-blocked row, §10 step 4).

**Commits (all on master, all pushed to `origin` = the testing repo, ls-remote verified):**
- morning: `1ac8ecb` (plan) → CI `35193023549` GREEN; `0fca1ef` (log entry appended after the
  regen) → `35193076106` RED on Survey freshness exactly as predicted; `9584011` (regen last) →
  `35193171249` GREEN. All three READ and recorded in EXECUTION_LOG.
- `bd074ea` dispatch-only host-capability probe workflow → run `35218542442` success (measured:
  ubuntu-latest kernel 6.17, `/dev/kvm` present, unprivileged userns BLOCKED by AppArmor, Landlock
  ABI 7, cgroup v2 ok, no bwrap/firecracker; macos-latest arm64 `kern.hv_support=0`, sandbox-exec
  present; windows-latest Server 2025 Hypervisor Platform ENABLED, session admin).
- afternoon: **`3fd69f2`** = the sandbox pass (22 files, Survey regenerated last, 0 errors,
  0 warnings, doctor 17/17, the four doc gates green). Its push run **`35223153049` READ: GREEN on
  every push job** (test ×3 OSes, arm64, supply-chain, miri ×2, miri-ffi, editor, lints, formal;
  10.5 min; heavy-gates/miri-slow skipped by design). `bd074ea`'s push run `35218534498` READ: red
  on the four test jobs, freshness gate ALONE, as predicted. Both recorded in EXECUTION_LOG Entry 3
  by the recording commit **`0bf8b11`** (= HEAD; also HANDOFF's count → 208 and a CROSS_PLATFORM §9
  pointer saying planning-commit runs live in the execution log), pushed, ls-remote verified, tree
  clean. **`0bf8b11`'s own push run `35224262541` (queued 12:58Z) has NOT been read** — the next
  session reads it and records it in the next EXECUTION_LOG entry, the morning's convention.

**Commission files:** Jesse moved the first commission BACK to `docs/design/DeluluLang_Fable_5.1_Master_Prompt.md`
and placed the second beside it (`…_Sandbox_VM_Integrated_Next_Evolution_Prompt.md`); both are now
committed there (D-NE-20 supersedes D-NE-12); the first has the banned word redacted in place with
a header note; `OWNER_COMMISSION.md` is a pointer. The copy beside the repo
(`D:\nelan\DeluluLang_Fable_5.1_Master_Prompt.md`) may still exist; it is Jesse's.

**The sandbox decision (MASTER_PLAN §12, D-NE-21…23):** sandboxing is a **cross-cutting execution
layer introduced early** — PS-0 (truth, probes, four runtime hardenings) inside P1; PS-A (L1 process
jail + effect channel on Win/Linux/macOS) right after P1; **the microVM is re-sequenced, not
deferred** → PS-C on Linux/KVM (CI has KVM); Windows/macOS microVMs stay deferred with triggers.
Load-bearing design: **the guest performs no effects** — capabilities are opaque handles, the host
performs every effect over one bounded channel (D-NE-22); the microVM guest runs the interpreter,
not the WASM engine (D-NE-23, a Stage 5 §6 deviation needing a ruling). Revised order:
P1(+PS-0) → PS-A → P2 → P4-01 → P3 → PS-B → P4-02…07 → PS-C → P6 → P5 → P7 → PS-D → P8.

**Verified findings that drive it (VERIFICATION_FINDINGS §4, all reproduced on the binary):**
NE-16b `--isolation` invisible under `--json`; NE-16c DL1408 ships without its promised repair;
**NE-17 `http.get` has NO network client** (unconditional `Err(Refused)` after the authority
checks); NE-18 `--grant net=169.254.169.254` accepted silently; **NE-19 Windows device names pass
containment** (`CON` created a file via `\\?\`); NE-20 trailing dots/spaces stripped AFTER the
decision; NE-21 foreign-worker channel has no read deadline (IPC-1's fix never reached it);
NE-22 no CPU/memory/time bound on the main program on either engine. Still the headline from the
morning: **NE-01 a program cannot load a plugin at run time** (`prim.rs:366` stub).

**Owner decisions pending (MASTER_PLAN §9 + §12.2 q12):** approve/reorder; PS-A before or after
P2; D-NE-23 interpreter-in-guest; D-NE-24 profile names; D-NE-25 `Secret.map` under strong
profiles; D-NE-26 `landlock`/`seccompiler` crates; D-NE-27 shipping a GPL guest kernel;
D-NE-28 first network client's TLS dependency + special-address spelling; D-NE-31 resource-limit
defaults; D-NE-33 `--sandbox` default; plus the morning's: release channel, installer posture,
loading-grant grammar, Constitution §5.15, rustfmt/CoC, the four pre-public items (untouched).

**Housekeeping facts:** two agent worktrees exist under `.claude/worktrees/` (`agent-ab8e3c649dff8d3e7`,
`agent-af964add0968902db`, both at `9584011`) — never delete without asking. The Bash tool here
FAILS on long heredocs (unexpected EOF) — write long content with the Write tool and apply with a
python script (a Bash `<<'EOF'` heredoc of ~60+ lines fails to parse; short ones work). HANDOFF's
"commits past v1.0.0" figure is 208 at `0bf8b11` (set by counting, not remembered).

**Why:** Jesse commissioned plan-first, explicit STOP before implementation, twice; the second
commission said not to treat the first roadmap as approved. **How to apply:** do NOT start P1/PS-0
without his approval; when he approves, follow the per-phase end-of-phase protocol
(`IMPLEMENTATION_ROADMAP.md`); research sources stay inside the plan folder
([[no-banned-word-mentions]] absolute); agents only under [[agent-usage-rule-2026-09-17]].
Related: [[delululang-project]], [[delulu-remaining-work-inventory]], [[delulu-github-remote]],
[[testing-repo-autopush]], [[delulu-p22-containment-campaign]] (the search key that found NE-19/20).
