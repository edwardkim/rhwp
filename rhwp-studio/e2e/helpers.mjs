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
import path from 'path';
import pixelmatch from 'pixelmatch';
import puppeteer from 'puppeteer-core';
import { existsSync, readdirSync } from 'fs';
import os from 'os';
import { PNG } from 'pngjs';
import { TestReporter } from './report-generator.mjs';

const MAX_INK_MASK_MATCH_EDGES = 5_000_000;
const CHROME_CDP = process.env.CHROME_CDP || 'http://172.21.192.1:19222';
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';
const REPORT_DIR = '../output/e2e';
const CHROME_EXTRA_ARGS = (process.env.CHROME_EXTRA_ARGS || '')
  .split(/\s+/)
  .map((arg) => arg.trim())
  .filter(Boolean);

function resolveChromePath() {
  const envPath = process.env.CHROME_PATH || process.env.PUPPETEER_EXECUTABLE_PATH;
  if (envPath && existsSync(envPath)) return envPath;

  const systemChrome = [
    '/usr/bin/google-chrome-stable',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium-browser',
    '/usr/bin/chromium',
  ].find((candidate) => existsSync(candidate));
  if (systemChrome) return systemChrome;

  const cacheRoot = path.join(os.homedir(), '.cache', 'puppeteer');
  if (!existsSync(cacheRoot)) return envPath || '';

  const stack = [cacheRoot];
  const candidates = [];
  while (stack.length) {
    const current = stack.pop();
    let entries;
    try {
      entries = readdirSync(current, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      const candidate = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(candidate);
      } else if (entry.isFile() && (entry.name === 'chrome' || entry.name === 'chrome-headless-shell')) {
        candidates.push(candidate);
      }
    }
  }

  const entries = candidates.sort().reverse();
  return entries.find((candidate) => path.basename(candidate) === 'chrome') || entries[0] || envPath || '';
}

const CHROME_PATH = resolveChromePath();

