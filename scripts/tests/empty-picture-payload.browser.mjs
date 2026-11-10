// 관계형: 빈 그림 payload는 브라우저 decode 대상이 아니며, 비어 있지 않은 손상 payload는 여전히 오류로 관측한다.
// 실물 배치 좌표·전체 이미지 해시는 고정하지 않는다. 설치 글꼴을 사용하지 않는다.
// 주석 갱신: 2026-11-10T09:00:00+09:00.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync,rmSync} from 'node:fs';
import {join,resolve} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {createSweepBrowserProfile,createSweepServer} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';
const root=resolve(fileURLToPath(new URL('../..',import.meta.url)));
const require=createRequire(join(root,'rhwp-studio/package.json'));
for(const corrupt of [false,true])test(`원본 빈 그림과 손상 그림의 decode 구분: corrupt=${corrupt}`,async t=>{
 const pkg=resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG||join(root,'pkg'));
 const files=new Map([
 ['/rhwp.js',['application/javascript',readFileSync(join(pkg,'rhwp.js'))]],
 ['/rhwp_bg.wasm',['application/wasm',readFileSync(join(pkg,'rhwp_bg.wasm'))]],
 ['/source',['application/octet-stream',readFileSync(join(root,'samples/hwp3-sample10.hwp'))]],
 ]);
 const server=createSweepServer(files,new Map());await new Promise((a,r)=>{server.once('error',r);server.listen(0,'127.0.0.1',a)});
 t.after(async()=>{server.closeAllConnections();await new Promise(a=>server.close(a))});
 const profile=createSweepBrowserProfile(join(root,'output/pr-review/browser-profile-tests'),'empty-picture');let browser;
 t.after(async()=>{if(browser)await browser.close();rmSync(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100})});
 browser=await require('puppeteer-core').launch({executablePath:findChrome(process.env.VISUAL_SWEEP_CHROME),headless:true,userDataDir:profile,protocolTimeout:180000});
 const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}`);
 const result=await page.evaluate(async corrupt=>{
  const w=await import('/rhwp.js');await w.default();const doc=corrupt?w.HwpDocument.createEmpty():new w.HwpDocument(new Uint8Array(await(await fetch('/source')).arrayBuffer()));
  const original=Object.getOwnPropertyDescriptor(HTMLImageElement.prototype,'src');const urls=[],pending=new Set(),errors=[];
  Object.defineProperty(HTMLImageElement.prototype,'src',{...original,set(value){urls.push(value);const job=new Promise(a=>{this.addEventListener('load',a,{once:true});this.addEventListener('error',()=>{errors.push(value);a()},{once:true})});pending.add(job);job.finally(()=>pending.delete(job));return original.set.call(this,value)}});
  try{
   if(corrupt)doc.insertPicture(0,0,0,'[]',new Uint8Array([137,80,78,71,13,10,26,10,0,0,0,0]),1000,1000,1,1,'png','nonempty corrupt control');
   const before=JSON.parse(doc.getPageRenderTree(0));const canvas=document.getElementById('page');doc.renderPageToCanvasFilteredWithProfile(0,canvas,1,'all','print');await Promise.all([...pending]);
   const after=JSON.parse(doc.getPageRenderTree(0));return {urls,errors,before,after,pages:doc.pageCount()};
  }finally{doc.free()}
 },corrupt);
 assert.deepEqual(result.after,result.before,'decode 준비 여부가 원본 트리·그림 칸 소속을 변경하지 않는다');
 assert.equal(result.pages,corrupt?1:763,'원본 전체 쪽수·내용 칸을 유지한다');
 const empty=result.urls.filter(x=>x==='data:application/octet-stream;base64,');
 if(!corrupt){assert.equal(empty.length,0,'빈 payload로 HtmlImageElement decode를 요청하지 않는다');assert.equal(result.errors.length,0,'없는 그림 데이터가 실제 exporter 오류를 만들지 않는다');}
 else{assert.ok(result.urls.some(x=>x.startsWith('data:image/png;base64,') && x.length>'data:image/png;base64,'.length),'비어 있지 않은 손상 데이터는 paint 시도를 유지한다');assert.ok(result.errors.some(x=>x.startsWith('data:image/png;base64,') && x.length>'data:image/png;base64,'.length),'실제 손상 그림은 decode 오류로 관측된다');}
});
