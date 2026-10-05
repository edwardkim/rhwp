import {pathToFileURL} from 'node:url';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const {chromium}=await import(pathToFileURL(process.cwd()+'/output/pr-review/semanticist21-20261005/browser/node_modules/playwright-core/index.mjs').href);
const out='output/pr-review/semanticist21-20261005/combobox-screen';
await fs.mkdir(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/snap/bin/chromium',headless:true,args:['--no-sandbox']});
try {
 const page=await browser.newPage({viewport:{width:900,height:1200}});
 await page.goto('http://127.0.0.1:18765/');
 const results=[];
 for(const file of ['samples/hwpx/form-01.hwpx','tests/fixtures/form-password/edit-password.hwpx']) {
  const result=await page.evaluate(async(file)=>{
   const module=await import('/pkg/rhwp.js');await module.default();
   const d=new module.HwpDocument(new Uint8Array(await(await fetch('/'+file)).arrayBuffer()));
   document.body.innerHTML='<canvas id="canvas" width="820" height="1150"></canvas>';
   const calls=[];const original=CanvasRenderingContext2D.prototype.fillText;
   CanvasRenderingContext2D.prototype.fillText=function(text,...args){calls.push({text,args});return original.call(this,text,...args)};
   try {d.renderPageToCanvas(0,document.querySelector('canvas'),1);}finally{CanvasRenderingContext2D.prototype.fillText=original;}
   const raw=JSON.parse(d.getFormValue(0,4,0));
   const svg=d.renderPageSvg(0);
   const reopened=new module.HwpDocument(d.exportHwpx());
   return {file,calls,raw,svg,reopenedRaw:JSON.parse(reopened.getFormValue(0,4,0)),reopenedSvg:reopened.renderPageSvg(0)};
  },file);
  assert.equal(result.raw.text,'');assert.equal(result.reopenedRaw.text,'');
  assert.ok(result.calls.some(c=>c.text==='계절 선택'));
  assert.ok(result.svg.includes('계절 선택'));assert.ok(result.reopenedSvg.includes('계절 선택'));
  if(file.includes('password')) {assert.ok(result.calls.some(c=>c.text==='*************'));assert.ok(!result.calls.some(c=>c.text.includes('MASK_SENTINEL')));}
  const key=file.includes('password')?'password':'original';
  await page.locator('canvas').screenshot({path:`${out}/${key}.png`});
  await fs.writeFile(`${out}/${key}.svg`,result.svg);
  results.push(result);
 }
 await fs.writeFile(`${out}/results.json`,JSON.stringify(results,null,2));
 console.log('PASS: both ComboBox Canvas/SVG/HWPX reopen, raw selection retained, password masked');
}finally{await browser.close()}
