import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { VirtualScroll } from '../src/view/virtual-scroll.ts';

const canvasViewSource = readFileSync(
  fileURLToPath(new URL('../src/view/canvas-view.ts', import.meta.url)),
  'utf8',
);

function pages(n: number) {
  return Array.from({ length: n }, () => ({ width: 800, height: 1000 })) as never;
}

function classMethodSource(name: string, nextName: string): string {
  const start = canvasViewSource.indexOf(`\n  ${name}(`);
  const end = canvasViewSource.indexOf(`\n  ${nextName}(`, start + 1);
  assert.ok(start >= 0, `${name} 메서드가 있어야 한다`);
  assert.ok(end > start, `${nextName} 경계가 ${name} 뒤에 있어야 한다`);
  return canvasViewSource.slice(start, end);
}

test('같은 행 그룹은 같은 토폴로지 키를 가지며 맞쪽은 두 쪽과 구분된다', () => {
  const scroll = new VirtualScroll(10);

  scroll.setPageDimensions(pages(6), 0.6, 1200, { kind: 'double' });
  const double = scroll.getLayoutTopologyKey();

  scroll.setPageDimensions(pages(6), 0.6, 1200, { kind: 'multiple', columns: 2, rows: 3 });
  assert.equal(scroll.getLayoutTopologyKey(), double, '동일 2열 행 그룹은 Canvas를 재사용할 수 있다');

  scroll.setPageDimensions(pages(6), 0.6, 1200, { kind: 'facing' });
  assert.notEqual(scroll.getLayoutTopologyKey(), double, '맞쪽 첫 빈 슬롯은 다른 행 토폴로지다');
});

test('자동 단일 열과 명시 한 쪽은 같은 토폴로지다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(4), 0.75, 1200, { kind: 'auto' });
  const automatic = scroll.getLayoutTopologyKey();
  scroll.setPageDimensions(pages(4), 0.75, 1200, { kind: 'single' });
  assert.equal(scroll.getLayoutTopologyKey(), automatic);
});

test('CanvasView는 저장된 쪽 배치를 레이아웃 계산에 전달한다', () => {
  assert.match(
    canvasViewSource,
    /resolvePageViewSettings\([\s\S]*?viewSettings\.pageArrangement,[\s\S]*?viewSettings\.pageMovement/,
  );
  assert.match(
    canvasViewSource,
    /setPageDimensions\([\s\S]*?this\.pageArrangement,[\s\S]*?this\.pageMovement\.direction,[\s\S]*?viewport\.height/,
  );
});

test('CanvasView는 보기 전용 이벤트로 배치를 바꾸고 문서 변경 이벤트를 발행하지 않는다', () => {
  assert.match(
    canvasViewSource,
    /eventBus\.on\('page-arrangement-changed',[\s\S]*?this\.setPageArrangement/,
  );
  const method = classMethodSource('setPageArrangement', 'getPageArrangement');
  assert.doesNotMatch(method, /document-(?:changed|mutated)/);
});

test('배치 전환은 중심 앵커를 복원하고 토폴로지가 달라질 때만 Canvas를 해제한다', () => {
  const method = classMethodSource('setPageViewSettings', 'getPageArrangement');
  assert.match(method, /calculateAnchoredScroll\(/);
  assert.match(method, /CENTER_ZOOM_ANCHOR/);
  assert.match(method, /previousTopology\s*!==\s*nextTopology/);
  assert.match(method, /releaseAllRenderedPages\(\)/);
});

test('가로 쪽 이동은 배치와 함께 한 번에 전환하고 가로 가시 범위를 사용한다', () => {
  assert.match(
    canvasViewSource,
    /eventBus\.on\('page-view-settings-changed',[\s\S]*?this\.setPageViewSettings/,
  );
  assert.match(
    canvasViewSource,
    /getVisiblePages\([\s\S]*?scrollX,[\s\S]*?vpWidth/,
  );
  const method = classMethodSource('setPageViewSettings', 'getPageArrangement');
  assert.match(method, /resolvePageViewSettings/);
  assert.doesNotMatch(method, /document-(?:changed|mutated)/);
});
