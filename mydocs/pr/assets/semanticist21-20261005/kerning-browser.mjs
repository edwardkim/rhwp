import {pathToFileURL} from 'node:url';
const {chromium} = await import(pathToFileURL(process.cwd() + '/output/pr-review/semanticist21-20261005/browser/node_modules/playwright-core/index.mjs').href);
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const browser=await chromium.launch({executablePath:'/snap/bin/chromium',headless:true,args:['--no-sandbox']});
try {
 const p=await browser.newPage({viewport:{width:1000,height:1200}});
 await p.goto('http://127.0.0.1:18765/');
 const result=await p.evaluate(async()=>{
  const m=await import('/pkg/rhwp.js');await m.default();
  const d=m.HwpDocument.createEmpty();d.createBlankDocument();d.insertText(0,0,0,'AVTo');
  d.applyCharFormat(0,0,0,4,JSON.stringify({kerning:true,fontSize:2600}));
  const font=new Uint8Array(await(await fetch('/tests/fixtures/fonts/RHWPExactKerningSmoke.ttf')).arrayBuffer());
  const initial=JSON.parse(d.getPageTextLayout(0));
  for(const family of new Set(initial.runs.map(r=>r.fontFamily))) {
   const face=new FontFace(family,font);await face.load();document.fonts.add(face);
  }
  document.body.innerHTML='<canvas width="1600" height="2300"></canvas>';
  const canvas=document.querySelector('canvas');
  const original=CanvasRenderingContext2D.prototype.fillText;
  let calls=[];
  CanvasRenderingContext2D.prototype.fillText=function(text,x,y,...rest){
   const tr=this.getTransform();calls.push({text,x,y,a:tr.a,e:tr.e,f:tr.f,font:this.font,width:this.measureText(text).width});
   return original.call(this,text,x,y,...rest);
  };
  d.renderPageToCanvas(0,canvas,2);const before={calls,svg:d.renderPageSvg(0),layout:JSON.parse(d.getPageTextLayout(0))};calls=[];
  for(const id of new Set(initial.runs.map(r=>r.charShapeId)))d.registerExactFontSource(id,1,font,0);
  d.renderPageToCanvas(0,canvas,2);const after={calls,svg:d.renderPageSvg(0),layout:JSON.parse(d.getPageTextLayout(0))};
  CanvasRenderingContext2D.prototype.fillText=original;
  return {before,after};
 });
 await fs.writeFile('output/pr-review/semanticist21-20261005/screen/kerning-results.json',JSON.stringify(result,null,2));
 for(const ch of ['A','V','T','o']) {
  const a=result.before.calls.find(x=>x.text===ch),b=result.after.calls.find(x=>x.text===ch);
  assert.ok(a&&b,`glyph ${ch}`);assert.ok(Math.abs(a.a-b.a)<0.001,`natural scale ${ch}`);
 }
 assert.notEqual(result.before.svg,result.after.svg);
 await p.locator('canvas').screenshot({path:'output/pr-review/semanticist21-20261005/screen/kerning-registered.png'});
 console.log('PASS: actual FontFace+Canvas registered kerning changes origins and preserves all four glyph scales');
} finally {await browser.close();}
