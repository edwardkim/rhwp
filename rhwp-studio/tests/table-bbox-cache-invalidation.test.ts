import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 표 리사이즈 런타임 캐시(cachedCellBboxes / cachedTableRef) 무효화 가드.
//
// cachedTableRef 는 {sec, ppi, ci} — 표의 "정체성"만 담는다. 줄 지우기 같은 구조 편집은
// 이 셋을 바꾸지 않으므로 handleResizeHover 의 신선도 검사(sec/ppi/ci/pageHint)를 그대로
// 통과한다. 반면 cachedCellBboxes 의 기하와 cellIdx 번호는 바뀐다.
//
//   10줄 표에 커서 → 셀 선택 모드에서 클릭(캐시 채워짐) → 줄 지우기 ×2 → Esc
//   → 지금의 5/6행 경계를 드래그
//   기대: 현재 5/6행이 리사이즈됨
//   실제: 옛 경계에 marker 가 뜨고, 옛 번호의 cellIdx 로 다른 두 행이 리사이즈됨
//
// resolveTableResizeHit(input-handler-mouse.ts)는 캐시를 아무 검증 없이 그대로 반환하고,
// finishResizeDrag 는 state.bboxes 의 cellIdx 로 resizeTableCells 를 부른다.
// undo/redo 경로는 이미 같은 이유로 clearTableResizeRuntimeCache 를 호출한다.
// 행위 증명(표 구조 편집 후 리사이즈)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');

/** 메서드 본문을 다음 최상위 멤버 직전까지 잘라낸다. */
function methodBody(name: string): string {
  const start = src.indexOf(`  private ${name}(`);
  assert.notEqual(start, -1, `${name} 메서드 not found`);
  const rest = src.slice(start + 1);
  const end = rest.search(/\n {2}(private |public )?[a-zA-Z_]\w*\s*\(/);
  return end === -1 ? rest : rest.slice(0, end);
}

test('편집 후 처리에서 표 리사이즈 런타임 캐시를 무효화한다', () => {
  const body = methodBody('afterEdit');

  assert.match(body, /clearTableResizeRuntimeCache\s*\(/,
    'afterEdit 가 캐시를 지우지 않으면 구조 편집 뒤 옛 bbox 로 엉뚱한 행이 리사이즈된다');

  // lastCellKey 만 지우고 끝내던 형태로 되돌아가면 안 된다.
  assert.match(body, /this\.lastCellKey = null/, 'lastCellKey 무효화 유지');
  assert.match(body, /this\.protectedCellHitCache = null/, 'protectedCellHitCache 무효화 유지');
});

test('무효화 루틴이 두 캐시 필드를 모두 비운다', () => {
  const body = methodBody('clearTableResizeRuntimeCache');
  assert.match(body, /this\.cachedTableRef = null/, 'cachedTableRef 를 비워야 함');
  assert.match(body, /this\.cachedCellBboxes = null/, 'cachedCellBboxes 를 비워야 함');
});

test('undo/redo 도 같은 루틴을 계속 사용한다', () => {
  // 구조 편집만 고치고 undo/redo 경로가 빠지면 반쪽짜리가 된다(회귀 그물).
  const calls = src.match(/this\.clearTableResizeRuntimeCache\(\)/g) ?? [];
  assert.ok(calls.length >= 3,
    `문서 로드·undo/redo·편집 후 처리에서 모두 호출돼야 함(현재 ${calls.length}곳)`);
});
