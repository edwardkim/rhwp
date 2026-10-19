#!/usr/bin/env python3
"""Generate a public positional-text control without embedding any font bytes.
Requires pikepdf, fontTools, and a locally licensed regular HCR Batang TTF.
"""
from argparse import ArgumentParser
from pathlib import Path
import pikepdf
from fontTools.ttLib import TTFont


def build(font_path: Path, output: Path):
    font = TTFont(font_path)
    if font['name'].getBestFamilyName() not in ('HCR Batang', '함초롬바탕'):
        raise ValueError('This control uses regular HCR Batang as its declared external program')
    cmap = font.getBestCmap()
    chars = ['가', '나', '다', '라']
    upem = font['head'].unitsPerEm
    widths = [font['hmtx'][cmap[ord(ch)]][0] * 1000 / upem for ch in chars]
    pdf = pikepdf.Pdf.new()
    D, N, A = pikepdf.Dictionary, pikepdf.Name, pikepdf.Array

    def unicode_map(first, count_bytes):
        pairs = '\n'.join(f'<{i:0{count_bytes*2}X}> <{ord(ch):04X}>'
                          for i, ch in enumerate(chars, first))
        return pdf.make_stream((
            '/CIDInit /ProcSet findresource begin 12 dict begin begincmap '
            '/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def '
            '/CMapName /WordControl def /CMapType 2 def 1 begincodespacerange '
            f'<{"00"*count_bytes}> <{"FF"*count_bytes}> endcodespacerange '
            f'4 beginbfchar\n{pairs}\nendbfchar endcmap '
            'CMapName currentdict /CMap defineresource pop end end').encode())

    ps = N('/' + font['name'].getDebugName(6))
    descriptor = pdf.make_indirect(D(Type=N.FontDescriptor, FontName=ps, Flags=32,
        FontBBox=A([0, -200, 1000, 1000]), ItalicAngle=0, Ascent=900,
        Descent=-200, CapHeight=700, StemV=80))
    gids = b'\0\0' + b''.join(font.getGlyphID(cmap[ord(ch)]).to_bytes(2, 'big') for ch in chars)
    cid = pdf.make_indirect(D(Type=N.Font, Subtype=N.CIDFontType2, BaseFont=ps,
        CIDSystemInfo=D(Registry='Adobe', Ordering='Identity', Supplement=0),
        FontDescriptor=descriptor, W=A([1, A(widths)]), CIDToGIDMap=pdf.make_stream(gids)))
    external = pdf.make_indirect(D(Type=N.Font, Subtype=N.Type0, BaseFont=ps,
        Encoding=N('/Identity-H'), DescendantFonts=A([cid]), ToUnicode=unicode_map(1, 2)))
    procs = D()
    for i, (glyph, width) in enumerate(zip('ABCD', widths), 1):
        # Valid Type3 programs delegate text to a non-embedded external font;
        # they contain no proprietary outline data and have no BaseFont name.
        procs['/' + glyph] = pdf.make_stream((
            f'{width} 0 0 -200 1000 1000 d1 BT /FG 1000 Tf '
            f'1 0 0 1 0 0 Tm <{i:04X}> Tj ET').encode())
    type3 = pdf.make_indirect(D(Type=N.Font, Subtype=N.Type3,
        FontBBox=A([0, -200, 1000, 1000]), FontMatrix=A([.001, 0, 0, .001, 0, 0]),
        CharProcs=procs, Resources=D(Font=D(FG=external)),
        Encoding=D(Type=N.Encoding, Differences=A([65, *[N('/' + g) for g in 'ABCD']])),
        FirstChar=65, LastChar=68, Widths=A(widths), ToUnicode=unicode_map(65, 1)))
    page = pdf.add_blank_page(page_size=(300, 240))
    page.Resources = D(Font=D(F1=type3))
    # A physical 0.25em gap between distinct text objects, no U+0020 in source.
    # The other two lines have no word gap: plain text and negative TJ kerning.
    second_x = (40 + 12 * (widths[0] + widths[1] + 250) / 1000) / .12
    content = (f'q .12 0 0 .12 0 0 cm\n'
        'BT /F1 100 Tf 1 0 0 1 333.333333 1500 Tm <4142> Tj ET\n'
        f'BT /F1 100 Tf 1 0 0 1 {second_x} 1500 Tm <4344> Tj ET\n'
        'BT /F1 100 Tf 1 0 0 1 333.333333 1166.666667 Tm <41424344> Tj ET\n'
        'BT /F1 100 Tf 1 0 0 1 333.333333 833.333333 Tm [<4142> 50 <4344>] TJ ET\nQ')
    page.Contents = pdf.make_stream(content.encode())
    pdf.save(output, deterministic_id=True)


if __name__ == '__main__':
    parser = ArgumentParser(description=__doc__)
    parser.add_argument('--font', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=Path(__file__).with_name('positioned-word-gap.pdf'))
    args = parser.parse_args()
    build(args.font, args.output)
