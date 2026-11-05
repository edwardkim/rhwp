# gen_picrow.py <781.hwpx> <out.hwpx> <fills> <before> <after>
# 781 축소본(문단 0·178·179·179·180·[170×fills]·182·183): 마지막 차트 표 두 칸의 그림 문단 앞/뒤에 라벨 줄을 복제
import sys, zipfile, re
src, dst, fills, before, after = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
z = zipfile.ZipFile(src)
sec = z.read('Contents/section0.xml').decode('utf-8')
head_end = sec.index('>', sec.index('<hs:sec')) + 1
body = sec[head_end:sec.rindex('</hs:sec>')]
paras, depth, start = [], 0, None
for m in re.finditer(r'<hp:p[\s>]|</hp:p>', body):
    if m.group().startswith('</'):
        depth -= 1
        if depth == 0: paras.append(body[start:m.end()])
    else:
        if depth == 0: start = m.start()
        depth += 1
order = [0, 178, 179, 179, 180] + [170] * fills + [182, 183]
chosen = [paras[k] for k in order]
last = chosen[-1]
LSA = re.compile(r'<hp:linesegarray>.*?</hp:linesegarray>', re.S)
def edit_cell(cell):
    sub_start = cell.index('>', cell.index('<hp:subList')) + 1
    sub_end = cell.index('</hp:subList>')
    inner = cell[sub_start:sub_end]
    ps, d, st = [], 0, None
    for m in re.finditer(r'<hp:p[\s>]|</hp:p>', inner):
        if m.group().startswith('</'):
            d -= 1
            if d == 0: ps.append(inner[st:m.end()])
        else:
            if d == 0: st = m.start()
            d += 1
    label = next(p for p in ps if '<hp:t>' in p and '<hp:pic' not in p)
    pic_i = next(i for i, p in enumerate(ps) if '<hp:pic' in p)
    lab = LSA.sub('', label)
    new = ps[:pic_i] + [lab] * before + [ps[pic_i]] + [lab] * after + ps[pic_i + 1:]
    new = [LSA.sub('', p) for p in new]
    return cell[:sub_start] + ''.join(new) + cell[sub_end:]
out_last = re.sub(r'<hp:tc .*?</hp:tc>', lambda m: edit_cell(m.group()), last, flags=re.S)
chosen[-1] = out_last
out = sec[:head_end] + ''.join(chosen) + '</hs:sec>'
with zipfile.ZipFile(dst, 'w') as w:
    for info in z.infolist():
        w.writestr(info, out.encode('utf-8') if info.filename == 'Contents/section0.xml' else z.read(info.filename))
