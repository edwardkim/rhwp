import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { isPointNearBoxBorder } from '../src/engine/table-border-hit.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const input = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');
const mouse = readFileSync(join(rootDir, 'src/engine/input-handler-mouse.ts'), 'utf8');

test('#2400 UI 114 텍스트 클릭은 현재 page fragment 경계가 아니다', () => {
  const point = { x: 142.8, y: 1057.3 };
  const firstFragment = { x: 75.6, y: 75.6, width: 597.2, height: 985.4 };
  const currentFragment = { x: 75.6, y: 75.6, width: 597.2, height: 1000.3 };

  assert.equal(isPointNearBoxBorder(point.x, point.y, firstFragment), true,
    '첫 fragment bbox를 사용하면 재현점이 잘못 경계로 판정되는 전제');
  assert.equal(isPointNearBoxBorder(point.x, point.y, currentFragment), false,
    '현재 page fragment에서는 텍스트 내부 클릭이어야 함');
});

test('현재 fragment의 실제 외곽 ±5px 계약은 유지한다', () => {
  const bbox = { x: 75.6, y: 75.6, width: 597.2, height: 1000.3 };
  const midX = bbox.x + bbox.width / 2;
  const midY = bbox.y + bbox.height / 2;

  assert.equal(isPointNearBoxBorder(bbox.x + 4.9, midY, bbox), true, 'left');
  assert.equal(isPointNearBoxBorder(bbox.x + bbox.width - 4.9, midY, bbox), true, 'right');
  assert.equal(isPointNearBoxBorder(midX, bbox.y + 4.9, bbox), true, 'top');
  assert.equal(isPointNearBoxBorder(midX, bbox.y + bbox.height - 4.9, bbox), true, 'bottom');
  assert.equal(isPointNearBoxBorder(midX, bbox.y + bbox.height - 5.1, bbox), false,
    'tolerance 밖의 내부 점');
  assert.equal(isPointNearBoxBorder(bbox.x - 6, bbox.y - 6, bbox), false,
    'tolerance 밖의 corner');
});

test('표 경계와 선택 표 hit 경로가 pointer page를 bbox 조회에 전달한다', () => {
  assert.match(input, /getTableBBoxAtPage\(sec, ppi, ci, pageIdx\)/,
    'isTableBorderClick은 현재 page fragment를 조회해야 함');
  assert.match(input, /isTableBorderClick\(pageIdx, pageX, pageY, sec, ppi, ci\)/,
    '표 외부 fallback도 현재 page를 전달해야 함');

  assert.match(mouse, /isTableBorderClick\(pi, px, py, hit\.sectionIndex/,
    '선택된 표의 셀 재진입 판정은 pointer page를 전달해야 함');
  assert.match(mouse, /isTableBorderClick\(pageIdx, pageX, pageY, hit\.sectionIndex/,
    '일반 셀 클릭 판정은 pointer page를 전달해야 함');
  assert.equal(mouse.match(/getTableBBoxAtPage\(ref\.sec, ref\.ppi, ref\.ci, pi\)/g)?.length, 2,
    '선택된 표의 이동 시작과 hover hit가 모두 현재 page bbox를 사용해야 함');
});
