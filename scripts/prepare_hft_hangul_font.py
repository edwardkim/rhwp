"""한컴 Print의 글자와 기호 비교에 사용할 독립 한글 HWPX 표본을 작성한다."""
from pathlib import Path
import argparse
import hashlib
import json
import copy
import zipfile
import xml.etree.ElementTree as ET


def validate_face(face):
    if (not isinstance(face, str) or not face.strip() or len(face) > 256
            or any(ord(character) < 32 or ord(character) == 127 for character in face)):
        raise ValueError('An explicit nonempty HFT face is required')
    return face


# 표준 글머리 기호 집합이다. 특정 문서의 문자 목록이나 좌표에서 만들지 않는다.
# 독립 한글 표본의 끝에 원문을 그대로 넣는다.
BULLET_CHARACTERS = '·•∙○◦'
ARROW_CHARACTERS = '←↑→↓➜'


def specimen_characters(include_bullets=False, include_arrows=False):
    return (''.join(chr(value) for value in range(0xAC00, 0xD7A4))
            + (BULLET_CHARACTERS if include_bullets else '')
            + (ARROW_CHARACTERS if include_arrows else ''))


def create_specimen(template, output, face="한양신명조", include_bullets=False, include_arrows=False):
    """한컴 빈 HWPX의 쪽/스타일을 쓰되 저장 줄과 기존 내용을 보존하지 않는다."""
    face = validate_face(face)
    hp = 'http://www.hancom.co.kr/hwpml/2011/paragraph'
    hh = 'http://www.hancom.co.kr/hwpml/2011/head'
    with zipfile.ZipFile(template) as source:
        header = ET.fromstring(source.read('Contents/header.xml'))
        section = ET.fromstring(source.read('Contents/section0.xml'))
        if len(section) != 1 or not list(section[0].iter('{' + hp + '}secPr')):
            raise ValueError('A blank single-section template is required')
        for font in header.iter('{' + hh + '}font'):
            font.set('face', face)
            font.set('type', 'HFT')
        paragraph = copy.deepcopy(section[0])
        for run in paragraph.findall('{' + hp + '}run'):
            for child in list(run):
                if child.tag.split('}')[-1] not in ['secPr', 'ctrl']:
                    run.remove(child)
        for child in list(paragraph):
            if child.tag != '{' + hp + '}run':
                paragraph.remove(child)
        characters = specimen_characters(include_bullets, include_arrows)
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
              'requestedFace': face, 'requestedFontType': 'HFT', 'characters': len(characters),
              'standardBullets': BULLET_CHARACTERS if include_bullets else '',
              'standardArrows': ARROW_CHARACTERS if include_arrows else ''}
    output.with_suffix('.provenance.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(record, ensure_ascii=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('create-specimen', help='한컴 빈 HWPX로 독립 한글 표본 작성')
    create.add_argument('--template', type=Path, required=True)
    create.add_argument('--out', type=Path, required=True)
    create.add_argument('--face', default='한양신명조', help='독립 표본에 요청할 face')
    create.add_argument('--with-bullets', action='store_true', help='표준 글머리 기호 포함')
    create.add_argument('--with-arrows', action='store_true', help='표준 화살표 포함')
    args = parser.parse_args()
    create_specimen(args.template, args.out, args.face, args.with_bullets, args.with_arrows)


if __name__ == '__main__':
    main()
