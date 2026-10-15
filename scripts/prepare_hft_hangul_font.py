"""독립 전체 한글 자모 표본의 Type3 윤곽을 로컬 CFF 글꼴로 옮기는 진단 도구.

대상 문서나 대상 정답 PDF는 받지 않는다. 표본 Print의 원본 윤곽/전진폭을
그대로 옮기며 화면 점수에 맞춘 스케일/이동/획 수정은 허용하지 않는다.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re
import copy
import zipfile
import xml.etree.ElementTree as ET
import pymupdf
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.misc.roundTools import otRound


def mapped_characters(document, reference):
    body = document.xref_stream(reference).decode('ascii')
    pairs = []
    for group in re.findall(r'beginbfchar(.*?)endbfchar', body, re.S):
        for source, destination in re.findall(r'<([\da-fA-F]+)>\s*<([\da-fA-F]+)>', group):
            value = bytes.fromhex(destination).decode('utf-16-be')
            if len(value) == 1:
                pairs.append((int(source, 16), ord(value)))
    for group in re.findall(r'beginbfrange(.*?)endbfrange', body, re.S):
        pattern = r'<([\da-fA-F]+)>\s*<([\da-fA-F]+)>\s*(<([\da-fA-F]+)>|\[(.*?)\])'
        for first, last, _, scalar, array in re.findall(pattern, group, re.S):
            first, last = int(first, 16), int(last, 16)
            if last < first or last - first > 65535:
                raise ValueError('Invalid character mapping range')
            if scalar:
                values = [(int(scalar, 16) + offset).to_bytes(len(scalar) // 2, 'big').decode('utf-16-be') for offset in range(last - first + 1)]
            else:
                values = [bytes.fromhex(value).decode('utf-16-be') for value in re.findall(r'<([\da-fA-F]+)>', array)]
            if len(values) != last - first + 1:
                raise ValueError('Character mapping range has missing destinations')
            pairs.extend((code, ord(value)) for code, value in zip(range(first, last + 1), values) if len(value) == 1)
    return pairs


def font_encoding(document, reference):
    body = document.xref_object(reference)
    group = re.search(r'/Differences\s*\[(.*?)\]', body, re.S)[1]
    result, position = {}, None
    for token in re.findall(r'/[^\s\[\]]+|\d+', group):
        if token.startswith('/'):
            if position is None:
                raise ValueError('Encoding has no first code')
            result[position] = token[1:]
            position += 1
        else:
            position = int(token)
    return result


def font_reference(body, key):
    return int(re.search(r'/' + key + r'\s+(\d+)\s+0\s+R', body)[1])


def source_horizontal_metrics(width, charstring):
    """CFF 윤곽의 xMin을 hmtx에도 기록한다. 글리프 자체는 이동하지 않는다."""
    bounds = charstring.calcBounds(None)
    return otRound(width), otRound(bounds[0]) if bounds else 0


def restore_outline(stream):
    # PDF Type3의 글리프 좌표와 CFF의 em 좌표는 같은 위쪽 양수 좌표계다.
    tokens = re.findall(r'[-+]?\d*\.?\d+(?:[Ee][-+]?\d+)?|[A-Za-z][A-Za-z0-9]*', stream)
    operands, pen, width = [], None, None
    for token in tokens:
        try:
            operands.append(float(token))
            continue
        except ValueError:
            pass
        if token == 'd1':
            if len(operands) != 6 or operands[1] != 0 or pen is not None:
                raise ValueError('Invalid glyph metrics')
            width = operands[0]
            pen = T2CharStringPen(width, None, roundTolerance=0)
        elif token == 'm':
            if pen is None or len(operands) != 2:
                raise ValueError('Invalid move')
            pen.moveTo(tuple(operands))
        elif token == 'l':
            if pen is None or len(operands) != 2:
                raise ValueError('Invalid line')
            pen.lineTo(tuple(operands))
        elif token == 'c':
            if pen is None or len(operands) != 6:
                raise ValueError('Invalid cubic')
            pen.curveTo(*[tuple(operands[i:i + 2]) for i in range(0, 6, 2)])
        elif token == 'h':
            if operands or pen is None:
                raise ValueError('Invalid close')
            # 한컴은 마지막 경로 뒤 h를 중복해서 쓰기도 한다.
            if pen._getCurrentPoint() is not None:
                pen.closePath()
        elif token != 'f' or operands:
            raise ValueError('Unsupported glyph operator: ' + token)
        operands = []
    if pen is None or operands:
        raise ValueError('Incomplete glyph program')
    return width, pen.getCharString()


def restore(specimen_pdf, output_font):
    document = pymupdf.open(specimen_pdf)
    expected_text = ''.join(chr(value) for value in range(0xAC00, 0xD7A4))
    observed = ''.join(ch for page in document for ch in page.get_text() if not ch.isspace())
    if observed != expected_text:
        raise ValueError('Only the independent complete alphabet specimen is accepted; target document PDFs are not font sources')
    glyphs = {}
    for reference in sorted({font[0] for page in document for font in page.get_fonts() if font[2] == 'Type3'}):
        body = document.xref_object(reference)
        matrix = list(map(float, re.search(r'/FontMatrix\s*\[([^\]]+)\]', body)[1].split()))
        if matrix != [0.001, 0, 0, 0.001, 0, 0]:
            raise ValueError('Non-original glyph matrix')
        encoding = font_encoding(document, font_reference(body, 'Encoding'))
        procedures = {name: int(value) for name, value in re.findall(r'/([^\s/]+)\s+(\d+)\s+0\s+R', document.xref_object(font_reference(body, 'CharProcs')))}
        for code, scalar in mapped_characters(document, font_reference(body, 'ToUnicode')):
            if not 0xAC00 <= scalar <= 0xD7A3:
                continue
            stream = document.xref_stream(procedures[encoding[code]]).decode('ascii')
            if scalar in glyphs and glyphs[scalar] != stream:
                raise ValueError(f'Conflicting source glyph U+{scalar:04X}')
            glyphs[scalar] = stream
    expected = set(range(0xAC00, 0xD7A4))
    if set(glyphs) != expected:
        raise ValueError(f'Complete original alphabet required: {len(glyphs)}/11172 glyphs')
    order = ['.notdef'] + [f'uni{value:04X}' for value in sorted(glyphs)]
    builder = FontBuilder(1000, isTTF=False)
    builder.setupGlyphOrder(order)
    builder.setupCharacterMap({value: f'uni{value:04X}' for value in sorted(glyphs)})
    pen = T2CharStringPen(1000, None)
    strings = {'.notdef': pen.getCharString()}
    widths = {'.notdef': (1000, 0)}
    source_hashes = {}
    for value, stream in sorted(glyphs.items()):
        width, charstring = restore_outline(stream)
        name = f'uni{value:04X}'
        strings[name] = charstring
        widths[name] = (round(width), 0)
        source_hashes[f'{value:04X}'] = hashlib.sha256(stream.encode()).hexdigest()
    builder.setupHorizontalMetrics(widths)
    builder.setupHorizontalHeader(ascent=1000, descent=-200)
    builder.setupNameTable({'familyName': '한양신명조', 'styleName': 'Regular',
                           'uniqueFontIdentifier': 'OriginalHFT-HGSMJ-IndependentAlphabet',
                           'fullName': '한양신명조 Original HFT Hangul',
                           'psName': 'HanyangSinMyeongJo-OriginalHFT-Hangul',
                           'version': 'Version 1.000'})
    builder.setupOS2(sTypoAscender=1000, sTypoDescender=-200, usWinAscent=1000, usWinDescent=200)
    builder.setupPost()
    builder.setupCFF('HanyangSinMyeongJo-OriginalHFT-Hangul',
                     {'FullName': 'Original HFT Hangul', 'FamilyName': 'Original HFT Hangul', 'Weight': 'Regular'}, strings, {})
    # CFF 자체에서 bearing을 읽는 rasterizer와 hmtx를 읽는 소비자가 같은
    # 원점을 사용해야 한다. setupCFF가 연결한 private dict로 원본 경계를 읽는다.
    builder.setupHorizontalMetrics({name: source_horizontal_metrics(widths[name][0], string)
                                    for name, string in strings.items()})
    output_font.parent.mkdir(parents=True, exist_ok=True)
    builder.save(output_font)
    record = {'source': str(specimen_pdf), 'sourceSha256': hashlib.sha256(specimen_pdf.read_bytes()).hexdigest(),
              'font': str(output_font), 'fontSha256': hashlib.sha256(output_font.read_bytes()).hexdigest(),
              'scope': 'Complete modern Hangul alphabet only; ASCII intentionally absent and uses the existing supplied chain',
              'glyphs': len(glyphs), 'sourceGlyphSha256': source_hashes, 'fontMatrix': [0.001, 0, 0, 0.001, 0, 0],
              'glyphTransform': 'identity; no score-dependent shift, scale, hinting or contour changes',
              'horizontalMetrics': 'original advance and rounded CFF xMin; outlines are not translated'}
    output_font.with_suffix('.provenance.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: value for key, value in record.items() if key != 'sourceGlyphSha256'}, ensure_ascii=True))


def create_specimen(template, output):
    """한컴 빈 HWPX의 쪽/스타일을 쓰되 저장 줄과 기존 내용을 보존하지 않는다."""
    hp = 'http://www.hancom.co.kr/hwpml/2011/paragraph'
    hh = 'http://www.hancom.co.kr/hwpml/2011/head'
    with zipfile.ZipFile(template) as source:
        header = ET.fromstring(source.read('Contents/header.xml'))
        section = ET.fromstring(source.read('Contents/section0.xml'))
        if len(section) != 1 or not list(section[0].iter('{' + hp + '}secPr')):
            raise ValueError('A blank single-section template is required')
        for font in header.iter('{' + hh + '}font'):
            font.set('face', '한양신명조')
            font.set('type', 'HFT')
        paragraph = copy.deepcopy(section[0])
        for run in paragraph.findall('{' + hp + '}run'):
            for child in list(run):
                if child.tag.split('}')[-1] not in ['secPr', 'ctrl']:
                    run.remove(child)
        for child in list(paragraph):
            if child.tag != '{' + hp + '}run':
                paragraph.remove(child)
        characters = ''.join(chr(value) for value in range(0xAC00, 0xD7A4))
        run = ET.SubElement(paragraph, '{' + hp + '}run', {'charPrIDRef': '0'})
        ET.SubElement(run, '{' + hp + '}t').text = characters
        section.remove(section[0])
        section.append(paragraph)
        output.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as target:
            for info in source.infolist():
                data = source.read(info.filename)
                if info.filename == 'Contents/header.xml':
                    data = ET.tostring(header, encoding='utf-8', xml_declaration=True)
                elif info.filename == 'Contents/section0.xml':
                    data = ET.tostring(section, encoding='utf-8', xml_declaration=True)
                elif info.filename == 'Preview/PrvText.txt':
                    data = characters.encode('utf-8')
                target.writestr(info, data)
    record = {'purpose': 'Independent complete Hangul alphabet, no target document data',
              'template': str(template), 'templateSha256': hashlib.sha256(template.read_bytes()).hexdigest(),
              'input': str(output), 'inputSha256': hashlib.sha256(output.read_bytes()).hexdigest(),
              'requestedFace': '한양신명조', 'requestedFontType': 'HFT', 'characters': len(characters)}
    output.with_suffix('.provenance.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(record, ensure_ascii=True))


def main():
    parser = argparse.ArgumentParser(description='독립 한글 전체 표본으로 원본 HFT 글꼴 공급 준비; 대상 정답 PDF 사용 금지')
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('create-specimen', help='한컴 빈 HWPX로 독립 11172자 표본 작성')
    create.add_argument('--template', type=Path, required=True)
    create.add_argument('--out', type=Path, required=True)
    recover = commands.add_parser('restore', help='그 별도 표본의 Print Type3 원본 윤곽을 CFF로 옮김')
    recover.add_argument('--specimen-pdf', type=Path, required=True)
    recover.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'create-specimen':
        create_specimen(args.template, args.out)
    else:
        restore(args.specimen_pdf, args.out)


if __name__ == '__main__':
    main()
