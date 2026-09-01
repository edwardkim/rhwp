import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 표 리사이즈 hover marker 의 pageHint 가드.
//
// 과거 handleResizeHover 는 `cachedTableRef.pageHint !== pageIdx` 로 캐시를 판정하고
// 불일치면 early return 했다. 그래서 채움 지점이 pageHint 대입을 빠뜨리면
// `undefined !== pageIdx` 가 항상 참이 되어 marker 가 영구히 표시되지 않았다
// (이 파일의 원래 가드 대상).
//
// #4117 이후 판정은 table-bbox-cache.ts 의 ensureTableCellBboxCache 로 이사했고,
// pageHint 불일치는 early return 이 아니라 페이지 포함 검사 → 재조회로 이어진다.
// 그 구조에서는 "비교만 하고 채우지 않아 영구 미표시" 실패 모드가 성립하지
// 않는다. 여기서는 그 구조가 유지되는지를 가드한다.
//
// 행위 증명(표 경계 hover → marker 표시)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler-mouse.ts'), 'utf8');
const cache = readFileSync(join(rootDir, 'src/engine/table-bbox-cache.ts'), 'utf8');

test('hover 가 pageHint 불일치를 early return 으로 처리하는 구조가 아니어야 한다 (#4117)', () => {
  assert.doesNotMatch(src, /cachedTableRef\.pageHint\s*!==/,
    'pageHint 불일치는 조기 반환이 아니라 ensureTableCellBboxCache 의 재조회로 이어져야 한다');

  // 새 판정: hint 일치 빠른 길 + O(1) 페이지 membership 검사, 그 아래 재조회.
  assert.match(cache, /cached\.pageHint === pageIdx \|\|/,
    'hint 일치가 셀 배열 스캔 없는 빠른 길이어야 함');
  assert.match(cache, /cached\.pageIndexes\?\.has\(pageIdx\)/,
    'hint 가 달라도 O(1) page membership으로 캐시 포함 여부를 판정해야 함');
  assert.doesNotMatch(cache, /cachedCellBboxes\.some\(/,
    '분할 표의 다른 페이지에서 mousemove마다 전체 bbox를 선형 검색하면 안 됨');
});

test('pageHint 는 채움 지점들이 pageIdx 로 기록한다', () => {
  // 셀 선택 mousedown 경로.
  assert.match(src, /cacheTableCellBboxes\(this, ctx, pageIdx, bboxes\);/,
    'mousedown 성공도 공통 채움 지점으로 실패 메모와 페이지 membership을 함께 갱신해야 한다');
  // hover 채움(choke point) 경로.
  assert.match(cache, /pageHint: pageIdx/,
    'ensureTableCellBboxCache 도 조회한 페이지를 pageHint 로 기록해야 함');

  // pageIdx 산출보다 뒤에서 대입해야 한다(선언 전 사용 방지).
  const pageIdxAt = src.search(/const pageIdx = this\.virtualScroll\.getPageAtPoint\(/);
  const assignAt = src.search(/cacheTableCellBboxes\(this, ctx, pageIdx, bboxes\);/);
  assert.ok(pageIdxAt >= 0 && assignAt >= 0, 'pageIdx 산출과 대입이 모두 존재해야 함');
  assert.ok(pageIdxAt < assignAt, 'pageHint 대입은 pageIdx 산출 뒤여야 함');
});
