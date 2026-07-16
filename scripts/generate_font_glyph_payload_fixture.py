#!/usr/bin/env python3
"""Generate the deterministic font-native BitmapGlyph/SvgGlyph proof fixture."""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import newTable
from fontTools.ttLib.tables.sbixGlyph import Glyph
from fontTools.ttLib.tables.sbixStrike import Strike


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "tests" / "fixtures" / "fonts" / "RHWPBitmapSvgGlyphSmoke.ttf"


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    )


def bitmap_png() -> bytes:
    width = 16
    height = 16
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            if 2 <= x < 14 and 2 <= y < 14:
                rows.extend((20, 110 + x * 6, 220 - y * 5, 255))
            else:
                rows.extend((0, 0, 0, 0))
    return (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + png_chunk(b"IDAT", zlib.compress(bytes(rows), level=9))
        + png_chunk(b"IEND", b"")
    )


def rectangle_glyph() -> object:
    pen = TTGlyphPen(None)
    pen.moveTo((100, 100))
    pen.lineTo((900, 100))
    pen.lineTo((900, 900))
    pen.lineTo((100, 900))
    pen.closePath()
    return pen.glyph()


def empty_glyph() -> object:
    return TTGlyphPen(None).glyph()


def main() -> int:
    glyph_order = [
        ".notdef",
        "bitmap",
        "svg",
        "unsafeSvg",
        "malformedSvg",
        "sharedSvgA",
        "sharedSvgB",
    ]
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder(glyph_order)
    builder.setupCharacterMap(
        {
            0xE100: "bitmap",
            0xE101: "svg",
            0xE102: "unsafeSvg",
            0xE103: "malformedSvg",
            0xE104: "sharedSvgA",
            0xE105: "sharedSvgB",
        }
    )
    builder.setupGlyf(
        {
            ".notdef": rectangle_glyph(),
            "bitmap": empty_glyph(),
            "svg": empty_glyph(),
            "unsafeSvg": empty_glyph(),
            "malformedSvg": empty_glyph(),
            "sharedSvgA": empty_glyph(),
            "sharedSvgB": empty_glyph(),
        }
    )
    builder.setupHorizontalMetrics({name: (1000, 0) for name in glyph_order})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupNameTable(
        {
            "familyName": "RHWP Bitmap SVG Glyph Smoke",
            "styleName": "Regular",
            "uniqueFontIdentifier": "RHWPBitmapSvgGlyphSmoke-Regular",
            "fullName": "RHWP Bitmap SVG Glyph Smoke Regular",
            "psName": "RHWPBitmapSvgGlyphSmoke-Regular",
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
    strike = Strike(ppem=16, resolution=72)
    strike.glyphs["bitmap"] = Glyph(
        glyphName="bitmap",
        originOffsetX=2,
        originOffsetY=3,
        graphicType="png ",
        imageData=bitmap_png(),
    )
    sbix = newTable("sbix")
    sbix.strikes[16] = strike
    font["sbix"] = sbix

    safe_svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
        f'<metadata>{"producer-fixture-" * 32}</metadata>'
        '<path d="M2 2H14V14H2Z" fill="#00a0c8"/></svg>'
    )
    svg = newTable("SVG ")
    svg.docList = [
        (
            safe_svg,
            2,
            2,
            True,
        ),
        (
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
            '<script>alert(1)</script><path d="M0 0H16V16H0Z"/></svg>',
            3,
            3,
        ),
        (
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
            '<path d="M0 0H16V16H0Z"/></svgx>',
            4,
            4,
        ),
        (
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
            '<g id="glyph5"><path d="M0 0H8V16H0Z"/></g>'
            '<g id="glyph6"><path d="M8 0H16V16H8Z"/></g></svg>',
            5,
            6,
        ),
    ]
    font["SVG "] = svg

    font["head"].created = 2082844800
    font["head"].modified = 2082844800
    font.recalcTimestamp = False
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    font.save(OUTPUT, reorderTables=False)
    print(OUTPUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
