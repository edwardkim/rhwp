/**
 * E2E 테스트 헬퍼 — Puppeteer + Chrome CDP
 *
 * 모드 (CLI 옵션 --mode):
 *   --mode=host    : 호스트 Windows Chrome CDP에 연결 (기본)
 *   --mode=headless: WSL2 내부 headless Chrome 실행
 *
 * 예시:
 *   node e2e/text-flow.test.mjs                  # 호스트 Chrome CDP
 *   node e2e/text-flow.test.mjs --mode=headless  # headless Chrome
 */
import puppeteer from 'puppeteer-core';
import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';
import { existsSync, readFileSync, readdirSync } from 'fs';
import { fileURLToPath } from 'url';
import os from 'os';
import path from 'path';
import { TestReporter } from './report-generator.mjs';

const CHROME_CDP = process.env.CHROME_CDP || 'http://172.21.192.1:19222';
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';
const REPORT_DIR = '../output/e2e';
const RHWP_ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)), '..');

function resolveChromePath() {
  const envPath = process.env.CHROME_PATH || process.env.PUPPETEER_EXECUTABLE_PATH;
  if (envPath && existsSync(envPath)) return envPath;

  const cacheRoot = path.join(os.homedir(), '.cache', 'puppeteer', 'chrome');
  if (!existsSync(cacheRoot)) return envPath || '';

  const entries = readdirSync(cacheRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(cacheRoot, entry.name, 'chrome-linux64', 'chrome'))
    .filter((candidate) => existsSync(candidate))
    .sort()
    .reverse();

  return entries[0] || envPath || '';
}

const CHROME_PATH = resolveChromePath();

/** CLI 인수에서 --mode=host|headless 파싱 */
function parseMode() {
  const modeArg = process.argv.find(a => a.startsWith('--mode='));
  if (modeArg) return modeArg.split('=')[1];
  return 'host';
}

const MODE = parseMode();

// ─── 내장 리포터 (runTest에서 자동 사용) ─────────────────

let _reporter = null;
let _currentTC = '';
let _lastScreenshot = null;

/** 현재 테스트 케이스 이름 설정 (보고서 그룹화용) */
export function setTestCase(name) {
  _currentTC = name;
}

// ─── 브라우저/페이지 생명주기 ────────────────────────────

/** Chrome 브라우저에 연결하거나 시작하고 반환 */
export async function launchBrowser() {
  if (MODE === 'headless') {
    if (!CHROME_PATH) {
      throw new Error('headless Chrome executable을 찾지 못했습니다. CHROME_PATH를 지정하거나 Puppeteer browser를 설치하세요.');
    }
    console.log('  [browser] headless Chrome 실행');
    return await puppeteer.launch({
      headless: true,
      executablePath: CHROME_PATH,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
    });
  }
  // 호스트 Chrome CDP에 연결
  console.log(`  [browser] 호스트 Chrome CDP 연결 (${CHROME_CDP})`);
  const browser = await puppeteer.connect({
    browserURL: CHROME_CDP,
    defaultViewport: null,
  });
  browser._isRemote = true;
  return browser;
}

/** 테스트용 페이지 생성 + 크기 설정
 * host 모드: 기본 1280x750 (윈도우 외곽 크기)
 * headless 모드: 기본 1280x900 (뷰포트)
 */
export async function createPage(browser, width, height) {
  if (!browser._testPages) browser._testPages = [];

  if (MODE === 'headless') {
    const page = await browser.newPage();
    await page.setViewport({ width: width || 1280, height: height || 900 });
    browser._testPages.push(page);
    return page;
  }
  // host 모드: 새 탭 열기 + 윈도우 크기 설정
  const page = await browser.newPage();
  browser._testPages.push(page);
  const w = width || 1280;
  const h = height || 750;
  const session = await page.createCDPSession();
  const { windowId } = await session.send('Browser.getWindowForTarget');
  await session.send('Browser.setWindowBounds', {
    windowId, bounds: { width: w, height: h, windowState: 'normal' },
  });
  await new Promise(r => setTimeout(r, 300));
  await session.detach();
  return page;
}

