// Data-preservation regression only; no rendering/page-fidelity expectation.
import {createRequire} from 'node:module';import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
const root=process.cwd(),out=path.resolve(process.env.ISSUE8174_EVIDENCE??'output/pr-review/issue8174/v2-s04-reproduce'),require=createRequire(path.join(root,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
if(!process.env.CHROME_PATH)throw Error('Set CHROME_PATH');fs.mkdirSync(out,{recursive:true});
const browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true});
try{
 const page=await browser.newPage();await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit');await page.waitForFunction(()=>window.__wasm?.doc,{timeout:90000});
 const bytes=Array.from(fs.readFileSync('samples/한글문서파일형식_5.0_revision1.3.hwp'));
 const result=await page.evaluate(bytes=>{const HwpDocument=window.__wasm.doc.constructor,d=new HwpDocument(new Uint8Array(bytes));try{const exported=d.exportHwpx(),round=new HwpDocument(exported);try{return {exported:Array.from(exported),textSame:d.getTextFileUnicode()===round.getTextFileUnicode(),beforePages:d.pageCount(),afterPages:round.pageCount()};}finally{round.free();}}finally{d.free();}},bytes);
 assert.ok(result.textSame);assert.equal(result.beforePages,result.afterPages);fs.writeFileSync(path.join(out,'distribution-export.hwpx'),Buffer.from(result.exported));delete result.exported;
 fs.writeFileSync(path.join(out,'browser-distribution.json'),JSON.stringify({checked:['public distribution HWP exports through actual product WASM','reopened export preserves source text and existing relative page-count contract'],...result,scope:'Data/roundtrip preservation only. Independent PDF fidelity is not asserted and no layout expectation changes.'},null,2)+'\n');console.log('PASS actual WASM distribution-envelope HWPX export and reparse');
}finally{await browser.close();}
