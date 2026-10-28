import assert from 'node:assert/strict';
import test from 'node:test';
import { pointer, rulerFixture } from './ruler-harness.mjs';

const operations = (f, kind) => f.operations.filter(op => op.kind === kind);
const resets = f => operations(f, 'reset').map(({ axis, dimension, value }) => ({ axis, dimension, value }));
const assertPainted = f => {
  for (const axis of ['h', 'v']) {
    assert.equal(f[axis].bitmapBlank, false, `${axis}: bitmap이 공백이면 안 된다`);
    assert.ok(f[axis].paint.labels.length > 0 && f[axis].paint.strokes > 0,
      `${axis}: 배경만이 아니라 숫자와 눈금을 그려야 한다`);
  }
};

test('resize 이벤트와 다음 rAF 사이에 두 눈금자의 기존 bitmap을 지우지 않는다', t => {
  const f = rulerFixture(t);
  assertPainted(f);
  f.operations.length = 0;
  f.container.clientWidth = 1004;
  f.container.clientHeight = 800;
  f.bus.emit('viewport-resize', 1004, 800);
  assertPainted(f); // 구 구현은 이벤트 callback에서 width/height를 대입해 여기서 실패한다.
  assert.deepEqual(f.operations, [], '크기 대입·CSS 변경·paint는 아직 실행하지 않는다');
  assert.equal(f.frames.size, 1);

  assert.equal(f.flush(), 1);
  assertPainted(f);
  assert.equal(f.h.width, 1004);
  assert.equal(f.v.height, 800);
  assert.equal(f.frames.size, 0, '다시 그리기를 다음 프레임으로 미루지 않는다');
  const frameIds = new Set(f.operations.map(op => op.frame));
  assert.equal(frameIds.size, 1, '크기 대입과 두 축 paint가 같은 callback에서 실행된다');
  assert.equal(frameIds.has(null), false, '이벤트 callback에서 bitmap을 지우지 않는다');
  for (const axis of ['h', 'v']) {
    const lastReset = f.operations.findLastIndex(op => op.axis === axis && op.kind === 'reset');
    const firstPaint = f.operations.findIndex(op => op.axis === axis && op.kind === 'paint');
    assert.ok(lastReset >= 0 && firstPaint > lastReset, '크기 변경 후 같은 갱신에서 paint한다');
  }
});

test('같은 크기의 연속 resize는 backing store와 CSS 크기를 다시 대입하지 않는다', t => {
  const f = rulerFixture(t);
  f.operations.length = 0;
  for (let i = 0; i < 10; i++) f.bus.emit('viewport-resize', 355, 700);
  assert.equal(f.frames.size, 1);
  f.flush();
  assert.deepEqual(resets(f), []);
  assert.deepEqual(operations(f, 'css'), []);
  assert.equal(operations(f, 'transform').length, 2, '축당 한 번만 그린다');
  assertPainted(f);
});

test('연속 resize는 마지막 이벤트 이후의 최신 실제 container geometry를 읽는다', t => {
  const f = rulerFixture(t);
  f.operations.length = 0;
  for (const width of [1003, 1004, 747, 748, 1004]) {
    f.container.clientWidth = width;
    f.bus.emit('viewport-resize', width, 700);
  }
  // 앞선 subscriber의 reflow나 도구 영역 변경은 이벤트 인수보다 나중에 확정될 수 있다.
  f.container.clientWidth = 1100;
  f.container.clientHeight = 650;
  f.scrollContent.offsetLeft = 90;
  assert.equal(f.frames.size, 1);
  f.flush();
  assert.deepEqual(resets(f), [
    { axis: 'h', dimension: 'width', value: 1100 },
    { axis: 'v', dimension: 'height', value: 650 },
  ]);
  assert.equal(f.ruler.hPins.find(pin => pin.kind === 'pageMarginLeft').x, 130);
  assertPainted(f);
});

for (const [property, value, expected] of [
  ['clientWidth', 600, { axis: 'h', dimension: 'width', value: 600 }],
  ['clientHeight', 900, { axis: 'v', dimension: 'height', value: 900 }],
]) {
  test(`${property}만 변경되면 해당 bitmap 차원만 재설정한다`, t => {
    const f = rulerFixture(t);
    f.operations.length = 0;
    f.container[property] = value;
    f.bus.emit('viewport-resize');
    f.flush();
    assert.deepEqual(resets(f), [expected]);
    assert.equal(operations(f, 'css').length, 1);
    assertPainted(f);
  });
}

