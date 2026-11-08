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
const root=process.cwd(),out=evidenceRoot,prior=fixtureRoot;
const require=createRequire(path.join(root,'rhwp-studio/package.json')),puppeteer=require('puppeteer-core');
const browser=await puppeteer.launch({executablePath:process.env.CHROME_PATH,headless:true,args:['--no-sandbox','--disable-dev-shm-usage']});
const page=await browser.newPage(),rows=[];fs.mkdirSync(path.join(out,'studio-svg'),{recursive:true});
try{
 await page.goto((process.env.VITE_URL??'http://127.0.0.1:7704')+'/?renderer=canvaskit',{waitUntil:'domcontentloaded',timeout:90000});await page.waitForFunction(()=>window.__renderBackend,{timeout:90000});
 for(const name of ['gradient','clip-opacity','internal-use','restricted-script','embedded-png','embedded-jpeg','foreground']){
  const tree=JSON.parse(fs.readFileSync(path.join(prior,'svg',`${name}.json`),'utf8'));
  const result=await page.evaluate(async tree=>{
   const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');
   const renderer=await CanvasKitLayerRenderer.create('strict','software');
   const canvas=document.createElement('canvas');canvas.width=180;canvas.height=80;document.body.append(canvas);
   const start=performance.now();const output=renderer.renderPage(tree,canvas,1,undefined,undefined,91);
   const cold=performance.now()-start;const image=output.toDataURL('image/png');const first={...renderer.diagnostics(),selections:renderer.getTextVariantSelectionDiagnostics()};
   const again=performance.now();renderer.renderPage(tree,canvas,1,undefined,undefined,91);const warm=performance.now()-again;
   const second={...renderer.diagnostics(),selections:renderer.getTextVariantSelectionDiagnostics()};renderer.dispose();canvas.remove();return {png:image.split(',')[1],cold,warm,first,second};
  },tree);
  assert.equal(result.first.lastRenderCompleted,true,`${name} completed`);assert.equal(result.first.passesRuntimeReadinessGate,true);assert.equal(result.first.imageCacheEntries,1);assert.ok(result.second.imageCacheHits>=result.first.imageCacheHits);
  fs.writeFileSync(path.join(out,'studio-svg',`${name}.png`),Buffer.from(result.png,'base64'));
  delete result.png;rows.push({name,...result});console.log('SVG',name,result.cold,result.first.textVariants);
 }

 const probe=JSON.parse(fs.readFileSync(path.join(prior,'svg/gradient.json'),'utf8'));
 const negative=await page.evaluate(async original=>{
  const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');const {glyphOutlinePayloadResourceKey}=await import('/src/view/glyph-outline-payload-status.ts');const r=await CanvasKitLayerRenderer.create('strict','software'),c=document.createElement('canvas');c.width=180;c.height=80;const output=[];
  for(const change of ['proof','resource','geometry','transform']){const t=structuredClone(original),op=t.root.ops.find(o=>o.type==='glyphOutline');if(change==='proof')op.svgGlyph.staticSanitized=false;if(change==='resource')t.resources.svgFragments[0]+=' ';if(change==='geometry')op.svgGlyph.viewBox.width=0;if(change==='transform'){op.svgGlyph.transformToRun={a:1,b:0,c:0,d:1,e:40,f:10};op.payloadResourceKey=glyphOutlinePayloadResourceKey(op)+':resource:'+t.resources.svgKeys[0];}const p=r.renderPage(t,c,1,undefined,undefined,200+output.length);output.push({change,png:p.toDataURL().split(',')[1],diagnostics:r.diagnostics(),selections:r.getTextVariantSelectionDiagnostics()});}r.dispose();return output;
 },probe);for(const row of negative){fs.writeFileSync(path.join(out,`studio-svg/gradient-${row.change}.png`),Buffer.from(row.png,'base64'));delete row.png;}fs.writeFileSync(path.join(out,'svg-boundaries.json'),JSON.stringify(negative,null,2));
 fs.writeFileSync(path.join(out,'browser-svg.json'),JSON.stringify(rows,null,2));
}finally{await browser.close();}
