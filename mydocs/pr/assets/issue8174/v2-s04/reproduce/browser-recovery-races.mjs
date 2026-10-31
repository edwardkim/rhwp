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
const profile=path.join(out,'recovery-races-profile'),checked=[],timings={};let browser,page;
const password=String.fromCharCode(49,50,51,52,53,54),marker='S04 recovery content';
const check=(condition,label)=>{assert.ok(condition,label);checked.push(label);console.log('PASS',label);};
async function launch(skipRecovery=false){browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true,args:['--no-sandbox','--disable-dev-shm-usage'],userDataDir:profile});page=await browser.newPage();await page.setViewport({width:1400,height:1000});page.on('dialog',d=>d.accept());await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit'+(skipRecovery?'&url=':''),{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__wasm&&window.__canvasView,{timeout:90000});}
async function store(code,...args){return page.evaluate(async(code,args)=>{const api=await import('/src/recovery/autosave-store.ts');return (new Function('api','args',`return (${code})(api,...args)`))(api,args);},code.toString(),args);}
async function pass(value){await page.waitForSelector('#hwp-password-input',{timeout:30000});await page.$eval('#hwp-password-input',el=>el.value='');await page.type('#hwp-password-input',value);await page.evaluate(()=>document.querySelector('#hwp-password-input').closest('.dialog-wrap').querySelector('.dialog-btn-primary').click());}
async function restart(){await browser.close();await launch();await page.waitForSelector('.recovery-dialog',{timeout:30000});}
async function restore(){await page.click('.recovery-dialog .dialog-btn-primary');await page.waitForSelector('#hwp-password-input',{timeout:30000});}
async function cancelPassword(){await page.evaluate(()=>{const d=document.querySelector('#hwp-password-input').closest('.dialog-wrap');[...d.querySelectorAll('button')].find(b=>b.textContent.trim()==='취소').click();});await page.waitForSelector('#hwp-password-input',{hidden:true});}
async function text(){return page.evaluate(()=>window.__wasm.doc.getTextFileUnicode());}