for (const dpr of [1.25, 1.333, 2]) {
  test(`DPR ${dpr}: callback 시점의 DPR로 backing 크기와 두 축 transform을 맞춘다`, t => {
    const f = rulerFixture(t);
    f.operations.length = 0;
    f.bus.emit('viewport-resize');
    f.win.devicePixelRatio = dpr; // 이벤트에서 미리 계산한 DPR을 쓰면 틀린다.
    f.flush();
    assert.deepEqual([f.h.width, f.h.height, f.v.width, f.v.height],
      [355, 20, 20, 700].map(size => Math.round(size * dpr)));
    assert.deepEqual(operations(f, 'transform').map(op => op.value),
      [[dpr, 0, 0, dpr, 0, 0], [dpr, 0, 0, dpr, 0, 0]]);
    assert.deepEqual(operations(f, 'css'), [], 'DPR만 변경되면 CSS 크기는 유지한다');
    assertPainted(f);
  });
}

test('초기 생성도 크기 설정만 하고 끝나지 않고 한 갱신 안에서 두 축을 그린다', t => {
  const f = rulerFixture(t, { initialize: false, context: false });
  assert.deepEqual(f.operations, [], '생성자는 초기 갱신만 예약한다');
  assert.equal(f.frames.size, 1);
  f.flush();
  assert.equal(f.h.bitmapBlank || f.v.bitmapBlank, false, '다른 이벤트 없이도 두 축 배경을 그린다');
  assert.deepEqual(f.h.paint.labels, [], '편집/viewport 문맥이 없으면 임의의 쪽 눈금을 만들지 않는다');
  assert.deepEqual(f.v.paint.labels, []);
  assert.equal(new Set(f.operations.map(op => op.frame)).size, 1);
  assert.equal(f.frames.size, 0);
});

test('서로 다른 프레임의 연속 resize도 이벤트 경계에서는 마지막 그림을 유지한다', t => {
  const f = rulerFixture(t);
  for (let round = 0; round < 10; round++) {
    for (const width of [1003, 1004, 747, 748]) {
      f.operations.length = 0;
      f.container.clientWidth = width;
      f.bus.emit('viewport-resize', width, 700);
      assertPainted(f);
      assert.deepEqual(f.operations, []);
      assert.equal(f.flush(), 1);
      assert.equal(f.h.width, width);
      assert.equal(f.ruler.focusedPageIndex, 0);
      assertPainted(f);
    }
  }
  assert.deepEqual(f.commits, []);
});

test('한 갱신에서 DPR을 한 번만 읽어 크기와 두 축 paint에 공유한다', t => {
  const f = rulerFixture(t);
  let reads = 0;
  Object.defineProperty(f.win, 'devicePixelRatio', { get: () => { reads++; return 1.25; } });
  f.bus.emit('viewport-resize');
  f.flush();
  assert.equal(reads, 1);
  assertPainted(f);
});

test('0 크기에서 문서가 로드되면 새 크기로 다시 그린다', t => {
  const f = rulerFixture(t, { width: 0, height: 0 });
  f.operations.length = 0;
  f.container.clientWidth = 355;
  f.container.clientHeight = 700;
  f.bus.emit('document-view-loaded');
  assert.deepEqual(f.operations, [], '문서 로드 알림도 bitmap을 먼저 지우지 않는다');
  f.flush();
  assert.equal(f.h.width, 355);
  assert.equal(f.v.height, 700);
  assertPainted(f);
});

