---
name: witness-a-denial-on-the-far-side
description: "Cloud review 2026-11-05 — an exit/DL code says what the host decided, not that nothing left; a logging HTTPS server (test CA via SSL_CERT_FILE, Linux) and a frame-logging tee before the device process proved no forbidden request/command arrived"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-05T15:23:18.643Z
---

Where a refusal is the claim, read what the other side received.

**Why:** an exit code and a DL code report the host's decision; they don't show that nothing left. Claude's cloud
review (2026-11-05) put a logging HTTPS server behind the egress client — it saw exactly the granted requests (`GET
/v1/ok` ×3, `POST /v1/ok`, `GET /redir`), none of the refused ones (path outside scope, missing method, `..`, redirect
to another host, plain http). A frame-logging tee in front of `delulu device sim` saw only in-envelope commands (42.5,
80.0), none from a one-bit-tampered driver (DL1504, 0 frames). Sentinel files' SHA-256 before/after did the same for
out-of-scope writes.

**How to apply:** for every denial witness, add the far-side reading (server log, device frames, file hash). Linux
trick: a self-signed leaf is refused; a test CA + a leaf it signed, `SSL_CERT_FILE` pointing at the CA, makes a release
build trust a local server. NOT established on Windows/macOS (platform trust stores; cf. the D-V2-208 lesson).
Related: [[discovery-over-checklist]], [[cloud-reviews-2026-11-05]].
