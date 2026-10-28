import fs from 'node:fs';
import { assert, closeBrowser, closePage, createPage, launchBrowser, loadApp } from './helpers.mjs';

// 저장소의 MIT 합성 글꼴을 메모리에서만 바꿔 같은 glyph를 보충 사설 문자에 연결한다.
// 글꼴 파일을 새로 만들거나 기여자의 시스템 글꼴을 배포하지 않는다.
const source = fs.readFileSync(new URL('../../tests/fixtures/fonts/RHWPHostFixture-Regular.ttf', import.meta.url));
const records = source.readUInt16BE(4);
const findRecord = tag => {
  for (let i = 0; i < records; i++) {
    const offset = 12 + i * 16;
    if (source.toString('ascii', offset, offset + 4) === tag) return offset;
  }
  throw new Error(`글꼴 테이블 없음: ${tag}`);
};
const cmapRecord = findRecord('cmap');
const headOffset = source.readUInt32BE(findRecord('head') + 8);
const cmap = Buffer.alloc(40);
cmap.writeUInt16BE(1, 2);
cmap.writeUInt16BE(3, 4);
cmap.writeUInt16BE(10, 6);
cmap.writeUInt32BE(12, 8);
cmap.writeUInt16BE(12, 12);
cmap.writeUInt32BE(28, 16);
cmap.writeUInt32BE(1, 24);
cmap.writeUInt32BE(0xf02fc, 28);
cmap.writeUInt32BE(0xf02fc, 32);
// 이 자체 제작 fixture에서 A는 hmtx 900/1000의 첫 번째 실제 glyph다.
cmap.writeUInt32BE(1, 36);
const offset = Math.ceil(source.length / 4) * 4;
const mapped = Buffer.alloc(offset + cmap.length);
source.copy(mapped);
cmap.copy(mapped, offset);
const checksum = data => {
  let sum = 0;
  for (let i = 0; i < data.length; i += 4) {
    let word = 0;
    for (let j = 0; j < 4; j++) word = (word * 256 + (data[i + j] ?? 0)) >>> 0;
    sum = (sum + word) >>> 0;
  }
  return sum;
};
mapped.writeUInt32BE(checksum(cmap), cmapRecord + 4);
mapped.writeUInt32BE(offset, cmapRecord + 8);
mapped.writeUInt32BE(cmap.length, cmapRecord + 12);
mapped.writeUInt32BE(0, headOffset + 8);
mapped.writeUInt32BE((0xb1b0afba - checksum(mapped)) >>> 0, headOffset + 8);
assert(checksum(mapped) === 0xb1b0afba, '합성 SFNT checksum');

const browser = await launchBrowser();
const page = await createPage(browser);
try {
  await loadApp(page, '?renderer=canvaskit&canvaskitSurface=software');
  const result = await page.evaluate(async ({ original, supplied }) => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const originalUrl = URL.createObjectURL(new Blob([new Uint8Array(original)], { type: 'font/ttf' }));
    const suppliedUrl = URL.createObjectURL(new Blob([new Uint8Array(supplied)], { type: 'font/ttf' }));
    const renderer = await CanvasKitLayerRenderer.create('default', 'software', { requirePreparedFontFamilies: true });
    try {
      const registered = await renderer.prepareBundledFonts([
        { url: originalUrl, aliases: ['RHWP Original Synthetic'] },
        { url: suppliedUrl, aliases: ['RHWP Supplied Synthetic'] },
      ]);
      const paint = (family, text, displayText) => {
        const bounds = { x: 0, y: 0, width: 100, height: 60 };
        const op = {
          type: 'textRun', text, bbox: { x: 20, y: 15, width: 18, height: 20 },
          baseline: 17, positions: [0, 18], style: { fontFamily: family, fontSize: 20, color: '#000000' },
          ...(displayText ? { displayText, displayPositions: [0, 18] } : {}),
        };
        const tree = { pageIndex: 0, pageWidth: 100, pageHeight: 60, profile: 'print',
          root: { kind: 'leaf', bounds, ops: [op] } };
        const canvas = document.createElement('canvas');
        canvas.width = 100; canvas.height = 60;
        const png = renderer.renderPage(tree, canvas, 1).toDataURL('image/png');
        return { png, diagnostics: renderer.diagnostics() };
      };
      const originalGlyph = paint('RHWP Original Synthetic', 'A');
      const suppliedGlyph = paint('RHWP Supplied Synthetic', '\u{f02fc}');
      const fallbackGlyph = paint('RHWP Supplied Synthetic', '\u{f02fc}', '►');
      return { registered, sameOutline: originalGlyph.png === suppliedGlyph.png,
        fallbackDifferent: fallbackGlyph.png !== suppliedGlyph.png,
        supplied: suppliedGlyph.diagnostics, fallback: fallbackGlyph.diagnostics };
    } finally {
      renderer.dispose(); URL.revokeObjectURL(originalUrl); URL.revokeObjectURL(suppliedUrl);
    }
  }, { original: Array.from(source), supplied: Array.from(mapped) });
  assert(result.registered === 2, '합성 대조 글꼴 두 개를 실제 준비한다');
  assert(result.sameOutline, '공급된 보충 사설 글리프는 같은 원본 윤곽으로 출력한다');
  assert(result.fallbackDifferent, '대체 표시와 실제 공급 글리프를 구분한다');
  assert(result.supplied.lastUnsupportedOps.length === 0, '공급된 글리프는 미지원으로 처리하지 않는다');
  assert(result.supplied.fontSubstitutions.length === 0, '공급된 글리프를 다른 글꼴로 대체하지 않는다');
  assert(result.fallback.fontSubstitutions.some(item => item.source === 'missingGlyphSymbol'), '대체 글리프 공급 경로를 확인한다');
  console.log(JSON.stringify({ registered: result.registered, sameOutline: result.sameOutline,
    fallbackDifferent: result.fallbackDifferent, suppliedUnsupported: result.supplied.lastUnsupportedOps }));
} finally {
  await closePage(page);
  await closeBrowser(browser);
}
