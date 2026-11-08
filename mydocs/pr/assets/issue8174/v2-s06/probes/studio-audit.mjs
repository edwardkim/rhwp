import {createRequire} from 'node:module';
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
const R=process.cwd(),O=path.join(R,'output/pr-review/issue8174/v2-s06-20261025');
const require=createRequire(path.join(R,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const browser=await puppeteer.launch({executablePath:'/home/edward/.cache/puppeteer/chrome-headless-shell/linux-155.0.8059.79/chrome-headless-shell-linux64/chrome-headless-shell',headless:true,args:['--no-sandbox','--disable-dev-shm-usage']});
const page=await browser.newPage();await page.setViewport({width:1400,height:1000});page.on('dialog',d=>d.accept());
const errors=[];page.on('pageerror',e=>errors.push(String(e)));
const dialog='.dialog-pdf-import';
async function open(name){await(await page.$('#file-input')).uploadFile(path.join(O,'inputs',name));await page.waitForSelector(dialog,{timeout:30000});await page.evaluate(()=>{const d=document.querySelector('.dialog-pdf-import');d.querySelector('details').open=true;const field=d.querySelector('textarea');field.value=JSON.stringify({backend:{fontEntries:1}});field.dispatchEvent(new Event('input'))})}
async function state(){return page.evaluate(async()=>({generation:window.__wasm.documentGeneration,name:window.__wasm.fileName,pages:window.__wasm.pageCount,draft:window.__autosaveManager.getCurrentDraftId(),stored:(await(await import('/src/recovery/autosave-store.ts')).listAutosaveDrafts()).map(x=>x.id),handle:window.__wasm.currentFileHandle?.name??null,recents:(await(await import('/src/recent/recent-store.ts')).listRecentDocs()).map(x=>x.fileName)}))}
try{
 await page.goto('http://127.0.0.1:7704/?renderer=canvaskit',{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__renderBackend&&window.__wasm.pageCount>0,{timeout:90000});
 await(await page.$('#file-input')).uploadFile(path.join(R,'samples/hwp_table_test.hwp'));await page.waitForFunction(()=>window.__wasm.fileName==='hwp_table_test.hwp'&&document.querySelector('canvas'),{timeout:90000});
 await page.evaluate(async()=>{await window.__autosaveManager.flushNow('md05-audit');window.__documentState.markClean('test')});
 const before=await state();
 await open('cid-width-control.pdf');await page.click(`${dialog} .dialog-btn-primary`);
 await page.waitForFunction(()=>document.querySelector('.dialog-pdf-import [role=status]')?.textContent.includes('PDF_RESOURCE_LIMIT'),{timeout:30000});
 const rejected=await page.$eval(`${dialog} [role=status]`,e=>e.textContent),afterFailure=await state();assert.deepEqual(afterFailure,before);
 await page.evaluate(()=>document.querySelector('.dialog-pdf-import .dialog-footer .dialog-btn:not(.dialog-btn-primary)').click());await page.waitForSelector(dialog,{hidden:true});
 await open('cff-cid-1001-glyphs.pdf');await page.click(`${dialog} .dialog-btn-primary`);
 await page.waitForFunction(()=>window.__wasm.getSourceFormat()==='pdf'&&!document.querySelector('.dialog-pdf-import'),{timeout:90000});
 const accepted=await page.evaluate(()=>({name:window.__wasm.fileName,pages:window.__wasm.pageCount,diagnostics:window.__wasm.getPdfImportDiagnostics()}));
 assert.equal(accepted.pages,1);assert.equal(accepted.diagnostics.resourcePolicy.backend.fontEntries,1);
 await page.screenshot({path:path.join(O,'studio-cff-accepted.png')});
 assert.equal(errors.length,0);
 fs.writeFileSync(path.join(O,'studio-audit.json'),JSON.stringify({before,afterFailure,rejected,accepted,errors,scope:'actual Studio UI/Worker/main-thread reopen audit; resource acceptance is a finding, not visual fidelity proof'},null,2)+'\n');
 console.log(JSON.stringify({preserved:JSON.stringify(before)===JSON.stringify(afterFailure),rejected,accepted:accepted.name,fontEntries:accepted.diagnostics.resourcePolicy.backend.fontEntries,pages:accepted.pages,errors},null,2));
}finally{await browser.close()}
