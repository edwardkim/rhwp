/**
 * E2E 테스트 — Studio Bridge (createStudio) 부모 페이지 제어
 *
 * 부모 페이지에서 iframe 안의 studio 를 JavaScript 만으로 조종한다. 전 경로를 밟는다:
 * SDK → MessageChannel → rpc-router → automation/plugin host → hwpctrl.
 *
 * 검증 항목:
 * 1. createStudio 로 div 에 심고 플러그인·chrome 초기값 적용
 * 2. 커맨드 질의·실행 (부모에서)
 * 3. chrome 토글 — 숨겨도 커맨드는 실행된다
 * 4. hwpctrl 배치 — 한 메시지, 한 트랜잭션, undo 1스텝
 * 5. exportBytes 가 부모까지 바이트로 온다
 * 6. destroy 회수 + 재생성
 *
 * 계획: mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md §7(P4)
 */
import { resolve } from 'path';

import { runTest, assert } from './helpers.mjs';

const EDITOR_MODULE_PATH = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const EDITOR_MODULE_URL = EDITOR_MODULE_PATH.startsWith('/')
  ? `/@fs${EDITOR_MODULE_PATH}`
  : `/@fs/${EDITOR_MODULE_PATH}`;
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

runTest('Studio Bridge', async ({ page }) => {
  await page.goto(`${VITE_URL}/@vite/client`, { waitUntil: 'domcontentloaded' });

  const result = await page.evaluate(async (editorModuleUrl) => {
    const { createStudio } = await import(editorModuleUrl);

    const host = document.createElement('div');
    host.id = 'app';
    host.style.cssText = 'width: 100vw; height: 100vh';
    document.body.appendChild(host);

    const studio = await createStudio('#app', {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
      plugins: ['hwpctrl'],
      chrome: { statusbar: false },
    });

    const out = {};
    out.mountedInContainer = host.contains(studio.element);

    // 문서를 먼저 연다 — studio 는 빈 상태로 부팅하고, 플러그인은 문서 교체를 통지받는다.
    const sample = await fetch('/samples/table-001.hwp').then((r) => r.arrayBuffer());
    out.loaded = await studio.loadFile(sample, 'table-001.hwp');

    // ── 커맨드 질의·실행 ──
    const commands = await studio.commands.list();
    const menu = await studio.commands.menuModel();
    out.commandCount = commands.length;
    out.menuTop = menu.map((m) => m.menuId);
    out.hasMenuOnlyGap = commands.some((c) => c.id === 'field:edit');

    // ── chrome 토글 ──
    out.chromeInitial = await studio.chrome.get();
    out.chromeAfterHide = await studio.chrome.set({ menu: false, toolbar: false });
    // 숨긴 뒤에도 커맨드는 살아 있어야 한다 (헤드리스 구성)
    out.executeWhileHidden = await studio.commands.execute('edit:select-all');
    out.chromeRestored = await studio.chrome.set({ menu: true, toolbar: true, statusbar: true });

    // ── 플러그인 ──
    out.pluginList = await studio.plugins.list();

    // ── hwpctrl 배치 ──
    // 판정은 **산출 바이트 내용**으로 한다. 길이는 정렬·패딩 때문에 안 변할 수 있다
    // (실측: 10752 → 10752). 텍스트 읽기도 함께 걸어 두 축으로 본다.
    const sameBytes = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);
    const pagesBefore = await studio.hwpctrl.call('PageCount', []);
    const bytesBefore = await studio.hwpctrl.exportBytes();

    const batchResult = await studio.hwpctrl.batch((h) => {
      h.SetTextFile('브리지에서 씀', 'TEXT', '');
      h.GetPos();
    });
    const bytesAfter = await studio.hwpctrl.exportBytes();
    out.textAfter = await studio.hwpctrl.call('GetTextFile', ['UNICODE', '']);

    await studio.hwpctrl.undo();
    const bytesAfterUndo = await studio.hwpctrl.exportBytes();

    out.pagesBefore = pagesBefore;
    out.batchLen = Array.isArray(batchResult) ? batchResult.length : -1;
    out.wroteText = !sameBytes(bytesBefore, bytesAfter);
    out.undoneText = sameBytes(bytesBefore, bytesAfterUndo);

    // ── 바이트 산출 ──
    const bytes = await studio.hwpctrl.exportBytes();
    out.bytesLen = bytes?.length ?? 0;
    out.bytesIsBinary = bytes instanceof Uint8Array;

    // ── destroy 회수 + 재생성 ──
    const iframeEl = studio.element;
    studio.destroy();
    out.iframeDetached = !document.body.contains(iframeEl);

    const again = await createStudio('#app', {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
    });
    out.recreated = host.contains(again.element);
    out.recreatedPages = await again.pageCount();
    again.destroy();
    out.cleanedUp = host.children.length === 0;

    return out;
  }, EDITOR_MODULE_URL);

  assert(result.mountedInContainer, 'TC1: 지정한 div 안에 심긴다');
  assert(result.commandCount > 100, `TC2: 부모에서 커맨드 질의 (${result.commandCount}개)`);
  assert(result.menuTop.length === 8, `TC2: 메뉴 모델 (${result.menuTop.join(',')})`);
  assert(result.hasMenuOnlyGap, 'TC2: 메뉴에 없는 커맨드도 목록에 포함');

  assert(result.chromeInitial.statusbar === false, 'TC3: chrome 초기값 적용 (statusbar 숨김)');
  assert(result.chromeAfterHide.menu === false && result.chromeAfterHide.toolbar === false,
    `TC3: chrome 토글 (${JSON.stringify(result.chromeAfterHide)})`);
  assert(result.executeWhileHidden.ok === true, 'TC3: 숨긴 상태에서도 커맨드 실행');
  assert(result.chromeRestored.menu === true, 'TC3: 복원');

  assert(result.pluginList.some((p) => p.id === 'hwpctrl'), 'TC4: hwpctrl 플러그인 로드됨');
  assert(result.pagesBefore >= 1, `TC4: 부모에서 hwpctrl 읽기 (${result.pagesBefore}쪽)`);
  assert(result.batchLen === 2, `TC4: 배치가 호출 수만큼 결과를 준다 (${result.batchLen})`);
  assert(result.wroteText, 'TC4: 배치 편집이 문서 바이트를 바꿈');
  assert(typeof result.textAfter === 'string' && result.textAfter.includes('브리지에서 씀'),
    `TC4: GetTextFile 로도 확인된다 (${String(result.textAfter).slice(0, 30)}…)`);
  assert(result.undoneText, 'TC4: undo 1회로 원래 바이트 복원');

  assert(result.bytesIsBinary && result.bytesLen > 0,
    `TC5: 바이트가 부모까지 온다 (${result.bytesLen} bytes, binary=${result.bytesIsBinary})`);

  assert(result.iframeDetached, 'TC6: destroy 가 iframe 을 걷어감');
  assert(result.recreated && result.recreatedPages >= 0, 'TC6: 같은 컨테이너에 재생성');
  assert(result.cleanedUp, 'TC6: 두 번째 destroy 후 컨테이너 비워짐');
});
