// 관계형: 실제 CanvasKit의 투명한 외곽선과 공통 획/점선 소비를 검사한다.
// 주석 갱신: 2026-11-05T16:08:50+09:00; 직전 파일 변경: 0e365e1611d3159acc1bd35ef8467a190819c2c9 2026-11-05T13:08:21+09:00.
// 측정일 2026-11-05 KST(source d53115fabc): 독립 Print 원본 전 1쪽, Native/fresh WASM 최저 각각 100.00%, 96 DPI·2px raw.
// 전쪽 TSV/확대 판독은 mydocs/pr/assets/semanticist21-20261105-next10/validation.md에 연결한다.
// 이 검사는 번들 Noto 대체 face를 사용한 효과 소비 검사이며 Print 글꼴 일치 검사를 대체하지 않는다.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
import {launchBrowser,createPage,loadApp,closePage,closeBrowser} from './helpers.mjs';
const browser=await launchBrowser();const page=await createPage(browser);
try {
 await loadApp(page);
 const bytes=readFileSync(path.join(root,'tests/fixtures/char_outline_types/outline.hwp'));
 const result=await page.evaluate(async bytes=>{
  const bridge=window.__wasm;bridge.loadDocument(new Uint8Array(bytes),'outline.hwp');
  bridge.borrowDocumentHandle().setShowTransparentBorders(false);
  const tree=bridge.withPortableMetrics(()=>bridge.getPageLayerTreeObject(0,'print'));
  const rows=[];let backgrounds=0;
  const visit=n=>{
   if(n.kind==='group')n.children.forEach(visit);else if(n.kind==='clipRect')visit(n.child);else for(const op of n.ops){
    if(op.type==='pageBackground'){op.backgroundColor='#ffff00';backgrounds++;}
    if((op.type==='textRun' || op.type==='glyphRun') && op.style?.outlineType>0) rows.push({type:op.type,kind:op.style.outlineType,stroke:op.style.outlineStroke,bbox:op.bbox});
   }
  };visit(tree.root);
  const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');const renderer=await CanvasKitLayerRenderer.create('default','software');
  try{
   const api=renderer.canvasKit.PathEffect,make=api.MakeDash;const calls=[];
   api.MakeDash=function(p,...rest){calls.push(Array.from(p));return make.call(this,p,...rest);};
   const canvas=document.createElement('canvas');canvas.width=Math.ceil(tree.pageWidth);canvas.height=Math.ceil(tree.pageHeight);
   const image=renderer.renderPage(tree,canvas,1);api.MakeDash=make;
   const copy=document.createElement('canvas');copy.width=canvas.width;copy.height=canvas.height;const ctx=copy.getContext('2d');ctx.drawImage(image,0,0);
   const ink=rows.map(row=>{
    const {x,y,width,height}=row.bbox;const px=ctx.getImageData(Math.max(0,Math.floor(x)),Math.max(0,Math.floor(y)),Math.max(1,Math.ceil(width)),Math.max(1,Math.ceil(height))).data;
    let white=0,ink=0;for(let i=0;i<px.length;i+=4){if(px[i]>245&&px[i+1]>245&&px[i+2]>245)white++;if(px[i]<200&&px[i+1]<200&&px[i+3]>0)ink++;}
    return {...row,white,ink};
   });return {backgrounds,rows:ink,calls,diagnostics:renderer.diagnostics(),png:copy.toDataURL('image/png')};
  }finally{renderer.dispose();}
 },Array.from(bytes));
 assert.ok(result.backgrounds>0,'yellow background must be exercised');
 for(const kind of [1,2,3,4,5,6]){
  const rows=result.rows.filter(r=>r.kind===kind);assert.ok(rows.length>0,`outline ${kind} present`);assert.ok(rows.every(r=>r.white===0),`outline ${kind} must not fill white`);assert.ok(rows.some(r=>r.ink>0),`outline ${kind} paints actual ink`);
  assert.ok(rows.every(r=>r.stroke.width>0),'producer supplies a positive physical stroke');
 }
 const patterns=result.rows.filter(r=>r.stroke.intervals?.length).map(r=>r.stroke.intervals);
 assert.equal(new Set(patterns.map(p=>JSON.stringify(p))).size,4,'four declared dashed outline patterns remain distinct');
 for(const expected of patterns)assert.ok(result.calls.some(p=>p.length===expected.length&&p.every((v,i)=>Math.abs(v-expected[i])<1e-6)),`CanvasKit consumes producer dash ${expected}`);
 assert.equal(result.diagnostics.lastRenderCompleted,true);assert.deepEqual(result.diagnostics.lastUnexpectedUnsupportedOps,[]);
 const png=result.png;delete result.png;const out=process.env.RHWP_OUTLINE_TEST_OUTPUT;
 if(out){mkdirSync(out,{recursive:true});writeFileSync(path.join(out,'outline-canvaskit.png'),Buffer.from(png.split(',')[1],'base64'));writeFileSync(path.join(out,'outline-canvaskit.json'),JSON.stringify(result,null,2));}
 console.log(JSON.stringify(result));
}finally{await closePage(page);await closeBrowser(browser);}
