import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const cursorSource = readFileSync(new URL('../src/engine/cursor.ts', import.meta.url), 'utf8');
const mouseSource = readFileSync(
  new URL('../src/engine/input-handler-mouse.ts', import.meta.url),
  'utf8',
);
const inputSource = readFileSync(
  new URL('../src/engine/input-handler.ts', import.meta.url),
  'utf8',
);

test('일반 편집 이동과 pointer hit 이동의 좌표 정책은 분리한다', () => {
  assert.match(cursorSource, /moveTo\(pos: DocumentPosition\): void \{[\s\S]*?this\.updateRect\(\);[\s\S]*?\n  \}/);
  assert.match(
    cursorSource,
    /moveToHit\(pos: DocumentPosition\): void \{[\s\S]*?if \(pos\.cursorRect\) \{[\s\S]*?this\.rect = \{ \.\.\.pos\.cursorRect \};[\s\S]*?\} else \{[\s\S]*?this\.updateRect\(\);/,
  );
});

test('#2400 pointer hit 좌표가 있으면 선행 경로 재조회를 수행하지 않는다', () => {
  const moveToHitBody = cursorSource.match(
    /moveToHit\(pos: DocumentPosition\): void \{([\s\S]*?)\n  \}/,
  )?.[1];

  assert.ok(moveToHitBody);
  assert.ok(moveToHitBody.indexOf('if (pos.cursorRect)') < moveToHitBody.indexOf('this.updateRect()'));
  assert.doesNotMatch(
    moveToHitBody.slice(0, moveToHitBody.indexOf('if (pos.cursorRect)')),
    /updateRect/,
  );
});

test('일반 클릭·셀 재진입·드래그 pointer 경로가 moveToHit을 사용한다', () => {
  assert.equal(mouseSource.match(/cursor\.moveTo\(hit\)/g), null);
  assert.ok((mouseSource.match(/cursor\.moveToHit\(hit\)/g)?.length ?? 0) >= 8);
  assert.match(inputSource, /const hit = this\.hitTestFromClientPoint[\s\S]*cursor\.moveToHit\(hit\);/);
});
