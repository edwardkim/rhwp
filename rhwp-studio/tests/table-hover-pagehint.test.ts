import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 표 리사이즈 hover marker 의 캐시 일치 판정 가드.
//
// handleMouseMove 는 `cachedTableRef.pageHint !== pageIdx` 로 캐시 유효성을 판정한다.
// 그런데 cachedTableRef 에 값을 넣는 유일한 지점(mousedown, 셀 선택 모드)이 pageHint 를
// 채우지 않으면 `undefined !== pageIdx` 가 항상 참이라 hover 가 매번 early return 하고,
// 그 아래 marker 갱신 코드는 도달하지 못한다(리사이즈 marker 가 절대 표시되지 않음).
//
// 행위 증명(표 경계 hover → marker 표시)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler-mouse.ts'), 'utf8');

test('pageHint 를 비교만 하고 채우지 않는 상태가 아니어야 한다', () => {
  const reads = src.match(/cachedTableRef\.pageHint\s*!==/g) ?? [];
  assert.ok(reads.length > 0, 'hover 캐시 판정이 pageHint 를 비교하고 있어야 함(전제 확인)');

  // 비교하는 이상 어딘가에서 반드시 값이 채워져야 한다.
  const writes = src.match(/cachedTableRef\.pageHint\s*=[^=]/g) ?? [];
  assert.ok(writes.length > 0,
    'pageHint 를 비교만 하고 대입하지 않으면 hover marker 가 영구히 표시되지 않는다');
});

test('pageHint 는 hover 판정과 같은 pageIdx 로 채운다', () => {
  assert.match(src, /this\.cachedTableRef\.pageHint = pageIdx;/,
    '캐시를 만든 페이지 번호를 그대로 기록해야 hover 판정과 축이 맞는다');

  // pageIdx 산출보다 뒤에서 대입해야 한다(선언 전 사용 방지).
  const pageIdxAt = src.search(/const pageIdx = this\.virtualScroll\.getPageAtPoint\(/);
  const assignAt = src.search(/this\.cachedTableRef\.pageHint = pageIdx;/);
  assert.ok(pageIdxAt >= 0 && assignAt >= 0, 'pageIdx 산출과 대입이 모두 존재해야 함');
  assert.ok(pageIdxAt < assignAt, 'pageHint 대입은 pageIdx 산출 뒤여야 함');
});
