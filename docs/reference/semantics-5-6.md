<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.6 Packages — declared, versioned authority

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.packages.manifest-bounds-the-code`

A package's code may not perform an effect its own manifest does not permit.

- **Enforced by:** `DL1009`
- **Coverage:** covered
- **Note:** The manifest is a ceiling the code is checked against, never a claim taken on trust.

## `ref.rule.packages.dependency-authority-is-pinned`

A dependency may not exceed the authority pinned for it in the lockfile; a changed authority under an unchanged version is refused.

- **Enforced by:** `DL1001`, `DL1002`, `DL1010`
- **Coverage:** covered
- **Note:** This is the xz scenario, refused mechanically: a patch release that adds an effect anywhere in the graph fails the build before it runs.

## `ref.rule.packages.semver-authority`

Widening a package's authority requires a major version bump.

- **Enforced by:** `DL1003`
- **Coverage:** covered
- **Note:** Authority is part of the public interface, so it obeys semver like any other part.

## `ref.rule.packages.sources-are-pinned`

A git dependency must pin a rev or tag, one package name resolves to one source, and a locked build requires a lockfile.

- **Enforced by:** `DL1007`, `DL1008`, `DL1011`
- **Coverage:** covered

## `ref.rule.packages.no-cycles`

Package and re-export graphs are acyclic.

- **Enforced by:** `DL1005`
- **Coverage:** covered

