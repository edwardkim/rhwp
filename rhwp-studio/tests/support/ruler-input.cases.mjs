import assert from 'node:assert/strict';
import test from 'node:test';
import { pointer, rulerFixture } from './ruler-harness.mjs';

for (const pointerType of ['touch', 'pen', '']) {
  for (const axis of ['h', 'v']) {
    test(`${pointerType || 'unknown'} ${axis}: 읽기 전용이며 기본 제스처를 취소하지 않는다`, t => {
      const f = rulerFixture(t);
      const point = axis === 'h' ? {} : { clientX: 10, clientY: 60 };
      const beforePaint = structuredClone(f[axis].paint);
      const down = pointer(f[axis], 'pointerdown', { pointerType, ...point });
      pointer(f.doc, 'pointermove', { pointerType, clientX: 80, clientY: 80 });
      pointer(f.doc, 'pointerup', { pointerType, clientX: 80, clientY: 80 });
      assert.equal(down.defaultPrevented, false);
      assert.equal(f.doc.listenerCount, 0);
      assert.equal(f[axis].captures.size, 0);
      assert.deepEqual(f.commits, []);
      assert.deepEqual(f[axis].paint, beforePaint, '읽기 전용 판정으로 표시를 지우지 않는다');
      assert.ok(beforePaint.labels.length > 0 && beforePaint.fills > 0, '숫자와 표시 핀 존재');
    });
  }
}

test('touch의 호환 mouse 이벤트로 핀 commit을 우회하지 못한다', t => {
  const f = rulerFixture(t);
  pointer(f.h, 'pointerdown', { pointerType: 'touch' });
  pointer(f.h, 'mousedown');
  pointer(f.doc, 'mousemove', { clientX: 80 });
  pointer(f.doc, 'mouseup', { clientX: 80 });
  assert.deepEqual(f.commits, []);
});

for (const [name, axis, down, move, expected] of [
  ['왼쪽 여백', 'h', {}, { clientX: 80 }, { kind: 'pageMargin', pageIdx: 0, marginKind: 'left', hwpunit: 4500 }],
  ['오른쪽 여백', 'h', { clientX: 280 }, { clientX: 260 }, { kind: 'pageMargin', pageIdx: 0, marginKind: 'right', hwpunit: 4500 }],
  ['들여쓰기', 'h', { clientY: 2 }, { clientX: 80, clientY: 2 }, { kind: 'paraProps', props: { indent: 3000 } }],
  ['위 여백', 'v', { clientX: 10, clientY: 60 }, { clientX: 10, clientY: 80 }, { kind: 'pageMargin', pageIdx: 0, marginKind: 'top', hwpunit: 4500 }],
  ['아래 여백', 'v', { clientX: 10, clientY: 580 }, { clientX: 10, clientY: 560 }, { kind: 'pageMargin', pageIdx: 0, marginKind: 'bottom', hwpunit: 4500 }],
]) {
  test(`375px 화면의 mouse ${name} drag는 기존 commit 경로를 한 번 호출한다`, t => {
    const f = rulerFixture(t);
    pointer(f[axis], 'pointerdown', down);
    assert.equal(f[axis].hasPointerCapture(1), true);
    pointer(f.doc, 'pointermove', move);
    pointer(f.doc, 'pointerup', move);
    pointer(f.doc, 'pointerup', move);
    assert.deepEqual(f.commits, [expected]);
    assert.equal(f.doc.listenerCount, 0);
    assert.equal(f.win.listenerCount, 0);
    assert.equal(f[axis].captures.size, 0);
  });
}

test('touch 이후 같은 세션의 mouse는 조작할 수 있다', t => {
  const f = rulerFixture(t);
  pointer(f.h, 'pointermove', { buttons: 0 });
  assert.equal(f.h.style.cursor, 'ew-resize');
  pointer(f.h, 'pointermove', { pointerType: 'touch' });
  assert.equal(f.h.style.cursor, 'default');
  pointer(f.h, 'pointerdown', { pointerType: 'touch' });
  pointer(f.h, 'pointerdown');
  pointer(f.doc, 'pointermove', { clientX: 80 });
  pointer(f.doc, 'pointerup');
  assert.equal(f.commits.length, 1);
});

