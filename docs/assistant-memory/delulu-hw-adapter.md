---
name: delulu-hw-adapter
description: "DeluluLang's first hardware adapter (D23) is an operator-supplied subprocess, NOT the signed plugin spec §5.4 describes — read before claiming hardware support"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-22T09:00:52.266Z
---

Shipped 2026-07-22 as build-order **D23** (commit `939a36e`), closing the third dish Jesse
commissioned. `Profile::Hw { adapter }` had existed only "so the artifact-hash gate has something
real to gate"; it now dispatches to a real driver.

**What it is:** an operator-supplied **subprocess** speaking a line protocol over stdio —
`CMD <device> <dim>=<v>,...` → `OK` | `ERR <reason>`, `READ <device>` → `VAL <f>` | `NODEV`.
Attached with `--adapter-cmd "<command>"` under `--broker-profile hw:<name>`.
Code: `crates/delulu-runtime/src/adapter.rs`; wiring in `device.rs`; tests in
`crates/delulu/tests/hw_adapter_cli.rs`.

**What it is NOT, and this must never be blurred:** spec §5.4 says hardware adapters are
Verified-class Stage-6 plugins with `require_signed: true`. This is a *different mechanism*. A
signed plugin would buy **supply-chain assurance** (you know who wrote the driver). `--adapter-cmd`
buys **isolation and reach** and carries **no signature check at all**. DL1905 approves the
*artifact* — the program's exact bytes — and says nothing about the driver. The signed-plugin path
remains unbuilt. Never describe D23 as satisfying §5.4.

**The load-bearing property, and how it was proven.** The envelope is enforced host-side, against
the grant, **before one byte reaches the driver** — so a driver can refuse *more* and can never
permit *more*. From inside DeluluLang "refused before dispatch" and "dispatched and rejected" look
identical, so the test reads **the driver's own log**: a program commanding 12° (in envelope) and
999° (out) leaves exactly **one** line in it. If you ever need to re-verify this property, that log
is the only evidence that works.

**Four fail-closed rules with witnesses:** a reply that is not exactly `OK` is a protocol error
(`"ok"`, `"ACK"`, `"1"`, `""` all tested, none accepted); a silent driver times out via a reader
thread + `recv_timeout` rather than wedging the loop; **a failed adapter stays poisoned** (a late
reply would otherwise answer the *next* command — how a robot executes yesterday's instruction); a
garbled or non-finite reading is an error, never `None` and never a number (invariant 50).

**Still true after D23:** no driver for any real device ships in-tree, every demonstration commands
the in-tree simulator, `STAGE10_AUTONOMY_ADDENDUM.md` §4's "no hardware ships" stands, and
certification is **none**. Running the adapter against a shell script proves the socket works, not
that anything physical moved.

Related: [[delululang-project]], [[delulu-federation-scope]], [[skip-branch-verification-rule]].