/** 페이지(탭) 정리 */
export async function closePage(page) {
  await page.close();
}

/** 브라우저 정리 — 테스트 탭 닫기 + CDP disconnect 또는 headless close */
export async function closeBrowser(browser) {
  if (browser._isRemote) {
    if (browser._testPages) {
      for (const p of browser._testPages) {
        await p.close().catch(() => {});
      }
      browser._testPages = [];
    }
    browser.disconnect();
  } else {
    await browser.close();
  }
}

// ─── 앱/문서 로드 ────────────────────────────────────────

/** 편집 영역 캔버스 셀렉터 (숨겨진 스크롤바 캔버스 제외) */
const CANVAS_SELECTOR = '#scroll-container canvas';

/** Vite dev server에서 앱을 로드하고 WASM 초기화 완료 대기 */
export async function loadApp(page, search = '') {
  await page.goto(`${VITE_URL}${search}`, { waitUntil: 'networkidle0', timeout: 30000 });
  await page.waitForFunction(() => !!window.__wasm && !!window.__canvasView, { timeout: 15000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));
}

/** 편집 영역 캔버스가 렌더링될 때까지 대기 */
export async function waitForCanvas(page, timeout = 10000) {
  await page.waitForSelector(CANVAS_SELECTOR, { timeout });
}

/** 새 빈 문서 생성 + 캔버스 대기 */
export async function createNewDocument(page) {
  await page.evaluate(() => window.__eventBus?.emit('create-new-document'));
  await page.waitForSelector(CANVAS_SELECTOR, { timeout: 10000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 1000)));
}

/** repo 샘플 HWP 파일을 Node에서 읽어 브라우저에 주입하고 문서를 로드한다 */
export async function loadHwpFile(page, filename) {
  const samplePath = path.join(RHWP_ROOT, 'samples', filename);
  if (!existsSync(samplePath)) {
    throw new Error(`샘플 파일이 없습니다: ${samplePath}`);
  }

  const bytes = [...readFileSync(samplePath)];
  const result = await page.evaluate(async ({ fname, sampleBytes }) => {
    try {
      const docInfo = window.__wasm?.loadDocument(new Uint8Array(sampleBytes), fname);
      if (!docInfo) return { error: 'loadDocument returned null' };
      const { loadWebFonts } = await import('/src/core/font-loader.ts');
      const includeOverlayFallbacks =
        (window.__canvasView?.getRenderBackend?.() ?? window.__renderBackend) === 'canvaskit';
      await loadWebFonts(docInfo.fontsUsed ?? [], undefined, { includeOverlayFallbacks });
      window.__canvasView?.loadDocument?.();
      return { pageCount: docInfo.pageCount };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  }, { fname: filename, sampleBytes: bytes });
  if (result.error) throw new Error(`파일 로드 실패 (${filename}): ${result.error}`);
  await page.waitForSelector(CANVAS_SELECTOR, { timeout: 10000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 1500)));
  return result;
}

// ─── 편집/입력 ────────────────────────────────────────────

/** 편집 영역(캔버스) 클릭하여 포커스 */
export async function clickEditArea(page) {
  const canvas = await page.$(CANVAS_SELECTOR);
  if (!canvas) throw new Error('편집 영역 캔버스를 찾을 수 없습니다');
  const box = await canvas.boundingBox();
  if (!box) throw new Error('캔버스 boundingBox가 null입니다');
  await page.mouse.click(box.x + box.width / 2, box.y + 100);
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));
}

/** 키보드로 텍스트 입력 */
export async function typeText(page, text) {
  for (const ch of text) {
    await page.keyboard.type(ch, { delay: 30 });
  }
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
}

