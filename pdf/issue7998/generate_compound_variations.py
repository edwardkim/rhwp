from pathlib import Path
import re,json,hashlib,argparse
parser=argparse.ArgumentParser(description='Independent compound PDF paint variations')
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args();b=args.output;b.mkdir(parents=True,exist_ok=True)
base=(Path(__file__).resolve().parent/'nested-dash-child.pdf').read_bytes()
stream=re.search(rb'4 0 obj\n.*?stream\n(.*?)\nendstream',base,re.S).group(1).decode()
def save(key,ops,annotation):
 objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 360] /Resources << /Font << /F1 5 0 R >> /XObject << /Im1 6 0 R >> /ExtGState << /GS1 7 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(ops.encode())} >>\nstream\n'.encode()+ops.encode()+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',b'<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 12 >>\nstream\n'+bytes([0,120,220,220,120,0,220,120,0,0,120,220])+b'\nendstream',b'<< /Type /ExtGState /CA 0.4 /ca 0.4 >>']
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,obj in enumerate(objects,1):offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 xref=len(data);data.extend(f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode())
 for offset in offsets:data.extend(f'{offset:010} 00000 n \n'.encode())
 data.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode());(b/(key+'.pdf')).write_bytes(data)
 (b/(key+'-source.json')).write_text(json.dumps({'source':'independent variation of original PDF operators; no engine output used as expected result','sha256':hashlib.sha256(data).hexdigest(),**annotation},indent=2)+'\n')
save('compound-phase-alpha',stream.replace('[2 2] 0 d','[2 2] 1.5 d /GS1 gs'),{'tables':2,'dashPhasePt':1.5,'strokeAlpha':0.4})
save('compound-clipped',stream.replace('[2 2] 0 d','[2 2] 0 d 39.75 219.75 140.5 60.5 re W n'),{'tables':2,'clipPt':[39.75,219.75,140.5,60.5]})
curve='q 0.1 0.7 0.9 rg 35 290 m 60 300 170 190 190 210 c 160 220 80 310 35 290 c f Q'
save('compound-backdrop',curve+'\n'+stream,{'tables':2,'paintOrder':'curved filled background before all labels; preserve winding, alpha and source crop'})
save('compound-foreground',stream+'\n'+curve,{'tables':0,'paintOrder':'curved foreground over labels; preserve original order, no behind-text lowering'})
