---
name: a-fast-path-must-see-what-the-slow-path-sees
description: "Hidden-complexity audit 2026-11-05 — the LSP checks one file alone (false DL0301 on the shipped greeter, RW 6.72), edit --if-checks is blind in packages both ways (RW 6.69), build --target wasm accepts a fn main() its host cannot call (RW 3.28)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: session_01Deh6W9bJdfxAeMARtXU9R9
  modified: 2026-11-05T12:00:00.000Z
---

A fast or machine path that does what another command does must see what that command sees.

**Why:** three of the hidden-complexity audit's findings were one habit. A component built for a narrower case was
reused where the work is wider:
- `delulu lsp` checks each open file alone (`lsp.rs` line 204). The shipped `examples/greeter/src/main.delulu` shows
  two false DL0301 errors that `delulu check examples/greeter` does not.
- `edit --if-checks` checks the edited file alone. It wrote a `Net` widening the package then refused (DL1009), and it
  refused a correct edit that calls an imported name.
- The WASM host always calls `main(root)` with no result (`host.rs` line 3018). `build --target wasm` exits 0 on
  `fn main() {}`, and every run of that `.dwx` is DL0904.

`docs/for-agents.md` recommended the first of these as the agent's warm loop.

**How to apply:** when a path is offered as the fast or machine way to do what another command does, keep one witness
that compares their answers:
- on a multi-module package, not only on one file;
- on every `main` shape the checker accepts.
