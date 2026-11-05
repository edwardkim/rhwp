#!/usr/bin/env python3
"""저장 사다리가 문단 높이를 담지 못한 뒤의 vpos 0 — 합성 HWPX 입력.

  python gen_stored_ladder.py samples/issue2527_empty_linesegs.hwpx <out_dir>

본문 높이 = 84188 - 2*4251 - 2*2834 = 70018 HU, 한 줄 전진 = 1000 + 600 = 1600 HU,
한 쪽 = 44 줄.

missing_lines.hwpx (결함)
  저장 줄 문단 36 개(1600 간격) → 저장 줄(linesegarray)이 없는 문단 10 개 → 저장 줄 문단
  3 개(사다리는 앞 10 문단의 높이를 세지 않고 이어진다) → vpos 0 문단 → 꼬리 10 문단.
  저장 사다리만 보면 vpos 0 앞에서 쪽이 끝나지만 실제 내용은 그 전에 한 쪽을 넘긴다.
저장 줄은 이 스크립트가 쓴 값이며 한/글 저장본이 아니다. 기준은 한/글 2022 Print PDF 다.
"""
import os
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
parts = [(item.filename, src.read(item.filename)) for item in src.infolist()]
out_dir = sys.argv[2]
os.makedirs(out_dir, exist_ok=True)
sec = dict(parts)['Contents/section0.xml'].decode('utf-8', 'surrogateescape')
head = sec[:sec.find('<hp:bookmark')]
STEP = 1600


def seg(vpos):
    return ('<hp:linesegarray><hp:lineseg textpos="0" vertpos="%d" vertsize="1000" textheight="1000" '
            'baseline="850" spacing="600" horzpos="0" horzsize="48190" flags="393216"/></hp:linesegarray>' % vpos)


def write(name, label, rows):
    xml = (head + '</hp:run><hp:run charPrIDRef="0"><hp:t>%s</hp:t></hp:run>%s</hp:p>\n' % (label, seg(0)))
    for n, (text, vpos) in enumerate(rows):
        run = '<hp:run charPrIDRef="0"><hp:t>%s</hp:t></hp:run>' % text if text else '<hp:run charPrIDRef="0"/>'
        xml += ('<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">%s%s</hp:p>\n'
                % (900 + n, run, '' if vpos is None else seg(vpos)))
    xml += '</hs:sec>'
    path = os.path.join(out_dir, name)
    with zipfile.ZipFile(path, 'w') as dst:
        for filename, data in parts:
            if filename == 'Contents/section0.xml':
                data = xml.encode('utf-8', 'surrogateescape')
            info = zipfile.ZipInfo(filename, date_time=(2026, 11, 3, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED if filename == 'mimetype' else zipfile.ZIP_DEFLATED
            dst.writestr(info, data)
    print(path)


tail = [('TAIL %02d' % (i + 1), (i + 1) * STEP) for i in range(10)]
rows = [('STORED %02d' % (i + 1), (i + 1) * STEP) for i in range(36)]
rows += [('NOLINE %02d' % (i + 1), None) for i in range(10)]
rows += [('LAST %02d' % (i + 1), (37 + i) * STEP) for i in range(3)]
write('missing_lines.hwpx', 'missing stored lines', rows + [('RESET', 0)] + tail)
