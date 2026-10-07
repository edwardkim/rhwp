/** Actual edit keys verify count, search calls, cursor and listener cleanup. */
import {runTest,createNewDocument} from './helpers.mjs';
import assert from 'node:assert/strict';
await runTest('find count refresh once per edit and history operation',async({page})=>{
 await page.evaluate(()=>{document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));const b=[...document.querySelectorAll('button.dialog-btn-primary')].find(b=>b.offsetParent!==null&&b.textContent?.trim()==='시작하기');b?.dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));b?.click();});
 await createNewDocument(page);await page.waitForFunction(()=>window.__inputHandler);
 await page.evaluate(()=>window.__inputHandler.textarea.focus());await page.keyboard.type('foo foo');
 const ctrl=async key=>{await page.keyboard.down('Control');await page.keyboard.press(key);await page.keyboard.up('Control');};
 await ctrl('KeyF');await page.waitForSelector('.find-dialog-input');await page.$eval('.find-dialog-input',e=>{e.focus();e.select();});await page.keyboard.type('foo');await page.keyboard.press('Enter');
 await page.evaluate(()=>{const w=window.__wasm,orig=w.searchText;window.__findCountCalls=0;w.searchText=function(...args){window.__findCountCalls++;return orig.apply(this,args);};const h=window.__inputHandler;h.cursor.clearSelection();h.moveCursorTo({sectionIndex:0,paragraphIndex:0,charOffset:1});h.textarea.focus();});
 for(const [key,count,offset] of [['x',1,2],['Backspace',2,1],['KeyZ',1,2],['KeyY',2,1]]){
  await page.evaluate(()=>window.__findCountCalls=0);if(key==='x')await page.keyboard.type('x');else if(key==='Backspace')await page.keyboard.press(key);else await ctrl(key);
  await page.waitForFunction(n=>document.querySelector('.find-dialog-match-count')?.textContent===`검색 결과 ${n}개`,{},count);
  const r=await page.evaluate(()=>({calls:window.__findCountCalls,cursor:window.__inputHandler.getCursorPosition()}));assert.equal(r.calls,1,key);assert.deepEqual([r.cursor.sectionIndex,r.cursor.paragraphIndex,r.cursor.charOffset],[0,0,offset],key+' preserves edit cursor');
 }
 await page.evaluate(()=>{const h=window.__inputHandler;h.moveCursorTo({sectionIndex:0,paragraphIndex:0,charOffset:0});h.textarea.focus();});
 for(const [key,count] of [['Delete',1],['KeyZ',2],['KeyY',1]]){
  await page.evaluate(()=>window.__findCountCalls=0);if(key==='Delete')await page.keyboard.press(key);else await ctrl(key);
  await page.waitForFunction(n=>document.querySelector('.find-dialog-match-count')?.textContent===`검색 결과 ${n}개`,{},count);assert.equal(await page.evaluate(()=>window.__findCountCalls),1,key);
 }
 await page.click('.find-dialog-input');await page.keyboard.press('Enter');assert.equal(await page.$eval('.find-dialog-match-count',e=>e.textContent),'검색 결과 1개');
 await page.evaluate(()=>{document.querySelector('.find-dialog .dialog-close').click();window.__findCountCalls=0;window.__inputHandler.textarea.focus();});await page.keyboard.type('x');await ctrl('KeyZ');await ctrl('KeyY');assert.equal(await page.evaluate(()=>window.__findCountCalls),0,'closed dialog unsubscribes');
});
