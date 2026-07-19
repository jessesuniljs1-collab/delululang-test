# The 1.0 release checklist

**Governs:** `STAGE9_SPECIFICATION.md` §8. This is the gate. A criterion is MET only when a named
test or a committed artifact says so — never because it looks done.

Status vocabulary: **MET** / **MET (local form)** / **NOT MET** / **PENDING-PUBLIC**.

---

## The ten acceptance criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | 100% conformance anchor coverage, all suites green across OSes/engines/custody/profiles | **NOT MET** | 273/290 (94.1%). `delulu-conform --coverage`; remainder classified in `STAGE9_BUILD_ORDER.md` D10. Suites green: Windows + Linux |
| 2 | Audit exploit set (F-1…F-6, R-7) permanent and re-verified | **MET** | `crates/delulu-check/tests/laundering.rs`; audit rules 7/7 covered in `docs/reference/audit-rules.md` |
| 3 | Studies A/B/C published with raw data; Study A injection catch 100% | **MET** | `measurements/` — A: 20/20 with validity fence + negative controls; B and C published as measured |
| 4 | Reproducible builds; provenance verifies; Scorecard ≥ floor | **MET (local form)** / **PENDING-PUBLIC** | Byte-identity witnessed by `criterion4_a_dwx_artifact_is_byte_identical_across_builds`; signatures live; SLSA L3 + Scorecard need public CI (D2, D5) |
| 5 | Registry live; publish→add→build round-trip; doctored line rejected | **MET (local form)** | `crates/delulu-registry` — 18 tests incl. `criterion5_*`; CDN hosting is a deployment act (D3) |
| 6 | Patch runbook rehearsed under target time, timeline recorded | **MET** | `docs/security/DRILL-001.md` — 5m28s end to end, and it found a real hole in the test suite |
| 7 | Book samples 100% CI-run; explain coverage 100% en-US | **MET** | `criterion7_every_book_sample_checks_clean`; `criterion7_every_code_has_a_long_form_explanation` |
| 8 | Fresh-machine first run under 5 minutes, three OSes | **NOT MET** | Windows + Linux only; macOS unverified (no Apple hardware — D8). Timed walkthrough not yet recorded |
| 9 | DL1801/DL1802 behave per §2.2 | **MET** | `crates/delulu/tests/stability_cli.rs` — 8 tests incl. the older-edition and unpinned skip branches |
| 10 | Announcement passes line-by-line honesty review, sign-off recorded | **MET (draft)** | `criterion10_the_announcement_makes_no_unsupported_claim`; sign-off below |

## The verdict

**1.0 DOES NOT SHIP YET.** Two criteria are NOT MET, and the checklist says so rather than rounding
them up:

- **Criterion 1** — 17 anchors still lack a witness. Each is classified; none is unknown. The work
  is bounded and listed.
- **Criterion 8** — the fresh-machine walkthrough has not been performed and timed on any OS, and
  macOS cannot be verified here at all.

A checklist that only has checkmarks is a wish list. These two are the honest state, and they are
what stands between here and a release.

## Blocked on public hosting (D2)

Not failures — they cannot be done from a private repository, and each ships as written policy plus
committed configuration that activates on publication:

- Branch protection and two-person review *(enforced procedurally today: the agent that writes a phase never commits it)*
- Signed commits and tags (Sigstore gitsign — needs OIDC identity)
- SLSA L3 attestation (needs a hosted, attested builder)
- OpenSSF Scorecard floor (needs a public repository to score)
- `security@` disclosure address and its PGP key

## Release procedure

1. `cargo test --workspace` — zero failures, on every OS available.
2. `cargo run -p delulu-conform -- --coverage` — record the number; it goes in the announcement.
3. `cargo run -p delulu-conform -- --check-reference` — the reference must not be stale.
4. `cargo run -p delulu -- fmt --check examples docs/book/samples`.
5. `cargo run -p delulu-measure -- all` — regenerate every study from a clean checkout.
6. Build release artifacts twice, in independent clones; **diff the bytes**.
7. Sign every artifact; verify each signature from a clean checkout **before** publishing.
8. Fill the digests in `PROVENANCE-1.0.json` and the SBOM's version.
9. Re-read the announcement against Constitution §9, line by line, with the measurements open.
10. Tag, publish, update the registry index.

## Honesty review sign-off

**Reviewed:** `ANNOUNCEMENT-1.0.md`, line by line, against Constitution §9.

**Method:** every quantitative claim was traced to a file in `measurements/` or to a named test.
Claims that could not be traced were deleted, not softened.

**Findings:**

- The performance section states *"not competitive with C"* explicitly, with the measured range.
  An earlier draft omitted performance entirely; omission is quieter than a false claim but it is
  the same failure, so the section was added.
- The repair-coverage figure appears as **8.5%** with the zero-reached-green decomposition, rather
  than as "typed repairs" unqualified.
- Conformance coverage appears as **273/290**, not as "comprehensive".
- The "what we found by looking" section was added deliberately. A project that publishes only what
  flatters it has trained its readers to discount everything it publishes.

**Mechanized:** `criterion10_the_announcement_makes_no_unsupported_claim` fails the build if a
forbidden claim appears *or* if an inconvenient measured result goes missing. Both directions,
because the likelier failure is quiet omission rather than a loud lie.

**Sign-off:** recorded here as part of the release gate. The announcement remains a **DRAFT** until
criteria 1 and 8 are met.
