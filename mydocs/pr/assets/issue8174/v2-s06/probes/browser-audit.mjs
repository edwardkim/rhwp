import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
const R=process.cwd(), O=path.join(R,'output/pr-review/issue8174/v2-s06-20261025');
const require=createRequire(path.join(R,'rhwp-studio/package.json'));
const puppeteer=require('puppeteer-core');
const files=new Map();
for(const name of ['rhwp.js','rhwp_bg.wasm']) files.set('/pkg/'+name,fs.readFileSync(path.join(R,'output/pr-review/issue8174/v2-s04-20261025/candidate-wasm',name)));
for(const name of ['import-session.js','import-worker.js']) files.set('/rhwp-shared/pdf/'+name,fs.readFileSync(path.join(R,'rhwp-shared/pdf',name)));
const inputs=JSON.parse(fs.readFileSync(path.join(O,'inputs.json'),'utf8'));
for(const item of inputs) files.set('/inputs/'+item.name,fs.readFileSync(path.join(O,'inputs',item.name)));
files.set('/healthy.pdf',fs.readFileSync(path.join(R,'pdf/issue8174/features/icc-rgb.pdf')));
const server=http.createServer((req,res)=>{
  if(req.url==='/') return res.end('<!doctype html><meta charset="utf-8">');
  const bytes=files.get(req.url); if(!bytes) return res.writeHead(404).end();
  res.setHeader('Content-Type',req.url.endsWith('.wasm')?'application/wasm':req.url.endsWith('.js')?'text/javascript':'application/pdf');res.end(bytes);
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
let browser;
try{
  browser=await puppeteer.launch({executablePath:'/home/edward/.cache/puppeteer/chrome-headless-shell/linux-155.0.8059.79/chrome-headless-shell-linux64/chrome-headless-shell',headless:true,args:['--no-sandbox','--disable-dev-shm-usage']});
  const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(String(e)));
  await page.goto('http://127.0.0.1:'+server.address().port+'/');
  await page.evaluate(async()=>{window.m=await import('/pkg/rhwp.js');await m.default()});
  const rows=await page.evaluate(async(names)=>{
    const rows=[];
    for(const name of names){
      const bytes=new Uint8Array(await(await fetch('/inputs/'+name)).arrayBuffer());
      const started=performance.now();const before=m.memory?.buffer?.byteLength;
      try{
        const d=m.HwpDocument.importPdf(bytes,{resourceProfile:'constrained',resourceLimits:{backend:{fontEntries:1}}});
        try { rows.push({name,accepted:true,pages:d.pageCount(),diagnostics:JSON.parse(d.getPdfImportDiagnostics()),elapsedMs:performance.now()-started,before,after:m.memory?.buffer?.byteLength}); }
        finally {d.free()}
      }catch(e){rows.push({name,accepted:false,error:String(e),elapsedMs:performance.now()-started})}
    }
    return rows;
  },inputs.map(i=>i.name));
  assert.equal(rows.find(r=>r.name==='cid-width-control.pdf').accepted,false);
  assert.ok(rows.find(r=>r.name==='cid-width-control.pdf').error.includes('PDF_RESOURCE_LIMIT'));
  assert.ok(rows.filter(r=>r.name!=='cid-width-control.pdf').every(r=>r.accepted&&r.pages===1));
  const worker=await page.evaluate(async()=>{
    const {PdfImportSession}=await import('/rhwp-shared/pdf/import-session.js');let created=0,terminated=0;
    const session=new PdfImportSession({workerFactory:()=>{created++;const worker=new Worker('/rhwp-shared/pdf/import-worker.js',{type:'module'});const stop=worker.terminate.bind(worker);worker.terminate=()=>{terminated++;stop()};return worker}});
    const input=async(name)=>new Uint8Array(await(await fetch(name)).arrayBuffer());
    const healthy=await input('/healthy.pdf'),bad=await input('/inputs/cid-width-control.pdf');
    const initial=await session.importPdf(healthy,{});let limited,timeout;
    try{await session.importPdf(bad,{resourceLimits:{backend:{fontEntries:1}}})}catch(e){limited=String(e)}
    const retained=await session.inspect();
    try{await session.importPdf(healthy,{}, {timeoutMs:1})}catch(e){timeout=String(e)}
    const retainedAfterTimeout=await session.inspect();const createdAfterTimeout=created,terminatedAfterTimeout=terminated;
    const next=await session.importPdf(healthy,{});const saved=await session.exportDocument('hwpx');session.dispose();
    return {created,terminated,createdAfterTimeout,terminatedAfterTimeout,initial:initial.pageCount,limited,retained:retained.pageCount,timeout,retainedAfterTimeout:retainedAfterTimeout.pageCount,next:next.pageCount,saved:saved.byteLength};
  });
  assert.equal(worker.initial,1);assert.equal(worker.retained,1);assert.equal(worker.retainedAfterTimeout,1);assert.equal(worker.next,1);
  assert.ok(worker.limited.includes('PDF_RESOURCE_LIMIT'));assert.ok(worker.timeout.includes('PDF_IMPORT_TIMEOUT'));assert.ok(worker.created>worker.createdAfterTimeout);assert.ok(worker.terminatedAfterTimeout>=1);assert.ok(worker.saved>100);
  const capabilities=await page.evaluate(()=>JSON.parse(m.HwpDocument.getPdfImportRuntimeCapabilities()));
  assert.equal(errors.length,0);
  fs.writeFileSync(path.join(O,'browser-audit.json'),JSON.stringify({rows,worker,capabilities,errors,producer:'retained S04 optimized product bundle; source identity checked against current HEAD; no product source change or new WASM build claimed'},null,2)+'\n');
  console.log(JSON.stringify({rows:rows.map(({name,accepted,pages,error})=>({name,accepted,pages,error})),worker,capabilities,errors},null,2));
}finally{await browser?.close();await new Promise(r=>server.close(r))}
