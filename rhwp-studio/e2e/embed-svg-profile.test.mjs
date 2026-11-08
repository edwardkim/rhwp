/** 실제 SDK의 SVG profile 전달·입력값 보존·인쇄 안내문 제외(#7715). */
import { resolve } from 'node:path';
import { mkdirSync, writeFileSync } from 'node:fs';
import { runTest, assert } from './helpers.mjs';

const modulePath = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const moduleUrl = modulePath.startsWith('/') ? `/@fs${modulePath}` : `/@fs/${modulePath}`;
const viteUrl = process.env.VITE_URL || 'http://localhost:7700';

runTest('SDK SVG 인쇄 profile', async ({ page }) => {
  await page.goto(`${viteUrl}/@vite/client`, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async ({ url, collect }) => {
    const { createEditor } = await import(url);
    const host = document.createElement('div');
    document.body.appendChild(host);
    const editor = await createEditor(host, { studioUrl: `${location.origin}/`, handshakeTimeoutMs: 30_000 });
    try {
      const bytes = await fetch('/samples/field-01.hwp').then(r => r.arrayBuffer());
      await editor.loadFile(bytes, 'field-01.hwp');
      const pageCount = await editor.pageCount();
      const win = editor.element.contentWindow;
      const numberPositions = [];
      const visit = (node) => {
        if (node.type === 'TextRun' && ['01', '02', '03', '04', '05'].includes(node.text?.trim())) {
          numberPositions.push({ text: node.text.trim(), x: node.bbox.x, y: node.bbox.y });
        }
        for (const child of node.children ?? []) visit(child);
      };
      visit(JSON.parse(win.__wasm.borrowDocumentHandle().getPageRenderTree(1)));
      const before = JSON.stringify(win.__wasm.getFieldList());
      const undoBefore = win.rhwpStudio.automation.getContext().canUndo;
      const omitted = await editor.getPageSvg(1);
      const legacy = win.__wasm.renderPageSvg(1);
      const texts = (svg) => [...new DOMParser().parseFromString(svg, 'image/svg+xml').querySelectorAll('text')]
        .map(node => node.textContent).join('').replace(/\s+/g, '');
      const profiles = {};
      for (const profile of ['fastPreview', 'screen', 'print', 'highQuality']) {
        const svg = await editor.getPageSvg(1, { profile });
        profiles[profile] = { svg: svg.startsWith('<svg') || svg.includes('<svg'), text: texts(svg) };
      }
      let invalid = false;
      try { await editor.getPageSvg(1, { profile: 'invalid-profile' }); } catch { invalid = true; }
      return {
        evidence: collect ? await Promise.all(Array.from({ length: pageCount }, (_, index) => editor.getPageSvg(index, { profile: 'print' }))) : null,
        legacyEqual: omitted === legacy, profiles, invalid, numberPositions, pageCount,
        valuesUnchanged: before === JSON.stringify(win.__wasm.getFieldList()),
        undoUnchanged: undoBefore === win.rhwpStudio.automation.getContext().canUndo,
      };
    } finally {
      editor.destroy();
    }
  }, { url: moduleUrl, collect: !!process.env.RHWP_SVG_PROFILE_EVIDENCE_DIR });
  if (result.evidence) {
    const dir = resolve(process.env.RHWP_SVG_PROFILE_EVIDENCE_DIR);
    mkdirSync(dir, { recursive: true });
    result.evidence.forEach((svg, index) => writeFileSync(resolve(dir, `sdk_${String(index + 1).padStart(3, '0')}.svg`), svg));
    delete result.evidence;
  }
  console.log(JSON.stringify(result));
  assert(result.pageCount === 3, 'TC9: SDK 문서 전체 쪽수도 독립 Print와 일치');
  assert(result.numberPositions.map(p => p.text).join(',') === '01,02,03,04,05'
      && result.numberPositions.every(p => Math.abs(p.x - 86.673024) <= 2)
      && result.numberPositions.every((p, index, items) => index === 0 || items[index - 1].y < p.y),
    'TC8: 완료된 2쪽 독립 Print의 번호 정렬 ±2px 및 내용 순서');
  assert(result.legacyEqual, 'TC1: profile 생략 시 기존 SVG 경로');
  assert(Object.values(result.profiles).every(p => p.svg), 'TC2: 허용 profile 네 가지 실제 SVG');
  assert(result.invalid, 'TC3: 잘못된 profile 거절');
  assert(result.valuesUnchanged && result.undoUnchanged, 'TC4: 출력은 값과 Undo를 바꾸지 않음');
  assert(!result.profiles.print.text.includes('목차입력'), 'TC5: Print에 초기 안내문 없음');
  assert(result.profiles.screen.text.includes('목차입력'), 'TC6: Screen 안내문 유지');
  assert(['01', '02', '03', '04', '05'].every(text => result.profiles.print.text.includes(text)),
    'TC7: Print의 실제 번호 내용 보존');
}, { skipLoadApp: true });
