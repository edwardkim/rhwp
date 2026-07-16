#!/usr/bin/env python3
"""Generate a deterministic HWPX fixture with a font-native bitmap glyph."""

from __future__ import annotations

import sys
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "samples" / "hwpx" / "ref" / "ref_empty.hwpx"
FONT = ROOT / "tests" / "fixtures" / "fonts" / "RHWPBitmapSvgGlyphSmoke.ttf"
OUTPUT = ROOT / "samples" / "render-p35-font-native-bitmap.hwpx"
ZIP_TIMESTAMP = (1980, 1, 1, 0, 0, 0)


def replace_once(value: str, before: str, after: str, label: str) -> str:
    if value.count(before) < 1:
        raise RuntimeError(f"{label} source marker is missing")
    return value.replace(before, after, 1)


def zip_info(name: str, compression: int) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(name, ZIP_TIMESTAMP)
    info.compress_type = compression
    info.external_attr = 0o100644 << 16
    return info


def main() -> int:
    if not SOURCE.is_file() or not FONT.is_file():
        raise RuntimeError("generate the font fixture and keep ref_empty.hwpx available first")

    with zipfile.ZipFile(SOURCE) as source:
        entries = {name: source.read(name) for name in source.namelist()}

    header = entries["Contents/header.xml"].decode("utf-8")
    header = replace_once(
        header,
        'face="함초롬바탕" type="TTF" isEmbedded="0"',
        'face="RHWP Bitmap SVG Glyph Smoke" type="TTF" isEmbedded="1" '
        'binaryItemIDRef="font-native-smoke"',
        "embedded font",
    )
    entries["Contents/header.xml"] = header.encode("utf-8")

    section = entries["Contents/section0.xml"].decode("utf-8")
    section = replace_once(section, "<hp:t/>", "<hp:t>\ue100</hp:t>", "fixture text")
    entries["Contents/section0.xml"] = section.encode("utf-8")

    content = entries["Contents/content.hpf"].decode("utf-8")
    manifest_item = (
        '<opf:item id="font-native-smoke" href="BinData/font-native-smoke.ttf" '
        'media-type="application/x-font-ttf" isEmbeded="1"/>'
    )
    content = replace_once(
        content,
        "</opf:manifest>",
        f"{manifest_item}</opf:manifest>",
        "package manifest",
    )
    entries["Contents/content.hpf"] = content.encode("utf-8")
    entries["BinData/font-native-smoke.ttf"] = FONT.read_bytes()

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUTPUT, "w") as output:
        for name, data in entries.items():
            compression = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
            output.writestr(zip_info(name, compression), data)

    print(OUTPUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
