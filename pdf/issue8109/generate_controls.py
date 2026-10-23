from pathlib import Path
import hashlib,json
from fontTools.ttLib import TTFont

out=Path(__file__).resolve().parent;out.mkdir(exist_ok=True)
source=Path('/usr/share/fonts/opentype/urw-base35/NimbusSans-Regular.otf')
font=TTFont(source);cff=font.getTableData('CFF ')

def stream(data,extra=''):
    if isinstance(data,str):data=data.encode()
    return f'<< /Length {len(data)} {extra} >>\nstream\n'.encode()+data+b'\nendstream'

def write(name,objects):
    pdf=b'%PDF-1.4\n';offsets=[]
    for index,value in enumerate(objects,1):
        offsets.append(len(pdf));pdf+=f'{index} 0 obj\n'.encode()+(value.encode()if isinstance(value,str)else value)+b'\nendobj\n'
    xref=len(pdf);pdf+=f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode()+b''.join(f'{offset:010} 00000 n \n'.encode()for offset in offsets)
    pdf+=f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode();path=out/(name+'.pdf');path.write_bytes(pdf)
    return {'name':name,'path':path.name,'sha256':hashlib.sha256(pdf).hexdigest()}

desc=' /Type /FontDescriptor /FontName /Helvetica /Flags 32 /FontBBox [-166 -225 1000 931] /ItalicAngle 0 /Ascent 718 /Descent -207 /CapHeight 718 /StemV 80 '
base=['<< /Type /Catalog /Pages 2 0 R >>','<< /Type /Pages /Count 1 /Kids [3 0 R] >>','<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 170] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',stream('BT /F1 12 Tf 20 110 Td (ALPHA LIST) Tj ET')]
cases=[]
for name,attrs,extra,expected in [
    ('absent','',[],True),
    ('inline-absent','/FontDescriptor <<'+desc+'>>',[],True),
    ('indirect-absent','/FontDescriptor 6 0 R',['<<'+desc+'>>'],True),
    ('embedded','/FontDescriptor 6 0 R',['<<'+desc+'/FontFile3 7 0 R >>',stream(cff,'/Subtype /Type1C')],False),
    ('unreadable-program','/FontDescriptor 6 0 R',['<<'+desc+'/FontFile3 7 0 R >>',stream(b'not-a-CFF-program','/Subtype /Type1C')],False),
    ('unresolved-descriptor','/FontDescriptor 77 0 R',[],False),
    ('invalid-descriptor','/FontDescriptor 42',[],False),
    ('arial','',[],False),
    ('non-type1','',[],False),
]:
    subtype='TrueType'if name=='non-type1'else'Type1';family='ArialMT'if name=='arial'else'Helvetica'
    row=write(name,base+[f'<< /Type /Font /Subtype /{subtype} /BaseFont /{family} /Encoding /WinAnsiEncoding {attrs} >>']+extra);row['expectArialSubstitute']=expected;cases.append(row)

styles=['Helvetica','Helvetica-Bold','Helvetica-Oblique','Helvetica-BoldOblique']
objects=['<< /Type /Catalog /Pages 2 0 R >>','<< /Type /Pages /Count 1 /Kids [3 0 R] >>','<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 240] /Resources << /Font << /F1 5 0 R /F2 6 0 R /F3 7 0 R /F4 8 0 R >> >> /Contents 4 0 R >>',stream('\n'.join(f'BT /F{i+1} 12 Tf 20 {200-i*40} Td (STYLE {i+1} ALPHA) Tj ET'for i in range(4)))]+[f'<< /Type /Font /Subtype /Type1 /BaseFont /{family} /Encoding /WinAnsiEncoding >>'for family in styles]
cases.append(write('styles',objects))

# Both Forms use F1 and declare Helvetica; only the second resource embeds a program.
objects=['<< /Type /Catalog /Pages 2 0 R >>','<< /Type /Pages /Count 1 /Kids [3 0 R] >>','<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << /XObject << /A 5 0 R /B 6 0 R >> >> /Contents 4 0 R >>',stream('q 1 0 0 1 0 100 cm /A Do Q q 1 0 0 1 0 40 cm /B Do Q'),stream('BT /F1 12 Tf 20 20 Td (UNEMBEDDED) Tj ET','/Type /XObject /Subtype /Form /BBox [0 0 300 60] /Resources << /Font << /F1 7 0 R >> >>'),stream('BT /F1 12 Tf 20 20 Td (EMBEDDED) Tj ET','/Type /XObject /Subtype /Form /BBox [0 0 300 60] /Resources << /Font << /F1 8 0 R >> >>'),'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>','<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding /FontDescriptor 9 0 R >>','<<'+desc+'/FontFile3 10 0 R >>',stream(cff,'/Subtype /Type1C')]
cases.append(write('form-scopes',objects))
(out/'manifest.json').write_text(json.dumps({'fontSource':'fonts-urw-base35 / NimbusSans-Regular.otf','fontSourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'purpose':'manual PDF contract controls; original bytes are independent visual references','cases':cases},indent=2)+'\n')
print('Generated',len(cases),'independent PDF controls')
