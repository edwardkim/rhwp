from pathlib import Path
import hashlib, json, struct

out = Path(__file__).resolve().parent / 'inputs'
out.mkdir(exist_ok=True)

def pdf(objects):
    data = bytearray(b'%PDF-1.7\n')
    offsets = [0]
    for i, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data.extend(f'{i} 0 obj\n'.encode() + obj + b'\nendobj\n')
    start = len(data)
    data.extend(f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode())
    for pos in offsets[1:]:
        data.extend(f'{pos:010d} 00000 n \n'.encode())
    data.extend(f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n'.encode())
    return data

def stream(data, extra=b''):
    return f'<< /Length {len(data)} '.encode() + extra + b' >>\nstream\n' + data + b'\nendstream'

def make_font_pdf(name, font, descriptor, program, content=b'BT /F1 12 Tf 10 150 Td (ABa) Tj ET'):
    objects = [b'<< /Type /Catalog /Pages 2 0 R >>', b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',
        b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 240 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',
        stream(content), font, descriptor, program]
    (out / (name+'.pdf')).write_bytes(pdf(objects))

type1 = b'/Encoding 256 array\ndup 65 /A put\ndup 66 /B put\ndup 97 /a put\nreadonly def\n'
make_font_pdf('type1-three-mappings', b'<< /Type /Font /Subtype /Type1 /BaseFont /AuditType1 /FontDescriptor 6 0 R >>',
    b'<< /Type /FontDescriptor /FontName /AuditType1 /Flags 32 /FontBBox [0 0 1000 1000] /Ascent 900 /Descent -200 /CapHeight 700 /ItalicAngle 0 /StemV 80 /FontFile 7 0 R >>', stream(type1))

def index(entries):
    if not entries:
        return b'\0\0'
    offsets = [1]
    for e in entries:
        offsets.append(offsets[-1]+len(e))
    size = max(1, (offsets[-1].bit_length()+7)//8)
    return struct.pack('>H',len(entries))+bytes([size])+b''.join(n.to_bytes(size,'big') for n in offsets)+b''.join(entries)

def number(n):
    return b'\x1d'+struct.pack('>i',n)

def cff(cid=False, glyphs=4):
    prefix=b'\1\0\4\1'+index([b'AuditCFF'])
    def dictionary(charset, charstrings):
        ros=(number(391)+number(392)+number(0)+b'\x0c\x1e') if cid else b''
        return ros+number(charset)+b'\x0f'+number(charstrings)+b'\x11'
    strings=index([b'Adobe',b'Identity']) if cid else index([])
    start=len(prefix)+len(index([dictionary(0,0)]))+len(strings)+2
    # A format-0 charset, genuine CharStrings INDEX, no fake xref repair.
    sids=list(range(1,glyphs)) if cid else [34,35,66]
    charset=b'\0'+b''.join(struct.pack('>H',n) for n in sids)
    chars=index([b'\x0e']*glyphs)
    return prefix+index([dictionary(start,start+len(charset))])+strings+b'\0\0'+charset+chars

make_font_pdf('cff-simple-four-glyphs', b'<< /Type /Font /Subtype /Type1 /BaseFont /AuditCFF /Encoding /WinAnsiEncoding /FontDescriptor 6 0 R >>',
    b'<< /Type /FontDescriptor /FontName /AuditCFF /Flags 32 /FontBBox [0 0 1000 1000] /Ascent 900 /Descent -200 /CapHeight 700 /ItalicAngle 0 /StemV 80 /FontFile3 7 0 R >>', stream(cff(),b'/Subtype /Type1C'))

make_font_pdf('cff-cid-1001-glyphs', b'<< /Type /Font /Subtype /Type0 /BaseFont /AuditCID /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType0 /BaseFont /AuditCID /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor 6 0 R >>] >>',
    b'<< /Type /FontDescriptor /FontName /AuditCID /Flags 32 /FontBBox [0 0 1000 1000] /Ascent 900 /Descent -200 /CapHeight 700 /ItalicAngle 0 /StemV 80 /FontFile3 7 0 R >>', stream(cff(True,1001),b'/Subtype /CIDFontType0C'), b'BT /F1 12 Tf 10 150 Td <004100420061> Tj ET')

# Already-budgeted CID widths are the independent negative control.
make_font_pdf('cid-width-control', b'<< /Type /Font /Subtype /Type0 /BaseFont /AuditWidthControl /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /AuditWidthControl /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /W [0 2 500] >>] >>',b'<< >>',stream(b''),b'BT /F1 12 Tf 10 150 Td <00010002> Tj ET')

(out.parent/'tiny-font-policy.json').write_text(json.dumps({'policy':{'backend':{'fontEntries':1}}})+'\n')
manifest=[]
for f in sorted(out.glob('*.pdf')):
    manifest.append({'name':f.name,'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'kind':'synthetic font/resource contract; no visual fidelity or executable exploit claim'})
(out.parent/'inputs.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('generated',len(manifest),'bounded synthetic PDFs')
