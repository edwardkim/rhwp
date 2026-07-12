// Issue #2021 계측 프로브 — 대형 표 셀 입력 1회의 wasm 호출별 시간 분해.
// 실행: CHROME_CDP=http://localhost:19222 node e2e/issue-2021-probe.mjs --mode=host
import { runTest, loadHwpFile, screenshot, assert } from './helpers.mjs';

runTest('issue-2021-probe', async ({ page }) => {
  const { pageCount } = await loadHwpFile(page, 'issue1949_giant_cell_nested_tables_perf.hwp');
  assert(pageCount >= 100, `문서 로드 (${pageCount}쪽)`);

  // wasm doc 메서드 계측 래퍼 설치
  await page.evaluate(() => {
    const doc = window.__wasm?.doc ?? window.__wasm;
    window.__probe = [];
    const targets = Object.getOwnPropertyNames(Object.getPrototypeOf(doc)).filter(
      (n) =>
        /cursorrect|insulttext|inserttext|pagecount|paginat|rendertree|caret/i.test(n) &&
        typeof doc[n] === 'function'
    );
    for (const n of targets) {
      const orig = doc[n].bind(doc);
      doc[n] = (...args) => {
        const t0 = performance.now();
        try {
          return orig(...args);
        } finally {
          const ms = performance.now() - t0;
          if (ms > 0.5) window.__probe.push({ fn: n, ms: +ms.toFixed(1) });
        }
      };
    }
    return targets;
  });

  // 편집 영역의 표 셀 클릭 (첫 페이지 셀 내부)
  const canvas = await page.$('#scroll-container canvas, #scroll-container svg, #scroll-container');
  const box = await canvas.boundingBox();
  await page.mouse.click(box.x + box.width / 2, box.y + 200);
  await new Promise((r) => setTimeout(r, 800));
  await page.evaluate(() => { window.__probe.length = 0; });

  // 입력 1회 + long task 관찰
  await page.evaluate(() => {
    window.__longTasks = [];
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) window.__longTasks.push(+e.duration.toFixed(0));
    }).observe({ entryTypes: ['longtask'] });
  });
  const t0 = Date.now();
  await page.keyboard.type('a', { delay: 0 });
  await new Promise((r) => setTimeout(r, 1500));

  // ── 후반 페이지 셀 (선형 탐색 최악 케이스) ──
  await page.evaluate(() => {
    const sc = document.querySelector('#scroll-container');
    sc.scrollTop = sc.scrollHeight * 0.8; // ~92쪽 부근
  });
  await new Promise((r) => setTimeout(r, 1200));
  const box2 = await (await page.$('#scroll-container')).boundingBox();
  await page.mouse.click(box2.x + box2.width / 2, box2.y + box2.height / 2);
  await new Promise((r) => setTimeout(r, 800));
  await page.evaluate(() => { window.__probe.length = 0; window.__longTasks.length = 0; });
  await page.keyboard.type('b', { delay: 0 });
  await new Promise((r) => setTimeout(r, 2500));
  const deep = await page.evaluate(() => ({
    probe: window.__probe.slice(0, 30),
    longTasks: window.__longTasks,
    cursor: window.__cursor?.position ?? null,
  }));
  console.log('=== [후반 페이지 셀] 입력 1회 분해 ===');
  for (const p of deep.probe) console.log(`  ${p.fn}: ${p.ms}ms`);
  console.log('=== long tasks ===', deep.longTasks);

  const result = await page.evaluate(() => ({
    probe: window.__probe.slice(0, 30),
    longTasks: window.__longTasks,
  }));
  console.log('=== 입력 1회 wasm 호출 분해 (>0.5ms) ===');
  for (const p of result.probe) console.log(`  ${p.fn}: ${p.ms}ms`);
  console.log('=== long tasks ===', result.longTasks, `(경과 ${Date.now() - t0}ms)`);
  await screenshot(page, '2021-01-after-input');
  assert(result.probe.length >= 0, '계측 수집 완료');
});
