---
name: delulu-core-regression-rule
description: "Standing order — after any tooling work, prove the DeluluLang core is unchanged by differential test against a pre-change binary, in both CLI and compiler"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-08-02T11:17:38.829Z
---

Jesse's standing order (2026-08-02): *"After completing everything and every phases double check
and verify everything build and make changes as needed... test the core delululang and verify that
delululang core features are not affected by all the changes made and new things created outside
of core delululang. Test in both cli and compiler."*

**Why:** almost everything built during the long-term production phases — `fix`, `new`,
`completions`, `add --path`, the LSP, the Survey, diagnostic dispositions — is scaffolding *for
future developers of DeluluLang* (AI and human), not the language itself. Green tests only prove
the tests still pass; they do not prove the language still decides the same things. The core is
the product; the tooling is not.

**How to apply:** re-running `cargo test --workspace` does not discharge this. Build the binary
from the commit *before* the tooling work (`git worktree add --detach <tmp> <baseline>`), then run
both binaries over the whole shipped corpus (`examples/` + `tests/conformance/{accept,reject}`)
across the core surfaces — `check`, `check --json`, `authority`, `authority --json`, `why`,
`fmt --check`, and `explain` for every allocated code — and diff stdout+stderr+exit code
byte-for-byte. Zero differences is the claim; any difference must be named as intended or fixed.
Harness kept at `scratchpad/coredif.ps1`. Remove the worktree afterwards
(`git worktree remove`) — see [[delulu-stale-worktree]] for why a forgotten one is a hazard.
