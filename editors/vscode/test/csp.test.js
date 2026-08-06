// The Content-Security-Policy applied to the atlas webview.
//
// This is tested separately because the first version was fail-open: it used
// `html.replace(/<head>/i, …)`, and `String.replace` with no match returns the input UNCHANGED.
// The day `delulu atlas` emitted a document without a literal `<head>` — an XML declaration first,
// an attribute on the tag, anything — the page would have been shown in a webview with no policy at
// all, and everything would have looked like it worked. A security control that disappears quietly
// is worse than one that was never added, because the code reads as though it is there.
//
// The function therefore returns `undefined` rather than an un-policied string, and the caller
// refuses to open the panel.

const { test } = require("node:test");
const assert = require("node:assert");

// The module under test is the extension itself, which requires the `vscode` API. Rather than stub
// that whole surface, the function is re-declared here from the same source shape and the real one
// is checked to match — see the last test.
const fs = require("node:fs");
const path = require("node:path");

const SRC = fs.readFileSync(path.join(__dirname, "..", "extension.js"), "utf8");

/// Extract `withContentSecurityPolicy` from extension.js and evaluate it in isolation.
function loadFn() {
  const start = SRC.indexOf("function withContentSecurityPolicy");
  assert.notStrictEqual(start, -1, "withContentSecurityPolicy is gone from extension.js");
  // Balance braces from the function's opening `{`.
  let i = SRC.indexOf("{", start);
  let depth = 0;
  let end = i;
  for (; end < SRC.length; end++) {
    if (SRC[end] === "{") depth++;
    else if (SRC[end] === "}") {
      depth--;
      if (depth === 0) break;
    }
  }
  // eslint-disable-next-line no-new-func
  return new Function(`${SRC.slice(start, end + 1)}; return withContentSecurityPolicy;`)();
}

const withCsp = loadFn();

test("a policy is inserted immediately after <head>", () => {
  const out = withCsp("<html><head><title>x</title></head><body>hi</body></html>");
  assert.ok(out, "expected a policed document");
  assert.match(out, /<head><meta http-equiv="Content-Security-Policy"/);
  assert.match(out, /default-src 'none'/);
  assert.ok(out.includes("<title>x</title>"), "the original document survives");
});

test("a <head> carrying attributes is still handled", () => {
  const out = withCsp('<html><head lang="en" data-x="1"><title>x</title></head></html>');
  assert.ok(out, "expected a policed document");
  assert.match(out, /<head lang="en" data-x="1"><meta http-equiv="Content-Security-Policy"/);
});

// The fail-open case, stated as a test.
test("a document with no <head> yields undefined, never an un-policied string", () => {
  for (const bad of [
    "<html><body>no head at all</body></html>",
    "just some text",
    "",
    "<HTML><BODY>shouting</BODY></HTML>",
  ]) {
    const out = withCsp(bad);
    assert.strictEqual(out, undefined, `${JSON.stringify(bad.slice(0, 30))} must not be policed`);
  }
});

test("case is ignored, because generated markup is not required to be lowercase", () => {
  const out = withCsp("<HTML><HEAD><TITLE>x</TITLE></HEAD></HTML>");
  assert.ok(out, "expected a policed document");
  assert.match(out, /Content-Security-Policy/);
});

// The whole point is that the caller cannot accidentally use an un-policied document.
test("the caller refuses to open a panel when no policy could be applied", () => {
  const at = SRC.indexOf("const policed = withContentSecurityPolicy(html);");
  assert.notStrictEqual(at, -1, "showAtlas no longer routes through withContentSecurityPolicy");
  const after = SRC.slice(at, at + 800);
  assert.ok(
    after.includes("if (!policed)") && after.indexOf("return;") < after.indexOf("createWebviewPanel"),
    "showAtlas must return BEFORE creating the panel when the policy could not be applied"
  );
  assert.ok(
    after.includes("panel.webview.html = policed"),
    "the panel must be given the policed document, not the raw one"
  );
});
