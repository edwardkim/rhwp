import assert from 'node:assert/strict';
import {createRequire,stripTypeScriptTypes} from 'node:module';
import {readFileSync,rmSync} from 'node:fs';
import {join,resolve} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {createSweepBrowserProfile,createSweepServer} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';
const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const require=createRequire(join(root,'rhwp-studio/package.json'));
// 기대 RGB는 대응하는 한컴2024 Print PDF 내장 이미지에서 직접 추출했다.
// tests/fixtures/issue_7751_color_controls/README.md의 SHA와 생성 job을 따른다.
const controls = {
  "hancom-blackwhite-zero-2024": [
    [[0,0,0],[50,50,50],[100,100,100],[128,128,128],[170,170,170],[206,206,206],[255,255,255],[80,100,120],[255,0,0],[0,255,0],[0,0,255]],
    [[20,20,20],[70,70,70],[120,120,120],[148,148,148],[190,190,190],[226,226,226],[255,255,255],[100,120,140],[255,20,20],[20,255,20],[20,20,255]],
    [[0,0,0],[43,43,43],[97,97,97],[128,128,128],[173,173,173],[212,212,212],[255,255,255],[76,97,119],[255,0,0],[0,255,0],[0,0,255]],
    [[51,51,51],[101,101,101],[151,151,151],[179,179,179],[221,221,221],[255,255,255],[255,255,255],[131,151,171],[255,51,51],[51,255,51],[51,51,255]],
    [[0,0,0],[30,30,30],[93,93,93],[128,128,128],[180,180,180],[225,225,225],[255,255,255],[68,93,118],[255,0,0],[0,255,0],[0,0,255]],
    [[25,25,25],[87,87,87],[150,150,150],[185,185,185],[237,237,237],[255,255,255],[255,255,255],[125,150,175],[255,25,25],[25,255,25],[25,25,255]],
    [[0,0,0],[0,0,0],[49,49,49],[77,77,77],[119,119,119],[155,155,155],[204,204,204],[29,49,69],[204,0,0],[0,204,0],[0,0,204]],
    [[25,25,25],[65,65,65],[105,105,105],[128,128,128],[161,161,161],[190,190,190],[229,229,229],[89,105,121],[229,25,25],[25,229,25],[25,25,229]],
    [[0,0,0],[0,0,0],[72,72,72],[128,128,128],[212,212,212],[255,255,255],[255,255,255],[64,64,64],[24,24,24],[170,170,170],[0,0,0]],
    [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[255,255,255],[0,0,0]],
  ],
  "hancom-color-probe-2024": [
    [[0,0,0],[50,50,50],[100,100,100],[128,128,128],[170,170,170],[206,206,206],[255,255,255],[80,100,120],[255,0,0],[0,255,0],[0,0,255]],
    [[20,20,20],[70,70,70],[120,120,120],[148,148,148],[190,190,190],[226,226,226],[255,255,255],[100,120,140],[255,20,20],[20,255,20],[20,20,255]],
    [[0,0,0],[43,43,43],[97,97,97],[128,128,128],[173,173,173],[212,212,212],[255,255,255],[76,97,119],[255,0,0],[0,255,0],[0,0,255]],
    [[51,51,51],[101,101,101],[151,151,151],[179,179,179],[221,221,221],[255,255,255],[255,255,255],[131,151,171],[255,51,51],[51,255,51],[51,51,255]],
    [[0,0,0],[30,30,30],[93,93,93],[128,128,128],[180,180,180],[225,225,225],[255,255,255],[68,93,118],[255,0,0],[0,255,0],[0,0,255]],
    [[25,25,25],[87,87,87],[150,150,150],[185,185,185],[237,237,237],[255,255,255],[255,255,255],[125,150,175],[255,25,25],[25,255,25],[25,25,255]],
    [[0,0,0],[0,0,0],[49,49,49],[77,77,77],[119,119,119],[155,155,155],[204,204,204],[29,49,69],[204,0,0],[0,204,0],[0,0,204]],
    [[25,25,25],[65,65,65],[105,105,105],[128,128,128],[161,161,161],[190,190,190],[229,229,229],[89,105,121],[229,25,25],[25,229,25],[25,25,229]],
    [[0,0,0],[0,0,0],[72,72,72],[128,128,128],[212,212,212],[255,255,255],[255,255,255],[64,64,64],[24,24,24],[170,170,170],[0,0,0]],
    [[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[255,255,255],[0,0,0]],
  ],
  "hancom-color-probe-cross-2024": [
    [[25,25,25],[50,50,50],[75,75,75],[89,89,89],[110,110,110],[128,128,128],[153,153,153],[65,75,85],[153,25,25],[25,153,25],[25,25,153]],
    [[102,102,102],[127,127,127],[152,152,152],[166,166,166],[187,187,187],[205,205,205],[229,229,229],[142,152,162],[229,102,102],[102,229,102],[102,102,229]],
    [[0,0,0],[19,19,19],[59,59,59],[82,82,82],[115,115,115],[144,144,144],[183,183,183],[43,59,75],[183,0,0],[0,183,0],[0,0,183]],
    [[71,71,71],[111,111,111],[151,151,151],[173,173,173],[207,207,207],[236,236,236],[255,255,255],[135,151,167],[255,71,71],[71,255,71],[71,71,255]],
    [[0,0,0],[0,0,0],[35,35,35],[70,70,70],[123,123,123],[168,168,168],[229,229,229],[10,35,60],[229,0,0],[0,229,0],[0,0,229]],
    [[25,25,25],[87,87,87],[150,150,150],[185,185,185],[237,237,237],[255,255,255],[255,255,255],[125,150,175],[255,25,25],[25,255,25],[25,25,255]],
    [[0,0,0],[0,0,0],[0,0,0],[51,51,51],[135,135,135],[207,207,207],[255,255,255],[0,0,35],[255,0,0],[0,255,0],[0,0,255]],
    [[0,0,0],[48,48,48],[148,148,148],[204,204,204],[255,255,255],[255,255,255],[255,255,255],[108,148,188],[255,0,0],[0,255,0],[0,0,255]],
    [[0,0,0],[0,0,0],[72,72,72],[128,128,128],[212,212,212],[255,255,255],[255,255,255],[64,64,64],[24,24,24],[170,170,170],[0,0,0]],
    [[0,0,0],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255]],
  ],
  "hancom-color-probe-boundary-2024": [
    [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
    [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
    [[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1],[1,1,1]],
    [[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52],[52,52,52]],
    [[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255]],
    [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
    [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[204,204,204],[204,204,204],[204,204,204],[0,0,0],[204,0,0],[0,204,0],[0,0,204]],
    [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[255,0,0],[0,255,0],[0,0,255]],
    [[51,51,51],[51,51,51],[51,51,51],[51,51,51],[255,255,255],[255,255,255],[255,255,255],[51,51,51],[255,51,51],[51,255,51],[51,51,255]],
    [[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255]],
  ],
};
controls["hancom-blackwhite-axis-probe-2024"] = [
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[255,255,255],[0,0,0]],
  [[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[255,255,255],[0,0,0]],
  [[0,0,0],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255]],
  [[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255],[255,255,255]],
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]],
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[255,255,255],[0,0,0]],
  [[0,0,0],[0,0,0],[0,0,0],[0,0,0],[255,255,255],[255,255,255],[255,255,255],[0,0,0],[0,0,0],[255,255,255],[0,0,0]],
];

