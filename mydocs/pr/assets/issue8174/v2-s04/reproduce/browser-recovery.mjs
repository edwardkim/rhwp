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
const profile=path.join(out,'recovery-profile-v2'),checked=[],timings={};let browser,page;
const password=String.fromCharCode(49,50,51,52,53,54),marker='S04 recovery content';
const check=(condition,label)=>{assert.ok(condition,label);checked.push(label);console.log('PASS',label);};
async function launch(skipRecovery=false){browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true,args:['--no-sandbox','--disable-dev-shm-usage'],userDataDir:profile});page=await browser.newPage();await page.setViewport({width:1400,height:1000});page.on('dialog',d=>d.accept());await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit'+(skipRecovery?'&url=':''),{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__wasm&&window.__canvasView,{timeout:90000});}
async function store(code,...args){return page.evaluate(async(code,args)=>{const api=await import('/src/recovery/autosave-store.ts');return (new Function('api','args',`return (${code})(api,...args)`))(api,args);},code.toString(),args);}
async function pass(value){await page.waitForSelector('#hwp-password-input',{timeout:30000});await page.$eval('#hwp-password-input',el=>el.value='');await page.type('#hwp-password-input',value);await page.evaluate(()=>document.querySelector('#hwp-password-input').closest('.dialog-wrap').querySelector('.dialog-btn-primary').click());}
async function restart(){await browser.close();await launch();await page.waitForSelector('.recovery-dialog',{timeout:30000});}
async function restore(){await page.click('.recovery-dialog .dialog-btn-primary');await page.waitForSelector('#hwp-password-input',{timeout:30000});}
async function cancelPassword(){await page.evaluate(()=>{const d=document.querySelector('#hwp-password-input').closest('.dialog-wrap');[...d.querySelectorAll('button')].find(b=>b.textContent.trim()==='취소').click();});await page.waitForSelector('#hwp-password-input',{hidden:true});}
async function text(){return page.evaluate(()=>window.__wasm.doc.getTextFileUnicode());}
try{
 await launch(true);await page.waitForFunction(()=>window.__wasm.pageCount>0&&window.__renderBackend,{timeout:90000});
 await store(api=>api.clearAutosaveDrafts());
 const seed=await page.evaluate(password=>{window.__wasm.insertText(0,0,0,'Recovery seed');return Array.from(window.__wasm.exportHwpWithPassword(password));},password);fs.writeFileSync(path.join(out,'protected-seed.hwp'),Buffer.from(seed));
 const t=performance.now();await (await page.$('#file-input')).uploadFile(path.join(out,'protected-seed.hwp'));await pass(password);
 await page.waitForFunction(()=>window.__wasm.requiresPasswordForSave&&window.__wasm.recoveryCipher&&window.__wasm.fileName==='protected-seed.hwp',{timeout:90000});timings.protectedOpenMs=performance.now()-t;
 await page.waitForFunction(()=>document.querySelector('canvas')?.width>0&&window.__renderBackend&&!document.getElementById('sb-message').textContent.includes('로딩'),{timeout:90000});await new Promise(r=>setTimeout(r,500));
 check(await page.evaluate(()=>document.querySelector('#hwp-password-input')===null),'password DOM removed after successful open');
 await page.evaluate(()=>{window.__inputHandler.cursor.moveTo({sectionIndex:0,paragraphIndex:0,charOffset:0});window.__inputHandler.textarea.focus();});
 await page.keyboard.sendCharacter(marker);await page.waitForFunction(marker=>window.__wasm.doc.getTextFileUnicode().includes(marker),{},marker);
 await page.evaluate(()=>window.__autosaveManager.flushNow('S04-browser-encrypted'));
 let rows=await store(async api=>(await api.listAutosaveDrafts()).map(r=>({...r,data:Array.from(r.data),protection:r.protection?{...r.protection,salt:Array.from(r.protection.salt),iv:Array.from(r.protection.iv)}:undefined})));
 check(rows.length===1&&rows[0].protection?.scheme==='PBKDF2-SHA256/AES-256-GCM','actual protected autosave persists one authenticated envelope');
 check(rows[0].data[0]!==0x50&&rows[0].data[0]!==0xd0&&!new TextDecoder().decode(new Uint8Array(rows[0].data)).includes(marker),'IndexedDB contains ciphertext instead of HWP/HWPX or content');
 check(!JSON.stringify(rows).includes(password),'draft metadata contains no password');
 const original=rows[0];fs.writeFileSync(path.join(out,'recovery-ciphertext-metadata.json'),JSON.stringify({...original,data:undefined},null,2));
 await restart();check(await page.$eval('.recovery-dialog',e=>e.textContent.includes('암호화')),'browser restart offers encrypted recovery');await restore();
 const old=await page.evaluate(()=>window.__wasm.documentGeneration);await pass('incorrect');
 await page.waitForFunction(()=>document.querySelector('#hwp-password-error')?.textContent.includes('변경되었습니다'),{timeout:30000});
 check((await page.evaluate(()=>window.__wasm.documentGeneration))===old,'wrong recovery password does not publish a document');
 await cancelPassword();check((await store(api=>api.listAutosaveDrafts())).length===1,'cancelled recovery preserves its draft');
 await restart();await restore();const rt=performance.now();await pass(password);
 await page.waitForFunction(()=>window.__wasm.fileName.includes('복구본')&&window.__wasm.requiresPasswordForSave&&!!window.__wasm.recoveryCipher,{timeout:90000});timings.recoveryUnlockOpenMs=performance.now()-rt;
 check((await text()).includes(marker),'restarted recovery restores actual edited text');
 await page.waitForFunction(()=>window.__documentState.isDirty(),{timeout:30000});check(await page.evaluate(()=>window.__documentState.isDirty()),'restored document is dirty and retains protected save intent');
 check((await store(api=>api.listAutosaveDrafts())).every(r=>r.id!==original.id),'successful recovery deletes only the consumed draft');
 await page.screenshot({path:path.join(out,'protected-recovery.png')});
 // New revision uses a fresh nonce under the same session key.
 await page.evaluate(()=>window.__autosaveManager.flushNow('S04-new-revision'));
 rows=await store(async api=>(await api.listAutosaveDrafts()).map(r=>({...r,data:Array.from(r.data),protection:{...r.protection,salt:Array.from(r.protection.salt),iv:Array.from(r.protection.iv)}})));
 const second=rows[0];check(second.protection.revision>original.protection.revision&&String(second.protection.iv)!==String(original.protection.iv),'recovery revision advances with a fresh nonce after restart');
 // Editing the authenticated metadata must fail before opening any plaintext.
 await store(async(api,id)=>{const r=await api.getAutosaveDraft(id);r.fileName='tampered.hwp';await api.saveAutosaveDraft(r);},second.id);
 await restart();await restore();await pass(password);await page.waitForFunction(()=>document.querySelector('#hwp-password-error')?.textContent.includes('변경되었습니다'),{timeout:30000});
 check(await page.evaluate(()=>window.__wasm.pageCount===0),'tampered metadata is rejected before document publication');await cancelPassword();
 // Restore the original envelope, then delete it while the password dialog is open.
 await store(async(api,r)=>{await api.clearAutosaveDrafts();r.data=new Uint8Array(r.data);r.protection.salt=new Uint8Array(r.protection.salt);r.protection.iv=new Uint8Array(r.protection.iv);await api.saveAutosaveDraft(r);},second);
 await restart();await restore();await store((api,id)=>api.deleteAutosaveDraft(id),second.id);await pass(password);
 await page.waitForFunction(()=>!document.querySelector('#hwp-password-input'),{timeout:30000});await new Promise(r=>setTimeout(r,300));
 check(await page.evaluate(()=>!window.__wasm.fileName.includes('복구본')),'a draft deleted during the password prompt is not restored');
 check((await store(api=>api.listAutosaveDrafts())).length===0,'stale recovery does not resurrect a deleted record');
 fs.writeFileSync(path.join(out,'browser-recovery.json'),JSON.stringify({checked,timings,source:'current checkout; record git rev-parse HEAD separately',notes:['Actual IndexedDB and browser process restart; public fixture password used only in memory. No host filesystem save was performed.']},null,2));
}catch(e){if(page){await page.screenshot({path:path.join(out,'recovery-failure.png')}).catch(()=>{});console.error(await page.evaluate(()=>({body:document.body.textContent.slice(-2500),name:window.__wasm?.fileName,pages:window.__wasm?.pageCount,text:window.__wasm?.doc?.getTextFileUnicode(),cursor:window.__inputHandler?.cursor?.getPosition()})).catch(()=>{}));}throw e;}finally{await browser?.close();}
