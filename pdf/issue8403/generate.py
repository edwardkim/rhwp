#!/usr/bin/env python3
"""Public synthetic PDF controls for #8403. See README.md for provenance."""
from pathlib import Path
import argparse
import pymupdf as fitz
parser=argparse.ArgumentParser(description='Regenerate public #8403 PDF controls; no private source is used.')
parser.add_argument('--out',type=Path,required=True)
ROOT=parser.parse_args().out
ROOT.mkdir(parents=True,exist_ok=True)
REPO=Path(__file__).resolve().parents[2]
class PDF:
 def __init__(self): self.objects=[b'',b'']; self.pages=[]
 def obj(self,s):
  self.objects.append(s.encode() if isinstance(s,str) else s); return len(self.objects)
 def stream(self,attrs,data):
  if isinstance(data,str): data=data.encode()
  return self.obj(f'<< {attrs} /Length {len(data)} >>\nstream\n'.encode()+data+b'\nendstream')
 def page(self,res,ops):
  c=self.stream('',ops); self.pages.append(self.obj(f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 140] /Resources << {res} >> /Contents {c} 0 R >>'))
 def write(self,path):
  self.objects[0]=b'<< /Type /Catalog /Pages 2 0 R >>'; self.objects[1]=f'<< /Type /Pages /Count {len(self.pages)} /Kids [ '.encode()+b' '.join(f'{x} 0 R'.encode() for x in self.pages)+b' ] >>'
  out=b'%PDF-1.7\n'; offsets=[0]
  for i,v in enumerate(self.objects,1): offsets.append(len(out)); out+=f'{i} 0 obj\n'.encode()+v+b'\nendobj\n'
  x=len(out);out+=f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()+b''.join(f'{o:010d} 00000 n \n'.encode() for o in offsets[1:]);out+=f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{x}\n%%EOF\n'.encode();path.write_bytes(out)
p=PDF()
mask=p.stream('/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceGray /BitsPerComponent 8',bytes([255,128,64,0]))
im=p.stream('/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8',bytes([255,0,0,0,255,0,0,0,255,255,255,0]))
masked=p.stream(f'/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask {mask} 0 R',bytes([255,0,0]*4))
def pat(img=im,matrix='1 0 0 1 7 9',dx=30,dy=18,ops=None):
 return p.stream(f'/Type /Pattern /PatternType 1 /PaintType 1 /TilingType 1 /BBox [-5 -3 25 15] /XStep {dx} /YStep {dy} /Matrix [{matrix}] /Resources << /XObject << /Im {img} 0 R >> >>', ops or 'q 20 0 0 12 0 0 cm /Im Do Q')
a=pat()
p.page(f'/Pattern << /P {a} 0 R >>','0.8 g 0 0 200 140 re f q 1 0 0 1 10 8 cm /Pattern cs /P scn 3 w 0 G 12 10 110 70 re B Q 10 110 40 15 re f')
b=pat(matrix='1 .2 .3 1 30 30',dx=-30,dy=-18)
p.page(f'/Pattern << /P {b} 0 R >>','q 10 10 m 170 20 l 140 110 l h W n /Pattern cs /P scn 0 0 190 130 re f Q')
c=pat(matrix='1 0 0 1 0 0')
f=p.stream(f'/Type /XObject /Subtype /Form /BBox [0 0 100 80] /Matrix [1.2 0 0 1.2 25 20] /Resources << /Pattern << /P {c} 0 R >> >>','/Pattern cs /P scn 0 0 100 80 re f')
p.page(f'/Pattern << /P {b} 0 R >> /XObject << /F {f} 0 R >>','/F Do')
d=pat(img=masked)
p.page(f'/Pattern << /P {d} 0 R >> /ExtGState << /Half << /ca .5 >> >>','0 0 1 rg 0 0 200 140 re f q /Half gs /Pattern cs /P scn 10 10 m 180 10 l 100 120 l h f Q')
p.page(f'/Pattern << /P {a} 0 R >>','/Pattern cs /P scn q 5 5 80 50 re f Q q 1 0 0 1 100 50 cm 5 5 80 50 re f Q')
p.write(ROOT/'image-patterns.pdf')
# Malformed and unsupported paints must refuse, not hang or report success.
for name,change in [('zero-step', b'/XStep 0 '),('huge-tiles',b'/XStep .000001 '),('unsupported-paint',None)]:
 raw=(ROOT/'image-patterns.pdf').read_bytes()
 # Use PyMuPDF to preserve a valid xref when changing a stream dictionary.
 import pymupdf as fitz
 doc=fitz.open(stream=raw,filetype='pdf')
 if change: doc.xref_set_key(a,'XStep',change.split()[1].decode())
 else: doc.update_stream(a,b'0 0 20 12 re f')
 doc.save(ROOT/(name+'.pdf'))

# Pattern recursion and dictionary validation controls.
for name,key,value in [('uncoloured-pattern','PaintType','2'),('shading-pattern','PatternType','2'),('singular-pattern','Matrix','[0 0 0 0 0 0]')]:
 d=fitz.open(ROOT/'image-patterns.pdf');d.xref_set_key(a,key,value);d.save(ROOT/(name+'.pdf'))
d=fitz.open(ROOT/'image-patterns.pdf');d.xref_set_key(a,'Resources',f'<< /Pattern << /Self {a} 0 R >> >>');d.update_stream(a,b'/Pattern cs /Self scn 0 0 20 10 re f');d.save(ROOT/'cyclic-pattern.pdf')

font=str(REPO/'ttfs/opensource/NotoSansKR-Regular.ttf')
def make_text(filename,word):
 d=fitz.open()
 for mode in range(6):
  p=d.new_page(width=420,height=240);p.insert_font(fontname='Fixture',fontfile=font)
  p.insert_text((20,65),word,fontsize=34,fontname='Fixture')
  x=p.get_contents()[-1];ops=d.xref_stream(x)
  if mode==0: before=b'1.715 w 2 Tr 0 0 0 RG '
  elif mode==1: before=b'1.2 w 1 Tr 0 0 .8 RG '
  elif mode==2:
   resources=int(d.xref_get_key(p.xref,'Resources')[1].split()[0])
   d.xref_set_key(resources,'ExtGState','<< /GS << /ca .5 /CA .75 /LW 1.1 /LJ 1 /ML 3 >> >>')
   before=b'/GS gs 2 Tr 0 0 .8 RG 1 .1 .1 rg '
  elif mode==3: before=b'[] 0 d 1.715 w 2 Tr 1 j 2 M 0 .4 0 RG 1 0 0 1.00001 0 0 cm '
  elif mode==4: before=b'1.715 w 2 Tr 0 0 0 RG 1.2 0 0 1.2 0 -20 cm '
  else: before=b'0 w 1 Tr 0 0 0 RG '
  # Put state inside q before the text object, so the later control proves Q restoration.
  ops=ops.replace(b'q\n',b'q\n'+before,1);d.update_stream(x,ops)
  p.insert_text((20,150),word,fontsize=34,fontname='Fixture')
 source=fitz.open(stream=d.tobytes(),filetype='pdf')
 p=d.new_page(width=420,height=240);p.show_pdf_page(fitz.Rect(20,20,400,220),source,0)
 d.subset_fonts();d.save(ROOT/filename,garbage=4,deflate=True)

make_text('text-stroke.pdf','Stroke 한글')
make_text('text-stroke-edited.pdf','ktroke 한글')

source=fitz.open(ROOT/'text-stroke.pdf')
for name,ops in [('dashed-text',b'[2 2] 0 d 1.715 w 2 Tr'),('anisotropic-text',b'1.715 w 2 Tr 2 0 0 1 0 0 cm'),('clipped-text-stroke',b'1.715 w 6 Tr')]:
 d=fitz.open();d.insert_pdf(source,from_page=0,to_page=0);p=d[0];x=p.get_contents()[0];d.update_stream(x,d.xref_stream(x).replace(b'1.715 w 2 Tr',ops));d.save(ROOT/(name+'.pdf'))
d=fitz.open();d.insert_pdf(source,from_page=0,to_page=0);p=d[0];pat=d.get_new_xref();d.update_object(pat,'<< /Type /Pattern /PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 4 4] /XStep 8 /YStep 8 /Resources << >> >>');d.update_stream(pat,b'1 0 0 rg 0 0 4 4 re f');res=int(d.xref_get_key(p.xref,'Resources')[1].split()[0]);d.xref_set_key(res,'Pattern',f'<< /P {pat} 0 R >>');x=p.get_contents()[0];d.update_stream(x,d.xref_stream(x).replace(b'1.715 w 2 Tr',b'/Pattern cs /P scn 1.715 w 2 Tr'));d.save(ROOT/'pattern-text.pdf')
