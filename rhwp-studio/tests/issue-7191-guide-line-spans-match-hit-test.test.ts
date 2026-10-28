// [Issue #7191] 표 크기 조절 가이드선이 실제 괘선이 없는 구간까지 표 전체 높이로 그어진다.
//
// `computeBorderLines` 는 선마다 표 전체 범위(`minX..maxX` / `minY..maxY`) 하나만 들고
// 있었고 `showMarker` 가 그 값으로 마커를 그렸다. 반면 적중 판정 `hitTestBorder` 는 칸
// 상자를 훑는다 — **그리는 범위와 잡는 범위가 서로 다른 출처**였다.
//
// 병합 칸이 있으면 한 열 경계가 일부 행에만 존재하므로 둘이 반드시 어긋난다. 신고 문서
// 3147199 1쪽에서는 `x=374.0`·`x=446.5` 가 두 칸에만 있는 경계인데 선은 표 꼭대기 56.7
// 부터 바닥 884.6 까지 828px 를 그었고, 그 구간에 마우스를 올리면 잡히지 않았다.
//
// 수정은 두 범위를 같은 출처(칸 상자)에서 뽑는다 — `computeBorderSpans` 가 경계별 구간을
// 만들고 `showMarker` 가 그 구간마다 하나씩 그린다.
//
// # 이 시험이 잠그는 것
//
// 구간 계산의 **실제 동작**(`table-border-lines.ts` 는 별칭 없는 순수 모듈이라 직접
// 부를 수 있다)과, 그리는 쪽이 그 구간을 쓴다는 소스 계약.
//
// # 잠그지 않는 것
//
// 마커의 색·두께·DOM 구조와 드래그 동작은 범위 밖이다.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const round = (v: number) => Math.round(v * 10) / 10;

interface SpanCell { x: number; y: number; w: number; h: number }

// 수정 전 트리에는 아래 두 함수가 없다. 정적 import 로 받으면 파일 전체가 모듈 오류로
// 죽어 "결함을 드러냈다" 고 말할 수 없다(빌드 실패는 재현이 아니다). 동적으로 받아
// 각 시험이 제 이유로 실패하게 한다.
async function geometry() {
  return await import('../src/engine/table-border-lines.ts');
}

/** 제품 코드(`computeBorderLines`)와 같은 순서로 경계선을 만든다. */
async function borderLines(cells: SpanCell[]) {
  const { mergeBorderCoords, computeBorderSpans } = await geometry();
  const rowYs = new Set<number>();
  const colXs = new Set<number>();
  for (const c of cells) {
    rowYs.add(round(c.y));
    rowYs.add(round(c.y + c.h));
    colXs.add(round(c.x));
    colXs.add(round(c.x + c.w));
  }
  const rows = mergeBorderCoords(rowYs);
  const cols = mergeBorderCoords(colXs);
  const { rowSpans, colSpans } = computeBorderSpans(cells, rows.indexByCoord, cols.indexByCoord, round);
  return {
    rowLines: rows.positions.map((y, i) => ({ y, spans: rowSpans.get(i) ?? [], index: i })),
    colLines: cols.positions.map((x, i) => ({ x, spans: colSpans.get(i) ?? [], index: i })),
    colIndexByX: cols.indexByCoord,
  };
}

const cell = (x: number, y: number, w: number, h: number): SpanCell => ({ x, y, w, h });

/**
 * 신고 문서와 같은 형상: 아래쪽 두 행에만 있는 세로 경계.
 *
 *   x:    0        100       200
 *   y=0   +------------------+     (0..200 을 한 칸이 가로지른다 — 병합)
 *   y=50  +--------+---------+     (x=100 경계는 여기부터만 있다)
 *   y=100 +--------+---------+
 *   y=150 +------------------+
 */
function mergedShape(): SpanCell[] {
  return [
    cell(0, 0, 200, 50),
    cell(0, 50, 100, 50),
    cell(100, 50, 100, 50),
    cell(0, 100, 100, 50),
    cell(100, 100, 100, 50),
  ];
}

