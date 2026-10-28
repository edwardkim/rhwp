import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const studioRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const read = (relativePath: string) => readFileSync(path.join(studioRoot, relativePath), 'utf8');

test('F5 1·2단계는 포커스 셀 중앙에 서로 다른 단계 마커를 표시한다', () => {
  const cursor = read('src/engine/cursor.ts');
  const inputHandler = read('src/engine/input-handler.ts');
  const renderer = read('src/engine/cell-selection-renderer.ts');
  const styles = read('src/styles/table-selection.css');
  const tokens = read('src/styles/base.css');

  assert.match(cursor, /getCellSelectionFocus\(\)/, '선택 방향을 잃는 정렬 range 대신 focus 좌표를 공개해야 한다');
  assert.match(inputHandler, /getCellSelectionFocus\(\)/);
  assert.match(inputHandler, /getCellSelectionPhase\(\)/);
  assert.match(renderer, /cell-selection-phase-marker/);
  assert.match(renderer, /cell-selection-phase-marker--single/);
  assert.match(renderer, /cell-selection-phase-marker--range/);
  assert.match(renderer, /focus\.row[\s\S]*focus\.col/, '병합 셀을 포함해 focus를 덮는 bbox를 찾아야 한다');
  assert.match(styles, /\.cell-selection-phase-marker\s*\{/);
  assert.match(styles, /\.cell-selection-phase-marker--single\s*\{/);
  assert.match(styles, /\.cell-selection-phase-marker--range\s*\{/);
  assert.match(tokens, /--table-selection-marker-single:/);
  assert.match(tokens, /--table-selection-marker-range:/);
});

test('F5 3단계는 마커 없이 표 전체 선택 상태를 전달한다', () => {
  const renderer = read('src/engine/cell-selection-renderer.ts');
  const phase = read('src/engine/cell-selection-phase.ts');

  assert.match(renderer, /phase === 1 \|\| phase === 2/);
  assert.match(phase, /case 3:[\s\S]*return '표 전체 선택'/);
});

test('F5 단계 이름은 기존 상태 메시지와 분리된 live status로 표시한다', () => {
  const html = read('index.html');
  const main = read('src/main.ts');
  const phase = read('src/engine/cell-selection-phase.ts');
  const statusStyles = read('src/styles/status-bar.css');

  assert.match(
    html,
    /<span id="sb-cell-selection" class="stb-item stb-cell-selection" role="status" aria-live="polite" hidden><\/span>/,
  );
  assert.match(main, /eventBus\.on\('cell-selection-phase-changed'/);
  assert.match(main, /cellSelectionPhaseLabel/);
  assert.match(main, /selectionStatus\.hidden = phase === null/);
  assert.match(phase, /case 1:[\s\S]*return '셀 선택 · 방향키로 이동'/);
  assert.match(phase, /case 2:[\s\S]*return '셀 범위 선택 · 방향키로 확장'/);
  assert.match(statusStyles, /\.stb-cell-selection\s*\{/);
  assert.match(statusStyles, /\.stb-cell-selection\[hidden\]\s*\{/);
  assert.match(html, /id="sb-message" class="stb-message"/, '기존 일시 메시지 채널을 보존해야 한다');
});

test('셀 선택 렌더러의 모든 clear 경로는 하단 단계 상태도 함께 해제한다', () => {
  const renderer = read('src/engine/cell-selection-renderer.ts');
  const main = read('src/main.ts');

  assert.match(renderer, /onPhaseChange/);
  assert.match(renderer, /clear\(\): void \{[\s\S]*this\.onPhaseChange\(null\)/);
  assert.match(main, /new CellSelectionRenderer\([\s\S]*eventBus\.emit\('cell-selection-phase-changed', phase\)/);
});
