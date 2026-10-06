# 원본 ZIP과 section XML 문자열을 보존하고 두 그림의 pos.vertOffset만 바꾸는 재현 절차.
from pathlib import Path
import zipfile,re,hashlib,json
root=Path.cwd();base=root/'output/pr-review/planet6897-7382-20260926/stage31-hancom-controls';base.mkdir(exist_ok=True)
src=root/'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx'
with zipfile.ZipFile(src) as z:entries=[(i,z.read(i.filename)) for i in z.infolist()]
raw=dict((i.filename,b) for i,b in entries)['Contents/section0.xml'].decode()
records=[]
for name,offset in [('offset-zero',0),('offset-positive',1000)]:
 text=raw;changes=[]
 for ident in ['1937553659','1937553732']:
  pat=r'(<hp:pic id="'+ident+r'".*?</hp:pic>)';m=re.search(pat,text,re.S);assert m
  old=m[1];new,count=re.subn(r'(<hp:pos\b[^>]*\bvertOffset=")[^"]+("[^>]*/>)',rf'\g<1>{offset}\g<2>',old);assert count==1
  changes.append({'picture_id':ident,'old_offset':re.search(r'<hp:pos\b[^>]*vertOffset="([^"]+)"',old)[1],'new_offset':offset})
  text=text[:m.start()]+new+text[m.end():]
 p=base/f'liver7379-cell-picture-{name}.hwpx'
 with zipfile.ZipFile(p,'w') as z:
  for i,b in entries:z.writestr(i,text.encode() if i.filename=='Contents/section0.xml' else b)
 with zipfile.ZipFile(p) as z:assert not z.testzip();assert all(z.read(i.filename)==b for i,b in entries if i.filename!='Contents/section0.xml')
 records.append({'path':str(p.relative_to(root)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'changes':changes})
(base/'input-provenance.json').write_text(json.dumps({'source':str(src.relative_to(root)),'source_sha256':hashlib.sha256(src.read_bytes()).hexdigest(),'variants':records,'unchanged':'section0.xml의 두 그림 pos.vertOffset 외 모든 ZIP 내용과 section XML 문자열 동일'},ensure_ascii=False,indent=2)+'\n')
print('단일 속성 대조군2개 준비')
