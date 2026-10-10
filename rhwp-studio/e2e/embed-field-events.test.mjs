/**
 * E2E 테스트 — 누름틀 진입·이탈 이벤트(field-focus-events-v1)가 부모 페이지까지 온다
 *
 * 검증 항목:
 * 1. createEditor 로 띄운 studio 가 capability 를 광고한다
 * 2. 캐럿이 누름틀에 들어가면 이름·안내문이 실린 inField:true 이벤트가 온다
 * 3. 같은 누름틀 안의 이동은 다시 알리지 않는다
 * 4. 다른 누름틀로 옮기면 그 누름틀, 밖으로 나오면 inField:false
 * 5. 해지 후에는 오지 않는다
 */
import { resolve } from 'path';

import { runTest, assert } from './helpers.mjs';

const EDITOR_MODULE_PATH = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const EDITOR_MODULE_URL = EDITOR_MODULE_PATH.startsWith('/')
  ? `/@fs${EDITOR_MODULE_PATH}`
  : `/@fs/${EDITOR_MODULE_PATH}`;
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

runTest('누름틀 진입·이탈 이벤트', async ({ page }) => {
  await page.goto(`${VITE_URL}/@vite/client`, { waitUntil: 'domcontentloaded' });

  const result = await page.evaluate(async (editorModuleUrl) => {
    const { createEditor } = await import(editorModuleUrl);
    const host = document.createElement('div');
    host.style.cssText = 'width: 100vw; height: 100vh';
    document.body.appendChild(host);
    const editor = await createEditor(host, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
    });
    const sample = await fetch('/samples/field-01.hwp').then((r) => r.arrayBuffer());
    await editor.loadFile(sample, 'field-01.hwp');

    const events = [];
    const off = editor.onFieldChanged((event) => events.push(event));
    const settle = () => new Promise((r) => setTimeout(r, 150));
    // 같은 출처라 iframe 안 입력 처리기로 캐럿을 옮긴다(사용자 클릭과 같은 갱신 경로).
    const ih = editor.element.contentWindow.__inputHandler;
    const moveTo = async (paragraphIndex, charOffset) => {
      ih.moveCursorTo({ sectionIndex: 0, paragraphIndex, charOffset });
      await settle();
    };
    const fields = editor.element.contentWindow.__wasm.getFieldList();
    const company = fields.find((f) => f.name === '회사명');
    const writer = fields.find((f) => f.name === '작성자');

    await moveTo(company.location.paraIndex, company.startCharIdx);
    const afterEnter = events.length;
    await moveTo(company.location.paraIndex, company.startCharIdx);
    const afterSame = events.length;
    await moveTo(writer.location.paraIndex, writer.startCharIdx);
    await moveTo(0, 0);
    const beforeOff = events.length;
    off();
    await moveTo(company.location.paraIndex, company.startCharIdx);

    const studioCapabilities = [...editor._transport._peerCapabilities];
    editor.destroy();
    return {
      events, afterEnter, afterSame, beforeOff, finalCount: events.length,
      studioCapabilities, company, writer,
    };
  }, EDITOR_MODULE_URL);

  assert(result.studioCapabilities.includes('field-focus-events-v1'), 'TC1: studio 가 capability 광고');
  assert(result.afterEnter === 1, `TC2: 진입 이벤트 1건 (${result.afterEnter})`);
  const [enter, change, leave] = result.events;
  assert(enter?.inField === true && enter.name === '회사명' && enter.guide === result.company.guide
      && enter.fieldId === result.company.fieldId && enter.fieldType === 'clickhere',
    `TC2: 이름·안내문·아이디 (${JSON.stringify(enter)})`);
  assert(result.afterSame === 1, `TC3: 같은 누름틀 재진입은 무알림 (${result.afterSame})`);
  assert(change?.inField === true && change.name === '작성자', `TC4: 다른 누름틀 (${JSON.stringify(change)})`);
  assert(leave?.inField === false && Object.keys(leave).length === 2, `TC4: 이탈 (${JSON.stringify(leave)})`);
  assert(result.beforeOff === 3 && result.finalCount === 3, `TC5: 해지 후 무알림 (${result.finalCount})`);
});
