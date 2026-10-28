/** Clicks, Chrome IME and controlled late commit ordering regression. */
import {runTest,createNewDocument,assert as reportAssert} from './helpers.mjs';
import assert from 'node:assert/strict';
await runTest('find first click, IME commit and stable status row',async({page})=>{
 await page.evaluate(()=>{document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));const b=[...document.querySelectorAll('button.dialog-btn-primary')].find(b=>b.offsetParent!==null&&b.textContent?.trim()==='시작하기');b?.dispatchEvent(new MouseEvent('mousedown',{bubbles:true}));b?.click();});
 const ctrl=async key=>{await page.keyboard.down('Control');await page.keyboard.press(key);await page.keyboard.up('Control');};
 const click=forward=>page.click(`.find-dialog-buttons .dialog-btn:nth-child(${forward?2:1})`);
 const state=()=>page.evaluate(()=>({count:document.querySelector('.find-dialog-match-count').textContent,hidden:document.querySelector('.find-dialog-match-count').hidden,status:document.querySelector('.find-dialog-status').textContent,selection:window.__inputHandler.cursor.getSelectionOrdered(),highlights:[...document.querySelectorAll('.selection-highlight')].filter(e=>e.getBoundingClientRect().width>0&&e.getBoundingClientRect().height>0).length,countBox:document.querySelector('.find-dialog-match-count').getBoundingClientRect().toJSON()}));
 async function setup(){await page.evaluate(()=>document.querySelector('.find-dialog .dialog-close')?.click());await createNewDocument(page);await page.evaluate(()=>{const h=window.__inputHandler;h.moveCursorTo({sectionIndex:0,paragraphIndex:0,charOffset:0});h.textarea.focus();});await page.keyboard.type('맘바마바 마바 마바 마바');await ctrl('KeyF');await page.waitForSelector('.find-dialog-input');await page.$eval('.find-dialog-input',e=>{e.focus();e.select();});}
 for(const forward of [true,false]){
  await setup();const cdp=await page.createCDPSession();await cdp.send('Input.imeSetComposition',{text:'마바',selectionStart:2,selectionEnd:2});await click(forward);await page.waitForFunction(()=>document.querySelector('.find-dialog-match-count').textContent==='검색 결과 4개');
  const first=await state();assert.equal(first.count,'검색 결과 4개');assert.equal(first.hidden,false);assert.equal(first.status,'');assert.equal(first.selection.start.charOffset,forward?2:11);assert.ok(first.highlights>0);
  await page.$eval('.find-dialog-input',e=>e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:e.value})));
  assert.deepEqual(await state(),first);
  for(let n=0;n<4;n++)await click(forward);const wrapped=await state();assert.equal(wrapped.status,forward?'처음으로 돌아왔습니다.':'끝으로 돌아왔습니다.');for(const key of ['x','y','width','height'])assert.equal(wrapped.countBox[key],first.countBox[key],key+' stable');
 }
 // Pressing and releasing outside the button cancels navigation, including IME.
 for(const forward of [true,false]){
  await setup();const cdp=await page.createCDPSession();await cdp.send('Input.imeSetComposition',{text:'마바',selectionStart:2,selectionEnd:2});
  await page.evaluate(()=>{const w=window.__wasm;window.__pointerSearchOriginal=w.searchText;window.__pointerSearchCalls=0;w.searchText=function(...args){window.__pointerSearchCalls++;return window.__pointerSearchOriginal.apply(this,args);};});
  const button=await page.$(`.find-dialog-buttons .dialog-btn:nth-child(${forward?2:1})`);const box=await button.boundingBox();await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();
  assert.equal(await page.evaluate(()=>window.__pointerSearchCalls),0);assert.equal((await state()).hidden,true);
  // Release outside: no completed click. The next normal click performs the first search.
  await page.mouse.move(1,1);await page.mouse.up();assert.equal(await page.evaluate(()=>window.__pointerSearchCalls),0);await click(forward);assert.equal((await state()).selection.start.charOffset,forward?2:11);assert.equal(await page.evaluate(()=>window.__pointerSearchCalls),1);
  await page.evaluate(()=>window.__wasm.searchText=window.__pointerSearchOriginal);
 }
 // Actual macOS trace: compositionend/input, then pointerup/mouseup(detail=0),
 // with no pointerdown, mousedown or click reaching the DOM.
 for(const forward of [true,false]){
  await setup();await page.evaluate(()=>{const w=window.__wasm;window.__releaseSearchOriginal=w.searchText;window.__releaseSearchCalls=0;w.searchText=function(...args){window.__releaseSearchCalls++;return window.__releaseSearchOriginal.apply(this,args);};});
  await page.$eval('.find-dialog-input',e=>{e.value='마바';e.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:'마바',isComposing:true}));e.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'바'}));e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:'바',isComposing:false}));});
  await page.$eval(`.find-dialog-buttons .dialog-btn:nth-child(${forward?2:1})`,e=>{e.dispatchEvent(new PointerEvent('pointerup',{bubbles:true,button:0,pointerType:'mouse'}));e.dispatchEvent(new MouseEvent('mouseup',{bubbles:true,button:0,detail:0}));});
  assert.equal((await state()).count,'검색 결과 4개','orphan mouseup must search on first activation');assert.equal((await state()).hidden,false);assert.equal((await state()).selection.start.charOffset,forward?2:11);assert.equal((await state()).status,'');assert.equal(await page.evaluate(()=>window.__releaseSearchCalls),1);
  // A second consumed gesture must work even when the first had no click.
  await page.$eval('.find-dialog-input',e=>e.focus());
  await page.$eval(`.find-dialog-buttons .dialog-btn:nth-child(${forward?2:1})`,e=>{e.dispatchEvent(new PointerEvent('pointerup',{bubbles:true,button:0,pointerType:'mouse'}));e.dispatchEvent(new MouseEvent('mouseup',{bubbles:true,button:0,detail:0}));});
  assert.equal((await state()).selection.start.charOffset,forward?5:8);assert.equal(await page.evaluate(()=>window.__releaseSearchCalls),2);
  // If click also arrives, it must not advance twice.
  await page.$eval(`.find-dialog-buttons .dialog-btn:nth-child(${forward?2:1})`,e=>e.dispatchEvent(new MouseEvent('click',{bubbles:true,detail:1})));
  assert.equal(await page.evaluate(()=>window.__releaseSearchCalls),2);
  await click(forward);assert.equal((await state()).selection.start.charOffset,forward?8:5);assert.equal(await page.evaluate(()=>window.__releaseSearchCalls),3);
  const button=await page.$(`.find-dialog-buttons .dialog-btn:nth-child(${forward?2:1})`);await button.focus();await page.keyboard.press('Enter');assert.equal(await page.evaluate(()=>window.__releaseSearchCalls),4);assert.equal((await state()).selection.start.charOffset,forward?11:2);
  await page.evaluate(()=>window.__wasm.searchText=window.__releaseSearchOriginal);
 }
 await setup();await page.$eval('.find-dialog-input',e=>{e.value='마';e.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:'마'}));});
 await page.evaluate(()=>{window.__imeSearchCalls=0;const w=window.__wasm,original=w.searchText;w.searchText=function(...args){window.__imeSearchCalls++;return original.apply(this,args);};});
 await click(true);assert.equal(await page.evaluate(()=>window.__imeSearchCalls),0);
 await page.$eval('.find-dialog-input',e=>{e.value='마바';e.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'마바'}));e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:'마바'}));});
 await page.waitForFunction(()=>document.querySelector('.find-dialog-match-count').textContent==='검색 결과 4개');assert.equal(await page.evaluate(()=>window.__imeSearchCalls),1);assert.equal((await state()).selection.start.charOffset,2);
 reportAssert(true,'first clicks, IME ordering and stable count passed');
});
