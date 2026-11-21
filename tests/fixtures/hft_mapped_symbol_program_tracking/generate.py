"""원본 이중선과 다른 전진을 가진 합성 TTF를 만든다. venv/bin/python으로 실행한다."""
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen


def main():
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder([".notdef", "doubleRule", "adjacentRule"])
    builder.setupCharacterMap({0xF0827: "doubleRule", 0xF0828: "adjacentRule"})
    rule = TTGlyphPen(None)
    rule.moveTo((0, 200))
    for point in [(480, 200), (480, 250), (0, 250)]:
        rule.lineTo(point)
    rule.closePath()
    builder.setupGlyf({".notdef": TTGlyphPen(None).glyph(),
                       "doubleRule": rule.glyph(), "adjacentRule": rule.glyph()})
    builder.setupHorizontalMetrics({".notdef": (500, 0), "doubleRule": (480, 0), "adjacentRule": (700, 0)})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
    builder.setupNameTable({"familyName": "RHWP Rule Fallback", "styleName": "Regular",
                            "fullName": "RHWP Rule Fallback Regular", "psName": "RHWPDoubleRuleFallback-Regular",
                            "licenseDescription": "MIT License. See LICENSE.txt."})
    builder.setupPost()
    builder.setupMaxp()
    builder.font["head"].created = builder.font["head"].modified = 2082844800
    builder.font.recalcTimestamp = False
    builder.save(Path(__file__).with_name("fallback.ttf"))


if __name__ == "__main__":
    main()
