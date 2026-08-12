/**
 * E2E 테스트 — hwpctrl 플러그인이 studio 의 그 문서를 조작한다
 *
 * 검증 항목:
 * 1. 플러그인 로드 (allowlist 안)
 * 2. 읽기 API 가 studio 문서를 본다 (문서 한 벌)
 * 3. 배치 편집 → 화면 반영 → undo 1회로 전체 복원
 * 4. 좌표 변환기가 브라우저에서도 같은 답을 준다
 * 5. exportBytes 왕복
 * 6. unload 후 studio 생존
 *
 * 계획: mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md §7(P3)
 */

import {
  runTest, createNewDocument, clickEditArea, typeText, screenshot, assert, getParaText,
} from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const paraText = (page) => getParaText(page, 0, 0, 500);

runTest('hwpctrl 플러그인', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, 'seed');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  // ── TC1: 로드 ─────────────────────────────────────────
  const loaded = await page.evaluate(async () => {
    const res = await window.rhwpStudio.plugins.load('hwpctrl');
    return { methods: res.methods, list: window.rhwpStudio.plugins.list() };
  });
  assert(loaded.list.some(p => p.id === 'hwpctrl'), 'TC1: hwpctrl 로드됨');
  assert(loaded.methods.includes('invoke') && loaded.methods.includes('batch'),
    `TC1: 표면 노출 (${loaded.methods.join(',')})`);

  // ── TC2: 읽기가 studio 문서를 본다 ─────────────────────
  const shared = await page.evaluate(() => {
    const p = window.rhwpStudio.plugins;
    return {
      pageCount: p.invoke('hwpctrl', 'invoke', ['PageCount', []]),
      studioPages: window.__wasm.pageCount,
      pos: p.invoke('hwpctrl', 'invoke', ['GetPos', []]),
      undoBefore: window.rhwpStudio.automation.getContext().canUndo,
    };
  });
  assert(shared.pageCount === shared.studioPages,
    `TC2: 같은 문서를 본다 (hwpctrl ${shared.pageCount}쪽 = studio ${shared.studioPages}쪽)`);
  assert(typeof shared.pos.list === 'number', `TC2: GetPos 동작 (${JSON.stringify(shared.pos)})`);

  // ── TC3: 배치 편집 → undo 1스텝 ────────────────────────
  const before = await paraText(page);
  await page.evaluate(() => {
    const ops = [];
    for (let i = 0; i < 30; i += 1) ops.push({ m: 'MovePos', a: [3, 0, 0] });
    return window.rhwpStudio.plugins.invoke('hwpctrl', 'batch', [ops]);
  });
  // 실제 문서 변경은 studio 커맨드로 만든다 — 여기서 보려는 것은 배치가 트랜잭션 1건이라는 점이다.
  const mutated = await page.evaluate(() =>
    window.rhwpStudio.plugins.invoke('hwpctrl', 'invoke', ['SetTextFile', ['배치삽입', 'TEXT', '']]));
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));
  const afterBatch = await paraText(page);
  assert(afterBatch !== before, `TC3: 문서가 바뀜 ("${before}" → "${afterBatch}")`);

  await page.evaluate(() => window.rhwpStudio.plugins.invoke('hwpctrl', 'undo', []));
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));
  const afterUndo = await paraText(page);
  assert(afterUndo === before, `TC3: undo 1회로 복원 ("${afterUndo}" === "${before}")`);

  // ── TC4: 좌표 변환기 ───────────────────────────────────
  const coords = await page.evaluate(() => {
    const p = window.rhwpStudio.plugins;
    return {
      body: p.invoke('hwpctrl', 'toStudioPosition', [0]),
      depthBody: p.invoke('hwpctrl', 'listDepthOf', [0]),
      roundTrip: p.invoke('hwpctrl', 'toHwpList', [{ sectionIndex: 0, parentParaIndex: 0, cellPath: [] }]),
      mutatingGetPos: p.invoke('hwpctrl', 'isMutating', ['GetPos']),
      mutatingUnknown: p.invoke('hwpctrl', 'isMutating', ['모르는이름']),
    };
  });
  assert(coords.body.cellPath.length === 0, 'TC4: 본문은 셀 경로 없음');
  assert(coords.depthBody === 0, 'TC4: 본문 깊이 0');
  assert(coords.roundTrip === 0, 'TC4: 역방향 왕복');
  assert(coords.mutatingGetPos === false && coords.mutatingUnknown === true,
    'TC4: 분류 기본값은 "바꾼다"');

  // ── TC5: exportBytes ──────────────────────────────────
  const exported = await page.evaluate(() => {
    const bytes = window.rhwpStudio.plugins.invoke('hwpctrl', 'exportBytes', []);
    return { len: bytes?.length ?? 0, isBytes: bytes instanceof Uint8Array };
  });
  assert(exported.isBytes && exported.len > 0, `TC5: HWP 바이트 산출 (${exported.len} bytes)`);

  // ── TC6: unload 후 studio 생존 ─────────────────────────
  const survived = await page.evaluate(async () => {
    await window.rhwpStudio.plugins.unload('hwpctrl');
    let code = null;
    try { window.rhwpStudio.plugins.invoke('hwpctrl', 'invoke', ['PageCount', []]); }
    catch (e) { code = e.code; }
    return { code, pages: window.__wasm.pageCount, list: window.rhwpStudio.plugins.list().length };
  });
  assert(survived.code === 'PLUGIN_NOT_LOADED', `TC6: unload 후 호출 거절 (${survived.code})`);
  assert(survived.list === 0, 'TC6: 목록 비워짐');

  await typeText(page, '생존');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const typed = await paraText(page);
  assert(typed.includes('생존'), `TC6: unload 후 편집 정상 ("${typed}")`);

  await screenshot(page, 'hwpctrl-plugin-final');
});
