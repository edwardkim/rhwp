import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {launchBrowser,createPage,loadApp,createNewDocument,closePage,closeBrowser,typeText} from '../../../../rhwp-studio/e2e/helpers.mjs';

const out='output/pr-review/pr8300/browser';fs.mkdirSync(out,{recursive:true});
const browser=await launchBrowser();const page=await createPage(browser,1300,1300);const results=[];
const settle=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
const command=id=>page.evaluate(id=>window.rhwpStudio.automation.execute(id,undefined,{allowDialog:true}),id);
async function newDoc(){await createNewDocument(page);await settle();}
async function openFull(){
 assert.equal((await command('table:create')).ok,true);
 await page.waitForSelector('[data-row="1"][data-col="1"]');
 await page.evaluate(()=>{const footer=[...document.querySelectorAll('div')].find(el=>el.style.cursor==='pointer'&&el.querySelector('span')?.textContent==='⊞'&&el.textContent.includes('표 만들기'));if(!footer)throw Error('table picker footer absent');footer.dataset.reviewFull='yes';});
 await page.click('[data-review-full="yes"]');await page.waitForSelector('#tc-treat-as-char');
 return page.$eval('#tc-treat-as-char',el=>({checked:el.checked,disabled:el.disabled,title:el.title}));
}
async function setup(rows,cols,cell,text=''){
 return page.evaluate(async({rows,cols,cell,text})=>{
  const wasm=window.__wasm,ih=window.__inputHandler;
  const r=wasm.createTableEx({sectionIdx:0,paraIdx:0,charOffset:0,rowCount:rows,colCount:cols,treatAsChar:true});if(!r.ok)throw Error(JSON.stringify(r));
  if(text)wasm.insertTextInCell(0,r.paraIdx,r.controlIdx,cell,0,0,text);
  const pos={sectionIndex:0,paragraphIndex:0,charOffset:text?2:0,parentParaIndex:r.paraIdx,controlIndex:r.controlIdx,cellIndex:cell,cellParaIndex:0,cellPath:[{controlIndex:r.controlIdx,cellIndex:cell,cellParaIndex:0}]};
  await window.__canvasView.loadDocument();ih.cursor.moveTo(pos);ih.history.clear(wasm);
  return{para:r.paraIdx,control:r.controlIdx,pos};
 },{rows,cols,cell,text});
}
const snapshot=()=>page.evaluate(()=>{
 const w=window.__wasm,ih=window.__inputHandler,pos=ih.getCursorPosition();let inner=null;
 if(pos.cellPath?.length>1)inner=w.getTableDimensionsByPath(0,pos.parentParaIndex,JSON.stringify(pos.cellPath));
 return{position:pos,inner,info:w.getDocumentInfo(),undoDepth:ih.history.undoStack.length,canUndo:window.rhwpStudio.automation.getContext().canUndo};
});
async function save(key){
 const data=await page.evaluate(()=>({hwp:[...window.__wasm.exportHwp()],hwpx:[...window.__wasm.exportHwpx()]}));
 const row={key};for(const ext of ['hwp','hwpx']){const b=Buffer.from(data[ext]);fs.writeFileSync(path.join(out,key+'.'+ext),b);row[ext]={bytes:b.length,sha256:createHash('sha256').update(b).digest('hex')};}return row;
}
try{
 await loadApp(page,'?renderer=canvas2d');
 await page.evaluate(()=>{const b=[...document.querySelectorAll('button.dialog-btn-primary')].find(b=>b.offsetParent!==null&&b.textContent.trim()==='시작하기');if(b){b.dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));b.click();}});
 await newDoc();const bodyDialog=await openFull();assert.equal(bodyDialog.checked,false);assert.equal(bodyDialog.disabled,false);await page.click('.dialog-close');results.push({case:'body-dialog-default',...bodyDialog});
 for(const [key,rows,cols,cell,text]of [['nested-hwp',3,3,4,''],['nested-mid',2,2,0,'가나다라']]){
  await newDoc();const base=await setup(rows,cols,cell,text);const before=await snapshot();
  const dialog=await openFull();assert.equal(dialog.checked,true);assert.equal(dialog.disabled,true);assert.ok(dialog.title);
  await page.screenshot({path:path.join(out,key+'-dialog.png'),fullPage:false});
  await page.evaluate(()=>{const inputs=[...document.querySelectorAll('.modal-overlay input[type="number"]')];inputs[0].value='2';inputs[1].value='2';for(const el of inputs.slice(0,2))el.dispatchEvent(new Event('change',{bubbles:true}));});
  await page.click('.modal-overlay .dialog-btn-primary');await settle();const after=await snapshot();
  assert.equal(after.position.cellPath.length,2);assert.equal(after.inner.rowCount,2);assert.equal(after.inner.colCount,2);assert.equal(after.undoDepth,before.undoDepth+1);
  const outer=await page.evaluate(({para,control})=>window.__wasm.getTableDimensions(0,para,control),base);assert.equal(outer.rowCount,rows);assert.equal(outer.colCount,cols);
  const saved=await save(key);await page.screenshot({path:path.join(out,key+'-after.png'),fullPage:false});
  assert.equal((await command('edit:undo')).ok,true);await settle();const undo=await snapshot();assert.equal((undo.position.cellPath?.length??(undo.position.parentParaIndex!==undefined?1:0)),1);
  assert.equal((await command('edit:redo')).ok,true);await settle();const redo=await snapshot();assert.equal(redo.position.cellPath.length,2);assert.deepEqual(redo.inner,after.inner);
  const invalid=await page.evaluate(({para,pos})=>{const w=window.__wasm,before=[...w.exportHwp()];let error;try{w.createTableInCellByPath(0,para,pos.cellPath,0,{rowCount:2,colCount:2,treatAsChar:false});}catch(e){error=String(e);}const after=[...w.exportHwp()];return{error,unchanged:before.length===after.length&&before.every((v,i)=>v===after[i])};},{para:base.para,pos:base.pos});assert.ok(invalid.error);assert.equal(invalid.unchanged,true);
  if(key==='nested-hwp'){
   assert.equal((await command('table:create')).ok,true);await page.waitForSelector('[data-row="1"][data-col="1"]');await page.click('[data-row="1"][data-col="1"]');await settle();const deep=await snapshot();assert.equal(deep.position.cellPath.length,3);assert.equal(deep.inner.rowCount,2);assert.equal((await command('edit:undo')).ok,true);await settle();assert.equal((await snapshot()).position.cellPath.length,2);assert.equal((await command('edit:redo')).ok,true);await settle();assert.equal((await snapshot()).position.cellPath.length,3);results.push({case:'depth-two-and-repeated-history',deep});
  }
  await page.evaluate(()=>window.__inputHandler.focus());await typeText(page,'중첩');await settle();
  const typed=await page.evaluate(()=>{const pos=window.__inputHandler.getCursorPosition();return{position:pos,text:window.__wasm.getTextInCellByPath(0,pos.parentParaIndex,JSON.stringify(pos.cellPath),0,2)};});assert.equal(typed.text,'중첩');
  results.push({case:key,dialog,before,after,outer,undo,redo,invalid,saved,typed});fs.writeFileSync(path.join(out,'partial.json'),JSON.stringify(results,null,2));
 }
 fs.writeFileSync(path.join(out,'result.json'),JSON.stringify({scope:'Actual Studio automation command router, real picker/detail DOM clicks, history Undo/Redo, new nested depth, WASM rejection and actual saved HWP/HWPX. This is an execution probe; visual comparison is measured separately.',results},null,2));console.log('Actual Studio nested-table create, dialogs, history and saves passed');
}finally{await closePage(page);await closeBrowser(browser);}
