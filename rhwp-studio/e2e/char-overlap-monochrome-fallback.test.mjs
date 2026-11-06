// 관계형 회귀: 독립 Print가 검정인 겹침 기호는 단색을 유지하고 파란 덧말은 색상을 보존한다.
// 주석 갱신: 2026-11-06 09:56 KST. 직전 파일 변경: d0c751965a158f767791f7e70cfe4b3ebdc73970 (2026-11-06 09:34:59 +09:00).
// 실제 Studio 측정: 2026-11-06 09:35:39 KST, 각 1쪽, 96 DPI·Print·색상 경계 보정 전 2px raw. 직접 exporter와 실제 Studio 결과는 근거 문서에서 구분한다.
// 2026-11-06 KST: PR #8616 HWPX 두 원본 각 1쪽 Native/fresh WASM raw 100.00% (96 DPI).
// 근거: mydocs/pr/assets/semanticist21_20261105/canvas-symbol-fallback-analysis.md.
// 특정 설치 글꼴·픽셀 좌표에 의존하지 않으며 실제 PageRenderer의 글꼴 후보 소비를 검사한다.
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { runTest, loadApp, loadHwpFile, captureCanvasScreenshot } from './helpers.mjs';

const output = process.env.RHWP_CANVAS_TEST_OUTPUT;
await runTest('겹침 모양의 단색과 덧말의 원래 색상 보존', async ({ page }) => {
  await page.evaluateOnNewDocument(() => {
    window.__recyclingPaints = [];
    const fill = CanvasRenderingContext2D.prototype.fillText;
    CanvasRenderingContext2D.prototype.fillText = function (text, ...args) {
      if (text === '♻') {
        window.__recyclingPaints.push({ text, font: this.font, color: this.fillStyle });
      }
      return fill.call(this, text, ...args);
    };
  });
  await loadApp(page, '?renderer=canvas2d&renderProfile=print');
  const results = [];
  for (const [file, monochrome] of [
    ['char_overlap/hancom-mac-shapes.hwpx', true],
    ['dutmal/hancom-mac-dutmal.hwpx', false],
  ]) {
    await loadHwpFile(page, file);
    const result = await page.evaluate(() => {
      const renderer = window.__canvasView.pageRenderer;
      renderer.cancelAll?.();
      document.getElementById('symbol-proof')?.remove();
      const container = document.createElement('div');
      container.id = 'symbol-proof';
      Object.assign(container.style, { position: 'fixed', left: '0', top: '0',
        background: '#fff', zIndex: '2147483647', overflow: 'hidden' });
      const canvas = document.createElement('canvas');
      container.appendChild(canvas);
      document.body.appendChild(container);
      window.__recyclingPaints = [];
      const rendered = renderer.renderPage(0, canvas, 1, 1, 1)?.renderedCanvas ?? canvas;
      Object.assign(container.style, { width: `${rendered.width}px`, height: `${rendered.height}px` });
      const bytes = rendered.getContext('2d').getImageData(0, 0, rendered.width, rendered.height).data;
      let coloredPixels = 0, inkPixels = 0;
      for (let i = 0; i < bytes.length; i += 4) {
        const rgb = [bytes[i], bytes[i + 1], bytes[i + 2]];
        if (Math.min(...rgb) < 200) inkPixels++;
        if (Math.max(...rgb) - Math.min(...rgb) > 25) coloredPixels++;
      }
      return { coloredPixels, inkPixels, paints: window.__recyclingPaints,
        backend: renderer.getActiveBackend?.() ?? 'canvas2d' };
    });
    assert.ok(result.inkPixels > 0, '빈 출력은 통과시키지 않는다');
    if (monochrome) {
      assert.equal(result.coloredPixels, 0, '원본의 검정 기호가 컬러 이모지로 변하면 안 된다');
      assert.ok(result.paints.length > 0, '실제 재활용 기호 paint를 실행한다');
      for (const paint of result.paints) {
        assert.equal(paint.color, '#000000');
        assert.ok(paint.font.indexOf('함초롬바탕') >= 0);
        assert.ok(paint.font.indexOf('함초롬바탕') < paint.font.indexOf('D2Coding'));
        // 글꼴 프로그램이 없는 최소 환경에도 같은 후보·색상 계약을 적용한다.
        // 실제 glyph 크기는 올바른 공급 환경의 독립 PDF 확대 판독으로 검증한다.
        assert.ok(paint.font.indexOf('D2Coding') < paint.font.indexOf('Segoe UI Emoji'));
      }
    } else {
      assert.ok(result.coloredPixels > 0, '원래 파란 덧말을 일괄 흑백 처리하면 안 된다');
    }
    results.push({ file, monochrome, ...result });
    if (output) {
      mkdirSync(output, { recursive: true });
      await captureCanvasScreenshot(page, path.join(output, monochrome ? 'overlap.png' : 'ruby.png'),
        '실제 Studio 출력', '#symbol-proof');
    }
    await page.evaluate(() => {
      window.__canvasView.pageRenderer.cancelReRender?.(0);
      document.getElementById('symbol-proof')?.remove();
    });
  }
  if (output) writeFileSync(path.join(output, 'result.json'), JSON.stringify(results, null, 2) + '\n');
  console.log(JSON.stringify(results));
}, { skipLoadApp: true });
