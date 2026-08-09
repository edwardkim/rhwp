import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { mergeBorderCoords, BORDER_LINE_MERGE_EPS_PX } from '../src/engine/table-border-lines.ts';

// 셀 bbox 좌표를 소수점 1자리로 반올림해 모으면, 한 물리적 경계를 공유하는 두 셀의 값이
// 반올림 경계에 걸쳐 서로 다른 원소로 남는다.
//
// 실측 (3x3 표, 새 문서, 100% 줌): 셀(0,0) 우측 299.9 / 셀(0,1) 좌측 299.8 → 괘선 2개.
// 이웃 괘선으로 드래그 범위를 정하는 쪽이 자기 자신을 이웃으로 잡아 범위가 뒤집힌다.
//   아래쪽 인덱스(299.8): max 297.2 < cur  → 오른쪽으로 못 감
//   위쪽 인덱스(299.9): min 302.5 > cur    → 왼쪽으로 못 감
// 어느 인덱스를 잡느냐는 hitTestBorder 의 후보 정렬에 달려 있어 방향이 갈린다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

/** 최소 셀 크기 (MIN_TABLE_CELL_SIZE_HWP=200 → px) */
const MIN_CELL_PX = 200 / 75;

test('반올림 경계에 걸쳐 갈라진 같은 경계를 하나로 묶는다', () => {
  // 실측 좌표: 113.4 / 299.8 / 299.9 / 486.3
  const { positions } = mergeBorderCoords([113.4, 299.8, 299.9, 486.3]);
  assert.deepEqual(positions, [113.4, 299.8, 486.3]);
});

test('묶여 사라진 좌표도 대표 괘선 인덱스를 가리킨다', () => {
  // 이 맵이 없으면 자기 좌표로 인덱스를 되찾는 셀의 경계가 hit-test 에서 사라진다.
  const { indexByCoord } = mergeBorderCoords([113.4, 299.8, 299.9, 486.3]);
  assert.equal(indexByCoord.get(299.8), 1);
  assert.equal(indexByCoord.get(299.9), 1, '병합된 좌표가 대표 인덱스를 못 찾는다');
  assert.equal(indexByCoord.get(113.4), 0);
  assert.equal(indexByCoord.get(486.3), 2);
});

test('최소 셀 크기만큼 떨어진 두 경계는 묶이지 않는다', () => {
  // 임계(1.0px)가 최소 셀 크기(2.67px)보다 작아야 실제로 떨어진 경계를 삼키지 않는다.
  assert.ok(BORDER_LINE_MERGE_EPS_PX < MIN_CELL_PX,
    `임계 ${BORDER_LINE_MERGE_EPS_PX}px 가 최소 셀 크기 ${MIN_CELL_PX}px 이상이다`);
  const { positions } = mergeBorderCoords([100, 100 + MIN_CELL_PX]);
  assert.equal(positions.length, 2, '실제로 떨어진 두 경계를 삼켰다');
});

test('임계를 넘는 좌표는 묶이지 않는다', () => {
  const { positions } = mergeBorderCoords([100, 101.5]);
  assert.deepEqual(positions, [100, 101.5]);
});

test('한 괘선으로 묶이는 좌표들의 폭은 임계를 넘지 않는다', () => {
  // 직전 좌표와의 거리로 이으면 0.9px 씩 이어진 사슬이 계속 자라 최소 셀 크기(2.67px)를
  // 삼킨다. 그룹 판정은 대표(그룹 첫 좌표) 기준이어야 폭이 임계로 묶인다.
  const coords = [100, 100.9, 101.8, 102.7, 103.6, 104.5];
  const { indexByCoord } = mergeBorderCoords(coords);

  const spanByIndex = new Map<number, { min: number; max: number }>();
  for (const c of coords) {
    const i = indexByCoord.get(c)!;
    const cur = spanByIndex.get(i);
    if (!cur) spanByIndex.set(i, { min: c, max: c });
    else spanByIndex.set(i, { min: Math.min(cur.min, c), max: Math.max(cur.max, c) });
  }

  for (const [i, { min, max }] of spanByIndex) {
    const span = Math.round((max - min) * 1000) / 1000;
    assert.ok(span <= BORDER_LINE_MERGE_EPS_PX,
      `괘선 ${i} 의 폭 ${span}px 가 임계 ${BORDER_LINE_MERGE_EPS_PX}px 를 넘었다 (사슬 병합)`);
    assert.ok(span < MIN_CELL_PX,
      `괘선 ${i} 의 폭 ${span}px 가 최소 셀 크기 ${MIN_CELL_PX}px 이상 — 셀 하나를 삼켰다`);
  }
});

test('빈 입력과 단일 좌표를 처리한다', () => {
  assert.deepEqual(mergeBorderCoords([]).positions, []);
  const one = mergeBorderCoords([42.5]);
  assert.deepEqual(one.positions, [42.5]);
  assert.equal(one.indexByCoord.get(42.5), 0);
});

test('입력 순서와 무관하게 같은 결과를 낸다', () => {
  const a = mergeBorderCoords([486.3, 299.9, 113.4, 299.8]);
  assert.deepEqual(a.positions, [113.4, 299.8, 486.3]);
  assert.equal(a.indexByCoord.get(299.9), 1);
});

test('hitTestBorder 는 computeBorderLines 가 준 인덱스 맵을 쓴다', () => {
  // 괘선 목록의 대표 좌표로 맵을 다시 만들면 병합돼 사라진 좌표가 빠져,
  // 그 좌표를 가진 셀의 경계가 잡히지 않는다.
  const renderer = source('src/engine/table-resize-renderer.ts');
  const start = renderer.indexOf('hitTestBorder(');
  assert.notEqual(start, -1, 'hitTestBorder 를 찾지 못했다');
  const body = renderer.slice(start, renderer.indexOf('\n  /** 경계선 위에 마커', start));

  assert.match(body, /const \{ rowIndexByY, colIndexByX \} = this\.computeBorderLines\(bboxes\)/,
    'computeBorderLines 의 인덱스 맵을 쓰지 않는다');
  assert.doesNotMatch(body, /new Map\(rowLines\.map/,
    '괘선 목록에서 인덱스 맵을 재조립한다');
  assert.doesNotMatch(body, /new Map\(colLines\.map/,
    '괘선 목록에서 인덱스 맵을 재조립한다');
});

test('computeBorderLines 는 병합을 거친다', () => {
  const renderer = source('src/engine/table-resize-renderer.ts');
  const start = renderer.indexOf('computeBorderLines(bboxes: CellBbox[])');
  assert.notEqual(start, -1, 'computeBorderLines 를 찾지 못했다');
  const body = renderer.slice(start, renderer.indexOf('\n  /** 마우스 좌표가', start));

  assert.match(body, /mergeBorderCoords\(rowYs\)/, '행 좌표를 병합하지 않는다');
  assert.match(body, /mergeBorderCoords\(colXs\)/, '열 좌표를 병합하지 않는다');
  assert.match(body, /rowIndexByY: rows\.indexByCoord/, '행 인덱스 맵을 돌려주지 않는다');
  assert.match(body, /colIndexByX: cols\.indexByCoord/, '열 인덱스 맵을 돌려주지 않는다');
});
