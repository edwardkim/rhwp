from pathlib import Path
import hashlib,json,shutil,struct,zlib
import olefile

root=Path.cwd()
source=root/'samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp'
out=root/'output/pr-review/planet6897-7382-20260926/controls/stage33'
out.mkdir(parents=True,exist_ok=True)
stream='BodyText/Section4'
with olefile.OleFileIO(source) as original:
    streams={tuple(p):original.openstream(p).read() for p in original.listdir(streams=True,storages=False)}
compressed=streams[tuple(stream.split('/'))]
body=zlib.decompress(compressed,-15)
records=[]
i=0
while i<len(body):
    header=struct.unpack_from('<I',body,i)[0]
    tag=header&1023
    size=header>>20
    i+=4
    if size==4095:
        size=struct.unpack_from('<I',body,i)[0]
        i+=4
    data=body[i:i+size]
    if tag==71 and len(data)>=28:
        attr,offset,x,width,height=struct.unpack_from('<IIIII',data,4)
        if (width,height) in [(3634,3806),(3608,3866)]:
            records.append({'record_data_offset':i,'vertical_offset':offset,'width':width,'height':height,'attr':attr})
    i+=size
assert len(records)==2
assert [r['vertical_offset'] for r in records]==[780,862]
results=[]
for name,offsets in [('zero',[0,0]),('forward-750',[1530,1612])]:
    changed=bytearray(body)
    for record,offset in zip(records,offsets):
        struct.pack_into('<I',changed,record['record_data_offset']+8,offset)
    # 동일 길이의 CFB 스트림을 보존한다. 디플레이트 끝 뒤의 0은 레코드 데이터가 아니다.
    encoded=zlib.compress(bytes(changed),level=9,wbits=-15)
    assert len(encoded)<=len(compressed),(name,len(encoded),len(compressed))
    encoded=encoded.ljust(len(compressed),b'\0')
    assert zlib.decompress(encoded,-15)==bytes(changed)
    dest=out/f'japan-cell-{name}.hwp'
    shutil.copy2(source,dest)
    with olefile.OleFileIO(dest,write_mode=True) as modified:
        modified.write_stream(stream,encoded)
    with olefile.OleFileIO(dest) as modified:
        for path,data in streams.items():
            actual=modified.openstream(list(path)).read()
            if '/'.join(path)==stream:
                assert zlib.decompress(actual,-15)==bytes(changed)
            else:
                assert actual==data,path
    # 압축 해제 레코드에서는 지정한 두 UINT32 오프셋 이외의 바이트가 변하지 않는다.
    restored=bytearray(changed)
    for record in records:
        struct.pack_into('<I',restored,record['record_data_offset']+8,record['vertical_offset'])
    assert bytes(restored)==body
    results.append({'name':name,'input':str(dest.relative_to(root)),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'vertical_offsets_hu':offsets,'all_other_cfb_streams_identical':True,'all_other_record_bytes_identical':True})
provenance={'original':str(source.relative_to(root)),'original_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'stream':stream,'records':records,'variants':results,'input_generation':'원본 HWP 공통 개체 레코드의 세로 오프셋 두 UINT32만 수정; 재저장/LineSeg 재생성 아님'}
(out/'provenance.json').write_text(json.dumps(provenance,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'variants':[{k:r[k] for k in ['name','vertical_offsets_hu','all_other_cfb_streams_identical','all_other_record_bytes_identical']} for r in results]},ensure_ascii=False))
