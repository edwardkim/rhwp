import re, sys, zipfile
src, dst, W = sys.argv[1], sys.argv[2], int(sys.argv[3])
z = zipfile.ZipFile(src); s = z.read('Contents/section0.xml').decode()
i = s.index('<hp:tbl '); j = s.index('</hp:tbl>', i)
t = s[i:j]; old = int(re.search(r'<hp:sz width="(\d+)"', t).group(1)); d = W - old
t = t.replace(f'<hp:sz width="{old}"', f'<hp:sz width="{W}"', 1)
# widen the last cell of each row by d
def row(m):
    r = m.group(0); cs = list(re.finditer(r'<hp:cellSz width="(\d+)"', r)); c = cs[-1]
    return r[:c.start()] + f'<hp:cellSz width="{int(c.group(1))+d}"' + r[c.end():]
t = re.sub(r'<hp:tr>.*?</hp:tr>', row, t, flags=re.S)
s = s[:i] + t + s[j:]
with zipfile.ZipFile(dst, 'w') as zo:
    for info in z.infolist():
        raw = s.encode() if info.filename == 'Contents/section0.xml' else z.read(info.filename)
        zo.writestr(info, raw, compress_type=zipfile.ZIP_STORED if info.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
print(old, '->', W)
