import fs from 'node:fs';import path from 'node:path';import http from 'node:http';import assert from 'node:assert/strict';import {createRequire} from 'node:module';
const R=process.cwd(),O=path.join(R,'output/pr-review/issue8174/v2-s06-20261025'),require=createRequire(path.join(R,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const cases=JSON.parse(fs.readFileSync(path.join(O,'jpeg-inputs.json'),'utf8')),files=new Map();
for(const name of ['jpeg_audit.js','jpeg_audit_bg.wasm'])files.set('/'+name,fs.readFileSync(path.join(O,'jpeg-wasm',name)));
for(const item of cases)files.set('/'+item.name,fs.readFileSync(path.join(O,'jpeg-inputs',item.name)));
const server=http.createServer((req,res)=>{if(req.url==='/')return res.end('<!doctype html><meta charset="utf-8">');const bytes=files.get(req.url);if(!bytes)return res.writeHead(404).end();res.setHeader('Content-Type',req.url.endsWith('.wasm')?'application/wasm':req.url.endsWith('.js')?'text/javascript':'image/jpeg');res.end(bytes)});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser;
try{
 browser=await puppeteer.launch({executablePath:'/home/edward/.cache/puppeteer/chrome-headless-shell/linux-155.0.8059.79/chrome-headless-shell-linux64/chrome-headless-shell',headless:true,args:['--no-sandbox','--disable-dev-shm-usage']});const page=await browser.newPage(),errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.goto('http://127.0.0.1:'+server.address().port+'/');
 const rows=await page.evaluate(async(cases)=>{const m=await import('/jpeg_audit.js');await m.default();const rows=[],healthy=new Uint8Array(await(await fetch('/baseline-1x1-s0.jpg')).arrayBuffer());for(const {name} of cases){const bytes=new Uint8Array(await(await fetch('/'+name)).arrayBuffer());const result=JSON.parse(m.audit_jpeg(bytes));const next=JSON.parse(m.audit_jpeg(healthy));rows.push({name,result,healthyFollowupPixels:next.pixel_prefix})}return rows},cases);
 assert.equal(rows.length,52);assert.equal(errors.length,0);assert.ok(rows.every(r=>r.result.original_decode_error?r.result.observer_error_equal:(r.result.unmodified_pixels_equal&&r.result.one_below_rejected&&r.result.exact_budget_passed&&r.result.healthy_followup_passed)));assert.ok(rows.every(r=>JSON.stringify(r.healthyFollowupPixels)==='[0,0,0]'));
 const hCalls=rows.reduce((n,r)=>n+r.result.worker_h_calls,0),growth=rows.reduce((n,r)=>n+r.result.worker_h_capacity_growths,0);assert.equal(hCalls,106);assert.equal(growth,10);assert.ok(rows.filter(r=>r.result.worker_h_capacity_growths>0).every(r=>r.result.worker_growth_reservation_rejected));
 assert.deepEqual(rows.find(r=>r.name==='baseline-1x1-mixed-Y1x2-Cb2x2-Cr2x2.jpg').result.pixel_prefix,[0,0,0]);
 fs.writeFileSync(path.join(O,'jpeg-wasm.json'),JSON.stringify({rows,errors,hCalls,growth,scope:'optimized audit-only observer module and unmodified JPEG source in the same module; not a replacement Studio product bundle'},null,2)+'\n');console.log(JSON.stringify({cases:rows.length,hCalls,growth,errors}));
}finally{await browser?.close();await new Promise(r=>server.close(r))}
