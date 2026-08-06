// Resolving `delulu.serverPath` to a concrete executable.
//
// This lives in its own module, with no dependency on the `vscode` API, for one reason: it is the
// piece of the extension that decides **which program gets launched**, and that decision should be
// testable without starting an editor. `test/resolve.test.js` exercises it directly.
//
// The security property it exists to hold: a bare or relative name is never handed to the OS to
// resolve. On Windows `CreateProcess` searches the *current directory* before `PATH`, so a bare
// name delegated to `spawn` can be satisfied by a file sitting in whatever directory the process
// happens to be standing in. Doing the lookup here makes the outcome a function of this code rather
// than of the host's working directory.

const fs = require("fs");
const path = require("path");

/// Every filename to try for one candidate location.
///
/// On Windows an executable is `delulu.exe`, not `delulu`, and which suffixes count is `PATHEXT`.
/// A name that already carries an extension is honoured as written first, so an operator who points
/// at `delulu.cmd` gets `delulu.cmd`.
function variants(base, exts) {
  const out = [];
  if (path.extname(base)) {
    out.push(base);
  }
  for (const e of exts) {
    out.push(base + e.toLowerCase());
  }
  return out;
}

function firstExistingFile(candidates) {
  for (const c of candidates) {
    try {
      if (fs.statSync(c).isFile()) {
        return c;
      }
    } catch {
      // Unreadable or absent — both mean "not this one". A directory named `delulu` on PATH is
      // skipped rather than returned, which `isFile()` is doing here.
    }
  }
  return undefined;
}

/// Resolve a configured server command.
///
/// Returns `{ path }` with an **absolute** path, or `{ error, missingFromPath? }` with a sentence
/// fit to show a person. `env` is injectable so the tests can describe a PATH without touching the
/// machine's own.
function resolveServer(configured, env = process.env, platform = process.platform) {
  const isWin = platform === "win32";
  const exts = isWin
    ? (env.PATHEXT || ".EXE;.CMD;.BAT").split(";").filter(Boolean)
    : [""];

  let spelled = String(configured == null ? "" : configured).trim();
  if (spelled === "") {
    return { error: "`delulu.serverPath` is empty." };
  }

  if (spelled.startsWith("~/") || spelled.startsWith("~\\")) {
    const home = env.HOME || env.USERPROFILE;
    if (home) {
      spelled = path.join(home, spelled.slice(2));
    }
  }

  if (spelled.includes("/") || spelled.includes("\\")) {
    if (!path.isAbsolute(spelled)) {
      return {
        error:
          `\`delulu.serverPath\` is the relative path \`${configured}\`. Relative paths are ` +
          `refused deliberately: they would be resolved against whatever directory this ` +
          `extension happens to be running in, which is not a location you chose. Use an ` +
          `absolute path.`,
      };
    }
    const hit = firstExistingFile(variants(spelled, exts));
    return hit
      ? { path: hit }
      : { error: `No file exists at \`delulu.serverPath\` (\`${configured}\`).` };
  }

  const dirs = (env.PATH || env.Path || "").split(path.delimiter).filter(Boolean);
  for (const dir of dirs) {
    const hit = firstExistingFile(variants(path.join(dir, spelled), exts));
    if (hit) {
      return { path: hit };
    }
  }
  return {
    error: `\`${configured}\` was not found on your PATH (${dirs.length} entries searched).`,
    missingFromPath: true,
  };
}

module.exports = { resolveServer };
