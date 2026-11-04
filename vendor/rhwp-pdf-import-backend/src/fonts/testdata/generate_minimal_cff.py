"""Generate the four-glyph CFF container used by the subset format contract.

Original synthetic rectangles; no external font data. MIT OR Apache-2.0,
matching the containing crate. Requires fontTools in the development venv.
This input tests container type and GID preservation, not visual fidelity.
"""
from pathlib import Path
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.t2CharStringPen import T2CharStringPen

names = [".notdef", "A", "B", "C"]
font = FontBuilder(1000, isTTF=False)
font.setupGlyphOrder(names)
font.setupCharacterMap({65: "A", 66: "B", 67: "C"})
font.setupHorizontalMetrics({name: (600, 0) for name in names})
font.setupHorizontalHeader(ascent=800, descent=-200)
font.setupNameTable({
    "familyName": "RHWP CFF Test", "styleName": "Regular",
    "uniqueFontIdentifier": "rhwp-cff-subset-test-1",
    "fullName": "RHWP CFF Test Regular", "psName": "RHWPCFFTest-Regular",
    "version": "Version 1.000",
    "copyright": "Original rhwp test data; MIT OR Apache-2.0",
})
font.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
font.setupPost()
charstrings = {}
for index, name in enumerate(names):
    pen = T2CharStringPen(600, None)
    if index:
        pen.moveTo((50, 0))
        pen.lineTo((50, 500 + index * 20))
        pen.lineTo((400, 500 + index * 20))
        pen.lineTo((400, 0))
        pen.closePath()
    charstrings[name] = pen.getCharString()
font.setupCFF("RHWPCFFTest-Regular", {
    "FullName": "RHWP CFF Test Regular", "FamilyName": "RHWP CFF Test",
    "Weight": "Regular", "Notice": "Original rhwp test data; MIT OR Apache-2.0",
}, charstrings, {})
font.setupMaxp()
font.font.recalcTimestamp = False
font.font["head"].created = font.font["head"].modified = 3029529600
font.save(Path(__file__).with_name("minimal-cff.otf"))
