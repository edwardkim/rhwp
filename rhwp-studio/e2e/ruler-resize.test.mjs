/**
 * #6187: resize 경계에서 눈금자 표시·배치·캡처된 그림을 확인한다.
 *
 * 이 검사는 viewport 변경 후 얻은 실제 화면 snapshot을 검사한다. 모든 compositor frame을
 * 관측하는 검사가 아니므로, Node의 reset→paint 동작 회귀 및 실제 창 드래그 검증과 함께 쓴다.
 * browser-client에서도 같은 read-only DOM/screenshot 검사를 실행하도록 driver를 주입할 수 있다.
 * 각 배율/쪽 이동 조합 시작 전에 문서 처음으로 이동해 마지막 편집 focus 쪽을 화면 안에 둔다.
 * focus 쪽이 화면 밖이면 세로 띠가 비는 것은 정상 계약이며 flicker 실패로 분류하지 않는다.
 */
import assert from 'node:assert/strict';
import { Buffer } from 'node:buffer';
import { pathToFileURL } from 'node:url';
import { PNG } from 'pngjs';

export function readRulerState() {
  const rect = element => {
    const r = element.getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height };
  };
  const editor = document.getElementById('editor-area');
  const scroll = document.getElementById('scroll-container');
  const h = document.getElementById('h-ruler');
  const v = document.getElementById('v-ruler');
  const corner = document.getElementById('ruler-corner');
  return {
    width: innerWidth, height: innerHeight,
    grid: getComputedStyle(editor).display === 'grid',
    visible: [h, v, corner].every(e => getComputedStyle(e).display !== 'none'
      && getComputedStyle(e).visibility !== 'hidden' && e.getBoundingClientRect().width > 0
      && e.getBoundingClientRect().height > 0),
    h: rect(h), v: rect(v), corner: rect(corner), scroll: rect(scroll),
    scrollWidth: scroll.clientWidth, scrollHeight: scroll.clientHeight,
    noOuterOverflow: document.documentElement.scrollWidth <= document.documentElement.clientWidth,
    status: document.getElementById('status-bar')?.textContent.replace(/\s+/g, ' ').trim() ?? '',
  };
}

/** screenshot의 눈금자 내부 색 다양성: 빈 단색 띠를 탐지하는 보조 지표, 정밀 OCR/geometry 검사는 아니다. */
export function inspectRulerScreenshot(bytes, state) {
  const png = PNG.sync.read(Buffer.from(bytes));
  const sx = png.width / state.width;
  const sy = png.height / state.height;
  const countColors = rect => {
    const colors = new Set();
    // 경계선과 이웃 콘텐츠를 제외한다. screenshot 밖으로 잘린 부분도 읽지 않는다.
    const left = Math.max(0, Math.ceil((rect.x + 2) * sx));
    const right = Math.min(png.width, Math.floor((rect.x + rect.width - 2) * sx));
    const top = Math.max(0, Math.ceil((rect.y + 2) * sy));
    const bottom = Math.min(png.height, Math.floor((rect.y + rect.height - 2) * sy));
    for (let y = top; y < bottom; y++) {
      for (let x = left; x < right; x++) {
        const i = (y * png.width + x) * 4;
        colors.add(`${png.data[i] >> 4},${png.data[i + 1] >> 4},${png.data[i + 2] >> 4}`);
      }
    }
    return colors.size;
  };
  return { hColors: countColors(state.h), vColors: countColors(state.v) };
}

export async function runResizeSnapshots(driver, { widths, height = 768, label = 'resize' }) {
  const samples = [];
  for (const width of widths) {
    await driver.setViewport({ width, height });
    const bytes = await driver.screenshot();
    const state = await driver.readState();
    const pixels = inspectRulerScreenshot(bytes, state);
    const near = (a, b) => Math.abs(a - b) <= 1;
    const aligned = near(state.h.x, state.scroll.x) && near(state.v.y, state.scroll.y)
      && near(state.h.y, state.corner.y) && near(state.v.x, state.corner.x)
      && near(state.h.width, state.scrollWidth) && near(state.v.height, state.scrollHeight)
      && near(state.h.height, 20) && near(state.v.width, 20)
      && near(state.corner.width, 20) && near(state.corner.height, 20);
    const sample = { label, index: samples.length, width, height, visible: state.visible,
      grid: state.grid, aligned, noOuterOverflow: state.noOuterOverflow, ...pixels, status: state.status };
    samples.push(sample);
    assert.equal(state.width, width, '지정한 viewport가 적용되어야 한다');
    assert.ok(state.visible && state.grid && aligned && state.noOuterOverflow,
      `${label}: 눈금자 표시·정렬 실패 ${JSON.stringify(sample)}`);
    assert.ok(pixels.hColors > 4 && pixels.vColors > 4,
      `${label}: 캡처에서 눈금자 그림을 확인하지 못함 ${JSON.stringify(sample)}`);
  }
  return samples;
}

// 저장소의 일반 E2E 진입점. browser-client에서는 위 driver 계약만 사용한다.
if (typeof process !== 'undefined' && process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { runTest } = await import('./helpers.mjs');
  runTest('눈금자 resize snapshot', async ({ page }) => {
    const base = process.env.VITE_URL || 'http://localhost:7700';
    await page.goto(`${base}/?url=/samples/exam_kor.hwp`, { waitUntil: 'networkidle0' });
    await page.waitForFunction(() => document.getElementById('status-bar')?.textContent.includes('20페이지'));
    // 첫 실행 안내는 실제 UI로 닫는다. 저장소 설정을 직접 덮지 않는다.
    const firstRun = await page.$('.skin-onboarding-body');
    if (firstRun) await page.keyboard.press('Escape');
    await page.focus('[aria-label="문서 편집 입력"]');
    await page.keyboard.down('Control');
    await page.keyboard.press('Home');
    await page.keyboard.up('Control');
    if (process.argv.includes('--mode=headless')) {
      assert.equal(await page.evaluate(() => devicePixelRatio), 1,
        'headless resize 검증은 실제 DPR 1 환경이어야 한다');
    }
    const samples = await runResizeSnapshots({
      setViewport: size => page.setViewport(size),
      screenshot: () => page.screenshot({ type: 'png' }),
      readState: () => page.evaluate(readRulerState),
    }, {
      widths: [...Array.from({ length: 10 }, () => [1023, 1024]).flat(), 767, 768, 807, 808, 961, 962, 375, 1280],
    });
    console.log(`RULER_RESIZE_SNAPSHOTS_OK ${samples.length} snapshots; compositor 전 프레임 검사는 아님`);
  });
}
