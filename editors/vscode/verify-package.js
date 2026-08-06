#!/usr/bin/env node
// Verify a built .vsix would actually ACTIVATE — not merely that it built.
//
// Why this exists: the first .vsix this project produced packaged cleanly, reported
// "DONE Packaged: … (130 files)", and would have thrown `Cannot find module
// 'vscode-languageserver-protocol'` the moment a user opened a .delulu file. `npm install` had put
// 8 packages in node_modules; vsce shipped only the one named in `dependencies`. A green package
// step proved nothing about whether the thing inside runs.
//
// Usage:  node verify-package.js [path-to-vsix]     (default: delulu-lang.vsix)

const fs = require("fs");
const path = require("path");
const os = require("os");
const { execFileSync } = require("child_process");

const vsix = path.resolve(process.argv[2] || "delulu-lang.vsix");
if (!fs.existsSync(vsix)) {
  console.error(`no such .vsix: ${vsix}\nBuild one first:  npm run package`);
  process.exit(2);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "delulu-vsix-"));
try {
  // A .vsix is a zip. Node has no unzip, so use the platform's. On Windows `Expand-Archive`
  // dispatches on the FILE EXTENSION and refuses anything but `.zip`, so copy it under a name it
  // will accept rather than renaming the artifact itself.
  if (process.platform === "win32") {
    const asZip = path.join(tmp, "pkg.zip");
    fs.copyFileSync(vsix, asZip);
    execFileSync(
      "powershell",
      ["-NoProfile", "-Command", `Expand-Archive -LiteralPath '${asZip}' -DestinationPath '${tmp}' -Force`],
      { stdio: "pipe" }
    );
    fs.rmSync(asZip, { force: true });
  } else {
    execFileSync("unzip", ["-q", vsix, "-d", tmp], { stdio: "pipe" });
  }

  const extDir = path.join(tmp, "extension");
  const pkg = JSON.parse(fs.readFileSync(path.join(extDir, "package.json"), "utf8"));
  const entry = path.join(extDir, pkg.main);

  const problems = [];

  if (!fs.existsSync(entry)) {
    problems.push(`the manifest's \`main\` (${pkg.main}) is not in the package`);
  } else {
    // Collect every non-relative require in EVERY shipped .js file — including the files inside
    // node_modules — and check each names something the package can actually load.
    //
    // An earlier version of this check walked only the relative requires reachable from the entry
    // point. It reported OK on a .vsix that was genuinely broken: the entry required
    // `vscode-languageclient`, which was present, and the walk stopped there — never seeing that
    // the client's own files require `vscode-languageserver-protocol`, `semver` and `minimatch`,
    // none of which had been packaged. A check that stops at the first hop cannot find a missing
    // second hop, which is where this class of bug lives.
    const builtin = new Set(require("module").builtinModules);
    const externals = new Map(); // package name -> the shipped file that requires it

    const walk = (dir) => {
      for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
        const p = path.join(dir, e.name);
        if (e.isDirectory()) {
          walk(p);
        } else if (e.name.endsWith(".js") || e.name.endsWith(".cjs")) {
          const src = fs.readFileSync(p, "utf8");
          for (const m of src.matchAll(/require\(\s*["']([^"']+)["']\s*\)/g)) {
            const spec = m[1];
            if (spec.startsWith(".")) continue;
            // The `node:` scheme is reserved for built-in modules, so a `node:`-prefixed specifier
            // is a builtin by definition and needs no package on disk. Checking the prefix rather
            // than the name matters: `builtinModules` does NOT list the prefix-only builtins
            // (`node:test`, `node:sqlite`, …), so stripping the prefix and looking the name up
            // reported `node:test` as a missing third-party package — which is exactly what this
            // verifier did the first time a test file was accidentally packaged.
            if (spec.startsWith("node:")) continue;
            const bare = spec;
            // Keep the scope on scoped packages: `@scope/name`, otherwise the first segment.
            const parts = bare.split("/");
            const name = bare.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0];
            if (!externals.has(name)) externals.set(name, path.relative(extDir, p));
          }
        }
      }
    };
    walk(extDir);

    for (const [dep, requiredBy] of [...externals].sort()) {
      if (builtin.has(dep) || dep === "vscode") continue;
      if (!fs.existsSync(path.join(extDir, "node_modules", dep))) {
        problems.push(
          `\`${requiredBy}\` requires \`${dep}\`, which is not in the package — activation would throw`
        );
      }
    }
  }

  // Every command the client registers must be declared, or it is invisible in the palette.
  const entrySrc = fs.existsSync(entry) ? fs.readFileSync(entry, "utf8") : "";
  const declared = new Set((pkg.contributes?.commands || []).map((c) => c.command));
  for (const m of entrySrc.matchAll(/registerCommand\(\s*["']([^"']+)["']/g)) {
    if (!declared.has(m[1])) problems.push(`registers \`${m[1]}\` but the manifest does not declare it`);
  }

  if (problems.length) {
    console.error(`FAIL — ${path.basename(vsix)} would not work:`);
    for (const p of problems) console.error(`  • ${p}`);
    process.exit(1);
  }

  const size = (fs.statSync(vsix).size / 1024).toFixed(1);
  console.log(`OK — ${path.basename(vsix)} (${size} KB)`);
  console.log(`     entry ${pkg.main} resolves with no missing modules`);
  console.log(`     ${declared.size} command(s) declared and registered`);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}
