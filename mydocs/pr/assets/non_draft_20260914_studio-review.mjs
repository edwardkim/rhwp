import puppeteer from '/Users/tsjang/rhwp/rhwp-studio/node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js';
import fs from 'node:fs';
const out='/private/tmp/rhwp-nondraft-review-20260914';
const browser=await puppeteer.launch({executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',headless:true,args:['--no-first-run','--disable-gpu']});
const page=await browser.newPage();page.on('dialog',d=>d.accept());await page.setViewport({width:1600,height:1100,deviceScaleFactor:1});
const errors=[];page.on('pageerror',e=>{errors.push(String(e));console.log('PAGEERROR',String(e));});page.on('console',msg=>{console.log(msg.type(),msg.text());if(msg.type()==='error')errors.push(msg.text());});
try {
await page.goto('http://127.0.0.1:7797/',{waitUntil:'domcontentloaded'});
await page.waitForFunction(()=>window.__wasm && window.__inputHandler,{timeout:120000});
const bytes=Array.from(fs.readFileSync('/Users/tsjang/rhwp/tests/fixtures/issue_7105/transistor-mosfet.hwp'));
await page.evaluate(async bytes=>{window.__wasm.loadDocument(new Uint8Array(bytes),'transistor-mosfet.hwp');await window.__canvasView.loadDocument();window.__inputHandler.activateWithCaretPosition();window.__canvasView.viewportManager.setZoom(0.4);},bytes);
await new Promise(r=>setTimeout(r,2000));
const initial=await page.evaluate(async()=>({pages:window.__wasm.pageCount,settings:(await import('/src/core/user-settings.ts')).userSettings.getViewSettings(),controls:Array.from({length:window.__wasm.pageCount},(_,i)=>({page:i,...window.__wasm.getPageControlLayout(i)}))}));
await page.screenshot({path:out+'/studio-single.png'});
const results=[];
for(const para of [18,22]) {
 const r=await page.evaluate(async para=>{
 const w=window.__wasm,ih=window.__inputHandler,a=window.rhwpStudio.automation;
 const all=()=>Array.from({length:w.pageCount},(_,i)=>w.getPageControlLayout(i).controls).flat();
 const target=all().find(x=>x.paraIdx===para&&x.type==='ole');if(!target)throw new Error('OLE missing para '+para);
 const before=all().filter(x=>x.type==='ole').length;
 ih.selectPictureObject(target.secIdx,target.paraIdx,target.controlIdx,target.type);
 const props=w.getShapeProperties(target.secIdx,target.paraIdx,target.controlIdx);
 const removed=a.execute('insert:picture-delete');const after=all().filter(x=>x.type==='ole').length;
 const undo=a.execute('edit:undo');const restored=all().filter(x=>x.type==='ole').length;
 return {para,target,before,after,restored,removed,undo,propertiesReadable:!!props};
 },para);results.push(r);
}
await page.screenshot({path:out+'/studio-ole-restored.png'});
await page.evaluate(()=>{const x=JSON.parse(localStorage.getItem('rhwp-settings')||'{}');x.view={...x.view,pageArrangement:{kind:'auto'}};localStorage.setItem('rhwp-settings',JSON.stringify(x));});
fs.writeFileSync(out+'/studio-result.json',JSON.stringify({browser:await browser.version(),initial,results,errors},null,2));
const second=await browser.newPage();await second.goto('http://127.0.0.1:7797/',{waitUntil:'domcontentloaded'});await second.waitForFunction(()=>window.__inputHandler,{timeout:120000});
const savedAuto=await second.evaluate(async()=> (await import('/src/core/user-settings.ts')).userSettings.getViewSettings().pageArrangement);
fs.writeFileSync(out+'/studio-result.json',JSON.stringify({browser:await browser.version(),initial,results,savedAuto,errors},null,2));
console.log(JSON.stringify({results,savedAuto,errors}));
} catch(e) {console.log('FAIL',String(e),errors); throw e;} finally {await browser.close();}
