import {pathToFileURL} from 'node:url';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const {chromium}=await import(pathToFileURL(process.cwd()+'/output/pr-review/semanticist21-20261005/browser/node_modules/playwright-core/index.mjs').href);
const browser=await chromium.launch({executablePath:'/snap/bin/chromium',headless:true,args:['--no-sandbox']});
try {
 const p=await browser.newPage();await p.goto('http://127.0.0.1:18765/');
 const result=await p.evaluate(async()=>{
  const m=await import('/pkg/rhwp.js');await m.default();
  const blank=()=>{const d=m.HwpDocument.createEmpty();d.createBlankDocument();return d;};
  function tableBBox(d) {
   const collect=n=>n.type==='Table'?n.bbox:(n.children||[]).map(collect).find(Boolean);
   return collect(JSON.parse(d.getPageRenderTree(0)));
  }
  const t=blank();const created=JSON.parse(t.createTable(0,0,0,3,2));
  t.resizeTableCells(0,created.paraIdx,created.controlIdx,JSON.stringify([{cellIdx:2,widthDelta:900},{cellIdx:3,widthDelta:-900}]));
  const widthBefore=tableBBox(t);
  t.setCellProperties(0,created.paraIdx,created.controlIdx,0,JSON.stringify({height:3000}));
  const widthAfter=tableBBox(t);
  const widthReopened=tableBBox(new m.HwpDocument(t.exportHwp()));
  const note=blank();note.insertText(0,0,0,'본문 내용');
  const ins=JSON.parse(note.insertFootnote(0,0,2));const text='첫째 각주🦦';
  note.insertTextInFootnote(0,0,ins.controlIdx,0,2,text);
  const caret=[];
  for(let i=2;i<=2+Array.from(text).length;i++)caret.push(JSON.parse(note.getCursorRectInNote(0,0,ins.controlIdx,0,i)));
  const empty=blank();empty.applyCharFormat(0,0,0,0,JSON.stringify({bold:true,fontSize:2000}));
  empty.insertText(0,0,0,'A');const format=JSON.parse(empty.getPageTextLayout(0));
  const selection=blank();selection.insertText(0,0,0,'abcdef');selection.renderPageSvg(0);
  const selectionBefore=JSON.parse(selection.getSelectionRects(0,0,1,0,4));
  selection.insertText(0,0,0,'앞');
  const selectionAfter=JSON.parse(selection.getSelectionRects(0,0,1,0,4));
  return {widthBefore,widthAfter,widthReopened,caret,format,selectionBefore,selectionAfter};
 });
 assert.equal(result.widthBefore.w,result.widthAfter.w);
 assert.equal(result.widthBefore.x,result.widthAfter.x);
 assert.equal(result.widthAfter.w,result.widthReopened.w);
 assert.ok(result.caret.slice(1).every((r,i)=>r.x>result.caret[i].x));
 assert.ok(result.format.runs.some(r=>r.text==='A'&&r.bold&&Math.abs(r.fontSize-26.7)<0.2));
 assert.notDeepEqual(result.selectionBefore,result.selectionAfter);
 await fs.writeFile('mydocs/pr/assets/semanticist21-20261005/query-geometry-results.json',JSON.stringify(result,null,2));
 console.log('PASS: table rendered width/origin + HWP reopen, final note caret, empty formatting, warm selection edit invalidation');
} finally {await browser.close();}
