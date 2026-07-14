/**
 * 저장 출력 포맷 선택 (file:save-as-hwp / file:save-as-hwpx) E2E — #1613
 *
 * 검증:
 *   1. HWP 문서 → "HWPX 형식으로 저장" → application/hwp+zip blob(PK 매직) 생성.
 *   2. HWPX 문서 → "HWP 형식으로 저장" → application/x-hwp blob(CFB 매직 D0CF11E0) 생성.
 *   3. 산출 blob 을 다시 loadDocument 했을 때 정상 재오픈(페이지 수 1 이상).
 *
 * 실행: node e2e/save-as-format.test.mjs --mode=headless
 */
import { runTest, assert, loadHwpFile } from './helpers.mjs';

// blob 캡처 훅 — showSaveFilePicker 를 모킹해 FS Access 경로로 blob 을 잡는다(폴백 다이얼로그 우회).
async function installSaveHooks(page) {
  await page.evaluate(() => {
    window.__alerts = [];
    window.alert = (m) => { window.__alerts.push(String(m)); };
    window.__savedBlob = null;
    // FS Access picker 를 가짜 핸들로 모킹: createWritable().write(blob) 에서 blob 캡처.
    window.showSaveFilePicker = async (options) => ({
      name: options?.suggestedName || 'saved.hwp',
      async createWritable() {
        return {
          async write(blob) { window.__savedBlob = blob; },
          async close() {},
        };
      },
    });
    // 폴백 download 경로도 혹시 모를 대비로 캡처.
    URL.createObjectURL = (blob) => { window.__savedBlob = blob; return 'blob:captured'; };
    HTMLAnchorElement.prototype.click = function () { /* no-op */ };
  });
}

// 파일 메뉴 열고 지정 명령 항목 클릭.
async function clickFileMenuItem(page, cmd) {
  return page.evaluate((cmdId) => {
    const fileItem = [...document.querySelectorAll('#menu-bar .menu-item')]
      .find((el) => (el.textContent || '').includes('파일'));
    const title = fileItem?.querySelector('.menu-title');
    if (!title) return { ok: false, reason: '파일 메뉴 없음' };
    title.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
    const item = document.querySelector(`.md-item[data-cmd="${cmdId}"]`);
    if (!item) return { ok: false, reason: `${cmdId} 항목 없음` };
    const disabled = item.classList.contains('disabled');
    item.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    return { ok: true, disabled };
  }, cmd);
}

async function capturedBlobInfo(page) {
  return page.evaluate(async () => {
    const blob = window.__savedBlob;
    if (!blob) return { captured: false };
    const buf = new Uint8Array(await blob.arrayBuffer());
    const head = Array.from(buf.slice(0, 4));
    const info = window.__wasm?.loadDocument(buf, 'saved.bin');
    return { captured: true, type: blob.type, len: buf.length, head, reopenPages: info?.pageCount };
  });
}

runTest('HWP 문서 → HWPX 형식으로 저장 (file:save-as-hwpx)', async ({ page }) => {
  await loadHwpFile(page, 'biz_plan.hwp');
  const fmt = await page.evaluate(() => window.__wasm?.getSourceFormat?.());
  assert(fmt === 'hwp', `HWP 출처여야 함 (current: ${fmt})`);

  await installSaveHooks(page);
  const t = await clickFileMenuItem(page, 'file:save-as-hwpx');
  assert(t.ok, `메뉴 항목 클릭 (${t.reason || ''})`);
  assert(!t.disabled, 'HWPX 형식으로 저장 항목이 활성이어야 함');

  await page.waitForFunction(() => window.__savedBlob !== null, { timeout: 8000 });
  const r = await capturedBlobInfo(page);
  assert(r.captured, '저장 blob 캡처');
  assert(r.type === 'application/hwp+zip', `MIME application/hwp+zip (current: ${r.type})`);
  // PK 매직 (ZIP): 0x50 0x4B
  assert(r.head[0] === 0x50 && r.head[1] === 0x4B, `PK(ZIP) 매직 (head: ${r.head})`);
  assert((r.reopenPages ?? 0) >= 1, `재오픈 페이지 1 이상 (current: ${r.reopenPages})`);
});

runTest('HWPX 문서 → HWP 형식으로 저장 (file:save-as-hwp)', async ({ page }) => {
  await loadHwpFile(page, 'hwpx/footnote-01.hwpx');
  const fmt = await page.evaluate(() => window.__wasm?.getSourceFormat?.());
  assert(fmt === 'hwpx', `HWPX 출처여야 함 (current: ${fmt})`);

  await installSaveHooks(page);
  const t = await clickFileMenuItem(page, 'file:save-as-hwp');
  assert(t.ok, `메뉴 항목 클릭 (${t.reason || ''})`);
  assert(!t.disabled, 'HWP 형식으로 저장 항목이 활성이어야 함');

  await page.waitForFunction(() => window.__savedBlob !== null, { timeout: 8000 });
  const r = await capturedBlobInfo(page);
  assert(r.captured, '저장 blob 캡처');
  assert(r.type === 'application/x-hwp', `MIME application/x-hwp (current: ${r.type})`);
  // CFB(OLE) 매직: D0 CF 11 E0
  assert(r.head[0] === 0xd0 && r.head[1] === 0xcf && r.head[2] === 0x11 && r.head[3] === 0xe0,
    `CFB(OLE) 매직 D0CF11E0 (head: ${r.head})`);
  assert((r.reopenPages ?? 0) >= 1, `재오픈 페이지 1 이상 (current: ${r.reopenPages})`);
});
