import { fileURLToPath } from 'node:url';
const evidenceRoot=path.resolve(process.env.ISSUE8174_EVIDENCE ?? 'output/pr-review/issue8174/v2-s04-reproduce');
const fixtureRoot=fileURLToPath(new URL('../fixtures/',import.meta.url));
if(!process.env.CHROME_PATH)throw Error('Set CHROME_PATH to the installed Chrome/Chromium executable');
fs.mkdirSync(evidenceRoot,{recursive:true});
fs.copyFileSync(path.join(fixtureRoot,'button-calculation-scripted.hwp'),path.join(evidenceRoot,'button-calculation-scripted.hwp'));
fs.copyFileSync(path.join(process.cwd(),'pdf/issue8174/features/icc-rgb.pdf'),path.join(evidenceRoot,'renamed-input.hwp'));
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
const root=process.cwd(),out=evidenceRoot;
const require=createRequire(path.join(root,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const profile=path.join(out,'recovery-open-races-profile'),checked=[],timings={};let browser,page;
const password=String.fromCharCode(49,50,51,52,53,54),marker='S04 recovery content';
const check=(condition,label)=>{assert.ok(condition,label);checked.push(label);console.log('PASS',label);};
async function launch(skipRecovery=false){browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true,args:['--no-sandbox','--disable-dev-shm-usage'],userDataDir:profile});page=await browser.newPage();await page.setViewport({width:1400,height:1000});page.on('dialog',d=>d.accept());await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit'+(skipRecovery?'&url=':''),{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__wasm&&window.__canvasView,{timeout:90000});}
async function store(code,...args){return page.evaluate(async(code,args)=>{const api=await import('/src/recovery/autosave-store.ts');return (new Function('api','args',`return (${code})(api,...args)`))(api,args);},code.toString(),args);}
async function pass(value){await page.waitForSelector('#hwp-password-input',{timeout:30000});await page.$eval('#hwp-password-input',el=>el.value='');await page.type('#hwp-password-input',value);await page.evaluate(()=>document.querySelector('#hwp-password-input').closest('.dialog-wrap').querySelector('.dialog-btn-primary').click());}
async function restart(){await browser.close();await launch();await page.waitForSelector('.recovery-dialog',{timeout:30000});}
async function restore(){await page.click('.recovery-dialog .dialog-btn-primary');await page.waitForSelector('#hwp-password-input',{timeout:30000});}
async function cancelPassword(){await page.evaluate(()=>{const d=document.querySelector('#hwp-password-input').closest('.dialog-wrap');[...d.querySelectorAll('button')].find(b=>b.textContent.trim()==='취소').click();});await page.waitForSelector('#hwp-password-input',{hidden:true});}
async function text(){return page.evaluate(()=>window.__wasm.doc.getTextFileUnicode());}

async function gateKdf(){await page.evaluate(()=>{const original=crypto.subtle.deriveKey.bind(crypto.subtle);window.__originalDerive=original;window.__kdfStarted=false;crypto.subtle.deriveKey=async(...args)=>{window.__kdfStarted=true;await new Promise(resolve=>window.__releaseKdf=resolve);return original(...args);};});}
async function replaceDuringKdf(){await page.waitForFunction(()=>window.__kdfStarted,{timeout:10000});const generation=await page.evaluate(()=>window.__wasm.documentGeneration);await page.evaluate(()=>window.__eventBus.emit('create-new-document',{skipUnsavedGuard:true}));await page.waitForFunction(g=>window.__wasm.documentGeneration!==g&&window.__wasm.pageCount===1,{timeout:30000},generation);const after=await page.evaluate(()=>{crypto.subtle.deriveKey=window.__originalDerive;window.__releaseKdf();return window.__wasm.documentGeneration;});await new Promise(r=>setTimeout(r,500));return after;}
try{
 await launch(true);await page.waitForFunction(()=>window.__wasm.pageCount>0&&window.__renderBackend,{timeout:90000});await store(api=>api.clearAutosaveDrafts());
 await(await page.$('#file-input')).uploadFile(path.join(out,'protected-seed.hwp'));await page.waitForSelector('#hwp-password-input');await gateKdf();await pass(password);const first=await replaceDuringKdf();
 check(await page.evaluate(g=>window.__wasm.documentGeneration===g&&!window.__wasm.requiresPasswordForSave&&window.__wasm.recoveryCipher===null,first),'a protected open cannot overwrite a document created during key derivation');
 await(await page.$('#file-input')).uploadFile(path.join(out,'protected-seed.hwp'));await pass(password);await page.waitForFunction(()=>window.__wasm.recoveryCipher&&document.querySelector('#file-input').value===''&&window.__wasm.fileName==='protected-seed.hwp',{timeout:30000});await page.evaluate(()=>window.__autosaveManager.flushNow('pending-recovery-race'));
 const id=(await store(api=>api.listAutosaveDrafts()))[0].id;await restart();await restore();await gateKdf();await pass(password);const second=await replaceDuringKdf();
 check(await page.evaluate(g=>window.__wasm.documentGeneration===g&&!window.__wasm.fileName.includes('복구본')&&window.__wasm.recoveryCipher===null,second),'a delayed recovery cannot overwrite a newer document');
 check(Boolean(await store((api,id)=>api.getAutosaveDraft(id),id)),'cancelled recovery during key derivation preserves the unconsumed encrypted draft');
 fs.writeFileSync(path.join(out,'browser-recovery-open-races.json'),JSON.stringify({checked,faultInjection:'A deterministic gate before actual WebCrypto deriveKey; real KDF and document replacement still execute.'},null,2));
}catch(e){await page.screenshot({path:path.join(out,'recovery-open-races-failure.png')}).catch(()=>{});throw e;}finally{await browser?.close();}
