"""한컴 Print의 구형 수식 기호 비교에 사용할 독립 HWPX 표본을 작성한다."""
from pathlib import Path
import argparse
import copy
import hashlib
import json
import string
import zipfile
import xml.etree.ElementTree as ET


HP = 'http://www.hancom.co.kr/hwpml/2011/paragraph'
ROLE = 'independent-legacy-equation-alphabet-v1'
ARITHMETIC_ROLE = 'independent-legacy-equation-alphabet-with-arithmetic-v2'
# 표본 manifest에 기록할 렌더러의 기존 요청 이름.
FAMILY = 'RHWP Hancom Legacy Equation'
ARITHMETIC_EXTRA = [('times', '×'), ('div', '÷')]
EXTRA = [
    ('+', '+'), ('-', '-'), ('=', '='), (':', ':'), ('<', '<'), ('>', '>'),
    (',', ','), ('.', '.'), ('/', '/'), ('(', '('), (')', ')'), ('[', '['),
    (']', ']'), ('alpha', 'α'), ('beta', 'β'), ('gamma', 'γ'), ('delta', 'δ'),
    ('Delta', 'Δ'), ('theta', 'θ'), ('pi', 'π'), ('sigma', 'σ'), ('lambda', 'λ'),
]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def probes(include_arithmetic=False):
    return [{'script': 'rm {' + c + '}', 'expected': c}
            for c in string.ascii_uppercase + string.ascii_lowercase + string.digits] + [
        {'script': script, 'expected': character}
        for script, character in (ARITHMETIC_EXTRA if include_arithmetic else []) + EXTRA]


def create(template, output, *, include_arithmetic=False):
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
        for index, item in enumerate(probes(include_arithmetic)):
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
    manifest = {'role': ARITHMETIC_ROLE if include_arithmetic else ROLE,
                'input': str(output.resolve()),
                'input_sha256': digest(output), 'font_family': FAMILY,
                'probes': probes(include_arithmetic), 'template_sha256': digest(template)}
    output.with_suffix('.manifest.json').write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    create_parser = commands.add_parser('create')
    create_parser.add_argument('--template', type=Path, required=True)
    create_parser.add_argument('--output', type=Path, required=True)
    create_parser.add_argument('--include-arithmetic-operators', action='store_true',
                               help='표본에 구형 ×·÷ 포함')
    args = parser.parse_args()
    create(args.template, args.output, include_arithmetic=args.include_arithmetic_operators)


if __name__ == '__main__':
    main()
