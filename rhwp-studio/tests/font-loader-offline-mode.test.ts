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
