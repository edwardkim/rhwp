import test from 'node:test';
import assert from 'node:assert/strict';

import {
  horizontalPinCommit,
  pageMarginPinX,
  paraIndentPinX,
  pxToHwpunit,
  pxToParaRaw,
  type HPinDropContext,
} from '../src/view/ruler-pin-geometry.ts';

// 가로 눈금자 핀의 소유 규칙과 좌표 역함수.
//
// 결함 형태: △(아래 핀)는 쪽 여백 경계 자리에 그려지면서 커밋은 ParaShape.marginLeft 로
// 했다 — 쪽 여백을 옮기려던 드래그가 커서가 놓인 문단 하나만 움직였고, 같은 분기가
// marginLeft 를 쓰는 바람에 ▽(들여쓰기)와도 값이 섞였다. 그리는 식과 되돌리는 식이 서로
// 다른 파일에 있어 한쪽만 고쳐도 컴파일이 통과한 것이 원인이라, 두 식을 한 모듈에 두고
// 여기서 왕복으로 묶는다.

/** A4 100% 기준 — 쪽 왼쪽 끝 x=313, 표시 너비 793.7, 쪽 여백 68px */
function ctx(over: Partial<HPinDropContext> = {}): HPinDropContext {
  return {
    zoom: 1,
    pageIdx: 0,
    pageLeft: 313,
    pageDisplayWidth: 793.7,
    refLeft: 381,
    paraMarginLeftPx: 0,
    ...over,
  };
}

test('△는 쪽 여백만 커밋한다 — 문단 필드가 페이로드에 없다', () => {
  const c = ctx();
  const commit = horizontalPinCommit('pageMarginLeft', pageMarginPinX('left', c.pageLeft, c.pageDisplayWidth, 127, c.zoom), c);

  assert.equal(commit.kind, 'pageMargin');
  assert.deepEqual(commit, {
    kind: 'pageMargin',
    pageIdx: 0,
    marginKind: 'left',
    hwpunit: pxToHwpunit(127),
  });
});

test('▽는 문단 들여쓰기만 커밋한다 — marginLeft 를 같이 쓰지 않는다', () => {
  const c = ctx({ paraMarginLeftPx: 48.4 });
  const commit = horizontalPinCommit('paraIndent', paraIndentPinX(c.refLeft, 48.4, 30, c.zoom), c);

  assert.equal(commit.kind, 'paraProps');
  assert(commit.kind === 'paraProps');
  assert.deepEqual(Object.keys(commit.props), ['indent']);
  assert.equal(commit.props.indent, pxToParaRaw(30));
  // marginLeft 가 페이로드에 섞이면 눈금자가 문단 여백의 두 번째 주인이 된다.
  assert.equal(commit.props.marginLeft, undefined);
  assert.equal(commit.props.marginRight, undefined);
});

test('그리는 식과 되돌리는 식이 왕복한다 (좌/우 쪽 여백, zoom 무관)', () => {
  for (const zoom of [1, 0.8694289241382309, 1.5]) {
    for (const marginPx of [0, 68, 127.4]) {
      const c = ctx({ zoom });

      const leftDrop = pageMarginPinX('left', c.pageLeft, c.pageDisplayWidth, marginPx, zoom);
      const left = horizontalPinCommit('pageMarginLeft', leftDrop, c);
      assert(left.kind === 'pageMargin');
      assert.equal(left.marginKind, 'left');
      assert.equal(left.hwpunit, pxToHwpunit(marginPx), `left zoom=${zoom} margin=${marginPx}`);

      const rightDrop = pageMarginPinX('right', c.pageLeft, c.pageDisplayWidth, marginPx, zoom);
      const right = horizontalPinCommit('pageMarginRight', rightDrop, c);
      assert(right.kind === 'pageMargin');
      assert.equal(right.marginKind, 'right');
      assert.equal(right.hwpunit, pxToHwpunit(marginPx), `right zoom=${zoom} margin=${marginPx}`);
    }
  }
});

