#!/usr/bin/env python3
"""Independent PDF geometry: convex winding, transformed Form BBox and reflection.

The low-resolution solid image deliberately exercises geometric clip quality;
source bitmap resolution must not turn a diagonal into large square blocks.
"""
from pathlib import Path
objects=[]
def obj(s):
 objects.append(s.encode() if isinstance(s,str) else s);return len(objects)
def stream(s,attrs=''):
 return f'<< /Length {len(s)} {attrs} >>\nstream\n'.encode()+s+b'\nendstream'
obj('<< /Type /Catalog /Pages 2 0 R >>');obj('')
image=obj(stream(bytes([255,0,0])*4,'/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8'))
form=obj(stream(b'q 70 0 0 70 -5 -5 cm /Im Do Q',f'/Type /XObject /Subtype /Form /BBox [0 0 60 60] /Resources << /XObject << /Im {image} 0 R >> >>'))
contents=[
 b'q 20 20 m 100 20 l 20 100 l h W n 80 0 0 80 20 20 cm /Im Do Q',
 b'q 15 15 100 100 re W n 0.8660254 0.5 -0.5 0.8660254 55 20 cm /Fm Do Q',
 b'q 20 100 m 100 100 l 100 20 l h W* n -80 0 0 80 100 20 cm /Fade gs /Im Do Q',
]
kids=[]
for content in contents:
 s=obj(stream(content));kids.append(obj(f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 130 130] /Resources << /XObject << /Im {image} 0 R /Fm {form} 0 R >> /ExtGState << /Fade << /Type /ExtGState /ca 0.5 >> >> >> /Contents {s} 0 R >>'))
objects[1]=f'<< /Type /Pages /Count 3 /Kids [{" ".join(f"{k} 0 R" for k in kids)}] >>'.encode()
result=bytearray(b'%PDF-1.4\n');offsets=[]
for i,content in enumerate(objects,1):
 offsets.append(len(result));result+=f'{i} 0 obj\n'.encode()+content+b'\nendobj\n'
xref=len(result);result+=f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode()
for off in offsets:result+=f'{off:010} 00000 n \n'.encode()
result+=f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode()
p=Path(__file__).with_name('affine-clip-controls.pdf')
if p.exists() and p.read_bytes()!=result:raise SystemExit('Existing fixture differs; preserve and review it first.')
p.write_bytes(result)
