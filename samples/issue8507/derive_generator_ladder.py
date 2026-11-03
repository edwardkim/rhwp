#!/usr/bin/env python3
"""derive_generator_ladder.py <한/글 저장 HWPX> <출력 HWPX> (#8507)

결재문서 생성기의 꼴을 만든다. 한/글 2020 이 저장한 일관된 문서에서
- 첫 표(A)의 선언 높이 hp:sz height 를 7536 → 8160HU 로 올리고(행 합은 그대로 7536HU),
- 표 A host 문단의 linesegarray 를 지우고(생성기는 표 host 줄을 쓰지 않는다),
- 그 뒤 최상위 문단의 저장 vertpos 를 +624HU 옮긴다(생성기가 선언 높이로 뒤 사다리를 적는다).
칸 안 문단과 다른 표는 건드리지 않는다."""
import re, sys, zipfile

src, dst = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(src)
s = z.read('Contents/section0.xml').decode()
i = s.index('<hp:sz ')
j = s.index('>', i)
assert 'height="7536"' in s[i:j]
s = s[:i] + s[i:j].replace('height="7536"', 'height="8160"') + s[j:]
out, pos, depth, after_a, in_a = [], 0, 0, False, False
for m in re.finditer(r'<hp:subList|</hp:subList>|<hp:tbl |<hp:linesegarray>.*?</hp:linesegarray>', s, re.S):
    t = m.group(0)
    out.append(s[pos:m.start()])
    pos = m.end()
    if t.startswith('<hp:subList'):
        depth += 1
    elif t.startswith('</hp:subList'):
        depth -= 1
    elif t.startswith('<hp:tbl ') and depth == 0 and not after_a and not in_a:
        in_a = True
    elif t.startswith('<hp:linesegarray') and depth == 0:
        if in_a:
            in_a, after_a = False, True
            continue
        if after_a:
            t = re.sub(r'vertpos="(\d+)"', lambda v: f'vertpos="{int(v.group(1)) + 624}"', t)
    out.append(t)
out.append(s[pos:])
s = ''.join(out)
assert after_a
with zipfile.ZipFile(dst, 'w') as zo:
    for info in z.infolist():
        raw = s.encode() if info.filename == 'Contents/section0.xml' else z.read(info.filename)
        zo.writestr(info, raw, compress_type=zipfile.ZIP_STORED if info.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
