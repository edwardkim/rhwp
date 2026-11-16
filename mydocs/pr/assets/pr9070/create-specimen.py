"""Create a standalone HWP3 glyph specimen from the published 3.0 record layouts.

This is a synthetic input. It is not a Hancom-saved document. The declared A4
page, font, character ratio, size and sequential line positions are valid,
unlike the original issue attachment's zero page and zero character ratio.
"""
from pathlib import Path
import struct,json,hashlib
root=Path(__file__).resolve().parents[4]
pack=lambda fmt,*args:struct.pack('<'+fmt,*args)
header=b'HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05'
assert len(header)==30
info=bytearray(128);info[4]=3;info[125]=1
for offset,value in [(6,21047),(8,14882),(10,1417),(12,1417),(14,1417),(16,1417),(98,1),(100,1)]:struct.pack_into('<H',info,offset,value)
font=b'HCR Batang'.ljust(40,b'\0')
fonts=(pack('H',1)+font)*7
char_shape=bytearray(31);struct.pack_into('<H',char_shape,0,300);char_shape[9:16]=bytes([100]*7);char_shape[23]=7
para_shape=bytearray(187);struct.pack_into('<H',para_shape,6,160);para_shape[10]=100;para_shape[11]=1;para_shape[172]=1
for i in range(40):struct.pack_into('<BBH',para_shape,12+i*4,0,0,(i+1)*1000)
def chars(text):return list(map(ord,text))
rows=[chars('HWP3 line characters: seven sets and seven symbols')]
for g in range(7):rows.append(chars(f'set {g} 0x{0x3013+11*g:04X}: ')+list(range(0x3013+g*11,0x3013+(g+1)*11)))
rows.append(chars('Plain symbol values: ')+[0x048A,32,0x2051,32,0x2053,32,0x223C,32,0x3151,32,0x2F66,32,0x2191])
rows.append(chars('End of glyph specimen.'))
paragraphs=[]
for i,text in enumerate(rows):
 data=text+[13]
 pi=pack('BHHBBIB',0,len(data),1,0,0,1<<13,0)+char_shape+para_shape
 assert len(pi)==230
 line=pack('HhHHHHH',0,0,300,i*480,0,0,0x8000)
 paragraphs.append(pi+line+pack('H'*len(data),*data))
styles=pack('H',1)+b'Normal'.ljust(20,b'\0')+char_shape+para_shape
body=fonts+styles+b''.join(paragraphs)+bytes(43)+bytes(8)+pack('II',0x80000000,0)
blob=header+info+bytes(1008)+body
p=root/'samples/pr9070/hwp3-linechar-specimen.hwp';p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(blob)
r={'input':str(p.relative_to(root)),'sha256':hashlib.sha256(blob).hexdigest(),'bytes':len(blob),'origin':'locally authored synthetic HWP3 based on format 3.0 sections3.2/3.5/4/5/6','sourceCharCodes':'same 77 line codes and 7 symbols as issue8912; source names/geometry are independent','font':'HCR Batang','fontSizePt':12,'characterRatio':100,'paragraphLineSpacingPercent':160,'lineHeightHunit':300,'lineAdvanceHunit':480,'rows':len(rows)}
Path(__file__).with_name('hwp3-specimen-provenance.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
print(p)
