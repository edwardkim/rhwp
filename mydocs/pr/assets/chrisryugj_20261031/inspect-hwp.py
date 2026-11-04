import olefile,zlib,struct,json,hashlib,sys
from pathlib import Path

def inspect(path):
 out={'path':str(path),'sha256':hashlib.sha256(Path(path).read_bytes()).hexdigest(),'paragraphs':[],'child_ctrl_data':[],'ctrl_count':0}
 with olefile.OleFileIO(path) as f:
  compressed=struct.unpack_from('<I',f.openstream('FileHeader').read(),36)[0]&1
  for p in sorted(p for p in f.listdir() if p[0]=='BodyText' and p[-1].startswith('Section')):
   b=f.openstream(p).read(); b=zlib.decompress(b,-15) if compressed else b; off=0;stack=[];para=-1
   while off+4<=len(b):
    h=struct.unpack_from('<I',b,off)[0];off+=4;t=h&1023;l=(h>>10)&1023;n=h>>20
    if n==4095:n=struct.unpack_from('<I',b,off)[0];off+=4
    d=b[off:off+n];off+=n
    if t==66 and l==0:para+=1
    if t==67 and l==1:
     u=struct.unpack('<'+'H'*(len(d)//2),d);i=0;text='';tabs=[]
     while i<len(u):
      c=u[i]
      if c==9:tabs.append(list(u[i+1:i+8]));text+='\t';i+=8
      elif c in [1,2,3,4,5,6,7,8,11,12,14,15,16,17,18,19,20,21,22,23]:i+=8
      elif c<32:i+=1
      else:text+=chr(c);i+=1
     out['paragraphs'].append({'section':p[-1],'paragraph':para,'text':text,'tabs':tabs})
    stack=stack[:l]
    if t==87:
     out['ctrl_count']+=1
     if len(stack)>=2 and stack[-2:]==[76,76]:out['child_ctrl_data'].append(hashlib.sha256(d).hexdigest())
    stack += [0]*(l-len(stack));stack.append(t)
 return out
if __name__=='__main__':
 r=[inspect(p) for p in sys.argv[1:]]
 print(json.dumps(r,ensure_ascii=False,indent=2))
