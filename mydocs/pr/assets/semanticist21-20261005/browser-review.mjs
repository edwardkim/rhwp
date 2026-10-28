import {pathToFileURL} from 'node:url';
const {chromium} = await import(pathToFileURL(process.cwd() + '/output/pr-review/semanticist21-20261005/browser/node_modules/playwright-core/index.mjs').href);
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const out = path.resolve('output/pr-review/semanticist21-20261005/screen');
await fs.mkdir(out,{recursive:true});
const browser = await chromium.launch({executablePath:'/snap/bin/chromium',headless:true,args:['--no-sandbox']});
try {
  const page = await browser.newPage({viewport:{width:1100,height:1250},deviceScaleFactor:1});
  await page.goto('http://127.0.0.1:18765/');
  await page.evaluate(async()=>{
    const module = await import('/pkg/rhwp.js');
    await module.default(); window.RHWP = module;
    document.body.innerHTML='<canvas id="canvas" width="820" height="1150"></canvas>';
    window.loadDoc=async(file)=>new module.HwpDocument(new Uint8Array(await(await fetch('/'+file)).arrayBuffer()));
    window.draw=(doc)=>{
      const canvas=document.querySelector('canvas');
      const layout=JSON.parse(doc.getPageLayerTree(0));
      canvas.width=Math.ceil(layout.width||820); canvas.height=Math.ceil(layout.height||1150);
      doc.renderPageToCanvas(0,canvas,1);
    };
  });
  const mask = await page.evaluate(async()=>{
    const doc=await loadDoc('tests/fixtures/form-password/edit-password.hwpx');
    const calls=[];
    const original=CanvasRenderingContext2D.prototype.fillText;
    CanvasRenderingContext2D.prototype.fillText=function(text,...args){calls.push(text);return original.call(this,text,...args);};
    draw(doc);
    CanvasRenderingContext2D.prototype.fillText=original;
    return {calls,svg:doc.renderPageSvg(0),value:JSON.parse(doc.getFormValue(0,8,0)),layer:JSON.parse(doc.getPageLayerTree(0))};
  });
  assert.equal(mask.value.text,'MASK_SENTINEL');
  assert.ok(mask.calls.includes('*************'));
  assert.ok(!mask.calls.some(x=>x.includes('MASK_SENTINEL')));
  assert.ok(!mask.svg.includes('MASK_SENTINEL'));
  await fs.writeFile(path.join(out,'password.svg'),mask.svg);
  await page.locator('canvas').screenshot({path:path.join(out,'password-canvas.png')});
  const guides=[];
  for (const state of ['inactive','first','second','clear']) {
    const result=await page.evaluate(async(state)=>{
      window.guideDoc??=await loadDoc('tests/fixtures/clickhere_cell_parent_identity/two_tables.hwp');
      const d=window.guideDoc;
      if(state==='first') d.setActiveFieldInCell(0,1,0,0,0,2,false);
      if(state==='second') d.setActiveFieldInCell(0,4,0,0,0,2,false);
      if(state==='clear') d.clearActiveField();
      draw(d);
      return {svg:d.renderPageSvg(0),layout:JSON.parse(d.getPageTextLayout(0))};
    },state);
    guides.push({state,...result});
    await fs.writeFile(path.join(out,`guides-${state}.svg`),result.svg);
    await page.locator('canvas').screenshot({path:path.join(out,`guides-${state}.png`)});
  }
  assert.equal(guides[0].svg,guides[3].svg);
  assert.notEqual(guides[0].svg,guides[1].svg);
  assert.notEqual(guides[1].svg,guides[2].svg);
  const editedHost=await page.evaluate(async()=>{
    const d=await loadDoc('samples/issue6190/center_align_first_line_indent.hwp');
    const before=d.renderPageSvg(0);
    d.insertText(0,4,7,'가');d.insertText(0,7,0,'가');
    draw(d);
    return {before,svg:d.renderPageSvg(0),tree:JSON.parse(d.getPageRenderTree(0)),bytes:Array.from(d.exportHwp())};
  });
  await fs.writeFile(path.join(out,'issue6190-before.svg'),editedHost.before);
  await fs.writeFile(path.join(out,'issue6190-edited.svg'),editedHost.svg);
  await fs.writeFile(path.join(out,'issue6190-edited.hwp'),Buffer.from(editedHost.bytes));
  await page.locator('canvas').screenshot({path:path.join(out,'issue6190-edited.png')});
  const indent=[];
  for(const value of [0,3000,-3000]) {
    const result=await page.evaluate(value=>{
      const d=RHWP.HwpDocument.createEmpty();d.createBlankDocument();
      d.insertText(0,0,0,'1) 가나다라마바사아자차카타파하 '.repeat(8));
      d.applyParaFormat(0,0,JSON.stringify({indent:value}));
      draw(d);
      return {svg:d.renderPageSvg(0),bytes:Array.from(d.exportHwp()),layout:JSON.parse(d.getPageTextLayout(0))};
    },value);
    const key=value===0?'flat':value>0?'indent':'hanging';
    await fs.writeFile(path.join(out,`${key}.hwp`),Buffer.from(result.bytes));
    await fs.writeFile(path.join(out,`${key}.svg`),result.svg);
    await page.locator('canvas').screenshot({path:path.join(out,`${key}.png`)});
    indent.push({value,layout:result.layout});
  }
  const longHTML=await page.evaluate(()=>{
    const a=RHWP.HwpDocument.createEmpty();a.createBlankDocument();
    a.pasteHtml(0,0,0,'x'.repeat(8001));
    const b=RHWP.HwpDocument.createEmpty();b.createBlankDocument();
    b.pasteHtml(0,0,0,'<b>'+'x'.repeat(8001)+'</b>');
    return {plain:{count:a.getParagraphCount(0),max:Math.max(...Array.from({length:a.getParagraphCount(0)},(_,i)=>a.getParagraphLength(0,i)))},bold:{count:b.getParagraphCount(0),max:Math.max(...Array.from({length:b.getParagraphCount(0)},(_,i)=>b.getParagraphLength(0,i)))}};
  });
  await fs.writeFile(path.join(out,'browser-results.json'),JSON.stringify({mask,guides,editedHost,indent,longHTML},null,2));
  console.log(JSON.stringify({password:'PASS',guides:'PASS',indentFixtures:3,longHTML}));
} finally {await browser.close();}
