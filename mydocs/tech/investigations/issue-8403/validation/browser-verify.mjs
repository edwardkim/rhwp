// Public reproduction for #8403; admission evidence is in README.md.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
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
 const result=await page.evaluate(name=>{
  const doc=window.__wasm.doc;
  return Array.from({length:doc.pageCount()},(_,i)=>{
   const c=document.createElement('canvas');doc.renderPageToCanvasFilteredWithProfile(i,c,1,'all','print');
   if(name.startsWith('controls')&&i===2)window.__assert8403PatternCrop(c);
   // PDF text lives in positioned text boxes; getPageText reads root-body paragraphs only.
   const text=JSON.parse(doc.getPageTextLayout(i)).runs.map(run=>run.text).join('');
   if(!name.startsWith('controls')&&!text.trim())throw Error(name+': missing rendered page text '+(i+1));
   return {page:i+1,width:c.width,height:c.height,text,png:c.toDataURL('image/png')};
  });
 },name);
 const dir=new URL('wasm-'+name+'/',out);mkdirSync(dir,{recursive:true});
 for(const r of result)writeFileSync(new URL('page-'+r.page+'.png',dir),Buffer.from(r.png.split(',')[1],'base64'));
 const summary=result.map(r=>({page:r.page,width:r.width,height:r.height,textCharacters:[...r.text].length,textHash:hash(r.text),pngHash:hash(Buffer.from(r.png.split(',')[1],'base64'))}));
 records.push({name,pages:summary}); console.log('PASS',name,summary.length,'pages');return summary;
}
await runTest('PDF pattern images, saves and failed-import recovery (#8403)',async({page})=>{
 await page.evaluate(()=>{
  window.__assert8403PatternCrop=canvas=>{
   const svg=new DOMParser().parseFromString(window.__wasm.doc.renderPageSvgWithProfile(2,'print'),'image/svg+xml');
   const frames=[...svg.querySelectorAll('svg[viewBox]')].filter(n=>n.getAttribute('viewBox')==='0 0 1 2');
   if(!frames.length)throw Error('half-width source selection was discarded');
   const ctx=canvas.getContext('2d');
   for(const frame of frames){
    const x=+frame.getAttribute('x'),y=+frame.getAttribute('y'),w=+frame.getAttribute('width'),h=+frame.getAttribute('height');
    for(const [fraction,color] of [[.25,[255,0,0]],[.75,[0,0,255]]]){
     const actual=[...ctx.getImageData(Math.floor(x+w/2),Math.floor(y+h*fraction),1,1).data].slice(0,3);
     if(actual.some((value,i)=>value!==color[i]))throw Error('cropped tile includes the wrong source column: '+actual);
    }
   }
  };
 });
 const served=await page.evaluate(async()=>{
  const results=[];
  for(const path of ['/rhwp.js','/rhwp_bg.wasm']){
   const response=await fetch(path,{cache:'no-store'});if(!response.ok)throw Error(path+': '+response.status);
   const bytes=await response.arrayBuffer();
   const digest=await crypto.subtle.digest('SHA-256',bytes);
   results.push({path,bytes:bytes.byteLength,sha256:[...new Uint8Array(digest)].map(b=>b.toString(16).padStart(2,'0')).join('')});
  }return results;
 });
 for(const item of served)assert.equal(item.sha256,hash(readFileSync(new URL('rhwp-studio/public'+item.path,project))),'served asset differs from final build');
 writeFileSync(new URL('served-assets.json',out),JSON.stringify(served,null,2));
 await page.evaluate(()=>window.__eventBus.on('open-document-bytes:done',r=>{if(r.requestId==='pattern-8403')window.__patternResult=r;}));
 for(const [name,path,count] of [...(process.env.RHWP_8403_ORIGINAL ? [['original',process.env.RHWP_8403_ORIGINAL,8]] : []),['controls',fileURLToPath(new URL('image-patterns.pdf',fixture)),5],['stroke',fileURLToPath(new URL('text-stroke.pdf',fixture)),7],['legacy-a',fileURLToPath(new URL('pdf/pr7704/6778-square-2020.pdf',project)),6],['legacy-b',fileURLToPath(new URL('pdf/pr7704/6778-square-2020.pdf',project)),6]]){
  if(name==='legacy-a'&&existsSync(new URL('local-font-manifest.json',out))){
   const fontEvidence=[];
   for(const entry of JSON.parse(readFileSync(new URL('local-font-manifest.json',out),'utf8'))){
    const bytes=readFileSync(entry.path);
    await page.evaluate(async({aliases,base64})=>{
     const {registerLoadedLocalFontFace}=await import('/src/core/font-loader.ts');
     const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));
     for(const alias of aliases){const face=await new FontFace(alias,bytes).load();document.fonts.add(face);registerLoadedLocalFontFace(face);}
    },{aliases:entry.aliases,base64:bytes.toString('base64')});
    fontEvidence.push({aliases:entry.aliases,sha256:hash(bytes)});
   }
   writeFileSync(new URL('local-font-evidence.json',out),JSON.stringify(fontEvidence,null,2));
  }
  if(name.startsWith('legacy-')){
   // The complete 12-page document exceeds the standard cumulative HWPX
   // serialization budget. Keep that limit and exercise all source pages via
   // the supported selection API, using the actual Worker/editor handoff.
   const sourcePages=Array.from({length:6},(_,i)=>i+(name==='legacy-a'?0:6));
   await page.evaluate(async({bytes,name,sourcePages})=>{
    const {preparePdfForEditor}=await import('/src/core/pdf-open.ts');
    const input=new Uint8Array(bytes),options={sourcePages};
    const controller=new AbortController();
    const result=await preparePdfForEditor({byteLength:input.length,read:async()=>input},options,
     window.__wasm.getPdfImportPolicy(options),{signal:controller.signal,timeoutMs:120000});
    const info=await window.__wasm.loadImportedPdf(result,name+'.pdf',controller.signal);
    const {loadWebFonts}=await import('/src/core/font-loader.ts');
    await loadWebFonts(info.fontsUsed??[]);await document.fonts.ready;
    await window.__canvasView.loadDocument();
   },{bytes:[...readFileSync(path)],name,sourcePages});
  }else{await openBytes(page,readFileSync(path),name+'.pdf');await done(page);}
  const initial=await capture(page,name);assert.equal(initial.length,count);
  if(name==='stroke'||name==='controls'){
   const kit=await page.evaluate(async({bytes,name})=>{
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
      if(name==='controls'&&i===2)window.__assert8403PatternCrop(canvas);
      const diagnostics=renderer.diagnostics();
      if(!diagnostics.lastRenderCompleted||diagnostics.lastUnexpectedUnsupportedOps.length)throw Error(JSON.stringify(diagnostics));
      results.push({png,diagnostics});
     }return results;
    }finally{renderer.dispose();URL.revokeObjectURL(url);}
   },{bytes:[...readFileSync(new URL('ttfs/opensource/NotoSansKR-Regular.ttf',project))],name});
   mkdirSync(new URL('canvaskit-'+name+'/',out),{recursive:true});
   for(let i=0;i<kit.length;i++)writeFileSync(new URL('canvaskit-'+name+'/page-'+(i+1)+'.png',out),Buffer.from(kit[i].png.split(',')[1],'base64'));
   writeFileSync(new URL('canvaskit-'+name+'-diagnostics.json',out),JSON.stringify(kit.map(v=>v.diagnostics),null,2));
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
 const editedText=await page.evaluate(()=>Array.from({length:window.__wasm.doc.pageCount()},(_,i)=>JSON.parse(window.__wasm.doc.getPageTextLayout(i)).runs.map(r=>r.text).join('')).join(''));
 assert.equal((editedText.match(/ktroke/g)||[]).length,14,'rendered text must include every edit');
 assert.equal((editedText.match(/Stroke/g)||[]).length,0,'rendered text must not retain old words');
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
