from pathlib import Path
import pikepdf,io,zlib
from PIL import Image
out=Path(__file__).resolve().parent;pdf=pikepdf.Pdf.new();N=pikepdf.Name;D=pikepdf.Dictionary
def update(obj,values):
 for k,v in values.items():obj[k]=v
red=pdf.make_stream(bytes([220,25,35])*16);update(red,D(Type=N.XObject,Subtype=N.Image,Width=4,Height=4,ColorSpace=N.DeviceRGB,BitsPerComponent=8))
font=pdf.make_indirect(D(Type=N.Font,Subtype=N.Type1,BaseFont=N.Helvetica))
def page(commands,image=red):
 p=pdf.add_blank_page(page_size=(140,140));p.Resources=D(XObject=D(Im=image),Font=D(F=font));p.Contents=pdf.make_stream(commands.encode());return p
circle='110 70 m 110 92.09139 92.09139 110 70 110 c 47.90861 110 30 92.09139 30 70 c 30 47.90861 47.90861 30 70 30 c 92.09139 30 110 47.90861 110 70 c h'
page(f'q {circle} W n 120 0 0 120 10 10 cm /Im Do Q')
page('q 20 20 100 100 re 50 50 40 40 re W* n 100 0 0 100 20 20 cm /Im Do Q')
page('q 20 20 100 100 re 50 50 m 50 90 l 90 90 l 90 50 l h W n 100 0 0 100 20 20 cm /Im Do Q')
page('q 1 0 0 1 10 10 cm 0 0 m 100 0 l 100 100 50 100 v 0 100 0 0 y h W n q 100 0 0 100 0 0 cm /Im Do Q Q q 15 0 0 15 120 120 cm /Im Do Q')
for kind in ['flate-default','flate-predictor','jpeg']:
 mask=bytes([0,64,128,255])*4
 if kind=='jpeg':
  im=Image.frombytes('L',(4,4),mask);buffer=io.BytesIO();im.save(buffer,format='JPEG',quality=100);encoded=buffer.getvalue()
 else: encoded=zlib.compress(b''.join(b'\0'+mask[y*4:y*4+4] for y in range(4)) if kind=='flate-predictor' else mask)
 sm=pdf.make_stream(encoded);update(sm,D(Type=N.XObject,Subtype=N.Image,Width=4,Height=4,ColorSpace=N.DeviceGray,BitsPerComponent=8,Filter=N.DCTDecode if kind=='jpeg' else N.FlateDecode))
 if kind=='flate-default':sm.DecodeParms=D(Columns=4,Colors=1,BitsPerComponent=4)
 if kind=='flate-predictor':sm.DecodeParms=D(Columns=4,Colors=1,BitsPerComponent=8,Predictor=15)
 im=pdf.make_stream(bytes([25,110,220])*16);update(im,D(Type=N.XObject,Subtype=N.Image,Width=4,Height=4,ColorSpace=N.DeviceRGB,BitsPerComponent=8,SMask=sm))
 page('q 100 0 0 100 20 20 cm /Im Do Q',im)
page('BT /F 1 Tf 0 12 -15 0 60 25 Tm (Rotate) Tj ET BT /F 12 Tf 1 0 0 1 15 10 Tm (Control) Tj ET')
# Signed re dimensions must bound every oriented subpath without changing winding.
page('0.2 0.4 0.7 rg 20 120 40 -30 re 20 80 40 -50 re 120 120 -40 -90 re f')
form=pdf.make_stream((f'q {circle} W n 100 0 0 100 20 20 cm /Im Do Q').encode())
update(form,D(Type=N.XObject,Subtype=N.Form,BBox=[15,15,125,125],Resources=D(XObject=D(Im=red)),Matrix=[0.7,0.2,-0.2,0.7,35,10]))
p=page('q /Fm Do Q');p.Resources.XObject.Fm=form
# Ordinary image CTM (not a pattern) includes rotation/reflection.
pattern=bytes([220,25,35,25,110,220,40,180,70,240,210,30])*4
multi=pdf.make_stream(pattern);update(multi,D(Type=N.XObject,Subtype=N.Image,Width=4,Height=4,ColorSpace=N.DeviceRGB,BitsPerComponent=8))
page('q 70 20 -20 70 35 20 cm /Im Do Q',multi)
empty=pdf.make_indirect(D(Type=N.Font,Subtype=N.Type3,FontBBox=[0,0,1,1],FontMatrix=[1,0,0,1,0,0],FirstChar=65,LastChar=66,Widths=[0.6,0.6],Encoding=D(Type=N.Encoding,Differences=[65,N.A,N.B]),CharProcs=D(A=pdf.make_stream(b'0.6 0 0 0 0.6 1 d1'),B=pdf.make_stream(b'0.6 0 0 0 0.6 1 d1')),Resources=D()))
p=page('0.2 0.4 0.7 rg 30 50 80 40 re f BT /Empty 50 Tf 1 0 0 1 35 60 Tm (AB) Tj ET 0 g BT /F 12 Tf 1 0 0 1 15 15 Tm (Visible) Tj ET')
p.Resources.Font.Empty=empty
# PDF RGB is already a display colour, not Hancom's RGB555 print transfer.
pixels=bytearray()
for y in range(64):
 for x in range(64):
  pixels.extend((239,239,239) if y<32 else (x*4,y*4,(x+y)*2))
color=pdf.make_stream(bytes(pixels));update(color,D(Type=N.XObject,Subtype=N.Image,Width=64,Height=64,ColorSpace=N.DeviceRGB,BitsPerComponent=8))
page('q 100 0 0 100 20 20 cm /Im Do Q',color)
pdf.save(out/'curve-mask-controls.pdf')
print('pages',len(pdf.pages))
