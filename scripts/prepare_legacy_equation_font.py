"""독립 구형 수식 문자 표본에서 로컬 CFF 글꼴을 준비한다.

대상 문서의 정답 PDF는 글꼴 원본으로 사용하지 않는다. 생성한 전체 문자 표본을
한컴에서 Print 출력한 PDF만 받는다. 사설 글꼴은 로컬에 공급하며 배포하지 않는다.
"""
from pathlib import Path
import argparse
import copy
import hashlib
import json
import re
import string
import zipfile
import xml.etree.ElementTree as ET

import pymupdf
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.t2CharStringPen import T2CharStringPen

from prepare_hft_hangul_font import (
    font_encoding, font_reference, mapped_characters, restore_outline,
    source_horizontal_metrics,
)

HP = 'http://www.hancom.co.kr/hwpml/2011/paragraph'
ROLE = 'independent-legacy-equation-alphabet-v1'
FAMILY = 'RHWP Hancom Legacy Equation'
EXTRA = [
    ('+', '+'), ('-', '-'), ('=', '='), (':', ':'), ('<', '<'), ('>', '>'),
    (',', ','), ('.', '.'), ('/', '/'), ('(', '('), (')', ')'), ('[', '['),
    (']', ']'), ('alpha', 'α'), ('beta', 'β'), ('gamma', 'γ'), ('delta', 'δ'),
    ('Delta', 'Δ'), ('theta', 'θ'), ('pi', 'π'), ('sigma', 'σ'), ('lambda', 'λ'),
]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def probes():
    return [{'script': 'rm {' + c + '}', 'expected': c}
            for c in string.ascii_uppercase + string.ascii_lowercase + string.digits] + [
        {'script': script, 'expected': character} for script, character in EXTRA]


