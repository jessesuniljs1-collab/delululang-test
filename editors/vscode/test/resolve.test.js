// Tests for `server-resolve.js` — the module that decides which program the extension launches.
//
// Run with `npm test` (Node's built-in runner; no test framework is added to the dependency tree,
// because every dependency an editor extension carries is a dependency shipped to every user).
//
// These run on the real filesystem in a temporary directory rather than against a mocked `fs`. A
// mock would let the tests agree with a wrong idea of how lookup works — and the entire point of
// this module is that the *real* rules (PATHEXT, directories-are-not-executables, PATH ordering)
// are followed exactly.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const { resolveServer } = require("../server-resolve");

/// A throwaway directory tree; returns the root.
function sandbox(files) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "delulu-resolve-"));
  for (const [rel, kind] of Object.entries(files)) {
    const full = path.join(root, rel);
    fs.mkdirSync(path.dirname(full), { recursive: true });
    if (kind === "dir") {
      fs.mkdirSync(full, { recursive: true });
    } else {
      fs.writeFileSync(full, "");
    }
  }
  return root;
}

test("an absolute path that exists resolves to itself", () => {
  const root = sandbox({ "bin/delulu.exe": "file" });
  const target = path.join(root, "bin", "delulu.exe");
  const got = resolveServer(target, { PATH: "" }, "win32");
  assert.strictEqual(got.path, target);
});

test("an absolute path that does not exist is an error naming the setting", () => {
  const root = sandbox({});
  const got = resolveServer(path.join(root, "nope.exe"), { PATH: "" }, "win32");
  assert.ok(got.error, "expected an error");
  assert.match(got.error, /No file exists/);
  assert.strictEqual(got.path, undefined);
});

test("a bare name is found by walking PATH, in order", () => {
  const root = sandbox({ "a/delulu.exe": "file", "b/delulu.exe": "file" });
  const env = { PATH: [path.join(root, "b"), path.join(root, "a")].join(path.delimiter), PATHEXT: ".EXE" };
  const got = resolveServer("delulu", env, "win32");
  assert.strictEqual(got.path, path.join(root, "b", "delulu.exe"), "first PATH entry must win");
});

test("a bare name missing from PATH reports how many places were searched", () => {
  const root = sandbox({ "a/somethingelse.exe": "file" });
  const got = resolveServer("delulu", { PATH: path.join(root, "a"), PATHEXT: ".EXE" }, "win32");
  assert.ok(got.error);
  assert.strictEqual(got.missingFromPath, true);
  assert.match(got.error, /not found on your PATH/);
});

// The security property, stated as a test.
//
// A *relative* path is refused rather than resolved. Resolving it would mean resolving against the
// extension host's working directory — a directory the user did not choose and cannot see — and on
// Windows a bare name delegated to `CreateProcess` is searched for in the current directory before
// PATH. Both are how a file sitting in an opened repository gets launched as the toolchain.
test("a relative path is refused, never resolved against the current directory", () => {
  const root = sandbox({ "delulu.exe": "file" });
  const cwd = process.cwd();
  try {
    process.chdir(root); // stand exactly where the attack would want us to stand
    for (const spelling of ["./delulu.exe", ".\\delulu.exe", "sub/delulu.exe", "..\\delulu.exe"]) {
      const got = resolveServer(spelling, { PATH: "", PATHEXT: ".EXE" }, "win32");
      assert.ok(got.error, `${spelling} must not resolve`);
      assert.strictEqual(got.path, undefined, `${spelling} must not produce a path`);
      assert.match(got.error, /[Rr]elative paths are refused/);
    }
  } finally {
    process.chdir(cwd);
  }
});

test("a bare name is never satisfied by the current directory", () => {
  const root = sandbox({ "delulu.exe": "file" });
  const cwd = process.cwd();
  try {
    process.chdir(root);
    // PATH is empty, and `delulu.exe` is right here. The OS would find it; we must not.
    const got = resolveServer("delulu", { PATH: "", PATHEXT: ".EXE" }, "win32");
    assert.ok(got.error, "a bare name must not be satisfied by the working directory");
    assert.strictEqual(got.path, undefined);
  } finally {
    process.chdir(cwd);
  }
});

test("a directory named like the binary is skipped, not returned", () => {
  const root = sandbox({ "a/delulu.exe": "dir", "b/delulu.exe": "file" });
  const env = { PATH: [path.join(root, "a"), path.join(root, "b")].join(path.delimiter), PATHEXT: ".EXE" };
  const got = resolveServer("delulu", env, "win32");
  assert.strictEqual(got.path, path.join(root, "b", "delulu.exe"));
});

test("PATHEXT decides the suffix on Windows; POSIX uses the bare name", () => {
  const root = sandbox({ "w/delulu.cmd": "file", "p/delulu": "file" });
  const win = resolveServer("delulu", { PATH: path.join(root, "w"), PATHEXT: ".EXE;.CMD" }, "win32");
  assert.strictEqual(win.path, path.join(root, "w", "delulu.cmd"));

  const posix = resolveServer("delulu", { PATH: path.join(root, "p") }, "linux");
  assert.strictEqual(posix.path, path.join(root, "p", "delulu"));
});

test("an empty or whitespace setting is an error, not a spawn of nothing", () => {
  for (const bad of ["", "   ", null, undefined]) {
    const got = resolveServer(bad, { PATH: "" }, "win32");
    assert.ok(got.error, `${JSON.stringify(bad)} must be an error`);
    assert.strictEqual(got.path, undefined);
  }
});

test("~ expands against the home directory", () => {
  const root = sandbox({ "tools/delulu.exe": "file" });
  const got = resolveServer("~/tools/delulu.exe", { HOME: root, PATH: "", PATHEXT: ".EXE" }, "win32");
  assert.strictEqual(got.path, path.join(root, "tools", "delulu.exe"));
});
