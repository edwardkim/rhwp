import assert from 'node:assert/strict';
import test from 'node:test';
import { loadWebFonts } from '../src/core/font-loader.ts';

const JSDELIVR_HOSTNAME = 'cdn.jsdelivr.net';
const CSS_URL_PATTERN = /url\((?:"([^"]+)"|'([^']+)'|([^'")]+))\)/g;

function extractFontUrls(source: string): URL[] {
  return Array.from(source.matchAll(CSS_URL_PATTERN), match => (
    match[1] ?? match[2] ?? match[3] ?? ''
  ).trim()).flatMap(rawUrl => {
    try {
      return [new URL(rawUrl)];
    } catch {
      return [];
    }
  });
}

function usesJsDelivrFontUrl(source: string): boolean {
  return extractFontUrls(source).some(url => (
    url.protocol === 'https:' && url.hostname === JSDELIVR_HOSTNAME
  ));
}

function usesExternalFontUrl(source: string): boolean {
  return extractFontUrls(source).some(url => (
    url.protocol === 'http:' || url.protocol === 'https:'
  ));
}

test('외부 웹폰트 사용 안 함 옵션은 CDN @font-face와 FontFace.load를 건너뛴다', async () => {
  const styles: Array<{ id: string; textContent: string }> = [];
  const fontFaceRequests: Array<{ family: string; source: string }> = [];
  const previousDocument = (globalThis as typeof globalThis & { document?: unknown }).document;
  const previousFontFace = (globalThis as typeof globalThis & { FontFace?: unknown }).FontFace;

  const fakeDocument = {
    head: {
      appendChild(element: { id: string; textContent: string }) {
        styles.push(element);
      },
    },
    createElement(tagName: string) {
      assert.equal(tagName, 'style');
      return { id: '', textContent: '' };
    },
    getElementById(id: string) {
      return styles.find(style => style.id === id) ?? null;
    },
    fonts: {
      check() {
        return false;
      },
      add() {
        // 테스트에서는 등록 호출 여부만 FontFace 생성 기록으로 확인한다.
      },
    },
  };

  class FakeFontFace {
    family: string;
    source: string;

    constructor(family: string, source: string) {
      this.family = family;
      this.source = source;
      fontFaceRequests.push({ family, source });
    }

    async load(): Promise<FakeFontFace> {
      return this;
    }
  }

  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: fakeDocument,
  });
  Object.defineProperty(globalThis, 'FontFace', {
    configurable: true,
    value: FakeFontFace,
  });

  try {
    await loadWebFonts([], undefined, { disableExternalWebFonts: true });

    assert.equal(styles.length, 1);
    assert.equal(usesJsDelivrFontUrl(styles[0].textContent), false);
    assert.equal(fontFaceRequests.some(request => usesExternalFontUrl(request.source)), false);

    fontFaceRequests.length = 0;
    await loadWebFonts([]);

    assert.equal(usesJsDelivrFontUrl(styles[0].textContent), true);
    assert.equal(fontFaceRequests.some(request => usesJsDelivrFontUrl(request.source)), true);
  } finally {
    Object.defineProperty(globalThis, 'document', {
      configurable: true,
      value: previousDocument,
    });
    Object.defineProperty(globalThis, 'FontFace', {
      configurable: true,
      value: previousFontFace,
    });
  }
});

