#!/usr/bin/env python3
"""Write THIRD-PARTY-NOTICES.txt for the portable `delulu` binary: every crate it links, its licence as its authors
declare it, and the licence and notice texts that travel with it (D-V2-143).

    python scripts/third-party-notices.py OUT [--target TRIPLE] [--sbom SBOM.json]

WHY. The archive's binary statically links a few hundred crates. MIT, BSD, ISC and Unicode licences require their
copyright and permission notices to accompany a binary distribution; Apache-2.0 requires its NOTICE files (section 4(d)).
`LICENSE` and `NOTICE` beside the binary are DeluluLang's own; this file is everyone else's.

WHAT IS COUNTED. Exactly the release build's graph — `cargo tree -p delulu --no-default-features --features net -e normal`
for the target — so neither a Python-capable build's crates nor a build script's tools (never linked) are listed. A
crate's texts are its own files (LICENSE*, LICENCE*, COPYING*, NOTICE*, COPYRIGHT*, at the top of the package, and the
licence files of C code a `-sys` crate bundles); where a crate ships none, the canonical text of its declared licence is
taken from the Rust toolchain's own copy (`<sysroot>/share/doc/rust/licenses/`) and the entry says so. A crate with
neither is a failure: the script exits 1 and names it, so no archive ships with a licence it could not show.

The Rust standard library, linked into every Rust binary, is included from the toolchain's own copyright notice.

`--sbom FILE` also writes the same graph as a CycloneDX 1.5 bill of materials: every linked crate with its version,
package URL, declared licence and the sha256 `Cargo.lock` pins for it — the SBOM of THIS archive, not of the workspace.
"""
import hashlib
import html
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LICENSE_FILE = re.compile(r"^(LICEN[CS]E|COPYING|NOTICE|COPYRIGHT|UNLICENSE)([-._].*)?$", re.IGNORECASE)
# C code a crate bundles into the binary, whose own licence is in a subdirectory.
BUNDLED = {"libffi-sys": ["libffi/LICENSE"], "ittapi-sys": ["c-library/LICENSES/BSD-3-Clause.txt"]}
# A crate that ships no licence text of its own, published from the same upstream repository as one that does.
SAME_PROJECT = {"ittapi": "ittapi-sys"}
# Where an expression offers a choice that includes a copyleft licence, the permissive side is the one used.
CHOSEN = {"GPL-2.0-only OR BSD-3-Clause": "BSD-3-Clause"}


def own_licence_files(pdir, chosen):
    """A package's licence and notice files: at its top, and in a REUSE `LICENSES/` directory — only the chosen
    licence's file there when the expression offered a choice (never the copyleft side's text)."""
    files = sorted(f for f in pdir.iterdir() if f.is_file() and LICENSE_FILE.match(f.name))
    reuse = pdir / "LICENSES"
    if reuse.is_dir():
        files += sorted(f for f in reuse.iterdir() if f.is_file() and (chosen is None or f.stem == chosen))
    return files


