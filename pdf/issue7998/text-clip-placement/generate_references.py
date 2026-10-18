from pathlib import Path
import pikepdf,json,hashlib
b=Path(__file__).resolve().parent;o=b/'reference';o.mkdir(exist_ok=True);records=[]
for v in json.loads((b/'contexts-origin.json').read_text()):
 src=b/'sources'/('first-'+v['source']+'.pdf');dest=o/(v['key']+'.pdf')
 with pikepdf.open(src)as pdf:
  data=pdf.pages[0].Contents.read_bytes()
  if not v['clip']:
   ops=pikepdf.parse_content_stream(pdf.pages[0]);ops=[op for op in ops if str(op.operator)not in ['W','W*']];data=pikepdf.unparse_content_stream(ops)
  shift=v['expectedShiftPx']*72/96;data=f'q 1 0 0 1 0 {-shift:.12f} cm\n'.encode()+data+b'\nQ\n';pdf.pages[0].Contents=pikepdf.Stream(pdf,data);pdf.docinfo['/Creator']='rhwp 7998 independent PDF translation fixture from Hancom source';pdf.save(dest)
 records.append({'key':v['key'],'shiftPt':shift,'sourceBytes':src.stat().st_size,'sourceSha256':hashlib.sha256(src.read_bytes()).hexdigest(),'referenceBytes':dest.stat().st_size,'referenceSha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'derivation':'source PDF graphics cm from authored page margins, source shape extent and requested alignment; clipping unchanged or explicitly removed for normal control'})
(b/'reference-origin.json').write_text(json.dumps(records,indent=2)+'\n');print('independent translated references',len(records))
