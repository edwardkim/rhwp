import { fileURLToPath } from 'node:url';
const evidenceRoot=path.resolve(process.env.ISSUE8174_EVIDENCE ?? 'output/pr-review/issue8174/v2-s04-reproduce');
const fixtureRoot=fileURLToPath(new URL('../fixtures/',import.meta.url));
if(!process.env.CHROME_PATH)throw Error('Set CHROME_PATH to the installed Chrome/Chromium executable');
fs.mkdirSync(evidenceRoot,{recursive:true});
fs.copyFileSync(path.join(fixtureRoot,'button-calculation-scripted.hwp'),path.join(evidenceRoot,'button-calculation-scripted.hwp'));
fs.copyFileSync(path.join(process.cwd(),'pdf/issue8174/features/icc-rgb.pdf'),path.join(evidenceRoot,'renamed-input.hwp'));
import {createRequire} from 'node:module';import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
const root=process.cwd(),out=evidenceRoot,require=createRequire(path.join(root,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const cases=JSON.parse(fs.readFileSync(path.join(fixtureRoot,'pdf-cases.json'),'utf8')).map(f=>({...f,input:path.join(root,f.input),reference:path.join(root,f.reference)}));
const browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true,args:['--no-sandbox','--disable-dev-shm-usage']});const page=await browser.newPage(),rows=[];fs.mkdirSync(path.join(out,'studio-matrix'),{recursive:true});
try{
 await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit',{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__renderBackend&&window.__wasm.pageCount>0,{timeout:90000});
 for(const fixture of cases){
  const input=Array.from(fs.readFileSync(fixture.input));
  const result=await page.evaluate(async({input,name})=>{
   const {preparePdfForEditor}=await import('/src/core/pdf-open.ts'),{CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');
   const w=window.__wasm,phases=[],bytes=new Uint8Array(input),start=performance.now();
   const prepared=await preparePdfForEditor({byteLength:bytes.length,read:async()=>bytes},{resourceProfile:'standard'},w.getPdfImportPolicy({}),{signal:new AbortController().signal,timeoutMs:30000,onProgress:p=>phases.push(p)});
   const converted=performance.now();w.loadImportedPdf(prepared,name+'.pdf');const reopened=performance.now();const tree=w.getPageLayerTreeObject(0,'print');
   const r=await CanvasKitLayerRenderer.create('strict','software'),c=document.createElement('canvas');c.width=Math.ceil(tree.pageWidth);c.height=Math.ceil(tree.pageHeight);const rendered=r.renderPage(tree,c,1,undefined,undefined,w.documentGeneration),png=rendered.toDataURL().split(',')[1],diagnostics=r.diagnostics();r.dispose();
   return {pages:w.pageCount,phases,conversionMs:converted-start,reopenMs:reopened-converted,totalMs:performance.now()-start,policy:prepared.diagnostics.resourcePolicy,png,hwpx:Array.from(w.exportHwpx()),diagnostics};
  },{input,name:fixture.name});
  assert.equal(result.pages,1,fixture.name);assert.equal(result.diagnostics.lastRenderCompleted,true,fixture.name);assert.equal(result.diagnostics.passesRuntimeReadinessGate,true,fixture.name);
  fs.writeFileSync(path.join(out,'studio-matrix',fixture.name+'.png'),Buffer.from(result.png,'base64'));fs.writeFileSync(path.join(out,'studio-matrix',fixture.name+'.hwpx'),Buffer.from(result.hwpx));delete result.png;delete result.hwpx;rows.push({...fixture,...result});console.log('PASS',fixture.name,result.totalMs);
 }
 fs.writeFileSync(path.join(out,'browser-matrix.json'),JSON.stringify(rows,null,2));check();
 function check(){assert.equal(rows.length,41);}
}finally{await browser.close();}
