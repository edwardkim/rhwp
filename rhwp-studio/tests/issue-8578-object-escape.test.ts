// 관계형: 개체 Esc는 소유 문단으로 돌아가며 삭제·붙여넣기 착지는 바꾸지 않는다.
// 주석 갱신: 2026-11-05T01:24:05+09:00. 시각 픽셀 위치를 고정하지 않는 편집 주소 계약이다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('#8578 묶음 Esc는 다음 문단으로 넘어가지 않고 원 소유 문단을 편집한다', () => {
  const src = new URL('../src/', import.meta.url).href;
  const driver = `
    import { registerHooks } from 'node:module';
    import assert from 'node:assert/strict';
    const root = ${JSON.stringify(src)};
    registerHooks({resolve(s,c,next) {
      if(s.startsWith('@/')) return next(root+s.slice(2)+'.ts',c);
      if(/^\\.{1,2}\\//.test(s) && !/\\.[a-z]+$/.test(s)) return next(s+'.ts',c);
      return next(s,c);
    }});
    const {CursorState}=await import(root+'engine/cursor.ts');
    const rect={pageIndex:2,x:10,y:10,height:12};
    const wasm={getParagraphCount:()=>10,getParagraphLength:()=>10,
      getControlTextPositions:()=>[0],
      getCellTextDirection:()=>0,getCursorRectByPath:()=>rect,
      getCursorRectByPathNear:()=>rect,
      getCursorRect:(sec,para,off)=>({pageIndex:para===4?2:3,x:10,y:10,height:12})};
    const cursor=new CursorState(wasm);
    cursor.enterPictureObjectSelectionDirect(0,4,0,'group');
    assert.equal(cursor.positionOutsideSelectedPicture().paragraphIndex,5,
      '삭제·붙여넣기 인접 문단 계약은 보존');
    (cursor.returnToSelectedObjectOwner ?? cursor.moveOutOfSelectedPicture).call(cursor);
    assert.deepEqual(cursor.getPosition(),{sectionIndex:0,paragraphIndex:4,charOffset:0});
    assert.equal(cursor.isInPictureObjectSelection(),false);
    assert.equal(cursor.getRect().pageIndex,2);
    const path=[{controlIndex:2,cellIndex:3,cellParaIndex:4}];
    cursor.enterPictureObjectSelectionDirect(0,2,1,'shape',3,4,undefined,2,path);
    cursor.returnToSelectedObjectOwner();
    assert.deepEqual(cursor.getPosition().cellPath,path,'중첩 개체도 원 소유 컨테이너 보존');
    assert.equal(cursor.getPosition().parentParaIndex,2);
    assert.equal(cursor.getPosition().cellParaIndex,4);
    assert.equal(cursor.getPosition().charOffset,0);
    console.log('owner escape PASS');
  `;
  const result=spawnSync(process.execPath,['--experimental-transform-types','--no-warnings','--input-type=module','-e',driver],{
    cwd:fileURLToPath(new URL('..',import.meta.url)),encoding:'utf8',
  });
  assert.equal(result.status,0,result.stderr||result.stdout);
  assert.match(result.stdout,/owner escape PASS/);
});
