#!/usr/bin/env python3
"""Copy existing repository notices into a binary/WASM package, without fonts or code."""
import argparse
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def notice_files(root):
    files = {root / "THIRD_PARTY_LICENSES.md",
             root / "data/font-metrics/nimbus-roman/URW-FONT-NOTICE.txt"}
    for directory in ("licenses", "assets/licenses", "assets/fonts/licenses"):
        files.update(p for p in (root / directory).rglob("*") if p.is_file())
    for pattern in ("*-OFL.txt", "*-LICENSE.txt", "MANIFEST-*.txt"):
        files.update((root / "assets/fonts").glob(pattern))
    for path in (root / "vendor").rglob("*"):
        if path.is_file() and (path.name.startswith(("LICENSE", "LICENCE", "NOTICE", "COPYING"))
                              or path.name in ("README.ijg", "RHWP_PATCHES.md", "UPSTREAM.json", "UPSTREAM_FILES.json")):
            files.add(path)
    return sorted(files)


def prepare(destination, root=ROOT):
    destination = destination.resolve()
    assert destination != root.resolve(), "Use a package notices directory"
    copied = []
    for source in notice_files(root):
        if not source.resolve().is_relative_to(root.resolve()):
            raise ValueError(f"Notice resolves outside the source tree: {source}")
        # A few upstream font notices are PDFs. Preserve their original bytes;
        # every other notice must be UTF-8 text, never a font/program binary.
        if source.suffix.lower() == ".pdf":
            if not source.read_bytes().startswith(b"%PDF-"):
                raise ValueError(f"Invalid PDF notice: {source}")
        else:
            source.read_text(encoding="utf-8")
        relative = source.relative_to(root)
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        copied.append(relative.as_posix())
    return copied


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    print(f"Copied {len(prepare(args.destination))} existing notice files")
