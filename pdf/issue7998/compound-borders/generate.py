from pathlib import Path
import re,json,hashlib
b=Path(__file__).resolve().parent
r=b.parents[2]
raw=(r/'pdf/issue7998/nested-dash-parent.pdf').read_bytes()
stream=re.search(rb'4 0 obj\n.*?stream\n(.*?)\nendstream',raw,re.S).group(1).decode()
a=stream.index('q 0 G');z=stream.index('Q',a);root=stream[a:z+1]
# Six independently authored MoveTo/LineTo subpaths, one shared stroke operation.
packed=re.sub(r' l S', ' l',root);packed=packed.replace('\nQ',' S\nQ')
for key,ops in [('compound-connected-axis',stream.replace(root,packed.replace('20 30 m 20 330 l','20 30 m 20 180 l 20 330 l'))),('compound-interior',stream.replace(root,packed.replace(' S\nQ',' 70 285 m 110 285 l S\nQ'))),('compound-extension',stream.replace(root,packed.replace('20 330 m 380 330 l','0 330 m 380 330 l'))),('compound-joined',stream.replace(root,packed.replace('20 330 m','20 330 l'))),('compound-solid',stream.replace(root,packed.replace('[2 2] 0 d',''))),('compound-dash',stream.replace(root,packed)),('compound-alpha',stream.replace(root,packed.replace('[2 2] 0 d','[2 2] 1.5 d /GS1 gs')))]:
 objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 360] /Resources << /Font << /F1 5 0 R >> /ExtGState << /GS1 6 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(ops.encode())} >>\nstream\n'.encode()+ops.encode()+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',b'<< /Type /ExtGState /CA 0.4 /ca 0.4 >>']
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,obj in enumerate(objects,1):offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 xref=len(data);data.extend(f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode())
 for off in offsets:data.extend(f'{off:010} 00000 n \n'.encode())
 data.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode());(b/(key+'.pdf')).write_bytes(data)
