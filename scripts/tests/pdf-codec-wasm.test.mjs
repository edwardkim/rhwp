import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync,writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import {createRequire} from 'node:module';
import { resolve,join } from 'node:path';
import http from 'node:http';
import {findChrome} from '../rasterize-svg-webfonts.mjs';
test('PDF codec product WASM and real Worker preserve pixels, budgets and recovery', {timeout: 180_000}, async () => {
const root=resolve(import.meta.dirname,'../..');
const out=resolve(process.env.PDF_CODEC_EVIDENCE||join(root,'output/pr-review/issue8183/wasm-contract'));mkdirSync(out,{recursive:true});
const require=createRequire(join(root,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const config=process.env.PDF_CODEC_EXPECT||'default',pkg=resolve(process.env.PDF_IMPORT_PKG||join(root,'pkg'));
assert(['default','none','jbig2','jpx','both'].includes(config),'PDF_CODEC_EXPECT must name a build configuration');
const expected={enabled:true,jbig2:['default','jbig2','both'].includes(config),jpeg2000:['default','jpx','both'].includes(config),icc:config==='default',boundedXrefRecovery:true};
const inputs=Object.fromEntries(Object.entries(JSON.parse(readFileSync(join(root,'pdf/issue8183/provenance.json'))).files).filter(([name])=>!name.startsWith('reference/')&&name!=='work-limit-jpx.pdf'));
const files=new Map();
for(const n of ['rhwp.js','rhwp_bg.wasm'])files.set('/pkg/'+n,readFileSync(join(pkg,n)));
for(const n of ['import-worker.js','import-session.js'])files.set('/rhwp-shared/pdf/'+n,readFileSync(join(root,'rhwp-shared/pdf',n)));
for(const n of Object.keys(inputs))files.set('/inputs/'+n,readFileSync(join(root,'pdf/issue8183',n)));
files.set('/negative/work-jpx.pdf',readFileSync(join(root,'pdf/issue8183/work-limit-jpx.pdf')));
const server=http.createServer((req,res)=>{if(req.url==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><meta charset="utf-8">');return}const b=files.get(req.url);if(!b){res.writeHead(404).end();return}res.setHeader('Content-Type',req.url.endsWith('.wasm')?'application/wasm':req.url.endsWith('.js')?'text/javascript':'application/pdf');res.end(b)});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
let browser;
try{
 browser=await puppeteer.launch({executablePath:findChrome(),headless:true});const page=await browser.newPage();
 await page.goto('http://127.0.0.1:'+server.address().port);
 const result=await page.evaluate(async({expected,names})=>{
  const module=await import('/pkg/rhwp.js'),wasm=await module.default();const {HwpDocument}=module;
  const support=JSON.parse(HwpDocument.getPdfImportSupport());
  const checked=[];function check(value,message){if(!value)throw Error(message);checked.push(message)}
  check(Object.keys(support).length===Object.keys(expected).length&&Object.keys(expected).every(k=>support[k]===expected[k]),'support matches requested build');
  const buffers={};for(const name of names)buffers[name]=new Uint8Array(await(await fetch('/inputs/'+name)).arrayBuffer());
  const exhaustedBytes=expected.jpeg2000?new Uint8Array(await(await fetch('/negative/work-jpx.pdf')).arrayBuffer()):null;
  const cases=[];
  for(const name of names){
   const jbig=name.startsWith('jbig2')||name.startsWith('scan-jbig2'),jpx=name.startsWith('jpx')||name.startsWith('scan-jpx')||name==='jbig2-mask.pdf';
   const positive=!name.startsWith('corrupt')&&(!jbig||expected.jbig2)&&(!jpx||expected.jpeg2000);
   let doc;let error;
   const before=performance.now();try{doc=HwpDocument.importPdf(buffers[name],{})}catch(e){error=String(e)}
   check(!!doc===positive,`${name}: actual acceptance ${error||'success'}`);
   const item={name,success:!!doc,error,milliseconds:performance.now()-before,wasmMemoryBytes:wasm.memory.buffer.byteLength};
   if(doc){
    try{const diag=JSON.parse(doc.getPdfImportDiagnostics());check(JSON.stringify(diag.pdfImportSupport)===JSON.stringify(support),`${name}: metadata support`);check(diag.parserAllocationBoundsProven===false,`${name}: bounds flag`);check(doc.pageCount()===1,`${name}: source page ownership`);
     item.resourceUsage=diag.resourceUsage;
     for(const fmt of ['hwp','hwpx']){const bytes=fmt==='hwp'?doc.exportHwp():doc.exportHwpx();const reopen=new HwpDocument(bytes);try{check(reopen.pageCount()===1,`${name}: ${fmt} reopened`);check(reopen.renderPageSvg(0).includes('<image'),`${name}: ${fmt} retains image`)}finally{reopen.free()}}
    }finally{doc.free()}
   }cases.push(item)
  }
  if(exhaustedBytes){let error;try{const d=HwpDocument.importPdf(exhaustedBytes,{});d.free()}catch(e){error=String(e)}check(error?.includes('PDF_RESOURCE_LIMIT'),'real codec cumulative work rejects in WASM');const next=HwpDocument.importPdf(buffers['jpx.pdf'],{});try{check(next.pageCount()===1,'normal request succeeds after codec exhaustion')}finally{next.free()}}
  const {PdfImportSession}=await import('/rhwp-shared/pdf/import-session.js');
  let entries=0,cancelOnEntry=null,cancelBytes=0;const entryCounts={};
  const tracePorts=[];
  const wrapperSource=`import {PdfImportRequest} from '${location.origin}/pkg/rhwp.js';
let port;const buffered=[];self.onmessage=e=>buffered.push(e);
const original=PdfImportRequest.importPdf;PdfImportRequest.importPdf=function(...args){port?.postMessage({event:'import-api-entered',inputBytes:args[0].byteLength});return original.apply(this,args)};
await import('${location.origin}/rhwp-shared/pdf/import-worker.js');
const receive=self.onmessage;self.onmessage=e=>{if(e.data.observePort){port=e.data.observePort;return}receive(e)};
for(const e of buffered)self.onmessage(e);`;
  const wrapperUrl=URL.createObjectURL(new Blob([wrapperSource],{type:'text/javascript'}));
  const session=new PdfImportSession({workerFactory:()=>{const channel=new MessageChannel();tracePorts.push(channel.port1);channel.port1.onmessage=({data})=>{entries++;entryCounts[data.inputBytes]=(entryCounts[data.inputBytes]||0)+1;if(data.inputBytes===cancelBytes)cancelOnEntry?.abort()};const worker=new Worker(wrapperUrl,{type:'module'});worker.postMessage({observePort:channel.port2},[channel.port2]);return worker}});
  const control=buffers['scan-jpeg-control.pdf'];
  try{
   await session.importPdf(control);check((await session.inspect()).pageCount===1,'real Worker control success');
   try{await session.importPdf(new Uint8Array([1,2,3]))}catch(e){check(e.code==='PDF_INVALID_INPUT','Worker malformed input fails')}
   check((await session.inspect()).pageCount===1,'Worker failure preserves active document');
   if(exhaustedBytes){let code;try{await session.importPdf(exhaustedBytes)}catch(e){code=e.code}check(code==='PDF_RESOURCE_LIMIT','Worker codec work budget rejects');check((await session.inspect()).pageCount===1,'codec exhaustion preserves active Worker document');await session.importPdf(control);check((await session.inspect()).pageCount===1,'Worker recovers after codec exhaustion')}
   const usable=expected.jpeg2000?buffers['scan-jpx-200.pdf']:expected.jbig2?buffers['scan-jbig2-200.pdf']:buffers['scan-jpeg-control-200.pdf'];
   try{await session.importPdf(new Uint8Array([1,2,3]))}catch(e){check(e.code==='PDF_INVALID_INPUT','warm candidate before cancellation')}
   const cancellationEntries=entryCounts[usable.byteLength]||0;
   const controller=new AbortController();cancelOnEntry=controller;cancelBytes=usable.byteLength;const cancelled=session.importPdf(usable,{}, {signal:controller.signal});
   try{await cancelled;throw Error('expected cancellation')}catch(e){check(e.code==='PDF_IMPORT_CANCELLED','Worker cancellation code')}
   cancelOnEntry=null;check((entryCounts[usable.byteLength]||0)>cancellationEntries,'cancel observed actual WASM import API entry');
   check((await session.inspect()).pageCount===1,'cancellation preserves active document');
   await session.importPdf(control);check((await session.inspect()).pageCount===1,'Worker recreated after cancel');
   try{await session.importPdf(new Uint8Array([1,2,3]))}catch(e){check(e.code==='PDF_INVALID_INPUT','warm candidate before deadline')}
   const deadlineEntries=entryCounts[usable.byteLength]||0;
   try{await session.importPdf(usable,{}, {timeoutMs:10});throw Error('expected timeout')}catch(e){check(e.code==='PDF_IMPORT_TIMEOUT','Worker deadline code')}
   check((entryCounts[usable.byteLength]||0)>deadlineEntries,'deadline observed actual WASM import API entry');
   check((await session.inspect()).pageCount===1,'deadline preserves active document');
   await session.importPdf(control);check((await session.inspect()).pageCount===1,'Worker recreated after deadline');
   if(expected.jpeg2000||expected.jbig2){const imported=await session.importPdf(usable);check(imported.diagnostics.resourceUsage.codecWorkUnits>0,'Worker codec work accounted')}
   const unsupported=names.find(n=>n==='jbig2.pdf'&&!expected.jbig2||n==='jpx.pdf'&&!expected.jpeg2000);
   if(unsupported){try{await session.importPdf(buffers[unsupported]);throw Error('expected unsupported')}catch(e){check(/PDF_(PARSE_FAILED|IMAGE_UNSUPPORTED)/.test(e.code),'Worker missing codec fails')};check((await session.inspect()).pageCount===1,'missing codec preserves active document');await session.importPdf(control);check((await session.inspect()).pageCount===1,'Worker recovery after missing codec')}
  }finally{session.dispose();for(const port of tracePorts)port.close();URL.revokeObjectURL(wrapperUrl)}
  return {support,cases,checked,wasmMemoryBytes:wasm.memory.buffer.byteLength};
 },{expected,names:Object.keys(inputs)});
 result.artifacts=Object.fromEntries(['rhwp.js','rhwp_bg.wasm'].map(n=>{const b=files.get('/pkg/'+n);return [n,{bytes:b.length,sha256:createHash('sha256').update(b).digest('hex')}]}));
 writeFileSync(join(out,`wasm-${config}-probe.json`),JSON.stringify(result,null,2)+'\n');console.log(config,'PASS',result.cases.length,'cases',result.checked.length,'checks');
}finally{await browser?.close();await new Promise(r=>server.close(r))}

});
