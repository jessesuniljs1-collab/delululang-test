# The 2.0 release checklist

**Governs:** the owner's commission of 2026-10-17 (`docs/design/DeluluLang_Autonomous_Development_Prompt.md`,
*Release decision*): *"Keep the release process gated on objective readiness"* and *"Do not claim release readiness
merely because CI is green. Confirm that the actual release artefacts, metadata, licence obligations and security claims
are correct."* This page is that gate for 2.0.0, the first published release (D-V2-149, D-V2-155). A criterion is
**MET** only when a named test, a read CI run or a recorded command says so — never because it looks done.

**Companion:** [`CHECKLIST-1.0.md`](CHECKLIST-1.0.md) — the 1.0 gate, which refused once before it said yes. 1.0.0 was
tagged and never distributed; 2.0.0 is the first release anyone can download.

Status vocabulary: **MET** / **PENDING** (decided at the launch, by the step named) / **NOT MET**.

---

## The criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | The whole suite green on every OS the project claims | **MET** on the prepared tree; **PENDING** on the tagged tree | CI `jobs=everything` on `launch/2.0.0`, every job read: run `39055892355` at `f34fea4` and run `39058514393` at `130193a` (2026-10-17), and the newest, run `40185296917` at `0a55a44` and run `40191260368` at `f0cb42b` (2026-10-25, re-cited 2026-10-26) — **18 of 18 jobs success in each**, the Linux, Windows, macOS and arm64 test jobs among them. WSL2 (ext4) 2026-10-17: 2,311 passed, 0 failed. The tagged tree's own run is read before the tag (procedure step 2). |
| 2 | Conformance: every anchor covered; the reference in sync; samples formatted | **MET** | 2026-10-17 on the launch tree: `delulu-conform --coverage` **355/355 (100%)** — diag 154, prim 84, grammar 27, audit 7, cli 29, rule 54; `--check-reference` in sync (24 chapters); `delulu fmt --check examples docs/book/samples` 0 would change, 26 clean. |
| 3 | The hardening gates: the laundering exploit set, fuzz, Miri, the formal models, the heavy gates | **MET** on the prepared tree; **PENDING** on the tagged tree | Runs `39055892355` and `39058514393`, and the newest, `40185296917` and `40191260368` (2026-10-25): lints, fuzz, formal, heavy-gates, miri (diag, atlas), miri-ffi and miri-slow (syntax, check, broker) success — the broker's Miri run took 2 h 54 min and 2 h 13 min, then 1 h 51 min and 3 h 12 min (after #69 made its methods test judge each entry once; the cap is 240 min); `crates/delulu-check/tests/laundering.rs` is in the suite. |
| 4 | The supply chain checked | **MET** on the prepared tree; **PENDING** on the tagged tree | Runs `39055892355` and `39058514393`, and the newest, `40185296917` and `40191260368` (2026-10-25): `supply-chain` (`cargo deny check`) success. **The advisory database moves under a fixed tree:** on 2026-10-21 three new advisories (RUSTSEC-2026-0378 and 0379 against `wasmtime` 48.0.6, 0382 against `rustls-webpki` 0.103.15) turned the gate red on every tree before `5470b53`, which updated both; on `6863bf7` run `39562457688`'s `supply-chain` success. On 2026-10-23 it moved again: RUSTSEC-2026-0385 against `ctutils` 0.4.2 (the constant-time comparisons ML-DSA signs with) turned it red on every tree before `269237d`, which updated it to 0.4.5 (run `39930247901`'s `supply-chain` success). So the tagged tree's own run is read before the tag (procedure step 2), as criterion 1's is. |
| 5 | One version everywhere, with its release notes | **MET** | `Cargo.toml`/`Cargo.lock` 2.0.0; the VS Code extension 2.0.0 (`editor_contract`); `CHANGELOG.md` `## [2.0.0]` with the breaking changes (`distribution`); `release-build.yml` refuses a tag without its section. The heading's date is set in the commit the tag names (procedure step 3). |
| 6 | Licence obligations met in the tree and in every archive | **MET** in the tree; **PENDING** in the archives | Apache-2.0 `LICENSE` and `NOTICE`; `THIRD-PARTY-NOTICES.txt` generated from `Cargo.lock` into each archive (D-V2-143, `distribution`); no kernel image is committed or shipped (D-NE-27, D-V2-144). Each published archive is listed and checked for `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES.txt`, the SBOM, and no kernel image (procedure step 8). |
| 7 | Every security claim backed, none ahead of its proof | **MET** in the tree; **PENDING** for the attestation | `SECURITY.md` decides every control (D-V2-154; `governance` test, watched red first); the public repository's private vulnerability reporting, secret scanning, push protection and Dependabot alerts on and read back (D-V2-153). SLSA Level 2 is claimed; Level 3 only once `gh attestation verify --signer-workflow` passes for every archive (procedure step 8). |
| 8 | The release artefacts are what they claim | **PENDING** | Rehearsed: the release dry run `39058516429` on `130193a` built, packaged and installed the four targets at 2.0.0 (success; its release job skipped — no tag); the newest, `40191262135` on `f0cb42b` (2026-10-25), the same, the four `package` jobs success. For the release: built by `release-build.yml` from the tag on GitHub's runners, four targets: each archive is for its target, its binary names target and commit, `scripts/check-install.sh` installs it the way `INSTALL.md` says. After the run: `sha256sum -c SHA256SUMS`, one archive unpacked and run here, and every attestation verified (procedure step 8). |
| 9 | The editor works on a real install | **MET** | 2026-10-17: `node editors/vscode/e2e.js` on Windows 11 with a freshly packaged `.vsix` — activated, the language server started, no errors; CI `editor` success. |
| 10 | The microVM layer runs where it claims, and its image is never shipped | **MET** | 2026-10-17, WSL2 with KVM, an image rebuilt from source (`vmlinux` `f19758e4…6fad`, `initramfs.cpio` `3272f0c6…3121`): `microvm_cli` 10/10 (its root-only test 1/1 as root), `microvm_criterion8` 1/1; CI `microvm` and `microvm-reproducible` success. The image stayed on the laptop. |
| 11 | The public history is fresh, clean and the tagged tree | **PENDING** | The snapshot commit has no parent, the no-reply author, and a message naming the archive; its tree hash equals the tagged launch tree's; `git grep -i` for the banned word finds nothing in it (procedure step 4). |
| 12 | The development history preserved and recoverable | **MET** | `delululang-test` keeps every ref and becomes a private archive, never deleted (D-V2-153). A bundle of every ref on the laptop: `D:\nelan\DeluluLang-history-2026-10-17.bundle`, 14,095,723 bytes, sha256 `a62498fc…f353e7`, `git bundle verify` okay (2026-10-17, 16 refs, `launch/2.0.0` at `3f652cf`). |
| 13 | The documents say what is true at 2.0 | **MET** | `SUPPORT_MATRIX.md` rewritten for 2.0 (D-V2-155: 2.0.x the first published release and LTS; 1.0.x never distributed, unsupported); every "nothing is released" statement made true for 2.0 — `README.md` (its status table and *Installing*), `docs/QUESTIONS.md` item 13, the Book's maintainer list, `REMAINING_WORK.md` 7.5, the VS Code extension's install text; the `repository` fields of `Cargo.toml` and the extension's `package.json`, which named `github.com/delulu-lang/delulu` — an address that is not this project's — now name `jessesuniljs1-collab/DeluluLang`; `evidence_claims`, `editor_contract` and `repository_structure` tests. |