function sampleFetchPath(filename) {
  const value = String(filename || '').trim();
  if (!value || value.includes('\0') || value.includes('\\') || value.includes('?') || value.includes('#')) {
    throw new Error(`잘못된 샘플 파일명: ${filename}`);
  }
  if (value.startsWith('/') || /^[A-Za-z][A-Za-z0-9+.-]*:/.test(value)) {
    throw new Error(`샘플 파일명은 /samples 하위 상대 경로여야 함: ${filename}`);
  }
  let decoded = value;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    throw new Error(`샘플 파일명 URL escape 오류: ${filename}`);
  }
  if (decoded !== value) {
    throw new Error(`샘플 파일명은 percent-encoding 없이 전달해야 함: ${filename}`);
  }
  const parts = value.split('/');
  if (parts.some(part => !part || part === '.' || part === '..')) {
    throw new Error(`샘플 파일명이 /samples 밖으로 벗어날 수 있음: ${filename}`);
  }
  return `/samples/${parts.map(encodeURIComponent).join('/')}`;
}

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
    console.log('  [browser] headless Chrome 실행');
    return await puppeteer.launch({
      headless: true,
      executablePath: CHROME_PATH,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu', ...CHROME_EXTRA_ARGS],
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
  await page.goto(`${VITE_URL}${search}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  const appReady = page.waitForFunction(() => !!window.__wasm && !!window.__canvasView, {
    timeout: 60000,
  });
  // CI/Vite/PWA 조합에서 networkidle0 이 끝까지 내려가지 않는 경우가 있어 보조 대기로만 사용한다.
  await Promise.race([
    appReady,
    page.waitForNetworkIdle({ idleTime: 500, timeout: 5000 }).catch(() => {}),
  ]);
  try {
    await appReady;
  } catch (err) {
    const state = await page.evaluate(() => ({
      readyState: document.readyState,
      hasWasm: !!window.__wasm,
      hasCanvasView: !!window.__canvasView,
      location: window.location.href,
      title: document.title,
    })).catch(() => null);
    throw new Error(`앱 초기화 대기 실패: ${err.message || err}; state=${JSON.stringify(state)}`);
  }
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

/** HWP 파일을 fetch하여 문서 로드 + 캔버스 대기 */
export async function loadHwpFile(page, filename) {
  const fetchPath = sampleFetchPath(filename);
  const result = await page.evaluate(async ({ fname, url }) => {
    const startedAt = performance.now();
    try {
      const resp = await fetch(url);
      if (!resp.ok) return { error: `HTTP ${resp.status}` };
      const buf = await resp.arrayBuffer();
      const docInfo = window.__wasm?.loadDocument(new Uint8Array(buf), fname);
      if (!docInfo) return { error: 'loadDocument returned null' };
      window.__canvasView?.loadDocument?.();
      return {
        pageCount: docInfo.pageCount,
        documentLoadAndInitialRenderMs: performance.now() - startedAt,
      };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  }, { fname: filename, url: fetchPath });
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

/** 편집 영역의 첫 번째 페이지 캔버스를 지정 경로로 캡처한다 */
export async function captureCanvasScreenshot(page, outputPath, logLabel = 'Canvas Screenshot') {
  const { mkdirSync, existsSync } = await import('fs');
  const outputDir = path.dirname(outputPath);
  if (!existsSync(outputDir)) mkdirSync(outputDir, { recursive: true });
  const canvas = await page.$(CANVAS_SELECTOR);
  if (!canvas) throw new Error('편집 영역 캔버스를 찾을 수 없습니다');
  const buffer = await canvas.screenshot({ path: outputPath });
  console.log(`  ${logLabel}: ${outputPath}`);
  _lastScreenshot = path.basename(outputPath);
  return { path: outputPath, buffer };
}

export function cropPngBuffer(buffer, { x = 0, y = 0, width, height }) {
  const source = PNG.sync.read(buffer);
  const cropX = Number(x);
  const cropY = Number(y);
  const cropWidth = Number(width);
  const cropHeight = Number(height);
  if (
    !Number.isInteger(cropX)
    || !Number.isInteger(cropY)
    || !Number.isInteger(cropWidth)
    || !Number.isInteger(cropHeight)
    || cropX < 0
    || cropY < 0
    || cropWidth <= 0
    || cropHeight <= 0
    || cropX + cropWidth > source.width
    || cropY + cropHeight > source.height
  ) {
    throw new Error(
      `invalid PNG crop: ${cropX},${cropY} ${cropWidth}x${cropHeight} `
        + `for ${source.width}x${source.height}`,
    );
  }

  const cropped = new PNG({ width: cropWidth, height: cropHeight });
  for (let row = 0; row < cropHeight; row += 1) {
    const sourceStart = ((cropY + row) * source.width + cropX) * 4;
    const targetStart = row * cropWidth * 4;
    source.data.copy(cropped.data, targetStart, sourceStart, sourceStart + cropWidth * 4);
  }
  return PNG.sync.write(cropped);
}

/** 두 PNG 버퍼를 exact/tolerant/raster-mask 기준으로 비교한다 */
export async function comparePngBuffers(expectedBuffer, actualBuffer, {
  threshold = 0,
  ignoreChannelDelta = 0,
  maxDiffPixels = null,
  maxDiffRatio = null,
  inkMaskWhiteDelta = 25,
  inkMaskAlphaThreshold = 8,
  inkMaskNeighborhoodRadius = 1,
  inkMaskMaxDiffPixels = null,
  inkMaskMaxDiffRatio = null,
  nonInkMaxDiffPixels = null,
  nonInkMaxDiffRatio = null,
  solidInkMaxDiffPixels = null,
  solidInkMaxDiffRatio = null,
  minimumInkPixels = null,
} = {}) {
  const expected = PNG.sync.read(expectedBuffer);
  const actual = PNG.sync.read(actualBuffer);

  if (expected.width !== actual.width || expected.height !== actual.height) {
    throw new Error(`이미지 크기 불일치: ${expected.width}x${expected.height} vs ${actual.width}x${actual.height}`);
  }

  const exactDiffPixels = pixelmatch(
    expected.data,
    actual.data,
    null,
    expected.width,
    expected.height,
    { threshold, includeAA: true },
  );

  let tolerantDiffPixels = 0;
  let inkMaskDiffPixels = 0;
  let nonInkDiffPixels = 0;
  let solidInkDiffPixels = 0;
  let totalChannelDelta = 0;
  let maxChannelDelta = 0;
  const totalPixels = expected.width * expected.height;
  const width = expected.width;
  const height = expected.height;
  const expectedInkMask = new Uint8Array(totalPixels);
  const actualInkMask = new Uint8Array(totalPixels);
  let expectedInkPixels = 0;
  let actualInkPixels = 0;

  const isInkPixel = (data, base) => data[base + 3] > inkMaskAlphaThreshold
    && Math.max(
      255 - data[base],
      255 - data[base + 1],
      255 - data[base + 2],
    ) > inkMaskWhiteDelta;

  const hasInkNearby = (mask, x, y) => {
    const minY = Math.max(0, y - inkMaskNeighborhoodRadius);
    const maxY = Math.min(height - 1, y + inkMaskNeighborhoodRadius);
    const minX = Math.max(0, x - inkMaskNeighborhoodRadius);
    const maxX = Math.min(width - 1, x + inkMaskNeighborhoodRadius);
    for (let ny = minY; ny <= maxY; ny++) {
      for (let nx = minX; nx <= maxX; nx++) {
        if (mask[ny * width + nx]) return true;
      }
    }
    return false;
  };

  const isSolidInk = (mask, x, y) => {
    const minY = Math.max(0, y - inkMaskNeighborhoodRadius);
    const maxY = Math.min(height - 1, y + inkMaskNeighborhoodRadius);
    const minX = Math.max(0, x - inkMaskNeighborhoodRadius);
    const maxX = Math.min(width - 1, x + inkMaskNeighborhoodRadius);
    for (let ny = minY; ny <= maxY; ny++) {
      for (let nx = minX; nx <= maxX; nx++) {
        if (!mask[ny * width + nx]) return false;
      }
    }
    return true;
  };

  for (let pixelIndex = 0; pixelIndex < totalPixels; pixelIndex++) {
    const base = pixelIndex * 4;
    expectedInkMask[pixelIndex] = isInkPixel(expected.data, base) ? 1 : 0;
    actualInkMask[pixelIndex] = isInkPixel(actual.data, base) ? 1 : 0;
    expectedInkPixels += expectedInkMask[pixelIndex];
    actualInkPixels += actualInkMask[pixelIndex];
  }

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
      const pixelIndex = i / 4;
      const x = pixelIndex % width;
      const y = (pixelIndex - x) / width;
      if (!hasInkNearby(expectedInkMask, x, y) && !hasInkNearby(actualInkMask, x, y)) {
        nonInkDiffPixels++;
      }
      if (
        expectedInkMask[pixelIndex]
        && actualInkMask[pixelIndex]
        && isSolidInk(expectedInkMask, x, y)
        && isSolidInk(actualInkMask, x, y)
      ) {
        solidInkDiffPixels++;
      }
    }
  }

  const expectedOnlyInk = [];
  const actualOnlyInk = [];
  for (let pixelIndex = 0; pixelIndex < totalPixels; pixelIndex++) {
    if (expectedInkMask[pixelIndex] && !actualInkMask[pixelIndex]) {
      expectedOnlyInk.push(pixelIndex);
    } else if (actualInkMask[pixelIndex] && !expectedInkMask[pixelIndex]) {
      actualOnlyInk.push(pixelIndex);
    }
  }
  const actualPositionByPixel = new Int32Array(totalPixels);
  actualPositionByPixel.fill(-1);
  for (let actualPosition = 0; actualPosition < actualOnlyInk.length; actualPosition++) {
    actualPositionByPixel[actualOnlyInk[actualPosition]] = actualPosition;
  }
  const adjacencyOffsets = new Int32Array(expectedOnlyInk.length + 1);
  let adjacencyEdgeCount = 0;
  for (let expectedPosition = 0; expectedPosition < expectedOnlyInk.length; expectedPosition++) {
    const expectedIndex = expectedOnlyInk[expectedPosition];
    const expectedX = expectedIndex % width;
    const expectedY = (expectedIndex - expectedX) / width;
    const minY = Math.max(0, expectedY - inkMaskNeighborhoodRadius);
    const maxY = Math.min(height - 1, expectedY + inkMaskNeighborhoodRadius);
    const minX = Math.max(0, expectedX - inkMaskNeighborhoodRadius);
    const maxX = Math.min(width - 1, expectedX + inkMaskNeighborhoodRadius);
    for (let actualY = minY; actualY <= maxY; actualY++) {
      for (let actualX = minX; actualX <= maxX; actualX++) {
        const actualIndex = actualY * width + actualX;
        const actualPosition = actualPositionByPixel[actualIndex];
        if (actualPosition >= 0) {
          adjacencyEdgeCount++;
          if (adjacencyEdgeCount > MAX_INK_MASK_MATCH_EDGES) {
            throw new Error(
              `ink-mask matching edge budget exceeded: ${adjacencyEdgeCount} > ${MAX_INK_MASK_MATCH_EDGES}`,
            );
          }
        }
      }
    }
    adjacencyOffsets[expectedPosition + 1] = adjacencyEdgeCount;
  }

  // Maximum-cardinality matching keeps the one-to-one ink comparison independent
  // of row-major traversal order. Each left node has at most (2r + 1)^2 edges.
  const adjacency = new Int32Array(adjacencyEdgeCount);
  for (let expectedPosition = 0; expectedPosition < expectedOnlyInk.length; expectedPosition++) {
    const expectedIndex = expectedOnlyInk[expectedPosition];
    const expectedX = expectedIndex % width;
    const expectedY = (expectedIndex - expectedX) / width;
    const minY = Math.max(0, expectedY - inkMaskNeighborhoodRadius);
    const maxY = Math.min(height - 1, expectedY + inkMaskNeighborhoodRadius);
    const minX = Math.max(0, expectedX - inkMaskNeighborhoodRadius);
    const maxX = Math.min(width - 1, expectedX + inkMaskNeighborhoodRadius);
    let edgePosition = adjacencyOffsets[expectedPosition];
    for (let actualY = minY; actualY <= maxY; actualY++) {
      for (let actualX = minX; actualX <= maxX; actualX++) {
        const actualPosition = actualPositionByPixel[actualY * width + actualX];
        if (actualPosition >= 0) adjacency[edgePosition++] = actualPosition;
      }
    }
  }
  const matchedActualByExpected = new Int32Array(expectedOnlyInk.length);
  const matchedExpectedByActual = new Int32Array(actualOnlyInk.length);
  const distances = new Int32Array(expectedOnlyInk.length);
  const queue = new Int32Array(expectedOnlyInk.length);
  matchedActualByExpected.fill(-1);
  matchedExpectedByActual.fill(-1);
  let matchingSize = 0;

  while (expectedOnlyInk.length > 0 && actualOnlyInk.length > 0) {
    distances.fill(-1);
    let queueStart = 0;
    let queueEnd = 0;
    for (let expectedPosition = 0; expectedPosition < expectedOnlyInk.length; expectedPosition++) {
      if (matchedActualByExpected[expectedPosition] < 0) {
        distances[expectedPosition] = 0;
        queue[queueEnd++] = expectedPosition;
      }
    }

    let shortestAugmentingPath = Number.POSITIVE_INFINITY;
    while (queueStart < queueEnd) {
      const expectedPosition = queue[queueStart++];
      const nextDistance = distances[expectedPosition] + 1;
      if (nextDistance > shortestAugmentingPath) continue;
      for (
        let edgePosition = adjacencyOffsets[expectedPosition];
        edgePosition < adjacencyOffsets[expectedPosition + 1];
        edgePosition++
      ) {
        const actualPosition = adjacency[edgePosition];
        const pairedExpected = matchedExpectedByActual[actualPosition];
        if (pairedExpected < 0) {
          shortestAugmentingPath = Math.min(shortestAugmentingPath, nextDistance);
        } else if (distances[pairedExpected] < 0 && nextDistance < shortestAugmentingPath) {
          distances[pairedExpected] = nextDistance;
          queue[queueEnd++] = pairedExpected;
        }
      }
    }
    if (!Number.isFinite(shortestAugmentingPath)) break;

    let phaseMatchingCount = 0;
    for (let root = 0; root < expectedOnlyInk.length; root++) {
      if (matchedActualByExpected[root] >= 0 || distances[root] !== 0) continue;
      const expectedStack = [root];
      const cursorStack = [adjacencyOffsets[root]];
      let augmented = false;
      while (expectedStack.length > 0 && !augmented) {
        const stackIndex = expectedStack.length - 1;
        const expectedPosition = expectedStack[stackIndex];
        const edgeEnd = adjacencyOffsets[expectedPosition + 1];
        let descended = false;
        while (cursorStack[stackIndex] < edgeEnd) {
          const actualPosition = adjacency[cursorStack[stackIndex]++];
          const pairedExpected = matchedExpectedByActual[actualPosition];
          if (pairedExpected < 0) {
            if (distances[expectedPosition] + 1 !== shortestAugmentingPath) continue;
            let nextActual = actualPosition;
            for (let pathIndex = expectedStack.length - 1; pathIndex >= 0; pathIndex--) {
              const pathExpected = expectedStack[pathIndex];
              const previousActual = matchedActualByExpected[pathExpected];
              matchedActualByExpected[pathExpected] = nextActual;
              matchedExpectedByActual[nextActual] = pathExpected;
              nextActual = previousActual;
            }
            matchingSize++;
            phaseMatchingCount++;
            augmented = true;
            break;
          }
          if (distances[pairedExpected] === distances[expectedPosition] + 1) {
            expectedStack.push(pairedExpected);
            cursorStack.push(adjacencyOffsets[pairedExpected]);
            descended = true;
            break;
          }
        }
        if (augmented || descended) continue;
        distances[expectedPosition] = -1;
        expectedStack.pop();
        cursorStack.pop();
      }
    }
    if (phaseMatchingCount === 0) break;
  }
  inkMaskDiffPixels = expectedOnlyInk.length + actualOnlyInk.length - 2 * matchingSize;

  const exactDiffRatio = totalPixels > 0 ? exactDiffPixels / totalPixels : 0;
  const rawTolerantDiffRatio = totalPixels > 0 ? tolerantDiffPixels / totalPixels : 0;
  const rawInkMaskDiffRatio = totalPixels > 0 ? inkMaskDiffPixels / totalPixels : 0;
  const rawNonInkDiffRatio = totalPixels > 0 ? nonInkDiffPixels / totalPixels : 0;
  const rawSolidInkDiffRatio = totalPixels > 0 ? solidInkDiffPixels / totalPixels : 0;
  const meanAbsChannelDelta = totalPixels > 0 ? totalChannelDelta / (totalPixels * 4) : 0;
  const hasPixelBudget = maxDiffPixels != null;
  const hasRatioBudget = maxDiffRatio != null;
  const hasInkMaskPixelBudget = inkMaskMaxDiffPixels != null;
  const hasInkMaskRatioBudget = inkMaskMaxDiffRatio != null;
  const hasNonInkPixelBudget = nonInkMaxDiffPixels != null;
  const hasNonInkRatioBudget = nonInkMaxDiffRatio != null;
  const hasSolidInkPixelBudget = solidInkMaxDiffPixels != null;
  const hasSolidInkRatioBudget = solidInkMaxDiffRatio != null;
  const hasMinimumInkBudget = minimumInkPixels != null;
  const usesTolerantBudget = hasPixelBudget || hasRatioBudget;
  const usesInkMaskBudget = hasInkMaskPixelBudget || hasInkMaskRatioBudget;
  const usesNonInkBudget = hasNonInkPixelBudget || hasNonInkRatioBudget;
  const usesSolidInkBudget = hasSolidInkPixelBudget || hasSolidInkRatioBudget;
  const usesRasterOnlyBudget = usesInkMaskBudget || usesNonInkBudget || usesSolidInkBudget;
  const tolerantBudgetPassed = usesTolerantBudget
    ? (!hasPixelBudget || tolerantDiffPixels <= maxDiffPixels)
      && (!hasRatioBudget || rawTolerantDiffRatio <= maxDiffRatio)
    : tolerantDiffPixels === 0;
  const inkMaskBudgetPassed = usesInkMaskBudget
    ? (!hasInkMaskPixelBudget || inkMaskDiffPixels <= inkMaskMaxDiffPixels)
      && (!hasInkMaskRatioBudget || rawInkMaskDiffRatio <= inkMaskMaxDiffRatio)
    : inkMaskDiffPixels === 0;
  const nonInkBudgetPassed = usesNonInkBudget
    ? (!hasNonInkPixelBudget || nonInkDiffPixels <= nonInkMaxDiffPixels)
      && (!hasNonInkRatioBudget || rawNonInkDiffRatio <= nonInkMaxDiffRatio)
    : nonInkDiffPixels === 0;
  const solidInkBudgetPassed = usesSolidInkBudget
    ? (!hasSolidInkPixelBudget || solidInkDiffPixels <= solidInkMaxDiffPixels)
      && (!hasSolidInkRatioBudget || rawSolidInkDiffRatio <= solidInkMaxDiffRatio)
    : solidInkDiffPixels === 0;
  const rasterOnlyBudgetPassed = (!usesInkMaskBudget || inkMaskBudgetPassed)
    && (!usesNonInkBudget || nonInkBudgetPassed)
    && (!usesSolidInkBudget || solidInkBudgetPassed);
  const minimumInkBudgetPassed = !hasMinimumInkBudget
    || (expectedInkPixels >= minimumInkPixels && actualInkPixels >= minimumInkPixels);
  const parityBudgetPassed = usesTolerantBudget
    ? tolerantBudgetPassed && (!usesRasterOnlyBudget || rasterOnlyBudgetPassed)
    : usesRasterOnlyBudget
      ? rasterOnlyBudgetPassed
      : tolerantDiffPixels === 0;
  const passed = parityBudgetPassed && minimumInkBudgetPassed;
  const selectedDiffPixels = usesTolerantBudget
    ? Math.max(tolerantDiffPixels, inkMaskDiffPixels, nonInkDiffPixels, solidInkDiffPixels)
    : usesRasterOnlyBudget
      ? Math.max(inkMaskDiffPixels, nonInkDiffPixels, solidInkDiffPixels)
      : tolerantDiffPixels;
  const selectedDiffRatio = usesTolerantBudget
    ? Math.max(rawTolerantDiffRatio, rawInkMaskDiffRatio, rawNonInkDiffRatio, rawSolidInkDiffRatio)
    : usesRasterOnlyBudget
      ? Math.max(rawInkMaskDiffRatio, rawNonInkDiffRatio, rawSolidInkDiffRatio)
      : rawTolerantDiffRatio;

  return {
    passed,
    hasVisualBudget: usesTolerantBudget || usesRasterOnlyBudget,
    passMetric: usesTolerantBudget && usesRasterOnlyBudget
      ? 'combined'
      : usesRasterOnlyBudget
        ? 'rasterOnly'
        : 'tolerant',
    width: expected.width,
    height: expected.height,
    exactDiffPixels,
    exactDiffRatio,
    tolerantDiffPixels,
    tolerantDiffRatio: rawTolerantDiffRatio,
    rawTolerantDiffPixels: tolerantDiffPixels,
    rawTolerantDiffRatio,
    inkMaskDiffPixels,
    inkMaskDiffRatio: rawInkMaskDiffRatio,
    rawInkMaskDiffPixels: inkMaskDiffPixels,
    rawInkMaskDiffRatio,
    nonInkDiffPixels,
    nonInkDiffRatio: rawNonInkDiffRatio,
    rawNonInkDiffPixels: nonInkDiffPixels,
    rawNonInkDiffRatio,
    solidInkDiffPixels,
    solidInkDiffRatio: rawSolidInkDiffRatio,
    rawSolidInkDiffPixels: solidInkDiffPixels,
    rawSolidInkDiffRatio,
    tolerantBudgetPassed,
    inkMaskBudgetPassed,
    nonInkBudgetPassed,
    solidInkBudgetPassed,
    rasterOnlyBudgetPassed,
    expectedInkPixels,
    actualInkPixels,
    minimumInkPixels,
    minimumInkBudgetPassed,
    diffPixels: selectedDiffPixels,
    diffRatio: selectedDiffRatio,
    maxChannelDelta,
    meanAbsChannelDelta,
    ignoreChannelDelta,
    maxDiffPixels,
    maxDiffRatio,
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