test('클릭만 하거나 drag가 시작점으로 돌아오면 commit하지 않는다', t => {
  const f = rulerFixture(t);
  pointer(f.h, 'pointerdown');
  assert.equal(f.h.hasPointerCapture(1), true);
  pointer(f.doc, 'pointerup');
  pointer(f.h, 'pointerdown');
  pointer(f.doc, 'pointermove', { clientX: 80 });
  pointer(f.doc, 'pointermove');
  pointer(f.doc, 'pointerup');
  assert.deepEqual(f.commits, []);
});

test('오른쪽 버튼·비주 포인터·핀 밖 클릭은 drag를 시작하지 않는다', t => {
  const f = rulerFixture(t);
  for (const init of [{ button: 2 }, { isPrimary: false }, { clientX: 170, clientY: 10 }]) {
    pointer(f.h, 'pointerdown', init);
    assert.equal(f.doc.listenerCount, 0);
    assert.equal(f.h.captures.size, 0);
  }
});

test('다른 pointer와 다른 축의 down/move/up/cancel은 진행 중인 drag를 바꾸지 않는다', t => {
  const f = rulerFixture(t);
  pointer(f.h, 'pointerdown');
  pointer(f.v, 'pointerdown', { clientX: 10, clientY: 60 });
  for (const init of [{ pointerId: 2 }, { pointerType: 'touch' }]) {
    pointer(f.doc, 'pointermove', { ...init, clientX: 100 });
    pointer(f.doc, 'pointerup', init);
    pointer(f.doc, 'pointercancel', init);
  }
  assert.deepEqual(f.commits, []);
  assert.equal(f.h.hasPointerCapture(1), true);
  pointer(f.doc, 'pointermove', { clientX: 80 });
  pointer(f.doc, 'pointerup');
  assert.deepEqual(f.commits, [{ kind: 'pageMargin', pageIdx: 0, marginKind: 'left', hwpunit: 4500 }]);
});

test('pointerId 0도 유효한 drag로 추적한다', t => {
  const f = rulerFixture(t);
  pointer(f.h, 'pointerdown', { pointerId: 0 });
  assert.equal(f.h.hasPointerCapture(0), true);
  pointer(f.doc, 'pointermove', { pointerId: 0, clientX: 80 });
  pointer(f.doc, 'pointerup', { pointerId: 0 });
  assert.equal(f.commits.length, 1);
  assert.equal(f.h.captures.size, 0);
});

test('commit callback이 실패해도 drag와 전역 listener는 먼저 정리된다', t => {
  const f = rulerFixture(t);
  f.ruler.onCommitPin = () => { throw new Error('commit failed'); };
  pointer(f.h, 'pointerdown');
  pointer(f.doc, 'pointermove', { clientX: 80 });
  // EventTarget의 비동기 예외 보고 대신 실제 종료 handler의 예외와 정리 순서를 확인한다.
  assert.throws(() => f.ruler.onPinDragUp({ pointerId: 1, pointerType: 'mouse' }), /commit failed/);
  assert.equal(f.doc.listenerCount, 0);
  assert.equal(f.win.listenerCount, 0);
  assert.equal(f.h.captures.size, 0);
  pointer(f.doc, 'pointerup');
});

for (const reason of ['pointercancel', 'lostpointercapture', 'blur', 'buttons-released', 'dispose']) {
  test(`${reason}: drag를 commit 없이 취소하고 listener/capture를 정리한다`, t => {
    const f = rulerFixture(t);
    pointer(f.h, 'pointerdown');
    pointer(f.doc, 'pointermove', { clientX: 80 });
    assert.equal(f.h.hasPointerCapture(1), true, '취소 전에 실제 drag가 시작되어야 한다');
    if (reason === 'blur') f.win.dispatchEvent(new Event('blur'));
    else if (reason === 'dispose') f.ruler.dispose();
    else if (reason === 'buttons-released') pointer(f.doc, 'pointermove', { buttons: 0 });
    else pointer(reason === 'lostpointercapture' ? f.h : f.doc, reason);
    pointer(f.doc, 'pointerup');
    assert.deepEqual(f.commits, []);
    assert.equal(f.doc.listenerCount, 0);
    assert.equal(f.win.listenerCount, 0);
    assert.equal(f.h.captures.size, 0);
    if (reason === 'dispose') {
      assert.equal(f.h.listenerCount + f.v.listenerCount, 0);
      assert.equal(f.frames.size, 0);
    }
  });
}
