# The 1.0 release checklist

**Governs:** `STAGE9_SPECIFICATION.md` §8. This is the gate. A criterion is MET only when a named
test or a committed artifact says so — never because it looks done.

Status vocabulary: **MET** / **MET (local form)** / **NOT MET** / **PENDING-PUBLIC**.

---

## The ten acceptance criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | 100% conformance anchor coverage, all suites green across OSes/engines/custody/profiles | **MET** | **287/287 (100%)** at the gate (`delulu-conform --coverage`, ruling D22: Class B produced from real emission sites, Class C constructor-level, 3 dead codes retired pre-freeze); `release_requires_full_coverage` now a hard per-commit gate; ratchet floor 287. Suites green: Windows 926/0/4, Linux 930/0/4 |
| 2 | Audit exploit set (F-1…F-6, R-7) permanent and re-verified | **MET** | `crates/delulu-check/tests/laundering.rs`; audit rules 7/7 covered in `docs/reference/audit-rules.md` |
| 3 | Studies A/B/C published with raw data; Study A injection catch 100% | **MET** | `measurements/` — A: 20/20 with validity fence + negative controls; B and C published as measured |
| 4 | Reproducible builds; provenance verifies; Scorecard ≥ floor | **MET (local form)** / **PENDING-PUBLIC** | Byte-identity witnessed by `criterion4_a_dwx_artifact_is_byte_identical_across_builds`; signatures live; SLSA L3 + Scorecard need public CI (D2, D5) |
| 5 | Registry live; publish→add→build round-trip; doctored line rejected | **MET (local form)** | `crates/delulu-registry` — 18 tests incl. `criterion5_*`; CDN hosting is a deployment act (D3) |
| 6 | Patch runbook rehearsed under target time, timeline recorded | **MET** | `docs/security/DRILL-001.md` — 5m28s end to end, and it found a real hole in the test suite |
| 7 | Book samples 100% CI-run; explain coverage 100% en-US | **MET** | `criterion7_every_book_sample_checks_clean`; `criterion7_every_code_has_a_long_form_explanation` |
| 8 | Fresh-machine first run under 5 minutes, three OSes | **MET (two OSes; macOS unverifiable per D8)** | D9 drill performed and recorded (`measurements/first-run/RECORD.md`): Windows **1.115 s**, Linux **0.019 s** for the Book's Chapter-1 journey on pristine homes; what the clock includes/excludes is stated. macOS: no Apple hardware — stated, not extrapolated |
| 9 | DL1801/DL1802 behave per §2.2 | **MET** | `crates/delulu/tests/stability_cli.rs` — 8 tests incl. the older-edition and unpinned skip branches |
| 10 | Announcement passes line-by-line honesty review, sign-off recorded | **MET (draft)** | `criterion10_the_announcement_makes_no_unsupported_claim`; sign-off below |

## The verdict

**1.0 SHIPS (2026-07-20).** The gate said no first, and that history stays on this page: at
`1.0.0-rc.1` criteria 1 and 8 were **NOT MET** — coverage stood at 273/290 and the fresh-machine
walkthrough had never been performed or timed — and the checklist said "1.0 DOES NOT SHIP YET" in
those words rather than rounding up (D19). Both blockers were then closed by work:

- **Criterion 1** — the 17 classified anchors closed honestly (D22): produced witnesses,
  constructor-level witnesses per D10's ruling, and three dead codes retired pre-freeze. Coverage
  is 287/287 and `release_requires_full_coverage` is a hard gate from here on.
- **Criterion 8** — the D9 drill was performed and recorded on both available OSes, with the
  clock's inclusions and exclusions stated (`measurements/first-run/RECORD.md`). macOS remains
  honestly unverified (D8).

A checklist that only has checkmarks is a wish list; this one refused once, and that refusal is
why its yes means something.

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
- The repair-coverage figure appears as **8.3%** (4 of 48 at the release-gate regeneration; the
  corpus grew by one D22 fixture, so the earlier 8.5% was restated, not defended) with the
  zero-reached-green decomposition, rather than as "typed repairs" unqualified.
- Conformance coverage appears as **287 of 287** — with the D22 retirements stated in the same
  section, because a 100% reached partly by removing dead codes is a fact the reader weighs, and
  the mechanized review now *requires* both the exact number and the word "retired".
- The OS section is tiered by evidence: Windows and Linux carry suite numbers; macOS says
  "expected, unverified" because no Apple hardware has ever run the suite; ports are invited
  through the open-source RFC process rather than promised.
- The first-run numbers cite the drill record, which states what the clock includes and excludes.
- The principle section ("freedom with authority; freedom with responsibility") ties every
  sentence to a mechanism this repository actually contains; the society metaphor is labeled a
  metaphor.
- The "what we found by looking" section was added deliberately. A project that publishes only what
  flatters it has trained its readers to discount everything it publishes.

**Mechanized:** `criterion10_the_announcement_makes_no_unsupported_claim` fails the build if a
forbidden claim appears *or* if an inconvenient measured result goes missing. Both directions,
because the likelier failure is quiet omission rather than a loud lie.

**Sign-off:** recorded here as part of the release gate, 2026-07-20, with criteria 1 and 8 closed
and the full re-review performed against the updated announcement. The announcement's status line
reads RELEASE; the rc.1 refusal that preceded it stays recorded above.
