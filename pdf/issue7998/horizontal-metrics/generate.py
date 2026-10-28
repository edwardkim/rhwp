from pathlib import Path
import argparse,json,hashlib
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--font',choices=['Arial','Helvetica'],default='Arial');p.add_argument('--arial-font',type=Path);p.add_argument('--paint',choices=['native','rectangle','compound','closed-large','alpha','occluded'],default='rectangle');a=p.parse_args();a.out.mkdir(exist_ok=False)
def text(x,y,sequence):return f'BT /F1 12 Tf 0 Tw 0 Tc 100 Tz 1 0 0 1 {x} {y} Tm {sequence} ET'
# Helvetica AFM A=667/1000 em. The gap is an independent PDF TJ/Tw
# declaration; the expected single blank is annotated, not read from rhwp IR.
font_source = None
if a.font == 'Arial':
 if a.arial_font is None: p.error('--arial-font is required for the explicit TrueType widths')
 from fontTools.ttLib import TTFont
 font = TTFont(a.arial_font); upem = font['head'].unitsPerEm; cmap=font.getBestCmap(); widths=[round(font['hmtx'].metrics[cmap[c]][0]*1000/upem)for c in range(32,127)]
 def metric(value): return round(value*1000/upem)
 head=font['head']; psname=font['name'].getDebugName(6)
 font_object=(f'<< /Type /Font /Subtype /TrueType /BaseFont /{psname} /Encoding /WinAnsiEncoding /FirstChar 32 /LastChar 126 /Widths ['+' '.join(map(str,widths))+'] /FontDescriptor 6 0 R >>').encode()
 descriptor=f'<< /Type /FontDescriptor /FontName /{psname} /Flags 32 /FontBBox [{metric(head.xMin)} {metric(head.yMin)} {metric(head.xMax)} {metric(head.yMax)}] /Ascent {metric(font["hhea"].ascent)} /Descent {metric(font["hhea"].descent)} /CapHeight 716 /StemV 80 /ItalicAngle 0 >>'.encode()
 font_source={'sha256':hashlib.sha256(a.arial_font.read_bytes()).hexdigest(),'postscriptName':psname,'unitsPerEm':upem,'firstChar':32,'widths':widths,'embedded':False}
else:
 font_object=b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>';descriptor=None
cases={
 'wide-inferred':('[(A) -1250 (B)] TJ',15,'inferred',True),
 'narrow-inferred':('[(A) -240 (B)] TJ',2.88,'inferred',True),
 'wide-explicit':('11.664 Tw (A B) Tj',15,'original space',True),
 'too-wide':('[(A) -2250 (B)] TJ',27,'inferred',False),
 'ordinary':('[(A) -500 (B)] TJ',6,'inferred',True),
}
for key,(sequence,gap,kind,representable)in cases.items():
 if a.paint in ['compound','closed-large','alpha','occluded'] and key!='wide-inferred':continue
 if a.paint=='native':
  ops=['0 G 0.4251968503937008 w 20 100 m 340 100 l S 20 220 m 340 220 l S 20 100 m 20 220 l S 340 100 m 340 220 l S 180 100 m 180 220 l S','q 20 100 160 120 re W n',text(30,197,sequence),text(30,180,'(C D) Tj'),'Q','q 180 100 160 120 re W n',text(190,197,'(CONTROL CELL) Tj'),'Q',text(20,70,'(AFTER TABLE) Tj')]
 elif a.paint=='compound':
  ops=['0 G 0.5 w 20 240 m 340 240 l 20 360 m 340 360 l 20 240 m 20 360 l 340 240 m 340 360 l 180 240 m 180 360 l S','q 20 240 160 120 re W n',text(30,337,sequence),text(30,320,'(C D) Tj'),'Q','q 180 240 160 120 re W n',text(190,337,'(CONTROL CELL) Tj'),'Q',text(20,70,'(AFTER TABLE) Tj')]
 elif a.paint=='closed-large':
  ops=['0 G 0.5 w 20 240 320 120 re S 180 240 m 180 360 l S','q 20 240 160 120 re W n',text(30,337,sequence),text(30,320,'(C D) Tj'),'Q','q 180 240 160 120 re W n',text(190,337,'(CONTROL CELL) Tj'),'Q',text(20,70,'(AFTER TABLE) Tj')]
 elif a.paint=='alpha':
  ops=['q /GS1 gs 0 G 0.5 w 20 100 320 120 re S Q 0 G 0.5 w 180 100 m 180 220 l S','q 20 100 160 120 re W n',text(30,197,sequence),text(30,180,'(C D) Tj'),'Q','q 180 100 160 120 re W n',text(190,197,'(CONTROL CELL) Tj'),'Q',text(20,70,'(AFTER TABLE) Tj')]
 elif a.paint=='occluded':
  ops=['0 G 0.5 w 20 100 320 120 re S 180 100 m 180 220 l S 1 g 19 135 3 20 re f 0 g','q 20 100 160 120 re W n',text(30,197,sequence),text(30,180,'(C D) Tj'),'Q','q 180 100 160 120 re W n',text(190,197,'(CONTROL CELL) Tj'),'Q',text(20,70,'(AFTER TABLE) Tj')]
 else:
  ops=['0 G 0.5 w 20 100 320 120 re S 180 100 m 180 220 l S','q 20 100 160 120 re W n',text(30,197,sequence),text(30,180,'(C D) Tj'),'Q','q 180 100 160 120 re W n',text(190,197,'(CONTROL CELL) Tj'),'Q',text(20,70,'(AFTER TABLE) Tj')]
 stream='\n'.join(ops).encode();resources=b'/Resources << /ExtGState << /GS1 << /Type /ExtGState /CA 0.5 >> >> /Font << /F1 5 0 R >> >>' if a.paint=='alpha' else b'/Resources << /Font << /F1 5 0 R >> >>';media = b'[0 0 600 400]' if a.paint in ['compound','closed-large'] else b'[0 0 400 260]';objs=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox '+media+b' '+resources+b' /Contents 4 0 R >>',f'<< /Length {len(stream)} >>\nstream\n'.encode()+stream+b'\nendstream',font_object];d=bytearray(b'%PDF-1.4\n');offsets=[]
 if descriptor: objs.append(descriptor)
 for i,obj in enumerate(objs,1):offsets.append(len(d));d.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 at=len(d);d.extend(f'xref\n0 {len(objs)+1}\n0000000000 65535 f \n'.encode())
 for off in offsets:d.extend(f'{off:010} 00000 n \n'.encode())
 d.extend(f'trailer\n<< /Size {len(objs)+1} /Root 1 0 R >>\nstartxref\n{at}\n%%EOF\n'.encode());dest=a.out/(key+'.pdf');dest.write_bytes(d)
 (a.out/(key+'-source.json')).write_text(json.dumps({'paintMode':a.paint,'source':'independently authored PDF operators and grouping annotation','font':a.font,'fontWidthSource':font_source,'sizePt':12,'blankAdvancePt':gap,'spaceOrigin':kind,'paragraphs':[['A B C D'],['CONTROL CELL']],'after':'AFTER TABLE','combinedFieldsRepresentable':representable,'bounds':{'ratio':[50,200],'spacing':[-50,50]},'sha256':hashlib.sha256(d).hexdigest()},indent=2)+'\n')
