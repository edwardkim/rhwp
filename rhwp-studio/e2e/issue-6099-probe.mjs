// [#6099] 일회성 프로브 — 90° 회전 그림의 DOM frame/<img> 실측 + 스크린샷.
import { launchBrowser, createPage, closePage, closeBrowser, loadApp, loadHwpFile, waitForCanvas } from './helpers.mjs';

const name = process.argv[2]; // samples/ 하위 상대 경로
const shot = process.argv[3];

const browser = await launchBrowser();
const page = await createPage(browser, 1400, 1000);
try {
  await loadApp(page);
  await page.evaluate(() => {
    for (const btn of document.querySelectorAll('button')) {
      if (btn.textContent && btn.textContent.includes('시작하기')) { btn.click(); return; }
    }
  });
  await loadHwpFile(page, name);
  await waitForCanvas(page);
  await new Promise((r) => setTimeout(r, 2500));
  const probe = await page.evaluate(() => {
    const layer = document.querySelector('[data-rhwp-overlay^="flow-images"]');
    if (!layer) return { error: 'no flow-images layer' };
    const out = [];
    for (const img of layer.querySelectorAll('img')) {
      const frame = img.parentElement;
      const fr = frame.getBoundingClientRect();
      const ir = img.getBoundingClientRect();
      out.push({
        frameStyle: { w: frame.style.width, h: frame.style.height, tf: frame.style.transform },
        frameRect: { w: +fr.width.toFixed(1), h: +fr.height.toFixed(1) },
        imgRect: { w: +ir.width.toFixed(1), h: +ir.height.toFixed(1) },
      });
    }
    return { images: out };
  });
  console.log(JSON.stringify(probe, null, 1));
  if (shot) await page.screenshot({ path: shot, fullPage: false });
} finally {
  await closePage(page);
  await closeBrowser(browser);
}
