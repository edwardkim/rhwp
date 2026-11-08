# 휴먼명조 ASCII 전진폭 사다리: 문자마다 "가c"×20 한 문단(왼쪽 정렬).
import re, sys, zipfile
from xml.sax.saxutils import escape
tpl, out, height, spacing = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
z = zipfile.ZipFile(tpl); files = {n: z.read(n) for n in z.namelist()}
h = files['Contents/header.xml'].decode()
h = re.sub(r'(<hh:font id="2" face=")[^"]*(")', r'\g<1>휴먼명조\2', h)
m = re.search(r'<hh:charPr id="7".*?</hh:charPr>', h, re.S)
c = re.sub(r'(<hh:spacing )[^/]*/>', r'\1hangul="%d" latin="%d" hanja="%d" japanese="%d" other="%d" symbol="%d" user="%d"/>' % ((spacing,) * 7), m.group(0))
c = re.sub(r'height="\d+"', 'height="%d"' % height, c, count=1)
h = h[:m.start()] + c + h[m.end():]
m = re.search(r'<hh:paraPr id="32".*?</hh:paraPr>', h, re.S)
p = m.group(0).replace('horizontal="JUSTIFY"', 'horizontal="LEFT"').replace('condense="75"', 'condense="0"').replace('snapToGrid="1"', 'snapToGrid="0"')
h = h[:m.start()] + p + h[m.end():]
files['Contents/header.xml'] = h.encode()
s = files['Contents/section0.xml'].decode(); e = s.find('</hp:p>') + 7
body = ''.join('<hp:p id="0" paraPrIDRef="32" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="7"><hp:t>%s</hp:t></hp:run></hp:p>' % escape(("가" + chr(cp)) * 20 + "가") for cp in range(33, 127))
files['Contents/section0.xml'] = (s[:e] + body + '</hs:sec>').encode()
with zipfile.ZipFile(out, 'w') as o:
    for n in z.namelist():
        o.writestr(z.getinfo(n), files[n], compress_type=zipfile.ZIP_STORED if n == 'mimetype' else zipfile.ZIP_DEFLATED)