/** 커서를 문서 위치로 이동한다 */
export async function moveCursorTo(page, sectionIndex, paragraphIndex, charOffset) {
  await page.evaluate((sec, para, offset) => {
    const handler = window.__inputHandler;
    if (handler?.cursor) {
      handler.cursor.moveTo({
        sectionIndex: sec,
        paragraphIndex: para,
        charOffset: offset,
      });
    }
  }, sectionIndex, paragraphIndex, charOffset);
  await page.evaluate(() => new Promise(r => setTimeout(r, 100)));
}

/** 커서를 문서 시작으로 이동한다 */
export async function moveCursorToStart(page) {
  await page.evaluate(() => {
    window.__inputHandler?.cursor?.moveToDocumentStart?.();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 100)));
}

/** 커서를 문서 끝으로 이동한다 */
export async function moveCursorToEnd(page) {
  await page.evaluate(() => {
    window.__inputHandler?.cursor?.moveToDocumentEnd?.();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 100)));
}

/** 현재 커서 위치를 반환한다 */
export async function getCursorPosition(page) {
  return await page.evaluate(() => {
    const pos = window.__inputHandler?.cursor?.getPosition?.();
    return pos ? {
      sectionIndex: pos.sectionIndex,
      paragraphIndex: pos.paragraphIndex,
      charOffset: pos.charOffset,
    } : null;
  });
}

// ─── 스크린샷/조회/검증 ──────────────────────────────────

/** 스크린샷을 파일로 저장 (리포터에 자동 연결) */
export async function screenshot(page, name) {
  const dir = 'e2e/screenshots';
  const { mkdirSync, existsSync } = await import('fs');
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  const path = `${dir}/${name}.png`;
  await page.screenshot({ path, fullPage: false });
  console.log(`  Screenshot: ${path}`);
  _lastScreenshot = `${name}.png`;
  // 리포터에 마지막 스크린샷 연결
  if (_reporter) {
    const results = _reporter.results;
    if (results.length > 0 && !results[results.length - 1].screenshot) {
      results[results.length - 1].screenshot = `${name}.png`;
    }
  }
  return path;
}

/** 편집 영역의 첫 번째 페이지 캔버스만 캡처한다 */
export async function screenshotCanvas(page, name) {
  const dir = 'e2e/screenshots';
  const { mkdirSync, existsSync } = await import('fs');
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  const path = `${dir}/${name}.png`;
  const canvas = await page.$(CANVAS_SELECTOR);
  if (!canvas) throw new Error('편집 영역 캔버스를 찾을 수 없습니다');
  const buffer = await canvas.screenshot({ path });
  console.log(`  Canvas Screenshot: ${path}`);
  _lastScreenshot = `${name}.png`;
  if (_reporter) {
    const results = _reporter.results;
    if (results.length > 0 && !results[results.length - 1].screenshot) {
      results[results.length - 1].screenshot = `${name}.png`;
    }
  }
  return { path, buffer };
}

/** layer tree에서 특정 op type의 bbox 목록을 수집한다 */
export async function getLayerOpBBoxes(page, opType) {
  return await page.evaluate((targetType) => {
    const tree = window.__wasm?.getPageLayerTree?.(0);
    const boxes = [];
    const walk = (node) => {
      if (!node) return;
      if (node.kind === 'leaf') {
        for (const op of node.ops) {
          if (op.type === targetType) boxes.push(op.bbox);
        }
        return;
      }
      if (node.kind === 'clipRect') {
        walk(node.child);
        return;
      }
      if (node.kind === 'group') {
        for (const child of node.children) walk(child);
      }
    };
    walk(tree?.root);
    return boxes;
  }, opType);
}

/** PNG 버퍼를 bbox 영역으로 잘라 새 PNG 버퍼를 반환한다 */
export function cropPngBuffer(buffer, bbox) {
  const image = PNG.sync.read(buffer);
  const x = Math.max(0, Math.floor(bbox.x));
  const y = Math.max(0, Math.floor(bbox.y));
  const width = Math.min(image.width - x, Math.ceil(bbox.width));
  const height = Math.min(image.height - y, Math.ceil(bbox.height));
  const out = new PNG({ width, height });
  PNG.bitblt(image, out, x, y, width, height, 0, 0);
  return PNG.sync.write(out);
}

