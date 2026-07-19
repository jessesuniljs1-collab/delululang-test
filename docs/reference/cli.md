<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::CLI_SUBCOMMANDS` (fenced against the `delulu` dispatch).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# CLI contracts

Every subcommand of the `delulu` binary. Exit codes and the `--json` envelopes are part of the stability contract from 1.0 (invariant 43).

A subcommand's **accepting** witness is a valid invocation that succeeds; its **rejecting** witness is an invalid one that must refuse with a nonzero exit and an honest message — never a silent success, never a panic.

> **Coverage (invariant 42):** 25 of 25 anchors in this chapter have both an accepting and a rejecting conformance witness (100.0%). Items marked otherwise are **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.

| Subcommand | Anchor | Coverage |
|---|---|---|
| `delulu check` | `ref.cli.check` | covered |
| `delulu fmt` | `ref.cli.fmt` | covered |
| `delulu test` | `ref.cli.test` | covered |
| `delulu lsp` | `ref.cli.lsp` | covered |
| `delulu keygen` | `ref.cli.keygen` | covered |
| `delulu sign` | `ref.cli.sign` | covered |
| `delulu verify-sig` | `ref.cli.verify-sig` | covered |
| `delulu publish` | `ref.cli.publish` | covered |
| `delulu add` | `ref.cli.add` | covered |
| `delulu login` | `ref.cli.login` | covered |
| `delulu build` | `ref.cli.build` | covered |
| `delulu lock` | `ref.cli.lock` | covered |
| `delulu run` | `ref.cli.run` | covered |
| `delulu plugin` | `ref.cli.plugin` | covered |
| `delulu authority` | `ref.cli.authority` | covered |
| `delulu why` | `ref.cli.why` | covered |
| `delulu atlas` | `ref.cli.atlas` | covered |
| `delulu repl` | `ref.cli.repl` | covered |
| `delulu audit` | `ref.cli.audit` | covered |
| `delulu grants` | `ref.cli.grants` | covered |
| `delulu guard` | `ref.cli.guard` | covered |
| `delulu broker` | `ref.cli.broker` | covered |
| `delulu secrets` | `ref.cli.secrets` | covered |
| `delulu locale` | `ref.cli.locale` | covered |
| `delulu explain` | `ref.cli.explain` | covered |
