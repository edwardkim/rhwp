// Stage 1 read-only-source API probe. Generated documents are not Hancom oracles.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
const { default: init, HwpDocument } = await import(pathToFileURL(path.resolve('pkg/rhwp.js')));

const sha = b => crypto.createHash('sha256').update(b).digest('hex');
const wasm = fs.readFileSync('pkg/rhwp_bg.wasm');
if (sha(wasm) !== '5cd0f9fe37155593aa5e89afebbc903923935eceefbf4c50827452f1172ceba8') {
  throw new Error('Expected baseline WASM hash changed; re-check provenance');
}
await init({ module_or_path: wasm });
// Python stdlib is used only to inspect ZIP/XML bytes received on stdin.
const inspect = bytes => JSON.parse(execFileSync('python3', ['-c', String.raw`
import sys, io, zipfile, json, xml.etree.ElementTree as ET
z=zipfile.ZipFile(io.BytesIO(sys.stdin.buffer.read()))
local=lambda e:e.tag.rsplit('}',1)[-1]
sections=[]; objects=[]; para_ids=[]
for name in sorted(z.namelist()):
 if not name.startswith('Contents/section') or not name.endswith('.xml'): continue
 root=ET.fromstring(z.read(name)); ps=[]
 for e in root.iter():
  tag=local(e)
  if tag=='p': para_ids.append(e.get('id'))
  if tag in ('tbl','rect','pic','container','ellipse','line','fieldBegin'):
   objects.append({'tag':tag,'id':e.get('id'),'instid':e.get('instid')})
 for p in root:
  if local(p)!='p': continue
  runs=[r for r in p if local(r)=='run']
  texts=[''.join(t.itertext()) for r in runs for t in r if local(t)=='t']
  ps.append({'attrs':{k:v for k,v in p.attrib.items() if k!='id'},
   'text':''.join(texts),
   'controls':[local(c) for r in runs for c in r if local(c) not in ('t','linesegarray')],
   'charRefs':[r.get('charPrIDRef') for r in runs]})
 sections.append(ps)
from collections import Counter
dups=[{'tag':t,'id':i,'count':n} for (t,i),n in Counter((o['tag'],o['id']) for o in objects).items() if n>1]
print(json.dumps({'sections':sections,'objects':objects,'duplicateObjectIds':dups,
 'paraCount':len(para_ids),'uniqueParaIds':len(set(para_ids))},ensure_ascii=False))
`], { input: bytes, maxBuffer: 4 * 1024 * 1024 }));
const state = d => inspect(d.exportHwpx());
const checkRoundtrip = d => {
  const out = {};
  for (const [format, bytes] of [['hwp', d.exportHwp()], ['hwpx', d.exportHwpx()]]) {
    try { const reopened = new HwpDocument(bytes); out[format] = state(reopened); reopened.free(); }
    catch (e) { out[format] = {error:String(e)}; }
  }
  return out;
};
const results = {sourceBase:'59a11f180ad1bd5cadcbbf0a6dc9d0162f4a0a21', wasmSha256:sha(wasm), cases:[]};
const d = HwpDocument.createEmpty();
d.createBlankDocument();
d.insertText(0,0,0,'BEFORE'); d.splitParagraph(0,0,6); d.insertText(0,1,0,'AFTER');
const table = JSON.parse(d.createTable(0,1,0,1,1));
const before = state(d);
d.copyControl(0,table.paraIdx,'',table.controlIdx);
const copies=[];
for(let n=0;n<2;n++) {
  const afterIdx=state(d).sections[0].findIndex(p=>p.text==='AFTER');
  copies.push(JSON.parse(d.pasteControl(0,afterIdx,0)));
}
const after=state(d);
const find=(s,t)=>s.sections[0].find(p=>p.text===t);
results.cases.push({name:'synthetic-boundary-contract-only',before,after,copies,
 boundaryPreserved:['BEFORE','AFTER'].every(t=>JSON.stringify(find(before,t))===JSON.stringify(find(after,t))),
 roundtrip:checkRoundtrip(d)});
const snap=d.saveSnapshot(); const text=d.getTextFileText();
d.beginBatch(); d.insertText(0,0,0,'CHANGED'); let error;
try {d.insertText(999,0,0,'invalid');} catch(e) {error=String(e);}
const retainedAfterError=d.getTextFileText()!==text;
d.endBatch(); d.restoreSnapshot(snap);
results.cases.push({name:'batch-versus-snapshot',error,earlierEditRetainedAfterError:retainedAfterError,
 explicitSnapshotRestoredText:d.getTextFileText()===text});d.free();
for(const [sample,pi,ci] of [['samples/hwp_table_test.hwp',3,0],['samples/table-in-tbox.hwp',0,2]]) {
 for(const inputMode of ['original-hwp','rhwp-exported-hwpx-not-Hancom-oracle']) {
 const bytes=fs.readFileSync(sample);let doc=new HwpDocument(bytes);
 if(inputMode!=='original-hwp') {const converted=doc.exportHwpx();doc.free();doc=new HwpDocument(converted);}
 const b=state(doc);doc.copyControl(0,pi,'',ci);
 const paste=JSON.parse(doc.pasteControl(0,doc.getParagraphCount(0)-1,0));
 const after=state(doc);let independence;
 if(ci===0) {
   const original=doc.getTextInCell(0,pi,ci,0,0,0,10000);
   const originalProps=doc.getCellCharPropertiesAt(0,pi,ci,0,0,0);
   doc.insertTextInCell(0,paste.paraIdx,0,0,0,0,'COPY_ONLY');
   doc.applyCharFormatInCell(0,paste.paraIdx,0,0,0,0,9,JSON.stringify({bold:true}));
   independence={originalTextPreserved:original===doc.getTextInCell(0,pi,ci,0,0,0,10000),
     originalPropertiesPreserved:originalProps===doc.getCellCharPropertiesAt(0,pi,ci,0,0,0),
     copiedTextHasMarker:doc.getTextInCell(0,paste.paraIdx,0,0,0,0,9).includes('COPY_ONLY')};
 }
 results.cases.push({name:sample,inputMode,inputSha256:sha(bytes),paste,before:b,after,independence,roundtrip:checkRoundtrip(doc)});
 doc.free();
 }
}
const outputDir=path.resolve('output/3587/stage1');
fs.mkdirSync(outputDir,{recursive:true});
fs.writeFileSync(path.join(outputDir,'probe-result.json'),JSON.stringify(results,null,2)+'\n');
for(const c of results.cases) console.log(JSON.stringify({name:c.name,inputMode:c.inputMode,independence:c.independence,boundaryPreserved:c.boundaryPreserved,
 beforeParas:c.before?.sections[0].length,afterParas:c.after?.sections[0].length,
 duplicateObjectIds:c.after?.duplicateObjectIds,earlierEditRetainedAfterError:c.earlierEditRetainedAfterError,
 explicitSnapshotRestoredText:c.explicitSnapshotRestoredText,
 roundtrip:c.roundtrip && Object.fromEntries(Object.entries(c.roundtrip).map(([f,v])=>[f,
 {error:v.error,duplicates:v.duplicateObjectIds,paraCount:v.paraCount,uniqueParaIds:v.uniqueParaIds}]))}));