/** 두 PNG 버퍼를 exact/tolerant 기준으로 비교하고 diff 아티팩트를 저장한다 */
export async function comparePngBuffers(expectedBuffer, actualBuffer, {
  diffName,
  threshold = 0,
  ignoreChannelDelta = 0,
  maxDiffPixels = null,
  maxDiffRatio = null,
} = {}) {
  const expected = PNG.sync.read(expectedBuffer);
  const actual = PNG.sync.read(actualBuffer);

  if (expected.width !== actual.width || expected.height !== actual.height) {
    throw new Error(`이미지 크기 불일치: ${expected.width}x${expected.height} vs ${actual.width}x${actual.height}`);
  }

  const exactDiff = new PNG({ width: expected.width, height: expected.height });
  const exactDiffPixels = pixelmatch(
    expected.data,
    actual.data,
    exactDiff.data,
    expected.width,
    expected.height,
    { threshold, includeAA: true },
  );

  const tolerantDiff = new PNG({ width: expected.width, height: expected.height });
  let tolerantDiffPixels = 0;
  let totalChannelDelta = 0;
  let maxChannelDelta = 0;

  for (let i = 0; i < expected.data.length; i += 4) {
    let pixelMaxDelta = 0;

    for (let channel = 0; channel < 4; channel++) {
      const delta = Math.abs(expected.data[i + channel] - actual.data[i + channel]);
      totalChannelDelta += delta;
      if (delta > pixelMaxDelta) pixelMaxDelta = delta;
      if (delta > maxChannelDelta) maxChannelDelta = delta;
    }

    if (pixelMaxDelta > ignoreChannelDelta) {
      tolerantDiffPixels++;
      tolerantDiff.data[i] = Math.max(pixelMaxDelta, 32);
      tolerantDiff.data[i + 1] = 0;
      tolerantDiff.data[i + 2] = 0;
      tolerantDiff.data[i + 3] = 255;
    }
  }

  const totalPixels = expected.width * expected.height;
  const exactDiffRatio = totalPixels > 0 ? exactDiffPixels / totalPixels : 0;
  const rawTolerantDiffRatio = totalPixels > 0 ? tolerantDiffPixels / totalPixels : 0;
  const meanAbsChannelDelta = totalPixels > 0 ? totalChannelDelta / (totalPixels * 4) : 0;
  const hasPixelBudget = maxDiffPixels != null;
  const hasRatioBudget = maxDiffRatio != null;
  const passed = hasPixelBudget || hasRatioBudget
    ? (!hasPixelBudget || tolerantDiffPixels <= maxDiffPixels)
      && (!hasRatioBudget || rawTolerantDiffRatio <= maxDiffRatio)
    : tolerantDiffPixels === 0;
  const budgetedTolerantDiffPixels = passed ? 0 : tolerantDiffPixels;
  const budgetedTolerantDiffRatio = passed ? 0 : rawTolerantDiffRatio;

  let exactDiffPath = null;
  let tolerantDiffPath = null;
  if (diffName && (exactDiffPixels > 0 || tolerantDiffPixels > 0)) {
    const { mkdirSync, existsSync, writeFileSync } = await import('fs');
    const outputDir = '../output/e2e/canvaskit-diff';
    if (!existsSync(outputDir)) mkdirSync(outputDir, { recursive: true });
    if (exactDiffPixels > 0) {
      exactDiffPath = `${outputDir}/${diffName}.png`;
      writeFileSync(exactDiffPath, PNG.sync.write(exactDiff));
      console.log(`  Exact Diff Artifact: ${exactDiffPath}`);
    }
    if (!passed && tolerantDiffPixels > 0) {
      tolerantDiffPath = `${outputDir}/${diffName}-tolerant.png`;
      writeFileSync(tolerantDiffPath, PNG.sync.write(tolerantDiff));
      console.log(`  Tolerant Diff Artifact: ${tolerantDiffPath}`);
    }
  }

  return {
    passed,
    diffPixels: budgetedTolerantDiffPixels,
    diffRatio: budgetedTolerantDiffRatio,
    exactDiffPixels,
    exactDiffRatio,
    tolerantDiffPixels: budgetedTolerantDiffPixels,
    tolerantDiffRatio: budgetedTolerantDiffRatio,
    rawTolerantDiffPixels: tolerantDiffPixels,
    rawTolerantDiffRatio,
    width: expected.width,
    height: expected.height,
    ignoreChannelDelta,
    meanAbsChannelDelta,
    maxChannelDelta,
    exactDiffPath,
    tolerantDiffPath,
  };
}

