import assert from 'node:assert/strict';
import test from 'node:test';
import { PNG } from 'pngjs';
import { inspectRulerScreenshot, runResizeSnapshots } from '../e2e/ruler-resize.test.mjs';

const state = {
  width: 100, height: 100, grid: true, visible: true, noOuterOverflow: true,
  h: { x: 20, y: 0, width: 80, height: 20 },
  v: { x: 0, y: 20, width: 20, height: 80 },
  corner: { x: 0, y: 0, width: 20, height: 20 },
  scroll: { x: 20, y: 20, width: 80, height: 80 }, scrollWidth: 80, scrollHeight: 80, status: '100%',
};

function picture(ink: (x: number, y: number) => boolean) {
  const png = new PNG({ width: 100, height: 100 });
  for (let y = 0; y < 100; y++) {
    for (let x = 0; x < 100; x++) {
      const offset = (y * 100 + x) * 4;
      const gray = ink(x, y) ? ((x + y) % 16) * 16 : 80;
      png.data[offset] = png.data[offset + 1] = png.data[offset + 2] = gray;
      png.data[offset + 3] = 255;
    }
  }
  return PNG.sync.write(png);
}

function driver(bytes: Uint8Array, override = {}) {
  return { setViewport: async () => {}, screenshot: async () => bytes,
    readState: async () => ({ ...state, ...override }) };
}

test('빈 눈금자 띠는 문서 영역에 그림이 있어도 캡처 검사에서 실패한다', async () => {
  for (const bytes of [picture(() => false), picture((x, y) => x >= 20 && y >= 20)]) {
    assert.deepEqual(inspectRulerScreenshot(bytes, state), { hColors: 1, vColors: 1 });
    await assert.rejects(runResizeSnapshots(driver(bytes), { widths: [100], height: 100 }), /그림을 확인하지 못함/);
  }
});

test('한 축만 그려진 화면은 통과시키지 않는다', async () => {
  for (const ink of [(x: number, y: number) => x >= 20 && y < 20,
    (x: number, y: number) => x < 20 && y >= 20]) {
    await assert.rejects(runResizeSnapshots(driver(picture(ink)), { widths: [100], height: 100 }), /그림을 확인하지 못함/);
  }
});

test('두 눈금자에 그림이 있고 표시·정렬이 맞으면 snapshot을 기록한다', async () => {
  const bytes = picture((x, y) => x < 20 || y < 20);
  const samples = await runResizeSnapshots(driver(bytes), { widths: [100, 100], height: 100 });
  assert.equal(samples.length, 2);
  assert.ok(samples.every(sample => sample.hColors > 4 && sample.vColors > 4));
});

test('그림이 있어도 숨김·grid 이탈·정렬 오류·outer overflow는 실패한다', async () => {
  const bytes = picture((x, y) => x < 20 || y < 20);
  for (const override of [{ visible: false }, { grid: false }, { noOuterOverflow: false },
    { h: { ...state.h, x: 30 } }]) {
    await assert.rejects(runResizeSnapshots(driver(bytes, override), { widths: [100], height: 100 }), /표시·정렬 실패/);
  }
});
