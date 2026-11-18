// 검사 유형: 관계형 — 실제 설치 글꼴이나 페이지 픽셀에 의존하지 않는 face 소유 계약.
// 주석 갱신: 2026-11-18T16:20:20+09:00; 직전 파일 변경 f8234f6f90 2026-11-15T08:17:05+09:00.
import assert from 'node:assert/strict';
import test, { before, after } from 'node:test';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';
import { closestLocalFontStyle } from '../src/view/canvaskit/local-font-style.ts';

let vite: ViteDevServer;
let prototype: Record<string, any>;
let localFonts: Record<string, any>;
before(async () => {
  vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  prototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
  localFonts = await vite.ssrLoadModule('/src/core/local-fonts.ts');
});
after(async () => { await vite?.close(); });

function face(id: string, weightClass = 400, italic = false) {
  return { id, record: { family: 'Contract Family', fullName: id, postscriptName: id,
    style: 'Regular', displayName: id, aliases: ['Contract Family'], weightClass, italic } };
}

test('동일 스타일의 다른 윤곽이 먼저 준비돼도 공통 resolver의 face를 유지한다', () => {
  const original = face('Preferred-TrueType');
  const alternate = face('Alternate-OpenType');
  for (const candidates of [[alternate, original], [original, alternate]]) {
    assert.equal(closestLocalFontStyle(original, candidates, false, false), original);
    assert.equal(closestLocalFontStyle(original, candidates, true, false), original);
  }
});

test('더 정확한 굵기·기울기 face는 기존처럼 선택하며 다른 family는 호출자가 제외한다', () => {
  const regular = face('Regular');
  const bold = face('Bold', 700);
  const italic = face('Italic', 400, true);
  assert.equal(closestLocalFontStyle(regular, [bold, italic], true, false), bold);
  assert.equal(closestLocalFontStyle(regular, [bold, italic], false, true), italic);
  assert.equal(closestLocalFontStyle(null, [], false, false), null);
});

test('실제 host 준비 뒤 분할 family의 가용성·일반 글리프·선언 대체 순서가 일치한다', async () => {
  const faces = ['Latin', 'Hangul', 'Fallback'].map((name, index) => ({
    id: name, family: index < 2 ? 'Split Original' : 'Declared Fallback',
    fullName: name, postscriptName: name, style: 'Regular', weight: 400, slant: 'normal',
  }));
  const reads: string[] = [];
  const renderer = Object.create(prototype);
  const typefaces = faces.map((face, index) => ({ name: face.id, chars: ['A', '가', '※'][index] }));
  Object.assign(renderer, { documentGeneration: 1, disposed: false,
    localTypefaces: new Map(), localTypefacePending: new Map(), localTypefaceLoadFailures: new Set(),
    fallbackFamilyCache: new Map(), bundledTypefaceAliases: new Map(),
    defaultFontFamily: null, defaultTypeface: null, defaultFontManager: null,
    paragraphFonts: { remember() {} },
    invalidateFontCaches() { this.fallbackFamilyCache.clear(); },
    canvasKit: { Typeface: { MakeFreeTypeFaceFromData(bytes: ArrayBuffer) {
      return typefaces[new Uint8Array(bytes)[0]];
    } }, FontMgr: { FromData() { return null; } } },
  });
  try {
    await localFonts.setHostFontProvider({
      getSnapshot: async () => ({ revision: 'prepared-split', faces }),
      readFace: async (id: string) => {
        reads.push(id);
        return { bytes: Uint8Array.of(faces.findIndex(face => face.id === id)).buffer };
      },
      subscribe: () => () => {},
    });
    const regular = { weight: 400, slant: 'normal' };
    const records = faces.flatMap(face => localFonts.resolveRendererLocalFontCandidates(face.postscriptName, regular));
    assert.equal(await renderer.prepareHostFonts(records, (bytes: ArrayBuffer) => bytes), 3);
    assert.deepEqual(reads, ['Latin', 'Hangul', 'Fallback']);
    assert.equal(localFonts.resolveRendererLocalFont('Split Original', regular), null,
      '분할 face를 단일 OS face로 확정하지 않는다');
    assert.equal(renderer.hasPreparedFontFamily('Split Original'), true);
    const chain = '"Split Original", "Declared Fallback"';
    const candidates = renderer.ordinaryTextTypefaces(chain, {});
    assert.deepEqual(candidates.map((face: any) => face.typeface.name), ['Latin', 'Hangul', 'Fallback']);
    assert.deepEqual([...('A가※')].map(ch => candidates.find((face: any) => face.typeface.chars.includes(ch))?.typeface.name),
      ['Latin', 'Hangul', 'Fallback'], 'glyph coverage는 준비한 원본 분할 face를 선언 대체보다 먼저 사용한다');
    assert.deepEqual(renderer.findPreparedFamilyFallbacks(chain, {}).map((face: any) => face.typeface.name),
      ['Hangul', 'Fallback']);
    assert.deepEqual(renderer.findPreparedTypeface('Split Original', {}).paragraphTypefaces, typefaces.slice(0, 2),
      '문단 shaping도 같은 분할 프로그램 전체를 받는다');
    assert.equal(renderer.findPreparedTypeface('Hangul', {}).typeface, typefaces[1], '명시 PostScript는 한 원본만 선택한다');
    assert.equal(renderer.hasPreparedFontFamily('Absent Family'), false, '준비되지 않은 family의 guard는 유지한다');
  } finally { await localFonts.setHostFontProvider(null); }
});

test('중복 PostScript 식별자의 모호성을 준비된 임의 face로 덮어쓰지 않는다', async () => {
  try {
    await localFonts.setHostFontProvider({
      getSnapshot: async () => ({ revision: 'ambiguous-split', faces: ['a', 'b'].map(id => ({
        id, family: 'Ambiguous', fullName: id, postscriptName: 'Duplicate',
        style: 'Regular', weight: 400, slant: 'normal',
      })) }),
      readFace: async () => null, subscribe: () => () => {},
    });
    const renderer = Object.create(prototype);
    Object.assign(renderer, { localTypefaces: new Map(), fallbackFamilyCache: new Map(),
      bundledTypefaceAliases: new Map(), defaultFontFamily: null });
    assert.equal(renderer.hasPreparedFontFamily('Ambiguous'), false);
    assert.deepEqual(renderer.ordinaryTextTypefaces('Ambiguous', {}), []);
  } finally { await localFonts.setHostFontProvider(null); }
});
