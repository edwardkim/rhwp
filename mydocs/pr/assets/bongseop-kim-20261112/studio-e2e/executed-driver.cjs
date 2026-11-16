const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const puppeteer = require('/home/edward/mygithub/rhwp/rhwp-studio/node_modules/puppeteer-core');
const O='/home/edward/mygithub/rhwp/output/pr-review/bongseop-kim-20261112';
(async()=>{
 const browser=await puppeteer.connect({browserURL:'http://127.0.0.1:19222'});const page=await browser.newPage();
 const errors=[],requests=[],warnings=[];page.on('console',m=>{if(m.type()==='warn'||m.type()==='error')warnings.push(m.text().slice(0,800));});page.on('pageerror',e=>errors.push(e.message));const {observeWasmArtifact}=await import('file:///home/edward/mygithub/rhwp-worktrees/review-integration-8942-bongseop-html-clipboard/rhwp-studio/e2e/wasm-artifact-check.mjs');const observation=await observeWasmArtifact(page,O+'/fresh-wasm-manifest.json');
 const result={browser:await browser.version(),transport:'CDP 19222',pasteEntry:'real Studio textarea ClipboardEvent with text/html and text/plain; synthetic event, no clipboard handler mock',cases:[],requests,errors};
 try{
  await page.setViewport({width:1280,height:950,deviceScaleFactor:1});await page.goto('http://localhost:7894/?pr-review=bongseop-20261112',{waitUntil:'domcontentloaded',timeout:60000});await page.waitForFunction(()=>!!window.__wasm&&!!window.__canvasView&&!!window.__inputHandler,{timeout:90000});
  const reset=async()=>{await page.evaluate(()=>window.__eventBus.emit('create-new-document',{skipUnsavedGuard:true}));await page.waitForFunction(()=>window.__wasm.getParagraphCount(0)===1&&window.__wasm.getParagraphLength(0,0)===0);await new Promise(r=>setTimeout(r,1200));await page.evaluate(()=>window.__canvasView.loadDocument());};
  const snapshot=async()=>page.evaluate(()=>{
   const w=window.__wasm,body=[],tables=[];
   for(let p=0;p<w.getParagraphCount(0);p++){
    const text=w.getTextRange(0,p,0,w.getParagraphLength(0,p));const marks=Array.from(text,(_,i)=>{const c=w.getCharPropertiesAt(0,p,i);return c.bold?'B':c.italic?'I':c.underline?'U':'.';}).join('');if(text)body.push({text,marks});
    for(let ci=0;ci<6;ci++){try{const info=w.getCellInfo(0,p,ci,0);if(info&&typeof info==='object'&&('row' in info||'rowIndex'in info)){const props=w.getCellProperties(0,p,ci,0);const count=w.getCellParagraphCount(0,p,ci,0);const cell=[];for(let q=0;q<count;q++){const t=w.getTextInCell(0,p,ci,0,q,0,w.getCellParagraphLength(0,p,ci,0,q));const m=Array.from(t,(_,i)=>{const c=w.getCellCharPropertiesAt(0,p,ci,0,q,i);return c.bold?'B':c.italic?'I':c.underline?'U':'.';}).join('');cell.push({text:t,marks:m});}tables.push({paragraph:p,cell, borders:[props.borderLeft,props.borderRight,props.borderTop,props.borderBottom]});}}catch{}}
   }return {body,tables};
  });
  const paste=async(html,text)=>{await page.evaluate(({html,text})=>{const input=window.__inputHandler;input.focus();const data=new DataTransfer();data.setData('text/html',html);data.setData('text/plain',text);input.textarea.dispatchEvent(new ClipboardEvent('paste',{clipboardData:data,bubbles:true,cancelable:true}));},{html,text});await page.waitForFunction(needle=>{const w=window.__wasm;for(let p=0;p<w.getParagraphCount(0);p++){if(w.getTextRange(0,p,0,w.getParagraphLength(0,p)).includes(needle))return true;for(let ci=0;ci<6;ci++){try{if(w.getTextInCell(0,p,ci,0,0,0,w.getCellParagraphLength(0,p,ci,0,0)).includes(needle))return true;}catch{}}}return false;},{timeout:15000},text.split(/\n/)[0]).catch(async error=>{result.failedPaste={snapshot:await snapshot(),cursor:await page.evaluate(()=>window.__inputHandler.cursor.getPosition()),warnings};throw error;});await new Promise(r=>setTimeout(r,350));};
  for(const [name,html,text] of [
   ['leading','<p>첫 문단</p><p>앞 <b>굵게</b> 뒤</p>','첫 문단\n앞 굵게 뒤'],
   ['border','<p>BEFORE</p><table border="1"><tr><td>가</td><td>나</td></tr></table><p>AFTER</p>','BEFORE\n가\t나\nAFTER'],
   ['combined','<p>첫 문단</p><p>앞 <b>굵게</b> 뒤</p><table border="1"><tr><td>셀 앞 <b>굵게</b> 뒤</td><td>일반 셀</td></tr></table><p>AFTER</p>','첫 문단\n앞 굵게 뒤\n셀 앞 굵게 뒤\t일반 셀\nAFTER'],
   ['body-styled-caret','<p>앞 <b>굵게</b> 뒤</p>','앞 굵게 뒤'],
   ['cell-styled-caret','<p>앞 <b>굵게</b> 뒤</p>','앞 굵게 뒤'],
   ['attribute-alias','<table data-border="1" border="0"><tr><td>A</td></tr></table>','A']]){
    if(process.env.REVIEW_CASE&&process.env.REVIEW_CASE!==name)continue;await reset();if(name.endsWith('styled-caret')){result.setup=await page.evaluate(async name=>{const w=window.__wasm;const cell=name.startsWith('cell');let parent=0,control=0;if(cell){const t=w.createTable(0,0,0,1,1);if(!t.ok)throw Error('cell setup createTable rejected');parent=t.paraIdx;control=t.controlIdx;w.pasteHtmlInCell(0,parent,t.controlIdx,0,0,0,'<p><i>LR</i></p>');}else{w.pasteHtml(0,0,0,'<p><i>LR</i></p>');}await window.__canvasView.loadDocument();window.__inputHandler.cursor.moveTo(cell?{sectionIndex:0,paragraphIndex:0,charOffset:1,parentParaIndex:parent,controlIndex:control,cellIndex:0,cellParaIndex:0,cellPath:[{controlIndex:control,cellIndex:0,cellParaIndex:0}]}:{sectionIndex:0,paragraphIndex:0,charOffset:1});const setupQuery=[];for(let pi=0;pi<w.getParagraphCount(0);pi++){try{setupQuery.push({pi,info:w.getCellInfo(0,pi,0,0)});}catch(e){setupQuery.push({pi,error:String(e)});}}window.__inputHandler.updateCaret();window.__inputHandler.focus();return {parent,control,count:w.getParagraphCount(0),setupQuery};},name);result.setupSnapshot=await snapshot();}await paste(html,text);const expected=await snapshot();result.lastSnapshot={name,expected};fs.writeFileSync(path.join(O,"e2e-result.json"),JSON.stringify(result,null,2));
    if(name==='leading'||name==='combined'){assert.deepEqual(expected.body.find(p=>p.text==='앞 굵게 뒤'),{text:'앞 굵게 뒤',marks:'..BB..'});}
    if(name==='body-styled-caret'){assert.deepEqual(expected.body.find(p=>p.text==='L앞 굵게 뒤R'),{text:'L앞 굵게 뒤R',marks:'I..BB..I'});}if(name==='cell-styled-caret'){assert.deepEqual(expected.tables[0]?.cell,[{text:'L앞 굵게 뒤R',marks:'I..BB..I'}]);}if(name!=='leading'&&name!=='body-styled-caret'){assert.equal(expected.tables.length,1, name+' table exists');const types=expected.tables[0].borders.map(b=>b?.lineType??b?.type);assert.ok(types.every(x=>x!==undefined),'border properties readable');assert.ok(types.every(x=>name==='attribute-alias'?(x===0||x==='None'):(x===1||x==='Solid')),name+' border intent');}const rec={name,afterPaste:expected,roundtrips:[]};
    fs.mkdirSync(path.join(O,'e2e'),{recursive:true});await page.screenshot({path:path.join(O,'e2e',name+'.png')});
    for(const format of ['hwp','hwpx']){
     for(let cycle=1;cycle<=2;cycle++){
      const data=await page.evaluate(fmt=>Array.from(fmt==='hwp'?window.__wasm.exportHwp():window.__wasm.exportHwpx()),format);const bytes=Buffer.from(data);fs.writeFileSync(path.join(O,'e2e',`${name}-${format}-${cycle}.${format}`),bytes);
      await page.evaluate(({data,file})=>window.__wasm.loadDocument(new Uint8Array(data),file),{data,file:name+'.'+format});assert.deepEqual(await snapshot(),expected,`${name}/${format}/${cycle}`);rec.roundtrips.push({format,cycle,sha256:crypto.createHash('sha256').update(bytes).digest('hex'),preserved:true});
     }
    }
    result.cases.push(rec);fs.writeFileSync(path.join(O,'e2e-result.json'),JSON.stringify(result,null,2));console.log(name,'PASS');
  }
  await observation.finish();result.allPassed=true;assert.equal(errors.length,0,'browser page errors');
 }finally{fs.writeFileSync(path.join(O,'e2e-result.json'),JSON.stringify(result,null,2));await observation.stop();await page.close();browser.disconnect();}
})().catch(e=>{console.error(e);process.exitCode=1;});
