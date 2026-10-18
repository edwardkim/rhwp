from pathlib import Path
import hashlib,json,argparse
parser=argparse.ArgumentParser(description='Independent continuous/periodic table-border sources');parser.add_argument('--output',type=Path,required=True);out=parser.parse_args().output;out.mkdir(parents=True,exist_ok=True)

def text(s,x,y,size=9):return f'BT /F1 {size} Tf 1 0 0 1 {x} {y} Tm ({s}) Tj ET'
def grid(x,y,w,h,periodic=False,mixed=False):
 result=[]
 for i,(a,b,c,d) in enumerate([(x,y+h,x+w,y+h),(x,y,x+w,y),(x,y+h,x,y),(x+w,y+h,x+w,y),(x+w/2,y+h,x+w/2,y),(x,y+h/2,x+w,y+h/2)]):
  result+=['q 0 G '+str((0.8 if key=='wrong-width-dot' else 0.84) if periodic else 0.5)+' w']
  if periodic and i>=4:
   length=abs(c-a)+abs(d-b);at=0.12 if key=='phase-shifted-dot' else 0.0
   while at<length:
    end=min(at+1.2,length)
    if key=='underreach-dot' and end+3.0>=length:end-=0.05
    if end>at:
     pos=lambda t:(a+t,b) if c!=a else(a,b-t)
     u,v=pos(at),pos(end)
     result+=[f'{u[0]:.4f} {u[1]:.4f} m {v[0]:.4f} {v[1]:.4f} l S']
    at+=3.0
  else:result+=[f'{a} {b} m {c} {d} l S']
  result+=['Q']
 return result
for key in ['compatible-inner-dot','occluded-inner-dot','phase-shifted-dot','underreach-dot','wrong-width-dot']:
 periodic=True
 ops=[text('OUTSIDE ABOVE',20,345),*grid(20,130,240,180,periodic,False),text('BEFORE CHILD',30,292),*grid(35,250,90,28,False),text('A',40,268,8),text('B',85,268,8),text('C',40,254,8),text('D',85,254,8),text('AFTER CHILD',30,232),text('UPPER RIGHT',150,292),text('LOWER LEFT',30,202),text('LOWER RIGHT',150,202),text('OUTSIDE AFTER',20,20)]
 if key=='occluded-inner-dot':ops+=['q 1 1 1 rg 18 260 4 18 re f Q']
 stream='\n'.join(ops).encode();objs=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 360] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(stream)} >>\nstream\n'.encode()+stream+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>']
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,v in enumerate(objs,1):offsets.append(len(data));data+=f'{i} 0 obj\n'.encode()+v+b'\nendobj\n'
 at=len(data);data+=f'xref\n0 {len(objs)+1}\n0000000000 65535 f \n'.encode()
 for offset in offsets:data+=f'{offset:010} 00000 n \n'.encode()
 data+=f'trailer\n<< /Size {len(objs)+1} /Root 1 0 R >>\nstartxref\n{at}\n%%EOF\n'.encode();p=out/(key+'.pdf');p.write_bytes(data);(out/(key+'-source.json')).write_text(json.dumps({'source':'independent PDF operators, no engine-derived positions','sha256':hashlib.sha256(data).hexdigest(),'tables':2,'nativeRootDotSides':8 if key=='compatible-inner-dot' else 0,'childSolid':True,'newRowBefore':0,'newColBefore':0,'cellContents':['BEFORE CHILD','A','B','C','D','AFTER CHILD','UPPER RIGHT','LOWER LEFT','LOWER RIGHT']},indent=2)+'\n');print(p)
