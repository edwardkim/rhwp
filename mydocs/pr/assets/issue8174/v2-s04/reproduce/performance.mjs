// Run from repository root, with an idle CPU after the required Cargo gates.
import fs from 'node:fs/promises';
import path from 'node:path';
import http from 'node:http';
import os from 'node:os';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const root=process.cwd(),out=path.resolve(process.env.ISSUE8174_EVIDENCE??'output/pr-review/issue8174/v2-s04-performance');
for(const key of ['CHROME_PATH','BASE_WASM_PKG','CANDIDATE_WASM_PKG'])if(!process.env[key])throw Error(`Set ${key}`);
await fs.mkdir(out,{recursive:true});const require=createRequire(path.join(root,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const files=new Map();for(const v of ['base','candidate'])for(const n of ['rhwp.js','rhwp_bg.wasm'])files.set('/'+v+'/'+n,await fs.readFile(path.join(process.env[v==='base'?'BASE_WASM_PKG':'CANDIDATE_WASM_PKG'],n)));
files.set('/input.pdf',await fs.readFile('pdf/issue8183/scan-jpeg-control.pdf'));files.set('/table.hwp',await fs.readFile('samples/hwp_table_test.hwp'));
const server=http.createServer((req,res)=>{if(req.url==='/')return res.end('<!doctype html>');const b=files.get(req.url);if(!b)return res.writeHead(404).end();res.setHeader('Content-Type',req.url.endsWith('.js')?'text/javascript':req.url.endsWith('.wasm')?'application/wasm':'application/octet-stream');res.end(b)});await new Promise(r=>server.listen(0,'127.0.0.1',r));
const wasm=[],handoff=[],cryptoRows=[];let browser;
try{
 browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true});
 for(let round=0;round<4;round++)for(const v of (round%2?['candidate','base']:['base','candidate'])){
  const p=await browser.newPage();await p.goto('http://127.0.0.1:'+server.address().port);
  const r=await p.evaluate(async({v,round})=>{const s=performance.now();const m=await import('/'+v+'/rhwp.js');const wasm=await m.default();const initMs=performance.now()-s;
   const inputs=[['jpeg-import',new Uint8Array(await(await fetch('/input.pdf')).arrayBuffer())],['table-open',new Uint8Array(await(await fetch('/table.hwp')).arrayBuffer())]],samples=[];
   for(let run=0;run<12;run++)for(const [name,b]of inputs){const t=performance.now(),d=name==='jpeg-import'?m.HwpDocument.importPdf(b,{}):new m.HwpDocument(b);const pages=d.pageCount();d.free();samples.push({name,run,ms:performance.now()-t,pages,memoryBytes:wasm.memory.buffer.byteLength})}
   return {v,round,initMs,samples};},{v,round});assert.ok(r.samples.every(x=>x.pages>0));wasm.push(r);await p.close();
 }
 await fs.writeFile(path.join(out,'wasm.json'),JSON.stringify(wasm,null,2)+'\n');console.log('PASS balanced S03/S04 pure WASM');
 const input=Array.from(files.get('/input.pdf'));
 for(let round=0;round<4;round++){
  const p=await browser.newPage();await p.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit',{waitUntil:'domcontentloaded'});await p.waitForFunction(()=>window.__wasm?.pageCount>0&&window.__renderBackend,{timeout:90000});
  const rows=await p.evaluate(async({input,round})=>{const {preparePdfForEditor}=await import('/src/core/pdf-open.ts');const w=window.__wasm,b=new Uint8Array(input),rows=[];
   for(let run=0;run<6;run++)for(const mode of (run%2?['worker','direct']:['direct','worker'])){
    const t=performance.now();let converted=t,editableBytes=null;
    if(mode==='direct')w.loadDocument(b,'benchmark.pdf');else{const prepared=await preparePdfForEditor({byteLength:b.length,read:async()=>b},{resourceProfile:'standard'},w.getPdfImportPolicy({}),{signal:new AbortController().signal,timeoutMs:30000});converted=performance.now();editableBytes=prepared.editable.length;w.loadImportedPdf(prepared,'benchmark.pdf');}
    rows.push({round,run,mode,totalMs:performance.now()-t,workerAndSerializeMs:mode==='worker'?converted-t:null,reopenMs:mode==='worker'?performance.now()-converted:null,pages:w.pageCount,editableBytes});
   }return rows;},{input,round});assert.ok(rows.every(x=>x.pages===1));handoff.push(...rows);await p.close();
 }
 await fs.writeFile(path.join(out,'handoff.json'),JSON.stringify(handoff,null,2)+'\n');console.log('PASS actual Studio direct/Worker handoff');
 const p=await browser.newPage();await p.goto(process.env.VITE_URL??'http://127.0.0.1:7704');await p.waitForFunction(()=>window.__wasm?.pageCount>0&&window.__renderBackend,{timeout:90000});
 const c=await p.evaluate(async()=>{const {RecoveryCipher}=await import('/src/recovery/recovery-crypto.ts');const rows=[];
  for(let round=0;round<4;round++){
   let t=performance.now();const cipher=await RecoveryCipher.create('public-benchmark-only');const createMs=performance.now()-t;
   for(const bytes of [65536,1048576]){const data=new Uint8Array(bytes).fill(37),draft={id:'benchmark',fileName:'public.hwp',sourceFormat:'hwp',savedAt:Date.now(),dirtyReason:'benchmark',data,byteLength:data.length};const protectMs=[];let encrypted;
    for(let run=0;run<10;run++){t=performance.now();encrypted=await cipher.protect(draft);protectMs.push(performance.now()-t);}
    t=performance.now();const unlocked=await RecoveryCipher.unlock(encrypted,'public-benchmark-only');const unlockMs=performance.now()-t;const equal=unlocked.data.length===bytes&&unlocked.data.every(x=>x===37);unlocked.data.fill(0);unlocked.cipher.dispose();data.fill(0);rows.push({round,bytes,createMs,protectMs,unlockIncludingKdfMs:unlockMs,cipherBytes:encrypted.data.length,equal});
   }cipher.dispose();
  }return rows;});assert.ok(c.every(x=>x.equal));cryptoRows.push(...c);await p.close();
 await fs.writeFile(path.join(out,'crypto.json'),JSON.stringify(cryptoRows,null,2)+'\n');
 const trees=[];for(const name of ['gradient','clip-opacity','internal-use','restricted-script','embedded-png','embedded-jpeg','foreground'])trees.push({name,tree:JSON.parse(await fs.readFile(new URL('../fixtures/svg/'+name+'.json',import.meta.url),'utf8'))});
 const svgPage=await browser.newPage();await svgPage.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit');await svgPage.waitForFunction(()=>window.__renderBackend,{timeout:90000});
 const svg=await svgPage.evaluate(async trees=>{const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');const rows=[];
  for(const {name,tree}of trees){const r=await CanvasKitLayerRenderer.create('strict','software'),c=document.createElement('canvas');c.width=180;c.height=80;let t=performance.now();r.renderPage(tree,c,1,undefined,undefined,1);const firstMs=performance.now()-t,warmMs=[];for(let run=0;run<20;run++){t=performance.now();r.renderPage(tree,c,1,undefined,undefined,1);warmMs.push(performance.now()-t);}const d=r.diagnostics();rows.push({name,firstMs,warmMs,imageCacheEntries:d.imageCacheEntries,imageCachePixels:d.imageCachePixels,imageCacheHits:d.imageCacheHits,ready:d.passesRuntimeReadinessGate});r.dispose();}return rows;},trees);
 assert.ok(svg.every(r=>r.ready&&r.imageCacheEntries===1&&r.imageCachePixels===900));await fs.writeFile(path.join(out,'svg-cache.json'),JSON.stringify(svg,null,2)+'\n');await svgPage.close();console.log('PASS actual SVG cold/warm image cache');
 await fs.writeFile(path.join(out,'environment.json'),JSON.stringify({platform:os.platform(),release:os.release(),cpu:os.cpus()[0].model,logicalCPUs:os.cpus().length,node:process.version,browser:await browser.version(),method:'4 fresh tabs/package, alternate S03/S04 order, 12 iterations/input; same producer direct/Worker uses 4 tabs x 6 rounds with alternate order; crypto 4 rounds x 10 encryptions/size; exclude upload/UI waits/render; memory is WASM linear high-water, not RSS; OS/JIT/cache not globally cold',sizes:{baseWasmBytes:files.get('/base/rhwp_bg.wasm').length,candidateWasmBytes:files.get('/candidate/rhwp_bg.wasm').length}},null,2)+'\n');console.log('PASS actual browser KDF/encryption and memory/size evidence');
}finally{await browser?.close();await new Promise(r=>server.close(r));}