async function saveCommand(){await page.evaluate(()=>{document.querySelector('#menu-bar .menu-item .menu-title').dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));document.querySelector('.md-item[data-cmd="file:save"]').click();});await page.waitForSelector('#hwp-save-password-input',{timeout:10000});}
async function savePassword(value){await page.type('#hwp-save-password-input',value);await page.type('#hwp-save-password-confirmation',value);await page.evaluate(()=>document.querySelector('#hwp-save-password-input').closest('.dialog-wrap').querySelector('.dialog-btn-primary').click());}
try{
 await launch(true);await page.waitForFunction(()=>window.__wasm.pageCount>0&&window.__renderBackend,{timeout:90000});await store(api=>api.clearAutosaveDrafts());
 await(await page.$('#file-input')).uploadFile(path.join(out,'protected-seed.hwp'));await pass(password);await page.waitForFunction(()=>window.__wasm.recoveryCipher&&document.querySelector('canvas'),{timeout:90000});await new Promise(r=>setTimeout(r,500));
 await page.evaluate(()=>window.__autosaveManager.flushNow('before-save'));
 await page.evaluate(()=>{window.__oldCipher=window.__wasm.recoveryCipher;window.__oldDraftId=window.__autosaveManager.getCurrentDraftId();window.__saveMode='ok';window.__savedBlob=null;const originalUrl=URL.createObjectURL;URL.createObjectURL=function(blob){if(window.__saveMode==='fail')throw new Error('injected fallback download failure');return originalUrl.call(this,blob);};window.showSaveFilePicker=async options=>({name:options.suggestedName,async createWritable(){if(window.__saveMode==='cancel')throw new DOMException('cancelled','AbortError');return {async write(blob){if(window.__saveMode==='fail')throw new Error('injected write failure');window.__savedBlob=blob;},async close(){}};}});});
 await saveCommand();await page.evaluate(()=>document.querySelector('#hwp-save-password-input').closest('.dialog-wrap').querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)').click());
 check(await page.evaluate(()=>window.__oldCipher===window.__wasm.recoveryCipher&&window.__wasm.requiresPasswordForSave),'cancelled normal Save preserves the original recovery key and protection');
 for(const mode of ['cancel','fail']){await page.evaluate(mode=>window.__saveMode=mode,mode);await saveCommand();await savePassword(password+'next');await page.waitForSelector('#hwp-save-password-input',{hidden:true});await new Promise(r=>setTimeout(r,500));check(await page.evaluate(()=>window.__oldCipher===window.__wasm.recoveryCipher),`${mode} storage failure does not publish a candidate recovery key`);}
 await page.evaluate(()=>window.__saveMode='ok');await saveCommand();await savePassword(password+'next');await page.waitForFunction(()=>window.__savedBlob&&window.__oldCipher!==window.__wasm.recoveryCipher,{timeout:30000});
 check(await page.evaluate(()=>window.__wasm.requiresPasswordForSave),'successful normal Save rotates the key and retains password protection');
 const saved=await page.evaluate(async()=>Array.from(new Uint8Array(await window.__savedBlob.arrayBuffer())));fs.writeFileSync(path.join(out,'protected-normal-save.hwp'),Buffer.from(saved));
 const savedCheck=await page.evaluate(async({bytes,password})=>{const HwpDocument=window.__wasm.doc.constructor;let old=false;try{const d=HwpDocument.openWithPassword(new Uint8Array(bytes),password);d.free();}catch{old=true;}const d=HwpDocument.openWithPassword(new Uint8Array(bytes),password+'next');try{return {old,pages:d.pageCount()};}finally{d.free();}},{bytes:saved,password});check(savedCheck.old&&savedCheck.pages===1,'the persisted HWP requires the changed password');
 await page.waitForFunction(async()=>!(await(await import('/src/recovery/autosave-store.ts')).listAutosaveDrafts()).some(r=>r.id===window.__oldDraftId),{timeout:10000});
 await page.evaluate(()=>window.__autosaveManager.flushNow('after-key-rotation'));
 check(await page.evaluate(async password=>{const api=await import('/src/recovery/autosave-store.ts'),{RecoveryCipher}=await import('/src/recovery/recovery-crypto.ts');const r=(await api.listAutosaveDrafts())[0];let old=false;try{await RecoveryCipher.unlock(r,password);}catch{old=true;}const result=await RecoveryCipher.unlock(r,password+'next');result.data.fill(0);result.cipher.dispose();return old;},password),'new recovery draft also rejects the old password');
 const race=await page.evaluate(async()=>{const manager=window.__autosaveManager,api=await import('/src/recovery/autosave-store.ts');let release,entered;const ready=new Promise(r=>entered=r),gate=new Promise(r=>release=r),original=manager.store.saveDraft;manager.store.saveDraft=async draft=>{entered();await gate;return original(draft);};const id=manager.getCurrentDraftId(),pending=manager.flushNow('race');await ready;window.__eventBus.emit('create-new-document',{skipUnsavedGuard:true});await new Promise(r=>setTimeout(r,100));release();await pending;manager.store.saveDraft=original;return {gone:!await api.getAutosaveDraft(id),keyCleared:window.__wasm.recoveryCipher===null,protected:window.__wasm.requiresPasswordForSave};});
 check(race.gone&&race.keyCleared&&!race.protected,'document replacement during a queued encrypted write deletes the stale draft and clears the key');
 await page.waitForFunction(()=>window.__wasm.pageCount>0,{timeout:10000});
 const blocked=await page.evaluate(async()=>{const m=window.__autosaveManager;let reads=0;const original=m.exportBytes;m.exportBytes=()=>{reads++;return original();};window.__wasm.requiresPasswordForSave=true;await m.flushNow('locked');m.exportBytes=original;window.__wasm.requiresPasswordForSave=false;return reads;});check(blocked===0,'a protected document without its key is blocked before the plaintext exporter');
 fs.writeFileSync(path.join(out,'browser-recovery-races.json'),JSON.stringify({checked,race,storage:'Fake File System Access sink captures actual encrypted WASM export. No user filesystem writes.'},null,2));
}catch(e){await page.screenshot({path:path.join(out,'recovery-races-failure.png')}).catch(()=>{});throw e;}finally{await browser?.close();}
