import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 툴바 줄 간격은 백분율 전용 목록이다. 문단의 줄 간격 종류는 네 가지이고
// (Percent / Fixed / SpaceOnly / Minimum — para-shape-dialog.ts 의 LINE_SPACING_TYPES)
// 문단 모양 대화상자가 넷 다 설정할 수 있다.
//
// 백분율이 아닌 문단에서 목록을 그대로 두면 직전 문단의 값이 남아 두 가지가 함께 깨진다.
//   (1) 표시가 실제와 어긋난다 — 실측: Fixed 13.3 문단에서 "160" 표시
//   (2) 그 표시값과 같은 항목을 고르면 select 의 change 가 발화하지 않는다.
//       줄 간격 핸들러는 'change' 에만 달려 있으므로 아무 일도 일어나지 않는다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

function updateParaStateBody(): string {
  const toolbar = source('src/ui/toolbar.ts');
  const start = toolbar.indexOf('private updateParaState(props: ParaProperties): void {');
  assert.notEqual(start, -1, 'updateParaState 를 찾지 못했다');
  const end = toolbar.indexOf('\n  }', start);
  assert.notEqual(end, -1, 'updateParaState 끝을 찾지 못했다');
  return toolbar.slice(start, end);
}

test('백분율이 아닌 줄 간격에서는 툴바 목록을 비운다', () => {
  const body = updateParaStateBody();
  assert.match(body, /this\.lsSelect\.selectedIndex = -1;/,
    '백분율이 아닌 문단에서 직전 값이 그대로 남는다');
});

test('백분율 갈래가 비우기 갈래로 흘러내리지 않는다', () => {
  // return 이 없으면 방금 맞춘 백분율 값을 곧바로 -1 로 지워 목록이 항상 비어 버린다.
  const body = updateParaStateBody();
  assert.match(body, /this\.lsSelect\.value = String\(val\);\s*\n\s*return;/,
    '백분율 갈래에 return 이 없다');
});

test('줄 간격 적용은 select 의 change 에 달려 있다', () => {
  // 이 배선이 change 인 한, 표시값과 원하는 값이 같으면 선택해도 발화하지 않는다.
  // 위 두 케이스가 막으려는 것이 정확히 그 상태다.
  const toolbar = source('src/ui/toolbar.ts');
  const start = toolbar.indexOf('private setupLineSpacingDropdown()');
  assert.notEqual(start, -1, 'setupLineSpacingDropdown 을 찾지 못했다');
  const body = toolbar.slice(start, toolbar.indexOf("this.lsSelect.addEventListener('dblclick'", start));
  assert.match(body, /this\.lsSelect\.addEventListener\('change'/,
    '줄 간격 적용 배선을 찾지 못했다 — 이 테스트의 전제가 바뀌었다');
});