def create(template, output):
    with zipfile.ZipFile(template) as source:
        if any('script' in name.lower() for name in source.namelist()):
            raise ValueError('스크립트가 없는 깨끗한 HWPX 서식을 사용하세요')
        root = ET.fromstring(source.read('Contents/section0.xml'))
        section = root.find('.//{' + HP + '}secPr')
        column = root.find('.//{' + HP + '}colPr')
        if section is None or column is None:
            raise ValueError('용지 설정·단 설정을 가진 HWPX 서식이 필요합니다')
        section = copy.deepcopy(section)
        section.set('masterPageCnt', '0')
        for child in list(root):
            root.remove(child)
        for index, item in enumerate(probes()):
            paragraph = ET.SubElement(root, '{' + HP + '}p', {
                'id': str(index), 'paraPrIDRef': '0', 'styleIDRef': '0',
                'pageBreak': '0', 'columnBreak': '0', 'merged': '0'})
            run = ET.SubElement(paragraph, '{' + HP + '}run', charPrIDRef='0')
            if index == 0:
                run.append(section)
                control = ET.SubElement(run, '{' + HP + '}ctrl')
                control.append(copy.deepcopy(column))
            equation = ET.SubElement(run, '{' + HP + '}equation', {
                'id': str(index), 'zOrder': '0', 'numberingType': 'EQUATION',
                'textWrap': 'TOP_AND_BOTTOM', 'textFlow': 'BOTH_SIDES', 'lock': '0',
                'version': '', 'baseLine': '80', 'textColor': '#000000',
                'baseUnit': '1000', 'lineMode': 'CHAR', 'font': 'HYhwpEQ'})
            ET.SubElement(equation, '{' + HP + '}sz', {
                'width': '1500', 'widthRelTo': 'ABSOLUTE', 'height': '1600',
                'heightRelTo': 'ABSOLUTE', 'protect': '0'})
            ET.SubElement(equation, '{' + HP + '}pos', {
                'treatAsChar': '1', 'affectLSpacing': '1', 'flowWithText': '1',
                'allowOverlap': '0', 'holdAnchorAndSO': '0', 'vertRelTo': 'PARA',
                'horzRelTo': 'COLUMN', 'vertAlign': 'TOP', 'horzAlign': 'LEFT',
                'vertOffset': '0', 'horzOffset': '0'})
            ET.SubElement(equation, '{' + HP + '}outMargin', {
                'left': '56', 'right': '56', 'top': '0', 'bottom': '0'})
            ET.SubElement(equation, '{' + HP + '}script').text = item['script']
            ET.SubElement(run, '{' + HP + '}t')
        output.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(output, 'w') as destination:
            for info in source.infolist():
                data = (ET.tostring(root, encoding='utf-8', xml_declaration=True)
                        if info.filename == 'Contents/section0.xml'
                        else source.read(info.filename))
                destination.writestr(info, data)
    manifest = {'role': ROLE, 'input': str(output.resolve()),
                'input_sha256': digest(output), 'font_family': FAMILY,
                'probes': probes(), 'template_sha256': digest(template)}
    output.with_suffix('.manifest.json').write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def recover(pdf, manifest_path, output):
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    if (manifest.get('role') != ROLE or manifest.get('probes') != probes()
            or digest(manifest['input']) != manifest['input_sha256']):
        raise ValueError('동일한 독립 문자 표본과 입력 SHA-256이 필요합니다')
    document = pymupdf.open(pdf)
    observed = ''.join(c for page in document for c in page.get_text() if not c.isspace())
    if observed != ''.join(item['expected'] for item in probes()):
        raise ValueError('대상 PDF는 사용할 수 없습니다. 전체 독립 문자 표본을 Print 하세요')
    glyphs = {}
    fonts = {font[0] for page in document for font in page.get_fonts() if font[2] == 'Type3'}
    for reference in sorted(fonts):
        body = document.xref_object(reference)
        matrix = list(map(float, re.search(r'/FontMatrix\s*\[([^\]]+)\]', body)[1].split()))
        if matrix != [0.001, 0, 0, 0.001, 0, 0]:
            raise ValueError('원본 em 행렬이 아닌 글리프는 받지 않습니다')
        encoding = font_encoding(document, font_reference(body, 'Encoding'))
        procedures = {name: int(value) for name, value in re.findall(
            r'/([^\s/]+)\s+(\d+)\s+0\s+R',
            document.xref_object(font_reference(body, 'CharProcs')))}
        for code, scalar in mapped_characters(document, font_reference(body, 'ToUnicode')):
            program = document.xref_stream(procedures[encoding[code]]).decode('ascii')
            if scalar in glyphs and glyphs[scalar] != program:
                raise ValueError(f'서로 다른 글리프 U+{scalar:04X}')
            glyphs[scalar] = program
    required = set(map(ord, string.ascii_letters + string.digits + '+-=:<>,./()[]'))
    if set(glyphs) != required:
        raise ValueError('구형의 독립 Type3 문자 75개 전체가 필요합니다')
    builder = FontBuilder(1000, isTTF=False)
    builder.setupGlyphOrder(['.notdef'] + [f'uni{c:04X}' for c in sorted(glyphs)])
    builder.setupCharacterMap({c: f'uni{c:04X}' for c in glyphs})
    strings = {'.notdef': T2CharStringPen(1000, None).getCharString()}
    widths = {'.notdef': (1000, 0)}
    for scalar, program in sorted(glyphs.items()):
        width, charstring = restore_outline(program)
        strings[f'uni{scalar:04X}'] = charstring
        widths[f'uni{scalar:04X}'] = (width, 0)
    ps_name = 'RHWP-Hancom-Legacy-Equation-Local'
    builder.setupHorizontalMetrics(widths)
    builder.setupHorizontalHeader(ascent=1000, descent=-200)
    builder.setupNameTable({'familyName': FAMILY, 'styleName': 'Regular',
                           'uniqueFontIdentifier': ps_name + '-' + digest(pdf)[:16],
                           'fullName': FAMILY, 'psName': ps_name, 'version': 'Version 1.000'})
    builder.setupOS2(sTypoAscender=1000, sTypoDescender=-200, usWinAscent=1000, usWinDescent=200)
    builder.setupPost()
    builder.setupCFF(ps_name, {'FullName': FAMILY, 'FamilyName': FAMILY, 'Weight': 'Regular'}, strings, {})
    builder.setupHorizontalMetrics({name: source_horizontal_metrics(widths[name][0], value)
                                    for name, value in strings.items()})
    output.parent.mkdir(parents=True, exist_ok=True)
    builder.save(output)
    evidence = {'role': ROLE, 'input_sha256': manifest['input_sha256'],
                'pdf': str(pdf), 'pdf_sha256': digest(pdf), 'font': str(output),
                'font_sha256': digest(output), 'family': FAMILY,
                'glyph_transform': 'identity; no target PDF, shift, scale or contour correction',
                'source_glyph_sha256': {f'{c:04X}': hashlib.sha256(value.encode()).hexdigest()
                                        for c, value in sorted(glyphs.items())}}
    output.with_suffix('.provenance.json').write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    create_parser = commands.add_parser('create')
    create_parser.add_argument('--template', type=Path, required=True)
    create_parser.add_argument('--output', type=Path, required=True)
    recover_parser = commands.add_parser('recover')
    recover_parser.add_argument('--pdf', type=Path, required=True)
    recover_parser.add_argument('--manifest', type=Path, required=True)
    recover_parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'create':
        create(args.template, args.output)
    else:
        recover(args.pdf, args.manifest, args.output)


if __name__ == '__main__':
    main()