test('문서 글꼴은 시스템에 없을 때만 조건부 웹폰트로 등록한다', async () => {
  const styles: Array<{ id: string; textContent: string }> = [];
  const fontFaceRequests: Array<{ family: string; source: string }> = [];
  const previousDocument = (globalThis as typeof globalThis & { document?: unknown }).document;
  const previousFontFace = (globalThis as typeof globalThis & { FontFace?: unknown }).FontFace;
  const previousConsoleDebug = console.debug;
  const debugLogs: string[] = [];

  const fakeDocument = {
    head: {
      appendChild(element: { id: string; textContent: string }) {
        styles.push(element);
      },
    },
    createElement(tagName: string) {
      assert.equal(tagName, 'style');
      return { id: '', textContent: '' };
    },
    getElementById(id: string) {
      return styles.find(style => style.id === id) ?? null;
    },
    fonts: {
      check(font: string) {
        return font.includes('DejaVu Serif');
      },
      add() {
        // 테스트에서는 FontFace 생성 기록으로 등록 여부를 검증한다.
      },
    },
  };

  class FakeFontFace {
    family: string;
    source: string;

    constructor(family: string, source: string) {
      this.family = family;
      this.source = source;
      fontFaceRequests.push({ family, source });
    }

    async load(): Promise<FakeFontFace> {
      return this;
    }
  }

  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: fakeDocument,
  });
  Object.defineProperty(globalThis, 'FontFace', {
    configurable: true,
    value: FakeFontFace,
  });
  console.debug = (...args: unknown[]) => {
    debugLogs.push(args.map(value => String(value)).join(' '));
  };

  try {
    await loadWebFonts([
      'DejaVu Serif',
      'Roboto',
      '정부상징 부처명_16040911',
      'KoPub돋움체 Medium',
      'KoPubBatangMedium',
      'KoPubWorld돋움체 Medium',
      '경기천년제목 Medium',
      '62570체',
    ]);

    assert.equal(fontFaceRequests.some(request => request.family === 'DejaVu Serif'), false);
    assert.equal(fontFaceRequests.some(request => request.family === 'Roboto'), true);
    assert.equal(fontFaceRequests.some(request => request.family === '정부상징 부처명_16040911'), true);
    assert.equal(fontFaceRequests.some(request => request.family === 'KoPub돋움체 Medium'), true);
    assert.equal(fontFaceRequests.some(request => request.family === 'KoPubBatangMedium'), true);
    assert.equal(fontFaceRequests.some(request => request.family === 'KoPubWorld돋움체 Medium'), true);
    assert.equal(fontFaceRequests.some(request => request.family === '경기천년제목 Medium'), true);
    assert.equal(fontFaceRequests.some(request => request.family === '62570체'), true);
    assert.equal(styles[0].textContent.includes('DejaVu Serif'), false);
    assert.equal(styles[0].textContent.includes('Roboto'), true);
    assert.equal(styles[0].textContent.includes('정부상징 부처명_16040911'), true);
    assert.equal(styles[0].textContent.includes('KoPub돋움체 Medium'), true);
    assert.equal(styles[0].textContent.includes('KoPubBatangMedium'), true);
    assert.equal(styles[0].textContent.includes('KoPubWorld돋움체 Medium'), true);
    assert.equal(styles[0].textContent.includes('경기천년제목 Medium'), true);
    assert.equal(styles[0].textContent.includes('62570체'), true);
    assert.equal(
      debugLogs.some(log => log.includes('시스템 글꼴 사용') && log.includes('DejaVu Serif')),
      true,
    );
    assert.equal(
      debugLogs.some(log => (
        log.includes('CDN 로드 성공')
        && log.includes('KoPubWorld돋움체 Medium')
        && log.includes('KoPubWorld-Dotum-Medium.woff2')
      )),
      true,
    );
    assert.equal(
      fontFaceRequests.some(request => (
        request.source.includes('korea-government-symbol-font@v1.0.0/fonts/Government_16040911.ttf')
        && request.source.includes("format('truetype')")
      )),
      true,
    );
    assert.equal(
      fontFaceRequests.some(request => (
        request.source.includes('font-kopub@1.0.2/fonts/KoPubDotum-Medium.woff')
        && request.source.includes("format('woff')")
      )),
      true,
    );
    assert.equal(
      fontFaceRequests.some(request => request.source.includes(
        'font-kopub@1.0.2/fonts/KoPubBatang-Medium.woff',
      )),
      true,
    );
    assert.equal(
      fontFaceRequests.some(request => (
        request.source.includes('font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Medium.woff2')
        && request.source.includes("format('woff2')")
      )),
      true,
    );
    assert.equal(
      fontFaceRequests.some(request => request.source.includes(
        'projectnoonnu/2410-3@1.0/Title_Medium.woff',
      )),
      true,
    );
    assert.equal(
      fontFaceRequests.some(request => request.source.includes(
        '@noonnu/62570che@0.1.0/fonts/62570-normal.woff',
      )),
      true,
    );
  } finally {
    Object.defineProperty(globalThis, 'document', {
      configurable: true,
      value: previousDocument,
    });
    Object.defineProperty(globalThis, 'FontFace', {
      configurable: true,
      value: previousFontFace,
    });
    console.debug = previousConsoleDebug;
  }
});
