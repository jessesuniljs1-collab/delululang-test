---
name: grammar-extension-keeps-old-misreadings
description: "Extending a grant grammar (net=HOST → HOST:METHODS, D-V2-182, 2026-10-23): a spelling the old grammar misread must stay misread the same way — `h.com:8443` kept its whole head as the host (matches nothing), else the broker's host check would newly match it; and a new dimension's default must equal what every old grant already allowed (no methods = GET alone)"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-23T10:41:28.346Z
---

Adding methods to `net` entries (`HOST[:METHODS][/PATTERN]`, D-V2-182). `split()` was shared by every reader, including
the broker's host-level use check (`host_matches(arg, host_of(entry))`). If a suffix that does not parse as methods were
simply cut off, `h.com:8443` would start matching `h.com` in that check, while the egress client's full judge refused
it. That is a record of `allow` before the check it trusts ([[gate-records-before-the-check]]). The fix: `read()` keeps
an unparsed suffix inside the host, so the entry reads exactly as it did before (a host string no request names), and
`check_entry` refuses it in words at every grant parser.

The same slice set the new dimension's default to what every existing grant already allowed: an entry naming no methods
is `{GET}`, since GET was the only request there was. The POST primitive (D-V2-183) then widens no grant ever written.
That is the semver-authority law in practice: the dimension comes first, the primitive after.

**Why:** a grammar extension changes the meaning of some strings that were already in the world (manifests, stored
grants, the broker's tree). The two failure modes are a malformed old string that starts to match, and an old valid
string that starts to reach more.

**How to apply:** when a grant, scope or wire grammar gains syntax, list every reader of the field (`grep` the split
function) and decide, for a string the old grammar accepted, (a) what each reader now reads and (b) whether that can
reach more. Keep both unchanged. Make every set have one spelling (`:GET` alone refused, methods in a fixed order), or
⊑-equivalent sets will canonicalize and hash differently. Related: [[static-verdict-skips-a-dimension]],
[[delulu-reproduce-the-shape]].