test('그리는 식과 되돌리는 식이 왕복한다 (첫 줄 들여쓰기)', () => {
  for (const zoom of [1, 0.8694289241382309, 1.5]) {
    for (const marginLeft of [0, 48.4]) {
      for (const indent of [0, 30, 120.5]) {
        const c = ctx({ zoom, paraMarginLeftPx: marginLeft });
        const drop = paraIndentPinX(c.refLeft, marginLeft, indent, zoom);
        const commit = horizontalPinCommit('paraIndent', drop, c);
        assert(commit.kind === 'paraProps');
        assert.equal(
          commit.props.indent,
          pxToParaRaw(indent),
          `zoom=${zoom} marginLeft=${marginLeft} indent=${indent}`,
        );
      }
    }
  }
});

test('△ 는 쪽 기준, ▽ 는 문단 기준 — 서로의 기준 좌표에 영향받지 않는다', () => {
  // 셀 안이면 refLeft 는 셀 왼쪽이라 pageLeft 와 무관하게 움직인다. 그래도 △는 쪽 좌표만,
  // ▽는 refLeft 만 본다.
  const a = ctx({ refLeft: 381 });
  const b = ctx({ refLeft: 615 });

  assert.deepEqual(
    horizontalPinCommit('pageMarginLeft', 440, a),
    horizontalPinCommit('pageMarginLeft', 440, b),
    '△ 결과가 문단/셀 기준 좌표에 끌려다닌다',
  );

  const indentInCell = horizontalPinCommit('paraIndent', 655, b);
  assert(indentInCell.kind === 'paraProps');
  assert.equal(indentInCell.props.indent, pxToParaRaw(40), '▽ 가 셀 기준을 쓰지 않는다');
});

test('내어쓰기(indent<0)의 ▽ 는 marginLeft 자리에 그대로 있다', () => {
  // renderer/layout/paragraph_layout.rs: 내어쓰기는 첫 줄이 아니라 둘째 줄부터를 밀어낸다.
  assert.equal(
    paraIndentPinX(381, 48.4, -40, 1),
    paraIndentPinX(381, 48.4, 0, 1),
  );
});

test('0 아래로는 clamp — 종이 밖 여백과 marginLeft 왼쪽의 첫 줄은 만들 수 없다', () => {
  const c = ctx({ paraMarginLeftPx: 48.4 });

  // 쪽 왼쪽 끝보다 왼쪽에 떨궈도 음수 여백이 되지 않는다
  assert.equal((horizontalPinCommit('pageMarginLeft', c.pageLeft - 50, c) as { hwpunit: number }).hwpunit, 0);
  // 오른쪽 △ 를 종이 오른쪽 끝 밖으로 떨궈도 마찬가지
  assert.equal(
    (horizontalPinCommit('pageMarginRight', c.pageLeft + c.pageDisplayWidth + 50, c) as { hwpunit: number }).hwpunit,
    0,
  );

  // ▽ 를 문단 왼쪽 여백보다 왼쪽에 떨구면 들여쓰기 0 (음수 indent = 내어쓰기는 첫 줄 핀의 뜻이 아니다)
  const indent = horizontalPinCommit('paraIndent', c.refLeft, c);
  assert(indent.kind === 'paraProps');
  assert.equal(indent.props.indent, 0);
});

test('HWPUNIT 은 가까운 쪽으로 반올림한다 — 안 움직인 드래그가 값을 깎지 않게', () => {
  assert.equal(pxToHwpunit(68), 5100);
  assert.equal(pxToParaRaw(48.4), 7260);

  // 0 방향 절삭이면 여기서 9554 가 나온다 (되돌린 px 가 1 ulp 작아서). 반올림이라 9555.
  const px = (313 + 127.4) - 313;
  assert.notEqual(px, 127.4, '이 회귀를 재현하려면 px 에 부동소수 오차가 있어야 한다');
  assert.equal(pxToHwpunit(px), pxToHwpunit(127.4));
});
