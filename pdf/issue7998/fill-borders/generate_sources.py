from pathlib import Path
import re, json, hashlib, argparse
parser=argparse.ArgumentParser(description='Independent filled-rule and occluder PDF controls');parser.add_argument('--output',type=Path,required=True);out=parser.parse_args().output;out.mkdir(parents=True,exist_ok=True)
base=(Path(__file__).resolve().parents[1]/'nested-dash-parent.pdf').read_bytes()
stream=re.search(rb'4 0 obj\n.*?stream\n(.*?)\nendstream',base,re.S).group(1).decode()
pattern=r'([0-9.]+) ([0-9.]+) m ([0-9.]+) ([0-9.]+) l S'
def band(m):
 x,y,a,c=map(float,m.groups())
 if y==c:return f'q 0 g {min(x,a)} {y-.25} {abs(a-x)} 0.5 re f Q'
 return f'q 0 g {x-.25} {min(y,c)} 0.5 {abs(c-y)} re f Q'
filled=re.sub(pattern,band,stream,count=6).replace('[2 2] 0 d','')
cases={'fill-parent':filled,
       'fill-background':filled.replace('q 0 G', 'q 0.9 0.95 1 rg 25 185 170 140 re f Q\nq 0 G',1),
       'fill-occluder':stream+'\nq 1 g 19.75 30 0.5 300 re f Q',
       'fill-dark-occluder':stream+'\nq 0 g 19.75 30 0.5 300 re f Q',
       'fill-highlight':filled.replace('q 0 G','q 0.9 0.95 1 rg 25 300 100 25 re f Q\nq 0 G',1),
       'fill-foreground':filled+'\nq 0.9 0.95 1 rg 25 185 170 140 re f Q',
       'fill-competing':filled.replace('q 0 G','q 0.9 0.95 1 rg 25 185 170 140 re f Q\nq 1 0.95 0.9 rg 26 186 168 138 re f Q\nq 0 G',1)}
for key,ops in cases.items():
 objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 360] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(ops.encode())} >>\nstream\n'.encode()+ops.encode()+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>']
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,obj in enumerate(objects,1):offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 xref=len(data);data.extend(f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode())
 for offset in offsets:data.extend(f'{offset:010} 00000 n \n'.encode())
 data.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode())
 (out/(key+'.pdf')).write_bytes(data)
 (out/(key+'-source.json')).write_text(json.dumps({'source':'independent PDF operators, six root rules authored as separate filled bands; occluder retains original dashed rules and overlays full-height white fill','sha256':hashlib.sha256(data).hexdigest(),'rootBandPt':0.5,'tables':2,'occluder':'occluder' in key},indent=2)+'\n')
