#!/usr/bin/env python3
"""Generate original outlines for the host-font CanvasKit contract (#7403)."""
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTCollection


ROOT = Path(__file__).resolve().parents[1] / "tests/fixtures/fonts"


def face(style, offset):
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder([".notdef", "test"])
    builder.setupCharacterMap({ord("A"): "test", ord("가"): "test"})
    empty = TTGlyphPen(None)
    pen = TTGlyphPen(None)
    # A filled parallelogram: different real outlines, not synthetic paint skew.
    pen.moveTo((100, 0))
    pen.lineTo((400, 0))
    pen.lineTo((400 + offset, 700))
    pen.lineTo((100 + offset, 700))
    pen.closePath()
    builder.setupGlyf({".notdef": empty.glyph(), "test": pen.glyph()})
    builder.setupHorizontalMetrics({name: (900, 100) for name in [".notdef", "test"]})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupNameTable({
        "familyName": "RHWP Host Fixture", "styleName": style,
        "fullName": f"RHWP Host Fixture {style}",
        "uniqueFontIdentifier": f"RHWPHostFixture-{style}",
        "psName": f"RHWPHostFixture-{style}", "version": "Version 1.000",
    })
    builder.setupOS2(sTypoAscender=800, sTypoDescender=-200,
                     usWinAscent=800, usWinDescent=200,
                     fsSelection=0x40 if offset == 0 else 1)
    builder.setupPost(italicAngle=0 if offset == 0 else -12)
    builder.setupMaxp()
    font = builder.font
    font["head"].macStyle = 0 if offset == 0 else 2
    font["head"].created = font["head"].modified = 2082844800
    font.recalcTimestamp = False
    return font


if __name__ == "__main__":
    collection = TTCollection()
    collection.fonts = [face("Regular", 0), face("Italic", 180), face("Oblique", 300)]
    for style, font in zip(["Regular", "Italic", "Oblique"], collection.fonts):
        # Standalone files are independent reference inputs for the TTC read path.
        font.save(ROOT / f"RHWPHostFixture-{style}.ttf")
    collection.save(ROOT / "RHWPHostFixture.ttc", shareTables=True)
