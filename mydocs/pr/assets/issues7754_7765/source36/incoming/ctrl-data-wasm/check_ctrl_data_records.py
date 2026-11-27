"""Independent CFB/DEFLATE reader of physical HWPTAG_CTRL_DATA owner chains and bytes."""
from pathlib import Path
import olefile,zlib,struct,hashlib,json,sys
R=Path('/home/tsjang/rhwp');O=R/'output/pr-review/issue7754-nine'
def packets(path):
 out=[]
 with olefile.OleFileIO(str(path)) as f:
  header=f.openstream('FileHeader').read();compressed=bool(struct.unpack_from('<I',header,36)[0]&1)
  for parts in sorted(p for p in f.listdir(streams=True,storages=False) if len(p)==2 and p[0]=='BodyText' and p[1].startswith('Section')):
   raw=f.openstream(parts).read();raw=zlib.decompress(raw,-15) if compressed else raw;offset=0;owners=[];ids={}
   while offset<len(raw):
    word=struct.unpack_from('<I',raw,offset)[0];offset+=4;tag=word&1023;level=(word>>10)&1023;size=word>>20
    if size==4095:size=struct.unpack_from('<I',raw,offset)[0];offset+=4
    data=raw[offset:offset+size];assert len(data)==size;offset+=size;owners=owners[:level];identity=None
    if tag in [71,76]:
     assert len(data)>=4;object_id=data[:4].hex();key=(tag,object_id);ids[key]=ids.get(key,0)+1;identity={'id_hex':object_id,'ordinal':ids[key]}
    if tag==87:out.append({'section':parts[1],'owners':owners.copy(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    owners.append({'tag':tag,'object':identity})
 return out
if __name__=='__main__':
 role=sys.argv[1];base=O/'source36-ctrl-data-wasm'/role;s=json.loads((base/'state.json').read_text());assert s.get('passed');results=[]
 for row,count in zip(s['results'],[60,2,2,2]):
  expected=packets(R/row['input']);assert len(expected)==count,(row['input'],len(expected),count)
  for round in row['rounds']:
   actual=packets(round['file']);equal=actual==expected;results.append({'input':row['input'],'round':round['round'],'expected_packets':len(expected),'actual_packets':len(actual),'owner_and_payload_sha_equal':equal,'expected':expected,'actual':actual});assert equal,(role,row['input'],round['round'])
 result={'role':role,'method':'independent olefile/DEFLATE record reader: tag10/level10/size12, owner stack, physical payload SHA256','source_head':s['source_head'],'wasm_sha256':s['wasm_sha256'],'results':results,'passed':True};(base/'physical-record-validation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(role,'8 source/output physical owner+payload comparisons PASS')
