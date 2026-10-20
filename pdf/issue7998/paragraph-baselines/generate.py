"""Change only source Tf size; retain Tm/CTM/paths and rebuild PDF syntax."""
from pathlib import Path
import hashlib,re,sys
root=Path(__file__).resolve().parents[3]
source=root/'pdf/issue7998/structure-borders/native-pen-blue.pdf'
objects=[v for _,v in re.findall(rb'(\d+) 0 obj\n(.*?)\nendobj',source.read_bytes(),re.S)]
stream=re.search(rb'stream\n(.*?)\nendstream',objects[3],re.S)[1].replace(b'/F1 10 Tf',b'/F1 10.5 Tf')
objects[3]=b'<< /Length '+str(len(stream)).encode()+b' >>\nstream\n'+stream+b'\nendstream'
data=bytearray(b'%PDF-1.4\n');offsets=[0]
for i,obj in enumerate(objects,1):
 offsets.append(len(data));data+=str(i).encode()+b' 0 obj\n'+obj+b'\nendobj\n'
xref=len(data);data+=f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()
for pos in offsets[1:]:data+=f'{pos:010} 00000 n \n'.encode()
data+=f'trailer << /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode()
dest=Path(sys.argv[1])if len(sys.argv)>1 else Path(__file__).with_name('half-point-regenerated.pdf');dest.write_bytes(data)
expected=Path(__file__).with_name('half-point.pdf').read_bytes()
assert data==expected,'Generated source differs; do not silently replace qualified PDF'
print(hashlib.sha256(data).hexdigest())
