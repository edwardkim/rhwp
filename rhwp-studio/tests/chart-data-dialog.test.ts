import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// ChartDataDialog 는 DOM 셸이라 node 로 인스턴스화할 수 없다 — 편집 판단 로직은
// chart-data-target.test.ts 가 순수 함수로 검증하고, 여기서는 셸의 배선 계약
// (검증 순서·undo 라우팅·재열거)을 소스에서 못 박는다 (bookmark-dialog 선례).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const dialog = readFileSync(join(rootDir, 'src/ui/chart-data-dialog.ts'), 'utf8');

function confirmBody(): string {
  const start = dialog.indexOf('onConfirm()');
  assert.notEqual(start, -1, 'onConfirm() 이 있어야 한다');
  return dialog.slice(start);
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
  const opStart = dialog.indexOf('operation:');
  assert.notEqual(opStart, -1);
  const opSlice = dialog.slice(opStart, dialog.indexOf('});', opStart));
  assert.match(opSlice, /listCharts\(\)/, 'operation 안에서 listCharts 재열거');
});

test('services 미주입 폴백은 직접 적용 후 document-changed 를 알린다', () => {
  assert.match(dialog, /document-changed/);
});
