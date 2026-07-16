#!/usr/bin/env python3
"""Generate the deterministic multi-face TTC exact-construction fixture."""

from __future__ import annotations

import sys
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTCollection, TTFont


ROOT = Path(__file__).resolve().parents[1]
FONT_DIR = ROOT / "tests" / "fixtures" / "fonts"
OUTPUT = FONT_DIR / "RHWPExactFaceSmoke.ttc"


def rectangle_glyph() -> object:
    pen = TTGlyphPen(None)
    pen.moveTo((120, 120))
    pen.lineTo((880, 120))
    pen.lineTo((880, 880))
    pen.lineTo((120, 880))
    pen.closePath()
    return pen.glyph()


def build_face(family_name: str, postscript_name: str, codepoint: int) -> TTFont:
    glyph_order = [".notdef", "exactFace"]
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder(glyph_order)
    builder.setupCharacterMap({codepoint: "exactFace"})
    builder.setupGlyf({name: rectangle_glyph() for name in glyph_order})
    builder.setupHorizontalMetrics({name: (1000, 0) for name in glyph_order})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupNameTable(
        {
            "familyName": family_name,
            "styleName": "Regular",
            "uniqueFontIdentifier": postscript_name,
            "fullName": f"{family_name} Regular",
            "psName": postscript_name,
            "version": "Version 1.000",
        }
    )
    builder.setupOS2(
        sTypoAscender=800,
        sTypoDescender=-200,
        usWinAscent=800,
        usWinDescent=200,
    )
    builder.setupPost()
    builder.setupMaxp()
    font = builder.font
    font["head"].created = 2082844800
    font["head"].modified = 2082844800
    font.recalcTimestamp = False
    return font


def main() -> int:
    collection = TTCollection()
    collection.fonts = [
        build_face("RHWP Exact Face Zero", "RHWPExactFaceZero-Regular", 0xE103),
        build_face("RHWP Exact Face One", "RHWPExactFaceOne-Regular", 0xE104),
    ]
    collection.save(OUTPUT, shareTables=True)
    print(OUTPUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
