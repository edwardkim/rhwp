"""검사용 U+257A 제공 프로그램을 만든다. 실행: venv/bin/python <이 파일>."""
from pathlib import Path
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

if __name__ == '__main__':
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder(['.notdef', 'providedMarker'])
    builder.setupCharacterMap({0x257A: 'providedMarker'})
    empty = TTGlyphPen(None)
    marker = TTGlyphPen(None)
    marker.moveTo((50, 200))
    for point in [(450, 200), (450, 300), (50, 300)]:
        marker.lineTo(point)
    marker.closePath()
    builder.setupGlyf({'.notdef': empty.glyph(), 'providedMarker': marker.glyph()})
    builder.setupHorizontalMetrics({'.notdef': (600, 0), 'providedMarker': (750, 50)})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
    builder.setupNameTable({'familyName': 'RHWP Box Marker Present', 'styleName': 'Regular',
        'uniqueFontIdentifier': 'RHWP-7733-Box-Marker-Present',
        'fullName': 'RHWP Box Marker Present Regular', 'psName': 'RHWPBoxMarkerPresent-Regular',
        'licenseDescription': 'MIT License. See LICENSE.txt.'})
    builder.setupPost()
    builder.setupMaxp()
    builder.font['head'].created = builder.font['head'].modified = 2082844800
    builder.save(Path(__file__).with_name('program-present.ttf'))
