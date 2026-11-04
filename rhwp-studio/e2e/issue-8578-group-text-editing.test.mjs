// 관계형: 묶음 자식 글상자와 배경 밖 본문의 실제 마우스·키보드 편집 소유 계약.
// 주석 갱신: 2026-11-05T00:50:00+09:00. 픽셀 기대 위치는 고정하지 않는다.
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { runTest } from './helpers.mjs';

const output = new URL('../../output/issue8578-browser/', import.meta.url);
mkdirSync(output, {recursive:true});
const settle = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const position = page => page.evaluate(() => window.__inputHandler.cursor.getPosition());
const read = (page,index,para=0) => page.evaluate(({index,para}) =>
  window.__wasm.getTextInCell(0,4,0,index,para,0,10000), {index,para});

async function documentPoint(page, pageIndex, x, y) {
  await page.evaluate(({pageIndex,y}) => {
    const input=window.__inputHandler;
    input.viewportManager.setScrollTop(Math.max(0,input.virtualScroll.getPageOffset(pageIndex)
      +y*input.viewportManager.getZoom()-300));
  }, {pageIndex,y});
  await settle(page);
  return page.evaluate(({pageIndex,x,y}) => {
    const input=window.__inputHandler, content=document.querySelector('#scroll-content');
    const bounds=content.getBoundingClientRect(), zoom=input.viewportManager.getZoom();
    return {x:bounds.left+input.virtualScroll.getPageLeftResolved(pageIndex,content.clientWidth)+x*zoom,
      y:bounds.top+input.virtualScroll.getPageOffset(pageIndex)+y*zoom};
  }, {pageIndex,x,y});
}

async function clickText(page, pageIndex, text) {
  const box=await page.evaluate(({pageIndex,text}) => {
    function find(node) {
      if(!node || typeof node!=='object') return null;
      if(typeof node.text==='string' && node.text.includes(text) && node.bbox) return node.bbox;
      for(const value of Object.values(node)){const found=find(value);if(found)return found;}
      return null;
    }
    const box=find(window.__wasm.getPageLayerTreeObject(pageIndex));
    if(!box)throw new Error(`TextRun 없음: ${pageIndex}/${text}`);
    return box;
  }, {pageIndex,text});
  const point=await documentPoint(page,pageIndex,box.x+(box.width??box.w)/2,box.y+(box.height??box.h)/2);
  await page.mouse.click(point.x,point.y);
  await settle(page);
  return {point,position:await position(page)};
}

runTest('#8578 묶음 글상자 클릭·편집·Esc 소유 보존',async({page}) => {
  const errors=[];
  page.on('console',message=>{
    if(/셀 문단.*실패|updateRect 실패|커맨드 실행 실패|getSelectionRects 실패/.test(message.text())) errors.push(message.text());
  });
  // #8578은 편람의 새 편집 결함을 명시적으로 검증한다. 기존 코퍼스 수집 제외 목록은 확장하지 않는다.
  await page.evaluate(async()=>{
    const file='2025 행정업무운영 편람(최종).hwpx';
    const bytes=new Uint8Array(await(await fetch('/samples/'+encodeURIComponent(file))).arrayBuffer());
    const info=window.__wasm.loadDocument(bytes,file);
    const {loadWebFonts}=await import('/src/core/font-loader.ts');
    await loadWebFonts(info.fontsUsed??[]); await document.fonts.ready;
    await window.__canvasView.loadDocument();
    window.__inputHandler.activateWithCaretPosition();
    window.__inputHandler.focus();
  });
  const title=await read(page,0), body=await read(page,1);
  const first=await clickText(page,2,'이 편람은');
  assert.equal(first.position.parentParaIndex,4);
  assert.equal(first.position.cellIndex,1,'안내 본문은 제목과 다른 글상자');
  const offset=first.position.charOffset;
  await page.keyboard.type('X8578'); await settle(page);
  const edited=await read(page,1);
  assert.equal(edited,[...body].slice(0,offset).join('')+'X8578'+[...body].slice(offset).join(''));
  assert.equal(await read(page,0),title,'본문 입력이 제목을 바꾸지 않는다');
  await page.keyboard.press('Control+z'); await settle(page);
  assert.equal(await read(page,1),body,'실제 키보드 undo');
  await page.keyboard.press('Control+y'); await settle(page);
  assert.equal(await read(page,1),edited,'실제 키보드 redo');
  const count=await page.evaluate(()=>window.__wasm.getCellParagraphCount(0,4,0,1));
  await page.keyboard.press('Enter'); await settle(page);
  assert.equal(await page.evaluate(()=>window.__wasm.getCellParagraphCount(0,4,0,1)),count+1);
  await page.keyboard.press('Backspace'); await settle(page);
  assert.equal(await read(page,1),edited,'Enter/Backspace 병합에서 내용 보존');
  assert.equal(await read(page,0),title);
  await page.screenshot({path:new URL('page003-edit.png',output).pathname});

  const secondBody=await read(page,1,1);
  const selection=await clickText(page,2,'이번 편람에서는');
  await page.mouse.move(selection.point.x,selection.point.y); await page.mouse.down();
  await page.mouse.move(selection.point.x+28,selection.point.y,{steps:5}); await page.mouse.up();
  await settle(page);
  assert.equal(await page.evaluate(()=>window.__inputHandler.cursor.hasSelection()),true,'실제 글상자 드래그 선택');
  await page.keyboard.press('Backspace'); await settle(page);
  assert.notEqual(await read(page,1,1),secondBody,'선택한 본문만 삭제');
  await page.keyboard.press('Control+z'); await settle(page);
  assert.equal(await read(page,1,1),secondBody);

  const contents=await clickText(page,3,'행정업무 운영 개요');
  assert.equal(contents.position.parentParaIndex,undefined,'목차 본문은 장식 묶음에 가로채이지 않는다');
  assert.equal(contents.position.paragraphIndex,8);
  await page.keyboard.type('C8578'); await settle(page);
  await page.keyboard.press('Control+z'); await settle(page);
  await page.screenshot({path:new URL('page004-body.png',output).pathname});

  const box=await page.evaluate(()=>window.__inputHandler.findPictureBbox({sec:0,ppi:4,ci:0,type:'group'}));
  assert.ok(box);
  const border=await documentPoint(page,box.pageIndex,box.x+1,box.y+box.h/2);
  await page.mouse.click(border.x,border.y); await settle(page);
  assert.equal(await page.evaluate(()=>window.__inputHandler.cursor.isInPictureObjectSelection()),true);
  await page.keyboard.press('Escape'); await settle(page);
  const escaped=await position(page);
  assert.equal(escaped.paragraphIndex,4,'Esc는 다음 쪽의 인접 문단이 아닌 소유 본문');
  assert.equal(escaped.parentParaIndex,undefined);
  await page.keyboard.type('E8578'); await settle(page);
  await page.keyboard.press('Control+z'); await settle(page);
  assert.equal(await read(page,1),edited,'소유 본문 입력은 묶음 자식 내용을 변경하지 않는다');
  assert.deepEqual(errors,[],'클릭·선택·편집에서 잘못된 셀 주소 경고 없음');
  writeFileSync(new URL('result.json',output),JSON.stringify({first,contents,escaped,errors},null,2)+'\n');
});