test('부분 경계는 자기 구간만 그린다 — 표 전체 높이를 긋지 않는다', async () => {
  const { colLines, colIndexByX } = await borderLines(mergedShape());
  const index = colIndexByX.get(100);
  assert.notEqual(index, undefined, 'x=100 경계가 있어야 한다');

  const line = colLines.find((l) => l.index === index)!;
  assert.deepEqual(
    line.spans,
    [{ start: 50, end: 150 }],
    'x=100 은 y=50..150 에만 있는 경계 — 표 꼭대기 y=0 부터 긋지 않는다',
  );
});

test('경계가 없는 구간은 어떤 span 에도 덮이지 않는다', async () => {
  const { colLines, colIndexByX } = await borderLines(mergedShape());
  const line = colLines.find((l) => l.index === colIndexByX.get(100))!;

  // 병합 칸이 가로지르는 y=25 는 x=100 경계가 없는 자리다.
  const covered = line.spans.some((s) => s.start <= 25 && 25 <= s.end);
  assert.equal(covered, false, 'y=25 는 x=100 경계가 없는 구간 — 그리면 안 된다');
});

test('끊긴 경계는 토막마다 따로 남는다', async () => {
  // x=100 경계가 위(0..50)와 아래(150..200)에만 있고 가운데는 병합으로 비어 있다.
  const { colLines, colIndexByX } = await borderLines([
    cell(0, 0, 100, 50),
    cell(100, 0, 100, 50),
    cell(0, 50, 200, 100),
    cell(0, 150, 100, 50),
    cell(100, 150, 100, 50),
  ]);
  const line = colLines.find((l) => l.index === colIndexByX.get(100))!;

  assert.deepEqual(
    line.spans,
    [{ start: 0, end: 50 }, { start: 150, end: 200 }],
    '가운데 병합 구간을 건너뛴 두 토막이어야 한다',
  );
});

test('격자 표에서는 경계가 표 전체를 가로지른다 — 회귀 방지', async () => {
  const { colLines, rowLines } = await borderLines([
    cell(0, 0, 100, 50), cell(100, 0, 100, 50),
    cell(0, 50, 100, 50), cell(100, 50, 100, 50),
  ]);

  for (const line of colLines) {
    assert.deepEqual(line.spans, [{ start: 0, end: 100 }], `x=${line.x} 은 표 전체를 지난다`);
  }
  for (const line of rowLines) {
    assert.deepEqual(line.spans, [{ start: 0, end: 200 }], `y=${line.y} 는 표 전체를 지난다`);
  }
});

test('맞닿은 칸 변은 잇고 벌어진 것은 끊는다', async () => {
  const { coalesceSpans } = await geometry();
  assert.deepEqual(
    coalesceSpans([{ start: 0, end: 50 }, { start: 50, end: 100 }]),
    [{ start: 0, end: 100 }],
    '변을 공유하면 한 구간',
  );
  assert.deepEqual(
    coalesceSpans([{ start: 0, end: 50 }, { start: 50.4, end: 100 }]),
    [{ start: 0, end: 100 }],
    '반올림 오차(0.5px 이내)는 잇는다',
  );
  assert.deepEqual(
    coalesceSpans([{ start: 0, end: 50 }, { start: 60, end: 100 }]),
    [{ start: 0, end: 50 }, { start: 60, end: 100 }],
    '실제로 벌어진 구간은 끊는다',
  );
  assert.deepEqual(
    coalesceSpans([{ start: 60, end: 100 }, { start: 0, end: 50 }]),
    [{ start: 0, end: 50 }, { start: 60, end: 100 }],
    '입력 순서와 무관하다',
  );
});

test('그리는 쪽은 표 전체 범위가 아니라 구간을 쓴다', () => {
  const renderer = readFileSync(join(rootDir, 'src/engine/table-resize-renderer.ts'), 'utf8');
  const start = renderer.indexOf('  showMarker(');
  assert.notEqual(start, -1, 'showMarker 를 찾지 못했다');
  const body = renderer.slice(start, renderer.indexOf('\n  /**', start + 1));

  assert.match(body, /line\.spans/, 'showMarker 는 경계 구간으로 그려야 한다');
  assert.doesNotMatch(
    body,
    /xStart|xEnd|yStart|yEnd/,
    '표 전체 범위는 적중 판정과 다른 출처다 — 두 번째 진실원을 남기지 않는다',
  );
});
