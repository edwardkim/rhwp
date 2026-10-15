#!/usr/bin/env python3
"""HFT paint 전용 CFF 계약용 독립 사각형을 생성한다. 한컴 출력 표본이 아니다."""
from pathlib import Path
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.t2CharStringPen import T2CharStringPen


def generate():
    builder = FontBuilder(1000, isTTF=False)
    builder.setupGlyphOrder(['.notdef', 'uniAC00'])
    builder.setupCharacterMap({0xAC00: 'uniAC00'})
    empty = T2CharStringPen(900, None)
    pen = T2CharStringPen(900, None)
    pen.moveTo((100, 0))
    pen.lineTo((400, 0))
    pen.lineTo((400, 700))
    pen.lineTo((100, 700))
    pen.closePath()
    builder.setupHorizontalMetrics({'.notdef': (900, 0), 'uniAC00': (900, 100)})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupNameTable({
        'familyName': 'RHWP HFT Outline Fixture', 'styleName': 'Regular',
        'fullName': 'RHWP HFT Outline Fixture Regular',
        'uniqueFontIdentifier': 'RHWP-HFT-Outline-Fixture-1',
        'psName': 'RHWP-HFT-Outline-Fixture', 'version': 'Version 1.000',
    })
    builder.setupOS2(sTypoAscender=800, sTypoDescender=-200,
                     usWinAscent=800, usWinDescent=200)
    builder.setupPost()
    builder.setupCFF('RHWP-HFT-Outline-Fixture', {},
                     {'.notdef': empty.getCharString(), 'uniAC00': pen.getCharString()}, {})
    builder.font['head'].created = builder.font['head'].modified = 2082844800
    builder.font.recalcTimestamp = False
    destination = Path(__file__).resolve().parents[1] / 'tests/fixtures/fonts/RHWPHftOutline.otf'
    builder.save(destination)


if __name__ == '__main__':
    generate()