test('문서 교체 중 없는 쪽은 핀을 제거하고 로드 알림만으로 새 용지 geometry를 그린다', t => {
  const f = rulerFixture(t);
  f.bus.emit('focused-page-changed', null);
  f.bus.emit('active-page-changed', null);
  f.wasm.pageCount = 0;
  f.bus.emit('document-changed');
  f.flush();
  assert.deepEqual(f.ruler.hPins, []);
  assert.deepEqual(f.ruler.vPins, []);
  assert.deepEqual(f.h.paint.labels, []);
  assert.deepEqual(f.v.paint.labels, []);

  Object.assign(f.page, { width: 400, height: 500, bodyLeft: 60, bodyRight: 370, marginTop: 25, marginBottom: 30 });
  f.wasm.pageCount = 1;
  f.bus.emit('active-page-changed', { pageIndex: 0, source: 'viewport' });
  f.flush();
  // 동일 zoom/scroll/focus인 재열기에도 document-view-loaded 자체가 다시 그려야 한다.
  f.page.bodyLeft = 70;
  f.operations.length = 0;
  f.bus.emit('document-view-loaded');
  f.flush();
  assert.deepEqual(resets(f), [], '같은 container 크기의 재열기는 bitmap 재할당이 필요 없다');
  assert.deepEqual(f.ruler.hPins, [
    { kind: 'pageMarginLeft', x: 70, y: 20 }, { kind: 'pageMarginRight', x: 370, y: 20 },
  ]); // 이전 문서의 문단 핀은 viewport fallback에 남지 않는다.
  assert.deepEqual(f.ruler.vPins, [
    { kind: 'top', y: 25, pageIdx: 0 }, { kind: 'bottom', y: 470, pageIdx: 0 },
  ]);
  assertPainted(f);
});

for (const event of ['viewport-scroll', 'zoom-changed', 'theme-changed', 'document-view-changed']) {
  test(`${event}: 예약된 갱신에서도 최신 크기를 반영하고 같은 크기는 재설정하지 않는다`, t => {
    const f = rulerFixture(t);
    f.container.clientWidth = 500;
    f.operations.length = 0;
    f.bus.emit(event);
    f.flush();
    assert.equal(f.h.width, 500);
    assertPainted(f);
    f.operations.length = 0;
    f.bus.emit(event);
    f.flush();
    assert.deepEqual(resets(f), []);
    assertPainted(f);
  });
}

test('resize와 zoom/scroll/theme 갱신은 최신 focus와 geometry를 두 축에 함께 적용한다', t => {
  const f = rulerFixture(t);
  f.wasm.pageCount = f.scroll.pageCount = 2;
  const secondPage = { ...f.page, width: 500, height: 650, bodyLeft: 80, bodyRight: 450 };
  f.wasm.getPageInfo = index => index === 1 ? secondPage : f.page;
  f.scroll.getPageLeftResolved = index => index * 550;
  f.scroll.getPageOffset = index => index * 750;
  f.bus.emit('focused-page-changed', 1);
  f.bus.emit('active-page-changed', { pageIndex: 0, source: 'viewport' });
  f.viewport.getZoom = () => 0.5;
  f.viewport.getScrollX = () => 500;
  f.viewport.getScrollY = () => 700;
  f.operations.length = 0;
  for (const event of ['zoom-changed', 'viewport-resize', 'viewport-scroll', 'theme-changed']) f.bus.emit(event);
  f.flush();
  assert.equal(f.ruler.focusedPageIndex, 1);
  assert.equal(f.ruler.hPageIdx, 1);
  assert.equal(f.ruler.hPins.find(pin => pin.kind === 'pageMarginLeft').x, 90);
  assert.deepEqual(f.ruler.vPins.map(pin => [pin.pageIdx, pin.y]), [[1, 70], [1, 355]]);
  assert.equal(operations(f, 'transform').length, 2);
  assert.deepEqual(f.commits, [], 'resize는 문서를 변경하지 않는다');
  assertPainted(f);
});

test('dispose는 대기 중 resize·paint와 drag listener를 제거한다', t => {
  const f = rulerFixture(t);
  pointer(f.h, 'pointerdown');
  pointer(f.doc, 'pointermove', { clientX: 80 });
  f.operations.length = 0;
  f.container.clientWidth = 500;
  f.bus.emit('viewport-resize');
  f.ruler.dispose();
  f.operations.length = 0;
  for (const event of ['viewport-resize', 'document-view-loaded', 'theme-changed']) f.bus.emit(event);
  assert.equal(f.flush(), 0);
  assert.deepEqual(f.operations, []);
  assert.equal(f.doc.listenerCount + f.win.listenerCount + f.h.listenerCount + f.v.listenerCount, 0);
  assert.equal(f.h.captures.size, 0);
  assert.deepEqual(f.commits, []);
});
