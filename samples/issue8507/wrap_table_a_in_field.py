import sys, zipfile
src, dst = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(src); s = z.read('Contents/section0.xml').decode()
FB = ('<hp:ctrl><hp:fieldBegin id="2021757239" type="CLICK_HERE" name="f" editable="1" dirty="1" zorder="-1" fieldid="627272811">'
      '<hp:parameters cnt="3" name=""><hp:integerParam name="Prop">9</hp:integerParam>'
      '<hp:stringParam name="Command" xml:space="preserve">Clickhere:set:38:Direction:wstring:1:f HelpState:wstring:0:  </hp:stringParam>'
      '<hp:stringParam name="Direction">f</hp:stringParam></hp:parameters></hp:fieldBegin></hp:ctrl>')
FE = '<hp:ctrl><hp:fieldEnd beginIDRef="2021757239" fieldid="627272811"/></hp:ctrl>'
i = s.index('<hp:tbl '); s = s[:i] + FB + s[i:]
j = s.index('</hp:tbl>') + len('</hp:tbl>')
k = s.index('<hp:run charPrIDRef="0">', j) + len('<hp:run charPrIDRef="0">')
s = s[:k] + FE + s[k:]
with zipfile.ZipFile(dst, 'w') as zo:
    for info in z.infolist():
        raw = s.encode() if info.filename == 'Contents/section0.xml' else z.read(info.filename)
        zo.writestr(info, raw, compress_type=zipfile.ZIP_STORED if info.filename == 'mimetype' else zipfile.ZIP_DEFLATED)
