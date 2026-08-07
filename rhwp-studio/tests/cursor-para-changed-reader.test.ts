import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// "커서 위치의 문단 속성"을 읽는 곳이 둘이었다.
//   - getParaProperties()      — 문단 모양 대화상자용. 머리말/각주/셀/본문 4문맥.
//   - cursor-para-changed 발행 — 툴바·눈금자용.      각주/셀/본문 3문맥 (머리말 누락).
// 갈래를 따로 두면 문맥이 하나 빠져도 컴파일이 통과한다. 실제로 머리말 편집 중 툴바가
// 본문 문단 값을 보여줬다.
//
// 실측 (headless Chrome, 본문 300% / 머리말 100%):
//   대화상자가 읽는 값 100  vs  툴바 표시 300
// 그 상태에서 사용자가 300을 고르면 select 값이 그대로라 change 가 발화하지 않아
// (toolbar.ts 의 핸들러는 'change' 에 달려 있다) 아무 일도 일어나지 않는다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

function paraChangedEmitBlock(): string {
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('// 문단 속성 (눈금자 마커용) + 스타일');
  assert.notEqual(start, -1, '문단 속성 발행 블록을 찾지 못했다');
  const end = ih.indexOf("this.eventBus.emit('cursor-para-changed'", start);
  assert.notEqual(end, -1, 'cursor-para-changed 발행을 찾지 못했다');
  return ih.slice(start, end);
}

test('cursor-para-changed 는 대화상자와 같은 리더를 쓴다', () => {
  const block = paraChangedEmitBlock();
  assert.match(block, /const paraProps = this\.getParaProperties\(\)/,
    '문단 속성 리더를 따로 구현하고 있다');
});

test('cursor-para-changed 는 문맥 갈래를 자체 구현하지 않는다', () => {
  // 자체 갈래가 남아 있으면 문맥이 추가될 때 또 한쪽만 갱신된다.
  const block = paraChangedEmitBlock();
  assert.doesNotMatch(block, /this\.wasm\.getParaPropertiesAt\(/,
    '본문 갈래를 발행 블록이 직접 호출한다');
  assert.doesNotMatch(block, /this\.wasm\.getCellParaPropertiesAt\(/,
    '셀 갈래를 발행 블록이 직접 호출한다');
  assert.doesNotMatch(block, /this\.wasm\.getParaPropertiesInFootnote\(/,
    '각주 갈래를 발행 블록이 직접 호출한다');
});

test('문단 속성 리더는 네 문맥을 모두 덮는다', () => {
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('getParaProperties(): ParaProperties {');
  assert.notEqual(start, -1, 'getParaProperties 를 찾지 못했다');
  const body = ih.slice(start, ih.indexOf('\n  }', start));

  assert.match(body, /this\.cursor\.isInHeaderFooter\(\)/, '머리말/꼬리말 문맥이 없다');
  assert.match(body, /getParaPropertiesInHf\(/, '머리말/꼬리말 조회가 없다');
  assert.match(body, /this\.cursor\.isInFootnote\(\)/, '각주 문맥이 없다');
  assert.match(body, /getParaPropertiesInFootnote\(/, '각주 조회가 없다');
  assert.match(body, /getCellParaPropertiesAt\(/, '셀 조회가 없다');
  assert.match(body, /getParaPropertiesAt\(/, '본문 조회가 없다');
});
