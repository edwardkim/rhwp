"""#7873 문서 경로 회귀 입력: 9 개 수식 스크립트를 한 문단씩, 저장 줄 없이(한/글·rhwp 모두 조판).

  python gen_font_state_doc.py samples/issue7873/eq-font-state-template.hwpx samples/issue7873/eq-font-state-doc.hwpx
"""
import zipfile, re, html, sys

src = sys.argv[1]
dst = sys.argv[2]
SCRIPTS = [
    ("k", "{rm a} b"),
    ("m", "rm a {b} c"),
    ("n", "{rm a} over {b} c"),
    ("o", "it A rm B it C"),
    ("p", "rm a it b c"),
    ("r", "a rm b ^{c} d"),
    ("t", "{a rm b} c"),
    ("u", "rm a LEFT ( b RIGHT ) c"),
    ("y", "a rm {b} c"),
]
z = zipfile.ZipFile(src)
entries = [(it.filename, z.read(it.filename), it.compress_type, it.date_time, it.external_attr) for it in z.infolist()]
sec = dict((e[0], e[1]) for e in entries)["Contents/section0.xml"].decode("utf-8")
paras = re.findall(r"<hp:p .*?</hp:p>", sec, re.S)
head = sec[:sec.index(paras[0])]
tail = sec[sec.index(paras[-1]) + len(paras[-1]):]
tpl = paras[1]
out = [re.sub(r"<hp:linesegarray>.*?</hp:linesegarray>", "", paras[0], flags=re.S)]
pid = 100
eid = 2130999100
for key, scr in SCRIPTS:
    p = tpl
    p = re.sub(r"<hp:p id=\"\d+\"", "<hp:p id=\"%d\"" % pid, p)
    pid += 1
    p = re.sub(r"<hp:equation id=\"\d+\"", "<hp:equation id=\"%d\"" % eid, p)
    eid += 1
    p = re.sub(r"<hp:sz width=\"\d+\"", "<hp:sz width=\"16000\"", p)
    p = re.sub(r"height=\"\d+\" heightRelTo", "height=\"2400\" heightRelTo", p)
    p = re.sub(r"<hp:script>.*?</hp:script>", "<hp:script>" + html.escape(scr, quote=False) + "</hp:script>", p, flags=re.S)
    p = re.sub(r"<hp:t>[^<]*</hp:t><hp:equation", "<hp:t>" + key + ". </hp:t><hp:equation", p, count=1)
    p = re.sub(r"</hp:equation><hp:t>[^<]*</hp:t>", "</hp:equation>", p, count=1)
    p = re.sub(r"<hp:linesegarray>.*?</hp:linesegarray>", "", p, flags=re.S)
    out.append(p)
s2 = head + "".join(out) + tail
with zipfile.ZipFile(dst, "w") as zo:
    for n, d, ct, dt, ea in entries:
        if n == "Contents/section0.xml":
            d = s2.encode("utf-8")
        zi = zipfile.ZipInfo(n, date_time=dt)
        zi.compress_type = ct
        zi.external_attr = ea
        zo.writestr(zi, d)
print("paras", len(out), "scripts", s2.count("<hp:script>"))