for (const [name,expected] of Object.entries(controls)) {
test(`한컴 저장 대조군 ${name}의 실제 Canvas·DOM RGB는 독립 Print를 따른다`,async t=>{
 const pkg=resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG||join(root,'pkg'));
 const files=new Map([
 ['/rhwp.js',['application/javascript',readFileSync(join(pkg,'rhwp.js'))]],
 ['/rhwp_bg.wasm',['application/wasm',readFileSync(join(pkg,'rhwp_bg.wasm'))]],
 ['/input.hwp',['application/octet-stream',readFileSync(join(root,`tests/fixtures/issue_7751_color_controls/${name}.hwp`))]],
 ['/flow.js',['application/javascript',Buffer.from(stripTypeScriptTypes(
  readFileSync(join(root,'rhwp-studio/src/view/flow-image-clip.ts'),'utf8'),{mode:'strip'}))]],
 ]);
 const server=createSweepServer(files,new Map());
 await new Promise((a,r)=>{server.once('error',r);server.listen(0,'127.0.0.1',a)});
 t.after(async()=>{server.closeAllConnections();await new Promise(a=>server.close(a))});
 const profile=createSweepBrowserProfile(join(root,'output/pr-review/browser-profile-tests'),'rgb');let browser;
 t.after(async()=>{if(browser)await browser.close();rmSync(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100})});
 browser=await require('puppeteer-core').launch({executablePath:findChrome(process.env.VISUAL_SWEEP_CHROME),headless:true,userDataDir:profile,protocolTimeout:180000});
 const page=await browser.newPage();
 await page.setViewport({width:1200,height:1400,deviceScaleFactor:1});
 await page.goto(`http://127.0.0.1:${server.address().port}`);
 const result=await page.evaluate(async()=>{
  const wasm=await import('/rhwp.js');await wasm.default();
  const doc=new wasm.HwpDocument(new Uint8Array(await(await fetch('/input.hwp')).arrayBuffer()));
  const descriptor=Object.getOwnPropertyDescriptor(HTMLImageElement.prototype,'src'),pending=new Set(),errors=[];
  Object.defineProperty(HTMLImageElement.prototype,'src',{...descriptor,set(v){
   const p=new Promise(a=>{this.addEventListener('load',a,{once:true});this.addEventListener('error',()=>{errors.push('그림 decode 실패');a()},{once:true})});pending.add(p);p.finally(()=>pending.delete(p));descriptor.set.call(this,v);
  }});
  try{
   doc.setHangul2024Compat(true);
   const layer=JSON.parse(doc.getPageLayerTreeWithProfile(0,'print',true,true)),images=[];
   function collect(n){if(!n||typeof n!=='object')return;if(n.type==='image')images.push(n);for(const v of Object.values(n))if(v&&typeof v==='object')collect(v)}collect(layer);
   images.sort((a,b)=>a.bbox.y-b.bbox.y);
   const canvas=document.getElementById('page');
   for(let i=0;i<6;i++){doc.renderPageToCanvasFilteredWithProfile(0,canvas,1,'all','print');if(!pending.size)break;await Promise.all([...pending])}
   const ctx=canvas.getContext('2d');
   const rows=images.map(im=>{const {x,y,width:w,height:h}=im.bbox;return Array.from({length:11},(_,k)=>Array.from(ctx.getImageData(Math.floor(x+(k+.5)*w/11),Math.floor(y+h/2),1,1).data))});
   // 실제 Studio의 좁은 질의·source 바이트·CSS 조립 함수를 연결한다.
   // Canvas의 올바른 픽셀이 DOM의 올바른 색조를 대신 입증하지 못한다.
   const {flowImageOpsFromNarrowQuery}=await import('/flow.js');
   const ops=flowImageOpsFromNarrowQuery(doc.getPageFlowImageOps(0),(key,mime)=>
    URL.createObjectURL(new Blob([doc.getSourceImageBytes(key)],{type:mime})));
   if(!ops||ops.length!==images.length)throw Error('DOM 그림의 행 소속·누락·중복');
   ops.sort((a,b)=>a.bbox.y-b.bbox.y);
   document.body.replaceChildren();document.body.style.cssText='margin:0;background:white;position:relative';
   for(const op of ops){
    const img=new Image(),b=op.bbox;
    img.style.cssText=`position:absolute;left:${b.x}px;top:${b.y}px;width:${b.width}px;height:${b.height}px;filter:${op.filter||'none'}`;
    const ready=new Promise((accept,reject)=>{img.onload=accept;img.onerror=reject});
    img.src=op.src;document.body.append(img);await ready;
   }
   await new Promise(a=>requestAnimationFrame(()=>requestAnimationFrame(a)));
   return{errors,images:images.map(im=>({effect:im.effect,brightness:im.brightness,contrast:im.contrast,key:im.sourceImageKey})),rows,
    domPoints:ops.map(({bbox:b})=>Array.from({length:11},(_,k)=>[Math.floor(b.x+(k+.5)*b.width/11),Math.floor(b.y+b.height/2)]))};
  }finally{Object.defineProperty(HTMLImageElement.prototype,'src',descriptor);doc.free()}
 });
 assert.deepEqual(result.errors,[]);assert.equal(result.rows.length,expected.length,'행 소속·누락·중복');
 if (name === 'hancom-blackwhite-axis-probe-2024') {
  const axes=[[-100,0],[-50,0],[-20,0],[0,0],[10,0],[50,0],[100,0],[0,-100],[0,-50],[0,100]];
  for(let i=0;i<axes.length;i++) {
   const [brightness,contrast]=axes[i], op=result.images[i];
   if(brightness===0&&contrast===0) assert.ok(op.key.endsWith(':bwpng'),'무보정 흑백 source 계약');
   else {
    assert.ok(op.key.endsWith(`:pictone_${brightness}_${contrast}_2`),'저장된 축·효과를 구분하는 색조 신원');
    assert.equal(op.effect,'realPic');assert.equal(op.brightness,0);assert.equal(op.contrast,0);
   }
  }
 }

 const screenshot=Array.from(await page.screenshot({type:'png'}));
 const domRows=await page.evaluate(async({bytes,points})=>{
  const bitmap=await createImageBitmap(new Blob([new Uint8Array(bytes)],{type:'image/png'}));
  const canvas=document.createElement('canvas');canvas.width=bitmap.width;canvas.height=bitmap.height;
  const ctx=canvas.getContext('2d');ctx.drawImage(bitmap,0,0);bitmap.close();
  return points.map(row=>row.map(([x,y])=>Array.from(ctx.getImageData(x,y,1,1).data)));
 },{bytes:screenshot,points:result.domPoints});
 for(let i=0;i<expected.length;i++)for(let k=0;k<11;k++){
  assert.equal(result.rows[i][k][3],255);
  for(let c=0;c<3;c++)assert.ok(Math.abs(result.rows[i][k][c]-expected[i][k][c])<=1,`행${i} 패치${k}: actual=${result.rows[i][k]} Print=${expected[i][k]}`);
  assert.equal(domRows[i][k][3],255);
  for(let c=0;c<3;c++)assert.ok(Math.abs(domRows[i][k][c]-expected[i][k][c])<=1,`DOM 행${i} 패치${k}: actual=${domRows[i][k]} Print=${expected[i][k]}`);
 }
});
}
