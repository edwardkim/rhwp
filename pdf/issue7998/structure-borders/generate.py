from pathlib import Path
import re
b=Path(__file__).resolve().parent
r=b.parents[2]
raw=(r/'pdf/issue7998/compound-borders/compound-solid.pdf').read_bytes()
ops=re.search(rb'4 0 obj\n.*?stream\n(.*?)\nendstream',raw,re.S).group(1)
base=ops.replace(b'0.5 w',b'0.4251968503937008 w').replace(b'1 0 0 1 20 12 Tm',b'1 0 0 1 20 -300 Tm')
for key,body in [('native-pen',base),('native-pen-blue',base.replace(b'q 0 G',b'q 0.2 0.4 0.6 RG'))]:
 body=b'q 1 0 0 1 0 360 cm\n'+body+b'\nQ'
 objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 720] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(body)} >>\nstream\n'.encode()+body+b'\nendstream',(b'<< /Type /Font /Subtype /Type1 /BaseFont /Arial >>'if key=='native-pen-blue'else b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>')]
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,obj in enumerate(objects,1):offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 xref=len(data);data.extend(f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode())
 for off in offsets:data.extend(f'{off:010} 00000 n \n'.encode())
 data.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode());(b/(key+'.pdf')).write_bytes(data)
