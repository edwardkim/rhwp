# p189 경계 합성: 1342000 셀 문단(휴먼명조 10pt, 자간 -10%, condense 10%, 양쪽)을
# 본문 폭 26705+d HU 로 바꿔 가며 반복한다. LineSeg 없음(한/글이 직접 조판).
import re, sys, zipfile, shutil, os
tpl, out = sys.argv[1], sys.argv[2]
DS = [int(x) for x in sys.argv[3].split(',')]
TEXT = sys.argv[4] if len(sys.argv) > 4 else "건강과 생활환경(전염성·비전염성 질병, 기호품 및 약물의 오·남용, 영양 등)의 관계를 이해하고, 건강한 생활 습관을 실천한다."
z = zipfile.ZipFile(tpl)
files = {n: z.read(n) for n in z.namelist()}
h = files['Contents/header.xml'].decode()
h = re.sub(r'(<hh:font id="2" face=")[^"]*(")', r'\g<1>휴먼명조\2', h)
def edit(tag, id, fn):
    global h
    m = re.search(r'<hh:%s id="%s".*?</hh:%s>' % (tag, id, tag), h, re.S)
    h = h[:m.start()] + fn(m.group(0)) + h[m.end():]
    return m.group(0)
edit('charPr', 7, lambda c: re.sub(r'(<hh:spacing )[^/]*/>', r'\1hangul="-10" latin="-10" hanja="-10" japanese="-10" other="-10" symbol="-10" user="-10"/>', c))
def para(c):
    c = c.replace('condense="75"', 'condense="10"').replace('snapToGrid="1"', 'snapToGrid="0"')
    c = c.replace('breakNonLatinWord="BREAK_WORD"', 'breakNonLatinWord="KEEP_WORD"')
    return c.replace('value="160"', 'value="130"')
base = edit('paraPr', 32, para)
base = para(base)
TW = 59528 - 2 * 8504
new = []
for i, d in enumerate(DS):
    r = TW - (26705 + d)
    p = re.sub(r'id="32"', 'id="%d"' % (33 + i), base, count=1)
    p = p.replace('<hc:right value="0" unit="HWPUNIT"/>', '<hc:right value="%d" unit="HWPUNIT"/>' % r)
    new.append(p)
end = h.find('</hh:paraProperties>')
h = h[:end] + ''.join(new) + h[end:]
h = h.replace('<hh:paraProperties itemCnt="33"', '<hh:paraProperties itemCnt="%d"' % (33 + len(DS)))
files['Contents/header.xml'] = h.encode()
s = files['Contents/section0.xml'].decode()
first_end = s.find('</hp:p>') + len('</hp:p>')
body = []
for i, d in enumerate(DS):
    body.append('<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>d=%d</hp:t></hp:run></hp:p>' % d)
    body.append('<hp:p id="0" paraPrIDRef="%d" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="7"><hp:t>%s</hp:t></hp:run></hp:p>' % (33 + i, TEXT))
s = s[:first_end] + ''.join(body) + '</hs:sec>'
files['Contents/section0.xml'] = s.encode()
with zipfile.ZipFile(out, 'w') as o:
    for n in z.namelist():
        o.writestr(z.getinfo(n), files[n], compress_type=zipfile.ZIP_STORED if n == 'mimetype' else zipfile.ZIP_DEFLATED)
