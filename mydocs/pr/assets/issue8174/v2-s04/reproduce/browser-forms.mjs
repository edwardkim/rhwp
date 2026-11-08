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
const browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true,args:['--no-sandbox','--disable-dev-shm-usage']});
const page=await browser.newPage(),checked=[];const check=(v,label)=>{assert.ok(v,label);checked.push(label);console.log('PASS',label);};
await page.setViewport({width:1400,height:1100});page.on('dialog',d=>d.accept());
try{
 await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit',{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__renderBackend&&window.__wasm.pageCount>0,{timeout:90000});
 await (await page.$('#file-input')).uploadFile(path.join(out,'button-calculation-scripted.hwp'));
 await page.waitForFunction(()=>window.__wasm.fileName==='button-calculation-scripted.hwp'&&document.querySelector('canvas'),{timeout:90000});await new Promise(r=>setTimeout(r,700));
 const target=await page.evaluate(()=>{
  const w=window.__wasm;const tree=JSON.parse(w.doc.getPageRenderTree(0));let found;
  const walk=n=>{if(n.type==='Form'){const b=n.bbox,h=w.getFormObjectAt(0,b.x+b.w/2,b.y+b.h/2);if(h.formType==='PushButton')found={...h,bbox:b};}for(const c of n.children??[])walk(c);};walk(tree);if(!found)throw Error('button not found');
  const point=w.hitTest(0,found.bbox.x+found.bbox.w/2,found.bbox.y+found.bbox.h/2);
  const address={section:found.sec,para:point.parentParaIndex??found.para,control:found.ci,cellPath:point.cellPath?.map(p=>[p.controlIndex,p.cellIndex,p.cellParaIndex])??[]};
  window.__buttonAddress=address;window.__buttonHit=found;
  const r=window.__inputHandler.formBboxToOverlayRect(found.bbox,0),b=document.querySelector('#scroll-content').getBoundingClientRect();
  return {address,hit:found,x:b.left+r.left+r.width/2,y:b.top+r.top+r.height/2,context:w.getFormActionContext(address)};
 });
 check(target.context.source.includes('total+=i'),'actual CFB Click script reaches the WASM action context');
 fs.writeFileSync(path.join(out,'form-before-context.json'),JSON.stringify(target.context,null,2));
 await page.screenshot({path:path.join(out,'form-before.png')});
 await page.mouse.click(target.x,target.y);
 await page.waitForFunction(()=>window.__wasm.getFormActionContext(window.__buttonAddress).forms.some(f=>f.name==='Edit'&&f.text==='합계: 10'),{timeout:30000});
 const after=await page.evaluate(()=>window.__wasm.getFormActionContext(window.__buttonAddress));
 check(after.forms.some(f=>f.name==='CheckBox'&&f.value===1)&&after.forms.some(f=>f.name==='PushButton'&&f.caption==='계산 완료'),'actual pointer click computes ten, checks the box and changes the caption');
 check(await page.evaluate(()=>window.__inputHandler.canUndo()),'button result is recorded in the existing undo history');
 await page.evaluate(()=>window.__inputHandler.performUndo());
 const undo=await page.evaluate(()=>window.__wasm.getFormActionContext(window.__buttonAddress));
 check(JSON.stringify(undo.forms)===JSON.stringify(target.context.forms),'one undo restores every form value atomically');
 await page.evaluate(()=>window.__inputHandler.performRedo());
 check(await page.evaluate(()=>window.__wasm.getFormActionContext(window.__buttonAddress).forms.some(f=>f.text==='합계: 10')),'one redo restores the calculation');
 for(const format of ['hwp','hwpx']){
  const saved=await page.evaluate(format=>Array.from(format==='hwp'?window.__wasm.exportHwp():window.__wasm.exportHwpx()),format);
  fs.writeFileSync(path.join(out,`button-after.${format}`),Buffer.from(saved));
  const reopened=await page.evaluate(async({format,bytes,address})=>{const HwpDocument=window.__wasm.doc.constructor;const d=new HwpDocument(new Uint8Array(bytes));try{return JSON.parse(d.getFormActionContext(JSON.stringify(address)));}finally{d.free();}},{format,bytes:saved,address:target.address});
  check(reopened.source===target.context.source&&['Edit','CheckBox','PushButton'].every(name=>{const expected=after.forms.find(f=>f.name===name),actual=reopened.forms.find(f=>f.name===name);return actual&&['text','caption','value','enabled'].every(k=>actual[k]===expected[k]);}),`${format} save and actual reparse preserve script and all three action results`);
 }
 await page.screenshot({path:path.join(out,'form-after.png')});
 const boundaries=await page.evaluate(async()=>{
  const {planFormAction}=await import('/src/engine/input-handler-form-action.ts');const w=window.__wasm,i=window.__inputHandler;const c=w.getFormActionContext(window.__buttonAddress),result={};
  const failing={...c,source:'function OnPushButton_Click(){Edit.Text="must not leak";while(true){}}'};
  try{await planFormAction(failing,new AbortController().signal);result.loop=false;}catch(e){result.loop=String(e).includes('FORM_ACTION_SCRIPT');}
  try{await planFormAction(c,new AbortController().signal,1);result.timeout=false;}catch(e){result.timeout=String(e).includes('FORM_ACTION_TIMEOUT');}
  const controller=new AbortController();const cancelled=planFormAction(c,controller.signal);controller.abort();try{await cancelled;result.cancel=false;}catch(e){result.cancel=String(e).includes('CANCELLED');}
  result.next=(await planFormAction(c,new AbortController().signal)).updates.some(f=>f.text==='합계: 10');
  result.noPartial=w.getFormActionContext(window.__buttonAddress).forms.every(f=>f.text!=='must not leak');
  window.rhwpStudio.forms.setActionsEnabled(false);try{i.executeFormActionPlan({updates:c.forms.filter(f=>f.name==='Edit').map(({name,formType,...f})=>({...f,text:'read-only leak'})),commands:[],steps:1});result.readOnly=false;}catch(e){result.readOnly=String(e).includes('READ_ONLY')&&w.getFormActionContext(window.__buttonAddress).forms.every(f=>f.text!=='read-only leak');}window.rhwpStudio.forms.setActionsEnabled(true);
  i.setEditMode('form');try{i.executeFormActionPlan({updates:[],commands:[{name:'MoveDocEnd'},{name:'InsertText',text:'forbidden'}],steps:1});result.formGuard=false;}catch(e){result.formGuard=String(e).includes('PROTECTED');}
  i.executeFormActionPlan({updates:c.forms.filter(f=>f.name==='Edit').map(({name,formType,...f})=>({...f,text:'양식 모드 입력'})),commands:[],steps:1});
  result.formWrite=w.getFormActionContext(window.__buttonAddress).forms.some(f=>f.text==='양식 모드 입력');i.performUndo();i.setEditMode('normal');
  const original=w.doc.getTextFileUnicode();i.executeFormActionPlan({updates:[],commands:[{name:'MoveDocEnd'},{name:'InsertText',text:' BODY ACTION'}],steps:1});result.body=w.doc.getTextFileUnicode().includes('BODY ACTION');i.performUndo();result.bodyUndo=w.doc.getTextFileUnicode()===original;
  return result;
 });
 for(const [name,value]of Object.entries(boundaries))check(value,`actual Worker/editor boundary: ${name}`);
 const lifecycle=await page.evaluate(async()=>{
  const {executeFormButton}=await import('/src/engine/input-handler-form-action.ts');const original=window.Worker;let created=0,terminated=0,applied=0;
  window.Worker=class extends original{constructor(...args){super(...args);created++;}postMessage(){}terminate(){terminated++;return super.terminate();}};
  const handler=window.__inputHandler,self={owner:handler,wasm:window.__wasm,eventBus:window.__eventBus,executePlan:()=>{applied++;}};
  window.rhwpStudio.forms.setActionsEnabled(false);let readOnly=false;try{await executeFormButton(self,window.__buttonHit,0);}catch(e){readOnly=String(e).includes('READ_ONLY')&&created===0;}window.rhwpStudio.forms.setActionsEnabled(true);
  const pending=executeFormButton(self,window.__buttonHit,0);let busy=false;try{await executeFormButton(self,window.__buttonHit,0);}catch(e){busy=String(e).includes('BUSY');}
  const generation=window.__wasm.documentGeneration;window.__eventBus.emit('create-new-document',{skipUnsavedGuard:true});let cancelled=false;try{await pending;}catch(e){cancelled=String(e).includes('CANCELLED')||String(e).includes('STALE');}finally{window.Worker=original;}
  return {readOnly,busy,cancelled,noPartial:applied===0,retired:created===terminated,generation};
 });for(const [name,value]of Object.entries(lifecycle))if(name!=='generation')check(value,`button owner lifecycle: ${name}`);
 await page.waitForFunction(generation=>window.__wasm.documentGeneration!==generation&&!window.__wasm.fileName.includes('scripted')&&document.querySelector('canvas')&&document.querySelector('#sb-message').textContent.includes('새 문서.hwp — 1페이지'),{timeout:30000},lifecycle.generation);
 await page.evaluate(()=>window.__inputHandler.executeFormActionPlan({updates:[],commands:[{name:'MoveDocEnd'},{name:'InsertText',text:'normal edit after cancellation'}],steps:1}));
 check(await page.evaluate(()=>window.__wasm.doc.getTextFileUnicode().includes('normal edit after cancellation')),'normal editing succeeds after a cancelled button and document replacement');

 fs.writeFileSync(path.join(out,'form-after-context.json'),JSON.stringify(after,null,2));fs.writeFileSync(path.join(out,'browser-forms.json'),JSON.stringify({checked,target:{address:target.address,hit:target.hit},boundaries,lifecycle},null,2));
}catch(e){await page.screenshot({path:path.join(out,'form-failure.png')}).catch(()=>{});console.error(await page.evaluate(()=>({name:window.__wasm?.fileName,status:document.querySelector('#sb-message')?.textContent,body:document.body.textContent.slice(-1000)})).catch(()=>{}));throw e;}finally{await browser.close();}
