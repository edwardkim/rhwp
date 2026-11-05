import sys, zipfile
src, dst = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(src); s = z.read('Contents/section0.xml').decode()
i = s.index('<hp:tbl '); j = s.index('>', i)
head = s[i:j]; assert 'pageBreak="NONE"' in head
s = s[:i] + head.replace('pageBreak="NONE"', 'pageBreak="CELL"') + s[j:]
with zipfile.ZipFile(dst, 'w') as zo:
    for info in z.infolist():
        raw = s.encode() if info.filename == 'Contents/section0.xml' else z.read(info.filename)
        zo.writestr(info, raw, compress_type=zipfile.ZIP_STORED if info.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