/** WASM bridge를 통해 페이지 수 조회 */
export async function getPageCount(page) {
  return await page.evaluate(() => window.__wasm?.pageCount ?? 0);
}

/** WASM bridge를 통해 문단 수 조회 */
export async function getParagraphCount(page, sectionIdx = 0) {
  return await page.evaluate((sec) => window.__wasm?.getParagraphCount(sec) ?? -1, sectionIdx);
}

/** WASM bridge를 통해 문단 텍스트 조회 */
export async function getParaText(page, secIdx, paraIdx, maxLen = 200) {
  return await page.evaluate((s, p, m) => {
    try { return window.__wasm?.getTextRange(s, p, 0, m) ?? ''; }
    catch { return ''; }
  }, secIdx, paraIdx, maxLen);
}

/** 테스트 결과 출력 + 리포터 자동 기록 */
export function assert(condition, message) {
  if (condition) {
    console.log(`  PASS: ${message}`);
    if (_reporter) _reporter.pass(_currentTC, message, _lastScreenshot);
  } else {
    console.error(`  FAIL: ${message}`);
    if (_reporter) _reporter.fail(_currentTC, message, _lastScreenshot);
    process.exitCode = 1;
  }
  _lastScreenshot = null;
}

// ─── 테스트 러너 ─────────────────────────────────────────

/**
 * 테스트 파일명에서 보고서 파일명 생성
 * e.g., "copy-paste.test.mjs" → "copy-paste-report.html"
 */
function getReportFilename() {
  const scriptPath = process.argv[1] || 'unknown';
  const basename = scriptPath.split('/').pop().replace(/\.test\.mjs$/, '');
  return `${basename}-report.html`;
}

/**
 * 테스트 실행 래퍼 — 공통 골격 (브라우저/페이지 생명주기 + 에러 처리 + HTML 보고서)
 *
 * 사용법:
 *   runTest('테스트 제목', async ({ page, browser }) => {
 *     await createNewDocument(page);
 *     // ... 테스트 로직
 *   });
 */
export async function runTest(title, testFn, { skipLoadApp = false } = {}) {
  console.log(`=== E2E: ${title} ===\n`);
  _reporter = new TestReporter(title);
  _currentTC = title;
  _lastScreenshot = null;

  const browser = await launchBrowser();
  const page = await createPage(browser);

  try {
    if (!skipLoadApp) await loadApp(page);
    await testFn({ page, browser });
  } catch (err) {
    console.error('테스트 오류:', err.message || err);
    await screenshot(page, 'error').catch(() => {});
    if (_reporter) _reporter.fail(_currentTC, `ERROR: ${err.message || err}`);
    process.exitCode = 1;
  } finally {
    // HTML 보고서 생성
    const reportFile = `${REPORT_DIR}/${getReportFilename()}`;
    _reporter.generate(reportFile);
    _reporter = null;
    _currentTC = '';
    _lastScreenshot = null;
    await closeBrowser(browser);
  }
}
