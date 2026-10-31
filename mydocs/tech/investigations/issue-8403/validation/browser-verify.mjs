// Public reproduction for #8403; admission evidence is in README.md.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {runTest} from '../../../../../rhwp-studio/e2e/helpers.mjs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
const project=new URL('../../../../../',import.meta.url);
const fixture=new URL('pdf/issue8403/',project);
const out=pathToFileURL(resolve(process.env.RHWP_8403_OUTPUT || fileURLToPath(new URL('output/issue8403-browser/',project)))+'/');
mkdirSync(out,{recursive:true});
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const records=[];
async function openBytes(page,bytes,name,pdf=true){
 await page.evaluate(({bytes,name})=>{
  window.__patternResult=null;
  window.__eventBus.emit('open-document-bytes',{bytes:new Uint8Array(bytes),fileName:name,
   skipUnsavedGuard:true,requestId:'pattern-8403'});
 },{bytes:[...bytes],name});
 if(pdf){await page.waitForSelector('.dialog-pdf-import');await page.click('.dialog-pdf-import .dialog-btn-primary');}
}
async function done(page){
 await page.waitForFunction(()=>window.__patternResult!==null&&!document.documentElement.classList.contains('rhwp-busy'),{timeout:120000});
 assert.equal((await page.evaluate(()=>window.__patternResult)).ok,true);
 await page.evaluate(()=>document.fonts.ready);
}
async function capture(page,name){
 const result=await page.evaluate(()=>{
  const doc=window.__wasm.doc;
  return Array.from({length:doc.pageCount()},(_,i)=>{
   const c=document.createElement('canvas');doc.renderPageToCanvasFilteredWithProfile(i,c,1,'all','print');
   return {page:i+1,width:c.width,height:c.height,text:doc.getPageText(i),png:c.toDataURL('image/png')};
  });
 });
 const dir=new URL('wasm-'+name+'/',out);mkdirSync(dir,{recursive:true});
 for(const r of result)writeFileSync(new URL('page-'+r.page+'.png',dir),Buffer.from(r.png.split(',')[1],'base64'));
 const summary=result.map(r=>({page:r.page,width:r.width,height:r.height,textHash:hash(r.text),pngHash:hash(Buffer.from(r.png.split(',')[1],'base64'))}));
 records.push({name,pages:summary}); console.log('PASS',name,summary.length,'pages');return summary;
}
await runTest('PDF pattern images, saves and failed-import recovery (#8403)',async({page})=>{
 await page.evaluate(()=>window.__eventBus.on('open-document-bytes:done',r=>{if(r.requestId==='pattern-8403')window.__patternResult=r;}));
 for(const [name,path,count] of [...(process.env.RHWP_8403_ORIGINAL ? [['original',process.env.RHWP_8403_ORIGINAL,8]] : []),['controls',fileURLToPath(new URL('image-patterns.pdf',fixture)),5],['stroke',fileURLToPath(new URL('text-stroke.pdf',fixture)),7]]){
  await openBytes(page,readFileSync(path),name+'.pdf');await done(page);
  const initial=await capture(page,name);assert.equal(initial.length,count);
  if(name==='stroke'){
   const kit=await page.evaluate(async bytes=>{
    const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');
    const renderer=await CanvasKitLayerRenderer.create('default','software');
    const url=URL.createObjectURL(new Blob([new Uint8Array(bytes)]));
    try{
     await renderer.prepareBundledFonts([{url,aliases:['Noto Sans KR','NotoSansKR-Regular']}]);
     const results=[];
     for(let i=0;i<window.__wasm.doc.pageCount();i++){
      const tree=window.__wasm.withPortableMetrics(()=>window.__wasm.getPageLayerTreeObject(i,'print'));
      await renderer.preparePageResources(tree);
      const canvas=document.createElement('canvas');canvas.width=Math.ceil(tree.pageWidth);canvas.height=Math.ceil(tree.pageHeight);
      const png=renderer.renderPage(tree,canvas,1).toDataURL('image/png');
      const diagnostics=renderer.diagnostics();
      if(!diagnostics.lastRenderCompleted||diagnostics.lastUnexpectedUnsupportedOps.length)throw Error(JSON.stringify(diagnostics));
      results.push({png,diagnostics});
     }return results;
    }finally{renderer.dispose();URL.revokeObjectURL(url);}
   },[...readFileSync(new URL('ttfs/opensource/NotoSansKR-Regular.ttf',project))]);
   mkdirSync(new URL('canvaskit-stroke/',out),{recursive:true});
   for(let i=0;i<kit.length;i++)writeFileSync(new URL('canvaskit-stroke/page-'+(i+1)+'.png',out),Buffer.from(kit[i].png.split(',')[1],'base64'));
   writeFileSync(new URL('canvaskit-stroke-diagnostics.json',out),JSON.stringify(kit.map(v=>v.diagnostics),null,2));
  }
  const saved=await page.evaluate(()=>({hwp:[...window.__wasm.doc.exportHwp()],hwpx:[...window.__wasm.doc.exportHwpx()]}));
  for(const [ext,bytes] of Object.entries(saved)){
   writeFileSync(new URL(name+'.'+ext,out),Buffer.from(bytes));
   await openBytes(page,bytes,name+'.'+ext,false);await done(page);
   const after=await capture(page,name+'-'+ext);
   assert.equal(after.length,count);assert.deepEqual(after.map(r=>r.textHash),initial.map(r=>r.textHash),'saved page text preserved');
  }
 }
 await openBytes(page,readFileSync(new URL('text-stroke.pdf',fixture)),'editable-stroke.pdf');await done(page);
 const edited=await page.evaluate(()=>{
  const doc=window.__wasm.doc;
  const result=JSON.parse(doc.replaceAll('Stroke','ktroke',true));
  return {result,svg:doc.renderPageSvgWithProfile(0,'print')};
 });
 console.log('EDIT',JSON.stringify(edited.result));
 assert.equal(edited.result.count,14,'every source occurrence must be edited');
 assert.ok(edited.svg.includes('paint-order="fill stroke"'),'edit lost the paint style');
 const editInitial=await capture(page,'stroke-edited');
 const editSaved=await page.evaluate(()=>({hwp:[...window.__wasm.doc.exportHwp()],hwpx:[...window.__wasm.doc.exportHwpx()]}));
 for(const [ext,bytes] of Object.entries(editSaved)){
  writeFileSync(new URL('stroke-edited.'+ext,out),Buffer.from(bytes));
  await openBytes(page,bytes,'stroke-edited.'+ext,false);await done(page);
  const after=await capture(page,'stroke-edited-'+ext);
  assert.deepEqual(after.map(v=>v.textHash),editInitial.map(v=>v.textHash));
 }
 for(const name of ['zero-step','huge-tiles','unsupported-paint','cyclic-pattern','uncoloured-pattern','shading-pattern','singular-pattern','dashed-text','anisotropic-text','clipped-text-stroke','pattern-text']){
  const before=await page.evaluate(()=>({generation:window.__wasm.documentGeneration,pages:window.__wasm.doc.pageCount()}));
  await openBytes(page,readFileSync(new URL(name+'.pdf',fixture)),name+'.pdf');
  await page.waitForFunction(()=>document.querySelector('.dialog-pdf-import .dialog-btn-primary')?.textContent==='다시 시도',{timeout:120000});
  const message=await page.$eval('.dialog-pdf-import [role="status"]',el=>el.textContent);
  assert.match(message,name==='huge-tiles'?/PDF_RESOURCE_LIMIT/:name.includes('text')?/PDF_TEXT_UNSUPPORTED/:/PDF_PATTERN_UNSUPPORTED/);
  assert.deepEqual(await page.evaluate(()=>({generation:window.__wasm.documentGeneration,pages:window.__wasm.doc.pageCount()})),before);
  await page.keyboard.press('Escape');await page.waitForSelector('.dialog-pdf-import',{hidden:true});
  await openBytes(page,readFileSync(new URL('image-patterns.pdf',fixture)),'recovered.pdf');await done(page);
  assert.equal(await page.evaluate(()=>window.__wasm.doc.pageCount()),5);
  records.push({name,rejected:true,recovered:true});console.log('PASS rejected and recovered',name);
 }
 writeFileSync(new URL('browser-verify-result.json',out),JSON.stringify(records,null,2));
});
