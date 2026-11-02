---
name: installed-toolchain-stale-2026-11-02
description: "The laptop's own `delulu` on PATH (~/.cargo/bin, 2026-08-07) and its VS Code extension (1.0.0, 2026-08-07) were three months behind master on 2026-11-02 — CI and tests use fresh builds, so nothing noticed"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-02T13:33:01.559Z
---

Found 2026-11-02 while touring every surface for Jesse: `C:\Users\jesse\.cargo\bin\delulu.exe` dated 2026-08-07, and
the installed extension `delulu-lang.delulu-lang@1.0.0` (installed 2026-08-07). VS Code's user `settings.json` sets no
`delulu.serverPath`, and the extension's default is `delulu` from PATH — so the editor's language server is that same
August binary. `--version` prints `1.0.0` for it AND for a fresh master build (only `launch/2.0.0` says 2.0.0), so
the version cannot tell them apart. Recorded as RW 6.53 (doctor could compare; needs a build identity first).

**Why:** Jesse uses the terminal and VS Code himself; three months of work (the Windows sandbox, Map, http methods,
every fix) is invisible to him there, while every test I run uses `target/`.

**How to apply:** at a laptop session start, check `which delulu` and its date beside `git log -1 --format=%cd`.
Updating it (`cargo install --path crates/delulu --locked`, then the `.vsix` from `editors/vscode`) was OFFERED to
Jesse on 2026-11-02 and not yet approved — do it only on his word. Related: [[readiness-includes-everyday-language]],
[[session-start-github-sync]].
