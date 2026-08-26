/**
 * E2E 테스트 — 플러그인 호스트 수명과 트랜잭션 계약
 *
 * 검증 항목:
 * 1. allowlist 밖은 거절 / 안은 로드
 * 2. 읽기(read)는 히스토리를 건드리지 않는다
 * 3. 트랜잭션 1건 = undo 1스텝 (배치 100회를 undo 1번으로 되돌림)
 * 4. 중첩 트랜잭션 거절(NESTED_TX)
 * 5. 본문이 던지면 롤백 — 문서는 진입 시점 그대로
 * 6. unload 가 등록물을 걷어가고 studio 는 살아 있다
 * 7. unload 후 invoke 는 PLUGIN_NOT_LOADED
 *
 * 계획: mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md §7(P2)
 */

import {
  runTest, createNewDocument, clickEditArea, typeText, screenshot, assert, getParaText,
} from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const paraText = (page) => getParaText(page, 0, 0, 500);

runTest('플러그인 호스트', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, '기준');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  // ── TC1: allowlist ────────────────────────────────────
  const load = await page.evaluate(async () => {
    const p = window.rhwpStudio.plugins;
    let denied = null;
    try { await p.load('not-allowed'); } catch (e) { denied = e.code; }
    const ok = await p.load('dev-probe');
    return { denied, ok, list: p.list(), ping: p.invoke('dev-probe', 'ping', []) };
  });
  assert(load.denied === 'PLUGIN_NOT_ALLOWED', `TC1: allowlist 밖 거절 (${load.denied})`);
  assert(load.list.length === 1 && load.list[0].id === 'dev-probe', 'TC1: 로드 목록에 반영');
  assert(load.ping === 'pong', 'TC1: 표면 호출 동작');
  assert(load.ok.methods.includes('appendRuns'), 'TC1: 표면 메서드 노출');

  // ── TC1.1: 문서 교체 알림은 studio가 한 번만 보낸다 ───
  const swaps = await page.evaluate(async () => {
    const p = window.rhwpStudio.plugins;
    p.invoke('dev-probe', 'replaceWithBlank', []);
    await new Promise((resolve) => setTimeout(resolve, 300));
    return p.invoke('dev-probe', 'documentSwapCount', []);
  });
  assert(swaps === 1, `TC1.1: 문서 교체 알림 1회 (${swaps})`);

  // ── TC2: 읽기는 히스토리를 건드리지 않는다 ─────────────
  const read = await page.evaluate(() => {
    const before = window.rhwpStudio.automation.getContext().canUndo;
    const pages = window.rhwpStudio.plugins.invoke('dev-probe', 'pageCount', []);
    return { before, pages, after: window.rhwpStudio.automation.getContext().canUndo };
  });
  assert(read.pages >= 1, `TC2: 읽기 결과 (${read.pages}쪽)`);
  assert(read.before === read.after, 'TC2: 읽기가 undo 상태를 바꾸지 않음');

  // ── TC3: 트랜잭션 1건 = undo 1스텝 ─────────────────────
  const textBefore = await paraText(page);
  const written = await page.evaluate(() =>
    window.rhwpStudio.plugins.invoke('dev-probe', 'appendRuns', ['가', 100]));
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const textAfterBatch = await paraText(page);
  assert(written === 100, `TC3: 배치 100회 실행 (${written}자)`);
  assert(textAfterBatch.length > textBefore.length + 50,
    `TC3: 문서에 반영 (${textBefore.length} → ${textAfterBatch.length}자)`);

  await page.evaluate(() => window.rhwpStudio.automation.execute('edit:undo'));
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const textAfterUndo = await paraText(page);
  assert(textAfterUndo === textBefore,
    `TC3: undo 1회로 배치 전체 복원 ("${textAfterUndo}" === "${textBefore}")`);

  // ── TC4: 중첩 트랜잭션 거절 ────────────────────────────
  const nested = await page.evaluate(() => {
    try { window.rhwpStudio.plugins.invoke('dev-probe', 'nestedTransaction', []); return null; }
    catch (e) { return e.code; }
  });
  assert(nested === 'NESTED_TX', `TC4: 중첩 트랜잭션 거절 (${nested})`);

  // ── TC5: 본문이 던지면 롤백 ────────────────────────────
  const beforeThrow = await paraText(page);
  const threw = await page.evaluate(() => {
    try { window.rhwpStudio.plugins.invoke('dev-probe', 'throwingTransaction', ['롤백대상']); return null; }
    catch (e) { return e.message; }
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const afterThrow = await paraText(page);
  assert(threw !== null, `TC5: 예외가 호출자에게 전달됨 (${threw})`);
  assert(afterThrow === beforeThrow,
    `TC5: 롤백으로 문서 원상 복구 ("${afterThrow}" === "${beforeThrow}")`);

  // ── TC6: unload 가 등록물을 걷어간다 ───────────────────
  const disposed = await page.evaluate(async () => {
    const p = window.rhwpStudio.plugins;
    const a = window.rhwpStudio.automation;
    p.invoke('dev-probe', 'addProbeCommand', []);
    p.invoke('dev-probe', 'subscribeProbe', []);
    const registered = a.listCommands().some(c => c.id === 'ext:dev-probe');
    const inMenu = !!document.querySelector('.md-item[data-cmd="ext:dev-probe"]');

    await p.unload('dev-probe');

    return {
      registered, inMenu,
      stillRegistered: a.listCommands().some(c => c.id === 'ext:dev-probe'),
      stillInMenu: !!document.querySelector('.md-item[data-cmd="ext:dev-probe"]'),
      list: p.list(),
    };
  });
  assert(disposed.registered && disposed.inMenu, 'TC6: 플러그인이 커맨드·메뉴를 심음');
  assert(!disposed.stillRegistered, 'TC6: unload 후 커맨드 회수');
  assert(!disposed.stillInMenu, 'TC6: unload 후 메뉴 항목 회수');
  assert(disposed.list.length === 0, 'TC6: 로드 목록 비워짐');

  // ── TC7: unload 후 호출은 거절, studio 는 살아 있다 ────
  const afterUnload = await page.evaluate(() => {
    let code = null;
    try { window.rhwpStudio.plugins.invoke('dev-probe', 'ping', []); } catch (e) { code = e.code; }
    return code;
  });
  assert(afterUnload === 'PLUGIN_NOT_LOADED', `TC7: unload 후 호출 거절 (${afterUnload})`);

  const survived = await paraText(page);
  assert(survived === beforeThrow, `TC7: 문서 그대로 ("${survived}")`);

  await typeText(page, '살아있음');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const typedAfter = await paraText(page);
  assert(typedAfter.length > survived.length, 'TC7: unload 후에도 키보드 편집 정상');

  const menuStillWorks = await page.evaluate(() =>
    window.rhwpStudio.automation.execute('edit:select-all').ok);
  assert(menuStillWorks, 'TC7: unload 후에도 커맨드 실행 정상');

  await screenshot(page, 'plugin-lifecycle-final');
});
