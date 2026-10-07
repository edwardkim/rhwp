import {pathToFileURL} from 'node:url';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const {chromium}=await import(pathToFileURL(process.cwd()+'/output/pr-review/semanticist21-20261005/browser/node_modules/playwright-core/index.mjs').href);
const out='output/pr-review/semanticist21-20261005/combobox-screen-final';
await fs.mkdir(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/snap/bin/chromium',headless:true,args:['--no-sandbox']});
try {
 const page=await browser.newPage({viewport:{width:900,height:1200}});
 const fontSvg=await fs.readFile('output/pr-review/semanticist21-20261005/appearance-final-native/form-original/svg/form-01.svg','utf8');
 const fontMatch=fontSvg.match(/@font-face \{ font-family: \"한컴바탕\"; src: url\(\"data:font\/opentype;base64,([A-Za-z0-9+/=]+)/);
 assert.ok(fontMatch,'Native full font embedding must precede browser check');
 const fontBytes=Buffer.from(fontMatch[1],'base64');
 await page.route('**/__review/hancom-form-font.ttf',route=>route.fulfill({body:fontBytes,contentType:'font/ttf'}));
 await page.goto('http://127.0.0.1:18765/');
 const results=[];
 for(const file of ['samples/hwpx/form-01.hwpx','tests/fixtures/form-password/edit-password.hwpx']) {
  const result=await page.evaluate(async(file)=>{
   const module=await import('/pkg/rhwp.js');await module.default();
   const font=new FontFace('한컴바탕',await(await fetch('/__review/hancom-form-font.ttf')).arrayBuffer());
   await font.load();document.fonts.add(font);await document.fonts.ready;
   const d=new module.HwpDocument(new Uint8Array(await(await fetch('/'+file)).arrayBuffer()));
   document.body.innerHTML='<canvas id="canvas" width="820" height="1150"></canvas>';
   const calls=[];const original=CanvasRenderingContext2D.prototype.fillText;
   CanvasRenderingContext2D.prototype.fillText=function(text,...args){calls.push({text,args,font:this.font});return original.call(this,text,...args)};
   try {d.renderPageToCanvas(0,document.querySelector('canvas'),1);}finally{CanvasRenderingContext2D.prototype.fillText=original;}
   const raw=JSON.parse(d.getFormValue(0,4,0));
   const forms=[];
   function visit(n){if(n.ops)forms.push(...n.ops.filter(op=>op.type==='formObject'));for(const child of n.children||[])visit(child);if(n.child)visit(n.child);}
   visit(JSON.parse(d.getPageLayerTree(0)).root);
   const comboBounds=forms.find(f=>f.formType==='comboBox').bbox;
   const svg=d.renderPageSvg(0);
   const reopened=new module.HwpDocument(d.exportHwpx());
   return {file,calls,raw,svg,comboBounds,reopenedRaw:JSON.parse(reopened.getFormValue(0,4,0)),reopenedSvg:reopened.renderPageSvg(0)};
  },file);
  assert.equal(result.raw.text,'');assert.equal(result.reopenedRaw.text,'');
  const title=result.calls.find(c=>c.text==='계절 선택');assert.ok(title);
  const box=result.comboBounds;
  assert.ok(title.args[0]>box.x&&title.args[0]<box.x+box.width);
  assert.ok(title.args[1]>box.y&&title.args[1]<box.y+box.height);
  assert.ok(result.svg.includes('계절 선택'));assert.ok(result.reopenedSvg.includes('계절 선택'));
  if(file.includes('password')) {assert.ok(result.calls.some(c=>c.text==='*************'));assert.ok(!result.calls.some(c=>c.text.includes('MASK_SENTINEL')));}
  const key=file.includes('password')?'password':'original';
  await page.locator('canvas').screenshot({path:`${out}/${key}.png`});
  await fs.writeFile(`${out}/${key}.svg`,result.svg);
  results.push(result);
 }
 await page.route('http://127.0.0.1:18765/**',async route=>route.fulfill({body:await fs.readFile(new URL(route.request().url()).pathname.slice(1)),contentType:'application/octet-stream',headers:{'access-control-allow-origin':'*'}}));
 await page.goto('http://127.0.0.1:18766/');
 const kitResults=[];
 for(const result of results) {
  const kit=await page.evaluate(async(svg)=>{
   const wasm=await import('/rhwp.js');await wasm.default();
   const source=await(await fetch('http://127.0.0.1:18765/'+svg.file)).arrayBuffer();
   const d=new wasm.HwpDocument(new Uint8Array(source));
   const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');
   const renderer=await CanvasKitLayerRenderer.create('default','software',{defaultFontUrl:'/__review/hancom-form-font.ttf'});
   await renderer.prepareBundledFonts([{url:'/__review/hancom-form-font.ttf',aliases:['한컴바탕','Haansoft Batang']}]);
   const tree=JSON.parse(d.getPageLayerTree(0));
   document.body.innerHTML='<canvas id="canvas" width="794" height="1123"></canvas>';
   const canvas=document.querySelector('canvas');renderer.renderPage(tree,canvas,1);
   const diagnostic=renderer.diagnostics();
   const forms=[];
   function visit(n){if(n.ops)forms.push(...n.ops.filter(op=>op.type==='formObject'));for(const child of n.children||[])visit(child);if(n.child)visit(n.child);}
   visit(tree.root);renderer.dispose();
   return {diagnostic,forms};
  },result);
  assert.ok(kit.forms.find(f=>f.formType==='comboBox').drawing.label.text==='계절 선택');
  assert.equal(kit.diagnostic.lastRenderCompleted,true);assert.equal(kit.diagnostic.lastRenderError,null);
  assert.equal(kit.diagnostic.unregisteredFontFallbacks,0);
  await page.locator('canvas').screenshot({path:`${out}/canvaskit-${result.file.includes('password')?'password':'original'}.png`});
  kitResults.push(kit);
 }
 await fs.writeFile(`${out}/results.json`,JSON.stringify({webCanvas:results,canvasKit:kitResults},null,2));
 console.log('PASS: both ComboBox Canvas/SVG/HWPX reopen, raw selection retained, password masked');
}finally{await browser.close()}
