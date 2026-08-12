/**
 * E2E 테스트 — 자동화 표면 (window.rhwpStudio.automation)
 *
 * 검증 항목:
 * 1. 커맨드 질의(listCommands) — 레지스트리 전체
 * 2. 메뉴 모델(getMenuModel) — DOM 파생, 최상위 8개
 * 3. 드리프트 가드 — 마크업의 data-cmd 가 전부 레지스트리에 있음
 * 4. 실행(execute) — 판정 반환, 문서에 실제 반영
 * 5. 게이트 동일성 — 미등록·비활성이 사유와 함께 거절
 * 6. ext: 커맨드 등록·메뉴 삽입·회수(disposeOwned)
 *
 * 계획: mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md §7(P1)
 */

import {
  runTest, createNewDocument, clickEditArea, typeText, screenshot, assert,
} from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

runTest('자동화 표면', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  // ── TC1: 표면 노출과 커맨드 질의 ──────────────────────────
  const surface = await page.evaluate(() => {
    const a = window.rhwpStudio?.automation;
    if (!a) return null;
    const cmds = a.listCommands();
    return {
      count: cmds.length,
      hasCopy: cmds.some(c => c.id === 'edit:copy'),
      // 메뉴 마크업에 없는 커맨드도 레지스트리에 있으면 나와야 한다
      hasFieldEdit: cmds.some(c => c.id === 'field:edit'),
      sample: cmds.find(c => c.id === 'edit:copy'),
    };
  });
  assert(surface !== null, 'TC1: window.rhwpStudio.automation 노출됨');
  assert(surface.count > 100, `TC1: 커맨드 질의 (${surface.count}개)`);
  assert(surface.hasCopy, 'TC1: edit:copy 포함');
  assert(surface.hasFieldEdit, 'TC1: 메뉴에 없는 커맨드(field:edit)도 목록에 포함');
  assert(typeof surface.sample.enabled === 'boolean', 'TC1: 활성 여부 동반');

  // ── TC2: 메뉴 모델은 DOM 파생 ─────────────────────────
  const menu = await page.evaluate(() => {
    const model = window.rhwpStudio.automation.getMenuModel();
    return {
      topLevel: model.map(m => m.menuId),
      fileItemCount: model.find(m => m.menuId === 'file')?.items.length ?? 0,
      hasSubmenu: model.some(m => m.items.some(i => Array.isArray(i.submenu) && i.submenu.length > 0)),
    };
  });
  assert(menu.topLevel.length === 8, `TC2: 최상위 메뉴 8개 (${menu.topLevel.join(',')})`);
  assert(menu.fileItemCount > 0, `TC2: 파일 메뉴 항목 파싱 (${menu.fileItemCount}개)`);
  assert(menu.hasSubmenu, 'TC2: 서브메뉴 파싱');

  // ── TC3: 마크업 ↔ 레지스트리 드리프트 0 ────────────────
  const drift = await page.evaluate(() => {
    const ids = new Set();
    document.querySelectorAll('[data-cmd]').forEach(el => ids.add(el.dataset.cmd));
    const known = new Set(window.rhwpStudio.automation.listCommands().map(c => c.id));
    return { markup: ids.size, missing: [...ids].filter(id => !known.has(id)) };
  });
  assert(
    drift.missing.length === 0,
    `TC3: 마크업 data-cmd ${drift.markup}개가 모두 레지스트리에 있음` +
      (drift.missing.length ? ` — 누락: ${drift.missing.join(', ')}` : ''),
  );

  // ── TC4: 실행이 문서에 반영된다 ────────────────────────
  await typeText(page, '자동화');
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));

  const executed = await page.evaluate(() => {
    const a = window.rhwpStudio.automation;
    const before = a.getContext().canUndo;
    const result = a.execute('edit:select-all');
    return { before, result, hasSelection: a.getContext().hasSelection };
  });
  assert(executed.result.ok === true, `TC4: execute 판정 ok (${JSON.stringify(executed.result)})`);
  assert(executed.hasSelection, 'TC4: 실행이 실제 상태를 바꿈 (선택 생성)');

  // ── TC5: 거절은 사유와 함께 ────────────────────────────
  const refused = await page.evaluate(() => {
    const a = window.rhwpStudio.automation;
    return {
      unknown: a.execute('nope:does-not-exist'),
      enabledKnown: a.isEnabled('edit:copy'),
      enabledUnknown: a.isEnabled('nope:does-not-exist'),
    };
  });
  assert(refused.unknown.ok === false && refused.unknown.reason === 'unregistered',
    `TC5: 미등록 커맨드가 사유와 함께 거절 (${JSON.stringify(refused.unknown)})`);
  assert(refused.enabledUnknown === false, 'TC5: 미등록 커맨드는 비활성');

  // ── TC5b: 다이얼로그 정책 ──────────────────────────────
  const dialogPolicy = await page.evaluate(() => {
    const a = window.rhwpStudio.automation;
    const info = a.listCommands().find(c => c.id === 'edit:goto');
    return {
      flagged: info?.opensDialog === true,
      refused: a.execute('edit:goto'),
      allowed: a.execute('edit:goto', undefined, { allowDialog: true }),
      dialogOpen: !!document.querySelector('.dialog, .goto-dialog, .find-dialog'),
    };
  });
  assert(dialogPolicy.flagged, 'TC5b: opensDialog 표기가 목록에 실림');
  assert(dialogPolicy.refused.ok === false && dialogPolicy.refused.reason === 'needs-dialog',
    `TC5b: 기본은 거절 (${JSON.stringify(dialogPolicy.refused)})`);
  assert(dialogPolicy.allowed.ok === true, `TC5b: allowDialog 로 실행 (${JSON.stringify(dialogPolicy.allowed)})`);
  await page.keyboard.press('Escape');
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));

  // ── TC6: ext: 커맨드 등록·메뉴 삽입·회수 ───────────────
  const ext = await page.evaluate(() => {
    const a = window.rhwpStudio.automation;
    let ran = 0;

    let prefixRejected = false;
    try { a.registerCommand({ id: 'bad:id', label: 'x', execute() {} }); }
    catch (e) { prefixRejected = e.code === 'EXT_PREFIX_REQUIRED'; }

    a.registerCommand({ id: 'ext:probe', label: '자동화 시험', execute() { ran += 1; } });
    a.addMenuItem({ menuId: 'tool', commandId: 'ext:probe' });

    const inMenu = !!document.querySelector('.md-item[data-cmd="ext:probe"]');
    const result = a.execute('ext:probe');
    const inModel = a.getMenuModel()
      .find(m => m.menuId === 'tool')?.items.some(i => i.commandId === 'ext:probe') ?? false;

    // 클릭 경로도 같은 커맨드를 태우는가 (마크업 계약 준수 확인)
    document.querySelector('.md-item[data-cmd="ext:probe"]').click();

    a.disposeOwned();
    return {
      prefixRejected, inMenu, inModel, result, ran,
      afterDispose: !!document.querySelector('.md-item[data-cmd="ext:probe"]'),
      stillRegistered: a.listCommands().some(c => c.id === 'ext:probe'),
    };
  });
  assert(ext.prefixRejected, 'TC6: ext: 접두사 위반 거절');
  assert(ext.inMenu, 'TC6: 메뉴에 항목 삽입됨');
  assert(ext.inModel, 'TC6: 메뉴 모델에 반영됨');
  assert(ext.result.ok === true && ext.ran >= 1, `TC6: 확장 커맨드 실행 (${ext.ran}회)`);
  assert(ext.ran >= 2, `TC6: 메뉴 클릭도 같은 커맨드를 태움 (${ext.ran}회)`);
  assert(!ext.afterDispose, 'TC6: disposeOwned 후 메뉴 항목 회수');
  assert(!ext.stillRegistered, 'TC6: disposeOwned 후 커맨드 해제');

  await screenshot(page, 'automation-final');
});
