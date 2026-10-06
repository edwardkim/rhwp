import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { balancedFrom, functionBodyFrom } from './support/source-guard.ts';

import { assertShowsText, assertDoesNotShowText } from './support/i18n-text.ts';
// ChartDataDialog 는 DOM 셸이라 node 로 인스턴스화할 수 없다 — 편집 판단 로직은
// chart-data-target.test.ts / chart-grid-model.test.ts 가 순수 함수로 검증하고,
// 여기서는 셸의 배선 계약(검증 순서·undo 라우팅·재열거)을 소스에서 못 박는다
// (bookmark-dialog 선례).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const dialog = readFileSync(join(rootDir, 'src/ui/chart-data-dialog.ts'), 'utf8');

function confirmBody(): string {
  return functionBodyFrom(dialog, 'onConfirm()');
}

test('무변경이면 어떤 wasm 쓰기도 없이 닫는다 — hasAnyEdit 가 첫 관문', () => {
  const body = confirmBody();
  const gate = body.indexOf('hasAnyEdit');
  const write = body.indexOf('setChartDataByIndex');
  assert.ok(gate !== -1, 'hasAnyEdit 판정이 있어야 한다');
  assert.ok(write !== -1, 'setChartDataByIndex 호출이 있어야 한다');
  assert.ok(gate < write, '무변경 판정이 쓰기(dryRun 포함)보다 앞서야 한다');
});

test('dryRun 선검증이 실쓰기(executeOperation)보다 앞선다', () => {
  const body = confirmBody();
  const dry = body.indexOf('dryRun: true');
  const exec = body.indexOf('executeOperation');
  assert.ok(dry !== -1, 'dryRun 선검증이 있어야 한다');
  assert.ok(exec !== -1, 'executeOperation 라우팅이 있어야 한다');
  assert.ok(dry < exec, 'dryRun 이 실쓰기보다 앞서야 한다');
});

test('실쓰기는 snapshot/objectProps 로 라우팅된다 — undo 스택 연동', () => {
  assert.match(dialog, /kind:\s*'snapshot'/);
  assert.match(dialog, /operationType:\s*'objectProps'/);
});

test('쓰기 직전 operation 클로저 안에서 재열거·재대조한다 — index 드리프트 차단', () => {
  // [#6053] 예전에는 `indexOf('});', opStart)` 로 잘랐다. 클로저 안에 `});` 로 끝나는
  // 콜백이 하나만 들어와도 슬라이스가 조기 절단돼 이 단언이 **거짓 실패**한다.
  // 중괄호 균형 절단으로 바꾼다 — 단언 대상은 서식이 아니라 배선이다.
  const opSlice = balancedFrom(dialog, 'operation:', '{');
  assert.match(opSlice, /listCharts\(\)/, 'operation 안에서 listCharts 재열거');
  assert.match(opSlice, /sameChartAddress/, '주소로 재대조해야 한다');
});

test('services 미주입 폴백은 직접 적용 후 document-changed 를 알린다', () => {
  assert.match(dialog, /document-changed/);
});

// ── B2 구조 편집 배선 (#6053) ─────────────────────────────

test('structure 는 페이로드에서 유도한다 — 항상 켜지 않는다', () => {
  const body = confirmBody();
  assert.match(body, /needsStructure\(/, 'structure 의도는 needsStructure 가 판정한다');
  // `structure: true` 를 리터럴로 박으면 무편집 왕복에도 실려 B1 의 네 거부가 꺼진다.
  assert.doesNotMatch(body, /structure:\s*true/, 'structure 를 리터럴로 켜면 안 된다');
});

test('그리드 상태는 모델이 쥔다 — 수집이 봉투(data.series)를 축으로 돌지 않는다', () => {
  const body = confirmBody();
  assert.match(body, /gridValues\(model\)/, '목표 행렬은 모델에서 나온다');
  assert.match(body, /gridSeriesNames\(model\)/);
  assert.doesNotMatch(
    body,
    /data\.series\.map|data\.series\.forEach/,
    '봉투를 축으로 돌면 구조가 못 바뀐다',
  );
});

test('셀 입력은 모델로 write-through 된다 — 재렌더가 무손실이어야 구조 연산이 성립한다', () => {
  const body = functionBodyFrom(dialog, 'private renderGrid(');
  for (const setter of ['setCell(', 'setLabel(', 'setSeriesName(']) {
    assert.ok(body.includes(setter), `${setter} 로 모델에 반영해야 한다`);
  }
});

test('그리드 셀은 좌표를 data-* 로 싣는다 — 우클릭이 어느 행·열인지 알아야 한다', () => {
  const body = functionBodyFrom(dialog, 'private renderGrid(');
  assert.match(body, /dataset\.series\s*=/);
  assert.match(body, /dataset\.row\s*=/);
});

test('우클릭은 전역 컨텍스트 메뉴가 아니라 로컬 메뉴를 쓴다', () => {
  assert.match(dialog, /from '\.\/local-context-menu'/);
  assert.doesNotMatch(dialog, /from '\.\/context-menu'/, '전역 메뉴는 커맨드 등재를 요구한다');
  const body = functionBodyFrom(dialog, 'private renderGrid(');
  assert.match(body, /addEventListener\('contextmenu'/);
});

test('사전 비활성은 봉투에서 유도되는 것만 — 캔들 양끝·마지막 행·열·라벨 구조', () => {
  const body = functionBodyFrom(dialog, 'private menuItems(');
  assert.match(body, /hasUpDownBars === true/, '[#6037] 캔들 장치는 양끝 계열을 막는다');
  assert.match(body, /labelsUsable/, '다층·비공유 라벨은 행 연산을 막는다');
  assert.match(body, /rowCount <= 1/, 'lastPointDeleteRefused 예방');
  assert.match(body, /series\.length <= 1/, 'lastSeriesDeleteRefused 예방');
  // 원형은 파손이 아니라 무효과다(#6037) — 막지 않고 알려만 준다.
  assert.match(body, /note: pieNote/, '원형은 안내만 한다');
  assert.doesNotMatch(body, /disabledReason:\s*pieNote/, '원형 계열 추가를 막으면 안 된다');
});

test('계열명 대체 문구는 표시 전용이다 — 모델에 들어가면 가짜 이름이 저장된다', () => {
  const body = functionBodyFrom(dialog, 'private renderGrid(');
  // `계열 N` 은 c:tx 가 없는 계열의 표시용 텍스트다. 그 가지에는 입력이 없어야 한다.
  const from = body.indexOf('s.name === null');
  const locked = body.slice(from, body.indexOf('} else {', from));
  assertShowsText(locked, '계열 {p1}');
  assert.doesNotMatch(locked, /setSeriesName/, '이름 칸이 없는 계열은 입력을 열지 않는다');
});