## The verdict

**Not yet decided.** The rows marked PENDING close at the launch, in the order below, each with the evidence it names; a
row that fails stops the launch and is recorded here as NOT MET, with what was done about it. The verdict is written
after step 8, before the draft is published.

## Release procedure

**Steps 4–10 wait for the owner's word** (2026-10-17, night: *"Only push to and use test repo until I tell u to change.
Do not change repo, unless told by me"*). Until he says so, only steps 1–3 run, on `delululang-test`.

1. `master` merged INTO `launch/2.0.0` (never the other way); the gates re-run there (`survey build`, `check`,
   `findings` 0 errors; `doctor --check`).
2. CI `jobs=everything` on the launch branch's final commit — every job read, success (criteria 1, 3, 4).
3. The CHANGELOG heading dated (`## [2.0.0] — YYYY-MM-DD`, the UTC date of the tag), with this page and
   `SUPPORT_MATRIX.md` in the same commit.
4. The snapshot: one commit with no parent, that tree, pushed to `DeluluLang` `main`; its tree hash compared with the
   launch commit's; the banned-word check on it (criterion 11).
5. `main` protected: no force-push, no deletion.
6. The Actions variable `RELEASES=on` on `DeluluLang`, then the tag `v2.0.0` on the snapshot commit.
7. `release.yml` watched to the end: four archives, four attestations, the install gate, a **draft** release.
8. Verified from the draft: `sha256sum -c SHA256SUMS`; `gh attestation verify <archive> -R jessesuniljs1-collab/DeluluLang
   --signer-workflow jessesuniljs1-collab/DeluluLang/.github/workflows/release-build.yml` for each archive; each archive's
   listing (criterion 6); the Windows archive unpacked and `delulu --version` run (criteria 6, 7, 8).
9. The verdict above written; the draft published.
10. The laptop's `origin`, the routine's repository and prompt moved to `DeluluLang`; `delululang-test` made private.