def run(args):
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True, encoding="utf-8").stdout


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    out = Path(sys.argv[1])
    target = sys.argv[sys.argv.index("--target") + 1] if "--target" in sys.argv else None
    if target is None:
        target = next(l.split(":", 1)[1].strip() for l in run(["rustc", "-vV"]).splitlines() if l.startswith("host:"))
    version = re.search(r'^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M).group(1)

    tree = run(["cargo", "tree", "-p", "delulu", "--no-default-features", "--features", "net", "-e", "normal",
                "--prefix", "none", "--format", "{p}|{l}", "--target", target])
    linked = {}
    for line in tree.splitlines():
        line = line.replace(" (*)", "").strip()
        if not line or "|" not in line:
            continue
        pkg, lic = line.split("|", 1)
        parts = pkg.split()
        linked[(parts[0], parts[1].lstrip("v"))] = lic.strip()

    meta = json.loads(run(["cargo", "metadata", "--format-version", "1"]))
    dirs = {(p["name"], p["version"]): p for p in meta["packages"]}
    sysroot = Path(run(["rustc", "--print", "sysroot"]).strip())
    canon_dir = sysroot / "share" / "doc" / "rust" / "licenses"
    canonical = {f.stem: f.read_text(encoding="utf-8", errors="replace") for f in canon_dir.glob("*.txt")} if canon_dir.is_dir() else {}

    texts = {}     # sha -> (text, [crates])
    entries = []
    missing = []
    for (name, ver), lic in sorted(linked.items()):
        p = dirs.get((name, ver))
        if p is None:
            missing.append(f"{name} {ver}: not in `cargo metadata`")
            continue
        if p.get("source") is None:
            entries.append(f"{name} {ver} — {lic} — this project (see LICENSE and NOTICE)")
            continue
        pdir = Path(p["manifest_path"]).parent
        own = own_licence_files(pdir, CHOSEN.get(lic))
        bundled = [pdir / rel for rel in BUNDLED.get(name, []) if (pdir / rel).is_file()]
        found = [(f"{name} {ver}: {f.relative_to(pdir).as_posix()}", f.read_text(encoding="utf-8", errors="replace"))
                 for f in own + bundled]
        how = "its own licence files"
        # A NOTICE alone is not a licence, and a bundled library's licence is not the crate's.
        sibling = SAME_PROJECT.get(name)
        if not any(not f.name.upper().startswith("NOTICE") for f in own) and sibling:
            sp = next((q for (n, _), q in dirs.items() if n == sibling), None)
            if sp is not None:
                sfiles = own_licence_files(Path(sp["manifest_path"]).parent, CHOSEN.get(lic))
                found += [(f"{name} {ver}, from {sibling} (the same upstream project, {p.get('repository') or ''}): "
                           f"{f.name}", f.read_text(encoding="utf-8", errors="replace")) for f in sfiles]
                own = sfiles
                how = f"none in the crate; its upstream project's text, as {sibling} ships it"
        if not any(not f.name.upper().startswith("NOTICE") for f in own):
            used = CHOSEN.get(lic, lic)
            ids = [i for i in re.split(r"\s+(?:OR|AND|WITH)\s+|/", used) if i]
            have = [i for i in ids if i in canonical]
            if not have:
                missing.append(f"{name} {ver}: ships no licence file, and `{used}` has no canonical text in {canon_dir}")
                continue
            authors = ", ".join(p.get("authors") or []) or "not declared in its Cargo.toml"
            how = ("no licence file in the crate; the canonical text of its declared licence, from the Rust toolchain; "
                   f"its copyright holders, as the crate declares them: {authors}")
            found += [(f"canonical {i}", canonical[i]) for i in have]
        if bundled:
            how += "; and the licence of the C code it bundles"
        chosen = f" (used under {CHOSEN[lic]})" if lic in CHOSEN else ""
        repo = p.get("repository") or ""
        entries.append(f"{name} {ver} — {lic}{chosen} — {repo} — texts: {how}")
        for label, text in found:
            h = hashlib.sha256(text.strip().encode("utf-8")).hexdigest()
            texts.setdefault(h, (text.strip(), []))[1].append(label)

    std = sysroot / "share" / "doc" / "rust" / "COPYRIGHT-library.html"
    std_text = ""
    if std.is_file():
        raw = std.read_text(encoding="utf-8", errors="replace")
        raw = re.sub(r"(?is)<(script|style)\b.*?</\1>", "", raw)
        std_text = html.unescape(re.sub(r"<[^>]+>", "", raw))
        std_text = re.sub(r"\n\s*\n+", "\n\n", std_text).strip()
    else:
        missing.append(f"the Rust standard library: no {std}")

    if missing:
        print("third-party notices: refused — a linked crate whose licence cannot be shown:", file=sys.stderr)
        for m in missing:
            print("  " + m, file=sys.stderr)
        return 1

    with out.open("w", encoding="utf-8", newline="\n") as w:
        w.write(f"THIRD-PARTY NOTICES — DeluluLang {version}, {target}\n\n")
        w.write("The `delulu` binary in this archive is DeluluLang's own work (LICENSE, NOTICE, TRADEMARK.md) linked with\n"
                f"the {len(entries)} crates below and the Rust standard library: the portable build, `cargo build --release -p\n"
                "delulu --no-default-features --features net`, as `cargo tree` resolves it for this target. Each crate is\n"
                "listed with its licence as its authors declare it; where the expression offers a choice, DeluluLang uses\n"
                "the crate under any one of the licences offered, and under the permissive side where one choice is a\n"
                "copyleft licence (said beside the crate). Their licence and notice texts follow, each distinct text once,\n"
                "with the crates it comes from. Generated by scripts/third-party-notices.py.\n\n")
        w.write("=" * 100 + "\nCRATES\n" + "=" * 100 + "\n")
        for e in entries:
            w.write(e + "\n")
        w.write("\n" + "=" * 100 + "\nTHE RUST STANDARD LIBRARY — the toolchain's own copyright notice\n" + "=" * 100 + "\n")
        w.write(std_text + "\n")
        w.write("\n" + "=" * 100 + "\nLICENCE AND NOTICE TEXTS\n" + "=" * 100 + "\n")
        for text, labels in sorted(texts.values(), key=lambda t: t[1][0]):
            w.write("\n" + "-" * 100 + "\nFrom: " + ", ".join(labels) + "\n" + "-" * 100 + "\n" + text + "\n")
    print(f"third-party notices: {out} — {len(entries)} crates, {len(texts)} distinct texts, target {target}")
    if "--sbom" in sys.argv:
        write_sbom(Path(sys.argv[sys.argv.index("--sbom") + 1]), version, target, linked, dirs)
    return 0


def write_sbom(path, version, target, linked, dirs):
    """CycloneDX 1.5: the archive's binary and every crate it links (the graph above), with Cargo.lock's checksums."""
    # Cargo.lock's `[[package]]` blocks, read without a TOML library (the macOS runner's system Python predates tomllib).
    sums = {}
    for block in (ROOT / "Cargo.lock").read_text(encoding="utf-8").split("[[package]]")[1:]:
        field = {k: v for k, v in re.findall(r'^(name|version|checksum) = "([^"]*)"', block, re.M)}
        if "name" in field and "version" in field:
            sums[(field["name"], field["version"])] = field.get("checksum")
    components = []
    for (name, ver), lic in sorted(linked.items()):
        meta = dirs.get((name, ver)) or {}
        if meta.get("source") is None:
            continue
        c = {"type": "library", "name": name, "version": ver, "purl": f"pkg:cargo/{name}@{ver}",
             "licenses": [{"expression": lic}]}
        if sums.get((name, ver)):
            c["hashes"] = [{"alg": "SHA-256", "content": sums[(name, ver)]}]
        if meta.get("repository"):
            c["externalReferences"] = [{"type": "vcs", "url": meta["repository"]}]
        components.append(c)
    bom = {
        "bomFormat": "CycloneDX", "specVersion": "1.5", "version": 1,
        "metadata": {
            "component": {"type": "application", "name": "delulu", "version": version, "purl": f"pkg:cargo/delulu@{version}",
                          "licenses": [{"license": {"id": "Apache-2.0"}}]},
            "properties": [{"name": "delulu:target", "value": target},
                           {"name": "delulu:build", "value": "cargo build --release -p delulu --no-default-features --features net"},
                           {"name": "delulu:note", "value": "Every crate the binary links (normal dependencies of the portable build, "
                                                            "as cargo tree resolves them for this target); the Rust standard library is "
                                                            "linked too and is not a crates.io component. Generated by scripts/third-party-notices.py."}],
        },
        "components": components,
    }
    path.write_text(json.dumps(bom, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"sbom: {path} — {len(components)} components")


if __name__ == "__main__":
    sys.exit(main())
